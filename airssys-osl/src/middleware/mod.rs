//! Middleware implementations for the OS Layer framework.
//!
//! This module contains concrete middleware implementations that provide
//! cross-cutting concerns for operation processing pipelines.
//!
//! # Available Middleware
//!
//! - **[`logger`]** - Activity logging and audit trail middleware
//! - **[`ext`]** - Extension trait for ergonomic middleware composition

// Layer 3: Internal module imports
// (none for this module)

// Public middleware modules
pub mod ext;
pub mod logger;

// Re-export extension trait for ergonomic imports
pub use ext::ExecutorExt;
