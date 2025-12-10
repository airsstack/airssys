# AirsSys Overview

**AirsSys** is a comprehensive system programming framework designed for the AirsStack ecosystem. It provides secure, modular components for building high-performance concurrent applications with strong security guarantees.

## What is AirsSys?

AirsSys addresses the critical challenges of modern system programming:

- **Security**: Direct OS interactions expose applications to security threats
- **Complexity**: Low-level programming is error-prone and platform-specific  
- **Concurrency**: Building fault-tolerant concurrent systems is difficult
- **Auditability**: Most OS operations lack comprehensive logging

AirsSys solves these challenges through:

1. **OSL (OS Layer)** - Secure, audited OS abstractions
2. **RT (Actor Runtime)** - Fault-tolerant concurrency
3. **Integration** - Components work together seamlessly

## Components

### OSL - OS Layer Framework

**Purpose**: Secure abstraction over operating system functionality

The OS Layer provides a cross-platform interface to system operations with built-in security policies and comprehensive audit logging. Instead of using raw `std::fs` or `std::process`, applications use OSL's helper functions that automatically enforce ACL/RBAC policies and log all activities.

**Architecture**:
```
Application Code
     ↓
Helper Functions API
     ↓
Middleware Pipeline (Logger → Security)
     ↓
Executors (Filesystem, Process, Network)
     ↓
Operating System
```

**Key capabilities**:
- File I/O with path-based access control
- Process spawning with security context
- Network operations with capability enforcement
- Extensible middleware for custom logic

**Security model**:
- Deny-by-default access control
- ACL (Access Control Lists) with glob patterns
- RBAC (Role-Based Access Control) with inheritance
- JSON audit logs for compliance

### RT - Actor Runtime System

**Purpose**: Erlang-inspired actor model for fault-tolerant concurrency

The Actor Runtime implements lightweight virtual processes with BEAM-inspired supervision. Applications build actor systems where isolated processes communicate through message passing, and supervisor trees automatically restart failed actors.

**Architecture**:
```
Supervisor Tree
     ↓
  Actors (isolated state)
     ↓
Message Broker (routing)
     ↓
  Mailboxes (backpressure)
```

**Key capabilities**:
- Zero-cost actor abstraction (~625ns spawn)
- High throughput (4.7M messages/sec)
- Supervision strategies (OneForOne, OneForAll, RestForOne)
- Automatic failure recovery
- Broker-based pub/sub messaging

**Concurrency model**:
- Encapsulated actor state (no shared memory)
- Asynchronous message passing
- Sequential message processing
- Fault isolation with supervisors

## Integration Patterns

AirsSys components are designed to work independently or together:

### Standalone Usage

**OSL standalone**:
```rust
use airssys_osl::helpers::*;

// Secure file operations
let content = read_file("/data/config.toml", "admin").await?;
```

**RT standalone**:
```rust
use airssys_rt::prelude::*;

// Actor-based service
let actor = MyActor::new();
let address = spawn_actor(actor, broker).await?;
```

### Combined Usage

**OSL actors supervised by RT**:
```rust
use airssys_rt::supervisor::OSLSupervisor;

// RT supervisor manages OSL operations
let supervisor = OSLSupervisor::new(broker);
supervisor.start().await?;

// Filesystem, Process, Network actors with fault tolerance
```

This pattern provides:
- Secure OS operations (OSL)
- Fault tolerance (RT supervision)
- Automatic recovery from failures
- Comprehensive audit trails

## Design Principles

### Security by Default

All operations are denied unless explicitly allowed. Security policies are enforced at the middleware layer before execution. Every operation is logged with security context for audit trails.

### Zero-Cost Abstractions

AirsSys uses Rust's generics and compile-time monomorphization to eliminate runtime overhead. The high-level API compiles to the same machine code as hand-written low-level code.

### Modularity

Each component has a clear purpose and can be used independently. OSL doesn't require RT, and RT doesn't require OSL. Integration is opt-in through well-defined interfaces.

### Fault Tolerance

Following Erlang/OTP, AirsSys embraces the "let it crash" philosophy:
- Write simple code for the happy path
- Let failures propagate cleanly
- Use supervisors to detect and recover from failures
- Isolate failures to prevent cascade effects

## Use Cases

### Enterprise Applications

**Requirements**:
- Secure file processing with compliance logging
- Multi-tenant access control
- Audit trails for SOC 2 / HIPAA

**Solution**: OSL with ACL policies and audit logging

### High-Concurrency Services

**Requirements**:
- Handle 10,000+ concurrent connections
- Graceful failure recovery
- Low latency (<100ms p99)

**Solution**: RT with supervisor trees

### System Administration

**Requirements**:
- Automate system operations
- Secure script execution
- Monitor all activities

**Solution**: OSL for operations, RT for workflow orchestration

### Microservices

**Requirements**:
- Service-to-service communication
- Circuit breakers and retries
- Distributed coordination

**Solution**: RT actors with message broker

## Component Comparison

| Feature | OSL | RT |
|---------|-----|-----|
| **Primary Focus** | OS abstraction & security | Concurrency & fault tolerance |
| **Security Model** | ACL/RBAC policies | Actor isolation |
| **Performance** | OS-bound | ~4.7M msgs/sec |
| **Fault Tolerance** | Error propagation | Supervision trees |
| **State Management** | Stateless operations | Encapsulated actor state |
| **Logging** | Activity audit logs | Event monitoring |
| **Use Alone** | ✅ Yes | ✅ Yes |
| **Use Together** | ✅ Yes | ✅ Yes |

## Technology Stack

AirsSys is built on proven Rust ecosystem crates:

- **Tokio** - Async runtime foundation
- **async-trait** - Trait async fn support
- **serde** - Serialization for logging
- **Platform-specific** - `nix` for Unix, `winapi` for Windows

## Getting Started

Ready to start using AirsSys?

1. **[Getting Started Guide](getting-started.md)** - Installation and first steps
2. **[OSL Documentation](components/osl/index.md)** - Secure OS operations
3. **[RT Documentation](components/rt/index.md)** - Actor concurrency
4. **[Examples](examples/index.md)** - Working code samples

## Next Steps

- **Architecture deep dive**: [Architecture](architecture.md)
- **Integration patterns**: [Integration Guide](guides/integration.md)
- **Performance tuning**: [Performance Guide](guides/performance.md)
- **Security best practices**: [Security Guide](guides/security.md)
