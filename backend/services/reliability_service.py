# =============================================================================
# services/reliability_service.py — Production Reliability & Coordination (Phase 3)
#
# 1. Distributed Conversation Locks with Leases & Fencing Tokens
# 2. Request Idempotency & Duplicate Prevention (SHA-256 hashing + cached replay)
# 3. Message Processing Lifecycle State Machine
# =============================================================================
from __future__ import annotations

import hashlib
import json
from datetime import datetime, timezone, timedelta
from typing import Optional, Tuple, Any, Dict

from sqlalchemy import select, update, delete, text
from sqlalchemy.ext.asyncio import AsyncSession

from backend_models import ConversationLock, IdempotencyRecord, Message


def _utcnow() -> datetime:
    return datetime.now(timezone.utc)


def _to_utc(dt: datetime) -> datetime:
    if dt.tzinfo is None:
        return dt.replace(tzinfo=timezone.utc)
    return dt


def compute_request_hash(prompt: str, confirmed: bool) -> str:
    """Computes deterministic SHA-256 hash for idempotency validation."""
    normalized = f"{prompt.strip()}:{confirmed}".encode("utf-8")
    return hashlib.sha256(normalized).hexdigest()


# ---------------------------------------------------------------------------
# 1. Conversation Locking & Fencing Tokens (MySQL Lease Mechanism)
# ---------------------------------------------------------------------------

async def acquire_conversation_lock(
    db: AsyncSession,
    conversation_id: str,
    lock_owner: str,
    lease_seconds: int = 30,
) -> Tuple[bool, int, str]:
    """
    Atomically acquires a per-conversation lock with a lease and fencing token.
    Uses MySQL atomic upsert logic:
      - If no lock row exists, inserts with fencing_token = 1.
      - If lock exists but lease has expired, increments fencing_token and takes ownership.
      - If lock exists and lease is active by another owner, rejects acquisition.
    Returns:
      (acquired: bool, fencing_token: int, status_message: str)
    """
    now = _utcnow()
    expires_at = now + timedelta(seconds=lease_seconds)

    # Atomic upsert statement in MySQL
    upsert_stmt = text("""
        INSERT INTO ConversationLocks (
            conversation_id, lock_owner, fencing_token, lease_expires_at, acquired_at, updated_at
        ) VALUES (
            :cid, :owner, 1, :expires, :now, :now
        ) ON DUPLICATE KEY UPDATE
            fencing_token = IF(lease_expires_at < :now, fencing_token + 1, fencing_token),
            lock_owner = IF(lease_expires_at < :now, :owner, lock_owner),
            acquired_at = IF(lease_expires_at < :now, :now, acquired_at),
            updated_at = :now,
            lease_expires_at = IF(lease_expires_at < :now, :expires, lease_expires_at)
    """)

    await db.execute(upsert_stmt, {
        "cid": conversation_id,
        "owner": lock_owner,
        "expires": expires_at,
        "now": now,
    })
    await db.commit()

    # Verify if we are the current active owner
    verify_stmt = select(ConversationLock).where(
        ConversationLock.conversation_id == conversation_id
    )
    res = await db.execute(verify_stmt)
    lock_row = res.scalar_one_or_none()

    if lock_row and lock_row.lock_owner == lock_owner and _to_utc(lock_row.lease_expires_at) >= now:
        return True, lock_row.fencing_token, "acquired"
    elif lock_row:
        return False, lock_row.fencing_token, f"locked_by_{lock_row.lock_owner}"
    return False, 0, "failed_to_acquire"


async def release_conversation_lock(
    db: AsyncSession,
    conversation_id: str,
    lock_owner: str,
    fencing_token: int,
) -> bool:
    """
    Safely releases the conversation lock only if this worker still holds the valid fencing token.
    This prevents stale workers whose lease expired from releasing a lock acquired by a newer worker.
    """
    del_stmt = delete(ConversationLock).where(
        ConversationLock.conversation_id == conversation_id,
        ConversationLock.lock_owner == lock_owner,
        ConversationLock.fencing_token == fencing_token,
    )
    res = await db.execute(del_stmt)
    await db.commit()
    return (res.rowcount or 0) > 0


# ---------------------------------------------------------------------------
# 2. Request Idempotency & Duplicate Prevention
# ---------------------------------------------------------------------------

async def get_or_create_idempotency_record(
    db: AsyncSession,
    request_id: str,
    user_id: str,
    conversation_id: str,
    prompt: str,
    confirmed: bool,
    expires_hours: int = 24,
) -> Tuple[str, Optional[Dict[str, Any]]]:
    """
    Checks or registers an idempotency record for request_id.
    Returns:
      ("NEW", None)               -> Brand new request, proceed with execution.
      ("COMPLETED", cached_resp)  -> Previously completed, replay cached response directly.
      ("PROCESSING", None)        -> Currently being processed by another worker.
      ("FAILED", None)            -> Previously failed, safe to retry.
      ("UNCERTAIN", None)         -> Marked uncertain, requires safety reconciliation.
    Raises:
      ValueError if request_id is reused with a different payload.
    """
    req_hash = compute_request_hash(prompt, confirmed)
    now = _utcnow()
    expires_at = now + timedelta(hours=expires_hours)

    res = await db.execute(
        select(IdempotencyRecord).where(IdempotencyRecord.request_id == request_id)
    )
    existing = res.scalar_one_or_none()

    if existing:
        if existing.request_hash != req_hash:
            raise ValueError(
                f"RequestId {request_id} was already used for a different prompt/payload."
            )
        if existing.status == "COMPLETED" and existing.response_json:
            try:
                cached = json.loads(existing.response_json)
                return "COMPLETED", cached
            except Exception:
                return "COMPLETED", None
        return existing.status, None

    # Register as PROCESSING
    record = IdempotencyRecord(
        request_id=request_id,
        user_id=user_id,
        conversation_id=conversation_id,
        request_hash=req_hash,
        status="PROCESSING",
        created_at=now,
        updated_at=now,
        expires_at=expires_at,
    )
    db.add(record)
    await db.commit()
    return "NEW", None


async def complete_idempotency_record(
    db: AsyncSession,
    request_id: str,
    response_dict: Optional[Dict[str, Any]],
    status: str = "COMPLETED",
) -> None:
    """Updates the idempotency record with final execution status and cached response."""
    now = _utcnow()
    resp_str = json.dumps(response_dict) if response_dict is not None else None

    await db.execute(
        update(IdempotencyRecord)
        .where(IdempotencyRecord.request_id == request_id)
        .values(
            status=status,
            response_json=resp_str,
            updated_at=now,
        )
    )
    await db.commit()


# ---------------------------------------------------------------------------
# 3. Message Processing State Machine
# ---------------------------------------------------------------------------

VALID_STATE_TRANSITIONS = {
    "PENDING": {"PROCESSING", "CANCELLED"},
    "QUEUED": {"PROCESSING", "CANCELLED"},
    "PROCESSING": {"COMPLETED", "FAILED", "UNCERTAIN"},
    "COMPLETED": set(),
    "FAILED": set(),
    "CANCELLED": set(),
    "UNCERTAIN": {"COMPLETED", "FAILED"},
}


class InvalidStateTransitionError(Exception):
    def __init__(self, current_state: str, target_state: str):
        super().__init__(
            f"Invalid message state transition from '{current_state}' to '{target_state}'."
        )


def validate_state_transition(current: str, target: str) -> bool:
    allowed = VALID_STATE_TRANSITIONS.get(current, set())
    return target in allowed


async def update_message_lifecycle(
    db: AsyncSession,
    message_id: str,
    target_status: str,
    error_code: Optional[str] = None,
    error_message: Optional[str] = None,
) -> None:
    """Validates and updates message status with timing and error tracking."""
    res = await db.execute(
        select(Message).where(Message.message_id == message_id)
    )
    msg = res.scalar_one_or_none()
    if not msg:
        return

    if not validate_state_transition(msg.status, target_status):
        raise InvalidStateTransitionError(msg.status, target_status)

    now = _utcnow()
    values: Dict[str, Any] = {"status": target_status}

    if target_status == "PROCESSING":
        values["processing_started_at"] = now
    elif target_status in ("COMPLETED", "FAILED", "UNCERTAIN", "CANCELLED"):
        values["processing_completed_at"] = now

    if error_code:
        values["error_code"] = error_code
    if error_message:
        values["error_message"] = error_message

    await db.execute(
        update(Message)
        .where(Message.message_id == message_id)
        .values(**values)
    )
    await db.commit()
