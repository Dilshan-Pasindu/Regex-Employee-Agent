# =============================================================================
# routers/health.py — GET /api/health
# =============================================================================

from fastapi import APIRouter
from models import HealthResponse
from services import rust_client

router = APIRouter(prefix="/api/health", tags=["health"])


@router.get("", response_model=HealthResponse)
async def health():
    try:
        data = await rust_client.health_check()
        return HealthResponse(
            status="ok",
            backend="ok",
            rust_service=data.get("rust", "ok"),
            dotnet_mcp=data.get("dotnet", "unknown"),
            tools_count=data.get("tools_count", 0),
        )
    except Exception as exc:
        return HealthResponse(
            status="degraded",
            backend="ok",
            rust_service="unreachable",
            dotnet_mcp="unknown",
            tools_count=0,
        )
