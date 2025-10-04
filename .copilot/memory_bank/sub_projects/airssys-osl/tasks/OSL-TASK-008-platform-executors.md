# Task: Implement Platform Executors

**Task ID:** OSL-TASK-008  
**Priority:** Critical  
**Status:** Pending  
**Created:** 2025-10-04  
**Estimated Effort:** 3-4 days  

## Task Overview
Implement platform-specific executor implementations that properly implement the `OSExecutor` trait for filesystem, process, and network operations. These executors perform the actual I/O operations and integrate with the executor registry.

## Task Description
Create production-ready executor implementations using tokio for async I/O, implement proper error handling and resource management, integrate with the executor registry, and provide comprehensive validation. This resolves the current gap where the registry only stores executor names instead of actual executor instances.

## Dependencies
- **Blocked by:** 
  - OSL-TASK-007 (Concrete Operations - executors need operation types to execute)
- **Blocks:** 
  - OSL-TASK-006 Phase 4 (Testing - needs real executors)
  - Real operation execution (can't execute until executors exist)
  - Integration testing (can't test real I/O)
- **Related:** 
  - DEBT-002 (Framework-Core Integration Gap)
  - KNOW-004 (Framework-Core Integration Pattern)
  - OSL-TASK-003 (Security Middleware - executors need validation)
  - OSL-TASK-004 (Pipeline Framework - executors called by pipeline)

## Acceptance Criteria

### 1. Module Structure Created
- ✅ `src/executors/mod.rs` - Executor module exports
- ✅ `src/executors/filesystem.rs` - Filesystem executor implementation
- ✅ `src/executors/process.rs` - Process executor implementation
- ✅ `src/executors/network.rs` - Network executor implementation
- ✅ Updated `src/lib.rs` to include executors module

### 2. Filesystem Executor Implementation
- ✅ `FilesystemExecutor` struct with proper initialization
- ✅ Implements `OSExecutor<FileReadOperation>`
- ✅ Implements `OSExecutor<FileWriteOperation>`
- ✅ Implements `OSExecutor<DirectoryCreateOperation>`
- ✅ Real tokio::fs operations (read, write, create_dir)
- ✅ Proper error handling with OSError conversion
- ✅ Timing information in ExecutionResult
- ✅ Validation before execution

### 3. Process Executor Implementation
- ✅ `ProcessExecutor` struct with proper initialization
- ✅ Implements `OSExecutor<ProcessSpawnOperation>`
- ✅ Real tokio::process operations
- ✅ Process output capture
- ✅ Exit code handling
- ✅ Timeout support
- ✅ Resource cleanup

### 4. Network Executor Implementation
- ✅ `NetworkExecutor` struct with proper initialization
- ✅ Implements `OSExecutor<NetworkConnectOperation>`
- ✅ Real tokio::net operations
- ✅ Connection handling
- ✅ Timeout support
- ✅ Socket cleanup

### 5. Executor Registry Integration
- ✅ Update `ExecutorRegistry` to store actual executors
- ✅ Type-safe executor storage and retrieval
- ✅ Automatic executor initialization
- ✅ Executor lifecycle management
- ✅ Remove executor name-only tracking

### 6. Quality Gates
- ✅ Zero compiler warnings
- ✅ Comprehensive rustdoc for all executors
- ✅ Unit tests for executor logic
- ✅ Integration tests with real I/O
- ✅ All clippy lints passing
- ✅ Error handling tested

## Implementation Details

### Module Structure
```
src/executors/
├── mod.rs              # Executor exports and registry setup
├── filesystem.rs       # FilesystemExecutor implementation
├── process.rs          # ProcessExecutor implementation
└── network.rs          # NetworkExecutor implementation
```

### Filesystem Executor

#### Structure and Initialization
```rust
/// Platform executor for filesystem operations.
///
/// Uses tokio::fs for async I/O operations.
#[derive(Debug)]
pub struct FilesystemExecutor {
    name: String,
}

impl FilesystemExecutor {
    /// Create a new filesystem executor.
    pub fn new() -> Self {
        Self {
            name: "filesystem-executor".to_string(),
        }
    }
}

impl Default for FilesystemExecutor {
    fn default() -> Self {
        Self::new()
    }
}
```

#### FileReadOperation Executor
```rust
#[async_trait]
impl OSExecutor<FileReadOperation> for FilesystemExecutor {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn supported_operation_types(&self) -> Vec<OperationType> {
        vec![OperationType::Filesystem]
    }
    
    async fn execute(
        &self,
        operation: FileReadOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let started_at = Utc::now();
        
        // Read file using tokio::fs
        let content = tokio::fs::read(&operation.path)
            .await
            .map_err(|e| OSError::filesystem_error(
                "read",
                &operation.path,
                e.to_string(),
            ))?;
        
        let completed_at = Utc::now();
        
        // Create result with timing information
        let mut result = ExecutionResult::success_with_timing(
            content,
            started_at,
            completed_at,
        );
        
        // Add metadata
        result = result
            .with_metadata("path".to_string(), operation.path.clone())
            .with_metadata("executor".to_string(), self.name.to_string())
            .with_metadata("user".to_string(), context.principal().to_string());
        
        Ok(result)
    }
    
    async fn validate_operation(
        &self,
        operation: &FileReadOperation,
        _context: &ExecutionContext,
    ) -> OSResult<()> {
        // Validate file exists
        if !tokio::fs::try_exists(&operation.path)
            .await
            .map_err(|e| OSError::filesystem_error(
                "validate",
                &operation.path,
                e.to_string(),
            ))? 
        {
            return Err(OSError::filesystem_error(
                "validate",
                &operation.path,
                "File does not exist",
            ));
        }
        
        // Validate path is not a directory
        let metadata = tokio::fs::metadata(&operation.path)
            .await
            .map_err(|e| OSError::filesystem_error(
                "validate",
                &operation.path,
                e.to_string(),
            ))?;
        
        if metadata.is_dir() {
            return Err(OSError::filesystem_error(
                "validate",
                &operation.path,
                "Path is a directory, not a file",
            ));
        }
        
        Ok(())
    }
    
    async fn cleanup(&self, _context: &ExecutionContext) -> OSResult<()> {
        // No cleanup needed for filesystem reads
        Ok(())
    }
}
```

#### FileWriteOperation Executor
```rust
#[async_trait]
impl OSExecutor<FileWriteOperation> for FilesystemExecutor {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn supported_operation_types(&self) -> Vec<OperationType> {
        vec![OperationType::Filesystem]
    }
    
    async fn execute(
        &self,
        operation: FileWriteOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let started_at = Utc::now();
        
        if operation.append {
            // Append mode
            use tokio::io::AsyncWriteExt;
            let mut file = tokio::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&operation.path)
                .await
                .map_err(|e| OSError::filesystem_error(
                    "open_append",
                    &operation.path,
                    e.to_string(),
                ))?;
            
            file.write_all(&operation.content)
                .await
                .map_err(|e| OSError::filesystem_error(
                    "write_append",
                    &operation.path,
                    e.to_string(),
                ))?;
                
            file.flush()
                .await
                .map_err(|e| OSError::filesystem_error(
                    "flush",
                    &operation.path,
                    e.to_string(),
                ))?;
        } else {
            // Overwrite mode
            tokio::fs::write(&operation.path, &operation.content)
                .await
                .map_err(|e| OSError::filesystem_error(
                    "write",
                    &operation.path,
                    e.to_string(),
                ))?;
        }
        
        let completed_at = Utc::now();
        
        let result = ExecutionResult::success_with_timing(
            Vec::new(),  // No output for write operations
            started_at,
            completed_at,
        )
        .with_metadata("path".to_string(), operation.path.clone())
        .with_metadata("bytes_written".to_string(), operation.content.len().to_string())
        .with_metadata("mode".to_string(), if operation.append { "append" } else { "overwrite" }.to_string())
        .with_metadata("executor".to_string(), self.name.to_string())
        .with_metadata("user".to_string(), context.principal().to_string());
        
        Ok(result)
    }
    
    async fn validate_operation(
        &self,
        operation: &FileWriteOperation,
        _context: &ExecutionContext,
    ) -> OSResult<()> {
        // Validate parent directory exists
        if let Some(parent) = std::path::Path::new(&operation.path).parent() {
            if !tokio::fs::try_exists(parent)
                .await
                .map_err(|e| OSError::filesystem_error(
                    "validate",
                    &operation.path,
                    format!("Cannot check parent directory: {}", e),
                ))? 
            {
                return Err(OSError::filesystem_error(
                    "validate",
                    &operation.path,
                    "Parent directory does not exist",
                ));
            }
        }
        
        Ok(())
    }
    
    async fn cleanup(&self, _context: &ExecutionContext) -> OSResult<()> {
        Ok(())
    }
}
```

### Process Executor

```rust
/// Platform executor for process operations.
///
/// Uses tokio::process for async process management.
#[derive(Debug)]
pub struct ProcessExecutor {
    name: String,
}

impl ProcessExecutor {
    pub fn new() -> Self {
        Self {
            name: "process-executor".to_string(),
        }
    }
}

#[async_trait]
impl OSExecutor<ProcessSpawnOperation> for ProcessExecutor {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn supported_operation_types(&self) -> Vec<OperationType> {
        vec![OperationType::Process]
    }
    
    async fn execute(
        &self,
        operation: ProcessSpawnOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let started_at = Utc::now();
        
        // Build command
        let mut command = tokio::process::Command::new(&operation.command);
        command.args(&operation.args);
        
        // Set environment variables
        for (key, value) in &operation.env {
            command.env(key, value);
        }
        
        // Set working directory if specified
        if let Some(working_dir) = &operation.working_dir {
            command.current_dir(working_dir);
        }
        
        // Capture output
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());
        
        // Spawn process
        let output = command
            .output()
            .await
            .map_err(|e| OSError::process_error(
                "spawn",
                format!("Failed to spawn '{}': {}", operation.command, e),
            ))?;
        
        let completed_at = Utc::now();
        
        // Combine stdout and stderr
        let mut combined_output = output.stdout;
        combined_output.extend(&output.stderr);
        
        let exit_code = output.status.code().unwrap_or(-1);
        
        let result = ExecutionResult::with_timing(
            combined_output,
            exit_code,
            started_at,
            completed_at,
        )
        .with_metadata("command".to_string(), operation.command.clone())
        .with_metadata("args".to_string(), operation.args.join(" "))
        .with_metadata("exit_code".to_string(), exit_code.to_string())
        .with_metadata("executor".to_string(), self.name.to_string())
        .with_metadata("user".to_string(), context.principal().to_string());
        
        Ok(result)
    }
    
    async fn validate_operation(
        &self,
        operation: &ProcessSpawnOperation,
        _context: &ExecutionContext,
    ) -> OSResult<()> {
        // Validate command is not empty
        if operation.command.is_empty() {
            return Err(OSError::process_error(
                "validate",
                "Command cannot be empty",
            ));
        }
        
        // Validate working directory exists if specified
        if let Some(working_dir) = &operation.working_dir {
            if !tokio::fs::try_exists(working_dir)
                .await
                .map_err(|e| OSError::process_error(
                    "validate",
                    format!("Cannot check working directory: {}", e),
                ))? 
            {
                return Err(OSError::process_error(
                    "validate",
                    format!("Working directory does not exist: {}", working_dir),
                ));
            }
        }
        
        Ok(())
    }
    
    async fn cleanup(&self, _context: &ExecutionContext) -> OSResult<()> {
        // Cleanup handled by tokio process management
        Ok(())
    }
}
```

### Network Executor

```rust
/// Platform executor for network operations.
///
/// Uses tokio::net for async network I/O.
#[derive(Debug)]
pub struct NetworkExecutor {
    name: String,
}

impl NetworkExecutor {
    pub fn new() -> Self {
        Self {
            name: "network-executor".to_string(),
        }
    }
}

#[async_trait]
impl OSExecutor<NetworkConnectOperation> for NetworkExecutor {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn supported_operation_types(&self) -> Vec<OperationType> {
        vec![OperationType::Network]
    }
    
    async fn execute(
        &self,
        operation: NetworkConnectOperation,
        context: &ExecutionContext,
    ) -> OSResult<ExecutionResult> {
        let started_at = Utc::now();
        
        // Parse address
        let addr = operation.address.parse::<std::net::SocketAddr>()
            .map_err(|e| OSError::network_error(
                "connect",
                format!("Invalid address '{}': {}", operation.address, e),
            ))?;
        
        // Connect with timeout if specified
        let stream = if let Some(timeout) = operation.timeout {
            tokio::time::timeout(
                timeout,
                tokio::net::TcpStream::connect(addr)
            )
            .await
            .map_err(|_| OSError::network_error(
                "connect",
                format!("Connection timeout after {:?}", timeout),
            ))?
            .map_err(|e| OSError::network_error(
                "connect",
                format!("Failed to connect to '{}': {}", operation.address, e),
            ))?
        } else {
            tokio::net::TcpStream::connect(addr)
                .await
                .map_err(|e| OSError::network_error(
                    "connect",
                    format!("Failed to connect to '{}': {}", operation.address, e),
                ))?
        };
        
        let completed_at = Utc::now();
        
        // Get connection info
        let local_addr = stream.local_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        let peer_addr = stream.peer_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        
        // Close the connection (this is just a connectivity test)
        drop(stream);
        
        let result = ExecutionResult::success_with_timing(
            format!("Connected to {}", operation.address).into_bytes(),
            started_at,
            completed_at,
        )
        .with_metadata("address".to_string(), operation.address.clone())
        .with_metadata("local_addr".to_string(), local_addr)
        .with_metadata("peer_addr".to_string(), peer_addr)
        .with_metadata("executor".to_string(), self.name.to_string())
        .with_metadata("user".to_string(), context.principal().to_string());
        
        Ok(result)
    }
    
    async fn validate_operation(
        &self,
        operation: &NetworkConnectOperation,
        _context: &ExecutionContext,
    ) -> OSResult<()> {
        // Validate address format
        operation.address.parse::<std::net::SocketAddr>()
            .map_err(|e| OSError::network_error(
                "validate",
                format!("Invalid address '{}': {}", operation.address, e),
            ))?;
        
        Ok(())
    }
    
    async fn cleanup(&self, _context: &ExecutionContext) -> OSResult<()> {
        Ok(())
    }
}
```

### Executor Registry Updates

```rust
/// Registry for managing operation executors with type-safe dispatch.
#[derive(Debug)]
pub struct ExecutorRegistry {
    filesystem_executor: Arc<FilesystemExecutor>,
    process_executor: Arc<ProcessExecutor>,
    network_executor: Arc<NetworkExecutor>,
}

impl ExecutorRegistry {
    /// Create a new executor registry with default executors.
    pub fn new() -> OSResult<Self> {
        Ok(Self {
            filesystem_executor: Arc::new(FilesystemExecutor::new()),
            process_executor: Arc::new(ProcessExecutor::new()),
            network_executor: Arc::new(NetworkExecutor::new()),
        })
    }
    
    /// Get the filesystem executor.
    pub fn filesystem_executor(&self) -> &FilesystemExecutor {
        &self.filesystem_executor
    }
    
    /// Get the process executor.
    pub fn process_executor(&self) -> &ProcessExecutor {
        &self.process_executor
    }
    
    /// Get the network executor.
    pub fn network_executor(&self) -> &NetworkExecutor {
        &self.network_executor
    }
    
    /// Get executor for a specific operation type.
    pub fn get_executor_for_type(&self, op_type: OperationType) -> OSResult<ExecutorType> {
        match op_type {
            OperationType::Filesystem => Ok(ExecutorType::Filesystem(&self.filesystem_executor)),
            OperationType::Process => Ok(ExecutorType::Process(&self.process_executor)),
            OperationType::Network => Ok(ExecutorType::Network(&self.network_executor)),
            OperationType::Utility => Err(OSError::execution_failed(
                "Utility executors not yet implemented"
            )),
        }
    }
}

/// Enum for different executor types (type-safe dispatch).
pub enum ExecutorType<'a> {
    Filesystem(&'a FilesystemExecutor),
    Process(&'a ProcessExecutor),
    Network(&'a NetworkExecutor),
}
```

## Testing Requirements

### Unit Tests
- ✅ Executor initialization and naming
- ✅ Supported operation types
- ✅ Validation logic for each operation
- ✅ Error handling and conversion
- ✅ Metadata generation

### Integration Tests
- ✅ Real file read operations
- ✅ Real file write operations
- ✅ Real directory creation
- ✅ Real process spawning
- ✅ Real network connections
- ✅ Error scenarios (file not found, etc.)
- ✅ Timeout handling

### Performance Tests
- ✅ File operation latency (<1ms for small files)
- ✅ Process spawn time (<10ms)
- ✅ Network connection time
- ✅ Resource cleanup verification

## Documentation Requirements
- ✅ Comprehensive rustdoc for all executors
- ✅ Examples for each executor
- ✅ Error handling patterns
- ✅ Platform-specific notes
- ✅ Security considerations

## Success Metrics

### Functionality
- ✅ All executors perform real I/O
- ✅ All operations return proper results
- ✅ Timing information accurate
- ✅ Metadata comprehensive

### Quality
- ✅ Zero compiler warnings
- ✅ All clippy lints passing
- ✅ >90% code coverage
- ✅ All integration tests passing

### Performance
- ✅ File reads <1ms (small files)
- ✅ Process spawns <10ms
- ✅ Network connects <100ms (local)
- ✅ No resource leaks

## Cross-References
- **DEBT-002:** Framework-Core Integration Gap (resolves)
- **KNOW-004:** Framework-Core Integration Pattern (implements)
- **OSL-TASK-007:** Concrete Operations (depends on)
- **OSL-TASK-006:** Core Framework Implementation (completes)
- **Core Module:** `src/core/executor.rs` (implements trait from)
- **Workspace Standards:** §2.1, §3.2, §4.3, §6.2, §6.3

## Related ADRs
- **ADR-027:** Builder Pattern Architecture Implementation
- **Microsoft Guidelines:** M-MOCKABLE-SYSCALLS (testable I/O)

## Notes
- Critical path for making framework functional
- Enables real operation execution
- Unblocks integration testing
- Should use tokio for all async I/O
- Platform-specific code should be clearly marked
- Error messages should be descriptive and actionable
