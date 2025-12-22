# Technical Debt Record: DEBT-WASM-004

**Document ID:** DEBT-WASM-004-message-delivery-runtime-glue-missing  
**Created:** 2025-12-22  
**Updated:** 2025-12-22  
**Status:** active  
**Category:** DEBT-ARCH  

## Summary

The message delivery system between host functions (`runtime/`) and actor mailboxes (`actor/`) lacks critical runtime glue code. All components exist and are correctly implemented, but they are not wired together - messages published to `MessageBroker` never reach `ComponentActor::handle_message()`, and request-response callbacks are never invoked.

## Context

### Background

The airssys-wasm project implements a WASM-based Plugin/Extension Platform (similar to smart contracts on NEAR/Polkadot). The architecture defines:

- **Two root entities**: Host + Plugin/Component
- **Each WASM component = Actor** (managed by airssys-rt)
- **Communication via Actor Mailbox** (Erlang-style via airssys-rt)

The implementation followed a phased approach where individual components were built and tested in isolation:
- Phase 1-2: Built `WasmEngine`, host functions, capability system
- Phase 3-4: Built `ComponentActor`, `ActorSystemSubscriber`, messaging infrastructure
- Phase 5: Built request-response with `CorrelationTracker`

### Decision Point

Each phase focused on its specific deliverables without the final integration step that wires everything together. The individual components were tested with unit tests and mocked dependencies, but end-to-end integration was deferred.

### Constraints

1. **Time pressure**: Delivering individual phases on schedule
2. **Complexity**: Integration touches multiple layers (runtime, actor, messaging)
3. **Testing limitations**: Full integration tests require complete WASM components
4. **Phased development**: Each task had narrow scope, integration fell between phases

## Technical Details

### Code Location

| File | Description | Issue |
|------|-------------|-------|
| `airssys-wasm/src/actor/component/component_spawner.rs:266-311` | `spawn_component()` | Does NOT create/register mailbox channels |
| `airssys-wasm/src/actor/message/actor_system_subscriber.rs` | Message routing | `register_mailbox()` exists but never called |
| `airssys-wasm/src/runtime/async_host.rs:529-548` | `SendMessageHostFunction` | Publishes to broker, but no consumer |
| `airssys-wasm/src/runtime/async_host.rs:688-692` | `SendRequestHostFunction` | `response_rx` dropped immediately |

### Components Affected

- **ComponentSpawner**: Missing mailbox creation and registration
- **ActorSystemSubscriber**: Has infrastructure but not initialized
- **SendMessageHostFunction**: Publishes messages that never arrive
- **SendRequestHostFunction**: Response channel dropped, callbacks never fire
- **WasmEngine**: `call_handle_callback()` exists but is never called

### Current Implementation

#### What EXISTS and WORKS ✅

| Component | Location | Status |
|-----------|----------|--------|
| `WasmEngine` | `runtime/engine.rs` | ✅ Works |
| `call_handle_message()` | `runtime/engine.rs` | ✅ Works |
| `call_handle_callback()` | `runtime/engine.rs` | ✅ Works (but never called) |
| `ComponentActor::handle_message()` | `actor/component/actor_impl.rs` | ✅ Works |
| `ActorSystemSubscriber::register_mailbox()` | `actor/message/actor_system_subscriber.rs` | ✅ Works |
| `ActorSystemSubscriber::route_message_to_subscribers()` | `actor/message/actor_system_subscriber.rs` | ✅ Works |
| `SendMessageHostFunction` | `runtime/async_host.rs` | ✅ Works (publishes to broker) |
| `SendRequestHostFunction` | `runtime/async_host.rs` | ✅ Works (registers pending) |
| `CorrelationTracker` | `actor/message/correlation_tracker.rs` | ✅ Works |
| Security checks | `actor/component/actor_impl.rs` | ✅ Works |

#### What is MISSING ❌

| Gap | Location | Description |
|-----|----------|-------------|
| Mailbox creation | `component_spawner.rs` | No `mpsc::unbounded_channel()` creation |
| Mailbox registration | `component_spawner.rs` | No call to `subscriber.register_mailbox()` |
| Subscriber initialization | Integration layer | `ActorSystemSubscriber::start()` never called |
| Message loop | Integration layer | No task consuming from mailbox and calling `handle_message()` |
| Response listener | `async_host.rs:692` | `response_rx` is dropped immediately |
| Callback invocation | Integration layer | Nobody calls `call_handle_callback()` |

### Data Flow Analysis

```
CURRENT (BROKEN):
Component A calls send-message("B", payload)
        ↓
SendMessageHostFunction::execute()
        ↓ publishes
MessageBroker: stores message ✅
        ↓
❌ BREAK: Nobody subscribed!
   - ActorSystemSubscriber::start() never called
   - No mailboxes registered

EXPECTED (WORKING):
Component A calls send-message("B", payload)
        ↓
SendMessageHostFunction::execute()
        ↓ publishes
MessageBroker ← ActorSystemSubscriber (subscribed)
        ↓ routes via mailbox_senders
Component B's mailbox (UnboundedReceiver)
        ↓ received by message loop
ComponentActor B::handle_message(InterComponent{...})
        ↓ invokes
WasmEngine::call_handle_message(sender, payload)
        ↓
WASM: handle-message(sender, payload) ✅
```

### Impact Assessment

- **Performance Impact:** None currently (feature doesn't work at all)
- **Maintainability Impact:** Medium - Integration code would touch multiple modules
- **Security Impact:** None - Security checks are implemented, just not reached
- **Scalability Impact:** None currently - System doesn't function
- **Functional Impact:** **CRITICAL** - Inter-component messaging completely non-functional

## Remediation Plan

### Ideal Solution

Wire the existing components together with ~50-100 lines of integration code:

1. `ComponentSpawner::spawn_component()` creates mailbox channel and registers it
2. `ActorSystemSubscriber::start()` is called during runtime initialization
3. Message loop task consumes from mailbox and calls `handle_message()`
4. `SendRequestHostFunction` spawns task to await response and call `call_handle_callback()`

### Implementation Steps

#### Step 1: Fix ComponentSpawner (~30 lines)

**File:** `airssys-wasm/src/actor/component/component_spawner.rs`

```rust
pub async fn spawn_component(
    &self,
    component_id: ComponentId,
    _wasm_path: PathBuf,
    metadata: ComponentMetadata,
    capabilities: CapabilitySet,
) -> Result<ActorAddress, WasmError> {
    // 1-3. (existing code) ✅
    
    // NEW: Create mailbox channel
    let (mailbox_tx, mailbox_rx) = tokio::sync::mpsc::unbounded_channel();
    
    // NEW: Register mailbox with ActorSystemSubscriber
    self.actor_system_subscriber
        .register_mailbox(component_id.clone(), mailbox_tx)
        .await?;
    
    // NEW: Store receiver in actor for message loop
    actor.set_mailbox_receiver(mailbox_rx);
    
    // 4-5. (existing code) ✅
    
    Ok(actor_ref)
}
```

#### Step 2: Add Message Loop (~20 lines)

**File:** `airssys-wasm/src/actor/component/component_actor.rs`

```rust
impl ComponentActor {
    /// Start the message processing loop.
    pub async fn run_message_loop(&mut self) {
        while let Some(msg) = self.mailbox_rx.recv().await {
            if let Err(e) = self.handle_message(msg, &mut self.ctx).await {
                tracing::error!(
                    component_id = %self.component_id.as_str(),
                    error = %e,
                    "Error processing message"
                );
            }
        }
    }
}
```

#### Step 3: Fix Response Channel (~25 lines)

**File:** `airssys-wasm/src/runtime/async_host.rs`

```rust
// In SendRequestHostFunction::execute()

// 5. Create oneshot channel for response
let (response_tx, response_rx) = oneshot::channel::<ResponseMessage>();

// ... existing registration code ...

// NEW: Spawn response listener task
let engine = self.engine.clone();
let component_id = context.component_id.clone();
let correlation_id_clone = correlation_id;
let timeout_duration = Duration::from_millis(timeout_ms);

tokio::spawn(async move {
    match tokio::time::timeout(timeout_duration, response_rx).await {
        Ok(Ok(response)) => {
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
            tracing::warn!(correlation_id = %correlation_id_clone, "Response channel closed");
        }
        Err(_) => {
            tracing::warn!(correlation_id = %correlation_id_clone, "Request timed out");
        }
    }
});

// 9. Return request_id
Ok(correlation_id.to_string().into_bytes())
```

#### Step 4: Initialize Subscriber (~10 lines)

**File:** `airssys-wasm/src/runtime/mod.rs` or initialization code

```rust
// During WasmRuntime initialization
pub async fn initialize(&mut self) -> Result<(), WasmError> {
    // ... existing initialization ...
    
    // NEW: Start the message subscriber
    self.actor_system_subscriber.start().await?;
    
    Ok(())
}
```

### Effort Estimate

- **Development Time:** 4-8 hours
- **Testing Time:** 4-8 hours (integration tests)
- **Risk Level:** Medium (touches multiple modules, but changes are additive)

### Dependencies

1. **None blocking** - All required infrastructure exists
2. **Test fixtures** - May need simple WASM components for integration tests

## Tracking

### GitHub Issue

- **Issue:** To be created
- **Labels:** `technical-debt`, `architecture`, `critical`, `messaging`

### Workspace Standards

- **Standards Violated:** 
  - Integration testing requirements (end-to-end tests missing)
  - Functional completeness (feature advertised but non-functional)
- **Compliance Impact:** High - Core messaging feature is non-functional

### Priority

- **Business Priority:** **CRITICAL** - Inter-component messaging is core functionality
- **Technical Priority:** **HIGH** - Blocks all messaging-dependent features
- **Recommended Timeline:** Immediate - Next development sprint

## History

### Changes

- **2025-12-22:** Initial documentation created based on deep architecture analysis

### Related Decisions

- **ADR References:**
  - ADR-WASM-009: Component Communication Model
  - ADR-WASM-020: Message Delivery Ownership
- **Knowledge Documents:**
  - KNOWLEDGE-WASM-031: Foundational Architecture
  - KNOWLEDGE-WASM-026: Message Delivery Architecture
- **Other Debt:** None directly related

## Resolution

*[To be filled when resolved]*

### Resolution Date
[Pending]

### Resolution Summary
[Pending]

### Lessons Learned
[Pending]

---

## Appendix: Verification Commands

After remediation, verify with:

```bash
# Unit tests
cargo test --package airssys-wasm --lib

# Integration test (to be created)
cargo test --package airssys-wasm --test message_delivery_integration

# Build verification
cargo build --package airssys-wasm

# Clippy
cargo clippy --package airssys-wasm --all-targets --all-features -- -D warnings
```

## Appendix: Test Goal

Create integration test proving end-to-end message flow:

```rust
#[tokio::test]
async fn test_component_a_sends_message_to_component_b() {
    // 1. Initialize runtime with two components
    // 2. Component A calls send-message("B", payload)
    // 3. Verify Component B's handle-message receives the message
    // 4. Verify correct sender, payload data
}

#[tokio::test]
async fn test_request_response_with_callback() {
    // 1. Initialize runtime with two components
    // 2. Component A calls send-request("B", request, timeout)
    // 3. Component B handles request and sends response
    // 4. Verify Component A's handle-callback receives response
    // 5. Verify correlation ID matches
}
```

---
**Template Version:** 1.0  
**Last Updated:** 2025-12-22
