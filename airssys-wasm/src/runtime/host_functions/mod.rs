//! Host function implementations and registration for WASM runtime
//!
//! This module provides the implementation of host functions that WASM components
//! can call to interact with the host application. Functions are organized by category:
//! - `messaging`: Message routing and publishing
//! - `services`: Service discovery and interaction
//! - `storage`: Component-isolated storage operations
//! - `marker_traits`: Host trait implementations and registration

// Submodules (module declarations only per PROJECTS_STANDARD.md §4.3)
pub mod marker_traits;
pub mod messaging;
pub mod services;
pub mod storage;
