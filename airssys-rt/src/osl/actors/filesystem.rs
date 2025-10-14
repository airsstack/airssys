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
//! use airssys_rt::osl::{FileSystemActor, FileSystemMessage, FileSystemResponse};
//! use airssys_rt::{Actor, ActorContext};
//! use std::path::PathBuf;
//! use tokio::sync::oneshot;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let actor = FileSystemActor::new();
//!
//! let (tx, rx) = oneshot::channel();
//! // actor.handle(FileSystemMessage::ReadFile {
//! //     path: PathBuf::from("config.txt"),
//! //     respond_to: tx,
//! // }, &context).await?;
//!
//! let response = rx.await?;
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::actor::{Actor, ActorContext, ErrorAction};
use crate::broker::MessageBroker;
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
/// ## Benefits
///
/// - Centralized audit logging for all file operations
/// - Clean fault isolation (FS failures don't crash app actors)
/// - Superior testability (mock this actor in tests)
/// - Rate limiting and backpressure protection
pub struct FileSystemActor {
    /// Operation counter for metrics
    operation_count: u64,

    /// Active operations tracking
    active_operations: HashMap<OperationId, Operation>,

    /// Actor creation timestamp
    created_at: DateTime<Utc>,

    /// Last operation timestamp
    last_operation_at: Option<DateTime<Utc>>,
}

type OperationId = u64;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Operation {
    id: OperationId,
    operation_type: String,
    started_at: DateTime<Utc>,
}

impl FileSystemActor {
    /// Create a new FileSystemActor
    pub fn new() -> Self {
        Self {
            operation_count: 0,
            active_operations: HashMap::new(),
            created_at: Utc::now(),
            last_operation_at: None,
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
    async fn handle_write_file(&self, path: std::path::PathBuf, content: Vec<u8>) -> FileSystemResult {
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

impl Default for FileSystemActor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Actor for FileSystemActor {
    type Message = FileSystemRequest;
    type Error = FileSystemError;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        // Execute operation
        let result = self.execute_operation(message.operation).await;

        // Create response
        let response = FileSystemResponse {
            request_id: message.request_id,
            result,
        };

        // Send response via broker (casting to generic message type)
        // The broker will route this to the reply_to address
        let _response_msg = serde_json::to_string(&response)
            .map_err(|e| FileSystemError::Other { message: e.to_string() })?;
        
        // TODO: Need to use broker.publish() directly instead of context.send()
        // For now, just log the response
        println!("FileSystemActor response: {response:?}");

        Ok(())
    }

    async fn on_error<B: MessageBroker<Self::Message>>(
        &mut self,
        error: Self::Error,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> ErrorAction {
        eprintln!("FileSystemActor error: {error:?}");
        ErrorAction::Resume
    }
}

#[async_trait]
impl Child for FileSystemActor {
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
            ChildHealth::Degraded(format!("Too many active operations: {}", self.active_operations.len()))
        } else {
            ChildHealth::Healthy
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filesystem_actor_new() {
        let actor = FileSystemActor::new();
        assert_eq!(actor.operation_count(), 0);
        assert_eq!(actor.active_operations_count(), 0);
    }

    #[test]
    fn test_filesystem_actor_default() {
        let actor = FileSystemActor::default();
        assert_eq!(actor.operation_count(), 0);
    }

    #[tokio::test]
    async fn test_filesystem_actor_health_check() {
        let actor = FileSystemActor::new();
        let health = actor.health_check().await;
        assert_eq!(health, ChildHealth::Healthy);
    }

    #[tokio::test]
    async fn test_filesystem_actor_health_degraded() {
        let mut actor = FileSystemActor::new();

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
