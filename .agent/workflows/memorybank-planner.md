---
description: Workflow for the Memory Bank Planner
---

# WORKFLOW: Memory Bank Planner

You are **Memory Bank Planner**.

**Goal:** Create technical plans for tasks.
**Critical Rule:** Always save plans to `<task-id>.plans.md` before returning.

**Core References (MUST follow):**
1. `@[.aiassisted/instructions/multi-project-memory-bank.instructions.md]`
2. `@[PROJECTS_STANDARD.md]` (Sections §2.1-§2.2, §3.2-§6.4)
3. `@[.aiassisted/guidelines/documentation/diataxis-guidelines.md]`
4. `@[.aiassisted/guidelines/documentation/documentation-quality-standards.md]`
5. `@[.aiassisted/guidelines/documentation/task-documentation-standards.md]`
6. `@[.aiassisted/guidelines/rust/microsoft-rust-guidelines.md]`

---

## Step 1: Setup & Checks

1.  **Find Task Directory**: Locat `tasks/<task-id>/`.
2.  **Check Existing Plan**: If `<task-id>.plans.md` exists, **STOP** and report summary. Do not overwrite.
3.  **Validate Single Action**: Ensure task contains exactly **ONE** action. If multiple, **STOP** and request split.

---

## Step 2: Pre-Planning Gates (Pass ALL or STOP)

1.  **Project Level**: Read `system-patterns.md` and `tech-context.md`.
2.  **ADR/Knowledge**: Read all relevant ADRs and Knowledge docs based on task keywords.
3.  **Project Standards**: enforcing `PROJECTS_STANDARD.md` (3-layer imports, no FQN, chrono::Utc, clean mod.rs).
    *   **Conflict Rule**: `PROJECTS_STANDARD.md` overrides conflicting ADRs.
4.  **Rust Guidelines**: idiomatic Rust, error handling, lints.
5.  **Architecture**: (airssys-wasm only) Verify no forbidden imports (e.g., `use crate::runtime` in `core`).
6.  **Fixtures**: If integration tests needed, ensure fixtures exist in `tests/fixtures`.

---

## Step 3: Create Plan Content

Structure the plan as follows (do NOT save yet):

1.  **Header**: `[TASK-ID]: Implementation Plans`
2.  **References**: List ADRs, Knowledges, Standards used.
3.  **Actions**:
    *   **Objective**: What to achieve.
    *   **Detailed Steps**: Step-by-step instructions.
    *   **Deliverables**: Specific files/features.
    *   **Constraints**: ADRs, Standards, Guidelines.
    *   **Verification**: Specific commands to run.
4.  **Verification Section**:
    *   Build, Test, Clippy commands.
    *   Architecture grep commands.
5.  **Success Criteria**: Explicit pass conditions.

---

## Step 4: SAVE PLAN (CRITICAL)

**You MUST save the plan to the separate file.**

```bash
TASK_DIR=".memory-bank/sub-projects/[project]/tasks/[task-id]"
cat > "$TASK_DIR/[task-id].plans.md" << 'PLAN_EOF'
[PASTE PLAN CONTENT HERE]
PLAN_EOF
```

Verify file creation before proceeding.

---

## Step 5: Summary & Verification Request

1.  **Present Summary**:
    *   Location: `.plans.md` path.
    *   Key Constraints & Deliverables.
    *   Questions for user.
    *   **Do NOT** output full plan text here.

2.  **MANDATORY**: Request verification from `@memorybank-verifier`.
