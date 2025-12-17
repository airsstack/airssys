# Troubleshooting

This guide covers common issues, error messages, and their solutions when working with AirsSys RT.

## Table of Contents

- [Compilation Issues](#compilation-issues)
- [Runtime Errors](#runtime-errors)
- [Performance Issues](#performance-issues)
- [Supervisor Problems](#supervisor-problems)
- [Message Passing Issues](#message-passing-issues)
- [Actor Lifecycle Issues](#actor-lifecycle-issues)
- [Mailbox Problems](#mailbox-problems)
- [System Configuration Issues](#system-configuration-issues)

---

## Compilation Issues

### Issue 1: `Actor` trait not in scope

**Error Message:**
```
error[E0405]: cannot find trait `Actor` in this scope
 --> src/main.rs:5:6
  |
5 | impl Actor for MyActor {
  |      ^^^^^ not found in this scope
```

**Cause:**
The `Actor` trait has not been imported into the current module scope.

**Solution:**
Add the necessary import:

```rust
use airssys_rt::actor::Actor;

// Or use the prelude for common traits
use airssys_rt::prelude::*;
```

**Prevention:**
Use `airssys_rt::prelude::*` to import all commonly used traits and types.

---

### Issue 2: `Handler<M>` trait not implemented

**Error Message:**
```
error[E0277]: the trait bound `MyActor: Handler<MyMessage>` is not satisfied
  --> src/main.rs:42:18
   |
42 |     actor_ref.send(msg).await;
   |               ^^^^ the trait `Handler<MyMessage>` is not implemented for `MyActor`
```

**Cause:**
The actor has not implemented `Handler<M>` for the message type being sent.

**Solution:**
Implement the `Handler<M>` trait for your actor:

```rust
use airssys_rt::prelude::*;
use async_trait::async_trait;

#[derive(Clone)]
struct MyMessage;

impl Message for MyMessage {
    type Result = ();
}

struct MyActor;

impl Actor for MyActor {}

#[async_trait]
impl Handler<MyMessage> for MyActor {
    async fn handle(&mut self, _msg: MyMessage, _ctx: &mut ActorContext<Self>) -> () {
        println!("Message handled!");
    }
}
```

**Prevention:**
Ensure every message type your actor receives has a corresponding `Handler<M>` implementation.

---

### Issue 3: Missing `async_trait` macro

**Error Message:**
```
error: async trait functions cannot be used in traits without `#[async_trait]`
  --> src/main.rs:15:5
   |
15 |     async fn handle(&mut self, msg: M, ctx: &mut ActorContext<Self>) -> M::Result;
   |     ^^^^^
```

**Cause:**
The `#[async_trait]` attribute is required for async trait methods but was not applied.

**Solution:**
Add `#[async_trait]` to both trait and implementation:

```rust
use async_trait::async_trait;
use airssys_rt::prelude::*;

#[async_trait]
impl Handler<MyMessage> for MyActor {
    async fn handle(&mut self, msg: MyMessage, ctx: &mut ActorContext<Self>) -> () {
        // Handler implementation
    }
}
```

**Prevention:**
Always use `#[async_trait]` when implementing `Handler<M>` or other async traits.

---

### Issue 4: `Message::Result` type mismatch

**Error Message:**
```
error[E0308]: mismatched types
  --> src/main.rs:28:9
   |
28 |         "OK".to_string()
   |         ^^^^^^^^^^^^^^^^ expected `()`, found `String`
```

**Cause:**
The return type in the `Handler<M>` implementation does not match the `Message::Result` associated type.

**Solution:**
Ensure the `Message::Result` type matches the handler's return type:

```rust
#[derive(Clone)]
struct MyMessage;

impl Message for MyMessage {
    type Result = String;  // Match handler return type
}

#[async_trait]
impl Handler<MyMessage> for MyActor {
    async fn handle(&mut self, _msg: MyMessage, _ctx: &mut ActorContext<Self>) -> String {
        "OK".to_string()  // Now matches Message::Result
    }
}
```

**Prevention:**
Always declare the correct `Message::Result` type when implementing `Message` trait.

---

## Runtime Errors

### Issue 5: Actor panics without restart

**Symptom:**
Actor encounters a panic and stops processing messages permanently, with no restart attempt.

**Cause:**
Actor is not supervised, or supervisor's restart strategy is set to `Abort`.

**Solution:**
Ensure actor is supervised with an appropriate restart strategy:

```rust
use airssys_rt::supervisor::{Supervisor, ChildSpec, RestartStrategy};
use airssys_rt::prelude::*;

// Create supervisor with OneForOne restart strategy
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForOne)
    .with_child(
        ChildSpec::new("my-actor")
            .with_actor::<MyActor>()
            .with_max_restarts(5)
            .with_restart_window(Duration::from_secs(60))
    )
    .build()
    .await?;
```

**Prevention:**
Always supervise critical actors and choose appropriate restart strategies.

---

### Issue 6: Deadlock in actor system

**Symptom:**
System hangs indefinitely with no progress in message processing.

**Cause:**
Circular message dependencies or synchronous blocking operations within actors.

**Cause Analysis:**

1. **Circular Dependencies:**
```rust
// Actor A sends to Actor B and waits
let result = actor_b_ref.send(msg).await;

// Actor B sends to Actor A and waits (DEADLOCK!)
let result = actor_a_ref.send(msg).await;
```

2. **Blocking Operations:**
```rust
// Blocking I/O in handler (BLOCKS EXECUTOR!)
async fn handle(&mut self, _msg: M, _ctx: &mut ActorContext<Self>) {
    std::fs::read_to_string("file.txt"); // Synchronous blocking!
}
```

**Solution:**

1. **Avoid Circular Dependencies:**
```rust
// Use fire-and-forget instead of request-reply
actor_b_ref.send(msg).await?;
// Don't wait for response if it creates cycle
```

2. **Use Async I/O:**
```rust
async fn handle(&mut self, _msg: M, _ctx: &mut ActorContext<Self>) {
    // Use async I/O instead
    tokio::fs::read_to_string("file.txt").await?;
}
```

3. **Use `spawn_blocking` for Unavoidable Blocking:**
```rust
use tokio::task;

async fn handle(&mut self, _msg: M, _ctx: &mut ActorContext<Self>) {
    let result = task::spawn_blocking(|| {
        // Synchronous blocking operation
        std::fs::read_to_string("file.txt")
    }).await??;
}
```

**Prevention:**

- Design message flow as directed acyclic graph (DAG)
- Use async I/O operations exclusively
- Reserve `spawn_blocking` for unavoidable synchronous code

---

### Issue 7: Messages not being received

**Symptom:**
Messages sent to actor are not processed, no errors reported.

**Cause:**
Multiple possible causes:

1. **Mailbox closed prematurely**
2. **Actor stopped before processing**
3. **Incorrect actor reference**
4. **Bounded mailbox full with Drop strategy**

**Diagnostic Steps:**

**Step 1: Check send result:**
```rust
match actor_ref.send(msg).await {
    Ok(_) => println!("Message sent successfully"),
    Err(e) => eprintln!("Send failed: {:?}", e),
}
```

**Step 2: Verify actor is running:**
```rust
// Check actor lifecycle state
if ctx.is_stopped() {
    eprintln!("Actor is stopped!");
}
```

**Step 3: Check mailbox configuration:**
```rust
// Verify mailbox capacity if bounded
let mailbox = Mailbox::bounded(100);  // May drop or block if full
```

**Solution:**

**If mailbox closed:**
```rust
// Check actor hasn't stopped
if !actor_ref.is_stopped() {
    actor_ref.send(msg).await?;
}
```

**If bounded mailbox full:**
```rust
// Use larger capacity or unbounded mailbox
let mailbox = Mailbox::unbounded();

// Or use blocking backpressure
let mailbox = Mailbox::bounded_with_backpressure(
    100,
    BackpressureStrategy::Block
);
```

**If wrong reference:**
```rust
// Use correct actor reference from spawn
let actor_ref = system.spawn(actor).await?;
// Use this exact reference, not cloned outdated references
```

**Prevention:**

- Always check send results
- Monitor actor lifecycle state
- Use appropriate mailbox configuration
- Keep track of valid actor references

---

## Performance Issues

### Issue 8: High message latency

**Symptom:**
Message processing takes significantly longer than expected (>10ms for simple operations).

**Diagnostic:**

**Check baseline performance:**
```bash
# Run benchmarks to establish baseline
cargo bench --bench message_benchmarks
```

**Expected baselines:**

- Point-to-point messaging: ~737ns
- Message broker routing: ~181ns overhead
- Actor spawn: ~625ns

**Causes and Solutions:**

**Cause 1: Message broker overhead**

**Diagnosis:**
```rust
// Measure with and without broker
let start = Instant::now();
actor_ref.send(msg).await?;  // Via broker
let duration = start.elapsed();
// If >10µs, broker may be bottleneck
```

**Solution:**
```rust
// Use direct actor references for hot paths
let actor_ref = system.spawn(actor).await?;
actor_ref.send(msg).await?;  // Direct send, no broker routing
```

**Cause 2: Bounded mailbox backpressure**

**Diagnosis:**
```rust
// Check if blocking on send
// If mailbox is bounded with Block strategy, may wait for capacity
```

**Solution:**
```rust
// Use larger capacity or unbounded mailbox for high-throughput actors
let actor = MyActor::builder()
    .with_mailbox(Mailbox::unbounded())
    .build();
```

**Cause 3: Inefficient message serialization**

**Diagnosis:**
```rust
// Large message types may cause allocation overhead
#[derive(Clone)]  // Clone overhead for large structs
struct HugeMessage {
    data: Vec<u8>,  // Large allocation
}
```

**Solution:**
```rust
// Use Arc for shared data
use std::sync::Arc;

#[derive(Clone)]
struct EfficientMessage {
    data: Arc<Vec<u8>>,  // Cheap clone via Arc
}
```

**Cause 4: Contention on message broker**

**Diagnosis:**
```rust
// Many actors publishing to broker simultaneously
// Check if broker registration lock is hot path
```

**Solution:**
```rust
// Use direct actor references instead of broker for high-frequency messaging
// Reserve broker for discovery and pub-sub patterns
```

**Prevention:**

- Benchmark message paths regularly
- Use direct actor references for hot paths
- Profile with `cargo flamegraph` to identify bottlenecks
- Monitor mailbox queue depth

---

### Issue 9: Memory usage growing unbounded

**Symptom:**
Process memory continuously grows, never stabilizes.

**Diagnostic:**

**Check mailbox growth:**
```rust
// Monitor mailbox queue depth
// If continuously growing, producer faster than consumer
```

**Run memory profiling:**
```bash
# Use heaptrack or valgrind
heaptrack target/release/my_app
heaptrack --analyze heaptrack.my_app.*.gz
```

**Causes and Solutions:**

**Cause 1: Unbounded mailbox with slow consumer**

**Diagnosis:**
```rust
// Messages accumulating faster than processing
// Unbounded mailbox allows unlimited growth
```

**Solution:**
```rust
// Use bounded mailbox with backpressure
let mailbox = Mailbox::bounded_with_backpressure(
    1000,
    BackpressureStrategy::Block  // Apply backpressure to producer
);
```

**Cause 2: Actors not being cleaned up**

**Diagnosis:**
```rust
// Actor references held indefinitely
// Stopped actors not garbage collected
```

**Solution:**
```rust
// Explicitly stop and drop actors
ctx.stop();
drop(actor_ref);  // Release reference
```

**Cause 3: Message retention in broker**

**Diagnosis:**
```rust
// Broker retains actor registrations after actors stop
```

**Solution:**
```rust
// Deregister actors on stop
broker.deregister(actor_id).await?;
```

**Cause 4: Leaked actor references**

**Diagnosis:**
```rust
// ActorRef stored in long-lived collections
static ACTORS: Mutex<Vec<ActorRef<MyActor>>> = Mutex::new(Vec::new());
```

**Solution:**
```rust
// Use weak references or clear collections
use std::sync::Weak;

// Or periodically clean up stopped actors
actors.retain(|actor_ref| !actor_ref.is_stopped());
```

**Prevention:**

- Use bounded mailboxes for all non-critical actors
- Implement proper actor lifecycle management
- Monitor memory metrics continuously
- Profile memory regularly during development

---

## Supervisor Problems

### Issue 10: Supervisor not restarting children

**Symptom:**
Child actor crashes but supervisor does not restart it.

**Diagnostic:**

**Check restart limits:**
```rust
// Verify max_restarts and restart_window configuration
let spec = ChildSpec::new("my-actor")
    .with_max_restarts(5)  // May have exceeded limit
    .with_restart_window(Duration::from_secs(60));
```

**Check supervisor strategy:**
```rust
// Verify strategy is not Abort
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForOne)  // Should restart
    .build()
    .await?;
```

**Causes and Solutions:**

**Cause 1: Restart limit exceeded**

**Diagnosis:**
```rust
// Child has crashed more than max_restarts within restart_window
// Supervisor gives up restarting
```

**Solution:**
```rust
// Increase restart limits if crashes are transient
let spec = ChildSpec::new("my-actor")
    .with_max_restarts(10)  // Higher limit
    .with_restart_window(Duration::from_secs(300));  // Longer window

// Or fix underlying issue causing repeated crashes
```

**Cause 2: Strategy set to Abort**

**Diagnosis:**
```rust
// Supervisor configured to abort on child failure
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::Abort)  // No restart!
    .build()
    .await?;
```

**Solution:**
```rust
// Use appropriate restart strategy
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForOne)  // Restart individual child
    .build()
    .await?;
```

**Cause 3: Child not properly registered**

**Diagnosis:**
```rust
// Child spawned outside supervisor
let actor_ref = system.spawn(actor).await?;  // Not supervised!
```

**Solution:**
```rust
// Register child with supervisor
let supervisor = SupervisorBuilder::new()
    .with_child(
        ChildSpec::new("my-actor")
            .with_actor::<MyActor>()
    )
    .build()
    .await?;
```

**Prevention:**

- Configure appropriate restart limits for expected failure rates
- Choose restart strategy matching fault tolerance requirements
- Always register critical actors with supervisor
- Monitor supervisor restart metrics

---

### Issue 11: OneForAll strategy causing cascading restarts

**Symptom:**
One child failure triggers restart of all children, causing service disruption.

**Diagnostic:**

**Check restart strategy:**
```rust
// Verify if OneForAll is actually needed
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForAll)  // Restarts all children
    .build()
    .await?;
```

**Cause:**
OneForAll strategy restarts all children when any child fails, which may be unnecessary overhead.

**Solution:**

**Use OneForOne if children are independent:**
```rust
// If children can fail independently without affecting others
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForOne)  // Only restart failed child
    .build()
    .await?;
```

**Use RestForOne if there's dependency order:**
```rust
// If later children depend on earlier children
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::RestForOne)  // Restart failed + later children
    .build()
    .await?;
```

**Keep OneForAll only if truly needed:**
```rust
// Only if all children must restart together
// Example: distributed transaction coordinators, state machines
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForAll)
    .build()
    .await?;
```

**Prevention:**

- Choose minimal restart strategy meeting fault tolerance needs
- Document why OneForAll or RestForOne is required
- Monitor restart metrics to detect excessive restarts

---

## Message Passing Issues

### Issue 12: Request-reply timeout

**Symptom:**
Request to actor times out without response.

**Diagnostic:**

**Check timeout configuration:**
```rust
// Verify timeout is reasonable for operation
let result = actor_ref.send(msg)
    .timeout(Duration::from_millis(100))  // May be too short
    .await?;
```

**Causes and Solutions:**

**Cause 1: Timeout too short**

**Diagnosis:**
```rust
// Operation legitimately takes longer than timeout
async fn handle(&mut self, _msg: M, _ctx: &mut ActorContext<Self>) {
    // Complex operation takes 500ms
    tokio::time::sleep(Duration::from_millis(500)).await;
}
```

**Solution:**
```rust
// Increase timeout to match operation duration
let result = actor_ref.send(msg)
    .timeout(Duration::from_secs(1))  // Adequate timeout
    .await?;
```

**Cause 2: Actor mailbox full (with Block strategy)**

**Diagnosis:**
```rust
// Message stuck in backpressure queue
// Bounded mailbox with Block strategy
```

**Solution:**
```rust
// Use larger mailbox capacity
let actor = MyActor::builder()
    .with_mailbox(Mailbox::bounded(1000))  // Larger capacity
    .build();

// Or use Error strategy to fail fast
let actor = MyActor::builder()
    .with_mailbox(Mailbox::bounded_with_backpressure(
        100,
        BackpressureStrategy::Error  // Immediate error if full
    ))
    .build();
```

**Cause 3: Handler not sending response**

**Diagnosis:**
```rust
// Handler doesn't return result properly
#[async_trait]
impl Handler<MyRequest> for MyActor {
    async fn handle(&mut self, _msg: MyRequest, _ctx: &mut ActorContext<Self>) -> String {
        // Forgot to return result!
    }
}
```

**Solution:**
```rust
// Ensure handler returns result
#[async_trait]
impl Handler<MyRequest> for MyActor {
    async fn handle(&mut self, _msg: MyRequest, _ctx: &mut ActorContext<Self>) -> String {
        "response".to_string()  // Return result
    }
}
```

**Prevention:**

- Set timeouts appropriate for operation latency
- Monitor mailbox queue depth
- Ensure all request handlers return results
- Use fire-and-forget for operations not needing responses

---

## Actor Lifecycle Issues

### Issue 13: Actor stops unexpectedly

**Symptom:**
Actor stops processing messages without explicit stop call.

**Diagnostic:**

**Check lifecycle hooks:**
```rust
#[async_trait]
impl ActorLifecycle for MyActor {
    async fn pre_start(&mut self, ctx: &mut ActorContext<Self>) -> Result<(), ActorError> {
        // May return error causing immediate stop
        Ok(())
    }
}
```

**Causes and Solutions:**

**Cause 1: `pre_start` returns error**

**Diagnosis:**
```rust
async fn pre_start(&mut self, ctx: &mut ActorContext<Self>) -> Result<(), ActorError> {
    // Initialization fails
    self.initialize()?;  // Error causes actor to stop
    Ok(())
}
```

**Solution:**
```rust
// Handle initialization errors gracefully
async fn pre_start(&mut self, ctx: &mut ActorContext<Self>) -> Result<(), ActorError> {
    match self.initialize() {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Initialization failed: {:?}", e);
            // Return Ok with degraded state, or Err to stop
            Err(ActorError::InitializationFailed(e))
        }
    }
}
```

**Cause 2: Unhandled panic in handler**

**Diagnosis:**
```rust
async fn handle(&mut self, msg: M, ctx: &mut ActorContext<Self>) -> M::Result {
    // Panic causes actor to stop if not supervised
    panic!("Unexpected error!");
}
```

**Solution:**
```rust
// Handle errors gracefully without panicking
async fn handle(&mut self, msg: M, ctx: &mut ActorContext<Self>) -> M::Result {
    match self.process(msg) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Handler error: {:?}", e);
            // Return error result instead of panicking
            Default::default()
        }
    }
}

// Or supervise actor to restart on panic
let supervisor = SupervisorBuilder::new()
    .with_child(
        ChildSpec::new("my-actor")
            .with_actor::<MyActor>()
            .with_max_restarts(5)
    )
    .build()
    .await?;
```

**Prevention:**

- Validate initialization conditions in `pre_start`
- Use supervisor for critical actors
- Avoid panics in handlers, use error results
- Log lifecycle events for debugging

---

## Mailbox Problems

### Issue 14: Bounded mailbox rejecting messages

**Symptom:**
Send operations fail with mailbox full error.

**Diagnostic:**

**Check backpressure strategy:**
```rust
let mailbox = Mailbox::bounded_with_backpressure(
    100,
    BackpressureStrategy::Error  // Immediate error if full
);
```

**Causes and Solutions:**

**Cause 1: Consumer slower than producer**

**Diagnosis:**
```rust
// Messages arriving faster than actor processes them
// Mailbox capacity exceeded
```

**Solution:**

**Option 1: Increase capacity:**
```rust
let mailbox = Mailbox::bounded(1000);  // Larger buffer
```

**Option 2: Use Block strategy:**
```rust
// Apply backpressure to producer
let mailbox = Mailbox::bounded_with_backpressure(
    100,
    BackpressureStrategy::Block
);
```

**Option 3: Optimize consumer:**
```rust
// Make handler more efficient
async fn handle(&mut self, msg: M, ctx: &mut ActorContext<Self>) {
    // Optimize processing logic
    self.process_efficiently(msg).await;
}
```

**Option 4: Add more consumers:**
```rust
// Use actor pool pattern
for i in 0..10 {
    let actor = MyActor::new();
    system.spawn(actor).await?;
}
```

**Cause 2: Bursty traffic patterns**

**Diagnosis:**
```rust
// Traffic comes in bursts exceeding mailbox capacity
// But average rate is sustainable
```

**Solution:**
```rust
// Size mailbox for burst capacity
let peak_rate = 1000;  // messages per second
let burst_duration = 5;  // seconds
let capacity = peak_rate * burst_duration;

let mailbox = Mailbox::bounded(capacity);
```

**Prevention:**

- Profile message rates during peak traffic
- Size mailbox capacity for burst handling
- Monitor mailbox queue depth metrics
- Use appropriate backpressure strategy

---

## System Configuration Issues

### Issue 15: ActorSystem initialization fails

**Symptom:**
System creation returns error or panics during initialization.

**Diagnostic:**

**Check configuration:**
```rust
let config = SystemConfig::builder()
    .with_name("my-system")
    .with_max_actors(10_000)
    .build()?;
```

**Causes and Solutions:**

**Cause 1: Invalid configuration values**

**Diagnosis:**
```rust
// Configuration values out of valid range
let config = SystemConfig::builder()
    .with_max_actors(0)  // Invalid: must be > 0
    .build()?;  // Returns error
```

**Solution:**
```rust
// Use valid configuration values
let config = SystemConfig::builder()
    .with_name("my-system")
    .with_max_actors(10_000)  // Valid: > 0
    .build()?;
```

**Cause 2: Insufficient resources**

**Diagnosis:**
```rust
// System configuration exceeds available resources
let config = SystemConfig::builder()
    .with_max_actors(1_000_000)  // May exceed memory limits
    .build()?;
```

**Solution:**
```rust
// Configure based on available resources
let available_memory = 4_000_000_000;  // 4GB in bytes
let actor_overhead = 1024;  // Approximate bytes per actor
let safe_max_actors = (available_memory / actor_overhead) / 2;  // 50% margin

let config = SystemConfig::builder()
    .with_max_actors(safe_max_actors)
    .build()?;
```

**Cause 3: Runtime not initialized**

**Diagnosis:**
```rust
// Tokio runtime not available
#[tokio::main]  // Missing!
async fn main() {
    let system = ActorSystem::new(config).await?;
}
```

**Solution:**
```rust
// Ensure Tokio runtime is initialized
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = SystemConfig::default();
    let system = ActorSystem::new(config).await?;
    Ok(())
}
```

**Prevention:**

- Validate configuration values before system creation
- Size system based on available resources
- Ensure async runtime is properly initialized
- Use SystemConfig::default() as starting point

---

## Additional Resources

### Debug Tools

**Enable debug logging:**
```rust
use tracing_subscriber;

tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

**Monitor actor health:**
```rust
use airssys_rt::monitoring::{HealthMonitor, HealthCheck};

let monitor = HealthMonitor::new();
monitor.register(actor_ref, PingHealthCheck::new()).await?;

let status = monitor.check_health(actor_ref).await?;
```

**Profile performance:**
```bash
# CPU profiling
cargo flamegraph --bin my_app

# Memory profiling
heaptrack target/release/my_app

# Benchmark comparisons
cargo bench --bench actor_benchmarks
```

### Common Patterns

**Graceful shutdown:**
```rust
// Shutdown signal handling
tokio::select! {
    _ = tokio::signal::ctrl_c() => {
        println!("Shutdown signal received");
        system.shutdown().await?;
    }
}
```

**Error recovery:**
```rust
// Retry with exponential backoff
use tokio::time::sleep;

for attempt in 0..5 {
    match operation().await {
        Ok(result) => return Ok(result),
        Err(e) if attempt < 4 => {
            let delay = Duration::from_millis(100 * 2_u64.pow(attempt));
            sleep(delay).await;
            continue;
        }
        Err(e) => return Err(e),
    }
}
```

### Getting Help

If you encounter issues not covered in this guide:

1. **Check API Reference:** Detailed API documentation available in [API Reference](api/core.md)
2. **Review Examples:** Working code examples in the `examples/` directory
3. **Run Benchmarks:** Verify baseline performance with `cargo bench`
4. **Enable Debug Logging:** Use `tracing` for detailed runtime information
5. **Report Issues:** GitHub issues for bugs and feature requests

### Performance Baselines

**Reference Performance (October 16, 2025 baseline):**

| Operation | P50 Latency | P95 Latency | P99 Latency |
|-----------|-------------|-------------|-------------|
| Actor Spawn | 624.74 ns | 675.14 ns | 762.47 ns |
| Message Send (Direct) | 737.16 ns | 798.93 ns | 876.44 ns |
| Child Spawn (Supervised) | 1.28 µs | 1.39 µs | 1.52 µs |
| Broker Routing | 181 ns | - | - |

See [Performance Reference](performance.md) for complete baseline metrics.

---

**Last Updated:** 2025-01-18 (RT-TASK-011 Phase 4 Day 7)
