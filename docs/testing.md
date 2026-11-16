# Testing Guide

## Rust Regex Engine — 46 Unit Tests

```bash
cd rust-client
cargo test
```

**Expected output:**
```
running 43 tests
test regex_tests::test_get_all_employees_variations ... ok
test regex_tests::test_get_by_id_variations ... ok
...
test result: ok. 43 passed; 0 failed; 0 ignored
```

### Test categories

| Category | Tests |
|----------|-------|
| `get_all_employees` | 5 |
| `get_employee_by_id` | 5 |
| `search_employees` (dept) | 5 |
| `search_employees` (status) | 4 |
| `search_employees` (name) | 3 |
| `get_employee_count` | 4 |
| `get_department_list` | 4 |
| `add_employee` | 5 |
| `update_employee` | 5 |
| Priority ordering | 3 |

---

## Frontend TypeScript Build

```bash
cd frontend
npm run build
```

**Expected output:**
```
✓ built in ~350ms
dist/assets/index-*.css   ~20 kB
dist/assets/index-*.js    ~240 kB
```

---

## End-to-End Manual Test

With all services running:

```bash
# 1. Health check
curl http://localhost:5000/api/health

# 2. Get all tools
curl http://localhost:5000/api/tools

# 3. Chat — read operation
curl -s -X POST http://localhost:5000/api/chat \
  -H "Content-Type: application/json" \
  -d '{"prompt":"find employee 1001","confirmed":false}' | python3 -m json.tool

# 4. Chat — write (step 1 — no confirmation)
curl -s -X POST http://localhost:5000/api/chat \
  -H "Content-Type: application/json" \
  -d '{"prompt":"update employee 1001 status to Inactive","confirmed":false}' | python3 -m json.tool
# Expected: requires_confirmation: true

# 5. Chat — write (step 2 — confirmed)
curl -s -X POST http://localhost:5000/api/chat \
  -H "Content-Type: application/json" \
  -d '{"prompt":"update employee 1001 status to Inactive","confirmed":true}' | python3 -m json.tool
# Expected: success: true
```

---

## Rust Service Direct Test

```bash
# Health
curl http://localhost:5001/health

# Process a prompt
curl -s -X POST http://localhost:5001/process \
  -H "Content-Type: application/json" \
  -d '{"prompt":"show all employees","confirmed":false}'
```

---

## Swagger UI Testing

Navigate to **http://localhost:5000/docs** for the full interactive API — all endpoints are documented with request/response schemas.

---

## .NET MCP Server Standalone Test

```bash
cd rust-client
cargo test    # Runs the Python-less Rust test suite

# Or test MCP handshake directly
cd dotnet-mcp-server/EmployeeMcpServer
dotnet run   # starts STDIO server; send JSON-RPC manually
```
