# ADR-026: Framework as Primary API Strategy

**Status**: Accepted  
**Date**: 2025-10-03  
**Deciders**: Development Team  
**Category**: Architecture Patterns  

## Context

AirsSys OSL provides two API levels:
1. **Explicit Primitives**: Direct access to core traits and types
2. **Framework Builder**: High-level ergonomic interface

The question is: Which API should be primary in documentation, examples, and developer guidance?

## Decision

**OSLFramework Builder will be the PRIMARY API** with explicit primitives available for advanced use cases.

### API Hierarchy (Primary → Advanced)
```rust
// ✅ PRIMARY API - Default recommendation for most engineers
use airssys_osl::prelude::*;

let osl = OSLFramework::builder()
    .with_security_logging(true)
    .with_policy_enforcement("/etc/osl/policies")
    .build().await?;

let result = osl
    .filesystem()
    .read_file("/app/config.toml")
    .with_user("app-user")
    .execute().await?;
```

```rust
// ✅ ADVANCED API - Available when needed for custom requirements
use airssys_osl::core::{
    context::{ExecutionContext, SecurityContext},
    middleware::{Middleware, LoggerMiddleware},
    executor::OSExecutor,
};

let middleware = LoggerMiddleware::new(config);
let result = middleware.before_execute(operation, &context).await?;
```

## Rationale

### Primary Framework API Benefits
1. **Reduced Cognitive Load**: Engineers focus on business logic, not infrastructure
2. **Best Practices Enforcement**: Framework ensures correct patterns automatically  
3. **Faster Onboarding**: New developers productive within 15 minutes
4. **Error Prevention**: Automatic middleware orchestration prevents common mistakes
5. **Consistency**: Standardized patterns across all applications

### When to Use Advanced API
- **Custom Middleware**: Implementing new middleware types
- **Performance Optimization**: Bypassing framework overhead in hot paths
- **Debugging**: Direct access to internal components for troubleshooting
- **Advanced Patterns**: Complex workflow orchestration not supported by framework

### Documentation Strategy
```
docs/
├── introduction.md         # Starts with Framework API
├── quick-start.md         # Framework examples only
├── guides/
│   ├── common-tasks.md    # Framework-based solutions
│   └── advanced/          # Explicit API documentation
│       ├── custom-middleware.md
│       ├── performance-tuning.md
│       └── debugging.md
└── api/
    ├── framework/         # Primary API documentation
    └── core/             # Advanced API documentation
```

## Implementation Requirements

### 1. Prelude Module Design
```rust
// src/prelude.rs - Primary imports for most applications
pub use crate::{
    OSLFramework, 
    OSLFrameworkBuilder,
    operations::{FilesystemBuilder, ProcessBuilder, NetworkBuilder},
};

pub use crate::core::{
    result::{OSResult, OSError},
    operation::OperationType,
};

// Common configurations
pub use crate::middleware::{LogLevel, EnforcementLevel};
```

### 2. Framework-First Examples
All documentation examples start with framework API:
```rust
use airssys_osl::prelude::*;

#[tokio::main]
async fn main() -> OSResult<()> {
    let osl = OSLFramework::builder()
        .with_default_security()
        .build().await?;
    
    // Common operations made simple
    let content = osl.filesystem()
        .read_file("/etc/config.toml")
        .execute().await?;
    
    Ok(())
}
```

### 3. Progressive Disclosure
- **Level 1**: Framework builder (covers 80% of use cases)
- **Level 2**: Framework + custom middleware (covers 15% of use cases)  
- **Level 3**: Direct primitives (covers 5% of advanced use cases)

## Consequences

### Positive
- ✅ Dramatically improved developer experience
- ✅ Faster adoption and onboarding
- ✅ Consistent application patterns
- ✅ Reduced support burden (fewer API misuse issues)
- ✅ Clear migration path (framework → primitives when needed)

### Negative
- ❌ Advanced users may need to discover primitive APIs
- ❌ Some applications may use framework when primitives would be better
- ❌ Documentation maintains examples for both API levels

### Mitigation Strategies
- **Clear Documentation**: Advanced API clearly documented and linked
- **Performance Guidance**: Clear guidance on when to use primitives
- **Migration Examples**: Show how to move between API levels
- **IDE Support**: Good autocomplete and type hints for discovery

## Success Metrics

### Developer Experience
- [ ] New developers achieve first success within 15 minutes
- [ ] Common operations require ≤ 10 lines of code
- [ ] 80% of applications use only framework API
- [ ] Support requests focus on business logic, not API usage

### Technical Quality
- [ ] Zero performance regression for primitive users
- [ ] Framework overhead < 10% for common operations
- [ ] 100% test coverage across both API levels
- [ ] Clear error messages guide users to appropriate API level

## Implementation Timeline

### Phase 1: Foundation (OSL-TASK-005)
- Create prelude module with framework exports
- Update documentation to lead with framework examples
- Basic OSLFramework structure

### Phase 2: Core Builder (OSL-TASK-006)  
- Complete OSLFramework builder functionality
- Operation builders for common tasks
- Comprehensive framework examples

### Phase 3: Documentation Completion
- Migrate all examples to framework-first approach
- Advanced API documentation for primitive access
- Performance guidance and migration patterns

## References
- Knowledge Doc 004: API Ergonomics Architecture Analysis
- Knowledge Doc 005: Strategic Prioritization Rationale
- OSL-TASK-005: API Ergonomics Foundation
- OSL-TASK-006: Core Builder Implementation