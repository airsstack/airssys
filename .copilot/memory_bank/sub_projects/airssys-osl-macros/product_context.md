# airssys-osl-macros Product Context

## Project Identity
**Project Name:** airssys-osl-macros  
**Version:** 0.1.0  
**Type:** Proc-Macro Crate  
**Status:** Foundation Setup  
**Created:** 2025-10-08

## Purpose and Vision

### Core Mission
Provide procedural macros for ergonomic implementation of airssys-osl core abstractions, reducing boilerplate code while maintaining type safety and zero-cost abstractions.

### Strategic Goals
1. **Developer Ergonomics**: Reduce ~85% of boilerplate code for custom executor implementations
2. **Type Safety**: Generate compile-time checked trait implementations from user methods
3. **Extensibility**: Support all core abstractions (Operation, OSExecutor, Middleware)
4. **Zero Runtime Cost**: Pure compile-time code generation with no runtime overhead

### Target Users
- **Application Developers**: Building custom executors for specialized environments (cloud, embedded, distributed)
- **Framework Integrators**: Creating domain-specific OS abstractions using airssys-osl primitives
- **Library Authors**: Building reusable components on top of airssys-osl foundation

## Product Scope

### In Scope
1. **#[executor] Macro** (Phase 1 - Priority 1):
   - Attribute macro for custom executor implementations
   - Generates `OSExecutor<O>` trait implementations from method names
   - Method naming convention: `file_read`, `process_spawn`, `tcp_connect`, etc.
   - Automatic operation type detection and mapping

2. **#[operation] Macro** (Phase 2 - Future):
   - Derive macro for custom operation types
   - Generates `Operation` trait implementation
   - Auto-implements required methods (operation_id, operation_type, etc.)

3. **#[middleware] Macro** (Phase 3 - Future, Maybe):
   - Attribute macro for custom middleware implementations
   - Generates `Middleware<O>` trait implementation with wrapper executor pattern
   - Optional: May be deferred or removed if manual implementation suffices

### Out of Scope
- Runtime dispatch or dynamic code generation
- Framework-level abstractions (handled by airssys-osl)
- Business logic implementation (macros only generate trait impls)
- Cross-crate macro expansion (macros work within single compilation unit)

## Technical Foundation

### Technology Stack
- **Language**: Rust (proc-macro)
- **Proc-Macro Framework**: syn v2.0 (parsing), quote v1.0 (code generation), proc-macro2 v1.0
- **Testing**: trybuild for compile-time macro expansion tests
- **Documentation**: Comprehensive rustdoc with macro expansion examples

### Architecture Principles
1. **Token-Based Generation**: Macros work with token streams, reference types as paths (no imports)
2. **Compile-Time Only**: Zero runtime dependencies, pure code generation
3. **Idiomatic Rust**: Generated code follows Rust idioms and workspace standards
4. **Error-First**: Clear, actionable error messages for invalid macro usage

## Integration Points

### airssys-osl Integration
- **Dependency Flow**: airssys-osl → airssys-osl-macros (one-way, feature-gated)
- **Feature Flag**: `macros` feature in airssys-osl (default enabled)
- **Re-exports**: Macros re-exported from airssys-osl::prelude when feature enabled
- **Type References**: Macros reference airssys-osl types via full paths (e.g., `airssys_osl::operations::filesystem::FileReadOperation`)

### Developer Experience
```rust
// Without macros (manual implementation - verbose)
#[async_trait]
impl OSExecutor<FileReadOperation> for MyExecutor {
    async fn execute(&self, op: FileReadOperation, ctx: &ExecutionContext) 
        -> OSResult<ExecutionResult> {
        // implementation
    }
}

// With macros (ergonomic - concise)
#[executor]
impl MyExecutor {
    async fn file_read(&self, op: FileReadOperation, ctx: &ExecutionContext) 
        -> OSResult<ExecutionResult> {
        // same implementation
    }
}
```

## Success Metrics

### Development Metrics
- **Code Reduction**: ~85% less boilerplate for custom executors
- **Compilation Time**: Macro expansion adds <2% to build time
- **Error Quality**: Clear, actionable error messages for all invalid usage patterns
- **Documentation Coverage**: 100% rustdoc coverage with expansion examples

### Adoption Metrics
- **Usage Patterns**: Preferred method for custom executor creation
- **Community Feedback**: Positive reception for ergonomics and clarity
- **Integration Success**: Seamless integration with airssys-osl features

## Constraints and Limitations

### Technical Constraints
1. **Proc-Macro Architecture**: Separate crate required (proc-macro compilation target)
2. **Token Stream Limitations**: No access to type information during expansion
3. **Naming Conventions**: Strict method naming required for operation detection
4. **Single Compilation Unit**: Macros expand within single crate boundary

### Design Constraints
1. **No Runtime Magic**: Pure compile-time code generation only
2. **No Dynamic Dispatch**: Generated code uses static dispatch (generic constraints)
3. **Explicit Over Implicit**: Clear method naming over convention-based inference
4. **Fail-Fast**: Compilation errors for invalid usage, no runtime surprises

### Implementation Constraints
1. **Workspace Standards**: Full compliance with §2.1, §3.2, §4.3, §5.1, §6.1, §6.2
2. **Microsoft Rust Guidelines**: Complete adherence to M-* patterns
3. **YAGNI Principles**: Build only what's explicitly needed, avoid speculation
4. **Zero Dependencies**: Minimal dependencies (syn, quote, proc-macro2 only)

## Roadmap Overview

### Phase 1: Foundation Setup (Current - 1 day)
- ✅ Memory bank structure creation
- ⏳ Cargo workspace member setup
- ⏳ Basic crate structure (lib.rs, Cargo.toml)
- ⏳ Development task planning

### Phase 2: #[executor] Macro (1-2 weeks)
- Core macro implementation (parsing, generation)
- Operation name mapping system
- Error handling and diagnostics
- Comprehensive testing (unit + trybuild)
- Documentation with examples

### Phase 3: #[operation] Macro (Future - TBD)
- Derive macro for custom operations
- Operation trait implementation generation
- Permission inference and validation
- Testing and documentation

### Phase 4: Advanced Features (Future - Maybe)
- #[middleware] macro (if needed)
- Macro composition patterns
- Advanced error handling
- Performance optimizations

## Documentation Standards

### Required Documentation
1. **Macro Usage Guide**: Comprehensive examples for each macro
2. **Expansion Examples**: Show generated code for clarity
3. **Error Reference**: Document all macro error messages
4. **Migration Guide**: Converting manual implementations to macro usage
5. **Best Practices**: Naming conventions, patterns, anti-patterns

### Documentation Quality
- Professional tone (no hyperbole, no excessive emoticons)
- Factual and sourced (reference airssys-osl types and traits)
- Clear implementation status (foundation, in-progress, complete)
- Code examples must be real and tested

## Dependencies and Relationships

### Upstream Dependencies
- **airssys-osl**: Core abstractions (Operation, OSExecutor, Middleware traits)
- **syn**: Proc-macro parsing library
- **quote**: Code generation library
- **proc-macro2**: Proc-macro utilities

### Downstream Consumers
- **airssys-osl**: Re-exports macros via feature flag
- **Application Code**: Uses macros for custom executor creation
- **Integration Projects**: airssys-rt, airssys-wasm (future)

### Related Projects
- **airssys-osl**: Low-level library providing core abstractions
- **airssys-rt**: Runtime system (future consumer)
- **airssys-wasm**: WASM system (future consumer)
