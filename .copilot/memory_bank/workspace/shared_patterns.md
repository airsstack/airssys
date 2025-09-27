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