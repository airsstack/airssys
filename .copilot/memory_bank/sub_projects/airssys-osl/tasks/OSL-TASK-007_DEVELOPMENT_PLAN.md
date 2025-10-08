# OSL-TASK-007 Development Plan

**Task:** Implement Concrete Operation Types  
**Status:** Pending  
**Created:** 2025-10-08  
**Total Estimated Effort:** 22-29 hours (2.75-3.5 days)

## Overview

This development plan breaks down OSL-TASK-007 into actionable phases based on the task's acceptance criteria. Each phase has specific deliverables, time estimates, and quality gates.

## Phase 1: Module Structure Setup

**Duration:** 2-3 hours  
**Acceptance Criteria Reference:** §1 - Module Structure Created

### Deliverables
- Create `src/operations/` directory
- Create `src/operations/mod.rs` with module exports and documentation
- Create `src/operations/filesystem.rs` (empty, structure only)
- Create `src/operations/process.rs` (empty, structure only)
- Create `src/operations/network.rs` (empty, structure only)
- Update `src/lib.rs` to include operations module

### Module Structure
```
src/operations/
├── mod.rs              # Operation type exports
├── filesystem.rs       # Filesystem operation types
├── process.rs          # Process operation types
└── network.rs          # Network operation types
```

### Quality Gates
- ✅ All modules compile without errors
- ✅ Module exports are public and documented
- ✅ Zero clippy warnings
- ✅ rustdoc builds successfully

### Workspace Standards Compliance
- §2.1: 3-Layer import organization in all files
- §3.2: Use `chrono::DateTime<Utc>` for timestamps
- §4.3: Clean module separation (mod.rs only exports)

---

## Phase 2: Filesystem Operations Implementation

**Duration:** 4-5 hours  
**Acceptance Criteria Reference:** §2 - Filesystem Operations Implementation

### Deliverables

#### 1. FileReadOperation
```rust
#[derive(Debug, Clone)]
pub struct FileReadOperation {
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub operation_id: Option<String>,
}
```
- Constructor: `new(path)`
- Builders: `with_timestamp()`, `with_operation_id()`
- Permission: `FilesystemRead(path)`

#### 2. FileWriteOperation
```rust
#[derive(Debug, Clone)]
pub struct FileWriteOperation {
    pub path: String,
    pub content: Vec<u8>,
    pub append: bool,
    pub created_at: DateTime<Utc>,
    pub operation_id: Option<String>,
}
```
- Constructors: `new(path, content)`, `append(path, content)`
- Permission: `FilesystemWrite(path)`

#### 3. DirectoryCreateOperation
```rust
#[derive(Debug, Clone)]
pub struct DirectoryCreateOperation {
    pub path: String,
    pub recursive: bool,
    pub created_at: DateTime<Utc>,
    pub operation_id: Option<String>,
}
```
- Constructor: `new(path)`
- Builder: `recursive()`
- Permission: `FilesystemWrite(path)`

#### 4. DirectoryListOperation
```rust
#[derive(Debug, Clone)]
pub struct DirectoryListOperation {
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub operation_id: Option<String>,
}
```
- Constructor: `new(path)`
- Permission: `FilesystemRead(path)`

#### 5. FileDeleteOperation
```rust
#[derive(Debug, Clone)]
pub struct FileDeleteOperation {
    pub path: String,
    pub created_at: DateTime<Utc>,
    pub operation_id: Option<String>,
}
```
- Constructor: `new(path)`
- Permission: `FilesystemWrite(path)`

### Quality Gates
- ✅ All 5 operations implement `Operation` trait
- ✅ All operations are `Clone + Debug + Send + Sync`
- ✅ Comprehensive rustdoc for all types and methods
- ✅ All operations store operation data (no unused parameters)
- ✅ All operations define required permissions properly
- ✅ Unit tests for Operation trait implementations
- ✅ Zero compiler warnings

---

## Phase 3: Process Operations Implementation

**Duration:** 3-4 hours  
**Acceptance Criteria Reference:** §3 - Process Operations Implementation

### Deliverables

#### 1. ProcessSpawnOperation
```rust
#[derive(Debug, Clone)]
pub struct ProcessSpawnOperation {
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: Option<String>,
    pub created_at: DateTime<Utc>,
    pub operation_id: Option<String>,
}
```
- Constructor: `new(command)`
- Builders: `with_args()`, `arg()`, `with_env()`, `env()`, `working_dir()`
- Permission: `ProcessSpawn`
- Elevated privileges: `true`

#### 2. ProcessKillOperation
```rust
#[derive(Debug, Clone)]
pub struct ProcessKillOperation {
    pub pid: u32,
    pub force: bool,
    pub created_at: DateTime<Utc>,
    pub operation_id: Option<String>,
}
```
- Constructor: `new(pid)`
- Builder: `force()`
- Permission: `ProcessKill`
- Elevated privileges: `true`

#### 3. ProcessSignalOperation
```rust
#[derive(Debug, Clone)]
pub struct ProcessSignalOperation {
    pub pid: u32,
    pub signal: i32,
    pub created_at: DateTime<Utc>,
    pub operation_id: Option<String>,
}
```
- Constructor: `new(pid, signal)`
- Permission: `ProcessSignal`
- Elevated privileges: `true`

### Quality Gates
- ✅ All 3 operations implement `Operation` trait
- ✅ All operations are `Clone + Debug + Send + Sync`
- ✅ Comprehensive rustdoc with examples
- ✅ All operations store operation data
- ✅ All operations define required permissions
- ✅ All operations set `requires_elevated_privileges() = true`
- ✅ Unit tests for Operation trait implementations
- ✅ Zero compiler warnings

---

## Phase 4: Network Operations Implementation

**Duration:** 3-4 hours  
**Acceptance Criteria Reference:** §4 - Network Operations Implementation

### Deliverables

#### 1. NetworkConnectOperation
```rust
#[derive(Debug, Clone)]
pub struct NetworkConnectOperation {
    pub address: String,
    pub timeout: Option<Duration>,
    pub created_at: DateTime<Utc>,
    pub operation_id: Option<String>,
}
```
- Constructor: `new(address)`
- Builder: `with_timeout()`
- Permission: `NetworkConnect(address)`
- Elevated privileges: `true`

#### 2. NetworkListenOperation
```rust
#[derive(Debug, Clone)]
pub struct NetworkListenOperation {
    pub address: String,
    pub backlog: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub operation_id: Option<String>,
}
```
- Constructor: `new(address)`
- Builder: `with_backlog()`
- Permission: `NetworkListen(address)`
- Elevated privileges: `true`

#### 3. NetworkSocketOperation
```rust
#[derive(Debug, Clone)]
pub struct NetworkSocketOperation {
    pub socket_type: String,
    pub created_at: DateTime<Utc>,
    pub operation_id: Option<String>,
}
```
- Constructor: `new(socket_type)`
- Permission: `NetworkSocket`
- Elevated privileges: `true`

### Quality Gates
- ✅ All 3 operations implement `Operation` trait
- ✅ All operations are `Clone + Debug + Send + Sync`
- ✅ Comprehensive rustdoc with examples
- ✅ All operations store operation data
- ✅ All operations define required permissions
- ✅ All operations set `requires_elevated_privileges() = true`
- ✅ Unit tests for Operation trait implementations
- ✅ Zero compiler warnings

---

## Phase 5: Framework Integration

**Duration:** 4-5 hours  
**Acceptance Criteria Reference:** §5 - Framework Integration

### Deliverables

#### Update `src/framework/operations.rs`

**Pattern: Transform placeholder builders to concrete operations**

Before (Phase 3 placeholder):
```rust
pub fn read_file(self, _path: &str) -> FileOperation<'a> {  // ❌ _path unused
    FileOperation {
        builder: self,
        operation: "read".to_string(),
    }
}
```

After (Phase 5 real operations):
```rust
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

#### Operations to Update
- ✅ `read_file()` → `FileReadOperationWrapper`
- ✅ `write_file()` → `FileWriteOperationWrapper`
- ✅ `create_directory()` → `DirectoryCreateOperationWrapper`
- ✅ `list_directory()` → `DirectoryListOperationWrapper`
- ✅ `delete_file()` → `FileDeleteOperationWrapper`
- ✅ `spawn_process()` → `ProcessSpawnOperationWrapper`
- ✅ `kill_process()` → `ProcessKillOperationWrapper`
- ✅ `signal_process()` → `ProcessSignalOperationWrapper`
- ✅ `connect()` → `NetworkConnectOperationWrapper`
- ✅ `listen()` → `NetworkListenOperationWrapper`
- ✅ `create_socket()` → `NetworkSocketOperationWrapper`

### Quality Gates
- ✅ Remove `_` prefix from all parameters (now used)
- ✅ All operation wrappers delegate to `framework.execute()`
- ✅ All operations flow through proper execution path
- ✅ No placeholder return types remain
- ✅ Zero unused parameters
- ✅ Zero compiler warnings
- ✅ All clippy lints passing

### Integration Testing
- ✅ Framework builders create correct operations
- ✅ Operations flow through `framework.execute()`
- ✅ Placeholder executors receive operations
- ✅ Middleware can validate operations

---

## Phase 6: Quality Gates - Testing & Documentation

**Duration:** 6-8 hours  
**Acceptance Criteria Reference:** §6 - Quality Gates

### Unit Testing (2-3 hours)

**Per operation module (filesystem, process, network):**
- ✅ Operation trait implementation correctness
- ✅ Permission calculation accuracy
- ✅ Operation ID generation and uniqueness
- ✅ Timestamp handling (Utc::now() and custom)
- ✅ Builder pattern fluent interfaces
- ✅ Clone implementation works correctly
- ✅ Debug formatting is useful

**Test coverage target:** >95% for operation types

### Integration Testing (2-3 hours)

**Framework integration:**
- ✅ Framework builders create correct operation types
- ✅ Operations flow through `framework.execute()`
- ✅ Placeholder executors can receive operations
- ✅ Middleware can validate operations
- ✅ Security policies can evaluate operations

**Cross-module integration:**
- ✅ All 11 operations work with framework
- ✅ All operations work with middleware pipeline
- ✅ All operations work with security validation

### Property Testing (1 hour)

**Property tests:**
- ✅ All operations are `Clone`
- ✅ All operations are `Send + Sync`
- ✅ Operation IDs are unique across instances
- ✅ Timestamps are reasonable (not future, not ancient)
- ✅ Builder methods are idempotent where appropriate

### Documentation (1-2 hours)

**Rustdoc requirements:**
- ✅ Comprehensive rustdoc for all 11 operation types
- ✅ Usage examples for each operation type
- ✅ Permission model documentation
- ✅ Builder pattern examples
- ✅ Integration examples with framework

**Migration documentation:**
- ✅ Migration guide from Phase 3 placeholders
- ✅ Breaking changes documentation
- ✅ API compatibility notes

### Final Quality Checks

**Code quality:**
- ✅ Zero compiler warnings
- ✅ All clippy lints passing
- ✅ No unused code or parameters
- ✅ No `_` prefixed parameters remain
- ✅ All imports follow §2.1 (3-layer organization)
- ✅ All timestamps use `chrono::DateTime<Utc>` (§3.2)
- ✅ All modules follow §4.3 (clean separation)

**Testing:**
- ✅ All tests pass (`cargo test --package airssys-osl`)
- ✅ >95% code coverage on operation types
- ✅ Integration tests pass
- ✅ Property tests pass

**Documentation:**
- ✅ rustdoc builds without warnings
- ✅ All public APIs documented
- ✅ Examples compile and run

---

## Success Metrics

### Code Quality Metrics
- ✅ Zero unused parameters (no `_` prefixes)
- ✅ All operations properly implement `Operation` trait
- ✅ 100% of operations define required permissions
- ✅ All operations store necessary data
- ✅ Zero compiler warnings
- ✅ All clippy lints passing

### Integration Metrics
- ✅ Framework builders create real operations
- ✅ Operations can be validated by middleware
- ✅ Operations can be executed by executors
- ✅ Security policies can evaluate operations

### Testing Metrics
- ✅ >95% code coverage on operation types
- ✅ All trait methods tested
- ✅ All builder patterns tested
- ✅ Integration tests pass
- ✅ Property tests pass

---

## What This Unlocks

### Immediate Unblocks
- **OSL-TASK-008** (Platform Executors) - Can execute real operations
- **OSL-TASK-006 Phase 4** (Testing) - Has real operations to test
- **OSL-TASK-003** (Security Middleware) - Can validate permissions
- **OSL-TASK-004** (Pipeline Framework) - Can orchestrate operations

### Technical Debt Resolution
- **DEBT-002** (Framework-Core Integration Gap) - Resolved

### Capabilities Enabled
- Real execution flow through framework
- Security validation with actual permissions
- Middleware can inspect operation data
- Testing with concrete operations

---

## Notes

### Critical Path
This task is on the **critical path** for making airssys-osl functional. Without concrete operations, the framework cannot execute real system operations.

### Breaking Changes
- Operation wrapper types change (e.g., `FileOperation` → `FileReadOperationWrapper`)
- Parameters now required (can't use `_path`)
- Execute returns real results (not placeholders)

### Compatibility
- User-facing API remains the same (builders)
- Internal implementation changes only
- No changes to core abstractions

### Workspace Standards
All code must comply with:
- **§2.1:** 3-Layer import organization
- **§3.2:** `chrono::DateTime<Utc>` for timestamps
- **§4.3:** Clean module separation
- **§6.1:** YAGNI principles
- **§6.2:** Avoid `dyn` patterns (use generics)
- **§6.3:** Microsoft Rust Guidelines

### Cross-References
- **Task:** OSL-TASK-007-concrete-operations.md
- **Debt:** DEBT-002 (Framework-Core Integration Gap)
- **Knowledge:** KNOW-004 (Framework-Core Integration Pattern)
- **ADR-027:** Builder Pattern Architecture Implementation
- **ADR-025:** dyn Pattern Exception

---

## Timeline Summary

| Phase | Description | Duration | Cumulative |
|-------|-------------|----------|------------|
| 1 | Module Structure Setup | 2-3h | 2-3h |
| 2 | Filesystem Operations | 4-5h | 6-8h |
| 3 | Process Operations | 3-4h | 9-12h |
| 4 | Network Operations | 3-4h | 12-16h |
| 5 | Framework Integration | 4-5h | 16-21h |
| 6 | Quality Gates | 6-8h | 22-29h |

**Total Estimated Effort:** 22-29 hours (2.75-3.5 days)

This aligns with the original task estimate of **2-3 days**.
