# DEBT-WASM-004: Task 1.3 Deferred Implementation Items

**Created:** 2025-12-13  
**Status:** ACTIVE - MUST BE RESOLVED  
**Priority:** ⚠️ CRITICAL - BLOCKING FUTURE TASKS  
**Category:** Technical Debt / Deferred Work  
**Affects:** Block 3 Phase 2+, Block 4, Block 6

## Executive Summary

Task 1.3 (Actor Trait Message Handling) completed the **message routing infrastructure** but intentionally deferred **5 critical implementation items** to future tasks due to dependency constraints. This document serves as a **MANDATORY CHECKLIST** to ensure NO deferred work is forgotten.

**⚠️ WARNING**: Failure to complete these items will result in:
- Non-functional message invocation (silent failures)
- Security vulnerabilities (no capability enforcement)
- Memory leaks (no registry cleanup)
- Incomplete actor system integration

## Deferred Implementation Items (MANDATORY)

### 1. WASM Function Invocation - Phase 2 Task 2.1 ⚠️ CRITICAL

**Status:** ❌ NOT IMPLEMENTED  
**Location:** `src/actor/actor_impl.rs` lines 181-195  
**Blocks:** Inter-component communication, message passing  
**Must Complete By:** Phase 2 Task 2.1 (ActorSystem Integration)

#### What's Missing
```rust
// CURRENT STATE (Line 181-195):
// - Multicodec deserialization: ✅ WORKING
// - WASM runtime verification: ✅ WORKING
// - Function existence check: ❌ MISSING
// - WASM function call: ❌ MISSING
// - Result serialization: ❌ MISSING
```

#### Required Implementation
```rust
// MUST IMPLEMENT IN PHASE 2 TASK 2.1:

// 1. Get function handle from WASM instance
let func = runtime
    .instance()
    .get_func(runtime.store_mut(), &function)
    .ok_or_else(|| WasmError::execution_failed(...))?;

// 2. Convert decoded_args to Vec<Val> for Wasmtime
let wasm_params = convert_to_wasm_vals(&decoded_args, func.ty(store))?;

// 3. Call WASM function asynchronously
let results = func
    .call_async(runtime.store_mut(), &wasm_params)
    .await
    .map_err(|e| WasmError::execution_failed_with_source(...))?;

// 4. Convert results back to bytes
let result_bytes = convert_from_wasm_vals(&results)?;

// 5. Encode with multicodec
let encoded_result = encode_multicodec(codec, &result_bytes)?;

// 6. Send reply via ActorContext
ctx.reply(ComponentMessage::InvokeResult {
    result: encoded_result,
    error: None,
}).await?;
```

#### Validation Criteria
- [ ] Function export verified before call
- [ ] Type conversion handles all WASM types (i32, i64, f32, f64, externref)
- [ ] Async execution works correctly
- [ ] WASM traps handled gracefully
- [ ] Result serialization works for all codecs
- [ ] Reply sent successfully via ActorContext
- [ ] Test coverage ≥90% for invocation path
- [ ] Performance: <100μs overhead per call

#### Estimated Effort
**8-12 hours** (type conversion system + testing)

---

### 2. InterComponent WASM Call - Phase 2 Task 2.1 ⚠️ CRITICAL

**Status:** ❌ NOT IMPLEMENTED  
**Location:** `src/actor/actor_impl.rs` lines 236-246  
**Blocks:** Component-to-component messaging  
**Must Complete By:** Phase 2 Task 2.1 (ActorSystem Integration)

#### What's Missing
```rust
// CURRENT STATE (Line 236-246):
// - Export detection: ✅ WORKING
// - Parameter preparation: ❌ MISSING
// - WASM call: ❌ MISSING
// - Error handling: ❌ MISSING
```

#### Required Implementation
```rust
// MUST IMPLEMENT IN PHASE 2 TASK 2.1:

if let Some(handle_fn) = &runtime.exports().handle_message {
    // 1. Convert payload to WASM parameters
    let wasm_params = prepare_handle_message_params(&payload)?;
    
    // 2. Call handle-message export
    handle_fn
        .call_async(runtime.store_mut(), &wasm_params)
        .await
        .map_err(|e| WasmError::execution_failed_with_source(
            format!("handle-message trap in {}", component_id_str),
            Box::new(e)
        ))?;
    
    // 3. Log success
    debug!("handle-message completed successfully");
}
```

#### Validation Criteria
- [ ] Payload correctly marshalled to WASM
- [ ] handle-message export called successfully
- [ ] Traps propagated to supervisor
- [ ] Performance: <1ms per message
- [ ] Test coverage ≥90%

#### Estimated Effort
**4-6 hours** (parameter marshalling + testing)

---

### 3. Capability Enforcement - Block 4 🔒 SECURITY

**Status:** ❌ NOT IMPLEMENTED  
**Location:** `src/actor/actor_impl.rs` lines 223-228  
**Blocks:** Security isolation, capability-based access control  
**Must Complete By:** Block 4 (Security & Isolation Layer)  
**Security Risk:** HIGH - No access control enforcement

#### What's Missing
```rust
// CURRENT STATE (Line 223-228):
// - Capability checking: ❌ MISSING
// - Security validation: ❌ MISSING
```

#### Required Implementation
```rust
// MUST IMPLEMENT IN BLOCK 4:

// 1. Check if sender is allowed to send to this component
if !self.capabilities().allows_receiving_from(&sender) {
    return Err(WasmError::capability_denied(
        format!("Component {} not authorized to send to {}", 
                sender.as_str(), 
                component_id_str)
    ));
}

// 2. Validate payload size limits
if payload.len() > self.capabilities().max_message_size() {
    return Err(WasmError::capability_denied("Message too large"));
}

// 3. Rate limiting check
if !self.capabilities().check_rate_limit(&sender) {
    return Err(WasmError::capability_denied("Rate limit exceeded"));
}
```

#### Validation Criteria
- [ ] Capability checks prevent unauthorized access
- [ ] Rate limiting prevents DoS attacks
- [ ] Size limits prevent memory exhaustion
- [ ] Security tests verify enforcement
- [ ] Performance: <5μs overhead per check
- [ ] Test coverage ≥95% (security-critical)

#### Estimated Effort
**16-20 hours** (capability system + security tests)

---

### 4. Health Check Export Parsing - Phase 3 Task 3.3 🏥 MONITORING

**Status:** ❌ NOT IMPLEMENTED  
**Location:** `src/actor/actor_impl.rs` lines 270-280  
**Blocks:** Component health monitoring, supervisor decisions  
**Must Complete By:** Phase 3 Task 3.3 (Component Health Monitoring)

#### What's Missing
```rust
// CURRENT STATE (Line 270-280):
// - Export detection: ✅ WORKING
// - Health call: ❌ MISSING
// - Return value parsing: ❌ MISSING
```

#### Required Implementation
```rust
// MUST IMPLEMENT IN PHASE 3 TASK 3.3:

if let Some(health_fn) = &runtime.exports().health {
    // 1. Call _health export (returns i32: 0=healthy, 1=degraded, 2=unhealthy)
    let results = health_fn
        .call_async(runtime.store_mut(), &[])
        .await
        .map_err(|e| {
            // Trap during health check = unhealthy
            HealthStatus::Unhealthy {
                reason: format!("Health check trapped: {}", e),
            }
        })?;
    
    // 2. Parse return value
    let status_code = results
        .get(0)
        .and_then(|v| v.i32())
        .ok_or(WasmError::execution_failed("Invalid health return type"))?;
    
    // 3. Convert to HealthStatus
    let health = match status_code {
        0 => HealthStatus::Healthy,
        1 => HealthStatus::Degraded { 
            reason: "Component reported degraded state".to_string() 
        },
        2 => HealthStatus::Unhealthy { 
            reason: "Component reported unhealthy state".to_string() 
        },
        _ => HealthStatus::Unhealthy { 
            reason: format!("Invalid health code: {}", status_code) 
        },
    };
    
    // 4. Send reply
    ctx.reply(ComponentMessage::HealthStatus(health)).await?;
}
```

#### Validation Criteria
- [ ] _health export called successfully
- [ ] Return values parsed correctly
- [ ] All health states handled
- [ ] Traps handled as unhealthy
- [ ] Reply sent via ActorContext
- [ ] Test coverage ≥90%

#### Estimated Effort
**4-6 hours** (return value parsing + testing)

---

### 5. Component Registry Integration - Block 6 💾 PERSISTENCE

**Status:** ❌ NOT IMPLEMENTED  
**Location:** `src/actor/actor_impl.rs` lines 364-372 (pre_start), 400-407 (post_stop)  
**Blocks:** Component lifecycle, persistence, restart recovery  
**Must Complete By:** Block 6 (Persistent Storage System)

#### What's Missing
```rust
// CURRENT STATE:
// pre_start (lines 364-372):
// - Registry registration: ❌ MISSING
// - Mailbox setup: ❌ MISSING

// post_stop (lines 400-407):
// - Registry cleanup: ❌ MISSING
// - Verification: ❌ MISSING
```

#### Required Implementation (pre_start)
```rust
// MUST IMPLEMENT IN BLOCK 6:

async fn pre_start<B: MessageBroker<Self::Message>>(
    &mut self,
    ctx: &mut ActorContext<Self::Message, B>,
) -> Result<(), Self::Error> {
    let component_id = self.component_id().clone();
    
    // 1. Register with component registry
    ctx.registry
        .register(component_id.clone(), self.metadata().clone())
        .await
        .map_err(|e| ComponentActorError::from(
            WasmError::storage_error_with_source("Registry registration failed", e)
        ))?;
    
    // 2. Setup mailbox receiver (if needed)
    // self.mailbox_rx = Some(ctx.mailbox.clone());
    
    // 3. Log successful registration
    debug!("Component {} registered with registry", component_id.as_str());
    
    Ok(())
}
```

#### Required Implementation (post_stop)
```rust
// MUST IMPLEMENT IN BLOCK 6:

async fn post_stop<B: MessageBroker<Self::Message>>(
    &mut self,
    ctx: &mut ActorContext<Self::Message, B>,
) -> Result<(), Self::Error> {
    let component_id = self.component_id().clone();
    
    // 1. Deregister from component registry
    ctx.registry
        .unregister(&component_id)
        .await
        .map_err(|e| ComponentActorError::from(
            WasmError::storage_error_with_source("Registry cleanup failed", e)
        ))?;
    
    // 2. Verify WASM runtime cleaned up
    if self.is_wasm_loaded() {
        warn!("WASM runtime still loaded during post_stop (potential leak)");
    }
    
    // 3. Transition to Terminated
    self.set_state(ActorState::Terminated);
    
    debug!("Component {} deregistered from registry", component_id.as_str());
    
    Ok(())
}
```

#### Validation Criteria
- [ ] Registry registration persists component metadata
- [ ] Registry cleanup removes component entry
- [ ] WASM cleanup verified
- [ ] Memory leaks prevented
- [ ] Test coverage ≥90%
- [ ] Integration tests with real registry

#### Estimated Effort
**8-10 hours** (registry integration + testing)

---

## Completion Tracking

### Phase 2 Task 2.1 - ActorSystem Integration
**Target Date:** After Task 1.3 completion  
**Dependencies:** Task 1.3 complete ✅

- [ ] **Item #1**: WASM Function Invocation (8-12h)
- [ ] **Item #2**: InterComponent WASM Call (4-6h)
- [ ] Integration tests for message invocation
- [ ] Performance benchmarks (>10,000 msg/sec)

**Total Estimated Effort:** 12-18 hours

### Phase 3 Task 3.3 - Component Health Monitoring
**Target Date:** Phase 3 Week 3-4  
**Dependencies:** Phase 2 complete, SupervisorNode integration

- [ ] **Item #4**: Health Check Export Parsing (4-6h)
- [ ] Health monitoring integration tests
- [ ] Supervisor restart on unhealthy

**Total Estimated Effort:** 4-6 hours

### Block 4 - Security & Isolation Layer
**Target Date:** Layer 2 (Months 5-6)  
**Dependencies:** Block 3 complete

- [ ] **Item #3**: Capability Enforcement (16-20h)
- [ ] Security validation tests
- [ ] Penetration testing
- [ ] Security audit

**Total Estimated Effort:** 16-20 hours

### Block 6 - Persistent Storage System
**Target Date:** Layer 2 (Months 7-8)  
**Dependencies:** Block 4, Block 5 complete

- [ ] **Item #5**: Component Registry Integration (8-10h)
- [ ] Registry persistence tests
- [ ] Restart recovery tests
- [ ] Memory leak tests

**Total Estimated Effort:** 8-10 hours

---

## Verification Process

### Pre-Completion Checklist (MANDATORY)

Before marking ANY future task as "complete", verify:

#### Phase 2 Task 2.1 Gate
- [ ] DEBT-WASM-004 Items #1 and #2 resolved
- [ ] All TODO comments at lines 181-195, 236-246 removed
- [ ] Integration tests cover WASM invocation
- [ ] Performance benchmarks meet targets
- [ ] Code review passed
- [ ] Documentation updated

#### Phase 3 Task 3.3 Gate
- [ ] DEBT-WASM-004 Item #4 resolved
- [ ] All TODO comments at lines 270-280 removed
- [ ] Health monitoring tests passing
- [ ] Supervisor integration verified

#### Block 4 Gate
- [ ] DEBT-WASM-004 Item #3 resolved
- [ ] All TODO comments at lines 223-228 removed
- [ ] Security tests passing (≥95% coverage)
- [ ] Security audit completed

#### Block 6 Gate
- [ ] DEBT-WASM-004 Item #5 resolved
- [ ] All TODO comments at lines 364-372, 400-407 removed
- [ ] Registry integration tests passing
- [ ] No memory leaks detected

### Automated Verification

Add to CI/CD pipeline:

```bash
# MUST PASS before merging Phase 2+
#!/bin/bash
set -e

echo "Checking for unresolved DEBT-WASM-004 items..."

# Check for TODO comments in actor_impl.rs
if grep -n "FUTURE WORK" airssys-wasm/src/actor/actor_impl.rs; then
    echo "ERROR: Unresolved deferred work found in actor_impl.rs"
    echo "See DEBT-WASM-004 for required implementation"
    exit 1
fi

echo "✅ All DEBT-WASM-004 items resolved"
```

---

## Consequences of Non-Completion

### If Item #1 (WASM Invocation) Not Implemented
- ❌ ComponentMessage::Invoke silently ignored
- ❌ No actual WASM function calls
- ❌ Inter-component communication broken
- ❌ Phase 2 cannot be considered complete

### If Item #2 (InterComponent Call) Not Implemented
- ❌ Component-to-component messages silently dropped
- ❌ No component collaboration possible
- ❌ MessageBroker integration incomplete

### If Item #3 (Capability Enforcement) Not Implemented
- 🔒 **SECURITY VULNERABILITY**: No access control
- 🔒 Components can send to any other component
- 🔒 DoS attacks possible (no rate limiting)
- 🔒 System cannot pass security audit

### If Item #4 (Health Check) Not Implemented
- 🏥 Supervisor cannot detect unhealthy components
- 🏥 No automatic restart on failures
- 🏥 System reliability degraded

### If Item #5 (Registry) Not Implemented
- 💾 Component lifecycle not persisted
- 💾 Memory leaks on component restart
- 💾 No component discovery
- 💾 Restart recovery broken

---

## Monitoring and Alerts

### Development Phase Alerts

**Phase 2 Task 2.1 Start:**
```
⚠️ REMINDER: Implement DEBT-WASM-004 Items #1 and #2
Location: src/actor/actor_impl.rs lines 181-195, 236-246
Estimated: 12-18 hours
```

**Phase 3 Task 3.3 Start:**
```
⚠️ REMINDER: Implement DEBT-WASM-004 Item #4
Location: src/actor/actor_impl.rs lines 270-280
Estimated: 4-6 hours
```

**Block 4 Start:**
```
🔒 SECURITY REMINDER: Implement DEBT-WASM-004 Item #3
Location: src/actor/actor_impl.rs lines 223-228
Estimated: 16-20 hours
CRITICAL: Security vulnerability until resolved
```

**Block 6 Start:**
```
⚠️ REMINDER: Implement DEBT-WASM-004 Item #5
Location: src/actor/actor_impl.rs lines 364-372, 400-407
Estimated: 8-10 hours
```

---

## Sign-Off Requirements

Each item MUST have sign-off before task completion:

### Item #1 - WASM Function Invocation
**Implementer:** ________________  
**Reviewer:** ________________  
**Date Completed:** ________________  
**Test Coverage:** _____% (must be ≥90%)

### Item #2 - InterComponent WASM Call
**Implementer:** ________________  
**Reviewer:** ________________  
**Date Completed:** ________________  
**Test Coverage:** _____% (must be ≥90%)

### Item #3 - Capability Enforcement
**Implementer:** ________________  
**Security Reviewer:** ________________  
**Date Completed:** ________________  
**Test Coverage:** _____% (must be ≥95%)  
**Security Audit:** ☐ PASSED ☐ FAILED

### Item #4 - Health Check Parsing
**Implementer:** ________________  
**Reviewer:** ________________  
**Date Completed:** ________________  
**Test Coverage:** _____% (must be ≥90%)

### Item #5 - Registry Integration
**Implementer:** ________________  
**Reviewer:** ________________  
**Date Completed:** ________________  
**Test Coverage:** _____% (must be ≥90%)  
**Memory Leak Test:** ☐ PASSED ☐ FAILED

---

## Document History

| Date | Change | Author |
|------|--------|--------|
| 2025-12-13 | Initial creation - Task 1.3 deferred work documented | AI Agent |

---

## Related Documents

- **WASM-TASK-004**: Block 3 Actor System Integration (parent task)
- **KNOWLEDGE-WASM-016**: Actor System Integration Implementation Guide
- **ADR-WASM-006**: Component Isolation and Sandboxing
- **ADR-WASM-001**: Inter-Component Communication Design
- **Task 1.3 Completion Summary**: (to be created)

---

**⚠️ FINAL WARNING ⚠️**

This document is a **BINDING CONTRACT** for future implementation. Failure to complete these items will result in:
- Incomplete actor system
- Security vulnerabilities
- System instability
- Failed integration

**ALL 5 ITEMS ARE MANDATORY AND MUST BE COMPLETED.**
