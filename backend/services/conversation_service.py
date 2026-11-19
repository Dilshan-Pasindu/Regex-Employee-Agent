# =============================================================================
# services/conversation_service.py — CQRS-style service for conversation CRUD
# Commands: create, rename, pin, delete
# Queries: list, get, get_messages, search
# =============================================================================
from __future__ import annotations

import json
import re
from datetime import datetime, timezone
from typing import Optional

from sqlalchemy import select, func, or_, update, delete
from sqlalchemy.ext.asyncio import AsyncSession

from backend_models import Conversation, Message, ConversationContext, ContextSnapshot


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
    # Clean up associated context
    await db.execute(
        delete(ConversationContext).where(
            ConversationContext.conversation_id == conversation_id
        )
    )
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
    request_id: str | None = None,
    status: str = "COMPLETED",
    error_code: str | None = None,
    error_message: str | None = None,
    processing_started_at: datetime | None = None,
    processing_completed_at: datetime | None = None,
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
        request_id=request_id,
        status=status,
        error_code=error_code,
        error_message=error_message,
        processing_started_at=processing_started_at,
        processing_completed_at=processing_completed_at,
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
# Context Persistence & Optimistic Concurrency Control (Phase 2)
# ---------------------------------------------------------------------------

class ConcurrencyConflictError(Exception):
    def __init__(self, conversation_id: str, expected_version: int, actual_version: int):
        super().__init__(
            f"Conflict: Conversation {conversation_id} context was modified concurrently. "
            f"Expected version {expected_version}, but found {actual_version}."
        )
        self.conversation_id = conversation_id
        self.expected_version = expected_version
        self.actual_version = actual_version


async def get_conversation_context(
    db: AsyncSession, conversation_id: str
) -> tuple[dict, int, int]:
    """
    Returns (context_dict, version, schema_version).
    If no context row exists yet, returns ({}, 0, 1).
    """
    result = await db.execute(
        select(ConversationContext).where(
            ConversationContext.conversation_id == conversation_id
        )
    )
    ctx_row = result.scalar_one_or_none()
    if not ctx_row:
        return {}, 0, 1
    try:
        data = json.loads(ctx_row.context_json) if ctx_row.context_json else {}
    except Exception:
        data = {}
    return data, ctx_row.version, ctx_row.schema_version


async def save_conversation_context(
    db: AsyncSession,
    conversation_id: str,
    context_dict: dict,
    expected_version: int,
    schema_version: int = 1,
) -> int:
    """
    Saves conversation context using optimistic concurrency control.
    - If expected_version == 0, creates the initial row at version 1.
    - If expected_version > 0, conditionally updates row where version == expected_version.
    Raises ConcurrencyConflictError if a concurrent update modified the row.
    Returns the new version number.
    """
    now = _utcnow()
    json_str = json.dumps(context_dict)

    if expected_version == 0:
        existing = await db.execute(
            select(ConversationContext).where(
                ConversationContext.conversation_id == conversation_id
            )
        )
        existing_row = existing.scalar_one_or_none()
        if existing_row:
            raise ConcurrencyConflictError(conversation_id, expected_version, existing_row.version)

        ctx_row = ConversationContext(
            conversation_id=conversation_id,
            version=1,
            schema_version=schema_version,
            context_json=json_str,
            created_at=now,
            updated_at=now,
        )
        db.add(ctx_row)
        await db.commit()
        return 1

    res = await db.execute(
        update(ConversationContext)
        .where(
            ConversationContext.conversation_id == conversation_id,
            ConversationContext.version == expected_version,
        )
        .values(
            context_json=json_str,
            version=ConversationContext.version + 1,
            schema_version=schema_version,
            updated_at=now,
        )
    )
    if res.rowcount == 0:
        actual_res = await db.execute(
            select(ConversationContext.version).where(
                ConversationContext.conversation_id == conversation_id
            )
        )
        actual_version = actual_res.scalar() or 0
        raise ConcurrencyConflictError(conversation_id, expected_version, actual_version)

    await db.commit()
    return expected_version + 1


# ---------------------------------------------------------------------------
# Context Snapshots & Checkpoints (Phase 5)
# ---------------------------------------------------------------------------

async def create_context_snapshot(
    db: AsyncSession,
    conversation_id: str,
    context_dict: dict,
    context_version: int,
    message_sequence_number: int,
    schema_version: int = 1,
) -> ContextSnapshot:
    """Creates a deterministic snapshot/checkpoint of the conversation context."""
    now = _utcnow()
    snapshot = ContextSnapshot(
        conversation_id=conversation_id,
        context_version=context_version,
        message_sequence_number=message_sequence_number,
        schema_version=schema_version,
        snapshot_json=json.dumps(context_dict, default=str),
        created_at=now,
    )
    db.add(snapshot)
    await db.commit()
    await db.refresh(snapshot)
    return snapshot


async def get_latest_context_snapshot(
    db: AsyncSession, conversation_id: str
) -> tuple[dict, int, int] | None:
    """
    Retrieves the most recent context snapshot for a conversation.
    Returns (snapshot_dict, context_version, message_sequence_number) or None.
    """
    res = await db.execute(
        select(ContextSnapshot)
        .where(ContextSnapshot.conversation_id == conversation_id)
        .order_by(ContextSnapshot.message_sequence_number.desc())
        .limit(1)
    )
    row = res.scalar_one_or_none()
    if not row:
        return None
    try:
        data = json.loads(row.snapshot_json)
    except Exception:
        data = {}
    return data, row.context_version, row.message_sequence_number


async def list_context_snapshots(
    db: AsyncSession, conversation_id: str, limit: int = 50
) -> list[ContextSnapshot]:
    """Retrieves context snapshots for a conversation ordered by message sequence number."""
    res = await db.execute(
        select(ContextSnapshot)
        .where(ContextSnapshot.conversation_id == conversation_id)
        .order_by(ContextSnapshot.message_sequence_number.asc())
        .limit(limit)
    )
    return list(res.scalars().all())


# ---------------------------------------------------------------------------
# Lightweight CQRS Facades (Phase 5)
# ---------------------------------------------------------------------------

class ConversationCommandService:
    create = staticmethod(create_conversation)
    rename = staticmethod(rename_conversation)
    pin = staticmethod(pin_conversation)
    delete = staticmethod(delete_conversation)
    add_message = staticmethod(add_message)
    save_context = staticmethod(save_conversation_context)
    create_snapshot = staticmethod(create_context_snapshot)


class ConversationQueryService:
    get = staticmethod(get_conversation)
    list = staticmethod(list_conversations)
    search = staticmethod(search_conversations)
    get_messages = staticmethod(get_messages)
    get_context = staticmethod(get_conversation_context)
    get_latest_snapshot = staticmethod(get_latest_context_snapshot)
    list_snapshots = staticmethod(list_context_snapshots)


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
