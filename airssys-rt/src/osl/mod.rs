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
//! use airssys_rt::osl::{FileSystemActor, FileSystemRequest, FileSystemOperation};
//! use airssys_rt::actor::{Actor, ActorContext};
//! use airssys_rt::broker::InMemoryMessageBroker;
//! use airssys_rt::util::{ActorAddress, MessageId};
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create FileSystemActor for centralized file operations
//! let mut fs_actor = FileSystemActor::new();
//! let broker = InMemoryMessageBroker::new();
//! let actor_addr = ActorAddress::named("fs-actor");
//! let mut context = ActorContext::new(actor_addr, broker);
//!
//! // Send message to FileSystemActor
//! let request = FileSystemRequest {
//!     request_id: MessageId::new(),
//!     reply_to: ActorAddress::named("client"),
//!     operation: FileSystemOperation::ReadFile {
//!         path: PathBuf::from("config.txt"),
//!     },
//! };
//!
//! fs_actor.handle_message(request, &mut context).await?;
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
