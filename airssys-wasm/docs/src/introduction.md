# Introduction

airssys-wasm is a planned WebAssembly component framework designed for hot deployment and component composition within the AirsSys ecosystem.

## What is airssys-wasm?

airssys-wasm is a planned component framework that will enable hot deployment of WebAssembly components without system restarts. The framework is designed to provide capability-based security and support multiple programming languages through the WebAssembly Component Model.

### Problem Statement

Current software deployment approaches have several identified limitations:
- Most systems require restart for updates
- Plugin systems often trade security for functionality  
- Systems typically support single programming languages
- Version management and rollbacks are complex
- Components may interfere with each other

### Planned Solution Approach

airssys-wasm is designed to address these challenges through:

1. **Hot Deployment**: Deploy and update components without restarting the host system
2. **Capability-Based Security**: Components run in controlled sandboxes with explicit permissions
3. **Multi-Domain Support**: Framework designed to work across different application domains
4. **Language Support**: Compatible with any WASM-capable programming language
5. **Component Composition**: Enable component orchestration and pipeline construction

## Core Concepts

### Components
A **component** is a planned self-contained unit of functionality compiled to WebAssembly. Components are designed to be:
- AI agents and MCP tools
- Microservices and web APIs
- Data processors and transformers
- IoT device controllers
- Game mods and extensions
- General-purpose application components

### Planned Hot Deployment
Similar to deploying smart contracts to a blockchain, the planned system will allow deploying components to airssys-wasm without stopping the host system:
```bash
# Planned deployment commands (future implementation)
airssys-wasm deploy my-component.wasm
airssys-wasm update my-component@v2.0.0 --strategy blue-green
airssys-wasm rollback my-component --to-version v1.5.0
```

## Core Concepts

### Components
A component is a self-contained unit of functionality compiled to WebAssembly. Components support various use cases including:
- AI agents and MCP tools
- Microservices and web APIs
- Data processors and transformers
- IoT device controllers
- Game modifications and extensions

### Hot Deployment
Components can be deployed to airssys-wasm without stopping the host system:
```bash
# Deploy new component
airssys-wasm deploy my-component.wasm

# Update existing component
airssys-wasm update my-component@v2.0.0 --strategy blue-green

# Rollback to previous version
airssys-wasm rollback my-component --to-version v1.5.0
```

### Capability-Based Security
Components will declare required permissions, enforced by the framework:
```rust
// Planned capability system (future implementation)
#[component_capabilities]
pub fn required_capabilities() -> Vec<Capability> {
    vec![
        Capability::FileRead("/data".into()),
        Capability::NetworkOutbound("api.example.com".into()),
    ]
}
```

### Component Composition
Complex systems are planned to be built by connecting components:
```rust
// Planned composition API (future implementation)
let pipeline = ComponentPipeline::builder()
    .add_component("input", data_source)
    .add_component("processor", ai_model)
    .add_component("output", result_handler)
    .connect("input.output", "processor.input")
    .connect("processor.output", "output.input")
    .build()?;
```

## Planned Framework Benefits

### Development
- SDK with derive macros for streamlined development (via airssys-wasm-component)
- Support for multiple WASM-compatible programming languages
- Fast build, test, and deploy cycles
- Visual component composition tools (planned)

### Operations
- Component updates without system downtime (planned)
- Version rollback capabilities (planned)
- Sandboxed component execution (planned)
- Resource management and monitoring (planned)

### Architecture
- Modular system design through component separation (planned)
- Controlled inter-component communication (planned)
- Scalable deployment patterns (planned)
- Integration with AirsSys ecosystem components (planned)

## Technology Foundation

The framework will be built on established technologies:
- **Wasmtime**: WebAssembly runtime with Component Model support
- **WebAssembly Component Model**: Standard for component composition
- **WIT**: Language-agnostic interface definitions
- **WASI Preview 2**: Standardized system interface

## Implementation Roadmap

To begin working with airssys-wasm:

1. **Architecture Overview**: Read the [Framework Overview](./architecture/overview.md)
2. **Implementation Planning**: Review the [Implementation Guide](./implementation/getting-started.md)
3. **Research Documentation**: Study the [Research Papers](./researches/)
4. **AirsSys Integration**: Understand dependencies on airssys-osl and airssys-rt

## Project Status

**Current Phase**: Architecture and Planning (15% Complete)
- Architecture design completed and documented
- Technology stack decisions made
- Implementation planning ready for when dependencies are mature
- Next phase scheduled for 2026 Q1 when airssys-osl and airssys-rt foundations are complete

airssys-wasm is currently in the planning and architecture design phase. The framework is designed to provide infrastructure for component-based application development with hot deployment capabilities.