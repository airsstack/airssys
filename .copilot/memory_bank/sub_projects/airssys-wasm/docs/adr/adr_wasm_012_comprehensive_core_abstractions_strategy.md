# ADR-WASM-012: Comprehensive Core Abstractions Strategy

**Status:** Accepted  
**Date:** 2025-10-21  
**Decision Makers:** Architecture Team  
**Related:** ADR-WASM-011 (Module Structure), KNOWLEDGE-WASM-012 (Module Architecture), WASM-TASK-000 (Core Abstractions Design)

---

## Context

The airssys-wasm framework consists of 11 implementation blocks (runtime, WIT, actor, security, messaging, storage, lifecycle, component management, OSL bridge, monitoring, SDK). Following ADR-WASM-011's hybrid module structure with `core/` module, we face a critical architectural decision: **What abstractions should live in `core/`, and what should live in domain modules?**

### Current Understanding

**ADR-WASM-011 Module Structure:**
```
airssys-wasm/src/
├── core/               # Core abstractions (foundation)
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
└── util/               # Utilities and helpers
```

**Initial WASM-TASK-000 Scope (Before This ADR):**
- Only 4 files in `core/`: component.rs, capability.rs, error.rs, config.rs
- Focused on "universal" types (ComponentId, Capability, WasmError)
- Missing domain-specific abstractions for each block

### The Fundamental Question

**Should `core/` contain ONLY universal types, or should it include domain-specific abstractions for each implementation block?**

**Option 1: Minimal Core (Universal Types Only)**
```
core/
├── component.rs    # Component, ComponentId, ComponentMetadata
├── capability.rs   # Capability, CapabilitySet
├── error.rs        # WasmError, WasmResult
└── config.rs       # RuntimeConfig
```
- ✅ Small, focused core
- ❌ Each block defines own types (potential inconsistency)
- ❌ Risk of circular dependencies between blocks
- ❌ No shared contracts between blocks

**Option 2: Comprehensive Core (Domain Abstractions Per Block)**
```
core/
├── component.rs      # Universal component types
├── capability.rs     # Universal capability types
├── error.rs          # Universal error types
├── config.rs         # Universal config types
├── runtime.rs        # Runtime engine abstractions (Block 1)
├── interface.rs      # WIT interface abstractions (Block 2)
├── actor.rs          # Actor integration abstractions (Block 3)
├── security.rs       # Security policy abstractions (Block 4)
├── messaging.rs      # Messaging protocol abstractions (Block 5)
├── storage.rs        # Storage backend abstractions (Block 6)
├── lifecycle.rs      # Lifecycle state abstractions (Block 7)
├── management.rs     # Component management abstractions (Block 8)
├── bridge.rs         # OSL bridge abstractions (Block 9)
└── observability.rs  # Monitoring abstractions (Block 10)
```
- ✅ Complete type system upfront
- ✅ Prevents circular dependencies
- ✅ Shared contracts via traits
- ⚠️ Larger core (3-4 weeks vs 1-2 weeks)

### Problem Statement

**Without comprehensive core abstractions:**
1. **Type Inconsistency:** Block 1 defines `RuntimeConfig`, Block 4 defines `SecurityConfig` - no shared patterns
2. **Circular Dependencies:** Block 5 (messaging) needs Block 3 (actor) types, Block 3 needs Block 5 types → deadlock
3. **Refactoring Nightmare:** Changing a type in one block breaks multiple dependent blocks
4. **Testing Difficulty:** Cannot mock implementations without trait contracts
5. **API Instability:** Public API changes ripple across entire codebase
6. **Parallel Development:** Teams cannot work independently without shared type contracts

**Real-World Example from Analysis:**
- **Block 1 (runtime/)** will create: `WasmEngine`, `ComponentInstance`, `MemoryLimits`, `FuelLimiter`
- **Block 3 (actor/)** will create: `ComponentActor`, `ActorMessage`, `SupervisionStrategy`
- **Block 5 (messaging/)** needs to reference both runtime instances AND actor types
- Without core abstractions: messaging → actor → runtime → circular dependency potential

### Precedents

**airssys-osl Pattern:**
```
airssys-osl/src/core/
├── context.rs      # Universal execution context
├── executor.rs     # OSExecutor trait (abstraction)
├── middleware.rs   # Middleware trait (abstraction)
├── operation.rs    # Operation trait (abstraction)
├── result.rs       # OSResult, OSError
└── security.rs     # SecurityContext, Permission
```
- Establishes traits FIRST (Executor, Middleware, Operation)
- Implementations in `executors/`, `middleware/`, `operations/`
- Zero circular dependencies

**Rust Standard Library Pattern:**
```
std::io/
├── mod.rs          # Read, Write, Seek traits (abstractions)
├── buffered.rs     # BufReader, BufWriter (implementations)
├── cursor.rs       # Cursor (implementation)
└── ...
```
- Core traits (Read, Write, Seek) in root module
- Concrete implementations in submodules
- All implementations satisfy trait contracts

---

## Decision

### Primary Decision: Comprehensive Core Abstractions Strategy

**We will implement comprehensive core abstractions covering ALL implementation blocks (Blocks 1-11) in the `core/` module.**

**Scope of `core/` Module:**

**1. Universal Abstractions (Already Planned):**
- `component.rs` - Component, ComponentId, ComponentMetadata, ResourceLimits, ComponentInput/Output
- `capability.rs` - Capability enum, CapabilitySet, pattern types (PathPattern, DomainPattern, etc.)
- `error.rs` - WasmError enum (14+ variants), WasmResult<T>, error constructors
- `config.rs` - RuntimeConfig, SecurityConfig, StorageConfig with defaults

**2. Domain-Specific Abstractions (New):**
- `runtime.rs` - Runtime engine traits, execution context types, resource limit abstractions
- `interface.rs` - WIT interface metadata types, binding abstractions, type descriptors
- `actor.rs` - Actor message envelopes, supervision strategy types, actor state abstractions
- `security.rs` - Security policy traits, permission request/response types, isolation boundaries
- `messaging.rs` - Message protocol types, routing abstractions, delivery guarantees
- `storage.rs` - Storage backend traits, operation types, transaction abstractions
- `lifecycle.rs` - Lifecycle state machines, transition types, version management
- `management.rs` - Component registry abstractions, installation types, update strategies
- `bridge.rs` - Host function abstraction traits, OSL integration types, capability mapping
- `observability.rs` - Metrics collection traits, event types, monitoring abstractions

**Each domain-specific core file contains:**
- **Traits** - Behavior contracts (e.g., `trait RuntimeEngine`, `trait StorageBackend`)
- **Enums** - State machines and variants (e.g., `LifecycleState`, `MessageType`)
- **Structs** - Data containers (e.g., `ExecutionContext`, `MessageEnvelope`)
- **Type Aliases** - Convenience types (e.g., `RuntimeResult<T> = Result<T, RuntimeError>`)
- **Constants** - Shared constants (e.g., `DEFAULT_MEMORY_LIMIT`, `MAX_RETRIES`)

**What DOES NOT go in core/:**
- ❌ Concrete implementations (e.g., `WasmEngine`, `SledBackend`)
- ❌ External crate integrations (e.g., wasmtime-specific code, sled-specific code)
- ❌ Business logic (algorithm implementations)
- ❌ Helper utilities (those go in `util/`)

### Supporting Decisions

#### 1. Abstractions-First Development

**Decision:** Design and implement ALL core abstractions BEFORE any implementation block work begins.

**Rationale:**
- Prevents circular dependencies by establishing compilation order
- Enables parallel block development (teams share type contracts)
- Facilitates test-driven development (tests against traits before implementation)
- Ensures API stability from day one

**Implementation:**
- WASM-TASK-000 must complete fully before WASM-TASK-001 (planning)
- All blocks (002-012) depend on completed core abstractions
- Core changes require cross-block impact analysis and coordination

#### 2. Trait-Centric Design

**Decision:** Domain abstractions should be trait-centric, defining behavior contracts that implementations satisfy.

**Rationale:**
- **Testability:** Traits enable mock implementations for unit tests
- **Flexibility:** Multiple implementations can satisfy same contract (e.g., InMemoryStorage, SledStorage, RocksDBStorage)
- **Decoupling:** Consumers depend on traits, not concrete types
- **Future-Proofing:** New implementations don't break existing code

**Pattern:**
```rust
// core/storage.rs - Abstraction
pub trait StorageBackend: Send + Sync {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError>;
    async fn set(&self, key: &[u8], value: &[u8]) -> Result<(), StorageError>;
    async fn delete(&self, key: &[u8]) -> Result<(), StorageError>;
}

// storage/sled.rs - Implementation
pub struct SledBackend { /* ... */ }

impl StorageBackend for SledBackend {
    async fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, StorageError> {
        // Sled-specific implementation
    }
    // ...
}
```

#### 3. Zero Internal Dependencies for Core

**Decision:** The `core/` module MUST have zero internal dependencies within airssys-wasm.

**Rationale:**
- Prevents circular dependencies (core depends on nothing, everything depends on core)
- Guarantees compilation order (core compiles first)
- Simplifies dependency graph (linear dependencies, not mesh)
- Enables independent core evolution

**Allowed Dependencies:**
- ✅ External crates (serde, thiserror, chrono, etc.)
- ✅ Rust standard library
- ❌ ANY internal airssys-wasm modules

**Enforcement:**
- Cargo will fail compilation if core imports from other modules
- Code review checklist includes core dependency verification
- CI/CD checks for core module dependency violations

#### 4. Expanded WASM-TASK-000 Timeline

**Decision:** Increase WASM-TASK-000 effort estimate from 1-2 weeks to 3-4 weeks to accommodate comprehensive core design.

**Rationale:**
- Designing 14 core files (vs original 4 files) requires more time
- Each domain abstraction needs architectural review
- Trait design requires iteration and validation
- Investment in strong core prevents weeks/months of refactoring later

**Justification:**
- **Time Investment:** +2 weeks upfront design
- **Time Saved:** Prevents 4-8 weeks of refactoring across 11 blocks
- **Quality Improvement:** Stable API, zero circular dependencies, testable architecture
- **Risk Mitigation:** Catches design flaws before implementation commits

---

## Rationale

### Why Comprehensive Core Abstractions?

**1. Type Safety and Consistency**
- **Problem:** Without core abstractions, each block invents own types
- **Solution:** Core defines canonical types, blocks use them
- **Example:** `ComponentId` defined once in core, used everywhere consistently

**2. Dependency Graph Simplification**
- **Problem:** Block A needs Block B types, Block B needs Block A types → circular dependency
- **Solution:** Both blocks depend on core abstractions, not each other
- **Example:** messaging (Block 5) and actor (Block 3) both depend on core/messaging.rs and core/actor.rs

**3. Testability and Mocking**
- **Problem:** Cannot test Block A without full Block B implementation
- **Solution:** Mock implementations of core traits for isolated testing
- **Example:** Test messaging with `MockStorageBackend` implementing `core::storage::StorageBackend`

**4. Parallel Development**
- **Problem:** Blocks cannot be developed independently due to type dependencies
- **Solution:** Once core complete, all blocks can develop in parallel
- **Example:** Team A works on runtime (Block 1), Team B works on storage (Block 6) - no conflicts

**5. API Stability**
- **Problem:** Changes to one block's types break multiple dependent blocks
- **Solution:** Core API changes are rare and coordinated, block-internal changes don't propagate
- **Example:** Changing `WasmEngine` implementation doesn't affect blocks using `core::runtime::RuntimeEngine` trait

**6. Refactoring Safety**
- **Problem:** Refactoring one block requires changes across entire codebase
- **Solution:** Refactoring isolated to single block if core contract maintained
- **Example:** Switching from Sled to RocksDB only changes storage/ module, not messaging, actor, lifecycle, etc.

### Precedents Supporting This Decision

**1. airssys-osl Success Story**
- Implemented strong `core/` with executor, middleware, operation traits
- Zero circular dependencies achieved
- Enabled framework development with stable API
- Lesson: Upfront core design investment pays off

**2. Rust Standard Library Design**
- `std::io` defines Read/Write/Seek traits
- All I/O implementations (File, TcpStream, Cursor) satisfy contracts
- Lesson: Traits enable flexible implementations with stable contracts

**3. Tokio Runtime Architecture**
- Tokio defines `Future` trait abstraction
- Runtime implementations (CurrentThread, MultiThread) satisfy contract
- Lesson: Core abstractions enable ecosystem of compatible implementations

**4. Industry Best Practices**
- **Gang of Four Design Patterns:** "Program to an interface, not an implementation"
- **SOLID Principles:** Dependency Inversion Principle (depend on abstractions)
- **Domain-Driven Design:** Bounded contexts with explicit contracts

### Risk Mitigation

**Risk: Over-Engineering Core**
- **Mitigation:** Follow YAGNI - only abstractions needed by current blocks (1-11)
- **Validation:** Each abstraction must map to concrete block requirement
- **Review:** Architectural review catches unnecessary complexity

**Risk: Core Design Mistakes**
- **Mitigation:** Iterative design with peer review before implementation
- **Validation:** Prototype key abstractions with sample implementations
- **Flexibility:** Core can evolve but breaking changes require coordination

**Risk: Extended Timeline**
- **Mitigation:** 3-4 weeks investment prevents months of refactoring
- **Validation:** Track time saved by avoiding circular dependency fixes
- **Acceptance:** Strong foundations worth the upfront time

---

## Consequences

### Positive Consequences

**1. Zero Circular Dependencies**
- Core has zero internal dependencies
- Blocks depend only on core, not each other
- Compilation order guaranteed

**2. Stable Public API**
- Core defines public API surface
- Block-internal changes don't affect consumers
- API evolution controlled and coordinated

**3. Enhanced Testability**
- Mock implementations trivial with traits
- Unit tests isolated from external dependencies
- Integration tests use real implementations

**4. Parallel Development Enabled**
- Teams work on blocks independently
- Type contracts defined upfront
- Integration happens via core contracts

**5. Refactoring Confidence**
- Changes isolated to single blocks
- Core contract maintained = no ripple effects
- Safe to swap implementations (Sled → RocksDB)

**6. Better Documentation**
- Core module serves as API reference
- Rustdoc for core types documents contracts
- Developers understand system via core docs

### Negative Consequences

**1. Extended WASM-TASK-000 Timeline**
- **Impact:** +2 weeks upfront (3-4 weeks total vs 1-2 weeks)
- **Mitigation:** Time investment justified by refactoring prevention
- **Acceptance:** Strong foundations worth the time

**2. Increased Core Complexity**
- **Impact:** 14 files in core/ vs original 4 files
- **Mitigation:** Clear separation of concerns, domain-specific files
- **Acceptance:** Complexity managed via organization, not avoided

**3. More Upfront Design Needed**
- **Impact:** Must design abstractions before implementation
- **Mitigation:** Iterative design with prototyping
- **Acceptance:** Design-first approach prevents technical debt

**4. Coordination Required for Core Changes**
- **Impact:** Core changes require cross-block analysis
- **Mitigation:** Core API should be stable after initial design
- **Acceptance:** Rare core changes acceptable tradeoff for stability

### Migration Strategy

**Phase 1: Core Foundation (WASM-TASK-000 - Weeks 1-4)**
1. Design universal abstractions (component, capability, error, config) - Week 1
2. Design Block 1-3 abstractions (runtime, interface, actor) - Week 2
3. Design Block 4-7 abstractions (security, messaging, storage, lifecycle) - Week 3
4. Design Block 8-10 abstractions (management, bridge, observability) - Week 4
5. Comprehensive review, validation, and rustdoc

**Phase 2: Validation (WASM-TASK-001 - Week 5)**
1. Validate abstractions against all 9 ADRs
2. Prototype key traits with sample implementations
3. Get peer architectural review
4. Finalize core API

**Phase 3: Implementation (WASM-TASK-002+ - Weeks 6+)**
1. Blocks implement core trait contracts
2. Core abstractions remain stable
3. Only core additions (not breaking changes) allowed

---

## Alternatives Considered

### Alternative 1: Minimal Core (Universal Types Only)

**Description:** Keep core small with only component, capability, error, config.

**Pros:**
- ✅ Faster initial implementation (1-2 weeks)
- ✅ Smaller core module
- ✅ Less upfront design

**Cons:**
- ❌ Each block defines own types (inconsistency)
- ❌ Risk of circular dependencies between blocks
- ❌ Refactoring nightmare later
- ❌ Difficult parallel development
- ❌ API instability

**Why Rejected:** Short-term speed gain, long-term maintenance nightmare. Circular dependency risk too high for complex system.

### Alternative 2: Domain Module Hierarchies (No Shared Core)

**Description:** Each block defines its own abstractions in sub-modules.

**Example:**
```
runtime/
├── traits.rs    # Runtime-specific traits
├── types.rs     # Runtime-specific types
└── engine.rs    # Implementation

storage/
├── traits.rs    # Storage-specific traits
├── types.rs     # Storage-specific types
└── backend.rs   # Implementation
```

**Pros:**
- ✅ Each block self-contained
- ✅ No central coordination needed

**Cons:**
- ❌ Duplicate type definitions across blocks
- ❌ Circular dependencies between blocks inevitable
- ❌ No shared contracts
- ❌ Difficult to ensure consistency

**Why Rejected:** Violates DRY principle, creates tight coupling, prevents parallel development.

### Alternative 3: Gradual Core Evolution (Minimal → Comprehensive)

**Description:** Start with minimal core, add domain abstractions as blocks are implemented.

**Pros:**
- ✅ Faster initial start
- ✅ Abstractions emerge from real needs

**Cons:**
- ❌ Core API changes break existing blocks
- ❌ Refactoring required as core expands
- ❌ Risk of not discovering abstractions until late
- ❌ Blocks may be built without considering future needs

**Why Rejected:** Reactive approach leads to technical debt. Better to design comprehensively upfront.

---

## Related Decisions

### ADR-WASM-011: Module Structure Organization
- **Relationship:** Defines THAT core/ exists, this ADR defines WHAT goes in core/
- **Impact:** Comprehensive core strategy refines module structure decision
- **Consistency:** Both ADRs align on hybrid block-aligned with core pattern

### KNOWLEDGE-WASM-012: Module Structure Architecture
- **Relationship:** Documents module organization analysis that led to ADR-WASM-011
- **Impact:** Core abstractions fulfill dependency management goals
- **Consistency:** Dependency graph rules enforced by comprehensive core

### WASM-TASK-000: Core Abstractions Design
- **Relationship:** This ADR documents the architectural decision, TASK-000 is implementation plan
- **Impact:** Task expanded from 4 files to 14 files, timeline 1-2 weeks → 3-4 weeks
- **Dependency:** Task must align with this ADR's scope and principles

---

## Implementation Notes

### Core Module Files (14 files)

**Universal Abstractions:**
1. `component.rs` - Component, ComponentId, ComponentMetadata, ResourceLimits
2. `capability.rs` - Capability, CapabilitySet, pattern types
3. `error.rs` - WasmError, WasmResult, error constructors
4. `config.rs` - RuntimeConfig, SecurityConfig, StorageConfig

**Domain-Specific Abstractions:**
5. `runtime.rs` - RuntimeEngine trait, ExecutionContext, resource limits
6. `interface.rs` - InterfaceDefinition, BindingMetadata, TypeDescriptor
7. `actor.rs` - ActorMessage, SupervisionStrategy, ActorState
8. `security.rs` - SecurityPolicy trait, PermissionRequest, IsolationBoundary
9. `messaging.rs` - MessageEnvelope, RoutingStrategy, DeliveryGuarantee
10. `storage.rs` - StorageBackend trait, StorageOperation, Transaction
11. `lifecycle.rs` - LifecycleState, VersionStrategy, UpdatePolicy
12. `management.rs` - ComponentRegistry trait, InstallationMetadata, UpdateStrategy
13. `bridge.rs` - HostFunction trait, CapabilityMapping, OSLIntegration
14. `observability.rs` - MetricsCollector trait, Event, MonitoringConfig

### Validation Checklist

**For Each Core Abstraction File:**
- [ ] Trait definitions for extensibility points
- [ ] Enum types for state machines and variants
- [ ] Struct types for data containers
- [ ] Type aliases for convenience
- [ ] Constants for shared configuration
- [ ] Comprehensive rustdoc with examples
- [ ] Zero internal dependencies (only external crates)
- [ ] Maps to at least one implementation block requirement
- [ ] Reviewed against relevant ADRs
- [ ] Peer architectural review completed

### Success Criteria

**WASM-TASK-000 considered successful when:**
1. All 14 core files implemented with comprehensive rustdoc
2. Zero circular dependencies verified via cargo check
3. All core types used by at least one block specification
4. Peer architectural review approval obtained
5. Prototype implementations validate key traits
6. >90% test coverage for core types
7. Core API validated against all 9 ADRs

---

## Review and Updates

**Review Schedule:** 
- After WASM-TASK-000 completion (validate abstractions)
- After first 3 blocks complete (validate trait sufficiency)
- After full implementation (validate no circular dependencies)

**Approval Required For:**
- Adding new core abstraction files (requires ADR update)
- Breaking changes to core API (requires cross-block coordination)
- Removing core abstractions (requires impact analysis)

**Document History:**
- 2025-10-21: Initial version - Comprehensive core abstractions strategy accepted

---

## References

**Internal Documentation:**
- ADR-WASM-011: Module Structure Organization
- KNOWLEDGE-WASM-012: Module Structure Architecture
- WASM-TASK-000: Core Abstractions Design
- airssys-osl core/ module implementation

**External References:**
- [Microsoft Rust Guidelines: Type Hierarchy](https://microsoft.github.io/rust-guidelines/)
- [Rust API Guidelines: Trait Design](https://rust-lang.github.io/api-guidelines/)
- [Gang of Four Design Patterns: Program to Interface](https://en.wikipedia.org/wiki/Design_Patterns)
- [SOLID Principles: Dependency Inversion](https://en.wikipedia.org/wiki/Dependency_inversion_principle)
