// =============================================================================
// intent/vocabulary.rs — Dynamic YAML Vocabulary Loader & Pattern Builder
//
// Loads config/vocabulary.yaml (with embedded fallback) to configure intent
// detection keywords, synonyms, suffixes, field aliases, and status normalisation.
// =============================================================================

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const EMBEDDED_YAML: &str = include_str!("../../config/vocabulary.yaml");

/// Global singleton instance of Vocabulary
pub static VOCABULARY: Lazy<Vocabulary> = Lazy::new(Vocabulary::load);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Vocabulary {
    pub intents: IntentsVocab,
    pub field_aliases: HashMap<String, Vec<String>>,
    pub status_mappings: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IntentsVocab {
    pub get_all_employees: GetAllEmployeesVocab,
    pub get_employee_by_id: GetEmployeeByIdVocab,
    pub search_employees: SearchEmployeesVocab,
    pub get_department_list: GetDepartmentListVocab,
    pub get_employee_count: GetEmployeeCountVocab,
    pub add_employee: AddEmployeeVocab,
    pub update_employee: UpdateEmployeeVocab,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetAllEmployeesVocab {
    pub verbs: Vec<String>,
    pub targets: Vec<String>,
    pub quantifiers: Vec<String>,
    pub suffixes: Vec<String>,
    pub standalone_phrases: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetEmployeeByIdVocab {
    pub verbs: Vec<String>,
    pub prefixes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchEmployeesVocab {
    pub verbs: Vec<String>,
    pub prepositions: Vec<String>,
    pub name_indicators: Vec<String>,
    pub designation_indicators: Vec<String>,
    pub statuses: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetDepartmentListVocab {
    pub verbs: Vec<String>,
    pub targets: Vec<String>,
    pub standalone_phrases: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetEmployeeCountVocab {
    pub count_triggers: Vec<String>,
    pub targets: Vec<String>,
    pub standalone_phrases: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AddEmployeeVocab {
    pub verbs: Vec<String>,
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateEmployeeVocab {
    pub verbs: Vec<String>,
    pub assignment_words: Vec<String>,
}

impl Vocabulary {
    /// Load vocabulary from config/vocabulary.yaml or embedded fallback
    pub fn load() -> Self {
        // Try paths in priority order:
        let candidates = [
            std::env::var("VOCABULARY_PATH").ok().map(PathBuf::from),
            Some(PathBuf::from("config/vocabulary.yaml")),
            Some(PathBuf::from("../config/vocabulary.yaml")),
            Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config/vocabulary.yaml")),
            Some(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .parent()
                    .map(|p| p.join("config/vocabulary.yaml"))
                    .unwrap_or_default(),
            ),
        ];

        for candidate in candidates.into_iter().flatten() {
            if candidate.exists() {
                if let Ok(content) = fs::read_to_string(&candidate) {
                    if let Ok(vocab) = serde_yaml::from_str::<Vocabulary>(&content) {
                        return vocab;
                    }
                }
            }
        }

        // Fallback to embedded YAML
        serde_yaml::from_str(EMBEDDED_YAML)
            .expect("Failed to parse embedded vocabulary.yaml fallback")
    }

    /// Access the global singleton
    pub fn get() -> &'static Vocabulary {
        &VOCABULARY
    }

    /// Escape and join items into a regex alternation: `(a|b|c)`
    pub fn alternation(items: &[String]) -> String {
        let escaped: Vec<String> = items.iter().map(|s| regex::escape(s)).collect();
        escaped.join("|")
    }

    /// Normalise status using configured mappings
    pub fn normalise_status(&self, raw: &str) -> String {
        let trimmed = raw.to_lowercase().trim().to_string();

        for (canonical, variants) in &self.status_mappings {
            for variant in variants {
                if trimmed == variant.to_lowercase() {
                    return canonical.clone();
                }
            }
        }

        // Fallback title-case
        let mut c = trimmed.chars();
        match c.next() {
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            None => String::new(),
        }
    }

    /// Extract requested fields using configured field aliases
    pub fn match_requested_fields(&self, prompt: &str) -> Vec<String> {
        let mut matched = Vec::new();
        let lower = prompt.to_lowercase();

        // Specific fields first
        let priority_fields = [
            "departmentId",
            "departmentName",
            "designationId",
            "designationName",
            "salary",
            "email",
            "phone",
            "hireDate",
            "status",
            "firstName",
            "lastName",
        ];

        for field in &priority_fields {
            if let Some(aliases) = self.field_aliases.get(*field) {
                for alias in aliases {
                    let pattern = format!(r"\b{}\b", regex::escape(alias));
                    if let Ok(re) = regex::Regex::new(&pattern) {
                        if re.is_match(&lower) {
                            if !matched.contains(&field.to_string()) {
                                matched.push(field.to_string());
                            }
                            break;
                        }
                    }
                }
            }
        }

        // Generic "name" / "full name" -> firstName + lastName
        if let Some(aliases) = self.field_aliases.get("fullName") {
            for alias in aliases {
                let pattern = format!(r"\b{}\b", regex::escape(alias));
                if let Ok(re) = regex::Regex::new(&pattern) {
                    if re.is_match(&lower)
                        && !matched.contains(&"firstName".to_string())
                        && !matched.contains(&"departmentName".to_string())
                        && !matched.contains(&"designationName".to_string())
                        && !lower.contains("department")
                        && !lower.contains("designation")
                    {
                        matched.push("firstName".to_string());
                        matched.push("lastName".to_string());
                        break;
                    }
                }
            }
        }

        matched
    }
}
