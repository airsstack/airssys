# ADR-RT-007: Hierarchical Supervisor Architecture for OSL Integration

**ADR ID:** ADR-RT-007  
**Created:** 2025-10-11  
**Updated:** 2025-10-11  
**Status:** Accepted  
**Deciders:** Architecture team, RT-TASK-009 planning team  

---

## Title
Hierarchical Supervisor Architecture with Dedicated OSL Supervisor

---

## Context

### Problem Statement

The airssys-rt (actor runtime) needs to integrate with airssys-osl (OS layer framework) to provide actors with cross-platform OS operation capabilities. We must decide the architectural pattern for this integration:

1. **Direct OSL helper usage**: Application actors call OSL helpers directly
2. **OSL middleware in actor framework**: Framework-level OSL integration
3. **OSL Integration Actors with dedicated supervisor**: Service-oriented architecture
4. **Mixed approach**: Some actors use helpers, others use dedicated actors

This decision fundamentally affects system architecture, testability, maintainability, and fault tolerance.

### Business Context

- **RT-TASK-009**: OSL Integration task requires clean integration pattern
- **Production Requirements**: Clear fault isolation, observable OS operations, centralized security
- **Future Requirements**: Must support WASM integration, external service actors
- **YAGNI Decision**: Focus on in-memory actors first (deferred process group management)

### Technical Context

**Current State:**
- airssys-osl: Production-ready with helper functions (OSL-TASK-009 complete)
- airssys-rt: Complete supervisor framework (RT-TASK-007 complete)
- No OSL integration exists yet
- Helper functions available: `read_file()`, `write_file()`, `spawn_process()`, `tcp_connect()`, etc.

**Related ADRs:**
- ADR-RT-004: Child Trait Separation (enables diverse supervised entities)
- ADR-RT-001: Zero-Cost Abstractions (performance constraints)

**Workspace Standards:**
- §6.1: YAGNI principles - build only what's needed
- §6.2: Avoid `dyn` patterns - prefer static dispatch
- §6.3: Microsoft Rust Guidelines - service `Clone` pattern, essential functions in inherent methods

**Stakeholders:**
- Application developers using RT actors with OS operations
- System integrators building RT-based services
- Security team requiring audit trails
- Operations team monitoring production systems

---

## Decision

### Summary

**ACCEPTED: Hierarchical Supervisor Architecture with Dedicated OSL Supervisor**

Implement a **service-oriented architecture** with a dedicated `OSLSupervisor` managing specialized OSL integration actors (`FileSystemActor`, `ProcessActor`, `NetworkActor`). Application actors communicate with OSL actors via message passing across supervisor boundaries.

**Architecture:**
```
RootSupervisor
├── OSLSupervisor (manages OS integration actors)
│   ├── FileSystemActor (all file/directory operations)
│   ├── ProcessActor (all process spawning/management)
│   └── NetworkActor (all network connections)
└── ApplicationSupervisor (manages business logic actors)
    ├── WorkerActor
    ├── AggregatorActor
    └── CoordinatorActor
```

**Communication Pattern:**
- Application actors send messages to OSL actors
- Cross-supervisor message passing supported (standard actor messaging)
- Request-response pattern with timeout protection
- Fire-and-forget pattern for non-critical operations

### Rationale

**1. Clean Fault Isolation** ⭐⭐⭐⭐⭐
- OSL failures don't cascade to application actors
- Application failures don't affect OSL infrastructure
- Independent restart policies per supervisor
- Localized failure domains

**2. Service-Oriented Design** ⭐⭐⭐⭐⭐
- OSL actors are stateless services
- Application actors are business logic components
- Clear service boundaries and contracts
- Aligns with microservices principles

**3. Centralized Management** ⭐⭐⭐⭐⭐
- Single source of truth for OS operations
- Easy to monitor: Check OSL actor metrics
- Easy to audit: All operations logged in OSL actors
- Easy to secure: Security enforcement in one place

**4. Superior Testability** ⭐⭐⭐⭐⭐
- Application actors testable with mock OSL actors
- OSL actors testable with mock OSL client
- No real OS operations needed in most tests
- Clear mocking boundaries

**5. Performance Optimization Opportunities** ⭐⭐⭐⭐
- Connection pooling in NetworkActor
- Request batching in FileSystemActor
- Rate limiting per OSL actor
- Mailbox backpressure protection

**6. BEAM/OTP Alignment** ⭐⭐⭐⭐⭐
- Erlang/OTP uses supervision trees for services
- Separate supervisors for different subsystems
- Standard pattern in Erlang production systems
- Well-understood failure semantics

**7. Future Extensibility** ⭐⭐⭐⭐
- Easy to add new OSL actors (DatabaseActor, CryptoActor)
- Easy to add new application supervisors
- WASM integration follows same pattern
- External service integration (HTTP APIs, databases)

### Assumptions

- Most OS operations are infrequent (not hot path)
- Message passing overhead is acceptable (<1% of operation cost)
- OSL actors can handle concurrent requests efficiently
- Application actors prefer async operations
- Cross-supervisor communication is well-supported

---

## Considered Options

### Option 1: Direct OSL Helper Usage

**Description:** Application actors call OSL helper functions directly (e.g., `osl::read_file()`).

**Example:**
```rust
impl WorkerActor {
    async fn handle(&mut self, msg: Message) -> Result<()> {
        let data = osl::read_file("config.txt").await?;
        self.process(data)?;
        Ok(())
    }
}
```

**Pros:**
- ✅ Simple implementation
- ✅ Direct function calls (no message passing overhead)
- ✅ Familiar pattern for developers

**Cons:**
- ❌ Tight coupling to OSL implementation
- ❌ Hard to test (requires real OS operations or complex mocking)
- ❌ Mixed concerns (business logic + OS operations)
- ❌ Scattered OS operations across many actors
- ❌ No centralized audit trail
- ❌ Duplicate error handling everywhere
- ❌ Process lifecycle issues (zombie processes if actor crashes)

**Verdict:** ❌ **Rejected** - Poor separation of concerns, testability issues

---

### Option 2: OSL Middleware in Actor Framework

**Description:** Actor framework provides OSL capabilities via context or middleware.

**Example:**
```rust
impl Actor for WorkerActor {
    async fn handle(&mut self, msg: Message, ctx: &ActorContext) -> Result<()> {
        let data = ctx.read_file("config.txt").await?;  // Framework-provided
        self.process(data)?;
        Ok(())
    }
}
```

**Pros:**
- ✅ OSL capabilities available to all actors
- ✅ Framework-level abstraction
- ✅ Consistent API across actors

**Cons:**
- ❌ Tight coupling between actor framework and OSL layer
- ❌ Violates separation of concerns
- ❌ Framework becomes heavyweight
- ❌ Hard to change OSL implementation
- ❌ Still requires real OS operations in tests
- ❌ No centralized management or monitoring
- ❌ Over-engineering (not YAGNI compliant)

**Verdict:** ❌ **Rejected** - Over-coupling, violates YAGNI

---

### Option 3: Hierarchical Supervisors with OSL Integration Actors (SELECTED)

**Description:** Dedicated `OSLSupervisor` manages specialized OSL actors. Application actors communicate via messages.

**Example:**
```rust
// Setup
let osl_supervisor = OSLSupervisor::new();
osl_supervisor.start_child(FileSystemActor::new()).await?;

let app_supervisor = AppSupervisor::new();
app_supervisor.start_child(WorkerActor::new(fs_actor_ref)).await?;

// Usage
impl WorkerActor {
    async fn handle(&mut self, msg: Message) -> Result<()> {
        let (tx, rx) = oneshot::channel();
        self.fs_actor.send(ReadFileMessage { 
            path: "config.txt".into(), 
            respond_to: tx 
        }).await?;
        
        let response = rx.await?;
        match response {
            FileSystemResponse::ReadSuccess { content } => {
                self.process(content)?;
                Ok(())
            }
            FileSystemResponse::Error { error } => Err(error.into()),
        }
    }
}
```

**Pros:**
- ✅ Clean separation: OSL actors vs. application actors
- ✅ Fault isolation: separate supervisors
- ✅ Easy to test: mock OSL actors
- ✅ Centralized management: single FileSystemActor handles all file ops
- ✅ Observable: metrics on OSL actor performance
- ✅ Secure: centralized security enforcement
- ✅ BEAM/OTP aligned: supervision trees for services
- ✅ Extensible: easy to add new OSL actors
- ✅ Process lifecycle safety: ProcessActor cleans up on shutdown
- ✅ Performance opportunities: pooling, batching, rate limiting

**Cons:**
- ❌ Message passing overhead (~1% of operation cost)
- ❌ More complex setup (supervisor hierarchy)
- ❌ Request-response pattern requires channels

**Verdict:** ✅ **SELECTED** - Best separation of concerns, testability, fault tolerance

---

### Option 4: Mixed Approach

**Description:** Allow both direct OSL helpers and OSL actors, let developers choose.

**Pros:**
- ✅ Flexibility for developers
- ✅ Optimize hot paths with direct calls

**Cons:**
- ❌ Inconsistent patterns across codebase
- ❌ Confusion about best practices
- ❌ Hard to maintain: two patterns for same thing
- ❌ Violates principle of least surprise

**Verdict:** ❌ **Rejected** - Inconsistency, maintenance burden

---

## Architectural Details

### Supervisor Hierarchy Structure

**Root Supervisor:**
```rust
pub struct RootSupervisor {
    id: SupervisorId,
    strategy: RestartStrategy,
    children: Vec<Box<dyn Child>>,
}

// Root supervisor manages two main branches
impl RootSupervisor {
    pub async fn setup() -> Result<Self> {
        let root = SupervisorNode::builder()
            .with_id("root")
            .with_strategy(RestartStrategy::OneForOne)
            .build();
        
        // Branch 1: OSL infrastructure
        let osl_supervisor = Self::create_osl_supervisor();
        root.start_child(osl_supervisor).await?;
        
        // Branch 2: Application logic
        let app_supervisor = Self::create_app_supervisor();
        root.start_child(app_supervisor).await?;
        
        Ok(root)
    }
}
```

**OSL Supervisor:**
```rust
impl RootSupervisor {
    fn create_osl_supervisor() -> OSLSupervisor {
        let supervisor = SupervisorNode::builder()
            .with_id("osl-supervisor")
            .with_strategy(RestartStrategy::OneForOne)  // Independent restarts
            .build();
        
        // OSL integration actors
        supervisor.add_child_factory(|| FileSystemActor::new(osl_client.clone()));
        supervisor.add_child_factory(|| ProcessActor::new(osl_client.clone()));
        supervisor.add_child_factory(|| NetworkActor::new(osl_client.clone()));
        
        supervisor
    }
}
```

**Application Supervisor:**
```rust
impl RootSupervisor {
    fn create_app_supervisor(
        fs_actor: ActorRef<FileSystemActor>,
        process_actor: ActorRef<ProcessActor>,
        network_actor: ActorRef<NetworkActor>,
    ) -> ApplicationSupervisor {
        let supervisor = SupervisorNode::builder()
            .with_id("app-supervisor")
            .with_strategy(RestartStrategy::RestForOne)  // Dependency-aware
            .build();
        
        // Application actors (with OSL actor references)
        supervisor.add_child_factory(move || {
            WorkerActor::new(fs_actor.clone(), process_actor.clone())
        });
        
        supervisor.add_child_factory(move || {
            AggregatorActor::new(network_actor.clone())
        });
        
        supervisor
    }
}
```

### Cross-Supervisor Communication

**Message Passing Across Boundaries:**
```rust
// Application actor sends message to OSL actor
impl WorkerActor {
    fs_actor: ActorRef<FileSystemActor>,  // Reference to OSL actor
}

impl Actor for WorkerActor {
    async fn handle(&mut self, msg: Message) -> Result<()> {
        // Send message across supervisor boundary
        self.fs_actor.send(FileSystemMessage::ReadFile { 
            path: "data.txt".into(),
            respond_to: self.response_channel(),
        }).await?;
        
        Ok(())
    }
}
```

**No Special Handling Required:**
- Actor messaging works across supervisor boundaries
- Supervisors don't restrict message routing
- Standard `ActorRef` messaging API
- Transparent cross-supervisor communication

### Failure Isolation

**Scenario 1: OSL Actor Crashes**
```
FileSystemActor crashes
↓
OSLSupervisor detects failure
↓
OSLSupervisor restarts FileSystemActor (OneForOne strategy)
↓
ApplicationSupervisor and app actors UNAFFECTED
↓
App actors experience temporary message delivery failure
↓ (retry with timeout)
App actors resume normal operation when FileSystemActor restarts
```

**Scenario 2: Application Actor Crashes**
```
WorkerActor crashes
↓
ApplicationSupervisor detects failure
↓
ApplicationSupervisor restarts WorkerActor (or dependent actors with RestForOne)
↓
OSLSupervisor and OSL actors UNAFFECTED
↓
OSL actors continue serving other application actors
```

**Scenario 3: OSL Supervisor Crashes**
```
OSLSupervisor crashes (rare - supervisor failure)
↓
RootSupervisor detects failure
↓
RootSupervisor restarts OSLSupervisor
↓
OSLSupervisor restarts all OSL actors
↓
Application actors reconnect to new OSL actors
```

---

## OSL Integration Actor Specifications

### FileSystemActor

**Responsibilities:**
- File I/O operations (read, write, append, delete)
- Directory operations (create, remove, list, traverse)
- File metadata queries (stat, permissions, ownership)
- Centralized file operation audit logging

**Message Protocol:**
```rust
pub enum FileSystemMessage {
    ReadFile { path: PathBuf, respond_to: Sender<FileSystemResponse> },
    WriteFile { path: PathBuf, content: Vec<u8>, respond_to: Sender<FileSystemResponse> },
    AppendFile { path: PathBuf, content: Vec<u8>, respond_to: Option<Sender<FileSystemResponse>> },
    DeleteFile { path: PathBuf, respond_to: Sender<FileSystemResponse> },
    ListDirectory { path: PathBuf, respond_to: Sender<FileSystemResponse> },
    CreateDirectory { path: PathBuf, respond_to: Sender<FileSystemResponse> },
}
```

**Performance Optimizations:**
- Request batching for small writes
- Read-ahead caching for sequential reads
- Mailbox backpressure for rate limiting

### ProcessActor

**Responsibilities:**
- OS process spawning and management
- Process lifecycle tracking (spawned, running, stopped)
- Process termination (graceful + forced)
- Resource cleanup on actor shutdown
- Centralized process audit logging

**Message Protocol:**
```rust
pub enum ProcessMessage {
    Spawn { program: PathBuf, args: Vec<String>, respond_to: Sender<ProcessResponse> },
    Kill { pid: u32, signal: Signal, respond_to: Sender<ProcessResponse> },
    WaitFor { pid: u32, respond_to: Sender<ProcessResponse> },
}
```

**Lifecycle Management:**
```rust
impl Child for ProcessActor {
    async fn stop(&mut self) -> Result<()> {
        // Cleanup all spawned processes
        for (pid, handle) in &self.processes {
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

### NetworkActor

**Responsibilities:**
- TCP/UDP connections
- HTTP client operations
- Network resource management
- Connection pooling
- Centralized network audit logging

**Message Protocol:**
```rust
pub enum NetworkMessage {
    TcpConnect { host: String, port: u16, respond_to: Sender<NetworkResponse> },
    HttpGet { url: String, respond_to: Sender<NetworkResponse> },
    HttpPost { url: String, body: Vec<u8>, respond_to: Sender<NetworkResponse> },
}
```

**Performance Optimizations:**
- Connection pooling (reuse TCP connections)
- HTTP/2 multiplexing
- DNS caching

---

## Implementation Guidelines

### Phase 1: OSL Actor Development (RT-TASK-009 Phase 1)

**Deliverables:**
- FileSystemActor implementation
- ProcessActor implementation
- NetworkActor implementation
- Message protocol definitions
- Basic tests (unit + integration)

**Estimated Effort:** 3-4 days

### Phase 2: Supervisor Integration (RT-TASK-009 Phase 2)

**Deliverables:**
- OSLSupervisor setup
- Hierarchical supervisor configuration
- Failure isolation validation
- Cross-supervisor communication tests

**Estimated Effort:** 2-3 days

### Phase 3: Examples and Documentation (RT-TASK-009 Phase 3)

**Deliverables:**
- Example application using OSL actors
- Migration guide from direct OSL helpers
- Performance benchmarks
- Security audit documentation

**Estimated Effort:** 2-3 days

### Phase 4: Testing and Validation (RT-TASK-009 Phase 4)

**Deliverables:**
- Comprehensive test suite
- Failure scenario tests
- Performance regression tests
- Security penetration tests

**Estimated Effort:** 2-3 days

---

## Testing Strategy

### Unit Testing

**Test OSL Actors with Mock OSL Client:**
```rust
#[tokio::test]
async fn test_filesystem_actor_read() {
    let mut mock_osl = MockOslClient::new();
    mock_osl.expect_read_file()
        .with(eq("/test.txt"))
        .returning(|_| Ok(b"content".to_vec()));
    
    let actor = FileSystemActor::new(Arc::new(mock_osl));
    // Test actor behavior
}
```

### Integration Testing

**Test Cross-Supervisor Communication:**
```rust
#[tokio::test]
async fn test_app_to_osl_communication() {
    let root = setup_supervisors().await?;
    
    // Send message from app actor to OSL actor
    let worker = root.get_actor::<WorkerActor>("worker")?;
    worker.send(ProcessDataMessage { data: "test" }).await?;
    
    // Verify OSL operation executed
    let fs_actor = root.get_actor::<FileSystemActor>("fs-actor")?;
    assert_eq!(fs_actor.operation_count(), 1);
}
```

### Failure Testing

**Test Fault Isolation:**
```rust
#[tokio::test]
async fn test_osl_actor_crash_isolation() {
    let root = setup_supervisors().await?;
    
    // Crash FileSystemActor
    crash_actor::<FileSystemActor>(&root).await?;
    
    // Verify app actors still running
    let worker = root.get_actor::<WorkerActor>("worker")?;
    assert!(worker.is_alive());
    
    // Verify FileSystemActor restarted
    tokio::time::sleep(Duration::from_millis(100)).await;
    let fs_actor = root.get_actor::<FileSystemActor>("fs-actor")?;
    assert!(fs_actor.is_alive());
}
```

---

## Security Considerations

### Security Context Propagation

**Application Actor → OSL Actor:**
```rust
impl WorkerActor {
    async fn handle(&mut self, msg: Message) -> Result<()> {
        let security_ctx = msg.security_context();  // From incoming message
        
        self.fs_actor.send(FileSystemMessage::ReadFile {
            path: "data.txt".into(),
            security_context: security_ctx,  // Propagate context
            respond_to: self.response_channel(),
        }).await?;
        
        Ok(())
    }
}
```

### Audit Logging

**Centralized in OSL Actors:**
```rust
impl FileSystemActor {
    async fn handle_read_file(&mut self, req: ReadFileRequest) -> Result<()> {
        // Centralized audit logging
        audit_log!(
            operation = "read_file",
            path = %req.path,
            actor_id = %req.requester,
            security_context = ?req.security_context,
        );
        
        let result = osl::read_file(&req.path).await?;
        Ok(result)
    }
}
```

---

## Performance Impact

### Message Passing Overhead

**Estimated Cost:**
- Actor message send: ~500ns - 1μs
- Typical file operation: 100μs - 10ms
- **Overhead: <1% of operation cost**

### Mitigation Strategies

1. **Request Batching**: Batch small operations
2. **Connection Pooling**: Reuse network connections
3. **Caching**: Cache frequently accessed data
4. **Async Processing**: Non-blocking operations

---

## Migration Path

### Phase 1: Introduce OSL Actors (Parallel)

Introduce OSL actors alongside existing direct OSL usage:
```rust
// Existing code continues to work
osl::read_file("config.txt").await?;

// New pattern available
fs_actor.send(ReadFileMessage { ... }).await?;
```

### Phase 2: Migrate Application Actors

Gradually migrate actors to OSL actors:
```rust
// Before
impl WorkerActor {
    async fn handle(&mut self) -> Result<()> {
        osl::read_file("data.txt").await?;
    }
}

// After
impl WorkerActor {
    async fn handle(&mut self) -> Result<()> {
        self.fs_actor.send(ReadFileMessage { ... }).await?;
    }
}
```

### Phase 3: Deprecate Direct OSL Usage in Actors

Mark direct OSL helper usage as deprecated in actor context:
```rust
#[deprecated(note = "Use FileSystemActor instead")]
pub async fn read_file_from_actor(path: &Path) -> Result<Vec<u8>> {
    osl::read_file(path).await
}
```

---

## Consequences

### Positive Consequences

1. **Clean Architecture**: Clear separation between infrastructure and application
2. **Fault Tolerance**: Independent failure domains with supervisor isolation
3. **Testability**: Easy to mock OSL actors in tests
4. **Observability**: Centralized metrics and audit logging
5. **Security**: Centralized security enforcement
6. **Extensibility**: Easy to add new OSL actors or application supervisors
7. **BEAM Alignment**: Follows Erlang/OTP best practices

### Negative Consequences

1. **Complexity**: More moving parts (supervisors, actors, message protocols)
2. **Message Overhead**: ~1% performance cost for message passing
3. **Learning Curve**: Developers must understand supervisor hierarchy
4. **Setup Boilerplate**: More code to setup supervisor hierarchy

### Mitigation Strategies

- **Complexity**: Provide builder patterns and helper functions
- **Performance**: Benchmark and optimize hot paths
- **Learning Curve**: Comprehensive documentation and examples
- **Boilerplate**: Create setup templates and macros

---

## Related Decisions

**Related ADRs:**
- ADR-RT-004: Child Trait Separation (enables diverse supervised entities)
- ADR-RT-001: Zero-Cost Abstractions (performance constraints)

**Related Knowledge Docs:**
- KNOWLEDGE-RT-016: Process Group Management - Future Considerations
- KNOWLEDGE-RT-017: OSL Integration Actors Pattern

**Related Tasks:**
- RT-TASK-009: OSL Integration Implementation
- OSL-TASK-010: Helper Function Middleware Integration

---

## Review and Updates

**Next Review Date:** 2025-12-01 (after RT-TASK-009 completion)  
**Trigger for Review:**
- Performance issues with message passing
- Developer feedback on complexity
- New integration requirements (WASM, external services)

---

## Key Takeaways

1. **Hierarchical supervisors** provide clean fault isolation
2. **OSL Integration Actors** (FileSystemActor, ProcessActor, NetworkActor) centralize OS operations
3. **Cross-supervisor communication** works via standard actor messaging
4. **Service-oriented design** separates infrastructure from business logic
5. **BEAM/OTP alignment** ensures proven fault-tolerance patterns
6. **YAGNI compliant** - focuses on in-memory actors, defers process group management

---

**Status:** ✅ **ACCEPTED**  
**Implementation:** RT-TASK-009 (Estimated 9-13 days)  
**Last Updated:** 2025-10-11
