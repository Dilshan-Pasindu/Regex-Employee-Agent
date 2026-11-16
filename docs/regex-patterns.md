# Regex Patterns Reference

All intent patterns are defined in `rust-client/src/intent/patterns.rs` using `once_cell::Lazy<Regex>`.

> **Zero LLM/AI policy:** Every pattern is a hand-crafted, deterministic Rust regex. No machine learning is involved anywhere in intent detection.

---

## Pattern Groups

### `get_all_employees` — RE_GET_ALL_EMPLOYEES

| Pattern | Matches |
|---------|---------|
| `(?i)^(show\|list\|display\|get\|fetch\|view)\s+(all\s+)?employees?(?:\s+(?:details?\|info(?:rmation)?\|records?\|data\|list))?$` | "show all employees", "show all employees details", "list employees details" |
| `(?i)^all\s+employees?(?:\s+(?:details?\|info(?:rmation)?\|records?\|data\|list))?$` | "all employees", "all employee details" |
| `(?i)^employees?\s+(list\|records?\|details?\|data\|info(?:rmation)?)$` | "employee details", "employees list", "employees records" |
| `(?i)^(give\s+me\s+\|show\s+me\s+)?all\s+employees?(?:\s+(?:details?\|info(?:rmation)?\|records?\|data\|list))?$` | "show me all employees details" |

**Does not match:** `show employee 1001` (ID present → `get_employee_by_id`)

---

### `get_employee_by_id` — RE_GET_BY_ID

| Pattern | Matches |
|---------|---------|
| `(?i)\b(find\|show\|get\|fetch\|display\|view\|lookup)\s+employee\s+(id\s+)?(\d{4,})\b` | "find employee 1001", "show employee ID 1005" |
| `(?i)\bemployee\s+(id\|number\|#)?\s*(\d{4,})\b` | "employee 1010", "employee #1001" |

**ID extraction:** Named capture `(?P<id>\d{4,})`

---

### `search_employees` — RE_SEARCH_EMPLOYEES

| Pattern | Matches |
|---------|---------|
| `(?i)\b(search\|filter\|find)\s+employees?\s+(in\|from\|by\|with\|named?\|called\|having)` | "search employees in IT" |
| `(?i)\bemployees?\s+(in\|from\|of)\s+(?:the\s+)?\w+\s*(department)?\b` | "employees in Engineering" |
| `(?i)\b(active\|inactive\|on\s+leave)\s+employees?\b` | "active employees" |
| `(?i)\bemployees?\s+named?\s+\w+` | "employees named John" |

**Parameter extraction:**
- Department: `(?P<dept>[A-Za-z][A-Za-z\s]+?)(?:\s+department|\s+dept|$)`
- Status: direct keyword match for `active`, `inactive`, `on leave`
- Name: `named?\s+(?P<fname>[A-Z][a-z]+)`

---

### `get_employee_count` — RE_GET_COUNT

| Pattern | Matches |
|---------|---------|
| `(?i)\b(how\s+many\|count\|total(\s+number\s+of)?)\s+employees?\b` | "how many employees", "count employees" |
| `(?i)\bemployee\s+(count\|total\|number)\b` | "employee count" |

---

### `get_department_list` — RE_GET_DEPARTMENTS

| Pattern | Matches |
|---------|---------|
| `(?i)\b(show\|list\|get\|display\|fetch)\s+(all\s+)?departments?\b` | "show all departments" |
| `(?i)\bwhat\s+departments?\s+(are\s+there\|exist\|do\s+you\s+have)` | "what departments are there" |

---

### `add_employee` — RE_ADD_EMPLOYEE

| Pattern | Matches |
|---------|---------|
| `(?i)\b(add\|create\|insert\|new\|hire\|onboard)\s+(an?\s+)?employee\b` | "add employee", "create an employee" |
| `(?i)\b(register\|enroll)\s+(an?\s+)?employee\b` | "register an employee" |

**Extraction anchors (prevent false positives):**
- First name: `named?\s+(?P<fname>[A-Z][a-z]+)` or `employee\s+(?P<fname>[A-Z][a-z]+)\s+(?P<lname>[A-Z][a-z]+)`
- Email: `(?i)\bemail\s+(?P<email>[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,})`
- Department: `(?i)\b(dept|department)\s+(?P<dept>[A-Za-z][A-Za-z\s]+?)(?:\s|$)`
- Designation: `(?i)\b(designation|role|title|position)\s+(?P<desig>[A-Za-z][A-Za-z\s]+?)(?:\s|$)`
- Salary: `(?i)\bsalary\s+(?P<salary>\d+(?:\.\d+)?)`

---

### `update_employee` — RE_UPDATE_EMPLOYEE ⚠️ Priority 1

| Pattern | Matches |
|---------|---------|
| `(?i)\b(update\|change\|modify\|set\|edit\|alter)\s+employee\s+\d{4,}` | "update employee 1001 status to Active" |
| `(?i)\b(update\|change\|modify\|set\|edit\|alter)\s+employee\s+id\s+\d{4,}` | "update employee ID 1005 salary to 90000" |

> **Critical:** This pattern is evaluated **before** `get_employee_by_id` to prevent `update employee 1001 department to IT` from being misclassified as a lookup (both contain a 4-digit ID).

**Field extraction:**
- `(?i)\b(?P<field>status|salary|department(?:name)?|designation(?:name)?|email|phone|firstname|lastname)\s+to\s+(?P<value>.+?)(?:\s*$)`

---

## Status Normalisation

| Input | Normalised |
|-------|-----------|
| `active`, `enabled`, `working` | `Active` |
| `inactive`, `disabled`, `terminated` | `Inactive` |
| `leave`, `on leave`, `absent` | `On Leave` |

---

## Known Limitations

1. **Multi-word first names** (e.g. "Mary Jane") — only single first name is extracted.
2. **Ambiguous department names** — "New" or "Find" cannot be a department name (anchor prevents it).
3. **Date extraction** — `hireDate` extraction is regex-only: `YYYY-MM-DD` or `MM/DD/YYYY` formats.
4. **Non-English input** — All patterns are ASCII/Latin only.
5. **Compound queries** — `show all employees AND count them` is not supported; each prompt handles one intent.
