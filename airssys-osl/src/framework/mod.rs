//! High-level framework interface for ergonomic OSL usage.
//!
//! This module provides the primary API for AirsSys OSL, implementing builder patterns
//! and automatic middleware orchestration while maintaining full compatibility with
//! explicit primitive APIs for advanced use cases.
//!
//! # Architecture
//!
//! The framework implements a multi-level architecture:
//!
//! ```text
//! Application Code
//!     ↓
//! Framework Layer (this module) - ergonomic patterns for common use cases
//!     ↓  
//! Core Primitives (src/core/) - generic patterns for performance
//!     ↓
//! OS Layer
//! ```
//!
//! # Usage Levels
//!
//! ## Level 1: Simple Builder (80% of use cases)
//! ```rust
//! use airssys_osl::prelude::*;
//!
//! # async fn example() -> OSResult<()> {
//! let osl = OSLFramework::builder()
//!     .with_default_security()
//!     .with_security_logging(true)
//!     .build().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Level 2: Custom Configuration (15% of use cases)  
//! ```rust
//! use airssys_osl::prelude::*;
//!
//! # async fn example() -> OSResult<()> {
//! let osl = OSLFramework::builder()
//!     .with_policy_file("/etc/osl/custom-policy.toml")
//!     .build().await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Note
//!
//! This is the foundation implementation (OSL-TASK-005). Full framework functionality
//! including middleware orchestration and operation builders will be completed in OSL-TASK-006.

// Module declarations
mod builder;
pub mod config;
mod framework;
mod operations;

// Re-exports
pub use builder::OSLFrameworkBuilder;
pub use config::{OSLConfig, OSLConfigBuilder, SecurityConfig};
pub use framework::OSLFramework;
pub use operations::{FilesystemBuilder, NetworkBuilder, ProcessBuilder};
