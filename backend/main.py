# =============================================================================
# main.py — FastAPI Application Entry Point (Phase 1 — Persistent Chat History)
# =============================================================================

import os
from contextlib import asynccontextmanager

from dotenv import load_dotenv
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

load_dotenv()

from db import init_db
from routers import chat, tools, health
from routers import conv_mgmt


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Startup / shutdown lifecycle."""
    print("╔══════════════════════════════════════════════════════╗")
    print("║   Employee Management — FastAPI Backend v2.0.0      ║")
    print("║   Phase 1: Persistent Chat History (MySQL)          ║")
    print("╚══════════════════════════════════════════════════════╝")
    rust_url = os.getenv("RUST_SERVICE_URL", "http://localhost:5001")
    print(f"  Rust service URL : {rust_url}")
    print(f"  Swagger UI       : http://localhost:{os.getenv('PORT', '5000')}/docs")
    print("  Initialising conversation tables in MySQL…")
    await init_db()
    print("  MySQL conversation tables ready.")
    print("  Starting background Outbox worker…")
    from services.outbox_worker import outbox_worker
    outbox_worker.start()
    print("  Outbox worker active.")
    print()
    yield
    print("  Stopping Outbox worker…")
    await outbox_worker.stop()
    print("  FastAPI backend shutting down.")


app = FastAPI(
    title="Employee Management Agentic API",
    description=(
        "Phase 1: Persistent Chat History — MySQL-backed conversation management.\n\n"
        "**Request flow:**\n"
        "React UI → FastAPI (5000) → Rust HTTP (5001) → .NET MCP Server (STDIO) → MySQL\n\n"
        "**Key design:**\n"
        "- Zero LLM/AI: intent detection is 100% deterministic Rust Regex.\n"
        "- Write operations (add/update) require a two-step confirmation flow.\n"
        "- Persistent conversation history in MySQL (survives restarts).\n"
    ),
    version="2.0.0",
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
app.include_router(conv_mgmt.router)


@app.get("/", include_in_schema=False)
async def root():
    return {
        "service": "Employee Management Agentic API",
        "version": "2.0.0",
        "docs": "/docs",
        "health": "/api/health",
    }
