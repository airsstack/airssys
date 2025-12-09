---
description: Commit current changes using conventional commits standard format
---

Git Status:
!`git status`

Git Diff (Unstaged):
!`git diff`

Git Diff (Staged):
!`git diff --staged`

You are an expert developer assistant helping to commit changes.
Your goal is to generate a valid `git commit` command with a message following the Conventional Commits specification.

Steps:
1. **Analyze Changes**: behavior depends on the git status.
   - **Unstaged changes present**: Identify which files are modified. Suggest `git add` commands for them, or use `git commit -a` if appropriate.
   - **Staged changes present**: Focus on these for the commit message.
   - **Untracked files present**: Identify untracked files that should be committed (new source files, documentation, configs). Exclude files that should be in `.gitignore` (build artifacts, IDE configs, etc.). Provide `git add` commands for relevant files.
   - **No changes**: Inform the user there is nothing to commit.

2. **Determine Scope**:
   - Look at the file paths of all changes (staged, unstaged, and untracked).
   - Scope determination rules:
     * If changes are within a specific sub-project directory (e.g., `airssys-osl/src/lib.rs` -> scope `airssys-osl`), use that directory name as the scope.
     * If changes are in documentation directories like `.memory-bank/`, `.aiassisted/`, or root-level docs, use scope `docs` or omit scope.
     * If changes are in configuration files (`.opencode/`, `.github/`, root config files), use scope `config` or `chore` type without scope.
     * If changes span multiple sub-projects or are mixed types, omit the scope or use a broad scope like `workspace` only if it adds clarity.
   - Format: `type(scope): description` or `type: description` if no scope.
   - **Important**: Consider ALL untracked files that should be committed as part of normal workflow (documentation snapshots, new source files, etc.).

3. **Generate Message**:
   - Determine the type: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`.
   - Type selection guidelines:
     * Use `docs` for documentation changes (including `.memory-bank/`, `README.md`, doc comments)
     * Use `chore` for maintenance tasks, tooling, and configuration changes
     * Use `feat` for new features or capabilities
     * Use `fix` for bug fixes
   - Write a concise description that explains what changed and why.
   - If unstaged or untracked files need to be committed, include the necessary `git add` step(s) in your proposed command sequence.

4. **Output**:
   - Provide the exact command(s) to run.
   - Explain the reasoning briefly.

Example Output:
```bash
# For sub-project changes:
git add airssys-osl/src/lib.rs
git commit -m "feat(airssys-osl): add new os layer abstraction"

# For documentation changes:
git add .memory-bank/context-snapshots/2025-11-30-task-complete.md
git commit -m "docs: add task completion snapshot"

# For configuration changes:
git add .opencode/command/new-command.md
git commit -m "chore: add custom git commit command"

# For multiple unrelated files:
git add file1.rs file2.md
git commit -m "chore: update multiple workspace files"
```
