// =============================================================================
// suggestions/suggestion_templates.rs — Deterministic prompt suggestion templates
// =============================================================================

/// Templates for missing employee ID
pub fn templates_missing_id() -> Vec<String> {
    vec![
        "Find employee 1001".to_string(),
        "Find employee using email john.smith@example.com".to_string(),
        "Show all employees".to_string(),
    ]
}

/// Templates for invalid employee ID
pub fn templates_invalid_id() -> Vec<String> {
    vec![
        "Find employee 1001".to_string(),
        "Find employee 1002".to_string(),
        "Show all employees".to_string(),
    ]
}

/// Templates for unsupported operations (e.g. delete employee <id>)
pub fn templates_unsupported_delete(entity_id: Option<&str>) -> Vec<String> {
    let id = entity_id.unwrap_or("1001");
    vec![
        format!("Find employee {}", id),
        format!("Show employee {} details", id),
        format!("Update employee {} status to Inactive", id),
    ]
}

/// Templates for incomplete update requests (e.g. update employee 1001)
pub fn templates_incomplete_update(entity_id: Option<&str>) -> Vec<String> {
    let id = entity_id.unwrap_or("1001");
    vec![
        format!("Update employee {} salary to 180000", id),
        format!("Update employee {} department to IT", id),
        format!("Update employee {} designation to Senior Developer", id),
        format!("Update employee {} status to Active", id),
    ]
}

/// Templates for department queries (e.g. "IT workers")
pub fn templates_department(dept: &str) -> Vec<String> {
    vec![
        format!("Show all employees in the {} department", dept),
        format!("Find {} employees earning more than 150000", dept),
        format!("Show employee names and designations in {}", dept),
        format!("Count employees in {}", dept),
    ]
}

/// Templates for unrelated prompts (weather, general banter)
pub fn templates_unrelated() -> Vec<String> {
    vec![
        "Find employees".to_string(),
        "Search employees in IT".to_string(),
        "View departments".to_string(),
        "Count employees".to_string(),
        "Add a new employee".to_string(),
        "Update employee 1001 salary to 120000".to_string(),
    ]
}

/// General fallback templates
pub fn templates_general_fallback() -> Vec<String> {
    vec![
        "Show all employees".to_string(),
        "Find employee 1001".to_string(),
        "Search employees in IT".to_string(),
        "Count active employees".to_string(),
        "Show all departments".to_string(),
    ]
}
