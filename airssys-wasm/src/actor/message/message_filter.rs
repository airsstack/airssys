//! Topic-based message filtering for inter-component communication.
//!
//! This module implements `TopicFilter` and `TopicPattern` for filtering messages
//! between WASM components in the airssys-wasm framework. Components subscribe to
//! topics and receive messages routed through the ActorSystem's MessageBroker.
//!
//! # Wildcard Patterns (MQTT-style)
//!
//! The pattern syntax is inspired by MQTT 3.1.1 topic filters:
//! - **`*` (Single-level)**: Matches exactly one segment
//!   - `events.user.*` matches `events.user.login` but NOT `events.user.login.success`
//! - **`#` (Multi-level)**: Matches **zero or more** segments (must be last segment)
//!   - **Zero segments**: `events.#` matches `events` (# consumes nothing after prefix)
//!   - **One segment**: `events.#` matches `events.user`
//!   - **Many segments**: `events.#` matches `events.user.login.success`
//!   - Pattern rule: Multi-level wildcard specified alone or next to separator
//!
//! # Architecture Context (ADR-WASM-009)
//!
//! Per ADR-WASM-009 Component Communication Model:
//! - **Inter-Component Messaging**: WASM components communicate via topic-based routing
//! - **ActorSystem Intermediation**: Messages flow through MessageBroker in airssys-rt
//! - **Subscription Filtering**: Efficient pattern matching for selective delivery
//! - **Decoupled Communication**: Publishers don't need to know subscriber identities
//!
//! # Use Case: Component Communication
//!
//! ```text
//! Component A                  ActorSystem/MessageBroker                Component B
//!    |                                   |                                   |
//!    |  subscribe("events.user.*")      |                                   |
//!    |---------------------------------->|                                   |
//!    |                                   |  subscribe("events.#")            |
//!    |                                   |<----------------------------------|
//!    |                                   |                                   |
//!    |  publish("events.user.login")    |                                   |
//!    |---------------------------------->|                                   |
//!    |                                   |  deliver(message)                 |
//!    |                                   |---------------------------------->|
//!    |                                   |  deliver(message)                 |
//!    |<----------------------------------|                                   |
//! ```
//!
//! # Performance
//!
//! Target: <50ns per match operation (recursive pattern matching algorithm)
//!
//! # Examples
//!
//! ```rust,ignore
//! use airssys_wasm::actor::TopicFilter;
//!
//! // Component A subscribes to user events
//! let filter = TopicFilter::from_patterns(vec!["events.user.*"]);
//! assert!(filter.matches("events.user.login"));
//! assert!(filter.matches("events.user.logout"));
//! assert!(!filter.matches("events.user.login.success"));
//!
//! // Component B subscribes to all events
//! let filter = TopicFilter::from_patterns(vec!["events.#"]);
//! assert!(filter.matches("events"));
//! assert!(filter.matches("events.user"));
//! assert!(filter.matches("events.user.login.success"));
//! assert!(!filter.matches("system.restart"));
//!
//! // Component C subscribes to multiple patterns
//! let filter = TopicFilter::from_patterns(vec!["events.user.*", "system.#"]);
//! assert!(filter.matches("events.user.login"));      // From user events
//! assert!(filter.matches("system.restart.initiated")); // From system events
//! ```
//!
//! # References
//!
//! - **ADR-WASM-009**: Component Communication Model (Inter-Component Messaging)
//! - **MQTT 3.1.1 §4.7**: Topic filter pattern syntax inspiration
//! - **WASM-TASK-004 Phase 4**: MessageBroker integration for component communication

// Layer 1: Standard library imports
// (none required)

// Layer 2: Third-party crate imports
// (none required)

// Layer 3: Internal module imports
// (none required)

/// Topic filter for subscription matching.
///
/// TopicFilter evaluates whether a message topic matches any of the configured
/// subscription patterns, supporting MQTT 3.1.1 wildcard syntax.
///
/// # Wildcard Semantics
///
/// - **Literal**: Exact segment match (e.g., `events` matches only `events`)
/// - **`*`**: Single-level wildcard (matches exactly one segment)
/// - **`#`**: Multi-level wildcard (matches zero or more segments, must be last)
///
/// # Thread Safety
///
/// TopicFilter is Send + Sync, allowing use across async task boundaries.
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::actor::TopicFilter;
///
/// let filter = TopicFilter::from_patterns(vec![
///     "events.user.*",
///     "system.#",
///     "metrics.cpu",
/// ]);
///
/// assert!(filter.matches("events.user.login"));
/// assert!(filter.matches("system.restart"));
/// assert!(filter.matches("metrics.cpu"));
/// assert!(!filter.matches("events.system.error"));
/// ```
#[derive(Debug, Clone)]
pub struct TopicFilter {
    /// List of topic patterns to match against
    patterns: Vec<TopicPattern>,
}

impl TopicFilter {
    /// Create filter from topic patterns.
    ///
    /// # Arguments
    ///
    /// * `patterns` - Slice of pattern strings (MQTT wildcard syntax)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let filter = TopicFilter::from_patterns(vec![
    ///     "events.*",
    ///     "system.#",
    /// ]);
    /// ```
    pub fn from_patterns(patterns: Vec<&str>) -> Self {
        Self {
            patterns: patterns
                .into_iter()
                .map(TopicPattern::parse)
                .collect(),
        }
    }

    /// Check if message topic matches any filter pattern.
    ///
    /// Returns `true` if the topic matches at least one configured pattern.
    ///
    /// # Parameters
    ///
    /// * `topic` - Topic name to test (e.g., "events.user.login")
    ///
    /// # Returns
    ///
    /// `true` if topic matches any pattern, `false` otherwise
    ///
    /// # Performance
    ///
    /// Time complexity: O(n × m) where n = number of patterns, m = segments per pattern
    /// Target: <50ns per operation
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let filter = TopicFilter::from_patterns(vec!["events.*"]);
    ///
    /// assert!(filter.matches("events.login"));
    /// assert!(filter.matches("events.logout"));
    /// assert!(!filter.matches("system.restart"));
    /// ```
    pub fn matches(&self, topic: &str) -> bool {
        self.patterns.iter().any(|pattern| pattern.matches(topic))
    }

    /// Get number of patterns in filter.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let filter = TopicFilter::from_patterns(vec!["events.*", "system.#"]);
    /// assert_eq!(filter.pattern_count(), 2);
    /// ```
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }
}

/// Topic pattern with wildcard support.
///
/// TopicPattern represents a single MQTT-style topic filter pattern with
/// support for single-level (`*`) and multi-level (`#`) wildcards.
///
/// # Pattern Syntax
///
/// - **Literal segments**: `events.user.login` matches only exact topic
/// - **Single wildcard**: `events.user.*` matches any single segment after `user`
/// - **Multi wildcard**: `events.#` matches any segments after `events`
///
/// # Parsing Rules
///
/// 1. Topics are split by `.` delimiter into segments
/// 2. `*` segment matches exactly one segment
/// 3. `#` segment matches zero or more segments (must be last)
/// 4. Literal segments match only exact string
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::actor::TopicPattern;
///
/// // Single-level wildcard
/// let pattern = TopicPattern::parse("events.user.*");
/// assert!(pattern.matches("events.user.login"));
/// assert!(!pattern.matches("events.user.login.success"));
///
/// // Multi-level wildcard
/// let pattern = TopicPattern::parse("events.#");
/// assert!(pattern.matches("events"));
/// assert!(pattern.matches("events.user.login"));
///
/// // Literal pattern
/// let pattern = TopicPattern::parse("events.user.login");
/// assert!(pattern.matches("events.user.login"));
/// assert!(!pattern.matches("events.user.logout"));
/// ```
#[derive(Debug, Clone)]
pub struct TopicPattern {
    /// Pattern segments (literals or wildcards)
    segments: Vec<PatternSegment>,
}

/// Pattern segment type.
///
/// Represents a single segment in a topic pattern, which can be:
/// - Literal string (exact match)
/// - Single-level wildcard (`*`)
/// - Multi-level wildcard (`#`)
#[derive(Debug, Clone, PartialEq, Eq)]
enum PatternSegment {
    /// Literal segment (exact string match)
    Literal(String),
    /// Single-level wildcard (*) - matches exactly one segment
    SingleWildcard,
    /// Multi-level wildcard (#) - matches zero or more segments (must be last)
    MultiWildcard,
}

impl TopicPattern {
    /// Parse topic pattern from string.
    ///
    /// Converts pattern string into internal representation with parsed segments.
    ///
    /// # Arguments
    ///
    /// * `pattern` - Pattern string with optional wildcards
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let pattern = TopicPattern::parse("events.user.*");
    /// let pattern = TopicPattern::parse("events.#");
    /// let pattern = TopicPattern::parse("events.user.login");
    /// ```
    pub fn parse(pattern: &str) -> Self {
        let segments = pattern
            .split('.')
            .map(|s| match s {
                "*" => PatternSegment::SingleWildcard,
                "#" => PatternSegment::MultiWildcard,
                literal => PatternSegment::Literal(literal.to_string()),
            })
            .collect();

        Self { segments }
    }

    /// Check if topic matches this pattern.
    ///
    /// Uses recursive backtracking algorithm to match topic segments against
    /// pattern segments, handling wildcards correctly.
    ///
    /// # Parameters
    ///
    /// * `topic` - Topic name to test
    ///
    /// # Returns
    ///
    /// `true` if topic matches pattern, `false` otherwise
    ///
    /// # Algorithm
    ///
    /// 1. Split topic into segments
    /// 2. Recursively match pattern segments against topic segments
    /// 3. Handle wildcards:
    ///    - `*`: Consume exactly one topic segment
    ///    - `#`: Consume remaining topic segments (greedy match)
    ///
    /// # Performance
    ///
    /// Time complexity: O(n × m) worst case where:
    /// - n = number of pattern segments
    /// - m = number of topic segments
    ///
    /// Target: <50ns per operation
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let pattern = TopicPattern::parse("events.*.*");
    /// assert!(pattern.matches("events.user.login"));
    /// assert!(pattern.matches("events.system.restart"));
    /// assert!(!pattern.matches("events.user"));
    /// ```
    pub fn matches(&self, topic: &str) -> bool {
        let topic_segments: Vec<&str> = topic.split('.').collect();
        self.matches_segments(&topic_segments, 0, 0)
    }

    /// Recursive segment matching algorithm.
    ///
    /// Matches pattern segments against topic segments using backtracking
    /// for wildcard handling.
    ///
    /// # Parameters
    ///
    /// * `topic_segments` - Topic segments to match
    /// * `pattern_idx` - Current pattern segment index
    /// * `topic_idx` - Current topic segment index
    ///
    /// # Returns
    ///
    /// `true` if remaining segments match, `false` otherwise
    ///
    /// # Algorithm
    ///
    /// **Base Cases:**
    /// 1. Both exhausted → match
    /// 2. Pattern has `#` at end → match (consumes all remaining)
    /// 3. One exhausted, other not → no match
    ///
    /// **Recursive Cases:**
    /// 1. Literal: Must match exactly, recurse on next segments
    /// 2. `*`: Must consume exactly one segment, recurse on next
    /// 3. `#`: Consumes all remaining segments, match
    fn matches_segments(
        &self,
        topic_segments: &[&str],
        pattern_idx: usize,
        topic_idx: usize,
    ) -> bool {
        // Base case: both exhausted
        if pattern_idx >= self.segments.len() && topic_idx >= topic_segments.len() {
            return true;
        }

        // Base case: pattern exhausted but topic has segments
        if pattern_idx >= self.segments.len() {
            return false;
        }

        // Check current pattern segment
        match &self.segments[pattern_idx] {
            PatternSegment::MultiWildcard => {
                // # matches zero or more segments (consumes all remaining)
                // MQTT spec: # must be last segment in pattern
                true
            }
            PatternSegment::SingleWildcard => {
                // * matches exactly one segment
                if topic_idx >= topic_segments.len() {
                    // No segment available to match
                    return false;
                }
                // Consume one topic segment, advance both indices
                self.matches_segments(topic_segments, pattern_idx + 1, topic_idx + 1)
            }
            PatternSegment::Literal(literal) => {
                // Literal must match exactly
                if topic_idx >= topic_segments.len() {
                    return false;
                }
                if topic_segments[topic_idx] != literal {
                    return false;
                }
                // Advance both indices
                self.matches_segments(topic_segments, pattern_idx + 1, topic_idx + 1)
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Test code: unwrap is acceptable
mod tests {
    use super::*;

    #[test]
    fn test_literal_pattern_exact_match() {
        let pattern = TopicPattern::parse("events.user.login");
        assert!(pattern.matches("events.user.login"));
    }

    #[test]
    fn test_literal_pattern_no_match() {
        let pattern = TopicPattern::parse("events.user.login");
        assert!(!pattern.matches("events.user.logout"));
        assert!(!pattern.matches("events.system.restart"));
    }

    #[test]
    fn test_single_wildcard_match() {
        let pattern = TopicPattern::parse("events.user.*");
        assert!(pattern.matches("events.user.login"));
        assert!(pattern.matches("events.user.logout"));
        assert!(pattern.matches("events.user.register"));
    }

    #[test]
    fn test_single_wildcard_no_match_different_prefix() {
        let pattern = TopicPattern::parse("events.user.*");
        assert!(!pattern.matches("events.system.restart"));
        assert!(!pattern.matches("system.user.login"));
    }

    #[test]
    fn test_single_wildcard_no_match_too_many_segments() {
        let pattern = TopicPattern::parse("events.user.*");
        assert!(!pattern.matches("events.user.login.success"));
        assert!(!pattern.matches("events.user.login.failure.retry"));
    }

    #[test]
    fn test_single_wildcard_no_match_too_few_segments() {
        let pattern = TopicPattern::parse("events.user.*");
        assert!(!pattern.matches("events.user"));
        assert!(!pattern.matches("events"));
    }

    /// Test that multi-level wildcard `#` matches zero segments after the prefix.
    ///
    /// MQTT 3.1.1 §4.7.1.2: The multi-level wildcard matches any number of levels,
    /// **including zero**. Pattern `events.#` should match topic `events` because
    /// `#` is positioned after the delimiter and consumes zero segments.
    ///
    /// Edge Case: This verifies the "zero or more" semantics explicitly.
    #[test]
    fn test_multi_wildcard_match_zero_segments() {
        let pattern = TopicPattern::parse("events.#");
        assert!(pattern.matches("events"));
    }

    #[test]
    fn test_multi_wildcard_match_one_segment() {
        let pattern = TopicPattern::parse("events.#");
        assert!(pattern.matches("events.user"));
    }

    #[test]
    fn test_multi_wildcard_match_multiple_segments() {
        let pattern = TopicPattern::parse("events.#");
        assert!(pattern.matches("events.user.login"));
        assert!(pattern.matches("events.user.login.success"));
        assert!(pattern.matches("events.user.login.success.timestamp"));
    }

    #[test]
    fn test_multi_wildcard_no_match_different_prefix() {
        let pattern = TopicPattern::parse("events.#");
        assert!(!pattern.matches("system.restart"));
        assert!(!pattern.matches("metrics.cpu"));
    }

    #[test]
    fn test_multiple_single_wildcards() {
        let pattern = TopicPattern::parse("events.*.*");
        assert!(pattern.matches("events.user.login"));
        assert!(pattern.matches("events.system.restart"));
        assert!(!pattern.matches("events.user"));
        assert!(!pattern.matches("events.user.login.success"));
    }

    #[test]
    fn test_mixed_wildcards_and_literals() {
        let pattern = TopicPattern::parse("events.*.login");
        assert!(pattern.matches("events.user.login"));
        assert!(pattern.matches("events.admin.login"));
        assert!(!pattern.matches("events.user.logout"));
        assert!(!pattern.matches("events.user.login.success"));
    }

    #[test]
    fn test_filter_single_pattern() {
        let filter = TopicFilter::from_patterns(vec!["events.user.*"]);
        assert!(filter.matches("events.user.login"));
        assert!(filter.matches("events.user.logout"));
        assert!(!filter.matches("system.restart"));
    }

    #[test]
    fn test_filter_multiple_patterns() {
        let filter = TopicFilter::from_patterns(vec!["events.user.*", "system.#"]);
        assert!(filter.matches("events.user.login"));
        assert!(filter.matches("system.restart"));
        assert!(filter.matches("system.restart.initiated"));
        assert!(!filter.matches("metrics.cpu"));
    }

    #[test]
    fn test_filter_no_patterns() {
        let filter = TopicFilter::from_patterns(vec![]);
        assert!(!filter.matches("events.user.login"));
        assert!(!filter.matches("system.restart"));
    }

    #[test]
    fn test_filter_pattern_count() {
        let filter = TopicFilter::from_patterns(vec!["events.*", "system.#"]);
        assert_eq!(filter.pattern_count(), 2);
    }

    #[test]
    fn test_empty_topic() {
        let pattern = TopicPattern::parse("events");
        assert!(!pattern.matches(""));
    }

    #[test]
    fn test_single_segment_topic_and_pattern() {
        let pattern = TopicPattern::parse("events");
        assert!(pattern.matches("events"));
        assert!(!pattern.matches("system"));
    }

    #[test]
    fn test_multi_wildcard_at_root() {
        let pattern = TopicPattern::parse("#");
        assert!(pattern.matches("events"));
        assert!(pattern.matches("events.user"));
        assert!(pattern.matches("events.user.login"));
        assert!(pattern.matches("system.restart"));
    }
}
