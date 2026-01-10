//! Storage abstractions for component-isolated key-value storage.
//!
//! This module provides types, traits, and errors for component storage.
//! Each component has access to isolated storage namespaced by its ID.
//!
//! # Architecture
//!
//! This module is part of the **core/** foundation (Layer 1). It contains:
//!
//! - **Types**: `StorageValue` (dedicated ADT for storage values)
//! - **Traits**: `ComponentStorage` (abstraction for key-value storage)
//! - **Errors**: `StorageError` (co-located)
//!
//! Concrete implementations are provided via host functions.
//!
//! # Design Decision
//!
//! `StorageValue` is a dedicated type (not `MessagePayload`) for domain
//! boundary clarity. Engineers immediately know the type's purpose.
//!
//! # Submodules
//!
//! - [`value`] - `StorageValue` ADT (dedicated storage value type)
//! - [`errors`] - `StorageError` enum (co-located with storage)
//! - [`traits`] - `ComponentStorage` trait
//!
//! # Usage
//!
//! ```rust
//! use airssys_wasm::core::storage::value::StorageValue;
//! use airssys_wasm::core::storage::errors::StorageError;
//! use airssys_wasm::core::storage::traits::ComponentStorage;
//!
//! // Create a storage value
//! let value = StorageValue::new(vec![1, 2, 3]);
//!
//! // Create an error
//! let error = StorageError::NotFound("user:123".to_string());
//! ```

// Module declarations (per PROJECTS_STANDARD.md §4.3)
pub mod errors;
pub mod traits;
pub mod value;

// NOTE: No glob re-exports per module grouping policy.
// Callers use namespaced access: core::storage::value::StorageValue
