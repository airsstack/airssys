//! High-level convenience functions for common OS operations.
//!
//! This module provides **three API levels** for different use cases.
//! (Full documentation will be added in Phase 1.3)

// ============================================================================
// Module Declarations (§4.3 Module Architecture - MANDATORY)
// ============================================================================

pub(crate) mod factories;

// Module declarations for simple helpers and composition
pub(crate) mod simple; // Phase 2-4: Simple helper functions (NOW ADDED)
                       // pub mod composition;       // Phase 8-10: Trait-based composition layer

// ============================================================================
// Re-exports (will be populated in later phases)
// ============================================================================

// Re-export middleware factories for internal use
pub(crate) use self::factories::{
    default_acl_policy, default_rbac_policy, default_security_middleware,
};

// Re-export simple helpers (Level 1 & 2) - NOW AVAILABLE
pub use self::simple::*;

// Re-export composition layer (Level 3) - Phase 8-10
// (public re-exports will be added here)
