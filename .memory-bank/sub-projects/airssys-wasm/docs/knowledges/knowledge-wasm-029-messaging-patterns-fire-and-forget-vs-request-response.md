# KNOWLEDGE-WASM-029: Messaging Patterns - Fire-and-Forget vs Request-Response

**Document ID:** KNOWLEDGE-WASM-029  
**Created:** 2025-12-22  
**Updated:** 2025-12-22  
**Category:** Architecture / Patterns / Messaging  
**Maturity:** Stable  
**Related:** KNOWLEDGE-WASM-005, KNOWLEDGE-WASM-024, ADR-WASM-009, WASM-TASK-006

## Overview

This document provides a comprehensive explanation of the two messaging patterns in airssys-wasm: **Fire-and-Forget** and **Request-Response**. It clarifies how each pattern works, the key differences in runtime behavior, and importantly, how the response mechanism works (return value from `handle-message`, NOT a separate host function).

## Context

### Problem Statement

During Block 5 Phase 3 Task 3.2 planning, there was confusion about how responses are sent back in the request-response pattern. Initial planning incorrectly assumed a `send-response` host function was needed.

### The Critical Insight

**There is NO `send-response` host function.** The response IS the return value from `handle-message`. The runtime decides what to do with this return value based on how the message was sent (fire-and-forget vs request-response).

### Scope

This knowledge applies to:
- Block 5 (Inter-Component Communication) implementation
- Task 3.2 (Response Routing and Callbacks)
- Any future work on messaging patterns

## Technical Content

### The Two Messaging Patterns

#### Pattern 1: Fire-and-Forget

**Sender calls:** `send-message(target, data)`

```
┌─────────────┐                 ┌──────────────┐                 ┌─────────────┐
│ Component A │                 │   Runtime    │                 │ Component B │
│  (Sender)   │                 │              │                 │ (Receiver)  │
└─────┬───────┘                 └──────┬───────┘                 └─────┬───────┘
      │                                │                               │
      │ send_message(B, data)          │                               │
      │───────────────────────────────>│                               │
      │                                │                               │
      │ Ok(())                         │ 1. Check capability           │
      │<───────────────────────────────│ 2. Check quota                │
      │ (returns immediately)          │ 3. Publish to broker          │
      │                                │ 4. NO callback registration   │
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

**Characteristics:**
- Returns `Ok(())` immediately - no request_id
- **No callback registration** in CorrelationTracker
- **No timeout enforcement**
- Component B's return value is **IGNORED** by runtime
- Fastest path: ~280ns
- Use case: Events, notifications, logging

#### Pattern 2: Request-Response

**Sender calls:** `send-request(target, data, timeout_ms)`

```
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
      │                                │ 4. Register in CorrelationTracker
      │                                │ 5. Start timeout timer        │
      │                                │ 6. Publish WITH correlation_id│
      │                                │                               │
      │ Ok("abc")                      │                               │
      │<───────────────────────────────│                               │
      │ (returns immediately)          │                               │
      │                                │                               │
      │                                │ handle_message(A, req)        │
      │                                │──────────────────────────────>│
      │                                │ (message has correlation_id)  │
      │                                │                               │
      │                                │                               │ Process
      │                                │                               │ request
      │                                │                               │
      │                                │ Ok(response)                  │
      │                                │<──────────────────────────────│
      │                                │ (return value = RESPONSE!)    │
      │                                │                               │
      │                                │ 7. Detect correlation_id      │
      │                                │ 8. Call CorrelationTracker::resolve()
      │                                │ 9. Route to requester         │
      │                                │                               │
      │ handle_callback("abc", Ok(response))                           │
      │<───────────────────────────────│                               │
      │ (async callback)               │                               │
```

**Characteristics:**
- Returns `Ok(request_id)` immediately
- **Callback registered** in CorrelationTracker
- **Timeout enforced** by runtime
- Component B's return value is **CAPTURED** and routed to callback
- Slower path: ~560ns (round-trip)
- Use case: RPC, queries, validation requests

### The Key Difference: How Runtime Knows

The **sender decides** which pattern by calling different functions:

| Sender Calls | Pattern | Message Has correlation_id? | Return Value Handling |
|--------------|---------|-----------------------------|-----------------------|
| `send-message(target, data)` | Fire-and-forget | ❌ NO | **IGNORED** |
| `send-request(target, data, timeout)` | Request-response | ✅ YES | **CAPTURED → routed** |

### Same Export, Different Behavior

**Component B implements the SAME `handle-message` export for BOTH patterns!**

```rust
// Component B - handles BOTH patterns with the SAME function!
#[export]
fn handle_message(sender: ComponentId, message: Vec<u8>) -> Result<Vec<u8>, Error> {
    // Component B doesn't know if this is fire-and-forget or request-response!
    // It just processes the message and returns a result.
    
    let data = decode(message)?;
    let result = process(data)?;
    
    Ok(encode(result)?)
    // ↑ If fire-and-forget: Runtime IGNORES this
    // ↑ If request-response: Runtime CAPTURES this and routes to callback
}
```

**Why this works:**
- Component B doesn't need to know the pattern
- Component B just processes and returns
- **Runtime** decides what to do with return value

### Runtime's Decision Logic (Task 3.2 Implementation)

```rust
// This is what Task 3.2 needs to implement
async fn deliver_message_to_component(
    component_b: &ComponentActor,
    sender: ComponentId,
    message: Vec<u8>,
    correlation_id: Option<CorrelationId>,  // KEY: This determines the pattern!
) -> Result<(), WasmError> {
    
    // Invoke handle-message on Component B
    let result = component_b.invoke_handle_message(sender, message).await?;
    
    // What to do with the return value?
    match correlation_id {
        None => {
            // FIRE-AND-FORGET: Ignore return value
            // (don't route anything back)
        }
        Some(corr_id) => {
            // REQUEST-RESPONSE: Capture and route!
            // 1. Look up pending request
            // 2. Resolve via CorrelationTracker
            // 3. Invoke handle-callback on requester
            route_response_to_requester(corr_id, result).await?;
        }
    }
    
    Ok(())
}
```

### Why NO `send-response` Host Function?

Common misconception: "Component B needs to call `send-response` to send a reply."

**Reality:** The response IS the return value from `handle-message`.

**Why this design?**
1. **Simpler for components**: Just return from `handle-message`
2. **Automatic correlation**: Runtime already knows the correlation_id
3. **Cleaner actor model**: Matches Erlang gen_server behavior
4. **No extra API surface**: One less host function to implement
5. **Type safety**: Return type enforces response structure

### WIT Interfaces

**Component Exports (what components implement):**
```wit
interface component-lifecycle {
    // Same export handles BOTH patterns
    handle-message: func(
        sender: component-id,
        message: list<u8>
    ) -> result<_, component-error>;
    
    // Only requester implements this (for request-response)
    handle-callback: func(
        request-id: request-id,
        callback-result: result<list<u8>, string>
    ) -> result<_, component-error>;
}
```

**Host Services (what components call):**
```wit
interface host-services {
    // Pattern 1: Fire-and-forget
    send-message: func(
        target: component-id,
        message: list<u8>
    ) -> result<_, messaging-error>;
    
    // Pattern 2: Request-response
    send-request: func(
        target: component-id,
        request: list<u8>,
        timeout-ms: u64
    ) -> result<request-id, messaging-error>;
    
    // NO send-response function! Response is return value.
}
```

## Summary Table

| Aspect | Fire-and-Forget | Request-Response |
|--------|-----------------|------------------|
| **Sender API** | `send-message()` | `send-request()` |
| **Returns** | `Ok(())` | `Ok(request_id)` |
| **Message has correlation_id** | ❌ No | ✅ Yes |
| **Callback Registration** | ❌ None | ✅ CorrelationTracker |
| **Timeout** | ❌ None | ✅ Enforced by runtime |
| **Receiver Export** | `handle-message` | `handle-message` (same!) |
| **Return Value** | **IGNORED** | **CAPTURED → routed** |
| **Requester Callback** | ❌ None | ✅ `handle-callback` invoked |
| **Latency** | ~280ns | ~560ns (round-trip) |
| **Use Case** | Events, notifications | RPC, queries |

## Task 3.2 Implications

Based on this knowledge, Task 3.2 (Response Routing and Callbacks) should:

**DO:**
1. Detect if message has correlation_id (from message metadata)
2. Capture return value from `handle-message`
3. Call `CorrelationTracker::resolve()` to route response
4. Invoke `handle-callback` on requester
5. Clean up callback registration

**DO NOT:**
1. ❌ Create a `send-response` host function
2. ❌ Add new WIT interfaces
3. ❌ Require components to call anything special

## References

### Related Documentation
- **KNOWLEDGE-WASM-005**: Inter-Component Messaging Architecture
- **KNOWLEDGE-WASM-024**: Component Messaging Clarifications
- **ADR-WASM-009**: Component Communication Model
- **WASM-TASK-006**: Block 5 - Inter-Component Communication

### Source Analysis
- Lines 254-292 of KNOWLEDGE-WASM-024 (unified receiver pattern)
- Lines 121-175 of KNOWLEDGE-WASM-005 (request-response flow)
- Lines 391-472 of KNOWLEDGE-WASM-024 (flow diagrams)

## History

### Version History
- **2025-12-22:** v1.0 - Initial document capturing messaging pattern differences and response mechanism clarification
