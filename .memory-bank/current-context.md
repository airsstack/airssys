# Current Context

**Active Sub-Project:** airssys-wasm

---

## Workspace Context

**Status:** 🚀 **FRESH START - PROJECT DELETED**

**What happened:**
- **airssys-wasm project was completely deleted**
- **Root cause:** AI agents repeatedly violated ADR-WASM-023 (Module Boundary Enforcement) and KNOWLEDGE-WASM-030 (Module Architecture Hard Requirements)
- **Multiple violations detected:**
  - `core/` → `runtime/` ❌ (FORBIDDEN)
  - `runtime/` → `actor/` ❌ (FORBIDDEN)
  - Implementation without reading ADRs/Knowledges
  - Claims of "verified" without evidence (grep output)
  - Creation of stub tests instead of REAL tests
  - Claims of "complete" without actual verification

- **Impact:**
  - Loss of user trust
  - Loss of 10+ days of development work
  - Complete project deletion
  - Architecture broken beyond repair

**Resolution:**
- User demanded complete rebuild from scratch
- New task management format enforced (single action per task)
- All architecture documentation intact (22 ADRs, 22 Knowledge docs)
- Fresh start with strict verification workflow

**Current recovery:**
- Memory Bank instructions file updated with new task management format
- WASM-TASK-001 created (setup project directory) - FIRST task using new format
- No code implementation yet - just project structure
- Architecture foundation tasks will follow ADR-WASM-023 strictly

---

## Sub-Project Context

### airssys-wasm
**Status:** 🚀 **REBUILDING FROM SCRATCH**

**What happened:**
- Complete codebase deleted due to architectural violations
- Project directory needs to be recreated from scratch

**Current work:**
- Task: WASM-TASK-001 (Setup Project Directory) - PENDING
- This is foundational task to establish project structure
- No code written yet, only directory structure and Cargo.toml

**What's different now:**
- OLD: Multi-phase tasks with complex tracking, scattered files, violations everywhere
- NEW: Single-action tasks with strict format, verification-first workflow

**Critical constraints:**
- MUST read ADRs/Knowledges before implementing
- MUST run verification commands and show output
- MUST write REAL tests, not stub tests
- MUST follow ADR-WASM-023 module boundaries strictly
- Plans MUST reference ADRs/Knowledges with full citations

**Available documentation:**
- ✅ 22 ADRs (all intact)
- ✅ 22 Knowledge documents (all intact)
- ✅ WASM-TASK-001 task file and plans file created
- ✅ New task management instructions in place

---

## Notes

**Why this is different:**
- OLD project had months of development work but architecture was fundamentally broken
- NEW start with clean slate, correct foundation

**Success criteria:**
- [ ] WASM-TASK-001 complete (Cargo.toml + structure)
- [ ] Build succeeds
- [ ] Architecture verified (all grep commands clean)
- [ ] No warnings (clippy clean)
- [ ] Documentation updated

**Next steps after WASM-TASK-001:**
1. Create foundational task (core/ types)
2. Security task (capabilities)
3. Runtime task (WASM engine)
4. Actor integration task
5. Messaging task

**All tasks will follow new format:**
- Single action per task
- Two files: task.md + plans.md
- Plans MUST reference ADRs/Knowledges
- Verification MANDATORY

**Root cause analysis:**
Previous violations were in:
- KNOWLEDGE-WASM-027 (Duplicate WASM Runtime Fatal Violation)
- KNOWLEDGE-WASM-028 (Circular Dependency actor/runtime)
- KNOWLEDGE-WASM-032 (Module Boundary Violations Audit)

All documented in detail with lessons learned.

**This rebuild must not repeat those mistakes.**
