#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationMethod {
    Exact,
    CaseInsensitive,
    NormalizedText,
    NumericExact,
}

impl VerificationMethod {
    pub fn from_db(value: &str) -> Option<Self> {
        match value {
            "exact" => Some(Self::Exact),
            "case_insensitive" => Some(Self::CaseInsensitive),
            "normalized_text" => Some(Self::NormalizedText),
            "numeric_exact" => Some(Self::NumericExact),
            _ => None,
        }
    }
}

pub fn answers_match(method: VerificationMethod, expected: &str, submitted: &str) -> bool {
    match method {
        VerificationMethod::Exact => submitted == expected,
        VerificationMethod::CaseInsensitive => submitted.to_lowercase() == expected.to_lowercase(),
        VerificationMethod::NormalizedText => {
            normalize_space(submitted) == normalize_space(expected)
        }
        VerificationMethod::NumericExact => numeric_text(submitted) == numeric_text(expected),
    }
}

fn normalize_space(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn numeric_text(value: &str) -> Option<String> {
    let trimmed = value.trim();
    let ok = trimmed
        .chars()
        .all(|c| c.is_ascii_digit() || c == '.' || c == '-')
        && trimmed
            .strip_prefix('-')
            .unwrap_or(trimmed)
            .chars()
            .filter(|c| *c == '.')
            .count()
            <= 1;
    if !ok {
        return None;
    }
    let re_ok = regex_numeric(trimmed);
    if re_ok {
        Some(trimmed.to_string())
    } else {
        None
    }
}

fn regex_numeric(trimmed: &str) -> bool {
    if trimmed.is_empty() {
        return false;
    }
    let bytes = trimmed.as_bytes();
    let mut i = 0;
    if bytes.get(i) == Some(&b'-') {
        i += 1;
        if i >= bytes.len() {
            return false;
        }
    }
    let mut saw_dot = false;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'.' {
            if saw_dot {
                return false;
            }
            saw_dot = true;
        } else if !b.is_ascii_digit() {
            return false;
        }
        i += 1;
    }
    if trimmed == "-" {
        return false;
    }
    !trimmed.ends_with('.')
        && (trimmed.starts_with('-') && trimmed.len() > 1 || !trimmed.starts_with('-'))
        && bytes.iter().any(|b| b.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_text_only() {
        assert!(answers_match(VerificationMethod::Exact, "42", "42"));
        assert!(!answers_match(VerificationMethod::Exact, "42", " 42 "));
    }

    #[test]
    fn case_insensitive_rules() {
        assert!(answers_match(
            VerificationMethod::CaseInsensitive,
            "Paris",
            "paris"
        ));
        assert!(!answers_match(
            VerificationMethod::CaseInsensitive,
            "Paris",
            " paris"
        ));
    }

    #[test]
    fn normalized_text_rules() {
        assert!(answers_match(
            VerificationMethod::NormalizedText,
            "42",
            " 42 "
        ));
        assert!(answers_match(
            VerificationMethod::NormalizedText,
            "Hello world",
            "Hello  world"
        ));
        assert!(!answers_match(
            VerificationMethod::NormalizedText,
            "Hello",
            "hello"
        ));
    }

    #[test]
    fn numeric_exact_rules() {
        assert!(answers_match(
            VerificationMethod::NumericExact,
            "42",
            " 42 "
        ));
        assert!(!answers_match(
            VerificationMethod::NumericExact,
            "42",
            "42.0"
        ));
        assert!(!answers_match(
            VerificationMethod::NumericExact,
            "42",
            "forty-two"
        ));
    }

    #[test]
    fn mcq_exact_match_accepts_correct_and_rejects_wrong_option() {
        assert!(answers_match(
            VerificationMethod::Exact,
            "Option B",
            "Option B"
        ));
        assert!(!answers_match(
            VerificationMethod::Exact,
            "Option B",
            "Option A"
        ));
        assert!(VerificationMethod::from_db("manual").is_none());
    }
}
