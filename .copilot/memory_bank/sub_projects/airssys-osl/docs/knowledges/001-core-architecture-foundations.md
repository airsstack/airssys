# Core Architecture Knowledge - airssys-osl

## Overview
This document captures the core architectural decisions and patterns for the airssys-osl implementation based on the analysis of requirements and technical standards.

## Core Architecture Principles

### 1. Generic-First Design Pattern
Following Microsoft Rust Guidelines M-DI-HIERARCHY and workspace standard §6.2:

**Hierarchy of Abstraction:**
1. **Concrete types** - Use specific implementations when behavior is fixed
2. **Generic constraints** - Use `impl Trait` or `<T: Trait>` for flexibility
3. **`dyn` traits** - Only when generics cause excessive nesting (last resort)

```rust
// ✅ PREFERRED: Generic constraints with clear bounds
pub trait OSExecutor<O>: Debug + Send + Sync + 'static 
where 
    O: Operation,
{
    async fn execute(&self, operation: O, context: &ExecutionContext) -> OSResult<ExecutionResult>;
}

// ❌ AVOID: dyn patterns
pub trait OSExecutor {
    async fn execute(&self, operation: &dyn Operation, context: &ExecutionContext) -> OSResult<ExecutionResult>;
}
```

### 2. Core-First Module Architecture
Following workspace standard §4.3 and YAGNI principles (§6.1):

**Module Hierarchy:**
```
src/
├── core/           # PRIORITY 1: Essential trait abstractions only
│   ├── operation.rs    # Core Operation trait and basic types
│   ├── executor.rs     # Core OSExecutor trait (simplified)
│   ├── middleware.rs   # Core Middleware trait
│   ├── context.rs      # Execution contexts
│   └── result.rs       # Core result and error types
├── middleware/     # PRIORITY 2: Standalone middleware modules
│   ├── logger/         # Activity logging subsystem
│   └── security/       # Security validation subsystem (consolidated)
├── api/           # PRIORITY 3: High-level user APIs
├── executor/      # PRIORITY 3: OS-specific implementations
└── config/        # PRIORITY 4: Configuration management
```

### 3. Security-Consolidated Architecture
Based on user feedback to consolidate security in `middleware/security/`:

**Security Module Structure:**
```
middleware/security/
├── mod.rs          # Security middleware exports and orchestration
├── policy.rs       # Policy evaluation (replaces separate SecurityPolicy trait)
├── acl.rs          # Access Control Lists implementation
├── rbac.rs         # Role-Based Access Control implementation
└── audit.rs        # Security audit logging
```

**Security Integration Pattern:**
- All security concerns handled within security middleware
- No separate `SecurityPolicy` trait - integrated into `SecurityMiddleware`
- Security middleware processes all operations before execution

### 4. Simplified Error Handling Architecture
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

// Each error provides contextual helper methods
impl OSError {
    pub fn is_security_violation(&self) -> bool {
        matches!(self, OSError::SecurityViolation { .. })
    }
    
    pub fn is_filesystem_error(&self) -> bool {
        matches!(self, OSError::FilesystemError { .. })
    }
}
```

## Key Architectural Decisions

### Decision 1: Remove `validate_operation` from OSExecutor
**Rationale:** Follows separation of concerns and YAGNI principles
- Security validation is middleware responsibility
- Eliminates redundant validation logic
- Cleaner executor interface focused on execution only

### Decision 2: Remove `capabilities()` method
**Rationale:** YAGNI principle - not currently needed
- Type system enforces operation/executor compatibility
- Configuration-driven resource limits preferred
- Can be added later if proven necessary

### Decision 3: Generic Constraints Over `dyn` Patterns
**Rationale:** Following workspace standard §6.2 and Microsoft Guidelines M-DI-HIERARCHY
- Better compile-time type safety
- Superior performance characteristics
- Cleaner error messages and IDE integration
- Follows Rust ecosystem best practices

### Decision 4: Middleware Error Action Pattern
**Rationale:** Provides comprehensive error handling flexibility

```rust
pub enum ErrorAction {
    Continue,                    // Pass original error through
    ReplaceError(OSError),      // Replace with different error
    Suppress,                   // Suppress error (use carefully)
}

pub enum MiddlewareResult<T> {
    Ok(T),
    Err(MiddlewareError),
}

pub enum MiddlewareError {
    Fatal(String),              // Stop pipeline immediately
    NonFatal(String),           // Log warning, continue pipeline
    SecurityViolation(String),  // Security audit + stop pipeline
}
```

## Integration Patterns

### Pattern 1: Core Trait Composition
```rust
// Core operation trait with required bounds
pub trait Operation: Debug + Send + Sync + Clone + 'static {
    fn operation_id(&self) -> &str;
    fn operation_type(&self) -> OperationType;
    fn required_permissions(&self) -> Vec<Permission>;
    fn created_at(&self) -> DateTime<Utc>; // Following §3.2
}

// Executor trait parameterized by operation type
pub trait OSExecutor<O>: Debug + Send + Sync + 'static 
where O: Operation 
{
    async fn execute(&self, operation: O, context: &ExecutionContext) -> OSResult<ExecutionResult>;
}
```

### Pattern 2: Middleware Pipeline Orchestration
```rust
pub struct MiddlewarePipeline {
    middlewares: Vec<Box<dyn MiddlewareDispatcher>>,
}

trait MiddlewareDispatcher: Debug + Send + Sync {
    async fn before_execute_any(&self, operation: &dyn Operation, context: &mut ExecutionContext) -> MiddlewareResult<()>;
    async fn after_execute_any(&self, operation: &dyn Operation, result: &ExecutionResult, context: &ExecutionContext) -> MiddlewareResult<()>;
    async fn on_error_any(&self, operation: &dyn Operation, error: &OSError, context: &ExecutionContext) -> MiddlewareResult<ErrorAction>;
    fn priority(&self) -> u32;
}
```

## Performance Considerations

### Non-Goals (Following YAGNI §6.1)
- **Premature optimization**: Focus on correctness and security first
- **Complex caching**: Implement only when performance metrics prove necessary
- **Connection pooling**: Add only when concurrent usage patterns emerge
- **Micro-benchmarks**: Defer until after core functionality complete

### Design for Future Optimization
- **Async-first**: All I/O operations use async/await pattern
- **Zero-copy paths**: Design APIs to minimize unnecessary data copying
- **Resource lifecycle**: Clear ownership and cleanup patterns
- **Monitoring hooks**: Built-in metrics collection points for future optimization

## Compliance Checklist

### Workspace Standards Compliance
- ✅ §2.1: 3-layer import organization in all files
- ✅ §3.2: chrono DateTime<Utc> for all timestamps
- ✅ §4.3: mod.rs files contain only declarations and re-exports
- ✅ §5.1: Proper dependency layering in Cargo.toml
- ✅ §6.1: YAGNI principles - build only what's needed
- ✅ §6.2: Avoid dyn patterns, prefer generic constraints
- ✅ §6.3: Microsoft Rust Guidelines integration

### Microsoft Guidelines Compliance
- ✅ M-DI-HIERARCHY: Types > Generics > dyn traits
- ✅ M-AVOID-WRAPPERS: No smart pointers in public APIs
- ✅ M-SIMPLE-ABSTRACTIONS: Prevent cognitive nesting
- ✅ M-ERRORS-CANONICAL-STRUCTS: Structured error types
- ✅ M-SERVICES-CLONE: Cheap service cloning pattern

## Next Steps

### Immediate Implementation Priority
1. **Core module foundation** - Essential trait definitions
2. **Error handling framework** - Structured error types with contextual information
3. **Basic middleware framework** - Pipeline orchestration with error handling
4. **Logger middleware** - Activity logging implementation
5. **Security middleware** - Consolidated security validation

### Future Phases
1. **Executor implementations** - OS-specific operation handlers
2. **High-level APIs** - User-friendly interfaces
3. **Configuration system** - Runtime configuration management
4. **Integration testing** - Cross-component validation
5. **Performance optimization** - Based on real usage metrics

## References
- [Microsoft Rust Guidelines](https://microsoft.github.io/rust-guidelines/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Workspace Standards: `.copilot/memory_bank/workspace/shared_patterns.md`
- Project Brief: `.copilot/memory_bank/sub_projects/airssys-osl/project_brief.md`