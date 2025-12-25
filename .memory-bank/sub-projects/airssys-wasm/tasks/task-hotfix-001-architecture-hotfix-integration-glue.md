# [WASM-TASK-HOTFIX-001] - Architecture Hotfix & Integration Glue

**Task ID:** WASM-TASK-HOTFIX-001  
**Created:** 2025-12-25  
**Priority:** 🔴 CRITICAL / BLOCKING  
**Status:** NOT STARTED  
**Blocks:** All subsequent WASM-TASK-006+ work  
**Estimated Effort:** 4.5-5.5 weeks  

---

## Executive Summary

### What Was Wrong (From Deep Code Audit)

Three critical architectural violations discovered during development:

1. **🔴 Circular Dependency:** runtime/ → actor/ imports violate one-way architecture
2. **🔴 Duplicate WASM Runtime:** actor/ created duplicate runtime using wrong API
3. **🔴 Missing Integration Glue:** MessageBroker → ComponentActor never delivers messages

### What Needs to Happen

**ONE CONSOLIDATED TASK** to fix all three issues before any further development.

---

## Problem Statement

### Issue 1: Circular Dependency Between runtime/ and actor/

**Current (WRONG):**
\`\`\`
runtime/async_host.rs:52  → use crate::actor::message::{PendingRequest, ResponseMessage}
runtime/messaging.rs:76 → use crate::actor::message::{CorrelationId, CorrelationTracker, RequestError, ResponseMessage}
\`\`\`

**Correct Architecture (ADR-WASM-018):**
\`\`\`
actor/ → runtime/ → core/ (one-way dependencies)
\`\`\`

**Impact:**
- Violates ADR-WASM-018 three-layer architecture
- Cannot test runtime/ in isolation
- Creates circular coupling
- Makes code harder to understand

### Issue 2: Duplicate WASM Runtime Using Wrong API

**Current (PARTIALLY FIXED):**
\`\`\`
ComponentActor has: component_engine: Option<Arc<WasmEngine>>  ← WRONG!
ComponentActor has: component_handle: Option<ComponentHandle>    ← WRONG!
Workaround code deleted, but engine still optional
\`\`\`

**Correct Architecture (ADR-WASM-002):**
\`\`\`
ComponentActor MUST have: engine: Arc<WasmEngine>  ← REQUIRED
ComponentActor MUST have: component_handle: Option<ComponentHandle>  ← Set when loaded
All execution MUST use WasmEngine::call_handle_message() (Component Model API)
\`\`\`

**Impact:**
- Generated WIT bindings (154KB) are not actually used in execution
- Type safety is bypassed
- Components can be created without WASM engine
- ADR-WASM-002 mandate not enforced

### Issue 3: Missing Message Delivery Integration Glue

**Current (NOT FIXED):**
\`\`\`
ComponentSpawner::spawn_component() does NOT create mailbox channels
ComponentSpawner does NOT register mailboxes with ActorSystemSubscriber
ComponentActor has mailbox_rx: Option<> but never initialized
No message loop consuming from mailbox_rx
Messages published to MessageBroker never reach ComponentActor
\`\`\`

**Correct Architecture (ADR-WASM-020):**
\`\`\`
ComponentSpawner MUST create mailbox channels
ComponentSpawner MUST register with ActorSystemSubscriber::register_mailbox()
ComponentActor MUST have run_message_loop() consuming from mailbox_rx
Messages flow: Broker → Subscriber → Mailbox → Loop → handle_message() → WASM
\`\`\`

**Impact:**
- Inter-component messaging DOES NOT WORK at all
- fire-and-forget broken
- request-response broken
- All 991 tests pass in isolation but end-to-end flow broken
- Block 5 (Inter-Component Communication) is 0% functional despite documentation claiming 67% complete

---

## Context

### Relevant Architecture Documents

- **KNOWLEDGE-WASM-002**: High-Level Overview
- **KNOWLEDGE-WASM-003**: Core Architecture Design
- **ADR-WASM-018**: Three-Layer Architecture (actor/ → runtime/ → core/)
- **ADR-WASM-020**: Message Delivery Ownership Architecture
- **ADR-WASM-023**: Module Boundary Enforcement
- **DEBT-WASM-004**: Message Delivery Runtime Glue Missing

### Completed Blocks (Foundation)

- ✅ **WASM-TASK-000**: Core Abstractions (9,283 lines, 363 tests)
- ✅ **WASM-TASK-002**: WASM Runtime Layer (338 lines, 214 tests)
- ✅ **WASM-TASK-003**: WIT Interface System (2,214 lines WIT + 176 lines build)
- ✅ **WASM-TASK-004**: Actor System Integration (15,620+ lines, 589 tests)
- ✅ **WASM-TASK-005**: Security & Isolation Layer (13,500+ lines, 388 tests)

### Foundation Quality Metrics

- **Total Code**: 275K+ lines (9,283 + 338 + 2,390 + 15,620 + 13,500)
- **Total Tests**: 1,654 tests (363 + 214 + 589 + 388)
- **Test Pass Rate**: 100% (all tests passing)
- **Code Quality**: Zero compiler warnings, zero clippy warnings
- **Architecture**: Block 4: 100% complete with production authorization

---

## Objectives

### Primary Objective

Fix all three critical architectural violations so that:

1. ✅ One-way dependency architecture enforced (actor/ → runtime/ → core/)
2. ✅ Component Model API mandatory and properly used
3. ✅ Message delivery integration glue implemented and working

### Secondary Objectives

- Make generated WIT bindings (154KB) actively used
- Enable end-to-end inter-component messaging
- Fix Block 5 (Inter-Component Communication) to be genuinely functional
- Maintain zero compiler/clippy warnings
- Add comprehensive integration tests proving end-to-end functionality
- Establish CI enforcement to prevent future architectural violations

---

## Implementation Plan

### Phase 1: Fix Circular Dependency (Days 1-2)

#### Task 1.1: Move Messaging Types to Core/

**Objective:** Move shared messaging types from actor/ to core/ to eliminate circular imports.

**Deliverables:**

**Files to Create:**
- \`src/core/message.rs\` (NEW) - Contains:
  - \`PendingRequest\` struct
  - \`ResponseMessage\` struct
  - \`CorrelationId\` type (alias for String)
  - \`CorrelationTracker\` struct
  - \`RequestError\` enum
  - \`ResponseRouterStats\` struct (move from runtime/messaging.rs)
  - \`ResponseRouter\` struct (move from runtime/messaging.rs)

**Files to Update:**
- \`src/core/mod.rs\` - Add \`pub mod message;\` and export types

**Files to Update Imports (runtime/):**
- \`src/runtime/async_host.rs\` line 52
- \`src/runtime/messaging.rs\` line 76
- Change: \`use crate::actor::message::{...}\` → \`use crate::core::message::{...}\`\`

**Files to Update Imports (actor/):**
- \`src/actor/message/mod.rs\` - Update imports from core/
- \`src/actor/component/component_actor.rs\` - Update imports from core/
- \`src/actor/message/correlation_tracker.rs\` - Update imports from core/

**Success Criteria:**
- ✅ All messaging types exported from core/message.rs
- ✅ runtime/ no longer imports from actor/
- ✅ \`grep -r "use crate::actor" src/runtime/\` returns nothing
- ✅ \`cargo build\` succeeds
- ✅ All tests pass

**Estimated Effort:** 1-2 days  
**Risk Level:** Low (compiler will catch import errors)

---

#### Task 1.2: Move MessagingSubscriptionService

**Objective:** Move messaging subscription logic from runtime/ to actor/message/ where it belongs.

**Deliverables:**

**Files to Move:**
- \`src/runtime/messaging_subscription.rs\` → \`src/actor/message/messaging_subscription.rs\`

**Files to Update:**
- \`src/runtime/mod.rs\` - Remove \`pub mod messaging_subscription;\`
- \`src/actor/message/mod.rs\` - Add \`pub mod messaging_subscription;\`
- Update internal imports as needed

**Success Criteria:**
- ✅ MessagingSubscriptionService in actor/message/
- ✅ runtime/ no longer has actor-level subscription code
- ✅ All tests pass
- \`cargo build\` succeeds
- \`cargo test\` passes

**Estimated Effort:** 2-4 hours  
**Risk Level:** Low

---

#### Task 1.3: Add CI Layer Dependency Enforcement

**Objective:** Create CI script to prevent future circular dependency violations.

**Deliverables:**

**Files to Create:**
- \`.github/scripts/check-layer-deps.sh\` (NEW CI script)

**CI Script Content:**
\`\`\`bash
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
if grep -rq "use crate::runtime\\|use crate::actor" src/core/ 2>/dev/null; then
    echo "❌ ERROR: core/ imports from higher layers"
    grep -rn "use crate::runtime\\|use crate::actor" src/core/
    exit 1
fi
echo "  ✅ core clean"

echo ""
echo "✅ All layer dependency checks passed!"
\`\`\`

**Files to Update:**
- \`.github/workflows/ci.yml\` - Add layer dependency check step

**Success Criteria:**
- ✅ CI script created and executable
- ✅ Integrated into CI workflow
- ✅ Prevents future circular dependency violations
- ✅ All existing code passes checks

**Estimated Effort:** 2-4 hours  
**Risk Level:** Low

---

### Phase 2: Fix Duplicate WASM Runtime (Days 2-3)

#### Task 2.1: Make WasmEngine Mandatory in ComponentActor

**Objective:** Change ComponentActor to require Arc<WasmEngine> instead of optional.

**Deliverables:**

**File to Update:** \`src/actor/component/component_actor.rs\`

**Changes:**
\`\`\`rust
// BEFORE (WRONG - lines around 486-487):
    component_engine: Option<Arc<WasmEngine>>,
    component_handle: Option<ComponentHandle>,

// AFTER (CORRECT):
    engine: Arc<WasmEngine>,        // Required parameter
    component_handle: Option<ComponentHandle>,  // Loaded after start()
\`\`\`

**Method Updates:**
- Update \`ComponentActor::new()\` to require \`engine: Arc<WasmEngine>\` parameter
- Update \`ComponentActor::with_component_engine()\` to be primary constructor
- Update \`ComponentActor::component_engine()\` to return \`&Arc<WasmEngine>\`
- Update \`ComponentActor::component_handle()\` to return \`Option<&ComponentHandle>\`
- Update \`ComponentActor::uses_component_model()\` to always return true

**Success Criteria:**
- ✅ ComponentActor requires WasmEngine to construct
- ✅ Cannot create ComponentActor without providing engine
- ✅ \`grep "component_engine: Option" src/actor/component/\` returns nothing
- ✅ All ComponentActor tests updated to provide engine
- \`cargo build\` succeeds
- \`cargo test\` passes

**Estimated Effort:** 4-6 hours  
**Risk Level:** Medium (breaking API change)

---

#### Task 2.2: Update Child::start() to Use WasmEngine

**Objective:** Rewrite ComponentActor's Child trait implementation to use WasmEngine instead of core WASM API.

**File to Update:** \`src/actor/component/child_impl.rs\`

**Changes:**
- Remove all \`wasmtime::{Config, Engine, Module, Store}\` imports (if any remain)
- Remove all core WASM Module, Linker, Instance usage
- Use \`self.engine.load_component(&component_id, &wasm_bytes).await?\` instead
- Use \`self.component_handle = Some(handle)\` to store result
- Update error messages to reference WasmEngine

**Success Criteria:**
- ✅ \`grep -rn "wasmtime::Module" src/actor/\` returns nothing
- ✅ \`grep -rn "wasmtime::Engine" src/actor/\` returns nothing
- ✅ \`grep -rn "wasmtime::Store" src/actor/\` returns nothing
- ✅ Child::start() uses WasmEngine::load_component()
- \`cargo build\` succeeds
- \`cargo test\` passes

**Estimated Effort:** 4-6 hours  
**Risk Level:** Medium (core execution path change)

---

#### Task 2.3: Update Actor::handle() to Use Component Model API

**File to Update:** \`src/actor/component/actor_impl.rs\`

**Changes:**
- Remove any remaining \`WasmBumpAllocator\` references
- Remove any remaining \`HandleMessageParams\` references
- Remove any remaining \`HandleMessageResult\` references
- Use \`self.engine.call_handle_message(handle, &msg.sender, &msg.payload).await?\` instead
- Update error handling to use WasmEngine errors

**Success Criteria:**
- ✅ No bump allocator usage
- ✅ No manual parameter marshalling
- ✅ Uses Component Model typed calls via WasmEngine
- ✅ Generated WIT bindings are actively used
- \`grep "WasmBumpAllocator" src/actor/component/\` returns nothing
- \`grep "HandleMessageParams" src/actor/component/\` returns nothing
- \`grep "HandleMessageResult" src/actor/component/\` returns nothing
- \`cargo build\` succeeds
- \`cargo test\` passes

**Estimated Effort:** 6-8 hours  
**Risk Level:** Medium (message handling path change)

---

#### Task 2.4: Extend WasmEngine if Needed

**Objective:** Ensure WasmEngine has all methods needed by ComponentActor.

**File to Update:** \`src/runtime/engine.rs\`

**Potential Additions:**
- Verify \`call_handle_message()\` exists
- Verify \`call_handle_callback()\` exists
- Add \`pub async fn call_handle_message(...)\` if missing
- Add \`pub async fn call_handle_callback(...)\` if missing
- Update documentation

**Success Criteria:**
- ✅ WasmEngine provides all required methods
- ✅ Generated bindings are actively used via WasmEngine
- \`cargo build\` succeeds
- \`cargo test\` passes

**Estimated Effort:** 2-4 hours (contingency)  
**Risk Level:** Low (additive only)

---

#### Task 2.5: Update All ComponentActor Tests

**Objective:** Update all ComponentActor tests to provide Arc<WasmEngine>.

**Files to Update:**
- All test files in \`tests/\` and \`src/actor/component/*_tests.rs\`

**Changes:**
- All \`ComponentActor::new()\` calls must provide \`Arc<WasmEngine>\`
- All fixture setups must provide engine
- Update test assertions to verify engine usage

**Success Criteria:**
- ✅ All ComponentActor tests provide engine
- ✅ \`cargo test --lib\` passes
- ✅ Zero test failures
- \`cargo clippy\` clean

**Estimated Effort:** 3-4 hours  
**Risk Level:** Low (test updates only)

---

### Phase 3: Implement Missing Integration Glue (Days 1.5-2)

#### Task 3.1: Fix ComponentSpawner to Create Mailbox Channels

**Objective:** Enable ComponentSpawner to create mailbox channels and register them.

**File to Update:** \`src/actor/component/component_spawner.rs\`

**Changes:**
\`\`\`rust
// In spawn_component() method, ADD:
    // Step 3: Create mailbox channel
    let (mailbox_tx, mailbox_rx) = tokio::sync::mpsc::unbounded_channel::<ComponentMessage>();
    
    // Step 4: Register mailbox with ActorSystemSubscriber
    if let Some(subscriber) = &self.actor_system_subscriber {
        subscriber.register_mailbox(component_id.clone(), mailbox_tx).await?;
    }
    
    // Step 5: Store receiver in actor
    actor.set_mailbox_receiver(mailbox_rx);
\`\`\`

**Add Dependencies if needed:**
- Import \`tokio::sync::mpsc\` at top of file
- Import \`ComponentMessage\` from \`crate::core::message\`

**Success Criteria:**
- ✅ \`grep "mpsc::unbounded_channel" src/actor/component/component_spawner.rs\` shows usage
- ✅ \`grep "register_mailbox" src/actor/component/component_spawner.rs\` shows usage
- ✅ \`grep "set_mailbox_receiver" src/actor/component/component_actor.rs\` shows usage
- \`cargo build\` succeeds
- \`cargo test\` passes

**Estimated Effort:** 2-3 hours  
**Risk Level:** Medium (adds new functionality)

---

#### Task 3.2: Add Message Loop to ComponentActor

**Objective:** Enable ComponentActor to consume messages from mailbox and process them.

**File to Update:** \`src/actor/component/component_actor.rs\`

**Changes:**
\`\`\`rust
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
\`\`\`

**Success Criteria:**
- ✅ \`grep "mailbox_rx: Option" src/actor/component/component_actor.rs\` shows field added
- ✅ \`grep "run_message_loop" src/actor/component/component_actor.rs\` shows method added
- ✅ Message loop calls \`handle_message()\`
- \`cargo build\` succeeds
- \`cargo test\` passes
- Integration tests prove end-to-end messaging works

**Estimated Effort:** 3-4 hours  
**Risk Level:** Medium (message handling logic)

---

#### Task 3.3: Verify End-to-End Message Flow

**Objective:** Create comprehensive integration test proving messages flow from A → B.

**File to Create:** \`tests/block5_messaging_e2e_integration_tests.rs\` (NEW)

**Test Cases:**
\`\`\`rust
#[tokio::test]
async fn test_component_a_sends_to_component_b_real() {
    // 1. Initialize runtime with two components
    let runtime = WasmRuntime::new(config);
    
    // 2. Load Component A
    let component_a = runtime.load_component(
        include_bytes!("fixtures/basic-component.wasm"),
        caps
    ).await?;
    
    // 3. Load Component B
    let component_b = runtime.load_component(
        include_bytes!("fixtures/basic-component.wasm"),
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
\`\`\`

**Success Criteria:**
- ✅ Integration test creates and runs successfully
- ✅ Message flows from A → B end-to-end
- ✅ Test proves mailbox delivery works
- \`cargo test --test block5_messaging_e2e\` passes
- All other tests still pass

**Estimated Effort:** 4-6 hours  
**Risk Level:** Low (new test, doesn't break existing)

---

#### Task 3.4: Update ActorSystemSubscriber Initialization

**Objective:** Ensure ActorSystemSubscriber is started when runtime initializes.

**File to Update:** \`src/runtime/mod.rs\` or initialization code

**Changes:**
- In WasmRuntime initialization or similar, call \`subscriber.start().await?\`
- Ensure subscriber is started before components begin messaging

**Success Criteria:**
- ✅ ActorSystemSubscriber is started
- ✅ \`grep "start()" src/actor/message/messaging_subscription.rs\` is called
- \`cargo build\` succeeds
- \`cargo test\` passes

**Estimated Effort:** 1-2 hours  
**Risk Level:** Low (single call addition)

---

### Phase 4: Verification & Testing (Days 2-3)

#### Task 4.1: Update All Integration Tests

**Objective:** Update all integration tests to work with fixed architecture.

**Files to Update:**
- All integration test files in \`tests/\`

**Changes:**
- Update tests that manually created channels to use real ComponentActor instances
- Update tests that depend on ComponentActor to provide Arc<WasmEngine>
- Verify all tests still pass

**Success Criteria:**
- ✅ All existing tests pass
- ✅ All new tests pass
- \`cargo test\` passes all 991 tests
- \`cargo clippy\` clean

**Estimated Effort:** 3-5 hours  
**Risk Level:** Low (test updates)

---

#### Task 4.2: Add Architecture Compliance Tests

**Objective:** Create tests that verify architectural rules are followed.

**File to Create:** \`tests/architecture_compliance_tests.rs\` (NEW)

**Test Cases:**
\`\`\`rust
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
\`\`\`

**Success Criteria:**
- ✅ Architecture compliance tests pass
- ✅ All forbidden import checks fail (as expected)
- ✅ \`grep -r "use crate::actor" src/runtime/\` returns nothing
- \`cargo test\` passes

**Estimated Effort:** 3-4 hours  
**Risk Level:** Low (new tests only)

---

#### Task 4.3: Comprehensive End-to-End Testing

**Objective:** Final verification that entire messaging system works.

**File to Create:** \`tests/e2e_messaging_comprehensive_tests.rs\` (NEW)

**Test Scenarios:**
- Fire-and-forget: A → B
- Request-response: A → B with callback
- Multiple components: A → B → C
- Concurrent messaging: Multiple senders to B
- Large payload handling
- Component crashes during message processing

**Success Criteria:**
- ✅ All scenarios pass
- ✅ End-to-end messaging is fully functional
- \`cargo test\` passes
- All 991 tests pass
- Zero clippy warnings

**Estimated Effort:** 4-6 hours  
**Risk Level:** Low (comprehensive testing)

---

### Phase 5: Documentation (Days 1-2)

#### Task 5.1: Create Architecture Hotfix Summary Document

**File to Create:** \`docs/architectural-decisions/architecture-hotfix-decision.md\` (NEW)

**Content:**
- Summary of what was wrong
- How it was fixed
- Lessons learned
- ADR updates/references

**Success Criteria:**
- ✅ Document created
- ✅ Clear explanation of architectural decisions
- ✅ References to ADRs

**Estimated Effort:** 2-3 hours  
**Risk Level:** Low

---

#### Task 5.2: Update Active Context

**File to Update:** \`current-context.md\`

**Changes:**
- Update status to reflect WASM-TASK-HOTFIX-001 in progress
- Update progress tracking
- Document blockers removed

**Success Criteria:**
- ✅ Current-context.md updated
- ✅ Progress is accurate
- All stakeholders aligned

**Estimated Effort:** 1-2 hours  
**Risk Level:** Low

---

## Success Criteria

This task is complete when:

### Phase 1: Circular Dependency Fixed ✅
- [ ] \`ComponentMessage\`, \`PendingRequest\`, \`ResponseMessage\`, \`CorrelationId\`, \`CorrelationTracker\`, \`ResponseRouterStats\`, \`ResponseRouter\` moved to \`core/message.rs\`
- [ ] \`MessagingSubscriptionService\` moved to \`actor/message/\`
- [ ] All imports updated (runtime/, actor/ core/)
- [ ] \`grep -r "use crate::actor" src/runtime/\` returns nothing
- [ ] CI layer dependency enforcement script in place
- [ ] All tests pass

### Phase 2: Duplicate Runtime Fixed ✅
- [ ] \`ComponentActor::new()\` requires \`Arc<WasmEngine>\` parameter
- [ ] \`ComponentActor::component_engine\` is required, not optional
- [ ] \`Child::start()\` uses \`WasmEngine::load_component()\`
- [ ] \`Actor::handle()\` uses Component Model API via \`WasmEngine\`
- [ ] All manual marshalling code removed
- [ ] All tests updated to provide engine
- [ ] \`cargo build\` succeeds
- [ ] \`cargo test\` passes
- [ ] \`grep -rn "wasmtime::Module" src/actor/\` returns nothing
- [ ] Generated WIT bindings are actively used

### Phase 3: Integration Glue Implemented ✅
- [ ] \`ComponentSpawner::spawn_component()\` creates mailbox channels
- [ ] \`ComponentSpawner::spawn_component()\` registers with ActorSystemSubscriber
- [ ] \`ComponentActor\` has \`run_message_loop()\` method
- [ ] Message loop started when component spawned
- [ ] End-to-end integration tests prove messaging works
- [ ] \`cargo test\` passes
- [ ] All 991 tests pass

### Phase 4: Verification Complete ✅
- [ ] All integration tests updated
- [ ] Architecture compliance tests created and passing
- [ ] Comprehensive end-to-end testing complete
- [ ] \`cargo test\` passes all tests
- [ ] Zero clippy warnings
- [ ] CI layer checks passing

### Phase 5: Documentation Complete ✅
- [ ] Architecture hotfix summary document created
- [ ] Active context updated
- [ ] Progress is accurate

### Overall Quality Gates ✅
- [ ] Zero compiler warnings (cargo build)
- [ ] Zero clippy warnings
- [ ] All 991 tests passing
- [ ] End-to-end messaging functional
- [ ] Inter-component communication working

---

## Timeline Summary

| Phase | Tasks | Duration | Dependencies |
|-------|-------|----------|--------------|
| **Phase 1** | 1.1-1.3 + CI | 1-2 days | None |
| **Phase 2** | 2.1-2.5 + tests | 2-3 days | Phase 1 complete |
| **Phase 3** | 3.1-3.4 | 1.5-2 days | Phase 2 complete |
| **Phase 4** | 4.1-4.3 | 2-3 days | Phase 3 complete |
| **Phase 5** | 5.1-5.2 | 1-2 days | Phase 4 complete |
| **TOTAL** | **4.5-5.5 weeks** | All phases sequential |

---

## Risk Assessment

### Identified Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| **Breaking API changes** | Medium | Medium | Comprehensive test coverage, gradual rollout |
| **Integration complexity** | Medium | Low | Well-designed, incremental approach |
| **Missing requirements** | Low | Clear scope, well-defined tasks |
| **Timeline slip** | Low | Conservative estimates, buffer time |
| **Test coverage gaps** | Low | Architecture compliance tests, comprehensive integration tests |

---

## References

### Architecture Documents
- **KNOWLEDGE-WASM-002**: High-Level Overview
- **KNOWLEDGE-WASM-003**: Core Architecture Design
- **ADR-WASM-018**: Three-Layer Architecture (actor/ → runtime/ → core/)
- **ADR-WASM-020**: Message Delivery Ownership Architecture
- **ADR-WASM-023**: Module Boundary Enforcement
- **DEBT-WASM-004**: Message Delivery Runtime Glue Missing

### Archived Tasks Referenced
- WASM-TASK-005: Security & Isolation Layer (COMPLETE - reference)
- WASM-TASK-006: Block 5 - Inter-Component Communication (PARTIAL - reference for remediation)
- WASM-TASK-006-HOTFIX: Critical Architecture Remediation (NOT STARTED - absorbed into this task)

### Related Knowledge
- **KNOWLEDGE-WASM-031**: Foundational Architecture
- **KNOWLEDGE-WASM-005**: Messaging Architecture
- **KNOWLE-WASM-026**: Message Delivery Architecture Clarifications

---

## Notes

### Why This Consolidated Task

**Previous Approach:**
- 30+ archived task files spread across different issues
- Some tasks marked "COMPLETE" when code never actually fixed
- Confusion about what's done vs. what's remaining
- No clear single source of truth

**New Approach:**
- **ONE consolidated task** with clear phases
- Each phase has explicit success criteria
- All changes tracked in single location
- Progress transparent and verifiable
- Prevents "paperwork over substance" problem

### Key Principles Applied

1. **Incremental Delivery**: Each phase has clear milestones
2. **Verification Gates**: Don't mark complete until verified
3. **Real Testing**: Add comprehensive integration tests, not just unit tests
4. **Documentation First**: Explain what and why before coding
5. **Standards Compliance**: Follow all ADRs and workspace standards

---

## History

| Date | Version | Changes |
|------|---------|---------|
| 2025-12-25 | 1.0 | Initial creation - consolidated architecture hotfix task |

---

**Task ID:** WASM-TASK-HOTFIX-001  
**Status:** NOT STARTED  
**Priority:** 🔴 CRITICAL / BLOCKING  
**Blocker For**: All WASM-TASK-006+ work and Block 5+ work  

---

