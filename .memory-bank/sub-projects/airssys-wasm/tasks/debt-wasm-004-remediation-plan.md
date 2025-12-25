# DEBT-WASM-004 Remediation Plan: Message Delivery Runtime Glue

**Task ID:** DEBT-WASM-004-REMEDIATION  
**Created:** 2025-12-22  
**Priority:** 🔴 CRITICAL  
**Status:** NOT STARTED  
**Estimated Effort:** 16-24 hours (2-3 days)  
**Blocks:** All inter-component messaging functionality

---

## Executive Summary

### The Problem

All messaging components exist and work individually, but they are **NOT wired together**:

| Component | Status | The Gap |
|-----------|--------|---------|
| `WasmEngine::call_handle_message()` | ✅ Works | Never called by message flow |
| `WasmEngine::call_handle_callback()` | ✅ Works | Never called (response_rx dropped) |
| `SendMessageHostFunction` | ✅ Works | Publishes to broker, but no consumer |
| `SendRequestHostFunction` | ✅ Works | `response_rx` dropped immediately at line 692 |
| `ActorSystemSubscriber` | ✅ Works | `start()` never called, mailboxes never registered |
| `ComponentSpawner` | ✅ Works | Doesn't create/register mailbox channels |
| `ComponentActor::handle_message()` | ✅ Works | Never receives messages |

### The Solution

Add ~150-200 lines of "glue code" to wire everything together across 4 phases.

### Success Criteria

After remediation, this test MUST pass:

```rust
#[tokio::test]
async fn test_end_to_end_message_flow() {
    // 1. Initialize runtime with ComponentA and ComponentB
    // 2. ComponentA calls send-message("component-b", payload)
    // 3. ComponentB's WASM handle-message export is invoked
    // 4. Verify sender="component-a" and payload matches
}

#[tokio::test]  
async fn test_request_response_callback() {
    // 1. ComponentA calls send-request("component-b", request, 5000ms)
    // 2. ComponentB handles request, returns response
    // 3. ComponentA's WASM handle-callback export is invoked
    // 4. Verify correlation_id matches and response payload correct
}
```

---

## Phase Overview

| Phase | Description | Effort | Dependencies |
|-------|-------------|--------|--------------|
| **Phase 1** | Infrastructure Setup | 4-6 hours | None |
| **Phase 2** | Fire-and-Forget Flow | 4-6 hours | Phase 1 |
| **Phase 3** | Request-Response Flow | 4-6 hours | Phase 2 |
| **Phase 4** | Integration Testing | 4-6 hours | Phase 3 |

**Total Estimated Effort:** 16-24 hours

---

## Phase 1: Infrastructure Setup (4-6 hours)

### Objective

Create the infrastructure needed for message delivery without breaking existing code.

### Task 1.1: Add Mailbox Receiver to ComponentActor

**File:** `airssys-wasm/src/actor/component/component_actor.rs`

**Changes:**
```rust
// Add new field
pub struct ComponentActor<S = ()> {
    // ... existing fields ...
    
    /// Mailbox receiver for incoming messages (optional until wired)
    mailbox_rx: Option<tokio::sync::mpsc::UnboundedReceiver<ComponentMessage>>,
}

impl<S> ComponentActor<S> {
    // Add setter method
    pub fn set_mailbox_receiver(
        &mut self, 
        rx: tokio::sync::mpsc::UnboundedReceiver<ComponentMessage>
    ) {
        self.mailbox_rx = Some(rx);
    }
    
    // Add getter for message loop
    pub fn take_mailbox_receiver(
        &mut self
    ) -> Option<tokio::sync::mpsc::UnboundedReceiver<ComponentMessage>> {
        self.mailbox_rx.take()
    }
}
```

**Effort:** 1 hour  
**Tests:** 2-3 unit tests for set/take methods

---

### Task 1.2: Add ActorSystemSubscriber to ComponentSpawner

**File:** `airssys-wasm/src/actor/component/component_spawner.rs`

**Changes:**
```rust
use crate::actor::message::ActorSystemSubscriber;

pub struct ComponentSpawner<B: MessageBroker<ComponentMessage>> {
    // ... existing fields ...
    
    /// Actor system subscriber for message delivery
    actor_system_subscriber: Arc<ActorSystemSubscriber<B>>,
}

impl<B: MessageBroker<ComponentMessage> + Clone + Send + Sync + 'static> ComponentSpawner<B> {
    pub fn new(
        actor_system: ActorSystem<ComponentMessage, B>,
        registry: ComponentRegistry,
        broker: B,
        actor_system_subscriber: Arc<ActorSystemSubscriber<B>>,  // NEW
    ) -> Self {
        Self {
            actor_system,
            registry,
            broker,
            supervisor: None,
            actor_system_subscriber,  // NEW
        }
    }
    
    // Also add with_supervision variant
}
```

**Effort:** 1 hour  
**Tests:** Update existing tests to pass new parameter

---

### Task 1.3: Add WasmEngine to SendRequestHostFunction

**File:** `airssys-wasm/src/runtime/async_host.rs`

**Changes:**
```rust
pub struct SendRequestHostFunction {
    messaging_service: Arc<MessagingService>,
    engine: Arc<WasmEngine>,  // NEW: For calling handle-callback
}

impl SendRequestHostFunction {
    pub fn new(
        messaging_service: Arc<MessagingService>,
        engine: Arc<WasmEngine>,  // NEW
    ) -> Self {
        Self { 
            messaging_service,
            engine,
        }
    }
}
```

**Effort:** 1 hour  
**Tests:** Update builder tests

---

### Task 1.4: Create RuntimeOrchestrator Struct

**File:** `airssys-wasm/src/runtime/orchestrator.rs` (NEW)

**Purpose:** Central coordination point that owns all components and wires them together.

```rust
//! Runtime orchestrator for component lifecycle and message routing.

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::actor::component::{ComponentSpawner, ComponentRegistry};
use crate::actor::message::{ActorSystemSubscriber, SubscriberManager};
use crate::core::{ComponentId, ComponentMessage, WasmError};
use crate::runtime::{WasmEngine, MessagingService};
use airssys_rt::broker::InMemoryMessageBroker;
use airssys_rt::system::{ActorSystem, SystemConfig};

/// Central orchestrator that coordinates component lifecycle and messaging.
///
/// RuntimeOrchestrator owns all major subsystems and ensures they are
/// properly wired together for end-to-end message delivery.
pub struct RuntimeOrchestrator {
    /// WASM execution engine
    engine: Arc<WasmEngine>,
    
    /// Message broker for pub-sub
    broker: Arc<InMemoryMessageBroker<ComponentMessage>>,
    
    /// Actor system for component actors
    actor_system: ActorSystem<ComponentMessage, InMemoryMessageBroker<ComponentMessage>>,
    
    /// Component registry
    registry: ComponentRegistry,
    
    /// Component spawner
    spawner: ComponentSpawner<InMemoryMessageBroker<ComponentMessage>>,
    
    /// Actor system subscriber for message routing
    subscriber: Arc<ActorSystemSubscriber<InMemoryMessageBroker<ComponentMessage>>>,
    
    /// Messaging service for host functions
    messaging_service: Arc<MessagingService>,
    
    /// Active message loops (component_id -> JoinHandle)
    message_loops: Arc<RwLock<HashMap<ComponentId, tokio::task::JoinHandle<()>>>>,
    
    /// Running state
    is_running: bool,
}

impl RuntimeOrchestrator {
    /// Create new orchestrator with default configuration.
    pub fn new() -> Result<Self, WasmError> {
        let engine = Arc::new(WasmEngine::new()?);
        let broker = Arc::new(InMemoryMessageBroker::new());
        let actor_system = ActorSystem::new(
            SystemConfig::default(), 
            (*broker).clone()
        );
        let registry = ComponentRegistry::new();
        let subscriber_manager = Arc::new(SubscriberManager::new());
        
        let subscriber = Arc::new(ActorSystemSubscriber::new(
            Arc::clone(&broker),
            registry.clone(),
            subscriber_manager,
        ));
        
        let spawner = ComponentSpawner::new(
            actor_system.clone(),
            registry.clone(),
            (*broker).clone(),
            Arc::clone(&subscriber),
        );
        
        let messaging_service = Arc::new(MessagingService::with_broker(
            (*broker).clone()
        ));
        
        Ok(Self {
            engine,
            broker,
            actor_system,
            registry,
            spawner,
            subscriber,
            messaging_service,
            message_loops: Arc::new(RwLock::new(HashMap::new())),
            is_running: false,
        })
    }
    
    /// Start the orchestrator (initializes message routing).
    pub async fn start(&mut self) -> Result<(), WasmError> {
        if self.is_running {
            return Ok(());
        }
        
        // Start the message subscriber
        self.subscriber.start().await?;
        self.is_running = true;
        
        Ok(())
    }
    
    /// Stop the orchestrator.
    pub async fn stop(&mut self) -> Result<(), WasmError> {
        if !self.is_running {
            return Ok(());
        }
        
        // Stop all message loops
        let mut loops = self.message_loops.write().await;
        for (_, handle) in loops.drain() {
            handle.abort();
        }
        
        // Stop subscriber
        // (subscriber.stop() - if mutable access available)
        
        self.is_running = false;
        Ok(())
    }
    
    /// Get reference to WASM engine.
    pub fn engine(&self) -> Arc<WasmEngine> {
        Arc::clone(&self.engine)
    }
    
    /// Get reference to messaging service.
    pub fn messaging_service(&self) -> Arc<MessagingService> {
        Arc::clone(&self.messaging_service)
    }
    
    /// Get reference to component spawner.
    pub fn spawner(&self) -> &ComponentSpawner<InMemoryMessageBroker<ComponentMessage>> {
        &self.spawner
    }
}
```

**Effort:** 2 hours  
**Tests:** 3-4 tests (creation, start/stop, accessors)

---

### Phase 1 Deliverables

| Deliverable | File | Lines |
|-------------|------|-------|
| Mailbox receiver field | `component_actor.rs` | ~20 |
| Subscriber in spawner | `component_spawner.rs` | ~15 |
| Engine in SendRequest | `async_host.rs` | ~10 |
| RuntimeOrchestrator | `orchestrator.rs` (NEW) | ~150 |
| Tests | Various | ~50 |

**Phase 1 Total:** ~245 lines

---

## Phase 2: Fire-and-Forget Flow (4-6 hours)

### Objective

Complete the fire-and-forget message flow so that `send-message` actually delivers to target component's WASM `handle-message` export.

### Task 2.1: Wire Mailbox Creation in ComponentSpawner

**File:** `airssys-wasm/src/actor/component/component_spawner.rs`

**Changes to `spawn_component()`:**

```rust
pub async fn spawn_component(
    &self,
    component_id: ComponentId,
    wasm_path: PathBuf,
    metadata: ComponentMetadata,
    capabilities: CapabilitySet,
) -> Result<ActorAddress, WasmError> {
    // 1. Create ComponentActor instance (existing)
    let mut actor = ComponentActor::new(component_id.clone(), metadata, capabilities, ());

    // 2. Inject MessageBroker bridge (existing)
    let broker_wrapper = Arc::new(crate::actor::message::MessageBrokerWrapper::new(
        self.broker.clone(),
    ));
    actor.set_broker(broker_wrapper as Arc<dyn crate::actor::message::MessageBrokerBridge>);

    // ================================================================
    // NEW: Create and register mailbox channel
    // ================================================================
    let (mailbox_tx, mailbox_rx) = tokio::sync::mpsc::unbounded_channel();
    
    // Register mailbox with ActorSystemSubscriber
    self.actor_system_subscriber
        .register_mailbox(component_id.clone(), mailbox_tx)
        .await
        .map_err(|e| WasmError::internal(format!(
            "Failed to register mailbox for {}: {}", 
            component_id.as_str(), e
        )))?;
    
    // Store receiver in actor
    actor.set_mailbox_receiver(mailbox_rx);
    // ================================================================

    // 3. Spawn via ActorSystem (existing)
    let actor_ref = self.actor_system.spawn()
        .with_name(component_id.as_str())
        .spawn(actor)
        .await
        .map_err(|e| WasmError::actor_error(format!(
            "Failed to spawn component {}: {}", component_id.as_str(), e
        )))?;

    // 4. Register in registry (existing)
    self.registry.register(component_id.clone(), actor_ref.clone())
        .map_err(|e| WasmError::internal(format!(
            "Failed to register component {} in registry: {}",
            component_id.as_str(), e
        )))?;

    Ok(actor_ref)
}
```

**Effort:** 1 hour  
**Tests:** 2-3 tests verifying mailbox is registered

---

### Task 2.2: Add Message Loop to RuntimeOrchestrator

**File:** `airssys-wasm/src/runtime/orchestrator.rs`

**Add method:**

```rust
impl RuntimeOrchestrator {
    /// Spawn a component and start its message loop.
    pub async fn spawn_component(
        &self,
        component_id: ComponentId,
        wasm_path: PathBuf,
        metadata: ComponentMetadata,
        capabilities: CapabilitySet,
    ) -> Result<ActorAddress, WasmError> {
        // 1. Spawn component via spawner
        let actor_ref = self.spawner
            .spawn_component(component_id.clone(), wasm_path, metadata, capabilities)
            .await?;
        
        // 2. Start message loop for this component
        self.start_message_loop(component_id.clone()).await?;
        
        Ok(actor_ref)
    }
    
    /// Start message loop for a component.
    async fn start_message_loop(&self, component_id: ComponentId) -> Result<(), WasmError> {
        // Get the actor from registry
        let actor_address = self.registry.lookup(&component_id)
            .map_err(|e| WasmError::component_not_found(format!(
                "Component {} not found in registry: {}", 
                component_id.as_str(), e
            )))?;
        
        // Get mailbox receiver from actor
        // NOTE: This requires architectural consideration - the actor owns the receiver
        // We need to either:
        // a) Take receiver before spawning into ActorSystem
        // b) Use a different pattern (channel from outside)
        
        // For now, use pattern (b): receiver is stored separately
        let mailbox_rx = self.pending_receivers
            .write()
            .await
            .remove(&component_id)
            .ok_or_else(|| WasmError::internal("No pending receiver for component"))?;
        
        // Clone what we need for the task
        let engine = Arc::clone(&self.engine);
        let component_id_clone = component_id.clone();
        
        // Spawn message loop task
        let handle = tokio::spawn(async move {
            let mut rx = mailbox_rx;
            
            while let Some(msg) = rx.recv().await {
                match &msg {
                    ComponentMessage::InterComponent { sender, payload, .. } |
                    ComponentMessage::InterComponentWithCorrelation { sender, payload, .. } => {
                        // Invoke WASM handle-message
                        if let Err(e) = engine.call_handle_message(
                            &component_id_clone,
                            sender.as_str(),
                            payload,
                        ).await {
                            tracing::error!(
                                component_id = %component_id_clone.as_str(),
                                sender = %sender.as_str(),
                                error = %e,
                                "Failed to invoke handle-message"
                            );
                        }
                    }
                    _ => {
                        tracing::debug!(
                            component_id = %component_id_clone.as_str(),
                            msg_type = ?std::any::type_name_of_val(&msg),
                            "Received non-inter-component message"
                        );
                    }
                }
            }
            
            tracing::info!(
                component_id = %component_id_clone.as_str(),
                "Message loop ended (channel closed)"
            );
        });
        
        // Store handle
        self.message_loops.write().await.insert(component_id, handle);
        
        Ok(())
    }
}
```

**Effort:** 2 hours  
**Tests:** 3-4 tests for message loop lifecycle

---

### Task 2.3: Verify ActorSystemSubscriber Delivery

**File:** Already implemented in `actor_system_subscriber.rs`

**Verification:** Ensure `route_message_to_subscribers()` actually calls `sender.send()`.

Current implementation at line 454-455:
```rust
sender.send(envelope.payload).map_err(|e| {
    WasmError::messaging_error(format!(
        "Failed to deliver message to {}: {}",
        target.as_str(), e
    ))
})?;
```

**This is already correct!** ✅

**Effort:** 30 minutes (verification only)  
**Tests:** Verify existing tests cover this path

---

### Task 2.4: Fire-and-Forget Integration Test

**File:** `airssys-wasm/tests/fire_and_forget_e2e_tests.rs` (NEW)

```rust
//! End-to-end tests for fire-and-forget messaging.

use airssys_wasm::runtime::RuntimeOrchestrator;
use airssys_wasm::core::{ComponentId, ComponentMetadata, CapabilitySet, ResourceLimits};
use std::path::PathBuf;

fn test_metadata(name: &str) -> ComponentMetadata {
    ComponentMetadata {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        author: "Test".to_string(),
        description: None,
        required_capabilities: vec![],
        resource_limits: ResourceLimits::default(),
    }
}

#[tokio::test]
async fn test_send_message_delivers_to_target() {
    // 1. Create orchestrator
    let mut orchestrator = RuntimeOrchestrator::new()
        .expect("Failed to create orchestrator");
    
    orchestrator.start().await
        .expect("Failed to start orchestrator");
    
    // 2. Load test WASM components
    // NOTE: Requires test fixture WASM files
    let component_a = ComponentId::new("component-a");
    let component_b = ComponentId::new("component-b");
    
    // For now, test with mock/stub - full test requires WASM fixtures
    // This proves the wiring works
    
    // 3. Send message from A to B
    // (Would require host function invocation context)
    
    // 4. Verify B received the message
    // (Would check B's internal state or use test hooks)
    
    // Cleanup
    orchestrator.stop().await.unwrap();
}

#[tokio::test]
async fn test_message_loop_processes_messages() {
    // Test that message loop correctly invokes handle_message
    // This can be done with a mock WasmEngine or test component
}

#[tokio::test]
async fn test_mailbox_registration_on_spawn() {
    // Verify that spawning a component registers its mailbox
    let mut orchestrator = RuntimeOrchestrator::new().unwrap();
    orchestrator.start().await.unwrap();
    
    // After spawn, mailbox should be registered
    // Verify via subscriber.mailbox_count()
    
    orchestrator.stop().await.unwrap();
}
```

**Effort:** 1.5 hours  
**Tests:** 3-4 integration tests

---

### Phase 2 Deliverables

| Deliverable | File | Lines |
|-------------|------|-------|
| Mailbox wiring in spawner | `component_spawner.rs` | ~25 |
| Message loop in orchestrator | `orchestrator.rs` | ~60 |
| Verification | `actor_system_subscriber.rs` | 0 (existing) |
| Integration tests | `fire_and_forget_e2e_tests.rs` | ~100 |

**Phase 2 Total:** ~185 lines

---

## Phase 3: Request-Response Flow (4-6 hours)

### Objective

Complete the request-response flow so that `send-request` triggers `handle-callback` on the requesting component when response arrives.

### Task 3.1: Fix response_rx Drop in SendRequestHostFunction

**File:** `airssys-wasm/src/runtime/async_host.rs`

**Current (BROKEN):**
```rust
// Line 688-692
let (response_tx, _response_rx) = oneshot::channel::<ResponseMessage>();
//                ^^^^^^^^^^^^^ DROPPED IMMEDIATELY!
```

**Fixed:**
```rust
async fn execute(&self, context: &HostCallContext, args: Vec<u8>) -> WasmResult<Vec<u8>> {
    // ... existing parsing and validation (lines 636-682) ...
    
    // 4. Generate correlation ID
    let correlation_id = Uuid::new_v4();
    
    // 5. Create oneshot channel for response
    let (response_tx, response_rx) = oneshot::channel::<ResponseMessage>();
    
    // 6. Register pending request (existing)
    let pending = PendingRequest {
        correlation_id,
        response_tx,
        requested_at: Instant::now(),
        timeout: Duration::from_millis(timeout_ms),
        from: context.component_id.clone(),
        to: target_id.clone(),
    };
    
    self.messaging_service
        .correlation_tracker()
        .register_pending(pending)
        .await
        .map_err(|e| WasmError::messaging_error(format!("Failed to register request: {e}")))?;
    
    // ================================================================
    // NEW: Spawn response listener task
    // ================================================================
    let engine = Arc::clone(&self.engine);
    let component_id = context.component_id.clone();
    let correlation_id_clone = correlation_id;
    let timeout_duration = Duration::from_millis(timeout_ms);
    let tracker = self.messaging_service.correlation_tracker().clone();
    
    tokio::spawn(async move {
        let result = tokio::time::timeout(timeout_duration, response_rx).await;
        
        match result {
            Ok(Ok(response)) => {
                // Response received - invoke handle-callback
                tracing::debug!(
                    correlation_id = %correlation_id_clone,
                    component_id = %component_id.as_str(),
                    is_error = response.is_error,
                    "Response received, invoking handle-callback"
                );
                
                if let Err(e) = engine.call_handle_callback(
                    &component_id,
                    &correlation_id_clone,
                    &response.payload,
                    response.is_error,
                ).await {
                    tracing::error!(
                        correlation_id = %correlation_id_clone,
                        error = %e,
                        "Failed to invoke handle-callback"
                    );
                }
            }
            Ok(Err(_)) => {
                // Sender dropped (shouldn't happen normally)
                tracing::warn!(
                    correlation_id = %correlation_id_clone,
                    "Response channel sender dropped unexpectedly"
                );
            }
            Err(_elapsed) => {
                // Timeout
                tracing::warn!(
                    correlation_id = %correlation_id_clone,
                    timeout_ms = timeout_ms,
                    "Request timed out"
                );
                
                // Invoke handle-callback with error
                let timeout_error = format!("Request timed out after {}ms", timeout_ms);
                if let Err(e) = engine.call_handle_callback(
                    &component_id,
                    &correlation_id_clone,
                    timeout_error.as_bytes(),
                    true, // is_error = true
                ).await {
                    tracing::error!(
                        correlation_id = %correlation_id_clone,
                        error = %e,
                        "Failed to invoke handle-callback for timeout"
                    );
                }
            }
        }
        
        // Cleanup: remove from tracker
        tracker.remove_pending(&correlation_id_clone).await;
    });
    // ================================================================
    
    // 7. Publish request message (existing)
    let component_message = ComponentMessage::InterComponentWithCorrelation {
        sender: context.component_id.clone(),
        to: target_id,
        payload: request_bytes,
        correlation_id,
    };
    
    let envelope = MessageEnvelope::new(component_message);
    self.messaging_service
        .broker()
        .publish(envelope)
        .await
        .map_err(|e| WasmError::messaging_error(format!("Broker publish failed: {e}")))?;
    
    // 8. Record metrics (existing)
    self.messaging_service.record_publish();
    self.messaging_service.record_request_sent();
    
    // 9. Return correlation ID
    Ok(correlation_id.to_string().into_bytes())
}
```

**Effort:** 2 hours  
**Tests:** 3-4 tests for response handling

---

### Task 3.2: Add send-response Host Function

**File:** `airssys-wasm/src/runtime/async_host.rs`

**New struct:**

```rust
/// Host function for sending response to a pending request.
///
/// Implements the response side of request-response messaging.
/// When a component receives a request (via handle-message), it can
/// call send-response to return a value to the requester.
///
/// # Argument Format
///
/// `[correlation_id_len: u32 LE][correlation_id_bytes][response_bytes]`
pub struct SendResponseHostFunction {
    messaging_service: Arc<MessagingService>,
}

impl SendResponseHostFunction {
    pub fn new(messaging_service: Arc<MessagingService>) -> Self {
        Self { messaging_service }
    }
}

#[async_trait]
impl HostFunction for SendResponseHostFunction {
    fn name(&self) -> &str {
        "messaging::send_response"
    }
    
    fn required_capability(&self) -> Capability {
        Capability::Messaging(TopicPattern::new("*"))
    }
    
    async fn execute(&self, context: &HostCallContext, args: Vec<u8>) -> WasmResult<Vec<u8>> {
        // 1. Parse arguments
        if args.len() < 4 {
            return Err(WasmError::messaging_error("Args too short for correlation ID length"));
        }
        
        let correlation_id_len = u32::from_le_bytes([args[0], args[1], args[2], args[3]]) as usize;
        let id_end = 4 + correlation_id_len;
        
        if args.len() < id_end {
            return Err(WasmError::messaging_error("Args too short for correlation ID"));
        }
        
        let correlation_id_str = String::from_utf8(args[4..id_end].to_vec())
            .map_err(|e| WasmError::messaging_error(format!("Invalid correlation ID UTF-8: {e}")))?;
        
        let correlation_id = Uuid::parse_str(&correlation_id_str)
            .map_err(|e| WasmError::messaging_error(format!("Invalid correlation ID UUID: {e}")))?;
        
        let response_bytes = args[id_end..].to_vec();
        
        // 2. Validate multicodec (if present)
        // Response can be raw bytes or multicodec-prefixed
        
        // 3. Find pending request and resolve it
        let resolved = self.messaging_service
            .correlation_tracker()
            .resolve_pending(&correlation_id, response_bytes, false)
            .await;
        
        match resolved {
            Ok(()) => {
                self.messaging_service.record_response_received();
                Ok(Vec::new())
            }
            Err(e) => Err(WasmError::messaging_error(format!(
                "Failed to resolve pending request {}: {}", correlation_id, e
            ))),
        }
    }
}
```

**Effort:** 1.5 hours  
**Tests:** 3-4 tests for send-response

---

### Task 3.3: Update AsyncHostRegistryBuilder

**File:** `airssys-wasm/src/runtime/async_host.rs`

```rust
impl AsyncHostRegistryBuilder {
    pub fn with_messaging_functions(
        mut self, 
        messaging_service: Arc<MessagingService>,
        engine: Arc<WasmEngine>,  // NEW
    ) -> Self {
        // Fire-and-forget
        let send_fn = SendMessageHostFunction::new(Arc::clone(&messaging_service));
        self.functions.insert(send_fn.name().to_string(), Box::new(send_fn));
        
        // Request-response
        let request_fn = SendRequestHostFunction::new(
            Arc::clone(&messaging_service),
            Arc::clone(&engine),
        );
        self.functions.insert(request_fn.name().to_string(), Box::new(request_fn));
        
        // Response (NEW)
        let response_fn = SendResponseHostFunction::new(messaging_service);
        self.functions.insert(response_fn.name().to_string(), Box::new(response_fn));
        
        self
    }
}
```

**Effort:** 30 minutes  
**Tests:** Update builder test

---

### Task 3.4: Request-Response Integration Test

**File:** `airssys-wasm/tests/request_response_e2e_tests.rs` (NEW)

```rust
//! End-to-end tests for request-response messaging.

#[tokio::test]
async fn test_request_receives_response_callback() {
    // 1. Setup orchestrator with two components
    // 2. Component A sends request to Component B
    // 3. Component B handles request and sends response
    // 4. Verify Component A's handle-callback is invoked
    // 5. Verify correlation ID matches
}

#[tokio::test]
async fn test_request_timeout_triggers_error_callback() {
    // 1. Setup with component that doesn't respond
    // 2. Send request with short timeout
    // 3. Verify handle-callback invoked with is_error=true
}

#[tokio::test]
async fn test_correlation_tracker_cleanup_after_response() {
    // Verify pending request is removed after response
}
```

**Effort:** 1.5 hours  
**Tests:** 3 integration tests

---

### Phase 3 Deliverables

| Deliverable | File | Lines |
|-------------|------|-------|
| Fix response_rx handling | `async_host.rs` | ~70 |
| SendResponseHostFunction | `async_host.rs` | ~80 |
| Builder update | `async_host.rs` | ~10 |
| Integration tests | `request_response_e2e_tests.rs` | ~100 |

**Phase 3 Total:** ~260 lines

---

## Phase 4: Integration Testing (4-6 hours)

### Objective

Create comprehensive integration tests proving the complete message flow works end-to-end.

### Task 4.1: Create Test WASM Fixtures

**Directory:** `airssys-wasm/tests/fixtures/`

**Files to create:**
1. `echo_component.wasm` - Echoes received messages
2. `request_responder.wasm` - Responds to requests with modified payload

**Effort:** 2 hours (may use existing fixtures or create minimal ones)

---

### Task 4.2: Full Integration Test Suite

**File:** `airssys-wasm/tests/message_delivery_e2e_tests.rs` (NEW)

```rust
//! Complete end-to-end message delivery tests.
//!
//! These tests prove that DEBT-WASM-004 is fully resolved.

mod common;

use airssys_wasm::runtime::RuntimeOrchestrator;
use airssys_wasm::core::{ComponentId, ComponentMessage};

/// CRITICAL TEST: Proves fire-and-forget message flow works.
///
/// This is the PRIMARY validation that DEBT-WASM-004 is resolved.
#[tokio::test]
async fn test_fire_and_forget_complete_flow() {
    // 1. Create and start orchestrator
    let mut orchestrator = RuntimeOrchestrator::new().unwrap();
    orchestrator.start().await.unwrap();
    
    // 2. Spawn sender and receiver components
    let sender_id = ComponentId::new("sender");
    let receiver_id = ComponentId::new("receiver");
    
    orchestrator.spawn_component(
        sender_id.clone(),
        PathBuf::from("tests/fixtures/sender.wasm"),
        test_metadata("sender"),
        CapabilitySet::with_messaging(),
    ).await.unwrap();
    
    orchestrator.spawn_component(
        receiver_id.clone(),
        PathBuf::from("tests/fixtures/receiver.wasm"),
        test_metadata("receiver"),
        CapabilitySet::with_messaging(),
    ).await.unwrap();
    
    // 3. Trigger sender to send message
    // (Via direct call or test hook)
    
    // 4. Wait for delivery
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // 5. Verify receiver got the message
    // (Check receiver's state or test hooks)
    
    // Cleanup
    orchestrator.stop().await.unwrap();
}

/// CRITICAL TEST: Proves request-response with callback works.
#[tokio::test]
async fn test_request_response_complete_flow() {
    // Similar to above but with send-request/handle-callback
}

/// Proves messages don't leak to wrong components.
#[tokio::test]
async fn test_message_isolation() {
    // Component C should NOT receive message sent to B
}

/// Proves timeout handling works correctly.
#[tokio::test]
async fn test_request_timeout_handling() {
    // Request to non-responsive component should timeout
    // handle-callback should be invoked with is_error=true
}

/// Proves concurrent messaging works.
#[tokio::test]
async fn test_concurrent_messaging() {
    // Multiple components sending/receiving simultaneously
}
```

**Effort:** 2.5 hours  
**Tests:** 5+ comprehensive tests

---

### Task 4.3: Performance Validation

**File:** `airssys-wasm/benches/message_delivery_benchmarks.rs`

```rust
//! Benchmarks for message delivery performance.

use criterion::{criterion_group, criterion_main, Criterion};

fn message_delivery_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("fire_and_forget_latency", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Measure send-message to handle-message latency
            });
        });
    });
    
    c.bench_function("request_response_roundtrip", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Measure send-request to handle-callback latency
            });
        });
    });
}

criterion_group!(benches, message_delivery_benchmark);
criterion_main!(benches);
```

**Effort:** 1 hour  
**Benchmarks:** 2-3 benchmarks

---

### Task 4.4: Documentation Update

**Files to update:**
- `README.md` - Add message delivery section
- Module documentation in `orchestrator.rs`
- Update DEBT-WASM-004 with resolution

**Effort:** 1 hour

---

### Phase 4 Deliverables

| Deliverable | File | Lines |
|-------------|------|-------|
| WASM test fixtures | `tests/fixtures/` | ~200 (Rust source) |
| E2E test suite | `message_delivery_e2e_tests.rs` | ~250 |
| Benchmarks | `message_delivery_benchmarks.rs` | ~80 |
| Documentation | Various | ~100 |

**Phase 4 Total:** ~630 lines

---

## Summary

### Total Estimated Lines of Code

| Phase | Lines |
|-------|-------|
| Phase 1: Infrastructure | ~245 |
| Phase 2: Fire-and-Forget | ~185 |
| Phase 3: Request-Response | ~260 |
| Phase 4: Integration Testing | ~630 |
| **Total** | **~1,320 lines** |

### Total Estimated Effort

| Phase | Hours |
|-------|-------|
| Phase 1 | 4-6 |
| Phase 2 | 4-6 |
| Phase 3 | 4-6 |
| Phase 4 | 4-6 |
| **Total** | **16-24 hours** |

### Success Criteria

| Criteria | Verification |
|----------|--------------|
| Fire-and-forget works | `test_fire_and_forget_complete_flow` passes |
| Request-response works | `test_request_response_complete_flow` passes |
| Callbacks are invoked | `handle-callback` export proven called |
| No message leakage | `test_message_isolation` passes |
| Timeout handling works | `test_request_timeout_handling` passes |
| All existing tests pass | `cargo test --package airssys-wasm` |
| Zero warnings | `cargo clippy -- -D warnings` |

### Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Architectural changes ripple | Medium | Medium | Phase 1 adds without breaking |
| WASM fixture complexity | Low | Low | Use simple echo components |
| Message loop ownership | Medium | Medium | Clear ownership in orchestrator |
| Test flakiness | Low | Medium | Use deterministic waits |

---

## Appendix: Files to Create/Modify

### New Files

| File | Purpose |
|------|---------|
| `src/runtime/orchestrator.rs` | Central coordination |
| `tests/fire_and_forget_e2e_tests.rs` | F&F integration tests |
| `tests/request_response_e2e_tests.rs` | R/R integration tests |
| `tests/message_delivery_e2e_tests.rs` | Complete E2E tests |
| `tests/fixtures/*.wasm` | Test WASM components |
| `benches/message_delivery_benchmarks.rs` | Performance benchmarks |

### Modified Files

| File | Changes |
|------|---------|
| `src/actor/component/component_actor.rs` | Add mailbox_rx field |
| `src/actor/component/component_spawner.rs` | Wire mailbox creation/registration |
| `src/runtime/async_host.rs` | Fix response_rx, add SendResponseHostFunction |
| `src/runtime/mod.rs` | Export orchestrator |

---

## Next Steps

1. **Review this plan** - Confirm approach with stakeholder
2. **Start Phase 1** - Infrastructure setup (lowest risk)
3. **Iterate through phases** - Each phase builds on previous
4. **Verify with tests** - Comprehensive testing at each phase

---

**Document Version:** 1.0  
**Last Updated:** 2025-12-22  
**Author:** AI Assistant  
**Reviewed By:** [Pending]
