# ADR-RT-010: Baseline-First Performance Strategy

**Status:** Accepted  
**Date:** 2025-10-15  
**Deciders:** Project Team  
**Related Tasks:** RT-TASK-008 (Performance Baseline Measurement)  
**Supersedes:** Original RT-TASK-008 scope (premature optimization)

## Context

During planning for RT-TASK-008, the original scope focused on implementing performance optimizations:
- Message routing optimization
- Advanced load balancing strategies
- Message batching for high throughput
- Routing caches and optimizations
- Speculative performance targets (e.g., "10,000+ concurrent actors support")

**Problem**: This approach violates YAGNI principles and constitutes premature optimization:
1. No performance issues reported or proven in current implementation
2. No real-world data on actual bottlenecks
3. Speculative targets without evidence-based requirements
4. Risk of optimizing non-critical paths
5. Potential for increased complexity without measurable benefit

**Current State**:
- Runtime architecture already uses zero-cost abstractions
- Generic constraints (no `Box<dyn Trait>` in hot paths)
- Static dispatch throughout core components
- Minimal allocations (Arc-based sharing patterns)
- No performance complaints or identified bottlenecks

## Decision

**Adopt a baseline-first performance strategy for airssys-rt:**

### Primary Decision: Measure Before Optimize

**RT-TASK-008 will focus on establishing performance baselines, NOT implementing optimizations.**

**New Scope**:
1. **Establish Benchmark Infrastructure** using `criterion`
2. **Measure Baseline Performance** of current architecture across all core operations
3. **Document Performance Characteristics** with honest, data-driven metrics
4. **Identify Actual Bottlenecks** through measurement, not assumption
5. **Enable Future Optimization** with data-driven prioritization

**Removed from Scope**:
- ❌ Message routing optimization (no data proving it's needed)
- ❌ Advanced load balancing strategies (no evidence current approach is insufficient)
- ❌ Message batching (no proof throughput is a bottleneck)
- ❌ Routing caches (no data showing address resolution overhead)
- ❌ Speculative performance targets (10k actors, etc.)

### Secondary Decisions

**1. No Performance Targets Without Data**
- Will NOT set arbitrary targets (10k actors, sub-microsecond latency, etc.)
- Will measure current capabilities and document them honestly
- Future targets will be based on real-world requirements, not speculation

**2. Comprehensive Benchmark Coverage**
- Actor system operations (spawn, execution, lifecycle)
- Message passing (latency, throughput, routing)
- Supervision operations (restart strategies, health checks)
- Resource usage (memory per actor, CPU patterns)

**3. Statistical Rigor**
- Use `criterion` for statistical analysis
- Establish baseline tracking for regression detection
- Measure variance and reproducibility
- Document measurement methodology

**4. Future Optimization Process**
If baseline reveals bottlenecks:
1. Profile with flamegraph/perf to identify root cause
2. Create focused optimization task with clear goals
3. Implement with benchmark validation
4. Measure impact against baseline
5. Document improvement

If baseline shows good performance:
1. Document current characteristics
2. Defer optimization until real-world needs emerge
3. Focus on other priorities (documentation, features)
4. Maintain regression tracking

## Rationale

### Why Baseline-First?

**1. YAGNI Principle Compliance**
> "You Aren't Gonna Need It" - Don't implement optimizations without proven need

- Current architecture shows no performance issues
- Zero-cost abstractions already in place
- Premature optimization wastes time and adds complexity

**2. Data-Driven Decision Making**
> "In God we trust, all others must bring data" - W. Edwards Deming

- Baselines provide objective performance data
- Identifies real bottlenecks vs assumed ones
- Enables informed cost-benefit analysis
- Prevents bikeshedding about performance

**3. Transparency and Honesty**
> "Premature optimization is the root of all evil" - Donald Knuth

- Users deserve honest performance characteristics
- No marketing hype or speculative claims
- Real measurements build trust
- Clear about current capabilities and limitations

**4. Regression Detection**
- Baseline tracking catches performance degradation early
- Validates that future changes don't harm performance
- Enables safe refactoring and feature additions

**5. Efficient Resource Allocation**
- 4 days for measurement vs 5-7 days for speculative optimization
- Focus on high-impact work (documentation, features)
- Optimization only when data justifies the effort

### Why NOT Premature Optimization?

**1. No Evidence of Need**
- Zero performance complaints from development work
- No profiling data showing bottlenecks
- No real-world workload requirements
- Architecture already optimized at design level

**2. Risk of Wasted Effort**
- Optimizing non-critical paths provides no value
- Complexity added without measurable benefit
- Maintenance burden for negligible gains
- Time better spent on documentation or features

**3. May Harm More Than Help**
- Optimization often trades readability for speed
- Increased complexity makes maintenance harder
- Potential for introducing bugs in optimization code
- May optimize wrong things without data

**4. Violates Microsoft Rust Guidelines**
> M-DESIGN-FOR-AI: Prioritize clarity and idiomatic code over premature optimization

Current zero-cost abstraction approach already optimal for Rust:
- Generics resolve at compile time
- Static dispatch everywhere
- No runtime overhead
- Excellent compiler optimization

## Consequences

### Positive Consequences

**1. Clear Performance Transparency**
- Users know actual runtime characteristics
- Honest documentation builds trust
- Clear about current capabilities

**2. Data-Driven Optimization Roadmap**
- Future work prioritized by measured impact
- Clear cost-benefit analysis for optimizations
- Avoids wasted effort on low-impact changes

**3. Regression Detection**
- Baseline tracking prevents performance degradation
- Safe refactoring with performance validation
- CI/CD integration potential

**4. Efficient Development**
- 4 days for baselines vs 5-7 days for speculation
- Focus on high-value work (docs, features)
- Optimization when justified by data

**5. YAGNI Compliance**
- Don't build what isn't proven necessary
- Maintain simplicity and clarity
- Reduce maintenance burden

### Negative Consequences

**1. No Immediate Optimizations**
- Users expecting performance improvements won't see them yet
- **Mitigation**: Current architecture already performant, no complaints

**2. Delayed Performance Features**
- Advanced load balancing, caching, etc. deferred
- **Mitigation**: Will implement when data shows need

**3. Requires Discipline**
- Team must resist urge to optimize prematurely
- **Mitigation**: Document this decision clearly, enforce in reviews

### Neutral Consequences

**1. Benchmark Maintenance**
- Benchmarks need updating when APIs change
- **Trade-off**: Small cost for regression detection value

**2. Baseline Data Storage**
- Criterion baselines committed to repo
- **Trade-off**: Minor repo size increase for valuable data

## Implementation

### RT-TASK-008 New Scope

**Phase 1: Infrastructure (1 day)**
- Set up `benches/` with criterion
- Create benchmark harnesses
- Configure baseline tracking

**Phase 2: Measurement (2 days)**
- Actor system benchmarks (6+)
- Message passing benchmarks (7+)
- Supervision benchmarks (7+)
- Resource usage benchmarks (6+)

**Phase 3: Analysis & Documentation (1 day)**
- Collect baseline data
- Analyze for bottlenecks
- Document characteristics
- Create optimization roadmap (data-driven)

**Total**: 4 days (reduced from 5-7)

### Benchmark Categories

1. **Actor System**: spawn, execution, lifecycle, state access
2. **Message Passing**: latency, throughput, routing, mailbox operations
3. **Supervision**: restart strategies, health checks, tree operations
4. **Resource Usage**: memory footprint, CPU patterns, task overhead

### Documentation Deliverables

1. **Baseline Performance Report** (memory bank)
   - Executive summary of metrics
   - Detailed benchmark results
   - Bottleneck analysis
   - Regression thresholds

2. **Performance Characteristics Guide** (rustdoc/mdBook)
   - Component performance characteristics
   - Usage patterns impact
   - Best practices

3. **Future Optimization Roadmap** (memory bank)
   - Data-driven priorities
   - Impact estimates
   - Cost-benefit analysis

## Alternatives Considered

### Alternative 1: Implement Optimizations Now
**Rejected because**:
- No data proving optimizations needed
- Violates YAGNI principle
- Risk of wasted effort
- May add complexity without benefit

### Alternative 2: No Performance Work
**Rejected because**:
- Need baseline data for future decisions
- Regression detection valuable
- Transparency about characteristics important

### Alternative 3: Minimal Benchmarks Only
**Rejected because**:
- Comprehensive coverage needed for informed decisions
- Resource usage metrics critical for production planning
- Supervision overhead often overlooked but important

### Alternative 4: Profile Real Workloads First
**Rejected because**:
- No real workloads available yet (runtime is foundation)
- Synthetic benchmarks provide reproducible baselines
- Can add real-world benchmarks later when use cases emerge

## Success Criteria

**This decision is successful if**:

1. ✅ Baseline data collected for all core operations
2. ✅ Performance characteristics honestly documented
3. ✅ Regression detection enabled
4. ✅ Future optimization roadmap is data-driven
5. ✅ No premature optimizations implemented
6. ✅ Team understands baseline-first approach
7. ✅ Users have transparent performance information

**This decision fails if**:

1. ❌ Optimizations implemented without baseline data
2. ❌ Speculative performance targets documented
3. ❌ Benchmarks show current architecture is inadequate
4. ❌ Team reverts to premature optimization habits

## Related Documents

- **RT-TASK-008**: Performance Baseline Measurement (updated scope)
- **Workspace Standards**: §6.1 (YAGNI Principles)
- **Microsoft Rust Guidelines**: M-DESIGN-FOR-AI (clarity over premature optimization)
- **Donald Knuth**: "Premature optimization is the root of all evil"
- **W. Edwards Deming**: "In God we trust, all others must bring data"

## Approval

**Accepted**: 2025-10-15  
**Reason**: Aligns with YAGNI principles, enables data-driven optimization, provides transparency

## Review Schedule

**Review Date**: After RT-TASK-008 completion
**Review Criteria**: 
- Are baselines comprehensive and informative?
- Do baselines reveal any unexpected bottlenecks?
- Is baseline tracking preventing regressions?
- Has team maintained discipline against premature optimization?

---

**Decision**: Measure first, optimize later, based on data, not speculation.
