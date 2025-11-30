# airssys-wasm Project Brief

## Project Overview
`airssys-wasm` is a **WASM Component Framework for Pluggable Systems** that enables runtime component deployment for general-purpose computing. Inspired by smart contract deployment patterns (like CosmWasm), it enables developers to build secure, composable, language-agnostic components that can be loaded and updated during runtime.

## Workspace Structure

**The airssys-wasm ecosystem consists of three distinct sub-projects** in the AirsSys workspace:

### 1. airssys-wasm (Core Framework Library)
**Location**: `airssys/airssys-wasm/`  
**Type**: Library crate (`[lib]`)  
**Implementation Tasks**: Blocks 1-9 (WASM-TASK-002 through WASM-TASK-010)  
**Status**: 95% of Layer 1 complete

**Responsibilities**:
- Component runtime (Wasmtime-based WASM execution)
- Security system (capability-based sandboxing)
- Storage system (persistent component state)
- Messaging system (inter-component communication)
- Actor integration (ComponentActor with airssys-rt)
- Deployment engine (runtime component updates)
- Monitoring (observability and metrics)

### 2. airssys-wasm-component (Procedural Macro Crate)
**Location**: `airssys/airssys-wasm-component/`  
**Type**: Procedural macro crate (`[lib] proc-macro = true`)  
**Implementation Task**: Block 10 (WASM-TASK-011)  
**Status**: 25% foundation complete

**Responsibilities**:
- `#[component]` macro for zero-boilerplate development
- `#[derive(ComponentOperation)]` for message types
- `#[derive(ComponentResult)]` for result types
- `#[derive(ComponentConfig)]` for configuration
- Code generation (`extern "C"` functions, memory management)
- Follows **serde pattern** (separation from core types)

### 3. airssys-wasm-cli (Command-Line Tool)
**Location**: `airssys/airssys-wasm-cli/`  
**Type**: Binary crate (`[[bin]] name = "airssys-wasm"`)  
**Implementation Task**: Block 11 (WASM-TASK-012)  
**Status**: 10% foundation complete

**Responsibilities**:
- 14 comprehensive commands for component lifecycle
- Cryptographic operations (Ed25519 keygen, signing, verification)
- Project management (init, build)
- Installation (multi-source: Git/Local/URL)
- Operations (list, info, status, logs)
- Configuration and shell completions

### Dependency Relationships
```
airssys-wasm (Core) ← Used by both
         ↑              ↑
         │              │
airssys-wasm-component  airssys-wasm-cli
   (Macros)             (CLI Tool)
```

**📚 Complete Reference**: See **KNOWLEDGE-WASM-015: Project Structure and Workspace Architecture** for comprehensive documentation of the three sub-projects and their relationships.

## Project Vision
This framework provides infrastructure for building pluggable systems with WebAssembly components, enabling secure isolation and runtime component management. Rather than building application-specific plugin systems, airssys-wasm provides a foundational platform for component-based architectures across multiple domains.

## Core Value Propositions
1. **Cross-Platform Component Framework**: Language-agnostic component development with standard WIT interfaces
2. **Runtime Deployment**: Components can be loaded and updated during runtime without system restart
3. **Capability-Based Security**: Sandbox isolation by default with fine-grained permissions
4. **Language-Agnostic Development**: Support for any WASM-compatible language (Rust, C++, Go, Python, etc.)
5. **Component Composition**: Chain and orchestrate components for complex processing pipelines

## Key Capabilities
- Component orchestration and isolation
- Runtime component deployment and management
- Secure, composable component execution with capability-based security

## Core Responsibilities

### Component Framework
- Language-agnostic component development with WIT interfaces
- Component lifecycle management (init, execute, shutdown)
- Component composition and orchestration engine
- Dependency resolution and graph execution

### Runtime Deployment System  
- Components can be loaded and updated during runtime
- Multiple deployment strategies (Blue-Green, Canary, Rolling)
- Version management with rollback capabilities
- Traffic routing and load balancing for component deployment

### Security & Isolation
- Capability-based security with deny-by-default policies
- Fine-grained permission system for resource access
- Component sandboxing and memory isolation
- Security audit logging and policy enforcement

### AirsSys Ecosystem Integration
- Deep integration with airssys-osl for secure system access
- Integration with airssys-rt for actor-based component hosting
- Unified logging and configuration management
- Host system interface and capability bridging

## Target Use Cases & Domains

### Target Use Cases (Examples)
- **AI Agents**: Secure, composable AI agent systems
- **MCP Tools**: Model Context Protocol tools as WASM components
- **Microservices**: Lightweight microservices with runtime updates
- **Data Processors**: ETL and data transformation pipelines
- **IoT Controllers**: Edge device controllers and processors
- **Plugin Systems**: Secure plugin architectures for applications

### Applicable Domains
- Enterprise software with secure plugin ecosystems
- Edge computing with lightweight, secure functions
- AI/ML infrastructure with composable model pipelines
- Gaming platforms with secure mod systems
- Web platforms with secure extensions

## Technical Requirements

### Core Framework Requirements
- General-purpose component interface supporting multiple domains
- Runtime deployment engine for component loading and updates
- Capability-based security system with fine-grained permissions
- Component composition engine for pipeline orchestration
- Version management with rollback capabilities

### Performance Requirements
- Component instantiation < 10ms (cold start time)
- Component deployment < 1 second (runtime loading)
- Memory isolation 100% (no component cross-access)
- Rollback time < 5 seconds
- Throughput > 10,000 component calls/second

### Developer Experience Requirements
- Rich SDK with derive macros for easy development
- WIT interface system for language-agnostic contracts
- Comprehensive testing framework and utilities
- Visual component composition and pipeline building
- Complete documentation with examples across domains

### Integration Requirements
- Deep airssys-osl integration for secure system access
- airssys-rt integration for actor-based hosting
- WASI Preview 2 support for system interface
- Monitoring and observability built-in
- Configuration management and service discovery
- Efficient resource sharing and reuse

### Compatibility Requirements
- WebAssembly Component Model compatibility
- WASI preview 2 support for system interface
- Multiple WASM language support (Rust, C/C++, JavaScript, etc.)
- Integration with existing WASM toolchains and package managers

## Architecture Constraints
- Follow workspace standards (§2.1, §3.2, §4.3, §5.1)
- Rust-based implementation with wasmtime or similar runtime
- Zero unsafe code blocks without security review
- Comprehensive security policy validation and enforcement

## Integration Points
- **airssys-osl**: Secure system access through OS layer abstraction
- **airssys-rt**: Actor-based component hosting and lifecycle management
- **Component Ecosystem**: Integration with WASM component registries and tooling

## Success Criteria
- Pass comprehensive security audit for component isolation
- Achieve target performance metrics for component execution
- Successful demonstration of polyglot component composition
- Seamless integration with airssys-osl and airssys-rt components