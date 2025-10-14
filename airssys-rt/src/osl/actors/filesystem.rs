//! # FileSystemActor
//!
//! Centralized actor for all file system operations with audit logging.
//!
//! ## Responsibilities
//!
//! - All file system operations (read, write, delete, list)
//! - Directory management (create, remove, traverse)
//! - File metadata queries (stat, permissions, ownership)
//! - Centralized file operation audit logging
//!
//! ## Example
//!
//! ```rust,no_run
//! use airssys_rt::osl::{FileSystemActor, FileSystemRequest, FileSystemOperation, OSLMessage};
//! use airssys_rt::actor::{Actor, ActorContext};
//! use airssys_rt::broker::InMemoryMessageBroker;
//! use airssys_rt::util::{ActorAddress, MessageId};
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let broker = InMemoryMessageBroker::<OSLMessage>::new();
//! let mut actor = FileSystemActor::new(broker.clone());
//! let actor_addr = ActorAddress::named("fs-actor");
//! let mut context = ActorContext::new(actor_addr, InMemoryMessageBroker::new());
//!
//! let request = FileSystemRequest {
//!     request_id: MessageId::new(),
//!     reply_to: ActorAddress::named("client"),
//!     operation: FileSystemOperation::ReadFile {
//!         path: PathBuf::from("config.txt"),
//!     },
//! };
//!
//! actor.handle_message(request, &mut context).await?;
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;
use std::marker::PhantomData;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::actor::{Actor, ActorContext, ErrorAction};
use crate::broker::MessageBroker;
use crate::message::{Message, MessageEnvelope};
use crate::supervisor::{Child, ChildHealth};

use super::messages::{
    DirEntry, FileSystemError, FileSystemOperation, FileSystemRequest, FileSystemResponse,
    FileSystemResult,
};

/// FileSystemActor - Centralized file system operations
///
/// This actor serves as the interface between the actor runtime and
/// file system operations. All application actors should send messages
/// to this actor rather than performing file operations directly.
///
/// ## Generic Parameters
///
/// - `M`: Message type implementing `Message` trait (for broker compatibility)
/// - `B`: MessageBroker implementation for publishing responses
///
/// ## Benefits
///
/// - Centralized audit logging for all file operations
/// - Clean fault isolation (FS failures don't crash app actors)
/// - Superior testability (mock this actor in tests)
/// - Rate limiting and backpressure protection
/// - Broker injection for flexible message routing
pub struct FileSystemActor<M, B>
where
    M: Message,
    B: MessageBroker<M>,
{
    /// Message broker for publishing responses
    broker: B,

    /// Operation counter for metrics
    operation_count: u64,

    /// Active operations tracking
    active_operations: HashMap<OperationId, Operation>,

    /// Actor creation timestamp
    created_at: DateTime<Utc>,

    /// Last operation timestamp
    last_operation_at: Option<DateTime<Utc>>,

    /// Phantom data for message type
    _phantom: PhantomData<M>,
}

type OperationId = u64;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Operation {
    id: OperationId,
    operation_type: String,
    started_at: DateTime<Utc>,
}

impl<M, B> FileSystemActor<M, B>
where
    M: Message + 'static,
    B: MessageBroker<M> + Clone + 'static,
{
    /// Create a new FileSystemActor with broker injection
    ///
    /// # Arguments
    ///
    /// * `broker` - MessageBroker implementation for publishing responses
    pub fn new(broker: B) -> Self {
        Self {
            broker,
            operation_count: 0,
            active_operations: HashMap::new(),
            created_at: Utc::now(),
            last_operation_at: None,
            _phantom: PhantomData,
        }
    }

    /// Get operation count
    pub fn operation_count(&self) -> u64 {
        self.operation_count
    }

    /// Get active operation count
    pub fn active_operations_count(&self) -> usize {
        self.active_operations.len()
    }

    /// Execute filesystem operation and return result
    async fn execute_operation(&mut self, operation: FileSystemOperation) -> FileSystemResult {
        self.operation_count += 1;
        self.last_operation_at = Some(Utc::now());

        let operation_id = self.operation_count;
        let operation_type = match &operation {
            FileSystemOperation::ReadFile { .. } => "read_file",
            FileSystemOperation::WriteFile { .. } => "write_file",
            FileSystemOperation::DeleteFile { .. } => "delete_file",
            FileSystemOperation::ListDirectory { .. } => "list_directory",
            FileSystemOperation::CreateDirectory { .. } => "create_directory",
            FileSystemOperation::DeleteDirectory { .. } => "delete_directory",
        };

        self.active_operations.insert(
            operation_id,
            Operation {
                id: operation_id,
                operation_type: operation_type.to_string(),
                started_at: Utc::now(),
            },
        );

        // Execute operation
        let result = match operation {
            FileSystemOperation::ReadFile { path } => self.handle_read_file(path).await,
            FileSystemOperation::WriteFile { path, content } => {
                self.handle_write_file(path, content).await
            }
            FileSystemOperation::DeleteFile { path } => self.handle_delete_file(path).await,
            FileSystemOperation::ListDirectory { path } => self.handle_list_directory(path).await,
            FileSystemOperation::CreateDirectory { path } => {
                self.handle_create_directory(path).await
            }
            FileSystemOperation::DeleteDirectory { path, recursive } => {
                self.handle_delete_directory(path, recursive).await
            }
        };

        self.active_operations.remove(&operation_id);
        result
    }

    /// Handle ReadFile operation
    async fn handle_read_file(&self, path: std::path::PathBuf) -> FileSystemResult {
        // TODO: Integrate with airssys-osl helper functions
        // For now, return mock response for compilation
        if path.exists() {
            match std::fs::read(&path) {
                Ok(content) => FileSystemResult::ReadSuccess { content },
                Err(e) => FileSystemResult::Error {
                    error: FileSystemError::IoError {
                        message: e.to_string(),
                    },
                },
            }
        } else {
            FileSystemResult::Error {
                error: FileSystemError::NotFound { path },
            }
        }
    }

    /// Handle WriteFile operation
    async fn handle_write_file(
        &self,
        path: std::path::PathBuf,
        content: Vec<u8>,
    ) -> FileSystemResult {
        // TODO: Integrate with airssys-osl helper functions
        match std::fs::write(&path, &content) {
            Ok(_) => FileSystemResult::WriteSuccess,
            Err(e) => FileSystemResult::Error {
                error: FileSystemError::IoError {
                    message: e.to_string(),
                },
            },
        }
    }

    /// Handle DeleteFile operation
    async fn handle_delete_file(&self, path: std::path::PathBuf) -> FileSystemResult {
        // TODO: Integrate with airssys-osl helper functions
        if path.exists() {
            match std::fs::remove_file(&path) {
                Ok(_) => FileSystemResult::DeleteSuccess,
                Err(e) => FileSystemResult::Error {
                    error: FileSystemError::IoError {
                        message: e.to_string(),
                    },
                },
            }
        } else {
            FileSystemResult::Error {
                error: FileSystemError::NotFound { path },
            }
        }
    }

    /// Handle ListDirectory operation
    async fn handle_list_directory(&self, path: std::path::PathBuf) -> FileSystemResult {
        // TODO: Integrate with airssys-osl helper functions
        if path.is_dir() {
            match std::fs::read_dir(&path) {
                Ok(entries) => {
                    let dir_entries: Vec<DirEntry> = entries
                        .filter_map(|e| e.ok())
                        .filter_map(|entry| {
                            let metadata = entry.metadata().ok()?;
                            Some(DirEntry {
                                path: entry.path(),
                                is_file: metadata.is_file(),
                                is_directory: metadata.is_dir(),
                                size: metadata.len(),
                            })
                        })
                        .collect();
                    FileSystemResult::ListSuccess {
                        entries: dir_entries,
                    }
                }
                Err(e) => FileSystemResult::Error {
                    error: FileSystemError::IoError {
                        message: e.to_string(),
                    },
                },
            }
        } else {
            FileSystemResult::Error {
                error: FileSystemError::NotFound { path },
            }
        }
    }

    /// Handle CreateDirectory operation
    async fn handle_create_directory(&self, path: std::path::PathBuf) -> FileSystemResult {
        // TODO: Integrate with airssys-osl helper functions
        match std::fs::create_dir_all(&path) {
            Ok(_) => FileSystemResult::CreateSuccess,
            Err(e) => FileSystemResult::Error {
                error: FileSystemError::IoError {
                    message: e.to_string(),
                },
            },
        }
    }

    /// Handle DeleteDirectory operation
    async fn handle_delete_directory(
        &self,
        path: std::path::PathBuf,
        recursive: bool,
    ) -> FileSystemResult {
        // TODO: Integrate with airssys-osl helper functions
        if path.is_dir() {
            let result = if recursive {
                std::fs::remove_dir_all(&path)
            } else {
                std::fs::remove_dir(&path)
            };

            match result {
                Ok(_) => FileSystemResult::DeleteDirectorySuccess,
                Err(e) => FileSystemResult::Error {
                    error: FileSystemError::IoError {
                        message: e.to_string(),
                    },
                },
            }
        } else {
            FileSystemResult::Error {
                error: FileSystemError::NotFound { path },
            }
        }
    }
}

// Note: No Default implementation - broker must be explicitly provided

#[async_trait]
impl<M, B> Actor for FileSystemActor<M, B>
where
    M: Message
        + serde::Serialize
        + serde::Deserialize<'static>
        + From<FileSystemResponse>
        + 'static,
    B: MessageBroker<M> + Clone + Send + Sync + 'static,
{
    type Message = FileSystemRequest;
    type Error = FileSystemError;

    async fn handle_message<Broker: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        _context: &mut ActorContext<Self::Message, Broker>,
    ) -> Result<(), Self::Error> {
        // Execute operation
        let result = self.execute_operation(message.operation).await;

        // Create response
        let response = FileSystemResponse {
            request_id: message.request_id,
            result,
        };

        // Convert response to generic message type and publish via broker
        let response_message: M = response.into();
        let envelope = MessageEnvelope::new(response_message).with_reply_to(message.reply_to);

        // Publish response through injected broker
        self.broker
            .publish(envelope)
            .await
            .map_err(|e| FileSystemError::Other {
                message: format!("Failed to publish response: {e}"),
            })?;

        Ok(())
    }

    async fn on_error<Broker: MessageBroker<Self::Message>>(
        &mut self,
        error: Self::Error,
        _context: &mut ActorContext<Self::Message, Broker>,
    ) -> ErrorAction {
        eprintln!("FileSystemActor error: {error:?}");
        ErrorAction::Resume
    }
}

#[async_trait]
impl<M, B> Child for FileSystemActor<M, B>
where
    M: Message + Send + 'static,
    B: MessageBroker<M> + Clone + Send + Sync + 'static,
{
    type Error = FileSystemError;

    async fn start(&mut self) -> Result<(), Self::Error> {
        println!("FileSystemActor starting at {}", self.created_at);
        Ok(())
    }

    async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
        // Cancel all active operations
        if !self.active_operations.is_empty() {
            println!(
                "FileSystemActor stopping with {} active operations",
                self.active_operations.len()
            );
            self.active_operations.clear();
        }
        println!(
            "FileSystemActor stopped. Total operations: {}",
            self.operation_count
        );
        Ok(())
    }

    async fn health_check(&self) -> ChildHealth {
        // Simple health check: too many active operations = degraded
        if self.active_operations.len() > 100 {
            ChildHealth::Degraded(format!(
                "Too many active operations: {}",
                self.active_operations.len()
            ))
        } else {
            ChildHealth::Healthy
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::broker::InMemoryMessageBroker;

    // Mock message type for testing
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct TestMessage;

    impl Message for TestMessage {
        const MESSAGE_TYPE: &'static str = "test_message";
        fn priority(&self) -> crate::message::MessagePriority {
            crate::message::MessagePriority::Normal
        }
    }

    impl From<FileSystemResponse> for TestMessage {
        fn from(_: FileSystemResponse) -> Self {
            TestMessage
        }
    }

    type TestFileSystemActor = FileSystemActor<TestMessage, InMemoryMessageBroker<TestMessage>>;

    #[test]
    fn test_filesystem_actor_new() {
        let broker = InMemoryMessageBroker::new();
        let actor: TestFileSystemActor = FileSystemActor::new(broker);
        assert_eq!(actor.operation_count(), 0);
        assert_eq!(actor.active_operations_count(), 0);
    }

    #[tokio::test]
    async fn test_filesystem_actor_health_check() {
        let broker = InMemoryMessageBroker::new();
        let actor: TestFileSystemActor = FileSystemActor::new(broker);
        let health = actor.health_check().await;
        assert_eq!(health, ChildHealth::Healthy);
    }

    #[tokio::test]
    async fn test_filesystem_actor_health_degraded() {
        let broker = InMemoryMessageBroker::new();
        let mut actor: TestFileSystemActor = FileSystemActor::new(broker);

        // Add many active operations to trigger degraded state
        for i in 0..101 {
            actor.active_operations.insert(
                i,
                Operation {
                    id: i,
                    operation_type: "test".to_string(),
                    started_at: Utc::now(),
                },
            );
        }

        let health = actor.health_check().await;
        assert!(matches!(health, ChildHealth::Degraded(_)));
    }
}
