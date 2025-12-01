# Technical Debt Record: Epoch-Based Preemption Future Enhancement

**Document ID:** DEBT-WASM-002  
**Created:** 2025-10-24  
**Updated:** 2025-10-24  
**Status:** active  
**Category:** DEBT-PERF  

## Summary

Wasmtime epoch-based preemption is not implemented. Current timeout mechanism uses tokio timeout wrapper which cancels Rust async execution but does not interrupt running WASM code. This is acceptable for current use cases but limits ability to preempt long-running or infinite WASM loops.

## Context

### Background

During WASM-TASK-002 Phase 3 (CPU Limiting Implementation), we implemented hybrid CPU limiting with fuel metering and wall-clock timeouts. The timeout mechanism uses `tokio::time::timeout()` to wrap component execution at the Rust async level.

### Decision Point

Task 3.2 initially attempted to enable Wasmtime's epoch interruption feature (`config.epoch_interruption(true)`), but this caused immediate execution traps without proper epoch management infrastructure. Rather than implementing full epoch-based preemption (which requires background threads, epoch deadline management, and careful coordination), we decided to:

1. Use simple tokio timeout wrapper for Task 3.3
2. Document epoch-based preemption as future enhancement
3. Deliver working CPU limits without complexity overhead

### Constraints

- **Limited computational resources:** User has constrained development environment
- **Complexity vs. benefit tradeoff:** Epoch management requires significant infrastructure
- **Pragmatic implementation:** Current tokio timeout mechanism works for most use cases
- **Time to delivery:** Simpler approach allows faster completion of Phase 3

## Technical Details

### Code Location

- **Files:** 
  - `airssys-wasm/src/runtime/engine.rs` (lines 154-156)
  - `airssys-wasm/src/runtime/engine.rs` (lines 269-278: timeout wrapper)
- **Components:** WasmEngine runtime execution
- **Dependencies:** Wasmtime engine configuration, tokio async runtime

### Current Implementation

**Tokio Timeout Wrapper (engine.rs:269-278):**
```rust
// Wrap execution with timeout (hybrid CPU limiting)
let timeout_duration = Duration::from_millis(context.timeout_ms);

match timeout(timeout_duration, self.execute_internal(handle, function, input, context.clone())).await {
    Ok(result) => result,
    Err(_elapsed) => {
        // Timeout exceeded - return ExecutionTimeout error
        Err(WasmError::execution_timeout(context.timeout_ms, None))
    }
}
```

**What This Provides:**
- ✅ Cancels Rust async execution after timeout
- ✅ Works for components that cooperate with async runtime
- ✅ Simple implementation with zero complexity overhead
- ✅ Sufficient for well-behaved components

**What This Does NOT Provide:**
- ❌ Cannot interrupt running WASM code mid-execution
- ❌ Cannot preempt infinite loops in WASM
- ❌ No WASM-level execution interruption

### Impact Assessment

**Performance Impact:**
- Minimal performance impact from current implementation
- Tokio timeout adds negligible overhead (<1μs)
- No background threads consuming resources

**Maintainability Impact:**
- Positive: Simple, easy-to-understand code
- Positive: No complex epoch management infrastructure
- Neutral: Well-documented limitation for future reference

**Security Impact:**
- Limited: Well-behaved components are properly limited
- Risk: Malicious components with tight infinite loops might not be preempted immediately
- Mitigation: Fuel metering provides deterministic CPU limiting (complements timeout)

**Scalability Impact:**
- Neutral: Current implementation scales well
- Future: Epoch-based approach would require per-engine background thread

## Remediation Plan

### Ideal Solution

Implement Wasmtime epoch-based preemption for true WASM-level interruption:

**Architecture:**
```rust
// 1. Enable epoch interruption in engine config
config.epoch_interruption(true);

// 2. Set epoch deadline before execution
let timeout_epochs = (context.timeout_ms / 10) as u64; // 10ms per epoch
store.set_epoch_deadline(timeout_epochs);

// 3. Background epoch increment task
let engine_clone = engine.clone();
let epoch_handle = tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_millis(10)).await;
        engine_clone.increment_epoch();
    }
});

// 4. Execute component (will trap if epoch deadline exceeded)
let result = instance.call(&mut store, function, args).await;

// 5. Cleanup epoch task
epoch_handle.abort();
```

**Benefits:**
- True WASM-level preemption (interrupts running code)
- Can stop infinite loops and malicious components
- More robust timeout enforcement

**Tradeoffs:**
- Adds complexity (background threads, epoch management)
- Requires careful coordination and cleanup
- Additional resource overhead (background task per engine)

### Implementation Steps

1. **Phase 1: Enable Epoch Interruption (1-2 hours)**
   - Enable `config.epoch_interruption(true)` in engine.rs
   - Verify engine initialization succeeds

2. **Phase 2: Epoch Deadline Management (2-3 hours)**
   - Calculate epoch deadline from timeout_ms
   - Set `store.set_epoch_deadline()` before execution
   - Handle epoch exceeded traps properly

3. **Phase 3: Background Epoch Increment (2-4 hours)**
   - Create background tokio task for epoch incrementing
   - Implement proper task lifecycle (spawn/cleanup)
   - Handle engine cloning and thread safety

4. **Phase 4: Testing and Validation (3-4 hours)**
   - Create WASM fixture with infinite loop
   - Verify epoch preemption works correctly
   - Test edge cases (rapid execution, cleanup, errors)
   - Performance testing (overhead measurement)

5. **Phase 5: Documentation and Integration (1-2 hours)**
   - Update rustdoc with epoch behavior
   - Document epoch increment interval (10ms recommended)
   - Integration guide for users

### Effort Estimate

- **Development Time:** 8-15 hours (depending on testing depth)
- **Testing Time:** 3-5 hours (comprehensive validation)
- **Risk Level:** medium (requires careful thread coordination)

### Dependencies

- Wasmtime epoch interruption feature (already available)
- Tokio async runtime (already used)
- No external dependencies required

## Tracking

### GitHub Issue

- **Issue:** Not yet created
- **Labels:** enhancement, performance, wasm-runtime

### Workspace Standards

- **Standards Violated:** None (current implementation is acceptable)
- **Compliance Impact:** No compliance issues
- **Note:** This is enhancement, not technical debt stricto sensu

### Priority

- **Business Priority:** low (current implementation sufficient for most use cases)
- **Technical Priority:** medium (nice-to-have for robustness)
- **Recommended Timeline:** 
  - **Q4 2025 or later:** When malicious component handling becomes critical
  - **Or:** When user reports timeout issues with specific components
  - **Or:** When computational resources improve and complexity is acceptable

## History

### Changes

- **2025-10-24:** Initial debt record created (WASM-TASK-002 Phase 3 Task 3.3)
- **2025-10-24:** Documented decision to use tokio timeout wrapper instead of epoch preemption

### Related Decisions

- **ADR References:** 
  - ADR-WASM-002: Runtime engine architecture (hybrid CPU limiting)
- **Task References:**
  - WASM-TASK-002 Phase 3 Task 3.2: Timeout infrastructure implementation
  - WASM-TASK-002 Phase 3 Task 3.3: CPU limit testing (this task)
- **Other Debt:** 
  - DEBT-WASM-001: Deferred WIT interface abstractions (unrelated)

### Related Knowledge

- **Implementation Reference:** See Task 3.2 completion summary for epoch interruption discovery
- **Test Coverage:** CPU limit tests validate current timeout mechanism (7 tests passing)

## Future Considerations

### When to Implement

Consider implementing epoch-based preemption when:

1. **Malicious Component Handling:** Need to defend against deliberately malicious WASM components
2. **Untrusted Code Execution:** Running completely untrusted third-party components
3. **Strict SLA Requirements:** Need guaranteed preemption for compliance/SLA reasons
4. **User Reports:** Actual user complaints about timeout effectiveness

### When NOT to Implement

Skip epoch-based preemption if:

1. **Trusted Components Only:** All components are from trusted sources
2. **Fuel Limiting Sufficient:** Fuel metering provides adequate deterministic limiting
3. **Resource Constraints:** Development resources better spent on other features
4. **Complexity Overhead:** Maintenance burden outweighs robustness benefits

### Alternative Approaches

If epoch-based preemption proves necessary, consider these alternatives:

1. **External Process Isolation:** Run components in separate processes with OS-level timeouts
2. **WASM Runtime Alternatives:** Evaluate other WASM runtimes with different timeout mechanisms
3. **Hybrid Approach:** Epoch preemption only for untrusted/high-risk components

## Resolution

*[Fill when resolved]*

### Resolution Date

[YYYY-MM-DD]

### Resolution Summary

Brief description of how the debt was resolved (or why it was closed without resolution).

### Lessons Learned

Key insights gained from resolving this debt.

---

**Template Version:** 1.0  
**Document Version:** 1.0  
**Last Updated:** 2025-10-24
