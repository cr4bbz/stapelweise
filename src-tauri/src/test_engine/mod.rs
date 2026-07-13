use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipleChoiceContent {
    pub options: Vec<MultipleChoiceOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipleChoiceOption {
    pub text: String,
    pub correct: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderingContent {
    pub items: Vec<String>,
}

/// Evaluates a user's answer against a question's expected solution.
/// Returns `(is_correct, points_earned)` out of max 1.0 point.
pub fn evaluate_answer(
    card_type: &str,
    expected_back: &str,
    content_json: Option<&str>,
    user_answer: &str,
) -> (bool, f64) {
    let clean_user = user_answer.trim();
    if clean_user.is_empty() {
        return (false, 0.0);
    }

    match card_type {
        "multiple_choice" => {
            if let Some(json_str) = content_json {
                if let Ok(mc) = serde_json::from_str::<MultipleChoiceContent>(json_str) {
                    // Expecting user_answer to be newline-separated or JSON array of selected texts
                    let user_selected: Vec<String> = if clean_user.starts_with('[') {
                        serde_json::from_str(clean_user).unwrap_or_else(|_| vec![clean_user.to_string()])
                    } else {
                        clean_user.lines().map(|s| s.trim().to_string()).collect()
                    };

                    let correct_options: Vec<String> = mc
                        .options
                        .iter()
                        .filter(|o| o.correct)
                        .map(|o| o.text.trim().to_string())
                        .collect();

                    let mut matches = 0;
                    for opt in &user_selected {
                        if correct_options.contains(opt) {
                            matches += 1;
                        }
                    }

                    let all_correct = matches == correct_options.len()
                        && user_selected.len() == correct_options.len();
                    let score = if all_correct { 1.0 } else { 0.0 };
                    return (all_correct, score);
                }
            }
            // Fallback string matching
            let is_match = normalize(clean_user) == normalize(expected_back);
            (is_match, if is_match { 1.0 } else { 0.0 })
        }
        "ordering" => {
            if let Some(json_str) = content_json {
                if let Ok(ord) = serde_json::from_str::<OrderingContent>(json_str) {
                    let user_items: Vec<String> = if clean_user.starts_with('[') {
                        serde_json::from_str(clean_user).unwrap_or_else(|_| vec![])
                    } else {
                        clean_user
                            .lines()
                            .map(|l| {
                                // Strip leading numbers like "1. "
                                let s = l.trim();
                                if let Some(pos) = s.find(". ") {
                                    s[pos + 2..].trim().to_string()
                                } else {
                                    s.to_string()
                                }
                            })
                            .filter(|s| !s.is_empty())
                            .collect()
                    };

                    let expected_items: Vec<String> = ord.items.iter().map(|s| s.trim().to_string()).collect();
                    let is_exact = user_items == expected_items;
                    return (is_exact, if is_exact { 1.0 } else { 0.0 });
                }
            }
            let is_match = normalize(clean_user) == normalize(expected_back);
            (is_match, if is_match { 1.0 } else { 0.0 })
        }
        "cloze" => {
            let extracted = extract_cloze_targets(expected_back);
            if !extracted.is_empty() {
                let user_parts: Vec<String> = clean_user
                    .lines()
                    .map(|s| normalize(s.trim()))
                    .filter(|s| !s.is_empty())
                    .collect();

                let mut correct_count = 0;
                for target in &extracted {
                    let norm_target = normalize(target);
                    if user_parts.iter().any(|p| p == &norm_target) {
                        correct_count += 1;
                    }
                }

                let all_correct = correct_count == extracted.len();
                let score = (correct_count as f64) / (extracted.len() as f64);
                return (all_correct, score);
            }
            let is_match = normalize(clean_user) == normalize(expected_back);
            (is_match, if is_match { 1.0 } else { 0.0 })
        }
        "free_text" | "basic" | _ => {
            let is_match = normalize(clean_user) == normalize(expected_back);
            (is_match, if is_match { 1.0 } else { 0.0 })
        }
    }
}

fn normalize(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .replace(['\r', '\n', '\t'], " ")
        .replace("  ", " ")
}

fn extract_cloze_targets(text: &str) -> Vec<String> {
    let mut targets = Vec::new();
    let mut rest = text;

    while let Some(start) = rest.find("==") {
        let after_start = &rest[start + 2..];
        if let Some(end) = after_start.find("==") {
            let target = &after_start[..end];
            if !target.trim().is_empty() {
                targets.push(target.trim().to_string());
            }
            rest = &after_start[end + 2..];
        } else {
            break;
        }
    }
    targets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_mc() {
        let json = r#"{"options":[{"text":"A","correct":true},{"text":"B","correct":false}]}"#;
        let (ok, score) = evaluate_answer("multiple_choice", "A", Some(json), "A");
        assert!(ok);
        assert_eq!(score, 1.0);

        let (bad, score2) = evaluate_answer("multiple_choice", "A", Some(json), "B");
        assert!(!bad);
        assert_eq!(score2, 0.0);
    }

    #[test]
    fn test_eval_ordering() {
        let json = r#"{"items":["Step 1","Step 2","Step 3"]}"#;
        let (ok, score) = evaluate_answer("ordering", "", Some(json), "1. Step 1\n2. Step 2\n3. Step 3");
        assert!(ok);
        assert_eq!(score, 1.0);
    }
}
