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
    extract::State,
    http::{Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    intent::detector::detect_intent,
    mcp::client::McpClient,
    models::{Intent, IntentResult},
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
    let prompt = req.prompt.trim().to_string();
    if prompt.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "prompt cannot be empty"})),
        ));
    }

    // Step 1 — Intent detection
    let intent_result: IntentResult = detect_intent(&prompt);

    let intent_name = intent_label(&intent_result.intent);
    let tool_name = intent_result.intent.tool_name().map(|s| s.to_string());

    let mut steps: Vec<Step> = Vec::new();
    steps.push(Step {
        step: 1,
        label: "Intent Recognized".to_string(),
        value: json!(intent_name),
    });

    // Build params JSON for transparency
    let params_json = params_to_json(&intent_result);
    steps.push(Step {
        step: 2,
        label: "Parameters Extracted".to_string(),
        value: params_json.clone(),
    });

    // Unknown intent → return early
    if matches!(intent_result.intent, Intent::Unknown(_)) {
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
                "I didn't understand that prompt. Try: 'show all employees', 'find employee 1001', \
                 'add employee...', 'update employee 1001...', 'count employees', 'show departments'."
                    .to_string(),
            ),
        }));
    }

    // Write operations — confirmation gate
    if intent_result.intent.requires_confirmation() && !req.confirmed {
        steps.push(Step {
            step: 3,
            label: "Awaiting Confirmation".to_string(),
            value: json!("Write operation requires explicit confirmation before execution."),
        });

        return Ok(Json(ProcessResponse {
            prompt,
            intent: intent_name,
            tool: tool_name,
            params: params_json,
            steps,
            result: None,
            success: false,
            requires_confirmation: Some(true),
            message: Some("Please confirm this write operation.".to_string()),
        }));
    }

    // Step 3 — build tool args
    let Some(ref tool) = tool_name else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "No tool mapped for this intent"})),
        ));
    };

    let args = build_tool_args_json(&intent_result).map_err(|msg| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": msg})),
        )
    })?;

    steps.push(Step {
        step: 3,
        label: "MCP Tool Dispatched".to_string(),
        value: json!(tool),
    });

    // Step 4 — call MCP tool
    let mut client = state.client.lock().await;
    match client.call_tool(tool, args).await {
        Ok(exec_result) => {
            steps.push(Step {
                step: 4,
                label: "Result Received".to_string(),
                value: json!(if exec_result.success { "200 OK" } else { "Error" }),
            });

            Ok(Json(ProcessResponse {
                prompt,
                intent: intent_name,
                tool: Some(tool.clone()),
                params: params_json,
                steps,
                result: exec_result.data.or_else(|| {
                    exec_result.message.as_ref().map(|m| json!({"message": m}))
                }),
                success: exec_result.success,
                requires_confirmation: None,
                message: exec_result.message,
            }))
        }
        Err(e) => {
            steps.push(Step {
                step: 4,
                label: "Error".to_string(),
                value: json!(e.to_string()),
            });

            Ok(Json(ProcessResponse {
                prompt,
                intent: intent_name,
                tool: Some(tool.clone()),
                params: params_json,
                steps,
                result: None,
                success: false,
                requires_confirmation: None,
                message: Some(e.to_string()),
            }))
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn intent_label(intent: &Intent) -> String {
    match intent {
        Intent::GetAllEmployees    => "get_all_employees".to_string(),
        Intent::GetEmployeeById    => "get_employee_by_id".to_string(),
        Intent::SearchEmployees    => "search_employees".to_string(),
        Intent::GetDepartmentList  => "get_department_list".to_string(),
        Intent::GetDesignationList => "get_designation_list".to_string(),
        Intent::GetEmployeeCount   => "get_employee_count".to_string(),
        Intent::AddEmployee        => "add_employee".to_string(),
        Intent::UpdateEmployee     => "update_employee".to_string(),
        Intent::Unknown(s)         => format!("unknown({})", s),
    }
}

fn params_to_json(ir: &IntentResult) -> Value {
    let p = &ir.params;
    let mut obj = serde_json::Map::new();
    if let Some(id) = p.employee_id      { obj.insert("employeeId".into(), json!(id)); }
    if let Some(ref v) = p.first_name    { obj.insert("firstName".into(), json!(v)); }
    if let Some(ref v) = p.last_name     { obj.insert("lastName".into(), json!(v)); }
    if let Some(ref v) = p.email         { obj.insert("email".into(), json!(v)); }
    if let Some(ref v) = p.phone         { obj.insert("phone".into(), json!(v)); }
    if let Some(ref v) = p.department_name { obj.insert("departmentName".into(), json!(v)); }
    if let Some(ref v) = p.designation_name { obj.insert("designationName".into(), json!(v)); }
    if let Some(v) = p.salary            { obj.insert("salary".into(), json!(v)); }
    if let Some(ref v) = p.hire_date     { obj.insert("hireDate".into(), json!(v)); }
    if let Some(ref v) = p.status        { obj.insert("status".into(), json!(v)); }
    if let Some(ref v) = p.field_to_update { obj.insert("fieldToUpdate".into(), json!(v)); }
    if let Some(ref v) = p.new_value     { obj.insert("newValue".into(), json!(v)); }
    Value::Object(obj)
}

/// Mirror of main.rs build_tool_args, but returns JSON Value directly.
fn build_tool_args_json(ir: &IntentResult) -> Result<Value, String> {
    use crate::models::Intent::*;
    let p = &ir.params;

    match &ir.intent {
        GetEmployeeById => {
            let id = p.employee_id.ok_or("Employee ID is required. Example: 'find employee 1001'")?;
            Ok(json!({"employeeId": id}))
        }
        GetEmployeeCount => {
            let mut args = serde_json::Map::new();
            if let Some(ref d) = p.department_name {
                args.insert("departmentName".into(), json!(d));
            }
            if let Some(ref s) = p.status {
                args.insert("status".into(), json!(s));
            }
            Ok(Value::Object(args))
        }
        SearchEmployees => {
            let mut args = serde_json::Map::new();
            if let Some(ref d) = p.department_name  { args.insert("departmentName".into(), json!(d)); }
            if let Some(ref dg) = p.designation_name { args.insert("designationName".into(), json!(dg)); }
            if let Some(ref fn_) = p.first_name     { args.insert("firstName".into(), json!(fn_)); }
            if let Some(ref ln) = p.last_name        { args.insert("lastName".into(), json!(ln)); }
            if let Some(ref s) = p.status            { args.insert("status".into(), json!(s)); }
            Ok(Value::Object(args))
        }
        AddEmployee => {
            let first = p.first_name.as_deref().ok_or("First name is required for AddEmployee")?;
            let last  = p.last_name.as_deref().ok_or("Last name is required for AddEmployee")?;
            let email = p.email.as_deref().ok_or("Email is required for AddEmployee")?;
            let dept  = p.department_name.as_deref().ok_or("Department is required for AddEmployee")?;
            let desig = p.designation_name.as_deref().ok_or("Designation is required for AddEmployee")?;

            let mut args = serde_json::Map::new();
            args.insert("firstName".into(), json!(first));
            args.insert("lastName".into(), json!(last));
            args.insert("email".into(), json!(email));
            args.insert("departmentName".into(), json!(dept));
            args.insert("designationName".into(), json!(desig));
            if let Some(s) = p.salary          { args.insert("salary".into(), json!(s)); }
            if let Some(ref d) = p.hire_date   { args.insert("hireDate".into(), json!(d)); }
            if let Some(ref ph) = p.phone      { args.insert("phone".into(), json!(ph)); }
            if let Some(ref st) = p.status     { args.insert("status".into(), json!(st)); }
            Ok(Value::Object(args))
        }
        UpdateEmployee => {
            let id = p.employee_id.ok_or("Employee ID is required for UpdateEmployee")?;
            let field = p.field_to_update.as_deref().ok_or("Field to update is required")?;
            let val   = p.new_value.as_deref().ok_or("New value is required")?;

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
        GetAllEmployees | GetDepartmentList | GetDesignationList => Ok(json!({})),
        Unknown(_) => Ok(json!({})),
    }
}
