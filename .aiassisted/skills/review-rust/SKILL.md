---
name: review-rust
description: Review Rust code changes for safety, idioms, and patterns. Use when reviewing Rust code, checking ownership and lifetimes, unsafe blocks, error handling, or Rust-specific patterns like builders and typestate.
license: MIT
compatibility: opencode
metadata:
  language: rust
  focus: code-review
---

# Rust Code Review Skill

Review Rust code changes following the project's Rust Code Review Guide.

## When to Use

Use this skill when:
- Reviewing Rust code changes or pull requests
- Checking Rust-specific patterns (ownership, lifetimes, traits)
- Reviewing `unsafe` code blocks
- Evaluating Rust API design and idioms

## Prerequisites

This skill extends the general code review process. For general review principles, see:
- `.aiassisted/guidelines/engineering/code-review-process.md`

## Instructions

### Step 1: Load the Guidelines

Read the Rust-specific review guide:
- **Primary**: `.aiassisted/guidelines/rust/rust-code-review-guide.md`

### Step 2: Verify Compilation

All code must compile with zero warnings:

```bash
cargo check 2>&1 | grep -c warning  # Must be 0
cargo clippy 2>&1 | grep -c warning  # Must be 0
```

Check for lint overrides - any `#[allow(...)]` must have documented reason. Prefer `#[expect(...)]` with `reason`.

### Step 3: Review Checklist

#### Safety and Soundness

- [ ] Is `unsafe` actually necessary? (FFI, performance with benchmarks, novel abstractions)
- [ ] Safety invariants documented in `// SAFETY:` comments
- [ ] No undefined behavior risks
- [ ] `Send`/`Sync` implementations are correct

#### Error Handling

- [ ] Libraries use canonical error structs (not `anyhow`/`eyre`)
- [ ] Errors implement `std::error::Error`, `Debug`, `Display`
- [ ] Panics only for programming errors, not recoverable errors
- [ ] `unwrap()`/`expect()` have clear justification

#### Ownership and Lifetimes

- [ ] Ownership transfers are intentional
- [ ] No unnecessary cloning
- [ ] References used where ownership not needed
- [ ] Lifetimes as simple as possible

#### API Design

- [ ] Public types implement common traits (`Debug`, `Clone`, `PartialEq`, etc.)
- [ ] Sensitive types have redacted `Debug`
- [ ] Follow `as_`/`to_`/`into_` conventions
- [ ] Getters have no `get_` prefix
- [ ] No smart pointers in public APIs unless fundamental

#### Performance

- [ ] No unnecessary allocations in hot paths
- [ ] `String` vs `&str`, `Vec<T>` vs `&[T]` used appropriately
- [ ] `Arc` only when concurrent shared ownership required

#### Testing

- [ ] Unit tests for success and error paths
- [ ] Edge cases covered (empty, boundaries, Unicode)
- [ ] Test names: `test_<function>_<scenario>`

#### Documentation

- [ ] All public items have doc comments
- [ ] `# Errors` section for `Result` functions
- [ ] `# Panics` section if function can panic
- [ ] `# Safety` section for `unsafe` functions

### Step 4: Load Pattern-Specific Guidelines

Based on code patterns, load additional guidelines from `.aiassisted/guidelines/rust/`:

| Code Pattern | Guideline File |
|--------------|----------------|
| Builder pattern | `rust-builder-pattern-guide.md` |
| Trait objects, generics | `rust-dispatch-guide.md` |
| `Box`, `Rc`, `Arc`, `RefCell` | `rust-smart-pointers-guide.md` |
| Enums with data, sum types | `rust-adt-implementation-guide.md` |
| Factory/creation patterns | `rust-factory-pattern-guide.md` |
| Typestate machines | `rust-typestate-pattern-guide.md` |
| `Cargo.toml` changes | `rust-dependency-management-guide.md` |
| Module reorganization | `rust-main-lib-crate-structure-guide.md` |

### Step 5: Provide Feedback

Format your review:

#### Strengths
- What follows guidelines well
- Good patterns and idioms

#### Issues
- File and line number
- Guideline violated (cite specific file/section)
- Recommended fix

#### Suggestions
- Optional improvements (use `Nit:` or `Optional:` labels)

### Example Comments

```
Nit: Consider using `&str` instead of `String` here since the function
doesn't need ownership.

---

This `unwrap()` could panic if the config file is missing. Consider
returning a `Result` or using `expect()` with a descriptive message.

---

The `Arc` here adds unnecessary overhead since this code runs sequentially.
A reference would suffice. See rust-smart-pointers-guide.md for guidance.

---

FYI: The builder pattern guide recommends consuming builders for this
use case. Not blocking, but worth considering for consistency.
```

## Core Principle

> **Approve a change once it definitely improves the overall code health of the Rust codebase, even if it is not perfect.**

Apply general code review principles with additional attention to Rust's unique characteristics: ownership, lifetimes, safety, and idiomatic patterns.

## References

- **Primary**: `.aiassisted/guidelines/rust/rust-code-review-guide.md`
- **Prerequisite**: `.aiassisted/guidelines/engineering/code-review-process.md`
- **Pattern guides**: `.aiassisted/guidelines/rust/*.md`

Always load and follow the guidelines as the authoritative source.
