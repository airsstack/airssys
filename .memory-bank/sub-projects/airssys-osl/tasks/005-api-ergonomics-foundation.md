# OSL-TASK-005: API Ergonomics Foundation

**Task ID**: OSL-TASK-005  
**Created**: 2025-09-29  
**Priority**: Critical  
**Estimated Duration**: 4-6 hours  
**Dependencies**: OSL-TASK-001 (Phase 4 completion)  
**Status**: Ready for Implementation  

## Objective

Implement foundational components for hybrid API architecture, providing both explicit primitives and ergonomic builder patterns. This task establishes the infrastructure for developer-friendly APIs while maintaining the current explicit primitive access.

## Background

Based on analysis in `004-api-ergonomics-architecture.md`, we identified a critical need for:
1. **Prelude module** for convenient imports
2. **OSLFramework foundation** for builder pattern infrastructure  
3. **Documentation alignment** with actual implementation
4. **Migration foundation** for future ergonomic enhancements

## Acceptance Criteria

### 1. Prelude Module Implementation
- [ ] Create `src/prelude.rs` with common type re-exports
- [ ] Include all frequently used types (SecurityContext, ExecutionContext, OSResult, etc.)
- [ ] Provide ergonomic imports without breaking explicit API
- [ ] Update `src/lib.rs` to expose prelude module

### 2. OSLFramework Foundation Structure
- [ ] Create `src/framework/mod.rs` with basic structure
- [ ] Implement `OSLFramework` struct with core fields
- [ ] Add `OSLFrameworkBuilder` with builder pattern foundation
- [ ] Design internal architecture for middleware pipeline integration

### 3. Documentation Alignment
- [ ] Update `docs/src/introduction.md` to reflect current explicit API
- [ ] Add "Quick Start with Explicit API" section
- [ ] Maintain builder API examples as "planned features"
- [ ] Ensure all documented examples actually compile and work

### 4. Integration Foundation
- [ ] Create infrastructure for middleware pipeline management
- [ ] Design executor registry system for framework
- [ ] Plan operation builder architecture
- [ ] Establish testing patterns for multiple API levels

## Technical Implementation Details

### Prelude Module Structure
```rust
// src/prelude.rs
//! Convenient imports for common AirsSys OSL usage patterns.

// Re-export core types that are used in 90%+ of applications
pub use crate::core::{
    context::{ExecutionContext, SecurityContext},
    result::{OSError, OSResult},
    operation::{Operation, OperationType, Permission},
    executor::{ExecutionResult, OSExecutor},
    middleware::{Middleware, MiddlewareError, ErrorAction},
};

// Framework types (to be implemented)
#[cfg(feature = "framework")]
pub use crate::framework::{OSLFramework, OSLFrameworkBuilder};

// Common standard library re-exports
pub use std::time::Duration;
pub use chrono::{DateTime, Utc};
```

### Framework Foundation Architecture
```rust
// src/framework/mod.rs
use std::collections::HashMap;
use crate::core::{
    context::SecurityContext,
    operation::OperationType,
    executor::OSExecutor,
    middleware::Middleware,
};

/// Main framework entry point for high-level OSL operations.
#[derive(Debug)]
pub struct OSLFramework {
    middleware_pipeline: MiddlewarePipeline,
    executors: ExecutorRegistry,
    security_context: SecurityContext,
    config: OSLConfig,
}

/// Builder for configuring and creating OSLFramework instances.
#[derive(Debug, Default)]
pub struct OSLFrameworkBuilder {
    middlewares: Vec<Box<dyn Middleware<dyn Operation>>>,
    executors: HashMap<OperationType, Box<dyn OSExecutor<dyn Operation>>>,
    security_config: Option<SecurityConfig>,
    logging_config: Option<LoggingConfig>,
}

impl OSLFrameworkBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    
    // Foundation methods (implementation in later tasks)
    pub fn with_security_logging(mut self, enabled: bool) -> Self {
        // TODO: Implementation in OSL-TASK-006
        self
    }
    
    pub async fn build(self) -> OSResult<OSLFramework> {
        // TODO: Implementation in OSL-TASK-006
        unimplemented!("Builder implementation in OSL-TASK-006")
    }
}
```

### Pipeline Infrastructure
```rust
// src/framework/pipeline.rs
/// Internal middleware pipeline management
#[derive(Debug)]
struct MiddlewarePipeline {
    middlewares: Vec<Box<dyn Middleware<dyn Operation>>>,
    sorted: bool,
}

impl MiddlewarePipeline {
    fn new() -> Self {
        Self {
            middlewares: Vec::new(),
            sorted: false,
        }
    }
    
    fn add_middleware(&mut self, middleware: Box<dyn Middleware<dyn Operation>>) {
        self.middlewares.push(middleware);
        self.sorted = false;
    }
    
    fn ensure_sorted(&mut self) {
        if !self.sorted {
            self.middlewares.sort_by_key(|m| m.priority());
            self.sorted = true;
        }
    }
    
    // Full implementation in OSL-TASK-006
}
```

## File Structure Changes

```
airssys-osl/
├── src/
│   ├── lib.rs                 # Updated: expose prelude
│   ├── prelude.rs            # New: convenient imports
│   ├── framework/            # New: framework infrastructure
│   │   ├── mod.rs           # New: OSLFramework + builder
│   │   ├── pipeline.rs      # New: middleware pipeline
│   │   └── config.rs        # New: framework configuration
│   └── core/                # Existing: unchanged
└── docs/
    └── src/
        └── introduction.md   # Updated: align with implementation
```

## Testing Strategy

### Unit Tests
- [ ] Prelude module imports work correctly
- [ ] OSLFrameworkBuilder basic construction
- [ ] Framework configuration validation
- [ ] Integration with existing core types

### Integration Tests  
- [ ] Prelude imports don't conflict with explicit imports
- [ ] Framework builder doesn't break existing API
- [ ] Documentation examples compile and execute
- [ ] Error handling consistency across API levels

### Documentation Tests
- [ ] All examples in introduction.md compile successfully
- [ ] Quick start guide works end-to-end
- [ ] API documentation is accurate and complete

## Dependencies and Blockers

### Prerequisites
- **OSL-TASK-001 Phase 4 completion**: Need finalized testing and documentation
- **Memory bank update**: Document new architecture decisions
- **Cargo.toml updates**: Add optional framework feature flag

### External Dependencies
- None (using existing crate dependencies)

### Potential Blockers
- Trait object compatibility issues with middleware/executor registries
- Generic parameter complexity in framework builder
- Performance implications of dynamic dispatch

## Implementation Plan

### Phase 1: Foundation Setup (2 hours)
1. Create prelude module with basic re-exports
2. Set up framework module structure
3. Add basic OSLFramework and builder structs
4. Update lib.rs and module declarations

### Phase 2: Infrastructure (2 hours)
1. Implement basic pipeline infrastructure
2. Create configuration types
3. Add executor registry foundation
4. Set up feature flag infrastructure

### Phase 3: Documentation Alignment (1-2 hours)
1. Update introduction.md with current API examples
2. Add prelude usage examples
3. Document migration path to builder API
4. Ensure all examples compile

### Phase 4: Testing and Validation (1 hour)
1. Write unit tests for new components
2. Integration tests for API compatibility
3. Documentation tests for examples
4. Performance baseline measurements

## Success Metrics

### Functionality Metrics
- [ ] All existing tests continue to pass
- [ ] New prelude module provides expected convenience
- [ ] Framework infrastructure ready for builder implementation
- [ ] Zero breaking changes to current API

### Developer Experience Metrics
- [ ] Import statements reduced by 50%+ with prelude
- [ ] Documentation examples all compile and execute
- [ ] Clear migration path established
- [ ] Consistent error messages across API levels

### Technical Quality Metrics
- [ ] No performance regression in core operations
- [ ] Clean separation between framework and core APIs
- [ ] Maintainable architecture for future enhancements
- [ ] Comprehensive test coverage (>95%)

## Deliverables

1. **Code**:
   - Functional prelude module
   - Framework foundation structure
   - Updated documentation with accurate examples
   - Comprehensive test suite

2. **Documentation**:
   - Updated introduction.md with current API
   - Architecture documentation for framework design
   - Migration guide for future builder adoption
   - API reference updates

3. **Memory Bank Updates**:
   - Task completion documentation
   - Architecture decision records
   - Development progress tracking
   - Next phase preparation

## Follow-up Tasks

This task enables:
- **OSL-TASK-006**: Core Builder Implementation
- **OSL-TASK-007**: Operation Builder Development  
- **OSL-TASK-008**: Advanced Framework Features
- **Documentation improvements** across all components

## Notes

- Maintain backward compatibility at all costs
- Use feature flags for optional framework components
- Keep framework implementation separate from core primitives
- Document all architectural decisions in memory bank