# airssys-rt Knowledge Documentation Index

**Sub-Project:** airssys-rt  
**Last Updated:** 2025-10-05  
**Total Knowledge Docs:** 6  
**Active Knowledge Docs:** 6  

## Active Knowledge Documentation

### Actor Model Category
- **[KNOWLEDGE-RT-001](knowledge_rt_001_zero_cost_actor_architecture.md)**: Zero-Cost Actor Model Architecture
  - *Status*: active | *Created*: 2025-10-02
  - *Summary*: Comprehensive guide to zero-cost abstractions, generic constraints, and compile-time optimizations for actor model implementation
  
- **[KNOWLEDGE-RT-003](knowledge_rt_003_supervisor_tree_strategies.md)**: Supervisor Tree Implementation Strategies  
  - *Status*: active | *Created*: 2025-10-02
  - *Summary*: BEAM-inspired supervisor trees, fault tolerance patterns, restart strategies, and hierarchical supervision design

### Performance Category
- **[KNOWLEDGE-RT-002](knowledge_rt_002_message_broker_zero_copy.md)**: Message Broker Zero-Copy Patterns
  - *Status*: active | *Created*: 2025-10-02
  - *Summary*: High-performance message routing, zero-copy delivery, lock-free data structures, and memory pool optimization

### Patterns Category
- **[KNOWLEDGE-RT-004](knowledge_rt_004_message_system_implementation_guide.md)**: Message System Implementation Guide
  - *Status*: active | *Created*: 2025-10-04
  - *Summary*: Complete RT-TASK-001 implementation guide with code examples, workspace standards compliance, and step-by-step instructions

- **[KNOWLEDGE-RT-005](knowledge_rt_005_actor_system_core_implementation_guide.md)**: Actor System Core Implementation Guide
  - *Status*: active | *Created*: 2025-10-04
  - *Summary*: Complete RT-TASK-002 implementation guide for Actor trait, ActorContext, and lifecycle management with zero-cost abstractions

- **[KNOWLEDGE-RT-006](knowledge_rt_006_mailbox_system_implementation_guide.md)**: Mailbox System Implementation Guide
  - *Status*: active | *Created*: 2025-10-05
  - *Summary*: Complete RT-TASK-003 implementation guide for generic Mailbox trait, bounded/unbounded mailboxes, and backpressure strategies

## Planned Knowledge Documentation

### Actor Model Category (Remaining)
- **Actor Lifecycle Patterns**: Creation, execution, and termination patterns
- **State Management**: Actor state storage and access patterns
- **Actor Pool Management**: Dynamic actor pool scaling and load balancing

### Performance Category (Remaining)
- **Concurrency Optimization**: High-performance actor scheduling and execution
- **Memory Management**: Efficient memory usage with thousands of actors
- **Resource Pooling**: Actor and message pooling strategies

### Integration Category
- **airssys-osl Integration**: OS layer integration patterns and best practices
- **airssys-wasm Integration**: WASM component hosting and management (future)
- **Monitoring Integration**: Metrics, tracing, and observability patterns
- **Testing Patterns**: Actor system testing and fault injection strategies

## Knowledge Cross-References

### Architecture Decision Records
- **ADR-RT-001**: Actor Model Implementation Strategy
- **ADR-RT-002**: Message Passing Architecture
- **ADR-RT-004**: Supervisor Tree Design (planned)

### Task Dependencies
- **RT-TASK-001**: Message System Implementation - implements KNOWLEDGE-RT-001 and KNOWLEDGE-RT-004 patterns
- **RT-TASK-002**: Actor System Core - implements KNOWLEDGE-RT-001 and KNOWLEDGE-RT-005 patterns
- **RT-TASK-003**: Mailbox System - implements KNOWLEDGE-RT-001 and KNOWLEDGE-RT-006 patterns
- **RT-TASK-007**: Supervisor Framework - implements KNOWLEDGE-RT-003 patterns

---
**Note:** Additional knowledge docs will be created during implementation phases to capture emerging patterns and optimizations.