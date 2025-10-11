# KNOWLEDGE-RT-016: Process Group Management - Future Considerations

**Created:** 2025-10-11  
**Updated:** 2025-10-11  
**Status:** Deferred (YAGNI Decision)  
**Related Tasks:** RT-TASK-009 (OSL Integration)  
**Related ADRs:** TBD (when implemented)

---

## Overview

This knowledge document captures the architectural discussion and decision to **defer process group management** implementation in airssys-rt. While the concern about zombie processes and orphaned child processes is valid and important, we've decided to apply YAGNI (You Aren't Gonna Need It) principles and defer implementation until a proven use case emerges.

**Key Decision:** Focus on in-memory actor-based OSL integration pattern instead of complex OS process lifecycle management.

---

## Problem Statement

### Zombie Process Risk

**Scenario:**
```
User creates supervisor → supervises actor → actor spawns OS process
         ↓
     Actor crashes
         ↓
OS process becomes orphaned → accumulates as zombie → resource leak
```

**Example Case Study:**
```rust
// Actor spawns a bash script
let actor = MyActor::new();
supervisor.start_child(actor).await?;

// Inside actor
async fn handle_message(&mut self, msg: Message) {
    // Spawns OS process via OSL
    let pid = osl::spawn_process("long_running_script.sh").await?;
    // Actor state tracks PID
    self.child_pid = pid;
}

// Problem: Actor crashes → bash script keeps running
// No process group management → can't kill entire process tree
// Result: Zombie accumulation over time
```

### Current Implementation Gap

**airssys-osl/src/operations/process/spawn.rs:**
```rust
pub struct SpawnProcessOperation {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: Option<PathBuf>,
    // ❌ Missing: process_group: Option<ProcessGroupId>
    // ❌ Missing: create_new_process_group: bool
}

pub struct SpawnProcessResult {
    pub pid: u32,
    // ❌ Missing: pgid: Option<u32>
    // ❌ Missing: job_id: Option<JobId> (Windows)
}
```

**Key Gaps Identified:**
1. No `setpgid()` call in Linux/macOS process spawning
2. No job object creation in Windows process spawning
3. No process group ID (PGID) tracking in results
4. No API to kill entire process group
5. No supervisor hooks for child process cleanup

---

## Proposed Solution (Deferred)

### Phase 1: Process Group Management (Linux/macOS)

**Implementation Scope:**
```rust
// Enhanced SpawnProcessOperation
pub struct SpawnProcessOperation {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: Option<PathBuf>,
    pub process_group: ProcessGroupConfig,  // NEW
}

pub enum ProcessGroupConfig {
    /// Join parent's process group (default)
    Inherit,
    /// Create new process group (child becomes leader)
    NewGroup,
    /// Join specific process group
    JoinGroup(u32),
}

pub struct SpawnProcessResult {
    pub pid: u32,
    pub pgid: u32,  // NEW - always tracked
}

// New operation: Kill entire process group
pub struct KillProcessGroupOperation {
    pub pgid: u32,
    pub signal: Signal,
}
```

**Platform-Specific Implementation:**

**Linux/macOS (POSIX):**
```rust
use nix::unistd::{setpgid, Pid};
use nix::sys::signal::{killpg, Signal};

// In executor after fork
match config.process_group {
    ProcessGroupConfig::NewGroup => {
        setpgid(Pid::from_raw(0), Pid::from_raw(0))?; // Create new group
    }
    ProcessGroupConfig::JoinGroup(pgid) => {
        setpgid(Pid::from_raw(0), Pid::from_raw(pgid as i32))?;
    }
    ProcessGroupConfig::Inherit => {
        // Default behavior - do nothing
    }
}

// Kill process group
pub fn kill_process_group(pgid: u32, signal: Signal) -> Result<()> {
    killpg(Pid::from_raw(pgid as i32), signal)?;
    Ok(())
}
```

**Windows (Job Objects):**
```rust
use windows::Win32::System::JobObjects::*;

pub struct JobObject {
    handle: HANDLE,
}

impl JobObject {
    pub fn create() -> Result<Self> {
        let handle = unsafe { CreateJobObjectW(None, None)? };
        
        // Configure: terminate all processes when handle closed
        let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        
        unsafe {
            SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const _,
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )?;
        }
        
        Ok(JobObject { handle })
    }
    
    pub fn assign_process(&self, process_handle: HANDLE) -> Result<()> {
        unsafe { AssignProcessToJobObject(self.handle, process_handle)? };
        Ok(())
    }
}

// Cleanup: Close job handle → all processes terminated automatically
```

### Phase 2: Supervisor Integration

**Child Trait Extension:**
```rust
pub trait Child: Send + Sync {
    // Existing methods...
    
    /// Get spawned OS process IDs for cleanup
    /// Returns (PID, PGID) pairs or Job Object handles
    fn child_processes(&self) -> Vec<ChildProcessInfo> {
        Vec::new()  // Default: no OS processes
    }
}

pub enum ChildProcessInfo {
    #[cfg(unix)]
    ProcessGroup { pid: u32, pgid: u32 },
    
    #[cfg(windows)]
    JobObject { handle: JobObjectHandle },
}
```

**Supervisor Cleanup Integration:**
```rust
// In supervisor shutdown logic
async fn stop_child_with_cleanup(&mut self, child: &mut dyn Child) -> Result<()> {
    // 1. Stop child actor gracefully
    child.stop().await?;
    
    // 2. Cleanup any spawned OS processes
    let child_processes = child.child_processes();
    for process_info in child_processes {
        match process_info {
            #[cfg(unix)]
            ChildProcessInfo::ProcessGroup { pgid, .. } => {
                // Kill entire process group
                kill_process_group(pgid, Signal::SIGTERM).await?;
                sleep(Duration::from_secs(5)).await;
                kill_process_group(pgid, Signal::SIGKILL).await.ok(); // Force cleanup
            }
            
            #[cfg(windows)]
            ChildProcessInfo::JobObject { handle } => {
                // Close job object → all processes terminated
                drop(handle);
            }
        }
    }
    
    Ok(())
}
```

### Phase 3: Detached Process Support

**Use Case:** Long-running background processes that outlive actor
```rust
pub enum ProcessLifetime {
    /// Tied to actor lifetime - cleanup on actor stop
    Managed { pgid: u32 },
    
    /// Survives actor crash/stop - runs independently
    Detached,
}

// Actor decides process lifetime
impl MyActor {
    async fn spawn_background_service(&mut self) -> Result<()> {
        let operation = SpawnProcessOperation {
            program: "background_service".into(),
            process_group: ProcessGroupConfig::NewGroup,
            lifetime: ProcessLifetime::Detached,  // NEW
            ..Default::default()
        };
        
        let result = self.osl.execute(operation).await?;
        // Don't track PID - process is fully independent
        Ok(())
    }
}
```

---

## YAGNI Decision Rationale

### Why Defer Implementation?

**1. No Proven Use Case Yet**
- airssys-rt default behavior: In-memory actors
- No actor implementation currently spawns OS processes
- OSL integration pattern uses dedicated actors (not direct spawning)
- Speculative feature for unproven requirement

**2. Complexity vs. Benefit Analysis**

**Implementation Cost:**
- Process group management: 3-4 days
- Supervisor integration: 2-3 days  
- Cross-platform testing: 2-3 days
- Documentation: 1 day
- **Total: 8-11 days of work**

**Current Benefit:**
- Zero actors need this feature today
- OSL integration uses actor pattern (no direct process spawning)
- Can implement when first real use case appears

**3. Alternative Solution: OSL Integration Actors**
```rust
// Recommended pattern - no process groups needed
let fs_actor = FileSystemActor::new(osl_client);
let process_actor = ProcessActor::new(osl_client);

supervisor.start_child(fs_actor).await?;
supervisor.start_child(process_actor).await?;

// Application actors send messages to OSL actors
app_actor.send_to(process_actor, SpawnRequest { ... }).await?;

// Benefit: ProcessActor can manage process lifecycle internally
// No need for supervisor-level process group management
```

**4. Incremental Implementation Path**
When needed, can implement in stages:
- Stage 1: Basic process group creation (2 days)
- Stage 2: Supervisor cleanup hooks (2 days)
- Stage 3: Detached process support (1 day)
- Stage 4: Cross-platform testing (2 days)

---

## Design Constraints for Future Implementation

### Critical Requirements

**1. Cross-Platform Consistency**
- **Linux/macOS:** Use POSIX `setpgid()` + `killpg()`
- **Windows:** Use Job Objects with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`
- **API Abstraction:** Hide platform differences behind unified interface

**2. Zero-Cost Abstraction**
- Process group creation only when explicitly requested
- No overhead for actors that don't spawn OS processes
- Default `child_processes()` returns empty Vec (no allocation)

**3. Backward Compatibility**
- Existing `SpawnProcessOperation` continues to work
- New fields are `Option<T>` or have sensible defaults
- No breaking changes to Child trait (default implementations)

**4. Security Considerations**
- Process group membership impacts signal propagation
- Audit log all process group operations
- Security context must propagate to child processes
- Consider resource limits per process group

### Integration Points

**With airssys-osl:**
- Extend `SpawnProcessOperation` with process group config
- Add `KillProcessGroupOperation` for cleanup
- Track PGID in `SpawnProcessResult`
- Platform-specific executors handle details

**With airssys-rt supervisors:**
- Optional `child_processes()` trait method
- Supervisor cleanup calls method during shutdown
- Configurable timeout for graceful → forced termination
- Error handling for cleanup failures

**With monitoring system:**
- Track process group lifecycle events
- Alert on orphaned process groups
- Metrics: spawned vs. cleaned up process groups
- Health checks for process group integrity

---

## Alternative Patterns (Current Recommendation)

### Pattern 1: OSL Integration Actors (Recommended)

**Architecture:**
```
RootSupervisor
├── OSLSupervisor (manages OSL integration actors)
│   ├── FileSystemActor
│   ├── ProcessActor       ← Manages all OS process spawning
│   └── NetworkActor
└── ApplicationSupervisor (manages business logic actors)
    ├── WorkerActor1
    ├── WorkerActor2
    └── WorkerActor3
```

**Benefits:**
- **Centralized process management:** ProcessActor handles all spawning
- **Clean lifecycle:** ProcessActor can track and cleanup child processes internally
- **No supervisor changes:** Supervisors just manage actors
- **Testable:** ProcessActor can be mocked
- **Observable:** ProcessActor metrics show all process operations

**Example:**
```rust
// ProcessActor maintains internal process registry
pub struct ProcessActor {
    osl: Arc<OslClient>,
    processes: HashMap<String, ProcessHandle>,
}

impl ProcessActor {
    async fn handle_spawn(&mut self, req: SpawnRequest) -> Result<SpawnResponse> {
        let result = self.osl.spawn_process(req.operation).await?;
        
        // Track process internally
        self.processes.insert(
            req.process_id.clone(),
            ProcessHandle {
                pid: result.pid,
                spawned_at: Utc::now(),
            }
        );
        
        Ok(SpawnResponse { pid: result.pid })
    }
    
    // Cleanup on actor stop
    async fn stop(&mut self) -> Result<()> {
        // Kill all tracked processes
        for (id, handle) in &self.processes {
            self.osl.kill_process(handle.pid, Signal::SIGTERM).await?;
        }
        Ok(())
    }
}
```

### Pattern 2: Process Pool Pattern

**For high-frequency process spawning:**
```rust
pub struct ProcessPoolActor {
    max_processes: usize,
    active: HashMap<Pid, ProcessInfo>,
    queue: VecDeque<SpawnRequest>,
}

// Benefits:
// - Rate limiting built-in
// - Centralized cleanup
// - Process accounting
// - Resource management
```

---

## When to Revisit This Decision

### Trigger Conditions

Implement process group management when **any** of these occur:

1. **Real use case emerges:**
   - Actor needs to spawn long-running OS process trees
   - Process must survive across actor restarts
   - Need to kill entire process hierarchy on cleanup

2. **Zombie process issues reported:**
   - Users report accumulating zombie processes
   - Resource leaks traced to orphaned child processes
   - Cleanup failures in production

3. **Integration requirements:**
   - External system requires process group integration
   - Container orchestration needs process group control
   - Security requirements mandate process isolation

4. **Performance requirements:**
   - Need to apply resource limits to process groups (cgroups)
   - CPU/memory quotas per process tree
   - Process accounting for billing/monitoring

### Implementation Priority

**High Priority:**
- Production zombie process issues
- Security requirements for process isolation
- Container integration requirements

**Medium Priority:**
- Performance optimization needs
- Resource accounting requirements
- Developer productivity improvements

**Low Priority:**
- Speculative "nice to have" features
- No concrete use case identified
- Alternative patterns work well

---

## Documentation and Knowledge Preservation

### Related Documentation

**Knowledge Documents:**
- KNOWLEDGE-RT-017: OSL Integration Actors Pattern (recommended approach)
- KNOWLEDGE-RT-001: Zero-Cost Actor Architecture (performance constraints)

**Architecture Decision Records:**
- ADR-RT-007: Hierarchical Supervisor Architecture (OSL integration design)

**Task Files:**
- RT-TASK-009: OSL Integration (current focus - uses actor pattern)

### Future Development Checklist

When implementing process group management:

- [ ] Review this document for constraints and requirements
- [ ] Create ADR documenting decision to implement
- [ ] Design cross-platform API (POSIX + Windows)
- [ ] Implement Linux/macOS with `setpgid()` + `killpg()`
- [ ] Implement Windows with Job Objects
- [ ] Extend `SpawnProcessOperation` with process group config
- [ ] Add `KillProcessGroupOperation` for cleanup
- [ ] Extend Child trait with `child_processes()` method
- [ ] Integrate supervisor cleanup logic
- [ ] Add comprehensive tests (unit + integration)
- [ ] Add security audit logging
- [ ] Update documentation (rustdoc + mdBook)
- [ ] Performance testing and benchmarking
- [ ] Cross-platform validation (Linux, macOS, Windows)

---

## Key Takeaways

1. **Valid Concern:** Zombie processes and orphaned child processes are real risks
2. **YAGNI Decision:** Defer implementation until proven use case emerges
3. **Current Solution:** OSL integration actors pattern (no process groups needed)
4. **Future Path:** Clear implementation plan when needed (8-11 days estimated)
5. **No Blocker:** Deferring doesn't prevent RT-TASK-009 completion
6. **Best Practice:** Build only what's needed now, document for future

**Quote from Discussion:**
> "Maybe for further improvements. But right now we are building actor in-memory first. 
> No need for now, because it's too over-engineering."

---

**Last Updated:** 2025-10-11  
**Next Review:** When first process-spawning actor use case appears  
**Status:** ✅ Decision documented, ready for future implementation
