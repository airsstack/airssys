# WASM-TASK-004 Phase 2 Task 2.1: ActorSystem Integration + Deferred WASM Invocation - Detailed Plan

**Generated:** 2025-12-13  
**Status:** ready-to-start  
**Estimated Effort:** 24-30 hours (deferred work 12-18h + ActorSystem 8-12h + testing 4-6h)  
**Priority:** 🔥 CRITICAL - BLOCKING PATH for Phase 2+  

## Overview

This document provides a comprehensive implementation plan for **WASM-TASK-004 Phase 2 Task 2.1**, which:
1. **Completes deferred work from Task 1.3** (DEBT-WASM-004 Items #1 and #2)
2. **Implements ActorSystem::spawn() integration** for component instantiation
3. **Establishes component registration and management** infrastructure

**⚠️ CRITICAL**: This task includes MANDATORY deferred work from Task 1.3. See DEBT-WASM-004 for complete context.

---

## Context

### What's Already Done (Task 1.3 COMPLETE ✅)

**Implemented Components** (Task 1.3 - Dec 13, 2025):
1. **Message Routing Infrastructure** (`src/actor/actor_impl.rs`):
   - ✅ Full `handle_message()` implementation for all ComponentMessage variants
   - ✅ Multicodec deserialization (Borsh, CBOR, JSON)
   - ✅ WASM runtime verification and export checking
   - ✅ Error handling with component context
   - ✅ 11 comprehensive tests (all passing, 306 total tests)

2. **Multicodec Support** (`src/core/multicodec.rs`):
   - ✅ Codec enum with Borsh, CBOR, JSON
   - ✅ encode_multicodec/decode_multicodec functions
   - ✅ 19 tests (all passing)

3. **Child Trait Implementation** (Task 1.2 - Nov 30, 2025):
   - ✅ Child::start() with WASM loading
   - ✅ Child::stop() with resource cleanup
   - ✅ WasmRuntime integration
   - ✅ 275 tests passing

**Quality Metrics:**
- Tests: 306 passing (airssys-wasm)
- Warnings: 0 (compiler + clippy)
- Coverage: >90% for actor module
- Code Quality: 9.0/10 (Task 1.3), 9.2/10 (Task 1.2)

### What Needs Implementation (Task 2.1) ⚠️

**Primary Objectives:**
1. **Complete Task 1.3 Deferred Work** (DEBT-WASM-004 Items #1 and #2)
2. **Implement ActorSystem Integration** (spawn, registration, tracking)
3. **Establish Performance Baseline** (>10,000 msg/sec target)

**Scope Definition:**

**PART 1: Deferred Work (12-18 hours) - MANDATORY**
1. **WASM Function Invocation** (8-12h):
   - Type conversion system (Rust types ↔ WASM Val)
   - Function export retrieval and validation
   - Async function call execution
   - Result serialization with multicodec
   - Error handling for WASM traps
   - Remove TODO comments at `actor_impl.rs:190-200`

2. **InterComponent WASM Call** (4-6h):
   - handle-message export invocation
   - Parameter marshalling for inter-component messages
   - Trap propagation to supervisor
   - Remove TODO comments at `actor_impl.rs:236-246`

**PART 2: ActorSystem Integration (8-12 hours)**
1. **ActorSystem::spawn() Integration** (4-6h):
   - Component spawning via ActorSystem (NOT tokio::spawn)
   - ComponentActor registration
   - ActorRef management
   - Spawn time optimization (<5ms target)

2. **Component Instance Management** (4-6h):
   - Component ID to ActorRef mapping
   - Instance registry implementation
   - O(1) lookup performance
   - Lifecycle tracking

**PART 3: Testing & Validation (4-6 hours)**
1. Integration tests for WASM invocation
2. Performance benchmarks (>10,000 msg/sec)
3. DEBT-WASM-004 verification checklist
4. Documentation updates

**OUT OF SCOPE for Task 2.1:**
- Capability enforcement (Block 4 - DEBT-WASM-004 Item #3)
- Health check parsing (Phase 3 Task 3.3 - DEBT-WASM-004 Item #4)
- Component registry integration (Block 6 - DEBT-WASM-004 Item #5)
- SupervisorNode integration (Phase 3)
- MessageBroker integration (Phase 4)

---

## Reference Documentation

**MANDATORY Reading:**
1. **DEBT-WASM-004**: `.memory-bank/sub-projects/airssys-wasm/docs/technical-debt/debt-wasm-004-task-1.3-deferred-implementation.md`
   - **Section 1** (lines 21-82): WASM Function Invocation requirements
   - **Section 2** (lines 85-132): InterComponent WASM Call requirements
   - **Validation Criteria**: Lines 70-78, 124-128

2. **KNOWLEDGE-WASM-016**: `.memory-bank/sub-projects/airssys-wasm/docs/knowledges/knowledge-wasm-016-actor-system-integration-implementation-guide.md`
   - **Phase 2 ActorSystem**: Lines 687-805
   - **Component Spawning**: Lines 691-753
   - **Testing Strategies**: Lines 756-805

3. **Task Specification**: `.memory-bank/sub-projects/airssys-wasm/tasks/task-004-block-3-actor-system-integration.md`
   - **Phase 2 Overview**: Lines 201-265
   - **Task 2.1 Details**: Lines 212-236

**Architecture Decisions**:
- **ADR-WASM-006**: Actor integration dual trait pattern
- **ADR-WASM-001**: Inter-Component Communication Design (multicodec)
- **ADR-RT-004**: Actor and Child Trait Separation

**Code Standards**:
- `.memory-bank/workspace/shared-patterns.md`: §2.1-§6.4
- `.memory-bank/workspace/microsoft-rust-guidelines.md`: M-ERRORS-CANONICAL-STRUCTS, M-LINT-OVERRIDE-EXPECT

---

## Current Code Structure

### Files to Modify

**Primary Implementation Files:**

1. **`airssys-wasm/src/actor/actor_impl.rs`** (444 lines)
   - **Lines 190-200**: WASM Function Invocation (FUTURE WORK marker)
   - **Lines 236-246**: InterComponent WASM Call (FUTURE WORK marker)
   - **Action**: Remove TODO comments, implement deferred work

2. **`airssys-wasm/src/core/mod.rs`** (New exports needed)
   - **Action**: Export type conversion utilities

3. **`airssys-wasm/src/runtime/`** (Integration point)
   - **Files**: `instance.rs`, `execution.rs`, `store.rs`
   - **Action**: Ensure WASM function call APIs are accessible

**New Files to Create:**

4. **`airssys-wasm/src/actor/type_conversion.rs`** (~300 lines)
   - Rust types ↔ WASM Val conversion
   - Parameter marshalling utilities
   - Result extraction helpers

5. **`airssys-wasm/src/actor/component_spawner.rs`** (~400 lines)
   - ActorSystem::spawn() wrapper
   - Component registration logic
   - Instance tracking

6. **`airssys-wasm/tests/actor_invocation_tests.rs`** (~300 lines)
   - WASM function invocation tests
   - InterComponent message flow tests
   - Performance benchmarks

### Current Stub Implementation

**File**: `airssys-wasm/src/actor/actor_impl.rs`

**Line 190-200 (WASM Function Invocation stub):**
```rust
// 3. WASM function invocation (FUTURE WORK - Phase 2 Task 2.1)
// NOTE: Actual WASM function call deferred to Phase 2 (ActorSystem Integration).
// Task 1.3 scope: Message routing + multicodec deserialization (COMPLETE ✅)
// Phase 2 scope: Full WASM invocation with type conversion
//
// Full implementation will require:
// - WASM type conversion system (Val marshalling)
// - Parameter preparation from decoded_args
// - Function call: runtime.instance().get_func().call_async()
// - Result serialization with multicodec
// - Error handling for WASM traps
```

**Line 236-246 (InterComponent Call stub):**
```rust
// 3. Route to WASM handle-message export
if let Some(_handle_fn) = &runtime.exports().handle_message {
    // WASM invocation (FUTURE WORK - Phase 2 Task 2.1)
    // NOTE: Actual handle-message call deferred to Phase 2.
    // Task 1.3 scope: Export verification + routing logic (COMPLETE ✅)
    // Phase 2 scope: Full WASM call with parameter conversion
    //
    // Full implementation will require:
    // let params = prepare_wasm_params(&payload)?;
    // handle_fn.call_async(runtime.store_mut(), &params).await
    //     .map_err(|e| WasmError::execution_failed_with_source(...))?;
}
```

---

## Implementation Plan

## PART 1: Deferred Work Implementation (12-18 hours)

### Step 1.1: Type Conversion System (4-6 hours)

**Objective**: Create Rust ↔ WASM Val conversion utilities

**File**: `airssys-wasm/src/actor/type_conversion.rs` (NEW)

**Implementation**:

```rust
// src/actor/type_conversion.rs

use wasmtime::{Val, ValType, FuncType};
use crate::core::WasmError;

/// Convert decoded multicodec bytes to WASM Val parameters
///
/// # Arguments
/// * `decoded_bytes` - Raw bytes from multicodec deserialization
/// * `func_type` - Function signature from WASM export
///
/// # Returns
/// Vec<Val> ready for WASM function call
///
/// # Errors
/// - Type mismatch between bytes and function signature
/// - Unsupported type conversion
pub fn prepare_wasm_params(
    decoded_bytes: &[u8],
    func_type: &FuncType,
) -> Result<Vec<Val>, WasmError> {
    // Parse decoded_bytes based on expected param types
    let mut params = Vec::new();
    let param_types = func_type.params();
    
    // Strategy: Use serde for structured data, direct conversion for primitives
    match param_types.len() {
        0 => Ok(vec![]),
        1 => {
            // Single parameter - direct conversion
            let val = bytes_to_val(decoded_bytes, param_types.nth(0).unwrap())?;
            Ok(vec![val])
        }
        _ => {
            // Multiple parameters - deserialize as tuple
            deserialize_tuple_params(decoded_bytes, param_types)
        }
    }
}

/// Convert raw bytes to a single WASM Val
fn bytes_to_val(bytes: &[u8], val_type: ValType) -> Result<Val, WasmError> {
    match val_type {
        ValType::I32 => {
            if bytes.len() != 4 {
                return Err(WasmError::type_mismatch("Expected 4 bytes for i32"));
            }
            let value = i32::from_le_bytes(bytes.try_into().unwrap());
            Ok(Val::I32(value))
        }
        ValType::I64 => {
            if bytes.len() != 8 {
                return Err(WasmError::type_mismatch("Expected 8 bytes for i64"));
            }
            let value = i64::from_le_bytes(bytes.try_into().unwrap());
            Ok(Val::I64(value))
        }
        ValType::F32 => {
            if bytes.len() != 4 {
                return Err(WasmError::type_mismatch("Expected 4 bytes for f32"));
            }
            let value = f32::from_le_bytes(bytes.try_into().unwrap());
            Ok(Val::F32(value.to_bits()))
        }
        ValType::F64 => {
            if bytes.len() != 8 {
                return Err(WasmError::type_mismatch("Expected 8 bytes for f64"));
            }
            let value = f64::from_le_bytes(bytes.try_into().unwrap());
            Ok(Val::F64(value.to_bits()))
        }
        ValType::V128 => {
            Err(WasmError::unsupported_type("V128 not supported"))
        }
        ValType::FuncRef | ValType::ExternRef => {
            Err(WasmError::unsupported_type("Reference types require special handling"))
        }
    }
}

/// Deserialize multiple parameters from bytes
fn deserialize_tuple_params(
    bytes: &[u8],
    param_types: impl ExactSizeIterator<Item = ValType>,
) -> Result<Vec<Val>, WasmError> {
    // Use borsh for structured deserialization
    // This assumes the caller encoded multiple params as a borsh-encoded tuple
    
    // For now, return error - this is a complex case
    // that requires schema information
    Err(WasmError::unsupported_type(
        "Multi-parameter functions require schema definition"
    ))
}

/// Convert WASM function results to bytes
///
/// # Arguments
/// * `results` - Vec<Val> from WASM function call
///
/// # Returns
/// Raw bytes ready for multicodec encoding
pub fn extract_wasm_results(results: &[Val]) -> Result<Vec<u8>, WasmError> {
    match results.len() {
        0 => Ok(vec![]),
        1 => val_to_bytes(&results[0]),
        _ => serialize_tuple_results(results),
    }
}

/// Convert single WASM Val to bytes
fn val_to_bytes(val: &Val) -> Result<Vec<u8>, WasmError> {
    match val {
        Val::I32(v) => Ok(v.to_le_bytes().to_vec()),
        Val::I64(v) => Ok(v.to_le_bytes().to_vec()),
        Val::F32(v) => Ok(f32::from_bits(*v).to_le_bytes().to_vec()),
        Val::F64(v) => Ok(f64::from_bits(*v).to_le_bytes().to_vec()),
        Val::V128(_) => Err(WasmError::unsupported_type("V128 not supported")),
        Val::FuncRef(_) | Val::ExternRef(_) => {
            Err(WasmError::unsupported_type("Cannot serialize reference types"))
        }
    }
}

/// Serialize multiple results to bytes
fn serialize_tuple_results(results: &[Val]) -> Result<Vec<u8>, WasmError> {
    // Use borsh to encode tuple
    Err(WasmError::unsupported_type(
        "Multi-value returns require schema definition"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i32_conversion() {
        let bytes = 42i32.to_le_bytes();
        let val = bytes_to_val(&bytes, ValType::I32).unwrap();
        assert_eq!(val.unwrap_i32(), 42);
    }

    #[test]
    fn test_i64_conversion() {
        let bytes = 12345i64.to_le_bytes();
        let val = bytes_to_val(&bytes, ValType::I64).unwrap();
        assert_eq!(val.unwrap_i64(), 12345);
    }

    #[test]
    fn test_f32_conversion() {
        let bytes = 3.14f32.to_le_bytes();
        let val = bytes_to_val(&bytes, ValType::F32).unwrap();
        assert_eq!(f32::from_bits(val.unwrap_f32()), 3.14f32);
    }

    #[test]
    fn test_f64_conversion() {
        let bytes = 2.718f64.to_le_bytes();
        let val = bytes_to_val(&bytes, ValType::F64).unwrap();
        assert_eq!(f64::from_bits(val.unwrap_f64()), 2.718f64);
    }

    #[test]
    fn test_result_extraction() {
        let val = Val::I32(100);
        let bytes = val_to_bytes(&val).unwrap();
        assert_eq!(i32::from_le_bytes(bytes.try_into().unwrap()), 100);
    }

    #[test]
    fn test_wrong_size_error() {
        let bytes = vec![1, 2];  // Only 2 bytes
        let result = bytes_to_val(&bytes, ValType::I32);
        assert!(result.is_err());
    }
}
```

**Testing Strategy**:
- Unit tests for each type conversion (i32, i64, f32, f64)
- Error cases (wrong size, unsupported types)
- Round-trip conversion tests
- Edge cases (zero, negative, max values)

**Success Criteria**:
- [ ] All primitive types convert correctly (i32, i64, f32, f64)
- [ ] Error handling for unsupported types
- [ ] Test coverage ≥90%
- [ ] Documentation complete with examples

---

### Step 1.2: WASM Function Invocation Implementation (4-6 hours)

**Objective**: Implement actual WASM function calls in `handle_message()`

**File**: `airssys-wasm/src/actor/actor_impl.rs` (lines 190-215)

**Implementation**:

```rust
// Replace lines 190-215 in actor_impl.rs

// 3. WASM function invocation
let runtime = self
    .wasm_runtime_mut()
    .ok_or_else(|| ComponentActorError::not_ready(&component_id_str))?;

// 3.1. Get function export
let func = runtime
    .instance()
    .get_func(runtime.store_mut(), &function)
    .ok_or_else(|| {
        ComponentActorError::from(WasmError::execution_failed(format!(
            "Function '{}' not found in component {}",
            function, component_id_str
        )))
    })?;

trace!(
    component_id = %component_id_str,
    function = %function,
    "Function export found, preparing parameters"
);

// 3.2. Convert decoded args to WASM Val parameters
let func_type = func.ty(runtime.store());
let wasm_params = prepare_wasm_params(&decoded_args, &func_type)
    .map_err(ComponentActorError::from)?;

trace!(
    component_id = %component_id_str,
    param_count = wasm_params.len(),
    "Parameters prepared, invoking WASM function"
);

// 3.3. Call WASM function asynchronously
let mut results = vec![Val::I32(0); func_type.results().len()];
func.call_async(runtime.store_mut(), &wasm_params, &mut results)
    .await
    .map_err(|e| {
        ComponentActorError::from(WasmError::execution_failed_with_source(
            format!(
                "WASM function '{}' trapped in component {}",
                function, component_id_str
            ),
            Box::new(e),
        ))
    })?;

debug!(
    component_id = %component_id_str,
    function = %function,
    result_count = results.len(),
    "WASM function execution completed"
);

// 3.4. Convert results to bytes
let result_bytes = extract_wasm_results(&results)
    .map_err(ComponentActorError::from)?;

// 3.5. Encode with multicodec (use same codec as request)
let encoded_result = encode_multicodec(codec, &result_bytes)
    .map_err(ComponentActorError::from)?;

// 3.6. Send reply via ActorContext
// TODO(Phase 2): Implement ctx.reply() once ActorContext is fully integrated
// For now, log the result
trace!(
    component_id = %component_id_str,
    result_len = encoded_result.len(),
    "Function result encoded, ready to send (reply mechanism pending ActorSystem)"
);

// TEMPORARY: Store result for testing
// Will be replaced with: ctx.reply(ComponentMessage::InvokeResult { ... }).await?;

Ok(())
```

**Testing Strategy**:
- Test with simple WASM functions (add, multiply, etc.)
- Test with functions that return values
- Test trap scenarios (divide by zero, out of bounds)
- Test parameter marshalling edge cases
- Performance benchmark (<100μs overhead)

**Success Criteria**:
- [ ] Function exports retrieved correctly
- [ ] Parameters marshalled to WASM Val
- [ ] Async function calls execute
- [ ] Results extracted and serialized
- [ ] Traps handled gracefully
- [ ] Test coverage ≥90%
- [ ] Performance: <100μs overhead per call

---

### Step 1.3: InterComponent WASM Call Implementation (2-4 hours)

**Objective**: Implement handle-message export invocation

**File**: `airssys-wasm/src/actor/actor_impl.rs` (lines 236-246)

**Implementation**:

```rust
// Replace lines 236-246 in actor_impl.rs

// 3. Route to WASM handle-message export
if let Some(handle_fn) = &runtime.exports().handle_message {
    trace!(
        component_id = %component_id_str,
        sender = %sender_str,
        payload_len = payload.len(),
        "Calling handle-message export"
    );

    // 3.1. Prepare parameters for handle-message
    // Signature: handle-message(sender: string, payload: list<u8>) -> ()
    let sender_bytes = sender_str.as_bytes();
    
    // Create WASM parameters
    // Note: This requires WIT-specific parameter marshalling
    // For now, pass payload directly as memory buffer
    let params = vec![
        Val::I32(sender_bytes.as_ptr() as i32),  // sender ptr
        Val::I32(sender_bytes.len() as i32),      // sender len
        Val::I32(payload.as_ptr() as i32),        // payload ptr
        Val::I32(payload.len() as i32),           // payload len
    ];

    // 3.2. Call handle-message export
    let mut results = vec![];
    handle_fn
        .call_async(runtime.store_mut(), &params, &mut results)
        .await
        .map_err(|e| {
            ComponentActorError::from(WasmError::execution_failed_with_source(
                format!(
                    "handle-message trapped in component {} (from {})",
                    component_id_str, sender_str
                ),
                Box::new(e),
            ))
        })?;

    debug!(
        component_id = %component_id_str,
        sender = %sender_str,
        "handle-message export call completed successfully"
    );
} else {
    warn!(
        component_id = %component_id_str,
        sender = %sender_str,
        "Component has no handle-message export, message discarded"
    );
}

Ok(())
```

**Testing Strategy**:
- Test with WASM component that has handle-message export
- Test with component without handle-message (warning logged)
- Test trap scenarios in handle-message
- Test parameter marshalling
- Performance test (<1ms per message)

**Success Criteria**:
- [ ] handle-message export called successfully
- [ ] Sender and payload marshalled correctly
- [ ] Traps propagated to supervisor
- [ ] Missing export handled gracefully
- [ ] Test coverage ≥90%
- [ ] Performance: <1ms per message

---

### Step 1.4: Integration Testing for Deferred Work (2-4 hours)

**Objective**: Comprehensive tests for WASM invocation

**File**: `airssys-wasm/tests/actor_invocation_tests.rs` (NEW)

**Implementation**:

```rust
// tests/actor_invocation_tests.rs

use airssys_wasm::actor::{ComponentActor, ComponentMessage};
use airssys_wasm::core::{ComponentId, ComponentMetadata, encode_multicodec, Codec};
use airssys_rt::actor::Actor;
use tokio;

/// Test fixture: Simple WASM component with add function
fn create_add_component() -> ComponentActor {
    // TODO: Load actual WASM bytes for add(i32, i32) -> i32
    let component_id = ComponentId::new("test-add-component");
    let metadata = ComponentMetadata::default();
    ComponentActor::new(component_id, metadata, CapabilitySet::default())
}

#[tokio::test]
async fn test_invoke_i32_function() {
    let mut actor = create_add_component();
    // Assume Child::start() already called
    
    // Prepare args: add(5, 3)
    let args_bytes = vec![5u8, 0, 0, 0, 3, 0, 0, 0];  // Two i32s
    let encoded_args = encode_multicodec(Codec::Borsh, &args_bytes).unwrap();
    
    let msg = ComponentMessage::Invoke {
        function: "add".to_string(),
        args: encoded_args,
    };
    
    let result = actor.handle_message(msg, &mut mock_ctx).await;
    assert!(result.is_ok());
    
    // TODO: Verify result is 8 when reply mechanism implemented
}

#[tokio::test]
async fn test_invoke_nonexistent_function() {
    let mut actor = create_add_component();
    
    let msg = ComponentMessage::Invoke {
        function: "nonexistent".to_string(),
        args: vec![],
    };
    
    let result = actor.handle_message(msg, &mut mock_ctx).await;
    assert!(result.is_err());
    assert!(format!("{:?}", result).contains("not found"));
}

#[tokio::test]
async fn test_invoke_with_trap() {
    // Test divide by zero or similar trap scenario
    let mut actor = create_divide_component();
    
    let args_bytes = vec![10u8, 0, 0, 0, 0, 0, 0, 0];  // divide(10, 0)
    let encoded_args = encode_multicodec(Codec::Borsh, &args_bytes).unwrap();
    
    let msg = ComponentMessage::Invoke {
        function: "divide".to_string(),
        args: encoded_args,
    };
    
    let result = actor.handle_message(msg, &mut mock_ctx).await;
    assert!(result.is_err());
    assert!(format!("{:?}", result).contains("trapped"));
}

#[tokio::test]
async fn test_intercomponent_message_flow() {
    let mut actor = create_receiver_component();
    
    let sender = ComponentId::new("sender-component");
    let payload = b"test message".to_vec();
    
    let msg = ComponentMessage::InterComponent {
        sender,
        payload,
    };
    
    let result = actor.handle_message(msg, &mut mock_ctx).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_intercomponent_without_handler() {
    let mut actor = create_add_component();  // No handle-message export
    
    let sender = ComponentId::new("sender-component");
    let msg = ComponentMessage::InterComponent {
        sender,
        payload: vec![1, 2, 3],
    };
    
    let result = actor.handle_message(msg, &mut mock_ctx).await;
    // Should succeed but log warning
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_message_throughput() {
    let mut actor = create_add_component();
    let args_bytes = vec![1u8, 0, 0, 0, 2, 0, 0, 0];
    let encoded_args = encode_multicodec(Codec::Borsh, &args_bytes).unwrap();
    
    let msg = ComponentMessage::Invoke {
        function: "add".to_string(),
        args: encoded_args,
    };
    
    let start = std::time::Instant::now();
    for _ in 0..10_000 {
        actor.handle_message(msg.clone(), &mut mock_ctx).await.ok();
    }
    let elapsed = start.elapsed();
    
    let throughput = 10_000.0 / elapsed.as_secs_f64();
    println!("Message throughput: {:.0} msg/sec", throughput);
    assert!(throughput > 10_000.0, "Target: >10,000 msg/sec");
}
```

**Success Criteria**:
- [ ] All integration tests pass
- [ ] Performance benchmark meets >10,000 msg/sec
- [ ] Error scenarios covered
- [ ] Trap handling verified

---

## PART 2: ActorSystem Integration (8-12 hours)

### Step 2.1: Component Spawner Implementation (4-6 hours)

**Objective**: Implement ActorSystem::spawn() wrapper for components

**File**: `airssys-wasm/src/actor/component_spawner.rs` (NEW)

**Implementation**:

```rust
// src/actor/component_spawner.rs

use crate::actor::ComponentActor;
use crate::core::{ComponentId, ComponentMetadata, ComponentSpec};
use airssys_rt::actor::{ActorSystem, ActorRef};
use airssys_rt::supervisor::{ChildSpec, SupervisorNode};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

/// Component spawner - wraps ActorSystem for component lifecycle
pub struct ComponentSpawner {
    actor_system: Arc<ActorSystem>,
    supervisor: Arc<SupervisorNode>,
}

impl ComponentSpawner {
    pub fn new(
        actor_system: Arc<ActorSystem>,
        supervisor: Arc<SupervisorNode>,
    ) -> Self {
        Self {
            actor_system,
            supervisor,
        }
    }
    
    /// Spawn a new component actor
    ///
    /// # Process
    /// 1. Create ComponentActor (WASM not loaded yet)
    /// 2. Add to supervisor tree
    /// 3. Spawn via ActorSystem (creates mailbox, starts actor)
    /// 4. Wait for Child::start() to complete (WASM loads)
    /// 5. Return ActorRef for messaging
    ///
    /// # Performance
    /// Target: <5ms P99 spawn time (including WASM load)
    pub async fn spawn_component(
        &self,
        spec: ComponentSpec,
    ) -> Result<ActorRef<ComponentMessage>, SpawnError> {
        let component_id = ComponentId::new(&spec.name);
        
        debug!(
            component_id = %component_id.as_str(),
            "Spawning component actor"
        );
        
        // 1. Create ComponentActor
        let actor = ComponentActor::new(
            component_id.clone(),
            spec.metadata.clone(),
            spec.capabilities.clone(),
        );
        
        // 2. Add to supervisor tree
        let child_spec = ChildSpec::new(component_id.clone(), actor);
        self.supervisor
            .start_child(child_spec)
            .await
            .map_err(|e| SpawnError::SupervisorError(e))?;
        
        // 3. Spawn via ActorSystem
        // This creates mailbox, calls Child::start(), begins message loop
        let actor_ref = self.actor_system
            .spawn(actor)
            .await
            .map_err(|e| SpawnError::ActorSystemError(e))?;
        
        // 4. Wait for Ready state (Child::start() completes WASM load)
        timeout(
            Duration::from_secs(10),
            self.wait_for_ready(&component_id),
        )
        .await
        .map_err(|_| SpawnError::Timeout)?
        .map_err(|e| SpawnError::StartupError(e))?;
        
        info!(
            component_id = %component_id.as_str(),
            "Component spawned and ready"
        );
        
        Ok(actor_ref)
    }
    
    async fn wait_for_ready(&self, component_id: &ComponentId) -> Result<(), String> {
        // Poll component health until Ready
        // For now, simplified - full implementation in Phase 3
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SpawnError {
    #[error("Supervisor error: {0}")]
    SupervisorError(String),
    
    #[error("ActorSystem error: {0}")]
    ActorSystemError(String),
    
    #[error("Component startup timeout (>10s)")]
    Timeout,
    
    #[error("Component startup error: {0}")]
    StartupError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_spawn_component() {
        let actor_system = Arc::new(ActorSystem::new());
        let supervisor = Arc::new(SupervisorNode::new());
        let spawner = ComponentSpawner::new(actor_system, supervisor);
        
        let spec = ComponentSpec::new("test-component");
        let actor_ref = spawner.spawn_component(spec).await.unwrap();
        
        assert!(actor_ref.is_alive());
    }
    
    #[tokio::test]
    async fn test_spawn_time_target() {
        let spawner = create_test_spawner();
        let spec = ComponentSpec::new("perf-test");
        
        let start = std::time::Instant::now();
        spawner.spawn_component(spec).await.unwrap();
        let elapsed = start.elapsed();
        
        println!("Spawn time: {:?}", elapsed);
        assert!(elapsed < Duration::from_millis(5), "Target: <5ms");
    }
}
```

**Testing Strategy**:
- Test successful component spawn
- Test spawn time performance (<5ms)
- Test concurrent spawns (100 components)
- Test spawn failures (invalid spec, supervisor errors)
- Test timeout scenarios

**Success Criteria**:
- [ ] Components spawn via ActorSystem
- [ ] ActorRef returned for messaging
- [ ] Spawn time <5ms average
- [ ] Concurrent spawns supported
- [ ] Test coverage ≥90%

---

### Step 2.2: Component Instance Registry (4-6 hours)

**Objective**: Implement component ID to ActorRef mapping

**File**: `airssys-wasm/src/actor/component_registry.rs` (NEW)

**Implementation**:

```rust
// src/actor/component_registry.rs

use crate::core::ComponentId;
use airssys_rt::actor::ActorRef;
use std::collections::HashMap;
use std::sync::RwLock;
use chrono::{DateTime, Utc};

/// Component instance registry for tracking active components
pub struct ComponentRegistry {
    entries: RwLock<HashMap<ComponentId, ComponentEntry>>,
}

#[derive(Debug, Clone)]
pub struct ComponentEntry {
    pub actor_ref: ActorRef<ComponentMessage>,
    pub status: ComponentStatus,
    pub spawned_at: DateTime<Utc>,
    pub metadata: ComponentMetadata,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentStatus {
    Starting,
    Running,
    Stopping,
    Terminated,
    Failed(String),
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }
    
    /// Register a new component instance
    pub fn register(
        &self,
        component_id: ComponentId,
        entry: ComponentEntry,
    ) -> Result<(), RegistryError> {
        let mut entries = self.entries.write().unwrap();
        
        if entries.contains_key(&component_id) {
            return Err(RegistryError::AlreadyRegistered(component_id));
        }
        
        entries.insert(component_id.clone(), entry);
        
        debug!(
            component_id = %component_id.as_str(),
            "Component registered in registry"
        );
        
        Ok(())
    }
    
    /// Get component entry by ID (O(1) lookup)
    pub fn get(&self, component_id: &ComponentId) -> Option<ComponentEntry> {
        let entries = self.entries.read().unwrap();
        entries.get(component_id).cloned()
    }
    
    /// Unregister component
    pub fn unregister(&self, component_id: &ComponentId) -> Result<(), RegistryError> {
        let mut entries = self.entries.write().unwrap();
        
        if entries.remove(component_id).is_none() {
            return Err(RegistryError::NotFound(component_id.clone()));
        }
        
        debug!(
            component_id = %component_id.as_str(),
            "Component unregistered from registry"
        );
        
        Ok(())
    }
    
    /// List all active components
    pub fn list_all(&self) -> Vec<ComponentId> {
        let entries = self.entries.read().unwrap();
        entries.keys().cloned().collect()
    }
    
    /// Update component status
    pub fn update_status(
        &self,
        component_id: &ComponentId,
        status: ComponentStatus,
    ) -> Result<(), RegistryError> {
        let mut entries = self.entries.write().unwrap();
        
        let entry = entries
            .get_mut(component_id)
            .ok_or_else(|| RegistryError::NotFound(component_id.clone()))?;
        
        entry.status = status;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Component {0} already registered")]
    AlreadyRegistered(ComponentId),
    
    #[error("Component {0} not found")]
    NotFound(ComponentId),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_register_and_get() {
        let registry = ComponentRegistry::new();
        let component_id = ComponentId::new("test-component");
        let entry = ComponentEntry {
            actor_ref: mock_actor_ref(),
            status: ComponentStatus::Running,
            spawned_at: Utc::now(),
            metadata: ComponentMetadata::default(),
        };
        
        registry.register(component_id.clone(), entry.clone()).unwrap();
        
        let retrieved = registry.get(&component_id).unwrap();
        assert_eq!(retrieved.status, ComponentStatus::Running);
    }
    
    #[test]
    fn test_duplicate_registration() {
        let registry = ComponentRegistry::new();
        let component_id = ComponentId::new("test");
        let entry = create_test_entry();
        
        registry.register(component_id.clone(), entry.clone()).unwrap();
        let result = registry.register(component_id, entry);
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_unregister() {
        let registry = ComponentRegistry::new();
        let component_id = ComponentId::new("test");
        registry.register(component_id.clone(), create_test_entry()).unwrap();
        
        registry.unregister(&component_id).unwrap();
        assert!(registry.get(&component_id).is_none());
    }
    
    #[test]
    fn test_o1_lookup_performance() {
        let registry = ComponentRegistry::new();
        
        // Register 1000 components
        for i in 0..1000 {
            let id = ComponentId::new(&format!("component-{}", i));
            registry.register(id, create_test_entry()).unwrap();
        }
        
        // Lookup should be O(1)
        let target = ComponentId::new("component-500");
        let start = std::time::Instant::now();
        registry.get(&target).unwrap();
        let elapsed = start.elapsed();
        
        println!("Lookup time: {:?}", elapsed);
        assert!(elapsed < Duration::from_micros(10));
    }
}
```

**Testing Strategy**:
- Test registration and retrieval
- Test duplicate registration (error)
- Test unregistration
- Test O(1) lookup performance
- Test concurrent access (RwLock)
- Test status updates

**Success Criteria**:
- [ ] O(1) lookup performance
- [ ] Thread-safe operations
- [ ] Clear error messages
- [ ] Test coverage ≥90%

---

## PART 3: Testing & Validation (4-6 hours)

### Step 3.1: DEBT-WASM-004 Verification (2-3 hours)

**Objective**: Ensure all deferred work completed per checklist

**File**: `.memory-bank/sub-projects/airssys-wasm/scripts/check-debt-wasm-004.sh`

**Implementation**:

```bash
#!/bin/bash
set -e

echo "🔍 Verifying DEBT-WASM-004 completion..."

# Check for FUTURE WORK comments
echo "Checking for unresolved TODO comments..."
if grep -n "FUTURE WORK" airssys-wasm/src/actor/actor_impl.rs; then
    echo "❌ ERROR: Unresolved deferred work found in actor_impl.rs"
    echo "See DEBT-WASM-004 for required implementation"
    exit 1
fi

echo "✅ No FUTURE WORK comments found"

# Check for required functions
echo "Checking for required function implementations..."

if ! grep -q "prepare_wasm_params" airssys-wasm/src/actor/type_conversion.rs; then
    echo "❌ ERROR: prepare_wasm_params not found"
    exit 1
fi

if ! grep -q "extract_wasm_results" airssys-wasm/src/actor/type_conversion.rs; then
    echo "❌ ERROR: extract_wasm_results not found"
    exit 1
fi

echo "✅ All required functions implemented"

# Run tests
echo "Running actor invocation tests..."
cargo test --package airssys-wasm --test actor_invocation_tests

echo "Running performance benchmarks..."
cargo test --package airssys-wasm test_message_throughput -- --nocapture

echo "✅ All DEBT-WASM-004 items resolved"
```

**Checklist**:
- [ ] No "FUTURE WORK" comments in actor_impl.rs
- [ ] prepare_wasm_params implemented
- [ ] extract_wasm_results implemented
- [ ] All integration tests passing
- [ ] Performance benchmarks meet targets (>10,000 msg/sec)
- [ ] Code review completed
- [ ] Documentation updated

---

### Step 3.2: Performance Benchmarking (1-2 hours)

**Objective**: Validate performance targets

**File**: `airssys-wasm/benches/actor_performance.rs` (NEW)

**Implementation**:

```rust
// benches/actor_performance.rs

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use airssys_wasm::actor::{ComponentActor, ComponentMessage};
use tokio::runtime::Runtime;

fn bench_message_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_throughput");
    
    let rt = Runtime::new().unwrap();
    let actor = rt.block_on(create_test_actor());
    
    group.bench_function("invoke_i32_function", |b| {
        b.to_async(&rt).iter(|| async {
            let msg = create_invoke_message();
            actor.handle_message(msg, &mock_ctx).await.unwrap();
        });
    });
    
    group.bench_function("intercomponent_message", |b| {
        b.to_async(&rt).iter(|| async {
            let msg = create_intercomponent_message();
            actor.handle_message(msg, &mock_ctx).await.unwrap();
        });
    });
    
    group.finish();
}

fn bench_spawn_time(c: &mut Criterion) {
    let mut group = c.benchmark_group("spawn_time");
    
    let rt = Runtime::new().unwrap();
    let spawner = rt.block_on(create_test_spawner());
    
    group.bench_function("spawn_component", |b| {
        b.to_async(&rt).iter(|| async {
            let spec = ComponentSpec::new("bench-component");
            spawner.spawn_component(spec).await.unwrap();
        });
    });
    
    group.finish();
}

criterion_group!(benches, bench_message_throughput, bench_spawn_time);
criterion_main!(benches);
```

**Performance Targets**:
- [ ] Message throughput: >10,000 msg/sec
- [ ] WASM call overhead: <100μs
- [ ] Component spawn: <5ms P99
- [ ] Registry lookup: <10μs

---

### Step 3.3: Documentation Updates (1 hour)

**Objective**: Update documentation with completed work

**Files to Update**:
1. `task-004-block-3-actor-system-integration.md` - Mark Task 2.1 complete
2. `DEBT-WASM-004` - Sign off Items #1 and #2
3. `actor_impl.rs` - Update module-level docs
4. `README.md` - Add Phase 2 progress

**Documentation Checklist**:
- [ ] Task 2.1 marked complete in task document
- [ ] DEBT-WASM-004 Items #1 and #2 signed off
- [ ] Module documentation updated
- [ ] Examples added for WASM invocation
- [ ] Performance results documented

---

## Success Criteria & Validation

### Phase 2 Task 2.1 Completion Criteria

**Functional Requirements:**
- [ ] DEBT-WASM-004 Item #1 complete (WASM Function Invocation)
- [ ] DEBT-WASM-004 Item #2 complete (InterComponent WASM Call)
- [ ] ActorSystem::spawn() integration working
- [ ] Component registry operational
- [ ] All "FUTURE WORK" comments removed

**Performance Requirements:**
- [ ] Message throughput: >10,000 msg/sec
- [ ] WASM call overhead: <100μs per call
- [ ] Component spawn time: <5ms P99
- [ ] Registry lookup: O(1), <10μs

**Quality Requirements:**
- [ ] Test coverage ≥90% for new code
- [ ] Zero clippy warnings
- [ ] All integration tests passing (306+ total tests)
- [ ] Code review completed
- [ ] Documentation complete

**DEBT-WASM-004 Sign-Off:**
- [ ] Item #1 implementer sign-off: ________________
- [ ] Item #1 reviewer sign-off: ________________
- [ ] Item #2 implementer sign-off: ________________
- [ ] Item #2 reviewer sign-off: ________________
- [ ] Test coverage verified: _____% (≥90%)
- [ ] Performance benchmarks passed: ☐ YES ☐ NO

---

## Risk Assessment & Mitigation

### Technical Risks

**Risk 1: Type Conversion Complexity**
- **Probability**: Medium
- **Impact**: High
- **Mitigation**: 
  - Start with simple types (i32, i64)
  - Defer complex types to future task
  - Use serde for structured data
  - Comprehensive unit tests

**Risk 2: WASM Trap Handling**
- **Probability**: Medium
- **Impact**: Medium
- **Mitigation**:
  - Wasmtime provides good trap context
  - Test all trap scenarios
  - Ensure supervisor can restart

**Risk 3: Performance Degradation**
- **Probability**: Low
- **Impact**: High
- **Mitigation**:
  - Early performance benchmarking
  - Profile hot paths
  - Optimize type conversion
  - Use Criterion for regression detection

**Risk 4: ActorSystem Integration Issues**
- **Probability**: Low
- **Impact**: Medium
- **Mitigation**:
  - airssys-rt API is stable
  - Reference Task 1.2 patterns
  - Incremental integration testing

---

## Timeline & Effort Estimates

### Detailed Breakdown

| Step | Task | Estimated Hours | Priority |
|------|------|----------------|----------|
| 1.1 | Type Conversion System | 4-6h | 🔥 CRITICAL |
| 1.2 | WASM Function Invocation | 4-6h | 🔥 CRITICAL |
| 1.3 | InterComponent Call | 2-4h | 🔥 CRITICAL |
| 1.4 | Integration Testing (Deferred) | 2-4h | 🔥 CRITICAL |
| 2.1 | Component Spawner | 4-6h | HIGH |
| 2.2 | Component Registry | 4-6h | HIGH |
| 3.1 | DEBT-WASM-004 Verification | 2-3h | MEDIUM |
| 3.2 | Performance Benchmarking | 1-2h | MEDIUM |
| 3.3 | Documentation | 1h | LOW |

**Total Estimated Effort: 24-30 hours**

### Recommended Schedule

**Day 1-2 (8-10h)**: Part 1 - Deferred Work
- Morning: Type conversion system (Step 1.1)
- Afternoon: WASM function invocation (Step 1.2)
- Evening: InterComponent call (Step 1.3)

**Day 3 (4-6h)**: Part 1 Completion
- Morning: Integration testing (Step 1.4)
- Afternoon: Fix any issues, refine implementation

**Day 4 (8-10h)**: Part 2 - ActorSystem Integration
- Morning: Component spawner (Step 2.1)
- Afternoon: Component registry (Step 2.2)

**Day 5 (4-6h)**: Part 3 - Validation
- Morning: DEBT-WASM-004 verification (Step 3.1)
- Afternoon: Performance benchmarking (Step 3.2)
- Evening: Documentation (Step 3.3)

---

## Next Steps After Completion

### Phase 2 Task 2.2: Component Instance Management
- Component ID to ActorRef mapping (already implemented in registry)
- Enhanced instance tracking
- Lifecycle event logging

### Phase 2 Task 2.3: Actor Address and Routing
- ActorRef wrapper for component addressing
- Message routing via ActorRef.send()
- Routing performance tests

### Phase 3 Task 3.1: Supervisor Tree Setup
- Requires DEBT-WASM-004 Item #4 (Health Check)
- SupervisorNode configuration
- Restart policies

---

## Appendix

### A. Reference Code Examples

See DEBT-WASM-004 for complete implementation examples:
- Lines 39-68: WASM Function Invocation
- Lines 102-121: InterComponent WASM Call
- Lines 203-241: Health Check Parsing (deferred to Phase 3)

### B. Testing Resources

**WASM Test Components** (to be created):
- `tests/fixtures/add.wasm` - Simple i32 addition
- `tests/fixtures/divide.wasm` - Division with trap testing
- `tests/fixtures/echo.wasm` - handle-message implementation
- `tests/fixtures/health.wasm` - _health export implementation

### C. Performance Baselines

**airssys-rt Proven Performance:**
- Actor spawn: ~625ns
- Message routing: ~211ns (MessageBroker)
- Throughput: 4.7M msg/sec

**Target Performance (with WASM overhead):**
- Component spawn: <5ms (includes WASM load)
- Message throughput: >10,000 msg/sec (470x slower acceptable due to WASM)
- WASM call overhead: <100μs

---

## Document History

| Date | Change | Author |
|------|--------|--------|
| 2025-12-13 | Initial creation - Comprehensive Task 2.1 plan | AI Agent |

---

## Related Documents

- **DEBT-WASM-004**: Technical debt tracking (MANDATORY reading)
- **KNOWLEDGE-WASM-016**: Implementation guide (reference)
- **WASM-TASK-004**: Block 3 task specification (parent)
- **Task 1.2 Completion Summary**: Reference implementation pattern
- **Task 1.3 Completion Summary**: (to be created after Task 2.1)

---

**🎯 READY TO START**

This plan provides:
- ✅ Clear implementation steps with code examples
- ✅ Comprehensive testing strategy
- ✅ Performance validation approach
- ✅ Risk mitigation strategies
- ✅ Realistic timeline (24-30 hours)
- ✅ Success criteria checklist
- ✅ Integration with existing codebase

**Next Action**: Begin Step 1.1 (Type Conversion System)
