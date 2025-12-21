# Message Delivery Architecture - Final Decision

**Document Type:** Knowledge Documentation  
**Document ID:** KNOWLEDGE-WASM-026  
**Created:** 2025-12-21  
**Status:** Active Reference (SUPERSEDES KNOWLEDGE-WASM-025)  
**Priority:** Critical - Foundation for Block 5 Message Delivery  
**Related:** ADR-WASM-020, ADR-WASM-009, KNOWLEDGE-WASM-024, KNOWLEDGE-WASM-018, WASM-TASK-006

---

## ⚠️ This Document Supersedes KNOWLEDGE-WASM-025

**KNOWLEDGE-WASM-025** proposed extending `ComponentRegistry` to store `MailboxSender`. After architectural review, this was **rejected** in favor of the design documented here.

**Key Change:** `ActorSystemSubscriber` owns message delivery, NOT `ComponentRegistry`.

---

## Overview

This document captures the **final architectural decision** for message delivery in airssys-wasm. It documents the complete message flow from when a WASM component sends a message to when the target component's `handle_message` or `handle_callback` export is invoked.

### Core Principle

> **`ComponentRegistry` stays pure (identity lookup only).**  
> **`ActorSystemSubscriber` owns message delivery (has `MailboxSender` references).**

---

## The Problem: Stubbed Message Delivery

### Current State (BROKEN)

`ActorSystemSubscriber::route_message_to_subscribers()` is **stubbed**:

```rust
// Location: airssys-wasm/src/actor/message/actor_system_subscriber.rs:272-290
async fn route_message_to_subscribers(
    _registry: &ComponentRegistry,        // UNUSED
    _subscriber_manager: &SubscriberManager,  // UNUSED
    envelope: MessageEnvelope<ComponentMessage>,
) -> Result<(), WasmError> {
    // 1. Extract target - ✅ Works
    let _target = Self::extract_target(&envelope.payload)?;

    // 2. STUBBED - No actual delivery! ❌
    tracing::debug!("Message routed through ActorSystemSubscriber");
    Ok(())  // Returns success without doing anything!
}
```

### Root Cause

`ActorAddress` is an **identifier**, not a **sender**. It provides no `send()` method:

```rust
// From airssys-rt/src/util/ids.rs
pub enum ActorAddress {
    Named { id: ActorId, name: String },
    Anonymous { id: ActorId },
}

impl ActorAddress {
    pub fn named(name: impl Into<String>) -> Self { ... }
    pub fn id(&self) -> &ActorId { ... }
    // NO send() method!
}
```

---

## Architectural Decision

### Rejected: Extend ComponentRegistry (KNOWLEDGE-WASM-025)

**Proposal:** Store `MailboxSender` alongside `ActorAddress` in `ComponentRegistry`.

**Why Rejected:**
1. **Violates Single Responsibility:** ComponentRegistry should be pure identity lookup
2. **Mixing Concerns:** Addressing + delivery in one component
3. **ADR-WASM-009 Violation:** ADR defines ComponentRegistry for identity, not delivery
4. **Architectural Debt:** Would require later refactoring

### Accepted: ActorSystemSubscriber Owns Delivery

**Design:** `ActorSystemSubscriber` maintains a `mailbox_senders` map for actual delivery.

**Why Accepted:**
1. **Clean Separation:** Registry = identity, Subscriber = delivery
2. **Follows ADR-WASM-009:** ActorSystem subscriber pattern for routing
3. **Runtime Responsibility:** Delivery is a runtime concern, not registry concern
4. **Architectural Clarity:** Each component has single responsibility

---

## Complete Message Flow

### Step-by-Step Flow

```
┌────────────────────────────────────────────────────────────────────────────┐
│  STEP 1: WASM Component A calls send-message(target, payload)              │
│                                                                            │
│  ↓ WIT host function call                                                  │
│                                                                            │
│  STEP 2: Host Function Handler                                             │
│  • Validates sender capabilities                                           │
│  • Rate limiting check                                                     │
│  • Payload size validation                                                 │
│  • Creates ComponentMessage::InterComponent { sender, to, payload }        │
│                                                                            │
│  ↓                                                                         │
│                                                                            │
│  STEP 3: ComponentActor.publish_message()                                  │
│  • Uses MessageBrokerBridge.publish(topic, message)                        │
│                                                                            │
│  ↓                                                                         │
│                                                                            │
│  STEP 4: MessageBrokerWrapper (Layer 2)                                    │
│  • Wraps in MessageEnvelope                                                │
│  • Delegates to Layer 3 MessageBroker                                      │
│                                                                            │
│  ↓                                                                         │
│                                                                            │
│  STEP 5: MessageBroker (airssys-rt Layer 3)                                │
│  • InMemoryMessageBroker.publish(envelope)                                 │
│  • Broadcasts to all subscribers (~211ns)                                  │
│                                                                            │
│  ↓                                                                         │
│                                                                            │
│  STEP 6: ActorSystemSubscriber (PRIMARY SUBSCRIBER)                        │
│  • Receives ALL messages from broker                                       │
│  • route_message_to_subscribers() extracts target                          │
│  • Looks up MailboxSender in mailbox_senders HashMap ← 🆕 NEW              │
│  • Calls sender.send(message) ← 🆕 ACTUAL DELIVERY                         │
│                                                                            │
│  ↓                                                                         │
│                                                                            │
│  STEP 7: Target ComponentActor Mailbox                                     │
│  • Message arrives in component's mailbox receiver                         │
│  • Actor's message processing loop receives the message                    │
│                                                                            │
│  ↓                                                                         │
│                                                                            │
│  STEP 8: Message Processing                                                │
│  • Match on ComponentMessage type                                          │
│  • InterComponent → invoke_handle_message_with_timeout()                   │
│  • InterComponentWithCorrelation → handle_callback or handle_message       │
│                                                                            │
│  ↓                                                                         │
│                                                                            │
│  STEP 9: WASM Boundary Crossing                                            │
│  • Get WasmRuntime from ComponentActor                                     │
│  • Get cached export function (handle-message or handle-callback)          │
│  • Wrap with tokio::time::timeout(delivery_timeout_ms)                     │
│  • Call handle_fn.call_async(&mut store, params, &mut results)             │
│                                                                            │
│  ↓                                                                         │
│                                                                            │
│  STEP 10: WASM Component B processes message                               │
│  • handle-message(sender, message) OR                                      │
│  • handle-callback(request-id, result)                                     │
└────────────────────────────────────────────────────────────────────────────┘
```

### What's Currently Working vs. Broken

| Step | Status | Notes |
|------|--------|-------|
| 1. WASM calls host function | ✅ Working | send-message WIT interface |
| 2. Host validates & creates message | ✅ Working | Security checks in place |
| 3. publish_message() | ✅ Working | MessageBrokerBridge |
| 4. MessageBrokerWrapper | ✅ Working | Envelope wrapping |
| 5. MessageBroker broadcast | ✅ Working | ~211ns routing |
| 6. ActorSystemSubscriber | ❌ STUBBED | No actual delivery |
| 7. Target mailbox | ❌ Never happens | No sender registered |
| 8. Message processing | ❌ Never happens | No message received |
| 9. WASM boundary | ❌ Never happens | handle-message never called |
| 10. Component B | ❌ Never happens | Message never arrives |

---

## Implementation Design

### Modified ActorSystemSubscriber

```rust
pub struct ActorSystemSubscriber<B: MessageBroker<ComponentMessage>> {
    /// MessageBroker for receiving messages
    broker: Arc<B>,
    
    /// ComponentRegistry for looking up component addresses (IDENTITY ONLY)
    registry: ComponentRegistry,
    
    /// SubscriberManager for topic-based routing decisions
    subscriber_manager: Arc<SubscriberManager>,
    
    /// Background routing task handle
    routing_task: Option<JoinHandle<()>>,
    
    /// 🆕 NEW: Map of ComponentId → MailboxSender for actual delivery
    mailbox_senders: Arc<RwLock<HashMap<ComponentId, MailboxSender<ComponentMessage>>>>,
}
```

### New Registration Methods

```rust
impl<B: MessageBroker<ComponentMessage>> ActorSystemSubscriber<B> {
    /// Register a component's mailbox sender for message delivery.
    /// Called by ComponentSpawner when ComponentActor is spawned.
    pub async fn register_mailbox(
        &self,
        component_id: ComponentId,
        sender: MailboxSender<ComponentMessage>,
    ) {
        let mut senders = self.mailbox_senders.write().await;
        senders.insert(component_id, sender);
        tracing::debug!(
            component_id = %component_id.as_str(),
            "Registered mailbox sender for message delivery"
        );
    }
    
    /// Unregister a component's mailbox sender.
    /// Called when ComponentActor is stopped.
    pub async fn unregister_mailbox(&self, component_id: &ComponentId) {
        let mut senders = self.mailbox_senders.write().await;
        senders.remove(component_id);
        tracing::debug!(
            component_id = %component_id.as_str(),
            "Unregistered mailbox sender"
        );
    }
}
```

### Fixed route_message_to_subscribers

```rust
async fn route_message_to_subscribers(
    mailbox_senders: &RwLock<HashMap<ComponentId, MailboxSender<ComponentMessage>>>,
    _subscriber_manager: &SubscriberManager,
    envelope: MessageEnvelope<ComponentMessage>,
) -> Result<(), WasmError> {
    // 1. Extract target ComponentId
    let target = Self::extract_target(&envelope.payload)?;
    
    // 2. Look up MailboxSender for target
    let senders = mailbox_senders.read().await;
    let sender = senders.get(&target).ok_or_else(|| {
        WasmError::component_not_found(format!(
            "No mailbox registered for component: {}",
            target.as_str()
        ))
    })?;
    
    // 3. ACTUAL DELIVERY!
    sender.send(envelope.payload).await.map_err(|e| {
        WasmError::messaging_error(format!(
            "Failed to deliver message to {}: {}",
            target.as_str(), e
        ))
    })?;
    
    tracing::debug!(
        target = %target.as_str(),
        "Message delivered to component mailbox"
    );
    
    Ok(())
}
```

### ComponentSpawner Integration

```rust
// In ComponentSpawner::spawn() or equivalent:

pub async fn spawn_component(
    &self,
    component_id: ComponentId,
    // ...
) -> Result<ActorAddress, WasmError> {
    // 1. Create mailbox channel
    let (mailbox_tx, mailbox_rx) = tokio::sync::mpsc::unbounded_channel();
    
    // 2. ComponentActor keeps the receiver
    let mut component_actor = ComponentActor::new(/* ... */);
    component_actor.set_mailbox_receiver(mailbox_rx);
    
    // 3. Register address with ComponentRegistry (identity only)
    let actor_address = ActorAddress::named(component_id.as_str());
    self.registry.register(component_id.clone(), actor_address.clone())?;
    
    // 4. Register sender with ActorSystemSubscriber (for delivery)
    self.actor_system_subscriber.register_mailbox(
        component_id.clone(),
        mailbox_tx,
    ).await;
    
    // 5. Start the component actor...
    
    Ok(actor_address)
}

pub async fn stop_component(&self, component_id: &ComponentId) -> Result<(), WasmError> {
    // 1. Unregister from ActorSystemSubscriber
    self.actor_system_subscriber.unregister_mailbox(component_id).await;
    
    // 2. Unregister from ComponentRegistry
    self.registry.unregister(component_id)?;
    
    // 3. Stop the component actor...
    
    Ok(())
}
```

---

## Responsibility Matrix

| Component | Responsibility | Stores | Does NOT Store |
|-----------|----------------|--------|----------------|
| **ComponentRegistry** | Identity lookup (pure) | `ComponentId → ActorAddress` | ❌ MailboxSender |
| **ActorSystemSubscriber** | Message delivery | `ComponentId → MailboxSender` | ❌ ActorAddress |
| **ComponentActor** | Message processing | `MailboxReceiver`, WASM instance | - |
| **MessageBroker** | Pub/sub broadcast | Internal queue | ❌ Component-specific data |
| **SubscriberManager** | Topic filtering | `Topic → Vec<ComponentId>` | - |

### Key Principle

> **Single Responsibility per Component:**
> - Registry = "Who exists and their address"
> - Subscriber = "How to deliver messages"
> - Actor = "How to process messages"

---

## WIT Interface Reference

### Component Exports (what WASM implements)

```wit
// wit/core/component-lifecycle.wit

/// Handle inter-component message
handle-message: func(
    sender: component-id,
    message: list<u8>
) -> result<_, component-error>;

/// Handle async callback
handle-callback: func(
    request-id: request-id,
    callback-result: result<list<u8>, string>
) -> result<_, component-error>;
```

### Host Imports (what host provides)

```wit
// wit/core/host-services.wit

/// Send fire-and-forget message
send-message: func(
    target: component-id,
    message: list<u8>
) -> result<_, messaging-error>;

/// Send request with callback
send-request: func(
    target: component-id,
    request: list<u8>,
    timeout-ms: u64
) -> result<request-id, messaging-error>;
```

---

## Performance Characteristics

| Operation | Latency | Notes |
|-----------|---------|-------|
| Host function call | ~100ns | WASM → Rust boundary |
| Security validation | <5μs | Capability + rate limit checks |
| MessageBroker routing | ~211ns | Proven airssys-rt baseline |
| ActorSystemSubscriber lookup | <100ns | HashMap read |
| MailboxSender.send() | ~100ns | Tokio unbounded channel |
| Mailbox → Actor processing | <1μs | Channel receive |
| WASM export invocation | ~10μs | handle-message call |
| **Total end-to-end** | **<20μs** | Within Block 5 targets |

---

## Testing Requirements

### Unit Tests (in ActorSystemSubscriber module)

```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_register_mailbox() {
        let subscriber = ActorSystemSubscriber::new(/*...*/);
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        let component_id = ComponentId::new("test");
        
        subscriber.register_mailbox(component_id.clone(), tx).await;
        
        // Verify registered
        let senders = subscriber.mailbox_senders.read().await;
        assert!(senders.contains_key(&component_id));
    }
    
    #[tokio::test]
    async fn test_message_actually_delivered() {
        let subscriber = ActorSystemSubscriber::new(/*...*/);
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let target = ComponentId::new("receiver");
        
        subscriber.register_mailbox(target.clone(), tx).await;
        
        let message = ComponentMessage::InterComponent {
            sender: ComponentId::new("sender"),
            to: target.clone(),
            payload: vec![1, 2, 3],
        };
        let envelope = MessageEnvelope::new(message);
        
        // Route message
        ActorSystemSubscriber::route_message_to_subscribers(
            &subscriber.mailbox_senders,
            &subscriber.subscriber_manager,
            envelope,
        ).await.unwrap();
        
        // Verify received
        let received = rx.try_recv().unwrap();
        assert!(matches!(received, ComponentMessage::InterComponent { .. }));
    }
    
    #[tokio::test]
    async fn test_send_to_unregistered_fails() {
        let subscriber = ActorSystemSubscriber::new(/*...*/);
        let target = ComponentId::new("nonexistent");
        
        let message = ComponentMessage::InterComponent {
            sender: ComponentId::new("sender"),
            to: target.clone(),
            payload: vec![],
        };
        let envelope = MessageEnvelope::new(message);
        
        let result = ActorSystemSubscriber::route_message_to_subscribers(
            &subscriber.mailbox_senders,
            &subscriber.subscriber_manager,
            envelope,
        ).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
```

### Integration Tests (in tests/ directory)

```rust
// tests/message_delivery_integration_tests.rs

#[tokio::test]
async fn test_end_to_end_message_delivery() {
    // 1. Create full runtime stack
    // 2. Spawn two components (A and B)
    // 3. Component A sends message to Component B via host function
    // 4. Verify Component B's handle-message was invoked
    // 5. Verify message payload matches
}

#[tokio::test]
async fn test_request_response_pattern() {
    // 1. Create full runtime stack
    // 2. Spawn two components
    // 3. Component A sends request to Component B
    // 4. Component B sends response
    // 5. Component A's handle-callback is invoked
    // 6. Verify correlation ID matches
}
```

---

## Implementation Checklist

- [ ] **Step 1:** Add `mailbox_senders` field to `ActorSystemSubscriber`
- [ ] **Step 2:** Add `register_mailbox()` method
- [ ] **Step 3:** Add `unregister_mailbox()` method
- [ ] **Step 4:** Fix `route_message_to_subscribers()` to use `mailbox_senders`
- [ ] **Step 5:** Update `ActorSystemSubscriber::new()` to initialize map
- [ ] **Step 6:** Update ComponentSpawner to call `register_mailbox()`
- [ ] **Step 7:** Update component shutdown to call `unregister_mailbox()`
- [ ] **Step 8:** Add unit tests for registration/unregistration
- [ ] **Step 9:** Add unit tests for message delivery
- [ ] **Step 10:** Add integration tests for end-to-end flow
- [ ] **Step 11:** Update remediation plan document
- [ ] **Step 12:** Deprecate KNOWLEDGE-WASM-025

---

## References

### ADRs
- **ADR-WASM-009:** Component Communication Model (ActorSystem subscriber pattern)
- **ADR-WASM-018:** Three-Layer Architecture

### Knowledge Documents
- **KNOWLEDGE-WASM-024:** Component Messaging Clarifications
- **KNOWLEDGE-WASM-018:** Component Definitions and Architecture Layers
- **KNOWLEDGE-WASM-005:** Inter-Component Messaging Architecture
- **KNOWLEDGE-WASM-025:** (SUPERSEDED by this document)

### Task Documentation
- **WASM-TASK-006:** Block 5 - Inter-Component Communication
- **task-006-phase-1-task-1.1-remediation-plan.md:** Implementation plan

### Source Files
- `airssys-wasm/src/actor/message/actor_system_subscriber.rs` - Main file to modify
- `airssys-wasm/src/actor/component/component_actor.rs` - ComponentActor with mailbox
- `airssys-wasm/src/actor/component/component_registry.rs` - Stays unchanged (pure)
- `airssys-wasm/wit/core/component-lifecycle.wit` - handle-message/handle-callback
- `airssys-wasm/wit/core/host-services.wit` - send-message/send-request

---

## Version History

| Date | Version | Changes |
|------|---------|---------|
| 2025-12-21 | 1.0 | Initial document - Final architecture decision |

---

**End of KNOWLEDGE-WASM-026**
