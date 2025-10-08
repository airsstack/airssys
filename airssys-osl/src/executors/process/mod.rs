//! Process executor module.
//!
//! This module provides the `ProcessExecutor` implementation for executing
//! process management operations with real tokio I/O.

// Module declarations (private - internal implementation)
mod executor;
mod kill;
mod signal;
mod spawn;

// Public re-exports
pub use executor::ProcessExecutor;
