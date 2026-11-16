# MCP Tool Catalogue

All 8 tools exposed by the `.NET EmployeeMcpServer` via JSON-RPC 2.0 over STDIO.

---

## `get_all_employees`

Returns all employees in the database.

**Input:** None

**Example prompt:** `show all employees` / `list everyone` / `get all employees`

**Output:**
```json
{
  "success": true,
  "data": [
    { "employeeId": 1001, "firstName": "John", "lastName": "Smith",
      "email": "john.smith@spillabs.com", "departmentName": "IT",
      "designationName": "Senior Engineer", "salary": 85000.00,
      "hireDate": "2019-03-15", "status": "Active" },
    ...
  ]
}
```

---

## `get_employee_by_id`

Looks up a single employee by their numeric ID.

**Input:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `employeeId` | integer | ✅ | Employee ID (1001–1020) |

**Example prompts:**
- `find employee 1001`
- `show employee ID 1005`
- `get employee 1010`

---

## `search_employees`

Filters employees by one or more criteria.

**Input:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `departmentName` | string | ❌ | Filter by department name |
| `designationName` | string | ❌ | Filter by designation |
| `firstName` | string | ❌ | Filter by first name |
| `lastName` | string | ❌ | Filter by last name |
| `status` | string | ❌ | `Active`, `Inactive`, or `On Leave` |

**Example prompts:**
- `search employees in IT`
- `find employees named John`
- `show active employees`
- `find employees in Engineering department`

---

## `get_department_list`

Returns all departments.

**Input:** None

**Example prompts:** `show all departments` / `list departments` / `what departments exist`

**Output:**
```json
{
  "success": true,
  "data": [
    { "departmentId": 1, "departmentName": "IT", "employeeCount": 5 },
    ...
  ]
}
```

---

## `get_designation_list`

Returns all designations.

**Input:** None

**Example prompts:** `show designations` / `list all designations`

---

## `get_employee_count`

Returns total employee count, optionally filtered.

**Input:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `departmentName` | string | ❌ | Count employees in a specific department |
| `status` | string | ❌ | Count by status |

**Example prompts:**
- `how many employees are there?`
- `count employees in HR`
- `how many active employees?`

---

## `add_employee`

Inserts a new employee. **Requires write confirmation.**

**Input:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `firstName` | string | ✅ | First name |
| `lastName` | string | ✅ | Last name |
| `email` | string | ✅ | Email address (must be unique) |
| `departmentName` | string | ✅ | Must match an existing department |
| `designationName` | string | ✅ | Must match an existing designation |
| `salary` | number | ❌ | Annual salary |
| `hireDate` | string | ❌ | ISO date (YYYY-MM-DD) |
| `phone` | string | ❌ | Phone number |
| `status` | string | ❌ | Default: `Active` |

**Example prompt:**
```
Add employee Jane Doe email j.doe@spillabs.com department IT designation Engineer salary 75000
```

---

## `update_employee`

Updates a single field on an existing employee. **Requires write confirmation.**

**Input:**
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `employeeId` | integer | ✅ | Employee to update |
| `fieldName` | string | ✅ | Field to change |
| `newValue` | string | ✅ | New value |

**Supported fields:** `status`, `salary`, `departmentName`, `designationName`, `email`, `phone`, `firstName`, `lastName`

**Example prompts:**
- `update employee 1001 status to Inactive`
- `update employee 1005 salary to 95000`
- `update employee 1010 department to Engineering`
