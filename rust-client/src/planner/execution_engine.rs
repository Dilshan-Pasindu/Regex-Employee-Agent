// =============================================================================
// planner/execution_engine.rs — Executes validated plans via MCP over STDIO
// =============================================================================

use crate::errors::Result;
use crate::mcp::client::McpClient;
use crate::planner::execution_plan::ExecutionPlan;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ExecutionOutcome {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<Value>,
    pub matching_count: usize,
    pub execution_log: Vec<String>,
}

/// Executes the plan: calls primary MCP tool, applies post-filters & projections.
pub async fn execute_plan(
    plan: &ExecutionPlan,
    client: &mut McpClient,
) -> Result<ExecutionOutcome> {
    let mut log = Vec::new();
    let mut current_data: Option<Value> = None;
    let mut outcome_msg: Option<String> = None;

    for step in &plan.steps {
        log.push(format!("Step {}: {}", step.step_number, step.description));

        // If step calls an MCP tool
        if let (Some(tool_name), Some(args)) = (&step.tool_name, &step.arguments) {
            let res = client.call_tool(tool_name, args.clone()).await?;
            if !res.success {
                return Ok(ExecutionOutcome {
                    success: false,
                    message: res.message,
                    data: None,
                    matching_count: 0,
                    execution_log: log,
                });
            }
            current_data = res.data;
            outcome_msg = res.message;
        }
    }

    // Apply post-filtering for salary comparison if present
    if let (Some(op), Some(target_sal)) = (
        plan.extracted_conditions.salary_operator,
        plan.extracted_conditions.salary,
    ) {
        if let Some(ref data) = current_data {
            if let Some(arr) = data.as_array() {
                let filtered: Vec<Value> = arr
                    .iter()
                    .filter(|item| {
                        if let Some(sal) = item.get("salary").and_then(|s| s.as_f64()) {
                            op.matches(sal, target_sal)
                        } else {
                            false
                        }
                    })
                    .cloned()
                    .collect();

                let count = filtered.len();
                log.push(format!(
                    "Post-filtered {} employees matching salary {} {}",
                    count,
                    op.symbol(),
                    target_sal
                ));
                outcome_msg = Some(format!(
                    "Found {} employee(s) matching salary criteria.",
                    count
                ));
                current_data = Some(Value::Array(filtered));
            }
        }
    }

    // Apply field selection projection if requested
    if !plan.requested_fields.is_empty() {
        if let Some(ref data) = current_data {
            current_data = Some(project_fields(data, &plan.requested_fields));
            log.push(format!(
                "Projected fields: [{}]",
                plan.requested_fields.join(", ")
            ));
        }
    }

    let count = match current_data {
        Some(Value::Array(ref arr)) => arr.len(),
        Some(Value::Object(_)) => 1,
        _ => 0,
    };

    Ok(ExecutionOutcome {
        success: true,
        message: outcome_msg,
        data: current_data,
        matching_count: count,
        execution_log: log,
    })
}

/// Projects only the requested fields while preserving order
pub fn project_fields(data: &Value, requested_fields: &[String]) -> Value {
    match data {
        Value::Array(arr) => {
            let projected: Vec<Value> = arr
                .iter()
                .map(|item| project_object_fields(item, requested_fields))
                .collect();
            Value::Array(projected)
        }
        Value::Object(_) => project_object_fields(data, requested_fields),
        _ => data.clone(),
    }
}

fn project_object_fields(obj: &Value, requested_fields: &[String]) -> Value {
    if let Some(map) = obj.as_object() {
        let mut new_map = serde_json::Map::new();

        // Always include employeeId for identity if present
        if let Some(id) = map.get("employeeId") {
            new_map.insert("employeeId".to_string(), id.clone());
        }

        for field in requested_fields {
            // Find key case-insensitively
            for (k, v) in map {
                if k.eq_ignore_ascii_case(field) {
                    new_map.insert(k.clone(), v.clone());
                    break;
                }
            }
        }
        Value::Object(new_map)
    } else {
        obj.clone()
    }
}
