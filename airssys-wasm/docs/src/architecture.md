# Architecture Overview

The airssys-wasm architecture provides a WebAssembly component framework for building modular, secure applications with runtime component management capabilities.

## Framework Vision

airssys-wasm is designed as a general-purpose component framework with these characteristics:

- **Runtime Component Management**: Load and update components during runtime
- **Cross-Platform**: Components run consistently across Linux, macOS, and Windows
- **Language-Agnostic**: Support for any WASM-compatible language (Rust, C++, Go, Python, JavaScript)
- **Security-First**: Capability-based access control with sandbox isolation

## System Architecture

The framework consists of integrated layers working together:

```
┌─────────────────────────────────────────────────────────────┐
│                    AirsSys Ecosystem                        │
│  ┌──────────────────────────────────────────────────────┐   │
│  │  airssys-osl: Secure system access primitives       │   │
│  │  airssys-rt: Actor-based component hosting          │   │
│  │  airssys-wasm: Component framework (this library)   │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│              Component Runtime Infrastructure               │
│  ┌──────────────┐ ┌──────────────┐ ┌───────────────────────┐│
│  │ Component    │ │ Security     │ │ WIT Interface         ││
│  │ Loading &    │ │ & Capability │ │ System                ││
│  │ Execution    │ │ Enforcement  │ │                       ││
│  └──────────────┘ └──────────────┘ └───────────────────────┘│
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│                  WebAssembly Foundation                     │
│  ┌──────────────┐ ┌──────────────┐ ┌───────────────────────┐│
│  │ Wasmtime     │ │ Component    │ │ WIT Bindings          ││
│  │ Runtime      │ │ Model        │ │ (wit-bindgen)         ││
│  └──────────────┘ └──────────────┘ └───────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### WIT Interface System

The framework provides a comprehensive interface system:

**Core Package (`airssys:core@1.0.0`):**
- Foundation types (ComponentId, ResourceLimits, etc.)
- Capability definitions (FileRead, FileWrite, Network, etc.)
- Component lifecycle (init, execute, shutdown)
- Host services (logging, messaging, timing)

**Extension Packages:**
- `airssys:ext-filesystem@1.0.0` - 36 filesystem operations
- `airssys:ext-network@1.0.0` - 32 network operations
- `airssys:ext-process@1.0.0` - 32 process operations

**Statistics:**
- 16 WIT interface files
- 2,214 lines of interface definitions
- 82 type definitions
- 115 operations total

See [WIT System Architecture](./reference/wit-system-architecture.md) for complete specifications.

### Permission System

Components declare required capabilities in Component.toml manifests:

```toml
[component]
name = "data-processor"
version = "1.0.0"

[resources.memory]
max_memory_bytes = 2097152  # 2MB

[permissions.filesystem]
readable_paths = [
    "/data/input/**",
]
writable_paths = [
    "/data/output/**",
]

[permissions.network]
allowed_hosts = [
    "api.example.com",
]
```

**Permission Categories:**
- **Filesystem**: read/write/execute permissions with path patterns
- **Network**: inbound/outbound with host restrictions  
- **Process**: spawn/signal permissions
- **Storage**: key-value storage access
- **Messaging**: inter-component communication

### Build System

The framework uses a two-stage build approach:

1. **WIT Validation** (`wasm-tools`): Validates interface definitions
2. **Binding Generation** (`wit-bindgen`): Generates language bindings

**Build Artifacts:**
- Auto-generated Rust bindings (~154KB, 2,794 lines)
- Type-safe structs for all WIT records
- Trait definitions for component interfaces
- Host function import stubs

## Design Principles

### Capability-Based Security

Components operate under deny-by-default security:
- Explicit permission grants required for all system access
- Fine-grained control over filesystem, network, and process operations
- Runtime permission enforcement
- Complete audit trail of component operations

### Component Isolation

Each component runs in isolated execution environment:
- Memory sandboxing via WebAssembly
- Configurable resource limits (memory, CPU, I/O)
- No shared state between components
- Crash isolation (component failures don't affect host)

### Language Agnosticism

WIT interfaces enable multi-language support:
- Interface definitions independent of implementation language
- Automatic binding generation for supported languages
- Type safety across language boundaries
- Consistent behavior regardless of source language

### Multi-Domain Support

The framework supports diverse application domains:

**AI and Machine Learning:**
- AI agent systems with secure isolation
- ML pipeline components
- Model serving with resource limits

**Web and Microservices:**
- API gateway components
- Request handlers and middleware
- Service mesh integration

**Enterprise Systems:**
- Business rule engines
- Data processing pipelines
- Integration adapters

**IoT and Edge:**
- Device controllers
- Sensor data processors
- Edge computing functions

## Technology Stack

### WebAssembly Component Model

The framework builds on the WebAssembly Component Model specification:
- Type-safe component composition
- Resource management and ownership
- Interface inheritance and composition
- Standard system interface (WASI)

### Wasmtime Runtime

Wasmtime provides the execution environment:
- Ahead-of-time (AOT) and just-in-time (JIT) compilation
- Memory sandboxing and isolation
- Resource limiting (memory, CPU, fuel)
- Async execution support

### WIT Interface Types

WIT defines language-agnostic interfaces:
- Type definitions (records, variants, enums)
- Function signatures with parameters and returns
- Resource types and handles
- Package dependencies

### Integration with AirsSys

**airssys-osl Integration:**
- Components access filesystem through OSL abstractions
- Network operations delegated to OSL layer
- Process management via OSL primitives
- Security context propagation

**airssys-rt Integration:**
- Components hosted as actors in runtime system
- Message passing for inter-component communication
- Supervision trees for fault tolerance
- Lifecycle management through actor model

## Security Model

### Sandbox Isolation

Components execute in WebAssembly sandbox:
- Linear memory isolation (no access to host memory)
- Function-level boundaries (only exposed functions callable)
- No direct system calls (all via host interfaces)
- Stack isolation and execution limits

### Permission Enforcement

Runtime enforces declared permissions:
- Filesystem access checked against path patterns
- Network connections validated against host allowlist
- Process operations require explicit grants
- Resource usage monitored and limited

### Audit Logging

All component operations are logged:
- System access attempts (granted and denied)
- Resource usage metrics
- Component lifecycle events
- Security-relevant operations

## Development Workflow

### Component Development

1. **Define WIT Interface**: Specify component API
2. **Implement Component**: Write business logic in chosen language
3. **Declare Permissions**: Create Component.toml manifest
4. **Build to WASM**: Compile with `cargo build --target wasm32-wasip1`
5. **Test**: Validate with framework test harness

### Host Application Development

1. **Add Dependency**: Include `airssys-wasm` in Cargo.toml
2. **Configure Runtime**: Set up security and resource limits
3. **Load Components**: Load WASM files at runtime
4. **Execute**: Call component functions through type-safe interfaces
5. **Monitor**: Track health, performance, and resource usage

## Performance Characteristics

### Resource Overhead

**Memory:**
- Base runtime: ~512KB per component
- Configurable limits: 512KB - 4MB per component
- Shared code across component instances

**Startup Time:**
- Component loading: Target < 10ms
- Instantiation: Target < 5ms
- First execution: Depends on JIT/AOT compilation

**Execution:**
- Near-native performance after JIT warmup
- Function call overhead: ~1-10μs
- Memory access: Native speed within sandbox

### Optimization Strategies

**Ahead-of-Time (AOT) Compilation:**
- Pre-compile components for faster instantiation
- Eliminates JIT warmup overhead
- Platform-specific optimizations

**Code Caching:**
- Cached compiled code across instances
- Reduced memory footprint for multiple instances
- Faster component loading

## Documentation Resources

**API Reference:**
- [WIT Interfaces](./api/wit-interfaces.md) - Complete interface specifications
- [Component.toml Spec](./reference/component-toml-spec.md) - Manifest format

**Guides:**
- [Getting Started](./guides/getting-started.md) - Setup and build instructions
- [Troubleshooting](./guides/troubleshooting.md) - Common issues and solutions

**Research:**
- [WIT Ecosystem Research](./researches/wit-ecosystem-research.md) - WIT system research
- See `researches/` directory for additional technical analysis

## Summary

airssys-wasm provides a complete WebAssembly component framework with:
- ✅ Comprehensive WIT interface system (115 operations)
- ✅ Capability-based security model
- ✅ Multi-language support via Component Model
- ✅ Integration with AirsSys ecosystem (osl, rt)
- ✅ Cross-platform execution (Linux, macOS, Windows)

The framework enables building modular, secure applications with runtime component management across diverse domains.
