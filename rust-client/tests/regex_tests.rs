// =============================================================================
// tests/regex_tests.rs — Comprehensive Test Suite for Regex Intent Engine
// 40+ Deterministic Test Cases (No LLM, Pure Regex Routing)
// Covers: Capitalization, phrasing variations, parameter extraction,
//         selective projection, precedence, edge cases, and safety confirmation.
// =============================================================================

use employee_mcp_client::{
    intent::detector::detect_intent,
    models::Intent,
};

// -----------------------------------------------------------------------------
// Category 1: GetAllEmployees (Case variations & Alternate phrasing)
// -----------------------------------------------------------------------------

#[test]
fn test_01_get_all_employees_standard() {
    let result = detect_intent("Show all employees");
    assert_eq!(result.intent, Intent::GetAllEmployees);
}

#[test]
fn test_02_get_all_employees_lowercase() {
    let result = detect_intent("list employees");
    assert_eq!(result.intent, Intent::GetAllEmployees);
}

#[test]
fn test_03_get_all_employees_uppercase() {
    let result = detect_intent("DISPLAY ALL EMPLOYEES");
    assert_eq!(result.intent, Intent::GetAllEmployees);
}

#[test]
fn test_04_get_all_employees_mixed_case() {
    let result = detect_intent("sHoW aLl eMpLoYeEs");
    assert_eq!(result.intent, Intent::GetAllEmployees);
}

#[test]
fn test_05_get_all_employees_alternate_staff() {
    let result = detect_intent("view all staff");
    assert_eq!(result.intent, Intent::GetAllEmployees);
}

#[test]
fn test_06_get_all_employees_records_phrasing() {
    let result = detect_intent("employee records");
    assert_eq!(result.intent, Intent::GetAllEmployees);
}

#[test]
fn test_06b_get_all_employees_with_details_keyword() {
    assert_eq!(detect_intent("show all employees details").intent, Intent::GetAllEmployees);
    assert_eq!(detect_intent("show all employee details").intent, Intent::GetAllEmployees);
    assert_eq!(detect_intent("show employees details").intent, Intent::GetAllEmployees);
    assert_eq!(detect_intent("list all employees info").intent, Intent::GetAllEmployees);
    assert_eq!(detect_intent("display all employees data").intent, Intent::GetAllEmployees);
}

// -----------------------------------------------------------------------------
// Category 2: GetEmployeeById (Extraction & Formatting)
// -----------------------------------------------------------------------------

#[test]
fn test_07_find_employee_by_id_simple() {
    let result = detect_intent("Find employee 1001");
    assert_eq!(result.intent, Intent::GetEmployeeById);
    assert_eq!(result.params.employee_id, Some(1001));
    assert!(result.params.requested_fields.is_empty());
}

#[test]
fn test_08_find_employee_by_id_with_label() {
    let result = detect_intent("Show employee ID 1002");
    assert_eq!(result.intent, Intent::GetEmployeeById);
    assert_eq!(result.params.employee_id, Some(1002));
}

#[test]
fn test_09_find_employee_by_id_hash_syntax() {
    let result = detect_intent("lookup employee #1005");
    assert_eq!(result.intent, Intent::GetEmployeeById);
    assert_eq!(result.params.employee_id, Some(1005));
}

#[test]
fn test_10_find_employee_by_id_colon_syntax() {
    let result = detect_intent("get employee id: 1010");
    assert_eq!(result.intent, Intent::GetEmployeeById);
    assert_eq!(result.params.employee_id, Some(1010));
}

// -----------------------------------------------------------------------------
// Category 3: Selective Field Projection on GetEmployeeById
// -----------------------------------------------------------------------------

#[test]
fn test_11_projection_department_and_designation() {
    let result = detect_intent("Find employee 1001 departmentName and designationId");
    assert_eq!(result.intent, Intent::GetEmployeeById);
    assert_eq!(result.params.employee_id, Some(1001));
    assert!(result.params.requested_fields.contains(&"departmentName".to_string()));
    assert!(result.params.requested_fields.contains(&"designationId".to_string()));
}

#[test]
fn test_12_projection_salary_and_email() {
    let result = detect_intent("Show employee 1002 salary and email");
    assert_eq!(result.intent, Intent::GetEmployeeById);
    assert_eq!(result.params.employee_id, Some(1002));
    assert!(result.params.requested_fields.contains(&"salary".to_string()));
    assert!(result.params.requested_fields.contains(&"email".to_string()));
}

#[test]
fn test_13_projection_status_only() {
    let result = detect_intent("Find employee 1003 status");
    assert_eq!(result.intent, Intent::GetEmployeeById);
    assert_eq!(result.params.employee_id, Some(1003));
    assert!(result.params.requested_fields.contains(&"status".to_string()));
}

#[test]
fn test_14_projection_hire_date() {
    let result = detect_intent("Get employee 1004 hire date");
    assert_eq!(result.intent, Intent::GetEmployeeById);
    assert_eq!(result.params.employee_id, Some(1004));
    assert!(result.params.requested_fields.contains(&"hireDate".to_string()));
}

// -----------------------------------------------------------------------------
// Category 4: SearchEmployees (Department, Designation, Name, Status)
// -----------------------------------------------------------------------------

#[test]
fn test_15_search_by_department() {
    let result = detect_intent("Search employees in IT department");
    assert_eq!(result.intent, Intent::SearchEmployees);
    assert_eq!(result.params.department_name.as_deref(), Some("IT"));
}

#[test]
fn test_16_search_by_department_short() {
    let result = detect_intent("find employees from HR");
    assert_eq!(result.intent, Intent::SearchEmployees);
    assert_eq!(result.params.department_name.as_deref(), Some("HR"));
}

#[test]
fn test_17_search_by_department_engineering() {
    let result = detect_intent("show employees in Engineering");
    assert_eq!(result.intent, Intent::SearchEmployees);
    assert_eq!(result.params.department_name.as_deref(), Some("Engineering"));
}

#[test]
fn test_18_search_by_name() {
    let result = detect_intent("find employees named John Smith");
    assert_eq!(result.intent, Intent::SearchEmployees);
    assert_eq!(result.params.first_name.as_deref(), Some("John"));
    assert_eq!(result.params.last_name.as_deref(), Some("Smith"));
}

#[test]
fn test_19_search_by_single_name() {
    let result = detect_intent("search for Nicole");
    assert_eq!(result.intent, Intent::SearchEmployees);
}

#[test]
fn test_20_search_active_status() {
    let result = detect_intent("show active employees");
    assert_eq!(result.intent, Intent::SearchEmployees);
    assert_eq!(result.params.status.as_deref(), Some("Active"));
}

#[test]
fn test_21_search_on_leave_status() {
    let result = detect_intent("list on leave employees");
    assert_eq!(result.intent, Intent::SearchEmployees);
    assert_eq!(result.params.status.as_deref(), Some("On Leave"));
}

// -----------------------------------------------------------------------------
// Category 5: GetEmployeeCount
// -----------------------------------------------------------------------------

#[test]
fn test_22_count_general() {
    let result = detect_intent("How many employees are there?");
    assert_eq!(result.intent, Intent::GetEmployeeCount);
}

#[test]
fn test_23_count_uppercase() {
    let result = detect_intent("HOW MANY EMPLOYEES");
    assert_eq!(result.intent, Intent::GetEmployeeCount);
}

#[test]
fn test_24_count_alternate_phrasing() {
    let result = detect_intent("total number of employees");
    assert_eq!(result.intent, Intent::GetEmployeeCount);
}

#[test]
fn test_25_count_by_department() {
    let result = detect_intent("count employees in IT department");
    assert_eq!(result.intent, Intent::GetEmployeeCount);
    assert_eq!(result.params.department_name.as_deref(), Some("IT"));
}

#[test]
fn test_26_count_by_department_hr() {
    let result = detect_intent("how many employees in HR");
    assert_eq!(result.intent, Intent::GetEmployeeCount);
    assert_eq!(result.params.department_name.as_deref(), Some("HR"));
}

#[test]
fn test_26b_count_active_employees() {
    let result = detect_intent("Count active employees");
    assert_eq!(result.intent, Intent::GetEmployeeCount);
    assert_eq!(result.params.status.as_deref(), Some("Active"));
}

#[test]
fn test_26c_count_inactive_employees_in_dept() {
    let result = detect_intent("Count inactive employees in IT");
    assert_eq!(result.intent, Intent::GetEmployeeCount);
    assert_eq!(result.params.status.as_deref(), Some("Inactive"));
    assert_eq!(result.params.department_name.as_deref(), Some("IT"));
}

#[test]
fn test_26d_count_on_leave_staff() {
    let result = detect_intent("how many on leave staff");
    assert_eq!(result.intent, Intent::GetEmployeeCount);
    assert_eq!(result.params.status.as_deref(), Some("On Leave"));
}

// -----------------------------------------------------------------------------
// Category 6: GetDepartmentList
// -----------------------------------------------------------------------------

#[test]
fn test_27_get_departments_standard() {
    let result = detect_intent("Show all departments");
    assert_eq!(result.intent, Intent::GetDepartmentList);
}

#[test]
fn test_28_get_departments_short() {
    let result = detect_intent("list departments");
    assert_eq!(result.intent, Intent::GetDepartmentList);
}

#[test]
fn test_29_get_departments_uppercase() {
    let result = detect_intent("DEPARTMENTS LIST");
    assert_eq!(result.intent, Intent::GetDepartmentList);
}

// -----------------------------------------------------------------------------
// Category 7: AddEmployee (Parameter Extraction)
// -----------------------------------------------------------------------------

#[test]
fn test_30_add_employee_simple() {
    let result = detect_intent("Add a new employee named John Smith");
    assert_eq!(result.intent, Intent::AddEmployee);
    assert_eq!(result.params.first_name.as_deref(), Some("John"));
    assert_eq!(result.params.last_name.as_deref(), Some("Smith"));
}

#[test]
fn test_31_add_employee_with_email_and_salary() {
    let result = detect_intent("create employee Alice Cooper email alice@example.com salary 95000");
    assert_eq!(result.intent, Intent::AddEmployee);
    assert_eq!(result.params.first_name.as_deref(), Some("Alice"));
    assert_eq!(result.params.last_name.as_deref(), Some("Cooper"));
    assert_eq!(result.params.email.as_deref(), Some("alice@example.com"));
    assert_eq!(result.params.salary, Some(95000.0));
}

// -----------------------------------------------------------------------------
// Category 8: UpdateEmployee
// -----------------------------------------------------------------------------

#[test]
fn test_32_update_department() {
    let result = detect_intent("Update employee 1001 department to Engineering");
    assert_eq!(result.intent, Intent::UpdateEmployee);
    assert_eq!(result.params.employee_id, Some(1001));
    assert_eq!(result.params.field_to_update.as_deref(), Some("department"));
    assert_eq!(result.params.department_name.as_deref(), Some("Engineering"));
}

#[test]
fn test_33_update_status() {
    let result = detect_intent("change employee 1002 status to Inactive");
    assert_eq!(result.intent, Intent::UpdateEmployee);
    assert_eq!(result.params.employee_id, Some(1002));
    assert_eq!(result.params.field_to_update.as_deref(), Some("status"));
    assert_eq!(result.params.status.as_deref(), Some("Inactive"));
}

#[test]
fn test_34_update_salary() {
    let result = detect_intent("update employee 1003 salary to 95000");
    assert_eq!(result.intent, Intent::UpdateEmployee);
    assert_eq!(result.params.employee_id, Some(1003));
    assert_eq!(result.params.salary, Some(95000.0));
}

// -----------------------------------------------------------------------------
// Category 9: Intent Precedence & Disambiguation
// -----------------------------------------------------------------------------

#[test]
fn test_35_precedence_id_over_list() {
    // "Show employee 1001" must resolve to GetEmployeeById, NOT GetAllEmployees
    let result = detect_intent("Show employee 1001");
    assert_eq!(result.intent, Intent::GetEmployeeById);
    assert_eq!(result.params.employee_id, Some(1001));
}

#[test]
fn test_36_precedence_update_over_search() {
    // "Update employee 1001 department to IT" must NOT be treated as search employees in IT
    let result = detect_intent("Update employee 1001 department to IT");
    assert_eq!(result.intent, Intent::UpdateEmployee);
}

#[test]
fn test_37_precedence_count_over_search() {
    // "Count employees in IT" must resolve to count, not search
    let result = detect_intent("Count employees in IT");
    assert_eq!(result.intent, Intent::GetEmployeeCount);
}

// -----------------------------------------------------------------------------
// Category 10: Unsupported, Ambiguous & Malicious Commands
// -----------------------------------------------------------------------------

#[test]
fn test_38_unsupported_weather_command() {
    let result = detect_intent("What is the weather today?");
    assert!(matches!(result.intent, Intent::Unknown(_)));
}

#[test]
fn test_39_unsupported_random_text() {
    let result = detect_intent("Order a pizza for lunch");
    assert!(matches!(result.intent, Intent::Unknown(_)));
}

#[test]
fn test_40_sql_injection_rejected() {
    // Arbitrary SQL injection attempts must NOT execute any tool
    let result = detect_intent("DROP TABLE Employees; SELECT * FROM Users;");
    assert!(matches!(result.intent, Intent::Unknown(_)));
}

#[test]
fn test_41_empty_string_rejected() {
    let result = detect_intent("   ");
    assert!(matches!(result.intent, Intent::Unknown(_)));
}

// -----------------------------------------------------------------------------
// Category 11: Safety Confirmation Gate Verification
// -----------------------------------------------------------------------------

#[test]
fn test_42_write_operations_require_confirmation() {
    assert!(Intent::AddEmployee.requires_confirmation(), "AddEmployee MUST require confirmation");
    assert!(Intent::UpdateEmployee.requires_confirmation(), "UpdateEmployee MUST require confirmation");
}

#[test]
fn test_43_read_operations_do_not_require_confirmation() {
    assert!(!Intent::GetAllEmployees.requires_confirmation(), "GetAllEmployees must be read-only");
    assert!(!Intent::GetEmployeeById.requires_confirmation(), "GetEmployeeById must be read-only");
    assert!(!Intent::SearchEmployees.requires_confirmation(), "SearchEmployees must be read-only");
    assert!(!Intent::GetEmployeeCount.requires_confirmation(), "GetEmployeeCount must be read-only");
    assert!(!Intent::GetDepartmentList.requires_confirmation(), "GetDepartmentList must be read-only");
}

// -----------------------------------------------------------------------------
// Category 12: External Vocabulary Customization Verification
// -----------------------------------------------------------------------------

#[test]
fn test_44_vocabulary_driven_field_aliases() {
    let result = detect_intent("find employee 1001 division and wage");
    assert_eq!(result.intent, Intent::GetEmployeeById);
    assert_eq!(result.params.employee_id, Some(1001));
    assert!(result.params.requested_fields.contains(&"departmentName".to_string()));
    assert!(result.params.requested_fields.contains(&"salary".to_string()));
}

#[test]
fn test_45_vocabulary_status_mapping() {
    let vocab = employee_mcp_client::intent::vocabulary::Vocabulary::get();
    assert_eq!(vocab.normalise_status("working"), "Active");
    assert_eq!(vocab.normalise_status("resigned"), "Inactive");
    assert_eq!(vocab.normalise_status("vacation"), "On Leave");
}
