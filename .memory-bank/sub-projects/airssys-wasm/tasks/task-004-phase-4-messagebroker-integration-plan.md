# WASM-TASK-004 Phase 4: MessageBroker Integration - Implementation Plan

**Date:** 2025-12-15  
**Status:** Ready for Implementation  
**Priority:** CRITICAL - Foundation for inter-component communication (Block 3 Phase 4)  
**Estimated Effort:** 12-16 hours (Task 4.1: 4-5h, Task 4.2: 4-5h, Task 4.3: 4-6h)  
**Quality Target:** 9.5/10 (Match Phase 2-3 quality)  
**Dependencies:** Phase 3 Complete ✅ (Tasks 3.1-3.3)

---

## Executive Summary

This plan details the integration of ComponentActor with airssys-rt MessageBroker for event-driven inter-component communication in WASM-TASK-004 Phase 4. This phase establishes the messaging backbone that will enable Block 5 (Inter-Component Communication) by implementing topic-based pub-sub, message routing, and the ActorSystem as Primary Subscriber pattern.

**Architecture Context:**  
Per **ADR-WASM-009** (Component Communication Model) and **ADR-WASM-018** (Three-Layer Architecture):
- **Layer 1** (WASM Config): Message subscription policies
- **Layer 2** (WASM Lifecycle): ComponentActor message handling
- **Layer 3** (Actor Runtime): MessageBroker pub-sub execution

**Phase 4 Objective:**  
Connect ComponentActor instances to MessageBroker for event-driven communication, establishing the foundation for inter-component messaging in Block 5.

**Key Deliverables:**
1. **Task 4.1:** MessageBrokerBridge trait for Layer 2 ↔ Layer 3 integration
2. **Task 4.2:** Component message publishing and topic-based routing
3. **Task 4.3:** ActorSystem as primary subscriber pattern implementation
4. **Performance:** Event routing overhead <100ns, throughput >4.7M msg/sec
5. **Testing:** 30+ integration tests with full message flow
6. **Documentation:** Comprehensive examples and architecture diagrams

**Success Metrics:**
- MessageBroker routes component messages successfully
- Components can subscribe to topics dynamically
- Topic-based message delivery working
- ActorSystem centralized routing operational
- Routing performance: ~211ns (airssys-rt baseline)
- All 30+ tests passing (target: 749+ total tests)
- Zero warnings (compiler + clippy)
- Code quality: 9.5/10

---

## Phase Completion Context

### Phase 3 Deliverables ✅ COMPLETE (Dec 15, 2025)

**What's Already Built (Phases 1-3):**
- ✅ **Phase 1** (ComponentActor Foundation): Actor + Child traits, message handling, health checks
  - 3,450 lines, 189 tests, 9.5/10 quality
- ✅ **Phase 2** (ActorSystem Integration): ComponentSpawner, ComponentRegistry, MessageRouter
  - 1,656 lines, 145+ tests, 9.5/10 quality
- ✅ **Phase 3** (SupervisorNode Integration): Supervision config, restart logic, health monitoring
  - 3,259 lines, 61 tests, 9.5/10 quality
- ✅ **Total:** 8,365 lines, 395 tests, 719 total lib tests passing, 0 warnings

**Current Architecture (Phase 3):**
```
ComponentSpawner
    ↓ spawn_component()
ComponentActor (implements Actor + Child)
    ↓ supervised by
SupervisorNode (via SupervisorNodeBridge)
    ↓ restart coordination
ComponentSupervisor (restart policy tracking)
    ↓ addressing via
ComponentRegistry (ComponentId → ActorAddress)
    ↓ (NOT YET INTEGRATED)
MessageBroker (airssys-rt) ← PHASE 4 CONNECTS HERE
```

**What Phase 4 Adds:**
Message broker integration for event-driven inter-component communication and pub-sub patterns.

---

## Architecture Review: MessageBroker Integration Layers

### Layer Responsibilities (from ADR-WASM-018 + ADR-WASM-009)

#### Layer 2: WASM Component Lifecycle & Spawning
**Location:** `src/actor/component_actor.rs`, `component_spawner.rs`, `message_router.rs`  
**Ownership:** airssys-wasm  

**OWNS:**
- ✅ ComponentActor - Message handling via Actor trait
- ✅ MessageRouter - Component-level routing logic
- ✅ Topic subscription management (WASM-specific)
- ✅ Message serialization helpers (multicodec integration)

**DOES NOT OWN:**
- ❌ MessageBroker implementation (Layer 3)
- ❌ Pub-sub execution engine (Layer 3)
- ❌ Message delivery guarantees (Layer 3)

#### Layer 3: Actor System Runtime
**Location:** `airssys-rt/src/broker/`  
**Ownership:** airssys-rt  

**OWNS:**
- ✅ MessageBroker trait - Pub-sub interface
- ✅ InMemoryMessageBroker - Implementation (~211ns routing)
- ✅ Message envelope handling
- ✅ Subscriber management
- ✅ Message delivery execution

**DOES NOT OWN:**
- ❌ WASM component subscription policies (Layer 2)
- ❌ Component-to-component routing logic (Layer 2)
- ❌ Multicodec message format (Layer 2)

### Integration Pattern (Phase 4)

**Data Flow (Topic-Based Pub-Sub):**
```
1. ComponentActor publishes message to topic
   ↓ (via ComponentMessage::InterComponent)
   
2. MessageRouter wraps in MessageEnvelope
   ↓ (Layer 2 → Layer 3 boundary)
   
3. MessageBroker.publish(envelope)
   ↓ (Layer 3 broadcasts to subscribers)
   
4. ActorSystem receives as primary subscriber
   ↓ (Layer 3 → Layer 2 routing)
   
5. ActorSystem routes to ComponentActor mailbox
   ↓ (via ComponentRegistry lookup)
   
6. ComponentActor.handle_message() invoked
   ↓ (Layer 2 processes WASM message)
```

**Key Principle:** Layer 3 handles message delivery, Layer 2 handles component-specific routing decisions.

---

## Task 4.1: MessageBroker Setup for Components

**Estimated Effort:** 4-5 hours  
**Priority:** CRITICAL - Foundation for Tasks 4.2-4.3

### Objective

Create the integration bridge between ComponentActor (Layer 2) and MessageBroker (Layer 3), enabling components to publish and subscribe to topics while maintaining layer separation per ADR-WASM-018.

### Deliverables

#### 1.1 MessageBrokerBridge Trait (Similar to SupervisorNodeBridge Pattern)

**File:** `src/actor/message_broker_bridge.rs` (new file, ~300 lines)

**Design Decision:**
Use bridge trait pattern to abstract MessageBroker access, preventing direct Layer 3 dependencies in ComponentActor.

**Implementation:**
```rust
// src/actor/message_broker_bridge.rs

use airssys_rt::broker::{MessageBroker, MessageEnvelope};
use crate::core::ComponentId;
use std::sync::Arc;
use async_trait::async_trait;

/// Bridge abstraction for MessageBroker access
/// Maintains layer separation (ADR-WASM-018)
#[async_trait]
pub trait MessageBrokerBridge: Send + Sync {
    /// Publish message to topic
    async fn publish(
        &self,
        topic: &str,
        message: ComponentMessage,
    ) -> Result<(), WasmError>;
    
    /// Subscribe to topic (returns subscription handle)
    async fn subscribe(
        &self,
        topic: &str,
    ) -> Result<SubscriptionHandle, WasmError>;
    
    /// Unsubscribe from topic
    async fn unsubscribe(
        &self,
        handle: SubscriptionHandle,
    ) -> Result<(), WasmError>;
    
    /// Get current subscriptions for component
    async fn subscriptions(
        &self,
        component_id: &ComponentId,
    ) -> Result<Vec<String>, WasmError>;
}

/// Concrete implementation wrapping airssys-rt MessageBroker
pub struct MessageBrokerWrapper {
    broker: Arc<InMemoryMessageBroker<ComponentMessage>>,
    subscription_tracker: Arc<RwLock<SubscriptionTracker>>,
}

impl MessageBrokerWrapper {
    pub fn new(broker: Arc<InMemoryMessageBroker<ComponentMessage>>) -> Self {
        Self {
            broker,
            subscription_tracker: Arc::new(RwLock::new(SubscriptionTracker::new())),
        }
    }
    
    /// Create from existing broker instance
    pub fn from_broker(broker: Arc<InMemoryMessageBroker<ComponentMessage>>) -> Self {
        Self::new(broker)
    }
}

#[async_trait]
impl MessageBrokerBridge for MessageBrokerWrapper {
    async fn publish(
        &self,
        topic: &str,
        message: ComponentMessage,
    ) -> Result<(), WasmError> {
        // Wrap in MessageEnvelope with topic metadata
        let envelope = MessageEnvelope {
            topic: topic.to_string(),
            payload: message,
            timestamp: Utc::now(),
            correlation_id: None,
        };
        
        // Delegate to Layer 3 MessageBroker
        self.broker.publish(envelope)
            .await
            .map_err(|e| WasmError::MessageBrokerError(e.to_string()))
    }
    
    async fn subscribe(
        &self,
        topic: &str,
    ) -> Result<SubscriptionHandle, WasmError> {
        // Subscribe to Layer 3 broker
        let stream = self.broker.subscribe()
            .await
            .map_err(|e| WasmError::MessageBrokerError(e.to_string()))?;
        
        // Create subscription handle
        let handle = SubscriptionHandle::new(topic);
        
        // Track subscription
        let mut tracker = self.subscription_tracker.write().await;
        tracker.add_subscription(handle.clone(), topic.to_string());
        
        Ok(handle)
    }
    
    async fn unsubscribe(
        &self,
        handle: SubscriptionHandle,
    ) -> Result<(), WasmError> {
        // Remove from tracking
        let mut tracker = self.subscription_tracker.write().await;
        tracker.remove_subscription(&handle)
            .map_err(|e| WasmError::Internal(e.to_string()))
    }
    
    async fn subscriptions(
        &self,
        component_id: &ComponentId,
    ) -> Result<Vec<String>, WasmError> {
        let tracker = self.subscription_tracker.read().await;
        Ok(tracker.get_subscriptions(component_id))
    }
}

/// Tracks component subscriptions (Layer 2 responsibility)
struct SubscriptionTracker {
    subscriptions: HashMap<ComponentId, Vec<String>>,
}

impl SubscriptionTracker {
    fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
        }
    }
    
    fn add_subscription(&mut self, handle: SubscriptionHandle, topic: String) {
        self.subscriptions
            .entry(handle.component_id.clone())
            .or_insert_with(Vec::new)
            .push(topic);
    }
    
    fn remove_subscription(&mut self, handle: &SubscriptionHandle) -> Result<(), String> {
        if let Some(topics) = self.subscriptions.get_mut(&handle.component_id) {
            topics.retain(|t| t != &handle.topic);
            Ok(())
        } else {
            Err(format!("No subscriptions found for component: {:?}", handle.component_id))
        }
    }
    
    fn get_subscriptions(&self, component_id: &ComponentId) -> Vec<String> {
        self.subscriptions
            .get(component_id)
            .cloned()
            .unwrap_or_default()
    }
}

/// Handle representing a subscription
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubscriptionHandle {
    component_id: ComponentId,
    topic: String,
    handle_id: Uuid,
}

impl SubscriptionHandle {
    fn new(topic: &str) -> Self {
        Self {
            component_id: ComponentId::new(), // TODO: Get from context
            topic: topic.to_string(),
            handle_id: Uuid::new_v4(),
        }
    }
}
```

**Rationale:**
- Bridge pattern maintains layer separation (same as SupervisorNodeBridge in Phase 3.2)
- SubscriptionTracker provides Layer 2 visibility into subscriptions
- MessageBrokerWrapper wraps Layer 3 broker without exposing internal details
- Follows ADR-WASM-018 ownership boundaries

#### 1.2 ComponentActor MessageBroker Integration

**File:** `src/actor/component_actor.rs` (modify existing, +150 lines)

**Add broker field to ComponentActor:**
```rust
pub struct ComponentActor {
    // ... existing fields ...
    
    /// MessageBroker bridge (set during spawn)
    broker: Option<Arc<dyn MessageBrokerBridge>>,
}

impl ComponentActor {
    /// Set message broker (called by ComponentSpawner)
    pub fn set_broker(&mut self, broker: Arc<dyn MessageBrokerBridge>) {
        self.broker = Some(broker);
    }
    
    /// Publish message to topic
    pub async fn publish_message(
        &self,
        topic: &str,
        message: ComponentMessage,
    ) -> Result<(), WasmError> {
        let broker = self.broker.as_ref()
            .ok_or_else(|| WasmError::BrokerNotConfigured)?;
        
        broker.publish(topic, message).await
    }
    
    /// Subscribe to topic
    pub async fn subscribe_topic(
        &mut self,
        topic: &str,
    ) -> Result<SubscriptionHandle, WasmError> {
        let broker = self.broker.as_ref()
            .ok_or_else(|| WasmError::BrokerNotConfigured)?;
        
        broker.subscribe(topic).await
    }
}
```

#### 1.3 ComponentSpawner Broker Integration

**File:** `src/actor/component_spawner.rs` (modify existing, +100 lines)

**Add broker to spawner:**
```rust
pub struct ComponentSpawner {
    actor_system: ActorSystem,
    registry: ComponentRegistry,
    supervisor_wrapper: Arc<SupervisorNodeWrapper>, // from Phase 3.2
    broker: Arc<MessageBrokerWrapper>,              // NEW for Phase 4
}

impl ComponentSpawner {
    pub fn new(
        actor_system: ActorSystem,
        registry: ComponentRegistry,
        supervisor_wrapper: Arc<SupervisorNodeWrapper>,
        broker: Arc<MessageBrokerWrapper>,
    ) -> Self {
        Self {
            actor_system,
            registry,
            supervisor_wrapper,
            broker,
        }
    }
    
    pub async fn spawn_component(
        &mut self,
        component_spec: ComponentSpec,
        capabilities: CapabilitySet,
    ) -> Result<(ComponentId, ActorAddress), WasmError> {
        // ... existing spawn logic ...
        
        // NEW: Set broker on component
        component_actor.set_broker(Arc::clone(&self.broker) as Arc<dyn MessageBrokerBridge>);
        
        // ... continue existing logic ...
    }
}
```

### Testing Strategy (Task 4.1)

**Unit Tests:** `tests/message_broker_bridge_tests.rs` (new file, 10 tests)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_broker_bridge_publish() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let wrapper = MessageBrokerWrapper::from_broker(broker);
        
        let message = ComponentMessage::InterComponent {
            sender: ComponentId::new(),
            payload: vec![1, 2, 3],
        };
        
        let result = wrapper.publish("test-topic", message).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_broker_bridge_subscribe() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let wrapper = MessageBrokerWrapper::from_broker(broker);
        
        let handle = wrapper.subscribe("test-topic").await.unwrap();
        assert_eq!(handle.topic, "test-topic");
    }
    
    #[tokio::test]
    async fn test_subscription_tracking() {
        let broker = Arc::new(InMemoryMessageBroker::new());
        let wrapper = MessageBrokerWrapper::from_broker(broker);
        
        let component_id = ComponentId::new();
        let handle = wrapper.subscribe("topic-1").await.unwrap();
        
        let subscriptions = wrapper.subscriptions(&component_id).await.unwrap();
        assert_eq!(subscriptions.len(), 1);
        assert_eq!(subscriptions[0], "topic-1");
    }
    
    // ... 7 more tests for error cases, unsubscribe, multiple subscriptions, etc.
}
```

**Integration Tests:** `tests/component_broker_integration_tests.rs` (new file, 5 tests)

```rust
#[tokio::test]
async fn test_component_publish_via_broker() {
    // Setup actor system, broker, spawner
    let actor_system = ActorSystem::new();
    let broker = Arc::new(InMemoryMessageBroker::new());
    let wrapper = Arc::new(MessageBrokerWrapper::from_broker(broker));
    let registry = ComponentRegistry::new();
    let spawner = ComponentSpawner::new(actor_system, registry, wrapper);
    
    // Spawn component with broker
    let (component_id, _) = spawner.spawn_component(spec, caps).await.unwrap();
    
    // Publish message via component
    let component = get_component(&component_id);
    let result = component.publish_message("test-topic", message).await;
    
    assert!(result.is_ok());
}
```

### Success Criteria (Task 4.1)

- ✅ MessageBrokerBridge trait compiles and trait bounds satisfied
- ✅ MessageBrokerWrapper wraps InMemoryMessageBroker correctly
- ✅ SubscriptionTracker maintains subscription state
- ✅ ComponentActor.set_broker() integration working
- ✅ ComponentSpawner passes broker to components
- ✅ 10 unit tests + 5 integration tests passing
- ✅ Zero warnings (compiler + clippy)
- ✅ Documentation complete with examples

### Estimated Timeline
- Implementation: 3-4 hours
- Testing: 1 hour
- Documentation: 0.5 hours
- **Total:** 4-5 hours

---

## Task 4.2: Pub-Sub Message Routing

**Estimated Effort:** 4-5 hours  
**Priority:** HIGH - Core messaging functionality  
**Dependencies:** Task 4.1 Complete

### Objective

Implement topic-based message publishing, filtering, and delivery to multiple subscribers using MessageBroker pub-sub patterns from ADR-WASM-009.

### Deliverables

#### 2.1 Topic-Based Message Publishing

**File:** `src/actor/message_publisher.rs` (new file, ~400 lines)

**Implementation:**
```rust
// src/actor/message_publisher.rs

use crate::actor::{ComponentMessage, MessageBrokerBridge};
use crate::core::ComponentId;

/// Component message publisher with topic support
pub struct MessagePublisher {
    component_id: ComponentId,
    broker: Arc<dyn MessageBrokerBridge>,
}

impl MessagePublisher {
    pub fn new(
        component_id: ComponentId,
        broker: Arc<dyn MessageBrokerBridge>,
    ) -> Self {
        Self {
            component_id,
            broker,
        }
    }
    
    /// Publish fire-and-forget message to topic
    pub async fn publish(
        &self,
        topic: &str,
        payload: Vec<u8>,
    ) -> Result<(), WasmError> {
        let message = ComponentMessage::InterComponent {
            sender: self.component_id.clone(),
            payload,
        };
        
        self.broker.publish(topic, message).await
    }
    
    /// Publish message to multiple topics (broadcast)
    pub async fn publish_multi(
        &self,
        topics: &[&str],
        payload: Vec<u8>,
    ) -> Result<(), WasmError> {
        for topic in topics {
            self.publish(topic, payload.clone()).await?;
        }
        Ok(())
    }
    
    /// Publish with correlation ID (for request-response)
    pub async fn publish_with_correlation(
        &self,
        topic: &str,
        payload: Vec<u8>,
        correlation_id: Uuid,
    ) -> Result<(), WasmError> {
        let message = ComponentMessage::InterComponentWithCorrelation {
            sender: self.component_id.clone(),
            payload,
            correlation_id,
        };
        
        self.broker.publish(topic, message).await
    }
}
```

#### 2.2 Topic-Based Message Filtering

**File:** `src/actor/message_filter.rs` (new file, ~350 lines)

**Implementation:**
```rust
// src/actor/message_filter.rs

/// Topic filter for subscription matching
pub struct TopicFilter {
    patterns: Vec<TopicPattern>,
}

impl TopicFilter {
    /// Create filter from topic patterns
    pub fn from_patterns(patterns: Vec<&str>) -> Self {
        Self {
            patterns: patterns
                .into_iter()
                .map(TopicPattern::parse)
                .collect(),
        }
    }
    
    /// Check if message topic matches filter
    pub fn matches(&self, topic: &str) -> bool {
        self.patterns.iter().any(|pattern| pattern.matches(topic))
    }
}

/// Topic pattern with wildcard support
/// Examples:
///   - "events.user.*" matches "events.user.login"
///   - "events.#" matches "events.user.login.success"
#[derive(Debug, Clone)]
pub struct TopicPattern {
    segments: Vec<PatternSegment>,
}

#[derive(Debug, Clone)]
enum PatternSegment {
    Literal(String),
    SingleWildcard,   // * matches one segment
    MultiWildcard,    // # matches any number of segments
}

impl TopicPattern {
    pub fn parse(pattern: &str) -> Self {
        let segments = pattern.split('.')
            .map(|s| match s {
                "*" => PatternSegment::SingleWildcard,
                "#" => PatternSegment::MultiWildcard,
                literal => PatternSegment::Literal(literal.to_string()),
            })
            .collect();
        
        Self { segments }
    }
    
    pub fn matches(&self, topic: &str) -> bool {
        let topic_segments: Vec<&str> = topic.split('.').collect();
        self.matches_segments(&topic_segments, 0, 0)
    }
    
    fn matches_segments(
        &self,
        topic_segments: &[&str],
        pattern_idx: usize,
        topic_idx: usize,
    ) -> bool {
        // Recursive pattern matching implementation
        // Details omitted for brevity
        true // Placeholder
    }
}
```

#### 2.3 Multiple Subscriber Handling

**File:** `src/actor/subscriber_manager.rs` (new file, ~300 lines)

**Implementation:**
```rust
// src/actor/subscriber_manager.rs

/// Manages component subscriptions
pub struct SubscriberManager {
    subscriptions: Arc<RwLock<HashMap<ComponentId, Vec<Subscription>>>>,
}

#[derive(Debug, Clone)]
struct Subscription {
    component_id: ComponentId,
    topics: Vec<String>,
    filter: TopicFilter,
}

impl SubscriberManager {
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Subscribe component to topics
    pub async fn subscribe(
        &self,
        component_id: ComponentId,
        topics: Vec<String>,
    ) -> Result<SubscriptionHandle, WasmError> {
        let filter = TopicFilter::from_patterns(
            topics.iter().map(|s| s.as_str()).collect()
        );
        
        let subscription = Subscription {
            component_id: component_id.clone(),
            topics: topics.clone(),
            filter,
        };
        
        let mut subs = self.subscriptions.write().await;
        subs.entry(component_id.clone())
            .or_insert_with(Vec::new)
            .push(subscription);
        
        Ok(SubscriptionHandle::new(component_id, topics))
    }
    
    /// Get all subscribers matching topic
    pub async fn get_subscribers(&self, topic: &str) -> Vec<ComponentId> {
        let subs = self.subscriptions.read().await;
        subs.values()
            .flatten()
            .filter(|sub| sub.filter.matches(topic))
            .map(|sub| sub.component_id.clone())
            .collect()
    }
    
    /// Unsubscribe component from topics
    pub async fn unsubscribe(
        &self,
        handle: &SubscriptionHandle,
    ) -> Result<(), WasmError> {
        let mut subs = self.subscriptions.write().await;
        if let Some(component_subs) = subs.get_mut(&handle.component_id) {
            component_subs.retain(|sub| !sub.topics.iter().any(|t| handle.topics.contains(t)));
            Ok(())
        } else {
            Err(WasmError::SubscriptionNotFound)
        }
    }
}
```

### Testing Strategy (Task 4.2)

**Unit Tests:** `tests/message_routing_tests.rs` (new file, 10 tests)

```rust
#[tokio::test]
async fn test_topic_filter_single_wildcard() {
    let filter = TopicFilter::from_patterns(vec!["events.user.*"]);
    assert!(filter.matches("events.user.login"));
    assert!(filter.matches("events.user.logout"));
    assert!(!filter.matches("events.system.restart"));
}

#[tokio::test]
async fn test_topic_filter_multi_wildcard() {
    let filter = TopicFilter::from_patterns(vec!["events.#"]);
    assert!(filter.matches("events.user.login"));
    assert!(filter.matches("events.user.login.success"));
    assert!(!filter.matches("system.restart"));
}

#[tokio::test]
async fn test_multiple_subscribers() {
    let manager = SubscriberManager::new();
    
    let component_a = ComponentId::new();
    let component_b = ComponentId::new();
    
    manager.subscribe(component_a.clone(), vec!["events.*".into()]).await.unwrap();
    manager.subscribe(component_b.clone(), vec!["events.user.*".into()]).await.unwrap();
    
    let subscribers = manager.get_subscribers("events.user.login").await;
    assert_eq!(subscribers.len(), 2);
}

// ... 7 more tests for filtering, unsubscribe, edge cases
```

**Integration Tests:** `tests/pub_sub_integration_tests.rs` (new file, 5 tests)

```rust
#[tokio::test]
async fn test_end_to_end_pub_sub() {
    // Setup system
    let actor_system = ActorSystem::new();
    let broker = Arc::new(InMemoryMessageBroker::new());
    let wrapper = Arc::new(MessageBrokerWrapper::from_broker(broker));
    
    // Spawn publisher component
    let (pub_id, _) = spawner.spawn_component(pub_spec, caps).await.unwrap();
    
    // Spawn subscriber component
    let (sub_id, _) = spawner.spawn_component(sub_spec, caps).await.unwrap();
    
    // Subscribe
    subscriber.subscribe_topic("test-topic").await.unwrap();
    
    // Publish
    publisher.publish_message("test-topic", message).await.unwrap();
    
    // Verify delivery
    let received = subscriber.receive_message().await.unwrap();
    assert_eq!(received.payload, message.payload);
}
```

### Success Criteria (Task 4.2)

- ✅ MessagePublisher publishes to topics
- ✅ TopicFilter matches wildcards correctly
- ✅ SubscriberManager tracks multiple subscribers
- ✅ Messages delivered to all matching subscribers
- ✅ Topic filtering works correctly
- ✅ 10 unit tests + 5 integration tests passing
- ✅ Zero warnings
- ✅ Documentation with topic pattern examples

### Estimated Timeline
- Implementation: 3-4 hours
- Testing: 1 hour
- Documentation: 0.5 hours
- **Total:** 4-5 hours

---

## Task 4.3: ActorSystem as Primary Subscriber Pattern

**Estimated Effort:** 4-6 hours  
**Priority:** CRITICAL - Unified routing architecture  
**Dependencies:** Tasks 4.1-4.2 Complete

### Objective

Implement the "ActorSystem as Primary Subscriber" pattern from ADR-WASM-009, where ActorSystem subscribes to all component messages and routes them to ComponentActor mailboxes, centralizing message routing decisions.

### Deliverables

#### 3.1 ActorSystem Subscription Manager

**File:** `src/actor/actor_system_subscriber.rs` (new file, ~450 lines)

**Implementation:**
```rust
// src/actor/actor_system_subscriber.rs

use airssys_rt::broker::{MessageBroker, MessageStream};
use crate::actor::{ComponentRegistry, ComponentMessage};

/// ActorSystem subscriber that routes messages to components
pub struct ActorSystemSubscriber {
    broker: Arc<InMemoryMessageBroker<ComponentMessage>>,
    registry: ComponentRegistry,
    message_stream: Option<MessageStream<ComponentMessage>>,
    routing_task: Option<JoinHandle<()>>,
}

impl ActorSystemSubscriber {
    pub fn new(
        broker: Arc<InMemoryMessageBroker<ComponentMessage>>,
        registry: ComponentRegistry,
    ) -> Self {
        Self {
            broker,
            registry,
            message_stream: None,
            routing_task: None,
        }
    }
    
    /// Start subscribing to broker and routing messages
    pub async fn start(&mut self) -> Result<(), WasmError> {
        // Subscribe to broker
        let mut stream = self.broker.subscribe()
            .await
            .map_err(|e| WasmError::MessageBrokerError(e.to_string()))?;
        
        let registry = self.registry.clone();
        
        // Spawn routing task
        let task = tokio::spawn(async move {
            while let Some(envelope) = stream.recv().await {
                if let Err(e) = Self::route_message(&registry, envelope).await {
                    error!("Failed to route message: {}", e);
                }
            }
        });
        
        self.message_stream = Some(stream);
        self.routing_task = Some(task);
        
        Ok(())
    }
    
    /// Route message to component mailbox
    async fn route_message(
        registry: &ComponentRegistry,
        envelope: MessageEnvelope<ComponentMessage>,
    ) -> Result<(), WasmError> {
        // Extract target component from message
        let target_id = Self::extract_target(&envelope.payload)?;
        
        // Lookup component address
        let actor_address = registry.lookup(&target_id)
            .await
            .map_err(|_| WasmError::ComponentNotFound(target_id.clone()))?;
        
        // Send to mailbox
        actor_address.send(envelope.payload)
            .await
            .map_err(|e| WasmError::MessageDeliveryFailed(e.to_string()))
    }
    
    fn extract_target(message: &ComponentMessage) -> Result<ComponentId, WasmError> {
        match message {
            ComponentMessage::InterComponent { sender, .. } => {
                // TODO: Proper target extraction from message metadata
                Ok(sender.clone()) // Placeholder
            }
            _ => Err(WasmError::InvalidMessageFormat),
        }
    }
    
    /// Stop routing and cleanup
    pub async fn stop(&mut self) -> Result<(), WasmError> {
        if let Some(task) = self.routing_task.take() {
            task.abort();
        }
        Ok(())
    }
}
```

#### 3.2 Unified Message Routing Architecture

**File:** `src/actor/unified_router.rs` (new file, ~400 lines)

**Implementation:**
```rust
// src/actor/unified_router.rs

/// Unified routing architecture coordinator
pub struct UnifiedRouter {
    actor_subscriber: Arc<Mutex<ActorSystemSubscriber>>,
    subscriber_manager: Arc<SubscriberManager>,
    routing_stats: Arc<RwLock<RoutingStats>>,
}

impl UnifiedRouter {
    pub fn new(
        broker: Arc<InMemoryMessageBroker<ComponentMessage>>,
        registry: ComponentRegistry,
    ) -> Self {
        let actor_subscriber = Arc::new(Mutex::new(
            ActorSystemSubscriber::new(broker, registry)
        ));
        
        Self {
            actor_subscriber,
            subscriber_manager: Arc::new(SubscriberManager::new()),
            routing_stats: Arc::new(RwLock::new(RoutingStats::new())),
        }
    }
    
    /// Start unified routing
    pub async fn start(&self) -> Result<(), WasmError> {
        self.actor_subscriber.lock().await.start().await
    }
    
    /// Route message with centralized logic
    pub async fn route(
        &self,
        source: ComponentId,
        target: ComponentId,
        message: ComponentMessage,
    ) -> Result<(), WasmError> {
        // Update stats
        self.routing_stats.write().await.record_route();
        
        // Get subscribers for message topic
        let subscribers = self.subscriber_manager
            .get_subscribers(&Self::extract_topic(&message))
            .await;
        
        // Route to all subscribers
        for subscriber_id in subscribers {
            self.route_to_component(&subscriber_id, message.clone()).await?;
        }
        
        Ok(())
    }
    
    async fn route_to_component(
        &self,
        component_id: &ComponentId,
        message: ComponentMessage,
    ) -> Result<(), WasmError> {
        // Delegate to ActorSystemSubscriber routing
        // Implementation delegates to existing routing logic
        Ok(())
    }
    
    fn extract_topic(message: &ComponentMessage) -> String {
        // Extract topic from message metadata
        "default".to_string() // Placeholder
    }
    
    /// Get routing statistics
    pub async fn stats(&self) -> RoutingStats {
        self.routing_stats.read().await.clone()
    }
}

#[derive(Debug, Clone, Default)]
pub struct RoutingStats {
    pub total_messages: u64,
    pub successful_routes: u64,
    pub failed_routes: u64,
    pub average_latency_ns: u64,
}

impl RoutingStats {
    fn new() -> Self {
        Self::default()
    }
    
    fn record_route(&mut self) {
        self.total_messages += 1;
        self.successful_routes += 1;
    }
}
```

#### 3.3 Pattern Documentation

**File:** `src/actor/README.md` (update existing, +200 lines)

**Documentation sections:**
1. ActorSystem as Primary Subscriber pattern explanation
2. Architecture diagram showing message flow
3. Code examples for common patterns
4. Performance characteristics
5. Troubleshooting guide

### Testing Strategy (Task 4.3)

**Integration Tests:** `tests/actor_system_subscriber_tests.rs` (new file, 10 tests)

```rust
#[tokio::test]
async fn test_actor_system_subscribes_to_broker() {
    let broker = Arc::new(InMemoryMessageBroker::new());
    let registry = ComponentRegistry::new();
    
    let mut subscriber = ActorSystemSubscriber::new(broker.clone(), registry);
    subscriber.start().await.unwrap();
    
    // Verify subscription active
    assert!(subscriber.message_stream.is_some());
    assert!(subscriber.routing_task.is_some());
}

#[tokio::test]
async fn test_message_routes_to_mailbox() {
    // Setup
    let broker = Arc::new(InMemoryMessageBroker::new());
    let registry = ComponentRegistry::new();
    let mut subscriber = ActorSystemSubscriber::new(broker.clone(), registry.clone());
    
    // Spawn component
    let (component_id, actor_addr) = spawn_test_component().await;
    registry.register(component_id.clone(), actor_addr.clone()).await.unwrap();
    
    // Start subscriber
    subscriber.start().await.unwrap();
    
    // Publish message
    let message = ComponentMessage::InterComponent {
        sender: ComponentId::new(),
        payload: vec![1, 2, 3],
    };
    broker.publish(MessageEnvelope::new("test-topic", message)).await.unwrap();
    
    // Verify delivery
    let received = actor_addr.recv().await.unwrap();
    assert_eq!(received.payload, vec![1, 2, 3]);
}

#[tokio::test]
async fn test_unified_router_centralizes_routing() {
    let broker = Arc::new(InMemoryMessageBroker::new());
    let registry = ComponentRegistry::new();
    
    let router = UnifiedRouter::new(broker, registry);
    router.start().await.unwrap();
    
    // Route message
    let result = router.route(
        ComponentId::new(),
        ComponentId::new(),
        ComponentMessage::InterComponent {
            sender: ComponentId::new(),
            payload: vec![1, 2, 3],
        }
    ).await;
    
    assert!(result.is_ok());
    
    // Check stats
    let stats = router.stats().await;
    assert_eq!(stats.total_messages, 1);
}

// ... 7 more tests for error handling, multiple subscribers, stats tracking
```

**End-to-End Tests:** `tests/phase_4_integration_tests.rs` (new file, 5 tests)

```rust
#[tokio::test]
async fn test_full_pub_sub_flow_with_actor_system() {
    // Setup complete system
    let actor_system = ActorSystem::new();
    let broker = Arc::new(InMemoryMessageBroker::new());
    let registry = ComponentRegistry::new();
    let wrapper = Arc::new(MessageBrokerWrapper::from_broker(broker.clone()));
    
    // Create unified router
    let router = UnifiedRouter::new(broker, registry.clone());
    router.start().await.unwrap();
    
    // Spawn components
    let spawner = ComponentSpawner::new(actor_system, registry, wrapper);
    let (publisher_id, _) = spawner.spawn_component(pub_spec, caps).await.unwrap();
    let (subscriber_id, _) = spawner.spawn_component(sub_spec, caps).await.unwrap();
    
    // Subscribe
    subscriber.subscribe_topic("events.user.login").await.unwrap();
    
    // Publish
    publisher.publish_message("events.user.login", message).await.unwrap();
    
    // Verify routing through ActorSystem
    let received = subscriber.receive_message().await.unwrap();
    assert_eq!(received.payload, message.payload);
    
    // Verify stats
    let stats = router.stats().await;
    assert_eq!(stats.total_messages, 1);
    assert_eq!(stats.successful_routes, 1);
}
```

### Success Criteria (Task 4.3)

- ✅ ActorSystem subscribes to MessageBroker
- ✅ Messages route through ActorSystem to mailboxes
- ✅ Routing logic centralized in UnifiedRouter
- ✅ Pattern clear and documented
- ✅ 10 integration tests + 5 end-to-end tests passing
- ✅ Routing performance <100ns overhead
- ✅ Zero warnings
- ✅ Architecture diagram in documentation

### Estimated Timeline
- Implementation: 3-4 hours
- Testing: 1.5 hours
- Documentation: 1 hour
- **Total:** 4-6 hours

---

## Performance Targets

### Phase 4 Performance Goals

| Metric | Target | Baseline (airssys-rt) | Notes |
|--------|--------|----------------------|-------|
| Event routing overhead | <100ns | ~211ns MessageBroker | Layer 2 bridge overhead |
| Message throughput | >4.7M msg/sec | 4.7M msg/sec | Match airssys-rt baseline |
| Topic filter match | <50ns | N/A | Wildcard pattern matching |
| Subscription lookup | <10ns | N/A | HashMap lookup |
| End-to-end latency | <1ms | N/A | Component → mailbox |

### Performance Validation

**Benchmarks:** `benches/phase_4_benchmarks.rs` (new file)

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_message_publish(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let broker = Arc::new(InMemoryMessageBroker::new());
    let wrapper = Arc::new(MessageBrokerWrapper::from_broker(broker));
    
    c.bench_function("message_publish", |b| {
        b.iter(|| {
            rt.block_on(async {
                wrapper.publish(
                    black_box("test-topic"),
                    black_box(ComponentMessage::InterComponent {
                        sender: ComponentId::new(),
                        payload: vec![1, 2, 3],
                    })
                ).await.unwrap();
            })
        })
    });
}

fn bench_topic_filter_match(c: &mut Criterion) {
    let filter = TopicFilter::from_patterns(vec!["events.user.*"]);
    
    c.bench_function("topic_filter_match", |b| {
        b.iter(|| {
            black_box(filter.matches("events.user.login"));
        })
    });
}

criterion_group!(benches, bench_message_publish, bench_topic_filter_match);
criterion_main!(benches);
```

---

## Integration Checklist

### Pre-Implementation Verification

- [ ] Phase 3 complete (Tasks 3.1-3.3) ✅
- [ ] 719 tests passing ✅
- [ ] 0 warnings ✅
- [ ] ADR-WASM-009 reviewed
- [ ] ADR-WASM-018 reviewed
- [ ] airssys-rt MessageBroker API understood

### Task 4.1 Checklist

- [ ] MessageBrokerBridge trait implemented
- [ ] MessageBrokerWrapper concrete implementation
- [ ] SubscriptionTracker working
- [ ] ComponentActor.set_broker() integration
- [ ] ComponentSpawner broker injection
- [ ] 15 tests passing (10 unit + 5 integration)
- [ ] Zero warnings
- [ ] Documentation complete

### Task 4.2 Checklist

- [ ] MessagePublisher implemented
- [ ] TopicFilter with wildcard support
- [ ] SubscriberManager tracking subscriptions
- [ ] Multiple subscriber delivery working
- [ ] Topic filtering correct
- [ ] 15 tests passing (10 unit + 5 integration)
- [ ] Zero warnings
- [ ] Topic pattern examples documented

### Task 4.3 Checklist

- [ ] ActorSystemSubscriber routing messages
- [ ] UnifiedRouter centralized logic
- [ ] Routing statistics tracking
- [ ] Pattern documentation complete
- [ ] 15 tests passing (10 integration + 5 end-to-end)
- [ ] Routing performance <100ns overhead
- [ ] Zero warnings
- [ ] Architecture diagram created

---

## Quality Gates

### Code Quality Standards (§2.1-§6.3 Compliance)

**Before Task Completion:**
1. ✅ All tests passing (target: 749+ total tests)
2. ✅ Zero compiler warnings
3. ✅ Zero clippy warnings (strict mode)
4. ✅ 100% rustdoc coverage for public APIs
5. ✅ Import organization per §2.1 (std → external → internal)
6. ✅ Error handling per Microsoft M-ERRORS-CANONICAL-STRUCTS
7. ✅ ADR-WASM-018 layer boundaries maintained

### Performance Validation

**Required Benchmarks:**
1. Message publish latency
2. Topic filter matching time
3. Subscription lookup time
4. End-to-end routing latency
5. Throughput under load

**Targets:**
- All benchmarks meet or exceed targets
- No performance regressions vs. Phase 3
- Document any deviations with justification

---

## Documentation Requirements

### Code Documentation

**Files to Document:**
1. `src/actor/message_broker_bridge.rs` - Bridge pattern explanation
2. `src/actor/message_publisher.rs` - Publishing patterns
3. `src/actor/message_filter.rs` - Topic filter examples
4. `src/actor/subscriber_manager.rs` - Subscription management
5. `src/actor/actor_system_subscriber.rs` - Primary subscriber pattern
6. `src/actor/unified_router.rs` - Routing architecture

**Documentation Standards:**
- 100% rustdoc coverage
- Code examples for all public APIs
- Architecture diagrams where appropriate
- Performance characteristics noted

### Architecture Documentation

**Required Documents:**
1. Phase 4 completion summary (this document updated)
2. Architecture decision record updates (ADR-WASM-009)
3. Integration guide for Block 5
4. Troubleshooting guide for common issues

---

## Risk Mitigation

### Risk 1: Layer Boundary Violations
**Impact:** High - ADR-WASM-018 compliance failure  
**Probability:** Medium - Complex bridge patterns  
**Mitigation:**
- Strict code review checklist
- Architectural review before implementation
- Unit tests for layer separation
- Reference Phase 3.2 SupervisorNodeBridge pattern

### Risk 2: Performance Degradation
**Impact:** High - Could impact overall system throughput  
**Probability:** Low - airssys-rt MessageBroker proven  
**Mitigation:**
- Benchmark early and often
- Profile with criterion.rs
- Target: <100ns overhead
- Maintain airssys-rt baseline: ~211ns routing

### Risk 3: Topic Filter Complexity
**Impact:** Medium - Incorrect message delivery  
**Probability:** Medium - Wildcard patterns non-trivial  
**Mitigation:**
- Comprehensive unit tests (10+ test cases)
- Reference MQTT topic filter patterns
- Document edge cases clearly
- Fuzz testing for pattern matching

### Risk 4: ActorSystem Subscription Integration
**Impact:** High - Broken centralized routing  
**Probability:** Low - Pattern proven in airssys-rt  
**Mitigation:**
- Follow airssys-rt subscriber patterns exactly
- Integration tests with full actor system
- End-to-end validation tests
- Clear failure logging

---

## Success Metrics Summary

### Phase 4 Overall Success Criteria

**Functional:**
- ✅ MessageBroker integration working
- ✅ Topic-based pub-sub operational
- ✅ Multiple subscriber delivery
- ✅ ActorSystem centralized routing
- ✅ Pattern clearly documented

**Quality:**
- ✅ 30+ new tests passing (target: 749+ total)
- ✅ Zero warnings (compiler + clippy)
- ✅ Code quality: 9.5/10
- ✅ 100% ADR-WASM-018 compliance
- ✅ 100% rustdoc coverage

**Performance:**
- ✅ Event routing: <100ns overhead
- ✅ Throughput: >4.7M msg/sec
- ✅ Topic filter: <50ns match
- ✅ End-to-end: <1ms latency

**Documentation:**
- ✅ Architecture diagrams created
- ✅ Integration guide for Block 5
- ✅ Code examples for all patterns
- ✅ Troubleshooting guide

---

## Next Steps After Phase 4

### Phase 5: Performance Optimization
- Component spawn optimization
- Message routing profiling
- Memory footprint reduction

### Phase 6: Testing and Integration Validation
- Integration test suite
- Performance validation
- Actor-based testing framework

### Block 5: Inter-Component Communication
- Host functions for send_message()
- Capability enforcement (ADR-WASM-005)
- Request-response patterns
- Quota management

---

## References

### ADRs
- **ADR-WASM-009:** Component Communication Model (pub-sub architecture)
- **ADR-WASM-018:** Three-Layer Architecture (layer boundaries)
- **ADR-WASM-001:** Multicodec Compatibility Strategy (message serialization)
- **ADR-WASM-006:** Component Isolation and Sandboxing (actor model)

### Knowledge Documentation
- **KNOWLEDGE-WASM-016:** Actor System Integration Implementation Guide
- **KNOWLEDGE-WASM-005:** Inter-Component Messaging Architecture
- **KNOWLEDGE-RT-013:** Actor Performance Benchmarking Results

### airssys-rt Documentation
- **RT-TASK-008:** Message Broker Performance Baseline (211ns routing)
- `airssys-rt/src/broker/in_memory.rs` - MessageBroker implementation
- `airssys-rt/src/broker/traits.rs` - MessageBroker trait definition

### External References
- [Erlang/OTP gen_server](https://www.erlang.org/doc/man/gen_server.html) - Actor messaging
- [MQTT Topic Filters](https://www.hivemq.com/blog/mqtt-essentials-part-5-mqtt-topics-best-practices/) - Wildcard patterns
- [Multicodec Specification](https://github.com/multiformats/multicodec) - Message encoding

---

## Approval Checklist

- [x] All task deliverables clearly defined
- [x] Success criteria measurable
- [x] Testing strategy comprehensive
- [x] Performance targets specified
- [x] ADR compliance verified
- [x] Risk mitigation planned
- [x] Documentation requirements clear
- [x] Timeline estimates reasonable (12-16 hours total)

**Status:** READY FOR IMPLEMENTATION  
**Approval Date:** 2025-12-15  
**Next Review:** After Task 4.1 completion
