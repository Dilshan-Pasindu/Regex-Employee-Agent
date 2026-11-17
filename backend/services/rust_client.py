# =============================================================================
# services/rust_client.py — HTTP client for the Rust service (port 5001)
# =============================================================================

from __future__ import annotations

import os
from typing import Any, Dict, Optional

import httpx

RUST_SERVICE_URL = os.getenv("RUST_SERVICE_URL", "http://localhost:5001")
_TIMEOUT = 30.0  # seconds


async def process_prompt(prompt: str, confirmed: bool = False, session_id: Optional[str] = None) -> Dict[str, Any]:
    """
    POST /process to the Rust HTTP service.
    Returns the full JSON response as a dict.
    Raises httpx.HTTPStatusError on 4xx/5xx, httpx.ConnectError when Rust is down.
    """
    async with httpx.AsyncClient(timeout=_TIMEOUT) as client:
        payload: Dict[str, Any] = {"prompt": prompt, "confirmed": confirmed}
        if session_id:
            payload["session_id"] = session_id
        resp = await client.post(
            f"{RUST_SERVICE_URL}/process",
            json=payload,
        )
        resp.raise_for_status()
        return resp.json()


async def get_tools() -> Dict[str, Any]:
    """GET /tools from the Rust HTTP service."""
    async with httpx.AsyncClient(timeout=_TIMEOUT) as client:
        resp = await client.get(f"{RUST_SERVICE_URL}/tools")
        resp.raise_for_status()
        return resp.json()


async def health_check() -> Dict[str, Any]:
    """GET /health from the Rust HTTP service."""
    async with httpx.AsyncClient(timeout=10.0) as client:
        resp = await client.get(f"{RUST_SERVICE_URL}/health")
        resp.raise_for_status()
        return resp.json()
