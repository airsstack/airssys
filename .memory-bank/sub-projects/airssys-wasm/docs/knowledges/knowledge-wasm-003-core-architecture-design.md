# Core Architecture Design - airssys-wasm

**Document Type:** Knowledge Documentation  
**Created:** 2025-10-02  
**Status:** Foundational Architecture Defined  
**Priority:** Critical - Core Framework Foundation  

## Project Purpose & Vision

### Core Problem Statement
Create a framework for building **pluggable/component-based architecture systems** based on WASM binary format, enabling:
- **Plugin Developers**: Create WASM components for any computation domain
- **Host System Developers**: Embed runtime to support WASM plugins in their applications

### Key Inspiration: Smart Contract Model
Inspired by **CosmWasm** and **NEAR Protocol** smart contract deployment patterns:
- Engineers create smart contracts → upload to blockchain network
- **Our Framework**: Engineers create WASM components → deploy to host systems
- **Difference**: Designed for **general computing**, not blockchain-specific

### Core Value Proposition: Cross-Platform Isolation
**Problem Solved**: Traditional containers (Docker) are Linux-specific (cgroups/namespaces), heavy, OS-dependent
**Solution**: WASM as cross-platform "container" format providing isolation across all operating systems

## Core Conceptual Model

### Two-Audience Developer Experience
```
Plugin Developer → Writes WASM Component → Publishes to Registry
                                              ↓
Host System Developer → Creates Host Application → Loads & Executes Components
```

**Key Insight**: Framework serves **two distinct developer audiences**:
1. **Plugin/Component Developers** - Write business logic in WASM
2. **Host System Developers** - Embed the runtime in their applications

## Core Architecture Layers

### Layer 1: Foundation (AirsSys Integration)
```
┌─────────────────────────────────────────────────────────────┐
│ airssys-osl: OS Abstraction, Security, Filesystem          │
├─────────────────────────────────────────────────────────────┤
│ airssys-rt: Actor System, Message Passing, Supervision     │
└─────────────────────────────────────────────────────────────┘
```

**Integration Pattern**: Each WASM component runs as an **actor** in airssys-rt:
- Component isolation through actor boundaries
- Message passing for inter-component communication
- Supervision trees for fault tolerance
- Integration with airssys-osl for secure system access

### Layer 2: WASM Runtime Engine
```
┌─────────────────────────────────────────────────────────────┐
│ Component Lifecycle: Load, Execute, Unload, Runtime Reload │
├─────────────────────────────────────────────────────────────┤
│ WASM Engine: wasmtime with Component Model support         │
├─────────────────────────────────────────────────────────────┤
│ WIT Interface System: Standard IDL with custom extensions  │
└─────────────────────────────────────────────────────────────┘
```

**Core Responsibilities**:
- WASM binary execution with isolation
- Component lifecycle management
- Memory management and resource limits
- Security sandbox enforcement

### Layer 3: Host Integration Framework
```
┌─────────────────────────────────────────────────────────────┐
│ Simple Embedding API: Easy integration for host developers │
├─────────────────────────────────────────────────────────────┤
│ Component Registry: Discovery, versioning, metadata        │
├─────────────────────────────────────────────────────────────┤
│ Security & Capabilities: Permission management, isolation  │
└─────────────────────────────────────────────────────────────┘
```

**Embedding API Pattern**:
```rust
// Simple embedding for host developers
let runtime = ComponentRuntime::new(config);
let component = runtime.load_component("my-plugin.wasm")?;
let result = component.call("process_data", &input)?;
```

### Layer 4: Developer Experience
```
┌─────────────────────────────────────────────────────────────┐
│ Development Tools: Code generation, testing, debugging     │
├─────────────────────────────────────────────────────────────┤
│ Documentation & Examples: Comprehensive guides & samples   │
├─────────────────────────────────────────────────────────────┤
│ AI Agent Integration: Future enhancement for code gen      │
└─────────────────────────────────────────────────────────────┘
```

**Note**: AI Agent integration is **future enhancement**, not current development priority.

## Key Architectural Decisions

### 1. Component Interface Strategy: WIT Standard
**Decision**: Use **WIT (WebAssembly Interface Types)** as standard IDL
**Rationale**:
- Standards-based, future-proof with WebAssembly ecosystem
- Language-agnostic multi-language support
- Growing tooling ecosystem (wit-bindgen)
- Structured interface definitions

### 2. System Access Strategy: Hybrid Approach
**Decision**: **WASI + Extensible Host Functions**
**Rationale**: Balance between standards compliance and flexibility

#### WASI Standard Functions (Basic Operations)
```wit
interface wasi:filesystem/types {
  read-file: func(path: string) -> result<list<u8>, error>
  write-file: func(path: string, data: list<u8>) -> result<_, error>
}
```

#### Custom Host Functions (Domain-Specific)
```wit
interface airssys:ai/model {
  load-model: func(model-path: string) -> result<model-handle, error>
  infer: func(handle: model-handle, input: tensor) -> result<tensor, error>
}

interface airssys:database/client {
  query: func(sql: string) -> result<list<row>, error>
  transaction: func(operations: list<db-operation>) -> result<_, error>
}
```

**Benefits**:
- WASI for standard operations (portable)
- Custom host functions for domain-specific capabilities
- Extensibility for host applications to add their own functions

### 3. Component Communication: Actor-Based via airssys-rt
**Decision**: Component communication through **airssys-rt message passing**
**Architecture**:
```
┌─────────────────────────────────────────────────────────────┐
│                    Host Application                         │
├─────────────────────────────────────────────────────────────┤
│                 airssys-wasm Runtime                        │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────┐ │
│  │ Component A │ │ Component B │ │ Component C             │ │
│  │ (Actor 1)   │ │ (Actor 2)   │ │ (Actor 3)               │ │
│  └─────────────┘ └─────────────┘ └─────────────────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                 airssys-rt Integration                      │
│         (Message Passing & Actor Supervision)              │
└─────────────────────────────────────────────────────────────┘
```

**Communication Flow**:
1. Component A sends message to Component B
2. Message routed through airssys-rt message passing system
3. airssys-rt handles routing, delivery guarantees, supervision
4. Component B receives message in actor mailbox

**WIT Interface**:
```wit
interface airssys:runtime/messaging {
  send-message: func(target-component: component-id, message: list<u8>) -> result<_, error>
  receive-message: func() -> result<option<message>, error>
  subscribe-to-events: func(event-types: list<string>) -> result<_, error>
}
```

### 4. Target Use Cases: General-Purpose First
**Decision**: Start with **general-purpose plugin system**
**Rationale**: 
- Broad applicability across domains
- AI/ML use cases emerge naturally
- Avoids premature specialization
- Larger market opportunity

## Fundamental Building Blocks

### Block 1: WASM Component Runtime
- **Purpose**: Execute WASM binaries with isolation
- **Responsibility**: Sandboxed execution, memory management, lifecycle
- **Interface**: Load, execute, unload components

### Block 2: Host Integration Framework
- **Purpose**: Allow host applications to embed the runtime
- **Responsibility**: API for host developers, lifecycle management
- **Interface**: Simple embedding API, configuration, component discovery

### Block 3: Component Interface System (WIT)
- **Purpose**: Define contracts between host and components
- **Responsibility**: Type-safe communication, function signatures
- **Interface**: WIT-based IDL, code generation

### Block 4: Security & Isolation System
- **Purpose**: Ensure components can't harm host or each other
- **Responsibility**: Capability-based permissions, resource limits
- **Interface**: Permission configuration, security policies

### Block 5: Component Registry & Distribution
- **Purpose**: Publish, discover, and download components
- **Responsibility**: Component metadata, versioning, distribution
- **Interface**: Publishing tools, discovery API

## Strategic Implementation Path

### Phase 1: Core Foundation (MVP)
**Focus**: Prove core value proposition
1. **Basic WASM Component Loading** with wasmtime
2. **WIT Interface System** with basic host functions
3. **Simple Embedding API** for host developers
4. **Basic airssys-rt Integration** for component communication

### Phase 2: Production Features
**Focus**: Enterprise capabilities
1. **Capability System** with security policies
2. **Component Registry** with versioning
3. **Enhanced Host Functions** (hybrid WASI + custom)
4. **Error Handling & Recovery** with supervision

### Phase 3: Advanced Features
**Focus**: Ecosystem capabilities
1. **Runtime Component Reload** for development and production
2. **Component Composition** patterns and tools
3. **Performance Optimization** and monitoring
4. **Ecosystem Tools** and community features

### Phase 4: AI Agent Integration (Future)
**Focus**: Developer experience enhancement
1. **Component Code Generation** from natural language
2. **Host Integration Generation** with best practices
3. **Testing Framework Generation** with comprehensive coverage
4. **Documentation Generation** with examples

**Note**: AI Agent development is **long-term vision**, not immediate priority.

## Critical Success Factors

### 1. Simplicity First
- Make embedding trivial for host developers
- Component development should be straightforward
- Clear, minimal APIs over complex features

### 2. Standards Compliance
- Build on WebAssembly standards (Component Model, WASI)
- Avoid proprietary solutions where standards exist
- Ensure future compatibility

### 3. AirsSys Integration Excellence
- Seamless integration with airssys-rt for actor-based hosting
- Deep integration with airssys-osl for secure system access
- Leverage existing AirsSys capabilities

### 4. Performance Acceptance
- Don't optimize prematurely, but ensure usability
- Focus on use cases where isolation value > performance cost
- Acceptable overhead for target applications

### 5. Documentation Excellence
- Clear examples for both component and host developers
- Comprehensive guides and tutorials
- Reference implementations and best practices

## Design Principles

### Universal Portability
WASM provides true universal execution - components run everywhere without OS-specific dependencies.

### Security by Default
Capability-based security with deny-by-default policies, explicit permission grants for component operations.

### Developer Experience Focus
Framework designed for ease of use, with AI enhancement as future multiplier (not dependency).

### Integration First
Deep integration with AirsSys ecosystem provides real value over standalone WASM runtime.

### Standards-Based
Build on WebAssembly standards for future compatibility and ecosystem integration.

---

**Status**: Foundational architecture defined, ready for detailed design and implementation planning.
**Next Steps**: Create detailed component specifications and implementation roadmap.