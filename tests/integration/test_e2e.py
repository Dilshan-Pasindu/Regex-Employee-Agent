#!/usr/bin/env python3
"""
Phase 8: End-to-End Integration Test Suite
==========================================
Tests all major flows against live services.

Usage:
  cd /path/to/Spillabs-intern
  python3 tests/integration/test_e2e.py

Requirements:
  - FastAPI backend running at http://localhost:5000
  - Rust service running at http://localhost:5001
  - .NET MCP server running (started by Rust automatically)
  - MySQL InternAgenticDB accessible

Install test deps:
  pip install httpx pytest pytest-asyncio
"""

import asyncio
import sys
import time
import httpx

BASE = "http://localhost:5000"
RUST = "http://localhost:5001"


class TestResult:
    def __init__(self):
        self.passed = []
        self.failed = []

    def ok(self, name):
        self.passed.append(name)
        print(f"  ✓ {name}")

    def fail(self, name, reason):
        self.failed.append(name)
        print(f"  ✗ {name}: {reason}")

    def summary(self):
        total = len(self.passed) + len(self.failed)
        print(f"\n{'='*60}")
        print(f"Results: {len(self.passed)}/{total} passed")
        if self.failed:
            print("Failed tests:")
            for t in self.failed:
                print(f"  - {t}")
        return len(self.failed) == 0


results = TestResult()


# ── Helpers ──────────────────────────────────────────────────────────────────

async def chat(client: httpx.AsyncClient, prompt: str, conversation_id=None, confirmed=False):
    r = await client.post(f"{BASE}/api/chat", json={
        "prompt": prompt,
        "session_id": conversation_id,
        "conversation_id": conversation_id,
        "confirmed": confirmed,
    }, timeout=30)
    r.raise_for_status()
    return r.json()


# ── Test: Health Checks ───────────────────────────────────────────────────────

async def test_health(client):
    print("\n[1] Health Checks")
    try:
        r = await client.get(f"{BASE}/api/health", timeout=10)
        assert r.status_code == 200
        results.ok("FastAPI /api/health returns 200")
    except Exception as e:
        results.fail("FastAPI /api/health returns 200", str(e))

    try:
        r = await client.get(f"{RUST}/health", timeout=10)
        assert r.status_code == 200
        results.ok("Rust /health returns 200")
    except Exception as e:
        results.fail("Rust /health returns 200", str(e))


# ── Test: MCP Tool Catalogue ─────────────────────────────────────────────────

async def test_tools(client):
    print("\n[2] MCP Tool Catalogue")
    try:
        r = await client.get(f"{BASE}/api/tools", timeout=10)
        assert r.status_code == 200
        data = r.json()
        tools = data.get("tools", [])
        assert len(tools) >= 6, f"Expected ≥6 tools, got {len(tools)}"
        tool_names = [t["name"] for t in tools]
        for expected in ["get_all_employees", "get_employee_by_id", "search_employees"]:
            assert expected in tool_names, f"Missing tool: {expected}"
        results.ok(f"Tool catalogue returns {len(tools)} tools")
    except Exception as e:
        results.fail("Tool catalogue", str(e))


# ── Test: Conversation CRUD ───────────────────────────────────────────────────

async def test_conversations(client):
    print("\n[3] Conversation CRUD")
    conv_id = None

    # Create via chat (auto-creates)
    try:
        resp = await chat(client, "Hello, show all employees")
        conv_id = resp.get("session_id")
        assert conv_id, "No session_id in response"
        results.ok("Chat auto-creates conversation")
    except Exception as e:
        results.fail("Chat auto-creates conversation", str(e))
        return None

    # List conversations
    try:
        r = await client.get(f"{BASE}/api/conversations", timeout=10)
        assert r.status_code == 200
        data = r.json()
        ids = [c["conversation_id"] for c in data.get("conversations", [])]
        assert conv_id in ids, "New conversation not in list"
        results.ok("List conversations includes new conversation")
    except Exception as e:
        results.fail("List conversations", str(e))

    # Rename
    try:
        r = await client.patch(f"{BASE}/api/conversations/{conv_id}", json={"title": "E2E Test Chat"}, timeout=10)
        assert r.status_code == 200
        results.ok("Rename conversation")
    except Exception as e:
        results.fail("Rename conversation", str(e))

    # Get messages
    try:
        r = await client.get(f"{BASE}/api/conversations/{conv_id}/messages", timeout=10)
        assert r.status_code == 200
        msgs = r.json().get("messages", [])
        assert len(msgs) >= 2, f"Expected ≥2 messages, got {len(msgs)}"
        results.ok(f"Get messages returns {len(msgs)} messages")
    except Exception as e:
        results.fail("Get messages", str(e))

    return conv_id


# ── Test: Read Queries ────────────────────────────────────────────────────────

async def test_read_queries(client):
    print("\n[4] Read Query Flows")

    # get_all_employees
    try:
        resp = await chat(client, "Show all employees")
        assert resp.get("success"), f"Failed: {resp.get('message')}"
        result = resp.get("result", [])
        assert len(result) > 0, "No employees returned"
        results.ok(f"get_all_employees returns {len(result)} employees")
    except Exception as e:
        results.fail("get_all_employees", str(e))

    # get_employee_by_id
    conv_id = None
    try:
        resp = await chat(client, "Find employee 1001")
        assert resp.get("success"), f"Failed: {resp.get('message')}"
        conv_id = resp.get("session_id")
        results.ok("get_employee_by_id (1001)")
    except Exception as e:
        results.fail("get_employee_by_id (1001)", str(e))

    # search by department
    try:
        resp = await chat(client, "Show all employees in IT department")
        assert resp.get("success"), f"Failed: {resp.get('message')}"
        results.ok("search_employees by department (IT)")
    except Exception as e:
        results.fail("search_employees by department (IT)", str(e))

    # get_department_list
    try:
        resp = await chat(client, "List all departments")
        assert resp.get("success"), f"Failed: {resp.get('message')}"
        results.ok("get_department_list")
    except Exception as e:
        results.fail("get_department_list", str(e))

    # get_employee_count
    try:
        resp = await chat(client, "How many employees are there?")
        assert resp.get("success"), f"Failed: {resp.get('message')}"
        results.ok("get_employee_count")
    except Exception as e:
        results.fail("get_employee_count", str(e))

    return conv_id


# ── Test: Context Resolution ──────────────────────────────────────────────────

async def test_context_resolution(client):
    print("\n[5] Context Resolution (Follow-up Queries)")
    try:
        # First message: establish context
        resp1 = await chat(client, "Find employee 1005")
        conv_id = resp1.get("session_id")
        assert resp1.get("success"), f"Initial query failed: {resp1.get('message')}"

        # Follow-up using pronoun
        resp2 = await chat(client, "Show his salary", conversation_id=conv_id)
        assert resp2.get("success"), f"Context follow-up failed: {resp2.get('message')}"

        steps = resp2.get("steps", [])
        has_context = any(
            "context" in (s.get("label", "") if isinstance(s, dict) else str(s)).lower()
            or "resolv" in (s.get("label", "") if isinstance(s, dict) else str(s)).lower()
            for s in steps
        )
        results.ok(f"Context resolution ({'context step found' if has_context else 'no explicit context step, but succeeded'})")
    except Exception as e:
        results.fail("Context resolution", str(e))


# ── Test: Correlation ID ──────────────────────────────────────────────────────

async def test_correlation_id(client):
    print("\n[6] Correlation ID Propagation")
    try:
        custom_id = "e2e-test-correlation-12345"
        r = await client.post(f"{BASE}/api/chat", json={
            "prompt": "List all departments",
            "confirmed": False,
        }, headers={"X-Correlation-ID": custom_id}, timeout=20)
        resp_id = r.headers.get("X-Correlation-ID") or r.headers.get("x-correlation-id")
        assert resp_id == custom_id, f"Expected {custom_id}, got {resp_id}"
        results.ok("X-Correlation-ID header round-trips correctly")
    except Exception as e:
        results.fail("X-Correlation-ID propagation", str(e))

    # Auto-generated
    try:
        r = await client.post(f"{BASE}/api/chat", json={
            "prompt": "List all departments",
            "confirmed": False,
        }, timeout=20)
        resp_id = r.headers.get("X-Correlation-ID") or r.headers.get("x-correlation-id")
        assert resp_id, "No correlation ID header in response"
        results.ok(f"Auto-generated correlation ID: {resp_id[:12]}…")
    except Exception as e:
        results.fail("Auto-generated correlation ID", str(e))


# ── Test: Write Safety ────────────────────────────────────────────────────────

async def test_write_safety(client):
    print("\n[7] Write Safety — requires_confirmation gate")
    try:
        resp = await chat(client, "Update employee 1001 salary to 185000")
        requires = resp.get("requires_confirmation")
        assert requires is True, f"Expected requires_confirmation=True, got {requires}"
        results.ok("Update request returns requires_confirmation=True")

        conv_id = resp.get("session_id")

        # Confirm
        resp2 = await chat(client, "Update employee 1001 salary to 185000", conversation_id=conv_id, confirmed=True)
        assert resp2.get("success"), f"Confirmed update failed: {resp2.get('message')}"
        results.ok("Confirmed update executes successfully")

        # Restore original (best-effort)
        await chat(client, "Update employee 1001 salary to 165000", conversation_id=conv_id, confirmed=True)
    except Exception as e:
        results.fail("Write safety / confirmation gate", str(e))


# ── Test: Pin / Unpin ─────────────────────────────────────────────────────────

async def test_pin_unpin(client, conv_id: str):
    print("\n[8] Pin / Unpin (max 3)")
    if not conv_id:
        results.fail("Pin/Unpin", "No conversation_id available")
        return

    try:
        r = await client.patch(f"{BASE}/api/conversations/{conv_id}/pin", json={"pinned": True}, timeout=10)
        assert r.status_code == 200
        results.ok("Pin conversation succeeds")
    except Exception as e:
        results.fail("Pin conversation", str(e))

    try:
        r = await client.patch(f"{BASE}/api/conversations/{conv_id}/pin", json={"pinned": False}, timeout=10)
        assert r.status_code == 200
        results.ok("Unpin conversation succeeds")
    except Exception as e:
        results.fail("Unpin conversation", str(e))


# ── Test: Context Snapshots ───────────────────────────────────────────────────

async def test_snapshots(client, conv_id: str):
    print("\n[9] Context Snapshots")
    if not conv_id:
        results.fail("Context Snapshots", "No conversation_id")
        return
    try:
        r = await client.get(f"{BASE}/api/conversations/{conv_id}/snapshots", timeout=10)
        assert r.status_code == 200
        data = r.json()
        results.ok(f"GET /snapshots returns {len(data.get('snapshots', []))} snapshots")
    except Exception as e:
        results.fail("Context snapshots endpoint", str(e))


# ── Test: Idempotency (429 / 409 guard) ──────────────────────────────────────

async def test_idempotency(client):
    print("\n[10] Outbox Status Endpoint")
    try:
        r = await client.get(f"{BASE}/api/conversations/outbox/status", timeout=10)
        assert r.status_code == 200
        data = r.json()
        results.ok(f"Outbox status: {data}")
    except Exception as e:
        results.fail("Outbox status", str(e))


# ── Test: Suggestion Chips ────────────────────────────────────────────────────

async def test_suggestions(client):
    print("\n[11] Suggestion Chips on Ambiguous Query")
    try:
        resp = await chat(client, "Find employee")
        sugg = resp.get("suggestions", [])
        assert len(sugg) > 0, "No suggestions returned for ambiguous query"
        results.ok(f"Ambiguous query returns {len(sugg)} suggestion(s): {sugg[:2]}")
    except Exception as e:
        results.fail("Suggestion chips", str(e))


# ── Main ──────────────────────────────────────────────────────────────────────

async def main():
    print("=" * 60)
    print("Employee Management Agentic System — E2E Integration Tests")
    print("=" * 60)

    async with httpx.AsyncClient(timeout=30) as client:
        await test_health(client)
        await test_tools(client)
        conv_id = await test_conversations(client)
        read_conv_id = await test_read_queries(client)
        await test_context_resolution(client)
        await test_correlation_id(client)
        await test_write_safety(client)
        await test_pin_unpin(client, conv_id)
        await test_snapshots(client, conv_id or read_conv_id)
        await test_idempotency(client)
        await test_suggestions(client)

    success = results.summary()
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    asyncio.run(main())
