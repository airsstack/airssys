//! Operation abstractions and types.
//!
//! This module defines the core `Operation` trait and related types that
//! represent system operations to be executed through the OS Layer Framework.

use std::fmt::Debug;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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