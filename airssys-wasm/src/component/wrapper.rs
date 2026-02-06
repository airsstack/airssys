//! # ComponentWrapper - WASM Component Actor Integration
//!
//! This module provides `ComponentWrapper` that wraps a WASM component instance
//! as an airssys-rt Actor. Each WASM component = one Actor.
//!
//! # Architecture
//!
//! ComponentWrapper is in Layer 3A (component/ module) and:
//! - Depends on `core/` types (ComponentId, ComponentHandle, ComponentMessage)
//! - Depends on `core/runtime/traits.rs` (RuntimeEngine abstraction)
//! - Integrates with `airssys-rt` (Actor trait, Message trait)
//! - Receives concrete RuntimeEngine injection from `system/` (Layer 4)
//!
//! # Key Design Principle
//!
//! Uses generic `E: RuntimeEngine` for dependency injection with static dispatch
//! (per PROJECTS_STANDARD.md S6.2). The concrete implementation (e.g., WasmtimeEngine)
//! is provided by the system/ module as a type parameter.
//!
//! # References
//!
//! - ADR-WASM-031: Component & Messaging Module Design
//! - KNOWLEDGE-WASM-038: Component Module Responsibility

// Layer 1: Standard library imports
use std::fmt;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use airssys_rt::broker::MessageBroker;
use airssys_rt::message::MessagePriority;
use airssys_rt::{Actor, ActorContext, ErrorAction, Message};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Layer 3: Internal module imports
use crate::core::component::handle::ComponentHandle;
use crate::core::component::id::ComponentId;
use crate::core::component::message::ComponentMessage;
use crate::core::runtime::errors::WasmError;
use crate::core::runtime::traits::RuntimeEngine;

/// Message type for ComponentWrapper actor.
///
/// Defines the messages that a ComponentWrapper can receive from the
/// actor system. Each variant corresponds to a specific operation.
///
/// # Variants
///
/// - `HandleMessage` - Invoke the WASM component's handle-message export
/// - `HandleCallback` - Deliver a response via handle-callback export
/// - `Shutdown` - Gracefully stop the component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentActorMessage {
    /// Forward a message to the WASM component's handle-message export.
    HandleMessage(ComponentMessage),

    /// Deliver a callback/response to the WASM component's handle-callback export.
    HandleCallback(ComponentMessage),

    /// Request graceful shutdown of the component.
    Shutdown,
}

// Implement airssys-rt Message trait
// ComponentActorMessage already derives Clone + Debug
// Enum is Send + Sync if all variants are Send + Sync
// ComponentMessage must be Send + Sync + Clone for this to work
impl Message for ComponentActorMessage {
    const MESSAGE_TYPE: &'static str = "component_actor_message";

    fn priority(&self) -> MessagePriority {
        MessagePriority::Normal
    }
}

/// ComponentWrapper wraps a WASM component as an airssys-rt Actor.
///
/// Each WASM component instance becomes one Actor, providing:
/// - Lifecycle management via Actor pre_start/post_stop hooks
/// - Message handling via Actor handle_message
/// - Fault tolerance via supervision (ErrorAction)
/// - Dependency injection via `Arc<E>` where `E: RuntimeEngine` (static dispatch per S6.2)
///
/// # Architecture
///
/// ComponentWrapper sits in Layer 3A and uses trait abstraction for
/// runtime engine access. The concrete `WasmtimeEngine` is injected
/// by the system/ module (Layer 4) at initialization time.
///
/// # Lifecycle
///
/// 1. Created with ComponentId, RuntimeEngine, and WASM bytes
/// 2. `pre_start()` loads WASM component via RuntimeEngine
/// 3. `handle_message()` processes incoming messages
/// 4. `post_stop()` unloads component and releases resources
///
/// # Examples
///
/// ```rust,ignore
/// use std::sync::Arc;
/// use airssys_wasm::component::wrapper::{ComponentWrapper, ComponentActorMessage};
/// use airssys_wasm::core::component::id::ComponentId;
///
/// // Engine type is determined at construction time (static dispatch)
/// let engine = Arc::new(WasmtimeEngine::new());
/// let id = ComponentId::new("system", "database", "prod");
/// let wasm_bytes = vec![/* WASM binary */];
///
/// // wrapper is ComponentWrapper<WasmtimeEngine> - fully monomorphized
/// let wrapper = ComponentWrapper::new(id, engine, wasm_bytes);
/// ```
pub struct ComponentWrapper<E: RuntimeEngine> {
    /// Unique identifier for this component instance
    id: ComponentId,

    /// Runtime engine for WASM execution (injected, static dispatch per S6.2)
    engine: Arc<E>,

    /// Handle to loaded component (Some after start, None before/after)
    handle: Option<ComponentHandle>,

    /// WASM component binary bytes
    wasm_bytes: Vec<u8>,
}

// Manual Debug implementation - engine field uses opaque display
// (RuntimeEngine trait does not require Debug)
impl<E: RuntimeEngine> fmt::Debug for ComponentWrapper<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ComponentWrapper")
            .field("id", &self.id)
            .field("engine", &"<RuntimeEngine>")
            .field("handle", &self.handle)
            .field("wasm_bytes_len", &self.wasm_bytes.len())
            .finish()
    }
}

impl<E: RuntimeEngine> ComponentWrapper<E> {
    /// Creates a new ComponentWrapper.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for this component instance
    /// * `engine` - Runtime engine for WASM execution (injected, static dispatch per S6.2)
    /// * `wasm_bytes` - WASM component binary bytes
    ///
    /// # Returns
    ///
    /// A new ComponentWrapper instance. The component is NOT loaded yet -
    /// loading happens during `pre_start()`.
    pub fn new(id: ComponentId, engine: Arc<E>, wasm_bytes: Vec<u8>) -> Self {
        Self {
            id,
            engine,
            handle: None,
            wasm_bytes,
        }
    }

    /// Returns a reference to the component's identifier.
    pub fn id(&self) -> &ComponentId {
        &self.id
    }

    /// Returns a reference to the component handle if loaded.
    ///
    /// Returns `Some` after successful `pre_start()`, `None` otherwise.
    pub fn handle(&self) -> Option<&ComponentHandle> {
        self.handle.as_ref()
    }

    /// Returns true if the component is currently loaded.
    pub fn is_loaded(&self) -> bool {
        self.handle.is_some()
    }
}

/// Error type for ComponentWrapper operations.
///
/// Uses `thiserror` for consistent error handling across the codebase.
#[derive(Debug, Clone, Error)]
pub enum ComponentWrapperError {
    /// Component not started - operation called before pre_start.
    #[error("Component not started: {0}")]
    NotStarted(String),

    /// WASM execution failed.
    #[error("WASM execution failed: {0}")]
    WasmExecution(String),
}

impl ComponentWrapperError {
    /// Create a new error from a WasmError.
    pub fn from_wasm_error(err: WasmError) -> Self {
        Self::WasmExecution(err.to_string())
    }

    /// Create a new NotStarted error with a message.
    pub fn new(message: impl Into<String>) -> Self {
        Self::NotStarted(message.into())
    }
}

#[async_trait]
impl<E: RuntimeEngine + 'static> Actor for ComponentWrapper<E> {
    type Message = ComponentActorMessage;
    type Error = ComponentWrapperError;

    /// Handle incoming messages by delegating to the runtime engine.
    ///
    /// # Message Processing
    ///
    /// - `HandleMessage` - Calls engine.call_handle_message()
    /// - `HandleCallback` - Calls engine.call_handle_callback()
    /// - `Shutdown` - Unloads component and returns
    ///
    /// # Errors
    ///
    /// Returns error if component is not loaded or WASM execution fails.
    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        match message {
            ComponentActorMessage::HandleMessage(component_msg) => {
                let handle = self.handle.as_ref().ok_or_else(|| {
                    ComponentWrapperError::new(
                        "Component not started - handle_message called before pre_start",
                    )
                })?;

                // Delegate to runtime engine for WASM execution
                let _response = self
                    .engine
                    .call_handle_message(handle, &component_msg)
                    .map_err(ComponentWrapperError::from_wasm_error)?;

                // Response routing is delegated to messaging module
                Ok(())
            }

            ComponentActorMessage::HandleCallback(component_msg) => {
                let handle = self.handle.as_ref().ok_or_else(|| {
                    ComponentWrapperError::new(
                        "Component not started - handle_callback called before pre_start",
                    )
                })?;

                self.engine
                    .call_handle_callback(handle, &component_msg)
                    .map_err(ComponentWrapperError::from_wasm_error)?;

                Ok(())
            }

            ComponentActorMessage::Shutdown => {
                // Graceful shutdown - unload component
                if let Some(handle) = self.handle.take() {
                    self.engine
                        .unload_component(&handle)
                        .map_err(ComponentWrapperError::from_wasm_error)?;
                }
                Ok(())
            }
        }
    }

    /// Initialize the component by loading WASM binary.
    ///
    /// Called before the actor starts processing messages.
    /// Loads the WASM component using the injected RuntimeEngine.
    async fn pre_start<B: MessageBroker<Self::Message>>(
        &mut self,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        // Load WASM component via injected engine
        let handle = self
            .engine
            .load_component(&self.id, &self.wasm_bytes)
            .map_err(ComponentWrapperError::from_wasm_error)?;

        self.handle = Some(handle);
        Ok(())
    }

    /// Cleanup when actor stops.
    ///
    /// Unloads the WASM component and releases all resources.
    async fn post_stop<B: MessageBroker<Self::Message>>(
        &mut self,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        // Unload component if still loaded
        if let Some(handle) = self.handle.take() {
            self.engine
                .unload_component(&handle)
                .map_err(ComponentWrapperError::from_wasm_error)?;
        }
        Ok(())
    }

    /// Handle errors and return supervision decision.
    ///
    /// Default strategy: Stop on error (conservative approach).
    /// Supervisors can override this via SupervisorConfig.
    async fn on_error<B: MessageBroker<Self::Message>>(
        &mut self,
        _error: Self::Error,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> ErrorAction {
        // Default: stop actor on error
        // Supervisor can restart based on SupervisorConfig
        ErrorAction::Stop
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Mutex;

    // Import airssys-rt test utilities
    use airssys_rt::broker::InMemoryMessageBroker;
    use airssys_rt::util::ActorAddress;

    // Import test-only types from core
    use crate::core::component::message::{MessageMetadata, MessagePayload};

    // ========================================
    // Mock RuntimeEngine for Testing
    // ========================================

    struct MockRuntimeEngine {
        load_called: AtomicBool,
        unload_called: AtomicBool,
        handle_message_called: AtomicBool,
        handle_callback_called: AtomicBool,
        should_fail_load: AtomicBool,
        should_fail_message: AtomicBool,
        last_message: Mutex<Option<ComponentMessage>>,
    }

    impl MockRuntimeEngine {
        fn new() -> Self {
            Self {
                load_called: AtomicBool::new(false),
                unload_called: AtomicBool::new(false),
                handle_message_called: AtomicBool::new(false),
                handle_callback_called: AtomicBool::new(false),
                should_fail_load: AtomicBool::new(false),
                should_fail_message: AtomicBool::new(false),
                last_message: Mutex::new(None),
            }
        }

        fn with_load_failure(self) -> Self {
            self.should_fail_load.store(true, Ordering::SeqCst);
            self
        }

        fn with_message_failure(self) -> Self {
            self.should_fail_message.store(true, Ordering::SeqCst);
            self
        }
    }

    impl RuntimeEngine for MockRuntimeEngine {
        fn load_component(
            &self,
            id: &ComponentId,
            _bytes: &[u8],
        ) -> Result<ComponentHandle, WasmError> {
            self.load_called.store(true, Ordering::SeqCst);

            if self.should_fail_load.load(Ordering::SeqCst) {
                return Err(WasmError::InstantiationFailed(
                    "Mock load failure".to_string(),
                ));
            }

            Ok(ComponentHandle::new(id.clone(), 1))
        }

        fn unload_component(&self, _handle: &ComponentHandle) -> Result<(), WasmError> {
            self.unload_called.store(true, Ordering::SeqCst);
            Ok(())
        }

        fn call_handle_message(
            &self,
            _handle: &ComponentHandle,
            msg: &ComponentMessage,
        ) -> Result<Option<MessagePayload>, WasmError> {
            self.handle_message_called.store(true, Ordering::SeqCst);
            if let Ok(mut guard) = self.last_message.lock() {
                *guard = Some(msg.clone());
            }

            if self.should_fail_message.load(Ordering::SeqCst) {
                return Err(WasmError::RuntimeError("Mock message failure".to_string()));
            }

            Ok(None)
        }

        fn call_handle_callback(
            &self,
            _handle: &ComponentHandle,
            msg: &ComponentMessage,
        ) -> Result<(), WasmError> {
            self.handle_callback_called.store(true, Ordering::SeqCst);
            if let Ok(mut guard) = self.last_message.lock() {
                *guard = Some(msg.clone());
            }
            Ok(())
        }
    }

    // ========================================
    // Helper Functions
    // ========================================

    fn create_test_id() -> ComponentId {
        ComponentId::new("test", "component", "instance-1")
    }

    fn create_test_message(sender: ComponentId) -> ComponentMessage {
        ComponentMessage::new(
            sender,
            MessagePayload::new(vec![1, 2, 3]),
            MessageMetadata::default(),
        )
    }

    fn create_test_context(
    ) -> ActorContext<ComponentActorMessage, InMemoryMessageBroker<ComponentActorMessage>> {
        let address = ActorAddress::anonymous();
        let broker = InMemoryMessageBroker::new();
        ActorContext::new(address, broker)
    }

    // ========================================
    // ComponentWrapper Creation Tests
    // ========================================

    #[test]
    fn test_wrapper_creation() {
        let id = create_test_id();
        let engine = Arc::new(MockRuntimeEngine::new());
        let bytes = vec![0u8; 100];

        let wrapper = ComponentWrapper::new(id.clone(), engine, bytes);

        assert_eq!(wrapper.id(), &id);
        assert!(wrapper.handle().is_none());
        assert!(!wrapper.is_loaded());
    }

    #[test]
    fn test_wrapper_id_accessor() {
        let id = ComponentId::new("system", "database", "prod");
        let engine = Arc::new(MockRuntimeEngine::new());
        let wrapper = ComponentWrapper::new(id.clone(), engine, vec![]);

        assert_eq!(wrapper.id().namespace, "system");
        assert_eq!(wrapper.id().name, "database");
        assert_eq!(wrapper.id().instance, "prod");
        assert_eq!(wrapper.id().to_string_id(), "system/database/prod");
    }

    #[test]
    fn test_wrapper_handle_before_start() {
        let id = create_test_id();
        let engine = Arc::new(MockRuntimeEngine::new());
        let wrapper = ComponentWrapper::new(id, engine, vec![]);

        assert!(wrapper.handle().is_none());
        assert!(!wrapper.is_loaded());
    }

    #[test]
    fn test_wrapper_debug_impl() {
        let id = create_test_id();
        let engine = Arc::new(MockRuntimeEngine::new());
        let wrapper = ComponentWrapper::new(id, engine, vec![1, 2, 3]);

        let debug_str = format!("{:?}", wrapper);
        println!("Debug string: {}", debug_str);
        assert!(debug_str.contains("ComponentWrapper"));
        // ComponentId uses struct format in Debug, not Display format
        assert!(debug_str.contains("namespace"));
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("<RuntimeEngine>"));
        assert!(debug_str.contains("wasm_bytes_len: 3"));
    }

    // ========================================
    // ComponentActorMessage Tests
    // ========================================

    #[test]
    fn test_message_handle_message_variant() {
        let msg = create_test_message(create_test_id());
        let actor_msg = ComponentActorMessage::HandleMessage(msg.clone());

        if let ComponentActorMessage::HandleMessage(inner) = actor_msg {
            assert_eq!(inner.sender, msg.sender);
        } else {
            panic!("Expected HandleMessage variant");
        }
    }

    #[test]
    fn test_message_handle_callback_variant() {
        let msg = create_test_message(create_test_id());
        let actor_msg = ComponentActorMessage::HandleCallback(msg);

        assert!(matches!(
            actor_msg,
            ComponentActorMessage::HandleCallback(_)
        ));
    }

    #[test]
    fn test_message_shutdown_variant() {
        let actor_msg = ComponentActorMessage::Shutdown;
        assert!(matches!(actor_msg, ComponentActorMessage::Shutdown));
    }

    #[test]
    fn test_message_debug_impl() {
        let msg = ComponentActorMessage::Shutdown;
        let debug_str = format!("{:?}", msg);
        assert!(debug_str.contains("Shutdown"));
    }

    #[test]
    fn test_message_clone() {
        let msg = create_test_message(create_test_id());
        let actor_msg = ComponentActorMessage::HandleMessage(msg);
        let cloned = actor_msg.clone();

        assert!(matches!(cloned, ComponentActorMessage::HandleMessage(_)));
    }

    #[test]
    fn test_message_type_const() {
        assert_eq!(
            ComponentActorMessage::MESSAGE_TYPE,
            "component_actor_message"
        );
    }

    #[test]
    fn test_message_priority_default() {
        let msg = ComponentActorMessage::Shutdown;
        assert_eq!(msg.priority(), MessagePriority::Normal);
    }

    // ========================================
    // ComponentWrapperError Tests
    // ========================================

    #[test]
    fn test_error_from_wasm_error() {
        let wasm_err = WasmError::RuntimeError("test error".to_string());
        let wrapper_err = ComponentWrapperError::from_wasm_error(wasm_err);

        assert!(wrapper_err.to_string().contains("test error"));
    }

    #[test]
    fn test_error_new() {
        let err = ComponentWrapperError::new("custom message");
        assert!(err.to_string().contains("custom message"));
    }

    #[test]
    fn test_error_display() {
        let err = ComponentWrapperError::new("display test");
        let display = format!("{}", err);
        assert!(display.contains("Component not started"));
        assert!(display.contains("display test"));
    }

    #[test]
    fn test_error_debug() {
        let err = ComponentWrapperError::new("debug test");
        let debug = format!("{:?}", err);
        assert!(debug.contains("NotStarted"));
    }

    // ========================================
    // Actor Trait Async Tests
    // ========================================

    #[tokio::test]
    async fn test_actor_pre_start_success() {
        let id = create_test_id();
        let mock_engine = Arc::new(MockRuntimeEngine::new());
        let mut wrapper = ComponentWrapper::new(id, Arc::clone(&mock_engine), vec![0u8; 100]);

        let mut context = create_test_context();

        // Before pre_start, component is not loaded
        assert!(!wrapper.is_loaded());

        // Call pre_start
        let result = wrapper.pre_start(&mut context).await;

        assert!(result.is_ok());
        assert!(wrapper.is_loaded());
        assert!(mock_engine.load_called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_actor_pre_start_failure() {
        let id = create_test_id();
        let mock_engine = Arc::new(MockRuntimeEngine::new().with_load_failure());
        let mut wrapper = ComponentWrapper::new(id, Arc::clone(&mock_engine), vec![0u8; 100]);

        let mut context = create_test_context();

        let result = wrapper.pre_start(&mut context).await;

        assert!(result.is_err());
        assert!(!wrapper.is_loaded());
        assert!(mock_engine.load_called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_actor_post_stop_with_loaded_component() {
        let id = create_test_id();
        let mock_engine = Arc::new(MockRuntimeEngine::new());
        let mut wrapper = ComponentWrapper::new(id, Arc::clone(&mock_engine), vec![0u8; 100]);

        let mut context = create_test_context();

        // First load the component
        let _ = wrapper.pre_start(&mut context).await;
        assert!(wrapper.is_loaded());

        // Now stop it
        let result = wrapper.post_stop(&mut context).await;

        assert!(result.is_ok());
        assert!(!wrapper.is_loaded());
        assert!(mock_engine.unload_called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_actor_post_stop_without_loaded_component() {
        let id = create_test_id();
        let mock_engine = Arc::new(MockRuntimeEngine::new());
        let mut wrapper = ComponentWrapper::new(id, Arc::clone(&mock_engine), vec![0u8; 100]);

        let mut context = create_test_context();

        // Don't load - just call post_stop
        let result = wrapper.post_stop(&mut context).await;

        assert!(result.is_ok());
        // unload_called should be false since there was nothing to unload
        assert!(!mock_engine.unload_called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_actor_handle_message_success() {
        let id = create_test_id();
        let mock_engine = Arc::new(MockRuntimeEngine::new());
        let mut wrapper =
            ComponentWrapper::new(id.clone(), Arc::clone(&mock_engine), vec![0u8; 100]);

        let mut context = create_test_context();

        // First load the component
        let _ = wrapper.pre_start(&mut context).await;

        // Create and send a message
        let component_msg = create_test_message(id);
        let actor_msg = ComponentActorMessage::HandleMessage(component_msg);

        let result = wrapper.handle_message(actor_msg, &mut context).await;

        assert!(result.is_ok());
        assert!(mock_engine.handle_message_called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_actor_handle_message_without_start() {
        let id = create_test_id();
        let engine = Arc::new(MockRuntimeEngine::new());
        let mut wrapper = ComponentWrapper::new(id.clone(), engine, vec![0u8; 100]);

        let mut context = create_test_context();

        // Don't call pre_start - try to handle message directly
        let component_msg = create_test_message(id);
        let actor_msg = ComponentActorMessage::HandleMessage(component_msg);

        let result = wrapper.handle_message(actor_msg, &mut context).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Component not started"));
    }

    #[tokio::test]
    async fn test_actor_handle_message_failure() {
        let id = create_test_id();
        let mock_engine = Arc::new(MockRuntimeEngine::new().with_message_failure());
        let mut wrapper =
            ComponentWrapper::new(id.clone(), Arc::clone(&mock_engine), vec![0u8; 100]);

        let mut context = create_test_context();

        // First load the component
        let _ = wrapper.pre_start(&mut context).await;

        // Create and send a message
        let component_msg = create_test_message(id);
        let actor_msg = ComponentActorMessage::HandleMessage(component_msg);

        let result = wrapper.handle_message(actor_msg, &mut context).await;

        assert!(result.is_err());
        assert!(mock_engine.handle_message_called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_actor_handle_callback_success() {
        let id = create_test_id();
        let mock_engine = Arc::new(MockRuntimeEngine::new());
        let mut wrapper =
            ComponentWrapper::new(id.clone(), Arc::clone(&mock_engine), vec![0u8; 100]);

        let mut context = create_test_context();

        // First load the component
        let _ = wrapper.pre_start(&mut context).await;

        // Create and send a callback
        let component_msg = create_test_message(id);
        let actor_msg = ComponentActorMessage::HandleCallback(component_msg);

        let result = wrapper.handle_message(actor_msg, &mut context).await;

        assert!(result.is_ok());
        assert!(mock_engine.handle_callback_called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_actor_handle_callback_without_start() {
        let id = create_test_id();
        let engine = Arc::new(MockRuntimeEngine::new());
        let mut wrapper = ComponentWrapper::new(id.clone(), engine, vec![0u8; 100]);

        let mut context = create_test_context();

        // Don't call pre_start - try to handle callback directly
        let component_msg = create_test_message(id);
        let actor_msg = ComponentActorMessage::HandleCallback(component_msg);

        let result = wrapper.handle_message(actor_msg, &mut context).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Component not started"));
    }

    #[tokio::test]
    async fn test_actor_shutdown_message() {
        let id = create_test_id();
        let mock_engine = Arc::new(MockRuntimeEngine::new());
        let mut wrapper = ComponentWrapper::new(id, Arc::clone(&mock_engine), vec![0u8; 100]);

        let mut context = create_test_context();

        // First load the component
        let _ = wrapper.pre_start(&mut context).await;
        assert!(wrapper.is_loaded());

        // Send shutdown message
        let actor_msg = ComponentActorMessage::Shutdown;
        let result = wrapper.handle_message(actor_msg, &mut context).await;

        assert!(result.is_ok());
        assert!(!wrapper.is_loaded());
        assert!(mock_engine.unload_called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_actor_shutdown_without_start() {
        let id = create_test_id();
        let mock_engine = Arc::new(MockRuntimeEngine::new());
        let mut wrapper = ComponentWrapper::new(id, Arc::clone(&mock_engine), vec![0u8; 100]);

        let mut context = create_test_context();

        // Don't call pre_start - send shutdown directly
        let actor_msg = ComponentActorMessage::Shutdown;
        let result = wrapper.handle_message(actor_msg, &mut context).await;

        // Should succeed even without start (no-op)
        assert!(result.is_ok());
        assert!(!wrapper.is_loaded());
        // unload_called should be false since there was nothing to unload
        assert!(!mock_engine.unload_called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_actor_on_error_returns_stop() {
        let id = create_test_id();
        let engine = Arc::new(MockRuntimeEngine::new());
        let mut wrapper = ComponentWrapper::new(id, engine, vec![0u8; 100]);

        let mut context = create_test_context();

        let error = ComponentWrapperError::new("test error");
        let action = wrapper.on_error(error, &mut context).await;

        assert_eq!(action, ErrorAction::Stop);
    }

    // ========================================
    // Full Lifecycle Integration Test
    // ========================================

    #[tokio::test]
    async fn test_full_actor_lifecycle() {
        let id = create_test_id();
        let mock_engine = Arc::new(MockRuntimeEngine::new());
        let mut wrapper =
            ComponentWrapper::new(id.clone(), Arc::clone(&mock_engine), vec![0u8; 100]);

        let mut context = create_test_context();

        // Phase 1: Pre-start
        let result = wrapper.pre_start(&mut context).await;
        assert!(result.is_ok());
        assert!(wrapper.is_loaded());

        // Phase 2: Handle some messages
        let msg1 = ComponentActorMessage::HandleMessage(create_test_message(id.clone()));
        let result = wrapper.handle_message(msg1, &mut context).await;
        assert!(result.is_ok());

        let msg2 = ComponentActorMessage::HandleCallback(create_test_message(id));
        let result = wrapper.handle_message(msg2, &mut context).await;
        assert!(result.is_ok());

        // Phase 3: Post-stop
        let result = wrapper.post_stop(&mut context).await;
        assert!(result.is_ok());
        assert!(!wrapper.is_loaded());

        // Verify all engine methods were called
        assert!(mock_engine.load_called.load(Ordering::SeqCst));
        assert!(mock_engine.handle_message_called.load(Ordering::SeqCst));
        assert!(mock_engine.handle_callback_called.load(Ordering::SeqCst));
        assert!(mock_engine.unload_called.load(Ordering::SeqCst));
    }

    // ========================================
    // Send + Sync Bounds Tests
    // ========================================

    #[test]
    fn test_component_actor_message_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<ComponentActorMessage>();
        assert_sync::<ComponentActorMessage>();
    }

    #[test]
    fn test_component_wrapper_error_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<ComponentWrapperError>();
        assert_sync::<ComponentWrapperError>();
    }

    #[test]
    fn test_component_wrapper_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<ComponentWrapper<MockRuntimeEngine>>();
        assert_sync::<ComponentWrapper<MockRuntimeEngine>>();
    }
}
