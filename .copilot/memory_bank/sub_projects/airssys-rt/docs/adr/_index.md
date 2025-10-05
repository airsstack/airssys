# airssys-rt Architecture Decision Records Index

**Sub-Project:** airssys-rt  
**Last Updated:** 2025-10-02  
**Total ADRs:** 2  
**Active ADRs:** 2  

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

## Planned ADR Categories

### Actor System Architecture (Remaining)
- **ADR-003: Actor State Management** - State storage and access patterns
- **ADR-004: Supervisor Tree Design** - Supervision strategies and implementation

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