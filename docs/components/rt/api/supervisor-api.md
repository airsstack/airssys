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

**Type:** Generic struct with factory function

```rust
pub struct ChildSpec<C, F>
where
    F: Fn() -> C + Send + Sync + 'static,
{
    /// Unique identifier for this child (for logging and monitoring)
    pub id: String,

    /// Factory function that creates new child instances
    pub factory: F,

    /// Restart policy determining when to restart this child
    pub restart_policy: RestartPolicy,

    /// Shutdown policy determining how to stop this child
    pub shutdown_policy: ShutdownPolicy,

    /// Maximum time to wait for child startup
    pub start_timeout: Duration,

    /// Maximum time to wait for child shutdown
    pub shutdown_timeout: Duration,
}
```

### Type Parameters

- **`C`**: Child type implementing the `Child` trait
- **`F`**: Factory function type that creates new child instances

### Fields

- **`id`**: String identifier for the child (not ChildId UUID)
- **`factory`**: Closure that creates new child instances
- **`restart_policy`**: When to restart the child
- **`shutdown_policy`**: How to shutdown the child
- **`start_timeout`**: Maximum time for child to start
- **`shutdown_timeout`**: Maximum time for child to stop

**Note:** The `significant` field does NOT exist in the implementation.

### Example

```rust
use airssys_rt::supervisor::{ChildSpec, RestartPolicy, ShutdownPolicy};
use std::time::Duration;

let spec = ChildSpec {
    id: "worker-1".into(),
    factory: || MyWorker::new(),
    restart_policy: RestartPolicy::Permanent,
    shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
    start_timeout: Duration::from_secs(10),
    shutdown_timeout: Duration::from_secs(10),
};
```

## ShutdownPolicy

Defines how a child should be shutdown.

```rust
pub enum ShutdownPolicy {
    Graceful(Duration),
    Immediate,
    Infinity,
}
```

### Variants

- **`Graceful(duration)`**: Wait for graceful shutdown, force stop after timeout
- **`Immediate`**: Immediately force stop (not "Brutal")
- **`Infinity`**: Wait indefinitely for graceful shutdown

### Example

```rust
use airssys_rt::supervisor::ShutdownPolicy;
use std::time::Duration;

// Wait 5 seconds for graceful shutdown
let graceful = ShutdownPolicy::Graceful(Duration::from_secs(5));

// Terminate immediately
let immediate = ShutdownPolicy::Immediate;

// Wait forever
let infinity = ShutdownPolicy::Infinity;
```

## ChildHealth

Health status of a supervised child.

```rust
pub enum ChildHealth {
    Healthy,
    Degraded(String),
    Failed(String),
}
```

### Variants

- **`Healthy`**: Child is operating normally
- **`Degraded(message)`**: Child is operational but showing signs of issues
- **`Failed(message)`**: Child has failed and requires restart

### Example

```rust
use airssys_rt::supervisor::{Child, ChildHealth};

#[async_trait]
impl Child for MyWorker {
    type Error = MyError;
    
    async fn health_check(&self) -> ChildHealth {
        if self.error_rate() > 0.5 {
            ChildHealth::Failed("Error rate too high".into())
        } else if self.error_rate() > 0.1 {
            ChildHealth::Degraded("Elevated error rate".into())
        } else {
            ChildHealth::Healthy
        }
    }
    
    // Required methods
    async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
    async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> { Ok(()) }
}
```

## Child Trait

Trait for entities that can be supervised.

```rust
#[async_trait]
pub trait Child: Send + Sync + 'static {
    /// Error type for child lifecycle operations
    type Error: Error + Send + Sync + 'static;

    /// Start the child process
    async fn start(&mut self) -> Result<(), Self::Error>;

    /// Stop the child process gracefully
    /// 
    /// # Parameters
    /// - `timeout`: Maximum time to wait for graceful shutdown
    async fn stop(&mut self, timeout: Duration) -> Result<(), Self::Error>;

    /// Check the health status of the child (optional)
    /// 
    /// Default implementation returns `ChildHealth::Healthy`
    async fn health_check(&self) -> ChildHealth {
        ChildHealth::Healthy
    }
}
```

### Required Methods

- **`start()`**: Initialize and start the child (no timeout parameter)
- **`stop(timeout)`**: Gracefully shutdown with timeout

### Optional Methods

- **`health_check()`**: Report health status (default: Healthy)

**Critical:** The `stop()` method MUST accept a `timeout: Duration` parameter, not zero parameters as shown in some examples.

### Example Implementation

```rust
use airssys_rt::supervisor::{Child, ChildHealth};
use async_trait::async_trait;
use std::time::Duration;

struct DatabaseWorker {
    connected: bool,
}

#[derive(Debug)]
struct WorkerError(String);

impl std::fmt::Display for WorkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for WorkerError {}

#[async_trait]
impl Child for DatabaseWorker {
    type Error = WorkerError;
    
    async fn start(&mut self) -> Result<(), Self::Error> {
        println!("Connecting to database...");
        self.connected = true;
        Ok(())
    }

    async fn stop(&mut self, timeout: Duration) -> Result<(), Self::Error> {
        println!("Disconnecting from database (timeout: {:?})...", timeout);
        self.connected = false;
        Ok(())
    }

    async fn health_check(&self) -> ChildHealth {
        if self.connected {
            ChildHealth::Healthy
        } else {
            ChildHealth::Failed("Not connected".into())
        }
    }
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
use std::time::Duration;

// Define a worker
struct DatabaseWorker {
    connected: bool,
}

#[derive(Debug)]
struct DbError(String);

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DbError {}

#[async_trait]
impl Child for DatabaseWorker {
    type Error = DbError;  // Must specify Error type
    
    async fn start(&mut self) -> Result<(), Self::Error> {
        println!("Connecting to database...");
        self.connected = true;
        Ok(())
    }

    async fn stop(&mut self, timeout: Duration) -> Result<(), Self::Error> {
        println!("Disconnecting from database (timeout: {:?})...", timeout);
        self.connected = false;
        Ok(())
    }

    async fn health_check(&self) -> ChildHealth {
        if self.connected {
            ChildHealth::Healthy
        } else {
            ChildHealth::Failed("Not connected".to_string())
        }
    }
}

// Create supervised system
async fn setup() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut supervisor = SupervisorNode::new(
        SupervisorId::new(),
        OneForOne::new(),
    );

    let spec = ChildSpec {
        id: "db-worker".into(),
        factory: || DatabaseWorker { connected: false },
        restart_policy: RestartPolicy::Permanent,
        shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
        start_timeout: Duration::from_secs(10),
        shutdown_timeout: Duration::from_secs(10),
    };

    supervisor.add_child(spec).await?;
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