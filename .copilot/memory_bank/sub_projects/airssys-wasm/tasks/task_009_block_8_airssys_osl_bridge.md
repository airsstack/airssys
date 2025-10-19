# [WASM-TASK-009] - Block 8: AirsSys-OSL Bridge

**Status:** not-started  
**Added:** 2025-10-20  
**Updated:** 2025-10-20  
**Priority:** Critical Path - Integration Layer  
**Layer:** 3 - Integration  
**Block:** 8 of 11  
**Estimated Effort:** 5-6 weeks  

## Overview

Implement comprehensive AirsSys-OSL bridge providing WASM host functions for filesystem, network, and process operations with layered security (capability checks → RBAC/ACL → audit logging), error translation between OSL Result types and WASM traps, async operation handling via tokio runtime integration, and resource lifecycle management achieving <100μs host function call overhead.

## Context

**Current State:**
- airssys-osl complete: 85% (filesystem, network, process operations)
- OSL security: RBAC, ACL, audit logging operational
- Architecture: KNOWLEDGE-WASM-011 (OSL Bridge Design)
- WIT interfaces: Defined in Block 2 (WASM-TASK-003)
- Capability system: Complete in Block 4 (WASM-TASK-005)

**Problem Statement:**
WASM components need safe access to host system operations:
1. **Filesystem Access** - Read/write files, list directories
2. **Network Operations** - HTTP requests, TCP/UDP sockets
3. **Process Operations** - Spawn processes, execute commands
4. **Security Enforcement** - Multi-layer security checks
5. **Error Handling** - OSL errors → WASM traps
6. **Async Operations** - Bridge sync WASM to async OSL
7. **Resource Management** - Handle file descriptors, network connections

Requirements:
- Complete OSL operation coverage (filesystem, network, process)
- Layered security: Capabilities → RBAC/ACL → Audit
- Error translation preserving context
- Async operation support (bridge to tokio)
- Resource tracking and cleanup
- Minimal call overhead (<100μs)

**Why This Block Matters:**
Without OSL bridge:
- Components cannot access filesystem
- Components cannot make network requests
- Components cannot spawn processes
- No real-world I/O capabilities
- Framework limited to pure computation

This block enables real I/O capabilities for components.

## Objectives

### Primary Objective
Implement comprehensive AirsSys-OSL bridge with host functions for filesystem/network/process operations, layered security integration (capability checks → OSL RBAC/ACL → audit logging), error translation, async operation handling, and resource lifecycle management achieving <100μs host function call overhead.

### Secondary Objectives
- Cover 100% of OSL operation surface area
- Achieve <100μs host function call overhead
- Zero security bypasses (complete enforcement)
- Comprehensive audit trail for all operations
- Graceful error handling and reporting
- Resource leak prevention

## Scope

### In Scope
1. **Filesystem Host Functions** - read, write, list, stat, delete operations
2. **Network Host Functions** - HTTP requests, TCP/UDP sockets
3. **Process Host Functions** - spawn, execute, signal operations
4. **Security Integration** - Capability checks + OSL RBAC/ACL
5. **Error Translation** - OSL Result → WASM trap with context
6. **Async Bridge** - Sync WASM → Async OSL (tokio)
7. **Resource Management** - FD tracking, connection lifecycle
8. **Audit Logging** - All operations logged via OSL

### Out of Scope
- Direct syscall exposure (OSL provides abstraction)
- Unrestricted filesystem access (capability-restricted)
- Raw socket access (OSL provides safe wrappers)
- Kernel driver operations (OSL level only)
- Custom security policies (use OSL RBAC/ACL)

## Implementation Plan

### Phase 1: OSL Integration Foundation (Week 1)

#### Task 1.1: OSL Dependency Integration
**Deliverables:**
- airssys-osl crate integration
- OSL operation context setup
- OSL security context initialization
- Dependency configuration
- Integration documentation

**Success Criteria:**
- OSL crate integrated correctly
- Operations executable from host
- Security context configurable
- Dependencies resolve
- Clear integration guide

#### Task 1.2: Host Function Error Translation
**Deliverables:**
- Error translation layer (OSL Error → WASM Trap)
- Context preservation in errors
- Error code mapping
- User-friendly error messages
- Error translation documentation

**Success Criteria:**
- All OSL errors translatable
- Context preserved (stack trace, operation)
- Error codes consistent
- Messages helpful for debugging
- Translation efficient (<1μs)

#### Task 1.3: Async Bridge Design
**Deliverables:**
- Sync WASM → Async OSL bridge pattern
- Tokio runtime integration
- Operation timeout handling
- Cancellation support
- Async bridge documentation

**Success Criteria:**
- WASM calls bridge to async OSL
- No blocking of actor thread
- Timeouts enforced
- Cancellation works
- Pattern well-documented

---

### Phase 2: Filesystem Host Functions (Week 1-2)

#### Task 2.1: File Read/Write Operations
**Deliverables:**
- fs-read host function (read file contents)
- fs-write host function (write file contents)
- fs-append host function (append to file)
- Path validation and sanitization
- Filesystem operation documentation

**Success Criteria:**
- Read/write operations functional
- Path validation prevents escapes
- Large files handled efficiently
- Errors translated correctly
- Clear operation documentation

#### Task 2.2: Directory Operations
**Deliverables:**
- fs-list host function (list directory contents)
- fs-stat host function (file metadata)
- fs-create-dir host function (create directory)
- fs-remove-dir host function (remove directory)
- Directory operation documentation

**Success Criteria:**
- Directory operations functional
- Recursive operations supported
- Symlink handling safe
- Errors handled gracefully
- Operations well-documented

#### Task 2.3: File Lifecycle Operations
**Deliverables:**
- fs-delete host function (delete file)
- fs-rename host function (rename/move file)
- fs-copy host function (copy file)
- File descriptor tracking
- Lifecycle operation documentation

**Success Criteria:**
- Lifecycle operations functional
- File descriptors tracked
- No resource leaks
- Atomic operations where possible
- Clear documentation

---

### Phase 3: Network Host Functions (Week 2-3)

#### Task 3.1: HTTP Client Operations
**Deliverables:**
- http-get host function (HTTP GET request)
- http-post host function (HTTP POST request)
- http-put/delete host functions
- Request/response header handling
- HTTP operation documentation

**Success Criteria:**
- HTTP operations functional
- Headers handled correctly
- Response bodies streamed efficiently
- Timeouts enforced
- Clear HTTP guide

#### Task 3.2: TCP Socket Operations
**Deliverables:**
- tcp-connect host function (connect to TCP server)
- tcp-send host function (send data)
- tcp-receive host function (receive data)
- tcp-close host function (close connection)
- TCP operation documentation

**Success Criteria:**
- TCP operations functional
- Connection lifecycle managed
- Data sent/received correctly
- Connections closed properly
- Operations documented

#### Task 3.3: UDP Socket Operations
**Deliverables:**
- udp-bind host function (bind UDP socket)
- udp-send host function (send datagram)
- udp-receive host function (receive datagram)
- udp-close host function (close socket)
- UDP operation documentation

**Success Criteria:**
- UDP operations functional
- Socket lifecycle managed
- Datagrams sent/received correctly
- Sockets closed properly
- Clear UDP documentation

---

### Phase 4: Process Host Functions (Week 3-4)

#### Task 4.1: Process Spawn Operations
**Deliverables:**
- process-spawn host function (spawn new process)
- Process argument handling
- Environment variable passing
- Working directory configuration
- Spawn operation documentation

**Success Criteria:**
- Process spawning functional
- Arguments passed correctly
- Environment configured
- Working directory set
- Clear spawn guide

#### Task 4.2: Process Control Operations
**Deliverables:**
- process-wait host function (wait for exit)
- process-kill host function (send signal)
- process-stdin/stdout/stderr handling
- Process exit code retrieval
- Control operation documentation

**Success Criteria:**
- Process control functional
- Signals sent correctly
- I/O streams handled
- Exit codes retrieved
- Operations documented

#### Task 4.3: Command Execution Helper
**Deliverables:**
- process-execute host function (spawn + wait)
- Output capture (stdout/stderr)
- Timeout enforcement
- Exit code checking
- Execution helper documentation

**Success Criteria:**
- Execution helper convenient
- Output captured correctly
- Timeouts prevent hangs
- Exit codes checked
- Helper well-documented

---

### Phase 5: Security Integration (Week 4-5)

#### Task 5.1: Capability-Based Access Control
**Deliverables:**
- Capability check before every OSL call
- Filesystem glob pattern matching
- Network domain/port checking
- Process command whitelist/blacklist
- Capability integration documentation

**Success Criteria:**
- All operations check capabilities first
- Filesystem patterns matched correctly
- Network restrictions enforced
- Process restrictions work
- No bypass vulnerabilities

#### Task 5.2: OSL RBAC/ACL Integration
**Deliverables:**
- OSL security context per component
- RBAC role assignment
- ACL permission checking
- Security context propagation
- RBAC/ACL integration documentation

**Success Criteria:**
- OSL security context configured
- RBAC roles enforced
- ACL permissions checked
- Context propagates correctly
- Integration comprehensive

#### Task 5.3: Audit Logging Integration
**Deliverables:**
- Audit log for every OSL operation
- Component identity in logs
- Operation parameters logged
- Success/failure recording
- Audit trail documentation

**Success Criteria:**
- All operations audited
- Component identity tracked
- Parameters logged safely (no secrets)
- Success/failure clear
- Audit trail comprehensive

---

### Phase 6: Resource Management and Testing (Week 5-6)

#### Task 6.1: Resource Lifecycle Management
**Deliverables:**
- File descriptor tracking per component
- Network connection tracking
- Process handle tracking
- Automatic cleanup on component shutdown
- Resource management documentation

**Success Criteria:**
- All resources tracked
- Tracking per component
- Cleanup automatic
- No resource leaks
- Clear lifecycle guarantees

#### Task 6.2: Performance Optimization
**Deliverables:**
- Host function call overhead benchmarks
- OSL call batching for efficiency
- Resource pooling where applicable
- Performance profiling results
- Optimization documentation

**Success Criteria:**
- Call overhead <100μs
- Batching reduces overhead
- Resource pooling works
- Performance targets met
- Optimizations documented

#### Task 6.3: Comprehensive Bridge Testing
**Deliverables:**
- Host function test suite
- Security enforcement tests
- Error handling tests
- Resource leak tests
- Performance benchmarks

**Success Criteria:**
- Test coverage >95%
- All host functions tested
- Security enforced in tests
- No resource leaks detected
- Performance validated

---

## Success Criteria

### Definition of Done
This task is complete when:

1. ✅ **Filesystem Operations Complete**
   - Read/write/list/stat/delete functional
   - Path validation prevents escapes
   - File descriptors tracked
   - Operations audited

2. ✅ **Network Operations Complete**
   - HTTP/TCP/UDP operations functional
   - Connection lifecycle managed
   - Network restrictions enforced
   - Operations audited

3. ✅ **Process Operations Complete**
   - Spawn/wait/kill functional
   - I/O streams handled
   - Process restrictions enforced
   - Operations audited

4. ✅ **Security Integration Complete**
   - Capabilities checked first
   - OSL RBAC/ACL enforced
   - Audit logging comprehensive
   - No bypass vulnerabilities

5. ✅ **Error Translation Working**
   - All OSL errors translatable
   - Context preserved
   - Error messages helpful
   - Translation efficient

6. ✅ **Async Bridge Operational**
   - Sync WASM → Async OSL working
   - No thread blocking
   - Timeouts enforced
   - Cancellation supported

7. ✅ **Resource Management Reliable**
   - All resources tracked
   - Automatic cleanup working
   - No resource leaks
   - Clear lifecycle guarantees

8. ✅ **Testing & Documentation Complete**
   - Test coverage >95%
   - Performance targets met (<100μs)
   - Complete bridge guide
   - Security audit passed

## Dependencies

### Upstream Dependencies
- ✅ airssys-osl (85% complete) - **REQUIRED** for all operations
- ✅ WASM-TASK-003: WIT Interface System (Block 2) - **REQUIRED** for host function signatures
- ✅ WASM-TASK-005: Security & Isolation (Block 4) - **REQUIRED** for capability checks
- ✅ KNOWLEDGE-WASM-011: OSL Bridge Design - **COMPLETE**

### Downstream Dependencies (Blocks This Task)
- WASM-TASK-010: Monitoring & Observability (Block 9) - needs bridge metrics
- WASM-TASK-011: Component SDK (Block 10) - needs OSL API wrappers
- All production components - depend on OSL operations

### External Dependencies
- airssys-osl crate (filesystem, network, process operations)
- tokio (async runtime for OSL operations)
- wasmtime (host function registration)

## Risks and Mitigations

### Risk 1: OSL API Surface Changes
**Impact:** High - Bridge must match OSL API  
**Probability:** Medium - OSL still evolving (85% complete)  
**Mitigation:**
- Close coordination with OSL team
- Version OSL dependency carefully
- Abstract OSL details behind bridge layer
- Update bridge when OSL changes

### Risk 2: Performance Overhead
**Impact:** Medium - Slow bridge impacts all components  
**Probability:** Low - Host functions are efficient  
**Mitigation:**
- Benchmark continuously
- Profile hot paths
- Batch operations where possible
- Target <100μs overhead

### Risk 3: Security Bypass Vulnerabilities
**Impact:** Critical - Security bypass unacceptable  
**Probability:** Low - Layered security defense  
**Mitigation:**
- Security review all host functions
- Automated security tests
- Penetration testing
- External security audit

### Risk 4: Resource Leaks
**Impact:** High - Leaks degrade performance over time  
**Probability:** Medium - Resource tracking complex  
**Mitigation:**
- Automatic cleanup on shutdown
- Resource leak detection tests
- Regular leak testing in CI
- Clear resource lifecycle rules

### Risk 5: Error Context Loss
**Impact:** Medium - Poor errors hard to debug  
**Probability:** Medium - Translation can lose context  
**Mitigation:**
- Preserve OSL error context
- Include component ID in errors
- Log detailed error information
- Provide debugging tools

## Progress Tracking

**Overall Status:** not-started - 0%

### Phase Breakdown
| Phase | Description | Status | Estimated Duration | Notes |
|-------|-------------|--------|-------------------|-------|
| 1 | OSL Integration Foundation | not-started | Week 1 | Foundation |
| 2 | Filesystem Host Functions | not-started | Week 1-2 | File I/O |
| 3 | Network Host Functions | not-started | Week 2-3 | Network I/O |
| 4 | Process Host Functions | not-started | Week 3-4 | Process management |
| 5 | Security Integration | not-started | Week 4-5 | Layered security |
| 6 | Resource Management and Testing | not-started | Week 5-6 | Reliability & QA |

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 1.1 | OSL Dependency Integration | not-started | - | Foundation |
| 1.2 | Host Function Error Translation | not-started | - | Error handling |
| 1.3 | Async Bridge Design | not-started | - | Sync→Async bridge |
| 2.1 | File Read/Write Operations | not-started | - | File I/O |
| 2.2 | Directory Operations | not-started | - | Directory ops |
| 2.3 | File Lifecycle Operations | not-started | - | Lifecycle |
| 3.1 | HTTP Client Operations | not-started | - | HTTP |
| 3.2 | TCP Socket Operations | not-started | - | TCP |
| 3.3 | UDP Socket Operations | not-started | - | UDP |
| 4.1 | Process Spawn Operations | not-started | - | Process spawn |
| 4.2 | Process Control Operations | not-started | - | Process control |
| 4.3 | Command Execution Helper | not-started | - | Convenience |
| 5.1 | Capability-Based Access Control | not-started | - | Capability checks |
| 5.2 | OSL RBAC/ACL Integration | not-started | - | RBAC/ACL |
| 5.3 | Audit Logging Integration | not-started | - | Audit trail |
| 6.1 | Resource Lifecycle Management | not-started | - | Resource tracking |
| 6.2 | Performance Optimization | not-started | - | Performance |
| 6.3 | Comprehensive Bridge Testing | not-started | - | Quality assurance |

## Progress Log

*No progress yet - task just created*

## Related Documentation

### ADRs
- **ADR-WASM-010: OSL Bridge Security Layers** - (Future) Security layer design rationale

### Knowledge Documentation
- **KNOWLEDGE-WASM-011: OSL Bridge Design** - Primary bridge design reference
- **KNOWLEDGE-WASM-004: WIT Management Architecture** - Host function WIT interfaces
- **KNOWLEDGE-WASM-006: Security & Isolation Architecture** - Capability integration

### External References
- [AirsSys OSL Documentation](../../airssys-osl/README.md)
- [WebAssembly Host Functions](https://docs.wasmtime.dev/api/wasmtime/struct.Linker.html)
- [Tokio Async Runtime](https://tokio.rs/)

## Notes

**Layered Security Model:**
Three security layers in order:
1. **Capability Check** - Component has declared permission?
2. **OSL RBAC/ACL** - Role/ACL allows operation?
3. **Audit Logging** - Log operation for compliance

All three must pass for operation to proceed.

**Error Translation Principles:**
OSL errors must translate to WASM traps while preserving:
- Error code (filesystem, network, permission, etc.)
- Context (what operation, which file/URL/process)
- Component identity (which component caused error)
- Stack trace (where in component code)

**Async Bridge Pattern:**
WASM is synchronous, OSL is async (tokio).
Bridge pattern:
1. WASM calls host function (sync)
2. Host function blocks on tokio future
3. OSL operation completes (async)
4. Future resolves, host function returns

No actor thread blocking achieved by:
- Each component has dedicated thread pool
- Blocking only blocks component thread, not actor system

**Resource Tracking Critical:**
Components can leak resources:
- File descriptors (open files)
- Network connections (sockets)
- Process handles (child processes)

Bridge tracks all resources per component.
On component shutdown, automatically close/kill/cleanup.

**Path Validation Security:**
Filesystem operations MUST validate paths:
- No `..` path traversal escapes
- No absolute paths (unless explicitly allowed)
- No symlink attacks
- Canonical path resolution

**Network Domain Restrictions:**
Capabilities specify allowed network access:
- `network:http:example.com` - HTTP to example.com only
- `network:tcp:*:8080` - TCP to any host on port 8080
- `network:udp:192.168.1.*:*` - UDP to local network

Bridge enforces these patterns.

**Process Command Whitelist:**
Capabilities specify allowed process execution:
- `process:spawn:git` - Only spawn git command
- `process:spawn:*` - Spawn any command (dangerous)
- Whitelist safer than blacklist

**Audit Logging Requirements:**
Every OSL operation logged with:
- Timestamp (ISO8601)
- Component ID
- Operation type (fs-read, http-get, process-spawn)
- Operation target (file path, URL, command)
- Result (success/failure with error)

No secrets logged (passwords, tokens, keys).

**Performance Considerations:**
Host function overhead target: <100μs
Breakdown:
- Capability check: <5μs
- OSL security context: <10μs
- Async bridge overhead: <20μs
- OSL operation: varies (file I/O, network latency)
- Error translation: <1μs
- Audit logging: <10μs (async)

**Phase 2 Enhancements:**
- Filesystem notification/watch operations
- Advanced network protocols (gRPC, WebSocket)
- Process IPC (pipes, shared memory)
- Credential management (secure token storage)
- OSL operation batching for efficiency
