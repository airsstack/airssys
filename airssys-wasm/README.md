# airssys-wasm: WASM Component Framework for Pluggable Systems

## Project Overview

**airssys-wasm** is a planned WASM Component Framework for Pluggable Systems designed to provide runtime deployment capabilities for modular applications. The framework is designed to enable component-based development with runtime updates and capability-based security.

## Planned Architecture

### Component Framework
The framework is designed to support various component types including:
- AI agents and MCP tools
- Microservices and web APIs
- Data processing pipelines
- IoT device controllers
- Game modifications and extensions
- Browser extensions
- General-purpose application components

### Runtime Deployment System
The planned runtime deployment capabilities include:
- Component updates without system restart using Blue-Green and Canary deployment strategies
- Version rollback capabilities
- A/B testing with traffic splitting
- Dynamic system scaling through component addition/removal

### Security Model
Components will operate within controlled security boundaries:
- Fine-grained permissions for file access, network, and system calls
- Memory isolation between components
- Audit logging of component activities
- Configurable security policy enforcement

### Language Support
Components can be written in WASM-compatible languages:
- Rust (primary development language)
- C/C++ (native WASM support)
- Go (TinyGo for WASM compilation)
- Python (via WASM compilation)
- JavaScript/TypeScript (via Component Model)
- Other languages that compile to WebAssembly

### Component Composition
Complex systems are planned to be built through component orchestration:
- Pipeline construction tools for component chaining
- Dependency management with automatic resolution
- Error handling with rollback and recovery capabilities
- Performance optimization through parallel execution

## Strategic Architecture

## Planned Framework Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Developer Experience                     │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │
│  │ SDK & Macros│ │ WIT Bindings│ │ Visual Composition      │ │
│  └─────────────┘ └─────────────┘ └─────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    Component Framework                      │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐ │
│  │ Hot Deployment  │ │ Composition     │ │ Version Manager │ │
│  │ Engine          │ │ Engine          │ │                 │ │
│  └─────────────────┘ └─────────────────┘ └─────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    Core Runtime System                      │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐ │
│  │ Capability      │ │ WASM Runtime    │ │ Resource        │ │
│  │ Manager         │ │ (Wasmtime)      │ │ Manager         │ │
│  └─────────────────┘ └─────────────────┘ └─────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Planned Component Interface

```rust
// Planned component implementation example
use airssys_wasm_sdk::prelude::*;

#[component]
pub struct MyComponent {
    state: ComponentState,
}

#[component_impl]
impl MyComponent {
    #[component_init]
    pub fn new(config: ComponentConfig) -> Result<Self, ComponentError> {
        Ok(Self { state: ComponentState::new(config)? })
    }
    
    #[component_execute]
    pub fn process(&mut self, input: ComponentInput) -> ComponentResult {
        // Component logic implementation
        Ok(ComponentOutput::new(processed_data))
    }
    
    #[component_capabilities]
    pub fn required_capabilities() -> Vec<Capability> {
        ]
    }
}
```

## Planned Deployment Operations

### Component Deployment (Planned)
```bash
# Planned CLI operations for future implementation
airssys-wasm deploy my-component.wasm --config config.json
airssys-wasm update my-component@v2.0.0 --strategy blue-green
airssys-wasm rollback my-component --to-version v1.5.0
airssys-wasm monitor my-component --metrics
```

### Component Composition (Planned)
```rust
// Planned pipeline construction API
let pipeline = ComponentPipeline::builder()
    .add_component("input", data_source_component)
    .add_component("processor", ai_model_component)
    .add_component("output", result_handler_component)
    .connect("input.output", "processor.input")
    .connect("processor.output", "output.input")
    .with_error_strategy(ErrorStrategy::Rollback)
    .build()?;

// Deploy pipeline
pipeline.deploy().await?;
```

## Technology Foundation

The framework will build on established technologies:
- **Wasmtime**: WebAssembly runtime with Component Model support
- **WebAssembly Component Model**: Standard for component composition
- **WIT (WebAssembly Interface Types)**: Language-agnostic interface definitions
- **WASI Preview 2**: Standardized system interface for capabilities
- **Capability-based Security**: Security model for controlled resource access

## Planned Use Cases

### AI and Machine Learning
- AI agent systems with secure component isolation
- MCP tools as WebAssembly components
- ML pipelines for data processing and model inference
- Model serving with deployment strategy support

### Web and Microservices
- API services with hot deployment capabilities
- Serverless functions with fast startup times
- Plugin systems for web applications
- Edge computing functions with instant deployment

### Enterprise and Systems
- Business logic with real-time updates
- Data processing pipelines with component updates
- Integration services for system connectivity
- Monitoring and alerting components

### Gaming and Entertainment
- Game modification systems
- Content processing and transformation
- User-generated content execution
- Real-time feature updates

## Framework Comparison Analysis

### Compared to Container Systems
- Planned faster deployment times (target: seconds vs minutes)
- Designed for lower resource overhead compared to containers
- Capability-based security design vs namespace isolation
- Intended reduced operational complexity

### Compared to Serverless Platforms
- Designed to eliminate cold start delays
- Planned component state preservation capabilities
- Built on open standards without vendor lock-in
- Resource efficiency design without per-invocation billing

### Compared to Plugin Systems
- Planned component updates without system restart
- Memory safety through WebAssembly sandboxing
- Multi-language support via WASM compilation
- Designed with built-in version management

## Development Roadmap

### Prerequisites (Implementation Dependencies)
- airssys-osl (for system access abstraction)
- airssys-rt (for actor-based component hosting)

### Planned Development Workflow
```bash
# Planned CLI commands for future implementation
airssys-wasm init my-first-component
airssys-wasm build
airssys-wasm test
airssys-wasm deploy my-first-component.wasm
```

## Project Status

**Current Phase**: WIT Interface System Implementation (67% Complete)
- **Architecture Design**: Complete architectural framework designed and documented
- **Technology Stack**: Core technology decisions made (Wasmtime, Component Model, WIT)
- **WIT Interface System**: Complete (4 packages, 13 interfaces, 115 operations, 82 types)
- **Build System**: Integrated wit-bindgen CLI with automatic Rust binding generation
- **Implementation Status**: Core abstractions complete, runtime system operational
- **Next Phase**: Permission System Integration and Component.toml parsing

### Recent Milestones

#### Phase 3 Complete: Build System Integration (Nov 2025)
- ✅ Automatic Rust binding generation from WIT definitions (2,794 lines generated)
- ✅ Two-stage validation (wasm-tools → wit-bindgen)
- ✅ Incremental build optimization (~2s for incremental builds)
- ✅ Error handling with clear, actionable messages
- ✅ Complete world definition for airssys-component

#### Phase 2 Complete: WIT Implementation Foundation (Oct 2025)
- ✅ Core package: airssys:core@1.0.0 (4 interfaces, 394 lines)
- ✅ Extension packages: filesystem, network, process (9 interfaces, 1,233 lines)
- ✅ 100% validation passing (wasm-tools 1.240.0)
- ✅ Zero type duplication through `use` statements
- ✅ Acyclic dependency graph

#### Phase 1 Complete: Research and Foundation (Oct 2025)
- ✅ WIT ecosystem thoroughly researched
- ✅ 7-package structure fully designed
- ✅ Build system integration strategy proven

## Build System

### Prerequisites

To build airssys-wasm, you need:

- **Rust 1.80+**: `rustup update`
- **wasm-tools 1.240.0**: `cargo install wasm-tools --version 1.240.0`
- **wit-bindgen 0.47.0**: `cargo install wit-bindgen-cli --version 0.47.0`

### Build Process

The build system uses a two-stage approach:

1. **Stage 1: WIT Validation** - `wasm-tools component wit` validates all WIT packages
2. **Stage 2: Binding Generation** - `wit-bindgen rust` generates Rust bindings from WIT

Generated bindings are output to `src/generated/` and automatically included in the library.

### Build Commands

```bash
# Standard build (automatically validates WIT and generates bindings)
cargo build

# Clean build (useful after WIT changes)
cargo clean && cargo build

# Verbose build output (shows WIT validation details)
AIRSSYS_BUILD_VERBOSE=1 cargo build

# Run tests (225 passing)
cargo test

# Build for WASM target (future)
cargo build --target wasm32-wasip1 --release
```

### Build Performance

- **Clean build**: ~10s (includes dependency compilation and binding generation)
- **Incremental build (no WIT changes)**: ~2s
- **Incremental build (WIT changes)**: ~4s (re-validates and regenerates bindings)

### Generated Code

The build system generates:
- **2,794 lines** of Rust bindings from WIT definitions
- Type-safe Rust structs for all WIT records and variants
- Trait definitions for component lifecycle interfaces
- Import stubs for host services (logging, messaging, timing)

### Troubleshooting

**Error: "wasm-tools: command not found"**
```bash
cargo install wasm-tools --version 1.240.0
```

**Error: "wit-bindgen: command not found"**
```bash
cargo install wit-bindgen-cli --version 0.47.0
```

**Error: "WIT validation failed"**
- Check WIT syntax in `wit/` directory
- Run `wasm-tools component wit wit/core` to see detailed errors
- Ensure all `use` statements reference defined types

**Error: "Binding generation failed"**
- Verify world definition exists in `wit/core/world.wit`
- Check that all exported/imported interfaces are defined
- Ensure wit-bindgen version is 0.47.0

## Documentation

Comprehensive technical documentation is available in the `docs/` directory:

- [Architecture Guide](docs/src/architecture/) - System architecture and design decisions
- [Implementation Guide](docs/src/implementation/) - Component development and deployment
- [API Reference](docs/src/api/) - Runtime API and interface specifications
- [Research Documentation](docs/src/researches/) - Technical research and analysis

## Contributing

This project is part of the AirsSys ecosystem and follows workspace development standards:

- **Memory Bank System**: All development must follow memory bank documentation and decision tracking
- **Workspace Standards**: 3-layer import organization, Microsoft Rust Guidelines compliance
- **Documentation Requirements**: Comprehensive documentation for all public APIs and architectural decisions
- **Testing Standards**: UI tests, integration tests, and security validation

## License

Licensed under either of
- Apache License, Version 2.0
- MIT License

## Related Projects

### AirsSys Ecosystem
- **airssys-wasm-component** - Procedural macros for simplified WASM component development
- **airssys-osl** - OS Layer Framework for system programming (foundation dependency)
- **airssys-rt** - Erlang-Actor model runtime system (runtime dependency)

### Technology Foundation
- **Bytecode Alliance** - Wasmtime and Component Model standards
- **WebAssembly Community** - WASM and WASI specifications
- **AirsStack Ecosystem** - Integrated system programming components

---

**airssys-wasm: WebAssembly component framework for modular application development.**