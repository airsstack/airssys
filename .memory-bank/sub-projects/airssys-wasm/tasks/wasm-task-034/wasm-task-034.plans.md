# WASM-TASK-034: Implementation Plans

## Plan References
- **ADR-WASM-030:** Runtime Module Design (primary specification)
- **ADR-WASM-027:** WIT Interface Design
- **ADR-WASM-026:** Implementation Roadmap (Phase 5)

## Target Structure Reference

```
runtime/
├── mod.rs
├── engine.rs        # (WASM-TASK-031) ✅
├── loader.rs        # (WASM-TASK-032)
├── store.rs         # (WASM-TASK-033)
├── host_fn.rs       # ← THIS TASK
└── limiter.rs       # (WASM-TASK-035)
```

---

## Implementation Actions

### Action 1: Create `runtime/host_fn.rs`

**Objective:** Implement host function registration

**File:** `airssys-wasm/src/runtime/host_fn.rs`

**Specification (ADR-WASM-030 lines 442-498):**

```rust
//! Host function bindings for WASM components.

use wasmtime::component::Linker;

use super::engine::HostState;

/// Error type for host function operations
#[derive(Debug, Clone)]
pub enum HostFunctionError {
    /// Registration failed
    RegistrationFailed(String),
}

impl std::fmt::Display for HostFunctionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RegistrationFailed(msg) => write!(f, "Host function registration failed: {}", msg),
        }
    }
}

impl std::error::Error for HostFunctionError {}

/// Register all host functions with the linker
pub fn register_host_functions(
    linker: &mut Linker<HostState>,
) -> Result<(), HostFunctionError> {
    register_messaging_functions(linker)?;
    register_services_functions(linker)?;
    register_storage_functions(linker)?;

    Ok(())
}

/// Register host-messaging interface functions
fn register_messaging_functions(
    _linker: &mut Linker<HostState>,
) -> Result<(), HostFunctionError> {
    // Placeholder - will be implemented with wit-bindgen generated bindings
    //
    // Example structure:
    // linker.root().func_wrap(
    //     "host-messaging",
    //     "send",
    //     |caller: Caller<HostState>, target: ComponentId, payload: Vec<u8>| {
    //         // Implementation
    //     },
    // )?;

    Ok(())
}

/// Register host-services interface functions
fn register_services_functions(
    _linker: &mut Linker<HostState>,
) -> Result<(), HostFunctionError> {
    // log, current-time, generate-id, etc.
    // Placeholder implementation

    Ok(())
}

/// Register storage interface functions
fn register_storage_functions(
    _linker: &mut Linker<HostState>,
) -> Result<(), HostFunctionError> {
    // get, set, delete, list-keys, etc.
    // Placeholder implementation

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmtime::{Config, Engine};

    fn create_test_linker() -> Linker<HostState> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        let engine = Engine::new(&config).unwrap();
        Linker::new(&engine)
    }

    #[test]
    fn test_register_host_functions() {
        let mut linker = create_test_linker();
        let result = register_host_functions(&mut linker);
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_messaging_functions() {
        let mut linker = create_test_linker();
        let result = register_messaging_functions(&mut linker);
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_services_functions() {
        let mut linker = create_test_linker();
        let result = register_services_functions(&mut linker);
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_storage_functions() {
        let mut linker = create_test_linker();
        let result = register_storage_functions(&mut linker);
        assert!(result.is_ok());
    }

    #[test]
    fn test_host_function_error_display() {
        let err = HostFunctionError::RegistrationFailed("test".to_string());
        assert!(err.to_string().contains("registration failed"));
    }
}
```

### Action 2: Update `runtime/mod.rs`

Add `pub mod host_fn;` to module declarations.

---

## Verification Commands

```bash
cargo build -p airssys-wasm
cargo clippy -p airssys-wasm --all-targets -- -D warnings
cargo test -p airssys-wasm --lib runtime::host_fn
```

---

## Success Criteria

- [ ] register_host_functions() implemented
- [ ] register_messaging_functions() implemented
- [ ] register_services_functions() implemented
- [ ] register_storage_functions() implemented
- [ ] Build/Clippy pass with zero warnings
- [ ] All unit tests pass (5+ tests)

## Notes

> **wit-bindgen Integration**: Full host function implementations require wit-bindgen generated bindings.
> Current implementations are placeholders that succeed but don't register actual functions.
