# airssys-rt - Lightweight Erlang-Actor Model Runtime

High-performance actor system with zero-cost abstractions, compile-time type safety, and BEAM-inspired supervision for building fault-tolerant concurrent applications.

## 🚀 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
airssys-rt = "0.1.0"
async-trait = "0.1"
tokio = { version = "1.47", features = ["full"] }
```

Basic example using the prelude:

```rust
use airssys_rt::prelude::*;
use async_trait::async_trait;

// 1. Define your message type
#[derive(Debug, Clone)]
enum CounterMsg {
    Increment,
    GetCount(tokio::sync::oneshot::Sender<u64>),
}

impl Message for CounterMsg {
    const MESSAGE_TYPE: &'static str = "counter";
}

// 2. Define your actor
struct CounterActor {
    count: u64,
}

// 3. Implement the Actor trait
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
            CounterMsg::Increment => self.count += 1,
            CounterMsg::GetCount(reply) => {
                let _ = reply.send(self.count);
            }
        }
        Ok(())
    }
}
```

## 🎯 Project Vision

`airssys-rt` implements the proven actor model and supervision patterns from Erlang/OTP in a Rust-native runtime with modern zero-cost abstractions. Rather than recreating the entire BEAM virtual machine, it focuses on the essential patterns that make concurrent systems resilient and scalable.

### What Makes This Different

- **Zero-Cost Abstractions**: Generic constraints instead of trait objects (§6.2), no runtime overhead
- **Type Safety**: Compile-time message type verification, no `dyn` traits in public APIs
- **High Performance**: ~625ns actor spawn, 31.5ns/msg processing, 4.7M msgs/sec throughput
- **BEAM-Inspired**: Supervision trees with OneForOne, OneForAll, RestForOne strategies  
- **System Programming Focus**: Optimized for low-level system operations and integration
- **AirsSys Integration**: Designed to work seamlessly with airssys-osl and airssys-wasm

## 📊 Performance Characteristics

Based on RT-TASK-008 baseline measurements (Oct 16, 2025):

- **Actor spawn**: ~625ns (single), ~681ns/actor (batch of 10)
- **Message creation**: ~737ns (with envelope and metadata)
- **Message processing**: ~31.5ns/message (direct actor handling)
- **Broker routing**: ~212ns/message (registry lookup + mailbox send)
- **Mailbox operations**: ~182ns/message (bounded mailbox with metrics)
- **Message throughput**: ~4.7M messages/sec (4.7x target of 1M/sec)
- **Scaling**: Linear with 6% overhead (1→50 actors)

Source: `BENCHMARKING.md` §6.1-§6.3

## 🏗️ Core Architecture

The runtime is built on several key architectural patterns:

### Actor System
- **Generic Actor Trait**: `Actor<M, B>` with compile-time type safety
- **Zero-Cost Dispatch**: Monomorphization eliminates runtime overhead  
- **Lifecycle Hooks**: `pre_start()`, `handle_message()`, `post_stop()`

### Message Passing
- **Type-Safe Messages**: Generic `Message` trait with compile-time verification
- **Message Envelopes**: Metadata wrappers (priority, expiration, correlation)
- **Broker-Based Routing**: `InMemoryMessageBroker` with pub-sub patterns
- **Backpressure Control**: Configurable strategies (Block, Drop, Reject)

### Supervision Trees
- **Child Trait**: Lifecycle interface (`start()`, `stop()`, `health_check()`)
- **Three Strategies**: OneForOne, OneForAll, RestForOne
- **Restart Policies**: Permanent, Transient, Temporary
- **Health Monitoring**: Proactive failure detection

### Monitoring & Observability
- **Event Tracking**: Actor, supervision, mailbox, broker events
- **Zero-Overhead Option**: `NoopMonitor` compiles away completely
- **Metrics**: Queue depth, throughput, error rates

For detailed architecture documentation and working examples, see:
- **API Documentation**: Run `cargo doc --open --package airssys-rt`
- **Working Examples**: See `examples/` directory
- **Memory Bank**: `.copilot/memory_bank/sub_projects/airssys-rt/`

## 🧩 Module Organization

- **`prelude`** - Convenient re-exports of commonly used types (start here!)
- **`actor`** - Actor trait, lifecycle, and context
- **`message`** - Message trait, envelopes, and priority
- **`mailbox`** - Message queuing with backpressure
- **`broker`** - Message routing and pub/sub
- **`supervisor`** - Supervision trees and fault tolerance
- **`monitoring`** - Event tracking and metrics
- **`system`** - Actor system configuration
- **`util`** - Utilities (addresses, IDs)

## 🎮 Actor Model Principles

The runtime implements core actor model principles:

- **Encapsulation**: Actors maintain private internal state, never shared directly
- **Message Passing**: Communication only through asynchronous message passing, no shared memory
- **Sequential Processing**: Messages processed one at a time, ensuring state consistency
- **Isolation**: Actor failures are isolated and managed by supervisors

See working examples demonstrating these principles:
- [`examples/actor_basic.rs`](./examples/actor_basic.rs) - Basic actor implementation
- [`examples/actor_lifecycle.rs`](./examples/actor_lifecycle.rs) - Actor lifecycle management
- [`examples/supervisor_basic.rs`](./examples/supervisor_basic.rs) - Supervision patterns

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

The runtime integrates with the AirsSys ecosystem through the Operating System Layer (OSL):
- FileSystem operations through dedicated actors
- Process management with lifecycle tracking  
- Network operations with connection handling

See the complete working integration: [`examples/osl_integration_example.rs`](./examples/osl_integration_example.rs)

### Future WASM Integration

Planned integration with `airssys-wasm` for WebAssembly component hosting. See [Future Use Cases](./docs/src/explanation/future-use-cases.md) for conceptual designs.

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

## 🚀 Getting Started

### Prerequisites
- Rust 2021 Edition or later
- Tokio async runtime

### Working Examples

The best way to get started is to explore the working examples:

```bash
# Basic actor implementation
cargo run --example actor_basic

# Actor lifecycle management  
cargo run --example actor_lifecycle

# Basic supervision patterns
cargo run --example supervisor_basic

# Supervisor restart strategies
cargo run --example supervisor_strategies

# OSL integration (complete system)
cargo run --example osl_integration_example
```

All examples are fully documented and demonstrate real, working patterns. See the [`examples/`](./examples/) directory for complete source code.

## 🔌 OSL Integration

`airssys-rt` now includes complete integration with the Operating System Layer (OSL) through a supervisor hierarchy managing FileSystem, Process, and Network actors with broker-based communication.

### Architecture

```text
┌─────────────────────────────────────┐
│     InMemoryMessageBroker          │
│         (OSLMessage)                │
└─────────────────────────────────────┘
             ↑           ↑
             │           │
   ┌─────────┴───────────┴─────────┐
   │     OSLSupervisor<M, B>       │
   │    (RestForOne Strategy)      │
   └───────────────────────────────┘
             ↓           ↓
   ┌─────────┴───────────┴─────────┐
   │  FileSystem │ Process │ Network│
   │   Actor     │  Actor  │  Actor │
   └─────────────┴─────────┴────────┘
```

### Usage Example

```rust
use airssys_rt::broker::{InMemoryMessageBroker, MessageBroker};
use airssys_rt::osl::OSLSupervisor;
use airssys_rt::osl::supervisor::OSLMessage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create shared message broker
    let broker = InMemoryMessageBroker::<OSLMessage>::new();
    
    // Create and start OSL supervisor
    let supervisor = OSLSupervisor::new(broker.clone());
    supervisor.start().await?;
    
    // Actors are now running and can handle messages
    // See examples/osl_integration_example.rs for complete demo
    
    Ok(())
}
```

### Key Features

- **Broker Dependency Injection**: All actors share a single message broker (ADR-RT-009)
- **RestForOne Strategy**: Failed actors restart dependent children
- **Generic Architecture**: `OSLSupervisor<M, B>` supports custom message and broker types
- **Type-Safe Messaging**: `OSLMessage` enum for unified actor communication
- **Actor Addresses**: Named addresses (osl-filesystem, osl-process, osl-network)

### Available Examples

```bash
# Run OSL integration example
cargo run --example osl_integration_example

# Run integration tests
cargo test --package airssys-rt --test supervisor_hierarchy_tests
```

### Implementation Status

✅ **Complete** - OSL Integration with Broker Injection
- FileSystemActor with broker injection (c1f1be0)
- ProcessActor with broker injection (811d966)
- NetworkActor with broker injection (df0c8b4)
- OSLSupervisor generic refactoring (ac910d4)
- Example application (5c8d0be)
- Integration tests (007a48c)

**Quality Metrics:**
- 336 library tests passing
- 9 integration tests passing
- Zero compilation warnings
- Full workspace standards compliance

## ⚠️ Important Notes

> [!IMPORTANT]
> 
> **`airssys-rt` is NOT a BEAM replacement**. This project implements actor model patterns inspired by Erlang/OTP but optimized for Rust system programming. The "processes" are virtual processes managed in-memory, not OS processes.

> [!NOTE]
>
> **Current Status**: Active development and architecture phase. Not ready for production use.

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
