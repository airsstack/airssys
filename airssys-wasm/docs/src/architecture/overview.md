# Framework Overview

airssys-wasm is a **Universal Hot-Deployable WASM Component Framework** that revolutionizes how software components are built, deployed, and managed. This document provides a high-level architectural overview of the framework.

## Strategic Positioning

### "CosmWasm for Everything"
While CosmWasm brings smart contract capabilities to the Cosmos blockchain, airssys-wasm brings smart contract-style deployment to **general-purpose computing**. This means:

- **Hot Deployment**: Deploy without restart (like smart contracts)
- **Universal Framework**: Works for any domain, not just blockchain
- **Capability Security**: Fine-grained permissions like smart contract capabilities
- **Component Composition**: Chain components like transaction flows

### Infrastructure Platform
airssys-wasm is **infrastructure**, not an application. It's the foundation that others build upon:
- **Framework Builders**: Create domain-specific frameworks on top
- **Application Developers**: Build components for any use case
- **System Integrators**: Compose complex systems from simple parts
- **Platform Providers**: Offer managed component hosting

## Core Architecture Layers

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
├─────────────────────────────────────────────────────────────┤
│                  AirsSys Integration                        │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐ │
│  │ airssys-osl     │ │ airssys-rt      │ │ Host System     │ │
│  │ Bridge          │ │ Bridge          │ │ Interface       │ │
│  └─────────────────┘ └─────────────────┘ └─────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Layer 1: AirsSys Integration
**Foundation**: Deep integration with the AirsSys ecosystem
- **airssys-osl Bridge**: Secure system access through OS layer
- **airssys-rt Bridge**: Actor-based component hosting  
- **Host System Interface**: Unified interface to host capabilities

### Layer 2: Core Runtime System
**Engine**: The heart of the WASM execution environment
- **WASM Runtime (Wasmtime)**: Industry-leading Component Model runtime
- **Capability Manager**: Fine-grained security and permission enforcement
- **Resource Manager**: Memory, CPU, and I/O resource management

### Layer 3: Component Framework
**Intelligence**: The smart deployment and composition system
- **Hot Deployment Engine**: Zero-downtime updates with multiple strategies
- **Composition Engine**: Component orchestration and pipeline management
- **Version Manager**: Git-like versioning with instant rollback

### Layer 4: Developer Experience
**Productivity**: Tools that make component development effortless
- **SDK & Macros**: Rich derive macros for easy component creation
- **WIT Bindings**: Language-agnostic interface generation
- **Visual Composition**: Drag-and-drop pipeline building

## Framework Design Principles

### 1. Universal Applicability
**Domain Agnostic**: The framework works for any domain without modification:
- AI agents and machine learning pipelines
- Web services and microservices
- IoT device controllers and sensors
- Game mods and entertainment systems
- Business logic and workflow engines

### 2. Smart Contract Paradigm
**Blockchain-Inspired**: Apply proven blockchain concepts to general computing:
- **Immutable Deployments**: Components are versioned and immutable
- **Capability-Based Security**: Explicit permission grants like smart contracts
- **Hot Updates**: Deploy without restart like contract upgrades
- **Audit Trail**: Complete history of deployments and changes

### 3. Developer Experience First
**Friction-Free Development**: Make component development as easy as writing functions:
- **Rich SDK**: Derive macros that generate boilerplate
- **Visual Tools**: Drag-and-drop component composition
- **Instant Feedback**: Fast build, test, deploy cycles
- **Any Language**: Support for all WASM-compatible languages

### 4. Production Ready
**Enterprise Grade**: Built for mission-critical production systems:
- **Security by Default**: Deny-by-default with explicit capability grants
- **Built-in Monitoring**: Performance metrics, health checks, alerting
- **Operational Excellence**: Logging, tracing, configuration management
- **High Availability**: No single points of failure

## Core Components

### Universal Component Interface
Every component implements the same universal interface:
```rust
pub trait UniversalComponent {
    // Lifecycle management
    fn init(&mut self, config: ComponentConfig) -> Result<(), ComponentError>;
    fn execute(&self, input: ComponentInput) -> Result<ComponentOutput, ComponentError>;
    fn shutdown(&mut self) -> Result<(), ComponentError>;
    
    // Component introspection
    fn metadata(&self) -> ComponentMetadata;
    fn required_capabilities(&self) -> Vec<Capability>;
    fn health_status(&self) -> HealthStatus;
}
```

### Hot Deployment Engine
The deployment engine manages component lifecycle without downtime:
```rust
pub struct HotDeploymentEngine {
    // Live component registry
    component_registry: Arc<RwLock<LiveComponentRegistry>>,
    
    // Multiple deployment strategies
    deployment_strategies: HashMap<String, Box<dyn DeploymentStrategy>>,
    
    // Version management
    version_manager: ComponentVersionManager,
    
    // Traffic routing
    traffic_router: TrafficRouter,
}
```

### Capability-Based Security
Fine-grained security with explicit permissions:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability {
    FileRead(PathBuf),
    FileWrite(PathBuf),
    NetworkOutbound(String),
    NetworkInbound(u16),
    SystemCall(String),
    Custom(String, serde_json::Value),
}
```

## Technology Stack

### Core Technologies
- **Wasmtime**: Production-ready WASM runtime with Component Model
- **WebAssembly Component Model**: Advanced component composition
- **WIT (WebAssembly Interface Types)**: Language-agnostic interfaces
- **WASI Preview 2**: Standardized system interface

### Integration Technologies
- **airssys-osl**: OS layer for secure system access
- **airssys-rt**: Actor system for component hosting
- **Tokio**: Async runtime for high-performance I/O
- **Capability-std**: Capability-based filesystem access

## Deployment Models

### Development
- **Local Runtime**: Embedded runtime for development and testing
- **Hot Reload**: Instant component updates during development
- **Debug Tools**: Rich debugging and profiling capabilities

### Production
- **Standalone Runtime**: Dedicated component hosting service
- **Cluster Mode**: Distributed component execution
- **Cloud Native**: Kubernetes and container integration

### Edge
- **Embedded Runtime**: Lightweight runtime for resource-constrained devices
- **Offline Capable**: Components work without network connectivity
- **Resource Aware**: Intelligent resource management for edge devices

## Competitive Advantages

### vs. Container Orchestration (Kubernetes)
- ✅ **10x Faster**: Seconds vs minutes for deployment
- ✅ **10x Lighter**: WASM overhead vs container overhead
- ✅ **Better Security**: Capability-based vs namespace isolation
- ✅ **Simpler Operations**: No complex orchestration required

### vs. Serverless Platforms (Lambda, etc.)
- ✅ **No Cold Starts**: Instant component activation
- ✅ **Stateful Components**: Components can maintain state
- ✅ **No Vendor Lock-in**: Open standard, runs anywhere
- ✅ **Cost Efficiency**: No per-invocation billing model

### vs. Traditional Plugin Systems
- ✅ **Zero Downtime**: Updates without restart
- ✅ **Memory Safety**: Sandboxed vs native code
- ✅ **Language Freedom**: Any language vs single language
- ✅ **Version Management**: Git-like versioning built-in

## Success Metrics

### Technical Performance
- **Component Instantiation**: < 10ms (fast cold starts)
- **Hot Deployment**: < 1 second (zero downtime)
- **Memory Isolation**: 100% (complete sandbox)
- **Throughput**: > 10,000 component calls/second

### Developer Experience
- **Setup Time**: < 5 minutes (new developer onboarding)
- **Build Time**: < 30 seconds (typical component)
- **Deploy Time**: < 60 seconds (development to production)

### Ecosystem Growth
- **Community Components**: > 50 (public registry)
- **Documentation Coverage**: > 95% (complete API docs)
- **Framework Adoption**: > 100 projects (production usage)

## Next Steps

1. **Explore the Component Model**: Learn how components work
2. **Understand Hot Deployment**: See how zero-downtime updates work
3. **Review Security Model**: Understand capability-based security
4. **Try the SDK**: Build your first component

This framework represents a fundamental shift in how we think about software deployment and composition. Welcome to the component-based future! 🚀