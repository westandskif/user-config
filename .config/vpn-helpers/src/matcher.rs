/// Checks if a domain matches any of the given suffix patterns
pub fn matches_patterns(domain: &str, patterns: &[String]) -> bool {
    let domain_lower = domain.to_lowercase();
    patterns.iter().any(|pattern| {
        let pattern_lower = pattern.to_lowercase();
        domain_lower.contains(&pattern_lower)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_patterns() {
        let patterns = vec![".by".to_string(), ".ru".to_string()];

        assert!(matches_patterns("example.by", &patterns));
        assert!(matches_patterns("api.example.by", &patterns));
        assert!(matches_patterns("test.ru", &patterns));
        assert!(matches_patterns("EXAMPLE.BY", &patterns));

        assert!(!matches_patterns("example.com", &patterns));
        assert!(!matches_patterns("ruby.org", &patterns));
    }
}
