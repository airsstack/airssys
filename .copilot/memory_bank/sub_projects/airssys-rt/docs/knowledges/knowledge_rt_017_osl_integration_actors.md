# KNOWLEDGE-RT-017: OSL Integration Actors Pattern

**Created:** 2025-10-11  
**Updated:** 2025-10-11  
**Status:** Active - Recommended Pattern  
**Related Tasks:** RT-TASK-009 (OSL Integration)  
**Related ADRs:** ADR-RT-007 (Hierarchical Supervisor Architecture)

---

## Overview

This knowledge document describes the **OSL Integration Actors Pattern** - the recommended architectural approach for integrating airssys-rt (actor runtime) with airssys-osl (OS layer framework). This pattern provides clean separation of concerns, centralized OS operation management, and superior testability compared to direct OSL helper usage in application actors.

**Core Principle:** Create dedicated actors that serve as the interface between the actor runtime and OS layer operations.

---

## Problem Statement

### Integration Challenges

**Challenge 1: Mixed Concerns in Application Actors**
```rust
// ❌ Anti-pattern: Application actor directly calling OSL
pub struct WorkerActor {
    osl: Arc<OslClient>,
    business_data: Data,
}

impl Actor for WorkerActor {
    async fn handle(&mut self, msg: Message) -> Result<()> {
        // Business logic mixed with OS operations
        let data = self.process_data(msg)?;
        
        // Direct OSL call - tight coupling
        osl::write_file("output.txt", &data).await?;
        
        // More business logic
        self.update_state(data)?;
        Ok(())
    }
}

// Problems:
// 1. Tight coupling to OSL implementation
// 2. Hard to test (requires real OS operations)
// 3. Security context unclear
// 4. No centralized audit trail
// 5. Duplicate error handling across actors
```

**Challenge 2: Process Lifecycle Management**
```rust
// ❌ Anti-pattern: Each actor manages its own OS processes
pub struct DataProcessorActor {
    spawned_processes: Vec<u32>,  // PIDs
}

impl DataProcessorActor {
    async fn spawn_helper(&mut self) -> Result<()> {
        let pid = osl::spawn_process("helper.sh").await?;
        self.spawned_processes.push(pid);
        // Problem: Who cleans up if actor crashes?
        Ok(())
    }
}
```

**Challenge 3: Scattered OS Operation Logic**
- File operations in 10 different actors
- Network calls in 5 different actors
- Process spawning in 3 different actors
- **Result:** No single source of truth, hard to monitor, hard to secure

---

## Solution Architecture

### Pattern Overview

**Dedicated OSL Integration Actors:**
```
┌─────────────────────────────────────────────────────┐
│                 RootSupervisor                      │
├─────────────────────┬───────────────────────────────┤
│                     │                               │
│  OSLSupervisor      │     ApplicationSupervisor     │
│  (OSL Integration)  │     (Business Logic)          │
│                     │                               │
│  ┌──────────────┐   │   ┌──────────────┐           │
│  │ FileSystem   │   │   │ Worker       │           │
│  │ Actor        │◄──┼───│ Actor        │           │
│  └──────────────┘   │   └──────────────┘           │
│                     │                               │
│  ┌──────────────┐   │   ┌──────────────┐           │
│  │ Process      │   │   │ Aggregator   │           │
│  │ Actor        │◄──┼───│ Actor        │           │
│  └──────────────┘   │   └──────────────┘           │
│                     │                               │
│  ┌──────────────┐   │   ┌──────────────┐           │
│  │ Network      │   │   │ Coordinator  │           │
│  │ Actor        │◄──┼───│ Actor        │           │
│  └──────────────┘   │   └──────────────┘           │
│                     │                               │
└─────────────────────┴───────────────────────────────┘

Legend:
◄── Message passing (actors send requests to OSL actors)
```

### Three Core OSL Integration Actors

#### 1. FileSystemActor

**Responsibilities:**
- All file system operations (read, write, delete, list)
- Directory management (create, remove, traverse)
- File metadata queries (stat, permissions, ownership)
- Centralized file operation audit logging

**Message Protocol:**
```rust
pub enum FileSystemMessage {
    ReadFile { path: PathBuf, respond_to: Sender<FileSystemResponse> },
    WriteFile { path: PathBuf, content: Vec<u8>, respond_to: Sender<FileSystemResponse> },
    DeleteFile { path: PathBuf, respond_to: Sender<FileSystemResponse> },
    ListDirectory { path: PathBuf, respond_to: Sender<FileSystemResponse> },
    CreateDirectory { path: PathBuf, respond_to: Sender<FileSystemResponse> },
}

pub enum FileSystemResponse {
    ReadSuccess { content: Vec<u8> },
    WriteSuccess,
    DeleteSuccess,
    ListSuccess { entries: Vec<DirEntry> },
    Error { error: FileSystemError },
}
```

**Implementation:**
```rust
pub struct FileSystemActor {
    osl: Arc<OslClient>,
    operation_count: u64,
    active_operations: HashMap<OperationId, Operation>,
}

impl Actor for FileSystemActor {
    async fn handle(&mut self, msg: Message) -> Result<()> {
        let fs_msg: FileSystemMessage = msg.deserialize()?;
        
        match fs_msg {
            FileSystemMessage::ReadFile { path, respond_to } => {
                self.operation_count += 1;
                
                // Use OSL helper (security + audit built-in)
                match osl::read_file(&path).await {
                    Ok(content) => {
                        respond_to.send(FileSystemResponse::ReadSuccess { content }).await?;
                    }
                    Err(e) => {
                        respond_to.send(FileSystemResponse::Error { 
                            error: FileSystemError::from(e) 
                        }).await?;
                    }
                }
            }
            
            FileSystemMessage::WriteFile { path, content, respond_to } => {
                self.operation_count += 1;
                
                match osl::write_file(&path, &content).await {
                    Ok(_) => {
                        respond_to.send(FileSystemResponse::WriteSuccess).await?;
                    }
                    Err(e) => {
                        respond_to.send(FileSystemResponse::Error { 
                            error: FileSystemError::from(e) 
                        }).await?;
                    }
                }
            }
            
            // ... other operations
        }
        
        Ok(())
    }
}
```

#### 2. ProcessActor

**Responsibilities:**
- OS process spawning and management
- Process lifecycle tracking (spawned, running, stopped)
- Process termination (graceful + forced)
- Resource cleanup on actor shutdown
- Centralized process audit logging

**Message Protocol:**
```rust
pub enum ProcessMessage {
    Spawn { 
        program: PathBuf, 
        args: Vec<String>,
        respond_to: Sender<ProcessResponse> 
    },
    Kill { 
        pid: u32, 
        signal: Signal,
        respond_to: Sender<ProcessResponse> 
    },
    WaitFor { 
        pid: u32,
        respond_to: Sender<ProcessResponse> 
    },
}

pub enum ProcessResponse {
    SpawnSuccess { pid: u32 },
    KillSuccess,
    WaitSuccess { exit_code: i32 },
    Error { error: ProcessError },
}
```

**Implementation:**
```rust
pub struct ProcessActor {
    osl: Arc<OslClient>,
    processes: HashMap<u32, ProcessHandle>,
}

pub struct ProcessHandle {
    pid: u32,
    program: String,
    spawned_at: DateTime<Utc>,
    spawned_by: ActorId,  // Track which actor requested spawn
}

impl ProcessActor {
    async fn handle_spawn(&mut self, req: SpawnRequest) -> Result<ProcessResponse> {
        let result = osl::spawn_process(req.program, req.args).await?;
        
        // Track process
        self.processes.insert(
            result.pid,
            ProcessHandle {
                pid: result.pid,
                program: req.program.display().to_string(),
                spawned_at: Utc::now(),
                spawned_by: req.requester_id,
            }
        );
        
        Ok(ProcessResponse::SpawnSuccess { pid: result.pid })
    }
}

impl Child for ProcessActor {
    async fn stop(&mut self) -> Result<()> {
        // Cleanup all spawned processes
        for (pid, handle) in &self.processes {
            tracing::info!(
                pid = *pid,
                program = %handle.program,
                "Cleaning up spawned process"
            );
            
            // Graceful termination attempt
            osl::kill_process(*pid, Signal::SIGTERM).await.ok();
        }
        
        // Wait for graceful shutdown
        tokio::time::sleep(Duration::from_secs(5)).await;
        
        // Force kill any remaining
        for (pid, _) in &self.processes {
            osl::kill_process(*pid, Signal::SIGKILL).await.ok();
        }
        
        Ok(())
    }
}
```

#### 3. NetworkActor

**Responsibilities:**
- TCP/UDP connections
- HTTP client operations
- Network resource management
- Connection pooling
- Centralized network audit logging

**Message Protocol:**
```rust
pub enum NetworkMessage {
    TcpConnect { 
        host: String, 
        port: u16,
        respond_to: Sender<NetworkResponse> 
    },
    HttpGet { 
        url: String,
        respond_to: Sender<NetworkResponse> 
    },
    // ... more operations
}

pub enum NetworkResponse {
    TcpConnected { stream_id: StreamId },
    HttpSuccess { status: u16, body: Vec<u8> },
    Error { error: NetworkError },
}
```

---

## Benefits Analysis

### 1. Separation of Concerns

**Before (Mixed Concerns):**
```rust
pub struct WorkerActor {
    business_data: Data,
    osl: Arc<OslClient>,  // ❌ OS layer dependency
}

impl Actor for WorkerActor {
    async fn handle(&mut self, msg: Message) -> Result<()> {
        // Business logic
        let result = self.process(msg.data)?;
        
        // OS operation - coupling!
        osl::write_file("output.txt", &result).await?;
        
        Ok(())
    }
}
```

**After (Clean Separation):**
```rust
pub struct WorkerActor {
    business_data: Data,
    fs_actor: ActorRef<FileSystemActor>,  // ✅ Actor reference
}

impl Actor for WorkerActor {
    async fn handle(&mut self, msg: Message) -> Result<()> {
        // Pure business logic
        let result = self.process(msg.data)?;
        
        // Message to OSL actor - clean!
        self.fs_actor.send(FileSystemMessage::WriteFile {
            path: "output.txt".into(),
            content: result,
            respond_to: self.response_channel(),
        }).await?;
        
        Ok(())
    }
}
```

### 2. Centralized Management

**Single Source of Truth:**
```rust
// All file operations go through FileSystemActor
// - Easy to monitor: Check FileSystemActor metrics
// - Easy to audit: All operations logged in one place
// - Easy to rate limit: Actor mailbox backpressure
// - Easy to secure: Security enforcement in one actor

impl FileSystemActor {
    pub fn metrics(&self) -> FileSystemMetrics {
        FileSystemMetrics {
            total_operations: self.operation_count,
            active_operations: self.active_operations.len(),
            errors_count: self.error_count,
        }
    }
}
```

### 3. Superior Testability

**Mock OSL Actors in Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_worker_without_real_filesystem() {
        // Create mock FileSystemActor
        let mock_fs = MockFileSystemActor::new();
        mock_fs.expect_write()
            .with(eq("output.txt"))
            .times(1)
            .returning(|_, _| Ok(()));
        
        // Test WorkerActor with mock
        let worker = WorkerActor::new(mock_fs.as_ref());
        worker.handle(test_message()).await.unwrap();
        
        // Verify business logic without real OS operations
        mock_fs.verify();
    }
}
```

### 4. Process Lifecycle Safety

**Centralized Cleanup:**
```rust
// ProcessActor automatically cleans up on shutdown
// No leaked processes even if application actors crash

let root = RootSupervisor::new();
let osl_supervisor = OSLSupervisor::new();

// Start ProcessActor under supervision
osl_supervisor.start_child(ProcessActor::new()).await?;

// Application actors crash → OSL supervisor still running
// OSL supervisor stops → ProcessActor.stop() cleans up all processes
```

### 5. Performance Optimization Opportunities

**Connection Pooling:**
```rust
pub struct NetworkActor {
    connection_pool: HashMap<String, Vec<TcpStream>>,
    max_connections_per_host: usize,
}

impl NetworkActor {
    async fn handle_tcp_connect(&mut self, host: String, port: u16) -> Result<StreamId> {
        // Check pool first
        if let Some(stream) = self.connection_pool.get_mut(&host).and_then(|v| v.pop()) {
            return Ok(stream.id());
        }
        
        // Create new connection
        let stream = osl::tcp_connect(&host, port).await?;
        Ok(stream.id())
    }
}
```

**Request Batching:**
```rust
// FileSystemActor can batch multiple small writes
pub struct FileSystemActor {
    pending_writes: Vec<WriteRequest>,
    batch_timer: Interval,
}

impl FileSystemActor {
    async fn maybe_flush_batch(&mut self) -> Result<()> {
        if self.pending_writes.len() >= 10 || self.batch_timer.elapsed() > Duration::from_millis(100) {
            // Batch write multiple files in one system call
            osl::write_files_batch(&self.pending_writes).await?;
            self.pending_writes.clear();
        }
        Ok(())
    }
}
```

---

## Implementation Guidelines

### Supervisor Hierarchy Setup

**Recommended Structure:**
```rust
use airssys_rt::supervisor::SupervisorNode;
use airssys_rt::supervisor::RestartStrategy;

pub async fn setup_application() -> Result<RootSupervisor> {
    // Root supervisor
    let root = SupervisorNode::builder()
        .with_id("root")
        .with_strategy(RestartStrategy::OneForOne)
        .build();
    
    // OSL supervisor - manages OS integration actors
    let osl_supervisor = SupervisorNode::builder()
        .with_id("osl-supervisor")
        .with_strategy(RestartStrategy::OneForOne)
        .build();
    
    // Start OSL integration actors
    let fs_actor = FileSystemActor::new(osl_client.clone());
    let process_actor = ProcessActor::new(osl_client.clone());
    let network_actor = NetworkActor::new(osl_client.clone());
    
    osl_supervisor.start_child(fs_actor).await?;
    osl_supervisor.start_child(process_actor).await?;
    osl_supervisor.start_child(network_actor).await?;
    
    // Application supervisor - manages business logic actors
    let app_supervisor = SupervisorNode::builder()
        .with_id("app-supervisor")
        .with_strategy(RestartStrategy::RestForOne)
        .build();
    
    // Start application actors (with references to OSL actors)
    let worker1 = WorkerActor::new(fs_actor.as_ref(), process_actor.as_ref());
    let worker2 = AggregatorActor::new(network_actor.as_ref());
    
    app_supervisor.start_child(worker1).await?;
    app_supervisor.start_child(worker2).await?;
    
    // Attach both supervisors to root
    root.start_child(osl_supervisor).await?;
    root.start_child(app_supervisor).await?;
    
    Ok(root)
}
```

### Message Passing Pattern

**Request-Response with Timeout:**
```rust
impl WorkerActor {
    async fn save_data(&mut self, data: Vec<u8>) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        
        // Send request to FileSystemActor
        self.fs_actor.send(FileSystemMessage::WriteFile {
            path: "data.bin".into(),
            content: data,
            respond_to: tx,
        }).await?;
        
        // Wait for response with timeout
        let response = tokio::time::timeout(
            Duration::from_secs(5),
            rx
        ).await??;
        
        match response {
            FileSystemResponse::WriteSuccess => Ok(()),
            FileSystemResponse::Error { error } => Err(error.into()),
            _ => Err(anyhow!("Unexpected response")),
        }
    }
}
```

**Fire-and-Forget (No Response Needed):**
```rust
impl LoggerActor {
    async fn log_to_file(&mut self, log: String) -> Result<()> {
        // Just send, don't wait for response
        self.fs_actor.send(FileSystemMessage::AppendFile {
            path: "app.log".into(),
            content: log.into_bytes(),
            respond_to: None,  // No response needed
        }).await?;
        
        Ok(())
    }
}
```

### Error Handling Strategy

**Layered Error Handling:**
```rust
// 1. OSL layer errors (system level)
pub enum OslError {
    FileNotFound,
    PermissionDenied,
    IoError(io::Error),
}

// 2. OSL actor errors (actor level)
pub enum FileSystemError {
    OslError(OslError),
    OperationTimeout,
    InvalidPath,
}

// 3. Application actor errors (business level)
pub enum BusinessError {
    FileSystemError(FileSystemError),
    InvalidData,
    ProcessingFailed,
}

// Application actors handle business errors, OSL actors handle system errors
impl WorkerActor {
    async fn process(&mut self, msg: Message) -> Result<(), BusinessError> {
        let data = self.validate(msg)?;  // Business logic
        
        // OSL operation
        self.fs_actor.send(write_request).await
            .map_err(|e| BusinessError::FileSystemError(e))?;
        
        Ok(())
    }
}
```

---

## Testing Strategy

### Unit Testing OSL Actors

**Test with Mock OSL Client:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    
    #[tokio::test]
    async fn test_filesystem_actor_read_success() {
        let mut mock_osl = MockOslClient::new();
        mock_osl.expect_read_file()
            .with(eq("/test.txt"))
            .times(1)
            .returning(|_| Ok(b"test content".to_vec()));
        
        let mut actor = FileSystemActor::new(Arc::new(mock_osl));
        
        let (tx, rx) = oneshot::channel();
        let msg = FileSystemMessage::ReadFile {
            path: "/test.txt".into(),
            respond_to: tx,
        };
        
        actor.handle(msg.into()).await.unwrap();
        
        let response = rx.await.unwrap();
        match response {
            FileSystemResponse::ReadSuccess { content } => {
                assert_eq!(content, b"test content");
            }
            _ => panic!("Expected ReadSuccess"),
        }
    }
}
```

### Integration Testing

**Test Actor Communication:**
```rust
#[tokio::test]
async fn test_worker_filesystem_integration() {
    let (actor_system, _) = ActorSystem::new();
    
    // Start real FileSystemActor (with real OSL)
    let fs_actor = actor_system.spawn(FileSystemActor::new(real_osl_client())).await?;
    
    // Start WorkerActor
    let worker = actor_system.spawn(WorkerActor::new(fs_actor.clone())).await?;
    
    // Send message to worker
    worker.send(ProcessDataMessage { data: "test".into() }).await?;
    
    // Verify file was created
    let content = tokio::fs::read("/tmp/output.txt").await?;
    assert_eq!(content, b"test");
}
```

---

## Performance Considerations

### Mailbox Sizing

**OSL actors may have high message volume:**
```rust
let fs_actor = FileSystemActor::builder()
    .with_mailbox_capacity(10000)  // High capacity
    .with_backpressure_strategy(BackpressureStrategy::DropOldest)
    .build();
```

### Concurrency Control

**Rate limiting for expensive operations:**
```rust
pub struct ProcessActor {
    semaphore: Arc<Semaphore>,  // Limit concurrent spawns
}

impl ProcessActor {
    async fn handle_spawn(&mut self, req: SpawnRequest) -> Result<ProcessResponse> {
        let _permit = self.semaphore.acquire().await?;
        
        // Only N concurrent process spawns allowed
        let result = osl::spawn_process(req.program, req.args).await?;
        
        Ok(ProcessResponse::SpawnSuccess { pid: result.pid })
    }
}
```

### Monitoring and Metrics

**OSL actor health metrics:**
```rust
pub struct OSLActorMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub average_latency: Duration,
    pub mailbox_size: usize,
}

impl FileSystemActor {
    pub fn metrics(&self) -> OSLActorMetrics {
        OSLActorMetrics {
            total_operations: self.operation_count,
            successful_operations: self.success_count,
            failed_operations: self.error_count,
            average_latency: self.latency_tracker.average(),
            mailbox_size: self.mailbox.len(),
        }
    }
}
```

---

## Migration Path

### Phase 1: Introduce OSL Actors (Parallel)

Add OSL actors alongside existing direct OSL usage:
```rust
// Keep existing
osl::read_file("config.txt").await?;

// Add new OSL actors
let fs_actor = FileSystemActor::new(osl);
supervisor.start_child(fs_actor).await?;
```

### Phase 2: Migrate Application Actors

Gradually migrate actors to use OSL actors:
```rust
// Before
impl WorkerActor {
    async fn handle(&mut self) -> Result<()> {
        osl::read_file("data.txt").await?;  // Direct call
    }
}

// After
impl WorkerActor {
    async fn handle(&mut self) -> Result<()> {
        self.fs_actor.send(ReadFileMessage { ... }).await?;  // Message
    }
}
```

### Phase 3: Deprecate Direct OSL Usage

Mark direct OSL helpers as deprecated in actor context:
```rust
#[deprecated(note = "Use FileSystemActor instead")]
pub async fn read_file_from_actor(path: &Path) -> Result<Vec<u8>> {
    osl::read_file(path).await
}
```

---

## Anti-Patterns to Avoid

### ❌ Anti-Pattern 1: Too Many OSL Actor Types

```rust
// ❌ DON'T: Create actor for every single operation
pub struct ReadFileActor { ... }
pub struct WriteFileActor { ... }
pub struct DeleteFileActor { ... }
// Too granular!

// ✅ DO: Group related operations
pub struct FileSystemActor {
    // Handles read, write, delete, list, etc.
}
```

### ❌ Anti-Pattern 2: Synchronous Blocking

```rust
// ❌ DON'T: Block actor thread waiting for response
impl WorkerActor {
    async fn handle(&mut self) -> Result<()> {
        let response = self.fs_actor.send(msg).await?;
        let result = response.recv().blocking_recv()?;  // ❌ Blocking!
    }
}

// ✅ DO: Use async waiting
impl WorkerActor {
    async fn handle(&mut self) -> Result<()> {
        let response = self.fs_actor.send(msg).await?;
        let result = response.recv().await?;  // ✅ Async
    }
}
```

### ❌ Anti-Pattern 3: Tight Coupling to OSL Actor Internals

```rust
// ❌ DON'T: Depend on OSL actor implementation details
impl WorkerActor {
    fn new(fs_actor: FileSystemActor) -> Self {  // ❌ Concrete type
        Self { fs_actor }
    }
}

// ✅ DO: Depend on abstraction
impl WorkerActor {
    fn new<T: FileSystemProvider>(fs_provider: T) -> Self {  // ✅ Trait
        Self { fs_provider }
    }
}
```

---

## Key Takeaways

1. **OSL Integration Actors Pattern** is the recommended approach for RT-OSL integration
2. **Three core actors**: FileSystemActor, ProcessActor, NetworkActor
3. **Hierarchical supervisors**: Separate OSL supervisor from application supervisors
4. **Message-based communication**: Clean, testable, observable
5. **Centralized management**: Single source of truth for OS operations
6. **Superior testability**: Mock OSL actors in tests
7. **Process safety**: Automatic cleanup on actor shutdown
8. **Performance opportunities**: Connection pooling, request batching, rate limiting

---

**Related Documentation:**
- KNOWLEDGE-RT-016: Process Group Management - Future Considerations
- ADR-RT-007: Hierarchical Supervisor Architecture
- RT-TASK-009: OSL Integration Implementation Plan

**Last Updated:** 2025-10-11  
**Status:** ✅ Recommended pattern for RT-TASK-009
