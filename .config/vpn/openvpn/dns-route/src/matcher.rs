/// Simple contains-based pattern matcher for domain names.
pub struct Patterns {
    patterns: Vec<String>,
}

impl Patterns {
    /// Create a new pattern matcher from comma-separated patterns.
    pub fn new(patterns_str: &str) -> Self {
        let patterns: Vec<String> = patterns_str
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        Self { patterns }
    }

    /// Check if the domain matches any pattern (case-insensitive contains).
    pub fn matches(&self, domain: &str) -> bool {
        let domain_lower = domain.to_lowercase();
        self.patterns.iter().any(|p| domain_lower.contains(p))
    }

    /// Returns the number of patterns.
    pub fn len(&self) -> usize {
        self.patterns.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_match() {
        let patterns = Patterns::new("corp,internal");
        assert!(patterns.matches("internal.corp.com"));
        assert!(patterns.matches("www.internal.example.com"));
        assert!(patterns.matches("CORP.EXAMPLE.COM"));
        assert!(!patterns.matches("www.example.com"));
    }

    #[test]
    fn test_empty_patterns() {
        let patterns = Patterns::new("");
        assert!(!patterns.matches("anything.com"));
    }
}
