# WASM-TASK-001: Setup airssys-wasm Project Directory

**Status:** pending  
**Added:** 2026-01-04  
**Updated:** 2026-01-04  
**Priority:** high  
**Estimated Duration:** 1 day

## Original Request
Setup airssys-wasm project directory including Cargo.toml and src/ directory structure.

## Thought Process
This is the foundational task for rebuilding airssys-wasm from scratch after the previous project was deleted. Before any code can be written, the project structure must be established. This task creates the directory structure and Cargo.toml that all subsequent tasks will depend on.

Based on the architecture documentation:
- ADR-WASM-011 defines the module structure
- ADR-WASM-023 defines the four-module architecture (core/, security/, runtime/, actor/)
- KNOWLEDGE-WASM-030 defines module responsibilities
- Workspace Cargo.toml provides dependency versions

This task MUST NOT violate any of these constraints.

## Deliverables
- [ ] airssys-wasm/Cargo.toml created with all required dependencies
- [ ] airssys-wasm/src/ directory created
- [ ] airssys-wasm/src/core/ module directory created
- [ ] airssys-wasm/src/security/ module directory created
- [ ] airssys-wasm/src/runtime/ module directory created
- [ ] airssys-wasm/src/actor/ module directory created
- [ ] airssys-wasm/tests/ directory created
- [ ] airssys-wasm/wit/ directory created
- [ ] airssys-wasm/src/lib.rs created with module declarations

## Success Criteria
- [ ] `cargo build -p airssys-wasm` succeeds
- [ ] Module structure matches ADR-WASM-023 (core/, security/, runtime/, actor/)
- [ ] Dependencies reference workspace versions correctly
- [ ] No compiler warnings
- [ ] All mod.rs files contain only declarations (§4.3 compliance)

## Progress Tracking
**Overall Status:** 0% complete

| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 1.1 | Create Cargo.toml | not_started | 2026-01-04 | Ready to implement |
| 1.2 | Create module directories | not_started | 2026-01-04 | Ready to implement |
| 1.3 | Create lib.rs entry point | not_started | 2026-01-04 | Ready to implement |

## Progress Log
*(No progress yet)*

## Standards Compliance Checklist
**Workspace Standards Applied** (Reference: PROJECTS_STANDARD.md):
- [ ] **§2.1 3-Layer Import Organization** - Evidence: All imports organized by std → external → internal
- [ ] **§3.2 chrono DateTime<Utc> Standard** - Evidence: N/A (no time operations in this task)
- [ ] **§4.3 Module Architecture Patterns** - Evidence: mod.rs files contain only declarations/re-exports
- [ ] **§5.1 Dependency Management** - Evidence: Dependencies ordered by layer, reference workspace
- [ ] **§6.1 YAGNI Principles** - Evidence: Only necessary structure created, no extra features
- [ ] **§6.2 Avoid `dyn` Patterns** - Evidence: N/A (no trait objects needed here)
- [ ] **§6.4 Implementation Quality Gates** - Evidence: Zero warnings, clean build

**Memory Bank References:**
- [ ] **ADR-WASM-011:** Module Structure Organization
- [ ] **ADR-WASM-023:** Module Boundary Enforcement
- [ ] **KNOWLEDGE-WASM-012:** Module Structure Architecture
- [ ] **KNOWLEDGE-WASM-030:** Module Architecture Hard Requirements
- [ ] **KNOWLEDGE-WASM-031:** Foundational Architecture

## Compliance Evidence
*(To be filled during implementation)*

## Definition of Done

### Mandatory Criteria (ALL must be true to mark complete)
- [ ] **All subtasks complete**
- [ ] **All deliverables created**
- [ ] **All success criteria met**
- [ ] **Code Quality (Zero Warnings)**
   - [ ] `cargo build -p airssys-wasm` completes cleanly
   - [ ] `cargo clippy -p airssys-wasm --all-targets --all-features -- -D warnings` passes
   - [ ] No compiler warnings
   - [ ] No clippy warnings

- [ ] **Standards Compliance (Per PROJECTS_STANDARD.md)**
   - [ ] §2.1 3-Layer Import Organization
   - [ ] §4.3 Module Architecture Patterns
   - [ ] §5.1 Dependency Management
   - [ ] ADR-WASM-023 Module Boundary Enforcement

- [ ] **Architecture Verification**
   - [ ] core/ imports nothing (only std)
   - [ ] security/ imports only core/
   - [ ] runtime/ imports only core/, security/
   - [ ] actor/ imports all three modules
