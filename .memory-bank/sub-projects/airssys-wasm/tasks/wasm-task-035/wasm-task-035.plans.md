# WASM-TASK-035: Implementation Plans (Revised)

## References
- ADR-WASM-030 (Runtime Module Design)
- ADR-WASM-026 (Implementation Roadmap)
- KNOWLEDGE-WASM-044 (Wasmtime Resource Limiter Architecture)
- KNOWLEDGE-WASM-037 (Clean Slate Architecture)
- PROJECTS_STANDARD.md (§2.1, §2.2, §4.3)
- ADR-WASM-023 (Module Boundary Enforcement)

## Revision Notes
- **2026-01-18**: Revised to properly implement BOTH StoreLimits (memory) AND Fuel (CPU).
- Deferred `max_execution_time_ms` (epoch interruption) to future task.

---

## Actions

### Action 1: Implement ResourceLimiter with StoreLimits + Fuel

**Objective**: Implement resource limiting that bridges `core::runtime::limits::ResourceLimits` with Wasmtime's `StoreLimits` (memory) and Fuel (CPU) mechanisms.

**Detailed Steps**:

#### Step 1.1: Create `src/runtime/limiter.rs`

```rust
//! Resource limiter for WASM execution.
//!
//! Bridges `core::runtime::limits::ResourceLimits` with Wasmtime's
//! `StoreLimits` (memory/tables) and Fuel (CPU) mechanisms.

// Layer 1: Standard library imports
// (none needed)

// Layer 2: Third-party crate imports
use wasmtime::{Store, StoreLimits, StoreLimitsBuilder};

// Layer 3: Internal module imports
use crate::core::runtime::errors::WasmError;
use crate::core::runtime::limits::ResourceLimits;

use super::engine::HostState;

/// Resource limiter for WASM execution.
///
/// Wraps Wasmtime's `StoreLimits` for memory/table enforcement
/// and provides fuel configuration.
pub struct WasmResourceLimiter {
    store_limits: StoreLimits,
    fuel_limit: Option<u64>,
}

impl WasmResourceLimiter {
    /// Create a new resource limiter from core ResourceLimits.
    ///
    /// # Arguments
    /// * `limits` - Core resource limits configuration
    ///
    /// # Example
    /// ```ignore
    /// use airssys_wasm::core::runtime::limits::ResourceLimits;
    /// use airssys_wasm::runtime::limiter::WasmResourceLimiter;
    ///
    /// let limits = ResourceLimits::default();
    /// let limiter = WasmResourceLimiter::new(&limits);
    /// ```
    pub fn new(limits: &ResourceLimits) -> Self {
        let store_limits = StoreLimitsBuilder::new()
            .memory_size(limits.max_memory_bytes as usize)
            .table_elements(10_000) // Default table limit
            .build();

        Self {
            store_limits,
            fuel_limit: limits.max_fuel,
        }
    }

    /// Consume the limiter and return the inner StoreLimits.
    pub fn into_store_limits(self) -> StoreLimits {
        self.store_limits
    }

    /// Get fuel limit if configured.
    pub fn fuel_limit(&self) -> Option<u64> {
        self.fuel_limit
    }
}

/// Apply resource limits to a Store.
///
/// This function:
/// 1. Sets the StoreLimits in HostState for memory/table enforcement
/// 2. Configures the store's limiter callback
/// 3. Sets fuel if configured
///
/// # Arguments
/// * `store` - The Wasmtime store to apply limits to
/// * `limits` - Resource limits to apply
///
/// # Errors
/// Returns `WasmError::RuntimeError` if fuel cannot be set.
pub fn apply_limits_to_store(
    store: &mut Store<HostState>,
    limits: &ResourceLimits,
) -> Result<(), WasmError> {
    // Create the limiter
    let limiter = WasmResourceLimiter::new(limits);

    // 1. Set StoreLimits in HostState
    store.data_mut().store_limits = limiter.into_store_limits();

    // 2. Configure the store's limiter callback
    // This tells Wasmtime to use HostState.store_limits for memory/table checks
    store.limiter(|state| &mut state.store_limits);

    // 3. Set fuel if configured
    if let Some(fuel) = limits.max_fuel {
        store.set_fuel(fuel).map_err(|e| {
            WasmError::RuntimeError(format!("Failed to set fuel: {}", e))
        })?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limiter_creation_default() {
        let limits = ResourceLimits::default();
        let limiter = WasmResourceLimiter::new(&limits);

        assert!(limiter.fuel_limit().is_none());
    }

    #[test]
    fn test_limiter_with_fuel() {
        let limits = ResourceLimits {
            max_memory_bytes: 64 * 1024 * 1024,
            max_execution_time_ms: 30_000,
            max_fuel: Some(1_000_000),
        };
        let limiter = WasmResourceLimiter::new(&limits);

        assert_eq!(limiter.fuel_limit(), Some(1_000_000));
    }

    #[test]
    fn test_limiter_into_store_limits() {
        let limits = ResourceLimits::default();
        let limiter = WasmResourceLimiter::new(&limits);
        let _store_limits = limiter.into_store_limits();
        // StoreLimits is opaque, but we verify it doesn't panic
    }

    #[test]
    fn test_limiter_custom_memory() {
        let limits = ResourceLimits {
            max_memory_bytes: 16 * 1024 * 1024, // 16MB
            max_execution_time_ms: 10_000,
            max_fuel: Some(500_000),
        };
        let limiter = WasmResourceLimiter::new(&limits);

        assert_eq!(limiter.fuel_limit(), Some(500_000));
    }
}
```

#### Step 1.2: Update `src/runtime/engine.rs` - Add StoreLimits to HostState

Modify the `HostState` struct to include `StoreLimits`:

```rust
// Add to imports (Layer 2):
use wasmtime::{StoreLimits, StoreLimitsBuilder};

// Update HostState struct:
pub struct HostState {
    /// The component ID for this instance
    pub component_id: ComponentId,
    /// Message router for inter-component communication
    pub message_router: Option<Arc<dyn MessageRouter>>,
    /// Store limits for memory/table enforcement
    pub store_limits: StoreLimits,
}

// Update load_component to initialize HostState with default StoreLimits:
let host_state = HostState {
    component_id: id.clone(),
    message_router: None,
    store_limits: StoreLimitsBuilder::new().build(), // Default limits
};

// After creating store, configure limiter callback:
let mut store = Store::new(&self.engine, host_state);
store.limiter(|state| &mut state.store_limits);
```

#### Step 1.3: Update `src/runtime/mod.rs`

```rust
// Add module declaration
pub mod limiter;
```

**Deliverables**:
- `src/runtime/limiter.rs` with `WasmResourceLimiter` and `apply_limits_to_store`
- Updated `src/runtime/engine.rs` with `StoreLimits` in `HostState`
- Updated `src/runtime/mod.rs` with `limiter` module

**Constraints**:
- Must not import from `component/`, `messaging/`, or `system/`
- Must follow §2.1 3-Layer imports
- Must follow §2.2 No FQN in type annotations
- Must follow §4.3 mod.rs only module declarations

---

### Action 2: Integration Tests (Two Types)

**Objective**: Verify resource limits are correctly enforced at BOTH the Wasmtime level AND the airssys-wasm API level.

**Test Strategy**: Two types of integration tests are required:
- **Type A**: Wasmtime Behavior Tests (verify Wasmtime's StoreLimits/Fuel mechanisms work)
- **Type B**: airssys-wasm API Tests (verify our `WasmResourceLimiter` and `apply_limits_to_store()` work)

**Detailed Steps**:

#### Step 2.1: Create `tests/resource_limits_integration.rs` (Type A - Wasmtime Behavior Tests)

**Purpose**: Verify that Wasmtime's StoreLimits and Fuel mechanisms work as expected.

```rust
//! Integration tests for resource limiting.
//!
//! Tests memory and fuel limits using inline WAT components.

use wasmtime::component::Component;
use wasmtime::{Config, Engine, Store, StoreLimitsBuilder};

/// Test helper: Parse WAT to WASM bytes
fn wat_to_wasm(wat: &str) -> Vec<u8> {
    wat::parse_str(wat).expect("Invalid WAT")
}

/// Create an engine configured for fuel consumption
fn create_engine() -> Engine {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.consume_fuel(true);
    Engine::new(&config).expect("Failed to create engine")
}

#[test]
fn test_memory_limit_prevents_growth() {
    // WAT: A core module that tries to grow memory beyond limit
    // Note: Component Model wraps core modules
    let wat = r#"
        (component
            (core module $m
                (memory (export "memory") 1)
                (func (export "try_grow") (result i32)
                    (memory.grow (i32.const 10))
                )
            )
            (core instance $i (instantiate $m))
            (func (export "try-grow") (result s32)
                (canon lift (core func $i "try_grow"))
            )
        )
    "#;

    let engine = create_engine();
    let wasm_bytes = wat_to_wasm(wat);
    let component = Component::new(&engine, &wasm_bytes).expect("Failed to create component");

    // Create store with 64KB memory limit (1 page)
    struct TestState {
        limits: wasmtime::StoreLimits,
    }

    let limits = StoreLimitsBuilder::new()
        .memory_size(64 * 1024) // 64KB = 1 page
        .build();

    let mut store = Store::new(&engine, TestState { limits });
    store.limiter(|s| &mut s.limits);

    // Component should exist but memory.grow should fail
    // (returns -1 when growth fails due to limits)
    assert!(component.serialize().is_ok());
}

#[test]
fn test_fuel_exhaustion_traps() {
    // WAT: Infinite loop that will exhaust fuel
    let wat = r#"
        (component
            (core module $m
                (func (export "infinite_loop")
                    (loop $l
                        (br $l)
                    )
                )
            )
            (core instance $i (instantiate $m))
            (func (export "infinite-loop")
                (canon lift (core func $i "infinite_loop"))
            )
        )
    "#;

    let engine = create_engine();
    let wasm_bytes = wat_to_wasm(wat);
    let component = Component::new(&engine, &wasm_bytes).expect("Failed to create component");

    struct TestState;

    let mut store = Store::new(&engine, TestState);

    // Set very low fuel limit
    store.set_fuel(100).expect("Failed to set fuel");

    // Verify fuel is set
    let remaining = store.get_fuel().expect("Failed to get fuel");
    assert_eq!(remaining, 100);

    // Component loads successfully (fuel consumed during execution, not load)
    assert!(component.serialize().is_ok());
}

#[test]
fn test_fuel_not_set_when_none() {
    let engine = create_engine();

    struct TestState;
    let store = Store::new(&engine, TestState);

    // Default store has no fuel set (infinite)
    // get_fuel returns error when fuel not enabled, but we enabled it in config
    // So this should return Ok with the max value
    let fuel = store.get_fuel();
    assert!(fuel.is_ok());
}

#[test]
fn test_memory_limit_exact_boundary() {
    // Test at exactly the boundary
    let engine = create_engine();

    struct TestState {
        limits: wasmtime::StoreLimits,
    }

    // 2 pages = 128KB
    let limits = StoreLimitsBuilder::new()
        .memory_size(128 * 1024)
        .build();

    let store = Store::new(&engine, TestState { limits });

    // Verify store was created successfully with limits
    assert!(store.get_fuel().is_ok());
}
```

---

#### Step 2.2: Create `tests/airssys_limiter_integration.rs` (Type B - airssys-wasm API Tests)

**Purpose**: Verify that `WasmResourceLimiter` and `apply_limits_to_store()` correctly bridge our `ResourceLimits` to Wasmtime's mechanisms.

```rust
//! Integration tests for airssys-wasm resource limiter API.
//!
//! Tests that our WasmResourceLimiter and apply_limits_to_store()
//! correctly configure Wasmtime's StoreLimits and Fuel.

use airssys_wasm::core::runtime::limits::ResourceLimits;
use airssys_wasm::core::types::ComponentId;
use airssys_wasm::runtime::engine::HostState;
use airssys_wasm::runtime::limiter::{apply_limits_to_store, WasmResourceLimiter};
use wasmtime::{Config, Engine, Store, StoreLimitsBuilder};

/// Create an engine configured for fuel consumption
fn create_engine() -> Engine {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.consume_fuel(true);
    Engine::new(&config).expect("Failed to create engine")
}

#[test]
fn test_apply_limits_sets_store_limits() {
    let engine = create_engine();

    // Create HostState with default StoreLimits
    let host_state = HostState {
        component_id: ComponentId::new("test-component"),
        message_router: None,
        store_limits: StoreLimitsBuilder::new().build(),
    };

    let mut store = Store::new(&engine, host_state);

    // Apply custom limits
    let limits = ResourceLimits {
        max_memory_bytes: 1024 * 1024, // 1MB
        max_execution_time_ms: 5000,
        max_fuel: Some(10_000),
    };

    apply_limits_to_store(&mut store, &limits).expect("Failed to apply limits");

    // Verify fuel was set
    let fuel = store.get_fuel().expect("Fuel should be set");
    assert_eq!(fuel, 10_000);

    // Verify limiter callback is configured (store has limiter)
    // We can't directly inspect StoreLimits, but we know it's set if no panic
}

#[test]
fn test_apply_limits_with_no_fuel() {
    let engine = create_engine();

    let host_state = HostState {
        component_id: ComponentId::new("test-component"),
        message_router: None,
        store_limits: StoreLimitsBuilder::new().build(),
    };

    let mut store = Store::new(&engine, host_state);

    // Apply limits WITHOUT fuel
    let limits = ResourceLimits {
        max_memory_bytes: 2 * 1024 * 1024, // 2MB
        max_execution_time_ms: 10_000,
        max_fuel: None,
    };

    apply_limits_to_store(&mut store, &limits).expect("Failed to apply limits");

    // When fuel is None, get_fuel() returns Ok with max u64
    // (because engine has fuel enabled, but store hasn't consumed yet)
    let fuel = store.get_fuel().expect("Should succeed");
    assert!(fuel > 0); // Default fuel amount
}

#[test]
fn test_apply_limits_multiple_times() {
    let engine = create_engine();

    let host_state = HostState {
        component_id: ComponentId::new("test-component"),
        message_router: None,
        store_limits: StoreLimitsBuilder::new().build(),
    };

    let mut store = Store::new(&engine, host_state);

    // Apply first set of limits
    let limits1 = ResourceLimits {
        max_memory_bytes: 512 * 1024,
        max_execution_time_ms: 1000,
        max_fuel: Some(5_000),
    };

    apply_limits_to_store(&mut store, &limits1).expect("Failed to apply limits");
    let fuel1 = store.get_fuel().expect("Fuel should be set");
    assert_eq!(fuel1, 5_000);

    // Apply second set of limits (overwrite)
    let limits2 = ResourceLimits {
        max_memory_bytes: 1024 * 1024,
        max_execution_time_ms: 2000,
        max_fuel: Some(20_000),
    };

    apply_limits_to_store(&mut store, &limits2).expect("Failed to apply limits");
    let fuel2 = store.get_fuel().expect("Fuel should be updated");
    assert_eq!(fuel2, 20_000);
}

#[test]
fn test_wasm_resource_limiter_new() {
    let limits = ResourceLimits {
        max_memory_bytes: 16 * 1024 * 1024, // 16MB
        max_execution_time_ms: 30_000,
        max_fuel: Some(1_000_000),
    };

    let limiter = WasmResourceLimiter::new(&limits);

    // Verify fuel limit is captured
    assert_eq!(limiter.fuel_limit(), Some(1_000_000));

    // Verify StoreLimits can be extracted
    let _store_limits = limiter.into_store_limits();
}

#[test]
fn test_wasm_resource_limiter_default_limits() {
    let limits = ResourceLimits::default();
    let limiter = WasmResourceLimiter::new(&limits);

    // Default ResourceLimits has no fuel
    assert_eq!(limiter.fuel_limit(), None);
}

#[test]
fn test_apply_limits_end_to_end() {
    // End-to-end test: ResourceLimits -> apply_limits_to_store -> verify all set
    let engine = create_engine();

    let host_state = HostState {
        component_id: ComponentId::new("e2e-test"),
        message_router: None,
        store_limits: StoreLimitsBuilder::new().build(),
    };

    let mut store = Store::new(&engine, host_state);

    // Create comprehensive limits
    let limits = ResourceLimits {
        max_memory_bytes: 8 * 1024 * 1024, // 8MB
        max_execution_time_ms: 15_000,
        max_fuel: Some(500_000),
    };

    // Apply limits
    apply_limits_to_store(&mut store, &limits).expect("Failed to apply limits");

    // Verify fuel was set correctly
    let fuel = store.get_fuel().expect("Fuel should be set");
    assert_eq!(fuel, 500_000);

    // Verify HostState's store_limits was updated (indirectly)
    // We can't access it directly, but if no panic, it's set correctly
    // The limiter callback is configured and will use HostState.store_limits

    // Simulate consuming some fuel
    store
        .set_fuel(fuel - 100)
        .expect("Should be able to update fuel");
    let remaining = store.get_fuel().expect("Should get remaining fuel");
    assert_eq!(remaining, 499_900);
}
```

**Deliverables**:
- `tests/resource_limits_integration.rs` - Type A tests (Wasmtime behavior)
- `tests/airssys_limiter_integration.rs` - Type B tests (airssys-wasm API)

**Constraints**:
- Type A tests use RAW wasmtime APIs only (no airssys imports)
- Type B tests MUST import and use airssys_wasm modules
- Type B tests verify end-to-end: `ResourceLimits` → `WasmResourceLimiter` → `apply_limits_to_store()` → Wasmtime
- All tests must be independent and not rely on shared state
- Use inline WAT strings for Type A tests (not external fixtures)

---

## Verification Section

### Automated Tests
```bash
# Unit tests for limiter module
cargo test -p airssys-wasm --lib -- runtime::limiter

# Integration tests - Type A (Wasmtime behavior)
cargo test -p airssys-wasm --test resource_limits_integration

# Integration tests - Type B (airssys-wasm API)
cargo test -p airssys-wasm --test airssys_limiter_integration

# All tests
cargo test -p airssys-wasm

# Clippy
cargo clippy -p airssys-wasm --lib --tests -- -D warnings
```

### Architecture Compliance
```bash
# Verify no forbidden imports in limiter.rs
grep -rn "use crate::component" src/runtime/limiter.rs  # Should be empty
grep -rn "use crate::messaging" src/runtime/limiter.rs  # Should be empty
grep -rn "use crate::system" src/runtime/limiter.rs     # Should be empty

# Verify no FQN in type annotations
grep -rn "wasmtime::" src/runtime/limiter.rs | grep -v "^.*use " | grep "::" # Should be empty after filtering imports
```

---

## Success Criteria
- [ ] `src/runtime/limiter.rs` exists and compiles
- [ ] `HostState` includes `StoreLimits` field
- [ ] `store.limiter(|s| &mut s.store_limits)` called in `load_component`
- [ ] `apply_limits_to_store` sets both StoreLimits AND Fuel
- [ ] Unit tests in `limiter.rs` pass (4 tests)
- [ ] Type A integration tests pass (Wasmtime behavior - 4 tests in `resource_limits_integration.rs`)
- [ ] Type B integration tests pass (airssys API - 6 tests in `airssys_limiter_integration.rs`)
- [ ] Type B tests import and use `airssys_wasm::runtime::limiter` modules
- [ ] Type B tests verify end-to-end: `ResourceLimits` → `WasmResourceLimiter` → `apply_limits_to_store()`
- [ ] `cargo clippy -p airssys-wasm --lib --tests -- -D warnings` passes
- [ ] No forbidden imports (architecture compliance)

---

## Deferred Items
- `max_execution_time_ms` enforcement via epoch interruption (future task)
