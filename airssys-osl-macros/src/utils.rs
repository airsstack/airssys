//! Shared utilities for macro implementations

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Information about an operation type for code generation.
///
/// Contains all metadata needed to generate OSExecutor trait implementations
/// for a specific operation type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationInfo {
    /// Method name (e.g., "file_read")
    pub method_name: &'static str,
    /// Operation type name (e.g., "FileReadOperation")
    pub type_name: &'static str,
    /// Module path within airssys_osl::operations (e.g., "filesystem")
    pub module_path: &'static str,
}

impl OperationInfo {
    /// Returns the fully qualified path to the operation type.
    ///
    /// Example: `airssys_osl::operations::filesystem::FileReadOperation`
    pub fn operation_path(&self) -> TokenStream {
        let module = syn::Ident::new(self.module_path, proc_macro2::Span::call_site());
        let type_name = syn::Ident::new(self.type_name, proc_macro2::Span::call_site());

        quote! {
            airssys_osl::operations::#module::#type_name
        }
    }
}

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

/// Gets operation information for a method name.
///
/// Returns `Some(OperationInfo)` if the method name matches a known operation,
/// `None` otherwise.
///
/// # Example
///
/// ```rust,ignore
/// let info = get_operation_info("file_read").unwrap();
/// assert_eq!(info.method_name, "file_read");
/// assert_eq!(info.type_name, "FileReadOperation");
/// assert_eq!(info.module_path, "filesystem");
/// ```
pub fn get_operation_info(name: &str) -> Option<OperationInfo> {
    match name {
        // Filesystem operations (verified from airssys-osl)
        "file_read" => Some(OperationInfo {
            method_name: "file_read",
            type_name: "FileReadOperation",
            module_path: "filesystem",
        }),
        "file_write" => Some(OperationInfo {
            method_name: "file_write",
            type_name: "FileWriteOperation",
            module_path: "filesystem",
        }),
        "file_delete" => Some(OperationInfo {
            method_name: "file_delete",
            type_name: "FileDeleteOperation",
            module_path: "filesystem",
        }),
        "directory_create" => Some(OperationInfo {
            method_name: "directory_create",
            type_name: "DirectoryCreateOperation",
            module_path: "filesystem",
        }),
        "directory_list" => Some(OperationInfo {
            method_name: "directory_list",
            type_name: "DirectoryListOperation",
            module_path: "filesystem",
        }),
        // Process operations (verified from airssys-osl)
        "process_spawn" => Some(OperationInfo {
            method_name: "process_spawn",
            type_name: "ProcessSpawnOperation",
            module_path: "process",
        }),
        "process_kill" => Some(OperationInfo {
            method_name: "process_kill",
            type_name: "ProcessKillOperation",
            module_path: "process",
        }),
        "process_signal" => Some(OperationInfo {
            method_name: "process_signal",
            type_name: "ProcessSignalOperation",
            module_path: "process",
        }),
        // Network operations (verified from airssys-osl)
        "network_connect" => Some(OperationInfo {
            method_name: "network_connect",
            type_name: "NetworkConnectOperation",
            module_path: "network",
        }),
        "network_listen" => Some(OperationInfo {
            method_name: "network_listen",
            type_name: "NetworkListenOperation",
            module_path: "network",
        }),
        "network_socket" => Some(OperationInfo {
            method_name: "network_socket",
            type_name: "NetworkSocketOperation",
            module_path: "network",
        }),
        _ => None,
    }
}

/// Maps method names to operation types.
///
/// Returns (OperationType, ModulePath) tuple.
/// Based on airssys-osl/src/operations/ structure.
///
/// **Deprecated:** Use `get_operation_info()` instead for richer information.
#[allow(dead_code)]
pub fn map_method_name_to_operation(name: &Ident) -> Option<(&'static str, &'static str)> {
    get_operation_info(&name.to_string()).map(|info| (info.type_name, info.module_path))
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
    #[allow(clippy::unwrap_used)]
    fn test_get_operation_info() {
        // Test filesystem operation
        let info = get_operation_info("file_read").unwrap();
        assert_eq!(info.method_name, "file_read");
        assert_eq!(info.type_name, "FileReadOperation");
        assert_eq!(info.module_path, "filesystem");

        // Test process operation
        let info = get_operation_info("process_spawn").unwrap();
        assert_eq!(info.method_name, "process_spawn");
        assert_eq!(info.type_name, "ProcessSpawnOperation");
        assert_eq!(info.module_path, "process");

        // Test network operation
        let info = get_operation_info("network_connect").unwrap();
        assert_eq!(info.method_name, "network_connect");
        assert_eq!(info.type_name, "NetworkConnectOperation");
        assert_eq!(info.module_path, "network");

        // Test invalid operation
        assert!(get_operation_info("invalid_op").is_none());
    }

    #[test]
    #[allow(clippy::unwrap_used, clippy::uninlined_format_args)]
    fn test_all_operations_have_info() {
        // Ensure all 11 operations return OperationInfo
        let operations = [
            "file_read",
            "file_write",
            "file_delete",
            "directory_create",
            "directory_list",
            "process_spawn",
            "process_kill",
            "process_signal",
            "network_connect",
            "network_listen",
            "network_socket",
        ];

        for op in operations {
            assert!(
                get_operation_info(op).is_some(),
                "Operation '{}' should have OperationInfo",
                op
            );
        }
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_operation_path_generation() {
        let info = get_operation_info("file_read").unwrap();
        let path = info.operation_path();

        // The path should contain the expected tokens
        let path_str = path.to_string();
        assert!(path_str.contains("airssys_osl"));
        assert!(path_str.contains("operations"));
        assert!(path_str.contains("filesystem"));
        assert!(path_str.contains("FileReadOperation"));
    }

    #[test]
    fn test_invalid_operation() {
        // Tests are allowed to use unwrap/expect
        #[allow(clippy::unwrap_used)]
        let ident = syn::parse_str::<Ident>("invalid_op").unwrap();
        assert_eq!(map_method_name_to_operation(&ident), None);
    }
}
