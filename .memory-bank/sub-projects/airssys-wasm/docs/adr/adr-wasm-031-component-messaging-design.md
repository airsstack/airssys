# ADR-WASM-031: Component & Messaging Module Design

**ADR ID:** ADR-WASM-031  
**Created:** 2026-01-05  
**Status:** Accepted  
**Deciders:** Architecture Team  
**Category:** Module Design / Actor Integration  
**Parent:** [ADR-WASM-026](adr-wasm-026-implementation-roadmap-clean-slate-rebuild.md) (Phase 6)

---

## Title

Component & Messaging Module Design for airssys-rt Integration

---

## Context

The `component/` and `messaging/` modules are **Layer 3** of the architecture. They:
- Integrate with airssys-rt Actor system
- Use traits from `core/` (dependency injection)
- Receive concrete implementations from `system/` (Layer 4)

**Import Rules:**
- ✅ Can import: `core/` traits only
- ✅ Can import: `airssys-rt` (external)
- ❌ Cannot import: `runtime/`, `security/` concrete types directly
- ❌ Cannot import: `system/`

### Design Principle

These modules depend on **abstractions** (traits in `core/`), not **implementations**. The `system/` module injects concrete types at runtime.

### References

- [ADR-WASM-006](adr-wasm-006-component-isolation-and-sandboxing.md): Component Isolation
- [ADR-WASM-009](adr-wasm-009-component-communication-model.md): Communication Model
- [ADR-WASM-025](adr-wasm-025-clean-slate-rebuild-architecture.md): Clean-Slate Architecture

---

## Decision

### Module Structures

```
component/
├── mod.rs
├── wrapper.rs          # ComponentWrapper (Actor + Child)
├── registry.rs         # ComponentRegistry
├── spawner.rs          # ComponentSpawner
└── supervisor.rs       # SupervisorConfig

messaging/
├── mod.rs
├── patterns.rs         # FireAndForget, RequestResponse
├── correlation.rs      # CorrelationTracker impl
├── router.rs           # ResponseRouter
└── subscriber.rs       # ComponentSubscriber
```

---

## Component Module Specifications

### component/wrapper.rs

```rust
use std::sync::Arc;

use airssys_rt::{Actor, Child, Message, SupervisorNode};

use crate::core::component::handle::ComponentHandle;
use crate::core::component::id::ComponentId;
use crate::core::component::message::ComponentMessage;
use crate::core::errors::wasm::WasmError;
use crate::core::messaging::payload::MessagePayload;
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
```

---

### component/registry.rs

```rust
use std::collections::HashMap;
use std::sync::RwLock;

use airssys_rt::ActorAddress;

use crate::core::component::id::ComponentId;

/// Registry for tracking loaded components
pub struct ComponentRegistry {
    /// Maps ComponentId to ActorAddress
    components: RwLock<HashMap<ComponentId, ActorAddress>>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            components: RwLock::new(HashMap::new()),
        }
    }

    /// Register a component with its actor address
    pub fn register(&self, id: ComponentId, address: ActorAddress) {
        let mut components = self.components.write().unwrap();
        components.insert(id, address);
    }

    /// Unregister a component
    pub fn unregister(&self, id: &ComponentId) -> Option<ActorAddress> {
        let mut components = self.components.write().unwrap();
        components.remove(id)
    }

    /// Get actor address for a component
    pub fn get(&self, id: &ComponentId) -> Option<ActorAddress> {
        let components = self.components.read().unwrap();
        components.get(id).cloned()
    }

    /// Check if component is registered
    pub fn contains(&self, id: &ComponentId) -> bool {
        let components = self.components.read().unwrap();
        components.contains_key(id)
    }

    /// List all registered component IDs
    pub fn list(&self) -> Vec<ComponentId> {
        let components = self.components.read().unwrap();
        components.keys().cloned().collect()
    }

    /// Count of registered components
    pub fn count(&self) -> usize {
        let components = self.components.read().unwrap();
        components.len()
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

---

### component/spawner.rs

```rust
use std::sync::Arc;

use airssys_rt::{ActorSystem, SupervisorNode};

use crate::core::component::id::ComponentId;
use crate::core::errors::wasm::WasmError;
use crate::core::runtime::traits::{ComponentLoader, RuntimeEngine};

use super::registry::ComponentRegistry;
use super::supervisor::SupervisorConfig;
use super::wrapper::ComponentWrapper;

/// Spawns and manages component actors
pub struct ComponentSpawner {
    engine: Arc<dyn RuntimeEngine>,
    loader: Arc<dyn ComponentLoader>,
    registry: Arc<ComponentRegistry>,
    supervisor_config: SupervisorConfig,
}

impl ComponentSpawner {
    pub fn new(
        engine: Arc<dyn RuntimeEngine>,
        loader: Arc<dyn ComponentLoader>,
        registry: Arc<ComponentRegistry>,
        supervisor_config: SupervisorConfig,
    ) -> Self {
        Self {
            engine,
            loader,
            registry,
            supervisor_config,
        }
    }

    /// Spawn a new component actor
    pub async fn spawn(
        &self,
        actor_system: &ActorSystem,
        id: ComponentId,
    ) -> Result<(), WasmError> {
        // Load component bytes
        let bytes = self.loader.load_bytes(&id)?;
        
        // Validate component
        self.loader.validate(&bytes)?;

        // Create the wrapper with injected engine
        let wrapper = ComponentWrapper::new(
            id.clone(),
            Arc::clone(&self.engine),
            bytes,
        );

        // Spawn as supervised actor
        let address = actor_system
            .spawn_supervised(
                wrapper,
                self.supervisor_config.to_supervisor_node(),
            )
            .await
            .map_err(|e| WasmError::RuntimeError(e.to_string()))?;

        // Register in registry
        self.registry.register(id, address);

        Ok(())
    }

    /// Stop a component
    pub async fn stop(&self, id: &ComponentId) -> Result<(), WasmError> {
        if let Some(address) = self.registry.unregister(id) {
            // Send shutdown message
            // address.send(ComponentActorMessage::Shutdown).await?;
        }
        Ok(())
    }
}
```

---

### component/supervisor.rs

```rust
use std::time::Duration;

use airssys_rt::SupervisorNode;

/// Supervisor configuration for component actors
#[derive(Debug, Clone)]
pub struct SupervisorConfig {
    pub max_restarts: u32,
    pub restart_window: Duration,
    pub backoff_strategy: BackoffStrategy,
}

#[derive(Debug, Clone)]
pub enum BackoffStrategy {
    /// Fixed delay between restarts
    Fixed(Duration),
    /// Exponential backoff with base and max
    Exponential {
        base: Duration,
        max: Duration,
    },
}

impl Default for SupervisorConfig {
    fn default() -> Self {
        Self {
            max_restarts: 3,
            restart_window: Duration::from_secs(60),
            backoff_strategy: BackoffStrategy::Exponential {
                base: Duration::from_millis(100),
                max: Duration::from_secs(30),
            },
        }
    }
}

impl SupervisorConfig {
    pub fn new(max_restarts: u32, restart_window: Duration) -> Self {
        Self {
            max_restarts,
            restart_window,
            backoff_strategy: BackoffStrategy::default(),
        }
    }

    pub fn with_backoff(mut self, strategy: BackoffStrategy) -> Self {
        self.backoff_strategy = strategy;
        self
    }

    /// Convert to airssys-rt SupervisorNode
    pub fn to_supervisor_node(&self) -> SupervisorNode {
        // Map to airssys-rt supervisor configuration
        todo!("Map to airssys-rt SupervisorNode")
    }
}

impl Default for BackoffStrategy {
    fn default() -> Self {
        Self::Exponential {
            base: Duration::from_millis(100),
            max: Duration::from_secs(30),
        }
    }
}
```

---

## Messaging Module Specifications

### messaging/patterns.rs

```rust
use crate::core::component::id::ComponentId;
use crate::core::errors::messaging::MessagingError;
use crate::core::messaging::payload::MessagePayload;

/// Fire-and-forget messaging pattern
pub struct FireAndForget;

impl FireAndForget {
    /// Send message without expecting response
    pub async fn send(
        target: &ComponentId,
        payload: MessagePayload,
        router: &impl MessageSender,
    ) -> Result<(), MessagingError> {
        router.send(target, payload).await
    }
}

/// Request-response messaging pattern
pub struct RequestResponse;

impl RequestResponse {
    /// Send request and await response via callback
    pub async fn request(
        target: &ComponentId,
        payload: MessagePayload,
        timeout_ms: u64,
        router: &impl MessageSender,
        tracker: &impl CorrelationManager,
    ) -> Result<String, MessagingError> {
        // Generate correlation ID
        let correlation_id = uuid::Uuid::new_v4().to_string();
        
        // Register correlation
        tracker.register(&correlation_id, timeout_ms).await?;
        
        // Send request
        router.send_with_correlation(target, payload, &correlation_id).await?;
        
        Ok(correlation_id)
    }
}

/// Trait for sending messages
pub trait MessageSender: Send + Sync {
    async fn send(
        &self,
        target: &ComponentId,
        payload: MessagePayload,
    ) -> Result<(), MessagingError>;

    async fn send_with_correlation(
        &self,
        target: &ComponentId,
        payload: MessagePayload,
        correlation_id: &str,
    ) -> Result<(), MessagingError>;
}

/// Trait for managing correlations
pub trait CorrelationManager: Send + Sync {
    async fn register(&self, id: &str, timeout_ms: u64) -> Result<(), MessagingError>;
    async fn complete(&self, id: &str, response: MessagePayload) -> Result<(), MessagingError>;
    async fn is_pending(&self, id: &str) -> bool;
}
```

---

### messaging/correlation.rs

```rust
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use tokio::sync::oneshot;

use crate::core::errors::messaging::MessagingError;
use crate::core::messaging::payload::MessagePayload;
use crate::core::messaging::traits::CorrelationTracker;

/// Implementation of CorrelationTracker for request-response patterns
pub struct CorrelationTrackerImpl {
    pending: RwLock<HashMap<String, PendingRequest>>,
}

struct PendingRequest {
    sender: oneshot::Sender<MessagePayload>,
    deadline: Instant,
}

impl CorrelationTrackerImpl {
    pub fn new() -> Self {
        Self {
            pending: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new correlation and return a receiver for the response
    pub fn create(&self, correlation_id: String, timeout_ms: u64) -> oneshot::Receiver<MessagePayload> {
        let (sender, receiver) = oneshot::channel();
        
        let request = PendingRequest {
            sender,
            deadline: Instant::now() + Duration::from_millis(timeout_ms),
        };

        let mut pending = self.pending.write().unwrap();
        pending.insert(correlation_id, request);

        receiver
    }

    /// Clean up expired correlations
    pub fn cleanup_expired(&self) {
        let now = Instant::now();
        let mut pending = self.pending.write().unwrap();
        pending.retain(|_, req| req.deadline > now);
    }
}

impl CorrelationTracker for CorrelationTrackerImpl {
    fn register(&self, correlation_id: &str, timeout_ms: u64) -> Result<(), MessagingError> {
        let (sender, _receiver) = oneshot::channel();
        
        let request = PendingRequest {
            sender,
            deadline: Instant::now() + Duration::from_millis(timeout_ms),
        };

        let mut pending = self.pending.write().unwrap();
        pending.insert(correlation_id.to_string(), request);
        
        Ok(())
    }

    fn complete(&self, correlation_id: &str, response: MessagePayload) -> Result<(), MessagingError> {
        let mut pending = self.pending.write().unwrap();
        
        if let Some(request) = pending.remove(correlation_id) {
            if request.deadline > Instant::now() {
                let _ = request.sender.send(response);
                Ok(())
            } else {
                Err(MessagingError::CorrelationTimeout(correlation_id.to_string()))
            }
        } else {
            Err(MessagingError::InvalidMessage(format!(
                "No pending request for correlation {}",
                correlation_id
            )))
        }
    }

    fn is_pending(&self, correlation_id: &str) -> bool {
        let pending = self.pending.read().unwrap();
        pending.contains_key(correlation_id)
    }
}

impl Default for CorrelationTrackerImpl {
    fn default() -> Self {
        Self::new()
    }
}
```

---

### messaging/router.rs

```rust
use std::sync::Arc;

use crate::core::component::id::ComponentId;
use crate::core::component::message::{ComponentMessage, MessageMetadata};
use crate::core::errors::messaging::MessagingError;
use crate::core::messaging::payload::MessagePayload;
use crate::core::messaging::traits::MessageRouter;

use crate::component::registry::ComponentRegistry;

/// Routes messages between components
pub struct ResponseRouter {
    registry: Arc<ComponentRegistry>,
    current_component: ComponentId,
}

impl ResponseRouter {
    pub fn new(registry: Arc<ComponentRegistry>, current_component: ComponentId) -> Self {
        Self {
            registry,
            current_component,
        }
    }

    fn create_message(&self, payload: MessagePayload, correlation_id: Option<String>) -> ComponentMessage {
        ComponentMessage {
            sender: self.current_component.clone(),
            payload,
            metadata: MessageMetadata {
                correlation_id,
                reply_to: Some(self.current_component.clone()),
                timestamp_ms: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                content_type: None,
            },
        }
    }
}

impl MessageRouter for ResponseRouter {
    fn send(&self, target: &ComponentId, payload: MessagePayload) -> Result<(), MessagingError> {
        let address = self.registry.get(target).ok_or_else(|| {
            MessagingError::TargetNotFound(target.to_string_id())
        })?;

        let message = self.create_message(payload, None);
        
        // Send via airssys-rt
        // address.send(ComponentActorMessage::HandleMessage(message))?;
        
        Ok(())
    }

    fn request(
        &self,
        target: &ComponentId,
        payload: MessagePayload,
        timeout_ms: u64,
    ) -> Result<String, MessagingError> {
        let correlation_id = uuid::Uuid::new_v4().to_string();
        
        let address = self.registry.get(target).ok_or_else(|| {
            MessagingError::TargetNotFound(target.to_string_id())
        })?;

        let message = self.create_message(payload, Some(correlation_id.clone()));
        
        // Send via airssys-rt
        // address.send(ComponentActorMessage::HandleMessage(message))?;
        
        Ok(correlation_id)
    }

    fn cancel_request(&self, request_id: &str) -> Result<(), MessagingError> {
        // Implementation for request cancellation
        Ok(())
    }
}
```

---

### messaging/subscriber.rs

```rust
use std::collections::HashMap;
use std::sync::RwLock;

use airssys_rt::MailboxSender;

use crate::core::component::id::ComponentId;
use crate::core::component::message::ComponentMessage;
use crate::core::errors::messaging::MessagingError;

/// Manages mailbox senders for message delivery
pub struct ComponentSubscriber {
    mailbox_senders: RwLock<HashMap<ComponentId, MailboxSender>>,
}

impl ComponentSubscriber {
    pub fn new() -> Self {
        Self {
            mailbox_senders: RwLock::new(HashMap::new()),
        }
    }

    /// Register a mailbox sender for a component
    pub fn register_mailbox(&self, id: ComponentId, sender: MailboxSender) {
        let mut senders = self.mailbox_senders.write().unwrap();
        senders.insert(id, sender);
    }

    /// Unregister a mailbox sender
    pub fn unregister_mailbox(&self, id: &ComponentId) {
        let mut senders = self.mailbox_senders.write().unwrap();
        senders.remove(id);
    }

    /// Deliver message to a component
    pub async fn deliver(
        &self,
        target: &ComponentId,
        message: ComponentMessage,
    ) -> Result<(), MessagingError> {
        let senders = self.mailbox_senders.read().unwrap();
        
        let sender = senders.get(target).ok_or_else(|| {
            MessagingError::TargetNotFound(target.to_string_id())
        })?;

        // Send message via mailbox
        // sender.send(message).await?;
        
        Ok(())
    }
}

impl Default for ComponentSubscriber {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## Dependency Injection Pattern

```
┌──────────────────────────────────────────────────────────────┐
│                      system/ (Layer 4)                       │
│  Creates concrete types and injects into Layer 3             │
│                                                              │
│  let engine: Arc<dyn RuntimeEngine> = Arc::new(WasmtimeEngine);
│  let wrapper = ComponentWrapper::new(id, engine, bytes);     │
└──────────────────────────────────────────────────────────────┘
                              │
                              ▼ (injects)
┌──────────────────────────────────────────────────────────────┐
│              component/ + messaging/ (Layer 3)               │
│  Uses Arc<dyn RuntimeEngine> - never knows concrete type     │
└──────────────────────────────────────────────────────────────┘
```

---

## History

| Date | Version | Change |
|------|---------|--------|
| 2026-01-05 | 1.0 | Initial component & messaging module design |

---

**This ADR defines the component and messaging module structure for Phase 6 of the rebuild.**
