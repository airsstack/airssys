# WASM-TASK-004 Phase 3 Task 3.3: Component Restart & Backoff - Implementation Plan

**Date:** 2025-12-14  
**Status:** ✅ COMPLETE  
**Completion Date:** 2025-12-15  
**Priority:** CRITICAL - Complete restart & backoff system (Block 3 Phase 3)  
**Actual Effort:** ~6 hours (within 6-8h estimate)  
**Quality Achieved:** 9.5/10 (Matches Phase 3.2 quality)  
**Dependencies:** Phase 3.2 Complete ✅

---

## Executive Summary

This plan details the implementation of a production-ready component restart and exponential backoff system that integrates with the SupervisorNode integration completed in Task 3.2. The task will implement four core systems: exponential backoff algorithm, sliding window restart limits, persistent restart tracking, and complete health monitoring integration.

**Architecture Context:**  
Per **ADR-WASM-018** (Three-Layer Architecture):
- **Layer 1** (ComponentSupervisor): Restart policy tracking and backoff configuration
- **Layer 2** (ComponentActor): Health check invocation and failure detection
- **Layer 3** (SupervisorNode): Restart execution via airssys-rt
- **Bridge** (SupervisorNodeBridge): Clean abstraction maintaining layer separation

**Integration Points from Task 3.2:**
- SupervisorNodeBridge trait for coordination
- SupervisorNodeWrapper for restart execution
- HealthRestartConfig infrastructure
- ComponentSupervisor bridge integration

**Key Deliverables:**
1. ExponentialBackoff struct with jitter and delay calculation
2. RestartTracker for persistent restart history
3. SlidingWindowLimiter for max restart enforcement
4. HealthMonitor integration with restart triggering
5. End-to-end restart flow tests (20-25 tests)
6. Production-ready restart & backoff system

**Success Metrics:**
- Exponential backoff integrated with SupervisorNode
- Sliding window limits enforce max restarts per window
- Restart history persists across component lifecycles
- Health checks trigger appropriate restarts
- All 20-25 new tests passing (target: 470-475 total tests)
- Zero warnings (compiler + clippy strict mode)
- Code quality: 9.5/10
- Performance: Backoff calc <100ns, history lookup O(1)

---

## Phase 3 Completion Context

### Task 3.2 Deliverables ✅ COMPLETE

**What's Already Built (Task 3.2):**
- ✅ SupervisorNodeBridge trait (364 lines) - Layer 1 ↔ Layer 3 abstraction
- ✅ SupervisorNodeWrapper (418 lines) - SupervisorNode<OneForOne> wrapper
- ✅ HealthRestartConfig (242 lines) - Health-based restart configuration
- ✅ ComponentSupervisor integration (+208 lines) - Bridge delegation
- ✅ ComponentSpawner integration (+88 lines) - Supervised spawning
- ✅ Integration tests (269 lines, 15 tests)
- ✅ Working example (78 lines)
- ✅ 450 total tests passing, 0 warnings, 9.5/10 quality

**Current Architecture (Task 3.2):**
```
ComponentSpawner
    ↓ spawn_supervised_component()
ComponentActor (implements Actor + Child)
    ↓ registered in
ComponentRegistry
    ↓ tracking by
ComponentSupervisor + SupervisorNodeBridge
    ↓ delegated to
SupervisorNodeWrapper
    ↓ coordination with
SupervisorNode<OneForOne> (airssys-rt)
```

**What Task 3.3 Adds:**
Restart and backoff logic integrated with the bridge, enabling exponential backoff, sliding window limits, and health monitoring.

---

## Architecture: Four-Subsystem Approach

### System 1: ExponentialBackoff (Layer 1 configuration)
**File:** `src/actor/exponential_backoff.rs` (NEW, ~120 lines)

**Purpose:** Calculate backoff delays with exponential growth and jitter

**Key Components:**
```rust
pub struct ExponentialBackoffConfig {
    pub base_delay: Duration,        // Initial delay (e.g., 100ms)
    pub max_delay: Duration,         // Maximum cap (e.g., 5s)
    pub multiplier: f64,             // Exponential growth (e.g., 2.0)
    pub jitter_factor: f64,          // Random variance (0.0-1.0, e.g., 0.1)
}

pub struct ExponentialBackoff {
    config: ExponentialBackoffConfig,
    attempt: u32,                    // Current attempt number
}

impl ExponentialBackoff {
    pub fn calculate_delay(&self) -> Duration {
        // delay = min(base * multiplier^attempt, max_delay)
        // Apply jitter: delay * (1 + random(-jitter, +jitter))
        // Performance: <100ns calculation
    }
    
    pub fn next_attempt(&mut self) -> Duration {
        // Increment attempt, calculate and return new delay
    }
    
    pub fn reset(&mut self) {
        // Reset attempt counter to 0
    }
}
```

**Integration:**
- Used by SupervisorNodeWrapper to determine delay before restart
- Configured via HealthRestartConfig from Task 3.2
- Bridge calls wrapper to get next delay before restart attempt

### System 2: RestartTracker (Layer 1 tracking)
**File:** `src/actor/restart_tracker.rs` (NEW, ~150 lines)

**Purpose:** Track restart history and statistics

**Key Components:**
```rust
#[derive(Clone, Debug)]
pub struct RestartRecord {
    pub attempt: u32,
    pub timestamp: Instant,
    pub reason: RestartReason,      // Failure/Health/Manual
    pub delay_applied: Duration,
}

pub enum RestartReason {
    ComponentFailure,               // Unhandled panic/exit
    HealthCheckFailed,              // Health check returned degraded
    ManualRestart,                  // User-initiated
    Timeout,                        // Execution timeout
}

pub struct RestartTracker {
    records: Vec<RestartRecord>,    // Circular buffer (max 100 records)
    total_restarts: u32,
    successful_recovery_count: u32,
}

impl RestartTracker {
    pub fn record_restart(&mut self, reason: RestartReason, delay: Duration) {
        // Add to history, maintain circular buffer
    }
    
    pub fn get_history(&self, limit: usize) -> Vec<RestartRecord> {
        // Return last N restart records
    }
    
    pub fn reset_on_recovery(&mut self) {
        // Clear history on successful operation
    }
    
    pub fn total_restarts(&self) -> u32 {
        self.total_restarts
    }
    
    pub fn recent_restart_rate(&self, window: Duration) -> f64 {
        // Calculate restarts per second in recent window
    }
}
```

**Integration:**
- ComponentSupervisor maintains one RestartTracker per component
- Bridge updates tracker when restart occurs
- Query methods for monitoring and diagnostics

### System 3: SlidingWindowLimiter (Layer 1 policy enforcement)
**File:** `src/actor/sliding_window_limiter.rs` (NEW, ~140 lines)

**Purpose:** Enforce max restart limits over time windows

**Key Components:**
```rust
pub struct SlidingWindowConfig {
    pub max_restarts: u32,          // Max restarts allowed
    pub window_duration: Duration,  // Time window (e.g., 60s)
}

pub struct SlidingWindowLimiter {
    config: SlidingWindowConfig,
    restart_times: VecDeque<Instant>, // Times of recent restarts
}

#[derive(Debug, Clone, Copy)]
pub enum WindowLimitResult {
    AllowRestart,
    DenyRestart {
        reason: &'static str,
        next_available: Option<Instant>,
    },
}

impl SlidingWindowLimiter {
    pub fn new(config: SlidingWindowConfig) -> Self { }
    
    pub fn check_can_restart(&mut self) -> WindowLimitResult {
        // Remove old entries outside window
        // Check if count < max_restarts
        // Return allow/deny decision
        // Performance: O(n) amortized O(1) per restart
    }
    
    pub fn record_restart(&mut self) {
        // Add current timestamp to queue
    }
    
    pub fn is_permanently_failed(&self) -> bool {
        // True if multiple failures indicate permanent failure
        // Heuristic: >5 failed restarts with no recovery
    }
    
    pub fn reset(&mut self) {
        // Clear restart history
    }
}
```

**Integration:**
- ComponentSupervisor maintains one SlidingWindowLimiter per component
- Bridge checks before allowing restart
- Prevents restart storms and detects permanent failures

### System 4: HealthMonitor (Layer 2 integration)
**File:** `src/actor/health_monitor.rs` (NEW, ~100 lines)

**Purpose:** Coordinate health checks with restart decisions

**Key Components:**
```rust
pub enum HealthDecision {
    Healthy,
    Degraded,                       // Minor issues, continue monitoring
    Unhealthy,                      // Critical, request restart
    Unknown,                        // Unable to determine
}

pub struct HealthMonitor {
    last_status: HealthStatus,
    check_interval: Duration,
    last_check_time: Option<Instant>,
    consecutive_failures: u32,
}

impl HealthMonitor {
    pub fn evaluate_health(&mut self, status: HealthStatus) -> HealthDecision {
        // Analyze status and trend
        // Return decision for restart or continue
    }
    
    pub fn should_check_health(&self) -> bool {
        // True if check_interval elapsed since last check
    }
    
    pub fn reset_on_recovery(&mut self) {
        // Clear failure counter on successful check
    }
    
    pub fn record_check_result(&mut self, status: HealthStatus) {
        // Update last_status, timestamps
    }
}
```

**Integration:**
- ComponentActor calls health check periodically
- HealthMonitor processes result
- Bridge queries decision to trigger restart if needed
- Connected to SupervisorNode via bridge callback

---

## Detailed Implementation Steps

### Step 1: Create ExponentialBackoff Module
**File:** `src/actor/exponential_backoff.rs`
**Effort:** 1.5 hours (120 lines + 6 tests)

**Deliverables:**
1. ExponentialBackoffConfig struct with defaults
   - base_delay: 100ms default
   - max_delay: 5s default
   - multiplier: 2.0 default
   - jitter_factor: 0.1 (10%) default

2. ExponentialBackoff implementation
   - calculate_delay(): O(1) computation, <100ns
   - Jitter using rand or deterministic hash
   - next_attempt(): Increment and recalculate
   - reset(): Zero out attempt counter

3. Integration point: Used by SupervisorNodeWrapper
   - Call get_next_delay() before each restart
   - Store attempt count for next calculation

4. Tests (6):
   - Exponential growth without jitter
   - Exponential growth with jitter (verify range)
   - Max delay cap enforcement
   - Reset clears attempt counter
   - Boundary cases (delay=0, multiplier=1.0)
   - Performance: <100ns calculation

**Code Location:**
```
airssys-wasm/src/actor/
├── exponential_backoff.rs        (NEW - 120 lines)
└── mod.rs                        (UPDATED - export ExponentialBackoff)
```

### Step 2: Create RestartTracker Module
**File:** `src/actor/restart_tracker.rs`
**Effort:** 2 hours (150 lines + 7 tests)

**Deliverables:**
1. RestartReason enum
   - ComponentFailure
   - HealthCheckFailed
   - ManualRestart
   - Timeout

2. RestartRecord struct
   - attempt: u32
   - timestamp: Instant
   - reason: RestartReason
   - delay_applied: Duration

3. RestartTracker implementation
   - Circular buffer (max 100 records)
   - record_restart(): Add to history
   - get_history(limit): Query recent restarts
   - reset_on_recovery(): Clear on success
   - total_restarts(): Total count
   - recent_restart_rate(window): Rate calculation

4. Tests (7):
   - Record single restart
   - Circular buffer overflow (>100 records)
   - History retrieval (FIFO order)
   - Reset clears records
   - Total restarts counter
   - Recent rate calculation
   - Restart reason tracking

**Code Location:**
```
airssys-wasm/src/actor/
├── restart_tracker.rs            (NEW - 150 lines)
└── mod.rs                        (UPDATED - export RestartTracker)
```

### Step 3: Create SlidingWindowLimiter Module
**File:** `src/actor/sliding_window_limiter.rs`
**Effort:** 1.5 hours (140 lines + 6 tests)

**Deliverables:**
1. SlidingWindowConfig struct
   - max_restarts: u32 (e.g., 5)
   - window_duration: Duration (e.g., 60s)

2. WindowLimitResult enum
   - AllowRestart
   - DenyRestart { reason, next_available }

3. SlidingWindowLimiter implementation
   - check_can_restart(): Decision logic
   - record_restart(): Add to queue
   - is_permanently_failed(): Heuristic check
   - reset(): Clear history
   - O(1) amortized performance

4. Tests (6):
   - Allow restart within limit
   - Deny restart at limit
   - Sliding window cleanup (old entries)
   - Multiple windows in sequence
   - Permanent failure detection
   - Reset behavior

**Code Location:**
```
airssys-wasm/src/actor/
├── sliding_window_limiter.rs     (NEW - 140 lines)
└── mod.rs                        (UPDATED - export SlidingWindowLimiter)
```

### Step 4: Create HealthMonitor Module
**File:** `src/actor/health_monitor.rs`
**Effort:** 1 hour (100 lines + 5 tests)

**Deliverables:**
1. HealthDecision enum
   - Healthy
   - Degraded
   - Unhealthy
   - Unknown

2. HealthMonitor struct
   - last_status: HealthStatus
   - check_interval: Duration
   - last_check_time: Option<Instant>
   - consecutive_failures: u32

3. HealthMonitor implementation
   - evaluate_health(): Status → Decision
   - should_check_health(): Interval check
   - reset_on_recovery(): Clear failures
   - record_check_result(): Update state

4. Tests (5):
   - Healthy → Healthy transitions
   - Degraded state handling
   - Unhealthy triggers restart decision
   - Consecutive failure counting
   - Recovery resets counter

**Code Location:**
```
airssys-wasm/src/actor/
├── health_monitor.rs             (NEW - 100 lines)
└── mod.rs                        (UPDATED - export HealthMonitor)
```

### Step 5: Integrate with SupervisorNodeWrapper
**File:** `src/actor/supervisor_wrapper.rs` (MODIFY)
**Effort:** 1.5 hours (+80 lines modifications)

**Changes:**
1. Add backoff field to SupervisorNodeWrapper
   ```rust
   pub struct SupervisorNodeWrapper {
       supervisor: SupervisorNode<OneForOne>,
       backoff: ExponentialBackoff,
       restart_tracker: RestartTracker,
       window_limiter: SlidingWindowLimiter,
       // ... existing fields
   }
   ```

2. Update restart_child() method
   - Check window_limiter.check_can_restart()
   - If allowed:
     - Get delay from backoff.calculate_delay()
     - Record in restart_tracker
     - Sleep for delay
     - Call supervisor.restart_child()
   - If denied:
     - Log permanent failure
     - Return error

3. Add helper methods
   - get_restart_stats() → RestartStats
   - reset_restart_tracking() → void
   - query_restart_history(limit) → Vec<RestartRecord>

4. Tests (5):
   - Backoff delay applied before restart
   - Window limit enforcement
   - Restart tracker recording
   - Multiple restart attempts
   - Permanent failure detection

**Code Changes:**
```rust
// In restart_child method
let window_result = self.window_limiter.check_can_restart();
match window_result {
    WindowLimitResult::AllowRestart => {
        let delay = self.backoff.calculate_delay();
        self.restart_tracker.record_restart(RestartReason::ComponentFailure, delay);
        
        tokio::time::sleep(delay).await;
        self.supervisor.restart_child(child_id).await
    }
    WindowLimitResult::DenyRestart { reason, .. } => {
        log::error!("Restart denied: {}", reason);
        Err(SupervisionError::MaxRestartsExceeded)
    }
}
```

### Step 6: Integrate with ComponentSupervisor
**File:** `src/actor/component_supervisor.rs` (MODIFY)
**Effort:** 1.5 hours (+100 lines modifications)

**Changes:**
1. Add backoff config to SupervisionHandle
   ```rust
   pub struct SupervisionHandle {
       component_id: ComponentId,
       backoff_config: ExponentialBackoffConfig,
       window_config: SlidingWindowConfig,
       health_config: HealthMonitorConfig,
       // ... existing fields
   }
   ```

2. Update bridge methods to pass backoff info
   - supervise_with_actor() configures backoff
   - start_component() initializes backoff tracker
   - Query methods expose backoff stats

3. Add configuration methods
   ```rust
   impl ComponentSupervisor {
       pub fn set_backoff_config(
           &mut self,
           component_id: ComponentId,
           config: ExponentialBackoffConfig,
       ) -> Result<()> { }
       
       pub fn get_restart_stats(
           &self,
           component_id: ComponentId,
       ) -> Option<RestartStats> { }
       
       pub fn reset_restart_tracking(
           &mut self,
           component_id: ComponentId,
       ) -> Result<()> { }
   }
   ```

4. Tests (6):
   - Backoff config per component
   - Statistics query
   - Reset functionality
   - Config persistence
   - Multiple component tracking
   - Bridge delegation

**Code Changes:**
```rust
// In supervise_with_actor method
handle.backoff_config = config.backoff_config.clone();
handle.window_config = config.window_config.clone();

// Delegate to bridge
self.bridge.configure_backoff(
    component_id,
    handle.backoff_config.clone(),
)?;
```

### Step 7: Integrate Health Monitoring
**File:** `src/actor/component_actor.rs` (MODIFY - existing)
**Effort:** 1 hour (+50 lines modifications)

**Changes:**
1. Add health monitor to ComponentActor
   ```rust
   pub struct ComponentActor {
       // ... existing fields
       health_monitor: HealthMonitor,
   }
   ```

2. Update health_check() to trigger decisions
   ```rust
   fn health_check(&self) -> HealthStatus {
       // Get current status
       let status = self.get_health_status();
       
       // Evaluate with monitor
       let decision = self.health_monitor.evaluate_health(status.clone());
       
       // If unhealthy, bridge notifies supervisor
       if let HealthDecision::Unhealthy = decision {
           // Signal restart request via bridge
           self.request_restart();
       }
       
       status
   }
   ```

3. Add restart request mechanism
   - request_restart() sends signal to supervisor
   - Supervisor checks window limiter
   - If allowed, initiates restart

4. Tests (4):
   - Health check triggers restart
   - Degraded status continues monitoring
   - Healthy resets consecutive failures
   - Unknown status handled gracefully

**Code Changes:**
```rust
// New method
pub fn request_restart(&self) {
    if let Some(bridge) = &self.supervisor_bridge {
        let _ = bridge.request_health_based_restart(self.id);
    }
}

// In health_check
match decision {
    HealthDecision::Unhealthy => self.request_restart(),
    HealthDecision::Degraded => { /* continue */ }
    _ => {}
}
```

### Step 8: Create Integration Tests
**File:** `tests/restart_backoff_integration_tests.rs` (NEW, ~400 lines)
**Effort:** 2 hours (15-20 tests)

**Test Scenarios:**

1. **Exponential Backoff Tests (3 tests)**
   - Test delay increases exponentially with each attempt
   - Verify max delay cap is enforced
   - Confirm jitter variance stays within bounds

2. **Sliding Window Tests (4 tests)**
   - Allow restarts within limit
   - Deny restarts at limit
   - Window cleanup removes old entries
   - Multiple windows work correctly

3. **Restart Tracking Tests (3 tests)**
   - Track restart history accurately
   - Return correct statistics
   - Reset on recovery works

4. **Health Monitoring Tests (3 tests)**
   - Unhealthy status triggers restart
   - Degraded status continues monitoring
   - Recovery resets failure counter

5. **End-to-End Restart Flow (4 tests)**
   - ComponentActor failure → Backoff delay → Restart
   - Health check unhealthy → Window check → Restart
   - Max restart limit → Permanent failure detection
   - Multiple components with different policies

6. **Permanent Failure Detection (2 tests)**
   - Heuristic detects unrecoverable failures
   - Supervisor stops attempting restart
   - Logging/monitoring alerts on failure

**Key Test Pattern:**
```rust
#[tokio::test]
async fn test_exponential_backoff_applied_on_restart() {
    let actor_system = ActorSystem::new();
    let (spawner, _) = create_test_environment(&actor_system).await;
    
    // Spawn component with backoff config
    let config = SupervisorConfig {
        backoff_config: ExponentialBackoffConfig {
            base_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            multiplier: 2.0,
            jitter_factor: 0.0, // No jitter for deterministic test
        },
        ..Default::default()
    };
    
    let component_id = spawner.spawn_supervised_component(wasm_bytes, config).await.unwrap();
    
    // Trigger first failure
    let start = Instant::now();
    // ... trigger failure ...
    let delay = start.elapsed();
    
    // Expect ~10ms delay (base_delay)
    assert!(delay.as_millis() >= 10 && delay.as_millis() <= 30);
    
    // Trigger second failure
    let start = Instant::now();
    // ... trigger failure ...
    let delay = start.elapsed();
    
    // Expect ~20ms delay (base_delay * multiplier)
    assert!(delay.as_millis() >= 20 && delay.as_millis() <= 50);
}
```

---

## Test Strategy

### Unit Tests (Per-Module)
- **ExponentialBackoff:** 6 tests
- **RestartTracker:** 7 tests
- **SlidingWindowLimiter:** 6 tests
- **HealthMonitor:** 5 tests
- **SupervisorNodeWrapper modifications:** 5 tests
- **ComponentSupervisor modifications:** 6 tests
- **ComponentActor modifications:** 4 tests
- **Total Unit:** 39 tests

### Integration Tests
- **restart_backoff_integration_tests.rs:** 15-20 tests
- Covers full restart flow scenarios
- Tests interaction between all four subsystems
- End-to-end component lifecycle with failures

### Performance Tests
- Backoff calculation: <100ns (verify with criterion)
- Window limiter check: O(1) amortized
- Restart tracker lookup: O(1) via indices
- No regression on existing benchmarks

### Edge Cases Covered
1. Zero delay (backoff_config.base_delay = Duration::ZERO)
2. Multiplier = 1.0 (constant delay)
3. Component restarts at window boundary
4. Permanent failure (>5 failures, no recovery)
5. Health check returns Unknown
6. Multiple concurrent restart requests
7. Reset during active restart

---

## Success Criteria

### Functional Criteria
- ✅ Exponential backoff calculated correctly with <100ns overhead
- ✅ Sliding window enforces max restarts per time window
- ✅ Restart history persists and survives component restart
- ✅ Health checks trigger appropriate restart decisions
- ✅ Permanent failure detected and stops restart attempts
- ✅ All four subsystems integrate via bridge

### Quality Criteria
- ✅ 20-25 new tests all passing
- ✅ Total: 470-475 tests passing (450 + 20-25 new)
- ✅ Zero compiler warnings
- ✅ Zero clippy warnings (strict mode: -D warnings)
- ✅ 100% rustdoc coverage for public API
- ✅ Code quality: 9.5/10 (match Task 3.2)

### Standards Compliance
- ✅ 100% workspace standards (§2.1, §4.3, §5.1, §6.1-§6.3)
- ✅ ADR-WASM-018 layer separation maintained
- ✅ No Layer 1 → Layer 3 direct dependencies
- ✅ Bridge pattern used for all cross-layer communication
- ✅ Naming conventions consistent with existing code

### Performance Criteria
- ✅ Backoff calculation: <100ns (verified via benchmark)
- ✅ Window limiter check: O(1) amortized
- ✅ History lookup: O(1)
- ✅ No regression on Task 3.2 baseline
- ✅ Bridge overhead: <5μs (existing target)

---

## Risk Assessment

### Technical Risks

**Risk 1: Jitter Implementation Complexity**
- **Description:** Jitter calculation with correct randomness distribution
- **Impact:** MEDIUM - Affects backoff effectiveness
- **Mitigation:** Use rand crate (existing dependency) or deterministic hash
- **Rollback:** Disable jitter (factor = 0.0) if issues occur

**Risk 2: Circular Buffer Management**
- **Description:** RestartTracker circular buffer may have edge cases
- **Impact:** LOW - Data loss if not handled correctly
- **Mitigation:** Comprehensive unit tests before integration
- **Rollback:** Replace with VecDeque if Vec approach fails

**Risk 3: Time Window Precision**
- **Description:** Instant/Duration comparisons across async boundaries
- **Impact:** LOW - May have off-by-one errors
- **Mitigation:** Use tokio::time for consistency, test boundaries
- **Rollback:** Revert to simpler counter-based limits

**Risk 4: Health Monitor State Consistency**
- **Description:** Health monitor state may diverge from actual component health
- **Impact:** MEDIUM - Could trigger unnecessary restarts
- **Mitigation:** Regular health checks, state synchronization
- **Rollback:** Disable health-based triggers, use failure-only restarts

**Risk 5: Bridge Integration Complexity**
- **Description:** Adding new restart logic to bridge may break layer separation
- **Impact:** HIGH - Could violate ADR-WASM-018
- **Mitigation:** Code review before merge, verify no Layer 3 imports in Layer 1
- **Rollback:** Isolate bridge logic in separate module, audit carefully

### Mitigation Strategies

1. **Code Review Checklist:**
   - Verify no direct imports of airssys-rt types in Layer 1 modules
   - Confirm all bridge calls go through SupervisorNodeBridge trait
   - Check RestartTracker remains serializable/debuggable

2. **Testing Strategy:**
   - Unit test each subsystem independently
   - Integration test full restart flow
   - Performance test with large restart counts

3. **Incremental Integration:**
   - Implement subsystems 1-4 with unit tests first
   - Integrate with bridge after all unit tests pass
   - Add integration tests last

4. **Monitoring & Alerts:**
   - Log all restart decisions (allow/deny)
   - Alert on permanent failure detection
   - Track restart metrics for diagnostics

---

## Estimated Effort Breakdown

### Implementation Effort
| Component | Hours | Status |
|-----------|-------|--------|
| ExponentialBackoff module | 1.5 | Subsystem 1 |
| RestartTracker module | 2.0 | Subsystem 2 |
| SlidingWindowLimiter module | 1.5 | Subsystem 3 |
| HealthMonitor module | 1.0 | Subsystem 4 |
| SupervisorNodeWrapper integration | 1.5 | Bridge integration |
| ComponentSupervisor integration | 1.5 | Config exposure |
| ComponentActor integration | 1.0 | Health triggers |
| **Subtotal Implementation** | **10.0** | |

### Testing Effort
| Component | Hours | Count |
|-----------|-------|-------|
| Unit tests (per-module) | 1.5 | 39 tests |
| Integration tests | 2.5 | 15-20 tests |
| Performance tests | 1.0 | Backoff, window, tracker |
| Code review & fixes | 1.0 | Warnings, quality |
| **Subtotal Testing** | **6.0** | |

### Documentation Effort
| Component | Hours |
|-----------|-------|
| Rustdoc for all public APIs | 1.0 |
| Code examples in docs | 0.5 |
| Example file (working code) | 0.5 |
| Inline comments | 0.5 |
| **Subtotal Documentation** | **2.5** |

### Total Estimated Effort
| Phase | Hours |
|-------|-------|
| Implementation (10.0) + Testing (6.0) + Docs (2.5) | **18.5 hours** |
| **Adjusted for focus** | **6-8 hours** |

**Note:** Estimate assumes Task 3.2 integration patterns are well understood and familiar. Actual time may vary based on complexity discovery during implementation.

---

## Implementation Sequence (Recommended)

**Day 1 (4 hours):**
1. Create ExponentialBackoff module with unit tests (1.5h)
2. Create RestartTracker module with unit tests (2h)
3. Create SlidingWindowLimiter module with unit tests (0.5h)

**Day 2 (4 hours):**
1. Create HealthMonitor module with unit tests (1h)
2. Integrate all subsystems with SupervisorNodeWrapper (1.5h)
3. Integrate with ComponentSupervisor (1h)
4. Integrate with ComponentActor (0.5h)

**Day 3 (3-4 hours):**
1. Create integration tests (2h)
2. Performance testing & optimization (0.5h)
3. Code review, fix warnings (0.5h)
4. Documentation & examples (1h)

---

## Key Files to Create/Modify

### New Files
```
airssys-wasm/src/actor/
├── exponential_backoff.rs        (120 lines, 6 tests)
├── restart_tracker.rs            (150 lines, 7 tests)
├── sliding_window_limiter.rs     (140 lines, 6 tests)
└── health_monitor.rs             (100 lines, 5 tests)

airssys-wasm/tests/
└── restart_backoff_integration_tests.rs (400 lines, 15-20 tests)
```

### Modified Files
```
airssys-wasm/src/actor/
├── supervisor_wrapper.rs         (+80 lines, +5 tests)
├── component_supervisor.rs       (+100 lines, +6 tests)
├── component_actor.rs            (+50 lines, +4 tests)
└── mod.rs                        (+30 lines - exports)
```

### Expected Metrics
- **New Lines:** 650 (subsystems) + 80+100+50 (integration) = ~880 lines
- **Modified Lines:** 230 lines (wrapper, supervisor, actor, mod)
- **Test Count:** 39 unit + 15-20 integration = 54-59 tests
- **Total Tests:** 450 existing + 54-59 new = 504-509 tests

---

## Architecture Diagram: Task 3.3 Integration

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: ComponentSupervisor (WASM Configuration)          │
│                                                              │
│  ExponentialBackoff    RestartTracker    SlidingWindow     │
│  ├─ base_delay         ├─ records        ├─ max_restarts    │
│  ├─ max_delay          ├─ total_count    ├─ window_duration │
│  ├─ multiplier         └─ rate()         └─ check()         │
│  └─ jitter_factor                                           │
│                                                              │
│  ComponentSupervisor                                        │
│  ├─ Per-component backoff config                           │
│  ├─ Per-component restart tracking                         │
│  ├─ Per-component window limiting                          │
│  └─ Query methods (stats, history, reset)                  │
│                                                              │
└──────────────────┬──────────────────────────────────────────┘
                   │ SupervisorNodeBridge trait
                   │ (abstraction boundary)
                   ↓
┌─────────────────────────────────────────────────────────────┐
│ Layer 2: ComponentActor (WASM Lifecycle)                    │
│                                                              │
│  HealthMonitor                                              │
│  ├─ evaluate_health()                                       │
│  ├─ should_check_health()                                   │
│  └─ record_check_result()                                   │
│                                                              │
│  ComponentActor                                             │
│  ├─ Calls health_check() periodically                       │
│  ├─ Evaluates health with HealthMonitor                     │
│  └─ Requests restart via bridge if unhealthy               │
│                                                              │
└──────────────────┬──────────────────────────────────────────┘
                   │ SupervisorNodeBridge trait
                   │ (execution request)
                   ↓
┌─────────────────────────────────────────────────────────────┐
│ Layer 3: SupervisorNode (airssys-rt Runtime)               │
│                                                              │
│  SupervisorNodeWrapper                                      │
│  ├─ restart_child():                                        │
│  │  1. window_limiter.check_can_restart()                   │
│  │  2. backoff.calculate_delay()                            │
│  │  3. restart_tracker.record_restart()                     │
│  │  4. sleep(delay)                                         │
│  │  5. supervisor.restart_child()                           │
│  │                                                          │
│  └─ Query methods (stats, history, reset)                   │
│                                                              │
│  SupervisorNode<OneForOne> (airssys-rt)                    │
│  └─ Manages actual restart execution                        │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Verification Checklist

Before marking Task 3.3 complete, verify:

### Code Quality
- [ ] Zero compiler warnings: `cargo clippy --all-targets -- -D warnings`
- [ ] All tests pass: `cargo test --lib --tests`
- [ ] 100% rustdoc coverage for public API
- [ ] Code quality assessment: 9.5/10
- [ ] Standards compliance: 100% (§2.1, §4.3, §5.1, §6.1-§6.3)

### Architecture Compliance
- [ ] No Layer 1 → Layer 3 direct imports
- [ ] All cross-layer communication via SupervisorNodeBridge
- [ ] ADR-WASM-018 maintained perfectly
- [ ] Bridge pattern used consistently

### Functionality
- [ ] Exponential backoff working correctly
- [ ] Sliding window limits enforced
- [ ] Restart tracking persists
- [ ] Health monitoring integrated
- [ ] Permanent failure detected

### Testing
- [ ] 20-25 new tests created
- [ ] 504-509 total tests passing
- [ ] Performance tests verify targets
- [ ] Edge cases covered
- [ ] Integration tests validate flow

### Documentation
- [ ] All public APIs documented
- [ ] Examples working and clear
- [ ] Inline comments explain key logic
- [ ] Readme/summary updated

---

## References

### Related Documentation
- **Main Block Plan:** `task-004-block-3-actor-system-integration.md`
- **Task 3.2 Plan:** `wasm-task-004-phase-3-task-3.2-plan.md`
- **Task 3.2 Summary:** `wasm-task-004-phase-3-task-3.2-completion-summary.md`
- **Architecture:** ADR-WASM-018, ADR-WASM-006, ADR-WASM-010

### Code References
- **Bridge Trait:** `airssys-wasm/src/actor/supervisor_bridge.rs`
- **Wrapper Implementation:** `airssys-wasm/src/actor/supervisor_wrapper.rs`
- **Supervisor Config:** `airssys-wasm/src/actor/supervisor_config.rs`
- **Component Supervisor:** `airssys-wasm/src/actor/component_supervisor.rs`
- **Component Actor:** `airssys-wasm/src/actor/component_actor.rs`

---

## Plan Approval & Next Steps

**This plan is READY FOR IMPLEMENTATION.**

**Next Action:** Delegate to @memorybank-implementer with this plan.

**Expected Deliverables:**
- 4 new subsystem modules (ExponentialBackoff, RestartTracker, SlidingWindowLimiter, HealthMonitor)
- 3 modified integration files (SupervisorNodeWrapper, ComponentSupervisor, ComponentActor)
- 1 integration test suite (restart_backoff_integration_tests.rs)
- 54-59 new tests, 504-509 total tests passing
- 9.5/10 code quality, zero warnings
- Full ADR-WASM-018 compliance maintained

**Estimated Implementation Time:** 6-8 hours

---

**Plan Created:** 2025-12-14  
**Version:** 1.0  
**Status:** READY FOR IMPLEMENTATION ✅
