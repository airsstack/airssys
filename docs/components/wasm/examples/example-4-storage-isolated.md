# Example 4: Storage-Isolated Multi-Tenant Component

## Scenario

You're building a **multi-tenant data processor** component that:
- Manages data for multiple customers (tenants)
- Each tenant has isolated storage namespace
- Components cannot access other tenants' data
- Quotas prevent one tenant from exhausting resources

## Component Configuration

**Component.toml:**
```toml
[package]
name = "tenant-data-processor"
version = "1.0.0"
description = "Multi-tenant data processor with namespace isolation"

[component]
name = "tenant-data-processor"
version = "1.0.0"

# Storage capabilities: namespace-based isolation
[[component.capabilities.storage]]
# Each tenant accesses only their namespace
namespaces = [
    "tenant-*/data/*",      # Tenant data directory
    "tenant-*/metadata/*",  # Tenant metadata
    "shared/indexes/*"      # Shared indexes (read-write)
]
permissions = ["read", "write"]
quota_gb = 10               # 10GB per component instance

# Filesystem: temporary working directory
[[component.capabilities.filesystem]]
paths = ["/tmp/processing-*/*"]
permissions = ["read", "write"]
```

## Storage Architecture

### Namespace Structure

```
storage/
├── tenant-123/
│   ├── data/
│   │   ├── records.json
│   │   ├── documents/
│   │   │   ├── doc-001.pdf
│   │   │   └── doc-002.pdf
│   │   └── 2025-12-20/
│   │       └── batch-001.json
│   └── metadata/
│       ├── config.json
│       └── schema.json
│
├── tenant-456/
│   ├── data/
│   │   ├── records.json
│   │   ├── documents/
│   │   └── 2025-12-20/
│   └── metadata/
│       ├── config.json
│       └── schema.json
│
└── shared/
    ├── indexes/
    │   ├── tenant-index.json
    │   └── search-index/
```

### Namespace Pattern Matching

```toml
namespaces = [
    "tenant-*/data/*",      # Pattern 1: tenant data
    "tenant-*/metadata/*",  # Pattern 2: tenant metadata
    "shared/indexes/*"      # Pattern 3: shared indexes
]
```

**What this allows:**

| Namespace | Pattern | Result |
|---|---|---|
| `tenant-123/data/records.json` | `tenant-*/data/*` | ✓ GRANT |
| `tenant-456/data/documents/file.pdf` | `tenant-*/data/*` | ✓ GRANT |
| `tenant-123/metadata/config.json` | `tenant-*/metadata/*` | ✓ GRANT |
| `shared/indexes/tenant-index.json` | `shared/indexes/*` | ✓ GRANT |
| `tenant-123/private/secret` | No pattern match | ✗ DENY |
| `tenant-456/data/secret` | Matches pattern but verified | ✓ GRANT |
| `other/data/file` | No pattern match | ✗ DENY |

## Isolation Mechanism

### Namespace Boundaries

Component can read/write within namespace, cannot cross boundaries:

```
Component Instance 1 processing Tenant-123
    ├─ Can read: tenant-123/*
    ├─ Can write: tenant-123/*
    └─ Cannot access: tenant-456/* or other/*

Component Instance 2 processing Tenant-456
    ├─ Can read: tenant-456/*
    ├─ Can write: tenant-456/*
    └─ Cannot access: tenant-123/* or other/*
```

### Host Function Implementation

```rust
pub fn storage_write(
    component_id: &str,
    namespace: &str,
    key: &str,
    data: &[u8],
) -> Result<()> {
    // 1. Build full path
    let full_path = format!("{}/{}", namespace, key);
    
    // 2. Verify namespace is in declared capabilities
    let mut is_allowed = false;
    for declared_namespace in component_capabilities.storage.namespaces {
        if matches_namespace_pattern(&full_path, &declared_namespace) {
            is_allowed = true;
            break;
        }
    }
    
    if !is_allowed {
        audit_logger.log_access_denied(component_id, &full_path, "write", "Namespace not in capability");
        return Err(format!("Access denied: {} not in namespaces", namespace));
    }
    
    // 3. Check permission
    if !"write".in("component_capabilities.storage.permissions") {
        return Err("Permission denied: write not in permissions");
    }
    
    // 4. Check quota
    let size_kb = data.len() / 1024;
    if quota_tracker.current_kb + size_kb > quota_limit_kb {
        return Err("Quota exceeded");
    }
    
    // 5. Write data
    storage_backend.write(&full_path, data)?;
    
    // 6. Update quota tracking
    quota_tracker.current_kb += size_kb;
    
    // 7. Audit log
    audit_logger.log_access_granted(component_id, &full_path, "write");
    
    Ok(())
}
```

## Tenant Isolation Example

### Scenario: Processing Two Tenants

**Tenant 123 Workflow:**
```
1. Component instance gets task: "process tenant-123"
   
2. Read tenant-123 config
   - storage_read(component_id, "tenant-123/metadata/config.json")
   - Capability check: "tenant-123/metadata/config.json" matches "tenant-*/metadata/*" ✓
   - Permission: "read" in ["read", "write"] ✓
   - Quota: 5KB fits in 10GB ✓
   - Result: GRANTED
   - Audit: 2025-12-20T15:30:00Z | tenant-data-processor | tenant-123/metadata/config.json | read | granted | usage: 5KB/10GB

3. Read tenant-123 data records
   - storage_read(component_id, "tenant-123/data/records.json")
   - Capability check: "tenant-123/data/records.json" matches "tenant-*/data/*" ✓
   - Result: GRANTED
   - Audit: 2025-12-20T15:30:01Z | tenant-data-processor | tenant-123/data/records.json | read | granted | usage: 505KB/10GB

4. Process records (no storage needed)

5. Write processed results
   - storage_write(component_id, "tenant-123/data/2025-12-20/results.json", data)
   - Capability check: "tenant-123/data/2025-12-20/results.json" matches "tenant-*/data/*" ✓
   - Permission: "write" in ["read", "write"] ✓
   - Quota: 1.2MB fits in 10GB (1.7MB total) ✓
   - Result: GRANTED
   - Audit: 2025-12-20T15:30:05Z | tenant-data-processor | tenant-123/data/2025-12-20/results.json | write | granted | usage: 1.7MB/10GB
```

**Tenant 456 Workflow:**
```
1. Component instance gets task: "process tenant-456"

2. Attempt to read tenant-123 config (SECURITY VIOLATION)
   - storage_read(component_id, "tenant-123/metadata/config.json")
   - Pattern check: "tenant-123/metadata/config.json" matches "tenant-*/metadata/*" ✓
   - BUT: Task is for tenant-456, not tenant-123
   - Host function MUST validate tenant context!
   - Application check: tenant_from_context != "tenant-123"
   - Result: DENIED (at application layer, not storage layer)
   - Audit: 2025-12-20T15:30:10Z | tenant-data-processor | tenant-123/metadata/config.json | read | denied | Wrong tenant in context
```

## Quota Management

### Per-Component Quota

```toml
[[component.capabilities.storage]]
namespaces = ["tenant-*/*"]
permissions = ["read", "write"]
quota_gb = 10  # Total quota for this component instance
```

**Quota applies to:**
- ALL data written by this component instance
- Across ALL tenant namespaces combined
- 10GB total, not per tenant

### Example Quota Usage

```
Component instance 1 (processes tenant-123):
  - Quota: 10GB
  - Current usage: 7.5GB
  
Component instance 2 (processes tenant-456):
  - Quota: 10GB
  - Current usage: 3.2GB

Component instance 3 (processes tenant-789):
  - Quota: 10GB
  - Current usage: 9.8GB

Each instance has independent quota!
Instances do not share quota pool.
```

### Quota Enforcement Flow

```
Component writes 2GB of data
    ↓
Check: current_usage (9.8GB) + write (2GB) <= quota (10GB)?
    ├─ NO: 11.8GB > 10GB
    │       ↓
    │   DENIED: Quota exceeded
    │   Error returned to component
    │
    └─ YES: 7.5GB + 2GB < 10GB ✓
            ↓
        Write allowed
        Quota updated: 9.5GB
        Audit logged
```

## Multi-Tenant Best Practices

### ✓ Verify Tenant Context

Every storage operation should verify component is accessing correct tenant:

```rust
// Host function MUST validate tenant context
pub fn storage_write(
    component_id: &str,
    namespace: &str,
    key: &str,
    data: &[u8],
    context: &ComponentContext,  // Must include tenant info
) -> Result<()> {
    // Extract tenant from context
    let expected_tenant = extract_tenant_from_context(context)?;
    
    // Extract tenant from namespace path
    let actual_tenant = extract_tenant_from_namespace(namespace)?;
    
    // Verify they match (defense in depth!)
    if expected_tenant != actual_tenant {
        return Err(format!(
            "Tenant mismatch: context says {}, namespace says {}",
            expected_tenant, actual_tenant
        ));
    }
    
    // Now safe to proceed
    // ...
}
```

### ✓ Isolate Processing

Each component instance should process only one tenant:

```
Good Architecture:
  - Pool of 3 component instances
  - Instance 1 → processes only tenant-123
  - Instance 2 → processes only tenant-456
  - Instance 3 → processes only tenant-789
  - No instance accesses multiple tenants

Bad Architecture:
  - Single component instance
  - Processes all tenants in sequence
  - Higher risk of cross-tenant data leakage
  - Single quota shared across all tenants
```

### ✓ Monitor Quota Usage

Track quota usage per tenant to detect anomalies:

```rust
struct QuotaMonitor {
    per_tenant_usage: HashMap<String, u64>,
}

impl QuotaMonitor {
    fn check_tenant_anomaly(&self, tenant: &str, new_bytes: u64) {
        let current = self.per_tenant_usage.get(tenant).unwrap_or(&0);
        
        // Alert if single write is >100MB or daily total >1GB
        if new_bytes > 100 * 1024 * 1024 {
            log_alert(format!("Large write for {}: {}MB", tenant, new_bytes / 1024 / 1024));
        }
    }
}
```

### ✓ Regular Cleanup

Implement periodic cleanup to prevent quota exhaustion:

```rust
pub fn cleanup_old_data(
    component_id: &str,
    tenant: &str,
    days: u32,
) -> Result<u64> {
    let cutoff = SystemTime::now() - Duration::days(days);
    let mut freed_kb = 0;
    
    // Delete tenant-specific old data
    let namespace = format!("tenant-{}/data", tenant);
    for entry in storage_list(&namespace)? {
        if entry.modified < cutoff {
            freed_kb += storage_delete(&entry.path)? / 1024;
        }
    }
    
    quota_tracker.current_kb -= freed_kb;
    audit_logger.log_cleanup(component_id, tenant, freed_kb);
    
    Ok(freed_kb)
}
```

## Shared Resources

### Shared Indexes

Components can read/write shared indexes for all tenants:

```toml
namespaces = [
    "tenant-*/data/*",
    "tenant-*/metadata/*",
    "shared/indexes/*"  # Shared index for search
]
```

**Use case:** Tenant-agnostic search index

```
tenant-123 writes: index entry for "user:123"
tenant-456 writes: index entry for "user:456"
tenant-123 reads: search index (finds both entries)
tenant-456 reads: search index (finds both entries)

No isolation on shared namespace!
Use with caution for truly shared data only.
```

## Audit Trail

A multi-tenant session produces comprehensive audit logs:

```
2025-12-20T15:30:00.001Z | processor-1 | tenant-123/metadata/config.json | read | granted | usage: 5KB/10GB
2025-12-20T15:30:00.500Z | processor-1 | tenant-123/data/records.json | read | granted | usage: 505KB/10GB
2025-12-20T15:30:05.000Z | processor-1 | tenant-123/data/2025-12-20/results.json | write | granted | usage: 1.7MB/10GB
2025-12-20T15:30:10.000Z | processor-2 | tenant-456/metadata/config.json | read | granted | usage: 5KB/10GB
2025-12-20T15:30:15.000Z | processor-3 | shared/indexes/search-index | write | granted | usage: 200MB/10GB
2025-12-20T15:30:20.000Z | processor-2 | tenant-456/data/records.json | read | granted | usage: 510KB/10GB
2025-12-20T15:30:25.000Z | processor-1 | tenant-123/data/2025-12-20/results-2.json | write | granted | usage: 3.4MB/10GB
2025-12-20T15:30:30.000Z | processor-3 | shared/indexes/search-index | read | granted | usage: 200MB/10GB

All cross-tenant accesses properly logged.
No denials (correct behavior).
```

## Next Steps

- Review [capability-declaration-guide.md](../capability-declaration-guide.md) for storage patterns
- See [example-5-multi-capability.md](example-5-multi-capability.md) for combining multiple capabilities
- Read [security-best-practices.md](../security-best-practices.md) for isolation patterns
