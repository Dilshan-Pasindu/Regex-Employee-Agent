// =============================================================================
// tests/planner_tests.rs — Comprehensive unit tests for task planning
// =============================================================================

use employee_mcp_client::intent::detector::detect_intent;
use employee_mcp_client::models::{McpTool, ToolInputSchema};
use employee_mcp_client::planner::{
    ComparisonOperator, create_execution_plan, extract_salary_comparison, project_fields,
    validate_plan,
};
use serde_json::json;

fn mock_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "get_all_employees".to_string(),
            description: None,
            input_schema: ToolInputSchema {
                schema_type: Some("object".into()),
                properties: None,
                required: None,
            },
        },
        McpTool {
            name: "get_employee_by_id".to_string(),
            description: None,
            input_schema: ToolInputSchema {
                schema_type: Some("object".into()),
                properties: None,
                required: Some(vec!["employeeId".into()]),
            },
        },
        McpTool {
            name: "search_employees".to_string(),
            description: None,
            input_schema: ToolInputSchema {
                schema_type: Some("object".into()),
                properties: None,
                required: None,
            },
        },
        McpTool {
            name: "add_employee".to_string(),
            description: None,
            input_schema: ToolInputSchema {
                schema_type: Some("object".into()),
                properties: None,
                required: Some(vec!["firstName".into(), "lastName".into(), "email".into()]),
            },
        },
        McpTool {
            name: "update_employee".to_string(),
            description: None,
            input_schema: ToolInputSchema {
                schema_type: Some("object".into()),
                properties: None,
                required: Some(vec!["employeeId".into()]),
            },
        },
    ]
}

#[test]
fn test_simple_employee_lookup_plan() {
    let prompt = "Find employee 1001";
    let ir = detect_intent(prompt);
    let plan = create_execution_plan(prompt, &ir);

    assert_eq!(plan.intent_label, "get_employee_by_id");
    assert_eq!(plan.extracted_conditions.employee_id, Some(1001));
    assert_eq!(plan.steps.len(), 2);
    assert_eq!(
        plan.steps[1].tool_name.as_deref(),
        Some("get_employee_by_id")
    );
    assert!(!plan.requires_confirmation);

    let tools = mock_tools();
    let validation = validate_plan(&plan, &ir, &tools);
    assert!(validation.is_valid);
}

#[test]
fn test_salary_comparison_extraction() {
    let (op, val) = extract_salary_comparison("earning more than 150000").unwrap();
    assert_eq!(op, ComparisonOperator::GreaterThan);
    assert!((val - 150000.0).abs() < f64::EPSILON);

    let (op2, val2) = extract_salary_comparison("salary under 90000").unwrap();
    assert_eq!(op2, ComparisonOperator::LessThan);
    assert!((val2 - 90000.0).abs() < f64::EPSILON);

    let (op3, val3) = extract_salary_comparison("salary equal to 125000").unwrap();
    assert_eq!(op3, ComparisonOperator::Equal);
    assert!((val3 - 125000.0).abs() < f64::EPSILON);
}

#[test]
fn test_complex_employee_search_plan() {
    let prompt =
        "Find all employees in IT earning more than 150000 and show their names and salaries";
    let ir = detect_intent(prompt);
    let plan = create_execution_plan(prompt, &ir);

    assert_eq!(plan.intent_label, "search_employees");
    assert_eq!(plan.extracted_conditions.department.as_deref(), Some("IT"));
    assert_eq!(
        plan.extracted_conditions.salary_operator,
        Some(ComparisonOperator::GreaterThan)
    );
    assert_eq!(plan.extracted_conditions.salary, Some(150000.0));
    assert!(plan.requested_fields.contains(&"firstName".to_string()));
    assert!(plan.requested_fields.contains(&"salary".to_string()));

    // Verify multi-step plan has: extraction step, tool step, post-filter step, projection step
    assert!(plan.steps.len() >= 3);
    assert!(
        plan.steps
            .iter()
            .any(|s| s.description.contains("search_employees"))
    );
    assert!(
        plan.steps
            .iter()
            .any(|s| s.description.contains("post-filtering"))
    );
    assert!(
        plan.steps
            .iter()
            .any(|s| s.description.contains("Project requested display fields"))
    );
}

#[test]
fn test_field_projection_preserves_order_and_id() {
    let raw_data = json!({
        "employeeId": 1001,
        "firstName": "John",
        "lastName": "Smith",
        "email": "john@example.com",
        "salary": 180000.0,
        "departmentName": "IT"
    });

    let requested = vec!["salary".to_string(), "departmentName".to_string()];
    let projected = project_fields(&raw_data, &requested);

    assert_eq!(
        projected.get("employeeId").and_then(|v| v.as_i64()),
        Some(1001)
    );
    assert_eq!(
        projected.get("salary").and_then(|v| v.as_f64()),
        Some(180000.0)
    );
    assert_eq!(
        projected.get("departmentName").and_then(|v| v.as_str()),
        Some("IT")
    );
    assert!(projected.get("email").is_none());
    assert!(projected.get("firstName").is_none());
}

#[test]
fn test_plan_validation_catches_negative_salary() {
    let prompt = "Add employee Jane Doe email jane@example.com salary -5000";
    let ir = detect_intent(prompt);
    let plan = create_execution_plan(prompt, &ir);
    let tools = mock_tools();

    let validation = validate_plan(&plan, &ir, &tools);
    assert!(!validation.is_valid);
    assert!(validation.errors.iter().any(|e| e.contains("negative")));
}

#[test]
fn test_write_operation_confirmation_flags() {
    let prompt_add = "Add employee Jane Doe email jane@example.com";
    let ir_add = detect_intent(prompt_add);
    let plan_add = create_execution_plan(prompt_add, &ir_add);
    assert!(plan_add.requires_confirmation);

    let prompt_update = "Update employee 1001 salary to 180000";
    let ir_update = detect_intent(prompt_update);
    let plan_update = create_execution_plan(prompt_update, &ir_update);
    assert!(plan_update.requires_confirmation);

    let prompt_read = "Show all employees";
    let ir_read = detect_intent(prompt_read);
    let plan_read = create_execution_plan(prompt_read, &ir_read);
    assert!(!plan_read.requires_confirmation);
}

#[test]
fn test_update_employee_plan_arguments() {
    let prompt = "Update employee 1002 salary to 95000";
    let ir = detect_intent(prompt);
    let plan = create_execution_plan(prompt, &ir);

    assert_eq!(plan.intent_label, "update_employee");
    assert!(plan.requires_confirmation);
    assert!(plan.requested_fields.is_empty()); // Updates should not be projected/truncated

    let tool_step = &plan.steps[1];
    assert_eq!(tool_step.tool_name.as_deref(), Some("update_employee"));

    let args = tool_step
        .arguments
        .as_ref()
        .expect("arguments must be present");
    assert_eq!(args.get("employeeId").and_then(|v| v.as_i64()), Some(1002));
    assert_eq!(args.get("salary").and_then(|v| v.as_f64()), Some(95000.0));
}

#[test]
fn test_update_employee_apostrophe_syntax() {
    let prompt = "Update employee 1001's salary to 180000";
    let ir = detect_intent(prompt);
    let plan = create_execution_plan(prompt, &ir);

    assert_eq!(plan.intent_label, "update_employee");
    let args = plan.steps[1]
        .arguments
        .as_ref()
        .expect("arguments must be present");
    assert_eq!(args.get("employeeId").and_then(|v| v.as_i64()), Some(1001));
    assert_eq!(args.get("salary").and_then(|v| v.as_f64()), Some(180000.0));
}
