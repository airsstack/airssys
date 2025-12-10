# AirsSys - System Programming Components for AirsStack

`AirsSys` is a collection of system programming components designed to facilitate the development of applications within the AirsStack ecosystem. It provides essential tools and libraries for managing system resources, handling low-level operations, and ensuring efficient performance.

`Airssys` is one of the `airsstack` projects, which is designed to manage the OS system programming, `Erlang Actor Model` runtime system, and pluggable system.

This project contains four important components:

- `airssys-osl`
- `airssys-rt`
- `airssys-wasm`
- `airssys-wasm-component`

# AirsSys - System Programming Components for AirsStack

`AirsSys` is a comprehensive collection of system programming components designed for the AirsStack ecosystem. It provides secure, modular, and high-performance tools for building robust concurrent applications with strong security guarantees.

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-0.1.0-green.svg)](https://github.com/airsstack/airssys)

## 📚 Documentation

**[View Complete Documentation →](https://airsstack.github.io/airssys/)**

The unified documentation covers all completed AirsSys components including architecture, guides, API references, and examples.

## Components

This project contains multiple specialized components:

### Completed Components

- **`airssys-osl`** - OS Layer Framework (✅ Complete)
- **`airssys-rt`** - Actor Runtime System (✅ Complete)

### In Development

- **`airssys-wasm`** - WASM Component Framework (⏳ In Development)
- **`airssys-wasm-cli`** - WASM CLI Tools (⏳ In Development)
- **`airssys-osl-macros`** - OSL Procedural Macros (⏳ In Development)
- **`airssys-wasm-component`** - WASM Component Macros (⏳ In Development)

## Quick Start

### OSL - Secure OS Operations

```rust
use airssys_osl::helpers::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Secure file operations with built-in ACL/RBAC
    let data = b"Hello, World!".to_vec();
    write_file("/tmp/test.txt", data, "admin").await?;
    let content = read_file("/tmp/test.txt", "admin").await?;
    Ok(())
}
```

### RT - Actor-Based Concurrency

```rust
use airssys_rt::prelude::*;
use async_trait::async_trait;

#[derive(Debug, Clone)]
enum CounterMsg { Increment }

impl Message for CounterMsg {
    const MESSAGE_TYPE: &'static str = "counter";
}

struct CounterActor { count: u64 }

#[async_trait]
impl Actor for CounterActor {
    type Message = CounterMsg;
    type Error = std::io::Error;
    
    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        msg: Self::Message,
        _ctx: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        self.count += 1;
        Ok(())
    }
}
```

## Documentation Structure

- **[Getting Started](https://airsstack.github.io/airssys/getting-started/)** - Installation and first steps
- **[Components](https://airsstack.github.io/airssys/components/)** - Detailed component documentation
  - [OSL Documentation](https://airsstack.github.io/airssys/components/osl/)
  - [RT Documentation](https://airsstack.github.io/airssys/components/rt/)
- **[Guides](https://airsstack.github.io/airssys/guides/integration/)** - Integration, security, performance
- **[Examples](https://airsstack.github.io/airssys/examples/)** - Practical usage examples
- **[Research](https://airsstack.github.io/airssys/research/)** - Design decisions and analysis

## Component Details

### airssys-osl (OS Layer Framework)

Secure, cross-platform abstraction over operating system functionality with comprehensive audit trails and security policy enforcement.

**Key Features:**
- Cross-platform OS abstractions (filesystem, process, network)
- Built-in ACL and RBAC security policies
- Comprehensive activity logging and audit trails
- Middleware pipeline for extensibility
- Helper functions for common operations

[View OSL Documentation →](https://airsstack.github.io/airssys/components/osl/)

### airssys-rt (Actor Runtime System)

Lightweight Erlang-Actor model runtime for high-concurrency applications with BEAM-inspired supervision and fault tolerance.

**Key Features:**
- Zero-cost actor abstractions with compile-time type safety
- BEAM-inspired supervision trees (OneForOne, OneForAll, RestForOne)
- High performance: ~625ns actor spawn, 4.7M msgs/sec throughput
- Broker-based message routing with backpressure control
- Comprehensive monitoring and observability

[View RT Documentation →](https://airsstack.github.io/airssys/components/rt/)

### airssys-wasm (WASM Component Framework)

WebAssembly Component Framework for pluggable systems (⏳ In Development).

**Note:** WASM components are under active development. See individual mdbook documentation in `airssys-wasm/docs/` for current status.

### Other Components

- **airssys-osl-macros** - Procedural macros for OSL custom executors
- **airssys-wasm-cli** - CLI tools for WASM component management
- **airssys-wasm-component** - Procedural macros for WASM development

## Development

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run component tests
cargo test --package airssys-osl
cargo test --package airssys-rt

# Run with features
cargo test --features macros
```

### Running Examples

```bash
# OSL examples
cargo run --example helper_functions_comprehensive
cargo run --example security_middleware_comprehensive

# RT examples
cargo run --example actor_basic
cargo run --example supervisor_basic
cargo run --example osl_integration_example
```

### Building Documentation

```bash
# Build unified MkDocs documentation
cd site-mkdocs
mkdocs serve  # Local preview at http://localhost:8000

# Build API documentation
cargo doc --open --workspace
```

## Contributing

See [Contributing Guide](https://airsstack.github.io/airssys/contributing/) for development guidelines.

## Resources

- **Documentation**: https://airsstack.github.io/airssys/
- **Repository**: https://github.com/airsstack/airssys
- **Issues**: https://github.com/airsstack/airssys/issues

## License

AirsSys is dual-licensed under:

- [Apache License 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

You may choose either license at your option.

---

**Current Version**: 0.1.0  
**Documentation**: https://airsstack.github.io/airssys/  
**Last Updated**: December 2025