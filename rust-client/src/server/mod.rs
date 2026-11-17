// =============================================================================
// server/mod.rs — Axum HTTP server layer (Phase 3)
//
// Endpoints:
//   GET  /health        → {"rust":"ok","dotnet":"ok","tools_count":N}
//   GET  /tools         → list of discovered MCP tool schemas
//   POST /process       → {"prompt":str,"confirmed":bool}
//                      ← {"intent","tool","params","steps","result","success",
//                          "requires_confirmation"}
//
// The server wraps the full Rust intent engine + MCP client and exposes it
// over HTTP so the FastAPI backend can call it without caring about STDIO/MCP.
// =============================================================================

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    context::{
        ResolvedContextAction, clear_session_context, get_or_create_context, resolve_context,
        update_context,
    },
    intent::detector::detect_intent,
    mcp::client::McpClient,
    models::{Intent, IntentResult},
    planner::{create_execution_plan, execute_plan, project_fields, validate_plan},
    suggestions::generate_suggestions,
};

// ---------------------------------------------------------------------------
// Shared state — a single McpClient protected by a Mutex
// ---------------------------------------------------------------------------
pub struct AppState {
    pub client: Mutex<McpClient>,
}

// ---------------------------------------------------------------------------
// Request / Response shapes
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ProcessRequest {
    pub prompt: String,
    #[serde(default)]
    pub confirmed: bool,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Step {
    pub step: u32,
    pub label: String,
    pub value: Value,
}

#[derive(Debug, Serialize)]
pub struct ProcessResponse {
    pub prompt: String,
    pub intent: String,
    pub tool: Option<String>,
    pub params: Value,
    pub steps: Vec<Step>,
    pub result: Option<Value>,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_confirmation: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_plan: Option<Value>,
}

// ---------------------------------------------------------------------------
// Router builder — called from main.rs when --server flag is present
// ---------------------------------------------------------------------------
pub fn build_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health_handler))
        .route("/tools", get(tools_handler))
        .route("/process", post(process_handler))
        .with_state(state)
        .layer(cors)
}

// ---------------------------------------------------------------------------
// GET /health
// ---------------------------------------------------------------------------
async fn health_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let client = state.client.lock().await;
    let tools_count = client.discovered_tools.len();
    let dotnet_status = if tools_count > 0 { "ok" } else { "degraded" };

    Json(json!({
        "rust": "ok",
        "dotnet": dotnet_status,
        "tools_count": tools_count,
    }))
}

// ---------------------------------------------------------------------------
// GET /tools
// ---------------------------------------------------------------------------
async fn tools_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let client = state.client.lock().await;
    let tools: Vec<Value> = client
        .discovered_tools
        .iter()
        .map(|t| {
            json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": t.input_schema,
            })
        })
        .collect();

    Json(json!({ "tools": tools, "count": tools.len() }))
}

// ---------------------------------------------------------------------------
// POST /process — core agentic endpoint
// ---------------------------------------------------------------------------
async fn process_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ProcessRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    let raw_prompt = req.prompt.trim().to_string();
    if raw_prompt.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "prompt cannot be empty"})),
        ));
    }

    let session_id = req.session_id.unwrap_or_else(|| "default".to_string());
    let mut context = get_or_create_context(&session_id);

    // -----------------------------------------------------------------------
    // Stage 1: Conversational Context Resolution
    // -----------------------------------------------------------------------
    let context_action = resolve_context(&raw_prompt, &context);
    let prompt = match context_action {
        ResolvedContextAction::ContextCleared => {
            clear_session_context(&session_id);
            return Ok(Json(ProcessResponse {
                prompt: raw_prompt,
                intent: "clear_context".to_string(),
                tool: None,
                params: json!({}),
                steps: vec![Step {
                    step: 1,
                    label: "Context Cleared".into(),
                    value: json!("Conversation memory reset"),
                }],
                result: None,
                success: true,
                requires_confirmation: None,
                message: Some("Conversation context has been cleared.".to_string()),
                suggestions: None,
                execution_plan: None,
            }));
        }
        ResolvedContextAction::InMemoryProjection { requested_fields } => {
            if let Some(ref data) = context.last_result_data {
                let projected = project_fields(data, &requested_fields);
                context.last_requested_fields = requested_fields.clone();
                update_context(&session_id, context);

                return Ok(Json(ProcessResponse {
                    prompt: raw_prompt,
                    intent: "filter_previous_result".to_string(),
                    tool: None,
                    params: json!({ "requestedFields": requested_fields }),
                    steps: vec![Step {
                        step: 1,
                        label: "In-Memory Projection".into(),
                        value: json!(requested_fields),
                    }],
                    result: Some(projected),
                    success: true,
                    requires_confirmation: None,
                    message: Some(format!(
                        "Filtered previous search results to {}.",
                        requested_fields.join(", ")
                    )),
                    suggestions: None,
                    execution_plan: None,
                }));
            }
            raw_prompt
        }
        ResolvedContextAction::EmployeeFollowUp {
            rewritten_prompt, ..
        } => rewritten_prompt,
        ResolvedContextAction::DepartmentSwitch {
            rewritten_prompt, ..
        } => rewritten_prompt,
        ResolvedContextAction::None => raw_prompt,
    };

    // -----------------------------------------------------------------------
    // Stage 2: Intent Detection & Parameter Extraction
    // -----------------------------------------------------------------------
    let intent_result: IntentResult = detect_intent(&prompt);
    let intent_name = intent_label(&intent_result.intent);

    let mut steps: Vec<Step> = Vec::new();
    steps.push(Step {
        step: 1,
        label: "Intent Recognized".to_string(),
        value: json!(intent_name),
    });

    let params_json = params_to_json(&intent_result);
    steps.push(Step {
        step: 2,
        label: "Parameters Extracted".to_string(),
        value: params_json.clone(),
    });

    // -----------------------------------------------------------------------
    // Stage 3: Prompt Suggestion & Recovery for Invalid/Unknown Prompts
    // -----------------------------------------------------------------------
    if matches!(intent_result.intent, Intent::Unknown(_)) {
        let suggestion_out = generate_suggestions(&prompt, &intent_result, &context);
        context.record_failure();
        update_context(&session_id, context);

        return Ok(Json(ProcessResponse {
            prompt,
            intent: intent_name,
            tool: None,
            params: params_json,
            steps,
            result: None,
            success: false,
            requires_confirmation: None,
            message: Some(suggestion_out.explanation),
            suggestions: Some(suggestion_out.suggestions),
            execution_plan: None,
        }));
    }

    // -----------------------------------------------------------------------
    // Stage 4: Multi-Step Task Planning & Validation
    // -----------------------------------------------------------------------
    let plan = create_execution_plan(&prompt, &intent_result);

    let mut client = state.client.lock().await;
    let validation = validate_plan(&plan, &intent_result, &client.discovered_tools);
    if !validation.is_valid {
        let suggestion_out = generate_suggestions(&prompt, &intent_result, &context);
        context.record_failure();
        update_context(&session_id, context);

        return Ok(Json(ProcessResponse {
            prompt,
            intent: intent_name,
            tool: None,
            params: params_json,
            steps,
            result: None,
            success: false,
            requires_confirmation: None,
            message: Some(if !suggestion_out.explanation.is_empty() {
                suggestion_out.explanation
            } else {
                validation.errors.join("; ")
            }),
            suggestions: Some(suggestion_out.suggestions),
            execution_plan: Some(json!(plan)),
        }));
    }

    // Write operations — confirmation gate
    if plan.requires_confirmation && !req.confirmed {
        steps.push(Step {
            step: 3,
            label: "Awaiting Confirmation".to_string(),
            value: json!("Write operation requires explicit confirmation before execution."),
        });

        // Special handling for Undo confirmation preview
        if let Intent::UndoLastUpdate = intent_result.intent {
            if let Some(ref undo) = context.last_undo {
                return Ok(Json(ProcessResponse {
                    prompt,
                    intent: intent_name,
                    tool: Some("update_employee".to_string()),
                    params: params_json,
                    steps,
                    result: Some(json!({
                        "employeeId": undo.employee_id,
                        "field": undo.field_to_update,
                        "revertingTo": undo.previous_value,
                    })),
                    success: false,
                    requires_confirmation: Some(true),
                    message: Some(format!(
                        "Revert {} for employee {} back to {}?",
                        undo.field_to_update, undo.employee_id, undo.previous_value
                    )),
                    suggestions: None,
                    execution_plan: Some(json!(plan)),
                }));
            } else {
                return Ok(Json(ProcessResponse {
                    prompt,
                    intent: intent_name,
                    tool: None,
                    params: params_json,
                    steps,
                    result: None,
                    success: false,
                    requires_confirmation: None,
                    message: Some(
                        "Nothing to undo: no previous update operations found in this session."
                            .to_string(),
                    ),
                    suggestions: None,
                    execution_plan: None,
                }));
            }
        }

        // Special handling for Batch preview before confirmation
        if let Intent::BatchUpdate = intent_result.intent {
            let search_args = if let Some(ref d) = plan.extracted_conditions.designation {
                json!({ "designationName": d })
            } else if let Some(ref dept) = plan.extracted_conditions.department {
                json!({ "departmentName": dept })
            } else {
                json!({})
            };
            let res = client.call_tool("search_employees", search_args).await;
            let count = match res {
                Ok(ref r) => r
                    .data
                    .as_ref()
                    .and_then(|d| d.as_array())
                    .map(|a| a.len())
                    .unwrap_or(0),
                _ => 0,
            };
            let preview_data = res.ok().and_then(|r| r.data);
            return Ok(Json(ProcessResponse {
                prompt,
                intent: intent_name,
                tool: Some("search_employees".to_string()),
                params: params_json,
                steps,
                result: preview_data,
                success: true,
                requires_confirmation: Some(true),
                message: Some(format!(
                    "Found {} matching employee(s) for batch operation. Please confirm to apply changes.",
                    count
                )),
                suggestions: None,
                execution_plan: Some(json!(plan)),
            }));
        }

        return Ok(Json(ProcessResponse {
            prompt,
            intent: intent_name,
            tool: intent_result.intent.tool_name().map(|s| s.to_string()),
            params: params_json,
            steps,
            result: None,
            success: false,
            requires_confirmation: Some(true),
            message: Some("Please confirm this write operation.".to_string()),
            suggestions: None,
            execution_plan: Some(json!(plan)),
        }));
    }

    // Capture undo snapshot before executing update_employee
    if let Intent::UpdateEmployee = intent_result.intent {
        if let Some(emp_id) = plan.extracted_conditions.employee_id {
            if let Ok(lookup_res) = client
                .call_tool("get_employee_by_id", json!({ "employeeId": emp_id }))
                .await
            {
                if let Some(emp_obj) = lookup_res.data {
                    let field = intent_result
                        .params
                        .field_to_update
                        .clone()
                        .unwrap_or_else(|| "salary".to_string());
                    let prev_val = match field.as_str() {
                        "salary" => emp_obj.get("salary").cloned().unwrap_or(json!(0.0)),
                        "status" => emp_obj.get("status").cloned().unwrap_or(json!("Active")),
                        "department" => emp_obj.get("departmentId").cloned().unwrap_or(json!(1)),
                        _ => json!(null),
                    };
                    context.last_undo = Some(crate::context::conversation_context::UndoRecord {
                        employee_id: emp_id,
                        field_to_update: field,
                        previous_value: prev_val,
                        new_value: json!(intent_result.params.new_value),
                    });
                }
            }
        }
    }

    // Execute Undo operation when confirmed
    if let Intent::UndoLastUpdate = intent_result.intent {
        if let Some(undo) = context.last_undo.take() {
            let mut undo_args = serde_json::Map::new();
            undo_args.insert("employeeId".to_string(), json!(undo.employee_id));
            if undo.field_to_update == "salary" {
                let sal_val = undo
                    .previous_value
                    .as_f64()
                    .or_else(|| undo.previous_value.as_i64().map(|i| i as f64))
                    .or_else(|| {
                        undo.previous_value
                            .as_str()
                            .and_then(|s| s.parse::<f64>().ok())
                    });
                if let Some(sal) = sal_val {
                    undo_args.insert("salary".to_string(), json!(sal));
                }
            } else if undo.field_to_update == "status" {
                if let Some(st) = undo.previous_value.as_str() {
                    undo_args.insert("status".to_string(), json!(st));
                }
            } else if undo.field_to_update == "department" {
                let did_val = undo.previous_value.as_i64().or_else(|| {
                    undo.previous_value
                        .as_str()
                        .and_then(|s| s.parse::<i64>().ok())
                });
                if let Some(did) = did_val {
                    undo_args.insert("departmentId".to_string(), json!(did));
                }
            }
            let res = client.call_tool("update_employee", json!(undo_args)).await;
            update_context(&session_id, context);
            return match res {
                Ok(r) => Ok(Json(ProcessResponse {
                    prompt,
                    intent: intent_name,
                    tool: Some("update_employee".to_string()),
                    params: params_json,
                    steps,
                    result: r.data,
                    success: r.success,
                    requires_confirmation: None,
                    message: Some(format!(
                        "Successfully reverted {} for employee {} back to {}.",
                        undo.field_to_update, undo.employee_id, undo.previous_value
                    )),
                    suggestions: None,
                    execution_plan: Some(json!(plan)),
                })),
                Err(e) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": e.to_string() })),
                )),
            };
        }
    }

    // Execute Batch operation when confirmed
    if let Intent::BatchUpdate = intent_result.intent {
        let search_args = if let Some(ref d) = plan.extracted_conditions.designation {
            json!({ "designationName": d })
        } else {
            json!({})
        };
        if let Ok(lookup_res) = client.call_tool("search_employees", search_args).await {
            if let Some(arr) = lookup_res.data.and_then(|d| d.as_array().cloned()) {
                let mut updated_count = 0;
                for emp in &arr {
                    if let Some(id) = emp.get("employeeId").and_then(|v| v.as_i64()) {
                        if let Some(delta) = plan.extracted_conditions.batch_delta {
                            let cur_sal = emp.get("salary").and_then(|v| v.as_f64()).unwrap_or(0.0);
                            let new_sal = cur_sal + delta;
                            let _ = client
                                .call_tool(
                                    "update_employee",
                                    json!({
                                        "employeeId": id,
                                        "salary": new_sal
                                    }),
                                )
                                .await;
                            updated_count += 1;
                        }
                    }
                }
                update_context(&session_id, context);
                return Ok(Json(ProcessResponse {
                    prompt,
                    intent: intent_name,
                    tool: Some("update_employee".to_string()),
                    params: params_json,
                    steps,
                    result: Some(json!({ "batchUpdatedCount": updated_count })),
                    success: true,
                    requires_confirmation: None,
                    message: Some(format!(
                        "Successfully applied batch update to {} employee(s).",
                        updated_count
                    )),
                    suggestions: None,
                    execution_plan: Some(json!(plan)),
                }));
            }
        }
    }

    // -----------------------------------------------------------------------
    // Stage 5: Execution Engine (MCP over STDIO + Post-Filtering + Projection)
    // -----------------------------------------------------------------------
    for p_step in &plan.steps {
        steps.push(Step {
            step: (steps.len() + 1) as u32,
            label: format!("Execution Step {}", p_step.step_number),
            value: json!(p_step.description),
        });
    }

    match execute_plan(&plan, &mut client).await {
        Ok(outcome) => {
            steps.push(Step {
                step: (steps.len() + 1) as u32,
                label: "Result Received".to_string(),
                value: json!(if outcome.success { "200 OK" } else { "Error" }),
            });

            if outcome.success {
                // Update conversational memory
                match intent_result.intent {
                    Intent::GetEmployeeById => {
                        let emp_id = plan.extracted_conditions.employee_id.unwrap_or(0);
                        let name = outcome.data.as_ref().and_then(|d| {
                            if let (Some(f), Some(l)) = (
                                d.get("firstName").and_then(|v| v.as_str()),
                                d.get("lastName").and_then(|v| v.as_str()),
                            ) {
                                Some(format!("{} {}", f, l))
                            } else {
                                None
                            }
                        });
                        let email = outcome
                            .data
                            .as_ref()
                            .and_then(|d| d.get("email").and_then(|v| v.as_str()))
                            .map(|s| s.to_string());
                        let dept = outcome
                            .data
                            .as_ref()
                            .and_then(|d| d.get("departmentName").and_then(|v| v.as_str()))
                            .map(|s| s.to_string());
                        let desig = outcome
                            .data
                            .as_ref()
                            .and_then(|d| d.get("designationName").and_then(|v| v.as_str()))
                            .map(|s| s.to_string());
                        let salary = outcome
                            .data
                            .as_ref()
                            .and_then(|d| d.get("salary").and_then(|v| v.as_f64()));
                        let status = outcome
                            .data
                            .as_ref()
                            .and_then(|d| d.get("status").and_then(|v| v.as_str()))
                            .map(|s| s.to_string());

                        context.record_employee_lookup(
                            emp_id,
                            name,
                            email,
                            dept,
                            desig,
                            salary,
                            status,
                            outcome.data.clone(),
                        );
                    }
                    Intent::SearchEmployees | Intent::GetAllEmployees => {
                        context.record_search(
                            plan.extracted_conditions.department.clone(),
                            plan.extracted_conditions.designation.clone(),
                            plan.extracted_conditions.status.clone(),
                            outcome.data.clone(),
                            outcome.matching_count,
                        );
                    }
                    _ => {}
                }
                update_context(&session_id, context);
            } else {
                context.record_failure();
                update_context(&session_id, context);
            }

            // Multi-result disambiguation suggestions (Feature 2)
            let disambiguation_suggestions: Option<Vec<String>> = if outcome.success {
                if let Some(ref d) = outcome.data {
                    if let Some(arr) = d.as_array() {
                        if arr.len() > 1 && arr.len() <= 6 {
                            let chips: Vec<String> = arr
                                .iter()
                                .filter_map(|item| {
                                    let id = item.get("employeeId").and_then(|v| v.as_i64())?;
                                    let first = item
                                        .get("firstName")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");
                                    let last =
                                        item.get("lastName").and_then(|v| v.as_str()).unwrap_or("");
                                    let dept = item
                                        .get("departmentName")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("");
                                    Some(format!(
                                        "Show employee {} ({} {} - {})",
                                        id, first, last, dept
                                    ))
                                })
                                .collect();
                            if !chips.is_empty() { Some(chips) } else { None }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            Ok(Json(ProcessResponse {
                prompt,
                intent: intent_name,
                tool: intent_result.intent.tool_name().map(|s| s.to_string()),
                params: params_json,
                steps,
                result: outcome
                    .data
                    .or_else(|| outcome.message.as_ref().map(|m| json!({ "message": m }))),
                success: outcome.success,
                requires_confirmation: None,
                message: outcome.message,
                suggestions: disambiguation_suggestions,
                execution_plan: Some(json!(plan)),
            }))
        }
        Err(e) => {
            context.record_failure();
            update_context(&session_id, context);

            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Plan execution failed: {}", e) })),
            ))
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn intent_label(intent: &Intent) -> String {
    match intent {
        Intent::GetAllEmployees => "get_all_employees".to_string(),
        Intent::GetEmployeeById => "get_employee_by_id".to_string(),
        Intent::SearchEmployees => "search_employees".to_string(),
        Intent::GetDepartmentList => "get_department_list".to_string(),
        Intent::GetDesignationList => "get_designation_list".to_string(),
        Intent::GetEmployeeCount => "get_employee_count".to_string(),
        Intent::AddEmployee => "add_employee".to_string(),
        Intent::UpdateEmployee => "update_employee".to_string(),
        Intent::CalculateAnalytics => "calculate_analytics".to_string(),
        Intent::UndoLastUpdate => "undo_last_update".to_string(),
        Intent::BatchUpdate => "batch_update".to_string(),
        Intent::Unknown(s) => format!("unknown({})", s),
    }
}

fn params_to_json(ir: &IntentResult) -> Value {
    let p = &ir.params;
    let mut obj = serde_json::Map::new();
    if !p.requested_fields.is_empty() {
        obj.insert("requestedFields".into(), json!(p.requested_fields));
    }
    if let Some(id) = p.employee_id {
        obj.insert("employeeId".into(), json!(id));
    }
    if let Some(ref v) = p.first_name {
        obj.insert("firstName".into(), json!(v));
    }
    if let Some(ref v) = p.last_name {
        obj.insert("lastName".into(), json!(v));
    }
    if let Some(ref v) = p.email {
        obj.insert("email".into(), json!(v));
    }
    if let Some(ref v) = p.phone {
        obj.insert("phone".into(), json!(v));
    }
    if let Some(ref v) = p.department_name {
        obj.insert("departmentName".into(), json!(v));
    }
    if let Some(ref v) = p.designation_name {
        obj.insert("designationName".into(), json!(v));
    }
    if let Some(v) = p.salary {
        obj.insert("salary".into(), json!(v));
    }
    if let Some(ref v) = p.hire_date {
        obj.insert("hireDate".into(), json!(v));
    }
    if let Some(ref v) = p.status {
        obj.insert("status".into(), json!(v));
    }
    if let Some(ref v) = p.field_to_update {
        obj.insert("fieldToUpdate".into(), json!(v));
    }
    if let Some(ref v) = p.new_value {
        obj.insert("newValue".into(), json!(v));
    }
    Value::Object(obj)
}

/// Mirror of main.rs build_tool_args, but returns JSON Value directly.
#[allow(dead_code)]
fn build_tool_args_json(ir: &IntentResult) -> Result<Value, String> {
    use crate::models::Intent::*;
    let p = &ir.params;

    match &ir.intent {
        GetEmployeeById => {
            let id = p
                .employee_id
                .ok_or("Employee ID is required. Example: 'find employee 1001'")?;
            Ok(json!({"employeeId": id}))
        }
        GetEmployeeCount => {
            let mut args = serde_json::Map::new();
            if let Some(ref d) = p.department_name {
                args.insert("department".into(), json!(d));
            }
            if let Some(ref s) = p.status {
                args.insert("status".into(), json!(s));
            }
            Ok(Value::Object(args))
        }
        SearchEmployees => {
            let mut args = serde_json::Map::new();
            if let Some(ref d) = p.department_name {
                args.insert("departmentName".into(), json!(d));
            }
            if let Some(ref dg) = p.designation_name {
                args.insert("designationName".into(), json!(dg));
            }
            if let Some(ref fn_) = p.first_name {
                args.insert("firstName".into(), json!(fn_));
            }
            if let Some(ref ln) = p.last_name {
                args.insert("lastName".into(), json!(ln));
            }
            if let Some(ref s) = p.status {
                args.insert("status".into(), json!(s));
            }
            Ok(Value::Object(args))
        }
        AddEmployee => {
            let first = p
                .first_name
                .as_deref()
                .ok_or("First name is required for AddEmployee")?;
            let last = p
                .last_name
                .as_deref()
                .ok_or("Last name is required for AddEmployee")?;
            let email = p
                .email
                .as_deref()
                .ok_or("Email is required for AddEmployee")?;
            let dept = p
                .department_name
                .as_deref()
                .ok_or("Department is required for AddEmployee")?;
            let desig = p
                .designation_name
                .as_deref()
                .ok_or("Designation is required for AddEmployee")?;

            let mut args = serde_json::Map::new();
            args.insert("firstName".into(), json!(first));
            args.insert("lastName".into(), json!(last));
            args.insert("email".into(), json!(email));
            args.insert("departmentName".into(), json!(dept));
            args.insert("designationName".into(), json!(desig));
            if let Some(s) = p.salary {
                args.insert("salary".into(), json!(s));
            }
            if let Some(ref d) = p.hire_date {
                args.insert("hireDate".into(), json!(d));
            }
            if let Some(ref ph) = p.phone {
                args.insert("phone".into(), json!(ph));
            }
            if let Some(ref st) = p.status {
                args.insert("status".into(), json!(st));
            }
            Ok(Value::Object(args))
        }
        UpdateEmployee => {
            let id = p
                .employee_id
                .ok_or("Employee ID is required for UpdateEmployee")?;
            let field = p
                .field_to_update
                .as_deref()
                .ok_or("Field to update is required")?;
            let val = p.new_value.as_deref().ok_or("New value is required")?;

            let field_key = match field.to_lowercase().as_str() {
                "department" | "departmentname" => "departmentName",
                "designation" | "designationname" => "designationName",
                "salary" => "salary",
                "status" => "status",
                "email" => "email",
                "phone" => "phone",
                "firstname" => "firstName",
                "lastname" => "lastName",
                _ => field,
            };

            Ok(json!({
                "employeeId": id,
                "fieldName": field_key,
                "newValue": val,
            }))
        }
        GetAllEmployees | GetDepartmentList | GetDesignationList | CalculateAnalytics
        | UndoLastUpdate | BatchUpdate => Ok(json!({})),
        Unknown(_) => Ok(json!({})),
    }
}
