# Future Use Cases

This document explores conceptual application patterns and future capabilities planned for `airssys-rt`. These examples illustrate the design vision and potential use cases that will be enabled as the runtime matures.

> [!IMPORTANT]
> **Implementation Status: Planned**
> 
> The APIs and patterns shown here are **conceptual designs** representing planned functionality. They are not currently implemented and should not be used in production code.
> 
> For working examples with current APIs, see:
> - [Getting Started Guide](../implementation/getting-started.md)
> - [OSL Integration Example](../implementation/osl-integration.md)
> - [Working Examples Directory](https://github.com/airsstack/airssys/tree/main/airssys-rt/examples)

## Design Vision

The `airssys-rt` runtime is designed to support high-level application patterns through its core actor model primitives. The following examples illustrate planned ergonomic APIs that will be built on top of the foundational components completed in Phases 1-2.

## Planned Use Case Patterns

### 1. High-Concurrency Server Pattern (Planned)

**Concept**: Leveraging actor isolation and supervision for handling thousands of concurrent network connections with fault tolerance.

**Planned API Design**:
```rust
// Conceptual API - NOT YET IMPLEMENTED
use airssys_rt::patterns::TcpServer;
use airssys_rt::supervisor::RestartStrategy;

let server = TcpServer::new()
    .with_connection_actor(ConnectionActor::new())
    .with_supervisor(connection_supervisor)
    .bind("0.0.0.0:8080").await?;
```

**Design Rationale**:
- Each connection managed by isolated actor (fault containment)
- Supervisor restarts failed connection handlers
- Backpressure through actor mailbox capacity
- Zero shared mutable state between connections

**Current Foundation**:
```rust
// What works today: Generic supervisor with broker-based actors
use airssys_rt::broker::InMemoryMessageBroker;
use airssys_rt::osl::OSLSupervisor;

let broker = InMemoryMessageBroker::new();
let supervisor = OSLSupervisor::new(broker.clone());
supervisor.start().await?;
// Build high-level patterns on this foundation
```

**Implementation Path**:
- Security context propagation for connection actors
- TcpServer pattern implementation
- Network actor integration with connection pooling

### 2. Event-Driven Architecture Pattern (Planned)

**Concept**: Complex event processing with stateful actors, enabling event sourcing and CQRS patterns.

**Planned API Design**:
```rust
// Conceptual API - NOT YET IMPLEMENTED
use airssys_rt::patterns::EventProcessor;

let event_processor = EventProcessor::new()
    .with_handler(OrderHandler::new())
    .with_handler(PaymentHandler::new())
    .with_supervisor(business_logic_supervisor)
    .start().await?;
```

**Design Rationale**:
- Event handlers as isolated actors (independent scaling)
- Supervisor ensures handler availability
- Event ordering preserved per handler
- Failed events can be retried or dead-lettered

**Current Foundation**:
```rust
// What works today: Message broker with pub-sub pattern
use airssys_rt::broker::{MessageBroker, InMemoryMessageBroker};
use airssys_rt::message::Message;

// Actors subscribe to message types, process independently
broker.subscribe().await?;
broker.publish(event_message).await?;
// Event handlers built on broker primitives
```

**Implementation Path**:
- Audit logging for event processing
- Event handler pattern library
- Event sourcing utilities and examples

### 3. System Service Management Pattern (Planned)

**Concept**: Reliable coordination of system services with dependency management and health monitoring.

**Planned API Design**:
```rust
// Conceptual API - NOT YET IMPLEMENTED
use airssys_rt::patterns::ServiceManager;
use airssys_rt::supervisor::RestartStrategy;

let service_manager = ServiceManager::new()
    .service("database", DatabaseService::new())
    .service("cache", CacheService::new())
    .service("metrics", MetricsService::new())
    .with_restart_strategy(RestartStrategy::OneForOne)
    .start().await?;
```

**Design Rationale**:
- Services as supervised actors (automatic restart)
- Dependency ordering with RestForOne strategy
- Health checks via actor lifecycle
- Graceful shutdown coordination

**Current Foundation**:
```rust
// What works today: Supervisor with restart strategies
use airssys_rt::supervisor::strategy::RestForOne;
use airssys_rt::osl::OSLSupervisor;

// OSLSupervisor manages FileSystem, Process, Network actors
// RestForOne strategy: dependent actors restart in order
let supervisor = OSLSupervisor::new(broker.clone());
// Service manager pattern extends this model
```

**Implementation Path**:
- Service health monitoring integration
- ServiceManager pattern with dependency graph
- Service orchestration examples

## Architecture Evolution

### Current Capabilities

**What's Working Today**:
- ✅ Generic `Actor<M, B>` trait with broker dependency injection
- ✅ `OSLSupervisor` managing FileSystem, Process, Network actors
- ✅ `InMemoryMessageBroker` with pub-sub messaging
- ✅ RestForOne restart strategy
- ✅ Named actor addresses and message correlation
- ✅ Integration with airssys-osl for system operations

**Reference Implementation**:
See `examples/osl_integration_example.rs` for complete working code demonstrating current capabilities.

### Planned Evolution

**Security and Audit Integration** (Next):
- Security context propagation through actor hierarchy
- Audit logging for all actor operations
- Permission validation in supervisors

**High-Level Patterns** (Future):
- TcpServer, EventProcessor, ServiceManager patterns
- Additional examples and pattern library
- Performance benchmarks and optimization
- Migration guides from raw actor APIs

## Design Principles

The planned high-level patterns follow these core principles:

1. **Composition over Configuration**: Patterns built from small, composable primitives
2. **Type Safety**: Generic constraints prevent runtime type errors
3. **Dependency Injection**: Broker and dependencies injected, not hard-coded
4. **Failure Isolation**: Supervisor strategies contain failures
5. **Message-Based**: No shared mutable state, only message passing

## Migration from Foundation to Patterns

When high-level patterns become available, migration will follow this model:

**Current (Foundation API)**:
```rust
// Direct use of core primitives
let broker = InMemoryMessageBroker::new();
let supervisor = OSLSupervisor::new(broker.clone());
let filesystem_actor = FileSystemActor::new(broker.clone());
```

**Future (Pattern API)**:
```rust
// Higher-level ergonomic patterns
let server = TcpServer::new()
    .with_supervisor(...)  // Built on foundation primitives
    .start().await?;
```

**Key Point**: Foundation APIs remain stable. Patterns are additive, not breaking changes.

## References

- **Current Implementation**: See [OSL Integration](../implementation/osl-integration.md)
- **Architecture**: See [Supervisor Trees](../architecture/supervision.md)
- **ADR-RT-009**: Broker Dependency Injection pattern
- **Development Status**: See main [README](../../README.md)

## Providing Feedback

These planned use cases represent our current design vision. If you have:
- **Feedback on planned APIs**: Open a discussion in the repository
- **Alternative patterns to suggest**: Submit an RFC (Request for Comments)
- **Use cases we haven't considered**: File an issue with your requirements

Your input helps shape the evolution of `airssys-rt` to better serve real-world needs.

---

**Last Updated**: 2025-10-14  
**Status**: Conceptual Design - Implementation Planned
