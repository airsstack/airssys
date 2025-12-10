# Contributing to AirsSys

Thank you for your interest in contributing to AirsSys! This document provides guidelines for contributing to the project.

## Getting Started

### Prerequisites

- Rust 2021 edition or later
- Git
- Familiarity with async/await Rust
- Understanding of the AirsStack ecosystem

### Development Setup

```bash
# Clone the repository
git clone https://github.com/airsstack/airssys
cd airssys

# Build all components
cargo build --workspace

# Run tests
cargo test --workspace

# Run examples
cargo run --example actor_basic
```

## Project Structure

```
airssys/
├── airssys-osl/          # OS Layer Framework
├── airssys-osl-macros/   # OSL procedural macros
├── airssys-rt/           # Actor Runtime
├── airssys-wasm/         # WASM Component Framework
├── airssys-wasm-cli/     # WASM CLI tools
├── airssys-wasm-component/ # WASM component macros
├── docs/                 # Unified documentation
├── site-mkdocs/          # MkDocs configuration
└── .github/workflows/    # CI/CD workflows
```

## Development Workflow

### 1. Find or Create an Issue

- Check existing issues on GitHub
- Create a new issue if needed
- Discuss approach before major changes

### 2. Create a Branch

```bash
git checkout -b feature/your-feature-name
```

Branch naming conventions:
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation updates
- `refactor/` - Code refactoring
- `test/` - Test additions/improvements

### 3. Make Changes

Follow the coding standards below.

### 4. Test Your Changes

```bash
# Run all tests
cargo test --workspace

# Run component-specific tests
cargo test --package airssys-osl
cargo test --package airssys-rt

# Run with features
cargo test --features macros

# Check code quality
cargo clippy --workspace --all-targets --all-features
cargo fmt --all -- --check
```

### 5. Commit Your Changes

Follow conventional commits:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `test`: Test changes
- `chore`: Build/tooling changes

Examples:
```
feat(osl): add rate limiting middleware
fix(rt): resolve actor spawn race condition
docs(guides): add integration examples
```

### 6. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a pull request on GitHub.

## Coding Standards

### Rust Style

Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

```rust
// ✅ Good: Clear, documented, idiomatic
/// Reads a file with security checks
///
/// # Arguments
/// * `path` - File path to read
/// * `principal` - Security principal
///
/// # Errors
/// Returns `OSError` if access denied or file not found
pub async fn read_file(
    path: &str,
    principal: &str,
) -> Result<Vec<u8>, OSError> {
    // Implementation
}

// ❌ Bad: Undocumented, unclear
pub async fn rf(p: &str, pr: &str) -> Result<Vec<u8>, OSError> {
    // Implementation
}
```

### Documentation

- Document all public APIs
- Include examples in doc comments
- Explain non-obvious design decisions
- Keep docs up-to-date with code

### Testing

- Write unit tests for new functionality
- Write integration tests for component interactions
- Aim for >80% code coverage
- Include both success and failure cases

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_read_success() {
        // Test successful read
    }

    #[tokio::test]
    async fn test_file_read_access_denied() {
        // Test access denied
    }
}
```

### Error Handling

- Use appropriate error types
- Provide helpful error messages
- Don't leak sensitive information
- Log errors appropriately

```rust
// ✅ Good: Clear error with context
return Err(OSError::AccessDenied {
    resource: path.to_string(),
    principal: context.principal().to_string(),
});

// ❌ Bad: Vague error
return Err(OSError::Unknown);
```

## Component-Specific Guidelines

### OSL Guidelines

- All operations must be security-checked
- Add middleware for cross-cutting concerns
- Executors must be platform-agnostic
- Helper functions must use default security

### RT Guidelines

- Actors must not share mutable state
- Use message passing exclusively
- Implement proper lifecycle hooks
- Add supervision for fault tolerance
- Messages must be Send + Sync

## Documentation Updates

When adding features:

1. Update relevant API documentation
2. Add examples demonstrating usage
3. Update integration guide if needed
4. Update migration guide for breaking changes

### Building Documentation

```bash
# Build MkDocs documentation
cd site-mkdocs
mkdocs serve

# Build API documentation
cargo doc --open --workspace
```

## Pull Request Process

### PR Checklist

- [ ] Code follows project style
- [ ] All tests pass
- [ ] New tests added for new functionality
- [ ] Documentation updated
- [ ] Commit messages follow conventions
- [ ] No merge conflicts
- [ ] CI/CD passes

### Review Process

1. Maintainer reviews PR
2. Feedback provided if needed
3. Make requested changes
4. Re-review after changes
5. Merge when approved

### After Merge

- PR is merged to `main` branch
- Documentation automatically deployed
- Close related issues
- Update any tracking documents

## Code of Conduct

### Our Standards

- Be respectful and inclusive
- Welcome newcomers
- Provide constructive feedback
- Focus on what's best for the community

### Our Responsibilities

Project maintainers are responsible for:
- Clarifying standards of acceptable behavior
- Taking corrective action when needed
- Enforcing the code of conduct

### Reporting Issues

Report unacceptable behavior to: hiraqdev@gmail.com

## Getting Help

- **Questions**: Open a GitHub Discussion
- **Bugs**: Open a GitHub Issue
- **Security**: Email hiraqdev@gmail.com
- **Chat**: Join our community (link coming soon)

## Recognition

Contributors are recognized in:
- CONTRIBUTORS.md file
- Release notes
- Project README

Thank you for contributing to AirsSys!

## License

By contributing to AirsSys, you agree that your contributions will be licensed under both the Apache License 2.0 and MIT License.
