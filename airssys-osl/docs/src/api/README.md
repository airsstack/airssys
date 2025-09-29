# API Reference

This section provides API documentation based on the documented architecture and implementation specifications.

*Note: AirsSys OSL is currently in foundation setup phase. The APIs shown represent the documented planned interfaces.*

## Core Modules

Based on the documented core module structure:

- **[Core Types](./core-types.md)**: Essential trait abstractions and types (`src/core/`)
- **[Filesystem Operations](./filesystem.md)**: File system operations (planned)
- **[Process Management](./process.md)**: Process spawning and management (planned)
- **[Security Framework](./security.md)**: Consolidated security middleware (`middleware/security/`)
- **[Activity Logging](./logging.md)**: Activity logging subsystem (`middleware/logger/`)

## Documented API Patterns

### Generic-First Design Pattern
Following Microsoft Rust Guidelines M-DI-HIERARCHY:

```rust
// ✅ DOCUMENTED: Generic constraints pattern
pub trait OSExecutor<O>: Debug + Send + Sync + 'static 
where O: Operation
{
    async fn execute(&self, operation: O, context: &ExecutionContext) -> OSResult<ExecutionResult>;
}

// ❌ AVOIDED: dyn patterns
// pub trait OSExecutor {
//     async fn execute(&self, operation: &dyn Operation) -> OSResult<ExecutionResult>;
// }
```

### Error Handling
Following Microsoft Guidelines M-ERRORS-CANONICAL-STRUCTS:

```rust
#[derive(Error, Debug)]
pub enum OSError {
    #[error("Security policy violation: {reason}")]
    SecurityViolation { reason: String },
    
    #[error("Middleware failed: {middleware}: {reason}")]
    MiddlewareFailed { middleware: String, reason: String },
    
    #[error("Filesystem operation failed: {operation} on {path}: {source}")]
    FilesystemError {
        operation: String,
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
}

// Contextual helper methods
impl OSError {
    pub fn is_security_violation(&self) -> bool;
    pub fn is_filesystem_error(&self) -> bool;
}
```

### Async Operations
Based on tech context documentation:

```rust
use tokio; // Primary async runtime
use chrono::{DateTime, Utc}; // Workspace standard §3.2

#[tokio::main]
async fn main() -> OSResult<()> {
    // Async-first design pattern
    // Implementation pending completion of core foundation
    Ok(())
}
```

### Security Context Integration
Based on documented security-consolidated architecture:

```rust
// Security handled within middleware/security/
// No separate SecurityPolicy trait - integrated into SecurityMiddleware
// Security middleware processes all operations before execution
```

## Implementation Status

**Current Phase**: Core module foundation (OSL-TASK-001) implementation  
**Priority**: Essential trait abstractions in `src/core/`  
**Next**: Security and logging middleware implementation

For detailed API specifications, see the individual module sections. APIs will be updated as implementation progresses following the documented architecture.