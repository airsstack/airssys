# ADR-WASM-009: Component Communication Model

**Status:** Accepted  
**Date:** 2025-10-19  
**Decision Makers:** Architecture Team  
**Related:** ADR-WASM-001 (Multicodec), ADR-WASM-005 (Security), ADR-WASM-006 (Isolation), KNOWLEDGE-WASM-005 (Messaging Implementation), RT-TASK-008 (MessageBroker Performance)

---

## Context

The airssys-wasm framework requires a robust inter-component communication system to enable WASM components written in different languages (Rust, JavaScript, Go, Python, etc.) to interact with each other. This communication must be:

- **Secure**: Enforce capability-based access control and prevent unauthorized interactions
- **Performant**: Low latency (<1ms) and high throughput (thousands of messages/sec)
- **Language-agnostic**: Support cross-language communication with automatic serialization
- **Actor-aligned**: Integrate with airssys-rt's lightweight Erlang-Actor model
- **Fault-tolerant**: Isolate component failures and provide supervision integration
- **Observable**: Provide audit logging and monitoring for security compliance

### The Challenge

**Traditional Approaches Have Limitations:**

**Shared Memory (Rejected):**
- ❌ Breaks isolation guarantees (ADR-WASM-006)
- ❌ Race conditions in multi-language scenarios
- ❌ No security boundaries between components
- ❌ Difficult to audit and monitor

**Synchronous RPC (Rejected):**
- ❌ Blocking calls don't align with actor model
- ❌ Deadlock risks in circular dependencies
- ❌ Poor fault isolation (caller blocks on failure)
- ❌ Doesn't leverage airssys-rt async infrastructure

**Custom Proxy Layer (Rejected - Initial Proposal):**
- ❌ Redundant with airssys-rt MessageBroker
- ❌ Additional performance overhead (~70ns extra)
- ❌ More complex architecture to maintain
- ❌ Doesn't leverage proven infrastructure

### Requirements

**Functional Requirements:**
- Fire-and-forget messaging (one-way notifications)
- Request-response pattern (async RPC with timeouts)
- Pub-sub broadcasting (event distribution)
- Manual correlation support (advanced use cases)
- Cross-language serialization (multicodec support)

**Security Requirements:**
- Capability-based permission checks (ADR-WASM-005)
- Per-component quota enforcement (API calls, network, storage)
- Audit logging of all inter-component messages
- No direct broker access from components (host-mediated only)

**Performance Requirements:**
- <300ns overhead per message (capability check + routing)
- >3M messages/sec throughput (support high-load scenarios)
- <1ms end-to-end latency (send → receive)
- Minimal memory overhead (<100KB per component for message buffers)

**Integration Requirements:**
- Leverage airssys-rt MessageBroker (proven ~211ns routing)
- Align with Actor/Child supervision model (ADR-WASM-006)
- Support WASM Component Model interfaces (WIT exports/imports)
- Platform portability (Linux, macOS, Windows via Tokio)

---

## Decision

### Core Decision: Message-Passing via airssys-rt MessageBroker with Host-Mediated Security

**We will use airssys-rt's InMemoryMessageBroker for inter-component communication, with security enforcement at the host function layer, supporting multiple messaging patterns through a unified actor-based architecture.**

### Key Design Principles

1. **Leverage Proven Infrastructure**: Use airssys-rt MessageBroker (211ns routing, 4.7M msg/sec proven)
2. **Host-Mediated Security**: All messages pass through host functions for capability validation
3. **Actor Model Alignment**: Components are actors, messages are delivered to mailboxes
4. **Push-Based Delivery**: No polling loops, immediate notification when messages arrive
5. **Language-Agnostic**: Multicodec self-describing serialization (ADR-WASM-001)
6. **Fail-Safe**: Component failures isolated via supervision trees (ADR-WASM-006)

---

## Detailed Decisions

### Decision 1: Messaging Patterns - Dual Mode Communication

**Decision:** Support two primary patterns with optional manual correlation for advanced use cases.

#### Pattern 1: Fire-and-Forget (One-Way Messaging)

**Use Cases:**
- Event notifications (user logged in, file uploaded)
- Status updates (progress reports, health checks)
- Pub-sub broadcasting (multiple subscribers)
- Logging and monitoring events
- Non-critical notifications

**Characteristics:**
- No response expected
- No correlation tracking
- Minimal overhead (~260ns total)
- Best for high-throughput scenarios

**WIT Interface:**
```wit
// Component imports (what components can call)
interface host-services {
    /// Send one-way message to another component
    send-message: func(
        target: component-id,
        message: list<u8>  // Multicodec-encoded payload
    ) -> result<_, messaging-error>;
}

// Component exports (what components must implement)
interface component-lifecycle {
    /// Receive pushed message from another component
    handle-message: func(
        sender: component-id,
        message: list<u8>
    ) -> result<_, messaging-error>;
}
```

**Message Flow:**
```text
1. Component A: send_message(target="component-b", data)
              ↓
2. Host Function: Validate capability + quota
              ↓
3. MessageBroker: Publish to subscribers (~211ns)
              ↓
4. ActorSystem: Route to ComponentActor B's mailbox
              ↓
5. Component B: handle_message(sender="component-a", data)
```

**Performance:**
- Host validation: ~50ns (capability lookup + atomic quota check)
- MessageBroker routing: ~211ns (proven RT-TASK-008)
- ActorSystem delivery: ~20ns (mailbox send)
- **Total overhead: ~280ns per message**

#### Pattern 2: Request-Response (Async RPC with Callbacks)

**Use Cases:**
- Database queries (request data, get response)
- API calls (invoke service, await result)
- Validation requests (check permission, receive decision)
- Resource acquisition (request lock, get confirmation)

**Characteristics:**
- Response expected
- Automatic correlation tracking by host
- Timeout enforcement by host runtime
- Callback delivered via `handle-callback` export

**WIT Interface:**
```wit
// Component imports
interface host-services {
    /// Send request and register callback
    send-request: func(
        target: component-id,
        request: list<u8>,
        timeout-ms: u32
    ) -> result<request-id, messaging-error>;
    
    /// Cancel pending request (optional)
    cancel-request: func(
        request-id: request-id
    ) -> result<_, messaging-error>;
}

// Component exports
interface component-lifecycle {
    /// Receive request (same as handle-message)
    handle-message: func(
        sender: component-id,
        message: list<u8>
    ) -> result<list<u8>, messaging-error>;  // Return value = response
    
    /// Receive response callback
    handle-callback: func(
        request-id: request-id,
        result: result<list<u8>, request-error>
    ) -> result<_, messaging-error>;
}
```

**Message Flow:**
```text
1. Component A: send_request(target="component-b", req, timeout=5s)
              ↓
2. Host: Generate request_id="abc-123", register callback with timeout
              ↓
3. Host: Return Ok("abc-123") to Component A
              ↓
4. MessageBroker: Route request to Component B
              ↓
5. Component B: handle_message(sender="component-a", req) → returns response
              ↓
6. Host: Capture return value, route to Component A's callback
              ↓
7. Component A: handle_callback("abc-123", Ok(response))
```

**Timeout Handling:**
```text
If Component B takes >5s:
  1. Host runtime triggers timeout
  2. Remove callback registration
  3. Component A: handle_callback("abc-123", Err(Timeout))
  4. Late response from B discarded (callback already removed)
```

**Performance:**
- Request path: ~280ns (same as fire-and-forget)
- Response path: ~280ns (callback delivery)
- Timeout tracking: ~10ns/request (timer registration)
- **Total round-trip: ~560ns + component processing time**

#### Pattern 3: Manual Correlation (Advanced)

**Use Cases:**
- Custom timeout strategies
- Batch request-response patterns
- Complex multi-step workflows
- Legacy protocol compatibility

**Approach:**
- Component generates correlation IDs manually
- Both request and response use `send_message`
- Component tracks pending requests in internal state
- Full control over correlation logic

**Example:**
```rust
// Sender manually tracks requests
let correlation_id = uuid::generate();
pending_requests.insert(correlation_id, callback_info);

send_message(target, encode(&Request {
    correlation_id,
    data: request_data
}))?;

// Later, in handle_message:
if let Some(corr_id) = extract_correlation_id(&message) {
    if let Some(callback) = pending_requests.remove(&corr_id) {
        callback.invoke(message);
    }
}
```

**Trade-offs:**
- ✅ Full control over correlation and timeouts
- ✅ Custom retry strategies
- ❌ More complex to implement
- ❌ Component responsible for cleanup

---

### Decision 2: MessageBroker Integration - Pure Pub-Sub Architecture

**Decision:** Use airssys-rt InMemoryMessageBroker with pure pub-sub pattern, ActorSystem as primary subscriber.

**Architecture:**

```text
┌─────────────────────────────────────────────────────────────┐
│                     Component Layer                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ Component A  │  │ Component B  │  │ Component C  │      │
│  │  (Actor)     │  │  (Actor)     │  │  (Actor)     │      │
│  └──────┬───────┘  └──────▲───────┘  └──────▲───────┘      │
│         │ send_message     │ handle_message  │              │
└─────────┼──────────────────┼─────────────────┼──────────────┘
          │                  │                 │
          ▼                  │                 │
┌─────────────────────────────────────────────────────────────┐
│                     Host Function Layer                      │
│  ┌────────────────────────────────────────────────────┐     │
│  │  send_message() / send_request()                   │     │
│  │  • Validate capability (ADR-WASM-005)              │     │
│  │  • Check quota (API calls, network, storage)       │     │
│  │  • Audit logging (sender, receiver, timestamp)     │     │
│  │  • Register callback (if request-response)         │     │
│  └────────────────────┬───────────────────────────────┘     │
└─────────────────────────┼───────────────────────────────────┘
                        │ publish
                        ▼
┌─────────────────────────────────────────────────────────────┐
│              airssys-rt MessageBroker (Pure Pub-Sub)        │
│  ┌────────────────────────────────────────────────────┐     │
│  │  InMemoryMessageBroker                             │     │
│  │  • Broadcast to all subscribers                    │     │
│  │  • ~211ns routing latency (RT-TASK-008)            │     │
│  │  • 4.7M messages/sec throughput                    │     │
│  └────────────────────┬───────────────────────────────┘     │
└─────────────────────────┼───────────────────────────────────┘
                        │ subscribe
                        ▼
┌─────────────────────────────────────────────────────────────┐
│                      ActorSystem                             │
│  ┌────────────────────────────────────────────────────┐     │
│  │  • Subscribes to MessageBroker                     │     │
│  │  • Resolves component_id → actor_address           │     │
│  │  • Routes to ComponentActor mailbox                │     │
│  │  • Delivers via handle_message() export            │     │
│  └──────────────────────────────────┬─────────────────┘     │
└─────────────────────────────────────┼───────────────────────┘
                                      │
                        ┌─────────────┼─────────────┐
                        │             │             │
                        ▼             ▼             ▼
                   Component A   Component B   Component C
                   (receives)    (receives)    (receives)
```

**Key Components:**

**1. InMemoryMessageBroker (airssys-rt):**
```rust
pub struct InMemoryMessageBroker<M: Message> {
    /// Pub-sub subscribers (ActorSystem, monitors, etc.)
    subscribers: RwLock<Vec<mpsc::UnboundedSender<MessageEnvelope<M>>>>,
    
    /// Pending request-reply channels (for request-response pattern)
    pending_requests: DashMap<Uuid, oneshot::Sender<Vec<u8>>>,
}

impl<M: Message> MessageBroker<M> for InMemoryMessageBroker<M> {
    async fn publish(&self, envelope: MessageEnvelope<M>) -> Result<()> {
        // Broadcast to all subscribers
        let subscribers = self.subscribers.read().await;
        for tx in subscribers.iter() {
            let _ = tx.send(envelope.clone());
        }
        Ok(())
    }
    
    async fn subscribe(&self) -> Result<MessageStream<M>> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.subscribers.write().await.push(tx);
        Ok(MessageStream::new(rx))
    }
}
```

**2. ActorSystem Subscriber:**
```rust
// ActorSystem subscribes to broker at initialization
let mut message_stream = broker.subscribe().await?;

tokio::spawn(async move {
    while let Some(envelope) = message_stream.recv().await {
        // Resolve component_id to actor address
        if let Some(actor_addr) = registry.resolve(&envelope.to).await {
            // Route to actor mailbox
            actor_addr.send(envelope.payload).await?;
        }
    }
});
```

**3. Host Function Integration:**
```rust
// Host function: send_message
pub fn send_message(
    ctx: &mut WasmContext,
    target: ComponentId,
    message: &[u8],
) -> Result<()> {
    // 1. Validate capability
    ctx.capabilities.check_permission("messaging:send", &target)?;
    
    // 2. Check quota
    ctx.quotas.check_api_call_quota()?;
    
    // 3. Audit log
    audit_log!(sender: ctx.component_id, receiver: target, size: message.len());
    
    // 4. Publish to broker
    let envelope = MessageEnvelope {
        from: ctx.component_id.clone(),
        to: target,
        payload: message.to_vec(),
        correlation_id: None,
        timestamp: Utc::now(),
    };
    
    ctx.broker.publish(envelope).await?;
    
    Ok(())
}
```

**Rationale:**

✅ **Reuses proven infrastructure**: MessageBroker tested with 4.7M msg/sec throughput
✅ **Simple architecture**: Pub-sub pattern well-understood and maintainable
✅ **Extensible**: Easy to add new subscribers (monitoring, logging, debugging)
✅ **Performance**: ~211ns routing is 3x faster than initial ~330ns proxy proposal
✅ **Fault tolerance**: Subscriber failures don't affect broker or other subscribers

---

### Decision 3: Host Function Security Layer - Capability-Based Enforcement

**Decision:** Enforce security at host function boundaries before message publication.

**Security Checks (Executed in Order):**

**1. Capability Validation:**
```rust
// Check: Does sender have "messaging:send" capability?
// Check: Is target in allowed_targets list?
ctx.capabilities.check_permission("messaging:send", &target)?;

// Example Component.toml declaration:
[capabilities]
messaging = ["component-b", "component-c"]  # Can only message these components
```

**2. Quota Enforcement:**
```rust
// Track API calls per component
ctx.quotas.api_calls.fetch_add(1, Ordering::Relaxed);
if ctx.quotas.api_calls.load(Ordering::Relaxed) > ctx.quotas.api_calls_limit {
    return Err(QuotaExceeded::ApiCalls);
}

// Default quota: 1000 API calls/minute per component
```

**3. Audit Logging:**
```rust
// Log every inter-component message for security audit
audit_log::info!(
    event: "component_message",
    sender: ctx.component_id,
    receiver: target,
    message_size: message.len(),
    timestamp: Utc::now(),
    request_id: envelope.correlation_id,
);
```

**4. Rate Limiting (Optional):**
```rust
// Advanced: Token bucket rate limiting
if !ctx.rate_limiter.try_acquire() {
    return Err(RateLimitExceeded);
}
```

**Performance Overhead:**

| Check Type | Latency | Implementation |
|------------|---------|----------------|
| Capability lookup | ~30 ns | HashMap lookup |
| Quota check | ~10 ns | Atomic counter load+add |
| Audit logging | ~5 ns | Async channel send (buffered) |
| Rate limiting | ~5 ns | Token bucket check (optional) |
| **Total** | **~50 ns** | Sequential execution |

**Security Guarantees:**

✅ **Principle of least privilege**: Components can only message declared targets
✅ **Resource accountability**: Per-component quotas prevent abuse
✅ **Audit trail**: All messages logged for security compliance
✅ **Fail-safe**: Validation failures prevent message publication
✅ **Defense in depth**: Security layer independent of MessageBroker

---

### Decision 4: Message Serialization - Multicodec Self-Describing Format

**Decision:** Use multicodec prefixes for language-agnostic serialization (ADR-WASM-001).

**Encoding Strategy:**

```text
Message Format:
┌──────────────┬─────────────────────────────┐
│ Multicodec   │ Serialized Payload          │
│ Prefix       │ (Format-Specific Bytes)     │
│ (varint)     │                             │
└──────────────┴─────────────────────────────┘
  1-2 bytes      Variable length

Examples:
• 0x701 → Borsh serialization (deterministic, cross-language)
• 0x51  → CBOR serialization (human-readable, compact)
• 0x0200 → JSON serialization (human-readable, universal)
```

**Language-Agnostic Communication:**

```rust
// Rust Sender (using borsh)
use borsh::{BorshSerialize, BorshDeserialize};

#[derive(BorshSerialize)]
struct UserEvent { user_id: String, action: String }

let event = UserEvent { user_id: "123".into(), action: "login".into() };
let encoded = encode_with_multicodec(0x701, &event.try_to_vec()?)?;
send_message(target, &encoded)?;
```

```javascript
// JavaScript Receiver (using borsh)
import { decode as borshDecode } from 'borsh';

export function handleMessage(sender, messageData) {
    // Read multicodec prefix
    const prefix = readVarint(messageData);  // 0x701
    
    if (prefix === 0x701) {  // Borsh
        const payload = messageData.slice(varintLength);
        const event = borshDecode(UserEventSchema, payload);
        console.log(`User ${event.userId} performed ${event.action}`);
    }
}
```

**Codec Selection (ADR-WASM-001):**

| Use Case | Codec | Multicodec | Rationale |
|----------|-------|------------|-----------|
| Inter-component | Borsh | 0x701 | Deterministic, fast, cross-language |
| External APIs | JSON | 0x0200 | Human-readable, universal |
| Binary protocols | CBOR | 0x51 | Compact, efficient |

**Rationale:**

✅ **Self-describing**: Receiver knows format from prefix
✅ **Cross-language**: Rust ↔ JavaScript ↔ Go ↔ Python
✅ **No negotiation**: Format embedded in message
✅ **Extensible**: Add new codecs without breaking compatibility

---

### Decision 5: Component Lifecycle Integration - Push-Based Actor Model

**Decision:** Components receive messages via pushed `handle-message` calls, no polling required.

**Component Exports (WIT):**

```wit
package airssys:component-lifecycle@1.0.0;

interface lifecycle {
    /// Called when component receives a message
    /// Push-based delivery (no polling needed)
    handle-message: func(
        sender: component-id,
        message: list<u8>
    ) -> result<list<u8>, messaging-error>;
    
    /// Called when request-response callback arrives
    handle-callback: func(
        request-id: request-id,
        result: result<list<u8>, request-error>
    ) -> result<_, messaging-error>;
}
```

**Component Imports (WIT):**

```wit
package airssys:host-services@1.0.0;

interface messaging {
    /// Send fire-and-forget message
    send-message: func(
        target: component-id,
        message: list<u8>
    ) -> result<_, messaging-error>;
    
    /// Send request with callback
    send-request: func(
        target: component-id,
        request: list<u8>,
        timeout-ms: u32
    ) -> result<request-id, messaging-error>;
}
```

**Message Delivery Flow:**

```text
┌─────────────────────────────────────────────────┐
│             ComponentActor Mailbox              │
│  (tokio unbounded channel, managed by runtime)  │
└──────────────────┬──────────────────────────────┘
                   │ Messages queued here
                   ▼
        ┌──────────────────────┐
        │  Actor Message Loop  │
        │  while let Some(msg) │
        │    = mailbox.recv()  │
        └──────────┬───────────┘
                   │ Pop message
                   ▼
        ┌──────────────────────┐
        │  Call WASM Export    │
        │  handle_message(     │
        │    sender, payload   │
        │  )                   │
        └──────────┬───────────┘
                   │ WASM executes
                   ▼
        ┌──────────────────────┐
        │  Component Logic     │
        │  • Decode message    │
        │  • Process data      │
        │  • Return response   │
        │    (if applicable)   │
        └──────────────────────┘
```

**No Polling Architecture:**

```rust
// ❌ OLD APPROACH - Polling (CPU waste)
loop {
    let messages = check_messages()?;  // Busy loop!
    for msg in messages {
        process(msg);
    }
    sleep(Duration::from_millis(10));  // Still wastes CPU
}

// ✅ NEW APPROACH - Push delivery (efficient)
#[export_name = "handle-message"]
pub extern "C" fn handle_message(
    sender_ptr: *const u8, sender_len: usize,
    message_ptr: *const u8, message_len: usize
) -> i32 {
    // Called ONLY when message arrives (no polling)
    let message = unsafe { std::slice::from_raw_parts(message_ptr, message_len) };
    process(message);
    0  // Success
}
```

**Benefits:**

✅ **Zero CPU waste**: No polling loops
✅ **Low latency**: Immediate delivery when message arrives
✅ **Backpressure**: Mailbox size limits prevent memory exhaustion
✅ **Actor model**: Natural alignment with supervision trees
✅ **Simple programming model**: Declarative exports, no event loops

---

## Consequences

### Positive Consequences

✅ **Excellent Performance**: ~280ns per message (50ns validation + 211ns routing + 20ns delivery)
✅ **High Throughput**: 4.7M messages/sec (MessageBroker proven capacity from RT-TASK-008)
✅ **Proven Infrastructure**: Reuses battle-tested MessageBroker, not custom code
✅ **Simple Architecture**: Clear separation (host functions → broker → actors)
✅ **Language-Agnostic**: Multicodec enables Rust ↔ JS ↔ Go ↔ Python communication
✅ **Security-First**: Capability checks + quotas + audit logging at host boundary
✅ **Fault Isolation**: Component failures don't affect MessageBroker or other components
✅ **Observable**: Audit trail for all inter-component messages
✅ **Actor Alignment**: Natural integration with supervision trees (ADR-WASM-006)
✅ **Platform Portable**: Works on Linux, macOS, Windows via Tokio + WASM

### Negative Consequences

❌ **Serialization Overhead**: Encoding/decoding adds latency (~100-500ns depending on codec)
❌ **Memory Copies**: Messages copied through multiple layers (component → host → broker → actor)
❌ **No Shared Memory**: Can't share large data structures efficiently (need to serialize)
❌ **Language Constraints**: Some languages (Go, Python) have less mature borsh support

### Neutral Consequences

➖ **Async-Only**: No synchronous messaging (intentional design for actor model)
➖ **Manual Correlation**: Advanced use cases require component-managed correlation IDs
➖ **Single Broker**: No distributed messaging (future enhancement if needed)

---

## Implementation Notes

### Phase 1: Fire-and-Forget Foundation (Week 1-2)

**Deliverables:**
- Host function: `send_message()` with capability validation
- MessageBroker subscription in ActorSystem
- Component export: `handle-message` invocation
- Basic multicodec encoding/decoding helpers
- Unit tests for happy path and error cases

**Acceptance Criteria:**
- Components can send one-way messages
- Messages delivered via push to `handle-message`
- Capability validation enforced (unauthorized messages rejected)
- Quota enforcement working (exceed limit → error)
- Audit logging captured all messages

### Phase 2: Request-Response Pattern (Week 3-4)

**Deliverables:**
- Host function: `send_request()` with callback registration
- Timeout enforcement (host runtime triggers timeouts)
- Component export: `handle-callback` invocation
- Correlation ID management (generate, track, cleanup)
- Response routing (capture return value → route to callback)

**Acceptance Criteria:**
- Components can send requests and receive responses
- Timeouts enforced (late responses discarded)
- Callback delivers Ok(response) or Err(timeout)
- Memory leaks prevented (timeouts cleanup pending requests)
- Integration tests for timeout scenarios

### Phase 3: Pub-Sub and Advanced Patterns (Week 5-6)

**Deliverables:**
- Topic-based pub-sub (components subscribe to topics)
- Broadcast messaging (one-to-many)
- Manual correlation examples (documentation + tests)
- Performance benchmarks (message throughput, latency)
- Security audit (capability bypass attempts, quota enforcement)

**Acceptance Criteria:**
- Components can pub-sub to topics
- Broadcast messages delivered to all subscribers
- Manual correlation pattern documented with examples
- Performance meets targets (>3M msg/sec, <300ns overhead)
- Security tests pass (no capability bypass, no quota bypass)

### Testing Strategy

**Unit Tests:**
- Host function validation (capability checks, quota enforcement)
- MessageBroker routing (publish → subscribers receive)
- Serialization (multicodec encode/decode round-trip)
- Timeout handling (request timeout → callback with error)

**Integration Tests:**
- End-to-end messaging (component A → host → broker → component B)
- Cross-language communication (Rust ↔ JavaScript)
- Fault tolerance (component crash → other components unaffected)
- Performance regression (benchmark against baseline)

**Security Tests:**
- Capability bypass attempts (unauthorized messaging)
- Quota exhaustion (exceed limits → rejected)
- Audit trail validation (all messages logged)
- Malicious payload handling (oversized messages, invalid multicodec)

---

## References

### ADRs
- **ADR-WASM-001**: Multicodec Compatibility Strategy (serialization format)
- **ADR-WASM-005**: Capability-Based Security Model (permission system)
- **ADR-WASM-006**: Component Isolation and Sandboxing (actor lifecycle, supervision)
- **ADR-WASM-002**: WASM Runtime Engine Selection (Wasmtime async support)

### Knowledge Documentation
- **KNOWLEDGE-WASM-005**: Inter-Component Messaging Architecture (implementation guide, code examples)
- **KNOWLEDGE-RT-013**: Actor System Performance Characteristics (625ns spawn, 211ns routing)

### airssys-rt Documentation
- **RT-TASK-008**: Message Broker Performance Baseline (211ns routing, 4.7M msg/sec)
- **airssys-rt/src/broker/in_memory.rs**: MessageBroker implementation
- **airssys-rt/src/broker/traits.rs**: MessageBroker trait definition

### External References
- [WebAssembly Component Model](https://github.com/WebAssembly/component-model) - WIT interface definitions
- [Multicodec Specification](https://github.com/multiformats/multicodec) - Self-describing format codes
- [Erlang/OTP gen_server](https://www.erlang.org/doc/man/gen_server.html) - Actor messaging patterns
- [Borsh Specification](https://borsh.io/) - Cross-language deterministic serialization

---

## Appendix: Code Examples

### Example 1: Fire-and-Forget Rust → JavaScript

**Sender (Rust):**
```rust
use airssys_wasm_bindings::host::messaging::send_message;
use borsh::BorshSerialize;

#[derive(BorshSerialize)]
struct LogEvent {
    level: String,
    message: String,
    timestamp: u64,
}

fn log_to_analytics(msg: &str) -> Result<()> {
    let event = LogEvent {
        level: "info".into(),
        message: msg.into(),
        timestamp: current_time_millis(),
    };
    
    let encoded = encode_borsh_with_multicodec(&event)?;
    send_message(&ComponentId::from("analytics"), &encoded)?;
    
    Ok(())
}
```

**Receiver (JavaScript):**
```javascript
import { decode as borshDecode } from 'borsh';

export function handleMessage(sender, messageData) {
    const prefix = readVarint(messageData);
    if (prefix === 0x701) {  // Borsh
        const event = borshDecode(LogEventSchema, messageData.slice(1));
        console.log(`[${event.level}] ${event.message} at ${event.timestamp}`);
        storeInDatabase(event);
    }
}
```

### Example 2: Request-Response with Timeout

**Requester (Rust):**
```rust
use airssys_wasm_bindings::host::messaging::send_request;

async fn query_user_data(user_id: &str) -> Result<UserData> {
    let request = UserQuery { user_id: user_id.into() };
    let encoded = encode_borsh_with_multicodec(&request)?;
    
    // Send request with 5 second timeout
    let request_id = send_request(
        &ComponentId::from("user-service"),
        &encoded,
        5000  // 5s timeout
    )?;
    
    // Request ID returned immediately, callback will arrive later
    Ok(request_id)
}

#[export_name = "handle-callback"]
pub extern "C" fn handle_callback(
    request_id_ptr: *const u8, request_id_len: usize,
    result_ptr: *const u8, result_len: usize
) -> i32 {
    let request_id = unsafe { RequestId::from_ptr(request_id_ptr, request_id_len) };
    let result_data = unsafe { std::slice::from_raw_parts(result_ptr, result_len) };
    
    match decode_borsh::<UserData>(result_data) {
        Ok(user_data) => {
            log_info(&format!("Received user data for: {}", user_data.name));
            0  // Success
        }
        Err(e) => {
            log_error(&format!("Callback error: {}", e));
            1  // Error
        }
    }
}
```

**Responder (JavaScript):**
```javascript
export function handleMessage(sender, messageData) {
    const query = borshDecode(UserQuerySchema, messageData);
    
    // Fetch user data from database
    const userData = database.getUser(query.userId);
    
    // Return response (automatically routed to requester's callback)
    return borshEncode(UserDataSchema, userData);
}
```

---

**End of ADR-WASM-009**
