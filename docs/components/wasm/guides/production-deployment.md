# Production Deployment Guide

This guide shows you how to deploy ComponentActor systems to production environments with confidence. Follow these steps to prepare, configure, deploy, and monitor your components in production.

## Prerequisites

### System Requirements

**Minimum Requirements:**
- **Rust**: 1.70+ (stable toolchain)
- **Tokio**: 1.x runtime
- **CPU**: 2+ cores (4+ recommended for production)
- **Memory**: 4GB+ RAM (8GB+ recommended)
- **OS**: Linux (Ubuntu 20.04+), macOS (10.15+), Windows (Server 2019+)

**Recommended for Production:**
- **CPU**: 8+ cores for high-throughput workloads
- **Memory**: 16GB+ RAM for 1000+ components
- **Storage**: SSD for logging and metrics
- **Network**: Low-latency network for distributed deployments

### Dependency Versions

Verify compatibility in your `Cargo.toml`:

```toml
[dependencies]
airssys-wasm = "0.1"
airssys-rt = "0.1"
tokio = { version = "1.47", features = ["full"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

## Configuration Patterns

### Component Configuration

Create a `Component.toml` for each component:

```toml
[component]
name = "example-component"
version = "1.0.0"
description = "Production component"

[runtime]
max_memory_mb = 512
max_cpu_cores = 2
message_queue_size = 10000
health_check_interval_ms = 5000

[security]
capabilities = [
    "file:read:/data/input",
    "file:write:/data/output",
    "network:outbound:api.example.com:443"
]

[monitoring]
enable_metrics = true
enable_tracing = true
log_level = "info"
```

### Environment Variables

Set production environment variables:

```bash
# Runtime configuration
export AIRSSYS_LOG_LEVEL=info
export AIRSSYS_METRICS_PORT=9090
export AIRSSYS_HEALTH_PORT=8080

# Resource limits
export AIRSSYS_MAX_COMPONENTS=1000
export AIRSSYS_MAX_MEMORY_GB=16
export AIRSSYS_MAX_CPU_PERCENT=80

# Security
export AIRSSYS_ENABLE_AUDIT_LOG=true
export AIRSSYS_AUDIT_LOG_PATH=/var/log/airssys/audit.log
```

## Resource Limits

### Memory Recommendations

**Per Component:**
- Small components (stateless): 64-128 MB
- Medium components (stateful): 128-512 MB
- Large components (data processing): 512 MB - 2 GB

**System-Wide:**
- Reserve 2 GB for ActorSystem overhead
- Example: 16 GB RAM → 14 GB available for components
- With 256 MB avg per component → ~55 components per node

### CPU Core Allocation

**Tokio Thread Pool:**
```rust
use tokio::runtime::Builder;

let runtime = Builder::new_multi_thread()
    .worker_threads(num_cpus::get() - 1)  // Reserve 1 core for system
    .thread_name("airssys-worker")
    .enable_all()
    .build()?;
```

**Component Count Recommendations:**
- 4 cores: 10-50 components (depending on workload)
- 8 cores: 50-200 components
- 16 cores: 200-1000 components

Measured performance (Task 6.2):
- Component spawn: 286ns (2.65 million spawns/sec capacity)
- Message throughput: 6.12 million msg/sec
- Registry lookup: 36ns O(1) (validated 10-1,000 components)

## Deployment Checklist

Follow this checklist before production deployment:

### Pre-Deployment

- [ ] **Build Release Binary**: `cargo build --release`
- [ ] **Run Tests**: `cargo test --workspace` (100% pass required)
- [ ] **Run Clippy**: `cargo clippy --all-targets` (zero warnings)
- [ ] **Verify Dependencies**: `cargo audit` (no known vulnerabilities)
- [ ] **Configuration Review**: Validate Component.toml for all components
- [ ] **Security Audit**: Review capability grants (principle of least privilege)
- [ ] **Resource Limits**: Set appropriate memory/CPU limits
- [ ] **Logging Configuration**: Enable structured logging and audit logs
- [ ] **Metrics Setup**: Configure Prometheus exporters
- [ ] **Health Checks**: Implement health check endpoints

### Deployment

- [ ] **Backup Current State**: Backup component registry and state stores
- [ ] **Deploy Binary**: Copy release binary to production server
- [ ] **Deploy Components**: Upload component WASM modules
- [ ] **Start ActorSystem**: Initialize runtime with production config
- [ ] **Register Components**: Register components with registry
- [ ] **Spawn Components**: Start components with SupervisorNode
- [ ] **Verify Health**: Check health endpoints return healthy
- [ ] **Monitor Logs**: Watch logs for errors or warnings
- [ ] **Load Testing**: Gradually increase traffic to production levels

### Post-Deployment

- [ ] **Monitor Metrics**: Track spawn rate, message throughput, latency P99
- [ ] **Alert Configuration**: Set up alerts for SLA violations
- [ ] **Documentation**: Update runbook with deployment notes
- [ ] **Rollback Plan**: Document rollback procedure
- [ ] **Team Notification**: Notify team of successful deployment

## Monitoring Setup

### Metrics to Track

**Component Lifecycle Metrics:**
```rust
use prometheus::{IntCounter, Histogram, register_int_counter, register_histogram};

// Component spawn rate (target: <1ms P99)
let spawn_duration = register_histogram!(
    "component_spawn_duration_seconds",
    "Component spawn duration",
)?;

// Component count
let active_components = register_int_counter!(
    "components_active_total",
    "Number of active components",
)?;

// Lifecycle events
let lifecycle_events = register_int_counter!(
    "component_lifecycle_events_total",
    "Component lifecycle events by type",
)?;
```

**Messaging Metrics:**
```rust
// Message throughput (target: >100k msg/sec minimum)
let messages_sent = register_int_counter!(
    "messages_sent_total",
    "Total messages sent",
)?;

// Message latency (target: <100µs P99)
let message_latency = register_histogram!(
    "message_latency_seconds",
    "Message routing latency",
)?;
```

**Registry Metrics:**
```rust
// Registry lookup latency (target: <1µs)
let registry_lookup_duration = register_histogram!(
    "registry_lookup_duration_seconds",
    "Registry lookup duration",
)?;
```

### Performance Baselines (Task 6.2)

Monitor for degradation beyond these baselines:

| Metric | Baseline | Alert Threshold | Source |
|--------|----------|-----------------|--------|
| Component spawn | 286ns | >1ms P99 | actor_lifecycle_benchmarks.rs |
| Full lifecycle | 1.49µs | >10µs P99 | actor_lifecycle_benchmarks.rs |
| Message routing | 1.05µs | >100µs P99 | messaging_benchmarks.rs |
| Request-response | 3.18µs | >1ms P99 | messaging_benchmarks.rs |
| Pub-sub fanout (100) | 85.2µs | >1ms P99 | messaging_benchmarks.rs |
| Message throughput | 6.12M msg/sec | <100k msg/sec | messaging_benchmarks.rs |
| Registry lookup | 36ns O(1) | >1µs P99 | scalability_benchmarks.rs |

### Prometheus Configuration

Example `prometheus.yml`:

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'airssys-wasm'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'
```

## Health Checks

### Component Health Check

Implement health checks in your components:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

use airssys_rt::prelude::*;
use airssys_wasm::actor::ComponentActor;

#[derive(Clone)]
pub struct HealthCheckComponent {
    state: Arc<RwLock<HealthState>>,
}

#[derive(Debug)]
struct HealthState {
    last_heartbeat: chrono::DateTime<chrono::Utc>,
    error_count: u64,
}

impl HealthCheckComponent {
    pub async fn is_healthy(&self) -> bool {
        let state = self.state.read().await;
        let now = chrono::Utc::now();
        let elapsed = now.signed_duration_since(state.last_heartbeat);
        
        // Healthy if heartbeat within 30s and error rate < 10%
        elapsed.num_seconds() < 30 && state.error_count < 100
    }
}
```

### HTTP Health Endpoint

Expose health checks via HTTP:

```rust
use axum::{Router, routing::get, Json};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    components_active: u64,
    uptime_seconds: u64,
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        components_active: 42,
        uptime_seconds: 3600,
    })
}

pub fn create_health_router() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
}
```

### Kubernetes Probes

Example Kubernetes deployment with health checks:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: airssys-wasm
spec:
  containers:
  - name: airssys-wasm
    image: airssys-wasm:latest
    ports:
    - containerPort: 8080
    livenessProbe:
      httpGet:
        path: /health
        port: 8080
      initialDelaySeconds: 10
      periodSeconds: 30
    readinessProbe:
      httpGet:
        path: /ready
        port: 8080
      initialDelaySeconds: 5
      periodSeconds: 10
```

## Graceful Shutdown

### Shutdown Sequence

Implement graceful shutdown to avoid data loss:

```rust
use std::sync::Arc;
use tokio::signal;
use tokio::sync::RwLock;

use airssys_rt::prelude::*;
use airssys_wasm::actor::ComponentRegistry;

pub async fn run_with_shutdown(
    actor_system: ActorSystem,
    registry: Arc<RwLock<ComponentRegistry>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Start components
    println!("Starting components...");
    
    // Wait for shutdown signal
    signal::ctrl_c().await?;
    println!("Shutdown signal received. Initiating graceful shutdown...");
    
    // Step 1: Stop accepting new messages (drain message queues)
    println!("Step 1: Draining message queues (timeout: 10s)...");
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    
    // Step 2: Stop components in reverse dependency order
    println!("Step 2: Stopping components...");
    let registry_read = registry.read().await;
    let component_ids: Vec<_> = registry_read.list_all().collect();
    drop(registry_read);
    
    for component_id in component_ids {
        println!("  Stopping component: {}", component_id);
        // Components auto-cleanup via Drop
    }
    
    // Step 3: Flush logs and metrics
    println!("Step 3: Flushing logs and metrics...");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Step 4: Shutdown ActorSystem
    println!("Step 4: Shutting down ActorSystem...");
    actor_system.shutdown().await?;
    
    println!("Graceful shutdown complete.");
    Ok(())
}
```

### Timeout Handling

Set timeouts for each shutdown phase:

- Message drain: 10s (allow in-flight messages to complete)
- Component stop: 5s per component (pre_stop + post_stop hooks)
- Log flush: 2s (ensure all logs written)
- Total shutdown timeout: 60s maximum

## Performance Tuning

### Based on Task 6.2 Benchmarks

**Batch Message Processing:**
```rust
// Process messages in batches for higher throughput
const BATCH_SIZE: usize = 100;

async fn process_message_batch(
    messages: Vec<ComponentMessage>,
) -> Result<(), ProcessError> {
    for message in messages {
        // Process without awaiting between messages
        // Measured: 6.12M msg/sec throughput achievable
    }
    Ok(())
}
```

**Thread Pool Configuration:**
```rust
// Optimize thread pool for component workload
let runtime = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(8)  // Match CPU cores
    .thread_stack_size(2 * 1024 * 1024)  // 2MB stack
    .thread_name("component-worker")
    .enable_all()
    .build()?;
```

**Registry Optimization:**
Registry already achieves O(1) lookup (36ns constant time from 10-1,000 components). No tuning needed.

**State Access Optimization:**
```rust
// Minimize lock duration (measured: 37-39ns read/write)
{
    let state = self.state.read().await;  // Hold lock briefly
    let value = state.value.clone();
} // Lock released immediately

// Process value outside lock
process_value(value).await;
```

## Troubleshooting

See [Troubleshooting Guide](troubleshooting.md) for common production issues:

- High lock contention (state access bottlenecks)
- Memory leaks (component cleanup issues)
- Message queue growth (backpressure handling)
- Performance degradation (monitoring and profiling)

## Security Considerations

### Capability Grants

Follow principle of least privilege:

```rust
use airssys_wasm::security::CapabilitySet;

let capabilities = CapabilitySet::new()
    .with_file_read("/data/input")  // Only specific paths
    .with_network_outbound("api.example.com:443");  // Only specific hosts

// ❌ AVOID: Overly permissive capabilities
let bad_capabilities = CapabilitySet::new()
    .with_file_read("/")  // Too broad
    .with_network_outbound("*:*");  // Unrestricted
```

### Audit Logging

Enable comprehensive audit logging:

```rust
use tracing::{info, warn};

info!(
    component_id = %component_id,
    operation = "file_read",
    path = %path,
    "Component accessed file system"
);

warn!(
    component_id = %component_id,
    operation = "capability_denied",
    requested = %capability,
    "Capability violation detected"
);
```

## Capacity Planning

### Calculating Node Capacity

**Example: 16 GB RAM, 8 cores**

- System overhead: 2 GB
- Available for components: 14 GB
- Average component size: 256 MB
- **Capacity**: ~55 components per node

**Component spawn capacity** (Task 6.2):
- Spawn rate: 2.65 million/sec
- For 100 components: 37.7 µs total spawn time
- For 1000 components: 377 µs total spawn time

**Message throughput capacity** (Task 6.2):
- System capacity: 6.12 million msg/sec
- Per component (100 components): 61,200 msg/sec
- Per component (1000 components): 6,120 msg/sec

### Horizontal Scaling

For workloads exceeding single-node capacity:

1. Deploy multiple ActorSystem nodes
2. Distribute components across nodes (load balancing)
3. Use message routing for cross-node communication
4. Shared registry for component discovery (future: distributed registry)

## Summary

Follow this production deployment guide to:

- ✅ Configure system and component settings for production
- ✅ Set appropriate resource limits (memory, CPU, message queues)
- ✅ Deploy with pre-deployment, deployment, and post-deployment checklists
- ✅ Monitor key metrics (spawn rate, message throughput, latency P99)
- ✅ Implement health checks (component and system-level)
- ✅ Perform graceful shutdown (10s drain, 5s per component)
- ✅ Tune performance based on Task 6.2 benchmarks
- ✅ Secure components with capability-based security

**Production Readiness**: This guide enables confident deployment of ComponentActor systems to production environments with validated performance (6.12M msg/sec throughput, 286ns spawn time, 36ns O(1) registry lookup).

## Next Steps

- [Supervision and Recovery](supervision-and-recovery.md) - Implement crash recovery
- [Troubleshooting Guide](troubleshooting.md) - Diagnose production issues
- [Best Practices](best-practices.md) - Production-tested patterns
