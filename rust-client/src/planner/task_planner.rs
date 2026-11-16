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

    // Step 4: Display field selection projection
    if !requested_fields.is_empty() {
        steps.push(ExecutionStep {
            step_number: step_num,
            description: format!(
                "Project requested display fields: [{}]",
                requested_fields.join(", ")
            ),
            tool_name: None,
            arguments: None,
        });
    }

    ExecutionPlan {
        intent_label: ir.intent.tool_name().unwrap_or("unknown").to_string(),
        steps,
        extracted_conditions: conditions,
        requested_fields,
        requires_confirmation: ir.intent.requires_confirmation(),
    }
}
