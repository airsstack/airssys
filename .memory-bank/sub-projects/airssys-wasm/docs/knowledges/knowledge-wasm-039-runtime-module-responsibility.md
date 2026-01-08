# KNOWLEDGE-WASM-039: Runtime Module Responsibility and Architecture

**Document ID:** KNOWLEDGE-WASM-039  
**Created:** 2026-01-08  
**Category:** Architecture / Module Design / Runtime  
**Maturity:** Stable  
**Severity:** 🟡 **IMPORTANT - Implementation Reference**

## Overview

This document clarifies the responsibility and architectural distinction between `core/runtime/` (Layer 1 foundation abstractions) and `runtime/` (Layer 3B WASM execution engine). It provides the definitive reference for understanding what belongs in each module and how they work together in the clean-slate rebuild architecture.

## Context

### Problem Statement

Developers often confuse `core/runtime/` and `runtime/` modules, leading to:
1. **Wrong code placement:** Implementations in `core/` or abstractions in `runtime/`
2. **Dependency violations:** `runtime/` importing from `core/` (wrong direction)
3. **Misunderstood boundaries:** Unclear what each module owns

### Scope

This knowledge applies to:
- **WASM-TASK-018** and all Phase 3 core module tasks (core/runtime/, etc.)
- **Phase 5** runtime/ execution engine tasks (WasmtimeEngine, ComponentLoader)
- All future development touching runtime-related code

### Prerequisites

- **KNOWLEDGE-WASM-037:** Rebuild Architecture Clean-Slate Design (six-module architecture)
- **KNOWLEDGE-WASM-038:** Component Module Responsibility and Architecture (core/component/ vs component/)
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture (layer organization)
- **ADR-WASM-023:** Module Boundary Enforcement (dependency rules)

## Technical Content

### Core Concepts: Two-Layer Distinction

The runtime-related functionality is split across **TWO SEPARATE MODULES** at different architectural layers:

```
┌─────────────────────────────────────────────────────────────┐
│                    airssys-wasm                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  core/runtime/ (LAYER 1)   runtime/ (LAYER 3B)              │
│                                                             │
│  Purpose: Abstractions      Purpose: WASM Execution         │
│  Trait Definitions         & Concrete Implementation       │
│                                                             │
│  Ownership:                  Ownership:                      │
│  - RuntimeEngine trait      - WasmtimeEngine               │
│  - ComponentLoader trait    - ComponentLoader impl         │
│  - ResourceLimits struct    - Host functions               │
│                                                             │
│  Dependencies:               Dependencies:                   │
│  - std + core/component/    - core/runtime/ (traits)       │
│  - core/messaging/          - core/component/ (types)       │
│                             - security/ (validation)       │
│                                                             │
│  Implementation:              Implementation:                │
│  - Trait definitions        - Wasmtime integration         │
│  - Data structs             - Component loading            │
│  - NO implementations       - Host function impls         │
│                             - Resource limit enforcement   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Core Module: `core/runtime/` (Layer 1)

**Purpose:** Foundation trait abstractions and resource limit structures that define **WHAT** a runtime engine must do.

**Key Types:**

| Type | Purpose | Implementation |
|------|---------|----------------|
| `RuntimeEngine` trait | Abstraction for WASM execution engine | Trait definition with `load_component`, `unload_component`, `call_handle_message` methods |
| `ComponentLoader` trait | Abstraction for loading WASM components | Trait definition with `load`, `validate`, `get_component_type` methods |
| `ResourceLimits` struct | Execution constraints (memory, time, fuel) | Data struct with configurable limits |

**Architectural Rules:**

1. **Imports ONLY `std` and own submodules** - Can import from `core/component/` and `core/messaging/` (same layer)
2. **NO implementations** - Only trait definitions and data structures
3. **NO external dependencies** - No Wasmtime, no airssys-rt, no airssys-osl
4. **Trait definitions only** - Implementations live in `runtime/` module

**Dependency Flow:**
```
All modules ───────────────────► core/runtime/
(runtime/, security/, actor/, component/, system/)
    ↓ all depend on these trait abstractions
```

**File Structure:**
```
core/runtime/
├── mod.rs           # Module declarations and re-exports
├── traits.rs        # RuntimeEngine, ComponentLoader traits
└── limits.rs        # ResourceLimits struct
```

### Execution Module: `runtime/` (Layer 3B)

**Purpose:** Concrete WASM execution engine that implements the traits from `core/runtime/`, defining **HOW** components are executed.

**Key Components:**

| Component | Purpose | Dependencies |
|-----------|---------|--------------|
| `WasmtimeEngine` | Concrete RuntimeEngine implementation using Wasmtime | `core/runtime/traits`, `core/component/*`, `wasmtime` crate |
| `WasmtimeComponentLoader` | Concrete ComponentLoader implementation | `core/runtime/traits`, `core/component/*`, `wasmtime` crate |
| `HostFunctions` | Host functions exposed to WASM components | `security/*`, `core/messaging/*`, `core/component/*` |
| `ResourceMonitor` | Enforces ResourceLimits during execution | `core/runtime/limits`, `core/component/*` |

**Architectural Rules:**

1. **Implements traits from `core/runtime/`** - WasmtimeEngine implements RuntimeEngine trait
2. **Depends on `security/`** - For capability validation, security context
3. **Depends on `core/component/`** - For ComponentId, ComponentHandle, ComponentMessage
4. **NO imports from `actor/`** - Violates ADR-WASM-023 (wrong direction)

**Dependency Flow:**
```
runtime/ ───────────────────► core/runtime/ (implements traits)
runtime/ ───────────────────► core/component/ (uses types)
runtime/ ───────────────────► security/ (validation)
runtime/ ───────────────────► core/messaging/ (message handling)
system/ ─────────────────────► runtime/ (coordinates)
```

**File Structure:**
```
runtime/
├── mod.rs              # Module declarations
├── engine.rs           # WasmtimeEngine (RuntimeEngine impl)
├── loader.rs           # WasmtimeComponentLoader (ComponentLoader impl)
├── host_functions.rs   # Host function implementations
└── resource_monitor.rs # Resource limit enforcement
```

### Dependency Inversion Pattern

The `runtime/` module implements traits from `core/runtime/` and receives security dependencies from `security/`:

```rust
// core/runtime/traits.rs - ABSTRACTION (Layer 1)
use crate::core::component::{ComponentId, ComponentHandle, ComponentMessage};
use crate::core::component::MessagePayload;
use crate::core::errors::wasm::WasmError;

pub trait RuntimeEngine: Send + Sync {
    fn load_component(&self, id: &ComponentId, wasm_bytes: &[u8]) -> Result<ComponentHandle, WasmError>;
    fn unload_component(&self, handle: &ComponentHandle) -> Result<(), WasmError>;
    fn call_handle_message(&self, handle: &ComponentHandle, message: &ComponentMessage) 
        -> Result<Option<MessagePayload>, WasmError>;
}

pub trait ComponentLoader: Send + Sync {
    fn load(&self, bytes: &[u8]) -> Result<Vec<u8>, WasmError>;
    fn validate(&self, bytes: &[u8]) -> Result<(), WasmError>;
    fn get_component_type(&self, bytes: &[u8]) -> Result<String, WasmError>;
}

// core/runtime/limits.rs - RESOURCE LIMITS (Layer 1)
pub struct ResourceLimits {
    pub max_memory: usize,      // Maximum memory in bytes
    pub max_execution_time: Duration,  // Maximum execution time
    pub max_fuel: Option<u64>,  // Maximum fuel units (optional)
}

// runtime/engine.rs - CONCRETE IMPLEMENTATION (Layer 3B)
use crate::core::runtime::traits::RuntimeEngine;
use crate::core::runtime::limits::ResourceLimits;
use crate::core::component::{ComponentId, ComponentHandle, ComponentMessage};
use crate::security::{SecurityContext, CapabilityValidator};
use wasmtime::{Engine, Module, Store, Instance};

pub struct WasmtimeEngine {
    engine: Engine,
    limits: ResourceLimits,
    security: Arc<dyn SecurityContext>,
}

impl RuntimeEngine for WasmtimeEngine {
    fn load_component(&self, id: &ComponentId, wasm_bytes: &[u8]) -> Result<ComponentHandle, WasmError> {
        // 1. Validate WASM bytes (security/)
        self.security.validate_wasm(wasm_bytes)?;
        
        // 2. Load Wasmtime module
        let module = Module::from_binary(&self.engine, wasm_bytes)?;
        
        // 3. Create instance with limits
        let mut store = Store::new(&self.engine, ());
        let instance = Instance::new(&mut store, &module, &[])?;
        
        // 4. Create ComponentHandle
        Ok(ComponentHandle::new(id.clone()))
    }
    
    fn call_handle_message(&self, handle: &ComponentHandle, message: &ComponentMessage) 
        -> Result<Option<MessagePayload>, WasmError> {
        // 1. Lookup component instance
        // 2. Validate message (security/)
        // 3. Call handle-message export with timeout (limits)
        // 4. Return response payload
        todo!("Implementation in Phase 5")
    }
}
```

## Usage Patterns

### Core Module Usage (Phase 3 Tasks)

**When to add code to `core/runtime/`:**
- ✅ New trait definitions (RuntimeEngine, ComponentLoader)
- ✅ New data structures (ResourceLimits, resource configuration)
- ✅ Foundation types used by multiple modules
- ✅ Trait method signatures and documentation

**When NOT to add code to `core/runtime/`:**
- ❌ Trait implementations (belongs to runtime/)
- ❌ Wasmtime-specific code
- ❌ External dependencies (Wasmtime, wasmtime-*)
- ❌ Business logic or algorithms

**Example - Creating New Traits:**
```rust
// core/runtime/traits.rs
use crate::core::component::{ComponentId, ComponentHandle, ComponentMessage};
use crate::core::component::MessagePayload;
use crate::core::errors::wasm::WasmError;

pub trait RuntimeEngine: Send + Sync {
    /// Load a WASM component and return a handle
    fn load_component(&self, id: &ComponentId, wasm_bytes: &[u8]) -> Result<ComponentHandle, WasmError>;
    
    /// Unload a WASM component and release resources
    fn unload_component(&self, handle: &ComponentHandle) -> Result<(), WasmError>;
    
    /// Call the `handle-message` export on a component
    fn call_handle_message(&self, handle: &ComponentHandle, message: &ComponentMessage) 
        -> Result<Option<MessagePayload>, WasmError>;
}

pub trait ComponentLoader: Send + Sync {
    /// Load WASM bytes into memory
    fn load(&self, bytes: &[u8]) -> Result<Vec<u8>, WasmError>;
    
    /// Validate WASM bytes for correctness
    fn validate(&self, bytes: &[u8]) -> Result<(), WasmError>;
    
    /// Determine the component type from WASM bytes
    fn get_component_type(&self, bytes: &[u8]) -> Result<String, WasmError>;
}
```

### Execution Module Usage (Phase 5 Tasks)

**When to add code to `runtime/`:**
- ✅ Concrete trait implementations (WasmtimeEngine, WasmtimeComponentLoader)
- ✅ Wasmtime-specific code (Module, Store, Instance)
- ✅ Host function implementations
- ✅ Resource limit enforcement logic
- ✅ WASM execution orchestration

**When NOT to add code to `runtime/`:**
- ❌ New trait definitions (belongs to core/runtime/)
- ❌ Core type definitions (belongs to core/component/)
- ❌ Security validation logic (belongs to security/)
- ❌ Actor lifecycle management (belongs to component/)

**Example - WasmtimeEngine:**
```rust
// runtime/engine.rs
use crate::core::runtime::traits::RuntimeEngine;
use crate::core::runtime::limits::ResourceLimits;
use crate::core::component::{ComponentId, ComponentHandle, ComponentMessage};
use crate::core::component::MessagePayload;
use crate::core::errors::wasm::WasmError;
use crate::security::{SecurityContext, CapabilityValidator};
use wasmtime::{Engine, Module, Store, Instance, Linker};
use std::sync::Arc;
use std::time::Duration;

pub struct WasmtimeEngine {
    engine: Engine,
    limits: ResourceLimits,
    security: Arc<dyn SecurityContext>,
    validator: Arc<dyn CapabilityValidator>,
}

impl WasmtimeEngine {
    pub fn new(limits: ResourceLimits, security: Arc<dyn SecurityContext>) -> Self {
        let config = wasmtime::Config::new();
        let engine = Engine::new(&config).expect("Failed to create Wasmtime engine");
        
        Self {
            engine,
            limits,
            security,
            validator: security.capability_validator(),
        }
    }
}

impl RuntimeEngine for WasmtimeEngine {
    fn load_component(&self, id: &ComponentId, wasm_bytes: &[u8]) -> Result<ComponentHandle, WasmError> {
        // 1. Security validation
        self.validator.validate_component_type(id, wasm_bytes)?;
        
        // 2. Validate WASM bytes
        Module::from_binary(&self.engine, wasm_bytes)?;
        
        // 3. Create ComponentHandle (actual instance creation deferred to first use)
        Ok(ComponentHandle::new(id.clone()))
    }
    
    fn call_handle_message(&self, handle: &ComponentHandle, message: &ComponentMessage) 
        -> Result<Option<MessagePayload>, WasmError> {
        // 1. Create store with limits
        let mut store = Store::new(&self.engine, ResourceLimiter::new(self.limits.clone()));
        
        // 2. Load or retrieve instance
        let module = Module::from_binary(&self.engine, handle.wasm_bytes())?;
        let mut linker = Linker::new(&self.engine);
        
        // 3. Register host functions (messaging, storage, etc.)
        register_host_functions(&mut linker, &self.security);
        
        // 4. Create instance
        let instance = linker.instantiate(&mut store, &module)?;
        
        // 5. Call handle-message export with timeout
        let handle_message = instance
            .get_typed_func::<(ComponentMessage,), Option<MessagePayload>>(&mut store, "handle-message")?;
        
        tokio::time::timeout(self.limits.max_execution_time, async {
            handle_message.call(&mut store, (message.clone(),))
        })
        .await
        .map_err(|_| WasmError::ExecutionTimeout(handle.id()))??;
        
        Ok(None)
    }
}
```

### Cross-Layer Collaboration Example

**Component Loading Flow: system/ → runtime/ → core/runtime/**

```rust
// 1. system/ coordinator initiates component loading
system::RuntimeManager::load_component(&ComponentId::new("ns", "component", "1"), wasm_bytes)

// 2. system/ calls ComponentLoader via runtime/ (concrete impl)
let loader = Arc::new(WasmtimeComponentLoader::new(security_context.clone()));
let loaded_bytes = loader.load(wasm_bytes)?;

// 3. system/ calls RuntimeEngine via runtime/ (concrete impl)
let engine = Arc::new(WasmtimeEngine::new(limits, security_context));
let handle = engine.load_component(&id, &loaded_bytes)?;

// 4. system/ stores handle in ComponentRegistry
registry.register(id.clone(), handle);

// 5. Component A sends message to Component B
//    → system/ looks up ComponentHandle
//    → system/ calls engine.call_handle_message()
//    → runtime/ executes WASM export with limits and security
```

## Best Practices

### Core Module Best Practices

1. **Keep traits generic and testable** - Use trait bounds, not concrete types
2. **Document thoroughly** - Every trait method must have clear documentation
3. **Use proper error types** - Import WasmError from core/errors/
4. **Define comprehensive traits** - Cover all operations needed by runtime/
5. **Test trait bounds** - Mock implementations for unit tests

### Execution Module Best Practices

1. **Implement all trait methods** - Don't leave unimplemented!()
2. **Enforce resource limits** - Use ResourceMonitor for all operations
3. **Integrate with security/** - Validate all operations
4. **Handle errors gracefully** - Convert Wasmtime errors to WasmError
5. **Test with real WASM** - Integration tests with actual .wasm files

### Antipatterns to Avoid

| Antipattern | Why Wrong | Correct Approach |
|-------------|-----------|------------------|
| Implementations in `core/` | Violates Layer 1 rule | Move to `runtime/` |
| New trait definitions in `runtime/` | Makes testing impossible | Define in `core/runtime/` |
| Importing `actor/` in `runtime/` | Breaks ADR-WASM-023 | Violates module boundary |
| Bypassing ResourceLimits | Security vulnerability | Always enforce limits |
| Direct Wasmtime in `component/` | Violates module responsibility | Use RuntimeEngine trait |

## Integration Points

### Module Dependency Map

```
system/ (LAYER 4)
    │
    ├── Coordinates: Component loading, message routing
    ├── Uses: WasmtimeEngine (from runtime/)
    └── Uses: WasmtimeComponentLoader (from runtime/)

runtime/ (LAYER 3B)
    │
    ├── Implements: RuntimeEngine trait (from core/runtime/)
    ├── Implements: ComponentLoader trait (from core/runtime/)
    ├── Enforces: ResourceLimits (from core/runtime/)
    ├── Depends: security/ (validation, SecurityContext)
    └── Depends: core/component/ (ComponentId, ComponentHandle, ComponentMessage)

core/runtime/ (LAYER 1)
    │
    ├── Defines: RuntimeEngine trait
    ├── Defines: ComponentLoader trait
    ├── Defines: ResourceLimits struct
    └── Depends: NOTHING except std, core/component/, core/messaging/
```

### External Integrations

| Integration | Module | Purpose |
|-------------|--------|---------|
| **Wasmtime** | `runtime/` | WASM execution engine, Module, Store, Instance, Linker |
| **Tokio** | `runtime/` | Async execution, timeout handling |
| **airssys-osl** | `security/` (used by runtime/) | SecurityContext, CapabilityValidator, AuditLogger |

## Implementation Guidelines for Tasks

### Phase 3 Tasks (Core Modules)

When implementing `core/runtime/` tasks (e.g., WASM-TASK-018):
1. **Define traits only** - No implementations
2. **Follow ADR-WASM-028** - Use exact specifications
3. **Add unit tests with mocks** - Mock RuntimeEngine, ComponentLoader
4. **Document thoroughly** - Every trait method with rustdoc
5. **Verify no violations** - Ensure no internal imports except core/submodules

### Phase 5 Tasks (Runtime Execution)

When implementing `runtime/` tasks:
1. **Implement all traits** - WasmtimeEngine, WasmtimeComponentLoader
2. **Integrate with security/** - Validate all operations
3. **Enforce resource limits** - Use ResourceMonitor
4. **Add unit + integration tests** - Mock security, real Wasmtime
5. **Verify architecture** - No imports from actor/, proper core/runtime/ usage

## References

### Related Documentation

**ADRs:**
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture (six-module design)
- **ADR-WASM-023:** Module Boundary Enforcement (dependency rules)
- **ADR-WASM-028:** Core Module Structure (core/ specifications)
- **ADR-WASM-026:** Implementation Roadmap (Phase 3 and Phase 5)

**Knowledges:**
- **KNOWLEDGE-WASM-037:** Rebuild Architecture Clean-Slate Design (layer organization)
- **KNOWLEDGE-WASM-038:** Component Module Responsibility and Architecture (core/component/ vs component/)
- **KNOWLEDGE-WASM-030:** Module Architecture Hard Requirements (mandatory rules)
- **KNOWLEDGE-WASM-019:** Runtime Dependency Architecture (Tokio vs airssys-rt)

**Standards:**
- **PROJECTS_STANDARD.md:** §2.1 (3-Layer Imports), §4.3 (Module Architecture)
- **Rust Guidelines:** M-DESIGN-FOR-AI, M-MODULE-DOCS, M-STATIC-VERIFICATION

## History

| Date | Version | Change |
|------|---------|--------|
| 2026-01-08 | 1.0 | Initial creation - Clarifies core/runtime/ vs runtime/ distinction |

---

**Template Version:** 1.0  
**Last Updated:** 2026-01-08
