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
        let vocab = crate::intent::vocabulary::Vocabulary::get();
        for status_val in vocab.status_mappings.keys() {
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
    // "salary to 95000" or "'s salary to 95000"
    if let Some(caps) =
        regex::Regex::new(r"(?i)(?:'s\s+)?salary\s+to\s+(?P<val>[0-9]+(?:\.[0-9]+)?)")
            .unwrap()
            .captures(prompt)
    {
        params.field_to_update = Some("salary".to_string());
        params.new_value = caps.name("val").map(|v| v.as_str().to_string());
        if params.salary.is_none() {
            params.salary = params.new_value.as_ref().and_then(|v| v.parse().ok());
        }
    }
    // "department to <name or id>"
    if let Some(caps) = regex::Regex::new(
        r"(?i)(?:'s\s+)?department\s+to\s+(?P<val>[A-Za-z0-9][A-Za-z0-9\s]*?)(?:\s*$)",
    )
    .unwrap()
    .captures(prompt)
    {
        params.field_to_update = Some("department".to_string());
        let dept = caps.name("val").map(|v| v.as_str().trim().to_string());
        params.new_value = dept.clone();
        if params.department_name.is_none() {
            params.department_name = dept;
        }
    }
    // "designation to <name or id>"
    if let Some(caps) = regex::Regex::new(
        r"(?i)(?:'s\s+)?designation\s+to\s+(?P<val>[A-Za-z0-9][A-Za-z0-9\s\/]+?)(?:\s*$)",
    )
    .unwrap()
    .captures(prompt)
    {
        params.field_to_update = Some("designation".to_string());
        let desig = caps.name("val").map(|v| v.as_str().trim().to_string());
        params.new_value = desig.clone();
        if params.designation_name.is_none() {
            params.designation_name = desig;
        }
    }
    // "status to <value>"
    if let Some(caps) =
        regex::Regex::new(r"(?i)(?:'s\s+)?status\s+to\s+(?P<val>active|inactive|on\s+leave)")
            .unwrap()
            .captures(prompt)
    {
        params.field_to_update = Some("status".to_string());
        let val = caps.name("val").map(|v| normalise_status(v.as_str()));
        params.new_value = val.clone();
        if params.status.is_none() {
            params.status = val;
        }
    }
    // "email to <email>"
    if let Some(caps) = regex::Regex::new(
        r"(?i)(?:'s\s+)?email\s+to\s+(?P<val>[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,})",
    )
    .unwrap()
    .captures(prompt)
    {
        params.field_to_update = Some("email".to_string());
        let val = caps.name("val").map(|v| v.as_str().to_lowercase());
        params.new_value = val.clone();
        if params.email.is_none() {
            params.email = val;
        }
    }
    // "phone to <number>"
    if let Some(caps) = regex::Regex::new(
        r"(?i)(?:'s\s+)?(?:phone|mobile|contact)\s+to\s+(?P<val>[0-9+\-\s]{7,20})",
    )
    .unwrap()
    .captures(prompt)
    {
        params.field_to_update = Some("phone".to_string());
        let val = caps.name("val").map(|v| v.as_str().trim().to_string());
        params.new_value = val.clone();
        if params.phone.is_none() {
            params.phone = val;
        }
    }
}

pub fn normalise_status(raw: &str) -> String {
    crate::intent::vocabulary::Vocabulary::get().normalise_status(raw)
}

/// Extract specific requested fields mentioned in the prompt for selective display (projection)
/// Uses field_aliases configured in config/vocabulary.yaml
pub fn extract_requested_fields(prompt: &str) -> Vec<String> {
    crate::intent::vocabulary::Vocabulary::get().match_requested_fields(prompt)
}
