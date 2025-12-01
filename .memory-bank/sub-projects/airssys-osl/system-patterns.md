# airssys-osl System Patterns

## Architecture Overview

### Layered Architecture
```
┌─────────────────────────────────────────┐
│           Application Layer             │ ← Client Applications
├─────────────────────────────────────────┤
│           API Abstraction Layer         │ ← High-level APIs
├─────────────────────────────────────────┤
│         Security & Policy Layer         │ ← Security enforcement
├─────────────────────────────────────────┤
│         Activity Logging Layer          │ ← Comprehensive logging
├─────────────────────────────────────────┤
│          Platform Abstraction           │ ← OS-specific implementations
├─────────────────────────────────────────┤
│          Operating System               │ ← Linux, macOS, Windows
└─────────────────────────────────────────┘
```

### Core Modules
```
airssys-osl/
├── fs/              # Filesystem operations
├── process/         # Process management
├── network/         # Network operations
├── utils/           # External tool integration
├── security/        # Security policies and enforcement
├── logging/         # Activity logging system
├── platform/        # Platform-specific implementations
└── config/          # Configuration management
```

## Design Patterns

### Security-First Pattern
```rust
use std::path::Path;
use chrono::{DateTime, Utc};

use crate::security::SecurityContext;
use crate::logging::ActivityLog;

pub struct SecureOperation<T> {
    context: SecurityContext,
    logger: ActivityLog,
    operation: T,
}

impl<T> SecureOperation<T> {
    pub async fn execute(&self) -> Result<T::Output, OSLError> 
    where
        T: Operation,
    {
        // 1. Security policy check
        self.context.check_permission(&self.operation)?;
        
        // 2. Pre-operation logging
        self.logger.log_operation_start(&self.operation).await?;
        
        // 3. Execute with monitoring
        let result = self.operation.execute().await;
        
        // 4. Post-operation logging
        self.logger.log_operation_result(&self.operation, &result).await?;
        
        result
    }
}
```

### Resource Management Pattern
```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct ResourceManager {
    file_handles: Arc<Semaphore>,
    network_connections: Arc<Semaphore>,
    processes: Arc<Semaphore>,
}

impl ResourceManager {
    pub fn new(config: &ResourceConfig) -> Self {
        Self {
            file_handles: Arc::new(Semaphore::new(config.max_file_handles)),
            network_connections: Arc::new(Semaphore::new(config.max_connections)),
            processes: Arc::new(Semaphore::new(config.max_processes)),
        }
    }
    
    pub async fn acquire_file_handle(&self) -> ResourceGuard {
        let permit = self.file_handles.acquire().await.unwrap();
        ResourceGuard::new(permit, ResourceType::FileHandle)
    }
}
```

### Cross-Platform Abstraction Pattern
```rust
pub trait PlatformFileSystem {
    async fn create_file(&self, path: &Path) -> Result<FileHandle, FSError>;
    async fn read_dir(&self, path: &Path) -> Result<DirIterator, FSError>;
    async fn set_permissions(&self, path: &Path, perms: Permissions) -> Result<(), FSError>;
}

#[cfg(unix)]
mod unix_impl {
    use super::*;
    
    pub struct UnixFileSystem;
    
    impl PlatformFileSystem for UnixFileSystem {
        async fn create_file(&self, path: &Path) -> Result<FileHandle, FSError> {
            // Unix-specific implementation
        }
    }
}

#[cfg(windows)]
mod windows_impl {
    use super::*;
    
    pub struct WindowsFileSystem;
    
    impl PlatformFileSystem for WindowsFileSystem {
        async fn create_file(&self, path: &Path) -> Result<FileHandle, FSError> {
            // Windows-specific implementation
        }
    }
}
```

## Key Technical Decisions

### Async-First Design
- All I/O operations use async/await patterns
- Tokio runtime for async execution
- Structured concurrency with proper cancellation
- Resource pooling for efficient async operation management

### Security Policy Engine
- YAML-based security policy configuration
- Runtime policy evaluation and enforcement
- Hierarchical policy inheritance
- Policy validation and testing framework

### Activity Logging System
- Structured logging with JSON output format
- Configurable log levels and filtering
- Async logging to prevent I/O blocking
- Integration with external monitoring systems

### Error Handling Strategy
- Structured errors using `thiserror`
- Contextual error information with operation details
- Error recovery and retry mechanisms
- Comprehensive error logging and reporting

## Integration Patterns

### airssys-rt Integration
```rust
use crate::process::ProcessManager;
use airssys_rt::Actor;

pub struct OSLProcessActor {
    process_mgr: ProcessManager,
}

impl Actor for OSLProcessActor {
    async fn handle_message(&mut self, msg: ProcessMessage) -> Result<(), ActorError> {
        match msg {
            ProcessMessage::Spawn(cmd) => {
                let handle = self.process_mgr.spawn_secure(cmd).await?;
                // Actor lifecycle management integration
            }
        }
    }
}
```

### airssys-wasm Integration
```rust
use crate::security::SandboxConfig;
use airssys_wasm::ComponentHost;

pub struct OSLSandbox {
    security_context: SecurityContext,
}

impl OSLSandbox {
    pub fn create_wasm_sandbox(&self, config: SandboxConfig) -> Result<ComponentHost, OSLError> {
        // Provide OS-level isolation for WASM components
        let host = ComponentHost::new_with_security(config, &self.security_context)?;
        Ok(host)
    }
}
```

## Performance Patterns

### Zero-Copy Operations
- Use memory mapping for large file operations
- Splice operations for efficient data transfer
- Buffer pooling to reduce allocations
- Copy-on-write semantics where appropriate

### Resource Pooling
- Connection pools for network operations
- Thread pools for CPU-intensive tasks
- File descriptor pooling
- Memory buffer reuse

### Monitoring and Metrics
- Performance counters for all operations
- Resource utilization tracking
- Latency and throughput measurements
- Integration with metrics collection systems