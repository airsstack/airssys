//! Integration tests for process operations.
//!
//! These tests validate cross-cutting behavior across all process operations
//! to ensure consistency and compliance with the Operation trait requirements.

use airssys_osl::core::operation::Operation;
use airssys_osl::operations::{
    ProcessKillOperation, ProcessSignalOperation, ProcessSpawnOperation,
};

/// Test that all process operations are cloneable (required by Operation trait)
#[test]
fn test_process_operations_are_cloneable() {
    let spawn = ProcessSpawnOperation::new("ls");
    let _cloned = spawn.clone();

    let kill = ProcessKillOperation::new(12345);
    let _cloned = kill.clone();

    let signal = ProcessSignalOperation::new(12345, 9);
    let _cloned = signal.clone();
}

/// Test Display implementations for all process operations
#[test]
fn test_process_operations_display() {
    let spawn = ProcessSpawnOperation::new("echo").arg("test");
    assert!(format!("{spawn}").contains("ProcessSpawn"));
    assert!(format!("{spawn}").contains("echo"));

    let kill = ProcessKillOperation::new(12345);
    assert_eq!(format!("{kill}"), "ProcessKill(pid=12345)");

    let signal = ProcessSignalOperation::new(12345, 9);
    assert_eq!(format!("{signal}"), "ProcessSignal(pid=12345, signal=9)");
}

/// Test that all process operations require elevated privileges
#[test]
fn test_process_operations_require_elevation() {
    let spawn = ProcessSpawnOperation::new("test");
    assert!(spawn.requires_elevated_privileges());

    let kill = ProcessKillOperation::new(12345);
    assert!(kill.requires_elevated_privileges());

    let signal = ProcessSignalOperation::new(12345, 9);
    assert!(signal.requires_elevated_privileges());
}
