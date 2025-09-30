# Introduction

airssys-wasm is a WebAssembly component framework designed for hot deployment and component composition within the AirsSys ecosystem.

## What is airssys-wasm?

airssys-wasm is a component framework that enables hot deployment of WebAssembly components without system restarts. The framework provides capability-based security and supports multiple programming languages through the WebAssembly Component Model.

### Problem Statement

Current software deployment approaches have several limitations:
- Most systems require restart for updates
- Plugin systems often trade security for functionality  
- Systems typically support single programming languages
- Version management and rollbacks are complex
- Components may interfere with each other

### Solution Approach

airssys-wasm addresses these challenges through:

1. **Hot Deployment**: Deploy and update components without restarting the host system
2. **Capability-Based Security**: Components run in controlled sandboxes with explicit permissions
3. **Multi-Domain Support**: Framework works across different application domains
4. **Language Support**: Compatible with any WASM-capable programming language
5. **Component Composition**: Enable component orchestration and pipeline construction

## Core Concepts

### Components
A **component** is a self-contained unit of functionality compiled to WebAssembly. Components can be:
- AI agents and MCP tools
- Microservices and web APIs
- Data processors and transformers
- IoT device controllers
- Game mods and extensions
- Anything you can imagine!

### Hot Deployment
Just like deploying smart contracts to a blockchain, you can deploy components to airssys-wasm **without stopping the host system**:
```bash
# Deploy new component instantly
airssys-wasm deploy my-component.wasm

# Update with zero downtime
airssys-wasm update my-component@v2.0.0 --strategy blue-green

# Rollback instantly if issues
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
Components declare required permissions, enforced by the framework:
```rust
#[component_capabilities]
pub fn required_capabilities() -> Vec<Capability> {
    vec![
        Capability::FileRead("/data".into()),
        Capability::NetworkOutbound("api.example.com".into()),
    ]
}
```

### Component Composition
Complex systems are built by connecting components:
```rust
let pipeline = ComponentPipeline::builder()
    .add_component("input", data_source)
    .add_component("processor", ai_model)
    .add_component("output", result_handler)
    .connect("input.output", "processor.input")
    .connect("processor.output", "output.input")
    .build()?;
```

## Framework Benefits

### Development
- SDK with derive macros for streamlined development
- Support for multiple WASM-compatible programming languages
- Fast build, test, and deploy cycles
- Visual component composition tools

### Operations
- Component updates without system downtime
- Version rollback capabilities
- Sandboxed component execution
- Resource management and monitoring

### Architecture
- Modular system design through component separation
- Controlled inter-component communication
- Scalable deployment patterns
- Integration with AirsSys ecosystem components

## Technology Foundation

The framework is built on established technologies:
- **Wasmtime**: WebAssembly runtime with Component Model support
- **WebAssembly Component Model**: Standard for component composition
- **WIT**: Language-agnostic interface definitions
- **WASI Preview 2**: Standardized system interface

## Getting Started

To begin working with airssys-wasm:

1. **Architecture Overview**: Read the [Framework Overview](./architecture/overview.md)
2. **First Component**: Follow the [Getting Started Guide](./guides/getting-started.md)
3. **Examples**: Review component implementations across different domains
4. **Community**: Participate in the component ecosystem development

## Project Status

airssys-wasm is currently in the planning and architecture design phase. The framework is designed to provide infrastructure for component-based application development with hot deployment capabilities.