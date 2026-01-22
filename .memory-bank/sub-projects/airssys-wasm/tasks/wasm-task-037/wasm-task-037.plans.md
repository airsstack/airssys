# WASM-TASK-037: Implementation Plans

## References
- ADR-WASM-031 (Component & Messaging Module Design)
- ADR-WASM-026 (Implementation Roadmap)
- ADR-WASM-025 (Clean Slate Architecture)
- ADR-WASM-023 (Module Boundary Enforcement)
- PROJECTS_STANDARD.md (§2.1, §2.2, §4.3)

---

## Actions

### Action 1: Implement ComponentWrapper struct and message type

**Objective**: Create the ComponentWrapper that bridges WASM components with airssys-rt Actor system.

**Detailed Steps**:

#### Step 1.1: Create `src/component/wrapper.rs`

```rust
//! ComponentWrapper - Wraps WASM components as airssys-rt Actors
//!
//! Each WASM component instance becomes one Actor, providing:
//! - Lifecycle management via Child trait (start/stop)
//! - Message handling via Actor trait
//! - Fault tolerance via supervision
//! - Dependency injection via Arc<dyn RuntimeEngine>

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
use airssys_rt::{Actor, Child, Message, SupervisorNode};

// Layer 3: Internal module imports
use crate::core::component::handle::ComponentHandle;
use crate::core::component::id::ComponentId;
use crate::core::component::message::ComponentMessage;
use crate::core::component::MessagePayload;
use crate::core::errors::wasm::WasmError;
use crate::core::runtime::traits::RuntimeEngine;

/// ComponentWrapper wraps a WASM component as an airssys-rt Actor
///
/// Key Design: Uses trait `RuntimeEngine` (not concrete `WasmtimeEngine`)
/// The concrete engine is injected by system/ module
pub struct ComponentWrapper {
    id: ComponentId,
    engine: Arc<dyn RuntimeEngine>,
    handle: Option<ComponentHandle>,
    wasm_bytes: Vec<u8>,
}

impl ComponentWrapper {
    pub fn new(
        id: ComponentId,
        engine: Arc<dyn RuntimeEngine>,
        wasm_bytes: Vec<u8>,
    ) -> Self {
        Self {
            id,
            engine,
            handle: None,
            wasm_bytes,
        }
    }

    pub fn id(&self) -> &ComponentId {
        &self.id
    }

    pub fn handle(&self) -> Option<&ComponentHandle> {
        self.handle.as_ref()
    }
}

/// Message type for ComponentWrapper actor
#[derive(Debug)]
pub enum ComponentActorMessage {
    HandleMessage(ComponentMessage),
    HandleCallback(ComponentMessage),
    Shutdown,
}

impl Message for ComponentActorMessage {}

/// Child trait implementation - manages WASM lifecycle
impl Child for ComponentWrapper {
    type Error = WasmError;

    async fn start(&mut self) -> Result<(), Self::Error> {
        // Load the WASM component using the injected engine
        let handle = self.engine.load_component(&self.id, &self.wasm_bytes)?;
        self.handle = Some(handle);
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Self::Error> {
        if let Some(handle) = self.handle.take() {
            self.engine.unload_component(&handle)?;
        }
        Ok(())
    }
}

/// Actor trait implementation - handles messages
impl Actor for ComponentWrapper {
    type Message = ComponentActorMessage;
    type Error = WasmError;

    async fn handle(&mut self, msg: Self::Message) -> Result<(), Self::Error> {
        let handle = self.handle.as_ref().ok_or_else(|| {
            WasmError::RuntimeError("Component not started".to_string())
        })?;

        match msg {
            ComponentActorMessage::HandleMessage(component_msg) => {
                // Delegate to the runtime engine
                let _response = self.engine.call_handle_message(handle, &component_msg)?;
                // Response handling delegated to messaging module
                Ok(())
            }
            ComponentActorMessage::HandleCallback(component_msg) => {
                self.engine.call_handle_callback(handle, &component_msg)?;
                Ok(())
            }
            ComponentActorMessage::Shutdown => {
                self.stop().await?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrapper_creation() {
        // Test requires a mock RuntimeEngine
        // This will be implemented as part of this task
    }

    #[test]
    fn test_wrapper_id_accessor() {
        // Verify id() returns correct ComponentId
    }

    #[test]
    fn test_wrapper_handle_before_start() {
        // Verify handle() returns None before start()
    }

    #[test]
    fn test_message_type_variants() {
        // Verify ComponentActorMessage enum variants
    }
}
```

#### Step 1.2: Update `src/component/mod.rs`

```rust
//! Component module - Actor system integration for WASM components
//!
//! This module integrates WASM components with the airssys-rt actor system.

pub mod wrapper;

pub use wrapper::{ComponentActorMessage, ComponentWrapper};
```

**Deliverables**:
- `src/component/wrapper.rs` with ComponentWrapper struct
- ComponentActorMessage enum with Debug trait
- Child trait implementation
- Actor trait implementation
- Module declaration in `src/component/mod.rs`

**Constraints**:
- Must not import from `runtime/`, `security/`, or `system/`
- Must follow §2.1 3-Layer imports
- Must follow §2.2 No FQN in type annotations
- Must use Arc<dyn RuntimeEngine> for dependency injection

---

### Action 2: Implement Unit Tests

**Objective**: Comprehensive unit tests for ComponentWrapper lifecycle and message handling.

**Detailed Steps**:

#### Step 2.1: Create Mock RuntimeEngine for Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Mock RuntimeEngine for testing
    struct MockRuntimeEngine {
        load_called: Mutex<bool>,
        unload_called: Mutex<bool>,
        handle_message_called: Mutex<bool>,
        handle_callback_called: Mutex<bool>,
    }

    impl MockRuntimeEngine {
        fn new() -> Self {
            Self {
                load_called: Mutex::new(false),
                unload_called: Mutex::new(false),
                handle_message_called: Mutex::new(false),
                handle_callback_called: Mutex::new(false),
            }
        }
    }

    impl RuntimeEngine for MockRuntimeEngine {
        fn load_component(
            &self,
            _id: &ComponentId,
            _bytes: &[u8],
        ) -> Result<ComponentHandle, WasmError> {
            *self.load_called.lock().unwrap() = true;
            Ok(ComponentHandle::new(ComponentId::new("test")))
        }

        fn unload_component(&self, _handle: &ComponentHandle) -> Result<(), WasmError> {
            *self.unload_called.lock().unwrap() = true;
            Ok(())
        }

        fn call_handle_message(
            &self,
            _handle: &ComponentHandle,
            _msg: &ComponentMessage,
        ) -> Result<Option<MessagePayload>, WasmError> {
            *self.handle_message_called.lock().unwrap() = true;
            Ok(None)
        }

        fn call_handle_callback(
            &self,
            _handle: &ComponentHandle,
            _msg: &ComponentMessage,
        ) -> Result<(), WasmError> {
            *self.handle_callback_called.lock().unwrap() = true;
            Ok(())
        }
    }

    #[test]
    fn test_wrapper_creation() {
        let id = ComponentId::new("test-component");
        let engine: Arc<dyn RuntimeEngine> = Arc::new(MockRuntimeEngine::new());
        let bytes = vec![0u8; 100];

        let wrapper = ComponentWrapper::new(id.clone(), engine, bytes);

        assert_eq!(wrapper.id(), &id);
        assert!(wrapper.handle().is_none());
    }

    #[test]
    fn test_wrapper_id_accessor() {
        let id = ComponentId::new("test-component");
        let engine: Arc<dyn RuntimeEngine> = Arc::new(MockRuntimeEngine::new());
        let wrapper = ComponentWrapper::new(id.clone(), engine, vec![]);

        assert_eq!(wrapper.id().to_string_id(), "test-component");
    }

    #[test]
    fn test_wrapper_handle_before_start() {
        let id = ComponentId::new("test-component");
        let engine: Arc<dyn RuntimeEngine> = Arc::new(MockRuntimeEngine::new());
        let wrapper = ComponentWrapper::new(id, engine, vec![]);

        assert!(wrapper.handle().is_none());
    }

    #[tokio::test]
    async fn test_child_start() {
        let id = ComponentId::new("test-component");
        let engine = Arc::new(MockRuntimeEngine::new());
        let mut wrapper = ComponentWrapper::new(id, Arc::clone(&engine), vec![]);

        let result = wrapper.start().await;
        assert!(result.is_ok());
        assert!(wrapper.handle().is_some());

        let load_called = *engine.load_called.lock().unwrap();
        assert!(load_called);
    }

    #[tokio::test]
    async fn test_child_stop() {
        let id = ComponentId::new("test-component");
        let engine = Arc::new(MockRuntimeEngine::new());
        let mut wrapper = ComponentWrapper::new(id, Arc::clone(&engine), vec![]);

        // Start first
        wrapper.start().await.unwrap();
        assert!(wrapper.handle().is_some());

        // Then stop
        let result = wrapper.stop().await;
        assert!(result.is_ok());
        assert!(wrapper.handle().is_none());

        let unload_called = *engine.unload_called.lock().unwrap();
        assert!(unload_called);
    }

    #[tokio::test]
    async fn test_actor_handle_message() {
        let id = ComponentId::new("test-component");
        let engine = Arc::new(MockRuntimeEngine::new());
        let mut wrapper = ComponentWrapper::new(id.clone(), Arc::clone(&engine), vec![]);

        // Start component
        wrapper.start().await.unwrap();

        // Handle message
        let msg = ComponentMessage {
            sender: id.clone(),
            payload: MessagePayload::Text("test".to_string()),
            metadata: Default::default(),
        };
        let result = wrapper.handle(ComponentActorMessage::HandleMessage(msg)).await;
        assert!(result.is_ok());

        let called = *engine.handle_message_called.lock().unwrap();
        assert!(called);
    }

    #[tokio::test]
    async fn test_actor_handle_callback() {
        let id = ComponentId::new("test-component");
        let engine = Arc::new(MockRuntimeEngine::new());
        let mut wrapper = ComponentWrapper::new(id.clone(), Arc::clone(&engine), vec![]);

        wrapper.start().await.unwrap();

        let msg = ComponentMessage {
            sender: id.clone(),
            payload: MessagePayload::Text("callback".to_string()),
            metadata: Default::default(),
        };
        let result = wrapper.handle(ComponentActorMessage::HandleCallback(msg)).await;
        assert!(result.is_ok());

        let called = *engine.handle_callback_called.lock().unwrap();
        assert!(called);
    }

    #[tokio::test]
    async fn test_actor_handle_shutdown() {
        let id = ComponentId::new("test-component");
        let engine = Arc::new(MockRuntimeEngine::new());
        let mut wrapper = ComponentWrapper::new(id, Arc::clone(&engine), vec![]);

        wrapper.start().await.unwrap();

        let result = wrapper.handle(ComponentActorMessage::Shutdown).await;
        assert!(result.is_ok());
        assert!(wrapper.handle().is_none());

        let unload_called = *engine.unload_called.lock().unwrap();
        assert!(unload_called);
    }

    #[tokio::test]
    async fn test_actor_handle_before_start_fails() {
        let id = ComponentId::new("test-component");
        let engine = Arc::new(MockRuntimeEngine::new());
        let mut wrapper = ComponentWrapper::new(id.clone(), engine, vec![]);

        // Try to handle message without starting
        let msg = ComponentMessage {
            sender: id.clone(),
            payload: MessagePayload::Text("test".to_string()),
            metadata: Default::default(),
        };
        let result = wrapper.handle(ComponentActorMessage::HandleMessage(msg)).await;
        assert!(result.is_err());
    }
}
```

**Deliverables**:
- MockRuntimeEngine for testing
- Unit tests for ComponentWrapper creation
- Unit tests for Child trait (start/stop)
- Unit tests for Actor trait (all message types)
- Unit tests for error cases

**Constraints**:
- Tests must be in #[cfg(test)] module
- Use tokio::test for async tests
- Mock external dependencies

---

## Verification Section

### Automated Tests
```bash
# Unit tests for wrapper module
cargo test -p airssys-wasm --lib -- component::wrapper

# All component module tests
cargo test -p airssys-wasm --lib -- component

# Build verification
cargo build -p airssys-wasm

# Clippy
cargo clippy -p airssys-wasm --lib -- -D warnings
```

### Architecture Compliance
```bash
# Verify no forbidden imports in wrapper.rs
grep -rn "use crate::runtime" src/component/wrapper.rs  # Should be empty
grep -rn "use crate::security" src/component/wrapper.rs  # Should be empty
grep -rn "use crate::system" src/component/wrapper.rs    # Should be empty

# Verify no FQN in type annotations
grep -rn "std::" src/component/wrapper.rs | grep -v "^.*use " | grep "::"  # Should be empty after filtering imports
grep -rn "airssys_rt::" src/component/wrapper.rs | grep -v "^.*use " | grep "::"  # Should be empty
```

---

## Success Criteria
- [ ] `src/component/wrapper.rs` exists and compiles
- [ ] ComponentWrapper struct with correct fields
- [ ] ComponentActorMessage enum with all variants
- [ ] Child trait implementation (start/stop)
- [ ] Actor trait implementation (handle)
- [ ] Unit tests pass (10 tests minimum)
- [ ] MockRuntimeEngine for testing
- [ ] `cargo clippy -p airssys-wasm --lib -- -D warnings` passes
- [ ] No forbidden imports (architecture compliance)
- [ ] §2.1 3-Layer imports verified
- [ ] §2.2 No FQN verified
