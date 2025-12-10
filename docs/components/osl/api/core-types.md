# Core Types

This section documents the core trait abstractions and types in AirsSys OSL.

## Current Status
**Implementation Phase**: ✅ **Implemented**  
**Module Location**: `src/core/`  
**RustDoc**: Run `cargo doc --open` in `airssys-osl` for complete API documentation

## Overview

The core module provides the foundational abstractions for the OSL framework:

- **Operation Trait**: Core Operation trait defining operations that can be executed
- **OSExecutor Trait**: Generic-based executor interface for running operations  
- **Middleware Trait**: Core middleware abstraction for operation interception
- **Context Types**: Execution contexts and security context for operation metadata
- **Error Types**: Structured error types following canonical error patterns

## Generic-First Pattern

The core types follow a generic-first design pattern to enable zero-cost abstractions:

```rust
pub trait OSExecutor<O>: Debug + Send + Sync + 'static 
where O: Operation
{
    async fn execute(&self, operation: O, context: &ExecutionContext) -> OSResult<ExecutionResult>;
}
```

## Key Traits

### Operation Trait

Defines the interface for operations that can be executed:

```rust
pub trait Operation: Debug + Send + Sync + 'static {
    fn operation_type(&self) -> OperationType;
    fn requires_privilege(&self) -> bool;
}
```

### OSExecutor Trait

Generic executor interface:

```rust
pub trait OSExecutor<O: Operation>: Debug + Send + Sync + 'static {
    async fn execute(&self, operation: O, context: &ExecutionContext) 
        -> OSResult<ExecutionResult>;
}
```

### Middleware Trait

Operation interception interface:

```rust
#[async_trait]
pub trait Middleware<O: Operation>: Send + Sync {
    fn name(&self) -> &str;
    fn priority(&self) -> u32;
    
    async fn before_execution(&self, operation: O, context: &ExecutionContext) 
        -> MiddlewareResult<Option<O>>;
        
    async fn after_execution(&self, operation: &O, result: ExecutionResult, context: &ExecutionContext) 
        -> MiddlewareResult<ExecutionResult>;
}
```

## Context Types

### ExecutionContext

Contains metadata for operation execution:

```rust
pub struct ExecutionContext {
    security: SecurityContext,
    metadata: HashMap<String, String>,
    // ... additional fields
}
```

### SecurityContext

Encapsulates security information:

```rust
pub struct SecurityContext {
    principal: String,
    roles: Vec<String>,
    permissions: Vec<String>,
}
```

## Error Types

Structured error handling with `OSError` and `OSResult<T>`:

```rust
pub type OSResult<T> = Result<T, OSError>;

pub enum OSError {
    PermissionDenied { operation: String, principal: String },
    NotFound { resource: String },
    ExecutionFailed { details: String },
    // ... other variants
}
```

For complete API documentation with all methods and examples, see the generated RustDoc (`cargo doc --open`).
