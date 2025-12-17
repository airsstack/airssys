# Supervisor Patterns Guide

This guide teaches you how to build fault-tolerant systems using supervision trees. You'll learn the "let it crash" philosophy, restart strategies, supervision hierarchies, and health monitoring integration.

**Prerequisites:**

- Completed [Getting Started](../implementation/getting-started.md)
- Understanding of basic Rust async programming
- Familiarity with error handling patterns

**What You'll Learn:**

- "Let it crash" philosophy and when to use it
- Restart strategies (OneForOne, OneForAll, RestForOne)
- Supervision tree patterns (flat, hierarchical)
- Child specification and factory patterns
- Health monitoring integration

**Note:** This guide documents the current supervisor API (RT-TASK-009). A builder pattern API (RT-TASK-013) is planned for future release.

---

## 1. Supervision Philosophy

### The "Let It Crash" Approach

Instead of defensive programming with extensive error handling, let actors fail and rely on supervisors to restart them with clean state.

**Traditional Approach (Defensive):**
```rust
// ❌ Overly defensive - cluttered with error handling
async fn handle_message(&mut self, msg: Message) -> Result<()> {
    if let Some(connection) = &self.connection {
        if connection.is_valid() {
            if let Ok(data) = connection.read().await {
                if data.is_valid() {
                    self.process(data)?;
                } else {
                    self.reconnect()?;
                }
            } else {
                self.reconnect()?;
            }
        } else {
            self.reconnect()?;
        }
    } else {
        self.connect()?;
    }
    Ok(())
}
```

**Supervision Approach (Let It Crash):**
```rust
use airssys_rt::supervisor::Child;
use async_trait::async_trait;

// ✅ Simple - let supervisor handle failures
struct Worker {
    connection: Option<Connection>,
}

#[async_trait]
impl Child for Worker {
    type Error = WorkerError;
    
    async fn start(&mut self) -> Result<(), Self::Error> {
        // Supervisor ensures we always start with fresh connection
        self.connection = Some(Connection::new().await?);
        Ok(())
    }
    
    async fn stop(&mut self, timeout: Duration) -> Result<(), Self::Error> {
        if let Some(conn) = &self.connection {
            conn.close().await?;
        }
        Ok(())
    }
}

// Main processing logic - if this fails, supervisor restarts us
async fn process_work(&mut self) -> Result<()> {
    let connection = self.connection.as_ref()
        .ok_or(Error::NotConnected)?;
    
    let data = connection.read().await?;
    self.process(data)?;  // If this fails, supervisor restarts us
    Ok(())
}
```

**Benefits:**

- **Simpler code**: Less error handling clutter
- **Clean state**: Restart gives fresh state
- **Fault isolation**: Failures don't cascade
- **Self-healing**: System automatically recovers

### When to Use Supervisors vs Defensive Programming

**Use Supervisors When:**

- Errors indicate corrupted state (restart needed)
- External dependencies fail (network, database)
- Resource exhaustion (memory, file handles)
- Recovery requires reinitialization

**Use Defensive Programming When:**

- Expected errors (user input validation)
- Recoverable conditions (retry-able operations)
- Performance-critical paths (avoid restart overhead)
- Errors don't indicate state corruption

**Example Decision Tree:**
```
Error Occurred
    ├─ Is state corrupted? 
    │   └─ YES → Let it crash (supervisor restart)
    │
    ├─ Is it a temporary failure?
    │   └─ YES → Retry with backoff
    │
    ├─ Is it expected/valid input?
    │   └─ YES → Handle defensively
    │
    └─ Is it a resource issue?
        └─ YES → Let it crash (supervisor restart)
```

### Fault Isolation Through Supervision Trees

Supervision trees prevent cascading failures by isolating faults:

```
                   Root Supervisor
                         │
        ┌────────────────┼────────────────┐
        │                │                │
   WebServer        Database          Cache
   Supervisor       Supervisor      Supervisor
        │                │                │
    ┌───┴───┐        ┌───┴───┐      ┌───┴───┐
Worker Worker    Conn  Conn      Read  Write
 Pool  Pool     Pool  Pool      Cache Cache
```

**Isolation Benefits:**

- Web server failure doesn't affect database
- Individual worker failure doesn't crash server
- Cache failure doesn't break core functionality

---

## 2. Restart Strategies in Practice

### OneForOne: Independent Workers

**Use When:**

- Workers are independent
- One failure shouldn't affect others
- Examples: HTTP request handlers, background jobs

**Pattern:**
```rust
use airssys_rt::prelude::*;

// Independent worker actors
struct HttpWorker {
    request_count: u64,
}

// Supervisor with OneForOne strategy
let supervisor = SupervisorNode::new(
    "http-workers",
    OneForOne,  // Each worker restarts independently
    RestartPolicy::Permanent,  // Always restart
);

// Spawn multiple independent workers
for i in 0..10 {
    supervisor.spawn_child(
        format!("worker-{}", i),
        HttpWorker { request_count: 0 },
    ).await?;
}
```

**Behavior:**

- Worker-3 crashes → Only Worker-3 restarts
- Other workers continue unaffected
- No cascading failures

**Real-World Example: HTTP Server**
```rust
struct RequestHandler {
    id: usize,
    processed: u64,
}

impl Actor for RequestHandler {
    type Message = HttpRequest;
    type Error = HandlerError;
    
    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        request: Self::Message,
        _ctx: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        // Process request
        let response = self.process_request(request)?;
        
        self.processed += 1;
        Ok(())
    }
}

// Setup supervisor with OneForOne
let http_supervisor = SupervisorNode::new(
    "http-server",
    OneForOne,  // Independent request handlers
    RestartPolicy::Permanent,
);

// Spawn worker pool
for id in 0..num_cpus::get() {
    http_supervisor.spawn_child(
        format!("handler-{}", id),
        RequestHandler { id, processed: 0 },
    ).await?;
}
```

### OneForAll: Tightly Coupled Services

**Use When:**

- Services depend on each other
- Inconsistent state if one fails
- Examples: Transaction processors, coordinated caches

**Pattern:**
```rust
// Tightly coupled services
struct OrderProcessor { /* ... */ }
struct InventoryManager { /* ... */ }
struct PaymentGateway { /* ... */ }

// Supervisor with OneForAll strategy
let supervisor = SupervisorNode::new(
    "transaction-services",
    OneForAll,  // All services restart together
    RestartPolicy::Permanent,
);

supervisor.spawn_child("orders", OrderProcessor::new()).await?;
supervisor.spawn_child("inventory", InventoryManager::new()).await?;
supervisor.spawn_child("payment", PaymentGateway::new()).await?;
```

**Behavior:**

- Payment gateway crashes → All three services restart
- Ensures consistent state across services
- Prevents partial transaction state

**Real-World Example: Trading System**
```rust
struct MarketDataFeed { positions: HashMap<String, Position> }
struct RiskCalculator { limits: HashMap<String, Limit> }
struct OrderExecutor { pending: Vec<Order> }

// All must be consistent - restart together
let trading_supervisor = SupervisorNode::new(
    "trading-system",
    OneForAll,  // Restart all on any failure
    RestartPolicy::Permanent,
);

trading_supervisor.spawn_child("market-data", MarketDataFeed::new()).await?;
trading_supervisor.spawn_child("risk-calc", RiskCalculator::new()).await?;
trading_supervisor.spawn_child("executor", OrderExecutor::new()).await?;
```

### RestForOne: Pipeline/Sequential Dependencies

**Use When:**

- Services form a pipeline
- Later stages depend on earlier ones
- Examples: Data processing pipelines, message queues

**Pattern:**
```rust
// Pipeline stages
struct DataIngestion { /* ... */ }
struct DataValidation { /* ... */ }
struct DataTransform { /* ... */ }
struct DataStorage { /* ... */ }

// Supervisor with RestForOne strategy
let supervisor = SupervisorNode::new(
    "data-pipeline",
    RestForOne,  // Restart this and following children
    RestartPolicy::Permanent,
);

// Order matters! Earlier stages first
supervisor.spawn_child("ingestion", DataIngestion::new()).await?;
supervisor.spawn_child("validation", DataValidation::new()).await?;
supervisor.spawn_child("transform", DataTransform::new()).await?;
supervisor.spawn_child("storage", DataStorage::new()).await?;
```

**Behavior:**

- Validation crashes → Restart validation, transform, storage
- Ingestion keeps running (not affected)
- Transform crashes → Restart only transform and storage
- Maintains pipeline order

**Real-World Example: ETL Pipeline**
```rust
struct Extractor { source: DataSource }
struct Transformer { rules: Vec<Rule> }
struct Loader { destination: Database }

// Sequential dependency: Extract → Transform → Load
let etl_supervisor = SupervisorNode::new(
    "etl-pipeline",
    RestForOne,  // Pipeline restart semantics
    RestartPolicy::Transient,  // Only restart on error
);

etl_supervisor.spawn_child("extractor", Extractor::new()).await?;
etl_supervisor.spawn_child("transformer", Transformer::new()).await?;
etl_supervisor.spawn_child("loader", Loader::new()).await?;
```

### Strategy Selection Decision Tree

```
What relationship do children have?

├─ Independent workers?
│   └─ Use OneForOne
│       • Web request handlers
│       • Background jobs
│       • Worker pools
│
├─ Tightly coupled/consistent state?
│   └─ Use OneForAll
│       • Transaction processors
│       • Coordinated caches
│       • Trading systems
│
└─ Sequential pipeline?
    └─ Use RestForOne
        • Data processing stages
        • Message queues
        • ETL pipelines
```

---

## 3. Supervision Tree Patterns

### Flat Supervision

**Pattern:** Single supervisor, many workers

```
     Supervisor
         │
    ┌────┼────┬────┬────┐
    W1   W2   W3   W4   W5
```

**Use When:**

- Simple worker pools
- All workers same type
- No worker dependencies

**Example:**
```rust
let supervisor = SupervisorNode::new(
    "worker-pool",
    OneForOne,
    RestartPolicy::Permanent,
);

// Flat structure - all workers at same level
for i in 0..10 {
    supervisor.spawn_child(
        format!("worker-{}", i),
        Worker::new(i),
    ).await?;
}
```

**Pros:**

- Simple to understand
- Easy to manage
- Low overhead

**Cons:**

- No subsystem isolation
- All failures handled same way
- Doesn't scale to complex systems

### Hierarchical Supervision

**Pattern:** Supervisor of supervisors

```
        Root Supervisor
              │
      ┌───────┼───────┐
  SubSup-A  SubSup-B  SubSup-C
      │         │         │
   ┌──┴──┐   ┌─┴─┐    ┌─┴─┐
   W1   W2   W3  W4   W5  W6
```

**Use When:**

- Multiple subsystems
- Different restart policies per subsystem
- Need fault isolation between components

**Example:**
```rust
// Root supervisor
let root = SupervisorNode::new(
    "application",
    OneForAll,  // Restart all subsystems if root fails
    RestartPolicy::Permanent,
);

// Web subsystem
let web_supervisor = SupervisorNode::new(
    "web-subsystem",
    OneForOne,  // Independent workers
    RestartPolicy::Permanent,
);
for i in 0..5 {
    web_supervisor.spawn_child(
        format!("http-worker-{}", i),
        HttpWorker::new(),
    ).await?;
}

// Database subsystem
let db_supervisor = SupervisorNode::new(
    "db-subsystem",
    RestForOne,  // Connection pool dependency
    RestartPolicy::Permanent,
);
db_supervisor.spawn_child("conn-pool", ConnectionPool::new()).await?;
db_supervisor.spawn_child("query-executor", QueryExecutor::new()).await?;

// Cache subsystem
let cache_supervisor = SupervisorNode::new(
    "cache-subsystem",
    OneForAll,  // Cache coherency
    RestartPolicy::Transient,
);
cache_supervisor.spawn_child("read-cache", ReadCache::new()).await?;
cache_supervisor.spawn_child("write-cache", WriteCache::new()).await?;

// Add subsystems to root
root.add_supervisor(web_supervisor).await?;
root.add_supervisor(db_supervisor).await?;
root.add_supervisor(cache_supervisor).await?;
```

**Pros:**

- Subsystem isolation
- Different policies per level
- Scales to large systems
- Clear component boundaries

**Cons:**

- More complex
- Higher overhead
- Requires design planning

### Mixed Strategies

**Pattern:** Different strategies at different levels

```
     Root (OneForAll)
          │
    ┌─────┼─────┐
API (OneForOne) | DB (RestForOne)
    │           │
 ┌──┴──┐     ┌──┴──┐
 W   W       Pool Exec
```

**Example:**
```rust
// Root: All subsystems must be consistent
let root = SupervisorNode::new(
    "app-root",
    OneForAll,  // Restart all on critical failure
    RestartPolicy::Permanent,
);

// API layer: Independent request handlers
let api_supervisor = SupervisorNode::new(
    "api-layer",
    OneForOne,  // Workers independent
    RestartPolicy::Permanent,
);

// Database layer: Sequential dependency
let db_supervisor = SupervisorNode::new(
    "db-layer",
    RestForOne,  // Pool → Executor dependency
    RestartPolicy::Permanent,
);

root.add_supervisor(api_supervisor).await?;
root.add_supervisor(db_supervisor).await?;
```

### Real-World Example: Microservice

```rust
use airssys_rt::prelude::*;

async fn build_microservice() -> Result<SupervisorNode, Box<dyn std::error::Error>> {
    // Root supervisor
    let root = SupervisorNode::new(
        "microservice",
        OneForAll,
        RestartPolicy::Permanent,
    );
    
    // HTTP API layer (independent workers)
    let http_supervisor = SupervisorNode::new(
        "http-api",
        OneForOne,
        RestartPolicy::Permanent,
    );
    for i in 0..num_cpus::get() {
        http_supervisor.spawn_child(
            format!("handler-{}", i),
            RequestHandler::new(i),
        ).await?;
    }
    
    // Business logic layer (stateful, coordinated)
    let logic_supervisor = SupervisorNode::new(
        "business-logic",
        OneForAll,  // Must be consistent
        RestartPolicy::Permanent,
    );
    logic_supervisor.spawn_child("order-service", OrderService::new()).await?;
    logic_supervisor.spawn_child("inventory-service", InventoryService::new()).await?;
    
    // Data layer (pipeline)
    let data_supervisor = SupervisorNode::new(
        "data-layer",
        RestForOne,  // Connection → Query dependency
        RestartPolicy::Permanent,
    );
    data_supervisor.spawn_child("connection-pool", ConnectionPool::new()).await?;
    data_supervisor.spawn_child("query-executor", QueryExecutor::new()).await?;
    data_supervisor.spawn_child("cache-manager", CacheManager::new()).await?;
    
    // Assemble hierarchy
    root.add_supervisor(http_supervisor).await?;
    root.add_supervisor(logic_supervisor).await?;
    root.add_supervisor(data_supervisor).await?;
    
    Ok(root)
}
```

---

## 4. Builder Pattern Usage (RT-TASK-013)

The builder pattern simplifies supervisor configuration.

### Migrating from Manual ChildSpec

**Old way (manual ChildSpec):**
```rust
let spec = ChildSpec {
    id: ChildId::new(),
    name: "worker-1".to_string(),
    restart_policy: RestartPolicy::Permanent,
    shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
};
supervisor.spawn_with_spec(spec, Worker::new()).await?;
```

**New way (builder pattern):**
```rust
supervisor
    .child("worker-1")
    .restart_policy(RestartPolicy::Permanent)
    .shutdown_timeout(Duration::from_secs(5))
    .spawn(Worker::new())
    .await?;
```

### Single Child Spawning

```rust
use airssys_rt::prelude::*;

let supervisor = SupervisorNode::new(
    "my-supervisor",
    OneForOne,
    RestartPolicy::Permanent,
);

// Simple spawn with defaults
supervisor
    .child("worker-1")
    .spawn(Worker::new())
    .await?;

// Custom configuration
supervisor
    .child("worker-2")
    .restart_policy(RestartPolicy::Transient)
    .shutdown_timeout(Duration::from_secs(10))
    .health_check_interval(Duration::from_secs(30))
    .spawn(Worker::new())
    .await?;
```

### Batch Spawning

```rust
// Spawn multiple workers with same config
supervisor
    .children("worker", 10)  // Creates worker-0 through worker-9
    .restart_policy(RestartPolicy::Permanent)
    .spawn_batch(|| Worker::new())
    .await?;

// Spawn with custom initialization
supervisor
    .children("handler", 5)
    .spawn_batch_with(|index| HttpHandler::new(index))
    .await?;
```

### Common Configurations

**Permanent Workers (always restart):**
```rust
supervisor
    .child("critical-service")
    .restart_policy(RestartPolicy::Permanent)
    .spawn(Service::new())
    .await?;
```

**Transient Workers (restart only on error):**
```rust
supervisor
    .child("task-processor")
    .restart_policy(RestartPolicy::Transient)
    .spawn(TaskProcessor::new())
    .await?;
```

**Temporary Workers (never restart):**
```rust
supervisor
    .child("one-time-job")
    .restart_policy(RestartPolicy::Temporary)
    .spawn(Job::new())
    .await?;
```

---

## 5. Health Monitoring Integration (RT-TASK-010)

Supervisors can integrate with the monitoring system for proactive health checks.

### Automatic Health Checks

```rust
use airssys_rt::prelude::*;
use std::time::Duration;

let supervisor = SupervisorNode::new(
    "monitored-workers",
    OneForOne,
    RestartPolicy::Permanent,
);

// Enable automatic health monitoring
supervisor
    .child("worker-1")
    .health_check_interval(Duration::from_secs(10))  // Check every 10s
    .health_check_timeout(Duration::from_secs(2))    // Timeout after 2s
    .unhealthy_threshold(3)                          // 3 failures → restart
    .spawn(Worker::new())
    .await?;
```

### Custom Health Check Logic

```rust
use airssys_rt::monitoring::{HealthCheck, HealthStatus};

struct DatabaseWorker {
    connection: Option<Connection>,
}

#[async_trait]
impl HealthCheck for DatabaseWorker {
    async fn check_health(&self) -> HealthStatus {
        match &self.connection {
            Some(conn) if conn.is_alive() => HealthStatus::Healthy,
            Some(_) => HealthStatus::Degraded("Connection stale".into()),
            None => HealthStatus::Unhealthy("No connection".into()),
        }
    }
}

// Supervisor will automatically restart if unhealthy
supervisor
    .child("db-worker")
    .health_check_interval(Duration::from_secs(5))
    .spawn(DatabaseWorker { connection: None })
    .await?;
```

### Threshold Configuration

```rust
// Conservative: Restart only after multiple failures
supervisor
    .child("stable-service")
    .unhealthy_threshold(5)  // 5 consecutive failures
    .health_check_interval(Duration::from_secs(30))
    .spawn(Service::new())
    .await?;

// Aggressive: Restart quickly on any issue
supervisor
    .child("critical-service")
    .unhealthy_threshold(1)  // Restart immediately
    .health_check_interval(Duration::from_secs(5))
    .spawn(CriticalService::new())
    .await?;
```

### Proactive vs Reactive Monitoring

**Reactive Monitoring (traditional):**

- Wait for errors
- React to failures
- Downtime during recovery

**Proactive Monitoring (health checks):**
```rust
struct ApiWorker {
    last_request: Instant,
    error_count: u32,
}

#[async_trait]
impl HealthCheck for ApiWorker {
    async fn check_health(&self) -> HealthStatus {
        // Proactive checks
        if self.last_request.elapsed() > Duration::from_secs(300) {
            return HealthStatus::Degraded("No recent requests".into());
        }
        
        if self.error_count > 10 {
            return HealthStatus::Degraded("High error rate".into());
        }
        
        HealthStatus::Healthy
    }
}

// Supervisor restarts before complete failure
supervisor
    .child("api-worker")
    .health_check_interval(Duration::from_secs(10))
    .unhealthy_threshold(2)
    .spawn(ApiWorker::new())
    .await?;
```

**Benefits:**

- Detect degradation early
- Restart before complete failure
- Minimize downtime
- Better user experience

---

## Next Steps

Congratulations! You now understand supervision patterns deeply. Continue your learning:

### 📨 **Master Message Patterns**
- [Message Passing Guide](./message-passing.md) - Communication patterns and optimization

### 🎯 **Build Production Systems**
- [Monitoring Guide](./monitoring.md) - Observability and metrics
- [Performance Guide](../performance/optimization.md) - Tuning for production

### 🏗️ **Advanced Architecture**
- [Distributed Supervision](../architecture/distributed.md) - Multi-node supervision
- [Fault Tolerance Patterns](../architecture/fault-tolerance.md) - Production-ready resilience

---

## Summary

✅ **"Let It Crash" Philosophy**: Simple code, supervisors handle recovery  
✅ **Restart Strategies**: OneForOne, OneForAll, RestForOne selection  
✅ **Supervision Trees**: Flat, hierarchical, mixed patterns  
✅ **Builder Pattern**: Simplified configuration (RT-TASK-013)  
✅ **Health Monitoring**: Proactive checks and automatic recovery (RT-TASK-010)  

You're now ready to build resilient, self-healing systems with AirsSys-RT!
