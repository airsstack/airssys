//! # Runtime Traits
//!
//! Trait definitions for WASM runtime engine and component loader.
//!
//! This module contains trait abstractions that define how WASM components
//! are loaded and executed. Concrete implementations are provided by the
//! runtime/ module (Layer 3B).
//!
//! # Traits
//!
//! - [`RuntimeEngine`] - WASM component execution trait
//! - [`ComponentLoader`] - Component binary loading trait
//!
//! # Usage
//!
//! These traits are implemented by concrete runtime engines (e.g., WasmtimeEngine)
//! and used by higher-level components to execute WASM code.

// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)
// (none - WasmError handles errors)

// Layer 2: Third-party crate imports (per PROJECTS_STANDARD.md §2.1)
// None needed for this module

// Layer 3: Internal module imports (per PROJECTS_STANDARD.md §2.1)
use super::errors::WasmError;
use crate::core::component::handle::ComponentHandle;
use crate::core::component::id::ComponentId;
use crate::core::component::message::{ComponentMessage, MessagePayload};

/// Trait for WASM runtime engine abstraction.
///
/// `RuntimeEngine` defines the interface for managing WASM component lifecycle
/// and execution. This trait is implemented by the runtime/ module (Layer 3B)
/// and used by component/ module (Layer 3A) through dependency injection.
///
/// # Architecture Note
///
/// `RuntimeEngine` is defined in `core/runtime/` (Layer 1) as an abstraction.
/// The concrete implementation (e.g., `WasmtimeEngine`) lives in `runtime/`
/// module (Layer 3B). This follows the Dependency Inversion Principle:
/// component/ depends on the trait (abstraction), not the concrete implementation.
///
/// # Generic Bounds
///
/// All methods take references (`&self`) to allow shared usage across
/// multiple components. The trait requires `Send + Sync` for thread safety
/// in concurrent contexts.
///
/// # Examples
///
/// ## Using RuntimeEngine trait
///
/// ```rust
/// use airssys_wasm::core::runtime::traits::RuntimeEngine;
/// use airssys_wasm::core::runtime::errors::WasmError;
/// use airssys_wasm::core::component::id::ComponentId;
/// use airssys_wasm::core::component::handle::ComponentHandle;
/// use airssys_wasm::core::component::message::{ComponentMessage, MessagePayload};
///
/// struct MyEngine;
///
/// impl RuntimeEngine for MyEngine {
///     fn load_component(&self, id: &ComponentId, _bytes: &[u8])
///         -> Result<ComponentHandle, WasmError>
///     {
///         // Implementation would load WASM bytes and return handle
///         Ok(ComponentHandle::new(id.clone(), 1))
///     }
///
///     fn unload_component(&self, _handle: &ComponentHandle)
///         -> Result<(), WasmError>
///     {
///         // Implementation would release component resources
///         Ok(())
///     }
///
///     fn call_handle_message(
///         &self,
///         _handle: &ComponentHandle,
///         _msg: &ComponentMessage,
///     ) -> Result<Option<MessagePayload>, WasmError>
///     {
///         // Implementation would call handle-message export
///         Ok(None)
///     }
///
///     fn call_handle_callback(
///         &self,
///         _handle: &ComponentHandle,
///         _msg: &ComponentMessage,
///     ) -> Result<(), WasmError>
///     {
///         // Implementation would call handle-callback export
///         Ok(())
///     }
/// }
/// ```
pub trait RuntimeEngine: Send + Sync {
    /// Load a WASM component from bytes.
    ///
    /// This method validates and loads WASM binary bytes into the runtime engine,
    /// creating a new component instance. The returned `ComponentHandle` can be used
    /// to execute the component via other trait methods.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique component identifier
    /// * `bytes` - Raw WASM binary bytes
    ///
    /// # Returns
    ///
    /// Returns `ComponentHandle` on success, or `WasmError` if loading fails.
    ///
    /// # Errors
    ///
    /// - `WasmError::InvalidComponent` - WASM bytes are invalid
    /// - `WasmError::InstantiationFailed` - Component instantiation failed
    /// - `WasmError::ResourceLimitExceeded` - Exceeded memory/fuel limits
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airssys_wasm::core::runtime::traits::RuntimeEngine;
    /// # use airssys_wasm::core::runtime::errors::WasmError;
    /// # use airssys_wasm::core::component::id::ComponentId;
    /// # use airssys_wasm::core::component::handle::ComponentHandle;
    /// # use airssys_wasm::core::component::message::ComponentMessage;
    /// # use airssys_wasm::core::component::message::MessagePayload;
    /// # struct MockEngine;
    /// # impl RuntimeEngine for MockEngine {
    /// #     fn load_component(&self, id: &ComponentId, _bytes: &[u8])
    /// #         -> Result<ComponentHandle, WasmError> {
    /// #         Ok(ComponentHandle::new(id.clone(), 1))
    /// #     }
    /// #     fn unload_component(&self, _handle: &ComponentHandle)
    /// #         -> Result<(), WasmError> { Ok(()) }
    /// #     fn call_handle_message(&self, _handle: &ComponentHandle, _msg: &ComponentMessage)
    /// #         -> Result<Option<MessagePayload>, WasmError> { Ok(None) }
    /// #     fn call_handle_callback(&self, _handle: &ComponentHandle, _msg: &ComponentMessage)
    /// #         -> Result<(), WasmError> { Ok(()) }
    /// # }
    ///
    /// let engine = MockEngine;
    /// let id = ComponentId::new("system", "database", "prod");
    /// let wasm_bytes = vec![0x00, 0x61, 0x73, 0x6d]; // WASM magic
    ///
    /// let handle = engine.load_component(&id, &wasm_bytes)?;
    /// # Ok::<(), WasmError>(())
    /// ```
    fn load_component(&self, id: &ComponentId, bytes: &[u8]) -> Result<ComponentHandle, WasmError>;

    /// Unload a component and release resources.
    ///
    /// This method releases all resources associated with a loaded component,
    /// including memory and runtime structures. After unloading, the `ComponentHandle`
    /// becomes invalid and should not be used.
    ///
    /// # Arguments
    ///
    /// * `handle` - Component handle to unload
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or `WasmError` if unloading fails.
    ///
    /// # Errors
    ///
    /// - `WasmError::ComponentNotFound` - Component handle is invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airssys_wasm::core::runtime::traits::RuntimeEngine;
    /// # use airssys_wasm::core::runtime::errors::WasmError;
    /// # use airssys_wasm::core::component::id::ComponentId;
    /// # use airssys_wasm::core::component::handle::ComponentHandle;
    /// # use airssys_wasm::core::component::message::ComponentMessage;
    /// # use airssys_wasm::core::component::message::MessagePayload;
    /// # struct MockEngine;
    /// # impl RuntimeEngine for MockEngine {
    /// #     fn load_component(&self, id: &ComponentId, _bytes: &[u8])
    /// #         -> Result<ComponentHandle, WasmError> {
    /// #         Ok(ComponentHandle::new(id.clone(), 1))
    /// #     }
    /// #     fn unload_component(&self, _handle: &ComponentHandle)
    /// #         -> Result<(), WasmError> { Ok(()) }
    /// #     fn call_handle_message(&self, _handle: &ComponentHandle, _msg: &ComponentMessage)
    /// #         -> Result<Option<MessagePayload>, WasmError> { Ok(None) }
    /// #     fn call_handle_callback(&self, _handle: &ComponentHandle, _msg: &ComponentMessage)
    /// #         -> Result<(), WasmError> { Ok(()) }
    /// # }
    ///
    /// let engine = MockEngine;
    /// let handle = ComponentHandle::new(ComponentId::new("test", "comp", "1"), 42);
    ///
    /// engine.unload_component(&handle)?;
    /// # Ok::<(), WasmError>(())
    /// ```
    fn unload_component(&self, handle: &ComponentHandle) -> Result<(), WasmError>;

    /// Call handle-message export on a component.
    ///
    /// This method invokes the `handle-message` export function on a loaded WASM
    /// component, passing the component message as input. The component may return
    /// an optional response payload.
    ///
    /// # Arguments
    ///
    /// * `handle` - Component handle to call
    /// * `msg` - Component message to pass to handle-message export
    ///
    /// # Returns
    ///
    /// Returns `Some(payload)` if component responds, or `None` if component
    /// does not produce a response (fire-and-forget pattern).
    ///
    /// # Errors
    ///
    /// - `WasmError::ComponentNotFound` - Component handle is invalid
    /// - `WasmError::ExportNotFound` - handle-message export not found
    /// - `WasmError::Timeout` - Execution exceeded time limit
    /// - `WasmError::RuntimeError` - WASM execution error occurred
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airssys_wasm::core::runtime::traits::RuntimeEngine;
    /// # use airssys_wasm::core::runtime::errors::WasmError;
    /// # use airssys_wasm::core::component::id::ComponentId;
    /// # use airssys_wasm::core::component::handle::ComponentHandle;
    /// # use airssys_wasm::core::component::message::{ComponentMessage, MessagePayload, MessageMetadata};
    /// # struct MockEngine;
    /// # impl RuntimeEngine for MockEngine {
    /// #     fn load_component(&self, id: &ComponentId, _bytes: &[u8])
    /// #         -> Result<ComponentHandle, WasmError> {
    /// #         Ok(ComponentHandle::new(id.clone(), 1))
    /// #     }
    /// #     fn unload_component(&self, _handle: &ComponentHandle)
    /// #         -> Result<(), WasmError> { Ok(()) }
    /// #     fn call_handle_message(&self, _handle: &ComponentHandle, _msg: &ComponentMessage)
    /// #         -> Result<Option<MessagePayload>, WasmError> { Ok(None) }
    /// #     fn call_handle_callback(&self, _handle: &ComponentHandle, _msg: &ComponentMessage)
    /// #         -> Result<(), WasmError> { Ok(()) }
    /// # }
    ///
    /// let engine = MockEngine;
    /// let handle = ComponentHandle::new(ComponentId::new("test", "comp", "1"), 42);
    /// let sender = ComponentId::new("sender", "comp", "1");
    /// let payload = vec![1, 2, 3];
    /// let metadata = MessageMetadata::default();
    /// let message = ComponentMessage::new(sender, payload, metadata);
    ///
    /// let response = engine.call_handle_message(&handle, &message)?;
    /// // Response may be Some(payload) or None for fire-and-forget
    /// # Ok::<(), WasmError>(())
    /// ```
    fn call_handle_message(
        &self,
        handle: &ComponentHandle,
        msg: &ComponentMessage,
    ) -> Result<Option<MessagePayload>, WasmError>;

    /// Call handle-callback export on a component.
    ///
    /// This method invokes the `handle-callback` export function on a loaded
    /// WASM component, used for delivering responses to request-response
    /// patterns. The callback does not return a value.
    ///
    /// # Arguments
    ///
    /// * `handle` - Component handle to call
    /// * `msg` - Component message (response) to pass to handle-callback export
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or `WasmError` if callback fails.
    ///
    /// # Errors
    ///
    /// - `WasmError::ComponentNotFound` - Component handle is invalid
    /// - `WasmError::ExportNotFound` - handle-callback export not found
    /// - `WasmError::Timeout` - Execution exceeded time limit
    /// - `WasmError::RuntimeError` - WASM execution error occurred
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airssys_wasm::core::runtime::traits::RuntimeEngine;
    /// # use airssys_wasm::core::runtime::errors::WasmError;
    /// # use airssys_wasm::core::component::id::ComponentId;
    /// # use airssys_wasm::core::component::handle::ComponentHandle;
    /// # use airssys_wasm::core::component::message::{ComponentMessage, MessagePayload, MessageMetadata};
    /// # struct MockEngine;
    /// # impl RuntimeEngine for MockEngine {
    /// #     fn load_component(&self, id: &ComponentId, _bytes: &[u8])
    /// #         -> Result<ComponentHandle, WasmError> {
    /// #         Ok(ComponentHandle::new(id.clone(), 1))
    /// #     }
    /// #     fn unload_component(&self, _handle: &ComponentHandle)
    /// #         -> Result<(), WasmError> { Ok(()) }
    /// #     fn call_handle_message(&self, _handle: &ComponentHandle, _msg: &ComponentMessage)
    /// #         -> Result<Option<MessagePayload>, WasmError> { Ok(None) }
    /// #     fn call_handle_callback(&self, _handle: &ComponentHandle, _msg: &ComponentMessage)
    /// #         -> Result<(), WasmError> { Ok(()) }
    /// # }
    ///
    /// let engine = MockEngine;
    /// let handle = ComponentHandle::new(ComponentId::new("test", "comp", "1"), 42);
    /// let sender = ComponentId::new("sender", "comp", "1");
    /// let response_payload = vec![4, 5, 6];
    /// let metadata = MessageMetadata::default();
    /// let callback = ComponentMessage::new(sender, response_payload, metadata);
    ///
    /// engine.call_handle_callback(&handle, &callback)?;
    /// # Ok::<(), WasmError>(())
    /// ```
    fn call_handle_callback(
        &self,
        handle: &ComponentHandle,
        msg: &ComponentMessage,
    ) -> Result<(), WasmError>;
}

/// Trait for loading component binaries.
///
/// `ComponentLoader` defines the interface for loading WASM component binaries
/// from storage or external sources. This trait is implemented by the runtime/
/// module (Layer 3B) and used by system/ module (Layer 4) for component
/// loading workflows.
///
/// # Architecture Note
///
/// `ComponentLoader` is defined in `core/runtime/` (Layer 1) as an abstraction.
/// The concrete implementation (e.g., `WasmtimeComponentLoader`) lives in
/// `runtime/` module (Layer 3B). This follows the Dependency Inversion
/// Principle: system/ depends on the trait (abstraction), not the concrete
/// implementation.
///
/// # Generic Bounds
///
/// All methods take references (`&self`) to allow shared usage. The trait
/// requires `Send + Sync` for thread safety in concurrent contexts.
///
/// # Examples
///
/// ## Using ComponentLoader trait
///
/// ```rust
/// use airssys_wasm::core::runtime::traits::ComponentLoader;
/// use airssys_wasm::core::runtime::errors::WasmError;
/// use airssys_wasm::core::component::id::ComponentId;
///
/// struct MyLoader;
///
/// impl ComponentLoader for MyLoader {
///     fn load_bytes(&self, _id: &ComponentId) -> Result<Vec<u8>, WasmError> {
///         // Implementation would load WASM bytes from storage
///         Ok(vec![1, 2, 3, 4])
///     }
///
///     fn validate(&self, _bytes: &[u8]) -> Result<(), WasmError> {
///         // Implementation would validate WASM binary structure
///         Ok(())
///     }
/// }
/// ```
pub trait ComponentLoader: Send + Sync {
    /// Load component bytes from storage or external source.
    ///
    /// This method retrieves raw WASM binary bytes for a component identified
    /// by `id`. The bytes can be loaded from filesystem, network, or other
    /// storage mechanisms depending on implementation.
    ///
    /// # Arguments
    ///
    /// * `id` - Component identifier
    ///
    /// # Returns
    ///
    /// Returns raw WASM binary bytes on success, or `WasmError` if loading fails.
    ///
    /// # Errors
    ///
    /// - `WasmError::ComponentNotFound` - Component not found in storage
    /// - `WasmError::RuntimeError` - I/O error or network error
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airssys_wasm::core::runtime::traits::ComponentLoader;
    /// # use airssys_wasm::core::runtime::errors::WasmError;
    /// # use airssys_wasm::core::component::id::ComponentId;
    /// # struct MyLoader;
    /// # impl ComponentLoader for MyLoader {
    /// #     fn load_bytes(&self, _id: &ComponentId) -> Result<Vec<u8>, WasmError> {
    /// #         Ok(vec![1, 2, 3, 4])
    /// #     }
    /// #     fn validate(&self, _bytes: &[u8]) -> Result<(), WasmError> { Ok(()) }
    /// # }
    ///
    /// let loader = MyLoader;
    /// let id = ComponentId::new("system", "database", "prod");
    ///
    /// let wasm_bytes = loader.load_bytes(&id)?;
    /// assert!(wasm_bytes.len() > 0);
    /// # Ok::<(), WasmError>(())
    /// ```
    fn load_bytes(&self, id: &ComponentId) -> Result<Vec<u8>, WasmError>;

    /// Validate component bytes before loading.
    ///
    /// This method performs structural validation on WASM binary bytes to ensure
    /// they are valid before attempting to load them. Validation checks for
    /// correct magic number, version, and structural integrity.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Raw WASM binary bytes to validate
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if bytes are valid, or `WasmError` if validation fails.
    ///
    /// # Errors
    ///
    /// - `WasmError::InvalidComponent` - WASM bytes are invalid or malformed
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airssys_wasm::core::runtime::traits::ComponentLoader;
    /// # use airssys_wasm::core::runtime::errors::WasmError;
    /// # use airssys_wasm::core::component::id::ComponentId;
    /// # struct MyLoader;
    /// # impl ComponentLoader for MyLoader {
    /// #     fn load_bytes(&self, _id: &ComponentId) -> Result<Vec<u8>, WasmError> {
    /// #         Ok(vec![1, 2, 3, 4])
    /// #     }
    /// #     fn validate(&self, _bytes: &[u8]) -> Result<(), WasmError> { Ok(()) }
    /// # }
    ///
    /// let loader = MyLoader;
    /// let wasm_bytes = vec![0x00, 0x61, 0x73, 0x6d]; // WASM magic
    ///
    /// loader.validate(&wasm_bytes)?;
    /// # Ok::<(), WasmError>(())
    /// ```
    fn validate(&self, bytes: &[u8]) -> Result<(), WasmError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementation for testing RuntimeEngine trait
    struct MockRuntimeEngine;

    impl RuntimeEngine for MockRuntimeEngine {
        fn load_component(
            &self,
            id: &ComponentId,
            _bytes: &[u8],
        ) -> Result<ComponentHandle, WasmError> {
            Ok(ComponentHandle::new(id.clone(), 1))
        }

        fn unload_component(&self, _handle: &ComponentHandle) -> Result<(), WasmError> {
            Ok(())
        }

        fn call_handle_message(
            &self,
            _handle: &ComponentHandle,
            _msg: &ComponentMessage,
        ) -> Result<Option<MessagePayload>, WasmError> {
            Ok(None)
        }

        fn call_handle_callback(
            &self,
            _handle: &ComponentHandle,
            _msg: &ComponentMessage,
        ) -> Result<(), WasmError> {
            Ok(())
        }
    }

    // Mock implementation for testing ComponentLoader trait
    struct MockComponentLoader;

    impl ComponentLoader for MockComponentLoader {
        fn load_bytes(&self, _id: &ComponentId) -> Result<Vec<u8>, WasmError> {
            Ok(vec![1, 2, 3, 4])
        }

        fn validate(&self, _bytes: &[u8]) -> Result<(), WasmError> {
            Ok(())
        }
    }

    #[test]
    fn test_runtime_engine_load_component_returns_handle() {
        let engine = MockRuntimeEngine;
        let id = ComponentId::new("system", "test", "1");
        let bytes = vec![0x00, 0x01, 0x02];

        let result = engine.load_component(&id, &bytes);
        assert!(result.is_ok());

        let handle = result.unwrap();
        assert_eq!(handle.id().to_string_id(), "system/test/1");
    }

    #[test]
    fn test_runtime_engine_unload_component_succeeds() {
        let engine = MockRuntimeEngine;
        let id = ComponentId::new("system", "test", "1");
        let handle = ComponentHandle::new(id.clone(), 1);

        let result = engine.unload_component(&handle);
        assert!(result.is_ok());
    }

    #[test]
    fn test_runtime_engine_call_handle_message_returns_optional_payload() {
        let engine = MockRuntimeEngine;
        let id = ComponentId::new("system", "test", "1");
        let handle = ComponentHandle::new(id, 1);

        let sender_id = ComponentId::new("system", "sender", "1");
        let payload = vec![1, 2, 3];
        let metadata = Default::default();
        let msg = ComponentMessage::new(sender_id, payload, metadata);

        let result = engine.call_handle_message(&handle, &msg);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_runtime_engine_call_handle_callback_succeeds() {
        let engine = MockRuntimeEngine;
        let id = ComponentId::new("system", "test", "1");
        let handle = ComponentHandle::new(id, 1);

        let sender_id = ComponentId::new("system", "sender", "1");
        let payload = vec![4, 5, 6];
        let metadata = Default::default();
        let msg = ComponentMessage::new(sender_id, payload, metadata);

        let result = engine.call_handle_callback(&handle, &msg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_component_loader_load_bytes_returns_data() {
        let loader = MockComponentLoader;
        let id = ComponentId::new("system", "test", "1");

        let result = loader.load_bytes(&id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_component_loader_validate_succeeds() {
        let loader = MockComponentLoader;
        let bytes = vec![0x00, 0x61, 0x73, 0x6d]; // WASM magic

        let result = loader.validate(&bytes);
        assert!(result.is_ok());
    }

    #[test]
    fn test_runtime_engine_send_sync_bounds() {
        // Verify RuntimeEngine can be used in async contexts
        fn requires_send<T: Send>(_val: T) {}
        fn requires_sync<T: Sync>(_val: T) {}

        let engine = MockRuntimeEngine;
        requires_send(&engine);
        requires_sync(&engine);
    }

    #[test]
    fn test_component_loader_send_sync_bounds() {
        // Verify ComponentLoader can be used in async contexts
        fn requires_send<T: Send>(_val: T) {}
        fn requires_sync<T: Sync>(_val: T) {}

        let loader = MockComponentLoader;
        requires_send(&loader);
        requires_sync(&loader);
    }

    #[test]
    fn test_wasm_error_display() {
        let error = WasmError::ComponentNotFound("test-comp".to_string());
        let display_str = format!("{}", error);
        assert!(display_str.contains("Component not found"));
        assert!(display_str.contains("test-comp"));
    }

    #[test]
    fn test_wasm_error_timeout_display() {
        let error = WasmError::Timeout;
        let display_str = format!("{}", error);
        assert!(display_str.contains("Execution timeout"));
    }

    #[test]
    fn test_wasm_error_resource_limit_display() {
        let error = WasmError::ResourceLimitExceeded("memory".to_string());
        let display_str = format!("{}", error);
        assert!(display_str.contains("Resource limit exceeded"));
        assert!(display_str.contains("memory"));
    }

    #[test]
    fn test_wasm_error_debug() {
        let error = WasmError::RuntimeError("test error".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("RuntimeError"));
    }

    #[test]
    fn test_wasm_error_clone() {
        let error1 = WasmError::InstantiationFailed("test".to_string());
        let error2 = error1.clone();
        assert_eq!(error1, error2);
    }

    #[test]
    fn test_runtime_engine_with_empty_message() {
        let engine = MockRuntimeEngine;
        let id = ComponentId::new("system", "test", "1");
        let handle = ComponentHandle::new(id, 1);

        let sender_id = ComponentId::new("system", "sender", "1");
        let payload = vec![];
        let metadata = Default::default();
        let msg = ComponentMessage::new(sender_id, payload, metadata);

        let result = engine.call_handle_message(&handle, &msg);
        assert!(result.is_ok());
    }

    #[test]
    fn test_component_loader_validate_with_empty_bytes() {
        let loader = MockComponentLoader;
        let bytes: Vec<u8> = vec![];

        let result = loader.validate(&bytes);
        // Mock implementation accepts empty bytes
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_load_unload_cycles() {
        let engine = MockRuntimeEngine;
        let id1 = ComponentId::new("system", "comp1", "1");
        let id2 = ComponentId::new("system", "comp2", "1");
        let bytes = vec![0x00, 0x01, 0x02];

        // Load first component
        let handle1 = engine.load_component(&id1, &bytes).unwrap();
        assert_eq!(handle1.id().to_string_id(), "system/comp1/1");

        // Load second component
        let handle2 = engine.load_component(&id2, &bytes).unwrap();
        assert_eq!(handle2.id().to_string_id(), "system/comp2/1");

        // Unload both components
        assert!(engine.unload_component(&handle1).is_ok());
        assert!(engine.unload_component(&handle2).is_ok());
    }

    #[test]
    fn test_runtime_engine_multiple_messages() {
        let engine = MockRuntimeEngine;
        let id = ComponentId::new("system", "test", "1");
        let handle = ComponentHandle::new(id, 1);

        let sender_id = ComponentId::new("system", "sender", "1");

        // Send multiple messages
        for i in 1..=5 {
            let payload = vec![i];
            let metadata = Default::default();
            let msg = ComponentMessage::new(sender_id.clone(), payload, metadata);

            let result = engine.call_handle_message(&handle, &msg);
            assert!(result.is_ok());
        }
    }
}
