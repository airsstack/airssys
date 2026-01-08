//! # Component Module
//!
//! Foundation types and trait abstractions for component-related functionality.
//!
//! This module contains data structures and trait definitions that are used
//! by all other modules (security/, runtime/, component/, messaging/, system/).
//!
//! # Architecture
//!
//! This module is part of the **core/** foundation (Layer 1). It contains
//! ONLY:
//!
//! - Data structures (ComponentId, ComponentHandle, ComponentMessage, MessageMetadata)
//! - Trait definitions (ComponentLifecycle)
//! - NO business logic
//! - NO external dependencies (only std)
//!
//! # Purpose
//!
//! The component submodule provides the foundational types that enable:
//!
//! - Component identification (ComponentId)
//! - Component instance management (ComponentHandle)
//! - Inter-component communication (ComponentMessage, MessageMetadata)
//! - Lifecycle management (ComponentLifecycle trait)
//!
//! # Usage
//!
//! These types are imported and used by:
//!
//! - **security/**: Uses ComponentId for capability scoping
//! - **runtime/**: Uses ComponentHandle for WASM execution
//! - **component/**: Uses ComponentLifecycle for actor wrapping
//! - **messaging/**: Uses ComponentMessage for communication
//! - **system/**: Coordinates all components using these types
//!
//! # Examples
//!
//! ```rust
//! use airssys_wasm::core::component::{
//!     ComponentId, ComponentHandle, ComponentMessage, ComponentLifecycle,
//! };
//!
//! // Create component identifier
//! let id = ComponentId::new("system", "database", "prod");
//!
//! // Create component handle
//! let handle = ComponentHandle::new(id, 12345);
//!
//! // Create message
//! let message = ComponentMessage::new(
//!     ComponentId::new("system", "cache", "dev"),
//!     vec![1, 2, 3],
//!     MessageMetadata::default(),
//! );
//! ```

// Module declarations (per PROJECTS_STANDARD.md §4.3)
pub mod handle;
pub mod id;
pub mod message;
pub mod traits;

// Re-exports for ergonomic API (per PROJECTS_STANDARD.md §4.3)
pub use handle::ComponentHandle;
pub use id::ComponentId;
pub use message::{ComponentMessage, MessageMetadata};
pub use traits::ComponentLifecycle;
