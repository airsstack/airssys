# WASM-TASK-005 Phase 2 Task 2.2: Approval Workflow Engine - IMPLEMENTATION PLAN

**Task:** Approval Workflow Engine  
**Status:** ✅ COMPLETE  
**Date Created:** 2025-12-17  
**Date Completed:** 2025-12-17  
**Estimated Duration:** 3-5 days (25 hours)  
**Actual Duration:** ~4 hours  
**Prerequisites:** ✅ Phase 1 complete (Tasks 1.1-1.3), ✅ Task 2.1 (Trust Level) COMPLETE (commit e1a5382)

**Completion Report:** See `task-005-phase-2-task-2.2-completion.md`

---

## Executive Summary

**What**: Implement an approval workflow state machine that routes components through different installation workflows based on their trust level: **Trusted** sources install instantly, **Unknown** sources enter a review queue for manual approval, and **DevMode** bypasses security with logged warnings.

**Why**: Components from unknown sources pose security risks and must be reviewed before granting host access. The approval workflow provides security oversight while maintaining developer productivity for trusted sources. The system must persist approval decisions to avoid re-prompting and provide a clear audit trail.

**How**: Create an approval state machine (Pending → Reviewing → Approved/Denied), a review queue for unknown components with capability preview, persistent approval storage, and integration with the trust level system (Task 2.1). The workflow engine coordinates between trust determination, capability analysis, and component installation.

**Architecture Position**: This workflow engine sits between trust determination (Task 2.1) and component instantiation, orchestrating the security review process.

---

## Implementation Strategy

### Core Design Principles

1. **Non-Blocking for Trusted**: Trusted sources bypass approval (instant install)
2. **Secure by Default**: Unknown sources enter review queue (deny-by-default)
3. **Persistent Decisions**: Store approvals to avoid re-prompting
4. **Audit Trail**: Log all approval decisions and state transitions
5. **Fail-Safe**: On error, deny installation (fail-closed)

### Approval Workflow State Machine

```text
Component Installation Request
          ↓
    TrustLevel Check (Task 2.1)
          ↓
     ┌────┴────┐
     │Trust Lvl │
     └────┬────┘
          │
    ┌─────┼─────┐
    │     │     │
  Trusted │  Unknown     DevMode
    │     │     │           │
    ↓     ↓     ↓           ↓
Auto-   Review  Bypass      │
Approve Queue   Security    │
    │     │     │           │
    │     ↓     │           │
    │  Pending  │           │
    │     │     │           │
    │     ↓     │           │
    │ Reviewing │           │
    │  (Admin)  │           │
    │     │     │           │
    │  ┌──┴──┐  │           │
    │  │Vote │  │           │
    │  └──┬──┘  │           │
    │     │     │           │
    │  ┌──┴────┐│           │
    │Approved Denied        │
    │  │       │            │
    └──┼───────┼────────────┘
       │       │
       ↓       ↓
   Install   Reject
```

---

## Data Structure Specifications

### 1. ApprovalState Enum

```rust
/// Approval workflow state for component installation.
/// 
/// State transitions:
/// - Trusted → AutoApproved (instant)
/// - Unknown → Pending → Reviewing → Approved/Denied
/// - DevMode → Bypassed (instant with warnings)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalState {
    /// Waiting in review queue (not yet reviewed).
    Pending {
        /// When component entered queue
        queued_at: DateTime<Utc>,
        
        /// Component metadata
        component_id: String,
        source: ComponentSource,
        
        /// Requested capabilities
        capabilities: WasmCapabilitySet,
    },
    
    /// Currently under review by administrator.
    Reviewing {
        /// When review started
        started_at: DateTime<Utc>,
        
        /// Reviewer identity (user ID, email, etc.)
        reviewer: String,
        
        /// Original request
        component_id: String,
        source: ComponentSource,
        capabilities: WasmCapabilitySet,
    },
    
    /// Approved for installation (manual or auto).
    Approved {
        /// When approval granted
        approved_at: DateTime<Utc>,
        
        /// Approver identity (or "auto" for trusted sources)
        approver: String,
        
        /// Approved capabilities (may be modified from original)
        approved_capabilities: WasmCapabilitySet,
        
        /// Approval reason/notes
        reason: Option<String>,
    },
    
    /// Denied installation.
    Denied {
        /// When denial issued
        denied_at: DateTime<Utc>,
        
        /// Denier identity
        denier: String,
        
        /// Denial reason
        reason: String,
    },
    
    /// Auto-approved (trusted source, no review needed).
    AutoApproved {
        /// When auto-approval occurred
        approved_at: DateTime<Utc>,
        
        /// Trust level that triggered auto-approval
        trust_level: TrustLevel,
        
        /// Approved capabilities (as declared)
        approved_capabilities: WasmCapabilitySet,
    },
    
    /// Bypassed (DevMode, security disabled).
    Bypassed {
        /// When bypass occurred
        bypassed_at: DateTime<Utc>,
        
        /// Warning message
        warning: String,
    },
}

impl ApprovalState {
    /// Returns true if installation can proceed.
    pub fn can_install(&self) -> bool {
        matches!(
            self,
            ApprovalState::Approved { .. }
                | ApprovalState::AutoApproved { .. }
                | ApprovalState::Bypassed { .. }
        )
    }
    
    /// Returns state name for logging.
    pub fn state_name(&self) -> &'static str;
    
    /// Returns timestamp of current state.
    pub fn timestamp(&self) -> DateTime<Utc>;
}
```

### 2. ApprovalRequest (Queue Entry)

```rust
/// Approval request entry in review queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Unique request ID
    pub request_id: Uuid,
    
    /// Component identifier
    pub component_id: String,
    
    /// Component source
    pub source: ComponentSource,
    
    /// Requested capabilities
    pub capabilities: WasmCapabilitySet,
    
    /// Current approval state
    pub state: ApprovalState,
    
    /// Trust level (from Task 2.1)
    pub trust_level: TrustLevel,
    
    /// State history (for audit trail)
    pub history: Vec<StateTransition>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl ApprovalRequest {
    /// Creates new request in Pending state.
    pub fn new(
        component_id: String,
        source: ComponentSource,
        capabilities: WasmCapabilitySet,
        trust_level: TrustLevel,
    ) -> Self;
    
    /// Transitions to next state.
    pub fn transition_to(&mut self, new_state: ApprovalState) -> Result<(), WorkflowError>;
    
    /// Checks if transition is valid.
    fn is_valid_transition(&self, new_state: &ApprovalState) -> bool;
}
```

### 3. StateTransition (Audit Log Entry)

```rust
/// State transition record for audit trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// Timestamp of transition
    pub timestamp: DateTime<Utc>,
    
    /// Previous state
    pub from_state: String,
    
    /// New state
    pub to_state: String,
    
    /// Actor who triggered transition (user ID, "system", etc.)
    pub actor: String,
    
    /// Transition reason/notes
    pub reason: Option<String>,
}
```

### 4. ReviewQueue (In-Memory Queue)

```rust
/// Review queue managing pending approval requests.
/// 
/// # Thread Safety
/// - Uses Arc<Mutex<>> for concurrent access
/// - Supports multiple reviewers
/// - Persistent storage via ApprovalStore
pub struct ReviewQueue {
    /// Pending requests (HashMap for O(1) lookup by component_id)
    pending: Arc<Mutex<HashMap<String, ApprovalRequest>>>,
    
    /// Approval persistence store
    store: Arc<ApprovalStore>,
    
    /// Audit logger
    audit_logger: Arc<SecurityAuditLogger>,
}

impl ReviewQueue {
    /// Creates new review queue with persistent store.
    pub fn new(store: Arc<ApprovalStore>, audit_logger: Arc<SecurityAuditLogger>) -> Self;
    
    /// Adds request to queue.
    pub fn enqueue(&self, request: ApprovalRequest) -> Result<(), WorkflowError>;
    
    /// Retrieves request by component_id.
    pub fn get_request(&self, component_id: &str) -> Option<ApprovalRequest>;
    
    /// Lists all pending requests.
    pub fn list_pending(&self) -> Vec<ApprovalRequest>;
    
    /// Starts review for request.
    pub fn start_review(
        &self,
        component_id: &str,
        reviewer: String,
    ) -> Result<ApprovalRequest, WorkflowError>;
    
    /// Approves request (optionally modify capabilities).
    pub fn approve(
        &self,
        component_id: &str,
        approver: String,
        approved_capabilities: Option<WasmCapabilitySet>,
        reason: Option<String>,
    ) -> Result<(), WorkflowError>;
    
    /// Denies request.
    pub fn deny(
        &self,
        component_id: &str,
        denier: String,
        reason: String,
    ) -> Result<(), WorkflowError>;
    
    /// Removes request from queue.
    pub fn dequeue(&self, component_id: &str) -> Result<ApprovalRequest, WorkflowError>;
}
```

### 5. ApprovalStore (Persistent Storage)

```rust
/// Persistent approval decision storage.
/// 
/// # Storage Format
/// - JSON file per approval decision
/// - Directory structure: `approvals/<component_id>/<request_id>.json`
/// - Enables decision lookup and audit trail
pub struct ApprovalStore {
    /// Storage directory path
    storage_path: PathBuf,
}

impl ApprovalStore {
    /// Creates store with specified storage directory.
    pub fn new(storage_path: PathBuf) -> Result<Self, WorkflowError>;
    
    /// Saves approval decision.
    pub fn save(&self, request: &ApprovalRequest) -> Result<(), WorkflowError>;
    
    /// Loads approval decision by component_id.
    pub fn load(&self, component_id: &str) -> Result<Option<ApprovalRequest>, WorkflowError>;
    
    /// Lists all stored approvals.
    pub fn list_all(&self) -> Result<Vec<ApprovalRequest>, WorkflowError>;
    
    /// Deletes approval decision.
    pub fn delete(&self, component_id: &str) -> Result<(), WorkflowError>;
    
    /// Checks if component has prior approval.
    pub fn has_approval(&self, component_id: &str) -> bool;
}
```

### 6. ApprovalWorkflow (Main Orchestrator)

```rust
/// Approval workflow orchestrator coordinating all components.
/// 
/// # Responsibilities
/// - Route components based on trust level
/// - Manage review queue
/// - Persist approval decisions
/// - Audit all workflow actions
pub struct ApprovalWorkflow {
    /// Trust registry (from Task 2.1)
    trust_registry: Arc<TrustRegistry>,
    
    /// Review queue
    review_queue: Arc<ReviewQueue>,
    
    /// Approval store
    approval_store: Arc<ApprovalStore>,
    
    /// Audit logger
    audit_logger: Arc<SecurityAuditLogger>,
}

impl ApprovalWorkflow {
    /// Creates new workflow with dependencies.
    pub fn new(
        trust_registry: Arc<TrustRegistry>,
        approval_store: Arc<ApprovalStore>,
        audit_logger: Arc<SecurityAuditLogger>,
    ) -> Self;
    
    /// Main entry point: Process component installation request.
    pub async fn process_installation_request(
        &self,
        component_id: String,
        source: ComponentSource,
        capabilities: WasmCapabilitySet,
    ) -> Result<ApprovalDecision, WorkflowError>;
    
    /// Auto-approve workflow (trusted sources).
    async fn auto_approve_workflow(
        &self,
        component_id: String,
        capabilities: WasmCapabilitySet,
        trust_level: TrustLevel,
    ) -> Result<ApprovalDecision, WorkflowError>;
    
    /// Review workflow (unknown sources).
    async fn review_workflow(
        &self,
        component_id: String,
        source: ComponentSource,
        capabilities: WasmCapabilitySet,
    ) -> Result<ApprovalDecision, WorkflowError>;
    
    /// Bypass workflow (DevMode).
    async fn bypass_workflow(
        &self,
        component_id: String,
    ) -> Result<ApprovalDecision, WorkflowError>;
}
```

### 7. ApprovalDecision (Workflow Result)

```rust
/// Approval workflow decision result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalDecision {
    /// Installation approved - proceed.
    Approved {
        /// Approved capabilities
        capabilities: WasmCapabilitySet,
        
        /// Approval metadata
        approval: ApprovalState,
    },
    
    /// Installation pending - waiting in review queue.
    Pending {
        /// Request ID for tracking
        request_id: Uuid,
        
        /// Queue position
        queue_position: usize,
    },
    
    /// Installation denied - reject.
    Denied {
        /// Denial reason
        reason: String,
        
        /// Denial metadata
        denial: ApprovalState,
    },
}

impl ApprovalDecision {
    /// Returns true if installation can proceed.
    pub fn can_proceed(&self) -> bool;
}
```

---

## Implementation Steps (20 Steps, ~25 hours)

### Step 1: Create Approval Module Structure (30 min)
- Create `airssys-wasm/src/security/approval.rs`
- Add module declaration to `security/mod.rs`
- Add 3-layer imports (§2.1)
- Define module-level rustdoc
- **Checkpoint**: `cargo check` passes

### Step 2: Implement ApprovalState Enum (1 hour)
- `ApprovalState` enum with 6 variants
- Helper methods (`can_install()`, `state_name()`, `timestamp()`)
- Derive traits (Debug, Clone, PartialEq, Eq, Serialize, Deserialize)
- 8 unit tests (one per state + transitions)
- **Checkpoint**: ApprovalState tests pass

### Step 3: Implement StateTransition (30 min)
- `StateTransition` struct
- Audit log entry format
- 2 unit tests
- **Checkpoint**: StateTransition serialization works

### Step 4: Implement ApprovalRequest (1.5 hours)
- `ApprovalRequest` struct with state machine
- `new()`, `transition_to()`, `is_valid_transition()`
- State transition validation
- 10 unit tests (valid/invalid transitions)
- **Checkpoint**: State machine tests pass

### Step 5: Implement ApprovalStore (2.5 hours)
- `ApprovalStore` struct with filesystem backend
- JSON serialization/deserialization
- Directory structure creation
- `save()`, `load()`, `delete()`, `has_approval()`
- 10 unit tests (save, load, delete, errors)
- **Checkpoint**: Persistence tests pass

### Step 6: Implement ReviewQueue Core (2 hours)
- `ReviewQueue` struct with Arc<Mutex<>>
- `enqueue()`, `dequeue()`, `get_request()`
- Thread safety
- 8 unit tests (enqueue, dequeue, concurrency)
- **Checkpoint**: Queue operations work

### Step 7: Implement Review Operations (2 hours)
- `list_pending()` method
- `start_review()` method
- `approve()` method
- `deny()` method
- 8 unit tests (one per operation)
- **Checkpoint**: Review operations tests pass

### Step 8: Implement Auto-Approve Workflow (2 hours)
- `auto_approve_workflow()` method
- Integration with Task 2.1 (TrustLevel::Trusted)
- Instant approval logic
- Audit logging
- 5 unit tests
- **Checkpoint**: Auto-approve tests pass

### Step 9: Implement Review Workflow (2.5 hours)
- `review_workflow()` method
- Queue entry creation
- Pending state handling
- 5 unit tests
- **Checkpoint**: Review workflow tests pass

### Step 10: Implement Bypass Workflow (1.5 hours)
- `bypass_workflow()` method
- DevMode handling
- Prominent warnings
- Audit logging
- 5 unit tests
- **Checkpoint**: Bypass workflow tests pass

### Step 11: Implement ApprovalWorkflow Orchestrator (2 hours)
- `ApprovalWorkflow` struct
- `process_installation_request()` main entry point
- Trust level routing
- 8 unit tests
- **Checkpoint**: Workflow orchestration works

### Step 12: Implement Approval Decision Types (1 hour)
- `ApprovalDecision` enum
- Helper methods
- 3 unit tests
- **Checkpoint**: Decision types work

### Step 13: Implement Prior Approval Check (1 hour)
- Check `ApprovalStore` for existing approvals
- Skip review if previously approved
- 5 unit tests
- **Checkpoint**: Prior approval tests pass

### Step 14: Implement Concurrent Review Handling (1.5 hours)
- Multiple reviewers support
- Reviewer assignment
- Conflict resolution
- 5 unit tests
- **Checkpoint**: Concurrency tests pass

### Step 15: Implement Audit Logging (1 hour)
- Integrate airssys-osl SecurityAuditLogger
- Log all state transitions
- Log all approval decisions
- 5 integration tests
- **Checkpoint**: Audit logs captured

### Step 16: Comprehensive Integration Tests (2 hours)
- End-to-end workflow tests (trusted, unknown, DevMode)
- Error handling tests
- Edge case tests (empty queue, concurrent operations)
- 15+ integration tests
- **Checkpoint**: All integration tests pass

### Step 17: Approval Module Documentation (2 hours)
- Module-level rustdoc with workflow diagrams
- Function rustdoc for all public APIs
- State machine documentation
- Examples for each workflow
- **Checkpoint**: Zero rustdoc warnings

### Step 18: Examples (1.5 hours)
- `examples/security_approval_trusted.rs`
- `examples/security_approval_review.rs`
- `examples/security_approval_devmode.rs`
- **Checkpoint**: All examples run

### Step 19: CLI Integration Stubs (1 hour)
- Review queue CLI commands (for Task 2.3)
- Approval/denial CLI commands
- List pending CLI command
- **Checkpoint**: CLI stubs compile

### Step 20: Final Quality Gates (30 min)
- `cargo clippy --all-targets` (zero warnings)
- `cargo test --all-targets` (all pass)
- `cargo doc --no-deps` (zero warnings)
- **Checkpoint**: All quality gates pass

---

## Test Plan (50+ Test Scenarios)

### Positive Tests (15 tests)

| Test ID | Scenario | Expected Output |
|---------|----------|-----------------|
| P01 | Trusted source auto-approval | ApprovalDecision::Approved |
| P02 | Unknown source enters queue | ApprovalDecision::Pending |
| P03 | DevMode bypass | ApprovalDecision::Approved (bypassed) |
| P04 | Admin approves request | State: Approved |
| P05 | Admin denies request | State: Denied |
| P06 | Prior approval found | Skip review, return Approved |
| P07 | Capability modification | Approved with modified capabilities |
| P08 | Multiple pending requests | List all correctly |
| P09 | Start review | State: Reviewing |
| P10 | Enqueue request | Request ID returned |
| P11 | Dequeue request | Request removed |
| P12 | Save approval to store | File created |
| P13 | Load approval from store | Approval loaded |
| P14 | State transition audit | All transitions logged |
| P15 | Workflow orchestration | Correct routing by trust level |

### Negative Tests (15 tests)

| Test ID | Scenario | Expected Output |
|---------|----------|----------------|
| N01 | Invalid state transition | ValidationError |
| N02 | Approve non-existent request | NotFoundError |
| N03 | Deny non-existent request | NotFoundError |
| N04 | Duplicate enqueue | AlreadyExistsError |
| N05 | Dequeue empty queue | EmptyQueueError |
| N06 | Load non-existent approval | None |
| N07 | Save to read-only directory | IoError |
| N08 | Malformed approval JSON | ParseError |
| N09 | Approve already approved | InvalidStateError |
| N10 | Deny already denied | InvalidStateError |
| N11 | Review already reviewing | ConflictError |
| N12 | Transition from terminal state | InvalidTransitionError |
| N13 | Empty component_id | ValidationError |
| N14 | Null capabilities | ValidationError |
| N15 | Corrupt approval store file | ParseError |

### Edge Case Tests (20 tests)

| Test ID | Scenario | Expected Behavior |
|---------|----------|-------------------|
| E01 | Very long capability list (1000+) | Process correctly |
| E02 | Concurrent enqueue operations | All succeed |
| E03 | Concurrent approve operations | First wins |
| E04 | Prior approval + new request | Use prior approval |
| E05 | DevMode + unknown source | DevMode takes precedence |
| E06 | Empty review queue | List returns empty |
| E07 | Queue position calculation | Correct position |
| E08 | State history length (100+ entries) | All recorded |
| E09 | Approval store with 10k files | List all correctly |
| E10 | Rapid state transitions | All recorded |
| E11 | Reviewer identity validation | Validate correctly |
| E12 | Approval reason length (10k chars) | Store correctly |
| E13 | UTF-8 in approval reason | Handle correctly |
| E14 | Filesystem race conditions | Handle gracefully |
| E15 | Approval store recovery | Rebuild from files |
| E16 | Partial approval (modify caps) | Apply correctly |
| E17 | Zero capabilities request | Handle correctly |
| E18 | Timestamp ordering | Chronological order |
| E19 | Multiple reviewers same request | Conflict resolution |
| E20 | Orphaned review queue entries | Cleanup correctly |

---

## Workflow Examples

### Example 1: Trusted Source (Auto-Approve)

```rust
// Component from trusted Git repository
let source = ComponentSource::Git {
    url: "https://github.com/mycompany/data-processor".to_string(),
    branch: "main".to_string(),
    commit: "abc123".to_string(),
};

let capabilities = WasmCapabilitySet::new()
    .grant(WasmCapability::Filesystem {
        paths: vec!["/app/data/*".to_string()],
        permissions: vec!["read".to_string()],
    });

// Process installation
let decision = workflow.process_installation_request(
    "data-processor".to_string(),
    source,
    capabilities,
).await?;

// Result: ApprovalDecision::Approved (instant)
match decision {
    ApprovalDecision::Approved { capabilities, approval } => {
        println!("✅ Auto-approved (trusted source)");
        // Proceed with installation
    }
    _ => unreachable!(),
}
```

### Example 2: Unknown Source (Review Queue)

```rust
// Component from unknown source
let source = ComponentSource::Git {
    url: "https://github.com/external/unknown-tool".to_string(),
    branch: "main".to_string(),
    commit: "xyz789".to_string(),
};

let capabilities = WasmCapabilitySet::new()
    .grant(WasmCapability::Filesystem {
        paths: vec!["/var/data/**".to_string()],
        permissions: vec!["write".to_string()],
    });

// Process installation
let decision = workflow.process_installation_request(
    "unknown-tool".to_string(),
    source,
    capabilities,
).await?;

// Result: ApprovalDecision::Pending
match decision {
    ApprovalDecision::Pending { request_id, queue_position } => {
        println!("⏳ Pending review (queue position: {})", queue_position);
        println!("   Request ID: {}", request_id);
        // Wait for admin approval
    }
    _ => unreachable!(),
}

// Admin reviews and approves
workflow.review_queue.start_review(
    "unknown-tool",
    "admin@example.com".to_string(),
)?;

workflow.review_queue.approve(
    "unknown-tool",
    "admin@example.com".to_string(),
    None, // Accept original capabilities
    Some("Verified source code, looks safe".to_string()),
)?;
```

### Example 3: DevMode (Bypass)

```rust
// Enable DevMode
trust_registry.set_dev_mode(true);

let source = ComponentSource::Local {
    path: PathBuf::from("./my-local-component"),
};

let capabilities = WasmCapabilitySet::new(); // Any capabilities

// Process installation
let decision = workflow.process_installation_request(
    "my-local-component".to_string(),
    source,
    capabilities,
).await?;

// Result: ApprovalDecision::Approved (bypassed)
// Console output: ⚠️  ⚠️  ⚠️  DEVELOPMENT MODE ACTIVE ⚠️  ⚠️  ⚠️
```

---

## Performance Targets

### Workflow Performance
- **Trusted Auto-Approve**: <1ms (instant)
- **Unknown Enqueue**: <5ms (queue insertion + persistence)
- **Review Approval**: <10ms (state update + persistence)
- **Prior Approval Check**: <100μs (filesystem lookup)
- **Queue List**: <50ms for 1000 entries

### Optimization Strategies
1. **In-Memory Queue**: HashMap for O(1) lookup
2. **Lazy Persistence**: Persist only on state change
3. **Cached Prior Approvals**: LRU cache for repeated checks
4. **Async I/O**: Non-blocking file operations

---

## Integration Points

### Task 2.1 Integration (Trust Level)

```rust
// Task 2.1 provides trust level determination
let trust_level = trust_registry.determine_trust_level(&component_id, &source);

// Task 2.2 uses trust level to route workflow
match trust_level {
    TrustLevel::Trusted => auto_approve_workflow(...),
    TrustLevel::Unknown => review_workflow(...),
    TrustLevel::DevMode => bypass_workflow(...),
}
```

### Task 2.3 Integration (CLI)

```bash
# Task 2.3 provides CLI for workflow management

# List pending reviews
$ airssys-wasm approval list
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Pending Approvals (3)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
1. unknown-tool (1 hour ago)
   Source: https://github.com/external/unknown-tool
   Capabilities:
     - filesystem.write: /var/data/**
   
2. data-processor (30 minutes ago)
   Source: https://github.com/unverified/data-processor
   Capabilities:
     - filesystem.read: /etc/app/*
     - network.connect: api.unknown.com:443

# Approve request
$ airssys-wasm approval approve unknown-tool
✅ Approved: unknown-tool

# Deny request
$ airssys-wasm approval deny data-processor --reason "Overly broad filesystem access"
❌ Denied: data-processor
```

---

## Quality Gates

### Cargo Clippy Requirements
- **Command**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Target**: Zero warnings (deny warnings)
- **Enforced Lints**: `unwrap_used`, `expect_used`, `panic` (deny)

### Rustdoc Requirements
- **Command**: `cargo doc --no-deps --document-private-items`
- **Target**: Zero rustdoc warnings
- **Standards**: Microsoft Rust Guidelines (M-MODULE-DOCS, M-CANONICAL-DOCS)

### Test Coverage Targets
- **Unit Test Coverage**: >90% (all workflow logic)
- **Integration Test Coverage**: 15+ integration tests
- **Edge Case Coverage**: 20+ edge case tests
- **Total Tests**: 50+ test cases

---

## Timeline Estimate

| Step | Description | Time | Cumulative |
|------|-------------|------|------------|
| 1 | Approval module structure | 30 min | 30 min |
| 2 | ApprovalState enum | 1 hour | 1.5 hours |
| 3 | StateTransition | 30 min | 2 hours |
| 4 | ApprovalRequest | 1.5 hours | 3.5 hours |
| 5 | ApprovalStore | 2.5 hours | 6 hours |
| 6 | ReviewQueue core | 2 hours | 8 hours |
| 7 | Review operations | 2 hours | 10 hours |
| 8 | Auto-approve workflow | 2 hours | 12 hours |
| 9 | Review workflow | 2.5 hours | 14.5 hours |
| 10 | Bypass workflow | 1.5 hours | 16 hours |
| 11 | Orchestrator | 2 hours | 18 hours |
| 12 | Decision types | 1 hour | 19 hours |
| 13 | Prior approval check | 1 hour | 20 hours |
| 14 | Concurrent handling | 1.5 hours | 21.5 hours |
| 15 | Audit logging | 1 hour | 22.5 hours |
| 16 | Integration tests | 2 hours | 24.5 hours |
| 17 | Documentation | 2 hours | 26.5 hours |
| 18 | Examples | 1.5 hours | 28 hours |
| 19 | CLI stubs | 1 hour | 29 hours |
| 20 | Final quality gates | 30 min | **25 hours** |

**Total Duration**: 25 hours ≈ **3-5 days** (6-8 hour workdays)

**Breakdown by Activity**:
- Core implementation: 17 hours (68%)
- Testing: 4 hours (16%)
- Documentation: 3.5 hours (14%)
- Quality assurance: 0.5 hours (2%)

---

## Risk Assessment

### Technical Risks

| Risk | Severity | Probability | Mitigation |
|------|----------|-------------|------------|
| **State Machine Bugs** | High | Medium | Comprehensive state transition tests |
| **Filesystem Race Conditions** | Medium | Medium | Atomic file operations, locks |
| **Queue Overflow** | Medium | Low | Queue size limits, cleanup policy |
| **Prior Approval Conflicts** | Low | Medium | Version-aware approval matching |

---

## Standards Compliance

### PROJECTS_STANDARD.md
- §2.1: 3-layer import organization ✅
- §4.3: Module architecture (mod.rs only re-exports) ✅
- §5.1: Dependency management ✅
- §6.1: YAGNI principles ✅

### Microsoft Rust Guidelines
- M-DESIGN-FOR-AI: Clear API, extensive docs ✅
- M-CANONICAL-DOCS: Comprehensive public API docs ✅
- M-EXAMPLES: Examples for all workflows ✅

### ADR Compliance
- ADR-WASM-005: Capability-Based Security Model ✅
- ADR-WASM-010: Implementation Strategy ✅

---

## Approval Status

**Planner**: Memory Bank Planner  
**Date**: 2025-12-17  
**Status**: ✅ **APPROVED** - Ready for implementation

This plan provides a comprehensive blueprint for implementing the approval workflow engine with clear state machines, persistent storage, and production-ready documentation.

**Ready to Start:** Task 2.2 implementation can begin after Task 2.1 completion.
