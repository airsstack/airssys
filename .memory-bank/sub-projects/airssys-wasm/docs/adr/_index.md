# airssys-wasm Architecture Decision Records Index

**Sub-Project:** airssys-wasm  
**Last Updated:** 2025-12-21  
**Total ADRs:** 16  
**Active ADRs:** 15  

## Active ADRs

### ADR-WASM-001: Multicodec Compatibility Strategy
- **Status:** Accepted
- **Date:** 2025-10-19
- **Category:** Serialization & Interoperability
- **Summary:** Host runtime validates codec compatibility but does NOT translate between codecs. Components are responsible for implementing their own codec support. Fail-fast approach for incompatible codecs.
- **Related:** KNOWLEDGE-WASM-006 (Multiformat), KNOWLEDGE-WASM-011 (Serialization)
- **Impact:** Critical - Foundation for inter-component messaging architecture
- **File:** `adr_wasm_001_multicodec_compatibility_strategy.md`

### ADR-WASM-002: WASM Runtime Engine Selection
- **Status:** Accepted
- **Date:** 2025-10-19
- **Category:** Core Runtime & Execution
- **Summary:** Wasmtime as primary runtime with JIT compilation, async-first architecture, mandatory engineer-defined memory limits, hybrid fuel+timeout CPU limiting, host-only enforcement, isolated component crashes.
- **Related:** KNOWLEDGE-WASM-001 (Component Framework), KNOWLEDGE-WASM-003 (Core Architecture)
- **Impact:** Critical - Most foundational decision, affects all subsequent architecture
- **Key Decisions:**
  - Runtime: Wasmtime (Component Model reference implementation)
  - Compilation: JIT with Cranelift (AOT optional future enhancement)
  - Async: Mandatory async-first (Tokio integration for airssys-rt)
  - Memory: REQUIRED in Component.toml (no defaults, engineer-defined)
  - CPU: Hybrid fuel metering + wall-clock timeout (dual protection)
  - Enforcement: Host runtime only (Phase 1 simplicity)
  - Errors: Isolated crashes (supervisor pattern, production resilience)
- **File:** `adr_wasm_002_wasm_runtime_engine_selection.md`

### ADR-WASM-003: Component Lifecycle Management
- **Status:** Accepted
- **Date:** 2025-10-19
- **Category:** Component System Architecture
- **Summary:** Immutable components with automatic retention policies (blockchain-inspired pattern). Three installation sources (Git reproducible, Local fast dev, Remote URL pre-built), 2-state lifecycle (Installed/Uninstalled), routing proxy for blue-green deployment, cryptographic Ed25519 ownership, actor-based isolation, configurable retention (default 24h rollback window with auto-cleanup).
- **Related:** KNOWLEDGE-WASM-001 (Component Framework), KNOWLEDGE-WASM-009 (Installation), ADR-WASM-002 (Runtime), ADR-WASM-005 (Security), ADR-WASM-007 (Storage)
- **Impact:** Critical - Foundation for component management and updates
- **File:** `adr_wasm_003_component_lifecycle_management.md`

### ADR-WASM-005: Capability-Based Security Model
- **Status:** Accepted
- **Date:** 2025-10-19
- **Category:** Security Architecture
- **Summary:** Fine-grained capability-based security with pattern matching for resources (filesystem globs, network domains, storage namespaces), trust-level system for auto-approval workflows (trusted sources instant install, unknown sources require review), layered integration with airssys-osl RBAC/ACL for defense-in-depth.
- **Related:** KNOWLEDGE-WASM-001 (Component Framework), KNOWLEDGE-WASM-004 (WIT Management), ADR-WASM-002 (Runtime Engine)
- **Impact:** Critical - Security foundation for all component operations
- **File:** `adr_wasm_005_capability_based_security_model.md`

### ADR-WASM-006: Component Isolation and Sandboxing (Revised)
- **Status:** Accepted (Revised 2025-10-19)
- **Date:** 2025-10-19 (Original), 2025-10-19 (Revised)
- **Category:** Security & Isolation Architecture
- **Summary:** Erlang-style lightweight process isolation using airssys-rt actors, combined with WASM linear memory sandboxing and hybrid resource enforcement. 4-layer defense in depth (Capability → WASM → Actor → Supervision), ComponentActor dual trait design with WASM lifecycle in Child::start()/stop(), inter-component messaging via MessageBroker (see ADR-WASM-009).
- **Related:** KNOWLEDGE-WASM-001 (Component Framework), KNOWLEDGE-RT-013 (Actor Performance), ADR-WASM-002 (Runtime), ADR-WASM-003 (Lifecycle), ADR-WASM-005 (Security), ADR-WASM-007 (Storage), ADR-WASM-009 (Communication), ADR-RT-004 (Actor/Child Separation), ADR-WASM-018 (Three-Layer Architecture)
- **Impact:** Critical - Security foundation for untrusted third-party components
- **File:** `adr_wasm_006_component_isolation_and_sandboxing.md`

### ADR-WASM-007: Storage Backend Selection
- **Status:** Accepted
- **Date:** 2025-10-19
- **Category:** Storage Architecture
- **Summary:** NEAR-style key-value storage API with Sled as default backend (pure Rust benefits) and RocksDB as optional fallback (proven stability), prefix-based component isolation, application-level quota tracking, NO transactions (actor model sequential processing eliminates need), export/import JSON Lines tool for migration and backups.
- **Related:** KNOWLEDGE-WASM-007 (Component Storage), KNOWLEDGE-WASM-008 (Backend Comparison), ADR-WASM-005 (Security)
- **Impact:** Critical - Persistent storage foundation for all components
- **File:** `adr_wasm_007_storage_backend_selection.md`

### ADR-WASM-009: Component Communication Model
- **Status:** Accepted
- **Date:** 2025-10-19
- **Category:** Communication Architecture
- **Summary:** Message-passing via airssys-rt MessageBroker with host-mediated security enforcement. Supports fire-and-forget (one-way), request-response (async RPC with callbacks), and pub-sub patterns. Pure pub-sub architecture with ActorSystem as primary subscriber, security enforcement at host function layer (capability validation + quota enforcement + audit logging), multicodec self-describing serialization for cross-language communication.
- **Related:** KNOWLEDGE-WASM-005 (Messaging Implementation), ADR-WASM-001 (Multicodec), ADR-WASM-005 (Security), ADR-WASM-006 (Isolation), RT-TASK-008 (MessageBroker Performance)
- **Impact:** Critical - Foundation for all inter-component communication
- **File:** `adr_wasm_009_component_communication_model.md`

### ADR-WASM-010: Implementation Strategy and Build Order
- **Status:** Accepted
- **Date:** 2025-10-20
- **Category:** Implementation Planning & Architecture
- **Summary:** 4-layer phased implementation strategy with Actor System Integration as foundational component (Layer 1, Block 3), NOT an integration layer component. Defines 11 major building blocks with correct dependency order, preventing circular dependencies and architectural rework. Critical correction: airssys-rt integration must come early as foundation (Block 3) because Inter-Component Communication (Block 5) and Component Lifecycle (Block 7) depend on MessageBroker and SupervisorNode.
- **Related:** All ADRs (implementation guidance), KNOWLEDGE-WASM-001 (Component Framework), KNOWLEDGE-WASM-005 (Messaging), ADR-WASM-006 (Actor Isolation), ADR-WASM-009 (MessageBroker dependency)
- **Impact:** Critical - Defines entire development roadmap and prevents build order mistakes
- **File:** `adr_wasm_010_implementation_strategy_and_build_order.md`

### ADR-WASM-011: Module Structure Organization
- **Status:** Accepted
- **Date:** 2025-10-21
- **Category:** Code Architecture & Organization
- **Summary:** Hybrid block-aligned module structure combining flat domain-driven organization (airssys-rt pattern) with core abstraction layer (airssys-osl pattern). Direct 1:1 mapping to 11 implementation blocks, core/ module prevents circular dependencies, prelude re-exports for ergonomic API, mod.rs declaration-only following workspace §4.3.
- **Related:** ADR-WASM-010 (Implementation Strategy), KNOWLEDGE-WASM-012 (Module Structure Architecture), Workspace §4.3 (Module Architecture), §2.1 (Imports), §6.1 (YAGNI)
- **Impact:** Critical - Defines entire crate organization and code structure for all implementations
- **File:** `adr_wasm_011_module_structure_organization.md`

### ADR-WASM-012: Comprehensive Core Abstractions Strategy
- **Status:** Accepted
- **Date:** 2025-10-21
- **Category:** Architecture & Type System Design
- **Summary:** Comprehensive core abstractions covering ALL implementation blocks (1-11). Core module contains universal types (component, capability, error, config) PLUS domain-specific abstractions for each block (runtime, interface, actor, security, messaging, storage, lifecycle, management, bridge, observability). Trait-centric design with zero internal dependencies.
- **Related:** ADR-WASM-011 (Module Structure), KNOWLEDGE-WASM-012 (Module Architecture), WASM-TASK-000 (Core Design)
- **Impact:** CRITICAL - Prevents circular dependencies, enables parallel development, ensures API stability
- **File:** `adr_wasm_012_comprehensive_core_abstractions_strategy.md`

### ADR-WASM-013: StorageTransaction Removal (YAGNI Simplification)
- **Status:** Accepted
- **Date:** 2025-10-22
- **Category:** Storage Architecture Simplification
- **Summary:** Remove unused StorageTransaction trait from storage.rs per YAGNI principle. Actor model sequential processing eliminates need for transactions. Aligns with ADR-WASM-007 Decision 5 rationale. Simplifies API surface, reduces maintenance burden, eliminates Box<dyn> pattern complexity.
- **Related:** ADR-WASM-007 (Storage Backend Selection, Decision 5), ADR-WASM-006 (Actor Isolation), KNOWLEDGE-WASM-007 (Component Storage)
- **Impact:** Low - Removes unused abstraction with zero usage across codebase
- **File:** `adr_wasm_013_storage_transaction_removal.md`

### ADR-WASM-014: RoutingStrategy Removal (YAGNI Simplification)
- **Status:** Accepted
- **Date:** 2025-10-22
- **Category:** Messaging Architecture Simplification
- **Summary:** Remove unused RoutingStrategy trait from messaging.rs per YAGNI principle. Routing handled exclusively by airssys-rt MessageBroker per ADR-WASM-009 architecture. No pluggable routing strategies in design. Simplifies API surface, removes fictional documentation, eliminates Box<dyn> pattern complexity.
- **Related:** ADR-WASM-009 (Component Communication Model), ADR-WASM-006 (Actor Isolation), ADR-WASM-005 (Security Model), KNOWLEDGE-WASM-005 (Messaging Architecture)
- **Impact:** Low - Removes unused abstraction with zero usage across codebase
- **File:** `adr_wasm_014_routing_strategy_removal.md`

### ADR-WASM-015: WIT Package Structure Organization
- **Status:** Accepted
- **Date:** 2025-10-25
- **Category:** Interface Design & Organization
- **Summary:** Adopt directory-based package structure with semantic naming following pattern `airssys:{directory}-{type}@{version}`. Resolves discrepancy between WASM-TASK-003 Phase 1 plan (6 separate WIT files) and delivery (2 consolidated packages). Establishes 7-package structure: 4 core packages (types, component, capabilities, host) and 3 extension packages (filesystem, network, process).
- **Related:** WASM-TASK-003 Phase 1, ADR-WASM-011 (Module Structure), KNOWLEDGE-WASM-004 (WIT Management)
- **Impact:** High - Defines definitive WIT package organization, resolves structural inconsistency, enables proper Phase 2 continuation
- **File:** `adr_wasm_015_wit_package_structure_organization.md`

### ADR-WASM-018: Three-Layer Architecture and Boundary Definitions ⭐ NEW
- **Status:** ACCEPTED
- **Date:** 2025-12-14
- **Category:** Architecture & Integration Design
- **Summary:** Explicit three-layer architecture with clear ownership boundaries: Layer 1 (WASM Component Configuration & Tracking), Layer 2 (WASM Component Lifecycle & Spawning), Layer 3 (Actor System Runtime - airssys-rt). Eliminates architectural confusion regarding actor system ownership, component definition, and feature implementation location. Prevents duplicate supervision logic, confirms airssys-wasm uses airssys-rt (not replacement), and codifies "component = WASM binary + dedicated actor" definition.
- **Related:** ADR-WASM-006 (Component Isolation), ADR-RT-004 (Actor/Child Separation), ADR-WASM-001 (Multicodec), KNOWLEDGE-WASM-018 (Component Definitions - Detailed Reference)
- **Impact:** CRITICAL - Architectural foundation for all Phase 3+ development, prevents major design errors
- **Key Decisions:**
  - Layer 1: SupervisorConfig, BackoffStrategy, ComponentSupervisor (WASM-specific)
  - Layer 2: ComponentActor, ComponentSpawner, ComponentRegistry, WasmRuntime (Component lifecycle)
  - Layer 3: ActorSystem, SupervisorNode, MessageBroker, Mailbox (Actor system - airssys-rt provides)
  - One-way dependency: Layer 2 → Layer 3 (airssys-wasm uses airssys-rt)
  - Component definition: ComponentActor (Actor + Child traits) wrapping loaded WASM binary
  - Explicit non-ownership: airssys-wasm does NOT own actor system, mailbox, message broker, or supervision strategies
- **Rationale:** Clarifies architecture after Phase 2 integration work revealed potential confusion points. Provides unambiguous reference for future developers.
- **File:** `adr_wasm_018_three_layer_architecture.md`

---

## Active Knowledge Documents

### KNOWLEDGE-WASM-018: Component Definitions and Three-Layer Architecture ⭐ NEW
- **Status:** CURRENT
- **Date:** 2025-12-14
- **Revision:** 1.0
- **Category:** Architecture Reference & Development Guidelines
- **Summary:** Comprehensive reference for: (1) What is a "Component" - precise runtime definition with code examples, (2) Three-Layer Architecture detailed breakdown with responsibilities matrix, (3) Ownership matrix showing which team owns what, (4) Integration patterns with data flow diagrams, (5) Development guidelines - "where should I put this feature" decision tree, (6) Common questions and answers, (7) Phase evolution roadmap
- **Related ADR:** ADR-WASM-018 (Three-Layer Architecture - decision record)
- **Purpose:** Definitive reference for component architecture, eliminates ambiguity in future development
- **Key Sections:**
  - Part 1: Component Definition (1.1-1.5) - What is a component at creation time vs. runtime
  - Part 2: Three-Layer Architecture Detailed (2.1-2.3) - Full breakdown with code examples
  - Part 3: Ownership & Responsibility Matrix (3.1-3.3) - Clear feature ownership, dependency flow
  - Part 4: Development Guidelines (4.1-4.4) - Decision tree, checklist, anti-patterns, integration checklist
  - Part 5: FAQs (Q1-Q7) - Common architectural questions
  - Part 6: Phase Evolution - How architecture evolves through phases
  - Appendix: Quick Reference - File locations and key types by layer
- **File:** `knowledge_wasm_018_component_definitions_and_architecture_layers.md`

---

## Planned ADR Categories (Future)

### WASM Runtime Decisions
- **ADR-001: WASM Runtime Selection** - wasmtime vs wasmer vs custom runtime
- **ADR-002: Component Model Implementation** - WebAssembly Component Model approach
- **ADR-003: WASI Implementation Strategy** - WASI preview 2 implementation approach
- **ADR-004: Performance Optimization Strategy** - JIT vs AOT vs interpreter selection

### Advanced Feature Decisions (Future)
- **ADR-017: Component Composition Model** - Component composition and linking strategy
- **ADR-019: Distributed Components** - Cross-system component communication
- **ADR-020: Development Tooling** - Development and debugging tool architecture

## Decision Priority (Future)

### Critical Path (Required Before Implementation)
1. ~~**ADR-001**: WASM Runtime Selection~~ - ✅ Completed as ADR-002 (2025-10-19)
2. ~~**ADR-005**: Capability-Based Security Model~~ - ✅ Completed (2025-10-19)
3. ~~**ADR-002**: Component Model Implementation~~ - ✅ Completed (2025-10-19)
4. ~~**ADR-006**: Sandbox Architecture~~ - ✅ Completed (2025-10-19)

### Implementation Phase
1. **ADR-003**: WASI Implementation Strategy
2. ~~**ADR-009**: Component Communication Model~~ - ✅ Completed (2025-10-19)
3. **ADR-013**: airssys-osl Integration
4. **ADR-014**: airssys-rt Integration

### Integration Phase
1. **ADR-007**: Security Policy System
2. **ADR-011**: Resource Management Strategy
3. **ADR-015**: Host Function Design
4. **ADR-016**: Performance Integration

## Decision Dependencies

### External Dependencies
- **AirsSys Architecture**: Depends on airssys-osl and airssys-rt architectural decisions
- **WASM Ecosystem**: Depends on WebAssembly Component Model specification stability
- **Security Framework**: Depends on AirsSys security framework maturity
- **Performance Requirements**: Depends on overall AirsSys performance targets

### Internal Dependencies
- **ADR-001** (Runtime Selection) → Blocks implementation ADRs
- **ADR-005** (Security Model) → Blocks security-related ADRs  
- **ADR-002** (Component Model) → Blocks component system ADRs
- **ADR-013/014** (Integration) → Depends on AirsSys component maturity

---

**Note:** ADRs and Knowledge documents are maintained during active development. New ADRs created as architectural decisions are made during implementation phases.

**Last Updated:** 2025-12-21  
**Total Decision Records:** 15 ADRs + 18 Knowledge Documents

## ADR-WASM-019: Runtime Dependency Management

**Status:** Accepted  
**Date:** 2025-12-16  
**Context:** Phase 5 Task 5.1 (Correlation Tracking) implementation review

**Decision:** Adopt multi-layer runtime dependency strategy:
- **Layer 0 (Tokio):** Direct usage for async primitives (channels, timers, tasks)
- **Layer 3 (airssys-rt):** Indirect usage for actor infrastructure (message routing, supervision)
- **Layer 2 (airssys-wasm):** WASM-specific features implementation

**Rationale:**
- Performance: Zero abstraction overhead with direct Tokio usage
- Reusability: Keeps airssys-rt generic (WASM-agnostic)
- Maintainability: Clear layer boundaries
- Flexibility: Full control over async behavior
- Standards: Industry-standard patterns

**Consequences:**
- ✅ Optimal performance (<50ns lookup, <5ms timeout)
- ✅ Clean separation of concerns
- ✅ airssys-rt remains reusable
- ⚠️ Developers need to understand layer boundaries
- ⚠️ Direct Tokio dependency (but already required)

**Implementation:**
- Use Tokio directly for: channels, timers, task spawning, synchronization
- Use airssys-rt indirectly for: message routing, actor lifecycle, supervision
- Implement in Layer 2: WASM-specific features (correlation tracking, permissions)

**Related ADRs:**
- ADR-WASM-018 (Three-Layer Architecture)
- ADR-WASM-009 (Component Communication Model)

**File:** `docs/adr/adr-wasm-019-runtime-dependency-management.md`

## ADR-WASM-020: Message Delivery Ownership Architecture

**Status:** Accepted  
**Date:** 2025-12-21  
**Category:** Communication Architecture / Message Routing

**Decision:** `ActorSystemSubscriber` owns message delivery. `ComponentRegistry` stays pure (identity lookup only).

**Context:** WASM-TASK-006 Task 1.1 revealed that `route_message_to_subscribers()` was stubbed because `ActorAddress` is an identifier, not a sender. The architectural question was where to store `MailboxSender` for actual message delivery.

**Options Considered:**
1. **Extend ComponentRegistry** (REJECTED) - Violates single responsibility, mixes concerns
2. **Create MailboxRegistry** (CONSIDERED) - Adds unnecessary complexity
3. **ActorSystemSubscriber owns mailbox_senders** (ACCEPTED) - Best alignment with ADR-WASM-009/018

**Key Design:**
```
ComponentRegistry (UNCHANGED)
    └── ComponentId → ActorAddress (identity only)

ActorSystemSubscriber (ENHANCED)
    └── mailbox_senders: HashMap<ComponentId, MailboxSender>
    └── register_mailbox() / unregister_mailbox()
    └── route_message_to_subscribers() → uses mailbox_senders for delivery
```

**Rationale:**
- Single Responsibility: Registry = identity, Subscriber = delivery
- ADR-WASM-009 Alignment: Subscriber handles routing AND delivery
- ADR-WASM-018 Compliance: Clear layer boundaries maintained
- No changes to existing ComponentRegistry API

**Related:**
- KNOWLEDGE-WASM-026 (Detailed implementation reference)
- ADR-WASM-009 (Component Communication Model)
- ADR-WASM-018 (Three-Layer Architecture)
- WASM-TASK-006 (Block 5 implementation)

**File:** `adr-wasm-020-message-delivery-ownership.md`


---

## ADR-WASM-021: Duplicate WASM Runtime Remediation 🔴

**Status:** Accepted  
**Date:** 2025-12-21  
**Category:** Architecture Remediation / Critical Fix  
**Severity:** 🔴 **CRITICAL**

**Summary:** Remediation plan for a fatal architectural violation where a duplicate WASM runtime was created in `actor/component/` using the wrong API (core WASM) instead of the correct API (Component Model). This ADR mandates deleting the duplicate runtime and refactoring ComponentActor to use the existing `runtime/WasmEngine`.

**Key Decisions:**
1. **DELETE** duplicate runtime code (WasmRuntime, WasmExports, WasmBumpAllocator, HandleMessageParams, HandleMessageResult)
2. **INJECT** `Arc<WasmEngine>` into ComponentActor
3. **FIX** Child::start() to use WasmEngine::load_component()
4. **FIX** Actor::handle() to use Component Model typed calls
5. **FIX** circular dependency by moving ComponentMessage to core/

**Impact:**
- Enables WIT interfaces (currently non-functional)
- Activates 154KB of generated bindings (currently unused)
- Removes 250+ lines of workaround code
- Restores correct layer architecture

**Timeline:** 4.5 days
- Phase 1: Fix circular dependency (0.5 days)
- Phase 2: Refactor ComponentActor (2 days)
- Phase 3: Update tests (2 days)

**Related:**
- KNOWLEDGE-WASM-027 (detailed violation documentation)
- ADR-WASM-002 (mandates Component Model)
- ADR-WASM-018 (three-layer architecture)

**File:** `adr-wasm-021-duplicate-runtime-remediation.md`

**Priority:** 🔴 CRITICAL - Must be resolved before Block 5 can proceed

## ADR-WASM-022: Circular Dependency Remediation (actor/ ↔ runtime/) 🔴

**Status:** Accepted  
**Date:** 2025-12-21  
**Category:** Architecture Remediation / Module Dependencies  
**Severity:** 🔴 **CRITICAL**

**Summary:** Remediation plan for circular dependency between `actor/` and `runtime/` modules. The correct architecture is one-way: `actor/ → runtime/ → core/`. Currently `runtime/` incorrectly imports from `actor/`.

**Key Decisions:**
1. **MOVE** `ComponentMessage` from `actor/` to `core/` (it's a data type)
2. **RELOCATE** `messaging_subscription.rs` from `runtime/` to `actor/component/` (it's integration logic)
3. **ADD** CI check to enforce layer boundaries and prevent future violations

**Evidence of Violation:**
- `runtime/async_host.rs:52` → `use crate::actor::ComponentMessage`
- `runtime/messaging.rs:76` → `use crate::actor::ComponentMessage`
- `runtime/messaging_subscription.rs:108-109` → `use crate::actor::component::{...}`

**Timeline:** 2.5-4.5 hours
- Phase 1: Move ComponentMessage to core/ (1-2 hours)
- Phase 2: Relocate messaging_subscription.rs (1-2 hours)
- Phase 3: Add CI enforcement (30 min)

**Dependencies:** This is a PREREQUISITE for ADR-WASM-021 (duplicate runtime fix)

**Related:**
- KNOWLEDGE-WASM-028 (detailed documentation)
- ADR-WASM-018 (three-layer architecture)
- ADR-WASM-021 (duplicate runtime - depends on this)

**File:** `adr-wasm-022-circular-dependency-remediation.md`

**Priority:** 🔴 CRITICAL - Must be done FIRST before ADR-WASM-021

## ADR-WASM-023: Module Boundary Enforcement 🔴 MANDATORY

**Status:** Accepted  
**Date:** 2025-12-22  
**Category:** Architecture / Module Design / MANDATORY  
**Severity:** 🔴 **CRITICAL - HARD REQUIREMENT**

**Summary:** Defines MANDATORY module boundaries for airssys-wasm. This ADR exists because repeated architectural violations have caused significant development delays.

**The Four Modules:**
- `core/` - Shared types and abstractions (imports: NOTHING)
- `security/` - Security logic (imports: core/ only)
- `runtime/` - WASM execution (imports: core/, security/ only)
- `actor/` - Actor integration (imports: core/, security/, runtime/)

**Key Rules:**
- ❌ `runtime/` → `actor/` is FORBIDDEN
- ❌ `security/` → `runtime/` or `actor/` is FORBIDDEN
- ❌ `core/` → anything is FORBIDDEN

**Enforcement:** Pre-commit verification with grep checks. Code violating these rules will be REJECTED.

**Related:**
- KNOWLEDGE-WASM-030 (detailed reference)
- ADR-WASM-018 (three-layer architecture)
- ADR-WASM-022 (circular dependency remediation)

**File:** `adr-wasm-023-module-boundary-enforcement.md`

**Priority:** 🔴 MANDATORY - All code must comply


### ADR-WASM-024: Refactor Messaging from Runtime to Top-Level Module
- **Status:** Accepted
- **Date:** 2025-12-26
- **Category:** Module Architecture / Remediation
- **Summary:** Move messaging infrastructure from `src/runtime/messaging.rs` to a new top-level `src/messaging/` module to fix module architecture violation and eliminate circular dependency risk.
- **Related:** KNOWLEDGE-WASM-012 (Module Structure), KNOWLEDGE-WASM-034 (Architecture Violation Document), ADR-WASM-018 (Three-Layer Architecture), ADR-WASM-023 (Module Boundary Enforcement)
- **Impact:** Critical - Fixes architectural violation and enables proper Block 5 development
- **Implementation:** WASM-TASK-HOTFIX-001
- **File:** `adr-wasm-024-refactor-messaging-from-runtime-to-top-level-module.md`

