//! Actor system framework with lifecycle management.
//!
//! Provides the main entry point for the actor runtime system.

pub mod config;
pub mod errors;

// Re-exports
pub use config::{
    SystemConfig, DEFAULT_ENABLE_METRICS, DEFAULT_MAILBOX_CAPACITY, DEFAULT_MAX_ACTORS,
    DEFAULT_SHUTDOWN_TIMEOUT, DEFAULT_SPAWN_TIMEOUT,
};
pub use errors::SystemError;
