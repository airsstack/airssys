//! Shared utilities for macro implementations

use syn::Ident;

/// Maps method names to operation types.
///
/// Returns (OperationType, ModulePath) tuple.
/// Full mapping table will be added in MACROS-TASK-002.
pub fn map_method_name_to_operation(name: &Ident) -> Option<(&'static str, &'static str)> {
    match name.to_string().as_str() {
        "file_read" => Some(("FileReadOperation", "filesystem")),
        "file_write" => Some(("FileWriteOperation", "filesystem")),
        "file_delete" => Some(("FileDeleteOperation", "filesystem")),
        "directory_create" => Some(("DirectoryCreateOperation", "filesystem")),
        "process_spawn" => Some(("ProcessSpawnOperation", "process")),
        "process_kill" => Some(("ProcessKillOperation", "process")),
        "process_query" => Some(("ProcessQueryOperation", "process")),
        "tcp_connect" => Some(("TcpConnectOperation", "network")),
        "tcp_listen" => Some(("TcpListenOperation", "network")),
        "udp_bind" => Some(("UdpBindOperation", "network")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
