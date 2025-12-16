# AirsSys Components

AirsSys consists of multiple specialized components designed to work independently or together. This section provides detailed documentation for each component.

## Completed Components

These components are production-ready and fully documented:

### [OSL (OS Layer Framework)](osl/index.md)

Secure, cross-platform abstraction over operating system functionality.

**Status**: ✅ Complete  
**Version**: 0.1.0  
**Key Features**:
- Cross-platform OS abstractions
- Built-in security (ACL/RBAC)
- Comprehensive audit logging
- Middleware pipeline
- Helper functions API

[View OSL Documentation →](osl/index.md)

### [RT (Actor Runtime)](rt/index.md)

Lightweight Erlang-Actor model runtime for high-concurrency applications.

**Status**: ✅ Complete  
**Version**: 0.1.0  
**Key Features**:
- Zero-cost actor abstractions
- BEAM-inspired supervision
- High performance (4.7M msgs/sec)
- Fault tolerance
- Message broker

[View RT Documentation →](rt/index.md)

### [WASM (Component Framework)](wasm/index.md)

WebAssembly Component Framework for pluggable systems.

**Status**: ⏳ In Development (Phase 6)  
**Version**: 0.1.0  
**Key Features**:
- ComponentActor pattern (dual-trait)
- Hot deployment capability
- O(1) registry (36ns lookup)
- High throughput (6.12M msg/sec)
- Supervisor integration

[View WASM Documentation →](wasm/index.md)

## Components in Development

These components are under active development and not yet covered in this unified documentation:

### airssys-wasm-cli

CLI tools for WASM component management.

**Status**: ⏳ In Development  
**Documentation**: Coming soon

### airssys-osl-macros

Procedural macros for OSL custom executors.

**Status**: ✅ Complete  
**Documentation**: See crate-level docs

### airssys-wasm-component

Procedural macros for WASM component development.

**Status**: ⏳ In Development  
**Documentation**: Coming soon

## Component Comparison

| Feature | OSL | RT | WASM |
|---------|-----|-----|------|
| **Purpose** | OS abstraction | Actor runtime | Component framework |
| **API Style** | Helper functions | Trait-based actors | ComponentActor pattern |
| **Security** | ACL/RBAC | Actor isolation | Capability-based |
| **Performance** | OS-bound | 4.7M msgs/sec | 6.12M msgs/sec |
| **State** | Stateless | Encapsulated | Arc<RwLock<T>> |
| **Async** | Tokio-based | Tokio-based | Tokio-based |
| **Testing** | 419 tests | 381 tests | 945 tests |
| **Benchmarks** | N/A | 12 benchmarks | 28 benchmarks |

## Integration

Components are designed to integrate seamlessly:

```rust
// WASM components run on RT actor system
use airssys_wasm::actor::ComponentActor;
use airssys_rt::system::ActorSystem;

let system = ActorSystem::new();
let component = ComponentActor::new(id, metadata, capabilities).await?;
system.spawn_component(component).await?;

// Components use OSL for secure operations (via WASI)
use airssys_osl::filesystem;
filesystem::read_file("/secure/config.toml").await?;
```

See [Integration Guide](../guides/integration.md) for detailed patterns.

## Choosing Components

### Use OSL when you need:
- Secure file I/O operations
- Process spawning with audit trails
- Network operations with ACL
- Cross-platform OS abstractions

### Use RT when you need:
- High-concurrency actor systems
- Fault-tolerant services
- Message-based architectures
- Supervision trees

### Use WASM when you need:
- Hot-deployable components
- Secure plugin systems
- Multi-language component support
- Runtime component updates

### Use all three when you need:
- Complete pluggable system architecture
- Secure, fault-tolerant components
- High-performance component runtime

## Component Roadmap

### Immediate (Current)
- ✅ OSL core functionality
- ✅ RT actor system
- ⏳ WASM component system (Phase 6)

### Short-term (Q1 2026)
- ⏳ WASM storage implementation
- ⏳ CLI tools
- ⏳ Production deployment guides

### Long-term (2026+)
- Distributed component system
- Advanced WASM capabilities
- Cloud-native features

## Getting Started

Choose your starting point:

1. **New to AirsSys?** Start with [Getting Started](../getting-started.md)
2. **Want secure OS ops?** Go to [OSL Documentation](osl/index.md)
3. **Need concurrency?** Go to [RT Documentation](rt/index.md)
4. **Building plugins?** Go to [WASM Documentation](wasm/index.md)
5. **Integrating multiple?** See [Integration Guide](../guides/integration.md)
