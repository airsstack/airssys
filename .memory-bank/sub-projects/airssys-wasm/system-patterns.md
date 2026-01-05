# airssys-wasm System Patterns

**Last Updated:** 2026-01-05

---

## Component Architecture (Four-Module System)

```
┌─────────────────────────────────────────────────────────┐
│         Component Host                  │ ← Component lifecycle
├─────────────────────────────────────────────────┤
│         Security Sandbox                │ ← Capability enforcement
├─────────────────────────────────────────┤
│         WASM Runtime                 │ ← WASM execution
├─────────────────────────────────────────┤
│         WASI Implementation             │ ← System interface
└─────────────────────────────────────────┘
```

## Current Implementation Status

### Module Structure (Clean-Slate Rebuild - Six Modules)

**Reference:** KNOWLEDGE-WASM-037, ADR-WASM-025

```
airssys-wasm/src/
├── core/           # LAYER 1: Foundation - imports std ONLY
├── security/       # LAYER 2A: Security logic - imports core/
├── runtime/        # LAYER 2B: WASM execution - imports core/, security/
├── component/      # LAYER 3A: airssys-rt integration - imports core/ traits
├── messaging/      # LAYER 3B: Messaging patterns - imports core/ traits
├── system/         # LAYER 4: Coordinator - imports ALL, injects concrete types
└── wit/            # WIT interfaces
```

### Dependency Rules (ADR-WASM-025, KNOWLEDGE-WASM-037)

**Dependency Inversion Principle:**
- Modules depend on TRAITS (defined in core/\<module\>/traits.rs), not concrete implementations
- `system/` is the only module that knows about concrete types
- `system/` injects dependencies into lower layers

**ALLOWED:**
```
✅ system/    → ALL modules (coordinator knows all concrete types)
✅ component/ → core/ traits + airssys-rt
✅ messaging/ → core/ traits + airssys-rt
✅ runtime/   → security/, core/
✅ security/  → core/
✅ core/      → std ONLY
```

**FORBIDDEN:**
```
❌ component/ → runtime/ concrete (use RuntimeEngine trait from core/)
❌ messaging/ → runtime/ concrete
❌ runtime/   → component/
❌ runtime/   → messaging/
❌ security/  → runtime/
❌ core/      → ANY MODULE
```

### Module Responsibilities

**core/** - Foundation (imports NOTHING)
- Shared types: ComponentId, ComponentMessage, errors, configs
- Trait abstractions: RuntimeEngine, SecurityValidator
- Status: EMPTY (not yet implemented)

**security/** - Security logic
- Capability types: Capability, CapabilitySet
- Policy validation
- ACL integration with airssys-osl
- Audit types
- Status: EMPTY (not yet implemented)

**runtime/** - WASM execution
- WasmEngine (Wasmtime with Component Model)
- ComponentLoader
- StoreManager
- Host function definitions
- Resource limits
- WASM execution (call_handle_message, call_handle_callback)
- Status: EMPTY (not yet implemented)

**actor/** - Actor integration
- ComponentActor
- ComponentRegistry
- Message routing
- Component lifecycle management
- Health monitoring
- Status: EMPTY (not yet implemented)

**wit/** - WIT interfaces
- Core WIT packages (types, capabilities, component-lifecycle, host-services)
- Extension packages (filesystem, network, process)
- Status: PLACEHOLDER (documentation only)
```

---

## Integration Patterns

### Actor-Based Hosting
```rust
use airssys_rt::{Actor, ActorSystem};

// Component as Actor
pub struct ComponentActor {
    component_id: ComponentId,
    actor_id: ActorId,
    capabilities: CapabilitySet,
}

// Host System
pub struct HostSystem {
    component_actors: HashMap<ComponentId, ActorId>,
    capabilities: HashMap<ComponentId, CapabilitySet>,
}
```

### Security-Based Sandboxing
```rust
// Capability validation
pub fn validate_component_access(
    component_id: &ComponentId,
    operation: FileSystemOperation,
) -> Result<(), SecurityError> {
    let capabilities = self.capabilities.get(&component_id)?;
    if !capabilities.can_perform_operation(&operation) {
        return Err(SecurityError::AccessDenied { component_id, operation });
    }
    
    // Delegate to airssys-osl with security context
    self.osl_context.execute_file_operation(operation).await?;
    Ok(())
}
```

### WASM Execution Pattern
```rust
// Component instance
pub struct ComponentInstance {
    instance: Instance,
    store: Store<ComponentContext>,
    capabilities: CapabilitySet,
}

// Execution engine
pub struct WasmEngine {
    engine: Engine,
    components: HashMap<ComponentId, ComponentInstance>,
    security_enforcer: SecurityEnforcer,
}
```

---

## Architecture Decisions (ADRs)

### Critical ADRs (READ FIRST)
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture (CRITICAL - NEW)
- **ADR-WASM-026:** Implementation Roadmap (7 phases, 53 tasks)
- **ADR-WASM-027:** WIT Interface Design (Phase 1 specifications)
- **ADR-WASM-002:** WASM Runtime Engine Selection (Wasmtime 24.0, Component Model)
- **ADR-WASM-005:** Capability-Based Security Model
- **ADR-WASM-023:** Module Boundary Enforcement (MANDATORY)

### Implementation ADRs
- **ADR-WASM-018:** Three-Layer Architecture (airssys-wasm = Layer 2, airssys-rt = Layer 3)
- **ADR-WASM-020:** Message Delivery Ownership (ActorSystemSubscriber owns delivery)
- **ADR-WASM-019:** Runtime Dependency Management

### Knowledge Documents (READ BEFORE IMPLEMENTATION)
- **KNOWLEDGE-WASM-037:** Rebuild Architecture - Clean Slate Design (READ FIRST - NEW)
- **KNOWLEDGE-WASM-030:** Module Architecture Hard Requirements (superseded by KNOWLEDGE-WASM-037)
- **KNOWLEDGE-WASM-031:** Foundational Architecture  
- **KNOWLEDGE-WASM-012:** Module Structure Architecture
- **KNOWLEDGE-WASM-001:** Component Framework Architecture
- **KNOWLEDGE-WASM-005:** Messaging Architecture
- **KNOWLEDGE-WASM-020:** AirsSys Security Integration

---

## Implementation Approach

### Phase 1: WIT Interface System (WASM-TASK-002 through WASM-TASK-012)
**Objective:** Define complete WIT interface contract
**Reference:** ADR-WASM-027 (WIT Interface Design)

**Status:** Ready to Start (all 11 tasks created)

**Tasks (All Pending):**
1. WASM-TASK-002: Setup WIT Directory Structure
2. WASM-TASK-003: Create types.wit
3. WASM-TASK-004: Create errors.wit
4. WASM-TASK-005: Create capabilities.wit
5. WASM-TASK-006: Create component-lifecycle.wit
6. WASM-TASK-007: Create host-messaging.wit
7. WASM-TASK-008: Create host-services.wit
8. WASM-TASK-009: Create storage.wit
9. WASM-TASK-010: Create world.wit
10. WASM-TASK-011: Validate WIT package
11. WASM-TASK-012: Setup wit-bindgen integration

**Verification:**
```bash
# Build check
cargo build -p airssys-wasm

# Architecture verification (ADR-WASM-023 MANDATORY)
grep -rn "use crate::runtime" src/core/
grep -rn "use crate::actor" src/core/
grep -rn "use crate::actor" src/security/
grep -rn "use crate::actor" src/runtime/
grep -rn "use crate::runtime" src/security/
grep -rn "use crate::" src/core/
```
**Expected:** All grep commands return empty (clean architecture)

**Risk Mitigation:**
- Plans reference ALL relevant ADRs and Knowledges
- Verification commands are MANDATORY per ADR-WASM-023
- NO proceeding without verification

---

## Testing Strategy

### Test Requirements (MANDATORY - per AGENTS.md)
- BOTH unit tests AND integration tests required
- Tests must prove REAL functionality (not just API existence)
- Tests must use real WASM fixtures (not mocks)

### Fixture Management
- tests/fixtures/ directory must exist
- README.md must document all fixtures

### Current Status
- All modules EMPTY (no code yet)
- No tests exist yet
- Only project structure and Cargo.toml

---

## Critical Lessons from KNOWLEDGE-WASM-033 (AI Fatal Mistakes)

### ❌ NEVER Do These Things
1. Claim "verified" without showing ACTUAL grep output
2. Proceed without reading ADRs/Knowledges
3. Ignore module boundary rules
4. Create stub tests instead of REAL tests
5. Claim "complete" without verification

### ✅ ALWAYS Do These Things
1. Read ADR-WASM-023 before planning
2. Read KNOWLEDGE-WASM-030 (Module Architecture Requirements)
3. Run verification commands
4. Show ACTUAL command output as proof
5. Write REAL tests, not stubs
6. Check module boundaries before and after implementation
7. Reference ADRs/Knowledges in plans

---

## Next Steps After WASM-TASK-001

### Phase 2: Core Types (Future Task)
- Implement ComponentId, ComponentMessage, WasmError, config types

### Phase 3: Security Module (Future Task)
- Implement capabilities, policies, ACL integration

### Phase 4: Runtime Module (Future Task)
- Implement WasmEngine, ComponentLoader, StoreManager

### Phase 5: Actor Integration (Future Task)
- Implement ComponentActor, ComponentRegistry, messaging

### Phase 6: WIT Interfaces (Future Task)
- Define core packages and extension packages

---

## Architecture Compliance Status

### Module Boundaries (ADR-WASM-023)
- ❌ NOT APPLICABLE YET (no code exists)

### Standards (PROJECTS_STANDARD.md)
- ❌ NOT APPLICABLE YET (no code exists)

---

## Notes

**Why previous project failed:**
- Violated ADR-WASM-023 repeatedly
- Created stub tests instead of REAL tests
- Proceeded without verification
- Ignored documentation

**Why this rebuild will succeed:**
- New task management format enforces single action per task
- Plans MUST reference ADRs/Knowledges
- Verification workflow is MANDATORY
- AI agents will read documentation first
- Single-action focus prevents scope creep

**Critical difference:**
OLD: Multi-phase complex tasks → violations accumulated
NEW: Single-action tasks → clear objectives, easier verification
