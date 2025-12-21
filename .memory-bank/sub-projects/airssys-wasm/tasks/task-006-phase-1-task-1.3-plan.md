# Action Plan for WASM-TASK-006 Phase 1 Task 1.3

**Task:** ActorSystem Event Subscription Infrastructure  
**Parent Task:** WASM-TASK-006 Block 5: Inter-Component Communication  
**Phase:** Phase 1 - MessageBroker Integration Foundation  
**Status:** READY FOR IMPLEMENTATION  
**Created:** 2025-12-21  
**Estimated Effort:** 12 hours

---

## Goal

Implement the ActorSystem event subscription infrastructure that enables:
1. ActorSystem to subscribe to MessageBroker events during initialization
2. ComponentId → ActorAddress registry management for address resolution
3. Message routing logic based on ComponentId addressing
4. Routing error handling and graceful fallback mechanisms
5. Internal subscription infrastructure documentation

This is **INTERNAL infrastructure** (runtime-level), NOT a component-facing API. Components are addressed by ComponentId directly. Topic-based pub-sub is an optional future enhancement (Phase 2+).

---

## Context & References

### Architecture Foundations

**ADR-WASM-020: Message Delivery Ownership**
- `ActorSystemSubscriber` owns message delivery via `mailbox_senders` map
- `ComponentRegistry` stays pure (identity lookup only: `ComponentId → ActorAddress`)
- Clean separation: Registry = identity, Subscriber = delivery

**ADR-WASM-009: Component Communication Model**
- ActorSystem as Primary Subscriber pattern
- ActorSystem subscribes to MessageBroker on behalf of all components
- Single subscriber routes to component mailboxes

**KNOWLEDGE-WASM-024: Component Messaging Clarifications**
- Phase 1 uses Direct ComponentId addressing (no topic routing)
- ActorSystem event-driven subscription IS the runtime-level subscription
- Components NEVER subscribe manually - runtime handles routing transparently

**KNOWLEDGE-WASM-026: Message Delivery Architecture - Final Decision**
- Complete message flow documented
- ActorSystemSubscriber owns `mailbox_senders: HashMap<ComponentId, MailboxSender>`
- Registration/unregistration lifecycle defined

### Current State (Post Task 1.1 & 1.2)

**Task 1.1 COMPLETE:**
- ✅ `ActorSystemSubscriber` has `mailbox_senders` field (line 186)
- ✅ `register_mailbox()` method implemented (lines 247-268)
- ✅ `unregister_mailbox()` method implemented (lines 297-317)
- ✅ `route_message_to_subscribers()` delivers via `sender.send(envelope.payload)` (line 454)
- ✅ 15 unit tests + 7 integration tests passing

**Task 1.2 COMPLETE:**
- ✅ `invoke_handle_message_with_timeout()` correctly invokes WASM exports
- ✅ 9 integration tests prove WASM invocation with real fixtures
- ✅ Message reception metrics and backpressure functional

### Remaining Gap for Task 1.3

While Task 1.1 implemented message delivery in `ActorSystemSubscriber`, the **subscription initialization** and **registration coordination** are not yet integrated with the broader runtime:

1. **WasmRuntime** doesn't auto-initialize MessageBroker subscription
2. **ComponentSpawner** doesn't auto-register mailbox with `ActorSystemSubscriber`
3. **Component shutdown** doesn't auto-unregister from `ActorSystemSubscriber`
4. **No centralized event subscription initialization** during runtime startup

---

## Implementation Steps

### Step 1: Create MessagingSubscriptionService Module (2 hours)

**File:** `airssys-wasm/src/runtime/messaging_subscription.rs` (NEW)

Create a service that coordinates:
- MessageBroker subscription during runtime initialization
- ActorSystemSubscriber lifecycle management
- Registration/unregistration hooks for components

```rust
/// MessagingSubscriptionService coordinates ActorSystem ↔ MessageBroker subscription.
///
/// This service:
/// 1. Initializes ActorSystemSubscriber during runtime startup
/// 2. Subscribes to MessageBroker and starts routing task
/// 3. Provides hooks for component registration/unregistration
/// 4. Handles graceful shutdown of subscription infrastructure
pub struct MessagingSubscriptionService<B: MessageBroker<ComponentMessage>> {
    subscriber: Arc<RwLock<ActorSystemSubscriber<B>>>,
    broker: Arc<B>,
    registry: ComponentRegistry,
    subscriber_manager: Arc<SubscriberManager>,
}

impl MessagingSubscriptionService {
    /// Initialize subscription service and start MessageBroker subscription.
    pub async fn start(&self) -> Result<(), WasmError>;
    
    /// Register component for message delivery.
    /// Called by ComponentSpawner after spawning.
    pub async fn register_component(
        &self,
        component_id: ComponentId,
        mailbox_sender: UnboundedSender<ComponentMessage>,
    ) -> Result<(), WasmError>;
    
    /// Unregister component from message delivery.
    /// Called during component shutdown.
    pub async fn unregister_component(&self, component_id: &ComponentId) -> Result<(), WasmError>;
    
    /// Resolve ComponentId to ActorAddress (identity lookup).
    pub fn resolve_address(&self, component_id: &ComponentId) -> Option<ActorAddress>;
    
    /// Gracefully stop subscription infrastructure.
    pub async fn stop(&self) -> Result<(), WasmError>;
    
    /// Get subscription status for monitoring.
    pub fn status(&self) -> SubscriptionStatus;
}
```

**Deliverables:**
- `MessagingSubscriptionService` struct
- Initialization and startup logic
- Component registration/unregistration methods
- Status reporting for monitoring
- Graceful shutdown handling

### Step 2: Integrate with WasmRuntime Initialization (2 hours)

**Files:**
- `airssys-wasm/src/runtime/mod.rs` - Add `MessagingSubscriptionService` export
- `airssys-wasm/src/core/wasm_runtime.rs` (or equivalent) - Integration point

**Changes:**
1. Add `messaging_subscription` field to WasmRuntime (or RuntimeContext)
2. Initialize `MessagingSubscriptionService` during runtime creation
3. Call `messaging_subscription.start()` during runtime startup
4. Call `messaging_subscription.stop()` during runtime shutdown

**Integration Pattern:**
```rust
impl WasmRuntime {
    pub async fn initialize_messaging(&mut self) -> Result<(), WasmError> {
        // 1. Create MessageBroker if not exists
        let broker = self.broker.clone();
        
        // 2. Create subscription service
        let subscription_service = MessagingSubscriptionService::new(
            broker,
            self.component_registry.clone(),
            Arc::new(SubscriberManager::new()),
        );
        
        // 3. Start subscription (subscribes to broker, spawns routing task)
        subscription_service.start().await?;
        
        self.messaging_subscription = Some(Arc::new(subscription_service));
        Ok(())
    }
}
```

### Step 3: ComponentId → ActorAddress Registry Integration (2 hours)

**File:** `airssys-wasm/src/actor/component/component_registry.rs`

**Current State:** ComponentRegistry already exists with:
- `ComponentId → ActorAddress` mapping
- `register()` and `unregister()` methods

**Task 1.3 Enhancement:**
- Ensure `ComponentRegistry` is properly integrated with subscription service
- Add `resolve_address()` helper if not present
- Add metrics for registry operations (optional)

**Integration Point:**
```rust
/// Resolve ComponentId to ActorAddress for routing decisions.
/// 
/// This is identity lookup only - actual delivery uses mailbox_senders.
impl ComponentRegistry {
    pub fn resolve_address(&self, component_id: &ComponentId) -> Option<ActorAddress> {
        self.components.read().get(component_id).cloned()
    }
    
    pub fn is_registered(&self, component_id: &ComponentId) -> bool {
        self.components.read().contains_key(component_id)
    }
    
    pub fn component_count(&self) -> usize {
        self.components.read().len()
    }
}
```

### Step 4: Message Routing Logic Enhancement (2 hours)

**File:** `airssys-wasm/src/actor/message/actor_system_subscriber.rs`

**Current State:** `route_message_to_subscribers()` works for basic delivery.

**Task 1.3 Enhancement:**
- Add routing metrics (messages routed, failures, latency)
- Add logging for routing decisions
- Handle edge cases (component shutting down mid-message)
- Add configurable routing behavior

**New Methods:**
```rust
impl<B: MessageBroker<ComponentMessage>> ActorSystemSubscriber<B> {
    /// Route message with full error context and metrics.
    pub async fn route_message_with_metrics(
        &self,
        envelope: MessageEnvelope<ComponentMessage>,
    ) -> RoutingResult {
        let start = Instant::now();
        let result = Self::route_message_to_subscribers(
            &self.mailbox_senders,
            &self.subscriber_manager,
            envelope,
        ).await;
        
        let latency = start.elapsed();
        self.record_routing_metrics(&result, latency);
        
        result.into()
    }
    
    /// Get routing statistics for monitoring.
    pub fn routing_stats(&self) -> RoutingStats {
        RoutingStats {
            messages_routed: self.messages_routed.load(Ordering::Relaxed),
            routing_errors: self.routing_errors.load(Ordering::Relaxed),
            avg_latency_ns: self.avg_latency_ns.load(Ordering::Relaxed),
        }
    }
}
```

### Step 5: Error Handling and Fallback (2 hours)

**Files:**
- `airssys-wasm/src/core/error.rs` - Add routing error types
- `airssys-wasm/src/actor/message/actor_system_subscriber.rs` - Error handling

**Error Types:**
```rust
/// Routing-specific error types for Task 1.3
#[derive(Debug, Clone)]
pub enum RoutingError {
    /// Target component not registered
    ComponentNotFound { component_id: String },
    
    /// Target component's mailbox is closed (shutting down)
    MailboxClosed { component_id: String },
    
    /// Message extraction failed (invalid message type)
    InvalidMessageType { message_type: String },
    
    /// Routing timeout (for future use with timeouts)
    RoutingTimeout { component_id: String, timeout_ms: u64 },
}
```

**Fallback Handling:**
```rust
async fn route_with_fallback(
    &self,
    envelope: MessageEnvelope<ComponentMessage>,
) -> Result<(), WasmError> {
    match self.try_route(&envelope).await {
        Ok(()) => Ok(()),
        Err(RoutingError::ComponentNotFound { component_id }) => {
            tracing::warn!(
                target = %component_id,
                "Message dropped: target component not found"
            );
            self.record_dropped_message(&envelope);
            // Return Ok - message is dropped gracefully
            Ok(())
        }
        Err(RoutingError::MailboxClosed { component_id }) => {
            tracing::warn!(
                target = %component_id,
                "Message dropped: target component shutting down"
            );
            self.record_dropped_message(&envelope);
            Ok(())
        }
        Err(e) => Err(WasmError::messaging_error(e.to_string())),
    }
}
```

### Step 6: Documentation (2 hours)

**Files:**
- Update module-level documentation in:
  - `src/runtime/messaging_subscription.rs`
  - `src/actor/message/actor_system_subscriber.rs`
- Update `src/runtime/mod.rs` with architecture overview

**Documentation Requirements:**
1. Architecture diagram for subscription flow
2. Lifecycle documentation (init → run → shutdown)
3. Integration guide for ComponentSpawner
4. Error handling and recovery documentation
5. Performance characteristics

---

## Unit Testing Plan

**MANDATORY**: Tests in module #[cfg(test)] blocks

**Location:** `src/runtime/messaging_subscription.rs`

### Test Category 1: Service Lifecycle
- [ ] `test_service_creation` - Service created with correct configuration
- [ ] `test_service_start_success` - Service starts and subscribes to broker
- [ ] `test_service_start_idempotent` - Multiple starts don't cause issues
- [ ] `test_service_stop_graceful` - Clean shutdown of routing task
- [ ] `test_service_status_reporting` - Status reflects actual state

### Test Category 2: Component Registration
- [ ] `test_register_component_success` - Component registered successfully
- [ ] `test_register_duplicate_fails` - Duplicate registration returns error
- [ ] `test_unregister_component_success` - Component unregistered successfully
- [ ] `test_unregister_nonexistent_safe` - Unregistering absent component is safe
- [ ] `test_register_after_unregister` - Can re-register after unregister

### Test Category 3: Address Resolution
- [ ] `test_resolve_registered_address` - Resolves registered component
- [ ] `test_resolve_unregistered_returns_none` - Returns None for unknown
- [ ] `test_resolve_after_unregister_none` - Returns None after unregister

### Test Category 4: Error Handling
- [ ] `test_route_to_nonexistent_logs_error` - Logs but doesn't crash
- [ ] `test_route_to_closed_mailbox_handled` - Graceful handling
- [ ] `test_invalid_message_type_error` - Clear error for invalid types

**Verification Command:**
```bash
cargo test --lib --package airssys-wasm -- messaging_subscription
```

---

## Integration Testing Plan

**MANDATORY**: Tests in `tests/` directory

**File:** `tests/messaging_subscription_integration_tests.rs` (NEW)

### Test 1: End-to-End Subscription Initialization
```rust
/// Proves MessagingSubscriptionService correctly initializes and subscribes to broker.
#[tokio::test]
async fn test_subscription_service_initialization() {
    // 1. Create broker
    // 2. Create subscription service
    // 3. Start service
    // 4. Verify service is running
    // 5. Verify subscriber count on broker increased
}
```

### Test 2: Component Registration During Spawn
```rust
/// Proves component registration integrates with subscription service.
#[tokio::test]
async fn test_component_registration_with_subscription_service() {
    // 1. Create subscription service
    // 2. Start service
    // 3. Register component with mailbox
    // 4. Publish message to broker targeting component
    // 5. Verify message arrives in component's mailbox
}
```

### Test 3: Message Routing Through Full Stack
```rust
/// End-to-end: broker publish → subscriber → mailbox → verification
#[tokio::test]
async fn test_full_stack_message_routing() {
    // 1. Create and start subscription service
    // 2. Register 3 components (A, B, C)
    // 3. Publish message targeting B
    // 4. Verify only B receives the message
    // 5. Verify A and C don't receive it
}
```

### Test 4: Graceful Shutdown During Message Flow
```rust
/// Proves shutdown doesn't lose in-flight messages (or handles gracefully).
#[tokio::test]
async fn test_graceful_shutdown_during_message_flow() {
    // 1. Create and start subscription service
    // 2. Register component
    // 3. Start publishing messages
    // 4. Call stop() during message flow
    // 5. Verify no panics, clean shutdown
}
```

### Test 5: Component Unregistration During Active Messaging
```rust
/// Proves unregistration is handled gracefully during active messaging.
#[tokio::test]
async fn test_unregister_during_active_messaging() {
    // 1. Create and start subscription service
    // 2. Register component
    // 3. Start message stream
    // 4. Unregister component
    // 5. Verify subsequent messages are dropped (logged, not crashed)
}
```

### Test 6: Multiple Concurrent Registrations
```rust
/// Stress test: concurrent registrations don't cause race conditions.
#[tokio::test]
async fn test_concurrent_registrations() {
    // 1. Create and start subscription service
    // 2. Spawn 20 tasks, each registering a component
    // 3. Verify all 20 registered
    // 4. Publish message to each
    // 5. Verify each receives its message
}
```

### Test 7: Integration with Existing Task 1.1/1.2 Tests
```rust
/// Proves Task 1.3 infrastructure works with Task 1.1 & 1.2 implementation.
#[tokio::test]
async fn test_integration_with_message_delivery_and_reception() {
    // Uses MessagingSubscriptionService
    // Uses ActorSystemSubscriber from Task 1.1
    // Uses ComponentActor message reception from Task 1.2
    // End-to-end flow with real WASM fixture
}
```

**Verification Commands:**
```bash
cargo test --test messaging_subscription_integration_tests
cargo test --test message_delivery_integration_tests
cargo test --test message_reception_integration_tests
```

---

## Quality Verification

- [ ] `cargo build` - builds cleanly
- [ ] `cargo test --lib` - all unit tests pass
- [ ] `cargo test --test messaging_subscription_integration_tests` - all integration tests pass
- [ ] `cargo test --test message_delivery_integration_tests` - existing tests pass
- [ ] `cargo test --test message_reception_integration_tests` - existing tests pass
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` - zero warnings
- [ ] Zero compiler warnings

---

## Verification Steps

### 1. Unit Test Verification
```bash
cd airssys-wasm
cargo test --lib -- messaging_subscription --nocapture
```
**Expected:** All unit tests passing (15+ tests)

### 2. Integration Test Verification
```bash
cd airssys-wasm
# Build WASM fixtures first
cd tests/fixtures && ./build.sh && cd ../..
cargo test --test messaging_subscription_integration_tests --nocapture
```
**Expected:** All integration tests passing (7+ tests)

### 3. Regression Test Verification
```bash
cd airssys-wasm
cargo test --test message_delivery_integration_tests
cargo test --test message_reception_integration_tests
```
**Expected:** All existing tests still passing

### 4. Build Verification
```bash
cd airssys-wasm
cargo build
cargo clippy --all-targets --all-features -- -D warnings
```
**Expected:** Zero warnings, clean build

### 5. Documentation Verification
- Module-level documentation present in new files
- Integration points documented
- Architecture diagram included

---

## Success Criteria Verification

| Success Criterion | How Verified |
|-------------------|--------------|
| ActorSystem successfully subscribes to MessageBroker | `test_subscription_service_initialization` passes |
| ComponentId → ActorAddress registry functional | `test_resolve_registered_address` passes |
| Message routing by ComponentId works correctly | `test_full_stack_message_routing` passes |
| Routing errors logged and handled gracefully | `test_route_to_nonexistent_logs_error` passes |
| Internal infrastructure documented clearly | Documentation exists in module files |

---

## Fixture Verification

**STATUS: READY** - All fixtures available

Existing fixtures from Task 1.1/1.2 are sufficient:
- ✅ `tests/fixtures/basic-handle-message.wat` (EXISTS)
- ✅ `tests/fixtures/rejecting-handler.wat` (EXISTS)
- ✅ `tests/fixtures/slow-handler.wat` (EXISTS)
- ✅ `tests/fixtures/no-handle-message.wat` (EXISTS)
- ✅ `tests/fixtures/build.sh` (EXISTS)

No new fixtures required for Task 1.3 - this task is infrastructure-focused.

---

## Risk Assessment

### Risk 1: Thread Safety with RwLock
**Impact:** Medium - Race conditions could cause message loss  
**Probability:** Low - RwLock pattern is well-tested in Task 1.1  
**Mitigation:** Use same patterns as Task 1.1, comprehensive concurrent tests

### Risk 2: Initialization Order Dependencies
**Impact:** Medium - Wrong order could cause missing subscriptions  
**Probability:** Medium - Multiple components need coordination  
**Mitigation:** Clear initialization sequence in documentation, startup tests

### Risk 3: Shutdown Coordination
**Impact:** Low - Could leave dangling tasks  
**Probability:** Low - Task abort patterns well-established  
**Mitigation:** Graceful shutdown tests, timeout on task abort

### Risk 4: Integration with Existing Task 1.1/1.2 Code
**Impact:** Medium - Could break existing functionality  
**Probability:** Low - Task 1.3 builds on, doesn't modify existing code  
**Mitigation:** Regression tests on existing test suites

---

## Estimated Effort Breakdown

| Step | Description | Hours |
|------|-------------|-------|
| 1 | Create MessagingSubscriptionService Module | 2 |
| 2 | Integrate with WasmRuntime Initialization | 2 |
| 3 | ComponentId → ActorAddress Registry Integration | 2 |
| 4 | Message Routing Logic Enhancement | 2 |
| 5 | Error Handling and Fallback | 2 |
| 6 | Documentation | 2 |
| **Total** | | **12 hours** |

---

## Dependencies

### Upstream (Completed)
- ✅ Task 1.1: MessageBroker Setup (mailbox_senders, register/unregister)
- ✅ Task 1.2: ComponentActor Message Reception (invoke_handle_message_with_timeout)

### Downstream (Blocked by this task)
- ⏳ Phase 2: Fire-and-Forget Messaging (needs subscription infrastructure)
- ⏳ Phase 3: Request-Response Pattern (needs routing infrastructure)

---

## Implementation Notes

### Key Patterns to Follow

1. **ADR-WASM-020 Compliance:**
   - `ActorSystemSubscriber` owns delivery (mailbox_senders)
   - `ComponentRegistry` stays pure (identity only)
   - New `MessagingSubscriptionService` coordinates both

2. **Error Handling Pattern:**
   - Log routing errors, don't crash
   - Dropped messages are acceptable (logged)
   - Clear error types for different failure modes

3. **Metrics Pattern:**
   - Use AtomicU64 for lock-free metrics (proven in Task 1.2)
   - Record routing count, failures, latency

4. **Test Quality:**
   - REAL integration tests (not stubs)
   - Prove end-to-end flow works
   - Test concurrent scenarios

---

**End of Action Plan for Task 1.3**
