# WIT Management Architecture - airssys-wasm

**Document Type:** Knowledge Documentation  
**Created:** 2025-10-02  
**Status:** Complete Architecture Plan  
**Priority:** Critical - WIT Interface Management Framework  

## Architecture Principles

### Core Design Decisions
- **Host Runtime**: Always Rust with deep airssys-osl/rt integration
- **Components**: Any WASM-compatible language, no SDK requirements
- **Rust Components**: Enhanced experience via optional cargo plugin
- **Interfaces**: WIT-only contracts, no code dependencies
- **Serialization**: Multicodec for self-describing, language-agnostic data

### Key Architectural Benefits
- **True Language Freedom**: No framework lock-in, engineers use native tools
- **Universal Compatibility**: Multicodec ensures evolutionary compatibility
- **Deep Integration**: Rich Rust ecosystem for host development
- **Standards-Based**: Built on proven WIT and multiformat specifications

## WIT Interface Structure

### Core WIT Organization
```
airssys-wasm/
├── wit/
│   ├── core/                       # Required base interfaces
│   │   ├── component.wit           # Universal component lifecycle
│   │   ├── host.wit               # Core host services
│   │   ├── messaging.wit          # Inter-component communication
│   │   ├── multicodec.wit         # Multicodec support
│   │   └── types.wit              # Common types and errors
│   ├── extensions/                 # Optional domain extensions
│   │   ├── ai/
│   │   │   ├── model.wit          # AI model management
│   │   │   └── inference.wit      # AI inference operations
│   │   ├── database/
│   │   │   └── client.wit         # Database operations
│   │   ├── filesystem/
│   │   │   └── operations.wit     # File system operations
│   │   ├── network/
│   │   │   └── client.wit         # Network operations
│   │   └── crypto/
│   │       └── operations.wit     # Cryptographic operations
│   └── examples/                   # Reference implementations
│       ├── simple-processor.wit   # Basic component example
│       ├── ai-agent.wit          # AI agent example
│       └── data-pipeline.wit     # Data processing pipeline
```

### Package Naming Convention
```
Core Packages (Required):
- airssys:component-core@1.0.0      # Universal component interface
- airssys:host-core@1.0.0           # Host services interface
- airssys:messaging-core@1.0.0      # Component communication
- airssys:multicodec-core@1.0.0     # Multicodec utilities

Extension Packages (Optional):
- airssys:ai-extensions@0.5.0       # AI/ML domain interfaces
- airssys:database-extensions@0.3.0 # Database access interfaces
- airssys:filesystem-extensions@0.4.0 # File system interfaces
- airssys:network-extensions@0.3.0  # Network operation interfaces
- airssys:crypto-extensions@0.2.0   # Cryptographic interfaces

Custom Packages (User-defined):
- company:custom-apis@1.0.0         # User-specific interfaces
- team:business-logic@2.1.0         # Domain-specific interfaces
```

## Core WIT Interface Definitions

### Universal Component Interface (core/component.wit)
```wit
package airssys:component-core@1.0.0;

/// Universal component lifecycle interface - ALL components MUST implement
interface component-lifecycle {
    /// Initialize component with configuration
    init: func(config: component-config) -> result<_, component-error>;
    
    /// Single entrypoint for all operations (Solana-inspired)
    /// Self-describing data with multicodec prefix
    execute: func(
        operation: list<u8>,        // Multicodec-prefixed operation data
        context: execution-context
    ) -> result<list<u8>, execution-error>;
    
    /// Component metadata and capabilities
    metadata: func() -> component-metadata;
    
    /// Health check for monitoring
    health: func() -> health-status;
    
    /// Supported multicodec formats by this component
    supported-codecs: func() -> list<multicodec-id>;
    
    /// Graceful shutdown and cleanup
    shutdown: func() -> result<_, component-error>;
}

/// Component configuration during initialization
record component-config {
    /// Environment variables specific to this component
    env-vars: list<tuple<string, string>>,
    
    /// Component-specific configuration data (multicodec-encoded)
    config-data: option<list<u8>>,
    
    /// Resource limits for this component instance
    resource-limits: resource-limits,
    
    /// Capabilities granted to this component
    granted-capabilities: list<capability>,
}

/// Execution context for each operation
record execution-context {
    request-id: string,
    caller: caller-info,
    available-resources: resource-allocation,
    timeout-ms: u64,
    trace-context: option<trace-context>,
}

/// Component metadata returned by metadata() function
///
/// **AUDIENCE SEPARATION**:
/// - Standard fields (identity, runtime requirements, characteristics) → Read by HOST RUNTIME
/// - Discovery fields (homepage, repository, tags) → Read by COMPONENTS and HUMANS
/// - Custom metadata → Read by OTHER COMPONENTS (host is transparent)
record component-metadata {
    // === IDENTITY ===
    name: string,
    version: string,
    description: string,
    author: string,
    license: string,
    
    // === RUNTIME REQUIREMENTS (Host Runtime) ===
    /// Explicit capability requirements (filesystem, network, etc.)
    required-capabilities: list<capability>,
    
    /// Operation types this component handles (e.g., ["process-data", "transform-image"])
    supported-operations: list<string>,
    
    /// Multicodec formats supported for execute() input/output
    supported-codecs: list<multicodec-id>,
    
    // === RUNTIME CHARACTERISTICS (Host Runtime) ===
    /// Source language: rust, javascript, go, python, etc.
    language: string,
    
    /// Memory requirements for this component
    memory-requirements: memory-requirements,
    
    /// Maximum execution time in milliseconds (safety timeout limit)
    timeout-ms: option<u64>,
    
    /// Does component maintain state between execute() calls?
    stateful: bool,
    
    // === TECHNICAL METADATA (Host Runtime) ===
    /// WIT interface version compatibility (e.g., "1.0.0")
    api-version: string,
    
    // === DISCOVERY & DOCUMENTATION (Components + Humans) ===
    /// Project homepage URL
    homepage: option<string>,
    
    /// Source code repository URL
    repository: option<string>,
    
    /// Searchable tags for categorization (e.g., ["ai", "ml", "image-processing"])
    tags: list<string>,
    
    // === COMPONENT-TO-COMPONENT METADATA (Other Components) ===
    /// Domain-specific metadata for component discovery and capability negotiation.
    /// 
    /// **AUDIENCE**: This field is intended for OTHER COMPONENTS/PLUGINS, not the host runtime.
    /// The host stores and returns this data but does NOT interpret or act on it.
    /// 
    /// **PURPOSE**: 
    /// - Enable component discovery and capability negotiation
    /// - Advertise domain-specific characteristics (ML models, data formats, protocols)
    /// - Support dynamic component composition and routing decisions
    /// 
    /// **HOST BEHAVIOR**:
    /// ✅ Host WILL: Store, return via get-component-metadata(), preserve key-value pairs
    /// ❌ Host WILL NOT: Parse, validate, or make decisions based on these values
    /// 
    /// **VALID USE CASES**:
    /// - Data format capabilities: [("input-formats", "csv,json,parquet")]
    /// - ML model characteristics: [("model-type", "llm"), ("context-window", "8192")]
    /// - Protocol details: [("db-type", "postgresql"), ("protocol-version", "3.0")]
    /// - Processing hints: [("batch-size-hint", "1000"), ("supports-streaming", "true")]
    /// 
    /// **INVALID USE CASES** (use standard fields instead):
    /// - ❌ Runtime configuration → use component-config in init()
    /// - ❌ Resource limits → use memory-requirements, timeout-ms
    /// - ❌ Security capabilities → use required-capabilities
    /// 
    /// **RECOMMENDATIONS**:
    /// - Use namespaced keys for clarity: "ml:model-type", "db:protocol"
    /// - Document your custom-metadata schema in component README
    /// - Consider domain-specific WIT extensions for complex typed metadata
    /// - Keep values simple (strings, numbers as strings)
    custom-metadata: option<list<tuple<string, string>>>,
}

/// Memory requirements for component
record memory-requirements {
    /// Minimum memory in bytes to function
    min-memory-bytes: u64,
    
    /// Maximum memory in bytes allowed
    max-memory-bytes: u64,
    
    /// Optimal memory allocation in bytes
    preferred-memory-bytes: u64,
}

/// Component world - what components export and import
world component {
    /// Required exports - all components MUST implement
    export component-lifecycle;
    
    /// Standard imports - always available to components
    import airssys:host-core/services.{host-services};
    import airssys:multicodec-core/codec.{multicodec-utilities};
    
    /// Optional imports - based on component needs
    import airssys:host-core/capabilities.{host-capabilities};
}
```

### Host Services Interface (core/host.wit)
```wit
package airssys:host-core@1.0.0;

/// Core services always available to ALL components
interface host-services {
    /// Structured logging with context
    log: func(
        level: log-level, 
        message: string, 
        context: option<list<tuple<string, string>>>
    );
    
    /// Inter-component messaging through airssys-rt
    send-message: func(
        target: component-id, 
        message: list<u8>    // Multicodec-encoded message
    ) -> result<_, messaging-error>;
    
    /// Receive messages from other components
    receive-message: func() -> result<option<incoming-message>, messaging-error>;
    
    /// Configuration access
    get-config: func(key: string) -> result<option<string>, config-error>;
    set-config: func(key: string, value: string) -> result<_, config-error>;
    
    /// Time and timing services
    current-time-millis: func() -> u64;
    sleep-millis: func(duration: u64);
    
    /// Component introspection
    list-components: func() -> list<component-id>;
    get-component-metadata: func(id: component-id) -> result<component-metadata, component-error>;
}

/// Advanced host capabilities (capability-gated)
interface host-capabilities {
    /// File system operations (requires filesystem capabilities)
    read-file: func(path: string) -> result<list<u8>, file-error>;
    write-file: func(path: string, data: list<u8>) -> result<_, file-error>;
    
    /// Network operations (requires network capabilities)
    http-request: func(request: http-request) -> result<http-response, http-error>;
    
    /// Custom host functions (extensible)
    call-host-function: func(
        function-name: string,
        args: list<u8>      // Multicodec-encoded arguments
    ) -> result<list<u8>, host-function-error>;
}
```

### Multicodec Support (core/multicodec.wit)
```wit
package airssys:multicodec-core@1.0.0;

/// Multicodec utilities for self-describing data
interface multicodec-utilities {
    /// Encode data with multicodec prefix
    encode: func(codec: multicodec-id, data: list<u8>) -> list<u8>;
    
    /// Decode multicodec-prefixed data
    decode: func(encoded: list<u8>) -> result<decoded-data, multicodec-error>;
    
    /// Check if codec is supported by framework
    is-codec-supported: func(codec: multicodec-id) -> bool;
    
    /// Get information about a codec
    get-codec-info: func(codec: multicodec-id) -> result<codec-info, multicodec-error>;
}

/// Standard multicodecs supported by framework
enum standard-multicodec {
    /// Binary formats (efficient)
    borsh = 0x701,                    // Reserved for airssys-wasm
    bincode = 0x702,                  // Reserved for airssys-wasm
    protobuf = 0x50,                  // Official multicodec
    msgpack = 0x0201,                 // Official multicodec
    
    /// Text formats (debugging/human-readable)
    json = 0x0200,                    // Official multicodec
    yaml = 0x0114,                    // Official multicodec
    
    /// Raw bytes (no interpretation)
    raw = 0x55,                       // Official multicodec
}
```

## Component Development Patterns

### Universal Component Manifest (component.toml)
```toml
[component]
name = "my-ai-processor"
version = "1.0.0"
description = "AI-powered data processing component"
authors = ["engineer@company.com"]
license = "MIT"
language = "rust"                   # rust, javascript, go, python, etc.

[wit-dependencies]
# Required core interfaces (ALL components must include these)
airssys-component-core = "1.0.0"
airssys-host-core = "1.0.0"
airssys-multicodec-core = "1.0.0"

# Optional extension interfaces
airssys-ai-extensions = "0.5.0"
airssys-database-extensions = "0.3.0"

[multicodec]
primary-format = "borsh"            # Preferred multicodec for this component
supported-formats = ["borsh", "json", "msgpack"]

[capabilities]
# Capabilities this component requires from host
required = [
    "filesystem:read:/models",
    "network:outbound:api.huggingface.co",
    "ai:model:load",
]

[metadata]
entrypoint = "execute"              # WASM export function name
memory-limit = "256MB"
execution-timeout = "60s"

# Operations this component supports
expected-operations = [
    "process-text",
    "analyze-sentiment",
    "get-status",
]
```

### Host Runtime Implementation (Rust)
```rust
// airssys-wasm crate for host developers (always Rust)
use airssys_wasm::{ComponentRuntime, RuntimeConfig, ComponentId};
use airssys_osl::SecurityContext;
use airssys_rt::ActorSystem;

pub struct HostApplication {
    wasm_runtime: ComponentRuntime,
    actor_system: ActorSystem,
    security_context: SecurityContext,
}

impl HostApplication {
    pub fn new() -> Result<Self> {
        // Configure WASM runtime with deep AirsSys integration
        let runtime_config = RuntimeConfig::builder()
            .with_osl_security_integration()    // airssys-osl security
            .with_rt_actor_integration()        // airssys-rt actors
            .with_multicodec_support()          // Multicodec framework
            .with_capability_enforcement()      // Capability-based security
            .build()?;
            
        let wasm_runtime = ComponentRuntime::new(runtime_config)?;
        
        Ok(Self {
            wasm_runtime,
            actor_system: ActorSystem::new()?,
            security_context: SecurityContext::new()?,
        })
    }
    
    pub async fn load_component(&mut self, path: &str) -> Result<ComponentId> {
        // Parse component manifest
        let manifest = ComponentManifest::from_wasm_file(path)?;
        
        // Validate capabilities against security policy
        self.security_context.validate_capabilities(&manifest.capabilities)?;
        
        // Load component as actor in airssys-rt
        let component_id = self.wasm_runtime.load_component_as_actor(
            path,
            &manifest,
            &self.actor_system,
            &self.security_context
        ).await?;
        
        Ok(component_id)
    }
    
    pub async fn execute_component(
        &self, 
        component_id: ComponentId, 
        operation: &[u8]  // Already multicodec-encoded
    ) -> Result<Vec<u8>> {
        // Route through airssys-rt message passing
        self.wasm_runtime.send_message_to_component(component_id, operation).await
    }
}
```

## Component Development Workflows

### Rust Component Developer (Enhanced with Cargo Plugin)
```bash
# Install cargo plugin for best Rust experience
cargo install cargo-airssys-wasm

# Create new Rust component with full scaffolding
cargo airssys-wasm new my-ai-processor
# Generates:
# - Cargo.toml with proper WASM configuration
# - component.toml with WIT dependencies
# - src/lib.rs with component trait implementation boilerplate
# - wit/ directory with local interface copies
# - tests/ directory with component test templates

# Add WIT extension dependencies
cargo airssys-wasm add-wit airssys:ai-extensions@0.5.0

# Build component WASM
cargo airssys-wasm build

# Test component (includes multicodec tests)
cargo airssys-wasm test

# Package for distribution
cargo airssys-wasm package

# Publish to git repository
cargo airssys-wasm publish --git https://github.com/myorg/my-ai-processor
```

### JavaScript Component Developer (Universal Tooling)
```bash
# Create component structure using universal tooling
airssys-wasm component new my-js-processor --lang javascript

# Generated:
# - component.toml with basic configuration
# - package.json with componentize-js setup
# - wit/ directory with interface definitions
# - src/component.js with basic implementation template

# Implement component logic using standard JavaScript tools
npm install msgpack-lite uuid

# Build using standard JavaScript tooling
npm run build

# Test using standard JavaScript testing
npm test

# Package and distribute via git
git tag v1.0.0 && git push origin v1.0.0
```

### Go Component Developer (Universal Tooling)
```bash
# Create component structure
airssys-wasm component new my-go-processor --lang go

# Add dependencies using standard go modules
go get github.com/vmihailenco/msgpack/v5

# Build using standard Go tooling
go build -o component.wasm

# Test using standard Go testing
go test
```

## Distribution and Dependency Management

### Git-Based Distribution Strategy
```
Component Repository Structure:
github.com/company/my-component/
├── component.toml              # Universal component manifest
├── wit/                        # Component-specific WIT interfaces (optional)
├── src/                        # Implementation (any language)
├── tests/                      # Component tests
├── examples/                   # Usage examples
├── README.md                   # Getting started guide
└── releases/                   # Tagged WASM releases
    └── v1.0.0/
        └── component.wasm      # Built WASM component

WIT Interface Repository:
github.com/airssys/wasm-wit-interfaces/
├── core/                       # Core required interfaces
├── extensions/                 # Optional domain extensions
└── releases/                   # Versioned interface releases
```

### Dependency Resolution Strategy
```
WIT Dependency Resolution Order:
1. Local wit/ directory in component (highest priority)
2. Git repositories specified in component.toml
3. Future: Centralized WIT registry (lowest priority)

Version Resolution:
- Semantic versioning (major.minor.patch)
- Major version: Breaking changes (must match exactly)
- Minor version: Backward compatible (>= required version)
- Patch version: Bug fixes only (>= required version)
```

## Implementation Phases

### Phase 1: Core Foundation (Weeks 1-6)
**Goal**: Establish fundamental framework with basic functionality

**Deliverables**:
- Core WIT interfaces (component, host, multicodec)
- Basic Rust host runtime with airssys-osl/rt integration
- Universal component manifest (component.toml)
- Basic multicodec support (borsh, json, raw)
- Universal CLI tooling for component creation

### Phase 2: Enhanced Tooling (Weeks 7-12)
**Goal**: Improve developer experience with language-specific tooling

**Deliverables**:
- Cargo plugin for Rust component developers
- JavaScript/Node.js component templates and build integration
- Go component templates and build integration
- Component testing framework
- Documentation and examples

### Phase 3: Extensions and Advanced Features (Weeks 13-18)
**Goal**: Add domain-specific capabilities and advanced runtime features

**Deliverables**:
- Extension WIT interfaces (AI, database, filesystem, network, crypto)
- Advanced host capabilities implementation
- Performance optimization and resource management
- Enhanced security and sandboxing
- Component composition and pipeline support

### Phase 4: Ecosystem and Production (Weeks 19-24)
**Goal**: Production deployment and ecosystem growth

**Deliverables**:
- Production monitoring and observability
- Component registry infrastructure (optional)
- Advanced debugging and profiling tools
- Community documentation and tutorials
- Real-world deployment examples

## Key Success Factors

### Technical Excellence
- **Standards Compliance**: Built on proven WIT and multicodec standards
- **Performance**: Near-native execution with minimal overhead
- **Security**: Capability-based security with deny-by-default policies
- **Reliability**: Robust error handling and fault tolerance

### Developer Experience
- **Language Freedom**: No SDK lock-in, use native language tools
- **Enhanced Rust Support**: Optional cargo plugin for best-in-class experience
- **Clear Documentation**: Comprehensive guides and examples
- **Testing Framework**: Automated testing for components and multicodec compatibility

### Ecosystem Growth
- **Git-Based Distribution**: Simple, familiar component sharing
- **Extension Points**: Easy addition of domain-specific capabilities
- **Community Friendly**: Open standards and contribution-friendly architecture
- **Future Proof**: Evolutionary compatibility through multicodec

---

**Status**: Complete WIT management architecture plan
**Next Steps**: Begin Phase 1 implementation with core WIT interfaces and basic runtime