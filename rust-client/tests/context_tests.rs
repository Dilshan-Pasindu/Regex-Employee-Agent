// =============================================================================
// tests/context_tests.rs — Comprehensive unit tests for conversational context
// =============================================================================

use employee_mcp_client::context::{
    ConversationContext, ResolvedContextAction, clear_session_context, get_or_create_context,
    resolve_context, update_context,
};
use serde_json::json;

#[test]
fn test_store_and_resolve_employee_pronoun_salary() {
    let mut ctx = ConversationContext::new();
    ctx.record_employee_lookup(
        1001,
        Some("John Smith".into()),
        Some("john@example.com".into()),
        Some("IT".into()),
        Some("Software Engineer".into()),
        Some(150000.0),
        Some("Active".into()),
        Some(json!({"employeeId": 1001, "firstName": "John", "lastName": "Smith"})),
    );

    let action = resolve_context("What is his salary?", &ctx);
    match action {
        ResolvedContextAction::EmployeeFollowUp {
            employee_id,
            requested_field,
            rewritten_prompt,
        } => {
            assert_eq!(employee_id, 1001);
            assert_eq!(requested_field, "salary");
            assert!(rewritten_prompt.contains("1001"));
            assert!(rewritten_prompt.contains("salary"));
        }
        _ => panic!("Expected EmployeeFollowUp for salary"),
    }
}

#[test]
fn test_resolve_his_department() {
    let mut ctx = ConversationContext::new();
    ctx.record_employee_lookup(1002, None, None, Some("HR".into()), None, None, None, None);

    let action = resolve_context("What about his department?", &ctx);
    match action {
        ResolvedContextAction::EmployeeFollowUp {
            employee_id,
            requested_field,
            ..
        } => {
            assert_eq!(employee_id, 1002);
            assert_eq!(requested_field, "departmentName");
        }
        _ => panic!("Expected EmployeeFollowUp for department"),
    }
}

#[test]
fn test_resolve_his_email() {
    let mut ctx = ConversationContext::new();
    ctx.record_employee_lookup(1003, None, None, None, None, None, None, None);

    let action = resolve_context("Show his email too", &ctx);
    match action {
        ResolvedContextAction::EmployeeFollowUp {
            employee_id,
            requested_field,
            ..
        } => {
            assert_eq!(employee_id, 1003);
            assert_eq!(requested_field, "email");
        }
        _ => panic!("Expected EmployeeFollowUp for email"),
    }
}

#[test]
fn test_resolve_same_employee_reference() {
    let mut ctx = ConversationContext::new();
    ctx.record_employee_lookup(1004, None, None, None, None, None, None, None);

    let action = resolve_context("Update that employee salary to 160000", &ctx);
    match action {
        ResolvedContextAction::EmployeeFollowUp {
            employee_id,
            rewritten_prompt,
            ..
        } => {
            assert_eq!(employee_id, 1004);
            assert!(rewritten_prompt.contains("employee 1004"));
        }
        _ => panic!("Expected generic pronoun replacement with employee 1004"),
    }
}

#[test]
fn test_resolve_department_switch() {
    let mut ctx = ConversationContext::new();
    ctx.record_search(Some("IT".into()), None, None, None, 5);

    let action = resolve_context("What about HR?", &ctx);
    match action {
        ResolvedContextAction::DepartmentSwitch {
            department,
            rewritten_prompt,
        } => {
            assert_eq!(department, "HR");
            assert_eq!(rewritten_prompt, "Search employees in HR");
        }
        _ => panic!("Expected DepartmentSwitch to HR"),
    }
}

#[test]
fn test_resolve_in_memory_projection() {
    let mut ctx = ConversationContext::new();
    let search_data = json!([
        {"employeeId": 1001, "firstName": "Alice", "departmentName": "IT"},
        {"employeeId": 1002, "firstName": "Bob", "departmentName": "IT"}
    ]);
    ctx.record_search(Some("IT".into()), None, None, Some(search_data), 2);

    let action = resolve_context("Only show their names", &ctx);
    match action {
        ResolvedContextAction::InMemoryProjection { requested_fields } => {
            assert!(requested_fields.contains(&"firstName".to_string()));
        }
        _ => panic!("Expected InMemoryProjection for names"),
    }
}

#[test]
fn test_clear_context_lifecycle() {
    let session = "test_clear_session";
    let mut ctx = get_or_create_context(session);
    ctx.record_employee_lookup(1001, None, None, None, None, None, None, None);
    update_context(session, ctx);

    let before = get_or_create_context(session);
    assert_eq!(before.last_employee_id, Some(1001));

    clear_session_context(session);
    let after = get_or_create_context(session);
    assert_eq!(after.last_employee_id, None);
}

#[test]
fn test_isolated_sessions_do_not_leak() {
    let s1 = "user_session_1";
    let s2 = "user_session_2";

    let mut ctx1 = get_or_create_context(s1);
    ctx1.record_employee_lookup(1001, None, None, None, None, None, None, None);
    update_context(s1, ctx1);

    let mut ctx2 = get_or_create_context(s2);
    ctx2.record_employee_lookup(2002, None, None, None, None, None, None, None);
    update_context(s2, ctx2);

    assert_eq!(get_or_create_context(s1).last_employee_id, Some(1001));
    assert_eq!(get_or_create_context(s2).last_employee_id, Some(2002));
}

#[test]
fn test_failure_preserves_valid_context() {
    let mut ctx = ConversationContext::new();
    ctx.record_employee_lookup(
        1001,
        Some("John".into()),
        None,
        Some("IT".into()),
        None,
        None,
        None,
        None,
    );

    // Record a failure on a subsequent query
    ctx.record_failure();

    // Valid employee ID and department should still be preserved
    assert_eq!(ctx.last_employee_id, Some(1001));
    assert_eq!(ctx.last_department.as_deref(), Some("IT"));
    assert_eq!(ctx.last_operation_success, Some(false));
}
