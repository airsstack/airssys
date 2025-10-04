# Technical Debt: Framework-Core Integration Gap

**Debt ID:** DEBT-002  
**Created:** 2025-10-04  
**Severity:** High  
**Category:** Architecture  
**Status:** Active  
**Estimated Effort:** 3-4 days  

## Summary
The current framework layer (OSL-TASK-006 Phase 3) provides a skeleton API but does not properly integrate with core abstractions. Operation builders create placeholder types that bypass the `Operation` trait, `OSExecutor` trait, and middleware pipeline.

## Technical Context

### Current Implementation State (OSL-TASK-006 Phase 3)
```rust
// operations.rs - Current state
pub struct FileOperation<'a> {
    builder: FilesystemBuilder<'a>,
    operation: String,  // ❌ Just operation name, not a proper Operation
}

impl<'a> FileOperation<'a> {
    pub async fn execute(self) -> OSResult<ExecutionResult> {
        Ok(ExecutionResult::success(
            format!("Phase 3: {} operation placeholder", self.operation).into_bytes()
        ))
    }
}

pub fn read_file(self, _path: &str) -> FileOperation<'a> {
    FileOperation {
        builder: self,
        operation: "read".to_string(),  // ❌ Path parameter unused
    }
}
```

### Core Abstractions (Properly Designed)
```rust
// core/operation.rs - What we SHOULD use
pub trait Operation: Debug + Send + Sync + Clone + 'static {
    fn operation_type(&self) -> OperationType;
    fn required_permissions(&self) -> Vec<Permission>;
    fn created_at(&self) -> DateTime<Utc>;
    fn operation_id(&self) -> String;
    fn requires_elevated_privileges(&self) -> bool;
}

// core/executor.rs - What we SHOULD use
#[async_trait]
pub trait OSExecutor<O>: Debug + Send + Sync + 'static
where O: Operation
{
    async fn execute(&self, operation: O, context: &ExecutionContext) 
        -> OSResult<ExecutionResult>;
    async fn validate_operation(&self, operation: &O, context: &ExecutionContext) 
        -> OSResult<()>;
}
```

## Specific Gaps Identified

### 1. **Operation Types Don't Implement Operation Trait** ❌
**Location:** `src/framework/operations.rs:46-51, 85-90, 124-129`

**Current:**
- `FileOperation<'a>` - Just a wrapper, not an `Operation`
- `ProcessOperation<'a>` - Just a wrapper, not an `Operation`  
- `NetworkOperation<'a>` - Just a wrapper, not an `Operation`

**Missing:**
- No `operation_type()` implementation
- No `required_permissions()` implementation
- No `created_at()` implementation
- No operation data storage (path, command, address all unused)

**Impact:**
- Can't be used with `OSExecutor<O>` trait
- Can't be used with `Middleware<O>` trait
- Can't be validated by security middleware
- Can't be logged with full context

### 2. **ExecutorRegistry Doesn't Store Executors** ❌
**Location:** `src/framework/registry.rs:37-43`

**Current:**
```rust
pub struct ExecutorRegistry {
    registered_types: HashMap<OperationType, String>,  // ❌ Only names
}

pub fn get_executor_name(&self, operation_type: &OperationType) -> Option<&str> {
    // ❌ Returns string, not actual executor
}
```

**Missing:**
- No storage of actual `OSExecutor` trait objects
- No executor dispatch mechanism
- Can't execute operations

**Impact:**
- Framework can't actually execute operations
- All executors must be provided externally
- Registry is just metadata, not functional

### 3. **Operation Builders Bypass Executor System** ❌
**Location:** `src/framework/operations.rs:52-57`

**Current:**
```rust
impl<'a> FileOperation<'a> {
    pub async fn execute(self) -> OSResult<ExecutionResult> {
        // ❌ Returns placeholder, doesn't use executor
        Ok(ExecutionResult::success(b"placeholder".to_vec()))
    }
}
```

**Should Be:**
```rust
impl<'a> FileOperation<'a> {
    pub async fn execute(self) -> OSResult<ExecutionResult> {
        // 1. Create proper Operation trait object
        let operation = FileReadOperation::new(self.path, Utc::now());
        
        // 2. Get executor from registry
        let executor = self.builder.framework.executors
            .get_executor(OperationType::Filesystem)?;
        
        // 3. Create execution context
        let context = ExecutionContext::new(
            self.builder.framework.security_context.clone()
        );
        
        // 4. Execute through pipeline
        executor.execute(operation, &context).await
    }
}
```

**Impact:**
- Entire executor architecture unused
- No actual file I/O, process spawning, network operations
- Test placeholder code in production API

### 4. **No Middleware Pipeline Integration** ❌
**Location:** `src/framework/framework.rs:142-154`, `src/framework/operations.rs:52-103`

**Current Framework Execute:**
```rust
pub async fn execute<O: Operation>(&self, operation: O) -> OSResult<ExecutionResult> {
    let _exec_context = ExecutionContext::new(self.security_context.clone());
    let _operation_type = operation.operation_type();
    
    // ❌ Returns placeholder, no pipeline execution
    Ok(ExecutionResult::success(b"Phase 1 placeholder".to_vec()))
}
```

**Missing:**
- No `middleware_pipeline.execute_before()`
- No `middleware_pipeline.execute_after()`
- No `middleware_pipeline.handle_error()`
- No security validation
- No activity logging

**Impact:**
- Security middleware never runs (OSL-TASK-002 logger bypassed)
- No audit trail generation
- No permission validation
- Operations execute without security checks

### 5. **Parameters Not Stored in Operations** ❌
**Location:** `src/framework/operations.rs:29-41`

**Current:**
```rust
pub fn read_file(self, _path: &str) -> FileOperation<'a> {  // ❌ _path unused
    FileOperation {
        builder: self,
        operation: "read".to_string(),  // ❌ Only name stored
    }
}
```

**Missing:**
- File paths not stored in `FileOperation`
- Commands not stored in `ProcessOperation`
- Addresses not stored in `NetworkOperation`

**Impact:**
- Can't implement `required_permissions()` - no resource info
- Can't perform actual operations - no data to work with
- Can't validate security policies - no context

## Why This Happened

### Intentional Design Decision (Phase 3 Scope)
**From OSL-TASK-006 Phase 3 description:**
> "Phase 3: Operation Builders (30 minutes) - API structure only, placeholder implementations"

This was **intentional** to:
1. Establish the developer-facing API shape first
2. Allow user testing of the ergonomic interface
3. Defer integration complexity to later phases

### Task Prioritization Strategy
**From AGENTS.md task sequence:**
> Tasks: 001 → 002 → 005 → 006 → 003 → 004

- Tasks 003 (Security) and 004 (Pipeline) were **deprioritized**
- Task 006 Phase 3 completed **before** middleware infrastructure
- This created the integration gap we see now

## Impact Assessment

### User Impact
- **Developer Experience:** API looks complete but doesn't work ⚠️
- **Testing:** Can't write real integration tests ⚠️
- **Production Readiness:** Framework not usable for real operations ❌

### System Impact
- **Security:** No security validation occurring 🔴 **CRITICAL**
- **Audit:** No activity logging 🔴 **CRITICAL**
- **Architecture:** Core abstractions unused 🔴 **CRITICAL**

### Integration Impact
- **airssys-rt:** Can't use OSL for process management ❌
- **airssys-wasm:** Can't use OSL for sandboxing ❌
- **External Tools:** Can't execute docker, gh CLI ❌

## Resolution Strategy

### Phase 1: Concrete Operation Types (OSL-TASK-007)
**Create proper Operation trait implementations:**
```rust
// Concrete operation types
#[derive(Debug, Clone)]
pub struct FileReadOperation {
    path: String,
    created_at: DateTime<Utc>,
}

impl Operation for FileReadOperation {
    fn operation_type(&self) -> OperationType { OperationType::Filesystem }
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::FilesystemRead(self.path.clone())]
    }
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
}

// Similar for: FileWriteOperation, ProcessSpawnOperation, NetworkConnectOperation
```

### Phase 2: Platform Executors (OSL-TASK-008)
**Implement OSExecutor trait for real operations:**
```rust
pub struct FilesystemExecutor {
    name: String,
}

#[async_trait]
impl OSExecutor<FileReadOperation> for FilesystemExecutor {
    async fn execute(&self, operation: FileReadOperation, context: &ExecutionContext) 
        -> OSResult<ExecutionResult> {
        // Real tokio::fs::read() implementation
        let content = tokio::fs::read(&operation.path).await
            .map_err(|e| OSError::filesystem_error("read", &operation.path, e.to_string()))?;
        
        Ok(ExecutionResult::success(content))
    }
}
```

### Phase 3: Registry Integration (Part of OSL-TASK-008)
**Store actual executors in registry:**
```rust
pub struct ExecutorRegistry {
    // ✅ Store trait objects
    filesystem_executor: Arc<dyn OSExecutor<FileReadOperation>>,
    process_executor: Arc<dyn OSExecutor<ProcessSpawnOperation>>,
    // ... etc
}
```

### Phase 4: Pipeline Integration (OSL-TASK-004 + OSL-TASK-006 Phase 4)
**Wire up middleware pipeline:**
```rust
pub async fn execute(self) -> OSResult<ExecutionResult> {
    let operation = FileReadOperation::new(self.path);
    
    // ✅ Run through middleware pipeline
    let context = ExecutionContext::new(self.builder.framework.security_context.clone());
    self.builder.framework.middleware_pipeline
        .execute_before(&operation, &context).await?;
    
    // ✅ Execute with real executor
    let result = self.builder.framework.executors
        .get_filesystem_executor()
        .execute(operation, &context).await;
    
    // ✅ After middleware
    self.builder.framework.middleware_pipeline
        .execute_after(&operation, &result, &context).await?;
    
    result
}
```

## Dependencies

### Must Complete First
- **OSL-TASK-003:** Security middleware implementation (provides security validation)
- **OSL-TASK-004:** Middleware pipeline framework (provides orchestration)

### Blocks
- **Real operation execution:** Can't execute until executors exist
- **Integration testing:** Can't test real operations
- **airssys-rt integration:** Runtime needs working process operations
- **airssys-wasm integration:** WASM needs working sandbox operations

## Success Metrics

### Technical Validation
- ✅ All operation types implement `Operation` trait
- ✅ All operations properly validated with `OSExecutor::validate_operation()`
- ✅ ExecutorRegistry stores and dispatches actual executors
- ✅ Middleware pipeline runs for every operation
- ✅ Security middleware validates all operations
- ✅ Logger middleware logs all operations

### Compliance Validation
- ✅ Zero operations bypass security validation
- ✅ Zero operations bypass audit logging
- ✅ All `_` prefixed unused parameters removed
- ✅ Core abstractions fully integrated

### Integration Validation
- ✅ Real file operations work (read/write)
- ✅ Real process operations work (spawn)
- ✅ Real network operations work (connect)
- ✅ Integration tests pass with actual I/O

## Related Documentation
- **ADR-027:** Builder Pattern Architecture Implementation
- **KNOW-003:** Operation Builder Lifetime Pattern
- **OSL-TASK-006:** Core Framework Implementation (current task)
- **OSL-TASK-003:** Security Middleware Module (needed for resolution)
- **OSL-TASK-004:** Middleware Pipeline Framework (needed for resolution)
- **Core Abstractions:** `src/core/operation.rs`, `src/core/executor.rs`

## Cross-References
- **Workspace Standards:** §6.1 (YAGNI - currently violated with unused code)
- **Microsoft Guidelines:** M-DI-HIERARCHY (should use generics, not dyn)
- **Memory Bank:** `.copilot/memory_bank/sub_projects/airssys-osl/progress.md`
