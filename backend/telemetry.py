# =============================================================================
# telemetry.py — Structured Telemetry, Correlation IDs & Observability (Phase 5)
# =============================================================================
from __future__ import annotations

import logging
import time
import uuid
from contextvars import ContextVar
from typing import Optional

from starlette.middleware.base import BaseHTTPMiddleware
from starlette.requests import Request
from starlette.responses import Response

# Context variable for holding correlation_id in async context
correlation_id_ctx: ContextVar[str] = ContextVar("correlation_id", default="")


class CorrelationIdFilter(logging.Filter):
    """Injects correlation_id into all logging records."""
    def filter(self, record):
        record.correlation_id = correlation_id_ctx.get() or "no_corr_id"
        return True


def setup_telemetry_logging():
    """Configures structured format with correlation_id."""
    formatter = logging.Formatter(
        "[%(asctime)s] [%(levelname)s] [corr:%(correlation_id)s] %(name)s: %(message)s",
        datefmt="%Y-%m-%d %H:%M:%S",
    )
    handler = logging.StreamHandler()
    handler.setFormatter(formatter)
    handler.addFilter(CorrelationIdFilter())

    root_logger = logging.getLogger()
    root_logger.setLevel(logging.INFO)
    # Avoid duplicate handlers if re-called
    if not any(isinstance(h, logging.StreamHandler) and any(isinstance(f, CorrelationIdFilter) for f in h.filters) for h in root_logger.handlers):
        root_logger.addHandler(handler)


class CorrelationIdMiddleware(BaseHTTPMiddleware):
    """
    Middleware that ensures every request has a CorrelationId.
    Propagates X-Correlation-ID through response headers and async context.
    """
    async def dispatch(self, request: Request, call_next) -> Response:
        corr_id = request.headers.get("X-Correlation-ID") or f"corr_{uuid.uuid4().hex[:12]}"
        token = correlation_id_ctx.set(corr_id)
        request.state.correlation_id = corr_id

        start_time = time.time()
        try:
            response = await call_next(request)
            duration_ms = round((time.time() - start_time) * 1000, 2)
            response.headers["X-Correlation-ID"] = corr_id
            response.headers["X-Response-Time-Ms"] = str(duration_ms)
            return response
        finally:
            correlation_id_ctx.reset(token)
