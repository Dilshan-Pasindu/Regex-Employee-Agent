// =============================================================================
// intent/detector.rs — Deterministic intent classification
// Priority-ordered matching. No LLM. No AI.
// =============================================================================

use crate::intent::{extractor::extract_params, patterns::*};
use crate::models::{Intent, IntentResult};

/// Classify a natural-language prompt into a deterministic Intent.
/// Matching is ordered by specificity — more specific patterns first.
pub fn detect_intent(raw_prompt: &str) -> IntentResult {
    let prompt = raw_prompt.trim();

    // Handle special commands first
    if RE_EXIT.is_match(prompt) {
        return IntentResult {
            intent: Intent::Unknown("exit".to_string()),
            params: Default::default(),
            confidence: 100,
            raw_prompt: raw_prompt.to_string(),
        };
    }

    // --- PRIORITY 1: get_employee_by_id ---
    // Must come BEFORE get_all_employees to avoid "show employee 1001" matching list
    for re in RE_GET_BY_ID.iter() {
        if re.is_match(prompt) {
            return IntentResult {
                intent: Intent::GetEmployeeById,
                params: extract_params(prompt),
                confidence: 95,
                raw_prompt: raw_prompt.to_string(),
            };
        }
    }

    // --- PRIORITY 2: update_employee ---
    // Must come BEFORE search to avoid "update employee 1001 department to IT" being mistaken
    for re in RE_UPDATE_EMPLOYEE.iter() {
        if re.is_match(prompt) {
            return IntentResult {
                intent: Intent::UpdateEmployee,
                params: extract_params(prompt),
                confidence: 90,
                raw_prompt: raw_prompt.to_string(),
            };
        }
    }

    // --- PRIORITY 3: add_employee ---
    for re in RE_ADD_EMPLOYEE.iter() {
        if re.is_match(prompt) {
            return IntentResult {
                intent: Intent::AddEmployee,
                params: extract_params(prompt),
                confidence: 90,
                raw_prompt: raw_prompt.to_string(),
            };
        }
    }

    // --- PRIORITY 4: get_employee_count ---
    for re in RE_GET_COUNT.iter() {
        if re.is_match(prompt) {
            return IntentResult {
                intent: Intent::GetEmployeeCount,
                params: extract_params(prompt),
                confidence: 90,
                raw_prompt: raw_prompt.to_string(),
            };
        }
    }

    // --- PRIORITY 5: get_department_list ---
    for re in RE_GET_DEPARTMENTS.iter() {
        if re.is_match(prompt) {
            return IntentResult {
                intent: Intent::GetDepartmentList,
                params: extract_params(prompt),
                confidence: 95,
                raw_prompt: raw_prompt.to_string(),
            };
        }
    }

    // --- PRIORITY 6: search_employees (filtered) ---
    for re in RE_SEARCH_EMPLOYEES.iter() {
        if re.is_match(prompt) {
            return IntentResult {
                intent: Intent::SearchEmployees,
                params: extract_params(prompt),
                confidence: 85,
                raw_prompt: raw_prompt.to_string(),
            };
        }
    }

    // --- PRIORITY 7: get_all_employees (broadest match — last) ---
    for re in RE_GET_ALL_EMPLOYEES.iter() {
        if re.is_match(prompt) {
            return IntentResult {
                intent: Intent::GetAllEmployees,
                params: extract_params(prompt),
                confidence: 95,
                raw_prompt: raw_prompt.to_string(),
            };
        }
    }

    // --- No pattern matched ---
    IntentResult {
        intent: Intent::Unknown(prompt.to_string()),
        params: Default::default(),
        confidence: 0,
        raw_prompt: raw_prompt.to_string(),
    }
}

