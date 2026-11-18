# =============================================================================
# routers/conv_mgmt.py — Persistent conversation management API (Phase 1)
#
# POST   /api/conversations                    → create conversation
# GET    /api/conversations                    → list conversations
# GET    /api/conversations/search             → search
# GET    /api/conversations/{id}               → get conversation
# PATCH  /api/conversations/{id}               → rename
# POST   /api/conversations/{id}/archive       → archive
# DELETE /api/conversations/{id}               → soft delete
# GET    /api/conversations/{id}/messages      → paginated messages
# =============================================================================
from __future__ import annotations

from typing import Optional
from fastapi import APIRouter, Depends, HTTPException, Query
from pydantic import BaseModel, Field
from sqlalchemy.ext.asyncio import AsyncSession

from db import get_db
from services.conversation_service import (
    create_conversation,
    rename_conversation,
    archive_conversation,
    delete_conversation,
    get_conversation,
    get_messages,
    list_conversations,
    search_conversations,
)

router = APIRouter(prefix="/api/conversations", tags=["conversation-management"])

# ---------------------------------------------------------------------------
# Phase 1: single-user system. user_id is hardcoded.
# Phase 2+ will replace this with real JWT auth.
# ---------------------------------------------------------------------------
DEFAULT_USER = "default_user"


# ---------------------------------------------------------------------------
# Pydantic response schemas
# ---------------------------------------------------------------------------

class ConvOut(BaseModel):
    conversation_id: str
    title: str
    created_at: str
    updated_at: str
    last_message_at: Optional[str]
    is_archived: bool

    @classmethod
    def from_orm(cls, c) -> "ConvOut":
        return cls(
            conversation_id=c.conversation_id,
            title=c.title,
            created_at=c.created_at.isoformat(),
            updated_at=c.updated_at.isoformat(),
            last_message_at=c.last_message_at.isoformat() if c.last_message_at else None,
            is_archived=c.is_archived,
        )


class MsgOut(BaseModel):
    message_id: str
    sequence_number: int
    role: str
    content: str
    intent: Optional[str]
    success: Optional[bool]
    created_at: str

    @classmethod
    def from_orm(cls, m) -> "MsgOut":
        return cls(
            message_id=m.message_id,
            sequence_number=m.sequence_number,
            role=m.role,
            content=m.content,
            intent=m.intent,
            success=m.success,
            created_at=m.created_at.isoformat(),
        )


class CreateConvBody(BaseModel):
    title: Optional[str] = Field(None, max_length=255)


class RenameBody(BaseModel):
    title: str = Field(..., min_length=1, max_length=255)


# ---------------------------------------------------------------------------
# Endpoints
# ---------------------------------------------------------------------------

@router.post("", status_code=201)
async def create_conv(body: CreateConvBody, db: AsyncSession = Depends(get_db)):
    conv = await create_conversation(db, DEFAULT_USER, body.title or None)
    return ConvOut.from_orm(conv)


@router.get("")
async def list_convs(
    page: int = Query(1, ge=1),
    page_size: int = Query(20, ge=1, le=100),
    include_archived: bool = Query(False),
    db: AsyncSession = Depends(get_db),
):
    convs = await list_conversations(db, DEFAULT_USER, page, page_size, include_archived)
    return {"conversations": [ConvOut.from_orm(c) for c in convs], "page": page, "page_size": page_size}


@router.get("/search")
async def search_convs(
    q: str = Query(..., min_length=1),
    page: int = Query(1, ge=1),
    page_size: int = Query(20, ge=1, le=100),
    db: AsyncSession = Depends(get_db),
):
    convs = await search_conversations(db, DEFAULT_USER, q, page, page_size)
    return {"conversations": [ConvOut.from_orm(c) for c in convs]}


@router.get("/{conversation_id}")
async def get_conv(conversation_id: str, db: AsyncSession = Depends(get_db)):
    conv = await get_conversation(db, conversation_id, DEFAULT_USER)
    if not conv:
        raise HTTPException(404, "Conversation not found")
    return ConvOut.from_orm(conv)


@router.patch("/{conversation_id}")
async def rename_conv(conversation_id: str, body: RenameBody, db: AsyncSession = Depends(get_db)):
    conv = await rename_conversation(db, conversation_id, DEFAULT_USER, body.title)
    if not conv:
        raise HTTPException(404, "Conversation not found")
    return ConvOut.from_orm(conv)


@router.post("/{conversation_id}/archive")
async def archive_conv(conversation_id: str, db: AsyncSession = Depends(get_db)):
    ok = await archive_conversation(db, conversation_id, DEFAULT_USER)
    if not ok:
        raise HTTPException(404, "Conversation not found")
    return {"message": "Conversation archived"}


@router.delete("/{conversation_id}")
async def delete_conv(conversation_id: str, db: AsyncSession = Depends(get_db)):
    ok = await delete_conversation(db, conversation_id, DEFAULT_USER)
    if not ok:
        raise HTTPException(404, "Conversation not found")
    return {"message": "Conversation deleted"}


@router.get("/{conversation_id}/messages")
async def get_conv_messages(
    conversation_id: str,
    page: int = Query(1, ge=1),
    page_size: int = Query(50, ge=1, le=200),
    db: AsyncSession = Depends(get_db),
):
    msgs = await get_messages(db, conversation_id, DEFAULT_USER, page, page_size)
    return {
        "conversation_id": conversation_id,
        "messages": [MsgOut.from_orm(m) for m in msgs],
        "page": page,
        "page_size": page_size,
    }
