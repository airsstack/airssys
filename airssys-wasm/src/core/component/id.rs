// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)
use std::fmt;

/// Unique identifier for a component instance.
///
/// A ComponentId consists of three parts:
/// - `namespace`: Logical grouping (e.g., "system", "user", "app")
/// - `name`: Component type name (e.g., "database", "cache", "auth")
/// - `instance`: Specific instance identifier (e.g., "v1", "prod", "dev")
///
/// This three-part structure enables hierarchical organization and prevents
/// naming collisions across different namespaces.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::component::ComponentId;
///
/// // Create a component ID for a production database instance
/// let id = ComponentId::new("system", "database", "prod");
///
/// // Format as string: "system/database/prod"
/// assert_eq!(id.to_string_id(), "system/database/prod");
///
/// // Display trait also uses formatted output
/// assert_eq!(format!("{}", id), "system/database/prod");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComponentId {
    /// Logical namespace for grouping components
    pub namespace: String,
    /// Component type name
    pub name: String,
    /// Specific instance identifier
    pub instance: String,
}

impl ComponentId {
    /// Creates a new ComponentId from namespace, name, and instance parts.
    ///
    /// Each parameter accepts any type that can be converted into a String,
    /// providing flexibility for string literals, String types, or other
    /// string-like types.
    ///
    /// # Arguments
    ///
    /// * `namespace` - Logical grouping (e.g., "system", "user")
    /// * `name` - Component type name (e.g., "database", "cache")
    /// * `instance` - Instance identifier (e.g., "v1", "prod")
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::component::ComponentId;
    ///
    /// // Using string literals
    /// let id1 = ComponentId::new("system", "database", "prod");
    ///
    /// // Using String types
    /// let ns = String::from("user");
    /// let id2 = ComponentId::new(ns, "cache", "dev");
    /// ```
    pub fn new(
        namespace: impl Into<String>,
        name: impl Into<String>,
        instance: impl Into<String>,
    ) -> Self {
        Self {
            namespace: namespace.into(),
            name: name.into(),
            instance: instance.into(),
        }
    }

    /// Formats the ComponentId as a string in "{namespace}/{name}/{instance}" format.
    ///
    /// This format is used for serialization, logging, and display purposes.
    /// The Display trait also uses this format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::component::ComponentId;
    ///
    /// let id = ComponentId::new("system", "database", "prod");
    /// assert_eq!(id.to_string_id(), "system/database/prod");
    /// ```
    pub fn to_string_id(&self) -> String {
        format!("{}/{}/{}", self.namespace, self.name, self.instance)
    }
}

impl fmt::Display for ComponentId {
    /// Formats the ComponentId using the "{namespace}/{name}/{instance}" format.
    ///
    /// This implementation is equivalent to `to_string_id()`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_id_creation_with_valid_strings() {
        let id = ComponentId::new("system", "database", "prod");

        assert_eq!(id.namespace, "system");
        assert_eq!(id.name, "database");
        assert_eq!(id.instance, "prod");
    }

    #[test]
    fn test_to_string_id_format() {
        let id = ComponentId::new("ns1", "name1", "instance1");
        assert_eq!(id.to_string_id(), "ns1/name1/instance1");
    }

    #[test]
    fn test_display_trait_matches_to_string_id() {
        let id = ComponentId::new("test", "comp", "1");
        assert_eq!(format!("{}", id), id.to_string_id());
    }

    #[test]
    fn test_partial_equality() {
        let id1 = ComponentId::new("system", "database", "prod");
        let id2 = ComponentId::new("system", "database", "prod");
        let id3 = ComponentId::new("system", "database", "dev");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_hash_trait_enables_hashmap_usage() {
        use std::collections::HashMap;

        let id1 = ComponentId::new("ns", "comp", "1");
        let id2 = ComponentId::new("ns", "comp", "2");

        let mut map = HashMap::new();
        map.insert(id1.clone(), "value1");
        map.insert(id2.clone(), "value2");

        assert_eq!(map.get(&id1), Some(&"value1"));
        assert_eq!(map.get(&id2), Some(&"value2"));
    }

    #[test]
    fn test_edge_cases_empty_strings() {
        // Test with empty namespace (edge case, not recommended but allowed)
        let id = ComponentId::new("", "comp", "1");
        assert_eq!(id.to_string_id(), "/comp/1");

        // Test with empty name
        let id = ComponentId::new("ns", "", "1");
        assert_eq!(id.to_string_id(), "ns//1");

        // Test with empty instance
        let id = ComponentId::new("ns", "comp", "");
        assert_eq!(id.to_string_id(), "ns/comp/");
    }

    #[test]
    fn test_edge_cases_special_characters() {
        // Test with special characters (allowed in strings)
        let id = ComponentId::new("my-ns", "my_comp", "instance.1");
        assert_eq!(id.to_string_id(), "my-ns/my_comp/instance.1");
    }

    #[test]
    fn test_clone_creates_independent_copy() {
        let id1 = ComponentId::new("ns", "comp", "1");
        let id2 = id1.clone();

        assert_eq!(id1, id2);

        // Verify independence by reassigning id1 to a new value
        // The clone id2 should retain its original value
        let id1 = ComponentId::new("ns", "comp", "2");
        assert_eq!(id1.to_string_id(), "ns/comp/2");
        assert_eq!(id2.to_string_id(), "ns/comp/1");
    }
}
