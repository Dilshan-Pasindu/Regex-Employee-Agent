# =============================================================================
# routers/chat.py — POST /api/chat  (core agentic endpoint)
# =============================================================================

from __future__ import annotations

import uuid
from datetime import datetime, timezone
from typing import Dict, List, Any

from fastapi import APIRouter, HTTPException, Request
import httpx

from models import ChatRequest, ChatResponse, StepTrace, ConversationEntry

router = APIRouter(prefix="/api/chat", tags=["chat"])

# In-memory conversation store: {session_id: [ConversationEntry]}
_conversations: Dict[str, List[ConversationEntry]] = {}


@router.post("", response_model=ChatResponse)
async def chat(request: Request, body: ChatRequest):
    from services import rust_client  # local import to allow env override in tests

    # Resolve / create session
    session_id = body.session_id or str(uuid.uuid4())

    # Call Rust HTTP service
    try:
        rust_resp = await rust_client.process_prompt(body.prompt, body.confirmed)
    except httpx.ConnectError:
        raise HTTPException(
            status_code=503,
            detail="Rust service is not reachable. Make sure to start it with: "
                   "cargo run -- --server  (in rust-client/)",
        )
    except httpx.HTTPStatusError as exc:
        raise HTTPException(status_code=exc.response.status_code, detail=exc.response.text)

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
    )

    # Persist conversation history
    history = _conversations.setdefault(session_id, [])
    history.append(ConversationEntry(role="user", content=body.prompt))

    agent_text = (
        rust_resp.get("message")
        or (str(rust_resp.get("result")) if rust_resp.get("result") else "Done.")
    )
    history.append(ConversationEntry(
        role="agent",
        content=agent_text,
        intent=rust_resp.get("intent"),
        success=rust_resp.get("success", False),
    ))

    return response


def get_conversation_store() -> Dict[str, List[ConversationEntry]]:
    """Expose store to the conversations router."""
    return _conversations
