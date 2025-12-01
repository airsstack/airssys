# KNOWLEDGE-WASM-012: Module Structure Architecture

**Created:** 2025-10-21  
**Updated:** 2025-10-21  
**Status:** Complete  
**Category:** Architecture & Design  
**Impact:** Critical - Defines entire crate module organization and boundaries

## Purpose

Define the comprehensive module structure architecture for the `airssys-wasm` crate, establishing clear organizational patterns, module boundaries, dependency rules, and public API surface. This document explores multiple organizational approaches, evaluates tradeoffs, and recommends the optimal structure for our needs based on established patterns from `airssys-rt` and `airssys-osl`.

## Context

### Current State
- Architecture complete: 8 ADRs defining technical decisions
- Knowledge base: 11 knowledge docs covering all major patterns
- Implementation tasks: 12 tasks (WASM-TASK-001 through WASM-TASK-012) defined
- **Problem**: No unified module structure design document

### Scattered Module References
ADRs and knowledge docs contain code examples with various module paths:
- `src/runtime/` - WASM runtime execution
- `src/storage/` - Component persistent storage
- `src/security/` - Capability-based security
- `src/installation/` - Component installation and distribution
- Integration references to `airssys-rt` (broker, actor, supervisor)

### Why Module Structure Matters
**Without clear module organization:**
- ❌ Inconsistent code placement across features
- ❌ Circular dependencies between modules
- ❌ Unclear public API surface and stability guarantees
- ❌ Difficulty navigating codebase for new contributors
- ❌ Testing organization confusion (unit vs integration)
- ❌ Refactoring challenges due to unclear boundaries

**With clear module structure:**
- ✅ Logical code organization following domain boundaries
- ✅ Clear dependency hierarchy preventing cycles
- ✅ Well-defined public API with stability guarantees
- ✅ Easy navigation for contributors (know where things go)
- ✅ Testing organization follows module structure
- ✅ Safe refactoring within module boundaries

## Existing Patterns Analysis

### Pattern 1: airssys-rt Module Structure

**Observation from `airssys-rt/src/`:**
```
airssys-rt/src/
├── lib.rs              # Crate root with comprehensive documentation
├── prelude.rs          # Re-exports for common use cases
├── actor/              # Actor trait and implementations
├── broker/             # MessageBroker trait and implementations
├── mailbox/            # Mailbox implementations (bounded, unbounded)
├── message/            # Message trait and envelope types
├── monitoring/         # Health monitoring and metrics
├── supervisor/         # Supervision trees and strategies
├── system/             # ActorSystem and system configuration
└── util/               # Utility functions and helpers
```

**Key Characteristics:**
- ✅ **Domain-driven organization**: Each module represents a core concept (actor, broker, mailbox)
- ✅ **Flat structure**: Single level deep (no nested submodules visible)
- ✅ **Clear boundaries**: Each module has distinct responsibility
- ✅ **Prelude pattern**: Common types re-exported for ergonomic imports
- ✅ **Separation**: System-level (`system/`) vs component-level (`actor/`, `mailbox/`)

**Module Dependency Rules (Observed):**
```
actor/ → message/, mailbox/
broker/ → message/
supervisor/ → actor/, system/
system/ → actor/, broker/, mailbox/
monitoring/ → actor/, system/
prelude → (re-exports from all public modules)
```

### Pattern 2: airssys-osl Module Structure

**Observation from `airssys-osl/src/`:**
```
airssys-osl/src/
├── lib.rs              # Crate root with comprehensive documentation
├── prelude.rs          # Re-exports for ergonomic imports
├── core/               # Core abstractions (context, operation, result)
│   ├── context.rs
│   ├── operation.rs
│   └── result.rs
├── executors/          # Operation executors
├── helpers/            # High-level helper functions API
├── middleware/         # Middleware components (logger, security)
└── operations/         # Concrete operation implementations
```

**Key Characteristics:**
- ✅ **Layered architecture**: Core abstractions separate from implementations
- ✅ **Two API levels**: Helper functions (ergonomic) + executors (advanced)
- ✅ **Cross-cutting concerns**: Middleware as separate module
- ✅ **Core abstractions first**: `core/` module establishes foundation
- ✅ **Implementation modules**: `executors/` and `operations/` build on core

**Module Dependency Rules (Observed):**
```
core/ → (no internal dependencies, foundation)
operations/ → core/
executors/ → core/, operations/
middleware/ → core/
helpers/ → core/, executors/, middleware/
prelude → (re-exports from core/ and helpers/)
```

### Pattern Comparison

| Aspect | airssys-rt Pattern | airssys-osl Pattern |
|--------|-------------------|---------------------|
| **Depth** | Flat (single level) | Layered (core + impl) |
| **Organization** | Domain-driven | Layer-driven |
| **Abstractions** | Distributed across modules | Centralized in `core/` |
| **API Levels** | Single API surface | Dual API (helpers + executors) |
| **Best For** | Clear domain boundaries | Clear abstraction layers |

## Module Structure Approaches

### Approach 1: Flat Domain-Driven (airssys-rt style)

**Structure:**
```
airssys-wasm/src/
├── lib.rs              # Crate root documentation
├── prelude.rs          # Common re-exports
├── runtime/            # WASM runtime execution (Block 1)
├── component/          # Component lifecycle and management (Block 8)
├── actor/              # Actor system integration (Block 3)
├── security/           # Capability-based security (Block 4)
├── messaging/          # Inter-component communication (Block 5)
├── storage/            # Persistent storage (Block 6)
├── lifecycle/          # Deployment and versioning (Block 7)
├── wit/                # WIT interface system (Block 2)
├── osl/                # AirsSys-OSL bridge (Block 9)
├── monitoring/         # Monitoring and observability (Block 10)
├── installation/       # Component installation (relates to Block 8)
└── util/               # Utility functions
```

**Pros:**
- ✅ Clear 1:1 mapping to implementation blocks (WASM-TASK-002 through 012)
- ✅ Easy to find code (each block has its own module)
- ✅ Intuitive for contributors (matches task structure)
- ✅ Flat structure is easy to navigate
- ✅ Similar to proven airssys-rt pattern

**Cons:**
- ⚠️ No clear abstraction vs implementation separation
- ⚠️ Potential cross-cutting concerns (security touches many modules)
- ⚠️ May encourage tight coupling between domain modules
- ⚠️ Doesn't emphasize layered architecture from ADRs

### Approach 2: Layered Architecture (airssys-osl style)

**Structure:**
```
airssys-wasm/src/
├── lib.rs              # Crate root documentation
├── prelude.rs          # Common re-exports
├── core/               # Core abstractions and traits
│   ├── component.rs    # Component trait and types
│   ├── capability.rs   # Capability system abstractions
│   ├── error.rs        # Error types
│   └── config.rs       # Configuration types
├── runtime/            # WASM runtime layer (Block 1)
│   ├── engine.rs       # Wasmtime engine
│   ├── instance.rs     # Component instances
│   ├── limits.rs       # Resource limiting
│   └── loader.rs       # Component loading
├── interface/          # WIT interface system (Block 2)
│   ├── bindings.rs     # WIT bindings generation
│   ├── definitions.rs  # WIT definitions
│   └── validation.rs   # Interface validation
├── integration/        # AirsSys integration (Block 3, 9)
│   ├── actor.rs        # ComponentActor implementation
│   ├── broker.rs       # MessageBroker integration
│   ├── supervisor.rs   # SupervisorNode integration
│   └── osl.rs          # OSL bridge host functions
├── services/           # Core services (Blocks 4-7)
│   ├── security/       # Capability-based security
│   ├── messaging/      # Inter-component communication
│   ├── storage/        # Persistent storage
│   └── lifecycle/      # Deployment and versioning
├── management/         # Operations (Blocks 8, 10)
│   ├── installation/   # Component installation
│   ├── registry/       # Component registry
│   └── monitoring/     # Observability
└── util/               # Utility functions
```

**Pros:**
- ✅ Clear separation of abstractions (`core/`) from implementations
- ✅ Emphasizes layered architecture from ADRs
- ✅ `core/` module prevents circular dependencies
- ✅ `services/` clearly groups related functionality
- ✅ `integration/` highlights AirsSys dependencies

**Cons:**
- ⚠️ Deeper nesting (3 levels in some cases)
- ⚠️ Less intuitive mapping to implementation tasks
- ⚠️ More complex navigation (need to know layers)
- ⚠️ May be over-engineered for current needs

### Approach 3: Hybrid Block-Aligned with Core

**Structure:**
```
airssys-wasm/src/
├── lib.rs              # Crate root documentation
├── prelude.rs          # Common re-exports
├── core/               # Core abstractions (minimal, foundational)
│   ├── component.rs    # Component trait
│   ├── capability.rs   # Capability types
│   ├── error.rs        # Error types
│   └── config.rs       # Configuration
├── runtime/            # Block 1: WASM Runtime Layer
├── wit/                # Block 2: WIT Interface System
├── actor/              # Block 3: Actor System Integration
├── security/           # Block 4: Security & Isolation
├── messaging/          # Block 5: Inter-Component Communication
├── storage/            # Block 6: Persistent Storage
├── lifecycle/          # Block 7: Component Lifecycle
├── component/          # Block 8: Component Management
├── osl/                # Block 9: AirsSys-OSL Bridge
├── monitoring/         # Block 10: Monitoring & Observability
├── installation/       # Installation system (supports Block 8)
└── util/               # Utility functions
```

**Pros:**
- ✅ Best of both: Core abstractions + flat domain modules
- ✅ Direct mapping to implementation blocks (easy task alignment)
- ✅ `core/` prevents circular dependencies
- ✅ Flat structure for easy navigation (mostly 2 levels)
- ✅ Follows YAGNI (simpler than full layered approach)

**Cons:**
- ⚠️ Still some potential for cross-cutting concerns
- ⚠️ May need sub-modules within some top-level modules

## Module Dependency Analysis

### Approach 3 Dependency Graph (Recommended)

**Foundation Layer (No dependencies within airssys-wasm):**
```
core/ → (external: serde, thiserror, chrono)
```

**Runtime Layer (Depends on core):**
```
runtime/ → core/, wasmtime
wit/ → core/, wit-bindgen
```

**Integration Layer (Depends on core + external):**
```
actor/ → core/, runtime/, airssys-rt
osl/ → core/, runtime/, airssys-osl
```

**Services Layer (Depends on core + integration):**
```
security/ → core/, runtime/
messaging/ → core/, actor/, airssys-rt
storage/ → core/
lifecycle/ → core/, runtime/, actor/
```

**Management Layer (Depends on everything):**
```
component/ → core/, runtime/, actor/, lifecycle/
installation/ → core/, component/, security/
monitoring/ → core/, actor/, airssys-rt
```

**Utility Layer (Can depend on anything, provides helpers):**
```
util/ → core/, (any module as needed)
```

**Dependency Rules:**
- ✅ **core/** has zero internal dependencies (foundation)
- ✅ **runtime/, wit/** only depend on core (WASM foundation)
- ✅ **actor/, osl/** integrate external systems (clear boundary)
- ✅ **services/** build on runtime and integration layers
- ✅ **management/** orchestrates all layers
- ❌ **No circular dependencies** (enforced by layer rules)

## Public API Surface Design

### Prelude Re-Exports Pattern

Following `airssys-rt` and `airssys-osl` patterns, the `prelude` module should re-export commonly used types for ergonomic imports.

**Recommended `prelude.rs`:**
```rust
//! Prelude module for common airssys-wasm types and traits.
//!
//! This module re-exports the most commonly used items to simplify imports.
//!
//! # Usage
//!
//! ```rust,ignore
//! use airssys_wasm::prelude::*;
//! ```

// Core types and traits
pub use crate::core::{
    component::{Component, ComponentConfig, ComponentId, ComponentMetadata},
    capability::{Capability, CapabilitySet},
    error::{WasmError, WasmResult},
    config::RuntimeConfig,
};

// Runtime essentials
pub use crate::runtime::{
    WasmEngine,
    WasmInstance,
    ResourceLimits,
};

// Actor integration (most common pattern)
pub use crate::actor::{
    ComponentActor,
    ComponentMessage,
};

// Security primitives
pub use crate::security::{
    CapabilityManager,
    SecurityPolicy,
};

// Storage API (common for component developers)
pub use crate::storage::{
    StorageBackend,
    ComponentStorage,
};

// Lifecycle management
pub use crate::lifecycle::{
    DeploymentStrategy,
    VersionManager,
};
```

### Lib.rs Module Declaration Pattern

**Recommended `lib.rs` structure:**
```rust
//! # airssys-wasm - WASM Component Framework for Pluggable Systems
//!
//! Runtime deployment infrastructure for general-purpose component-based architectures.
//!
//! [Comprehensive crate-level documentation here]
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use airssys_wasm::prelude::*;
//! ```
//!
//! # Module Organization
//!
//! - [`core`] - Core abstractions and types (foundation)
//! - [`runtime`] - WASM runtime execution (Block 1)
//! - [`wit`] - WIT interface system (Block 2)  
//! - [`actor`] - Actor system integration (Block 3)
//! - [`security`] - Capability-based security (Block 4)
//! - [`messaging`] - Inter-component communication (Block 5)
//! - [`storage`] - Persistent storage (Block 6)
//! - [`lifecycle`] - Component lifecycle management (Block 7)
//! - [`component`] - Component management (Block 8)
//! - [`osl`] - AirsSys-OSL bridge (Block 9)
//! - [`monitoring`] - Monitoring and observability (Block 10)
//! - [`installation`] - Component installation system
//! - [`prelude`] - Common re-exports

// Core abstractions (foundation, no internal deps)
pub mod core;

// WASM runtime foundation (Blocks 1-2)
pub mod runtime;
pub mod wit;

// AirsSys integration layer (Blocks 3, 9)
pub mod actor;
pub mod osl;

// Core services layer (Blocks 4-7)
pub mod security;
pub mod messaging;
pub mod storage;
pub mod lifecycle;

// Management layer (Blocks 8, 10)
pub mod component;
pub mod installation;
pub mod monitoring;

// Utility and prelude
pub mod util;
pub mod prelude;

// Re-export core error types at crate root for convenience
pub use core::error::{WasmError, WasmResult};
```

### Module Visibility Rules

**Recommendation:**
- ✅ **Public modules** (`pub mod`): All top-level modules (as shown above)
- ✅ **Public APIs within modules**: Types, traits, functions used by external consumers
- ✅ **Private submodules**: Internal organization within modules (e.g., `security/internal/`)
- ✅ **Crate-public types** (`pub(crate)`): Shared between modules but not external API

**Example visibility pattern:**
```rust
// In security/mod.rs
pub mod capabilities;      // Public: External API
pub mod policies;          // Public: External API
pub(crate) mod internal;   // Crate-public: Shared with other modules
mod enforcement;           // Private: Implementation detail

// Public API surface
pub use capabilities::{Capability, CapabilitySet, CapabilityManager};
pub use policies::{SecurityPolicy, DenyByDefault, AllowList};
```

## Testing Module Organization

### Unit Tests (Co-located)

Following Rust conventions, unit tests should be co-located with implementation code.

**Pattern:**
```rust
// In security/capabilities.rs
pub struct CapabilityManager {
    // implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_grant() {
        // test implementation
    }
}
```

### Integration Tests (Separate Directory)

Integration tests should live in `airssys-wasm/tests/` and test cross-module interactions.

**Recommended test organization:**
```
airssys-wasm/tests/
├── runtime_tests.rs               # Block 1: Runtime execution tests
├── wit_integration_tests.rs       # Block 2: WIT binding tests
├── actor_integration_tests.rs     # Block 3: Actor system tests
├── security_integration_tests.rs  # Block 4: Capability enforcement tests
├── messaging_tests.rs             # Block 5: Inter-component messaging tests
├── storage_tests.rs               # Block 6: Storage backend tests
├── lifecycle_tests.rs             # Block 7: Deployment lifecycle tests
├── component_management_tests.rs  # Block 8: Component management tests
├── osl_bridge_tests.rs            # Block 9: OSL integration tests
├── monitoring_tests.rs            # Block 10: Observability tests
└── end_to_end_tests.rs            # Full stack integration tests
```

**Integration test naming aligns with blocks for clear task tracking.**

## Recommended Module Structure

### Final Recommendation: Approach 3 (Hybrid Block-Aligned with Core)

**Rationale:**
1. ✅ **Clear task alignment**: Direct mapping to WASM-TASK-002 through 012
2. ✅ **Prevents circular deps**: `core/` module establishes foundation
3. ✅ **Easy navigation**: Mostly flat, 2-level structure
4. ✅ **YAGNI compliant**: Simpler than full layered architecture
5. ✅ **Proven pattern**: Combines best of airssys-rt and airssys-osl
6. ✅ **Contributor-friendly**: Intuitive structure for new developers

### Complete Module Structure

```
airssys-wasm/
├── Cargo.toml
├── README.md
├── LICENSE-MIT
├── LICENSE-APACHE
├── src/
│   ├── lib.rs                      # Crate root with comprehensive docs
│   ├── prelude.rs                  # Common re-exports
│   │
│   ├── core/                       # Core abstractions (foundation)
│   │   ├── mod.rs                  # Module declarations only
│   │   ├── component.rs            # Component trait and types
│   │   ├── capability.rs           # Capability system abstractions
│   │   ├── error.rs                # Error types (thiserror-based)
│   │   └── config.rs               # Configuration types
│   │
│   ├── runtime/                    # Block 1: WASM Runtime Layer
│   │   ├── mod.rs
│   │   ├── engine.rs               # Wasmtime engine wrapper
│   │   ├── instance.rs             # Component instance management
│   │   ├── limits.rs               # Resource limits (memory, CPU)
│   │   ├── loader.rs               # Component loading
│   │   └── executor.rs             # Component execution
│   │
│   ├── wit/                        # Block 2: WIT Interface System
│   │   ├── mod.rs
│   │   ├── bindings.rs             # WIT bindings generation
│   │   ├── definitions.rs          # WIT interface definitions
│   │   ├── validation.rs           # Interface validation
│   │   └── codegen.rs              # Code generation utilities
│   │
│   ├── actor/                      # Block 3: Actor System Integration
│   │   ├── mod.rs
│   │   ├── component_actor.rs      # ComponentActor (Actor + Child)
│   │   ├── lifecycle.rs            # Child::start/stop implementations
│   │   ├── spawning.rs             # ActorSystem::spawn integration
│   │   └── supervision.rs          # SupervisorNode integration
│   │
│   ├── security/                   # Block 4: Security & Isolation
│   │   ├── mod.rs
│   │   ├── capabilities/           # Capability system
│   │   │   ├── mod.rs
│   │   │   ├── manager.rs          # CapabilityManager
│   │   │   ├── grant.rs            # Capability granting
│   │   │   └── check.rs            # Capability checking
│   │   ├── policies/               # Security policies
│   │   │   ├── mod.rs
│   │   │   ├── deny_by_default.rs  # Default deny policy
│   │   │   └── allow_list.rs       # Allow list policy
│   │   └── isolation/              # Component isolation
│   │       ├── mod.rs
│   │       ├── sandbox.rs          # Sandbox enforcement
│   │       └── boundaries.rs       # Isolation boundaries
│   │
│   ├── messaging/                  # Block 5: Inter-Component Communication
│   │   ├── mod.rs
│   │   ├── router.rs               # MessageBroker routing integration
│   │   ├── fire_and_forget.rs      # Fire-and-forget messaging
│   │   ├── request_response.rs     # Request-response pattern
│   │   ├── codec.rs                # Multicodec message encoding
│   │   └── topics.rs               # Topic-based pub-sub
│   │
│   ├── storage/                    # Block 6: Persistent Storage
│   │   ├── mod.rs
│   │   ├── backend.rs              # StorageBackend trait
│   │   ├── component.rs            # ComponentStorage API
│   │   ├── backends/               # Backend implementations
│   │   │   ├── mod.rs
│   │   │   ├── sled.rs             # Sled backend (default)
│   │   │   └── rocksdb.rs          # RocksDB backend (optional)
│   │   ├── namespace.rs            # Namespace isolation
│   │   └── quota.rs                # Storage quota management
│   │
│   ├── lifecycle/                  # Block 7: Component Lifecycle
│   │   ├── mod.rs
│   │   ├── deployment/             # Deployment strategies
│   │   │   ├── mod.rs
│   │   │   ├── blue_green.rs       # Blue-green deployment
│   │   │   ├── canary.rs           # Canary deployment
│   │   │   └── rolling.rs          # Rolling update
│   │   ├── versioning/             # Version management
│   │   │   ├── mod.rs
│   │   │   ├── manager.rs          # VersionManager
│   │   │   └── registry.rs         # Version registry
│   │   ├── routing.rs              # Traffic routing
│   │   └── rollback.rs             # Rollback handling
│   │
│   ├── component/                  # Block 8: Component Management
│   │   ├── mod.rs
│   │   ├── registry.rs             # Component registry
│   │   ├── metadata.rs             # Component metadata
│   │   ├── loader.rs               # Component loading coordination
│   │   └── manager.rs              # Component lifecycle manager
│   │
│   ├── osl/                        # Block 9: AirsSys-OSL Bridge
│   │   ├── mod.rs
│   │   ├── host_functions/         # Host function implementations
│   │   │   ├── mod.rs
│   │   │   ├── filesystem.rs       # Filesystem operations
│   │   │   ├── network.rs          # Network operations
│   │   │   └── process.rs          # Process operations
│   │   ├── security.rs             # Layered security integration
│   │   └── bindings.rs             # WIT bindings for OSL
│   │
│   ├── monitoring/                 # Block 10: Monitoring & Observability
│   │   ├── mod.rs
│   │   ├── metrics.rs              # Metrics collection
│   │   ├── health.rs               # Health monitoring
│   │   ├── prometheus.rs           # Prometheus exporter
│   │   └── alerts.rs               # Alerting system
│   │
│   ├── installation/               # Installation system
│   │   ├── mod.rs
│   │   ├── installer.rs            # Component installer
│   │   ├── sources/                # Installation sources
│   │   │   ├── mod.rs
│   │   │   ├── git.rs              # Git-based installation
│   │   │   ├── file.rs             # File-based installation
│   │   │   └── url.rs              # URL-based installation
│   │   ├── signature.rs            # Ed25519 signature verification
│   │   └── manifest.rs             # Component.toml parsing
│   │
│   └── util/                       # Utility functions
│       ├── mod.rs
│       ├── serialization.rs        # Serialization helpers
│       └── time.rs                 # Time utilities
│
├── tests/                          # Integration tests
│   ├── runtime_tests.rs
│   ├── wit_integration_tests.rs
│   ├── actor_integration_tests.rs
│   ├── security_integration_tests.rs
│   ├── messaging_tests.rs
│   ├── storage_tests.rs
│   ├── lifecycle_tests.rs
│   ├── component_management_tests.rs
│   ├── osl_bridge_tests.rs
│   ├── monitoring_tests.rs
│   └── end_to_end_tests.rs
│
├── examples/                       # Usage examples
│   ├── basic_component.rs
│   ├── actor_hosted_component.rs
│   ├── inter_component_messaging.rs
│   ├── persistent_storage.rs
│   ├── blue_green_deployment.rs
│   └── custom_capabilities.rs
│
├── benches/                        # Criterion benchmarks
│   ├── runtime_benchmarks.rs
│   ├── messaging_benchmarks.rs
│   ├── storage_benchmarks.rs
│   └── actor_benchmarks.rs
│
└── docs/                           # mdBook documentation
    ├── book.toml
    └── src/
        ├── SUMMARY.md
        ├── introduction.md
        ├── tutorials/
        ├── guides/
        ├── reference/
        └── explanation/
```

### Module Responsibility Matrix

| Module | Responsibility | Primary Dependencies | Public API |
|--------|---------------|---------------------|------------|
| `core/` | Core abstractions and types | External only | Component, Capability, Error, Config |
| `runtime/` | WASM execution engine | core, wasmtime | WasmEngine, WasmInstance, ResourceLimits |
| `wit/` | WIT interface system | core, wit-bindgen | Bindings, Definitions, Validation |
| `actor/` | Actor system integration | core, runtime, airssys-rt | ComponentActor, lifecycle hooks |
| `security/` | Capability-based security | core, runtime | CapabilityManager, SecurityPolicy |
| `messaging/` | Inter-component communication | core, actor, airssys-rt | Router, FireAndForget, RequestResponse |
| `storage/` | Persistent storage | core | StorageBackend, ComponentStorage |
| `lifecycle/` | Deployment and versioning | core, runtime, actor | DeploymentStrategy, VersionManager |
| `component/` | Component management | core, runtime, lifecycle | Registry, Metadata, Manager |
| `osl/` | AirsSys-OSL bridge | core, runtime, airssys-osl | Host functions, security integration |
| `monitoring/` | Monitoring and observability | core, actor, airssys-rt | Metrics, Health, Prometheus |
| `installation/` | Component installation | core, component, security | Installer, signature verification |
| `util/` | Utility functions | core | Serialization, time helpers |
| `prelude/` | Common re-exports | All public modules | Re-exports for ergonomic imports |

## Migration and Implementation Strategy

### Phase 1: Foundation Setup (Week 1 of WASM-TASK-002)
**Create core module structure:**
1. Create `src/lib.rs` with module declarations
2. Create `src/core/` with abstractions (component, capability, error, config)
3. Create `src/prelude.rs` with initial re-exports
4. Create `src/util/` for shared utilities
5. Add comprehensive crate-level documentation

**Deliverables:**
- Compilable crate structure
- Core types defined
- Documentation framework established
- CI integration (cargo check passes)

### Phase 2: Block-by-Block Implementation (Weeks 2-6+)
**Follow WASM-TASK-002 through 012 order:**
1. **WASM-TASK-002**: Implement `runtime/` module (Block 1)
2. **WASM-TASK-003**: Implement `wit/` module (Block 2)
3. **WASM-TASK-004**: Implement `actor/` module (Block 3) ⭐ CRITICAL
4. **WASM-TASK-005**: Implement `security/` module (Block 4)
5. **WASM-TASK-006**: Implement `messaging/` module (Block 5)
6. **WASM-TASK-007**: Implement `storage/` module (Block 6)
7. **WASM-TASK-008**: Implement `lifecycle/` module (Block 7)
8. **WASM-TASK-009**: Implement `component/` module (Block 8)
9. **WASM-TASK-010**: Implement `osl/` module (Block 9)
10. **WASM-TASK-011**: Implement `monitoring/` module (Block 10)
11. **WASM-TASK-012**: Implement `installation/` module

**Each block implementation:**
- Create module directory structure
- Implement core types and traits
- Add unit tests (co-located)
- Add integration tests (tests/ directory)
- Update prelude.rs with new public types
- Update lib.rs module documentation

### Phase 3: Refinement and Documentation (Ongoing)
**Continuous improvements:**
1. Refine public API surface based on usage
2. Add examples for each module (examples/ directory)
3. Create mdBook documentation (docs/ directory)
4. Add benchmarks (benches/ directory)
5. Performance profiling and optimization

## Module Guidelines and Best Practices

### Rule 1: Core Module is Foundation
- ✅ `core/` has zero internal dependencies (only external crates)
- ✅ All other modules can depend on `core/`
- ✅ Core types are minimal, stable, and well-documented
- ❌ Never add implementation logic to `core/` (abstractions only)

### Rule 2: Prevent Circular Dependencies
- ✅ Dependency graph must be acyclic (DAG)
- ✅ Use `pub(crate)` for internal shared types if needed
- ✅ Extract shared code to `core/` or `util/` if circular dependency arises
- ❌ Never have module A → B → A dependency chains

### Rule 3: mod.rs is Declaration Only
**Following §4.3 workspace standards:**
- ✅ `mod.rs` contains module declarations and re-exports ONLY
- ✅ Implementation code goes in named files (e.g., `engine.rs`, `manager.rs`)
- ❌ No implementation code in `mod.rs` files

**Example:**
```rust
// security/mod.rs - CORRECT
pub mod capabilities;
pub mod policies;
pub mod isolation;

pub use capabilities::{CapabilityManager, Capability};
pub use policies::{SecurityPolicy, DenyByDefault};
```

```rust
// security/mod.rs - WRONG (has implementation)
pub struct CapabilityManager {
    // implementation here - WRONG, should be in capabilities/manager.rs
}
```

### Rule 4: Public API Stability
- ✅ Types in `prelude` are stable public API
- ✅ Public module APIs should be well-documented and stable
- ✅ Use semantic versioning for breaking changes
- ⚠️ Mark experimental APIs with `#[doc(hidden)]` or feature flags

### Rule 5: Testing Mirrors Structure
- ✅ Unit tests co-located with implementation (in `mod tests`)
- ✅ Integration tests in `tests/` directory mirror module structure
- ✅ Test file names match module names (e.g., `runtime_tests.rs` for `runtime/`)
- ✅ End-to-end tests in `tests/end_to_end_tests.rs`

### Rule 6: Documentation Requirements
- ✅ Every public module has comprehensive module-level docs
- ✅ Every public type has rustdoc with examples
- ✅ Every public function has doc comments
- ✅ `lib.rs` has comprehensive crate-level documentation
- ✅ Examples provided for common use cases

## Examples of Module Implementation

### Example 1: runtime/mod.rs

```rust
//! WASM runtime execution engine.
//!
//! This module provides the core WASM runtime layer using Wasmtime with
//! Component Model support, memory sandboxing, CPU limiting, and crash isolation.
//!
//! # Overview
//!
//! The runtime module implements Block 1 (WASM Runtime Layer) from the
//! implementation roadmap. It provides:
//!
//! - Component Model execution via Wasmtime
//! - Memory limits (512KB-4MB configurable)
//! - CPU limiting (hybrid fuel + wall-clock timeout)
//! - Async execution with Tokio integration
//! - Crash isolation (component failures don't crash host)
//!
//! # Examples
//!
//! ```rust,ignore
//! use airssys_wasm::runtime::{WasmEngine, ResourceLimits};
//!
//! let engine = WasmEngine::new()?;
//! let limits = ResourceLimits {
//!     max_memory: 1024 * 1024, // 1MB
//!     max_fuel: 1_000_000,
//!     max_execution_ms: 100,
//! };
//! let instance = engine.load_component("component.wasm", limits).await?;
//! ```

mod engine;
mod instance;
mod limits;
mod loader;
mod executor;

pub use engine::WasmEngine;
pub use instance::WasmInstance;
pub use limits::ResourceLimits;
pub use loader::ComponentLoader;
pub use executor::ComponentExecutor;
```

### Example 2: actor/component_actor.rs

```rust
//! ComponentActor implementation (Actor + Child dual trait).
//!
//! This module implements the actor-hosted component pattern where each WASM
//! component instance is hosted within its own ComponentActor for isolation
//! and supervision.

use airssys_rt::prelude::*;
use async_trait::async_trait;

use crate::core::component::ComponentId;
use crate::runtime::WasmInstance;

/// Actor that hosts a WASM component instance.
///
/// ComponentActor implements both `Actor` (for message handling) and `Child`
/// (for WASM lifecycle management within supervision trees).
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::prelude::*;
///
/// let system = ActorSystem::new(SystemConfig::default());
/// let component_actor = ComponentActor::new(component_id, wasm_instance);
/// let address = system.spawn(component_actor).await?;
/// ```
pub struct ComponentActor {
    component_id: ComponentId,
    instance: Option<WasmInstance>,
}

impl ComponentActor {
    /// Create a new ComponentActor.
    pub fn new(component_id: ComponentId, instance: WasmInstance) -> Self {
        Self {
            component_id,
            instance: Some(instance),
        }
    }
}

#[async_trait]
impl Actor for ComponentActor {
    type Message = ComponentMessage;
    type Error = crate::core::error::WasmError;

    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        msg: Self::Message,
        ctx: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        // Implementation
        todo!()
    }
}

#[async_trait]
impl Child for ComponentActor {
    async fn start(&mut self, ctx: &mut ChildContext) -> Result<(), Box<dyn std::error::Error>> {
        // Load WASM component, initialize instance
        todo!()
    }

    async fn stop(&mut self, ctx: &mut ChildContext) -> Result<(), Box<dyn std::error::Error>> {
        // Clean up WASM instance, release resources
        todo!()
    }
}
```

## Success Criteria

This module structure design is successful when:

1. ✅ **Clear organization**: New contributors can find code intuitively
2. ✅ **No circular dependencies**: Dependency graph is acyclic
3. ✅ **Stable public API**: Prelude provides ergonomic imports
4. ✅ **Task alignment**: Direct mapping to WASM-TASK-002 through 012
5. ✅ **Testing clarity**: Test organization mirrors module structure
6. ✅ **Documentation completeness**: Every public item documented
7. ✅ **Workspace compliance**: Follows §4.3 module architecture standards
8. ✅ **Integration success**: airssys-rt and airssys-osl integrate cleanly
9. ✅ **Maintainability**: Easy to refactor within module boundaries
10. ✅ **YAGNI compliance**: Simple, practical, not over-engineered

## Related Documentation

### ADRs
- **ADR-WASM-002**: WASM Runtime Engine Selection (runtime/ module)
- **ADR-WASM-005**: Capability-Based Security Model (security/ module)
- **ADR-WASM-006**: Component Isolation and Sandboxing (actor/ module)
- **ADR-WASM-007**: Storage Backend Selection (storage/ module)
- **ADR-WASM-009**: Component Communication Model (messaging/ module)
- **ADR-WASM-010**: Implementation Strategy and Build Order (block alignment)

### Knowledge Documentation
- **KNOWLEDGE-WASM-001**: Component Framework Architecture (overall structure)
- **KNOWLEDGE-WASM-003**: Core Architecture Design (integration patterns)
- **KNOWLEDGE-WASM-004**: WIT Management Architecture (wit/ module)
- **KNOWLEDGE-WASM-005**: Inter-Component Messaging Architecture (messaging/ module)
- **KNOWLEDGE-WASM-007**: Component Storage Architecture (storage/ module)
- **KNOWLEDGE-WASM-009**: Component Installation Architecture (installation/ module)

### Workspace Standards
- **Workspace §4.3**: Module Architecture (mod.rs declaration-only pattern)
- **Workspace §2.1**: 3-Layer Import Organization (applies to all modules)
- **Workspace §6.1**: YAGNI Principles (simple module structure)

### External References
- [The Rust Programming Language - Modules](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)
- [Rust API Guidelines - Organization](https://rust-lang.github.io/api-guidelines/organization.html)
- [Microsoft Rust Guidelines - Module Organization](https://microsoft.github.io/rust-guidelines/)

## Notes

**Foundation First:**
The `core/` module establishes the foundation. All types here must be stable, minimal, and well-designed before building on top.

**Block Alignment:**
Module structure directly maps to implementation blocks (WASM-TASK-002 through 012). This makes task tracking and implementation straightforward.

**YAGNI Application:**
We're choosing the simpler hybrid approach (Approach 3) over the more complex layered approach (Approach 2). This follows YAGNI - we can refactor to deeper layering later if proven necessary.

**Proven Patterns:**
We're combining the best of airssys-rt (flat, domain-driven) and airssys-osl (core abstractions) based on their successful implementations.

**Integration Clarity:**
Modules `actor/` and `osl/` explicitly highlight integration with airssys-rt and airssys-osl, making external dependencies clear.

**Future Flexibility:**
The structure supports future growth. If a module becomes too large, we can add submodules without breaking the overall organization.
