# Introduction

**airssys-rt** is a lightweight Erlang-Actor model runtime system designed for high-concurrency applications within the AirsSys ecosystem. This component provides an in-memory process management system inspired by the BEAM virtual machine, focusing on actor-based concurrency patterns and supervisor tree fault tolerance.

## What is airssys-rt?

`airssys-rt` implements core concepts from the Erlang/OTP runtime in Rust, specifically:

- **Virtual Process Management**: Lightweight, isolated processes similar to BEAM's green threads
- **Actor Model**: Encapsulated state with asynchronous message passing
- **Supervisor Trees**: Hierarchical fault tolerance and automatic recovery
- **Mailbox Pattern**: Sequential message processing with backpressure control

## Design Philosophy

### Lightweight & Focused
Unlike attempting to recreate the entire BEAM runtime, `airssys-rt` focuses on the essential patterns that make Erlang/OTP systems resilient and scalable. It provides a Rust-native implementation that integrates seamlessly with the async/await ecosystem.

### In-Memory Process Model
The "processes" in `airssys-rt` are virtual processes - lightweight units of execution managed entirely in memory, similar to:
- Erlang processes in BEAM
- Green threads in Go
- Actors in Akka/Actor model frameworks

These are **not** OS processes, but rather managed execution contexts with isolated state.

### Integration with AirsSys Ecosystem
`airssys-rt` is designed to work closely with other AirsSys components:
- **airssys-osl**: Provides OS-level integration and security context
- **airssys-wasm**: Future integration for WebAssembly component hosting

## Core Principles

### 1. Encapsulation
Each actor maintains private internal state that cannot be directly accessed by other actors. State mutations happen only through message processing.

### 2. Asynchronous Message Passing
Actors communicate exclusively through immutable messages. No shared memory, no locks, no race conditions.

### 3. Mailbox & Sequential Processing
Every actor has a mailbox that queues incoming messages. Messages are processed one at a time, ensuring state consistency.

### 4. Fault Isolation & Supervision
Failures are contained within individual actors. Supervisor actors monitor their children and implement restart strategies.

## Key Features

- **High Concurrency**: Support for 10,000+ concurrent actors
- **Low Latency**: Sub-millisecond message delivery for local communication
- **Fault Tolerance**: Supervisor-based fault recovery and isolation
- **Memory Efficiency**: Minimal overhead per actor (<1KB baseline)
- **Async Integration**: Built on Tokio for seamless async/await compatibility

## Use Cases

`airssys-rt` is ideal for:
- **High-concurrency servers** requiring fault tolerance
- **Event-driven architectures** with complex state management
- **System programming** requiring reliable process supervision
- **Microservice coordination** within the AirsSys ecosystem
- **Real-time applications** with soft latency requirements

## Getting Started

To begin using `airssys-rt`, see the [Getting Started](./implementation/getting-started.md) guide in the Implementation section.

For architectural details, explore the [Core Concepts](./architecture/core-concepts.md) and [Actor Model Design](./architecture/actor-model.md) sections.

To understand the research and design decisions behind `airssys-rt`, review the [Research Overview](./researches.md) section.