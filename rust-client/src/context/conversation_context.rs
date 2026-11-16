// =============================================================================
// context/conversation_context.rs — Session-based structured conversational memory
// =============================================================================

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Global in-memory session store: session_id -> ConversationContext
pub static SESSION_STORE: Lazy<Arc<Mutex<HashMap<String, ConversationContext>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Structured context preserved across messages in a conversation session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConversationContext {
    pub last_intent: Option<String>,
    pub last_employee_id: Option<i64>,
    pub last_employee_name: Option<String>,
    pub last_employee_email: Option<String>,
    pub last_department: Option<String>,
    pub last_designation: Option<String>,
    pub last_salary: Option<f64>,
    pub last_status: Option<String>,
    pub last_requested_fields: Vec<String>,
    pub last_result_data: Option<Value>,
    pub last_result_count: Option<usize>,
    pub last_operation_success: Option<bool>,
    pub turn_count: usize,
}

impl ConversationContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update context with successful employee lookup
    pub fn record_employee_lookup(
        &mut self,
        id: i64,
        name: Option<String>,
        email: Option<String>,
        dept: Option<String>,
        desig: Option<String>,
        salary: Option<f64>,
        status: Option<String>,
        data: Option<Value>,
    ) {
        self.last_intent = Some("get_employee_by_id".to_string());
        self.last_employee_id = Some(id);
        if name.is_some() {
            self.last_employee_name = name;
        }
        if email.is_some() {
            self.last_employee_email = email;
        }
        if dept.is_some() {
            self.last_department = dept;
        }
        if desig.is_some() {
            self.last_designation = desig;
        }
        if salary.is_some() {
            self.last_salary = salary;
        }
        if status.is_some() {
            self.last_status = status;
        }
        self.last_result_data = data;
        self.last_operation_success = Some(true);
        self.turn_count += 1;
    }

    /// Update context with successful search
    pub fn record_search(
        &mut self,
        dept: Option<String>,
        desig: Option<String>,
        status: Option<String>,
        data: Option<Value>,
        count: usize,
    ) {
        self.last_intent = Some("search_employees".to_string());
        if dept.is_some() {
            self.last_department = dept;
        }
        if desig.is_some() {
            self.last_designation = desig;
        }
        if status.is_some() {
            self.last_status = status;
        }
        self.last_result_data = data;
        self.last_result_count = Some(count);
        self.last_operation_success = Some(true);
        self.turn_count += 1;
    }

    /// Record operation failure — preserves valid entity memory
    pub fn record_failure(&mut self) {
        self.last_operation_success = Some(false);
        self.turn_count += 1;
    }

    /// Reset/clear context explicitly
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

/// Helper to get or create a session context
pub fn get_or_create_context(session_id: &str) -> ConversationContext {
    let mut store = SESSION_STORE.lock().unwrap();
    store.entry(session_id.to_string()).or_default().clone()
}

/// Helper to update a session context
pub fn update_context(session_id: &str, ctx: ConversationContext) {
    let mut store = SESSION_STORE.lock().unwrap();
    store.insert(session_id.to_string(), ctx);
}

/// Helper to clear session context
pub fn clear_session_context(session_id: &str) {
    let mut store = SESSION_STORE.lock().unwrap();
    store.remove(session_id);
}
