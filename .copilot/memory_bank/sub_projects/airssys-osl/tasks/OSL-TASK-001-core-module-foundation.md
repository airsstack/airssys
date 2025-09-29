# Task: Implement Core Module Foundation

**Task ID:** OSL-TASK-001  
**Priority:** Critical  
**Status:** Phases 1 & 2 COMPLETED ✅ - Ready for Phase 3  
**Created:** 2025-09-27  
**Updated:** 2025-09-29  
**Estimated Effort:** 2-3 days  
**Implementation Plan:** [OSL-TASK-001-implementation-plan.md](./OSL-TASK-001-implementation-plan.md)  

## Task Overview
Implement the core module foundation containing all essential trait abstractions and types following the revised architecture plan and technical standards.

## Task Description
Create the complete `src/core/` module with all foundational traits, types, and abstractions that other components will depend on. This is the critical path foundation that must be completed before any other implementation work.

## Acceptance Criteria

### 1. Module Structure Created
- ✅ `src/core/mod.rs` - Clean module declarations and re-exports only (§4.3)
- ✅ `src/core/operation.rs` - Core Operation trait and operation types
- ✅ `src/core/executor.rs` - Core OSExecutor trait (generic-based, no dyn)
- ✅ `src/core/middleware.rs` - Core Middleware trait with error handling
- ✅ `src/core/context.rs` - ExecutionContext and SecurityContext types
- ✅ `src/core/result.rs` - Structured error types following M-ERRORS-CANONICAL-STRUCTS

### 2. Technical Standards Compliance
- ✅ All files follow §2.1 3-layer import organization
- ✅ All timestamps use chrono DateTime<Utc> (§3.2)
- ✅ Zero dyn patterns - use generic constraints only (§6.2)
- ✅ YAGNI principles applied - no premature complexity (§6.1)
- ✅ Microsoft Rust Guidelines compliance (§6.3)

### 3. Core Trait Definitions
- ✅ `Operation` trait with required bounds and methods
- ✅ `OSExecutor<O: Operation>` trait with generic constraints
- ✅ `Middleware<O: Operation>` trait with comprehensive error handling
- ✅ Error types with contextual helper methods (is_security(), is_filesystem(), etc.)

### 4. Quality Gates
- ✅ Zero compiler warnings
- ✅ All public APIs have comprehensive rustdoc
- ✅ Unit tests for all core trait boundaries
- ✅ Compiles with `cargo check --workspace`
- ✅ Passes `cargo clippy --workspace --all-targets --all-features`

## Implementation Details

### Core Module Structure
```
src/core/
├── mod.rs              # Module declarations and re-exports only
├── operation.rs        # Operation trait and OperationType enum
├── executor.rs         # OSExecutor trait with generic constraints  
├── middleware.rs       # Middleware trait and pipeline types
├── context.rs          # ExecutionContext and SecurityContext
└── result.rs           # OSError, OSResult, and related error types
```

### Key Design Requirements

#### Generic-Based Design (No dyn)
```rust
// ✅ REQUIRED: Generic constraints pattern
pub trait OSExecutor<O>: Debug + Send + Sync + 'static 
where O: Operation
{
    async fn execute(&self, operation: O, context: &ExecutionContext) -> OSResult<ExecutionResult>;
}

// ❌ FORBIDDEN: dyn patterns
pub trait OSExecutor {
    async fn execute(&self, operation: &dyn Operation) -> OSResult<ExecutionResult>;
}
```

#### Comprehensive Error Handling
```rust
#[derive(Error, Debug)]
pub enum OSError {
    #[error("Security policy violation: {reason}")]
    SecurityViolation { reason: String },
    
    #[error("Middleware failed: {middleware}: {reason}")]  
    MiddlewareFailed { middleware: String, reason: String },
    
    // Include Backtrace following M-ERRORS-CANONICAL-STRUCTS
}

impl OSError {
    pub fn is_security_violation(&self) -> bool;
    pub fn is_middleware_failure(&self) -> bool;
}
```

#### Middleware Error Actions
```rust
pub enum MiddlewareResult<T> {
    Ok(T),
    Err(MiddlewareError),
}

pub enum MiddlewareError {
    Fatal(String),              // Stop pipeline immediately
    NonFatal(String),           // Log warning, continue
    SecurityViolation(String),  // Audit + stop
}

pub enum ErrorAction {
    Continue,                   // Pass original error
    ReplaceError(OSError),      // Replace with different error
    Suppress,                   // Suppress error (dangerous)
}
```

## Dependencies
- **Blocked by:** None (critical path foundation)
- **Blocks:** All other OSL implementation tasks
- **Related:** Memory bank technical standards update (completed)

## Testing Requirements
- Unit tests for all trait boundaries and constraints
- Error type instantiation and helper method tests
- Context type serialization/deserialization tests
- Integration test scaffolding preparation

## Documentation Requirements
- Comprehensive rustdoc for all public traits and types
- Code examples in documentation
- Cross-references to Microsoft Rust Guidelines
- Integration with existing knowledge documentation

## Success Metrics
- All core traits compile without warnings
- Other team members can implement executors and middleware against these traits
- Clean separation between core abstractions and implementation details
- Foundation ready for middleware and executor implementation

## Notes
- This is the critical path foundation task - highest priority
- All architectural decisions documented in 001-core-architecture-foundations.md
- Must be completed before any middleware or executor work begins
- Sets the pattern for all future implementation work

## Dependencies and Blockers
**Prerequisites:**
- Technical standards update (completed)
- Core architecture knowledge documentation (completed)

**Deliverables Needed For:**
- OSL-TASK-002: Logger Middleware Implementation
- OSL-TASK-003: Security Middleware Implementation  
- OSL-TASK-004: Executor Implementation Framework

**Cross-References:**
- Knowledge Doc: 001-core-architecture-foundations.md
- Workspace Standards: §2.1, §3.2, §4.3, §5.1, §6.1, §6.2, §6.3
- Microsoft Guidelines: M-DI-HIERARCHY, M-ERRORS-CANONICAL-STRUCTS, M-AVOID-WRAPPERS