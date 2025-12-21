# WASM-TASK-006 Phase 2 Task 2.2 - handle-message Component Export

**Status:** planned  
**Created:** 2025-12-21  
**Estimated Effort:** 24 hours  
**Priority:** High - Completes Fire-and-Forget Pattern

---

## Executive Summary

Task 2.2 implements the **component-side message handling** that completes the fire-and-forget messaging pattern. Task 2.1 provided the `send-message` host function (sending side), and Task 2.2 provides the `handle-message` component export (receiving side) with proper sender metadata, payload handling, and error propagation.

**Key Insight**: The WIT interface for `handle-message` already exists at `wit/core/component-lifecycle.wit:86-89`:
```wit
handle-message: func(
    sender: component-id,
    message: list<u8>
) -> result<_, component-error>;
```

However, the current implementation uses a **simplified parameterless invocation** (TODO at `component_actor.rs:2051-2052`). Task 2.2 must implement **proper parameter marshalling** to pass sender metadata and message payload to the WASM handle-message export.

---

## Goal

Implement full WIT-compliant `handle-message` component export invocation with:
1. Sender metadata (component-id) passed to WASM
2. Message payload (list<u8>) passed to WASM
3. Result handling (success/component-error) propagated back
4. Updated test fixtures that accept parameters
5. Real integration tests proving end-to-end functionality

---

## Context & References

### Architectural Alignment
- **ADR-WASM-001**: Multicodec Compatibility Strategy - Payload is self-describing
- **ADR-WASM-009**: Component Communication Model - Fire-and-forget pattern
- **ADR-WASM-020**: Message Delivery Ownership - ActorSystemSubscriber delivers to mailboxes
- **KNOWLEDGE-WASM-024**: Component Messaging Clarifications - Direct ComponentId addressing
- **KNOWLEDGE-WASM-026**: Message Delivery Architecture - Final decision

### Existing Infrastructure (Task 1.2 - COMPLETE)
- `invoke_handle_message_with_timeout()` at `src/actor/component/component_actor.rs:2000-2095`
- Result slot allocation fixed (line 2055)
- 9 integration tests prove WASM export is invoked
- WAT fixtures: `basic-handle-message.wat`, `rejecting-handler.wat`, `slow-handler.wat`

### Current Limitation (TODO to Address)
From `component_actor.rs:2051-2052`:
```rust
// TODO(WASM-TASK-006 Phase 2 Task 2.2): Implement proper parameter
// marshalling using wasmtime component model bindings once generated.
```

Current invocation is parameterless: `handle_fn.call_async(&mut *runtime.store_mut(), &[], &mut results)`

**This task removes the TODO and implements full parameter marshalling.**

---

## Technical Analysis

### What Exists
1. ✅ WIT interface definition (`wit/core/component-lifecycle.wit:86-89`)
2. ✅ `invoke_handle_message_with_timeout()` method structure
3. ✅ WASM runtime integration (Engine, Store, Instance)
4. ✅ Timeout enforcement with tokio
5. ✅ Error handling for traps and timeouts
6. ✅ Message reception metrics (`MessageReceptionMetrics`)
7. ✅ Test fixtures (WAT format, parameterless)

### What Needs Implementation
1. ❌ Parameter marshalling for `sender: component-id`
2. ❌ Parameter marshalling for `message: list<u8>`
3. ❌ Result parsing for `result<_, component-error>`
4. ❌ Memory allocation in WASM for parameters
5. ❌ Updated test fixtures with parameters
6. ❌ Integration tests proving parameter passing

### WASM Memory Considerations

Since we're using **core WASM modules** (not Component Model), parameter passing requires:
1. **Memory allocation**: Allocate space in WASM linear memory for sender and payload
2. **Data copying**: Copy sender bytes and payload bytes into WASM memory
3. **Pointer passing**: Pass pointers (i32) and lengths (i32) to the function
4. **Result interpretation**: Parse i32 return value (0=success, non-zero=error code)

**WIT to Core WASM Signature Translation:**
```
WIT: handle-message(sender: component-id, message: list<u8>) -> result<_, component-error>

Core WASM (Simplified): func(
    sender_ptr: i32,   // Pointer to serialized sender (JSON or simple string)
    sender_len: i32,   // Sender string length
    message_ptr: i32,  // Pointer to message bytes
    message_len: i32   // Message length
) -> i32  // 0 = success, non-zero = error code
```

---

## Implementation Steps

### Step 1: Memory Export Detection (2 hours)
**Deliverables:**
- Add memory export detection in `WasmRuntime`
- Implement `get_memory()` method to access WASM linear memory
- Handle case where memory is not exported

**Success Criteria:**
- Can access WASM memory export
- Error handling for missing memory export

### Step 2: Memory Allocation Helper (4 hours)
**Deliverables:**
- Implement `WasmMemoryAllocator` helper struct
- Track allocated regions in WASM memory
- Implement simple bump allocator or use WASM malloc if available
- Add memory bounds checking

**Success Criteria:**
- Can allocate N bytes in WASM memory
- Returns pointer (i32) to allocated region
- Prevents buffer overflows

### Step 3: Parameter Marshalling (6 hours)
**Deliverables:**
- Implement `marshal_sender()` to copy sender ComponentId to WASM memory
- Implement `marshal_message()` to copy message bytes to WASM memory
- Create `HandleMessageParams` struct for parameter packaging
- Update `invoke_handle_message_with_timeout()` to use marshalled parameters

**Success Criteria:**
- Sender ID copied to WASM memory as UTF-8 string
- Message bytes copied to WASM memory
- Pointers/lengths passed to handle-message function

### Step 4: Result Parsing (2 hours)
**Deliverables:**
- Parse i32 return value from handle-message
- Map error codes to `WasmError` variants
- Add logging for non-zero results
- Document error code conventions

**Success Criteria:**
- 0 = success, properly returns `Ok(())`
- Non-zero = error, properly returns `Err(WasmError::...)`
- Error codes logged with context

### Step 5: Update Test Fixtures (4 hours)
**Deliverables:**
- Update `basic-handle-message.wat` to accept parameters
- Update `rejecting-handler.wat` to accept parameters
- Update `slow-handler.wat` to accept parameters
- Create `echo-handler.wat` that echoes received data (for validation)
- Create `sender-validator.wat` that validates sender metadata

**WAT Fixture Template (with parameters):**
```wat
(module
  (memory (export "memory") 1)
  
  ;; Handle message with sender and payload
  ;; Args: sender_ptr, sender_len, message_ptr, message_len
  ;; Returns: 0 on success, non-zero on error
  (func $handle_message (export "handle-message")
    (param $sender_ptr i32)
    (param $sender_len i32)
    (param $message_ptr i32)
    (param $message_len i32)
    (result i32)
    
    ;; Success - return 0
    i32.const 0
  )
)
```

**Success Criteria:**
- All fixtures compile with `wasm-tools parse`
- All fixtures export `handle-message` with correct signature
- All fixtures export `memory`

### Step 6: Update Integration Tests (4 hours)
**Deliverables:**
- Update `load_wasm_fixture_into_actor()` to handle new fixture format
- Update existing 9 integration tests for parameter passing
- Add tests proving sender metadata is received
- Add tests proving message payload is received
- Add tests for edge cases (empty payload, long sender, etc.)

**Success Criteria:**
- All 9 existing tests pass with new parameter format
- New tests prove sender/payload are passed correctly
- Edge cases handled (empty strings, max lengths)

### Step 7: Documentation and Cleanup (2 hours)
**Deliverables:**
- Remove TODO comment from `component_actor.rs:2051-2052`
- Add comprehensive documentation for parameter marshalling
- Update Knowledge documentation if needed
- Update fixture README with new signature

**Success Criteria:**
- TODO removed
- All public APIs documented
- Fixture format documented

---

## Unit Testing Plan

**MANDATORY**: Tests in module `#[cfg(test)]` blocks

### Test Location: `src/actor/component/component_actor.rs`

| Test | Description | Proves |
|------|-------------|--------|
| `test_marshal_sender_to_memory` | Marshal ComponentId to WASM memory | Sender serialization works |
| `test_marshal_message_to_memory` | Marshal payload bytes to WASM memory | Payload copying works |
| `test_marshal_empty_sender` | Handle empty sender edge case | Edge case: empty sender |
| `test_marshal_empty_message` | Handle empty message edge case | Edge case: empty payload |
| `test_marshal_large_message` | Handle large message (1MB) | Large payload handling |
| `test_handle_message_params_creation` | Create HandleMessageParams struct | Parameter packaging |
| `test_parse_success_result` | Parse 0 return value | Success path |
| `test_parse_error_result` | Parse non-zero return value | Error path |
| `test_error_code_mapping` | Map error codes to WasmError | Error translation |

**Verification**: `cargo test --lib` - all tests passing

---

## Integration Testing Plan

**MANDATORY**: Tests in `tests/` directory

### Test File: `tests/handle_message_export_integration_tests.rs`

| Test | Description | Proves | Fixture |
|------|-------------|--------|---------|
| `test_handle_message_receives_sender_metadata` | CRITICAL - Sender ID passed to WASM | Sender marshalling works | sender-validator.wasm |
| `test_handle_message_receives_message_payload` | CRITICAL - Payload passed to WASM | Payload marshalling works | echo-handler.wasm |
| `test_handle_message_with_empty_payload` | Empty message accepted | Edge case: empty payload | basic-handle-message.wasm |
| `test_handle_message_with_large_payload` | 1MB message accepted | Large payload handling | basic-handle-message.wasm |
| `test_handle_message_success_result_propagated` | Success (0) returns Ok | Result parsing | basic-handle-message.wasm |
| `test_handle_message_error_result_propagated` | Error (99) returns Err | Error propagation | rejecting-handler.wasm |
| `test_handle_message_with_multicodec_prefix` | Payload with multicodec accepted | ADR-WASM-001 compliance | basic-handle-message.wasm |
| `test_multiple_messages_sequential` | Multiple messages to same component | Sequential processing | basic-handle-message.wasm |
| `test_end_to_end_send_to_handle` | send-message → handle-message flow | Full integration | basic-handle-message.wasm |
| `test_sender_id_format_preserved` | ComponentId serialization/deserialization | ID format consistency | sender-validator.wasm |

### Fixture Requirements

| Fixture | Status | Purpose |
|---------|--------|---------|
| `basic-handle-message.wat` | ⚠️ UPDATE | Accept params, return 0 |
| `rejecting-handler.wat` | ⚠️ UPDATE | Accept params, return 99 |
| `slow-handler.wat` | ⚠️ UPDATE | Accept params, consume fuel |
| `sender-validator.wat` | 🆕 CREATE | Validate sender is passed |
| `echo-handler.wat` | 🆕 CREATE | Echo received payload |

**Verification**: `cargo test --test handle_message_export_integration_tests` - all tests passing

---

## Quality Verification

- [ ] `cargo build` - builds cleanly
- [ ] `cargo test --lib` - all unit tests pass
- [ ] `cargo test --test handle_message_export_integration_tests` - all integration tests pass
- [ ] `cargo test --test message_reception_integration_tests` - regression tests pass
- [ ] `cargo test --test send_message_host_function_tests` - Task 2.1 tests pass
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` - zero warnings
- [ ] Zero compiler warnings
- [ ] 100% public API documentation

---

## Verification Steps

1. **Build fixtures:**
   ```bash
   cd airssys-wasm/tests/fixtures && ./build.sh
   ```
   - Expected: All .wat files compiled to .wasm

2. **Run unit tests:**
   ```bash
   cargo test --lib -p airssys-wasm
   ```
   - Expected: All tests passing (including new marshalling tests)

3. **Run Task 2.2 integration tests:**
   ```bash
   cargo test --test handle_message_export_integration_tests -p airssys-wasm
   ```
   - Expected: All new integration tests passing

4. **Run regression tests:**
   ```bash
   cargo test --test message_reception_integration_tests -p airssys-wasm
   cargo test --test send_message_host_function_tests -p airssys-wasm
   ```
   - Expected: All Phase 1 and Task 2.1 tests still passing

5. **Run clippy:**
   ```bash
   cargo clippy -p airssys-wasm --all-targets --all-features -- -D warnings
   ```
   - Expected: Zero clippy warnings

6. **Full test suite:**
   ```bash
   cargo test -p airssys-wasm
   ```
   - Expected: All tests passing, zero regressions

---

## Dependencies

### Upstream (Completed)
- ✅ Task 1.2: ComponentActor Message Reception - Provides `invoke_handle_message_with_timeout()`
- ✅ Task 2.1: send-message Host Function - Provides sending side

### Downstream (Enables)
- Task 2.3: Fire-and-Forget Performance - Can measure end-to-end latency
- Phase 3: Request-Response Pattern - Uses same marshalling infrastructure

---

## Risks and Mitigations

### Risk 1: Memory Allocation Complexity
**Impact:** Medium - WASM memory management is error-prone
**Probability:** Medium - No malloc in simple WAT modules
**Mitigation:**
- Use simple bump allocator starting from known offset (e.g., 0x1000)
- Pre-allocate fixed buffer sizes for sender (256 bytes) and payload (64KB)
- Document memory layout clearly

### Risk 2: Signature Compatibility
**Impact:** Medium - Changing function signature breaks existing tests
**Probability:** High - Existing tests use parameterless signature
**Mitigation:**
- Update all fixtures in one batch
- Update all tests in one batch
- Run full test suite after changes

### Risk 3: Performance Overhead
**Impact:** Medium - Memory copying adds latency
**Probability:** Low - Payload sizes typically small
**Mitigation:**
- Measure overhead with benchmarks
- Consider zero-copy for large payloads (Phase 2)
- Document performance characteristics

---

## Estimated Time

| Step | Hours |
|------|-------|
| 1. Memory Export Detection | 2 |
| 2. Memory Allocation Helper | 4 |
| 3. Parameter Marshalling | 6 |
| 4. Result Parsing | 2 |
| 5. Update Test Fixtures | 4 |
| 6. Update Integration Tests | 4 |
| 7. Documentation and Cleanup | 2 |
| **Total** | **24 hours** |

---

## Success Criteria Summary

1. ✅ TODO at `component_actor.rs:2051-2052` is REMOVED
2. ✅ Sender ComponentId is passed to WASM handle-message export
3. ✅ Message payload is passed to WASM handle-message export
4. ✅ Result (success/error) is propagated back to caller
5. ✅ All 9+ existing integration tests pass (no regression)
6. ✅ 10+ NEW integration tests prove sender/payload passing
7. ✅ Zero clippy warnings
8. ✅ Zero compiler warnings
9. ✅ 100% public API documentation

---

## Approval

**Plan Status:** ✅ APPROVED  
**Approved By:** User  
**Approval Date:** 2025-12-21  
**Verified By:** @memorybank-verifier (PARTIAL - technical claims verified)

---

## Next Steps

1. **Implementation**: Invoke `@memorybank-implementer` with this plan
2. **Code Review**: Request `@rust-reviewer` after implementation
3. **Audit**: Request `@memorybank-auditor` after code review passes
4. **Completion**: Update Memory Bank via `@memorybank-completer`
