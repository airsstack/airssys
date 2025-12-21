# ADR-WASM-022: Circular Dependency Remediation (actor/ ↔ runtime/)

**ADR ID:** ADR-WASM-022  
**Created:** 2025-12-21  
**Updated:** 2025-12-21  
**Status:** Accepted  
**Deciders:** Architecture Team  
**Severity:** 🔴 **CRITICAL**

## Title

Remediation of Circular Dependency Between actor/ and runtime/ Modules

## Context

### Problem Statement

The `runtime/` module incorrectly imports types from `actor/` module, violating the one-way dependency architecture mandated by ADR-WASM-018.

**Correct Architecture:**
```
actor/ → runtime/ → core/  (one-way)
```

**Actual Architecture:**
```
actor/ ↔ runtime/  (circular - WRONG!)
```

### Evidence

| File | Line | Wrong Import |
|------|------|--------------|
| `runtime/async_host.rs` | 52 | `use crate::actor::ComponentMessage` |
| `runtime/messaging.rs` | 76 | `use crate::actor::ComponentMessage` |
| `runtime/messaging_subscription.rs` | 108-109 | `use crate::actor::component::{ComponentRegistry, ActorSystemSubscriber, SubscriberManager}` |

### Business Context

- Circular dependencies indicate confused module responsibilities
- Makes testing harder (can't test runtime/ in isolation)
- Increases coupling and reduces maintainability
- Violates ADR-WASM-018 three-layer architecture

### Technical Context

**Why does this happen?**

1. `ComponentMessage` is a **data type** but was placed in `actor/` instead of `core/`
2. `messaging_subscription.rs` contains logic that touches actor registration (should be in `actor/`)
3. No enforcement of layer boundaries during development

## Decision

### Summary

1. **Move `ComponentMessage`** from `actor/` to `core/` (it's a data type)
2. **Move `messaging_subscription.rs`** from `runtime/` to `actor/` (it's integration logic)
3. **Verify** no `runtime/ → actor/` imports remain
4. **Add CI check** to prevent future violations

### Decisions

#### Decision 1: Move ComponentMessage to core/

**Rationale:** `ComponentMessage` is a data structure with no actor-specific logic. It should be in `core/` where both `runtime/` and `actor/` can import it.

**Action:**
```rust
// NEW: src/core/component_message.rs
use crate::core::ComponentId;

#[derive(Debug, Clone)]
pub struct ComponentMessage {
    pub sender: ComponentId,
    pub recipient: ComponentId,
    pub payload: Vec<u8>,
    pub correlation_id: Option<String>,
}

// UPDATE: src/core/mod.rs
pub mod component_message;
pub use component_message::ComponentMessage;
```

**Files to update imports:**
- `src/runtime/async_host.rs`: `use crate::core::ComponentMessage`
- `src/runtime/messaging.rs`: `use crate::core::ComponentMessage`
- `src/actor/component/*.rs`: `use crate::core::ComponentMessage`

#### Decision 2: Relocate messaging_subscription.rs

**Rationale:** `messaging_subscription.rs` imports `ComponentRegistry` and `ActorSystemSubscriber` because it handles actor-level subscription logic. This belongs in `actor/`, not `runtime/`.

**Action:**
```
BEFORE:
  src/runtime/messaging_subscription.rs

AFTER:
  src/actor/component/messaging_subscription.rs
```

**Update runtime/mod.rs:** Remove the export  
**Update actor/component/mod.rs:** Add the export

#### Decision 3: Establish Layer Boundary Enforcement

**Rationale:** Prevent future violations with automated checking.

**Action:** Add CI script:

```bash
#!/bin/bash
# .github/scripts/check-layer-deps.sh

echo "Checking layer dependencies..."

# runtime/ should NEVER import from actor/
if grep -rq "use crate::actor" src/runtime/; then
    echo "❌ ERROR: runtime/ imports from actor/"
    grep -rn "use crate::actor" src/runtime/
    exit 1
fi

# core/ should NEVER import from runtime/ or actor/
if grep -rq "use crate::runtime\|use crate::actor" src/core/; then
    echo "❌ ERROR: core/ imports from higher layers"
    grep -rn "use crate::runtime\|use crate::actor" src/core/
    exit 1
fi

echo "✅ Layer dependencies OK"
```

### Assumptions

1. `ComponentMessage` has no dependencies on actor-specific types
2. `messaging_subscription.rs` functionality is primarily actor-integration
3. Moving files won't break external API (internal reorganization)

## Considered Options

### Option 1: Move Types to core/, Relocate Files (CHOSEN)

**Description:** Move shared types to `core/`, move integration logic to `actor/`

**Pros:**
- Clean layer separation
- Each module has clear responsibility
- Enables isolated testing
- Follows ADR-WASM-018

**Cons:**
- Requires file moves and import updates
- Some effort to update tests

**Implementation Effort:** Low (2-4 hours)  
**Risk Level:** Low

### Option 2: Create Intermediate shared/ Module (REJECTED)

**Description:** Create a `shared/` module for types used by multiple layers

**Pros:**
- Minimal file movement
- Quick fix

**Cons:**
- Adds another module to understand
- Doesn't fix the conceptual issue
- `shared/` is vague - where does it fit in layers?

**Implementation Effort:** Low  
**Risk Level:** Medium (architectural ambiguity)

### Option 3: Leave As-Is, Document as Debt (REJECTED)

**Description:** Accept the circular dependency, document it

**Pros:**
- No work required

**Cons:**
- Violates ADR-WASM-018
- Technical debt compounds
- Testing remains coupled
- Sets bad precedent

**Implementation Effort:** None  
**Risk Level:** High (long-term)

## Implementation

### Implementation Plan

**Phase 1: Move ComponentMessage (1-2 hours)**
1. Create `src/core/component_message.rs`
2. Move `ComponentMessage` struct definition
3. Update `src/core/mod.rs` to export
4. Update all imports in `runtime/` and `actor/`
5. Run `cargo build` to verify

**Phase 2: Relocate messaging_subscription.rs (1-2 hours)**
1. Move file from `runtime/` to `actor/component/`
2. Update `runtime/mod.rs` (remove export)
3. Update `actor/component/mod.rs` (add export)
4. Update any internal imports
5. Run `cargo build` and `cargo test`

**Phase 3: Add CI Enforcement (30 min)**
1. Create `.github/scripts/check-layer-deps.sh`
2. Add to CI workflow
3. Verify it passes

### Timeline

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| Phase 1 | 1-2 hours | None |
| Phase 2 | 1-2 hours | Phase 1 |
| Phase 3 | 30 min | Phase 2 |
| **Total** | **2.5-4.5 hours** | |

### Dependencies

- Should be done **before** ADR-WASM-021 Phase 2 (duplicate runtime fix)
- Part of the overall architecture remediation effort

## Implications

### System Impact

| Aspect | Impact |
|--------|--------|
| **Module Boundaries** | Clearly defined and enforced |
| **Testing** | `runtime/` can be tested in isolation |
| **Maintainability** | Improved - changes don't cascade circularly |
| **Import Paths** | Some imports change (internal only) |

### Maintainability Impact

**Significant Improvement:**
- Clear layer responsibilities
- Automated enforcement prevents regression
- Easier to understand module structure

## Compliance

### Workspace Standards

- **ADR-WASM-018:** Now compliant (three-layer architecture)
- **ADR-WASM-011:** Now compliant (module structure)
- **§4.3 Module Architecture:** Restored correct dependency flow

### Technical Debt

- **Debt Resolved:** Circular dependency between actor/ and runtime/
- **Debt Created:** None

## Monitoring and Validation

### Success Criteria

1. ✅ Zero `use crate::actor` in `src/runtime/`
2. ✅ Zero `use crate::runtime` or `use crate::actor` in `src/core/`
3. ✅ `ComponentMessage` exported from `core/`
4. ✅ CI check passes
5. ✅ All tests pass

### Verification Commands

```bash
# Should return NO results after fix
grep -r "use crate::actor" src/runtime/

# Should return NO results
grep -r "use crate::runtime\|use crate::actor" src/core/

# All tests should pass
cargo test
```

## Risks and Mitigations

### Identified Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Missed import update | Low | Low | Compiler will catch it |
| Test failures | Low | Low | Run full test suite |
| Functionality regression | Very Low | Medium | Integration tests |

### Contingency Plans

If issues arise during move, the changes are easily reversible (just move files back and revert import changes).

## References

### Related Documents

- **KNOWLEDGE-WASM-028:** Circular Dependency Documentation
- **KNOWLEDGE-WASM-027:** Duplicate WASM Runtime (related issue)
- **ADR-WASM-018:** Three-Layer Architecture
- **ADR-WASM-011:** Module Structure Organization
- **ADR-WASM-021:** Duplicate Runtime Remediation (this is prerequisite)

### Affected Files

| File | Change |
|------|--------|
| `src/core/component_message.rs` | CREATE |
| `src/core/mod.rs` | UPDATE (add export) |
| `src/runtime/async_host.rs` | UPDATE (change import) |
| `src/runtime/messaging.rs` | UPDATE (change import) |
| `src/runtime/messaging_subscription.rs` | MOVE to actor/ |
| `src/runtime/mod.rs` | UPDATE (remove export) |
| `src/actor/component/mod.rs` | UPDATE (add export) |
| `src/actor/component/*.rs` | UPDATE (change imports) |

## History

### Status Changes

- **2025-12-21:** Status set to Accepted

### Discovery Timeline

- **2025-12-21:** Circular dependency identified during architecture analysis
- **2025-12-21:** ADR created with remediation plan

---
**ADR Status:** Accepted  
**Priority:** 🔴 CRITICAL - Prerequisite for ADR-WASM-021  
**Effort:** 2.5-4.5 hours  
**Template Version:** 1.0
