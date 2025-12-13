# WASM-TASK-004 Phase 1 Task 1.4: Health Check Implementation - Action Plan

**Task ID:** WASM-TASK-004-P1-T1.4  
**Parent Task:** WASM-TASK-004 Block 3 - Actor System Integration  
**Phase:** Phase 1 - ComponentActor Foundation (Task 4 of 7)  
**Status:** READY TO START  
**Created:** 2025-12-13  
**Estimated Effort:** 8-10 hours

---

## 1. Executive Summary

### 1.1 Task Purpose

Implement comprehensive health check functionality for WASM components, enabling production-ready health monitoring, readiness probes, and supervisor integration. This task transforms the current health_check() stub into a fully operational system that:

1. Calls the optional _health WASM export and deserializes the result
2. Aggregates health status from multiple factors (ActorState, WASM health, resource usage)
3. Provides both readiness (can serve traffic) and liveness (needs restart) semantics
4. Achieves <50ms health check latency for production monitoring

### 1.2 Context & Prerequisites

**Completed Foundation (Tasks 1.1-1.3):**
- ✅ Task 1.1: ComponentActor struct with HealthStatus enum
- ✅ Task 1.2: Child trait with health_check() stub returning Healthy always
- ✅ Task 1.3: Actor trait with HealthCheck message handler using health export detection
- ✅ WasmExports struct with cached _health: Option<Func>
- ✅ Multicodec module for Borsh/CBOR/JSON deserialization (495 lines, 17 tests)
- ✅ Type conversion module for WASM Val <-> Rust bytes (342 lines, 30 tests)

**Current Stub Implementation:**
```rust
// child_impl.rs:447-458
async fn health_check(&self) -> ChildHealth {
    // TODO(Task 3.3): Implement actual health checking
    // Stub: Always healthy
    ChildHealth::Healthy
}

// actor_impl.rs:340-363
ComponentMessage::HealthCheck => {
    let health = if let Some(runtime) = self.wasm_runtime_mut() {
        if let Some(_health_fn) = &runtime.exports().health {
            // Health export invocation (FUTURE WORK - Phase 3 Task 3.3)
            HealthStatus::Healthy
        } else {
            // No _health export - use state-based health
            match self.state() { /* ... */ }
        }
    } else {
        HealthStatus::Unhealthy { reason: "WASM not loaded".to_string() }
    };
    // ...
}
```

### 1.3 Success Criteria

**Functional:**
- [ ] Child::health_check() calls _health export when available
- [ ] HealthStatus deserialization from WASM (Borsh, CBOR, JSON)
- [ ] Health aggregation from multiple factors (state, WASM, resources)
- [ ] HealthCheck message handler uses real health data
- [ ] Readiness vs liveness probe semantics implemented
- [ ] Graceful timeout handling for slow/hung health checks

**Quality:**
- [ ] 15-20 comprehensive tests covering all health scenarios
- [ ] Zero compiler warnings (cargo check + clippy)
- [ ] 100% rustdoc coverage for new code
- [ ] Integration with existing 341 tests (all passing)
- [ ] Workspace standards compliance (§2.1-§6.3)

**Performance:**
- [ ] <50ms health check latency (P99)
- [ ] <10ms typical health check (P50)
- [ ] Timeout protection for hung health checks
- [ ] Zero-copy deserialization where possible

---

## 2. Technical Approach

### 2.1 Architecture Overview

```text
┌────────────────────────────────────────────────────────────┐
│                ComponentActor::health_check()              │
│                                                            │
│  1. WASM State Check                                       │
│     ├─ is_none() → Unhealthy("WASM not loaded")          │
│     └─ is_some() → Continue to step 2                     │
│                                                            │
│  2. ActorState Check                                       │
│     ├─ Failed → Unhealthy(reason)                         │
│     ├─ Creating/Starting → Degraded("Starting")           │
│     └─ Ready → Continue to step 3                         │
│                                                            │
│  3. _health Export Invocation (if available)               │
│     ├─ Call WASM _health() with timeout (1000ms)          │
│     ├─ Get i32 result                                      │
│     ├─ If i32 > 0: Get memory bytes [ptr..ptr+len]       │
│     ├─ Decode multicodec (Borsh/CBOR/JSON)               │
│     └─ Deserialize HealthStatus from bytes                │
│                                                            │
│  4. Resource Health Check (future: CPU/memory pressure)    │
│     └─ Check fuel usage, memory usage (placeholder)       │
│                                                            │
│  5. Aggregate Health Status                                │
│     ├─ Unhealthy beats all (any unhealthy → Unhealthy)   │
│     ├─ Degraded beats Healthy (any degraded → Degraded)  │
│     └─ Return final ChildHealth                           │
└────────────────────────────────────────────────────────────┘
```

### 2.2 Health Status Mapping

**HealthStatus (WASM) → ChildHealth (airssys-rt):**

```rust
// airssys-wasm/src/actor/component_actor.rs:749-764
pub enum HealthStatus {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
}

// airssys-rt (from airssys-rt dependency)
pub enum ChildHealth {
    Healthy,
    Degraded { reason: String },
    Failed { reason: String },  // Maps from Unhealthy
}
```

**Mapping Logic:**
- `HealthStatus::Healthy` → `ChildHealth::Healthy`
- `HealthStatus::Degraded { reason }` → `ChildHealth::Degraded { reason }`
- `HealthStatus::Unhealthy { reason }` → `ChildHealth::Failed { reason }`

### 2.3 _health Export Signature

**WIT Interface (from airssys:core/component-lifecycle.wit):**
```wit
// Optional health check export
_health: func() -> list<u8>
```

**Expected Return Format:**
- **Multicodec prefix** (1-4 bytes): Borsh (0x701), CBOR (0x51), JSON (0x0200)
- **Payload** (N bytes): Serialized HealthStatus
  - Borsh: `{ variant: u8, reason?: String }` (~10-100 bytes)
  - CBOR: `{ "status": "healthy|degraded|unhealthy", "reason": "..." }` (~20-150 bytes)
  - JSON: `{"status":"healthy","reason":"..."}` (~30-200 bytes)

**Example WASM Return (JSON):**
```json
[0x02, 0x00, 0x7b, 0x22, 0x73, 0x74, 0x61, 0x74, 0x75, 0x73, ...]
 └─────┬─────┘ └──────────────────┬────────────────────────┘
   JSON prefix       {"status":"healthy"}
```

### 2.4 Timeout Strategy

**Problem:** Health checks can hang if WASM component is deadlocked or in infinite loop.

**Solution:** Use tokio::time::timeout() wrapper (proven in Task 1.2 Child::stop()).

```rust
async fn health_check(&self) -> ChildHealth {
    const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_millis(1000);
    
    match tokio::time::timeout(HEALTH_CHECK_TIMEOUT, self.health_check_inner()).await {
        Ok(health) => health,
        Err(_timeout) => {
            warn!(component_id = %self.component_id(), "Health check timed out");
            ChildHealth::Degraded {
                reason: "Health check timeout (>1s)".to_string(),
            }
        }
    }
}
```

### 2.5 Readiness vs Liveness Semantics

**Readiness Probe** (Can component serve traffic?):
- `Creating` → Not ready (return Degraded)
- `Starting` → Not ready (return Degraded)
- `Ready` → Check _health export (if available)
- `Failed` → Not ready (return Failed)

**Liveness Probe** (Should component be restarted?):
- `Failed` → Restart (return Failed)
- `Unhealthy` (from _health) → Consider restart
- `Degraded` → Keep running (may self-heal)
- `Healthy` → No action

**Implementation:** Single health_check() method serves both purposes. Supervisor decides action based on ChildHealth return value.

---

## 3. Implementation Plan

### 3.1 Phase 1: Core Health Check Logic (3-4 hours)

#### Step 1.1: Implement health_check_inner() (1.5 hours)

**File:** `airssys-wasm/src/actor/child_impl.rs`

**Tasks:**
1. Create private `health_check_inner()` method with full logic
2. Check WASM runtime loaded state
3. Evaluate ActorState for immediate health determination
4. Call _health export if available (with error handling)
5. Deserialize HealthStatus from WASM response
6. Map HealthStatus → ChildHealth

**Code Pattern:**
```rust
impl ComponentActor {
    /// Inner health check implementation without timeout protection.
    async fn health_check_inner(&self) -> ChildHealth {
        // 1. Check if WASM loaded
        let runtime = match self.wasm_runtime() {
            Some(rt) => rt,
            None => {
                return ChildHealth::Failed {
                    reason: "WASM runtime not loaded".to_string(),
                };
            }
        };
        
        // 2. Check ActorState
        match self.state() {
            ActorState::Failed(reason) => {
                return ChildHealth::Failed {
                    reason: reason.clone(),
                };
            }
            ActorState::Creating | ActorState::Starting => {
                return ChildHealth::Degraded {
                    reason: "Component starting".to_string(),
                };
            }
            ActorState::Stopping => {
                return ChildHealth::Degraded {
                    reason: "Component stopping".to_string(),
                };
            }
            ActorState::Ready => {
                // Continue to _health export check
            }
            _ => {
                return ChildHealth::Degraded {
                    reason: format!("Component in state: {:?}", self.state()),
                };
            }
        }
        
        // 3. Call _health export if available
        let health_status = match &runtime.exports().health {
            Some(health_fn) => {
                match self.call_health_export(health_fn, runtime).await {
                    Ok(status) => status,
                    Err(e) => {
                        warn!(
                            component_id = %self.component_id(),
                            error = %e,
                            "_health export call failed"
                        );
                        // Fall back to state-based health
                        HealthStatus::Healthy
                    }
                }
            }
            None => {
                // No _health export - component is healthy if Ready
                HealthStatus::Healthy
            }
        };
        
        // 4. Map HealthStatus → ChildHealth
        match health_status {
            HealthStatus::Healthy => ChildHealth::Healthy,
            HealthStatus::Degraded { reason } => ChildHealth::Degraded { reason },
            HealthStatus::Unhealthy { reason } => ChildHealth::Failed { reason },
        }
    }
}
```

#### Step 1.2: Implement call_health_export() (2 hours)

**File:** `airssys-wasm/src/actor/child_impl.rs`

**Tasks:**
1. Call _health WASM export with call_async()
2. Extract i32 result (ptr to health bytes)
3. Read bytes from WASM linear memory
4. Decode multicodec prefix
5. Deserialize HealthStatus from payload
6. Handle all error cases gracefully

**Code Pattern:**
```rust
impl ComponentActor {
    /// Call _health WASM export and deserialize result.
    ///
    /// # Protocol
    ///
    /// 1. Call _health() → i32 (pointer to health bytes)
    /// 2. If ptr > 0: Read bytes from linear memory
    /// 3. Decode multicodec prefix (Borsh/CBOR/JSON)
    /// 4. Deserialize HealthStatus from payload
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - WASM call traps
    /// - Invalid pointer returned
    /// - Memory read fails
    /// - Multicodec decoding fails
    /// - Deserialization fails
    async fn call_health_export(
        &self,
        health_fn: &wasmtime::Func,
        runtime: &WasmRuntime,
    ) -> Result<HealthStatus, WasmError> {
        use crate::actor::multicodec::{decode_multicodec, Codec};
        
        // 1. Call _health() → i32
        let mut results = vec![wasmtime::Val::I32(0)];
        health_fn
            .call_async(runtime.store_mut(), &[], &mut results)
            .await
            .map_err(|e| WasmError::HealthCheckFailed {
                component_id: self.component_id().clone(),
                reason: format!("_health call trapped: {}", e),
            })?;
        
        // 2. Extract pointer
        let ptr = match results.get(0) {
            Some(wasmtime::Val::I32(p)) if *p > 0 => *p as usize,
            Some(wasmtime::Val::I32(0)) | Some(wasmtime::Val::I32(-1)) => {
                // No health data returned → assume healthy
                return Ok(HealthStatus::Healthy);
            }
            _ => {
                return Err(WasmError::HealthCheckFailed {
                    component_id: self.component_id().clone(),
                    reason: "Invalid _health return value".to_string(),
                });
            }
        };
        
        // 3. Read bytes from linear memory
        let memory = runtime.instance()
            .get_memory(runtime.store(), "memory")
            .ok_or_else(|| WasmError::HealthCheckFailed {
                component_id: self.component_id().clone(),
                reason: "WASM linear memory not found".to_string(),
            })?;
        
        let data = memory.data(runtime.store());
        let health_bytes = data.get(ptr..ptr + 256)  // Max 256 bytes for health status
            .ok_or_else(|| WasmError::HealthCheckFailed {
                component_id: self.component_id().clone(),
                reason: format!("Invalid memory pointer: {}", ptr),
            })?;
        
        // 4. Decode multicodec
        let payload = decode_multicodec(health_bytes)
            .map_err(|e| WasmError::HealthCheckFailed {
                component_id: self.component_id().clone(),
                reason: format!("Multicodec decoding failed: {}", e),
            })?;
        
        // 5. Deserialize HealthStatus
        // Try Borsh first (most common), then CBOR, then JSON
        if let Ok(status) = borsh::from_slice::<HealthStatus>(&payload) {
            return Ok(status);
        }
        if let Ok(status) = serde_cbor::from_slice::<HealthStatus>(&payload) {
            return Ok(status);
        }
        if let Ok(status) = serde_json::from_slice::<HealthStatus>(&payload) {
            return Ok(status);
        }
        
        Err(WasmError::HealthCheckFailed {
            component_id: self.component_id().clone(),
            reason: "Failed to deserialize HealthStatus (tried Borsh, CBOR, JSON)".to_string(),
        })
    }
}
```

#### Step 1.3: Add timeout wrapper (0.5 hours)

**File:** `airssys-wasm/src/actor/child_impl.rs`

**Tasks:**
1. Wrap health_check_inner() with tokio::time::timeout()
2. Return Degraded on timeout
3. Add tracing for timeout events

**Code Pattern:**
```rust
#[async_trait]
impl Child for ComponentActor {
    async fn health_check(&self) -> ChildHealth {
        const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_millis(1000);
        
        match tokio::time::timeout(HEALTH_CHECK_TIMEOUT, self.health_check_inner()).await {
            Ok(health) => health,
            Err(_timeout) => {
                warn!(
                    component_id = %self.component_id(),
                    "Health check timed out after {}ms",
                    HEALTH_CHECK_TIMEOUT.as_millis()
                );
                ChildHealth::Degraded {
                    reason: format!("Health check timeout (>{}ms)", HEALTH_CHECK_TIMEOUT.as_millis()),
                }
            }
        }
    }
}
```

### 3.2 Phase 2: Update HealthCheck Message Handler (1 hour)

#### Step 2.1: Integrate real health_check() in actor_impl.rs

**File:** `airssys-wasm/src/actor/actor_impl.rs`

**Tasks:**
1. Replace stub HealthCheck handler with call to Child::health_check()
2. Map ChildHealth → HealthStatus for response message
3. Update tracing to log actual health results

**Code Pattern:**
```rust
ComponentMessage::HealthCheck => {
    let component_id_str = self.component_id().as_str().to_string();
    
    trace!(
        component_id = %component_id_str,
        "Processing HealthCheck message"
    );

    // Call Child::health_check() (now fully implemented)
    let child_health = Child::health_check(self).await;
    
    // Map ChildHealth → HealthStatus for message response
    let health_status = match child_health {
        ChildHealth::Healthy => HealthStatus::Healthy,
        ChildHealth::Degraded { reason } => HealthStatus::Degraded { reason },
        ChildHealth::Failed { reason } => HealthStatus::Unhealthy { reason },
    };
    
    info!(
        component_id = %component_id_str,
        health = ?health_status,
        "Health check complete"
    );

    // Reply with health status (Phase 2 Task 2.3 - ActorContext reply)
    // ctx.reply(ComponentMessage::HealthStatus(health_status)).await.ok();
    
    Ok(())
}
```

### 3.3 Phase 3: Serde Implementation for HealthStatus (1 hour)

#### Step 3.1: Add Serialize/Deserialize to HealthStatus

**File:** `airssys-wasm/src/actor/component_actor.rs`

**Tasks:**
1. Add `#[derive(Serialize, Deserialize)]` to HealthStatus enum
2. Add serde attributes for JSON/CBOR friendly format
3. Add borsh support for compact binary serialization

**Code Pattern:**
```rust
/// Component health status for monitoring and supervision.
///
/// HealthStatus represents the operational state of a component, used by both
/// internal health checks and external monitoring systems. This enum supports
/// serialization via Borsh (binary), CBOR (binary), and JSON (text).
///
/// # Serialization Formats
///
/// **Borsh (Recommended):**
/// ```text
/// Healthy:    [0x00]
/// Degraded:   [0x01, len_u32, reason_bytes...]
/// Unhealthy:  [0x02, len_u32, reason_bytes...]
/// ```
///
/// **JSON:**
/// ```json
/// { "status": "healthy" }
/// { "status": "degraded", "reason": "High latency" }
/// { "status": "unhealthy", "reason": "Database unreachable" }
/// ```
///
/// **CBOR:** Binary equivalent of JSON structure
///
/// # Example
///
/// ```rust
/// use airssys_wasm::actor::HealthStatus;
/// use serde_json;
///
/// let status = HealthStatus::Degraded {
///     reason: "High memory usage".to_string(),
/// };
///
/// // JSON serialization
/// let json = serde_json::to_string(&status)?;
/// assert_eq!(json, r#"{"status":"degraded","reason":"High memory usage"}"#);
///
/// // Deserialization
/// let parsed: HealthStatus = serde_json::from_str(&json)?;
/// assert!(matches!(parsed, HealthStatus::Degraded { .. }));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", content = "reason", rename_all = "lowercase")]
pub enum HealthStatus {
    /// Component operating normally
    #[serde(rename = "healthy")]
    Healthy,

    /// Component operational but experiencing issues
    #[serde(rename = "degraded")]
    Degraded {
        /// Reason for degraded status
        reason: String,
    },

    /// Component failed or non-functional
    #[serde(rename = "unhealthy")]
    Unhealthy {
        /// Reason for unhealthy status
        reason: String,
    },
}

// Borsh serialization for compact binary format
impl borsh::BorshSerialize for HealthStatus {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            HealthStatus::Healthy => 0u8.serialize(writer),
            HealthStatus::Degraded { reason } => {
                1u8.serialize(writer)?;
                reason.serialize(writer)
            }
            HealthStatus::Unhealthy { reason } => {
                2u8.serialize(writer)?;
                reason.serialize(writer)
            }
        }
    }
}

impl borsh::BorshDeserialize for HealthStatus {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let variant = u8::deserialize(buf)?;
        match variant {
            0 => Ok(HealthStatus::Healthy),
            1 => Ok(HealthStatus::Degraded {
                reason: String::deserialize(buf)?,
            }),
            2 => Ok(HealthStatus::Unhealthy {
                reason: String::deserialize(buf)?,
            }),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid HealthStatus variant: {}", variant),
            )),
        }
    }
}
```

### 3.4 Phase 4: Comprehensive Testing (3-4 hours)

#### Step 4.1: Unit Tests for health_check_inner() (1.5 hours)

**File:** `airssys-wasm/src/actor/child_impl.rs` (tests module)

**Test Coverage:**
1. ✅ Health check when WASM not loaded → Unhealthy
2. ✅ Health check in Creating state → Degraded
3. ✅ Health check in Starting state → Degraded
4. ✅ Health check in Failed state → Unhealthy
5. ✅ Health check in Ready state with no _health export → Healthy
6. ✅ Health check with _health returning Healthy → Healthy
7. ✅ Health check with _health returning Degraded → Degraded
8. ✅ Health check with _health returning Unhealthy → Unhealthy
9. ✅ Health check with _health trap → Degraded (fallback)
10. ✅ Health check timeout → Degraded

**Test Template:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_health_check_wasm_not_loaded() {
        let actor = create_test_actor();
        // WASM not loaded (wasm_runtime is None)
        
        let health = actor.health_check().await;
        
        assert!(matches!(health, ChildHealth::Failed { .. }));
        if let ChildHealth::Failed { reason } = health {
            assert!(reason.contains("WASM runtime not loaded"));
        }
    }
    
    #[tokio::test]
    async fn test_health_check_creating_state() {
        let actor = create_test_actor();
        assert_eq!(*actor.state(), ActorState::Creating);
        
        let health = actor.health_check().await;
        
        assert!(matches!(health, ChildHealth::Degraded { .. }));
    }
    
    #[tokio::test]
    async fn test_health_check_ready_no_export() {
        let mut actor = create_test_actor();
        actor.start().await.unwrap();  // Loads WASM, transitions to Ready
        
        // Assume test WASM has no _health export
        let health = actor.health_check().await;
        
        assert_eq!(health, ChildHealth::Healthy);
    }
    
    // ... 15-20 total tests
}
```

#### Step 4.2: Integration Tests for HealthStatus Serialization (1 hour)

**File:** `airssys-wasm/tests/health_status_serialization_tests.rs`

**Test Coverage:**
1. ✅ HealthStatus::Healthy → Borsh → HealthStatus::Healthy
2. ✅ HealthStatus::Degraded → JSON → HealthStatus::Degraded
3. ✅ HealthStatus::Unhealthy → CBOR → HealthStatus::Unhealthy
4. ✅ Multicodec round-trip (encode Borsh + decode Borsh)
5. ✅ Cross-format compatibility (encode JSON, decode as serde_json)

**Test Template:**
```rust
use airssys_wasm::actor::HealthStatus;
use airssys_wasm::actor::multicodec::{encode_multicodec, decode_multicodec, Codec};

#[test]
fn test_health_status_borsh_round_trip() {
    let original = HealthStatus::Degraded {
        reason: "High latency".to_string(),
    };
    
    // Serialize
    let bytes = borsh::to_vec(&original).unwrap();
    
    // Deserialize
    let decoded: HealthStatus = borsh::from_slice(&bytes).unwrap();
    
    assert_eq!(decoded, original);
}

#[test]
fn test_health_status_json_format() {
    let status = HealthStatus::Unhealthy {
        reason: "Database unreachable".to_string(),
    };
    
    let json = serde_json::to_string(&status).unwrap();
    
    assert_eq!(
        json,
        r#"{"status":"unhealthy","reason":"Database unreachable"}"#
    );
}

#[test]
fn test_health_status_multicodec_round_trip() {
    let original = HealthStatus::Healthy;
    
    // Encode with Borsh multicodec
    let bytes = borsh::to_vec(&original).unwrap();
    let encoded = encode_multicodec(Codec::Borsh, &bytes).unwrap();
    
    // Decode multicodec
    let payload = decode_multicodec(&encoded).unwrap();
    
    // Deserialize
    let decoded: HealthStatus = borsh::from_slice(&payload).unwrap();
    
    assert_eq!(decoded, original);
}
```

#### Step 4.3: Performance Benchmarks (0.5 hours)

**File:** `airssys-wasm/benches/health_check_benchmarks.rs`

**Benchmarks:**
1. Health check latency (no _health export)
2. Health check latency (with _health export)
3. HealthStatus serialization overhead (Borsh/CBOR/JSON)
4. Multicodec encoding/decoding overhead

**Target Metrics:**
- P50 latency: <10ms
- P99 latency: <50ms
- Borsh serialize: <1μs
- Multicodec overhead: <5μs

#### Step 4.4: Documentation Tests (1 hour)

**Tasks:**
1. Add comprehensive rustdoc examples to health_check()
2. Add rustdoc examples to HealthStatus variants
3. Add module-level documentation explaining health check semantics
4. Document readiness vs liveness probe patterns

**Example:**
```rust
/// Check component health status.
///
/// This method implements comprehensive health checking by:
/// 1. Checking if WASM runtime is loaded
/// 2. Evaluating ActorState for immediate failures
/// 3. Calling optional _health WASM export
/// 4. Aggregating health from multiple factors
///
/// # Health Semantics
///
/// **Readiness Probe:** Can component serve traffic?
/// - `Creating/Starting` → Degraded (not ready yet)
/// - `Ready + Healthy` → Healthy (ready to serve)
/// - `Failed` → Failed (restart needed)
///
/// **Liveness Probe:** Should component be restarted?
/// - `Failed` → Restart required
/// - `Unhealthy` → Consider restart
/// - `Degraded` → Keep running (may self-heal)
///
/// # Performance
///
/// - **Without _health export:** <1ms (state check only)
/// - **With _health export:** <10ms typical, <50ms P99
/// - **Timeout protection:** 1000ms (returns Degraded on timeout)
///
/// # Example
///
/// ```rust,ignore
/// use airssys_wasm::actor::ComponentActor;
/// use airssys_rt::supervisor::{Child, ChildHealth};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut actor = create_component_actor().await?;
///     actor.start().await?;
///     
///     // Periodic health check (e.g., every 30 seconds)
///     let health = actor.health_check().await;
///     
///     match health {
///         ChildHealth::Healthy => {
///             println!("Component healthy");
///         }
///         ChildHealth::Degraded { reason } => {
///             println!("Component degraded: {}", reason);
///             // May self-heal, keep monitoring
///         }
///         ChildHealth::Failed { reason } => {
///             println!("Component failed: {}", reason);
///             // Supervisor will restart
///         }
///     }
///     
///     Ok(())
/// }
/// ```
async fn health_check(&self) -> ChildHealth { /* ... */ }
```

---

## 4. Integration Points

### 4.1 Task 1.1 (ComponentActor Foundation)

**Dependencies:**
- ✅ `HealthStatus` enum (component_actor.rs:749-764)
- ✅ `ActorState` enum for state-based health checks
- ✅ `ComponentActor` struct fields (component_id, state, wasm_runtime)

**Integration:**
- Use HealthStatus enum for WASM health representation
- Check ActorState to determine immediate health failures
- Access WasmRuntime for _health export invocation

### 4.2 Task 1.2 (Child Trait WASM Lifecycle)

**Dependencies:**
- ✅ `Child::health_check()` stub (child_impl.rs:447-458)
- ✅ `WasmRuntime` struct with Wasmtime Store/Instance
- ✅ `WasmExports` struct with _health: Option<Func> cache
- ✅ Timeout pattern from Child::stop() (proven to work)

**Integration:**
- Replace health_check() stub with full implementation
- Use WasmRuntime to access _health export and linear memory
- Apply same timeout pattern as Child::stop()

### 4.3 Task 1.3 (Actor Trait Message Handling)

**Dependencies:**
- ✅ `ComponentMessage::HealthCheck` handler (actor_impl.rs:340-363)
- ✅ Multicodec module (decode_multicodec, Codec enum)
- ✅ Type conversion patterns (WASM Val <-> Rust bytes)

**Integration:**
- Update HealthCheck handler to use Child::health_check()
- Reuse multicodec deserialization for _health response
- Apply same error handling patterns as Invoke handler

### 4.4 Block 1 (WASM Runtime Layer)

**Dependencies:**
- ✅ Wasmtime Func::call_async() for _health invocation
- ✅ Memory::data() for reading health bytes from linear memory
- ✅ Trap handling from execute_function()

**Integration:**
- Use call_async() to invoke _health export
- Read health bytes from WASM linear memory
- Handle traps gracefully (fall back to state-based health)

### 4.5 airssys-rt (Actor System)

**Dependencies:**
- ✅ `Child` trait (from airssys-rt dependency)
- ✅ `ChildHealth` enum (Healthy, Degraded, Failed)

**Integration:**
- Implement Child::health_check() → ChildHealth
- Map HealthStatus (WASM) → ChildHealth (airssys-rt)
- Enable SupervisorNode to use health checks for restart decisions

---

## 5. Error Handling Strategy

### 5.1 Error Categories

**1. WASM Not Loaded:**
- **Scenario:** health_check() called before Child::start()
- **Response:** `ChildHealth::Failed { reason: "WASM runtime not loaded" }`
- **Recovery:** Call start() first

**2. _health Export Trap:**
- **Scenario:** _health export panics or traps
- **Response:** Log warning, fall back to state-based health
- **Recovery:** Return Healthy if state is Ready, Degraded otherwise

**3. Invalid _health Return:**
- **Scenario:** _health returns invalid pointer or malformed data
- **Response:** Log error, fall back to state-based health
- **Recovery:** Same as trap handling

**4. Multicodec Decode Failure:**
- **Scenario:** _health returns bytes with invalid multicodec prefix
- **Response:** Log error, fall back to state-based health
- **Recovery:** Component may need redeployment with correct serialization

**5. HealthStatus Deserialization Failure:**
- **Scenario:** Valid multicodec but invalid HealthStatus structure
- **Response:** Log error, try all formats (Borsh → CBOR → JSON)
- **Recovery:** If all fail, fall back to state-based health

**6. Health Check Timeout:**
- **Scenario:** _health export takes >1000ms (deadlock, infinite loop)
- **Response:** Return `ChildHealth::Degraded { reason: "Health check timeout" }`
- **Recovery:** Supervisor may restart if timeout persists

### 5.2 Error Handling Code Patterns

```rust
// Pattern 1: Graceful fallback on _health export error
match self.call_health_export(health_fn, runtime).await {
    Ok(status) => status,
    Err(e) => {
        warn!(
            component_id = %self.component_id(),
            error = %e,
            "_health export failed, falling back to state-based health"
        );
        // Fall back to Healthy if Ready, Degraded otherwise
        if matches!(self.state(), ActorState::Ready) {
            HealthStatus::Healthy
        } else {
            HealthStatus::Degraded {
                reason: format!("State: {:?}", self.state()),
            }
        }
    }
}

// Pattern 2: Try all deserialization formats
if let Ok(status) = borsh::from_slice::<HealthStatus>(&payload) {
    return Ok(status);
}
if let Ok(status) = serde_cbor::from_slice::<HealthStatus>(&payload) {
    return Ok(status);
}
if let Ok(status) = serde_json::from_slice::<HealthStatus>(&payload) {
    return Ok(status);
}
// All failed
Err(WasmError::HealthCheckFailed {
    component_id: self.component_id().clone(),
    reason: "Failed to deserialize HealthStatus (tried Borsh, CBOR, JSON)".to_string(),
})
```

---

## 6. Performance Optimization

### 6.1 Target Metrics

| Operation | Target P50 | Target P99 | Strategy |
|-----------|-----------|-----------|----------|
| Health check (no _health) | <1ms | <5ms | State check only |
| Health check (with _health) | <10ms | <50ms | WASM call + deserialize |
| _health WASM call | <5ms | <20ms | call_async() |
| Multicodec decode | <5μs | <20μs | Single-pass parsing |
| HealthStatus deserialize | <10μs | <50μs | Zero-copy where possible |

### 6.2 Optimization Strategies

**1. Cache _health Export:**
- ✅ Already cached in WasmExports::health (Task 1.1)
- Avoid repeated export lookup overhead

**2. Zero-Copy Deserialization:**
- Use Borsh for compact binary format (no intermediate JSON parsing)
- Read directly from WASM linear memory (no extra allocations)

**3. Early Exit Patterns:**
- Check WASM loaded state first (avoids unnecessary work)
- Return immediately on ActorState::Failed (no need to call _health)

**4. Timeout Protection:**
- Prevent hung health checks from blocking supervisor
- 1000ms timeout ensures supervisor can make progress

**5. Multicodec Single-Pass:**
- Decode multicodec prefix in one pass (no buffering)
- Reuse existing multicodec module from Task 1.3

### 6.3 Memory Efficiency

**Health Check Allocations:**
- HealthStatus: 24-56 bytes (enum + String)
- Multicodec buffer: 256 bytes (max health status size)
- Total: ~280 bytes per health check

**No Long-Lived Allocations:**
- All buffers freed after health check completes
- No caching of health results (always fresh check)

---

## 7. Testing Strategy

### 7.1 Test Coverage Matrix

| Scenario | Unit Test | Integration Test | Performance Bench |
|----------|-----------|------------------|-------------------|
| WASM not loaded | ✅ | ✅ | N/A |
| Creating state | ✅ | ✅ | N/A |
| Starting state | ✅ | ✅ | N/A |
| Ready state (no _health) | ✅ | ✅ | ✅ |
| Ready state (with _health) | ✅ | ✅ | ✅ |
| _health returns Healthy | ✅ | ✅ | N/A |
| _health returns Degraded | ✅ | ✅ | N/A |
| _health returns Unhealthy | ✅ | ✅ | N/A |
| _health trap | ✅ | ✅ | N/A |
| _health timeout | ✅ | ✅ | N/A |
| Invalid _health pointer | ✅ | ✅ | N/A |
| Multicodec decode error | ✅ | ✅ | N/A |
| Deserialize error | ✅ | ✅ | N/A |
| Borsh round-trip | N/A | ✅ | ✅ |
| JSON round-trip | N/A | ✅ | ✅ |
| CBOR round-trip | N/A | ✅ | ✅ |

**Total Tests:** 15-20 comprehensive tests

### 7.2 Test Fixtures

**Test WASM Components:**
1. `health_healthy.wasm` - Returns Healthy always
2. `health_degraded.wasm` - Returns Degraded with reason
3. `health_unhealthy.wasm` - Returns Unhealthy with reason
4. `health_trap.wasm` - _health export panics
5. `health_timeout.wasm` - _health hangs (infinite loop)
6. `no_health_export.wasm` - No _health export

**Build Script:**
```bash
# Compile test WASM components with cargo-component
cd tests/fixtures/health_components
cargo component build --release

# Output: target/wasm32-wasip1/release/*.wasm
```

### 7.3 CI/CD Integration

**GitHub Actions Workflow:**
```yaml
- name: Run health check tests
  run: |
    cargo test --package airssys-wasm --test health_*
    cargo test --package airssys-wasm --lib health_check

- name: Run health check benchmarks
  run: |
    cargo bench --package airssys-wasm health_check
    
- name: Check health check performance
  run: |
    # Fail if P99 > 50ms
    cargo bench --package airssys-wasm health_check -- --save-baseline health_p99
```

---

## 8. Documentation Requirements

### 8.1 Rustdoc Coverage

**Files to Document:**
- [ ] `child_impl.rs::health_check()` - Main entry point
- [ ] `child_impl.rs::health_check_inner()` - Core logic
- [ ] `child_impl.rs::call_health_export()` - WASM invocation
- [ ] `component_actor.rs::HealthStatus` - Enum variants
- [ ] Module-level docs explaining health check architecture

**Documentation Standards:**
- Explain readiness vs liveness probe semantics
- Document all error cases and recovery strategies
- Provide runnable examples (with `ignore` for WASM dependencies)
- Cross-reference ADRs (WASM-003, WASM-006, RT-004)

### 8.2 Knowledge Base Updates

**Files to Create/Update:**
- [ ] Update `KNOWLEDGE-WASM-016` with Task 1.4 completion notes
- [ ] Document health check patterns in `docs/guides/health-monitoring.md`
- [ ] Add health check examples to `examples/health_check_patterns.rs`

### 8.3 Architecture Decision Records

**Decisions to Document:**
- **Timeout Duration:** 1000ms (rationale: balance between responsiveness and false positives)
- **Fallback Strategy:** State-based health on _health failure (rationale: fail-safe defaults)
- **Multicodec Support:** Borsh/CBOR/JSON (rationale: interoperability with different WASM languages)

---

## 9. Success Criteria

### 9.1 Functional Completeness

- [ ] Child::health_check() fully implemented (replaces stub)
- [ ] _health WASM export invoked correctly
- [ ] HealthStatus deserialization works (Borsh/CBOR/JSON)
- [ ] Health aggregation logic correct (state + WASM + resources)
- [ ] HealthCheck message handler uses real health data
- [ ] Timeout protection prevents hung health checks
- [ ] All error cases handled gracefully with fallbacks

### 9.2 Quality Standards

- [ ] 15-20 comprehensive tests passing
- [ ] Zero compiler warnings (cargo check + clippy)
- [ ] 100% rustdoc coverage for new code
- [ ] Integration with 341 existing tests (all passing)
- [ ] Workspace standards compliance (§2.1-§6.3)
- [ ] Microsoft Rust Guidelines compliance

### 9.3 Performance Targets

- [ ] P50 latency <10ms (with _health export)
- [ ] P99 latency <50ms (with _health export)
- [ ] <1ms state-only health check (no _health export)
- [ ] Timeout protection at 1000ms
- [ ] Zero memory leaks in stress tests

### 9.4 Documentation Standards

- [ ] Complete rustdoc for all new functions
- [ ] Module-level health check architecture docs
- [ ] Readiness vs liveness probe patterns documented
- [ ] Knowledge base updated (KNOWLEDGE-WASM-016)
- [ ] Examples added (health_check_patterns.rs)

---

## 10. Risk Assessment

### 10.1 Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| _health export compatibility issues | Medium | High | Support 3 formats (Borsh/CBOR/JSON) |
| Health check timeout too short | Low | Medium | Make timeout configurable (future) |
| WASM memory read failures | Low | High | Validate pointers, handle errors gracefully |
| Deserialization performance | Low | Medium | Use Borsh for efficiency, benchmark |
| Test WASM component complexity | Medium | Low | Use simple health_*.wasm fixtures |

### 10.2 Integration Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| airssys-rt ChildHealth mismatch | Low | High | Test mapping thoroughly |
| Supervisor restart logic confusion | Medium | High | Document readiness vs liveness |
| Block 6 storage dependency | High | Medium | Use test stubs for now |
| Performance target miss | Low | Medium | Benchmark early, optimize |

### 10.3 Schedule Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Serde implementation complexity | Medium | Low | Reuse patterns from Task 1.3 |
| Test fixture creation time | Medium | Low | Start with simple fixtures first |
| Documentation time underestimate | Medium | Low | Write docs alongside code |
| Integration testing delays | Low | Medium | Test incrementally |

---

## 11. Dependencies & Blockers

### 11.1 Internal Dependencies (Resolved)

- ✅ Task 1.1: ComponentActor struct, HealthStatus enum
- ✅ Task 1.2: Child trait, WasmRuntime, WasmExports
- ✅ Task 1.3: Multicodec module, type conversion patterns
- ✅ Block 1: Wasmtime Func::call_async(), Memory::data()

### 11.2 External Dependencies

- ✅ `airssys-rt` crate: Child trait, ChildHealth enum
- ✅ `wasmtime` crate: Func, Store, Memory, call_async()
- ✅ `borsh` crate: Borsh serialization
- ✅ `serde_json` crate: JSON serialization
- ✅ `serde_cbor` crate: CBOR serialization
- ✅ `tokio` crate: timeout() wrapper

### 11.3 Known Blockers

**None.** All prerequisites are complete. Task 1.4 is ready to start immediately.

---

## 12. Approval & Sign-Off

### 12.1 Plan Review

**Reviewers:**
- [ ] Technical Lead: Architecture review
- [ ] Performance Engineer: Performance target review
- [ ] QA Lead: Test coverage review

### 12.2 Approval Criteria

- [ ] All integration points validated
- [ ] Performance targets achievable
- [ ] Test coverage sufficient (15-20 tests)
- [ ] Documentation plan complete
- [ ] Risk mitigation strategies identified

### 12.3 Approval Status

**Status:** AWAITING USER APPROVAL  
**Date:** 2025-12-13  
**Approver:** User (project owner)

---

## 13. References

### 13.1 Task Documents

- **WASM-TASK-004:** Block 3 - Actor System Integration (parent task)
- **Task 1.1 Completion:** ComponentActor foundation (1,334 lines, 43 tests)
- **Task 1.2 Completion:** Child trait WASM lifecycle (588 lines, 50 tests)
- **Task 1.3 Completion:** Actor trait message handling (1,500 lines, 58 tests)

### 13.2 Knowledge Base

- **KNOWLEDGE-WASM-016:** Actor System Integration Implementation Guide (lines 669+)
- **KNOWLEDGE-WASM-005:** Inter-Component Messaging Architecture
- **KNOWLEDGE-RT-013:** Actor Performance Benchmarking Results

### 13.3 Architecture Decision Records

- **ADR-WASM-003:** Component Lifecycle Management
- **ADR-WASM-006:** Component Isolation and Sandboxing (dual trait pattern)
- **ADR-RT-004:** Actor and Child Trait Separation
- **ADR-WASM-001:** Inter-Component Communication Design (multicodec)

### 13.4 External References

- [Kubernetes Health Checks](https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/)
- [Erlang OTP Supervision](https://www.erlang.org/doc/design_principles/sup_princ.html)
- [Borsh Specification](https://borsh.io/)
- [CBOR RFC 8949](https://www.rfc-editor.org/rfc/rfc8949.html)

---

## 14. Appendix: Code Checklist

### 14.1 Implementation Checklist

**Files to Modify:**
- [ ] `airssys-wasm/src/actor/child_impl.rs` (health_check, health_check_inner, call_health_export)
- [ ] `airssys-wasm/src/actor/actor_impl.rs` (HealthCheck handler)
- [ ] `airssys-wasm/src/actor/component_actor.rs` (HealthStatus serde)

**Files to Create:**
- [ ] `airssys-wasm/tests/health_status_serialization_tests.rs`
- [ ] `airssys-wasm/benches/health_check_benchmarks.rs`
- [ ] `airssys-wasm/examples/health_check_patterns.rs`

**Test Fixtures to Create:**
- [ ] `tests/fixtures/health_components/health_healthy.wasm`
- [ ] `tests/fixtures/health_components/health_degraded.wasm`
- [ ] `tests/fixtures/health_components/health_unhealthy.wasm`
- [ ] `tests/fixtures/health_components/health_trap.wasm`
- [ ] `tests/fixtures/health_components/no_health_export.wasm`

### 14.2 Quality Checklist

- [ ] Zero compiler warnings (cargo check)
- [ ] Zero clippy warnings (cargo clippy --all-targets --all-features)
- [ ] All tests passing (cargo test)
- [ ] All doc tests passing (cargo test --doc)
- [ ] Benchmarks executable (cargo bench health_check)
- [ ] Documentation generated (cargo doc --no-deps)

### 14.3 Standards Compliance

- [ ] §2.1: 3-layer import organization
- [ ] §4.3: Module organization (child_impl.rs)
- [ ] §5.1: Workspace dependency versions
- [ ] §6.1: Error handling patterns
- [ ] §6.2: Async/await patterns
- [ ] §6.3: Logging and tracing

---

**END OF ACTION PLAN**

This plan is ready for implementation. Please approve to proceed with Task 1.4.
