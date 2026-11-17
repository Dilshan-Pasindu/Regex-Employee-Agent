// =============================================================================
// suggestions/error_classifier.rs — Deterministic prompt error classification
// =============================================================================

use crate::intent::patterns::{RE_BARE_FIND_EMPLOYEE, RE_INVALID_ID};
use crate::models::{Intent, IntentResult};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromptErrorType {
    MissingParameter,
    InvalidParameter,
    UnsupportedOperation,
    UnknownField,
    AmbiguousIntent,
    UnrelatedPrompt,
    InvalidFilter,
    ValidationError,
    McpToolError,
    McpConnectionError,
}

// Regex patterns for classification
static RE_DELETE_OP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(delete|remove|destroy|drop|terminate|fire|archive)\b").unwrap()
});

static RE_WEATHER_OR_UNRELATED: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(weather|forecast|joke|movie|recipe|dinner|song|music|game|football|news|time\s+is\s+it|hello|hi|hey|how\s+are\s+you)\b").unwrap()
});

static RE_BARE_UPDATE_EMPLOYEE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)^(update|change|modify|edit)\s+(?:an?\s+)?employee(?:\s+(?P<id>\d{4,}))?\s*(\?|\.)?$",
    )
    .unwrap()
});

static RE_BARE_DEPT_WORKERS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(show|find|list|get)\s+(?:me\s+)?(?P<dept>[A-Za-z]+)\s+(workers|people|guys|members)\s*(\?|\.)?$").unwrap()
});

pub struct ClassificationResult {
    pub error_type: PromptErrorType,
    pub user_explanation: String,
    pub detected_entity: Option<String>,
}

/// Classifies why a user prompt failed or cannot be cleanly executed.
pub fn classify_prompt_error(prompt: &str, ir: &IntentResult) -> ClassificationResult {
    let p = prompt.trim();

    // 1. Check for explicitly unsupported operations (e.g. delete employee)
    if RE_DELETE_OP.is_match(p) {
        let emp_id = crate::intent::patterns::RE_EXTRACT_ID
            .captures(p)
            .and_then(|c| c.name("id"))
            .map(|m| m.as_str().to_string());

        return ClassificationResult {
            error_type: PromptErrorType::UnsupportedOperation,
            user_explanation: "Employee deletion is not currently supported. However, I can help you view or update this employee.".to_string(),
            detected_entity: emp_id,
        };
    }

    // 2. Check for bare "Find employee" (missing ID or filter)
    if RE_BARE_FIND_EMPLOYEE.is_match(p) {
        return ClassificationResult {
            error_type: PromptErrorType::MissingParameter,
            user_explanation:
                "I can help you find an employee. Please provide an employee ID or email address."
                    .to_string(),
            detected_entity: None,
        };
    }

    // 3. Check for incomplete update: "Update employee 1001"
    if let Some(caps) = RE_BARE_UPDATE_EMPLOYEE.captures(p) {
        let id_str = caps.name("id").map(|m| m.as_str().to_string());
        let id_display = id_str.as_deref().unwrap_or("this employee");
        return ClassificationResult {
            error_type: PromptErrorType::MissingParameter,
            user_explanation: format!(
                "What information would you like to update for employee {}?",
                id_display
            ),
            detected_entity: id_str,
        };
    }

    // 4. Check for invalid non-numeric employee ID: "Find employee abc"
    if !crate::intent::patterns::RE_EXTRACT_ID.is_match(p) {
        if let Some(caps) = RE_INVALID_ID.captures(p) {
            let val = caps.name("id").map(|m| m.as_str().to_string());
            // Avoid classifying valid keywords like "details" or "all"
            if let Some(ref v) = val {
                let lower = v.to_lowercase();
                if lower != "id"
                    && lower != "details"
                    && lower != "all"
                    && lower != "list"
                    && lower != "count"
                    && lower != "records"
                    && lower != "record"
                    && lower != "data"
                    && lower != "info"
                {
                    return ClassificationResult {
                        error_type: PromptErrorType::InvalidParameter,
                        user_explanation: "The employee ID appears to be invalid. Please provide a valid numeric employee ID.".to_string(),
                        detected_entity: val,
                    };
                }
            }
        }
    }

    // 5. Check for department-related informal queries: "Show me IT workers"
    if let Some(caps) = RE_BARE_DEPT_WORKERS.captures(p) {
        let dept = caps.name("dept").map(|m| m.as_str().to_string());
        return ClassificationResult {
            error_type: PromptErrorType::AmbiguousIntent,
            user_explanation: format!(
                "I found a request regarding the {} department. Here are suggested ways to query it:",
                dept.as_deref().unwrap_or("")
            ),
            detected_entity: dept,
        };
    }

    // 6. Check for completely unrelated queries (e.g. weather, recipes)
    if RE_WEATHER_OR_UNRELATED.is_match(p) {
        return ClassificationResult {
            error_type: PromptErrorType::UnrelatedPrompt,
            user_explanation:
                "I'm currently designed to assist with employee management operations.".to_string(),
            detected_entity: None,
        };
    }

    // 7. General unknown intent
    match &ir.intent {
        Intent::Unknown(_) => {
            // Check if user mentioned an employee ID anywhere in the prompt
            if let Some(caps) = crate::intent::patterns::RE_EXTRACT_ID.captures(p) {
                if let Some(id_str) = caps.name("id") {
                    let id = id_str.as_str().to_string();
                    return ClassificationResult {
                        error_type: PromptErrorType::AmbiguousIntent,
                        user_explanation: format!(
                            "I recognized employee ID {}. Here are suggested ways to view or update this employee:",
                            id
                        ),
                        detected_entity: Some(id),
                    };
                }
            }

            let corrected = crate::intent::fuzzy::autocorrect_prompt(p);
            if corrected != p {
                return ClassificationResult {
                    error_type: PromptErrorType::AmbiguousIntent,
                    user_explanation: format!("Did you mean: '{}'?", corrected),
                    detected_entity: None,
                };
            }

            ClassificationResult {
                error_type: PromptErrorType::AmbiguousIntent,
                user_explanation: "I didn't recognize that request. Please see the suggestions below for supported capabilities:".to_string(),
                detected_entity: None,
            }
        }
        _ => ClassificationResult {
            error_type: PromptErrorType::ValidationError,
            user_explanation: "The request could not be processed due to missing or invalid parameters.".to_string(),
            detected_entity: None,
        },
    }
}
