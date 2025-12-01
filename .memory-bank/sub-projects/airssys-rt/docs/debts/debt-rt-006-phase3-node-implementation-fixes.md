# DEBT-RT-006: Phase 3 node.rs Implementation Alignment Fixes

**Sub-Project:** airssys-rt  
**Category:** Technical Debt  
**Created:** 2025-10-07  
**Status:** active  
**Priority:** HIGH - Blocking Phase 3 completion  
**Estimated Effort:** 2-3 hours  

## Context

During RT-TASK-007 Phase 3 implementation, `supervisor/node.rs` was created (~850 lines) but has 39 compilation errors due to API misalignment with Phase 1 implementations and RT-TASK-010 monitoring infrastructure.

**Root Cause:** Implementation was based on knowledge documentation examples (KNOWLEDGE-RT-003) without verifying actual Phase 1 API interfaces. Knowledge docs show aspirational/example code, not implemented reality.

## Problem Statement

The current `node.rs` implementation has the following mismatches:

### 1. Supervisor Trait Signature Mismatch
**Current (Wrong):**
```rust
impl<S, C, M> Supervisor<C> for SupervisorNode<S, C, M>
```

**Actual (From traits.rs):**
```rust
pub trait Supervisor: Send + Sync + 'static {
    type Child: Child;
    // No generic parameter on trait
}
```

### 2. Missing SupervisorNode Fields
**Current (Incomplete):**
```rust
pub struct SupervisorNode<S, C, M> {
    strategy: S,
    children: HashMap<ChildId, ChildHandle<C>>,
    backoff: RestartBackoff,  // ← Wrong: should be per-child
    monitor: M,
    child_order: Vec<ChildId>,
}
```

**Required (From KNOWLEDGE-RT-013):**
```rust
pub struct SupervisorNode<S, C, M> {
    id: Uuid,                                    // ← MISSING
    strategy: S,
    children: HashMap<ChildId, ChildHandle<C>>,
    child_order: Vec<ChildId>,
    backoff: HashMap<ChildId, RestartBackoff>,   // ← Wrong type
    monitor: M,
    state: SupervisorState,                      // ← MISSING
}
```

### 3. SupervisionEvent Structure Mismatch
**Current (Wrong):**
```rust
self.monitor.record(SupervisionEvent::ChildStarted {
    child_id: child_id.clone(),
    timestamp: Utc::now(),
})
```

**Actual (From monitoring/types.rs):**
```rust
pub struct SupervisionEvent {
    pub timestamp: DateTime<Utc>,
    pub supervisor_id: String,
    pub child_id: Option<String>,
    pub event_kind: SupervisionEventKind,
    pub metadata: HashMap<String, String>,
}

pub enum SupervisionEventKind {
    ChildStarted,
    ChildStopped,
    ChildFailed { error: String, restart_count: u32 },
    ChildRestarted { restart_count: u32 },
    RestartLimitExceeded { restart_count: u32, window: Duration },
    StrategyApplied { strategy: String },
}
```

### 4. SupervisorError Usage Mismatch
**Current (Wrong):**
```rust
SupervisorError::ChildStartFailed {
    id: child_id.clone(),  // ← Wrong: using ChildId
    source: e,             // ← Wrong: using C::Error directly
}

SupervisorError::ChildStartTimeout { ... }  // ← Doesn't exist
SupervisorError::ChildShutdownTimeout { ... }  // ← Wrong name
```

**Actual (From error.rs):**
```rust
pub enum SupervisorError {
    ChildNotFound { id: ChildId },
    ChildStartFailed { id: String, source: Box<dyn Error + Send + Sync> },
    ChildStopFailed { id: String, source: Box<dyn Error + Send + Sync> },
    RestartLimitExceeded { id: String, max_restarts: u32, window: Duration },
    ShutdownTimeout { id: String, timeout: Duration },  // ← Correct name
    // No ChildStartTimeout variant exists
}
```

### 5. RestartBackoff API Mismatch
**Current (Wrong):**
```rust
RestartBackoff::new()  // ← Takes 2 args, not 0
backoff.max_restarts()  // ← Field, not method
backoff.restart_window()  // ← Field, not method
backoff.calculate_delay(count)  // ← Takes 0 args, not 1
RestartBackoff::with_config(...)  // ← Doesn't exist
```

**Actual (From backoff.rs Phase 2):**
```rust
RestartBackoff::new(max_restarts: u32, restart_window: Duration)
RestartBackoff::with_delays(max_restarts, restart_window, base_delay, max_delay)
backoff.calculate_delay(&mut self) -> Duration  // Mutable, no args
// Fields are private, need accessor methods or direct field access isn't allowed
```

### 6. InMemoryMonitor Construction
**Current (Wrong):**
```rust
let monitor = InMemoryMonitor::new();  // ← Missing MonitoringConfig arg
```

**Actual (From monitoring/in_memory.rs):**
```rust
InMemoryMonitor::new(config: MonitoringConfig)
```

### 7. SupervisionDecision Enum
**Current (Wrong):**
```rust
SupervisionDecision::Ignore  // ← Doesn't exist
```

**Actual (Need to check types.rs):**
```rust
// Check what variants actually exist in Phase 1
```

## Action Plan

### Phase A: Read Actual Phase 1 Implementations (30 min)

**Files to Review:**
1. ✅ **Already reviewed:**
   - `.memory-bank/sub_projects/airssys-rt/docs/adr/adr_rt_004_child_trait_separation.md`
   - `.memory-bank/sub_projects/airssys-rt/docs/knowledges/knowledge_rt_013_task_007_010_action_plans.md`
   - `airssys-rt/src/supervisor/error.rs` (lines 1-150)
   - `airssys-rt/src/monitoring/types.rs` (lines 80-150)

2. **Still need to read:**
   - [ ] `airssys-rt/src/supervisor/traits.rs` (full Supervisor trait definition)
   - [ ] `airssys-rt/src/supervisor/types.rs` (SupervisionDecision, ChildState, etc.)
   - [ ] `airssys-rt/src/supervisor/backoff.rs` (actual API methods)
   - [ ] `airssys-rt/src/monitoring/in_memory.rs` (constructor signature)
   - [ ] `airssys-rt/src/monitoring/types.rs` (complete SupervisionEvent structure)

### Phase B: Fix SupervisorNode Structure (30 min)

1. **Add missing fields:**
   - [ ] Add `id: Uuid` field
   - [ ] Add `state: SupervisorState` field (need to check if exists in types.rs)
   - [ ] Change `backoff: RestartBackoff` to `backoff: HashMap<ChildId, RestartBackoff>`

2. **Add missing imports:**
   - [ ] `use uuid::Uuid;`
   - [ ] Remove unused imports (Arc, RwLock, OneForAll, etc.)

3. **Fix constructor:**
   - [ ] Generate UUID in `new()` method
   - [ ] Initialize backoff as empty HashMap
   - [ ] Initialize state field

### Phase C: Fix Supervisor Trait Implementation (45 min)

1. **Fix trait impl signature:**
   - [ ] Change from `impl<S, C, M> Supervisor<C>` to `impl<S, C, M> Supervisor`
   - [ ] Add `type Child = C;` associated type
   - [ ] Add `'static` bound to M parameter

2. **Fix method signatures:**
   - [ ] Change `Result<ChildId, Self::Error>` to `Result<ChildId, SupervisorError>`
   - [ ] Change `Result<(), Self::Error>` to `Result<(), SupervisorError>`
   - [ ] Fix `handle_child_error` signature (check actual parameter types)

3. **Fix method implementations:**
   - [ ] Remove internal helper methods or make them standalone
   - [ ] Implement methods according to Supervisor trait contract
   - [ ] Add per-child backoff management

### Phase D: Fix Monitoring Integration (30 min)

1. **Fix SupervisionEvent recording:**
   - [ ] Create full `SupervisionEvent` struct with all required fields
   - [ ] Use `SupervisionEventKind` enum for `event_kind` field
   - [ ] Add `supervisor_id: String` (convert from UUID)
   - [ ] Add `child_id: Option<String>` (convert from ChildId)
   - [ ] Add `metadata: HashMap<String, String>`

2. **Examples:**
```rust
// Correct way to record events:
self.monitor.record(SupervisionEvent {
    timestamp: Utc::now(),
    supervisor_id: self.id.to_string(),
    child_id: Some(child_id.to_string()),
    event_kind: SupervisionEventKind::ChildStarted,
    metadata: HashMap::new(),
}).await;
```

### Phase E: Fix Error Handling (30 min)

1. **Fix SupervisorError construction:**
   - [ ] Convert `ChildId` to `String` with `.to_string()`
   - [ ] Box child errors: `Box::new(e) as Box<dyn Error + Send + Sync>`
   - [ ] Remove ChildStartTimeout (doesn't exist)
   - [ ] Rename ChildShutdownTimeout to ShutdownTimeout
   - [ ] Add missing `id` field to RestartLimitExceeded

2. **Fix backoff method calls:**
   - [ ] Get backoff from HashMap: `self.backoff.get_mut(&child_id)`
   - [ ] Call `calculate_delay()` with mutable borrow, no args
   - [ ] Handle Option from HashMap lookup

### Phase F: Fix Tests (30 min)

1. **Fix test setup:**
   - [ ] Add `MonitoringConfig` to InMemoryMonitor::new() calls
   - [ ] Update test assertions for correct error types
   - [ ] Fix test helper construction

2. **Example:**
```rust
let config = MonitoringConfig {
    enabled: true,
    max_history_size: 100,
    severity_filter: EventSeverity::Info,
};
let monitor = InMemoryMonitor::new(config);
```

### Phase G: Remove Unused Code (15 min)

1. **Clean up imports:**
   - [ ] Remove `std::sync::Arc`
   - [ ] Remove `tokio::sync::RwLock`
   - [ ] Remove `OneForAll` (not used in node.rs)
   - [ ] Remove `ChildHealth` if not used

2. **Remove dead code:**
   - [ ] Check for any unused helper methods
   - [ ] Remove commented-out code

## Acceptance Criteria

- [ ] `cargo check --package airssys-rt` passes with zero errors
- [ ] `cargo clippy --package airssys-rt` passes with zero warnings
- [ ] `cargo test --package airssys-rt --lib supervisor::node` passes
- [ ] All 14 unit tests in node.rs passing
- [ ] No unused imports or dead code
- [ ] Code follows workspace standards (§2.1-§6.3)
- [ ] Proper error handling (no unwrap/expect)
- [ ] Monitoring integration working correctly

## Related Documentation

- **KNOWLEDGE-RT-013**: RT-TASK-007 Phase 3 action plan (lines 700-900)
- **ADR-RT-004**: Child Trait Separation architecture
- **KNOWLEDGE-RT-003**: Supervisor Tree Implementation Strategies
- **RT-TASK-007**: Task file with phase definitions

## Resolution Strategy

1. **Incremental fixes**: Fix one category at a time (structure → trait → monitoring → errors → tests)
2. **Verify after each phase**: Run `cargo check` after each major fix category
3. **Reference actual code**: Always check actual Phase 1 implementation, not knowledge docs
4. **Document learnings**: Update knowledge docs if they diverge from reality

## Estimated Timeline

- Phase A: 30 min (read actual implementations)
- Phase B: 30 min (fix structure)
- Phase C: 45 min (fix trait impl)
- Phase D: 30 min (fix monitoring)
- Phase E: 30 min (fix errors)
- Phase F: 30 min (fix tests)
- Phase G: 15 min (cleanup)

**Total:** 3 hours 30 minutes

## Success Metrics

- ✅ Zero compilation errors
- ✅ Zero clippy warnings
- ✅ 14/14 tests passing
- ✅ Proper API alignment with Phase 1
- ✅ Monitoring integration working
- ✅ Ready for Phase 3 completion (tree.rs implementation)

---

**Created:** 2025-10-07  
**Author:** RT-TASK-007 Phase 3 Implementation  
**Status:** Active - Ready to execute
