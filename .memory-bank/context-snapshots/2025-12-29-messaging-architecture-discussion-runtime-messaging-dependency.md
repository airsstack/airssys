# Architecture Discussion: runtime/ vs. messaging/ Correlation

**Date:** 2025-12-29

---

## Problem Identified

**Current Situation:**
- WASM-TASK-006 Phase 2 & 3 implemented messaging functions in `runtime/` module
- These need to be moved to `messaging/` submodules to complete refactoring
- Current task file (WASM-TASK-HOTFIX-001) has Phase 3 tasks that describe this migration

**Key Issue:** Messaging implementations in `runtime/` create dependencies:
  - messaging/ needs to access runtime components (WasmEngine, ComponentActor, etc.)
  - This creates potential ADR-WASM-023 violations (messaging → runtime/)

---

## Implementation Options Discussed

### Option A: Import runtime/ from messaging/ (CURRENT PLAN - INVERTED)

**Description:** messaging/ imports types from runtime/

**Example:**
```rust
// messaging/fire_and_forget.rs
use crate::runtime::{WasmEngine, ComponentActor, ...};

pub struct SendMessageHostFunction {
    engine: Arc<WasmEngine>,  // ❌ messaging depends on runtime
}
```

**Problems:**
- ❌ **Reverse dependency**: messaging/ depends on runtime/ (violates ADR-WASM-018 one-way dependency)
- ❌ **Module boundary violation**: messaging/ (Block 5) depends on runtime/ (Block 1)
- ❌ **Wrong direction**: Should be: core → runtime → actor → messaging (one-way flow)

**Why This is Wrong:**
- ADR-WASM-018: "Dependencies flow ONE WAY (top to bottom)"
  - core/ (foundation)
  - runtime/ (WASM execution)
  - actor/ (system integration)
  - messaging/ (communication)
  - Higher layers CANNOT import from lower layers

---

### Option B: Create Trait Abstraction

**Description:** messaging/ defines trait, runtime/ implements it

**Example:**
```rust
// messaging/fire_and_forget.rs
pub trait MessagePublisher {
    fn publish(&self, from: ComponentId, to: ComponentId, payload: Vec<u8>) -> Result<(), WasmError>;
}

// runtime/async_host.rs
impl MessagePublisher for WasmEngine {
    fn publish(&self, ...) { ... }
}
```

**Pros:**
- ✅ messaging/ doesn't import from runtime/
- ✅ Clean interface separation

**Cons:**
- ⏳ Complex trait design
- ⏳ New abstraction layer
- ⏳ More code changes

---

### Option C: Pass Runtime as Parameter (DISCUSSED & AGREED)

**Description:** messaging/ functions accept runtime components as parameters instead of storing them

**Example for fire-and-forget:**
```rust
// messaging/fire_and_forget.rs (CORRECT APPROACH)
pub struct SendMessageHostFunction {
    broker: Arc<InMemoryMessageBroker<ComponentMessage>>,
    // ✅ NO WasmEngine field
}

impl SendMessageHostFunction {
    pub fn send(
        &self,
        from: ComponentId,
        to: ComponentId,
        payload: Vec<u8>,
        codec: parameter: u32,
    ) -> Result<(), WasmError> {
        // Use MessageBrokerBridge (doesn't need engine)
        MessageBrokerBridge::publish(&self.broker, from, to, payload)
    }
}
```

**Example for request-response (with invoke_callback):**
```rust
// messaging/request_response.rs (CORRECT APPROACH)
pub struct SendRequest {
    broker: Arc<InMemoryMessageBroker<ComponentMessage>>,
    correlation_tracker: Arc<CorrelationTracker>,
    response_router: Arc<ResponseRouter>,
    // ✅ NO WasmEngine field
}

impl SendRequest {
    /// Invoke handle-callback export on component
    /// Engine is PASSED AS PARAMETER to avoid dependency violation
    pub async fn invoke_callback(
        &self,
        engine: &WasmEngine,  // ← Pass engine as parameter
        component_id: ComponentId,
        correlation_id: CorrelationId,
        result: Result<Vec<u8>, RequestError>,
    ) -> Result<(), WasmError> {
        // Move logic from engine::call_handle_callback here
        // Implementation from engine (~80 lines)
        Ok(())
    }
}
```

**Pros:**
- ✅ **ADR-WASM-023 compliant**: messaging/ doesn't import from runtime/
- ✅ **Simple and clear**: No complex traits or abstractions
- ✅ **Parameter passing pattern**: runtime passes engine when calling messaging
- ✅ **Works for all cases**: send_message, send_request, invoke_callback
- ✅ **No dependency issues**: messaging/ ↔ runtime is via parameters, not imports

**Cons:**
- ⏳ runtime/ must pass engine to messaging functions
- ⏳ Changes to calling code in runtime/

---

## Task Status Analysis

### Task 3.1: Verify All Imports Updated
**Status:** ✅ COMPLETE
**Evidence:**
- `src/runtime/messaging.rs` does not exist (deleted in Phase 1)
- No imports of `runtime::messaging` found anywhere
- `runtime/mod.rs` doesn't export messaging types

---

### Task 3.2: Delete runtime/messaging.rs
**Status:** ✅ COMPLETE
**Evidence:**
- File already deleted (no backup needed)
- `runtime/mod.rs` clean

---

### Task 3.3: Move SendMessageHostFunction to messaging/fire_and_forget.rs
**Status:** ⏳ NOT STARTED
**Evidence:**
- `SendMessageHostFunction` still exists in `runtime/async_host.rs` (lines 450-600)
- Still imported via `use airssys_wasm::runtime::{SendMessageHostFunction, MessagingService}`
- Placeholder still in `messaging/fire_and_forget.rs` (line 69-71)

**Action Needed:**
1. Review if SendMessageHostFunction actually needs WasmEngine
2. Move implementation to `messaging/fire_and_forget.rs`
3. Update `runtime/async_host.rs` to use messaging version
4. Move unit tests
5. Verify ADR-WASM-023 compliance

---

### Task 3.4: Move SendRequestHostFunction to messaging/request_response.rs
**Status:** ⏳ NOT STARTED
**Evidence:**
- `SendRequestHostFunction` still exists in `runtime/async_host.rs` (lines 602-826)
- Placeholder still in `messaging/request_response.rs` (line 86-88)

**Action Needed:**
1. Rename to `SendRequest` and move implementation
2. Update `runtime/async_host.rs` to use messaging version
3. Move unit tests
4. Verify ADR-WASM-023 compliance

---

### Task 3.5: Move call_handle_callback to messaging/request_response.rs
**Status:** ⏳ NOT STARTED
**Evidence:**
- `call_handle_callback()` still exists in `runtime/engine.rs` (lines 593-710)
- Not yet in `messaging/request_response.rs`

**Action Needed:**
1. Rename to `invoke_callback` and move to `request_response.rs`
2. Pass `WasmEngine` as parameter (agreed approach)
3. Update `runtime/engine.rs` to use messaging version
4. Move unit tests
5. Verify ADR-WASM-023 compliance

---

### Task 3.6: Move Multicodec Validation to messaging/codec.rs
**Status:** ⏳ NOT STARTED
**Evidence:**
- Placeholder still in `messaging/codec.rs` (line 73-75)
- Multicodec validation logic inlined in `SendMessageHostFunction` and `SendRequestHostFunction`

**Action Needed:**
1. Extract multicodec validation to `messaging/codec.rs`
2. Update both `fire_and_forget.rs` and `request_response.rs` to use `MulticodecCodec`
3. Remove inlined validation from `async_host.rs`
4. Move unit tests
5. Verify ADR-WASM-023 compliance

---

## Key Findings

### 1. Current Task File Status is INCOMPLETE

**Problem:** Task file was updated with expanded Phase 3 tasks (3.3-3.6), but tasks 3.3-3.6 have NOT been executed yet.

**Why This Matters:**
- Tasks 3.1 and 3.2 marked as "ALREADY COMPLETE"
- But tasks 3.3-3.6 show as "NOT STARTED"
- Verification shows code still in runtime/ (not yet moved)

**Task File Mismatch:**
```markdown
### Phase 3: Move WASM-TASK-006 Implementations to messaging/ Submodules (Days 3-4)

#### Task 3.1: Verify All Imports Updated
**Status:** ✅ COMPLETE  (correct)

#### Task 3.2: Delete runtime/messaging.rs
**Status:** ✅ COMPLETE  (correct)

#### Task 3.3: Move SendMessageHostFunction to messaging/fire_and_forget.rs
**Status:** ⏳ NOT STARTED  ← INCORRECT
```

---

### 2. Code Still in runtime/

**Evidence:**
```bash
# Code still exists:
grep -n "pub struct SendRequestHostFunction" src/runtime/async_host.rs
# Returns: line 602 (exists!)

grep -n "pub struct SendMessageHostFunction" src/runtime/async_host.rs
# Returns: line 450 (exists!)

grep -n "fn call_handle_callback" src/runtime/engine.rs
# Returns: line 593 (exists!)

# Still referenced:
grep -rn "use.*runtime::SendRequestHostFunction\|use.*runtime::SendMessageHostFunction" src/ tests/ examples/
# Returns: Multiple matches (still used!)
```

---

### 3. Agreed Approach: Pass Runtime as Parameter

**Decision Date:** 2025-12-29

**Agreement:**
- ✅ messaging/ will NOT import from runtime/
- ✅ messaging/ will accept runtime components as parameters
- ✅ ADR-WASM-023 one-way dependency maintained
- ✅ Engine passed as parameter to `invoke_callback()`
- ✅ No complex trait abstractions

---

## Recommendations

### 1. Update Task File Progress

**Action Needed:** Mark tasks 3.3-3.6 with correct status

**Change:**
```markdown
#### Task 3.3: Move SendMessageHostFunction to messaging/fire_and_forget.rs
**Status:** ⏳ NOT STARTED

To:
#### Task 3.3: Move SendMessageHostFunction to messaging/fire_and_forget.rs
**Status:** ⏳ NOT STARTED
**Estimated Work:** In Progress
```

---

### 2. Complete Tasks 3.3-3.6 First

**Reason:** Before proceeding with 3.5 (invoke_callback) and 3.6 (Multicodec), need to complete 3.3 and 3.4.

**Execution Order:**
1. Task 3.3: Move `SendMessageHostFunction` → fire_and_forget.rs
2. Task 3.4: Move `SendRequestHostFunction` → request_response.rs
3. Verify both tasks complete
4. Then proceed with 3.5, 3.6

---

### 3. Context Snapshot Creation

**Action:** Save this discussion as context snapshot

**File:** `.memory-bank/context-snapshots/2025-12-29-messaging-architecture-discussion-runtime-messaging-dependency.md`

**Purpose:**
- Document architecture discussion
- Clarify agreed approach
- Record current task status
- Prevent confusion in future

---

## Conclusion

**Current State:**
- ✅ Tasks 3.1 and 3.2: Complete (verification only)
- ❌ Tasks 3.3-3.6: Not Started (code not moved yet)
- ⚠️ Task file status: Incorrect (shows 3.3-3.6 as NOT STARTED but this is misleading)

**Critical Finding:**
Tasks 3.3-3.6 CANNOT be "NOT STARTED" because:
- Code still exists in runtime/
- Placeholders still exist in messaging/
- Actual implementation work needed (not just planning)

**Correct Status Should Be:**
- Tasks 3.3-3.6: "NOT STARTED" is accurate, but needs clarification
- Better to show: "AWAITING IMPLEMENTATION" to be more precise

**Next Steps:**
1. Save this context snapshot
2. Ask user to confirm understanding
3. Wait for user approval before updating task file
4. Once approved, proceed with Task 3.3 execution
