# Supervision and Recovery Guide

This guide shows you how to implement supervision and crash recovery for ComponentActor systems. Supervision enables automatic restart of failed components, providing fault tolerance and high availability.

## Overview

Supervision patterns allow components to recover automatically from failures without manual intervention. The SupervisorNode (from airssys-rt) monitors component health and restarts crashed components according to configured policies.

**Key Benefits:**
- **Automatic Recovery**: Components restart automatically after crashes
- **Fault Isolation**: One component crash doesn't affect others
- **Configurable Strategies**: Choose restart policies for your use case
- **Health Monitoring**: Continuous health checks detect failures early

## Supervisor Integration

### Basic Supervisor Setup

Integrate SupervisorNode with ComponentActor:

```rust
// Layer 1: Standard library
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crates
use tokio::sync::RwLock;

// Layer 3: Internal modules
use airssys_rt::prelude::*;
use airssys_rt::supervisor::{SupervisorNode, SupervisorConfig, RestartStrategy};
use airssys_wasm::actor::ComponentActor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create ActorSystem
    let actor_system = ActorSystem::new("production-system").await?;
    
    // Configure supervisor with restart strategy
    let supervisor_config = SupervisorConfig {
        max_restarts: 5,
        within_duration: Duration::from_secs(60),
        restart_strategy: RestartStrategy::ExponentialBackoff {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
        },
    };
    
    // Create supervisor
    let supervisor = SupervisorNode::new(supervisor_config);
    let supervisor_ref = actor_system.spawn_actor(supervisor).await?;
    
    // Spawn component under supervision
    let component = MyComponent::new();
    let component_ref = supervisor_ref
        .send(SupervisorMessage::SpawnChild(Box::new(component)))
        .await?;
    
    println!("Component spawned under supervision: {}", component_ref);
    Ok(())
}
```

## Restart Strategies

### Immediate Restart

Restart component immediately after crash (development use):

```rust
use airssys_rt::supervisor::RestartStrategy;

let config = SupervisorConfig {
    max_restarts: 10,
    within_duration: Duration::from_secs(30),
    restart_strategy: RestartStrategy::Immediate,
};
```

**Use When:**
- Development and testing
- Transient failures expected
- Fast recovery more important than avoiding cascading failures

**Avoid When:**
- Resource exhaustion causes crashes (immediate restart amplifies problem)
- External dependency failures (restart won't help)

### Delayed Restart

Restart with fixed delay (production default):

```rust
let config = SupervisorConfig {
    max_restarts: 5,
    within_duration: Duration::from_secs(60),
    restart_strategy: RestartStrategy::Delayed {
        delay: Duration::from_secs(5),
    },
};
```

**Use When:**
- Transient external failures (API rate limits, network hiccups)
- Give external systems time to recover
- Production deployments with moderate failure rates

**Benefits:**
- Prevents rapid restart loops
- Allows external dependencies to stabilize
- Reduces supervisor overhead

### Exponential Backoff (Recommended)

Restart with increasing delays:

```rust
let config = SupervisorConfig {
    max_restarts: 10,
    within_duration: Duration::from_secs(300),
    restart_strategy: RestartStrategy::ExponentialBackoff {
        initial_delay: Duration::from_secs(1),
        max_delay: Duration::from_secs(60),
        multiplier: 2.0,
    },
};
```

**Delay Sequence:**
- 1st restart: 1s
- 2nd restart: 2s (1s × 2.0)
- 3rd restart: 4s (2s × 2.0)
- 4th restart: 8s
- 5th restart: 16s
- 6th restart: 32s
- 7th+ restart: 60s (capped at max_delay)

**Use When:**
- Production environments (recommended default)
- Persistent failures possible
- Want to balance recovery speed and stability

**Benefits:**
- Fast recovery for transient failures (1s initial delay)
- Prevents restart storms for persistent failures
- Adaptive to failure patterns

## SupervisorConfig Setup

### Configuration Options

```rust
use std::time::Duration;
use airssys_rt::supervisor::{SupervisorConfig, RestartStrategy};

pub struct SupervisorConfig {
    /// Maximum restarts allowed within `within_duration`
    /// If exceeded, component is permanently stopped
    pub max_restarts: u32,
    
    /// Time window for counting restarts
    /// Restart count resets after this duration
    pub within_duration: Duration,
    
    /// Strategy for restart delays
    pub restart_strategy: RestartStrategy,
}

// Recommended production config
let production_config = SupervisorConfig {
    max_restarts: 5,           // Allow 5 restarts
    within_duration: Duration::from_secs(60),  // Within 60 seconds
    restart_strategy: RestartStrategy::ExponentialBackoff {
        initial_delay: Duration::from_secs(1),
        max_delay: Duration::from_secs(30),
        multiplier: 2.0,
    },
};

// Strict config for critical components
let strict_config = SupervisorConfig {
    max_restarts: 3,           // Allow only 3 restarts
    within_duration: Duration::from_secs(30),  // Within 30 seconds
    restart_strategy: RestartStrategy::Delayed {
        delay: Duration::from_secs(10),  // 10s fixed delay
    },
};

// Lenient config for flaky components
let lenient_config = SupervisorConfig {
    max_restarts: 20,          // Allow many restarts
    within_duration: Duration::from_secs(600),  // Within 10 minutes
    restart_strategy: RestartStrategy::ExponentialBackoff {
        initial_delay: Duration::from_millis(500),
        max_delay: Duration::from_secs(120),
        multiplier: 1.5,
    },
};
```

## Health Monitoring

### Component Health Checks

Implement health checks in your component:

```rust
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct MonitoredComponent {
    state: Arc<RwLock<ComponentState>>,
}

#[derive(Debug)]
struct ComponentState {
    last_heartbeat: DateTime<Utc>,
    error_count: u64,
    total_requests: u64,
}

impl MonitoredComponent {
    /// Health check called by supervisor
    pub async fn health_check(&self) -> HealthStatus {
        let state = self.state.read().await;
        let now = Utc::now();
        let elapsed = now.signed_duration_since(state.last_heartbeat);
        
        // Check 1: Heartbeat within 30 seconds
        if elapsed.num_seconds() > 30 {
            return HealthStatus::Unhealthy("Heartbeat timeout".to_string());
        }
        
        // Check 2: Error rate < 10%
        let error_rate = if state.total_requests > 0 {
            (state.error_count as f64) / (state.total_requests as f64)
        } else {
            0.0
        };
        
        if error_rate > 0.1 {
            return HealthStatus::Degraded(format!(
                "High error rate: {:.2}%",
                error_rate * 100.0
            ));
        }
        
        HealthStatus::Healthy
    }
    
    /// Update heartbeat on message processing
    async fn update_heartbeat(&self) {
        let mut state = self.state.write().await;
        state.last_heartbeat = Utc::now();
        state.total_requests += 1;
    }
}

#[derive(Debug)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}
```

### Periodic Health Checks

Configure supervisor to perform periodic checks:

```rust
use std::time::Duration;

// Health check configuration
pub struct HealthCheckConfig {
    pub interval: Duration,
    pub timeout: Duration,
    pub unhealthy_threshold: u32,
}

let health_config = HealthCheckConfig {
    interval: Duration::from_secs(10),  // Check every 10 seconds
    timeout: Duration::from_secs(5),    // 5 second timeout
    unhealthy_threshold: 3,             // Restart after 3 consecutive failures
};
```

## Crash Recovery Patterns

### Isolated Restart Pattern

Restart only the crashed component (default behavior):

```rust
// Component A crashes
// Supervisor restarts Component A
// Components B and C continue running unaffected
```

**Benefits:**
- Minimal disruption
- Fast recovery
- Other components unaffected

**Use For:**
- Stateless components
- Independent components
- Components with no shared state

### State Recovery Pattern

Restore component state after restart:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct PersistentComponent {
    state: Arc<RwLock<ComponentState>>,
    state_store: Arc<StateStore>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ComponentState {
    counter: u64,
    last_value: String,
}

impl Child for PersistentComponent {
    fn pre_start(&mut self, context: &ChildContext) -> Result<(), ChildError> {
        // Restore state from persistent storage
        tokio::task::block_in_place(|| {
            let runtime = tokio::runtime::Handle::current();
            runtime.block_on(async {
                let restored_state = self.state_store
                    .load_state(&context.component_id)
                    .await?;
                
                let mut state = self.state.write().await;
                *state = restored_state;
                
                Ok(())
            })
        })
    }
    
    fn pre_stop(&mut self, context: &ChildContext) -> Result<(), ChildError> {
        // Persist state before shutdown
        tokio::task::block_in_place(|| {
            let runtime = tokio::runtime::Handle::current();
            runtime.block_on(async {
                let state = self.state.read().await;
                self.state_store
                    .save_state(&context.component_id, &*state)
                    .await?;
                
                Ok(())
            })
        })
    }
}
```

## Cascading Failure Prevention

### Restart Limits

Prevent infinite restart loops:

```rust
let config = SupervisorConfig {
    max_restarts: 5,           // Stop after 5 restarts
    within_duration: Duration::from_secs(60),
    restart_strategy: RestartStrategy::ExponentialBackoff {
        initial_delay: Duration::from_secs(1),
        max_delay: Duration::from_secs(30),
        multiplier: 2.0,
    },
};
```

**Behavior:**
- Allow 5 restarts within 60 seconds
- If 6th restart needed within window → component permanently stopped
- After 60 seconds pass without restart → counter resets to 0

### Circuit Breaker Pattern

Implement circuit breaker for external dependencies:

```rust
use std::sync::atomic::{AtomicU64, Ordering};

pub struct CircuitBreaker {
    failure_count: AtomicU64,
    failure_threshold: u64,
    recovery_timeout: Duration,
    last_failure: Arc<RwLock<Option<DateTime<Utc>>>>,
}

impl CircuitBreaker {
    pub async fn call<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        // Check if circuit is open (too many failures)
        let failures = self.failure_count.load(Ordering::Relaxed);
        if failures >= self.failure_threshold {
            // Check if recovery timeout elapsed
            let last_failure = self.last_failure.read().await;
            if let Some(failure_time) = *last_failure {
                let elapsed = Utc::now().signed_duration_since(failure_time);
                if elapsed.num_seconds() < self.recovery_timeout.as_secs() as i64 {
                    return Err(/* CircuitOpen */);
                }
            }
            
            // Recovery timeout elapsed, attempt to close circuit
            self.failure_count.store(0, Ordering::Relaxed);
        }
        
        // Execute function
        match f() {
            Ok(result) => {
                // Success - reset failure count
                self.failure_count.store(0, Ordering::Relaxed);
                Ok(result)
            }
            Err(err) => {
                // Failure - increment count and record time
                self.failure_count.fetch_add(1, Ordering::Relaxed);
                let mut last_failure = self.last_failure.write().await;
                *last_failure = Some(Utc::now());
                Err(err)
            }
        }
    }
}
```

## Supervision Tree Patterns

### Flat Supervision

All components supervised by single supervisor (simple):

```
SupervisorNode
  ├─ Component A
  ├─ Component B
  └─ Component C
```

**Use When:**
- All components have similar restart policies
- Simple architectures (< 10 components)
- All components are independent

### Hierarchical Supervision

Nested supervisors for complex systems:

```
RootSupervisor
  ├─ APISupervisor
  │   ├─ APIGateway
  │   └─ APIProcessor
  └─ DataSupervisor
      ├─ DataIngress
      └─ DataEgress
```

**Use When:**
- Components have different restart policies
- Logical grouping desired (API, data, compute)
- Want to isolate failure domains

## Testing Crash Recovery

### Simulating Crashes

Test supervision behavior:

```rust
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone)]
pub struct CrashTestComponent {
    state: Arc<RwLock<ComponentState>>,
    crash_after: AtomicU64,  // Crash after N messages
}

struct ComponentState {
    message_count: u64,
}

#[async_trait::async_trait]
impl Actor for CrashTestComponent {
    type Message = TestMessage;
    type Error = ComponentError;
    
    async fn handle_message(
        &mut self,
        message: Self::Message,
        _context: &ActorContext,
    ) -> Result<(), Self::Error> {
        let mut state = self.state.write().await;
        state.message_count += 1;
        
        // Crash after configured message count
        let crash_threshold = self.crash_after.load(Ordering::Relaxed);
        if crash_threshold > 0 && state.message_count >= crash_threshold {
            panic!("Simulated crash after {} messages", crash_threshold);
        }
        
        Ok(())
    }
}

#[tokio::test]
async fn test_supervisor_restart() {
    let actor_system = ActorSystem::new("test").await.unwrap();
    
    // Configure supervisor with immediate restart
    let config = SupervisorConfig {
        max_restarts: 3,
        within_duration: Duration::from_secs(10),
        restart_strategy: RestartStrategy::Immediate,
    };
    
    let supervisor = SupervisorNode::new(config);
    let supervisor_ref = actor_system.spawn_actor(supervisor).await.unwrap();
    
    // Spawn component that crashes after 5 messages
    let component = CrashTestComponent::new(5);
    let component_ref = supervisor_ref
        .send(SupervisorMessage::SpawnChild(Box::new(component)))
        .await
        .unwrap();
    
    // Send 10 messages (component will crash and restart)
    for i in 1..=10 {
        component_ref.send(TestMessage::new()).await.ok();
    }
    
    // Verify component restarted and continued processing
    tokio::time::sleep(Duration::from_secs(1)).await;
    // Component should have restarted after message 5
}
```

## Summary

Implement supervision and crash recovery in under 1 hour:

1. **Integrate SupervisorNode**: Spawn components under supervision
2. **Configure Restart Strategy**: Choose immediate, delayed, or exponential backoff
3. **Set Restart Limits**: Prevent infinite restart loops (max_restarts, within_duration)
4. **Implement Health Checks**: Monitor component health with periodic checks
5. **Handle State Recovery**: Persist and restore component state on restart
6. **Test Crash Recovery**: Simulate failures and verify restart behavior

**Recommended Production Config:**
```rust
SupervisorConfig {
    max_restarts: 5,
    within_duration: Duration::from_secs(60),
    restart_strategy: RestartStrategy::ExponentialBackoff {
        initial_delay: Duration::from_secs(1),
        max_delay: Duration::from_secs(30),
        multiplier: 2.0,
    },
}
```

## Next Steps

- [Component Composition](component-composition.md) - Orchestrate multiple components
- [Production Deployment](production-deployment.md) - Deploy with supervision
- [Troubleshooting](troubleshooting.md) - Debug crash recovery issues
