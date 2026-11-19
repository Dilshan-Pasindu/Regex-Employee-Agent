# =============================================================================
# routers/chat.py — POST /api/chat  (core agentic endpoint)
# Phase 1: adds MySQL persistence for messages alongside existing in-memory store
# =============================================================================

from __future__ import annotations

import uuid
from datetime import datetime, timezone
from typing import Dict, List, Any

from fastapi import APIRouter, Depends, HTTPException, Request
import httpx
from sqlalchemy.ext.asyncio import AsyncSession

from db import get_db
from models import ChatRequest, ChatResponse, StepTrace, ConversationEntry
from services import conversation_service

router = APIRouter(prefix="/api/chat", tags=["chat"])

# In-memory conversation store still used for Rust session_id continuity
_conversations: Dict[str, List[ConversationEntry]] = {}


@router.post("", response_model=ChatResponse)
async def chat(request: Request, body: ChatRequest, db: AsyncSession = Depends(get_db)):
    from services import rust_client  # local import to allow env override in tests

    # Resolve / create session
    session_id = body.session_id or str(uuid.uuid4())

    # --- Ensure a Conversation row exists in MySQL for this session_id ---
    # We use the session_id as the conversation_id directly (they are the same UUID)
    conv = await conversation_service.get_conversation(db, session_id, "default_user")
    if conv is None:
        conv = await conversation_service.create_conversation(
            db, "default_user", body.prompt, conversation_id=session_id
        )

    # Persist the user message
    await conversation_service.add_message(
        db, conv.conversation_id, "user", body.prompt
    )

    # Phase 2: Load current persistent conversation context from MySQL
    current_context, context_version, schema_version = await conversation_service.get_conversation_context(
        db, conv.conversation_id
    )

    # Call Rust HTTP service with persistent context
    try:
        rust_resp = await rust_client.process_prompt(
            body.prompt,
            body.confirmed,
            session_id,
            context=current_context if current_context else None,
            context_version=context_version if context_version > 0 else None,
        )
    except httpx.ConnectError:
        raise HTTPException(
            status_code=503,
            detail="Rust service is not reachable. Make sure to start it with: "
                   "cargo run -- --server  (in rust-client/)",
        )
    except httpx.HTTPStatusError as exc:
        raise HTTPException(status_code=exc.response.status_code, detail=exc.response.text)

    # Phase 2: Persist updated context back to MySQL with optimistic concurrency control
    returned_context = rust_resp.get("context")
    if returned_context is not None:
        try:
            await conversation_service.save_conversation_context(
                db,
                conv.conversation_id,
                returned_context,
                expected_version=context_version,
                schema_version=schema_version,
            )
        except conversation_service.ConcurrencyConflictError:
            # Safe concurrency resolution: reload newest version and save latest state
            _, latest_version, _ = await conversation_service.get_conversation_context(
                db, conv.conversation_id
            )
            await conversation_service.save_conversation_context(
                db,
                conv.conversation_id,
                returned_context,
                expected_version=latest_version,
                schema_version=schema_version,
            )

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
    )

    # Persist the assistant message to MySQL with full metadata for rich UI reconstruction
    agent_text = rust_resp.get("message") or "Done."
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
    }
    import json
    metadata_str = json.dumps(metadata_payload)

    await conversation_service.add_message(
        db,
        conv.conversation_id,
        "assistant",
        agent_text,
        intent=rust_resp.get("intent"),
        success=rust_resp.get("success", False),
        metadata_json=metadata_str,
    )

    # Also keep in-memory for backward compat with the old conversations router
    history = _conversations.setdefault(session_id, [])
    history.append(ConversationEntry(role="user", content=body.prompt))
    history.append(ConversationEntry(
        role="agent",
        content=agent_text,
        intent=rust_resp.get("intent"),
        success=rust_resp.get("success", False),
    ))

    return response


def get_conversation_store() -> Dict[str, List[ConversationEntry]]:
    """Expose store to the old conversations router (kept for backward compat)."""
    return _conversations
