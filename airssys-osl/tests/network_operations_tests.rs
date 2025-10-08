//! Integration tests for network operations.
//!
//! These tests validate cross-cutting behavior across all network operations
//! to ensure consistency and compliance with the Operation trait requirements.

use airssys_osl::core::operation::{Operation, Permission};
use airssys_osl::operations::{
    NetworkConnectOperation, NetworkListenOperation, NetworkSocketOperation,
};

/// Test that all network operations are cloneable (required by Operation trait)
#[test]
fn test_network_operations_are_cloneable() {
    let connect = NetworkConnectOperation::new("localhost:8080");
    let _cloned = connect.clone();

    let listen = NetworkListenOperation::new("0.0.0.0:8080");
    let _cloned = listen.clone();

    let socket = NetworkSocketOperation::new("tcp");
    let _cloned = socket.clone();
}

/// Test Display implementations for all network operations
#[test]
fn test_network_operations_display() {
    let connect = NetworkConnectOperation::new("localhost:8080");
    assert_eq!(
        format!("{connect}"),
        "NetworkConnect(address=localhost:8080)"
    );

    let listen = NetworkListenOperation::new("0.0.0.0:8080");
    assert_eq!(format!("{listen}"), "NetworkListen(address=0.0.0.0:8080)");

    let listen_with_backlog = NetworkListenOperation::new("0.0.0.0:8080").with_backlog(128);
    assert_eq!(
        format!("{listen_with_backlog}"),
        "NetworkListen(address=0.0.0.0:8080, backlog=128)"
    );

    let socket = NetworkSocketOperation::new("tcp");
    assert_eq!(format!("{socket}"), "NetworkSocket(type=tcp)");
}

/// Test that all network operations require elevated privileges
#[test]
fn test_network_operations_require_elevation() {
    let connect = NetworkConnectOperation::new("localhost:8080");
    assert!(connect.requires_elevated_privileges());

    let listen = NetworkListenOperation::new("0.0.0.0:8080");
    assert!(listen.requires_elevated_privileges());

    let socket = NetworkSocketOperation::new("tcp");
    assert!(socket.requires_elevated_privileges());
}

/// Test Unix domain socket support in NetworkListenOperation
#[test]
fn test_unix_socket_listen_operation() {
    let op = NetworkListenOperation::new("unix-socket").with_socket_path("/tmp/test.sock");

    // Should have both NetworkSocket and FilesystemWrite permissions
    let perms = op.required_permissions();
    assert_eq!(perms.len(), 2);
    assert!(perms.contains(&Permission::NetworkSocket));
    assert!(perms.contains(&Permission::FilesystemWrite("/tmp/test.sock".to_string())));

    // Display should show socket_path
    assert!(format!("{op}").contains("socket_path=/tmp/test.sock"));
}

/// Test Unix domain socket with backlog
#[test]
fn test_unix_socket_with_backlog() {
    let op = NetworkListenOperation::new("unix-socket")
        .with_socket_path("/var/run/app.sock")
        .with_backlog(128);

    assert_eq!(op.socket_path, Some("/var/run/app.sock".to_string()));
    assert_eq!(op.backlog, Some(128));

    let display = format!("{op}");
    assert!(display.contains("socket_path=/var/run/app.sock"));
    assert!(display.contains("backlog=128"));
}
