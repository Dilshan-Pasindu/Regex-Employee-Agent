# Architecture — Employee Management Agentic Application

## System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                    USER INTERFACE (Port 5173)                        │
│                   React + Vite + Tailwind CSS                        │
│                                                                      │
│  ChatPanel   StepTrace   EmployeeTable   ConfirmModal   ToolDrawer  │
└───────────────────────────────┬─────────────────────────────────────┘
                                │ POST /api/chat
                                │ GET  /api/tools
                                │ GET  /api/health
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                   PYTHON / FASTAPI BACKEND (Port 5000)               │
│                                                                      │
│  POST /api/chat          → routers/chat.py                          │
│  GET  /api/tools         → routers/tools.py                         │
│  GET  /api/health        → routers/health.py                        │
│  GET  /api/conversations → routers/conversations.py                 │
│                                                                      │
│  In-memory session store (Dict[session_id, List[ConversationEntry]])│
│  Swagger UI auto-generated at /docs                                 │
└───────────────────────────────┬─────────────────────────────────────┘
                                │ POST /process
                                │ GET  /tools
                                │ GET  /health
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                   RUST HTTP SERVICE (Port 5001)                      │
│                      employee-mcp-client                             │
│                                                                      │
│  Axum HTTP layer (server/mod.rs)                                    │
│  ├── POST /process  → Intent detection → MCP tool call              │
│  ├── GET  /tools    → Discovered tool catalogue                     │
│  └── GET  /health   → Ping dotnet process                           │
│                                                                      │
│  Intent Engine (pure Regex, NO LLM)                                 │
│  ├── intent/detector.rs    — priority-ordered classifier            │
│  ├── intent/patterns.rs    — once_cell Lazy<Regex> patterns         │
│  └── intent/extractor.rs   — named capture group extraction         │
│                                                                      │
│  MCP Client (mcp/client.rs)                                         │
│  ├── MCP initialize handshake (JSON-RPC 2.0 over STDIO)             │
│  ├── tools/list discovery                                            │
│  └── tools/call dispatch                                            │
│                                                                      │
│  Process Manager (process/mod.rs)                                   │
│  └── Tokio child process — STDIO pipes to .NET server               │
└───────────────────────────────┬─────────────────────────────────────┘
                                │ STDIO (JSON-RPC 2.0)
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                   .NET 8 / C# MCP SERVER (STDIO)                    │
│                   EmployeeMcpServer                                  │
│                                                                      │
│  8 MCP Tools (Tools/EmployeeTools.cs)                               │
│  ├── get_all_employees                                              │
│  ├── get_employee_by_id                                             │
│  ├── search_employees                                               │
│  ├── get_department_list                                            │
│  ├── get_designation_list                                           │
│  ├── get_employee_count                                             │
│  ├── add_employee                                                   │
│  └── update_employee                                                │
│                                                                      │
│  EmployeeService.cs — All DB operations with validation             │
│  EmployeeDbContext.cs — EF Core 8 + Pomelo MySQL provider           │
│                                                                      │
│  STDOUT discipline: Console.SetOut(Console.Error)                   │
│  All logs → STDERR only (protects JSON-RPC STDOUT channel)         │
└───────────────────────────────┬─────────────────────────────────────┘
                                │ EF Core 8 (Pomelo)
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                   MySQL 8.x (Port 3306)                             │
│                   Database: InternAgenticDB                          │
│                                                                      │
│  Tables:                                                            │
│  ├── Departments (6 rows)                                           │
│  ├── Designations (12 rows)                                         │
│  └── Employees (20 rows, IDs 1001–1020)                             │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Confirmation Flow (Write Operations)

Write operations (`add_employee`, `update_employee`) use a mandatory two-step flow:

```
sequenceDiagram
    User->>React: "Add employee Jane Doe email j@co.com dept IT..."
    React->>FastAPI: POST /api/chat { prompt, confirmed: false }
    FastAPI->>Rust: POST /process { prompt, confirmed: false }
    Rust->>Rust: Detect intent = AddEmployee, extract params
    Rust-->>FastAPI: { requires_confirmation: true, params: {...} }
    FastAPI-->>React: { requires_confirmation: true, params: {...} }
    React->>User: Show ConfirmationModal with extracted params
    User->>React: Click "Confirm"
    React->>FastAPI: POST /api/chat { prompt, confirmed: true }
    FastAPI->>Rust: POST /process { prompt, confirmed: true }
    Rust->>DotNet: tools/call add_employee(params)
    DotNet->>MySQL: INSERT INTO Employees ...
    MySQL-->>DotNet: OK
    DotNet-->>Rust: { success: true, data: {...} }
    Rust-->>FastAPI: { success: true, result: {...} }
    FastAPI-->>React: { success: true, result: {...} }
    React->>User: Show success + new employee card
```

---

## Intent Detection Priority

The Rust detector evaluates patterns in strict priority order to avoid ambiguity:

| Priority | Intent | Reason |
|----------|--------|--------|
| 1 | `UpdateEmployee` | Must match before GetById: `update employee 1001 ...` contains a 4-digit ID |
| 2 | `GetEmployeeById` | Must match before GetAll: `show employee 1001` |
| 3 | `AddEmployee` | Explicit creation keywords |
| 4 | `GetEmployeeCount` | Count/how-many patterns |
| 5 | `GetDepartmentList` | Department listing |
| 6 | `SearchEmployees` | Filtered search |
| 7 | `GetAllEmployees` | Broadest pattern — last resort |

---

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Rust for intent engine | Zero-dependency, deterministic, type-safe Regex with `once_cell::Lazy` compilation |
| .NET MCP over STDIO | Follows MCP spec exactly; Rust manages process lifetime |
| FastAPI for backend | Self-documenting Swagger UI; widely used for agentic gateways; async-native |
| MySQL only | Apple Silicon Mac constraint; SQL Server has no ARM64 support |
| Two-step confirmation | Prevents accidental writes; UX-friendly with param review |
| In-memory session store | Sufficient for demo; replace with Redis for production |
