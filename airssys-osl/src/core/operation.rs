//! Operation abstractions and types.
//!
//! This module defines the core `Operation` trait and related types that
//! represent system operations to be executed through the OS Layer Framework.

use std::fmt::Debug;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Core trait for all system operations.
///
/// All operations that can be executed through the OSL framework must implement
/// this trait. Operations are designed to be stateless and contain all the
/// information needed for execution.
pub trait Operation: Debug + Send + Sync + Clone + 'static {
    /// Returns the type of operation for categorization and permission checking.
    fn operation_type(&self) -> OperationType;
    
    /// Returns the permissions required to execute this operation.
    fn required_permissions(&self) -> Vec<Permission>;
    
    /// Returns when this operation was created.
    fn created_at(&self) -> DateTime<Utc>;
    
    /// Returns a unique identifier for this operation instance.
    fn operation_id(&self) -> String {
        format!("{}:{}", self.operation_type().as_str(), Uuid::new_v4())
    }
    
    /// Returns true if this operation requires elevated privileges.
    fn requires_elevated_privileges(&self) -> bool {
        self.required_permissions().iter().any(|p| p.is_elevated())
    }
}

/// Categories of system operations supported by the framework.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationType {
    /// Filesystem operations (read, write, create, delete)
    Filesystem,
    /// Process management operations (spawn, kill, signal)
    Process,
    /// Network operations (socket creation, connection)
    Network,
    /// External utility execution (docker, gh CLI, etc.)
    Utility,
}

impl OperationType {
    /// Returns the string representation of this operation type.
    pub fn as_str(&self) -> &'static str {
        match self {
            OperationType::Filesystem => "filesystem",
            OperationType::Process => "process",
            OperationType::Network => "network",
            OperationType::Utility => "utility",
        }
    }
    
    /// Returns true if this operation type typically requires elevated privileges.
    pub fn is_privileged(&self) -> bool {
        matches!(self, OperationType::Process | OperationType::Network)
    }
}

/// Permission types for operation authorization.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    /// Read access to filesystem paths
    FilesystemRead(String),
    /// Write access to filesystem paths  
    FilesystemWrite(String),
    /// Execute access to filesystem paths
    FilesystemExecute(String),
    /// Process spawning permission
    ProcessSpawn,
    /// Process management permission (kill, signal)
    ProcessManage,
    /// Network socket creation permission
    NetworkSocket,
    /// Network connection permission to specific endpoints
    NetworkConnect(String),
    /// External utility execution permission
    UtilityExecute(String),
}

impl Permission {
    /// Returns true if this permission requires elevated privileges.
    pub fn is_elevated(&self) -> bool {
        matches!(
            self,
            Permission::ProcessSpawn 
            | Permission::ProcessManage 
            | Permission::NetworkSocket
            | Permission::FilesystemExecute(_)
        )
    }
    
    /// Returns the operation type associated with this permission.
    pub fn operation_type(&self) -> OperationType {
        match self {
            Permission::FilesystemRead(_) 
            | Permission::FilesystemWrite(_) 
            | Permission::FilesystemExecute(_) => OperationType::Filesystem,
            Permission::ProcessSpawn 
            | Permission::ProcessManage => OperationType::Process,
            Permission::NetworkSocket 
            | Permission::NetworkConnect(_) => OperationType::Network,
            Permission::UtilityExecute(_) => OperationType::Utility,
        }
    }
    
    /// Returns a human-readable description of this permission.
    pub fn description(&self) -> String {
        match self {
            Permission::FilesystemRead(path) => format!("Read access to '{path}'"),
            Permission::FilesystemWrite(path) => format!("Write access to '{path}'"),
            Permission::FilesystemExecute(path) => format!("Execute access to '{path}'"),
            Permission::ProcessSpawn => "Process spawning".to_string(),
            Permission::ProcessManage => "Process management".to_string(),
            Permission::NetworkSocket => "Network socket creation".to_string(),
            Permission::NetworkConnect(endpoint) => format!("Network connection to '{endpoint}'"),
            Permission::UtilityExecute(utility) => format!("Execute utility '{utility}'"),
        }
    }
    
    /// Returns true if this permission grants access to the specified resource.
    pub fn grants_access_to(&self, resource: &str) -> bool {
        match self {
            Permission::FilesystemRead(path) 
            | Permission::FilesystemWrite(path) 
            | Permission::FilesystemExecute(path) => {
                resource.starts_with(path) || path == "*"
            },
            Permission::NetworkConnect(endpoint) => {
                resource == endpoint || endpoint == "*"
            },
            Permission::UtilityExecute(utility) => {
                resource == utility || utility == "*"
            },
            _ => true, // Broad permissions like ProcessSpawn don't have specific resources
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_type_string_conversion() {
        assert_eq!(OperationType::Filesystem.as_str(), "filesystem");
        assert_eq!(OperationType::Process.as_str(), "process");
        assert_eq!(OperationType::Network.as_str(), "network");
        assert_eq!(OperationType::Utility.as_str(), "utility");
    }
    
    #[test]
    fn test_operation_type_privileges() {
        assert!(!OperationType::Filesystem.is_privileged());
        assert!(OperationType::Process.is_privileged());
        assert!(OperationType::Network.is_privileged());
        assert!(!OperationType::Utility.is_privileged());
    }
    
    #[test]
    fn test_permission_elevation() {
        assert!(!Permission::FilesystemRead("/tmp".to_string()).is_elevated());
        assert!(!Permission::FilesystemWrite("/tmp".to_string()).is_elevated());
        assert!(Permission::FilesystemExecute("/bin/bash".to_string()).is_elevated());
        assert!(Permission::ProcessSpawn.is_elevated());
        assert!(Permission::ProcessManage.is_elevated());
        assert!(Permission::NetworkSocket.is_elevated());
        assert!(!Permission::NetworkConnect("localhost:8080".to_string()).is_elevated());
        assert!(!Permission::UtilityExecute("docker".to_string()).is_elevated());
    }
    
    #[test]
    fn test_permission_operation_type_mapping() {
        assert_eq!(Permission::FilesystemRead("/tmp".to_string()).operation_type(), OperationType::Filesystem);
        assert_eq!(Permission::ProcessSpawn.operation_type(), OperationType::Process);
        assert_eq!(Permission::NetworkSocket.operation_type(), OperationType::Network);
        assert_eq!(Permission::UtilityExecute("docker".to_string()).operation_type(), OperationType::Utility);
    }
    
    #[test]
    fn test_permission_descriptions() {
        let read_perm = Permission::FilesystemRead("/tmp".to_string());
        assert_eq!(read_perm.description(), "Read access to '/tmp'");
        
        let spawn_perm = Permission::ProcessSpawn;
        assert_eq!(spawn_perm.description(), "Process spawning");
        
        let connect_perm = Permission::NetworkConnect("localhost:8080".to_string());
        assert_eq!(connect_perm.description(), "Network connection to 'localhost:8080'");
    }
    
    #[test]
    fn test_permission_grants_access() {
        let read_perm = Permission::FilesystemRead("/tmp".to_string());
        assert!(read_perm.grants_access_to("/tmp/file.txt"));
        assert!(!read_perm.grants_access_to("/home/user/file.txt"));
        
        let wildcard_perm = Permission::FilesystemRead("*".to_string());
        assert!(wildcard_perm.grants_access_to("/any/path"));
        
        let connect_perm = Permission::NetworkConnect("localhost:8080".to_string());
        assert!(connect_perm.grants_access_to("localhost:8080"));
        assert!(!connect_perm.grants_access_to("example.com:80"));
        
        // Broad permissions always grant access
        assert!(Permission::ProcessSpawn.grants_access_to("anything"));
    }
}