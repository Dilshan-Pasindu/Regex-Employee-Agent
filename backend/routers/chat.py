# =============================================================================
# routers/chat.py — POST /api/chat  (core agentic endpoint)
# Phase 1: MySQL persistence for conversations & messages
# Phase 2: MySQL context persistence with optimistic concurrency control
# Phase 3: Per-conversation distributed locking, request idempotency & message lifecycle
# =============================================================================

from __future__ import annotations

import asyncio
import json
import logging
import uuid
from datetime import datetime, timezone
from typing import Dict, List, Any, Optional

from fastapi import APIRouter, Depends, HTTPException, Request
import httpx
from sqlalchemy.ext.asyncio import AsyncSession

from db import get_db
from models import ChatRequest, ChatResponse, StepTrace, ConversationEntry
from services import conversation_service, reliability_service, outbox_service

router = APIRouter(prefix="/api/chat", tags=["chat"])

# In-memory conversation store still used for backward compatibility
_conversations: Dict[str, List[ConversationEntry]] = {}

DEFAULT_USER = "default_user"


@router.post("", response_model=ChatResponse)
async def chat(request: Request, body: ChatRequest, db: AsyncSession = Depends(get_db)):
    from services import rust_client  # local import to allow env override in tests

    # 1. Resolve session and request IDs
    session_id = body.session_id or body.conversation_id or str(uuid.uuid4())
    request_id = body.request_id or f"req_{uuid.uuid4().hex}"
    worker_id = f"worker_{uuid.uuid4().hex[:12]}"
    correlation_id = (
        getattr(request.state, "correlation_id", None)
        or request.headers.get("X-Correlation-ID")
        or f"corr_{uuid.uuid4().hex[:12]}"
    )
    logging.info("Processing chat request %s (correlation_id=%s) for session=%s", request_id, correlation_id, session_id)

    # 2. Check / create conversation in MySQL
    conv = await conversation_service.get_conversation(db, session_id, DEFAULT_USER)
    if conv is None:
        conv = await conversation_service.create_conversation(
            db, DEFAULT_USER, body.prompt, conversation_id=session_id
        )

    # 3. Idempotency Check & Duplicate Prevention
    try:
        idemp_status, cached_response = await reliability_service.get_or_create_idempotency_record(
            db=db,
            request_id=request_id,
            user_id=DEFAULT_USER,
            conversation_id=conv.conversation_id,
            prompt=body.prompt,
            confirmed=body.confirmed,
        )
    except ValueError as err:
        raise HTTPException(status_code=400, detail=str(err))

    # Replay cached response immediately if already completed
    if idemp_status == "COMPLETED" and cached_response:
        logging.info("Idempotency match: returning cached response for %s", request_id)
        steps = [
            StepTrace(step=s["step"], label=s["label"], value=s["value"])
            for s in cached_response.get("steps", [])
        ]
        return ChatResponse(
            session_id=cached_response.get("session_id", session_id),
            prompt=cached_response.get("prompt", body.prompt),
            intent=cached_response.get("intent", "unknown"),
            tool=cached_response.get("tool"),
            params=cached_response.get("params", {}),
            steps=steps,
            result=cached_response.get("result"),
            success=cached_response.get("success", True),
            requires_confirmation=cached_response.get("requires_confirmation"),
            message=cached_response.get("message"),
            suggestions=cached_response.get("suggestions"),
            request_id=request_id,
            status="COMPLETED",
            correlation_id=correlation_id,
            execution_plan=cached_response.get("execution_plan"),
        )

    if idemp_status == "PROCESSING":
        # Another worker is actively processing this exact request
        raise HTTPException(
            status_code=409,
            detail="This request is currently being processed. Please wait.",
        )

    # 4. Acquire Per-Conversation Distributed Lock (Serialize within conversation)
    lock_acquired = False
    fencing_token = 0
    max_lock_attempts = 4
    for attempt in range(max_lock_attempts):
        lock_acquired, fencing_token, status_desc = await reliability_service.acquire_conversation_lock(
            db=db,
            conversation_id=conv.conversation_id,
            lock_owner=worker_id,
            lease_seconds=35,
        )
        if lock_acquired:
            break
        if attempt < max_lock_attempts - 1:
            await asyncio.sleep(0.5)

    if not lock_acquired:
        raise HTTPException(
            status_code=429,
            detail=f"Conversation {conv.conversation_id} is currently busy processing another request ({status_desc}). Please retry in a few seconds.",
        )

    start_time = datetime.now(timezone.utc)

    try:
        # 5. Persist the user message with PROCESSING lifecycle status
        await conversation_service.add_message(
            db=db,
            conversation_id=conv.conversation_id,
            role="user",
            content=body.prompt,
            request_id=request_id,
            status="PROCESSING",
            processing_started_at=start_time,
        )

        # Emit Outbox event for message receipt
        await outbox_service.record_outbox_event(
            db=db,
            event_type="MESSAGE_RECEIVED",
            aggregate_id=conv.conversation_id,
            payload={"request_id": request_id, "prompt": body.prompt, "correlation_id": correlation_id},
        )
        await db.commit()

        # 6. Load persistent context & version (Phase 2)
        current_context, context_version, schema_version = await conversation_service.get_conversation_context(
            db, conv.conversation_id
        )

        # 7. Call Rust HTTP service with persistent context and correlation_id (Phase 2 & 5)
        try:
            rust_resp = await rust_client.process_prompt(
                prompt=body.prompt,
                confirmed=body.confirmed,
                session_id=session_id,
                context=current_context if current_context else None,
                context_version=context_version if context_version > 0 else None,
                correlation_id=correlation_id,
            )
        except httpx.ConnectError:
            await reliability_service.complete_idempotency_record(
                db, request_id, None, status="FAILED"
            )
            raise HTTPException(
                status_code=503,
                detail="Rust service is not reachable. Make sure to start it with: "
                       "cargo run -- --server  (in rust-client/)",
            )
        except httpx.HTTPStatusError as exc:
            await reliability_service.complete_idempotency_record(
                db, request_id, None, status="FAILED"
            )
            raise HTTPException(status_code=exc.response.status_code, detail=exc.response.text)

        # 8. Persist updated context back to MySQL with optimistic concurrency check
        returned_context = rust_resp.get("context")
        if returned_context is not None:
            try:
                new_ver = await conversation_service.save_conversation_context(
                    db,
                    conv.conversation_id,
                    returned_context,
                    expected_version=context_version,
                    schema_version=schema_version,
                )
            except conversation_service.ConcurrencyConflictError:
                # Safe concurrency resolution: reload newest version and commit
                _, latest_version, _ = await conversation_service.get_conversation_context(
                    db, conv.conversation_id
                )
                new_ver = await conversation_service.save_conversation_context(
                    db,
                    conv.conversation_id,
                    returned_context,
                    expected_version=latest_version,
                    schema_version=schema_version,
                )
            await outbox_service.record_outbox_event(
                db=db,
                event_type="CONTEXT_UPDATED",
                aggregate_id=conv.conversation_id,
                payload={"version": new_ver, "context_version": context_version},
            )

        completed_time = datetime.now(timezone.utc)
        agent_text = rust_resp.get("message") or "Done."

        # Parse step traces
        steps = [
            StepTrace(step=s["step"], label=s["label"], value=s["value"])
            for s in rust_resp.get("steps", [])
        ]

        response = ChatResponse(
            session_id=session_id,
            prompt=body.prompt,
            intent=rust_resp.get("intent", "unknown"),
            tool=rust_resp.get("tool"),
            params=rust_resp.get("params", {}),
            steps=steps,
            result=rust_resp.get("result"),
            success=rust_resp.get("success", False),
            requires_confirmation=rust_resp.get("requires_confirmation"),
            message=rust_resp.get("message"),
            suggestions=rust_resp.get("suggestions"),
            request_id=request_id,
            status="COMPLETED",
            correlation_id=correlation_id,
            execution_plan=rust_resp.get("execution_plan"),
        )

        metadata_payload = {
            "session_id": session_id,
            "prompt": body.prompt,
            "intent": rust_resp.get("intent"),
            "tool": rust_resp.get("tool"),
            "params": rust_resp.get("params", {}),
            "steps": rust_resp.get("steps", []),
            "result": rust_resp.get("result"),
            "suggestions": rust_resp.get("suggestions"),
            "requires_confirmation": rust_resp.get("requires_confirmation"),
            "success": rust_resp.get("success", False),
            "request_id": request_id,
            "correlation_id": correlation_id,
            "execution_plan": rust_resp.get("execution_plan"),
        }
        metadata_str = json.dumps(metadata_payload)

        # 9. Persist assistant message as COMPLETED
        assistant_msg = await conversation_service.add_message(
            db=db,
            conversation_id=conv.conversation_id,
            role="assistant",
            content=agent_text,
            intent=rust_resp.get("intent"),
            success=rust_resp.get("success", False),
            metadata_json=metadata_str,
            request_id=request_id,
            status="COMPLETED",
            processing_started_at=start_time,
            processing_completed_at=completed_time,
        )

        # 10. Checkpoint context snapshot (Phase 5)
        # Snapshots created every 5 messages or on initial bootstrap turns (seq <= 2)
        active_context = returned_context if returned_context is not None else current_context
        active_version = new_ver if returned_context is not None else context_version
        if active_context and (assistant_msg.sequence_number % 5 == 0 or assistant_msg.sequence_number <= 2):
            await conversation_service.create_context_snapshot(
                db=db,
                conversation_id=conv.conversation_id,
                context_dict=active_context,
                context_version=active_version,
                message_sequence_number=assistant_msg.sequence_number,
                schema_version=schema_version,
            )
            await outbox_service.record_outbox_event(
                db=db,
                event_type="CONTEXT_SNAPSHOT_CREATED",
                aggregate_id=conv.conversation_id,
                payload={
                    "context_version": active_version,
                    "message_sequence_number": assistant_msg.sequence_number,
                    "correlation_id": correlation_id,
                },
            )

        # 11. Track operation record & Write Safety (Phase 6)
        requires_conf = rust_resp.get("requires_confirmation") is True
        is_write = (
            body.confirmed
            or requires_conf
            or (rust_resp.get("tool") in ["add_employee", "update_employee", "batch_update"])
        )
        op_type = "WRITE" if is_write else "READ"
        op_rec = await outbox_service.record_operation_start(
            db,
            request_id,
            conv.conversation_id,
            op_type,
            rust_resp.get("tool") or "none",
            rust_resp.get("params", {}),
        )

        if requires_conf and not body.confirmed:
            op_status = "PENDING_CONFIRMATION"
        else:
            op_status = "COMPLETED" if rust_resp.get("success") else "FAILED"

        await outbox_service.record_operation_complete(
            db, op_rec.operation_id, op_status, rust_resp.get("result")
        )

        # Emit Outbox Event based on execution vs confirmation
        if requires_conf and not body.confirmed:
            await outbox_service.record_outbox_event(
                db=db,
                event_type="OPERATION_AWAITING_CONFIRMATION",
                aggregate_id=conv.conversation_id,
                payload={
                    "request_id": request_id,
                    "tool": rust_resp.get("tool"),
                    "params": rust_resp.get("params", {}),
                    "correlation_id": correlation_id,
                },
            )
        else:
            await outbox_service.record_outbox_event(
                db=db,
                event_type="MESSAGE_COMPLETED",
                aggregate_id=conv.conversation_id,
                payload={
                    "request_id": request_id,
                    "intent": rust_resp.get("intent"),
                    "tool": rust_resp.get("tool"),
                    "success": rust_resp.get("success", False),
                    "correlation_id": correlation_id,
                },
            )
        await db.commit()

        # 10. Mark idempotency record COMPLETED with cached response
        await reliability_service.complete_idempotency_record(
            db=db,
            request_id=request_id,
            response_dict=response.dict(),
            status="COMPLETED",
        )

        # Keep in-memory store for backward compat
        history = _conversations.setdefault(session_id, [])
        history.append(ConversationEntry(role="user", content=body.prompt))
        history.append(ConversationEntry(
            role="agent",
            content=agent_text,
            intent=rust_resp.get("intent"),
            success=rust_resp.get("success", False),
        ))

        return response

    finally:
        # 11. Always release the conversation lock if we held it
        if lock_acquired:
            await reliability_service.release_conversation_lock(
                db=db,
                conversation_id=conv.conversation_id,
                lock_owner=worker_id,
                fencing_token=fencing_token,
            )


def get_conversation_store() -> Dict[str, List[ConversationEntry]]:
    """Expose store to the old conversations router (kept for backward compat)."""
    return _conversations
