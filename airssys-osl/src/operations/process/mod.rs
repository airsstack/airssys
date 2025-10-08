//! Process operation types.
//!
//! This module provides concrete implementations of process management operations
//! that implement the `Operation` trait. These types are used by the framework's
//! process builder API.
//!
//! # Operations
//!
//! - [`ProcessSpawnOperation`] - Spawn new processes with command, args, and environment
//! - [`ProcessKillOperation`] - Terminate processes by PID
//! - [`ProcessSignalOperation`] - Send signals to processes
//!
//! # Security Notes
//!
//! **All process operations require elevated privileges** by default as they involve
//! system-level process management capabilities. The framework's security middleware
//! will validate permissions before execution.
//!
//! # Examples
//!
//! ```rust
//! use airssys_osl::operations::{ProcessSpawnOperation, ProcessKillOperation};
//! use airssys_osl::core::operation::{Operation, OperationType, Permission};
//!
//! // Spawn a process
//! let spawn_op = ProcessSpawnOperation::new("echo")
//!     .arg("Hello, World!");
//! assert_eq!(spawn_op.operation_type(), OperationType::Process);
//! assert!(spawn_op.requires_elevated_privileges());
//!
//! // Kill a process
//! let kill_op = ProcessKillOperation::new(12345);
//! assert_eq!(kill_op.required_permissions(), vec![Permission::ProcessManage]);
//! ```

// Operation modules
pub mod kill;
pub mod signal;
pub mod spawn;

// Re-export all operation types
pub use kill::ProcessKillOperation;
pub use signal::ProcessSignalOperation;
pub use spawn::ProcessSpawnOperation;
