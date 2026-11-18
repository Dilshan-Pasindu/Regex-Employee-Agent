# =============================================================================
# db.py — Async SQLAlchemy engine and session factory (Phase 1)
# Database: InternAgenticDB (shared with .NET EF Core for employee tables)
# FastAPI owns: Conversations, Messages (conversation management tables only)
# =============================================================================
from __future__ import annotations

import os
from sqlalchemy.ext.asyncio import AsyncSession, async_sessionmaker, create_async_engine
from sqlalchemy.orm import DeclarativeBase

DATABASE_URL = os.getenv(
    "DATABASE_URL",
    "mysql+aiomysql://root:Jdilshan%40123@127.0.0.1:3306/InternAgenticDB?charset=utf8mb4",
)

engine = create_async_engine(
    DATABASE_URL,
    pool_size=10,
    max_overflow=5,
    pool_recycle=300,
    pool_pre_ping=True,
    echo=False,
)

AsyncSessionLocal = async_sessionmaker(
    engine,
    class_=AsyncSession,
    expire_on_commit=False,
)


class Base(DeclarativeBase):
    pass


async def get_db() -> AsyncSession:
    """FastAPI dependency — yields an async DB session."""
    async with AsyncSessionLocal() as session:
        yield session


async def init_db() -> None:
    """Create conversation-management tables (idempotent)."""
    from backend_models import Conversation, Message  # noqa: F401
    async with engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)
