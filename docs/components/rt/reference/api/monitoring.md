# Monitoring API Reference

This reference documents the monitoring and health check system for actors and supervisors.

## Module: `monitoring`

Health monitoring and metrics collection for actors.

### Trait: `HealthCheck`

```rust
pub trait HealthCheck: Send + Sync {
    async fn check_health(&self, actor_id: ActorId) -> HealthStatus;
    fn health_check_interval(&self) -> Duration;
    fn health_check_timeout(&self) -> Duration;
}
```

Trait for implementing health checks on actors.

**Required Methods:**

- `check_health()`: Performs health check and returns status
- `health_check_interval()`: How often to perform checks
- `health_check_timeout()`: Maximum time to wait for response

**Trait Bounds:**

- `Send + Sync`: Can be safely shared across threads

**Example:**

```rust
use airssys_rt::monitoring::{HealthCheck, HealthStatus};
use airssys_rt::util::ActorId;
use std::time::Duration;

struct PingHealthCheck;

#[async_trait::async_trait]
impl HealthCheck for PingHealthCheck {
    async fn check_health(&self, actor_id: ActorId) -> HealthStatus {
        // Send ping message to actor
        match send_ping(actor_id).await {
            Ok(()) => HealthStatus::Healthy,
            Err(e) => HealthStatus::Unhealthy {
                reason: format!("Ping failed: {}", e),
            },
        }
    }
    
    fn health_check_interval(&self) -> Duration {
        Duration::from_secs(30)
    }
    
    fn health_check_timeout(&self) -> Duration {
        Duration::from_secs(5)
    }
}
```

### Enum: `HealthStatus`

```rust
pub enum HealthStatus {
    Healthy,
    Degraded {
        reason: String,
    },
    Unhealthy {
        reason: String,
    },
    Unknown,
}
```

Represents the health state of an actor.

**Variants:**

- `Healthy`: Actor is functioning normally
- `Degraded { reason }`: Actor is operational but with reduced performance
- `Unhealthy { reason }`: Actor is not functioning correctly
- `Unknown`: Health status cannot be determined

**State Transitions:**

```
Unknown -> Healthy       (successful health check)
Unknown -> Unhealthy     (failed health check)
Healthy -> Degraded      (performance issue detected)
Healthy -> Unhealthy     (critical failure)
Degraded -> Healthy      (issue resolved)
Degraded -> Unhealthy    (issue worsened)
Unhealthy -> Degraded    (partial recovery)
Unhealthy -> Healthy     (full recovery)
```

**Example:**

```rust
use airssys_rt::monitoring::HealthStatus;

fn handle_health_status(status: HealthStatus) {
    match status {
        HealthStatus::Healthy => {
            println!("✓ Actor is healthy");
        }
        HealthStatus::Degraded { reason } => {
            eprintln!("⚠ Actor degraded: {}", reason);
            // Maybe reduce load
        }
        HealthStatus::Unhealthy { reason } => {
            eprintln!("✗ Actor unhealthy: {}", reason);
            // Trigger restart or failover
        }
        HealthStatus::Unknown => {
            eprintln!("? Actor health unknown");
            // Retry health check
        }
    }
}
```

## Health Monitoring

### Struct: `HealthMonitor`

```rust
pub struct HealthMonitor {
    // fields omitted
}
```

Monitors actor health and triggers recovery actions.

**Features:**

- Periodic health checks
- Automatic unhealthy actor detection
- Integration with supervisor for restarts
- Configurable check intervals and timeouts
- Health history tracking

#### Constructors

##### `new()`

```rust
pub fn new(system: Arc<ActorSystem>) -> Self
```

Creates a new health monitor.

**Parameters:**

- `system`: The actor system to monitor

**Example:**

```rust
use airssys_rt::monitoring::HealthMonitor;
use std::sync::Arc;

let monitor = HealthMonitor::new(Arc::clone(&system));
```

##### `with_config()`

```rust
pub fn with_config(system: Arc<ActorSystem>, config: HealthMonitorConfig) -> Self
```

Creates a health monitor with custom configuration.

**Parameters:**

- `system`: Actor system to monitor
- `config`: Health monitor configuration

**Example:**

```rust
use airssys_rt::monitoring::{HealthMonitor, HealthMonitorConfig};
use std::time::Duration;

let config = HealthMonitorConfig {
    check_interval: Duration::from_secs(10),
    check_timeout: Duration::from_secs(2),
    failure_threshold: 3,
    recovery_threshold: 2,
};

let monitor = HealthMonitor::with_config(Arc::clone(&system), config);
```

#### Methods

##### `monitor_actor()`

```rust
pub async fn monitor_actor<H>(&self, actor_id: ActorId, health_check: H)
where
    H: HealthCheck + 'static,
```

Starts monitoring an actor with a custom health check.

**Type Parameters:**

- `H`: The health check implementation

**Parameters:**

- `actor_id`: Actor to monitor
- `health_check`: Health check implementation

**Behavior:**

- Spawns background task for periodic checks
- Continues until actor stops or monitor is stopped
- Reports status changes to supervisor (if supervised)

**Example:**

```rust
use airssys_rt::monitoring::PingHealthCheck;

monitor.monitor_actor(actor_id, PingHealthCheck).await;
```

##### `stop_monitoring()`

```rust
pub fn stop_monitoring(&self, actor_id: ActorId) -> Result<(), MonitoringError>
```

Stops monitoring an actor.

**Parameters:**

- `actor_id`: Actor to stop monitoring

**Returns:**

- `Ok(())`: Monitoring stopped successfully
- `Err(MonitoringError::NotMonitored)`: Actor was not being monitored

**Example:**

```rust
monitor.stop_monitoring(actor_id)?;
```

##### `get_health_status()`

```rust
pub fn get_health_status(&self, actor_id: ActorId) -> Option<HealthStatus>
```

Gets the current health status of an actor.

**Parameters:**

- `actor_id`: Actor to query

**Returns:**

- `Some(HealthStatus)`: Current health status
- `None`: Actor not being monitored

**Example:**

```rust
if let Some(status) = monitor.get_health_status(actor_id) {
    println!("Actor health: {:?}", status);
}
```

##### `get_health_history()`

```rust
pub fn get_health_history(&self, actor_id: ActorId, limit: usize) -> Vec<HealthRecord>
```

Gets the health check history for an actor.

**Parameters:**

- `actor_id`: Actor to query
- `limit`: Maximum number of records to return

**Returns:**

- `Vec<HealthRecord>`: Health check history (most recent first)

**Example:**

```rust
let history = monitor.get_health_history(actor_id, 10);
for record in history {
    println!("{}: {:?}", record.timestamp, record.status);
}
```

## Health Monitoring Configuration

### Struct: `HealthMonitorConfig`

```rust
pub struct HealthMonitorConfig {
    pub check_interval: Duration,
    pub check_timeout: Duration,
    pub failure_threshold: u32,
    pub recovery_threshold: u32,
}
```

Configuration for health monitoring behavior.

**Fields:**

- `check_interval`: Time between health checks
- `check_timeout`: Maximum time to wait for health check response
- `failure_threshold`: Number of consecutive failures before marking unhealthy
- `recovery_threshold`: Number of consecutive successes before marking healthy

**Default Values:**

```rust
impl Default for HealthMonitorConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            check_timeout: Duration::from_secs(5),
            failure_threshold: 3,
            recovery_threshold: 2,
        }
    }
}
```

**Example:**

```rust
use airssys_rt::monitoring::HealthMonitorConfig;
use std::time::Duration;

// Aggressive monitoring for critical service
let critical_config = HealthMonitorConfig {
    check_interval: Duration::from_secs(5),
    check_timeout: Duration::from_secs(1),
    failure_threshold: 2,
    recovery_threshold: 3,
};

// Relaxed monitoring for background worker
let worker_config = HealthMonitorConfig {
    check_interval: Duration::from_secs(60),
    check_timeout: Duration::from_secs(10),
    failure_threshold: 5,
    recovery_threshold: 2,
};
```

## Built-in Health Checks

### Struct: `PingHealthCheck`

```rust
pub struct PingHealthCheck;
```

Simple ping-based health check.

**Behavior:**

- Sends ping message to actor
- Expects pong response within timeout
- Marks healthy if response received

**Example:**

```rust
use airssys_rt::monitoring::PingHealthCheck;

monitor.monitor_actor(actor_id, PingHealthCheck).await;
```

### Struct: `MessageRateHealthCheck`

```rust
pub struct MessageRateHealthCheck {
    pub min_messages_per_sec: f64,
}
```

Health check based on message processing rate.

**Fields:**

- `min_messages_per_sec`: Minimum expected message processing rate

**Behavior:**

- Tracks actor's message processing rate
- Marks degraded if below minimum rate
- Marks unhealthy if processing stopped

**Example:**

```rust
use airssys_rt::monitoring::MessageRateHealthCheck;

let health_check = MessageRateHealthCheck {
    min_messages_per_sec: 100.0,
};

monitor.monitor_actor(worker_id, health_check).await;
```

### Struct: `MemoryHealthCheck`

```rust
pub struct MemoryHealthCheck {
    pub max_memory_mb: usize,
}
```

Health check based on actor memory usage.

**Fields:**

- `max_memory_mb`: Maximum acceptable memory usage in MB

**Behavior:**

- Monitors actor's memory footprint
- Marks degraded if approaching limit (>80%)
- Marks unhealthy if exceeding limit

**Example:**

```rust
use airssys_rt::monitoring::MemoryHealthCheck;

let health_check = MemoryHealthCheck {
    max_memory_mb: 100,  // 100 MB limit
};

monitor.monitor_actor(actor_id, health_check).await;
```

### Struct: `CompositeHealthCheck`

```rust
pub struct CompositeHealthCheck {
    checks: Vec<Box<dyn HealthCheck>>,
}
```

Combines multiple health checks with AND logic.

**Behavior:**

- Runs all health checks in parallel
- Healthy only if all checks are healthy
- Degraded if any check is degraded
- Unhealthy if any check is unhealthy

**Example:**

```rust
use airssys_rt::monitoring::{CompositeHealthCheck, PingHealthCheck, MessageRateHealthCheck};

let composite = CompositeHealthCheck::new()
    .add_check(PingHealthCheck)
    .add_check(MessageRateHealthCheck { min_messages_per_sec: 50.0 })
    .add_check(MemoryHealthCheck { max_memory_mb: 200 });

monitor.monitor_actor(actor_id, composite).await;
```

## Health Records

### Struct: `HealthRecord`

```rust
pub struct HealthRecord {
    pub timestamp: DateTime<Utc>,
    pub status: HealthStatus,
    pub check_duration: Duration,
}
```

Record of a single health check execution.

**Fields:**

- `timestamp`: When the health check was performed (UTC)
- `status`: The health status result
- `check_duration`: How long the health check took

**Example:**

```rust
use chrono::{DateTime, Utc};

let history = monitor.get_health_history(actor_id, 5);
for record in history {
    println!("[{}] {:?} (took {:?})",
        record.timestamp.format("%Y-%m-%d %H:%M:%S"),
        record.status,
        record.check_duration
    );
}
```

## Supervisor Integration

### Automatic Health Monitoring

Supervisors can automatically monitor child actors.

```rust
use airssys_rt::{Supervisor, ChildSpec};
use airssys_rt::monitoring::{HealthMonitor, PingHealthCheck};
use std::time::Duration;

impl Supervisor for MonitoredSupervisor {
    fn child_specs(&self) -> Vec<ChildSpec> {
        vec![
            ChildSpec::new("worker", || Worker::new())
                .with_health_check(PingHealthCheck)
                .with_health_interval(Duration::from_secs(30))
                .with_restart_on_unhealthy(true),
        ]
    }
    
    fn restart_strategy(&self) -> RestartStrategy {
        RestartStrategy::OneForOne
    }
}
```

### Health-Based Restart Policy

```rust
use airssys_rt::monitoring::HealthBasedRestartPolicy;

let policy = HealthBasedRestartPolicy {
    restart_on_unhealthy: true,
    restart_on_degraded: false,
    max_restarts: 3,
    restart_window: Duration::from_secs(60),
};
```

## Metrics and Reporting

### Struct: `HealthMetrics`

```rust
pub struct HealthMetrics {
    pub total_checks: u64,
    pub healthy_checks: u64,
    pub degraded_checks: u64,
    pub unhealthy_checks: u64,
    pub avg_check_duration: Duration,
}
```

Aggregated health check metrics.

**Fields:**

- `total_checks`: Total number of health checks performed
- `healthy_checks`: Number of healthy results
- `degraded_checks`: Number of degraded results
- `unhealthy_checks`: Number of unhealthy results
- `avg_check_duration`: Average time per health check

#### Methods

##### `health_percentage()`

```rust
pub fn health_percentage(&self) -> f64
```

Calculates percentage of healthy checks.

**Example:**

```rust
let metrics = monitor.get_metrics(actor_id);
println!("Health: {:.1}%", metrics.health_percentage());
```

## Performance Characteristics

### Health Check Overhead

| Check Type | Latency | Frequency | Overhead |
|------------|---------|-----------|----------|
| Ping | 0.5-2ms | 30s | Negligible |
| MessageRate | 50-100µs | 30s | <0.01% |
| Memory | 100-500µs | 60s | <0.01% |
| Composite (3 checks) | 1-3ms | 30s | <0.1% |

### Memory Usage

| Component | Size | Per Actor | Notes |
|-----------|------|-----------|-------|
| HealthMonitor | ~512 bytes | - | Base structure |
| Per-actor state | ~256 bytes | Yes | Status + history |
| Health history (10 records) | ~480 bytes | Yes | Circular buffer |

### Recommended Check Intervals

| Actor Type | Check Interval | Timeout | Failure Threshold |
|------------|----------------|---------|-------------------|
| Critical service | 10s | 2s | 2 |
| Standard actor | 30s | 5s | 3 |
| Background worker | 60s | 10s | 5 |
| Batch processor | 120s | 30s | 3 |

## Error Types

### Enum: `MonitoringError`

```rust
pub enum MonitoringError {
    NotMonitored,
    CheckFailed(String),
    Timeout,
    SystemError(String),
}
```

Errors specific to monitoring operations.

**Variants:**

- `NotMonitored`: Actor is not being monitored
- `CheckFailed(String)`: Health check execution failed
- `Timeout`: Health check exceeded timeout
- `SystemError(String)`: System-level monitoring error

**Example:**

```rust
use airssys_rt::monitoring::MonitoringError;

match monitor.stop_monitoring(actor_id) {
    Ok(()) => println!("Stopped monitoring"),
    Err(MonitoringError::NotMonitored) => {
        println!("Actor wasn't being monitored");
    }
    Err(e) => eprintln!("Error: {:?}", e),
}
```

## Testing Utilities

### Struct: `MockHealthCheck`

```rust
pub struct MockHealthCheck {
    // fields omitted
}
```

Mock health check for testing.

**Available in:** Test builds only (`#[cfg(test)]`)

#### Methods

##### `new()`

```rust
pub fn new() -> Self
```

Creates a new mock health check.

##### `set_status()`

```rust
pub fn set_status(&mut self, status: HealthStatus)
```

Sets the status this health check will return.

**Example:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use airssys_rt::monitoring::{MockHealthCheck, HealthStatus};
    
    #[tokio::test]
    async fn test_unhealthy_actor_restart() {
        let mut health_check = MockHealthCheck::new();
        health_check.set_status(HealthStatus::Unhealthy {
            reason: "Test failure".to_string(),
        });
        
        monitor.monitor_actor(actor_id, health_check).await;
        
        // Wait for health check
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Verify actor was restarted
        assert!(supervisor_probe.was_restarted(actor_id));
    }
}
```

## Best Practices

### Health Check Design

```rust
// ✅ Good - Lightweight and focused
struct QuickHealthCheck;
impl HealthCheck for QuickHealthCheck {
    async fn check_health(&self, actor_id: ActorId) -> HealthStatus {
        // Simple ping, returns quickly
        ping_actor(actor_id).await
    }
}

// ❌ Bad - Expensive operations
struct SlowHealthCheck;
impl HealthCheck for SlowHealthCheck {
    async fn check_health(&self, actor_id: ActorId) -> HealthStatus {
        // Complex database query (too slow)
        database.complex_query().await;
        HealthStatus::Healthy
    }
}
```

### Monitoring Configuration

```rust
// ✅ Good - Reasonable intervals and thresholds
let config = HealthMonitorConfig {
    check_interval: Duration::from_secs(30),
    check_timeout: Duration::from_secs(5),
    failure_threshold: 3,  // Avoid false positives
    recovery_threshold: 2,
};

// ❌ Bad - Too aggressive, overhead too high
let bad_config = HealthMonitorConfig {
    check_interval: Duration::from_millis(100),  // Too frequent!
    check_timeout: Duration::from_secs(30),      // Timeout > interval!
    failure_threshold: 1,                        // No tolerance for transients
    recovery_threshold: 10,                      // Takes too long to recover
};
```

## See Also

- [Core API Reference](core.md) - Core types and system
- [Actors API Reference](actors.md) - Actor lifecycle
- [Supervisors API Reference](supervisors.md) - Supervision integration
- [Architecture: Supervision](../../architecture/supervision.md) - Design overview
- [How-To: Supervisor Patterns](../../guides/supervisor-patterns.md) - Usage patterns
