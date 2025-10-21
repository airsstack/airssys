# [WASM-TASK-000] - Core Abstractions Design ⚡ CRITICAL FOUNDATION

**Status:** in-progress (Phases 1 & 2 complete, Phase 3 next)  
**Added:** 2025-10-21  
**Updated:** 2025-10-21  
**Priority:** CRITICAL - Absolute Foundation (Layer 0)  
**Layer:** 0 - Foundation  
**Block:** 0 (Pre-Implementation)  
**Estimated Effort:** 3-4 weeks (expanded from 1-2 weeks per ADR-WASM-012)  
**Progress:** 30% (4/12 phases complete)  

## Overview

**⚡ CRITICAL FOUNDATION - MUST COMPLETE BEFORE ALL OTHER IMPLEMENTATION**

Design and implement the `core/` module containing **comprehensive abstractions for ALL implementation blocks (Blocks 1-11)**. This module establishes the type system, trait contracts, error handling, and configuration patterns that ALL other modules will depend on. Following ADR-WASM-012 (Comprehensive Core Abstractions Strategy), this task now includes domain-specific abstractions for runtime, WIT interfaces, actor integration, security, messaging, storage, lifecycle, component management, OSL bridge, and observability.

## Context

### Current State
- **Architecture complete**: 10 ADRs defining technical decisions (including ADR-WASM-012)
- **Knowledge base**: 12 knowledge docs covering all major patterns
- **Module structure**: ADR-WASM-011 defines hybrid block-aligned structure with core/
- **Comprehensive strategy**: ADR-WASM-012 mandates domain abstractions for all blocks
- **Problem**: Core abstractions NOT yet designed or implemented
- **Risk**: Without strong comprehensive core, implementation blocks will create inconsistent types and circular dependencies

### Why Core Abstractions Matter

**The Foundation Principle:**
> "A system is only as strong as its core abstractions. Weak foundations lead to brittle implementations, refactoring nightmares, and architectural tech debt."

**Without Strong Core:**
- ❌ Each block defines its own types (inconsistency, duplication)
- ❌ Circular dependencies between modules (unmaintainable)
- ❌ Refactoring requires changes across entire codebase
- ❌ Testing becomes difficult due to tight coupling
- ❌ API stability is impossible to maintain

**With Strong Core:**
- ✅ Single source of truth for fundamental types
- ✅ Clear contracts via traits (testable, mockable)
- ✅ Zero circular dependencies (core depends on nothing internal)
- ✅ Refactoring isolated to specific modules
- ✅ Stable public API from day one

### Strategic Importance

This task is **more critical than WASM-TASK-001 (planning)** because:
1. Planning requires understanding what core types exist
2. Implementation tasks (002-012) all depend on core abstractions
3. Getting core wrong means refactoring 11 blocks later
4. Core types define the "language" of the system

**Analogy:**
- Core abstractions = **Language grammar and vocabulary**
- Implementation blocks = **Sentences and paragraphs**
- You must define grammar before writing sentences

## Objectives

### Primary Objective
Design and implement a minimal, stable, well-documented `core/` module containing all foundational abstractions (traits, types, errors, configs) required by ALL implementation blocks (1-11), ensuring zero internal dependencies and maximum stability.

### Secondary Objectives
- Establish type safety patterns for the entire crate
- Define error handling strategy (thiserror-based)
- Create configuration type patterns
- Document all core abstractions comprehensively
- Validate core design through review before implementation

## Scope

### In Scope
1. **Core Module Structure** - `core/` directory organization
2. **Component Abstractions** - Component trait, ComponentId, ComponentMetadata, ComponentConfig
3. **Capability Abstractions** - Capability enum, CapabilitySet, capability patterns
4. **Error Types** - WasmError enum, WasmResult type, error context patterns
5. **Configuration Types** - RuntimeConfig, ComponentConfig, configuration patterns
6. **Common Types** - Shared types used by 3+ modules
7. **Core Traits** - Fundamental behavior contracts
8. **Documentation** - Comprehensive rustdoc for all public types

### Out of Scope
- Implementation logic (belongs in domain modules)
- Integration code (belongs in actor/, osl/ modules)
- Feature-specific types (belongs in respective blocks)
- Helper functions (belongs in util/ module)
- Performance optimization (premature at this stage)

## Design Principles

### Principle 1: Minimalism (YAGNI)
- ✅ Include ONLY types needed by multiple modules
- ✅ Defer feature-specific types to domain modules
- ❌ No speculative abstractions "we might need later"
- ❌ No over-engineered type hierarchies

### Principle 2: Stability First
- ✅ Core types should rarely change (semver major bump if they do)
- ✅ Design for evolution (use traits for extensibility)
- ✅ Comprehensive documentation (prevent misuse)
- ❌ No experimental APIs in core

### Principle 3: Zero Internal Dependencies
- ✅ Core depends ONLY on external crates (serde, thiserror, chrono)
- ✅ All other modules can depend on core
- ❌ Core NEVER depends on any airssys-wasm module
- ❌ Prevents all circular dependency issues

### Principle 4: Type Safety Over Convenience
- ✅ Use newtype pattern for IDs (ComponentId vs String)
- ✅ Use enums for known variants (not stringly-typed)
- ✅ Non-null types by default (Option only when truly optional)
- ❌ No `dyn` trait objects in core (§6.2 workspace standards)

### Principle 5: Clear Error Semantics
- ✅ Structured errors with context (thiserror)
- ✅ Error types map to failure modes
- ✅ Errors carry actionable information
- ❌ No generic "Error" or "Failed" variants

## Core Abstractions Specification

### 1. Component Abstractions (`core/component.rs`)

**Purpose:** Define what a WASM component IS and how it behaves.

**Core Types:**
```rust
/// Unique identifier for a component instance.
/// 
/// Uses newtype pattern to prevent accidental string misuse.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(String);

impl ComponentId {
    /// Create a new component ID from a string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    /// Get the ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Component metadata describing a component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    /// Component name (e.g., "image-processor")
    pub name: String,
    
    /// Semantic version (e.g., "1.2.3")
    pub version: String,
    
    /// Component author/publisher
    pub author: String,
    
    /// Optional description
    pub description: Option<String>,
    
    /// Required capabilities
    pub required_capabilities: Vec<Capability>,
    
    /// Resource limits from Component.toml
    pub resource_limits: ResourceLimits,
}

/// Resource limits for component execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory in bytes (REQUIRED from Component.toml)
    pub max_memory_bytes: u64,
    
    /// Maximum fuel per execution (CPU limiting)
    pub max_fuel: u64,
    
    /// Maximum execution time in milliseconds (wall-clock timeout)
    pub max_execution_ms: u64,
    
    /// Maximum storage quota in bytes
    pub max_storage_bytes: u64,
}

/// Component input for execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInput {
    /// Input data (multicodec-encoded)
    pub data: Vec<u8>,
    
    /// Multicodec prefix identifying format
    pub codec: u64,
    
    /// Optional metadata
    pub metadata: HashMap<String, String>,
}

/// Component output from execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentOutput {
    /// Output data (multicodec-encoded)
    pub data: Vec<u8>,
    
    /// Multicodec prefix identifying format
    pub codec: u64,
    
    /// Optional metadata
    pub metadata: HashMap<String, String>,
}

/// Component configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentConfig {
    /// Component ID
    pub id: ComponentId,
    
    /// Component metadata
    pub metadata: ComponentMetadata,
    
    /// Installation source (Git, File, URL)
    pub source: InstallationSource,
    
    /// Component state (Installed, Uninstalled)
    pub state: ComponentState,
}

/// Installation source for components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallationSource {
    /// Git repository (reproducible builds)
    Git {
        url: String,
        commit: String,
    },
    /// Local file path (fast development)
    File {
        path: PathBuf,
    },
    /// Remote URL (pre-built, offline-capable)
    Url {
        url: String,
    },
}

/// Component lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentState {
    /// Component is installed and can be executed
    Installed,
    /// Component is uninstalled (awaiting cleanup or already removed)
    Uninstalled,
}
```

**Core Trait:**
```rust
/// Core component trait defining component behavior.
/// 
/// All WASM components implement this trait (through generated bindings).
/// Host runtime calls these methods during component lifecycle.
pub trait Component {
    /// Initialize component with configuration.
    /// 
    /// Called once when component is first loaded.
    fn init(&mut self, config: ComponentConfig) -> Result<(), WasmError>;
    
    /// Execute component with input, producing output.
    /// 
    /// This is the main entry point for component logic.
    fn execute(&self, input: ComponentInput) -> Result<ComponentOutput, WasmError>;
    
    /// Shutdown component gracefully.
    /// 
    /// Called when component is being unloaded or system is shutting down.
    fn shutdown(&mut self) -> Result<(), WasmError>;
    
    /// Get component metadata.
    fn metadata(&self) -> &ComponentMetadata;
}
```

**Design Rationale:**
- ✅ ComponentId newtype prevents string confusion
- ✅ ResourceLimits enforces ADR-WASM-002 (mandatory limits)
- ✅ Multicodec integration (ComponentInput/Output) follows ADR-WASM-001
- ✅ Component trait is minimal (init, execute, shutdown, metadata)
- ✅ InstallationSource enum follows ADR-WASM-003 (3 sources)
- ✅ ComponentState enum follows ADR-WASM-003 (2-state lifecycle)

---

### 2. Capability Abstractions (`core/capability.rs`)

**Purpose:** Define capability-based security model primitives.

**Core Types:**
```rust
/// Fine-grained capability for component permissions.
/// 
/// Capabilities follow pattern-based matching (globs, domains, namespaces)
/// as defined in ADR-WASM-005.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    /// Filesystem read access (glob pattern)
    FileRead(PathPattern),
    
    /// Filesystem write access (glob pattern)
    FileWrite(PathPattern),
    
    /// Network outbound connection (domain pattern)
    NetworkOutbound(DomainPattern),
    
    /// Network inbound listener (port)
    NetworkInbound(u16),
    
    /// Storage access (namespace pattern)
    Storage(NamespacePattern),
    
    /// Process spawn capability
    ProcessSpawn,
    
    /// Inter-component messaging (topic pattern)
    Messaging(TopicPattern),
    
    /// Custom capability (extensible)
    Custom {
        name: String,
        parameters: serde_json::Value,
    },
}

/// Path pattern for filesystem capabilities (supports globs).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PathPattern(String);

impl PathPattern {
    /// Create a new path pattern.
    pub fn new(pattern: impl Into<String>) -> Self {
        Self(pattern.into())
    }
    
    /// Check if a path matches this pattern.
    pub fn matches(&self, path: &Path) -> bool {
        // Implementation will use glob matching (in security/ module)
        todo!()
    }
}

/// Domain pattern for network capabilities (supports wildcards).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DomainPattern(String);

impl DomainPattern {
    pub fn new(pattern: impl Into<String>) -> Self {
        Self(pattern.into())
    }
    
    pub fn matches(&self, domain: &str) -> bool {
        todo!()
    }
}

/// Namespace pattern for storage capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NamespacePattern(String);

impl NamespacePattern {
    pub fn new(pattern: impl Into<String>) -> Self {
        Self(pattern.into())
    }
    
    pub fn matches(&self, namespace: &str) -> bool {
        todo!()
    }
}

/// Topic pattern for messaging capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TopicPattern(String);

impl TopicPattern {
    pub fn new(pattern: impl Into<String>) -> Self {
        Self(pattern.into())
    }
    
    pub fn matches(&self, topic: &str) -> bool {
        todo!()
    }
}

/// Set of capabilities granted to a component.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilitySet {
    capabilities: HashSet<Capability>,
}

impl CapabilitySet {
    /// Create an empty capability set.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create a capability set from a vector.
    pub fn from_vec(capabilities: Vec<Capability>) -> Self {
        Self {
            capabilities: capabilities.into_iter().collect(),
        }
    }
    
    /// Add a capability to the set.
    pub fn grant(&mut self, capability: Capability) {
        self.capabilities.insert(capability);
    }
    
    /// Remove a capability from the set.
    pub fn revoke(&mut self, capability: &Capability) {
        self.capabilities.remove(capability);
    }
    
    /// Check if a capability is granted.
    pub fn has(&self, capability: &Capability) -> bool {
        self.capabilities.contains(capability)
    }
    
    /// Check if any capability in the set matches the given capability.
    /// 
    /// Uses pattern matching (e.g., FileRead("/data/*") matches FileRead("/data/file.txt"))
    pub fn matches(&self, capability: &Capability) -> bool {
        // Implementation in security/ module
        todo!()
    }
    
    /// Iterate over all capabilities.
    pub fn iter(&self) -> impl Iterator<Item = &Capability> {
        self.capabilities.iter()
    }
}
```

**Design Rationale:**
- ✅ Capability enum follows ADR-WASM-005 (fine-grained patterns)
- ✅ Pattern types (PathPattern, DomainPattern) enable security/ module validation
- ✅ CapabilitySet provides ergonomic API for capability management
- ✅ Custom variant allows extensibility (Phase 2+)
- ✅ Newtype patterns for type safety

---

### 3. Error Types (`core/error.rs`)

**Purpose:** Define comprehensive error handling for entire crate.

**Core Types:**
```rust
use thiserror::Error;

/// Comprehensive error type for airssys-wasm operations.
/// 
/// Following workspace standards and Microsoft Rust Guidelines (M-ERRORS-CANONICAL-STRUCTS).
#[derive(Error, Debug)]
pub enum WasmError {
    /// Component loading failed
    #[error("Failed to load component '{component_id}': {reason}")]
    ComponentLoadFailed {
        component_id: String,
        reason: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Component execution failed
    #[error("Component execution failed: {reason}")]
    ExecutionFailed {
        reason: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Component trapped (WASM trap)
    #[error("Component trapped: {reason}")]
    ComponentTrapped {
        reason: String,
        fuel_consumed: Option<u64>,
    },
    
    /// Execution timeout exceeded
    #[error("Execution timeout exceeded ({max_execution_ms}ms)")]
    ExecutionTimeout {
        max_execution_ms: u64,
        fuel_consumed: Option<u64>,
    },
    
    /// Resource limit exceeded
    #[error("Resource limit exceeded: {resource}")]
    ResourceLimitExceeded {
        resource: String,
        limit: u64,
        attempted: u64,
    },
    
    /// Capability denied
    #[error("Capability denied: {capability:?}")]
    CapabilityDenied {
        capability: Capability,
        reason: String,
    },
    
    /// Invalid configuration
    #[error("Invalid configuration: {reason}")]
    InvalidConfiguration {
        reason: String,
    },
    
    /// Component not found
    #[error("Component not found: {component_id}")]
    ComponentNotFound {
        component_id: String,
    },
    
    /// Storage error
    #[error("Storage error: {reason}")]
    StorageError {
        reason: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Messaging error
    #[error("Messaging error: {reason}")]
    MessagingError {
        reason: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Actor system error
    #[error("Actor system error: {reason}")]
    ActorError {
        reason: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// I/O error (filesystem, network)
    #[error("I/O error: {operation}")]
    IoError {
        operation: String,
        #[source]
        source: std::io::Error,
    },
    
    /// Serialization error
    #[error("Serialization error: {reason}")]
    SerializationError {
        reason: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Internal error (should not happen in normal operation)
    #[error("Internal error: {reason}")]
    Internal {
        reason: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

/// Result type alias for airssys-wasm operations.
pub type WasmResult<T> = Result<T, WasmError>;

impl WasmError {
    /// Create a component load error.
    pub fn component_load_failed(
        component_id: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::ComponentLoadFailed {
            component_id: component_id.into(),
            reason: reason.into(),
            source: None,
        }
    }
    
    /// Create a capability denied error.
    pub fn capability_denied(
        capability: Capability,
        reason: impl Into<String>,
    ) -> Self {
        Self::CapabilityDenied {
            capability,
            reason: reason.into(),
        }
    }
    
    /// Create an execution timeout error.
    pub fn execution_timeout(
        max_execution_ms: u64,
        fuel_consumed: Option<u64>,
    ) -> Self {
        Self::ExecutionTimeout {
            max_execution_ms,
            fuel_consumed,
        }
    }
    
    // Additional helper constructors...
}
```

**Design Rationale:**
- ✅ thiserror for structured errors (workspace standard)
- ✅ Comprehensive variants covering all failure modes
- ✅ Error messages are actionable (include context)
- ✅ Source error chaining for debugging
- ✅ Helper constructors for common cases
- ✅ Follows M-ERRORS-CANONICAL-STRUCTS (Microsoft Rust Guidelines)

---

### 4. Configuration Types (`core/config.rs`)

**Purpose:** Define configuration patterns for runtime and components.

**Core Types:**
```rust
/// Runtime configuration for WASM engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Enable async execution (default: true)
    pub async_enabled: bool,
    
    /// Enable fuel metering (default: true)
    pub fuel_metering_enabled: bool,
    
    /// Default fuel limit (can be overridden per component)
    pub default_max_fuel: u64,
    
    /// Default execution timeout in milliseconds
    pub default_execution_timeout_ms: u64,
    
    /// Enable module caching for faster instantiation
    pub module_caching_enabled: bool,
    
    /// Maximum cached modules (LRU eviction)
    pub max_cached_modules: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            async_enabled: true,
            fuel_metering_enabled: true,
            default_max_fuel: 1_000_000,        // 1M fuel
            default_execution_timeout_ms: 100,   // 100ms
            module_caching_enabled: true,
            max_cached_modules: 100,
        }
    }
}

/// Security configuration for capability enforcement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Security mode (Strict, Permissive, Development)
    pub mode: SecurityMode,
    
    /// Audit logging enabled
    pub audit_logging: bool,
    
    /// Capability check timeout (microseconds)
    pub capability_check_timeout_us: u64,
}

/// Security enforcement mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityMode {
    /// Strict mode: All capabilities must be explicitly granted
    Strict,
    
    /// Permissive mode: Allows some auto-approval for trusted sources
    Permissive,
    
    /// Development mode: Bypass capability checks (DEV ONLY)
    Development,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            mode: SecurityMode::Strict,
            audit_logging: true,
            capability_check_timeout_us: 5, // 5μs target
        }
    }
}

/// Storage configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Storage backend (Sled, RocksDB)
    pub backend: StorageBackend,
    
    /// Storage directory path
    pub storage_path: PathBuf,
    
    /// Enable storage quotas
    pub quotas_enabled: bool,
}

/// Storage backend selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageBackend {
    /// Sled (pure Rust, default)
    Sled,
    
    /// RocksDB (production-proven, optional)
    RocksDB,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackend::Sled,
            storage_path: PathBuf::from("./airssys_wasm_storage"),
            quotas_enabled: true,
        }
    }
}
```

**Design Rationale:**
- ✅ Sensible defaults for all configs
- ✅ Configuration types are serializable (TOML/JSON support)
- ✅ Security mode enum enables different enforcement levels
- ✅ Storage backend enum follows ADR-WASM-007
- ✅ All configs have Default implementations

---

## Implementation Plan

**⚠️ CRITICAL: Detailed Phase 1 Action Plan Available**

For comprehensive step-by-step implementation guidance for Phase 1, see:
**`task_000_phase_1_action_plan.md`** - Complete action plan with:
- Detailed task breakdown (Tasks 1.1-2.3)
- Code examples and templates
- Success criteria for each task
- Quality checklists and validation steps
- References to all relevant ADRs and standards

The action plan below provides the overview. Follow the detailed action plan for implementation.

---

### Phase 1: Core Module Foundation (Days 1-4)

**Status:** ✅ COMPLETE (Oct 21, 2025)  
**Action Plan:** See `task_000_phase_1_action_plan.md` for detailed guidance  
**Completion Summary:** See `task_000_phase_1_completion_summary.md`

#### Task 1.1: Create Core Module Structure ✅
**Deliverables:**
- ✅ Created `src/core/` directory
- ✅ Created `src/core/mod.rs` with module declarations (declaration-only per §4.3)
- ✅ Created `component.rs` (560+ lines with comprehensive implementation)
- ✅ Updated `src/lib.rs` to include `pub mod core;`
- ✅ Verified compilation (`cargo check` passes with zero warnings)

**Success Criteria:**
- ✅ Core module compiles
- ✅ No circular dependency warnings (zero internal dependencies validated)
- ✅ Module structure follows ADR-WASM-011

#### Task 1.2: Add External Dependencies ✅
**Deliverables:**
- ✅ Added workspace dependencies to `Cargo.toml`:
  - ✅ `serde = { workspace = true }` (serialization)
  - ✅ `thiserror = { workspace = true }` (error handling)
  - ✅ `chrono = { workspace = true }` (timestamps per §3.2)
  - ✅ `async-trait = { workspace = true }` (async traits - fixed to workspace per §5.1)
  - ✅ `serde_json = { workspace = true }` (dev dependency)
- ✅ Verified dependencies resolve (`cargo check` passes)

**Success Criteria:**
- ✅ All dependencies added following §5.1 workspace pattern
- ✅ No version conflicts
- ✅ Compilation successful with zero warnings

---

### Phase 2: Component Abstractions (Days 3-4)

**Status:** ✅ COMPLETE (Oct 21, 2025)  
**Note:** Phase 1 Action Plan included both Phase 1 and Phase 2 tasks

#### Task 2.1: Implement Component Types ✅
**Deliverables:**
- ✅ Implemented `ComponentId` newtype with `new()` and `as_str()`
- ✅ Implemented `ComponentMetadata` struct (name, version, author, description, capabilities, limits)
- ✅ Implemented `ResourceLimits` struct (all 4 mandatory limits per ADR-WASM-002)
- ✅ Implemented `ComponentInput` and `ComponentOutput` structs (multicodec support per ADR-WASM-001)
- ✅ Implemented `ComponentConfig` struct
- ✅ Implemented `InstallationSource` enum (Git, File, Url per ADR-WASM-003)
- ✅ Implemented `ComponentState` enum (Installed, Uninstalled per ADR-WASM-003)
- ✅ Added comprehensive rustdoc for all types (100% coverage)

**Success Criteria:**
- ✅ All types compile with zero warnings
- ✅ Derive macros work (Debug, Clone, Serialize, Deserialize)
- ✅ Rustdoc examples compile and run (9 doc tests passing)

#### Task 2.2: Implement Component Trait ✅
**Deliverables:**
- ✅ Defined `Component` trait with 4 methods: init, execute, shutdown, metadata
- ✅ Added comprehensive trait documentation with lifecycle diagram
- ✅ Added usage examples in documentation

**Success Criteria:**
- ✅ Trait compiles
- ✅ Documentation is comprehensive
- ✅ Examples are clear and correct (validated via doc tests)

#### Task 2.3: Unit Tests for Component Types ✅
**Deliverables:**
- ✅ Test ComponentId creation and equality (3 tests)
- ✅ Test serialization/deserialization for all types (6 tests)
- ✅ Test InstallationSource variants (3 tests)
- ✅ Test ComponentState transitions (2 tests)
- ✅ Test Component trait implementation (MockComponent - 3 tests)
- ✅ Total: 17 unit tests + 9 doc tests = 26 tests (all passing)

**Success Criteria:**
- ✅ All tests pass (26/26)
- ✅ >90% code coverage for component types (achieved)

---

### Phase 3: Capability Abstractions (Days 5-6) ✅ COMPLETE - Oct 21, 2025

#### Task 3.1: Implement Capability Types ✅
**Deliverables:**
- ✅ Implement `Capability` enum with all variants (8 variants: FileRead, FileWrite, NetworkOutbound, NetworkInbound, Storage, ProcessSpawn, Messaging, Custom)
- ✅ Implement pattern types: PathPattern, DomainPattern, NamespacePattern, TopicPattern
- ✅ Implement `CapabilitySet` with grant/revoke/has/matches/iter/len/is_empty/from_vec methods
- ✅ Add rustdoc with examples (100% coverage, 29 doc tests)

**Success Criteria:**
- ✅ All types compile (zero warnings)
- ✅ CapabilitySet API is ergonomic (8 methods, fluent API)
- ✅ Pattern types use newtype pattern correctly (all 4 patterns)

#### Task 3.2: Unit Tests for Capability Types ✅
**Deliverables:**
- ✅ Test Capability variant creation (all 8 variants tested)
- ✅ Test CapabilitySet operations (grant, revoke, has, matches, iter)
- ✅ Test serialization/deserialization (JSON format)
- ✅ Test equality and hashing (for HashSet compatibility)

**Success Criteria:**
- ✅ All tests pass (45 tests: 16 unit + 29 doc)
- ✅ >90% code coverage for capability types (100% coverage achieved)

**Implementation Summary:**
- File: `src/core/capability.rs` (844 lines)
- Integration: Replaced `pub type Capability = String;` placeholder in component.rs
- Dependencies: serde_json moved to main dependencies for Custom capability
- ADR Compliance: ADR-WASM-005 (Capability-Based Security Model) validated
- Quality: 71 total tests passing, zero warnings, 100% rustdoc

---

### Phase 4: Error Types (Days 7-8) ⏳ NEXT

#### Task 4.1: Implement WasmError Enum
**Deliverables:**
- Implement all WasmError variants with thiserror attributes
- Implement helper constructor methods
- Add comprehensive error documentation
- Add source error chaining examples

**Success Criteria:**
- All error variants compile
- Error messages are clear and actionable
- Helper constructors simplify error creation

#### Task 4.2: Unit Tests for Error Types
**Deliverables:**
- Test error creation with helper methods
- Test error message formatting
- Test source error chaining
- Test Debug and Display implementations

**Success Criteria:**
- All tests pass
- Error messages verified for clarity

---

### Phase 5: Configuration Types (Days 9-10)

#### Task 5.1: Implement Configuration Structs
**Deliverables:**
- Implement RuntimeConfig with Default
- Implement SecurityConfig with SecurityMode enum and Default
- Implement StorageConfig with StorageBackend enum and Default
- Add rustdoc for all configs

**Success Criteria:**
- All configs compile
- Default implementations provide sensible values
- Serialization/deserialization works

#### Task 5.2: Unit Tests for Configuration Types
**Deliverables:**
- Test Default implementations
- Test serialization to TOML/JSON
- Test deserialization from TOML/JSON
- Test enum variants

**Success Criteria:**
- All tests pass
- Configuration files parse correctly

---

### Phase 6: Domain-Specific Abstractions - Part 1: Runtime & Interface (Days 11-13)

**Purpose:** Implement domain-specific abstractions for Blocks 1-2 (runtime execution and WIT interfaces). These abstractions prevent circular dependencies and provide trait contracts for implementation blocks.

#### Task 6.1: Runtime Abstractions (`core/runtime.rs`) - Block 1

**Deliverables:**
- `trait RuntimeEngine` - Core execution engine contract
- `struct ExecutionContext` - Execution environment state
- `enum ExecutionState` - Runtime state machine
- `enum ExecutionResult` - Execution outcomes
- Comprehensive rustdoc

**Core Types:**
```rust
/// Core runtime engine trait for WASM execution.
/// 
/// Implemented by `runtime::WasmEngine` using Wasmtime.
pub trait RuntimeEngine: Send + Sync {
    /// Load a component from bytes.
    async fn load_component(
        &self,
        component_id: &ComponentId,
        bytes: &[u8],
    ) -> WasmResult<ComponentHandle>;
    
    /// Execute a component function.
    async fn execute(
        &self,
        handle: &ComponentHandle,
        function: &str,
        input: ComponentInput,
        context: ExecutionContext,
    ) -> WasmResult<ComponentOutput>;
    
    /// Get resource usage statistics.
    fn resource_usage(&self, handle: &ComponentHandle) -> ResourceUsage;
}

/// Execution context passed to runtime engine.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub component_id: ComponentId,
    pub limits: ResourceLimits,
    pub capabilities: CapabilitySet,
    pub timeout_ms: u64,
}

/// Component handle (opaque reference to loaded component).
#[derive(Debug, Clone)]
pub struct ComponentHandle(String); // Internal implementation detail

/// Runtime state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionState {
    Idle,
    Loading,
    Executing,
    Trapped,
    TimedOut,
    Completed,
}

/// Resource usage statistics.
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub memory_bytes: u64,
    pub fuel_consumed: u64,
    pub execution_time_ms: u64,
}
```

**Success Criteria:**
- RuntimeEngine trait compiles and is Send + Sync
- ExecutionContext provides all necessary runtime information
- All types have comprehensive rustdoc
- Unit tests for ExecutionContext creation and validation

#### Task 6.2: Interface Abstractions (`core/interface.rs`) - Block 2

**Deliverables:**
- `struct InterfaceDefinition` - WIT interface metadata
- `struct TypeDescriptor` - WIT type system representation
- `enum InterfaceKind` - Interface classification (import/export)
- `struct BindingMetadata` - Language binding information
- Comprehensive rustdoc

**Core Types:**
```rust
/// WIT interface definition metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceDefinition {
    pub name: String,
    pub version: String,
    pub kind: InterfaceKind,
    pub functions: Vec<FunctionSignature>,
    pub types: Vec<TypeDescriptor>,
}

/// Interface classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterfaceKind {
    /// Host functions exported to components
    Export,
    /// Component functions imported by host
    Import,
}

/// Function signature in WIT interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSignature {
    pub name: String,
    pub parameters: Vec<(String, TypeDescriptor)>,
    pub return_type: Option<TypeDescriptor>,
    pub required_capabilities: Vec<Capability>,
}

/// WIT type system representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeDescriptor {
    Bool,
    U8, U16, U32, U64,
    S8, S16, S32, S64,
    F32, F64,
    String,
    List(Box<TypeDescriptor>),
    Option(Box<TypeDescriptor>),
    Result { ok: Box<TypeDescriptor>, err: Box<TypeDescriptor> },
    Record { fields: Vec<(String, TypeDescriptor)> },
    Variant { cases: Vec<(String, Option<TypeDescriptor>)> },
}

/// Language binding metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingMetadata {
    pub language: String,
    pub binding_version: String,
    pub generator: String,
}
```

**Success Criteria:**
- All interface types compile and serialize properly
- TypeDescriptor covers complete WIT type system
- FunctionSignature includes capability requirements
- Unit tests for interface metadata creation

---

### Phase 7: Domain-Specific Abstractions - Part 2: Actor & Security (Days 14-16)

**Purpose:** Implement domain-specific abstractions for Blocks 3-4 (actor integration and security policies).

#### Task 7.1: Actor Abstractions (`core/actor.rs`) - Block 3

**Deliverables:**
- `struct ActorMessage` - Message envelope for actor system
- `enum SupervisionStrategy` - Supervisor behavior patterns
- `enum ActorState` - Actor lifecycle states
- `struct ActorMetadata` - Actor system metadata
- Comprehensive rustdoc

**Core Types:**
```rust
/// Message envelope for actor-based messaging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorMessage {
    pub from: ComponentId,
    pub to: ComponentId,
    pub message_id: String,
    pub correlation_id: Option<String>,
    pub payload: ComponentOutput,
    pub timestamp: DateTime<Utc>,
}

/// Supervision strategy for component actors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupervisionStrategy {
    /// Restart failed component
    Restart,
    /// Stop failed component
    Stop,
    /// Escalate to parent supervisor
    Escalate,
}

/// Actor lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorState {
    Initializing,
    Ready,
    Processing,
    Suspended,
    Terminating,
    Terminated,
}

/// Actor system metadata.
#[derive(Debug, Clone)]
pub struct ActorMetadata {
    pub actor_id: String,
    pub component_id: ComponentId,
    pub mailbox_size: usize,
    pub state: ActorState,
    pub restart_count: u32,
}
```

**Success Criteria:**
- ActorMessage supports actor communication patterns
- SupervisionStrategy covers all supervision modes
- ActorState represents complete lifecycle
- Unit tests for message creation and state transitions

#### Task 7.2: Security Abstractions (`core/security.rs`) - Block 4

**Deliverables:**
- `trait SecurityPolicy` - Security enforcement contract
- `struct PermissionRequest` - Permission check requests
- `enum PermissionResult` - Permission check outcomes
- `struct IsolationBoundary` - Sandbox boundary definition
- Comprehensive rustdoc

**Core Types:**
```rust
/// Security policy enforcement trait.
pub trait SecurityPolicy: Send + Sync {
    /// Check if a capability is allowed for a component.
    fn check_permission(
        &self,
        component_id: &ComponentId,
        capability: &Capability,
    ) -> WasmResult<PermissionResult>;
    
    /// Get all granted capabilities for a component.
    fn granted_capabilities(&self, component_id: &ComponentId) -> CapabilitySet;
}

/// Permission check request.
#[derive(Debug, Clone)]
pub struct PermissionRequest {
    pub component_id: ComponentId,
    pub capability: Capability,
    pub context: SecurityContext,
}

/// Permission check result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionResult {
    Allowed,
    Denied { reason: String },
    NeedsReview,
}

/// Security context for permission checks.
#[derive(Debug, Clone)]
pub struct SecurityContext {
    pub mode: SecurityMode,
    pub trust_level: TrustLevel,
    pub audit_enabled: bool,
}

/// Component trust level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustLevel {
    Trusted,      // Auto-approve capabilities
    Unknown,      // Require user review
    Development,  // Bypass checks (dev only)
}

/// Isolation boundary definition.
#[derive(Debug, Clone)]
pub struct IsolationBoundary {
    pub memory_isolated: bool,
    pub filesystem_isolated: bool,
    pub network_isolated: bool,
    pub allowed_paths: Vec<PathPattern>,
    pub allowed_domains: Vec<DomainPattern>,
}
```

**Success Criteria:**
- SecurityPolicy trait defines clear enforcement contract
- PermissionRequest/Result support complete permission workflow
- IsolationBoundary defines complete sandbox configuration
- Unit tests for permission checking logic

---

### Phase 8: Domain-Specific Abstractions - Part 3: Messaging & Storage (Days 17-19)

**Purpose:** Implement domain-specific abstractions for Blocks 5-6 (messaging and storage).

#### Task 8.1: Messaging Abstractions (`core/messaging.rs`) - Block 5

**Deliverables:**
- `struct MessageEnvelope` - Inter-component message container
- `enum MessageType` - Message pattern classification
- `trait RoutingStrategy` - Message routing contract
- `enum DeliveryGuarantee` - Delivery semantics
- Comprehensive rustdoc

**Core Types:**
```rust
/// Inter-component message envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    pub message_type: MessageType,
    pub from: ComponentId,
    pub to: ComponentId,
    pub topic: Option<String>,
    pub payload: Vec<u8>,
    pub codec: u64,
    pub message_id: String,
    pub correlation_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Message pattern type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Fire-and-forget one-way message
    FireAndForget,
    /// Request expecting response
    Request,
    /// Response to request
    Response,
    /// Pub-sub topic message
    Publish,
}

/// Message routing strategy trait.
pub trait RoutingStrategy: Send + Sync {
    /// Route a message to destination component.
    fn route(&self, envelope: &MessageEnvelope) -> WasmResult<()>;
}

/// Delivery guarantee semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryGuarantee {
    AtMostOnce,   // Fire-and-forget
    AtLeastOnce,  // May duplicate
    ExactlyOnce,  // Exactly once (future)
}
```

**Success Criteria:**
- MessageEnvelope supports all messaging patterns
- MessageType covers fire-and-forget, request-response, pub-sub
- RoutingStrategy provides clear routing contract
- Unit tests for message envelope creation

#### Task 8.2: Storage Abstractions (`core/storage.rs`) - Block 6

**Deliverables:**
- `trait StorageBackend` - Storage implementation contract
- `enum StorageOperation` - Operation types
- `struct StorageTransaction` - Transaction abstraction
- `enum StorageError` - Storage-specific errors
- Comprehensive rustdoc

**Core Types:**
```rust
/// Storage backend trait for component data persistence.
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Get value by key.
    async fn get(&self, namespace: &str, key: &[u8]) -> WasmResult<Option<Vec<u8>>>;
    
    /// Set key-value pair.
    async fn set(&self, namespace: &str, key: &[u8], value: &[u8]) -> WasmResult<()>;
    
    /// Delete key.
    async fn delete(&self, namespace: &str, key: &[u8]) -> WasmResult<()>;
    
    /// List keys with prefix.
    async fn list_keys(&self, namespace: &str, prefix: &[u8]) -> WasmResult<Vec<Vec<u8>>>;
    
    /// Begin transaction (if supported).
    async fn begin_transaction(&self) -> WasmResult<Box<dyn StorageTransaction>>;
}

/// Storage operation type.
#[derive(Debug, Clone)]
pub enum StorageOperation {
    Get { namespace: String, key: Vec<u8> },
    Set { namespace: String, key: Vec<u8>, value: Vec<u8> },
    Delete { namespace: String, key: Vec<u8> },
    List { namespace: String, prefix: Vec<u8> },
}

/// Transaction abstraction for atomic operations.
#[async_trait]
pub trait StorageTransaction: Send + Sync {
    /// Add operation to transaction.
    async fn add_operation(&mut self, op: StorageOperation) -> WasmResult<()>;
    
    /// Commit transaction.
    async fn commit(self: Box<Self>) -> WasmResult<()>;
    
    /// Rollback transaction.
    async fn rollback(self: Box<Self>) -> WasmResult<()>;
}
```

**Success Criteria:**
- StorageBackend trait supports complete KV operations
- StorageOperation enum covers all operation types
- StorageTransaction provides atomic operation support
- Unit tests for storage operation creation

---

### Phase 9: Domain-Specific Abstractions - Part 4: Lifecycle & Management (Days 20-22)

**Purpose:** Implement domain-specific abstractions for Blocks 7-8 (lifecycle and component management).

#### Task 9.1: Lifecycle Abstractions (`core/lifecycle.rs`) - Block 7

**Deliverables:**
- `enum LifecycleState` - Component lifecycle state machine
- `struct VersionInfo` - Component versioning information
- `enum UpdateStrategy` - Update behavior patterns
- `struct LifecycleEvent` - Lifecycle event notifications
- Comprehensive rustdoc

**Core Types:**
```rust
/// Component lifecycle state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleState {
    Uninstalled,
    Installing,
    Installed,
    Starting,
    Running,
    Updating,
    Stopping,
    Stopped,
    Failed,
}

/// Component version information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub hash: String,
    pub signature: Option<Vec<u8>>,
    pub installed_at: DateTime<Utc>,
}

/// Update strategy for component upgrades.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateStrategy {
    /// Stop old, start new (downtime)
    StopStart,
    /// Start new, switch traffic, stop old (blue-green)
    BlueGreen,
    /// Gradual traffic shift (canary)
    Canary,
}

/// Lifecycle state transition event.
#[derive(Debug, Clone)]
pub struct LifecycleEvent {
    pub component_id: ComponentId,
    pub from_state: LifecycleState,
    pub to_state: LifecycleState,
    pub timestamp: DateTime<Utc>,
    pub reason: Option<String>,
}
```

**Success Criteria:**
- LifecycleState covers complete component lifecycle
- VersionInfo supports versioning and verification
- UpdateStrategy supports all update patterns
- Unit tests for state transitions

#### Task 9.2: Management Abstractions (`core/management.rs`) - Block 8

**Deliverables:**
- `trait ComponentRegistry` - Component registry contract
- `struct InstallationMetadata` - Installation tracking
- `enum RegistryOperation` - Registry operations
- `struct ComponentQuery` - Query and filter types
- Comprehensive rustdoc

**Core Types:**
```rust
/// Component registry trait for component management.
#[async_trait]
pub trait ComponentRegistry: Send + Sync {
    /// Register a new component.
    async fn register(
        &mut self,
        component_id: ComponentId,
        metadata: ComponentMetadata,
    ) -> WasmResult<()>;
    
    /// Unregister a component.
    async fn unregister(&mut self, component_id: &ComponentId) -> WasmResult<()>;
    
    /// Get component metadata.
    async fn get_metadata(
        &self,
        component_id: &ComponentId,
    ) -> WasmResult<Option<ComponentMetadata>>;
    
    /// Query components by filter.
    async fn query(&self, query: ComponentQuery) -> WasmResult<Vec<ComponentMetadata>>;
}

/// Installation metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationMetadata {
    pub component_id: ComponentId,
    pub version: VersionInfo,
    pub source: InstallationSource,
    pub installed_at: DateTime<Utc>,
    pub install_path: PathBuf,
    pub state: LifecycleState,
}

/// Component query for filtering.
#[derive(Debug, Clone, Default)]
pub struct ComponentQuery {
    pub name_pattern: Option<String>,
    pub state: Option<LifecycleState>,
    pub installed_after: Option<DateTime<Utc>>,
    pub has_capability: Option<Capability>,
}

/// Registry operation types.
#[derive(Debug, Clone)]
pub enum RegistryOperation {
    Register(ComponentId, ComponentMetadata),
    Unregister(ComponentId),
    Update(ComponentId, ComponentMetadata),
    Query(ComponentQuery),
}
```

**Success Criteria:**
- ComponentRegistry trait supports complete registry operations
- InstallationMetadata tracks all installation details
- ComponentQuery enables flexible component filtering
- Unit tests for registry operations

---

### Phase 10: Domain-Specific Abstractions - Part 5: Bridge & Observability (Days 23-25)

**Purpose:** Implement domain-specific abstractions for Blocks 9-10 (OSL bridge and monitoring).

#### Task 10.1: Bridge Abstractions (`core/bridge.rs`) - Block 9

**Deliverables:**
- `trait HostFunction` - Host function contract
- `struct CapabilityMapping` - Capability to OSL permission mapping
- `struct HostCallContext` - Host function call context
- `enum HostFunctionCategory` - Function classification
- Comprehensive rustdoc

**Core Types:**
```rust
/// Host function trait for OSL bridge integration.
#[async_trait]
pub trait HostFunction: Send + Sync {
    /// Function name (e.g., "filesystem::read")
    fn name(&self) -> &str;
    
    /// Required capability
    fn required_capability(&self) -> Capability;
    
    /// Execute host function
    async fn execute(
        &self,
        context: &HostCallContext,
        args: Vec<u8>,
    ) -> WasmResult<Vec<u8>>;
}

/// Capability to OSL permission mapping.
#[derive(Debug, Clone)]
pub struct CapabilityMapping {
    pub capability: Capability,
    pub osl_operation: String,
    pub osl_permissions: Vec<String>,
}

/// Host function call context.
#[derive(Debug, Clone)]
pub struct HostCallContext {
    pub component_id: ComponentId,
    pub capabilities: CapabilitySet,
    pub security_mode: SecurityMode,
}

/// Host function category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostFunctionCategory {
    Filesystem,
    Network,
    Process,
    Storage,
    Messaging,
    Logging,
}
```

**Success Criteria:**
- HostFunction trait defines clear host function contract
- CapabilityMapping bridges WASM capabilities to OSL permissions
- HostCallContext provides complete call context
- Unit tests for capability mapping logic

#### Task 10.2: Observability Abstractions (`core/observability.rs`) - Block 10

**Deliverables:**
- `trait MetricsCollector` - Metrics collection contract
- `enum MetricType` - Metric classification
- `struct ObservabilityEvent` - Observable events
- `struct HealthStatus` - Component health reporting
- Comprehensive rustdoc

**Core Types:**
```rust
/// Metrics collector trait for observability.
pub trait MetricsCollector: Send + Sync {
    /// Record a metric value.
    fn record_metric(&self, metric: Metric) -> WasmResult<()>;
    
    /// Get current metrics snapshot.
    fn snapshot(&self) -> MetricsSnapshot;
}

/// Metric type classification.
#[derive(Debug, Clone)]
pub enum MetricType {
    Counter { name: String, value: u64 },
    Gauge { name: String, value: f64 },
    Histogram { name: String, value: f64 },
    Timing { name: String, duration_ms: u64 },
}

/// Metric value.
#[derive(Debug, Clone)]
pub struct Metric {
    pub component_id: ComponentId,
    pub metric_type: MetricType,
    pub labels: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

/// Metrics snapshot.
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub component_metrics: HashMap<ComponentId, Vec<Metric>>,
    pub timestamp: DateTime<Utc>,
}

/// Observability event for monitoring.
#[derive(Debug, Clone)]
pub struct ObservabilityEvent {
    pub component_id: ComponentId,
    pub event_type: String,
    pub severity: EventSeverity,
    pub message: String,
    pub metadata: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

/// Event severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum EventSeverity {
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

/// Component health status.
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub component_id: ComponentId,
    pub is_healthy: bool,
    pub last_check: DateTime<Utc>,
    pub failure_count: u32,
    pub message: Option<String>,
}
```

**Success Criteria:**
- MetricsCollector trait supports complete metrics collection
- MetricType covers all metric types (counter, gauge, histogram, timing)
- ObservabilityEvent enables comprehensive event tracking
- HealthStatus provides component health reporting
- Unit tests for metric recording and health checks

---

### Phase 11: Documentation, Integration, and Validation (Days 26-28)

**Purpose:** Finalize documentation, integrate all abstractions, and validate against ADRs.

#### Task 11.1: Comprehensive Documentation
**Deliverables:**
- Write module-level documentation for `core/`
- Add examples for all public types and traits
- Document design rationale in code comments
- Create examples/ directory with usage examples

**Success Criteria:**
- `cargo doc` generates complete documentation
- All public items have comprehensive rustdoc
- Examples compile and demonstrate usage

#### Task 11.2: Integration with Prelude
**Deliverables:**
- Create `src/prelude.rs`
- Re-export core types for ergonomic imports
- Document prelude usage in lib.rs

**Success Criteria:**
- `use airssys_wasm::prelude::*` works
- Common types available without full paths

#### Task 11.3: Architecture Review and Validation
**Deliverables:**
- Review core abstractions against ADRs (001, 002, 003, 005, 006, 007, 009, 010, 011, 012)
- Validate zero circular dependencies (`cargo check`)
- Peer review of core module design
- Documentation review for clarity and completeness

**Success Criteria:**
- All ADR requirements met (especially ADR-WASM-012)
- Zero circular dependencies confirmed
- Peer review approves design
- Documentation is comprehensive and clear

---

### Phase 12: Final Validation and Handoff (Days 27-28)

**Purpose:** Final validation, testing, and preparation for Block 1 implementation.

#### Task 12.1: Comprehensive Testing
**Deliverables:**
#### Task 12.1: Comprehensive Testing
**Deliverables:**
- >90% code coverage for all core modules
- Unit tests for all universal abstractions (component, capability, error, config)
- Unit tests for all domain-specific abstractions (runtime, interface, actor, security, messaging, storage, lifecycle, management, bridge, observability)
- Integration tests for trait contracts
- Documentation tests (code examples in rustdoc)

**Success Criteria:**
- All tests pass
- Coverage >90% across all core modules
- No compiler warnings
- Documentation examples compile

#### Task 12.2: Block Readiness Validation
**Deliverables:**
- Validate core abstractions cover ALL Block 1-11 requirements
- Confirm each implementation block can proceed with core as foundation
- Document any missing abstractions (unlikely if ADR-WASM-012 followed)
- Create handoff documentation for Block 1 (WASM-TASK-002)

**Success Criteria:**
- Each block's requirements mapped to core abstractions
- No missing abstractions identified
- Block 1 team can begin implementation
- Handoff documentation complete

---

## Success Criteria (Updated for Comprehensive Core)

### Definition of Done

This task is complete when:

1. ✅ **Module Structure Created (14 Files)**
   - `src/core/` directory exists
   - **Universal abstractions (4 files):** component.rs, capability.rs, error.rs, config.rs
   - **Domain abstractions (10 files):** runtime.rs, interface.rs, actor.rs, security.rs, messaging.rs, storage.rs, lifecycle.rs, management.rs, bridge.rs, observability.rs
   - `mod.rs` has declarations only (no implementation)

2. ✅ **Universal Abstractions Complete**
   - Component types: ComponentId, ComponentMetadata, ResourceLimits, ComponentInput, ComponentOutput, Component trait
   - Capability types: Capability enum, CapabilitySet, pattern types
   - Error types: WasmError enum (14+ variants), WasmResult, error helpers
   - Config types: RuntimeConfig, SecurityConfig, StorageConfig with defaults

3. ✅ **Domain Abstractions Complete (Blocks 1-10)**
   - **Block 1 (Runtime):** RuntimeEngine trait, ExecutionContext, ExecutionState, ComponentHandle
   - **Block 2 (Interface):** InterfaceDefinition, TypeDescriptor, FunctionSignature, BindingMetadata
   - **Block 3 (Actor):** ActorMessage, SupervisionStrategy, ActorState, ActorMetadata
   - **Block 4 (Security):** SecurityPolicy trait, PermissionRequest/Result, IsolationBoundary
   - **Block 5 (Messaging):** MessageEnvelope, MessageType, RoutingStrategy trait, DeliveryGuarantee
   - **Block 6 (Storage):** StorageBackend trait, StorageOperation, StorageTransaction trait
   - **Block 7 (Lifecycle):** LifecycleState, VersionInfo, UpdateStrategy, LifecycleEvent
   - **Block 8 (Management):** ComponentRegistry trait, InstallationMetadata, ComponentQuery
   - **Block 9 (Bridge):** HostFunction trait, CapabilityMapping, HostCallContext
   - **Block 10 (Observability):** MetricsCollector trait, MetricType, ObservabilityEvent, HealthStatus

4. ✅ **Testing Complete**
   - >90% code coverage for core module
   - All unit tests pass for universal abstractions
   - All unit tests pass for domain abstractions
   - Trait contract tests validate interfaces
   - Documentation tests compile

5. ✅ **Documentation Complete**
   - Comprehensive rustdoc for all 14 core files
   - Module-level documentation explains core purpose
   - Examples provided for all major types and traits
   - `cargo doc` generates complete documentation
   - Code comments explain design rationale

6. ✅ **Validation Complete**
   - Zero circular dependencies confirmed (`cargo check`)
   - ADR compliance validated (especially ADR-WASM-012)
   - Peer review completed and approved
   - All 11 blocks validated against core abstractions
   - Ready for Block 1 (WASM-TASK-002) to begin

7. ✅ **Quality Standards Met**
   - Zero compiler warnings
   - Follows workspace standards (§4.3, §2.1, §6.1, §6.2)
   - Follows Microsoft Rust Guidelines (M-ERRORS-CANONICAL-STRUCTS, M-DI-HIERARCHY)
   - Trait-centric design for extensibility
   - Newtype patterns for type safety

## Dependencies (Updated)

### Upstream Dependencies
- ✅ ADR-WASM-011: Module Structure Organization - **COMPLETE**
- ✅ ADR-WASM-012: Comprehensive Core Abstractions Strategy - **COMPLETE** ⚡ **NEW**
- ✅ KNOWLEDGE-WASM-012: Module Structure Architecture - **COMPLETE**
- ✅ All relevant ADRs (001, 002, 003, 005, 006, 007, 009, 010) - **COMPLETE**
- ✅ Workspace standards (§4.3, §2.1, §6.1, §6.2) - **DOCUMENTED**

### Downstream Dependencies (This Task Blocks)
- ⏸️ WASM-TASK-001: Implementation Roadmap (needs core types to plan)
- ⏸️ WASM-TASK-002: Block 1 - WASM Runtime Layer (needs RuntimeEngine trait, ExecutionContext, ComponentHandle)
- ⏸️ WASM-TASK-003: Block 2 - WIT Interface System (needs InterfaceDefinition, TypeDescriptor)
- ⏸️ WASM-TASK-004: Block 3 - Actor System Integration (needs ActorMessage, SupervisionStrategy)
- ⏸️ WASM-TASK-005: Block 4 - Security & Isolation (needs SecurityPolicy trait, PermissionRequest/Result)
- ⏸️ WASM-TASK-006: Block 5 - Inter-Component Communication (needs MessageEnvelope, RoutingStrategy)
- ⏸️ WASM-TASK-007: Block 6 - Persistent Storage (needs StorageBackend trait, StorageTransaction)
- ⏸️ WASM-TASK-008: Block 7 - Component Lifecycle (needs LifecycleState, UpdateStrategy)
- ⏸️ WASM-TASK-009: Block 8 - Component Management (needs ComponentRegistry trait, InstallationMetadata)
- ⏸️ WASM-TASK-010: Block 9 - OSL Bridge (needs HostFunction trait, CapabilityMapping)
- ⏸️ WASM-TASK-011: Block 10 - Monitoring & Observability (needs MetricsCollector trait, ObservabilityEvent)
- ⏸️ WASM-TASK-012: Block 11 - CLI Tool (needs all core types)
- ⏸️ ALL OTHER TASKS (all depend on comprehensive core abstractions)

**This task MUST be completed before any implementation work begins on any block.**

## Progress Tracking (Updated)

**Overall Status:** not-started - 0%  
**Estimated Timeline:** 3-4 weeks (28 days) - expanded per ADR-WASM-012

### Phase Breakdown (Updated)
| Phase | Description | Status | Days | Notes |
|-------|-------------|--------|------|-------|
| 1 | Core Module Foundation | not-started | 1-2 | Directory structure, dependencies |
| 2 | Universal: Component Abstractions | not-started | 3-4 | Component types and trait |
| 3 | Universal: Capability Abstractions | not-started | 5-6 | Capability types and patterns |
| 4 | Universal: Error Types | not-started | 7-8 | WasmError and helpers |
| 5 | Universal: Configuration Types | not-started | 9-10 | Runtime, Security, Storage configs |
| 6 | Domain Part 1: Runtime & Interface | not-started | 11-13 | Blocks 1-2 abstractions ⚡ **NEW** |
| 7 | Domain Part 2: Actor & Security | not-started | 14-16 | Blocks 3-4 abstractions ⚡ **NEW** |
| 8 | Domain Part 3: Messaging & Storage | not-started | 17-19 | Blocks 5-6 abstractions ⚡ **NEW** |
| 9 | Domain Part 4: Lifecycle & Management | not-started | 20-22 | Blocks 7-8 abstractions ⚡ **NEW** |
| 10 | Domain Part 5: Bridge & Observability | not-started | 23-25 | Blocks 9-10 abstractions ⚡ **NEW** |
| 11 | Documentation & Integration | not-started | 26 | Comprehensive docs, prelude |
| 12 | Final Validation & Handoff | not-started | 27-28 | Testing, validation, block readiness |

### Subtasks (Updated - Now 24 Tasks)
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 1.1 | Create Core Module Structure | not-started | - | Foundation |
| 1.2 | Add External Dependencies | not-started | - | thiserror, serde, chrono, async-trait |
| 2.1 | Implement Component Types | not-started | - | ComponentId, metadata, etc. |
| 2.2 | Implement Component Trait | not-started | - | Core behavior contract |
| 2.3 | Unit Tests for Component Types | not-started | - | >90% coverage |
| 3.1 | Implement Capability Types | not-started | - | Capability enum, patterns |
| 3.2 | Unit Tests for Capability Types | not-started | - | >90% coverage |
| 4.1 | Implement WasmError Enum | not-started | - | Comprehensive error types |
| 4.2 | Unit Tests for Error Types | not-started | - | Error message validation |
| 5.1 | Implement Configuration Structs | not-started | - | Runtime, Security, Storage |
| 5.2 | Unit Tests for Configuration Types | not-started | - | Default and serialization |
| 6.1 | Implement Runtime Abstractions ⚡ | not-started | - | RuntimeEngine trait, ExecutionContext |
| 6.2 | Implement Interface Abstractions ⚡ | not-started | - | InterfaceDefinition, TypeDescriptor |
| 7.1 | Implement Actor Abstractions ⚡ | not-started | - | ActorMessage, SupervisionStrategy |
| 7.2 | Implement Security Abstractions ⚡ | not-started | - | SecurityPolicy trait, PermissionRequest |
| 8.1 | Implement Messaging Abstractions ⚡ | not-started | - | MessageEnvelope, RoutingStrategy |
| 8.2 | Implement Storage Abstractions ⚡ | not-started | - | StorageBackend trait, Transaction |
| 9.1 | Implement Lifecycle Abstractions ⚡ | not-started | - | LifecycleState, UpdateStrategy |
| 9.2 | Implement Management Abstractions ⚡ | not-started | - | ComponentRegistry trait, Query |
| 10.1 | Implement Bridge Abstractions ⚡ | not-started | - | HostFunction trait, CapabilityMapping |
| 10.2 | Implement Observability Abstractions ⚡ | not-started | - | MetricsCollector trait, Events |
| 11.1 | Comprehensive Documentation | not-started | - | Rustdoc for all 14 files |
| 11.2 | Integration with Prelude | not-started | - | Ergonomic re-exports |
| 11.3 | Architecture Review | not-started | - | ADR compliance, peer review |
| 12.1 | Comprehensive Testing | not-started | - | >90% coverage all modules |
| 12.2 | Block Readiness Validation | not-started | - | Validate all blocks can proceed |

**⚡ = New tasks added per ADR-WASM-012 (Comprehensive Core Abstractions)**
**Deliverables:**
- Create `src/prelude.rs`
- Re-export core types for ergonomic imports
- Document prelude usage in lib.rs

**Success Criteria:**
- `use airssys_wasm::prelude::*` works
- Common types available without full paths

#### Task 6.3: Architecture Review and Validation
**Deliverables:**
- Review core abstractions against ADRs (001, 002, 003, 005, 006, 007, 009, 010, 011)
- Validate zero circular dependencies (`cargo check`)
- Peer review of core module design
- Documentation review for clarity and completeness

**Success Criteria:**
- All ADR requirements met
- Zero circular dependencies confirmed
- Peer review approves design
- Documentation is comprehensive and clear

---

## Success Criteria

### Definition of Done

This task is complete when:

1. ✅ **Module Structure Created**
   - `src/core/` directory exists
   - `component.rs`, `capability.rs`, `error.rs`, `config.rs` implemented
   - `mod.rs` has declarations only (no implementation)

2. ✅ **Component Abstractions Complete**
   - ComponentId, ComponentMetadata, ResourceLimits, ComponentInput, ComponentOutput implemented
   - Component trait defined
   - InstallationSource and ComponentState enums implemented
   - All types properly documented

3. ✅ **Capability Abstractions Complete**
   - Capability enum with all variants implemented
   - Pattern types (PathPattern, DomainPattern, etc.) implemented
   - CapabilitySet with ergonomic API implemented
   - All types properly documented

4. ✅ **Error Handling Complete**
   - WasmError enum with comprehensive variants
   - WasmResult type alias
   - Helper constructors for common errors
   - Error messages are actionable and clear

5. ✅ **Configuration Types Complete**
   - RuntimeConfig, SecurityConfig, StorageConfig implemented
   - All configs have sensible Default implementations
   - SecurityMode and StorageBackend enums implemented

6. ✅ **Testing Complete**
   - >90% code coverage for core module
   - All unit tests pass
   - Serialization/deserialization tested

7. ✅ **Documentation Complete**
   - Comprehensive rustdoc for all public items
   - Module-level documentation explains core purpose
   - Examples provided for all major types
   - `cargo doc` generates complete documentation

8. ✅ **Validation Complete**
   - Zero circular dependencies confirmed
   - ADR compliance validated (especially ADR-WASM-011)
   - Peer review completed and approved
   - Ready for Block 1 (WASM-TASK-002) to begin

## Dependencies

### Upstream Dependencies
- ✅ ADR-WASM-011: Module Structure Organization - **COMPLETE**
- ✅ KNOWLEDGE-WASM-012: Module Structure Architecture - **COMPLETE**
- ✅ All relevant ADRs (001, 002, 003, 005, 006, 007, 009, 010) - **COMPLETE**
- ✅ Workspace standards (§4.3, §2.1, §6.1) - **DOCUMENTED**

### Downstream Dependencies (This Task Blocks)
- ⏸️ WASM-TASK-001: Implementation Roadmap (needs core types to plan)
- ⏸️ WASM-TASK-002: WASM Runtime Layer (needs Component, ResourceLimits, WasmError)
- ⏸️ WASM-TASK-003: WIT Interface System (needs Component trait)
- ⏸️ WASM-TASK-004: Actor System Integration (needs ComponentId, ComponentConfig)
- ⏸️ WASM-TASK-005: Security & Isolation (needs Capability, CapabilitySet, WasmError)
- ⏸️ WASM-TASK-006: Inter-Component Communication (needs ComponentId, WasmError)
- ⏸️ WASM-TASK-007: Persistent Storage (needs ComponentId, StorageConfig)
- ⏸️ ALL OTHER TASKS (all depend on core abstractions)

**This task MUST be completed before any implementation work begins.**

## Risks and Mitigations

### Risk 1: Over-Engineering Core Abstractions
**Impact:** High - Overly complex core makes implementation difficult  
**Probability:** Medium - Easy to add "just in case" abstractions  
**Mitigation:**
- Strict YAGNI enforcement (only types needed by 3+ modules)
- Defer feature-specific types to domain modules
- Review against minimalism principle
- Get peer review before finalizing

### Risk 2: Under-Engineering Core Abstractions
**Impact:** High - Missing abstractions require later refactoring  
**Probability:** Low - We have comprehensive ADRs and knowledge docs  
**Mitigation:**
- Review core design against all 9 ADRs
- Validate against 11 implementation blocks
- Identify common patterns across blocks
- Add abstractions for shared concepts only

### Risk 3: Breaking Changes to Core
**Impact:** Critical - Core changes affect ALL modules  
**Probability:** Medium - Design may evolve during implementation  
**Mitigation:**
- Thorough design review BEFORE implementation
- Peer review of core module design
- Stability-first mindset (prefer traits for extensibility)
- Comprehensive documentation to prevent misuse

### Risk 4: Circular Dependencies
**Impact:** Critical - Makes codebase unmaintainable  
**Probability:** Very Low - Core has zero internal deps by design  
**Mitigation:**
- Enforce "core depends only on external crates" rule
- Regular `cargo check` during development
- Code review enforcement

### Risk 5: Inadequate Documentation
**Impact:** Medium - Poor docs lead to misuse  
**Probability:** Low - Documentation is in success criteria  
**Mitigation:**
- Documentation required for all public items
- Examples required for major types
- Peer review of documentation quality

## Progress Tracking

**Overall Status:** not-started - 0%

### Phase Breakdown
| Phase | Description | Status | Days | Notes |
|-------|-------------|--------|------|-------|
| 1 | Core Module Foundation | not-started | 1-2 | Directory structure, dependencies |
| 2 | Component Abstractions | not-started | 3-4 | Component types and trait |
| 3 | Capability Abstractions | not-started | 5-6 | Capability types and patterns |
| 4 | Error Types | not-started | 7-8 | WasmError and helpers |
| 5 | Configuration Types | not-started | 9-10 | Runtime, Security, Storage configs |
| 6 | Documentation and Validation | not-started | 11-14 | Docs, prelude, review |

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 1.1 | Create Core Module Structure | not-started | - | Foundation |
| 1.2 | Add External Dependencies | not-started | - | thiserror, serde, chrono |
| 2.1 | Implement Component Types | not-started | - | ComponentId, metadata, etc. |
| 2.2 | Implement Component Trait | not-started | - | Core behavior contract |
| 2.3 | Unit Tests for Component Types | not-started | - | >90% coverage |
| 3.1 | Implement Capability Types | not-started | - | Capability enum, patterns |
| 3.2 | Unit Tests for Capability Types | not-started | - | >90% coverage |
| 4.1 | Implement WasmError Enum | not-started | - | Comprehensive error types |
| 4.2 | Unit Tests for Error Types | not-started | - | Error message validation |
| 5.1 | Implement Configuration Structs | not-started | - | Runtime, Security, Storage |
| 5.2 | Unit Tests for Configuration Types | not-started | - | Default and serialization |
| 6.1 | Comprehensive Documentation | not-started | - | Rustdoc for all items |
| 6.2 | Integration with Prelude | not-started | - | Ergonomic re-exports |
| 6.3 | Architecture Review and Validation | not-started | - | ADR compliance check |

## Progress Log

*No progress yet - task just created*

## Related Documentation

### ADRs
- **ADR-WASM-012**: Comprehensive Core Abstractions Strategy - **CRITICAL** ⚡ (defines complete scope) **NEW**
- **ADR-WASM-011**: Module Structure Organization - **CRITICAL** (defines core/ module existence)
- **ADR-WASM-002**: WASM Runtime Engine Selection (RuntimeEngine trait, ResourceLimits design)
- **ADR-WASM-003**: Component Lifecycle Management (InstallationSource, ComponentState, LifecycleState)
- **ADR-WASM-005**: Capability-Based Security Model (Capability enum design, SecurityPolicy)
- **ADR-WASM-007**: Storage Backend Selection (StorageBackend trait, StorageConfig design)
- **ADR-WASM-001**: Multicodec Compatibility (ComponentInput/Output codec field)
- **ADR-WASM-009**: Inter-Component Messaging (MessageEnvelope, MessageType)
- **ADR-WASM-010**: Implementation Strategy (actor-hosted WASM components pattern)

### Knowledge Documentation
- **KNOWLEDGE-WASM-012**: Module Structure Architecture - **CRITICAL** (core module design rationale)
- **KNOWLEDGE-WASM-001**: Component Framework Architecture (overall architecture context)
- **KNOWLEDGE-WASM-003**: Core Architecture Design (component interface patterns)
- **KNOWLEDGE-WASM-005**: Inter-Component Messaging (messaging abstractions context)

### Workspace Standards
- **§4.3**: Module Architecture (mod.rs declaration-only pattern)
- **§2.1**: 3-Layer Import Organization (applies to core/ module)
- **§6.1**: YAGNI Principles (minimalism in core abstractions)
- **§6.2**: Avoid `dyn` Patterns (trait-centric but avoid unnecessary trait objects)

### Microsoft Rust Guidelines
- **M-ERRORS-CANONICAL-STRUCTS**: Structured error types with context and helpers
- **M-DI-HIERARCHY**: Trait-centric design for extensibility
- **M-ESSENTIAL-FN-INHERENT**: Core functionality in inherent methods

### External References
- [thiserror Documentation](https://docs.rs/thiserror/)
- [serde Documentation](https://serde.rs/)
- [async-trait Documentation](https://docs.rs/async-trait/)
- [Rust API Guidelines - Error Handling](https://rust-lang.github.io/api-guidelines/type-safety.html#errors-are-represented-by-a-dedicated-type)

## Notes (Updated for Comprehensive Core)

**Critical Foundation - Expanded Scope:**
This task is THE foundation of airssys-wasm, now expanded per ADR-WASM-012 to include comprehensive domain-specific abstractions for ALL 11 implementation blocks. Everything else builds on these abstractions. Get this right, and the rest flows naturally. Get this wrong, and we'll be refactoring 11 blocks later.

**Scope Expansion (ADR-WASM-012):**
- **Original scope (pre-ADR-012):** 4 files (component, capability, error, config) - 1-2 weeks
- **Expanded scope (post-ADR-012):** 14 files (4 universal + 10 domain-specific) - 3-4 weeks
- **Rationale:** Prevents circular dependencies, enables parallel development, ensures API stability
- **Investment:** +2 weeks upfront saves 4-8 weeks of refactoring across 11 blocks

**14 Core Files Structure:**
```
core/
├── Universal Abstractions (4 files):
│   ├── component.rs    # Component, ComponentId, Metadata, Limits
│   ├── capability.rs   # Capability, CapabilitySet, Patterns
│   ├── error.rs        # WasmError, WasmResult
│   └── config.rs       # RuntimeConfig, SecurityConfig, StorageConfig
│
└── Domain-Specific Abstractions (10 files):
    ├── runtime.rs      # RuntimeEngine trait, ExecutionContext (Block 1)
    ├── interface.rs    # InterfaceDefinition, TypeDescriptor (Block 2)
    ├── actor.rs        # ActorMessage, SupervisionStrategy (Block 3)
    ├── security.rs     # SecurityPolicy trait, PermissionRequest (Block 4)
    ├── messaging.rs    # MessageEnvelope, RoutingStrategy (Block 5)
    ├── storage.rs      # StorageBackend trait, Transaction (Block 6)
    ├── lifecycle.rs    # LifecycleState, UpdateStrategy (Block 7)
    ├── management.rs   # ComponentRegistry trait, Query (Block 8)
    ├── bridge.rs       # HostFunction trait, CapabilityMapping (Block 9)
    └── observability.rs # MetricsCollector trait, Events (Block 10)
```

**Trait-Centric Design Pattern:**
Each domain abstraction file follows this pattern:
- **Traits** - Behavior contracts (RuntimeEngine, StorageBackend, SecurityPolicy, etc.)
- **Enums** - State machines (ExecutionState, LifecycleState, MessageType, etc.)
- **Structs** - Data containers (ExecutionContext, MessageEnvelope, ComponentQuery, etc.)
- **Type Aliases** - Convenience (StorageResult<T>, etc.)

**Zero Internal Dependencies (CRITICAL):**
The core/ module depends ONLY on external crates (thiserror, serde, chrono, async-trait). It NEVER depends on any other airssys-wasm module. This prevents ALL circular dependency issues and guarantees compilation order.

**Minimalism with Completeness:**
- **Include:** Types shared by 3+ modules OR critical contracts for implementation blocks
- **Exclude:** Feature-specific types (belong in domain modules), helper utilities (belong in util/)
- **Principle:** Complete coverage of all blocks, but minimal within each domain

**Stability Over Features:**
Core types should rarely change after initial design. Design for long-term stability. Use traits for extensibility rather than frequent concrete type changes. Breaking core changes affect ALL 11 blocks.

**Documentation is Contract:**
Every public type must have comprehensive rustdoc. This is the API contract that all other modules depend on. Poor documentation leads to misuse and bugs. Examples for all major types required.

**Review Before Implement:**
Get peer review of core module design BEFORE implementing the rest of the system. Changing core after blocks are implemented is expensive and risky.

**Type Safety Matters:**
Use newtype patterns (ComponentId vs String, ActorMessage vs generic message) and enums (LifecycleState vs String, MessageType vs String) for type safety. The compiler should prevent misuse.

**Errors are First-Class:**
Error handling is part of the API contract. Use thiserror for structured errors with context. Error messages should be actionable. Each domain may add domain-specific error variants.

**Parallel Development Enablement:**
Once core is complete, all 11 blocks can be implemented in parallel by different teams. Core provides the shared type contracts. This is the primary benefit of comprehensive core abstractions.
