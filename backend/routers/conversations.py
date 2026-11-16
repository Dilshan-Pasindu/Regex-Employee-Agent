# =============================================================================
# routers/conversations.py — GET /api/conversations/{session_id}
#                             DELETE /api/conversations/{session_id}
# =============================================================================

from fastapi import APIRouter, HTTPException
from models import ConversationResponse
from routers.chat import get_conversation_store

router = APIRouter(prefix="/api/conversations", tags=["conversations"])


@router.get("/{session_id}", response_model=ConversationResponse)
async def get_conversation(session_id: str):
    store = get_conversation_store()
    history = store.get(session_id)
    if history is None:
        raise HTTPException(404, f"Session '{session_id}' not found")
    return ConversationResponse(
        session_id=session_id,
        history=history,
        total_messages=len(history),
    )


@router.delete("/{session_id}")
async def clear_conversation(session_id: str):
    store = get_conversation_store()
    if session_id not in store:
        raise HTTPException(404, f"Session '{session_id}' not found")
    del store[session_id]
    return {"message": f"Session '{session_id}' cleared."}


@router.get("", summary="List all active session IDs")
async def list_sessions():
    store = get_conversation_store()
    return {
        "sessions": list(store.keys()),
        "count": len(store),
    }
