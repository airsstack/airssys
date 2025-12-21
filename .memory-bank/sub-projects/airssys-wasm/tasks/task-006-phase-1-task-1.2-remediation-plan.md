# Remediation Plan: WASM-TASK-006 Phase 1 Task 1.2 - Message Reception Integration Tests

**Status:** proposed  
**Created:** 2025-12-21  
**Task ID:** WASM-TASK-006-1.2-REMEDIATION  
**Parent Task:** task-006-phase-1-task-1.2-plan.md  
**Estimated Effort:** 8-10 hours  
**Priority:** Critical - Tests do NOT prove message flow works  
**Dependency:** Task 1.1 Remediation (delivery side must work first)

---

## Executive Summary

This remediation plan addresses the critical issue identified in Task 1.2: **41 tests only validate APIs (AtomicU64 counters, config structs) - they do NOT test actual message flow or WASM invocation**.

### The Problem

Post-completion review discovered that the 41 "passing" tests do NOT prove the implementation works:

**Evidence from tests (`messaging_reception_tests.rs` lines 271-306):**
```rust
// Note: Testing actual WASM invocation requires instantiating a real WASM module,
// which needs the full WasmEngine infrastructure. These tests focus on the
// message reception logic and metrics tracking. Full integration tests with
// real WASM modules are in the main test suite.
```

**Evidence from implementation (`component_actor.rs` lines 2051-2052):**
```rust
// TODO(WASM-TASK-006 Task 1.2 Follow-up): Implement proper parameter
// marshalling using wasmtime component model bindings once generated.
```

### What the Tests Actually Test

| Test Category | Count | What They Test | What They DON'T Test |
|--------------|-------|----------------|---------------------|
| Reception Tests | 22 | `MessageReceptionMetrics` (AtomicU64) | Actual message delivery |
| Backpressure Tests | 19 | `BackpressureConfig` structs | Real message flow |
| **Total** | **41** | **API/Config validation only** | **WASM invocation** |

### The Solution

Per **ADR-WASM-020** and **AGENTS.md Section 8** (Mandatory Testing Requirements):

1. **Fix the TODO:** Implement proper parameter marshalling in `invoke_handle_message_with_timeout()`
2. **Add real integration tests:** Use existing WASM fixtures (`basic-handle-message.wasm`)
3. **Prove end-to-end flow:** Message → ComponentActor → WASM `handle-message` export

### References

- **ADR-WASM-020:** Message Delivery Ownership Architecture (Accepted 2025-12-21)
- **KNOWLEDGE-WASM-026:** Message Delivery Architecture - Final Decision
- **AGENTS.md Section 8:** Mandatory Testing Requirements
- **Task 1.1 Remediation:** Required first (delivery side)

---

## Goal

**Fix Task 1.2 so that tests PROVE actual message reception and WASM invocation works.**

After this remediation:
1. Integration tests instantiate real `ComponentActor` with WASM module
2. Tests send actual `ComponentMessage` messages
3. Tests verify WASM `handle-message` export is invoked
4. Tests verify the message content reaches the WASM component
5. End-to-end message flow is proven, not just API validation

---

## Context & References

### Current State (BROKEN)

```
Tests Claim ✅: 41 tests passing (100%)
Reality ❌: Tests only validate AtomicU64 counters and config structs

What's Tested:
✅ MessageReceptionMetrics.new() works
✅ MessageReceptionMetrics.record_message_received() increments counter
✅ BackpressureConfig fields can be set
✅ API validation passes

What's NOT Tested:
❌ ComponentActor receives a ComponentMessage
❌ invoke_handle_message_with_timeout() is called
❌ WASM handle-message export is actually invoked
❌ Message payload reaches WASM component
❌ Error handling with real WASM traps
❌ Timeout enforcement with real WASM execution
```

### Why This Matters

Per **AGENTS.md Section 8**:

> **Integration Tests MUST:**
> - Test end-to-end workflows
> - Test component/module interaction
> - Instantiate real components
> - Send actual messages
> - Verify actual behavior
>
> **Integration Tests MUST NOT:**
> - Only test metrics/counters ❌
> - Only test config structs ❌
> - Avoid testing real functionality ❌

**Current tests violate ALL of these requirements.**

### Dependency on Task 1.1 Remediation

Task 1.2 remediation depends on Task 1.1 remediation because:
- Messages must be **delivered** to ComponentActor mailbox (Task 1.1)
- Then ComponentActor must **receive** and **process** them (Task 1.2)

If Task 1.1 delivery is still stubbed, Task 1.2 tests cannot prove end-to-end flow.

---

## Implementation Steps

### Step 1: Analyze Parameter Marshalling TODO (1 hour)

**File:** `airssys-wasm/src/actor/component/component_actor.rs`  
**Lines:** 2044-2055

**Current (INCOMPLETE):**
```rust
// Note: handle-message WIT signature is:
// handle-message: func(sender: component-id, message: list<u8>) -> result<_, component-error>
//
// For now, we pass empty params as the actual parameter marshalling
// depends on the WIT bindings generation. This is a known limitation
// that will be addressed when proper WIT bindings are generated.
//
// TODO(WASM-TASK-006 Task 1.2 Follow-up): Implement proper parameter
// marshalling using wasmtime component model bindings once generated.
let mut results = vec![];
handle_fn
    .call_async(&mut *runtime.store_mut(), &[], &mut results)  // ← Empty params!
    .await
```

**Analysis Required:**
1. Check what the WIT bindings expect for `handle-message`
2. Determine how to marshal `sender: ComponentId` and `payload: Vec<u8>`
3. Check wasmtime component model binding patterns
4. Review existing WASM fixture (`basic-handle-message.wat`) for expected signature

### Step 2: Implement Parameter Marshalling (2 hours)

**File:** `airssys-wasm/src/actor/component/component_actor.rs`

**Expected Fix:**
```rust
// Convert ComponentId to WASM-compatible format
let sender_str = sender.as_str();

// Create WASM parameters
let params = vec![
    wasmtime::Val::from(sender_str),  // component-id as string
    wasmtime::Val::from(&payload[..]), // message as list<u8>
];

let mut results = vec![];
handle_fn
    .call_async(&mut *runtime.store_mut(), &params, &mut results)
    .await
```

**Note:** Exact marshalling depends on WIT bindings. May need to use wasmtime component model APIs.

### Step 3: Review Existing WASM Fixtures (0.5 hours)

**Location:** `airssys-wasm/tests/fixtures/`

**Available Fixtures:**
- `basic-handle-message.wasm` - Basic message handler
- `hello_world.wasm` - Simple hello world
- `rejecting-handler.wasm` - Handler that rejects messages
- `slow-handler.wasm` - Handler with intentional delay

**Tasks:**
1. Verify `basic-handle-message.wasm` exports `handle-message` function
2. Verify signature matches expected WIT interface
3. Check if fixtures need updates for new parameter format

### Step 4: Create Real Integration Tests (3 hours)

**File:** `airssys-wasm/tests/message_reception_integration_tests.rs` (NEW)

**Test Cases to Implement:**

#### Test 1: `test_component_actor_receives_message_and_invokes_wasm`

**Purpose:** PROVE ComponentActor receives a message and invokes WASM

```rust
/// CRITICAL TEST: Proves end-to-end message reception works
/// This is THE test that was missing - verifies WASM invocation
#[tokio::test]
async fn test_component_actor_receives_message_and_invokes_wasm() {
    // Step 1: Create WasmEngine
    let engine = WasmEngine::new().await.unwrap();
    
    // Step 2: Load basic-handle-message.wasm fixture
    let wasm_bytes = include_bytes!("fixtures/basic-handle-message.wasm");
    
    // Step 3: Create ComponentActor with WASM module
    let component_id = ComponentId::new("test-receiver");
    let mut actor = create_component_actor(
        engine.clone(),
        component_id.clone(),
        wasm_bytes,
    ).await.unwrap();
    
    // Step 4: Send message to ComponentActor
    let sender = ComponentId::new("test-sender");
    let payload = vec![1, 2, 3, 4, 5];
    
    // Step 5: Invoke handle_message
    let result = actor.invoke_handle_message_with_timeout(
        sender.clone(),
        payload.clone(),
    ).await;
    
    // Step 6: CRITICAL - Verify WASM was invoked
    assert!(result.is_ok(), "handle-message should succeed: {:?}", result);
    
    // Step 7: Verify metrics updated
    let metrics = actor.message_metrics();
    assert_eq!(metrics.messages_received(), 1);
}
```

#### Test 2: `test_component_actor_handles_wasm_trap`

**Purpose:** PROVE error handling works with real WASM traps

```rust
#[tokio::test]
async fn test_component_actor_handles_wasm_trap() {
    // Use rejecting-handler.wasm which traps on message
    let wasm_bytes = include_bytes!("fixtures/rejecting-handler.wasm");
    
    let mut actor = create_component_actor(/* ... */).await.unwrap();
    
    let result = actor.invoke_handle_message_with_timeout(
        ComponentId::new("sender"),
        vec![1, 2, 3],
    ).await;
    
    // Should fail with execution error (not panic)
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, WasmError::ExecutionFailed { .. }));
}
```

#### Test 3: `test_component_actor_enforces_timeout`

**Purpose:** PROVE timeout enforcement works with real slow WASM

```rust
#[tokio::test]
async fn test_component_actor_enforces_timeout() {
    // Use slow-handler.wasm which takes >1 second
    let wasm_bytes = include_bytes!("fixtures/slow-handler.wasm");
    
    let mut actor = create_component_actor(/* ... */).await.unwrap();
    
    // Configure short timeout (100ms)
    actor.set_delivery_timeout_ms(100);
    
    let start = Instant::now();
    let result = actor.invoke_handle_message_with_timeout(
        ComponentId::new("sender"),
        vec![1, 2, 3],
    ).await;
    
    let elapsed = start.elapsed();
    
    // Should fail with timeout
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, WasmError::ExecutionTimeout { .. }));
    
    // Should complete near 100ms (not wait for full WASM execution)
    assert!(elapsed < Duration::from_millis(500));
}
```

#### Test 4: `test_component_actor_message_payload_reaches_wasm`

**Purpose:** PROVE message payload is passed correctly to WASM

```rust
#[tokio::test]
async fn test_component_actor_message_payload_reaches_wasm() {
    // Use fixture that echoes back message payload
    let wasm_bytes = include_bytes!("fixtures/basic-handle-message.wasm");
    
    let mut actor = create_component_actor(/* ... */).await.unwrap();
    
    let sender = ComponentId::new("sender");
    let payload = b"Hello from integration test!".to_vec();
    
    let result = actor.invoke_handle_message_with_timeout(
        sender,
        payload,
    ).await;
    
    // Verify success - WASM received and processed payload
    assert!(result.is_ok());
    
    // If fixture supports output verification, check here
}
```

#### Test 5: `test_multiple_messages_processed_sequentially`

**Purpose:** PROVE multiple messages work correctly

```rust
#[tokio::test]
async fn test_multiple_messages_processed_sequentially() {
    let mut actor = create_component_actor(/* ... */).await.unwrap();
    
    // Send 10 messages
    for i in 0..10 {
        let result = actor.invoke_handle_message_with_timeout(
            ComponentId::new("sender"),
            vec![i as u8],
        ).await;
        
        assert!(result.is_ok(), "Message {} failed", i);
    }
    
    // Verify all processed
    let metrics = actor.message_metrics();
    assert_eq!(metrics.messages_received(), 10);
}
```

### Step 5: Add Helper Functions for Tests (1 hour)

**File:** `airssys-wasm/tests/message_reception_integration_tests.rs`

```rust
/// Helper: Create a ComponentActor with loaded WASM module
async fn create_component_actor(
    engine: Arc<WasmEngine>,
    component_id: ComponentId,
    wasm_bytes: &[u8],
) -> Result<ComponentActor<()>, WasmError> {
    // Create actor with default state
    let mut actor = ComponentActor::new(
        component_id,
        Default::default(), // Custom state
    );
    
    // Load WASM module
    actor.load_wasm(engine, wasm_bytes).await?;
    
    Ok(actor)
}

/// Helper: Include WASM fixture bytes
macro_rules! include_fixture {
    ($name:literal) => {
        include_bytes!(concat!("fixtures/", $name, ".wasm"))
    };
}
```

### Step 6: Update Existing Test File Comments (0.5 hours)

**File:** `airssys-wasm/tests/messaging_reception_tests.rs`

**Update the comment (lines 271-306):**

```rust
// Note: This test file focuses on MessageReceptionMetrics and BackpressureConfig
// API validation. It tests the infrastructure components in isolation.
//
// For tests that prove actual WASM message invocation works, see:
// - tests/message_reception_integration_tests.rs (WASM fixtures)
// - tests/actor_invocation_tests.rs (full actor stack)
//
// This API test suite validates:
// - MessageReceptionMetrics atomic counter operations
// - BackpressureConfig struct initialization and validation
// - Error type construction
//
// Integration tests validate:
// - ComponentActor receives ComponentMessage
// - invoke_handle_message_with_timeout() invokes WASM
// - handle-message export receives correct parameters
// - Error handling with real WASM traps
// - Timeout enforcement with real slow WASM
```

### Step 7: Verify Fixtures Support Required Behavior (1 hour)

**File:** `airssys-wasm/tests/fixtures/basic-handle-message.wat`

**Required:** Fixture must export `handle-message` with signature matching WIT

**Verification Steps:**
1. Read `basic-handle-message.wat` to verify export
2. If missing, update fixture to export correct function
3. Rebuild `.wasm` file using `build.sh`

**Expected WAT structure:**
```wat
(module
  (func $handle_message (export "handle-message")
    (param $sender i32)  ;; component-id pointer
    (param $sender_len i32)
    (param $message i32)  ;; message pointer
    (param $message_len i32)
    (result i32)  ;; 0 = success, non-zero = error
    
    ;; Implementation: accept message (return 0)
    i32.const 0
  )
)
```

---

## Unit Testing Plan

**MANDATORY**: Tests in module #[cfg(test)] blocks

**Test file location:** `src/actor/component/component_actor.rs`

### Unit Tests Already Exist (API Validation):

These tests exist and validate infrastructure - they're fine as-is:
- `test_message_reception_metrics_new()`
- `test_message_reception_metrics_record_message()`
- `test_backpressure_config_defaults()`
- etc.

### Unit Tests to Add:

#### Test: `test_invoke_handle_message_returns_error_without_wasm`

```rust
#[tokio::test]
async fn test_invoke_handle_message_returns_error_without_wasm() {
    let actor = ComponentActor::<()>::new(
        ComponentId::new("test"),
        (),
    );
    
    let result = actor.invoke_handle_message_with_timeout(
        ComponentId::new("sender"),
        vec![1, 2, 3],
    ).await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), WasmError::ComponentNotFound { .. }));
}
```

#### Test: `test_invoke_handle_message_returns_error_without_export`

```rust
#[tokio::test]
async fn test_invoke_handle_message_returns_error_without_export() {
    // Create actor with WASM that doesn't export handle-message
    let mut actor = create_actor_with_minimal_wasm().await;
    
    let result = actor.invoke_handle_message_with_timeout(
        ComponentId::new("sender"),
        vec![1, 2, 3],
    ).await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), WasmError::ExecutionFailed { .. }));
}
```

**Verification:** `cargo test --lib -p airssys-wasm -- component_actor::tests`

---

## Integration Testing Plan

**MANDATORY**: Tests in tests/ directory proving END-TO-END message reception

**Test file:** `tests/message_reception_integration_tests.rs` (NEW)

### Integration Tests to Create:

| Test Name | Purpose | Fixture | Verifies |
|-----------|---------|---------|----------|
| `test_component_actor_receives_message_and_invokes_wasm` | Prove WASM invocation | `basic-handle-message.wasm` | handle-message called |
| `test_component_actor_handles_wasm_trap` | Error handling | `rejecting-handler.wasm` | Trap caught, no panic |
| `test_component_actor_enforces_timeout` | Timeout works | `slow-handler.wasm` | Times out correctly |
| `test_component_actor_message_payload_reaches_wasm` | Payload delivery | `basic-handle-message.wasm` | Data passes correctly |
| `test_multiple_messages_processed_sequentially` | Multiple messages | `basic-handle-message.wasm` | All processed |
| `test_component_actor_metrics_updated_on_success` | Metrics tracking | `basic-handle-message.wasm` | Counters increment |
| `test_component_actor_metrics_updated_on_failure` | Error metrics | `rejecting-handler.wasm` | Error counters |

### WASM Fixture Prerequisites

> ⚠️ **IMPORTANT:** WASM test fixtures follow a source/compiled workflow.

| File Type | Description | Git Status |
|-----------|-------------|------------|
| `.wat` | WebAssembly Text (SOURCE) | ✅ Committed |
| `.wasm` | Compiled binary (GENERATED) | ❌ Gitignored |

**Before running integration tests**, compile the fixtures:

```bash
cd airssys-wasm/tests/fixtures
./build.sh
```

This runs `wasm-tools parse` to compile each `.wat` file to `.wasm`.

**Rationale:** Binary `.wasm` files are not committed because:
- They bloat the repository (binaries don't diff well)
- They can be regenerated from source `.wat` files
- Only source files should be version-controlled

**Current .gitignore rules (already correct):**
```gitignore
tests/fixtures/*.wasm
**/tests/fixtures/*.wasm
*.wasm
```

### Fixture Verification

**STATUS: READY** - All source fixtures exist (compile with `build.sh`)

| Fixture | Source File | Status | Used For |
|---------|-------------|--------|----------|
| `basic-handle-message` | `basic-handle-message.wat` ✅ | Compile with `build.sh` | Message reception tests |
| `rejecting-handler` | `rejecting-handler.wat` ✅ | Compile with `build.sh` | WASM trap handling tests |
| `slow-handler` | `slow-handler.wat` ✅ | Compile with `build.sh` | Timeout enforcement tests |
| `hello_world` | `hello_world.wat` ✅ | Compile with `build.sh` | Basic functionality tests |

**No new fixtures needed!** All source `.wat` files exist in `tests/fixtures/`.

**Before testing:**
```bash
cd airssys-wasm/tests/fixtures && ./build.sh
```

**Verification:** `cargo test --test message_reception_integration_tests`

---

## Quality Verification

- [ ] `cargo build -p airssys-wasm` - builds cleanly
- [ ] `cargo test --lib -p airssys-wasm` - all unit tests pass
- [ ] `cargo test --test message_reception_integration_tests -p airssys-wasm` - all integration tests pass
- [ ] `cargo clippy -p airssys-wasm --all-targets --all-features -- -D warnings` - zero warnings
- [ ] Zero compiler warnings

---

## Verification Steps

### Step 1: Run Unit Tests
```bash
cargo test --lib -p airssys-wasm -- component_actor::tests
```
**Expected:** All unit tests passing

### Step 2: Run Integration Tests
```bash
cargo test --test message_reception_integration_tests -p airssys-wasm
```
**Expected:** All integration tests passing, WASM invocation proven

### Step 3: Build Check
```bash
cargo build -p airssys-wasm
```
**Expected:** No warnings, builds cleanly

### Step 4: Clippy Check
```bash
cargo clippy -p airssys-wasm --all-targets --all-features -- -D warnings
```
**Expected:** Zero clippy warnings

### Step 5: Verify End-to-End Message Flow
**CRITICAL:** The `test_component_actor_receives_message_and_invokes_wasm` test MUST:
- Instantiate real ComponentActor with WASM
- Send real ComponentMessage
- Invoke real WASM handle-message export
- Verify success (not timeout, not error)
- Complete in reasonable time (<1s)

---

## Acceptance Criteria

### Primary (MUST be met)

1. ✅ **Parameter marshalling implemented**
   - `invoke_handle_message_with_timeout()` passes sender and payload to WASM
   - TODO comment at lines 2051-2052 resolved

2. ✅ **Integration tests PROVE WASM invocation**
   - At least 5 integration tests using real WASM fixtures
   - Tests instantiate real ComponentActor
   - Tests invoke real WASM handle-message export
   - Tests verify actual behavior (not just APIs)

3. ✅ **Error handling tested with real WASM**
   - WASM trap handling verified with `rejecting-handler.wasm`
   - Timeout enforcement verified with `slow-handler.wasm`

4. ✅ **Metrics updated correctly**
   - `messages_received` counter increments on success
   - Error counters increment on failure

5. ✅ **Zero warnings**
   - No compiler warnings
   - No clippy warnings

### Secondary

6. ✅ **Existing API tests preserved**
   - 41 existing tests still pass
   - Comments updated to clarify test scope

7. ✅ **Test documentation updated**
   - Comments explain what each test file validates
   - Cross-references to integration tests

---

## Risk Assessment

### Risk 1: Parameter Marshalling Complexity

**Impact:** High - Wrong marshalling = WASM trap or wrong data  
**Probability:** Medium - Depends on WIT bindings  
**Mitigation:**
- Study existing WASM invocation patterns in codebase
- Review wasmtime component model documentation
- Test with simple fixture first

### Risk 2: Fixture Compatibility

**Impact:** Medium - Fixtures may not match expected signature  
**Probability:** Low - Fixtures already exist  
**Mitigation:**
- Verify fixture exports before writing tests
- Update fixtures if needed
- Use `wasmtime` CLI to inspect exports

### Risk 3: Test Flakiness

**Impact:** Medium - Flaky tests erode confidence  
**Probability:** Low - Using async/await properly  
**Mitigation:**
- Use deterministic timeouts
- Avoid race conditions
- Run tests multiple times to verify stability

---

## Dependencies

### Requires Task 1.1 Remediation

**WHY:** Task 1.2 tests need messages to **arrive** at ComponentActor:
- If Task 1.1 delivery is still stubbed, messages never reach ComponentActor
- Task 1.2 integration tests cannot prove end-to-end flow
- Must complete Task 1.1 remediation first

**Order of Operations:**
1. Complete Task 1.1 Remediation (delivery side)
2. Then complete Task 1.2 Remediation (reception side)
3. Then verify full end-to-end flow

### No External Dependencies

- All fixtures exist in `tests/fixtures/`
- All types exist in crate
- No new dependencies needed

---

## Implementation Notes

### What to Change:

1. **component_actor.rs** - Fix parameter marshalling TODO
2. **NEW test file** - Create message_reception_integration_tests.rs
3. **messaging_reception_tests.rs** - Update comments for clarity

### What NOT to Change:

- ❌ Don't delete existing 41 tests (they validate APIs)
- ❌ Don't change MessageReceptionMetrics (it works)
- ❌ Don't change BackpressureConfig (it works)

### Estimated Lines of Code

1. **Parameter marshalling fix:** ~20 lines
2. **Integration test file:** ~300 lines
3. **Comment updates:** ~20 lines
4. **Unit test additions:** ~40 lines

Total: **~380 lines of code**

---

## References

- **ADR-WASM-020:** Message Delivery Ownership Architecture (Accepted 2025-12-21)
- **KNOWLEDGE-WASM-026:** Message Delivery Architecture - Final Decision
- **AGENTS.md Section 8:** Mandatory Testing Requirements
- **Task 1.1 Remediation Plan:** task-006-phase-1-task-1.1-remediation-plan.md

### Source Files

- `airssys-wasm/src/actor/component/component_actor.rs` - Lines 2005-2088 (invoke_handle_message_with_timeout)
- `airssys-wasm/tests/messaging_reception_tests.rs` - Lines 271-306 (limitation admission)
- `airssys-wasm/tests/fixtures/*.wasm` - Existing WASM fixtures

---

## Approval

**Plan Status:** Proposed  
**Reviewer:** [To be assigned]  
**Approved By:** [To be filled]  
**Approval Date:** [To be filled]

---

**Do you approve this remediation plan? (Yes/No)**
