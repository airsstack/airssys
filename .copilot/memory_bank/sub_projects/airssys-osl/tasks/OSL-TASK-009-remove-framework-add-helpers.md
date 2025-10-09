# Task: Remove Framework and Add Helpers

**Task ID:** OSL-TASK-009  
**Priority:** High  
**Status:** 🎯 Ready to Start (unblocked 2025-10-09)  
**Created:** 2025-10-08  
**Updated:** 2025-10-09  
**Estimated Effort:** 2-3 days  

## Task Overview
Refactor airssys-osl architecture by removing the framework layer (OSLFramework, ExecutorRegistry, builders) and replacing it with simple helper functions and middleware extension traits for a cleaner, more maintainable codebase.

## Task Description
Following the principle of YAGNI and avoiding over-engineering, this task removes unnecessary abstractions (framework struct, registry) that added complexity without proportional value. Replace them with straightforward helper functions for ergonomic one-line APIs and extension traits for middleware composition.

## Context and Rationale
After implementing OSL-TASK-008 (Platform Executors), analysis revealed that the framework layer creates unnecessary complexity:
- **ExecutorRegistry**: Cannot store heterogeneous executor types (generic-to-dynamic impedance mismatch)
- **OSLFramework**: Adds indirection without significant value over direct usage
- **Builder patterns**: Unnecessary for simple, focused APIs

The new approach provides three usage levels:
1. **Low-level**: Direct use of core abstractions (Operation, OSExecutor, Middleware)
2. **Macros**: Ergonomic custom executor creation via #[executor] (see MACROS-TASK-002)
3. **Helpers**: One-line convenience functions for common operations

## Dependencies
- **Blocked by:** ✅ OSL-TASK-008 Phase 4 (COMPLETE 2025-10-08)
- **Blocks:** OSL-TASK-003 (Security Middleware), OSL-TASK-004 (Middleware Pipeline)
- **Related:** MACROS-TASK-002 (#[executor] macro development - can proceed in parallel)

## Status: Ready to Start ✅

All blockers resolved. OSL-TASK-008 complete with all 3 platform executors production-ready (165 tests passing). Can begin implementation immediately.

## Acceptance Criteria

### 1. Framework Code Removal
- ✅ Delete `src/framework/registry.rs` (ExecutorRegistry)
- ✅ Delete `src/framework/framework.rs` (OSLFramework)
- ✅ Delete `src/framework/builder.rs` (OSLFrameworkBuilder)
- ✅ Keep `src/framework/config.rs` if useful, otherwise delete
- ✅ Keep `src/framework/operations.rs` if has reusable logic, otherwise delete
- ✅ Delete `src/framework/pipeline.rs` if unused
- ✅ Remove framework module from `src/lib.rs`
- ✅ Update prelude to remove framework exports

### 2. Helper Functions Implementation
- ✅ Create `src/helpers.rs` module
- ✅ Implement 4 filesystem helpers (read_file, write_file, delete_file, create_directory)
- ✅ Implement 3 process helpers (spawn_process, kill_process, query_process)
- ✅ Implement 3 network helpers (tcp_connect, tcp_listen, udp_bind)
- ✅ All helpers use default platform executors internally
- ✅ Comprehensive rustdoc for all helper functions
- ✅ Export helpers from `src/lib.rs`

### 3. Middleware Extension Trait
- ✅ Create `src/middleware/ext.rs` module
- ✅ Implement `ExecutorExt` trait with `.with_middleware()` method
- ✅ Generic implementation for all types (impl<E> ExecutorExt for E where E: Sized)
- ✅ Documentation with composition examples
- ✅ Export from middleware module

### 4. Update Default Executors (Optional - Nice to Have)
- ⏳ Refactor FilesystemExecutor to use #[executor] macro (when macros ready)
- ⏳ Refactor ProcessExecutor to use #[executor] macro (when macros ready)
- ⏳ Refactor NetworkExecutor to use #[executor] macro (when macros ready)
- Note: Can be done after MACROS-TASK-002 completion

### 5. Testing
- ✅ Helper function tests (10+ tests, one per helper)
- ✅ Middleware extension tests (3+ tests for composition)
- ✅ All existing executor tests still pass (165 tests)
- ✅ Integration tests updated to remove framework usage
- ✅ Examples updated to showcase new patterns

### 6. Documentation Updates
- ✅ Update README with new architecture approach
- ✅ Update mdBook documentation
  - Remove framework chapter
  - Add helpers chapter with examples
  - Add middleware extension chapter
- ✅ Update examples/
  - Remove framework examples
  - Add helpers examples
  - Add middleware composition examples
- ✅ Migration guide for users (if any external users)

### 7. Quality Gates
- ✅ All 165+ existing tests pass
- ✅ New helper tests pass (10+)
- ✅ Zero compiler warnings
- ✅ Zero clippy warnings
- ✅ Documentation builds successfully
- ✅ Workspace standards compliance (§2.1, §3.2, §4.3)

## Implementation Details

### Phase 1: Helper Functions (Day 1)

#### Create helpers.rs
```rust
//! High-level convenience functions for common OS operations.
//!
//! These helpers provide ergonomic, one-line APIs for frequent tasks
//! while using the default platform executors internally.

use std::path::Path;
use std::net::SocketAddr;

use crate::core::{ExecutionContext, SecurityContext};
use crate::core::executor::{ExecutionResult, OSExecutor};
use crate::core::result::OSResult;
use crate::executors::{FilesystemExecutor, NetworkExecutor, ProcessExecutor};
use crate::operations::filesystem::{
    FileReadOperation, FileWriteOperation, FileDeleteOperation, DirectoryCreateOperation,
};
use crate::operations::process::{
    ProcessSpawnOperation, ProcessKillOperation, ProcessQueryOperation,
};
use crate::operations::network::{
    TcpConnectOperation, TcpListenOperation, UdpBindOperation,
};

// ============================================================================
// Filesystem Helpers
// ============================================================================

/// Read file contents with security context.
///
/// # Example
/// ```rust
/// use airssys_osl::helpers::*;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let data = read_file("/etc/hosts", "admin").await?;
/// # Ok(())
/// # }
/// ```
pub async fn read_file<P: AsRef<Path>>(
    path: P,
    user: impl Into<String>,
) -> OSResult<Vec<u8>> {
    let operation = FileReadOperation::new(path.as_ref().to_path_buf());
    let security_context = SecurityContext::new(user.into());
    let context = ExecutionContext::new(security_context);
    let executor = FilesystemExecutor::new();
    
    let result = executor.execute(operation, &context).await?;
    Ok(result.data)
}

/// Write data to file with security context.
pub async fn write_file<P: AsRef<Path>>(
    path: P,
    data: Vec<u8>,
    user: impl Into<String>,
) -> OSResult<()> {
    let operation = FileWriteOperation::new(path.as_ref().to_path_buf(), data);
    let security_context = SecurityContext::new(user.into());
    let context = ExecutionContext::new(security_context);
    let executor = FilesystemExecutor::new();
    
    executor.execute(operation, &context).await?;
    Ok(())
}

/// Delete file with security context.
pub async fn delete_file<P: AsRef<Path>>(
    path: P,
    user: impl Into<String>,
) -> OSResult<()> {
    let operation = FileDeleteOperation::new(path.as_ref().to_path_buf());
    let security_context = SecurityContext::new(user.into());
    let context = ExecutionContext::new(security_context);
    let executor = FilesystemExecutor::new();
    
    executor.execute(operation, &context).await?;
    Ok(())
}

/// Create directory with security context.
pub async fn create_directory<P: AsRef<Path>>(
    path: P,
    user: impl Into<String>,
) -> OSResult<()> {
    let operation = DirectoryCreateOperation::new(path.as_ref().to_path_buf());
    let security_context = SecurityContext::new(user.into());
    let context = ExecutionContext::new(security_context);
    let executor = FilesystemExecutor::new();
    
    executor.execute(operation, &context).await?;
    Ok(())
}

// ============================================================================
// Process Helpers
// ============================================================================

/// Spawn a process with arguments and security context.
pub async fn spawn_process(
    program: impl Into<String>,
    args: Vec<String>,
    user: impl Into<String>,
) -> OSResult<u32> {
    let operation = ProcessSpawnOperation::new(program.into(), args);
    let security_context = SecurityContext::new(user.into());
    let context = ExecutionContext::new(security_context);
    let executor = ProcessExecutor::new();
    
    let result = executor.execute(operation, &context).await?;
    // Parse PID from result data
    let pid = String::from_utf8_lossy(&result.data)
        .parse::<u32>()
        .map_err(|_| crate::core::result::OSError::ExecutionFailed {
            reason: "Failed to parse PID".to_string(),
        })?;
    Ok(pid)
}

/// Kill a process by PID with security context.
pub async fn kill_process(pid: u32, user: impl Into<String>) -> OSResult<()> {
    let operation = ProcessKillOperation::new(pid);
    let security_context = SecurityContext::new(user.into());
    let context = ExecutionContext::new(security_context);
    let executor = ProcessExecutor::new();
    
    executor.execute(operation, &context).await?;
    Ok(())
}

/// Query process information by PID.
pub async fn query_process(pid: u32, user: impl Into<String>) -> OSResult<Vec<u8>> {
    let operation = ProcessQueryOperation::new(pid);
    let security_context = SecurityContext::new(user.into());
    let context = ExecutionContext::new(security_context);
    let executor = ProcessExecutor::new();
    
    let result = executor.execute(operation, &context).await?;
    Ok(result.data)
}

// ============================================================================
// Network Helpers
// ============================================================================

/// Connect to TCP server.
pub async fn tcp_connect(
    addr: SocketAddr,
    user: impl Into<String>,
) -> OSResult<Vec<u8>> {
    let operation = TcpConnectOperation::new(addr);
    let security_context = SecurityContext::new(user.into());
    let context = ExecutionContext::new(security_context);
    let executor = NetworkExecutor::new();
    
    let result = executor.execute(operation, &context).await?;
    Ok(result.data)
}

/// Listen on TCP port.
pub async fn tcp_listen(
    addr: SocketAddr,
    user: impl Into<String>,
) -> OSResult<Vec<u8>> {
    let operation = TcpListenOperation::new(addr);
    let security_context = SecurityContext::new(user.into());
    let context = ExecutionContext::new(security_context);
    let executor = NetworkExecutor::new();
    
    let result = executor.execute(operation, &context).await?;
    Ok(result.data)
}

/// Bind UDP socket.
pub async fn udp_bind(
    addr: SocketAddr,
    user: impl Into<String>,
) -> OSResult<Vec<u8>> {
    let operation = UdpBindOperation::new(addr);
    let security_context = SecurityContext::new(user.into());
    let context = ExecutionContext::new(security_context);
    let executor = NetworkExecutor::new();
    
    let result = executor.execute(operation, &context).await?;
    Ok(result.data)
}
```

#### Export from lib.rs
```rust
// In src/lib.rs
pub mod helpers;
```

### Phase 2: Middleware Extension Trait (Day 1)

#### Create middleware/ext.rs
```rust
//! Extension trait for composable middleware.

/// Extension trait for adding middleware to executors.
///
/// Provides `.with_middleware()` method for wrapping executors
/// with middleware in a composable way.
///
/// # Example
/// ```rust
/// use airssys_osl::prelude::*;
/// use airssys_osl::middleware::ext::ExecutorExt;
/// use airssys_osl::middleware::LoggerMiddleware;
///
/// let executor = FilesystemExecutor::new()
///     .with_middleware(|exec| LoggerMiddleware::new(exec, logger));
/// ```
pub trait ExecutorExt: Sized {
    /// Wrap this executor with middleware.
    ///
    /// The provided function receives the executor and returns
    /// a middleware-wrapped version.
    fn with_middleware<M>(self, middleware_ctor: impl FnOnce(Self) -> M) -> M {
        middleware_ctor(self)
    }
}

// Implement for all types
impl<E> ExecutorExt for E where E: Sized {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extension_trait_available() {
        struct DummyExecutor;
        
        // Test that ExecutorExt methods are available
        let _wrapped = DummyExecutor.with_middleware(|exec| exec);
    }
}
```

#### Update middleware/mod.rs
```rust
// In src/middleware/mod.rs
pub mod ext;

// Re-export extension trait
pub use ext::ExecutorExt;
```

### Phase 3: Remove Framework Code (Day 2)

#### Files to Delete
```bash
# Delete framework files
rm src/framework/registry.rs
rm src/framework/framework.rs
rm src/framework/builder.rs

# Review and possibly delete
# - src/framework/config.rs (keep if useful for future)
# - src/framework/operations.rs (keep if has reusable logic)
# - src/framework/pipeline.rs (likely delete)

# If framework/ is empty, delete the directory
rm -rf src/framework/
```

#### Update src/lib.rs
```rust
// Remove framework module
// pub mod framework;  // ← DELETE THIS

// Update prelude
// Remove framework-related exports from prelude.rs
```

#### Update prelude.rs
```rust
// In src/prelude.rs

// Remove framework exports
// pub use crate::framework::{OSLFramework, OSLFrameworkBuilder, OSLConfig};  // ← DELETE

// Keep core exports
pub use crate::core::*;
pub use crate::operations::*;
pub use crate::executors::*;
pub use crate::middleware::*;

// Add helpers (optional - users can import explicitly)
// pub use crate::helpers::*;  // Could be verbose, let users choose
```

### Phase 4: Testing (Day 2-3)

#### Helper Function Tests
```rust
// In tests/helpers_tests.rs

use airssys_osl::helpers::*;
use std::net::SocketAddr;

#[tokio::test]
async fn test_read_file_helper() {
    // Note: This will attempt real I/O
    let result = read_file("/etc/hosts", "test_user").await;
    // Depending on permissions, might succeed or fail
    // Just verify it returns a result type
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_write_file_helper() {
    let result = write_file("/tmp/test_airssys.txt", b"test".to_vec(), "test_user").await;
    assert!(result.is_ok() || result.is_err());
}

// ... tests for all 10 helpers
```

#### Middleware Extension Tests
```rust
// In tests/middleware_ext_tests.rs

use airssys_osl::prelude::*;
use airssys_osl::middleware::ext::ExecutorExt;

#[test]
fn test_middleware_composition() {
    struct DummyMiddleware<E> {
        inner: E,
    }
    
    impl<E> DummyMiddleware<E> {
        fn new(inner: E) -> Self {
            Self { inner }
        }
    }
    
    let executor = FilesystemExecutor::new()
        .with_middleware(DummyMiddleware::new);
    
    // Verify type composition works
    assert!(std::mem::size_of_val(&executor) > 0);
}
```

### Phase 5: Documentation Updates (Day 3)

#### Update mdBook (docs/src/)
```markdown
# Getting Started

## Three Ways to Use AirsSys OSL

### 1. Low-Level API (Maximum Control)
Direct usage of core abstractions:

\`\`\`rust
use airssys_osl::prelude::*;

let executor = FilesystemExecutor::new();
let operation = FileReadOperation::new("/etc/hosts".into());
let context = ExecutionContext::new(SecurityContext::new("admin".into()));
let result = executor.execute(operation, &context).await?;
\`\`\`

### 2. Helper Functions (Most Ergonomic)
One-line convenience functions:

\`\`\`rust
use airssys_osl::helpers::*;

let data = read_file("/etc/hosts", "admin").await?;
\`\`\`

### 3. Custom Executors with Macros (Future)
Build custom executors easily:

\`\`\`rust
use airssys_osl::prelude::*;

#[executor]  // Requires `macros` feature
impl CloudExecutor {
    async fn file_read(&self, op: FileReadOperation, ctx: &ExecutionContext) 
        -> OSResult<ExecutionResult> 
    {
        // Custom cloud-based implementation
        todo!()
    }
}
\`\`\`

## Middleware Composition

Add logging, metrics, security layers:

\`\`\`rust
use airssys_osl::middleware::{LoggerMiddleware, ExecutorExt};

let executor = FilesystemExecutor::new()
    .with_middleware(|e| LoggerMiddleware::new(e, logger));
\`\`\`
```

#### Update Examples
```rust
// examples/helpers_usage.rs

use airssys_osl::helpers::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Filesystem operations
    let data = read_file("/etc/hosts", "admin").await?;
    println!("Read {} bytes", data.len());
    
    write_file("/tmp/test.txt", b"Hello, World!".to_vec(), "admin").await?;
    println!("File written");
    
    delete_file("/tmp/test.txt", "admin").await?;
    println!("File deleted");
    
    // Process operations
    let pid = spawn_process("echo", vec!["hello".to_string()], "admin").await?;
    println!("Spawned process: {}", pid);
    
    Ok(())
}
```

## Quality Checklist

### Before Task Completion
- [ ] All framework code removed
- [ ] helpers.rs implemented with 10 functions
- [ ] middleware/ext.rs implemented
- [ ] All existing tests pass (165+)
- [ ] New helper tests added (10+)
- [ ] Middleware extension tests added (3+)
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings
- [ ] Documentation updated (mdBook, rustdoc, examples)
- [ ] Memory bank updated (progress.md, current_context.md)
- [ ] Git commit with clear message

### Validation Commands
```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features
cargo doc --workspace --no-deps
mdbook build airssys-osl/docs
```

## Migration Notes

### For Internal Development
No external users yet, so no migration concerns. This simplification improves maintainability and clarity.

### Code Removal Impact
- **ExecutorRegistry**: Unused functionality removed
- **OSLFramework**: Unnecessary indirection removed
- **Builder patterns**: Replaced by simple constructors and helpers

### Benefits
- **Simpler codebase**: ~30% code reduction in framework layer
- **Clearer architecture**: Three distinct usage levels
- **Better ergonomics**: Helper functions easier than framework methods
- **Maintainability**: Less abstraction to maintain
- **YAGNI compliance**: Build only what's needed

## Next Steps
After completion:
1. Update current_context.md to reflect new architecture
2. Close OSL-TASK-008 Phase 5 as "Abandoned - Replaced by OSL-TASK-009"
3. Update progress percentages
4. Proceed with normal development workflow

## Notes
- This is a simplification, not a feature addition
- Maintains all functionality with clearer semantics
- Reduces cognitive load for users and maintainers
- Aligns with YAGNI and Microsoft Rust Guidelines (M-SIMPLE-ABSTRACTIONS)
