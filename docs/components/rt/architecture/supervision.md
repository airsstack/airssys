# Supervisor Trees

The supervision system in `airssys-rt` provides Erlang/OTP-inspired fault tolerance with builder-based configuration and automatic health monitoring.

> **Note**: All code examples are from actual implementation (RT-TASK-013). See [examples directory](../../examples/) for complete working code.

## Architecture Overview

### Design Principles

The supervision system is built on three core concepts:

1. **Child Trait** - Supervised entity contract
2. **Restart Strategies** - Failure propagation control (OneForOne, OneForAll, RestForOne)
3. **Builder Pattern** - Type-safe supervisor configuration (RT-TASK-013)

**Performance Characteristics** (from BENCHMARKING.md):
- **Child spawn**: 5-20 µs per child (builder API)
- **OneForOne restart**: 10-50 µs (stop → start cycle)
- **OneForAll restart**: 30-150 µs for 3 children (~3x OneForOne)
- **Supervision tree creation**: 20-100 µs for supervisor + 3 children

## Child Trait

### Definition

Any entity can be supervised by implementing the `Child` trait (from `src/supervisor/child.rs`):

```rust
#[async_trait]
pub trait Child: Send + Sync {
    async fn start(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn stop(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn health_check(&self) -> ChildHealth;
}
```

**Design Rationale:**
- `start/stop`: Lifecycle control for supervisor restarts
- `health_check`: Automatic monitoring integration
- `Send + Sync`: Multi-threaded supervision support
- Actors automatically implement `Child` via blanket implementation

### Health Status

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChildHealth {
    Healthy,
    Unhealthy(String),  // Reason for unhealthy state
    Unknown,            // Health check failed or not implemented
}
```

### Example Implementation

```rust
use airssys_rt::supervisor::{Child, ChildHealth};

struct DatabaseWorker {
    connection: Option<Connection>,
}

#[async_trait]
impl Child for DatabaseWorker {
    async fn start(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("DatabaseWorker starting...");
        self.connection = Some(Connection::new().await?);
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        println!("DatabaseWorker stopping...");
        if let Some(conn) = self.connection.take() {
            conn.close().await?;
        }
        Ok(())
    }

    async fn health_check(&self) -> ChildHealth {
        match &self.connection {
            Some(conn) if conn.is_alive() => ChildHealth::Healthy,
            Some(_) => ChildHealth::Unhealthy("Connection lost".to_string()),
            None => ChildHealth::Unhealthy("Not connected".to_string()),
        }
    }
}
```

## Restart Strategies

### Strategy Types

Three BEAM-inspired strategies (from `src/supervisor/strategy.rs`):

```rust
pub enum RestartStrategy {
    OneForOne,   // Restart only the failed child
    OneForAll,   // Restart all children when one fails
    RestForOne,  // Restart failed child and those started after it
}
```

### OneForOne Strategy

**Behavior:** Only the failed child is restarted.

**Use case:** Independent children where one failure doesn't affect others.

```
Before failure:        After restart:
┌─────────────┐       ┌─────────────┐
│ Supervisor  │       │ Supervisor  │
└──┬──┬──┬───┘       └──┬──┬──┬───┘
   │  │  │               │  │  │
   A  B  C               A  B' C
      ✗                     ↻
```

**Example:** Worker pool where each worker processes independent tasks.

```rust
use airssys_rt::supervisor::{SupervisorNode, OneForOne};

let mut supervisor = SupervisorNode::builder()
    .with_strategy(OneForOne::new())
    .build()?;
```

See `examples/worker_pool.rs` for complete implementation.

### OneForAll Strategy

**Behavior:** All children are restarted when any child fails.

**Use case:** Dependent children where all must be in consistent state.

```
Before failure:        After restart:
┌─────────────┐       ┌─────────────┐
│ Supervisor  │       │ Supervisor  │
└──┬──┬──┬───┘       └──┬──┬──┬───┘
   │  │  │               │  │  │
   A  B  C               A' B' C'
      ✗                  ↻  ↻  ↻
```

**Example:** Cache, database connection pool, configuration loader that must stay synchronized.

```rust
let mut supervisor = SupervisorNode::builder()
    .with_strategy(OneForAll::new())
    .build()?;
```

### RestForOne Strategy

**Behavior:** Restart failed child and all children started after it.

**Use case:** Pipeline where later stages depend on earlier stages.

```
Before failure:        After restart:
┌─────────────┐       ┌─────────────┐
│ Supervisor  │       │ Supervisor  │
└──┬──┬──┬───┘       └──┬──┬──┬───┘
   │  │  │               │  │  │
   A  B  C               A  B' C'
      ✗                     ↻  ↻
```

**Example:** Event processing pipeline: Collector → Transformer → Writer.

```rust
let mut supervisor = SupervisorNode::builder()
    .with_strategy(RestForOne::new())
    .build()?;
```

See `examples/event_pipeline.rs` for complete RestForOne implementation.

### Performance Comparison

From `benches/supervisor_benchmarks.rs`:

| Strategy | Benchmark | Latency | Notes |
|----------|-----------|---------|-------|
| OneForOne | 1 child restart | 10-50 µs | Baseline |
| OneForAll | 3 children restart | 30-150 µs | ~3x OneForOne |
| RestForOne | 2 children restart | 20-100 µs | Between OneForOne and OneForAll |

**Overhead hierarchy:** OneForOne < RestForOne < OneForAll

## Restart Policies

### Policy Types

Control when children should be restarted (from `src/supervisor/child.rs`):

```rust
pub enum RestartPolicy {
    Permanent,   // Always restart on failure
    Transient,   // Restart only on abnormal termination
    Temporary,   // Never restart
}
```

### Policy Selection Guide

| Policy | Restart Conditions | Use Case |
|--------|-------------------|----------|
| **Permanent** | Always restart | Core services that must run continuously |
| **Transient** | Only on error (not normal shutdown) | Optional services, retryable operations |
| **Temporary** | Never restart | One-shot tasks, event handlers |

### Example Configuration

```rust
use airssys_rt::supervisor::{ChildSpec, RestartPolicy, ShutdownPolicy};

// Critical worker - must always run
let spec_permanent = ChildSpec {
    id: ChildId::new(),
    restart_policy: RestartPolicy::Permanent,
    shutdown_policy: ShutdownPolicy::default(),
    significant: true,  // Failure affects supervisor
};

// Optional background task
let spec_transient = ChildSpec {
    id: ChildId::new(),
    restart_policy: RestartPolicy::Transient,
    shutdown_policy: ShutdownPolicy::default(),
    significant: false,  // Failure doesn't affect supervisor
};

// One-shot initialization
let spec_temporary = ChildSpec {
    id: ChildId::new(),
    restart_policy: RestartPolicy::Temporary,
    shutdown_policy: ShutdownPolicy::default(),
    significant: false,
};
```

## Child Specification

### ChildSpec Structure

Configure supervised children (from `src/supervisor/child.rs`):

```rust
pub struct ChildSpec {
    pub id: ChildId,
    pub restart_policy: RestartPolicy,
    pub shutdown_policy: ShutdownPolicy,
    pub significant: bool,  // Does failure affect supervisor?
}
```

**Fields:**
- `id`: Unique child identifier
- `restart_policy`: When to restart (Permanent/Transient/Temporary)
- `shutdown_policy`: How to stop gracefully
- `significant`: If `true`, child failure propagates to supervisor

### Significance Flag

The `significant` flag controls fault propagation:

```rust
// Significant child - failure propagates up
ChildSpec {
    significant: true,  // Supervisor health depends on this child
    // ...
}

// Non-significant child - failure contained
ChildSpec {
    significant: false,  // Supervisor stays healthy even if child fails
    // ...
}
```

**Use cases:**
- `significant: true` - Core services (database, auth, cache)
- `significant: false` - Optional services (metrics, logs, monitoring)

## Builder Pattern (RT-TASK-013)

### Supervisor Builder

Type-safe supervisor construction with builder API:

```rust
use airssys_rt::supervisor::{SupervisorNode, SupervisorBuilder, OneForOne};

// Create supervisor with builder
let supervisor = SupervisorNode::builder()
    .with_strategy(OneForOne::new())
    .add_child(
        ChildSpec {
            id: ChildId::new(),
            restart_policy: RestartPolicy::Permanent,
            shutdown_policy: ShutdownPolicy::default(),
            significant: true,
        },
        Box::new(MyWorker::new()),
    )
    .add_child(
        ChildSpec {
            id: ChildId::new(),
            restart_policy: RestartPolicy::Transient,
            shutdown_policy: ShutdownPolicy::default(),
            significant: false,
        },
        Box::new(MonitorWorker::new()),
    )
    .build()?;
```

### Builder Methods

```rust
impl SupervisorBuilder {
    pub fn new() -> Self;
    
    pub fn with_strategy<S: RestartStrategy + 'static>(
        mut self, 
        strategy: S
    ) -> Self;
    
    pub fn add_child(
        mut self,
        spec: ChildSpec,
        child: Box<dyn Child>,
    ) -> Self;
    
    pub fn build(self) -> Result<SupervisorNode, SupervisorError>;
}
```

**Benefits:**
- Type-safe configuration (compile-time checks)
- Fluent API (method chaining)
- Minimal overhead (5-20 µs spawn latency)
- Clear child declaration

See `examples/supervisor_builder_phase1.rs` and `examples/supervisor_builder_phase2.rs` for complete examples.

## Supervisor Node

### Structure

Core supervisor implementation (from `src/supervisor/supervisor.rs`):

```rust
pub struct SupervisorNode {
    id: SupervisorId,
    strategy: Box<dyn RestartStrategy>,
    children: Vec<(ChildSpec, Box<dyn Child>)>,
    lifecycle: ActorLifecycle,
}
```

### Key Methods

```rust
impl SupervisorNode {
    // Create with builder (recommended)
    pub fn builder() -> SupervisorBuilder;
    
    // Legacy constructor (still supported)
    pub fn new<S: RestartStrategy + 'static>(
        id: SupervisorId,
        strategy: S,
    ) -> Self;
    
    // Add child at runtime
    pub async fn add_child(
        &mut self,
        spec: ChildSpec,
        child: Box<dyn Child>,
    ) -> Result<(), SupervisorError>;
    
    // Start all children
    pub async fn start_all(&mut self) -> Result<(), SupervisorError>;
    
    // Stop all children
    pub async fn stop_all(&mut self) -> Result<(), SupervisorError>;
    
    // Handle child failure
    pub async fn handle_child_failure(
        &mut self,
        child_id: &ChildId,
    ) -> Result<(), SupervisorError>;
}
```

## Automatic Health Monitoring

### Health Monitoring System

Supervisors can automatically monitor child health (from RT-TASK-009):

```rust
use airssys_rt::monitoring::{HealthMonitor, HealthConfig};

// Create supervisor with health monitoring
let mut supervisor = SupervisorNode::builder()
    .with_strategy(OneForOne::new())
    .build()?;

// Add health monitor
let monitor = HealthMonitor::new(
    HealthConfig {
        check_interval: Duration::from_secs(5),
        unhealthy_threshold: 3,  // Restart after 3 consecutive failures
        auto_restart: true,
    }
);

monitor.monitor_supervisor(&mut supervisor).await?;
```

**Features:**
- Periodic health checks via `Child::health_check()`
- Configurable thresholds
- Automatic restart on unhealthy children
- Health status aggregation

See `examples/supervisor_automatic_health.rs` for complete implementation.

## Supervision Trees

### Hierarchical Supervisors

Supervisors can supervise other supervisors:

```
┌─────────────────────────┐
│  Root Supervisor        │
│  (OneForAll)            │
└──┬──────────────────┬───┘
   │                  │
   │                  │
┌──▼─────────┐   ┌───▼──────────┐
│ Worker Pool│   │ Cache Manager│
│ Supervisor │   │ Supervisor   │
│ (OneForOne)│   │ (OneForAll)  │
└──┬──┬──┬──┘   └──┬───────┬───┘
   │  │  │         │       │
   W1 W2 W3     Cache   Persistence
```

**Example:**

```rust
// Create worker pool supervisor
let worker_supervisor = SupervisorNode::builder()
    .with_strategy(OneForOne::new())
    .add_child(spec1, Box::new(Worker1::new()))
    .add_child(spec2, Box::new(Worker2::new()))
    .build()?;

// Create cache supervisor
let cache_supervisor = SupervisorNode::builder()
    .with_strategy(OneForAll::new())
    .add_child(cache_spec, Box::new(Cache::new()))
    .add_child(db_spec, Box::new(Database::new()))
    .build()?;

// Create root supervisor (supervisors implement Child trait)
let root_supervisor = SupervisorNode::builder()
    .with_strategy(OneForAll::new())
    .add_child(
        worker_spec,
        Box::new(worker_supervisor),
    )
    .add_child(
        cache_spec,
        Box::new(cache_supervisor),
    )
    .build()?;
```

## Error Handling

### Supervisor Errors

```rust
#[derive(Debug)]
pub enum SupervisorError {
    ChildStartFailed(ChildId, Box<dyn Error + Send + Sync>),
    ChildStopFailed(ChildId, Box<dyn Error + Send + Sync>),
    ChildNotFound(ChildId),
    StrategyError(String),
}
```

### Error Flow

1. Child fails during operation
2. Child's `start()` or `stop()` returns `Err`
3. Supervisor catches error
4. Supervisor applies restart strategy
5. On restart failure, error propagates to parent supervisor (if significant child)

### Recovery Strategies

```rust
async fn handle_child_failure(
    &mut self,
    child_id: &ChildId,
) -> Result<(), SupervisorError> {
    match self.restart_policy {
        RestartPolicy::Permanent => {
            // Always restart
            self.restart_child(child_id).await?;
        }
        RestartPolicy::Transient => {
            // Check if abnormal termination
            if self.is_abnormal_termination(child_id) {
                self.restart_child(child_id).await?;
            }
        }
        RestartPolicy::Temporary => {
            // Just cleanup, no restart
            self.remove_child(child_id).await?;
        }
    }
    Ok(())
}
```

## Best Practices

### Strategy Selection

**OneForOne:**
- ✅ Use for: Independent workers, pools, parallel tasks
- ✅ Example: HTTP request handlers, background jobs
- ❌ Avoid for: Stateful, interdependent services

**OneForAll:**
- ✅ Use for: Tightly coupled services, consistent state requirements
- ✅ Example: Cache + database, auth + session store
- ❌ Avoid for: Large numbers of independent children (restart overhead)

**RestForOne:**
- ✅ Use for: Pipelines, sequential dependencies
- ✅ Example: Data ingestion → transform → storage
- ❌ Avoid for: Parallel, independent processing

### Child Design

**DO:**
- ✅ Implement proper `start()`/`stop()` lifecycle
- ✅ Make `health_check()` fast and accurate (<1ms)
- ✅ Use `significant` flag appropriately
- ✅ Keep child state minimal for fast restarts

**DON'T:**
- ❌ Block in `start()`/`stop()` (use async properly)
- ❌ Ignore errors in lifecycle methods
- ❌ Make all children significant (limits fault isolation)
- ❌ Store unrecoverable state in children

### Performance Tuning

**Minimize restart latency:**
- Keep `start()` logic simple (<10ms ideal)
- Preallocate resources where possible
- Use connection pools instead of per-child connections

**Monitor restart patterns:**
- Track restart counts (via `ActorLifecycle`)
- Alert on excessive restarts (possible underlying issue)
- Use `HealthMonitor` for automatic detection

## Testing Patterns

### Unit Testing Children

```rust
#[tokio::test]
async fn test_child_lifecycle() {
    let mut child = MyWorker::new();
    
    // Test start
    child.start().await.expect("start failed");
    assert_eq!(child.health_check().await, ChildHealth::Healthy);
    
    // Test stop
    child.stop().await.expect("stop failed");
}
```

### Integration Testing Supervisors

```rust
#[tokio::test]
async fn test_one_for_one_restart() {
    let mut supervisor = SupervisorNode::builder()
        .with_strategy(OneForOne::new())
        .add_child(spec, Box::new(FailingWorker::new()))
        .build()
        .unwrap();
    
    supervisor.start_all().await.unwrap();
    
    // Trigger failure and verify restart
    let child_id = supervisor.children()[0].0.id.clone();
    supervisor.handle_child_failure(&child_id).await.unwrap();
    
    // Verify child was restarted
    assert!(supervisor.is_child_running(&child_id));
}
```

## Working Examples

Explore supervision in these examples:

| Example | Demonstrates | Command |
|---------|--------------|---------|
| `supervisor_basic.rs` | Basic supervision setup | `cargo run --example supervisor_basic` |
| `supervisor_strategies.rs` | All three strategies | `cargo run --example supervisor_strategies` |
| `supervisor_builder_phase1.rs` | Builder API basics | `cargo run --example supervisor_builder_phase1` |
| `supervisor_builder_phase2.rs` | Advanced builder patterns | `cargo run --example supervisor_builder_phase2` |
| `supervisor_automatic_health.rs` | Health monitoring | `cargo run --example supervisor_automatic_health` |
| `worker_pool.rs` | Production worker pool | `cargo run --example worker_pool` |
| `event_pipeline.rs` | RestForOne pipeline | `cargo run --example event_pipeline` |

All examples are in the `examples/` directory with complete, runnable implementations.
