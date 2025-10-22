//! OSL bridge integration abstractions.
//!
//! This module provides traits and types for bridging WASM components to the
//! AirsSys OS Layer (OSL). The bridge enables controlled access to host system
//! resources through capability-based security.
//!
//! # Architecture
//!
//! The bridge architecture follows these principles:
//!
//! - **Capability Mapping**: WASM capabilities map to OSL operations and permissions
//! - **Trait-Based Contract**: `HostFunction` trait defines host function behavior
//! - **Context Propagation**: `HostCallContext` carries security and identity information
//! - **Category Classification**: Host functions organized by domain (filesystem, network, etc.)
//!
//! # Examples
//!
//! ```rust
//! use airssys_wasm::core::{
//!     Capability, CapabilitySet, ComponentId, SecurityMode, PathPattern,
//!     bridge::{CapabilityMapping, HostCallContext, HostFunctionCategory}
//! };
//!
//! let mapping = CapabilityMapping {
//!     capability: Capability::FileRead(PathPattern::new("/data")),
//!     osl_operation: "filesystem::read".to_string(),
//!     osl_permissions: vec!["read".to_string()],
//! };
//!
//! let context = HostCallContext {
//!     component_id: ComponentId::new("my-component"),
//!     capabilities: CapabilitySet::new(),
//!     security_mode: SecurityMode::Strict,
//! };
//! ```
//!
//! # References
//!
//! - **ADR-WASM-012**: Comprehensive Core Abstractions Strategy
//! - **WASM-TASK-000 Phase 10**: Bridge abstractions design
//! - **Workspace Standards**: §6.1 (YAGNI), §6.2 (Avoid dyn)

use async_trait::async_trait;

use super::{Capability, CapabilitySet, ComponentId, SecurityMode, WasmError, WasmResult};

/// Host function trait for OSL bridge integration.
///
/// Implementors of this trait provide host-side functionality that WASM components
/// can invoke through the bridge. Each host function declares its required capability
/// and implements the execution logic.
///
/// # Security
///
/// Host functions are the security boundary between WASM components and the host system.
/// Implementations MUST:
///
/// - Validate all inputs thoroughly
/// - Check capability permissions before execution
/// - Implement proper resource cleanup
/// - Log security-relevant operations
///
/// # Examples
///
/// ```rust,ignore
/// use async_trait::async_trait;
/// use airssys_wasm::core::{Capability, WasmResult};
/// use airssys_wasm::core::bridge::{HostFunction, HostCallContext};
///
/// struct ReadFileFunction;
///
/// #[async_trait]
/// impl HostFunction for ReadFileFunction {
///     fn name(&self) -> &str {
///         "filesystem::read"
///     }
///     
///     fn required_capability(&self) -> Capability {
///         Capability::FileRead(PathPattern::new("/*"))
///     }
///     
///     async fn execute(
///         &self,
///         context: &HostCallContext,
///         args: Vec<u8>,
///     ) -> WasmResult<Vec<u8>> {
///         Ok(vec![])
///     }
/// }
/// ```
#[async_trait]
pub trait HostFunction: Send + Sync {
    /// Returns the fully-qualified function name.
    ///
    /// Function names follow the format: `{category}::{operation}`
    /// (e.g., "filesystem::read", "network::connect").
    fn name(&self) -> &str;

    /// Returns the capability required to invoke this function.
    ///
    /// The bridge validates that the calling component has this capability
    /// before allowing execution.
    fn required_capability(&self) -> Capability;

    /// Executes the host function with provided context and arguments.
    ///
    /// # Arguments
    ///
    /// - `context`: Security and identity context for the call
    /// - `args`: Serialized function arguments (format implementation-defined)
    ///
    /// # Returns
    ///
    /// Serialized function result on success, or `WasmError` on failure.
    ///
    /// # Errors
    ///
    /// - `WasmError::PermissionDenied`: Insufficient capabilities
    /// - `WasmError::InvalidInput`: Invalid argument format
    /// - `WasmError::ExecutionFailed`: Host operation failed
    async fn execute(
        &self,
        context: &HostCallContext,
        args: Vec<u8>,
    ) -> WasmResult<Vec<u8>>;
}

/// Capability to OSL permission mapping.
///
/// Maps a WASM capability to the corresponding OSL operation and its required
/// permissions. This enables the bridge to translate capability checks into
/// OSL security policy evaluations.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::{Capability, PathPattern, DomainPattern, bridge::CapabilityMapping};
///
/// let fs_read = CapabilityMapping {
///     capability: Capability::FileRead(PathPattern::new("/config")),
///     osl_operation: "filesystem::read".to_string(),
///     osl_permissions: vec!["read".to_string()],
/// };
///
/// let net_connect = CapabilityMapping {
///     capability: Capability::NetworkOutbound(DomainPattern::new("api.example.com:443")),
///     osl_operation: "network::connect".to_string(),
///     osl_permissions: vec!["connect".to_string(), "tls".to_string()],
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityMapping {
    /// WASM capability that grants access.
    pub capability: Capability,
    
    /// OSL operation identifier (e.g., "filesystem::read").
    pub osl_operation: String,
    
    /// OSL permissions required for this operation.
    pub osl_permissions: Vec<String>,
}

impl CapabilityMapping {
    /// Creates a new capability mapping.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::{Capability, PathPattern, bridge::CapabilityMapping};
    ///
    /// let mapping = CapabilityMapping::new(
    ///     Capability::FileWrite(PathPattern::new("/data")),
    ///     "filesystem::write",
    ///     vec!["write"],
    /// );
    /// ```
    pub fn new(
        capability: Capability,
        osl_operation: impl Into<String>,
        osl_permissions: Vec<impl Into<String>>,
    ) -> Self {
        Self {
            capability,
            osl_operation: osl_operation.into(),
            osl_permissions: osl_permissions.into_iter().map(Into::into).collect(),
        }
    }

    /// Checks if a capability set contains the required capability.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::{Capability, CapabilitySet, PathPattern, bridge::CapabilityMapping};
    ///
    /// let mapping = CapabilityMapping::new(
    ///     Capability::FileRead(PathPattern::new("/data")),
    ///     "filesystem::read",
    ///     vec!["read"],
    /// );
    ///
    /// let mut caps = CapabilitySet::new();
    /// caps.grant(Capability::FileRead(PathPattern::new("/data")));
    ///
    /// assert!(mapping.is_granted(&caps));
    /// ```
    pub fn is_granted(&self, capabilities: &CapabilitySet) -> bool {
        capabilities.has(&self.capability)
    }
}

/// Host function call context.
///
/// Provides the security and identity context for a host function invocation.
/// The context includes the calling component's identity, granted capabilities,
/// and the active security mode.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::{ComponentId, CapabilitySet, SecurityMode};
/// use airssys_wasm::core::bridge::HostCallContext;
///
/// let context = HostCallContext {
///     component_id: ComponentId::new("my-component"),
///     capabilities: CapabilitySet::new(),
///     security_mode: SecurityMode::Strict,
/// };
/// ```
#[derive(Debug, Clone)]
pub struct HostCallContext {
    /// Identity of the calling component.
    pub component_id: ComponentId,
    
    /// Capabilities granted to the component.
    pub capabilities: CapabilitySet,
    
    /// Active security mode for validation.
    pub security_mode: SecurityMode,
}

impl HostCallContext {
    /// Creates a new host call context.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::{ComponentId, CapabilitySet, SecurityMode};
    /// use airssys_wasm::core::bridge::HostCallContext;
    ///
    /// let context = HostCallContext::new(
    ///     ComponentId::new("my-component"),
    ///     CapabilitySet::new(),
    ///     SecurityMode::Permissive,
    /// );
    /// ```
    pub fn new(
        component_id: ComponentId,
        capabilities: CapabilitySet,
        security_mode: SecurityMode,
    ) -> Self {
        Self {
            component_id,
            capabilities,
            security_mode,
        }
    }

    /// Checks if the context has a specific capability.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::{Capability, ComponentId, CapabilitySet, SecurityMode, PathPattern};
    /// use airssys_wasm::core::bridge::HostCallContext;
    ///
    /// let mut caps = CapabilitySet::new();
    /// caps.grant(Capability::FileRead(PathPattern::new("/data")));
    ///
    /// let context = HostCallContext::new(
    ///     ComponentId::new("my-component"),
    ///     caps,
    ///     SecurityMode::Strict,
    /// );
    ///
    /// assert!(context.has_capability(&Capability::FileRead(PathPattern::new("/data"))));
    /// ```
    pub fn has_capability(&self, capability: &Capability) -> bool {
        self.capabilities.has(capability)
    }

    /// Validates that the context has required capability.
    ///
    /// # Errors
    ///
    /// Returns `WasmError::PermissionDenied` if the capability is missing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::{Capability, ComponentId, CapabilitySet, SecurityMode, PathPattern, DomainPattern};
    /// use airssys_wasm::core::bridge::HostCallContext;
    ///
    /// let mut caps = CapabilitySet::new();
    /// caps.grant(Capability::FileRead(PathPattern::new("/data")));
    ///
    /// let context = HostCallContext::new(
    ///     ComponentId::new("my-component"),
    ///     caps,
    ///     SecurityMode::Strict,
    /// );
    ///
    /// assert!(context.validate_capability(&Capability::FileRead(PathPattern::new("/data"))).is_ok());
    /// assert!(context.validate_capability(&Capability::NetworkOutbound(DomainPattern::new("*"))).is_err());
    /// ```
    pub fn validate_capability(&self, capability: &Capability) -> WasmResult<()> {
        if self.has_capability(capability) {
            Ok(())
        } else {
            Err(WasmError::capability_denied(
                capability.clone(),
                format!("Component {:?} lacks required capability", self.component_id)
            ))
        }
    }
}

/// Host function category classification.
///
/// Categorizes host functions by their domain to enable organized discovery,
/// documentation, and policy enforcement.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::bridge::HostFunctionCategory;
///
/// let category = HostFunctionCategory::Filesystem;
/// assert_eq!(category.as_str(), "filesystem");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HostFunctionCategory {
    /// Filesystem operations (read, write, delete).
    Filesystem,
    
    /// Network operations (connect, listen, send, receive).
    Network,
    
    /// Process operations (spawn, kill, signal).
    Process,
    
    /// Storage operations (get, set, delete, list).
    Storage,
    
    /// Messaging operations (publish, subscribe, send).
    Messaging,
    
    /// Logging operations (log, trace, audit).
    Logging,
}

impl HostFunctionCategory {
    /// Returns the category as a string identifier.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::bridge::HostFunctionCategory;
    ///
    /// assert_eq!(HostFunctionCategory::Filesystem.as_str(), "filesystem");
    /// assert_eq!(HostFunctionCategory::Network.as_str(), "network");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Filesystem => "filesystem",
            Self::Network => "network",
            Self::Process => "process",
            Self::Storage => "storage",
            Self::Messaging => "messaging",
            Self::Logging => "logging",
        }
    }

    /// Returns all available categories.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::bridge::HostFunctionCategory;
    ///
    /// let categories = HostFunctionCategory::all();
    /// assert_eq!(categories.len(), 6);
    /// ```
    pub fn all() -> Vec<Self> {
        vec![
            Self::Filesystem,
            Self::Network,
            Self::Process,
            Self::Storage,
            Self::Messaging,
            Self::Logging,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{PathPattern, DomainPattern};

    #[test]
    fn test_capability_mapping_new() {
        let mapping = CapabilityMapping::new(
            Capability::FileRead(PathPattern::new("/data")),
            "filesystem::read",
            vec!["read"],
        );

        assert_eq!(mapping.osl_operation, "filesystem::read");
        assert_eq!(mapping.osl_permissions, vec!["read"]);
    }

    #[test]
    fn test_capability_mapping_is_granted() {
        let mapping = CapabilityMapping::new(
            Capability::FileRead(PathPattern::new("/data")),
            "filesystem::read",
            vec!["read"],
        );

        let mut caps = CapabilitySet::new();
        assert!(!mapping.is_granted(&caps));

        caps.grant(Capability::FileRead(PathPattern::new("/data")));
        assert!(mapping.is_granted(&caps));
    }

    #[test]
    fn test_host_call_context_new() {
        let component_id = ComponentId::new("test-component");
        let caps = CapabilitySet::new();
        let security_mode = SecurityMode::Strict;

        let context = HostCallContext::new(
            component_id.clone(),
            caps.clone(),
            security_mode,
        );

        assert_eq!(context.component_id, component_id);
        assert_eq!(context.security_mode, SecurityMode::Strict);
    }

    #[test]
    fn test_host_call_context_has_capability() {
        let mut caps = CapabilitySet::new();
        caps.grant(Capability::FileRead(PathPattern::new("/data")));

        let context = HostCallContext::new(
            ComponentId::new("test-component"),
            caps,
            SecurityMode::Strict,
        );

        assert!(context.has_capability(&Capability::FileRead(PathPattern::new("/data"))));
        assert!(!context.has_capability(&Capability::NetworkOutbound(DomainPattern::new("*"))));
    }

    #[test]
    fn test_host_call_context_validate_capability() {
        let mut caps = CapabilitySet::new();
        caps.grant(Capability::FileRead(PathPattern::new("/data")));

        let context = HostCallContext::new(
            ComponentId::new("test-component"),
            caps,
            SecurityMode::Strict,
        );

        assert!(context.validate_capability(&Capability::FileRead(PathPattern::new("/data"))).is_ok());
        assert!(context.validate_capability(&Capability::NetworkOutbound(DomainPattern::new("*"))).is_err());
    }

    #[test]
    fn test_host_function_category_as_str() {
        assert_eq!(HostFunctionCategory::Filesystem.as_str(), "filesystem");
        assert_eq!(HostFunctionCategory::Network.as_str(), "network");
        assert_eq!(HostFunctionCategory::Process.as_str(), "process");
        assert_eq!(HostFunctionCategory::Storage.as_str(), "storage");
        assert_eq!(HostFunctionCategory::Messaging.as_str(), "messaging");
        assert_eq!(HostFunctionCategory::Logging.as_str(), "logging");
    }

    #[test]
    fn test_host_function_category_all() {
        let categories = HostFunctionCategory::all();
        assert_eq!(categories.len(), 6);
        assert!(categories.contains(&HostFunctionCategory::Filesystem));
        assert!(categories.contains(&HostFunctionCategory::Network));
        assert!(categories.contains(&HostFunctionCategory::Process));
        assert!(categories.contains(&HostFunctionCategory::Storage));
        assert!(categories.contains(&HostFunctionCategory::Messaging));
        assert!(categories.contains(&HostFunctionCategory::Logging));
    }
}
