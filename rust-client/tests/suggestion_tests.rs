// =============================================================================
// tests/suggestion_tests.rs — Comprehensive unit tests for prompt suggestions
// =============================================================================

use employee_mcp_client::context::ConversationContext;
use employee_mcp_client::intent::detector::detect_intent;
use employee_mcp_client::suggestions::{
    PromptErrorType, classify_prompt_error, generate_suggestions,
};

#[test]
fn test_missing_employee_id() {
    let prompt = "Find employee";
    let ir = detect_intent(prompt);
    assert!(matches!(
        ir.intent,
        employee_mcp_client::models::Intent::Unknown(_)
    ));
    let ctx = ConversationContext::new();

    let classification = classify_prompt_error(prompt, &ir);
    assert_eq!(classification.error_type, PromptErrorType::MissingParameter);
    assert!(
        classification
            .user_explanation
            .contains("Please provide an employee ID or email address")
    );

    let out = generate_suggestions(prompt, &ir, &ctx);
    assert_eq!(out.error_type, PromptErrorType::MissingParameter);
    assert!(out.suggestions.contains(&"Find employee 1001".to_string()));
    assert!(out.suggestions.iter().any(|s| s.contains("email")));
}

#[test]
fn test_invalid_employee_id() {
    let prompt = "Find employee abc";
    let ir = detect_intent(prompt);
    assert!(matches!(
        ir.intent,
        employee_mcp_client::models::Intent::Unknown(_)
    ));
    let ctx = ConversationContext::new();

    let classification = classify_prompt_error(prompt, &ir);
    assert_eq!(classification.error_type, PromptErrorType::InvalidParameter);
    assert!(classification.user_explanation.contains("invalid"));

    let out = generate_suggestions(prompt, &ir, &ctx);
    assert_eq!(out.error_type, PromptErrorType::InvalidParameter);
    assert!(out.suggestions.contains(&"Find employee 1001".to_string()));
}

#[test]
fn test_unsupported_delete_operation() {
    let prompt = "Delete employee 1001";
    let ir = detect_intent(prompt);
    let ctx = ConversationContext::new();

    let classification = classify_prompt_error(prompt, &ir);
    assert_eq!(
        classification.error_type,
        PromptErrorType::UnsupportedOperation
    );
    assert!(
        classification
            .user_explanation
            .contains("deletion is not currently supported")
    );
    assert_eq!(classification.detected_entity.as_deref(), Some("1001"));

    let out = generate_suggestions(prompt, &ir, &ctx);
    assert_eq!(out.error_type, PromptErrorType::UnsupportedOperation);
    assert!(out.suggestions.contains(&"Find employee 1001".to_string()));
    assert!(
        out.suggestions
            .contains(&"Show employee 1001 details".to_string())
    );
    assert!(
        out.suggestions
            .contains(&"Update employee 1001 status to Inactive".to_string())
    );
    // Must NOT suggest delete
    assert!(
        !out.suggestions
            .iter()
            .any(|s| s.to_lowercase().contains("delete"))
    );
}

#[test]
fn test_incomplete_update_request() {
    let prompt = "Update employee 1001";
    let ir = detect_intent(prompt);
    let ctx = ConversationContext::new();

    let classification = classify_prompt_error(prompt, &ir);
    assert_eq!(classification.error_type, PromptErrorType::MissingParameter);
    assert!(classification.user_explanation.contains("1001"));

    let out = generate_suggestions(prompt, &ir, &ctx);
    assert_eq!(out.error_type, PromptErrorType::MissingParameter);
    assert!(out.suggestions.iter().any(|s| s.contains("salary")));
    assert!(out.suggestions.iter().any(|s| s.contains("department")));
    assert!(out.suggestions.iter().any(|s| s.contains("designation")));
}

#[test]
fn test_department_informal_query() {
    let prompt = "Show me IT workers";
    let ir = detect_intent(prompt);
    let ctx = ConversationContext::new();

    let out = generate_suggestions(prompt, &ir, &ctx);
    assert_eq!(out.error_type, PromptErrorType::AmbiguousIntent);
    assert!(out.suggestions.iter().any(|s| s.contains("IT department")));
    assert!(
        out.suggestions
            .iter()
            .any(|s| s.contains("more than 150000"))
    );
}

#[test]
fn test_unrelated_weather_prompt() {
    let prompt = "What's the weather today?";
    let ir = detect_intent(prompt);
    let ctx = ConversationContext::new();

    let classification = classify_prompt_error(prompt, &ir);
    assert_eq!(classification.error_type, PromptErrorType::UnrelatedPrompt);
    assert!(
        classification
            .user_explanation
            .contains("designed to assist with employee management")
    );

    let out = generate_suggestions(prompt, &ir, &ctx);
    assert_eq!(out.error_type, PromptErrorType::UnrelatedPrompt);
    // Suggestions must be related to employee management, not weather
    assert!(out.suggestions.contains(&"Find employees".to_string()));
    assert!(out.suggestions.contains(&"View departments".to_string()));
    assert!(out.suggestions.contains(&"Count employees".to_string()));
}

#[test]
fn test_suggestions_deduplicated_and_bounded() {
    let prompt = "random ambiguous gibberish";
    let ir = detect_intent(prompt);
    let ctx = ConversationContext::new();

    let out = generate_suggestions(prompt, &ir, &ctx);
    assert!(out.suggestions.len() <= 4);

    let mut unique = out.suggestions.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), out.suggestions.len());
}

#[test]
fn test_context_preserving_suggestions() {
    let mut ctx = ConversationContext::new();
    ctx.record_employee_lookup(
        1005,
        Some("Alice Brown".into()),
        None,
        Some("HR".into()),
        None,
        None,
        None,
        None,
    );

    let prompt = "Update employee";
    let ir = detect_intent(prompt);

    let out = generate_suggestions(prompt, &ir, &ctx);
    assert!(out.suggestions.iter().any(|s| s.contains("1005")));
}

#[test]
fn test_unknown_prompt_with_numeric_id_suggests_employee_operations() {
    let prompt = "some unfamiliar text about 1001";
    let ir = detect_intent(prompt);
    let ctx = ConversationContext::new();

    let classification = classify_prompt_error(prompt, &ir);
    assert_eq!(classification.error_type, PromptErrorType::AmbiguousIntent);
    assert_eq!(classification.detected_entity.as_deref(), Some("1001"));

    let out = generate_suggestions(prompt, &ir, &ctx);
    assert_eq!(out.error_type, PromptErrorType::AmbiguousIntent);
    assert!(out.suggestions.contains(&"Find employee 1001".to_string()));
    assert!(
        out.suggestions
            .contains(&"Show employee 1001 details".to_string())
    );
}
