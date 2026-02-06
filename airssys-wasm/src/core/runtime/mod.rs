//! # Runtime Module
//!
//! WASM runtime engine abstractions.
//!
//! This module contains trait abstractions that define how WASM components
//! are loaded, executed, and managed. The actual implementations
//! are provided by the **runtime/** module (Layer 3B).
//!
//! # Architecture
//!
//! This module is part of **core/** foundation (Layer 1). It contains
//! ONLY:
//!
//! - Trait definitions (RuntimeEngine, ComponentLoader)
//! - Resource constraint types (ResourceLimits)
//! - NO business logic
//! - NO external dependencies (only std and core/component/)
//!
//! # Purpose
//!
//! The runtime submodule provides foundational abstractions that enable:
//!
//! - WASM component loading (ComponentLoader trait)
//! - WASM component execution (RuntimeEngine trait)
//! - Resource limits enforcement (ResourceLimits struct)
//!
//! # Usage
//!
//! These types are imported and used by:
//!
//! - **runtime/**: Implements RuntimeEngine and ComponentLoader traits
//! - **component/**: Uses RuntimeEngine trait for WASM execution
//! - **system/**: Injects concrete runtime implementations into components
//!
//! # Examples
//!
//! ## Creating resource limits
//!
//! ```rust
//! use airssys_wasm::core::runtime::limits::ResourceLimits;
//!
//! let limits = ResourceLimits {
//!     max_memory_bytes: 128 * 1024 * 1024,
//!     max_execution_time_ms: 60_000,
//!     max_fuel: Some(1_000_000),
//! };
//! ```
//!
//! ## Using RuntimeEngine trait (implemented by runtime/ module)
//!
//! ```rust
//! use airssys_wasm::core::runtime::traits::RuntimeEngine;
//! use airssys_wasm::core::runtime::errors::WasmError;
//! use airssys_wasm::core::component::id::ComponentId;
//! use airssys_wasm::core::component::handle::ComponentHandle;
//! use airssys_wasm::core::component::message::{ComponentMessage, MessagePayload, MessageMetadata};
//!
//! fn load_component<E: RuntimeEngine>(
//!     engine: &E,
//!     id: &ComponentId,
//!     bytes: &[u8],
//! ) -> Result<ComponentHandle, WasmError> {
//!     engine.load_component(id, bytes)
//! }
//! ```
//!
//! ## Dependency Inversion Pattern
//!
//! ```rust
//! // component/ module (Layer 3A) depends on ABSTRACTION via generics (S6.2)
//! use airssys_wasm::core::runtime::traits::RuntimeEngine;
//! use airssys_wasm::core::runtime::errors::WasmError;
//!
//! pub struct ComponentWrapper<E: RuntimeEngine> {
//!     engine: std::sync::Arc<E>,
//! }
//!
//! impl<E: RuntimeEngine> ComponentWrapper<E> {
//!     # pub fn new(engine: std::sync::Arc<E>) -> Self {
//!     #     Self { engine }
//!     # }
//! }
//!
//! // runtime/ module (Layer 3B) implements ABSTRACTION
//! use airssys_wasm::core::component::id::ComponentId;
//! use airssys_wasm::core::component::handle::ComponentHandle;
//! use airssys_wasm::core::component::message::{ComponentMessage, MessagePayload};
//!
//! pub struct WasmtimeEngine { /* ... */ }
//!
//! impl WasmtimeEngine {
//!     # pub fn new() -> Self {
//!     #     Self { }
//!     # }
//! }
//!
//! impl RuntimeEngine for WasmtimeEngine {
//!     fn load_component(&self, id: &ComponentId, _bytes: &[u8])
//!         -> Result<ComponentHandle, WasmError>
//!     {
//!         // Real WASM implementation
//!         # Ok(ComponentHandle::new(id.clone(), 1))
//!     }
//!
//!     fn unload_component(&self, _handle: &ComponentHandle)
//!         -> Result<(), WasmError>
//!     {
//!         // Real WASM implementation
//!         # Ok(())
//!     }
//!
//!     fn call_handle_message(
//!         &self,
//!         _handle: &ComponentHandle,
//!         _msg: &ComponentMessage,
//!     ) -> Result<Option<MessagePayload>, WasmError>
//!     {
//!         // Real WASM implementation
//!         # Ok(None)
//!     }
//!
//!     fn call_handle_callback(
//!         &self,
//!         _handle: &ComponentHandle,
//!         _msg: &ComponentMessage,
//!     ) -> Result<(), WasmError>
//!     {
//!         // Real WASM implementation
//!         # Ok(())
//!     }
//! }
//!
//! // system/ module (Layer 4) injects concrete type
//! let engine = std::sync::Arc::new(WasmtimeEngine::new());
//! let wrapper = ComponentWrapper::new(engine);
//! // wrapper is ComponentWrapper<WasmtimeEngine> - static dispatch
//! ```

// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)
// None needed for mod.rs

// Layer 2: Third-party crate imports (per PROJECTS_STANDARD.md §2.1)
// None needed for mod.rs

// Layer 3: Internal module imports (per PROJECTS_STANDARD.md §2.1)
// None needed for mod.rs (imports only in submodules)

// Module declarations (per PROJECTS_STANDARD.md §4.3)
pub mod errors;
pub mod limits;
pub mod traits;

// NOTE: No type re-exports per PROJECTS_STANDARD.md §4.3.
// Callers use namespaced access:
