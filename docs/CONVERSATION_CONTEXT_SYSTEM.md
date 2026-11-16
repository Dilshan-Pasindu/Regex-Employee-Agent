# Conversational Context and Memory System

## 1. Overview
The Conversational Context and Memory System enables session-isolated memory in the Rust MCP Client. Users can ask follow-up questions, refer to previously inspected employees using pronouns (`"his"`, `"her"`, `"their"`, `"that employee"`), switch filter contexts, or filter prior results without retyping details.

The system is strictly deterministic and in-memory, requiring no external databases or LLMs.

---

## 2. Conversation Context Model

```rust
pub struct ConversationContext {
    pub last_intent: Option<String>,
    pub last_employee_id: Option<i64>,
    pub last_employee_name: Option<String>,
    pub last_employee_email: Option<String>,
    pub last_department: Option<String>,
    pub last_designation: Option<String>,
    pub last_salary: Option<f64>,
    pub last_status: Option<String>,
    pub last_requested_fields: Vec<String>,
    pub last_result_data: Option<serde_json::Value>,
    pub last_result_count: Option<usize>,
    pub last_operation_success: Option<bool>,
    pub turn_count: usize,
}
```

Sessions are isolated using a global in-memory `SESSION_STORE: Arc<Mutex<HashMap<String, ConversationContext>>>`. Terminal CLI users use a single session, while web API users maintain isolated sessions via their unique `session_id`.

---

## 3. Pronoun and Reference Resolution

The context resolver matches pronouns and follow-up phrases deterministically:

| Follow-Up Input | Active Context | Resolved Interpretation |
|---|---|---|
| `"What is his salary?"` | `last_employee_id = 1001` | `Find employee 1001 salary` |
| `"What about his department?"` | `last_employee_id = 1001` | `Find employee 1001 department name` |
| `"Show his email too"` | `last_employee_id = 1001` | `Find employee 1001 email` |
| `"Update that employee salary to 160000"` | `last_employee_id = 1001` | `Update employee 1001 salary to 160000` |
| `"What about HR?"` | `last_department = "IT"` | `Search employees in HR` |
| `"Only show their names"` | `last_result_data = [...]` | Projects previous result in-memory to names |

---

## 4. Context Lifecycle Rules

1. **Update on Success**: Context entities are only updated after a tool execution succeeds.
2. **Preservation on Failure**: When an invalid prompt or failed command occurs, existing valid context is preserved.
3. **Explicit Reset**: Prompts like `"clear context"`, `"reset"`, or `"new conversation"` wipe the session context cleanly.
4. **Session Isolation**: Contexts for different session IDs never leak or interfere with each other.
