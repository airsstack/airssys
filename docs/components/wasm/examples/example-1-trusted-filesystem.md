# Example 1: Trusted Filesystem Component

## Scenario

You're building a **configuration loader** component that your organization created. This component:
- Loads application configuration from a well-known path
- Reads environment-specific settings
- Is trusted (from your org's Git repo)
- Should have instant approval

## Component Configuration

**Component.toml:**
```toml
[package]
name = "config-loader"
version = "1.0.0"
description = "Loads application configuration files"

[component]
name = "config-loader"
version = "1.0.0"

# Trust configuration: trusted from organization repo
trust = "org-internal"

# Filesystem capabilities: read-only access to config directory
[[component.capabilities.filesystem]]
paths = [
    "/etc/myapp/config.toml",      # Main config file
    "/etc/myapp/config/*.toml",    # Environment-specific configs
    "/etc/myapp/config/**/*.json"  # Nested JSON configs
]
permissions = ["read"]

# No network access
# No storage access
```

## Explanation

### Capability Declaration

```
[[component.capabilities.filesystem]]
paths = [
    "/etc/myapp/config.toml",      # Exact file match
    "/etc/myapp/config/*.toml",    # Glob pattern: .toml files in /config
    "/etc/myapp/config/**/*.json"  # Recursive: all .json files in subdirs
]
permissions = ["read"]
```

**What this allows:**
- ✓ Read `/etc/myapp/config.toml`
- ✓ Read `/etc/myapp/config/prod.toml`
- ✓ Read `/etc/myapp/config/staging/db.json`
- ✓ Read `/etc/myapp/config/2024/12/overrides.json`

**What this denies:**
- ✗ Write to any files (permission not declared)
- ✗ Read from `/etc/myapp/scripts/` (outside pattern)
- ✗ Read from `/home/user/config/` (outside pattern)
- ✗ Execute any files (execute permission not declared)

### Principle of Least Privilege

This component declares exactly what it needs:
- Only reads config files (no write access)
- Only accesses specific directories
- Cannot accidentally modify anything
- Cannot access unrelated directories

### Pattern Matching

| Path Requested | Pattern Match | Result |
|---|---|---|
| `/etc/myapp/config.toml` | Exact match | ✓ GRANT |
| `/etc/myapp/config/prod.toml` | Glob `*.toml` | ✓ GRANT |
| `/etc/myapp/config/db/conn.json` | Recursive `**/*.json` | ✓ GRANT |
| `/etc/myapp/scripts/init.sh` | No pattern matches | ✗ DENY |
| `/home/user/config.toml` | No pattern matches | ✗ DENY |

## Trust Configuration

In your trust configuration file (`trust-config.toml`):

```toml
[trust]
dev_mode = false

# Trust internal organization repositories
[[trust.sources]]
type = "git"
url_pattern = "https://github.com/myorg/*"
branch = "main"
description = "Internal organization repositories"
```

**Behavior:**
1. Organization requests component installation
2. Component metadata shows Git URL: `https://github.com/myorg/config-loader`
3. TrustRegistry checks against patterns
4. URL matches `https://github.com/myorg/*` ✓
5. TrustLevel determined: **Trusted**
6. Component instantly installed (no manual approval needed)

## Host Function Implementation

When the component calls host functions to read config files:

```rust
// Host function for reading files
pub fn host_read_file(
    component_id: &str,
    path: &str,
) -> Result<Vec<u8>> {
    // 1. Normalize the path (resolve .. and . components)
    let normalized = std::fs::canonicalize(path)?;
    
    // 2. Check capability
    check_capability(component_id, &normalized, "read")?;
    
    // 3. Audit log the access
    audit_logger.log_access_granted(component_id, &normalized, "read");
    
    // 4. Perform the actual read
    std::fs::read(&normalized)
}
```

**Example flow:**
```
Component calls: read("/etc/myapp/config/prod.toml")
        ↓
Normalize: /etc/myapp/config/prod.toml (no ../ to resolve)
        ↓
check_capability("config-loader", "/etc/myapp/config/prod.toml", "read")
        ↓
Pattern match: Matches /etc/myapp/config/*.toml ✓
Permission check: "read" in ["read"] ✓
        ↓
Audit log: 2025-12-20T15:30:00Z | config-loader | /etc/myapp/config/prod.toml | read | granted
        ↓
Return file contents to component
```

## Audit Trail

When this component runs, the audit log shows:

```
2025-12-20T15:30:45.123Z | config-loader | /etc/myapp/config.toml | read | granted | -
2025-12-20T15:30:45.456Z | config-loader | /etc/myapp/config/prod.toml | read | granted | -
2025-12-20T15:30:45.789Z | config-loader | /etc/myapp/config/db/connection.json | read | granted | -
```

No denials expected (all requested paths are declared).

## Benefits of This Pattern

✓ **Security:**
- Component cannot write config files (read-only)
- Cannot access other applications' configs
- Cannot execute scripts
- All access is auditable

✓ **Productivity:**
- Trusted source: instant installation
- No manual approval needed
- Team can deploy rapidly

✓ **Maintainability:**
- Clear what component needs (visible in Component.toml)
- Easy to audit (all access logged)
- Easy to update (modify patterns as needed)

## Common Issues & Solutions

### Issue: Component reads from unexpected path

**Error:** `Permission denied: /etc/myapp/custom.toml`

**Cause:** Path not in declared patterns

**Solution:** Add pattern to Component.toml:
```toml
[[component.capabilities.filesystem]]
paths = [
    "/etc/myapp/config.toml",
    "/etc/myapp/config/*.toml",
    "/etc/myapp/custom.toml"    # NEW: Add custom config
]
```

### Issue: Multiple environment configs scattered

**Error:** Component cannot read from `/etc/myapp/env/prod.json`

**Cause:** Pattern `*.toml` doesn't match `.json` files in `/env`

**Solution:** Expand patterns:
```toml
[[component.capabilities.filesystem]]
paths = [
    "/etc/myapp/config/**/*",    # All files in config subtree
    "/etc/myapp/env/**/*"        # All files in env subtree
]
```

## Next Steps

- Review [capability-declaration-guide.md](../capability-declaration-guide.md) for more pattern examples
- Read [security-best-practices.md](../security-best-practices.md) for principle of least privilege
- See [example-2-unknown-approval.md](example-2-unknown-approval.md) for third-party component workflow
