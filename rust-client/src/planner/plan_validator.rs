// =============================================================================
// planner/plan_validator.rs — Deterministic validation of execution plans
// =============================================================================

use crate::models::{Intent, IntentResult, McpTool};
use crate::planner::execution_plan::ExecutionPlan;

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

pub fn validate_plan(
    plan: &ExecutionPlan,
    ir: &IntentResult,
    discovered_tools: &[McpTool],
) -> ValidationResult {
    let mut errors = Vec::new();

    // 1. Check intent validity
    if let Intent::Unknown(ref u) = ir.intent {
        errors.push(format!("Cannot execute unknown intent: '{}'", u));
    }

    // 2. Validate tool availability if steps require tools
    for step in &plan.steps {
        if let Some(ref tool_name) = step.tool_name {
            let exists = discovered_tools.iter().any(|t| t.name == *tool_name);
            if !exists {
                errors.push(format!(
                    "Required MCP tool '{}' is not available in discovered catalogue.",
                    tool_name
                ));
            }
        }
    }

    // 3. Validate specific intent requirements
    match ir.intent {
        Intent::GetEmployeeById => {
            if plan.extracted_conditions.employee_id.is_none() {
                errors.push("Missing required employee ID for lookup.".to_string());
            }
        }
        Intent::AddEmployee => {
            if ir.params.first_name.is_none() || ir.params.last_name.is_none() {
                errors
                    .push("First name and last name are required to add an employee.".to_string());
            }
            if ir.params.email.is_none() {
                errors.push("Valid email address is required to add an employee.".to_string());
            }
            if let Some(s) = ir.params.salary {
                if s < 0.0 {
                    errors.push("Salary cannot be negative.".to_string());
                }
            }
        }
        Intent::UpdateEmployee => {
            if plan.extracted_conditions.employee_id.is_none() {
                errors.push("Employee ID is required for update operation.".to_string());
            }
            if ir.params.field_to_update.is_none() && ir.params.new_value.is_none() {
                errors.push("Missing field to update and new value.".to_string());
            }
        }
        _ => {}
    }

    ValidationResult {
        is_valid: errors.is_empty(),
        errors,
    }
}
