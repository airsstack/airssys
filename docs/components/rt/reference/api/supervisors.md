# Supervisors API Reference

This reference documents the supervision system, restart strategies, and fault tolerance mechanisms.

## Module: `supervisor`

Supervision and fault tolerance types.

### Trait: `Supervisor`

```rust
pub trait Supervisor: Actor {
    fn child_specs(&self) -> Vec<ChildSpec>;
    fn restart_strategy(&self) -> RestartStrategy;
    
    async fn handle_child_failure(
        &mut self,
        child_id: ActorId,
        error: ActorError,
        ctx: &mut ActorContext<Self>,
    ) -> ErrorAction {
        // Default implementation based on restart_strategy()
    }
}
```

Trait for actors that supervise child actors.

**Required Methods:**

- `child_specs()`: Returns specifications for children to spawn
- `restart_strategy()`: Returns the restart strategy to use

**Provided Methods:**

- `handle_child_failure()`: Handles child actor failures (default: apply restart_strategy)

**Trait Bounds:**
- Must implement `Actor`

**Example:**

```rust
use airssys_rt::{Actor, ActorContext, Supervisor, ChildSpec, RestartStrategy};

struct AppSupervisor {
    workers: Vec<ActorRef<Worker>>,
}

impl Actor for AppSupervisor {
    async fn receive(&mut self, ctx: &mut ActorContext<Self>, msg: Box<dyn Message>) {
        // Handle messages
    }
}

impl Supervisor for AppSupervisor {
    fn child_specs(&self) -> Vec<ChildSpec> {
        vec![
            ChildSpec::new("worker-1", || Worker::new()),
            ChildSpec::new("worker-2", || Worker::new()),
            ChildSpec::new("worker-3", || Worker::new()),
        ]
    }
    
    fn restart_strategy(&self) -> RestartStrategy {
        RestartStrategy::OneForOne
    }
}
```

### Trait: `Child`

```rust
pub trait Child: Actor {
    fn child_id(&self) -> &str;
    fn restart_policy(&self) -> RestartPolicy;
}
```

Trait for actors that can be supervised.

**Required Methods:**

- `child_id()`: Returns unique identifier for this child
- `restart_policy()`: Returns restart policy for this child

**Example:**

```rust
use airssys_rt::{Actor, Child, RestartPolicy};

struct Worker {
    id: String,
    max_retries: u32,
}

impl Actor for Worker {
    async fn receive(&mut self, ctx: &mut ActorContext<Self>, msg: Box<dyn Message>) {
        // Handle messages
    }
}

impl Child for Worker {
    fn child_id(&self) -> &str {
        &self.id
    }
    
    fn restart_policy(&self) -> RestartPolicy {
        RestartPolicy::Transient {
            max_retries: self.max_retries,
            backoff: Duration::from_secs(1),
        }
    }
}
```

## Child Specifications

### Struct: `ChildSpec`

```rust
pub struct ChildSpec {
    pub id: String,
    pub factory: Box<dyn Fn() -> Box<dyn Actor> + Send + Sync>,
    pub restart_policy: RestartPolicy,
}
```

Specification for spawning and managing a child actor.

**Fields:**

- `id`: Unique identifier for the child
- `factory`: Function to create new child instances
- `restart_policy`: Policy for restarting on failure

#### Constructors

##### `new()`

```rust
pub fn new<F, A>(id: &str, factory: F) -> Self
where
    F: Fn() -> A + Send + Sync + 'static,
    A: Actor,
```

Creates a child specification with default restart policy.

**Type Parameters:**
- `F`: Factory function type
- `A`: Actor type

**Parameters:**
- `id`: Unique identifier
- `factory`: Function to create actor instances

**Default Policy:**
- `RestartPolicy::Permanent` (always restart)

**Example:**

```rust
use airssys_rt::supervisor::ChildSpec;

let spec = ChildSpec::new("worker", || Worker::new());
```

##### `with_policy()`

```rust
pub fn with_policy<F, A>(id: &str, factory: F, policy: RestartPolicy) -> Self
where
    F: Fn() -> A + Send + Sync + 'static,
    A: Actor,
```

Creates a child specification with custom restart policy.

**Example:**

```rust
use airssys_rt::supervisor::{ChildSpec, RestartPolicy};
use std::time::Duration;

let spec = ChildSpec::with_policy(
    "worker",
    || Worker::new(),
    RestartPolicy::Transient {
        max_retries: 3,
        backoff: Duration::from_secs(1),
    },
);
```

## Restart Strategies

### Enum: `RestartStrategy`

```rust
pub enum RestartStrategy {
    OneForOne,
    OneForAll,
    RestForOne,
}
```

Strategy for restarting child actors when failures occur.

**Variants:**

- `OneForOne`: Restart only the failed child
- `OneForAll`: Restart all children when any child fails
- `RestForOne`: Restart failed child and all children started after it

**Performance Characteristics:**

| Strategy | Latency | Disruption | Use Case |
|----------|---------|------------|----------|
| OneForOne | 10-50µs | Minimal | Independent workers |
| OneForAll | 30-150µs | Complete | Interdependent components |
| RestForOne | 20-100µs | Partial | Pipeline stages |

**Visual Representation:**

```
OneForOne:
Before: [A] [B] [C] [D]
B fails: [A] [B'] [C] [D]  (only B restarted)

OneForAll:
Before: [A] [B] [C] [D]
B fails: [A'] [B'] [C'] [D']  (all restarted)

RestForOne:
Before: [A] [B] [C] [D]
B fails: [A] [B'] [C'] [D']  (B, C, D restarted)
```

**Decision Guide:**

```rust
// Independent workers - failures don't affect each other
impl Supervisor for WorkerPool {
    fn restart_strategy(&self) -> RestartStrategy {
        RestartStrategy::OneForOne
    }
}

// Interconnected services - need consistent state
impl Supervisor for ServiceCluster {
    fn restart_strategy(&self) -> RestartStrategy {
        RestartStrategy::OneForAll
    }
}

// Pipeline stages - downstream depends on upstream
impl Supervisor for Pipeline {
    fn restart_strategy(&self) -> RestartStrategy {
        RestartStrategy::RestForOne
    }
}
```

### Enum: `RestartPolicy`

```rust
pub enum RestartPolicy {
    Permanent,
    Temporary,
    Transient {
        max_retries: u32,
        backoff: Duration,
    },
}
```

Policy for when and how to restart a specific child.

**Variants:**

- `Permanent`: Always restart on failure (default)
- `Temporary`: Never restart, remove on failure
- `Transient { max_retries, backoff }`: Restart up to max_retries with exponential backoff

**Use Cases:**

```rust
use airssys_rt::supervisor::RestartPolicy;
use std::time::Duration;

// Critical service - must always be running
let database_policy = RestartPolicy::Permanent;

// One-time initialization task
let init_policy = RestartPolicy::Temporary;

// Flaky external service - retry with backoff
let api_policy = RestartPolicy::Transient {
    max_retries: 5,
    backoff: Duration::from_secs(2),
};
```

## Supervisor Builder Pattern (RT-TASK-013)

### Struct: `SupervisorBuilder`

```rust
pub struct SupervisorBuilder {
    // fields omitted
}
```

Builder for creating supervisors with fluent API.

#### Methods

##### `new()`

```rust
pub fn new(name: &str) -> Self
```

Creates a new supervisor builder.

**Parameters:**
- `name`: Supervisor identifier

**Example:**

```rust
use airssys_rt::supervisor::SupervisorBuilder;

let builder = SupervisorBuilder::new("app-supervisor");
```

##### `with_strategy()`

```rust
pub fn with_strategy(mut self, strategy: RestartStrategy) -> Self
```

Sets the restart strategy.

**Parameters:**
- `strategy`: Restart strategy to use

**Returns:**
- `Self`: Builder for method chaining

**Example:**

```rust
builder.with_strategy(RestartStrategy::OneForAll)
```

##### `add_child()`

```rust
pub fn add_child(mut self, spec: ChildSpec) -> Self
```

Adds a child specification.

**Parameters:**
- `spec`: Child specification

**Returns:**
- `Self`: Builder for method chaining

**Example:**

```rust
builder.add_child(ChildSpec::new("worker-1", || Worker::new()))
```

##### `build()`

```rust
pub fn build(self) -> Result<GenericSupervisor, BuildError>
```

Builds the supervisor.

**Returns:**
- `Ok(GenericSupervisor)`: Successfully built supervisor
- `Err(BuildError)`: Build failed (e.g., no children specified)

**Example:**

```rust
use airssys_rt::supervisor::{SupervisorBuilder, ChildSpec, RestartStrategy};

let supervisor = SupervisorBuilder::new("app")
    .with_strategy(RestartStrategy::OneForOne)
    .add_child(ChildSpec::new("worker-1", || Worker::new()))
    .add_child(ChildSpec::new("worker-2", || Worker::new()))
    .add_child(ChildSpec::new("worker-3", || Worker::new()))
    .build()?;
```

## Supervision Patterns

### Basic Supervision

```rust
use airssys_rt::{Actor, ActorContext, Supervisor, ChildSpec, RestartStrategy};

struct WorkerSupervisor;

impl Actor for WorkerSupervisor {
    async fn receive(&mut self, ctx: &mut ActorContext<Self>, msg: Box<dyn Message>) {
        // Handle supervisor messages
    }
}

impl Supervisor for WorkerSupervisor {
    fn child_specs(&self) -> Vec<ChildSpec> {
        vec![
            ChildSpec::new("worker-1", || Worker::new()),
            ChildSpec::new("worker-2", || Worker::new()),
        ]
    }
    
    fn restart_strategy(&self) -> RestartStrategy {
        RestartStrategy::OneForOne
    }
}
```

### Hierarchical Supervision

```rust
// Top-level supervisor
struct AppSupervisor;

impl Supervisor for AppSupervisor {
    fn child_specs(&self) -> Vec<ChildSpec> {
        vec![
            ChildSpec::new("web-supervisor", || WebSupervisor),
            ChildSpec::new("db-supervisor", || DbSupervisor),
            ChildSpec::new("cache-supervisor", || CacheSupervisor),
        ]
    }
    
    fn restart_strategy(&self) -> RestartStrategy {
        RestartStrategy::OneForAll  // Coordinated restart
    }
}

// Mid-level supervisor
struct WebSupervisor;

impl Supervisor for WebSupervisor {
    fn child_specs(&self) -> Vec<ChildSpec> {
        vec![
            ChildSpec::new("http-server", || HttpServer::new()),
            ChildSpec::new("websocket-server", || WebSocketServer::new()),
        ]
    }
    
    fn restart_strategy(&self) -> RestartStrategy {
        RestartStrategy::RestForOne  // Sequential dependency
    }
}
```

### Custom Failure Handling

```rust
impl Supervisor for CustomSupervisor {
    fn child_specs(&self) -> Vec<ChildSpec> {
        // ... child specs ...
    }
    
    fn restart_strategy(&self) -> RestartStrategy {
        RestartStrategy::OneForOne
    }
    
    async fn handle_child_failure(
        &mut self,
        child_id: ActorId,
        error: ActorError,
        ctx: &mut ActorContext<Self>,
    ) -> ErrorAction {
        // Log failure
        eprintln!("Child {} failed: {:?}", child_id, error);
        
        // Custom logic based on error type
        match error {
            ActorError::Timeout => {
                // Timeouts are transient, restart
                ErrorAction::Restart
            }
            ActorError::ConfigError(_) => {
                // Config errors are permanent, stop
                ErrorAction::Stop
            }
            _ => {
                // Unknown errors, escalate
                ErrorAction::Escalate
            }
        }
    }
}
```

## Health Monitoring Integration

### Automatic Health Checks

```rust
use airssys_rt::supervisor::HealthCheck;

impl Supervisor for MonitoredSupervisor {
    fn child_specs(&self) -> Vec<ChildSpec> {
        vec![
            ChildSpec::new("worker", || Worker::new())
                .with_health_check(HealthCheck {
                    interval: Duration::from_secs(30),
                    timeout: Duration::from_secs(5),
                }),
        ]
    }
    
    fn restart_strategy(&self) -> RestartStrategy {
        RestartStrategy::OneForOne
    }
}
```

### Manual Health Checks

```rust
use airssys_rt::Message;

struct CheckHealth;
impl Message for CheckHealth {
    type Result = HealthStatus;
}

impl Handler<CheckHealth> for Worker {
    async fn handle(&mut self, _msg: CheckHealth, _ctx: &mut ActorContext<Self>) -> HealthStatus {
        if self.is_healthy() {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unhealthy {
                reason: "Connection lost".to_string(),
            }
        }
    }
}
```

## Performance Characteristics

### Restart Latency

| Strategy | Min | Avg | Max | Notes |
|----------|-----|-----|-----|-------|
| OneForOne | 8µs | 10-50µs | 200µs | Single actor restart |
| OneForAll (4 children) | 25µs | 30-150µs | 500µs | Parallel restart |
| RestForOne (3 affected) | 18µs | 20-100µs | 350µs | Sequential restart |

### Restart Count Performance

| Restart Count | Cold Start | Warm Restart | Delta |
|---------------|------------|--------------|-------|
| 1st restart | 50µs | 10µs | -80% |
| 5th restart | 50µs | 8µs | -84% |
| 10th restart | 50µs | 7µs | -86% |

Warm restarts are faster due to:
- Cached actor metadata
- Pre-allocated mailboxes
- Optimized supervision tree traversal

### Memory Overhead

| Component | Size | Per Child | Notes |
|-----------|------|-----------|-------|
| Supervisor state | ~512 bytes | - | Base overhead |
| ChildSpec | ~128 bytes | Yes | Per child spec |
| Restart tracking | ~64 bytes | Yes | Per active child |
| Health check state | ~96 bytes | Yes | If enabled |

## Error Types

### Enum: `SupervisorError`

```rust
pub enum SupervisorError {
    ChildStartFailed(String),
    TooManyRestarts {
        child_id: String,
        count: u32,
        window: Duration,
    },
    InvalidStrategy,
}
```

Errors specific to supervisor operations.

**Variants:**

- `ChildStartFailed(String)`: Child actor failed to start
- `TooManyRestarts`: Restart limit exceeded
- `InvalidStrategy`: Unsupported restart strategy configuration

**Example:**

```rust
use airssys_rt::supervisor::SupervisorError;

match supervisor_result {
    Err(SupervisorError::TooManyRestarts { child_id, count, window }) => {
        eprintln!("Child {} restarted {} times in {:?}", child_id, count, window);
        // Take corrective action
    }
    _ => {}
}
```

## Testing Utilities

### Struct: `SupervisorTestProbe`

```rust
pub struct SupervisorTestProbe {
    // fields omitted
}
```

Testing utility for supervisor behavior.

**Available in:** Test builds only (`#[cfg(test)]`)

#### Methods

##### `expect_restart()`

```rust
pub async fn expect_restart(&mut self, child_id: &str, timeout: Duration) -> bool
```

Waits for a child restart event.

**Example:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use airssys_rt::supervisor::SupervisorTestProbe;
    
    #[tokio::test]
    async fn test_supervisor_restarts_failed_child() {
        let mut probe = SupervisorTestProbe::new();
        // ... trigger child failure ...
        assert!(probe.expect_restart("worker-1", Duration::from_secs(1)).await);
    }
}
```

##### `restart_count()`

```rust
pub fn restart_count(&self, child_id: &str) -> u32
```

Returns the number of times a child has been restarted.

## See Also

- [Core API Reference](core.md) - Core types and system API
- [Actors API Reference](actors.md) - Actor types and lifecycle
- [Monitoring API Reference](monitoring.md) - Health monitoring system
- [Architecture: Supervision](../../architecture/supervision.md) - Design overview
- [How-To: Supervisor Patterns](../../guides/supervisor-patterns.md) - Usage patterns
- [Explanation: Supervision](../../explanation/supervision.md) - Design rationale
