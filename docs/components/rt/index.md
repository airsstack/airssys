# RT - Actor Runtime

**Lightweight Erlang-Actor model runtime system for high-concurrency applications with BEAM-inspired supervision and fault tolerance.**

## Vision

Bring the proven resilience patterns of Erlang/OTP to Rust system programming with zero-cost abstractions, enabling developers to build fault-tolerant concurrent systems without sacrificing performance or type safety.

## Motivation

The need for `airssys-rt` emerged from studying decades of battle-tested concurrent systems and recognizing a gap in the Rust ecosystem:

### The Problem

Building highly concurrent, fault-tolerant systems is notoriously difficult:

1. **Shared State Complexity**: Traditional threading with shared memory leads to race conditions, deadlocks, and hard-to-debug concurrency bugs. Even with Rust's safety guarantees, reasoning about shared state across threads is cognitively expensive.

2. **Failure Propagation**: In monolithic systems, a single component failure often cascades to take down the entire application. Error handling becomes defensive programming scattered throughout the codebase.

3. **No Built-in Supervision**: Rust's standard library provides threads and async tasks, but no higher-level patterns for managing process lifecycles, automatic restarts, or hierarchical fault tolerance.

4. **High Abstraction Cost**: Existing actor libraries either sacrifice performance (dynamic dispatch, allocations) or require complex type gymnastics that hurt developer experience.

### The Erlang/OTP Inspiration

Erlang's BEAM virtual machine has proven for 35+ years that the actor model with supervision trees enables:

- **Systems that never go down** (99.9999999% uptime - "nine nines")
- **Graceful degradation** under failure
- **Simple reasoning** about concurrent systems
- **Hot code reloading** without downtime

Key insight: **"Let it crash"** philosophy - instead of defensive programming everywhere, write simple code for the happy path and let supervisors handle failures.

### Why Build airssys-rt?

**What makes airssys-rt different from existing Rust actor libraries:**

1. **Zero-Cost Abstractions**: Generic constraints instead of trait objects (no `dyn` in public APIs), compile-time type checking eliminates runtime overhead
2. **Type-Safe Messaging**: Strongly-typed message protocols verified at compile time
3. **BEAM-Inspired Supervision**: OneForOne, OneForAll, RestForOne strategies proven in production
4. **System Programming Focus**: Designed for low-level system operations and AirsSys ecosystem integration
5. **Tokio Integration**: Seamless integration with Rust's async ecosystem

**Real-World Example: File Processing Service**

Without RT (traditional approach):
```rust
// Shared state with locks - complexity, potential deadlocks
let files = Arc::new(Mutex::new(FileQueue::new()));

// Manual error handling - failure propagation
tokio::spawn(async move {
    loop {
        match process_file().await {
            Ok(_) => {},
            Err(e) => {
                log::error!("File processing failed: {}", e);
                // Now what? Manual restart? Crash? Ignore?
            }
        }
    }
});
```

With RT (actor model):
```rust
// No shared state - messages only
#[async_trait]
impl Actor for FileProcessor {
    type Message = FileMsg;
    
    async fn handle_message(&mut self, msg: FileMsg, ctx: &mut ActorContext) -> Result<(), Error> {
        match msg {
            FileMsg::Process(file) => {
                // Simple happy-path code
                self.process(file).await?
            }
        }
        Ok(())
    }
}

// Supervisor handles failures automatically
let supervisor = SupervisorBuilder::new()
    .strategy(RestartStrategy::OneForOne)  // Restart only failed processor
    .max_restarts(3, Duration::from_secs(60))
    .build();

// If processor crashes, supervisor restarts it automatically
supervisor.spawn(FileProcessor::new()).await?;
```

**Benefits:**

- ✅ No locks, no shared state, no race conditions
- ✅ Simple error handling - just propagate with `?`
- ✅ Automatic failure recovery - supervisor restarts failed actors
- ✅ Fault isolation - one file failure doesn't crash others
- ✅ Type-safe messages - compiler prevents invalid messages

## Key Features

### ⚡ High Performance

Benchmarked performance characteristics:

- **Actor Spawn**: ~625ns (single), ~681ns/actor (batch of 10)
- **Message Processing**: ~31.5ns/message (direct actor handling)
- **Message Throughput**: ~4.7M messages/sec (4.7x target)
- **Broker Routing**: ~212ns/message (registry lookup + mailbox send)
- **Linear Scaling**: 6% overhead scaling from 1→50 actors

See [Benchmarking Documentation](../../BENCHMARKING.md) for details.

### 🛡️ Fault Tolerance

**Supervision Strategies** (BEAM-inspired):

- **OneForOne**: Restart only the failed child
- **OneForAll**: Restart all children if one fails
- **RestForOne**: Restart failed child and all started after it

**Restart Policies**:

- **Permanent**: Always restart (critical services)
- **Temporary**: Never restart (one-time tasks)
- **Transient**: Restart only if abnormal termination

**Health Monitoring**:

- Proactive health checks
- Configurable restart limits
- Exponential backoff

### 🔒 Zero-Cost Abstractions

**No Runtime Overhead**:

- Generic constraints (`Actor<M, B>`) - monomorphization eliminates dynamic dispatch
- Compile-time type checking - no runtime message type errors
- Inline optimizations - compiler can optimize across abstraction boundaries

**Type Safety**:
```rust
// Compiler prevents invalid messages
impl Actor for CounterActor {
    type Message = CounterMsg;  // Only CounterMsg accepted
    // ...
}

// ✅ Compiler accepts
actor.send(CounterMsg::Increment).await?;

// ❌ Compiler rejects
actor.send(FileMsg::Process(file)).await?;  // Type mismatch!
```

### 💬 Flexible Messaging

**Message Patterns**:

- Point-to-point (actor addresses)
- Pub-Sub (topic-based broadcasting)
- Request-Response (with correlation IDs)
- Message priority and expiration

**Backpressure Control**:

- **Block**: Wait until mailbox has space
- **Drop**: Drop message if mailbox full
- **Reject**: Return error to sender

### 🔍 Monitoring & Observability

**Event Tracking**:

- Actor lifecycle events (start, stop, crash)
- Message delivery and processing
- Supervisor actions (restarts, escalations)
- Mailbox metrics (queue depth, throughput)

**Zero-Overhead Option**:

- `NoopMonitor` compiles away completely (zero runtime cost)

## Use Cases

### High-Concurrency Servers

Build servers handling thousands of concurrent connections:

```rust
use airssys_rt::prelude::*;

struct ConnectionActor {
    socket: TcpStream,
}

#[async_trait]
impl Actor for ConnectionActor {
    type Message = ConnectionMsg;
    
    async fn handle_message(&mut self, msg: ConnectionMsg, ctx: &mut ActorContext) -> Result<(), Error> {
        match msg {
            ConnectionMsg::Data(bytes) => {
                self.socket.write_all(&bytes).await?;
                Ok(())
            }
        }
    }
}

// Supervisor restarts failed connections automatically
let supervisor = SupervisorBuilder::new()
    .strategy(RestartStrategy::OneForOne)
    .build();
```

### Event-Driven Architectures

Build complex event processing pipelines:

```rust
// Pipeline: Ingestion → Validation → Processing → Storage

struct PipelineActor {
    broker: InMemoryMessageBroker<PipelineMsg>,
}

#[async_trait]
impl Actor for PipelineActor {
    async fn handle_message(&mut self, msg: PipelineMsg, ctx: &mut ActorContext) -> Result<(), Error> {
        match msg {
            PipelineMsg::Ingest(data) => {
                // Validate and forward
                let validated = self.validate(data)?;
                self.broker.publish("pipeline.validated", PipelineMsg::Validate(validated)).await?;
            }
            // ... more stages
        }
        Ok(())
    }
}
```

### System Programming with Fault Tolerance

Integrate with airssys-osl for secure system operations:

```rust
use airssys_rt::prelude::*;
use airssys_osl::operations::filesystem::FileReadOperation;

struct OSLActor;

#[async_trait]
impl Actor for OSLActor {
    type Message = OSLMessage;
    
    async fn handle_message(&mut self, msg: OSLMessage, ctx: &mut ActorContext) -> Result<(), Error> {
        match msg {
            OSLMessage::ReadFile(path) => {
                // Secure file operation via OSL
                let operation = FileReadOperation::new(path);
                let result = execute_osl_operation(operation).await?;
                // Process result
                Ok(())
            }
        }
    }
}

// Supervisor ensures OSL operations are fault-tolerant
let supervisor = OSLSupervisor::new(broker.clone());
supervisor.start().await?;
```

### Microservice Coordination

Coordinate complex workflows across services:

```rust
// Saga pattern for distributed transactions
struct SagaCoordinator {
    steps: Vec<SagaStep>,
    compensations: Vec<Compensation>,
}

#[async_trait]
impl Actor for SagaCoordinator {
    async fn handle_message(&mut self, msg: SagaMsg, ctx: &mut ActorContext) -> Result<(), Error> {
        match msg {
            SagaMsg::Execute => {
                // Execute steps, collect compensations
                for step in &self.steps {
                    match step.execute().await {
                        Ok(_) => self.compensations.push(step.compensation()),
                        Err(e) => {
                            // Rollback all compensations
                            self.rollback().await?;
                            return Err(e);
                        }
                    }
                }
                Ok(())
            }
        }
    }
}
```

## Quick Start

### Installation

```toml
[dependencies]
airssys-rt = "0.1.0"
async-trait = "0.1"
tokio = { version = "1.47", features = ["full"] }
```

### Your First Actor

```rust
use airssys_rt::prelude::*;
use async_trait::async_trait;

// 1. Define message type
#[derive(Debug, Clone)]
enum CounterMsg {
    Increment,
    GetCount(tokio::sync::oneshot::Sender<u64>),
}

impl Message for CounterMsg {
    const MESSAGE_TYPE: &'static str = "counter";
}

// 2. Define actor with state
struct CounterActor {
    count: u64,
}

// 3. Implement Actor trait
#[async_trait]
impl Actor for CounterActor {
    type Message = CounterMsg;
    type Error = std::io::Error;
    
    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        msg: Self::Message,
        ctx: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        match msg {
            CounterMsg::Increment => {
                self.count += 1;
                println!("Count: {}", self.count);
            }
            CounterMsg::GetCount(reply) => {
                let _ = reply.send(self.count);
            }
        }
        Ok(())
    }
}

// 4. Create and run actor
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let broker = InMemoryMessageBroker::new();
    let actor = CounterActor { count: 0 };
    
    // Spawn actor
    let mailbox = BoundedMailbox::new(100);
    let mut ctx = ActorContext::new("counter-1".to_string(), mailbox, broker.clone());
    
    tokio::spawn(async move {
        actor.run(ctx).await
    });
    
    // Send messages
    broker.send("counter-1", CounterMsg::Increment).await?;
    
    Ok(())
}
```

### Adding Supervision

```rust
use airssys_rt::supervisor::*;

// Create supervisor with restart strategy
let supervisor = SupervisorBuilder::new()
    .strategy(RestartStrategy::OneForOne)      // Restart only failed child
    .max_restarts(3, Duration::from_secs(60))  // Max 3 restarts per minute
    .build();

// Spawn supervised actors
let counter = CounterActor { count: 0 };
supervisor.spawn_child(
    "counter-1",
    Box::new(counter),
    RestartPolicy::Permanent,  // Always restart on failure
).await?;

// Supervisor automatically restarts actor on crash
```

## Architecture

```
┌──────────────────────────────────────────────────┐
│           Application Layer                      │
│  ┌────────────────────────────────────────────┐  │
│  │  Your Actors (Business Logic)              │  │
│  │  - State encapsulation                     │  │
│  │  - Message handling                        │  │
│  └────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────┘
                       ↓
┌──────────────────────────────────────────────────┐
│       Actor Runtime (airssys-rt)                 │
│  ┌───────────┐  ┌──────────┐  ┌──────────────┐  │
│  │  Mailbox  │  │  Broker  │  │  Supervisor  │  │
│  │  System   │  │  Routing │  │  Trees       │  │
│  └───────────┘  └──────────┘  └──────────────┘  │
│  ┌───────────┐  ┌──────────┐  ┌──────────────┐  │
│  │ Lifecycle │  │Monitoring│  │  Message     │  │
│  │Management │  │ Events   │  │  Envelope    │  │
│  └───────────┘  └──────────┘  └──────────────┘  │
└──────────────────────────────────────────────────┘
                       ↓
┌──────────────────────────────────────────────────┐
│         Tokio Async Runtime                      │
└──────────────────────────────────────────────────┘
```

## Documentation

### Tutorials (Learn by Doing)

- [Getting Started](tutorials/getting-started.md) - Set up your first project
- [Your First Actor](tutorials/actor-creation.md) - Create and run an actor
- [Message Handling](tutorials/message-handling.md) - Handle messages
- [Building a Supervisor Tree](tutorials/supervision-setup.md) - Add fault tolerance

### How-To Guides (Practical Tasks)

- [Actor Development](guides/actor-development.md) - Actor patterns and best practices
- [Supervisor Patterns](guides/supervisor-patterns.md) - Fault tolerance strategies
- [Message Passing](guides/message-passing.md) - Advanced messaging patterns

### Architecture (Understanding)

- [Core Concepts](architecture/core-concepts.md) - Fundamental concepts
- [Actor Model Design](architecture/actor-model.md) - Actor model implementation
- [Supervision](architecture/supervision.md) - Supervision tree architecture

### API Reference

- [Core Types](api/core-types.md) - Essential types and traits
- [Actor Traits](api/actor-traits.md) - Actor trait reference
- [Supervisor API](api/supervisor-api.md) - Supervisor API

### Explanation (Deep Dives)

- [The Actor Model](explanation/actor-model.md) - Why actors?
- [Supervision & Fault Tolerance](explanation/supervision.md) - BEAM inspiration
- [Performance by Design](explanation/performance-design.md) - Zero-cost abstractions

## Current Status

**Version**: 0.1.0  
**Status**: ✅ Production Ready

### What's Complete

- ✅ **Actor System**: Zero-cost actor abstractions with compile-time type safety
- ✅ **Message Passing**: Type-safe messaging with broker-based routing
- ✅ **Supervision Trees**: OneForOne, OneForAll, RestForOne strategies
- ✅ **Mailbox System**: Bounded/unbounded mailboxes with backpressure
- ✅ **Monitoring**: Event tracking and metrics
- ✅ **OSL Integration**: Complete integration with airssys-osl
- ✅ **Comprehensive Testing**: 381 tests passing
- ✅ **Performance Benchmarks**: 12 benchmarks (all targets exceeded)

### Performance Achievements

- ✅ **4.7M messages/sec** (4.7x target of 1M/sec)
- ✅ **~625ns actor spawn** (sub-microsecond)
- ✅ **~31.5ns message processing** (direct handling)
- ✅ **6% overhead** (linear scaling 1→50 actors)

## Examples

See the [`examples/` directory](https://github.com/airsstack/airssys/tree/main/airssys-rt/examples):

- `getting_started.rs` - Basic actor setup ⭐
- `actor_basic.rs` - Simple actor implementation
- `actor_lifecycle.rs` - Lifecycle hooks
- `supervisor_basic.rs` - Basic supervision
- `supervisor_strategies.rs` - All restart strategies
- `message_patterns.rs` - Advanced messaging
- `worker_pool.rs` - Worker pool pattern

Run examples:
```bash
cargo run --example getting_started
```

## Resources

- **Repository**: [github.com/airsstack/airssys](https://github.com/airsstack/airssys)
- **Crate**: [crates.io/crates/airssys-rt](https://crates.io/crates/airssys-rt)
- **API Docs**: Run `cargo doc --open` in `airssys-rt/`
- **Benchmarks**: See [BENCHMARKING.md](../../airssys-rt/BENCHMARKING.md)
- **Research**: See [Research Documentation](research/index.md)

## License

Dual-licensed under Apache License 2.0 or MIT License.

---

**Next Steps**: Start with [Your First Actor Tutorial](tutorials/actor-creation.md) or explore [Working Examples](https://github.com/airsstack/airssys/tree/main/airssys-rt/examples).
