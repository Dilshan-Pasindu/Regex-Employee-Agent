// =============================================================================
// response/formatter.rs — Deterministic result formatting
// =============================================================================

use serde_json::Value;

/// Formats tool data into clean, readable text
pub fn format_mcp_data(data: &Value) -> String {
    match data {
        Value::Array(arr) => {
            if arr.is_empty() {
                return "No employees found matching the criteria.".to_string();
            }
            let mut lines = Vec::new();
            lines.push(format!(
                "Employees matching your search (total {}):",
                arr.len()
            ));
            for (idx, item) in arr.iter().enumerate() {
                lines.push(format!("{}. {}", idx + 1, format_single_employee(item)));
            }
            lines.join("\n")
        }
        Value::Object(map) => {
            // Check if it's a count result
            if let Some(c) = map.get("count") {
                return format!("Total count: {}", c);
            }
            format_single_employee(data)
        }
        _ => data.to_string(),
    }
}

pub fn format_single_employee(val: &Value) -> String {
    if let Some(map) = val.as_object() {
        let mut parts = Vec::new();
        if let (Some(f), Some(l)) = (map.get("firstName"), map.get("lastName")) {
            let id = map
                .get("employeeId")
                .map(|i| format!(" (#{})", i))
                .unwrap_or_default();
            parts.push(format!(
                "{} {}{}",
                f.as_str().unwrap_or(""),
                l.as_str().unwrap_or(""),
                id
            ));
        } else if let Some(id) = map.get("employeeId") {
            parts.push(format!("Employee #{}", id));
        }

        if let Some(dept) = map.get("departmentName") {
            parts.push(format!("Dept: {}", dept.as_str().unwrap_or("")));
        }
        if let Some(desig) = map.get("designationName") {
            parts.push(format!("Role: {}", desig.as_str().unwrap_or("")));
        }
        if let Some(sal) = map.get("salary") {
            parts.push(format!("Salary: {}", sal));
        }
        if let Some(email) = map.get("email") {
            parts.push(format!("Email: {}", email.as_str().unwrap_or("")));
        }
        if let Some(status) = map.get("status") {
            parts.push(format!("Status: {}", status.as_str().unwrap_or("")));
        }

        if parts.is_empty() {
            return val.to_string();
        }
        parts.join(" | ")
    } else {
        val.to_string()
    }
}
