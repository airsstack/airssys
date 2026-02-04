---
name: policy-rust
description: Enforces Rust development policies including zero warnings, static dispatch preference, minimal Arc usage, and comprehensive testing requirements. Use when reviewing Rust code, writing new Rust code, or checking compliance with project policies.
metadata:
  author: aiassisted
  version: "1.0"
---

# Rust Policy Enforcement

Enforces core Rust development policies for the aiassisted project. Use this skill to verify code compliance or guide implementation.

## When to use this skill

Use this skill when:
- Writing new Rust code in this project
- Reviewing Rust code for policy compliance
- Checking if code meets project standards
- Fixing policy violations

## Quick Reference

| Policy | Requirement |
|--------|-------------|
| Zero warnings | `cargo check` must produce 0 warnings |
| Static dispatch | Prefer generics over `dyn` traits |
| Minimal Arc | Use Arc only for actual concurrent sharing |
| Testing | Unit tests for all modules (positive + negative) |
| Integration tests | Workflow tests in `tests/` directory |

## Policy Checklist

### 1. Zero Warning Policy

Verify no warnings:

```bash
cargo check 2>&1 | grep -c warning  # Must be 0
```

**Common fixes:**
- Unused variable → prefix with `_` or remove
- Unused import → remove
- Dead code → remove or justify `#[allow(dead_code)]`

### 2. Static Dispatch

Check for dynamic dispatch patterns:

```rust
// ❌ Avoid
fn process(handler: &dyn Handler) { }
fn process(handler: Box<dyn Handler>) { }

// ✅ Prefer
fn process<H: Handler>(handler: &H) { }
fn process(handler: impl Handler) { }
```

**Acceptable exceptions:**
- Heterogeneous collections
- Plugin systems
- Breaking dependency cycles

### 3. Minimal Arc Usage

Review Arc usage - this CLI runs sequentially with no shared state:

```rust
// ❌ Avoid in this codebase
fn execute(fs: Arc<dyn FileSystem>) { }

// ✅ Prefer
fn execute<F: FileSystem>(fs: &F) { }
```

**Arc is acceptable for:**
- Actual multi-threaded sharing
- Async tasks across threads

### 4. Comprehensive Unit Tests

Every module must have:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_success() { }  // Positive test

    #[test]
    fn test_function_error() { }    // Negative test

    #[test]
    fn test_function_edge_case() { } // Edge case
}
```

**Required coverage:**
- Success paths
- Error handling
- Edge cases (empty, large, Unicode)
- Boundary conditions

**Mock dependencies with mockall:**

```rust
use mockall::{mock, predicate::*};

mock! {
    pub FileSystem {}
    impl crate::core::infra::FileSystem for FileSystem {
        async fn read(&self, path: &Path) -> Result<String>;
    }
}
```

### 5. Integration Tests

Located in `tests/` directory, use real implementations:

```rust
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_full_workflow() {
    let mock_server = MockServer::start().await;
    // Test with real HTTP client
}
```

## Review Workflow

1. **Check warnings:**
   ```bash
   cargo check 2>&1 | grep warning
   ```

2. **Search for dyn usage:**
   ```bash
   grep -r "dyn " src/
   ```

3. **Search for Arc usage:**
   ```bash
   grep -r "Arc<" src/
   ```

4. **Verify test coverage:**
   ```bash
   cargo test
   ```

5. **Check for missing tests:**
   Look for modules without `#[cfg(test)]`

## Detailed Reference

For comprehensive policy documentation, see [rust-policy-guide.md](../../guidelines/rust/rust-policy-guide.md).

## Related Guidelines

- [rust-dispatch-guide.md](../../guidelines/rust/rust-dispatch-guide.md) - Static vs dynamic dispatch
- [rust-smart-pointers-guide.md](../../guidelines/rust/rust-smart-pointers-guide.md) - Box, Rc, Arc, RefCell usage
- [rust-code-review-guide.md](../../guidelines/rust/rust-code-review-guide.md) - Code review checklist
