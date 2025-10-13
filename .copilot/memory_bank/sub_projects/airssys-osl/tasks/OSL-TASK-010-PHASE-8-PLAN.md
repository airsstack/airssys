# OSL-TASK-010 Phase 8 Detailed Implementation Plan
# Trait Composition Infrastructure

**Phase:** 8 of 11  
**Status:** ⏳ Ready to Start  
**Priority:** High  
**Estimated Time:** 3-4 hours  
**Created:** 2025-10-13  
**Dependencies:** Phases 1-7 ✅ Complete

---

## Executive Summary

Phase 8 builds the **Level 3 API** - a trait-based composition system enabling power users to create reusable middleware pipelines using functional programming patterns.

**Key Deliverables:**
- `HelperPipeline<O>` trait for fluent composition API
- `ComposedHelper<O, E>` wrapper enabling method chaining
- Three helper builders: `FileHelper`, `ProcessHelper`, `NetworkHelper`
- Execution methods for all 10 operation types
- Comprehensive rustdoc with usage examples

**Value Proposition:**
- **Build once, use many**: Configure pipeline once, reuse across many operations
- **Zero-cost abstractions**: No runtime overhead vs manual composition
- **Type-safe**: Compile-time middleware compatibility guarantees
- **Functional style**: Chainable `.with_middleware()` methods
- **Production-ready**: Efficient for high-throughput scenarios

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Sub-Phase Breakdown](#sub-phase-breakdown)
   - 8.1: HelperPipeline Trait Design
   - 8.2: ComposedHelper & Builders
   - 8.3: Execution Methods Implementation
3. [File Organization](#file-organization)
4. [API Design Patterns](#api-design-patterns)
5. [Type System Design](#type-system-design)
6. [Implementation Checklist](#implementation-checklist)
7. [Testing Strategy](#testing-strategy)
8. [Success Criteria](#success-criteria)
9. [Risk Mitigation](#risk-mitigation)

---

## Architecture Overview

### Three Core Components

```
┌─────────────────────────────────────────────────────────────┐
│  Level 3 API: Trait-Based Composition                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. HelperPipeline<O> Trait                                 │
│     ├─ with_security(middleware) → ComposedHelper           │
│     ├─ with_logger(middleware) → ComposedHelper             │
│     ├─ with_middleware<M>(middleware) → ComposedHelper      │
│     └─ executor() → &Executor                               │
│                                                              │
│  2. ComposedHelper<O, E> Wrapper                            │
│     ├─ Wraps: Any OSExecutor<O> implementation              │
│     ├─ Implements: HelperPipeline<O> (enables chaining)     │
│     └─ Provides: Execution methods per operation type       │
│                                                              │
│  3. Helper Builders (Entry Points)                          │
│     ├─ FileHelper::new() → ComposedHelper<FileOp, FsExec>  │
│     ├─ ProcessHelper::new() → ComposedHelper<ProcOp, Exec> │
│     └─ NetworkHelper::new() → ComposedHelper<NetOp, Exec>  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
          ↓ Builds on (Existing Infrastructure) ↓
┌─────────────────────────────────────────────────────────────┐
│  ExecutorExt Trait (OSL-TASK-009) ✅                        │
│  ├─ with_middleware<M>() → MiddlewareExecutor<E, M, O>     │
│  └─ Type-safe middleware wrapping                           │
└─────────────────────────────────────────────────────────────┘
```

### Composition Flow Example

```rust
// User writes:
let reader = FileHelper::new()           // ComposedHelper<FileReadOp, FilesystemExecutor>
    .with_security(SecurityMiddleware)    // ComposedHelper<FileReadOp, MiddlewareExecutor<...>>
    .with_middleware(RateLimitMiddleware) // ComposedHelper<FileReadOp, MiddlewareExecutor<...>>
    .with_logger(LoggerMiddleware);       // ComposedHelper<FileReadOp, MiddlewareExecutor<...>>

// Compiler generates nested MiddlewareExecutor types:
// MiddlewareExecutor<
//   MiddlewareExecutor<
//     MiddlewareExecutor<
//       FilesystemExecutor,
//       SecurityMiddleware,
//       FileReadOp>,
//     RateLimitMiddleware,
//     FileReadOp>,
//   LoggerMiddleware,
//   FileReadOp>

// Use many times efficiently:
for file in files {
    let data = reader.read(file, "admin").await?;
}
```

---

## Sub-Phase Breakdown

### Phase 8.1: HelperPipeline Trait Design (1 hour)

#### Objectives
- Define core composition trait
- Establish fluent API pattern
- Create comprehensive module documentation

#### Implementation Details

**File:** `airssys-osl/src/helpers/composition.rs`

**Module Documentation (80-100 lines):**
```rust
//! Trait-based composition for reusable helper pipelines.
//!
//! This module provides a functional programming approach to building
//! reusable middleware pipelines for OS operations.
//!
//! # The Problem
//!
//! When using Level 1 or Level 2 APIs, middleware must be specified
//! for every operation:
//!
//! ```rust,ignore
//! // Repetitive - middleware configured 3 times
//! let data1 = read_file_with_middleware("/file1", "user", middleware.clone()).await?;
//! let data2 = read_file_with_middleware("/file2", "user", middleware.clone()).await?;
//! let data3 = read_file_with_middleware("/file3", "user", middleware.clone()).await?;
//! ```
//!
//! # The Solution: Trait Composition
//!
//! Build the pipeline **once**, reuse it **many times**:
//!
//! ```rust,ignore
//! // Configure once
//! let reader = FileHelper::new()
//!     .with_security(SecurityMiddleware::default())
//!     .with_middleware(RateLimitMiddleware::new(100));
//!
//! // Use many times efficiently
//! let data1 = reader.read("/file1", "user").await?;
//! let data2 = reader.read("/file2", "user").await?;
//! let data3 = reader.read("/file3", "user").await?;
//! ```
//!
//! # Performance
//!
//! - **Zero-cost abstraction**: No runtime overhead vs manual composition
//! - **Compile-time optimization**: Generic types fully monomorphized
//! - **Efficient reuse**: Pipeline setup amortized across many operations
//! - **No allocations**: All composition happens at compile time
//!
//! # Usage Patterns
//!
//! ## Pattern 1: Specialized Helpers for Different Contexts
//!
//! ```rust,ignore
//! // Public-facing reader with caching
//! let public_reader = FileHelper::new()
//!     .with_middleware(CachingMiddleware::new(Duration::from_secs(60)));
//!
//! // Admin writer with strict security
//! let admin_writer = FileHelper::new()
//!     .with_security(strict_security_policy)
//!     .with_middleware(AuditMiddleware::new());
//! ```
//!
//! ## Pattern 2: Long-Running Services
//!
//! ```rust,ignore
//! // Configure at service startup
//! struct MyService {
//!     config_reader: ComposedHelper<...>,
//!     data_writer: ComposedHelper<...>,
//! }
//!
//! impl MyService {
//!     fn new() -> Self {
//!         Self {
//!             config_reader: FileHelper::new().with_middleware(...),
//!             data_writer: FileHelper::new().with_middleware(...),
//!         }
//!     }
//!
//!     async fn process_request(&self) -> Result<()> {
//!         // Reuse pre-configured pipelines
//!         let config = self.config_reader.read(...).await?;
//!         self.data_writer.write(...).await?;
//!         Ok(())
//!     }
//! }
//! ```
//!
//! # When to Use Level 3
//!
//! **Use Level 3 (Trait Composition) when:**
//! - Running long-lived services or microservices
//! - Processing high-throughput operations (>100 ops/sec)
//! - Building libraries with consistent middleware patterns
//! - Need clear separation of pipeline config from execution
//! - Functional programming style preferred
//!
//! **Use Level 2 (Custom Middleware) when:**
//! - One-off operations with custom middleware
//! - Exploring different middleware combinations
//! - Simple scripts with few operations
//!
//! **Use Level 1 (Simple Helpers) when:**
//! - Quick scripts and utilities
//! - Default security is sufficient
//! - Learning the framework
//!
//! # See Also
//!
//! - [`HelperPipeline`]: Core composition trait
//! - [`ComposedHelper`]: Pipeline wrapper
//! - [`FileHelper`], [`ProcessHelper`], [`NetworkHelper`]: Helper builders
//! - Level 1 API: `airssys_osl::helpers::*` (simple functions)
//! - Level 2 API: `airssys_osl::helpers::*_with_middleware()` variants
```

**Trait Definition:**
```rust
use std::marker::PhantomData;

use crate::core::context::{ExecutionContext, SecurityContext};
use crate::core::executor::{ExecutionResult, OSExecutor};
use crate::core::middleware::Middleware;
use crate::core::operation::Operation;
use crate::core::result::OSResult;
use crate::middleware::ext::ExecutorExt;

/// Trait for composable helper pipelines.
///
/// This trait provides a fluent API for building operation pipelines
/// with middleware composition.
///
/// # Type Parameters
///
/// - `O`: The operation type this pipeline handles
///
/// # Methods
///
/// - [`with_security()`]: Add SecurityMiddleware to the pipeline
/// - [`with_logger()`]: Add LoggerMiddleware to the pipeline
/// - [`with_middleware()`]: Add custom middleware to the pipeline
/// - [`executor()`]: Access the underlying executor
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::composition::*;
/// use airssys_osl::middleware::security::SecurityMiddleware;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let helper = FileHelper::new()
///     .with_security(SecurityMiddleware::default());
///
/// let data = helper.read("/etc/hosts", "admin").await?;
/// # Ok(())
/// # }
/// ```
pub trait HelperPipeline<O: Operation>: Sized {
    /// The underlying executor type.
    type Executor: OSExecutor<O>;

    /// Add security middleware to the pipeline.
    ///
    /// This is a convenience method for adding SecurityMiddleware.
    /// Use [`with_middleware()`] for other middleware types.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use airssys_osl::helpers::composition::*;
    /// # use airssys_osl::middleware::security::SecurityMiddleware;
    /// # async fn example() -> airssys_osl::core::result::OSResult<()> {
    /// let helper = FileHelper::new()
    ///     .with_security(SecurityMiddleware::default());
    /// # Ok(())
    /// # }
    /// ```
    fn with_security(
        self,
        middleware: crate::middleware::security::SecurityMiddleware,
    ) -> ComposedHelper<O, crate::middleware::ext::MiddlewareExecutor<Self::Executor, crate::middleware::security::SecurityMiddleware, O>>;

    /// Add logger middleware to the pipeline.
    ///
    /// This is a convenience method for adding LoggerMiddleware.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use airssys_osl::helpers::composition::*;
    /// # use airssys_osl::middleware::logger::LoggerMiddleware;
    /// # async fn example() -> airssys_osl::core::result::OSResult<()> {
    /// let helper = FileHelper::new()
    ///     .with_logger(LoggerMiddleware::default());
    /// # Ok(())
    /// # }
    /// ```
    fn with_logger(
        self,
        middleware: crate::middleware::logger::LoggerMiddleware,
    ) -> ComposedHelper<O, crate::middleware::ext::MiddlewareExecutor<Self::Executor, crate::middleware::logger::LoggerMiddleware, O>>;

    /// Add custom middleware to the pipeline.
    ///
    /// This is the generic method that accepts any middleware type.
    /// Use this for custom middleware like rate limiting, caching, metrics, etc.
    ///
    /// # Type Parameters
    ///
    /// - `M`: Middleware type implementing `Middleware<O>`
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use airssys_osl::helpers::composition::*;
    /// # use airssys_osl::middleware::security::SecurityMiddleware;
    /// // Using custom middleware (pseudocode - requires custom implementation)
    /// // let helper = FileHelper::new()
    /// //     .with_middleware(RateLimitMiddleware::new(100))
    /// //     .with_middleware(MetricsMiddleware::new());
    /// # async fn example() -> airssys_osl::core::result::OSResult<()> {
    /// # Ok(())
    /// # }
    /// ```
    fn with_middleware<M>(
        self,
        middleware: M,
    ) -> ComposedHelper<O, crate::middleware::ext::MiddlewareExecutor<Self::Executor, M, O>>
    where
        M: Middleware<O> + Send + Sync + std::fmt::Debug + 'static;

    /// Get reference to the underlying executor.
    ///
    /// This is primarily useful for testing or advanced use cases
    /// where direct executor access is needed.
    fn executor(&self) -> &Self::Executor;
}
```

**Deliverables:**
- ✅ Comprehensive module documentation (80-100 lines)
- ✅ `HelperPipeline<O>` trait definition
- ✅ Four trait methods with rustdoc examples
- ✅ Clear explanation of when to use Level 3 vs Level 1/2

---

### Phase 8.2: ComposedHelper & Builders (1 hour)

#### Objectives
- Implement `ComposedHelper<O, E>` wrapper struct
- Enable chaining by implementing `HelperPipeline` for `ComposedHelper`
- Create three helper builders as entry points

#### Implementation Details

**ComposedHelper Definition:**
```rust
/// A composed helper with middleware pipeline.
///
/// This type wraps an executor with a composable middleware pipeline.
/// It implements [`HelperPipeline`] to enable chaining.
///
/// # Type Parameters
///
/// - `O`: Operation type this helper executes
/// - `E`: Executor type (may be wrapped with middleware)
///
/// # Usage
///
/// `ComposedHelper` is typically created via helper builders like
/// [`FileHelper::new()`], not constructed directly.
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::composition::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// // Created via builder
/// let helper = FileHelper::new()
///     .with_security(/* ... */);
///
/// // Use the helper
/// let data = helper.read("/file.txt", "user").await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct ComposedHelper<O, E>
where
    O: Operation,
    E: OSExecutor<O>,
{
    executor: E,
    _phantom: PhantomData<O>,
}

impl<O, E> ComposedHelper<O, E>
where
    O: Operation,
    E: OSExecutor<O>,
{
    /// Create a new composed helper.
    ///
    /// This is typically called by helper builders, not user code.
    pub fn new(executor: E) -> Self {
        Self {
            executor,
            _phantom: PhantomData,
        }
    }

    /// Get reference to the executor.
    ///
    /// Primarily useful for testing or advanced use cases.
    pub fn executor(&self) -> &E {
        &self.executor
    }
}

// Implement HelperPipeline for ComposedHelper to enable chaining
impl<O, E> HelperPipeline<O> for ComposedHelper<O, E>
where
    O: Operation,
    E: OSExecutor<O> + Send + Sync + std::fmt::Debug,
{
    type Executor = E;

    fn with_security(
        self,
        middleware: crate::middleware::security::SecurityMiddleware,
    ) -> ComposedHelper<O, crate::middleware::ext::MiddlewareExecutor<E, crate::middleware::security::SecurityMiddleware, O>> {
        ComposedHelper::new(self.executor.with_middleware(middleware))
    }

    fn with_logger(
        self,
        middleware: crate::middleware::logger::LoggerMiddleware,
    ) -> ComposedHelper<O, crate::middleware::ext::MiddlewareExecutor<E, crate::middleware::logger::LoggerMiddleware, O>> {
        ComposedHelper::new(self.executor.with_middleware(middleware))
    }

    fn with_middleware<M>(
        self,
        middleware: M,
    ) -> ComposedHelper<O, crate::middleware::ext::MiddlewareExecutor<E, M, O>>
    where
        M: Middleware<O> + Send + Sync + std::fmt::Debug + 'static,
    {
        ComposedHelper::new(self.executor.with_middleware(middleware))
    }

    fn executor(&self) -> &E {
        &self.executor
    }
}
```

**Helper Builders:**
```rust
/// Filesystem operation helper builder.
///
/// Use this to create composable file operation pipelines.
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::composition::*;
/// use airssys_osl::middleware::security::SecurityMiddleware;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let reader = FileHelper::new()
///     .with_security(SecurityMiddleware::default());
///
/// let data = reader.read("/etc/hosts", "admin").await?;
/// # Ok(())
/// # }
/// ```
pub struct FileHelper;

impl FileHelper {
    /// Create a new file helper builder.
    ///
    /// Returns a `ComposedHelper` with a fresh `FilesystemExecutor`.
    /// Chain middleware using `.with_security()`, `.with_middleware()`, etc.
    pub fn new() -> ComposedHelper<
        crate::operations::filesystem::FileReadOperation,
        crate::executors::filesystem::FilesystemExecutor,
    > {
        ComposedHelper::new(crate::executors::filesystem::FilesystemExecutor::new())
    }
}

/// Process operation helper builder.
///
/// Use this to create composable process operation pipelines.
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::composition::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let spawner = ProcessHelper::new()
///     .with_security(/* ... */);
///
/// let pid = spawner.spawn("ls", vec!["-la".to_string()], "admin").await?;
/// # Ok(())
/// # }
/// ```
pub struct ProcessHelper;

impl ProcessHelper {
    /// Create a new process helper builder.
    ///
    /// Returns a `ComposedHelper` with a fresh `ProcessExecutor`.
    pub fn new() -> ComposedHelper<
        crate::operations::process::ProcessSpawnOperation,
        crate::executors::process::ProcessExecutor,
    > {
        ComposedHelper::new(crate::executors::process::ProcessExecutor::new(
            "composition_helper",
        ))
    }
}

/// Network operation helper builder.
///
/// Use this to create composable network operation pipelines.
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::composition::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let connector = NetworkHelper::new()
///     .with_security(/* ... */);
///
/// let socket = connector.connect("127.0.0.1:8080", "admin").await?;
/// # Ok(())
/// # }
/// ```
pub struct NetworkHelper;

impl NetworkHelper {
    /// Create a new network helper builder.
    ///
    /// Returns a `ComposedHelper` with a fresh `NetworkExecutor`.
    pub fn new() -> ComposedHelper<
        crate::operations::network::NetworkConnectOperation,
        crate::executors::network::NetworkExecutor,
    > {
        ComposedHelper::new(crate::executors::network::NetworkExecutor::new(
            "composition_helper",
        ))
    }
}
```

**Deliverables:**
- ✅ `ComposedHelper<O, E>` with `HelperPipeline` implementation
- ✅ Three helper builders with comprehensive rustdoc
- ✅ Type-safe chaining verified

---

### Phase 8.3: Execution Methods Implementation (1-2 hours)

#### Objectives
- Implement execution methods for all 10 operation types
- Enable actual operation execution on composed pipelines
- Provide ergonomic APIs matching simple helper signatures

#### Implementation Strategy

For each operation type, implement methods on `ComposedHelper` specialized for that operation:

**Filesystem Operations (4 implementations):**

```rust
// 1. FileReadOperation
impl<E> ComposedHelper<crate::operations::filesystem::FileReadOperation, E>
where
    E: OSExecutor<crate::operations::filesystem::FileReadOperation> + Send + Sync + std::fmt::Debug,
{
    /// Read file contents.
    ///
    /// # Arguments
    ///
    /// - `path`: File path to read
    /// - `user`: User identity for security context
    ///
    /// # Returns
    ///
    /// File contents as bytes
    ///
    /// # Errors
    ///
    /// - `OSError::SecurityViolation`: Security policy denied access
    /// - `OSError::NotFound`: File does not exist
    /// - `OSError::PermissionDenied`: OS-level permission denied
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use airssys_osl::helpers::composition::*;
    ///
    /// # async fn example() -> airssys_osl::core::result::OSResult<()> {
    /// let reader = FileHelper::new()
    ///     .with_security(SecurityMiddleware::default());
    ///
    /// let data = reader.read("/etc/hosts", "admin").await?;
    /// println!("Read {} bytes", data.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn read<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        user: impl Into<String>,
    ) -> OSResult<Vec<u8>> {
        let path_str = path.as_ref().display().to_string();
        let operation = crate::operations::filesystem::FileReadOperation::new(path_str);
        let context = ExecutionContext::new(SecurityContext::new(user.into()));
        let result = self.executor.execute(operation, &context).await?;
        Ok(result.output)
    }
}

// 2. FileWriteOperation
impl<E> ComposedHelper<crate::operations::filesystem::FileWriteOperation, E>
where
    E: OSExecutor<crate::operations::filesystem::FileWriteOperation> + Send + Sync + std::fmt::Debug,
{
    /// Write data to file.
    ///
    /// # Arguments
    ///
    /// - `path`: File path to write
    /// - `data`: Data to write (bytes)
    /// - `user`: User identity for security context
    ///
    /// # Returns
    ///
    /// Number of bytes written
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use airssys_osl::helpers::composition::*;
    /// # async fn example() -> airssys_osl::core::result::OSResult<()> {
    /// let writer = FileHelper::new().with_security(/* ... */);
    /// let bytes = writer.write("/tmp/test.txt", b"data".to_vec(), "admin").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn write<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        data: Vec<u8>,
        user: impl Into<String>,
    ) -> OSResult<usize> {
        let path_str = path.as_ref().display().to_string();
        let operation = crate::operations::filesystem::FileWriteOperation::new(path_str, data);
        let context = ExecutionContext::new(SecurityContext::new(user.into()));
        let result = self.executor.execute(operation, &context).await?;
        Ok(result.output)
    }
}

// 3. DirectoryCreateOperation - similar pattern
// 4. FileDeleteOperation - similar pattern
```

**Process Operations (3 implementations):**
```rust
// 5. ProcessSpawnOperation
// 6. ProcessKillOperation (requires signal)
// 7. ProcessSignalOperation
```

**Network Operations (3 implementations):**
```rust
// 8. NetworkConnectOperation
// 9. NetworkListenOperation
// 10. NetworkSocketOperation (UDP)
```

**Note:** Each implementation follows the same pattern:
1. Build operation from arguments
2. Create ExecutionContext with SecurityContext
3. Call executor.execute()
4. Return operation output

**Deliverables:**
- ✅ 10 execution method implementations
- ✅ Comprehensive rustdoc for each method
- ✅ Example code in documentation
- ✅ Proper error documentation

---

## File Organization

**Decision:** Module-based structure (Option B from DEVELOPMENT-PLAN.md)

```
airssys-osl/src/helpers/
├── mod.rs              # Module header, re-exports, top-level docs
├── simple.rs           # ~600 lines - Level 1 & 2 APIs (existing)
└── composition.rs      # ~400-500 lines - Level 3 API (NEW - Phase 8)
```

**composition.rs Structure (~400-500 lines):**
```
1. Module documentation (80-100 lines)
2. Imports (20 lines)
3. HelperPipeline trait (80 lines with docs)
4. ComposedHelper struct + implementation (100 lines)
5. Helper builders (60 lines)
6. Execution methods (150-200 lines for 10 operations)
```

**mod.rs Updates:**
```rust
// Add to existing mod.rs
pub mod composition;
pub use composition::{
    ComposedHelper, FileHelper, HelperPipeline, NetworkHelper, ProcessHelper,
};
```

---

## API Design Patterns

### Pattern 1: Builder Entry Points
```rust
// Each domain gets a dedicated builder
FileHelper::new()     // For filesystem operations
ProcessHelper::new()  // For process operations
NetworkHelper::new()  // For network operations
```

### Pattern 2: Fluent Chaining
```rust
FileHelper::new()
    .with_security(...)    // Specific middleware
    .with_logger(...)      // Specific middleware
    .with_middleware(...)  // Generic middleware
    .with_middleware(...)  // Chain multiple
```

### Pattern 3: Type Evolution
```rust
// Type changes with each chaining step:
FileHelper::new()
// → ComposedHelper<FileReadOp, FilesystemExecutor>

.with_security(sec)
// → ComposedHelper<FileReadOp, MiddlewareExecutor<FilesystemExecutor, SecurityMw, FileReadOp>>

.with_middleware(rate)
// → ComposedHelper<FileReadOp, MiddlewareExecutor<MiddlewareExecutor<...>, RateLimitMw, FileReadOp>>
```

### Pattern 4: Execution Methods
```rust
// Methods specialized per operation type
impl<E> ComposedHelper<FileReadOperation, E> {
    pub async fn read(...) -> OSResult<Vec<u8>> { ... }
}

impl<E> ComposedHelper<FileWriteOperation, E> {
    pub async fn write(...) -> OSResult<usize> { ... }
}
```

---

## Type System Design

### Generic Constraints

**HelperPipeline Trait:**
```rust
pub trait HelperPipeline<O: Operation>: Sized {
    type Executor: OSExecutor<O>;
    // ...
}
```

**ComposedHelper Implementation:**
```rust
impl<O, E> HelperPipeline<O> for ComposedHelper<O, E>
where
    O: Operation,
    E: OSExecutor<O> + Send + Sync + std::fmt::Debug,
{
    // Implementation enables chaining
}
```

**Execution Methods:**
```rust
impl<E> ComposedHelper<FileReadOperation, E>
where
    E: OSExecutor<FileReadOperation> + Send + Sync + std::fmt::Debug,
{
    // Specialized for specific operation type
}
```

### PhantomData Usage

```rust
pub struct ComposedHelper<O, E>
where
    O: Operation,
    E: OSExecutor<O>,
{
    executor: E,
    _phantom: PhantomData<O>,  // Ties struct to operation type
}
```

**Why PhantomData:**
- Links struct to operation type at compile time
- Enables type-safe execution methods
- No runtime overhead (zero-sized type)
- Prevents operation type mismatch

---

## Implementation Checklist

### Phase 8.1: HelperPipeline Trait
- [ ] Create `composition.rs` file
- [ ] Write comprehensive module documentation (80-100 lines)
- [ ] Add import statements (Layer 1, 2, 3 organization §2.1)
- [ ] Define `HelperPipeline<O>` trait
- [ ] Add rustdoc for trait with examples
- [ ] Add rustdoc for each method (4 methods)

### Phase 8.2: ComposedHelper & Builders
- [ ] Implement `ComposedHelper<O, E>` struct
- [ ] Implement `new()` and `executor()` methods
- [ ] Implement `HelperPipeline<O>` for `ComposedHelper`
- [ ] Create `FileHelper` builder
- [ ] Create `ProcessHelper` builder
- [ ] Create `NetworkHelper` builder
- [ ] Add comprehensive rustdoc for all types

### Phase 8.3: Execution Methods
- [ ] Filesystem: `read()` on `ComposedHelper<FileReadOperation, E>`
- [ ] Filesystem: `write()` on `ComposedHelper<FileWriteOperation, E>`
- [ ] Filesystem: `create()` on `ComposedHelper<DirectoryCreateOperation, E>`
- [ ] Filesystem: `delete()` on `ComposedHelper<FileDeleteOperation, E>`
- [ ] Process: `spawn()` on `ComposedHelper<ProcessSpawnOperation, E>`
- [ ] Process: `kill()` on `ComposedHelper<ProcessKillOperation, E>`
- [ ] Process: `send_signal()` on `ComposedHelper<ProcessSignalOperation, E>`
- [ ] Network: `connect()` on `ComposedHelper<NetworkConnectOperation, E>`
- [ ] Network: `listen()` on `ComposedHelper<NetworkListenOperation, E>`
- [ ] Network: `create_socket()` on `ComposedHelper<NetworkSocketOperation, E>`

### Phase 8.4: Integration
- [ ] Update `helpers/mod.rs` with re-exports
- [ ] Verify all imports working
- [ ] Run `cargo check --workspace` (zero errors)
- [ ] Run `cargo clippy --workspace` (zero warnings)
- [ ] Run `cargo doc --no-deps` (builds cleanly)

---

## Testing Strategy

**Phase 8 Testing (minimal - comprehensive testing in Phase 9):**

### Compilation Tests
- Verify all type signatures compile
- Check trait bounds are correct
- Ensure chaining works without errors

### Basic Smoke Tests
```rust
#[tokio::test]
async fn test_file_helper_basic_composition() {
    let helper = FileHelper::new()
        .with_security(SecurityMiddleware::default());
    
    // Type check - should compile
    assert!(true);
}

#[tokio::test]
async fn test_chaining_multiple_middleware() {
    let helper = FileHelper::new()
        .with_security(SecurityMiddleware::default())
        .with_logger(LoggerMiddleware::default());
    
    // Type check - should compile
    assert!(true);
}
```

**Note:** Comprehensive functional tests will be in Phase 9 once execution methods are complete.

---

## Success Criteria

### Functional Requirements
- ✅ `HelperPipeline<O>` trait compiles
- ✅ `ComposedHelper<O, E>` implements `HelperPipeline`
- ✅ Three helper builders create initial `ComposedHelper` instances
- ✅ Chaining works (`.with_security().with_middleware()`)
- ✅ All 10 execution methods compile
- ✅ Type inference works correctly

### Quality Requirements
- ✅ Zero compiler errors
- ✅ Zero compiler warnings
- ✅ Zero clippy warnings
- ✅ Comprehensive rustdoc (module + trait + types + methods)
- ✅ Code examples in all public documentation
- ✅ Follows workspace standards (§2.1-§5.1)
- ✅ Microsoft Rust Guidelines compliance

### Documentation Requirements
- ✅ Module-level docs explain composition concept
- ✅ When to use Level 3 vs Level 1/2 documented
- ✅ All types have rustdoc with examples
- ✅ All methods have rustdoc with examples
- ✅ Performance characteristics documented
- ✅ Usage patterns documented

### Code Quality
- ✅ Proper error handling
- ✅ Type-safe APIs
- ✅ No unsafe code
- ✅ No unwrap() or expect() in library code
- ✅ Consistent naming conventions
- ✅ Clean separation of concerns

---

## Risk Mitigation

### Risk 1: Complex Type Signatures
**Problem:** Nested `MiddlewareExecutor` types become verbose

**Mitigation:**
- Use type aliases for common patterns (if needed)
- Focus on API ergonomics over implementation simplicity
- Let compiler handle complex types internally
- Document user-facing types clearly

### Risk 2: Trait Bound Complexity
**Problem:** Incorrect trait bounds prevent compilation

**Mitigation:**
- Start with minimal bounds, add as needed
- Follow patterns from `ExecutorExt` (OSL-TASK-009)
- Iterative refinement based on compiler feedback
- Test chaining incrementally

### Risk 3: PhantomData Lifetime Issues
**Problem:** Incorrect PhantomData usage causes lifetime errors

**Mitigation:**
- Follow established Rust patterns (`PhantomData<O>`)
- No lifetime parameters needed (all owned types)
- Verify with compilation tests

### Risk 4: API Usability
**Problem:** APIs too complex or unintuitive

**Mitigation:**
- Focus on examples in documentation
- Provide builder entry points for clarity
- Match simple helper signatures where possible
- User testing with example code

### Risk 5: Documentation Complexity
**Problem:** Generic types hard to document clearly

**Mitigation:**
- Use concrete examples in rustdoc
- Focus on usage patterns over implementation
- Provide "when to use" guidance
- Link to simpler Level 1/2 APIs

---

## Next Steps After Phase 8

**Phase 9: Trait Composition Implementation**
- Comprehensive functional tests for composition layer
- Test all 10 execution methods
- Test middleware chaining with multiple types
- Validate zero-cost abstraction claims
- Integration tests with custom middleware

**Phase 10: Advanced Usage Patterns**
- Create `examples/helper_composition.rs`
- Document complex composition patterns
- Real-world use case demonstrations
- Performance comparison with Level 1/2

**Phase 11: Final QA & Documentation**
- Complete integration testing
- Performance benchmarking
- Final documentation review
- Production readiness verification

---

## References

### Related Memory Bank Files
- **OSL-TASK-010-DEVELOPMENT-PLAN.md**: Complete 11-phase roadmap
- **OSL-TASK-010-helper-middleware-integration.md**: Main task tracking
- **KNOW-013**: Helper Composition Strategies (architecture analysis)

### Related Code
- **airssys-osl/src/middleware/ext.rs**: `ExecutorExt` trait (OSL-TASK-009)
- **airssys-osl/src/helpers/simple.rs**: Level 1 & 2 APIs (Phases 1-7)
- **airssys-osl/src/middleware/security/**: SecurityMiddleware
- **airssys-osl/src/operations/**: All operation types

### Standards
- **§2.1**: 3-Layer Import Organization
- **§4.3**: Module Architecture
- **§6.1**: YAGNI Principles
- **§6.3**: Microsoft Rust Guidelines

---

**This detailed plan provides everything needed to implement Phase 8 successfully!** 🚀
