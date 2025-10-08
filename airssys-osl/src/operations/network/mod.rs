//! Network operation types.
//!
//! This module provides concrete implementations of network operations that
//! implement the `Operation` trait. These types are used by the framework's
//! network builder API.

// Module declarations
pub mod connect;
pub mod listen;
pub mod socket;

// Re-export all operation types for convenient access
pub use connect::NetworkConnectOperation;
pub use listen::NetworkListenOperation;
pub use socket::NetworkSocketOperation;
