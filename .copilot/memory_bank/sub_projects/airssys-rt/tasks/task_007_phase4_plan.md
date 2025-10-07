# RT-TASK-007 Phase 4: Health Monitoring & Restart Logic

**Status**: In Progress  
**Created**: 2025-10-07  
**Estimated Duration**: 18-24 hours  
**Dependencies**: Phase 1-3 Complete ✅

---

## Overview

Phase 4 adds **optional** health monitoring capabilities to SupervisorNode, enabling:
- Periodic health checks of supervised children
- Automatic restart on health check failures
- Configurable failure thresholds
- Integration with existing restart strategies

## Design Philosophy

### YAGNI Compliance (§6.1)
- **Optional feature**: Health monitoring is opt-in, not mandatory
- **Simple API**: Single method to enable health checks
- **No new abstractions**: Extend existing SupervisorNode
- **No separate module**: Avoid creating `health.rs` until proven necessary

### Integration Points
- Leverage existing `Child::health_check()` method (already defined)
- Use existing `RestartBackoff` for rate limiting
- Emit `SupervisionEvent` for monitoring
- Work with all supervision strategies (OneForOne, OneForAll, RestForOne)

## Implementation Strategy

### Option A: Extend SupervisorNode (RECOMMENDED - YAGNI)
**Add health monitoring capabilities directly to SupervisorNode:**

```rust
// Add to SupervisorNode struct
pub struct SupervisorNode<S, C, M> {
    // ... existing fields ...
    
    /// Optional health monitoring configuration
    health_config: Option<HealthConfig>,
    
    /// Health check task handle (if enabled)
    health_task: Option<JoinHandle<()>>,
}

pub struct HealthConfig {
    /// How often to check child health
    check_interval: Duration,
    
    /// Timeout for each health check
    check_timeout: Duration,
    
    /// Consecutive failures before restart
    failure_threshold: u32,
    
    /// Per-child failure counts
    consecutive_failures: HashMap<ChildId, u32>,
}

impl<S, C, M> SupervisorNode<S, C, M> {
    /// Enable health monitoring with configuration.
    pub fn enable_health_checks(
        &mut self,
        check_interval: Duration,
        check_timeout: Duration,
        failure_threshold: u32,
    ) {
        // Implementation
    }
    
    /// Disable health monitoring.
    pub fn disable_health_checks(&mut self) {
        // Implementation
    }
    
    /// Manually trigger health check for a child.
    pub async fn check_child_health(
        &mut self,
        child_id: &ChildId,
    ) -> Result<ChildHealth, SupervisorError> {
        // Implementation
    }
}
```

**Pros**:
- ✅ Simple, minimal API surface
- ✅ No new modules or files
- ✅ Optional feature (YAGNI compliant)
- ✅ Direct integration with existing restart logic

**Cons**:
- ❌ Couples health monitoring to SupervisorNode
- ❌ Background task management in node lifecycle

### Option B: Separate HealthMonitor Module
**Create `src/supervisor/health.rs` with separate health monitoring:**

**Pros**:
- ✅ Separation of concerns
- ✅ Reusable health monitoring
- ✅ Easier to test in isolation

**Cons**:
- ❌ More complex architecture
- ❌ Additional abstraction layer
- ❌ Violates YAGNI (not needed yet)
- ❌ Requires integration API design

## Decision: Option A (Extend SupervisorNode)

**Rationale**:
1. **YAGNI**: We don't need separate health monitoring module yet
2. **Simplicity**: Fewer moving parts, simpler API
3. **Integration**: Direct access to children and restart logic
4. **Opt-in**: Can refactor later if complexity grows

---

## Implementation Checklist

### Phase 4a: Health Configuration (6-8 hours)

**Add to `SupervisorNode` struct:**
- [ ] `health_config: Option<HealthConfig>` field
- [ ] `HealthConfig` struct with interval, timeout, threshold
- [ ] `consecutive_failures: HashMap<ChildId, u32>` tracking
- [ ] `enable_health_checks()` method
- [ ] `disable_health_checks()` method
- [ ] `is_health_monitoring_enabled()` query method

**Testing:**
- [ ] Test health config initialization
- [ ] Test enable/disable toggle
- [ ] Test failure count tracking
- [ ] ~5-6 unit tests

### Phase 4b: Health Check Logic (8-10 hours)

**Add health checking methods:**
- [ ] `check_child_health(child_id)` - Manual check with timeout
- [ ] `should_restart_unhealthy(child_id, health)` - Decision logic
- [ ] `handle_health_check_failure(child_id)` - Increment failures, trigger restart
- [ ] `reset_health_status(child_id)` - Clear failures on successful check
- [ ] Integration with `restart_child()` method

**Health Check Decision Logic:**
```rust
async fn check_child_health(&mut self, child_id: &ChildId) -> Result<ChildHealth, SupervisorError> {
    // 1. Get child handle
    // 2. Call child.health_check() with timeout
    // 3. Update consecutive_failures based on result
    // 4. If threshold exceeded, trigger restart
    // 5. Emit SupervisionEvent
    // 6. Return health status
}
```

**Testing:**
- [ ] Test successful health check
- [ ] Test health check timeout
- [ ] Test failure threshold logic
- [ ] Test restart triggered by health check
- [ ] Test health status reset after success
- [ ] ~8-10 unit tests

### Phase 4c: Background Health Monitoring (4-6 hours)

**Background task implementation:**
- [ ] Spawn background task in `enable_health_checks()`
- [ ] Periodic check all children on interval
- [ ] Graceful shutdown of background task
- [ ] Task handle storage and lifecycle

**Considerations:**
```rust
// Background task pseudo-code
async fn health_monitoring_task(interval: Duration) {
    loop {
        tokio::time::sleep(interval).await;
        
        // Check all children
        for child_id in children.keys() {
            if let Err(e) = check_child_health(child_id).await {
                // Log error, emit event
            }
        }
    }
}
```

**Challenge**: How to share mutable access to SupervisorNode between task and main thread?

**Solutions**:
1. **Channel-based**: Background task sends health check requests
2. **Periodic manual calls**: No background task, user calls manually
3. **Arc<Mutex<>>**: Shared mutable access (not ideal for async)

**Decision**: **Option 2 (Manual) for Phase 4** 
- YAGNI: Background task can be added in future phase if needed
- Users can implement their own background checking loop
- Keeps implementation simple and testable

**Testing:**
- [ ] Test manual periodic checking
- [ ] Test concurrent health checks
- [ ] Test cleanup on disable
- [ ] ~5-6 unit tests

---

## API Design

### Public API

```rust
// Enable health monitoring
supervisor.enable_health_checks(
    Duration::from_secs(30),  // check_interval
    Duration::from_secs(5),   // check_timeout
    3,                        // failure_threshold
);

// Manual health check
let health = supervisor.check_child_health(&child_id).await?;

// Check if enabled
if supervisor.is_health_monitoring_enabled() {
    // ...
}

// Disable when done
supervisor.disable_health_checks();
```

### Integration Example

```rust
// User-driven background checking
tokio::spawn(async move {
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        
        for child_id in supervisor.child_ids() {
            if let Err(e) = supervisor.check_child_health(&child_id).await {
                eprintln!("Health check failed: {}", e);
            }
        }
    }
});
```

---

## Testing Strategy

### Unit Tests (~20-25 tests)

**Health Configuration (5-6 tests):**
- enable_health_checks_initializes_config
- disable_health_checks_clears_config
- is_health_monitoring_enabled_returns_correct_state
- consecutive_failures_tracking
- failure_threshold_configuration

**Health Check Logic (8-10 tests):**
- check_child_health_success
- check_child_health_degraded
- check_child_health_failed
- check_child_health_timeout
- consecutive_failure_increment
- consecutive_failure_reset_on_success
- restart_triggered_on_threshold
- health_check_emits_monitoring_event
- health_check_with_no_config_returns_error

**Integration Tests (5-6 tests):**
- health_check_integration_with_restart
- health_check_respects_restart_policy
- health_check_works_with_one_for_one
- health_check_works_with_one_for_all
- health_check_disabled_by_default
- manual_periodic_health_checking

**Edge Cases (2-3 tests):**
- health_check_nonexistent_child
- health_check_stopped_child
- concurrent_health_checks

---

## Acceptance Criteria

- ✅ Health monitoring is optional (disabled by default)
- ✅ Can enable/disable health monitoring dynamically
- ✅ Manual health checks work with timeout
- ✅ Consecutive failure tracking per child
- ✅ Automatic restart on threshold exceeded
- ✅ Integration with existing restart strategies
- ✅ SupervisionEvent emitted for health checks
- ✅ 20-25 tests passing
- ✅ Zero warnings
- ✅ Complete rustdoc documentation

---

## Timeline

| Phase | Task | Duration | Status |
|-------|------|----------|--------|
| 4a | Health Configuration | 6-8h | Not Started |
| 4b | Health Check Logic | 8-10h | Not Started |
| 4c | Manual Periodic Checks | 4-6h | Not Started |
| **Total** | | **18-24h** | **0% Complete** |

---

## Next Steps After Phase 4

**Phase 5: Integration & Examples (4-6 hours)**
- Create `examples/supervisor_health.rs`
- Create `examples/supervisor_strategies.rs`
- Integration tests in `tests/supervisor_integration.rs`
- Update progress tracking and documentation

---

## References

- **Child::health_check()**: Already defined in `supervisor/traits.rs`
- **ChildHealth enum**: Defined in `supervisor/types.rs`
- **SupervisorNode**: `supervisor/node.rs` (current implementation)
- **YAGNI Principles**: Workspace standards §6.1
- **Microsoft Guidelines**: M-DESIGN-FOR-AI, M-DI-HIERARCHY
