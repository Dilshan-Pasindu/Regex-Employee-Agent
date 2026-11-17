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

    // Handle invalid or bare employee lookups (e.g. "find employee abc", "find employee")
    if is_invalid_employee_id(prompt) || is_bare_employee_lookup(prompt) {
        return IntentResult {
            intent: Intent::Unknown(prompt.to_string()),
            params: Default::default(),
            confidence: 0,
            raw_prompt: raw_prompt.to_string(),
        };
    }

    // --- PRIORITY 0: undo_last_update (Feature 3) ---
    if let Some(caps) = RE_UNDO.captures(prompt) {
        let mut params = crate::models::ExtractedParams::default();
        if let Some(m) = caps.name("id") {
            params.employee_id = m.as_str().parse::<i64>().ok();
        }
        return IntentResult {
            intent: Intent::UndoLastUpdate,
            params,
            confidence: 95,
            raw_prompt: raw_prompt.to_string(),
        };
    }

    // --- PRIORITY 0.5: batch_update (Feature 5) ---
    if let Some(caps) = RE_BATCH_SALARY_INCREASE.captures(prompt) {
        let mut params = crate::models::ExtractedParams::default();
        if let Some(role) = caps.name("role") {
            params.designation_name = Some(role.as_str().trim().to_string());
        }
        if let Some(amt) = caps.name("amount") {
            params.batch_delta = amt.as_str().parse::<f64>().ok();
        }
        params.batch_action = Some("salary_increase".to_string());
        return IntentResult {
            intent: Intent::BatchUpdate,
            params,
            confidence: 90,
            raw_prompt: raw_prompt.to_string(),
        };
    }

    if let Some(caps) = RE_BATCH_TRANSFER.captures(prompt) {
        let mut params = crate::models::ExtractedParams::default();
        if let Some(role) = caps.name("role") {
            params.designation_name = Some(role.as_str().trim().to_string());
        }
        if let Some(from) = caps.name("from_dept") {
            params.department_name = Some(from.as_str().trim().to_string());
        }
        if let Some(to) = caps.name("to_dept") {
            params.new_value = Some(to.as_str().trim().to_string());
            params.field_to_update = Some("department".to_string());
        }
        params.batch_action = Some("transfer".to_string());
        return IntentResult {
            intent: Intent::BatchUpdate,
            params,
            confidence: 90,
            raw_prompt: raw_prompt.to_string(),
        };
    }

    // --- PRIORITY 0.8: calculate_analytics (Feature 1) ---
    if let Some(caps) = RE_ANALYTICS_AVERAGE.captures(prompt) {
        let mut params = extract_params(prompt);
        if let Some(dept) = caps.name("dept") {
            params.department_name = Some(dept.as_str().trim().to_string());
        }
        params.aggregation_type = Some("average".to_string());
        return IntentResult {
            intent: Intent::CalculateAnalytics,
            params,
            confidence: 92,
            raw_prompt: raw_prompt.to_string(),
        };
    }

    if let Some(caps) = RE_ANALYTICS_HIGHEST.captures(prompt) {
        let mut params = extract_params(prompt);
        if let Some(dept) = caps.name("dept") {
            params.department_name = Some(dept.as_str().trim().to_string());
        }
        params.aggregation_type = Some("highest".to_string());
        return IntentResult {
            intent: Intent::CalculateAnalytics,
            params,
            confidence: 92,
            raw_prompt: raw_prompt.to_string(),
        };
    }

    if let Some(caps) = RE_ANALYTICS_LOWEST.captures(prompt) {
        let mut params = extract_params(prompt);
        if let Some(dept) = caps.name("dept") {
            params.department_name = Some(dept.as_str().trim().to_string());
        }
        params.aggregation_type = Some("lowest".to_string());
        return IntentResult {
            intent: Intent::CalculateAnalytics,
            params,
            confidence: 92,
            raw_prompt: raw_prompt.to_string(),
        };
    }

    if let Some(caps) = RE_ANALYTICS_TOTAL.captures(prompt) {
        let mut params = extract_params(prompt);
        if let Some(dept) = caps.name("dept") {
            params.department_name = Some(dept.as_str().trim().to_string());
        }
        params.aggregation_type = Some("total".to_string());
        return IntentResult {
            intent: Intent::CalculateAnalytics,
            params,
            confidence: 92,
            raw_prompt: raw_prompt.to_string(),
        };
    }

    // --- PRIORITY 1: update_employee ---
    // Must come BEFORE get_employee_by_id so "update employee 1001 ..." is not mistaken for a read lookup
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

    // --- PRIORITY 2: get_employee_by_id ---
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

    // --- Typo-tolerance fallback via deterministic Levenshtein distance ---
    let corrected = crate::intent::fuzzy::autocorrect_prompt(prompt);
    if corrected != prompt {
        // Only retry if not an invalid ID or bare query
        if !is_invalid_employee_id(&corrected) && !is_bare_employee_lookup(&corrected) {
            for re in RE_GET_BY_ID.iter() {
                if re.is_match(&corrected) {
                    return IntentResult {
                        intent: Intent::GetEmployeeById,
                        params: extract_params(&corrected),
                        confidence: 85,
                        raw_prompt: raw_prompt.to_string(),
                    };
                }
            }
            for re in RE_SEARCH_EMPLOYEES.iter() {
                if re.is_match(&corrected) {
                    return IntentResult {
                        intent: Intent::SearchEmployees,
                        params: extract_params(&corrected),
                        confidence: 85,
                        raw_prompt: raw_prompt.to_string(),
                    };
                }
            }
            for re in RE_GET_COUNT.iter() {
                if re.is_match(&corrected) {
                    return IntentResult {
                        intent: Intent::GetEmployeeCount,
                        params: extract_params(&corrected),
                        confidence: 85,
                        raw_prompt: raw_prompt.to_string(),
                    };
                }
            }
            for re in RE_GET_ALL_EMPLOYEES.iter() {
                if re.is_match(&corrected) {
                    return IntentResult {
                        intent: Intent::GetAllEmployees,
                        params: extract_params(&corrected),
                        confidence: 85,
                        raw_prompt: raw_prompt.to_string(),
                    };
                }
            }
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
