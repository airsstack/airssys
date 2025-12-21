# Component Messaging Clarifications - airssys-wasm

**Document Type:** Knowledge Documentation  
**Created:** 2025-12-21  
**Status:** Active Reference  
**Priority:** High - Foundation for Block 5 Implementation  
**Related:** ADR-WASM-009, KNOWLEDGE-WASM-005, WASM-TASK-006

---

## Overview

This document captures critical clarifications about the component messaging architecture that emerged during Block 5 Phase 1 planning. These clarifications address common misconceptions and ensure consistent understanding across the development team.

### Purpose

- **Clarify async-only communication model** (no synchronous RPC)
- **Document the two send methods** (`send-message` vs `send-request`)
- **Explain component perspective** (high-level API, no manual subscriptions)
- **Distinguish internal vs component-facing features** (runtime infrastructure vs component API)

---

## Critical Clarifications

### 1. No Synchronous Communication (Async-Only Architecture)

**Clarification:** The messaging system is **purely asynchronous**. There is NO synchronous/blocking communication.

#### What Was Rejected

From **ADR-WASM-009 (lines 31-35):**

```
Synchronous RPC (Rejected):
- ❌ Blocking calls don't align with actor model
- ❌ Deadlock risks in circular dependencies
- ❌ Poor fault isolation (caller blocks on failure)
- ❌ Doesn't leverage airssys-rt async infrastructure
```

#### Why This Matters

**Misconception:**
> "Components have sync and async communication"

**Reality:**
> "Components have TWO async patterns: fire-and-forget and request-response (both non-blocking)"

**Impact on Implementation:**
- No blocking wait for responses
- All communication via async callbacks
- No risk of deadlocks from circular dependencies
- Aligns with actor model (Erlang gen_server patterns)

---

### 2. Two Separate Send Methods (Not One Generic Send)

**Clarification:** There are **TWO distinct methods** for sending messages, not one generic method.

#### The Two Methods

**Method 1: Fire-and-Forget (`send-message`)**

```wit
interface host-services {
    /// Send one-way message (no response expected)
    send-message: func(
        target: component-id,
        message: list<u8>
    ) -> result<_, messaging-error>;
}
```

**Characteristics:**
- Returns immediately: `Ok(())` or `Err(error)`
- No response expected
- No callback registration
- Fastest: ~280ns latency
- Use case: Events, notifications, logging

**Method 2: Request-Response (`send-request`)**

```wit
interface host-services {
    /// Send request with callback (response expected)
    send-request: func(
        target: component-id,
        request: list<u8>,
        timeout-ms: u32
    ) -> result<request-id, messaging-error>;
}
```

**Characteristics:**
- Returns immediately: `Ok(request_id)` or `Err(error)`
- Response expected via `handle-callback` export
- Runtime tracks correlation automatically
- Runtime enforces timeout
- Slower: ~560ns round-trip
- Use case: RPC, queries, validation requests

#### Why Two Methods?

**Design Rationale:**

1. **Clarity of Intent:**
   - Explicit distinction between one-way and two-way communication
   - Self-documenting code (`send_message` vs `send_request`)

2. **Type Safety:**
   - Different return types force proper handling
   - Compiler ensures callbacks implemented for requests

3. **Performance:**
   - Fire-and-forget avoids callback registration overhead
   - Request-response tracks correlation + timeout

4. **Correctness:**
   - Can't accidentally expect response from fire-and-forget
   - Can't forget to handle response callback

---

### 3. Component Perspective: High-Level API Only

**Clarification:** Components see **only simple, high-level APIs**. All complexity handled by runtime.

#### What Components DO (Simple API)

```rust
// 1. SEND MESSAGES (two options)
send_message(target_id, data)?;              // Fire-and-forget
send_request(target_id, data, timeout_ms)?;  // Request-response

// 2. IMPLEMENT EXPORTS (runtime invokes these)
#[export]
fn handle_message(sender: ComponentId, data: Vec<u8>) -> Result<Vec<u8>, Error> {
    // Process message, optionally return response
}

#[export]
fn handle_callback(request_id: RequestId, result: Result<Vec<u8>, Error>) {
    // Handle response or timeout
}
```

#### What Components DON'T DO (Runtime Handles)

**Components NEVER:**
- ❌ Subscribe to MessageBroker topics manually
- ❌ Poll for messages (push-based delivery)
- ❌ Manage correlation IDs (runtime tracks)
- ❌ Handle timeouts (runtime enforces)
- ❌ Route messages (runtime decides)
- ❌ Access MessageBroker directly
- ❌ Manage actor mailboxes
- ❌ Deal with ActorSystem

**Runtime ALWAYS:**
- ✅ Validates capabilities (Block 4 security)
- ✅ Enforces quotas (Block 4 resource limits)
- ✅ Routes messages (MessageBroker → mailbox)
- ✅ Tracks requests (correlation IDs)
- ✅ Enforces timeouts (automatic cleanup)
- ✅ Delivers messages (push-based, no polling)
- ✅ Audit logs (all messaging events)

#### Why This Matters

**Misconception:**
> "Components need to subscribe to topics to receive messages"

**Reality:**
> "Runtime automatically subscribes components. Components just implement `handle-message` export."

**Impact:**
- Simpler component development
- Less error-prone (no manual subscription management)
- Consistent behavior (runtime enforces patterns)
- Better security (no direct broker access)

---

### 4. Internal Infrastructure vs Component API

**Clarification:** Distinguish between **runtime infrastructure** (internal) and **component API** (exposed).

#### Layer 1: Runtime Infrastructure (Internal)

**Purpose:** Enable MessageBroker routing and delivery

**Components:**
- MessageBroker initialization
- ActorSystem subscription to broker
- Component mailbox management
- Message routing logic
- Correlation tracking
- Timeout enforcement

**Visibility:** **INTERNAL ONLY** - Components never see this

**Implementation Location:**
- `airssys-wasm/src/runtime/messaging.rs`
- `airssys-wasm/src/actor/component_actor.rs`
- `airssys-wasm/src/runtime/topics.rs`

**Phase 1 Task 1.3 Focus:**
This layer! Internal subscription management, NOT component-facing API.

#### Layer 2: Component API (Exposed)

**Purpose:** Simple API for components to communicate

**API Surface:**
```wit
// Component imports (what components call)
interface host-services {
    send-message: func(...);
    send-request: func(...);
    cancel-request: func(...);  // Optional
}

// Component exports (what runtime invokes)
interface component-lifecycle {
    handle-message: func(...);
    handle-callback: func(...);
}
```

**Visibility:** **PUBLIC API** - Component developers use this

**Implementation Location:**
- `airssys-wasm/wit/messaging.wit` (WIT definitions)
- `airssys-wasm/src/runtime/async_host.rs` (host function implementations)

**Phase 2+ Focus:**
Implement host functions that use Layer 1 infrastructure.

#### Common Confusion

**Misconception:**
> "Task 1.3 (Topic Subscription System) means components subscribe to topics"

**Reality:**
> "Task 1.3 implements INTERNAL subscription infrastructure (MessageBroker → ActorSystem → ComponentActor). Components addressed by ComponentId, not topics."

**Optional Future Enhancement:**
> "Topic-based pub-sub (components subscribe to patterns) could be Phase 2+ feature, but NOT required for basic messaging."

---

### 5. Unified Receiver (One Export for Both Patterns)

**Clarification:** Both fire-and-forget AND request-response deliver to the **SAME** `handle-message` export.

#### How It Works

**Component B (Receiver):**
```rust
#[export]
fn handle_message(sender: ComponentId, message: Vec<u8>) -> Result<Vec<u8>, Error> {
    // This SAME function handles:
    // 1. Fire-and-forget messages (return value ignored)
    // 2. Request-response messages (return value = response)
    
    let data = decode(message)?;
    let result = process(data)?;
    
    Ok(encode(result)?)
    // ↑ If fire-and-forget: Ignored by runtime
    // ↑ If request-response: Becomes the response
}
```

#### Runtime's Decision Logic

```rust
// Runtime delivery (simplified)
match message_type {
    MessageType::FireAndForget => {
        component.handle_message(sender, data)?;
        // Ignore return value
    }
    MessageType::Request(request_id) => {
        let response = component.handle_message(sender, data)?;
        // Capture return value and route to requester's callback
        route_response(request_id, response).await?;
    }
}
```

#### Why Unified Receiver?

**Design Rationale:**

1. **Simplicity:**
   - One export to implement, not two
   - Component doesn't need to know message type

2. **Flexibility:**
   - Same processing logic for both patterns
   - Return value optional (fire-and-forget ignores it)

3. **Type Safety:**
   - Return type is `Result<Vec<u8>, Error>`
   - Forces error handling

4. **Actor Model Alignment:**
   - Matches Erlang gen_server behavior
   - Single message handler per actor

---

### 6. Multicodec Serialization (REQUIRED, Not Optional)

**Clarification:** Multicodec prefix validation is **REQUIRED** for all inter-component messages. This is NOT optional.

#### What the Host Runtime MUST Do

From **ADR-WASM-001** (Multicodec Compatibility Strategy):

```
Host Responsibilities:
1. ✅ Parse multicodec prefixes (detect codec from message bytes)
2. ✅ Maintain multicodec ID registry (known codec identifiers)
3. ✅ Validate component codec declarations (from Component.toml manifests)
4. ✅ Check compatibility at message send time (fail fast if incompatible)
5. ✅ Provide clear error messages (indicate supported codecs)
6. ✅ Route messages as opaque bytes (no decoding/encoding)
```

#### What the Host Runtime Does NOT Do

```
Host Does NOT:
1. ❌ Translate between codecs (no MessagePack → borsh conversion)
2. ❌ Implement codec serialization/deserialization
3. ❌ Depend on codec libraries (borsh, bincode, etc.)
```

#### Common Misconception

**Misconception:**
> "Multicodec is optional because the payload is already bytes from WASM"

**Reality:**
> "Multicodec prefix validation is REQUIRED. The host parses the prefix to validate codec compatibility between sender and receiver. No translation occurs, but validation does."

#### Why This Matters for Task 2.1

When implementing `send-message` host function:

1. **Parse** the multicodec prefix from the message bytes (first 2 bytes typically)
2. **Validate** that the target component supports this codec
3. **Fail fast** with clear error if codec incompatible
4. **Route** message as opaque bytes (no decode/encode)

```rust
// Example: send-message host function validation
pub async fn send_message(target: ComponentId, message: Vec<u8>) -> Result<(), MessagingError> {
    // 1. Parse multicodec prefix (REQUIRED)
    let (codec, _prefix_len) = Multicodec::from_prefix(&message)
        .map_err(|_| MessagingError::InvalidMessage)?;
    
    // 2. Validate receiver supports this codec (REQUIRED)
    let receiver_codecs = get_component_supported_codecs(&target)?;
    if !receiver_codecs.contains(&codec) {
        return Err(MessagingError::InvalidMessage); // Fail fast
    }
    
    // 3. Route as opaque bytes (no translation)
    broker.publish(target, message).await
}
```

#### Performance Impact

Multicodec validation adds minimal overhead:
- Prefix parsing: ~10ns (read 2 bytes, match enum)
- Compatibility check: ~50ns (HashSet lookup)
- Total: ~60ns additional latency

This fits within the ~280ns total target (211ns routing + 69ns overhead).

---

## Message Flow Diagrams

### Fire-and-Forget Flow (Detailed)

```text
┌─────────────┐                 ┌──────────────┐                 ┌─────────────┐
│ Component A │                 │   Runtime    │                 │ Component B │
│  (Sender)   │                 │              │                 │ (Receiver)  │
└─────┬───────┘                 └──────┬───────┘                 └─────┬───────┘
      │                                │                               │
      │ send_message(B, data)          │                               │
      │───────────────────────────────>│                               │
      │                                │                               │
      │                                │ 1. Check capability (Block 4) │
      │                                │ 2. Check quota (Block 4)      │
      │                                │ 3. Audit log (Block 4)        │
      │                                │ 4. Publish to broker          │
      │                                │                               │
      │ Ok(())                         │                               │
      │<───────────────────────────────│                               │
      │ (returns immediately)          │                               │
      │                                │                               │
      │                                │ 5. Broker routes (~211ns)     │
      │                                │ 6. ActorSystem resolves       │
      │                                │ 7. Deliver to mailbox         │
      │                                │                               │
      │                                │ handle_message(A, data)       │
      │                                │──────────────────────────────>│
      │                                │                               │
      │                                │                               │ Process
      │                                │                               │ message
      │                                │                               │
      │                                │ Ok(response)                  │
      │                                │<──────────────────────────────│
      │                                │ (return value IGNORED)        │
      │                                │                               │
```

**Total Latency:** ~280ns (50ns validation + 211ns routing + 20ns delivery)

### Request-Response Flow (Detailed)

```text
┌─────────────┐                 ┌──────────────┐                 ┌─────────────┐
│ Component A │                 │   Runtime    │                 │ Component B │
│ (Requester) │                 │              │                 │ (Responder) │
└─────┬───────┘                 └──────┬───────┘                 └─────┬───────┘
      │                                │                               │
      │ send_request(B, req, 5000ms)   │                               │
      │───────────────────────────────>│                               │
      │                                │                               │
      │                                │ 1. Check capability           │
      │                                │ 2. Check quota                │
      │                                │ 3. Generate request_id="abc"  │
      │                                │ 4. Register callback + timeout│
      │                                │ 5. Publish to broker          │
      │                                │                               │
      │ Ok("abc")                      │                               │
      │<───────────────────────────────│                               │
      │ (returns immediately)          │                               │
      │                                │                               │
      │                                │ 6. Broker routes              │
      │                                │ 7. Deliver to mailbox         │
      │                                │                               │
      │                                │ handle_message(A, req)        │
      │                                │──────────────────────────────>│
      │                                │                               │
      │                                │                               │ Process
      │                                │                               │ request
      │                                │                               │
      │                                │ Ok(response)                  │
      │                                │<──────────────────────────────│
      │                                │ (return value = RESPONSE)     │
      │                                │                               │
      │                                │ 8. Lookup callback for "abc"  │
      │                                │ 9. Route to requester mailbox │
      │                                │                               │
      │ handle_callback("abc", Ok(response))                           │
      │<───────────────────────────────│                               │
      │ (async callback)               │                               │
      │                                │                               │
```

**Total Round-Trip:** ~560ns (280ns request + 280ns response)

**Timeout Scenario:**
```text
If Component B takes >5000ms:
  1. Runtime timeout timer fires
  2. Remove callback registration for "abc"
  3. Deliver timeout error: handle_callback("abc", Err(Timeout))
  4. Late response from B discarded (callback already removed)
```

---

## Implementation Guidelines for Block 5

### Phase 1: MessageBroker Integration Foundation

**Focus:** Internal infrastructure ONLY (Layer 1)

**Task 1.1: MessageBroker Setup**
- Initialize MessageBroker in WasmRuntime
- ActorSystem subscribes to broker
- **NOT exposed to components**

**Task 1.2: Message Reception**
- Mailbox integration
- Push-based delivery to `handle-message` export
- **NOT exposed to components** (automatic)

**Task 1.3: Subscription Management** ⚠️ CLARIFIED
- **Internal subscription infrastructure**
- MessageBroker → ActorSystem → ComponentActor routing
- Components addressed by ComponentId (direct addressing)
- **NOT topic-based pub-sub exposed to components**
- **Optional future:** Topic patterns could be Phase 2+ enhancement

### Phase 2: Fire-and-Forget Messaging

**Focus:** Implement component-facing API (Layer 2)

**Task 2.1: send-message Host Function**
- WIT interface: `send-message(target, message)`
- Capability validation (Block 4)
- Quota enforcement (Block 4)
- MessageBroker publish

**Task 2.2: handle-message Component Export**
- Push-based delivery implementation
- WASM export invocation
- Error handling

**Task 2.3: Performance Validation**
- Verify ~280ns target
- Throughput benchmarks

### Phase 3: Request-Response Pattern

**Focus:** Add async RPC with callbacks

**Task 3.1: send-request Host Function**
- WIT interface: `send-request(target, request, timeout-ms)`
- Generate request_id
- Register callback with timeout
- MessageBroker publish

**Task 3.2: Response Routing**
- Capture return value from `handle-message`
- Correlate response to request_id
- Route to `handle-callback` export

**Task 3.3: Timeout Enforcement**
- tokio::time::timeout integration
- Timeout error delivery to callback
- Cleanup (prevent memory leaks)

---

## Common Misconceptions (Corrected)

### Misconception 1: "Components subscribe to topics manually"

**Correction:**
- Components do NOT subscribe manually
- Runtime automatically routes messages to components by ComponentId
- `handle-message` export is invoked automatically (push delivery)
- Topic-based pub-sub is optional future enhancement, NOT core requirement

### Misconception 2: "There's sync and async communication"

**Correction:**
- Everything is async (no synchronous/blocking communication)
- Two async patterns: fire-and-forget and request-response
- Both are non-blocking (immediate return)
- Request-response uses callbacks (async RPC, not blocking RPC)

### Misconception 3: "One generic send method for everything"

**Correction:**
- Two separate methods: `send-message` and `send-request`
- Different return types: `Result<(), Error>` vs `Result<RequestId, Error>`
- Different semantics: one-way vs two-way
- Compiler enforces correct usage

### Misconception 4: "Components need to poll for messages"

**Correction:**
- Push-based delivery (no polling)
- Runtime invokes `handle-message` when message arrives
- Zero CPU waste (no event loops)
- Immediate delivery (low latency)

### Misconception 5: "Different receivers for different message types"

**Correction:**
- Single `handle-message` export handles both patterns
- Return value usage depends on message type
- Fire-and-forget: Return value ignored
- Request-response: Return value becomes response

---

## Architecture Alignment

### With ADR-WASM-009

✅ **Pure async actor model** (lines 74-85)  
✅ **Two messaging patterns** (lines 91-228)  
✅ **Host-mediated security** (lines 415-481)  
✅ **Push-based delivery** (lines 551-664)  
✅ **MessageBroker integration** (lines 270-412)

### With KNOWLEDGE-WASM-005

✅ **Actor-based message passing** (lines 15-29)  
✅ **Dual interaction patterns** (lines 21-23)  
✅ **Push-based event model** (lines 32-64)  
✅ **Component exports vs imports** (lines 66-84)

### With Block 4 (Security)

✅ **Capability validation before send** (ADR-WASM-005)  
✅ **Quota enforcement per component** (Task 4.3)  
✅ **Audit logging integration** (Task 3.3)  
✅ **No direct broker access** (security boundary at host functions)

---

## Frequently Asked Questions

### Q1: Can components communicate synchronously?

**A:** No. All communication is asynchronous. Request-response provides RPC semantics but via async callbacks, not blocking calls.

### Q2: Do components subscribe to topics?

**A:** Not in Phase 1. Components are addressed directly by ComponentId. Topic-based pub-sub is an optional future enhancement.

### Q3: How do components receive messages?

**A:** Push-based delivery. Runtime automatically invokes the `handle-message` export when a message arrives. No polling required.

### Q4: Are there different receivers for fire-and-forget vs request-response?

**A:** No. Both patterns deliver to the same `handle-message` export. The runtime decides what to do with the return value.

### Q5: Can components bypass the MessageBroker?

**A:** No. All messaging goes through host functions → security layer → MessageBroker → runtime delivery. No direct broker access.

### Q6: What's the difference between `send-message` and `send-request`?

**A:** 
- `send-message`: Fire-and-forget, no response expected, returns `Ok(())`
- `send-request`: Request-response, response via callback, returns `Ok(request_id)`

### Q7: How are request-response callbacks correlated?

**A:** The runtime automatically generates request_ids and tracks correlation. Components just receive `handle-callback(request_id, result)`.

### Q8: What happens if a component doesn't respond in time?

**A:** The runtime enforces timeouts. If no response within the timeout period, the requester's `handle-callback` is invoked with `Err(Timeout)`.

---

## References

### ADRs
- **ADR-WASM-009**: Component Communication Model (primary architecture)
- **ADR-WASM-005**: Capability-Based Security Model (security validation)
- **ADR-WASM-006**: Component Isolation and Sandboxing (actor lifecycle)
- **ADR-WASM-001**: Multicodec Compatibility Strategy (serialization)

### Knowledge Documentation
- **KNOWLEDGE-WASM-005**: Inter-Component Messaging Architecture (implementation details)
- **KNOWLEDGE-WASM-020**: airssys-osl Security Integration (capability checks)

### Task Documentation
- **WASM-TASK-006**: Block 5 - Inter-Component Communication (master task)
- **WASM-TASK-005**: Block 4 - Security & Isolation Layer (security foundation)
- **WASM-TASK-004**: Block 3 - Actor System Integration (MessageBroker baseline)

---

## Version History

| Date | Version | Changes |
|------|---------|---------|
| 2025-12-21 | 1.1 | Added Section 6 (Multicodec Serialization), fixed stale host_functions.rs reference to async_host.rs |
| 2025-12-21 | 1.0 | Initial document capturing Block 5 Phase 1 planning clarifications |

---

**End of KNOWLEDGE-WASM-024**
