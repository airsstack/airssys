//! # Capability Submodule
//!
//! Capability management for security validation.
//!
//! ## Modules
//!
//! - [`types`] - PatternMatcher and core re-exports
//! - [`set`] - CapabilitySet for permission management
//! - [`validator`] - CapabilityValidator for SecurityValidator trait implementation
//! - [`grant`] - CapabilityGrant for permission grants

pub mod grant;
pub mod set;
pub mod types;
pub mod validator;
