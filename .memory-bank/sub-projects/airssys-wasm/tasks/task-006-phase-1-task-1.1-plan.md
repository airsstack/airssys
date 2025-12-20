# Implementation Plan: WASM-TASK-006 Phase 1 Task 1.1 - MessageBroker Setup for Components

**Status:** approved  
**Created:** 2025-12-20  
**Updated:** 2025-12-21 (Clarifications from KNOWLEDGE-WASM-024)  
**Task ID:** WASM-TASK-006-1.1  
**Phase:** Phase 1 - MessageBroker Integration Foundation  
**Estimated Effort:** 12 hours (1.5 days)  
**Priority:** Critical - Foundation for Inter-Component Communication

---

## Executive Summary

This plan details the implementation of MessageBroker integration into the WASM runtime, establishing the foundation for inter-component communication in Block 5. The implementation leverages the proven airssys-rt InMemoryMessageBroker (211ns routing, 4.7M msg/sec) and integrates it with the existing ComponentActor infrastructure from Block 3.

**Architectural Clarification (KNOWLEDGE-WASM-024):**
- Phase 1 uses **direct ComponentId addressing ONLY** - NO topic-based routing
- ActorSystem subscription to MessageBroker IS the event-driven subscription (runtime-level)
- Components NEVER subscribe manually - Runtime handles all routing transparently
- Two async patterns: fire-and-forget and request-response (both use ComponentId, not topics)

**Key Deliverables:**
- MessageBroker initialization in WasmRuntime
- ActorSystem event-driven subscription (runtime-level, subscribes to MessageBroker)
- ComponentId-based message routing to ComponentActor mailboxes
- Performance validation (≤220ns total routing)

**Success Criteria:**
- MessageBroker routes component messages by ComponentId
- ActorSystem successfully subscribes to MessageBroker and routes to ComponentActor mailboxes
- Routing performance ≤220ns (no regression from 211ns baseline)
- All unit tests pass (100% pass rate)
- Benchmarks validate performance targets

---

## Table of Contents

1. [Context & Dependencies](#context--dependencies)
2. [Technical Design](#technical-design)
3. [Implementation Steps](#implementation-steps)
4. [Code Structure](#code-structure)
5. [Testing Strategy](#testing-strategy)
6. [Performance Validation](#performance-validation)
7. [Quality Gates](#quality-gates)
8. [Integration Checklist](#integration-checklist)
9. [Risk Assessment](#risk-assessment)
10. [References](#references)

---

## Context & Dependencies

### Upstream Dependencies (All Met ✅)

**Block 3 (WASM-TASK-004): Actor System Integration**
- ✅ ComponentActor implemented with Actor + Child traits
- ✅ ActorSystem integration with mailbox infrastructure
- ✅ MessageBroker pattern established (~211ns routing proven)
- ✅ ComponentMessage enum defined for inter-component communication

**Block 4 (WASM-TASK-005): Security & Isolation Layer**
- ✅ Capability-based security system complete
- ✅ Security checks (<2ns overhead) for message permissions (future)

**airssys-rt Infrastructure**
- ✅ InMemoryMessageBroker: Pure pub-sub implementation
- ✅ Performance baseline: 211ns routing, 4.7M msg/sec throughput
- ✅ MessageBroker trait with publish/subscribe API
- ✅ MessageStream for async message reception

### Architecture References

**ADR-WASM-009: Component Communication Model**
- Message-passing via airssys-rt MessageBroker
- ActorSystem as primary subscriber pattern
- Push-based delivery (no polling)
- Host-mediated security enforcement

**KNOWLEDGE-WASM-005: Inter-Component Messaging Architecture**
- Complete messaging specification
- Actor-based message passing patterns
- Integration with airssys-rt MessageBroker

### Current WasmRuntime Architecture

From `airssys-wasm/src/runtime/mod.rs`:
```rust
pub mod async_host;
pub mod engine;
pub mod limits;
pub mod loader;
pub mod store_manager;

pub use engine::WasmEngine;
pub use loader::ComponentLoader;
```

Current structure is modular with clear separation:
- `engine.rs`: WasmEngine implementation
- `loader.rs`: ComponentLoader implementation
- `store_manager.rs`: StoreWrapper for component stores

We'll extend this with:
- `messaging.rs`: MessageBroker integration (~300 lines, simplified from ~400)

### Current ComponentActor Architecture

From `airssys-wasm/src/actor/component/component_actor.rs`:
```rust
pub struct ComponentActor {
    component_id: ComponentId,
    state: Arc<RwLock<ActorState>>,
    wasm_runtime: Option<WasmRuntime>,
    metadata: ComponentMetadata,
    capabilities: CapabilitySet,
    message_count: Arc<AtomicU64>,
}

pub enum ComponentMessage {
    // Messages from other components
    InterComponent { from: ComponentId, data: Vec<u8> },
    // External RPC requests
    Execute { operation: String, data: Vec<u8> },
    // Health check requests
    HealthCheck,
}
```

We'll extend this with:
- Direct ComponentId-based routing (no topic subscriptions)
- MessageBroker reference for publishing
- Message routing from broker to mailbox

---

## Technical Design

### Architecture Overview

```text
┌─────────────────────────────────────────────────────────────────┐
│                     Component Layer                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ Component A  │  │ Component B  │  │ Component C  │          │
│  │  (Actor)     │  │  (Actor)     │  │  (Actor)     │          │
│  └──────┬───────┘  └──────▲───────┘  └──────▲───────┘          │
│         │ send_message     │ handle_message  │                  │
└─────────┼──────────────────┼─────────────────┼──────────────────┘
          │                  │                 │
          ▼                  │                 │
┌─────────────────────────────────────────────────────────────────┐
│                  WasmRuntime (New Module)                        │
│  ┌────────────────────────────────────────────────────┐         │
│  │  MessagingService (runtime/messaging.rs)           │         │
│  │  • initialize_broker() → InMemoryMessageBroker<M>  │         │
│  │  • get_broker() → Arc<MessageBroker>               │         │
│  │  • Direct ComponentId addressing (no topics)       │         │
│  └────────────────────────────────────────────────────┘         │
└─────────────────────────┼───────────────────────────────────────┘
                        │ publish
                        ▼
┌─────────────────────────────────────────────────────────────────┐
│         airssys-rt MessageBroker (Pure Pub-Sub)                 │
│  ┌────────────────────────────────────────────────────┐         │
│  │  InMemoryMessageBroker<ComponentMessage>           │         │
│  │  • publish(envelope) → broadcast to subscribers    │         │
│  │  • subscribe() → MessageStream                     │         │
│  │  • ~211ns routing latency (proven)                 │         │
│  │  • 4.7M messages/sec throughput                    │         │
│  └────────────────────────────────────────────────────┘         │
└─────────────────────────┼───────────────────────────────────────┘
                        │ subscribe (runtime-level)
                        ▼
┌─────────────────────────────────────────────────────────────────┐
│                      ActorSystem                                 │
│  ┌────────────────────────────────────────────────────┐         │
│  │  ActorSystemSubscriber (actor/message/...)         │         │
│  │  • THIS IS THE EVENT-DRIVEN SUBSCRIPTION           │         │
│  │  • Subscribes to MessageBroker at initialization   │         │
│  │  • Resolves ComponentId → ActorAddress             │         │
│  │  • Routes to ComponentActor mailbox                │         │
│  │  • Delivers via mailbox.send(envelope)             │         │
│  └────────────────────────────────────────────────────┘         │
└─────────────────────────┼───────────────────────────────────────┘
                        │
          ┌─────────────┼─────────────┐
          │             │             │
          ▼             ▼             ▼
     Component A   Component B   Component C
     (mailbox)     (mailbox)     (mailbox)
```

### Data Structures

#### 1. MessagingService (runtime/messaging.rs)

```rust
/// Service managing MessageBroker integration for components.
///
/// MessagingService is responsible for:
/// - Initializing the MessageBroker singleton
/// - Providing broker access to components
/// - Coordinating with ActorSystem subscriber
///
/// NOTE: Phase 1 uses direct ComponentId addressing (no topic subscriptions)
pub struct MessagingService {
    /// Shared MessageBroker instance for all components
    broker: Arc<InMemoryMessageBroker<ComponentMessage>>,
    
    /// Metrics for monitoring
    metrics: Arc<MessagingMetrics>,
}

/// Component message type (already defined in ComponentActor)
pub enum ComponentMessage {
    InterComponent {
        from: ComponentId,
        to: ComponentId,    // Direct addressing (no topics in Phase 1)
        data: Vec<u8>
    },
    Execute { operation: String, data: Vec<u8> },
    HealthCheck,
}
```

### Data Structures

#### 1. MessagingService (runtime/messaging.rs)

#### 2. ActorSystemSubscriber (actor/message/actor_system_subscriber.rs)

```rust
/// ActorSystem subscriber that routes messages to ComponentActor mailboxes.
///
/// **THIS IS THE EVENT-DRIVEN SUBSCRIPTION** (KNOWLEDGE-WASM-024):
/// - ActorSystem subscribes to MessageBroker at runtime initialization
/// - This is runtime-level subscription, NOT component-level
/// - Components are addressed by ComponentId (direct addressing)
/// - Components NEVER subscribe manually
///
/// ActorSystemSubscriber:
/// - Subscribes to MessageBroker event stream
/// - Receives all published messages
/// - Resolves ComponentId to actor address
/// - Routes messages to ComponentActor mailboxes
pub struct ActorSystemSubscriber {
    /// Reference to MessageBroker
    broker: Arc<InMemoryMessageBroker<ComponentMessage>>,
    
    /// ComponentActor registry for address resolution
    actor_registry: Arc<RwLock<HashMap<ComponentId, ActorAddress>>>,
    
    /// Message stream from broker
    message_stream: MessageStream<ComponentMessage>,
    
    /// Routing statistics
    stats: Arc<RwLock<RoutingStats>>,
}

impl ActorSystemSubscriber {
    /// Subscribe to MessageBroker and start routing loop.
    pub async fn start(&mut self) -> Result<(), WasmError>;
    
    /// Routing loop: receive messages and route to mailboxes.
    async fn routing_loop(&mut self) -> Result<(), WasmError>;
    
    /// Resolve ComponentId to actor address.
    fn resolve_actor(&self, component_id: &ComponentId) -> Option<ActorAddress>;
}
```

### API Surface

#### Public API (MessagingService)

#### Public API (MessagingService)

```rust
impl MessagingService {
    /// Create new MessagingService with initialized broker.
    pub fn new() -> Self;
    
    /// Get reference to MessageBroker.
    pub fn broker(&self) -> Arc<InMemoryMessageBroker<ComponentMessage>>;
    
    /// Get messaging statistics.
    pub async fn get_stats(&self) -> MessagingStats;
}

// NOTE: No subscribe_component/unsubscribe_component APIs in Phase 1
// Components are addressed directly by ComponentId (no manual subscriptions)
```

### API Surface

#### Public API (MessagingService)

### Message Flow

#### Flow 1: Component Registration (Runtime-Level)

### Message Flow

#### Flow 1: Component Registration (Runtime-Level)

```text
1. WasmRuntime initializes ActorSystem
   ↓
2. ActorSystem subscribes to MessageBroker
   (THIS IS THE EVENT-DRIVEN SUBSCRIPTION)
   ↓
3. Component A starts and registers with ActorSystem
   ↓
4. ActorSystemSubscriber maps: ComponentId("A") → ActorAddress
   ↓
5. Component A is now addressable by ComponentId
   (No manual subscription by component!)
```

#### Flow 2: Message Publication and Routing (Direct ComponentId)

```text
1. Component A publishes message to ComponentId("component-b")
   ↓
2. MessageBroker.publish(
     ComponentMessage::InterComponent {
       from: "component-a",
       to: "component-b",     // Direct addressing
       data: encoded_message,
     }
   )
   ↓
3. Broker broadcasts to ALL subscribers (~211ns)
   ↓
4. ActorSystemSubscriber receives message
   ↓
5. Resolve ComponentId("component-b") → actor_address via registry
   ↓
6. Send to ComponentActor mailbox (~9ns)
   ↓
7. ComponentActor.handle_message() invoked
   ↓
8. WASM handle-message export called (Phase 2)
```

### Performance Breakdown

**Target: ≤220ns total routing overhead**

| Operation | Latency | Implementation |
|-----------|---------|----------------|
| MessageBroker.publish() | 211ns | airssys-rt InMemoryMessageBroker (proven) |
| ActorSystem subscriber receive | <1ns | Async channel receive |
| ComponentId → address lookup | ~5ns | HashMap lookup |
| mailbox.send() | ~3ns | Tokio unbounded channel send |
| **Total Overhead** | **≤220ns** | **Within target** |

**Note:** No topic routing overhead in Phase 1 (direct ComponentId addressing)

---

## Implementation Steps

### Phase 1: Foundation Setup (3 hours)

#### Step 1.1: Create messaging.rs Module (2 hours)

**File:** `airssys-wasm/src/runtime/messaging.rs`

**Tasks:**
1. Create `MessagingService` struct with MessageBroker initialization
2. Add `MessagingMetrics` for monitoring
3. Implement public API: `new()`, `broker()`, `get_stats()`
4. Add comprehensive rustdoc documentation
5. Ensure thread-safety with Arc<RwLock<T>>

**Note:** Simplified from original - no SubscriptionRegistry, no topic management

**Code Template:**
```rust
//! MessageBroker integration for inter-component communication.
//!
//! This module provides the MessagingService which manages the MessageBroker
//! singleton and coordinates runtime-level message routing.
//!
//! **Phase 1 Scope (KNOWLEDGE-WASM-024):**
//! - Direct ComponentId addressing only
//! - No topic-based routing
//! - ActorSystem handles subscriptions (runtime-level)
//! - Components never subscribe manually

use std::sync::Arc;
use tokio::sync::RwLock;
use airssys_rt::broker::InMemoryMessageBroker;
use crate::core::{ComponentId, WasmError};
use crate::actor::ComponentMessage;

/// Service managing MessageBroker integration.
pub struct MessagingService {
    broker: Arc<InMemoryMessageBroker<ComponentMessage>>,
    metrics: Arc<MessagingMetrics>,
}

impl MessagingService {
    /// Create new MessagingService with initialized broker.
    pub fn new() -> Self {
        Self {
            broker: Arc::new(InMemoryMessageBroker::new()),
            metrics: Arc::new(MessagingMetrics::default()),
        }
    }
    
    /// Get reference to MessageBroker.
    pub fn broker(&self) -> Arc<InMemoryMessageBroker<ComponentMessage>> {
        Arc::clone(&self.broker)
    }
    
    // ... implement remaining methods
}
```

**Success Criteria:**
- Module compiles without errors
- All public APIs documented with rustdoc
- Unit tests for broker initialization pass

#### Step 1.2: Update mod.rs to Export New Module (0.5 hours)

**File:** `airssys-wasm/src/runtime/mod.rs`

**Tasks:**
1. Declare `messaging` module
2. Re-export public types
3. Update module-level documentation

**Code:**
```rust
pub mod messaging;

pub use messaging::{MessagingService, MessagingMetrics, MessagingStats};
```

**Success Criteria:**
- Module accessible via `use airssys_wasm::runtime::*;`

#### Step 1.3: Update ComponentMessage Enum (0.5 hours)

**File:** `airssys-wasm/src/actor/component/component_actor.rs`

**Tasks:**
1. Update `InterComponent` variant with `to` field for direct addressing
2. Implement `Message` trait for ComponentMessage (if not already)
3. Add serialization support with serde

**Code:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComponentMessage {
    InterComponent {
        from: ComponentId,
        to: ComponentId,    // Direct ComponentId addressing (Phase 1)
        data: Vec<u8>,
    },
    Execute {
        operation: String,
        data: Vec<u8>,
    },
    HealthCheck,
}
```

**Success Criteria:**
- ComponentMessage implements Message trait
- Serialization works correctly
- Direct ComponentId addressing supported

---

### Phase 2: ActorSystem Subscriber Implementation (5 hours)

#### Step 2.1: Create actor_system_subscriber.rs (2 hours)

**File:** `airssys-wasm/src/actor/message/actor_system_subscriber.rs`

**Important Context (KNOWLEDGE-WASM-024):**
- THIS IS THE EVENT-DRIVEN SUBSCRIPTION (runtime-level, not component-level)
- ActorSystem subscribes to MessageBroker at initialization
- Components are addressed by ComponentId (direct addressing)
- Components NEVER subscribe manually

**Tasks:**
1. Create `ActorSystemSubscriber` struct
2. Implement `start()` method to subscribe to MessageBroker
3. Implement `routing_loop()` for message routing by ComponentId
4. Implement `resolve_actor()` for ComponentId → actor_address resolution
5. Add error handling for routing failures
6. Add routing statistics tracking

**Code Template:**
```rust
//! ActorSystem subscriber for routing messages to ComponentActor mailboxes.
//!
//! **THIS IS THE EVENT-DRIVEN SUBSCRIPTION** (KNOWLEDGE-WASM-024):
//! - ActorSystem subscribes to MessageBroker event stream at runtime initialization
//! - This is runtime-level subscription, NOT component-level
//! - Components addressed by ComponentId (direct addressing, not topics)
//! - Components NEVER subscribe manually - runtime handles all routing

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use airssys_rt::broker::{MessageBroker, MessageStream};
use crate::actor::ComponentMessage;
use crate::core::{ComponentId, WasmError};

/// ActorSystem subscriber routing messages to ComponentActor mailboxes.
pub struct ActorSystemSubscriber {
    broker: Arc<InMemoryMessageBroker<ComponentMessage>>,
    actor_registry: Arc<RwLock<HashMap<ComponentId, ActorAddress>>>,
    message_stream: Option<MessageStream<ComponentMessage>>,
    stats: Arc<RwLock<RoutingStats>>,
}

impl ActorSystemSubscriber {
    /// Subscribe to MessageBroker and start routing loop.
    pub async fn start(&mut self) -> Result<(), WasmError> {
        // Subscribe to broker
        let stream = self.broker.subscribe().await
            .map_err(|e| WasmError::Runtime(format!("Broker subscribe failed: {:?}", e)))?;
        
        self.message_stream = Some(stream);
        
        // Start routing loop
        self.routing_loop().await
    }
    
    /// Routing loop: receive messages and route to mailboxes by ComponentId.
    async fn routing_loop(&mut self) -> Result<(), WasmError> {
        let mut stream = self.message_stream.take()
            .ok_or_else(|| WasmError::Runtime("No message stream".to_string()))?;
        
        while let Some(envelope) = stream.recv().await {
            self.route_message(envelope).await?;
        }
        
        Ok(())
    }
    
    /// Route message to ComponentActor mailbox by ComponentId.
    async fn route_message(&self, envelope: MessageEnvelope<ComponentMessage>) -> Result<(), WasmError> {
        // Extract target ComponentId from message
        let target_id = match &envelope.payload {
            ComponentMessage::InterComponent { to, .. } => to.clone(),
            _ => return Ok(()), // Skip non-inter-component messages
        };
        
        // Resolve actor address by ComponentId
        let actor_addr = self.resolve_actor(&target_id)
            .ok_or_else(|| WasmError::Runtime(format!("Component not found: {:?}", target_id)))?;
        
        // Send to mailbox
        actor_addr.send(envelope.payload).await
            .map_err(|e| WasmError::Runtime(format!("Mailbox send failed: {:?}", e)))?;
        
        // Update stats
        self.stats.write().await.messages_routed += 1;
        
        Ok(())
    }
    
    /// Resolve ComponentId to actor address.
    fn resolve_actor(&self, component_id: &ComponentId) -> Option<ActorAddress> {
        self.actor_registry.read().await.get(component_id).cloned()
    }
}
```

**Success Criteria:**
- Subscriber successfully subscribes to broker
- Routing loop processes messages by ComponentId
- Messages delivered to correct ComponentActor mailboxes
- Direct addressing works (no topic routing)

#### Step 2.2: Integrate with ComponentRegistry (1.5 hours)

**File:** `airssys-wasm/src/actor/component/component_registry.rs`

**Tasks:**
1. Add ActorSystemSubscriber to ComponentRegistry
2. Initialize subscriber when registry is created
3. Register ComponentActor addresses with subscriber
4. Start subscriber routing loop in background task

**Code Addition:**
```rust
pub struct ComponentRegistry {
    // ... existing fields
    actor_system_subscriber: Arc<ActorSystemSubscriber>,
}

impl ComponentRegistry {
    pub fn new(messaging_service: Arc<MessagingService>) -> Self {
        let subscriber = Arc::new(ActorSystemSubscriber::new(
            messaging_service.broker(),
        ));
        
        // Start routing loop in background
        let subscriber_clone = Arc::clone(&subscriber);
        tokio::spawn(async move {
            if let Err(e) = subscriber_clone.start().await {
                log::error!("ActorSystemSubscriber failed: {:?}", e);
            }
        });
        
        Self {
            // ... existing fields
            actor_system_subscriber: subscriber,
        }
    }
    
    pub async fn register_component(
        &mut self,
        component_id: ComponentId,
        actor_addr: ActorAddress,
    ) -> Result<(), WasmError> {
        // Register with ActorSystemSubscriber
        self.actor_system_subscriber
            .register_actor(component_id.clone(), actor_addr.clone())
            .await?;
        
        // ... existing registration logic
        Ok(())
    }
}
```

**Success Criteria:**
- ActorSystemSubscriber initialized with registry
- ComponentActor addresses registered correctly
- Routing loop running in background

#### Step 2.3: Add Routing Statistics (1 hour)

**File:** `airssys-wasm/src/actor/message/routing_stats.rs`

**Tasks:**
1. Create `RoutingStats` struct
2. Track messages routed, failures, latency
3. Add metrics export for monitoring
4. Implement `Default` trait

**Code:**
```rust
//! Routing statistics for ActorSystemSubscriber.

use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Deserialize, Serialize};

/// Routing statistics.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RoutingStats {
    pub messages_routed: AtomicU64,
    pub routing_failures: AtomicU64,
    pub components_not_found: AtomicU64,
    pub mailbox_send_failures: AtomicU64,
}

impl RoutingStats {
    pub fn snapshot(&self) -> RoutingStatsSnapshot {
        RoutingStatsSnapshot {
            messages_routed: self.messages_routed.load(Ordering::Relaxed),
            routing_failures: self.routing_failures.load(Ordering::Relaxed),
            components_not_found: self.components_not_found.load(Ordering::Relaxed),
            mailbox_send_failures: self.mailbox_send_failures.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoutingStatsSnapshot {
    pub messages_routed: u64,
    pub routing_failures: u64,
    pub components_not_found: u64,
    pub mailbox_send_failures: u64,
}
```

**Success Criteria:**
- Statistics tracked correctly
- Metrics exportable for monitoring

#### Step 2.4: Update actor/message/mod.rs (0.5 hours)

**File:** `airssys-wasm/src/actor/message/mod.rs`

**Tasks:**
1. Declare new submodules
2. Re-export public types
3. Update module documentation

**Success Criteria:**
- All types accessible

---

### Phase 3: Integration and API Finalization (2 hours)

#### Step 3.1: Integrate MessagingService with WasmRuntime (2 hours)

**File:** `airssys-wasm/src/runtime/engine.rs` (or appropriate file)

**Tasks:**
1. Add MessagingService field to WasmRuntime/WasmEngine
2. Initialize MessagingService during engine creation
3. Expose MessagingService via public getter
4. Update documentation

**Code:**
```rust
pub struct WasmEngine {
    // ... existing fields
    messaging_service: Arc<MessagingService>,
}

impl WasmEngine {
    pub fn new() -> Result<Self, WasmError> {
        let messaging_service = Arc::new(MessagingService::new());
        
        Ok(Self {
            // ... existing initialization
            messaging_service,
        })
    }
    
    pub fn messaging_service(&self) -> Arc<MessagingService> {
        Arc::clone(&self.messaging_service)
    }
}
```

**Success Criteria:**
- MessagingService accessible from WasmEngine
- Engine initialization succeeds
- ActorSystemSubscriber properly integrated

---

### Phase 4: Testing (2 hours)

#### Step 4.1: Unit Tests for MessagingService (0.5 hours)

**File:** `airssys-wasm/tests/messaging_broker_tests.rs`

**Test Cases:**
1. Test broker initialization
2. Test broker access
3. Test metrics tracking

**Test Template:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_broker_initialization() {
        let service = MessagingService::new();
        assert!(service.broker().subscriber_count().await == 0);
    }
    
    #[tokio::test]
    async fn test_broker_access() {
        let service = MessagingService::new();
        let broker = service.broker();
        
        // Verify broker is accessible
        assert!(Arc::strong_count(&broker) >= 1);
    }
}
```

**Success Criteria:**
- All unit tests pass
- Code coverage >95% for MessagingService

#### Step 4.2: Integration Tests for ActorSystemSubscriber (1 hour)

**File:** `airssys-wasm/tests/actor_system_subscriber_tests.rs`

**Test Cases:**
1. Test subscriber initialization
2. Test message routing to mailboxes by ComponentId
3. Test ComponentId resolution
4. Test routing statistics
5. Test error handling (component not found)

**Test Template:**
```rust
#[tokio::test]
async fn test_message_routing_to_mailbox_by_component_id() {
    let messaging_service = Arc::new(MessagingService::new());
    let registry = ComponentRegistry::new(messaging_service.clone());
    
    // Register component
    let component_id = ComponentId::new("test-component");
    let (tx, mut rx) = mpsc::unbounded_channel();
    let actor_addr = ActorAddress::new(tx);
    
    registry.register_component(component_id.clone(), actor_addr).await.unwrap();
    
    // Publish message with direct ComponentId addressing
    let message = ComponentMessage::InterComponent {
        from: ComponentId::new("sender"),
        to: component_id.clone(),  // Direct addressing
        data: vec![1, 2, 3],
    };
    
    messaging_service.broker().publish(MessageEnvelope::new(message)).await.unwrap();
    
    // Verify message received
    let received = rx.recv().await.unwrap();
    assert!(matches!(received, ComponentMessage::InterComponent { .. }));
}
```

**Success Criteria:**
- All integration tests pass
- Message delivery validated end-to-end with ComponentId addressing

#### Step 4.3: Benchmarks (0.5 hours)

**File:** `airssys-wasm/benches/message_routing_benchmarks.rs`

**Benchmark Cases:**
1. Broker routing latency
2. ActorSystem subscriber overhead (ComponentId lookup)
3. Total routing latency (end-to-end)
4. Throughput (messages/sec)

**Benchmark Template:**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_broker_routing(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let broker = Arc::new(InMemoryMessageBroker::new());
    
    c.bench_function("broker_publish", |b| {
        b.to_async(&rt).iter(|| async {
            let message = ComponentMessage::InterComponent {
                from: ComponentId::new("sender"),
                to: ComponentId::new("receiver"),  // Direct addressing
                data: vec![],
            };
            broker.publish(MessageEnvelope::new(black_box(message))).await.unwrap();
        });
    });
}

criterion_group!(benches, bench_broker_routing);
criterion_main!(benches);
```

**Success Criteria:**
- Routing latency ≤220ns
- Throughput ≥4.7M msg/sec (broker capacity)

---

## Code Structure

### New Files

```
airssys-wasm/
├── src/
│   ├── runtime/
│   │   ├── messaging.rs              (~300 lines) - MessagingService (simplified)
│   │   └── mod.rs                    (updated)    - Module exports
│   └── actor/
│       ├── message/
│       │   ├── actor_system_subscriber.rs (~250 lines) - ActorSystemSubscriber
│       │   ├── routing_stats.rs           (~80 lines) - Routing statistics
│       │   └── mod.rs                     (updated)  - Module exports
│       └── component/
│           └── component_actor.rs         (updated)  - ComponentMessage with direct addressing
├── tests/
│   ├── messaging_broker_tests.rs     (~40 lines) - MessagingService tests (simplified)
│   └── actor_system_subscriber_tests.rs (~120 lines) - Integration tests
└── benches/
    └── message_routing_benchmarks.rs (~120 lines) - Performance benchmarks
```

**Note:** Removed `topics.rs` and `topic_routing_tests.rs` (not needed in Phase 1)

### Modified Files

```
airssys-wasm/
├── src/
│   ├── runtime/
│   │   ├── engine.rs                 (+50 lines) - Add MessagingService field
│   │   └── mod.rs                    (+5 lines) - Export messaging module
│   └── actor/
│       ├── component/
│       │   ├── component_actor.rs    (+20 lines) - Update ComponentMessage enum
│       │   └── component_registry.rs (+100 lines) - Integrate ActorSystemSubscriber
│       └── message/
│           └── mod.rs                (+20 lines) - Export new modules
└── Cargo.toml                        (updated) - Add dependencies if needed
```

---

## Testing Strategy

### Test Pyramid

```text
         ┌─────────────────┐
         │  Benchmarks     │ ← Performance validation
         │  (~120 lines)   │
         └─────────────────┘
       ┌───────────────────────┐
       │  Integration Tests    │ ← End-to-end message routing
       │  (~120 lines)         │
       └───────────────────────┘
  ┌────────────────────────────────┐
  │     Unit Tests                 │ ← Individual component testing
  │     (~180 lines)               │
  └────────────────────────────────┘
```

### Test Coverage Goals

- **Unit Tests**: >95% code coverage for new modules
- **Integration Tests**: 100% API surface coverage
- **Benchmarks**: Validate all performance targets

### Test Execution

```bash
# Run all tests
cargo test --package airssys-wasm

# Run specific test module
cargo test --test messaging_broker_tests

# Run benchmarks
cargo bench --bench message_routing_benchmarks

# Check coverage (with tarpaulin)
cargo tarpaulin --out Html --output-dir coverage
```

---

## Performance Validation

### Baseline Metrics (airssys-rt)

From RT-TASK-008 and Block 3 measurements:
- **MessageBroker routing**: 211ns
- **Mailbox delivery**: 100ns
- **Actor spawn**: 625ns

### Target Metrics (Task 1.1)

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Broker routing | ≤211ns | Criterion benchmark |
| ActorSystem overhead | ≤9ns | Criterion benchmark (component_id lookup + mailbox send) |
| Total routing | ≤220ns | End-to-end benchmark |
| Throughput | ≥4.7M msg/sec | Broker capacity test |
| Memory overhead | <50KB per component | Memory profiling |

### Benchmark Suite

**bench_broker_routing:**
```rust
// Measure pure MessageBroker.publish() latency
// Target: ≤211ns (proven baseline)
```

**bench_actor_system_overhead:**
```rust
// Measure component_id → address lookup + mailbox send
// Target: ≤9ns
```

**bench_end_to_end_routing:**
```rust
// Measure total: publish → broker → subscriber → mailbox
// Target: ≤220ns
```

**bench_throughput:**
```rust
// Measure messages/sec capacity
// Target: ≥4.7M msg/sec
```

### Performance Regression Detection

```bash
# Baseline measurement
cargo bench --bench message_routing_benchmarks -- --save-baseline baseline

# After changes, compare
cargo bench --bench message_routing_benchmarks -- --baseline baseline
```

---

## Quality Gates

### Pre-Merge Checklist

- [ ] All unit tests pass (100% pass rate)
- [ ] All integration tests pass
- [ ] Benchmarks meet performance targets (≤220ns routing)
- [ ] Code coverage >95% for new modules
- [ ] All public APIs documented with rustdoc
- [ ] No clippy warnings (`cargo clippy --all-targets`)
- [ ] No format violations (`cargo fmt --check`)
- [ ] No unsafe code (or justified with SAFETY comments)
- [ ] Error handling comprehensive (no unwrap/expect in prod code)

### Documentation Requirements

- [ ] Module-level documentation (`//!`) for all modules
- [ ] Function-level documentation (`///`) for all public APIs
- [ ] Examples in rustdoc for common usage patterns
- [ ] Architecture diagrams in module docs
- [ ] References to ADRs and knowledge docs

### Review Criteria

- [ ] Code follows Microsoft Rust Guidelines (@[.aiassisted/guidelines/rust/microsoft-rust-guidelines.md])
- [ ] Follows workspace standards (@[PROJECTS_STANDARD.md])
- [ ] Memory safety validated (no data races, no leaks)
- [ ] Error propagation uses proper Result types
- [ ] Logging appropriate (tracing events at correct levels)

---

## Integration Checklist

### Dependency Verification

- [x] airssys-rt MessageBroker available
- [x] ComponentActor with Actor trait implemented
- [x] ActorSystem mailbox infrastructure
- [x] ComponentMessage enum defined

### API Compatibility

- [ ] ComponentMessage enum extended without breaking changes
- [ ] New APIs follow existing naming conventions
- [ ] Error types consistent with existing WasmError
- [ ] Module organization follows existing patterns

### Backward Compatibility

- [ ] No breaking changes to existing ComponentActor API
- [ ] Existing tests still pass
- [ ] Performance regression tests pass

### Future-Proofing

- [ ] API surface extensible for Phase 2 (fire-and-forget, request-response)
- [ ] Topic routing supports future wildcard patterns
- [ ] MessageBroker integration supports future distributed scenarios

---

## Risk Assessment

### Risk 1: Performance Not Meeting Targets

**Impact:** High - Slow routing defeats purpose of inter-component communication  
**Probability:** Low - Building on proven 211ns MessageBroker baseline  
**Mitigation:**
- Use airssys-rt InMemoryMessageBroker directly (no wrapper overhead)
- Minimize allocations in routing path
- Profile extensively with criterion
- Optimize hot paths based on benchmark data

**Contingency:**
- If >220ns, profile and optimize bottlenecks
- Consider zero-copy optimizations
- Evaluate DashMap for actor registry if HashMap is bottleneck

### Risk 2: ActorSystemSubscriber Routing Failures

**Impact:** High - Messages not delivered breaks communication  
**Probability:** Medium - Complex routing logic with multiple failure modes  
**Mitigation:**
- Comprehensive error handling
- Dead letter queue for undeliverable messages (future)
- Extensive integration tests
- Monitoring and alerting for routing failures

**Contingency:**
- Add retry logic for transient failures
- Implement circuit breaker for repeated failures
- Add detailed logging for debugging

### Risk 3: Topic Pattern Matching Bugs

**Impact:** Medium - Messages routed to wrong components  
**Probability:** Low - Simple wildcard logic  
**Mitigation:**
- Exhaustive unit tests for pattern matching
- Property-based testing (proptest) for edge cases
- Clear documentation of pattern syntax

**Contingency:**
- Fall back to exact topic matching if wildcards problematic
- Add validation for topic patterns at subscription time

### Risk 4: Memory Leaks in Subscription Registry

**Impact:** Medium - Long-running systems accumulate subscriptions  
**Probability:** Low - Using Arc<RwLock<T>> with proper cleanup  
**Mitigation:**
- Ensure unsubscribe called during component shutdown
- Add cleanup logic in ComponentActor Drop implementation
- Memory profiling tests

**Contingency:**
- Add periodic cleanup of stale subscriptions
- Implement subscription expiry/refresh mechanism

---

## References

### ADRs

- **ADR-WASM-009**: Component Communication Model
  - Location: `.memory-bank/sub-projects/airssys-wasm/docs/adr/adr-wasm-009-component-communication-model.md`
  - Key Decisions: Message-passing via MessageBroker, ActorSystem subscriber pattern

- **ADR-WASM-006**: Component Isolation and Sandboxing
  - Actor-based approach for component hosting
  - Child trait for lifecycle management

- **ADR-WASM-005**: Capability-Based Security Model
  - Security enforcement for message permissions (Phase 2)

### Knowledge Documentation

- **KNOWLEDGE-WASM-024**: Component Messaging Clarifications
  - Location: `.memory-bank/sub-projects/airssys-wasm/docs/knowledges/knowledge-wasm-024-component-messaging-clarifications.md`
  - **Critical clarifications for Phase 1 implementation**
  - Direct ComponentId addressing (no topics in Phase 1)
  - ActorSystem subscription IS the event-driven subscription (runtime-level)
  - Two async patterns: fire-and-forget and request-response
  - Components NEVER subscribe manually

- **KNOWLEDGE-WASM-005**: Inter-Component Messaging Architecture
  - Location: `.memory-bank/sub-projects/airssys-wasm/docs/knowledges/knowledge-wasm-005-messaging-architecture.md`
  - Complete messaging specification and patterns
  - Push-based delivery (no polling)
  - Actor model alignment

- **KNOWLEDGE-RT-013**: Actor Performance Benchmarking (airssys-rt)
  - MessageBroker 211ns routing baseline
  - Actor spawn 625ns baseline

### airssys-rt References

- **RT-TASK-008**: Message Broker Performance Baseline
  - 211ns routing, 4.7M msg/sec throughput

- **MessageBroker Implementation**:
  - File: `airssys-rt/src/broker/in_memory.rs`
  - Pure pub-sub architecture with no actor registry

- **MessageBroker Trait**:
  - File: `airssys-rt/src/broker/traits.rs`
  - Generic MessageBroker<M> trait definition

### Project Standards

- **Microsoft Rust Guidelines**: `.aiassisted/guidelines/rust/microsoft-rust-guidelines.md`
  - API design principles
  - Error handling patterns
  - Documentation standards

- **Workspace Standards**: `PROJECTS_STANDARD.md`
  - Module organization (§4.3)
  - Import layering (§2.1)
  - Documentation requirements

### External References

- [Erlang/OTP gen_server](https://www.erlang.org/doc/man/gen_server.html) - Actor messaging patterns
- [Tokio Documentation](https://docs.rs/tokio) - Async runtime primitives
- [Criterion.rs](https://bheisler.github.io/criterion.rs) - Benchmarking framework

---

## Next Steps After Task 1.1 Completion

**Important Clarifications (KNOWLEDGE-WASM-024):**

1. **Task 1.2**: ComponentActor Message Reception
   - Implement mailbox integration
   - Handle message delivery to WASM via `handle-message` export
   - Backpressure management
   - **Note:** Push-based delivery (no polling)

2. **Task 1.3**: Internal Event Subscription Infrastructure
   - **THIS IS INTERNAL INFRASTRUCTURE**, not component-facing API
   - ActorSystem → MessageBroker event stream subscription
   - ComponentId-based routing (direct addressing)
   - **NOT topic-based pub-sub for components**
   - **Optional future enhancement:** Topic patterns could be Phase 2+

3. **Phase 2**: Fire-and-Forget Messaging
   - `send-message` host function (component-facing API)
   - `handle-message` component export
   - Multicodec serialization
   - **Note:** Uses ComponentId addressing from Task 1.1

4. **Phase 3**: Request-Response Pattern
   - `send-request` host function
   - Response correlation (automatic by runtime)
   - Timeout enforcement (automatic by runtime)
   - **Note:** Async RPC via callbacks, not blocking

---

## Approval

**Plan Status:** Ready for Implementation  
**Reviewer:** [To be assigned]  
**Approved By:** [To be filled]  
**Approval Date:** [To be filled]

**Implementation Start Date:** [To be filled]  
**Target Completion Date:** [Start Date + 2 days]

---

**End of Implementation Plan**
