# Architecture Overview

This section provides comprehensive documentation of `airssys-rt`'s architecture, covering the fundamental design principles, core components, and system organization that make up the actor runtime.

## Architectural Philosophy

`airssys-rt` is designed around the proven actor model principles from Erlang/OTP, adapted for Rust's type system and performance characteristics. The architecture emphasizes:

- **Isolation**: Each actor maintains private state with no shared memory
- **Message Passing**: Asynchronous communication through immutable messages
- **Fault Tolerance**: Hierarchical supervision for automatic recovery
- **Scalability**: Lightweight virtual processes for high concurrency

## System Components

### Virtual Process Management
The runtime manages lightweight virtual processes (not OS processes) that serve as execution contexts for actors. Each virtual process encapsulates:
- Unique process identifier (PID)
- Private actor state
- Message mailbox
- Supervision relationships

### Actor Model Implementation
The actor model provides the programming interface for building concurrent applications:
- **Actor Trait**: Core interface for message handling
- **Message Types**: Strongly-typed communication protocols
- **Lifecycle Management**: Actor creation, execution, and termination
- **State Encapsulation**: Private actor state with controlled access

### Supervision Framework
Hierarchical fault tolerance through supervisor trees:
- **Supervisor Actors**: Monitor and manage child actors
- **Restart Strategies**: Configurable failure recovery policies
- **Error Propagation**: Structured error handling and escalation
- **System Resilience**: Automatic recovery from component failures

### Message Passing System
Efficient asynchronous communication infrastructure:
- **Mailbox Queues**: Per-actor message queues with backpressure
- **Message Routing**: Efficient delivery to target actors
- **Zero-Copy Optimization**: Ownership transfer for performance
- **Type Safety**: Compile-time message type verification

## Architecture Sections

The architecture documentation is organized into the following detailed sections:

### [Core Concepts](./architecture/core-concepts.md)
Fundamental concepts and terminology used throughout the system:
- Virtual processes and their characteristics
- Actor model principles and implementation
- Message passing semantics and patterns
- Supervision tree organization
- System architecture layers and performance considerations

### [Actor Model Design](./architecture/actor-model.md)
Detailed design of the actor programming model:
- Actor trait architecture and lifecycle
- Message design patterns and type safety
- State management and encapsulation
- Communication patterns and best practices
- Performance optimizations and error handling

### [Message Passing System](./architecture/message-passing.md)
In-depth coverage of the message passing infrastructure:
- Mailbox implementation and queuing strategies
- Message routing and delivery mechanisms
- Performance optimizations and zero-copy techniques
- Backpressure and flow control

### [Supervisor Trees](./architecture/supervision.md)
Comprehensive supervision and fault tolerance design:
- Supervision hierarchy and relationships
- Restart strategies and policies
- Error handling and recovery mechanisms
- System-wide resilience patterns

### [Process Lifecycle](./architecture/process-lifecycle.md)
Complete actor lifecycle management:
- Actor spawning and initialization
- Execution phases and state transitions
- Graceful shutdown and cleanup
- Resource management and optimization

## Design Principles

### 1. Type Safety First
Leverage Rust's type system to prevent common concurrency errors:
- Compile-time message type verification
- Actor state ownership and borrowing
- Safe inter-actor communication
- Memory safety without garbage collection

### 2. Performance by Design
Optimize for high-throughput, low-latency scenarios:
- Minimal per-actor memory overhead (<1KB)
- Zero-copy message passing where possible
- Efficient scheduling and context switching
- Integration with Tokio's async runtime

### 3. Fault Tolerance as Default
Build resilience into the system architecture:
- Automatic failure detection and recovery
- Hierarchical error handling and escalation
- Isolation of failures to prevent cascading
- Self-healing system characteristics

### 4. Ergonomic APIs
Provide clean, intuitive interfaces for developers:
- Minimal boilerplate for common patterns
- Clear error handling and diagnostics
- Comprehensive documentation and examples
- Integration with Rust ecosystem tools

## Integration Points

### AirsSys Ecosystem
Integration with other AirsSys components:
- **airssys-osl**: OS-level operations and security context
- **airssys-wasm**: WebAssembly component hosting (planned)
- **Shared Standards**: Common error handling and logging patterns

### Rust Ecosystem
Compatibility with standard Rust tools and libraries:
- **Tokio Runtime**: Async/await foundation and scheduling
- **Serde**: Message serialization and configuration
- **Tracing**: Observability and debugging support
- **Standard Library**: File I/O, networking, and system APIs

## Performance Characteristics

### Scalability Targets
- **Concurrent Actors**: 10,000+ per runtime instance
- **Message Throughput**: 1M+ messages/second under optimal conditions
- **Memory Efficiency**: <1KB baseline overhead per actor
- **Startup Time**: <100μs for actor creation and initialization

### Latency Goals
- **Message Delivery**: <1ms for local actor communication
- **Actor Spawning**: <100μs from request to ready state
- **Supervision Response**: <10ms for failure detection and restart
- **System Overhead**: <5% CPU overhead for runtime management

The architecture documentation provides the foundation for understanding how `airssys-rt` implements the actor model in Rust while maintaining high performance and strong fault tolerance guarantees.