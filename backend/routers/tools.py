# =============================================================================
# routers/tools.py — GET /api/tools
# =============================================================================

from fastapi import APIRouter, HTTPException
import httpx

from models import ToolCatalogueResponse
from services import rust_client

router = APIRouter(prefix="/api/tools", tags=["tools"])


@router.get("", response_model=ToolCatalogueResponse)
async def get_tools():
    try:
        data = await rust_client.get_tools()
    except httpx.ConnectError:
        raise HTTPException(503, "Rust service unreachable")
    except httpx.HTTPStatusError as exc:
        raise HTTPException(exc.response.status_code, exc.response.text)

    return ToolCatalogueResponse(
        tools=data.get("tools", []),
        count=data.get("count", 0),
    )
