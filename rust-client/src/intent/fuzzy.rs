// =============================================================================
// intent/fuzzy.rs — Deterministic Levenshtein Distance & Typo Correction
// Zero LLM. Pure classical dynamic programming algorithm.
// =============================================================================

/// Computes the Levenshtein edit distance between two strings.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut dp = vec![vec![0; n + 1]; m + 1];

    for (i, row) in dp.iter_mut().enumerate().take(m + 1) {
        row[0] = i;
    }
    for (j, val) in dp[0].iter_mut().enumerate().take(n + 1) {
        *val = j;
    }

    for i in 1..=m {
        for j in 1..=n {
            if a_chars[i - 1].to_ascii_lowercase() == b_chars[j - 1].to_ascii_lowercase() {
                dp[i][j] = dp[i - 1][j - 1];
            } else {
                dp[i][j] = 1 + dp[i - 1][j - 1].min(dp[i - 1][j]).min(dp[i][j - 1]);
            }
        }
    }

    dp[m][n]
}

/// Known dictionary words for autocorrection
const KNOWN_KEYWORDS: &[&str] = &[
    "employee",
    "employees",
    "department",
    "departments",
    "designation",
    "designations",
    "salary",
    "salaries",
    "status",
    "active",
    "inactive",
    "leave",
    "search",
    "find",
    "update",
    "count",
    "show",
    "list",
    // Departments
    "engineering",
    "operations",
    "finance",
    "sales",
    // Designations
    "engineer",
    "manager",
    "analyst",
    "executive",
    "intern",
];

/// Finds the closest matching keyword if the word is misspelled (edit distance <= 2).
pub fn find_closest_keyword(word: &str) -> Option<&'static str> {
    let clean = word.trim().to_lowercase();
    if clean.len() < 3 {
        return None;
    }

    let mut best_match = None;
    let mut min_distance = 3; // only accept distance 1 or 2

    for &kw in KNOWN_KEYWORDS {
        let dist = levenshtein(&clean, kw);
        if dist < min_distance {
            min_distance = dist;
            best_match = Some(kw);
        }
    }

    best_match
}

/// Corrects known typos in a prompt while preserving numbers, symbols, and formatting.
pub fn autocorrect_prompt(prompt: &str) -> String {
    let words: Vec<&str> = prompt.split_whitespace().collect();
    let mut corrected_words = Vec::new();

    for word in words {
        // Strip trailing punctuation if any
        let trimmed = word.trim_matches(|c: char| !c.is_alphanumeric());
        if !trimmed.is_empty() && !trimmed.chars().all(|c| c.is_ascii_digit()) {
            if let Some(closest) = find_closest_keyword(trimmed) {
                // If it was misspelled (dist 1 or 2, but not identical)
                let dist = levenshtein(&trimmed.to_lowercase(), closest);
                if dist > 0 && dist <= 2 {
                    corrected_words.push(closest.to_string());
                    continue;
                }
            }
        }
        corrected_words.push(word.to_string());
    }

    corrected_words.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("employe", "employee"), 1);
        assert_eq!(levenshtein("enginering", "engineering"), 1);
        assert_eq!(levenshtein("salry", "salary"), 1);
    }

    #[test]
    fn test_autocorrect_prompt() {
        let corrected = autocorrect_prompt("show employes in enginering");
        assert_eq!(corrected, "show employee in engineering");

        let corrected2 = autocorrect_prompt("find employe 1001 salry");
        assert_eq!(corrected2, "find employee 1001 salary");
    }
}
