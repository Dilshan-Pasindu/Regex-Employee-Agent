# End-to-End Agentic Architecture

## 1. Architectural Overview

The application architecture follows a strict decoupled pipeline:

```
┌─────────────────────────────────┐
│       React + Vite Frontend     │
└────────────────┬────────────────┘
                 │ HTTP (JSON)
┌────────────────▼────────────────┐
│      FastAPI Python Backend     │
└────────────────┬────────────────┘
                 │ HTTP (JSON)
┌────────────────▼────────────────┐
│         Rust MCP Client         │
│  - Conversational Memory        │
│  - Intent Engine (Regex)        │
│  - Error Classifier             │
│  - Prompt Suggestion Engine     │
│  - Multi-Step Task Planner      │
│  - Post-Filter & Projection     │
└────────────────┬────────────────┘
                 │ STDIO (MCP Protocol / JSON-RPC 2.0)
┌────────────────▼────────────────┐
│      .NET 8 MCP Server          │
│  - Tool Handlers                │
│  - Entity Framework Core        │
└────────────────┬────────────────┘
                 │ TCP
┌────────────────▼────────────────┐
│         MySQL Database          │
└─────────────────────────────────┘
```

---

## 2. Integrated Processing Pipeline

Every user prompt traverses the deterministic intelligence pipeline:

```
                       User Prompt
                            │
                            ▼
              Conversation Context Resolver
           (Resolves pronouns & follow-ups)
                            │
                            ▼
                     Intent Detector
                            │
                            ▼
             Parameter & Field Extractor
                            │
                            ▼
                     Input Validator
                            │
                            ▼
                   Error Classifier
                           / \
               Valid Prompt   Invalid / Incomplete Prompt
                     /             \
                    ▼               ▼
              Task Planner     Suggestion Engine
                    │               │
                    ▼               ▼
              Plan Validator   Generate Contextual
                    │          Suggestions & Explanation
                    ▼               │
           Write Confirmation?      ▼
              [Yes / No]      Return Suggestions
                    │
                    ▼
             Execution Engine
         (Invokes MCP over STDIO)
                    │
                    ▼
          Post-Filter & Projection
         (Salary comparison, fields)
                    │
                    ▼
            Response Formatter
                    │
                    ▼
        Update Conversation Context
                    │
                    ▼
              Final Result
```

---

## 3. Key Design Principles

1. **Deterministic Intelligence**: Zero LLMs. All intent recognition, extraction, task planning, and suggestion generation use deterministic Regex and structured rules.
2. **Protocol Conformance**: Communication with the .NET backend strictly uses the Model Context Protocol over STDIO. No REST APIs between Rust and .NET.
3. **Execution Safety**: All write operations require explicit confirmation. Diagnostic logs are sent to STDERR, keeping STDOUT pristine for protocol messages.
4. **Resilience**: Embedded YAML fallbacks, in-memory context preservation on errors, and actionable suggestions on invalid queries ensure zero runtime crashes.
