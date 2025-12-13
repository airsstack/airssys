# Task 1.3: Actor Trait Message Handling Implementation Plan

**Task ID:** WASM-TASK-004 Phase 1 Task 1.3  
**Parent Task:** WASM-TASK-004 Block 3 - Actor System Integration  
**Status:** READY TO START  
**Priority:** CRITICAL PATH  
**Created:** 2025-12-13  
**Estimated Effort:** 16-20 hours  

---

## Executive Summary

This plan details the implementation of Actor trait message handling for ComponentActor, enabling component invocation through the airssys-rt actor system. Task 1.3 builds on the completed Task 1.1 (ComponentActor foundation) and Task 1.2 (WASM lifecycle) to deliver the message routing layer that connects actor-based messaging with WASM function execution.

**Goal:** Implement `Actor::handle_message()` to route ComponentMessage variants to WASM exports with multicodec deserialization, type conversion, and comprehensive error handling.

**Success Criteria:**
- ✅ `handle_message()` fully implemented for all 6 ComponentMessage variants
- ✅ Multicodec deserialization (Borsh, CBOR, JSON) integrated
- ✅ WASM function invocation with parameter marshalling
- ✅ `pre_start()` and `post_stop()` lifecycle hooks implemented
- ✅ 20-30 comprehensive tests passing
- ✅ Zero warnings (clippy --all-targets --all-features)
- ✅ 100% rustdoc coverage
- ✅ Performance target: >10,000 msg/sec throughput

---

## 1. Context & Dependencies

### 1.1 What's Already Complete

#### Task 1.1: ComponentActor Foundation ✅ (Nov 29, 2025)
- ComponentActor struct (1,334 lines)
- ActorState enum (7-state machine)
- ComponentMessage enum (6 message types)
- HealthStatus enum
- WasmRuntime with Wasmtime Engine, Store, Instance
- WasmExports caching (_start, _cleanup, _health, handle-message)
- ComponentResourceLimiter implementing wasmtime::ResourceLimiter
- Actor trait stub implementation (297 lines in actor_impl.rs)
- **43 tests passing, zero warnings**

#### Task 1.2: Child Trait WASM Lifecycle ✅ (Nov 30, 2025)
- Full Child trait implementation (588 lines in child_impl.rs)
- Child::start() with WASM loading, compilation, instantiation
- Child::stop() with graceful shutdown and resource cleanup
- WasmRuntime integration complete
- Security configuration integrated
- **50 tests total passing, zero warnings**

### 1.2 Current State Analysis

**File:** `airssys-wasm/src/actor/actor_impl.rs` (297 lines)

**Existing Stubs (Lines 132-436):**
```rust
#[async_trait]
impl Actor for ComponentActor {
    type Message = ComponentMessage;
    type Error = ComponentActorError;

    async fn handle_message(&mut self, msg: Self::Message, ctx: &mut ActorContext) -> Result<(), Self::Error> {
        match msg {
            ComponentMessage::Invoke { function, args } => {
                // TODO Task 1.3: Full implementation
                // 1. Verify WASM loaded
                // 2. Deserialize args using multicodec
                // 3. Call WASM function export
                // 4. Encode result and reply
            }
            ComponentMessage::InterComponent { sender, payload } => {
                // TODO Task 1.3: Route to handle-message export
            }
            ComponentMessage::HealthCheck => {
                // TODO Task 1.3: Call _health export
            }
            ComponentMessage::Shutdown => {
                // TODO Task 1.3: Signal ActorSystem
            }
            _ => Ok(())
        }
    }

    async fn pre_start(&mut self, ctx: &mut ActorContext) -> Result<(), Self::Error> {
        // TODO Task 1.3: Registry integration (Block 6)
    }

    async fn post_stop(&mut self, ctx: &mut ActorContext) -> Result<(), Self::Error> {
        // TODO Task 1.3: Registry cleanup (Block 6)
    }
}
```

**What Exists:**
- ComponentActorError wrapper type (lines 64-99)
- Message trait impl for ComponentMessage (lines 101-106)
- Actor trait skeleton with proper type declarations
- 11 basic tests (lines 511-662)

**What's Missing:**
- Multicodec deserialization logic
- WASM function invocation with parameter conversion
- Type marshalling (Rust values → WASM Val parameters)
- Result extraction (WASM Val results → Rust bytes)
- Host function registration (Wasmtime Linker)
- Full health check implementation
- Comprehensive test coverage

### 1.3 Dependencies

#### Internal Dependencies (COMPLETE ✅)
- **Block 1 (WASM Runtime)**: WasmRuntime, ComponentResourceLimiter ✅
- **Task 1.1**: ComponentActor struct, ComponentMessage enum ✅
- **Task 1.2**: Child::start() with WASM loading ✅
- **ADR-WASM-001**: Multicodec strategy (Borsh, CBOR, JSON) ✅
- **ADR-WASM-003**: Component lifecycle management ✅
- **ADR-WASM-006**: Actor-based isolation model ✅

#### External Dependencies (AVAILABLE ✅)
- **airssys-rt**: Actor, Child, ActorContext, MessageBroker ✅
- **wasmtime**: Func, Val, Store, Instance ✅
- **Multicodec crates**: multicodec, unsigned-varint ✅

#### Future Dependencies (DEFERRED TO LATER TASKS)
- **Block 4**: Capability-based security enforcement (deferred)
- **Block 6**: Component registry integration (pre_start/post_stop full impl)
- **Phase 2**: ActorSystem::spawn() integration (deferred)

---

## 2. Scope & Objectives

### 2.1 In-Scope for Task 1.3

#### Core Message Handling
1. **Invoke Message** (Primary complexity)
   - Multicodec deserialization (Borsh, CBOR, JSON)
   - WASM function export lookup
   - Type conversion: decoded args → WASM Val parameters
   - Async WASM function invocation
   - Result extraction: WASM Val results → bytes
   - Multicodec encoding for response
   - Error handling with component context

2. **InterComponent Message**
   - Capability checking placeholder (deferred to Block 4)
   - Route to WASM handle-message export
   - Handle missing export gracefully

3. **HealthCheck Message**
   - Determine health status
   - Call _health export if available
   - Return HealthStatus enum

4. **Shutdown Message**
   - State transition to Stopping
   - Signal ActorSystem (stub for Phase 2)

5. **Response Messages** (InvokeResult, HealthStatus)
   - Log for debugging
   - No action required

#### Lifecycle Hooks
6. **pre_start()**
   - Verify WASM runtime loaded
   - Log initialization
   - Registry stub (Block 6)

7. **post_stop()**
   - State transition to Terminated
   - Log cleanup
   - Registry deregistration stub (Block 6)

#### Type Conversion Module
8. **New File:** `src/actor/type_conversion.rs`
   - `prepare_wasm_params()`: Convert decoded args to Vec<Val>
   - `extract_wasm_results()`: Convert Vec<Val> to bytes
   - Support all Wasmtime Val types (I32, I64, F32, F64)

#### Multicodec Integration
9. **Core Module Extension:** `src/core/multicodec.rs`
   - `decode_multicodec()`: Parse varint prefix + deserialize
   - `encode_multicodec()`: Serialize + prepend varint
   - `Codec` enum (Borsh = 0x701, CBOR = 0x51, JSON = 0x0200)
   - Varint encoding/decoding utilities

### 2.2 Out-of-Scope (Deferred)

#### Phase 2 (ActorSystem Integration)
- Full ActorContext messaging (ctx.reply(), ctx.send())
- ActorSystem::spawn() integration
- MessageBroker routing integration
- Mailbox management

#### Block 4 (Security Layer)
- Fine-grained capability enforcement
- Security policy validation
- Audit logging

#### Block 6 (Component Registry)
- Registry registration in pre_start()
- Registry deregistration in post_stop()
- Component discovery
- Metadata persistence

### 2.3 Explicit Non-Goals

❌ **NOT implementing in Task 1.3:**
- Host function registration (Linker setup deferred to Phase 2 Task 2.2)
- Full ActorContext reply mechanism (pending Phase 2 Task 2.3)
- Component registry integration (Block 6)
- Inter-component routing (Block 5)
- Performance optimization (baseline first, optimize in Phase 3)

---

## 3. Technical Approach

### 3.1 Message Handling Architecture

```text
┌─────────────────────────────────────────────────────┐
│           Actor::handle_message()                   │
│                                                     │
│  ┌─────────────────────────────────────────────┐  │
│  │ ComponentMessage::Invoke                    │  │
│  │  1. decode_multicodec(&args) → (Codec, Vec<u8>) │
│  │  2. prepare_wasm_params() → Vec<Val>        │  │
│  │  3. get_func(&function) → Typed<Func>       │  │
│  │  4. call_async() → Vec<Val>                 │  │
│  │  5. extract_wasm_results() → Vec<u8>        │  │
│  │  6. encode_multicodec() → Vec<u8>           │  │
│  └─────────────────────────────────────────────┘  │
│                                                     │
│  ┌─────────────────────────────────────────────┐  │
│  │ ComponentMessage::InterComponent            │  │
│  │  1. Check capabilities (stub)                │  │
│  │  2. Route to handle-message export          │  │
│  │  3. call_async() with raw payload           │  │
│  └─────────────────────────────────────────────┘  │
│                                                     │
│  ┌─────────────────────────────────────────────┐  │
│  │ ComponentMessage::HealthCheck               │  │
│  │  1. Check WASM runtime loaded               │  │
│  │  2. Call _health export if exists           │  │
│  │  3. Return HealthStatus enum                │  │
│  └─────────────────────────────────────────────┘  │
│                                                     │
│  ┌─────────────────────────────────────────────┐  │
│  │ ComponentMessage::Shutdown                  │  │
│  │  1. Set state to Stopping                   │  │
│  │  2. ctx.stop() (stub for Phase 2)           │  │
│  └─────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

### 3.2 Multicodec Deserialization Flow

```text
Input: &[u8] with multicodec prefix
│
├─ 1. Parse Varint Prefix (1-4 bytes)
│   └─ Extract codec identifier (0x701, 0x51, 0x0200)
│
├─ 2. Identify Codec
│   ├─ 0x701 → Borsh
│   ├─ 0x51  → CBOR
│   └─ 0x0200 → JSON
│
└─ 3. Return (Codec, Payload)
    └─ Payload: remaining bytes after varint prefix
```

### 3.3 Type Conversion Strategy

#### Rust Values → WASM Parameters

```rust
pub fn prepare_wasm_params(
    decoded_args: &[u8],
    func_type: &wasmtime::FuncType,
) -> Result<Vec<wasmtime::Val>, WasmError> {
    // Strategy: For Task 1.3, assume simple parameter types
    // Phase 2 will add full type marshalling
    
    let param_types = func_type.params();
    let mut wasm_params = Vec::with_capacity(param_types.len());
    
    // For now, if no params expected, return empty vec
    if param_types.len() == 0 {
        return Ok(wasm_params);
    }
    
    // Simple case: single i32 parameter (common for test functions)
    if param_types.len() == 1 {
        match param_types[0] {
            wasmtime::ValType::I32 => {
                // Decode first 4 bytes as i32
                if decoded_args.len() >= 4 {
                    let val = i32::from_le_bytes([
                        decoded_args[0],
                        decoded_args[1],
                        decoded_args[2],
                        decoded_args[3],
                    ]);
                    wasm_params.push(wasmtime::Val::I32(val));
                }
            }
            _ => return Err(WasmError::execution_failed("Unsupported parameter type")),
        }
    }
    
    Ok(wasm_params)
}
```

#### WASM Results → Rust Bytes

```rust
pub fn extract_wasm_results(
    results: &[wasmtime::Val],
) -> Result<Vec<u8>, WasmError> {
    // For Task 1.3: Extract simple return types
    
    if results.is_empty() {
        return Ok(Vec::new());
    }
    
    // Single result case (most common)
    if results.len() == 1 {
        match &results[0] {
            wasmtime::Val::I32(v) => Ok(v.to_le_bytes().to_vec()),
            wasmtime::Val::I64(v) => Ok(v.to_le_bytes().to_vec()),
            wasmtime::Val::F32(v) => Ok(v.to_le_bytes().to_vec()),
            wasmtime::Val::F64(v) => Ok(v.to_le_bytes().to_vec()),
            _ => Err(WasmError::execution_failed("Unsupported return type")),
        }
    } else {
        // Multiple results: serialize as array
        let mut bytes = Vec::new();
        for val in results {
            match val {
                wasmtime::Val::I32(v) => bytes.extend_from_slice(&v.to_le_bytes()),
                wasmtime::Val::I64(v) => bytes.extend_from_slice(&v.to_le_bytes()),
                _ => return Err(WasmError::execution_failed("Unsupported multi-result type")),
            }
        }
        Ok(bytes)
    }
}
```

### 3.4 Error Handling Strategy

**Principle:** All errors include component_id context for debugging.

```rust
// Error creation pattern (already exists in ComponentActorError)
fn not_ready(component_id: &str) -> ComponentActorError {
    ComponentActorError::new(WasmError::component_not_found(format!(
        "Component {component_id} not ready (WASM not loaded)"
    )))
}

// Usage in handle_message
let runtime = self.wasm_runtime_mut()
    .ok_or_else(|| ComponentActorError::not_ready(&component_id_str))?;
```

**Error Types Handled:**
- `ComponentNotReady`: WASM not loaded (missing start())
- `FunctionNotFound`: Export doesn't exist
- `MulticodecError`: Invalid codec or truncated data
- `TypeConversionError`: Parameter/result type mismatch
- `ExecutionTrapped`: WASM function panicked
- `CapabilityDenied`: Security violation (stub for Block 4)

### 3.5 Performance Considerations

**Target:** >10,000 messages/second throughput (proven by MessageBroker: ~211ns routing)

**Optimizations:**
1. **Zero-copy where possible**: Pass `&[u8]` references, avoid cloning
2. **Pre-allocated buffers**: Reuse Vec for results
3. **Efficient varint parsing**: Single-pass decoding
4. **Cached exports**: WasmExports struct already caches function handles
5. **Async execution**: call_async() enables concurrent processing

**Measurement:**
- Add tracing spans with timing
- Benchmark in Phase 3 Task 3.4 (Performance Optimization)

---

## 4. Implementation Tasks

### 4.1 Task Breakdown (16-20 hours)

#### **Step 1: Multicodec Module** (3-4 hours)
**File:** `src/core/multicodec.rs`

**Deliverables:**
- `Codec` enum with From<u32> and Into<u32> traits
- `decode_multicodec(&[u8]) -> Result<(Codec, Vec<u8>)>`
- `encode_multicodec(Codec, &[u8]) -> Result<Vec<u8>>`
- Varint encoding/decoding utilities
- 8-10 unit tests (round-trip, edge cases, error cases)

**Acceptance Criteria:**
- ✅ All 3 codecs supported (Borsh, CBOR, JSON)
- ✅ Varint parsing handles 1-4 byte prefixes
- ✅ Invalid codec returns clear error
- ✅ Round-trip tests pass for all codecs
- ✅ 100% rustdoc coverage

**Implementation Details:**
```rust
// src/core/multicodec.rs

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    Borsh = 0x701,
    CBOR = 0x51,
    JSON = 0x0200,
}

#[derive(Debug, Error)]
pub enum MulticodecError {
    #[error("Empty data")]
    EmptyData,
    #[error("Truncated varint")]
    TruncatedVarint,
    #[error("Invalid varint encoding")]
    InvalidVarint,
    #[error("Unsupported codec: {0:#x}")]
    UnsupportedCodec(u32),
}

impl Codec {
    pub fn from_varint(varint: u32) -> Result<Self, MulticodecError> {
        match varint {
            0x701 => Ok(Codec::Borsh),
            0x51 => Ok(Codec::CBOR),
            0x0200 => Ok(Codec::JSON),
            v => Err(MulticodecError::UnsupportedCodec(v)),
        }
    }
}

pub fn decode_multicodec(data: &[u8]) -> Result<(Codec, Vec<u8>), MulticodecError> {
    if data.is_empty() {
        return Err(MulticodecError::EmptyData);
    }
    
    // Parse varint prefix
    let mut cursor = 0;
    let mut varint = 0u32;
    let mut shift = 0;
    
    loop {
        let byte = *data.get(cursor).ok_or(MulticodecError::TruncatedVarint)?;
        cursor += 1;
        
        varint |= ((byte & 0x7F) as u32) << shift;
        
        if byte & 0x80 == 0 {
            break;
        }
        
        shift += 7;
        if shift > 28 {
            return Err(MulticodecError::InvalidVarint);
        }
    }
    
    let codec = Codec::from_varint(varint)?;
    let payload = data[cursor..].to_vec();
    
    Ok((codec, payload))
}

pub fn encode_multicodec(codec: Codec, data: &[u8]) -> Result<Vec<u8>, MulticodecError> {
    let mut result = Vec::new();
    
    // Encode varint prefix
    let mut varint = codec as u32;
    while varint >= 0x80 {
        result.push(((varint & 0x7F) as u8) | 0x80);
        varint >>= 7;
    }
    result.push(varint as u8);
    
    // Append payload
    result.extend_from_slice(data);
    
    Ok(result)
}
```

---

#### **Step 2: Type Conversion Module** (3-4 hours)
**File:** `src/actor/type_conversion.rs`

**Deliverables:**
- `prepare_wasm_params(&[u8], &FuncType) -> Result<Vec<Val>>`
- `extract_wasm_results(&[Val]) -> Result<Vec<u8>>`
- 6-8 unit tests (simple types, edge cases)

**Acceptance Criteria:**
- ✅ I32, I64, F32, F64 parameter types supported
- ✅ Empty parameter list handled
- ✅ Single and multiple results supported
- ✅ Clear errors for unsupported types
- ✅ 100% rustdoc coverage

**Implementation Details:**
```rust
// src/actor/type_conversion.rs

use wasmtime::{Val, FuncType};
use crate::core::WasmError;

/// Convert decoded arguments to WASM Val parameters.
///
/// # Task 1.3 Scope
///
/// Supports simple parameter types (I32, I64, F32, F64). Complex types
/// (structs, arrays) will be added in Phase 2 Task 2.2.
///
/// # Arguments
///
/// * `decoded_args` - Byte array from multicodec deserialization
/// * `func_type` - Function signature from WASM export
///
/// # Returns
///
/// Vec of WASM Val parameters ready for call_async()
///
/// # Errors
///
/// Returns WasmError::ExecutionFailed if:
/// - Parameter count mismatch
/// - Unsupported parameter type
/// - Insufficient bytes for type
pub fn prepare_wasm_params(
    decoded_args: &[u8],
    func_type: &FuncType,
) -> Result<Vec<Val>, WasmError> {
    let param_types = func_type.params();
    
    // Handle no-parameter case
    if param_types.len() == 0 {
        return Ok(Vec::new());
    }
    
    // Task 1.3: Simple single-parameter case
    if param_types.len() == 1 {
        match param_types[0] {
            wasmtime::ValType::I32 => {
                if decoded_args.len() < 4 {
                    return Err(WasmError::execution_failed("Insufficient bytes for i32"));
                }
                let val = i32::from_le_bytes([
                    decoded_args[0],
                    decoded_args[1],
                    decoded_args[2],
                    decoded_args[3],
                ]);
                Ok(vec![Val::I32(val)])
            }
            wasmtime::ValType::I64 => {
                if decoded_args.len() < 8 {
                    return Err(WasmError::execution_failed("Insufficient bytes for i64"));
                }
                let val = i64::from_le_bytes([
                    decoded_args[0],
                    decoded_args[1],
                    decoded_args[2],
                    decoded_args[3],
                    decoded_args[4],
                    decoded_args[5],
                    decoded_args[6],
                    decoded_args[7],
                ]);
                Ok(vec![Val::I64(val)])
            }
            _ => Err(WasmError::execution_failed("Unsupported parameter type")),
        }
    } else {
        // Multi-parameter: Phase 2 Task 2.2
        Err(WasmError::execution_failed("Multi-parameter functions not yet supported"))
    }
}

/// Extract WASM results to byte array.
///
/// # Task 1.3 Scope
///
/// Supports simple return types (I32, I64, F32, F64). Complex types
/// will be added in Phase 2 Task 2.2.
///
/// # Arguments
///
/// * `results` - WASM Val results from call_async()
///
/// # Returns
///
/// Byte array ready for multicodec encoding
///
/// # Errors
///
/// Returns WasmError::ExecutionFailed if unsupported result type
pub fn extract_wasm_results(results: &[Val]) -> Result<Vec<u8>, WasmError> {
    if results.is_empty() {
        return Ok(Vec::new());
    }
    
    // Single result case (most common)
    if results.len() == 1 {
        match &results[0] {
            Val::I32(v) => Ok(v.to_le_bytes().to_vec()),
            Val::I64(v) => Ok(v.to_le_bytes().to_vec()),
            Val::F32(v) => Ok(v.to_bits().to_le_bytes().to_vec()),
            Val::F64(v) => Ok(v.to_bits().to_le_bytes().to_vec()),
            _ => Err(WasmError::execution_failed("Unsupported return type")),
        }
    } else {
        // Multiple results: concatenate as byte array
        let mut bytes = Vec::new();
        for val in results {
            match val {
                Val::I32(v) => bytes.extend_from_slice(&v.to_le_bytes()),
                Val::I64(v) => bytes.extend_from_slice(&v.to_le_bytes()),
                _ => return Err(WasmError::execution_failed("Unsupported multi-result type")),
            }
        }
        Ok(bytes)
    }
}
```

**Module Export:**
```rust
// src/actor/mod.rs
pub mod type_conversion;
```

---

#### **Step 3: Implement Invoke Message Handler** (4-5 hours)
**File:** `src/actor/actor_impl.rs` (lines 164-267)

**Deliverables:**
- Full `ComponentMessage::Invoke` implementation
- Multicodec integration
- Type conversion integration
- WASM function invocation
- Result encoding
- 5-7 tests

**Acceptance Criteria:**
- ✅ Borsh/CBOR/JSON codecs handled
- ✅ Function export lookup works
- ✅ Type conversion integrated
- ✅ call_async() properly awaited
- ✅ Results encoded with same codec as request
- ✅ All errors include component_id context
- ✅ Tracing logs at debug/trace levels

**Implementation Outline:**
```rust
ComponentMessage::Invoke { function, args } => {
    let component_id_str = self.component_id().as_str().to_string();
    
    debug!(
        component_id = %component_id_str,
        function = %function,
        args_len = args.len(),
        "Processing Invoke message"
    );

    // 1. Verify WASM loaded
    let _runtime = self.wasm_runtime_mut()
        .ok_or_else(|| ComponentActorError::not_ready(&component_id_str))?;

    // 2. Deserialize args using multicodec
    let (codec, decoded_args) = decode_multicodec(&args)
        .map_err(ComponentActorError::from)?;

    // 3. Get function export
    let runtime = self.wasm_runtime_mut().unwrap();
    let instance = *runtime.instance();
    let func = instance.get_func(&mut *runtime.store_mut(), &function)
        .ok_or_else(|| {
            ComponentActorError::from(WasmError::execution_failed(format!(
                "Function '{}' not found in component {}",
                function, component_id_str
            )))
        })?;

    // 4. Convert args to WASM parameters
    let func_type = func.ty(&mut *runtime.store_mut());
    let wasm_params = prepare_wasm_params(&decoded_args, &func_type)
        .map_err(ComponentActorError::from)?;

    // 5. Call WASM function
    let result_count = func_type.results().len();
    let mut results = vec![wasmtime::Val::I32(0); result_count];
    func.call_async(&mut *runtime.store_mut(), &wasm_params, &mut results)
        .await
        .map_err(|e| {
            ComponentActorError::from(WasmError::execution_failed(
                format!("WASM function '{}' trapped: {}", function, e)
            ))
        })?;

    // 6. Extract and encode results
    let result_bytes = extract_wasm_results(&results)
        .map_err(ComponentActorError::from)?;
    let encoded_result = encode_multicodec(codec, &result_bytes)
        .map_err(ComponentActorError::from)?;

    debug!(
        component_id = %component_id_str,
        result_len = encoded_result.len(),
        "Function execution completed"
    );

    // Reply stub (Phase 2 Task 2.3 will add ctx.reply())
    Ok(())
}
```

---

#### **Step 4: Implement InterComponent Message Handler** (2-3 hours)
**File:** `src/actor/actor_impl.rs` (lines 269-337)

**Deliverables:**
- Full `ComponentMessage::InterComponent` implementation
- Capability check placeholder
- handle-message export routing
- 3-4 tests

**Acceptance Criteria:**
- ✅ Capability check stub in place (with TODO for Block 4)
- ✅ handle-message export called if exists
- ✅ Missing export logged as warning
- ✅ Payload passed as-is (no deserialization)
- ✅ Tracing logs with sender context

---

#### **Step 5: Implement HealthCheck Message Handler** (1-2 hours)
**File:** `src/actor/actor_impl.rs` (lines 340-386)

**Deliverables:**
- Full `ComponentMessage::HealthCheck` implementation
- _health export detection
- HealthStatus determination
- 2-3 tests

**Acceptance Criteria:**
- ✅ Returns Unhealthy if WASM not loaded
- ✅ Returns Healthy if no _health export
- ✅ Detects _health export existence (call stub for Task 3.3)
- ✅ Logs health check results

---

#### **Step 6: Implement Shutdown Message Handler** (1 hour)
**File:** `src/actor/actor_impl.rs` (lines 388-403)

**Deliverables:**
- Full `ComponentMessage::Shutdown` implementation
- State transition to Stopping
- 1-2 tests

**Acceptance Criteria:**
- ✅ Sets state to ActorState::Stopping
- ✅ Logs shutdown request
- ✅ ctx.stop() stub documented for Phase 2

---

#### **Step 7: Implement Response Message Handlers** (30 min)
**File:** `src/actor/actor_impl.rs` (lines 405-435)

**Deliverables:**
- InvokeResult handler (log only)
- HealthStatus handler (log only)
- 2 tests

---

#### **Step 8: Implement Lifecycle Hooks** (1-2 hours)
**File:** `src/actor/actor_impl.rs` (lines 438-509)

**Deliverables:**
- `pre_start()` with WASM verification
- `post_stop()` with state transition
- Registry stubs documented
- 2-3 tests

**Acceptance Criteria:**
- ✅ pre_start warns if WASM not loaded
- ✅ post_stop sets state to Terminated
- ✅ Registry stubs documented with Block 6 reference

---

#### **Step 9: Comprehensive Testing** (3-4 hours)
**Files:**
- `src/actor/actor_impl.rs` (test module at bottom)
- `tests/actor_message_handling_tests.rs` (new integration test file)

**Test Coverage Target: 20-30 tests**

**Unit Tests (actor_impl.rs):**
1. test_actor_trait_compiles
2. test_message_trait_implemented
3. test_component_actor_error_display
4. test_component_actor_error_from_wasm_error
5. test_invoke_message_not_ready
6. test_health_check_message
7. test_shutdown_message
8. test_inter_component_message
9. test_multicodec_with_invoke_message
10. test_actor_pre_start
11. test_actor_post_stop

**Integration Tests (actor_message_handling_tests.rs):**
12. test_invoke_message_with_borsh_codec
13. test_invoke_message_with_cbor_codec
14. test_invoke_message_with_json_codec
15. test_invoke_function_not_found
16. test_invoke_wasm_trap
17. test_inter_component_message_with_export
18. test_inter_component_message_no_export
19. test_health_check_wasm_not_loaded
20. test_health_check_with_export
21. test_health_check_without_export
22. test_shutdown_transitions_state
23. test_invoke_result_logging
24. test_health_status_logging
25. test_lifecycle_hooks_called
26. test_multicodec_round_trip_all_codecs
27. test_type_conversion_i32_param
28. test_type_conversion_i64_param
29. test_type_conversion_multi_result
30. test_message_throughput_benchmark

---

#### **Step 10: Documentation & Code Review** (2-3 hours)

**Deliverables:**
- 100% rustdoc coverage for all new code
- Inline documentation for complex logic
- Module-level docs updated
- Code review checklist completed

**Documentation Requirements:**
- ✅ Every public function has rustdoc with examples
- ✅ Complex algorithms have inline comments
- ✅ Error cases documented
- ✅ Performance characteristics noted
- ✅ References to ADRs and task documents
- ✅ Future work clearly marked with TODOs

---

### 4.2 Implementation Order

**Day 1 (6-7 hours):**
1. Step 1: Multicodec module (3-4h)
2. Step 2: Type conversion module (3-4h)

**Day 2 (6-7 hours):**
3. Step 3: Invoke message handler (4-5h)
4. Step 4: InterComponent message handler (2-3h)

**Day 3 (4-6 hours):**
5. Step 5: HealthCheck message handler (1-2h)
6. Step 6: Shutdown message handler (1h)
7. Step 7: Response message handlers (30min)
8. Step 8: Lifecycle hooks (1-2h)

**Day 4 (3-4 hours):**
9. Step 9: Comprehensive testing (3-4h)

**Optional Day 5 (2-3 hours if needed):**
10. Step 10: Documentation & code review (2-3h)

**Total: 16-20 hours (3-5 days)**

---

## 5. Testing Strategy

### 5.1 Test Pyramid

```text
        ┌──────────────┐
        │ Integration  │  30% (9 tests)
        │   Tests      │  - End-to-end message handling
        └──────────────┘  - WASM function calls
       ┌────────────────┐
       │   Unit Tests   │  70% (21 tests)
       │                │  - Multicodec parsing
       └────────────────┘  - Type conversion
                           - Error handling
```

### 5.2 Unit Test Coverage

**Multicodec Module (8-10 tests):**
- ✅ decode_multicodec with valid Borsh/CBOR/JSON
- ✅ encode_multicodec round-trip for all codecs
- ✅ decode_multicodec with empty data (error)
- ✅ decode_multicodec with truncated varint (error)
- ✅ decode_multicodec with unsupported codec (error)
- ✅ encode_multicodec with large payload

**Type Conversion Module (6-8 tests):**
- ✅ prepare_wasm_params with i32 parameter
- ✅ prepare_wasm_params with i64 parameter
- ✅ prepare_wasm_params with no parameters
- ✅ prepare_wasm_params with insufficient bytes (error)
- ✅ extract_wasm_results with i32 result
- ✅ extract_wasm_results with i64 result
- ✅ extract_wasm_results with multiple results
- ✅ extract_wasm_results with empty results

**Actor Message Handling (11 tests in actor_impl.rs):**
- Already exist, documented in Step 9

### 5.3 Integration Test Coverage

**File:** `tests/actor_message_handling_tests.rs`

**Setup Helpers:**
```rust
async fn create_test_component() -> ComponentActor {
    // Load minimal WASM with test functions
    let mut actor = ComponentActor::new(...);
    actor.start().await.unwrap();
    actor
}

fn create_mock_actor_context() -> ActorContext<ComponentMessage, InMemoryMessageBroker> {
    // Stub ActorContext for testing (Phase 2 will provide real impl)
    // For Task 1.3, use minimal mock
}
```

**Test Cases (9 integration tests):**
1. **test_invoke_message_with_borsh_codec**: End-to-end Invoke with Borsh
2. **test_invoke_message_with_cbor_codec**: End-to-end Invoke with CBOR
3. **test_invoke_message_with_json_codec**: End-to-end Invoke with JSON
4. **test_invoke_function_not_found**: Missing export error handling
5. **test_invoke_wasm_trap**: WASM trap error handling
6. **test_inter_component_message_routing**: InterComponent to handle-message
7. **test_health_check_comprehensive**: Full health check flow
8. **test_lifecycle_hooks_integration**: pre_start → handle_message → post_stop
9. **test_message_throughput_benchmark**: Performance baseline (>10,000 msg/sec)

### 5.4 Performance Testing

**Benchmark:** `test_message_throughput_benchmark`

```rust
#[tokio::test]
async fn test_message_throughput_benchmark() {
    let mut actor = create_test_component().await;
    let mut ctx = create_mock_actor_context();
    
    let iterations = 10_000;
    let start = std::time::Instant::now();
    
    for i in 0..iterations {
        let args = encode_multicodec(Codec::Borsh, &i.to_le_bytes()).unwrap();
        let msg = ComponentMessage::Invoke {
            function: "test_func".to_string(),
            args,
        };
        actor.handle_message(msg, &mut ctx).await.unwrap();
    }
    
    let elapsed = start.elapsed();
    let msg_per_sec = (iterations as f64) / elapsed.as_secs_f64();
    
    println!("Throughput: {:.0} msg/sec", msg_per_sec);
    assert!(msg_per_sec > 10_000.0, "Throughput too low: {:.0}", msg_per_sec);
}
```

**Target:** >10,000 messages/second

---

## 6. Integration Points

### 6.1 Task 1.1 (ComponentActor Foundation)

**Dependencies:**
- ✅ ComponentActor struct
- ✅ ComponentMessage enum
- ✅ ActorState enum
- ✅ HealthStatus enum
- ✅ WasmRuntime struct
- ✅ WasmExports caching

**Integration:**
- Use `self.wasm_runtime_mut()` to access WASM
- Use `self.component_id()` for logging
- Use `self.state()` and `self.set_state()` for lifecycle
- Access `runtime.exports()` for cached function handles

### 6.2 Task 1.2 (Child Trait WASM Lifecycle)

**Dependencies:**
- ✅ Child::start() loads WASM
- ✅ WasmRuntime initialized with Engine, Store, Instance
- ✅ ComponentResourceLimiter enforcing limits

**Integration:**
- Assume `start()` called before `handle_message()`
- Verify WASM loaded: `self.wasm_runtime_mut().is_some()`
- Access Wasmtime primitives: `runtime.store_mut()`, `runtime.instance()`

### 6.3 Block 1 (WASM Runtime Layer)

**Dependencies:**
- ✅ Wasmtime Engine, Store, Instance
- ✅ Fuel metering and timeouts
- ✅ Memory limits enforced

**Integration:**
- Use `wasmtime::Func`, `wasmtime::Val` for function calls
- Leverage async execution: `func.call_async()`
- Respect resource limits (handled by ComponentResourceLimiter)

### 6.4 airssys-rt Actor System

**Dependencies:**
- ✅ Actor trait (Actor, handle_message, pre_start, post_stop)
- ✅ ActorContext (ctx parameter)
- ✅ MessageBroker trait

**Integration:**
- Implement `Actor for ComponentActor`
- Use `ActorContext` for messaging (Phase 2 will add full impl)
- Return `Result<(), ComponentActorError>` from all handlers

### 6.5 Future Tasks (Preparation)

**Phase 2 Task 2.1 (ActorSystem Integration):**
- Prepare for `ActorSystem::spawn(ComponentActor)`
- Document ctx.reply() stubs for later implementation

**Phase 2 Task 2.2 (Host Functions):**
- Document Linker integration points in Invoke handler
- Prepare for host function registration

**Phase 3 Task 3.3 (Health Check Full Impl):**
- Document _health export parsing stubs
- Prepare for HealthStatus deserialization

**Block 4 (Security Layer):**
- Document capability check stubs in InterComponent handler
- Prepare for fine-grained capability enforcement

**Block 6 (Component Registry):**
- Document registry stubs in pre_start/post_stop
- Prepare for component registration/deregistration

---

## 7. Success Criteria

### 7.1 Functional Requirements

✅ **FR-1: Invoke Message Handling**
- Invoke message deserializes multicodec args
- WASM function export located correctly
- Type conversion produces valid WASM parameters
- WASM function executes asynchronously
- Results extracted and encoded with same codec
- Errors include component_id context

✅ **FR-2: InterComponent Message Routing**
- InterComponent message routes to handle-message export
- Missing export handled gracefully (logged warning)
- Capability check placeholder in place

✅ **FR-3: HealthCheck Implementation**
- Health status determined correctly
- _health export detected if present
- Returns Unhealthy if WASM not loaded

✅ **FR-4: Lifecycle Hooks**
- pre_start verifies WASM loaded
- post_stop transitions to Terminated state
- Registry stubs documented

✅ **FR-5: Multicodec Support**
- Borsh, CBOR, JSON codecs supported
- Varint encoding/decoding correct
- Round-trip tests pass

### 7.2 Quality Requirements

✅ **QR-1: Zero Warnings**
- `cargo check` passes with zero warnings
- `cargo clippy --all-targets --all-features` passes with zero warnings
- All expected warnings suppressed with `#[expect(...)]` and reason

✅ **QR-2: Test Coverage**
- 20-30 comprehensive tests
- All critical paths covered
- Edge cases and error cases tested
- Performance benchmark included

✅ **QR-3: Documentation**
- 100% rustdoc coverage for public items
- Complex logic has inline comments
- Examples in rustdoc compile and run
- References to ADRs and task documents

✅ **QR-4: Code Quality**
- Follows workspace standards (§2.1-§6.3)
- Microsoft Rust Guidelines compliance
- Clean separation of concerns
- No code duplication

### 7.3 Performance Requirements

✅ **PR-1: Message Throughput**
- Target: >10,000 messages/second
- Baseline benchmark included in tests
- Tracing spans for profiling

✅ **PR-2: Multicodec Overhead**
- Target: <100μs for typical payloads
- Efficient varint parsing (single-pass)

✅ **PR-3: Type Conversion Overhead**
- Target: <10μs for simple types
- Zero-copy where possible

---

## 8. Risk Assessment

### 8.1 Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Multicodec implementation complexity | Medium | Medium | Start with simple varint parser, test extensively |
| Type conversion edge cases | Medium | High | Limit scope to simple types, defer complex cases to Phase 2 |
| WASM function call errors | Low | Medium | Comprehensive error handling with component context |
| Performance below target | Low | Medium | Benchmark early, optimize in Phase 3 if needed |
| ActorContext API changes | Low | Low | Use minimal stubs, full integration in Phase 2 |

### 8.2 Integration Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| airssys-rt API incompatibility | Low | High | Test against latest airssys-rt version |
| Wasmtime async behavior | Low | Medium | Follow Wasmtime docs, test async execution |
| Missing WASM exports | Medium | Low | Graceful fallback, clear error messages |

### 8.3 Schedule Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Underestimated complexity | Medium | Medium | 20% buffer in 16-20h estimate |
| Test failures during integration | Medium | Low | Incremental testing after each step |
| Documentation time | Low | Low | Write rustdoc during implementation |

---

## 9. Validation & Acceptance

### 9.1 Code Review Checklist

**Functional Completeness:**
- [ ] All 6 ComponentMessage variants handled
- [ ] Multicodec deserialization works for Borsh, CBOR, JSON
- [ ] Type conversion handles I32, I64, F32, F64
- [ ] WASM function invocation executes asynchronously
- [ ] Lifecycle hooks (pre_start, post_stop) implemented
- [ ] Error handling includes component_id context

**Code Quality:**
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings (--all-targets --all-features)
- [ ] No unwrap() outside test code
- [ ] No panic!() outside test code
- [ ] Proper error propagation with `?`
- [ ] Workspace standards compliance (§2.1-§6.3)

**Testing:**
- [ ] 20-30 tests passing
- [ ] All critical paths covered
- [ ] Edge cases tested
- [ ] Error cases tested
- [ ] Performance benchmark included
- [ ] No expected test failures (excluding Block 6 dependencies)

**Documentation:**
- [ ] 100% rustdoc coverage
- [ ] Examples in rustdoc compile
- [ ] Complex logic has inline comments
- [ ] TODOs clearly marked with task references
- [ ] ADR references included

**Performance:**
- [ ] Throughput benchmark >10,000 msg/sec
- [ ] Tracing spans added for profiling
- [ ] No obvious performance bottlenecks

### 9.2 Acceptance Tests

**Test 1: End-to-End Invoke**
```bash
cargo test test_invoke_message_with_borsh_codec -- --nocapture
```
Expected: Message processed, function executed, result encoded

**Test 2: Multicodec Round-Trip**
```bash
cargo test test_multicodec_round_trip_all_codecs -- --nocapture
```
Expected: All 3 codecs encode/decode correctly

**Test 3: Type Conversion**
```bash
cargo test test_type_conversion -- --nocapture
```
Expected: I32, I64 parameters converted correctly

**Test 4: Performance Baseline**
```bash
cargo test test_message_throughput_benchmark -- --nocapture
```
Expected: >10,000 messages/second throughput

**Test 5: Zero Warnings**
```bash
cargo clippy --all-targets --all-features
```
Expected: Exit code 0, zero warnings

---

## 10. Handoff & Next Steps

### 10.1 Deliverables

**Code:**
- ✅ `src/core/multicodec.rs` (new, ~200 lines)
- ✅ `src/actor/type_conversion.rs` (new, ~150 lines)
- ✅ `src/actor/actor_impl.rs` (updated, ~600 lines total)
- ✅ `tests/actor_message_handling_tests.rs` (new, ~400 lines)

**Documentation:**
- ✅ Rustdoc for all new public items
- ✅ Task completion summary (this document updated)
- ✅ Memory bank updated with progress

**Total New Code:** ~750 lines (implementation + tests)

### 10.2 Post-Task Activities

**Immediate (Same Session):**
1. Update progress.md with Task 1.3 completion
2. Update active-context.md with current status
3. Create completion summary document
4. Git commit with detailed message

**Next Task (Phase 1 Task 1.4):**
- **Task 1.4: Health Check Implementation** (8-10 hours)
  - Full _health export parsing
  - HealthStatus deserialization
  - Health check aggregation
  - Readiness probe integration

**Future Tasks:**
- **Phase 2 Task 2.1**: ActorSystem integration (spawn, messaging)
- **Phase 2 Task 2.2**: Host function registration (Wasmtime Linker)
- **Phase 2 Task 2.3**: Full ActorContext messaging (reply, send)

### 10.3 Known Limitations (Deferred Work)

**Documented in Code:**
1. **Phase 2 Task 2.2**: Complex type conversion (structs, arrays)
2. **Phase 2 Task 2.3**: Full ActorContext reply mechanism
3. **Phase 3 Task 3.3**: Full _health export parsing
4. **Block 4**: Fine-grained capability enforcement
5. **Block 6**: Component registry integration

**Technical Debt:**
- DEBT-WASM-004: Task 1.3 deferred implementation (already exists)
  - Multi-parameter function calls
  - Complex return types
  - Host function registration

---

## 11. References

### 11.1 Architecture Decision Records

- **ADR-WASM-001**: Inter-Component Communication Design (multicodec strategy)
- **ADR-WASM-002**: WASM Runtime Engine Selection (Wasmtime)
- **ADR-WASM-003**: Component Lifecycle Management (Actor + Child)
- **ADR-WASM-006**: Component Isolation and Sandboxing (dual trait pattern)
- **ADR-RT-004**: Actor and Child Trait Separation

### 11.2 Knowledge Documents

- **KNOWLEDGE-WASM-016**: Actor System Integration Implementation Guide (lines 438-668)
- **KNOWLEDGE-WASM-001**: Component Framework Architecture
- **KNOWLEDGE-WASM-005**: Messaging Architecture

### 11.3 Task Documents

- **WASM-TASK-004**: Block 3 - Actor System Integration (parent task)
- **Task 1.1 Completion**: ComponentActor Foundation (Nov 29, 2025)
- **Task 1.2 Completion**: Child Trait WASM Lifecycle (Nov 30, 2025)

### 11.4 Workspace Standards

- **§2.1**: Module organization (3-layer imports)
- **§4.3**: Module structure (declaration-only mod.rs)
- **§5.1**: Dependency management (workspace dependencies)
- **§6.1-§6.3**: Error handling, testing, documentation

### 11.5 External Documentation

- **Wasmtime Book**: https://docs.wasmtime.dev/
- **WASM Component Model**: https://github.com/WebAssembly/component-model
- **Multicodec Table**: https://github.com/multiformats/multicodec

---

## 12. Appendix

### 12.1 Glossary

- **ComponentActor**: Bridge between WASM components and airssys-rt actor system
- **Multicodec**: Self-describing binary format with codec identifier prefix
- **Type Conversion**: Translation between Rust values and WASM Val parameters
- **Handle-message Export**: WASM function receiving inter-component messages
- **ActorContext**: Runtime context for actor message passing

### 12.2 File Structure After Task 1.3

```text
airssys-wasm/
├── src/
│   ├── core/
│   │   ├── mod.rs (updated: export multicodec)
│   │   └── multicodec.rs (NEW: ~200 lines)
│   └── actor/
│       ├── mod.rs (updated: export type_conversion)
│       ├── component_actor.rs (no changes)
│       ├── actor_impl.rs (UPDATED: ~600 lines total)
│       ├── child_impl.rs (no changes)
│       └── type_conversion.rs (NEW: ~150 lines)
└── tests/
    └── actor_message_handling_tests.rs (NEW: ~400 lines)
```

### 12.3 Estimated Line Counts

| File | Before | After | Delta |
|------|--------|-------|-------|
| src/core/multicodec.rs | 0 | 200 | +200 |
| src/actor/type_conversion.rs | 0 | 150 | +150 |
| src/actor/actor_impl.rs | 297 | 600 | +303 |
| tests/actor_message_handling_tests.rs | 0 | 400 | +400 |
| **Total** | **297** | **1,350** | **+1,053** |

**Implementation:** ~753 lines  
**Tests:** ~400 lines  
**Total New Code:** ~1,053 lines

---

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2025-12-13 | Memory Bank Planner | Initial plan created based on Task 1.1 & 1.2 completion |

---

**END OF IMPLEMENTATION PLAN**
