# airssys-rt Project Brief

## Project Overview
`airssys-rt` (Runtime) provides a lightweight Erlang-Actor model implementation for high-concurrent applications within the AirsSys ecosystem. It follows BEAM runtime principles while being specifically designed for system programming contexts.

## Project Goals
1. **Lightweight Actor Model**: Efficient actor-based concurrency without BEAM overhead
2. **Supervisor Tree Management**: Robust fault tolerance and recovery mechanisms  
3. **High Concurrency Support**: Handle thousands of concurrent actors efficiently
4. **AirsSys Integration**: Seamless integration with airssys-osl and airssys-wasm
5. **Message Passing Excellence**: Fast, reliable asynchronous message passing

## Core Responsibilities

### Actor Model Implementation
- Lightweight actor creation and lifecycle management
- Private state encapsulation per actor
- Efficient message mailbox implementation
- Sequential message processing guarantees

### Message Passing System
- Asynchronous message delivery
- Immutable message semantics
- Message routing and addressing
- Backpressure and flow control

### Supervisor Tree
- Hierarchical process supervision
- Configurable restart strategies
- Fault isolation and recovery
- System health monitoring

### Process Management
- Actor spawning and termination
- Resource management and cleanup
- Performance monitoring and metrics
- Integration with airssys-osl for OS-level process management

## Technical Requirements

### Performance Requirements
- Support 10,000+ concurrent actors
- Sub-millisecond message delivery latency
- Minimal memory overhead per actor
- Efficient CPU utilization under load

### Reliability Requirements
- Fault tolerance through supervisor trees
- Graceful degradation under resource pressure
- Deterministic failure recovery
- Comprehensive error reporting

### Integration Requirements
- Use airssys-osl for OS-level operations
- Provide actor primitives for airssys-wasm components
- Support distributed actor communication (future)
- Integration with monitoring and logging systems

## Architecture Constraints
- Follow workspace standards (§2.1, §3.2, §4.3, §5.1)
- Async-first design using Tokio runtime
- Zero-copy message passing where possible
- Memory-safe implementation without compromising performance

## Success Criteria
- Achieve target performance metrics under load
- Demonstrate fault tolerance through supervisor trees
- Successful integration with airssys-osl and airssys-wasm
- Comprehensive test coverage including fault injection testing