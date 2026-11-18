# =============================================================================
# services/conversation_service.py — CQRS-style service for conversation CRUD
# Commands: create, rename, pin, delete
# Queries: list, get, get_messages, search
# =============================================================================
from __future__ import annotations

import re
from datetime import datetime, timezone
from typing import Optional

from sqlalchemy import select, func, or_, update
from sqlalchemy.ext.asyncio import AsyncSession

from backend_models import Conversation, Message


def _utcnow() -> datetime:
    return datetime.now(timezone.utc)


# ---------------------------------------------------------------------------
# Title generation — deterministic, no LLM
# ---------------------------------------------------------------------------
_TITLE_PATTERNS = [
    (r"(?i)show\s+all\s+employees", "All Employees"),
    (r"(?i)find\s+employee\s+(\d+)", lambda m: f"Employee {m.group(1)}"),
    (r"(?i)employees?\s+in\s+(\w+)", lambda m: f"{m.group(1).title()} Department"),
    (r"(?i)add\s+employee", "Add Employee"),
    (r"(?i)update\s+employee\s+(\d+)", lambda m: f"Update Employee {m.group(1)}"),
    (r"(?i)salary\s+of\s+employee", "Employee Salary"),
    (r"(?i)count\s+employees?", "Employee Count"),
    (r"(?i)average\s+salary", "Average Salary"),
    (r"(?i)departments?", "Department Info"),
]

def generate_title(first_prompt: str) -> str:
    for pattern, result in _TITLE_PATTERNS:
        m = re.search(pattern, first_prompt)
        if m:
            return result(m) if callable(result) else result
    # Fallback: capitalise first 40 chars
    trimmed = first_prompt.strip()[:40]
    return trimmed.title() if trimmed else "New Conversation"


# ---------------------------------------------------------------------------
# Commands
# ---------------------------------------------------------------------------

async def create_conversation(
    db: AsyncSession,
    user_id: str,
    first_prompt: str | None = None,
    conversation_id: str | None = None,
) -> Conversation:
    title = generate_title(first_prompt) if first_prompt else "New Conversation"
    kwargs: dict = {"user_id": user_id, "title": title}
    if conversation_id:
        kwargs["conversation_id"] = conversation_id
    conv = Conversation(**kwargs)
    db.add(conv)
    await db.commit()
    await db.refresh(conv)
    return conv


async def rename_conversation(db: AsyncSession, conversation_id: str, user_id: str, new_title: str) -> Conversation | None:
    conv = await _get_owned(db, conversation_id, user_id)
    if not conv:
        return None
    conv.title = new_title[:255]
    conv.updated_at = _utcnow()
    await db.commit()
    await db.refresh(conv)
    return conv


async def pin_conversation(
    db: AsyncSession, conversation_id: str, user_id: str, pin: bool
) -> Conversation | None:
    conv = await _get_owned(db, conversation_id, user_id)
    if not conv:
        return None
    if pin:
        # Count currently pinned chats for this user
        result = await db.execute(
            select(func.count()).where(
                Conversation.user_id == user_id,
                Conversation.is_pinned == True,
                Conversation.is_deleted == False,
            )
        )
        pinned_count = result.scalar() or 0
        # Only block if this conversation is not already pinned and would exceed 3
        if not conv.is_pinned and pinned_count >= 3:
            raise ValueError("max_pins")
    conv.is_pinned = pin
    conv.pinned_at = _utcnow() if pin else None
    conv.updated_at = _utcnow()
    await db.commit()
    await db.refresh(conv)
    return conv


async def delete_conversation(db: AsyncSession, conversation_id: str, user_id: str) -> bool:
    conv = await _get_owned(db, conversation_id, user_id)
    if not conv:
        return False
    conv.is_deleted = True
    conv.updated_at = _utcnow()
    await db.commit()
    return True


async def add_message(
    db: AsyncSession,
    conversation_id: str,
    role: str,
    content: str,
    intent: str | None = None,
    success: bool | None = None,
    metadata_json: str | None = None,
) -> Message:
    # Get next sequence number
    result = await db.execute(
        select(func.count()).where(Message.conversation_id == conversation_id)
    )
    count = result.scalar() or 0

    msg = Message(
        conversation_id=conversation_id,
        sequence_number=count + 1,
        role=role,
        content=content,
        intent=intent,
        success=success,
        metadata_json=metadata_json,
    )
    db.add(msg)

    # Update conversation last_message_at and updated_at
    await db.execute(
        update(Conversation)
        .where(Conversation.conversation_id == conversation_id)
        .values(last_message_at=_utcnow(), updated_at=_utcnow())
    )
    await db.commit()
    await db.refresh(msg)
    return msg


# ---------------------------------------------------------------------------
# Queries
# ---------------------------------------------------------------------------

async def list_conversations(
    db: AsyncSession, user_id: str, page: int = 1, page_size: int = 20
) -> list[Conversation]:
    q = (
        select(Conversation)
        .where(Conversation.user_id == user_id)
        .where(Conversation.is_deleted == False)
        # Pinned first (is_pinned DESC), then pinned_at DESC for pinned group,
        # then last_message_at DESC / created_at DESC for unpinned group.
        # MySQL-compatible: no NULLS LAST needed since NULLs sort last on DESC.
        .order_by(
            Conversation.is_pinned.desc(),
            Conversation.pinned_at.desc(),
            Conversation.last_message_at.desc(),
            Conversation.created_at.desc(),
        )
        .offset((page - 1) * page_size)
        .limit(page_size)
    )
    result = await db.execute(q)
    return list(result.scalars().all())


async def get_conversation(db: AsyncSession, conversation_id: str, user_id: str) -> Conversation | None:
    return await _get_owned(db, conversation_id, user_id)


async def get_messages(
    db: AsyncSession,
    conversation_id: str,
    user_id: str,
    page: int = 1,
    page_size: int = 50,
) -> list[Message]:
    # Verify ownership first
    conv = await _get_owned(db, conversation_id, user_id)
    if not conv:
        return []
    result = await db.execute(
        select(Message)
        .where(Message.conversation_id == conversation_id)
        .order_by(Message.sequence_number)
        .offset((page - 1) * page_size)
        .limit(page_size)
    )
    return list(result.scalars().all())


async def search_conversations(
    db: AsyncSession, user_id: str, query: str, page: int = 1, page_size: int = 20
) -> list[Conversation]:
    q = (
        select(Conversation)
        .where(Conversation.user_id == user_id)
        .where(Conversation.is_deleted == False)
        .where(Conversation.title.ilike(f"%{query}%"))
        .order_by(
            Conversation.is_pinned.desc(),
            Conversation.updated_at.desc(),
        )
        .offset((page - 1) * page_size)
        .limit(page_size)
    )
    result = await db.execute(q)
    return list(result.scalars().all())


# ---------------------------------------------------------------------------
# Internal helpers
# ---------------------------------------------------------------------------

async def _get_owned(db: AsyncSession, conversation_id: str, user_id: str) -> Conversation | None:
    result = await db.execute(
        select(Conversation).where(
            Conversation.conversation_id == conversation_id,
            Conversation.user_id == user_id,
            Conversation.is_deleted == False,
        )
    )
    return result.scalar_one_or_none()
