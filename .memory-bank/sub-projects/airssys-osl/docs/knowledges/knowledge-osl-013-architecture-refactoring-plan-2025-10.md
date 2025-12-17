# Architecture Refactoring Plan - October 2025

**Date:** 2025-10-08  
**Status:** Approved - Ready for Implementation  
**Impact:** airssys-osl (framework removal), airssys-osl-macros (new project)

## Overview

After completing OSL-TASK-008 Phases 1-4 (Platform Executors with 165 tests passing), extensive architectural analysis revealed that the framework layer adds unnecessary complexity. This document outlines the approved refactoring plan to simplify the architecture while improving ergonomics.

## Strategic Decision

**Remove framework abstractions, provide three usage levels:**
1. **Low-level Library** (`airssys-osl`): Core abstractions for direct usage
2. **Proc-Macros** (`airssys-osl-macros`): Ergonomic custom executor creation  
3. **Helper Functions**: One-line convenience APIs for common operations

## Rationale

### Problems Identified
1. **ExecutorRegistry**: Cannot store heterogeneous executor types (generic-to-dynamic impedance mismatch)
2. **OSLFramework**: Adds indirection without proportional value
3. **Builder Patterns**: Unnecessary complexity for simple APIs
4. **Over-engineering**: Violates YAGNI principles

### Benefits of New Approach
- **Simpler codebase**: ~30% code reduction in framework layer
- **Clearer architecture**: Three distinct, focused usage levels
- **Better ergonomics**: Helper functions easier than framework methods
- **Type safety**: Macros generate compile-time checked code
- **Maintainability**: Less abstraction to understand and maintain
- **Zero cost**: All abstractions are compile-time only

## Implementation Plan

### Sub-Project 1: airssys-osl-macros (New)

**Purpose:** Procedural macros for ergonomic trait implementations

#### Tasks
1. **MACROS-TASK-001: Foundation Setup** (4 hours, In Progress)
   - Create proc-macro crate structure
   - Add to workspace members
   - Configure dependencies (syn, quote, proc-macro2)
   - Setup test infrastructure
   - Memory bank structure complete ✅

2. **MACROS-TASK-002: #[executor] Macro** (2-3 weeks, Pending)
   - Implement method parsing with syn v2
   - Create operation name mapping table (10 operations)
   - Generate OSExecutor<O> trait implementations
   - Comprehensive error messages
   - Full test coverage (unit, integration, UI tests)

3. **MACROS-TASK-003: Integration** (1 week, Pending)
   - Feature flag setup in airssys-osl
   - Re-export macros from prelude
   - Integration testing
   - Documentation and examples

#### Deliverables
- Proc-macro crate for code generation
- `#[executor]` macro for custom executors
- ~85% code reduction for custom executor implementations
- Zero runtime cost (compile-time only)

### Sub-Project 2: airssys-osl Refactoring

**Purpose:** Remove framework, add helpers and extension traits

#### OSL-TASK-009: Remove Framework and Add Helpers (2-3 days, Pending)

**Phase 1: Framework Removal**
- Delete `src/framework/registry.rs`
- Delete `src/framework/framework.rs`  
- Delete `src/framework/builder.rs`
- Delete `src/framework/pipeline.rs` (if unused)
- Review and possibly keep `config.rs`, `operations.rs`
- Update `lib.rs` and `prelude.rs`

**Phase 2: Helper Functions**
- Create `src/helpers.rs` with 10 convenience functions:
  - Filesystem (4): read_file, write_file, delete_file, create_directory
  - Process (3): spawn_process, kill_process, query_process
  - Network (3): tcp_connect, tcp_listen, udp_bind
- All helpers use default platform executors internally
- Comprehensive rustdoc with examples

**Phase 3: Middleware Extension Trait**
- Create `src/middleware/ext.rs`
- Implement `ExecutorExt` trait with `.with_middleware()` method
- Enable composable middleware wrapping
- Generic implementation for all types

**Phase 4: Testing & Documentation**
- Helper function tests (10+)
- Middleware extension tests (3+)
- Verify all existing tests still pass (165+)
- Update mdBook documentation
- Update examples

#### Deliverables
- Simplified core library structure
- 10 ergonomic helper functions
- Composable middleware via extension trait
- All 165+ tests passing
- Updated documentation

#### Impact on OSL-TASK-008 Phase 5
- **Status:** Abandoned - Replaced by OSL-TASK-009
- **Reason:** Registry integration impossible due to type system constraints
- **Resolution:** New architecture eliminates need for registry

## Three Usage Levels

### Level 1: Low-Level API (Maximum Control)
```rust
use airssys_osl::prelude::*;

let executor = FilesystemExecutor::new();
let operation = FileReadOperation::new("/etc/hosts".into());
let context = ExecutionContext::new(SecurityContext::new("admin".into()));
let result = executor.execute(operation, &context).await?;
```

**When to use:** Custom executor requirements, fine-grained control, library development

### Level 2: Helper Functions (Most Ergonomic)
```rust
use airssys_osl::helpers::*;

let data = read_file("/etc/hosts", "admin").await?;
```

**When to use:** Application development, quick prototyping, common operations

### Level 3: Custom Executors with Macros (Advanced)
```rust
use airssys_osl::prelude::*;

#[executor]
impl CloudExecutor {
    async fn file_read(&self, op: FileReadOperation, ctx: &ExecutionContext) 
        -> OSResult<ExecutionResult> 
    {
        // Custom cloud-based implementation
        let data = self.client.download(&op.path).await?;
        Ok(ExecutionResult::success(data))
    }
}
```

**When to use:** Custom storage backends, specialized environments, framework integration

### Middleware Composition (All Levels)
```rust
use airssys_osl::middleware::{LoggerMiddleware, ExecutorExt};

let executor = FilesystemExecutor::new()
    .with_middleware(|e| LoggerMiddleware::new(e, logger))
    .with_middleware(|e| MetricsMiddleware::new(e, collector))
    .with_middleware(|e| SecurityMiddleware::new(e, policy));
```

**How it works:** Extension trait provides `.with_middleware()` for composable wrapping

## Technical Details

### Operation Name Mapping (Macros)

The `#[executor]` macro maps method names to operation types:

| Method Name | Operation Type | Module Path |
|-------------|----------------|-------------|
| file_read | FileReadOperation | airssys_osl::operations::filesystem |
| file_write | FileWriteOperation | airssys_osl::operations::filesystem |
| file_delete | FileDeleteOperation | airssys_osl::operations::filesystem |
| directory_create | DirectoryCreateOperation | airssys_osl::operations::filesystem |
| process_spawn | ProcessSpawnOperation | airssys_osl::operations::process |
| process_kill | ProcessKillOperation | airssys_osl::operations::process |
| process_query | ProcessQueryOperation | airssys_osl::operations::process |
| tcp_connect | TcpConnectOperation | airssys_osl::operations::network |
| tcp_listen | TcpListenOperation | airssys_osl::operations::network |
| udp_bind | UdpBindOperation | airssys_osl::operations::network |

### Middleware Extension Pattern

```rust
// In middleware/ext.rs
pub trait ExecutorExt: Sized {
    fn with_middleware<M>(self, middleware_ctor: impl FnOnce(Self) -> M) -> M {
        middleware_ctor(self)
    }
}

impl<E> ExecutorExt for E where E: Sized {}
```

**Usage:**
```rust
let executor = FilesystemExecutor::new()
    .with_middleware(|e| LoggerMiddleware::new(e, logger));
```

**Type composition:**
```
WithMiddleware<
    WithMiddleware<
        FilesystemExecutor,
        LoggerMiddleware
    >,
    MetricsMiddleware
>
```

## Timeline and Prioritization

### Week 1 (Current)
- **MACROS-TASK-001**: Complete foundation setup (4 hours remaining)
- **OSL-TASK-009**: Start framework removal and helpers (2-3 days)

### Week 2-3
- **OSL-TASK-009**: Complete and test
- **MACROS-TASK-002**: Implement #[executor] macro (ongoing)

### Week 4-5
- **MACROS-TASK-002**: Testing and documentation
- **MACROS-TASK-003**: Integration with airssys-osl

### Week 6
- **Polish**: Documentation, examples, performance validation
- **Release**: airssys-osl v0.2.0 (simplified architecture)

## Success Metrics

### Code Quality
- ✅ All existing tests pass (165+)
- ✅ New tests for helpers and macros (20+)
- ✅ Zero compiler warnings
- ✅ Zero clippy warnings
- ✅ 100% rustdoc coverage

### Architecture
- ✅ ~30% code reduction in airssys-osl
- ✅ ~85% code reduction for custom executors (with macros)
- ✅ Clear separation of concerns (3 usage levels)
- ✅ YAGNI compliance

### Developer Experience
- ✅ One-line APIs for common operations
- ✅ Clear, actionable macro error messages
- ✅ Comprehensive documentation with examples
- ✅ Easy migration path (no breaking changes for core APIs)

## Risk Mitigation

### Risk: Macro Complexity
- **Mitigation**: Start with minimal viable implementation, iterate based on feedback
- **Validation**: Comprehensive UI tests for error messages

### Risk: Breaking Changes
- **Mitigation**: Core abstractions unchanged, only framework layer affected
- **Validation**: No external users yet, so minimal migration concerns

### Risk: Proc-Macro Compilation Time
- **Mitigation**: Generate minimal code, use lazy static for mappings
- **Validation**: Measure build time impact (<2% target)

## Documentation Updates

### Memory Bank
- ✅ Created airssys-osl-macros sub-project
- ✅ Product context, technical context, progress tracking
- ✅ System patterns documentation
- ✅ Task definitions (MACROS-TASK-001, 002)
- ⏳ Update airssys-osl progress.md (pending OSL-TASK-009 start)
- ⏳ Update current_context.md (pending)

### User Documentation
- ⏳ mdBook updates (new architecture chapter)
- ⏳ README updates (three usage levels)
- ⏳ Examples (helpers, macros, middleware composition)
- ⏳ Migration guide (if needed)

## Approval and Next Steps

### Approved By
User confirmed on 2025-10-08: "I'm agree with your approaches, not using framework and extension trait it much simpler."

### Next Actions
1. ✅ Save complete plan to memory bank
2. ⏳ Add airssys-osl-macros to workspace Cargo.toml
3. ⏳ Create basic proc-macro crate structure
4. ⏳ Complete MACROS-TASK-001 foundation setup
5. ⏳ Start OSL-TASK-009 implementation

## Notes

- This is a simplification refactoring, not a feature addition
- Maintains all existing functionality with clearer semantics
- Aligns with YAGNI and Microsoft Rust Guidelines
- No external users yet, so no breaking change concerns
- Foundation for future macro expansions (#[operation], #[middleware])
