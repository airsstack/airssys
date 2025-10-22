//! Storage backend abstractions for WASM component persistence.
//!
//! These types define the storage backend trait, operation types, transaction
//! abstraction, and namespace isolation needed for Block 6 (Component Storage).
//! They provide a NEAR-style key-value API foundation without backend-specific
//! implementation details, following YAGNI principles (§6.1).
//!
//! # Design Rationale
//!
//! - **StorageBackend**: Async trait for pluggable storage implementations (Sled,
//!   RocksDB, custom). Uses namespace parameter for component isolation via key
//!   prefixing (`component:<id>:key`). Generic over backend for zero-cost abstraction.
//!
//! - **StorageOperation**: Enum representing storage operations for transaction
//!   batching and audit logging. Contains all data needed for execution.
//!
//! - **StorageTransaction**: Async trait for atomic multi-operation transactions.
//!   Uses Box<dyn> pattern (§6.2 exception) for heap-allocated transaction state.
//!
//! All types are async-first (async_trait) for non-blocking I/O and integration
//! with tokio runtime (airssys-rt foundation).
//!
//! # Architecture
//!
//! Storage flow follows NEAR-style KV pattern:
//! 1. Component calls storage host function (get/set/delete/scan_prefix)
//! 2. Host runtime validates capabilities and quotas
//! 3. Storage Manager prefixes key with component ID for isolation
//! 4. Backend implementation handles actual I/O (Sled/RocksDB)
//!
//! Performance targets: <1ms per operation (ADR-WASM-007)
//!
//! # Backend Selection
//!
//! - **Sled (Default)**: Pure Rust, zero C++ dependencies, fast compilation
//! - **RocksDB (Optional)**: Battle-tested, production stability, C++ dependency
//! - **Custom**: Implement StorageBackend trait for specialized needs
//!
//! See KNOWLEDGE-WASM-008 for comprehensive backend comparison.
//!
//! # Namespace Isolation
//!
//! Components are isolated via key prefixing:
//! - Component A stores "config" → "component:a:config"
//! - Component B stores "config" → "component:b:config"
//! - No cross-component access possible
//!
//! # References
//!
//! - ADR-WASM-007: Storage Backend Selection (Sled/RocksDB decision)
//! - KNOWLEDGE-WASM-007: Component Storage Architecture (NEAR-style API)
//! - KNOWLEDGE-WASM-008: Storage Backend Comparison (Sled vs RocksDB)

// Layer 2: External crates
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// Layer 3: Internal (core only)
use crate::core::error::WasmResult;

/// Storage backend trait for component persistence.
///
/// Defines the contract for pluggable storage implementations providing
/// key-value storage with namespace isolation. Backends handle raw byte
/// storage while the runtime manages component-specific namespacing via
/// key prefixing (`component:<id>:key`).
///
/// # NEAR-Style API
///
/// The API follows NEAR Protocol's storage patterns:
/// - Simple KV operations (get, set, delete)
/// - Prefix-based key scanning for iteration
/// - Namespace isolation via key prefixing
/// - Binary keys and values (no schema constraints)
///
/// # Async Design
///
/// All methods are async to support non-blocking I/O and integration with
/// airssys-rt actor system. Implementations should use tokio for I/O operations.
///
/// # Backend Implementations
///
/// - **SledBackend**: Pure Rust embedded database (default, fast compilation)
/// - **RocksDBBackend**: Production-proven LSM database (optional, C++ dependency)
/// - **MemoryBackend**: In-memory storage for testing
/// - **CustomBackend**: Application-specific storage (e.g., cloud services)
///
/// # Performance Requirements
///
/// Based on ADR-WASM-007 targets:
/// - get/set/delete: <1ms for values up to 1MB
/// - list_keys: <10ms for up to 1000 keys
/// - Transaction commit: <5ms for up to 100 operations
///
/// # Example
///
/// ```
/// use airssys_wasm::core::storage::StorageBackend;
/// use airssys_wasm::core::error::WasmResult;
/// use async_trait::async_trait;
///
/// struct MemoryBackend;
///
/// #[async_trait]
/// impl StorageBackend for MemoryBackend {
///     async fn get(&self, namespace: &str, key: &[u8]) -> WasmResult<Option<Vec<u8>>> {
///         Ok(None)
///     }
///
///     async fn set(&self, namespace: &str, key: &[u8], value: &[u8]) -> WasmResult<()> {
///         Ok(())
///     }
///
///     async fn delete(&self, namespace: &str, key: &[u8]) -> WasmResult<()> {
///         Ok(())
///     }
///
///     async fn list_keys(&self, namespace: &str, prefix: &[u8]) -> WasmResult<Vec<Vec<u8>>> {
///         Ok(vec![])
///     }
///
///     async fn begin_transaction(&self) -> WasmResult<Box<dyn airssys_wasm::core::storage::StorageTransaction>> {
///         unimplemented!()
///     }
/// }
/// ```
///
/// # References
///
/// - KNOWLEDGE-WASM-007 §5-§6: Storage Architecture and Backend Integration
/// - KNOWLEDGE-WASM-008: Backend Comparison (Sled vs RocksDB)
/// - ADR-WASM-007: Storage Backend Selection
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Get value for a key in the namespace.
    ///
    /// Returns `Some(value)` if key exists, `None` if key not found.
    /// The namespace parameter is already component-scoped by the runtime
    /// (e.g., "component:my-app").
    ///
    /// # Parameters
    ///
    /// - `namespace`: Component-specific namespace (pre-prefixed by runtime)
    /// - `key`: Raw byte key to retrieve
    ///
    /// # Returns
    ///
    /// - `Ok(Some(value))`: Key exists, value returned
    /// - `Ok(None)`: Key does not exist
    /// - `Err(WasmError)`: I/O error or backend failure
    ///
    /// # Performance Target
    ///
    /// <1ms for values up to 1MB (ADR-WASM-007)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use airssys_wasm::core::storage::StorageBackend;
    /// # async fn example(backend: &dyn StorageBackend) {
    /// let value = backend.get("component:my-app", b"config").await.unwrap();
    /// if let Some(data) = value {
    ///     println!("Config: {} bytes", data.len());
    /// }
    /// # }
    /// ```
    async fn get(&self, namespace: &str, key: &[u8]) -> WasmResult<Option<Vec<u8>>>;

    /// Set value for a key in the namespace.
    ///
    /// Overwrites existing value if key exists. Creates key if not found.
    /// Subject to component quota limits enforced by the runtime.
    ///
    /// # Parameters
    ///
    /// - `namespace`: Component-specific namespace (pre-prefixed by runtime)
    /// - `key`: Raw byte key to set
    /// - `value`: Raw byte value to store
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Value successfully stored
    /// - `Err(WasmError::QuotaExceeded)`: Component storage quota exceeded
    /// - `Err(WasmError)`: I/O error or backend failure
    ///
    /// # Performance Target
    ///
    /// <1ms for values up to 1MB (ADR-WASM-007)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use airssys_wasm::core::storage::StorageBackend;
    /// # async fn example(backend: &dyn StorageBackend) {
    /// backend.set("component:my-app", b"config", b"{\"key\":\"value\"}").await.unwrap();
    /// # }
    /// ```
    async fn set(&self, namespace: &str, key: &[u8], value: &[u8]) -> WasmResult<()>;

    /// Delete a key from the namespace.
    ///
    /// No-op if key doesn't exist (not an error). Deletes free up storage
    /// quota for the component.
    ///
    /// # Parameters
    ///
    /// - `namespace`: Component-specific namespace (pre-prefixed by runtime)
    /// - `key`: Raw byte key to delete
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Key deleted or didn't exist
    /// - `Err(WasmError)`: I/O error or backend failure
    ///
    /// # Performance Target
    ///
    /// <1ms (ADR-WASM-007)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use airssys_wasm::core::storage::StorageBackend;
    /// # async fn example(backend: &dyn StorageBackend) {
    /// backend.delete("component:my-app", b"old_config").await.unwrap();
    /// # }
    /// ```
    async fn delete(&self, namespace: &str, key: &[u8]) -> WasmResult<()>;

    /// List all keys with a given prefix in the namespace.
    ///
    /// Returns all keys starting with `prefix`. Empty prefix returns all keys
    /// in the namespace. Used for iterating over component data.
    ///
    /// # Parameters
    ///
    /// - `namespace`: Component-specific namespace (pre-prefixed by runtime)
    /// - `prefix`: Key prefix to match (empty for all keys)
    ///
    /// # Returns
    ///
    /// - `Ok(keys)`: Vector of matching keys (may be empty)
    /// - `Err(WasmError)`: I/O error or backend failure
    ///
    /// # Performance Target
    ///
    /// <10ms for up to 1000 keys (ADR-WASM-007)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use airssys_wasm::core::storage::StorageBackend;
    /// # async fn example(backend: &dyn StorageBackend) {
    /// let keys = backend.list_keys("component:my-app", b"user:").await.unwrap();
    /// for key in keys {
    ///     println!("Found key: {:?}", key);
    /// }
    /// # }
    /// ```
    async fn list_keys(&self, namespace: &str, prefix: &[u8]) -> WasmResult<Vec<Vec<u8>>>;

    /// Begin a new transaction for atomic operations.
    ///
    /// Creates a transaction that can batch multiple operations (get/set/delete)
    /// for atomic commit. Not all backends support transactions (e.g., simple
    /// file-based storage).
    ///
    /// # Returns
    ///
    /// - `Ok(transaction)`: Transaction ready for operations
    /// - `Err(WasmError::Unsupported)`: Backend doesn't support transactions
    /// - `Err(WasmError)`: Backend failure
    ///
    /// # Performance Target
    ///
    /// <5ms commit for up to 100 operations (ADR-WASM-007)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use airssys_wasm::core::storage::{StorageBackend, StorageOperation};
    /// # async fn example(backend: &dyn StorageBackend) {
    /// let mut txn = backend.begin_transaction().await.unwrap();
    /// txn.add_operation(StorageOperation::Set {
    ///     namespace: "component:my-app".to_string(),
    ///     key: b"key1".to_vec(),
    ///     value: b"value1".to_vec(),
    /// }).await.unwrap();
    /// txn.commit().await.unwrap();
    /// # }
    /// ```
    async fn begin_transaction(&self) -> WasmResult<Box<dyn StorageTransaction>>;
}

/// Storage operation for transaction batching and audit logging.
///
/// Represents a single storage operation (get/set/delete/list) with all
/// data needed for execution. Used for:
/// - Batching operations in transactions
/// - Audit logging of storage activity
/// - Undo/redo capabilities (future)
///
/// Each variant contains the namespace and operation-specific data.
///
/// # Example
///
/// ```
/// use airssys_wasm::core::storage::StorageOperation;
///
/// let set_op = StorageOperation::Set {
///     namespace: "component:my-app".to_string(),
///     key: b"config".to_vec(),
///     value: b"{\"debug\":true}".to_vec(),
/// };
///
/// assert_eq!(set_op.namespace(), "component:my-app");
/// assert_eq!(set_op.operation_type(), "set");
///
/// let get_op = StorageOperation::Get {
///     namespace: "component:my-app".to_string(),
///     key: b"config".to_vec(),
/// };
///
/// assert_eq!(get_op.operation_type(), "get");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageOperation {
    /// Get operation: retrieve value for key.
    Get {
        /// Component namespace.
        namespace: String,
        /// Key to retrieve.
        key: Vec<u8>,
    },

    /// Set operation: store value for key.
    Set {
        /// Component namespace.
        namespace: String,
        /// Key to set.
        key: Vec<u8>,
        /// Value to store.
        value: Vec<u8>,
    },

    /// Delete operation: remove key.
    Delete {
        /// Component namespace.
        namespace: String,
        /// Key to delete.
        key: Vec<u8>,
    },

    /// List operation: scan keys with prefix.
    List {
        /// Component namespace.
        namespace: String,
        /// Key prefix (empty for all keys).
        prefix: Vec<u8>,
    },
}

impl StorageOperation {
    /// Get the namespace for this operation.
    ///
    /// All operations include a namespace field for component isolation.
    ///
    /// # Example
    ///
    /// ```
    /// use airssys_wasm::core::storage::StorageOperation;
    ///
    /// let op = StorageOperation::Set {
    ///     namespace: "component:my-app".to_string(),
    ///     key: b"key".to_vec(),
    ///     value: b"value".to_vec(),
    /// };
    ///
    /// assert_eq!(op.namespace(), "component:my-app");
    /// ```
    pub fn namespace(&self) -> &str {
        match self {
            Self::Get { namespace, .. }
            | Self::Set { namespace, .. }
            | Self::Delete { namespace, .. }
            | Self::List { namespace, .. } => namespace,
        }
    }

    /// Get the operation type as a string.
    ///
    /// Returns one of: "get", "set", "delete", "list"
    ///
    /// # Example
    ///
    /// ```
    /// use airssys_wasm::core::storage::StorageOperation;
    ///
    /// let op = StorageOperation::Delete {
    ///     namespace: "component:test".to_string(),
    ///     key: b"old_data".to_vec(),
    /// };
    ///
    /// assert_eq!(op.operation_type(), "delete");
    /// ```
    pub fn operation_type(&self) -> &'static str {
        match self {
            Self::Get { .. } => "get",
            Self::Set { .. } => "set",
            Self::Delete { .. } => "delete",
            Self::List { .. } => "list",
        }
    }
}

/// Transaction trait for atomic multi-operation storage.
///
/// Provides batch execution of storage operations with atomicity guarantees.
/// Operations are added to the transaction, then committed (apply all) or
/// rolled back (discard all) as a unit.
///
/// # Atomicity Semantics
///
/// - **Add**: Operations are queued but not executed immediately
/// - **Commit**: All operations execute atomically (all succeed or all fail)
/// - **Rollback**: All pending operations discarded, no changes applied
///
/// # Box<dyn> Pattern (§6.2 Exception)
///
/// The trait uses `Box<Self>` for commit/rollback to consume the transaction
/// and prevent reuse. This is a justified exception to §6.2 (avoid dyn) because:
/// - Transaction state must be heap-allocated (varies by backend)
/// - One-time use pattern (commit/rollback consumes transaction)
/// - Cannot use generics (trait object needed for runtime polymorphism)
///
/// # Example
///
/// ```no_run
/// # use airssys_wasm::core::storage::{StorageBackend, StorageOperation};
/// # async fn example(backend: &dyn StorageBackend) {
/// let mut txn = backend.begin_transaction().await.unwrap();
///
/// txn.add_operation(StorageOperation::Set {
///     namespace: "component:my-app".to_string(),
///     key: b"key1".to_vec(),
///     value: b"value1".to_vec(),
/// }).await.unwrap();
///
/// txn.add_operation(StorageOperation::Set {
///     namespace: "component:my-app".to_string(),
///     key: b"key2".to_vec(),
///     value: b"value2".to_vec(),
/// }).await.unwrap();
///
/// txn.commit().await.unwrap();
/// # }
/// ```
///
/// # References
///
/// - KNOWLEDGE-WASM-007 §6: Transaction Support
/// - ADR-WASM-007: Storage Backend Selection (transaction requirements)
#[async_trait]
pub trait StorageTransaction: Send + Sync {
    /// Add an operation to the transaction batch.
    ///
    /// Operations are queued but not executed until commit(). Multiple operations
    /// can be added before committing. Operations are executed in the order added.
    ///
    /// # Parameters
    ///
    /// - `op`: Storage operation to add (Get/Set/Delete/List)
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Operation added to batch
    /// - `Err(WasmError)`: Operation invalid or batch full
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use airssys_wasm::core::storage::{StorageTransaction, StorageOperation};
    /// # async fn example(txn: &mut dyn StorageTransaction) {
    /// txn.add_operation(StorageOperation::Set {
    ///     namespace: "component:test".to_string(),
    ///     key: b"counter".to_vec(),
    ///     value: b"42".to_vec(),
    /// }).await.unwrap();
    /// # }
    /// ```
    async fn add_operation(&mut self, op: StorageOperation) -> WasmResult<()>;

    /// Commit all pending operations atomically.
    ///
    /// Executes all operations in order. If any operation fails, the entire
    /// transaction is rolled back and no changes are applied. Consumes the
    /// transaction (cannot be reused after commit).
    ///
    /// # Returns
    ///
    /// - `Ok(())`: All operations succeeded
    /// - `Err(WasmError)`: At least one operation failed (all rolled back)
    ///
    /// # Performance Target
    ///
    /// <5ms for up to 100 operations (ADR-WASM-007)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use airssys_wasm::core::storage::{StorageBackend, StorageOperation};
    /// # async fn example(backend: &dyn StorageBackend) {
    /// let mut txn = backend.begin_transaction().await.unwrap();
    /// txn.add_operation(StorageOperation::Set {
    ///     namespace: "component:test".to_string(),
    ///     key: b"key".to_vec(),
    ///     value: b"value".to_vec(),
    /// }).await.unwrap();
    ///
    /// txn.commit().await.unwrap();
    /// # }
    /// ```
    async fn commit(self: Box<Self>) -> WasmResult<()>;

    /// Rollback all pending operations.
    ///
    /// Discards all operations without executing them. Use for explicit abort
    /// or error handling. Consumes the transaction (cannot be reused after rollback).
    ///
    /// # Returns
    ///
    /// Always returns `Ok(())` (rollback cannot fail)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use airssys_wasm::core::storage::{StorageBackend, StorageOperation};
    /// # async fn example(backend: &dyn StorageBackend) {
    /// let mut txn = backend.begin_transaction().await.unwrap();
    /// txn.add_operation(StorageOperation::Set {
    ///     namespace: "component:test".to_string(),
    ///     key: b"temp".to_vec(),
    ///     value: b"data".to_vec(),
    /// }).await.unwrap();
    ///
    /// txn.rollback().await.unwrap();
    /// # }
    /// ```
    async fn rollback(self: Box<Self>) -> WasmResult<()>;
}

#[cfg(test)]
#[allow(clippy::panic)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_operation_get_creation() {
        let op = StorageOperation::Get {
            namespace: "component:test".to_string(),
            key: b"config".to_vec(),
        };

        assert_eq!(op.namespace(), "component:test");
        assert_eq!(op.operation_type(), "get");
    }

    #[test]
    fn test_storage_operation_set_creation() {
        let op = StorageOperation::Set {
            namespace: "component:app".to_string(),
            key: b"key1".to_vec(),
            value: b"value1".to_vec(),
        };

        assert_eq!(op.namespace(), "component:app");
        assert_eq!(op.operation_type(), "set");
    }

    #[test]
    fn test_storage_operation_delete_creation() {
        let op = StorageOperation::Delete {
            namespace: "component:service".to_string(),
            key: b"old_data".to_vec(),
        };

        assert_eq!(op.namespace(), "component:service");
        assert_eq!(op.operation_type(), "delete");
    }

    #[test]
    fn test_storage_operation_list_creation() {
        let op = StorageOperation::List {
            namespace: "component:cache".to_string(),
            prefix: b"user:".to_vec(),
        };

        assert_eq!(op.namespace(), "component:cache");
        assert_eq!(op.operation_type(), "list");
    }

    #[test]
    fn test_storage_operation_namespace() {
        let ops = vec![
            StorageOperation::Get {
                namespace: "ns1".to_string(),
                key: vec![],
            },
            StorageOperation::Set {
                namespace: "ns2".to_string(),
                key: vec![],
                value: vec![],
            },
            StorageOperation::Delete {
                namespace: "ns3".to_string(),
                key: vec![],
            },
            StorageOperation::List {
                namespace: "ns4".to_string(),
                prefix: vec![],
            },
        ];

        assert_eq!(ops[0].namespace(), "ns1");
        assert_eq!(ops[1].namespace(), "ns2");
        assert_eq!(ops[2].namespace(), "ns3");
        assert_eq!(ops[3].namespace(), "ns4");
    }

    #[test]
    fn test_storage_operation_type() {
        let get_op = StorageOperation::Get {
            namespace: "test".to_string(),
            key: vec![],
        };
        let set_op = StorageOperation::Set {
            namespace: "test".to_string(),
            key: vec![],
            value: vec![],
        };
        let delete_op = StorageOperation::Delete {
            namespace: "test".to_string(),
            key: vec![],
        };
        let list_op = StorageOperation::List {
            namespace: "test".to_string(),
            prefix: vec![],
        };

        assert_eq!(get_op.operation_type(), "get");
        assert_eq!(set_op.operation_type(), "set");
        assert_eq!(delete_op.operation_type(), "delete");
        assert_eq!(list_op.operation_type(), "list");
    }

    #[test]
    fn test_storage_operation_serialization() {
        let op = StorageOperation::Set {
            namespace: "component:test".to_string(),
            key: b"key".to_vec(),
            value: b"value".to_vec(),
        };

        let json = serde_json::to_value(&op).unwrap_or_else(|e| {
            panic!("serialization should succeed: {e}")
        });
        assert_eq!(json["Set"]["namespace"], "component:test");

        let deserialized: StorageOperation =
            serde_json::from_value(json).unwrap_or_else(|e| {
                panic!("deserialization should succeed: {e}")
            });
        assert_eq!(deserialized.namespace(), "component:test");
        assert_eq!(deserialized.operation_type(), "set");
    }

    struct MockBackend;

    #[async_trait]
    impl StorageBackend for MockBackend {
        async fn get(&self, _namespace: &str, _key: &[u8]) -> WasmResult<Option<Vec<u8>>> {
            Ok(None)
        }

        async fn set(&self, _namespace: &str, _key: &[u8], _value: &[u8]) -> WasmResult<()> {
            Ok(())
        }

        async fn delete(&self, _namespace: &str, _key: &[u8]) -> WasmResult<()> {
            Ok(())
        }

        async fn list_keys(&self, _namespace: &str, _prefix: &[u8]) -> WasmResult<Vec<Vec<u8>>> {
            Ok(vec![])
        }

        async fn begin_transaction(&self) -> WasmResult<Box<dyn StorageTransaction>> {
            Ok(Box::new(MockTransaction))
        }
    }

    struct MockTransaction;

    #[async_trait]
    impl StorageTransaction for MockTransaction {
        async fn add_operation(&mut self, _op: StorageOperation) -> WasmResult<()> {
            Ok(())
        }

        async fn commit(self: Box<Self>) -> WasmResult<()> {
            Ok(())
        }

        async fn rollback(self: Box<Self>) -> WasmResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_storage_backend_trait_object() {
        let backend: Box<dyn StorageBackend> = Box::new(MockBackend);

        let result = backend.get("component:test", b"key").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap_or_else(|e| {
            panic!("get should succeed: {e}")
        }), None);

        let result = backend.set("component:test", b"key", b"value").await;
        assert!(result.is_ok());

        let result = backend.delete("component:test", b"key").await;
        assert!(result.is_ok());

        let result = backend.list_keys("component:test", b"prefix").await;
        assert!(result.is_ok());
        assert!(result.unwrap_or_else(|e| {
            panic!("list_keys should succeed: {e}")
        }).is_empty());
    }

    #[tokio::test]
    async fn test_storage_backend_async_methods() {
        let backend = MockBackend;

        assert!(backend.get("ns", b"k").await.is_ok());
        assert!(backend.set("ns", b"k", b"v").await.is_ok());
        assert!(backend.delete("ns", b"k").await.is_ok());
        assert!(backend.list_keys("ns", b"").await.is_ok());
    }

    #[tokio::test]
    async fn test_storage_transaction_trait_object() {
        let backend = MockBackend;
        let mut txn = backend.begin_transaction().await.unwrap_or_else(|e| {
            panic!("begin_transaction should succeed: {e}")
        });

        let op = StorageOperation::Set {
            namespace: "component:test".to_string(),
            key: b"key".to_vec(),
            value: b"value".to_vec(),
        };

        assert!(txn.add_operation(op).await.is_ok());
        assert!(txn.commit().await.is_ok());
    }

    #[tokio::test]
    async fn test_storage_transaction_lifecycle() {
        let backend = MockBackend;
        let mut txn = backend.begin_transaction().await.unwrap_or_else(|e| {
            panic!("begin_transaction should succeed: {e}")
        });

        txn.add_operation(StorageOperation::Set {
            namespace: "component:test".to_string(),
            key: b"k1".to_vec(),
            value: b"v1".to_vec(),
        })
        .await
        .unwrap_or_else(|e| {
            panic!("add_operation should succeed: {e}")
        });

        txn.add_operation(StorageOperation::Set {
            namespace: "component:test".to_string(),
            key: b"k2".to_vec(),
            value: b"v2".to_vec(),
        })
        .await
        .unwrap_or_else(|e| {
            panic!("add_operation should succeed: {e}")
        });

        assert!(txn.commit().await.is_ok());
    }

    #[tokio::test]
    async fn test_storage_transaction_rollback() {
        let backend = MockBackend;
        let mut txn = backend.begin_transaction().await.unwrap_or_else(|e| {
            panic!("begin_transaction should succeed: {e}")
        });

        txn.add_operation(StorageOperation::Delete {
            namespace: "component:test".to_string(),
            key: b"temp".to_vec(),
        })
        .await
        .unwrap_or_else(|e| {
            panic!("add_operation should succeed: {e}")
        });

        assert!(txn.rollback().await.is_ok());
    }
}
