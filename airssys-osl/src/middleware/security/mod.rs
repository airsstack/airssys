//! Security middleware module for comprehensive security validation and policy enforcement.
//!
//! This module provides a complete security middleware implementation with policy-based
//! access control, security validation, and comprehensive audit logging. All security
//! enforcement happens in this middleware layer before operations reach executors.
//!
//! # Architecture
//!
//! The security middleware follows a layered architecture:
//!
//! - **Policy Framework** - Abstract policy evaluation engine with ACL and RBAC support
//! - **Access Control** - ACL (Access Control Lists) and RBAC (Role-Based Access Control)
//! - **Audit Logging** - Comprehensive security event logging and audit trails
//! - **Middleware Integration** - SecurityMiddleware implementing the Middleware trait
//!
//! # Security Model
//!
//! - **Deny-by-Default**: All operations are denied unless explicitly allowed by policy
//! - **Priority 100**: Security middleware runs FIRST before all other middleware
//! - **Comprehensive Audit**: All security decisions are logged for compliance
//! - **Policy Composition**: Multiple policies can be active simultaneously
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use airssys_osl::middleware::security::{
//!     SecurityMiddleware, SecurityMiddlewareBuilder,
//! };
//! use airssys_osl::core::security::SecurityConfig;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Build security middleware (Phase 1 - basic structure)
//! let security = SecurityMiddlewareBuilder::new()
//!     .with_config(SecurityConfig::default())
//!     .build()?;
//!
//! // Use in middleware pipeline (priority 100 - runs first)
//! // TODO: Phase 2 will add ACL/RBAC policy support
//! // middleware_pipeline.add(security);
//! # Ok(())
//! # }
//! ```
//!
//! # Module Organization
//!
//! This module follows workspace standard §4.3 - mod.rs files contain ONLY
//! module declarations and re-exports, with implementation in separate files:
//!
//! - [`policy`] - Security policy framework and evaluation engine
//! - [`acl`] - Access Control Lists implementation
//! - [`rbac`] - Role-Based Access Control implementation
//! - [`audit`] - Security audit logging framework
//! - [`middleware`] - SecurityMiddleware implementation

// Layer 3: Internal module imports
// (none for this module - only declarations)

// Module declarations (§4.3 - ONLY declarations in mod.rs)
pub mod acl;
pub mod audit;
pub mod middleware;
pub mod policy;
pub mod rbac;

// Re-export primary types for ergonomic imports
pub use acl::{AccessControlList, AclEntry, AclPolicy};
pub use audit::{SecurityAuditLog, SecurityAuditLogger, SecurityEventType};
pub use middleware::{SecurityMiddleware, SecurityMiddlewareBuilder};
pub use policy::{AuthRequirement, PolicyDecision, PolicyScope, SecurityPolicy};
pub use rbac::{Permission, PermissionId, Role, RoleBasedAccessControl, RoleId, UserId};
