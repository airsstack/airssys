# [WASM-TASK-004] - Block 3: Actor System Integration ⭐ FOUNDATIONAL

**Status:** not-started  
**Added:** 2025-10-20  
**Updated:** 2025-10-20  
**Priority:** CRITICAL PATH - Foundation Layer  
**Layer:** 1 - Foundation  
**Block:** 3 of 11  
**Estimated Effort:** 4-5 weeks  

## Overview

**⭐ CRITICAL FOUNDATIONAL BLOCK**

Integrate WASM component execution with the airssys-rt actor system, establishing the core architectural pattern of "actor-hosted WASM components from the start". This block implements ComponentActor (Actor + Child dual trait), integrates ActorSystem spawning, SupervisorNode supervision, and MessageBroker routing to create the fundamental isolation and communication infrastructure that ALL subsequent blocks depend on.

## Context

**Current State:**
- Architecture complete: ADR-WASM-006 (Component Isolation and Sandboxing - Actor-based approach)
- Actor system ready: airssys-rt foundation phase complete (100% COMPLETE)
- Performance proven: ~625ns actor spawn, ~211ns MessageBroker routing, 10,000+ concurrent actors
- Integration requirements: ComponentActor, SupervisorNode, MessageBroker, ActorSystem::spawn()

**Critical Architectural Correction (ADR-WASM-010):**
Initial planning placed Actor System Integration as Block 9 (integration layer). Architectural review revealed it MUST be Block 3 (foundation layer):

**Why This Is Foundational, Not Integration:**
1. **MessageBroker is Core Infrastructure** - Block 5 (Inter-Component Communication) depends on MessageBroker
2. **ComponentActor Pattern is Fundamental** - All components ARE actors from the start
3. **Supervision is Not Optional** - Block 7 (Component Lifecycle) requires SupervisorNode for health management
4. **Isolation Model Requires Actors** - ADR-WASM-006 explicitly uses Erlang-style lightweight processes

**The Correct Mental Model:**
- ❌ **Wrong**: "WASM components, then integrate actors later"
- ✅ **Right**: "Actor-hosted WASM components from the start"

This is analogous to:
- You can't build async code without Tokio
- You can't build a web server without an HTTP library
- **You can't build airssys-wasm without airssys-rt**

**Problem Statement:**
The framework needs to:
1. Host each WASM component instance within its own ComponentActor
2. Implement Actor trait for message handling and Child trait for WASM lifecycle
3. Integrate ActorSystem::spawn() for all component instantiation (NOT manual tokio::spawn)
4. Connect SupervisorNode for automatic component restart on crashes
5. Route inter-component messages through MessageBroker
6. Achieve target performance: ~2-11ms component spawn (includes WASM loading)

**Why This Block Matters:**
Without this block:
- Inter-component communication (Block 5) cannot be implemented
- Component lifecycle management (Block 7) has no supervision system
- Component isolation (Block 4) has no actor boundaries
- Monitoring (Block 9) has no health status to observe

**This block MUST complete before Layer 2 (Blocks 4-7) can begin.**

## Objectives

### Primary Objective
Integrate WASM component execution with airssys-rt actor system by implementing ComponentActor (Actor + Child), ActorSystem spawning, SupervisorNode supervision, and MessageBroker routing to establish the foundational actor-hosted component architecture.

### Secondary Objectives
- Achieve target performance: <5ms average component spawn (including WASM load)
- Implement automatic component restart on crashes via SupervisorNode
- Route all inter-component messages through MessageBroker
- Establish ComponentActor as the standard isolation unit
- Document actor-based component patterns

## Scope

### In Scope
1. **ComponentActor Implementation** - Dual trait (Actor + Child) structure
2. **WASM Lifecycle Integration** - Child::start() loads WASM, Child::stop() cleans up
3. **ActorSystem Spawning** - ActorSystem::spawn() integration for component instantiation
4. **SupervisorNode Integration** - Component supervision with automatic restart
5. **MessageBroker Integration** - Pub-sub routing for inter-component messages
6. **Message Handling** - Actor::handle_message() implementation for component messages
7. **Performance Optimization** - Achieve spawn and routing performance targets
8. **Testing Framework** - Actor-based component testing utilities

### Out of Scope
- Capability-based security enforcement (Block 4)
- Full inter-component messaging patterns (Block 5)
- Persistent storage integration (Block 6)
- Component installation/updates (Block 7)
- Monitoring and metrics collection (Block 9)

## Implementation Plan

### Phase 1: ComponentActor Foundation (Week 1-2)

#### Task 1.1: ComponentActor Struct Design
**Deliverables:**
- `ComponentActor` struct definition
- Actor trait implementation stub
- Child trait implementation stub
- WASM instance storage design
- Message queue integration
- ComponentActor documentation

**Success Criteria:**
- ComponentActor implements both Actor and Child
- Struct design reviewed and approved
- Traits compile successfully
- Clear ownership model for WASM instance

#### Task 1.2: Child Trait WASM Lifecycle
**Deliverables:**
- Child::start() implementation (loads WASM from Block 1 runtime)
- Child::stop() implementation (cleanup WASM instance)
- WASM instance initialization in start()
- Resource cleanup in stop()
- Lifecycle error handling
- Lifecycle testing

**Success Criteria:**
- Child::start() successfully loads WASM components
- Child::stop() cleans up all resources
- Supervisor can control lifecycle via Child trait
- No resource leaks on component shutdown

#### Task 1.3: Actor Trait Message Handling
**Deliverables:**
- Actor::handle_message() implementation
- Message type definitions
- Message deserialization (multicodec support)
- WASM message dispatch (call component's handle-message export)
- Message handling error propagation
- Message handling tests

**Success Criteria:**
- ComponentActor receives messages via mailbox
- Messages dispatched to WASM handle-message function
- Errors handled gracefully
- Message throughput meets targets (>10,000/sec)

---

### Phase 2: ActorSystem Integration (Week 2-3)

#### Task 2.1: ActorSystem::spawn() Integration
**Deliverables:**
- Component spawning via ActorSystem::spawn()
- ComponentActor registration with ActorSystem
- Actor address (ActorRef) management
- Component instance tracking
- Spawn performance optimization
- Spawning tests

**Success Criteria:**
- Components spawn via ActorSystem (NOT tokio::spawn)
- ActorRef returned for message sending
- Component instances tracked by ActorSystem
- Spawn time <5ms average (including WASM load)

#### Task 2.2: Component Instance Management
**Deliverables:**
- Component ID to ActorRef mapping
- Component instance registry
- Instance lookup by ID
- Instance lifecycle tracking
- Registry documentation

**Success Criteria:**
- Component instances addressable by ID
- Registry provides O(1) lookup
- Instance lifecycle visible
- Clear registry API

#### Task 2.3: Actor Address and Routing
**Deliverables:**
- ActorRef wrapper for component addressing
- Message routing via ActorRef.send()
- Asynchronous message delivery
- Routing error handling
- Routing performance tests

**Success Criteria:**
- Messages route to correct ComponentActor
- Routing latency <500ns (airssys-rt proven)
- Failed routing handled gracefully
- Routing performance documented

---

### Phase 3: SupervisorNode Integration (Week 3-4)

#### Task 3.1: Supervisor Tree Setup
**Deliverables:**
- SupervisorNode for component management
- Restart policy configuration
- Component supervision tree design
- Supervisor strategy implementation
- Supervision documentation

**Success Criteria:**
- Components supervised by SupervisorNode
- Restart policies configurable
- Supervision tree hierarchical
- Clear supervision patterns

#### Task 3.2: Automatic Component Restart
**Deliverables:**
- Crash detection and restart
- Restart backoff strategies
- Max restart limits
- Restart state handling
- Restart testing (crash scenarios)

**Success Criteria:**
- Component crashes trigger restart
- Backoff prevents restart storms
- Max restart limits enforced
- Clean state on restart

#### Task 3.3: Component Health Monitoring
**Deliverables:**
- Health check integration (Child trait)
- Health status reporting
- Failed health check handling
- Health monitoring configuration
- Health monitoring tests

**Success Criteria:**
- Components report health status
- Failed health checks trigger restart
- Health status queryable
- Clear health semantics

---

### Phase 4: MessageBroker Integration (Week 4-5)

#### Task 4.1: MessageBroker Setup for Components
**Deliverables:**
- MessageBroker instance for components
- Topic subscription management
- Component subscription registration
- Broker configuration
- Broker integration documentation

**Success Criteria:**
- MessageBroker routes component messages
- Components can subscribe to topics
- Topic-based message delivery works
- Routing performance: ~211ns (airssys-rt proven)

#### Task 4.2: Pub-Sub Message Routing
**Deliverables:**
- Component message publishing
- Topic-based message filtering
- Multiple subscriber handling
- Message delivery guarantees
- Pub-sub pattern tests

**Success Criteria:**
- Components publish to topics
- Messages delivered to all subscribers
- Topic filtering works correctly
- Delivery semantics clear

#### Task 4.3: ActorSystem as Primary Subscriber Pattern
**Deliverables:**
- ActorSystem subscribes to all component messages
- Routing decisions by ActorSystem
- ComponentActor mailbox delivery
- Unified message routing architecture
- Pattern documentation

**Success Criteria:**
- ActorSystem is primary subscriber
- Messages route through ActorSystem to mailboxes
- Routing logic centralized
- Pattern clear and documented

---

### Phase 5: Performance Optimization (Week 5)

#### Task 5.1: Component Spawn Optimization
**Deliverables:**
- Spawn time profiling
- WASM loading optimization
- Actor initialization optimization
- Benchmark suite
- Optimization documentation

**Success Criteria:**
- Average spawn time <5ms
- P99 spawn time <10ms
- Bottlenecks identified and mitigated
- Performance documented

#### Task 5.2: Message Routing Performance
**Deliverables:**
- Message routing profiling
- MessageBroker routing optimization
- ActorRef routing optimization
- Throughput benchmarks
- Performance documentation

**Success Criteria:**
- Message routing <1ms end-to-end
- Throughput >10,000 messages/sec per component
- Routing overhead <10% of total message time
- Performance characteristics documented

#### Task 5.3: Memory and Resource Optimization
**Deliverables:**
- Memory usage profiling
- ComponentActor memory footprint optimization
- Resource cleanup verification
- Memory leak testing
- Resource usage documentation

**Success Criteria:**
- ComponentActor memory <2MB per instance
- No memory leaks detected
- Resource cleanup verified
- 10,000+ concurrent components achievable

---

### Phase 6: Testing and Integration Validation (Week 5)

#### Task 6.1: Integration Test Suite
**Deliverables:**
- End-to-end component lifecycle tests
- Message routing integration tests
- Supervisor restart integration tests
- Multi-component communication tests
- Integration test documentation

**Success Criteria:**
- Complete lifecycle tested (spawn → message → crash → restart)
- MessageBroker routing validated
- Supervisor behavior validated
- Multi-component scenarios work

#### Task 6.2: Performance Validation
**Deliverables:**
- Performance benchmark suite
- Spawn time validation
- Message throughput validation
- Concurrent component testing
- Performance validation documentation

**Success Criteria:**
- All performance targets met
- Benchmarks reproducible
- Concurrent scaling demonstrated
- Performance regression detection

#### Task 6.3: Actor-Based Component Testing Framework
**Deliverables:**
- Test utilities for ComponentActor
- Mock ActorSystem for testing
- Mock MessageBroker for testing
- Component test patterns
- Testing framework documentation

**Success Criteria:**
- Components testable in isolation
- Mock system supports unit tests
- Clear testing patterns established
- Examples demonstrate usage

---

## Success Criteria

### Definition of Done
This task is complete when:

1. ✅ **ComponentActor Implementation Complete**
   - Implements Actor + Child dual trait
   - WASM loads in Child::start(), cleans up in Child::stop()
   - Messages handled via Actor::handle_message()
   - Compiles and unit tests pass

2. ✅ **ActorSystem Integration Working**
   - Components spawn via ActorSystem::spawn()
   - ActorRef addressing functional
   - Component registry operational
   - Spawn time <5ms average

3. ✅ **SupervisorNode Integration Working**
   - Components supervised by SupervisorNode
   - Automatic restart on crashes functional
   - Health monitoring operational
   - Restart policies configurable

4. ✅ **MessageBroker Integration Working**
   - MessageBroker routes component messages
   - Pub-sub subscriptions functional
   - ActorSystem as primary subscriber pattern implemented
   - Routing performance: ~211ns (airssys-rt baseline)

5. ✅ **Performance Targets Met**
   - Component spawn: <5ms average, <10ms P99
   - Message routing: <1ms end-to-end
   - Throughput: >10,000 messages/sec per component
   - Concurrent components: 10,000+ achievable

6. ✅ **Testing & Documentation Complete**
   - Integration test suite passing (>90% coverage)
   - Performance benchmarks established
   - Actor-based testing framework operational
   - Complete documentation with patterns

7. ✅ **Layer 1 Foundation Ready**
   - Blocks 4-7 can begin implementation
   - Actor-hosted component pattern proven
   - Integration points clear
   - Performance validated

## Dependencies

### Upstream Dependencies
- ✅ WASM-TASK-002: WASM Runtime Layer (Block 1) - **REQUIRED** for WASM loading in Child::start()
- ✅ ADR-WASM-006: Component Isolation and Sandboxing - **COMPLETE**
- ✅ ADR-WASM-010: Implementation Strategy - **COMPLETE**
- ✅ airssys-rt foundation - **COMPLETE** (100% complete, proven performance)
- ✅ KNOWLEDGE-RT-013: Actor Performance Benchmarking Results - **COMPLETE**

### Downstream Dependencies (Blocks This Task)
**⭐ CRITICAL: These blocks CANNOT start until this block completes:**
- WASM-TASK-005: Security & Isolation (Block 4) - needs Actor isolation boundaries
- WASM-TASK-006: Inter-Component Communication (Block 5) - needs MessageBroker integration
- WASM-TASK-007: Persistent Storage (Block 6) - needs ComponentActor context
- WASM-TASK-008: Component Lifecycle (Block 7) - needs SupervisorNode for health
- WASM-TASK-010: Monitoring & Observability (Block 9) - needs health monitoring

### External Dependencies
- airssys-rt MessageBroker (InMemoryMessageBroker)
- airssys-rt Actor and Child traits
- airssys-rt ActorSystem::spawn()
- airssys-rt SupervisorNode
- Tokio async runtime (from airssys-rt)

### Sequential Constraint
**This block MUST complete BEFORE Layer 2 (Blocks 4-7) begins.**

## Risks and Mitigations

### Risk 1: Actor Pattern Complexity
**Impact:** High - Wrong actor design could require major refactoring  
**Probability:** Medium - First time integrating actors with WASM  
**Mitigation:**
- Follow ADR-WASM-006 actor design closely
- Reference airssys-rt patterns and examples
- Early prototyping of ComponentActor
- Code review by airssys-rt experts

### Risk 2: Performance Not Meeting Targets
**Impact:** High - Could make framework unusable at scale  
**Probability:** Medium - Adding WASM overhead to actor spawning  
**Mitigation:**
- Profile spawn path extensively
- Optimize WASM loading (module caching from Block 1)
- Benchmark continuously during development
- airssys-rt proven performance provides baseline

### Risk 3: Supervisor Complexity
**Impact:** Medium - Wrong supervision could cause instability  
**Probability:** Low - airssys-rt SupervisorNode is mature  
**Mitigation:**
- Use SupervisorNode patterns from airssys-rt
- Test restart scenarios extensively
- Implement conservative restart policies initially
- Document supervision behavior clearly

### Risk 4: MessageBroker Routing Issues
**Impact:** High - Broken routing breaks inter-component communication  
**Probability:** Low - MessageBroker proven in airssys-rt  
**Mitigation:**
- Follow MessageBroker usage patterns from airssys-rt
- Test pub-sub scenarios thoroughly
- Validate routing performance early
- airssys-rt provides proven routing (~211ns)

### Risk 5: Integration Timing with Block 1
**Impact:** Medium - Block 1 delays could block this work  
**Probability:** Low - Blocks can overlap partially  
**Mitigation:**
- Stub WASM runtime initially for actor testing
- Parallel development where possible
- Integration testing as Block 1 completes
- Clear interface contract between blocks

## Progress Tracking

**Overall Status:** not-started - 0%

### Phase Breakdown
| Phase | Description | Status | Estimated Duration | Notes |
|-------|-------------|--------|-------------------|-------|
| 1 | ComponentActor Foundation | not-started | Week 1-2 | Core actor design |
| 2 | ActorSystem Integration | not-started | Week 2-3 | Spawning and registry |
| 3 | SupervisorNode Integration | not-started | Week 3-4 | Supervision and health |
| 4 | MessageBroker Integration | not-started | Week 4-5 | Pub-sub routing |
| 5 | Performance Optimization | not-started | Week 5 | Performance targets |
| 6 | Testing and Integration Validation | not-started | Week 5 | Validation |

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 1.1 | ComponentActor Struct Design | not-started | - | Core foundation |
| 1.2 | Child Trait WASM Lifecycle | not-started | - | WASM integration |
| 1.3 | Actor Trait Message Handling | not-started | - | Message dispatch |
| 2.1 | ActorSystem::spawn() Integration | not-started | - | Spawning |
| 2.2 | Component Instance Management | not-started | - | Registry |
| 2.3 | Actor Address and Routing | not-started | - | Addressing |
| 3.1 | Supervisor Tree Setup | not-started | - | Supervision foundation |
| 3.2 | Automatic Component Restart | not-started | - | Restart logic |
| 3.3 | Component Health Monitoring | not-started | - | Health checks |
| 4.1 | MessageBroker Setup for Components | not-started | - | Broker integration |
| 4.2 | Pub-Sub Message Routing | not-started | - | Topic routing |
| 4.3 | ActorSystem as Primary Subscriber | not-started | - | Routing pattern |
| 5.1 | Component Spawn Optimization | not-started | - | Performance |
| 5.2 | Message Routing Performance | not-started | - | Throughput |
| 5.3 | Memory and Resource Optimization | not-started | - | Efficiency |
| 6.1 | Integration Test Suite | not-started | - | Validation |
| 6.2 | Performance Validation | not-started | - | Benchmarks |
| 6.3 | Actor-Based Component Testing Framework | not-started | - | Testing utils |

## Progress Log

### 2025-11-30 - Implementation Guide Created
**Added by:** AI Agent  
**Changes:**
- **NEW KNOWLEDGE DOC**: Created KNOWLEDGE-WASM-016 (Actor System Integration Implementation Guide)
- **Purpose**: Provide detailed implementation guidance beyond task definition
- **Content**: Code-level examples for all 6 phases and 18 subtasks
- **Details**:
  - ComponentActor struct design with full trait implementations
  - WASM lifecycle management (Child::start() and Child::stop())
  - Message handling patterns (multicodec support)
  - ActorSystem integration examples
  - Component registry implementation with O(1) lookup
  - SupervisorNode supervision patterns
  - MessageBroker integration
  - Testing strategies for each phase
  - Performance validation approach
  - Integration verification checklist
- **Effort**: Detailed per-task estimates (8-20 hours each)
- **Reference**: Complements task_004_block_3_actor_system_integration.md
- **Status**: Ready for implementation

## Related Documentation

### ⭐ Essential Reading (MUST READ BEFORE STARTING)
- **KNOWLEDGE-WASM-016: Actor System Integration Implementation Guide** - **CRITICAL IMPLEMENTATION REFERENCE**
  - Code-level examples for all 18 subtasks
  - Concrete patterns (ComponentActor struct, Child trait, Actor trait)
  - Per-task effort estimates (hours)
  - Testing strategies for each component
  - Performance validation approach
  - **READ THIS FIRST** for detailed implementation guidance

### ADRs
- **ADR-WASM-006: Component Isolation and Sandboxing (Revised)** - Primary reference for actor-based architecture
- **ADR-WASM-010: Implementation Strategy and Build Order** - Explains why Block 3 is foundational
- **ADR-WASM-009: Component Communication Model** - MessageBroker integration requirements
- **ADR-RT-004: Actor and Child Trait Separation** - Dual trait design pattern

### Knowledge Documentation
- **KNOWLEDGE-WASM-001: Component Framework Architecture** - Actor-hosted component vision
- **KNOWLEDGE-WASM-005: Inter-Component Messaging Architecture** - MessageBroker usage patterns
- **KNOWLEDGE-RT-013: Actor Performance Benchmarking Results** - Performance baseline (625ns spawn, 211ns routing)
- **KNOWLEDGE-RT-016: SupervisorNode Comprehensive Guide** - Supervision patterns

### airssys-rt References
- **RT-TASK-004: PubSub System Foundation** - MessageBroker implementation
- **RT-TASK-007: SupervisorNode Hierarchical Orchestration** - SupervisorNode implementation
- Actor trait and Child trait API documentation
- ActorSystem::spawn() API documentation

### External References
- [Erlang OTP Supervision Principles](https://www.erlang.org/doc/design_principles/sup_princ.html)
- Actor model design patterns
- Tokio async patterns

## Notes

**⭐ CRITICAL FOUNDATIONAL BLOCK:**
This is NOT an "integration layer" block. It's FOUNDATIONAL. All of Layer 2 (Blocks 4-7) depends on this.

**Mental Model Correction:**
- ❌ Wrong: "Build WASM components, then add actors for communication"
- ✅ Right: "Every component IS an actor from the start"

**Analogy:**
- Can't build web app without HTTP library
- Can't build async code without Tokio
- Can't build airssys-wasm without airssys-rt

**Performance Baseline:**
airssys-rt has proven performance:
- Actor spawn: ~625ns
- MessageBroker routing: ~211ns
- 10,000+ concurrent actors

Adding WASM overhead (2-10ms) gives target: <5ms average spawn, <10ms P99.

**Dual Trait Pattern:**
ComponentActor implements BOTH:
- Actor: for message handling (mailbox pattern)
- Child: for WASM lifecycle (supervisor-controlled)

This is NOT optional. It's the core pattern.

**ActorSystem::spawn() Requirement:**
ALL component instantiation MUST use ActorSystem::spawn(), NOT tokio::spawn().
This provides proper actor registration and supervision.

**SupervisorNode Integration:**
SupervisorNode is NOT optional. All components MUST be supervised for production reliability.

**MessageBroker as Communication Backbone:**
Block 5 (Inter-Component Communication) builds on MessageBroker foundation established here.

**Layer 1 Gate:**
This block's completion is the Layer 1 validation gate. Blocks 4-7 CANNOT proceed until this passes.

**Testing Strategy:**
Early prototyping critical. Test actor patterns before full WASM integration. Mock WASM runtime if Block 1 delays.

**Code Review Requirement:**
This block MUST have code review by airssys-rt experts to ensure correct actor usage patterns.
