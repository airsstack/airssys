# KNOW-012: Process Groups for Process Management

**Category**: System Programming Pattern  
**Created**: 2025-10-08  
**Status**: Future Enhancement (Not Implemented)  
**Related**: OSL-TASK-008 Phase 2 (Process Executor), ProcessSpawnOperation

---

## Overview

Process groups are a Unix/POSIX mechanism for organizing related processes together, enabling collective management of process trees. This document captures design considerations for potential future implementation in the Process Executor.

## What Are Process Groups?

### Core Concepts

**Process Group:**
- Collection of one or more processes
- Identified by Process Group ID (PGID)
- Process group leader has PID == PGID
- Used for signal distribution and job control

**Key Capabilities:**
1. **Collective Signaling** - Send signals to entire process group at once
2. **Process Tree Management** - Manage parent + all child processes together
3. **Job Control** - Foreground/background process management in shells
4. **Session Management** - Organize processes into terminal sessions

### Why Process Groups Matter

**Problem:** When you spawn a process that creates child processes, killing the parent doesn't kill children:
```rust
// Spawn a shell script that launches multiple background processes
let parent = spawn("./script.sh").await?;
kill(parent_pid).await?; // Parent dies, children become orphaned zombies!
```

**Solution with Process Groups:** Signal entire process group to kill all related processes:
```rust
// With process group support
let parent = spawn("./script.sh").with_process_group(true).await?;
signal_process_group(-parent_pid, SIGTERM).await?; // Kills parent + all children
```

## Platform Differences

### Unix/Linux/macOS (POSIX)

**API:** `setpgid()` system call
```rust
#[cfg(unix)]
use std::os::unix::process::CommandExt;

cmd.process_group(0); // Create new process group, process becomes leader
```

**Process Group Operations:**
- `setpgid(0, 0)` - Create new group, current process is leader
- `setpgid(pid, pgid)` - Move process to existing group
- `kill(-pgid, signal)` - Signal entire process group (negative PID)

**Session Management:**
- `setsid()` - Create new session and process group
- Used for daemon processes and terminal management

### Windows

**No Direct Equivalent** - Different process model:

**Alternative: Job Objects**
```rust
#[cfg(windows)]
// Use CreateJobObject API
// Requires more complex implementation with Windows-specific APIs
use windows::Win32::System::JobObjects::*;

let job = CreateJobObjectW()?;
AssignProcessToJobObject(job, process_handle)?;
TerminateJobObject(job, exit_code)?; // Kills all processes in job
```

**Key Differences:**
- Jobs are kernel objects (handles), not simple integer IDs
- More features (resource limits, security, priority)
- Require explicit handle management
- No concept of "negative PID" signaling

## Design Options for Future Implementation

### Option 1: Implicit Process Group Creation (Simplest)

**Approach:** Always create new process group on spawn (Unix only)

```rust
#[cfg(unix)]
impl OSExecutor<ProcessSpawnOperation> for ProcessExecutor {
    async fn execute(&self, operation: ProcessSpawnOperation, ...) -> OSResult<ExecutionResult> {
        let mut cmd = tokio::process::Command::new(&operation.command);
        
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0); // Always create new process group
        }
        
        let child = cmd.spawn()?;
        // ...
    }
}
```

**Pros:**
- ✅ Simple - no API changes required
- ✅ Safer default - prevents orphaned child processes
- ✅ Enables process tree cleanup with kill(-pgid)

**Cons:**
- ❌ Breaking change - alters existing behavior
- ❌ No flexibility - can't opt out
- ❌ May conflict with shell job control expectations

**Verdict:** ❌ **Not Recommended** - Too opinionated for a general-purpose API

---

### Option 2: Optional Field in ProcessSpawnOperation (Recommended)

**Approach:** Add optional process group configuration to spawn operation

```rust
// In operations/process/spawn.rs
pub struct ProcessSpawnOperation {
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: Option<String>,
    pub process_group: Option<ProcessGroupConfig>, // NEW FIELD
    // ... existing fields
}

#[derive(Debug, Clone)]
pub enum ProcessGroupConfig {
    /// Create new process group with spawned process as leader (Unix: setpgid(0,0))
    NewGroup,
    
    /// Join existing process group by PGID (Unix: setpgid(pid, pgid))
    JoinGroup(u32),
    
    /// Use parent's process group (default behavior)
    Inherit,
}

impl ProcessSpawnOperation {
    /// Create new process group for spawned process and its children
    pub fn with_new_process_group(mut self) -> Self {
        self.process_group = Some(ProcessGroupConfig::NewGroup);
        self
    }
    
    /// Join existing process group
    pub fn with_process_group(mut self, pgid: u32) -> Self {
        self.process_group = Some(ProcessGroupConfig::JoinGroup(pgid));
        self
    }
}
```

**Executor Implementation:**
```rust
// In executors/process/spawn.rs
#[cfg(unix)]
if let Some(pg_config) = &operation.process_group {
    use std::os::unix::process::CommandExt;
    
    match pg_config {
        ProcessGroupConfig::NewGroup => {
            cmd.process_group(0); // New group, process is leader
        }
        ProcessGroupConfig::JoinGroup(pgid) => {
            cmd.process_group(*pgid as i32); // Join existing group
        }
        ProcessGroupConfig::Inherit => {
            // No action - default behavior
        }
    }
}

#[cfg(windows)]
if operation.process_group.is_some() {
    // Windows: Would need Job Objects implementation
    // For now, return error or log warning
    return Err(OSError::execution_failed(
        "Process groups not supported on Windows. Use Job Objects API instead."
    ));
}
```

**Usage Example:**
```rust
// Spawn process tree that can be killed as group
let operation = ProcessSpawnOperation::new("./run-services.sh")
    .with_new_process_group(); // Create new process group

let result = executor.execute(operation, &context).await?;
let pid = result.get_metadata("pid").unwrap().parse::<u32>()?;

// Later: kill entire process group
let kill_group = ProcessSignalOperation::new_for_group(pid, 15); // SIGTERM to group
executor.execute(kill_group, &context).await?;
```

**Pros:**
- ✅ Explicit opt-in - backward compatible
- ✅ Flexible - supports multiple use cases
- ✅ Clean API - follows builder pattern
- ✅ Platform-agnostic design - can add Windows Job Objects later

**Cons:**
- ⚠️ Requires additional field in operation struct
- ⚠️ Platform-specific implementation complexity
- ⚠️ Need separate operation for signaling groups (or extend ProcessSignalOperation)

**Verdict:** ✅ **RECOMMENDED** - Best balance of flexibility and clean design

---

### Option 3: Separate ProcessGroupOperation

**Approach:** New operation type for post-spawn process group management

```rust
// New operation type
pub struct ProcessGroupOperation {
    pub pid: u32,
    pub action: ProcessGroupAction,
}

pub enum ProcessGroupAction {
    /// Make process a group leader (setpgid(pid, 0))
    CreateGroup,
    
    /// Move process to existing group (setpgid(pid, pgid))
    JoinGroup(u32),
    
    /// Get process group ID
    GetGroupId,
    
    /// Signal entire process group
    SignalGroup(i32),
}
```

**Pros:**
- ✅ Separation of concerns - spawning vs group management
- ✅ Can modify groups after process creation
- ✅ More granular control

**Cons:**
- ❌ Complex - requires multiple operations for common workflow
- ❌ Can't set group at spawn time (Unix limitation: must set before exec)
- ❌ Higher API surface area

**Verdict:** ❌ **Not Recommended** - Over-engineered for typical use cases

---

## Implementation Considerations

### Signal Handling for Process Groups

**Extend ProcessSignalOperation** to support process groups:

```rust
pub struct ProcessSignalOperation {
    pub pid: u32,
    pub signal: i32,
    pub target_group: bool, // NEW: If true, signal entire process group
    // ... existing fields
}

impl ProcessSignalOperation {
    /// Signal entire process group
    pub fn signal_group(pgid: u32, signal: i32) -> Self {
        Self {
            pid: pgid,
            signal,
            target_group: true,
            // ...
        }
    }
}
```

**Executor Implementation:**
```rust
#[cfg(unix)]
{
    use nix::sys::signal::Signal;
    use nix::unistd::Pid;
    
    let pid = if operation.target_group {
        Pid::from_raw(-(operation.pid as i32)) // Negative PID = process group
    } else {
        Pid::from_raw(operation.pid as i32)
    };
    
    let signal = Signal::try_from(operation.signal)?;
    nix::sys::signal::kill(pid, signal)?;
}
```

### Metadata Tracking

**Capture process group information** in ExecutionResult:

```rust
// In spawn executor
let result = ExecutionResult::success_with_timing(output, started_at, completed_at)
    .with_metadata("pid", pid.to_string())
    .with_metadata("pgid", pgid.to_string()) // NEW: Process group ID
    .with_metadata("is_group_leader", is_leader.to_string()) // NEW
    // ... existing metadata
```

### Permission Considerations

**Process group operations may require elevated privileges:**

```rust
// Check if operation requires elevation
impl ProcessSpawnOperation {
    fn requires_elevation(&self) -> bool {
        // Creating new process group might require privileges
        if let Some(ProcessGroupConfig::NewGroup) = self.process_group {
            return true;
        }
        // ... other checks
    }
}
```

## Use Cases and Examples

### Use Case 1: Shell Script with Child Processes

**Problem:** Shell script launches multiple background processes. Need to kill all when done.

```rust
// Spawn script with new process group
let operation = ProcessSpawnOperation::new("./deploy-services.sh")
    .with_new_process_group();

let result = framework.spawn(operation).await?;
let pgid = result.get_metadata("pgid").unwrap().parse::<u32>()?;

// Later: graceful shutdown of entire service tree
framework.signal_process_group(pgid, 15).await?; // SIGTERM to all
tokio::time::sleep(Duration::from_secs(5)).await;
framework.signal_process_group(pgid, 9).await?; // SIGKILL remaining
```

### Use Case 2: Daemon Process Isolation

**Problem:** Create daemon that shouldn't receive terminal signals.

```rust
// Create new session + process group (Unix setsid equivalent)
let operation = ProcessSpawnOperation::new("./daemon")
    .with_new_process_group()
    .with_detach_from_terminal(); // Additional field for setsid()

framework.spawn(operation).await?;
```

### Use Case 3: Container-like Process Trees

**Problem:** Manage isolated process trees for sandboxing.

```rust
// Create isolated process tree
let supervisor = ProcessSpawnOperation::new("supervisor")
    .with_new_process_group();

let result = framework.spawn(supervisor).await?;
let pgid = result.get_metadata("pgid").unwrap().parse::<u32>()?;

// All child processes inherit the group automatically
// Can monitor and control entire tree as unit
```

## Testing Strategy

### Unit Tests for Process Group Creation

```rust
#[cfg(unix)]
#[tokio::test]
async fn test_spawn_with_new_process_group() {
    let executor = ProcessExecutor::new("test-executor");
    let operation = ProcessSpawnOperation::new("sleep")
        .arg("10")
        .with_new_process_group();
    
    let result = executor.execute(operation, &context).await?;
    let pid = result.get_metadata("pid").unwrap().parse::<u32>()?;
    let pgid = result.get_metadata("pgid").unwrap().parse::<u32>()?;
    
    // Process should be its own group leader
    assert_eq!(pid, pgid);
}
```

### Integration Tests for Process Group Signaling

```rust
#[cfg(unix)]
#[tokio::test]
async fn test_signal_entire_process_group() {
    // Spawn parent that creates children
    let operation = ProcessSpawnOperation::new("./spawn-children.sh")
        .with_new_process_group();
    
    let result = framework.spawn(operation).await?;
    let pgid = result.get_metadata("pgid").unwrap().parse::<u32>()?;
    
    // Wait for children to spawn
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Signal entire group
    framework.signal_process_group(pgid, 15).await?;
    
    // Verify all processes terminated
    // ... verification logic
}
```

## Performance Impact

**Minimal overhead:**
- Process group creation is single `setpgid()` syscall
- No runtime overhead after creation
- Signal delivery to group is same cost as single process

**Memory impact:**
- ProcessGroupConfig enum: 8 bytes (Option<enum>)
- Negligible metadata overhead

## Security Considerations

### Privilege Escalation Risk

**Process group operations can bypass security boundaries:**
- Moving process to another group may require privileges
- Signaling groups can affect unrelated processes if PGID collision
- Need careful validation of PGID values

**Mitigation:**
```rust
// Validate PGID exists and caller has permission
async fn validate_operation(&self, operation: &ProcessSignalOperation, ...) -> OSResult<()> {
    if operation.target_group {
        // Check if PGID exists
        // Check if caller owns processes in group
        // Prevent signaling privileged process groups
    }
}
```

### Audit Logging

**Process group operations should be logged:**
```rust
// In metadata
.with_metadata("operation_type", "process_group_signal")
.with_metadata("target_pgid", pgid.to_string())
.with_metadata("processes_affected", count.to_string())
```

## Migration Path

### Phase 1: Foundation (Future - When Needed)
1. Add `ProcessGroupConfig` enum to shared types
2. Add optional `process_group` field to `ProcessSpawnOperation`
3. Implement Unix support in spawn executor
4. Add comprehensive unit tests

### Phase 2: Signal Integration (Future)
1. Add `target_group` field to `ProcessSignalOperation`
2. Implement group signaling in signal executor
3. Add integration tests for group lifecycle

### Phase 3: Windows Support (Future)
1. Research Job Objects API
2. Design Windows-specific abstraction
3. Implement parallel Job Objects support
4. Cross-platform integration tests

### Phase 4: Advanced Features (Far Future)
1. Session management (`setsid()`)
2. Terminal control integration
3. Resource limits per group (cgroups on Linux)

## References

### Documentation
- **POSIX setpgid**: https://pubs.opengroup.org/onlinepubs/9699919799/functions/setpgid.html
- **Unix Process Groups**: https://www.gnu.org/software/libc/manual/html_node/Process-Group-Functions.html
- **Windows Job Objects**: https://docs.microsoft.com/en-us/windows/win32/procthread/job-objects

### Related Patterns
- **KNOW-004**: Builder-to-Operation Bridge Pattern
- **ADR-027**: Builder Pattern Architecture (Multi-level builders)

### Related Code
- `operations/process/spawn.rs` - ProcessSpawnOperation definition
- `operations/process/signal.rs` - ProcessSignalOperation definition
- `executors/process/spawn.rs` - Spawn executor implementation
- `executors/process/signal.rs` - Signal executor implementation

## Decision

**Status:** ⏳ **Deferred** - Not implemented in OSL-TASK-008 Phase 2

**Reasoning:**
- ✅ Follows YAGNI principle (§6.1) - No current use case requiring it
- ✅ Phase 2 focus is basic process management (spawn, kill, signal)
- ✅ Process groups are advanced feature for specific scenarios
- ✅ Can be added later without breaking existing API

**When to Implement:**
- Real use case emerges requiring process tree management
- User requests ability to kill process hierarchies
- Container or sandbox implementation needs process isolation
- Daemon or service management requires session control

**Recommended Approach When Implementing:**
- Use **Option 2** (Optional field in ProcessSpawnOperation)
- Extend ProcessSignalOperation for group signaling
- Start with Unix support, add Windows Job Objects later
- Comprehensive testing for process lifecycle edge cases

---

**Document Status**: Knowledge Base Entry (Not a Task)  
**Next Review**: When process tree management requirements emerge  
**Owner**: Core OSL Team
