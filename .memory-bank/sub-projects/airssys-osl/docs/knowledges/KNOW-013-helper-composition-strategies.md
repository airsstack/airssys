# KNOW-013: Helper Function Composition Strategies

**Knowledge ID:** KNOW-013  
**Category:** Patterns  
**Maturity:** Draft  
**Created:** 2025-10-11  
**Last Updated:** 2025-10-11  
**Related Tasks:** OSL-TASK-010, OSL-TASK-011 (future)  
**Related ADRs:** ADR-029  
**Dependencies:** ExecutorExt pattern (OSL-TASK-009)

---

## Overview

This document captures the technical analysis and design considerations for implementing functional composition patterns in airssys-osl helper functions. Two primary approaches were analyzed: trait-based composition and pipeline macro composition.

## Context

### Current State (OSL-TASK-010)

Helper functions in `src/helpers.rs` currently use simple function signatures with direct executor calls:

```rust
// Current approach - simple but not composable
pub async fn read_file<P: AsRef<Path>>(path: P, user: impl Into<String>) -> OSResult<Vec<u8>> {
    // TODO(OSL-TASK-003): Add security validation here
    // TODO(OSL-TASK-004): Wire through middleware pipeline
    let executor = FilesystemExecutor::new();  // Direct - bypasses middleware!
    let result = executor.execute(operation, &context).await?;
    Ok(result.output)
}
```

### Problem Statement

**Identified Limitations:**
1. ❌ **Not composable** - Can't easily chain middlewares
2. ❌ **Fixed pattern** - Each function hardcodes the middleware application
3. ❌ **Not functional** - Imperative style, not declarative
4. ❌ **Limited flexibility** - Hard to build custom pipelines
5. ❌ **No reusability** - Can't create reusable pipeline configurations

### Design Goals

1. **Functional composition** - Enable declarative pipeline building
2. **Type safety** - Maintain compile-time guarantees
3. **Ergonomic API** - Simple for common cases, powerful for advanced use
4. **Performance** - Zero-cost abstractions
5. **Backward compatibility** - Don't break existing simple helper usage

---

## Approach 1: Trait-Based Composition

### Architecture

**Core Design Pattern:**
```rust
pub trait HelperPipeline<O: Operation> {
    type Executor: OSExecutor<O>;
    
    fn with_security(self, middleware: SecurityMiddleware) -> ComposedHelper<...>;
    fn with_logger(self, middleware: LoggerMiddleware) -> ComposedHelper<...>;
    fn with_middleware<M>(self, middleware: M) -> ComposedHelper<...>;
}

pub struct ComposedHelper<O, E> 
where 
    O: Operation,
    E: OSExecutor<O>,
{
    executor: E,
    _phantom: PhantomData<O>,
}
```

### Type System Compatibility

**Verified Compatibility with Existing Types:**

| Component | Current Type | Trait Requirement | Compatible |
|-----------|--------------|-------------------|------------|
| Operation | `Operation: Debug + Send + Sync + Clone + 'static` | Same | ✅ Yes |
| OSExecutor | `OSExecutor<O>: Send + Sync + Debug` | Same | ✅ Yes |
| Middleware | `Middleware<O>: Send + Sync + Debug` | Same | ✅ Yes |
| ExecutorExt | `trait ExecutorExt: Sized` | Reused directly | ✅ Yes |
| MiddlewareExecutor | `MiddlewareExecutor<E, M, O>` | Chained via composition | ✅ Yes |

**Conclusion:** Fully compatible - no type system changes required.

### Usage Examples

#### Simple Usage (No Middleware)
```rust
use airssys_osl::helpers::composition::*;

let reader = FileHelper::new();
let data = reader.read("/etc/hosts", "admin").await?;
```

#### With Security Middleware
```rust
let security = SecurityMiddleware::builder()
    .with_acl_policy(my_acl)
    .build();

let reader = FileHelper::new()
    .with_security(security);

let data = reader.read("/etc/hosts", "admin").await?;
```

#### Full Composition Chain
```rust
let helper = FileHelper::new()
    .with_security(SecurityMiddleware::default())
    .with_logger(LoggerMiddleware::default())
    .with_middleware(CustomMiddleware::new());

// Reuse the same pipeline
let data1 = helper.read("/file1.txt", "admin").await?;
let data2 = helper.read("/file2.txt", "admin").await?;
```

### Advantages ✅

1. **Zero Macro Complexity**
   - Pure Rust traits - no proc-macro magic required
   - Easy to debug (no token expansion)
   - IDE autocomplete works perfectly
   - Compiler errors are clear and actionable

2. **Type Safety Excellence**
   - Compile-time type checking
   - Associated types prevent misuse
   - Generic constraints enforce correctness
   - No runtime type erasure needed

3. **Reusable Pipelines**
   - Build once, use many times
   - Immutable transformations
   - No hidden state
   - Clear ownership semantics

4. **Microsoft Rust Guidelines Compliant**
   - **M-SIMPLE-ABSTRACTIONS**: Single level of abstraction
   - **M-DI-HIERARCHY**: Concrete types > Generics (no dyn)
   - **M-DESIGN-FOR-AI**: Idiomatic, easy to understand
   - **M-ESSENTIAL-FN-INHERENT**: Core methods are inherent

5. **Minimal Implementation Cost**
   - ~200 lines of code for full implementation
   - Leverages existing ExecutorExt infrastructure
   - Can be implemented in 1 day
   - No new dependencies required

6. **Excellent Debugging**
   - Standard Rust debugging works
   - Stack traces are clear
   - Type errors point to actual problem
   - `dbg!()` and breakpoints work normally

### Disadvantages ❌

1. **Longer Type Signatures**
   ```rust
   // Type gets complex with chains
   ComposedHelper<FileReadOperation, 
       MiddlewareExecutor<
           MiddlewareExecutor<FilesystemExecutor, SecurityMiddleware, FileReadOperation>,
           LoggerMiddleware,
           FileReadOperation
       >>
   ```
   - **Mitigation**: Type aliases, `impl Trait` hiding

2. **Learning Curve**
   - Users need to understand trait composition
   - Not as immediately obvious as simple functions
   - **Mitigation**: Good documentation + examples

3. **Verbose for Simple Cases**
   ```rust
   // Trait approach (3 lines)
   let reader = FileHelper::new();
   let data = reader.read("/file", "user").await?;
   
   // vs. Simple function (1 line)
   let data = read_file("/file", "user").await?;
   ```
   - **Mitigation**: Keep both APIs (hybrid approach)

### Implementation Estimate

- **Effort**: 4-6 hours (1 day)
- **Lines of Code**: ~200 lines
- **Files Created**: 1 (`src/helpers/composition.rs`)
- **Tests Required**: ~15 integration tests
- **Documentation**: Module docs + examples

---

## Approach 2: Pipeline Macro Composition

### Architecture

**Proposed Syntax:**
```rust
let read_with_security = compose_helper! {
    FileReadOperation
    |> FilesystemExecutor::new()
    |> SecurityMiddleware::default()
    |> LoggerMiddleware::default()
};

// Use like a normal function
let data = read_with_security("/etc/hosts", "admin").await?;
```

### Macro Expansion Strategy

**Conceptual Expansion:**
```rust
// Input
compose_helper! {
    FileReadOperation
    |> FilesystemExecutor::new()
    |> SecurityMiddleware::default()
}

// Expands to
{
    let __executor = FilesystemExecutor::new()
        .with_middleware(SecurityMiddleware::default());
    
    move |path: impl AsRef<Path>, user: impl Into<String>| {
        async move {
            let operation = FileReadOperation::new(path.as_ref().display().to_string());
            let context = ExecutionContext::new(SecurityContext::new(user.into()));
            let result = __executor.execute(operation, &context).await?;
            Ok(result.output)
        }
    }
}
```

### Usage Examples

#### Basic Composition
```rust
let helper = compose_helper! {
    FileReadOperation
    |> FilesystemExecutor::new()
    |> SecurityMiddleware::default()
};

let data = helper("/etc/hosts", "admin").await?;
```

#### Advanced Composition
```rust
let secure_file_ops = compose_helper! {
    FileReadOperation
    |> FilesystemExecutor::new()
    |> SecurityMiddleware::builder()
        .with_acl_policy(my_acl)
        .build()
    |> LoggerMiddleware::new()
    |> CustomMiddleware::new()
};

// Reuse
let data1 = secure_file_ops("/file1", "user").await?;
let data2 = secure_file_ops("/file2", "user").await?;
```

### Advantages ✅

1. **Beautiful Syntax**
   - Reads like a data pipeline
   - Clear data flow visualization
   - Minimal ceremony
   - Functional programming aesthetic

2. **Type Inference Magic**
   - Macro infers types from expressions
   - No need to write complex generic bounds
   - Clean output types (hidden complexity)

3. **Concise**
   - Fewer lines than trait approach
   - No trait bounds to write
   - No type annotations needed

4. **Familiar to FP Developers**
   - Similar to Elixir's `|>` operator
   - Similar to F#'s `|>` operator
   - Similar to Haskell's composition

5. **Reusable Compositions**
   - Build once, use many times
   - Closures are first-class values
   - Can pass to other functions

6. **Zero Runtime Cost**
   - All expansion at compile time
   - No vtables, no dynamic dispatch
   - Same performance as hand-written code

### Disadvantages ❌

1. **Macro Complexity**
   - Requires proc-macro infrastructure
   - Token parsing can be error-prone
   - Need to handle all edge cases
   - **Estimated effort**: 2-3 days for robust implementation

2. **Error Messages**
   ```rust
   error: the trait bound `WrongExecutor: OSExecutor<FileReadOperation>` is not satisfied
     --> src/main.rs:10:15
      |
   10 |     let bad = compose_helper! {
      |               ^^^^^^^^^^^^^^^^ the trait `OSExecutor<FileReadOperation>` 
      |                                is not implemented for `WrongExecutor`
   ```
   - Hard to debug: Error location may be unclear
   - Cognitive load: Users need to understand macro expansion

3. **IDE Support Limitations**
   - rust-analyzer may not expand macros correctly
   - Autocomplete limited inside macro invocations
   - Go-to-definition may not work through macro
   - **Mitigation**: Use `cargo expand` for debugging

4. **Debugging Difficulty**
   - Can't step through macro code in debugger
   - Harder to understand for newcomers
   - Need `cargo expand` to see expansion
   - Stack traces may be confusing

5. **Syntax Rigidity**
   - Custom `|>` syntax must be parsed correctly
   - Can't easily extend or modify
   - Breaking changes harder to manage
   - Conditional middleware is difficult:
     ```rust
     // This won't work!
     compose_helper! {
         FileReadOperation
         |> FilesystemExecutor::new()
         |> if cfg!(debug) { SecurityMiddleware::default() }
     }
     ```

6. **Macro Hygiene Concerns**
   - Need to use `__executor` prefix to avoid collisions
   - Must import traits in expansion
   - Harder to maintain than regular code

7. **Microsoft Guidelines Concerns**
   - **M-SIMPLE-ABSTRACTIONS**: Macros add cognitive complexity
   - **M-DESIGN-FOR-AI**: AI models struggle with custom syntax
   - May violate "keep it simple" principle

8. **Testing Complexity**
   ```rust
   // Requires UI testing framework
   #[test]
   fn test_compose_helper() {
       let t = trybuild::TestCases::new();
       t.pass("tests/ui/compose_helper_pass.rs");
       t.compile_fail("tests/ui/compose_helper_fail.rs");
   }
   ```
   - Requires additional testing infrastructure
   - Slower test cycles
   - More maintenance burden

### Implementation Estimate

- **Effort**: 16-24 hours (2-3 days)
- **Lines of Code**: ~400 lines (macro + tests)
- **Files Created**: 2 (`airssys-osl-macros/src/compose.rs` + tests)
- **Dependencies**: `syn`, `quote`, `proc-macro2` (already available)
- **Tests Required**: ~20 UI tests + integration tests
- **Documentation**: Macro docs + expansion examples

---

## Comparison Matrix

| Aspect | Trait-Based | Pipeline Macro | Winner |
|--------|-------------|----------------|--------|
| **Syntax Beauty** | Good | Excellent | Macro |
| **Type Safety** | Excellent | Good | Trait |
| **IDE Support** | Excellent | Fair | Trait |
| **Error Messages** | Clear | Can be cryptic | Trait |
| **Debugging** | Easy | Hard | Trait |
| **Learning Curve** | Moderate | Steep | Trait |
| **Code Reuse** | Excellent | Excellent | Tie |
| **Performance** | Excellent | Excellent | Tie |
| **Extensibility** | Easy | Rigid | Trait |
| **Testing** | Easy | Complex | Trait |
| **Maintenance** | Easy | Moderate | Trait |
| **FP Style** | Good | Excellent | Macro |
| **Implementation Time** | 1 day | 2-3 days | Trait |
| **Microsoft Guidelines** | ✅ Compliant | ⚠️ Borderline | Trait |
| **AI-Friendliness** | ✅ High | ⚠️ Medium | Trait |

---

## Recommended Strategy

### Phase 1: Trait-Based Composition (Recommended First)

**Rationale:**
1. ✅ Fully compatible with existing type system
2. ✅ Better IDE support, debugging, error messages
3. ✅ Easier to maintain and extend
4. ✅ Microsoft Rust Guidelines compliant
5. ✅ Lower complexity for users and maintainers
6. ✅ Can implement in 1 day vs. 2-3 days for macro
7. ✅ Provides immediate value

**Implementation:**
- Create `airssys-osl/src/helpers/composition.rs`
- Define `HelperPipeline` trait + `ComposedHelper` struct
- Implement for `FileHelper`, `ProcessHelper`, `NetworkHelper`
- Add comprehensive examples and documentation
- Keep existing `helpers.rs` functions for backward compatibility

**Result: Hybrid API**
```rust
// Simple API (existing) - for basic use cases
let data = read_file("/file", "user").await?;

// Composition API (new) - for advanced use cases
let helper = FileHelper::new()
    .with_security(SecurityMiddleware::default())
    .with_logger(LoggerMiddleware::default());
let data = helper.read("/file", "user").await?;
```

### Phase 2: Pipeline Macro (Optional Enhancement)

**When to Consider:**
- After trait-based composition proves successful
- If users demand more concise syntax
- As syntactic sugar on top of trait system
- When we have time for polish and UX refinement

**Implementation Path:**
1. Prove trait-based composition works in production
2. Gather user feedback on API ergonomics
3. Implement macro as thin layer over trait infrastructure
4. Make it optional (both APIs remain available)
5. Document macro expansion patterns

**Result: Triple API Choice**
```rust
// Level 1: Simple functions (most users)
let data = read_file("/file", "user").await?;

// Level 2: Trait composition (power users)
let helper = FileHelper::new().with_security(...);
let data = helper.read("/file", "user").await?;

// Level 3: Pipeline macro (FP enthusiasts)
let helper = compose_helper! { FileReadOperation |> ... };
let data = helper("/file", "user").await?;
```

---

## Design Principles

### Hybrid Approach Benefits

1. **Progressive Disclosure**
   - Simple API for simple cases
   - Advanced API for complex needs
   - Expert API for FP style

2. **Backward Compatibility**
   - Existing code continues to work
   - Migration path is opt-in
   - No breaking changes

3. **Risk Mitigation**
   - Start with proven patterns (traits)
   - Add experimental features later (macros)
   - Can deprecate if unsuccessful

4. **User Choice**
   - Different users prefer different styles
   - All approaches are valid
   - Framework doesn't impose opinions

### Microsoft Rust Guidelines Alignment

| Guideline | Trait Approach | Macro Approach | Alignment |
|-----------|----------------|----------------|-----------|
| M-SIMPLE-ABSTRACTIONS | ✅ Single level | ⚠️ Multiple levels | Trait wins |
| M-DI-HIERARCHY | ✅ Concrete > Generic | ✅ Same | Both good |
| M-DESIGN-FOR-AI | ✅ Idiomatic | ⚠️ Custom syntax | Trait wins |
| M-ESSENTIAL-FN-INHERENT | ✅ Inherent methods | ✅ Same | Both good |
| M-AVOID-WRAPPERS | ✅ Minimal wrappers | ✅ Zero-cost | Both good |

---

## Implementation Checklist

### Trait-Based Composition (Phase 1)

- [ ] Create `src/helpers/composition.rs` module
- [ ] Define `HelperPipeline` trait
- [ ] Implement `ComposedHelper<O, E>` struct
- [ ] Create `FileHelper`, `ProcessHelper`, `NetworkHelper` builders
- [ ] Add execution methods for each operation type
- [ ] Write comprehensive rustdoc with examples
- [ ] Add 15+ integration tests
- [ ] Update `lib.rs` and `prelude.rs` exports
- [ ] Create example: `examples/helper_composition.rs`
- [ ] Update README with composition patterns

### Pipeline Macro (Phase 2 - Future)

- [ ] Evaluate user feedback from trait-based approach
- [ ] Create `airssys-osl-macros/src/compose.rs`
- [ ] Implement `compose_helper!` proc-macro
- [ ] Write AST parser for `|>` syntax
- [ ] Add macro expansion tests (trybuild)
- [ ] Write comprehensive macro documentation
- [ ] Add cargo-expand examples
- [ ] Create debugging guide for macro users
- [ ] Benchmark macro vs. trait performance
- [ ] Update documentation with all three API styles

---

## Custom Middleware Extensibility

### Overview

**All composition approaches fully support custom middleware.** Engineers can create their own middleware implementations and use them seamlessly with all three API levels (simple functions, ExecutorExt, trait composition).

This is a **core design feature**, not an afterthought. The middleware system is designed to be extensible, composable, and type-safe.

### Creating Custom Middleware

#### Step 1: Define Your Middleware Struct

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct RateLimitMiddleware {
    max_operations_per_second: u32,
    current_count: Arc<Mutex<u32>>,
    last_reset: Arc<Mutex<Instant>>,
}

impl RateLimitMiddleware {
    pub fn new(max_ops: u32) -> Self {
        Self {
            max_operations_per_second: max_ops,
            current_count: Arc::new(Mutex::new(0)),
            last_reset: Arc::new(Mutex::new(Instant::now())),
        }
    }
}
```

#### Step 2: Implement the Middleware Trait

```rust
use async_trait::async_trait;
use airssys_osl::core::middleware::{Middleware, MiddlewareResult, MiddlewareError, ErrorAction};
use airssys_osl::core::context::ExecutionContext;
use airssys_osl::core::operation::Operation;
use airssys_osl::core::executor::ExecutionResult;
use airssys_osl::core::result::OSResult;

#[async_trait]
impl<O: Operation> Middleware<O> for RateLimitMiddleware {
    fn name(&self) -> &str {
        "rate_limit_middleware"
    }

    fn priority(&self) -> u32 {
        // Run after security (100) but before logger (50)
        75
    }

    async fn can_process(&self, _operation: &O, _context: &ExecutionContext) -> bool {
        true // Apply to all operations
    }

    async fn before_execution(
        &self,
        operation: O,
        _context: &ExecutionContext,
    ) -> MiddlewareResult<Option<O>> {
        // Check and enforce rate limit
        let mut count = self.current_count.lock().await;
        let mut last_reset = self.last_reset.lock().await;

        // Reset counter if 1 second has passed
        if last_reset.elapsed() >= Duration::from_secs(1) {
            *count = 0;
            *last_reset = Instant::now();
        }

        // Check if we've exceeded the limit
        if *count >= self.max_operations_per_second {
            return Err(MiddlewareError::Fatal(
                format!("Rate limit exceeded: {} ops/sec", self.max_operations_per_second)
            ));
        }

        // Increment counter
        *count += 1;

        // Allow operation to proceed
        Ok(Some(operation))
    }

    async fn after_execution(
        &self,
        _context: &ExecutionContext,
        _result: &OSResult<ExecutionResult>,
    ) -> MiddlewareResult<()> {
        Ok(())
    }

    async fn handle_error(
        &self,
        _error: crate::core::result::OSError,
        _context: &ExecutionContext,
    ) -> ErrorAction {
        ErrorAction::Continue
    }
}
```

### Using Custom Middleware

#### Level 1: With Simple Helper Functions

```rust
use airssys_osl::helpers::*;
use my_middleware::RateLimitMiddleware;

// Create custom middleware
let rate_limiter = RateLimitMiddleware::new(100); // 100 ops/sec

// Use with *_with_middleware variant
let data = read_file_with_middleware(
    "/etc/hosts",
    "admin",
    rate_limiter
).await?;
```

#### Level 2: With ExecutorExt Manual Composition

```rust
use airssys_osl::middleware::ext::ExecutorExt;
use my_middleware::RateLimitMiddleware;

// Build custom middleware stack
let executor = FilesystemExecutor::new()
    .with_middleware(SecurityMiddleware::default())
    .with_middleware(RateLimitMiddleware::new(100))
    .with_middleware(LoggerMiddleware::default());

let operation = FileReadOperation::new("/etc/hosts");
let context = ExecutionContext::new(SecurityContext::new("admin".to_string()));
let result = executor.execute(operation, &context).await?;
```

#### Level 3: With Trait Composition

```rust
use airssys_osl::helpers::composition::*;
use my_middleware::{RateLimitMiddleware, MetricsMiddleware};

// Build pipeline with custom middleware
let helper = FileHelper::new()
    .with_security(SecurityMiddleware::default())
    .with_middleware(RateLimitMiddleware::new(100))     // ✅ Custom middleware
    .with_logger(LoggerMiddleware::default())
    .with_middleware(MetricsMiddleware::new());         // ✅ Another custom one

// Reuse pipeline with custom middleware
for file in files {
    let data = helper.read(file, "admin").await?;
    process(data);
}
```

### Real-World Custom Middleware Examples

#### Example 1: Caching Middleware

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct CachingMiddleware {
    cache: Arc<Mutex<HashMap<String, (Vec<u8>, Instant)>>>,
    ttl: Duration,
}

impl CachingMiddleware {
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            ttl,
        }
    }
    
    async fn get_cached(&self, key: &str) -> Option<Vec<u8>> {
        let cache = self.cache.lock().await;
        cache.get(key).and_then(|(data, timestamp)| {
            if timestamp.elapsed() < self.ttl {
                Some(data.clone())
            } else {
                None
            }
        })
    }
    
    async fn set_cached(&self, key: String, data: Vec<u8>) {
        let mut cache = self.cache.lock().await;
        cache.insert(key, (data, Instant::now()));
    }
}

#[async_trait]
impl Middleware<FileReadOperation> for CachingMiddleware {
    fn name(&self) -> &str {
        "caching_middleware"
    }

    fn priority(&self) -> u32 {
        90 // Run before security to avoid unnecessary validation
    }

    async fn before_execution(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> MiddlewareResult<Option<FileReadOperation>> {
        // Check cache
        if let Some(cached_data) = self.get_cached(&operation.path).await {
            // Store cached data in context for retrieval
            // Note: Would need context enhancement to support this
            // For now, this is conceptual
        }
        
        // Proceed with operation if not cached
        Ok(Some(operation))
    }
    
    async fn after_execution(
        &self,
        _context: &ExecutionContext,
        result: &OSResult<ExecutionResult>,
    ) -> MiddlewareResult<()> {
        // Cache successful results
        if let Ok(exec_result) = result {
            // Extract path from operation and cache result
            // (implementation details depend on context access)
        }
        Ok(())
    }

    async fn handle_error(
        &self,
        _error: crate::core::result::OSError,
        _context: &ExecutionContext,
    ) -> ErrorAction {
        ErrorAction::Continue
    }
}

// Usage
let helper = FileHelper::new()
    .with_middleware(CachingMiddleware::new(Duration::from_secs(60)))
    .with_security(SecurityMiddleware::default());
```

#### Example 2: Metrics Collection Middleware

```rust
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone)]
pub struct MetricsMiddleware {
    total_operations: Arc<AtomicU64>,
    successful_operations: Arc<AtomicU64>,
    failed_operations: Arc<AtomicU64>,
    total_duration_ms: Arc<AtomicU64>,
}

impl MetricsMiddleware {
    pub fn new() -> Self {
        Self {
            total_operations: Arc::new(AtomicU64::new(0)),
            successful_operations: Arc::new(AtomicU64::new(0)),
            failed_operations: Arc::new(AtomicU64::new(0)),
            total_duration_ms: Arc::new(AtomicU64::new(0)),
        }
    }
    
    pub fn get_metrics(&self) -> OperationMetrics {
        OperationMetrics {
            total: self.total_operations.load(Ordering::SeqCst),
            successful: self.successful_operations.load(Ordering::SeqCst),
            failed: self.failed_operations.load(Ordering::SeqCst),
            avg_duration_ms: self.total_duration_ms.load(Ordering::SeqCst) 
                / self.total_operations.load(Ordering::SeqCst).max(1),
        }
    }
}

#[async_trait]
impl<O: Operation> Middleware<O> for MetricsMiddleware {
    fn name(&self) -> &str {
        "metrics_middleware"
    }

    fn priority(&self) -> u32 {
        10 // Run last to capture everything
    }

    async fn before_execution(
        &self,
        operation: O,
        _context: &ExecutionContext,
    ) -> MiddlewareResult<Option<O>> {
        self.total_operations.fetch_add(1, Ordering::SeqCst);
        Ok(Some(operation))
    }
    
    async fn after_execution(
        &self,
        _context: &ExecutionContext,
        result: &OSResult<ExecutionResult>,
    ) -> MiddlewareResult<()> {
        match result {
            Ok(exec_result) => {
                self.successful_operations.fetch_add(1, Ordering::SeqCst);
                self.total_duration_ms.fetch_add(
                    exec_result.duration.as_millis() as u64, 
                    Ordering::SeqCst
                );
            }
            Err(_) => {
                self.failed_operations.fetch_add(1, Ordering::SeqCst);
            }
        }
        Ok(())
    }

    async fn handle_error(
        &self,
        _error: crate::core::result::OSError,
        _context: &ExecutionContext,
    ) -> ErrorAction {
        ErrorAction::Continue
    }
}

// Usage
let metrics = MetricsMiddleware::new();
let helper = FileHelper::new()
    .with_middleware(metrics.clone())
    .with_security(SecurityMiddleware::default());

// Later, query metrics
let stats = metrics.get_metrics();
println!("Success rate: {}/{}", stats.successful, stats.total);
```

#### Example 3: Retry Middleware

```rust
#[derive(Debug, Clone)]
pub struct RetryMiddleware {
    max_retries: u32,
    initial_backoff: Duration,
    max_backoff: Duration,
}

impl RetryMiddleware {
    pub fn new(max_retries: u32, initial_backoff: Duration) -> Self {
        Self {
            max_retries,
            initial_backoff,
            max_backoff: initial_backoff * 10,
        }
    }
    
    fn calculate_backoff(&self, attempt: u32) -> Duration {
        let backoff = self.initial_backoff * 2_u32.pow(attempt);
        std::cmp::min(backoff, self.max_backoff)
    }
}

#[async_trait]
impl<O: Operation + Clone> Middleware<O> for RetryMiddleware {
    fn name(&self) -> &str {
        "retry_middleware"
    }

    fn priority(&self) -> u32 {
        20 // Run near the end, but before metrics
    }

    async fn before_execution(
        &self,
        operation: O,
        _context: &ExecutionContext,
    ) -> MiddlewareResult<Option<O>> {
        Ok(Some(operation))
    }
    
    async fn after_execution(
        &self,
        _context: &ExecutionContext,
        _result: &OSResult<ExecutionResult>,
    ) -> MiddlewareResult<()> {
        Ok(())
    }

    async fn handle_error(
        &self,
        error: crate::core::result::OSError,
        context: &ExecutionContext,
    ) -> ErrorAction {
        // Check if error is retryable (network errors, temporary failures, etc.)
        if !error.is_retryable() {
            return ErrorAction::Stop;
        }
        
        // Get retry count from context metadata
        let retry_count = context.metadata()
            .get("retry_count")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        
        if retry_count < self.max_retries {
            // Calculate backoff and wait
            let backoff = self.calculate_backoff(retry_count);
            tokio::time::sleep(backoff).await;
            
            // Return retry action (would need context mutation support)
            ErrorAction::Retry
        } else {
            ErrorAction::Stop
        }
    }
}

// Usage
let helper = FileHelper::new()
    .with_middleware(RetryMiddleware::new(3, Duration::from_millis(100)))
    .with_security(SecurityMiddleware::default());
```

### Custom Middleware Capabilities

| Capability | Support | Details |
|------------|---------|---------|
| **Create custom middleware** | ✅ Full | Implement `Middleware<O>` trait |
| **Use with simple functions** | ✅ Full | `*_with_middleware()` variants |
| **Use with ExecutorExt** | ✅ Full | `.with_middleware()` chaining |
| **Use with trait composition** | ✅ Full | `.with_middleware()` in pipeline |
| **Combine multiple custom** | ✅ Full | Chain unlimited middleware |
| **Mix custom + built-in** | ✅ Full | Any order in composition |
| **Type safety** | ✅ Compile-time | Generic constraints enforce correctness |
| **Priority control** | ✅ Full | Set `priority()` to control execution order |
| **Generic over operations** | ✅ Full | `impl<O: Operation> Middleware<O>` |
| **Operation-specific** | ✅ Full | `impl Middleware<FileReadOperation>` |
| **Stateful middleware** | ✅ Full | Use `Arc<Mutex<State>>` pattern |
| **Async operations** | ✅ Full | All methods are async |
| **Error handling** | ✅ Full | `handle_error()` with retry/stop/continue |
| **Context access** | ✅ Full | `ExecutionContext` in all hooks |
| **Result inspection** | ✅ Full | Access results in `after_execution()` |

### Design Guidelines for Custom Middleware

1. **Single Responsibility**
   - Each middleware should do one thing well
   - Don't combine rate limiting, caching, and metrics in one middleware
   - Compose multiple simple middleware instead

2. **Priority Planning**
   - Security: 100 (runs first)
   - Rate limiting: 75
   - Logger: 50
   - Retry: 20
   - Metrics: 10 (runs last)
   - Plan your priority based on when you need to run

3. **Error Handling**
   - Only return `ErrorAction::Stop` for unrecoverable errors
   - Use `ErrorAction::Retry` for transient failures
   - Use `ErrorAction::Continue` to log but not block

4. **State Management**
   - Use `Arc<Mutex<T>>` for mutable state
   - Use `Arc<AtomicU64>` for counters
   - Clone the middleware when needed (Arc makes this cheap)

5. **Type Generics**
   - `impl<O: Operation> Middleware<O>` for all operations
   - `impl Middleware<FileReadOperation>` for specific operations
   - Choose based on your middleware's scope

### Enhanced Trait Design

The `HelperPipeline` trait explicitly supports custom middleware:

```rust
pub trait HelperPipeline<O: Operation> {
    type Executor: OSExecutor<O>;

    /// Add security middleware to the pipeline.
    fn with_security(self, middleware: SecurityMiddleware) 
        -> ComposedHelper<O, MiddlewareExecutor<Self::Executor, SecurityMiddleware, O>>;

    /// Add logger middleware to the pipeline.
    fn with_logger(self, middleware: LoggerMiddleware) 
        -> ComposedHelper<O, MiddlewareExecutor<Self::Executor, LoggerMiddleware, O>>;

    /// Add ANY custom middleware to the pipeline.
    /// 
    /// This is the generic method that accepts any type implementing Middleware<O>.
    /// Engineers can create their own middleware and use this method to add it
    /// to the composition pipeline.
    ///
    /// # Type Parameters
    /// - `M`: Any type implementing `Middleware<O>`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use my_middleware::{RateLimitMiddleware, MetricsMiddleware, CachingMiddleware};
    ///
    /// // Compose multiple custom middleware
    /// let helper = FileHelper::new()
    ///     .with_middleware(RateLimitMiddleware::new(100))
    ///     .with_middleware(MetricsMiddleware::new())
    ///     .with_middleware(CachingMiddleware::new(Duration::from_secs(60)))
    ///     .with_security(SecurityMiddleware::default());
    ///
    /// // Use the composed helper
    /// let data = helper.read("/etc/hosts", "admin").await?;
    /// ```
    fn with_middleware<M>(self, middleware: M) 
        -> ComposedHelper<O, MiddlewareExecutor<Self::Executor, M, O>>
    where
        M: Middleware<O> + Send + Sync + std::fmt::Debug + 'static;

    /// Get the underlying executor.
    fn executor(&self) -> &Self::Executor;
}
```

### Summary

**Custom middleware is a first-class feature**, not an afterthought:

- ✅ **Fully extensible**: Create unlimited custom middleware
- ✅ **Type-safe**: Compile-time guarantees
- ✅ **Composable**: Mix and match freely
- ✅ **Flexible**: Control priority, scope, behavior
- ✅ **Reusable**: Share across projects and teams
- ✅ **Well-integrated**: Works with all API levels

Engineers have complete freedom to build sophisticated middleware pipelines tailored to their specific needs.

---

## Future Considerations

### Potential Enhancements

1. **Async Composition Operators**
   ```rust
   let pipeline = async_compose! {
       read_file |> validate |> transform |> write_file
   };
   ```

2. **Error Handling Composition**
   ```rust
   let helper = FileHelper::new()
       .with_retry_policy(ExponentialBackoff::new())
       .with_fallback(default_value);
   ```

3. **Conditional Middleware**
   ```rust
   let helper = FileHelper::new()
       .with_security_if(cfg!(production), security)
       .with_logger_if(cfg!(debug), logger);
   ```

4. **Pipeline Metrics**
   ```rust
   let helper = FileHelper::new()
       .with_metrics(MetricsCollector::new())
       .with_tracing(TracingMiddleware::new());
   ```

5. **Middleware Marketplace/Registry**
   - Community-contributed middleware
   - Common patterns (rate limiting, caching, retry)
   - Best practices and examples
   - Performance benchmarks

### Related Work

- **Elixir Pipe Operator**: Inspiration for `|>` syntax
- **F# Pipeline**: Functional composition patterns
- **Tower Middleware**: Similar composition in web frameworks
- **Tokio Layers**: Layer-based composition for async

---

## References

- **Microsoft Rust Guidelines**: https://microsoft.github.io/rust-guidelines/
- **Workspace Standards**: `.memory-bank/workspace/shared_patterns.md`
- **ADR-029**: Abandon OSL-TASK-004 and Create OSL-TASK-010
- **OSL-TASK-009**: ExecutorExt Middleware Extension Pattern
- **Tower Middleware**: https://docs.rs/tower/latest/tower/

---

## Change Log

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2025-10-11 | 1.0 | Initial knowledge document created from technical analysis | AI Agent |
| 2025-10-11 | 1.1 | Added custom middleware extensibility section with examples | AI Agent |

---

**Status:** Draft - Pending implementation and validation  
**Next Review:** After trait-based composition implementation (Phase 1)
