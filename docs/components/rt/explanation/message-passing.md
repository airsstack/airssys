# Understanding Message Passing

This document explains message passing in AirsSys RT, why it's fundamental to the actor model, design decisions around message routing, and performance considerations.

## Table of Contents

- [Why Message Passing?](#why-message-passing)
- [Message Passing Semantics](#message-passing-semantics)
- [Message Routing Architecture](#message-routing-architecture)
- [Design Decisions](#design-decisions)
- [Performance Characteristics](#performance-characteristics)
- [Comparison with Alternatives](#comparison-with-alternatives)

---

## Why Message Passing?

### The Shared-Memory Problem

Traditional concurrent programming relies on **shared mutable state** protected by locks:

```rust
// Shared-memory approach
use std::sync::{Arc, Mutex};

struct SharedCounter {
    value: Arc<Mutex<i32>>,
}

impl SharedCounter {
    fn increment(&self) {
        let mut count = self.value.lock().unwrap();  // Acquire lock
        *count += 1;  // Critical section - only one thread at a time
        // Lock released when count drops
    }
}

// Problems:
// 1. Deadlocks: Complex lock ordering requirements
// 2. Race conditions: Forgot to lock? Data corruption!
// 3. Scalability: Lock contention limits parallelism
// 4. Complexity: Difficult to reason about all interleavings
```

**Fundamental Issue:** Shared mutable state + concurrency = complexity and bugs.

### The Message Passing Solution

**Actors eliminate shared mutable state** by making message passing the sole communication mechanism:

```rust
// Message passing approach
struct Counter {
    value: i32,  // Private state, no locks needed!
}

impl Actor for Counter {}

#[async_trait]
impl Handler<Increment> for Counter {
    async fn handle(&mut self, _msg: Increment, _ctx: &mut ActorContext<Self>) {
        self.value += 1;  // Safe! Sequential message processing
        // No locks, no race conditions, no deadlocks
    }
}

// Send message asynchronously
counter_ref.send(Increment).await?;
```

**Benefits:**

- **No locks:** Actors process messages sequentially
- **No race conditions:** Each actor owns its state exclusively
- **No deadlocks:** Asynchronous message passing never blocks
- **Scalable:** Independent actors run in parallel
- **Simple reasoning:** Each actor is an isolated state machine

**Tradeoff:** Message passing adds latency (~737ns) vs. shared memory (~nanoseconds), but eliminates concurrency complexity.

---

## Message Passing Semantics

### Fire-and-Forget (Asynchronous Send)

**Semantics:** Send message and continue immediately without waiting for response.

```rust
// Sender continues without waiting
actor_ref.send(DoWork { task_id: 1 }).await?;
println!("Message sent!");  // Executes before message is processed

// Message delivered to mailbox
// Actor processes when ready
```

**Use Cases:**

- Logging and auditing (don't wait for log to be written)
- Event notifications (fire events without blocking)
- Background processing (queue work without waiting)

**Performance:** Lowest latency (~181ns mailbox enqueue + ~400ns processing = ~600ns total)

**Guarantees:**

- ✅ **At-most-once delivery:** Message delivered to mailbox or error returned
- ✅ **Ordered within sender:** Messages from same sender arrive in order
- ❌ **No delivery confirmation:** Sender doesn't know if message was processed
- ❌ **No response:** Sender cannot receive result

### Request-Reply (Synchronous Send)

**Semantics:** Send message and wait for response before continuing.

```rust
// Sender waits for response
let result: String = actor_ref.send(Query { id: 42 }).await?;
println!("Got response: {}", result);  // Waits for actor to respond
```

**Use Cases:**

- Queries requiring responses (database lookups, calculations)
- RPC-style interactions (client-server communication)
- Synchronous workflows (need result before proceeding)

**Performance:** Higher latency (~737ns roundtrip: send + receive + processing)

**Guarantees:**

- ✅ **Response guaranteed:** Receive typed response or timeout error
- ✅ **Type-safe:** Response type matches `Message::Result`
- ✅ **Timeout support:** Prevent indefinite waiting
- ❌ **Blocks sender:** Sender waits for response (can't do other work)

### Broadcast (Pub-Sub)

**Semantics:** Send message to all subscribers of a topic.

```rust
// Publish event to all subscribers
broker.publish("user.login", UserLoginEvent { user_id: 123 }).await?;

// Multiple subscribers receive the event
subscriber_a.handle(UserLoginEvent { user_id: 123 });  // Analytics
subscriber_b.handle(UserLoginEvent { user_id: 123 });  // Audit log
subscriber_c.handle(UserLoginEvent { user_id: 123 });  // Notification service
```

**Use Cases:**

- Event-driven architectures (domain events, notifications)
- Multi-subscriber patterns (multiple services react to same event)
- Decoupling (publishers don't know subscribers)

**Performance:** Scales with subscriber count (~395ns per subscriber)

**Guarantees:**

- ✅ **All subscribers notified:** Every active subscriber receives message
- ✅ **Parallel delivery:** Subscribers process independently
- ❌ **No delivery confirmation:** Publisher doesn't know who received
- ❌ **No ordering between publishers:** Messages from different publishers may interleave

---

## Message Routing Architecture

AirsSys RT provides two message routing mechanisms:

### Direct Actor References

**Mechanism:** Send messages directly to actor via `ActorRef<A>`.

```rust
// Spawn actor, get direct reference
let actor_ref: ActorRef<MyActor> = system.spawn(MyActor::new()).await?;

// Send message directly (no routing overhead)
actor_ref.send(MyMessage).await?;
```

**Performance:** Fastest path (~737ns roundtrip, no routing overhead)

**Use Cases:**

- Parent-child communication (supervisor → children)
- Request-reply patterns (client → server)
- Known recipient (reference available at compile time)

**Tradeoffs:**

- ✅ **Fastest:** No routing overhead
- ✅ **Type-safe:** Compiler ensures actor handles message type
- ❌ **Tight coupling:** Sender must have reference to specific actor
- ❌ **No discovery:** Cannot find actors dynamically

### Message Broker (Pub-Sub)

**Mechanism:** Send messages through `MessageBroker` which routes to subscribers.

```rust
// Register actor with broker for topic
broker.subscribe("events.user", user_analytics_ref).await?;
broker.subscribe("events.user", user_audit_ref).await?;

// Publish to topic (broker routes to all subscribers)
broker.publish("events.user", UserEvent { user_id: 123 }).await?;
// Both user_analytics_ref and user_audit_ref receive the event
```

**Performance:** Adds routing overhead (~180ns + ~395ns per subscriber)

**Use Cases:**

- Pub-sub patterns (one-to-many messaging)
- Dynamic discovery (find actors by topic at runtime)
- Decoupling (senders don't need specific actor references)

**Tradeoffs:**

- ✅ **Decoupling:** Publisher doesn't know subscribers
- ✅ **Dynamic:** Subscribe/unsubscribe at runtime
- ✅ **One-to-many:** Single publish reaches multiple subscribers
- ❌ **Routing overhead:** ~180ns per message
- ❌ **Topic management:** Must agree on topic naming scheme

### Choosing Between Direct and Broker

**Use Direct References When:**

- Communication is point-to-point (one sender, one receiver)
- Actors are tightly coupled (parent-child, client-server)
- Performance critical (hot path, high-frequency messaging)
- Type safety important (compiler enforces handler exists)

**Use Message Broker When:**

- Communication is one-to-many (pub-sub, events)
- Actors are loosely coupled (decoupled services)
- Dynamic discovery needed (find actors at runtime)
- Flexibility more important than performance

**Hybrid Approach:**
```rust
// Use direct references for hot path
let worker_ref = system.spawn(Worker::new()).await?;
worker_ref.send(HighFrequencyRequest).await?;  // Fast path

// Use broker for events
broker.publish("worker.completed", WorkCompleted { id: 1 }).await?;  // Decoupled
```

---

## Design Decisions

### Decision: Typed Messages with Associated Result

**Context:** Messages must specify their response type for request-reply.

**Choice:** Use associated type `Message::Result` for type-safe responses.

```rust
#[derive(Clone)]
struct Query {
    id: u64,
}

impl Message for Query {
    type Result = String;  // Response type
}

#[async_trait]
impl Handler<Query> for MyActor {
    async fn handle(&mut self, msg: Query, _ctx: &mut ActorContext<Self>) -> String {
        format!("Result for {}", msg.id)  // Must return String
    }
}
```

**Rationale:**

- Compile-time type checking (response type must match)
- Self-documenting (message definition includes response type)
- No runtime type errors (impossible to return wrong type)

**Tradeoff:** More verbose (must define `Message::Result`) vs. Type safety.

### Decision: Asynchronous Send with `async/await`

**Context:** Message sending could be blocking or asynchronous.

**Choice:** All message sends are `async` and use `await`.

```rust
// All sends are async
actor_ref.send(msg).await?;
```

**Rationale:**

- **Non-blocking:** Sender can do other work while message is in flight
- **Integrates with Tokio:** Natural fit with async ecosystem
- **Backpressure support:** Bounded mailbox can apply backpressure

**Tradeoff:** `async/await` syntax overhead vs. Non-blocking concurrency.

### Decision: Mailbox as Message Buffer

**Context:** Messages need temporary storage between send and receive.

**Choice:** Each actor has a mailbox (queue) for pending messages.

```rust
pub enum Mailbox<A> {
    Bounded(BoundedMailbox<A>),    // Fixed capacity, backpressure
    Unbounded(UnboundedMailbox<A>), // Unlimited capacity
}
```

**Rationale:**

- **Decoupling:** Sender doesn't block waiting for receiver
- **Buffering:** Absorbs traffic bursts
- **Backpressure:** Bounded mailbox prevents memory exhaustion

**Tradeoff:** Memory overhead (mailbox storage) vs. Asynchronous messaging.

### Decision: FIFO Message Order

**Context:** Messages could be processed in FIFO, LIFO, or priority order.

**Choice:** Strictly FIFO (first-in-first-out) message processing.

**Rationale:**

- **Predictable:** Messages processed in send order
- **Fair:** No message starvation
- **Simple:** Easy to reason about

**Tradeoff:** No priority support vs. Simplicity and predictability.

---

## Performance Characteristics

### Latency Breakdown (October 2025 Baseline)

**Point-to-Point Messaging:**

| Operation | Latency (P50) | Components |
|-----------|---------------|------------|
| **Direct Send** | 737ns | Enqueue (181ns) + Processing (400ns) + Response (156ns) |
| **Via Broker** | 917ns | Routing (180ns) + Direct Send (737ns) |

**Broadcast Messaging:**

| Subscribers | Latency (P50) | Per-Subscriber |
|-------------|---------------|----------------|
| 1 subscriber | 395ns | 395ns |
| 10 subscribers | 3.95µs | 395ns |
| 100 subscribers | 39.5µs | 395ns |

**Scaling:** Linear with subscriber count (expected for independent delivery).

### Throughput Capacity

**Theoretical Limits:**

| Pattern | Throughput | Calculation |
|---------|-----------|-------------|
| **Direct Point-to-Point** | 1.36M msgs/sec | 1 / 737ns |
| **Via Broker (1:1)** | 1.09M msgs/sec | 1 / 917ns |
| **Broadcast (1:10)** | 253K events/sec | 1 / 3.95µs |

**Real-World Capacity (Conservative):**

- **Direct messaging:** ~1M msgs/sec (accounting for processing overhead)
- **Broker routing:** ~800K msgs/sec (with routing overhead)
- **Broadcast events:** ~200K events/sec (10 subscribers)

**Bottlenecks:**

- Mailbox contention (multiple senders to one actor)
- Message processing time (handler complexity)
- Memory allocation (large message payloads)

### Memory Overhead

**Per-Message Overhead:**

```rust
struct MessageEnvelope<M> {
    message: M,              // Message size
    reply_channel: Option<...>, // 24 bytes (if request-reply)
}
```

**Total:** Message size + 24 bytes (request-reply) or 0 bytes (fire-and-forget)

**Mailbox Memory:**

- **Bounded:** `capacity * (message_size + envelope)` bytes
- **Unbounded:** `current_queue_depth * (message_size + envelope)` bytes

**Optimization:** Use `Arc<T>` for large messages to share data instead of copying.

---

## Comparison with Alternatives

### Message Passing vs. Channels

**Rust Channels (mpsc, broadcast):**

```rust
// Channel-based communication
let (tx, rx) = tokio::sync::mpsc::channel(100);

tokio::spawn(async move {
    tx.send(42).await.unwrap();
});

let value = rx.recv().await.unwrap();
```

**Pros:**

- Lightweight (no actor framework)
- Simple producer-consumer pattern
- Built into Tokio

**Cons:**

- No state encapsulation (just pipes)
- No supervision or fault tolerance
- Manual lifecycle management

**When to Use:** Simple pipelines, stream processing, basic producer-consumer.

**Actor Message Passing:**

**Pros:**

- Encapsulates state with behavior
- Built-in supervision and fault tolerance
- Typed message handlers

**Cons:**

- Requires actor framework
- More abstraction overhead

**When to Use:** Stateful services, fault-tolerant systems, entity-oriented design.

### Message Passing vs. RPC

**Remote Procedure Call (gRPC, JSON-RPC):**

```rust
// RPC-style call
let response = client.call_remote("service.method", params).await?;
```

**Pros:**

- Familiar (like local function calls)
- Language-agnostic (network protocols)
- Tooling (code generation, service definitions)

**Cons:**

- Synchronous (blocks waiting for response)
- Network overhead (serialization, latency)
- Tight coupling (client must know service API)

**When to Use:** Cross-language communication, network services, REST APIs.

**Actor Message Passing:**

**Pros:**

- Asynchronous by default (non-blocking)
- Local optimization (no serialization overhead)
- Type-safe (compile-time checking)

**Cons:**

- Local only (not distributed by default)
- Requires actor framework

**When to Use:** Local concurrency, high-performance messaging, type-safe APIs.

---

## Message Passing Patterns

### Pattern 1: Request-Reply

```rust
// Client sends query, waits for response
let result: Data = data_actor.send(GetData { id: 123 }).await?;
println!("Received: {:?}", result);
```

**Use Case:** Synchronous workflows, queries, RPC-style calls.

### Pattern 2: Fire-and-Forget

```rust
// Client sends notification, continues immediately
logger.send(LogEvent { message: "User logged in" }).await?;
// Don't wait for log to be written
```

**Use Case:** Logging, auditing, background processing.

### Pattern 3: Pub-Sub

```rust
// Multiple subscribers react to event
broker.publish("order.created", OrderCreated { order_id: 456 }).await?;
// Analytics, inventory, shipping all receive event
```

**Use Case:** Event-driven architectures, decoupled services.

### Pattern 4: Request-Multicast

```rust
// Send request to multiple actors, collect responses
let futures: Vec<_> = actors.iter()
    .map(|actor| actor.send(Query { id }))
    .collect();
let results = futures::future::join_all(futures).await;
```

**Use Case:** Scatter-gather, parallel queries, map-reduce.

### Pattern 5: Pipeline

```rust
// Chain actors in processing pipeline
reader_ref.send(ReadData).await?;
// Reader sends to Processor
// Processor sends to Writer
// Writer completes pipeline
```

**Use Case:** ETL pipelines, stream processing, multi-stage workflows.

---

## Best Practices

### 1. Choose Appropriate Messaging Pattern

- **Known recipient + need response:** Request-Reply (direct reference)
- **Known recipient + no response needed:** Fire-and-Forget (direct reference)
- **Multiple recipients:** Pub-Sub (message broker)
- **Unknown recipient:** Discovery via broker, then direct reference

### 2. Optimize Hot Paths

```rust
// Hot path: Use direct reference
let worker_ref = get_worker().await?;
for i in 0..1_000_000 {
    worker_ref.send(HighFrequency { data: i }).await?;  // Fast!
}

// Cold path: Use broker
broker.publish("worker.stats", WorkerStats { ... }).await?;  // Infrequent
```

### 3. Use Arc for Large Messages

```rust
// ❌ Bad: Copy large data for each message
#[derive(Clone)]
struct LargeMessage {
    data: Vec<u8>,  // Copied on every send!
}

// ✅ Good: Share data via Arc
use std::sync::Arc;

#[derive(Clone)]
struct EfficientMessage {
    data: Arc<Vec<u8>>,  // Cheap clone, shared data
}
```

### 4. Monitor Mailbox Queue Depth

```rust
// Detect mailbox backlog
if ctx.mailbox_size() > 1000 {
    log::warn!("Mailbox backlog: {} messages", ctx.mailbox_size());
    // Consider scaling out (more workers)
}
```

---

## Further Reading

### AirsSys RT Documentation

- [Messaging API Reference](../reference/api/messaging.md)
- [Message Broker Reference](../reference/api/broker.md)
- [Performance Reference](../reference/performance.md)

### External Resources

- **Erlang Message Passing:** Official Erlang documentation
- **Akka Messaging:** Akka framework message passing guide
- **CSP vs. Actors:** Comparing communicating sequential processes with actors

---

**Last Updated:** 2025-01-18 (RT-TASK-011 Phase 4 Day 7)
