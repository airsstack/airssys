# Rust Code Review Guide

**Document Type:** Engineering Standard
**Status:** Active
**Scope:** All Rust Code Reviews
**Prerequisites:** [Code Review Process Guidelines](../engineering/code-review-process.md)

## Purpose

This guide provides Rust-specific code review criteria that complement the general [Code Review Process Guidelines](../engineering/code-review-process.md). All reviews must follow the general process; this document adds language-specific considerations.

---

## Core Principle

> **Approve a change once it definitely improves the overall code health of the Rust codebase, even if it is not perfect.**

Apply the same decision-making principles from the general code review process, with additional attention to Rust's unique characteristics: ownership, lifetimes, safety, and idiomatic patterns.

---

## Rust-Specific Review Checklist

### 1. Compilation and Warnings

**Zero Warning Policy**

All code must compile with zero warnings. Verify:

```bash
cargo check 2>&1 | grep -c warning  # Must be 0
cargo clippy 2>&1 | grep -c warning  # Must be 0
```

- [ ] Code compiles without warnings
- [ ] Clippy lints pass without warnings
- [ ] No `#[allow(...)]` without documented reason (prefer `#[expect(...)]` with `reason`)

**Lint Override Review**

When reviewing lint overrides:

```rust
// Good: Expected lint with reason
#[expect(clippy::unused_async, reason = "API fixed, will use I/O later")]
pub async fn ping_server() { }

// Requires justification: Why is this allowed?
#[allow(dead_code)]
fn unused_helper() { }
```

---

### 2. Safety and Soundness

**Unsafe Code Review**

Unsafe code requires elevated scrutiny. Verify:

- [ ] Is `unsafe` actually necessary? (Valid reasons: FFI, performance with benchmarks, novel abstractions)
- [ ] Safety invariants documented in `// SAFETY:` comments
- [ ] Code passes Miri if applicable
- [ ] No undefined behavior risks

```rust
// Good: Documented safety invariants
// SAFETY: `ptr` is guaranteed non-null and properly aligned by `allocate()`.
// The memory region is exclusively owned by this function.
unsafe { *ptr = value; }

// Bad: No safety documentation
unsafe { *ptr = value; }
```

**Soundness Verification**

- [ ] No safe functions that can cause undefined behavior when called from safe code
- [ ] `Send`/`Sync` implementations are correct (no manual `unsafe impl` without justification)
- [ ] No memory leaks in ownership transfers

---

### 3. Error Handling

**Error Types**

- [ ] Libraries use canonical error structs with `Backtrace` (not `anyhow`/`eyre`)
- [ ] Applications may use `anyhow`/`eyre` for application-level errors
- [ ] Errors implement `std::error::Error`, `Debug`, and `Display`
- [ ] Error messages are actionable and contextual

```rust
// Good: Canonical error struct for libraries
pub struct ConfigError {
    kind: ConfigErrorKind,
    backtrace: Backtrace,
}

impl ConfigError {
    pub fn is_not_found(&self) -> bool { ... }
    pub fn config_path(&self) -> &Path { ... }
}

// Good: anyhow for applications
fn main() -> anyhow::Result<()> { ... }
```

**Panic Policy**

- [ ] Panics only for programming errors (contract violations), not recoverable errors
- [ ] No panics in library code for conditions callers could handle
- [ ] `unwrap()` and `expect()` have clear justification or use in tests only

```rust
// Good: Panic on programming error with message
let config = config.expect("config must be initialized before use");

// Bad: Panic on recoverable error
let file = File::open(path).unwrap(); // Should return Result
```

---

### 4. Ownership and Lifetimes

**Ownership Review**

- [ ] Ownership transfers are intentional and documented if non-obvious
- [ ] No unnecessary cloning (check for `clone()` that could be avoided)
- [ ] References used where ownership transfer is not needed

**Lifetime Review**

- [ ] Lifetimes are as simple as possible (avoid lifetime proliferation)
- [ ] No unnecessary `'static` bounds
- [ ] Lifetime elision used where possible

```rust
// Good: Simple lifetime
fn process(data: &str) -> &str { ... }

// Requires justification: Complex lifetimes
fn process<'a, 'b, 'c>(x: &'a str, y: &'b str) -> &'c str
where 'a: 'c, 'b: 'c { ... }
```

---

### 5. API Design

**Type Design**

- [ ] Public types implement common traits: `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`, `Default` (where appropriate)
- [ ] Sensitive types have custom `Debug` that redacts secrets
- [ ] `Display` implemented for user-facing types
- [ ] Types are `Send` (especially futures and async-related types)

```rust
// Good: Common trait implementations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct UserId(String);

// Good: Redacted Debug for sensitive data
impl Debug for ApiKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ApiKey(***)")
    }
}
```

**Function Signatures**

- [ ] Follow `as_`, `to_`, `into_` conventions for conversions
- [ ] Getters follow Rust convention (no `get_` prefix)
- [ ] Constructors are `new()` or `default()`
- [ ] No smart pointers in public APIs unless fundamental to purpose

```rust
// Good: Idiomatic API
impl Config {
    pub fn new() -> Self { ... }
    pub fn name(&self) -> &str { ... }        // Not get_name()
    pub fn into_inner(self) -> Inner { ... }  // Consumes self
    pub fn as_bytes(&self) -> &[u8] { ... }   // Borrows
    pub fn to_string(&self) -> String { ... } // Allocates new
}

// Bad: Smart pointers in public API
pub fn process(data: Arc<Mutex<Data>>) -> Box<Result> { ... }
```

---

### 6. Performance Considerations

**Allocation Review**

- [ ] No unnecessary allocations in hot paths
- [ ] `String` vs `&str`, `Vec<T>` vs `&[T]` used appropriately
- [ ] Consider `Cow<'_, T>` for conditionally owned data

**Concurrency Review**

- [ ] No contended locks in hot paths
- [ ] `Arc` only when concurrent shared ownership is required
- [ ] Prefer owned values or references for sequential code

```rust
// Good: No Arc needed for sequential code
fn process(config: &Config) -> Result { ... }

// Requires justification: Arc in non-concurrent context
fn process(config: Arc<Config>) -> Result { ... }
```

---

### 7. Testing

**Test Coverage**

- [ ] Unit tests for success and error paths
- [ ] Edge cases covered (empty inputs, boundaries, Unicode)
- [ ] Tests use mocks for external dependencies (via `mockall` or similar)

**Test Quality**

- [ ] Test names describe scenario: `test_<function>_<scenario>`
- [ ] Tests verify behavior, not implementation details
- [ ] No `unwrap()` in tests without clear reason (prefer `?` with `Result` return)

```rust
#[tokio::test]
async fn test_download_file_checksum_mismatch() {
    let mut mock_http = MockHttpClient::new();
    mock_http.expect_get()
        .returning(|_| Ok("content".to_string()));

    let result = download_file(&mock_http, &entry).await;

    assert!(result.is_err());
    assert!(matches!(result, Err(Error::ChecksumMismatch { .. })));
}
```

---

### 8. Documentation

**Required Documentation**

- [ ] All public items have doc comments
- [ ] Summary sentence is under 15 words
- [ ] `# Examples` section for non-trivial APIs
- [ ] `# Errors` section for functions returning `Result`
- [ ] `# Panics` section if function can panic
- [ ] `# Safety` section for `unsafe` functions

```rust
/// Loads configuration from the specified path.
///
/// # Errors
///
/// Returns `ConfigError::NotFound` if the file does not exist.
/// Returns `ConfigError::Parse` if the file contains invalid TOML.
///
/// # Examples
///
/// ```
/// let config = load_config("config.toml")?;
/// ```
pub fn load_config(path: &Path) -> Result<Config, ConfigError> { ... }
```

**Module Documentation**

- [ ] Modules have `//!` documentation explaining purpose
- [ ] Complex modules explain when to use them

---

### 9. Code Organization

**Crate Structure**

- [ ] Logical module organization
- [ ] No circular dependencies between modules
- [ ] `pub(crate)` used appropriately to limit visibility
- [ ] No glob re-exports (`pub use foo::*`) except for HAL patterns

**Feature Flags**

- [ ] Features are additive (no `no-std` feature; use `std` feature instead)
- [ ] Test utilities behind `test-util` feature
- [ ] No mutually exclusive features

---

### 10. Idiomatic Patterns

**Pattern Verification**

When reviewing code that uses specific patterns, verify correct implementation:

| Pattern | Check For |
|---------|-----------|
| Builder | Consuming `build()`, validation, ergonomic API |
| Typestate | Compile-time state transitions, zero runtime cost |
| Newtype | Proper `Deref` if appropriate, no leaky abstraction |
| Error handling | `?` operator, proper error propagation |

---

## Conditional Reference Guidelines

The following guidelines provide detailed guidance for specific patterns and concepts. **Reference them only when the code under review uses these patterns.**

### When Reviewing Builder Patterns

If the code implements or modifies a builder pattern:
- **Reference:** [Rust Builder Pattern Guide](rust-builder-pattern-guide.md)
- Verify: Consuming vs non-consuming builders, validation timing, ergonomics

### When Reviewing Dispatch Mechanisms

If the code involves trait objects, generics, or dispatch decisions:
- **Reference:** [Rust Dispatch Guide](rust-dispatch-guide.md)
- Verify: Static dispatch preferred, `dyn` justified, no unnecessary boxing

### When Reviewing Smart Pointer Usage

If the code uses `Box`, `Rc`, `Arc`, `RefCell`, or similar:
- **Reference:** [Rust Smart Pointers Guide](rust-smart-pointers-guide.md)
- Verify: Pointer type appropriate, no `Arc` without concurrency needs

### When Reviewing Algebraic Data Types

If the code defines enums with data or uses sum types:
- **Reference:** [Rust ADT Implementation Guide](rust-adt-implementation-guide.md)
- Verify: Exhaustive matching, `#[non_exhaustive]` for public enums if appropriate

### When Reviewing Factory Patterns

If the code implements factory or creation patterns:
- **Reference:** [Rust Factory Pattern Guide](rust-factory-pattern-guide.md)
- Verify: Prefer `Fn() -> T` over factory traits, builder pattern where appropriate

### When Reviewing Typestate Patterns

If the code uses typestate for compile-time state machines:
- **Reference:** [Rust Typestate Pattern Guide](rust-typestate-pattern-guide.md)
- Verify: Zero runtime cost, impossible states unrepresentable

### When Reviewing Dependency Management

If the code modifies `Cargo.toml` or introduces new dependencies:
- **Reference:** [Rust Dependency Management Guide](rust-dependency-management-guide.md)
- Verify: Minimal dependencies, version constraints appropriate, no duplicate deps

### When Reviewing Crate Structure

If the code reorganizes modules or crate boundaries:
- **Reference:** [Rust Main/Lib Crate Structure Guide](rust-main-lib-crate-structure-guide.md)
- Verify: Clear separation of concerns, appropriate visibility

---

## Review Comment Examples

### Good Comments

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

### Avoid

```
// Too vague
This doesn't look right.

// Prescriptive without explanation
Use Arc instead of Rc.

// Missing context
This is wrong.
```

---

## Summary Checklist

### For Every Rust Review

- [ ] Zero warnings (cargo check, clippy)
- [ ] No unjustified `unsafe`
- [ ] Proper error handling (no inappropriate panics)
- [ ] Common traits implemented for public types
- [ ] Documentation for public APIs
- [ ] Tests for success and error paths

### When Applicable

- [ ] Ownership/lifetime complexity justified
- [ ] Performance-sensitive code avoids unnecessary allocations
- [ ] Smart pointers used appropriately
- [ ] Patterns implemented correctly (reference guides as needed)
- [ ] Dependencies minimal and appropriate

---

## References

### Required Reading
- [Code Review Process Guidelines](../engineering/code-review-process.md)
- [Microsoft Rust Guidelines](microsoft-rust-guidelines.md)

### Conditional References (Use When Relevant)
- [Rust Builder Pattern Guide](rust-builder-pattern-guide.md)
- [Rust Dispatch Guide](rust-dispatch-guide.md)
- [Rust Smart Pointers Guide](rust-smart-pointers-guide.md)
- [Rust ADT Implementation Guide](rust-adt-implementation-guide.md)
- [Rust Factory Pattern Guide](rust-factory-pattern-guide.md)
- [Rust Typestate Pattern Guide](rust-typestate-pattern-guide.md)
- [Rust Dependency Management Guide](rust-dependency-management-guide.md)
- [Rust Main/Lib Crate Structure Guide](rust-main-lib-crate-structure-guide.md)

### External References
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/checklist.html)
- [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/)
- [Rust Design Patterns](https://rust-unofficial.github.io/patterns/intro.html)
