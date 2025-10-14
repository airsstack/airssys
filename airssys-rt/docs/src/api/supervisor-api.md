# Supervisor API

This section documents the supervision and fault tolerance API in `airssys-rt`.

## RestartPolicy

Defines when a child should be restarted after failure.

```rust
pub enum RestartPolicy {
    Permanent,   // Always restart on failure
    Transient,   // Restart only on abnormal termination
    Temporary,   // Never restart
}
```

### Usage

```rust
use airssys_rt::supervisor::RestartPolicy;

// Critical service - always restart
let policy = RestartPolicy::Permanent;

// Background task - restart only on crashes
let policy = RestartPolicy::Transient;

// One-time operation - never restart
let policy = RestartPolicy::Temporary;
```

## RestartStrategy

Defines how a supervisor handles child failures.

### OneForOne

Restart only the failed child.

```rust
use airssys_rt::supervisor::OneForOne;

let strategy = OneForOne::new();
```

**Use when**: Children are independent and one child's failure doesn't affect others.

### OneForAll

Restart all children when one fails.

```rust
use airssys_rt::supervisor::OneForAll;

let strategy = OneForAll::new();
```

**Use when**: Children are interdependent and must all restart together.

### RestForOne

Restart the failed child and all children started after it.

```rust
use airssys_rt::supervisor::RestForOne;

let strategy = RestForOne::new();
```

**Use when**: Children have startup dependencies (e.g., database → cache → API).

## ChildSpec

Configuration for a supervised child.

```rust
pub struct ChildSpec {
    pub id: ChildId,
    pub restart_policy: RestartPolicy,
    pub shutdown_policy: ShutdownPolicy,
    pub significant: bool,
}
```

### Fields

- **`id`**: Unique identifier for the child
- **`restart_policy`**: When to restart the child
- **`shutdown_policy`**: How to shutdown the child
- **`significant`**: Whether child failure affects supervisor

### Example

```rust
use airssys_rt::supervisor::{ChildSpec, ChildId, RestartPolicy, ShutdownPolicy};

let spec = ChildSpec {
    id: ChildId::new(),
    restart_policy: RestartPolicy::Permanent,
    shutdown_policy: ShutdownPolicy::default(),
    significant: true,
};
```

## ShutdownPolicy

Defines how a child should be shutdown.

```rust
pub enum ShutdownPolicy {
    Timeout(Duration),
    Brutal,
    Infinity,
}
```

### Variants

- **`Timeout(duration)`**: Wait for graceful shutdown, force stop after timeout
- **`Brutal`**: Immediately force stop
- **`Infinity`**: Wait indefinitely for graceful shutdown

## ChildHealth

Health status of a supervised child.

```rust
pub enum ChildHealth {
    Healthy,
    Unhealthy(String),
}
```

### Example

```rust
use airssys_rt::supervisor::{Child, ChildHealth};

#[async_trait]
impl Child for MyWorker {
    async fn health_check(&self) -> ChildHealth {
        if self.is_connected() {
            ChildHealth::Healthy
        } else {
            ChildHealth::Unhealthy("Connection lost".to_string())
        }
    }
    
    // ... other methods
}
```

## SupervisorNode

Main supervisor implementation.

```rust
pub struct SupervisorNode<M, S> 
where
    M: Message,
    S: SupervisionStrategy<M>,
{
    // Implementation details hidden
}
```

### Methods

```rust
impl<M: Message, S: SupervisionStrategy<M>> SupervisorNode<M, S> {
    /// Create a new supervisor
    pub fn new(id: SupervisorId, strategy: S) -> Self;
    
    /// Add a child to supervise
    pub async fn add_child(
        &mut self,
        spec: ChildSpec,
        child: Box<dyn Child>,
    ) -> Result<(), SupervisorError>;
    
    /// Start all children
    pub async fn start_all_children(&mut self) -> Result<(), SupervisorError>;
    
    /// Stop all children
    pub async fn stop_all_children(&mut self) -> Result<(), SupervisorError>;
    
    /// Handle child failure
    pub async fn handle_child_failure(
        &mut self,
        child_id: &ChildId,
    ) -> Result<(), SupervisorError>;
}
```

### Example

```rust
use airssys_rt::supervisor::{
    SupervisorNode, SupervisorId, OneForOne,
    ChildSpec, ChildId, RestartPolicy, ShutdownPolicy,
};

async fn create_supervisor() -> Result<(), Box<dyn std::error::Error>> {
    // Create supervisor with OneForOne strategy
    let mut supervisor = SupervisorNode::new(
        SupervisorId::new(),
        OneForOne::new(),
    );

    // Add children
    supervisor.add_child(
        ChildSpec {
            id: ChildId::new(),
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::default(),
            significant: true,
        },
        Box::new(my_worker),
    ).await?;

    // Start all children
    supervisor.start_all_children().await?;

    Ok(())
}
```

See `examples/supervisor_basic.rs` for complete implementation.

## HealthConfig

Configuration for automatic health monitoring.

```rust
pub struct HealthConfig {
    pub check_interval: Duration,
    pub unhealthy_threshold: u32,
    pub restart_on_unhealthy: bool,
}
```

### Example

```rust
use airssys_rt::supervisor::HealthConfig;
use std::time::Duration;

let config = HealthConfig {
    check_interval: Duration::from_secs(5),
    unhealthy_threshold: 3,
    restart_on_unhealthy: true,
};
```

See `examples/supervisor_automatic_health.rs` for health monitoring.

## SupervisionStrategy Trait

Trait for implementing custom supervision strategies.

```rust
#[async_trait]
pub trait SupervisionStrategy<M: Message>: Send + Sync {
    async fn handle_failure(
        &self,
        context: &mut StrategyContext<M>,
        failed_child: &ChildId,
    ) -> SupervisionDecision;
}
```

Current implementations: `OneForOne`, `OneForAll`, `RestForOne`

## SupervisionDecision

Result of supervision strategy decision.

```rust
pub enum SupervisionDecision {
    Restart(Vec<ChildId>),  // List of children to restart
    Escalate,               // Escalate to parent supervisor
    Stop,                   // Stop supervision
}
```

## Complete Supervision Example

```rust
use airssys_rt::supervisor::{
    Child, ChildHealth, ChildSpec, ChildId,
    RestartPolicy, ShutdownPolicy,
    SupervisorNode, SupervisorId, OneForOne,
};
use async_trait::async_trait;
use std::error::Error;

// Define a worker
struct DatabaseWorker {
    connected: bool,
}

#[async_trait]
impl Child for DatabaseWorker {
    async fn start(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("Connecting to database...");
        self.connected = true;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("Disconnecting from database...");
        self.connected = false;
        Ok(())
    }

    async fn health_check(&self) -> ChildHealth {
        if self.connected {
            ChildHealth::Healthy
        } else {
            ChildHealth::Unhealthy("Not connected".to_string())
        }
    }
}

// Create supervised system
async fn setup() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut supervisor = SupervisorNode::new(
        SupervisorId::new(),
        OneForOne::new(),
    );

    supervisor.add_child(
        ChildSpec {
            id: ChildId::new(),
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::default(),
            significant: true,
        },
        Box::new(DatabaseWorker { connected: false }),
    ).await?;

    supervisor.start_all_children().await?;
    
    Ok(())
}
```

## Supervisor Hierarchy

Supervisors can supervise other supervisors:

```rust
// Parent supervisor
let mut parent = SupervisorNode::new(
    SupervisorId::new(),
    OneForOne::new(),
);

// Child supervisor
let child_supervisor = SupervisorNode::new(
    SupervisorId::new(),
    RestForOne::new(),
);

// Add child supervisor as a child (via Child trait implementation)
parent.add_child(
    ChildSpec { /* ... */ },
    Box::new(child_supervisor),
).await?;
```

See the examples directory for complete supervision patterns:
- `examples/supervisor_basic.rs` - Basic supervision
- `examples/supervisor_strategies.rs` - Strategy comparison
- `examples/supervisor_automatic_health.rs` - Health monitoring