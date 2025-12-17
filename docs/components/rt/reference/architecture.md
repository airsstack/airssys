# Architecture Reference

Technical specifications for AirsSys RT system architecture - authoritative documentation of architectural components, interactions, and design decisions.

---

## Overview

This section provides detailed architectural documentation for the AirsSys RT actor system. Each page follows Reference documentation principles from the Diátaxis framework: neutral descriptions of system design, component interactions, and architectural patterns.

**Organization:** Architecture documentation is organized by architectural concern (system overview, components, message passing, supervision, lifecycle) to provide different perspectives on the system design.

---

## System Overview

**[System Overview](./architecture/system-overview.md)**

High-level architectural overview of the entire actor system:
- System architecture diagram
- Component relationships and dependencies
- Data flow through the system
- Key architectural decisions and rationale
- System boundaries and external integrations

**When to use:** Understanding the big picture of how AirsSys RT components work together.

---

## Components

**[Components](./architecture/components.md)**

Detailed component architecture and interactions:
- Core components: Actor, ActorContext, Mailbox, MessageBroker, Supervisor
- Component responsibilities and interfaces
- Inter-component communication patterns
- Dependency injection architecture (ADR-006)
- Component lifecycle management

**When to use:** Understanding individual components and how they interact.

---

## Message Passing

**[Message Passing Architecture](./architecture/message-passing.md)**

Message routing and communication architecture:
- Direct messaging (actor references)
- Broker-based messaging (pub-sub)
- Routing mechanisms and performance
- Message delivery guarantees
- Backpressure and flow control

**When to use:** Understanding message flow and routing decisions.

---

## Supervision

**[Supervision Architecture](./architecture/supervision.md)**

Fault tolerance and supervision tree architecture:
- Supervision tree structure and hierarchy
- Restart strategy implementations
- Failure detection and recovery mechanisms
- Restart rate limiting and escalation
- Supervisor-child communication protocols

**When to use:** Understanding fault tolerance architecture and supervision mechanisms.

---

## Process Lifecycle

**[Process Lifecycle](./architecture/process-lifecycle.md)**

Actor lifecycle state machine and transitions:
- Actor lifecycle states: Created, Starting, Running, Stopping, Stopped, Restarting, Failed
- State transition rules and triggers
- Lifecycle hook execution order
- Resource management and cleanup
- Error handling during lifecycle transitions

**When to use:** Understanding actor lifecycle management and state transitions.

---

## Architectural Principles

AirsSys RT architecture follows these core principles:

### 1. Actor Model Foundations

**Encapsulation and Isolation:**

- Actors own their state (no shared mutable state)
- Message passing for all communication
- Location transparency (local and remote actors use same API)

**Sequential Message Processing:**

- One message at a time (no internal concurrency)
- FIFO message ordering guaranteed
- Deterministic message handling

### 2. Erlang/OTP-Inspired Supervision

**"Let it Crash" Philosophy:**

- Simple happy path logic in actors
- Supervisors handle failure recovery
- Clear separation: actors do work, supervisors provide fault tolerance

**Supervision Trees:**

- Hierarchical fault isolation
- Different strategies for different failure scenarios
- Automatic restart with configurable limits

### 3. Zero-Cost Abstractions

**Performance Without Compromise:**

- Generic compilation (monomorphization)
- No runtime type dispatch on hot paths
- Direct memory access (no unnecessary indirection)
- Inline message processing

**Benchmarked Performance:**

- Actor spawn: 624ns (1.6M actors/sec)
- Message passing: 737ns roundtrip (1.36M msgs/sec)
- Supervision overhead: ~2x base actor (acceptable for fault tolerance)

### 4. Type Safety

**Compile-Time Guarantees:**

- Associated types for messages and errors
- Generic constraints for trait bounds
- Type-state pattern (future) for lifecycle validation

**No Runtime Surprises:**

- Explicit error types (`Result<T, E>`)
- No `Any` or type erasure on hot paths
- Clear message protocols with associated `Result` types

### 5. Dependency Injection

**Testability and Flexibility (ADR-006):**

- `MessageBroker` trait injected into `ActorContext`
- Swap implementations (in-memory, distributed, test doubles)
- No global state or singletons

**Benefits:**

- Unit test actors in isolation
- Different brokers for different deployment scenarios
- Clear component boundaries

---

## Architectural Layers

```
┌─────────────────────────────────────────────────────────┐
│                   Application Layer                      │
│  (User Actors, Supervisor Trees, Business Logic)        │
└─────────────────────────────────────────────────────────┘
                          ▼
┌─────────────────────────────────────────────────────────┐
│                   Actor Framework Layer                  │
│  (Actor Trait, Supervision, Lifecycle Management)       │
└─────────────────────────────────────────────────────────┘
                          ▼
┌─────────────────────────────────────────────────────────┐
│                   Messaging Layer                        │
│  (MessageBroker, Mailbox, Routing)                      │
└─────────────────────────────────────────────────────────┘
                          ▼
┌─────────────────────────────────────────────────────────┐
│                   Runtime Layer                          │
│  (Tokio Runtime, Async Execution)                       │
└─────────────────────────────────────────────────────────┘
```

### Layer Responsibilities

**Application Layer:**

- Business logic implementation
- Actor and supervisor instantiation
- Message protocol definition
- Application-specific error handling

**Actor Framework Layer:**

- Actor lifecycle management
- Supervision tree construction
- Restart strategy implementation
- Actor context and metadata

**Messaging Layer:**

- Message routing (direct and broker-based)
- Mailbox management and backpressure
- Pub-sub topic management
- Message delivery guarantees

**Runtime Layer:**

- Async task scheduling (Tokio)
- Concurrency primitives (channels, tasks)
- I/O event loop
- System resource management

---

## Cross-Cutting Concerns

### Monitoring and Observability

**Built-in Metrics:**

- Actor message counts
- Supervisor restart counts
- Mailbox depth tracking
- Health check API

**Integration Points:**

- Custom metrics via `ActorContext`
- Supervision events for monitoring
- Health check endpoints

### Error Handling

**Layered Error Strategy:**

- Actor-level: `Result<T, Self::Error>`
- Supervisor-level: `ErrorAction` (Resume, Restart, Stop, Escalate)
- System-level: Supervisor escalation and shutdown

**Error Propagation:**
```
Actor Error → on_error() → ErrorAction → Supervisor → Restart Strategy
```

### Performance Optimization

**Hot Path Optimizations:**

- Inline message processing (31.55ns)
- Direct mailbox access (no indirection)
- Monomorphized generics (no dynamic dispatch)
- Bounded mailboxes for predictable performance

**Cold Path Acceptable Costs:**

- Actor spawn: 624ns (infrequent)
- Supervision overhead: +656ns (fault tolerance worth it)
- Broker routing: +180ns (flexibility worth it)

---

## Deployment Patterns

### Single Process

**Architecture:**

- All actors in one process
- In-memory message passing
- Shared Tokio runtime

**Use Cases:**

- Development and testing
- Single-machine deployments
- Embedded systems

### Distributed (Future)

**Architecture:**

- Actors across multiple processes/machines
- Network-based message passing
- Location transparency maintained

**Use Cases:**

- Horizontal scaling
- Geographic distribution
- High availability

---

## Design Tradeoffs

### Type Safety vs. Flexibility

**Decision:** Favor type safety
- Associated types for messages/errors
- Generic constraints over trait objects
- **Tradeoff:** Less runtime flexibility, more compile-time safety

### Performance vs. Features

**Decision:** Measure first, optimize hot paths
- Benchmarked all core operations
- Zero-cost abstractions on hot paths
- **Tradeoff:** Complexity in implementation, predictable performance

### Simplicity vs. Power

**Decision:** Start simple, add complexity when needed (YAGNI)
- Basic actor model first
- Supervision added incrementally
- **Tradeoff:** May miss some features initially, cleaner codebase

---

## Architectural Evolution

### Current State (v0.1)
- ✅ Core actor model
- ✅ Supervision trees
- ✅ In-memory messaging
- ✅ Basic monitoring

### Planned (v0.2+)
- 🔄 Distributed actors
- 🔄 Persistence
- 🔄 Advanced routing
- 🔄 Cluster management

---

## Further Reading

- **[System Overview](./architecture/system-overview.md)** - Detailed system architecture
- **[Components](./architecture/components.md)** - Component interactions
- **[API Reference](./api.md)** - API specifications
- **[Performance Reference](./performance.md)** - Benchmarking data
- **[Explanation: Actor Model](../explanation/actor-model.md)** - Understanding the actor model
- **[Explanation: Supervision](../explanation/supervision.md)** - Understanding supervision philosophy
