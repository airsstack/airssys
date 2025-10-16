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

**Status:** Pending Phase 2 measurement (RT-TASK-008)

This section will be updated after baseline data collection. Expected completion: October 17, 2025.

### Measurement Methodology

1. **Hardware:** macOS (primary development platform)
2. **Conditions:** Idle system, AC power, no thermal throttling
3. **Runs:** 30 samples per benchmark (default criterion config)
4. **Statistics:** 95% confidence intervals

### Planned Results Format

```markdown
### Actor System Baseline

| Benchmark | Mean | Median | p95 | p99 | Std Dev |
|-----------|------|--------|-----|-----|---------|
| actor_spawn_single | TBD | TBD | TBD | TBD | TBD |
| actor_spawn_batch_small | TBD | TBD | TBD | TBD | TBD |
| actor_message_throughput | TBD | TBD | TBD | TBD | TBD |
```

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
