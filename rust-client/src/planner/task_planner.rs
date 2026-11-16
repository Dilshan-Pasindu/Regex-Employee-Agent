// =============================================================================
// planner/task_planner.rs — Multi-step task planner
// =============================================================================

use crate::models::{Intent, IntentResult};
use crate::planner::execution_plan::{
    ComparisonOperator, ExecutionPlan, ExecutionStep, ExtractedConditions,
};
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::json;

static RE_SALARY_COMPARISON: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:earning|salary|makes?|paid)?\s*(?:more\s+than|greater\s+than|above|over|>)\s*(?P<val>[0-9]+(?:\.[0-9]+)?)").unwrap()
});

static RE_SALARY_LESS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:earning|salary|makes?|paid)?\s*(?:less\s+than|under|below|<)\s*(?P<val>[0-9]+(?:\.[0-9]+)?)").unwrap()
});

static RE_SALARY_EQUAL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:earning|salary|makes?|paid)?\s*(?:equal\s+to|exactly|=)\s*(?P<val>[0-9]+(?:\.[0-9]+)?)").unwrap()
});

/// Extracts comparison operators and threshold values for salary
pub fn extract_salary_comparison(prompt: &str) -> Option<(ComparisonOperator, f64)> {
    if let Some(caps) = RE_SALARY_COMPARISON.captures(prompt) {
        if let Some(val_str) = caps.name("val") {
            if let Ok(v) = val_str.as_str().parse::<f64>() {
                return Some((ComparisonOperator::GreaterThan, v));
            }
        }
    }
    if let Some(caps) = RE_SALARY_LESS.captures(prompt) {
        if let Some(val_str) = caps.name("val") {
            if let Ok(v) = val_str.as_str().parse::<f64>() {
                return Some((ComparisonOperator::LessThan, v));
            }
        }
    }
    if let Some(caps) = RE_SALARY_EQUAL.captures(prompt) {
        if let Some(val_str) = caps.name("val") {
            if let Ok(v) = val_str.as_str().parse::<f64>() {
                return Some((ComparisonOperator::Equal, v));
            }
        }
    }
    None
}

/// Creates a structured execution plan from prompt, intent, and parameters.
pub fn create_execution_plan(prompt: &str, ir: &IntentResult) -> ExecutionPlan {
    let mut steps = Vec::new();
    let mut step_num = 1;

    let salary_comp = extract_salary_comparison(prompt);

    let conditions = ExtractedConditions {
        employee_id: ir.params.employee_id,
        department: ir.params.department_name.clone(),
        designation: ir.params.designation_name.clone(),
        email: ir.params.email.clone(),
        status: ir.params.status.clone(),
        salary: salary_comp.map(|c| c.1).or(ir.params.salary),
        salary_operator: salary_comp.map(|c| c.0),
    };

    let requested_fields = ir.params.requested_fields.clone();

    // Step 1: Intent & entity extraction
    steps.push(ExecutionStep {
        step_number: step_num,
        description: format!(
            "Recognized intent '{}' with extracted parameters",
            ir.intent.tool_name().unwrap_or("unknown")
        ),
        tool_name: None,
        arguments: None,
    });
    step_num += 1;

    // Step 2: Primary MCP Tool Invocation
    if let Some(tool_name) = ir.intent.tool_name() {
        let args = match ir.intent {
            Intent::GetAllEmployees => json!({}),
            Intent::GetEmployeeById => json!({ "employeeId": ir.params.employee_id }),
            Intent::GetDepartmentList => json!({}),
            Intent::GetDesignationList => json!({}),
            Intent::GetEmployeeCount => {
                let mut map = serde_json::Map::new();
                if let Some(ref d) = ir.params.department_name {
                    map.insert("department".to_string(), json!(d));
                }
                if let Some(ref s) = ir.params.status {
                    map.insert("status".to_string(), json!(s));
                }
                json!(map)
            }
            Intent::SearchEmployees => {
                let mut map = serde_json::Map::new();
                if let Some(ref d) = ir.params.department_name {
                    map.insert("departmentName".to_string(), json!(d));
                }
                if let Some(ref desig) = ir.params.designation_name {
                    map.insert("designationName".to_string(), json!(desig));
                }
                if let Some(ref s) = ir.params.status {
                    map.insert("status".to_string(), json!(s));
                }
                if let Some(ref f) = ir.params.first_name {
                    map.insert("firstName".to_string(), json!(f));
                }
                if let Some(ref l) = ir.params.last_name {
                    map.insert("lastName".to_string(), json!(l));
                }
                if let Some(ref e) = ir.params.email {
                    map.insert("email".to_string(), json!(e));
                }
                json!(map)
            }
            Intent::UpdateEmployee => {
                let mut map = serde_json::Map::new();
                if let Some(id) = ir.params.employee_id {
                    map.insert("employeeId".to_string(), json!(id));
                }
                if let Some(ref f) = ir.params.first_name {
                    map.insert("firstName".to_string(), json!(f));
                }
                if let Some(ref l) = ir.params.last_name {
                    map.insert("lastName".to_string(), json!(l));
                }
                if let Some(ref e) = ir.params.email {
                    map.insert("email".to_string(), json!(e));
                }
                if let Some(ref ph) = ir.params.phone {
                    map.insert("phone".to_string(), json!(ph));
                }
                if let Some(ref st) = ir.params.status {
                    map.insert("status".to_string(), json!(st));
                }
                if let Some(sal) = ir.params.salary {
                    map.insert("salary".to_string(), json!(sal));
                }
                if let Some(ref d) = ir.params.hire_date {
                    map.insert("hireDate".to_string(), json!(d));
                }
                // Handle department update (either ID directly or department name resolution)
                let dept_val = ir
                    .params
                    .department_name
                    .as_ref()
                    .or_else(|| {
                        if ir.params.field_to_update.as_deref() == Some("department") {
                            ir.params.new_value.as_ref()
                        } else {
                            None
                        }
                    });
                if let Some(d_val) = dept_val {
                    if let Ok(id) = d_val.parse::<i32>() {
                        map.insert("departmentId".to_string(), json!(id));
                    } else {
                        let dept_id = match d_val.to_lowercase().trim() {
                            "it" => Some(1),
                            "hr" => Some(2),
                            "finance" => Some(3),
                            "sales" => Some(4),
                            "engineering" => Some(5),
                            "operations" => Some(6),
                            _ => None,
                        };
                        if let Some(did) = dept_id {
                            map.insert("departmentId".to_string(), json!(did));
                        }
                    }
                }
                // Handle designation update
                let des_val = ir
                    .params
                    .designation_name
                    .as_ref()
                    .or_else(|| {
                        if ir.params.field_to_update.as_deref() == Some("designation") {
                            ir.params.new_value.as_ref()
                        } else {
                            None
                        }
                    });
                if let Some(des_str) = des_val {
                    if let Ok(id) = des_str.parse::<i32>() {
                        map.insert("designationId".to_string(), json!(id));
                    } else {
                        let desig_id = match des_str.to_lowercase().trim() {
                            "software engineer" => Some(1),
                            "senior software engineer" => Some(2),
                            "team lead" => Some(3),
                            "project manager" => Some(4),
                            "qa engineer" => Some(5),
                            "business analyst" => Some(6),
                            "hr executive" => Some(7),
                            "accountant" => Some(8),
                            "sales executive" => Some(9),
                            "devops engineer" => Some(10),
                            "ui/ux designer" | "designer" => Some(11),
                            "intern" => Some(12),
                            _ => None,
                        };
                        if let Some(gid) = desig_id {
                            map.insert("designationId".to_string(), json!(gid));
                        }
                    }
                }
                json!(map)
            }
            Intent::AddEmployee => {
                let mut map = serde_json::Map::new();
                if let Some(ref f) = ir.params.first_name {
                    map.insert("firstName".to_string(), json!(f));
                }
                if let Some(ref l) = ir.params.last_name {
                    map.insert("lastName".to_string(), json!(l));
                }
                if let Some(ref e) = ir.params.email {
                    map.insert("email".to_string(), json!(e));
                }
                if let Some(ref ph) = ir.params.phone {
                    map.insert("phone".to_string(), json!(ph));
                }
                if let Some(sal) = ir.params.salary {
                    map.insert("salary".to_string(), json!(sal));
                } else {
                    map.insert("salary".to_string(), json!(0.0));
                }
                map.insert(
                    "hireDate".to_string(),
                    json!(ir.params.hire_date.as_deref().unwrap_or("2024-01-01")),
                );
                map.insert(
                    "status".to_string(),
                    json!(ir.params.status.as_deref().unwrap_or("Active")),
                );

                // Department ID resolution
                let dept_id = if let Some(ref d_val) = ir.params.department_name {
                    if let Ok(id) = d_val.parse::<i32>() {
                        id
                    } else {
                        match d_val.to_lowercase().trim() {
                            "it" => 1,
                            "hr" => 2,
                            "finance" => 3,
                            "sales" => 4,
                            "engineering" => 5,
                            "operations" => 6,
                            _ => 1,
                        }
                    }
                } else {
                    1
                };
                map.insert("departmentId".to_string(), json!(dept_id));

                // Designation ID resolution
                let desig_id = if let Some(ref des_val) = ir.params.designation_name {
                    if let Ok(id) = des_val.parse::<i32>() {
                        id
                    } else {
                        match des_val.to_lowercase().trim() {
                            "software engineer" => 1,
                            "senior software engineer" => 2,
                            "team lead" => 3,
                            "project manager" => 4,
                            "qa engineer" => 5,
                            "business analyst" => 6,
                            "hr executive" => 7,
                            "accountant" => 8,
                            "sales executive" => 9,
                            "devops engineer" => 10,
                            "ui/ux designer" | "designer" => 11,
                            "intern" => 12,
                            _ => 1,
                        }
                    }
                } else {
                    1
                };
                map.insert("designationId".to_string(), json!(desig_id));

                json!(map)
            }
            _ => json!({}),
        };

        steps.push(ExecutionStep {
            step_number: step_num,
            description: format!("Invoke MCP tool '{}' with validated arguments", tool_name),
            tool_name: Some(tool_name.to_string()),
            arguments: Some(args),
        });
        step_num += 1;
    }

    // Step 3: Post-filter by comparison condition if tool doesn't support numeric range natively
    if let (Some(op), Some(sal)) = (conditions.salary_operator, conditions.salary) {
        steps.push(ExecutionStep {
            step_number: step_num,
            description: format!("Apply post-filtering: salary {} {}", op.symbol(), sal),
            tool_name: None,
            arguments: None,
        });
        step_num += 1;
    }

    // Step 4: Display field selection projection (only for read operations, not write/updates)
    let is_update = ir.intent == Intent::UpdateEmployee;
    let effective_requested_fields = if is_update {
        Vec::new()
    } else {
        requested_fields
    };

    if !effective_requested_fields.is_empty() {
        steps.push(ExecutionStep {
            step_number: step_num,
            description: format!(
                "Project requested display fields: [{}]",
                effective_requested_fields.join(", ")
            ),
            tool_name: None,
            arguments: None,
        });
    }

    ExecutionPlan {
        intent_label: ir.intent.tool_name().unwrap_or("unknown").to_string(),
        steps,
        extracted_conditions: conditions,
        requested_fields: effective_requested_fields,
        requires_confirmation: ir.intent.requires_confirmation(),
    }
}
