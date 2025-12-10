# Welcome to AirsSys

**AirsSys** is a comprehensive collection of system programming components designed for the AirsStack ecosystem. It provides secure, modular, and high-performance tools for building robust concurrent applications with strong security guarantees.

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-0.1.0-green.svg)](https://github.com/airsstack/airssys)

## Overview

AirsSys consists of multiple specialized components that work together to provide a complete system programming solution:

### 🛡️ [OSL (OS Layer Framework)](components/osl/index.md)

Secure, cross-platform abstraction over operating system functionality with comprehensive audit trails and security policy enforcement.

**Key Features:**
- Cross-platform OS abstraction (filesystem, process, network)
- Built-in ACL and RBAC security policies
- Comprehensive activity logging and audit trails
- Middleware pipeline for extensibility
- Helper functions for common operations

**Use Cases:**
- Secure application development requiring system resources
- Enterprise system administration with compliance requirements
- Foundation for higher-level AirsStack components

### ⚡ [RT (Actor Runtime)](components/rt/index.md)

Lightweight Erlang-Actor model runtime system for high-concurrency applications with BEAM-inspired supervision and fault tolerance.

**Key Features:**
- Zero-cost actor abstraction with compile-time type safety
- BEAM-inspired supervision trees (OneForOne, OneForAll, RestForOne)
- High performance: ~625ns actor spawn, 4.7M msgs/sec throughput
- Broker-based message routing with backpressure control
- Comprehensive monitoring and observability

**Use Cases:**
- High-concurrency servers requiring fault tolerance
- Event-driven architectures with complex state management
- System programming with reliable process supervision
- Microservice coordination

## Component Status

| Component | Status | Documentation |
|-----------|--------|---------------|
| **airssys-osl** | ✅ Complete | [View Docs](components/osl/index.md) |
| **airssys-rt** | ✅ Complete | [View Docs](components/rt/index.md) |
| **airssys-wasm** | ⏳ In Development | *Not yet migrated* |
| **airssys-wasm-cli** | ⏳ In Development | *Not yet migrated* |
| **airssys-osl-macros** | ⏳ In Development | *Not yet migrated* |
| **airssys-wasm-component** | ⏳ In Development | *Not yet migrated* |

!!! note "Documentation Scope"
    This unified documentation covers **completed components only** (OSL and RT). Components still in active development maintain their individual mdbook documentation until they reach stable status.

## Quick Start

### OSL Quick Start

```rust
use airssys_osl::helpers::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Filesystem operations with built-in security
    let data = b"Hello, World!".to_vec();
    write_file("/tmp/test.txt", data, "admin").await?;
    let content = read_file("/tmp/test.txt", "admin").await?;
    
    // Process operations
    let output = spawn_process("echo", vec!["Hello!".to_string()], "admin").await?;
    
    // Network operations
    let listener = network_listen("127.0.0.1:0", "admin").await?;
    
    Ok(())
}
```

### RT Quick Start

```rust
use airssys_rt::prelude::*;
use async_trait::async_trait;

// Define message type
#[derive(Debug, Clone)]
enum CounterMsg {
    Increment,
    GetCount(tokio::sync::oneshot::Sender<u64>),
}

impl Message for CounterMsg {
    const MESSAGE_TYPE: &'static str = "counter";
}

// Define actor
struct CounterActor {
    count: u64,
}

// Implement Actor trait
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

## Design Philosophy

### Security by Default

All AirsSys components implement a **deny-by-default** security model. Operations are only permitted when explicitly allowed by security policies, with comprehensive audit trails for compliance.

### Zero-Cost Abstractions

AirsSys leverages Rust's zero-cost abstractions to provide high-level APIs without runtime overhead. Generic constraints and compile-time monomorphization eliminate the need for dynamic dispatch in hot paths.

### Modular Architecture

Components are designed to work independently or together. Use OSL for secure system operations, RT for actor-based concurrency, or combine them for complete system programming solutions.

### Fault Tolerance

Following Erlang/OTP principles, AirsSys embraces the "let it crash" philosophy. Supervisor trees monitor process health and automatically restart failed components with configurable strategies.

## Integration

AirsSys components are designed to integrate seamlessly:

```rust
// OSL actors managed by RT supervisor
use airssys_rt::supervisor::OSLSupervisor;
use airssys_osl::operations::filesystem::FileReadOperation;

// RT manages OSL operations with fault tolerance
let supervisor = OSLSupervisor::new(broker.clone());
supervisor.start().await?;

// Actors handle OS operations with supervision
// See integration guides for complete examples
```

See [Integration Guide](guides/integration.md) for detailed patterns and examples.

## Getting Started

Choose your path based on your needs:

1. **[Getting Started Guide](getting-started.md)** - Installation and basic setup
2. **[Architecture Overview](architecture.md)** - System design and principles
3. **[OSL Documentation](components/osl/index.md)** - OS abstraction layer
4. **[RT Documentation](components/rt/index.md)** - Actor runtime system
5. **[Examples](examples/index.md)** - Practical usage patterns

## Resources

- **Repository**: [github.com/airsstack/airssys](https://github.com/airsstack/airssys)
- **Issues**: [Report bugs and request features](https://github.com/airsstack/airssys/issues)
- **API Documentation**: Run `cargo doc --open` in the repository
- **Contributing**: See [Contributing Guide](contributing.md)

## License

AirsSys is dual-licensed under:

- [Apache License 2.0](https://github.com/airsstack/airssys/blob/main/LICENSE-APACHE)
- [MIT License](https://github.com/airsstack/airssys/blob/main/LICENSE-MIT)

You may choose either license at your option.

---

**Current Version**: 0.1.0  
**Last Updated**: December 2025
