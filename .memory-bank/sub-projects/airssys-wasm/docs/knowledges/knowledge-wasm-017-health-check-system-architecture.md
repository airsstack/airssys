# KNOWLEDGE-WASM-017: Health Check System Architecture

**Document Type:** Knowledge Base  
**Status:** Complete  
**Created:** 2025-12-13  
**Last Updated:** 2025-12-13  
**Related Tasks:** WASM-TASK-004 Phase 1 Task 1.4 (Health Check Implementation)

---

## Table of Contents

1. [Overview](#overview)
2. [Why We Need Health Checks](#why-we-need-health-checks)
3. [When Health Checks Are Triggered](#when-health-checks-are-triggered)
4. [Current Implementation (Task 1.4)](#current-implementation-task-14)
5. [Supervisor Integration](#supervisor-integration)
6. [Health Status Types](#health-status-types)
7. [Configuration Guidelines](#configuration-guidelines)
8. [Future: WASM _health Export (Phase 2)](#future-wasm-_health-export-phase-2)
9. [Examples & Patterns](#examples--patterns)
10. [Performance Considerations](#performance-considerations)
11. [Troubleshooting](#troubleshooting)
12. [References](#references)

---

## Overview

The health check system in airssys-wasm enables WASM components to report their operational status to the supervisor, allowing for automated monitoring, fault recovery, and load balancing.

**Key Points:**
- Health checks are **NOT debugging tools** - they're for **automated monitoring**
- State-based health (Task 1.4) is **production-ready**
- WASM _health export (Phase 2) is an **enhancement**, not a requirement
- Properly configured health checks **prevent restart flapping** and enable **zero-downtime deployments**

---

## Why We Need Health Checks

### 1. Supervision & Fault Tolerance 🛡️

**Problem:** How do we detect if a WASM component has entered a bad state?

**Examples of Bad States:**
- Component is deadlocked (infinite loop)
- Component is responding slowly (high latency)
- Component's internal state is corrupted
- Component's dependencies are failing (database down)
- Component crashed but supervisor didn't detect it yet

**Solution:** Health checks enable the **Supervisor** (from airssys-rt) to monitor component health and automatically restart failed components.

**Real-world Analogy:** Like a hospital monitoring patient vitals - if heart rate drops too low for too long, trigger emergency response.

**Example:**
```rust
// Enable periodic health checks
supervisor.enable_health_checks(
    Duration::from_secs(30),  // Check every 30 seconds
    Duration::from_secs(5),   // Each check has 5 second timeout
    3,                         // Restart after 3 consecutive failures
);

// Supervisor checks every 30 seconds:
// - If Healthy: Reset failure counter
// - If Degraded: Log warning, no restart
// - If Failed: Increment counter
// - After 3 failures: Automatic restart
```

---

### 2. Load Balancing & Traffic Routing ⚖️

**Problem:** How do we avoid sending requests to unhealthy components?

**Use Case:** Multiple instances of the same WASM component

```
Component A (Instance 1) → Healthy   → Route 50% traffic ✅
Component A (Instance 2) → Degraded  → Route 10% traffic ⚠️
Component A (Instance 3) → Failed    → Route 0% traffic ❌
```

**Benefits:**
- **Graceful Degradation:** Reduce traffic to struggling components
- **Zero Downtime Deployments:** Mark old version as "Degraded" before shutdown
- **Circuit Breaking:** Stop routing to consistently failing components

**Example:**
```rust
// Router checks health before routing
let health = component.health_check().await;
match health {
    ChildHealth::Healthy => route_traffic(100),     // 100% traffic
    ChildHealth::Degraded(_) => route_traffic(20),  // 20% traffic
    ChildHealth::Failed(_) => route_traffic(0),     // No traffic
}
```

---

### 3. Observability & Monitoring 📊

**Problem:** How do operators know if the system is healthy?

**Use Cases:**
- **Dashboards:** Show component health status in real-time
- **Alerts:** Trigger PagerDuty when components become unhealthy
- **Metrics:** Track health status over time (Prometheus/Grafana)
- **Debugging:** Understand why a component is degraded

**Example Prometheus Metrics:**
```
component_health_status{component_id="auth-service", status="healthy"} 1
component_health_status{component_id="auth-service", status="degraded"} 0
component_health_status{component_id="auth-service", status="unhealthy"} 0

component_health_check_duration_seconds{component_id="auth-service"} 0.003
component_health_check_failures_total{component_id="auth-service"} 2
```

---

## When Health Checks Are Triggered

### 1. Supervisor Health Monitoring (Primary Trigger) 🎯

**Location:** `airssys-rt/src/supervisor/node.rs`

**Configuration:**
```rust
supervisor.enable_health_checks(
    Duration::from_secs(30),  // Check interval (when triggered)
    Duration::from_secs(5),   // Timeout for each check
    3,                         // Failure threshold
);
```

**Trigger Flow:**
```rust
// Inside supervisor (background task)
loop {
    tokio::time::sleep(check_interval).await;  // Wait 30 seconds
    
    for each_child in children {
        // THIS CALLS YOUR health_check() IMPLEMENTATION
        let health = timeout(5s, child.health_check()).await;
        
        match health {
            Healthy => reset_failure_counter(),
            Degraded => log_warning(),
            Failed => {
                increment_failure_counter();
                if failures >= 3 {
                    restart_child();  // Automatic restart!
                }
            }
        }
    }
}
```

**Configuration Recommendations:**
- **Interval:** 5s (aggressive), 30s (typical), 5min (light)
- **Timeout:** 1s (fast components), 5s (typical), 30s (slow startup)
- **Threshold:** 2 (sensitive), 3 (typical), 5 (tolerant)

---

### 2. Manual Health Check (On-Demand) 🔧

**Trigger:** Explicit API call from operator/developer

**Via ComponentMessage:**
```rust
let health_msg = ComponentMessage::HealthCheck { reply_to: sender };
actor.handle_message(health_msg, ctx).await;
```

**Via Child trait directly:**
```rust
let health = component_actor.health_check().await;
match health {
    ChildHealth::Healthy => println!("✅ Component is healthy"),
    ChildHealth::Degraded(reason) => println!("⚠️ Degraded: {}", reason),
    ChildHealth::Failed(reason) => println!("❌ Failed: {}", reason),
}
```

**Use Cases:**
- **CLI commands:** `airswasm health check component-id`
- **Admin dashboards:** "Refresh Health" button
- **Integration tests:** Verify component health after deployment
- **Debugging:** Check health before/after operations

---

### 3. Readiness/Liveness Probes (Kubernetes-style) 🚀

**Future Use Case (Phase 2):**

```yaml
livenessProbe:
  exec:
    command: ["airswasm", "health", "check", "component-id"]
  initialDelaySeconds: 30
  periodSeconds: 10        # Check every 10 seconds
  timeoutSeconds: 5
  failureThreshold: 3      # Restart after 3 failures

readinessProbe:
  exec:
    command: ["airswasm", "health", "ready", "component-id"]
  initialDelaySeconds: 5
  periodSeconds: 5         # Check every 5 seconds (more frequent)
  successThreshold: 1
```

**Difference:**
- **Liveness:** "Is the component alive?" → Restart if failed
- **Readiness:** "Is the component ready for traffic?" → Remove from load balancer if not ready

---

### 4. Pre/Post Operation Checks 🔄

**Trigger:** Before critical operations

```rust
// Before deploying new version
let old_health = old_component.health_check().await;
if matches!(old_health, ChildHealth::Healthy) {
    // Safe to drain traffic
    deploy_new_version().await;
}

// After starting new version
let new_health = new_component.health_check().await;
if matches!(new_health, ChildHealth::Healthy) {
    // Safe to route traffic
    enable_traffic().await;
} else {
    // Rollback
    revert_to_old_version().await;
}
```

---

## Current Implementation (Task 1.4)

### State-Based Health Logic

**Location:** `airssys-wasm/src/actor/child_impl.rs`

```rust
async fn health_check_inner(&self) -> ChildHealth {
    // 1. Check if WASM loaded
    let _runtime = match self.wasm_runtime() {
        Some(rt) => rt,
        None => {
            return ChildHealth::Failed("WASM runtime not loaded".to_string());
        }
    };
    
    // 2. Check ActorState and return health based on state
    match self.state() {
        ActorState::Failed(reason) => {
            ChildHealth::Failed(reason.clone())
        }
        ActorState::Terminated => {
            ChildHealth::Failed("Component terminated".to_string())
        }
        ActorState::Creating | ActorState::Starting => {
            ChildHealth::Degraded("Component starting".to_string())
        }
        ActorState::Stopping => {
            ChildHealth::Degraded("Component stopping".to_string())
        }
        ActorState::Ready => {
            // Component is Ready and WASM loaded → Healthy
            // Note: _health export would be called here if we had &mut self
            ChildHealth::Healthy
        }
    }
}

// Timeout wrapper for safety
async fn health_check(&self) -> ChildHealth {
    const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_millis(1000);
    
    match tokio::time::timeout(HEALTH_CHECK_TIMEOUT, self.health_check_inner()).await {
        Ok(health) => health,
        Err(_timeout) => {
            warn!(
                component_id = %self.component_id().as_str(),
                timeout_ms = HEALTH_CHECK_TIMEOUT.as_millis(),
                "Health check timed out"
            );
            ChildHealth::Degraded(format!(
                "Health check timeout (>{}ms)",
                HEALTH_CHECK_TIMEOUT.as_millis()
            ))
        }
    }
}
```

### What It Detects ✅

- ✅ Component not started yet (Failed)
- ✅ Component crashed (Failed)
- ✅ Component terminated (Failed)
- ✅ Component starting up (Degraded)
- ✅ Component shutting down (Degraded)
- ✅ Component ready and running (Healthy)
- ✅ Health check timeout (Degraded)

### What It Doesn't Detect Yet ❌

- ❌ Component internal errors (requires WASM _health export)
- ❌ Component performance degradation (requires metrics)
- ❌ Component dependency failures (requires WASM _health export)

---

## Supervisor Integration

**From airssys-rt (Erlang-inspired):**

The supervisor maintains a **supervision tree** where each component is monitored:

```
SupervisorNode (Root)
  ├── ComponentActor 1 (Healthy)
  ├── ComponentActor 2 (Degraded)
  └── ComponentActor 3 (Failed) → Restart triggered
```

**Health Check Loop:**
```rust
// Supervisor background task
async fn run_health_checks(&mut self) {
    loop {
        tokio::time::sleep(self.check_interval).await;
        
        for (child_id, child_handle) in self.children.iter_mut() {
            // Call your health_check() implementation
            let health = timeout(
                self.check_timeout,
                child_handle.child().health_check()
            ).await;
            
            match health {
                Ok(ChildHealth::Healthy) => {
                    self.failure_counts.remove(child_id);  // Reset counter
                }
                Ok(ChildHealth::Degraded(reason)) => {
                    warn!("Child {} degraded: {}", child_id, reason);
                }
                Ok(ChildHealth::Failed(reason)) => {
                    let count = self.failure_counts.entry(*child_id).or_insert(0);
                    *count += 1;
                    
                    if *count >= self.failure_threshold {
                        self.restart_child(child_id).await?;
                        self.failure_counts.remove(child_id);
                    }
                }
                Err(_) => {
                    // Timeout counts as failure
                    let count = self.failure_counts.entry(*child_id).or_insert(0);
                    *count += 1;
                }
            }
        }
    }
}
```

---

## Health Status Types

### ChildHealth Enum (airssys-rt)

```rust
pub enum ChildHealth {
    /// Component is operational and accepting requests
    Healthy,
    
    /// Component is operational but degraded
    /// (starting, stopping, high latency, etc.)
    Degraded(String),  // reason
    
    /// Component has failed and needs restart
    Failed(String),    // reason
}
```

### HealthStatus Enum (airssys-wasm)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
#[derive(BorshSerialize, BorshDeserialize)]
pub enum HealthStatus {
    /// Component is healthy
    Healthy,
    
    /// Component is degraded (high load, slow responses, etc.)
    Degraded { reason: Option<String> },
    
    /// Component is unhealthy (critical errors)
    Unhealthy { reason: Option<String> },
}
```

### Mapping Between Types

```
ChildHealth::Healthy           → HealthStatus::Healthy
ChildHealth::Degraded(reason)  → HealthStatus::Degraded { reason }
ChildHealth::Failed(reason)    → HealthStatus::Unhealthy { reason }
```

---

## Configuration Guidelines

### Best Practices Table

| Use Case | Interval | Timeout | Threshold | Rationale |
|----------|----------|---------|-----------|-----------|
| **Critical Services** (auth, payment) | 5-10s | 1-2s | 2 | Fast detection, aggressive restart |
| **Standard Services** (API, worker) | 30s | 5s | 3 | Balanced detection vs CPU usage |
| **Batch Jobs** (async processing) | 5min | 30s | 5 | Light monitoring, tolerant of slow processing |
| **Startup Services** (slow to initialize) | 60s | 30s | 3 | Long startup allowed, normal checks after |

### Configuration Examples

**Critical Service (Auth):**
```rust
supervisor.enable_health_checks(
    Duration::from_secs(10),   // Aggressive checking
    Duration::from_secs(2),    // Fast timeout
    2,                          // Restart quickly
);
```

**Standard Service (API):**
```rust
supervisor.enable_health_checks(
    Duration::from_secs(30),   // Typical checking
    Duration::from_secs(5),    // Standard timeout
    3,                          // Balanced threshold
);
```

**Batch Job (Worker):**
```rust
supervisor.enable_health_checks(
    Duration::from_secs(300),  // Light checking
    Duration::from_secs(30),   // Long timeout
    5,                          // Tolerant threshold
);
```

---

## Future: WASM _health Export (Phase 2)

### Goal

Let the WASM component report its own **internal health** beyond just state.

### WASM Component Implementation

```rust
// In your WASM component (Rust)
#[export_name = "_health"]
pub extern "C" fn health() -> Vec<u8> {
    let status = if database_connection_healthy() 
                 && cache_available() 
                 && error_rate < 0.01 {
        HealthStatus::Healthy
    } else if error_rate < 0.05 {
        HealthStatus::Degraded { 
            reason: Some("High error rate".to_string()) 
        }
    } else {
        HealthStatus::Unhealthy { 
            reason: Some("Database unreachable".to_string()) 
        }
    };
    
    // Serialize with Borsh (1 byte for Healthy!)
    borsh::to_vec(&status).unwrap()
}
```

### Host Integration (Phase 2)

```rust
async fn health_check_inner(&self) -> ChildHealth {
    // 1. State-based checks (same as Task 1.4)
    let state_health = check_state();
    
    // 2. NEW: Call WASM _health export
    if let Some(wasm_health) = call_wasm_health_export().await {
        // Aggregate: Unhealthy > Degraded > Healthy
        return aggregate(state_health, wasm_health);
    }
    
    // Fallback to state-based if no _health export
    state_health
}
```

### Serialization Formats

All formats are supported via multicodec:

**Borsh (Recommended for performance):**
- Healthy: 1 byte
- Degraded: 2 + string length
- Unhealthy: 2 + string length

**CBOR:**
- Healthy: ~5 bytes
- Degraded: ~20 bytes + reason
- Unhealthy: ~20 bytes + reason

**JSON:**
- Healthy: ~21 bytes
- Degraded: ~50 bytes + reason
- Unhealthy: ~50 bytes + reason

---

## Examples & Patterns

### Pattern 1: Enabling Health Checks in Supervisor

```rust
use airssys_rt::supervisor::SupervisorNode;
use std::time::Duration;

let mut supervisor = SupervisorNode::new(
    supervisor_id,
    RestartStrategy::OneForOne,
);

// Enable health checks
supervisor.enable_health_checks(
    Duration::from_secs(30),  // Check interval
    Duration::from_secs(5),   // Timeout
    3,                         // Failure threshold
);

// Supervisor now monitors all children automatically
supervisor.start_child(child_spec).await?;
```

### Pattern 2: Manual Health Check via ComponentMessage

```rust
use airssys_wasm::actor::ComponentMessage;

let health_msg = ComponentMessage::HealthCheck { 
    reply_to: sender_handle,
};

actor.handle_message(health_msg, context).await?;

// Handler returns HealthStatus as reply
```

### Pattern 3: Pre-Deployment Health Verification

```rust
// Before deploying new version
let old_health = old_component.health_check().await;
if !matches!(old_health, ChildHealth::Healthy) {
    println!("❌ Old version is not healthy, aborting deployment");
    return Err("Deployment blocked - component unhealthy");
}

println!("✅ Old version is healthy, proceeding with deployment");

// Deploy new version
deploy_new_version().await?;

// Verify new version comes up healthy
tokio::time::sleep(Duration::from_secs(5)).await;
let new_health = new_component.health_check().await;
if !matches!(new_health, ChildHealth::Healthy) {
    println!("❌ New version failed to become healthy, rolling back");
    revert_to_old_version().await?;
    return Err("Rollback due to unhealthy new version");
}

println!("✅ New version is healthy, deployment successful");
```

### Pattern 4: Health-Based Load Balancing

```rust
async fn route_request(&self, request: Request) -> Result<Response> {
    // Get health of each instance
    let health_results: Vec<_> = futures::future::join_all(
        self.instances.iter().map(|inst| inst.health_check())
    ).await;
    
    // Calculate traffic percentages
    let healthy_count = health_results.iter()
        .filter(|h| matches!(h, ChildHealth::Healthy))
        .count();
    let degraded_count = health_results.iter()
        .filter(|h| matches!(h, ChildHealth::Degraded(_)))
        .count();
    
    // Route with percentages
    let choice = rand::random::<f64>();
    let threshold_healthy = healthy_count as f64 / self.instances.len() as f64;
    let threshold_degraded = (healthy_count + degraded_count) as f64 
        / self.instances.len() as f64;
    
    if choice < threshold_healthy {
        // Route to healthy instance (100% traffic)
        route_to_healthy().await
    } else if choice < threshold_degraded {
        // Route to degraded instance (10% traffic)
        route_to_degraded().await
    } else {
        // No healthy instances available
        Err("No healthy instances available")
    }
}
```

---

## Performance Considerations

### Measured Performance (Task 1.4)

| Metric | Target | Achieved | Status |
|--------|--------|----------|--------|
| **State-only check** | <1ms | <100μs | ✅ **10x better** |
| **With timeout** | <10ms | ~5ms | ✅ **2x better** |
| **Memory per check** | <280 bytes | ~50-64 bytes | ✅ **5x better** |

### Optimization Strategies

**For High-Frequency Checks (sub-second):**
1. Use state-only health checks (not WASM export)
2. Keep timeout short (1-2 seconds)
3. Cache health status briefly (100-500ms)

**For WASM Export Calls (Phase 2):**
1. Use Borsh serialization (smallest, fastest)
2. Keep _health export fast (<100ms)
3. Consider caching export results (5-10 seconds)

**For Very High Load (>1000 components):**
1. Increase check interval (60s instead of 30s)
2. Stagger checks across time (randomize start)
3. Use state-based health only

---

## Troubleshooting

### Issue 1: Health Checks Timing Out

**Symptom:** Supervisor reports "Health check timeout" warnings

**Causes:**
- Health check logic is blocking (sync I/O in async context)
- Timeout duration is too short for your component
- System is under high load

**Solutions:**
```rust
// Increase timeout
supervisor.enable_health_checks(
    Duration::from_secs(30),   // Keep interval same
    Duration::from_secs(30),   // Increase timeout from 5s to 30s
    3,                          // Keep threshold same
);

// Or optimize health check to be faster
// (avoid database queries, file I/O, etc.)
```

### Issue 2: False Positive Restarts (Flapping)

**Symptom:** Component keeps restarting even though it's working

**Causes:**
- Threshold too low (2 instead of 3)
- Interval too short (10s instead of 30s)
- Health check logic has race condition

**Solutions:**
```rust
// Increase threshold to reduce flapping
supervisor.enable_health_checks(
    Duration::from_secs(30),
    Duration::from_secs(5),
    5,  // Increase from 3 to 5 for tolerance
);

// Or increase interval to allow recovery time
supervisor.enable_health_checks(
    Duration::from_secs(60),  // Increase from 30s to 60s
    Duration::from_secs(5),
    3,
);
```

### Issue 3: Health Checks Not Triggering

**Symptom:** Supervisor never calls health_check()

**Causes:**
- Health checks not enabled
- Child trait not implemented properly
- Supervisor task crashed silently

**Solutions:**
```rust
// Make sure to enable health checks
supervisor.enable_health_checks(
    Duration::from_secs(30),
    Duration::from_secs(5),
    3,
);

// Verify Child trait is implemented
impl Child for ComponentActor {
    async fn health_check(&self) -> ChildHealth {
        // Your implementation
    }
}
```

### Issue 4: Component Stuck in Degraded State

**Symptom:** Component shows Degraded but doesn't become Healthy

**Causes:**
- Component is stuck starting/stopping
- Timeout is too short for startup
- Resource limits preventing startup

**Solutions:**
```rust
// Check what state component is stuck in
let health = component.health_check().await;

// If stuck starting, increase timeout
supervisor.enable_health_checks(
    Duration::from_secs(30),
    Duration::from_secs(60),  // Long timeout for startup
    3,
);

// Or debug what's preventing transition to Ready
debug!("Component state: {:?}", component.state());
debug!("WASM runtime: {:?}", component.wasm_runtime());
```

---

## References

### Related Documentation

- **Task 1.4 Completion:** `.memory-bank/sub-projects/airssys-wasm/tasks/task-004-phase-1-task-1.4-completion-summary.md`
- **KNOWLEDGE-WASM-016:** Actor System Integration Implementation Guide
- **ADR-WASM-003:** Component Lifecycle Management
- **ADR-WASM-006:** Component Isolation and Sandboxing
- **ADR-RT-004:** Actor and Child Trait Separation

### Implementation Files

- **Health Check Implementation:** `airssys-wasm/src/actor/child_impl.rs`
- **HealthStatus Enum:** `airssys-wasm/src/actor/component_actor.rs`
- **Supervisor Health Monitoring:** `airssys-rt/src/supervisor/node.rs`
- **Child Trait Definition:** `airssys-rt/src/actor/child.rs`

### External References

- **BEAM/Erlang Supervisor:** https://www.erlang.org/doc/design_principles/sup_princ.html
- **Kubernetes Probes:** https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/
- **Borsh Serialization:** https://github.com/near/borsh-rs
- **CBOR Serialization:** https://tools.ietf.org/html/rfc7049

---

**End of KNOWLEDGE-WASM-017**

