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

### §2.2 No Fully Qualified Names in Type Annotations (MANDATORY)
**ALL type annotations MUST use imported types, NOT fully qualified names (FQN):**

This policy enforces consistency with §2.1 (3-Layer Import Organization). Types MUST be imported at the top of the file and referenced by their simple name throughout the code.

**✅ CORRECT - Import types, use simple names:**
```rust
use std::path::PathBuf;
use std::fs::File;
use std::collections::HashMap;

struct Config {
    path: PathBuf,           // ✅ Uses imported type
    files: Vec<File>,        // ✅ Uses imported type
    cache: HashMap<String, u32>,  // ✅ Uses imported type
}

fn process(file: File) -> Result<PathBuf, Error> {  // ✅ Uses imported types
    // implementation
}
```

**❌ FORBIDDEN - Fully qualified names in type annotations:**
```rust
struct Config {
    path: std::path::PathBuf,    // ❌ FQN - must be imported
    files: Vec<std::fs::File>,   // ❌ FQN - must be imported
    cache: std::collections::HashMap<String, u32>,  // ❌ FQN - must be imported
}

fn process(file: std::fs::File) -> Result<std::path::PathBuf, Error> {  // ❌ FQN
    // implementation
}
```

**❌ FORBIDDEN - Mixed imports and FQN:**
```rust
use std::path::PathBuf;

struct Config {
    path: PathBuf,                         // ✅ Uses imported type
    files: Vec<std::fs::File>,            // ❌ Inconsistent - FQN
    cache: std::collections::HashMap<...>,  // ❌ Inconsistent - FQN
}
```

**Rationale:**
- **Readability**: Simple type names are easier to read than long FQNs
- **Consistency**: Follows §2.1's requirement for organized imports
- **Maintainability**: All type dependencies are visible at file top
- **Clarity**: Clear separation between import section and implementation

**Exceptions (RARE):**
- Type alias imports that would create name conflicts AND no suitable renaming available
- Foreign function interface (FFI) types where FQN is standard practice
- **Note**: These exceptions MUST be justified with code comments

**Verification:**
```bash
# Check for FQN usage in struct fields, function signatures, type aliases
grep -rnE "struct\s+\w+\s*\{[^}]*std::::" src/**/*.rs
grep -rnE "fn\s+\w+\([^)]*:std::::" src/**/*.rs
grep -rnE "->\s*Result<std::::|->\s*std::::" src/**/*.rs
```
**Expected:** No FQN usage in type annotations found.

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
airssys-osl = { path = "airssys-osl" }
airssys-rt = { path = "airssys-rt" }
airssys-wasm = { path = "airssys-wasm" }

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

### §6.4 Implementation Quality Gates (MANDATORY)
**All implementations must meet these criteria:**
- **Safety First**: No `unsafe` blocks without thorough justification
- **Zero Warnings**: All code must compile cleanly with clippy
- **Comprehensive Tests**: >90% code coverage with unit and integration tests
- **Security Logging**: All operations must generate audit trails
- **Resource Management**: Proper cleanup and lifecycle management
