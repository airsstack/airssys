# Knowledge: Framework-Core Architecture Integration Pattern

**Knowledge ID:** KNOW-004  
**Created:** 2025-10-04  
**Category:** Architecture Pattern  
**Status:** Active  
**Related Tasks:** OSL-TASK-006, OSL-TASK-007, OSL-TASK-008  

## Overview
This document explains the correct integration pattern between the framework layer (high-level API) and core abstractions (Operation trait, OSExecutor trait, Middleware pipeline) in AirsSys OSL.

## Problem Statement

### The Layering Dilemma
**Framework Layer (User-Facing API):**
- Should provide ergonomic builder patterns
- Should hide complexity from users
- Should offer fluent, chainable interfaces

**Core Layer (Implementation):**
- Requires `Operation` trait implementation
- Requires `OSExecutor` trait for execution
- Requires `ExecutionContext` for security
- Requires middleware pipeline orchestration

**Challenge:** How to provide simple API while maintaining architectural integrity?

## Architecture Pattern: Builder-to-Operation Bridge

### Pattern Overview
```
User Code
    ↓ (uses fluent API)
FilesystemBuilder::read_file(path)
    ↓ (creates wrapper)
FileOperation<'a> { builder, path, ... }
    ↓ (execute() method)
ConcreteOperation (implements Operation trait)
    ↓ (passed to framework)
OSLFramework::execute(operation)
    ↓ (orchestrates)
MiddlewarePipeline::before → OSExecutor::execute → MiddlewarePipeline::after
    ↓ (returns)
ExecutionResult
```

### Layer Responsibilities

#### Layer 1: Builder API (User-Facing)
**Location:** `src/framework/operations.rs`

**Responsibilities:**
- Provide fluent interface
- Collect operation parameters
- Offer configuration (timeout, retries)
- Return operation wrapper

**Example:**
```rust
pub struct FilesystemBuilder<'a> {
    framework: &'a OSLFramework,
    timeout: Option<Duration>,
}

impl<'a> FilesystemBuilder<'a> {
    pub fn read_file(self, path: impl Into<String>) -> FileReadOperationBuilder<'a> {
        FileReadOperationBuilder {
            framework: self.framework,
            path: path.into(),
            timeout: self.timeout,
        }
    }
}
```

#### Layer 2: Operation Wrapper (Bridge)
**Location:** `src/framework/operations.rs`

**Responsibilities:**
- Store operation parameters
- Hold framework reference
- Convert to concrete Operation on execute()
- Bridge user API to core abstractions

**Example:**
```rust
pub struct FileReadOperationBuilder<'a> {
    framework: &'a OSLFramework,
    path: String,
    timeout: Option<Duration>,
}

impl<'a> FileReadOperationBuilder<'a> {
    pub async fn execute(self) -> OSResult<ExecutionResult> {
        // 1. Create concrete Operation trait object
        let operation = FileReadOperation {
            path: self.path,
            created_at: Utc::now(),
        };
        
        // 2. Pass to framework for proper execution
        self.framework.execute(operation).await
    }
}
```

#### Layer 3: Concrete Operations (Operation Trait Impl)
**Location:** `src/operations/` (new module, OSL-TASK-007)

**Responsibilities:**
- Implement `Operation` trait
- Define required permissions
- Store operation data
- Provide operation metadata

**Example:**
```rust
#[derive(Debug, Clone)]
pub struct FileReadOperation {
    pub path: String,
    pub created_at: DateTime<Utc>,
}

impl Operation for FileReadOperation {
    fn operation_type(&self) -> OperationType {
        OperationType::Filesystem
    }
    
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::FilesystemRead(self.path.clone())]
    }
    
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
```

#### Layer 4: Framework Orchestration
**Location:** `src/framework/framework.rs`

**Responsibilities:**
- Create ExecutionContext
- Orchestrate middleware pipeline
- Dispatch to appropriate executor
- Handle errors and results

**Example:**
```rust
impl OSLFramework {
    pub async fn execute<O: Operation>(&self, operation: O) -> OSResult<ExecutionResult> {
        // 1. Create execution context
        let mut context = ExecutionContext::new(self.security_context.clone());
        
        // 2. Run before middleware
        self.middleware_pipeline
            .execute_before(&operation, &mut context)
            .await?;
        
        // 3. Get appropriate executor
        let executor = self.executors
            .get_executor_for(&operation)?;
        
        // 4. Execute operation
        let result = executor.execute(operation.clone(), &context).await;
        
        // 5. Run after middleware
        self.middleware_pipeline
            .execute_after(&operation, &result, &context)
            .await?;
        
        result
    }
}
```

#### Layer 5: Platform Executors (OSExecutor Impl)
**Location:** `src/executors/` (new module, OSL-TASK-008)

**Responsibilities:**
- Implement actual I/O operations
- Platform-specific code
- Error handling and conversion
- Resource management

**Example:**
```rust
pub struct FilesystemExecutor;

#[async_trait]
impl OSExecutor<FileReadOperation> for FilesystemExecutor {
    fn name(&self) -> &str { "filesystem-executor" }
    
    fn supported_operation_types(&self) -> Vec<OperationType> {
        vec![OperationType::Filesystem]
    }
    
    async fn execute(
        &self,
        operation: FileReadOperation,
        _context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let started_at = Utc::now();
        
        // Real tokio::fs operation
        let content = tokio::fs::read(&operation.path)
            .await
            .map_err(|e| OSError::filesystem_error(
                "read",
                &operation.path,
                e.to_string()
            ))?;
        
        let completed_at = Utc::now();
        
        Ok(ExecutionResult::success_with_timing(
            content,
            started_at,
            completed_at,
        ))
    }
    
    async fn validate_operation(
        &self,
        operation: &FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<()> {
        // Validate file exists, check permissions, etc.
        if !tokio::fs::try_exists(&operation.path).await? {
            return Err(OSError::filesystem_error(
                "validate",
                &operation.path,
                "File does not exist",
            ));
        }
        
        // Check security context has required permission
        let required = Permission::FilesystemRead(operation.path.clone());
        // ... permission validation logic
        
        Ok(())
    }
}
```

## Type Flow Diagram

```
┌─────────────────────────────────────────────────────────────┐
│ User Code                                                    │
├─────────────────────────────────────────────────────────────┤
│ osl.filesystem().read_file("/path/to/file").execute().await │
└────────┬────────────────────────────────────────────────────┘
         │
         ↓ (FilesystemBuilder::read_file)
┌─────────────────────────────────────────┐
│ FileReadOperationBuilder<'a>            │
│  - framework: &'a OSLFramework          │
│  - path: String                         │
│  - timeout: Option<Duration>            │
└────────┬────────────────────────────────┘
         │
         ↓ (.execute())
┌─────────────────────────────────────────┐
│ FileReadOperation                       │
│  - path: String                         │
│  - created_at: DateTime<Utc>            │
│  impl Operation ✅                      │
└────────┬────────────────────────────────┘
         │
         ↓ (OSLFramework::execute)
┌─────────────────────────────────────────┐
│ ExecutionContext                        │
│  - execution_id: Uuid                   │
│  - security_context: SecurityContext    │
│  - metadata: HashMap                    │
└────────┬────────────────────────────────┘
         │
         ↓ (Middleware Pipeline)
┌─────────────────────────────────────────┐
│ SecurityMiddleware::before_execution()  │
│ LoggerMiddleware::before_execution()    │
└────────┬────────────────────────────────┘
         │
         ↓ (Executor Dispatch)
┌─────────────────────────────────────────┐
│ FilesystemExecutor::execute()           │
│  → tokio::fs::read()                    │
└────────┬────────────────────────────────┘
         │
         ↓ (Middleware Pipeline)
┌─────────────────────────────────────────┐
│ LoggerMiddleware::after_execution()     │
│ SecurityMiddleware::after_execution()   │
└────────┬────────────────────────────────┘
         │
         ↓
┌─────────────────────────────────────────┐
│ ExecutionResult                         │
│  - output: Vec<u8>                      │
│  - exit_code: i32                       │
│  - duration: Duration                   │
└─────────────────────────────────────────┘
```

## Key Design Principles

### 1. Separation of Concerns
- **Builder:** User ergonomics
- **Operation:** Data and metadata
- **Executor:** Platform implementation
- **Middleware:** Cross-cutting concerns
- **Framework:** Orchestration

### 2. Type Safety
- Each layer has strongly-typed interfaces
- Generic constraints prevent type mismatches
- Compile-time verification of compatibility

### 3. Testability
- Builders can be mocked
- Operations are pure data structures
- Executors can be swapped for testing
- Middleware can be individually tested

### 4. Security First
- All operations validated through middleware
- Security context required for execution
- Permissions checked before execution
- Audit trail automatic

## Common Pitfalls

### ❌ Pitfall 1: Operations Execute Themselves
```rust
// WRONG - Bypasses executor architecture
impl FileOperation {
    pub async fn execute(self) -> OSResult<ExecutionResult> {
        let content = tokio::fs::read(&self.path).await?;  // ❌ Direct I/O
        Ok(ExecutionResult::success(content))
    }
}
```

**Why Wrong:**
- No security validation
- No middleware pipeline
- No audit logging
- Can't mock for testing

### ❌ Pitfall 2: Framework Calls Executors Directly
```rust
// WRONG - Bypasses middleware
impl OSLFramework {
    pub async fn read_file(&self, path: &str) -> OSResult<Vec<u8>> {
        self.executors.filesystem_executor  // ❌ Direct executor call
            .execute(FileReadOperation::new(path))
            .await
    }
}
```

**Why Wrong:**
- No middleware execution
- No security checks
- No logging
- Breaks architectural layers

### ❌ Pitfall 3: Storing Parameters in Builders
```rust
// WRONG - Data should be in Operation, not builder
pub struct FileOperation<'a> {
    builder: FilesystemBuilder<'a>,  // ❌ Stores entire builder
    operation: String,                // ❌ Just operation name
    // Missing: path, permissions, timestamp
}
```

**Why Wrong:**
- Can't implement Operation trait
- No permission information
- Can't validate security
- Wastes memory (stores builder)

## Correct Implementation Checklist

### For Operation Builders
- ✅ Collect all necessary parameters
- ✅ Store framework reference (lifetime-bound)
- ✅ Return operation wrapper/builder
- ✅ Don't perform I/O or execution

### For Operation Wrappers
- ✅ Store operation-specific data
- ✅ Hold framework reference
- ✅ `execute()` creates concrete Operation
- ✅ `execute()` delegates to framework

### For Concrete Operations
- ✅ Implement `Operation` trait
- ✅ Store all operation data
- ✅ Define required permissions
- ✅ Include timestamp
- ✅ Be Clone + Debug + Send + Sync

### For Framework::execute()
- ✅ Create ExecutionContext
- ✅ Run before middleware
- ✅ Dispatch to executor
- ✅ Run after middleware
- ✅ Handle errors properly

### For Platform Executors
- ✅ Implement `OSExecutor<O>` trait
- ✅ Perform actual I/O operations
- ✅ Validate before execution
- ✅ Include timing information
- ✅ Convert platform errors to OSError

## Integration with Existing Tasks

### OSL-TASK-003 (Security Middleware)
Security middleware needs concrete operations to validate:
```rust
impl SecurityMiddleware {
    async fn before_execution<O: Operation>(
        &self,
        operation: &O,  // ✅ Real Operation trait object
        context: &ExecutionContext,
    ) -> MiddlewareResult<()> {
        // Can call: operation.required_permissions()
        // Can check: context.security_context
        // Can validate: policy.evaluate(operation, context)
    }
}
```

### OSL-TASK-004 (Middleware Pipeline)
Pipeline needs to orchestrate middleware:
```rust
impl MiddlewarePipeline {
    pub async fn execute_before<O: Operation>(
        &self,
        operation: &O,  // ✅ Real Operation trait object
        context: &mut ExecutionContext,
    ) -> OSResult<()> {
        for middleware in &self.middleware_stack {
            middleware.before_execution(operation, context).await?;
        }
        Ok(())
    }
}
```

### OSL-TASK-007 (Concrete Operations) - NEW
Creates the bridge types:
```rust
// Filesystem operations
pub struct FileReadOperation { ... }
pub struct FileWriteOperation { ... }
pub struct DirectoryCreateOperation { ... }

// Process operations
pub struct ProcessSpawnOperation { ... }
pub struct ProcessKillOperation { ... }

// Network operations
pub struct NetworkConnectOperation { ... }
pub struct NetworkListenOperation { ... }

// All implement Operation trait
```

### OSL-TASK-008 (Platform Executors) - NEW
Creates the actual executors:
```rust
pub struct FilesystemExecutor;
pub struct ProcessExecutor;
pub struct NetworkExecutor;

// All implement OSExecutor<O> for their operation types
```

## Success Indicators

### Architectural
- ✅ Clean separation between layers
- ✅ No layer bypassing
- ✅ All operations flow through middleware
- ✅ All operations use executors

### Implementation
- ✅ All operations implement `Operation` trait
- ✅ All executors implement `OSExecutor<O>` trait
- ✅ No `_` prefixed unused parameters
- ✅ Framework::execute() orchestrates properly

### Testing
- ✅ Can mock executors for unit tests
- ✅ Can test middleware independently
- ✅ Can test operations as pure data
- ✅ Integration tests use real I/O

## Related Documentation
- **DEBT-002:** Framework-Core Integration Gap
- **ADR-027:** Builder Pattern Architecture Implementation
- **KNOW-003:** Operation Builder Lifetime Pattern
- **Core Abstractions:** `src/core/operation.rs`, `src/core/executor.rs`, `src/core/middleware.rs`

## Cross-References
- **Microsoft Guidelines:** M-DI-HIERARCHY (generic constraints over dyn)
- **Workspace Standards:** §6.1 (YAGNI - avoid premature abstraction)
- **Task Dependencies:** OSL-TASK-006 → 007 → 008 → 003 → 004
