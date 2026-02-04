---
description: Core Rust development policies for the aiassisted project. Covers zero warnings, static dispatch, minimal Arc usage, and comprehensive testing requirements.
globs: "**/*.rs"
---

# Rust Policy Guide

This guide defines the core development policies for Rust code in the aiassisted project. These policies ensure code quality, maintainability, and consistency.

## 1. Zero Warning Policy

All code must compile with zero warnings.

### Verification

```bash
cargo check 2>&1 | grep -c warning  # Must be 0
```

### Guidelines

- Treat warnings as errors during development
- Fix all warnings before committing
- Do not suppress warnings with `#[allow(...)]` unless absolutely necessary
- If suppression is required, document the reason

### Common Warnings to Address

| Warning | Fix |
|---------|-----|
| Unused variable | Prefix with `_` or remove |
| Unused import | Remove the import |
| Dead code | Remove or add `#[allow(dead_code)]` with justification |
| Deprecated item | Update to non-deprecated alternative |

## 2. Static Dispatch Over Dynamic Dispatch

Prefer generics over `dyn` traits for better performance and compile-time guarantees.

### Why Static Dispatch?

- **Performance**: No runtime vtable lookup
- **Inlining**: Compiler can inline method calls
- **Type safety**: Errors caught at compile time
- **Monomorphization**: Specialized code for each type

### Guidelines

```rust
// ❌ Avoid dynamic dispatch
fn process(handler: &dyn Handler) { }
fn process(handler: Box<dyn Handler>) { }

// ✅ Prefer static dispatch with generics
fn process<H: Handler>(handler: &H) { }
fn process(handler: impl Handler) { }
```

### When Dynamic Dispatch is Acceptable

- Heterogeneous collections requiring different types
- Plugin systems with runtime-loaded types
- Reducing binary size when generic bloat is a concern
- Breaking dependency cycles

```rust
// Acceptable: Heterogeneous collection
let handlers: Vec<Box<dyn Handler>> = vec![
    Box::new(FileHandler::new()),
    Box::new(NetworkHandler::new()),
];
```

## 3. Minimal Arc Usage

Only use `Arc` when concurrent shared ownership is actually required.

### Context for CLI Tools

For this CLI tool:
- Commands run sequentially
- No shared state between threads
- Owned values or references are sufficient

### Guidelines

```rust
// ❌ Avoid unnecessary Arc
fn execute(fs: Arc<dyn FileSystem>) { }

// ✅ Use owned values or references
fn execute(fs: impl FileSystem) { }
fn execute<F: FileSystem>(fs: &F) { }
```

### When Arc is Appropriate

- Sharing state across actual concurrent threads (e.g., thread pools)
- Async tasks that may run on different threads
- Long-lived shared data in multi-threaded contexts

```rust
// Appropriate: Actual multi-threaded sharing
use std::sync::Arc;
use std::thread;

let shared_config = Arc::new(Config::load()?);
let handles: Vec<_> = (0..4).map(|_| {
    let config = Arc::clone(&shared_config);
    thread::spawn(move || process(&config))
}).collect();
```

### Alternatives to Arc

| Instead of | Use |
|------------|-----|
| `Arc<Config>` | Pass `&Config` reference |
| `Arc<Mutex<State>>` | Pass `&mut State` when single-threaded |
| `Arc<dyn Trait>` | Use generics with ownership or borrowing |

## 4. Comprehensive Testing Policy

All code must have comprehensive unit tests covering both positive and negative scenarios.

### Test Coverage Requirements

Every module must include tests for:

1. **Success paths** (positive tests)
2. **Error handling** (negative tests)
3. **Edge cases** (empty data, large data, Unicode, etc.)
4. **Boundary conditions**

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::{mock, predicate::*};

    #[test]
    fn test_function_success_scenario() {
        // Arrange, Act, Assert
    }

    #[test]
    fn test_function_error_scenario() {
        // Test error handling
    }
}
```

### Naming Convention

Use descriptive test names: `test_<function>_<scenario>`

```rust
#[test]
fn test_parse_config_valid_input() { }

#[test]
fn test_parse_config_empty_file() { }

#[test]
fn test_parse_config_invalid_json() { }
```

### Edge Cases to Cover

| Category | Test Cases |
|----------|------------|
| Empty inputs | Empty strings, empty vectors, empty files |
| Missing resources | Nonexistent files, missing paths |
| Invalid data | Malformed JSON, wrong checksums |
| Network failures | Timeouts, connection errors |
| Large data | Files > 1MB, long strings |
| Unicode/UTF-8 | Non-ASCII content, emoji |
| Permission/IO errors | Read-only, access denied |
| Concurrent operations | Race conditions (if applicable) |

### Mock Dependencies

Use `mockall` for trait-based mocking:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::{mock, predicate::*};

    mock! {
        pub FileSystem {}

        #[async_trait::async_trait]
        impl crate::core::infra::FileSystem for FileSystem {
            async fn read(&self, path: &Path) -> Result<String>;
            async fn write(&self, path: &Path, content: &str) -> Result<()>;
        }
    }

    #[tokio::test]
    async fn test_process_file_success() {
        let mut mock_fs = MockFileSystem::new();
        mock_fs.expect_read()
            .returning(|_| Ok("content".to_string()));

        let result = process_file(&mock_fs, Path::new("/test")).await;
        assert!(result.is_ok());
    }
}
```

### Infrastructure Tests

Use real implementations with temporary files:

```rust
#[tokio::test]
async fn test_write_and_read() {
    let fs = StdFileSystem::new();
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test.txt");

    fs.write(&path, "content").await.unwrap();
    let result = fs.read(&path).await.unwrap();

    assert_eq!(result, "content");
}
```

## 5. Integration Testing

Integration tests verify that modules work correctly together using real implementations.

### Test Location

- Integration tests are in the `tests/` directory at project root
- Each test file is compiled as a separate crate

### HTTP Mocking with wiremock

Use `wiremock` for HTTP server mocking in integration tests:

```rust
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn test_download_workflow() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/file.txt"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"content"))
        .mount(&mock_server)
        .await;

    let url = format!("{}/file.txt", mock_server.uri());
    // Test with real HTTP client against mock server
}
```

### mockall vs wiremock

| Aspect | mockall | wiremock |
|--------|---------|----------|
| Test type | Unit tests | Integration tests |
| What's mocked | Rust traits | HTTP endpoints |
| Real I/O? | No | Yes (localhost) |
| Speed | Very fast | Slower |
| Tests | Business logic | HTTP layer |

### What to Test in Integration

1. **Module integration**: Modules working together
2. **Full workflows**: Complete user scenarios
3. **Real data handling**: Large files, Unicode, nested structures

### Running Tests

```bash
# Run all tests (unit + integration)
cargo test

# Run only integration tests
cargo test --test content_integration

# Run specific test
cargo test --test content_integration test_full_download_workflow
```

## Quick Reference

| Policy | Verification |
|--------|--------------|
| Zero warnings | `cargo check 2>&1 \| grep -c warning` returns 0 |
| Static dispatch | Review for `dyn` usage, prefer generics |
| Minimal Arc | Review for Arc, ensure threading justification |
| Unit tests | Every module has `#[cfg(test)]` with positive/negative tests |
| Integration tests | `tests/` directory covers workflows |

## References

- [rust-dispatch-guide.md](rust-dispatch-guide.md) - Detailed static vs dynamic dispatch guide
- [rust-smart-pointers-guide.md](rust-smart-pointers-guide.md) - When to use Box, Rc, Arc, RefCell
- [rust-code-review-guide.md](rust-code-review-guide.md) - Code review checklist
