# AirsSys Architecture

This document describes the overall architecture of the AirsSys ecosystem and how its components work together.

## System Overview

AirsSys is designed as a layered system where each component has clear responsibilities:

```
┌─────────────────────────────────────────────────────────┐
│                   Applications                           │
│            (Your Code Using AirsSys)                     │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│              AirsSys Components Layer                    │
│  ┌──────────────────┐      ┌─────────────────────────┐ │
│  │  OSL Framework   │←────→│    RT Runtime System    │ │
│  │  (OS Operations) │      │  (Actor Concurrency)    │ │
│  └──────────────────┘      └─────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                Operating System Layer                     │
│         (Linux, macOS, Windows, etc.)                    │
└─────────────────────────────────────────────────────────┘
```

## Component Architecture

### OSL (OS Layer) Architecture

The OS Layer uses a middleware pipeline pattern:

```
┌────────────────────────────────────────┐
│         Application Code                │
└────────────────┬───────────────────────┘
                 │
                 ↓
┌────────────────────────────────────────┐
│      Helper Functions API (Level 1)    │
│  read_file(), write_file(), etc.       │
└────────────────┬───────────────────────┘
                 │
                 ↓
┌────────────────────────────────────────┐
│       Middleware Pipeline               │
│  ┌──────────────┐  ┌────────────────┐ │
│  │   Logger     │→ │   Security     │ │
│  │  Middleware  │  │   Middleware   │ │
│  └──────────────┘  └────────────────┘ │
└────────────────┬───────────────────────┘
                 │
                 ↓
┌────────────────────────────────────────┐
│            Executors                    │
│  ┌──────────┐ ┌─────────┐ ┌─────────┐│
│  │Filesystem│ │ Process │ │ Network ││
│  │ Executor │ │Executor │ │Executor ││
│  └──────────┘ └─────────┘ └─────────┘│
└────────────────┬───────────────────────┘
                 │
                 ↓
┌────────────────────────────────────────┐
│           Operating System              │
└────────────────────────────────────────┘
```

**Key design decisions**:
- Operations are immutable types
- Middleware has opportunity to inspect/modify
- Executors are pluggable and testable
- Security enforced before execution

### RT (Actor Runtime) Architecture

The Actor Runtime uses a supervisor tree pattern:

```
┌────────────────────────────────────────┐
│        Supervisor Tree Root             │
│     (restart strategy: OneForOne)       │
└────────┬─────────┬──────────┬──────────┘
         │         │          │
    ┌────▼────┐┌──▼───┐┌────▼────┐
    │ Actor 1 ││Actor 2││ Actor 3 │
    │(Worker) ││(Super)││(Worker) │
    └─────────┘└───┬───┘└─────────┘
                   │
              ┌────▼────┐
              │ Actor 4 │
              │(Worker) │
              └─────────┘

Message Flow:
    Application
         ↓
    Message Broker (routing)
         ↓
    Actor Mailbox (buffering)
         ↓
    Actor (sequential processing)
```

**Key design decisions**:
- Actors are isolated (no shared state)
- Messages are async and immutable
- Mailboxes provide backpressure
- Supervisors monitor and restart

## Integration Patterns

### Pattern 1: OSL with RT Supervision

Wrap OSL operations in RT actors for fault tolerance:

```rust
// OSL actor managed by RT supervisor
struct FileSystemActor;

#[async_trait]
impl Actor for FileSystemActor {
    async fn handle_message(&mut self, msg: FileOp) {
        match msg {
            FileOp::Read(path) => {
                // Use OSL helper
                let content = read_file(&path, "system").await?;
                // Process content...
            }
        }
    }
}
```

The supervisor automatically restarts the actor if OSL operations fail.

### Pattern 2: OSL Middleware in RT Message Handler

Use OSL middleware directly in RT actors:

```rust
struct SecureFileActor {
    security: SecurityMiddleware,
}

#[async_trait]
impl Actor for SecureFileActor {
    async fn handle_message(&mut self, msg: FileOp) {
        // OSL security check
        self.security.check_access(&msg.path, &msg.principal)?;
        
        // Execute operation
        let result = execute_file_operation(msg).await?;
        Ok(result)
    }
}
```

### Pattern 3: Complete Integration

Use the built-in `OSLSupervisor`:

```rust
use airssys_rt::supervisor::OSLSupervisor;

let supervisor = OSLSupervisor::new(broker);
supervisor.start().await?;

// FileSystem, Process, Network actors now running
// with full OSL security and RT fault tolerance
```

## Data Flow

### OSL Operation Execution

1. **Application** calls helper function: `read_file(path, principal)`
2. **Helper** creates operation: `FileReadOperation::new(path)`
3. **Helper** creates context: `ExecutionContext::new(SecurityContext::new(principal))`
4. **Middleware** logs operation: `Logger::log_operation(op, ctx)`
5. **Middleware** checks security: `Security::check_policy(op, ctx)`
6. **Executor** executes: `FilesystemExecutor::execute(op, ctx)`
7. **Result** returned to application

### RT Message Processing

1. **Application** publishes message: `broker.publish(msg, address)`
2. **Broker** routes to mailbox: `mailbox.send(envelope)`
3. **Mailbox** queues message (applies backpressure if full)
4. **Actor** dequeues message: `mailbox.receive()`
5. **Actor** processes sequentially: `actor.handle_message(msg)`
6. **Supervisor** monitors health: `supervisor.check_child()`
7. **Supervisor** restarts on failure: `supervisor.restart_child()`

## Security Architecture

### OSL Security Model

**Deny-by-default**: All operations denied unless explicitly allowed.

**Policy hierarchy**:
```
1. Check ACL (Access Control List)
   - Glob pattern matching on paths
   - Per-resource permissions
   
2. Check RBAC (Role-Based Access Control)
   - Role hierarchy traversal
   - Permission inheritance
   
3. Check custom policies
   - Rate limiting
   - Time-based access
   - Custom logic
```

**Audit trail**:
```json
{
  "timestamp": "2025-12-10T10:30:00Z",
  "principal": "alice",
  "operation": "FileRead",
  "resource": "/data/sensitive.txt",
  "result": "allowed",
  "policy": "acl:data-read"
}
```

### RT Security Model

**Isolation**: Each actor has private state, enforced by Rust ownership.

**Message validation**: Messages must implement `Message` trait.

**Supervision**: Failed actors restarted without affecting siblings.

## Performance Characteristics

### OSL Performance

- **Helper functions**: ~100μs overhead (logging + security)
- **Middleware**: ~10μs per middleware
- **Executors**: Direct OS call performance

### RT Performance

- **Actor spawn**: ~625ns
- **Message throughput**: 4.7M msgs/sec
- **Message latency**: <1ms p99
- **Mailbox overhead**: ~182ns per message

## Deployment Architecture

### Development

```
Local Development
├── cargo run              # Direct execution
├── cargo test            # Unit tests
└── cargo doc --open      # API docs
```

### Production

```
Production Deployment
├── Binary with tokio runtime
├── OSL with file/console logging
├── RT with monitoring enabled
└── External metrics (Prometheus/etc)
```

## Component Boundaries

### OSL Boundaries

**Inputs**: Operation types, security context
**Outputs**: Execution results, audit logs
**External deps**: OS syscalls, filesystem, network stack

### RT Boundaries

**Inputs**: Messages, actor addresses
**Outputs**: Message routing, supervision events
**External deps**: Tokio runtime, async channels

## Next Steps

- **[OSL Architecture Details](components/osl/index.md)**
- **[RT Architecture Details](components/rt/architecture/core-concepts.md)**
- **[Integration Guide](guides/integration.md)**
- **[Performance Guide](guides/performance.md)**
