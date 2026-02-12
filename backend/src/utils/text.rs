pub fn normalize_text(text: &str) -> String {
    text.to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn tokenize(text: &str) -> Vec<String> {
    normalize_text(text)
        .split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

pub fn similarity(a: &str, b: &str) -> f64 {
    let a_tokens: std::collections::HashSet<&str> = a.split_whitespace().collect();
    let b_tokens: std::collections::HashSet<&str> = b.split_whitespace().collect();

    if a_tokens.is_empty() || b_tokens.is_empty() {
        return 0.0;
    }

    let intersection = a_tokens.intersection(&b_tokens).count();
    let union = a_tokens.union(&b_tokens).count();

    intersection as f64 / union as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_text() {
        assert_eq!(normalize_text("Hello, World!"), "hello world");
        assert_eq!(normalize_text("  Multiple   Spaces  "), "multiple spaces");
        assert_eq!(normalize_text("Test123"), "test123");
    }

    #[test]
    fn test_tokenize() {
        let tokens = tokenize("Hello, World!");
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_similarity() {
        let sim = similarity("hello world", "hello there");
        assert!(sim > 0.0 && sim < 1.0);

        let identical = similarity("same text", "same text");
        assert_eq!(identical, 1.0);

        let no_match = similarity("one two", "three four");
        assert_eq!(no_match, 0.0);
    }
}
