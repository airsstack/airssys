# ADR-WASM-010: Implementation Strategy and Build Order

**ADR ID:** ADR-WASM-010  
**Created:** 2025-10-20  
**Updated:** 2025-10-20  
**Status:** Accepted  
**Deciders:** Architecture Team  

## Title
Implementation Strategy and Build Order for airssys-wasm Component Framework

## Context

### Problem Statement

The airssys-wasm project is a complex, multi-layered component framework consisting of 11 major building blocks with intricate dependencies. Without a clear implementation strategy and build order, development could proceed in the wrong sequence, leading to:

1. **Architectural Rework** - Building components that depend on non-existent foundations
2. **Circular Dependencies** - Discovering dependency cycles late in development
3. **Integration Failures** - Components built in isolation failing to integrate properly
4. **Wasted Effort** - Implementing features that need to be refactored due to wrong ordering
5. **Timeline Delays** - Blocked work streams waiting for prerequisite components

### Business Context

The airssys-wasm framework targets secure plugin systems, polyglot application development, and AirsSys ecosystem enhancement. Given its complexity and the target delivery timeline (Q3 2026+), a clear implementation roadmap is critical for:

- **Predictable Delivery** - Stakeholders need confidence in timeline estimates
- **Resource Planning** - Team allocation requires understanding of parallel vs. sequential work
- **Risk Management** - Early identification of critical path components
- **Incremental Value** - Delivering working subsystems before full completion

### Technical Context

airssys-wasm integrates with two foundational AirsSys components:

1. **airssys-osl** (OS Layer) - Currently 85% complete, provides secure system operations
2. **airssys-rt** (Runtime System) - Foundation phase complete, provides actor model and message routing

The framework consists of 11 identified building blocks:
1. WASM Runtime Layer
2. Component Lifecycle System
3. Security & Isolation Layer
4. Inter-Component Communication
5. Persistent Storage System
6. WIT Interface System
7. Component Development SDK
8. CLI Tool
9. Actor System Integration
10. AirsSys Ecosystem Bridges
11. Monitoring & Observability

**Critical Discovery:** Initial analysis suggested Actor System Integration as Block 9 (integration layer), but architectural review revealed it must be Block 3 (foundation layer) due to widespread dependencies.

### Stakeholders

- **Framework Developers** - Need clear implementation sequence
- **Component Developers** - Need SDK and tooling delivered early
- **System Architects** - Need assurance of sound architectural approach
- **Project Management** - Need accurate timeline and dependency understanding

## Decision

### Summary

**We adopt a 4-layer phased implementation strategy with Actor System Integration as a foundational component (Layer 1, Block 3), not an integration layer component.**

The implementation follows this corrected build order:

**Layer 1: Foundation (Blocks 1-3)**
1. WASM Runtime Layer
2. WIT Interface System
3. Actor System Integration ⭐ **CRITICAL CORRECTION**

**Layer 2: Core Services (Blocks 4-7)**
4. Security & Isolation Layer
5. Inter-Component Communication
6. Persistent Storage System
7. Component Lifecycle System

**Layer 3: Integration & Operations (Blocks 8-9)**
8. AirsSys-OSL Bridge
9. Monitoring & Observability

**Layer 4: Developer Experience (Blocks 10-11)**
10. Component Development SDK
11. CLI Tool

### Rationale

**Why Actor System Integration Must Be Block 3 (Foundation):**

1. **MessageBroker is Core Infrastructure** - Not an "integration", it's the communication backbone
   - Inter-Component Communication (Block 5) requires `airssys-rt::MessageBroker`
   - Component Lifecycle (Block 7) requires `SupervisorNode` for health management
   - Security & Isolation (Block 4) requires `Actor` and `Child` traits

2. **ComponentActor Pattern is Fundamental** - All components ARE actors from the start
   - ADR-WASM-006: "Each component instance is hosted within its own ComponentActor"
   - KNOWLEDGE-WASM-005: "Components execute within ComponentActor instances managed by airssys-rt ActorSystem"

3. **Supervision is Not Optional** - Component lifecycle depends on SupervisorNode
   - Automatic restart on component failure
   - Health monitoring and status reporting
   - Graceful shutdown and cleanup

4. **Isolation Model Requires Actors** - ADR-WASM-006 explicitly uses Erlang-style lightweight processes
   - ~625ns actor spawn time
   - 10,000+ concurrent actors proven
   - Private mailbox isolation

**The Correct Mental Model:**
- ❌ **Wrong**: "WASM components, then integrate actors later"
- ✅ **Right**: "Actor-hosted WASM components from the start"

This is analogous to:
- You can't build a web server without an HTTP library
- You can't build async code without Tokio
- **You can't build airssys-wasm without airssys-rt**

### Assumptions

1. **airssys-rt Stability** - airssys-rt foundation phase is complete and API stable
2. **Sequential Implementation** - Later layers cannot begin until prerequisites complete
3. **Parallel Work Within Layers** - Blocks within same layer can be developed in parallel
4. **Incremental Integration** - Each layer integrates and validates before next layer begins
5. **Resource Availability** - Development team can work on multiple blocks when parallelizable

## Implementation Strategy

### Layer 1: Foundation (Estimated: 3-4 months)

#### Block 1: WASM Runtime Layer
**Purpose:** Execute WASM components with security and resource control

**Key Deliverables:**
- Wasmtime integration with Component Model support
- Linear memory sandbox (512KB-4MB configurable)
- Hybrid CPU limiting (fuel metering + wall-clock timeout)
- Async execution with Tokio integration
- Isolated crash handling

**Dependencies:** None (can start immediately)

**Success Criteria:**
- Load and execute WASM Component Model modules
- Enforce memory limits configured in Component.toml
- Enforce CPU limits with dual protection
- Handle component crashes without host termination
- Async function calls work correctly

**Estimated Effort:** 4-6 weeks

---

#### Block 2: WIT Interface System
**Purpose:** Define component interfaces and host services

**Key Deliverables:**
- Core WIT interface definitions (messaging, storage, logging, capabilities)
- Host service interface specifications
- Permission annotation patterns in WIT
- Binding generation for Rust (initial target)
- Interface validation at component load time

**Dependencies:** None (can start immediately, parallel with Block 1)

**Success Criteria:**
- WIT files define complete component interface
- Host service WIT files define all host functions
- Rust bindings generate correctly
- Interface compatibility checking works
- Documentation for WIT interface design

**Estimated Effort:** 3-4 weeks

---

#### Block 3: Actor System Integration ⭐ **FOUNDATIONAL**
**Purpose:** Bridge WASM components with airssys-rt actor system

**Key Deliverables:**
- `ComponentActor` struct implementing `Actor` + `Child` traits
- WASM lifecycle hooks in `Child::start()` and `Child::stop()`
- `ActorSystem::spawn()` integration (NOT manual `tokio::spawn`)
- `SupervisorNode` integration for component supervision
- `MessageBroker` integration for pub-sub messaging
- ActorSystem as primary subscriber pattern

**Dependencies:** 
- Block 1 (WASM Runtime Layer) - needs WASM instance management
- airssys-rt foundation (external dependency)

**Success Criteria:**
- ComponentActor implements both Actor and Child traits
- WASM loads in Child::start(), cleans up in Child::stop()
- SupervisorNode successfully supervises component lifecycle
- MessageBroker routes messages to ComponentActor mailbox
- Component crashes trigger supervisor restart policies
- ActorSystem::spawn() used for all component instantiation

**Estimated Effort:** 4-5 weeks

**Critical Note:** This MUST complete before Layer 2 can begin. All core services depend on actor infrastructure.

---

### Layer 2: Core Services (Estimated: 4-5 months)

**Prerequisites:** Layer 1 complete (Blocks 1-3)

#### Block 4: Security & Isolation Layer
**Purpose:** Multi-layered defense for untrusted third-party components

**Key Deliverables:**
- Capability system with pattern matching (filesystem globs, network domains, storage namespaces)
- Trust-level workflow (trusted instant, unknown review, dev mode bypass)
- Actor isolation patterns using ComponentActor
- Supervision trees with automatic restart
- Component.toml capability declarations
- Host function entry point capability checks

**Dependencies:** 
- Block 1 (WASM Runtime) - memory sandbox
- Block 3 (Actor System) - actor isolation and supervision

**Success Criteria:**
- Capability patterns correctly match resources
- Trust levels enforce appropriate approval workflows
- Actor isolation provides process-like separation
- Supervision automatically restarts failed components
- Host functions enforce capabilities before system access
- ~1-5μs capability check overhead (0.1% of operation)

**Estimated Effort:** 5-6 weeks

---

#### Block 5: Inter-Component Communication
**Purpose:** Enable components to communicate securely

**Key Deliverables:**
- MessageBroker integration for message routing
- Fire-and-forget message pattern (~280ns)
- Request-response with callbacks (~560ns round-trip)
- Pub-sub topic subscription
- Host function security layer (capability + quota + audit)
- Multicodec self-describing serialization
- Push-based delivery via `handle-message` export

**Dependencies:** 
- Block 3 (Actor System) - **REQUIRES MessageBroker** ⚠️ CRITICAL
- Block 4 (Security) - capability enforcement

**Success Criteria:**
- MessageBroker routes messages at ~211ns baseline
- Total messaging overhead ~260ns (validation + routing + delivery)
- Fire-and-forget completes in ~280ns
- Request-response round-trip ~560ns
- Capability checks enforce messaging permissions
- Quota enforcement prevents message spam
- Audit logging captures all message events
- Throughput: 4.7M messages/sec (MessageBroker proven capacity)

**Estimated Effort:** 5-6 weeks

---

#### Block 6: Persistent Storage System
**Purpose:** Component state persistence with backend flexibility

**Key Deliverables:**
- NEAR-style KV API (get/set/delete/scan_prefix)
- `StorageBackend` trait abstraction
- Sled backend implementation (default)
- RocksDB backend implementation (optional)
- Prefix-based namespace isolation (`component:<id>:`)
- Application-level quota tracking and enforcement
- Export/import JSON Lines tool for migration

**Dependencies:** 
- Block 4 (Security) - capability enforcement for storage operations

**Success Criteria:**
- KV API works correctly with Sled backend
- RocksDB backend passes same test suite
- Namespace isolation prevents cross-component access
- Quota enforcement limits storage per component
- Export/import tool successfully migrates data
- Performance: <1ms for typical get/set operations

**Estimated Effort:** 4-5 weeks

---

#### Block 7: Component Lifecycle System
**Purpose:** Install, update, and manage component versions

**Key Deliverables:**
- Installation engine (Git via libgit2, Local file, Remote URL)
- Immutable component storage with versioning
- Blue-green routing proxy for zero-downtime updates
- Configurable retention policies (default 24h rollback window)
- Ed25519 cryptographic ownership verification
- Component registry with version tracking
- Routing table for component addressing

**Dependencies:** 
- Block 3 (Actor System) - **REQUIRES SupervisorNode for lifecycle** ⚠️ CRITICAL
- Block 4 (Security) - Ed25519 signature verification
- Block 6 (Storage) - component version storage

**Success Criteria:**
- Install from Git repository works correctly
- Install from local file works for development
- Install from URL works for pre-built components
- Blue-green routing switches in <1ms
- Retention policies auto-cleanup old versions
- Ed25519 signatures prevent unauthorized updates
- Rollback to previous version works correctly

**Estimated Effort:** 6-7 weeks

---

### Layer 3: Integration & Operations (Estimated: 2-3 months)

**Prerequisites:** Layer 2 complete (Blocks 4-7)

#### Block 8: AirsSys-OSL Bridge
**Purpose:** Connect WASM components to OS layer operations

**Key Deliverables:**
- Host functions for filesystem operations (read/write/stat)
- Host functions for process operations (spawn/kill/signal)
- Host functions for network operations (TCP/UDP/HTTP)
- Layered security enforcement (capabilities + RBAC/ACL + OS permissions)
- Operation audit logging
- Error translation from OS errors to WASM traps

**Dependencies:** 
- Block 4 (Security) - capability enforcement
- airssys-osl maturity (external dependency - currently 85% complete)

**Success Criteria:**
- Filesystem operations work with capability checks
- Process operations enforce security policies
- Network operations validate permissions
- Layered security provides defense-in-depth
- Audit logs capture all system operations
- Error handling provides meaningful component feedback

**Estimated Effort:** 5-6 weeks

---

#### Block 9: Monitoring & Observability
**Purpose:** Production visibility into component behavior

**Key Deliverables:**
- Metrics collection (CPU, memory, storage, network, message throughput)
- Health monitoring with SupervisorNode integration
- Audit logging (security events, capability usage, permission denials)
- Performance tracing (message latency, WASM execution time, host function overhead)
- Alerting system with threshold-based triggers
- Dashboard integration (Prometheus/Grafana compatible)

**Dependencies:** 
- Block 3 (Actor System) - SupervisorNode health monitoring
- Block 4 (Security) - security event logging
- Block 5 (Communication) - message metrics
- Block 6 (Storage) - storage usage metrics

**Success Criteria:**
- Metrics exported in Prometheus format
- Health checks report component status accurately
- Security events logged with full context
- Performance traces identify bottlenecks
- Alerts trigger on threshold violations
- Dashboard provides real-time visibility

**Estimated Effort:** 4-5 weeks

---

### Layer 4: Developer Experience (Estimated: 2-3 months)

**Prerequisites:** Layer 3 complete (Blocks 8-9)

**Note:** These can begin earlier with mocked backends, but full functionality requires core systems.

#### Block 10: Component Development SDK
**Purpose:** Simplify component creation for developers

**Key Deliverables:**
- `#[component]` procedural macro (airssys-wasm-component crate)
- Component.toml generation and validation
- Builder patterns for component creation
- Testing utilities and mock framework
- Documentation and examples (Rust, C, Go, Python, JS)
- Component template generator

**Dependencies:** 
- Block 2 (WIT Interface) - interface definitions
- Block 7 (Lifecycle) - Component.toml specification

**Success Criteria:**
- `#[component]` macro generates correct boilerplate
- Component.toml validated at build time
- Builder patterns reduce boilerplate significantly
- Testing utilities enable unit testing without host
- Examples demonstrate all supported languages
- Template generator creates working component scaffold

**Estimated Effort:** 5-6 weeks

---

#### Block 11: CLI Tool
**Purpose:** Command-line tool for component lifecycle management

**Key Deliverables:**
- Key management: `keygen` for Ed25519 keypair generation
- Component creation: `init` scaffolding, `build` compilation, `sign` cryptographic signing
- Deployment: `install`, `update`, `uninstall` with multi-source support
- Inspection: `list`, `info`, `status`, `logs` for visibility
- Security: `verify` signature validation
- Configuration: `config` management, shell `completions`

**Dependencies:** 
- Block 7 (Lifecycle) - installation and update operations
- Block 9 (Monitoring) - logs and status commands
- Block 10 (SDK) - integration with development workflow

**Success Criteria:**
- Complete developer workflow: keygen → init → build → sign → install
- Multi-source installation works (Git/Local/URL)
- Component inspection provides useful information
- Logs command streams component output
- Verify command catches invalid signatures
- Shell completions work for bash/zsh/fish

**Estimated Effort:** 4-5 weeks

---

## Dependency Graph

```
┌─────────────────────────────────────────────────────────────────┐
│ LAYER 1: FOUNDATION                                             │
│                                                                 │
│  ┌──────────────────┐    ┌─────────────────┐                  │
│  │ 1. WASM Runtime  │    │ 2. WIT Interface│                  │
│  │    (Wasmtime)    │    │     System      │                  │
│  └────────┬─────────┘    └─────────────────┘                  │
│           │                                                     │
│           └───────────────────┐                                │
│                               ▼                                │
│                 ┌──────────────────────────────┐               │
│                 │ 3. Actor System Integration  │ ⭐ CRITICAL   │
│                 │    (airssys-rt bridge)       │               │
│                 │  - ComponentActor (Actor+Child)             │
│                 │  - MessageBroker integration │               │
│                 │  - SupervisorNode integration│               │
│                 └────────────┬─────────────────┘               │
│                              │                                 │
└──────────────────────────────┼─────────────────────────────────┘
                               │
                               │ BLOCKS 4-7 ALL DEPEND ON BLOCK 3
                               │
┌──────────────────────────────┼─────────────────────────────────┐
│ LAYER 2: CORE SERVICES       │                                 │
│                              │                                 │
│           ┌──────────────────┴──────────────────┐              │
│           │                                     │              │
│           ▼                                     ▼              │
│  ┌─────────────────────┐          ┌──────────────────────────┐│
│  │ 4. Security &       │          │ 5. Inter-Component       ││
│  │    Isolation        │◄─────────│    Communication         ││
│  │  (uses Actor)       │          │  (uses MessageBroker)    ││
│  └──────────┬──────────┘          └────────────┬─────────────┘│
│             │                                   │              │
│             │              ┌────────────────────┘              │
│             │              │                                   │
│             ▼              ▼                                   │
│  ┌─────────────────────────────────┐                          │
│  │ 6. Persistent Storage           │                          │
│  │    (capabilities from Block 4)  │                          │
│  └──────────┬──────────────────────┘                          │
│             │                                                  │
│             ▼                                                  │
│  ┌─────────────────────────────────┐                          │
│  │ 7. Component Lifecycle          │                          │
│  │    (uses SupervisorNode)        │                          │
│  └──────────┬──────────────────────┘                          │
│             │                                                  │
└─────────────┼──────────────────────────────────────────────────┘
              │
┌─────────────┼──────────────────────────────────────────────────┐
│ LAYER 3: INTEGRATION & OPERATIONS                              │
│             │                                                  │
│             ▼                                                  │
│  ┌─────────────────────┐    ┌────────────────────────┐        │
│  │ 8. AirsSys-OSL      │    │ 9. Monitoring &        │        │
│  │    Bridge           │    │    Observability       │        │
│  │  (host functions)   │    │  (metrics + health)    │        │
│  └─────────────────────┘    └────────────────────────┘        │
│                                                                │
└────────────────────────────────────────────────────────────────┘
              │
┌─────────────┼──────────────────────────────────────────────────┐
│ LAYER 4: DEVELOPER EXPERIENCE                                  │
│             │                                                  │
│             ▼                                                  │
│  ┌─────────────────────┐    ┌────────────────────────┐        │
│  │ 10. Component SDK   │    │ 11. CLI Tool           │        │
│  │     (macros)        │    │     (workflow)         │        │
│  └─────────────────────┘    └────────────────────────┘        │
│                                                                │
└────────────────────────────────────────────────────────────────┘
```

## Critical Dependencies

### Why Block 3 (Actor System) is Foundational

| **Dependent Block** | **Requires from Block 3** | **Impact if Built Out of Order** |
|---------------------|---------------------------|-----------------------------------|
| **Block 4: Security & Isolation** | `Actor` + `Child` traits for isolation | Cannot implement actor-based isolation, must use inferior OS processes |
| **Block 5: Inter-Component Communication** | `MessageBroker` for routing | Cannot route messages, must build custom routing (duplicate work) |
| **Block 7: Component Lifecycle** | `SupervisorNode` for supervision | Cannot auto-restart components, manual lifecycle management |
| **Block 9: Monitoring** | `SupervisorNode` for health | Cannot monitor component health, no supervisor integration |

### Performance Characteristics

**Foundation Layer (Blocks 1-3):**
- WASM instantiation: 2-11ms (Block 1)
- Actor spawn: ~625ns (Block 3)
- Message routing: ~211ns (Block 3)

**Core Services Layer (Blocks 4-7):**
- Capability check: ~1-5μs, 0.1% overhead (Block 4)
- Fire-and-forget message: ~280ns (Block 5)
- Request-response: ~560ns round-trip (Block 5)
- Storage get/set: <1ms (Block 6)
- Blue-green route switch: <1ms (Block 7)

**Target Metrics:**
- Concurrent components: 10,000+ (actor isolation)
- Message throughput: 4.7M messages/sec (MessageBroker proven)
- Memory per component: ~1-2MB (WASM instance + actor)

## Timeline and Milestones

### Overall Timeline: 11-15 months

**Phase 1: Foundation (Months 1-4)**
- Milestone 1.1: WASM Runtime operational (Month 2)
- Milestone 1.2: WIT interfaces complete (Month 2)
- Milestone 1.3: Actor integration complete (Month 4)
- **Gate**: Foundation layer validated before proceeding

**Phase 2: Core Services (Months 5-9)**
- Milestone 2.1: Security & Communication operational (Month 7)
- Milestone 2.2: Storage system complete (Month 8)
- Milestone 2.3: Component lifecycle working (Month 9)
- **Gate**: Core services validated before proceeding

**Phase 3: Integration & Operations (Months 10-12)**
- Milestone 3.1: OSL bridge operational (Month 11)
- Milestone 3.2: Monitoring complete (Month 12)
- **Gate**: Integration validated before proceeding

**Phase 4: Developer Experience (Months 13-15)**
- Milestone 4.1: SDK complete (Month 14)
- Milestone 4.2: CLI tool complete (Month 15)
- **Gate**: End-to-end developer workflow validated

### Parallelization Opportunities

**Layer 1 (Blocks 1-2 parallel):**
- Block 1 (WASM Runtime) and Block 2 (WIT Interface) can develop in parallel
- Block 3 (Actor Integration) waits for Block 1

**Layer 2 (Blocks 4-6 parallel after Block 3):**
- Block 4 (Security) can start immediately after Block 3
- Block 5 (Communication) can start immediately after Block 3
- Block 6 (Storage) can start after Block 4 begins
- Block 7 (Lifecycle) waits for Blocks 4, 6

**Layer 3 (Blocks 8-9 parallel):**
- Both can develop in parallel after Layer 2 completes

**Layer 4 (Blocks 10-11 parallel):**
- Both can develop in parallel (with mocked backends earlier)

## Implications

### System Impact

**Architectural Correctness:**
- Correct build order prevents circular dependencies
- Actor-first foundation ensures consistent isolation model
- Layered approach enables incremental validation

**Integration Quality:**
- Early actor integration ensures seamless MessageBroker usage
- WIT interfaces defined before implementation prevents interface mismatches
- Security layer built before services ensures security-first development

### Performance Impact

**Positive Performance Outcomes:**
- Actor-based isolation: ~625ns spawn vs 50-100ms OS processes (100,000x faster)
- MessageBroker routing: ~211ns vs custom routing (proven performance)
- Supervision integration: Automatic restart without custom failure handling

**Performance Validation:**
- Each layer includes performance benchmarks before next layer
- Target metrics tracked from Block 3 onwards
- Performance regression detection automated

### Security Impact

**Security-First Development:**
- Security layer (Block 4) built before most services ensures deny-by-default
- Capability enforcement integrated into all system operations
- Trust-level workflow prevents accidental untrusted component installation

**Defense-in-Depth:**
- 4-layer isolation (Capability → WASM → Actor → Supervision) built correctly
- Layered security (capabilities + RBAC/ACL + OS) implemented systematically
- Audit logging integrated from the start

### Scalability Impact

**Horizontal Scalability:**
- Actor model enables 10,000+ concurrent components
- MessageBroker scales to 4.7M messages/sec
- Storage abstraction allows backend optimization

**Development Scalability:**
- Clear layer boundaries enable team parallelization
- Each block has well-defined interfaces
- Incremental validation reduces integration risk

### Operational Impact

**Production Readiness:**
- Monitoring (Block 9) built before full deployment
- Health checks integrated with supervisor from start
- Audit logging enables security incident response

**Developer Experience:**
- SDK and CLI (Blocks 10-11) built after core systems are stable
- Developer tools use production-ready backends
- Documentation benefits from complete system understanding

## Risks and Mitigations

### Risk 1: airssys-rt API Changes
**Impact:** High - Block 3 depends on stable airssys-rt API  
**Probability:** Low - airssys-rt foundation phase complete  
**Mitigation:**
- Lock airssys-rt version before Block 3 implementation
- Coordinate with airssys-rt team on API stability guarantees
- Build adapter layer if breaking changes occur

### Risk 2: Wasmtime Breaking Changes
**Impact:** High - Block 1 depends on Wasmtime Component Model  
**Probability:** Medium - Wasmtime still evolving  
**Mitigation:**
- Lock Wasmtime version to stable release
- Follow Wasmtime release notes carefully
- Allocate time for Wasmtime upgrade sprints

### Risk 3: Layer Gate Validation Failures
**Impact:** Medium - Could block next layer development  
**Probability:** Medium - Complex integration points  
**Mitigation:**
- Comprehensive integration tests at each gate
- Buffer time between layers for issue resolution
- Parallel investigation of next layer during validation

### Risk 4: Performance Target Misses
**Impact:** Medium - Could require architectural changes  
**Probability:** Low - Based on proven airssys-rt benchmarks  
**Mitigation:**
- Continuous performance monitoring from Block 3
- Early performance testing with realistic workloads
- Performance budget allocated per block

### Risk 5: Resource Constraints
**Impact:** High - Could delay timeline significantly  
**Probability:** Medium - 11-15 month commitment  
**Mitigation:**
- Clearly defined block boundaries enable outsourcing
- Documentation enables onboarding of additional resources
- Parallelization opportunities reduce critical path

## Related Decisions

### Foundation ADRs (Dependency Chain)
- **ADR-WASM-002: WASM Runtime Engine Selection** - Block 1 implementation
- **ADR-WASM-006: Component Isolation and Sandboxing** - Block 3 actor integration, Block 4 security
- **ADR-WASM-009: Component Communication Model** - Block 5 messaging (depends on Block 3)

### Core Services ADRs
- **ADR-WASM-005: Capability-Based Security Model** - Block 4 security implementation
- **ADR-WASM-001: Multicodec Compatibility Strategy** - Block 5 serialization
- **ADR-WASM-007: Storage Backend Selection** - Block 6 storage implementation
- **ADR-WASM-003: Component Lifecycle Management** - Block 7 lifecycle implementation

### Knowledge Documentation
- **KNOWLEDGE-WASM-001: Component Framework Architecture** - Overall architecture context
- **KNOWLEDGE-WASM-005: Inter-Component Messaging Architecture** - Block 5 detailed design
- **KNOWLEDGE-WASM-009: Component Installation Architecture** - Block 7 installation design
- **KNOWLEDGE-WASM-010: CLI Tool Specification** - Block 11 CLI design

### External Dependencies
- **airssys-rt foundation** - Block 3 dependency (MessageBroker, Actor system, SupervisorNode)
- **airssys-osl maturity** - Block 8 dependency (filesystem, process, network operations)

## Revision History

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-10-20 | 1.0 | Initial ADR creation documenting implementation strategy and corrected build order | Architecture Team |

## Approval

**Status:** Accepted  
**Date:** 2025-10-20  
**Approvers:** Architecture Team

---

**Critical Takeaway:** Actor System Integration (Block 3) is foundational infrastructure, not an integration layer. Building in the wrong order (Block 3 as Block 9) would cause circular dependencies and force architectural rework. The correct sequence ensures each layer builds on stable foundations.
