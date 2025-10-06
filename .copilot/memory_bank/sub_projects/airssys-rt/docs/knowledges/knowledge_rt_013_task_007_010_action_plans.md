# KNOWLEDGE-RT-013: RT-TASK-007 and RT-TASK-010 Implementation Action Plans

**Sub-Project:** airssys-rt  
**Category:** Implementation Guides  
**Created:** 2025-10-06  
**Last Updated:** 2025-10-06  
**Status:** active  

## Context and Problem

RT-TASK-007 (Supervisor Framework) and RT-TASK-010 (Monitoring Module) are foundational components for fault tolerance and observability in airssys-rt. The implementation requires careful sequencing: RT-TASK-010 must be completed first to provide monitoring infrastructure for RT-TASK-007. This document provides comprehensive action plans for both tasks with detailed phase breakdowns, acceptance criteria, and integration strategies.

## Task Sequencing Strategy

```
RT-TASK-010 (Monitoring Module)  →  RT-TASK-007 (Supervisor Framework)
     2-3 days                              8-10 days
     
     ↓                                      ↓
   BLOCKS                                BLOCKS
     ↓                                      ↓
RT-TASK-007, RT-TASK-008            RT-TASK-008, RT-TASK-009
```

**Rationale:**
- Monitoring is foundational infrastructure needed by multiple components
- Building monitoring in RT-TASK-010 reduces RT-TASK-007 complexity
- Generic Monitor<E> trait enables reuse across supervisor, performance, system monitoring
- Clean task separation improves maintainability

---

## RT-TASK-010: Universal Monitoring Infrastructure

**Priority:** CRITICAL - Foundational infrastructure  
**Status:** Not Started  
**Estimated Duration:** 2-3 days (16-20 hours)  
**Dependencies:** None (standalone)  
**Blocks:** RT-TASK-007, RT-TASK-008  

### Architecture Overview

**Core Design Principles:**
1. **Generic Monitor<E> Trait**: Universal monitoring for any entity type
2. **Zero-Cost Abstraction**: NoopMonitor compiles away when disabled
3. **Lock-Free Recording**: Atomic counters for concurrent event tracking
4. **Bounded Memory**: Ring buffer prevents unbounded growth
5. **Type Safety**: MonitoringEvent trait ensures compile-time correctness

**Key Components:**
```rust
// Generic monitoring trait
pub trait Monitor<E: MonitoringEvent>: Send + Sync + Clone {
    async fn record(&self, event: E) -> Result<(), Self::Error>;
    async fn snapshot(&self) -> Result<MonitoringSnapshot<E>, Self::Error>;
    async fn reset(&self) -> Result<(), Self::Error>;
}

// Event type abstraction
pub trait MonitoringEvent: Send + Sync + Clone + Debug + Serialize + 'static {
    const EVENT_TYPE: &'static str;
    fn timestamp(&self) -> DateTime<Utc>;
    fn severity(&self) -> EventSeverity;
}

// Concrete implementations
pub struct InMemoryMonitor<E> { inner: Arc<InMemoryMonitorInner<E>> }
pub struct NoopMonitor<E> { _phantom: PhantomData<E> }
```

### Phase 1: Core Traits & Types (Day 1 - 6-8 hours)

**Files to Create:**
- `src/monitoring/mod.rs` - Module declarations (~50 lines)
- `src/monitoring/traits.rs` - Core Monitor<E> trait (~200-250 lines)
- `src/monitoring/types.rs` - Event types and configuration (~300-350 lines)

**Implementation Checklist:**

#### `src/monitoring/traits.rs`
- [ ] Monitor<E> trait with async methods (record, snapshot, reset)
- [ ] MonitoringEvent trait with const EVENT_TYPE
- [ ] EventSeverity enum (Trace, Debug, Info, Warning, Error, Critical)
- [ ] Comprehensive trait documentation with examples
- [ ] Import organization following §2.1 (std → third-party → internal)

#### `src/monitoring/types.rs`
- [ ] SupervisionEvent with SupervisionEventKind enum
  - ChildStarted, ChildStopped, ChildFailed, ChildRestarted
  - RestartLimitExceeded, StrategyApplied
- [ ] ActorEvent with ActorEventKind enum
  - Spawned, Started, MessageReceived, MessageProcessed
  - ErrorOccurred, Stopped
- [ ] SystemEvent with SystemEventKind enum
- [ ] BrokerEvent with BrokerEventKind enum
- [ ] MailboxEvent with MailboxEventKind enum
- [ ] MonitoringConfig struct (enabled, max_history_size, severity_filter)
- [ ] MonitoringSnapshot<E> struct (timestamp, counters, recent_events)
- [ ] All types implement Clone, Debug, Serialize
- [ ] All timestamps use chrono DateTime<Utc> (§3.2)

**Testing Strategy:**
```rust
#[cfg(test)]
mod tests {
    // Test event creation and serialization
    #[test]
    fn test_supervision_event_creation() { }
    
    // Test MonitoringEvent trait implementations
    #[test]
    fn test_monitoring_event_trait() { }
    
    // Test severity levels
    #[test]
    fn test_event_severity() { }
    
    // Test configuration validation
    #[test]
    fn test_monitoring_config() { }
    
    // 15+ tests total for phase 1
}
```

**Acceptance Criteria:**
- ✅ Monitor<E> trait with comprehensive rustdoc
- ✅ MonitoringEvent implemented for 5+ event types
- ✅ All types follow workspace standards (§2.1-§6.3)
- ✅ 15+ unit tests passing
- ✅ Zero compiler/clippy warnings

---

### Phase 2: Monitor Implementations (Day 2 - 6-8 hours)

**Files to Create:**
- `src/monitoring/in_memory.rs` - InMemoryMonitor<E> (~400-450 lines)
- `src/monitoring/noop.rs` - NoopMonitor<E> (~100-150 lines)

**Implementation Checklist:**

#### `src/monitoring/in_memory.rs`
**Architecture Pattern: M-SERVICES-CLONE**
```rust
pub struct InMemoryMonitor<E: MonitoringEvent> {
    inner: Arc<InMemoryMonitorInner<E>>,
}

struct InMemoryMonitorInner<E: MonitoringEvent> {
    config: MonitoringConfig,
    
    // Lock-free atomic counters
    total_events: AtomicU64,
    trace_count: AtomicU64,
    debug_count: AtomicU64,
    info_count: AtomicU64,
    warning_count: AtomicU64,
    error_count: AtomicU64,
    critical_count: AtomicU64,
    
    // Ring buffer with RwLock (read-heavy optimization)
    history: RwLock<VecDeque<E>>,
}
```

**Implementation Tasks:**
- [ ] InMemoryMonitorInner struct with atomic counters
- [ ] InMemoryMonitor::new(config) constructor
- [ ] Clone implementation using Arc::clone (M-SERVICES-CLONE)
- [ ] Monitor::record() with severity filtering
  - Atomic counter increment (lock-free)
  - Ring buffer insert with overflow handling
  - Return early if below severity threshold
- [ ] Monitor::snapshot() with current state
  - Collect atomic counter values
  - Read history with RwLock
  - Build MonitoringSnapshot<E>
- [ ] Monitor::reset() clearing all state
  - Reset all atomic counters to 0
  - Clear ring buffer history
- [ ] Comprehensive error handling

#### `src/monitoring/noop.rs`
**Architecture Pattern: Zero-Cost Abstraction**
```rust
#[derive(Debug, Clone)]
pub struct NoopMonitor<E: MonitoringEvent> {
    _phantom: PhantomData<E>,
}

impl<E: MonitoringEvent> NoopMonitor<E> {
    #[inline(always)]
    pub fn new() -> Self {
        Self { _phantom: PhantomData }
    }
}

#[async_trait]
impl<E: MonitoringEvent> Monitor<E> for NoopMonitor<E> {
    type Error = MonitoringError;

    #[inline(always)]
    async fn record(&self, _event: E) -> Result<(), Self::Error> {
        Ok(()) // Compiles away
    }

    // ... other methods also inline(always)
}
```

**Implementation Tasks:**
- [ ] NoopMonitor struct with PhantomData<E>
- [ ] All methods marked #[inline(always)]
- [ ] Default implementation for easy use
- [ ] Verify zero overhead in release builds

**Testing Strategy:**
```rust
#[cfg(test)]
mod tests {
    // InMemoryMonitor tests
    #[tokio::test]
    async fn test_concurrent_event_recording() {
        // Multi-threaded concurrent recording
    }
    
    #[tokio::test]
    async fn test_ring_buffer_overflow() {
        // Test max_history_size enforcement
    }
    
    #[tokio::test]
    async fn test_severity_filtering() {
        // Test events below threshold are filtered
    }
    
    #[tokio::test]
    async fn test_snapshot_accuracy() {
        // Verify snapshot reflects actual state
    }
    
    #[tokio::test]
    async fn test_reset_functionality() {
        // Verify reset clears all state
    }
    
    #[test]
    fn test_clone_implementation() {
        // Verify Arc-based cheap clone
    }
    
    // NoopMonitor tests
    #[tokio::test]
    async fn test_noop_monitor() {
        // Verify no-op behavior
    }
    
    // 20+ tests total for phase 2
}
```

**Performance Verification:**
```bash
# Verify NoopMonitor zero overhead
cargo bench --bench monitoring_overhead
# Expect: NoopMonitor ≈ 0ns, InMemoryMonitor < 100ns
```

**Acceptance Criteria:**
- ✅ InMemoryMonitor with lock-free atomic counters
- ✅ Ring buffer with overflow handling
- ✅ NoopMonitor compiles to zero overhead
- ✅ 20+ unit tests passing
- ✅ Concurrent recording tests pass
- ✅ Zero warnings

---

### Phase 3: Integration & Examples (Day 3 - 4-6 hours)

**Files to Create:**
- `examples/monitoring_basic.rs` - Basic monitoring example (~200 lines)
- `examples/monitoring_supervisor.rs` - Supervisor integration preview (~250 lines)
- `tests/monitoring_tests.rs` - Integration tests (~300 lines)
- Update `src/lib.rs` - Add monitoring module exports

**Implementation Checklist:**

#### Module Exports in `src/lib.rs`
```rust
// Add to src/lib.rs
pub mod monitoring {
    pub use crate::monitoring::traits::{Monitor, MonitoringEvent, EventSeverity};
    pub use crate::monitoring::types::{
        SupervisionEvent, ActorEvent, SystemEvent, BrokerEvent, MailboxEvent,
        MonitoringConfig, MonitoringSnapshot,
    };
    pub use crate::monitoring::in_memory::InMemoryMonitor;
    pub use crate::monitoring::noop::NoopMonitor;
}
```

#### `examples/monitoring_basic.rs`
```rust
/// Demonstrates basic monitoring with InMemoryMonitor
/// 
/// Shows:
/// - Creating InMemoryMonitor with configuration
/// - Recording various event types
/// - Taking snapshots
/// - Severity filtering
/// - Resetting monitor state

use airssys_rt::monitoring::{
    InMemoryMonitor, ActorEvent, ActorEventKind,
    MonitoringConfig, EventSeverity,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup monitoring with configuration
    let config = MonitoringConfig {
        enabled: true,
        max_history_size: 100,
        severity_filter: EventSeverity::Info,
        snapshot_interval: Duration::from_secs(5),
    };
    
    let monitor = InMemoryMonitor::new(config);
    
    // Record actor events
    monitor.record(ActorEvent {
        timestamp: Utc::now(),
        actor_id: "actor-1".to_string(),
        event_kind: ActorEventKind::Spawned,
    }).await?;
    
    // Take snapshot
    let snapshot = monitor.snapshot().await?;
    println!("Total events: {}", snapshot.total_events);
    
    Ok(())
}
```

#### `examples/monitoring_supervisor.rs`
```rust
/// Demonstrates monitoring integration with supervisor (conceptual)
/// 
/// Shows:
/// - How RT-TASK-007 will use Monitor<SupervisionEvent>
/// - Recording supervision events
/// - Monitoring restart decisions
/// 
/// Note: Full functionality requires RT-TASK-007 completion

use airssys_rt::monitoring::{
    InMemoryMonitor, SupervisionEvent, SupervisionEventKind,
    MonitoringConfig, EventSeverity,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let monitor = InMemoryMonitor::new(MonitoringConfig::default());
    
    // Conceptual: How supervisor will use monitoring
    monitor.record(SupervisionEvent {
        timestamp: Utc::now(),
        supervisor_id: "supervisor-1".to_string(),
        child_id: Some("child-1".to_string()),
        event_kind: SupervisionEventKind::ChildStarted,
        metadata: HashMap::new(),
    }).await?;
    
    // Monitoring restart decisions
    monitor.record(SupervisionEvent {
        timestamp: Utc::now(),
        supervisor_id: "supervisor-1".to_string(),
        child_id: Some("child-1".to_string()),
        event_kind: SupervisionEventKind::ChildFailed {
            error: "Connection lost".to_string(),
            restart_count: 1,
        },
        metadata: HashMap::new(),
    }).await?;
    
    Ok(())
}
```

#### Integration Tests
```rust
// tests/monitoring_tests.rs
#[cfg(test)]
mod monitoring_integration_tests {
    #[tokio::test]
    async fn test_monitoring_with_actors() {
        // Test monitoring integration with actor system
    }
    
    #[tokio::test]
    async fn test_monitoring_with_broker() {
        // Test monitoring integration with message broker
    }
    
    #[tokio::test]
    async fn test_monitoring_configuration_changes() {
        // Test dynamic configuration updates
    }
    
    #[tokio::test]
    async fn test_monitoring_under_load() {
        // Stress test with high event volume
    }
    
    // 10+ integration tests
}
```

**Documentation Requirements:**
- [ ] Comprehensive rustdoc for all public APIs
- [ ] Module-level documentation with overview
- [ ] Event type documentation with use cases
- [ ] Configuration guide in rustdoc
- [ ] Examples documented with /// comments
- [ ] Integration guide for RT-TASK-007

**Acceptance Criteria:**
- ✅ Module exports in src/lib.rs
- ✅ 2+ working examples compile and run
- ✅ 10+ integration tests passing
- ✅ Comprehensive documentation
- ✅ Total test count: 45+ tests
- ✅ Zero warnings

---

### RT-TASK-010 Summary

**Total Deliverables:**
- 7 new source files (~1,300-1,600 lines total)
- 2 examples (~450 lines)
- 45+ tests (unit + integration)
- Comprehensive rustdoc documentation
- Ready for RT-TASK-007 integration

**Key Achievements:**
- Generic Monitor<E> trait for universal monitoring
- Zero-overhead NoopMonitor when disabled
- Lock-free InMemoryMonitor for high-performance
- 5+ concrete event types covering all runtime entities
- Full workspace standards compliance (§2.1-§6.3)
- Microsoft Rust Guidelines compliance

**Integration Points for RT-TASK-007:**
```rust
// Supervisor will use monitoring like this:
pub struct SupervisorNode<S, C, M>
where
    S: SupervisionStrategy,
    C: Child,
    M: Monitor<SupervisionEvent>,
{
    strategy: S,
    children: HashMap<ChildId, ChildHandle<C>>,
    monitor: M,  // ← Uses Monitor<SupervisionEvent>
}
```

---

## RT-TASK-007: Supervisor Framework

**Priority:** HIGH - Core fault tolerance component  
**Status:** Not Started  
**Estimated Duration:** 8-10 days (64-80 hours)  
**Dependencies:** RT-TASK-010 (Monitoring Module) - REQUIRED  
**Blocks:** RT-TASK-008, RT-TASK-009  

### Architecture Overview

**Core Design Principles:**
1. **BEAM-Inspired**: Supervision strategies from Erlang/OTP
2. **Type-Safe Strategies**: OneForOne, OneForAll, RestForOne as compile-time types
3. **Generic Constraints**: SupervisorNode<S, C, M> with strategy, child, monitor
4. **Restart Policies**: Permanent, Transient, Temporary
5. **Health Monitoring**: Integration with RT-TASK-010 Monitor<SupervisionEvent>

**Key Components:**
```rust
// Core supervisor trait
pub trait Supervisor: Send + Sync + 'static {
    type Child: Child;
    type Strategy: SupervisionStrategy;
    type Monitor: Monitor<SupervisionEvent>;
    
    async fn start_child(&mut self, spec: ChildSpec<Self::Child>) -> Result<ChildId, SupervisorError>;
    async fn stop_child(&mut self, id: &ChildId) -> Result<(), SupervisorError>;
    async fn restart_child(&mut self, id: &ChildId) -> Result<(), SupervisorError>;
    async fn handle_child_error(&mut self, id: &ChildId, error: Box<dyn Error + Send>) -> SupervisionDecision;
}

// Supervision strategies (zero-cost)
pub struct OneForOne;
pub struct OneForAll;
pub struct RestForOne;

// Generic supervisor node
pub struct SupervisorNode<S, C, M>
where
    S: SupervisionStrategy,
    C: Child,
    M: Monitor<SupervisionEvent>,
{
    strategy: S,
    children: HashMap<ChildId, ChildHandle<C>>,
    monitor: M,
}
```

### Phase 1: Supervisor Traits & Core Types (Days 1-2 - 12-16 hours)

**Files to Create:**
- `src/supervisor/mod.rs` - Module declarations (~100 lines)
- `src/supervisor/traits.rs` - Core traits (~400-500 lines)
- `src/supervisor/types.rs` - Types and specifications (~300-400 lines)
- `src/supervisor/error.rs` - Error types (~200-250 lines)

**Implementation Checklist:**

#### `src/supervisor/traits.rs`
- [ ] Supervisor trait with generic Child, Strategy, Monitor
- [ ] Child trait for supervised actors
  - start(), stop(), health_check()
- [ ] SupervisionStrategy trait
  - handle_child_failure() method
- [ ] Strategy marker types (OneForOne, OneForAll, RestForOne)
- [ ] ChildId newtype (UUID-based)
- [ ] Comprehensive trait documentation
- [ ] Import organization (§2.1)

#### `src/supervisor/types.rs`
- [ ] ChildSpec<C: Child> struct
  - id, child_factory, restart_policy, shutdown_policy
  - start_timeout, shutdown_timeout
  - max_restarts, restart_window
- [ ] RestartPolicy enum (Permanent, Transient, Temporary)
- [ ] ShutdownPolicy enum (Graceful, Immediate, Infinity)
- [ ] ChildState enum (Starting, Running, Stopping, Stopped, Restarting, Failed)
- [ ] SupervisionDecision enum
  - RestartChild, RestartAll, RestartSubset, StopChild, StopAll, Escalate
- [ ] ChildHandle<C> struct
  - id, name, child, state, restart_count, last_restart
- [ ] All timestamps use chrono DateTime<Utc> (§3.2)

#### `src/supervisor/error.rs`
- [ ] SupervisorError enum with variants:
  - ChildNotFound, ChildStartFailed, ChildStopFailed
  - RestartLimitExceeded, InvalidConfiguration
  - MonitoringError
- [ ] Implement thiserror::Error
- [ ] Helper methods (is_fatal(), is_retryable())
- [ ] Structured error with Backtrace (M-ERRORS-CANONICAL-STRUCTS)

**Knowledge Base References:**
- **KNOWLEDGE-RT-003**: Supervisor Tree Implementation Strategies (review before implementation)
- **ADR-RT-002**: Message Passing Architecture (for supervisor-child communication)

**Testing Strategy:**
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_child_id_uniqueness() { }
    
    #[test]
    fn test_restart_policy_behavior() { }
    
    #[test]
    fn test_shutdown_policy_timeout() { }
    
    #[test]
    fn test_child_spec_validation() { }
    
    #[test]
    fn test_supervision_decision_patterns() { }
    
    // 20+ tests total
}
```

**Acceptance Criteria:**
- ✅ All traits with comprehensive documentation
- ✅ All types follow workspace standards
- ✅ 20+ unit tests passing
- ✅ Zero warnings

---

### Phase 2: Restart Strategies (Days 3-4 - 12-16 hours)

**Files to Create:**
- `src/supervisor/strategy.rs` - Strategy implementations (~500-600 lines)
- `src/supervisor/backoff.rs` - Restart backoff logic (~300-400 lines)

**Implementation Checklist:**

#### `src/supervisor/strategy.rs`
**OneForOne Strategy:**
```rust
impl SupervisionStrategy for OneForOne {
    fn handle_child_failure<C: Child>(
        supervisor: &SupervisorNode<Self, C>,
        failed_child: &ChildId,
        error: Box<dyn Error + Send>,
    ) -> SupervisionDecision {
        // Check restart policy
        // Check restart limits
        // Return RestartChild or StopChild
    }
}
```

**Implementation Tasks:**
- [ ] OneForOne strategy implementation
  - Only restart failed child
  - Check restart policy (Permanent/Transient/Temporary)
  - Enforce restart rate limits
- [ ] OneForAll strategy implementation
  - Stop all children
  - Restart all children in order
  - Check any child's restart policy
- [ ] RestForOne strategy implementation
  - Restart failed child and all started after it
  - Maintain child start order
  - Check restart policies for subset
- [ ] Helper functions:
  - should_restart(policy, error) -> bool
  - is_normal_exit(error) -> bool
  - should_restart_any(children, error) -> bool

#### `src/supervisor/backoff.rs`
**Restart Rate Limiting:**
```rust
pub struct RestartBackoff {
    max_restarts: u32,
    restart_window: Duration,
    restart_history: VecDeque<DateTime<Utc>>,
}

impl RestartBackoff {
    pub fn is_limit_exceeded(&mut self) -> bool {
        // Remove old restarts outside window
        // Check if at limit
    }
    
    pub fn record_restart(&mut self) {
        // Add restart to history
    }
    
    pub fn calculate_delay(&self) -> Duration {
        // Exponential backoff: base * 2^restart_count
    }
}
```

**Implementation Tasks:**
- [ ] RestartBackoff struct with restart history
- [ ] is_limit_exceeded() with sliding window
- [ ] record_restart() tracking
- [ ] calculate_delay() with exponential backoff
  - Base delay: 100ms
  - Max delay: 60s
  - Formula: base * 2^(min(restart_count, 10))
- [ ] Window expiration cleanup

**Testing Strategy:**
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_one_for_one_restart() { }
    
    #[test]
    fn test_one_for_all_cascade() { }
    
    #[test]
    fn test_rest_for_one_partial() { }
    
    #[test]
    fn test_restart_rate_limiting() { }
    
    #[test]
    fn test_exponential_backoff() { }
    
    #[test]
    fn test_restart_window_expiration() { }
    
    #[test]
    fn test_permanent_policy() { }
    
    #[test]
    fn test_transient_policy() { }
    
    #[test]
    fn test_temporary_policy() { }
    
    // 25+ tests total
}
```

**Acceptance Criteria:**
- ✅ All 3 strategies implemented
- ✅ Restart rate limiting working
- ✅ Exponential backoff calculation
- ✅ 25+ tests passing
- ✅ Zero warnings

---

### Phase 3: Supervisor Tree & Node Management (Days 5-7 - 18-24 hours)

**Files to Create:**
- `src/supervisor/node.rs` - SupervisorNode implementation (~600-700 lines)
- `src/supervisor/tree.rs` - Supervisor tree hierarchy (~400-500 lines)

**Implementation Checklist:**

#### `src/supervisor/node.rs`
**Core Structure:**
```rust
pub struct SupervisorNode<S, C, M>
where
    S: SupervisionStrategy,
    C: Child,
    M: Monitor<SupervisionEvent>,
{
    id: Uuid,
    strategy: S,
    children: HashMap<ChildId, ChildHandle<C>>,
    child_order: Vec<ChildId>,  // For RestForOne
    backoff: HashMap<ChildId, RestartBackoff>,
    monitor: M,
    state: SupervisorState,
}
```

**Implementation Tasks:**
- [ ] SupervisorNode::new(strategy, monitor) constructor
- [ ] add_child(spec) method
  - Create child using factory
  - Initialize ChildHandle
  - Add to children map and order
  - Initialize backoff tracking
  - Start child
  - Record monitoring event
- [ ] start_child_internal(child_id) method
  - Set state to Starting
  - Call child.start()
  - Set state to Running
  - Record monitoring event
- [ ] stop_child_internal(child_id) method
  - Set state to Stopping
  - Call child.stop() with timeout
  - Set state to Stopped
  - Record monitoring event
- [ ] restart_child_internal(child_id) method
  - Check restart limits
  - Stop old child
  - Create new child from factory
  - Update restart count
  - Record restart in backoff
  - Start new child
  - Record monitoring events
- [ ] get_children_started_after(child_id) method
  - Find position in child_order
  - Return slice from position to end
- [ ] is_restart_limit_exceeded(child_id) method
  - Check backoff tracker
- [ ] Supervisor trait implementation
  - Delegate to internal methods
  - Apply supervision strategy on errors

#### `src/supervisor/tree.rs`
**Hierarchical Structure:**
```rust
pub struct SupervisorTree<S, C, M>
where
    S: SupervisionStrategy,
    C: Child,
    M: Monitor<SupervisionEvent>,
{
    root: SupervisorNode<S, C, M>,
    children: HashMap<Uuid, Box<dyn Supervisor<Child = C, Strategy = S, Monitor = M>>>,
    parent: Option<Uuid>,
}
```

**Implementation Tasks:**
- [ ] SupervisorTree::new(root_strategy, monitor) constructor
- [ ] add_supervisor(parent_id, child_supervisor) method
- [ ] remove_supervisor(supervisor_id) method
- [ ] get_supervisor(supervisor_id) method
- [ ] Error escalation to parent supervisor
- [ ] Tree traversal methods
- [ ] Shutdown propagation (top-down)

**Testing Strategy:**
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_supervisor_node_creation() { }
    
    #[tokio::test]
    async fn test_add_child() { }
    
    #[tokio::test]
    async fn test_start_child() { }
    
    #[tokio::test]
    async fn test_stop_child() { }
    
    #[tokio::test]
    async fn test_restart_child() { }
    
    #[tokio::test]
    async fn test_restart_limit_enforcement() { }
    
    #[tokio::test]
    async fn test_strategy_execution() { }
    
    #[tokio::test]
    async fn test_monitoring_integration() { }
    
    #[tokio::test]
    async fn test_children_started_after() { }
    
    #[tokio::test]
    async fn test_supervisor_tree_hierarchy() { }
    
    #[tokio::test]
    async fn test_error_escalation() { }
    
    // 30+ tests total
}
```

**Acceptance Criteria:**
- ✅ SupervisorNode fully implemented
- ✅ All supervisor operations working
- ✅ Monitoring integration complete
- ✅ Tree hierarchy support
- ✅ 30+ tests passing
- ✅ Zero warnings

---

### Phase 4: Health Monitoring & Restart Logic (Days 8-10 - 18-24 hours)

**Files to Create:**
- `src/supervisor/health.rs` - Health monitoring (~400-500 lines)

**Implementation Checklist:**

#### `src/supervisor/health.rs`
**Health Monitoring Structure:**
```rust
pub struct HealthMonitor<C: Child> {
    check_interval: Duration,
    check_timeout: Duration,
    health_checks: HashMap<ChildId, HealthStatus>,
}

pub struct HealthStatus {
    pub last_check: DateTime<Utc>,
    pub consecutive_failures: u32,
    pub is_healthy: bool,
}
```

**Implementation Tasks:**
- [ ] HealthMonitor::new(check_interval, check_timeout) constructor
- [ ] check_child(child_id, child) method
  - Timeout health check call
  - Update health status
  - Track consecutive failures
  - Return health result
- [ ] should_restart(child_id, threshold) method
  - Check consecutive failures against threshold
- [ ] get_health_status(child_id) method
- [ ] reset_health(child_id) method
- [ ] Background health check task
  - Periodic health checking
  - Automatic restart trigger on threshold
- [ ] Integration with SupervisorNode
  - Optional health monitoring
  - Configurable check interval
  - Configurable failure threshold

**Testing Strategy:**
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_health_check_success() { }
    
    #[tokio::test]
    async fn test_health_check_timeout() { }
    
    #[tokio::test]
    async fn test_consecutive_failure_tracking() { }
    
    #[tokio::test]
    async fn test_restart_threshold() { }
    
    #[tokio::test]
    async fn test_health_status_reset() { }
    
    #[tokio::test]
    async fn test_background_health_checking() { }
    
    // 20+ tests total
}
```

**Acceptance Criteria:**
- ✅ Health monitoring fully implemented
- ✅ Timeout handling working
- ✅ Restart threshold logic
- ✅ Background health checks
- ✅ 20+ tests passing
- ✅ Zero warnings

---

### Phase 5: Integration & Examples (Days 11-12 - REDUCED SCOPE)

**Status:** Simplified due to separate RT-TASK-010 monitoring

**Files to Create:**
- `examples/supervisor_basic.rs` - Basic supervisor example (~300 lines)
- `examples/supervisor_strategies.rs` - Strategy comparison (~400 lines)
- `tests/supervisor_tests.rs` - Integration tests (~500 lines)

**Implementation Checklist:**

#### `examples/supervisor_basic.rs`
```rust
/// Demonstrates basic supervisor usage
/// 
/// Shows:
/// - Creating a supervisor with OneForOne strategy
/// - Adding children with different restart policies
/// - Handling child failures and automatic restarts
/// - Monitoring supervision events

use airssys_rt::supervisor::{SupervisorNode, OneForOne, ChildSpec, RestartPolicy};
use airssys_rt::monitoring::InMemoryMonitor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create monitoring
    let monitor = InMemoryMonitor::new(MonitoringConfig::default());
    
    // Create supervisor with OneForOne strategy
    let mut supervisor = SupervisorNode::new(OneForOne, monitor.clone());
    
    // Add children
    let child_spec = ChildSpec {
        id: "worker-1".to_string(),
        child_factory: Box::new(|| Ok(WorkerActor::new())),
        restart_policy: RestartPolicy::Permanent,
        shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
        start_timeout: Duration::from_secs(10),
        shutdown_timeout: Duration::from_secs(10),
        max_restarts: 5,
        restart_window: Duration::from_secs(60),
    };
    
    let child_id = supervisor.start_child(child_spec).await?;
    
    // Check monitoring snapshot
    let snapshot = monitor.snapshot().await?;
    println!("Supervision events: {}", snapshot.total_events);
    
    Ok(())
}
```

#### `examples/supervisor_strategies.rs`
- Demonstrates all 3 strategies
- Shows restart behavior differences
- Compares fault isolation patterns

#### Integration Tests
```rust
#[cfg(test)]
mod supervisor_integration_tests {
    #[tokio::test]
    async fn test_one_for_one_complete_scenario() { }
    
    #[tokio::test]
    async fn test_one_for_all_complete_scenario() { }
    
    #[tokio::test]
    async fn test_rest_for_one_complete_scenario() { }
    
    #[tokio::test]
    async fn test_error_escalation_to_parent() { }
    
    #[tokio::test]
    async fn test_graceful_shutdown_propagation() { }
    
    // 15+ integration tests
}
```

**Acceptance Criteria:**
- ✅ 2+ working examples
- ✅ 15+ integration tests
- ✅ Comprehensive documentation
- ✅ Zero warnings

---

### RT-TASK-007 Summary

**Total Deliverables:**
- 10 new source files (~3,500-4,000 lines total)
- 2 examples (~700 lines)
- 110+ tests (unit + integration)
- Comprehensive rustdoc documentation
- Full BEAM-inspired fault tolerance

**Key Achievements:**
- Generic SupervisorNode<S, C, M> with type safety
- All 3 BEAM strategies (OneForOne, OneForAll, RestForOne)
- Restart rate limiting with exponential backoff
- Health monitoring with configurable checks
- Supervisor tree hierarchy with error escalation
- Full RT-TASK-010 monitoring integration
- Workspace standards compliance (§2.1-§6.3)

---

## Combined Implementation Summary

### Task Dependencies
```
RT-TASK-010 (2-3 days) → RT-TASK-007 (8-10 days)
```

### Total Effort
- **RT-TASK-010**: 16-20 hours
- **RT-TASK-007**: 64-80 hours  
- **Combined**: 80-100 hours (10-13 days sequential)

### Key Architectural Decisions

**1. Monitoring First Strategy**
- RT-TASK-010 provides foundational infrastructure
- Reduces RT-TASK-007 complexity (just use Monitor<E>)
- Enables reuse for RT-TASK-008 performance features

**2. Generic Abstractions**
- Monitor<E> for universal monitoring
- SupervisorNode<S, C, M> for type-safe supervision
- Compile-time strategy selection (zero-cost)

**3. BEAM-Inspired Patterns**
- Proven fault tolerance from Erlang/OTP
- Supervision strategies: OneForOne, OneForAll, RestForOne
- Restart policies: Permanent, Transient, Temporary

**4. Microsoft Rust Guidelines Compliance**
- M-SERVICES-CLONE: Arc<Inner> pattern for cheap cloning
- M-DI-HIERARCHY: Concrete > Generics > dyn
- M-ERRORS-CANONICAL-STRUCTS: Structured errors with Backtrace
- M-MOCKABLE-SYSCALLS: All I/O mockable for testing
- M-ESSENTIAL-FN-INHERENT: Core functionality in inherent methods

### Integration Points

**RT-TASK-010 → RT-TASK-007:**
```rust
// Supervisor uses Monitor<SupervisionEvent>
pub struct SupervisorNode<S, C, M>
where
    M: Monitor<SupervisionEvent>,
{
    monitor: M,  // InMemoryMonitor or NoopMonitor
}
```

**RT-TASK-007 → ActorSystem:**
```rust
// ActorSystem can spawn with supervisor
let actor_id = system.spawn()
    .actor(my_actor)
    .with_supervisor(supervisor_node)
    .start()
    .await?;
```

**RT-TASK-010 → RT-TASK-008:**
```rust
// Performance monitoring uses same Monitor<E> trait
let perf_monitor = InMemoryMonitor::<PerformanceEvent>::new(config);
```

## References

### Knowledge Base
- **KNOWLEDGE-RT-001**: Zero-Cost Actor Model Architecture
- **KNOWLEDGE-RT-003**: Supervisor Tree Implementation Strategies
- **KNOWLEDGE-RT-009**: Message Broker Architecture

### ADRs
- **ADR-RT-002**: Message Passing Architecture
- **ADR-RT-006**: Actor System Pub-Sub Architecture (RT-TASK-006)

### Workspace Standards
- **§2.1**: 3-Layer Import Organization
- **§3.2**: chrono DateTime<Utc> Standard
- **§4.3**: Module Architecture Patterns
- **§6.1**: YAGNI Principles
- **§6.2**: Avoid dyn Patterns
- **§6.3**: Microsoft Rust Guidelines Integration

### External References
- [Microsoft Rust Guidelines](https://microsoft.github.io/rust-guidelines/)
- [Erlang/OTP Supervisor Behavior](https://www.erlang.org/doc/design_principles/sup_princ.html)
- [BEAM VM Architecture](https://www.erlang.org/blog/beam-compiler-history/)

## Conclusion

These comprehensive action plans provide step-by-step implementation guidance for both RT-TASK-010 and RT-TASK-007. The plans are grounded in documented architecture decisions, workspace standards, and proven patterns from the BEAM ecosystem. Following these plans will result in production-quality fault tolerance and monitoring infrastructure for airssys-rt.

**Next Steps:**
1. Begin RT-TASK-010 Phase 1 (Core Traits & Types)
2. Complete RT-TASK-010 (2-3 days)
3. Begin RT-TASK-007 Phase 1 (Supervisor Traits)
4. Complete RT-TASK-007 (8-10 days)
5. Integration testing and documentation finalization
