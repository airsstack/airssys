---
description: Commit current changes using conventional commits standard format
instructions: .aiassisted/instructions/conventional-commits.instructions.md
---

# Variables

```
$ROOT_PROJECT = $(git rev-parse --show-toplevel)
$AIASSISTED_DIR = $ROOT_PROJECT/.aiassisted/
$PROMPT = $AIASSISTED_DIR/prompts/git.commit.prompt.md
$INSTRUCTION = $AIASSISTED_DIR/instructions/conventional-commits.instructions.md
```

# Git Context

Git Status:
!`git status`

Git Diff (Unstaged):
!`git diff`

Git Diff (Staged):
!`git diff --staged`

Recent Commits (for reference):
!`git log --oneline -5`

# Instructions

You MUST read and understand the following files before proceeding:
1. Read the prompt template at: `$PROMPT`
2. Read the detailed conventional commits specification at: `$INSTRUCTION`

# Workflow Steps

1. **Analyze Git State**:
   - **Unstaged changes**: Identify modified files and suggest `git add` commands
   - **Staged changes**: Focus on these for the commit message
   - **Untracked files**: Identify files that should be committed (exclude build artifacts, IDE configs)
   - **No changes**: Inform the user there is nothing to commit

2. **Determine Scope** (Project-Specific):
   - Analyze file paths of ALL changes (staged, unstaged, untracked)
   - Review recent commits to maintain consistency with existing commit patterns
   - Apply scope rules:
     * **Sub-project changes** (e.g., `airssys-osl/`, `airssys-rt/`): Use directory name as scope
     * **Documentation** (`.memory-bank/`, `.aiassisted/`, root docs): Use `docs` or omit scope
     * **Configuration** (`.opencode/`, `.github/`, root configs): Use `config` or no scope with `chore` type
     * **Multi-project/mixed**: Omit scope or use `workspace` only if it adds clarity
   - Include untracked files that are part of normal workflow (snapshots, new sources, etc.)

3. **Detect Breaking Changes**:
   - Check if changes modify public APIs, remove features, or change behavior
   - If breaking: Add `!` after type/scope OR include `BREAKING CHANGE:` footer

4. **Generate Commit Message**:
   - Follow the format from `$INSTRUCTION`
   - Subject line: Imperative mood, under 50 chars, no period
   - Body (if needed): Wrap at 72 chars, explain what and why
   - Footer (if needed): Reference issues, document breaking changes

5. **Execute Commands**:
   - Add relevant files using `git add` commands
   - Execute `git commit` with the generated message
   - Show the commit result to the user

# Examples

Here are illustrative examples of the commit message patterns you should follow. These demonstrate the format - you will generate and execute similar commands based on the actual changes:

```bash
# Example 1: Sub-project feature
git add airssys-osl/src/lib.rs
git commit -m "feat(airssys-osl): add new os layer abstraction"

# Example 2: Breaking change
git add airssys-rt/src/actor/mod.rs
git commit -m "feat(airssys-rt)!: remove deprecated actor spawn method"

# Example 3: Documentation
git add .memory-bank/context-snapshots/2025-11-30-task-complete.md
git commit -m "docs: add task completion snapshot"

# Example 4: Configuration
git add .opencode/command/git.commit.md
git commit -m "chore: update git commit command with prompt references"

# Example 5: With body and footer
git add airssys-wasm/src/runtime/mod.rs
git commit -m "fix(airssys-wasm): resolve memory leak in component cleanup

Previously, component instances were not properly dropped after
execution, causing memory to accumulate over time.

Closes #42"
```

# Execution Notes

- Generate appropriate `git add` and `git commit` commands based on the actual changes (not the examples above)
- Always execute the `git add` commands for relevant files first
- Then execute the `git commit` command with the generated message
- Verify the commit succeeded by checking the output
- If commit fails, report the error to the user
