# airssys-wasm Project Brief

**Last Updated:** 2026-01-05

---

## Project Overview

**Status:** 🚀 **REBUILDING FROM SCRATCH**

**What is airssys-wasm?**
airssys-wasm is a **WASM Component Framework for Pluggable Systems**. It provides infrastructure for component-based architectures, enabling runtime deployment patterns inspired by smart contract systems for general-purpose computing.

**Core Value Proposition:**
- **Cross-Platform Component Framework** - Support any WASM-compatible language (Rust, C/C++, JavaScript, Go, Python)
- **Runtime Deployment** - Components can be loaded/updated without system restart
- **Immutable Versions** - Component versions are immutable and auditable
- **Capability-Based Security** - Fine-grained permission system (deny-by-default)
- **Language Agnostic** - WIT-based interfaces work across languages

---

## Project Recovery Status

### 2026-01-04: CATASTROPHIC EVENT - PROJECT DELETION
**What happened:**
The entire airssys-wasm codebase was deleted due to repeated architectural violations that could not be fixed despite multiple "hotfix" attempts.

**Root Cause:**
- Repeated violations of ADR-WASM-023 (Module Boundary Enforcement)
  - `core/` importing from `runtime/` (FORBIDDEN)
- `runtime/` importing from `actor/` (FORBIDDEN)
- Circular dependencies that created architectural tangles

**Documentation of violations:**
- KNOWLEDGE-WASM-027: Duplicate WASM Runtime - Fatal Architecture Violation
- KNOWLEDGE-Wasm-028: Circular Dependency actor/runtime
- KNOWLEDGE-WASM-032: Module Boundary Violations Audit

**Attempts to fix:**
- Multiple hotfix tasks (WASM-TASK-006, etc.) all claimed "fixed" but violations still existed
- Claims of "verified" without showing actual grep output
- Plans didn't reference ADRs/Knowledges
- Implementer proceeded without checking architecture

**User Response:**
- After discovering violations persisted despite multiple fix attempts, user deleted entire codebase
- Demanded complete rebuild from scratch

**Impact:**
- Loss of 10+ days of development work
- Complete destruction of user trust in AI agents
- Project deletion

---

## Recovery Strategy

### Current Phase: Fresh Start
**Objective:** Rebuild airssys-wasm from scratch with strict verification-first approach

**New Task Management Format (ENFORCED):**
- Tasks in directories: `tasks/<task-identifier>/`
- Two files per task:
  - `<task-id>.md` - Task file (objectives, deliverables, checklist)
  - `<task-id>.plans.md` - Plans file (actions with ADR/Knowledge references)
- SINGLE action per task - DO ONE THING, DO IT RIGHT
- Plans MUST reference ADRs and Knowledge documents

**Verification Rules:**
- Read ADRs/Knowledges BEFORE planning
- Run architecture verification commands and show ACTUAL output
- Write REAL tests, not stubs
- Only proceed when verification passes

###Foundation Complete
**Task ID:** WASM-TASK-001
**Task Name:** Setup airssys-wasm Project Directory
**Status:** ✅ COMPLETE (2026-01-05)
**Priority:** high
**Description:** Created basic project structure (Cargo.toml, src/ directories) as foundation for all subsequent tasks

**Achievement:**
- Established correct project structure before code implementation
- Zero architecture violations (verified with grep commands)
- All documentation references included in plans

### Current Phase
**Phase:** Phase 1 - WIT Interface System
**Tasks:** WASM-TASK-002 through WASM-TASK-012 (11 tasks total)
**Status:** All tasks created, ready to start
**Reference:** ADR-WASM-026 (Implementation Roadmap), ADR-WASM-027 (WIT Interface Design)

**Phase 1 Tasks (All Pending):**
- WASM-TASK-002: Setup WIT Directory Structure
- WASM-TASK-003: Create types.wit
- WASM-TASK-004: Create errors.wit
- WASM-TASK-005: Create capabilities.wit
- WASM-TASK-006: Create component-lifecycle.wit
- WASM-TASK-007: Create host-messaging.wit
- WASM-TASK-008: Create host-services.wit
- WASM-TASK-009: Create storage.wit
- WASM-TASK-010: Create world.wit
- WASM-TASK-011: Validate WIT package
- WASM-TASK-012: Setup wit-bindgen integration

---

## Core Technologies

### WASM Runtime (ADR-WASM-002)
- **Engine:** Wasmtime 24.0 with Component Model support
- **Features:** async, cranelift JIT, fuel metering, memory limits
- **Purpose:** Execute WASM components with proper sandboxing

### AirsSys Integration
**airssys-rt:** Actor system for component hosting (ADR-WASM-018)
- **airssys-osl:** Security layer for system access (ADR-WASM-005)

---

## Project Structure (Clean-Slate Rebuild)

**Reference:** KNOWLEDGE-WASM-037 (Rebuild Architecture - Clean Slate Design)

```
airssys-wasm/
├── src/
│   ├── core/           # LAYER 1: Foundation (std only)
│   ├── security/       # LAYER 2A: Security
│   ├── runtime/        # LAYER 2B: WASM Execution
│   ├── component/      # LAYER 3A: airssys-rt integration (renamed from actor/)
│   ├── messaging/      # LAYER 3B: Messaging patterns (new module)
│   └── system/         # LAYER 4: Coordinator (new module)
├── tests/         # Integration tests
├── wit/           # WIT interfaces
│   └── core/      # Package: airssys:core@1.0.0
└── Cargo.toml
```

**Dependency Hierarchy (NEW - ADR-WASM-025, KNOWLEDGE-WASM-037):**
```
LAYER 4 (system/) → injects concrete implementations into all layers
  ↓
LAYER 3 (component/, messaging/) → depend on core/ traits ONLY + airssys-rt
  ↓
LAYER 2 (runtime/, security/) → depend on core/ (runtime also depends on security/)
  ↓  
LAYER 1 (core/) → depends on std ONLY

KEY PRINCIPLE: Dependency Inversion
- Modules depend on TRAITS (defined in core/), not concrete implementations
- system/ is the only module that knows about concrete types
- system/ injects dependencies into lower layers
```

---

## Key Constraints

### MANDATORY Standards (PROJECTS_STANDARD.md)
- **§2.1 3-Layer Import Organization** - All imports organized std → external → internal
- **§3.2 chrono DateTime<Utc>** - All time operations use Utc
- **§4.3 Module Architecture** - mod.rs only declarations (no implementation)
- **§5.1 Dependency Management** - Dependencies ordered by layer (workspace, then external)
- **§6.1 YAGNI** - Only necessary features
- **§6.2 Avoid `dyn`** - Use generics instead
- **§6.4 Quality Gates** - Zero warnings, >90% coverage

### ADR-WASM-023 (Module Boundary Enforcement) - MANDATORY
- NO forbidden imports in any direction

---

## Architecture Documentation (25+ ADRs, 23+ Knowledge docs)

### Clean-Slate Rebuild Documents (NEW - CRITICAL)
**Foundation:**
- **KNOWLEDGE-WASM-037:** Rebuild Architecture - Clean Slate Design (READ FIRST)
- **ADR-WASM-025:** Clean-Slate Rebuild Architecture (decision record)
- **ADR-WASM-026:** Implementation Roadmap (master plan: 7 phases, 53 tasks)
- **ADR-WASM-027:** WIT Interface Design (Phase 1 specifications)

**Previous Foundation (Historical):**
- KNOWLEDGE-WASM-031: Foundational Architecture
- KNOWLEDGE-WASM-030: Module Architecture Requirements (superseded by KNOWLEDGE-WASM-037)

**Module Structure:**
- ADR-WASM-011: Module Structure Organization
- KNOWLEDGE-WASM-012: Module Structure Architecture

**Security:**
- ADR-WASM-005: Capability-Based Security Model
- KNOWLEDGE-WASM-020: AirsSys Security Integration

**Messaging:**
- ADR-WASM-009: Component Communication Model
- KNOWLEDGE-WASM-005: Messaging Architecture

**Integration:**
- ADR-WASM-018: Three-Layer Architecture
- KNOWLEDGE-WASM-018: Component Definitions
- KNOWLEDGE-WASM-016: Actor System Integration

**Lessons Learned:**
- KNOWLEDGE-WASM-033: AI Fatal Mistakes

---

## Dependencies

### AirsSys Dependencies
- **airssys-osl:** Security layer for system access
- **airssys-rt:** Actor system for component hosting

### External Dependencies
- **Wasmtime:** WASM runtime engine (24.0)
- **wit-bindgen:** WIT code generation (0.47.0)

---

## Success Criteria

### For Fresh Start Success
- [ ] Project structure created (Cargo.toml + modules)
- [ ] All modules follow four-module architecture
- [ ] Architecture verified (all grep commands clean)
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings
- [ ] All documentation referenced in plans

### For Subsequent Tasks
- [ ] ADR-WASM-023 compliance verified
- [ ] KNOWLEDGE-WASM-033 lessons understood
- [ ] STRICT verification workflow followed

---

## Next Steps

1. **Start Phase 1: WIT Interface System** (WASM-TASK-002 through WASM-TASK-012)
   - Follow ADR-WASM-027 specifications exactly
   - Create each WIT file according to task plans
   - Validate with `wasm-tools component wit` after each task
   - Complete wit-bindgen integration (WASM-TASK-012)

2. **After Phase 1 Complete** (Begin Phase 2: Project Restructuring)
   - Follow ADR-WASM-026 roadmap sequence
   - Each phase references specific ADRs
   - Maintain  single-action task discipline

3. **Verification MANDATORY**
   - Read ALL relevant ADRs before planning
   - Run ALL architecture verification commands
   - Show ACTUAL grep output (not just "verified")
   - Write REAL tests (not stubs)

---

## Notes

**Why previous project failed:**
- Multi-phase tasks led to complexity and loss of focus
- No single-action enforcement
- Plans didn't reference documentation
- Claims without evidence
- Verification commands not run or shown
- Architecture violations accumulated without correction

**What's different now:**
- Single-action tasks enforce clear objectives
- Plans MUST reference documentation
- Verification workflow is mandatory
- Architecture verification is automated and evidence-based

**Commitment:**
- We will NOT repeat the same mistakes
- We will ALWAYS read ADRs/Knowledges before planning
- We will ALWAYS run verification commands and show output
- We will write REAL tests, not stubs
