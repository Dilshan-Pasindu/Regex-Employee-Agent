// =============================================================================
// suggestions/context_analyzer.rs — Extracts context & entities for suggestions
// =============================================================================

use crate::context::conversation_context::ConversationContext;
use crate::models::IntentResult;

#[derive(Debug, Clone, Default)]
pub struct ContextSummary {
    pub active_employee_id: Option<i64>,
    pub active_department: Option<String>,
    pub last_intent: Option<String>,
}

pub fn analyze_context(ir: &IntentResult, ctx: &ConversationContext) -> ContextSummary {
    let mut summary = ContextSummary::default();

    // Prefer newly extracted entities
    if let Some(id) = ir.params.employee_id {
        summary.active_employee_id = Some(id);
    } else if let Some(last_id) = ctx.last_employee_id {
        summary.active_employee_id = Some(last_id);
    }

    if let Some(ref dept) = ir.params.department_name {
        summary.active_department = Some(dept.clone());
    } else if let Some(ref last_dept) = ctx.last_department {
        summary.active_department = Some(last_dept.clone());
    }

    summary.last_intent = ctx.last_intent.clone();
    summary
}
