//! # OSL Integration Actors
//!
//! Dedicated actors that serve as the interface between the actor runtime
//! and OS layer operations.
//!
//! ## Actors
//!
//! - **FileSystemActor**: All file/directory operations with audit logging
//! - **ProcessActor**: Process spawning/management with lifecycle tracking
//! - **NetworkActor**: Network connections with connection pooling

pub mod filesystem;
pub mod messages;
pub mod network;
pub mod process;

// Re-export actor types
pub use filesystem::FileSystemActor;
pub use messages::{
    ConnectionId, ConnectionState, DirEntry, FileSystemError, FileSystemOperation,
    FileSystemRequest, FileSystemResponse, FileSystemResult, NetworkError, NetworkOperation,
    NetworkRequest, NetworkResponse, NetworkResult, ProcessError, ProcessId, ProcessOperation,
    ProcessRequest, ProcessResponse, ProcessResult, ProcessState, SocketId,
};
pub use network::NetworkActor;
pub use process::ProcessActor;
