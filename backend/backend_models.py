# =============================================================================
# backend_models.py — SQLAlchemy ORM models for conversation management
#
# IMPORTANT: These tables are SEPARATE from the .NET EF Core employee tables.
# FastAPI owns ONLY: Conversations, Messages
# .NET EF Core owns: Employees, Departments, Designations
# =============================================================================
from __future__ import annotations

import uuid
from datetime import datetime, timezone

from sqlalchemy import (
    BigInteger, Boolean, DateTime, ForeignKey, Integer, String, Text, func
)
from sqlalchemy.orm import Mapped, mapped_column, relationship

from db import Base


def _utcnow() -> datetime:
    return datetime.now(timezone.utc)


def _new_uuid() -> str:
    return str(uuid.uuid4())


class Conversation(Base):
    __tablename__ = "Conversations"

    conversation_id: Mapped[str] = mapped_column(
        String(36), primary_key=True, default=_new_uuid, index=True
    )
    # Phase 1: single-user system; user_id is a placeholder for future auth
    user_id: Mapped[str] = mapped_column(String(100), nullable=False, index=True, default="default_user")
    title: Mapped[str] = mapped_column(String(255), nullable=False, default="New Conversation")
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=_utcnow, nullable=False)
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=_utcnow, onupdate=_utcnow, nullable=False)
    last_message_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    is_pinned: Mapped[bool] = mapped_column(Boolean, default=False, nullable=False)
    pinned_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    is_deleted: Mapped[bool] = mapped_column(Boolean, default=False, nullable=False)

    messages: Mapped[list["Message"]] = relationship(
        "Message", back_populates="conversation", order_by="Message.sequence_number"
    )
    context: Mapped["ConversationContext | None"] = relationship(
        "ConversationContext", back_populates="conversation", uselist=False, cascade="all, delete-orphan"
    )
    snapshots: Mapped[list["ContextSnapshot"]] = relationship(
        "ContextSnapshot", back_populates="conversation", cascade="all, delete-orphan", order_by="ContextSnapshot.message_sequence_number"
    )


class ConversationContext(Base):
    __tablename__ = "ConversationContexts"

    conversation_id: Mapped[str] = mapped_column(
        String(36),
        ForeignKey("Conversations.conversation_id", ondelete="CASCADE"),
        primary_key=True,
    )
    version: Mapped[int] = mapped_column(Integer, default=1, nullable=False)
    schema_version: Mapped[int] = mapped_column(Integer, default=1, nullable=False)
    context_json: Mapped[str] = mapped_column(Text, nullable=False, default="{}")
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=_utcnow, nullable=False)
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=_utcnow, onupdate=_utcnow, nullable=False)

    conversation: Mapped["Conversation"] = relationship("Conversation", back_populates="context")


class Message(Base):
    __tablename__ = "Messages"

    message_id: Mapped[str] = mapped_column(
        String(36), primary_key=True, default=_new_uuid, index=True
    )
    conversation_id: Mapped[str] = mapped_column(
        String(36), ForeignKey("Conversations.conversation_id", ondelete="CASCADE"),
        nullable=False, index=True
    )
    sequence_number: Mapped[int] = mapped_column(Integer, nullable=False)
    role: Mapped[str] = mapped_column(String(20), nullable=False)  # user | assistant
    content: Mapped[str] = mapped_column(Text, nullable=False)
    intent: Mapped[str | None] = mapped_column(String(100), nullable=True)
    success: Mapped[bool | None] = mapped_column(Boolean, nullable=True)
    metadata_json: Mapped[str | None] = mapped_column(Text, nullable=True)
    request_id: Mapped[str | None] = mapped_column(String(64), nullable=True, index=True)
    status: Mapped[str] = mapped_column(String(20), default="COMPLETED", nullable=False)
    error_code: Mapped[str | None] = mapped_column(String(50), nullable=True)
    error_message: Mapped[str | None] = mapped_column(Text, nullable=True)
    processing_started_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    processing_completed_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=_utcnow, nullable=False)

    conversation: Mapped["Conversation"] = relationship("Conversation", back_populates="messages")


class ConversationLock(Base):
    __tablename__ = "ConversationLocks"

    conversation_id: Mapped[str] = mapped_column(
        String(36),
        ForeignKey("Conversations.conversation_id", ondelete="CASCADE"),
        primary_key=True,
    )
    lock_owner: Mapped[str] = mapped_column(String(100), nullable=False)
    fencing_token: Mapped[int] = mapped_column(BigInteger, default=1, nullable=False)
    lease_expires_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False)
    acquired_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=_utcnow, nullable=False)
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=_utcnow, onupdate=_utcnow, nullable=False)

    conversation: Mapped["Conversation"] = relationship("Conversation")


class IdempotencyRecord(Base):
    __tablename__ = "IdempotencyRecords"

    request_id: Mapped[str] = mapped_column(String(64), primary_key=True)
    user_id: Mapped[str] = mapped_column(String(100), nullable=False, index=True)
    conversation_id: Mapped[str] = mapped_column(
        String(36),
        ForeignKey("Conversations.conversation_id", ondelete="CASCADE"),
        nullable=False,
        index=True,
    )
    request_hash: Mapped[str] = mapped_column(String(64), nullable=False)
    status: Mapped[str] = mapped_column(String(20), nullable=False)  # PROCESSING, COMPLETED, FAILED, UNCERTAIN
    response_json: Mapped[str | None] = mapped_column(Text, nullable=True)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=_utcnow, nullable=False)
    updated_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=_utcnow, onupdate=_utcnow, nullable=False)
    expires_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), nullable=False, index=True)

    conversation: Mapped["Conversation"] = relationship("Conversation")


class OutboxEvent(Base):
    __tablename__ = "OutboxEvents"

    event_id: Mapped[str] = mapped_column(
        String(36), primary_key=True, default=_new_uuid
    )
    event_type: Mapped[str] = mapped_column(String(50), nullable=False)
    aggregate_id: Mapped[str] = mapped_column(String(64), nullable=False, index=True)
    payload_json: Mapped[str] = mapped_column(Text, nullable=False)
    status: Mapped[str] = mapped_column(String(20), default="PENDING", nullable=False, index=True)
    attempt_count: Mapped[int] = mapped_column(Integer, default=0, nullable=False)
    available_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=_utcnow, nullable=False, index=True)
    locked_by: Mapped[str | None] = mapped_column(String(100), nullable=True)
    lease_expires_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True, index=True)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=_utcnow, nullable=False)
    processed_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    last_error: Mapped[str | None] = mapped_column(Text, nullable=True)


class OperationRecord(Base):
    __tablename__ = "OperationRecords"

    operation_id: Mapped[str] = mapped_column(
        String(36), primary_key=True, default=_new_uuid
    )
    request_id: Mapped[str] = mapped_column(String(64), nullable=False, index=True)
    conversation_id: Mapped[str] = mapped_column(
        String(36),
        ForeignKey("Conversations.conversation_id", ondelete="CASCADE"),
        nullable=False,
        index=True,
    )
    operation_type: Mapped[str] = mapped_column(String(50), nullable=False)  # READ, WRITE, BATCH_WRITE, UNDO
    tool_name: Mapped[str] = mapped_column(String(100), nullable=False)
    payload_hash: Mapped[str] = mapped_column(String(64), nullable=False)
    status: Mapped[str] = mapped_column(String(20), default="STARTED", nullable=False, index=True)
    started_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=_utcnow, nullable=False)
    completed_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True), nullable=True)
    result_json: Mapped[str | None] = mapped_column(Text, nullable=True)
    error_code: Mapped[str | None] = mapped_column(String(50), nullable=True)
    error_message: Mapped[str | None] = mapped_column(Text, nullable=True)
    attempt_count: Mapped[int] = mapped_column(Integer, default=1, nullable=False)

    conversation: Mapped["Conversation"] = relationship("Conversation")


class ContextSnapshot(Base):
    __tablename__ = "ContextSnapshots"

    snapshot_id: Mapped[str] = mapped_column(
        String(36), primary_key=True, default=_new_uuid
    )
    conversation_id: Mapped[str] = mapped_column(
        String(36),
        ForeignKey("Conversations.conversation_id", ondelete="CASCADE"),
        nullable=False,
        index=True,
    )
    context_version: Mapped[int] = mapped_column(Integer, nullable=False)
    message_sequence_number: Mapped[int] = mapped_column(Integer, nullable=False)
    schema_version: Mapped[int] = mapped_column(Integer, default=1, nullable=False)
    snapshot_json: Mapped[str] = mapped_column(Text, nullable=False)
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=_utcnow, nullable=False)

    conversation: Mapped["Conversation"] = relationship("Conversation", back_populates="snapshots")
