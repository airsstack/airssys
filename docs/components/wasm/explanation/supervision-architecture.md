# Supervision Architecture Explanation

This document explains why supervision is critical for production ComponentActor systems, the design decisions behind the supervision architecture, and the tradeoffs involved in fault tolerance and automatic recovery.

## Why Supervision is Critical

### The Problem: Component Failures in Production

**Reality of Production Systems:**

- Components crash due to bugs, resource exhaustion, or external failures
- Network timeouts cause component hangs
- Memory leaks eventually cause OOM crashes
- External dependencies fail (databases, APIs)
- Hardware failures (disk full, network partition)

**Without Supervision:**
```
Component A crashes
   ↓
System loses functionality
   ↓
Manual intervention required (restart component)
   ↓
Minutes to hours of downtime
   ↓
User-visible impact
```

**With Supervision:**
```
Component A crashes
   ↓
Supervisor detects failure (health check or crash signal)
   ↓
Supervisor restarts Component A automatically (1-60s delay)
   ↓
Component A recovers
   ↓
System self-heals (minimal downtime)
```

**Benefits:**
1. **Automatic Recovery**: No manual intervention needed (reduces MTTR from hours to seconds)
2. **Fault Isolation**: One component crash doesn't affect others
3. **High Availability**: System remains operational during failures (99.9%+ uptime possible)
4. **Graceful Degradation**: Partial functionality maintained even when components fail

### Erlang/OTP Inspiration

ComponentActor supervision is inspired by Erlang/OTP's "Let it Crash" philosophy:

**Erlang/OTP Principles:**
1. **Fail Fast**: Don't try to handle every error, crash and restart
2. **Supervision Trees**: Organize processes hierarchically with supervisors
3. **Isolated Failure**: Process crashes don't affect other processes
4. **Automatic Recovery**: Supervisor restarts crashed processes

**Applied to ComponentActor:**
```rust
// Don't write defensive code for every error
pub async fn process_message(&self, msg: Message) -> Result<(), Error> {
    // If this panics, supervisor will restart component
    let result = self.external_api_call(msg).await?;
    self.update_state(result).await?;
    Ok(())
}

// Supervisor handles crash automatically
// Result: Simpler code, automatic recovery
```

**Tradeoff:**

- **Pro**: Simpler component code (no complex error handling)
- **Pro**: Guaranteed recovery path (supervisor restarts component)
- **Con**: State lost on crash (must be restored from persistent storage)
- **Con**: Restart overhead (~1µs full lifecycle, measured in Task 6.2)

## Design Decisions

### Decision 1: Isolated Restart vs System-Wide Restart

**Option A: Isolated Restart (CHOSEN)**

Restart only the crashed component:

```
Component A crashes
   ↓
Supervisor restarts only Component A
   ↓
Components B, C, D continue running unaffected
```

**Advantages:**

- Minimal disruption (only crashed component affected)
- Fast recovery (286ns spawn + 1.49µs lifecycle, Task 6.2)
- Other components maintain state
- Scalable (100+ components can fail independently)

**Disadvantages:**

- Component state lost (must restore from storage)
- Dependent components may see temporary failures

**Option B: System-Wide Restart (REJECTED)**

Restart entire system when any component crashes:

```
Component A crashes
   ↓
Supervisor restarts entire system (all components)
   ↓
All components lose state
   ↓
Long recovery time (N × lifecycle time)
```

**Why Rejected:**

- High disruption (all components affected by one failure)
- Slow recovery (1,000 components × 1.49µs = 1.49ms total)
- Unnecessary state loss (components B-Z were working fine)
- Poor scalability (downtime proportional to component count)

**Decision Rationale:** Isolated restart provides better fault isolation, faster recovery, and minimal user impact.

### Decision 2: Supervision Tree Structure

**Option A: Flat Supervision (CURRENT)**

Single supervisor manages all components:

```
SupervisorNode
  ├─ Component A
  ├─ Component B
  ├─ Component C
  └─ Component D
```

**Advantages:**

- Simple to implement (one supervisor instance)
- Easy to reason about (all restart policies centralized)
- Low overhead (one supervisor, minimal memory)
- Sufficient for most use cases (< 100 components)

**Disadvantages:**

- All components share restart policy (can't customize per-component easily)
- Single point of failure (if supervisor crashes, all components orphaned)

**Option B: Hierarchical Supervision (FUTURE)**

Nested supervisors with different policies:

```
RootSupervisor (max_restarts: 3, exponential backoff)
  ├─ APISupervisor (max_restarts: 10, immediate restart)
  │   ├─ APIGateway
  │   └─ APIProcessor
  └─ DataSupervisor (max_restarts: 5, delayed restart)
      ├─ DataIngress
      └─ DataEgress
```

**Advantages:**

- Customizable policies per component group
- Logical isolation (API failures don't affect Data components)
- Supervisor redundancy (root supervisor can restart child supervisors)
- Better for large systems (100+ components)

**Disadvantages:**

- Complex to implement (nested supervisor state)
- Higher overhead (multiple supervisor instances)
- More difficult to debug (multi-level failure propagation)

**Decision Rationale:** Start with flat supervision (YAGNI principle). Implement hierarchical supervision only when proven necessary (>100 components or different restart policies required).

### Decision 3: Restart Strategies

**Three Strategies Provided:**

**1. Immediate Restart**
```rust
RestartStrategy::Immediate
```

**When to Use:**

- Development and testing (fast iteration)
- Transient failures expected (network hiccup)
- Fast recovery more important than stability

**Behavior:**

- Restart immediately on crash (no delay)
- High restart rate possible (max_restarts limit prevents infinite loop)

**Tradeoff:**

- **Pro**: Fastest recovery (1.49µs lifecycle, Task 6.2)
- **Con**: Can amplify problems (resource exhaustion causes immediate re-crash)

**2. Delayed Restart**
```rust
RestartStrategy::Delayed { delay: Duration::from_secs(5) }
```

**When to Use:**

- External dependency failures (give API time to recover)
- Known transient issues (cache refresh, DNS resolution)
- Production default for stable components

**Behavior:**

- Wait fixed delay before restart (5s typical)
- Prevents restart storms (5s cooldown between attempts)

**Tradeoff:**

- **Pro**: Prevents rapid restart loops
- **Con**: 5s downtime per restart

**3. Exponential Backoff (RECOMMENDED)**
```rust
RestartStrategy::ExponentialBackoff {
    initial_delay: Duration::from_secs(1),
    max_delay: Duration::from_secs(60),
    multiplier: 2.0,
}
```

**When to Use:**

- Production default (recommended)
- Unknown failure causes (adaptive approach)
- Persistent failures possible

**Behavior:**

- 1st restart: 1s delay
- 2nd restart: 2s delay (1s × 2.0)
- 3rd restart: 4s delay
- 4th restart: 8s delay
- Nth restart: capped at 60s

**Tradeoff:**

- **Pro**: Fast recovery for transient failures (1s first attempt)
- **Pro**: Prevents restart storms for persistent failures (60s max delay)
- **Pro**: Adaptive to failure patterns
- **Con**: Slightly complex to implement (tracking restart count)

**Decision Rationale:** Provide all three strategies. Recommend exponential backoff for production (balances recovery speed and stability).

## Tradeoffs and Benefits

### Tradeoff 1: Automatic Restart vs Manual Intervention

**Automatic Restart (CHOSEN):**

**Benefits:**

- Fast recovery (seconds vs minutes/hours)
- No human involvement (24/7 automated recovery)
- Consistent behavior (same recovery process every time)
- Lower operational overhead (no on-call escalation for every crash)

**Costs:**

- State loss on restart (must restore from storage)
- Potential for restart loops (mitigated by max_restarts limit)
- Masks underlying issues (component repeatedly crashes/restarts)

**Manual Intervention:**

**Benefits:**

- Human can diagnose root cause before restart
- Prevents restart loops (human decides when to restart)
- State can be preserved (debug crashed component)

**Costs:**

- Slow recovery (minutes to hours)
- Requires on-call staff (24/7 availability)
- Inconsistent (depends on human availability/expertise)
- High operational overhead (manual restarts expensive)

**Decision Rationale:** Automatic restart is superior for production. Manual intervention reserved for persistent failures exceeding max_restarts limit.

### Tradeoff 2: Restart Limits vs Infinite Retries

**Restart Limits (CHOSEN):**

```rust
SupervisorConfig {
    max_restarts: 5,           // Stop after 5 restarts
    within_duration: Duration::from_secs(60),
}
```

**Benefits:**

- Prevents infinite loops (component permanently crashes → eventually stops)
- Resource protection (doesn't exhaust CPU/memory with restart attempts)
- Clear failure signal (max restarts exceeded → alert ops team)
- Detects persistent issues (5 crashes in 60s indicates broken component)

**Costs:**

- Component eventually stops (functionality lost after 5 crashes)
- Must manually restart after limit exceeded (ops intervention required)

**Infinite Retries:**

```rust
SupervisorConfig {
    max_restarts: u32::MAX,    // Never stop retrying
    within_duration: Duration::from_secs(60),
}
```

**Benefits:**

- Never gives up (keeps trying forever)
- Eventual recovery possible (if issue self-resolves)

**Costs:**

- Resource exhaustion (CPU/memory wasted on infinite restart attempts)
- Masks persistent issues (component repeatedly crashes, never surfaces as alert)
- No clear failure signal (how do you know component is broken?)

**Decision Rationale:** Restart limits are essential. Recommended: 3-10 restarts (catches transient failures, stops persistent failures).

### Tradeoff 3: Health Checks vs Crash Detection

**Crash Detection (CURRENT):**

Supervisor detects component crash (panic, process termination):

**Benefits:**

- Zero overhead (no periodic health checks)
- Immediate detection (crash signal immediate)
- Simple to implement (no health check protocol)

**Costs:**

- Only detects crashes (not hangs or degraded performance)
- Misses deadlocks (component alive but not responding)

**Health Checks (FUTURE):**

Supervisor periodically checks component health:

```rust
let health_config = HealthCheckConfig {
    interval: Duration::from_secs(10),  // Check every 10s
    timeout: Duration::from_secs(5),    // 5s timeout
    unhealthy_threshold: 3,             // Restart after 3 consecutive failures
};
```

**Benefits:**

- Detects hangs (component not responding → restart)
- Detects degraded performance (slow health check → restart)
- Proactive (catch issues before user impact)

**Costs:**

- Overhead (health check every 10s per component)
- False positives (slow health check != component broken)
- Complex to implement (health check protocol, timeout handling)

**Decision Rationale:** Start with crash detection (YAGNI). Add health checks when proven necessary (production deployments with hangs/deadlocks).

## Integration with ActorSystem Supervision

### Layered Supervision Architecture

ComponentActor supervision builds on airssys-rt's ActorSystem:

```
Layer 3: SupervisorNode (ComponentActor-aware)
   ├─ Restart policies (immediate, delayed, exponential)
   ├─ Component-specific configuration
   └─ Health monitoring (future)

Layer 2: ActorSystem Supervision (Actor-level)
   ├─ Actor spawn/stop
   ├─ Message routing
   └─ Basic failure detection (crash signals)

Layer 1: Tokio Runtime (Task-level)
   ├─ Task spawning
   ├─ Panic handling
   └─ Resource limits
```

**Separation of Concerns:**

- **Layer 1 (Tokio)**: Low-level task management
- **Layer 2 (ActorSystem)**: Actor lifecycle and messaging
- **Layer 3 (SupervisorNode)**: High-level restart policies and component-specific logic

**Integration Pattern:**

```rust
// SupervisorNode uses ActorSystem to spawn components
let component_ref = actor_system.spawn_actor(component).await?;

// SupervisorNode monitors component via ActorSystem
actor_system.monitor_actor(component_ref, |event| {
    match event {
        ActorEvent::Crashed { actor_id, error } => {
            // SupervisorNode restart logic
            supervisor.handle_crash(actor_id, error).await;
        }
        ActorEvent::Stopped { actor_id } => {
            // Normal stop, no restart
        }
    }
}).await;
```

**Benefits of Layered Approach:**

- **Reusability**: ActorSystem supervision useful for non-WASM actors
- **Testability**: Can test SupervisorNode independent of ActorSystem
- **Flexibility**: Different supervision strategies for different actor types
- **Maintainability**: Clear boundaries between layers (ADR-WASM-018 compliance)

## Failure Isolation Guarantees

### Component Isolation

**Guarantee 1: Memory Isolation**

Components have separate memory spaces (WASM linear memory):

```
Component A memory: [0x0000 - 0x1000]
Component B memory: [0x1000 - 0x2000]
```

**Result:**

- Component A crash cannot corrupt Component B memory
- Memory leak in A doesn't affect B
- Buffer overflow in A cannot escape to B

**Guarantee 2: State Isolation**

Components have separate state:

```rust
#[derive(Clone)]
pub struct ComponentA {
    state: Arc<RwLock<StateA>>,  // Only Component A accesses
}

#[derive(Clone)]
pub struct ComponentB {
    state: Arc<RwLock<StateB>>,  // Only Component B accesses
}
```

**Result:**

- Component A crash loses only StateA
- Component B state unaffected
- Restart restores only StateA

**Guarantee 3: Actor Isolation (ADR-WASM-006)**

Each component runs in separate Actor:

```
Actor 1: Component A (isolated message queue, isolated execution)
Actor 2: Component B (isolated message queue, isolated execution)
```

**Result:**

- Component A panic doesn't crash Actor 2
- Component A slow processing doesn't block Actor 2
- Message to A doesn't affect B's message queue

**Validation:** Task 6.1 integration tests validated isolation (945 tests, 100% pass).

### Cascading Failure Prevention

**Problem:**
```
Component A crashes
   ↓
Component B depends on A → B fails too
   ↓
Component C depends on B → C fails too
   ↓
Entire system down
```

**Solution 1: Restart Limits**

```rust
SupervisorConfig {
    max_restarts: 5,
    within_duration: Duration::from_secs(60),
}
```

**Behavior:**

- Component A crashes 5 times in 60s → permanently stopped
- Component B sees A down → enters degraded mode (doesn't crash)
- Cascading failure prevented (only A stopped, B and C operational)

**Solution 2: Circuit Breaker Pattern**

```rust
pub struct CircuitBreaker {
    failure_count: AtomicU64,
    failure_threshold: u64,  // Open circuit after N failures
    recovery_timeout: Duration,  // Try to close after timeout
}
```

**Behavior:**

- Component B calls A repeatedly → A crashes
- Circuit breaker opens after 5 failures → B stops calling A
- B enters degraded mode (no longer depends on A)
- After 60s timeout → circuit breaker tries to close (call A again)

**Result:** Dependent components degrade gracefully instead of crashing.

### Supervision Tree Patterns

**Pattern 1: Flat Tree (Current)**

```
SupervisorNode
  ├─ ComponentA (isolated)
  ├─ ComponentB (isolated)
  └─ ComponentC (isolated)
```

**Failure Behavior:**

- ComponentA crashes → only A restarts
- ComponentB and C unaffected
- Simple, effective for < 100 components

**Pattern 2: Hierarchical Tree (Future)**

```
RootSupervisor
  ├─ APISupervisor
  │   ├─ APIGateway (crashes)
  │   └─ APIProcessor (unaffected)
  └─ DataSupervisor (unaffected)
      ├─ DataIngress
      └─ DataEgress
```

**Failure Behavior:**

- APIGateway crashes → APISupervisor restarts APIGateway
- DataSupervisor unaffected (different supervisor)
- Logical isolation (API failures don't affect Data)

**When to Use Hierarchical:**

- > 100 components
- Different restart policies per component group
- Clear logical boundaries (API, data, compute)

## Historical Context

### Evolution from Phase 4

**Phase 4: ComponentActor Foundation**

Initial design (no supervision):

```rust
pub struct ComponentActor {
    // Dual-trait pattern (Child + Actor)
    // No supervision integration
}
```

**Problems:**

- Component crashes required manual restart
- No automatic recovery
- Production deployment risky (downtime on every crash)

**Phase 5: Supervisor Integration**

Added SupervisorNode integration:

```rust
pub struct ComponentActor {
    // Dual-trait pattern maintained
    // Added supervisor awareness
    supervisor_ref: Option<ActorRef<SupervisorNode>>,
}
```

**Improvements:**

- Automatic restart on crash
- Configurable restart strategies
- Production-ready fault tolerance

**Design Lessons:**
1. **Separation of Concerns**: ComponentActor (lifecycle) vs SupervisorNode (restart) kept separate
2. **Backward Compatibility**: Components without supervision still work (supervisor optional)
3. **Performance**: Supervision adds minimal overhead (1.49µs full lifecycle, Task 6.2)

### Comparison to Other Frameworks

**Erlang/OTP:**

- **Similarity**: "Let it Crash" philosophy, supervision trees
- **Difference**: Erlang supervisors built into language, ComponentActor uses library

**Kubernetes:**

- **Similarity**: Restart policies (Always, OnFailure, Never)
- **Difference**: Kubernetes restarts containers (slow), ComponentActor restarts components (1.49µs)

**Akka (Scala):**

- **Similarity**: Actor supervision, restart strategies
- **Difference**: Akka JVM-based, ComponentActor WASM-based (memory isolation)

**Result:** ComponentActor supervision combines best practices from Erlang, Kubernetes, and Akka, optimized for WASM components.

## Summary

Supervision is critical for production ComponentActor systems:

1. **Automatic Recovery**: Components restart automatically on crash (1.49µs lifecycle)
2. **Fault Isolation**: One component crash doesn't affect others (memory, state, actor isolation)
3. **Configurable Strategies**: Immediate, delayed, or exponential backoff restart
4. **Cascading Prevention**: Restart limits and circuit breakers prevent system-wide failures
5. **Production-Proven**: Design inspired by Erlang/OTP's 20+ years of production experience

**Key Design Decisions:**

- ✅ Isolated restart (not system-wide)
- ✅ Flat supervision (simple, sufficient for most cases)
- ✅ Three restart strategies (immediate, delayed, exponential backoff)
- ✅ Restart limits (prevents infinite loops)
- ✅ Layered architecture (SupervisorNode → ActorSystem → Tokio)

**Performance Impact:** Minimal overhead (1.49µs full lifecycle, 286ns spawn, measured in Task 6.2).

**Future Enhancements:**

- Health checks (proactive failure detection)
- Hierarchical supervision (for >100 components)
- State restoration (automatic state recovery)

## Next Steps

- [Supervision and Recovery Guide](../guides/supervision-and-recovery.md) - Implement supervision
- [Production Readiness](production-readiness.md) - Deployment considerations
- [Component Composition](../guides/component-composition.md) - Build complex systems
