//! Storage value type.
//!
//! This module contains the `StorageValue` ADT for component storage.
//! This type is exclusively for storage operations, providing domain
//! boundary clarity separate from messaging types.

// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)
// (none needed for this module)

// Layer 2: Third-party crate imports (per PROJECTS_STANDARD.md §2.1)
// (none needed for this module)

// Layer 3: Internal module imports (per PROJECTS_STANDARD.md §2.1)
// (none - value type has no internal dependencies)

/// Storage value wrapper for raw bytes.
///
/// `StorageValue` wraps raw bytes for component storage operations.
/// This type is semantically distinct from `MessagePayload` - it is
/// used exclusively for storage get/set operations.
///
/// # Architecture Note
///
/// Per domain boundary principle, storage values have their own type
/// even though the underlying representation is similar to other
/// byte wrappers. This ensures engineers know the type's purpose.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::storage::value::StorageValue;
///
/// // Create from Vec<u8>
/// let value = StorageValue::new(vec![1, 2, 3, 4]);
/// assert_eq!(value.len(), 4);
/// assert!(!value.is_empty());
///
/// // Access bytes
/// assert_eq!(value.as_bytes(), &[1, 2, 3, 4]);
///
/// // Convert back to Vec<u8>
/// let bytes = value.into_bytes();
/// assert_eq!(bytes, vec![1, 2, 3, 4]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageValue(Vec<u8>);

impl StorageValue {
    /// Creates a new `StorageValue` from raw bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw bytes to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::storage::value::StorageValue;
    ///
    /// let value = StorageValue::new(vec![1, 2, 3]);
    /// assert_eq!(value.len(), 3);
    /// ```
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    /// Returns the value as a byte slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::storage::value::StorageValue;
    ///
    /// let value = StorageValue::new(vec![1, 2, 3]);
    /// assert_eq!(value.as_bytes(), &[1, 2, 3]);
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Consumes the value and returns the underlying bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::storage::value::StorageValue;
    ///
    /// let value = StorageValue::new(vec![1, 2, 3]);
    /// let bytes = value.into_bytes();
    /// assert_eq!(bytes, vec![1, 2, 3]);
    /// ```
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }

    /// Returns the length of the value in bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::storage::value::StorageValue;
    ///
    /// let value = StorageValue::new(vec![1, 2, 3, 4, 5]);
    /// assert_eq!(value.len(), 5);
    /// ```
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the value is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::storage::value::StorageValue;
    ///
    /// let empty = StorageValue::new(vec![]);
    /// assert!(empty.is_empty());
    ///
    /// let non_empty = StorageValue::new(vec![1]);
    /// assert!(!non_empty.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Vec<u8>> for StorageValue {
    fn from(data: Vec<u8>) -> Self {
        Self::new(data)
    }
}

impl From<&[u8]> for StorageValue {
    fn from(data: &[u8]) -> Self {
        Self::new(data.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_value_new() {
        let value = StorageValue::new(vec![1, 2, 3, 4, 5]);
        assert_eq!(value.len(), 5);
        assert!(!value.is_empty());
    }

    #[test]
    fn test_storage_value_as_bytes() {
        let value = StorageValue::new(vec![10, 20, 30]);
        assert_eq!(value.as_bytes(), &[10, 20, 30]);
    }

    #[test]
    fn test_storage_value_into_bytes() {
        let value = StorageValue::new(vec![1, 2, 3]);
        let bytes = value.into_bytes();
        assert_eq!(bytes, vec![1, 2, 3]);
    }

    #[test]
    fn test_storage_value_from_vec() {
        let value: StorageValue = vec![1, 2, 3].into();
        assert_eq!(value.len(), 3);
    }

    #[test]
    fn test_storage_value_from_slice() {
        let data: &[u8] = &[1, 2, 3, 4];
        let value: StorageValue = data.into();
        assert_eq!(value.len(), 4);
    }

    #[test]
    fn test_storage_value_empty() {
        let value = StorageValue::new(vec![]);
        assert!(value.is_empty());
        assert_eq!(value.len(), 0);
    }

    #[test]
    fn test_storage_value_equality() {
        let v1 = StorageValue::new(vec![1, 2, 3]);
        let v2 = StorageValue::new(vec![1, 2, 3]);
        let v3 = StorageValue::new(vec![3, 2, 1]);

        assert_eq!(v1, v2);
        assert_ne!(v1, v3);
    }

    #[test]
    fn test_storage_value_clone() {
        let v1 = StorageValue::new(vec![1, 2, 3]);
        let v2 = v1.clone();
        assert_eq!(v1, v2);
    }

    #[test]
    fn test_storage_value_debug() {
        let value = StorageValue::new(vec![1, 2, 3]);
        let debug_str = format!("{:?}", value);
        assert!(debug_str.contains("StorageValue"));
    }
}
