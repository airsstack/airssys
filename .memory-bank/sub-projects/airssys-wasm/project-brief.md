# airssys-wasm Project Brief

**Last Updated:** 2026-01-04

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

### First Task
**Task ID:** WASM-TASK-001
**Task Name:** Setup airssys-wasm Project Directory
**Status:** pending
**Priority:** high
**Description:** Create basic project structure (Cargo.toml, src/ directories) as foundation for all subsequent tasks

**Why First:**
- Must establish correct project structure before any code implementation
- Architecture violations from previous project must NOT be repeated
- Foundation tasks must reference existing ADRs and Knowledges

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

## Project Structure (After WASM-TASK-001 Complete)

```
airssys-wasm/
├── src/
│   ├── core/      # Shared types (imports NOTHING)
│   ├── security/  # Security logic (imports core/)
│   ├── runtime/   # WASM execution (imports core/, security/)
│   └── actor/     # Actor integration (imports core/, security/, runtime/)
├── tests/         # Integration tests
│   └── wit/           # WIT interfaces
└── Cargo.toml
└── README.md
```

**Dependency Hierarchy (MANDATORY - ADR-WASM-023):**
```
ALLOWED:
  ✅ actor/ → runtime/
  ✅ actor/ → security/
  ✅ actor/ → core/
  ✅ runtime/ → security/
  ✅ runtime/ → core/
  ✅ security/ → core/

FORBIDDEN (NEVER):
  ❌ runtime/ → actor/
  ❌ security/ → runtime/
  ❌ security/ → actor/
  ❌ core/ → ANY MODULE
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

## Architecture Documentation (INTACT - 22 ADRs, 22 Knowledge docs)

### Critical Documents (MUST READ)
**Foundation:**
- KNOWLEDGE-WASM-031: Foundational Architecture (READ FIRST)
- KNOWLEDGE-WASM-030: Module Architecture Requirements (MANDATORY)

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

1. **Implement WASM-TASK-001** (Setup Project Directory)
   - Follow implementation plans in plans.md
   - Verify each action before marking complete
   - Run grep commands after implementation
   - Show ACTUAL output as proof

2. **Create foundational tasks** (in order based on ADR-WASM-010 build order)
   - Each task will have single action
   - Each task will reference ADRs/Knowledges
   - Each plan will reference documentation with citations

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
