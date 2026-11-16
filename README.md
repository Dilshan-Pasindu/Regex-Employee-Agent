# Employee Management Agentic Application

> **End-to-End Agentic Employee Management System** built with MCP, .NET 8/C#, Rust, Python/FastAPI, React, and MySQL.

---

## 🏗️ Architecture

```
React UI (5173)
    │  POST /api/chat
    ▼
Python FastAPI Backend (5000)
    │  POST /process
    ▼
Rust HTTP Service (5001)      ← Deterministic Regex intent engine
    │  STDIO (JSON-RPC 2.0)
    ▼
.NET 8 MCP Server             ← 8 MCP tools
    │  EF Core 8 (Pomelo)
    ▼
MySQL 8.x Database            ← InternAgenticDB
```

**Key design principle:** Zero LLM/AI anywhere in the stack. Intent detection is 100% deterministic Rust Regex patterns.

---

## 📦 Project Structure

```
Spillabs-intern/
├── database/                     # MySQL seed script
│   └── InternAgenticDB_Seed_MySQL.sql
├── dotnet-mcp-server/            # Phase 1 — .NET 8 MCP Server
│   └── EmployeeMcpServer/
│       ├── Models/               # EF Core entity models
│       ├── DTOs/                 # Input/output data transfer objects
│       ├── Services/             # EmployeeService (8 operations)
│       ├── Tools/                # MCP tool declarations
│       ├── Data/                 # EF Core DbContext
│       ├── Program.cs            # MCP STDIO server entry point
│       └── appsettings.example.json
├── rust-client/                  # Phase 2 + Phase 3 Layer 1 — Rust
│   ├── src/
│   │   ├── intent/               # Regex intent detector + extractor + patterns
│   │   ├── mcp/                  # MCP protocol client (STDIO)
│   │   ├── server/               # Axum HTTP server (port 5001)
│   │   ├── process/              # .NET child process manager
│   │   └── main.rs               # CLI mode + --server mode
│   └── tests/
│       └── regex_tests.rs        # 43 deterministic regex tests
├── backend/                      # Phase 3 Layer 2 — Python/FastAPI (port 5000)
│   ├── routers/
│   │   ├── chat.py               # POST /api/chat
│   │   ├── tools.py              # GET /api/tools
│   │   ├── health.py             # GET /api/health
│   │   └── conversations.py      # Session history
│   ├── services/
│   │   └── rust_client.py        # HTTP client for Rust service
│   ├── main.py                   # FastAPI app with Swagger UI
│   ├── models.py                 # Pydantic models
│   ├── requirements.txt
│   └── .env.example
├── frontend/                     # Phase 3 Layer 3 — React + Tailwind (port 5173)
│   ├── src/
│   │   ├── api/client.ts         # Fetch wrapper for FastAPI
│   │   ├── types/index.ts        # Shared TypeScript types
│   │   ├── components/
│   │   │   ├── ChatPanel.tsx     # Main chat timeline
│   │   │   ├── StepTrace.tsx     # Execution step accordion
│   │   │   ├── EmployeeTable.tsx # Sortable result table with status badges
│   │   │   ├── ConfirmationModal.tsx  # Write-op confirmation gate
│   │   │   └── ToolCatalogDrawer.tsx  # MCP tool sidebar
│   │   └── App.tsx               # Root shell with confirmation flow
│   └── vite.config.ts            # Vite + Tailwind + proxy config
└── docs/                         # Documentation
```

---

## 🚀 Quick Start

### Prerequisites

| Tool | Version | Purpose |
|------|---------|---------|
| .NET | 8.x | MCP Server |
| Rust | 1.75+ | Intent engine + HTTP layer |
| Python | 3.11+ | FastAPI backend |
| Node.js | 18+ | React frontend |
| MySQL | 8.x | Database |

---

### Step 1 — Database

```bash
mysql -u root -p < database/InternAgenticDB_Seed_MySQL.sql
```

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
# Edit appsettings.json — set your MySQL password
dotnet build
```

The server is started automatically by the Rust process — you do **not** run it manually.

---

### Step 3 — Rust HTTP Service

```bash
cd rust-client
cargo build --release
cargo run -- --server
# Listening on http://localhost:5001
```

For CLI mode (interactive terminal):
```bash
cargo run
```

---

### Step 4 — FastAPI Backend

```bash
cd backend
cp .env.example .env
python3 -m venv .venv
source .venv/bin/activate   # Windows: .venv\Scripts\activate
pip install -r requirements.txt
uvicorn main:app --host 0.0.0.0 --port 5000 --reload
# Swagger UI: http://localhost:5000/docs
```

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
| MySQL | 3306 | localhost:3306/InternAgenticDB |

---

## 🔧 MCP Tools (8 total)

| Tool | Description |
|------|-------------|
| `get_all_employees` | Returns all 20 employees |
| `get_employee_by_id` | Lookup by employee ID (1001–1020) |
| `search_employees` | Filter by department / designation / name / status |
| `get_department_list` | All 6 departments |
| `get_designation_list` | All 12 designations |
| `get_employee_count` | Total count, optionally filtered |
| `add_employee` | Insert new employee (requires confirmation) |
| `update_employee` | Update a field (requires confirmation) |

---

## 🛡️ Confirmation Flow (Write Operations)

Write operations (`add_employee`, `update_employee`) use a **two-step confirmation**:

1. **First request** — `confirmed: false` → returns extracted params, does NOT execute
2. **Frontend shows ConfirmationModal** — user reviews params
3. **Second request** — `confirmed: true` → executes MCP tool

---

## 📡 API Reference (FastAPI)

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/chat` | Submit a prompt, get intent + result |
| GET | `/api/tools` | MCP tool catalogue |
| GET | `/api/health` | Stack health check |
| GET | `/api/conversations` | List active sessions |
| GET | `/api/conversations/{id}` | Get session history |
| DELETE | `/api/conversations/{id}` | Clear session |

Full interactive docs: **http://localhost:5000/docs**

---

## 🧪 Running Tests

```bash
# Rust regex engine — 43 tests
cd rust-client
cargo test

# Frontend TypeScript check + build
cd frontend
npm run build
```

---

## ⚙️ Configuration

### `.env` (backend)

```env
RUST_SERVICE_URL=http://localhost:5001
PORT=5000
HOST=0.0.0.0
```

### `appsettings.json` (.NET — gitignored)

```json
{
  "ConnectionStrings": {
    "DefaultConnection": "Server=localhost;Port=3306;Database=InternAgenticDB;User Id=root;Password=YOUR_PASSWORD;"
  }
}
```

### `.env.local` (frontend — optional)

```env
VITE_API_URL=http://localhost:5000
```

---

## 🔍 Troubleshooting

| Problem | Fix |
|---------|-----|
| Rust service says "Failed to connect to MCP Server" | Run `dotnet build` in `dotnet-mcp-server/EmployeeMcpServer` first |
| `Connection string not found` | Ensure `appsettings.json` exists and has correct MySQL password |
| FastAPI returns 503 | Make sure Rust is running: `cargo run -- --server` |
| Frontend blank page | Check browser console; verify Rust + FastAPI are running |
| MySQL connection refused | Start MySQL: `sudo /usr/local/mysql/support-files/mysql.server start` |

---

## 📄 Documentation

- [Architecture](docs/architecture.md)
- [MCP Tool Catalogue](docs/mcp-tool-catalogue.md)
- [Regex Patterns](docs/regex-patterns.md)
- [Database Setup](docs/database-setup.md)
- [Testing Guide](docs/testing.md)
