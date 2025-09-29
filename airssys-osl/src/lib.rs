//! AirsSys OS Layer Framework
//!
//! `airssys-osl` provides a comprehensive, security-first abstraction layer
//! for operating system interactions. It offers cross-platform OS operations
//! with robust security policies, comprehensive activity logging, and
//! middleware-based extensibility.
//!
//! # Core Components
//!
//! * **Core Abstractions** - Foundational traits and types in the [`core`] module
//! * **Security Framework** - Policy-based security with audit trails
//! * **Activity Logging** - Comprehensive operation logging and monitoring
//! * **Cross-Platform Support** - Unified API across Linux, macOS, and Windows
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use airssys_osl::core::{ExecutionContext, SecurityContext};
//!
//! // Create a security context for the current principal
//! let security_context = SecurityContext::new("user@example.com".to_string());
//! let execution_context = ExecutionContext::new(security_context);
//!
//! // Operations and executors use these contexts for security and audit trails
//! ```

pub mod core;

// Re-export core types for convenient access
pub use core::{
    ExecutionContext, ExecutionResult, OSError, OSExecutor, OSResult, Operation, OperationType,
    Permission, SecurityContext,
};
