# airssys-rt (Lightweight Actor Runtime Model)

A high-performance, fault-tolerant actor runtime for Rust, inspired by the Erlang/BEAM virtual machine. `airssys-rt` provides lightweight virtual process management with supervisor trees, designed specifically for system programming within the AirsSys ecosystem.

## 🎯 Project Vision

`airssys-rt` implements the proven actor model and supervision patterns from Erlang/OTP in a Rust-native runtime. Rather than recreating the entire BEAM virtual machine, it focuses on the essential patterns that make concurrent systems resilient and scalable.

### What Makes This Different

- **Virtual Process Model**: Lightweight, isolated execution contexts (not OS processes)
- **In-Memory Management**: Pure in-memory process lifecycle and communication
- **System Programming Focus**: Optimized for low-level system operations and integration
- **AirsSys Integration**: Designed to work seamlessly with airssys-osl and airssys-wasm

## 🏗️ Core Architecture

### Actor Model Implementation
```rust
// Virtual processes with encapsulated state
struct VirtualProcess {
    pid: ProcessId,
    state: ActorState,
    mailbox: MessageQueue,
    supervisor: Option<ProcessId>,
}

// Message-passing communication
actor.send(Message::Command(data)).await?;
```

### Supervision Trees
```rust
// Hierarchical fault tolerance
let supervisor = Supervisor::new()
    .strategy(RestartStrategy::OneForOne)
    .child("worker1", WorkerActor::new())
    .child("worker2", WorkerActor::new())
    .start().await?;
```

### Message Processing
```rust
// Sequential message handling with state isolation
impl Actor for MyActor {
    async fn handle(&mut self, msg: Message) -> Result<(), ActorError> {
        match msg {
            Message::Process(data) => self.process_data(data).await,
            Message::Status => self.report_status().await,
        }
    }
}
```

## ⚡ Performance Characteristics

| Metric | Target | Current Status |
|--------|--------|----------------|
| **Concurrent Actors** | 10,000+ | 🏗️ In Development |
| **Message Latency** | <1ms | 🏗️ In Development |
| **Message Throughput** | 1M+/sec | 🏗️ In Development |
| **Memory Per Actor** | <1KB | 🏗️ In Development |
| **Spawn Time** | <100μs | 🏗️ In Development |
| **CPU Overhead** | <5% | 🏗️ In Development |

## 🧩 Core Components

### 1. Virtual Process Manager
- Lightweight process creation and lifecycle management
- Process registry and addressing
- Memory-efficient process storage

### 2. Message Passing System
- Zero-copy message delivery where possible
- Backpressure and flow control
- Message routing and addressing

### 3. Supervision Framework
- Hierarchical process monitoring
- Configurable restart strategies (OneForOne, OneForAll, RestForOne)
- Fault isolation and error propagation

### 4. Scheduler Integration
- Tokio-based cooperative scheduling
- Fair scheduling across actors
- Integration with async/await ecosystem

## 🎮 Actor Model Principles

### Encapsulation
```rust
// Actors maintain private internal state
struct CounterActor {
    count: i64,        // Private state - never shared
    name: String,      // Only accessible via message handling
}
```

### Asynchronous Message Passing
```rust
// No shared memory - only message communication
let response = counter_actor
    .send(CounterMessage::Increment)
    .await?;
```

### Sequential Processing
```rust
// Messages processed one at a time, ensuring consistency
async fn handle_message(&mut self, msg: CounterMessage) {
    match msg {
        CounterMessage::Increment => self.count += 1,
        CounterMessage::Decrement => self.count -= 1,
        CounterMessage::GetValue => // return current count
    }
}
```

## 🛡️ Fault Tolerance Model

### "Let It Crash" Philosophy
Instead of defensive programming with extensive error handling, `airssys-rt` follows the Erlang principle:
- Write simple, clear code for the expected case
- Let processes fail cleanly when unexpected errors occur
- Use supervisors to detect failures and restart processes

### Supervision Strategies
```rust
pub enum RestartStrategy {
    OneForOne,    // Restart only the failed process
    OneForAll,    // Restart all supervised processes
    RestForOne,   // Restart failed process and those started after it
}

pub enum RestartPolicy {
    Permanent,    // Always restart
    Temporary,    // Never restart
    Transient,    // Restart only if abnormal termination
}
```

## 🏛️ System Integration

### AirsSys Ecosystem Integration
```rust
use airssys_osl::{SecurityContext, ActivityLogger, ResourceLimits};

// Integration with OS layer for system operations
let actor = SystemActor::new()
    .with_security_context(security_ctx)
    .with_activity_logger(logger)
    .with_resource_limits(limits)
    .spawn().await?;
```

### Future WASM Integration
```rust
// Planned integration with airssys-wasm
let wasm_actor = WasmActor::new()
    .load_component("./component.wasm")
    .supervised_by(supervisor)
    .spawn().await?;
```

## 📊 Development Timeline

### Phase 1: Foundation (Q1 2026)
- ✅ Research and architecture design
- 🏗️ Core virtual process implementation
- 🏗️ Basic message passing system
- 🏗️ Simple actor lifecycle management

### Phase 2: Supervision (Q2 2026)
- 🏗️ Supervision tree implementation
- 🏗️ Restart strategies and policies
- 🏗️ Fault isolation and error handling
- 🏗️ Process linking and monitoring

### Phase 3: Optimization (Q2-Q3 2026)
- 🏗️ Performance optimization and tuning
- 🏗️ Advanced scheduling strategies
- 🏗️ Memory management optimization
- 🏗️ Comprehensive benchmarking

### Phase 4: Advanced Features (Q3-Q4 2026)
- 🏗️ Distribution support (planned)
- 🏗️ Hot code loading (research phase)
- 🏗️ Advanced monitoring and metrics
- 🏗️ Ecosystem integration completion

## 📚 Documentation

Comprehensive documentation is available in the [docs](./docs/) directory:

- **[Architecture Guide](./docs/src/architecture/)** - Core concepts and design patterns
- **[Research Documentation](./docs/src/researches/)** - BEAM analysis and design decisions  
- **[Implementation Guide](./docs/src/implementation/)** - Practical usage examples
- **[API Reference](./docs/src/api/)** - Complete API documentation

### Building Documentation
```bash
# Install mdBook (one-time setup)
cargo install mdbook

# Serve documentation locally
mdbook serve docs

# Build documentation
mdbook build docs
```

## 🎯 Use Cases

### High-Concurrency Servers
```rust
// Handle thousands of concurrent connections
let server = TcpServer::new()
    .with_connection_actor(ConnectionActor::new())
    .with_supervisor(connection_supervisor)
    .bind("0.0.0.0:8080").await?;
```

### Event-Driven Architecture
```rust
// Complex event processing with state management
let event_processor = EventProcessor::new()
    .with_handler(OrderHandler::new())
    .with_handler(PaymentHandler::new())
    .with_supervisor(business_logic_supervisor)
    .start().await?;
```

### System Service Management
```rust
// Reliable system service coordination
let service_manager = ServiceManager::new()
    .service("database", DatabaseService::new())
    .service("cache", CacheService::new())
    .service("metrics", MetricsService::new())
    .with_restart_strategy(RestartStrategy::OneForOne)
    .start().await?;
```

## 🚀 Getting Started

### Prerequisites
- Rust 2021 Edition or later
- Tokio async runtime

### Basic Example
```rust
use airssys_rt::{Actor, ActorSystem, Message};

#[derive(Debug)]
struct PingMessage;

struct PingActor {
    count: usize,
}

impl Actor for PingActor {
    type Message = PingMessage;
    
    async fn handle(&mut self, _msg: PingMessage) -> Result<(), ActorError> {
        self.count += 1;
        println!("Ping! Count: {}", self.count);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let system = ActorSystem::new().await?;
    
    let ping_actor = PingActor { count: 0 };
    let actor_ref = system.spawn(ping_actor).await?;
    
    actor_ref.send(PingMessage).await?;
    
    Ok(())
}
```

## ⚠️ Important Notes

> [!IMPORTANT]
> 
> **`airssys-rt` is NOT a BEAM replacement**. This project implements actor model patterns inspired by Erlang/OTP but optimized for Rust system programming. The "processes" are virtual processes managed in-memory, not OS processes.

> [!NOTE]
>
> **Current Status**: Active development and architecture phase. Not ready for production use. Target timeline: Q1-Q4 2026.

## 🤝 Contributing

`airssys-rt` is part of the larger AirsSys ecosystem. See the main [AGENTS.md](../AGENTS.md) for development guidelines and contribution standards.

### Development Commands
```bash
# Check code quality
cargo check --workspace
cargo clippy --workspace --all-targets --all-features

# Run tests
cargo test --workspace

# Serve documentation
mdbook serve docs
```

## 📄 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT License ([LICENSE-MIT](../LICENSE-MIT))

at your option.
