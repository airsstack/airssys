# WASM-TASK-018: Implementation Plans

## Plan References

### Architecture Documents
- **ADR-WASM-028:** Core Module Structure (primary specification for core/runtime/ types)
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture (dependency inversion principles)
- **ADR-WASM-023:** Module Boundary Enforcement (import direction rules)
- **KNOWLEDGE-WASM-037:** Rebuild Architecture Clean-Slate (technical reference)
- **KNOWLEDGE-WASM-039:** Runtime Module Responsibility (core/runtime/ vs runtime/ distinction)

### System Patterns
- **Layer 1 Foundation:** core/runtime/ contains ONLY abstractions (traits and types)
- **Dependency Inversion:** runtime/ module implements these traits, component/ uses these traits
- **No External Imports:** core/ imports NOTHING except std (prevents cycles)
- **Module Responsibility:** core/runtime/ defines WHAT (traits), runtime/ defines HOW (implementations) - See KNOWLEDGE-WASM-039

### PROJECTS_STANDARD.md Compliance
- **§2.1:** 3-Layer Import Organization (std only for core/)
- **§2.2:** No FQN in type annotations (import types, use simple names)
- **§4.3:** Module Architecture Patterns (mod.rs only declarations and re-exports)
- **§6.1:** YAGNI Principles (implement only what's specified in ADR-WASM-028)
- **§6.2:** Avoid `dyn` Patterns (use generics for traits)
- **§6.4:** Implementation Quality Gates (zero warnings, comprehensive tests)

### Rust Guidelines Applied
- **M-DESIGN-FOR-AI:** Idiomatic APIs, thorough docs, testable code
- **M-MODULE-DOCS:** Module documentation with examples
- **M-ERRORS-CANONICAL-STRUCTS:** Use existing WasmError from core/errors/ (future task)
- **M-STATIC-VERIFICATION:** All lints enabled, clippy used
- **M-PUBLIC-DEBUG:** All public types implement Debug
- **M-FIRST-DOC-SENTENCE:** First doc sentence < 15 words

### Documentation Standards
- **Diátaxis Type:** Reference documentation for APIs
- **Quality:** Technical language, no marketing terms per documentation-quality-standards.md
- **Compliance:** Standards Compliance Checklist in task file

---

## Target Structure Reference

Per ADR-WASM-028 (lines 204-267):
```
core/runtime/
├── mod.rs           # Module declarations and re-exports
├── traits.rs        # RuntimeEngine, ComponentLoader traits
└── limits.rs        # ResourceLimits
```

---

## Implementation Actions

### Action 1: Create core/runtime/limits.rs
**Objective:** Implement ResourceLimits struct with execution resource constraints

**Steps:**
1. Create `src/core/runtime/limits.rs` with ResourceLimits struct
2. Implement Default trait with sensible defaults
3. Add comprehensive rustdoc documentation
4. Add unit tests for ResourceLimits creation and defaults

**Deliverables:**
- `src/core/runtime/limits.rs` with ResourceLimits struct

**ADR Constraints:**
- **ADR-WASM-028:** Must match specification at lines 247-267
  ```rust
  pub struct ResourceLimits {
      pub max_memory_bytes: u64,
      pub max_execution_time_ms: u64,
      pub max_fuel: Option<u64>,
  }
  ```
- **ADR-WASM-023:** core/ can ONLY import std (no external dependencies)
- **ADR-WASM-025:** This is a Layer 1 foundation type

**PROJECTS_STANDARD.md Compliance:**
- **§2.1:** Import only std
- **§2.2:** Use imported types (no FQN)
- **§6.1:** Implement only what ADR-WASM-028 specifies (YAGNI)

**Rust Guidelines:**
- **M-PUBLIC-DEBUG:** Implement Debug derive for ResourceLimits
- **M-CLONE:** Implement Clone derive for ResourceLimits (per ADR-WASM-028)
- **M-MODULE-DOCS:** Add module and struct documentation with examples

**Documentation:**
- **Diátaxis type:** Reference documentation
- **Quality:** Technical language describing resource constraints
- **Examples:** Show creating ResourceLimits with custom values

**Unit Testing Plan:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_limits_default_values() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_memory_bytes, 64 * 1024 * 1024); // 64MB
        assert_eq!(limits.max_execution_time_ms, 30_000); // 30 seconds
        assert_eq!(limits.max_fuel, None);
    }

    #[test]
    fn test_resource_limits_custom_values() {
        let limits = ResourceLimits {
            max_memory_bytes: 128 * 1024 * 1024, // 128MB
            max_execution_time_ms: 60_000, // 60 seconds
            max_fuel: Some(1_000_000),
        };

        assert_eq!(limits.max_memory_bytes, 128 * 1024 * 1024);
        assert_eq!(limits.max_execution_time_ms, 60_000);
        assert_eq!(limits.max_fuel, Some(1_000_000));
    }

    #[test]
    fn test_resource_limits_clone_creates_independent_copy() {
        let limits1 = ResourceLimits {
            max_memory_bytes: 100,
            max_execution_time_ms: 200,
            max_fuel: Some(300),
        };
        let limits2 = limits1.clone();

        assert_eq!(limits1, limits2);
        assert_eq!(limits1.max_memory_bytes, 100);
        assert_eq!(limits2.max_memory_bytes, 100);
    }

    #[test]
    fn test_resource_limits_debug_format() {
        let limits = ResourceLimits::default();
        let debug_str = format!("{:?}", limits);
        
        // Verify Debug trait is implemented and shows structure
        assert!(debug_str.contains("ResourceLimits"));
        assert!(debug_str.contains("max_memory_bytes"));
    }
}
```

**Verification:**
```bash
# Verify core/ only imports std
grep "use crate::" src/core/runtime/limits.rs
# Expected: no output (clean)

# Verify struct compiles
cargo build -p airssys-wasm

# Verify zero warnings
cargo clippy -p airssys-wasm --all-targets -- -D warnings

# Run unit tests
cargo test -p airssys-wasm core::runtime::limits
```

---

### Action 2: Create core/runtime/traits.rs
**Objective:** Implement RuntimeEngine and ComponentLoader trait abstractions

**Steps:**
1. Create `src/core/runtime/traits.rs` with trait definitions
2. Import ComponentId, ComponentHandle, ComponentMessage from core/component/
3. Import WasmError (will use placeholder until core/errors/ exists)
4. Add comprehensive rustdoc documentation for all trait methods
5. Add unit tests for trait bounds and method signatures

**Deliverables:**
- `src/core/runtime/traits.rs` with RuntimeEngine and ComponentLoader traits

**ADR Constraints:**
- **ADR-WASM-028:** Must match specification at lines 204-243
  - RuntimeEngine trait with load_component, unload_component, call_handle_message, call_handle_callback
  - ComponentLoader trait with load_bytes, validate methods
- **ADR-WASM-023:** core/ can import from core/component/ but NOT from other modules
- **ADR-WASM-025:** These are Layer 1 abstractions for dependency inversion

**PROJECTS_STANDARD.md Compliance:**
- **§2.1:** Layer 1: std imports, Layer 2: core/component/ imports
- **§2.2:** Import types from core/component/, use simple names (no FQN)
- **§4.3:** Traits only, no implementation logic in core/
- **§6.2:** Prefer generic constraints over `dyn` (use `impl Trait`)

**Rust Guidelines:**
- **M-DESIGN-FOR-AI:** Create idiomatic trait signatures
- **M-MODULE-DOCS:** Document each trait with examples
- **M-UNSAFE-IMPLIES-UB:** No unsafe needed for these trait definitions

**Documentation:**
- **Diátaxis type:** Reference documentation
- **Quality:** Technical language describing trait contracts
- **Examples:** Show trait usage patterns for implementers

**Unit Testing Plan:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::component::{ComponentId, ComponentHandle, ComponentMessage};
    use crate::core::messaging::payload::MessagePayload;

    // Mock implementation for testing trait bounds
    struct MockRuntimeEngine;

    impl RuntimeEngine for MockRuntimeEngine {
        fn load_component(&self, id: &ComponentId, bytes: &[u8]) 
            -> Result<ComponentHandle, MockWasmError> {
            Ok(ComponentHandle::new(id.clone(), 1))
        }

        fn unload_component(&self, _handle: &ComponentHandle) 
            -> Result<(), MockWasmError> {
            Ok(())
        }

        fn call_handle_message(
            &self,
            _handle: &ComponentHandle,
            _msg: &ComponentMessage,
        ) -> Result<Option<MessagePayload>, MockWasmError> {
            Ok(None)
        }

        fn call_handle_callback(
            &self,
            _handle: &ComponentHandle,
            _msg: &ComponentMessage,
        ) -> Result<(), MockWasmError> {
            Ok(())
        }
    }

    // Mock error type (placeholder until core/errors/ exists)
    #[derive(Debug)]
    enum MockWasmError {
        ComponentNotFound(String),
    }

    // Mock implementation for ComponentLoader
    struct MockComponentLoader;

    impl ComponentLoader for MockComponentLoader {
        fn load_bytes(&self, _id: &ComponentId) 
            -> Result<Vec<u8>, MockWasmError> {
            Ok(vec![1, 2, 3, 4])
        }

        fn validate(&self, _bytes: &[u8]) -> Result<(), MockWasmError> {
            Ok(())
        }
    }

    #[test]
    fn test_runtime_engine_load_component_returns_handle() {
        let engine = MockRuntimeEngine;
        let id = ComponentId::new("system", "test", "1");
        let bytes = vec![0x00, 0x01, 0x02];

        let result = engine.load_component(&id, &bytes);
        assert!(result.is_ok());
        
        let handle = result.unwrap();
        assert_eq!(handle.id().to_string_id(), "system/test/1");
    }

    #[test]
    fn test_runtime_engine_unload_component_succeeds() {
        let engine = MockRuntimeEngine;
        let id = ComponentId::new("system", "test", "1");
        let handle = ComponentHandle::new(id.clone(), 1);

        let result = engine.unload_component(&handle);
        assert!(result.is_ok());
    }

    #[test]
    fn test_runtime_engine_call_handle_message_returns_optional_payload() {
        let engine = MockRuntimeEngine;
        let id = ComponentId::new("system", "test", "1");
        let handle = ComponentHandle::new(id, 1);
        
        let sender_id = ComponentId::new("system", "sender", "1");
        let payload = MessagePayload::new(vec![1, 2, 3]);
        let msg = ComponentMessage::new(
            sender_id,
            payload,
            Default::default(),
        );

        let result = engine.call_handle_message(&handle, &msg);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_component_loader_load_bytes_returns_data() {
        let loader = MockComponentLoader;
        let id = ComponentId::new("system", "test", "1");

        let result = loader.load_bytes(&id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_component_loader_validate_succeeds() {
        let loader = MockComponentLoader;
        let bytes = vec![0x00, 0x61, 0x73, 0x6d]; // WASM magic

        let result = loader.validate(&bytes);
        assert!(result.is_ok());
    }

    #[test]
    fn test_trait_send_sync_bounds() {
        // Verify traits can be used in async contexts
        fn requires_send<T: Send>(_val: T) {}
        fn requires_sync<T: Sync>(_val: T) {}

        let engine = MockRuntimeEngine;
        requires_send(&engine);
        requires_sync(&engine);

        let loader = MockComponentLoader;
        requires_send(&loader);
        requires_sync(&loader);
    }
}
```

**Verification:**
```bash
# Verify core/runtime only imports std and core/component/
grep "use crate::" src/core/runtime/traits.rs
# Expected: Only "use crate::core::component" lines (no other crate:: imports)

# Verify traits compile
cargo build -p airssys-wasm

# Verify zero warnings
cargo clippy -p airssys-wasm --all-targets -- -D warnings

# Run unit tests
cargo test -p airssys-wasm core::runtime::traits
```

---

### Action 3: Create core/runtime/mod.rs
**Objective:** Module declarations and re-exports for runtime submodule

**Steps:**
1. Create `src/core/runtime/mod.rs` with module declarations
2. Re-export types from traits.rs and limits.rs
3. Add module documentation explaining runtime abstractions
4. Verify all exports are accessible

**Deliverables:**
- `src/core/runtime/mod.rs` with module structure

**ADR Constraints:**
- **ADR-WASM-028:** Must declare traits.rs and limits.rs submodules
- **ADR-WASM-023:** core/ can import from core/component/ only
- **ADR-WASM-025:** Module is Layer 1 foundation, only declarations

**PROJECTS_STANDARD.md Compliance:**
- **§4.3:** mod.rs contains ONLY declarations and re-exports
- **§2.1:** Import organization with std and core/component/ imports

**Rust Guidelines:**
- **M-MODULE-DOCS:** Add comprehensive module documentation
- **M-DOC-INLINE:** Use #[doc(inline)] for re-exports

**Documentation:**
- **Diátaxis type:** Reference documentation
- **Quality:** Technical language describing runtime abstractions

**Structure:**
```rust
//! # Runtime Module
//!
//! WASM runtime engine abstractions.
//!
//! This module contains trait abstractions that define how WASM components
//! are loaded, executed, and managed. The actual implementations
//! are provided by the **runtime/** module (Layer 2B).
//!
//! # Architecture
//!
//! This module is part of **core/** foundation (Layer 1). It contains
//! ONLY:
//!
//! - Trait definitions (RuntimeEngine, ComponentLoader)
//! - Resource constraint types (ResourceLimits)
//! - NO business logic
//! - NO external dependencies (only std and core/component/)
//!
//! # Purpose
//!
//! The runtime submodule provides foundational abstractions that enable:
//!
//! - WASM component loading (ComponentLoader trait)
//! - WASM component execution (RuntimeEngine trait)
//! - Resource limits enforcement (ResourceLimits struct)
//!
//! # Usage
//!
//! These types are imported and used by:
//!
//! - **runtime/**: Implements RuntimeEngine and ComponentLoader traits
//! - **component/**: Uses RuntimeEngine trait for WASM execution
//! - **system/**: Injects concrete runtime implementations into components
//!
//! # Examples
//!
//! ```rust
//! use airssys_wasm::core::runtime::{
//!     RuntimeEngine, ComponentLoader, ResourceLimits,
//! };
//!
//! // Create resource limits
//! let limits = ResourceLimits {
//!     max_memory_bytes: 128 * 1024 * 1024,
//!     max_execution_time_ms: 60_000,
//!     max_fuel: Some(1_000_000),
//! };
//!
//! // Use RuntimeEngine trait (implemented by runtime/ module)
//! fn load_component<E: RuntimeEngine>(
//!     engine: &E,
//!     id: &ComponentId,
//!     bytes: &[u8],
//! ) -> Result<ComponentHandle, WasmError> {
//!     engine.load_component(id, bytes)
//! }
//! ```

// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)

// Layer 2: Internal module imports (core/component/ only)
use crate::core::component::{ComponentHandle, ComponentId, ComponentMessage};
use crate::core::messaging::payload::MessagePayload;

// Module declarations (per PROJECTS_STANDARD.md §4.3)
pub mod limits;
pub mod traits;

// Re-exports for ergonomic API (per PROJECTS_STANDARD.md §4.3)
pub use limits::ResourceLimits;
pub use traits::{ComponentLoader, RuntimeEngine};
```

**Verification:**
```bash
# Verify mod.rs only has declarations and re-exports
# (no implementation code)
grep "fn \|struct \|impl \|enum" src/core/runtime/mod.rs
# Expected: no results (only mod and use statements)

# Verify imports are only std and core/component/
grep "use crate::" src/core/runtime/mod.rs
# Expected: Only core/component/ and core/messaging/ imports

# Verify module compiles
cargo build -p airssys-wasm

# Verify zero warnings
cargo clippy -p airssys-wasm --all-targets -- -D warnings
```

---

### Action 4: Update core/mod.rs to export runtime submodule
**Objective:** Add runtime submodule to core module exports

**Steps:**
1. Update `src/core/mod.rs` to declare runtime module
2. Re-export runtime types for ergonomic API
3. Update module documentation to include runtime/
4. Verify all exports are accessible

**Deliverables:**
- Updated `src/core/mod.rs` with runtime submodule

**ADR Constraints:**
- **ADR-WASM-028:** Core module must export all submodules
- **ADR-WASM-023:** core/mod.rs can import from core/component/, core/runtime/

**PROJECTS_STANDARD.md Compliance:**
- **§4.3:** mod.rs contains only declarations and re-exports
- **§2.2:** Re-export types for ergonomic API

**Rust Guidelines:**
- **M-MODULE-DOCS:** Update module documentation with runtime/ description

**Changes to core/mod.rs:**
```rust
//! # Core Module
//!
//! Core data types and abstractions shared by ALL other modules.
//!
//! This module contains foundational types that prevent circular dependencies.
//! Any type that multiple modules need should be defined here.
//!
//! # Submodules
//!
//! - **component/** - Component-related types (ComponentId, ComponentHandle, ComponentMessage, ComponentLifecycle)
//! - **runtime/** - WASM runtime abstractions (RuntimeEngine, ComponentLoader, ResourceLimits)
//!
//! # Architecture
//!
//! This is **Layer 1** of the architecture. Core imports NOTHING except `std` and its own submodules.
//! All other modules (security/, runtime/, component/, messaging/, system/) depend on core/.

// Module declarations (per PROJECTS_STANDARD.md §4.3)
pub mod component;
pub mod runtime;

// Re-exports for ergonomic API (per PROJECTS_STANDARD.md §4.3)
pub use component::*;
pub use runtime::*;
```

**Verification:**
```bash
# Verify core/mod.rs exports runtime module
grep "pub mod runtime" src/core/mod.rs
# Expected: Line with "pub mod runtime;"

# Verify re-exports include runtime types
grep "pub use runtime" src/core/mod.rs
# Expected: Line with "pub use runtime::*;"

# Verify core module compiles
cargo build -p airssys-wasm

# Verify runtime types are accessible through core
cargo test -p airssys-wasm --lib core::runtime
```

---

## Integration Testing Plan

**Note:** Integration tests for core/runtime/ will be deferred to WASM-TASK-024 (Core unit tests) as specified in ADR-WASM-026 Phase 3. This is because:
1. core/runtime/ contains only trait abstractions (no implementation)
2. Integration tests require concrete implementations from runtime/ module (Phase 5)
3. Unit tests with mocks verify trait contract compliance

**Future Integration Tests (WASM-TASK-024):**
- Test RuntimeEngine trait with actual WasmtimeEngine implementation
- Test ComponentLoader trait with actual ComponentLoader implementation
- Test resource limits with real WASM execution
- Test cross-module interaction (core/ → runtime/)

---

## Verification Commands

Run after ALL actions complete:

### 1. Build Check
```bash
# Verify package builds successfully
cargo build -p airssys-wasm
```

### 2. Module Boundary Check (ADR-WASM-023)
```bash
# Verify core/ only imports std and its own submodules
grep -rn "use crate::" src/core/ | grep -v "use crate::core::"
# Expected: no output (clean - no forbidden imports)

# Verify core/runtime/ only imports std and core/component/
grep "use crate::" src/core/runtime/
# Expected: Only core/component/ and core/messaging/ imports
```

### 3. Lint Check
```bash
# Verify zero compiler warnings
cargo clippy -p airssys-wasm --all-targets -- -D warnings
```

### 4. Test Check
```bash
# Run all unit tests for core/runtime/
cargo test -p airssys-wasm core::runtime

# Run all core/ unit tests
cargo test -p airssys-wasm core
```

### 5. Documentation Check
```bash
# Verify documentation builds
cargo doc -p airssys-wasm --no-deps
```

---

## Success Criteria

All of the following must be true:

- [ ] `src/core/runtime/limits.rs` exists with ResourceLimits struct
- [ ] `src/core/runtime/traits.rs` exists with RuntimeEngine and ComponentLoader traits
- [ ] `src/core/runtime/mod.rs` exists with module declarations and re-exports
- [ ] `src/core/mod.rs` updated to export runtime submodule
- [ ] All code follows 3-layer import organization (PROJECTS_STANDARD.md §2.1)
- [ ] All code uses simple type names, no FQN (PROJECTS_STANDARD.md §2.2)
- [ ] mod.rs files contain only declarations and re-exports (PROJECTS_STANDARD.md §4.3)
- [ ] Module boundary checks pass (ADR-WASM-023): core/ only imports std
- [ ] Traits match ADR-WASM-028 specification exactly
- [ ] All public types have rustdoc documentation with examples (M-MODULE-DOCS)
- [ ] All public types implement Debug trait (M-PUBLIC-DEBUG)
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings
- [ ] All unit tests pass
- [ ] Code compiles successfully

---

## Dependency Information

### Upstream Dependencies
- **WASM-TASK-017** (Create core/component/ submodule) - must be complete

### Downstream Dependencies
This task enables:
- **WASM-TASK-019** (Create core/messaging/ submodule)
- **WASM-TASK-020** (Create core/security/ submodule)
- **WASM-TASK-024** (Core unit tests)
- Phase 5 runtime implementation (WASM-TASK-031 to 036)

---

## Standards Compliance Checklist

- [ ] **§2.1 3-Layer Import Organization** - Only std and core/component/ imports
- [ ] **§2.2 No FQN in Type Annotations** - Import types, use simple names
- [ ] **§4.3 Module Architecture Patterns** - mod.rs only declarations
- [ ] **ADR-WASM-028** - Core module structure compliance
- [ ] **ADR-WASM-025** - Clean-slate rebuild architecture
- [ ] **ADR-WASM-023** - Module boundary enforcement (no forbidden imports)
- [ ] **KNOWLEDGE-WASM-037** - Technical reference alignment
- [ ] **M-DESIGN-FOR-AI** - Idiomatic APIs with thorough docs
- [ ] **M-MODULE-DOCS** - Module documentation with examples
- [ ] **M-PUBLIC-DEBUG** - All public types implement Debug
- [ ] **Documentation Quality Standards** - Technical language, no hyperbole
- [ ] **Diátaxis Guidelines** - Reference documentation type
- [ ] **Testing Mandate** - Unit tests for all types

---

**This plan provides detailed step-by-step actions for creating the core/runtime/ submodule with comprehensive testing and verification.**
