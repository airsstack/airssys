# Performance Guide

Performance optimization strategies for AirsSys components.

## OSL Performance

### Operation Overhead

OSL adds minimal overhead for security and logging:

- **Helper functions**: ~100μs total overhead
- **Logger middleware**: ~10μs per middleware
- **Security checks**: ~50μs for ACL/RBAC
- **Executor**: Direct OS call performance

### Optimization Strategies

**1. Disable unnecessary logging:**

```rust
let logger_config = LoggerConfig {
    log_success: false,  // Only log failures
    log_failures: true,
    include_duration: false,
    ..Default::default()
};
```

**2. Use simpler security policies:**

```rust
// Faster: Direct ACL match
let acl = AccessControlList::new()
    .add_entry(AclEntry::new(user, "/exact/path.txt", perms, Allow));

// Slower: Glob pattern matching
let acl = AccessControlList::new()
    .add_entry(AclEntry::new(user, "/data/**/*.txt", perms, Allow));
```

**3. Batch operations:**

```rust
// Slower: Many small operations
for file in files {
    write_file(&file.path, file.data, principal).await?;
}

// Faster: Batch with single security check
let batch_op = BatchWriteOperation::new(files);
executor.execute(batch_op, context).await?;
```

## RT Performance

### Baseline Performance

From RT-TASK-008 benchmarks (Oct 16, 2025):

- **Actor spawn**: ~625ns (single), ~681ns/actor (batch of 10)
- **Message throughput**: 4.7M messages/sec
- **Message latency**: <1ms p99
- **Mailbox operations**: ~182ns per message

### Tuning Mailbox Sizes

```rust
use airssys_rt::mailbox::*;

// Small mailbox: Lower memory, more backpressure
let mailbox = BoundedMailbox::new(100);

// Large mailbox: Higher memory, less backpressure
let mailbox = BoundedMailbox::new(10000);

// Unbounded: No backpressure (use with caution)
let mailbox = UnboundedMailbox::new();
```

### Choosing Backpressure Strategy

```rust
use airssys_rt::mailbox::BackpressureStrategy;

// Block: Wait for space (preserves order)
let config = MailboxConfig {
    strategy: BackpressureStrategy::Block,
    capacity: 1000,
};

// Drop: Discard new messages (best-effort)
let config = MailboxConfig {
    strategy: BackpressureStrategy::Drop,
    capacity: 1000,
};

// Reject: Return error (explicit failure)
let config = MailboxConfig {
    strategy: BackpressureStrategy::Reject,
    capacity: 1000,
};
```

### Message Batching

```rust
// Slower: Process one-by-one
for msg in messages {
    broker.publish(msg, address.clone()).await?;
}

// Faster: Batch messages
broker.publish_batch(messages, address).await?;
```

### Actor Pool Pattern

```rust
// Create worker pool
let pool_size = num_cpus::get();
let mut workers = Vec::new();

for i in 0..pool_size {
    let worker = WorkerActor::new();
    let address = ActorAddress::new(format!("worker-{}", i));
    spawn_actor(worker, address.clone(), broker.clone()).await?;
    workers.push(address);
}

// Round-robin distribution
let mut idx = 0;
for msg in messages {
    broker.publish(msg, workers[idx].clone()).await?;
    idx = (idx + 1) % pool_size;
}
```

## Monitoring Performance

### OSL Metrics

```rust
let logger = FileActivityLogger::new("/var/log/metrics.log").await?;
let middleware = LoggerMiddleware::with_config(
    logger,
    LoggerConfig {
        include_duration: true,  // Log operation duration
        ..Default::default()
    }
);
```

### RT Metrics

```rust
use airssys_rt::monitoring::*;

let monitor = InMemoryMonitor::new();

// Query metrics
let metrics = monitor.get_mailbox_metrics(&address)?;
println!("Queue depth: {}", metrics.queue_depth);
println!("Messages processed: {}", metrics.total_processed);
println!("Average latency: {}ms", metrics.avg_latency_ms);
```

## Profiling

### CPU Profiling

```bash
# Build with debugging symbols
cargo build --release --features debug

# Profile with perf (Linux)
perf record --call-graph=dwarf ./target/release/my-app
perf report

# Profile with Instruments (macOS)
instruments -t "Time Profiler" ./target/release/my-app
```

### Memory Profiling

```bash
# Use valgrind (Linux)
valgrind --tool=massif ./target/release/my-app

# Use heaptrack (Linux)
heaptrack ./target/release/my-app
```

## Benchmarking

### OSL Benchmarks

```bash
cd airssys-osl
cargo bench
```

### RT Benchmarks

```bash
cd airssys-rt
cargo bench

# Specific benchmarks
cargo bench --bench actor_benchmarks
cargo bench --bench message_benchmarks
cargo bench --bench supervisor_benchmarks
```

## Performance Checklist

### Before Optimization
- [ ] Profile to identify bottlenecks
- [ ] Measure current performance
- [ ] Set performance targets
- [ ] Identify critical paths

### Optimization Strategies
- [ ] Tune mailbox sizes
- [ ] Choose appropriate backpressure strategy
- [ ] Batch operations where possible
- [ ] Use actor pools for parallelism
- [ ] Minimize logging in hot paths
- [ ] Simplify security policies if possible

### After Optimization
- [ ] Verify performance improvements
- [ ] Test under load
- [ ] Monitor in production
- [ ] Document configuration

## Next Steps

- [Integration Guide](integration.md)
- [Security Guide](security.md)
- [RT Benchmarking Details](../components/rt/../../../BENCHMARKING.md)
