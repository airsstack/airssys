# OSL-TASK-010 Development Plan
# Helper Function Middleware Integration

**Task ID:** OSL-TASK-010  
**Status:** 🎯 Ready to Start  
**Priority:** High  
**Estimated Effort:** 1-2 days (8-16 hours)  
**Created:** 2025-10-10  
**Target Completion:** 2025-10-12

---

## Executive Summary

**Goal:** Integrate security validation and audit logging into all 10 helper functions in `src/helpers.rs` using the ExecutorExt middleware composition pattern.

**Why:** Helper functions currently bypass middleware (security + logging), making them unsafe for production use. This is the **final task** to complete airssys-osl.

**Impact:** Upon completion, airssys-osl reaches 100% production-ready status with:
- ✅ Security enforcement in all APIs (low-level + helpers)
- ✅ Complete audit logging
- ✅ Zero security bypass paths
- ✅ Production-grade quality

---

## Current State Analysis

### What We Have ✅

1. **SecurityMiddleware** (OSL-TASK-003 - COMPLETE)
   - ACL policy enforcement
   - RBAC policy enforcement  
   - Audit logging system
   - Policy composition (AND/OR/NOT)
   - Threat model validation
   - 66 integration tests passing

2. **ExecutorExt Trait** (OSL-TASK-009 - COMPLETE)
   - `.with_middleware()` extension method
   - MiddlewareExecutor wrapper
   - Type-safe composition
   - 5 integration tests passing

3. **10 Helper Functions** (OSL-TASK-009 - COMPLETE)
   - read_file, write_file, create_directory, delete_file
   - spawn_process, kill_process, send_signal
   - tcp_connect, tcp_listen, udp_socket

### What's Missing ❌

**Security Bypass Issue:**
```rust
// ❌ Current: Helpers bypass middleware
pub async fn read_file<P: AsRef<Path>>(path: P, user: impl Into<String>) -> OSResult<Vec<u8>> {
    // TODO(OSL-TASK-003): Add security validation here
    // TODO(OSL-TASK-004): Wire through middleware pipeline
    let executor = FilesystemExecutor::new();  // Direct - no security!
    let result = executor.execute(operation, &context).await?;
    Ok(result.output)
}
```

**Impact:**
- ❌ No ACL/RBAC enforcement
- ❌ No audit logging
- ❌ No permission validation
- ❌ Production deployment blocked

### Technical Debt
- **20 TODO comments** across 10 functions (2 per function)
- **10 functions** need security integration
- **0 integration tests** for helper security enforcement

---

## Development Phases

### Phase 1: Design & Setup (2-3 hours)

#### 1.1 Choose Implementation Approach (30 min)

**Decision:** Use **Hybrid Approach** (Approach C from task spec)

```rust
// Simple default API
pub async fn read_file<P: AsRef<Path>>(path: P, user: impl Into<String>) -> OSResult<Vec<u8>> {
    read_file_with_middleware(path, user, default_middleware_stack()).await
}

// Advanced API with custom middleware
pub async fn read_file_with_middleware<P, M>(
    path: P,
    user: impl Into<String>, 
    middleware: M,
) -> OSResult<Vec<u8>>
where
    P: AsRef<Path>,
    M: Middleware<FileReadOperation>,
{
    let path_str = path.as_ref().display().to_string();
    let operation = FileReadOperation::new(path_str);
    let context = ExecutionContext::new(SecurityContext::new(user.into()));
    let executor = FilesystemExecutor::new().with_middleware(middleware);
    let result = executor.execute(operation, &context).await?;
    Ok(result.output)
}
```

**Rationale:**
- ✅ Simple default for 90% use cases
- ✅ Power-user flexibility when needed
- ✅ No global state complexity
- ✅ Easy to test both paths

#### 1.2 Create Middleware Stack Factory (1 hour)

**Location:** `airssys-osl/src/helpers.rs` (top of module)

```rust
use std::sync::Arc;
use crate::middleware::security::{
    SecurityMiddleware, AccessControlList, RoleBasedAccessControl,
    ConsoleSecurityAuditLogger, AclEntry, Permission,
};
use crate::middleware::logger::LoggerMiddleware;

/// Default middleware stack for helper functions.
/// 
/// Provides:
/// - ACL policy enforcement (default deny-all)
/// - RBAC policy enforcement (role-based access)
/// - Security audit logging (console output)
///
/// # Security Model
/// - Deny by default (no policies = deny)
/// - User must have explicit permission
/// - All operations audited
fn default_security_middleware() -> SecurityMiddleware {
    SecurityMiddleware::builder()
        .with_acl_policy(default_acl_policy())
        .with_rbac_policy(default_rbac_policy())
        .with_audit_logger(Arc::new(ConsoleSecurityAuditLogger::new()))
        .build()
}

/// Default ACL policy for helpers.
/// 
/// **Important:** This is deny-by-default. Users must configure
/// their own ACL policies for production use.
fn default_acl_policy() -> AccessControlList {
    AccessControlList::new()
        // Example: Allow admin full access
        .with_entry(AclEntry::allow("admin", "*", vec!["*".to_string()]))
        // Add more default rules as needed
}

/// Default RBAC policy for helpers.
fn default_rbac_policy() -> RoleBasedAccessControl {
    RoleBasedAccessControl::new()
        // Example: admin role
        .with_role("admin", vec![
            "file:read", "file:write", "file:delete",
            "process:spawn", "process:kill", "process:signal",
            "network:connect", "network:listen", "network:socket"
        ])
        // Add more default roles
}
```

**Deliverables:**
- ✅ `default_security_middleware()` function
- ✅ `default_acl_policy()` function  
- ✅ `default_rbac_policy()` function
- ✅ Comprehensive rustdoc explaining security model

#### 1.3 Update Module Documentation (30 min)

**Location:** `airssys-osl/src/helpers.rs` (module header)

```rust
//! High-level convenience functions for common OS operations.
//!
//! # Security and Middleware Integration
//!
//! **All helper functions enforce security policies by default.**
//!
//! ## Security Model
//!
//! Helpers use a **deny-by-default** security model:
//! - ACL policy enforcement (glob pattern matching)
//! - RBAC policy enforcement (role-based access)  
//! - Security audit logging (all operations logged)
//! - Context-aware validation (user identity required)
//!
//! ## Basic Usage (Default Security)
//!
//! ```rust,no_run
//! use airssys_osl::helpers::*;
//!
//! # async fn example() -> airssys_osl::core::result::OSResult<()> {
//! // Uses default security middleware
//! let data = read_file("/etc/hosts", "admin").await?;
//! println!("Read {} bytes", data.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Custom Middleware (Advanced)
//!
//! ```rust,no_run
//! use airssys_osl::helpers::*;
//! use airssys_osl::middleware::security::*;
//!
//! # async fn example() -> airssys_osl::core::result::OSResult<()> {
//! // Custom ACL policy
//! let acl = AccessControlList::new()
//!     .with_entry(AclEntry::allow("alice", "/data/*", vec!["read".to_string()]));
//!
//! let security = SecurityMiddleware::builder()
//!     .with_acl_policy(acl)
//!     .build();
//!
//! // Use custom middleware
//! let data = read_file_with_middleware("/data/file.txt", "alice", security).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Security Considerations
//!
//! - **Deny-by-default:** No policies = access denied
//! - **Audit everything:** All operations logged with user context
//! - **Policy violations:** Return `OSError::SecurityViolation`
//! - **Production use:** Configure ACL/RBAC policies for your environment
```

**Deliverables:**
- ✅ Updated module-level documentation
- ✅ Security model explanation
- ✅ Basic and advanced usage examples
- ✅ Security considerations documented

---

### Phase 2: Filesystem Helpers Implementation (2-3 hours)

#### 2.1 Update read_file() (30 min)

**Files:** `airssys-osl/src/helpers.rs`

```rust
/// Read file contents with security validation.
///
/// # Security
/// - Enforces ACL/RBAC policies
/// - Generates audit log
/// - Requires user context
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let data = read_file("/etc/hosts", "admin").await?;
/// println!("Read {} bytes", data.len());
/// # Ok(())
/// # }
/// ```
///
/// # Errors
/// - `OSError::SecurityViolation` if ACL/RBAC denies access
/// - `OSError::NotFound` if file doesn't exist
/// - `OSError::PermissionDenied` if OS denies access
pub async fn read_file<P: AsRef<Path>>(path: P, user: impl Into<String>) -> OSResult<Vec<u8>> {
    read_file_with_middleware(path, user, default_security_middleware()).await
}

/// Read file with custom middleware.
///
/// # Advanced Usage
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
/// use airssys_osl::middleware::security::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let acl = AccessControlList::new()
///     .with_entry(AclEntry::allow("alice", "/data/*", vec!["read".to_string()]));
/// let security = SecurityMiddleware::builder().with_acl_policy(acl).build();
///
/// let data = read_file_with_middleware("/data/file.txt", "alice", security).await?;
/// # Ok(())
/// # }
/// ```
pub async fn read_file_with_middleware<P, M>(
    path: P,
    user: impl Into<String>,
    middleware: M,
) -> OSResult<Vec<u8>>
where
    P: AsRef<Path>,
    M: Middleware<FileReadOperation> + Send + Sync + std::fmt::Debug + 'static,
{
    use crate::middleware::ext::ExecutorExt;
    
    let path_str = path.as_ref().display().to_string();
    let operation = FileReadOperation::new(path_str);
    let context = ExecutionContext::new(SecurityContext::new(user.into()));
    let executor = FilesystemExecutor::new().with_middleware(middleware);
    let result = executor.execute(operation, &context).await?;
    Ok(result.output)
}
```

**Tasks:**
- ✅ Update `read_file()` to use default middleware
- ✅ Add `read_file_with_middleware()` variant
- ✅ Update rustdoc with security examples
- ✅ Remove TODO comments
- ✅ Add error documentation

#### 2.2 Update write_file() (30 min)
#### 2.3 Update create_directory() (30 min)
#### 2.4 Update delete_file() (30 min)

**Same pattern as read_file()** - Apply to all filesystem helpers

---

### Phase 3: Process Helpers Implementation (2-3 hours)

#### 3.1 Update spawn_process() (30 min)

```rust
/// Spawn process with security validation.
///
/// # Security
/// - Enforces ACL/RBAC policies for process:spawn permission
/// - Validates program execution rights
/// - Generates audit log with process details
///
/// # Example
///
/// ```rust,no_run
/// use airssys_osl::helpers::*;
///
/// # async fn example() -> airssys_osl::core::result::OSResult<()> {
/// let pid_bytes = spawn_process("ls", vec!["-la".to_string()], "admin").await?;
/// let pid = String::from_utf8_lossy(&pid_bytes).parse::<u32>().unwrap();
/// println!("Spawned PID: {}", pid);
/// # Ok(())
/// # }
/// ```
pub async fn spawn_process(
    program: impl Into<String>,
    args: Vec<String>,
    user: impl Into<String>,
) -> OSResult<Vec<u8>> {
    spawn_process_with_middleware(program, args, user, default_security_middleware()).await
}

/// Spawn process with custom middleware.
pub async fn spawn_process_with_middleware<M>(
    program: impl Into<String>,
    args: Vec<String>,
    user: impl Into<String>,
    middleware: M,
) -> OSResult<Vec<u8>>
where
    M: Middleware<ProcessSpawnOperation> + Send + Sync + std::fmt::Debug + 'static,
{
    use crate::middleware::ext::ExecutorExt;
    
    let operation = ProcessSpawnOperation::new(program).with_args(args);
    let context = ExecutionContext::new(SecurityContext::new(user.into()));
    let executor = ProcessExecutor::new("helper_executor").with_middleware(middleware);
    let result = executor.execute(operation, &context).await?;
    Ok(result.output)
}
```

#### 3.2 Update kill_process() (30 min)
#### 3.3 Update send_signal() (30 min)

**Same pattern** - Apply to all process helpers

---

### Phase 4: Network Helpers Implementation (2-3 hours)

#### 4.1 Update tcp_connect() (30 min)
#### 4.2 Update tcp_listen() (30 min)
#### 4.3 Update udp_socket() (30 min)

**Same pattern** - Apply to all network helpers

---

### Phase 5: Integration Testing (3-4 hours)

#### 5.1 Security Enforcement Tests (2 hours)

**File:** `airssys-osl/tests/helpers_security_tests.rs` (new file)

```rust
//! Integration tests for helper function security enforcement.

use airssys_osl::helpers::*;
use airssys_osl::middleware::security::*;
use airssys_osl::core::result::OSError;
use std::sync::Arc;

#[tokio::test]
async fn test_read_file_acl_allow() {
    let acl = AccessControlList::new()
        .with_entry(AclEntry::allow("alice", "/tmp/*", vec!["read".to_string()]));
    
    let security = SecurityMiddleware::builder()
        .with_acl_policy(acl)
        .with_audit_logger(Arc::new(ConsoleSecurityAuditLogger::new()))
        .build();
    
    // Should succeed - ACL allows
    let result = read_file_with_middleware("/tmp/test.txt", "alice", security).await;
    // Note: May fail if file doesn't exist, but should not be security violation
    match result {
        Ok(_) => (),
        Err(OSError::NotFound { .. }) => (), // File doesn't exist - OK
        Err(e) => panic!("Expected success or NotFound, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_read_file_acl_deny() {
    let acl = AccessControlList::new()
        .with_entry(AclEntry::deny("bob", "/secret/*", vec!["*".to_string()]));
    
    let security = SecurityMiddleware::builder()
        .with_acl_policy(acl)
        .build();
    
    // Should fail - ACL denies
    let result = read_file_with_middleware("/secret/data.txt", "bob", security).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), OSError::SecurityViolation { .. }));
}

#[tokio::test]
async fn test_spawn_process_rbac_enforcement() {
    let rbac = RoleBasedAccessControl::new()
        .with_role("reader", vec!["file:read"])
        .with_user_role("charlie", "reader");
    
    let security = SecurityMiddleware::builder()
        .with_rbac_policy(rbac)
        .build();
    
    // Should fail - reader role cannot spawn processes
    let result = spawn_process_with_middleware("ls", vec![], "charlie", security).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_write_file_combined_policies() {
    // ACL allows, RBAC denies = should deny (ANY deny = deny)
    let acl = AccessControlList::new()
        .with_entry(AclEntry::allow("dave", "/data/*", vec!["write".to_string()]));
    
    let rbac = RoleBasedAccessControl::new()
        .with_role("viewer", vec!["file:read"])
        .with_user_role("dave", "viewer");
    
    let security = SecurityMiddleware::builder()
        .with_acl_policy(acl)
        .with_rbac_policy(rbac)
        .build();
    
    let result = write_file_with_middleware("/data/test.txt", b"data", "dave", security).await;
    assert!(result.is_err()); // RBAC denies write
}

// Add 6 more tests covering all helpers...
```

**Test Coverage:**
- ✅ ACL allow scenarios (3 tests)
- ✅ ACL deny scenarios (3 tests)
- ✅ RBAC allow scenarios (3 tests)
- ✅ RBAC deny scenarios (3 tests)
- ✅ Combined policy tests (3 tests)
- ✅ All 10 helpers covered (10 tests)

**Total:** ~25 new integration tests

#### 5.2 Audit Logging Tests (1 hour)

**File:** `airssys-osl/tests/helpers_audit_tests.rs` (new file)

```rust
//! Integration tests for helper audit logging.

use airssys_osl::helpers::*;
use airssys_osl::middleware::security::*;
use std::sync::Arc;

// Test audit logger that captures events
struct TestAuditLogger {
    events: Arc<parking_lot::Mutex<Vec<SecurityAuditLog>>>,
}

impl TestAuditLogger {
    fn new() -> Self {
        Self {
            events: Arc::new(parking_lot::Mutex::new(Vec::new())),
        }
    }
    
    fn get_events(&self) -> Vec<SecurityAuditLog> {
        self.events.lock().clone()
    }
}

impl SecurityAuditLogger for TestAuditLogger {
    fn log(&self, event: SecurityAuditLog) {
        self.events.lock().push(event);
    }
}

#[tokio::test]
async fn test_read_file_audit_logging() {
    let logger = Arc::new(TestAuditLogger::new());
    let security = SecurityMiddleware::builder()
        .with_audit_logger(logger.clone() as Arc<dyn SecurityAuditLogger>)
        .build();
    
    let _ = read_file_with_middleware("/tmp/test.txt", "admin", security).await;
    
    let events = logger.get_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].user, "admin");
    assert_eq!(events[0].resource, "/tmp/test.txt");
}

// Add 5 more audit logging tests...
```

**Test Coverage:**
- ✅ Audit log creation (5 tests)
- ✅ Event metadata capture (5 tests)
- ✅ Policy decision logging (5 tests)

**Total:** ~15 new audit tests

#### 5.3 Error Handling Tests (1 hour)

**Test scenarios:**
- Policy violation errors
- OS-level errors (file not found, permission denied)
- Error context preservation
- Error message quality

**Total:** ~10 new error handling tests

---

### Phase 6: Documentation (2-3 hours)

#### 6.1 Update Rustdoc for All Helpers (1.5 hours)

**For each helper function:**
- ✅ Security model explanation
- ✅ ACL/RBAC enforcement details
- ✅ Audit logging behavior
- ✅ Error scenarios
- ✅ Basic usage example
- ✅ Advanced usage example (custom middleware)

#### 6.2 Create Comprehensive Example (1 hour)

**File:** `airssys-osl/examples/helpers_with_security.rs` (new file)

```rust
//! Comprehensive example of using helper functions with security middleware.
//!
//! This example demonstrates:
//! - Default security enforcement
//! - Custom ACL policies
//! - Custom RBAC policies
//! - Audit logging
//! - Error handling

use airssys_osl::helpers::*;
use airssys_osl::middleware::security::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Helper Functions with Security Middleware ===\n");
    
    // Example 1: Default security (deny-by-default)
    println!("Example 1: Default Security");
    match read_file("/etc/hosts", "admin").await {
        Ok(data) => println!("  ✅ Admin read {} bytes", data.len()),
        Err(e) => println!("  ❌ Error: {:?}", e),
    }
    
    // Example 2: Custom ACL policy
    println!("\nExample 2: Custom ACL Policy");
    let acl = AccessControlList::new()
        .with_entry(AclEntry::allow("alice", "/data/*", vec!["read".to_string(), "write".to_string()]))
        .with_entry(AclEntry::deny("bob", "/secret/*", vec!["*".to_string()]));
    
    let security = SecurityMiddleware::builder()
        .with_acl_policy(acl)
        .with_audit_logger(Arc::new(ConsoleSecurityAuditLogger::new()))
        .build();
    
    match read_file_with_middleware("/data/file.txt", "alice", security.clone()).await {
        Ok(data) => println!("  ✅ Alice read {} bytes from /data/file.txt", data.len()),
        Err(e) => println!("  ❌ Alice denied: {:?}", e),
    }
    
    match read_file_with_middleware("/secret/data.txt", "bob", security).await {
        Ok(_) => println!("  ❌ Bob should have been denied!"),
        Err(e) => println!("  ✅ Bob correctly denied: {:?}", e),
    }
    
    // Example 3: RBAC with roles
    println!("\nExample 3: RBAC Policies");
    let rbac = RoleBasedAccessControl::new()
        .with_role("admin", vec![
            "file:read", "file:write", "file:delete",
            "process:spawn", "process:kill",
        ])
        .with_role("user", vec!["file:read"])
        .with_user_role("charlie", "admin")
        .with_user_role("dave", "user");
    
    let security = SecurityMiddleware::builder()
        .with_rbac_policy(rbac)
        .build();
    
    match spawn_process_with_middleware("ls", vec!["-la".to_string()], "charlie", security.clone()).await {
        Ok(_) => println!("  ✅ Admin charlie spawned process"),
        Err(e) => println!("  ❌ Charlie denied: {:?}", e),
    }
    
    match spawn_process_with_middleware("ls", vec![], "dave", security).await {
        Ok(_) => println!("  ❌ User dave should not spawn processes!"),
        Err(e) => println!("  ✅ User dave correctly denied: {:?}", e),
    }
    
    println!("\n=== All examples complete ===");
    Ok(())
}
```

#### 6.3 Update README (30 min)

**File:** `airssys-osl/README.md`

Add section:
```markdown
### Security-First Helper Functions

All helper functions enforce security policies by default:

\`\`\`rust
use airssys_osl::helpers::*;

// Secure by default
let data = read_file("/etc/hosts", "admin").await?;

// Custom security policies
use airssys_osl::middleware::security::*;

let acl = AccessControlList::new()
    .with_entry(AclEntry::allow("alice", "/data/*", vec!["read"]));

let security = SecurityMiddleware::builder()
    .with_acl_policy(acl)
    .build();

let data = read_file_with_middleware("/data/file.txt", "alice", security).await?;
\`\`\`

See `examples/helpers_with_security.rs` for comprehensive examples.
```

---

### Phase 7: Quality Assurance & Polish (1-2 hours)

#### 7.1 Code Quality Checks (30 min)

```bash
# Run all checks
cargo check --package airssys-osl
cargo test --package airssys-osl
cargo clippy --package airssys-osl --all-targets --all-features
cargo doc --package airssys-osl --no-deps
```

**Acceptance:**
- ✅ Zero compiler warnings
- ✅ Zero clippy warnings
- ✅ All tests passing (311 + 50 new = 361 tests)
- ✅ All doctests compile
- ✅ Documentation builds cleanly

#### 7.2 Performance Validation (30 min)

**Verify middleware overhead:**
```rust
// Benchmark helper with middleware vs direct executor
use std::time::Instant;

let start = Instant::now();
for _ in 0..1000 {
    let _ = read_file("/tmp/test.txt", "admin").await;
}
let duration = start.elapsed();
println!("1000 operations: {:?} ({:?} per op)", duration, duration / 1000);
```

**Target:** <1ms overhead per operation

#### 7.3 Final Review (30 min)

**Checklist:**
- [ ] All 10 helpers updated
- [ ] All 20 TODO comments removed
- [ ] 50+ new tests added
- [ ] All tests passing
- [ ] Zero warnings
- [ ] Documentation complete
- [ ] Examples working
- [ ] Performance acceptable

---

## Timeline & Milestones

### Day 1 (6-8 hours)
- ✅ **Morning:** Phase 1 (Design & Setup) - 2-3 hours
- ✅ **Afternoon:** Phase 2 (Filesystem Helpers) - 2-3 hours
- ✅ **Evening:** Phase 3 (Process Helpers) - 2-3 hours

**Milestone 1:** Core integration complete

### Day 2 (6-8 hours)
- ✅ **Morning:** Phase 4 (Network Helpers) - 2-3 hours
- ✅ **Afternoon:** Phase 5 (Testing) - 3-4 hours
- ✅ **Evening:** Phase 6-7 (Docs & Polish) - 2-3 hours

**Milestone 2:** Production-ready delivery

---

## Risk Management

### Risk 1: Type System Complexity
**Issue:** Middleware trait bounds might not work with all helper signatures  
**Mitigation:** Use concrete types or trait objects where needed  
**Backup:** Simplify to single middleware type if generics fail

### Risk 2: Performance Overhead
**Issue:** Middleware composition adds latency  
**Mitigation:** Benchmark early, optimize if needed  
**Backup:** Make middleware optional via feature flag

### Risk 3: Breaking Changes
**Issue:** Existing helper users might break  
**Mitigation:** Provide both APIs (with/without middleware)  
**Backup:** Version bump to 0.2.0 if needed

---

## Success Criteria

### Functional Requirements
- [x] All 10 helpers use ExecutorExt middleware composition
- [x] Security validation works (ACL/RBAC enforced)
- [x] Audit logging captures all operations
- [x] Error handling preserves context

### Quality Requirements
- [x] All 20 TODO comments removed
- [x] 50+ new integration tests
- [x] 361+ total tests passing
- [x] >95% code coverage maintained
- [x] Zero compiler warnings
- [x] Zero clippy warnings
- [x] All doctests passing

### Documentation Requirements
- [x] Rustdoc updated for all helpers
- [x] Comprehensive examples added
- [x] README updated
- [x] Security model documented

### Performance Requirements
- [x] <1ms middleware overhead
- [x] No memory leaks
- [x] Efficient error propagation

---

## Completion Checklist

### Code Changes
- [ ] 10 helper functions updated (simple + _with_middleware variants)
- [ ] Default middleware stack factory created
- [ ] Module documentation updated
- [ ] 20 TODO comments removed
- [ ] All imports added (ExecutorExt, SecurityMiddleware, etc.)

### Testing
- [ ] 25 security enforcement tests
- [ ] 15 audit logging tests
- [ ] 10 error handling tests
- [ ] All tests passing
- [ ] No test warnings

### Documentation
- [ ] Rustdoc for all 10 helpers
- [ ] Module-level security documentation
- [ ] `examples/helpers_with_security.rs` created
- [ ] README updated
- [ ] Migration guide (if needed)

### Quality Gates
- [ ] `cargo check` - zero errors
- [ ] `cargo test` - all passing
- [ ] `cargo clippy` - zero warnings
- [ ] `cargo doc` - builds cleanly
- [ ] Performance validation passed

### Final Steps
- [ ] Code review
- [ ] Git commit with clear message
- [ ] Update progress.md to 100%
- [ ] Update _index.md (9/10 → 10/10 complete)
- [ ] Mark OSL-TASK-010 as COMPLETE
- [ ] Celebrate! 🎉

---

## Post-Completion

Upon OSL-TASK-010 completion:
- **airssys-osl:** 100% production-ready
- **Total tasks:** 10 (9 complete + 1 abandoned = 90% success rate)
- **Test coverage:** 361+ tests
- **Quality:** Zero warnings, full security enforcement
- **Status:** Ready for production deployment

**Next steps:**
- Integration with airssys-rt
- Real-world deployment testing
- Performance benchmarking at scale
- Community feedback and iteration

---

## Notes

### Design Decisions Made
1. **Hybrid API approach:** Simple default + advanced variant
2. **Deny-by-default security:** No policies = deny access
3. **Console audit logging:** Default output, can override
4. **No global state:** Each call gets middleware instance

### Implementation Patterns
- Use `ExecutorExt::with_middleware()` for composition
- Clone middleware where needed (Arc for policies)
- Preserve error context through middleware chain
- Type-safe generics for middleware parameters

### Testing Strategy
- Integration tests in separate files
- Test both allow and deny scenarios
- Verify audit logs capture all events
- Edge cases: empty policies, multiple policies, conflicts

---

**This development plan provides a complete roadmap to finish OSL-TASK-010 and achieve 100% production-ready status for airssys-osl!**
