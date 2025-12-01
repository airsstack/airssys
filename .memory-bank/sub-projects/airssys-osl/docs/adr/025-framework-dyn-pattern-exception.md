# ADR-025: Framework dyn Pattern Exception

**Status**: Accepted  
**Date**: 2025-10-03  
**Deciders**: Development Team  
**Category**: Architecture Patterns  

## Context

AirsSys OSL follows a strict "Generic-First" design pattern as documented in:
- Knowledge Document 001: Core Architecture Foundations
- Workspace Standard §6.2: "Avoid dyn patterns, prefer generic constraints"
- Microsoft Guidelines M-DI-HIERARCHY: "Concrete types > Generics > dyn traits"

However, the OSLFramework builder layer requires dynamic dispatch for:
1. **Middleware Collection**: `Vec<Box<dyn Middleware<dyn Operation>>>`
2. **Executor Registry**: `HashMap<OperationType, Box<dyn OSExecutor<dyn Operation>>>`
3. **Runtime Composition**: Users can add arbitrary middleware at runtime

## Decision

We will create a **Strategic Architecture Exception** allowing `dyn` patterns specifically in the OSLFramework layer while maintaining generic-first patterns in core primitives.

### Framework Layer (dyn Allowed)
```rust
// ✅ EXCEPTION: Framework ergonomics require dynamic dispatch
pub struct OSLFramework {
    middleware_pipeline: MiddlewarePipeline,
    executors: HashMap<OperationType, Box<dyn OSExecutor<dyn Operation>>>,
    security_context: SecurityContext,
}

pub struct OSLFrameworkBuilder {
    middlewares: Vec<Box<dyn Middleware<dyn Operation>>>,
    executors: HashMap<OperationType, Box<dyn OSExecutor<dyn Operation>>>,
}
```

### Core Primitives Layer (Generic-First Maintained)
```rust
// ✅ STANDARD: Core primitives remain generic-first
pub trait OSExecutor<O>: Debug + Send + Sync + 'static 
where O: Operation 
{
    async fn execute(&self, operation: O, context: &ExecutionContext) -> OSResult<ExecutionResult>;
}

pub trait Middleware<O>: Debug + Send + Sync + 'static 
where O: Operation 
{
    async fn before_execute(&self, operation: &O, context: &mut ExecutionContext) -> MiddlewareResult<()>;
}
```

## Rationale

### Benefits of dyn Exception in Framework Layer
1. **Developer Ergonomics**: Simple builder API without complex generic signatures
2. **Runtime Flexibility**: Users can register middleware dynamically
3. **API Simplicity**: Framework hides type complexity from application developers
4. **Plugin Architecture**: Enables middleware plugins and dynamic loading

### Maintaining Generic-First in Core
1. **Performance**: Core operations maintain zero-cost abstractions
2. **Type Safety**: Compile-time guarantees for performance-critical paths
3. **Advanced Users**: Direct access to high-performance primitives
4. **Testing**: Easier mocking and testing with generic constraints

### Architecture Boundary
```
Application Code
    ↓
Framework Layer (dyn allowed for ergonomics)
    ↓
Core Primitives (generic-first for performance)
    ↓
OS Layer
```

## Consequences

### Positive
- ✅ Simple framework API reduces cognitive load
- ✅ Core performance maintained for advanced users
- ✅ Clear separation between ergonomic and performance layers
- ✅ Enables dynamic plugin architectures

### Negative
- ❌ Some runtime overhead in framework layer
- ❌ Requires maintaining two API paradigms
- ❌ Documentation complexity (multiple usage patterns)

### Mitigation
- **Performance**: Profile framework overhead and optimize hot paths
- **Complexity**: Clear documentation hierarchy (framework → primitives)
- **Maintenance**: Comprehensive testing across both API levels

## Compliance Notes

This decision creates a **documented exception** to workspace standard §6.2 for the specific case of framework ergonomics while maintaining compliance in core components.

## References
- Workspace Standards: §6.2 (dyn pattern avoidance)
- Knowledge Doc 001: Core Architecture Foundations
- Knowledge Doc 004: API Ergonomics Architecture Analysis
- Microsoft Rust Guidelines: M-DI-HIERARCHY