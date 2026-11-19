# =============================================================================
# services/outbox_service.py — Transactional Outbox & Operation Tracking (Phase 4)
#
# Emits durable events atomically within MySQL transactions.
# Tracks employee write operations & manages reconciliation for uncertain states.
# =============================================================================
from __future__ import annotations

import hashlib
import json
import uuid
from datetime import datetime, timezone, timedelta
from typing import Any, Dict, Optional, Tuple

from sqlalchemy import select, update
from sqlalchemy.ext.asyncio import AsyncSession

from backend_models import OutboxEvent, OperationRecord


def _utcnow() -> datetime:
    return datetime.now(timezone.utc)


def _compute_payload_hash(data: Any) -> str:
    normalized = json.dumps(data, sort_keys=True, default=str).encode("utf-8")
    return hashlib.sha256(normalized).hexdigest()


# ---------------------------------------------------------------------------
# 1. Transactional Outbox Event Publishing
# ---------------------------------------------------------------------------

async def record_outbox_event(
    db: AsyncSession,
    event_type: str,
    aggregate_id: str,
    payload: Dict[str, Any],
    delay_seconds: int = 0,
) -> OutboxEvent:
    """
    Atomically enqueues an event into OutboxEvents.
    Must be committed as part of the calling unit-of-work.
    """
    now = _utcnow()
    available_at = now + timedelta(seconds=delay_seconds)
    payload_str = json.dumps(payload, default=str)

    event = OutboxEvent(
        event_id=str(uuid.uuid4()),
        event_type=event_type,
        aggregate_id=aggregate_id,
        payload_json=payload_str,
        status="PENDING",
        attempt_count=0,
        available_at=available_at,
        created_at=now,
    )
    db.add(event)
    return event


# ---------------------------------------------------------------------------
# 2. Employee Operation Tracking & Safety Lifecycle
# ---------------------------------------------------------------------------

async def record_operation_start(
    db: AsyncSession,
    request_id: str,
    conversation_id: str,
    operation_type: str,
    tool_name: str,
    params: Dict[str, Any],
) -> OperationRecord:
    """
    Records the beginning of an employee database operation.
    Particularly critical for write operations (add_employee, update_employee, batch).
    """
    now = _utcnow()
    op_hash = _compute_payload_hash(params)

    record = OperationRecord(
        operation_id=str(uuid.uuid4()),
        request_id=request_id,
        conversation_id=conversation_id,
        operation_type=operation_type,
        tool_name=tool_name,
        payload_hash=op_hash,
        status="STARTED",
        started_at=now,
        attempt_count=1,
    )
    db.add(record)
    await db.commit()
    await db.refresh(record)
    return record


async def record_operation_complete(
    db: AsyncSession,
    operation_id: str,
    status: str,  # COMPLETED, FAILED, UNCERTAIN, RECONCILED
    result: Optional[Any] = None,
    error_code: Optional[str] = None,
    error_message: Optional[str] = None,
) -> None:
    """Finalizes an operation record with completion status, timing, and error details."""
    now = _utcnow()
    res_str = json.dumps(result, default=str) if result is not None else None

    await db.execute(
        update(OperationRecord)
        .where(OperationRecord.operation_id == operation_id)
        .values(
            status=status,
            completed_at=now,
            result_json=res_str,
            error_code=error_code,
            error_message=error_message,
        )
    )
    await db.commit()


# ---------------------------------------------------------------------------
# 3. Uncertain Operation Reconciliation
# ---------------------------------------------------------------------------

async def get_uncertain_operations(db: AsyncSession) -> list[OperationRecord]:
    """Retrieves all operations marked UNCERTAIN requiring safety verification."""
    res = await db.execute(
        select(OperationRecord).where(OperationRecord.status == "UNCERTAIN")
    )
    return list(res.scalars().all())
