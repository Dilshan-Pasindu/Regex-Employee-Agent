# =============================================================================
# models.py — Pydantic request/response models for the FastAPI backend
# =============================================================================

from __future__ import annotations

from typing import Any, Dict, List, Optional
from pydantic import BaseModel, Field


# ---------------------------------------------------------------------------
# Inbound requests
# ---------------------------------------------------------------------------

class ChatRequest(BaseModel):
    """Body for POST /api/chat"""
    prompt: str = Field(..., min_length=1, description="Natural-language employee management command")
    confirmed: bool = Field(False, description="Set true to confirm a pending write operation")
    session_id: Optional[str] = Field(None, description="Session ID for conversation continuity (auto-generated if absent)")


# ---------------------------------------------------------------------------
# Step trace (mirrors Rust /process response)
# ---------------------------------------------------------------------------


class StepTrace(BaseModel):
    step: int
    label: str
    value: Any


# ---------------------------------------------------------------------------
# Chat response
# ---------------------------------------------------------------------------

class ChatResponse(BaseModel):
    session_id: str
    prompt: str
    intent: str
    tool: Optional[str] = None
    params: Dict[str, Any] = Field(default_factory=dict)
    steps: List[StepTrace] = Field(default_factory=list)
    result: Optional[Any] = None
    success: bool
    requires_confirmation: Optional[bool] = None
    message: Optional[str] = None


# ---------------------------------------------------------------------------
# Conversation history entry
# ---------------------------------------------------------------------------

class ConversationEntry(BaseModel):
    role: str            # "user" | "agent"
    content: str
    intent: Optional[str] = None
    success: Optional[bool] = None


class ConversationResponse(BaseModel):
    session_id: str
    history: List[ConversationEntry]
    total_messages: int


# ---------------------------------------------------------------------------
# Tool catalogue (proxied from Rust /tools)
# ---------------------------------------------------------------------------

class ToolCatalogueResponse(BaseModel):
    tools: List[Dict[str, Any]]
    count: int


# ---------------------------------------------------------------------------
# Health check
# ---------------------------------------------------------------------------

class HealthResponse(BaseModel):
    status: str
    backend: str
    rust_service: str
    dotnet_mcp: str
    tools_count: int
