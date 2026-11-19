# =============================================================================
# services/outbox_worker.py — Background Outbox Processing Worker (Phase 4)
#
# Reliably claims and processes OutboxEvents from MySQL:
# - Uses leases to prevent duplicate processing by concurrent workers.
# - Recovers abandoned events after server restarts.
# - Exponential backoff with dead-letter queueing on permanent failures.
# =============================================================================
from __future__ import annotations

import asyncio
import json
import logging
import uuid
from datetime import datetime, timezone, timedelta
from typing import Optional

from sqlalchemy import select, update, or_
from sqlalchemy.ext.asyncio import AsyncSession

from db import AsyncSessionLocal
from backend_models import OutboxEvent

logger = logging.getLogger("outbox_worker")

WORKER_ID = f"worker_{uuid.uuid4().hex[:10]}"
POLL_INTERVAL_SECONDS = 1.0
LEASE_DURATION_SECONDS = 30
MAX_ATTEMPTS = 5


def _utcnow() -> datetime:
    return datetime.now(timezone.utc)


class OutboxWorker:
    def __init__(self):
        self._running = False
        self._task: Optional[asyncio.Task] = None

    def start(self):
        """Starts the background worker task."""
        if not self._running:
            self._running = True
            self._task = asyncio.create_task(self._run_loop())
            logger.info("Outbox worker started with ID: %s", WORKER_ID)

    async def stop(self):
        """Gracefully stops the worker."""
        if self._running:
            self._running = False
            if self._task:
                self._task.cancel()
                try:
                    await self._task
                except asyncio.CancelledError:
                    pass
            logger.info("Outbox worker stopped cleanly.")

    async def _run_loop(self):
        """Main polling loop."""
        while self._running:
            try:
                processed_count = await self.process_batch(batch_size=10)
                # If we processed events, immediately check for more; otherwise sleep
                if processed_count == 0:
                    await asyncio.sleep(POLL_INTERVAL_SECONDS)
            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error("Error in outbox polling loop: %s", e, exc_info=True)
                await asyncio.sleep(POLL_INTERVAL_SECONDS)

    async def process_batch(self, batch_size: int = 10) -> int:
        """
        Polls, claims, and executes a batch of available outbox events.
        Returns the number of events processed.
        """
        now = _utcnow()

        async with AsyncSessionLocal() as db:
            # Query candidate events:
            # 1. PENDING events whose available_at <= now
            # 2. Abandoned PROCESSING events whose lease has expired
            query = (
                select(OutboxEvent.event_id)
                .where(
                    or_(
                        OutboxEvent.status == "PENDING",
                        OutboxEvent.lease_expires_at < now,
                    )
                )
                .where(OutboxEvent.available_at <= now)
                .order_by(OutboxEvent.available_at.asc())
                .limit(batch_size)
            )
            result = await db.execute(query)
            event_ids = list(result.scalars().all())

            if not event_ids:
                return 0

            processed_count = 0
            for event_id in event_ids:
                # Atomically claim event with a lease
                claimed = await self._claim_event(db, event_id, now)
                if not claimed:
                    continue

                success = await self._dispatch_event(db, claimed)
                if success:
                    await self._mark_processed(db, event_id)
                else:
                    await self._mark_failed(db, event_id, claimed.attempt_count)

                processed_count += 1

            return processed_count

    async def _claim_event(
        self, db: AsyncSession, event_id: str, now: datetime
    ) -> Optional[OutboxEvent]:
        """Atomically takes a lease on the event."""
        lease_expires = now + timedelta(seconds=LEASE_DURATION_SECONDS)

        # Atomic conditional update
        res = await db.execute(
            update(OutboxEvent)
            .where(OutboxEvent.event_id == event_id)
            .where(
                or_(
                    OutboxEvent.status == "PENDING",
                    OutboxEvent.lease_expires_at < now,
                )
            )
            .values(
                status="PROCESSING",
                locked_by=WORKER_ID,
                lease_expires_at=lease_expires,
                attempt_count=OutboxEvent.attempt_count + 1,
            )
        )
        await db.commit()

        if (res.rowcount or 0) > 0:
            row_res = await db.execute(
                select(OutboxEvent).where(OutboxEvent.event_id == event_id)
            )
            return row_res.scalar_one_or_none()
        return None

    async def _dispatch_event(self, db: AsyncSession, event: OutboxEvent) -> bool:
        """
        Dispatches event according to event_type.
        Returns True if processing succeeded, False on failure.
        """
        try:
            payload = json.loads(event.payload_json) if event.payload_json else {}

            if event.event_type == "MESSAGE_RECEIVED":
                logger.info(
                    "[Event: MESSAGE_RECEIVED] Aggregate: %s, Prompt: %s",
                    event.aggregate_id,
                    payload.get("prompt"),
                )
            elif event.event_type == "MESSAGE_COMPLETED":
                logger.info(
                    "[Event: MESSAGE_COMPLETED] Aggregate: %s, Intent: %s, Success: %s",
                    event.aggregate_id,
                    payload.get("intent"),
                    payload.get("success"),
                )
            elif event.event_type == "MESSAGE_FAILED":
                logger.warning(
                    "[Event: MESSAGE_FAILED] Aggregate: %s, Error: %s",
                    event.aggregate_id,
                    payload.get("error"),
                )
            elif event.event_type == "CONTEXT_UPDATED":
                logger.info(
                    "[Event: CONTEXT_UPDATED] Aggregate: %s, Version: %s",
                    event.aggregate_id,
                    payload.get("version"),
                )
            else:
                logger.info(
                    "[Event: %s] Aggregate: %s",
                    event.event_type,
                    event.aggregate_id,
                )

            return True
        except Exception as exc:
            logger.error("Failed to process event %s: %s", event.event_id, exc)
            return False

    async def _mark_processed(self, db: AsyncSession, event_id: str) -> None:
        """Marks event PROCESSED."""
        now = _utcnow()
        await db.execute(
            update(OutboxEvent)
            .where(OutboxEvent.event_id == event_id)
            .values(
                status="PROCESSED",
                processed_at=now,
                locked_by=None,
                lease_expires_at=None,
            )
        )
        await db.commit()

    async def _mark_failed(
        self, db: AsyncSession, event_id: str, attempt_count: int
    ) -> None:
        """Applies exponential backoff or moves to DEAD_LETTER."""
        now = _utcnow()
        if attempt_count >= MAX_ATTEMPTS:
            new_status = "DEAD_LETTER"
            delay_sec = 0
        else:
            new_status = "PENDING"
            delay_sec = 2 ** attempt_count  # 2, 4, 8, 16...

        available_at = now + timedelta(seconds=delay_sec)
        await db.execute(
            update(OutboxEvent)
            .where(OutboxEvent.event_id == event_id)
            .values(
                status=new_status,
                available_at=available_at,
                locked_by=None,
                lease_expires_at=None,
                last_error=f"Attempt {attempt_count} failed at {now.isoformat()}",
            )
        )
        await db.commit()


# Singleton worker instance
outbox_worker = OutboxWorker()
