# RT-TASK-009 Phase 3: Security and Audit Integration - Action Plan

**Created:** 2025-10-14  
**Status:** Ready for Implementation  
**Phase:** Phase 3 of 4 (OSL Integration)  
**Prerequisites:** Phase 2 Complete (100% ✅)  
**Duration Estimate:** 2 days (16 hours)

---

## Executive Summary

**Objective:** Integrate airssys-osl security contexts and audit logging into OSL integration actors, enabling security policy enforcement and comprehensive audit trails for all system operations.

**Key Deliverables:**
1. Security context propagation through message protocol
2. Audit logging infrastructure for all OSL operations
3. Security-focused integration tests
4. Documentation updates showing security patterns

**Critical Dependencies:**
- ⚠️ airssys-osl security context types (verify OSL-TASK-003 status)
- ⚠️ airssys-osl audit logging infrastructure (verify OSL implementation)

---

## Phase 3 Overview

### Goals
- Enable security context flow: Application actors → OSL actors → OSL operations
- Implement comprehensive audit logging for all system operations
- Validate security boundaries and permission enforcement
- Document security integration patterns

### Success Criteria
- ✅ Security context propagates across actor boundaries
- ✅ All OSL operations logged with complete audit trail
- ✅ Permission validation examples working
- ✅ Security tests validate context flow and enforcement
- ✅ Zero security warnings from review checklist

---

## Task Breakdown

### Task 3.1: Security Context Propagation (Day 7, ~8 hours)

#### Subtask 3.1.1: Security Context Types (2 hours)
**File:** `src/osl/security.rs` (~150-200 lines)

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

### Task 3.2: Audit Logging Integration (Day 8, ~8 hours)

#### Subtask 3.2.1: Audit Logging Infrastructure (3 hours)
**File:** `src/osl/actors/audit.rs` (~200-250 lines)

**Implementation:**

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use async_trait::async_trait;

/// Audit event capturing operation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Actor that performed the operation
    pub actor_id: String,
    
    /// Operation type (read_file, spawn_process, etc.)
    pub operation: String,
    
    /// Security context (if present)
    pub security_context: Option<SecurityContext>,
    
    /// Operation-specific details
    pub details: serde_json::Value,
    
    /// Operation result (success/failure)
    pub result: AuditResult,
    
    /// Execution duration
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failure { error: String },
    PermissionDenied { reason: String },
}

/// Trait for pluggable audit backends
#[async_trait]
pub trait AuditLogger: Send + Sync {
    /// Log an audit event
    async fn log(&self, event: AuditEvent) -> Result<(), AuditError>;
    
    /// Flush buffered events
    async fn flush(&self) -> Result<(), AuditError>;
}

/// Console audit logger (stdout for testing)
pub struct ConsoleAuditLogger;

#[async_trait]
impl AuditLogger for ConsoleAuditLogger {
    async fn log(&self, event: AuditEvent) -> Result<(), AuditError> {
        println!("[AUDIT] {}", serde_json::to_string_pretty(&event)?);
        Ok(())
    }
    
    async fn flush(&self) -> Result<(), AuditError> {
        Ok(())
    }
}

/// File-based audit logger with rotation
pub struct FileAuditLogger {
    path: PathBuf,
    buffer: Arc<Mutex<Vec<AuditEvent>>>,
    max_buffer_size: usize,
}

impl FileAuditLogger {
    pub fn new(path: PathBuf) -> Self { ... }
    
    async fn write_to_file(&self, events: Vec<AuditEvent>) -> Result<(), AuditError> {
        // Append events to audit log file (JSON lines format)
        // Implement rotation if file exceeds max size
        ...
    }
}

#[async_trait]
impl AuditLogger for FileAuditLogger {
    async fn log(&self, event: AuditEvent) -> Result<(), AuditError> {
        let mut buffer = self.buffer.lock().await;
        buffer.push(event);
        
        if buffer.len() >= self.max_buffer_size {
            let events = std::mem::take(&mut *buffer);
            drop(buffer);
            self.write_to_file(events).await?;
        }
        
        Ok(())
    }
    
    async fn flush(&self) -> Result<(), AuditError> {
        let mut buffer = self.buffer.lock().await;
        if !buffer.is_empty() {
            let events = std::mem::take(&mut *buffer);
            drop(buffer);
            self.write_to_file(events).await?;
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
```

**Key Features:**
- Pluggable backend via trait
- Buffered async writing for performance
- JSON format for easy parsing
- Console logger for testing
- File logger for production

**Acceptance Criteria:**
- ✅ AuditLogger trait defined
- ✅ ConsoleAuditLogger implemented
- ✅ FileAuditLogger implemented with buffering
- ✅ Proper error handling
- ✅ Unit tests for both loggers

#### Subtask 3.2.2: OSL Actor Audit Integration (3 hours)
**Files:** 
- `src/osl/actors/filesystem.rs` (modify)
- `src/osl/actors/process.rs` (modify)
- `src/osl/actors/network.rs` (modify)

**Changes to each actor:**

```rust
pub struct FileSystemActor {
    audit_logger: Arc<dyn AuditLogger>,  // ← NEW field
}

impl FileSystemActor {
    pub fn new(audit_logger: Arc<dyn AuditLogger>) -> Self {
        Self { audit_logger }
    }
    
    async fn execute_operation(
        &mut self,
        operation: FileSystemOperation,
        security_context: Option<SecurityContext>,
    ) -> FileSystemResponse {
        let start = std::time::Instant::now();
        let operation_name = operation.name();
        
        // Pre-operation audit log
        let event_details = serde_json::to_value(&operation).unwrap();
        
        // Execute operation
        let result = match operation {
            FileSystemOperation::ReadFile { path } => {
                match airssys_osl::helpers::read_file_with_context(&path, security_context.clone()) {
                    Ok(content) => FileSystemResponse::FileContent(content),
                    Err(e) => FileSystemResponse::Error(e.to_string()),
                }
            }
            // ... other operations
        };
        
        // Post-operation audit log
        let duration_ms = start.elapsed().as_millis() as u64;
        let audit_result = match &result {
            FileSystemResponse::Error(e) if e.contains("Permission") => {
                AuditResult::PermissionDenied { reason: e.clone() }
            }
            FileSystemResponse::Error(e) => {
                AuditResult::Failure { error: e.clone() }
            }
            _ => AuditResult::Success,
        };
        
        let audit_event = AuditEvent {
            timestamp: Utc::now(),
            actor_id: "filesystem".to_string(),
            operation: operation_name,
            security_context: security_context.clone(),
            details: event_details,
            result: audit_result,
            duration_ms,
        };
        
        if let Err(e) = self.audit_logger.log(audit_event).await {
            eprintln!("Failed to log audit event: {}", e);
        }
        
        result
    }
}
```

**Apply to all actors:**
- FileSystemActor: Audit file operations
- ProcessActor: Audit process spawning
- NetworkActor: Audit network connections

**Acceptance Criteria:**
- ✅ All actors have audit_logger field
- ✅ Pre and post-operation logging
- ✅ Complete operation details captured
- ✅ Duration tracking
- ✅ Error cases properly logged
- ✅ No performance degradation (async logging)

#### Subtask 3.2.3: Centralized Audit Configuration (1 hour)
**Integration with OSLSupervisor:**

```rust
impl OSLSupervisor {
    pub fn with_audit_logger(mut self, logger: Arc<dyn AuditLogger>) -> Self {
        // Pass logger to all child actors during spawn
        self.audit_logger = Some(logger);
        self
    }
}
```

**Configuration examples:**
- Development: ConsoleAuditLogger
- Production: FileAuditLogger with rotation
- Testing: InMemoryAuditLogger (capture events for assertions)

**Acceptance Criteria:**
- ✅ Audit logger configurable per supervisor
- ✅ Same logger shared across all OSL actors
- ✅ Examples in documentation

#### Subtask 3.2.4: Audit Query and Analysis (1 hour)
**Optional utilities for audit log analysis:**

```rust
// src/osl/actors/audit_query.rs
pub struct AuditQuery {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub actor_filter: Option<String>,
    pub operation_filter: Option<String>,
    pub result_filter: Option<AuditResult>,
}

impl AuditQuery {
    pub async fn execute(&self, log_path: &Path) -> Result<Vec<AuditEvent>, AuditError> {
        // Read and parse audit log file
        // Filter events based on query criteria
        // Return matching events
        ...
    }
}
```

**Acceptance Criteria:**
- ✅ Basic query functionality
- ✅ Time range filtering
- ✅ Actor/operation filtering
- ✅ Examples in documentation

---

### Task 3.3: Security-Focused Tests (Day 8, ~4 hours)

#### Subtask 3.3.1: Security Integration Tests
**File:** `tests/security_integration_tests.rs` (~300-400 lines)

**Test Categories:**

**1. Security Context Flow Tests:**
```rust
#[tokio::test]
async fn test_security_context_propagation() {
    // Create security context with file:read permission
    let ctx = SecurityContext::new(...)
        .with_permission(Permission::new("file:read", "/allowed/path"));
    
    // Create request with context
    let request = FileSystemRequest {
        operation: FileSystemOperation::ReadFile { path: "/allowed/path/file.txt" },
        request_id: MessageId::new(),
        security_context: Some(ctx),
    };
    
    // Send to actor via broker
    broker.publish(request).await;
    
    // Verify operation succeeds
    let response = wait_for_response(&broker).await;
    assert!(matches!(response, FileSystemResponse::FileContent(_)));
}
```

**2. Permission Enforcement Tests:**
```rust
#[tokio::test]
async fn test_permission_denied() {
    // Create context WITHOUT required permission
    let ctx = SecurityContext::new(...)
        .with_permission(Permission::new("file:read", "/other/path"));
    
    // Try to write file (no write permission)
    let request = FileSystemRequest {
        operation: FileSystemOperation::WriteFile { 
            path: "/denied/file.txt",
            content: vec![],
        },
        request_id: MessageId::new(),
        security_context: Some(ctx),
    };
    
    broker.publish(request).await;
    
    // Verify operation denied
    let response = wait_for_response(&broker).await;
    assert!(matches!(response, FileSystemResponse::Error(e) if e.contains("Permission")));
}
```

**3. Audit Log Completeness Tests:**
```rust
#[tokio::test]
async fn test_audit_log_completeness() {
    let audit_logger = Arc::new(InMemoryAuditLogger::new());
    let actor = FileSystemActor::new(audit_logger.clone());
    
    // Execute operation
    actor.execute_operation(...).await;
    
    // Verify audit event logged
    let events = audit_logger.get_events().await;
    assert_eq!(events.len(), 1);
    
    let event = &events[0];
    assert_eq!(event.actor_id, "filesystem");
    assert_eq!(event.operation, "read_file");
    assert!(event.security_context.is_some());
    assert!(matches!(event.result, AuditResult::Success));
    assert!(event.duration_ms > 0);
}
```

**4. Cross-Supervisor Security Tests:**
```rust
#[tokio::test]
async fn test_security_across_supervisors() {
    // Create supervisor hierarchy
    let root = RootSupervisor::new();
    let osl_supervisor = OSLSupervisor::new(...);
    
    // Send message with context from app supervisor to OSL supervisor
    // Verify context preserved across boundary
    ...
}
```

**5. Security Boundary Isolation Tests:**
```rust
#[tokio::test]
async fn test_security_boundary_isolation() {
    // Verify OSL actor failure doesn't leak security context
    // Verify app actors can't bypass OSL security
    ...
}
```

**Test Coverage:**
- ✅ 15+ security-focused tests
- ✅ Context flow validation
- ✅ Permission enforcement (positive and negative cases)
- ✅ Audit log completeness
- ✅ Cross-supervisor security
- ✅ Security boundary isolation
- ✅ Error handling with security context

**Acceptance Criteria:**
- ✅ All security tests passing
- ✅ >95% coverage of security code paths
- ✅ No security bypass vulnerabilities
- ✅ Audit logging validated in tests
- ✅ Documentation of security patterns

---

## Integration Points

### airssys-osl Integration
**Dependencies to verify:**

1. **Security Context Types** (OSL-TASK-003)
   - Check if OSL has SecurityContext implementation
   - If yes: Use OSL types directly or create adapter
   - If no: Implement in airssys-rt, plan future OSL migration

2. **Audit Logging Infrastructure**
   - Check if OSL has audit logging
   - Reuse OSL audit infrastructure if available
   - Ensure compatibility with OSL helpers

3. **Helper Function Updates**
   - Verify OSL helpers accept security context parameter
   - Update helper signatures if needed
   - Document helper API changes

### Actor Message Protocol
**Backward Compatibility:**
- Use `Option<SecurityContext>` for gradual migration
- Existing code without context continues to work
- No breaking changes to Phase 1/2 implementation

### Supervisor Hierarchy
**Security Context Propagation:**
- Context passed through message protocol
- No special supervisor handling needed
- Supervisor restart preserves security semantics

---

## Testing Strategy

### Unit Tests
- Security context creation and validation
- Permission checking logic
- Audit event creation
- Audit logger implementations

### Integration Tests
- End-to-end security context flow
- Permission enforcement scenarios
- Audit log completeness
- Cross-supervisor security

### Security Review
- [ ] No security context bypass vulnerabilities
- [ ] No privilege escalation paths
- [ ] Audit logging covers all operations
- [ ] Security boundaries properly enforced
- [ ] Error messages don't leak sensitive info

---

## Documentation Updates

### Code Documentation
- SecurityContext usage examples in rustdoc
- Permission validation patterns
- Audit logging best practices
- Security integration guide

### Examples
- Update existing examples with security context
- Add security-focused example
- Document permission configuration

### README Updates
- Add security integration section
- Document audit logging configuration
- Link to security best practices

---

## Quality Checklist

### Code Quality
- [ ] Zero compilation errors
- [ ] Zero warnings
- [ ] Zero clippy warnings
- [ ] Workspace standards compliance (§2.1-§6.3)
- [ ] Microsoft Rust Guidelines compliance

### Testing
- [ ] All unit tests passing
- [ ] All integration tests passing
- [ ] Security tests comprehensive
- [ ] >95% test coverage for security code
- [ ] Performance impact measured (<5% overhead)

### Documentation
- [ ] Rustdoc complete and accurate
- [ ] Examples working and tested
- [ ] Security patterns documented
- [ ] Audit configuration documented

### Security
- [ ] Security review checklist complete
- [ ] No known vulnerabilities
- [ ] Audit logging comprehensive
- [ ] Permission model validated

---

## Success Metrics

**Phase 3 Complete When:**
- ✅ Security context flows end-to-end (app → OSL → OS)
- ✅ All OSL operations audited with complete details
- ✅ Permission validation working and tested
- ✅ Security tests validate all critical paths
- ✅ Zero security warnings from review
- ✅ Documentation comprehensive and accurate
- ✅ Performance impact acceptable (<5% overhead)

**Deliverables:**
- ✅ `src/osl/security.rs` (~150-200 lines)
- ✅ `src/osl/actors/audit.rs` (~200-250 lines)
- ✅ `tests/security_integration_tests.rs` (~300-400 lines)
- ✅ Updated actor implementations with security
- ✅ Documentation updates
- ✅ All tests passing

---

## Next Steps After Phase 3

**Phase 4 Prerequisites:**
- ✅ Phase 3 complete (security & audit working)
- ✅ All integration tests passing
- ✅ Security review approved
- ✅ Performance validated

**Phase 4 Focus:**
- Comprehensive examples
- Migration guide
- Performance benchmarks
- mdBook documentation

---

**This action plan provides complete guidance for implementing security and audit integration in Phase 3, ensuring secure, auditable OSL operations with comprehensive testing and validation.**
