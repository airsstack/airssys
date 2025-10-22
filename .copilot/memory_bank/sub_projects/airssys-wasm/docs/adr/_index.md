# airssys-wasm Architecture Decision Records Index

**Sub-Project:** airssys-wasm  
**Last Updated:** 2025-10-22  
**Total ADRs:** 12  
**Active ADRs:** 12  

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

### ADR-WASM-005: Capability-Based Security Model
- **Status:** Accepted
- **Date:** 2025-10-19
- **Category:** Security Architecture
- **Summary:** Fine-grained capability-based security with pattern matching for resources (filesystem globs, network domains, storage namespaces), trust-level system for auto-approval workflows (trusted sources instant install, unknown sources require review), layered integration with airssys-osl RBAC/ACL for defense-in-depth.
- **Related:** KNOWLEDGE-WASM-001 (Component Framework), KNOWLEDGE-WASM-004 (WIT Management), ADR-WASM-002 (Runtime Engine)
- **Impact:** Critical - Security foundation for all component operations
- **Key Decisions:**
  - Granularity: Fine-grained pattern-based (filesystem globs, network domains, storage namespaces)
  - Declaration: Component.toml manifest (visible before installation, language-agnostic)
  - Workflow: Trust levels with auto-approval (trusted instant, unknown review, dev mode bypass)
  - Enforcement: Host function entry checks with pattern matching (~1-5μs overhead, 0.1% of operation)
  - Integration: Layered security (component capabilities + airssys-osl RBAC/ACL + OS permissions)
  - Custom Capabilities: Deferred to Phase 2+ (YAGNI, focus on core foundation)
- **File:** `adr_wasm_005_capability_based_security_model.md`

### ADR-WASM-003: Component Lifecycle Management
- **Status:** Accepted
- **Date:** 2025-10-19
- **Category:** Component System Architecture
- **Summary:** Immutable components with automatic retention policies (blockchain-inspired pattern). Three installation sources (Git reproducible, Local fast dev, Remote URL pre-built), 2-state lifecycle (Installed/Uninstalled), routing proxy for blue-green deployment, cryptographic Ed25519 ownership, actor-based isolation, configurable retention (default 24h rollback window with auto-cleanup).
- **Related:** KNOWLEDGE-WASM-001 (Component Framework), KNOWLEDGE-WASM-009 (Installation), ADR-WASM-002 (Runtime), ADR-WASM-005 (Security), ADR-WASM-007 (Storage)
- **Impact:** Critical - Foundation for component management and updates
- **Key Decisions:**
  - Installation: Git (libgit2 platform-agnostic), Local (fast iteration), Remote URL (offline/CDN)
  - Lifecycle: 2-state machine (Installed → Uninstalled), immutable like smart contracts
  - Updates: Blue-green deployment via routing proxy (zero-downtime, <1ms route switch)
  - Retention: Configurable policies (KeepOldVersion 24h default, DestroyImmediately opt-in, KeepLastN audit)
  - Ownership: Ed25519 signatures (only private key holder can update/uninstall)
  - Isolation: Actor-based (ComponentProxyActor routes, ComponentActor executes, supervisor handles crashes)
  - Pattern: Inspired by Ethereum proxy contracts, Solana program upgrades (immutability + routing)
- **File:** `adr_wasm_003_component_lifecycle_management.md`

### ADR-WASM-006: Component Isolation and Sandboxing (Revised)
- **Status:** Accepted (Revised 2025-10-19)
- **Date:** 2025-10-19 (Original), 2025-10-19 (Revised)
- **Category:** Security & Isolation Architecture
- **Summary:** Erlang-style lightweight process isolation using airssys-rt actors, combined with WASM linear memory sandboxing and hybrid resource enforcement. 4-layer defense in depth (Capability → WASM → Actor → Supervision), ComponentActor dual trait design with WASM lifecycle in Child::start()/stop(), inter-component messaging via MessageBroker (see ADR-WASM-009).
- **Related:** KNOWLEDGE-WASM-001 (Component Framework), KNOWLEDGE-RT-013 (Actor Performance), ADR-WASM-002 (Runtime), ADR-WASM-003 (Lifecycle), ADR-WASM-005 (Security), ADR-WASM-007 (Storage), ADR-WASM-009 (Communication), ADR-RT-004 (Actor/Child Separation)
- **Impact:** Critical - Security foundation for untrusted third-party components
- **Key Decisions:**
  - Isolation Strategy: Erlang-style lightweight processes (airssys-rt actors, ~625ns spawn, 10,000+ concurrent)
  - Layer 1: Capability-based security (permission checks at host functions)
  - Layer 2: WASM linear memory sandbox (512KB-4MB isolated heap, bounds checking)
  - Layer 3: Actor isolation (private mailbox, message passing only, ~211ns routing)
  - Layer 4: Supervision trees (automatic restart, health monitoring, graceful shutdown)
  - ComponentActor: Dual trait (Actor for message handling, Child for WASM lifecycle)
  - WASM Lifecycle: Child::start() loads WASM, Child::stop() cleans up (supervisor-controlled)
  - Resource Limits: Hybrid enforcement (Wasmtime: memory/CPU + App-level: storage/network/API quotas)
  - Inter-Component Communication: MessageBroker routing with host function security (see ADR-WASM-009)
  - Performance: ~2-11ms spawn, ~1-2MB per component, <5% resource check overhead
  - Platform: Cross-platform (Tokio + WASM on Linux, macOS, Windows)
- **Revision Note:** Original proposal used OS process isolation. Revised to use airssys-rt lightweight actors after architectural review. Provides 100,000x faster spawn (625ns vs 50-100ms), 10x less memory (<1KB vs 5-10MB), and cross-platform compatibility.
- **File:** `adr_wasm_006_component_isolation_and_sandboxing.md`

### ADR-WASM-007: Storage Backend Selection
- **Status:** Accepted
- **Date:** 2025-10-19
- **Category:** Storage Architecture
- **Summary:** NEAR-style key-value storage API with Sled as default backend (pure Rust benefits) and RocksDB as optional fallback (proven stability), prefix-based component isolation, application-level quota tracking, NO transactions (actor model sequential processing eliminates need), export/import JSON Lines tool for migration and backups.
- **Related:** KNOWLEDGE-WASM-007 (Component Storage), KNOWLEDGE-WASM-008 (Backend Comparison), ADR-WASM-005 (Security)
- **Impact:** Critical - Persistent storage foundation for all components
- **Key Decisions:**
  - API: NEAR-style KV (simple get/set/delete/scan_prefix, language-agnostic)
  - Backend: Sled default (pure Rust, fast builds), RocksDB optional (proven production)
  - Isolation: Prefix-based namespacing (`component:<id>:` prefix for all keys)
  - Quota: Application-level tracking (real-time enforcement, periodic reconciliation)
  - Transactions: NOT required (actor model sequential processing ensures consistency)
  - Migration: Export/import JSON Lines tool (backend migration and backups)
- **File:** `adr_wasm_007_storage_backend_selection.md`

### ADR-WASM-009: Component Communication Model
- **Status:** Accepted
- **Date:** 2025-10-19
- **Category:** Communication Architecture
- **Summary:** Message-passing via airssys-rt MessageBroker with host-mediated security enforcement. Supports fire-and-forget (one-way), request-response (async RPC with callbacks), and pub-sub patterns. Pure pub-sub architecture with ActorSystem as primary subscriber, security enforcement at host function layer (capability validation + quota enforcement + audit logging), multicodec self-describing serialization for cross-language communication.
- **Related:** KNOWLEDGE-WASM-005 (Messaging Implementation), ADR-WASM-001 (Multicodec), ADR-WASM-005 (Security), ADR-WASM-006 (Isolation), RT-TASK-008 (MessageBroker Performance)
- **Impact:** Critical - Foundation for all inter-component communication
- **Key Decisions:**
  - Architecture: airssys-rt InMemoryMessageBroker (pure pub-sub, ~211ns routing, 4.7M msg/sec)
  - Patterns: Fire-and-forget (~280ns), Request-response with callbacks (~560ns round-trip), Manual correlation (advanced)
  - Security: Host function layer (capability checks ~50ns, quota enforcement, audit logging)
  - Delivery: Push-based (no polling), messages delivered via handle-message export
  - Serialization: Multicodec self-describing (borsh for inter-component, CBOR/JSON for external)
  - Performance: ~260ns total overhead (50ns validation + 211ns routing), >3M msg/sec throughput
  - Integration: ActorSystem subscribes to broker, routes to ComponentActor mailboxes
  - Timeout: Host runtime enforces request timeouts, automatic callback cleanup
- **File:** `adr_wasm_009_component_communication_model.md`

### ADR-WASM-010: Implementation Strategy and Build Order
- **Status:** Accepted
- **Date:** 2025-10-20
- **Category:** Implementation Planning & Architecture
- **Summary:** 4-layer phased implementation strategy with Actor System Integration as foundational component (Layer 1, Block 3), NOT an integration layer component. Defines 11 major building blocks with correct dependency order, preventing circular dependencies and architectural rework. Critical correction: airssys-rt integration must come early as foundation (Block 3) because Inter-Component Communication (Block 5) and Component Lifecycle (Block 7) depend on MessageBroker and SupervisorNode.
- **Related:** All ADRs (implementation guidance), KNOWLEDGE-WASM-001 (Component Framework), KNOWLEDGE-WASM-005 (Messaging), ADR-WASM-006 (Actor Isolation), ADR-WASM-009 (MessageBroker dependency)
- **Impact:** Critical - Defines entire development roadmap and prevents build order mistakes
- **Key Decisions:**
  - Build Order: Foundation (Blocks 1-3) → Core Services (4-7) → Integration (8-9) → Developer Experience (10-11)
  - Block 3 Position: Actor System Integration is FOUNDATIONAL (Block 3), not integration layer (was incorrectly Block 9)
  - Rationale: MessageBroker, SupervisorNode, and ComponentActor are core infrastructure, not integrations
  - Timeline: 11-15 months total (4 months foundation, 5 months core services, 3 months integration, 3 months dev tools)
  - Parallelization: Blocks within same layer can develop in parallel (e.g., Blocks 1-2, Blocks 4-6, Blocks 8-9, Blocks 10-11)
  - Layer Gates: Each layer must validate before next layer begins
  - Performance Targets: Tracked from Block 3 onwards (actor spawn ~625ns, message routing ~211ns)
- **Mental Model:** "Actor-hosted WASM components from the start" (NOT "WASM components, then integrate actors later")
- **File:** `adr_wasm_010_implementation_strategy_and_build_order.md`

### ADR-WASM-011: Module Structure Organization
- **Status:** Accepted
- **Date:** 2025-10-21
- **Category:** Code Architecture & Organization
- **Summary:** Hybrid block-aligned module structure combining flat domain-driven organization (airssys-rt pattern) with core abstraction layer (airssys-osl pattern). Direct 1:1 mapping to 11 implementation blocks, core/ module prevents circular dependencies, prelude re-exports for ergonomic API, mod.rs declaration-only following workspace §4.3.
- **Related:** ADR-WASM-010 (Implementation Strategy), KNOWLEDGE-WASM-012 (Module Structure Architecture), Workspace §4.3 (Module Architecture), §2.1 (Imports), §6.1 (YAGNI)
- **Impact:** Critical - Defines entire crate organization and code structure for all implementations
- **Key Decisions:**
  - Organization: Hybrid block-aligned with core (Option 3 from knowledge doc)
  - Structure: core/ (foundation, zero internal deps) + 11 flat domain modules (runtime/, wit/, actor/, security/, messaging/, storage/, lifecycle/, component/, osl/, monitoring/, installation/) + util/ + prelude/
  - Dependencies: Acyclic graph enforced (core → runtime/wit → actor/osl → services → management)
  - Public API: Prelude re-exports for ergonomic imports (airssys_wasm::prelude::*)
  - Testing: Unit tests co-located, integration tests in tests/ mirror module structure
  - Standards: mod.rs declaration-only (§4.3), 3-layer imports (§2.1), YAGNI-compliant (§6.1)
  - Task Alignment: Direct mapping to WASM-TASK-002 through 012 (runtime/ = Block 1, wit/ = Block 2, etc.)
- **Rationale:** Combines proven patterns from airssys-rt (flat, intuitive) and airssys-osl (core abstractions), prevents circular dependencies, easy contributor navigation, clear task-to-module mapping
- **File:** `adr_wasm_011_module_structure_organization.md`

### ADR-WASM-012: Comprehensive Core Abstractions Strategy
- **Status:** Accepted
- **Date:** 2025-10-21
- **Category:** Architecture & Type System Design
- **Summary:** Comprehensive core abstractions covering ALL implementation blocks (1-11). Core module contains universal types (component, capability, error, config) PLUS domain-specific abstractions for each block (runtime, interface, actor, security, messaging, storage, lifecycle, management, bridge, observability). Trait-centric design with zero internal dependencies.
- **Related:** ADR-WASM-011 (Module Structure), KNOWLEDGE-WASM-012 (Module Architecture), WASM-TASK-000 (Core Design)
- **Impact:** CRITICAL - Prevents circular dependencies, enables parallel development, ensures API stability
- **Key Decisions:**
  - Scope: 14 core files (4 universal + 10 domain-specific)
  - Pattern: Trait-centric design for extensibility
  - Dependencies: Zero internal dependencies (core depends on external crates only)
  - Timeline: WASM-TASK-000 expanded from 1-2 weeks to 3-4 weeks
  - Development: Abstractions-first (all core complete before block implementation)
  - Validation: Architectural review, prototype validation, ADR compliance check
- **Rationale:** Prevents circular dependencies, enables parallel block development, ensures API stability, facilitates testability via traits, provides refactoring safety
- **File:** `adr_wasm_012_comprehensive_core_abstractions_strategy.md`

### ADR-WASM-013: StorageTransaction Removal (YAGNI Simplification)
- **Status:** Accepted
- **Date:** 2025-10-22
- **Category:** Storage Architecture Simplification
- **Summary:** Remove unused StorageTransaction trait from storage.rs per YAGNI principle. Actor model sequential processing eliminates need for transactions. Aligns with ADR-WASM-007 Decision 5 rationale. Simplifies API surface, reduces maintenance burden, eliminates Box<dyn> pattern complexity.
- **Related:** ADR-WASM-007 (Storage Backend Selection, Decision 5), ADR-WASM-006 (Actor Isolation), KNOWLEDGE-WASM-007 (Component Storage)
- **Impact:** Low - Removes unused abstraction with zero usage across codebase
- **Key Decisions:**
  - Remove StorageTransaction trait (3 methods: add_operation, commit, rollback)
  - Remove begin_transaction() method from StorageBackend trait
  - Remove MockTransaction test implementation and 3 related tests
  - Total removal: ~165 lines of code
  - Actor model sequential processing provides sufficient consistency guarantees
  - YAGNI compliance: No current need, actor model eliminates transaction requirement
- **Rationale:** Actor model sequential message processing ensures operation atomicity without explicit transactions. Per ADR-WASM-007 Decision 5, transactions are unnecessary complexity. Reduces API surface and eliminates Box<dyn> anti-pattern (§6.2).
- **File:** `adr_wasm_013_storage_transaction_removal.md`

### ADR-WASM-014: RoutingStrategy Removal (YAGNI Simplification)
- **Status:** Accepted
- **Date:** 2025-10-22
- **Category:** Messaging Architecture Simplification
- **Summary:** Remove unused RoutingStrategy trait from messaging.rs per YAGNI principle. Routing handled exclusively by airssys-rt MessageBroker per ADR-WASM-009 architecture. No pluggable routing strategies in design. Simplifies API surface, removes fictional documentation, eliminates Box<dyn> pattern complexity.
- **Related:** ADR-WASM-009 (Component Communication Model), ADR-WASM-006 (Actor Isolation), ADR-WASM-005 (Security Model), KNOWLEDGE-WASM-005 (Messaging Architecture)
- **Impact:** Low - Removes unused abstraction with zero usage across codebase
- **Key Decisions:**
  - Remove RoutingStrategy trait (1 method: route)
  - Remove fictional implementations from documentation (DirectRoutingStrategy, TopicRoutingStrategy, BroadcastRoutingStrategy, CustomRoutingStrategy)
  - Remove test_routing_strategy_trait_object test
  - Total removal: ~115 lines of code
  - MessageBroker handles routing (211ns per message, 4.7M msg/sec)
  - Host-mediated security prevents custom routing strategies
  - YAGNI compliance: Speculative abstraction with no identified use case
- **Rationale:** ADR-WASM-009 specifies fixed MessageBroker architecture with pub-sub routing. Host-mediated security model prevents components from implementing custom routing strategies. Similar anti-pattern to StorageTransaction (ADR-WASM-013).
- **File:** `adr_wasm_014_routing_strategy_removal.md`

---

## Planned ADR Categories (Future)

### WASM Runtime Decisions
- **ADR-001: WASM Runtime Selection** - wasmtime vs wasmer vs custom runtime
- **ADR-002: Component Model Implementation** - WebAssembly Component Model approach
- **ADR-003: WASI Implementation Strategy** - WASI preview 2 implementation approach
- **ADR-004: Performance Optimization Strategy** - JIT vs AOT vs interpreter selection

### Security Architecture Decisions
- ~~**ADR-005: Capability-Based Security Model**~~ - ✅ Completed (2025-10-19)
- ~~**ADR-006: Sandbox Architecture**~~ - ✅ Completed as Component Isolation (2025-10-19)
- ~~**ADR-007: Security Policy System**~~ - ✅ Repurposed as Storage Backend Selection
- **ADR-008: Audit and Logging Strategy** - Security audit logging and compliance

### Component System Decisions
- ~~**ADR-009: Component Communication Model**~~ - ✅ Completed (2025-10-19)
- **ADR-010: Component Registry Design** - Component discovery and management
- **ADR-011: Resource Management Strategy** - Component resource limits and monitoring
- ~~**ADR-012: Component Lifecycle Management**~~ - ✅ Completed as ADR-003 (2025-10-19)

### Integration Decisions
- **ADR-013: airssys-osl Integration** - OS layer integration patterns and interfaces
- **ADR-014: airssys-rt Integration** - Actor system integration and component hosting
- **ADR-015: Host Function Design** - Custom host function architecture
- **ADR-016: Performance Integration** - Performance optimization for integrated systems

### Advanced Feature Decisions (Future)
- **ADR-017: Component Composition Model** - Component composition and linking strategy
- **ADR-018: Runtime Component Reload Architecture** - Dynamic component update mechanism
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

## Decision Dependencies (Future)

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

## Quality Assurance (Future)

### Review Requirements
- **Security Review**: All security-related ADRs require comprehensive security review
- **Performance Review**: Performance-critical ADRs require performance analysis
- **Integration Review**: Integration ADRs require review by affected AirsSys teams
- **Architecture Review**: High-impact ADRs require architectural review

### Implementation Validation
- **Prototype Validation**: Critical decisions validated through prototyping
- **Performance Benchmarking**: Performance decisions validated through benchmarks
- **Security Testing**: Security decisions validated through security testing
- **Integration Testing**: Integration decisions validated through integration testing

---
**Note:** ADRs will be created when project implementation begins (estimated Q3 2026).