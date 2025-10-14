//! # OSL Message Protocols
//!
//! Message types for OSL integration actors following ADR-RT-008 wrapper pattern.
//!
//! ## Architecture
//!
//! All OSL messages follow a three-layer wrapper pattern (ADR-RT-008):
//! 1. **Operation** - Cloneable enum describing the operation
//! 2. **Request** - Cloneable struct with operation + reply_to + request_id
//! 3. **Response** - Cloneable struct with request_id + result
//!
//! This design ensures all messages implement `Clone` (required by Message trait)
//! while enabling request-response correlation via MessageId.
//!
//! ## Example
//!
//! ```rust,no_run
//! use airssys_rt::osl::{FileSystemRequest, FileSystemOperation, FileSystemResponse};
//! use airssys_rt::util::{ActorAddress, MessageId};
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create request
//! let request = FileSystemRequest {
//!     operation: FileSystemOperation::ReadFile {
//!         path: PathBuf::from("config.txt"),
//!     },
//!     reply_to: ActorAddress::named("my-app"),
//!     request_id: MessageId::new(),
//! };
//!
//! // Send via broker (handled by ActorContext)
//! // context.send(request, fs_actor_address).await?;
//!
//! // Response received via broker subscription
//! // match response {
//! //     FileSystemResponse { request_id, result } => { ... }
//! // }
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::message::Message;
use crate::util::{ActorAddress, MessageId};

// ============================================================================
// FileSystem Messages (ADR-RT-008 Wrapper Pattern)
// ============================================================================

/// FileSystem operations (Layer 1: Cloneable operation types)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileSystemOperation {
    /// Read file contents
    ReadFile { path: PathBuf },

    /// Write file contents
    WriteFile { path: PathBuf, content: Vec<u8> },

    /// Delete file
    DeleteFile { path: PathBuf },

    /// List directory entries
    ListDirectory { path: PathBuf },

    /// Create directory
    CreateDirectory { path: PathBuf },

    /// Delete directory
    DeleteDirectory { path: PathBuf, recursive: bool },
}

/// FileSystem request message (Layer 2: Cloneable request with correlation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemRequest {
    /// The operation to perform
    pub operation: FileSystemOperation,

    /// Actor address to send response to
    pub reply_to: ActorAddress,

    /// Unique request identifier for correlation
    pub request_id: MessageId,
}

impl Message for FileSystemRequest {
    const MESSAGE_TYPE: &'static str = "osl::filesystem::request";
}

/// FileSystem response message (Layer 3: Cloneable response with correlation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemResponse {
    /// Request ID for correlation
    pub request_id: MessageId,

    /// Operation result
    pub result: FileSystemResult,
}

impl Message for FileSystemResponse {
    const MESSAGE_TYPE: &'static str = "osl::filesystem::response";
}

/// FileSystem operation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileSystemResult {
    /// File read successfully
    ReadSuccess { content: Vec<u8> },

    /// File write successfully
    WriteSuccess,

    /// File delete successfully
    DeleteSuccess,

    /// Directory list successfully
    ListSuccess { entries: Vec<DirEntry> },

    /// Directory create successfully
    CreateSuccess,

    /// Directory delete successfully
    DeleteDirectorySuccess,

    /// Operation error
    Error { error: FileSystemError },
}

/// Directory entry information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirEntry {
    pub path: PathBuf,
    pub is_file: bool,
    pub is_directory: bool,
    pub size: u64,
}

/// FileSystem operation errors
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum FileSystemError {
    /// File not found
    #[error("File not found: {path:?}")]
    NotFound { path: PathBuf },

    /// Permission denied
    #[error("Permission denied: {path:?}")]
    PermissionDenied { path: PathBuf },

    /// I/O error
    #[error("I/O error: {message}")]
    IoError { message: String },

    /// Invalid path
    #[error("Invalid path: {path:?}")]
    InvalidPath { path: PathBuf },

    /// Other error
    #[error("Error: {message}")]
    Other { message: String },
}

// ============================================================================
// Process Messages (ADR-RT-008 Wrapper Pattern)
// ============================================================================

/// Process operations (Layer 1: Cloneable operation types)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessOperation {
    /// Spawn a new process
    Spawn {
        program: PathBuf,
        args: Vec<String>,
        env: HashMap<String, String>,
        working_dir: Option<PathBuf>,
    },

    /// Terminate a running process
    Terminate {
        pid: u32,
        graceful: bool,
        timeout: Duration,
    },

    /// Get process status
    GetStatus { pid: u32 },

    /// Wait for process completion
    Wait { pid: u32, timeout: Option<Duration> },
}

/// Process request message (Layer 2: Cloneable request with correlation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessRequest {
    /// The operation to perform
    pub operation: ProcessOperation,

    /// Actor address to send response to
    pub reply_to: ActorAddress,

    /// Unique request identifier for correlation
    pub request_id: MessageId,
}

impl Message for ProcessRequest {
    const MESSAGE_TYPE: &'static str = "osl::process::request";
}

/// Process response message (Layer 3: Cloneable response with correlation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessResponse {
    /// Request ID for correlation
    pub request_id: MessageId,

    /// Operation result
    pub result: ProcessResult,
}

impl Message for ProcessResponse {
    const MESSAGE_TYPE: &'static str = "osl::process::response";
}

/// Process operation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessResult {
    /// Process spawned successfully
    SpawnSuccess { pid: u32, process_id: ProcessId },

    /// Process terminated successfully
    TerminateSuccess { pid: u32 },

    /// Process status
    Status { pid: u32, state: ProcessState },

    /// Process wait completed
    WaitSuccess { pid: u32, exit_code: i32 },

    /// Process wait timeout
    WaitTimeout { pid: u32 },

    /// Operation error
    Error { error: ProcessError },
}

/// Internal process identifier
pub type ProcessId = u64;

/// Process state
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProcessState {
    /// Process is running
    Running,

    /// Process has exited
    Exited { exit_code: i32 },

    /// Process state unknown
    Unknown,
}

/// Process operation errors
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum ProcessError {
    /// Process not found
    #[error("Process not found: {pid}")]
    NotFound { pid: u32 },

    /// Failed to spawn process
    #[error("Failed to spawn {program}: {message}")]
    SpawnFailed { program: String, message: String },

    /// Failed to terminate process
    #[error("Failed to terminate process {pid}: {message}")]
    TerminateFailed { pid: u32, message: String },

    /// Permission denied
    #[error("Permission denied: {message}")]
    PermissionDenied { message: String },

    /// Timeout
    #[error("Timeout: {operation}")]
    Timeout { operation: String },

    /// Other error
    #[error("Error: {message}")]
    Other { message: String },
}

// ============================================================================
// Network Messages (ADR-RT-008 Wrapper Pattern)
// ============================================================================

/// Network operations (Layer 1: Cloneable operation types)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkOperation {
    /// Connect to TCP endpoint
    TcpConnect { addr: SocketAddr, timeout: Duration },

    /// Disconnect TCP connection
    TcpDisconnect { connection_id: ConnectionId },

    /// Bind UDP socket
    UdpBind { addr: SocketAddr },

    /// Close UDP socket
    UdpClose { socket_id: SocketId },

    /// Get connection status
    GetConnectionStatus { connection_id: ConnectionId },
}

/// Network request message (Layer 2: Cloneable request with correlation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    /// The operation to perform
    pub operation: NetworkOperation,

    /// Actor address to send response to
    pub reply_to: ActorAddress,

    /// Unique request identifier for correlation
    pub request_id: MessageId,
}

impl Message for NetworkRequest {
    const MESSAGE_TYPE: &'static str = "osl::network::request";
}

/// Network response message (Layer 3: Cloneable response with correlation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkResponse {
    /// Request ID for correlation
    pub request_id: MessageId,

    /// Operation result
    pub result: NetworkResult,
}

impl Message for NetworkResponse {
    const MESSAGE_TYPE: &'static str = "osl::network::response";
}

/// Network operation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkResult {
    /// TCP connection established
    TcpConnectSuccess {
        connection_id: ConnectionId,
        local_addr: SocketAddr,
        remote_addr: SocketAddr,
    },

    /// TCP connection closed
    TcpDisconnectSuccess { connection_id: ConnectionId },

    /// UDP socket bound
    UdpBindSuccess {
        socket_id: SocketId,
        local_addr: SocketAddr,
    },

    /// UDP socket closed
    UdpCloseSuccess { socket_id: SocketId },

    /// Connection status
    ConnectionStatus {
        connection_id: ConnectionId,
        state: ConnectionState,
    },

    /// Operation error
    Error { error: NetworkError },
}

/// Internal connection identifier
pub type ConnectionId = u64;

/// Internal socket identifier
pub type SocketId = u64;

/// Connection state
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConnectionState {
    /// Connection is active
    Connected,

    /// Connection is disconnected
    Disconnected,

    /// Connection state unknown
    Unknown,
}

/// Network operation errors
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum NetworkError {
    /// Connection not found
    #[error("Connection not found: {connection_id}")]
    NotFound { connection_id: ConnectionId },

    /// Failed to connect
    #[error("Failed to connect to {addr}: {message}")]
    ConnectFailed { addr: SocketAddr, message: String },

    /// Failed to bind
    #[error("Failed to bind to {addr}: {message}")]
    BindFailed { addr: SocketAddr, message: String },

    /// Connection timeout
    #[error("Connection timeout: {addr}")]
    Timeout { addr: SocketAddr },

    /// Other error
    #[error("Error: {message}")]
    Other { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_types() {
        assert_eq!(FileSystemRequest::MESSAGE_TYPE, "osl::filesystem::request");
        assert_eq!(
            FileSystemResponse::MESSAGE_TYPE,
            "osl::filesystem::response"
        );
        assert_eq!(ProcessRequest::MESSAGE_TYPE, "osl::process::request");
        assert_eq!(ProcessResponse::MESSAGE_TYPE, "osl::process::response");
        assert_eq!(NetworkRequest::MESSAGE_TYPE, "osl::network::request");
        assert_eq!(NetworkResponse::MESSAGE_TYPE, "osl::network::response");
    }

    #[test]
    fn test_filesystem_operation_clone() {
        let op = FileSystemOperation::ReadFile {
            path: PathBuf::from("/test"),
        };
        let cloned = op.clone();
        assert!(matches!(cloned, FileSystemOperation::ReadFile { path: _ }));
    }

    #[test]
    fn test_process_state_equality() {
        assert_eq!(ProcessState::Running, ProcessState::Running);
        assert_eq!(
            ProcessState::Exited { exit_code: 0 },
            ProcessState::Exited { exit_code: 0 }
        );
        assert_ne!(
            ProcessState::Exited { exit_code: 0 },
            ProcessState::Exited { exit_code: 1 }
        );
    }

    #[test]
    fn test_connection_state_equality() {
        assert_eq!(ConnectionState::Connected, ConnectionState::Connected);
        assert_ne!(ConnectionState::Connected, ConnectionState::Disconnected);
    }

    #[test]
    fn test_request_response_correlation() {
        let request_id = MessageId::new();
        let request = FileSystemRequest {
            operation: FileSystemOperation::ReadFile {
                path: PathBuf::from("/test"),
            },
            reply_to: ActorAddress::anonymous(),
            request_id,
        };

        let response = FileSystemResponse {
            request_id,
            result: FileSystemResult::ReadSuccess {
                content: vec![1, 2, 3],
            },
        };

        assert_eq!(request.request_id, response.request_id);
    }
}
