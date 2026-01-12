# ADR-WASM-026: Implementation Roadmap for Clean-Slate Rebuild

**ADR ID:** ADR-WASM-026  
**Created:** 2026-01-05  
**Status:** Accepted  
**Deciders:** Architecture Team  
**Category:** Implementation Planning / Roadmap  
**Severity:** 🔴 **CRITICAL - MASTER IMPLEMENTATION PLAN**  
**Last Updated:** 2026-01-12

---

## Title

Implementation Roadmap for Clean-Slate Rebuild of airssys-wasm

---

## Context

### Background

Following ADR-WASM-025 (Clean-Slate Rebuild Architecture), this ADR establishes a **master implementation roadmap** for rebuilding airssys-wasm from scratch. This document serves as a **single source of truth** for implementation phases and task ordering.

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
 | [Multi-Project Memory Bank Instructions](../../../.aiassisted/instructions/multi-project-memory-bank.instructions.md) | Task management (UPDATED 2026-01-12) |

---

## Decision

### Summary

Implement airssys-wasm rebuild in **7 phases** with **strict task ordering**:

1. **Phase1:** WIT Interface System → [ADR-WASM-027](adr-wasm-027-wit-interface-design.md)
2. **Phase 2:** Project Restructuring (inline - simple changes)
3. **Phase 3:** Core Module (Layer 1) → [ADR-WASM-028](adr-wasm-028-core-module-structure.md)
4. **Phase 4:** Security Module (Layer 2A) → [ADR-WASM-029](adr-wasm-029-security-module-design.md)
5. **Phase 5:** Runtime Module (Layer 2B) → [ADR-WASM-030](adr-wasm-030-runtime-module-design.md)
6. **Phase 6:** Component & Messaging (Layer 3) → [ADR-WASM-031](adr-wasm-031-component-messaging-design.md)
7. **Phase 7:** System Module & Integration (Layer 4) → [ADR-WASM-032](adr-wasm-032-system-module-design.md)

---

## Phase 1: WIT Interface System

**Detailed Spec:** [ADR-WASM-027](adr-wasm-027-wit-interface-design.md)

 | Task ID | Name | Depends On | Status | Compliance |
 |---------|------|------------|--------|------------|
 | WASM-TASK-002 | Setup WIT directory structure | WASM-TASK-001 | ✅ Complete | ✅ Single-Action |
 | WASM-TASK-003 | Create types.wit | WASM-TASK-002 | ✅ Complete | ✅ Single-Action |
 | WASM-TASK-004 | Create errors.wit | WASM-TASK-003 | ✅ Complete | ✅ Single-Action |
 | WASM-TASK-005 | Create capabilities.wit | WASM-TASK-003 | ✅ Complete | ✅ Single-Action |
 | WASM-TASK-006 | Create component-lifecycle.wit | WASM-TASK-004 | ✅ Complete | ✅ Single-Action |
 | WASM-TASK-007 | Create host-messaging.wit | WASM-TASK-004 | ✅ Complete | ✅ Single-Action |
 | WASM-TASK-008 | Create host-services.wit | WASM-TASK-004 | ✅ Complete | ✅ Single-Action |
 | WASM-TASK-009 | Create storage.wit | WASM-TASK-004 | ✅ Complete | ✅ Single-Action |
 | WASM-TASK-010 | Create world.wit | WASM-TASK-006 to 009 | ✅ Complete | ✅ Single-Action |
 | WASM-TASK-011 | Validate WIT package | WASM-TASK-010 | ✅ Complete | ✅ Single-Action |
 | WASM-TASK-012 | Setup wit-bindgen integration | WASM-TASK-011 | ✅ Complete | ✅ Single-Action |

---

## Phase 2: Project Restructuring

**Simple changes - no separate ADR needed**

 | Task ID | Name | Depends On | Status | Compliance |
 |---------|------|------------|--------|------------|
 | WASM-TASK-013 | Rename actor/ to component/ | WASM-TASK-012 | ✅ Complete | ✅ Single-Action |
 | WASM-TASK-014 | Create system/ module | WASM-TASK-013 | ✅ Complete | ✅ Single-Action |
 | WASM-TASK-015 | Create messaging/ module | WASM-TASK-014 | ✅ Complete | ✅ Single-Action |
 | WASM-TASK-016 | Update lib.rs exports | WASM-TASK-015 | ✅ Complete | ✅ Single-Action |

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

 | Task ID | Name | Depends On | Status | Compliance |
 |---------|------|------------|--------|------------|
 | WASM-TASK-017 | Create core/component/ submodule | WASM-TASK-016 | ✅ Complete | ✅ Single-Action |
 | WASM-TASK-018 | Create core/runtime/ submodule | WASM-TASK-017 | ✅ Complete | ✅ Single-Action |
 | WASM-TASK-019 | Create core/messaging/ submodule | WASM-TASK-017 | ✅ Complete | ✅ Single-Action |
 | WASM-TASK-020 | Create core/security/ submodule | WASM-TASK-017 | ✅ Complete | ✅ Single-Action |
 | WASM-TASK-021 | Create core/storage/ submodule | WASM-TASK-017 | ✅ Complete | ✅ Single-Action |
 | WASM-TASK-022 | Create core/errors/ submodule | WASM-TASK-017 | ⚠️ **ABANDONED** | N/A |
 | WASM-TASK-023 | Create core/config/ submodule | WASM-TASK-017 | ✅ Complete | ✅ Single-Action |
 | WASM-TASK-024 | Write core/ unit tests | WASM-TASK-022 | ✅ Complete | ⚠️ **Pre-Updated Policy** |

> **NOTE on WASM-TASK-024:** This task writes tests for multiple core/ submodules (component/, runtime/, messaging/, security/, storage/, config/) simultaneously. This was completed before the updated task management instructions (2026-01-12) which now prohibit multi-file testing tasks. Future testing tasks must be single-file focused.

---

## Phase 4: Security Module (Layer 2A)

**Detailed Spec:** [ADR-WASM-029](adr-wasm-029-security-module-design.md)

 | Task ID | Name | Depends On | Status | Compliance |
 |---------|------|------------|--------|------------|
 | WASM-TASK-025 | Create security/capability/ submodule | WASM-TASK-024 | ✅ Complete | ✅ Single-Action + Tests |
 | WASM-TASK-026 | Implement CapabilityValidator | WASM-TASK-025 | ✅ Complete | ✅ Single-Action + Tests |
 | WASM-TASK-027 | Create security/policy/ submodule | WASM-TASK-025 | ✅ Complete | ✅ Single-Action + Tests |
 | WASM-TASK-028 | Implement SecurityAuditLogger | WASM-TASK-025 | ✅ Complete | ✅ Single-Action + Tests |
 | WASM-TASK-029 | Create airssys-osl bridge | WASM-TASK-026 | ✅ Complete | ✅ Single-Action + Tests |
 | ~~WASM-TASK-030~~ | ~~Write security/ unit tests~~ | WASM-TASK-029 | ❌ **ABANDONED** | ❌ Violated Single-Action |

> **NOTE on WASM-TASK-030:** This task was **ABANDONED** on 2026-01-12 for violating the single-action principle. It attempted to write unit tests for 8 security module files simultaneously. All security modules (025-029) already included tests in their creation tasks, following the rule that "module creation = includes testing." No separate testing tasks are allowed.

---

## Phase 5: Runtime Module (Layer 2B)

**Detailed Spec:** [ADR-WASM-030](adr-wasm-030-runtime-module-design.md)

 | Task ID | Name | Depends On | Status | Compliance |
 |---------|------|------------|--------|------------|
 | WASM-TASK-031 | Implement WasmtimeEngine | N/A (start of Phase 5) | Not Started | TBD |
 | WASM-TASK-032 | Implement ComponentLoader | WASM-TASK-031 | Not Started | TBD |
 | WASM-TASK-033 | Implement StoreManager | WASM-TASK-031 | Not Started | TBD |
 | WASM-TASK-034 | Implement host functions | WASM-TASK-031 | Not Started | TBD |
 | WASM-TASK-035 | Implement ResourceLimiter | WASM-TASK-031 | Not Started | TBD |
 | WASM-TASK-036 | Write runtime/ unit tests | WASM-TASK-035 | Not Started | TBD |

> ⚠️ **MANDATORY**: Use `wasmtime::component::Component` (Component Model), NOT `wasmtime::Module` (core WASM). See KNOWLEDGE-WASM-027.
> ⚠️ **MANDATORY (UPDATED 2026-01-12)**: Module creation tasks must include tests in the SAME task. No separate "write unit tests" tasks allowed for Phase 5 or any subsequent phases.

---

## Phase 6: Component & Messaging Modules (Layer 3)

**Detailed Spec:** [ADR-WASM-031](adr-wasm-031-component-messaging-design.md)

 | Task ID | Name | Depends On | Status | Compliance |
 |---------|------|------------|--------|------------|
 | WASM-TASK-037 | Implement ComponentWrapper | WASM-TASK-036 | Not Started | TBD |
 | WASM-TASK-038 | Implement ComponentRegistry | WASM-TASK-037 | Not Started | TBD |
 | WASM-TASK-039 | Implement ComponentSpawner | WASM-TASK-037 | Not Started | TBD |
 | WASM-TASK-040 | Implement SupervisorConfig | WASM-TASK-037 | Not Started | TBD |
 | WASM-TASK-041 | Implement fire-and-forget pattern | WASM-TASK-038 | Not Started | TBD |
 | WASM-TASK-042 | Implement request-response pattern | WASM-TASK-041 | Not Started | TBD |
 | WASM-TASK-043 | Implement CorrelationTracker | WASM-TASK-042 | Not Started | TBD |
 | WASM-TASK-044 | Implement ResponseRouter | WASM-TASK-043 | Not Started | TBD |
 | WASM-TASK-045 | Write component/ unit tests | WASM-TASK-040 | Not Started | TBD |
 | WASM-TASK-046 | Write messaging/ unit tests | WASM-TASK-044 | Not Started | TBD |

> ⚠️ **MANDATORY (UPDATED 2026-01-12)**: Module creation tasks (037-044) must include tests in the SAME task. Tasks 045-046 are legacy task entries and should be **ABANDONED** if created. Module creation = testing, no separate testing tasks.

---

## Phase 7: System Module & Integration (Layer 4)

**Detailed Spec:** [ADR-WASM-032](adr-wasm-032-system-module-design.md)

 | Task ID | Name | Depends On | Status | Compliance |
 |---------|------|------------|--------|------------|
 | WASM-TASK-047 | Implement RuntimeManager | WASM-TASK-046 | Not Started | TBD |
 | WASM-TASK-048 | Implement lifecycle management | WASM-TASK-047 | Not Started | TBD |
 | WASM-TASK-049 | Implement RuntimeBuilder | WASM-TASK-047 | Not Started | TBD |
 | WASM-TASK-050 | Create echo.wasm fixture | WASM-TASK-049 | Not Started | TBD |
 | WASM-TASK-051 | Create counter.wasm fixture | WASM-TASK-050 | Not Started | TBD |
 | WASM-TASK-052 | Create callback.wasm fixture | WASM-TASK-051 | Not Started | TBD |
 | WASM-TASK-053 | Write integration tests | WASM-TASK-052 | Not Started | TBD |
 | WASM-TASK-054 | Write system/ unit tests | WASM-TASK-049 | Not Started | TBD |

> ⚠️ **MANDATORY (UPDATED 2026-01-12)**: Module creation tasks (047-049) must include tests in the SAME task. Task 054 is a legacy entry and should be **ABANDONED** if created. Module creation = testing, no separate testing tasks.

---

## Task Summary

 | Phase | Tasks | Range | Detailed ADR | Status |
 |-------|-------|-------|--------------|--------|
 | Phase 1: WIT Interface System | 11 | WASM-TASK-002 to 012 | ADR-WASM-027 | ✅ Complete (100%) |
 | Phase 2: Project Restructuring | 4 | WASM-TASK-013 to 016 | (inline) | ✅ Complete (100%) |
 | Phase 3: Core Module | 7 | WASM-TASK-017 to 024 | ADR-WASM-028 | ✅ Complete (100%, 022 abandoned) |
 | Phase 4: Security Module | 5 | WASM-TASK-025 to 029 | ADR-WASM-029 | ✅ Complete (100%, 030 abandoned) |
 | Phase 5: Runtime Module | 6 | WASM-TASK-031 to 036 | ADR-WASM-030 | ⏸ Not Started |
 | Phase 6: Component & Messaging | 10 | WASM-TASK-037 to 046 | ADR-WASM-031 | ⏸ Not Started |
 | Phase 7: System & Integration | 8 | WASM-TASK-047 to 054 | ADR-WASM-032 | ⏸ Not Started |
 | **TOTAL** | **51** | WASM-TASK-002 to 054 | |

> **UPDATE (2026-01-12):** WASM-TASK-022 abandoned, WASM-TASK-030 abandoned. Legacy testing tasks (045, 046, 054) noted as should-be-abandoned. Total reflects active tasks.

---

## Verification Strategy

### Per-Task Verification (UPDATED 2026-01-12)

Each task must include:
1. **Module creation = testing**: When creating a module/submodule, unit tests MUST be included in the same task
2. **Build check**: `cargo build -p airssys-wasm`
3. **Lint check**: `cargo clippy -p airssys-wasm --all-targets -- -D warnings`
4. **Module boundary check**: Verify import direction compliance per ADR-WASM-023

### Module Boundary Checks (CI Enforcement)

```bash
# All must return empty for compliance
grep -rn "use crate::component" src/runtime/
grep -rn "use crate::messaging" src/runtime/
grep -rn "use crate::system" src/runtime/
grep -rn "use crate::runtime" src/security/
grep -rn "use crate::" src/core/  # core imports only std
```

### Task Compliance Rules (UPDATED 2026-01-12)

**IMMEDIATE TASK REJECTION:**
- ✗ Test multiple files or modules → REJECT IMMEDIATELY
- ✗ Enhance multiple existing modules → REJECT IMMEDIATELY
- ✗ "Write unit tests for X module" where X already exists → REJECT IMMEDIATELY
- ✗ Cover multiple files or directories → REJECT IMMEDIATELY
- ✗ Combine multiple phases or stages → REJECT IMMEDIATELY

**MODULE CREATION = INCLUDES TESTING (MANDATORY):**
- When creating a module/submodule → Tests MUST be included in SAME task
- **NO separate testing tasks allowed**
- Unit tests: Go in src/module.rs with #[cfg(test)]
- Integration tests: Go in tests/module-integration-tests.rs
- If tests are incomplete → Task is INCOMPLETE, not "ready for test task later"

---

## Adjustment Protocol

### When to Adjust

1. After completing a task, if new requirements discovered
2. If a task reveals design issues requiring phase changes
3. If external dependencies change (wasmtime, wit-bindgen versions)
4. **NEW (2026-01-12):** If task management policies change significantly

### How to Adjust

1. Update relevant **detailed ADR** (027-032) for implementation changes
2. Update **this ADR** only for phase/task ordering changes
3. Document reason in History section

---

## History

 | Date | Version | Change |
 |------|---------|--------|
 | 2026-01-05 | 1.0 | Initial roadmap creation |
 | 2026-01-05 | 2.0 | Refactored to high-level roadmap with links to detailed ADRs |
 | 2026-01-12 | 3.0 | Updated with task compliance analysis. Added WASM-TASK-022 abandonment notes. Added WASM-TASK-030 abandonment notes. Added compliance column to all task tables. Added reference to updated Multi-Project Memory Bank instructions with single-action guard rules. Marked legacy testing tasks (045, 046, 054) as should-be-abandoned. Added mandatory requirement that module creation tasks include testing in same task. Updated task summary to reflect abandoned tasks. |

---

**This ADR establishes master implementation plan. See linked ADRs for detailed specifications.**
