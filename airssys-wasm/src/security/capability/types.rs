//! Capability type utilities and re-exports.

// Re-export from core
pub use crate::core::security::capability::*;

/// Pattern matcher for capability validation.
///
/// Provides glob-like pattern matching for capability patterns.
/// Supports wildcard (`*`), prefix patterns (`prefix/*`), and suffix patterns (`*.suffix`).
pub struct PatternMatcher;

impl PatternMatcher {
    /// Match a target against a pattern (glob-like).
    ///
    /// # Pattern Syntax
    ///
    /// - `"*"` - Matches any target
    /// - `"prefix/*"` - Matches any target starting with `prefix/` (requires at least one character after /)
    /// - `"*.suffix"` - Matches any target ending with `.suffix` (requires at least one character before .)
    /// - `"exact"` - Matches exactly the target string
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::security::capability::types::PatternMatcher;
    ///
    /// // Wildcard matches anything
    /// assert!(PatternMatcher::matches("*", "anything"));
    ///
    /// // Prefix pattern
    /// assert!(PatternMatcher::matches("comp-a/*", "comp-a/123"));
    /// assert!(!PatternMatcher::matches("comp-a/*", "comp-b/123"));
    ///
    /// // Exact match
    /// assert!(PatternMatcher::matches("exact", "exact"));
    /// assert!(!PatternMatcher::matches("exact", "different"));
    /// ```
    pub fn matches(pattern: &str, target: &str) -> bool {
        if pattern == "*" {
            return true;
        }
        if let Some(prefix) = pattern.strip_suffix("/*") {
            // Pattern is "prefix/*" - target must start with prefix/ and have at least one more character
            return target.starts_with(prefix) && target.len() > prefix.len();
        }
        if let Some(suffix) = pattern.strip_prefix("*.") {
            // Pattern is "*.suffix" - target must end with .suffix and have at least one character before .
            return target.ends_with(suffix) && target.len() > suffix.len();
        }
        pattern == target
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard_matches_anything() {
        assert!(PatternMatcher::matches("*", "anything"));
        assert!(PatternMatcher::matches("*", ""));
        assert!(PatternMatcher::matches("*", "prefix/suffix"));
    }

    #[test]
    fn test_prefix_pattern_matches_correctly() {
        assert!(PatternMatcher::matches("prefix/*", "prefix/suffix"));
        assert!(PatternMatcher::matches("prefix/*", "prefix/sub/path"));
        assert!(!PatternMatcher::matches("prefix/*", "other/path"));
        // Must have at least one character after /
        assert!(!PatternMatcher::matches("prefix/*", "prefix"));
    }

    #[test]
    fn test_exact_match() {
        assert!(PatternMatcher::matches("exact", "exact"));
        assert!(!PatternMatcher::matches("exact", "exact-mismatch"));
    }

    #[test]
    fn test_non_matching_pattern() {
        assert!(!PatternMatcher::matches("pattern/*", "other"));
        assert!(!PatternMatcher::matches("exact", "different"));
    }

    #[test]
    fn test_edge_cases() {
        assert!(PatternMatcher::matches("*", ""));
        assert!(!PatternMatcher::matches("test/*", ""));
        assert!(PatternMatcher::matches("", ""));
    }

    #[test]
    fn test_suffix_wildcard_pattern() {
        assert!(PatternMatcher::matches("*.internal", "service.internal"));
        assert!(PatternMatcher::matches("*.internal", "api.internal"));
        assert!(!PatternMatcher::matches("*.internal", "external.com"));
        // Must have at least one character before the dot
        assert!(!PatternMatcher::matches("*.internal", "internal"));
    }
}
