// =============================================================================
// intent/extractor.rs — Parameter extraction from matched prompts
// =============================================================================

use crate::intent::patterns::*;
use crate::models::ExtractedParams;

/// Extract all recognisable parameters from a user prompt.
/// Only sets fields that it can confidently extract — never invents values.
pub fn extract_params(prompt: &str) -> ExtractedParams {
    let mut params = ExtractedParams::default();

    // Employee ID
    if let Some(caps) = RE_EXTRACT_ID.captures(prompt) {
        if let Some(id_str) = caps.name("id") {
            params.employee_id = id_str.as_str().parse::<i64>().ok();
        }
    }

    // Email (high priority — before name extraction to avoid partial matches)
    if let Some(caps) = RE_EXTRACT_EMAIL.captures(prompt) {
        if let Some(email) = caps.name("email") {
            params.email = Some(email.as_str().to_lowercase());
        }
    }

    // Full name (first + last)
    // 1. Check explicit "named/called <first> <last>"
    if let Some(caps) = regex::Regex::new(r"(?i)(?:named?|called)\s+(?P<first>[A-Za-z]+)\s+(?P<last>[A-Za-z]+)").unwrap().captures(prompt) {
        if let (Some(first), Some(last)) = (caps.name("first"), caps.name("last")) {
            params.first_name = Some(first.as_str().to_string());
            params.last_name  = Some(last.as_str().to_string());
        }
    } else if let Some(caps) = regex::Regex::new(r"(?i)(?:add|create|insert|hire)\s+(?:a\s+)?(?:new\s+)?employee\s+(?P<first>[A-Za-z]+)\s+(?P<last>[A-Za-z]+)").unwrap().captures(prompt) {
        if let (Some(first), Some(last)) = (caps.name("first"), caps.name("last")) {
            params.first_name = Some(first.as_str().to_string());
            params.last_name  = Some(last.as_str().to_string());
        }
    }

    // Department name
    if let Some(caps) = RE_EXTRACT_DEPT.captures(prompt) {
        if let Some(dept) = caps.name("dept") {
            params.department_name = Some(dept.as_str().trim().to_string());
        }
    }

    // Designation
    if let Some(caps) = RE_EXTRACT_DESIG.captures(prompt) {
        if let Some(desig) = caps.name("desig") {
            params.designation_name = Some(desig.as_str().trim().to_string());
        }
    }

    // Salary
    if let Some(caps) = RE_EXTRACT_SALARY.captures(prompt) {
        if let Some(sal) = caps.name("salary") {
            params.salary = sal.as_str().parse::<f64>().ok();
        }
    }

    // Hire date
    if let Some(caps) = RE_EXTRACT_DATE.captures(prompt) {
        if let Some(date) = caps.name("date") {
            params.hire_date = Some(date.as_str().to_string());
        }
    }

    // Status
    if let Some(caps) = RE_EXTRACT_STATUS.captures(prompt) {
        if let Some(status) = caps.name("status") {
            // Normalise "on leave" capitalisation
            let s = status.as_str().to_string();
            params.status = Some(normalise_status(&s));
        }
    }

    // Also try to extract status as a standalone keyword (e.g. "show active employees", "on leave employees")
    if params.status.is_none() {
        for status_val in &["on leave", "inactive", "active"] {
            let pattern = format!(r"(?i)\b{}\b", regex::escape(status_val));
            if let Ok(re) = regex::Regex::new(&pattern) {
                if re.is_match(prompt) {
                    params.status = Some(normalise_status(status_val));
                    break;
                }
            }
        }
    }

    // Phone
    if let Some(caps) = RE_EXTRACT_PHONE.captures(prompt) {
        if let Some(phone) = caps.name("phone") {
            params.phone = Some(phone.as_str().trim().to_string());
        }
    }

    // Field + new value for update commands (e.g. "department to IT")
    extract_update_field(prompt, &mut params);

    // Requested fields projection (e.g. "Find employee 1001 departmentName and designationId")
    params.requested_fields = extract_requested_fields(prompt);

    params
}

/// Extract field name and new value for update_employee operations
fn extract_update_field(prompt: &str, params: &mut ExtractedParams) {
    // "salary to 95000"
    if let Some(caps) = regex::Regex::new(r"(?i)salary\s+to\s+(?P<val>[0-9]+(?:\.[0-9]+)?)").unwrap().captures(prompt) {
        params.field_to_update = Some("salary".to_string());
        params.new_value = caps.name("val").map(|v| v.as_str().to_string());
        if params.salary.is_none() {
            params.salary = params.new_value.as_ref().and_then(|v| v.parse().ok());
        }
    }
    // "department to <name>"
    if let Some(caps) = regex::Regex::new(r"(?i)department\s+to\s+(?P<val>[A-Za-z][A-Za-z\s]*?)(?:\s*$)").unwrap().captures(prompt) {
        params.field_to_update = Some("department".to_string());
        let dept = caps.name("val").map(|v| v.as_str().trim().to_string());
        params.new_value = dept.clone();
        if params.department_name.is_none() {
            params.department_name = dept;
        }
    }
    // "designation to <name>"
    if let Some(caps) = regex::Regex::new(r"(?i)designation\s+to\s+(?P<val>[A-Za-z][A-Za-z\s\/]+?)(?:\s*$)").unwrap().captures(prompt) {
        params.field_to_update = Some("designation".to_string());
        let desig = caps.name("val").map(|v| v.as_str().trim().to_string());
        params.new_value = desig.clone();
        if params.designation_name.is_none() {
            params.designation_name = desig;
        }
    }
    // "status to <value>"
    if let Some(caps) = regex::Regex::new(r"(?i)status\s+to\s+(?P<val>active|inactive|on\s+leave)").unwrap().captures(prompt) {
        params.field_to_update = Some("status".to_string());
        let val = caps.name("val").map(|v| normalise_status(v.as_str()));
        params.new_value = val.clone();
        if params.status.is_none() {
            params.status = val;
        }
    }
}

pub fn normalise_status(raw: &str) -> String {
    match raw.to_lowercase().trim() {
        "active"   => "Active".to_string(),
        "inactive" => "Inactive".to_string(),
        s if s.contains("leave") => "On Leave".to_string(),
        other => {
            let mut c = other.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        }
    }
}

/// Extract specific requested fields mentioned in the prompt for selective display (projection)
pub fn extract_requested_fields(prompt: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let lower = prompt.to_lowercase();

    // Check specific fields:
    // departmentId
    if lower.contains("departmentid") || lower.contains("department id") || lower.contains("dept id") {
        fields.push("departmentId".to_string());
    }
    // departmentName (or generic department)
    if lower.contains("departmentname") || lower.contains("department name") || lower.contains("dept name") {
        fields.push("departmentName".to_string());
    } else if (lower.contains("department") || lower.contains("dept")) && !fields.contains(&"departmentId".to_string()) {
        fields.push("departmentName".to_string());
    }

    // designationId
    if lower.contains("designationid") || lower.contains("designation id") || lower.contains("desig id") {
        fields.push("designationId".to_string());
    }
    // designationName (or generic designation / title / role)
    if lower.contains("designationname") || lower.contains("designation name") || lower.contains("desig name") {
        fields.push("designationName".to_string());
    } else if (lower.contains("designation") || lower.contains("role") || lower.contains("title")) && !fields.contains(&"designationId".to_string()) {
        fields.push("designationName".to_string());
    }

    // salary
    if lower.contains("salary") || lower.contains("pay") || lower.contains("wage") || lower.contains("compensation") {
        fields.push("salary".to_string());
    }

    // email
    if lower.contains("email") || lower.contains("mail") {
        fields.push("email".to_string());
    }

    // phone
    if lower.contains("phone") || lower.contains("mobile") || lower.contains("contact") {
        fields.push("phone".to_string());
    }

    // hireDate
    if lower.contains("hiredate") || lower.contains("hire date") || lower.contains("hired date") {
        fields.push("hireDate".to_string());
    }

    // status
    if lower.contains("status") {
        fields.push("status".to_string());
    }

    // first / last name
    if lower.contains("firstname") || lower.contains("first name") {
        fields.push("firstName".to_string());
    }
    if lower.contains("lastname") || lower.contains("last name") {
        fields.push("lastName".to_string());
    }
    if (lower.contains("name") || lower.contains("full name")) 
        && !fields.contains(&"firstName".to_string()) 
        && !fields.contains(&"departmentName".to_string())
        && !fields.contains(&"designationName".to_string())
        && !lower.contains("department")
        && !lower.contains("designation")
    {
        fields.push("firstName".to_string());
        fields.push("lastName".to_string());
    }

    fields
}

