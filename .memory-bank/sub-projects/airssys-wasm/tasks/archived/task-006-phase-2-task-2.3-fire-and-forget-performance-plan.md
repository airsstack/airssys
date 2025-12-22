# Action Plan for WASM-TASK-006 Phase 2 Task 2.3: Fire-and-Forget Performance

**Status:** PENDING APPROVAL  
**Created:** 2025-12-22  
**Revised:** 2025-12-22 (v2 - Flaky Test Removal)  
**Supersedes:** N/A (new plan)  
**Priority:** High - Completes Phase 2 Fire-and-Forget Messaging  
**Estimated Effort:** 12-21 hours

---

## Goal

Validate and document the fire-and-forget messaging performance, ensuring the system meets the ~280ns latency target and >10,000 msg/sec throughput target through comprehensive **benchmarks** (authoritative performance validation) and **integration tests** (correctness verification).

---

## Context & References

### Architectural References
- **ADR-WASM-009**: Component Communication Model - Defines ~280ns latency target (211ns routing + 49ns overhead + 20ns WASM call)
- **ADR-WASM-001**: Multicodec Compatibility Strategy - Serialization format handling
- **KNOWLEDGE-WASM-005**: Inter-Component Messaging Architecture - Implementation guide

### Dependency Status
- **Task 2.1**: ✅ COMPLETE - `SendMessageHostFunction` at `src/runtime/async_host.rs:446-545`
- **Task 2.2**: ✅ COMPLETE - `WasmEngine::call_handle_message()` at `src/runtime/engine.rs:455-531`

### Performance Targets (from ADR-WASM-009)
| Metric | Target | Source |
|--------|--------|--------|
| Total Fire-and-Forget Latency | ~280ns | ADR-WASM-009 |
| MessageBroker Routing | ~211ns | RT-TASK-008 (proven) |
| Host Validation Overhead | ~50ns | ADR-WASM-009 |
| WASM Call Overhead | ~20ns | ADR-WASM-009 |
| Throughput | >10,000 msg/sec | ADR-WASM-009 |

### Existing Benchmarks
- `benches/messaging_benchmarks.rs` - Existing MessageRouter and CorrelationTracker benchmarks
- `benches/routing_benchmarks.rs` - Component routing benchmarks
- `benches/capability_check_benchmarks.rs` - Security check benchmarks

---

## Performance Validation Strategy

**Separation of Concerns:**

| Test Type | Purpose | Timing Assertions? |
|-----------|---------|-------------------|
| **Integration Tests** | Verify CORRECTNESS (code paths work) | ❌ NO timing |
| **Benchmarks** | Measure PERFORMANCE (latency, throughput) | ✅ YES timing |

**Integration Tests** verify correctness:
- Message delivery works end-to-end
- Validation accepts/rejects appropriately
- Concurrent operations are stable
- Various payload sizes handled

**Benchmarks** (run locally via `cargo bench`) validate performance:
- Latency meets ~280ns target
- Throughput exceeds >10,000 msg/sec
- Overhead breakdown matches ADR-WASM-009

This separation ensures:
- ✅ CI tests are stable and deterministic
- ✅ Performance is still validated (via benchmarks)
- ✅ No flaky tests due to timing variability

---

## Fixture Verification ✅

**STATUS: READY** - All fixtures verified to exist

| Fixture | Path | Purpose | Status |
|---------|------|---------|--------|
| handle-message-component.wasm | tests/fixtures/ | Component Model handle-message | ✅ EXISTS |
| basic-handle-message.wasm | tests/fixtures/ | Core WASM handle-message | ✅ EXISTS |
| echo-handler.wasm | tests/fixtures/ | Echo response handler | ✅ EXISTS |
| slow-handler.wasm | tests/fixtures/ | Timeout testing | ✅ EXISTS |
| rejecting-handler.wasm | tests/fixtures/ | Error handling | ✅ EXISTS |
| hello_world.wasm | tests/fixtures/ | Basic component | ✅ EXISTS |

**No BLOCKERS** - All required fixtures are available.

---

## Implementation Steps

### Step 1: Create Fire-and-Forget Benchmarks (4-6 hours)

**Objective:** Create comprehensive benchmarks for fire-and-forget messaging performance.

**File:** `benches/fire_and_forget_benchmarks.rs`

**Benchmarks to Implement:**

#### Category A: Overhead Breakdown (5 benchmarks)
1. `bench_host_validation_overhead` - Measure SendMessageHostFunction validation (~50ns target)
2. `bench_message_serialization_overhead` - Measure multicodec encoding
3. `bench_broker_publish_overhead` - Measure MessageBroker publish (~211ns target)
4. `bench_wasm_call_overhead` - Measure call_handle_message invocation (~20ns target)
5. `bench_total_fire_and_forget_latency` - End-to-end measurement (~280ns target)

#### Category B: Throughput (4 benchmarks)
6. `bench_throughput_single_sender` - Single component sending messages
7. `bench_throughput_10_senders` - 10 concurrent senders
8. `bench_throughput_100_messages_burst` - Burst of 100 messages
9. `bench_sustained_throughput_1000` - Sustained load (>10,000 msg/sec target)

#### Category C: Latency Distribution (4 benchmarks)
10. `bench_latency_p50` - Median latency
11. `bench_latency_p95` - 95th percentile latency
12. `bench_latency_p99` - 99th percentile latency
13. `bench_latency_under_load` - Latency with concurrent operations

**Deliverables:**
- [ ] `benches/fire_and_forget_benchmarks.rs` created
- [ ] 13 benchmarks implemented
- [ ] All benchmarks use real WASM fixtures (handle-message-component.wasm)
- [ ] Criterion configuration with 100 samples, 5s measurement window

### Step 2: Create Correctness Integration Tests (4-6 hours)

**Objective:** Create integration tests that verify **correctness** (NOT performance). No timing assertions.

**File:** `tests/fire_and_forget_performance_tests.rs`

**Tests to Implement (8 correctness-focused tests):**

| Test Name | Purpose | Fixture | Success Criteria |
|-----------|---------|---------|------------------|
| `test_end_to_end_message_delivery` | Verify complete message flow works | handle-message-component.wasm | Message delivered, no errors |
| `test_sustained_message_delivery` | Verify 1000 messages delivered correctly | handle-message-component.wasm | All 1000 delivered, no errors |
| `test_host_validation_accepts_valid` | Verify validation passes for valid messages | basic-handle-message.wasm | Validation succeeds |
| `test_host_validation_rejects_invalid` | Verify validation fails for invalid messages | basic-handle-message.wasm | Validation fails with error |
| `test_wasm_handle_message_invoked` | Verify WASM function is actually called | echo-handler.wasm | Function called, returns success |
| `test_concurrent_senders_stable` | Verify 10 concurrent senders work | handle-message-component.wasm | All messages delivered, no races |
| `test_large_payload_delivery` | Verify 64KB payload handled correctly | handle-message-component.wasm | Payload delivered intact |
| `test_small_payload_delivery` | Verify 16-byte payload handled correctly | handle-message-component.wasm | Payload delivered intact |

**Key Design Decisions (NO FLAKY TESTS):**
- ❌ NO latency thresholds (no `<500ns`, `<100ns`, etc.)
- ❌ NO throughput thresholds (no `>5,000 msg/sec`)
- ❌ NO regression comparisons based on timing
- ✅ YES correctness verification (messages delivered, no errors)
- ✅ YES error handling (invalid inputs rejected)

**Deliverables:**
- [ ] `tests/fire_and_forget_performance_tests.rs` created
- [ ] 8 correctness-focused integration tests implemented
- [ ] NO timing assertions in any test
- [ ] All tests use real WASM fixtures

### Step 3: Measure and Document Overhead Breakdown (2-3 hours)

**Objective:** Document the exact overhead contribution of each layer.

**Actions:**
1. Run benchmarks to collect data for each component
2. Create overhead breakdown table
3. Document any deviations from ADR-WASM-009 targets
4. Identify optimization opportunities if targets not met

**Expected Overhead Breakdown (from ADR-WASM-009):**

| Component | Target | Measurement Method |
|-----------|--------|-------------------|
| Host Validation | ~50ns | bench_host_validation_overhead |
| MessageBroker Routing | ~211ns | bench_broker_publish_overhead |
| WASM Call | ~20ns | bench_wasm_call_overhead |
| **Total** | **~280ns** | bench_total_fire_and_forget_latency |

**Deliverables:**
- [ ] Overhead breakdown table with actual measurements
- [ ] Comparison against ADR-WASM-009 targets
- [ ] Performance documentation in task completion file

### Step 4: Optimize if Targets Not Met (0-4 hours, conditional)

**Objective:** If benchmarks show performance below targets, implement optimizations.

**Potential Optimizations:**
1. Reduce validation overhead (cache capability lookups)
2. Optimize message serialization (zero-copy where possible)
3. Reduce WASM invocation overhead (pre-compiled functions)
4. Improve broker efficiency (batching, pooling)

**Note:** This step only executes if Step 3 reveals performance gaps.

**Deliverables (conditional):**
- [ ] Optimization implemented (if needed)
- [ ] Re-benchmarks confirm improvement
- [ ] Documentation of optimization changes

### Step 5: Create Performance Documentation (2-3 hours)

**Objective:** Document performance characteristics for users and future maintainers.

**Deliverables:**
- [ ] Update `examples/fire_and_forget_messaging.rs` with performance notes
- [ ] Add performance section to task completion file
- [ ] Document benchmark reproduction steps

---

## Unit Testing Plan

**MANDATORY**: Tests in module #[cfg(test)] blocks

No new unit tests required for Task 2.3 - this is a performance validation task, not a feature implementation task. The core functionality was implemented and unit tested in Tasks 2.1 and 2.2.

**Existing Unit Test Coverage:**
- `src/runtime/async_host.rs` - 8 unit tests for SendMessageHostFunction (Task 2.1)
- `src/runtime/engine.rs` - 4 unit tests for call_handle_message (Task 2.2)

**Verification:** `cargo test --package airssys-wasm --lib` - all existing tests remain passing

---

## Integration Testing Plan

**MANDATORY**: Tests in tests/ directory

**Test File:** `tests/fire_and_forget_performance_tests.rs`

**DESIGN PRINCIPLE:** These tests verify CORRECTNESS only. No timing assertions.

| Test | Purpose | Fixture | Success Criteria |
|------|---------|---------|------------------|
| `test_end_to_end_message_delivery` | Verify complete fire-and-forget flow works | handle-message-component.wasm | Message delivered successfully, no errors |
| `test_sustained_message_delivery` | Verify 1000 messages can be delivered | handle-message-component.wasm | All 1000 messages delivered, no errors |
| `test_host_validation_accepts_valid` | Verify validation allows valid messages | basic-handle-message.wasm | Validation passes, message proceeds |
| `test_host_validation_rejects_invalid` | Verify validation rejects invalid messages | basic-handle-message.wasm | Validation fails with appropriate error |
| `test_wasm_handle_message_invoked` | Verify WASM function actually runs | echo-handler.wasm | Function called, returns expected result |
| `test_concurrent_senders_stable` | Verify 10 concurrent senders work | handle-message-component.wasm | All messages delivered, no race conditions |
| `test_large_payload_delivery` | Verify 64KB payload is handled | handle-message-component.wasm | Payload delivered intact |
| `test_small_payload_delivery` | Verify 16-byte payload is handled | handle-message-component.wasm | Payload delivered intact |

**What These Tests DO NOT Have:**
- ❌ Latency assertions (no `<500ns`, no `<280ns`)
- ❌ Throughput assertions (no `>5,000 msg/sec`, no `>10,000 msg/sec`)
- ❌ Timing-based regression checks

**What These Tests DO Have:**
- ✅ Success/failure verification
- ✅ Error handling verification
- ✅ Message integrity verification
- ✅ Concurrency stability verification

**Verification:** `cargo test --package airssys-wasm --test fire_and_forget_performance_tests` - all tests passing

---

## Benchmarks Plan

**MANDATORY**: Benchmarks in benches/ directory

**Benchmark File:** `benches/fire_and_forget_benchmarks.rs`

**These are the AUTHORITATIVE performance validation.** Benchmarks measure actual timing and are run locally, not in CI.

### Overhead Breakdown Benchmarks (5)

| Benchmark | Target | Measures |
|-----------|--------|----------|
| `bench_host_validation_overhead` | ~50ns | SendMessageHostFunction validation |
| `bench_message_serialization_overhead` | <100ns | Multicodec encode/decode |
| `bench_broker_publish_overhead` | ~211ns | InMemoryMessageBroker::publish |
| `bench_wasm_call_overhead` | ~20ns | WasmEngine::call_handle_message |
| `bench_total_fire_and_forget_latency` | ~280ns | End-to-end flow |

### Throughput Benchmarks (4)

| Benchmark | Target | Measures |
|-----------|--------|----------|
| `bench_throughput_single_sender` | >10,000 msg/sec | Single sender throughput |
| `bench_throughput_10_senders` | >50,000 msg/sec | Concurrent scaling |
| `bench_throughput_100_messages_burst` | <10ms for 100 | Burst performance |
| `bench_sustained_throughput_1000` | >10,000 msg/sec | Sustained load |

### Latency Distribution Benchmarks (4)

| Benchmark | Target | Measures |
|-----------|--------|----------|
| `bench_latency_p50` | <300ns | Median latency |
| `bench_latency_p95` | <1μs | 95th percentile |
| `bench_latency_p99` | <10μs | 99th percentile |
| `bench_latency_under_load` | <500ns | Latency under concurrent load |

**Verification:** `cargo bench --package airssys-wasm --bench fire_and_forget_benchmarks`

---

## Verification Commands

```bash
# Build verification
cargo build --package airssys-wasm

# Unit tests
cargo test --package airssys-wasm --lib

# Integration tests (correctness only, no timing)
cargo test --package airssys-wasm --test fire_and_forget_performance_tests

# Clippy
cargo clippy --package airssys-wasm -- -D warnings

# Run benchmarks (authoritative performance validation - run locally)
cargo bench --package airssys-wasm --bench fire_and_forget_benchmarks
```

---

## Quality Verification

- [ ] `cargo build --package airssys-wasm` - builds cleanly
- [ ] `cargo test --package airssys-wasm --lib` - all unit tests pass
- [ ] `cargo test --package airssys-wasm --test fire_and_forget_performance_tests` - all integration tests pass
- [ ] `cargo clippy --package airssys-wasm -- -D warnings` - zero warnings
- [ ] `cargo bench --package airssys-wasm --bench fire_and_forget_benchmarks` - all benchmarks complete
- [ ] Zero compiler warnings
- [ ] Performance documentation complete

---

## Verification Steps

1. **Build verification:**
   ```bash
   cargo build --package airssys-wasm
   ```
   - Expected: No warnings, builds cleanly

2. **Run unit tests:**
   ```bash
   cargo test --package airssys-wasm --lib
   ```
   - Expected: All 955+ tests passing

3. **Run integration tests:**
   ```bash
   cargo test --package airssys-wasm --test fire_and_forget_performance_tests
   ```
   - Expected: All 8 correctness tests passing (no flaky failures)

4. **Run Clippy:**
   ```bash
   cargo clippy --package airssys-wasm -- -D warnings
   ```
   - Expected: Zero warnings

5. **Run benchmarks (local only, not CI):**
   ```bash
   cargo bench --package airssys-wasm --bench fire_and_forget_benchmarks
   ```
   - Expected: All 13 benchmarks complete with results
   - Expected: Benchmark results show ~280ns latency, >10,000 msg/sec throughput

---

## Success Criteria Summary

### Task 2.3 Requirements vs. Deliverables

| Requirement | Deliverable | Status |
|-------------|-------------|--------|
| End-to-end message delivery verification | `test_end_to_end_message_delivery` | ⏳ Planned |
| Sustained message delivery verification | `test_sustained_message_delivery` | ⏳ Planned |
| Host validation correctness | `test_host_validation_accepts_valid`, `test_host_validation_rejects_invalid` | ⏳ Planned |
| WASM invocation verification | `test_wasm_handle_message_invoked` | ⏳ Planned |
| Concurrency stability verification | `test_concurrent_senders_stable` | ⏳ Planned |
| Payload handling verification | `test_large_payload_delivery`, `test_small_payload_delivery` | ⏳ Planned |
| Latency benchmarks (authoritative) | 5 overhead benchmarks | ⏳ Planned |
| Throughput benchmarks (authoritative) | 4 throughput benchmarks | ⏳ Planned |
| Latency distribution benchmarks | 4 latency distribution benchmarks | ⏳ Planned |
| Performance documentation | Task completion file + example updates | ⏳ Planned |
| Total latency: ~280ns | Benchmark validation (not integration test) | ⏳ Planned |
| Throughput: >10,000 msg/sec | Benchmark validation (not integration test) | ⏳ Planned |

### Checklist

- [ ] All 13 benchmarks created and running
- [ ] All 8 integration tests created and passing (correctness only, no timing)
- [ ] NO timing assertions in integration tests
- [ ] Overhead breakdown documented with actual measurements
- [ ] Performance meets ADR-WASM-009 targets (validated by benchmarks)
- [ ] Documentation complete
- [ ] Zero clippy warnings
- [ ] Zero compiler warnings

---

## Estimated Time

| Step | Hours (Min-Max) |
|------|-----------------|
| 1. Create Benchmarks | 4-6 |
| 2. Create Correctness Integration Tests | 4-6 |
| 3. Measure and Document | 2-3 |
| 4. Optimize (conditional) | 0-4 |
| 5. Documentation | 2-3 |
| **Total** | **12-21** |

---

## Files to Create

| File | Purpose | Lines (Est.) |
|------|---------|--------------|
| `benches/fire_and_forget_benchmarks.rs` | 13 performance benchmarks | ~400 |
| `tests/fire_and_forget_performance_tests.rs` | 8 correctness integration tests | ~300 |

## Files to Update

| File | Purpose |
|------|---------|
| `examples/fire_and_forget_messaging.rs` | Add performance notes |
| `tasks/task-006-phase-2-task-2.3-fire-and-forget-performance-plan.md` | Completion status |

---

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| v1 | 2025-12-22 | Original plan with timing-based integration tests |
| v2 | 2025-12-22 | **Flaky Test Removal** - Replaced timing-based integration tests with correctness-focused tests. Removed CI Tolerance section. Added Performance Validation Strategy. Benchmarks remain unchanged as authoritative performance validation. |

---

## Plan Approval

**Plan Status:** ⏳ PENDING APPROVAL  
**Created By:** @memorybank-planner  
**Revised By:** @memorybank-planner  
**Date:** 2025-12-22 (v2)

**Awaiting Approval:** Do you approve this revised plan? (Yes/No)
