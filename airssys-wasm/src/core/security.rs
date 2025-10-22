//! Security abstractions for WASM component isolation and policy enforcement.
//!
//! These types define the security policy trait, permission checking workflow,
//! context for decisions, trust levels, and isolation boundaries needed for
//! Block 4 (Security & Isolation Layer). They provide the foundation for
//! capability-based security without implementation details, following YAGNI
//! principles (§6.1).
//!
//! # Design Rationale
//!
//! - **SecurityPolicy**: Trait contract for pluggable policies (e.g., RBAC, ACL).
//!   Async check_permission for non-blocking validation; granted_capabilities for
//!   pre-fetching permissions.
//! - **PermissionRequest/Result**: Explicit workflow for capability checks.
//! - **SecurityContext**: Combines runtime mode with trust assessment.
//! - **TrustLevel**: Simple enum for source-based trust (e.g., signed vs unsigned).
//! - **IsolationBoundary**: Defines sandbox configuration for components.
//!
//! All types integrate with core::Capability and core::config::SecurityMode.
//! No internal dependencies beyond core (zero circular deps).
//!
//! # References
//!
//! - ADR-WASM-005: Capability-Based Security Model
//! - ADR-WASM-006: Component Isolation and Sandboxing
//! - KNOWLEDGE-WASM-001: Component Framework Architecture (security layer)
//! - Workspace Standards: §6.2 (no dyn), M-ERRORS-CANONICAL-STRUCTS

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::core::{Capability, CapabilitySet, ComponentId, PathPattern, DomainPattern, SecurityMode, WasmResult};

/// Security policy enforcement trait.
///
/// Implementations provide pluggable security logic (e.g., file-based policies,
/// database-backed RBAC). Used in Block 4 for middleware that wraps component
/// execution, validating capabilities before allowing operations.
///
/// The trait is async for non-blocking checks (e.g., external auth services)
/// and Send + Sync for use in actor systems.
///
/// # Example
///
/// ```
/// use airssys_wasm::core::security::{SecurityPolicy, PermissionResult};
/// use airssys_wasm::core::component::ComponentId;
/// use airssys_wasm::core::capability::{Capability, CapabilitySet};
/// use airssys_wasm::core::error::WasmResult;
/// use async_trait::async_trait;
///
/// struct AllowAllPolicy;
///
/// #[async_trait]
/// impl SecurityPolicy for AllowAllPolicy {
///     async fn check_permission(
///         &self,
///         _component_id: &ComponentId,
///         _capability: &Capability,
///     ) -> WasmResult<PermissionResult> {
///         Ok(PermissionResult::Allowed)
///     }
///
///     fn granted_capabilities(&self, _component_id: &ComponentId) -> CapabilitySet {
///         CapabilitySet::default()
///     }
/// }
/// ```
#[async_trait]
pub trait SecurityPolicy: Send + Sync {
    /// Check if a capability is allowed for the component.
    ///
    /// Returns PermissionResult indicating allow/deny/review. Implementations
    /// should be fast (<5μs target) and log denials for audit.
    async fn check_permission(
        &self,
        component_id: &ComponentId,
        capability: &Capability,
    ) -> WasmResult<PermissionResult>;

    /// Get all granted capabilities for the component.
    ///
    /// Used for pre-loading permissions during component init. Returns a
    /// CapabilitySet that can be used for fast matching in check_permission.
    fn granted_capabilities(&self, component_id: &ComponentId) -> CapabilitySet;
}

/// Permission check request structure.
///
/// Bundles the capability request with context for policy decisions.
/// Passed to SecurityPolicy::check_permission.
///
/// # Example
///
/// ```
/// use airssys_wasm::core::capability::{Capability, PathPattern};
/// use airssys_wasm::core::component::ComponentId;
/// use airssys_wasm::core::config::SecurityMode;
/// use airssys_wasm::core::security::{PermissionRequest, SecurityContext, TrustLevel};
///
/// let request = PermissionRequest {
///     component_id: ComponentId::new("my-component"),
///     capability: Capability::FileRead(PathPattern::new("/data/*")),
///     context: SecurityContext {
///         mode: SecurityMode::Strict,
///         trust_level: TrustLevel::Trusted,
///         audit_enabled: true,
///     },
/// };
/// ```
#[derive(Debug, Clone)]
pub struct PermissionRequest {
    /// The component requesting the capability.
    pub component_id: ComponentId,

    /// The capability being requested.
    pub capability: Capability,

    /// Security context for the request.
    pub context: SecurityContext,
}

/// Result of a permission check.
///
/// Indicates whether the capability is allowed, denied (with reason), or needs
/// manual review (e.g., unknown component). Used by SecurityPolicy implementations
/// to communicate outcomes to the runtime.
///
/// # Example
///
/// ```
/// use airssys_wasm::core::security::PermissionResult;
///
/// let result = PermissionResult::Denied {
///     reason: "Component lacks file read capability".to_string(),
/// };
///
/// match result {
///     PermissionResult::Allowed => println!("Access granted"),
///     PermissionResult::Denied { reason } => println!("Access denied: {}", reason),
///     PermissionResult::NeedsReview => println!("Manual review required"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionResult {
    /// Capability is explicitly allowed.
    Allowed,

    /// Capability is denied (with human-readable reason).
    Denied {
        /// Reason for denial (logged for audit).
        reason: String,
    },

    /// Capability needs manual/admin review (e.g., first-time access).
    NeedsReview,
}

impl PermissionResult {
    /// Create a denied result with reason.
    pub fn denied(reason: impl Into<String>) -> Self {
        Self::Denied {
            reason: reason.into(),
        }
    }

    /// Check if permission is granted.
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed)
    }

    /// Get the denial reason if denied.
    pub fn denial_reason(&self) -> Option<&str> {
        match self {
            Self::Denied { reason } => Some(reason),
            _ => None,
        }
    }
}

/// Security context for permission decisions.
///
/// Provides runtime information (mode, trust, audit) to influence policy
/// outcomes. Passed in PermissionRequest to SecurityPolicy::check_permission.
///
/// # Example
///
/// ```
/// use airssys_wasm::core::config::SecurityMode;
/// use airssys_wasm::core::security::{SecurityContext, TrustLevel};
///
/// let context = SecurityContext {
///     mode: SecurityMode::Strict,
///     trust_level: TrustLevel::Trusted,
///     audit_enabled: true,
/// };
///
/// assert_eq!(context.mode, SecurityMode::Strict);
/// ```
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// Current security enforcement mode.
    pub mode: SecurityMode,

    /// Trust level of the component (based on source/signature).
    pub trust_level: TrustLevel,

    /// Whether audit logging is enabled for this context.
    pub audit_enabled: bool,
}

impl SecurityContext {
    /// Create a strict context for trusted components.
    pub fn strict_trusted(audit: bool) -> Self {
        Self {
            mode: SecurityMode::Strict,
            trust_level: TrustLevel::Trusted,
            audit_enabled: audit,
        }
    }

    /// Check if context allows permissive decisions (e.g., dev mode).
    pub fn is_permissive(&self) -> bool {
        matches!(self.mode, SecurityMode::Permissive | SecurityMode::Development)
    }

    /// Check if component is trusted (auto-approves some capabilities).
    pub fn is_trusted(&self) -> bool {
        matches!(self.trust_level, TrustLevel::Trusted)
    }
}

/// Component trust level for security decisions.
///
/// Determines baseline permissions based on component source (e.g., signed vs
/// unsigned). Higher trust levels may auto-approve common capabilities.
///
/// # Example
///
/// ```
/// use airssys_wasm::core::security::TrustLevel;
///
/// let level = TrustLevel::Trusted;
/// assert!(level.is_trusted());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustLevel {
    /// Fully trusted (e.g., signed by known publisher) - auto-approves common ops.
    Trusted,

    /// Unknown trust (e.g., unsigned local file) - requires explicit grants.
    Unknown,

    /// Development mode - bypasses checks (DEV ONLY, not for production).
    Development,
}

impl TrustLevel {
    /// Check if level is trusted (auto-approves).
    pub fn is_trusted(&self) -> bool {
        matches!(self, Self::Trusted)
    }

    /// Check if level bypasses security (development only).
    pub fn bypasses_security(&self) -> bool {
        matches!(self, Self::Development)
    }
}

/// Isolation boundary definition for component sandboxes.
///
/// Specifies which resources are isolated and what access is allowed.
/// Used in Block 4 to configure per-component sandboxes via Wasmtime config.
///
/// # Example
///
/// ```
/// use airssys_wasm::core::capability::{DomainPattern, PathPattern};
/// use airssys_wasm::core::security::IsolationBoundary;
///
/// let boundary = IsolationBoundary {
///     memory_isolated: true,
///     filesystem_isolated: true,
///     network_isolated: true,
///     allowed_paths: vec![PathPattern::new("/tmp/*")],
///     allowed_domains: vec![DomainPattern::new("*.example.com")],
/// };
///
/// assert!(boundary.memory_isolated);
/// ```
#[derive(Debug, Clone)]
pub struct IsolationBoundary {
    /// Whether memory is fully isolated (no shared regions).
    pub memory_isolated: bool,

    /// Whether filesystem access is sandboxed.
    pub filesystem_isolated: bool,

    /// Whether network access is restricted.
    pub network_isolated: bool,

    /// Allowed filesystem paths (glob patterns).
    pub allowed_paths: Vec<PathPattern>,

    /// Allowed network domains (wildcard patterns).
    pub allowed_domains: Vec<DomainPattern>,
}

impl IsolationBoundary {
    /// Create a fully isolated boundary (no access by default).
    pub fn fully_isolated() -> Self {
        Self {
            memory_isolated: true,
            filesystem_isolated: true,
            network_isolated: true,
            allowed_paths: Vec::new(),
            allowed_domains: Vec::new(),
        }
    }

    /// Add a filesystem path allowance.
    pub fn allow_path(&mut self, pattern: PathPattern) {
        self.allowed_paths.push(pattern);
    }

    /// Add a network domain allowance.
    pub fn allow_domain(&mut self, pattern: DomainPattern) {
        self.allowed_domains.push(pattern);
    }

    /// Check if boundary allows any filesystem access.
    pub fn allows_filesystem(&self) -> bool {
        !self.allowed_paths.is_empty()
    }

    /// Check if boundary allows any network access.
    pub fn allows_network(&self) -> bool {
        !self.allowed_domains.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_permission_result() {
        let allowed = PermissionResult::Allowed;
        assert!(allowed.is_allowed());

        let denied = PermissionResult::denied("Access forbidden");
        assert!(!denied.is_allowed());
        assert_eq!(denied.denial_reason(), Some("Access forbidden"));

        let review = PermissionResult::NeedsReview;
        assert!(!review.is_allowed());
        assert_eq!(review.denial_reason(), None);
    }

    #[test]
    fn test_security_context() {
        let ctx = SecurityContext::strict_trusted(true);
        assert_eq!(ctx.mode, SecurityMode::Strict);
        assert!(ctx.is_trusted());
        assert!(!ctx.is_permissive());
    }

    #[test]
    fn test_trust_level() {
        assert!(TrustLevel::Trusted.is_trusted());
        assert!(!TrustLevel::Unknown.is_trusted());
        assert!(TrustLevel::Development.bypasses_security());
        assert!(!TrustLevel::Trusted.bypasses_security());
    }

    #[test]
    fn test_isolation_boundary() {
        let mut boundary = IsolationBoundary::fully_isolated();
        assert!(boundary.memory_isolated);
        assert!(!boundary.allows_filesystem());
        assert!(!boundary.allows_network());

        boundary.allow_path(PathPattern::new("/tmp/*"));
        assert!(boundary.allows_filesystem());

        boundary.allow_domain(DomainPattern::new("*.local"));
        assert!(boundary.allows_network());
    }

    // Mock policy for trait testing
    struct MockPolicy {
        allow_all: bool,
    }

    #[async_trait]
    impl SecurityPolicy for MockPolicy {
        async fn check_permission(
            &self,
            _component_id: &ComponentId,
            _capability: &Capability,
        ) -> WasmResult<PermissionResult> {
            if self.allow_all {
                Ok(PermissionResult::Allowed)
            } else {
                Ok(PermissionResult::denied("Mock denial"))
            }
        }

        fn granted_capabilities(&self, _component_id: &ComponentId) -> CapabilitySet {
            CapabilitySet::default()
        }
    }

    #[tokio::test]
    #[allow(clippy::expect_used)]
    async fn test_security_policy() {
        let policy = MockPolicy { allow_all: true };
        let component = ComponentId::new("test");
        let cap = Capability::FileRead(PathPattern::new("/test"));

        let result = policy.check_permission(&component, &cap).await.expect("check_permission should succeed");
        assert!(result.is_allowed());

        let deny_policy = MockPolicy { allow_all: false };
        let deny_result = deny_policy.check_permission(&component, &cap).await.expect("check_permission should succeed");
        assert!(!deny_result.is_allowed());
    }

    #[test]
    fn test_permission_request_creation() {
        let request = PermissionRequest {
            component_id: ComponentId::new("comp-001"),
            capability: Capability::NetworkOutbound(DomainPattern::new("api.example.com")),
            context: SecurityContext {
                mode: SecurityMode::Strict,
                trust_level: TrustLevel::Trusted,
                audit_enabled: true,
            },
        };

        assert_eq!(request.component_id.as_str(), "comp-001");
        assert!(request.context.is_trusted());
    }
}
