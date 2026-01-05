# ADR-WASM-025: Clean-Slate Rebuild Architecture

**ADR ID:** ADR-WASM-025  
**Created:** 2026-01-05  
**Status:** Accepted  
**Deciders:** Architecture Team  
**Category:** Architecture / Module Design / Rebuild  
**Severity:** 🔴 **CRITICAL - FOUNDATIONAL DECISION**

---

## Title

Clean-Slate Rebuild of airssys-wasm with Dependency Inversion Architecture

---

## Context

### Problem Statement

The previous airssys-wasm implementation suffered from **critical architectural violations** that made incremental fixes impractical:

1. **Circular Dependencies:** `runtime/` ↔ `actor/` (documented in KNOWLEDGE-WASM-028)
2. **DI/DIP Violations:** Modules importing concrete implementations instead of abstractions
3. **Wrong Code Placement:** Messaging in `runtime/`, correlation in wrong modules (KNOWLEDGE-WASM-032, KNOWLEDGE-WASM-034)
4. **Fake Tests:** Tests that passed without validating functionality (KNOWLEDGE-WASM-033)
5. **Duplicate WASM Runtime:** Two runtimes using different APIs (KNOWLEDGE-WASM-027)

### Business Context

- Development blocked due to architectural violations
- 10+ days lost to architectural rework
- Need clear foundation for future development

### Technical Context

- Previous attempts at incremental fixes created more violations
- Architecture requires fundamental redesign following DIP
- Must integrate cleanly with airssys-rt and airssys-osl

---

## Decision

### Summary

**Rebuild airssys-wasm from scratch** with a new six-module architecture featuring:

1. **Layer-organized `core/`** with abstractions grouped by their target module
2. **Strict Dependency Inversion** (modules depend on traits, not implementations)
3. **One-way dependency flow** with `system/` as coordinator
4. **Renamed modules** for clarity: `actor/` → `component/`, `host_system/` → `system/`

### The Six Modules

```
airssys-wasm/src/
├── core/           # LAYER 1: Foundation (std only)
├── security/       # LAYER 2A: Security (deps: core/)
├── runtime/        # LAYER 2B: WASM Only (deps: core/, security/)
├── component/      # LAYER 3A: airssys-rt integration (deps: core/ traits)
├── messaging/      # LAYER 3B: Messaging patterns (deps: core/ traits)
└── system/         # LAYER 4: Coordinator (deps: ALL, injects concrete types)
```

### Dependency Flow

```
system/ (LAYER 4) ─── injects concrete ───► component/, messaging/
     │
     └─── knows concrete types from runtime/, security/

component/, messaging/ (LAYER 3) ─► core/ traits ONLY
                                    + airssys-rt

runtime/ (LAYER 2B) ─► core/, security/

security/ (LAYER 2A) ─► core/ + airssys-osl

core/ (LAYER 1) ─► std ONLY
```

### Key Design Patterns

**1. Layer-Organized Core:**
```
core/
├── component/         # Types for component/ module
│   ├── id.rs          # ComponentId
│   ├── traits.rs      # Component-related traits
├── runtime/           # Types for runtime/ module
│   └── traits.rs      # RuntimeEngine trait
├── messaging/         # Types for messaging/ module
│   └── traits.rs      # MessageRouter trait
└── errors/            # Error types
```

**2. Dependency Inversion:**
```rust
// component/ depends on TRAIT (core/runtime/traits.rs)
pub struct ComponentWrapper {
    engine: Arc<dyn RuntimeEngine>,  // Abstraction
}

// runtime/ IMPLEMENTS the trait
impl RuntimeEngine for WasmtimeEngine { ... }

// system/ INJECTS concrete into abstract
let engine = Arc::new(WasmtimeEngine::new());
let wrapper = ComponentWrapper::new(engine);
```

### Rationale

1. **Prevents circular dependencies:** All arrows point TO `core/`, never between sibling modules
2. **Enables testing:** Mock traits for unit tests
3. **Swappable implementations:** Test engine vs production engine
4. **Clear contracts:** Traits document expectations

### Assumptions

- Clean-slate rebuild is more efficient than incremental fixes
- Six modules provide appropriate granularity
- airssys-rt and airssys-osl APIs are stable

---

## Considered Options

### Option 1: Incremental Fix (REJECTED)

**Description:** Fix violations in existing codebase  

**Pros:**
- Preserves existing code
- Faster if changes are small

**Cons:**
- Each fix revealed more violations
- Technical debt compounds
- No clear end state

**Implementation Effort:** Ongoing (10+ days already spent)  
**Risk Level:** High (proven to fail)

### Option 2: Four-Module Architecture - KNOWLEDGE-WASM-036 (REJECTED)

**Description:** Previous proposal with `host_system/`, `actor/`, `messaging/`, `runtime/`

**Pros:**
- Fewer modules to manage
- Addressed some violations

**Cons:**
- Did not properly apply DIP
- `core/` not organized by layer
- Naming confusion (`actor/` vs `component/`)

**Implementation Effort:** Medium  
**Risk Level:** Medium (incomplete solution)

### Option 3: Clean-Slate Six-Module with DIP (ACCEPTED)

**Description:** Complete rebuild with layer-organized `core/` and proper DIP

**Pros:**
- Complete architectural reset
- Proper DIP from the start
- Layer-organized abstractions
- Clear naming (`component/` not `actor/`)

**Cons:**
- Requires full rebuild
- Previous code discarded

**Implementation Effort:** High  
**Risk Level:** Low (clean design)

---

## Implementation

### Implementation Plan

1. **Phase 1:** Create project skeleton with module structure
2. **Phase 2:** Build test fixtures (echo.wasm, counter.wasm, etc.)
3. **Phase 3:** Implement layer-by-layer (core → security → runtime → component/messaging → system)

### Dependencies

- WASI Preview 2 (for standardized host capabilities)
- wasmtime 24.0+ (for component model)
- airssys-rt (for actor system)
- airssys-osl (for security context)

---

## Implications

### System Impact

- **Breaking Change:** Complete rebuild discards previous implementation
- **Module Names:** `actor/` renamed to `component/`, `host_system/` renamed to `system/`
- **Dependency Direction:** Strict one-way flow prevents future violations

### Maintainability Impact

- **Improved:** Layer-organized `core/` makes abstractions discoverable
- **Improved:** DIP enables unit testing with mocks
- **Improved:** Clear module responsibilities reduce confusion

---

## Supersedes

This ADR supersedes and consolidates learnings from:

- **ADR-WASM-021:** Duplicate Runtime Remediation
- **ADR-WASM-022:** Circular Dependency Remediation  
- **ADR-WASM-023:** Module Boundary Enforcement (principles preserved, module structure updated)
- **ADR-WASM-024:** Refactor Messaging (incorporated into design)

---

## References

### Related Documents

- **KNOWLEDGE-WASM-037:** Rebuild Architecture - Clean Slate Design (detailed reference)
- **KNOWLEDGE-WASM-030:** Module Architecture Hard Requirements (historical)
- **KNOWLEDGE-WASM-033:** AI Fatal Mistakes - Lessons Learned
- **ADR-WASM-018:** Three-Layer Architecture

---

## History

| Date | Status | Reason |
|------|--------|--------|
| 2026-01-05 | ACCEPTED | Foundation for clean-slate rebuild |

---

**This ADR establishes the architectural foundation for the complete rebuild of airssys-wasm.**
