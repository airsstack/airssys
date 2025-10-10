//! Middleware implementations for the OS Layer framework.
//!
//! This module contains concrete middleware implementations that provide
//! cross-cutting concerns for operation processing pipelines.
//!
//! # Available Middleware
//!
//! - **[`security`]** - Security policy enforcement and access control (Priority 100)
//! - **[`logger`]** - Activity logging and audit trail middleware (Priority 200)
//! - **[`ext`]** - Extension trait for ergonomic middleware composition

// Layer 3: Internal module imports
// (none for this module)

// Public middleware modules
pub mod ext;
pub mod logger;
pub mod security;

// Re-export extension trait for ergonomic imports
pub use ext::ExecutorExt;
