// =============================================================================
// context/context_resolver.rs — Deterministic follow-up & pronoun resolution
// =============================================================================

use crate::context::conversation_context::ConversationContext;
use crate::models::ExtractedParams;
use once_cell::sync::Lazy;
use regex::Regex;

pub static RE_PRONOUN_REF: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(his|her|their|that\s+employee|this\s+employee|same\s+employee|the\s+employee)\b",
    )
    .unwrap()
});

pub static RE_FOLLOWUP_SALARY: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(what\s+is\s+|show\s+|get\s+|tell\s+me\s+)?(his|her|their|that\s+employee's|the\s+employee's)?\s*salary\??$").unwrap()
});

pub static RE_FOLLOWUP_DEPT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(what\s+about\s+|what\s+is\s+|show\s+|tell\s+me\s+)?(his|her|their|that\s+employee's|the\s+employee's)?\s*(department|dept)\??$").unwrap()
});

pub static RE_FOLLOWUP_EMAIL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(what\s+is\s+|show\s+|get\s+|tell\s+me\s+)?(his|her|their|that\s+employee's|the\s+employee's)?\s*email(\s+too)?\??$").unwrap()
});

pub static RE_FOLLOWUP_DESIG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(what\s+is\s+|show\s+|get\s+|tell\s+me\s+)?(his|her|their|that\s+employee's|the\s+employee's)?\s*(designation|role|title|job\s+title)\??$").unwrap()
});

pub static RE_FOLLOWUP_STATUS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(what\s+is\s+|show\s+|get\s+|tell\s+me\s+)?(his|her|their|that\s+employee's|the\s+employee's)?\s*status\??$").unwrap()
});

pub static RE_DEPT_SWITCH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)^what\s+about\s+(?:the\s+)?(?P<dept>[A-Za-z][A-Za-z\s]*?)(?:\s+department)?\??$",
    )
    .unwrap()
});

pub static RE_IN_MEMORY_FILTER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(only\s+show|just\s+show|show\s+only)\s+(their|the)?\s*(?P<fields>.+)$")
        .unwrap()
});

pub static RE_CLEAR_CONTEXT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)^(clear\s+context|reset\s+memory|new\s+conversation|forget\s+everything|reset)$",
    )
    .unwrap()
});

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedContextAction {
    /// Context cleared
    ContextCleared,
    /// Rewrite prompt to an explicit query using resolved employee ID
    EmployeeFollowUp {
        employee_id: i64,
        requested_field: String,
        rewritten_prompt: String,
    },
    /// Switch search department
    DepartmentSwitch {
        department: String,
        rewritten_prompt: String,
    },
    /// Filter previous search result in memory
    InMemoryProjection { requested_fields: Vec<String> },
    /// No contextual follow-up detected; proceed with raw prompt
    None,
}

/// Analyzes the user prompt in the context of previous conversation turns.
pub fn resolve_context(prompt: &str, ctx: &ConversationContext) -> ResolvedContextAction {
    let p = prompt.trim();

    // Check for clear context request
    if RE_CLEAR_CONTEXT.is_match(p) {
        return ResolvedContextAction::ContextCleared;
    }

    // Check for "What about HR?" department switch follow-up
    if let Some(caps) = RE_DEPT_SWITCH.captures(p) {
        if let Some(dept_match) = caps.name("dept") {
            let dept = dept_match.as_str().trim().to_string();
            // Don't mistake "what about his department" for a department switch
            if !dept.eq_ignore_ascii_case("his")
                && !dept.eq_ignore_ascii_case("her")
                && !dept.eq_ignore_ascii_case("their")
            {
                return ResolvedContextAction::DepartmentSwitch {
                    department: dept.clone(),
                    rewritten_prompt: format!("Search employees in {}", dept),
                };
            }
        }
    }

    // Check for "Only show their names" in-memory projection
    if let Some(caps) = RE_IN_MEMORY_FILTER.captures(p) {
        if ctx.last_result_data.is_some() {
            let fields_str = caps.name("fields").map(|f| f.as_str()).unwrap_or("");
            let fields =
                crate::intent::vocabulary::Vocabulary::get().match_requested_fields(fields_str);
            if !fields.is_empty() {
                return ResolvedContextAction::InMemoryProjection {
                    requested_fields: fields,
                };
            }
        }
    }

    // Check for employee pronoun reference or specific field follow-up
    if let Some(emp_id) = ctx.last_employee_id {
        if RE_FOLLOWUP_SALARY.is_match(p) {
            return ResolvedContextAction::EmployeeFollowUp {
                employee_id: emp_id,
                requested_field: "salary".to_string(),
                rewritten_prompt: format!("Find employee {} salary", emp_id),
            };
        }
        if RE_FOLLOWUP_DEPT.is_match(p) {
            return ResolvedContextAction::EmployeeFollowUp {
                employee_id: emp_id,
                requested_field: "departmentName".to_string(),
                rewritten_prompt: format!("Find employee {} department name", emp_id),
            };
        }
        if RE_FOLLOWUP_EMAIL.is_match(p) {
            return ResolvedContextAction::EmployeeFollowUp {
                employee_id: emp_id,
                requested_field: "email".to_string(),
                rewritten_prompt: format!("Find employee {} email", emp_id),
            };
        }
        if RE_FOLLOWUP_DESIG.is_match(p) {
            return ResolvedContextAction::EmployeeFollowUp {
                employee_id: emp_id,
                requested_field: "designationName".to_string(),
                rewritten_prompt: format!("Find employee {} designation", emp_id),
            };
        }
        if RE_FOLLOWUP_STATUS.is_match(p) {
            return ResolvedContextAction::EmployeeFollowUp {
                employee_id: emp_id,
                requested_field: "status".to_string(),
                rewritten_prompt: format!("Find employee {} status", emp_id),
            };
        }

        // Generic pronoun replacement (e.g., "update his salary to 180000" or "show his details")
        if RE_PRONOUN_REF.is_match(p) {
            let rewritten = RE_PRONOUN_REF
                .replace_all(p, &format!("employee {}", emp_id))
                .to_string();
            return ResolvedContextAction::EmployeeFollowUp {
                employee_id: emp_id,
                requested_field: String::new(),
                rewritten_prompt: rewritten,
            };
        }
    }

    ResolvedContextAction::None
}

/// Enriches extracted parameters with context if applicable
pub fn enrich_params_with_context(params: &mut ExtractedParams, ctx: &ConversationContext) {
    if params.employee_id.is_none() {
        if let Some(id) = ctx.last_employee_id {
            // Only backfill if the prompt contains a reference to previous employee
            params.employee_id = Some(id);
        }
    }
}
