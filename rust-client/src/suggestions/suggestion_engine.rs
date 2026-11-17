// =============================================================================
// suggestions/suggestion_engine.rs — Generates context-aware prompt suggestions
// =============================================================================

use crate::context::conversation_context::ConversationContext;
use crate::models::IntentResult;
use crate::suggestions::context_analyzer::analyze_context;
use crate::suggestions::error_classifier::{
    ClassificationResult, PromptErrorType, classify_prompt_error,
};
use crate::suggestions::suggestion_templates::*;

#[derive(Debug, Clone)]
pub struct SuggestionOutput {
    pub error_type: PromptErrorType,
    pub explanation: String,
    pub suggestions: Vec<String>,
}

/// Generates up to 3-5 relevant, deduplicated suggestions based on classification & context.
pub fn generate_suggestions(
    prompt: &str,
    ir: &IntentResult,
    ctx: &ConversationContext,
) -> SuggestionOutput {
    let classification: ClassificationResult = classify_prompt_error(prompt, ir);
    let context_summary = analyze_context(ir, ctx);

    let context_id_str = context_summary
        .active_employee_id
        .map(|id| format!("{}", id));

    let mut raw_suggestions = match classification.error_type {
        PromptErrorType::UnsupportedOperation => {
            let entity_id = classification
                .detected_entity
                .as_deref()
                .or(context_id_str.as_deref());
            templates_unsupported_delete(entity_id)
        }
        PromptErrorType::MissingParameter => {
            if prompt.to_lowercase().contains("update") {
                let entity_id = classification
                    .detected_entity
                    .as_deref()
                    .or(context_id_str.as_deref());
                templates_incomplete_update(entity_id)
            } else {
                templates_missing_id()
            }
        }
        PromptErrorType::InvalidParameter => templates_invalid_id(),
        PromptErrorType::AmbiguousIntent => {
            if let Some(ref entity) = classification.detected_entity {
                if entity.chars().all(|c| c.is_ascii_digit()) {
                    vec![
                        format!("Find employee {}", entity),
                        format!("Show employee {} details", entity),
                        format!("Update employee {} salary to 180000", entity),
                        format!("Find employee {} department name", entity),
                    ]
                } else {
                    templates_department(entity)
                }
            } else if let Some(ref dept) = context_summary.active_department {
                templates_department(dept)
            } else {
                templates_general_fallback()
            }
        }
        PromptErrorType::UnrelatedPrompt => templates_unrelated(),
        _ => templates_general_fallback(),
    };

    // If a typo autocorrection was proposed, put it as the primary suggestion chip
    if classification.user_explanation.starts_with("Did you mean: '") {
        if let Some(start) = classification.user_explanation.find('\'') {
            if let Some(end) = classification.user_explanation.rfind('\'') {
                if start < end {
                    let corrected = classification.user_explanation[start + 1..end].to_string();
                    raw_suggestions.insert(0, corrected);
                }
            }
        }
    }

    // Deduplicate suggestions and limit to at most 4
    let mut deduplicated: Vec<String> = Vec::new();
    for s in raw_suggestions {
        if !deduplicated.contains(&s) {
            deduplicated.push(s);
        }
        if deduplicated.len() >= 4 {
            break;
        }
    }

    SuggestionOutput {
        error_type: classification.error_type,
        explanation: classification.user_explanation,
        suggestions: deduplicated,
    }
}
