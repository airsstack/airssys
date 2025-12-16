# Troubleshooting

**Category:** How-To Guide (Task-Oriented)  
**Purpose:** Solutions to common ComponentActor issues.

## Component Won't Start

### Issue: pre_start Hook Fails

**Symptom:**
```
Error: Component initialization failed: pre_start returned error
```

**Causes:**
1. State initialization error
2. External resource unavailable (database, file system)
3. Configuration invalid

**Solutions:**
```rust
impl Child for MyComponent {
    fn pre_start(&mut self) {
        // Add error handling and logging
        if let Err(e) = self.initialize_state() {
            log::error!("State initialization failed: {}", e);
            // Decide: panic to prevent start, or log and continue with defaults
        }
    }
}
```

**Diagnostic:**
```bash
# Enable debug logging
export RUST_LOG=airssys_wasm=debug
cargo run --example my_component
```

### Issue: WASM Module Won't Load

**Symptom:**
```
Error: WasmError::ModuleLoadFailed("module validation failed")
```

**Causes:**
1. Invalid WASM binary
2. Missing WIT interfaces
3. Incompatible WASM version

**Solutions:**

**Validate WASM module:**
```bash
wasm-tools validate component.wasm
```

**Check WIT interfaces:**
```bash
wasm-tools component wit component.wasm
```

**Rebuild with correct toolchain:**
```bash
cargo build --target wasm32-wasi
```

### Issue: Security/Permission Denied

**Symptom:**
```
Error: WasmError::PermissionDenied("filesystem access not allowed")
```

**Causes:**
1. Component.toml missing permissions
2. Capability not granted

**Solutions:**
```toml
# Component.toml
[permissions]
filesystem = ["read", "write"]
network = ["connect"]
```

**Verify capabilities:**
```rust
// Check component capabilities during initialization
if !capabilities.file_system.can_read(&path) {
    log::warn!("Missing read permission for {}", path);
}
```

## Messages Not Delivered

### Issue: Component Not Found

**Symptom:**
```
Error: WasmError::ComponentNotFound(ComponentId("sensor-1"))
```

**Causes:**
1. Component never registered
2. Component unregistered (stopped)
3. Typo in ComponentId

**Solutions:**
```rust
// Verify component is registered
let count = registry.count()?;
let components = registry.list_components()?;
log::info!("Registered components: {:?}", components);

// Check if target exists before sending
if let Ok(addr) = registry.lookup(&target_id) {
    router.send_message(&target_id, msg).await?;
} else {
    log::warn!("Component {} not found, queueing message", target_id);
    // Queue for later delivery
}
```

**Diagnostic:**
```rust
// Log all registrations
impl ComponentRegistry {
    pub fn register(&mut self, id: ComponentId, addr: ActorRef) {
        log::info!("Registering component: {}", id);
        self.components.insert(id, addr);
    }
}
```

### Issue: Component Stopped

**Symptom:**
Messages sent but no response

**Causes:**
1. Component stopped but still registered
2. Actor mailbox full
3. Component crashed

**Solutions:**
```rust
// Unregister during shutdown
impl Drop for MyComponent {
    fn drop(&mut self) {
        // Unregister from registry
        registry.unregister(&self.id).ok();
    }
}

// Check component health
if let Ok(health) = supervisor.get_health(&component_id) {
    match health {
        ChildHealth::Healthy => { /* send message */ }
        _ => log::warn!("Component unhealthy, skipping message"),
    }
}
```

**Diagnostic:**
```rust
// Monitor mailbox size
let queue_size = actor_ref.mailbox_size();
if queue_size > 1000 {
    log::warn!("Large mailbox: {} messages", queue_size);
}
```

### Issue: Registry Lookup Fails

**Symptom:**
```
Error: Lock poisoned: "RwLock poisoned"
```

**Causes:**
1. Panic while holding registry lock
2. Thread killed unexpectedly

**Solutions:**
```rust
// Graceful panic handling
let result = std::panic::catch_unwind(|| {
    // Code that might panic
});

match result {
    Ok(value) => { /* success */ }
    Err(e) => {
        log::error!("Panic caught: {:?}", e);
        // Restart component or system
    }
}
```

**Recovery:**
```bash
# Restart application
# RwLock poisoning requires process restart
```

## Performance Degradation

### Issue: High Lock Contention

**Symptom:**
- Slow response times
- High CPU usage
- Messages queuing up

**Diagnosis:**
```rust
use tokio::time::Instant;

let start = Instant::now();
let state = self.state.write().await;
let elapsed = start.elapsed();

if elapsed > Duration::from_millis(10) {
    log::warn!("High lock contention: {}ms", elapsed.as_millis());
}
```

**Solutions:**

**1. Minimize lock duration:**
```rust
// Good: release lock before async operation
let new_value = expensive_computation().await;
let mut state = self.state.write().await;
state.value = new_value;
```

**2. Use read locks when possible:**
```rust
// Read locks allow concurrent access
let state = self.state.read().await;
let value = state.count;
```

**3. Split state into multiple locks:**
```rust
struct ComponentState {
    config: Arc<RwLock<Config>>,    // Separate lock
    metrics: Arc<RwLock<Metrics>>,  // Separate lock
}
```

**4. Use message-based state updates:**
```rust
// Send message instead of locking directly
self.send(UpdateStateMessage { new_value }).await?;
```

**Performance Baseline:** State access measured at 37-39ns (Task 6.2). Alert if >1µs.

### Issue: Unbounded Message Queue

**Symptom:**
- Memory usage growing
- Message delays increasing
- System becomes unresponsive

**Diagnosis:**
```rust
let queue_size = actor_ref.mailbox_size();
if queue_size > 1000 {
    log::warn!("Large mailbox: {} messages", queue_size);
}
```

**Solutions:**

**1. Add backpressure:**
```rust
if actor_ref.mailbox_size() > MAX_QUEUE_SIZE {
    // Drop message or wait
    log::warn!("Mailbox full, applying backpressure");
    return Err(WasmError::Backpressure);
}
```

**2. Use bounded channel:**
```rust
// Max 100 messages
let (tx, rx) = mpsc::channel(100);
```

**3. Increase processing speed:**
```rust
// Process messages in batches
async fn process_batch(&mut self, messages: Vec<Message>) {
    for msg in messages {
        self.process_message(msg).await;
    }
}
```

**Performance Baseline:** Sustained throughput 6.12M msg/sec (Task 6.2). Alert if <100k msg/sec.

### Issue: Memory Leaks

**Symptom:**
- Memory usage grows over time
- Component not garbage collected

**Diagnosis:**
```rust
// Track component instances
static INSTANCE_COUNT: AtomicUsize = AtomicUsize::new(0);

impl MyComponent {
    fn new() -> Self {
        INSTANCE_COUNT.fetch_add(1, Ordering::Relaxed);
        // ...
    }
}

impl Drop for MyComponent {
    fn drop(&mut self) {
        INSTANCE_COUNT.fetch_sub(1, Ordering::Relaxed);
        log::info!("Component dropped, instances: {}", 
            INSTANCE_COUNT.load(Ordering::Relaxed));
    }
}
```

**Solutions:**

**1. Verify components are dropped after stop:**
```rust
// Component should be dropped after unregister
registry.unregister(&component_id)?;
// Verify INSTANCE_COUNT decreases
```

**2. Check for reference cycles:**
```rust
// Bad: reference cycle prevents drop
struct ComponentA {
    b: Arc<ComponentB>,
}
struct ComponentB {
    a: Arc<ComponentA>,  // Cycle!
}

// Good: use Weak references
struct ComponentB {
    a: Weak<ComponentA>,  // No cycle
}
```

**3. Cleanup resources in pre_stop:**
```rust
impl Child for MyComponent {
    fn pre_stop(&mut self) {
        // Close connections
        self.db_connection.close();
        
        // Clear caches
        self.cache.clear();
        
        // Drop large buffers
        self.buffer = Vec::new();
    }
}
```

## Crash Recovery Not Working

### Issue: Supervisor Not Restarting Component

**Symptom:**
Component crashes but doesn't restart

**Causes:**
1. Restart limit exceeded
2. RestartPolicy is Temporary (no restart)
3. Supervisor not configured

**Solutions:**

**1. Check restart policy:**
```rust
let config = SupervisorConfig {
    restart_policy: RestartPolicy::Permanent,  // Always restart
    restart_strategy: RestartStrategy::ExponentialBackoff {
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(30),
        multiplier: 2.0,
    },
    max_restarts: 5,  // Increase if needed
    time_window: Duration::from_secs(60),
};
```

**2. Verify supervisor is attached:**
```rust
// Spawn with supervisor
let component = system.spawn_supervised(
    "my-component",
    MyComponent::new(),
    config,
).await?;
```

**3. Check restart count:**
```rust
let stats = supervisor.get_stats(&component_id)?;
log::info!("Restart count: {}/{}", stats.restart_count, config.max_restarts);
```

### Issue: Health Checks Failing

**Symptom:**
```
Warning: Component failed health check 3 times
```

**Causes:**
1. Health check too strict
2. Component genuinely unhealthy
3. Health check timeout too short

**Solutions:**

**1. Implement realistic health check:**
```rust
async fn health_check(&self) -> ChildHealth {
    // Check critical resources
    match self.state.read().await.status {
        Status::Running => ChildHealth::Healthy,
        Status::Degraded => ChildHealth::Degraded("High latency".into()),
        Status::Failed => ChildHealth::Failed("Critical error".into()),
    }
}
```

**2. Check external dependencies:**
```rust
async fn health_check(&self) -> ChildHealth {
    if self.database.is_available().await {
        ChildHealth::Healthy
    } else {
        ChildHealth::Degraded("Database unavailable".into())
    }
}
```

**3. Adjust health check interval:**
```rust
let config = SupervisorConfig {
    health_check_interval: Duration::from_secs(10),  // Less frequent
    // ...
};
```

### Issue: Restart Loop

**Symptom:**
Component restarts repeatedly without success

**Causes:**
1. Initialization always fails
2. No exponential backoff
3. External dependency unavailable

**Solutions:**

**1. Use exponential backoff:**
```rust
let config = SupervisorConfig {
    restart_strategy: RestartStrategy::ExponentialBackoff {
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(30),
        multiplier: 2.0,
    },
    // ...
};
```

**2. Check dependencies during initialization:**
```rust
impl Child for MyComponent {
    fn pre_start(&mut self) {
        // Verify dependencies before starting
        if !self.database.is_available() {
            log::error!("Database unavailable, component will retry");
            panic!("Database unavailable");
        }
    }
}
```

**3. Set restart limit:**
```rust
let config = SupervisorConfig {
    max_restarts: 5,
    time_window: Duration::from_secs(60),
    // After 5 restarts in 60s, give up
};
```

## Debug Tools Setup

### Enable Rust Logging

```bash
# Set log level
export RUST_LOG=airssys_wasm=debug,airssys_rt=info

# Run with logging
cargo run --example my_component
```

**Output:**
```
DEBUG airssys_wasm::actor: Component registered: sensor-1
INFO airssys_wasm::actor: Component started: sensor-1
DEBUG airssys_wasm::router: Message routed: sensor-1 -> processor-1
```

### Tokio Console (Async Tracing)

```toml
# Cargo.toml
[dependencies]
console-subscriber = "0.2"
```

```rust
// Enable tracing
#[tokio::main]
async fn main() {
    console_subscriber::init();
    // ... your code ...
}
```

```bash
# Run tokio-console
tokio-console
```

**Features:**
- Task visualization
- Async call stack traces
- Resource usage tracking

### Memory Profiling

**Linux: valgrind**
```bash
valgrind --leak-check=full --show-leak-kinds=all ./target/debug/my_app
```

**Linux: heaptrack**
```bash
heaptrack ./target/debug/my_app
heaptrack_gui heaptrack.my_app.*.gz
```

**macOS: Instruments**
```bash
# Build with debug symbols
cargo build --release

# Run Instruments
instruments -t "Leaks" target/release/my_app
```

## Common Error Messages

### "Lock poisoned"

**Meaning:** A thread panicked while holding a lock

**Solution:**
1. Find and fix the panic
2. Restart component or application
3. Use `catch_unwind` to prevent lock poisoning

### "ComponentNotFound"

**Meaning:** Component not registered in registry

**Solution:**
1. Verify component is registered before sending messages
2. Check component ID spelling
3. Ensure component hasn't stopped

### "PermissionDenied"

**Meaning:** WASM component lacks required permissions

**Solution:**
1. Add permissions to Component.toml
2. Grant capabilities during component spawn
3. Check sandbox configuration

### "Backpressure"

**Meaning:** Message queue is full

**Solution:**
1. Slow down message production
2. Increase queue size
3. Improve message processing speed
4. Add more component instances

### "CorrelationIdNotFound"

**Meaning:** Response received for unknown request

**Solution:**
1. Check timeout settings (request may have timed out)
2. Verify request was registered
3. Check for duplicate responses

## Performance Debugging

### Measure Component Latency

```rust
use tokio::time::Instant;

async fn handle_message(&mut self, msg: Message) {
    let start = Instant::now();
    
    // Process message
    self.process(msg).await;
    
    let elapsed = start.elapsed();
    metrics::histogram!("message_latency_us", elapsed.as_micros() as f64);
    
    if elapsed > Duration::from_millis(100) {
        log::warn!("Slow message processing: {}ms", elapsed.as_millis());
    }
}
```

### Identify Bottlenecks

```rust
// Measure state lock wait time
let start = Instant::now();
let state = self.state.write().await;
let lock_wait = start.elapsed();

if lock_wait > Duration::from_millis(10) {
    log::warn!("High lock contention: {}ms", lock_wait.as_millis());
}

// Measure processing time
let start = Instant::now();
let result = self.expensive_operation();
let processing = start.elapsed();

if processing > Duration::from_millis(100) {
    log::warn!("Slow operation: {}ms", processing.as_millis());
}
```

### Compare with Baselines

**Task 6.2 Baselines (macOS M1):**
- Component spawn: 286ns
- Message routing: 1.05µs
- Registry lookup: 36ns
- Request-response: 3.18µs
- Throughput: 6.12M msg/sec

**Alert Thresholds:**
- Component spawn >1ms → Investigate spawn bottleneck
- Message latency >100µs → Check lock contention
- Throughput <100k msg/sec → Check system load
- Registry lookup >1µs → Investigate registry size

## References

- [Best Practices](./best-practices.md)
- [Production Readiness](../explanation/production-readiness.md)
- [Supervision and Recovery](./supervision-and-recovery.md)
- [Performance Characteristics](../reference/performance-characteristics.md)
- [Production Deployment](./production-deployment.md)

---

**Document Status:** ✅ Complete  
**Last Updated:** 2025-12-16  
**Coverage:** 80%+ of common issues
