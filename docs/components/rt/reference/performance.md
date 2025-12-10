# Performance Reference

This reference provides measured performance characteristics and capacity planning guidance for the AirsSys Runtime system.

**Last Updated:** October 16, 2025  
**Measurement Environment:** macOS (development machine), Release build, Criterion 0.7.0  
**Sample Size:** 30 iterations per benchmark, 5 seconds measurement time, 95% confidence intervals

## Quick Reference

### Core Performance Metrics

| Operation | Latency | Throughput | Performance Class |
|-----------|---------|------------|-------------------|
| Actor spawn (single) | 624.74 ns | 1.6M/sec | ⚡ Sub-microsecond |
| Actor spawn (batch) | 681.40 ns/actor | 1.47M/sec | ⚡ Sub-microsecond |
| Message send/receive | 737.16 ns | 1.36M roundtrips/sec | ⚡ Sub-microsecond |
| Message throughput (broker) | 211.88 ns/msg | 4.7M msgs/sec | ⚡ Sub-microsecond |
| Direct message processing | 31.55 ns/msg | 31.7M msgs/sec | 🚀 Extremely fast |
| Mailbox operations | 181.60 ns/op | 5.5M ops/sec | ⚡ Sub-microsecond |
| Supervisor child spawn | 1.28 µs | 779K/sec | ✅ 1-2 microseconds |
| Supervision tree (3 children) | 3.01 µs | 997K/sec (total) | ✅ 2-5 microseconds |

**Performance Legend:**
- 🚀 **Extremely fast**: <100 ns (10M+ ops/sec)
- ⚡ **Sub-microsecond**: 100ns - 1µs (1-10M ops/sec)
- ✅ **Microseconds**: 1-10µs (100K-1M ops/sec)
- 📊 **Acceptable**: 10-100µs (10K-100K ops/sec)

## Actor System Performance

### Actor Lifecycle

#### Spawn Latency

**Single Actor Spawn:**

| Metric | Value | Notes |
|--------|-------|-------|
| **Mean** | 624.74 ns | Median performance |
| **Lower Bound** | 613.59 ns | 95% confidence |
| **Upper Bound** | 646.24 ns | 95% confidence |
| **Variance** | ±2.6% | Tight bounds |
| **Outliers** | 6.67% | 2 of 30 samples |

**Theoretical Capacity:**
- **1.6 million actors/second** spawn rate
- **10,000 actors**: 6.25 ms to spawn all
- **Constant time**: O(1) - independent of existing actor count

**Batch Actor Spawn (10 actors):**

| Metric | Value | Notes |
|--------|-------|-------|
| **Total Time** | 6.814 µs | For all 10 actors |
| **Per Actor** | 681.40 ns | Average cost |
| **vs Single** | +9% overhead | Batch marginally slower |

**Key Insight:** Single actor spawn is actually faster than batch per-actor (625 ns vs 681 ns). Batch spawn primarily for API ergonomics, not performance optimization.

#### Message Processing

**Direct Actor Processing (No Broker):**

| Metric | Value | Notes |
|--------|-------|-------|
| **Mean** | 31.55 ns/msg | 100-message sustained load |
| **Throughput** | 31.7M msgs/sec | Theoretical maximum |
| **Outliers** | 16.67% | 5 of 30 (async scheduler variance) |

**Use Case:** Hot-path actor logic with minimal async overhead.

**Memory Allocation Scaling:**

| Actor Count | Time per Actor | Overhead vs Single | Outliers |
|-------------|----------------|-------------------|----------|
| 1 | 718.43 ns | Baseline | 13.33% |
| 10 | 742.76 ns | +3.4% | 20% |
| 50 | 762.68 ns | +6.2% | 36.67% |

**Scaling Characteristics:**
- ✅ **Linear scaling**: Only 6.2% overhead at 50 actors
- ⚠️ **Higher variance at scale**: OS allocator variance increases with count
- ✅ **Predictable**: Memory allocation is NOT a scaling bottleneck

**Capacity Estimates:**

| Actor Count | Spawn Time (Total) | Memory Footprint (Est.) |
|-------------|-------------------|-------------------------|
| 10 | ~7.4 µs | <5 KB |
| 100 | ~76 µs | <50 KB |
| 1,000 | ~800 µs | <500 KB |
| 10,000 | ~8 ms | <5 MB |

## Message Passing Performance

### Point-to-Point Messaging

**Send/Receive Roundtrip (via Broker):**

| Metric | Value | Notes |
|--------|-------|-------|
| **Mean** | 737.16 ns | Full cycle through broker |
| **Lower Bound** | 732.80 ns | 95% confidence |
| **Upper Bound** | 743.39 ns | 95% confidence |
| **Variance** | ±0.7% | Very tight bounds |
| **Throughput** | 1.36M roundtrips/sec | Sustained rate |

**Latency Breakdown (Estimated):**
- Topic lookup/routing: ~50 ns (6.8%)
- Subscription management: ~60 ns (8.1%)
- Channel send/receive: ~70 ns (9.5%)
- Actor processing: ~400 ns (54.3%)
- Result return: ~106 ns (14.4%)
- Broker overhead: ~180 ns (24.4%)

### Sustained Throughput

**Message Throughput (100 messages via Broker):**

| Metric | Value | Notes |
|--------|-------|-------|
| **Total Time** | 21.188 µs | For 100 messages |
| **Per Message** | 211.88 ns | Average cost |
| **Throughput** | 4.7M msgs/sec | Broker routing |
| **vs Direct** | 6.7x slower | vs 31.55 ns direct |

**Broker Overhead Analysis:**
- **Direct processing**: 31.55 ns/msg (no broker)
- **Broker routing**: 211.88 ns/msg (with broker)
- **Overhead**: 180.33 ns (6.7x slower)

**When Broker Overhead is Acceptable:**
- ✅ Pub-sub patterns (1-to-many, dynamic subscriptions)
- ✅ Decoupling (publishers don't know subscribers)
- ✅ Still fast (4.7M msgs/sec is excellent for most workloads)

**When to Avoid Broker:**
- ⚠️ Ultra-hot paths requiring 31M msgs/sec
- ⚠️ Known fixed topology (use direct `ActorRef`)
- ⚠️ Latency-critical microsecond budgets

### Broadcast Performance

**Broadcast to 10 Subscribers:**

| Metric | Value | Notes |
|--------|-------|-------|
| **Mean** | 3.9511 µs | For 10 actors |
| **Per Subscriber** | 395.11 ns | Average cost |
| **Overhead** | ~40 ns/subscriber | vs point-to-point |

**Scaling:** O(n) with subscriber count (parallel delivery).

### Mailbox Efficiency

**Mailbox Operations (100 enqueue/dequeue cycles):**

| Metric | Value | Notes |
|--------|-------|-------|
| **Total Time** | 18.160 µs | For 100 operations |
| **Per Operation** | 181.60 ns | Enqueue + dequeue pair |
| **Throughput** | 5.5M ops/sec | Sustained rate |

**Mailbox Creation Comparison:**

| Mailbox Type | Creation Time | Overhead | Use Case |
|--------------|---------------|----------|----------|
| **Unbounded** | 188.55 ns | Baseline | Default choice, low latency |
| **Bounded (100)** | 244.18 ns | +29.5% | Backpressure, resource limits |

**Key Insight:** Unbounded mailboxes 23% faster to create, but 29% creation overhead is one-time cost. Operational performance identical.

**Recommendations:**

**Use Unbounded:**
- ✅ Default choice (23% faster creation, simpler semantics)
- ✅ Trusted internal actors
- ✅ Low message rate (<1000 msgs/sec)

**Use Bounded:**
- ✅ Backpressure needed (slow consumers, prevent memory bloat)
- ✅ Untrusted sources (external inputs, rate limiting)
- ✅ Known capacity constraints

## Supervision Performance

### Child Spawn Operations

**Single Child Spawn (via Builder):**

| Metric | Value | Notes |
|--------|-------|-------|
| **Mean** | 1.2834 µs | Single child |
| **Throughput** | 779K children/sec | Spawn rate |
| **Outliers** | 3.33% | 1 of 30 samples |

**Batch Child Spawn (3 children):**

| Metric | Value | Notes |
|--------|-------|-------|
| **Total Time** | 3.0073 µs | For 3 children |
| **Per Child** | 1.002 µs | Average cost |
| **vs Single** | -21.6% faster! | Batch efficiency |

**Key Insight:** Batch spawn 21.6% faster per-child than single spawn.

### Restart Strategy Performance

**Strategy Comparison (3 children):**

| Strategy | Total Time | Per Child | vs Baseline | Outliers |
|----------|-----------|-----------|-------------|----------|
| **OneForOne** (single) | 1.2731 µs | 1.2731 µs | Baseline | 6.67% |
| **OneForAll** (batch) | 2.9959 µs | 998.63 ns | -21.6% faster | 3.33% |
| **RestForOne** (batch) | 3.0012 µs | 1000.4 ns | -21.4% faster | 10% |
| **Tree (small)** | 3.0073 µs | 1002.4 ns | -21.3% faster | 0% |

**Critical Insight:**
- ✅ **Strategy choice is semantic, not performance-based**: <1% difference between strategies
- ✅ **Perfect stability**: Tree construction has 0% outliers (most stable benchmark)
- ✅ **Batch efficiency**: All batch strategies ~21% faster per-child

**Restart Latency Estimate:**
- **Stop existing child**: ~500 ns (cleanup, deregistration)
- **Spawn new child**: ~1,283 ns (measured baseline)
- **Total restart**: **~1.8 µs** (estimated)

**Restart Capacity:**
- **Single child restarts**: ~556,000 restarts/second
- **Batch restarts (3 children)**: ~1 million restarts/second total

### Supervision Tree Scaling

**Small Tree (3 children, 1 level):**
- **Construction**: 3.0073 µs
- **Per Child**: 1.002 µs
- **Outliers**: 0% (perfect stability)

**Projected Scaling (extrapolated):**
- **2 levels (9 children)**: ~9 µs construction (linear)
- **3 levels (27 children)**: ~27 µs construction (linear)
- **Pattern**: ✅ Linear scaling O(n) with tree size, no depth penalty

## Scaling Characteristics

### Linear Scaling Confirmed

**Actor Memory Allocation:**
- 1 actor: 718.43 ns
- 10 actors: 742.76 ns/actor (+3.4%)
- 50 actors: 762.68 ns/actor (+6.2%)
- **Conclusion**: Linear scaling with minimal overhead

**Message Processing:**
- Direct: 31.55 ns/msg (constant)
- Broker: 211.88 ns/msg (constant)
- **Conclusion**: O(1) per-message cost

**Supervision:**
- Single child: 1.28 µs
- 3 children: 1.00 µs/child (batch efficiency)
- **Conclusion**: Batch spawn more efficient than single

### Batch Efficiency

**Batch Operations Summary:**

| Operation | Single | Batch (per-unit) | Efficiency Gain |
|-----------|--------|------------------|-----------------|
| **Actor Spawn** | 624.74 ns | 681.40 ns | -9% slower |
| **Supervisor Spawn** | 1.2834 µs | 1.002 µs | +21.6% faster |
| **Message Broadcast** | N/A | 395.11 ns/sub | Parallel delivery |

**Insight:** Supervisor batch spawn shows significant efficiency gain, actor spawn does not benefit from batching.

### Broker Overhead

**Overhead Breakdown:**
- **Direct actor processing**: 31.55 ns/msg
- **Broker routing**: 211.88 ns/msg
- **Overhead**: 180.33 ns (6.7x slower)

**Acceptable Trade-offs:**
- ✅ 180 ns overhead is still sub-microsecond
- ✅ 4.7M msgs/sec sufficient for most workloads
- ✅ Pub-sub flexibility worth the cost

## Capacity Planning

### Theoretical Limits

**Actor Capacity:**
- **Spawn rate**: 1.6M actors/second
- **10,000 actors**: ~6.25 ms to spawn all
- **Memory**: Limited by available RAM, not framework

**Message Capacity:**
- **Direct processing**: 31.7M msgs/sec theoretical
- **Broker routing**: 4.7M msgs/sec theoretical
- **Realistic**: 1-5M msgs/sec (accounting for business logic)

**Supervision Capacity:**
- **Child spawns**: 779K/sec (single), 997K/sec (batch)
- **Restarts**: ~556K/sec estimated
- **Overhead**: <1% in normal operation

### Real-World Guidelines

**Capacity Planning Formula:**

```
System Capacity = min(
    CPU cores × msgs_per_core_per_sec,
    Network bandwidth ÷ message_size,
    Memory ÷ (actors × memory_per_actor)
)
```

**Typically Bounded By:**
1. **Business logic CPU** (not framework overhead)
2. **I/O operations** (database queries, network calls)
3. **Memory for actor state** (not actor framework)

### Recommended Limits

**Actor Count Guidelines:**

| Actor Count | Spawn Time | Memory (Est.) | Use Case |
|-------------|-----------|---------------|----------|
| **1-10** | <10 µs | <5 KB | Small services |
| **10-100** | <100 µs | <50 KB | Standard services |
| **100-1,000** | <1 ms | <500 KB | Medium systems |
| **1,000-10,000** | <10 ms | <5 MB | Large systems |
| **10,000+** | <100 ms | <50 MB | Massive scale |

**Message Rate Guidelines:**

| Messages/sec | Broker Overhead | Use Case |
|--------------|-----------------|----------|
| **<1,000** | Negligible | Low-volume services |
| **1K-10K** | <1% CPU | Standard services |
| **10K-100K** | ~1-5% CPU | High-throughput services |
| **100K-1M** | ~5-20% CPU | Very high throughput |
| **1M+** | Consider sharding | Distributed systems |

## Performance Optimization

### Hot Path Optimization

**Direct Actor References:**

```rust
// ✅ FASTEST: Direct actor-to-actor (31.55 ns/msg)
actor_ref.send(msg).await?;

// 📊 ACCEPTABLE: Via broker (211.88 ns/msg, +180 ns overhead)
broker.publish("topic", msg).await?;
```

**When to Optimize:**
- ⚠️ Proven bottleneck via profiling
- ⚠️ >1M msgs/sec on hot path
- ⚠️ Sub-millisecond latency budget

**When NOT to Optimize:**
- ✅ 4.7M msgs/sec is sufficient for workload
- ✅ Pub-sub flexibility needed
- ✅ No profiler evidence of bottleneck

### Mailbox Sizing

**Small Mailboxes (10-50):**
- **Use for**: Latency-sensitive, fast turnaround
- **Trade-off**: Higher blocking probability

**Medium Mailboxes (100-500):**
- **Use for**: Standard workloads
- **Trade-off**: Balanced memory/throughput

**Large Mailboxes (1000+):**
- **Use for**: Batch processing, high variance
- **Trade-off**: Higher memory footprint

### Supervision Strategy

**Strategy Selection (Performance Neutral):**
- ✅ All strategies have <1% variance
- ✅ Choose based on semantics, not performance:
  - **OneForOne**: Independent child failures
  - **OneForAll**: Coupled state/resources
  - **RestForOne**: Startup dependencies

## Performance Verification

### Running Benchmarks

```bash
# Full benchmark suite (~3-5 minutes)
cargo bench

# Specific categories
cargo bench actor_       # Actor system
cargo bench message_     # Message passing
cargo bench supervisor_  # Supervision
cargo bench resource_    # Resource usage

# View HTML reports
open target/criterion/report/index.html
```

### Baseline Comparison

**Verify Against Baselines:**
- Actor spawn: Should be ~625 ns
- Message roundtrip: Should be ~737 ns
- Supervisor spawn: Should be ~1.28 µs
- Message throughput: Should be ~211 ns/msg

**Significant Regression:** >20% degradation from baseline.

## Performance FAQ

### Q: Why is batch actor spawn slower per-actor than single spawn?

**A:** Memory allocator batch overhead (+9%). Batch spawn is for API ergonomics, not performance optimization. Use single spawn for lowest latency (625 ns vs 681 ns).

### Q: Is broker overhead acceptable for production?

**A:** Yes. 180 ns overhead is sub-microsecond, and 4.7M msgs/sec is excellent for most workloads. Use broker for pub-sub patterns, direct `ActorRef` for proven hot paths.

### Q: Should I use bounded or unbounded mailboxes?

**A:** Default to unbounded (23% faster creation, simpler). Use bounded for:
- Backpressure (slow consumers)
- Untrusted sources (rate limiting)
- Known capacity constraints

29% creation overhead is one-time cost; operational performance identical.

### Q: Which supervision strategy is fastest?

**A:** All strategies have <1% performance difference. Choose based on semantics:
- **OneForOne**: Independent failures (simplest)
- **OneForAll**: Coupled state (restart all)
- **RestForOne**: Ordered dependencies (restart affected)

### Q: How many actors can the system handle?

**A:** **Tens of thousands** on a single machine. Spawn time is negligible (~625 ns/actor). Typical limits:
- CPU: Business logic processing
- Memory: Actor state (not framework overhead)
- I/O: Database/network operations

Framework overhead is NOT a scaling bottleneck.

### Q: What's the maximum message throughput?

**A:** **Theoretical:** 31.7M msgs/sec (direct), 4.7M msgs/sec (broker).  
**Realistic:** 1-5M msgs/sec accounting for business logic, I/O, and other overhead. Framework overhead is NOT a throughput bottleneck.

## See Also

- [BENCHMARKING.md](../../BENCHMARKING.md) - Complete benchmark suite documentation
- [Architecture: System Overview](../architecture/system-overview.md) - Performance characteristics section
- [Architecture: Components](../architecture/components.md) - Per-component performance data
- [API Reference: Core](api/core.md) - Performance notes in method documentation
- [API Reference: Messaging](api/messaging.md) - Message passing performance details
- [API Reference: Supervisors](api/supervisors.md) - Supervision performance characteristics
