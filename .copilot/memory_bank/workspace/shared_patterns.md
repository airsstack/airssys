# AirsSys Workspace Shared Patterns

## Core Implementation Patterns

### §2.1 3-Layer Import Organization (MANDATORY)
**ALL Rust files MUST follow this exact pattern:**
```rust
// Layer 1: Standard library imports
use std::collections::HashMap;
use std::time::Duration;

// Layer 2: Third-party crate imports  
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

// Layer 3: Internal module imports
use crate::shared::protocol::core::McpMethod;
use crate::transport::http::config::HttpConfig;
```

### §3.2 chrono DateTime<Utc> Standard (MANDATORY)
**ALL time operations MUST use chrono DateTime<Utc>:**
```rust
// ✅ CORRECT
use chrono::{DateTime, Utc};
let now = Utc::now();

// ❌ FORBIDDEN
use std::time::SystemTime;
use std::time::Instant; // Only for performance measuring, never business logic
```

### §4.3 Module Architecture Patterns (MANDATORY)
**mod.rs files MUST contain ONLY:**
- Module declarations (`pub mod example;`)
- Re-exports (`pub use example::ExampleType;`)
- NO implementation code

```rust
// ✅ CORRECT mod.rs
pub mod config;
pub mod context;
pub mod error;

pub use config::{OSLConfig, SecurityConfig};
pub use context::{SystemContext, ActivityLog};
```

### §5.1 Dependency Management (MANDATORY)
**Workspace dependency priority hierarchy:**
```toml
[workspace.dependencies]
# Layer 1: AirsSys Foundation Crates (MUST be at top)
airssys-osl = { path = "crates/airssys-osl" }
airssys-rt = { path = "crates/airssys-rt" }
airssys-wasm = { path = "crates/airssys-wasm" }

# Layer 2: Core Runtime Dependencies
tokio = { version = "1.47", features = ["full"] }
futures = { version = "0.3" }
chrono = { version = "0.4", features = ["serde"] }

# Layer 3: External Dependencies (by category)
serde = { version = "1.0", features = ["derive"] }
```

## Architecture Patterns

### Error Handling Strategy
- Use `thiserror` for error definitions
- Implement `From` traits for error conversion
- Include contextual information in all errors
- Log errors at appropriate levels with structured logging

### Async/Await Patterns
- Prefer `async/await` over manual Future implementations
- Use `tokio::spawn` for fire-and-forget tasks
- Implement proper cancellation for long-running operations
- Use `select!` for concurrent operations with cancellation

### Security Patterns
- Deny-by-default access control
- Comprehensive activity logging
- Input validation at all boundaries
- Secure defaults for all configurations

### Testing Patterns
- Unit tests for all public functions
- Integration tests for component interactions
- Property-based testing for complex algorithms
- Mock external dependencies appropriately

## Methodology Patterns

### Development Workflow
1. **Standards Check**: Verify workspace standards compliance before coding
2. **Test-First**: Write tests before implementation when possible
3. **Zero Warnings**: Address all compiler/clippy warnings immediately
4. **Documentation**: Update docs concurrent with implementation
5. **Security Review**: Consider security implications in all changes

### Code Review Standards
- All code must follow §2.1-§5.1 mandatory patterns
- Performance implications must be documented
- Security considerations must be addressed
- Test coverage must be maintained or improved

### Release Process
- All tests must pass (`cargo test --workspace`)
- Zero warnings (`cargo clippy --workspace --all-targets --all-features`)
- Documentation must be current
- Security audit must be completed for public releases

## Extended Technical Standards

### §6.1 YAGNI Principles (MANDATORY)
**You Aren't Gonna Need It - Build only what is currently required:**
- Implement features only when explicitly needed
- Avoid speculative generalization and future-proofing
- Remove capabilities() methods and complex abstractions until proven necessary
- Focus on core functionality before adding peripheral features
- Prefer simple, direct solutions over elaborate architectures

### §6.2 Avoid `dyn` Patterns (MANDATORY) 
**Prefer static dispatch and compile-time type safety:**
```rust
// ✅ CORRECT - Use generic constraints instead of dyn
pub trait MyTrait<T: Operation> {
    fn process(&self, operation: T) -> Result<(), MyError>;
}

// ❌ FORBIDDEN - Avoid dyn trait objects
pub fn process(handler: Box<dyn MyTrait>) -> Result<(), MyError>;
```

**Hierarchy of abstraction preference:**
1. **Concrete types first** - Use specific types when possible
2. **Generics with constraints** - Use `impl Trait` or `<T: Trait>` for flexibility
3. **`dyn` only as last resort** - When generics become a nesting problem

### §6.3 Microsoft Rust Guidelines Integration (MANDATORY)
**Follow Microsoft Rust Guidelines for production-quality Rust:**

#### M-DI-HIERARCHY: Dependency Injection Hierarchy
- **Prefer concrete types** over generics when implementations are fixed
- **Prefer generics** over `dyn` traits for flexibility with type safety
- **Use `dyn` only** when generics cause excessive nesting complexity

#### M-AVOID-WRAPPERS: Smart Pointer API Restriction  
- Avoid `Arc<T>`, `Box<T>`, `Rc<T>` in public APIs
- Use `&T`, `&mut T`, or `T` directly in function signatures
- Hide implementation details behind clean APIs

#### M-SIMPLE-ABSTRACTIONS: Prevent Cognitive Nesting
- Avoid `Service<Backend<Store>>` patterns in public APIs
- Limit visible type parameter nesting to 1 level deep
- Service types should not require users to deal with `Foo<Bar<FooBar>>`

#### M-ERRORS-CANONICAL-STRUCTS: Structured Error Handling
- Errors are situation-specific structs with `Backtrace`
- Implement `Debug`, `Display`, and `std::error::Error`
- Provide helper methods for error categorization (e.g., `is_io()`, `is_security()`)

#### M-SERVICES-CLONE: Shared Service Pattern
- Service types implement cheap `Clone` via `Arc<Inner>` pattern
- Enable shared ownership without expensive copying
- Allow services to be passed to multiple consumers easily

#### M-DESIGN-FOR-AI: AI-Optimized Development (CRITICAL)
- **Create idiomatic Rust API patterns** - Follow standard conventions for better AI understanding
- **Provide thorough documentation** - Comprehensive rustdoc for all modules and public items
- **Provide thorough examples** - Directly usable examples in docs and repository
- **Use strong types** - Avoid primitive obsession, use newtype patterns
- **Make APIs testable** - Design for unit testing with mocks/fakes
- **Ensure test coverage** - Observable behavior coverage for hands-off refactoring

#### Additional Core Guidelines
- **M-UNSAFE**: Unsafe needs justification (novel abstractions, performance, FFI only)
- **M-UNSOUND**: All code must be sound - no exceptions ever
- **M-PANIC-IS-STOP**: Panics mean program termination, not error handling
- **M-ESSENTIAL-FN-INHERENT**: Core functionality in inherent methods, not just traits
- **M-IMPL-ASREF**: Accept `impl AsRef<T>` for flexible function parameters  
- **M-MOCKABLE-SYSCALLS**: I/O and system calls must be mockable for testing
- **M-SMALLER-CRATES**: Err on side of too many crates rather than too few

**Reference**: [Microsoft Rust Guidelines](https://microsoft.github.io/rust-guidelines/)  
**AI Guidelines**: [Complete Reference](https://microsoft.github.io/rust-guidelines/agents/all.txt)

### §6.4 Implementation Quality Gates (MANDATORY)
**All implementations must meet these criteria:**
- **Safety First**: No `unsafe` blocks without thorough justification
- **Zero Warnings**: All code must compile cleanly with clippy
- **Comprehensive Tests**: >90% code coverage with unit and integration tests
- **Security Logging**: All operations must generate audit trails
- **Resource Management**: Proper cleanup and lifecycle management