// =============================================================================
// models.rs — Shared data structures for MCP JSON-RPC and tool results
// =============================================================================

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// JSON-RPC 2.0 structures
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    pub fn new(id: u64, method: &str, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(Value::Number(id.into())),
            method: method.to_string(),
            params,
        }
    }

    pub fn notification(method: &str, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: method.to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    pub data: Option<Value>,
}

// ---------------------------------------------------------------------------
// MCP Tool Schema structures (from tools/list)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpTool {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: ToolInputSchema,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolInputSchema {
    #[serde(rename = "type")]
    pub schema_type: Option<String>,
    pub properties: Option<HashMap<String, PropertySchema>>,
    pub required: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PropertySchema {
    #[serde(rename = "type")]
    pub prop_type: Option<String>,
    pub description: Option<String>,
    pub default: Option<Value>,
}

// ---------------------------------------------------------------------------
// Intent & extraction models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    GetAllEmployees,
    GetEmployeeById,
    SearchEmployees,
    GetDepartmentList,
    GetDesignationList,
    GetEmployeeCount,
    AddEmployee,
    UpdateEmployee,
    Unknown(String),
}

impl Intent {
    pub fn tool_name(&self) -> Option<&str> {
        match self {
            Intent::GetAllEmployees => Some("get_all_employees"),
            Intent::GetEmployeeById => Some("get_employee_by_id"),
            Intent::SearchEmployees => Some("search_employees"),
            Intent::GetDepartmentList => Some("get_department_list"),
            Intent::GetDesignationList => Some("get_designation_list"),
            Intent::GetEmployeeCount => Some("get_employee_count"),
            Intent::AddEmployee => Some("add_employee"),
            Intent::UpdateEmployee => Some("update_employee"),
            Intent::Unknown(_) => None,
        }
    }

    pub fn requires_confirmation(&self) -> bool {
        matches!(self, Intent::AddEmployee | Intent::UpdateEmployee)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExtractedParams {
    pub employee_id: Option<i64>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub department_name: Option<String>,
    pub designation_name: Option<String>,
    pub salary: Option<f64>,
    pub hire_date: Option<String>,
    pub status: Option<String>,
    pub field_to_update: Option<String>,
    pub new_value: Option<String>,
    pub requested_fields: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct IntentResult {
    pub intent: Intent,
    pub params: ExtractedParams,
    pub confidence: u8, // 1-100
    pub raw_prompt: String,
}

// ---------------------------------------------------------------------------
// Tool execution result
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct ToolExecutionResult {
    pub tool_name: String,
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<Value>,
    pub raw_response: String,
}
