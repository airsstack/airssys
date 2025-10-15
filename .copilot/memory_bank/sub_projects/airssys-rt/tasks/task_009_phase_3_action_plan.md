# RT-TASK-009 Phase 3: Security and Audit Integration - Action Plan

**Created:** 2025-10-14  
**Updated:** 2025-10-15 (Architecture Alignment with OSL Middleware)  
**Status:** Ready for Implementation  
**Phase:** Phase 3 of 4 (OSL Integration)  
**Prerequisites:** Phase 2 Complete (100% ✅)  
**Duration Estimate:** 1.5 days (12 hours) - Reduced due to OSL middleware reuse

---

## Executive Summary

**Objective:** Integrate RT actors with airssys-osl's existing middleware infrastructure (SecurityMiddleware and SecurityAuditLogger) to enable security policy enforcement and comprehensive audit trails for all system operations.

**Key Architectural Principle:** ✅ **REUSE OSL Middleware - DO NOT DUPLICATE**

**Key Deliverables:**
1. SecurityContext propagation through RT message protocol
2. Integration with OSL SecurityMiddleware for policy enforcement
3. Integration with OSL SecurityAuditLogger for audit trails
4. Security-focused integration tests
5. Documentation updates showing middleware integration patterns

**Critical Dependencies:**
- ✅ airssys-osl SecurityContext (VERIFIED: `src/core/context.rs`)
- ✅ airssys-osl SecurityMiddleware (VERIFIED: `src/middleware/security/middleware.rs`)
- ✅ airssys-osl SecurityAuditLogger (VERIFIED: `src/middleware/security/audit.rs`)
- ✅ airssys-osl helper functions with middleware support (VERIFIED: OSL-TASK-010 complete)

---

## Phase 3 Overview

### Goals
- Enable security context flow: Application actors → RT OSL actors → OSL ExecutionContext → OSL Middleware
- Leverage OSL SecurityMiddleware for automatic policy enforcement
- Leverage OSL SecurityAuditLogger for automatic audit logging
- Validate security boundaries through OSL middleware pipeline
- Document RT-to-OSL middleware integration patterns

### Success Criteria
- ✅ SecurityContext propagates: RT messages → OSL ExecutionContext
- ✅ OSL SecurityMiddleware enforces policies on RT-initiated operations
- ✅ OSL SecurityAuditLogger logs all security decisions automatically
- ✅ Integration tests validate end-to-end security flow
- ✅ Zero duplication of security/audit infrastructure
- ✅ Clean architectural alignment with OSL middleware layer
- ✅ Zero security warnings from review checklist


---

## Task Breakdown

### Task 3.1: SecurityContext Propagation Through Messages (Day 7, ~6 hours)

**Architecture:** Import and use `airssys_osl::core::context::SecurityContext` - DO NOT create new types.

#### Subtask 3.1.1: Import SecurityContext from OSL (1 hour)
**Files Modified:** 
- `src/osl/actors/messages.rs` (~50 lines modified)
- `Cargo.toml` (verify airssys-osl dependency)

**Implementation:**

```rust
// File: src/osl/actors/messages.rs

// Layer 1: Standard library imports
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
use crate::message::{Message, MessageId};

// ← NEW: Import SecurityContext from OSL
use airssys_osl::core::context::SecurityContext;

// Update all *Request types with security_context field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemRequest {
    pub operation: FileSystemOperation,
    pub request_id: MessageId,
    pub security_context: Option<SecurityContext>,  // ← NEW
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessRequest {
    pub operation: ProcessOperation,
    pub request_id: MessageId,
    pub security_context: Option<SecurityContext>,  // ← NEW
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    pub operation: NetworkOperation,
    pub request_id: MessageId,
    pub security_context: Option<SecurityContext>,  // ← NEW
}
```

**Key Design Decisions:**
- ✅ **Reuse OSL type** - No new SecurityContext implementation
- ✅ **Option type** - Backward compatible (None = default system context)
- ✅ **Clone trait** - SecurityContext already implements Clone in OSL
- ✅ **Serializable** - SecurityContext already implements Serialize/Deserialize

**Acceptance Criteria:**
- ✅ All request types updated with security_context field
- ✅ Imports from airssys_osl::core::context::SecurityContext
- ✅ No breaking changes to existing Phase 1/2 tests
- ✅ Message serialization/deserialization works

#### Subtask 3.1.2: Pass SecurityContext to OSL ExecutionContext (3 hours)
**Files Modified:**
- `src/osl/actors/filesystem.rs` (~100 lines modified)
- `src/osl/actors/process.rs` (~100 lines modified)
- `src/osl/actors/network.rs` (~100 lines modified)

**Implementation Pattern:**

```rust
// File: src/osl/actors/filesystem.rs

// Add imports
use airssys_osl::core::context::{ExecutionContext, SecurityContext};
use airssys_osl::helpers::filesystem::read_file_with_middleware;
use airssys_osl::middleware::security::{SecurityMiddleware, SecurityMiddlewareBuilder};

impl FileSystemActor {
    async fn execute_operation(
        &mut self,
        operation: FileSystemOperation,
        security_context: Option<SecurityContext>,  // ← NEW parameter
    ) -> FileSystemResponse {
        // Build ExecutionContext with SecurityContext
        let mut exec_context = ExecutionContext::new();
        if let Some(ctx) = security_context.clone() {
            exec_context = exec_context.with_security_context(ctx);
        }
        
        match operation {
            FileSystemOperation::ReadFile { path } => {
                // Option A: Use OSL helper with middleware (if available)
                // This automatically goes through SecurityMiddleware!
                match read_file_with_middleware(&path, &exec_context, self.security_middleware.clone()).await {
                    Ok(content) => FileSystemResponse::FileContent(content),
                    Err(e) => FileSystemResponse::Error(e.to_string()),
                }
                
                // Option B: Build operation and execute with middleware manually
                // let op = ReadFileOperation::new(path);
                // match self.executor.execute_with_middleware(op, &exec_context, &self.middleware_pipeline).await {
                //     Ok(result) => FileSystemResponse::FileContent(result),
                //     Err(e) => FileSystemResponse::Error(e.to_string()),
                // }
            }
            FileSystemOperation::WriteFile { path, content } => {
                // Similar pattern - OSL middleware handles security automatically
                match write_file_with_middleware(&path, content, &exec_context, self.security_middleware.clone()).await {
                    Ok(_) => FileSystemResponse::Success,
                    Err(e) => FileSystemResponse::Error(e.to_string()),
                }
            }
            // ... other operations follow same pattern
        }
    }
}

// Update handle_message to extract and pass context
#[async_trait]
impl<M, B> Actor<M, B> for FileSystemActor
where
    M: Message,
    B: MessageBroker<M>,
{
    async fn handle_message(&mut self, message: M, context: &mut ActorContext<M, B>) {
        if let Some(request) = message.downcast_ref::<FileSystemRequest>() {
            let response = self.execute_operation(
                request.operation.clone(),
                request.security_context.clone(),  // ← Pass context
            ).await;
            
            // Publish response through broker
            let response_msg = FileSystemResponse::from(response);
            if let Err(e) = context.broker().publish(response_msg).await {
                eprintln!("Failed to publish response: {e}");
            }
        }
    }
}
```

**Apply pattern to all three actors:**
- FileSystemActor: Pass context to OSL filesystem helpers
- ProcessActor: Pass context to OSL process helpers  
- NetworkActor: Pass context to OSL network helpers

**Key Points:**
- ✅ **NO manual permission validation** - OSL SecurityMiddleware does this
- ✅ **NO manual audit logging** - OSL SecurityAuditLogger does this
- ✅ **Thin integration layer** - RT actors just pass context to OSL

**Acceptance Criteria:**
- ✅ All actors extract security_context from messages
- ✅ ExecutionContext built with SecurityContext
- ✅ Context passed to OSL helpers/operations
- ✅ Existing tests still pass (with None context)
- ✅ No duplication of security logic

#### Subtask 3.1.3: Integration Verification (2 hours)
**Testing Approach:**

1. **Context Flow Test:**
   - Create SecurityContext in application actor
   - Send in message to RT OSL actor
   - Verify context reaches OSL ExecutionContext
   - Verify OSL SecurityMiddleware receives context

2. **Middleware Integration Test:**
   - Configure SecurityMiddleware with RBAC policy
   - Send operation with valid permissions
   - Verify operation succeeds
   - Send operation with insufficient permissions
   - Verify OSL SecurityMiddleware denies

3. **Cross-Actor Test:**
   - Verify context preserved across actor message passing
   - Test broker-based message routing with context

**Acceptance Criteria:**
- ✅ Security context flows: RT message → OSL ExecutionContext
- ✅ OSL SecurityMiddleware enforces policies correctly
- ✅ No context leaks or bypasses
- ✅ Documentation updated with context flow diagrams

**Implementation:**

```rust
// Security context structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// User or service identity
    pub principal: Principal,
    
    /// Granted permissions
    pub permissions: Vec<Permission>,
    
    /// Session metadata
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    
    /// Additional context
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principal {
    pub user_id: String,
    pub user_name: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,  // e.g., "file:read", "process:spawn"
    pub action: String,
    pub constraints: Option<PermissionConstraints>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionConstraints {
    pub paths: Option<Vec<PathBuf>>,      // Allowed paths
    pub commands: Option<Vec<String>>,    // Allowed commands
    pub hosts: Option<Vec<String>>,       // Allowed network hosts
}

impl SecurityContext {
    pub fn new(principal: Principal) -> Self { ... }
    pub fn with_permission(mut self, permission: Permission) -> Self { ... }
    pub fn has_permission(&self, resource: &str, action: &str) -> bool { ... }
    pub fn validate(&self) -> Result<(), SecurityError> { ... }
}
```

**Key Design Decisions:**
- Use `Arc<SecurityContext>` internally if needed for cheap cloning
- Ensure full `Clone` compatibility (required by Message trait)
- Integrate with airssys-osl security types (adapter pattern if needed)
- Serializable for message passing

**Acceptance Criteria:**
- ✅ SecurityContext implements Clone + Serialize + Deserialize
- ✅ Permission validation methods working
- ✅ Compatible with OSL security infrastructure
- ✅ Documentation with usage examples

#### Subtask 3.1.2: Message Protocol Updates (2 hours)
**Files:** `src/osl/actors/messages.rs` (modify existing)

**Changes:**

```rust
// Update all *Request types with security context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemRequest {
    pub operation: FileSystemOperation,
    pub request_id: MessageId,
    pub security_context: Option<SecurityContext>,  // ← NEW
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessRequest {
    pub operation: ProcessOperation,
    pub request_id: MessageId,
    pub security_context: Option<SecurityContext>,  // ← NEW
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    pub operation: NetworkOperation,
    pub request_id: MessageId,
    pub security_context: Option<SecurityContext>,  // ← NEW
}
```

**Backward Compatibility:**
- `Option<SecurityContext>` allows gradual migration
- None = use default/system context
- Some = enforce provided context

**Acceptance Criteria:**
- ✅ All request types updated with security_context field
- ✅ Backward compatible (Option type)
- ✅ No breaking changes to existing tests
- ✅ Message serialization/deserialization works

#### Subtask 3.1.3: Actor Implementation Updates (3 hours)
**Files:** 
- `src/osl/actors/filesystem.rs` (modify)
- `src/osl/actors/process.rs` (modify)
- `src/osl/actors/network.rs` (modify)

**Changes to each actor:**

```rust
impl FileSystemActor {
    async fn execute_operation(
        &mut self,
        operation: FileSystemOperation,
        security_context: Option<SecurityContext>,  // ← NEW parameter
    ) -> FileSystemResponse {
        // 1. Validate security context
        if let Some(ref ctx) = security_context {
            if let Err(e) = self.validate_permission(ctx, &operation) {
                return FileSystemResponse::Error(e.to_string());
            }
        }
        
        // 2. Execute operation with context
        match operation {
            FileSystemOperation::ReadFile { path } => {
                // Pass context to OSL helper
                match airssys_osl::helpers::read_file_with_context(&path, security_context) {
                    Ok(content) => FileSystemResponse::FileContent(content),
                    Err(e) => FileSystemResponse::Error(e.to_string()),
                }
            }
            // ... other operations
        }
    }
    
    fn validate_permission(
        &self,
        ctx: &SecurityContext,
        operation: &FileSystemOperation,
    ) -> Result<(), SecurityError> {
        match operation {
            FileSystemOperation::ReadFile { path } => {
                ctx.has_permission("file:read", path.to_str().unwrap())
                    .then_some(())
                    .ok_or(SecurityError::PermissionDenied)
            }
            FileSystemOperation::WriteFile { path, .. } => {
                ctx.has_permission("file:write", path.to_str().unwrap())
                    .then_some(())
                    .ok_or(SecurityError::PermissionDenied)
            }
            // ... other operations
        }
    }
}

// Update handle_message to extract and pass context
#[async_trait]
impl<M, B> Actor<M, B> for FileSystemActor
where
    M: Message,
    B: MessageBroker<M>,
{
    async fn handle_message(&mut self, message: M, context: &mut ActorContext<M, B>) {
        if let Some(request) = message.downcast_ref::<FileSystemRequest>() {
            let response = self.execute_operation(
                request.operation.clone(),
                request.security_context.clone(),  // ← Pass context
            ).await;
            // ... publish response
        }
    }
}
```

**Apply to all three actors:**
- FileSystemActor: File/directory permission validation
- ProcessActor: Process spawn permission validation
- NetworkActor: Network connection permission validation

**Acceptance Criteria:**
- ✅ All actors accept security_context parameter
- ✅ Permission validation before operation execution
- ✅ Context passed to OSL helper functions
- ✅ Proper error handling for permission denied
- ✅ Existing tests still pass (with None context)

#### Subtask 3.1.4: Integration Verification (1 hour)
**Validation:**

1. **Context Flow Test:**
   - Create SecurityContext in application actor
   - Send in message to OSL actor
   - Verify context reaches OSL helper
   - Verify operation succeeds with valid permissions

2. **Permission Enforcement Test:**
   - Send operation with insufficient permissions
   - Verify operation rejected
   - Verify appropriate error returned

3. **Cross-Supervisor Test:**
   - Verify context preserved across supervisor boundaries
   - Test RootSupervisor → OSLSupervisor → OSL actor flow

**Acceptance Criteria:**
- ✅ Security context flows end-to-end
- ✅ Permission validation working
- ✅ No context leaks or security bypasses
- ✅ Documentation updated with context flow diagrams

---

### Task 3.2: OSL Middleware Configuration and Integration (Day 8, ~4 hours)

**Architecture:** Reuse `airssys_osl::middleware::security::SecurityMiddleware` and `SecurityAuditLogger` - DO NOT create new audit infrastructure.

#### Subtask 3.2.1: Configure SecurityMiddleware for OSL Actors (2 hours)
**Files Modified:**
- `src/osl/supervisor.rs` (~50 lines modified)
- `src/osl/actors/filesystem.rs` (~30 lines modified)
- `src/osl/actors/process.rs` (~30 lines modified)
- `src/osl/actors/network.rs` (~30 lines modified)

**Implementation - OSLSupervisor Configuration:**

```rust
// File: src/osl/supervisor.rs

// Add imports for OSL middleware
use airssys_osl::middleware::security::{
    SecurityMiddleware, SecurityMiddlewareBuilder,
    RoleBasedAccessControl, AccessControlList, Role,
};
use airssys_osl::middleware::security::audit::{
    SecurityAuditLogger, ConsoleSecurityAuditLogger,
};
use std::sync::Arc;

pub struct OSLSupervisor<M, B>
where
    M: Message,
    B: MessageBroker<M>,
{
    // ... existing fields
    
    // NEW: Security middleware configuration
    security_middleware: Option<SecurityMiddleware>,
    audit_logger: Option<Arc<dyn SecurityAuditLogger>>,
}

impl<M, B> OSLSupervisor<M, B>
where
    M: Message,
    B: MessageBroker<M>,
{
    /// Configure security middleware with RBAC/ACL policies
    pub fn with_security_middleware(mut self, middleware: SecurityMiddleware) -> Self {
        self.security_middleware = Some(middleware);
        self
    }
    
    /// Configure audit logger for security events
    pub fn with_audit_logger(mut self, logger: Arc<dyn SecurityAuditLogger>) -> Self {
        self.audit_logger = Some(logger);
        self
    }
    
    // Helper: Build default security middleware with audit logger
    pub fn with_default_security(mut self, audit_logger: Arc<dyn SecurityAuditLogger>) -> Self {
        let middleware = SecurityMiddlewareBuilder::new()
            .with_audit_logger(audit_logger.clone())
            .build()
            .expect("Failed to build security middleware");
        
        self.security_middleware = Some(middleware);
        self.audit_logger = Some(audit_logger);
        self
    }
}
```

**Implementation - OSL Actor Field:**

```rust
// File: src/osl/actors/filesystem.rs (and process.rs, network.rs)

pub struct FileSystemActor {
    // NEW: SecurityMiddleware from OSL (optional - use if helpers don't auto-apply)
    security_middleware: Option<SecurityMiddleware>,
}

impl FileSystemActor {
    pub fn new(security_middleware: Option<SecurityMiddleware>) -> Self {
        Self { security_middleware }
    }
}
```

**Key Points:**
- ✅ **Reuse OSL SecurityMiddleware** - No new audit infrastructure
- ✅ **Reuse OSL SecurityAuditLogger** - Automatic security event logging
- ✅ **OSL helpers with middleware** - Use `*_with_middleware()` variants
- ✅ **Middleware auto-logs** - No manual audit logging needed

**Acceptance Criteria:**
- ✅ OSLSupervisor can configure SecurityMiddleware
- ✅ SecurityAuditLogger shared across OSL actors
- ✅ Builder pattern for fluent configuration
- ✅ Default security configuration available

#### Subtask 3.2.2: Integration with OSL Helper Functions (2 hours)
**Files Modified:**
- `src/osl/actors/filesystem.rs` (update execute_operation)
- `src/osl/actors/process.rs` (update execute_operation)
- `src/osl/actors/network.rs` (update execute_operation)

**Implementation Pattern:**

```rust
// File: src/osl/actors/filesystem.rs

use airssys_osl::helpers::filesystem::{
    read_file_with_middleware,
    write_file_with_middleware,
    delete_file_with_middleware,
    create_directory_with_middleware,
};
use airssys_osl::core::context::ExecutionContext;

impl FileSystemActor {
    async fn execute_operation(
        &mut self,
        operation: FileSystemOperation,
        security_context: Option<SecurityContext>,
    ) -> FileSystemResponse {
        // Build ExecutionContext with SecurityContext
        let mut exec_context = ExecutionContext::new();
        if let Some(ctx) = security_context {
            exec_context = exec_context.with_security_context(ctx);
        }
        
        // Get security middleware (if configured)
        let middleware = self.security_middleware.clone();
        
        match operation {
            FileSystemOperation::ReadFile { path } => {
                // Use OSL helper with middleware
                // SecurityMiddleware automatically enforces policies!
                // SecurityAuditLogger automatically logs security events!
                match read_file_with_middleware(&path, &exec_context, middleware).await {
                    Ok(content) => FileSystemResponse::FileContent(content),
                    Err(e) => FileSystemResponse::Error(e.to_string()),
                }
            }
            FileSystemOperation::WriteFile { path, content } => {
                match write_file_with_middleware(&path, content, &exec_context, middleware).await {
                    Ok(_) => FileSystemResponse::Success,
                    Err(e) => FileSystemResponse::Error(e.to_string()),
                }
            }
            FileSystemOperation::DeleteFile { path } => {
                match delete_file_with_middleware(&path, &exec_context, middleware).await {
                    Ok(_) => FileSystemResponse::Success,
                    Err(e) => FileSystemResponse::Error(e.to_string()),
                }
            }
            FileSystemOperation::CreateDirectory { path } => {
                match create_directory_with_middleware(&path, &exec_context, middleware).await {
                    Ok(_) => FileSystemResponse::Success,
                    Err(e) => FileSystemResponse::Error(e.to_string()),
                }
            }
        }
    }
}
```

**What Happens Automatically in OSL Middleware:**

1. **SecurityMiddleware.before_execution()** (Priority 100):
   - Evaluates security policies (RBAC, ACL)
   - Logs security decision via SecurityAuditLogger
   - Denies operation if permission not granted
   - Returns `SecurityViolation` error if denied

2. **SecurityAuditLogger automatically logs:**
   - Timestamp, principal, session_id
   - Operation attempted
   - Policy decisions (allow/deny)
   - Policy names that evaluated
   - Complete audit trail in JSON format

3. **No Manual Logging Required:**
   - RT actors don't call audit logger directly
   - OSL middleware pipeline handles everything
   - Consistent audit format across all operations

**Apply to all OSL actors:**
- FileSystemActor: `read_file_with_middleware`, `write_file_with_middleware`, etc.
- ProcessActor: `spawn_process_with_middleware`, `kill_process_with_middleware`, etc.
- NetworkActor: `network_connect_with_middleware`, `network_listen_with_middleware`, etc.

**Acceptance Criteria:**
- ✅ All actors use OSL `*_with_middleware()` helpers
- ✅ ExecutionContext passed with SecurityContext
- ✅ SecurityMiddleware automatically enforces policies
- ✅ SecurityAuditLogger automatically logs events
- ✅ No manual permission checks in RT actors
- ✅ No manual audit logging in RT actors

---

### Task 3.3: Security-Focused Integration Tests (Day 8, ~2 hours)

**Architecture:** Test OSL SecurityMiddleware integration through RT actors - validate middleware enforcement, not duplicate it.

#### Subtask 3.3.1: Security Integration Tests
**File:** `tests/security_integration_tests.rs` (~200-300 lines)

**Test Categories:**

**1. SecurityContext Flow Tests:**
```rust
#[tokio::test]
async fn test_security_context_propagation_to_osl() {
    use airssys_osl::core::context::SecurityContext;
    use airssys_osl::middleware::security::{SecurityMiddlewareBuilder, RoleBasedAccessControl, Role};
    
    // Create OSL security middleware with RBAC
    let mut rbac = RoleBasedAccessControl::new();
    rbac = rbac.add_role(
        Role::new("user".to_string(), "Regular User".to_string())
            .with_permission("file:read".to_string())
    );
    rbac = rbac.assign_roles("alice".to_string(), vec!["user".to_string()]);
    
    let security = SecurityMiddlewareBuilder::new()
        .add_policy(Box::new(rbac))
        .build()
        .expect("Failed to build security middleware");
    
    // Create broker and OSL actor with security middleware
    let broker = Arc::new(InMemoryMessageBroker::new());
    let filesystem_actor = FileSystemActor::new(Some(security));
    
    // Create SecurityContext for alice with user role
    let ctx = SecurityContext::new("alice".to_string());
    
    // Send FileSystemRequest with SecurityContext
    let request = FileSystemRequest {
        operation: FileSystemOperation::ReadFile { 
            path: PathBuf::from("/tmp/test.txt") 
        },
        request_id: MessageId::new(),
        security_context: Some(ctx),  // ← SecurityContext flows through
    };
    
    broker.publish(request).await.unwrap();
    
    // OSL SecurityMiddleware should allow (alice has file:read permission)
    // Verify operation succeeds (assuming file exists)
    // Note: This tests that context reaches OSL middleware correctly
}
```

**2. OSL Middleware Permission Enforcement Tests:**
```rust
#[tokio::test]
async fn test_osl_middleware_denies_insufficient_permissions() {
    use airssys_osl::core::context::SecurityContext;
    use airssys_osl::middleware::security::{SecurityMiddlewareBuilder, RoleBasedAccessControl, Role};
    
    // Create RBAC with limited permissions
    let mut rbac = RoleBasedAccessControl::new();
    rbac = rbac.add_role(
        Role::new("readonly".to_string(), "Read Only".to_string())
            .with_permission("file:read".to_string())  // Only read permission
    );
    rbac = rbac.assign_roles("bob".to_string(), vec!["readonly".to_string()]);
    
    let security = SecurityMiddlewareBuilder::new()
        .add_policy(Box::new(rbac))
        .build()
        .expect("Failed to build security middleware");
    
    let filesystem_actor = FileSystemActor::new(Some(security));
    
    // Create context for bob (readonly role)
    let ctx = SecurityContext::new("bob".to_string());
    
    // Try to WRITE file (bob only has file:read permission)
    let request = FileSystemRequest {
        operation: FileSystemOperation::WriteFile {
            path: PathBuf::from("/tmp/test.txt"),
            content: vec![1, 2, 3],
        },
        request_id: MessageId::new(),
        security_context: Some(ctx),
    };
    
    // OSL SecurityMiddleware should DENY (bob lacks file:write permission)
    // Verify response contains SecurityViolation error
    // This proves OSL middleware is enforcing policies correctly
}
```

**3. OSL SecurityAuditLogger Integration Tests:**
```rust
#[tokio::test]
async fn test_osl_audit_logger_captures_security_events() {
    use airssys_osl::middleware::security::audit::{SecurityAuditLogger, InMemorySecurityAuditLogger};
    
    // Create in-memory audit logger (for testing)
    let audit_logger = Arc::new(InMemorySecurityAuditLogger::new());
    
    // Build security middleware with audit logger
    let security = SecurityMiddlewareBuilder::new()
        .with_audit_logger(audit_logger.clone())
        .add_policy(Box::new(rbac))
        .build()
        .expect("Failed to build security middleware");
    
    // Create actor with security middleware
    let filesystem_actor = FileSystemActor::new(Some(security));
    
    // Execute operation
    let ctx = SecurityContext::new("alice".to_string());
    let request = FileSystemRequest {
        operation: FileSystemOperation::ReadFile { 
            path: PathBuf::from("/tmp/test.txt") 
        },
        security_context: Some(ctx),
    };
    
    // ... execute request
    
    // Verify OSL SecurityAuditLogger captured security event
    let events = audit_logger.get_events().await;
    assert!(!events.is_empty());
    
    // Verify event contains:
    // - Principal ("alice")
    // - Operation type
    // - Policy decision (allow/deny)
    // - Timestamp
    // This proves OSL audit logging works through RT actors
}
```

**4. Cross-Actor Security Context Tests:**
```rust
#[tokio::test]
async fn test_security_context_across_broker_messages() {
    // Verify SecurityContext preserved when messages routed through broker
    // Test: AppActor → Broker → OSLFileSystemActor → OSL middleware
    // Ensure context doesn't get lost in message passing
}
```

**5. Backward Compatibility Tests:**
```rust
#[tokio::test]
async fn test_none_security_context_still_works() {
    // Verify operations with security_context = None still execute
    // Tests backward compatibility with Phase 1/2 code
    // None should use system default context or allow unrestricted (depending on policy)
}
```

**Test Coverage:**
- ✅ 10+ security integration tests
- ✅ SecurityContext flow validation (RT → OSL)
- ✅ OSL SecurityMiddleware enforcement verified
- ✅ OSL SecurityAuditLogger integration validated
- ✅ Cross-broker message security verified
- ✅ Backward compatibility ensured

**Acceptance Criteria:**
- ✅ All security tests passing
- ✅ OSL middleware correctly enforces policies on RT-initiated operations
- ✅ OSL audit logger captures all security events
- ✅ No security bypass vulnerabilities
- ✅ Documentation of middleware integration patterns

---

## Integration Points

### airssys-osl Integration (VERIFIED)

**Dependencies Status:**

1. **✅ Security Context Types** - `airssys_osl::core::context::SecurityContext`
   - Location: `airssys-osl/src/core/context.rs`
   - Status: Implemented and stable
   - Usage: Import directly, no adapter needed

2. **✅ SecurityMiddleware** - `airssys_osl::middleware::security::SecurityMiddleware`
   - Location: `airssys-osl/src/middleware/security/middleware.rs`
   - Status: Complete with RBAC, ACL, policy framework
   - Priority: 100 (runs first in middleware pipeline)
   - Features: Deny-by-default, comprehensive audit logging

3. **✅ SecurityAuditLogger** - `airssys_osl::middleware::security::audit::SecurityAuditLogger`
   - Location: `airssys-osl/src/middleware/security/audit.rs`
   - Status: Trait + ConsoleSecurityAuditLogger implemented
   - Features: Async logging, JSON format, pluggable backends

4. **✅ Helper Functions** - OSL-TASK-010 Complete
   - Location: `airssys-osl/src/helpers/`
   - Status: All `*_with_middleware()` variants implemented
   - Features: Automatic middleware pipeline execution

### Actor Message Protocol
**Backward Compatibility:**
- ✅ Use `Option<SecurityContext>` for gradual migration
- ✅ Existing code without context continues to work (`None` value)
- ✅ No breaking changes to Phase 1/2 implementation
- ✅ Serialization/deserialization compatible

### Supervisor Hierarchy
**Security Context Propagation:**
- ✅ Context passed through RT message protocol
- ✅ No special supervisor handling needed (transparent)
- ✅ Supervisor restart preserves security semantics
- ✅ OSL middleware enforcement automatic

---

## Testing Strategy

### Unit Tests (Minimal - OSL Already Tested)
- SecurityContext extraction from messages
- ExecutionContext building with SecurityContext
- Message serialization with SecurityContext field

### Integration Tests (Primary Focus)
- **End-to-end security context flow** (RT → OSL → Middleware)
- **OSL SecurityMiddleware enforcement** through RT actors
- **OSL SecurityAuditLogger integration** validation
- **Cross-supervisor/broker security** verification
- **Backward compatibility** with None context

### Security Review Checklist
- ✅ No security context bypass vulnerabilities
- ✅ No privilege escalation paths (OSL middleware enforces)
- ✅ Audit logging covers all operations (OSL handles)
- ✅ Security boundaries properly enforced (middleware priority 100)
- ✅ Error messages don't leak sensitive info
- ✅ No manual security logic duplicated in RT

---

## Documentation Updates

### Code Documentation
- **SecurityContext import pattern** in rustdoc
- **OSL middleware configuration** examples
- **Helper function integration** best practices
- **Security integration guide** linking to OSL middleware docs

### Examples
- **Update existing examples** with SecurityContext in messages
- **Add security integration example** showing RBAC/ACL with RT actors
- **Document OSLSupervisor configuration** with SecurityMiddleware

### README Updates
- **Add OSL middleware integration section**
- **Document SecurityContext propagation pattern**
- **Link to OSL SecurityMiddleware documentation**
- **Security best practices** (defer to OSL middleware)

---

## Quality Checklist

### Code Quality
- [ ] Zero compilation errors
- [ ] Zero warnings
- [ ] Zero clippy warnings
- [ ] Workspace standards compliance (§2.1-§6.3)
- [ ] Microsoft Rust Guidelines compliance
- [ ] No duplicated security/audit logic (reuse OSL middleware)

### Testing
- [ ] All unit tests passing
- [ ] All integration tests passing (10+ security-focused tests)
- [ ] OSL SecurityMiddleware enforcement validated
- [ ] OSL SecurityAuditLogger integration validated
- [ ] Backward compatibility verified (None context)
- [ ] Performance impact measured (<2% overhead - minimal RT layer)

### Documentation
- [ ] Rustdoc complete for SecurityContext integration
- [ ] Examples showing OSL middleware configuration
- [ ] Security integration guide linking to OSL docs
- [ ] OSLSupervisor configuration documented

### Security
- [ ] Security review checklist complete
- [ ] No security context bypass vulnerabilities
- [ ] OSL middleware correctly enforces all policies
- [ ] OSL audit logging captures all security events
- [ ] No manual security logic in RT (delegated to OSL)

---

## Success Metrics

**Phase 3 Complete When:**
- ✅ SecurityContext flows end-to-end: RT messages → OSL ExecutionContext → OSL Middleware
- ✅ OSL SecurityMiddleware enforces policies on RT-initiated operations
- ✅ OSL SecurityAuditLogger automatically logs all security decisions
- ✅ Integration tests validate OSL middleware enforcement through RT actors
- ✅ Zero duplication of security/audit infrastructure
- ✅ Clean architectural alignment with OSL middleware layer
- ✅ Documentation comprehensive and accurate
- ✅ Performance impact minimal (<2% overhead for context passing)

**Deliverables (CORRECTED - No New Infrastructure):**
- ✅ `src/osl/actors/messages.rs` - Updated with `security_context` field (~50 lines modified)
- ✅ `src/osl/actors/filesystem.rs` - OSL middleware integration (~100 lines modified)
- ✅ `src/osl/actors/process.rs` - OSL middleware integration (~100 lines modified)
- ✅ `src/osl/actors/network.rs` - OSL middleware integration (~100 lines modified)
- ✅ `src/osl/supervisor.rs` - SecurityMiddleware configuration (~50 lines modified)
- ✅ `tests/security_integration_tests.rs` - Integration tests (~200-300 lines)
- ✅ Documentation updates (examples, README, rustdoc)
- ✅ All tests passing (existing + new security tests)

**What's NOT Being Created (Architectural Correction):**
- ❌ NO `src/osl/security.rs` - Use `airssys_osl::core::context::SecurityContext`
- ❌ NO `src/osl/actors/audit.rs` - Use `airssys_osl::middleware::security::audit::SecurityAuditLogger`
- ❌ NO manual permission validation in RT actors - Delegated to OSL SecurityMiddleware
- ❌ NO manual audit logging in RT actors - Handled by OSL SecurityAuditLogger
- ❌ NO security infrastructure duplication - Full OSL middleware reuse

---

## Architectural Benefits Summary

### ✅ Advantages of OSL Middleware Integration:

1. **Zero Code Duplication**
   - Reuse OSL's battle-tested security implementation
   - Single source of truth for security policies
   - Consistent security model across RT and OSL

2. **Automatic Security Enforcement**
   - OSL SecurityMiddleware runs at priority 100 (first)
   - Deny-by-default model automatically applied
   - No risk of missing security checks

3. **Comprehensive Audit Trail**
   - OSL SecurityAuditLogger captures all decisions
   - Consistent JSON format across all operations
   - Pluggable backends (console, file, custom)

4. **Clean Separation of Concerns**
   - RT layer: Message passing with SecurityContext
   - OSL layer: Security enforcement and audit logging
   - No mixing of concerns or responsibilities

5. **Maintainability**
   - Security updates in OSL automatically apply to RT
   - Testing focused on integration, not duplication
   - Simplified codebase with clear boundaries

### Architecture Flow Diagram:

```
Application Actor
      │
      │ Creates SecurityContext
      │
      ▼
RT Message (FileSystemRequest)
      │
      │ security_context: Option<SecurityContext>
      │
      ▼
RT OSL Actor (FileSystemActor)
      │
      │ Builds ExecutionContext with SecurityContext
      │
      ▼
OSL Helper Function (*_with_middleware)
      │
      │ Passes ExecutionContext + SecurityMiddleware
      │
      ▼
OSL Middleware Pipeline
      │
      ├─► Priority 100: SecurityMiddleware
      │   │
      │   ├─► Evaluate RBAC/ACL policies
      │   ├─► Log decision via SecurityAuditLogger
      │   └─► Allow/Deny operation
      │
      ├─► Priority 200: LoggerMiddleware (if configured)
      │
      └─► Execute operation (if allowed)
```

---

## Next Steps After Phase 3

**Phase 4 Prerequisites:**
- ✅ Phase 3 complete (OSL middleware integration working)
- ✅ All integration tests passing
- ✅ Security review approved
- ✅ Performance validated (<2% overhead)

**Phase 4 Focus:**
- Comprehensive examples showing RBAC/ACL with RT actors
- Migration guide for adding SecurityContext to existing apps
- Performance benchmarks (RT → OSL middleware flow)
- mdBook documentation updates

---

**REVISION SUMMARY (2025-10-15):**

This action plan has been **completely revised** to align with airssys-osl's existing middleware architecture. The original plan incorrectly proposed creating duplicate security and audit infrastructure in airssys-rt. The corrected approach:

- ✅ **Reuses** OSL SecurityMiddleware for policy enforcement
- ✅ **Reuses** OSL SecurityAuditLogger for audit logging  
- ✅ **Imports** OSL SecurityContext type (no duplication)
- ✅ **Integrates** RT actors with OSL middleware pipeline
- ✅ **Eliminates** ~600-800 lines of duplicated code
- ✅ **Maintains** clean architectural separation of concerns

**This approach is architecturally superior and follows the established OSL middleware design patterns.**

