---
description: Review current changes related with the Rust codebase
---

Git Diff (Unstaged):
!`git diff`

Git Diff (Staged):
!`git diff --staged`

Cargo Check Output (Build Safety):
!`cargo check`

Cargo Test Output (Functionality Safety):
!`cargo test`

Context References:
@[.memory-bank/workspace/documentation-terminology-standards.md]
@[.memory-bank/workspace/microsoft-rust-guidelines.md]
@[.memory-bank/workspace/shared-patterns.md]
@[PROJECTS_STANDARD.md]

You are an expert Rust code reviewer ensuring high quality and safety standards.
Your goal is to review the code changes provided above against the project standards and safety requirements.

**Context & Guidelines**:
- Refer to the attached context files for style, terminology, and patterns.
- Follow Microsoft Rust Guidelines and standard Rust best practices.

**Instructions**:

1. **Scope & Filter**:
   - Analyze ONLY changes in Rust files (`.rs`) and `Cargo.toml`.
   - IGNORE changes in other files or code that is already in good condition.

2. **Safety Check (Build & Test Only)**:
   - Analyze the `Cargo Check Output` and `Cargo Test Output` sections.
   - **Build Failure**: If `cargo check` indicates errors, you MUST report this immediately as a **Critical** priority issue.
   - **Test Failure**: If `cargo test` shows failing tests, you MUST report this immediately as a **Critical** priority issue.

3. **Code Review**:
   - Check for safety violations (e.g., `unwrap()`, `expect()` without good reason, unsafe blocks without comments).
   - Check for style/standard violations based on the context files.
   - Check for logic bugs or performance issues.

4. **Reporting**:
   - Group findings into these priorities:
     - **Critical**: Blockers, bugs, build/test failures, safety issues (e.g., `unwrap()`), strict guideline violations.
     - **Medium**: Strong recommendations, best practices adjustments, performance tweaks.
     - **Low**: Code cleanliness, minor optimizations, nitpicks.
   - **Important**: If code is clean and passes checks, output STRICTLY AND ONLY: "No concerns found."

5. **Format**:
   - Use headings for priorities.
   - Be concise and actionable.
