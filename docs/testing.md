# Testing Guide & Prompt Test Cases

## Assignment Prompt Test Suite (85 Automated Tests)

The assignment requires:
> *"Prepare at least 30 prompt test cases, including alternate phrasing, case variations, missing parameters, invalid IDs, ambiguous requests, unsupported requests, and write-operation confirmation."*

These test cases are implemented across **automated test files** under `rust-client/tests/`:
1. **[regex_tests.rs](file:///Users/dilshanpasindu/Documents/GitHub/Spillabs-intern/rust-client/tests/regex_tests.rs)** (57 tests)
2. **[suggestion_tests.rs](file:///Users/dilshanpasindu/Documents/GitHub/Spillabs-intern/rust-client/tests/suggestion_tests.rs)** (9 tests)
3. **[planner_tests.rs](file:///Users/dilshanpasindu/Documents/GitHub/Spillabs-intern/rust-client/tests/planner_tests.rs)** (8 tests)
4. **[context_tests.rs](file:///Users/dilshanpasindu/Documents/GitHub/Spillabs-intern/rust-client/tests/context_tests.rs)** (9 tests)
5. **[fuzzy.rs](file:///Users/dilshanpasindu/Documents/GitHub/Spillabs-intern/rust-client/src/intent/fuzzy.rs)** (2 tests)

---

### Categorized Prompt Test Cases Matrix (35+ Prompts)

| # | Required Category | Prompt Input | Expected Intent / Behavior | Test Location |
|---|---|---|---|---|
| **1** | Alternate Phrasing | `"view all staff"` | `GetAllEmployees` | `regex_tests::test_05` |
| **2** | Alternate Phrasing | `"employee records"` | `GetAllEmployees` | `regex_tests::test_06` |
| **3** | Alternate Phrasing | `"show all employees details"` | `GetAllEmployees` | `regex_tests::test_06b` |
| **4** | Alternate Phrasing | `"gimme the 1001 details"` | `GetEmployeeById` (ID: 1001) | `regex_tests::test_46` |
| **5** | Alternate Phrasing | `"give me employee 1001"` | `GetEmployeeById` (ID: 1001) | `regex_tests::test_47` |
| **6** | Alternate Phrasing | `"how many on leave staff"` | `GetEmployeeCount` (status: On Leave) | `regex_tests::test_26d` |
| **7** | Alternate Phrasing | `"find employee 1001 division and wage"` | `GetEmployeeById` (field aliases) | `regex_tests::test_44` |
| **8** | Case Variation | `"DISPLAY ALL EMPLOYEES"` | `GetAllEmployees` (uppercase) | `regex_tests::test_03` |
| **9** | Case Variation | `"sHoW aLl eMpLoYeEs"` | `GetAllEmployees` (mixed case) | `regex_tests::test_04` |
| **10** | Case Variation | `"list employees"` | `GetAllEmployees` (lowercase) | `regex_tests::test_02` |
| **11** | Case Variation | `"COUNT EMPLOYEES"` | `GetEmployeeCount` (uppercase) | `regex_tests::test_23` |
| **12** | Case Variation | `"SHOW ALL DEPARTMENTS"` | `GetDepartmentList` (uppercase) | `regex_tests::test_29` |
| **13** | Missing Parameters | `"Find employee"` | Classification: `MissingParameter`<br>Suggests: `Find employee 1001`, email | `suggestion_tests::test_missing_employee_id` |
| **14** | Missing Parameters | `"Update employee 1001"` | Classification: `MissingParameter`<br>Suggests: update salary, dept, desig | `suggestion_tests::test_incomplete_update_request` |
| **15** | Missing Parameters | `"Find employee 1001"` (plan check) | Plan validator requires ID | `planner_tests::test_simple_employee_lookup_plan` |
| **16** | Invalid IDs | `"Find employee abc"` | Classification: `InvalidParameter`<br>Explains numeric ID required | `suggestion_tests::test_invalid_employee_id` |
| **17** | Invalid IDs | `"find employee abc"` (detector) | Routed to `Intent::Unknown` (prevents empty search dump) | `regex_tests::test_invalid_employee_id` |
| **18** | Ambiguous Requests | `"Show me IT workers"` | Classification: `AmbiguousIntent`<br>Suggests structured IT dept queries | `suggestion_tests::test_department_informal_query` |
| **19** | Ambiguous Requests | `"some unfamiliar text about 1001"` | Extracts ID 1001, suggests operations for 1001 | `suggestion_tests::test_unknown_prompt_with_numeric_id...` |
| **20** | Ambiguous Requests | `"What is his salary?"` | Memory: resolves `"his"` ➔ `1001` | `context_tests::test_store_and_resolve_employee_pronoun_salary` |
| **21** | Ambiguous Requests | `"What about HR?"` | Memory: pivots search to `"HR"` | `context_tests::test_resolve_department_switch` |
| **22** | Ambiguous Requests | `"Only show their names"` | In-memory projection cache hit | `context_tests::test_resolve_in_memory_projection` |
| **23** | Unsupported Requests | `"What's the weather today?"` | Classification: `UnrelatedPrompt`<br>Outlines employee system capabilities | `suggestion_tests::test_unrelated_weather_prompt` |
| **24** | Unsupported Requests | `"Delete employee 1001"` | Classification: `UnsupportedOperation`<br>Redirects to view/update; never deletes | `suggestion_tests::test_unsupported_delete_operation` |
| **25** | Unsupported Requests | `"What is the weather today"` (detector) | Returns `Intent::Unknown` | `regex_tests::test_38` |
| **26** | Unsupported Requests | `"random gibberish that makes no sense"` | Returns `Intent::Unknown` | `regex_tests::test_39` |
| **27** | Unsupported Requests | `"DROP TABLE Employees; --"` | SQL injection rejected as `Intent::Unknown` | `regex_tests::test_40` |
| **28** | Unsupported Requests | `""` (empty string) | Rejected as `Intent::Unknown` | `regex_tests::test_41` |
| **29** | Write-Operation Confirmation | `"Add employee Jane Doe email jane@example.com"` | `requires_confirmation = true` | `planner_tests::test_write_operation_confirmation_flags` |
| **30** | Write-Operation Confirmation | `"Update employee 1001 salary to 180000"` | `requires_confirmation = true` | `planner_tests::test_write_operation_confirmation_flags` |
| **31** | Write-Operation Confirmation | `"Update employee 1001 status to Active"` | `requires_confirmation = true` | `regex_tests::test_42` |
| **32** | Read-Operation Safety | `"Show all employees"` | `requires_confirmation = false` | `regex_tests::test_43` |
| **33** | Read-Operation Safety | `"Find employee 1001"` | `requires_confirmation = false` | `regex_tests::test_43` |
| **34** | Read-Operation Safety | `"count employees"` | `requires_confirmation = false` | `regex_tests::test_43` |
| **35** | Read-Operation Safety | `"show departments"` | `requires_confirmation = false` | `regex_tests::test_43` |

---

## Running the Automated Test Suite

```bash
# Run all 77 unit and integration tests
cargo test --manifest-path rust-client/Cargo.toml
```

**Expected output:**
```
test result: ok. 51 passed (regex_tests)
test result: ok. 9 passed  (suggestion_tests)
test result: ok. 8 passed  (planner_tests)
test result: ok. 9 passed  (context_tests)
Total: 77 passed; 0 failed; 0 ignored
```

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
