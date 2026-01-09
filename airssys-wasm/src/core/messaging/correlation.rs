//! Correlation ID types for request-response patterns.
//!
//! This module contains the `CorrelationId` type used to track and correlate
//! request-response pairs in inter-component messaging.

// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)
use std::fmt;

// Layer 2: Third-party crate imports (per PROJECTS_STANDARD.md §2.1)
use uuid::Uuid;

// Layer 3: Internal module imports (per PROJECTS_STANDARD.md §2.1)
// (none)

/// Unique identifier for correlating request-response pairs.
///
/// `CorrelationId` is used to track pending requests and match them with
/// their corresponding responses. Each request generates a unique correlation
/// ID that is included in the message metadata.
///
/// # Features
///
/// - Generate new unique IDs using UUID v4
/// - Create from existing string identifiers
/// - Hashable for use in collections
/// - Cloneable for passing between functions
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::messaging::correlation::CorrelationId;
///
/// // Generate a new unique correlation ID
/// let id = CorrelationId::generate();
/// assert!(!id.as_str().is_empty());
///
/// // Create from existing string
/// let id2 = CorrelationId::new("my-correlation-123");
/// assert_eq!(id2.as_str(), "my-correlation-123");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CorrelationId(String);

impl CorrelationId {
    /// Creates a new `CorrelationId` from a string.
    ///
    /// # Arguments
    ///
    /// * `id` - Any type that can be converted into a `String`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::messaging::correlation::CorrelationId;
    ///
    /// let id = CorrelationId::new("test-123");
    /// assert_eq!(id.as_str(), "test-123");
    ///
    /// // Also works with String
    /// let id2 = CorrelationId::new(String::from("test-456"));
    /// assert_eq!(id2.as_str(), "test-456");
    /// ```
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Generates a new unique `CorrelationId` using UUID v4.
    ///
    /// The generated ID is guaranteed to be unique across all instances
    /// within the same runtime.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::messaging::correlation::CorrelationId;
    ///
    /// let id1 = CorrelationId::generate();
    /// let id2 = CorrelationId::generate();
    ///
    /// // Each generated ID is unique
    /// assert_ne!(id1, id2);
    ///
    /// // Generated IDs are valid UUID v4 format (36 chars with dashes)
    /// assert_eq!(id1.as_str().len(), 36);
    /// ```
    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Returns the correlation ID as a string slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::messaging::correlation::CorrelationId;
    ///
    /// let id = CorrelationId::new("my-id");
    /// assert_eq!(id.as_str(), "my-id");
    /// ```
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for CorrelationId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for CorrelationId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_new_from_string() {
        let id = CorrelationId::new("test-123");
        assert_eq!(id.as_str(), "test-123");
    }

    #[test]
    fn test_new_from_owned_string() {
        let id = CorrelationId::new(String::from("owned-string"));
        assert_eq!(id.as_str(), "owned-string");
    }

    #[test]
    fn test_generate_unique() {
        let id1 = CorrelationId::generate();
        let id2 = CorrelationId::generate();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_is_valid_uuid() {
        let id = CorrelationId::generate();
        // UUID v4 format: 8-4-4-4-12 = 36 characters with dashes
        assert_eq!(id.as_str().len(), 36);
        assert!(id.as_str().contains('-'));
    }

    #[test]
    fn test_display_trait() {
        let id = CorrelationId::new("display-test");
        assert_eq!(format!("{}", id), "display-test");
    }

    #[test]
    fn test_from_string() {
        let id: CorrelationId = String::from("from-string").into();
        assert_eq!(id.as_str(), "from-string");
    }

    #[test]
    fn test_from_str() {
        let id: CorrelationId = "from-str".into();
        assert_eq!(id.as_str(), "from-str");
    }

    #[test]
    fn test_hash_and_eq() {
        let id1 = CorrelationId::new("same");
        let id2 = CorrelationId::new("same");

        assert_eq!(id1, id2);

        let mut set = HashSet::new();
        set.insert(id1.clone());
        assert!(set.contains(&id2));
    }

    #[test]
    fn test_clone_creates_independent_copy() {
        let id1 = CorrelationId::new("original");
        let id2 = id1.clone();

        assert_eq!(id1, id2);
        assert_eq!(id1.as_str(), id2.as_str());
    }

    #[test]
    fn test_debug_format() {
        let id = CorrelationId::new("debug-test");
        let debug_str = format!("{:?}", id);
        assert!(debug_str.contains("CorrelationId"));
        assert!(debug_str.contains("debug-test"));
    }

    #[test]
    fn test_empty_id() {
        let id = CorrelationId::new("");
        assert!(id.as_str().is_empty());
    }
}
