# ADR-WASM-026: Implementation Roadmap for Clean-Slate Rebuild

**ADR ID:** ADR-WASM-026  
**Created:** 2026-01-05  
**Status:** Accepted  
**Deciders:** Architecture Team  
**Category:** Implementation Planning / Roadmap  
**Severity:** 🔴 **CRITICAL - MASTER IMPLEMENTATION PLAN**

---

## Title

Implementation Roadmap for Clean-Slate Rebuild of airssys-wasm

---

## Context

### Background

Following ADR-WASM-025 (Clean-Slate Rebuild Architecture), this ADR establishes the **master implementation roadmap** for rebuilding airssys-wasm from scratch. This document serves as the **single source of truth** for implementation phases and task ordering.

### Design Principles

1. **Single-Objective Tasks**: Each task contains exactly ONE objective with ONE plan
2. **WIT-First Approach**: WIT interfaces defined before module implementation
3. **Layer-by-Layer Building**: Build from core/ → security/ → runtime/ → component/messaging/ → system/
4. **Incremental Verification**: Re-analyze after each task completion
5. **DIP Compliance**: All modules depend on traits in `core/`, not concrete implementations

### Related Documents

| Document | Purpose |
|----------|---------|
| [ADR-WASM-025](adr-wasm-025-clean-slate-rebuild-architecture.md) | Architectural foundation |
| [KNOWLEDGE-WASM-037](../knowledges/knowledge-wasm-037-rebuild-architecture-clean-slate.md) | Technical reference |
| [ADR-WASM-027](adr-wasm-027-wit-interface-design.md) | WIT Interface specifications |
| [ADR-WASM-028](adr-wasm-028-core-module-structure.md) | Core module design |
| [ADR-WASM-029](adr-wasm-029-security-module-design.md) | Security module design |
| [ADR-WASM-030](adr-wasm-030-runtime-module-design.md) | Runtime module design |
| [ADR-WASM-031](adr-wasm-031-component-messaging-design.md) | Component & Messaging design |
| [ADR-WASM-032](adr-wasm-032-system-module-design.md) | System module design |

---

## Decision

### Summary

Implement airssys-wasm rebuild in **7 phases** with **strict task ordering**:

1. **Phase 1:** WIT Interface System → [ADR-WASM-027](adr-wasm-027-wit-interface-design.md)
2. **Phase 2:** Project Restructuring (inline - simple changes)
3. **Phase 3:** Core Module (Layer 1) → [ADR-WASM-028](adr-wasm-028-core-module-structure.md)
4. **Phase 4:** Security Module (Layer 2A) → [ADR-WASM-029](adr-wasm-029-security-module-design.md)
5. **Phase 5:** Runtime Module (Layer 2B) → [ADR-WASM-030](adr-wasm-030-runtime-module-design.md)
6. **Phase 6:** Component & Messaging (Layer 3) → [ADR-WASM-031](adr-wasm-031-component-messaging-design.md)
7. **Phase 7:** System Module & Integration (Layer 4) → [ADR-WASM-032](adr-wasm-032-system-module-design.md)

---

## Phase 1: WIT Interface System

**Detailed Spec:** [ADR-WASM-027](adr-wasm-027-wit-interface-design.md)

| Task ID | Name | Depends On |
|---------|------|------------|
| WASM-TASK-002 | Setup WIT directory structure | WASM-TASK-001 |
| WASM-TASK-003 | Create types.wit | WASM-TASK-002 |
| WASM-TASK-004 | Create errors.wit | WASM-TASK-003 |
| WASM-TASK-005 | Create capabilities.wit | WASM-TASK-003 |
| WASM-TASK-006 | Create component-lifecycle.wit | WASM-TASK-004 |
| WASM-TASK-007 | Create host-messaging.wit | WASM-TASK-004 |
| WASM-TASK-008 | Create host-services.wit | WASM-TASK-004 |
| WASM-TASK-009 | Create storage.wit | WASM-TASK-004 |
| WASM-TASK-010 | Create world.wit | WASM-TASK-006 to 009 |
| WASM-TASK-011 | Validate WIT package | WASM-TASK-010 |
| WASM-TASK-012 | Setup wit-bindgen integration | WASM-TASK-011 |

---

## Phase 2: Project Restructuring

**Simple changes - no separate ADR needed**

| Task ID | Name | Depends On |
|---------|------|------------|
| WASM-TASK-013 | Rename actor/ to component/ | WASM-TASK-012 |
| WASM-TASK-014 | Create system/ module | WASM-TASK-013 |
| WASM-TASK-015 | Create messaging/ module | WASM-TASK-014 |
| WASM-TASK-016 | Update lib.rs exports | WASM-TASK-015 |

**Target Structure:**
```
airssys-wasm/src/
├── lib.rs
├── prelude.rs
├── core/               # LAYER 1
├── security/           # LAYER 2A
├── runtime/            # LAYER 2B
├── component/          # LAYER 3A (renamed from actor/)
├── messaging/          # LAYER 3B (new)
└── system/             # LAYER 4 (new)
```

---

## Phase 3: Core Module (Layer 1)

**Detailed Spec:** [ADR-WASM-028](adr-wasm-028-core-module-structure.md)

| Task ID | Name | Depends On |
|---------|------|------------|
| WASM-TASK-017 | Create core/component/ submodule | WASM-TASK-016 |
| WASM-TASK-018 | Create core/runtime/ submodule | WASM-TASK-017 |
| WASM-TASK-019 | Create core/messaging/ submodule | WASM-TASK-017 |
| WASM-TASK-020 | Create core/security/ submodule | WASM-TASK-017 |
| WASM-TASK-021 | Create core/storage/ submodule | WASM-TASK-017 |
| WASM-TASK-022 | Create core/errors/ submodule | WASM-TASK-017 |
| WASM-TASK-023 | Create core/config/ submodule | WASM-TASK-017 |
| WASM-TASK-024 | Write core/ unit tests | WASM-TASK-022 |

---

## Phase 4: Security Module (Layer 2A)

**Detailed Spec:** [ADR-WASM-029](adr-wasm-029-security-module-design.md)

| Task ID | Name | Depends On |
|---------|------|------------|
| WASM-TASK-025 | Create security/capability/ submodule | WASM-TASK-024 |
| WASM-TASK-026 | Implement CapabilityValidator | WASM-TASK-025 |
| WASM-TASK-027 | Create security/policy/ submodule | WASM-TASK-025 |
| WASM-TASK-028 | Implement SecurityAuditLogger | WASM-TASK-025 |
| WASM-TASK-029 | Create airssys-osl bridge | WASM-TASK-026 |
| WASM-TASK-030 | Write security/ unit tests | WASM-TASK-029 |

---

## Phase 5: Runtime Module (Layer 2B)

**Detailed Spec:** [ADR-WASM-030](adr-wasm-030-runtime-module-design.md)

| Task ID | Name | Depends On |
|---------|------|------------|
| WASM-TASK-031 | Implement WasmtimeEngine | WASM-TASK-030 |
| WASM-TASK-032 | Implement ComponentLoader | WASM-TASK-031 |
| WASM-TASK-033 | Implement StoreManager | WASM-TASK-031 |
| WASM-TASK-034 | Implement host functions | WASM-TASK-031 |
| WASM-TASK-035 | Implement ResourceLimiter | WASM-TASK-031 |
| WASM-TASK-036 | Write runtime/ unit tests | WASM-TASK-035 |

> ⚠️ **MANDATORY**: Use `wasmtime::component::Component` (Component Model), NOT `wasmtime::Module` (core WASM). See KNOWLEDGE-WASM-027.

---

## Phase 6: Component & Messaging Modules (Layer 3)

**Detailed Spec:** [ADR-WASM-031](adr-wasm-031-component-messaging-design.md)

| Task ID | Name | Depends On |
|---------|------|------------|
| WASM-TASK-037 | Implement ComponentWrapper | WASM-TASK-036 |
| WASM-TASK-038 | Implement ComponentRegistry | WASM-TASK-037 |
| WASM-TASK-039 | Implement ComponentSpawner | WASM-TASK-037 |
| WASM-TASK-040 | Implement SupervisorConfig | WASM-TASK-037 |
| WASM-TASK-041 | Implement fire-and-forget pattern | WASM-TASK-038 |
| WASM-TASK-042 | Implement request-response pattern | WASM-TASK-041 |
| WASM-TASK-043 | Implement CorrelationTracker | WASM-TASK-042 |
| WASM-TASK-044 | Implement ResponseRouter | WASM-TASK-043 |
| WASM-TASK-045 | Write component/ unit tests | WASM-TASK-040 |
| WASM-TASK-046 | Write messaging/ unit tests | WASM-TASK-044 |

---

## Phase 7: System Module & Integration (Layer 4)

**Detailed Spec:** [ADR-WASM-032](adr-wasm-032-system-module-design.md)

| Task ID | Name | Depends On |
|---------|------|------------|
| WASM-TASK-047 | Implement RuntimeManager | WASM-TASK-046 |
| WASM-TASK-048 | Implement lifecycle management | WASM-TASK-047 |
| WASM-TASK-049 | Implement RuntimeBuilder | WASM-TASK-047 |
| WASM-TASK-050 | Create echo.wasm fixture | WASM-TASK-049 |
| WASM-TASK-051 | Create counter.wasm fixture | WASM-TASK-050 |
| WASM-TASK-052 | Create callback.wasm fixture | WASM-TASK-051 |
| WASM-TASK-053 | Write integration tests | WASM-TASK-052 |
| WASM-TASK-054 | Write system/ unit tests | WASM-TASK-049 |

---

## Task Summary

| Phase | Tasks | Range | Detailed ADR |
|-------|-------|-------|--------------|
| Phase 1: WIT Interface System | 11 | WASM-TASK-002 to 012 | ADR-WASM-027 |
| Phase 2: Project Restructuring | 4 | WASM-TASK-013 to 016 | (inline) |
| Phase 3: Core Module | 8 | WASM-TASK-017 to 024 | ADR-WASM-028 |
| Phase 4: Security Module | 6 | WASM-TASK-025 to 030 | ADR-WASM-029 |
| Phase 5: Runtime Module | 6 | WASM-TASK-031 to 036 | ADR-WASM-030 |
| Phase 6: Component & Messaging | 10 | WASM-TASK-037 to 046 | ADR-WASM-031 |
| Phase 7: System & Integration | 8 | WASM-TASK-047 to 054 | ADR-WASM-032 |
| **TOTAL** | **53** | WASM-TASK-002 to 054 | |

---

## Verification Strategy

### Per-Task Verification

Each task must include:
1. **Build check**: `cargo build -p airssys-wasm`
2. **Lint check**: `cargo clippy -p airssys-wasm --all-targets -- -D warnings`
3. **Module boundary check**: Verify import direction compliance

### Module Boundary Checks (CI Enforcement)

```bash
# All must return empty for compliance
grep -rn "use crate::component" src/runtime/
grep -rn "use crate::messaging" src/runtime/
grep -rn "use crate::system" src/runtime/
grep -rn "use crate::runtime" src/security/
grep -rn "use crate::" src/core/  # core imports only std
```

---

## Adjustment Protocol

### When to Adjust

1. After completing a task, if new requirements discovered
2. If a task reveals design issues requiring phase changes
3. If external dependencies change (wasmtime, wit-bindgen versions)

### How to Adjust

1. Update the relevant **detailed ADR** (027-032) for implementation changes
2. Update **this ADR** only for phase/task ordering changes
3. Document the reason in History section

---

## History

| Date | Version | Change |
|------|---------|--------|
| 2026-01-05 | 1.0 | Initial roadmap creation |
| 2026-01-05 | 2.0 | Refactored to high-level roadmap with links to detailed ADRs |

---

**This ADR establishes the master implementation plan. See linked ADRs for detailed specifications.**
