// =============================================================================
// intent/patterns.rs — Dynamic Regex compilation powered by vocabulary.yaml
// No LLM, no AI. Pure Regex compiled dynamically from external vocabulary.
// =============================================================================

use crate::intent::vocabulary::Vocabulary;
use once_cell::sync::Lazy;
use regex::Regex;

// ---------------------------------------------------------------------------
// get_all_employees patterns
// Matches: "show all employees", "show all employees details", "list employees", etc.
// ---------------------------------------------------------------------------
pub static RE_GET_ALL_EMPLOYEES: Lazy<Vec<Regex>> = Lazy::new(|| {
    let v = &Vocabulary::get().intents.get_all_employees;
    let verbs = Vocabulary::alternation(&v.verbs);
    let targets = Vocabulary::alternation(&v.targets);
    let quantifiers = Vocabulary::alternation(&v.quantifiers);
    let suffixes = Vocabulary::alternation(&v.suffixes);
    let standalone = Vocabulary::alternation(&v.standalone_phrases);

    vec![
        Regex::new(&format!(r"(?i)^({verbs})\s+(?:({quantifiers})\s+)?({targets})(?:\s+({suffixes}))?$")).unwrap(),
        Regex::new(&format!(r"(?i)^({verbs})\s+(?:the\s+)?({targets})\s+({suffixes})$")).unwrap(),
        Regex::new(&format!(r"(?i)^({verbs})\s+all\s+staff(?:\s+({suffixes}))?$")).unwrap(),
        Regex::new(&format!(r"(?i)^({standalone})$")).unwrap(),
        Regex::new(&format!(r"(?i)^(give\s+me\s+|show\s+me\s+)?(?:({quantifiers})\s+)?({targets})(?:\s+({suffixes}))?$")).unwrap(),
    ]
});

// ---------------------------------------------------------------------------
// get_employee_by_id patterns
// Matches: "find employee 1001", "show employee ID 1002", etc.
// ---------------------------------------------------------------------------
pub static RE_GET_BY_ID: Lazy<Vec<Regex>> = Lazy::new(|| {
    let v = &Vocabulary::get().intents.get_employee_by_id;
    let verbs = Vocabulary::alternation(&v.verbs);

    vec![
        Regex::new(&format!(
            r"(?i)({verbs})\s+employee\s+(?:id\s*[:#]?\s*|#\s*)?(?P<id>\d{{4,}})"
        ))
        .unwrap(),
        Regex::new(r"(?i)employee\s+(?:id\s*[:#]?\s*|#\s*)?(?P<id>\d{4,})").unwrap(),
        Regex::new(&format!(
            r"(?i)({verbs})\s+(?:the\s+)?(?:employee\s+)?(?:id\s*[:#]?\s*|#\s*)?(?P<id>\d{{4,}})(?:\s+(?:details?|records?|data|info(?:rmation)?))?"
        ))
        .unwrap(),
    ]
});

// ---------------------------------------------------------------------------
// search_employees patterns
// Matches: "search employees in IT", "find employees from HR", etc.
// ---------------------------------------------------------------------------
pub static RE_SEARCH_EMPLOYEES: Lazy<Vec<Regex>> = Lazy::new(|| {
    let v = &Vocabulary::get().intents.search_employees;
    let verbs = Vocabulary::alternation(&v.verbs);
    let preps = Vocabulary::alternation(&v.prepositions);
    let name_ind = Vocabulary::alternation(&v.name_indicators);
    let desig_ind = Vocabulary::alternation(&v.designation_indicators);
    let statuses = Vocabulary::alternation(&v.statuses);

    vec![
        // By department
        Regex::new(&format!(r"(?i)({verbs})\s+(?:all\s+)?employees?\s+({preps})\s+(?:the\s+)?(?P<dept>[A-Za-z][A-Za-z\s]*?)(?:\s+department)?(?:\s+(?:earning|salary|making|with|who|and)\b.*)?$")).unwrap(),
        Regex::new(r"(?i)(?P<dept>[A-Za-z]+)\s+department\s+employees?").unwrap(),
        // By name
        Regex::new(&format!(r"(?i)({verbs})\s+(employees?|staff)\s+({name_ind})\s+(?P<name>[A-Za-z][A-Za-z\s]+)$")).unwrap(),
        // By designation
        Regex::new(&format!(r"(?i)({verbs})\s+(employees?|staff)\s+({desig_ind})\s+(?P<desig>[A-Za-z][A-Za-z\s]+)$")).unwrap(),
        // By status
        Regex::new(&format!(r"(?i)({verbs})\s+(all\s+)?(?P<status>{statuses})\s+employees?")).unwrap(),
        // General search for a name
        Regex::new(r"(?i)(find|search)\s+(?P<name>[A-Za-z]+(?:\s+[A-Za-z]+)?)\s+in\s+employees?").unwrap(),
        // "search for John"
        Regex::new(r"(?i)^(?:find|search|look\s*up)\s+for\s+(?P<name>[A-Za-z][A-Za-z\s]+)$").unwrap(),
    ]
});

// ---------------------------------------------------------------------------
// Error & Incomplete Prompt Patterns
// ---------------------------------------------------------------------------
pub static RE_INVALID_ID: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)\b(?:find|show|get|lookup)\s+employee\s+(?:id\s*[:#]?\s*)?(?P<id>[a-zA-Z]{1,15})\b",
    )
    .unwrap()
});

pub static RE_BARE_FIND_EMPLOYEE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^(find|show|get|search|lookup)\s+(?:an?\s+)?employee\s*(\?|\.)?$").unwrap()
});

pub fn is_invalid_employee_id(prompt: &str) -> bool {
    // If the prompt contains a valid numeric employee ID, it is not an invalid ID query
    if RE_EXTRACT_ID.is_match(prompt) {
        return false;
    }

    if let Some(caps) = RE_INVALID_ID.captures(prompt) {
        if let Some(id_match) = caps.name("id") {
            let val = id_match.as_str().to_lowercase();
            if val != "id"
                && val != "details"
                && val != "all"
                && val != "list"
                && val != "count"
                && val != "records"
                && val != "record"
                && val != "data"
                && val != "info"
            {
                return true;
            }
        }
    }
    false
}

pub fn is_bare_employee_lookup(prompt: &str) -> bool {
    RE_BARE_FIND_EMPLOYEE.is_match(prompt)
}

// ---------------------------------------------------------------------------
// get_department_list patterns
// ---------------------------------------------------------------------------
pub static RE_GET_DEPARTMENTS: Lazy<Vec<Regex>> = Lazy::new(|| {
    let v = &Vocabulary::get().intents.get_department_list;
    let verbs = Vocabulary::alternation(&v.verbs);
    let targets = Vocabulary::alternation(&v.targets);
    let standalone = Vocabulary::alternation(&v.standalone_phrases);

    vec![
        Regex::new(&format!(r"(?i)^({verbs})\s+(all\s+)?({targets})$")).unwrap(),
        Regex::new(&format!(r"(?i)^({standalone})$")).unwrap(),
        Regex::new(&format!(
            r"(?i)^(give\s+me\s+|show\s+me\s+)?(all\s+)?({targets})$"
        ))
        .unwrap(),
    ]
});

// ---------------------------------------------------------------------------
// get_employee_count patterns
// ---------------------------------------------------------------------------
pub static RE_GET_COUNT: Lazy<Vec<Regex>> = Lazy::new(|| {
    let vocab = Vocabulary::get();
    let v = &vocab.intents.get_employee_count;
    let triggers = Vocabulary::alternation(&v.count_triggers);
    let targets = Vocabulary::alternation(&v.targets);
    let standalone = Vocabulary::alternation(&v.standalone_phrases);
    let statuses = Vocabulary::alternation(&vocab.intents.search_employees.statuses);

    vec![
        // "count active employees", "how many on leave staff", "count employees"
        Regex::new(&format!(r"(?i)({triggers})\s+(?:all\s+)?(?:({statuses})\s+)?({targets})")).unwrap(),
        // "count employees who are active", "how many employees are active"
        Regex::new(&format!(r"(?i)({triggers})\s+({targets})\s+(?:who\s+are\s+|that\s+are\s+|are\s+|with\s+status\s+)?({statuses})")).unwrap(),
        // "active employee count"
        Regex::new(&format!(r"(?i)^({statuses})\s+({standalone})$")).unwrap(),
        Regex::new(&format!(r"(?i)^({standalone})$")).unwrap(),
        Regex::new(r"(?i)total\s+(number\s+of\s+)?employees?").unwrap(),
        Regex::new(r"(?i)count\s+(?:all\s+)?(?:[A-Za-z\s]+\s+)?employees?\s+(in|from|of)\s+(?P<dept>[A-Za-z][A-Za-z\s]*?)(\s+department)?$").unwrap(),
        Regex::new(r"(?i)how\s+many\s+(?:all\s+)?(?:[A-Za-z\s]+\s+)?employees?\s+(are\s+)?(in|from|of)\s+(?P<dept>[A-Za-z][A-Za-z\s]*?)(\s+department)?$").unwrap(),
    ]
});

// ---------------------------------------------------------------------------
// add_employee patterns
// ---------------------------------------------------------------------------
pub static RE_ADD_EMPLOYEE: Lazy<Vec<Regex>> = Lazy::new(|| {
    let v = &Vocabulary::get().intents.add_employee;
    let verbs = Vocabulary::alternation(&v.verbs);

    vec![
        Regex::new(&format!(r"(?i)^({verbs})\s+(an?\s+)?(new\s+)?employee")).unwrap(),
        Regex::new(&format!(
            r"(?i)^({verbs})\s+(?P<first>[A-Za-z]+)\s+(?P<last>[A-Za-z]+)"
        ))
        .unwrap(),
        Regex::new(r"(?i)^new\s+employee\s+(?:named?\s+)?(?P<first>[A-Za-z]+)").unwrap(),
    ]
});

// ---------------------------------------------------------------------------
// update_employee patterns
// ---------------------------------------------------------------------------
pub static RE_UPDATE_EMPLOYEE: Lazy<Vec<Regex>> = Lazy::new(|| {
    let v = &Vocabulary::get().intents.update_employee;
    let verbs = Vocabulary::alternation(&v.verbs);
    let assign = Vocabulary::alternation(&v.assignment_words);

    vec![
        Regex::new(&format!(r"(?i)({verbs})\s+employee\s+(?:id\s*[:#]?\s*)?(?P<id>\d{{4,}})\s+(?P<field>department|designation|salary|status|phone|email|first\s*name|last\s*name|hire\s*date|name)\s+({assign})\s+(?P<val>.+)$")).unwrap(),
        Regex::new(&format!(r"(?i)({verbs})\s+(?:employee\s+)?(?:id\s*[:#]?\s*)?(?P<id>\d{{4,}})")).unwrap(),
    ]
});

// ---------------------------------------------------------------------------
// Static Parameter extraction patterns
// ---------------------------------------------------------------------------

/// Extract employee ID (4+ digit number)
pub static RE_EXTRACT_ID: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)(?:employee\s+)?(?:id\s*[:#]?\s*|#\s*)?(?P<id>\d{4,})").unwrap());

/// Extract first + last name
pub static RE_EXTRACT_FULL_NAME: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:named?|called|\bemployee)\s+(?P<first>[A-Za-z]+)\s+(?P<last>[A-Za-z]+)")
        .unwrap()
});

/// Extract department name from "in/from/of <DEPT> department"
pub static RE_EXTRACT_DEPT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:in|from|of|at|within|to)\s+(?:the\s+)?(?P<dept>[A-Za-z][A-Za-z\s]*?)(?:\s+department)?(?:\s+(?:earning|salary|making|with|who|and)\b|$)").unwrap()
});

/// Extract designation name after "as/designation"
pub static RE_EXTRACT_DESIG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:as|designation)\s+(?P<desig>[A-Za-z][A-Za-z\s\/]+?)(?:\s+with|\s+salary|\s+hire|$)").unwrap()
});

/// Extract email
pub static RE_EXTRACT_EMAIL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:email\s+)?(?P<email>[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,})")
        .unwrap()
});

/// Extract salary (number after "salary")
pub static RE_EXTRACT_SALARY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)salary\s+(?P<salary>-?[0-9]+(?:\.[0-9]{1,2})?)").unwrap());

/// Extract hire date (yyyy-MM-dd format)
pub static RE_EXTRACT_DATE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)(?:hire\s*date|hired?)\s+(?P<date>\d{4}-\d{2}-\d{2})").unwrap());

/// Extract status
pub static RE_EXTRACT_STATUS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)status\s+(?:to\s+)?(?P<status>active|inactive|on\s+leave)").unwrap()
});

/// Extract phone number
pub static RE_EXTRACT_PHONE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(?:phone|mobile|tel|contact)\s+(?P<phone>[0-9+\-\s]{7,20})").unwrap()
});

/// Exit commands
pub static RE_EXIT: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)^(exit|quit|bye|goodbye|q|:q)$").unwrap());

/// Help commands
pub static RE_HELP: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^(help|\?|commands?)$").unwrap());
