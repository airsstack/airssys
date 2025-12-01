# OSL-TASK-009 Development Plan
# Remove Framework and Add Helpers

**Task ID:** OSL-TASK-009  
**Status:** ✅ COMPLETE (All 4 phases)  
**Priority:** High  
**Actual Effort:** 2 days  
**Created:** 2025-10-09  
**Completed:** 2025-10-09

---

## Quick Summary

**Goal:** Simplify airssys-osl by removing framework abstractions and replacing with helper functions and extension traits.

**Why:** Framework layer adds unnecessary complexity without proportional value. New architecture provides three clean usage levels: low-level API, helper functions, and macros.

**Impact:** ~30% code reduction, clearer architecture, better ergonomics, easier maintenance.

---

## Development Timeline

### **Day 1: Framework Removal & Helper Functions (6-8 hours)**

#### **Phase 1: Delete Framework Code (1-2 hours)**

**Morning - Part 1: Delete Framework Files**

Delete these files:
```bash
rm airssys-osl/src/framework/registry.rs
rm airssys-osl/src/framework/framework.rs
rm airssys-osl/src/framework/builder.rs
rm airssys-osl/src/framework/pipeline.rs
rm airssys-osl/src/framework/operations.rs
```

Keep if useful (review during deletion):
- `config.rs` - May have reusable SecurityConfig

**Update module structure:**
1. Remove framework from `src/lib.rs`
2. Update `src/prelude.rs` - remove framework exports
3. Check if `src/framework/mod.rs` can be deleted or needs refactoring

**Verify:**
```bash
cargo check --package airssys-osl
# Expect errors - this is normal, we'll fix in next phases
```

---

#### **Phase 2: Create Helper Functions Module (3-4 hours)**

**Morning/Afternoon - Part 2: Implement Helpers**

**⚠️ IMPORTANT NOTE - Future Integration:**
These helper functions are currently implemented with **direct executor calls** for simplicity. After OSL-TASK-003 (Security Middleware) and OSL-TASK-004 (Middleware Pipeline) are complete, these helpers will need to be updated to:
1. **Integrate security validation** - Apply security policies before execution
2. **Support middleware pipeline** - Allow middleware composition (logging, metrics, etc.)
3. **Maintain backward compatibility** - Keep simple API, add internal pipeline support

**TODO for Future Tasks:**
- [ ] OSL-TASK-003: Add security validation to all helpers
- [ ] OSL-TASK-004: Wire helpers through middleware pipeline
- [ ] Add optional `with_middleware()` parameter to helpers for advanced usage

**Step 1: Create `src/helpers.rs`**

Structure:
```rust
//! High-level convenience functions for common OS operations.
//!
//! # Security and Middleware Integration
//!
//! **Current Implementation:** These helpers use direct executor calls for simplicity.
//!
//! **Future Enhancement (OSL-TASK-003, OSL-TASK-004):**
//! - Security policy validation will be integrated
//! - Middleware pipeline support will be added
//! - APIs will remain backward compatible
//!
//! See: OSL-TASK-003 (Security Middleware), OSL-TASK-004 (Middleware Pipeline)

use std::path::Path;
use std::net::SocketAddr;
use crate::prelude::*;
use crate::executors::{FilesystemExecutor, ProcessExecutor, NetworkExecutor};

// ============================================================================
// Filesystem Helpers (4 functions)
// ============================================================================

/// Read file contents with security context.
///
/// # Current Implementation
/// Direct executor call for simplicity.
///
/// # Future Integration (OSL-TASK-003, OSL-TASK-004)
/// - TODO: Add security policy validation
/// - TODO: Wire through middleware pipeline
/// - TODO: Support optional middleware composition
///
/// # Example
/// ```rust
/// use airssys_osl::helpers::*;
///
/// let data = read_file("/etc/hosts", "admin").await?;
/// ```
pub async fn read_file<P: AsRef<Path>>(path: P, user: impl Into<String>) -> OSResult<Vec<u8>> {
    // TODO(OSL-TASK-003): Add security validation here
    // TODO(OSL-TASK-004): Wire through middleware pipeline
    let operation = FileReadOperation::new(path.as_ref().to_path_buf());
    let context = ExecutionContext::new(SecurityContext::new(user.into()));
    let executor = FilesystemExecutor::new();
    let result = executor.execute(operation, &context).await?;
    Ok(result.data)
}

/// Write data to file with security context.
///
/// # Future Integration (OSL-TASK-003, OSL-TASK-004)
/// - TODO: Add security policy validation
/// - TODO: Wire through middleware pipeline
pub async fn write_file<P: AsRef<Path>>(
    path: P, 
    data: Vec<u8>, 
    user: impl Into<String>
) -> OSResult<()> {
    // TODO(OSL-TASK-003): Add security validation
    // TODO(OSL-TASK-004): Wire through middleware pipeline
    let operation = FileWriteOperation::new(path.as_ref().to_path_buf(), data);
    let context = ExecutionContext::new(SecurityContext::new(user.into()));
    let executor = FilesystemExecutor::new();
    executor.execute(operation, &context).await?;
    Ok(())
}

pub async fn delete_file<P: AsRef<Path>>(path: P, user: impl Into<String>) -> OSResult<()> {
    let operation = FileDeleteOperation::new(path.as_ref().to_path_buf());
    let context = ExecutionContext::new(SecurityContext::new(user.into()));
    let executor = FilesystemExecutor::new();
    executor.execute(operation, &context).await?;
    Ok(())
}

pub async fn create_directory<P: AsRef<Path>>(
    path: P, 
    user: impl Into<String>
) -> OSResult<()> {
    let operation = DirectoryCreateOperation::new(path.as_ref().to_path_buf());
    let context = ExecutionContext::new(SecurityContext::new(user.into()));
    let executor = FilesystemExecutor::new();
    executor.execute(operation, &context).await?;
    Ok(())
}

// ============================================================================
// Process Helpers (3 functions)
// ============================================================================

pub async fn spawn_process(
    program: impl Into<String>,
    args: Vec<String>,
    user: impl Into<String>,
) -> OSResult<Vec<u8>> {
    let operation = ProcessSpawnOperation::new(program.into(), args);
    let context = ExecutionContext::new(SecurityContext::new(user.into()));
    let executor = ProcessExecutor::new();
    let result = executor.execute(operation, &context).await?;
    Ok(result.data)
}

pub async fn kill_process(pid: u32, user: impl Into<String>) -> OSResult<()> {
    let operation = ProcessKillOperation::new(pid);
    let context = ExecutionContext::new(SecurityContext::new(user.into()));
    let executor = ProcessExecutor::new();
    executor.execute(operation, &context).await?;
    Ok(())
}

pub async fn send_signal(
    pid: u32,
    signal: i32,
    user: impl Into<String>
) -> OSResult<()> {
    let operation = ProcessSignalOperation::new(pid, signal);
    let context = ExecutionContext::new(SecurityContext::new(user.into()));
    let executor = ProcessExecutor::new();
    executor.execute(operation, &context).await?;
    Ok(())
}

// ============================================================================
// Network Helpers (3 functions)
// ============================================================================

pub async fn network_connect(
    addr: impl Into<String>,
    user: impl Into<String>,
) -> OSResult<Vec<u8>> {
    let operation = NetworkConnectOperation::new(addr.into());
    let context = ExecutionContext::new(SecurityContext::new(user.into()));
    let executor = NetworkExecutor::new();
    let result = executor.execute(operation, &context).await?;
    Ok(result.data)
}

pub async fn network_listen(
    addr: impl Into<String>,
    user: impl Into<String>,
) -> OSResult<Vec<u8>> {
    let operation = NetworkListenOperation::new(addr.into());
    let context = ExecutionContext::new(SecurityContext::new(user.into()));
    let executor = NetworkExecutor::new();
    let result = executor.execute(operation, &context).await?;
    Ok(result.data)
}

pub async fn create_socket(
    socket_type: impl Into<String>,
    user: impl Into<String>,
) -> OSResult<Vec<u8>> {
    let operation = NetworkSocketOperation::new(socket_type.into());
    let context = ExecutionContext::new(SecurityContext::new(user.into()));
    let executor = NetworkExecutor::new();
    let result = executor.execute(operation, &context).await?;
    Ok(result.data)
}
```

**Step 2: Export from `lib.rs`**
```rust
pub mod helpers;
```

**Step 3: Write Helper Tests**

Create `src/helpers.rs` tests at bottom of file:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_read_file_helper() {
        // Test implementation
    }
    
    #[tokio::test]
    async fn test_write_file_helper() {
        // Test implementation
    }
    
    // ... 8 more tests (one per helper)
}
```

**Verify:**
```bash
cargo test --package airssys-osl helpers::tests
```

---

#### **Phase 3: Middleware Extension Trait (2 hours)**

**Afternoon - Part 3: Add Extension Trait**

**Step 1: Create `src/middleware/ext.rs`**

```rust
//! Extension trait for composable middleware.

/// Extension trait enabling middleware composition.
///
/// This trait provides `.with_middleware()` for wrapping executors
/// with middleware in a composable, fluent API style.
///
/// # Example
/// ```rust
/// use airssys_osl::prelude::*;
/// use airssys_osl::middleware::ExecutorExt;
///
/// let executor = FilesystemExecutor::new()
///     .with_middleware(|e| LoggerMiddleware::new(e, logger))
///     .with_middleware(|e| MetricsMiddleware::new(e, collector));
/// ```
pub trait ExecutorExt: Sized {
    /// Wrap this executor with middleware.
    ///
    /// The `middleware_ctor` function receives ownership of this executor
    /// and returns the wrapped middleware instance.
    fn with_middleware<M>(self, middleware_ctor: impl FnOnce(Self) -> M) -> M {
        middleware_ctor(self)
    }
}

// Blanket implementation for all types
impl<E> ExecutorExt for E where E: Sized {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_executor_ext_single_middleware() {
        // Test wrapping with single middleware
    }
    
    #[test]
    fn test_executor_ext_multiple_middleware() {
        // Test chaining multiple middleware
    }
    
    #[test]
    fn test_executor_ext_preserves_type() {
        // Test type safety
    }
}
```

**Step 2: Export from `middleware/mod.rs`**
```rust
pub mod ext;
pub use ext::ExecutorExt;
```

**Verify:**
```bash
cargo test --package airssys-osl middleware::ext::tests
```

---

### **Day 2: Testing, Documentation & Migration (6-8 hours)**

#### **Phase 4: Update Existing Tests (2-3 hours)**

**Morning - Part 1: Fix Broken Tests**

1. **Find all framework-dependent tests:**
```bash
grep -r "OSLFramework\|ExecutorRegistry\|OSLFrameworkBuilder" airssys-osl/tests/
```

2. **Update integration tests:**
   - Replace framework usage with direct executor usage
   - Update to use helper functions where appropriate
   - Ensure 165+ existing tests still pass

3. **Update examples:**
   - `examples/basic_usage.rs` - use helpers
   - `examples/middleware_pipeline.rs` - use extension trait
   - Remove/update framework-dependent examples

**Verify:**
```bash
cargo test --workspace
cargo run --example basic_usage
```

---

#### **Phase 5: Documentation Updates (2-3 hours)**

**Morning/Afternoon - Part 2: Update Documentation**

**1. Update README.md**

Add new usage examples:
```markdown
## Quick Start

### Helper Functions (Easiest)
```rust
use airssys_osl::helpers::*;

let data = read_file("/etc/hosts", "admin").await?;
```

### Direct API (Maximum Control)
```rust
use airssys_osl::prelude::*;

let executor = FilesystemExecutor::new();
let operation = FileReadOperation::new("/etc/hosts".into());
let context = ExecutionContext::new(SecurityContext::new("admin".into()));
let result = executor.execute(operation, &context).await?;
```

### Custom Executors with Macros
```rust
use airssys_osl::prelude::*;

#[executor]  // Requires 'macros' feature
impl MyExecutor {
    async fn file_read(&self, op: FileReadOperation, ctx: &ExecutionContext) 
        -> OSResult<ExecutionResult> {
        // Custom implementation
    }
}
```
```

**2. Update mdBook Documentation**

Create/update these files:
- `docs/src/guides/helper-functions.md` (new)
- `docs/src/guides/middleware-composition.md` (new)
- `docs/src/guides/custom-executors.md` (update - remove framework references)
- Delete `docs/src/guides/framework-usage.md` (obsolete)

**3. Update API Documentation**

Add comprehensive rustdoc to:
- All 10 helper functions
- ExecutorExt trait
- Usage examples in each function

**Verify:**
```bash
cargo doc --package airssys-osl --open
mdbook build airssys-osl/docs
mdbook serve airssys-osl/docs
```

---

#### **Phase 6: Final Validation (1-2 hours)**

**Afternoon - Part 3: Quality Gates**

**1. Run all tests:**
```bash
cargo test --workspace
```

**2. Check for warnings:**
```bash
cargo check --workspace --all-features
cargo clippy --workspace --all-targets --all-features
```

**3. Verify examples:**
```bash
cargo run --example basic_usage
cargo run --example logger_comprehensive
cargo run --example middleware_pipeline
```

**4. Build documentation:**
```bash
cargo doc --workspace --no-deps
mdbook build airssys-osl/docs
```

**5. Update memory bank:**
- Update `progress.md` - mark OSL-TASK-009 complete
- Update `tasks/_index.md` - update task status

**6. Git commit:**
```bash
git add -A
git commit -m "feat(osl): Complete OSL-TASK-009 - Remove framework, add helpers

- Delete framework layer (registry, framework, builder, pipeline)
- Add 10 helper functions for ergonomic APIs
- Add ExecutorExt trait for middleware composition
- Update all tests and examples
- Update documentation (README, mdBook)
- 165+ tests passing, zero warnings

Architecture simplification per refactoring plan.
Closes OSL-TASK-009"
```

---

## Important Notes

### 🔒 Security & Middleware Integration (Future)

**CRITICAL:** All helper functions implemented in Phase 2 are **temporary direct executor calls**. They will be enhanced in future tasks:

1. **OSL-TASK-003 (Security Middleware):**
   - Add security policy validation to all 10 helpers
   - Implement ACL/RBAC checks before execution
   - Add security audit logging

2. **OSL-TASK-004 (Middleware Pipeline):**
   - Wire all 10 helpers through middleware pipeline
   - Support composable middleware (logging, metrics, security)
   - Maintain backward-compatible simple APIs

**Current Implementation Philosophy:**
- ✅ Start simple: Direct executor calls
- ✅ Build foundation: Helper APIs with clean signatures
- ✅ Plan for future: Add TODO comments for integration points
- ✅ Maintain compatibility: APIs won't change, internals will

**Developer Note:**
When working on OSL-TASK-003 or OSL-TASK-004, search codebase for `TODO(OSL-TASK-003)` and `TODO(OSL-TASK-004)` comments to find all integration points.

---

## Task Checklist

### Phase 1: Framework Removal ✅
- [ ] Delete `framework/registry.rs`
- [ ] Delete `framework/framework.rs`
- [ ] Delete `framework/builder.rs`
- [ ] Delete `framework/pipeline.rs`
- [ ] Delete `framework/operations.rs`
- [ ] Update `lib.rs` - remove framework module
- [ ] Update `prelude.rs` - remove framework exports

### Phase 2: Helper Functions ✅
- [ ] Create `helpers.rs` module
- [ ] Add module documentation with future integration notes
- [ ] Implement 4 filesystem helpers (with TODO comments for OSL-TASK-003/004)
- [ ] Implement 3 process helpers (with TODO comments for OSL-TASK-003/004)
- [ ] Implement 3 network helpers (with TODO comments for OSL-TASK-003/004)
- [ ] Export from `lib.rs`
- [ ] Write 10+ helper tests
- [ ] All helper tests passing
- [ ] Document future integration requirements in rustdoc

### Phase 3: Middleware Extension ✅
- [ ] Create `middleware/ext.rs`
- [ ] Implement `ExecutorExt` trait
- [ ] Blanket implementation
- [ ] Export from `middleware/mod.rs`
- [ ] Write 3+ extension tests
- [ ] All extension tests passing

### Phase 4: Testing Updates ✅
- [ ] Update integration tests
- [ ] Update examples
- [ ] All 165+ tests passing
- [ ] Examples compile and run

### Phase 5: Documentation ✅
- [ ] Update README.md
- [ ] Create helper functions guide
- [ ] Create middleware composition guide
- [ ] Update custom executors guide
- [ ] Delete framework guide
- [ ] Comprehensive rustdoc
- [ ] mdBook builds successfully

### Phase 6: Quality Gates ✅
- [ ] All tests passing (165+)
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings
- [ ] Documentation builds
- [ ] Examples work
- [ ] Memory bank updated
- [ ] Git commit

---

## Success Criteria

### Must Have ✅
- [x] Framework code deleted
- [x] 10 helper functions implemented
- [x] ExecutorExt trait implemented
- [x] All 165+ tests passing
- [x] Zero warnings
- [x] Documentation updated

### Should Have ✅
- [x] Helper function tests (10+)
- [x] Extension trait tests (3+)
- [x] Examples updated
- [x] mdBook documentation complete

### Nice to Have 🎯
- [ ] Performance benchmarks
- [ ] Migration guide for external users
- [ ] Advanced examples

---

## File Changes Summary

### Files to DELETE ❌
1. `src/framework/registry.rs`
2. `src/framework/framework.rs`
3. `src/framework/builder.rs`
4. `src/framework/pipeline.rs`
5. `src/framework/operations.rs`
6. `src/framework/mod.rs` (possibly)
7. `docs/src/guides/framework-usage.md`

### Files to CREATE ✅
1. `src/helpers.rs` (new module)
2. `src/middleware/ext.rs` (extension trait)
3. `docs/src/guides/helper-functions.md`
4. `docs/src/guides/middleware-composition.md`

### Files to UPDATE 📝
1. `src/lib.rs` - module exports
2. `src/prelude.rs` - remove framework, add helpers
3. `src/middleware/mod.rs` - export ExecutorExt
4. `README.md` - new usage examples
5. `docs/src/guides/custom-executors.md` - remove framework
6. `tests/integration_tests.rs` - update tests
7. `examples/*.rs` - update examples

---

## Risk Mitigation

### Risk 1: Breaking Changes
- **Impact:** External users may depend on framework
- **Mitigation:** Document migration path, provide deprecation notice
- **Fallback:** Keep framework behind feature flag (not recommended)

### Risk 2: Test Failures
- **Impact:** Integration tests may fail after framework removal
- **Mitigation:** Update tests incrementally, verify at each step
- **Fallback:** Revert specific changes if critical tests fail

### Risk 3: Documentation Gaps
- **Impact:** Users may not understand new architecture
- **Mitigation:** Comprehensive examples and migration guide
- **Fallback:** Add more examples based on user feedback

---

## Estimated Timeline

| Day | Phase | Duration | Tasks |
|-----|-------|----------|-------|
| **Day 1 AM** | Phase 1 | 1-2h | Delete framework code |
| **Day 1 PM** | Phase 2 | 3-4h | Implement helpers |
| **Day 1 PM** | Phase 3 | 2h | Extension trait |
| **Day 2 AM** | Phase 4 | 2-3h | Update tests |
| **Day 2 PM** | Phase 5 | 2-3h | Documentation |
| **Day 2 PM** | Phase 6 | 1-2h | Final validation |

**Total:** 2-3 days (11-16 hours)

---

## Next Steps After Completion

### Immediate Follow-up Tasks

1. **OSL-TASK-003: Security Middleware** (2-3 days)
   - Implement security policy validation
   - **MUST UPDATE:** Integrate security checks into all 10 helper functions
   - Add ACL/RBAC support
   - Security audit logging

2. **OSL-TASK-004: Middleware Pipeline** (1-2 days)
   - Implement middleware pipeline orchestration
   - **MUST UPDATE:** Wire all 10 helper functions through pipeline
   - Support middleware composition (logging, metrics, security)
   - Maintain backward-compatible helper APIs

3. **Production Ready:** All core features complete

### Helper Functions Integration Checklist (For OSL-TASK-003 & 004)

When implementing OSL-TASK-003 and OSL-TASK-004, **ALL 10 helper functions** must be updated:

**Filesystem Helpers (4):**
- [ ] `read_file()` - Add security validation + pipeline
- [ ] `write_file()` - Add security validation + pipeline
- [ ] `delete_file()` - Add security validation + pipeline
- [ ] `create_directory()` - Add security validation + pipeline

**Process Helpers (3):**
- [ ] `spawn_process()` - Add security validation + pipeline
- [ ] `kill_process()` - Add security validation + pipeline
- [ ] `send_signal()` - Add security validation + pipeline

**Network Helpers (3):**
- [ ] `network_connect()` - Add security validation + pipeline
- [ ] `network_listen()` - Add security validation + pipeline
- [ ] `create_socket()` - Add security validation + pipeline

### Integration Pattern (Future Reference)

```rust
// Current (OSL-TASK-009):
pub async fn read_file(path: P, user: impl Into<String>) -> OSResult<Vec<u8>> {
    let executor = FilesystemExecutor::new();
    let result = executor.execute(operation, &context).await?;
    Ok(result.data)
}

// After OSL-TASK-003 + 004:
pub async fn read_file(path: P, user: impl Into<String>) -> OSResult<Vec<u8>> {
    // Security validation
    SecurityPolicy::validate(&operation, &context)?;
    
    // Wire through middleware pipeline
    let executor = FilesystemExecutor::new()
        .with_middleware(|e| SecurityMiddleware::new(e, policy))
        .with_middleware(|e| LoggerMiddleware::new(e, logger));
    
    let result = executor.execute(operation, &context).await?;
    Ok(result.data)
}
```

---

**Status:** 🎯 Ready to Start  
**Dependencies:** ✅ All resolved (OSL-TASK-008 complete)  
**Blocks:** OSL-TASK-003, OSL-TASK-004

**Let's simplify this codebase! 🚀**
