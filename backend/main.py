# =============================================================================
# main.py — FastAPI Application Entry Point (Phase 3 — Backend)
#
# Port: 5000 (default)  — configure via .env PORT variable
#
# API surface:
#   POST   /api/chat                     → core agentic prompt endpoint
#   GET    /api/tools                    → MCP tool catalogue (from Rust)
#   GET    /api/health                   → system health check
#   GET    /api/conversations            → list active sessions
#   GET    /api/conversations/{id}       → fetch session history
#   DELETE /api/conversations/{id}       → clear session
#   GET    /docs                         → Swagger UI (auto-generated)
#   GET    /redoc                        → ReDoc UI (auto-generated)
# =============================================================================

import os
from contextlib import asynccontextmanager

from dotenv import load_dotenv
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

load_dotenv()

from routers import chat, tools, health, conversations


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Startup / shutdown lifecycle."""
    print("╔══════════════════════════════════════════════════════╗")
    print("║   Employee Management — FastAPI Backend v1.0.0      ║")
    print("║   Phase 3: Python/FastAPI → Rust → .NET MCP → MySQL ║")
    print("╚══════════════════════════════════════════════════════╝")
    rust_url = os.getenv("RUST_SERVICE_URL", "http://localhost:5001")
    print(f"  Rust service URL : {rust_url}")
    print(f"  Swagger UI       : http://localhost:{os.getenv('PORT', '5000')}/docs")
    print()
    yield
    print("  FastAPI backend shutting down.")


app = FastAPI(
    title="Employee Management Agentic API",
    description=(
        "Phase 3 Backend — FastAPI bridges the React frontend and the Rust/MCP gateway.\n\n"
        "**Request flow:**\n"
        "React UI → FastAPI (5000) → Rust HTTP (5001) → .NET MCP Server (STDIO) → MySQL\n\n"
        "**Key design:**\n"
        "- Zero LLM/AI: intent detection is 100% deterministic Rust Regex.\n"
        "- Write operations (add/update) require a two-step confirmation flow.\n"
        "- Session-based conversation history (in-memory per backend process).\n"
    ),
    version="1.0.0",
    lifespan=lifespan,
    docs_url="/docs",
    redoc_url="/redoc",
)

# ---------------------------------------------------------------------------
# CORS — allow React dev server on :5173 (and any other local origin)
# ---------------------------------------------------------------------------
app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "http://localhost:5173",
        "http://127.0.0.1:5173",
        "http://localhost:3000",
    ],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# ---------------------------------------------------------------------------
# Routers
# ---------------------------------------------------------------------------
app.include_router(chat.router)
app.include_router(tools.router)
app.include_router(health.router)
app.include_router(conversations.router)


@app.get("/", include_in_schema=False)
async def root():
    return {
        "service": "Employee Management Agentic API",
        "version": "1.0.0",
        "docs": "/docs",
        "health": "/api/health",
    }
