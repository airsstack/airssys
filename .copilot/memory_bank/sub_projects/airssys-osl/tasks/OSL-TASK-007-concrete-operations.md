# Task: Implement Concrete Operation Types

**Task ID:** OSL-TASK-007  
**Priority:** Critical  
**Status:** In Progress (Phase 2 Complete)  
**Created:** 2025-10-04  
**Estimated Effort:** 2-3 days  
**Progress:** Phase 1 ✅ | Phase 2 ✅ | Phase 3 🔄 | Phase 4 ⏳ | Phase 5 ⏳  

## Task Overview
Implement concrete operation types that properly implement the `Operation` trait for filesystem, process, and network operations. These types bridge the framework API layer with the core executor architecture.

## Task Description
Create complete concrete operation type implementations that store operation data, implement the `Operation` trait, define required permissions, and integrate with the framework builder API. This resolves the current gap where operation builders create placeholder types that bypass core abstractions.

## Dependencies
- **Blocked by:** None (can start immediately)
- **Blocks:** 
  - OSL-TASK-008 (Platform Executors - needs concrete operations to execute)
  - OSL-TASK-006 Phase 4 (Testing - needs real operations)
  - OSL-TASK-003 (Security Middleware - needs operations to validate)
  - OSL-TASK-004 (Pipeline Framework - needs operations to orchestrate)
- **Related:** 
  - DEBT-002 (Framework-Core Integration Gap)
  - KNOW-004 (Framework-Core Integration Pattern)
  - OSL-TASK-006 Phase 3 (Current placeholder implementation)

## Acceptance Criteria

### 1. Module Structure Created
- ✅ `src/operations/mod.rs` - Clean module exports and documentation
- ✅ `src/operations/filesystem.rs` - Filesystem operation types
- ✅ `src/operations/process.rs` - Process operation types
- ✅ `src/operations/network.rs` - Network operation types
- ✅ Updated `src/lib.rs` to include operations module

### 2. Filesystem Operations Implementation
- ✅ `FileReadOperation` - Read file with path and permissions (COMPLETE - 180 lines, 4 tests)
- ✅ `FileWriteOperation` - Write file with path, content, and permissions (COMPLETE - 170 lines, 3 tests)
- ✅ `DirectoryCreateOperation` - Create directory with path and permissions (COMPLETE - 160 lines, 3 tests)
- ✅ `DirectoryListOperation` - List directory contents (COMPLETE - 120 lines, 2 tests)
- ✅ `FileDeleteOperation` - Delete file with path and permissions (COMPLETE - 120 lines, 2 tests)
- ✅ All implement `Operation` trait correctly
- ✅ All store operation data (no unused parameters)
- ✅ All define required permissions properly
- ✅ **Modular structure**: Refactored to `filesystem/` subdirectory with 6 files for scalability
- ✅ **Comprehensive testing**: 16 unit tests + 16 doc tests, 100% pass rate

### 3. Process Operations Implementation
- ⏳ `ProcessSpawnOperation` - Spawn process with command, args, env
- ⏳ `ProcessKillOperation` - Kill process with PID
- ⏳ `ProcessSignalOperation` - Send signal to process
- ⏳ All implement `Operation` trait correctly
- ⏳ All store operation data
- ⏳ All define required permissions properly
- **Next Phase**: Will follow modular `process/` subdirectory pattern

### 4. Network Operations Implementation
- ⏳ `NetworkConnectOperation` - Connect to endpoint
- ⏳ `NetworkListenOperation` - Listen on address
- ⏳ `NetworkSocketOperation` - Create socket
- ⏳ All implement `Operation` trait correctly
- ⏳ All store operation data
- ⏳ All define required permissions properly
- **Next Phase**: Will follow modular `network/` subdirectory pattern

### 5. Framework Integration
- ⏳ Update `src/framework/operations.rs` to create concrete operations
- ⏳ Remove `_` prefix from all parameters (now used)
- ⏳ Operation wrappers delegate to framework.execute()
- ⏳ All operations flow through proper execution path
- **Phase 5**: Will be completed after all concrete operations exist

### 6. Quality Gates
- ✅ Zero compiler warnings
- ✅ Comprehensive rustdoc for all operation types
- ✅ Unit tests for Operation trait implementations
- ✅ All clippy lints passing
- ✅ No unused code or parameters

## Implementation Details

### Module Structure
```
src/operations/
├── mod.rs              # Operation type exports
├── filesystem.rs       # Filesystem operation types
├── process.rs          # Process operation types
└── network.rs          # Network operation types
```

### Filesystem Operations

#### FileReadOperation
```rust
/// Operation to read a file from the filesystem.
///
/// Requires read permission for the specified path.
#[derive(Debug, Clone)]
pub struct FileReadOperation {
    /// Path to the file to read
    pub path: String,
    
    /// When this operation was created
    pub created_at: DateTime<Utc>,
    
    /// Optional operation ID (generated if None)
    pub operation_id: Option<String>,
}

impl FileReadOperation {
    /// Create a new file read operation.
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            created_at: Utc::now(),
            operation_id: None,
        }
    }
    
    /// Create with explicit timestamp (for testing).
    pub fn with_timestamp(path: impl Into<String>, created_at: DateTime<Utc>) -> Self {
        Self {
            path: path.into(),
            created_at,
            operation_id: None,
        }
    }
    
    /// Set custom operation ID.
    pub fn with_operation_id(mut self, id: impl Into<String>) -> Self {
        self.operation_id = Some(id.into());
        self
    }
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
    
    fn operation_id(&self) -> String {
        self.operation_id.clone().unwrap_or_else(|| {
            format!("{}:{}", self.operation_type().as_str(), Uuid::new_v4())
        })
    }
}
```

#### FileWriteOperation
```rust
/// Operation to write data to a file.
///
/// Requires write permission for the specified path.
#[derive(Debug, Clone)]
pub struct FileWriteOperation {
    /// Path to the file to write
    pub path: String,
    
    /// Content to write to the file
    pub content: Vec<u8>,
    
    /// Whether to append or overwrite
    pub append: bool,
    
    /// When this operation was created
    pub created_at: DateTime<Utc>,
    
    /// Optional operation ID
    pub operation_id: Option<String>,
}

impl FileWriteOperation {
    /// Create a new file write operation (overwrite mode).
    pub fn new(path: impl Into<String>, content: Vec<u8>) -> Self {
        Self {
            path: path.into(),
            content,
            append: false,
            created_at: Utc::now(),
            operation_id: None,
        }
    }
    
    /// Create a new file write operation in append mode.
    pub fn append(path: impl Into<String>, content: Vec<u8>) -> Self {
        Self {
            path: path.into(),
            content,
            append: true,
            created_at: Utc::now(),
            operation_id: None,
        }
    }
}

impl Operation for FileWriteOperation {
    fn operation_type(&self) -> OperationType {
        OperationType::Filesystem
    }
    
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::FilesystemWrite(self.path.clone())]
    }
    
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
```

#### DirectoryCreateOperation
```rust
/// Operation to create a directory.
#[derive(Debug, Clone)]
pub struct DirectoryCreateOperation {
    pub path: String,
    pub recursive: bool,  // Create parent directories if needed
    pub created_at: DateTime<Utc>,
    pub operation_id: Option<String>,
}

impl DirectoryCreateOperation {
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            recursive: false,
            created_at: Utc::now(),
            operation_id: None,
        }
    }
    
    pub fn recursive(mut self) -> Self {
        self.recursive = true;
        self
    }
}

impl Operation for DirectoryCreateOperation {
    fn operation_type(&self) -> OperationType {
        OperationType::Filesystem
    }
    
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::FilesystemWrite(self.path.clone())]
    }
    
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
```

### Process Operations

#### ProcessSpawnOperation
```rust
/// Operation to spawn a new process.
///
/// Requires ProcessSpawn permission.
#[derive(Debug, Clone)]
pub struct ProcessSpawnOperation {
    /// Command to execute
    pub command: String,
    
    /// Command arguments
    pub args: Vec<String>,
    
    /// Environment variables
    pub env: HashMap<String, String>,
    
    /// Working directory (None = inherit)
    pub working_dir: Option<String>,
    
    /// When this operation was created
    pub created_at: DateTime<Utc>,
    
    /// Optional operation ID
    pub operation_id: Option<String>,
}

impl ProcessSpawnOperation {
    /// Create a new process spawn operation.
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            env: HashMap::new(),
            working_dir: None,
            created_at: Utc::now(),
            operation_id: None,
        }
    }
    
    /// Add command arguments.
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }
    
    /// Add a single argument.
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }
    
    /// Set environment variables.
    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = env;
        self
    }
    
    /// Add a single environment variable.
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
    
    /// Set working directory.
    pub fn working_dir(mut self, dir: impl Into<String>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }
}

impl Operation for ProcessSpawnOperation {
    fn operation_type(&self) -> OperationType {
        OperationType::Process
    }
    
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::ProcessSpawn]
    }
    
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    
    fn requires_elevated_privileges(&self) -> bool {
        true  // Process spawning is privileged
    }
}
```

### Network Operations

#### NetworkConnectOperation
```rust
/// Operation to connect to a network endpoint.
///
/// Requires NetworkConnect permission.
#[derive(Debug, Clone)]
pub struct NetworkConnectOperation {
    /// Address to connect to (e.g., "localhost:8080")
    pub address: String,
    
    /// Connection timeout
    pub timeout: Option<Duration>,
    
    /// When this operation was created
    pub created_at: DateTime<Utc>,
    
    /// Optional operation ID
    pub operation_id: Option<String>,
}

impl NetworkConnectOperation {
    /// Create a new network connect operation.
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            timeout: None,
            created_at: Utc::now(),
            operation_id: None,
        }
    }
    
    /// Set connection timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

impl Operation for NetworkConnectOperation {
    fn operation_type(&self) -> OperationType {
        OperationType::Network
    }
    
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::NetworkConnect(self.address.clone())]
    }
    
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    
    fn requires_elevated_privileges(&self) -> bool {
        true  // Network operations are privileged
    }
}
```

### Framework Integration Updates

#### Update operations.rs builders
```rust
// Before (Phase 3):
pub fn read_file(self, _path: &str) -> FileOperation<'a> {  // ❌ _path unused
    FileOperation {
        builder: self,
        operation: "read".to_string(),
    }
}

// After (Phase 4):
pub fn read_file(self, path: impl Into<String>) -> FileReadOperationWrapper<'a> {
    FileReadOperationWrapper {
        framework: self.framework,
        path: path.into(),
        timeout: self.timeout,
    }
}

pub struct FileReadOperationWrapper<'a> {
    framework: &'a OSLFramework,
    path: String,
    timeout: Option<Duration>,
}

impl<'a> FileReadOperationWrapper<'a> {
    pub async fn execute(self) -> OSResult<ExecutionResult> {
        // Create concrete operation
        let operation = FileReadOperation::new(self.path);
        
        // Execute through framework (uses executor + middleware)
        self.framework.execute(operation).await
    }
}
```

## Testing Requirements

### Unit Tests
- ✅ Operation trait implementation for all types
- ✅ Permission calculation correctness
- ✅ Operation ID generation
- ✅ Timestamp handling
- ✅ Builder pattern fluent interfaces

### Integration Tests
- ✅ Framework builder creates correct operations
- ✅ Operations flow through framework.execute()
- ✅ Placeholder executors can receive operations
- ✅ Middleware can validate operations

### Property Tests
- ✅ All operations are Clone
- ✅ All operations are Send + Sync
- ✅ Operation IDs are unique
- ✅ Timestamps are reasonable

## Documentation Requirements
- ✅ Comprehensive rustdoc for all operation types
- ✅ Examples for each operation type
- ✅ Permission model documentation
- ✅ Integration guide with framework builders
- ✅ Migration guide from Phase 3 placeholders

## Success Metrics

### Code Quality
- ✅ Zero unused parameters (no `_` prefixes)
- ✅ All operations properly implement Operation trait
- ✅ 100% of operations define required permissions
- ✅ All operations store necessary data

### Integration
- ✅ Framework builders create real operations
- ✅ Operations can be validated by middleware
- ✅ Operations can be executed by executors
- ✅ Security policies can evaluate operations

### Testing
- ✅ >95% code coverage on operation types
- ✅ All trait methods tested
- ✅ All builder patterns tested
- ✅ Integration tests pass

## Migration Impact

### Breaking Changes
- ✅ Operation wrapper types change (FileOperation → FileReadOperationWrapper)
- ✅ Parameters now required (can't use `_path`)
- ✅ Execute returns real results (not placeholders)

### Compatibility
- ✅ User-facing API remains the same (builders)
- ✅ Internal implementation changes only
- ✅ No changes to core abstractions

## Cross-References
- **DEBT-002:** Framework-Core Integration Gap (resolves)
- **KNOW-004:** Framework-Core Integration Pattern (implements)
- **OSL-TASK-006:** Core Framework Implementation (completes)
- **OSL-TASK-008:** Platform Executors (enables)
- **Core Module:** `src/core/operation.rs` (implements trait from)
- **Workspace Standards:** §2.1, §3.2, §4.3, §6.1, §6.2

## Related ADRs
- **ADR-027:** Builder Pattern Architecture Implementation
- **ADR-025:** dyn Pattern Exception (operations use generics)

## Notes
- This task is critical path for making the framework functional
- Unblocks executor implementation (OSL-TASK-008)
- Enables middleware testing (OSL-TASK-003, OSL-TASK-004)
- Should be completed before moving to tasks 003 and 004
