# Core Types

*This section will document the core trait abstractions and types once OSL-TASK-001 (Core Module Foundation) is implemented.*

## Current Status
**Implementation Phase**: Foundation setup  
**Module Location**: `src/core/`  
**Documentation Source**: Memory bank specifications

Based on the documented architecture, this section will cover:

- **Operation Trait**: Core Operation trait and basic types
- **OSExecutor Trait**: Generic-based executor interface  
- **Middleware Trait**: Core middleware abstraction
- **Context Types**: Execution contexts and security context
- **Error Types**: Structured error types following M-ERRORS-CANONICAL-STRUCTS

## Generic-First Pattern
The documented core types follow the generic-first design pattern:

```rust
// Documented pattern - implementation pending
pub trait OSExecutor<O>: Debug + Send + Sync + 'static 
where O: Operation
{
    async fn execute(&self, operation: O, context: &ExecutionContext) -> OSResult<ExecutionResult>;
}
```

This section will be updated with comprehensive API documentation once the core implementation is completed.
