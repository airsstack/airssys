# WASM-OSL Security Architecture

## Overview

The WASM-OSL security architecture implements **capability-based access control** across four integrated layers, from WASM component capabilities through airssys-osl enforcement to actor-level isolation.

This document explains how the security system works, from capability declaration through enforcement and audit logging.

---

## Four-Layer Security Model

AirsSys security operates across four complementary layers:

```
┌─────────────────────────────────────────────────────────────┐
│ Layer 1: WASM Capabilities                                   │
│ - WasmCapability enum (Filesystem, Network, Storage, Custom) │
│ - Declared in Component.toml                                │
│ - Pattern-based resource matching                           │
├─────────────────────────────────────────────────────────────┤
│ Layer 2: WASM Security Context & Audit                      │
│ - WasmSecurityContext encapsulates component capabilities   │
│ - WasmAuditLogger tracks all access decisions               │
│ - Thread-safe via DashMap                                   │
├─────────────────────────────────────────────────────────────┤
│ Layer 3: airssys-osl ACL/RBAC Enforcement                   │
│ - SecurityPolicy evaluates WasmCapability → AclEntry        │
│ - Permission checking via SecurityMiddleware                │
│ - <5μs performance via optimized ACL evaluation             │
├─────────────────────────────────────────────────────────────┤
│ Layer 4: Actor Supervision & Isolation                      │
│ - Each component runs in isolated actor context             │
│ - Supervisor enforces per-actor resource quotas             │
│ - Runtime monitoring and termination on violations          │
└─────────────────────────────────────────────────────────────┘
```

### Layer 1: WASM Capabilities

**Components declare their resource needs in Component.toml:**

```toml
[[component.capabilities.filesystem]]
paths = ["/app/data/*.json"]
permissions = ["read", "write"]

[[component.capabilities.network]]
endpoints = ["https://api.example.com/*"]
permissions = ["connect"]
```

**WasmCapability Types** (enum):

```rust
pub enum WasmCapability {
    // Filesystem access with pattern matching
    Filesystem {
        paths: Vec<String>,
        permissions: Vec<String>,
    },
    
    // Network access with endpoint whitelisting
    Network {
        endpoints: Vec<String>,
        permissions: Vec<String>,
    },
    
    // Storage access with namespace isolation
    Storage {
        namespaces: Vec<String>,
        permissions: Vec<String>,
        quota_gb: Option<u32>,
    },
    
    // Application-specific capabilities
    Custom {
        resource_type: String,
        actions: Vec<String>,
        metadata: Map<String, Value>,
    }
}
```

### Layer 2: WASM Security Context & Audit

**WasmSecurityContext** manages component security state:

```rust
pub struct WasmSecurityContext {
    component_id: String,
    capabilities: WasmCapabilitySet,
    trust_level: TrustLevel,
    created_at: DateTime<Utc>,
    audit_logger: Arc<WasmAuditLogger>,
}
```

**WasmCapabilitySet** - Efficient capability container:

```rust
pub struct WasmCapabilitySet {
    filesystem: Vec<WasmCapability::Filesystem>,
    network: Vec<WasmCapability::Network>,
    storage: Vec<WasmCapability::Storage>,
    custom: Vec<WasmCapability::Custom>,
}

impl WasmCapabilitySet {
    pub fn grant(&mut self, capability: WasmCapability) -> Self
    pub fn has_filesystem_access(&self, path: &str, perm: &str) -> bool
    pub fn has_network_access(&self, endpoint: &str, perm: &str) -> bool
}
```

**WasmAuditLogger** - Thread-safe audit logging:

```rust
pub struct WasmAuditLogger {
    logger: Arc<SecurityAuditLogger>,
}

impl WasmAuditLogger {
    pub fn log_capability_check(&self, check: &CapabilityCheckResult)
    pub fn log_access_granted(&self, resource: &str, permission: &str)
    pub fn log_access_denied(&self, resource: &str, reason: &str)
}
```

### Layer 3: airssys-osl ACL/RBAC Enforcement

**Mapping WasmCapability → ACL**

The CapabilityChecker converts WasmCapabilities to airssys-osl AclEntries:

```rust
fn wasm_capability_to_acl(capability: &WasmCapability) -> Vec<AclEntry> {
    match capability {
        WasmCapability::Filesystem { paths, permissions } => {
            paths.iter().map(|path| AclEntry {
                resource: format!("filesystem:{}", path),
                permissions: permissions.clone(),
                effect: Allow,
            }).collect()
        }
        WasmCapability::Network { endpoints, permissions } => {
            endpoints.iter().map(|endpoint| AclEntry {
                resource: format!("network:{}", endpoint),
                permissions: permissions.clone(),
                effect: Allow,
            }).collect()
        }
        // Similar for Storage and Custom...
    }
}
```

**Capability Check Flow:**

```
Host Function Called
  e.g., filesystem_read("/app/data/file.json")
             ↓
Extract component_id, resource, permission
             ↓
CapabilityChecker::check(component_id, resource, perm)
             ↓
Lookup WasmSecurityContext from DashMap
             ↓
Pattern match: does component capability match resource?
  - Exact: /app/config.json = /app/config.json ✓
  - Glob: /app/*.json matches /app/config.json ✓
  - Recursive: /app/**/*.log matches /app/2024/12/app.log ✓
             ↓
Permission check: is perm in declared permissions?
  - declared: [read, write]
  - requested: read ✓
  - requested: execute ✗
             ↓
Delegate to airssys-osl SecurityPolicy
             ↓
SecurityPolicy::evaluate(acl_entry, operation)
             ↓
PolicyDecision::Allow or Deny
             ↓
Log result via WasmAuditLogger
             ↓
Return to host function
```

### Layer 4: Actor Supervision & Isolation

**Per-Component Resource Quotas**

Each component runs as supervised actor with enforced quotas:

```rust
pub struct ComponentActorQuota {
    component_id: String,
    filesystem_quota: QuotaTracker,  // Max files, bandwidth
    network_quota: QuotaTracker,     // Max connections, bandwidth
    storage_quota: QuotaTracker,     // Max GB per namespace
    cpu_quota: QuotaTracker,         // Max CPU time
}
```

**Quota Enforcement in Supervisor:**

```
Actor processes message
           ↓
Check resource quotas
  - filesystem_quota.can_allocate(cost)
  - network_quota.can_allocate(cost)
  - storage_quota.can_allocate(cost)
           ↓
        Quota OK?
    ├─ YES → Process message
    │        Update quota tracking
    │
    └─ NO → Send QuotaExceeded error
            Log violation
            Consider terminating actor if repeated
```

---

## WasmCapability → AclEntry Mapping

### Filesystem Mapping

**WasmCapability:**
```rust
WasmCapability::Filesystem {
    paths: vec!["/app/data/*.json"],
    permissions: vec!["read", "write"],
}
```

**→ AclEntry:**
```rust
AclEntry {
    resource: "filesystem:/app/data/*.json",
    permissions: vec!["read", "write"],
    effect: Allow,
}
```

**Pattern Evaluation:**
```
Pattern: /app/data/*.json
Request: /app/data/config.json
  1. Split pattern: ["app", "data", "*.json"]
  2. Split request: ["app", "data", "config.json"]
  3. Match each component:
     - "app" = "app" ✓
     - "data" = "data" ✓
     - "*.json" matches "config.json" ✓
  4. Result: ALLOW
```

### Network Mapping

**WasmCapability:**
```rust
WasmCapability::Network {
    endpoints: vec!["https://api.example.com/v1/*"],
    permissions: vec!["connect"],
}
```

**→ AclEntry:**
```rust
AclEntry {
    resource: "network:https://api.example.com/v1/*",
    permissions: vec!["connect"],
    effect: Allow,
}
```

**Endpoint Matching:**
```
Pattern: https://api.example.com/v1/*
Request: https://api.example.com/v1/users
  1. Parse URLs
  2. Compare scheme: https = https ✓
  3. Compare host: api.example.com = api.example.com ✓
  4. Pattern match path: /v1/* matches /v1/users ✓
  5. Result: ALLOW
```

### Storage Mapping

**WasmCapability:**
```rust
WasmCapability::Storage {
    namespaces: vec!["tenant-*/data/*"],
    permissions: vec!["read", "write"],
    quota_gb: Some(5),
}
```

**→ AclEntry:**
```rust
AclEntry {
    resource: "storage:tenant-*/data/*",
    permissions: vec!["read", "write"],
    effect: Allow,
}
```

Plus quota enforcement:
```rust
QuotaEntry {
    namespace: "tenant-*/data/*",
    limit_gb: 5,
    current_usage: 0,
}
```

---

## Security Context Lifecycle

### 1. Component Initialization

```
Component manifest loaded
           ↓
Parse Component.toml
  - Extract capabilities
  - Extract trust metadata
           ↓
Create WasmCapabilitySet
           ↓
Create WasmSecurityContext
  {
    component_id: "my-component",
    capabilities: WasmCapabilitySet { ... },
    trust_level: TrustLevel::Trusted,
    created_at: 2025-12-20T15:30:00Z,
    audit_logger: Arc::new(WasmAuditLogger::new()),
  }
           ↓
Register with CapabilityChecker
  checker.register_component(ctx)?
           ↓
Component ready for execution
```

### 2. Runtime Capability Checking

```
Host function invoked
           ↓
Extract operation parameters
  - component_id
  - resource (path, endpoint, namespace, etc.)
  - permission (read, write, connect, etc.)
           ↓
Invoke CapabilityChecker::check()
           ↓
Lookup WasmSecurityContext
           ↓
Evaluate capability match
           ↓
Audit log the decision
           ↓
Return Allow or Deny
           ↓
Host function proceeds or returns error
```

### 3. Component Cleanup

```
Component instance shutting down
           ↓
Final audit log entries flushed
           ↓
Unregister from CapabilityChecker
  checker.unregister_component(component_id)?
           ↓
Release WasmSecurityContext
           ↓
Cleanup complete
```

---

## Audit Logging Integration

### Audit Log Entries

Every capability check is logged with structured data:

```rust
pub struct WasmCapabilityAuditLog {
    timestamp: DateTime<Utc>,
    component_id: String,
    operation: String,      // "capability_check"
    resource: String,       // path, endpoint, namespace
    permission: String,     // read, write, connect, etc.
    result: String,         // "granted" or "denied"
    reason: Option<String>, // if denied, why?
}
```

### Example Audit Trail

```
2025-12-20T15:30:45.123Z | my-component | capability_check | /app/data/file.json | read | granted | -
2025-12-20T15:30:46.456Z | my-component | capability_check | /etc/passwd | read | denied | Outside pattern /app/data/*
2025-12-20T15:30:47.789Z | my-component | capability_check | /app/data/config.json | write | granted | -
```

### Audit Log Analysis

Audit logs enable security monitoring:

```bash
# Find all denials for a component
grep "my-component.*denied" audit.log

# Find attempts to access sensitive paths
grep "^.*| denied | .*(/etc|/root|/home)" audit.log

# Find permission mismatches
grep "denied.*Permission.*not in" audit.log
```

---

## Attack Vectors & Mitigations

### Vector 1: Path Traversal

**Attack:** Component with `/app/data/*` tries to access `/etc/passwd` via `../../../etc/passwd`

**Mitigation:**
1. Host function MUST normalize paths before passing to CapabilityChecker
2. Patterns match post-normalization only
3. Audit logging captures traversal attempts

**Implementation:**
```rust
// Host function MUST do this:
fn filesystem_read(component_id: &str, path: &str) -> Result<Vec<u8>> {
    // CRITICAL: Normalize path (resolve .., .)
    let normalized = normalize_path(path)?;
    
    // Check capability against normalized path
    check_capability(component_id, &normalized, "read")?;
    
    // Now safe to read
    std::fs::read(&normalized)
}
```

### Vector 2: Privilege Escalation

**Attack:** Component with read permission tries to write

**Mitigation:**
1. Permission check strictly enforces declared permissions
2. Write attempt returns error if not in permission list
3. All attempts audited

**Example:**
```
Component declared: permissions = ["read"]
Attempt: write to file
Result: Denied - "write" not in [read]
Audit: Access denied - permission not in declared permissions
```

### Vector 3: Quota Exhaustion

**Attack:** Component rapidly exceeds resource quotas

**Mitigation:**
1. QuotaTracker maintains per-component usage
2. Each operation checks quota before proceeding
3. Component terminated if quota exceeded
4. Quota violations logged

**Enforcement:**
```rust
if !quota_tracker.can_allocate(cost) {
    return Err(QuotaExceededError {
        current: quota_tracker.current(),
        limit: quota_tracker.limit(),
    });
}
```

### Vector 4: Pattern Vulnerabilities

**Attack:** Component uses wildcard pattern to match system-wide paths

**Mitigation:**
1. Patterns are statically validated at declaration time
2. Invalid patterns rejected (empty, too broad)
3. Runtime matching uses safe glob implementation

**Examples:**
```
✗ REJECTED: "**" (matches entire filesystem)
✗ REJECTED: "/" (root directory)
✓ ALLOWED: "/app/**" (confined to /app)
✓ ALLOWED: "/app/data/*.json" (specific extension)
```

### Vector 5: Trust Bypass

**Attack:** Unknown component claims Trusted status

**Mitigation:**
1. Trust determination is done by TrustRegistry
2. Component manifest used to determine source
3. Claimed source verified against trust configuration
4. Components from unknown sources require approval

**Flow:**
```
Component manifest includes: git_url = "https://github.com/example/foo"
Trust configuration includes: url_pattern = "https://github.com/other/*"
Result: MISMATCH → TrustLevel::Unknown
Outcome: Requires manual approval
```

---

## Performance Characteristics

### Capability Check Performance

**Target:** <5μs per check

**Breakdown:**
- DashMap lookup: ~500ns (lock-free concurrent hash map)
- Pattern matching: ~1-2μs (optimized glob engine)
- airssys-osl ACL evaluation: ~1-2μs (cached security policy)
- Audit logging: ~200ns (async, non-blocking)

**Total: ~3-4μs typical case**

### Fast Path Optimization

Components with no capabilities (read-only, no network):

```
Component has no capabilities
           ↓
Pattern match immediately fails
           ↓
Return Deny without ACL evaluation
           ↓
<1μs fast path
```

### Cache Behavior

```
First check (cache miss):
  - DashMap lookup: miss → create entry
  - Full evaluation: 3-4μs

Subsequent checks (cache hit):
  - DashMap lookup: hit → cached context
  - Evaluation: 2-3μs (reduced lock contention)
```

---

## Integration with airssys-rt

Components run as actors with supervised lifecycle:

```
Actor receives message
           ↓
CapabilityChecker::check() called for resource access
           ↓
Decision made (allow/deny)
           ↓
Audit logged via WasmAuditLogger
           ↓
If denied: Send error to actor
If allowed: Proceed with operation
           ↓
Quota updated
           ↓
Supervisor monitors quota usage
           ↓
If exceeded: Send termination signal to actor
```

---

## References

- **ADR-WASM-005**: Capability-Based Security Model
- **ADR-WASM-006**: Component Isolation and Sandboxing
- **Capability Declaration**: [capability-declaration-guide.md](capability-declaration-guide.md)
- **Trust Configuration**: [trust-configuration-guide.md](trust-configuration-guide.md)
- **Best Practices**: [security-best-practices.md](security-best-practices.md)
- **Host Integration**: [host-integration-guide.md](host-integration-guide.md)
