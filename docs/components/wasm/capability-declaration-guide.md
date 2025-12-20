# How to Declare Component Capabilities in Component.toml

## Overview

**Capabilities** are the foundation of WASM component security in AirsSys. They describe what resources your component needs to access and with what permissions. By explicitly declaring capabilities upfront, you enable the security system to enforce strict access controls and audit component behavior.

This guide shows you how to declare capabilities for your components, from basic file access to complex multi-resource scenarios. All examples are based on real patterns from the AirsSys security test suite.

### Why Declare Capabilities?

1. **Security**: Components can only access resources they explicitly declare
2. **Transparency**: All component requirements are visible in Component.toml
3. **Auditability**: Access attempts are logged against declared capabilities
4. **Isolation**: Prevents accidental or malicious cross-component access

---

## Capability Types

AirsSys supports four capability types, each controlling access to different resource categories.

### Filesystem Capabilities

Grant access to filesystem resources with pattern matching and permission control.

**Syntax:**
```toml
[[component.capabilities.filesystem]]
paths = [
    "/app/config/*.json",      # Glob pattern
    "/app/data/**/*.log",      # Recursive wildcard
    "/var/cache/exact-file"    # Exact path
]
permissions = ["read", "write"]
```

**Permission Types:**
- `read`: Read file contents
- `write`: Write/modify file contents
- `execute`: Execute file (Unix permissions)

**Pattern Syntax:**

| Pattern | Matches | Example |
|---------|---------|---------|
| Exact path | Only exact file | `/app/config.json` → `/app/config.json` ✓ |
| Glob `*` | Files in directory | `/app/*.json` → `/app/config.json`, `/app/settings.json` ✓ |
| Glob `*.ext` | Files by extension | `/app/*.log` → `/app/app.log`, `/var/error.log` ✗ |
| Recursive `**` | Nested directories | `/app/**/*.log` → `/app/logs/app.log`, `/app/2024/12/error.log` ✓ |

**Common Mistakes:**

```toml
# ❌ WRONG: Glob doesn't cross directory boundaries
paths = ["/app/*/file.txt"]  # Matches /app/data/file.txt but NOT /app/data/subdir/file.txt

# ✅ CORRECT: Use recursive wildcard for nested access
paths = ["/app/**/file.txt"]  # Matches /app/data/subdir/file.txt
```

### Network Capabilities

Grant access to remote endpoints with domain whitelisting.

**Syntax:**
```toml
[[component.capabilities.network]]
endpoints = [
    "https://api.example.com/*",          # Domain with glob
    "https://data.example.com:8443/v1/*"  # Specific port
]
permissions = ["connect", "bind"]
```

**Permission Types:**
- `connect`: Initiate outbound connections to endpoints
- `bind`: Listen for inbound connections on specified ports

**Domain Matching:**

| Pattern | Matches |
|---------|---------|
| `https://api.example.com/*` | `https://api.example.com/users` ✓, `https://api.example.com/v1/data` ✓ |
| `https://api.example.com/v1/*` | `https://api.example.com/v1/users` ✓, `https://api.example.com/v2/users` ✗ |
| Exact domain | `https://api.example.com` | Only exact match |

### Storage Capabilities

Grant access to isolated namespaced storage with multi-tenancy support.

**Syntax:**
```toml
[[component.capabilities.storage]]
namespaces = [
    "tenant-123/*",           # Namespace with glob
    "shared/component-data"   # Shared namespace
]
permissions = ["read", "write"]
quota_gb = 1                  # Optional: limit to 1 GB
```

**Permission Types:**
- `read`: Read stored data
- `write`: Write/modify stored data

### Custom Capabilities

Define application-specific capabilities for custom resource types.

**Syntax:**
```toml
[[component.capabilities.custom]]
resource_type = "payment-processor"
actions = ["process", "refund", "audit"]
metadata = { region = "us-east-1", environment = "sandbox" }
```

---

## Complete Examples

### Example 1: Simple Config Reader

A component that only needs to read application configuration files.

```toml
[component]
name = "config-reader"
version = "1.0.0"

[[component.capabilities.filesystem]]
paths = [
    "/etc/myapp/config.toml",
    "/etc/myapp/settings/*.json"
]
permissions = ["read"]
```

**What this allows:**
- ✓ Read `/etc/myapp/config.toml`
- ✓ Read `/etc/myapp/settings/app.json`
- ✓ Read `/etc/myapp/settings/database.json`
- ✗ Write to any file
- ✗ Read `/etc/passwd`

---

### Example 2: Log File Writer

A component that writes application logs with recursive directory access.

```toml
[component]
name = "log-writer"
version = "1.0.0"

[[component.capabilities.filesystem]]
paths = ["/var/log/myapp/**/*.log"]
permissions = ["read", "write"]
```

**What this allows:**
- ✓ Write to `/var/log/myapp/app.log`
- ✓ Read from `/var/log/myapp/archive/2024-12.log`
- ✓ Write to `/var/log/myapp/2024/12/error.log`
- ✗ Access `/var/log/other-app/*`

---

### Example 3: API Client

A network-enabled component that communicates with specific services.

```toml
[component]
name = "api-client"
version = "1.0.0"

[[component.capabilities.network]]
endpoints = [
    "https://api.example.com/v1/*",
    "https://api.example.com/v2/*",
    "https://auth.example.com/oauth/*"
]
permissions = ["connect"]

[[component.capabilities.filesystem]]
paths = ["/app/cache/api/*"]
permissions = ["read", "write"]
```

**What this allows:**
- ✓ Connect to `https://api.example.com/v1/users`
- ✓ Connect to `https://auth.example.com/oauth/token`
- ✓ Cache responses in `/app/cache/api/`
- ✗ Connect to `https://other-service.com`
- ✗ Write outside cache directory

---

### Example 4: Multi-Tenant Storage Manager

A component managing isolated storage for multiple tenants.

```toml
[component]
name = "storage-manager"
version = "1.0.0"

[[component.capabilities.storage]]
namespaces = [
    "tenant-*/data/*",      # All tenants' data
    "shared/metadata"       # Shared metadata
]
permissions = ["read", "write"]
quota_gb = 10               # 10 GB total quota
```

**What this allows:**
- ✓ Read/write `/tenant-123/data/file`
- ✓ Read/write `/tenant-456/data/config`
- ✓ Read/write `/shared/metadata`
- ✗ Exceed 10 GB total storage
- ✗ Access `/tenant-789/private` (requires explicit pattern)

---

### Example 5: Database Adapter

A component with both filesystem and storage access.

```toml
[component]
name = "database-adapter"
version = "1.0.0"

[[component.capabilities.filesystem]]
paths = [
    "/var/lib/mydb/data/*.db",
    "/var/lib/mydb/wal/**/*.wal"
]
permissions = ["read", "write"]

[[component.capabilities.storage]]
namespaces = ["cache/*", "indexes/*"]
permissions = ["read", "write"]
quota_gb = 5
```

**What this allows:**
- ✓ Read/write database files in `/var/lib/mydb/`
- ✓ Read/write WAL files in nested directories
- ✓ Cache data and indexes in storage
- ✗ Access files outside declared paths
- ✗ Exceed 5 GB in storage

---

### Example 6: Restricted Service Client

A component with minimal, principle-of-least-privilege configuration.

```toml
[component]
name = "restricted-client"
version = "1.0.0"

[[component.capabilities.network]]
endpoints = ["https://api.example.com/health"]
permissions = ["connect"]
```

**What this allows:**
- ✓ Check service health at `https://api.example.com/health`
- ✗ Access any other endpoints
- ✗ Access filesystem
- ✗ Access storage

---

### Example 7: Background Job Processor

A component with time-based scheduling and event logging.

```toml
[component]
name = "job-processor"
version = "1.0.0"

[[component.capabilities.filesystem]]
paths = ["/var/log/jobs/**/*"]
permissions = ["write"]

[[component.capabilities.storage]]
namespaces = ["job-queue/*", "job-results/*"]
permissions = ["read", "write"]
quota_gb = 2

[[component.capabilities.network]]
endpoints = ["https://webhook.example.com/job-complete"]
permissions = ["connect"]
```

**What this allows:**
- ✓ Log job execution to `/var/log/jobs/`
- ✓ Read/write job queue and results
- ✓ Notify webhook on completion
- ✗ Modify files it hasn't created
- ✗ Access other services

---

### Example 8: Data Processor with Multiple Capabilities

Complex component demonstrating all capability types together.

```toml
[component]
name = "data-processor"
version = "1.0.0"

# Input/output files
[[component.capabilities.filesystem]]
paths = [
    "/data/input/**/*.csv",
    "/data/output/**/*.json"
]
permissions = ["read", "write"]

# Temporary working directory
[[component.capabilities.filesystem]]
paths = ["/tmp/processing-*/*"]
permissions = ["read", "write"]

# API communication
[[component.capabilities.network]]
endpoints = [
    "https://api.example.com/v1/data/*",
    "https://webhook.example.com/progress"
]
permissions = ["connect"]

# Result storage
[[component.capabilities.storage]]
namespaces = ["results/batch-*/*"]
permissions = ["read", "write"]
quota_gb = 5

# Custom capability for payment processor
[[component.capabilities.custom]]
resource_type = "payment"
actions = ["validate_only"]
metadata = { mode = "read_only", environment = "sandbox" }
```

---

## Best Practices

### ✅ Principle of Least Privilege

Declare only the minimum permissions needed for component functionality.

```toml
# ❌ WRONG: Overly permissive
[[component.capabilities.filesystem]]
paths = ["/**"]
permissions = ["read", "write", "execute"]

# ✅ CORRECT: Minimum required
[[component.capabilities.filesystem]]
paths = ["/app/data/config.json"]
permissions = ["read"]
```

### ✅ Use Glob Patterns Precisely

Match files accurately without over-matching.

```toml
# ❌ WRONG: Matches too many files
paths = ["/app/**"]  # Entire /app directory tree

# ✅ CORRECT: Match specific files
paths = ["/app/config/**/*.toml", "/app/data/**/*.json"]
```

### ✅ Separate Permissions by Path

Group related paths with their required permissions.

```toml
# ✅ CORRECT: Organize logically
[[component.capabilities.filesystem]]
paths = ["/app/config/*"]
permissions = ["read"]

[[component.capabilities.filesystem]]
paths = ["/app/logs/*"]
permissions = ["write"]
```

### ✅ Document Complex Capabilities

Add comments explaining why each capability is required.

```toml
# Configuration files are read-only - component only loads startup config
[[component.capabilities.filesystem]]
paths = ["/etc/myapp/config/*"]
permissions = ["read"]

# Log directory requires write access for application logs
[[component.capabilities.filesystem]]
paths = ["/var/log/myapp/*"]
permissions = ["write"]
```

### ✅ Use Quotas for Storage

Always set storage quotas to prevent resource exhaustion.

```toml
[[component.capabilities.storage]]
namespaces = ["cache/*"]
permissions = ["read", "write"]
quota_gb = 2  # Prevent unbounded cache growth
```

---

## Validation and Error Handling

### Checking Your Configuration

The AirsSys security system validates capabilities at component load time:

```bash
# Component.toml is validated when component loads
# Invalid patterns are rejected with clear error messages
```

### Common Validation Errors

| Error | Cause | Solution |
|-------|-------|----------|
| Pattern mismatch | Path doesn't match declared pattern | Update Component.toml pattern |
| Permission denied | Operation not in declared permissions | Add permission to capabilities |
| Quota exceeded | Storage usage exceeds quota | Increase quota_gb or optimize storage |
| Invalid pattern | Syntax error in path pattern | Use glob (*.ext) or recursive (**) syntax |

---

## Testing Your Capabilities

### Manual Testing

Once deployed, verify capabilities work as expected:

1. **Check Positive Case**: Component can access declared resources
2. **Check Negative Case**: Component cannot access undeclared resources
3. **Check Quotas**: Quota limits are enforced correctly

### Example Test Scenarios

```bash
# Test 1: Verify read access works
$ component read /app/config/app.json
✓ Success

# Test 2: Verify write access denied on read-only file
$ component write /app/config/app.json "new data"
✗ Permission denied: write not in [read]

# Test 3: Verify quota enforcement
$ component write /storage/data 100MB
✓ Success (total: 100MB)

$ component write /storage/data 900MB
✗ Quota exceeded: would exceed 1GB limit
```

---

## References

- **Security Architecture**: [security-architecture.md](security-architecture.md)
- **Trust Configuration**: [trust-configuration-guide.md](trust-configuration-guide.md)
- **Best Practices**: [security-best-practices.md](security-best-practices.md)
- **Examples**: [examples/](examples/)
- **Troubleshooting**: [troubleshooting-security.md](troubleshooting-security.md)
