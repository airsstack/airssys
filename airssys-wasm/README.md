# airssys-wasm: WASM Component Framework for Pluggable Systems

## Project Overview

**airssys-wasm** is a WASM Component Framework for Pluggable Systems that enables runtime deployment capabilities inspired by smart contract patterns (like CosmWasm) for general-purpose computing. This is a **host runtime library** - you use it to build applications that load and execute WASM components with security isolation, not to build the components themselves.

### What is airssys-wasm?

**airssys-wasm is the HOST runtime framework** - it provides the infrastructure for applications to load, execute, and manage WASM components at runtime. Think of it as:
- **Browser for WASM components**: Like Chrome runs JavaScript, airssys-wasm runs WASM components
- **Plugin host infrastructure**: The runtime that loads and manages third-party components
- **Component orchestration engine**: Manages lifecycle, security, communication, and composition

**What you build with airssys-wasm**: Applications that host WASM components (the host system)  
**What you DON'T build with airssys-wasm**: The WASM components themselves (use `airssys-wasm-component` for that)

### Original Objectives

The framework was designed to solve five key challenges in pluggable system architecture:

1. **Runtime Component Deployment** - Load and update components during runtime without system restart (inspired by smart contract deployment patterns like CosmWasm)

2. **Security-First Architecture** - Capability-based security with sandbox isolation by default, preventing malicious components from compromising the host system

3. **Language-Agnostic Development** - Support components written in any WASM-compatible language through WIT (WebAssembly Interface Types) interfaces

4. **Component Composition** - Chain and orchestrate components for complex processing pipelines with secure inter-component communication

5. **General-Purpose Infrastructure** - Provide foundation platform for component-based architectures across multiple domains (AI, microservices, IoT, gaming, etc.) rather than being domain-specific

## Planned Architecture

## High-Level Architecture

### How It Works: The Three-Layer Stack

```
┌─────────────────────────────────────────────────────────────┐
│  APPLICATION LAYER (What You Build)                        │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  Your Host Application                                  ││
│  │  - Loads WASM components at runtime                     ││
│  │  - Manages component lifecycle                          ││
│  │  - Orchestrates component communication                 ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
                           ↓ uses
┌─────────────────────────────────────────────────────────────┐
│  HOST RUNTIME LAYER (airssys-wasm - This Library)          │
│  ┌──────────────┐ ┌──────────────┐ ┌───────────────────────┐│
│  │ Component    │ │ Security     │ │ Actor Integration     ││
│  │ Loading &    │ │ & Capability │ │ (airssys-rt)          ││
│  │ Execution    │ │ Enforcement  │ │                       ││
│  └──────────────┘ └──────────────┘ └───────────────────────┘│
│  ┌──────────────┐ ┌──────────────┐ ┌───────────────────────┐│
│  │ Messaging &  │ │ Storage      │ │ OS Integration        ││
│  │ Communication│ │ Backend      │ │ (airssys-osl)         ││
│  └──────────────┘ └──────────────┘ └───────────────────────┘│
└─────────────────────────────────────────────────────────────┘
                           ↓ loads & executes
┌─────────────────────────────────────────────────────────────┐
│  COMPONENT LAYER (Third-Party Plugins)                     │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  WASM Components (.wasm files)                          ││
│  │  - Written in any language (Rust, C++, Go, Python, JS)  ││
│  │  - Compiled to WebAssembly                              ││
│  │  - Isolated and sandboxed                               ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

### Core Architecture Components

#### 1. Runtime Component Deployment
- **Live Component Registry**: Manages components without system restart
- **Deployment Strategies**: Blue-Green, Canary, Rolling updates
- **Version Management**: Git-like versioning with rollback capabilities
- **Traffic Routing**: Load balancing and traffic splitting during deployment

#### 2. Security & Isolation
- **Wasmtime Runtime**: Memory-isolated sandbox execution
- **Capability System**: Fine-grained permissions (deny-by-default)
  - File access: Readable/writable/executable path patterns
  - Network: Inbound/outbound with host restrictions
  - Process: Spawn/signal permissions
- **Security Enforcer**: Runtime validation before every system access
- **Audit Logging**: All component operations logged via airssys-osl

#### 3. Language-Agnostic Development
- **WIT Interfaces**: Language-agnostic component contracts (7-package system)
- **wit-bindgen**: Auto-generate bindings for Rust, C++, Go, Python, JS
- **WASI Preview 2**: Standard system interface across all languages
- **Component Model**: WebAssembly Component Model for composition

#### 4. Component Composition
- **Component Bridge**: Inter-component messaging system
- **Shared Memory**: Zero-copy communication for large data
- **Message Router**: Secure routing with permission checks
- **Pipeline Orchestration**: Chain components for complex workflows

#### 5. AirsSys Ecosystem Integration
- **OSL Bridge**: Delegates system operations to airssys-osl with security context
- **RT Integration**: Components hosted as actors in airssys-rt system
- **Unified Monitoring**: All events logged through OSL activity logger

## The AirsSys WASM Ecosystem

airssys-wasm is part of a three-project ecosystem that works together to provide a complete WASM component development and hosting experience:

### 1. airssys-wasm (This Library - Host Runtime)

**Role**: Runtime library for building applications that host/run WASM components

**What it provides:**
- Component loading and execution engine
- Security (capability enforcement)
- Actor-based isolation
- Messaging between components
- Storage backend
- Integration with airssys-osl and airssys-rt

**Who uses it**: Application developers building systems that load WASM components

**Usage Example:**
```rust
// Your application uses airssys-wasm to load and run components
use airssys_wasm::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the component runtime
    let runtime = ComponentRuntime::new(config)?;
    
    // Load a WASM component with security capabilities
    let component_id = runtime.load_component(
        wasm_bytes,
        capabilities
    ).await?;
    
    // Execute the component
    let result = runtime.execute(component_id, input).await?;
    
    Ok(())
}
```

### 2. airssys-wasm-component (Developer Macros)

**Role**: Procedural macros to simplify building WASM components

**What it provides:**
- `#[component]` macro - Eliminates `extern "C"` boilerplate
- Derive macros - `ComponentOperation`, `ComponentResult`, `ComponentConfig`
- Automatic code generation (memory management, serialization)

**Who uses it**: Component developers writing plugins/components

**Architecture Pattern**: Follows the **serde pattern** (separate macro crate from core types)
- Optional: Can use airssys-wasm types directly without macros
- Faster builds: Macro compilation is separate
- Flexible: Choose your level of abstraction

**Usage Example:**
```rust
// Component developers use macros to build plugins easily
use airssys_wasm_component::component;

#[component(name = "my-plugin", version = "1.0.0")]
pub struct MyPlugin {
    // Focus on business logic - no WASM complexity!
    state: PluginState,
}

impl Component for MyPlugin {
    fn execute(&mut self, input: ComponentInput) -> Result<ComponentOutput, ComponentError> {
        // Your plugin logic here
        Ok(output)
    }
}
```

### 3. airssys-wasm-cli (Developer Tooling)

**Role**: Command-line tool for managing WASM components

**What it provides:**
- `keygen` - Generate Ed25519 signing keys
- `init` - Create new component projects
- `build` - Compile components to WASM
- `sign` - Cryptographically sign components
- `install` - Install from Git/file/URL
- `update`/`uninstall` - Component lifecycle management
- `list`/`info`/`logs` - Monitoring and inspection

**Who uses it**: Component developers during development workflow

**Usage Example:**
```bash
# Complete developer workflow using CLI
airssys-wasm keygen                          # Generate signing keys
airssys-wasm init my-plugin                  # Create new component project
cd my-plugin
airssys-wasm build --release                 # Build component to WASM
airssys-wasm sign my-plugin.wasm             # Sign the component
airssys-wasm install ./my-plugin.wasm        # Install locally for testing
```

### How The Ecosystem Works Together

```
┌─────────────────────────────────────────────────┐
│  Component Developer Workflow                   │
├─────────────────────────────────────────────────┤
│  1. airssys-wasm-cli init my-plugin            │  ← CLI creates project
│  2. Write code using airssys-wasm-component    │  ← Macros simplify development
│  3. airssys-wasm-cli build --release           │  ← CLI builds to .wasm
│  4. airssys-wasm-cli sign my-plugin.wasm       │  ← CLI signs component
│                                                  │
│  Output: my-plugin.wasm (signed component)      │
└─────────────────────────────────────────────────┘
                        │
                        ▼ Deploy/Install
┌─────────────────────────────────────────────────┐
│  Host Application Developer Workflow            │
├─────────────────────────────────────────────────┤
│  1. Build app with airssys-wasm runtime         │  ← Use host library
│  2. Load my-plugin.wasm at runtime              │  ← Runtime loads component
│  3. Execute with security isolation             │  ← Capability enforcement
│  4. Manage lifecycle and communication          │  ← Component orchestration
└─────────────────────────────────────────────────┘
```

### Dependency Relationships

```
airssys-wasm-cli
  └─► airssys-wasm (uses to validate and manage components)

airssys-wasm-component  
  └─► airssys-wasm (uses core types: Component, ComponentError, etc.)

airssys-wasm
  ├─► airssys-osl (secure system access)
  └─► airssys-rt (actor-based component hosting)
```

### Real-World Analogy

Think of it like web development:

| Web Ecosystem | AirsSys WASM Ecosystem |
|---------------|------------------------|
| **Browser** (Chrome/Firefox) | **airssys-wasm** (host runtime) |
| **React/JSX** (developer framework) | **airssys-wasm-component** (macros) |
| **npm CLI** (package manager) | **airssys-wasm-cli** (component manager) |
| **JavaScript code** | **WASM components** (plugins) |

Just as a browser loads and runs JavaScript, airssys-wasm loads and runs WASM components. Just as React makes JavaScript development easier, airssys-wasm-component makes WASM component development easier. Just as npm manages packages, airssys-wasm-cli manages components.

## Technical Implementation Layers

The framework achieves its objectives through a layered implementation strategy:

```
Layer 1: WIT Interface System (95% complete)
  ├── Core package (types, capabilities, lifecycle, host)
  └── Extension packages (filesystem, network, process)
  
Layer 2: Runtime & Security (Next - Block 3)
  ├── Wasmtime integration
  ├── Component loading/instantiation  
  └── Capability enforcement
  
Layer 3: Deployment & Orchestration (Planned)
  ├── Component registry
  ├── Deployment strategies
  └── Component composition
  
Layer 4: AirsSys Integration (Planned)
  ├── OSL bridge (secure system access)
  └── RT integration (actor hosting)
```

### Current Implementation Status

**Phase**: Block 2 Phase 3 - **95% Complete** (Documentation gap only)

**What's Implemented:**
- ✅ Complete WIT interface system (16 files, 2,214 lines)
- ✅ Build system functional (build.rs + wit-bindgen integration)
- ✅ Permission system complete (Component.toml parser + validation)
- ✅ 250+ tests passing
- ✅ Core abstractions defined (Component, Capability, Security, etc.)

**Next Steps:**
- 🚀 Block 3: Actor System Integration (runtime implementation)
- 📚 User documentation completion (non-blocking)

## Technology Foundation

The framework will build on established technologies:
- **Wasmtime**: WebAssembly runtime with Component Model support
- **WebAssembly Component Model**: Standard for component composition
- **WIT (WebAssembly Interface Types)**: Language-agnostic interface definitions
- **WASI Preview 2**: Standardized system interface for capabilities
- **Capability-based Security**: Security model for controlled resource access

## Planned Use Cases

### AI and Machine Learning
- **AI Agent Component Systems**: Isolated agent components with secure inter-agent communication
- **MCP Tool Execution**: Model Context Protocol tools as WebAssembly components with controlled access
- **ML Pipeline Stages**: Data preprocessing, feature extraction, and model inference as composable components
- **Model Serving**: Runtime model deployment with version management and A/B testing support

### Web and Microservices
- **API Gateway Components**: Request routing, authentication, and rate limiting as hot-swappable components
- **Serverless Functions**: Event-driven functions with sub-10ms startup times
- **Plugin Systems**: Third-party extensions for web applications with security sandboxing
- **Edge Functions**: Lightweight functions deployed to edge locations with instant updates

### Enterprise and Systems
- **Business Rule Engines**: Runtime business logic updates without application restarts
- **Data Processing Pipelines**: ETL components with hot-swappable transformation stages
- **Integration Adapters**: Protocol converters and system connectors as managed components
- **Workflow Orchestration**: State machine components with dynamic workflow updates

### Gaming and Entertainment
- **Game Mod Systems**: User-created content with security isolation and resource limits
- **Content Pipeline**: Asset processing and transformation components
- **Real-time Feature Rollout**: Game features and balance changes deployed during gameplay
- **Event Systems**: Game events and challenges as hot-deployable components

## Framework Comparison Analysis

### Compared to Container Systems (Docker, Podman)

**Target Performance Characteristics:**

| Aspect | WASM Components (Target) | Docker Containers (Typical) |
|--------|--------------------------|----------------------------|
| Startup Time | < 10ms | 1-5 seconds |
| Memory Overhead | < 512KB baseline | 10-100+ MB |
| Deployment Time | < 1 second | 10-60 seconds |
| Isolation Level | Memory (WASM sandbox) | OS-level (cgroups/namespaces) |
| Platform Support | Cross-platform native | Linux-native (others via emulation) |

**Tradeoffs:**
- WASM: Better for code-level components, lower overhead, faster startup
- Containers: Better for full applications, stronger OS-level isolation

### Compared to Serverless Platforms (AWS Lambda, Cloud Functions)

**Design Goals:**

| Aspect | This Framework (Goal) | Serverless Platforms (Typical) |
|--------|-----------------------|--------------------------------|
| Cold Start | Eliminate (< 10ms) | 100-1000+ ms |
| Vendor Lock-in | None (open standards) | High (proprietary APIs) |
| Execution Model | Long-running + event | Event-driven only |
| Resource Management | Direct control | Abstracted billing units |

**Use Case Fit:**
- This framework: Long-running services, real-time processing, custom infrastructure
- Serverless: Event-driven workloads, pay-per-use, managed infrastructure

### Compared to Plugin Systems (Native Shared Libraries)

**Security and Isolation:**

| Feature | WASM Components | Native Plugins (.so/.dll) |
|---------|----------------|---------------------------|
| Memory Safety | Sandboxed (enforced) | Shared process (unsafe) |
| Crash Isolation | Yes (trap handling) | No (process crash) |
| Resource Limits | Configurable (enforced) | OS-level only |
| Updates | Runtime without restart | Requires restart |
| Cross-platform | Yes (WASM portable) | No (platform-specific) |

**Tradeoffs:**
- WASM: Safer, portable, runtime updates; slightly higher call overhead
- Native: Faster call overhead; no safety guarantees, platform-specific

**Note:** These are design targets based on architectural decisions and preliminary benchmarks. Production performance will be validated and documented as implementation progresses.

## Development Roadmap

### Completed Dependencies ✅
- ✅ **airssys-osl** (100% complete) - OS layer for system access abstraction
- ✅ **airssys-rt** (100% complete) - Actor system for component hosting
- ✅ **WASM-TASK-000** (100% complete) - Core abstractions foundation
- ✅ **WASM-TASK-002** (100% complete) - WASM runtime layer (6 phases)
- ✅ **WASM-TASK-003 Phases 1-3** (95% complete) - WIT system, build system, permissions

### Current Phase: Block 3 - Actor System Integration
**Status:** Ready to start (all prerequisites met)

**Objectives:**
- Integrate WASM components with airssys-rt actor system
- Component-as-actor hosting patterns
- Supervisor tree integration for component lifecycle
- Message passing between components via actors

### Future Phases
- **Block 4**: Security & Capability Enforcement
- **Block 5**: Component Composition & Orchestration
- **Block 6**: Deployment Strategies (Blue-Green, Canary, Rolling)
- **Blocks 7-11**: Advanced features (monitoring, marketplace, etc.)

## Project Status

**Current Phase**: Block 2 Phase 3 - **95% Complete** (Documentation Gap Only)

**What's Implemented:**
- ✅ Complete WIT interface system (16 files, 2,214 lines)
- ✅ Build system functional (build.rs + wit-bindgen integration)
- ✅ Permission system complete (Component.toml parser + validation)
- ✅ 250+ tests passing
- ✅ Core abstractions defined (Component, Capability, Security, etc.)

**What's Remaining:**
- ⏳ User documentation (5% - Getting Started guides, tutorials, examples)

**Next Steps:**
- 🚀 **Block 3: Actor System Integration** (ready to start - all prerequisites met)
- 📚 Documentation sprint (parallel track, non-blocking)

### Recent Milestones

#### Phase 3 Substantially Complete: Build System + Permission System (Nov 2025)
- ✅ **Complete WIT System**: 2,214 lines across 16 WIT files
  - Core interfaces: 569 lines (types, capabilities, lifecycle, host, permissions, worlds)
  - Extension interfaces: 1,645 lines (filesystem, network, process)
- ✅ **Build System**: Fully functional with wit-bindgen integration
  - Automatic Rust binding generation (154KB generated code)
  - Two-stage validation (wasm-tools → wit-bindgen)
  - Incremental build optimization (~2s incremental builds)
- ✅ **Permission System**: Complete implementation
  - Component.toml parser and validator
  - Pattern matching (PathPattern, DomainPattern, etc.)
  - Comprehensive test coverage (250+ tests passing)
- ✅ **Architectural Decisions**: All deviations justified and documented
  - DEBT-WASM-003: Single-package structure (Component Model v0.1 constraints)
  - KNOWLEDGE-WASM-009: Component.toml manifest architecture
  - KNOWLEDGE-WASM-014: Phase 3 completion retrospective

#### Phase 2 Complete: WIT Implementation Foundation (Oct 2025)
- ✅ Core + Extension packages implementation (16 WIT files, 2,214 lines)
- ✅ 100% validation passing (wasm-tools 1.240.0)
- ✅ Zero type duplication through `use` statements
- ✅ Acyclic dependency graph verified

#### Phase 1 Complete: Research and Foundation (Oct 2025)
- ✅ WIT ecosystem thoroughly researched (25 documents, 6,500+ lines)
- ✅ Package structure fully designed
- ✅ Build system integration strategy proven

## Getting Started

### Prerequisites

To build airssys-wasm, you need:

- **Rust 1.80+**: `rustup update`
- **wasm-tools 1.240.0**: `cargo install wasm-tools --version 1.240.0`
- **wit-bindgen 0.47.0**: `cargo install wit-bindgen-cli --version 0.47.0`

### Quick Start

```bash
# Clone and build
git clone https://github.com/airsstack/airssys
cd airssys/airssys-wasm
cargo build

# Run tests
cargo test
```

**For detailed build instructions, troubleshooting, and development guides, see [Documentation](#documentation).**

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