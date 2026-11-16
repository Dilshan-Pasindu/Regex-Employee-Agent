# Multi-Step Agentic Task Planning System

## 1. Overview
The Multi-Step Agentic Task Planning System enables the Rust MCP Client to process complex queries that require multiple processing stages before and after invoking MCP tools. Rather than simply forwarding a prompt to a single tool, the system:
1. Deconstructs the user prompt into structured conditions.
2. Identifies primary search filters, numeric comparison constraints, and requested display fields.
3. Formulates an ordered multi-step `ExecutionPlan`.
4. Validates parameters and tool schemas prior to execution.
5. Invokes the underlying MCP tool over STDIO.
6. Executes post-filtering (e.g. numeric salary comparisons) and selective field projection in memory.
7. Formats the final response.

All planning logic is 100% deterministic, without LLMs.

---

## 2. Core Data Models

```rust
pub struct ExecutionPlan {
    pub intent_label: String,
    pub steps: Vec<ExecutionStep>,
    pub extracted_conditions: ExtractedConditions,
    pub requested_fields: Vec<String>,
    pub requires_confirmation: bool,
}

pub struct ExecutionStep {
    pub step_number: usize,
    pub description: String,
    pub tool_name: Option<String>,
    pub arguments: Option<serde_json::Value>,
}

pub struct ExtractedConditions {
    pub employee_id: Option<i64>,
    pub department: Option<String>,
    pub designation: Option<String>,
    pub email: Option<String>,
    pub status: Option<String>,
    pub salary: Option<f64>,
    pub salary_operator: Option<ComparisonOperator>,
}

pub enum ComparisonOperator {
    Equal,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}
```

---

## 3. Plan Generation Workflow

When given a complex prompt like:
> `"Find all employees in IT earning more than 150000 and show their names and salaries"`

The task planner generates the following 4-step execution plan:

1. **Step 1 (Entity & Condition Extraction)**:
   - Department: `IT`
   - Salary Condition: `> 150000.0` (`ComparisonOperator::GreaterThan`)
   - Requested Fields: `["firstName", "lastName", "salary"]`
2. **Step 2 (Primary MCP Invocation)**:
   - Tool: `search_employees`
   - Arguments: `{"departmentName": "IT"}`
3. **Step 3 (Client-Side Post-Filtering)**:
   - Filter returned array in-memory where `salary > 150000.0`.
4. **Step 4 (Field Selection Projection)**:
   - Project only the requested fields `employeeId`, `firstName`, `lastName`, and `salary`, removing non-requested properties.

---

## 4. Multi-Condition Query Support

The planner supports natural language conditions including:
- **Department**: `"in IT department"`, `"from HR"`, `"in Engineering"`
- **Salary Comparisons**:
  - `earning more than 150000` (`> 150000`)
  - `salary under 90000` (`< 90000`)
  - `salary equal to 125000` (`= 125000`)
- **Status Filters**: `Active`, `Inactive`, `On Leave`
- **Field Selection**:
  - `"names"` / `"full name"` → `firstName`, `lastName`
  - `"salary"` / `"pay"` → `salary`
  - `"department name"` / `"dept"` → `departmentName`
  - `"designation"` / `"role"` → `designationName`
  - `"email"` → `email`

---

## 5. Plan Validation & Safety Gates

Before any plan is executed:
- **Tool Existence**: Verified against `client.discovered_tools`.
- **Parameter Sanity**: Checks required fields (e.g. non-empty names/email for `AddEmployee`, non-negative salaries).
- **Confirmation Gate**: Write operations (`AddEmployee`, `UpdateEmployee`) require explicit user confirmation before dispatching tool calls.
