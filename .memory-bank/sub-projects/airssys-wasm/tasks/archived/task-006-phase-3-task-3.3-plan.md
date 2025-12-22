# Action Plan for WASM-TASK-006 Phase 3 Task 3.3: Timeout and Cancellation

**Document ID:** WASM-TASK-006-Phase-3-Task-3.3-Plan  
**Created:** 2025-12-22  
**Status:** READY FOR IMPLEMENTATION  
**Priority:** 🔴 CRITICAL (includes integration gap fix)  
**Estimated Effort:** 16-20 hours

---

## ⚠️ CRITICAL CONTEXT

**A MAJOR BUG EXISTS**: Task 3.2 was marked complete but the end-to-end request-response flow is **BROKEN**.

### The Problem (src/runtime/async_host.rs lines 688-692)

```rust
// NOTE: _response_rx is intentionally unused in Task 3.1. Response delivery
// to WASM components via handle-response callback is implemented in Task 3.2.
// The CorrelationTracker stores response_tx; Task 3.2 will resolve it when
// the target component sends a response.
let (response_tx, _response_rx) = oneshot::channel::<ResponseMessage>();
```

**The issue:**
1. `SendRequestHostFunction` creates `(response_tx, _response_rx)` but **DROPS** `_response_rx` immediately
2. `ResponseRouter.route_response()` sends responses to `response_tx` via `CorrelationTracker::resolve()`
3. `WasmEngine::call_handle_callback()` exists to invoke WASM callback on the requester
4. **NOBODY SPAWNS A TASK TO AWAIT `response_rx` AND CALL `call_handle_callback()`!**

The request-response pattern is fundamentally broken - responses are sent to a channel that nobody is listening to.

---

## Goal

This task will:
1. **Part A: Fix the Critical Integration Gap** - Spawn a background task to receive responses and deliver them to WASM components via `call_handle_callback()`
2. **Part B: Implement Timeout and Cancellation** - Add timeout error delivery, cancel-request API, and edge case handling

---

## Context & References

### Architectural Documents
- **ADR-WASM-009**: Component Communication Model (Pattern 2: Request-Response)
- **ADR-WASM-020**: Message Delivery Ownership Architecture
- **KNOWLEDGE-WASM-029**: Messaging Patterns - Fire-and-Forget vs Request-Response

### Existing Implementation (Task 3.1 & 3.2)
- `SendRequestHostFunction` at `src/runtime/async_host.rs:599-731`
- `ResponseRouter` at `src/runtime/messaging.rs:512-666`
- `WasmEngine::call_handle_callback()` at `src/runtime/engine.rs:593-673`
- `TimeoutHandler` at `src/actor/message/timeout_handler.rs`
- `CorrelationTracker` at `src/actor/message/correlation_tracker.rs`

### Test Fixtures Available
- `callback-receiver-component.wat/.wasm` - For callback testing (630 bytes)
- `basic-handle-message.wat/.wasm` - Basic handler
- `slow-handler.wat/.wasm` - For timeout testing

---

## Implementation Steps

### Part A: Fix Integration Gap (CRITICAL - Hours 1-8)

#### Step A.1: Add Engine to SendRequestHostFunction (1-2 hours)

**File:** `src/runtime/async_host.rs`

**Changes:**
1. Add `engine: Arc<WasmEngine>` field to `SendRequestHostFunction`
2. Update `new()` constructor to accept engine
3. Update `AsyncHostRegistryBuilder::with_messaging_functions()` to pass engine

**Code Pattern:**
```rust
pub struct SendRequestHostFunction {
    messaging_service: Arc<MessagingService>,
    engine: Arc<WasmEngine>,  // NEW: For callback invocation
}

impl SendRequestHostFunction {
    pub fn new(
        messaging_service: Arc<MessagingService>,
        engine: Arc<WasmEngine>,  // NEW parameter
    ) -> Self {
        Self { 
            messaging_service,
            engine,
        }
    }
}
```

**Success Criteria:**
- [x] Compiles without errors
- [x] Engine is accessible in `execute()` method

---

#### Step A.2: Spawn Response Receiver Task (4-5 hours)

**File:** `src/runtime/async_host.rs`

**Changes:**
In `SendRequestHostFunction::execute()`, after registering the pending request:

1. **DO NOT** drop `response_rx` - save it
2. Clone necessary Arc references for the spawned task
3. Spawn a background task that:
   - Awaits `response_rx.await`
   - When response arrives, extract result
   - Look up requester's `ComponentHandle` (from registry)
   - Call `call_handle_callback()` on the requester

**Code Pattern:**
```rust
// In execute(), REPLACE the current pattern:
// OLD:
// let (response_tx, _response_rx) = oneshot::channel::<ResponseMessage>();

// NEW:
let (response_tx, response_rx) = oneshot::channel::<ResponseMessage>();

// ... register pending request as before ...

// NEW: Spawn response receiver task
let engine = Arc::clone(&self.engine);
let correlation_id_str = correlation_id.to_string();
let requester_id = context.component_id.clone();

tokio::spawn(async move {
    match response_rx.await {
        Ok(response) => {
            // Determine if error or success
            let (is_error, payload) = match response.result {
                Ok(data) => (0_i32, data),
                Err(e) => (1_i32, format!("{}", e).into_bytes()),
            };
            
            // Get component handle for requester
            // Note: Handle lookup via engine's component registry
            if let Some(handle) = engine.get_component_handle(&requester_id).await {
                // Invoke handle-callback on requester
                let _ = engine.call_handle_callback(
                    &handle,
                    &correlation_id_str,
                    &payload,
                    is_error,
                ).await;
            }
        }
        Err(_) => {
            // Channel closed - sender dropped without sending
            // This happens on cancellation - no action needed
        }
    }
});
```

**Success Criteria:**
- [x] Response receiver task spawned for each request
- [x] Task handles success responses
- [x] Task handles error responses
- [x] Task handles channel closure (cancellation)

---

#### Step A.3: Add Component Handle Lookup to WasmEngine (2-3 hours)

**File:** `src/runtime/engine.rs`

**Changes:**
Add method to retrieve `ComponentHandle` by `ComponentId`. This is needed because the spawned response receiver task needs access to the requester's handle to call the callback.

**Option A: Handle Registry in WasmEngine**
```rust
impl WasmEngine {
    /// Get a component handle by ID.
    ///
    /// Returns the component handle if the component is loaded and active.
    pub async fn get_component_handle(&self, id: &ComponentId) -> Option<ComponentHandle> {
        // Implementation depends on how handles are stored
        // May need to add a handle_registry: Arc<DashMap<ComponentId, ComponentHandle>>
    }
}
```

**Option B: Pass Handle via Context**
Alternatively, store the requester's handle in the spawned task closure directly if available from `HostCallContext`.

**Success Criteria:**
- [x] Handle lookup method exists
- [x] Returns correct handle for valid component
- [x] Returns None for unknown component

---

#### Step A.4: Write Part A Integration Tests (2 hours)

**File:** `tests/timeout_cancellation_integration_tests.rs` (NEW)

**Tests to prove Part A works:**

```rust
#[tokio::test]
async fn test_send_request_response_invokes_handle_callback() {
    // CRITICAL TEST: Proves the full request-response path works
    // 1. Load requester component (has handle-callback export)
    // 2. Load responder component (has handle-message export)
    // 3. Requester calls send-request to responder
    // 4. Responder's handle-message returns response
    // 5. VERIFY: Requester's handle-callback is invoked with response
}

#[tokio::test]
async fn test_request_response_with_success_payload() {
    // Verify success response is delivered correctly
}

#[tokio::test]
async fn test_request_response_with_error_payload() {
    // Verify error response is delivered with is_error=1
}
```

**Success Criteria:**
- [x] Test proves end-to-end flow works
- [x] Test uses real WASM fixtures
- [x] All tests pass

---

### Part B: Timeout and Cancellation (Hours 9-16)

#### Step B.1: Verify TimeoutHandler Integration (1-2 hours)

**File:** `src/actor/message/timeout_handler.rs`

The `TimeoutHandler` already sends `Err(RequestError::Timeout)` to the response channel when timeout fires. With Part A fix, this will automatically trigger `call_handle_callback()` with error.

**Verification Steps:**
1. Review `TimeoutHandler::register_timeout()` - confirms it sends to `pending.response_tx`
2. Review `cleanup_expired()` in CorrelationTracker - confirms timeout_count is incremented
3. Write test to prove timeout triggers callback

**Test:**
```rust
#[tokio::test]
async fn test_send_request_timeout_invokes_handle_callback_with_error() {
    // 1. Load requester with handle-callback
    // 2. Send request to non-existent component OR slow handler
    // 3. Wait for timeout (use short timeout like 100ms)
    // 4. VERIFY: handle-callback invoked with is_error=1 and "Timeout" message
}
```

---

#### Step B.2: Add RequestError::Cancelled Variant (1 hour)

**File:** `src/actor/message/request_response.rs`

**Changes:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum RequestError {
    Timeout,
    ComponentNotFound(ComponentId),
    ProcessingFailed(String),
    InvalidPayload(String),
    Cancelled,  // NEW: Request was cancelled by caller
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // ... existing cases ...
            RequestError::Cancelled => write!(f, "Request cancelled"),
        }
    }
}
```

**Success Criteria:**
- [x] New variant added
- [x] Display implemented
- [x] Serialization works

---

#### Step B.3: Implement CancelRequestHostFunction (4-5 hours)

**File:** `src/runtime/async_host.rs`

**New struct:**
```rust
/// Host function: cancel-request
///
/// Cancels a pending request by correlation ID. If the request is still
/// pending, delivers a Cancelled error to the requester's handle-callback.
///
/// # WIT Interface
///
/// ```wit
/// cancel-request: func(request-id: string) -> result<_, cancel-error>;
/// ```
///
/// # Arguments
///
/// * `request_id` - Correlation ID string (UUID v4 from send-request)
///
/// # Returns
///
/// * `Ok(())` - Request cancelled successfully
/// * `Err(CancelError::NotFound)` - Request already completed or invalid ID
/// * `Err(CancelError::NotOwner)` - Caller is not the request owner
///
/// # References
///
/// - **WASM-TASK-006 Phase 3 Task 3.3**: Timeout and Cancellation
pub struct CancelRequestHostFunction {
    messaging_service: Arc<MessagingService>,
}

impl CancelRequestHostFunction {
    pub fn new(messaging_service: Arc<MessagingService>) -> Self {
        Self { messaging_service }
    }
}

#[async_trait]
impl HostFunction for CancelRequestHostFunction {
    fn name(&self) -> &str {
        "messaging::cancel_request"
    }

    fn required_capability(&self) -> Capability {
        Capability::Messaging(TopicPattern::new("*"))
    }

    async fn execute(&self, context: &HostCallContext, args: Vec<u8>) -> WasmResult<Vec<u8>> {
        // 1. Parse request_id from args
        let request_id = String::from_utf8(args)
            .map_err(|e| WasmError::messaging_error(format!("Invalid request ID: {e}")))?;
        
        let correlation_id = Uuid::parse_str(&request_id)
            .map_err(|e| WasmError::messaging_error(format!("Invalid UUID: {e}")))?;
        
        // 2. Attempt to cancel via CorrelationTracker
        match self.messaging_service
            .correlation_tracker()
            .cancel(&correlation_id, &context.component_id)
            .await
        {
            Ok(()) => {
                self.messaging_service.record_request_cancelled();
                Ok(vec![])  // Empty success
            }
            Err(e) => Err(e),
        }
    }
}
```

**Success Criteria:**
- [x] Struct implemented with proper documentation
- [x] Registered in AsyncHostRegistryBuilder
- [x] Cancel delivers Cancelled error to callback

---

#### Step B.4: Add cancel() Method to CorrelationTracker (2-3 hours)

**File:** `src/actor/message/correlation_tracker.rs`

**Changes:**
```rust
impl CorrelationTracker {
    /// Cancel a pending request.
    ///
    /// Removes the request from the pending map and sends a Cancelled error
    /// to the response channel, triggering the callback with error status.
    ///
    /// # Arguments
    ///
    /// * `correlation_id` - Correlation ID of the request to cancel
    /// * `caller_id` - Component ID of the caller (must match request owner)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Request cancelled successfully
    /// * `Err(WasmError)` - Request not found or not owned by caller
    ///
    /// # Security
    ///
    /// Only the component that made the request can cancel it.
    pub async fn cancel(
        &self,
        correlation_id: &CorrelationId,
        caller_id: &ComponentId,
    ) -> Result<(), WasmError> {
        // 1. Remove from pending map
        let pending = self
            .pending
            .remove(correlation_id)
            .ok_or_else(|| {
                WasmError::messaging_error(format!(
                    "Request not found: {}",
                    correlation_id
                ))
            })?
            .1;
        
        // 2. Verify ownership (security check)
        if &pending.from != caller_id {
            // Re-insert the request since caller doesn't own it
            self.pending.insert(*correlation_id, pending);
            return Err(WasmError::capability_denied(
                Capability::Messaging(TopicPattern::new("*")),
                format!("Cannot cancel request owned by another component"),
            ));
        }
        
        // 3. Cancel timeout handler
        self.timeout_handler.cancel_timeout(correlation_id);
        
        // 4. Send Cancelled error to response channel
        let _ = pending.response_tx.send(ResponseMessage {
            correlation_id: *correlation_id,
            from: pending.to.clone(),
            to: pending.from.clone(),
            result: Err(RequestError::Cancelled),
            timestamp: Utc::now(),
        });
        
        // 5. Increment cancelled count
        self.cancelled_count.fetch_add(1, Ordering::Relaxed);
        
        Ok(())
    }
    
    /// Get the number of cancelled requests.
    pub fn cancelled_count(&self) -> u64 {
        self.cancelled_count.load(Ordering::Relaxed)
    }
}
```

**Also add the field:**
```rust
pub struct CorrelationTracker {
    pending: Arc<DashMap<CorrelationId, PendingRequest>>,
    timeout_handler: Arc<TimeoutHandler>,
    completed_count: Arc<AtomicU64>,
    timeout_count: Arc<AtomicU64>,
    cancelled_count: Arc<AtomicU64>,  // NEW
}
```

**Success Criteria:**
- [x] cancel() method implemented
- [x] Ownership verification works
- [x] Cancelled error sent to channel
- [x] Metrics updated

---

#### Step B.5: Add Cancellation Metrics to MessagingService (1 hour)

**File:** `src/runtime/messaging.rs`

**Changes:**
```rust
// In MessagingStats:
pub struct MessagingStats {
    // ... existing fields ...
    pub requests_cancelled: u64,  // NEW
}

// In MessagingService:
impl MessagingService {
    pub fn record_request_cancelled(&self) {
        // Increment counter
    }
}
```

---

#### Step B.6: Write Part B Integration Tests (3-4 hours)

**File:** `tests/timeout_cancellation_integration_tests.rs`

**Tests:**
```rust
#[tokio::test]
async fn test_cancel_request_invokes_handle_callback_with_cancelled() {
    // 1. Load requester with handle-callback
    // 2. Load slow handler that takes 5 seconds
    // 3. Send request with 10 second timeout
    // 4. Immediately call cancel-request
    // 5. VERIFY: handle-callback invoked with is_error=1 and "Cancelled" message
}

#[tokio::test]
async fn test_response_after_timeout_ignored() {
    // 1. Send request with 50ms timeout
    // 2. Responder takes 200ms to respond
    // 3. VERIFY: Timeout callback received
    // 4. VERIFY: Late response is ignored (orphaned count incremented)
}

#[tokio::test]
async fn test_cancel_after_response_fails() {
    // 1. Send request with long timeout
    // 2. Response arrives immediately
    // 3. Try to cancel
    // 4. VERIFY: Cancel returns NotFound error
}

#[tokio::test]
async fn test_cancel_by_non_owner_fails() {
    // 1. Component A sends request
    // 2. Component B tries to cancel it
    // 3. VERIFY: Cancel returns NotOwner error
}

#[tokio::test]
async fn test_double_cancel_fails() {
    // 1. Send request
    // 2. Cancel once (succeeds)
    // 3. Cancel again
    // 4. VERIFY: Second cancel returns NotFound
}

#[tokio::test]
async fn test_timeout_metrics_tracking() {
    // VERIFY: timeout_count incremented on timeout
}

#[tokio::test]
async fn test_cancellation_metrics_tracking() {
    // VERIFY: cancelled_count incremented on cancel
}
```

---

## Unit Testing Plan

**MANDATORY**: Tests in module `#[cfg(test)]` blocks

### src/runtime/async_host.rs
- [ ] `test_send_request_spawns_response_receiver` - Verify task is spawned
- [ ] `test_cancel_request_validates_uuid` - Invalid UUID rejected
- [ ] `test_cancel_request_requires_messaging_capability` - Capability check
- [ ] `test_send_request_with_engine_integration` - Engine field works

### src/actor/message/correlation_tracker.rs
- [ ] `test_cancel_success` - Basic cancel works
- [ ] `test_cancel_not_found` - Request already completed
- [ ] `test_cancel_ownership_check` - Non-owner rejected
- [ ] `test_cancel_sends_cancelled_error` - Response channel receives error
- [ ] `test_cancelled_count_initial` - Starts at 0
- [ ] `test_cancelled_count_after_cancel` - Incremented correctly

### src/actor/message/request_response.rs
- [ ] `test_request_error_cancelled_display` - Display format
- [ ] `test_request_error_cancelled_serialization` - JSON round-trip

### src/runtime/messaging.rs
- [ ] `test_requests_cancelled_metric` - Metric tracking

**Verification**: `cargo test --lib` - all tests passing

---

## Integration Testing Plan

**MANDATORY**: Tests in `tests/` directory

**Test file:** `tests/timeout_cancellation_integration_tests.rs`

### Critical End-to-End Tests (MUST PASS)

| Test | What It Proves | Fixture Used | Status |
|------|----------------|--------------|--------|
| `test_send_request_response_invokes_handle_callback` | **PROVES PART A WORKS** - Full success path | `callback-receiver-component.wasm` + `echo-handler.wasm` | Ready |
| `test_send_request_timeout_invokes_handle_callback_with_error` | Timeout path works | `callback-receiver-component.wasm` + `slow-handler.wasm` | Ready |
| `test_cancel_request_invokes_handle_callback_with_cancelled` | Cancel path works | `callback-receiver-component.wasm` + `slow-handler.wasm` | Ready |
| `test_response_after_timeout_ignored` | Late response handling | Same as above | Ready |
| `test_cancel_after_response_fails` | Already completed request | Same as above | Ready |
| `test_cancel_by_non_owner_fails` | Security: ownership check | Two requesters | Ready |
| `test_double_cancel_fails` | Idempotency | `callback-receiver-component.wasm` | Ready |
| `test_timeout_metrics_tracking` | Timeout metric incremented | `slow-handler.wasm` | Ready |
| `test_cancellation_metrics_tracking` | Cancel metric incremented | Same as above | Ready |

**Verification**: `cargo test --test timeout_cancellation_integration_tests` - all tests passing

---

## Fixture Verification

### Fixtures Needed

| Fixture | Purpose | Status |
|---------|---------|--------|
| `callback-receiver-component.wasm` | Requester with handle-callback export | ✅ EXISTS (630 bytes) |
| `echo-handler.wasm` | Responder that returns response immediately | ✅ EXISTS (177 bytes) |
| `slow-handler.wasm` | Responder that delays (for timeout testing) | ✅ EXISTS (223 bytes) |
| `basic-handle-message.wasm` | Simple handler for basic tests | ✅ EXISTS (162 bytes) |

**STATUS: READY** - All required fixtures exist in `tests/fixtures/`

---

## Quality Verification

- [ ] `cargo build` - builds cleanly
- [ ] `cargo test --lib` - all unit tests pass
- [ ] `cargo test --test timeout_cancellation_integration_tests` - all integration tests pass
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` - zero warnings
- [ ] Zero compiler warnings

---

## Verification Steps

1. **After Part A (Step A.4):**
   ```bash
   cargo test --lib -- send_request
   cargo test --test timeout_cancellation_integration_tests -- test_send_request_response_invokes_handle_callback
   ```
   - Expected: Critical end-to-end test passes (proves integration gap is fixed)

2. **After Part B (Step B.6):**
   ```bash
   cargo test --lib -- cancel
   cargo test --test timeout_cancellation_integration_tests
   ```
   - Expected: All 9+ integration tests pass

3. **Final Verification:**
   ```bash
   cargo build
   cargo test --lib
   cargo test --test '*'
   cargo clippy --all-targets --all-features -- -D warnings
   ```
   - Expected: No warnings, all tests pass

---

## Success Criteria

### Part A Success (Integration Gap Fixed)
- [ ] Response receiver task spawned in `SendRequestHostFunction::execute()`
- [ ] Task awaits `response_rx` and handles response
- [ ] Task calls `call_handle_callback()` on requester component
- [ ] End-to-end test proves full flow works: request → response → callback

### Part B Success (Timeout and Cancellation)
- [ ] Timeout already works (via TimeoutHandler) - just verify with test
- [ ] `RequestError::Cancelled` variant added
- [ ] `CancelRequestHostFunction` implemented
- [ ] `CorrelationTracker::cancel()` method implemented
- [ ] Ownership check prevents unauthorized cancellation
- [ ] Metrics track cancelled count
- [ ] All edge cases tested

---

## Files to Modify

| File | Changes |
|------|---------|
| `src/runtime/async_host.rs` | Add engine field, spawn response receiver, add `CancelRequestHostFunction` |
| `src/runtime/engine.rs` | Add `get_component_handle()` method (if needed) |
| `src/actor/message/request_response.rs` | Add `RequestError::Cancelled` variant |
| `src/actor/message/correlation_tracker.rs` | Add `cancel()` method, `cancelled_count` field |
| `src/runtime/messaging.rs` | Add `requests_cancelled` metric |
| `src/runtime/mod.rs` | Export `CancelRequestHostFunction` |
| `tests/timeout_cancellation_integration_tests.rs` | **NEW**: End-to-end integration tests |

---

## Estimated Timeline

| Phase | Hours | Description |
|-------|-------|-------------|
| Part A (Steps A.1-A.4) | 8-10 | Integration gap fix + tests |
| Part B (Steps B.1-B.6) | 8-10 | Timeout/cancellation + tests |
| **Total** | **16-20** | Complete Task 3.3 |

---

## Definition of Done

- [ ] All Part A steps complete (integration gap fixed)
- [ ] All Part B steps complete (timeout/cancellation)
- [ ] All unit tests pass (15+ new tests)
- [ ] All integration tests pass (9+ new tests)
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings
- [ ] Code reviewed and approved
- [ ] Progress tracking updated in task-006 main file
- [ ] _index.md updated

---

## Approval Requested

**Do you approve this plan? (Yes/No)**

If approved, implementation can begin immediately with Part A (Critical Integration Gap Fix).
