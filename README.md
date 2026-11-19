# Employee Management Agentic System

> **Production-grade, end-to-end Agentic Employee Management System** built with MCP, .NET 8/C#, Rust, Python/FastAPI, React/TypeScript, and MySQL — with zero LLM/AI dependency.

---

## 🏗️ Architecture

```
React UI (Vite, port 5173)
    │  POST /api/chat  ·  X-Correlation-ID header
    ▼
Python FastAPI Backend (port 5000)
    │  CorrelationIdMiddleware · OutboxWorker · CQRS facades
    │  POST http://localhost:5001/process
    ▼
Rust HTTP Service (Axum, port 5001)       ← 100% deterministic Regex intent engine
    │  STDIO (JSON-RPC 2.0 / MCP protocol)
    ▼
.NET 8 MCP Server (STDIO child process)   ← 8 MCP tools, EF Core 8 (Pomelo)
    │  Entity Framework Core
    ▼
MySQL 8.x  ·  InternAgenticDB             ← All persistent state
```

**Key design principle:** Zero LLM/AI anywhere in the stack. All intent detection, parameter extraction, and query planning is done by **100% deterministic Rust Regex patterns**.

---

## 🗂️ Project Structure

```
Spillabs-intern/
├── config/                              # Shared vocabulary configuration
│   └── vocabulary.yaml                 # Verbs, field aliases, department/status synonyms
├── database/
│   └── InternAgenticDB_Seed_MySQL.sql  # MySQL schema + 20 seed employees
├── dotnet-mcp-server/
│   └── EmployeeMcpServer/
│       ├── Models/                     # EF Core entity models
│       ├── DTOs/                       # Input/output data transfer objects
│       ├── Services/                   # EmployeeService (8 MCP operations)
│       ├── Tools/                      # MCP tool declarations
│       ├── Data/                       # AppDbContext (Pomelo/MySQL)
│       ├── Program.cs                  # MCP STDIO server entry point
│       └── appsettings.example.json
├── rust-client/
│   ├── config/vocabulary.yaml          # Runtime vocabulary (copied from /config)
│   ├── src/
│   │   ├── intent/                     # Regex intent detector, extractor, planner
│   │   ├── mcp/                        # MCP protocol client (STDIO JSON-RPC)
│   │   ├── server/                     # Axum HTTP server (port 5001)
│   │   ├── process/                    # .NET child process manager
│   │   └── main.rs
│   └── tests/                          # 85 deterministic regression tests
├── backend/
│   ├── routers/
│   │   ├── chat.py                     # POST /api/chat
│   │   ├── conv_mgmt.py                # Full conversation CRUD + snapshots
│   │   ├── tools.py                    # GET /api/tools
│   │   ├── health.py                   # GET /api/health
│   │   └── conversations.py            # Legacy session endpoints
│   ├── services/
│   │   ├── rust_client.py              # HTTP client for Rust service
│   │   ├── reliability_service.py      # Distributed locking + idempotency
│   │   ├── outbox_service.py           # Transactional Outbox pattern
│   │   ├── outbox_worker.py            # Background durable event worker
│   │   └── cqrs_facades.py             # ConversationCommandService / QueryService
│   ├── telemetry.py                    # CorrelationIdMiddleware + structured logging
│   ├── db.py                           # SQLAlchemy async engine + session factory
│   ├── models.py                       # Pydantic + SQLAlchemy ORM models
│   ├── requirements.txt
│   └── .env.example
└── frontend/
    └── src/
        ├── api/client.ts               # Typed fetch wrapper (sendChat, snapshots, etc.)
        ├── types/index.ts              # Shared TypeScript interfaces
        ├── components/
        │   ├── ChatPanel.tsx           # Message timeline with write-safety badges
        │   ├── StepTrace.tsx           # Execution step accordion + ExecutionPlanPanel
        │   ├── EmployeeTable.tsx       # Sortable result table with status badges
        │   ├── ConversationSidebar.tsx # Pinnable conversation list (max 3 pins)
        │   ├── ConfirmationModal.tsx   # Write-operation two-step confirmation gate
        │   └── ToolCatalogDrawer.tsx   # MCP tool catalogue sidebar
        └── App.tsx                     # Root shell: toast system, confirmation flow
```

---

## 🚀 Quick Start

### Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| .NET | 8.x | MCP Server |
| Rust | 1.75+ | Intent engine + HTTP service |
| Python | 3.11+ | FastAPI backend |
| Node.js | 18+ | React frontend |
| MySQL | 8.x | All persistent state |

---

### Step 1 — Database

```bash
mysql -u root -p < database/InternAgenticDB_Seed_MySQL.sql
```

This creates the `InternAgenticDB` database and all required tables (see [Database Schema](#-database-schema) below), and seeds 20 employees.

Verify:
```bash
mysql -u root -p -e "USE InternAgenticDB; SELECT COUNT(*) FROM Employees;"
# Expected: 20
```

---

### Step 2 — .NET MCP Server

```bash
cd dotnet-mcp-server/EmployeeMcpServer
cp appsettings.example.json appsettings.json
# Edit appsettings.json — set your MySQL connection string password
dotnet build
```

> **Note:** The MCP server is launched automatically as a child process by the Rust service. You do **not** run it manually.

---

### Step 3 — Rust HTTP Service

```bash
cd rust-client
cargo build --release
cargo run --release -- --server
# Listening on http://localhost:5001
```

For interactive CLI mode (no React required):
```bash
cargo run --release
```

---

### Step 4 — FastAPI Backend

```bash
cd backend
cp .env.example .env
python3 -m venv .venv
source .venv/bin/activate        # Windows: .venv\Scripts\activate
pip install -r requirements.txt
uvicorn main:app --host 0.0.0.0 --port 5000 --reload
```

Swagger UI: **http://localhost:5000/docs**

---

### Step 5 — React Frontend

```bash
cd frontend
npm install
npm run dev
# App: http://localhost:5173
```

---

## 🌐 Service Ports

| Service | Port | URL |
|---------|------|-----|
| React (Vite dev) | 5173 | http://localhost:5173 |
| FastAPI backend | 5000 | http://localhost:5000/docs |
| Rust HTTP service | 5001 | http://localhost:5001/health |
| MySQL | 3306 | `localhost:3306/InternAgenticDB` |

---

## 🔧 MCP Tools (8 total)

| Tool | Description | Write Op |
|------|-------------|----------|
| `get_all_employees` | Returns all employees with field projection | ✗ |
| `get_employee_by_id` | Lookup by employee ID (1001–1020) | ✗ |
| `search_employees` | Filter by department / designation / name / status | ✗ |
| `get_department_list` | All department names | ✗ |
| `get_designation_list` | All designation names | ✗ |
| `get_employee_count` | Total count, optionally filtered | ✗ |
| `add_employee` | Insert new employee record | ✓ requires confirmation |
| `update_employee` | Update a field on an existing employee | ✓ requires confirmation |

---

## 🛡️ Write-Safety Confirmation Flow

Write operations (`add_employee`, `update_employee`) use a **two-step confirmation gate**:

```
User: "Update employee 1001 salary to 185000"
  │
  ▼  confirmed=false
FastAPI → Rust → (params extracted, NOT executed)
  ▼
Backend sets requires_confirmation=true + PENDING_CONFIRMATION OperationRecord
  ▼
React shows ConfirmationModal with extracted params
  │
  ├─ User clicks "Cancel" → cancelled, no DB change
  │
  └─ User clicks "Confirm"
        │  confirmed=true
        ▼
     FastAPI → Rust → .NET MCP → MySQL UPDATE
```

---

## 📡 API Reference

### Chat

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/chat` | Submit a prompt; returns intent, result, execution plan, correlation ID |

**Request body:**
```json
{
  "prompt": "Find all employees in IT earning more than 150000",
  "conversation_id": "uuid-or-null",
  "confirmed": false
}
```

**Response:**
```json
{
  "session_id": "uuid",
  "intent": "search_employees",
  "tool": "search_employees",
  "params": { "department": "IT", "min_salary": 150000 },
  "steps": ["Intent Recognized", "Parameters Extracted", "MCP Tool Invoked"],
  "result": [ ... ],
  "success": true,
  "message": "Found 3 employees.",
  "requires_confirmation": false,
  "execution_plan": { "steps": [ ... ] },
  "correlation_id": "abc123..."
}
```

### Conversations

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/conversations` | Create new conversation |
| `GET` | `/api/conversations` | List conversations (paginated) |
| `GET` | `/api/conversations/search?q=` | Full-text search conversations |
| `GET` | `/api/conversations/outbox/status` | Outbox queue health |
| `GET` | `/api/conversations/{id}` | Get single conversation |
| `PATCH` | `/api/conversations/{id}` | Rename conversation |
| `PATCH` | `/api/conversations/{id}/pin` | Pin / unpin (max 3 pinned) |
| `DELETE` | `/api/conversations/{id}` | Delete conversation + all messages |
| `GET` | `/api/conversations/{id}/context` | Get active context JSON + version |
| `GET` | `/api/conversations/{id}/snapshots` | List context snapshots |
| `GET` | `/api/conversations/{id}/messages` | Full message history |

### Other

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/tools` | MCP tool catalogue |
| `GET` | `/api/health` | Stack health check |

Full interactive docs: **http://localhost:5000/docs**

---

## 🗄️ Database Schema

All 12 tables are auto-created by `init_db()` on FastAPI startup:

| Table | Purpose |
|-------|---------|
| `Employees` | Core employee records (seeded, managed by .NET MCP) |
| `Conversations` | Chat session metadata (title, is_pinned, is_active) |
| `Messages` | Persistent chat messages with role + metadata_json |
| `ConversationContexts` | Active context blob with optimistic concurrency (version) |
| `ConversationLocks` | Distributed advisory locks (prevents concurrent processing) |
| `IdempotencyRecords` | Idempotency keys for exactly-once request processing |
| `OutboxEvents` | Transactional Outbox events (exponential-backoff retry) |
| `OperationRecords` | Durable operation tracking (PENDING → COMPLETED/FAILED) |
| `ContextSnapshots` | Point-in-time context snapshots every 5 messages |

---

## 📐 Key Subsystems

### Deterministic Regex Intent Engine (Rust)

- **No LLM** — all intent classification via compiled regex patterns
- Vocabulary loaded from `config/vocabulary.yaml` (hot-reloadable synonyms)
- Supports: `get_all_employees`, `get_employee_by_id`, `search_employees`, `add_employee`, `update_employee`, `batch_update`, `get_department_list`, `get_designation_list`, `get_employee_count`
- Field projection ("`show only name and salary`") — applied in-memory in Rust
- Context resolution ("`show his salary`" → resolves "his" to last seen employee ID)
- Suggestion chips on ambiguous/incomplete queries

### Transactional Outbox (Phase 4)

- Every chat event emits an `OutboxEvent` atomically within the same DB transaction
- Background `outbox_worker` processes events with exponential backoff (max 5 retries)
- Guarantees at-least-once delivery even on backend crash/restart

### CQRS Facades (Phase 5)

- `ConversationCommandService` — all write operations (create, rename, pin, delete, context update)
- `ConversationQueryService` — all read operations (list, search, get messages, snapshots)
- Context snapshots auto-created every 5 messages or on first 2 messages

### Structured Telemetry (Phase 5)

- Every request gets a `X-Correlation-ID` header (UUID, generated by middleware if absent)
- Correlation ID propagated through: FastAPI → Rust HTTP client headers → Outbox events → frontend footer
- JSON-structured log output with `correlation_id` field on every log line

---

## 🧪 Running Tests

```bash
# Rust deterministic intent engine — 85 tests
cd rust-client
cargo test
# Expected: 85 passed, 0 failed

# Frontend TypeScript compile check + production build
cd frontend
npm run build
# Expected: 0 errors, built in ~1s
```

---

## ⚙️ Configuration

### `backend/.env`

```env
RUST_SERVICE_URL=http://localhost:5001
PORT=5000
HOST=0.0.0.0
DB_HOST=127.0.0.1
DB_PORT=3306
DB_USER=root
DB_PASSWORD=your_password_here
DB_NAME=InternAgenticDB
```

### `dotnet-mcp-server/EmployeeMcpServer/appsettings.json`

```json
{
  "ConnectionStrings": {
    "DefaultConnection": "Server=localhost;Port=3306;Database=InternAgenticDB;User Id=root;Password=YOUR_PASSWORD;"
  }
}
```

### `frontend/.env.local` (optional)

```env
VITE_API_URL=http://localhost:5000
```

---

## 🎨 UI Features

| Feature | Description |
|---------|-------------|
| Conversation Sidebar | Pinnable conversation list (max 3 pins), rename on double-click, delete |
| Write-Safety Badges | Amber `PENDING CONFIRMATION` banner on write-op messages |
| Execution Plan Panel | Accordion showing structured plan steps, tool pills, arguments |
| Step Trace Accordion | Detailed execution step timeline with Lucide icons |
| Employee Table | Sortable columns, status badges (Active/Inactive), field projection |
| Suggestion Chips | Clickable refinement hints on ambiguous queries |
| Toast Notifications | 429 (busy) and 409 (duplicate) errors show inline amber toasts |
| Correlation ID Footer | Truncated correlation ID shown in each agent message footer |
| Dark / Light Mode | Full theme toggle with CSS custom property system |
| Voice Input | Web Speech API dictation into the chat input box |
| Confirmation Modal | Params review before any write operation executes |

---

## 🔍 Troubleshooting

| Problem | Fix |
|---------|-----|
| Rust fails: "Failed to connect to MCP Server" | Run `dotnet build` in `dotnet-mcp-server/EmployeeMcpServer` first |
| `Connection string not found` (.NET) | Ensure `appsettings.json` exists with correct password |
| FastAPI returns 503 | Ensure Rust is running: `cargo run --release -- --server` |
| `pool_pre_ping` error on aiomysql | Confirm `pool_pre_ping=False` is set in `backend/db.py` |
| Frontend blank page | Check browser console; verify Rust + FastAPI are running |
| MySQL connection refused | Start MySQL: `sudo /usr/local/mysql/support-files/mysql.server start` |
| Toast shows 429 on rapid send | Expected — conversation lock prevents concurrent processing |

---

## 📋 Development Phases

| Phase | Scope | Commit |
|-------|-------|--------|
| Phase 0 | Codebase audit & planning | — |
| Phase 1 | DB tables, message persistence, conversation CRUD, pin/unpin sidebar | `1a7e248` |
| Phase 2 | ConversationContexts, optimistic concurrency, Rust context passing | `7297aca` |
| Phase 3 | Distributed locking, idempotency, message lifecycle PENDING→COMPLETED | `58d39de` |
| Phase 4 | Transactional Outbox, durable worker, OperationRecords | `9fe14b6` |
| Phase 5 | ContextSnapshots, CQRS facades, structured telemetry, correlation ID | `86dab4f` |
| Phase 6 | Agentic capabilities — execution plan, write-safety lifecycle, Outbox events | `48d3ecf` |
| Phase 7 | React refinements — ExecutionPlanPanel, write-safety badges, toast system | `40341c6` |
| Phase 8 | End-to-end integration testing & documentation | *current* |

---

## 📄 Additional Documentation

- [Architecture Deep-Dive](docs/architecture.md)
- [MCP Tool Catalogue](docs/mcp-tool-catalogue.md)
- [Regex Pattern Reference](docs/regex-patterns.md)
- [Database Setup Guide](docs/database-setup.md)
- [Testing Guide](docs/testing.md)
