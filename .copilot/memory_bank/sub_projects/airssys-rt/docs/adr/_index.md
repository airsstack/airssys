# airssys-rt Architecture Decision Records Index

**Sub-Project:** airssys-rt  
**Last Updated:** 2025-10-14  
**Total ADRs:** 9  
**Active ADRs:** 9  

## Active ADRs

### ADR-RT-001: Zero-Cost Abstractions and Generic Constraints
**Status**: Accepted | **Date**: 2025-10-04  
**File**: [adr_rt_001_zero_cost_abstractions.md](./adr_rt_001_zero_cost_abstractions.md)

Establishes zero-cost abstractions as the architectural foundation for airssys-rt through generic constraints and compile-time polymorphism instead of runtime trait objects.

**Key Decisions**:
- Use generic constraints (`T: Message`) over `dyn` trait objects
- Compile-time type resolution for zero runtime overhead
- Static dispatch for all message passing operations

**Impact**: All message and actor types use generics, no `Box<dyn Trait>` patterns

---

### ADR-RT-002: Message Passing Architecture
**Status**: Accepted | **Date**: 2025-10-04  
**File**: [adr_rt_002_message_passing_architecture.md](./adr_rt_002_message_passing_architecture.md)

Defines the message passing system architecture using type-safe envelopes and zero-cost generic constraints.

**Key Decisions**:
- Generic `MessageEnvelope<M: Message>` wrapper for all messages
- Type-safe message routing with compile-time guarantees
- Metadata support (priority, TTL, correlation, sender, reply-to)

**Impact**: All actor communication uses strongly-typed message envelopes

---

### ADR-RT-003: Backpressure Strategy Simplification
**Status**: Accepted | **Date**: 2025-10-05  
**File**: [adr_rt_003_backpressure_strategy_simplification.md](./adr_rt_003_backpressure_strategy_simplification.md)

Simplifies backpressure strategies from four to three by removing misleading `DropOldest` and `DropNewest` variants that had identical behavior due to tokio mpsc limitations.

**Key Decisions**:
- Simplified to `Block`, `Drop`, and `Error` strategies
- Honest API that accurately reflects implementation capabilities
- YAGNI-compliant design (§6.1)

**Impact**: Clearer backpressure API, no false promises about drop-oldest semantics

---

### ADR-RT-004: Child Trait Separation from Actor Trait ⚠️ **NEW**
**Status**: Accepted | **Date**: 2025-10-07  
**File**: [adr_rt_004_child_trait_separation.md](./adr_rt_004_child_trait_separation.md)

**CRITICAL ARCHITECTURE DECISION**: Establishes separate `Child` trait independent from `Actor` trait for supervision framework, with blanket implementation bridge providing automatic Child implementation for all actors.

**Key Decisions**:
- Separate `Child` trait with `start()`, `stop()`, `health_check()` methods
- Blanket impl `impl<A: Actor> Child for A` provides automatic bridge
- Child trait enables supervision of ANY entity (actors, tasks, I/O handlers, services)
- Zero breaking changes to existing Actor implementations
- BEAM/OTP alignment - supervisors manage processes, not just actors

**Impact**:
- RT-TASK-007 Phase 1 implementation pattern
- Enables heterogeneous supervision trees (actors + non-actors)
- Future-proof for WASM components, OSL services integration
- Zero performance overhead via static dispatch and monomorphization
- All existing actors automatically supervisable without code changes

---

### ADR-RT-006: MessageBroker Pub-Sub Architecture
**Status**: Accepted | **Date**: 2025-10-06  
**File**: [adr_006_messagebroker_pubsub_architecture.md](./adr_006_messagebroker_pubsub_architecture.md)

**CRITICAL ARCHITECTURE DECISION**: Establishes MessageBroker as a true pub-sub message bus instead of a direct routing system, enabling proper separation of concerns and extensibility.

**Key Decisions**:
- MessageBroker trait provides `publish()` and `subscribe()` methods
- ActorSystem subscribes to broker and routes messages via ActorRegistry
- Clear separation: Broker = transport, Registry = routing, System = orchestration
- Dependency Injection pattern for broker in ActorSystem
- Enables multiple subscribers (monitoring, audit, dead letters)

**Impact**: 
- Changes MessageBroker trait API (RT-TASK-004 modification)
- ActorSystem implementation pattern (RT-TASK-006)
- Enables distributed brokers (Redis, NATS) without changing actors
- Natural extensibility hooks for logging, metrics, persistence

---

### ADR-RT-007: Hierarchical Supervisor Architecture for OSL Integration
**Status**: Accepted | **Date**: 2025-10-11  
**File**: [adr_rt_007_hierarchical_supervisor_architecture.md](./adr_rt_007_hierarchical_supervisor_architecture.md)

**CRITICAL ARCHITECTURE DECISION**: Establishes service-oriented architecture with dedicated `OSLSupervisor` managing specialized OSL integration actors, providing clean separation between infrastructure and application concerns.

**Key Decisions**:
- Hierarchical supervisor structure: RootSupervisor → OSLSupervisor + ApplicationSupervisor
- Three core OSL actors: FileSystemActor, ProcessActor, NetworkActor
- Message-based communication across supervisor boundaries (no special handling required)
- Independent failure domains with supervisor-level fault isolation
- YAGNI-compliant: Focus on in-memory actors, defer process group management

**Impact**:
- RT-TASK-009 implementation pattern (OSL Integration)
- Clean fault isolation: OSL failures don't cascade to application actors
- Superior testability: Mock OSL actors in tests
- Centralized management: Single source of truth for OS operations
- Performance opportunities: Connection pooling, request batching, rate limiting
- Future extensibility: Easy to add DatabaseActor, CryptoActor, etc.

**Related Knowledge Docs**:
- KNOWLEDGE-RT-016: Process Group Management - Future Considerations (deferred features)
- KNOWLEDGE-RT-017: OSL Integration Actors Pattern (recommended implementation)

---

### ADR-RT-008: OSL Message Wrapper Pattern for Cloneable Messages
**Status**: Accepted | **Date**: 2025-10-14  
**File**: [adr_rt_008_osl_message_wrapper_pattern.md](./adr_rt_008_osl_message_wrapper_pattern.md)  
**Supersedes**: Initial OSL actor design with oneshot channels

Resolves the incompatibility between OSL request-response pattern (oneshot channels) and Actor trait requirements (`Clone` messages) by redesigning OSL messages as cloneable wrappers with broker-based response routing.

**Key Decisions**:
- OSL messages split into cloneable types: `*Operation`, `*Request`, `*Response`
- Request-response correlation via `MessageId` + `reply_to: ActorAddress`
- Responses sent through `MessageBroker.publish()` instead of oneshot channels
- OSL actors implement standard `Actor` + `Child` traits (no custom traits)
- Full integration with existing ActorSystem and MessageBroker infrastructure

**Impact**:
- OSL actors fully integrated with actor system (message passing + supervision)
- Request-response requires two message passes (request → response via broker)
- Application actors must handle response messages and track pending requests
- Pattern aligns with Erlang's `gen_server:call`, Akka's `ask` pattern
- No fragmentation of actor system (no custom OslActor trait needed)

**Related ADR**:
- ADR-RT-007: Hierarchical Supervisor Architecture (defines OSL actor structure)

---

### ADR-RT-009: OSL Broker Dependency Injection ⭐ **NEW**
**Status**: Accepted | **Date**: 2025-10-14  
**File**: [adr_rt_009_osl_broker_injection.md](./adr_rt_009_osl_broker_injection.md)  
**Related**: ADR-RT-001 (Zero-Cost Abstractions), ADR-RT-002 (Message Passing), ADR-RT-007 (OSL Architecture)

Addresses critical architectural gap discovered during RT-TASK-009 Phase 2: OSL actors had no MessageBroker integration, preventing message-based communication. Refactors OSLSupervisor and OSL actors to accept broker injection via generic constraints.

**Key Decisions**:
- OSL actors generic over broker: `FileSystemActor<M, B: MessageBroker<M>>`
- OSLSupervisor generic over broker: `OSLSupervisor<M, B>`
- Broker injected via constructor, cloned to child actors
- Factory closures capture broker for SupervisorNode ChildSpec
- Follows Microsoft Rust Guidelines M-DI-HIERARCHY (trait bounds over concrete types)

**Impact**:
- **Testability**: Can inject mock brokers for comprehensive testing
- **Flexibility**: Support different broker implementations (in-memory, distributed)
- **Standards Compliance**: Aligns with Microsoft Rust Guidelines
- **Generic Complexity**: OSLSupervisor becomes generic, types propagate upward
- **Breaking Change**: Existing OSL code requires broker injection parameter

**Architecture Pattern**:
```rust
// Production
let broker = InMemoryMessageBroker::new();
let osl_supervisor = OSLSupervisor::new(broker);

// Testing
let mock_broker = MockBroker::new();
let osl_supervisor = OSLSupervisor::new(mock_broker);
```

---

## Planned ADR Categories

### Actor System Architecture (Remaining)
- **ADR-RT-005: Actor State Management** - State storage and access patterns (planned)

### Performance and Concurrency  
- **ADR-005: Async Runtime Selection** - Tokio integration and configuration
- **ADR-006: Message Serialization Strategy** - Zero-copy vs traditional serialization
- **ADR-007: Concurrency Model** - Actor scheduling and execution model
- **ADR-008: Resource Management** - Memory and CPU resource optimization

### Integration Decisions
- **ADR-009: airssys-osl Integration** - OS layer integration patterns
- **ADR-010: Monitoring Strategy** - Metrics, tracing, and observability
- **ADR-011: Testing Strategy** - Actor system testing and validation
- **ADR-012: airssys-wasm Integration** - WASM component integration (future)

## Decision Priority

### Completed (Foundation)
1. ✅ **ADR-RT-001**: Actor Model Implementation Strategy (Zero-cost abstractions)
2. ✅ **ADR-RT-002**: Message Passing Architecture (Hybrid routing with type safety)
3. ✅ **ADR-RT-003**: Backpressure Strategy Simplification (YAGNI compliance)
4. ✅ **ADR-RT-004**: Child Trait Separation (Supervision lifecycle architecture)
5. ✅ **ADR-RT-006**: MessageBroker Pub-Sub Architecture (True pub-sub message bus)
6. ✅ **ADR-RT-007**: Hierarchical Supervisor Architecture for OSL Integration
7. ✅ **ADR-RT-008**: OSL Message Wrapper Pattern (Cloneable messages for Actor integration)

### Critical Path (Required Before Implementation)
1. **ADR-RT-005**: Async Runtime Selection
2. **ADR-RT-004**: Supervisor Tree Design

### Implementation Phase
1. **ADR-RT-003**: Actor State Management
2. **ADR-RT-006**: Message Serialization Strategy
3. **ADR-RT-009**: airssys-osl Integration
4. **ADR-RT-011**: Testing Strategy

## Decision Cross-References

### Knowledge Documentation
- **KNOWLEDGE-RT-001**: Zero-Cost Actor Model Architecture
- **KNOWLEDGE-RT-002**: Message Broker Zero-Copy Patterns  
- **KNOWLEDGE-RT-003**: Supervisor Tree Implementation Strategies

### Task Dependencies
- **RT-TASK-001**: Foundation Setup - implements ADR-RT-001
- **RT-TASK-002**: Message System - implements ADR-RT-002
- **RT-TASK-007**: Supervisor Framework - will implement ADR-RT-004

---
**Note:** Additional ADRs will be created as architectural decisions are needed during implementation.