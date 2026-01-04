# airssys-wasm Active Context

**Last Updated:** 2026-01-04  
**Active Sub-Project:** airss-wasm  
**Current Status:** 🚀 **FRESH START - PROJECT REBIRTH**

## Current Focus

### Primary Task
**Task ID:** WASM-TASK-001  
**Task Name:** Setup airss-wasm Project Directory  
**Status:** pending  
**Priority:** high  
**Estimated Duration:** 1 day

**Description:**
This is a foundational task for rebuilding airssys-wasm from scratch. The entire airssys-wasm codebase was deleted due to repeated architectural violations. This task creates the basic project structure (Cargo.toml, src/ directory with modules) before any code implementation can begin.

### Why This Task is Critical
- All subsequent tasks depend on correct project structure
- Architecture violations from previous iterations caused the project deletion
- This is the ONLY chance to rebuild correctly
- MUST follow ADR-WASM-023 and KNOWLEDGE-WASM-030 strictly

---

## Recent Work

### 2026-01-04: Task Creation & Memory Bank Refactoring
**Completed:**
- ✅ Updated Memory Bank instructions with new task management format
- ✅ Created task structure: `tasks/wasm-task-001-setup-project-directory/`
- ✅ Created task file: `wasm-task-001-setup-project-directory.md`
- ✅ Created plans file: `wasm-task-001-setup-project-directory.plans.md`
- ✅ Updated task index: `tasks/_index.md`

**Key Achievement:**
- New task format enforces: SINGLE action per task, plans reference ADRs/Knowledges
- Plans MUST document references to architecture documents before implementation

---

## Next Steps

1. **Immediate:** Implement WASM-TASK-001 (Setup Project Directory)
   - Action 1: Create Cargo.toml
   - Action 2: Create module structure
   - Action 3: Create lib.rs entry point
   - Action 4: Create test fixtures directory
   - Action 5: Create WIT directory

2. **Verification:** Run architecture verification commands after implementation
   - Ensure ADR-WASM-023 compliance
   - Ensure zero compiler/clippy warnings

---

## Context Notes

### Previous State (DELETED)
**Old Task:** WASM-TASK-014 (Block 1 - Host System Architecture Implementation)
- **Old Phase:** Block 5 (Refactor ActorSystemSubscriber)
- **Old Status:** Phase 4 COMPLETE (100%), Phase 5 IN PROGRESS
- **Problem:** The old task directory and all progress files for WASM-TASK-014 were deleted with the airssys-wasm codebase
- **Resolution:** Fresh start required with new task management format

### Current Decision
**Mode:** Clean rebuild from scratch  
**Approach:** Start with foundational setup task (WASM-TASK-001)  
**Focus:** Establish correct project structure before any implementation  
**Risk:** Repeating same violations would cause another project deletion

---

## Technical Context

### Module Architecture (MANDATORY)
**Four-Module Structure (ADR-WASM-023):**
```
airssys-wasm/src/
├── core/      # Foundation - shared types, imports NOTHING
├── security/  # Security logic - imports core/
├── runtime/   # WASM execution - imports core/, security/
└── actor/     # Actor integration - imports core/, security/, runtime/
```

**Dependency Rules:**
- ❌ runtime/ → actor/ (FORBIDDEN)
- ❌ security/ → runtime/ (FORBIDDEN)
- ❌ security/ → actor/ (FORBIDDEN)
- ❌ core/ → ANY MODULE (FORBIDDEN)

**Verification Commands:**
```bash
# ALL must return NOTHING for valid architecture
grep -rn "use crate::runtime" src/core/
grep -rn "use crate::actor" src/core/
grep -rn "use crate::runtime" src/security/
grep -rn "use crate::actor" src/security/
grep -rn "use crate::actor" src/runtime/
```

### Reference Documentation
- **ADR-WASM-023:** Module Boundary Enforcement (MANDATORY)
- **KNOWLEDGE-WASM-030:** Module Architecture Hard Requirements (MANDATORY)
- **KNOWLEDGE-WASM-031:** Foundational Architecture (READ FIRST)
- **KNOWLEDGE-WASM-012:** Module Structure Architecture

---

## Dependencies

### Upstream
- None - This is the first task after project deletion

### Downstream
- All subsequent airssys-wasm tasks
- Will depend on correct project structure established by this task

---

## Definition of Done Criteria

- [ ] Project structure created (Cargo.toml + src/)
- [ ] Module directories match four-module architecture
- [ ] All architecture verification commands pass
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings
- [ ] Documentation references in plans file
