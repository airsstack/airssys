# Introduction

airssys-wasm is a WASM Component Framework for Pluggable Systems, part of the AirsSys ecosystem.

## What is airssys-wasm?

airssys-wasm provides WebAssembly component integration with type-safe interfaces and capability-based security. The framework supports multiple programming languages through the WebAssembly Component Model.

### Problem Statement

Current software deployment approaches face several challenges:
- Most systems require restarts for updates
- Plugin systems often trade security for functionality
- Systems typically support single programming languages
- Version management and rollbacks can be complex
- Components may interfere with each other

### Solution Approach

airssys-wasm addresses these challenges through:

1. **Component Model Foundation**: Type-safe component boundaries using WIT (WASM Interface Types)
2. **Capability-Based Security**: Component permission declarations via Component.toml manifests
3. **General-Purpose Design**: Framework works across different application domains
4. **Language Agnostic**: Compatible with any WASM-capable programming language

## Core Concepts

### Components

A **component** is a self-contained unit of functionality compiled to WebAssembly. Components support various use cases:
- AI agents and MCP tools
- Microservices and web APIs
- Data processors and transformers
- IoT device controllers
- Game modifications and extensions
- System utilities and tools

### Component Interfaces

Components interact with the host system through WIT (WebAssembly Interface Types) interfaces:

```wit
// Component lifecycle interface
interface component-lifecycle {
    // Initialize component with configuration
    init: func(config: component-config) -> result<_, component-error>;
    
    // Execute component with input data
    execute: func(input: component-input) -> result<component-output, execution-error>;
    
    // Shutdown and cleanup
    shutdown: func() -> result<_, component-error>;
}
```

### Permission System

Components declare required permissions in Component.toml:

```toml
[component]
name = "file-processor"
version = "0.1.0"

[permissions]
filesystem = [
    { action = "read", path = "/data/**" },
    { action = "write", path = "/output/**" }
]
```

### Extension Interfaces

The framework provides three extension domains:

1. **Filesystem Operations** (36 operations)
   - File read/write/delete operations
   - Directory management
   - Metadata queries
   - Path manipulation

2. **Network Operations** (32 operations)
   - Socket creation and management
   - TCP/UDP communication
   - DNS resolution
   - Connection handling

3. **Process Operations** (32 operations)
   - Process spawning and management
   - Signal handling
   - Standard I/O redirection
   - Exit code handling

## Framework Architecture

### WIT Interface System

The framework uses a layered WIT interface architecture:

```
Extension Tier (100 operations)
├── Filesystem (36 ops) - ext-filesystem@1.0.0
├── Network (32 ops)    - ext-network@1.0.0
└── Process (32 ops)    - ext-process@1.0.0
        ↓
Core Package (15 operations)
└── airssys:core@1.0.0
    ├── types.wit (13 foundation types)
    ├── capabilities.wit (10 permission types)
    ├── component-lifecycle.wit (7 functions)
    └── host-services.wit (8 functions)
```

**Total Interface Coverage:**
- 16 WIT files
- 2,214 lines of interface definitions
- 82 types
- 115 operations

See [WIT System Architecture](./reference/wit-system-architecture.md) for complete reference.

## Technology Foundation

The framework is built on established WebAssembly technologies:

- **Wasmtime**: WebAssembly runtime with Component Model support
- **WebAssembly Component Model**: Standard for component composition and interfaces
- **WIT (WASM Interface Types)**: Language-agnostic interface definitions
- **wit-bindgen**: Automatic binding generation for Rust and other languages
- **Component.toml**: Manifest format for component metadata and permissions

## Documentation

**Available Documentation**:
- [WIT Interface Specifications](./api/wit-interfaces.md) - Complete API definitions (115 operations)
- [Component.toml Specification](./reference/component-toml-spec.md) - Manifest format specification
- [WIT System Architecture](./reference/wit-system-architecture.md) - Complete technical reference
- [Getting Started](./guides/getting-started.md) - Setup and build instructions
- [Troubleshooting](./guides/troubleshooting.md) - Common issues and solutions

**For Framework Developers**:
- See `airssys-wasm/tests/` for WIT validation tests
- See `airssys-wasm/src/core/` for framework internals
- See memory bank documentation for architecture decisions

## The AirsSys Ecosystem

airssys-wasm integrates with other AirsSys components:

### airssys-osl (OS Layer)
Provides secure system access primitives for components:
- Filesystem operations with security context
- Network access with capability enforcement
- Process management with sandboxing

### airssys-rt (Actor Runtime)
Provides actor-based component hosting:
- Component lifecycle management
- Message passing between components
- Supervision trees for fault tolerance

### airssys-wasm (This Framework)
Provides WASM component infrastructure:
- Component loading and execution
- WIT interface system
- Permission and capability management
- Cross-platform component support

## Next Steps

To start using airssys-wasm:

1. **Installation**: Follow the [Getting Started Guide](./guides/getting-started.md)
2. **Learn WIT**: Review the [WIT Interface Reference](./api/wit-interfaces.md)
3. **Configure Permissions**: Study the [Component.toml Specification](./reference/component-toml-spec.md)
4. **Explore Examples**: Check the research documentation for patterns and examples
