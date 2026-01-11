//! # Capability Submodule
//!
//! Capability management for security validation.
//!
//! ## Modules
//!
//! - [`types`] - PatternMatcher and core re-exports
//! - [`set`] - CapabilitySet for permission management
//! - [`grant`] - CapabilityGrant for permission grants
//!
//! ## Examples
//!
//! Creating capabilities with builder pattern:
//!
//! ```no_run
//! use airssys_wasm::security::capability::{
//!     CapabilitySet,
//!     MessagingPermission,
//!     StoragePermission,
//! };
//!
//! let capabilities = CapabilitySet::builder()
//!     .messaging(MessagingPermission {
//!         can_send_to: vec!["comp-a/*".to_string()],
//!         can_receive_from: vec![],
//!     })
//!     .storage(StoragePermission {
//!         can_write_keys: vec!["user/*".to_string()],
//!         can_read_keys: vec!["*".to_string()],
//!     })
//!     .build();
//! ```

pub mod grant;
pub mod set;
pub mod types;

// Re-export commonly used types for convenience
pub use set::{CapabilitySet, CapabilitySetBuilder};
pub use grant::CapabilityGrant;
