//! Storage trait abstractions.
//!
//! This module contains trait definitions for component-isolated storage.
//! These traits are implemented by the storage system and consumed by
//! components via host functions.

// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)
// (none)

// Layer 3: Internal module imports (per PROJECTS_STANDARD.md §2.1)
use super::errors::StorageError;
use super::value::StorageValue;

/// Trait for component-isolated key-value storage.
///
/// `ComponentStorage` defines the interface for storage operations.
/// Each component has isolated storage namespaced by its `ComponentId`.
///
/// # Storage Isolation (Solana-Inspired)
///
/// Storage is automatically namespaced by the calling component's ID.
/// Components **cannot** access storage outside their namespace.
/// This isolation is enforced at the host/runtime level.
///
/// ```text
/// Component A calls: get("user:123")
///   → Host internally: get("component-A/user:123")
///   → Returns Component A's data only
///
/// Component B calls: get("user:123")
///   → Host internally: get("component-B/user:123")
///   → Returns Component B's data (different namespace)
/// ```
///
/// # Architecture Note
///
/// This trait is defined in `core/storage/` (Layer 1) as an abstraction.
/// Concrete implementations are provided via host functions or a storage
/// system module. This follows the Dependency Inversion Principle.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` for multi-threaded access.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::storage::traits::ComponentStorage;
/// use airssys_wasm::core::storage::errors::StorageError;
/// use airssys_wasm::core::storage::value::StorageValue;
///
/// struct MockStorage;
///
/// impl ComponentStorage for MockStorage {
///     fn get(&self, _key: &str) -> Result<Option<StorageValue>, StorageError> {
///         Ok(None)
///     }
///
///     fn set(&self, _key: &str, _value: StorageValue) -> Result<(), StorageError> {
///         Ok(())
///     }
///
///     fn delete(&self, _key: &str) -> Result<(), StorageError> {
///         Ok(())
///     }
///
///     fn exists(&self, _key: &str) -> Result<bool, StorageError> {
///         Ok(false)
///     }
///
///     fn list_keys(&self, _prefix: Option<&str>) -> Result<Vec<String>, StorageError> {
///         Ok(vec![])
///     }
/// }
/// ```
pub trait ComponentStorage: Send + Sync {
    /// Gets a value by key.
    ///
    /// Returns `None` if the key does not exist in storage.
    ///
    /// # Arguments
    ///
    /// * `key` - The storage key to retrieve
    ///
    /// # Returns
    ///
    /// * `Ok(Some(value))` - Key exists, returns value
    /// * `Ok(None)` - Key does not exist
    /// * `Err(StorageError)` - Operation failed
    ///
    /// # Errors
    ///
    /// - `StorageError::InvalidKey` - Key format is invalid
    /// - `StorageError::IoError` - I/O operation failed
    fn get(&self, key: &str) -> Result<Option<StorageValue>, StorageError>;

    /// Sets a value by key.
    ///
    /// Creates the key if it doesn't exist, overwrites if it does.
    ///
    /// # Arguments
    ///
    /// * `key` - The storage key to set
    /// * `value` - The value to store
    ///
    /// # Errors
    ///
    /// - `StorageError::QuotaExceeded` - Storage quota exceeded
    /// - `StorageError::InvalidKey` - Key format is invalid
    /// - `StorageError::IoError` - I/O operation failed
    fn set(&self, key: &str, value: StorageValue) -> Result<(), StorageError>;

    /// Deletes a value by key.
    ///
    /// No-op if the key doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `key` - The storage key to delete
    ///
    /// # Errors
    ///
    /// - `StorageError::InvalidKey` - Key format is invalid
    /// - `StorageError::IoError` - I/O operation failed
    fn delete(&self, key: &str) -> Result<(), StorageError>;

    /// Checks if a key exists in storage.
    ///
    /// # Arguments
    ///
    /// * `key` - The storage key to check
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Key exists
    /// * `Ok(false)` - Key does not exist
    ///
    /// # Errors
    ///
    /// - `StorageError::InvalidKey` - Key format is invalid
    /// - `StorageError::IoError` - I/O operation failed
    fn exists(&self, key: &str) -> Result<bool, StorageError>;

    /// Lists keys with an optional prefix filter.
    ///
    /// # Arguments
    ///
    /// * `prefix` - Optional prefix to filter keys. If `None`, returns all keys.
    ///
    /// # Returns
    ///
    /// A vector of matching key names.
    ///
    /// # Errors
    ///
    /// - `StorageError::IoError` - I/O operation failed
    fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>, StorageError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct MockStorage {
        data: Mutex<HashMap<String, StorageValue>>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                data: Mutex::new(HashMap::new()),
            }
        }
    }

    impl ComponentStorage for MockStorage {
        fn get(&self, key: &str) -> Result<Option<StorageValue>, StorageError> {
            Ok(self.data.lock().unwrap().get(key).cloned())
        }

        fn set(&self, key: &str, value: StorageValue) -> Result<(), StorageError> {
            self.data.lock().unwrap().insert(key.to_string(), value);
            Ok(())
        }

        fn delete(&self, key: &str) -> Result<(), StorageError> {
            self.data.lock().unwrap().remove(key);
            Ok(())
        }

        fn exists(&self, key: &str) -> Result<bool, StorageError> {
            Ok(self.data.lock().unwrap().contains_key(key))
        }

        fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>, StorageError> {
            let data = self.data.lock().unwrap();
            let keys: Vec<String> = match prefix {
                Some(p) => data.keys().filter(|k| k.starts_with(p)).cloned().collect(),
                None => data.keys().cloned().collect(),
            };
            Ok(keys)
        }
    }

    #[test]
    fn test_component_storage_is_send_sync() {
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}
        assert_send_sync::<dyn ComponentStorage>();
    }

    #[test]
    fn test_mock_storage_set_and_get() {
        let storage = MockStorage::new();
        let value = StorageValue::new(vec![1, 2, 3]);

        storage.set("key1", value.clone()).unwrap();
        let retrieved = storage.get("key1").unwrap();

        assert_eq!(retrieved, Some(value));
    }

    #[test]
    fn test_mock_storage_get_nonexistent() {
        let storage = MockStorage::new();
        let result = storage.get("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_mock_storage_delete() {
        let storage = MockStorage::new();
        storage.set("key1", StorageValue::new(vec![1])).unwrap();

        assert!(storage.exists("key1").unwrap());
        storage.delete("key1").unwrap();
        assert!(!storage.exists("key1").unwrap());
    }

    #[test]
    fn test_mock_storage_exists() {
        let storage = MockStorage::new();

        assert!(!storage.exists("key1").unwrap());
        storage.set("key1", StorageValue::new(vec![1])).unwrap();
        assert!(storage.exists("key1").unwrap());
    }

    #[test]
    fn test_mock_storage_list_keys() {
        let storage = MockStorage::new();
        storage.set("user:1", StorageValue::new(vec![1])).unwrap();
        storage.set("user:2", StorageValue::new(vec![2])).unwrap();
        storage
            .set("config:main", StorageValue::new(vec![3]))
            .unwrap();

        let user_keys = storage.list_keys(Some("user:")).unwrap();
        assert_eq!(user_keys.len(), 2);

        let all_keys = storage.list_keys(None).unwrap();
        assert_eq!(all_keys.len(), 3);
    }

    #[test]
    fn test_mock_storage_overwrite() {
        let storage = MockStorage::new();
        storage.set("key1", StorageValue::new(vec![1])).unwrap();

        let retrieved = storage.get("key1").unwrap();
        assert_eq!(retrieved.unwrap().as_bytes(), &[1]);

        storage.set("key1", StorageValue::new(vec![2])).unwrap();

        let retrieved = storage.get("key1").unwrap();
        assert_eq!(retrieved.unwrap().as_bytes(), &[2]);
    }

    #[test]
    fn test_mock_storage_delete_nonexistent() {
        let storage = MockStorage::new();
        // Should not error on deleting non-existent key
        storage.delete("nonexistent").unwrap();
    }
}
