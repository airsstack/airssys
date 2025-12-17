# Understanding Supervision

This document explains the supervision model in AirsSys RT, its design philosophy, how it achieves fault tolerance, and the reasoning behind different restart strategies.

## Table of Contents

- [What is Supervision?](#what-is-supervision)
- [The "Let it Crash" Philosophy](#the-let-it-crash-philosophy)
- [Supervision Tree Architecture](#supervision-tree-architecture)
- [Restart Strategies Explained](#restart-strategies-explained)
- [Design Decisions and Rationale](#design-decisions-and-rationale)
- [Comparison with Alternative Approaches](#comparison-with-alternative-approaches)
- [When to Use Each Strategy](#when-to-use-each-strategy)

---

## What is Supervision?

**Supervision** is a fault tolerance pattern where a "supervisor" actor monitors "child" actors and automatically restarts them when they fail. This creates a hierarchical structure of fault tolerance called a **supervision tree**.

### The Supervision Relationship

```
Supervisor (Parent)
├── Child A (Worker)
├── Child B (Worker)
└── Child C (Supervisor)
    ├── Child C.1 (Worker)
    └── Child C.2 (Worker)
```

**Key Responsibilities:**

**Supervisor:**

- Starts child actors during initialization
- Monitors child actors for failures
- Decides how to handle failures (restart, stop, escalate)
- Cleans up resources when children terminate

**Child:**

- Performs actual work (processing messages, managing state)
- Reports failures to supervisor
- Accepts restart commands from supervisor
- Cleans up resources during shutdown

### Why Supervision Matters

**Without Supervision:**
```rust
// Actor crashes -> error propagates -> entire system crashes
let actor = MyActor::new();
actor.process(msg)?;  // If this fails, what happens?
// System stops. No recovery mechanism.
```

**With Supervision:**
```rust
// Actor crashes -> supervisor detects -> restart actor -> system continues
let supervisor = SupervisorBuilder::new()
    .with_child(ChildSpec::new("worker").with_actor::<MyActor>())
    .build()
    .await?;
// Child crashes? Supervisor automatically restarts it.
// System continues operating with fresh child instance.
```

**The Value Proposition:** Supervision isolates failures and enables automatic recovery, transforming crashes from catastrophic failures into recoverable events.

---

## The "Let it Crash" Philosophy

### Traditional Error Handling

**Defensive Programming Approach:**

```rust
// Traditional: anticipate and handle every possible error
async fn process_request(req: Request) -> Result<Response, Error> {
    // Validate input
    if req.data.is_empty() {
        return Err(Error::InvalidInput);
    }
    
    // Check preconditions
    if !is_ready() {
        return Err(Error::NotReady);
    }
    
    // Retry on transient errors
    let result = match call_external_service(&req).await {
        Ok(r) => r,
        Err(NetworkError::Timeout) => {
            // Retry with backoff
            retry_with_backoff().await?
        }
        Err(NetworkError::ConnectionRefused) => {
            // Use cached data
            get_cached_response()?
        }
        Err(e) => return Err(e),
    };
    
    // Validate output
    if !result.is_valid() {
        return Err(Error::InvalidResponse);
    }
    
    Ok(result)
}
```

**Problems:**

- Complex error handling logic in every function
- Difficult to anticipate all possible failure modes
- Error handling code often larger than business logic
- Recovery strategies may be incorrect or incomplete

### "Let it Crash" Approach

**Supervision-Based Recovery:**

```rust
// Supervision: simple error handling, rely on supervisor for recovery
async fn handle(&mut self, msg: ProcessRequest, ctx: &mut ActorContext<Self>) -> Result<Response, ActorError> {
    let result = call_external_service(&msg).await?;  // If error, actor crashes
    
    // Supervisor detects crash and restarts actor
    // Fresh actor instance begins with clean state
    
    Ok(result)
}

// Supervisor configuration
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForOne)
    .with_child(
        ChildSpec::new("processor")
            .with_actor::<RequestProcessor>()
            .with_max_restarts(5)  // Limit restart rate
            .with_restart_window(Duration::from_secs(60))
    )
    .build()
    .await?;
```

**Benefits:**

- Simpler code focused on happy path
- Supervisor handles recovery consistently
- Failed actor state is discarded (no corrupted state)
- Restart limits prevent infinite restart loops

### When to "Let it Crash"

**✅ Good Candidates:**

- **Transient failures:** Network timeouts, temporary resource unavailability
- **Corrupted state:** State machine in unexpected state, data corruption detected
- **Resource exhaustion:** Out of memory, file descriptors exhausted
- **Unexpected conditions:** Assertion failures, invariant violations

**❌ Poor Candidates:**

- **Expected errors:** User input validation, business logic errors
- **Recoverable errors:** Authentication failures (should return error, not crash)
- **Deterministic failures:** Configuration errors (restart won't fix)
- **Critical operations:** Financial transactions (must complete or rollback, not crash)

**Guideline:** Let actors crash for unexpected errors that indicate corrupted state. Handle expected errors explicitly.

---

## Supervision Tree Architecture

### Hierarchical Fault Tolerance

Supervision trees create **fault isolation boundaries** where failures are contained and handled locally rather than propagating system-wide.

```
                Application Supervisor (Root)
                       |
        ┌──────────────┼──────────────┐
        │              │              │
   API Supervisor   DB Supervisor  Worker Pool Supervisor
        │              │              │
   ┌────┴───┐     ┌───┴───┐     ┌────┴────┐
   │        │     │       │     │    ...  │
HTTP    WebSocket │      │    Worker1  WorkerN
Handler  Handler  Reader Writer
```

**Fault Isolation Zones:**

**Level 1 (Application Supervisor):**

- Supervises major subsystems (API, Database, Workers)
- Strategy: OneForOne (independent subsystems)
- Failure impact: Only failed subsystem restarts

**Level 2 (Subsystem Supervisors):**

- Supervises related components (HTTP handlers, DB connections)
- Strategy: May vary (OneForAll for coordinated state)
- Failure impact: Contained within subsystem

**Level 3 (Worker Actors):**

- Performs actual work
- Not supervisors themselves
- Failure impact: Individual worker only

### Supervision Decision Tree

When a child fails, the supervisor follows this decision process:

```
Child Failure Detected
        │
        ▼
Has max_restarts been exceeded? ──YES──> Escalate to Parent Supervisor
        │
       NO
        │
        ▼
What is the restart_policy?
        │
        ├─ Permanent ──> Always Restart
        │
        ├─ Transient ──> Normal Exit? ──YES──> Don't Restart
        │                    │
        │                   NO
        │                    │
        │                    ▼
        │                 Restart
        │
        └─ Temporary ──> Never Restart, Remove Child
```

**Escalation:** If a supervisor cannot recover a child (max restarts exceeded), it escalates to its parent supervisor, which may restart the entire subtree.

---

## Restart Strategies Explained

AirsSys RT provides three restart strategies inspired by Erlang/OTP.

### OneForOne Strategy

**Behavior:** When a child fails, **only that child** is restarted. Other children continue running unaffected.

**Diagram:**
```
Before Failure:          After Failure:
┌─────────────┐          ┌─────────────┐
│ Supervisor  │          │ Supervisor  │
├─────────────┤          ├─────────────┤
│ Child A ✓   │          │ Child A ✓   │ ← Continues running
│ Child B ✗   │  ──────> │ Child B ↻   │ ← Restarted
│ Child C ✓   │          │ Child C ✓   │ ← Continues running
└─────────────┘          └─────────────┘
```

**Use Case:** Children are **independent** - failure of one does not affect others.

**Example:**
```rust
// HTTP request handlers - each independent
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForOne)  // Only restart failed handler
    .with_child(ChildSpec::new("handler-1").with_actor::<HttpHandler>())
    .with_child(ChildSpec::new("handler-2").with_actor::<HttpHandler>())
    .with_child(ChildSpec::new("handler-3").with_actor::<HttpHandler>())
    .build()
    .await?;
// Handler-2 crashes -> Only handler-2 restarts
// Handler-1 and Handler-3 continue processing requests
```

**Performance:** Low overhead - only failed child affected (~1.28µs restart latency).

**When to Use:**

- Workers processing independent tasks (HTTP handlers, job processors)
- Stateless services with no shared state
- High-throughput systems where restarting everything is too expensive

### OneForAll Strategy

**Behavior:** When **any child** fails, **all children** are restarted. Ensures coordinated state across children.

**Diagram:**
```
Before Failure:          After Failure:
┌─────────────┐          ┌─────────────┐
│ Supervisor  │          │ Supervisor  │
├─────────────┤          ├─────────────┤
│ Child A ✓   │          │ Child A ↻   │ ← Restarted (even though healthy)
│ Child B ✗   │  ──────> │ Child B ↻   │ ← Restarted (failure trigger)
│ Child C ✓   │          │ Child C ↻   │ ← Restarted (even though healthy)
└─────────────┘          └─────────────┘
```

**Use Case:** Children have **interdependent state** - all must restart together to maintain consistency.

**Example:**
```rust
// Distributed transaction coordinators - must stay synchronized
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForAll)  // Restart all to maintain consistency
    .with_child(ChildSpec::new("coordinator-1").with_actor::<TxCoordinator>())
    .with_child(ChildSpec::new("coordinator-2").with_actor::<TxCoordinator>())
    .with_child(ChildSpec::new("coordinator-3").with_actor::<TxCoordinator>())
    .build()
    .await?;
// Coordinator-2 crashes -> All coordinators restart
// Ensures consistent state across distributed transaction
```

**Performance:** Higher overhead - all children affected (30-150µs for full restart).

**When to Use:**

- State machines with coordinated state across actors
- Distributed consensus protocols (Raft, Paxos)
- Tightly coupled services that must stay synchronized

### RestForOne Strategy

**Behavior:** When a child fails, restart **that child and all children started after it**. Preserves dependency order.

**Diagram:**
```
Before Failure:          After Failure:
┌─────────────┐          ┌─────────────┐
│ Supervisor  │          │ Supervisor  │
├─────────────┤          ├─────────────┤
│ Child A ✓   │          │ Child A ✓   │ ← Continues (started before B)
│ Child B ✗   │  ──────> │ Child B ↻   │ ← Restarted (failure trigger)
│ Child C ✓   │          │ Child C ↻   │ ← Restarted (depends on B)
└─────────────┘          └─────────────┘
```

**Use Case:** Children have **dependency chains** - later children depend on earlier ones.

**Example:**
```rust
// Pipeline: Reader -> Processor -> Writer
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::RestForOne)  // Maintain dependency order
    .with_child(ChildSpec::new("reader").with_actor::<DataReader>())     // Independent
    .with_child(ChildSpec::new("processor").with_actor::<DataProcessor>()) // Depends on reader
    .with_child(ChildSpec::new("writer").with_actor::<DataWriter>())     // Depends on processor
    .build()
    .await?;
// Processor crashes -> Processor + Writer restart (Reader continues)
// Writer crashes -> Only Writer restarts
// Reader crashes -> All restart (everything depends on reader)
```

**Performance:** Medium overhead - only dependent children affected.

**When to Use:**

- Processing pipelines with dependency order
- Services with startup dependencies (database connection before query executor)
- Systems where later stages depend on earlier stages

---

## Design Decisions and Rationale

### Decision: Three Restart Strategies

**Context:** Erlang provides OneForOne, OneForAll, RestForOne, and SimpleOneForOne.

**Choice:** Implement OneForOne, OneForAll, RestForOne (skip SimpleOneForOne).

**Rationale:**

- **OneForOne:** Most common, covers independent workers (80% of use cases)
- **OneForAll:** Essential for coordinated state (15% of use cases)
- **RestForOne:** Handles dependency chains (5% of use cases)
- **SimpleOneForOne:** Dynamic worker pools - can be built with actor pools instead

**Tradeoff:** Simpler API (fewer strategies to choose from) vs. Completeness (missing SimpleOneForOne).

### Decision: Restart Rate Limiting

**Context:** Unlimited restarts can cause "restart storms" consuming resources.

**Choice:** Require `max_restarts` and `restart_window` configuration.

```rust
ChildSpec::new("worker")
    .with_max_restarts(5)  // Max 5 restarts
    .with_restart_window(Duration::from_secs(60))  // Within 60 seconds
```

**Rationale:**

- Prevents infinite restart loops (deterministic failures)
- Escalates persistent failures to parent supervisor
- Protects system resources from thrashing

**Behavior:** If child crashes more than `max_restarts` times within `restart_window`, supervisor escalates to its parent.

**Tradeoff:** Configuration complexity vs. System stability.

### Decision: Graceful Shutdown

**Context:** Child actors may need time to clean up resources (flush buffers, close connections).

**Choice:** Provide `ShutdownPolicy` with timeout options.

```rust
pub enum ShutdownPolicy {
    Graceful(Duration),  // Wait for graceful shutdown, timeout after duration
    Immediate,           // Stop immediately, no cleanup
    Infinity,            // Wait indefinitely for shutdown
}
```

**Rationale:**

- **Graceful:** Most services need cleanup time (default)
- **Immediate:** For non-critical workers (faster restarts)
- **Infinity:** For critical transactions (must complete)

**Tradeoff:** Shutdown latency vs. Data consistency.

### Decision: Supervisor Builder Pattern

**Context:** Creating supervisors requires many configuration options.

**Choice:** Implement builder pattern for ergonomic configuration.

```rust
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForOne)
    .with_max_restarts(10)
    .with_restart_window(Duration::from_secs(60))
    .with_child(
        ChildSpec::new("worker-1")
            .with_actor::<Worker>()
            .with_restart_policy(RestartPolicy::Permanent)
    )
    .with_child(
        ChildSpec::new("worker-2")
            .with_actor::<Worker>()
            .with_restart_policy(RestartPolicy::Transient)
    )
    .build()
    .await?;
```

**Rationale:**

- Clear, readable configuration
- Compile-time validation of required fields
- Follows Rust builder pattern conventions (RT-TASK-013)

**Tradeoff:** More code to maintain vs. Better developer experience.

---

## Comparison with Alternative Approaches

### Supervision vs. Try-Catch Error Handling

**Try-Catch Approach:**

```rust
// Traditional exception handling
loop {
    match worker.process().await {
        Ok(_) => continue,
        Err(e) => {
            log::error!("Worker failed: {:?}", e);
            // Manual recovery logic
            worker = Worker::new()?;  // Recreate worker
            // What about worker state? Cleanup? Resource leaks?
        }
    }
}
```

**Problems:**

- Recovery logic scattered throughout code
- Inconsistent recovery strategies
- Difficult to test recovery paths
- No isolation (failure may corrupt surrounding code)

**Supervision Approach:**

```rust
// Supervisor handles recovery consistently
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForOne)
    .with_child(ChildSpec::new("worker").with_actor::<Worker>())
    .build()
    .await?;
// Supervisor monitors worker, restarts on failure
// Recovery strategy centralized and tested
// Failure isolated to child actor boundary
```

**Advantages:**

- Centralized recovery strategy
- Consistent across all supervised actors
- Testable recovery logic
- Clear fault isolation boundaries

### Supervision vs. Circuit Breaker Pattern

**Circuit Breaker:**

```rust
// Circuit breaker prevents cascading failures
let circuit_breaker = CircuitBreaker::new()
    .failure_threshold(5)
    .timeout(Duration::from_secs(60));

match circuit_breaker.call(|| external_service.call()).await {
    Ok(result) => handle_success(result),
    Err(CircuitBreakerError::Open) => handle_circuit_open(),
    Err(CircuitBreakerError::ServiceError(e)) => handle_service_error(e),
}
```

**Use Case:** Protects caller from failing service, prevents resource exhaustion.

**Supervision:**

```rust
// Supervision restarts failed actors
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForOne)
    .with_child(ChildSpec::new("service").with_actor::<ExternalService>())
    .build()
    .await?;
```

**Use Case:** Recovers failed actor with fresh state, maintains service availability.

**Comparison:**

| Aspect | Circuit Breaker | Supervision |
|--------|----------------|-------------|
| **Purpose** | Prevent cascading failures | Recover from failures |
| **Behavior** | Stop calling failing service | Restart failed actor |
| **When to Use** | External dependencies | Internal actors |
| **Failure Detection** | Error rate threshold | Crash detection |
| **Recovery** | Manual (wait for circuit close) | Automatic (supervisor restart) |

**Best Practice:** Use **both** - circuit breakers for external services, supervision for internal actors.

---

## When to Use Each Strategy

### OneForOne Strategy

**Choose When:**

- ✅ Workers are **stateless** or have **independent state**
- ✅ High throughput required (minimize restart impact)
- ✅ Failures are **isolated** to individual workers

**Examples:**

- HTTP request handlers (each request independent)
- Job processors (each job independent)
- WebSocket connections (each connection independent)

**Anti-Patterns:**

- ❌ Workers share mutable state (use OneForAll)
- ❌ Workers have dependencies (use RestForOne)

### OneForAll Strategy

**Choose When:**

- ✅ Workers have **coordinated state** (must stay synchronized)
- ✅ Partial restart would cause **inconsistency**
- ✅ Workers are **tightly coupled**

**Examples:**

- Distributed consensus (Raft nodes must synchronize)
- State machine replicas (must maintain identical state)
- Transaction coordinators (must agree on outcome)

**Anti-Patterns:**

- ❌ Workers are independent (unnecessary restart overhead)
- ❌ High-throughput system (OneForAll restarts too disruptive)

### RestForOne Strategy

**Choose When:**

- ✅ Workers have **dependency chains** (A depends on B depends on C)
- ✅ Later workers **depend on** earlier workers
- ✅ Restart order matters

**Examples:**

- Processing pipelines (Reader -> Transformer -> Writer)
- Service initialization (Config -> Database -> API Server)
- Layered architecture (Data Layer -> Business Layer -> Presentation Layer)

**Anti-Patterns:**

- ❌ No clear dependency order (use OneForOne)
- ❌ Circular dependencies (redesign architecture)

---

## Supervision Best Practices

### 1. Design Supervision Trees Top-Down

**Start with fault isolation boundaries:**

```rust
// Level 1: Major subsystems
Application Supervisor
├── API Supervisor
├── Database Supervisor
└── Background Jobs Supervisor
```

**Then drill down into components:**

```rust
// Level 2: Components within subsystem
API Supervisor
├── HTTP Handler Pool
├── WebSocket Handler Pool
└── Authentication Service
```

**Guideline:** Each supervisor manages 3-10 children for clarity.

### 2. Match Strategy to Failure Characteristics

**Ask yourself:**

1. **Are children independent?** → Use OneForOne
2. **Must children stay synchronized?** → Use OneForAll
3. **Do children have dependencies?** → Use RestForOne

### 3. Set Realistic Restart Limits

**Conservative (critical services):**
```rust
.with_max_restarts(3)
.with_restart_window(Duration::from_secs(300))  // 3 restarts per 5 minutes
```

**Moderate (typical services):**
```rust
.with_max_restarts(5)
.with_restart_window(Duration::from_secs(60))  // 5 restarts per minute
```

**Aggressive (transient failures expected):**
```rust
.with_max_restarts(10)
.with_restart_window(Duration::from_secs(60))  // 10 restarts per minute
```

**Guideline:** Start conservative, increase limits if transient failures are common.

### 4. Monitor Supervisor Health

```rust
use airssys_rt::monitoring::{HealthMonitor, SupervisorHealthCheck};

let monitor = HealthMonitor::new();
monitor.register(supervisor_ref, SupervisorHealthCheck::new()).await?;

// Check supervisor health periodically
let status = monitor.check_health(supervisor_ref).await?;
if status.is_unhealthy() {
    log::error!("Supervisor unhealthy: {:?}", status);
    // Alert operations team
}
```

---

## Further Reading

### AirsSys RT Documentation

- [Supervisor API Reference](../reference/api/supervisors.md)
- [Supervisor Patterns Guide](../guides/supervisor-patterns.md)
- [Architecture: Supervision](../reference/architecture/supervision.md)

### External Resources

- **Erlang/OTP Design Principles:** Official Erlang supervision documentation
- **"Let it Crash" Philosophy:** Joe Armstrong's writings on fault tolerance
- **Release It!** (Michael Nygard): Stability patterns including supervision

---

**Last Updated:** 2025-01-18 (RT-TASK-011 Phase 4 Day 7)
