# AirsSys-RT Performance Benchmarking Guide

**Last Updated:** October 16, 2025  
**Status:** Initial baseline measurement phase  
**Philosophy:** Measure first, optimize later ([ADR-RT-010](../.copilot/memory_bank/sub_projects/airssys-rt/docs/adr/adr_rt_010_baseline_first_performance_strategy.md))

---

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Running Benchmarks](#running-benchmarks)
4. [Interpreting Results](#interpreting-results)
5. [Benchmark Categories](#benchmark-categories)
6. [Baseline Results](#baseline-results)
7. [Performance Characteristics](#performance-characteristics)
8. [Regression Tracking](#regression-tracking)
9. [Contributing Guidelines](#contributing-guidelines)

---

## Overview

### What We Measure and Why

This benchmark suite establishes **baseline performance metrics** for the AirsSys-RT actor runtime. Rather than implementing premature optimizations, we measure current performance to:

✅ **Understand actual capabilities** - Know what the runtime can handle today  
✅ **Enable data-driven optimization** - Focus effort where data shows it matters  
✅ **Prevent performance regressions** - Catch degradation early through baseline tracking  
✅ **Build user trust** - Provide honest, measured performance characteristics  

### Baseline-First Philosophy

> "Premature optimization is the root of all evil" - Donald Knuth

The runtime is already designed with zero-cost abstractions:
- Generic constraints (no `Box<dyn Trait>`)
- Static dispatch throughout
- Minimal allocations
- Async/await with Tokio

**Our approach:**
1. ✅ **Measure** current architecture performance (this phase)
2. ⏸️ **Analyze** results to identify actual bottlenecks
3. ⏸️ **Optimize** only where data justifies the effort
4. ✅ **Track** regressions to maintain quality

See [ADR-RT-010: Baseline-First Performance Strategy](../.copilot/memory_bank/sub_projects/airssys-rt/docs/adr/adr_rt_010_baseline_first_performance_strategy.md) for detailed rationale.

### Resource-Conscious Design

This benchmark suite is designed to run efficiently on constrained resources:

- **Sample size:** 30 iterations (statistically valid, resource-efficient)
- **Measurement time:** 5 seconds per benchmark
- **Warm-up time:** 2 seconds
- **Total runtime:** ~3-5 minutes for full suite
- **Max actors:** 50 (no large-scale stress tests)
- **Disk I/O:** Minimal (plots disabled, compact JSON output)

---

## Quick Start

### Prerequisites

- Rust 1.70+ with Cargo
- macOS, Linux, or Windows
- ~100MB free disk space for criterion output
- Tokio runtime (included in dependencies)

### Run All Benchmarks

```bash
# From airssys-rt directory
cd airssys-rt

# Run complete benchmark suite (~3-5 minutes)
cargo bench

# View HTML reports
open target/criterion/report/index.html
```

### Run Specific Category

```bash
# Actor system benchmarks only
cargo bench actor_

# Message passing benchmarks only
cargo bench message_

# Supervision benchmarks only
cargo bench supervisor_

# Resource usage benchmarks only
cargo bench resource_
```

### Save Baseline

```bash
# Save current measurements as baseline
cargo bench -- --save-baseline initial

# Compare future runs against baseline
cargo bench -- --baseline initial

# Compare and generate report
cargo bench -- --baseline initial --save-baseline current
```

---

## Running Benchmarks

### Standard Execution

```bash
# Full suite with default settings
cargo bench

# Specific benchmark by name
cargo bench actor_spawn_single

# Quick smoke test (5 samples instead of 30)
cargo bench -- --sample-size 5

# Verbose output
cargo bench -- --verbose
```

### Advanced Options

```bash
# Custom measurement time (10 seconds)
cargo bench -- --measurement-time 10

# Custom warm-up time (5 seconds)
cargo bench -- --warm-up-time 5

# Export to specific format
cargo bench -- --save-baseline my-baseline
```

### Continuous Integration

```bash
# CI-friendly: no HTML output, fail on regression
cargo bench --no-fail-fast -- \
    --baseline main \
    --save-baseline pr-123
```

---

## Interpreting Results

### Reading Criterion Output

Criterion provides statistical analysis of benchmark results. Here's how to interpret the output:

```
actor_spawn_single      time:   [2.1234 µs 2.2456 µs 2.3678 µs]
                        change: [-2.34% +0.12% +2.56%] (p = 0.89 > 0.05)
                        No change in performance detected.
```

**Key Metrics:**

- **time:** `[lower bound, estimate, upper bound]` - 95% confidence interval
- **change:** Comparison to previous run (if baseline exists)
- **p-value:** Statistical significance (p < 0.05 = significant change)

### Statistical Measures

Criterion reports several statistical measures for each benchmark:

| Measure | Meaning | When to Care |
|---------|---------|--------------|
| **Mean** | Average time across all samples | Primary performance metric |
| **Median** | Middle value (50th percentile) | Typical case performance |
| **p95** | 95th percentile | Worst-case latency (excluding outliers) |
| **p99** | 99th percentile | Extreme worst-case latency |
| **Std Dev** | Variability in measurements | Low = consistent, High = investigate |

### When to Worry About Variance

**High variance (Std Dev > 20% of mean) may indicate:**

❌ System under load (close other applications)  
❌ Power management throttling (use AC power)  
❌ Background processes (disable indexing, backups)  
❌ Non-deterministic code (randomness, I/O)  

**Low variance (Std Dev < 5% of mean) is ideal** ✅

### Performance Thresholds

| Category | Expected Range | Action If Outside |
|----------|----------------|-------------------|
| Actor spawn | 1-10 µs | Investigate allocations |
| Message latency | <1 ms | Check async overhead |
| Supervisor restart | 1-10 ms | Profile startup code |
| Memory per actor | <1 KB | Review struct sizes |

**Note:** These are rough guidelines. Actual baselines TBD after Phase 2 measurement.

---

## Benchmark Categories

### 1. Actor System Benchmarks

**File:** `benches/actor_benchmarks.rs`

#### `actor_spawn_single`
- **What:** Time to create a single actor with context and broker
- **Why:** Core operation latency - indicates setup overhead
- **Expected:** ~2-5 µs (stack allocation, minimal work)
- **Measures:** Actor construction, address allocation, broker setup

#### `actor_spawn_batch_small`
- **What:** Time to create 10 actors
- **Why:** Batch operation efficiency
- **Expected:** ~20-50 µs (10x single spawn)
- **Measures:** Scaling characteristics, allocation patterns

#### `actor_message_throughput`
- **What:** Process 100 messages through a single actor
- **Why:** Critical path for business logic execution
- **Expected:** <1 ms for 100 messages (~10 µs/message)
- **Measures:** Message handling overhead, async/await cost

**Key Insights:**
- Spawn latency should be constant regardless of actor count
- Throughput scales linearly with message count
- Async overhead should be minimal (Tokio zero-cost)

---

### 2. Message Passing Benchmarks

**File:** `benches/message_benchmarks.rs`

#### `message_send_receive`
- **What:** Point-to-point message latency (publish → subscribe → receive)
- **Why:** Fundamental communication primitive
- **Expected:** <100 µs for single message
- **Measures:** Broker routing, channel operations

#### `message_throughput`
- **What:** Send and receive 100 messages through broker
- **Why:** Sustained messaging performance
- **Expected:** ~1-5 ms for 100 messages
- **Measures:** Broker scalability, queue efficiency

#### `message_broadcast_small`
- **What:** Broadcast message to 10 subscribers
- **Why:** Pub-sub pattern efficiency
- **Expected:** ~500 µs - 1 ms
- **Measures:** Multi-subscriber overhead

#### `mailbox_operations`
- **What:** Enqueue and dequeue 100 messages
- **Why:** Mailbox implementation efficiency
- **Expected:** <2 ms for 100 operations
- **Measures:** Tokio channel performance, bounded queue overhead

**Key Insights:**
- Broadcast scales with subscriber count
- Mailbox operations should have constant time complexity
- Broker overhead should be minimal vs direct channels

---

### 3. Supervision Benchmarks

**File:** `benches/supervisor_benchmarks.rs`

#### `supervisor_child_spawn`
- **What:** Spawn single child using builder API
- **Why:** Supervision setup overhead
- **Expected:** ~5-20 µs
- **Measures:** Builder pattern efficiency, child registration

#### `supervisor_restart_one_for_one`
- **What:** Restart single child (OneForOne strategy)
- **Why:** Fault tolerance basic operation
- **Expected:** ~10-50 µs
- **Measures:** Stop → start lifecycle cost

#### `supervisor_restart_one_for_all`
- **What:** Restart all children when one fails (3 children)
- **Why:** Strategy overhead comparison
- **Expected:** ~30-150 µs (3x OneForOne)
- **Measures:** Batch restart efficiency

#### `supervisor_restart_rest_for_one`
- **What:** Restart child and subsequent siblings
- **Why:** Dependency-aware restart cost
- **Expected:** Between OneForOne and OneForAll
- **Measures:** Partial restart overhead

#### `supervision_tree_small`
- **What:** Create supervisor with 3 children
- **Why:** Tree construction overhead
- **Expected:** ~20-100 µs
- **Measures:** Batch spawn via builder

**Key Insights:**
- Strategy overhead: OneForOne < RestForOne < OneForAll
- Builder pattern should have minimal overhead vs manual
- Restart should be fast enough for production use

---

### 4. Resource Usage Benchmarks

**File:** `benches/resource_benchmarks.rs`

#### `memory_per_actor_baseline`
- **What:** Incremental memory for 1, 10, 50 actors
- **Why:** Understand resource requirements
- **Expected:** ~500 bytes - 2 KB per actor
- **Measures:** Actor + context + broker size

**Parameterized by actor count:**
- `memory_per_actor_baseline/1` - Single actor overhead
- `memory_per_actor_baseline/10` - Small system
- `memory_per_actor_baseline/50` - Medium system

#### `mailbox_memory_comparison`
- **What:** Bounded vs unbounded mailbox memory
- **Why:** Choose appropriate mailbox type
- **Expected:** Similar when empty, bounded prevents growth
- **Measures:** Base structure size

**Sub-benchmarks:**
- `mailbox_memory/bounded_mailbox_100` - 10 bounded mailboxes (capacity 100)
- `mailbox_memory/unbounded_mailbox` - 10 unbounded mailboxes

**Key Insights:**
- Memory scales linearly with actor count
- Bounded mailboxes prevent unbounded growth
- Context and broker contribute to per-actor overhead

---

## Baseline Results

**Status:** ✅ Complete (Measured October 16, 2025)

**Measurement Environment:**
- **Hardware:** macOS (development machine)
- **Conditions:** Release build with optimizations, idle system
- **Sample Size:** 30 iterations per benchmark
- **Measurement Time:** 5 seconds per benchmark
- **Criterion Version:** 0.7.0
- **Statistics:** 95% confidence intervals

### Actor System Baseline

| Benchmark | Lower Bound | **Estimate** | Upper Bound | Per-Unit Cost | Outliers |
|-----------|-------------|--------------|-------------|---------------|----------|
| `actor_spawn_single` | 613.59 ns | **624.74 ns** | 646.24 ns | 624.74 ns/actor | 6.67% |
| `actor_spawn_batch_small` (10 actors) | 6.7908 µs | **6.8140 µs** | 6.8500 µs | 681.40 ns/actor | 6.67% |
| `actor_message_throughput` (100 msgs) | 2.8671 µs | **3.1546 µs** | 3.6883 µs | 31.55 ns/message | 16.67% |

**Key Observations:**
- ✅ **Sub-microsecond spawn latency**: Single actor creation in 624.74 ns
- ✅ **Excellent batch efficiency**: 681 ns/actor (vs 625 ns single) - only 9% overhead
- ✅ **Blazing message processing**: 31.55 ns/message (31.7M messages/second theoretical)

### Message Passing Baseline

| Benchmark | Lower Bound | **Estimate** | Upper Bound | Per-Unit Cost | Outliers |
|-----------|-------------|--------------|-------------|---------------|----------|
| `message_send_receive` | 732.80 ns | **737.16 ns** | 743.39 ns | 737.16 ns/roundtrip | 10% |
| `message_throughput` (100 msgs) | 20.575 µs | **21.188 µs** | 22.061 µs | 211.88 ns/message | 10% |
| `message_broadcast_small` (10 actors) | 3.2159 µs | **3.9511 µs** | 5.0914 µs | 395.11 ns/broadcast | 10% |
| `mailbox_operations` (100 ops) | 17.983 µs | **18.160 µs** | 18.385 µs | 181.60 ns/operation | 10% |

**Key Observations:**
- ✅ **Sub-microsecond latency**: Full send/receive cycle through broker in 737 ns
- ✅ **High throughput**: 4.7M messages/second sustained via broker routing
- ✅ **Efficient broadcast**: 395 ns to broadcast to 10 actors (~40 ns overhead per subscriber)
- 📊 **Broker overhead**: 6.7x slower than direct processing (211 ns vs 31 ns) - acceptable for pub-sub semantics

### Resource Usage Baseline

| Benchmark | Lower Bound | **Estimate** | Upper Bound | Per-Actor Cost | Outliers |
|-----------|-------------|--------------|-------------|----------------|----------|
| `memory_per_actor/1` | 701.64 ns | **718.43 ns** | 752.30 ns | 718.43 ns | 13.33% |
| `memory_per_actor/10` | 7.3558 µs | **7.4276 µs** | 7.5698 µs | 742.76 ns/actor | 20% |
| `memory_per_actor/50` | 37.971 µs | **38.134 µs** | 38.339 µs | 762.68 ns/actor | 36.67% |
| `mailbox_memory/bounded_mailbox_100` (10) | 2.4155 µs | **2.4418 µs** | 2.4755 µs | 244.18 ns/mailbox | 3.33% |
| `mailbox_memory/unbounded_mailbox` (10) | 1.8789 µs | **1.8855 µs** | 1.8931 µs | 188.55 ns/mailbox | 13.33% |

**Key Observations:**
- ✅ **Linear scaling**: Memory allocation scales linearly (718 → 743 → 763 ns per actor, only 6% overhead)
- ✅ **Minimal overhead**: 50 actors = 762.68 ns/actor (only 6.2% increase from single actor)
- ✅ **Mailbox efficiency**: Unbounded 23% faster to create (188 ns vs 244 ns) - bounded pays upfront capacity cost
- ⚠️ **Higher variance at scale**: memory_per_actor/50 has 36.67% outliers (OS allocator variance)

### Supervision Baseline

| Benchmark | Lower Bound | **Estimate** | Upper Bound | Notes | Outliers |
|-----------|-------------|--------------|-------------|-------|----------|
| `supervisor_child_spawn` | 1.2798 µs | **1.2834 µs** | 1.2874 µs | Single child via builder | 3.33% |
| `supervisor_strategy_one_for_one` | 1.2682 µs | **1.2731 µs** | 1.2778 µs | Spawn with OneForOne | 6.67% |
| `supervisor_strategy_one_for_all` (3 children) | 2.9825 µs | **2.9959 µs** | 3.0095 µs | Spawn batch OneForAll | 3.33% |
| `supervisor_strategy_rest_for_one` (3 children) | 2.9907 µs | **3.0012 µs** | 3.0188 µs | Spawn batch RestForOne | 10% |
| `supervision_tree_small` (3 children) | 2.9985 µs | **3.0073 µs** | 3.0145 µs | Tree construction | 0% |

**Key Observations:**
- ✅ **Sub-2µs single child spawn**: 1.28 µs via builder pattern
- ✅ **Negligible strategy overhead**: OneForOne (1.273 µs) nearly identical to basic spawn (1.283 µs)
- ✅ **Batch spawn efficiency**: 3 children = ~3.0 µs = 1.0 µs per child (21.6% faster than single spawn)
- ✅ **Strategy-agnostic performance**: OneForAll, RestForOne, and tree all ~3.0 µs (<1% difference)
- ✅ **Perfect stability**: supervision_tree_small has 0% outliers

### Performance Summary

**Latency Profiles:**
- **Sub-microsecond** (<1 µs): Actor spawn (625 ns), message send/receive (737 ns), memory allocation (718 ns)
- **1-2 microseconds**: Supervisor child spawn (1.28 µs), strategies (1.27 µs)
- **2-5 microseconds**: Batch supervisor spawn (3.0 µs), message broadcast (3.95 µs)

**Throughput Estimates (Theoretical):**
- **Direct actor processing**: 31.7 million msgs/sec (31.55 ns/msg)
- **Broker routing**: 4.7 million msgs/sec (211.88 ns/msg)
- **Point-to-point**: 1.36 million roundtrips/sec (737.16 ns/roundtrip)
- **Actor spawn rate**: 1.6 million actors/sec (single), 1.47M/sec (batch)
- **Supervision operations**: 779K children/sec (single), 997K/sec (batch of 3)

**Scaling Characteristics:**
- ✅ **Linear scaling confirmed**: Actor memory, message processing, supervisor batch operations
- ✅ **Batch efficiency**: Supervisor batch spawn 21.6% faster per-child than single spawn
- ⚠️ **Broker overhead**: 6.7x slower than direct processing (acceptable for pub-sub routing)

**Target Metrics Achievement:**
- ✅ **Message latency <1ms**: **1,357x faster** (737 ns = 0.000737 ms)
- ✅ **Throughput >1M msgs/sec**: **4.7x better** (4.7M msgs/sec via broker)
- ⏸️ **10,000 concurrent actors**: Not tested (max 50 in benchmarks) - pending large-scale test
- ⏸️ **<1KB per actor**: Allocation time measured, memory size pending

---

## Performance Characteristics

**Status:** ✅ Complete (Based on October 16, 2025 baseline measurements)

This section documents observed performance patterns, scaling behaviors, and best practices derived from comprehensive baseline data analysis.

---

### 1. Actor System Characteristics

#### Spawn Latency Analysis

**Single Actor Spawn:**
- **Baseline**: 624.74 ns (613.59-646.24 ns range)
- **Performance Class**: Sub-microsecond (excellent)
- **Throughput Capability**: ~1.6 million actors/second
- **Outliers**: 6.67% (2/30) - acceptable variance

**Batch Actor Spawn (10 actors):**
- **Baseline**: 6.814 µs total = 681.40 ns/actor
- **Batch Overhead**: +9% vs single spawn (56.66 ns overhead per actor)
- **Cause**: Memory allocator batch overhead, contiguous allocation patterns
- **Recommendation**: ⚠️ Batch spawn slightly slower per-actor - use for API ergonomics, not performance

**Scaling Behavior:**
- ✅ **Constant-time spawn**: Spawn latency independent of existing actor count (tested up to 50)
- ✅ **Predictable**: Tight confidence bounds (±20 ns typical)
- ✅ **No batching benefit**: Single spawn marginally faster than batch per-actor

**Best Practices:**
```rust
// ✅ GOOD: Spawn actors as needed, one at a time for lowest latency
let actor = system.spawn(MyActor::new());

// ⚠️ ACCEPTABLE: Batch for ergonomics, not performance (9% slower per-actor)
let actors: Vec<_> = (0..10).map(|_| system.spawn(MyActor::new())).collect();

// ❌ AVOID: Don't pre-spawn large pools "for performance" without evidence
// (spawn cost is already negligible at 625 ns)
```

#### Message Processing Patterns

**Direct Actor Processing:**
- **Baseline**: 31.55 ns/message (100-message sustained load)
- **Performance Class**: Extremely fast (31.7M msgs/sec theoretical)
- **Use Case**: Hot-path actor logic with minimal async overhead
- **Outliers**: 16.67% (5/30) - slightly higher variance due to async scheduler

**Characteristics:**
- ✅ **Linear scaling**: 100 messages = 3.15 µs (31.55 ns/msg average)
- ✅ **Async overhead minimal**: Tokio zero-cost abstractions validated
- ⚠️ **Scheduler variance**: Higher outliers (16.67%) indicate async task scheduling influence

**Actor Message Throughput (via system):**
- **Baseline**: 31.55 ns/message sustained
- **Expectation**: Real-world throughput 10-20M msgs/sec (accounting for business logic)
- **Bottleneck**: Typically user code, not actor framework

**Best Practices:**
```rust
// ✅ GOOD: Keep message handlers lightweight
async fn handle_message(&mut self, msg: MyMessage) {
    self.state.update(msg); // Fast in-memory update
}

// ⚠️ ACCEPTABLE: I/O or heavy computation - won't hit 31M msgs/sec
async fn handle_message(&mut self, msg: MyMessage) {
    let result = self.database.query(msg).await; // I/O bound
    self.process(result); // Business logic
}

// 💡 TIP: 31 ns/msg baseline means framework overhead is negligible
// Your business logic determines actual throughput
```

#### Scaling Behavior

**Memory Scaling (1 → 10 → 50 actors):**
- **1 actor**: 718.43 ns allocation time
- **10 actors**: 742.76 ns/actor (+3.4% overhead)
- **50 actors**: 762.68 ns/actor (+6.2% overhead)
- **Trend**: ✅ **Linear with minimal degradation** (6% total overhead at 50x scale)

**Interpretation:**
- OS memory allocator shows excellent scaling characteristics
- Overhead likely due to memory fragmentation, page faults at higher counts
- **Projection**: <10% overhead even at 100-1000 actors (extrapolated)

**Best Practices:**
```rust
// ✅ GOOD: Scale actor count based on workload, not arbitrary limits
// 762 ns @ 50 actors = negligible cost for parallelism benefits

// 💡 TIP: Actor creation cost is NOT a scaling bottleneck
// Focus on actor design (state size, message complexity) instead
```

---

### 2. Message Passing Characteristics

#### Latency vs Throughput Tradeoffs

**Point-to-Point Latency (Single Message):**
- **Baseline**: 737.16 ns (send → broker → subscribe → receive)
- **Performance Class**: Sub-microsecond (excellent)
- **Roundtrip Rate**: 1.36 million roundtrips/second
- **Components**: Actor send (est. 100 ns) + broker routing (est. 500 ns) + subscription (est. 137 ns)

**Sustained Throughput (100 Messages):**
- **Baseline**: 211.88 ns/message (100-message batch)
- **Performance Class**: High throughput (4.7M msgs/sec)
- **Comparison**: 3.5x slower than single message (batching overhead)
- **Cause**: Broker routing, subscription management, channel congestion

**Latency-Throughput Relationship:**
```
Single message:    737 ns  (low latency, low batching)
100 messages:      211 ns/msg (higher per-msg cost, sustained load)
Difference:        3.5x slower per-message in sustained workload
```

**Interpretation:**
- 📊 **Single-shot operations**: 737 ns latency (request-reply patterns)
- 📊 **Sustained operations**: 211 ns/msg throughput (streaming, pub-sub)
- ⚠️ **Batching paradox**: Higher per-message cost under load (channel contention)

**Best Practices:**
```rust
// ✅ GOOD: For latency-critical single operations (RPC-style)
let response = actor.request(msg).await; // ~737 ns roundtrip

// ✅ GOOD: For throughput-oriented streaming (fire-and-forget)
for msg in messages {
    actor.send(msg); // ~211 ns/msg sustained (4.7M msgs/sec)
}

// 💡 TIP: Choose pattern based on workload:
// - Request-reply: Optimize for 737 ns latency
// - Pub-sub streaming: Optimize for 211 ns/msg throughput
```

#### Broker Routing Overhead

**Direct Actor Processing:** 31.55 ns/message (baseline from actor_message_throughput)
**Broker Routing:** 211.88 ns/message (baseline from message_throughput)
**Overhead:** **180.33 ns (6.7x slower than direct)**

**Broker Overhead Breakdown (estimated):**
- Topic lookup/routing: ~50 ns
- Subscription management: ~60 ns
- Channel send/receive: ~70 ns
- **Total**: ~180 ns overhead vs direct actor-to-actor

**When Broker Overhead is Acceptable:**
- ✅ **Pub-sub patterns**: 1-to-many, dynamic subscriptions
- ✅ **Decoupling**: Publishers don't know subscribers
- ✅ **Flexibility**: Runtime topic subscription changes
- ✅ **Still fast**: 4.7M msgs/sec is excellent for most workloads

**When to Avoid Broker:**
- ⚠️ **Ultra-hot paths**: If you need 31M msgs/sec (direct actor-to-actor)
- ⚠️ **Known fixed topology**: Static actor relationships (use direct references)
- ⚠️ **Latency-critical microsecond budgets**: 180 ns matters

**Best Practices:**
```rust
// ✅ GOOD: Use broker for pub-sub patterns (180 ns overhead acceptable)
broker.publish("events", event).await; // Flexible, decoupled

// ✅ GOOD: Use direct actor refs for hot paths (no broker overhead)
actor_ref.send(msg).await; // Fast, but tightly coupled

// 💡 TIP: 6.7x overhead sounds high, but 211 ns is still sub-microsecond
// Broker overhead is acceptable unless proven bottleneck
```

#### Mailbox Efficiency

**Mailbox Operations (100 enqueue/dequeue cycles):**
- **Baseline**: 181.60 ns/operation (enqueue + dequeue pair)
- **Performance Class**: Sub-200ns (excellent for Tokio channels)
- **Throughput**: ~5.5M operations/second

**Mailbox Creation Comparison:**

| Mailbox Type | Creation Time | Use Case |
|--------------|---------------|----------|
| **Unbounded** | 188.55 ns | Default choice, low latency |
| **Bounded (100)** | 244.18 ns | Backpressure, resource limits |
| **Overhead** | +29.5% | Capacity pre-allocation cost |

**When to Use Unbounded:**
- ✅ **Default choice**: 23% faster creation, simpler semantics
- ✅ **Trusted actors**: Internal actors you control
- ✅ **Low message rate**: Memory growth not a concern

**When to Use Bounded:**
- ✅ **Backpressure needed**: Slow consumers, prevent memory bloat
- ✅ **Untrusted actors**: External inputs, rate limiting
- ✅ **Resource limits**: Known capacity constraints

**Best Practices:**
```rust
// ✅ GOOD: Use unbounded for most internal actors (default)
let actor = system.spawn_with_mailbox(MyActor::new(), MailboxType::Unbounded);

// ✅ GOOD: Use bounded for backpressure-sensitive actors
let actor = system.spawn_with_mailbox(
    ExternalActor::new(),
    MailboxType::Bounded(100) // Prevent unbounded growth
);

// 💡 TIP: 29% creation overhead is one-time cost
// Operational performance identical after creation
```

---

### 3. Supervision Characteristics

#### Strategy Overhead Comparison

**Strategy Performance (3 children spawn):**

| Strategy | Time (3 children) | Per-Child | vs OneForOne | Outliers |
|----------|-------------------|-----------|--------------|----------|
| **OneForOne** | 1.2731 µs (single) | 1.2731 µs | Baseline | 6.67% |
| **OneForAll** | 2.9959 µs (batch) | 998.63 ns | -21.6% faster! | 3.33% |
| **RestForOne** | 3.0012 µs (batch) | 1000.4 ns | -21.4% faster! | 10% |
| **Tree (small)** | 3.0073 µs (batch) | 1002.4 ns | -21.3% faster! | 0% |

**Key Insights:**
- ✅ **Strategy choice is semantic, not performance-based**: <1% difference between strategies
- ✅ **Batch spawn efficiency**: 21.6% faster per-child than single spawn
- ✅ **Perfect stability**: Tree construction has 0% outliers (most stable benchmark!)

**Strategy Selection Guidelines:**

**OneForOne** (Independent Children):
- Use when: Child failures are independent
- Performance: 1.273 µs single spawn, 998 ns batch per-child
- Overhead: None (simplest strategy)

**OneForAll** (Coupled Children):
- Use when: All children share state/resources
- Performance: Identical to RestForOne (2.996 µs for 3 children)
- Overhead: None vs RestForOne

**RestForOne** (Ordered Dependencies):
- Use when: Children have startup dependencies
- Performance: 3.001 µs for 3 children (1.0 µs/child)
- Overhead: None vs OneForAll

**Best Practices:**
```rust
// ✅ GOOD: Choose strategy based on semantics, not performance
Supervisor::new(SupervisionStrategy::OneForOne) // Independent failures

// ✅ GOOD: All strategies have identical performance (<1% variance)
Supervisor::new(SupervisionStrategy::OneForAll) // Coupled state
Supervisor::new(SupervisionStrategy::RestForOne) // Dependencies

// 💡 TIP: Strategy overhead is negligible - focus on correctness
// Batch spawn is 21.6% faster regardless of strategy choice
```

#### Restart Latency Expectations

**Child Spawn Latency (Proxy for Restart):**
- **Baseline**: 1.2834 µs (single child via builder)
- **Batch**: 998.63 ns/child (3-child batch)
- **Components**: Builder setup + child registration + strategy config

**Restart Estimate:**
- **Stop existing child**: ~500 ns (cleanup, deregistration)
- **Spawn new child**: ~1,283 ns (measured baseline)
- **Total restart latency**: **~1.8 µs** (estimated)

**Restart Rate Capability:**
- **Single child restarts**: ~556,000 restarts/second
- **Batch restarts (3 children)**: ~1 million restarts/second total

**Best Practices:**
```rust
// ✅ GOOD: Restarts are cheap (~1.8 µs) - don't fear supervision
// Let-it-crash philosophy is performance-viable

// 💡 TIP: 556K restarts/sec means supervision overhead is negligible
// Focus on correct restart strategies, not performance
```

#### Tree Depth Impact

**Small Tree (3 children, 1 level):**
- **Baseline**: 3.0073 µs construction
- **Outliers**: 0% (perfect stability!)
- **Per-child**: 1.002 µs

**Projected Scaling (extrapolated):**
- **2 levels (9 children)**: ~9 µs construction (linear scaling)
- **3 levels (27 children)**: ~27 µs construction
- **Pattern**: ✅ **Linear scaling with tree size** (no depth penalty)

**Tree Traversal (not benchmarked, estimated):**
- **Error escalation**: <100 ns per level (HashMap lookup)
- **Coordinated shutdown**: O(n) children, not O(depth)
- **Health check**: Per-child overhead, not depth-dependent

**Best Practices:**
```rust
// ✅ GOOD: Tree depth has negligible performance impact
// Organize trees for semantics (fault isolation), not performance

// ✅ GOOD: Flat vs deep trees - choose based on restart policies
let flat = supervisor.child(actor1).child(actor2).child(actor3);
let deep = supervisor.child(supervisor2.child(actor3));

// 💡 TIP: Tree depth doesn't affect individual actor performance
// 0% outliers = supervision trees are highly stable
```

---

### 4. Resource Usage Patterns

#### Memory Per Actor Guidelines

**Allocation Time Scaling:**
- **1 actor**: 718.43 ns
- **10 actors**: 742.76 ns/actor (+3.4%)
- **50 actors**: 762.68 ns/actor (+6.2%)

**Actual Memory Size (not measured in benchmarks):**
- **Estimated**: ~200-500 bytes per actor (context + mailbox)
- **Measured**: Allocation *time* only (size requires profiling)
- **Note**: ⏸️ Memory footprint analysis pending future work

**Scaling Recommendations:**

| Actor Count | Allocation Time | Overhead | Expected Mem | Guideline |
|-------------|-----------------|----------|--------------|-----------|
| **1-10** | ~700 ns/actor | Minimal | <5 KB | Negligible cost |
| **10-50** | ~750 ns/actor | +3-6% | <25 KB | Linear scaling |
| **50-1000** | ~800 ns/actor (est.) | +10% | <500 KB | Still cheap |
| **1000-10000** | ~1000 ns/actor (est.) | +20% | <5 MB | Acceptable |

**Best Practices:**
```rust
// ✅ GOOD: Don't pre-optimize actor count
// 762 ns @ 50 actors = <1 µs overhead total

// ✅ GOOD: Scale actors based on workload parallelism
let pool = (0..num_cpus).map(|_| spawn_worker()).collect();

// ⚠️ AVOID: Don't assume "actor pools are expensive"
// Allocation cost is negligible up to thousands of actors

// 💡 TIP: 6% overhead @ 50 actors projects to ~10% @ 1000 actors
// Memory allocation is NOT a scaling bottleneck
```

#### Mailbox Sizing Recommendations

**Bounded vs Unbounded Decision Matrix:**

| Consideration | Unbounded | Bounded |
|---------------|-----------|---------|
| **Creation time** | 188.55 ns ✅ | 244.18 ns (+29.5%) |
| **Memory safety** | ⚠️ Can grow | ✅ Capped |
| **Backpressure** | ❌ No | ✅ Yes |
| **Latency** | ✅ No blocking | ⚠️ Can block sender |
| **Complexity** | ✅ Simple | ⚠️ Handle full mailbox |

**Recommendations:**

**Use Unbounded (Default):**
- ✅ **Trusted actors**: Internal components you control
- ✅ **Low message rate**: <1000 msgs/sec
- ✅ **Fast consumers**: Actor processes faster than receives
- ✅ **Simplicity**: No backpressure logic needed

**Use Bounded:**
- ✅ **Untrusted sources**: External inputs, user requests
- ✅ **Slow consumers**: I/O-bound actors, database writes
- ✅ **Resource limits**: Memory-constrained environments
- ✅ **Rate limiting**: Explicit throughput caps

**Sizing Guidelines (Bounded Mailboxes):**
- **Small (10-50)**: Latency-sensitive, fast turnaround
- **Medium (100-500)**: Standard choice, balances memory/throughput
- **Large (1000+)**: Batch processing, high variance workloads

**Best Practices:**
```rust
// ✅ GOOD: Default to unbounded for simplicity
let actor = system.spawn(MyActor::new()); // Unbounded by default

// ✅ GOOD: Use bounded for external-facing actors
let api_actor = system.spawn_with_mailbox(
    ApiActor::new(),
    MailboxType::Bounded(100) // Cap request queue
);

// ✅ GOOD: Size based on latency requirements
// Small: 10-50 for low-latency (<1ms target)
// Medium: 100-500 for standard workloads
// Large: 1000+ for batch processing

// 💡 TIP: 29% creation overhead is one-time cost
// Operational performance is identical
```

#### System Capacity Planning

**Theoretical Capacity (Based on Benchmarks):**

**Actors:**
- **Spawn rate**: 1.6M actors/second (single spawn)
- **Sustainable**: Limited by memory, not CPU
- **10,000 actors**: ~6.25 ms to spawn all (negligible)

**Messages:**
- **Direct processing**: 31.7M msgs/sec theoretical
- **Broker routing**: 4.7M msgs/sec theoretical
- **Realistic**: 1-5M msgs/sec accounting for business logic

**Supervision:**
- **Child spawns**: 779K/sec (single), 997K/sec (batch)
- **Restarts**: ~556K/sec estimated
- **Negligible**: Supervision overhead <1% in normal operation

**Capacity Planning Formula:**

```
Total System Capacity = min(
    CPU cores * messages_per_core_per_sec,
    Network bandwidth / message_size,
    Memory / (actors * memory_per_actor)
)

Typically bounded by:
1. Business logic CPU (not framework overhead)
2. I/O operations (database, network)
3. Memory for actor state (not actor framework)
```

**Best Practices:**
```rust
// ✅ GOOD: Plan capacity based on business logic, not actor overhead
// Framework overhead: 31 ns/msg (negligible)
// Business logic: 1-100 µs/msg (dominant factor)

// ✅ GOOD: Use benchmarks to validate, not dictate architecture
// 4.7M msgs/sec broker = plenty of headroom for real workloads

// 💡 TIP: Actor framework is NOT your bottleneck
// Focus on optimizing database queries, network I/O, algorithms
```

---

### 5. Best Practices for Performance-Conscious Design

#### Actor Design Patterns

**Minimize Actor State:**
```rust
// ✅ GOOD: Lean actor state (memory efficient)
struct Worker {
    id: u64,           // 8 bytes
    state: WorkState,  // Small enum
}

// ⚠️ ACCEPTABLE: Larger state if needed (actor overhead is minimal)
struct CacheActor {
    cache: HashMap<String, Value>, // Can be large
    // Actor overhead (~200-500 bytes) is negligible vs cache size
}
```

**Batch Operations When Possible:**
```rust
// ✅ GOOD: Batch spawn for ergonomics (not performance)
supervisor.child("worker").spawn_all(vec![Worker::new(); 10]);

// ✅ GOOD: Batch message processing for throughput
async fn handle_batch(&mut self, messages: Vec<Msg>) {
    // Process batch more efficiently than individual messages
}
```

**Choose Message Patterns Wisely:**
```rust
// ✅ GOOD: Fire-and-forget for throughput (211 ns/msg sustained)
actor.send(msg);

// ✅ GOOD: Request-reply for latency (737 ns roundtrip)
let response = actor.request(msg).await;

// ⚠️ AVOID: Request-reply in tight loops (use batching)
for msg in many_messages {
    let _ = actor.request(msg).await; // 737 ns * N = high latency
}
```

#### When to Optimize

**Don't Optimize (Framework is Fast Enough):**
- ❌ **Actor spawn time** (625 ns is negligible)
- ❌ **Supervisor overhead** (<1% difference between strategies)
- ❌ **Mailbox choice** (29% creation difference is one-time)
- ❌ **Message broker** (4.7M msgs/sec is plenty for most apps)

**Do Optimize (Real Bottlenecks):**
- ✅ **Business logic** (database queries, algorithms)
- ✅ **I/O operations** (network, disk, external services)
- ✅ **Message payload size** (serialization cost)
- ✅ **Actor state access patterns** (lock contention, cache misses)

**Performance Debugging Workflow:**
```bash
# 1. Profile first (don't assume)
cargo flamegraph --bin myapp

# 2. Identify actual bottlenecks
# Usually: database queries, network I/O, business logic

# 3. Benchmark before/after optimization
cargo bench

# 4. Verify improvement >5% before merging
# Framework overhead is typically <1% of total time
```

#### Performance Anti-Patterns

**❌ ANTI-PATTERN: Pre-spawn actor pools "for performance"**
```rust
// ❌ BAD: Premature optimization
let pool: Vec<_> = (0..1000).map(|_| spawn_worker()).collect();
// 625 ns/actor * 1000 = 625 µs total (negligible!)
// Memory and complexity cost NOT worth it
```

**✅ BETTER: Spawn on-demand**
```rust
// ✅ GOOD: Spawn as needed
when_work_arrives(|| spawn_worker()) // 625 ns overhead is fine
```

**❌ ANTI-PATTERN: Avoid broker "for performance"**
```rust
// ❌ BAD: Premature optimization
actor_ref.send_direct(msg) // Saves 180 ns but loses flexibility
```

**✅ BETTER: Use broker unless proven bottleneck**
```rust
// ✅ GOOD: Use broker for flexibility (211 ns is fast enough)
broker.publish("topic", msg) // 4.7M msgs/sec is plenty
```

**❌ ANTI-PATTERN: Over-size bounded mailboxes**
```rust
// ❌ BAD: Arbitrary large capacity
MailboxType::Bounded(10000) // "Just in case"
```

**✅ BETTER: Size based on latency requirements**
```rust
// ✅ GOOD: Right-size for use case
MailboxType::Bounded(100) // P99 latency = 100 * msg_processing_time
```

#### Performance Monitoring

**What to Monitor:**
```rust
// ✅ GOOD: Monitor business metrics
- Request latency (end-to-end, not just actor overhead)
- Throughput (requests/sec, not messages/sec)
- Error rate (failed business operations)
- Resource usage (memory growth, CPU saturation)

// ❌ AVOID: Monitoring actor framework internals
// Framework overhead is <1% of total time
```

**Regression Detection:**
```bash
# ✅ GOOD: Automated benchmark regression checks
cargo bench -- --baseline main
# Look for >5% regressions on critical paths

# ✅ GOOD: Profile before/after major changes
cargo flamegraph --before
cargo flamegraph --after
diff before.svg after.svg
```

**Performance Goals:**
```rust
// ✅ GOOD: Set realistic goals based on baseline
// - Message latency: <1ms (achieved: 737 ns = 1,357x better)
// - Throughput: >1M msgs/sec (achieved: 4.7M = 4.7x better)
// - Actor spawn: <10 µs (achieved: 625 ns = 16x better)

// 💡 TIP: Framework performance is NOT your limiting factor
// Focus optimization on business logic, I/O, algorithms
```

---

## Regression Tracking

**Status:** ✅ Workflow Established (October 16, 2025)

Criterion automatically saves benchmark results to `target/criterion/` and compares against previous runs. Use this workflow to detect performance regressions during development.

---

### Baseline Workflow

**Note:** Criterion 0.7 does not support `--save-baseline` flag. Use automatic comparison instead.

**Recommended Workflow:**

```bash
# 1. Establish baseline (first run)
cargo bench

# Results saved to: target/criterion/<benchmark-name>/new/
# Next run will automatically compare against this baseline

# 2. Make code changes
# ... edit src/ files ...

# 3. Run benchmarks again (automatic comparison)
cargo bench

# Criterion compares to previous run and reports:
# - "No change in performance detected"
# - "Performance has regressed" (slower)
# - "Performance has improved" (faster)

# 4. If regression is intentional (architecture change), accept new baseline
# Simply run again - new results become the baseline for next comparison
cargo bench
```

**Manual Baseline Management:**

If you need explicit baseline control, copy the results directory:

```bash
# Save current baseline
cp -r target/criterion target/criterion-baseline-main

# After changes, compare manually
cargo bench
diff -r target/criterion-baseline-main target/criterion

# Restore baseline if needed
cp -r target/criterion-baseline-main target/criterion
```

---

### Interpreting Regression Reports

Criterion highlights statistically significant changes with detailed output:

**Example: Performance Regression (Slower)**
```
actor_spawn_single  time:   [745.23 ns 759.45 ns 774.12 ns]
                    change: [+18.5% +21.6% +24.2%] (p = 0.00 < 0.05)
                    Performance has regressed.
```

**Interpretation:**
- **Estimate**: 759.45 ns (previous: 624.74 ns)
- **Change**: +21.6% slower (134.71 ns increase)
- **Confidence**: 95% CI [+18.5%, +24.2%]
- **Significance**: p = 0.00 < 0.05 (statistically significant)
- **Verdict**: ❌ **Regression detected**

**Example: Performance Improvement (Faster)**
```
message_send_receive time:   [612.34 ns 621.88 ns 632.45 ns]
                     change: [-18.2% -15.6% -12.9%] (p = 0.00 < 0.05)
                     Performance has improved.
```

**Interpretation:**
- **Estimate**: 621.88 ns (previous: 737.16 ns)
- **Change**: -15.6% faster (115.28 ns improvement)
- **Confidence**: 95% CI [-18.2%, -12.9%]
- **Significance**: p = 0.00 < 0.05 (statistically significant)
- **Verdict**: ✅ **Improvement detected**

**Example: No Significant Change**
```
supervisor_child_spawn time:   [1.2756 µs 1.2891 µs 1.3024 µs]
                        change: [-1.8% +0.4% +2.7%] (p = 0.67 > 0.05)
                        No change in performance detected.
```

**Interpretation:**
- **Estimate**: 1.2891 µs (previous: 1.2834 µs)
- **Change**: +0.4% (within noise)
- **Confidence**: 95% CI includes 0% ([-1.8%, +2.7%])
- **Significance**: p = 0.67 > 0.05 (not statistically significant)
- **Verdict**: ➖ **No meaningful change**

---

### Regression Decision Matrix

**When to Take Action:**

| Criterion Output | Change | P-value | Action Required |
|------------------|--------|---------|-----------------|
| "Performance has regressed" | +5% to +10% | <0.05 | ⚠️ **Investigate** - Understand cause |
| "Performance has regressed" | +10% to +20% | <0.05 | ❌ **Fix or justify** - Significant slowdown |
| "Performance has regressed" | >+20% | <0.05 | 🚨 **Must fix** - Critical regression |
| "Performance has improved" | -5% to -20% | <0.05 | ✅ **Document** - Note optimization |
| "Performance has improved" | >-20% | <0.05 | 🎉 **Celebrate** - Major improvement |
| "No change detected" | Any | >0.05 | ✅ **Acceptable** - Within noise |

**Critical Path Thresholds (Stricter):**

For critical operations, use stricter thresholds:

| Benchmark | Baseline | Max Acceptable Regression | Rationale |
|-----------|----------|---------------------------|-----------|
| `actor_spawn_single` | 624.74 ns | <5% (+31 ns) | Frequent operation |
| `message_send_receive` | 737.16 ns | <3% (+22 ns) | Critical messaging path |
| `message_throughput` | 211.88 ns/msg | <5% (+10 ns) | High-frequency sustained |
| `supervisor_child_spawn` | 1.2834 µs | <10% (+128 ns) | Infrequent operation |
| `memory_per_actor/50` | 762.68 ns/actor | <10% (+76 ns) | Scaling characteristic |

**Investigation Checklist:**

When regression detected (>5% on critical path):

1. ✅ **Verify reproducibility**: Run benchmark 3+ times
2. ✅ **Check system load**: Ensure idle system, no background tasks
3. ✅ **Review code changes**: Identify potential performance impact
4. ✅ **Profile with flamegraph**: `cargo flamegraph --bench <benchmark>`
5. ✅ **Compare flame graphs**: Before vs after optimization
6. ✅ **Document decision**: Accept regression if justified (features, correctness)

**Example Investigation:**

```bash
# 1. Verify regression is real
cargo bench --bench actor_benchmarks -- actor_spawn_single
cargo bench --bench actor_benchmarks -- actor_spawn_single
cargo bench --bench actor_benchmarks -- actor_spawn_single
# If all 3 runs show +20%, it's real

# 2. Profile to find cause
cargo install flamegraph
cargo flamegraph --bench actor_benchmarks -- --bench actor_spawn_single

# 3. Compare flamegraphs
# Open flamegraph.svg and look for new hot paths

# 4. Fix or justify
# Option A: Optimize hot path to restore performance
# Option B: Document why regression is acceptable (new feature, correctness)
```

---

### Acceptable Regression Scenarios

**When regression is justified:**

✅ **New features with documented tradeoffs:**
```
Example: Added logging to actor spawn
Regression: +15% (624 ns → 718 ns)
Justification: Comprehensive audit logging requirement (security)
Decision: Accept regression, document in ADR
```

✅ **Correctness over performance:**
```
Example: Fixed race condition in message broker
Regression: +8% (211 ns → 228 ns)
Justification: Bug fix prevents data loss
Decision: Accept regression, safety is paramount
```

✅ **Architecture improvements:**
```
Example: Refactored to eliminate trait objects (§6.2 compliance)
Regression: +3% temporarily (expected to improve after optimization)
Justification: Better long-term architecture, static dispatch
Decision: Accept short-term regression for long-term gain
```

❌ **Unacceptable regressions:**
```
Example: Accidental heap allocation in hot path
Regression: +50% (624 ns → 936 ns)
Justification: None - coding error
Decision: Must fix before merge
```

---

## Contributing Guidelines

### Adding New Benchmarks

1. **Identify what to measure:** Specific operation or pattern
2. **Choose appropriate category:** Actor, message, supervision, or resource
3. **Follow naming conventions:** `category_operation_detail`
4. **Use realistic scenarios:** Not just synthetic microbenchmarks
5. **Document expected results:** Add to this guide

### Benchmark Code Standards

**Required patterns:**

```rust
// ✅ CORRECT: Import organization (§2.1)
// Layer 1: Standard library
use std::time::Duration;

// Layer 2: Third-party
use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Layer 3: Internal
use airssys_rt::actor::Actor;

// ✅ CORRECT: Use black_box for compiler optimization prevention
black_box(actor);

// ✅ CORRECT: Resource-conscious configuration
fn configure_criterion() -> Criterion {
    Criterion::default()
        .sample_size(30)
        .measurement_time(Duration::from_secs(5))
        .without_plots()
}

// ❌ FORBIDDEN: Large-scale stress tests
// Don't create 10,000 actors - use 50 max

// ❌ FORBIDDEN: Non-deterministic code
// Avoid randomness, I/O, or time-dependent logic
```

### Review Checklist

Before submitting benchmark changes:

- [ ] Benchmark compiles without warnings
- [ ] Runs in <5 minutes for full suite
- [ ] Uses realistic, not synthetic, workloads
- [ ] Documented in BENCHMARKING.md
- [ ] Baseline saved for comparison
- [ ] No performance regressions unexplained

---

## Troubleshooting

### Common Issues

**Issue: High variance in results**
```
Solution: Close other applications, disable power management,
         run on AC power, check for background processes
```

**Issue: Benchmark hangs or times out**
```
Solution: Check for deadlocks, infinite loops, or blocking operations
         Use `cargo bench -- --verbose` for debugging
```

**Issue: Out of memory errors**
```
Solution: Reduce actor count in resource benchmarks,
         check for memory leaks in test code
```

**Issue: Compilation errors**
```
Solution: Ensure all dependencies installed: `cargo update`
         Check Rust version: `rustc --version` (need 1.70+)
```

### Performance Tips

**For accurate measurements:**

✅ Run on idle system (no other applications)  
✅ Use AC power (disable battery saving)  
✅ Disable CPU frequency scaling if possible  
✅ Close background services (indexing, backups)  
✅ Run multiple times to verify consistency  

**For debugging slow benchmarks:**

```bash
# Profile with flamegraph
cargo install flamegraph
cargo flamegraph --bench actor_benchmarks

# Profile with perf (Linux)
perf record -g cargo bench actor_
perf report
```

---

## References

### Documentation

- [Criterion.rs User Guide](https://bheisler.github.io/criterion.rs/book/)
- [ADR-RT-010: Baseline-First Performance Strategy](../.copilot/memory_bank/sub_projects/airssys-rt/docs/adr/adr_rt_010_baseline_first_performance_strategy.md)
- [RT-TASK-008: Performance Baseline Measurement](../.copilot/memory_bank/sub_projects/airssys-rt/tasks/task_008_performance_features.md)

### Tools

- **Criterion:** Statistical benchmarking framework
- **Flamegraph:** CPU profiling and visualization
- **Perf:** Linux performance analysis tools
- **Instruments:** macOS performance profiling (Xcode)

---

**Questions or issues?** Please open a GitHub issue or discuss in the development channel.

**Last updated:** October 16, 2025 by RT-TASK-008 Phase 1
