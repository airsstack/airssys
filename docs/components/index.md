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

## Components in Development

These components are under active development and not yet covered in this unified documentation:

### airssys-wasm

WebAssembly Component Framework for pluggable systems.

**Status**: ⏳ In Development  
**Documentation**: See individual mdbook docs in `airssys-wasm/docs/`

### airssys-wasm-cli

CLI tools for WASM component management.

**Status**: ⏳ In Development  
**Documentation**: Coming soon

### airssys-osl-macros

Procedural macros for OSL custom executors.

**Status**: ⏳ In Development  
**Documentation**: See crate-level docs

### airssys-wasm-component

Procedural macros for WASM component development.

**Status**: ⏳ In Development  
**Documentation**: Coming soon

## Component Comparison

| Feature | OSL | RT |
|---------|-----|-----|
| **Purpose** | OS abstraction & security | Actor concurrency |
| **API Style** | Helper functions | Trait-based actors |
| **Security** | ACL/RBAC policies | Actor isolation |
| **Performance** | OS-bound operations | 4.7M msgs/sec |
| **State** | Stateless operations | Encapsulated state |
| **Async** | Tokio-based | Tokio-based |
| **Testing** | 60+ tests | 336+ tests |

## Integration

Components are designed to integrate seamlessly:

```rust
// RT supervises OSL operations
use airssys_rt::supervisor::OSLSupervisor;

let supervisor = OSLSupervisor::new(broker);
supervisor.start().await?;
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

### Use both when you need:
- Secure concurrent operations
- Fault-tolerant system programming
- Complete AirsStack integration

## Component Roadmap

### Immediate (Current)
- ✅ OSL core functionality
- ✅ RT actor system
- ✅ OSL-RT integration

### Short-term (Q1 2026)
- ⏳ WASM component system
- ⏳ CLI tools
- ⏳ Macro enhancements

### Long-term (2026+)
- Distributed actor system
- Advanced WASM capabilities
- Cloud-native features

## Getting Started

Choose your starting point:

1. **New to AirsSys?** Start with [Getting Started](../getting-started.md)
2. **Want secure OS ops?** Go to [OSL Documentation](osl/index.md)
3. **Need concurrency?** Go to [RT Documentation](rt/index.md)
4. **Integrating both?** See [Integration Guide](../guides/integration.md)
