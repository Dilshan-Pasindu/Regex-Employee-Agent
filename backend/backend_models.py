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
    Boolean, DateTime, ForeignKey, Integer, String, Text, func
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
    created_at: Mapped[datetime] = mapped_column(DateTime(timezone=True), default=_utcnow, nullable=False)

    conversation: Mapped["Conversation"] = relationship("Conversation", back_populates="messages")
