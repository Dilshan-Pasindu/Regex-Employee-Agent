// =============================================================================
// intent/patterns.rs — All Regex patterns for deterministic intent detection
// No LLM, no AI. Pure Regex. Case-insensitive via (?i) flag.
// =============================================================================

use once_cell::sync::Lazy;
use regex::Regex;

// ---------------------------------------------------------------------------
// get_all_employees patterns
// Matches: "show all employees", "show all employees details", "list employees", "display all staff", etc.
// ---------------------------------------------------------------------------
pub static RE_GET_ALL_EMPLOYEES: Lazy<Vec<Regex>> = Lazy::new(|| vec![
    Regex::new(r"(?i)^(show|list|display|get|fetch|view)\s+(all\s+)?employees?(?:\s+(?:details?|info(?:rmation)?|records?|data|list))?$").unwrap(),
    Regex::new(r"(?i)^(show|list|display|get|fetch|view)\s+(?:the\s+)?employees?\s+(?:details?|info(?:rmation)?|records?|data|list)$").unwrap(),
    Regex::new(r"(?i)^(show|list|display|get|fetch|view)\s+all\s+staff(?:\s+(?:details?|info(?:rmation)?|records?|data|list))?$").unwrap(),
    Regex::new(r"(?i)^all\s+employees?(?:\s+(?:details?|info(?:rmation)?|records?|data|list))?$").unwrap(),
    Regex::new(r"(?i)^employees?\s+(list|records?|details?|data|info(?:rmation)?)$").unwrap(),
    Regex::new(r"(?i)^(give\s+me\s+|show\s+me\s+)?all\s+employees?(?:\s+(?:details?|info(?:rmation)?|records?|data|list))?$").unwrap(),
]);

// ---------------------------------------------------------------------------
// get_employee_by_id patterns
// Matches: "find employee 1001", "show employee ID 1002", etc.
// ---------------------------------------------------------------------------
pub static RE_GET_BY_ID: Lazy<Vec<Regex>> = Lazy::new(|| vec![
    Regex::new(r"(?i)(find|get|show|fetch|lookup|view|display)\s+employee\s+(?:id\s*[:#]?\s*|#\s*)?(?P<id>\d{4,})").unwrap(),
    Regex::new(r"(?i)employee\s+(?:id\s*[:#]?\s*|#\s*)?(?P<id>\d{4,})").unwrap(),
    Regex::new(r"(?i)(find|show|get)\s+(?:employee\s+)?(?:id\s*[:#]?\s*)?(?P<id>\d{4,})").unwrap(),
]);

// ---------------------------------------------------------------------------
// search_employees patterns
// Matches: "search employees in IT", "find employees from HR", etc.
// ---------------------------------------------------------------------------
pub static RE_SEARCH_EMPLOYEES: Lazy<Vec<Regex>> = Lazy::new(|| vec![
    // By department
    Regex::new(r"(?i)(find|search|show|get|list|fetch)\s+employees?\s+(in|from|of|at|within)\s+(?:the\s+)?(?P<dept>[A-Za-z][A-Za-z\s]*?)(?:\s+department)?$").unwrap(),
    Regex::new(r"(?i)(?P<dept>[A-Za-z]+)\s+department\s+employees?").unwrap(),
    // By name
    Regex::new(r"(?i)(find|search|show|get)\s+(employees?|staff)\s+(named?|called)\s+(?P<name>[A-Za-z][A-Za-z\s]+)$").unwrap(),
    // By designation
    Regex::new(r"(?i)(find|search|show|get)\s+(employees?|staff)\s+(with\s+designation|who\s+are|as)\s+(?P<desig>[A-Za-z][A-Za-z\s]+)$").unwrap(),
    // By status
    Regex::new(r"(?i)(find|search|show|get|list)\s+(all\s+)?(?P<status>active|inactive|on\s+leave)\s+employees?").unwrap(),
    // General search for a name
    Regex::new(r"(?i)(find|search)\s+(?P<name>[A-Za-z]+(?:\s+[A-Za-z]+)?)\s+in\s+employees?").unwrap(),
    // "search for John" or "find John"
    Regex::new(r"(?i)^(find|search|look\s+up|lookup)\s+(?P<name>[A-Za-z][A-Za-z\s]+)$").unwrap(),
]);

// ---------------------------------------------------------------------------
// get_department_list patterns
// ---------------------------------------------------------------------------
pub static RE_GET_DEPARTMENTS: Lazy<Vec<Regex>> = Lazy::new(|| vec![
    Regex::new(r"(?i)^(show|list|get|display|fetch|view)\s+(all\s+)?departments?$").unwrap(),
    Regex::new(r"(?i)^departments?(\s+list)?$").unwrap(),
    Regex::new(r"(?i)^(give\s+me\s+|show\s+me\s+)?(all\s+)?departments?$").unwrap(),
]);

// ---------------------------------------------------------------------------
// get_employee_count patterns
// ---------------------------------------------------------------------------
pub static RE_GET_COUNT: Lazy<Vec<Regex>> = Lazy::new(|| vec![
    Regex::new(r"(?i)how\s+many\s+employees?").unwrap(),
    Regex::new(r"(?i)count\s+(of\s+)?(all\s+)?employees?").unwrap(),
    Regex::new(r"(?i)employee\s+count").unwrap(),
    Regex::new(r"(?i)total\s+(number\s+of\s+)?employees?").unwrap(),
    Regex::new(r"(?i)count\s+employees?\s+(in|from|of)\s+(?P<dept>[A-Za-z][A-Za-z\s]*?)(\s+department)?$").unwrap(),
    Regex::new(r"(?i)how\s+many\s+employees?\s+(are\s+)?(in|from|of)\s+(?P<dept>[A-Za-z][A-Za-z\s]*?)(\s+department)?$").unwrap(),
]);

// ---------------------------------------------------------------------------
// add_employee patterns
// ---------------------------------------------------------------------------
pub static RE_ADD_EMPLOYEE: Lazy<Vec<Regex>> = Lazy::new(|| vec![
    Regex::new(r"(?i)^(add|create|insert|register|new|hire)\s+(a\s+)?(new\s+)?employee").unwrap(),
    Regex::new(r"(?i)^(add|create|insert|register|hire)\s+(?P<first>[A-Za-z]+)\s+(?P<last>[A-Za-z]+)").unwrap(),
    Regex::new(r"(?i)^new\s+employee\s+(?:named?\s+)?(?P<first>[A-Za-z]+)").unwrap(),
]);

// ---------------------------------------------------------------------------
// update_employee patterns
// ---------------------------------------------------------------------------
pub static RE_UPDATE_EMPLOYEE: Lazy<Vec<Regex>> = Lazy::new(|| vec![
    Regex::new(r"(?i)(update|change|modify|edit|set)\s+employee\s+(?:id\s*[:#]?\s*)?(?P<id>\d{4,})\s+(?P<field>department|designation|salary|status|phone|email|first\s*name|last\s*name|hire\s*date|name)\s+(to|as)\s+(?P<val>.+)$").unwrap(),
    Regex::new(r"(?i)(update|change|modify|edit)\s+(?:employee\s+)?(?:id\s*[:#]?\s*)?(?P<id>\d{4,})").unwrap(),
]);

// ---------------------------------------------------------------------------
// Parameter extraction patterns
// ---------------------------------------------------------------------------

/// Extract employee ID (4+ digit number)
pub static RE_EXTRACT_ID: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?i)(?:employee\s+)?(?:id\s*[:#]?\s*|#\s*)?(?P<id>\d{4,})").unwrap()
);

/// Extract first + last name
pub static RE_EXTRACT_FULL_NAME: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?i)(?:named?|called|\bemployee)\s+(?P<first>[A-Za-z]+)\s+(?P<last>[A-Za-z]+)").unwrap()
);

/// Extract department name from "in/from/of <DEPT> department"
pub static RE_EXTRACT_DEPT: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?i)(?:in|from|of|at|within|to)\s+(?:the\s+)?(?P<dept>[A-Za-z][A-Za-z\s]*?)(?:\s+department)?$").unwrap()
);

/// Extract designation name after "as/designation"
pub static RE_EXTRACT_DESIG: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?i)(?:as|designation)\s+(?P<desig>[A-Za-z][A-Za-z\s\/]+?)(?:\s+with|\s+salary|\s+hire|$)").unwrap()
);

/// Extract email
pub static RE_EXTRACT_EMAIL: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?i)(?:email\s+)?(?P<email>[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,})").unwrap()
);

/// Extract salary (number after "salary")
pub static RE_EXTRACT_SALARY: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?i)salary\s+(?P<salary>[0-9]+(?:\.[0-9]{1,2})?)").unwrap()
);

/// Extract hire date (yyyy-MM-dd format)
pub static RE_EXTRACT_DATE: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?i)(?:hire\s*date|hired?)\s+(?P<date>\d{4}-\d{2}-\d{2})").unwrap()
);

/// Extract status
pub static RE_EXTRACT_STATUS: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?i)status\s+(?:to\s+)?(?P<status>active|inactive|on\s+leave)").unwrap()
);

/// Extract phone number
pub static RE_EXTRACT_PHONE: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?i)(?:phone|mobile|tel|contact)\s+(?P<phone>[0-9+\-\s]{7,20})").unwrap()
);

/// Exit commands
pub static RE_EXIT: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?i)^(exit|quit|bye|goodbye|q|:q)$").unwrap()
);

/// Help commands
pub static RE_HELP: Lazy<Regex> = Lazy::new(||
    Regex::new(r"(?i)^(help|\?|commands?)$").unwrap()
);

