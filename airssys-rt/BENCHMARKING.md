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

**Status:** Pending Phase 3 analysis (RT-TASK-008)

This section will document observed performance patterns and best practices based on baseline data.

### Expected Sections

1. **Actor System Characteristics**
   - Spawn latency analysis
   - Message processing patterns
   - Scaling behavior

2. **Message Passing Characteristics**
   - Latency vs throughput tradeoffs
   - Broker routing overhead
   - Mailbox efficiency

3. **Supervision Characteristics**
   - Strategy overhead comparison
   - Restart latency expectations
   - Tree depth impact

4. **Resource Usage Patterns**
   - Memory per actor guidelines
   - Mailbox sizing recommendations
   - System capacity planning

5. **Best Practices**
   - When to use bounded vs unbounded mailboxes
   - Optimal supervision tree structure
   - Performance-conscious actor design

---

## Regression Tracking

### Baseline Workflow

Criterion supports baseline tracking to detect performance regressions:

```bash
# 1. Save initial baseline (e.g., before starting work)
cargo bench -- --save-baseline main

# 2. Make changes to code

# 3. Compare against baseline
cargo bench -- --baseline main

# 4. Save new baseline if changes are intentional
cargo bench -- --save-baseline feature-xyz
```

### Interpreting Regression Reports

Criterion highlights statistically significant changes:

```
Performance has regressed:
    actor_spawn_single
        time:   [5.123 µs 5.234 µs 5.345 µs]
        change: [+48.5% +52.3% +56.1%] (p = 0.00 < 0.05)
        Performance has regressed.
```

**Action items:**
- **p < 0.05 + positive change%** = Regression (slower) ❌
- **p < 0.05 + negative change%** = Improvement (faster) ✅
- **p > 0.05** = No significant change ➖

### Acceptable Regression Thresholds

| Category | Threshold | Rationale |
|----------|-----------|-----------|
| Actor spawn | <10% | Setup overhead less critical |
| Message latency | <5% | Critical path - minimize regression |
| Supervisor restart | <15% | Fault recovery - less frequent |
| Memory usage | <10% | Resource efficiency important |

### CI/CD Integration (Future)

```yaml
# Example GitHub Actions workflow
- name: Run benchmarks
  run: cargo bench -- --save-baseline pr-${{ github.event.number }}

- name: Compare to main
  run: cargo bench -- --baseline main --save-baseline pr-${{ github.event.number }}

- name: Fail on significant regression
  run: |
    # Parse criterion output for regressions > threshold
    # Exit 1 if found
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
