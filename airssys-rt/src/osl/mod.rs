//! # OSL Integration Module
//!
//! Provides dedicated actors for integrating airssys-rt with airssys-osl (OS Layer).
//!
//! ## Architecture
//!
//! This module implements the Hierarchical Supervisor Architecture (ADR-RT-007)
//! with dedicated OSL integration actors:
//!
//! ```text
//! RootSupervisor
//! ├── OSLSupervisor (manages OS integration actors)
//! │   ├── FileSystemActor (all file/directory operations)
//! │   ├── ProcessActor (all process spawning/management)
//! │   └── NetworkActor (all network connections)
//! └── ApplicationSupervisor (manages business logic actors)
//!     ├── WorkerActor
//!     ├── AggregatorActor
//!     └── CoordinatorActor
//! ```
//!
//! ## Key Benefits
//!
//! - **Clean Fault Isolation**: OSL failures don't cascade to application actors
//! - **Superior Testability**: Mock OSL actors in tests (no real OS operations)
//! - **Centralized Management**: Single source of truth for OS operations
//! - **Service-Oriented Design**: Clear service boundaries and contracts
//! - **Process Lifecycle Safety**: Automatic cleanup in ProcessActor.stop()
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use airssys_rt::osl::{FileSystemActor, FileSystemMessage, FileSystemResponse};
//! use airssys_rt::{ActorSystem, SupervisorNode, OneForOne};
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create OSL supervisor with FileSystemActor
//! let mut supervisor = SupervisorNode::new(
//!     "osl-supervisor".into(),
//!     OneForOne::new(),
//!     airssys_rt::NoopMonitor,
//! );
//!
//! // Application actors send messages to FileSystemActor
//! // let response = filesystem_actor.send(FileSystemMessage::ReadFile {
//! //     path: PathBuf::from("config.txt"),
//! //     respond_to: tx,
//! // }).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Related Documentation
//!
//! - **ADR-RT-007**: Hierarchical Supervisor Architecture for OSL Integration
//! - **KNOWLEDGE-RT-017**: OSL Integration Actors Pattern
//! - **KNOWLEDGE-RT-016**: Process Group Management - Future Considerations

pub mod actors;

// Re-export commonly used types
pub use actors::{
    FileSystemActor, FileSystemError, FileSystemOperation, FileSystemRequest, FileSystemResponse,
    FileSystemResult, NetworkActor, NetworkError, NetworkOperation, NetworkRequest,
    NetworkResponse, NetworkResult, ProcessActor, ProcessError, ProcessOperation, ProcessRequest,
    ProcessResponse, ProcessResult,
};
