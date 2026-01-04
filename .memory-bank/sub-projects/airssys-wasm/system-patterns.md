# airssys-wasm System Patterns

**Last Updated:** 2026-01-04

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

### Module Structure (Rebuilding from scratch)

```
airssys-wasm/src/
├── core/      # Foundation - imports NOTHING (empty now)
├── security/  # Security logic - imports core/
├── runtime/   # WASM execution - imports core/, security/
├── actor/     # Actor integration - imports core/, security/, runtime/
└── wit/        # WIT interfaces
```

### Dependency Rules (ADR-WASM-023 - MANDATORY)

**ALLOWED:**
```
✅ actor/    → runtime/
✅ actor/    → security/
✅ actor/    → core/
✅ runtime/  → security/
✅ runtime/  → core/
✅ security/ → core/
```

**FORBIDDEN:**
```
❌ runtime/  → actor/
❌ security/ → runtime/
❌ security/ → actor/
❌ core/     → ANY MODULE
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
- **ADR-WASM-002:** WASM Runtime Engine Selection (Wasmtime 24.0, Component Model)
- **ADR-WASM-005:** Capability-Based Security Model
- **ADR-WASM-011:** Module Structure Organization
- **ADR-WASM-023:** Module Boundary Enforcement (MANDATORY)

### Implementation ADRs
- **ADR-WASM-018:** Three-Layer Architecture (airssys-wasm = Layer 2, airssys-rt = Layer 3)
- **ADR-WASM-020:** Message Delivery Ownership (ActorSystemSubscriber owns delivery)
- **ADR-WASM-019:** Runtime Dependency Management

### Knowledge Documents (READ BEFORE IMPLEMENTATION)
- **KNOWLEDGE-WASM-031:** Foundational Architecture (READ FIRST)
- **KNOWLEDGE-WASM-030:** Module Architecture Hard Requirements (MANDATORY)
- **KNOWLEDGE-WASM-012:** Module Structure Architecture
- **KNOWLEDGE-WASM-001:** Component Framework Architecture
- **KNOWLEDGE-WASM-005:** Messaging Architecture
- **KNOWLEDGE-WASM-020:** AirsSys Security Integration

---

## Implementation Approach

### Phase 1: Foundation (WASM-TASK-001)
**Objective:** Create basic project structure
**Actions:**
1. Create Cargo.toml with workspace dependencies
2. Create four-module directory structure (core/, security/, runtime/, actor/)
3. Create lib.rs entry point
4. Create tests/fixtures/ directory
5. Create wit/ directory structure

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
