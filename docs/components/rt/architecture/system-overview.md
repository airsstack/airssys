# System Overview

A comprehensive architectural overview of the `airssys-rt` actor runtime system, including component relationships, data flow, and design principles.

> **Note**: This document provides the high-level architecture. See [Components](./components.md) for detailed subsystem documentation.

## System Philosophy

### Core Principles

`airssys-rt` is designed around the Erlang/OTP actor model with Rust-native performance and type safety:

1. **Fault Tolerance** - "Let it crash" philosophy with supervision trees
2. **Concurrency** - Lightweight actors with message-passing isolation
3. **Type Safety** - Compile-time guarantees via generics and associated types
4. **Performance** - Zero-cost abstractions, static dispatch, minimal allocations
5. **Composability** - Builder patterns and trait-based design

### Design Guidelines

All components follow **Microsoft Rust Guidelines** documented in `.copilot/memory_bank/workspace/microsoft_rust_guidelines.md`:

- **M-DI-HIERARCHY**: Concrete types > generics > `dyn` traits
- **M-AVOID-WRAPPERS**: No smart pointers in public APIs
- **M-SIMPLE-ABSTRACTIONS**: Maximum 1 level of cognitive nesting
- **M-SERVICES-CLONE**: Services implement cheap `Clone` via `Arc<Inner>`
- **M-ESSENTIAL-FN-INHERENT**: Core functionality in inherent methods
- **M-MOCKABLE-SYSCALLS**: All I/O and system calls must be mockable

## High-Level Architecture

### Component Layers

The runtime is organized in seven layers, each building on the previous:

```
┌───────────────────────────────────────────────────────────┐
│                    System Layer (Planned)                  │
│  Runtime coordination, actor registry, distributed nodes   │
└────────────────────┬──────────────────────────────────────┘
                     │
┌────────────────────▼──────────────────────────────────────┐
│                   Monitoring Layer                         │
│   Health checks, metrics, performance tracking, alerting   │
└────────────────────┬──────────────────────────────────────┘
                     │
┌────────────────────▼──────────────────────────────────────┐
│                  Supervisor Layer                          │
│  Fault tolerance, restart strategies, supervision trees    │
└────────────────────┬──────────────────────────────────────┘
                     │
┌────────────────────▼──────────────────────────────────────┐
│                   Mailbox Layer                            │
│    Message queue management, backpressure, buffering       │
└────────────────────┬──────────────────────────────────────┘
                     │
┌────────────────────▼──────────────────────────────────────┐
│                    Actor Layer                             │
│      Actor trait, context, lifecycle, error handling       │
└────────────────────┬──────────────────────────────────────┘
                     │
┌────────────────────▼──────────────────────────────────────┐
│                   Broker Layer                             │
│       Pub/sub message routing, subscriber management       │
└────────────────────┬──────────────────────────────────────┘
                     │
┌────────────────────▼──────────────────────────────────────┐
│                   Message Layer                            │
│         Message types, envelopes, identifiers              │
└───────────────────────────────────────────────────────────┘
```

### Layer Responsibilities

| Layer | Responsibility | Key Components |
|-------|----------------|----------------|
| **Message** | Type-safe message contracts | `Message`, `MessageEnvelope`, `MessageId` |
| **Broker** | Pub/sub routing | `MessageBroker`, `InMemoryMessageBroker` |
| **Actor** | Business logic execution | `Actor`, `ActorContext`, `ActorLifecycle` |
| **Mailbox** | Message buffering | `BoundedMailbox`, `UnboundedMailbox` |
| **Supervisor** | Fault tolerance | `SupervisorNode`, `RestartStrategy`, `Child` |
| **Monitoring** | Health and metrics | `HealthMonitor`, `ActorMetrics`, `SupervisorMetrics` |
| **System** | Runtime coordination | `ActorSystem` (planned), registry, clustering |

## Core Component Diagram

### Component Relationships

```
┌────────────────────────────────────────────────────────────────┐
│                         ActorSystem                             │
│                     (Planned - Q1 2026)                         │
└──────────────┬──────────────────────────────────┬──────────────┘
               │                                  │
               │                                  │
     ┌─────────▼──────────┐            ┌─────────▼─────────┐
     │  SupervisorNode    │            │   HealthMonitor    │
     │  - RestartStrategy │            │   - HealthConfig   │
     │  - Children        │            │   - Metrics        │
     └─────────┬──────────┘            └─────────┬─────────┘
               │                                  │
               │ supervises                       │ monitors
               │                                  │
     ┌─────────▼──────────────────────────────────▼─────────┐
     │                    Actor                              │
     │    ┌──────────────────────────────────────────┐      │
     │    │        ActorContext                      │      │
     │    │  - ActorAddress                          │      │
     │    │  - ActorLifecycle                        │      │
     │    │  - MessageBroker                         │      │
     │    │  - send() / request()                    │      │
     │    └──────────────┬───────────────────────────┘      │
     └───────────────────┼──────────────────────────────────┘
                         │
                         │ publishes/subscribes
                         │
     ┌───────────────────▼───────────────────────────────────┐
     │            InMemoryMessageBroker                       │
     │  - Subscribers: HashMap<ActorId, Sender<Envelope>>    │
     │  - publish(envelope)                                   │
     │  - subscribe(actor_id) → Receiver                      │
     └───────────────────┬───────────────────────────────────┘
                         │
                         │ routes messages
                         │
     ┌───────────────────▼───────────────────────────────────┐
     │                  Mailbox                               │
     │  - BoundedMailbox   (capacity + backpressure)         │
     │  - UnboundedMailbox (unlimited capacity)              │
     │  - Metrics tracking                                    │
     └────────────────────────────────────────────────────────┘
```

## Message Flow Architecture

### Complete Message Path

The following diagram shows the complete path of a message from sender to receiver:

```
  Sender Actor                                       Receiver Actor
┌──────────────┐                                    ┌──────────────┐
│ handle_msg() │                                    │ handle_msg() │
└──────┬───────┘                                    └──────▲───────┘
       │                                                   │
       │ 1. context.send(msg, recipient)                  │ 6. Process message
       ▼                                                   │
┌──────────────────────┐                                  │
│   ActorContext       │                                  │
│ - Wrap in envelope   │                                  │
│ - Add metadata       │                                  │
│ - Set timestamp      │                                  │
└──────┬───────────────┘                                  │
       │                                                   │
       │ 2. broker.publish(envelope)                      │
       ▼                                                   │
┌───────────────────────────────────────┐                 │
│      InMemoryMessageBroker            │                 │
│ ┌───────────────────────────────────┐ │                 │
│ │ Subscribers: HashMap               │ │                 │
│ │ - ActorId → mpsc::Sender<Envelope> │ │                 │
│ └───────────────────────────────────┘ │                 │
│                                       │                 │
│ - Find subscriber by ActorId          │                 │
│ - Clone envelope for each subscriber  │                 │
│ - Send to channel                     │                 │
└──────┬────────────────────────────────┘                 │
       │                                                   │
       │ 3. channel.send(envelope)                        │
       ▼                                                   │
┌───────────────────────────────────────┐                 │
│         Mailbox Queue                 │                 │
│                                       │                 │
│  BoundedMailbox:                      │                 │
│  ┌─────────────────────────────────┐ │                 │
│  │ [Env1][Env2][Env3]...│  │  │  │ │ │                 │
│  └─────────────────────────────────┘ │                 │
│  - Capacity limit                     │                 │
│  - Backpressure (Block/Drop/...)      │                 │
│  - Metrics tracking                   │                 │
│                                       │                 │
│  UnboundedMailbox:                    │                 │
│  ┌─────────────────────────────────┐ │                 │
│  │ [Env1][Env2][Env3]..............│ │                 │
│  └─────────────────────────────────┘ │                 │
│  - No capacity limit                  │                 │
│  - No backpressure                    │                 │
└──────┬────────────────────────────────┘                 │
       │                                                   │
       │ 4. receiver.recv()                               │
       ▼                                                   │
┌───────────────────────────────────────┐                 │
│      Actor Message Loop               │                 │
│                                       │                 │
│  loop {                               │                 │
│    envelope = receiver.recv().await   │                 │
│    actor.handle_message(msg, ctx)     │ ────────────────┘
│  }                                    │    5. Deliver message
└───────────────────────────────────────┘
```

### Latency Breakdown

Based on measurements from `BENCHMARKING.md`:

| Step | Operation | Latency | Percentage |
|------|-----------|---------|------------|
| 1 | Message wrapping (envelope creation) | ~10 ns | 1.4% |
| 2 | Broker routing (mutex + channel send) | ~180 ns | 24.4% |
| 3 | Channel transfer (Tokio mpsc) | ~20 ns | 2.7% |
| 4 | Mailbox buffering | ~181 ns | 24.6% |
| 5 | Actor processing overhead | ~31 ns | 4.2% |
| 6 | Business logic (varies) | ~315 ns | 42.7% |
| **Total** | **Point-to-point roundtrip** | **737 ns** | **100%** |

**Key Insights:**
- Broker routing and mailbox operations dominate latency (~49%)
- Actual business logic is still the majority of time (~43%)
- Infrastructure overhead is sub-microsecond (422 ns)

## Supervision Tree Architecture

### Hierarchical Fault Tolerance

Supervisors can supervise other supervisors, creating a fault-tolerant tree:

```
                    ┌─────────────────────┐
                    │  Root Supervisor    │
                    │  (OneForAll)        │
                    │  RestartPolicy:     │
                    │    Permanent        │
                    └──────┬──────────┬───┘
                           │          │
            ┌──────────────┘          └──────────────┐
            │                                        │
┌───────────▼──────────┐                 ┌───────────▼──────────┐
│ Worker Pool          │                 │ Cache Manager        │
│ Supervisor           │                 │ Supervisor           │
│ (OneForOne)          │                 │ (OneForAll)          │
│ RestartPolicy:       │                 │ RestartPolicy:       │
│   Permanent          │                 │   Transient          │
└───────┬──┬──┬────────┘                 └──────┬───────┬───────┘
        │  │  │                                 │       │
   ┌────┘  │  └────┐                      ┌─────┘       └─────┐
   │       │       │                      │                   │
┌──▼──┐ ┌──▼──┐ ┌──▼──┐            ┌─────▼─────┐      ┌──────▼──────┐
│ W-1 │ │ W-2 │ │ W-3 │            │   Cache   │      │ Persistence │
│Actor│ │Actor│ │Actor│            │   Actor   │      │   Actor     │
└─────┘ └─────┘ └─────┘            └───────────┘      └─────────────┘
```

### Failure Propagation

**Scenario: Worker-2 fails**

With OneForOne strategy in Worker Pool:
1. W-2 fails → `handle_message` returns `Err`
2. Worker Pool Supervisor detects failure
3. W-2's `on_error()` returns `ErrorAction::Restart`
4. Supervisor applies OneForOne: restart only W-2
5. W-2 lifecycle: Running → Failed → Stopping → Starting → Running
6. W-1 and W-3 continue processing unaffected
7. Root Supervisor unaware (non-significant child)

**Scenario: Cache Actor fails**

With OneForAll strategy in Cache Manager:
1. Cache fails → `handle_message` returns `Err`
2. Cache Manager Supervisor detects failure
3. Cache's `on_error()` returns `ErrorAction::Restart`
4. Supervisor applies OneForAll: restart Cache AND Persistence
5. Both actors lifecycle: Running → Failed → Stopping → Starting → Running
6. Consistent state guaranteed across both actors
7. If Cache Manager marked `significant: true`, Root Supervisor would be notified

## Performance Characteristics

### Baseline Metrics

Measured on macOS development machine (October 16, 2025) with release build:

#### Actor System Performance

| Metric | Latency | Throughput | Notes |
|--------|---------|------------|-------|
| Actor spawn (single) | 624.74 ns | 1.6M actors/sec | Sub-microsecond creation |
| Actor spawn (batch of 10) | 681.40 ns/actor | 1.47M actors/sec | Only 9% overhead |
| Message processing | 31.55 ns/msg | 31.7M msgs/sec | Direct processing |
| Message via broker | 211.88 ns/msg | 4.7M msgs/sec | Pub-sub overhead: 6.7x |

#### Message Passing Performance

| Metric | Latency | Throughput | Notes |
|--------|---------|------------|-------|
| Point-to-point | 737 ns roundtrip | 1.36M msgs/sec | Sub-microsecond latency |
| Broadcast (10 actors) | 395 ns total | ~40 ns/subscriber | Efficient multi-cast |
| Mailbox operations | 181.60 ns/op | 5.5M ops/sec | Enqueue + dequeue |

#### Supervision Performance

| Metric | Latency | Notes |
|--------|---------|-------|
| Child spawn (builder API) | 5-20 µs | Type-safe configuration |
| OneForOne restart | 10-50 µs | Single child stop → start |
| OneForAll restart (3 children) | 30-150 µs | ~3x OneForOne |
| RestForOne restart (2 children) | 20-100 µs | Between OneForOne and OneForAll |

### Scalability Characteristics

**Memory per actor:**
- Actor struct: ~500 bytes - 2 KB (depends on state)
- ActorContext: ~200 bytes
- Mailbox (unbounded): ~100 bytes base
- Mailbox (bounded 100): ~244 bytes
- **Total**: ~1-3 KB per actor typical

**Throughput scaling:**
- Message processing scales linearly with message count
- Broadcast scales linearly with subscriber count (~40 ns/subscriber)
- Batch actor spawn has 9% overhead vs single spawn (excellent)

**Concurrency:**
- Actors are `Send + Sync` - true parallelism
- Message broker uses `Arc<Mutex<HashMap>>` - contention point for many subscribers
- Tokio runtime handles async/await scheduling efficiently

## Type Safety Architecture

### Generic Constraints

The runtime uses generics instead of trait objects for zero-cost abstraction:

```rust
// ✅ CORRECT - Generic constraints (static dispatch)
pub trait Actor: Send + Sync + 'static {
    type Message: Message;
    type Error: Error + Send + Sync + 'static;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error>;
}

// ❌ FORBIDDEN - Trait objects (dynamic dispatch, heap allocation)
async fn handle_message(
    &mut self,
    message: Box<dyn Message>,
    context: &mut ActorContext<Box<dyn Message>, Box<dyn MessageBroker>>,
) -> Result<(), Box<dyn Error>>;
```

**Benefits:**
- Compile-time type checking
- No runtime type dispatch overhead
- No heap allocations for message passing
- Better compiler optimizations

### Associated Types

Associated types provide type safety without type parameter explosion:

```rust
impl Actor for CounterActor {
    type Message = CounterMessage;  // Specific message type
    type Error = CounterError;      // Specific error type

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,  // CounterMessage, not generic
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {  // CounterError, not generic
        // Fully type-safe, no casts needed
        self.value += message.delta;
        Ok(())
    }
}
```

## Future Integrations (Planned)

**airssys-wasm** - WebAssembly component hosting:
- Actors can host WASM components as children
- Sandboxed component execution
- Capability-based security
- Component lifecycle management

**Distributed nodes** - Multi-node actor systems:
- Network message broker
- Transparent remote actor addressing
- Distributed supervision
- Cluster membership

## Directory Structure

The runtime codebase follows the layered architecture:

```
airssys-rt/
├── src/
│   ├── lib.rs              # Public API surface
│   ├── message/            # Message Layer
│   │   ├── mod.rs          # Message trait, MessageId
│   │   └── envelope.rs     # MessageEnvelope
│   ├── broker/             # Broker Layer
│   │   ├── mod.rs          # Re-exports
│   │   ├── traits.rs       # MessageBroker trait
│   │   └── in_memory.rs    # InMemoryMessageBroker
│   ├── actor/              # Actor Layer
│   │   ├── mod.rs          # Re-exports
│   │   ├── traits.rs       # Actor trait
│   │   ├── context.rs      # ActorContext
│   │   └── lifecycle.rs    # ActorLifecycle, ActorState
│   ├── mailbox/            # Mailbox Layer
│   │   ├── mod.rs          # Re-exports
│   │   ├── bounded.rs      # BoundedMailbox
│   │   └── unbounded.rs    # UnboundedMailbox
│   ├── supervisor/         # Supervisor Layer
│   │   ├── mod.rs          # Re-exports
│   │   ├── supervisor.rs   # SupervisorNode
│   │   ├── builder.rs      # SupervisorBuilder (RT-TASK-013)
│   │   ├── strategy.rs     # OneForOne, OneForAll, RestForOne
│   │   └── child.rs        # Child trait, ChildSpec
│   ├── monitoring/         # Monitoring Layer
│   │   ├── mod.rs          # Re-exports
│   │   ├── health.rs       # HealthMonitor, ChildHealth
│   │   └── metrics.rs      # ActorMetrics, SupervisorMetrics
│   ├── system/             # System Layer (Planned)
│   │   └── mod.rs          # Future: ActorSystem, registry
│   └── util/               # Utilities
│       ├── mod.rs          # Re-exports
│       ├── address.rs      # ActorAddress, ActorId
│       └── id.rs           # ChildId, SupervisorId
├── examples/               # Working examples (15 total)
├── tests/                  # Integration tests
├── benches/                # Criterion benchmarks
└── docs/                   # mdBook documentation
```

## Design Patterns

### Builder Pattern (RT-TASK-013)

Type-safe configuration using builders:

```rust
let supervisor = SupervisorNode::builder()
    .with_strategy(OneForOne::new())
    .add_child(spec1, Box::new(worker1))
    .add_child(spec2, Box::new(worker2))
    .build()?;
```

**Benefits:**
- Compile-time validation
- Fluent API
- Clear intent
- Minimal overhead (5-20 µs)

### Services Clone Pattern

Services implement cheap `Clone` via `Arc<Inner>`:

```rust
#[derive(Clone)]
pub struct InMemoryMessageBroker<M: Message> {
    // Arc makes clone cheap (just increment refcount)
    subscribers: Arc<Mutex<HashMap<ActorId, mpsc::Sender<MessageEnvelope<M>>>>>,
}
```

**Benefits:**
- Services can be shared across actors
- No deep copying overhead
- Thread-safe via Arc
- Simple ownership model

### Dependency Injection

Generic constraints for testability:

```rust
async fn handle_message<B: MessageBroker<Self::Message>>(
    &mut self,
    message: Self::Message,
    context: &mut ActorContext<Self::Message, B>,  // B is injected
) -> Result<(), Self::Error>
```

**Benefits:**
- Mock brokers in tests
- Swap implementations (InMemory, Network, etc.)
- No runtime coupling
- Compile-time verification

## Next Steps

For detailed subsystem architecture, see:

- [Components](./components.md) - Deep dive into each layer
- [Core Concepts](./core-concepts.md) - Fundamental concepts and examples
- [Actor Model](./actor-model.md) - Actor trait and lifecycle
- [Message Passing](./message-passing.md) - Messaging system details
- [Supervision](./supervision.md) - Fault tolerance and restart strategies
- [Process Lifecycle](./process-lifecycle.md) - State management

For API reference:
- [API Reference](../reference/api/core.md) - Complete API documentation

For performance details:
- [Performance Reference](../reference/performance.md) - Detailed metrics and benchmarks
