# airssys-wasm Knowledge Documentation Index

**Sub-Project:** airssys-wasm  
**Last Updated:** 2026-01-08  
**Total Knowledge Docs:** 26  
**Active Knowledge Docs:** 25  

## Current Knowledge Documentation
- **[KNOWLEDGE-WASM-031: Foundational Architecture](knowledge-wasm-031-foundational-architecture.md)** 🔴 **READ FIRST**
  - **Purpose**: The MOST fundamental understanding of what airssys-wasm is
  - **Scope**: Two root entities (Host + Plugin/Component), Actor model, isolated storage, mailbox communication
  - **Key Content**: Each WASM component = Actor (via airssys-rt), isolated storage per component, private state, Erlang-style mailbox communication
  - **Status**: Active (Created 2025-12-22)
  - **Impact**: 🔴 CRITICAL - Read this BEFORE any other document
  - **Audience**: EVERYONE - developers, architects, planners, implementers
  - **Inspiration**: Smart contract platforms (NEAR, Polkadot) using WASM

- **[KNOWLEDGE-WASM-037: Rebuild Architecture - Clean Slate Design](knowledge-wasm-037-rebuild-architecture-clean-slate.md)** 🔴 **CRITICAL REBUILD**
  - **Purpose**: Clean-slate rebuild architecture for airssys-wasm eliminating previous circular dependencies and DIP violations
  - **Scope**: Complete module structure (core/, security/, runtime/, component/, messaging/, system/), Dependency Inversion patterns, airssys-rt/osl integration, WIT structure, testing strategy
  - **Key Content**: Layer-organized core/ with abstractions per module, one-way dependency flow, WASM-only runtime/, system/ as coordinator with DI, test fixture standards
  - **Status**: Active (Created 2026-01-05)
  - **Impact**: 🔴 CRITICAL - Foundation for complete rebuild from scratch
  - **Audience**: ALL developers working on rebuild - MANDATORY reading
  - **Supersedes**: KNOWLEDGE-WASM-035, KNOWLEDGE-WASM-036 (previous architectures with violations)

### Architecture & Design Category ✅
- **[KNOWLEDGE-WASM-039: Runtime Module Responsibility and Architecture](knowledge-wasm-039-runtime-module-responsibility.md)** 🟡 **IMPORTANT**
  - **Purpose**: Clarifies distinction between core/runtime/ (Layer 1) and runtime/ (Layer 3B) modules
  - **Scope**: Two-layer architecture, module ownership, dependency patterns, integration guidelines
  - **Key Content**: core/runtime/ foundation traits (RuntimeEngine, ComponentLoader, ResourceLimits), runtime/ WASM execution (WasmtimeEngine, ComponentLoader, HostFunctions), Dependency Inversion pattern, implementation guidelines for Phase 3 and Phase 5 tasks
  - **Status**: Active (Created 2026-01-08)
  - **Impact**: Important - Prevents confusion between core abstractions and execution logic, guides code placement
  - **Audience**: All developers working on runtime-related tasks (WASM-TASK-018, future runtime/ module tasks)
  - **Related**: KNOWLEDGE-WASM-037 (clean-slate architecture), ADR-WASM-025 (six-module design), ADR-WASM-028 (core module structure)

- **[KNOWLEDGE-WASM-038: Component Module Responsibility and Architecture](knowledge-wasm-038-component-module-responsibility.md)** 🟡 **IMPORTANT**
  - **Purpose**: Clarifies distinction between core/component/ (Layer 1) and component/ (Layer 3A) modules
  - **Scope**: Two-layer architecture, module ownership, dependency patterns, integration guidelines
  - **Key Content**: core/component/ foundation types (ComponentId, ComponentHandle, ComponentMessage), component/ actor integration (ComponentWrapper, ComponentRegistry, ComponentSpawner), Dependency Inversion pattern, implementation guidelines for Phase 3 and Phase 4 tasks
  - **Status**: Active (Created 2026-01-08)
  - **Impact**: Important - Prevents confusion between core types and integration logic, guides code placement
  - **Audience**: All developers working on component-related tasks (WASM-TASK-017, future component/ module tasks)
  - **Related**: KNOWLEDGE-WASM-037 (clean-slate architecture), ADR-WASM-025 (six-module design), ADR-WASM-028 (core module structure)

- **[KNOWLEDGE-WASM-040: Messaging Module - Comprehensive Reference](knowledge-wasm-040-messaging-module-comprehensive-reference.md)** 🔴 **CRITICAL**
  - **Purpose**: Definitive reference for the messaging module - MUST be referenced by all messaging-related tasks
  - **Scope**: Two-layer architecture (core/messaging/ + messaging/), messaging patterns, type definitions, WIT interfaces, dependency injection
  - **Key Content**: Fire-and-forget vs request-response patterns, CorrelationId/MessageRouter/CorrelationTracker types, complete 10-step message flow diagram, WIT interface definitions, verification checklist
  - **Status**: Active (Created 2026-01-09)
  - **Impact**: 🔴 CRITICAL - Definitive reference for all messaging development
  - **Audience**: All developers working on messaging-related tasks (WASM-TASK-019, future messaging/ module tasks)
  - **Related**: KNOWLEDGE-WASM-005, KNOWLEDGE-WASM-024, KNOWLEDGE-WASM-029, ADR-WASM-009, ADR-WASM-031

- **[KNOWLEDGE-WASM-041: Storage Management Architecture (Clean-Slate Rebuild)](knowledge-wasm-041-storage-management-architecture-clean-slate.md)** 🟡 **IMPORTANT**
  - **Purpose**: Defines storage management architecture for clean-slate rebuild with namespace isolation
  - **Scope**: Storage call flow, namespace isolation (Solana-inspired), type system design, layered composition, caching strategy
  - **Key Content**: Complete call flow from component to backend, host-enforced implicit namespacing, `StorageValue` ADT, `ComponentStorage` trait, `NamespacedStorage` wrapper, `CachedStorage` decorator, type locations
  - **Status**: Active (Created 2026-01-10)
  - **Impact**: Important - Foundation for WASM-TASK-021 and all storage-related development
  - **Audience**: All developers working on storage-related tasks (WASM-TASK-021, future storage backends)
  - **Related**: KNOWLEDGE-WASM-007 (foundational concepts), KNOWLEDGE-WASM-008 (backend comparison), ADR-WASM-028 (core module structure)

### Core Concepts Category ✅
- **[KNOWLEDGE-WASM-002: High-Level Overview](knowledge_wasm_002_high_level_overview.md)** ✅ **ESSENTIAL**
  - **Purpose**: Authoritative high-level overview and conceptual foundation
  - **Scope**: Core concepts, strategic positioning, problem statement, design philosophy
  - **Key Content**: What airssys-wasm is, problems it solves, key characteristics, target use cases
  - **Status**: Complete definitive overview (Created 2025-10-17)
  - **Impact**: Essential - primary reference for understanding the project
  - **Audience**: Anyone seeking to understand airssys-wasm at conceptual level

### Architecture & Design Category ✅
- **[KNOWLEDGE-WASM-015: Project Structure and Workspace Architecture](knowledge_wasm_015_project_structure_and_workspace_architecture.md)** ✅ **CRITICAL**
  - **Purpose**: Comprehensive documentation of three sub-projects and their relationships
  - **Scope**: Workspace structure, crate relationships, task-to-crate mapping, integration architecture
  - **Key Content**: airssys-wasm (core), airssys-wasm-component (macros), airssys-wasm-cli (CLI), dependency graph, implementation timeline, developer guidance
  - **Status**: Complete authoritative reference (Created 2025-11-30)
  - **Impact**: Critical - prevents confusion between sub-projects, essential for task implementation
  - **Audience**: All developers, architects, project managers, anyone working on WASM-TASK-011 or WASM-TASK-012
  - **Related**: All WASM tasks, KNOWLEDGE-WASM-010 (CLI spec), KNOWLEDGE-WASM-012 (SDK patterns)

- **[KNOWLEDGE-WASM-001: Component Framework Architecture](knowledge_wasm_001_component_framework_architecture.md)** ✅ **CRITICAL**
  - **Purpose**: Foundational architecture for WASM Component Framework for Pluggable Systems
  - **Scope**: Complete architectural decisions and design principles
  - **Key Content**: 4-layer architecture, component model, runtime deployment, integration patterns
  - **Status**: Complete foundational design (Updated 2025-10-17 - terminology standardized)
  - **Impact**: Critical - defines entire framework foundation
  - **Audience**: Architects and senior developers implementing framework

- **[KNOWLEDGE-WASM-003: Core Architecture Design](knowledge_wasm_003_core_architecture_design.md)** ✅ **CRITICAL**
  - **Purpose**: Two-audience developer experience and architectural layers
  - **Scope**: Plugin developers vs host developers, AirsSys integration patterns
  - **Key Content**: Component interface, deployment patterns, security model, implementation phases
  - **Status**: Complete foundational design (Updated 2025-10-17 - renamed to standard format)
  - **Impact**: Critical - defines framework approach and developer experience
  - **Audience**: Framework designers and integration engineers

- **[KNOWLEDGE-WASM-012: Module Structure Architecture](knowledge_wasm_012_module_structure_architecture.md)** ✅ **CRITICAL**
  - **Purpose**: Complete module structure design for airssys-wasm crate organization
  - **Scope**: Module organization, dependency rules, public API surface, testing structure
  - **Key Content**: 3 organizational approaches evaluated, hybrid block-aligned recommendation, dependency graph, prelude pattern, module responsibility matrix, implementation guidelines
  - **Status**: Complete module structure design (Created 2025-10-21)
  - **Impact**: Critical - defines entire crate organization and code structure
  - **Audience**: All implementers, architects, contributors
  - **Key Decision**: Approach 3 (Hybrid Block-Aligned with Core) - combines airssys-rt flat structure with airssys-osl core abstractions
  - **Related**: All implementation tasks (WASM-TASK-002 through 012), workspace §4.3 module standards

- **[KNOWLEDGE-WASM-004: WIT Management Architecture](knowledge_wasm_004_wit_management_architecture.md)** ✅ **CRITICAL**
  - **Purpose**: Complete WIT interface management and component development framework
  - **Scope**: WIT interface design, multicodec integration, development workflows, messaging overview
  - **Key Content**: Component interfaces, host services, permission-based security, messaging model
  - **Status**: Complete implementation plan (Updated 2025-10-18 - actor-based messaging)
  - **Impact**: Critical - defines component development and interface management
  - **Audience**: Component developers and interface designers

- **[KNOWLEDGE-WASM-005: Inter-Component Messaging Architecture](knowledge_wasm_005_messaging_architecture.md)** ✅ **CRITICAL**
  - **Purpose**: Comprehensive messaging system architecture for component communication
  - **Scope**: Actor-based message passing, dual interaction patterns, airssys-rt integration, sequence diagrams
  - **Key Content**: Fire-and-forget messaging, request-response with callbacks, multicodec message encoding, host runtime implementation, performance optimization, error handling
  - **Status**: Complete messaging architecture with sequence diagrams (Updated 2025-10-18)
  - **Impact**: Critical - defines inter-component communication patterns and implementation
  - **Audience**: Component developers, runtime implementers, and integration engineers

- **[KNOWLEDGE-WASM-006: Multiformat Strategy](knowledge_wasm_006_multiformat_strategy.md)** ✅ **CRITICAL**
  - **Purpose**: Complete multiformats integration strategy and self-describing data foundation
  - **Scope**: Multicodec specification, format selection, language-specific implementations, performance characteristics
  - **Key Content**: Multiformats overview, multicodec table, AirsSys codec reservations (borsh 0x701, bincode 0x702), encoding/decoding patterns, format evolution, component manifest integration
  - **Status**: Complete multiformat integration strategy (Created 2025-10-18)
  - **Impact**: Critical - defines language-agnostic data interchange and future-proof serialization
  - **Audience**: Component developers across all languages, format designers, and interoperability engineers
  - **Standards**: Based on official Protocol Labs multiformats specification (https://github.com/multiformats/multiformats)
  - **Related**: KNOWLEDGE-WASM-004 (WIT interface definitions)

- **[KNOWLEDGE-WASM-013: Core WIT Package Structure](knowledge_wasm_013_core_wit_package_structure.md)** ✅ **CRITICAL**
  - **Purpose**: Comprehensive explanation of airssys:core@1.0.0 WIT package structure and host-component contract
  - **Scope**: 4-layer interface architecture, bidirectional contract, type reuse patterns, implementation directions
  - **Key Content**: Layer 0 (types.wit - shared vocabulary), Layer 1 (capabilities.wit - security model), Layer 2 (component-lifecycle.wit - component contract), Layer 3 (host-services.wit - host services), bidirectional relationship diagram, Component Model v0.1 type reuse via use statements
  - **Status**: Complete architectural knowledge (Created 2025-11-24)
  - **Impact**: Critical - defines foundational contract between host and components
  - **Audience**: Component developers, host runtime implementers, framework architects
  - **Related**: KNOWLEDGE-WASM-004 (WIT management), WASM-TASK-003 (WIT implementation), DEBT-WASM-003 (Component Model v0.1 limitations)

- **[KNOWLEDGE-WASM-016: Actor System Integration Implementation Guide](knowledge_wasm_016_actor_system_integration_implementation_guide.md)** ✅ **CRITICAL**
   - **Purpose**: Detailed implementation guidance for WASM-TASK-004 (Block 3 - Actor System Integration)
   - **Scope**: Code-level examples for all 18 subtasks, concrete patterns, effort estimates, testing strategies
   - **Key Content**: ComponentActor struct design, Child trait WASM lifecycle, Actor trait message handling, ActorSystem spawning, component registry, supervisor tree, MessageBroker integration, performance targets, testing framework
   - **Status**: Complete implementation reference (Created 2025-11-30)
   - **Impact**: Critical - provides detailed guidance for developers implementing actor-based components
   - **Audience**: Developers implementing WASM-TASK-004, architects reviewing implementation
   - **Complements**: task_004_block_3_actor_system_integration.md (task definition)
   - **Related**: ADR-WASM-006 (actor isolation), ADR-WASM-009 (messaging), KNOWLEDGE-RT-013 (performance baselines)

- **[KNOWLEDGE-WASM-007: Component Storage Architecture](knowledge_wasm_007_component_storage_architecture.md)** ✅ **CRITICAL**
  - **Purpose**: Persistent storage architecture for component state management with trait-based backend abstraction
  - **Scope**: Storage API design, blockchain model comparison, backend abstraction layer, permission model, quota management
  - **Key Content**: NEAR-style KV storage API, Solana/NEAR/EVM model comparison, `StorageBackend` trait abstraction, pluggable backends (Sled default, RocksDB optional), component namespace isolation, permission-based access control, storage quotas and limits, implementation patterns by language
  - **Status**: Complete storage architecture with abstraction layer (Updated 2025-10-18)
  - **Impact**: Critical - defines component persistent storage with pluggable backend architecture
  - **Audience**: Component developers, runtime implementers, storage system designers, backend engineers
  - **Research**: Based on production blockchain storage models (Solana AccountsDB, NEAR RocksDB, Ethereum Patricia Trie)
  - **Related**: KNOWLEDGE-WASM-004 (permission model), KNOWLEDGE-WASM-005 (messaging), KNOWLEDGE-WASM-006 (serialization), KNOWLEDGE-WASM-008 (backend comparison)

- **[KNOWLEDGE-WASM-008: Storage Backend Comparison](knowledge_wasm_008_storage_backend_comparison.md)** ✅ **CRITICAL**
  - **Purpose**: Comprehensive analysis of storage backend options in Rust ecosystem for informed selection
  - **Scope**: Detailed comparison of sled vs RocksDB vs alternatives, compilation complexity, production stability, performance benchmarks
  - **Key Content**: Sled (pure Rust, recommended default) vs RocksDB (C++ bindings, production alternative) detailed analysis, compilation/build experience comparison, performance benchmarks, space efficiency, production readiness assessment, feature matrix, backend selection criteria, migration strategy between backends, redb and fjall alternatives evaluation
  - **Status**: Complete comprehensive backend comparison (Created 2025-10-18)
  - **Impact**: Critical - enables informed backend selection with clear tradeoffs documented
  - **Audience**: System architects, backend engineers, production deployment teams, decision makers
  - **Key Decision**: Sled as default (pure Rust, fast builds), RocksDB optional (proven stability, C++ complexity)
  - **Research**: Based on official documentation, production usage analysis, and compilation experience
  - **Related**: KNOWLEDGE-WASM-007 (storage architecture and abstraction layer)

### Installation & Deployment Category ✅
- **[KNOWLEDGE-WASM-009: Component Installation Architecture](knowledge_wasm_009_component_installation_architecture.md)** ✅ **CRITICAL**
  - **Purpose**: Complete specification of component installation, distribution, and cryptographic security
  - **Scope**: Git-based installation, TOML manifest format, Ed25519 digital signatures, multi-source deployment
  - **Key Content**: Installation philosophy, Component.toml specification, cryptographic ownership model, installation workflows (Git/file/URL), update/uninstall operations, security verification layers
  - **Status**: Complete architecture design (Created 2025-10-18)
  - **Impact**: Critical - defines how components are distributed, installed, and secured
  - **Audience**: Architects, security engineers, component developers, host application developers

### Development Tooling Category ✅
- **[KNOWLEDGE-WASM-010: CLI Tool Specification](knowledge_wasm_010_cli_tool_specification.md)** ✅ **CRITICAL**
  - **Purpose**: Complete specification for airssys-wasm-cli command-line tool
  - **Scope**: Developer workflow automation, component lifecycle management, modern CLI UX patterns
  - **Key Content**: CLI design philosophy, 14 core commands (keygen, init, build, sign, install, update, uninstall, list, info, logs, status, verify, config, completions), configuration management, error handling, implementation architecture
  - **Status**: Complete CLI specification (Created 2025-10-18)
  - **Impact**: Critical - primary developer interface for component development and deployment
  - **Audience**: Component developers, DevOps engineers, system administrators

### Technical Implementation Category ✅
- **[KNOWLEDGE-WASM-011: Serialization Strategy - bincode vs borsh](knowledge_wasm_011_serialization_strategy.md)** ✅ **CRITICAL**
  - **Purpose**: Comprehensive comparison of bincode vs borsh serialization formats for airssys-wasm
  - **Scope**: Performance benchmarks, schema evolution, cross-language support, production usage analysis
  - **Key Content**: bincode deep dive (Rust-only, 20% faster), borsh deep dive (cross-language, deterministic), hybrid approach recommendation, use case mapping (storage vs messaging), implementation guidelines, migration strategy
  - **Status**: Complete analysis with recommendation (Created 2025-10-19)
  - **Impact**: Critical - foundation decision for component state persistence and inter-component messaging
  - **Audience**: Architects, runtime implementers, component developers
  - **Key Decision**: Hybrid approach - bincode for internal Rust-only storage (performance), borsh for cross-language messaging (multicodec 0x701, deterministic)
  - **Related**: KNOWLEDGE-WASM-006 (multiformat strategy), KNOWLEDGE-WASM-007 (storage architecture), KNOWLEDGE-WASM-005 (messaging)

- **[KNOWLEDGE-WASM-014: Phase 3 Completion Retrospective](knowledge_wasm_014_phase_3_completion_retrospective.md)** ✅ **CRITICAL**
  - **Purpose**: Retrospective analysis of WASM-TASK-003 Phase 3 actual completion status vs. documented status
  - **Scope**: Gap analysis, completion verification, architectural deviations, justification documentation
  - **Key Content**: 95% actual completion (vs. 67% documented), complete WIT system (2,214 lines), extension interfaces fully implemented (1,645 lines), permission system complete, build system functional, all deviations justified (Component.toml manifests, single-package structure), readiness assessment for Block 3
  - **Status**: Complete retrospective (Created 2025-11-29)
  - **Impact**: Critical - reveals Phase 3 ready for Block 3 with only user documentation remaining
  - **Audience**: Project managers, architects, technical leads, memory bank maintainers
  - **Key Finding**: Implementation exceeded original plans with well-documented improvements
  - **Related**: WASM-TASK-003 (Phase 3), DEBT-WASM-003 (Component Model v0.1), KNOWLEDGE-WASM-009 (Component.toml manifests), ADR-WASM-015 (package structure)

## Planned Knowledge Documentation (Future)

### WASM Runtime Category
- **Component Model Implementation**: WebAssembly Component Model patterns and implementation
- **WASM Execution Optimization**: High-performance WASM execution techniques
- **Runtime Security**: WASM runtime security and sandboxing implementation
- **Component Lifecycle**: Component loading, instantiation, and cleanup patterns

### Security Category  
- **Capability-Based Security**: Capability model implementation and enforcement
- **Sandbox Architecture**: WASM security sandbox design and implementation
- **Threat Mitigation**: Security threat analysis and mitigation strategies
- **Audit and Compliance**: Security audit logging and compliance patterns

### Component System Category
- **Component Communication**: Inter-component communication patterns and optimization
- **Component Composition**: Component composition and linking strategies
- **Resource Management**: Component resource management and isolation
- **Hot Reloading**: Dynamic component update and reloading patterns

### Development Category (Partially Complete - 2/4)
- ✅ **Component Installation**: KNOWLEDGE-WASM-009 (Installation architecture and workflows)
- ✅ **CLI Tooling**: KNOWLEDGE-WASM-010 (airssys-wasm-cli specification)
- **Testing Strategies**: Component testing, security testing, and performance testing
- **Debugging Techniques**: Component debugging and troubleshooting approaches

### Integration Category
- **AirsSys Integration**: Integration patterns with airssys-osl and airssys-rt
- **WASI Implementation**: WASI system interface implementation and extensions
- **Host Functions**: Custom host function design and implementation
- **Performance Integration**: Performance optimization for integrated systems

## Knowledge Creation Strategy (Future)

### Phase 1: Foundation Knowledge
1. **WASM Runtime Security**: Document security model and implementation
2. **Component Model Patterns**: Document Component Model implementation patterns
3. **Integration Patterns**: Document AirsSys integration approaches
4. **Performance Optimization**: Document performance-critical implementation details

### Phase 2: Advanced Knowledge  
1. **Component Communication**: Document communication optimization patterns
2. **Security Enforcement**: Document capability enforcement implementation
3. **Resource Management**: Document resource isolation and management
4. **Development Workflows**: Document component development best practices

### Phase 3: Ecosystem Knowledge
1. **Composition Patterns**: Document advanced component composition
2. **Distributed Components**: Document cross-system component patterns
3. **Tool Integration**: Document development tool integration
4. **Production Deployment**: Document production deployment and operations

---
**Note:** Knowledge documentation will be created during implementation phases (estimated Q3 2026+).
## KNOWLEDGE-WASM-019: Runtime Dependency Architecture

**Created:** 2025-12-16  
**Status:** Active  
**Category:** Architecture / Runtime Dependencies

**Purpose:** Comprehensive analysis of runtime dependency architecture, documenting the relationship between Layer 0 (Tokio), Layer 2 (airssys-wasm), and Layer 3 (airssys-rt). Provides verification checklist, design principles, and decision guidelines for future development.

**Key Topics:**
- Runtime layers (Layer 0: Tokio, Layer 3: airssys-rt, Layer 2: airssys-wasm)
- Correlation tracking case study (Phase 5 Task 5.1)
- Runtime responsibility matrix
- Request-response flow analysis
- Performance implications
- Design principles validation
- Common misconceptions
- Verification checklist

**Related:**
- ADR-WASM-018 (Three-Layer Architecture)
- ADR-WASM-019 (Runtime Dependency Management)
- KNOWLEDGE-WASM-016 (Actor System Integration)
- KNOWLEDGE-WASM-018 (Component Definitions)

**Use When:**
- Implementing new airssys-wasm features
- Deciding between Tokio vs airssys-rt usage
- Verifying runtime dependency decisions
- Understanding layer boundaries
- Performance optimization

**File:** `docs/knowledges/knowledge-wasm-019-runtime-dependency-architecture.md`

---
 
## KNOWLEDGE-WASM-020: airssys-osl Security Integration Architecture

**Category**: Integration / Security  
**Created**: 2025-12-17  
**Status**: Active  
**Related**: ADR-WASM-005 (Capability-Based Security), ADR-WASM-010 (Implementation Strategy)

**Purpose**: Documents the integration architecture between airssys-wasm component security and airssys-osl security infrastructure (ACL, RBAC, audit logging).

**Key Content**:
- Integration verification methodology
- ACL, SecurityPolicy, SecurityContext, and audit logger integration points
- Data flow validation (Component.toml → Parser → ACL → Policy)
- Security model alignment verification (deny-by-default, glob patterns)
- Future integration patterns (Tasks 1.3, 3.1, 3.3)

**Audience**: Developers implementing security features in airssys-wasm

**File**: `knowledge-wasm-020-airssys-osl-security-integration.md`

---

## KNOWLEDGE-WASM-021: Architecture References

**Category**: Architecture  
**Created**: 2025-10-23 (moved 2025-12-17)  
**Status**: Active  

**Purpose**: Central reference for airssys-wasm architecture documentation

**File**: `knowledge-wasm-021-architecture-references.md`

---

## KNOWLEDGE-WASM-022: Runtime Architecture Summary

**Category**: Architecture / Runtime  
**Created**: 2025-12-16 (moved 2025-12-17)  
**Status**: Active  
**Related**: ADR-WASM-018 (Three-Layer Architecture), ADR-WASM-019 (Runtime Dependency Management)

**Purpose**: Quick reference for runtime dependency decisions - Tokio vs airssys-rt usage patterns

**Key Content**:
- Three runtime layers (Layer 2: airssys-wasm, Layer 3: airssys-rt, Layer 0: Tokio)
- Golden rules for runtime selection
- Decision guide (when to use which layer)
- Common patterns (timeout handling, channels, message routing)

**Audience**: Developers working on airssys-wasm runtime features

**File**: `knowledge-wasm-022-runtime-architecture-summary.md`

---

## KNOWLEDGE-WASM-023: DashMap Migration Rationale (2025-12-19)

**File:** `knowledge-wasm-023-dashmap-migration-rationale.md`  
**Status:** Active  
**Related:** WASM-TASK-005 Phase 3, ADR-WASM-005

**Summary:** Documents the rationale for migrating from `RwLock<HashMap>` to `DashMap` in Task 3.1 capability registry implementation. Explains how DashMap eliminates RwLock poisoning risks, simplifies code (30% less boilerplate), and improves concurrency through shard-based isolation.

**Key Points:**
- RwLock poisoning poses security risk (one panic kills entire system)
- DashMap provides shard-based locking with failure isolation  
- API simplified from 4-param to 3-param (no registry parameter)
- All tests pass (37 tests, 100% pass rate)
- Production-ready with comprehensive testing

**Impact:** Future tasks (3.2+) must use DashMap-based API, not planned RwLock design.

- **[KNOWLEDGE-WASM-024: Component Messaging Clarifications](knowledge-wasm-024-component-messaging-clarifications.md)** ✅ **CRITICAL**
  - **Purpose**: Critical clarifications about component messaging architecture from Block 5 Phase 1 planning
  - **Scope**: Async-only communication model, two send methods, component perspective, internal vs component-facing features
  - **Key Content**: No synchronous communication, `send-message` vs `send-request` distinction, unified receiver pattern, push-based delivery, runtime-managed subscriptions, common misconceptions corrected
  - **Status**: Active reference (Created 2025-12-21)
  - **Impact**: Critical - prevents implementation mistakes in Block 5, clarifies component developer experience
  - **Audience**: All Block 5 implementers, component developers, architects
  - **Related**: ADR-WASM-009, KNOWLEDGE-WASM-005, WASM-TASK-006

- **[KNOWLEDGE-WASM-025: Message Delivery Mechanism](knowledge-wasm-025-message-delivery-mechanism.md)** ⚠️ **SUPERSEDED**
  - **Purpose**: (SUPERSEDED) Originally proposed extending ComponentRegistry for message delivery
  - **Status**: SUPERSEDED by KNOWLEDGE-WASM-026 (2025-12-21)
  - **Note**: The solution proposed in this document (ComponentRegistry extension) was REJECTED. See KNOWLEDGE-WASM-026 for correct architecture.
  - **Related**: KNOWLEDGE-WASM-026 (replacement)

- **[KNOWLEDGE-WASM-026: Message Delivery Architecture - Final Decision](knowledge-wasm-026-message-delivery-architecture-final.md)** ✅ **CRITICAL**
  - **Purpose**: Final architectural decision for message delivery - ActorSystemSubscriber owns delivery, ComponentRegistry stays pure
  - **Scope**: Complete message flow from WASM send to handle_message invocation, architecture decision rationale, implementation design
  - **Key Content**: Complete 10-step message flow diagram, ActorSystemSubscriber enhancement with `mailbox_senders` HashMap, ComponentSpawner integration, responsibility matrix (Registry=identity, Subscriber=delivery), testing requirements, implementation checklist
  - **Status**: Active reference (Created 2025-12-21) - **SUPERSEDES KNOWLEDGE-WASM-025**
  - **Impact**: Critical - definitive architecture for Block 5 message delivery
  - **Audience**: All Block 5 implementers, message routing developers, architects
  - **Key Decision**: `ActorSystemSubscriber` owns `mailbox_senders` for delivery; `ComponentRegistry` stays pure (identity lookup only)
  - **Related**: ADR-WASM-009, KNOWLEDGE-WASM-024, KNOWLEDGE-WASM-018, WASM-TASK-006
  - **Note**: This document supersedes KNOWLEDGE-WASM-025 which proposed extending ComponentRegistry (rejected)

---

## KNOWLEDGE-WASM-027: Duplicate WASM Runtime - Fatal Architecture Violation 🔴

**File:** `knowledge-wasm-027-duplicate-wasm-runtime-fatal-architecture-violation.md`  
**Status:** Active  
**Created:** 2025-12-21  
**Category:** Architecture / Fatal Errors / Lessons Learned  
**Severity:** 🔴 **CRITICAL / FATAL**

**Summary:** Documents a fatal architectural violation where a duplicate WASM runtime was created in `actor/component/` using the WRONG API (core WASM `wasmtime::Module`) instead of the CORRECT API (Component Model `wasmtime::component::Component`). This violates ADR-WASM-002 and renders the entire WIT interface system non-functional.

**Key Points:**
- `runtime/engine.rs` correctly uses Component Model API (IGNORED)
- `actor/component/child_impl.rs` uses core WASM API (WRONG)
- 154KB of generated bindings (`src/generated/`) are completely UNUSED
- 250+ lines of workaround code (WasmBumpAllocator, etc.) exist as workarounds
- Circular dependency: `runtime/` imports from `actor/` (WRONG direction)

**Impact:**
- All WIT interfaces non-functional
- Type safety bypassed
- Security guarantees of Canonical ABI bypassed
- 3-5 days of refactoring required to fix

**Related:**
- ADR-WASM-002 (mandates Component Model)
- ADR-WASM-021 (remediation plan)
- ADR-WASM-018 (layer architecture)

**Audience:** All developers, architects, future maintainers - MUST READ before working on component execution

## KNOWLEDGE-WASM-028: Circular Dependency Between actor/ and runtime/ 🔴

**File:** `knowledge-wasm-028-circular-dependency-actor-runtime.md`  
**Status:** Active  
**Created:** 2025-12-21  
**Category:** Architecture / Fatal Errors / Module Dependencies  
**Severity:** 🔴 **CRITICAL**

**Summary:** Documents a critical architectural violation where `runtime/` module incorrectly imports types from `actor/` module, creating a circular dependency. The correct dependency direction is one-way: `actor/ → runtime/ → core/`.

**Evidence:**
- `runtime/async_host.rs:52` imports `crate::actor::ComponentMessage`
- `runtime/messaging.rs:76` imports `crate::actor::ComponentMessage`
- `runtime/messaging_subscription.rs:108-109` imports `ComponentRegistry`, `ActorSystemSubscriber`

**Root Cause:**
- `ComponentMessage` is a data type but placed in `actor/` instead of `core/`
- `messaging_subscription.rs` contains integration logic that belongs in `actor/`

**Remediation:**
1. Move `ComponentMessage` to `core/`
2. Move `messaging_subscription.rs` to `actor/component/`
3. Add CI check to enforce layer boundaries

**Related:**
- ADR-WASM-018 (three-layer architecture)
- ADR-WASM-022 (remediation plan)
- KNOWLEDGE-WASM-027 (related duplicate runtime issue)

**Audience:** All developers - MUST understand correct layer dependencies

## KNOWLEDGE-WASM-029: Messaging Patterns - Fire-and-Forget vs Request-Response

**File:** `knowledge-wasm-029-messaging-patterns-fire-and-forget-vs-request-response.md`  
**Status:** Active  
**Created:** 2025-12-22  
**Category:** Architecture / Patterns / Messaging  
**Maturity:** Stable

**Summary:** Comprehensive explanation of the two messaging patterns in airssys-wasm. Documents the critical insight that there is NO `send-response` host function - the response IS the return value from `handle-message`. Runtime decides what to do with the return value based on whether the message has a correlation_id (request-response) or not (fire-and-forget).

**Key Points:**
- Fire-and-forget: `send-message()` → return value IGNORED
- Request-response: `send-request()` → return value CAPTURED and routed to `handle-callback`
- Same `handle-message` export handles BOTH patterns
- Component B doesn't know which pattern is being used - runtime decides
- NO `send-response` host function needed

**Impact:** Critical for Task 3.2 implementation - clarifies that response routing captures `handle-message` return value, not a separate host call.

**Related:**
- KNOWLEDGE-WASM-005 (Messaging Architecture)
- KNOWLEDGE-WASM-024 (Component Messaging Clarifications)  
- ADR-WASM-009 (Component Communication Model)
- WASM-TASK-006 (Block 5 - Inter-Component Communication)

**Audience:** Block 5 implementers, component developers, architects

## KNOWLEDGE-WASM-030: Module Architecture - Hard Requirements 🔴 MANDATORY

**File:** `knowledge-wasm-030-module-architecture-hard-requirements.md`  
**Status:** 🔴 **MANDATORY - HARD REQUIREMENTS**  
**Created:** 2025-12-22  
**Category:** Architecture / Module Design / Enforcement  
**Related ADR:** ADR-WASM-023 (Module Boundary Enforcement)

**Summary:** Defines MANDATORY module architecture for airssys-wasm. These are HARD REQUIREMENTS that MUST BE FOLLOWED without exception.

**The Four Modules and Their Purposes:**

| Module | Purpose | Imports From | Never Imports From |
|--------|---------|--------------|-------------------|
| `core/` | Shared types & abstractions | Nothing | Everything |
| `security/` | Security logic | `core/` | `runtime/`, `actor/` |
| `runtime/` | WASM execution | `core/`, `security/` | `actor/` |
| `actor/` | Actor integration | `core/`, `security/`, `runtime/` | N/A |

**Key Content:**
- Module responsibilities (what belongs where)
- Decision flow diagram (where does this code belong?)
- Common mistakes and corrections
- Verification commands
- Enforcement requirements

**Why This Exists:** Repeated architectural violations have caused significant development delays. This document ensures module separation (which was designed for specific reasons) is followed.

**Audience:** ALL developers, planners, implementers - MANDATORY reading before any code changes

---

## KNOWLEDGE-WASM-032: Module Boundary Violations Audit

**Created:** 2025-12-22  
**Status:** 🔴 CRITICAL - ARCHITECTURE BROKEN  
**Category:** Architecture Audit

**Purpose:** Comprehensive audit of module boundary violations in airssys-wasm. Documents that architecture is fundamentally broken with `core/` importing from `runtime/` and `runtime/` importing from `actor/`, violating ADR-WASM-023.

**Key Topics:**
- ADR-WASM-023 required module hierarchy
- Violation #1: `core/config.rs` → `runtime/limits.rs` (CRITICAL)
- Violation #2: `runtime/messaging.rs` → `actor/message/` (CRITICAL)
- Verification commands that MUST pass
- Required fix actions
- CI enforcement recommendations
- Historical context and lessons learned

**Impact:** 🔴 **FATAL** - Architecture is broken. No new features until fixed.

**Related:**
- ADR-WASM-023 (Module Boundary Enforcement)
- KNOWLEDGE-WASM-030 (Module Architecture Hard Requirements)
- WASM-TASK-006-HOTFIX-002 (Hotfix task - NOT COMPLETED)

**Use When:**
- Before implementing ANY new code
- Before claiming ANY task is "verified"
- As verification checklist after ANY module changes

**Verification Commands:**
```bash
# ALL must return NOTHING for architecture to be valid
grep -rn "use crate::runtime" src/core/
grep -rn "use crate::actor" src/core/
grep -rn "use crate::actor" src/runtime/
```

**Current Status (2025-12-22):** ❌ ALL CHECKS FAIL

---

## KNOWLEDGE-WASM-033: AI Fatal Mistakes - Lessons Learned 🔴 MANDATORY READING

**File:** `knowledge-wasm-033-ai-fatal-mistakes-lessons-learned.md`  
**Status:** 🔴 **CRITICAL - MANDATORY READING**  
**Created:** 2025-12-22  
**Category:** Lessons Learned / AI Failures / Process Improvement  
**Severity:** 🔴 **FATAL**

**Summary:** Documents FATAL MISTAKES made by AI agents during airssys-wasm development that caused architecture violations, wasted 10+ days of development time, and destroyed trust. Records specific failures: claims without evidence, proceeding without reading ADRs, ignoring module boundaries, creating STUB tests, and claiming completion without verification.

**Key Topics:**
- FATAL MISTAKE #1: Claims of "Verified" Without Evidence
- FATAL MISTAKE #2: Proceeding Without Reading ADRs/Knowledges
- FATAL MISTAKE #3: Ignoring Module Boundary Rules
- FATAL MISTAKE #4: Creating STUB Tests Instead of REAL Tests
- FATAL MISTAKE #5: Claiming Completion Without Verification
- Root cause analysis
- New mandatory rules
- Verification commands that MUST be run
- Commitment to never repeat these mistakes

**Impact:** 10+ days wasted, complete loss of trust, architecture broken

**Audience:** 🔴 **ALL AI AGENTS - MANDATORY READING BEFORE ANY WORK**

**Related:**
- ADR-WASM-023 (Module Boundary Enforcement)
- KNOWLEDGE-WASM-030 (Module Architecture Hard Requirements)
- KNOWLEDGE-WASM-032 (Module Boundary Violations Audit)
- AGENTS.md §9-12

### Architecture & Remediation Category 🔴
- **[KNOWLEDGE-WASM-034: Module Architecture Violation - Messaging in Runtime](knowledge-wasm-034-module-architecture-violation-messaging-in-runtime.md)** 🔴 **CRITICAL FIX**
  - **Purpose**: Documents critical architectural violation where messaging infrastructure is incorrectly placed in `runtime/` module
  - **Scope**: Module boundary violations, circular dependency risk, missing top-level messaging/ module
  - **Key Content**: Current wrong state (runtime/messaging.rs with 1,313 lines), correct architecture (top-level messaging/), one-way dependency chain, violation of ADR-WASM-018 and KNOWLEDGE-WASM-012
  - **Status**: New document (Created 2025-12-26)
  - **Impact**: 🔴 CRITICAL - Must fix before Block 5 development
  - **Audience**: All developers, anyone working on module architecture or refactoring
  - **Related**: ADR-WASM-024 (decision to refactor messaging to top-level), WASM-TASK-HOTFIX-001 (implementation)

### Module Architecture Reference ✅
- **[KNOWLEDGE-WASM-012: Module Structure Architecture](knowledge_wasm_012_module_structure_architecture.md)** ✅ **CRITICAL**
  - **Purpose**: Definitive module organization specification for airssys-wasm
  - **Scope**: Complete module structure, dependency rules, public API surface
  - **Key Content**: Three-layer architecture, module responsibility matrix, dependency graph, block alignment, testing organization
  - **Status**: Complete definitive specification (Updated 2025-10-21)
  - **Impact**: Critical - defines all module boundaries and rules
   - **Audience**: All developers, architects, task implementers
   - **Related**: All tasks and ADRs reference this for module structure compliance

- **[KNOWLEDGE-WASM-036: Three-Module Architecture](knowledge-wasm-036-three-module-architecture.md)** ⚠️ **SUPERSEDED**
  - **Purpose**: (SUPERSEDED) Previous four-module architecture with host_system/
  - **Status**: ⚠️ SUPERSEDED by KNOWLEDGE-WASM-037 (2026-01-05)
  - **Note**: This document has been superseded. See KNOWLEDGE-WASM-037 for the current rebuild architecture.
  - **Related**: KNOWLEDGE-WASM-037 (replacement)

---
