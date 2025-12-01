# ADR-WASM-011: Module Structure Organization

**Status:** Accepted  
**Date:** 2025-10-21  
**Category:** Code Architecture & Organization  
**Deciders:** Architecture Team  
**Related ADRs:** ADR-WASM-010 (Implementation Strategy)  
**Related Knowledge:** KNOWLEDGE-WASM-012 (Module Structure Architecture)

## Context and Problem Statement

The `airssys-wasm` crate requires a clear, maintainable module structure that will guide all implementation work across 11 major building blocks (WASM-TASK-002 through WASM-TASK-012). Without a defined organizational pattern, code will be placed inconsistently, creating maintenance challenges, unclear public APIs, potential circular dependencies, and contributor confusion.

### Current State
- **Architecture complete**: 8 ADRs defining technical decisions
- **Knowledge base**: 12 knowledge docs covering all major patterns
- **Implementation tasks**: 12 tasks (WASM-TASK-001 through 012) defined
- **Problem**: Scattered module path references in ADRs but no unified structure
- **Risk**: Inconsistent code placement, circular dependencies, unclear API boundaries

### Existing Patterns in AirsSys Ecosystem

We have two successful module organization patterns to learn from:

**Pattern 1: airssys-rt (Flat Domain-Driven)**
- Single-level flat structure (`actor/`, `broker/`, `mailbox/`, `supervisor/`)
- Each module represents a clear domain concept
- Prelude pattern for ergonomic re-exports
- Proven at scale: ~5,300 lines, 381 tests, zero circular dependencies

**Pattern 2: airssys-osl (Layered with Core)**
- `core/` module establishes foundational abstractions
- Implementation modules build on core (`executors/`, `helpers/`, `middleware/`)
- Two API levels: helper functions (ergonomic) + executors (advanced)
- Clear separation of abstractions from implementations

### Key Questions
1. How should we organize modules to prevent circular dependencies?
2. Should we use flat (domain-driven) or layered (abstraction-first) structure?
3. How do we ensure clear task-to-module mapping for 11 implementation blocks?
4. What should be the public API surface and stability guarantees?
5. How do we organize testing to mirror module structure?

## Decision Drivers

### Must Have
- ✅ **Prevent circular dependencies** - Enforce acyclic dependency graph
- ✅ **Clear task alignment** - Direct mapping to WASM-TASK-002 through 012
- ✅ **Contributor-friendly** - Intuitive structure for new developers
- ✅ **Stable public API** - Clear API surface with prelude pattern
- ✅ **Workspace compliance** - Follow §4.3 module architecture standards

### Should Have
- ✅ **YAGNI compliant** - Simple, not over-engineered
- ✅ **Proven patterns** - Learn from airssys-rt and airssys-osl success
- ✅ **Testing clarity** - Test organization mirrors module structure
- ✅ **Refactoring-friendly** - Easy to refactor within module boundaries

### Nice to Have
- ✅ **Integration visibility** - Clear highlighting of external dependencies
- ✅ **Documentation alignment** - Module structure aids navigation
- ✅ **Future flexibility** - Can evolve without breaking overall organization

## Considered Options

### Option 1: Flat Domain-Driven (airssys-rt style)

**Structure:**
```
airssys-wasm/src/
├── lib.rs
├── prelude.rs
├── runtime/       # Block 1: WASM Runtime Layer
├── wit/           # Block 2: WIT Interface System
├── actor/         # Block 3: Actor System Integration
├── security/      # Block 4: Security & Isolation
├── messaging/     # Block 5: Inter-Component Communication
├── storage/       # Block 6: Persistent Storage
├── lifecycle/     # Block 7: Component Lifecycle
├── component/     # Block 8: Component Management
├── osl/           # Block 9: AirsSys-OSL Bridge
├── monitoring/    # Block 10: Monitoring & Observability
├── installation/  # Installation system
└── util/          # Utility functions
```

**Pros:**
- ✅ Clear 1:1 mapping to implementation blocks
- ✅ Easy to find code (each block has its own module)
- ✅ Intuitive for contributors (matches task structure)
- ✅ Flat structure is easy to navigate
- ✅ Similar to proven airssys-rt pattern

**Cons:**
- ⚠️ No clear abstraction vs implementation separation
- ⚠️ Potential cross-cutting concerns (security touches many modules)
- ⚠️ May encourage tight coupling between domain modules
- ⚠️ Doesn't emphasize layered architecture from ADRs

### Option 2: Layered Architecture (airssys-osl style)

**Structure:**
```
airssys-wasm/src/
├── lib.rs
├── prelude.rs
├── core/          # Core abstractions and traits
│   ├── component.rs
│   ├── capability.rs
│   ├── error.rs
│   └── config.rs
├── runtime/       # WASM runtime layer (Block 1)
├── interface/     # WIT interface system (Block 2)
├── integration/   # AirsSys integration (Blocks 3, 9)
│   ├── actor.rs
│   ├── osl.rs
│   └── ...
├── services/      # Core services (Blocks 4-7)
│   ├── security/
│   ├── messaging/
│   ├── storage/
│   └── lifecycle/
└── management/    # Operations (Blocks 8, 10)
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

### Option 3: Hybrid Block-Aligned with Core ⭐ RECOMMENDED

**Structure:**
```
airssys-wasm/src/
├── lib.rs              # Crate root documentation
├── prelude.rs          # Common re-exports
├── core/               # Core abstractions (foundation)
│   ├── component.rs    # Component trait and types
│   ├── capability.rs   # Capability system abstractions
│   ├── error.rs        # Error types
│   └── config.rs       # Configuration types
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
├── installation/       # Installation system
└── util/               # Utility functions
```

**Pros:**
- ✅ Best of both: Core abstractions + flat domain modules
- ✅ Direct mapping to implementation blocks (easy task alignment)
- ✅ `core/` prevents circular dependencies
- ✅ Flat structure for easy navigation (mostly 2 levels)
- ✅ Follows YAGNI (simpler than full layered approach)
- ✅ Combines proven patterns from both airssys-rt and airssys-osl

**Cons:**
- ⚠️ Still some potential for cross-cutting concerns
- ⚠️ May need sub-modules within some top-level modules

## Decision Outcome

**Chosen option:** "Option 3: Hybrid Block-Aligned with Core"

**Rationale:**
1. ✅ **Clear task alignment** - Direct 1:1 mapping to WASM-TASK-002 through 012
2. ✅ **Prevents circular deps** - `core/` module establishes foundation
3. ✅ **Easy navigation** - Mostly flat, 2-level structure
4. ✅ **YAGNI compliant** - Simpler than full layered architecture
5. ✅ **Proven pattern** - Combines best of airssys-rt and airssys-osl
6. ✅ **Contributor-friendly** - Intuitive structure for new developers

This hybrid approach provides the simplicity and task alignment of the flat domain-driven pattern while incorporating the circular-dependency prevention and abstraction clarity of the layered pattern.

## Module Dependency Rules

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

## Public API Surface

### Prelude Re-Exports Pattern

Following `airssys-rt` and `airssys-osl` patterns:

```rust
//! Prelude module for common airssys-wasm types and traits.

// Core types and traits
pub use crate::core::{
    component::{Component, ComponentConfig, ComponentId, ComponentMetadata},
    capability::{Capability, CapabilitySet},
    error::{WasmError, WasmResult},
    config::RuntimeConfig,
};

// Runtime essentials
pub use crate::runtime::{WasmEngine, WasmInstance, ResourceLimits};

// Actor integration (most common pattern)
pub use crate::actor::{ComponentActor, ComponentMessage};

// Security primitives
pub use crate::security::{CapabilityManager, SecurityPolicy};

// Storage API
pub use crate::storage::{StorageBackend, ComponentStorage};

// Lifecycle management
pub use crate::lifecycle::{DeploymentStrategy, VersionManager};
```

### Lib.rs Module Declaration

Following workspace §4.3 standards (mod.rs declaration-only):

```rust
//! # airssys-wasm - WASM Component Framework for Pluggable Systems
//!
//! [Comprehensive crate-level documentation]

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

// Re-export core error types at crate root
pub use core::error::{WasmError, WasmResult};
```

## Module Responsibility Matrix

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

## Testing Organization

### Unit Tests (Co-located)
Following Rust conventions, unit tests co-located with implementation:

```rust
// In security/capabilities.rs
pub struct CapabilityManager { /* ... */ }

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
Integration tests in `airssys-wasm/tests/` test cross-module interactions:

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

## Implementation Guidelines

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
**Following workspace §4.3 standards:**
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

## Consequences

### Positive Consequences

✅ **Clear Organization**
- New contributors can find code intuitively (module names match tasks)
- Navigation is straightforward (mostly 2-level structure)
- Documentation structure aligns with code structure

✅ **No Circular Dependencies**
- `core/` foundation prevents cycles
- Clear dependency layers enforce acyclic graph
- Refactoring is safer within module boundaries

✅ **Stable Public API**
- `prelude` provides ergonomic single-line imports
- Clear separation of public vs internal APIs
- Semantic versioning for API evolution

✅ **Task-Module Alignment**
- Direct 1:1 mapping to WASM-TASK-002 through 012
- Easy to track implementation progress
- Task documentation references specific modules

✅ **Testing Clarity**
- Unit tests live with implementation
- Integration tests mirror module structure
- End-to-end tests validate full stack

✅ **Workspace Compliance**
- Follows §4.3 module architecture standards (mod.rs declaration-only)
- Follows §2.1 import organization (3-layer pattern)
- Follows §6.1 YAGNI principles (simple, practical)

✅ **Integration Visibility**
- `actor/` and `osl/` modules clearly highlight external dependencies
- Integration points are explicit, not hidden
- Easier to mock/test integration boundaries

✅ **Proven Pattern**
- Combines successful elements from airssys-rt and airssys-osl
- Builds on battle-tested organizational approaches
- Reduces risk of architectural mistakes

### Negative Consequences

⚠️ **Cross-Cutting Concerns**
- Security, error handling, logging may touch many modules
- **Mitigation**: Use `core/` for shared abstractions, avoid duplication

⚠️ **Module Growth**
- Some modules (e.g., `security/`, `lifecycle/`) may grow large
- **Mitigation**: Use submodules when needed (e.g., `security/capabilities/`, `security/policies/`)

⚠️ **Refactoring Impact**
- Moving code between modules may break imports
- **Mitigation**: Use re-exports to maintain API compatibility during refactoring

### Neutral Consequences

📝 **Two-Level Nesting**
- Most modules are 2 levels deep (`src/module/file.rs`)
- Some may grow to 3 levels (`src/module/submodule/file.rs`)
- This is standard Rust practice and acceptable

📝 **Prelude Maintenance**
- Prelude must be updated as public API evolves
- This is intentional - prelude is curated, not automatic

## Compliance and Validation

### Workspace Standards Compliance
- ✅ **§2.1**: 3-layer import organization (all modules follow pattern)
- ✅ **§4.3**: Module architecture (mod.rs declaration-only, no implementation)
- ✅ **§6.1**: YAGNI principles (simple structure, not over-engineered)

### Validation Approach
**During implementation, we will validate:**
1. ✅ No circular dependencies (enforce with `cargo check`)
2. ✅ Module declarations in `mod.rs` only (code review enforcement)
3. ✅ Integration tests align with modules (test naming convention)
4. ✅ Prelude re-exports are up to date (documentation review)
5. ✅ Module-level documentation complete (rustdoc checks)

## References

### Related Documentation
- **KNOWLEDGE-WASM-012**: Module Structure Architecture (complete analysis of all approaches)
- **ADR-WASM-010**: Implementation Strategy and Build Order (11 building blocks)
- **Workspace §4.3**: Module Architecture Standards (mod.rs declaration-only)
- **Workspace §2.1**: 3-Layer Import Organization (all modules must comply)
- **Workspace §6.1**: YAGNI Principles (simplicity over complexity)

### External References
- [The Rust Programming Language - Modules](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)
- [Rust API Guidelines - Organization](https://rust-lang.github.io/api-guidelines/organization.html)
- [Microsoft Rust Guidelines - Module Organization](https://microsoft.github.io/rust-guidelines/)

### AirsSys Ecosystem Patterns
- **airssys-rt/src/lib.rs**: Flat domain-driven pattern reference
- **airssys-osl/src/lib.rs**: Layered with core pattern reference

## Notes

**Foundation First:**
The `core/` module establishes the foundation. All types here must be stable, minimal, and well-designed before building on top.

**Block Alignment:**
Module structure directly maps to implementation blocks (WASM-TASK-002 through 012). This makes task tracking and implementation straightforward.

**YAGNI Application:**
We're choosing the simpler hybrid approach (Option 3) over the more complex layered approach (Option 2). This follows YAGNI - we can refactor to deeper layering later if proven necessary.

**Proven Patterns:**
We're combining the best of airssys-rt (flat, domain-driven) and airssys-osl (core abstractions) based on their successful implementations.

**Integration Clarity:**
Modules `actor/` and `osl/` explicitly highlight integration with airssys-rt and airssys-osl, making external dependencies clear.

**Future Flexibility:**
The structure supports future growth. If a module becomes too large, we can add submodules without breaking the overall organization.

**Learning from Success:**
Both airssys-rt and airssys-osl have proven their module structures work well at scale. This decision leverages that success rather than experimenting with unproven patterns.
