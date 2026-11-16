# Context-Aware Prompt Suggestion and Recovery System

## 1. Overview
The Context-Aware Prompt Suggestion and Recovery System provides intelligent recovery and helpful prompt suggestions whenever a user's prompt cannot be executed directly. Rather than presenting generic responses, suggestions are derived deterministically based on:
- Extracted entities (e.g. employee IDs, department names)
- Recognized partial intent
- Conversation context and history
- Classified error reasons

This system operates without any LLM, using deterministic Regex and template-driven logic.

---

## 2. Error Classification Architecture

Every prompt that fails to produce a valid execution plan is passed through the `error_classifier::classify_prompt_error` engine. The failure is classified into one of the following structured categories:

| Error Type | Description | Example Input | Explanation Provided |
|---|---|---|---|
| `MissingParameter` | Action is recognized, but a required identifier or value is missing | `"Find employee"` | Asks for employee ID or email |
| `MissingParameter` (Update) | Incomplete update missing fields | `"Update employee 1001"` | Asks what field to update for employee 1001 |
| `InvalidParameter` | Syntactically invalid value (e.g. alphabetical ID) | `"Find employee abc"` | Explains employee ID must be numeric |
| `UnsupportedOperation` | Attempted operation not supported by system | `"Delete employee 1001"` | Explains deletion is unsupported, suggests view/update |
| `AmbiguousIntent` | Informal or partial query | `"Show me IT workers"` | Suggests structured queries for the IT department |
| `UnrelatedPrompt` | Out-of-domain prompt (weather, food, chit-chat) | `"What's the weather today?"` | Clarifies system boundaries and shows supported capabilities |
| `ValidationError` | Parameter constraint violation (e.g. negative salary) | `"Add employee Jane Doe salary -500"` | Highlights rule violation |
| `McpToolError` | Error returned by downstream MCP tool | Tool invocation failure | Relays error description |
| `McpConnectionError` | Process or transport communication failure | Subprocess crash | Explains technical failure |

---

## 3. Suggestion Engine Pipeline

The suggestion engine executes as follows:
1. **Error Classification**: Identifies the primary barrier preventing execution.
2. **Context Entity Extraction**: Retrieves any extracted entity from the current prompt or previous conversational context (e.g. `last_employee_id`, `last_department`).
3. **Template Population**: Fills deterministic templates with preserved entities.
4. **Deduplication & Limiting**: Removes duplicates and caps suggestions at 3–4 items.

---

## 4. Example Scenarios & Outputs

### Scenario 1: Missing Employee ID
* **User Input**: `"Find employee"`
* **Explanation**: `"I can help you find an employee. Please provide an employee ID or email address."`
* **Suggested Prompts**:
  1. `Find employee 1001`
  2. `Find employee using email john.smith@example.com`
  3. `Show all employees`

### Scenario 2: Unsupported Delete Operation
* **User Input**: `"Delete employee 1001"`
* **Explanation**: `"Employee deletion is not currently supported. However, I can help you view or update this employee."`
* **Suggested Prompts**:
  1. `Find employee 1001`
  2. `Show employee 1001 details`
  3. `Update employee 1001 status to Inactive`

### Scenario 3: Department Informal Query
* **User Input**: `"Show me IT workers"`
* **Explanation**: `"I found a request regarding the IT department. Here are suggested ways to query it:"`
* **Suggested Prompts**:
  1. `Show all employees in the IT department`
  2. `Find IT employees earning more than 150000`
  3. `Show employee names and designations in IT`
  4. `Count employees in IT`

### Scenario 4: Unrelated Query
* **User Input**: `"What's the weather today?"`
* **Explanation**: `"I'm currently designed to assist with employee management operations."`
* **Suggested Prompts**:
  1. `Find employees`
  2. `Search employees in IT`
  3. `View departments`
  4. `Count employees`
