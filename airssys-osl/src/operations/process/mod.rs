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

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that all operations are cloneable (required by Operation trait)
    #[test]
    fn test_all_operations_are_cloneable() {
        let spawn = ProcessSpawnOperation::new("ls");
        let _cloned = spawn.clone();

        let kill = ProcessKillOperation::new(12345);
        let _cloned = kill.clone();

        let signal = ProcessSignalOperation::new(12345, 9);
        let _cloned = signal.clone();
    }

    /// Test Display implementations for all operations
    #[test]
    fn test_operations_display() {
        let spawn = ProcessSpawnOperation::new("echo").arg("test");
        assert!(format!("{}", spawn).contains("ProcessSpawn"));
        assert!(format!("{}", spawn).contains("echo"));

        let kill = ProcessKillOperation::new(12345);
        assert_eq!(format!("{}", kill), "ProcessKill(pid=12345)");

        let signal = ProcessSignalOperation::new(12345, 9);
        assert_eq!(format!("{}", signal), "ProcessSignal(pid=12345, signal=9)");
    }

    /// Test that all process operations require elevated privileges
    #[test]
    fn test_all_operations_require_elevation() {
        use crate::core::operation::Operation;

        let spawn = ProcessSpawnOperation::new("test");
        assert!(spawn.requires_elevated_privileges());

        let kill = ProcessKillOperation::new(12345);
        assert!(kill.requires_elevated_privileges());

        let signal = ProcessSignalOperation::new(12345, 9);
        assert!(signal.requires_elevated_privileges());
    }
}
