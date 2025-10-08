//! Shared utilities for macro implementations

use syn::Ident;

/// Checks if a method name is a valid operation method.
///
/// Returns true if the method name matches one of the 11 supported operations.
pub fn is_operation_method(name: &str) -> bool {
    matches!(
        name,
        "file_read"
            | "file_write"
            | "file_delete"
            | "directory_create"
            | "directory_list"
            | "process_spawn"
            | "process_kill"
            | "process_signal"
            | "network_connect"
            | "network_listen"
            | "network_socket"
    )
}

/// Maps method names to operation types.
///
/// Returns (OperationType, ModulePath) tuple.
/// Based on airssys-osl/src/operations/ structure.
#[allow(dead_code)]
pub fn map_method_name_to_operation(name: &Ident) -> Option<(&'static str, &'static str)> {
    match name.to_string().as_str() {
        // Filesystem operations (verified from airssys-osl)
        "file_read" => Some(("FileReadOperation", "filesystem")),
        "file_write" => Some(("FileWriteOperation", "filesystem")),
        "file_delete" => Some(("FileDeleteOperation", "filesystem")),
        "directory_create" => Some(("DirectoryCreateOperation", "filesystem")),
        "directory_list" => Some(("DirectoryListOperation", "filesystem")),
        // Process operations (verified from airssys-osl)
        "process_spawn" => Some(("ProcessSpawnOperation", "process")),
        "process_kill" => Some(("ProcessKillOperation", "process")),
        "process_signal" => Some(("ProcessSignalOperation", "process")),
        // Network operations (verified from airssys-osl)
        "network_connect" => Some(("NetworkConnectOperation", "network")),
        "network_listen" => Some(("NetworkListenOperation", "network")),
        "network_socket" => Some(("NetworkSocketOperation", "network")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_operation_method() {
        // Filesystem operations
        assert!(is_operation_method("file_read"));
        assert!(is_operation_method("file_write"));
        assert!(is_operation_method("file_delete"));
        assert!(is_operation_method("directory_create"));
        assert!(is_operation_method("directory_list"));

        // Process operations
        assert!(is_operation_method("process_spawn"));
        assert!(is_operation_method("process_kill"));
        assert!(is_operation_method("process_signal"));

        // Network operations
        assert!(is_operation_method("network_connect"));
        assert!(is_operation_method("network_listen"));
        assert!(is_operation_method("network_socket"));

        // Not an operation
        assert!(!is_operation_method("invalid_op"));
        assert!(!is_operation_method("helper_method"));
    }

    #[test]
    fn test_filesystem_operations() {
        // Tests are allowed to use unwrap/expect
        #[allow(clippy::unwrap_used)]
        let ident = syn::parse_str::<Ident>("file_read").unwrap();
        assert_eq!(
            map_method_name_to_operation(&ident),
            Some(("FileReadOperation", "filesystem"))
        );
    }

    #[test]
    fn test_invalid_operation() {
        // Tests are allowed to use unwrap/expect
        #[allow(clippy::unwrap_used)]
        let ident = syn::parse_str::<Ident>("invalid_op").unwrap();
        assert_eq!(map_method_name_to_operation(&ident), None);
    }
}
