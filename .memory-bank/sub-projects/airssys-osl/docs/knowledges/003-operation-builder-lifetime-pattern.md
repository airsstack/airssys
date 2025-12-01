# Knowledge Document: Operation Builder Lifetime Pattern

**ID**: KNOW-003  
**Type**: Architecture Pattern  
**Status**: Active  
**Created**: 2025-10-04  
**Phase**: Phase 1 (OSL-TASK-006)

## Problem Statement

Operation builders (FilesystemBuilder, ProcessBuilder, NetworkBuilder) need access to the OSLFramework's internal components during operation execution, but we want to avoid expensive cloning or awkward API patterns.

## Solution: Lifetime-Parameterized Builders

### Pattern Definition

```rust
pub struct FilesystemBuilder<'a> {
    _framework: &'a OSLFramework,
}

impl<'a> FilesystemBuilder<'a> {
    pub(crate) fn new(framework: &'a OSLFramework) -> Self {
        Self { _framework: framework }
    }
}
```

### Why Lifetime Annotations Are Required

The lifetime parameter `'a` ensures that:
1. The builder cannot outlive the framework it references
2. Rust's borrow checker prevents dangling references at compile-time
3. Zero runtime overhead - no reference counting needed

## Rationale

### Why Builders Need Framework Access

During Phase 3 implementation, the `execute()` method must access:

1. **Middleware Pipeline** - For security checks, logging, validation
2. **Executor Registry** - To dispatch operations to the correct executor
3. **Security Context** - For permission checks and audit trails
4. **Configuration** - For timeouts, retry logic, and policies

### Complete Execution Flow

```rust
// User code (Phase 3)
let result = osl.filesystem()           // Returns FilesystemBuilder<'a>
    .read_file("/etc/passwd")           // Builds operation
    .with_permissions(0o644)            // Adds metadata
    .execute().await?;                  // Needs framework internals!

// What execute() does internally:
impl<'a> FilesystemBuilder<'a> {
    pub async fn execute(self) -> OSResult<ExecutionResult> {
        // Access framework components through the reference
        let pipeline = self._framework.middleware_pipeline();
        let executors = self._framework.executor_registry();
        let context = self._framework.security_context();
        
        // Execute through middleware pipeline
        pipeline.execute(self.operation, context, executors).await
    }
}
```

## Alternative Designs Considered

### ❌ Option 1: Pass Framework to execute()
```rust
// Awkward API - user must pass framework twice
osl.filesystem()
    .read_file("/path")
    .execute(&osl).await?;
```
**Rejected**: Poor developer experience, violates fluent API principles

### ❌ Option 2: Clone Framework
```rust
pub struct FilesystemBuilder {
    framework: OSLFramework,  // Owned copy
}
```
**Rejected**: Expensive cloning, violates YAGNI (§6.1)

### ❌ Option 3: Arc<OSLFramework>
```rust
pub struct FilesystemBuilder {
    framework: Arc<OSLFramework>,  // Shared ownership
}
```
**Rejected**: Unnecessary runtime overhead, complex lifetime management

### ✅ Option 4: Borrowed Reference (Chosen)
```rust
pub struct FilesystemBuilder<'a> {
    _framework: &'a OSLFramework,  // Zero-cost borrow
}
```
**Selected**: Zero-cost, compile-time safety, clean API

## Benefits

1. **Zero Runtime Overhead**: No cloning, no reference counting
2. **Compile-Time Safety**: Borrow checker prevents dangling references
3. **Clean API**: Users don't see implementation details
4. **Flexible**: Builders can access any framework component
5. **Idiomatic Rust**: Standard lifetime pattern for borrowed data

## Constraints

1. Builder lifetime cannot exceed framework lifetime
2. Framework must remain in scope while builder exists
3. Cannot return builder from function that drops framework

### Safe Usage Example

```rust
async fn safe_usage() -> OSResult<()> {
    let osl = OSLFramework::builder().build().await?;
    
    let fs_builder = osl.filesystem();  // fs_builder borrows osl
    
    // ✅ SAFE: Both in scope
    // Phase 3: fs_builder.read_file("/path").execute().await?
    
    Ok(())
} // Both osl and fs_builder dropped together
```

### Unsafe Pattern (Won't Compile)

```rust
async fn broken_usage() -> FilesystemBuilder<'static> {
    let osl = OSLFramework::builder().build().await.unwrap();
    let fs_builder = osl.filesystem();
    
    fs_builder  // ❌ ERROR: Can't return reference to dropped osl
} // osl dropped here!
```

## Implementation Guidelines

### When Creating Builders

1. Always use lifetime parameter: `<'a>`
2. Store framework reference: `&'a OSLFramework`
3. Use `pub(crate)` for constructor visibility
4. Document Phase 3 usage in examples

### When Implementing execute()

Phase 3 will implement execute() by accessing framework components:

```rust
impl<'a> FilesystemBuilder<'a> {
    pub async fn execute(self) -> OSResult<ExecutionResult> {
        let pipeline = self._framework.middleware_pipeline();
        let executors = self._framework.executor_registry();
        let context = ExecutionContext::new(
            self._framework.security_context().clone()
        );
        
        pipeline.execute(self.operation, context, executors).await
    }
}
```

## Related Documentation

- **ADR-026**: Framework as Primary API Strategy (80/20 principle)
- **ADR-027**: Builder Pattern Architecture Implementation
- **KNOW-001**: Core Architecture Foundations
- **OSL-TASK-006**: Core Builder Implementation (Phase 1-4)

## Microsoft Rust Guidelines Compliance

- **M-DI-HIERARCHY**: Using borrowed references (preferred over Arc)
- **M-DESIGN-FOR-AI**: Clear lifetime semantics, well-documented
- **M-AVOID-WRAPPERS**: No smart pointers in public builder API
- **§6.2**: Avoiding dyn patterns in favor of static dispatch with lifetimes

## FAQ

**Q: Why not make builders independent of the framework?**  
A: Builders need middleware pipeline, executor registry, and security context for proper execution. Independence would require bypassing the framework's security and logging infrastructure.

**Q: Can I store a builder in a struct?**  
A: Yes, but the struct must also be lifetime-parameterized:
```rust
struct MyApp<'a> {
    fs: FilesystemBuilder<'a>,
}
```

**Q: What if I need the builder to live longer?**  
A: Use Arc<OSLFramework> for shared ownership if truly needed, but this is rarely necessary in practice.

**Q: Does this pattern work with async?**  
A: Yes, lifetimes work seamlessly with async/await. The borrow checker ensures safety across await points.

## Conclusion

The lifetime-parameterized builder pattern provides:
- Zero-cost abstraction through borrowed references
- Compile-time safety guarantees
- Clean, fluent API for users
- Full access to framework internals for execution

This pattern is fundamental to the OSL framework's architecture and enables the elegant operation builder API while maintaining security, logging, and proper middleware orchestration.
