# WIT Package Content Design - Interface Specifications

**Version:** 1.0.0

---

## Executive Summary

This document specifies the detailed interface content for all 7 packages in the airssys-wasm WIT structure. Each package section includes:
- Package purpose and responsibilities
- Key interface definitions
- Exported types and functions
- Dependencies on other packages
- File organization within package

This is a **design document** - actual `.wit` file implementation happens in Phase 2.

---

## Package 1: airssys:core-types@1.0.0

### Purpose

Foundation package providing common types, errors, and data structures shared across all other packages.

**Responsibilities:**
- Core error types (component errors, execution errors)
- Identifier types (component-id, request-id)
- Common result types and status enums
- Shared data structures (timestamps, metadata)

**Dependencies:** None (foundation package)

### Key Interfaces

#### Interface: `types`


**Exported Types:**

```wit
// Component Identity
record component-id {
    namespace: string,       // e.g., "airssys"
    name: string,           // e.g., "my-processor"
    version: string,        // e.g., "1.0.0"
}

// Request Tracking
type request-id = string;   // UUID for request correlation

// Timestamps
record timestamp {
    seconds: u64,           // Unix timestamp seconds
    nanoseconds: u32,       // Subsecond precision
}

// Common Result Types
variant execution-status {
    success,
    failed,
    timeout,
    cancelled,
}

// Error Types
variant component-error {
    initialization-failed(string),
    configuration-invalid(string),
    resource-exhausted(string),
    internal-error(string),
}

variant execution-error {
    invalid-input(string),
    processing-failed(string),
    timeout(string),
    resource-limit-exceeded(string),
}

variant file-error {
    not-found(string),
    permission-denied(string),
    already-exists(string),
    io-error(string),
}

variant network-error {
    connection-failed(string),
    timeout(string),
    invalid-url(string),
    protocol-error(string),
}

variant process-error {
    spawn-failed(string),
    not-found(string),
    permission-denied(string),
    timeout(string),
}

// Health Status
enum health-status {
    healthy,
    degraded,
    unhealthy,
    unknown,
}

// Log Levels
enum log-level {
    trace,
    debug,
    info,
    warn,
    error,
}
```

**Exported Functions:** None (pure type definitions)

### File Organization

```
wit/core/types/
├── types.wit          # All type definitions in single file
└── deps.toml          # Empty (no dependencies)
```

**Rationale:** All types in single file for simplicity, as they're closely related and foundational.

---

## Package 2: airssys:core-component@1.0.0

### Purpose

Component lifecycle interface definition - the contract ALL components must implement.

**Responsibilities:**
- Component initialization interface
- Execution and request handling
- Message handling (inter-component communication)
- Health check and metadata
- Graceful shutdown

**Dependencies:**
- `airssys:core-types@1.0.0` - Uses component-id, error types, health-status

### Key Interfaces

#### Interface: `component-lifecycle`


**Imported Types:**
```wit
use airssys:core-types@1.0.0.{
    component-id,
    component-error,
    execution-error,
    health-status,
    request-id
};
```

**Exported Types:**

```wit
// Component Configuration
record component-config {
    env-vars: list<tuple<string, string>>,
    config-data: option<list<u8>>,      // Multicodec-encoded
    resource-limits: resource-limits,
}

// Resource Limits
record resource-limits {
    max-memory-bytes: u64,
    max-cpu-time-ms: u64,
    max-execution-time-ms: u64,
}

// Execution Context
record execution-context {
    request-id: request-id,
    timeout-ms: u64,
    caller-info: option<caller-info>,
}

// Caller Information
record caller-info {
    component-id: option<component-id>,
    external-source: option<string>,
}

// Component Metadata
record component-metadata {
    name: string,
    version: string,
    description: string,
    author: string,
    supported-operations: list<string>,
    memory-requirements: memory-requirements,
}

// Memory Requirements
record memory-requirements {
    min-memory-bytes: u64,
    max-memory-bytes: u64,
    preferred-memory-bytes: u64,
}
```

**Exported Functions:**

```wit
// Initialization
init: func(config: component-config) -> result<_, component-error>;

// External RPC Execution
execute: func(
    operation: list<u8>,          // Multicodec-prefixed operation data
    context: execution-context
) -> result<list<u8>, execution-error>;

// Inter-Component Message Handling
handle-message: func(
    sender: component-id,
    message: list<u8>               // Multicodec-encoded message
) -> result<_, component-error>;

// Request-Response Callback
handle-callback: func(
    request-id: request-id,
    result: result<list<u8>, string>
) -> result<_, component-error>;

// Introspection
metadata: func() -> component-metadata;
health: func() -> health-status;

// Shutdown
shutdown: func() -> result<_, component-error>;
```

### File Organization

```
wit/core/component/
├── component.wit      # Package declaration, imports, and lifecycle interface
└── deps.toml          # Dependency on core-types
```

---

## Package 3: airssys:core-capabilities@1.0.0

### Purpose

Permission and capability system types - defines how components declare and request capabilities.

**Responsibilities:**
- Filesystem permission types
- Network permission types
- Process permission types
- Permission action enums
- Capability request structures

**Dependencies:**
- `airssys:core-types@1.0.0` - Uses error types

### Key Interfaces

#### Interface: `capabilities`


**Imported Types:**
```wit
use airssys:core-types@1.0.0.{component-error};
```

**Exported Types:**

```wit
// Filesystem Permissions
record filesystem-permission {
    action: filesystem-action,
    path-pattern: string,          // Glob pattern: "/data/**", "/*.txt"
}

enum filesystem-action {
    read,
    write,
    delete,
    list,
}

// Network Permissions
record network-permission {
    action: network-action,
    host-pattern: string,          // Wildcard: "api.example.com", "*.github.com"
    port: option<u16>,
}

enum network-action {
    outbound,
    inbound,
}

// Process Permissions
record process-permission {
    action: process-action,
    command-pattern: string,       // Glob pattern: "/usr/bin/*", "node"
}

enum process-action {
    spawn,
    kill,
    signal,
}

// Consolidated Permission Set
record requested-permissions {
    filesystem: list<filesystem-permission>,
    network: list<network-permission>,
    process: list<process-permission>,
}

// Permission Check Result
variant permission-result {
    granted,
    denied(string),               // Reason for denial
}
```

**Exported Functions:** None (pure type definitions for runtime use)

### File Organization

```
wit/core/capabilities/
├── capabilities.wit   # Permission types and structures
└── deps.toml          # Dependency on core-types
```

---

## Package 4: airssys:core-host@1.0.0

### Purpose

Essential host services available to ALL components - logging, messaging, timing, introspection.

**Responsibilities:**
- Structured logging interface
- Inter-component messaging (send-message, send-request)
- Time and timing services
- Component introspection

**Dependencies:**
- `airssys:core-types@1.0.0` - Uses component-id, error types, log-level
- `airssys:core-capabilities@1.0.0` - Uses permission result types (for future capability queries)

### Key Interfaces

#### Interface: `host-services`


**Imported Types:**
```wit
use airssys:core-types@1.0.0.{
    component-id,
    request-id,
    component-error,
    log-level,
    timestamp
};
```

**Exported Types:**

```wit
// Messaging Errors
variant messaging-error {
    component-not-found(string),
    send-failed(string),
    timeout(string),
    invalid-message(string),
}

// Component Metadata (returned from introspection)
record component-metadata {
    name: string,
    version: string,
    description: string,
    status: string,
}
```

**Exported Functions:**

```wit
// Logging
log: func(
    level: log-level,
    message: string,
    context: option<list<tuple<string, string>>>
);

// Messaging - Fire and Forget
send-message: func(
    target: component-id,
    message: list<u8>              // Multicodec-encoded
) -> result<_, messaging-error>;

// Messaging - Request-Response
send-request: func(
    target: component-id,
    request: list<u8>,             // Multicodec-encoded
    timeout-ms: u64
) -> result<request-id, messaging-error>;

// Messaging - Cancel Request
cancel-request: func(
    request-id: request-id
) -> result<_, messaging-error>;

// Timing Services
current-time-millis: func() -> u64;
sleep-millis: func(duration-ms: u64);

// Component Introspection
list-components: func() -> list<component-id>;
get-component-metadata: func(id: component-id) -> result<component-metadata, component-error>;
```

### File Organization

```
wit/core/host/
├── host.wit           # Host services interface
└── deps.toml          # Dependencies on core-types and core-capabilities
```

---

## Package 5: airssys:ext-filesystem@1.0.0

### Purpose

Filesystem operation interfaces - capability-gated file I/O operations.

**Responsibilities:**
- File read/write/delete operations
- Directory listing and traversal
- File metadata retrieval
- Path operations

**Dependencies:**
- `airssys:core-types@1.0.0` - Uses file-error
- `airssys:core-capabilities@1.0.0` - Uses filesystem-permission

**Note:** Access requires declared `filesystem-permission` in component manifest.

### Key Interfaces

#### Interface: `filesystem`


**Imported Types:**
```wit
use airssys:core-types@1.0.0.{file-error, timestamp};
use airssys:core-capabilities@1.0.0.{filesystem-permission};
```

**Exported Types:**

```wit
// File Metadata
record file-stat {
    size: u64,
    is-directory: bool,
    is-file: bool,
    is-symlink: bool,
    created-at: option<timestamp>,
    modified-at: option<timestamp>,
    accessed-at: option<timestamp>,
}

// Directory Entry
record dir-entry {
    name: string,
    path: string,
    file-type: file-type,
}

enum file-type {
    file,
    directory,
    symlink,
    unknown,
}
```

**Exported Functions:**

```wit
// File Operations
read-file: func(path: string) -> result<list<u8>, file-error>;
write-file: func(path: string, data: list<u8>) -> result<_, file-error>;
delete-file: func(path: string) -> result<_, file-error>;
file-exists: func(path: string) -> result<bool, file-error>;

// File Metadata
stat: func(path: string) -> result<file-stat, file-error>;

// Directory Operations
list-directory: func(path: string) -> result<list<dir-entry>, file-error>;
create-directory: func(path: string) -> result<_, file-error>;
remove-directory: func(path: string, recursive: bool) -> result<_, file-error>;
```

**Permission Enforcement:** Host checks `filesystem-permission` before executing operations.

### File Organization

```
wit/ext/filesystem/
├── filesystem.wit     # Filesystem interface
└── deps.toml          # Dependencies on core-types and core-capabilities
```

---

## Package 6: airssys:ext-network@1.0.0

### Purpose

Network operation interfaces - capability-gated HTTP and socket operations.

**Responsibilities:**
- HTTP client operations (GET, POST, etc.)
- TCP/UDP socket interfaces (future)
- Network error handling

**Dependencies:**
- `airssys:core-types@1.0.0` - Uses network-error
- `airssys:core-capabilities@1.0.0` - Uses network-permission

**Note:** Access requires declared `network-permission` in component manifest.

### Key Interfaces

#### Interface: `network`


**Imported Types:**
```wit
use airssys:core-types@1.0.0.{network-error};
use airssys:core-capabilities@1.0.0.{network-permission};
```

**Exported Types:**

```wit
// HTTP Request
record http-request {
    method: http-method,
    url: string,
    headers: list<tuple<string, string>>,
    body: option<list<u8>>,
    timeout-ms: u64,
}

enum http-method {
    get,
    post,
    put,
    delete,
    patch,
    head,
    options,
}

// HTTP Response
record http-response {
    status-code: u16,
    headers: list<tuple<string, string>>,
    body: list<u8>,
}

// Network Address (for future socket operations)
record network-address {
    host: string,
    port: u16,
}
```

**Exported Functions:**

```wit
// HTTP Client
http-request: func(request: http-request) -> result<http-response, network-error>;

// Future: TCP/UDP Socket Operations (commented out for Phase 1)
// tcp-connect: func(address: network-address) -> result<tcp-socket, network-error>;
// udp-bind: func(port: u16) -> result<udp-socket, network-error>;
```

**Permission Enforcement:** Host checks `network-permission` before executing operations.

### File Organization

```
wit/ext/network/
├── network.wit        # Network interface
└── deps.toml          # Dependencies on core-types and core-capabilities
```

---

## Package 7: airssys:ext-process@1.0.0

### Purpose

Process operation interfaces - capability-gated process spawning and management.

**Responsibilities:**
- Process spawning with arguments and environment
- Process status and wait operations
- Process signals (future)
- Environment variable access

**Dependencies:**
- `airssys:core-types@1.0.0` - Uses process-error
- `airssys:core-capabilities@1.0.0` - Uses process-permission

**Note:** Access requires declared `process-permission` in component manifest.

### Key Interfaces

#### Interface: `process`


**Imported Types:**
```wit
use airssys:core-types@1.0.0.{process-error};
use airssys:core-capabilities@1.0.0.{process-permission};
```

**Exported Types:**

```wit
// Process Spawn Configuration
record process-config {
    command: string,
    args: list<string>,
    env: list<tuple<string, string>>,
    working-dir: option<string>,
    timeout-ms: option<u64>,
}

// Process Handle
resource process-handle {
    // Resource type for managing spawned process
}

// Process Status
record process-status {
    exit-code: option<s32>,
    running: bool,
    stdout: list<u8>,
    stderr: list<u8>,
}

// Process Signal (future)
enum process-signal {
    term,
    kill,
    int,
    hup,
}
```

**Exported Functions:**

```wit
// Process Spawning
spawn-process: func(config: process-config) -> result<process-handle, process-error>;

// Process Management
wait-process: func(handle: process-handle, timeout-ms: u64) -> result<process-status, process-error>;
kill-process: func(handle: process-handle) -> result<_, process-error>;

// Future: Process Signals (commented out for Phase 1)
// send-signal: func(handle: process-handle, signal: process-signal) -> result<_, process-error>;

// Environment Access
get-environment-variable: func(name: string) -> option<string>;
```

**Permission Enforcement:** Host checks `process-permission` before executing operations.

### File Organization

```
wit/ext/process/
├── process.wit        # Process interface
└── deps.toml          # Dependencies on core-types and core-capabilities
```

---

## Cross-Package Type Usage

### Type Sharing Strategy

**Foundation Types (core-types):**
- Imported by ALL other packages
- Never duplicated
- Version pinned to @1.0.0

**Component Types (core-component):**
- Used by host runtime for component management
- Not imported by other core/ext packages (component implementation detail)

**Capability Types (core-capabilities):**
- Imported by ALL extension packages
- Used by host runtime for permission checks

**Host Service Types (core-host):**
- Used by components for host interaction
- Not imported by other packages (service interface, not types)

### Import Pattern Examples

**Extension Package Importing Core Types:**
```wit
// In ext/filesystem/filesystem.wit
package airssys:ext-filesystem@1.0.0;

use airssys:core-types@1.0.0.{file-error, timestamp};
use airssys:core-capabilities@1.0.0.{filesystem-permission};

interface filesystem {
    read-file: func(path: string) -> result<list<u8>, file-error>;
    stat: func(path: string) -> result<file-stat, file-error>;
}
```

**Core Package Importing Foundation:**
```wit
// In core/component/component.wit
package airssys:core-component@1.0.0;

use airssys:core-types@1.0.0.{
    component-id,
    component-error,
    execution-error,
    health-status
};

interface component-lifecycle {
    init: func(config: component-config) -> result<_, component-error>;
    health: func() -> health-status;
}
```

---

## World Definitions (Future Phase)

### Component World


```wit
// In core/component/component.wit (future)
world component {
    // Required exports - all components MUST implement
    export component-lifecycle;
    
    // Universal imports - SAME for all components
    import airssys:core-host/host-services.{host-services};
    
    // Optional imports - declared via component.toml permissions
    // import airssys:ext-filesystem/filesystem.{filesystem};
    // import airssys:ext-network/network.{network};
    // import airssys:ext-process/process.{process};
}
```

**Note:** World definitions deferred to Phase 3 (Build System Integration).

---

## Design Decisions Summary

### Type Organization

**Decision:** Common types in core-types, domain types in domain packages

**Rationale:**
- Avoid circular dependencies
- Clear ownership of types
- Easy to find type definitions

**Example:**
- `file-error` in core-types (used by multiple packages)
- `file-stat` in ext-filesystem (filesystem-specific)

### Function Signatures

**Decision:** Use `list<u8>` for data payloads (multicodec-encoded)

**Rationale:**
- Language-agnostic binary data
- Multicodec self-describing format
- No type coupling between components

**Example:**
```wit
execute: func(operation: list<u8>) -> result<list<u8>, execution-error>;
```

### Error Handling

**Decision:** Variant errors with string context

**Rationale:**
- Explicit error cases
- Contextual error messages
- WIT-idiomatic pattern

**Example:**
```wit
variant file-error {
    not-found(string),           // Path included in string
    permission-denied(string),   // Details included
    io-error(string),           // System error context
}
```

### Resource Handles (Advanced)

**Decision:** Use `resource` for managed handles (e.g., process-handle)

**Rationale:**
- Lifecycle management by host
- Prevents handle leaks
- WIT resource type pattern

**Example:**
```wit
resource process-handle {
    // Managed by host runtime
}
```

**Note:** Resource types implemented in Phase 3 when needed.

---

## Validation Against KNOWLEDGE-WASM-004

### Alignment Check

| KNOWLEDGE-WASM-004 Requirement | Implementation | Status |
|-------------------------------|----------------|--------|
| Universal component interface | `core-component` lifecycle | ✅ Complete |
| Host services interface | `core-host` services | ✅ Complete |
| Permission-based security | `core-capabilities` types | ✅ Complete |
| Multicodec data format | `list<u8>` payloads | ✅ Complete |
| Inter-component messaging | send-message, handle-message | ✅ Complete |
| Capability-gated extensions | filesystem, network, process | ✅ Complete |
| Component metadata | component-metadata record | ✅ Complete |

**Conclusion:** Design fully aligns with KNOWLEDGE-WASM-004 architecture.

---

## Implementation Priorities (Phase 2)

### Priority 1: Foundation (Day 4)

Implement in this order (topological dependency order):
1. `core-types` - No dependencies, foundation for all
2. `core-capabilities` - Depends only on core-types
3. `core-component` - Depends only on core-types
4. `core-host` - Depends on core-types and core-capabilities

### Priority 2: Extensions (Day 5)

Implement in parallel (all depend on same core packages):
1. `ext-filesystem` - Depends on core-types, core-capabilities
2. `ext-network` - Depends on core-types, core-capabilities
3. `ext-process` - Depends on core-types, core-capabilities

---

## Success Criteria

### Interface Completeness
- ✅ All 7 packages have interface specifications
- ✅ Core types cover common error cases
- ✅ Component lifecycle interface complete
- ✅ Host services interface complete
- ✅ Extension interfaces cover major capabilities

### Type Safety
- ✅ All error types have explicit variants
- ✅ Result types used for fallible operations
- ✅ Option types used for optional data
- ✅ No untyped strings for structured data

### Dependency Clarity
- ✅ Each package lists imported types explicitly
- ✅ Import sources clearly documented
- ✅ No circular dependencies in design
- ✅ Foundation types separated from domain types

### Documentation Quality
- ✅ Every interface has clear purpose statement
- ✅ Every type has usage explanation
- ✅ Examples provided for import patterns
- ✅ Rationale documented for design decisions

---

## References

### Source Documents
- **ADR-WASM-015**: WIT Package Structure Organization
- **KNOWLEDGE-WASM-004**: WIT Management Architecture
- **Task 1.1**: WIT specification constraints documentation
- **WASI Preview 2**: Reference examples for interface patterns

### Related Deliverables
- **structure_plan.md**: Directory and file organization
- **dependency_graph.md**: Cross-package dependency analysis (next)
- **import_patterns.md**: Import syntax examples (Hour 5)

---

**Document Version:** 1.0.0  
**Created:** 2025-10-25  
**Status:** Complete - Interface designs ready for Phase 2 implementation  
**Next Action:** Create `dependency_graph.md` with topological ordering
