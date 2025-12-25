# Implementation Plan for WASM-TASK-HOTFIX-001

**Task ID:** WASM-TASK-HOTFIX-001
**Created:** 2025-12-25
**Status:** NOT STARTED
**Priority:** 🔴 CRITICAL / BLOCKING

---

## ADR/Knowledge References (MANDATORY)

### ADRs Referenced:
- **ADR-WASM-018**: Three-Layer Architecture and Boundary Definitions - Defines correct dependency flow (actor/ → runtime/ → core/)
- **ADR-WASM-020**: Message Delivery Ownership Architecture - Defines ActorSystemSubscriber owns delivery, ComponentRegistry stays pure
- **ADR-WASM-021**: Duplicate WASM Runtime Remediation - Mandates Component Model usage, removal of duplicate runtime
- **ADR-WASM-022**: Circular Dependency Remediation - Defines fix for runtime/ → actor/ imports
- **ADR-WASM-023**: Module Boundary Enforcement - MANDATORY module dependency rules

### Knowledges Referenced:
- **KNOWLEDGE-WASM-018**: Component Definitions and Three-Layer Architecture - Detailed component definition and layer responsibilities
- **KNOWLEDGE-WASM-026**: Message Delivery Architecture - Final Decision - Complete message flow and mailbox registration design
- **KNOWLEDGE-WASM-027**: Duplicate WASM Runtime - Fatal Architecture Violation - Documents the violation to be fixed
- **KNOWLEDGE-WASM-028**: Circular Dependency Between actor/ and runtime/ - Documents the circular dependency issue
- **KNOWLEDGE-WASM-030**: Module Architecture - Hard Requirements - MANDATORY module architecture requirements
- **KNOWLEDGE-WASM-032**: Module Boundary Violations Audit - Current state of violations

---

## Module Architecture Verification (MANDATORY for airssys-wasm)

### The Four Modules and Their Dependencies:

```
┌─────────────────────────────────────────────────────────┐
│                                                         │
│   actor/  ──────►  runtime/  ──────►  security/  ──────►  core/
│     │                 │                   │                │
│     │                 │                   │                │
│     └─────────────────┴───────────────────┴────────────────┘
│                           ALL import from core/
│
└─────────────────────────────────────────────────────────┘
```

### Forbidden Imports (ADR-WASM-023):
- ❌ `runtime/` → `actor/` (CREATES CIRCULAR DEPENDENCY)
- ❌ `security/` → `runtime/` or `actor/` (SECURITY MUST BE INDEPENDENT)
- ❌ `core/` → ANY MODULE (CORE IS FOUNDATION, NO DEPENDENCIES)

### Module Responsibilities (KNOWLEDGE-WASM-030):

| Module | Purpose | OWNS | DOES NOT OWN |
|--------|---------|------|--------------|
| `core/` | Shared types & abstractions | ComponentId, ComponentMessage, PendingRequest, ResponseMessage, CorrelationId, CorrelationTracker, RequestError, traits, errors, configs | Any implementation logic, any imports from other modules |
| `security/` | Security logic | Capabilities, permissions, policies, validation | WASM execution, messaging, actor lifecycle |
| `runtime/` | WASM execution engine | WasmEngine, ComponentLoader, StoreManager, host function definitions, resource limits, WASM memory management | Message routing, actor lifecycle, correlation tracking, response routing, mailbox management, subscription management |
| `actor/` | Actor system integration | ComponentActor, ComponentRegistry, ComponentSpawner, ActorSystemSubscriber, CorrelationTracker, ResponseRouter, PendingRequest, ResponseMessage, RequestError types, ALL inter-component messaging orchestration, supervision, health monitoring | WASM execution engine (uses runtime/), WASM loading (uses runtime/), host function execution (uses runtime/) |

### Code Placement by Phase:

**Phase 1: Fix Circular Dependency**
- `src/core/message.rs` (NEW) - Shared messaging data types
- All messaging types moved from `actor/` to `core/`
- `MessagingSubscriptionService` moved to `actor/message/`

**Phase 2: Fix Duplicate WASM Runtime**
- `src/actor/component/component_actor.rs` - Updated to require Arc<WasmEngine>
- `src/actor/component/child_impl.rs` - Updated to use WasmEngine API
- `src/actor/component/actor_impl.rs` - Updated to use Component Model calls

**Phase 3: Implement Missing Integration Glue**
- `src/actor/message/actor_system_subscriber.rs` - Add mailbox_senders map and register methods
- `src/actor/component/component_spawner.rs` - Create mailbox channels and register
- `src/actor/component/component_actor.rs` - Add run_message_loop() method

### Verification Commands (MUST PASS AFTER EACH PHASE):

```bash
# Check 1: core/ has no forbidden imports
grep -rn "use crate::runtime" src/core/
grep -rn "use crate::actor" src/core/
grep -rn "use crate::security" src/core/

# Check 2: security/ has no forbidden imports
grep -rn "use crate::runtime" src/security/
grep -rn "use crate::actor" src/security/

# Check 3: runtime/ has no forbidden imports
grep -rn "use crate::actor" src/runtime/

# ALL MUST RETURN NOTHING for architecture to be valid
```

---

## Context & References

### Adheres to:
- **ADR-WASM-018**: Three-Layer Architecture
- **ADR-WASM-020**: Message Delivery Ownership Architecture
- **ADR-WASM-021**: Duplicate WASM Runtime Remediation
- **ADR-WASM-022**: Circular Dependency Remediation
- **ADR-WASM-023**: Module Boundary Enforcement

### Relevant Patterns from system-patterns.md:
- Component Communication Pattern (actor-based message passing)
- Component Host Pattern (WasmEngine integration)

### Technical Context from tech-context.md:
- WASMtime as primary runtime with Component Model support
- Erlang-style lightweight process isolation using airssys-rt actors
- Capability-based security with fine-grained permissions

---

## Phase-by-Phase Implementation Plan

### Phase 1: Fix Circular Dependency (Days 1-2)

#### Task 1.1: Move Messaging Types to Core/

**Objective:** Move shared messaging types from actor/ to core/ to eliminate circular imports.

**Files to Create:**
- `src/core/message.rs` (NEW) - Contains:
  - `PendingRequest` struct
  - `ResponseMessage` struct
  - `CorrelationId` type (alias for String)
  - `CorrelationTracker` struct
  - `RequestError` enum
  - `ResponseRouterStats` struct (move from runtime/messaging.rs)
  - `ResponseRouter` struct (move from runtime/messaging.rs)

**Files to Update:**
- `src/core/mod.rs` - Add `pub mod message;` and export types

**Files to Update Imports (runtime/):**
- `src/runtime/async_host.rs` line 52
  - Change: `use crate::actor::message::{...}` → `use crate::core::message::{...}`
- `src/runtime/messaging.rs` line 76
  - Change: `use crate::actor::message::{...}` → `use crate::core::message::{...}`

**Files to Update Imports (actor/):**
- `src/actor/message/mod.rs` - Update imports from core/
- `src/actor/component/component_actor.rs` - Update imports from core/
- `src/actor/message/correlation_tracker.rs` - Update imports from core/

**Success Criteria:**
- ✅ All messaging types exported from core/message.rs
- ✅ runtime/ no longer imports from actor/
- ✅ `grep -r "use crate::actor" src/runtime/` returns nothing
- ✅ `cargo build` succeeds
- ✅ All tests pass

**Unit Testing Plan:**
**File:** `src/core/message.rs` module tests
- Test file location: `src/core/message.rs` in `#[cfg(test)]` block
- [ ] Test `PendingRequest` struct creation and fields
- [ ] Test `ResponseMessage` struct creation and fields
- [ ] Test `CorrelationId` type alias
- [ ] Test `CorrelationTracker::new()` initialization
- [ ] Test `CorrelationTracker::register()` pending request registration
- [ ] Test `CorrelationTracker::resolve()` pending request resolution
- [ ] Test `CorrelationTracker::timeout_expired()` timeout handling
- [ ] Test `RequestError` enum variants
- [ ] Test `ResponseRouterStats` struct
- [ ] Test `ResponseRouter::new()` initialization
- [ ] Test `ResponseRouter::route_response()` routing logic
- [ ] Test `ResponseRouter::cleanup_expired()` cleanup logic
**Verification:** `cargo test --lib core::message::` - all tests passing

**Integration Testing Plan:**
**File:** `tests/messaging-types-integration-tests.rs` (NEW)
- [ ] Test shared types work across module boundaries
- [ ] Test CorrelationTracker used by runtime and actor
- [ ] Test ResponseRouter cross-module functionality
**Verification:** `cargo test --test messaging-types-integration-tests` - all tests passing

**Architecture Verification:**
```bash
# After task 1.1 completion:
grep -rn "use crate::actor" src/runtime/
# Expected: Empty result (no imports from actor/)
```

---

#### Task 1.2: Move MessagingSubscriptionService

**Objective:** Move messaging subscription logic from runtime/ to actor/message/ where it belongs.

**Deliverables:**

**Files to Move:**
- `src/runtime/messaging_subscription.rs` → `src/actor/message/messaging_subscription.rs`

**Files to Update:**
- `src/runtime/mod.rs` - Remove `pub mod messaging_subscription;`
- `src/actor/message/mod.rs` - Add `pub mod messaging_subscription;`
- Update internal imports as needed

**Success Criteria:**
- ✅ MessagingSubscriptionService in actor/message/
- ✅ runtime/ no longer has actor-level subscription code
- ✅ All tests pass
- ✅ `cargo build` succeeds
- ✅ `cargo test` passes

**Unit Testing Plan:**
**File:** `src/actor/message/messaging_subscription.rs` module tests
- [ ] Test `MessagingSubscriptionService::new()` initialization
- [ ] Test `MessagingSubscriptionService::subscribe()` subscription logic
- [ ] Test `MessagingSubscriptionService::unsubscribe()` unsubscription logic
- [ ] Test `MessagingSubscriptionService::get_subscribers()` retrieval
- [ ] Test error cases (invalid component, already subscribed)
**Verification:** `cargo test --lib actor::message::messaging_subscription::` - all tests passing

**Integration Testing Plan:**
**File:** `tests/messaging-subscription-integration-tests.rs` (NEW)
- [ ] Test subscription/unsubscription end-to-end
- [ ] Test multiple subscribers to same topic
- [ ] Test subscription persistence across operations
**Verification:** `cargo test --test messaging-subscription-integration-tests` - all tests passing

**Architecture Verification:**
```bash
# After task 1.2 completion:
grep -rn "use crate::actor" src/runtime/
# Expected: Empty result (no imports from actor/)
```

---

#### Task 1.3: Add CI Layer Dependency Enforcement

**Objective:** Create CI script to prevent future circular dependency violations.

**Files to Create:**
- `.github/scripts/check-layer-deps.sh` (NEW CI script)

**CI Script Content:**
```bash
#!/bin/bash
set -e

echo "🔍 Checking layer dependencies..."

# Check 1: runtime/ must NOT import from actor/
echo "  Checking runtime/ → actor/ (should be NONE)..."
if grep -rq "use crate::actor" src/runtime/ 2>/dev/null; then
    echo "❌ ERROR: runtime/ imports from actor/"
    grep -rn "use crate::actor" src/runtime/
    exit 1
fi
echo "  ✅ runtime/ clean"

# Check 2: core/ must NOT import from runtime/ or actor/
echo "  Checking core → higher layers (should be NONE)..."
if grep -rq "use crate::runtime\|use crate::actor" src/core/ 2>/dev/null; then
    echo "❌ ERROR: core/ imports from higher layers"
    grep -rn "use crate::runtime\|use crate::actor" src/core/
    exit 1
fi
echo "  ✅ core clean"

# Check 3: security/ must NOT import from runtime/ or actor/
echo "  Checking security → higher layers (should be NONE)..."
if grep -rq "use crate::runtime\|use crate::actor" src/security/ 2>/dev/null; then
    echo "❌ ERROR: security/ imports from higher layers"
    grep -rn "use crate::runtime\|use crate::actor" src/security/
    exit 1
fi
echo "  ✅ security clean"

echo ""
echo "✅ All layer dependency checks passed!"
```

**Files to Update:**
- `.github/workflows/ci.yml` - Add layer dependency check step

**Success Criteria:**
- ✅ CI script created and executable
- ✅ Integrated into CI workflow
- ✅ Prevents future circular dependency violations
- ✅ All existing code passes checks

**Verification:**
```bash
# Run the CI script manually to verify:
chmod +x .github/scripts/check-layer-deps.sh
.github/scripts/check-layer-deps.sh
# Expected: All checks pass
```

---

### Phase 2: Fix Duplicate WASM Runtime (Days 2-3)

#### Task 2.1: Make WasmEngine Mandatory in ComponentActor

**Objective:** Change ComponentActor to require Arc<WasmEngine> instead of optional.

**File to Update:** `src/actor/component/component_actor.rs`

**Changes:**
```rust
// BEFORE (WRONG - lines around 486-487):
    component_engine: Option<Arc<WasmEngine>>,
    component_handle: Option<ComponentHandle>,

// AFTER (CORRECT):
    engine: Arc<WasmEngine>,        // Required parameter
    component_handle: Option<ComponentHandle>,  // Loaded after start()
```

**Method Updates:**
- Update `ComponentActor::new()` to require `engine: Arc<WasmEngine>` parameter
- Update `ComponentActor::with_component_engine()` to be primary constructor
- Update `ComponentActor::component_engine()` to return `&Arc<WasmEngine>`
- Update `ComponentActor::component_handle()` to return `Option<&ComponentHandle>`
- Update `ComponentActor::uses_component_model()` to always return true

**Success Criteria:**
- ✅ ComponentActor requires WasmEngine to construct
- ✅ Cannot create ComponentActor without providing engine
- ✅ `grep "component_engine: Option" src/actor/component/` returns nothing
- ✅ All ComponentActor tests updated to provide engine
- ✅ `cargo build` succeeds
- ✅ `cargo test` passes

**Unit Testing Plan:**
**File:** `src/actor/component/component_actor.rs` module tests
- [ ] Test `ComponentActor::new()` requires WasmEngine parameter
- [ ] Test `ComponentActor::new()` fails without WasmEngine
- [ ] Test `ComponentActor::engine()` returns injected WasmEngine
- [ ] Test `ComponentActor::component_handle()` returns handle when loaded
- [ ] Test `ComponentActor::uses_component_model()` always returns true
- [ ] Test ComponentActor construction with valid WasmEngine
- [ ] Test ComponentActor fields are properly initialized
**Verification:** `cargo test --lib actor::component::component_actor::` - all tests passing

**Integration Testing Plan:**
**File:** `tests/component-actor-engine-injection-integration-tests.rs` (NEW)
- [ ] Test ComponentActor spawned with WasmEngine
- [ ] Test ComponentActor can load WASM component
- [ ] Test ComponentActor handle_message uses injected engine
- [ ] Test ComponentActor lifecycle with engine dependency
**Verification:** `cargo test --test component-actor-engine-injection-integration-tests` - all tests passing

**Architecture Verification:**
```bash
# After task 2.1 completion:
grep -rn "component_engine: Option" src/actor/component/
# Expected: Empty result (no optional engine field)
```

---

#### Task 2.2: Update Child::start() to Use WasmEngine

**Objective:** Rewrite ComponentActor's Child trait implementation to use WasmEngine instead of core WASM API.

**File to Update:** `src/actor/component/child_impl.rs`

**Changes:**
- Remove all `wasmtime::{Config, Engine, Module, Store}` imports (if any remain)
- Remove all core WASM Module, Linker, Instance usage
- Use `self.engine.load_component(&component_id, &wasm_bytes).await?` instead
- Use `self.component_handle = Some(handle)` to store result
- Update error messages to reference WasmEngine

**Success Criteria:**
- ✅ `grep -rn "wasmtime::Module" src/actor/` returns nothing
- ✅ `grep -rn "wasmtime::Engine" src/actor/` returns nothing
- ✅ `grep -rn "wasmtime::Store" src/actor/` returns nothing
- ✅ Child::start() uses WasmEngine::load_component()
- ✅ `cargo build` succeeds
- ✅ `cargo test` passes

**Unit Testing Plan:**
**File:** `src/actor/component/child_impl.rs` module tests
- [ ] Test `Child::start()` loads component via WasmEngine
- [ ] Test `Child::start()` stores component handle
- [ ] Test `Child::start()` returns error on invalid WASM
- [ ] Test `Child::start()` returns error on capability violation
- [ ] Test `Child::stop()` cleans up component
- [ ] Test component lifecycle (start, handle_message, stop)
**Verification:** `cargo test --lib actor::component::child_impl::` - all tests passing

**Integration Testing Plan:**
**File:** `tests/component-loading-integration-tests.rs` (NEW)
- [ ] Test end-to-end component loading with WasmEngine
- [ ] Test component loads WIT interfaces correctly
- [ ] Test component handle_message function accessible
- [ ] Test component cleanup after stop
**Verification:** `cargo test --test component-loading-integration-tests` - all tests passing

**Architecture Verification:**
```bash
# After task 2.2 completion:
grep -rn "wasmtime::Module\|wasmtime::Engine\|wasmtime::Store" src/actor/
# Expected: Empty result (no core WASM API usage)
```

---

#### Task 2.3: Update Actor::handle() to Use Component Model API

**File to Update:** `src/actor/component/actor_impl.rs`

**Changes:**
- Remove any remaining `WasmBumpAllocator` references
- Remove any remaining `HandleMessageParams` references
- Remove any remaining `HandleMessageResult` references
- Use `self.engine.call_handle_message(handle, &msg.sender, &msg.payload).await?` instead
- Update error handling to use WasmEngine errors

**Success Criteria:**
- ✅ No bump allocator usage
- ✅ No manual parameter marshalling
- ✅ Uses Component Model typed calls via WasmEngine
- ✅ Generated WIT bindings are actively used
- ✅ `grep "WasmBumpAllocator" src/actor/component/` returns nothing
- ✅ `grep "HandleMessageParams" src/actor/component/` returns nothing
- ✅ `grep "HandleMessageResult" src/actor/component/` returns nothing
- ✅ `cargo build` succeeds
- ✅ `cargo test` passes

**Unit Testing Plan:**
**File:** `src/actor/component/actor_impl.rs` module tests
- [ ] Test `Actor::handle()` calls handle_message via WasmEngine
- [ ] Test `Actor::handle()` with ComponentMessage::InterComponent
- [ ] Test `Actor::handle()` with ComponentMessage::InterComponentWithCorrelation
- [ ] Test `Actor::handle()` with ComponentMessage::SystemMessage
- [ ] Test `Actor::handle()` returns error on WASM failure
- [ ] Test `Actor::handle()` returns error on timeout
**Verification:** `cargo test --lib actor::component::actor_impl::` - all tests passing

**Integration Testing Plan:**
**File:** `tests/component-message-handling-integration-tests.rs` (NEW)
- [ ] Test ComponentActor receives message via mailbox
- [ ] Test ComponentActor invokes handle_message on WASM
- [ ] Test ComponentActor returns correct response
- [ ] Test ComponentActor handles correlation tracking
- [ ] Test ComponentActor handles request-response pattern
**Verification:** `cargo test --test component-message-handling-integration-tests` - all tests passing

**Architecture Verification:**
```bash
# After task 2.3 completion:
grep -rn "WasmBumpAllocator\|HandleMessageParams\|HandleMessageResult" src/actor/component/
# Expected: Empty result (no workaround code)
```

---

#### Task 2.4: Extend WasmEngine if Needed

**Objective:** Ensure WasmEngine has all methods needed by ComponentActor.

**File to Update:** `src/runtime/engine.rs`

**Potential Additions:**
- Verify `call_handle_message()` exists
- Verify `call_handle_callback()` exists
- Add `pub async fn call_handle_message(...)` if missing
- Add `pub async fn call_handle_callback(...)` if missing
- Update documentation

**Success Criteria:**
- ✅ WasmEngine provides all required methods
- ✅ Generated bindings are actively used via WasmEngine
- ✅ `cargo build` succeeds
- ✅ `cargo test` passes

**Unit Testing Plan:**
**File:** `src/runtime/engine.rs` module tests
- [ ] Test `WasmEngine::call_handle_message()` execution
- [ ] Test `WasmEngine::call_handle_callback()` execution
- [ ] Test `WasmEngine::call_handle_message()` returns error on invalid handle
- [ ] Test `WasmEngine::call_handle_callback()` returns error on invalid correlation
- [ ] Test timeout handling in handle_message
- [ ] Test timeout handling in handle_callback
**Verification:** `cargo test --lib runtime::engine::` - all tests passing

**Architecture Verification:**
```bash
# After task 2.4 completion:
# Verify no actor/ code implements runtime functionality
grep -rn "impl.*Engine\|impl.*Loader" src/actor/
# Expected: Empty result (only runtime/ implements engine logic)
```

---

#### Task 2.5: Update All ComponentActor Tests

**Objective:** Update all ComponentActor tests to provide Arc<WasmEngine>.

**Files to Update:**
- All test files in `tests/` and `src/actor/component/*_tests.rs`

**Changes:**
- All `ComponentActor::new()` calls must provide `Arc<WasmEngine>`
- All fixture setups must provide engine
- Update test assertions to verify engine usage

**Success Criteria:**
- ✅ All ComponentActor tests provide engine
- ✅ `cargo test --lib` passes
- ✅ Zero test failures
- ✅ `cargo clippy` clean

**Unit Testing Plan:**
All existing unit tests updated:
- [ ] All `ComponentActor` tests compile with WasmEngine parameter
- [ ] All tests pass with new ComponentActor API
- [ ] All tests verify correct engine usage
**Verification:** `cargo test --lib` - all tests passing

**Integration Testing Plan:**
All existing integration tests updated:
- [ ] All component tests provide valid WasmEngine
- [ ] All tests pass with new architecture
- [ ] All tests verify Component Model API usage
**Verification:** `cargo test --test '*'` - all tests passing

---

### Phase 3: Implement Missing Integration Glue (Days 1.5-2)

#### Task 3.1: Fix ComponentSpawner to Create Mailbox Channels

**Objective:** Enable ComponentSpawner to create mailbox channels and register them.

**File to Update:** `src/actor/component/component_spawner.rs`

**Changes:**
```rust
// In spawn_component() method, ADD:
    // Step 3: Create mailbox channel
    let (mailbox_tx, mailbox_rx) = tokio::sync::mpsc::unbounded_channel::<ComponentMessage>();

    // Step 4: Register mailbox with ActorSystemSubscriber
    if let Some(subscriber) = &self.actor_system_subscriber {
        subscriber.register_mailbox(component_id.clone(), mailbox_tx).await?;
    }

    // Step 5: Store receiver in actor
    actor.set_mailbox_receiver(mailbox_rx);
```

**Add Dependencies if needed:**
- Import `tokio::sync::mpsc` at top of file
- Import `ComponentMessage` from `crate::core::message`

**Success Criteria:**
- ✅ `grep "mpsc::unbounded_channel" src/actor/component/component_spawner.rs` shows usage
- ✅ `grep "register_mailbox" src/actor/component/component_spawner.rs` shows usage
- ✅ `grep "set_mailbox_receiver" src/actor/component/component_actor.rs` shows usage
- ✅ `cargo build` succeeds
- ✅ `cargo test` passes

**Unit Testing Plan:**
**File:** `src/actor/component/component_spawner.rs` module tests
- [ ] Test `ComponentSpawner::spawn_component()` creates mailbox channel
- [ ] Test `ComponentSpawner::spawn_component()` registers mailbox with ActorSystemSubscriber
- [ ] Test `ComponentSpawner::spawn_component()` sets mailbox receiver on actor
- [ ] Test `ComponentSpawner::spawn_component()` returns ActorAddress
- [ ] Test `ComponentSpawner::stop_component()` unregisters mailbox
- [ ] Test error handling when subscriber unavailable
**Verification:** `cargo test --lib actor::component::component_spawner::` - all tests passing

**Integration Testing Plan:**
**File:** `tests/component-spawning-integration-tests.rs` (NEW)
- [ ] Test end-to-end component spawning with mailbox
- [ ] Test mailbox registered with ActorSystemSubscriber
- [ ] Test messages can be delivered to spawned component
- [ ] Test component cleanup unregisters mailbox
**Verification:** `cargo test --test component-spawning-integration-tests` - all tests passing

**Architecture Verification:**
```bash
# After task 3.1 completion:
grep -rn "mpsc::unbounded_channel\|register_mailbox" src/actor/component/component_spawner.rs
# Expected: Both patterns found (mailbox creation and registration)
```

---

#### Task 3.2: Add Message Loop to ComponentActor

**Objective:** Enable ComponentActor to consume messages from mailbox and process them.

**File to Update:** `src/actor/component/component_actor.rs`

**Changes:**
```rust
// Add to ComponentActor struct:
    mailbox_rx: Option<UnboundedReceiver<ComponentMessage>>,

// Add public async method:
    pub async fn run_message_loop(&mut self) {
        while let Some(msg) = self.mailbox_rx.as_mut().unwrap().recv().await {
            if let Err(e) = self.handle_message(msg, &mut self.ctx).await {
                tracing::error!(
                    component_id = %self.component_id.as_str(),
                    error = %e,
                    "Error processing message"
                );
            }
        }
    }

// Update Child trait implementation to start message loop:
impl<S> Child for ComponentActor<S> {
    async fn start(&mut self) -> Result<(), Self::Error> {
        // ... existing start logic ...

        // ADD: Start message loop
        tokio::spawn(async move {
            if let Some(mut actor) = Arc::try_unwrap(actor) {
                if let Err(e) = actor.run_message_loop().await {
                    tracing::error!("Message loop error: {e}");
                }
            }
        });

        Ok(())
    }
}
```

**Success Criteria:**
- ✅ `grep "mailbox_rx: Option" src/actor/component/component_actor.rs` shows field added
- ✅ `grep "run_message_loop" src/actor/component/component_actor.rs` shows method added
- ✅ Message loop calls `handle_message()`
- ✅ `cargo build` succeeds
- ✅ `cargo test` passes
- ✅ Integration tests prove end-to-end messaging works

**Unit Testing Plan:**
**File:** `src/actor/component/component_actor.rs` module tests
- [ ] Test `ComponentActor::run_message_loop()` processes messages
- [ ] Test `ComponentActor::run_message_loop()` stops on channel close
- [ ] Test `ComponentActor::run_message_loop()` handles errors
- [ ] Test `Child::start()` spawns message loop task
- [ ] Test mailbox receiver is properly stored
- [ ] Test message loop receives and processes messages
**Verification:** `cargo test --lib actor::component::component_actor::` - all tests passing

**Integration Testing Plan:**
**File:** `tests/message-loop-integration-tests.rs` (NEW)
- [ ] Test message loop receives messages from mailbox
- [ ] Test message loop processes messages with handle_message
- [ ] Test message loop handles multiple sequential messages
- [ ] Test message loop handles concurrent messages
- [ ] Test message loop handles errors gracefully
- [ ] Test message loop stops when component stopped
**Verification:** `cargo test --test message-loop-integration-tests` - all tests passing

**Architecture Verification:**
```bash
# After task 3.2 completion:
grep -rn "run_message_loop\|mailbox_rx: Option" src/actor/component/component_actor.rs
# Expected: Both patterns found (message loop implementation)
```

---

#### Task 3.3: Verify End-to-End Message Flow

**Objective:** Create comprehensive integration test proving messages flow from A → B.

**File to Create:** `tests/block5_messaging_e2e_integration_tests.rs` (NEW)

**Test Cases:**
```rust
#[tokio::test]
async fn test_component_a_sends_to_component_b_real() {
    // 1. Initialize runtime with two components
    let runtime = WasmRuntime::new(config);

    // 2. Load Component A
    let component_a = runtime.load_component(
        include_bytes!("fixtures/basic-handle-message.wasm"),
        caps
    ).await?;

    // 3. Load Component B
    let component_b = runtime.load_component(
        include_bytes!("fixtures/basic-handle-message.wasm"),
        caps
    ).await?;

    // 4. Component A sends message
    component_a.send_message("component-b", b"hello").await?;

    // 5. Wait for delivery
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 6. Verify Component B received it
    let messages_b = component_b.received_messages().lock().unwrap();
    assert_eq!(messages_b.len(), 1);
    assert_eq!(messages_b[0].payload, b"hello");
}
```

**Additional Test Cases:**
- [ ] Test fire-and-forget messaging (A → B)
- [ ] Test request-response messaging (A → B with callback)
- [ ] Test multiple components (A → B → C)
- [ ] Test concurrent messaging (multiple senders to B)
- [ ] Test message to non-existent component (error handling)
- [ ] Test large payload handling
- [ ] Test component crash during message processing

**Success Criteria:**
- ✅ Integration test creates and runs successfully
- ✅ Message flows from A → B end-to-end
- ✅ Test proves mailbox delivery works
- ✅ `cargo test --test block5_messaging_e2e` passes
- ✅ All other tests still pass

**Fixtures Required:**
- ✅ `basic-handle-message.wasm` - EXISTS
- ✅ `handle-message-component.wasm` - EXISTS
- ✅ `callback-receiver-component.wasm` - EXISTS

**Verification:** `cargo test --test block5_messaging_e2e` - all tests passing

---

#### Task 3.4: Update ActorSystemSubscriber Initialization

**Objective:** Ensure ActorSystemSubscriber is started when runtime initializes.

**File to Update:** `src/runtime/mod.rs` or initialization code

**Changes:**
- In WasmRuntime initialization or similar, call `subscriber.start().await?`
- Ensure subscriber is started before components begin messaging

**Success Criteria:**
- ✅ ActorSystemSubscriber is started
- ✅ `grep "start()" src/actor/message/messaging_subscription.rs` is called
- ✅ `cargo build` succeeds
- ✅ `cargo test` passes

**Unit Testing Plan:**
**File:** `src/runtime/mod.rs` module tests
- [ ] Test WasmRuntime initialization starts ActorSystemSubscriber
- [ ] Test ActorSystemSubscriber receives messages from broker
- [ ] Test ActorSystemSubscriber routes messages to components
**Verification:** `cargo test --lib runtime::` - all tests passing

**Integration Testing Plan:**
**File:** `tests/subscriber-initialization-integration-tests.rs` (NEW)
- [ ] Test ActorSystemSubscriber initialized with runtime
- [ ] Test ActorSystemSubscriber subscribed to MessageBroker
- [ ] Test messages flow through subscriber to components
**Verification:** `cargo test --test subscriber-initialization-integration-tests` - all tests passing

---

### Phase 4: Verification & Testing (Days 2-3)

#### Task 4.1: Update All Integration Tests

**Objective:** Update all integration tests to work with fixed architecture.

**Files to Update:**
- All integration test files in `tests/`

**Changes:**
- Update tests that manually created channels to use real ComponentActor instances
- Update tests that depend on ComponentActor to provide Arc<WasmEngine>
- Verify all tests still pass

**Success Criteria:**
- ✅ All existing tests pass
- ✅ All new tests pass
- ✅ `cargo test` passes all 991+ tests
- ✅ `cargo clippy` clean

**Integration Testing Plan:**
All existing integration tests updated:
- [ ] All actor integration tests updated for WasmEngine injection
- [ ] All messaging integration tests updated for mailbox registration
- [ ] All component lifecycle tests updated for Component Model API
**Verification:** `cargo test` - all tests passing

---

#### Task 4.2: Add Architecture Compliance Tests

**Objective:** Create tests that verify architectural rules are followed.

**File to Create:** `tests/architecture_compliance_tests.rs` (NEW)

**Test Cases:**
```rust
#[test]
fn test_runtime_never_imports_from_actor() {
    // Meta-test: Verify no forbidden imports
    let runtime_code = include_str!("../src/runtime/mod.rs");
    assert!(!runtime_code.contains("use crate::actor"),
        "runtime/ should not import from actor/");
}

#[test]
fn test_core_never_imports_from_higher() {
    // Meta-test: Verify core doesn't import from runtime/actor/
    let core_code = include_str!("../src/core/mod.rs");
    assert!(!core_code.contains("use crate::runtime"),
        "core should not import from runtime/");
    assert!(!core_code.contains("use crate::actor"),
        "core should not import from actor/");
}

#[test]
fn test_component_model_is_mandatory() {
    // Meta-test: Verify ComponentModel is required, not optional
    let actor_code = include_str!("../src/actor/component/component_actor.rs");
    assert!(actor_code.contains("engine: Arc<WasmEngine>"),
        "ComponentActor should require Arc<WasmEngine>");
    assert!(!actor_code.contains("component_engine: Option<Arc<WasmEngine>>"),
        "ComponentActor engine field should not be Optional");
}

#[test]
fn test_mailbox_channels_created() {
    // Meta-test: Verify mailbox channels are created
    let spawner_code = include_str!("../src/actor/component/component_spawner.rs");
    assert!(spawner_code.contains("mpsc::unbounded_channel"),
        "ComponentSpawner should create mailbox channels");
    assert!(spawner_code.contains("register_mailbox"),
        "ComponentSpawner should register mailboxes");
}

#[test]
fn test_message_loop_exists() {
    // Meta-test: Verify message loop exists
    let actor_code = include_str!("../src/actor/component/component_actor.rs");
    assert!(actor_code.contains("run_message_loop"),
        "ComponentActor should have run_message_loop() method");
    assert!(actor_code.contains("mailbox_rx: Option<"),
        "ComponentActor should have mailbox_rx field");
}
```

**Success Criteria:**
- ✅ Architecture compliance tests pass
- ✅ All forbidden import checks fail (as expected - verifying they don't exist)
- ✅ `grep -r "use crate::actor" src/runtime/` returns nothing
- ✅ `cargo test` passes

**Verification:** `cargo test --test architecture_compliance` - all tests passing

---

#### Task 4.3: Comprehensive End-to-End Testing

**Objective:** Final verification that entire messaging system works.

**File to Create:** `tests/e2e_messaging_comprehensive_tests.rs` (NEW)

**Test Scenarios:**
- [ ] Fire-and-forget: A → B
- [ ] Request-response: A → B with callback
- [ ] Multiple components: A → B → C
- [ ] Concurrent messaging: Multiple senders to B
- [ ] Large payload handling
- [ ] Component crashes during message processing
- [ ] Timeout handling
- [ ] Correlation ID tracking
- [ ] Message ordering guarantees
- [ ] Resource cleanup on component shutdown

**Success Criteria:**
- ✅ All scenarios pass
- ✅ End-to-end messaging is fully functional
- ✅ `cargo test` passes
- ✅ All 991+ tests pass
- ✅ Zero clippy warnings

**Verification:** `cargo test --test e2e_messaging_comprehensive` - all tests passing

---

### Phase 5: Documentation (Days 1-2)

#### Task 5.1: Create Architecture Hotfix Summary Document

**File to Create:** `docs/architectural-decisions/architecture-hotfix-decision.md` (NEW)

**Content:**
- Summary of what was wrong
- How it was fixed
- Lessons learned
- ADR updates/references

**Success Criteria:**
- ✅ Document created
- ✅ Clear explanation of architectural decisions
- ✅ References to ADRs

---

#### Task 5.2: Update Active Context

**File to Update:** `active-context.md`

**Changes:**
- Update status to reflect WASM-TASK-HOTFIX-001 in progress
- Update progress tracking
- Document blockers removed

**Success Criteria:**
- ✅ Current-context.md updated
- ✅ Progress is accurate
- ✅ All stakeholders aligned

---

## Overall Success Criteria

This task is complete when:

### Phase 1: Circular Dependency Fixed ✅
- [ ] `ComponentMessage`, `PendingRequest`, `ResponseMessage`, `CorrelationId`, `CorrelationTracker`, `ResponseRouterStats`, `ResponseRouter` moved to `core/message.rs`
- [ ] `MessagingSubscriptionService` moved to `actor/message/`
- [ ] All imports updated (runtime/, actor/, core/)
- [ ] `grep -r "use crate::actor" src/runtime/` returns nothing
- [ ] CI layer dependency enforcement script in place
- [ ] All tests pass

### Phase 2: Duplicate Runtime Fixed ✅
- [ ] `ComponentActor::new()` requires `Arc<WasmEngine>` parameter
- [ ] `ComponentActor::component_engine` is required, not optional
- [ ] `Child::start()` uses `WasmEngine::load_component()`
- [ ] `Actor::handle()` uses Component Model API via `WasmEngine`
- [ ] All manual marshalling code removed
- [ ] All tests updated to provide engine
- [ ] `cargo build` succeeds
- [ ] `cargo test` passes
- [ ] `grep -rn "wasmtime::Module" src/actor/` returns nothing
- [ ] Generated WIT bindings are actively used

### Phase 3: Integration Glue Implemented ✅
- [ ] `ComponentSpawner::spawn_component()` creates mailbox channels
- [ ] `ComponentSpawner::spawn_component()` registers with ActorSystemSubscriber
- [ ] `ComponentActor` has `run_message_loop()` method
- [ ] Message loop started when component spawned
- [ ] End-to-end integration tests prove messaging works
- [ ] `cargo test` passes
- [ ] All 991+ tests pass

### Phase 4: Verification Complete ✅
- [ ] All integration tests updated
- [ ] Architecture compliance tests created and passing
- [ ] Comprehensive end-to-end testing complete
- [ ] `cargo test` passes all tests
- [ ] Zero clippy warnings
- [ ] CI layer checks passing

### Phase 5: Documentation Complete ✅
- [ ] Architecture hotfix summary document created
- [ ] Active context updated
- [ ] Progress is accurate

### Overall Quality Gates ✅
- [ ] Zero compiler warnings (cargo build)
- [ ] Zero clippy warnings
- [ ] All 991+ tests passing
- [ ] End-to-end messaging functional
- [ ] Inter-component communication working
- [ ] Module boundary compliance verified

---

## Timeline Summary

| Phase | Tasks | Duration | Dependencies |
|-------|-------|----------|--------------|
| **Phase 1** | 1.1-1.3 + CI | 1-2 days | None |
| **Phase 2** | 2.1-2.5 + tests | 2-3 days | Phase 1 complete |
| **Phase 3** | 3.1-3.4 | 1.5-2 days | Phase 2 complete |
| **Phase 4** | 4.1-4.3 | 2-3 days | Phase 3 complete |
| **Phase 5** | 5.1-5.2 | 1-2 days | Phase 4 complete |
| **TOTAL** | **13 tasks** | **4.5-5.5 weeks** | All phases sequential |

---

## Quality Verification

### Per Phase Verification:

**Phase 1:**
```bash
# Architecture compliance
grep -r "use crate::actor" src/runtime/
# Expected: Empty

# All tests pass
cargo test --lib

# Clean build
cargo build

# Zero warnings
cargo clippy --all-targets --all-features -- -D warnings
```

**Phase 2:**
```bash
# No core WASM API usage
grep -rn "wasmtime::Module\|wasmtime::Engine\|wasmtime::Store" src/actor/
# Expected: Empty

# No workaround code
grep -rn "WasmBumpAllocator\|HandleMessageParams\|HandleMessageResult" src/actor/
# Expected: Empty

# All tests pass
cargo test

# Clean build
cargo build

# Zero warnings
cargo clippy --all-targets --all-features -- -D warnings
```

**Phase 3:**
```bash
# Mailbox channels created
grep -rn "mpsc::unbounded_channel\|register_mailbox" src/actor/component/
# Expected: Both patterns found

# Message loop exists
grep -rn "run_message_loop\|mailbox_rx" src/actor/component/
# Expected: Both patterns found

# End-to-end tests
cargo test --test block5_messaging_e2e
# Expected: All tests pass

# All tests pass
cargo test

# Clean build
cargo build

# Zero warnings
cargo clippy --all-targets --all-features -- -D warnings
```

**Phase 4:**
```bash
# Architecture compliance tests
cargo test --test architecture_compliance
# Expected: All tests pass

# End-to-end comprehensive tests
cargo test --test e2e_messaging_comprehensive
# Expected: All tests pass

# All tests pass (991+)
cargo test
# Expected: All tests pass

# Clean build
cargo build

# Zero warnings
cargo clippy --all-targets --all-features -- -D warnings
```

**Final Verification:**
```bash
# ALL must return NOTHING for valid architecture
grep -rn "use crate::runtime" src/core/
grep -rn "use crate::actor" src/core/
grep -rn "use crate::security" src/core/
grep -rn "use crate::runtime" src/security/
grep -rn "use crate::actor" src/security/
grep -rn "use crate::actor" src/runtime/

# All tests pass
cargo test

# Clean build
cargo build

# Zero warnings
cargo clippy --all-targets --all-features -- -D warnings
```

---

## Risk Assessment

### Identified Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| **Breaking API changes** | Medium | Medium | Comprehensive test coverage, gradual rollout |
| **Integration complexity** | Medium | Low | Well-designed, incremental approach |
| **Missing requirements** | Low | Low | Clear scope, well-defined tasks |
| **Timeline slip** | Low | Medium | Conservative estimates, buffer time |
| **Test coverage gaps** | Low | Medium | Architecture compliance tests, comprehensive integration tests |
| **Module boundary violations** | Low | High | CI enforcement, verification commands |

---

## Implementation Notes

### Key Principles:
1. **Incremental Delivery**: Each phase has clear milestones
2. **Verification Gates**: Don't mark complete until verified
3. **Real Testing**: Add comprehensive integration tests, not just unit tests
4. **Documentation First**: Explain what and why before coding
5. **Standards Compliance**: Follow all ADRs and workspace standards

### Critical Dependencies:
- Phase 2 depends on Phase 1 (circular dependency must be fixed first)
- Phase 3 depends on Phase 2 (Component Model API must be used first)
- Phase 4 depends on Phase 3 (messaging glue must be in place)

### Parallel Opportunities:
- Tasks within a phase can be done in parallel where feasible
- Unit tests can be written in parallel with implementation
- Integration tests can be written while implementation is in progress

---

**Status:** NOT STARTED - AWAITING APPROVAL

**Approval Required:** Do you approve this implementation plan? (Yes/No)
