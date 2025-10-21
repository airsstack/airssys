//! WIT interface abstractions for component metadata and validation.
//!
//! This module provides simplified abstractions for representing WebAssembly Interface
//! Type (WIT) interfaces at runtime. These types support interface validation and
//! capability-based security enforcement without duplicating wit-bindgen's compile-time
//! type system.
//!
//! # Design Rationale
//!
//! Following YAGNI principles (§6.1), this module provides **minimal** interface metadata.
//! Several abstractions were intentionally deferred after evidence-based analysis:
//!
//! - **TypeDescriptor**: wit-bindgen generates strongly-typed Rust bindings at compile-time,
//!   eliminating the need for runtime type introspection. Security validation only requires
//!   function names and capabilities, not full type signatures.
//!
//! - **InterfaceKind (Import/Export)**: The universal imports pattern (KNOWLEDGE-WASM-004)
//!   means all components have identical import/export structure. Interface directionality
//!   does not affect runtime behavior.
//!
//! - **BindingMetadata**: wit-bindgen runs at build time. Language binding metadata has
//!   no identified runtime consumer. Phase 1 is Rust-only; multi-language support is
//!   Block 10 Phase 2+ (Q2 2026+).
//!
//! See DEBT-WASM-001 for detailed rationale and re-evaluation criteria.
//!
//! # Architecture
//!
//! ```text
//! Compile-time:           Runtime:
//! ┌─────────────┐        ┌─────────────┐
//! │ WIT files   │        │ WitInterface│
//! └──────┬──────┘        └──────┬──────┘
//!        │                      │
//!        ▼                      ▼
//! ┌─────────────┐        ┌─────────────┐
//! │ wit-bindgen │        │  Security   │
//! │ (types)     │        │ Validation  │
//! └─────────────┘        └─────────────┘
//! ```
//!
//! - **Compile-time**: wit-bindgen generates Rust types from WIT definitions
//! - **Runtime**: WitInterface provides metadata for validation and security
//!
//! # Example
//!
//! ```rust
//! use airssys_wasm::core::{WitInterface, FunctionSignature, Capability};
//!
//! // Define interface metadata
//! let interface = WitInterface {
//!     name: "wasi:http/incoming-handler".to_string(),
//!     version: "0.2.0".to_string(),
//!     functions: vec![
//!         FunctionSignature {
//!             name: "handle".to_string(),
//!             required_capabilities: vec![
//!                 Capability::NetworkInbound(8080),
//!             ],
//!         },
//!     ],
//! };
//!
//! // Validate version compatibility
//! assert_eq!(interface.version, "0.2.0");
//!
//! // Check function capabilities
//! let handle_fn = &interface.functions[0];
//! assert_eq!(handle_fn.name, "handle");
//! assert!(!handle_fn.required_capabilities.is_empty());
//! ```

use serde::{Deserialize, Serialize};

use super::capability::Capability;

/// WIT interface metadata for version validation and capability checking.
///
/// This struct represents a WIT (WebAssembly Interface Type) interface,
/// providing the minimal information needed for runtime interface validation
/// and capability-based security enforcement.
///
/// # Design Rationale
///
/// This is a simplified abstraction following YAGNI principles (§6.1).
/// Type-level metadata (TypeDescriptor) was intentionally omitted because
/// wit-bindgen handles type bindings at compile-time, not runtime.
/// Interface directionality (InterfaceKind) was omitted because the universal
/// imports pattern means all components have identical import/export structure.
/// See DEBT-WASM-001 for deferred abstractions and re-evaluation criteria.
///
/// # Fields
///
/// - `name`: Interface name following WIT namespace conventions (e.g., `"wasi:http/incoming-handler"`)
/// - `version`: Semantic version for compatibility checking (e.g., `"0.2.0"`)
/// - `functions`: Functions exported by this interface with capability requirements
///
/// # Example
///
/// ```rust
/// use airssys_wasm::core::{WitInterface, FunctionSignature, Capability, DomainPattern};
///
/// let interface = WitInterface {
///     name: "wasi:http/incoming-handler".to_string(),
///     version: "0.2.0".to_string(),
///     functions: vec![
///         FunctionSignature {
///             name: "handle".to_string(),
///             required_capabilities: vec![
///                 Capability::NetworkInbound(8080),
///                 Capability::NetworkOutbound(DomainPattern::new("api.example.com")),
///             ],
///         },
///     ],
/// };
///
/// // Version validation
/// assert!(interface.version.starts_with("0.2"));
///
/// // Function lookup
/// let handle = interface.functions.iter()
///     .find(|f| f.name == "handle")
///     .expect("handle function not found");
/// assert_eq!(handle.required_capabilities.len(), 2);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WitInterface {
    /// Interface name following WIT namespace conventions.
    ///
    /// Format: `namespace:package/interface`
    ///
    /// # Examples
    /// - `"wasi:http/incoming-handler"` - WASI HTTP interface
    /// - `"wasi:filesystem/types"` - WASI filesystem types
    /// - `"custom:service/api"` - Custom application interface
    ///
    /// # Validation
    ///
    /// Names should follow WIT naming conventions:
    /// - Lowercase alphanumeric with hyphens
    /// - Colon separates namespace and package
    /// - Slash separates package and interface
    pub name: String,

    /// Semantic version for compatibility checking.
    ///
    /// Used for interface version validation during component loading.
    /// Follows semver specification (major.minor.patch).
    ///
    /// # Example
    /// ```rust
    /// # let version = "0.2.0";
    /// assert_eq!(version, "0.2.0");
    /// ```
    ///
    /// # Compatibility Rules
    ///
    /// - Major version mismatch: Incompatible
    /// - Minor version decrease: Potentially incompatible (missing features)
    /// - Patch version: Always compatible
    pub version: String,

    /// Functions exported by this interface.
    ///
    /// Each function specifies its capability requirements for
    /// security validation during execution.
    ///
    /// # Empty Vector
    ///
    /// An empty vector is valid for marker interfaces or interfaces
    /// that only define types without functions.
    pub functions: Vec<FunctionSignature>,
}

impl WitInterface {
    /// Create a new WIT interface with the given name and version.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::core::WitInterface;
    ///
    /// let interface = WitInterface::new("wasi:http/incoming-handler", "0.2.0");
    /// assert_eq!(interface.name, "wasi:http/incoming-handler");
    /// assert_eq!(interface.version, "0.2.0");
    /// assert!(interface.functions.is_empty());
    /// ```
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            functions: Vec::new(),
        }
    }

    /// Add a function to this interface.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::core::{WitInterface, FunctionSignature, Capability, NamespacePattern};
    ///
    /// let mut interface = WitInterface::new("custom:service/api", "1.0.0");
    /// interface.add_function(FunctionSignature {
    ///     name: "process".to_string(),
    ///     required_capabilities: vec![Capability::Storage(NamespacePattern::new("data"))],
    /// });
    ///
    /// assert_eq!(interface.functions.len(), 1);
    /// assert_eq!(interface.functions[0].name, "process");
    /// ```
    pub fn add_function(&mut self, function: FunctionSignature) {
        self.functions.push(function);
    }

    /// Find a function by name.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::core::{WitInterface, FunctionSignature};
    ///
    /// let mut interface = WitInterface::new("custom:service/api", "1.0.0");
    /// interface.add_function(FunctionSignature {
    ///     name: "process".to_string(),
    ///     required_capabilities: vec![],
    /// });
    ///
    /// let func = interface.find_function("process");
    /// assert!(func.is_some());
    /// assert_eq!(func.unwrap().name, "process");
    ///
    /// assert!(interface.find_function("nonexistent").is_none());
    /// ```
    pub fn find_function(&self, name: &str) -> Option<&FunctionSignature> {
        self.functions.iter().find(|f| f.name == name)
    }
}

/// Function signature with capability requirements.
///
/// Represents a single function in a WIT interface with its security
/// requirements. Used for capability-based permission checking during
/// component execution.
///
/// # Design Rationale
///
/// Parameter and return type metadata were intentionally omitted following
/// YAGNI analysis. wit-bindgen generates strongly-typed Rust bindings at
/// compile-time, eliminating the need for runtime type descriptors.
/// Security validation only requires function name and capabilities.
/// See DEBT-WASM-001 for detailed rationale.
///
/// # Fields
///
/// - `name`: Function name for identification and permission matching
/// - `required_capabilities`: Capabilities required to invoke this function
///
/// # Example
///
/// ```rust
/// use airssys_wasm::core::{FunctionSignature, Capability, PathPattern, DomainPattern};
///
/// let signature = FunctionSignature {
///     name: "process_file".to_string(),
///     required_capabilities: vec![
///         Capability::FileRead(PathPattern::new("/data/*")),
///         Capability::FileWrite(PathPattern::new("/output/*")),
///         Capability::NetworkOutbound(DomainPattern::new("api.example.com")),
///     ],
/// };
///
/// assert_eq!(signature.name, "process_file");
/// assert_eq!(signature.required_capabilities.len(), 3);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FunctionSignature {
    /// Function name for identification and permission matching.
    ///
    /// Used by security middleware to match capability requirements
    /// against granted permissions during execution.
    ///
    /// # Naming Conventions
    ///
    /// - Lowercase with underscores (snake_case)
    /// - Descriptive verb-noun pairs (e.g., `process_data`, `send_message`)
    /// - Follow WIT naming conventions
    ///
    /// # Example
    /// ```rust
    /// # let name = "handle";
    /// assert_eq!(name, "handle");
    /// // Other examples: "process_request", "send_notification"
    /// ```
    pub name: String,

    /// Capabilities required to invoke this function.
    ///
    /// Security middleware validates these capabilities against the
    /// component's granted CapabilitySet before allowing execution.
    /// Empty vector means function requires no special capabilities.
    ///
    /// # Example
    /// ```rust
    /// use airssys_wasm::core::{Capability, PathPattern, DomainPattern, NamespacePattern};
    ///
    /// let capabilities = vec![
    ///     Capability::FileRead(PathPattern::new("/data/*")),
    ///     Capability::NetworkOutbound(DomainPattern::new("*.api.com")),
    ///     Capability::Storage(NamespacePattern::new("data")),
    /// ];
    /// ```
    ///
    /// # Empty Capabilities
    ///
    /// Functions with no capabilities can be invoked by any component
    /// without permission checks (e.g., pure computation functions).
    pub required_capabilities: Vec<Capability>,
}

impl FunctionSignature {
    /// Create a new function signature with the given name.
    ///
    /// Creates a function with no required capabilities.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::core::FunctionSignature;
    ///
    /// let func = FunctionSignature::new("compute");
    /// assert_eq!(func.name, "compute");
    /// assert!(func.required_capabilities.is_empty());
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            required_capabilities: Vec::new(),
        }
    }

    /// Create a function signature with required capabilities.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::core::{FunctionSignature, Capability, PathPattern};
    ///
    /// let func = FunctionSignature::with_capabilities(
    ///     "read_config",
    ///     vec![Capability::FileRead(PathPattern::new("/config/*"))],
    /// );
    ///
    /// assert_eq!(func.name, "read_config");
    /// assert_eq!(func.required_capabilities.len(), 1);
    /// ```
    pub fn with_capabilities(
        name: impl Into<String>,
        capabilities: Vec<Capability>,
    ) -> Self {
        Self {
            name: name.into(),
            required_capabilities: capabilities,
        }
    }

    /// Add a required capability to this function.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::core::{FunctionSignature, Capability, NamespacePattern};
    ///
    /// let mut func = FunctionSignature::new("process");
    /// func.add_capability(Capability::Storage(NamespacePattern::new("data")));
    ///
    /// assert_eq!(func.required_capabilities.len(), 1);
    /// ```
    pub fn add_capability(&mut self, capability: Capability) {
        self.required_capabilities.push(capability);
    }

    /// Check if this function requires no capabilities.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::core::{FunctionSignature, Capability, NamespacePattern};
    ///
    /// let func1 = FunctionSignature::new("compute");
    /// assert!(func1.requires_no_capabilities());
    ///
    /// let func2 = FunctionSignature::with_capabilities(
    ///     "read",
    ///     vec![Capability::Storage(NamespacePattern::new("data"))],
    /// );
    /// assert!(!func2.requires_no_capabilities());
    /// ```
    pub fn requires_no_capabilities(&self) -> bool {
        self.required_capabilities.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::capability::{Capability, PathPattern, NamespacePattern, TopicPattern};

    #[test]
    fn test_wit_interface_creation() {
        let interface = WitInterface::new("wasi:http/incoming-handler", "0.2.0");
        assert_eq!(interface.name, "wasi:http/incoming-handler");
        assert_eq!(interface.version, "0.2.0");
        assert!(interface.functions.is_empty());
    }

    #[test]
    fn test_wit_interface_add_function() {
        let mut interface = WitInterface::new("custom:service/api", "1.0.0");
        interface.add_function(FunctionSignature::new("process"));

        assert_eq!(interface.functions.len(), 1);
        assert_eq!(interface.functions[0].name, "process");
    }

    #[test]
    fn test_wit_interface_find_function() {
        let mut interface = WitInterface::new("custom:service/api", "1.0.0");
        interface.add_function(FunctionSignature::new("process"));
        interface.add_function(FunctionSignature::new("validate"));

        let func = interface.find_function("process");
        assert!(func.is_some());
        if let Some(f) = func {
            assert_eq!(f.name, "process");
        }

        assert!(interface.find_function("nonexistent").is_none());
    }

    #[test]
    fn test_function_signature_creation() {
        let func = FunctionSignature::new("handle");
        assert_eq!(func.name, "handle");
        assert!(func.required_capabilities.is_empty());
        assert!(func.requires_no_capabilities());
    }

    #[test]
    fn test_function_signature_with_capabilities() {
        let func = FunctionSignature::with_capabilities(
            "read_file",
            vec![Capability::FileRead(PathPattern::new("data/*"))],
        );

        assert_eq!(func.name, "read_file");
        assert_eq!(func.required_capabilities.len(), 1);
        assert!(!func.requires_no_capabilities());
    }

    #[test]
    fn test_function_signature_add_capability() {
        let mut func = FunctionSignature::new("process");
        assert!(func.requires_no_capabilities());

        func.add_capability(Capability::Storage(NamespacePattern::new("config")));
        assert_eq!(func.required_capabilities.len(), 1);
        assert!(!func.requires_no_capabilities());

        func.add_capability(Capability::Messaging(TopicPattern::new("events/*")));
        assert_eq!(func.required_capabilities.len(), 2);
    }

    #[test]
    fn test_wit_interface_serialization() {
        let mut interface = WitInterface::new("wasi:http/incoming-handler", "0.2.0");
        interface.add_function(FunctionSignature::with_capabilities(
            "handle",
            vec![Capability::NetworkInbound(8080)],
        ));

        // Test JSON serialization
        match serde_json::to_string(&interface) {
            Ok(json) => match serde_json::from_str::<WitInterface>(&json) {
                Ok(deserialized) => {
                    assert_eq!(deserialized, interface);
                    assert_eq!(deserialized.name, "wasi:http/incoming-handler");
                    assert_eq!(deserialized.functions.len(), 1);
                }
                Err(e) => assert!(false, "deserialization failed: {e}"),
            },
            Err(e) => assert!(false, "serialization failed: {e}"),
        }
    }

    #[test]
    fn test_function_signature_clone() {
        let func = FunctionSignature::with_capabilities(
            "process",
            vec![Capability::Storage(NamespacePattern::new("data"))],
        );

        let cloned = func.clone();
        assert_eq!(cloned.name, "process");
        assert_eq!(cloned.required_capabilities.len(), 1);
    }

    #[test]
    fn test_wit_interface_multiple_functions() {
        let mut interface = WitInterface::new("custom:service/api", "1.0.0");

        interface.add_function(FunctionSignature::with_capabilities(
            "read",
            vec![Capability::FileRead(PathPattern::new("data/*"))],
        ));

        interface.add_function(FunctionSignature::with_capabilities(
            "write",
            vec![Capability::FileWrite(PathPattern::new("output/*"))],
        ));

        interface.add_function(FunctionSignature::new("compute"));

        assert_eq!(interface.functions.len(), 3);

        if let Some(read_fn) = interface.find_function("read") {
            assert_eq!(read_fn.required_capabilities.len(), 1);
        } else {
            assert!(false, "read function not found");
        }

        if let Some(compute_fn) = interface.find_function("compute") {
            assert!(compute_fn.requires_no_capabilities());
        } else {
            assert!(false, "compute function not found");
        }
    }
}
