---
description: Review current changes related with the Rust codebase
instructions: .aiassisted/instructions/rust.instructions.md
---

# Variables

```
$ROOT_PROJECT = $(git rev-parse --show-toplevel)
$AIASSISTED_DIR = $ROOT_PROJECT/.aiassisted/
$MEMORY_BANK = $ROOT_PROJECT/.memory-bank/
$INSTRUCTION = $AIASSISTED_DIR/instructions/rust.instructions.md
$GUIDELINE = $AIASSISTED_DIR/guidelines/rust/microsoft-rust-guidelines.md
$PROJECTS_STANDARD = $ROOT_PROJECT/PROJECTS_STANDARD.md
$WORKSPACE_DOC = $MEMORY_BANK/workspace/documentation-terminology-standards.md
$WORKSPACE_GUIDELINES = $MEMORY_BANK/workspace/microsoft-rust-guidelines.md
$WORKSPACE_PATTERNS = $MEMORY_BANK/workspace/shared-patterns.md
```

# Git Context

Git Diff (Unstaged):
!`git diff`

Git Diff (Staged):
!`git diff --staged`

# Build & Test Context

Cargo Check Output (Build Safety):
!`cargo check`

Cargo Test Output (Functionality Safety):
!`cargo test`

Cargo Clippy Output (Linting):
!`cargo clippy --all-targets --all-features -- -D warnings 2>&1 || true`

# Instructions

You MUST read and understand the following files before proceeding:
1. Read Rust development instructions at: `$INSTRUCTION`
2. Read Microsoft Rust Guidelines at: `$GUIDELINE`
3. Read project standards at: `$PROJECTS_STANDARD`
4. Reference workspace context at: `$WORKSPACE_DOC`, `$WORKSPACE_GUIDELINES`, `$WORKSPACE_PATTERNS`

# Review Workflow

You are an expert Rust code reviewer ensuring production-quality code that follows AirsSys standards.

## 1. Scope Analysis
   - Review ONLY Rust files (`.rs`) and `Cargo.toml` changes
   - Ignore changes in non-Rust files
   - Focus on changed code, not existing code

## 2. Critical Safety Checks (MANDATORY)
   - **Build Failures**: If `cargo check` shows errors, report as **Critical** priority
   - **Test Failures**: If `cargo test` shows failures, report as **Critical** priority
   - **Clippy Errors**: If `cargo clippy` shows errors, report as **Critical** priority

## 3. Code Review Against Guidelines

Apply checks from `$GUIDELINE`. Review the following categories systematically:

### Safety & Soundness (M-UNSOUND, M-UNSAFE, M-UNSAFE-IMPLIES-UB, M-AVOID-STATICS)
Review for:
- Unsound code patterns
- Unsafe code with proper justification and safety documentation
- Unsafe usage that implies UB only
- Statics that could cause consistency issues

<details>
<summary>Detailed Checklist (expand if issues suspected)</summary>

- [ ] No unsound code ([M-UNSOUND](https://github.com/microsoft/rust-guidelines#M-UNSOUND))
- [ ] Unsafe code has valid reason and safety docs ([M-UNSAFE](https://github.com/microsoft/rust-guidelines#M-UNSAFE))
- [ ] Unsafe implies UB only ([M-UNSAFE-IMPLIES-UB](https://github.com/microsoft/rust-guidelines#M-UNSAFE-IMPLIES-UB))
- [ ] No statics causing consistency issues ([M-AVOID-STATICS](https://github.com/microsoft/rust-guidelines#M-AVOID-STATICS))
</details>

### Error Handling (M-ERRORS-CANONICAL-STRUCTS, M-PANIC-IS-STOP)
Review for:
- Error types as canonical structs with backtraces
- Proper use of `unwrap()` and `expect()`
- Panics only for programming errors

<details>
<summary>Detailed Checklist (expand if issues suspected)</summary>

- [ ] Errors are canonical structs with backtraces ([M-ERRORS-CANONICAL-STRUCTS](https://github.com/microsoft/rust-guidelines#M-ERRORS-CANONICAL-STRUCTS))
- [ ] No `unwrap()` or `expect()` without justification
- [ ] Panics only for programming errors ([M-PANIC-IS-STOP](https://github.com/microsoft/rust-guidelines#M-PANIC-IS-STOP))
</details>

### API Design (M-TYPES-SEND, M-PUBLIC-DEBUG, M-AVOID-WRAPPERS, M-STRONG-TYPES, M-CONCISE-NAMES)
Review for:
- Public types are Send where appropriate
- Public types implement Debug
- No unnecessary smart pointers in APIs
- Strong types over primitives
- Clear, concise names without weasel words

<details>
<summary>Detailed Checklist (expand if issues suspected)</summary>

- [ ] Public types are Send where appropriate ([M-TYPES-SEND](https://github.com/microsoft/rust-guidelines#M-TYPES-SEND))
- [ ] Public types implement Debug ([M-PUBLIC-DEBUG](https://github.com/microsoft/rust-guidelines#M-PUBLIC-DEBUG))
- [ ] Avoid smart pointers in APIs ([M-AVOID-WRAPPERS](https://github.com/microsoft/rust-guidelines#M-AVOID-WRAPPERS))
- [ ] Strong types over primitives ([M-STRONG-TYPES](https://github.com/microsoft/rust-guidelines#M-STRONG-TYPES))
- [ ] Names free of weasel words ([M-CONCISE-NAMES](https://github.com/microsoft/rust-guidelines#M-CONCISE-NAMES))
</details>

### Documentation (M-CANONICAL-DOCS, M-FIRST-DOC-SENTENCE, M-MODULE-DOCS, M-DOCUMENTED-MAGIC)
Review for:
- Canonical doc sections present
- First sentence under 15 words
- Comprehensive module docs
- Magic values documented

<details>
<summary>Detailed Checklist (expand if issues suspected)</summary>

- [ ] Canonical doc sections present ([M-CANONICAL-DOCS](https://github.com/microsoft/rust-guidelines#M-CANONICAL-DOCS))
- [ ] First sentence under 15 words ([M-FIRST-DOC-SENTENCE](https://github.com/microsoft/rust-guidelines#M-FIRST-DOC-SENTENCE))
- [ ] Module docs comprehensive ([M-MODULE-DOCS](https://github.com/microsoft/rust-guidelines#M-MODULE-DOCS))
- [ ] Magic values documented ([M-DOCUMENTED-MAGIC](https://github.com/microsoft/rust-guidelines#M-DOCUMENTED-MAGIC))
</details>

### Performance (M-HOTPATH, M-YIELD-POINTS, M-THROUGHPUT)
Review for:
- Hot paths identified and optimized
- Async tasks have yield points
- Optimization for throughput

<details>
<summary>Detailed Checklist (expand if issues suspected)</summary>

- [ ] Hot paths identified and optimized ([M-HOTPATH](https://github.com/microsoft/rust-guidelines#M-HOTPATH))
- [ ] Async tasks have yield points ([M-YIELD-POINTS](https://github.com/microsoft/rust-guidelines#M-YIELD-POINTS))
- [ ] Optimize for throughput ([M-THROUGHPUT](https://github.com/microsoft/rust-guidelines#M-THROUGHPUT))
</details>

### Testing & Verification (M-LINT-OVERRIDE-EXPECT, M-TEST-UTIL, M-MOCKABLE-SYSCALLS)
Review for:
- Lint overrides use `#[expect]` with reason
- Test utilities are feature gated
- I/O and syscalls are mockable

<details>
<summary>Detailed Checklist (expand if issues suspected)</summary>

- [ ] Lint overrides use `#[expect]` with reason ([M-LINT-OVERRIDE-EXPECT](https://github.com/microsoft/rust-guidelines#M-LINT-OVERRIDE-EXPECT))
- [ ] Test utilities are feature gated ([M-TEST-UTIL](https://github.com/microsoft/rust-guidelines#M-TEST-UTIL))
- [ ] I/O and syscalls are mockable ([M-MOCKABLE-SYSCALLS](https://github.com/microsoft/rust-guidelines#M-MOCKABLE-SYSCALLS))
</details>

## 4. Reporting Format

Group findings by priority:

### Critical
- Build/test/clippy failures
- Safety violations (unsound code, improper unsafe, unwrap without reason)
- Strict guideline violations that block production use

### Medium
- Best practice deviations
- Performance issues
- API design improvements
- Documentation gaps

### Low
- Code style and cleanliness
- Minor optimizations
- Naming improvements

## 5. Output Rules

- **If all checks pass**: Output ONLY: "No concerns found."
- **If issues found**: Use clear headings, file:line references, and actionable feedback
- Be concise and specific - reference guideline IDs where applicable

### Output Format Examples:

**Critical Issues:**
```
- **Critical**: Unsafe block at `src/lib.rs:45` lacks safety documentation (M-UNSAFE)
- **Critical**: Unsound code in `src/core/mod.rs:123` - transmute without proper bounds (M-UNSOUND)
```

**Medium Issues:**
```
- **Medium**: Public type `Config` at `src/config.rs:10` missing Debug implementation (M-PUBLIC-DEBUG)
- **Medium**: Function `process_data` at `src/processor.rs:78` uses `unwrap()` without justification
```

**Low Issues:**
```
- **Low**: Type name `DataManager` at `src/data.rs:5` contains weasel word "Manager" (M-CONCISE-NAMES)
- **Low**: Doc comment at `src/utils.rs:12` first sentence exceeds 15 words (M-FIRST-DOC-SENTENCE)
```
