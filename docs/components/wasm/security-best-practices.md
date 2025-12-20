# Security Best Practices for WASM Components

## Overview

This guide explains security principles and best practices for designing, implementing, and operating secure WASM components in AirsSys. It covers everything from capability design to preventing common attacks.

---

## Principle 1: Least Privilege

Every component should have exactly the minimum permissions needed for its function.

### The Principle

**Components should:**
- ✓ Access only the resources they need
- ✓ Have only the permissions they use
- ✓ Be unable to exceed their scope
- ✓ Fail safely if permissions are insufficient

**Components should NOT:**
- ✗ Request "all" permissions as fallback
- ✗ Ask for permissions they might need "someday"
- ✗ Have write access when read-only would work
- ✗ Access entire directory trees when specific files suffice

### Example: Read-Only Component

```toml
# ✓ CORRECT: Exactly what's needed
[[component.capabilities.filesystem]]
paths = ["/etc/myapp/config.toml"]
permissions = ["read"]
```

Why this is better:
- Component cannot accidentally modify configuration
- If compromised, attacker cannot change config
- Audit logs show no write attempts (any write is suspicious)

### Example: Write with Containment

```toml
# ❌ WRONG: Overly permissive
[[component.capabilities.filesystem]]
paths = ["/"]
permissions = ["read", "write"]

# ✓ CORRECT: Limited to specific directory
[[component.capabilities.filesystem]]
paths = ["/var/log/myapp/"]
permissions = ["write"]
```

Why this is better:
- Component cannot write outside log directory
- If compromised, damage is limited to one directory
- Filesystem quota prevents disk exhaustion in that directory

---

## Principle 2: Deny-by-Default

The security system uses deny-by-default: components can only access resources they explicitly declare.

### Understanding Deny-by-Default

```
Component requests access to /app/data/file.json
           ↓
Check: Is /app/data/file.json in declared capabilities?
           ├─ YES → Check permission
           │        Is operation in declared permissions?
           │        ├─ YES → ALLOW
           │        └─ NO → DENY
           │
           └─ NO → DENY (not in any declared capability)
```

### Default Denials

Capabilities that are NOT granted:

```toml
# This component declaration:
[[component.capabilities.filesystem]]
paths = ["/app/config/*"]
permissions = ["read"]

# Implicitly denies:
# - ✗ Writing to /app/config/*
# - ✗ Reading from /app/data/*
# - ✗ Accessing /etc/*
# - ✗ Network access
# - ✗ Storage access
```

### Safe-by-Default Error Handling

When a component encounters a denial:

```rust
// Host function must handle denials gracefully
fn host_read(component_id: &str, path: &str) -> Result<Vec<u8>> {
    match check_capability(component_id, path, "read") {
        Ok(_) => Ok(std::fs::read(path)?),
        Err(CapabilityDeniedError { reason }) => {
            // Log attempt and return error to component
            log_security_event("access_denied", component_id, path, &reason);
            Err(format!("Access denied to {}: {}", path, reason))
        }
    }
}
```

---

## Principle 3: Capability Pattern Design

Patterns control which resources a component can access. Design patterns carefully to be precise yet practical.

### Pattern Types & Trade-offs

**Exact Paths** - Maximum safety, least flexibility
```toml
paths = ["/app/config.json"]  # Only this exact file
```

**Glob Patterns** - Good balance
```toml
paths = ["/app/config/*.json"]  # All .json files in config/
```

**Recursive Wildcard** - More flexibility, greater risk
```toml
paths = ["/app/data/**/*"]  # Everything under data/
```

### Safe Pattern Design

#### ✓ Confine to Application Directory

```toml
# ✓ CORRECT: Limited to app directory
paths = ["/app/data/*"]

# ❌ WRONG: Starts at root
paths = ["/*"]
```

#### ✓ Use File Extensions for Typing

```toml
# ✓ CORRECT: Only logs
paths = ["/var/log/myapp/*.log"]

# ❌ WRONG: All files
paths = ["/var/log/myapp/*"]
```

#### ✓ Avoid Overly Broad Wildcards

```toml
# ❌ WRONG: Could match sensitive files
paths = ["/app/**"]  # Everything under /app

# ✓ CORRECT: Specific file types
paths = ["/app/**/*.json", "/app/**/*.log"]
```

#### ✓ Namespace Isolation in Multi-Tenant

```toml
# ✓ CORRECT: Each tenant isolated
[[component.capabilities.storage]]
namespaces = ["tenant-123/*"]

# ❌ WRONG: Access to all tenants
[[component.capabilities.storage]]
namespaces = ["**"]
```

### Pattern Validation Rules

The security system enforces:

| Rule | Example | Status |
|------|---------|--------|
| No root access | `paths = ["/"]` | ✗ REJECTED |
| No empty patterns | `paths = [""]` | ✗ REJECTED |
| Max depth 5 levels | `paths = ["/a/b/c/d/e/f/**"]` | ⚠️ WARNING |
| Confined to path | `paths = ["/*"]` when data in `/data` | ✗ DENIED at runtime |

---

## Principle 4: Trust Level Selection

Different trust levels suit different scenarios. Choose appropriately.

### Trusted Components

**Use when:**
- Component is from your organization
- Code review is mandatory (enforced)
- Component signed with authorized key
- Component in your development workspace

**Security posture:** High (preapproved, instant installation)

```toml
[trust]
[[trust.sources]]
type = "git"
url_pattern = "https://github.com/myorg/*"
description = "Internal org repositories"
```

### Unknown Components

**Use when:**
- Component from third-party source
- No prior relationship with vendor
- First time using this component
- Requires security review

**Security posture:** Balanced (reviewed before deployment)

**Workflow:**
1. Request component installation
2. Security team reviews Component.toml
3. Team approves/rejects
4. Component installed (if approved)

### DevMode

**Use when:**
- Local development only
- Rapid iteration required
- Testing new component locally
- CI/CD pipeline development

**Security posture:** Low (security checks bypassed)

⚠️ **CRITICAL RULE:** Never use DevMode in production or staging environments.

---

## Attack Prevention Techniques

### Attack 1: Path Traversal

**Attack Goal:** Component with `/app/data/*` access tries to read `/etc/passwd` via `../../../etc/passwd`

**Prevention Layer 1 - Host Function**
```rust
// Host functions MUST normalize paths
fn host_read(component_id: &str, path: &str) -> Result<Vec<u8>> {
    // Canonicalize resolves .. and .
    let normalized = std::fs::canonicalize(path)?;
    
    // Check against normalized path
    check_capability(component_id, &normalized, "read")?;
    
    // Read from normalized path
    std::fs::read(&normalized)
}
```

**Prevention Layer 2 - Pattern Matching**
```
Pattern: /app/data/*
Request: /app/data/../../../etc/passwd
After normalization: /etc/passwd
Match result: NO MATCH → DENIED
```

**Prevention Layer 3 - Audit Trail**
```
Audit log shows all traversal attempts, even failed ones:
2025-12-20T15:30:00Z | component | capability_check | /etc/passwd | read | denied | Pattern mismatch
```

### Attack 2: Privilege Escalation

**Attack Goal:** Component with read permission tries to write, escalating privileges

**Prevention:** Permission checking is strict

```
Component declared: permissions = ["read"]
Attempt: write /app/data/file
Check: is "write" in ["read"]?
Result: NO → DENIED
Audit: Access denied - write not in [read]
```

**Code Review Checklist:**
- [ ] Verify declared permissions match actual usage
- [ ] No code attempts to write with read-only permissions
- [ ] Error handling expects permission denials
- [ ] No workarounds attempting to escalate

### Attack 3: Quota Exhaustion

**Attack Goal:** Component rapidly exhausts disk/network quotas

**Prevention Layer 1 - Quota Tracking**
```rust
// Every operation is metered
let result = quota_tracker.can_allocate(cost);
match result {
    Ok(_) => { quota_tracker.update(cost); },
    Err(QuotaExceeded) => return Err("Quota exceeded"),
}
```

**Prevention Layer 2 - Supervisor Termination**
```
Component exceeds quota
           ↓
Supervisor detects violation
           ↓
Send error to component
           ↓
If repeated: Terminate component actor
```

**Prevention Layer 3 - Rate Limiting**
```toml
[quotas]
storage_gb = 5                    # Max 5GB
max_files = 1000                  # Max 1000 files
network_connections_per_second = 100
```

### Attack 4: Wildcard Exploitation

**Attack Goal:** Component with `**` wildcard tries to access entire filesystem

**Prevention: Pattern Validation**

```
Pattern: /**
Validation: Root wildcard detected
Result: REJECTED
```

```
Pattern: /app/**
Validation: Confined to /app → OK
Result: ALLOWED
```

### Attack 5: Trust Spoofing

**Attack Goal:** Unknown component claims to be from trusted source

**Prevention: Source Verification**

```
Component metadata: git_url = "https://github.com/trusted-org/foo"
Claimed trust source: matches internal org pattern
Actual source verification: Check Git URL in metadata
Result: If URL doesn't match, TrustLevel::Unknown
```

---

## Testing Security Configuration

### Test 1: Positive Case - Allowed Access

```bash
# Component should succeed accessing declared resource
$ component read /app/config/settings.json
✓ Success: [file contents]
```

### Test 2: Negative Case - Denied Access

```bash
# Component should fail accessing non-declared resource
$ component read /etc/passwd
✗ Error: Permission denied: /etc/passwd not in declared capabilities
```

### Test 3: Permission Mismatch

```bash
# Component with read permission should not write
$ component write /app/data/file.txt "new content"
✗ Error: Permission denied: write not in [read]
```

### Test 4: Quota Enforcement

```bash
# Component should hit quota limit
$ component write /storage/large-file.bin (900MB)
✓ Success (total: 900MB/1GB)

$ component write /storage/another.bin (200MB)
✗ Error: Quota exceeded: would exceed 1GB limit
```

### Test 5: Audit Logging

```bash
# All attempts should be logged
$ grep "component-id" audit.log | tail -20
2025-12-20T15:30:00Z | component-id | /app/config/settings.json | read | granted
2025-12-20T15:30:01Z | component-id | /etc/passwd | read | denied | Pattern mismatch
2025-12-20T15:30:02Z | component-id | /app/data/file | write | denied | write not in [read]
```

---

## Code Review Checklist

**Every component with security-sensitive operations should be reviewed against:**

- [ ] **Principle of Least Privilege**
  - [ ] Component requests only needed permissions
  - [ ] No overly broad wildcards (e.g., `/**`)
  - [ ] Read-only when write not needed

- [ ] **Capability Declaration**
  - [ ] All resource access declared in Component.toml
  - [ ] Paths are specific and confined
  - [ ] Permissions match actual usage

- [ ] **Path Handling**
  - [ ] Host functions normalize paths
  - [ ] No path concatenation without validation
  - [ ] Traversal attempts return errors

- [ ] **Error Handling**
  - [ ] Component handles permission denials gracefully
  - [ ] No hardcoded fallback paths
  - [ ] Errors logged appropriately

- [ ] **Quota Management**
  - [ ] Quotas configured for storage access
  - [ ] Component monitors quota usage
  - [ ] Graceful handling when quota exceeded

- [ ] **Network Access**
  - [ ] Only required endpoints declared
  - [ ] Domains whitelisted, not blacklisted
  - [ ] TLS validation enforced

- [ ] **Secret Management**
  - [ ] No credentials hardcoded
  - [ ] Secrets loaded from secure storage
  - [ ] API keys not logged or audited

---

## Performance Considerations

### Capability Check Overhead

```
Per-operation overhead: ~3-4μs (lock-free DashMap + ACL eval)
Budget per 1ms: ~250-300 checks possible

This is negligible for most operations (file I/O is µs-ms timescale)
```

### Optimization Guidelines

#### ✓ Use Exact Paths When Possible

```toml
# Faster: Exact match is O(1)
paths = ["/app/config/settings.json"]

# Slower: Glob requires pattern matching
paths = ["/app/config/*.json"]
```

#### ✓ Group Related Capabilities

```toml
# ✓ Efficient: One entry with multiple paths
[[component.capabilities.filesystem]]
paths = ["/app/config/*", "/app/data/*"]

# Less efficient: Multiple entries (more lookups)
[[component.capabilities.filesystem]]
paths = ["/app/config/*"]
[[component.capabilities.filesystem]]
paths = ["/app/data/*"]
```

#### ✓ Cache-Friendly Patterns

```toml
# Component contexts are cached in DashMap
# Repeated checks for same component are fast (~1μs)
# First check: 3-4μs, subsequent: 1-2μs
```

---

## Common Mistakes to Avoid

### ❌ Mistake 1: Root-Level Access

```toml
# DANGEROUS: Can access entire system
[[component.capabilities.filesystem]]
paths = ["/"]
permissions = ["read", "write"]
```

**Fix:** Confine to application directory
```toml
[[component.capabilities.filesystem]]
paths = ["/app/*"]
permissions = ["read"]
```

### ❌ Mistake 2: Overly Broad Networks

```toml
# DANGEROUS: Can connect anywhere
[[component.capabilities.network]]
endpoints = ["*"]
permissions = ["connect"]
```

**Fix:** Whitelist specific endpoints
```toml
[[component.capabilities.network]]
endpoints = ["https://api.example.com/v1/*"]
permissions = ["connect"]
```

### ❌ Mistake 3: Write Permission for Reads

```toml
# WRONG: More permission than needed
[[component.capabilities.filesystem]]
paths = ["/app/config/*"]
permissions = ["read", "write"]  # Needs only read!
```

**Fix:** Only required permission
```toml
[[component.capabilities.filesystem]]
paths = ["/app/config/*"]
permissions = ["read"]
```

### ❌ Mistake 4: No Quota Limits

```toml
# WRONG: Unbounded storage growth
[[component.capabilities.storage]]
namespaces = ["cache/*"]
permissions = ["read", "write"]
# Missing quota_gb!
```

**Fix:** Set reasonable quota
```toml
[[component.capabilities.storage]]
namespaces = ["cache/*"]
permissions = ["read", "write"]
quota_gb = 2  # Prevent unbounded growth
```

### ❌ Mistake 5: Production DevMode

```toml
# CRITICAL: Never do this!
[trust]
dev_mode = true  # In production config!
```

**Fix:** DevMode only in development config
```toml
# development.toml
[trust]
dev_mode = true

# production.toml
[trust]
dev_mode = false
```

---

## Auditing Best Practices

### ✓ Monitor Audit Logs

```bash
# Daily review of denials
$ grep "denied" audit.log | tail -50

# Find attempted policy bypasses
$ grep "traversal\|privilege\|overflow" audit.log

# Monitor quota violations
$ grep "quota_exceeded" audit.log
```

### ✓ Alert on Suspicious Patterns

```
Alert conditions:
- High number of denials from one component (possible misconfiguration or attack)
- Attempts to access /etc, /root, /home (suspicious)
- Rapid quota increases (possible resource exhaustion)
- Multiple components accessing unusual paths (possible coordinated attack)
```

### ✓ Regular Security Reviews

```
Quarterly Reviews:
- [ ] Audit all denial logs (last 90 days)
- [ ] Verify no over-privileged components
- [ ] Review trust configuration changes
- [ ] Check quota limit appropriateness
- [ ] Update capability patterns as needed
```

---

## References

- **Architecture**: [security-architecture.md](security-architecture.md)
- **Capability Declaration**: [capability-declaration-guide.md](capability-declaration-guide.md)
- **Trust Configuration**: [trust-configuration-guide.md](trust-configuration-guide.md)
- **Host Integration**: [host-integration-guide.md](host-integration-guide.md)
- **Troubleshooting**: [troubleshooting-security.md](troubleshooting-security.md)
- **Examples**: [examples/](examples/)
