//! Storage error types.
//!
//! This module contains error types for component storage operations.
//! These errors are co-located with the storage module per ADR-WASM-028.

// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)
// (none needed for this module)

// Layer 2: Third-party crate imports (per PROJECTS_STANDARD.md §2.1)
use thiserror::Error;

// Layer 3: Internal module imports (per PROJECTS_STANDARD.md §2.1)
// (none - errors have no internal dependencies)

/// Storage errors for component storage operations.
///
/// `StorageError` represents errors from key-value storage operations.
/// Aligned with WIT `storage-error` variant in `errors.wit`.
///
/// # Variants
///
/// - `NotFound` - Key does not exist in storage
/// - `AlreadyExists` - Key already exists (for create-only operations)
/// - `QuotaExceeded` - Storage quota for component exceeded
/// - `InvalidKey` - Key format is invalid
/// - `IoError` - Underlying I/O operation failed
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::storage::errors::StorageError;
///
/// let err = StorageError::NotFound("user:123".to_string());
/// assert!(format!("{}", err).contains("not found"));
/// ```
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum StorageError {
    /// Key not found in storage.
    #[error("Key not found: {0}")]
    NotFound(String),

    /// Key already exists (for create-only operations).
    #[error("Key already exists: {0}")]
    AlreadyExists(String),

    /// Component storage quota exceeded.
    #[error("Storage quota exceeded")]
    QuotaExceeded,

    /// Invalid key format.
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    /// Underlying I/O error.
    #[error("Storage I/O error: {0}")]
    IoError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_display() {
        let err = StorageError::NotFound("user:123".to_string());
        assert_eq!(format!("{}", err), "Key not found: user:123");
    }

    #[test]
    fn test_already_exists_display() {
        let err = StorageError::AlreadyExists("config:main".to_string());
        assert_eq!(format!("{}", err), "Key already exists: config:main");
    }

    #[test]
    fn test_quota_exceeded_display() {
        let err = StorageError::QuotaExceeded;
        assert_eq!(format!("{}", err), "Storage quota exceeded");
    }

    #[test]
    fn test_invalid_key_display() {
        let err = StorageError::InvalidKey("bad\\key".to_string());
        assert_eq!(format!("{}", err), "Invalid key: bad\\key");
    }

    #[test]
    fn test_io_error_display() {
        let err = StorageError::IoError("disk full".to_string());
        assert_eq!(format!("{}", err), "Storage I/O error: disk full");
    }

    #[test]
    fn test_error_is_clone() {
        let err = StorageError::QuotaExceeded;
        let cloned = err.clone();
        assert!(matches!(cloned, StorageError::QuotaExceeded));
    }

    #[test]
    fn test_error_is_debug() {
        let err = StorageError::NotFound("test".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("NotFound"));
    }

    #[test]
    fn test_error_equality() {
        let err1 = StorageError::QuotaExceeded;
        let err2 = StorageError::QuotaExceeded;
        let err3 = StorageError::NotFound("x".to_string());

        assert_eq!(err1, err2);
        assert_ne!(err1, err3);
    }
}
