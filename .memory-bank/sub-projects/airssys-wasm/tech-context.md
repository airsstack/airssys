# airssys-wasm Tech Context

**Last Updated:** 2026-01-05

---

## Status
**Current State:** 🚀 **FRESH START - CODEBASE DELETED**
**Active Sub-Project:** airssys-wasm

---

## What Happened
**Critical Event:** PROJECT CATASTROPHE (2026-01-04)

**Root Cause:**
Complete airssys-wasm codebase was deleted due to repeated, irrecoverable architectural violations across multiple tasks (WASM-TASK-014 Phase 4, etc.)

**Violations Discovered:**
- `core/` → `runtime/` ❌ (FORBIDDEN - ADR-WASM-023)
- `runtime/` → `actor/` ❌ (FORBIDDEN - ADR-WASM-023)
- `security/` → `runtime/` ❌ (FORBIDDEN - ADR-WASM-023)
- `security/ → `actor/` ❌ (FORBIDDEN - ADR-WASM-023)
- `core/` → ANY MODULE ❌ (FORBIDDEN - core/ must import NOTHING)

**AI Agent Failures:**
- Claims of "verified" without evidence (grep output)
- Plans didn't reference ADRs/Knowledges
- Implementation proceeded without reading architecture documentation
- Stub tests created instead of REAL tests
- Multiple "hotfix" attempts made violations worse

**Impact:**
- Loss of 10+ days of development work
- Complete loss of user trust in AI agents
- Architecture broken beyond repair
- Project deletion (entire codebase)

**User Response:**
- Demanded complete rebuild from scratch
- Enforced new task management format with strict single-action rule
- Plans MUST reference ADRs and Knowledges
- Verification workflow is non-negotiable

---

## Current Recovery Work

### Task Management Refactoring (2026-01-04)
**Completed:**
- ✅ Updated Memory Bank instructions file with new task format
  - Enforced: `tasks/<task-id>/` directory structure
  - Enforced: Two files per task (task.md + plans.md)
  - Enforced: SINGLE action per task rule
  - Enforced: Plans MUST reference ADRs and Knowledges

**Key Changes:**
- Old format: Multi-phase tasks with scattered files
- New format: Single-action tasks in directories with task.md + plans.md

### Task Creation (2026-01-04)
**Completed:**
- ✅ Created WASM-TASK-001 (setup-project-directory)
  - Task directory: `tasks/wasm-task-001-setup-project-directory/`
  - Task file: `wasm-task-001-setup-project-directory.md`
  - Plans file: `wasm-task-001-setup-project-directory.plans.md`
  - Task index: Updated `tasks/_index.md`

**Status:** ✅ COMPLETE (all 11 Phase 1 tasks created)

**Phase 1 Tasks Created (2026-01-05):**
- WASM-TASK-002 through WASM-TASK-012 (11 tasks total)
- All tasks have task.md + plans.md files  
- All plans reference ADR-WASM-027 (WIT Interface Design)
- All plans reference KNOWLEDGE-WASM-037 (Clean Slate Architecture)
- All tasks follow single-action rule

**Deliverables:**
- 11 task directories created with complete documentation
- tasks/_index.md updated to register all Phase 1 tasks
- All tasks pending, ready for implementation
- Clean-slate rebuild documentation complete:
  - ADR-WASM-025 (Clean-Slate Rebuild Architecture)
  - ADR-WASM-026 (Implementation Roadmap: 7 phases, 53 tasks)
  - ADR-WASM-027 (WIT Interface Design)
  - KNOWLEDGE-WASM-037 (Rebuild Architecture - Clean Slate Design)

---

## Recovery State

### What We Have Left
**Architecture Documentation (100% Intact)**
- ✅ 22 Architecture Decision Records (ADRs)
- ✅ 22 Knowledge Documents
- ✅ All critical ADR-WASM-023 (Module Boundary Enforcement)
- ✅ All critical KNOWLEDGE-WASM-030 (Module Architecture Requirements)
- ✅ All critical KNOWLEDGE-WASM-031 (Foundational Architecture)
- ✅ KNOWLEDGE-WASM-001 (Component Framework Architecture)
- ✅ Multiple historical ADRs on messaging, runtime, security, integration

**Memory Bank Structure (Intact)**
- ✅ Updated instructions file with new task format
- ✅ Task management refactored for single-action enforcement
  ✅ Task directory structure established

**Project Context Files (Updated 2026-01-05)**
- ✅ active-context.md (Phase 1 tasks status)
- ✅ current-context.md (clean-slate rebuild focus)
- ✅ progress.md (Phase 1 tasks created)
- ✅ project-brief.md (six-module architecture)
- ✅ system-patterns.md (new architecture patterns)
- ✅ tech-context.md (recovery status)

**Workspace Files**
- ✅ Cargo.toml (provides all dependencies)
- ✅ PROJECTS_STANDARD.md (workspace standards)

**NO SOURCE CODE:**
- airssys-wasm/src/ directory DOES NOT EXIST
- airssys-wasm/tests/ directory DOES NOT EXIST
- airssys-wasm/wit/ directory DOES NOT EXIST

---

## Recovery Approach

### Phase 1: Foundation (Current: WASM-TASK-001)
**Objective:** Establish clean project foundation

**Strategy:**
1. Implement WASM-TASK-001 following plans.md exactly
2. Run ALL verification commands (grep for architecture, cargo build, clippy)
3. Only mark complete when ALL verifications pass
4. Trigger @memorybank-verifier for all subagent reports

**Next Tasks (Phase 1: WIT Interface System):**
1. WASM-TASK-002: Setup WIT Directory Structure (ADR-WASM-027)
2. WASM-TASK-003 through WASM-TASK-012: Create WIT interfaces
3. Follow ADR-WASM-026 roadmap sequence

**Critical:**
- MUST follow ADR-WASM-023 module boundaries
- MUST read ALL relevant ADRs before any implementation
- MUST run verification commands and show ACTUAL output
- MUST write REAL tests, not stubs
- Plans MUST reference documentation with full citations

**Risk Mitigation:**
- Verification workflow is non-negotiable
- All subagent reports MUST be verified by @memorybank-verifier
- Implementation will be audited by @memorybank-auditor
- Auditor reports will be verified by @memorybank-verifier
- Only complete when ALL verifications pass

---

## Standards Compliance

### Workspace Standards (PROJECTS_STANDARD.md)
**MUST BE FOLLOWED:**
- ✅ §2.1 3-Layer Import Organization
- ✅ §3.2 chrono DateTime<Utc> Standard
- ✅ §4.3 Module Architecture Patterns
- ✅ §5.1 Dependency Management
- ✅ §6.1 YAGNI Principles
- ✅ §6.2 Avoid `dyn` Patterns
- ✅ §6.4 Implementation Quality Gates

### Architecture Compliance (ADR-WASM-023)
**MUST BE FOLLOWED:**
- ✅ core/ imports NOTHING
- ✅ security/ imports core/ ONLY
- ✅ runtime/ imports core/, security/ ONLY
- ✅ actor/ imports core/, security/, runtime/
- ✅ NO reverse imports

**Verification Commands (MUST PASS):**
```bash
grep -rn "use crate::runtime" src/core/
grep -rn "use crate::actor" src/core/
grep -rn "use crate::runtime" src/security/
grep -rn "use crate::actor" src/security/
grep -rn "use crate::actor" src/runtime/
```
**Expected:** ALL return empty

---

## Success Criteria

WASM-TASK-001 is COMPLETE when:
- [ ] Cargo.toml created
- [ ] Module directories match ADR-WASM-011 four-module structure
- [ ] lib.rs entry point created
- [ ] All verification commands pass
- [ ] All module boundary checks pass
- [ ] Zero compiler/clippy warnings

**Definition of Done:**
- [ ] All deliverables complete
- [ ] Architecture verified (all grep commands empty)
- [ ] Zero warnings
- [ ] Documentation updated

---

## Current Focus

**Priority:** Execute WASM-TASK-001
- Read plans.md carefully
- Reference ALL 22 ADRs and 22 Knowledge documents before each action
- Run verification after each action
- Show ACTUAL grep/cargo output as proof

**We will NOT:**
- Skip reading ADRs/Knowledges
- Create stub tests
- Make assumptions
- Violate ADR-WASM-023
- Claim "verified" without evidence
- Mark tasks "complete" without actual verification

**We WILL:**
- Read ALL documentation before any action
- Run ALL verification commands
- Show ACTUAL output (grep, cargo build, clippy)
- Write REAL tests (not stubs)
- Only mark complete when ALL verifications pass
- Trigger @memorybank-verifier for every subagent report
- Trigger @memorybank-auditor after implementation
- Trigger @memorybank-verifier after auditor
