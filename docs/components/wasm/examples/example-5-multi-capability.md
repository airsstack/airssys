# Example 5: Complex Multi-Capability Component

## Scenario

You're building a **comprehensive data pipeline component** that requires multiple capability types. This component:
- Reads input files from filesystem
- Processes data with temporary files
- Stores results in multiple locations
- Sends notifications to external services
- Implements sophisticated security model combining all capability types

## Component Configuration

**Component.toml:**
```toml
[package]
name = "data-pipeline"
version = "2.0.0"
description = "Enterprise data pipeline with multi-capability security"

[component]
name = "data-pipeline"
version = "2.0.0"

# ============================================================================
# CAPABILITY 1: Input Data Files (Read-Only)
# ============================================================================
[[component.capabilities.filesystem]]
description = "Input data directory - read only"
paths = ["/data/input/**/*.csv"]
permissions = ["read"]

# ============================================================================
# CAPABILITY 2: Output Results (Write-Only)
# ============================================================================
[[component.capabilities.filesystem]]
description = "Output results directory - write only"
paths = ["/data/output/**/*.json"]
permissions = ["write"]

# ============================================================================
# CAPABILITY 3: Processing Temporary Files (Read/Write)
# ============================================================================
[[component.capabilities.filesystem]]
description = "Temporary processing directory"
paths = ["/tmp/pipeline-process-*/"]
permissions = ["read", "write"]

# ============================================================================
# CAPABILITY 4: API Communication - Multiple Endpoints
# ============================================================================
[[component.capabilities.network]]
description = "API endpoints for data enrichment"
endpoints = [
    "https://api.data-provider.com/v1/*",      # Main data provider
    "https://backup-api.data-provider.com/*"   # Backup provider
]
permissions = ["connect"]

[[component.capabilities.network]]
description = "Webhook notifications"
endpoints = [
    "https://webhook.company.com/pipeline/*"
]
permissions = ["connect"]

# ============================================================================
# CAPABILITY 5: Persistent State Storage
# ============================================================================
[[component.capabilities.storage]]
description = "Processing state and cache"
namespaces = [
    "pipeline/state/*",      # State machine data
    "pipeline/cache/*",      # Result caching
    "pipeline/archive/*"     # Historical data
]
permissions = ["read", "write"]
quota_gb = 5

# ============================================================================
# CAPABILITY 6: Application Custom Capability
# ============================================================================
[[component.capabilities.custom]]
description = "Database query capability"
resource_type = "database"
actions = ["query_read", "query_write"]
metadata = {
    database = "analytics",
    environment = "production",
    max_query_time_seconds = 30
}
```

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│ Data Pipeline Component                                              │
│                                                                      │
│ ┌──────────────────────────────────────────────────────────────┐   │
│ │ INPUT: Read CSV files from /data/input/                      │   │
│ │ Capability: filesystem [/data/input/**/*.csv] [read]         │   │
│ └──────────────────────────────────────────────────────────────┘   │
│          ↓                                                           │
│ ┌──────────────────────────────────────────────────────────────┐   │
│ │ ENRICH: Call APIs to add external data                       │   │
│ │ Capability: network [https://api.data-provider.com/*] [connect] │
│ └──────────────────────────────────────────────────────────────┘   │
│          ↓                                                           │
│ ┌──────────────────────────────────────────────────────────────┐   │
│ │ QUERY: Execute database queries for analytics                │   │
│ │ Capability: custom [database] [query_read, query_write]      │   │
│ └──────────────────────────────────────────────────────────────┘   │
│          ↓                                                           │
│ ┌──────────────────────────────────────────────────────────────┐   │
│ │ PROCESS: Use temp files for intermediate data                │   │
│ │ Capability: filesystem [/tmp/pipeline-process-*/] [read/write]  │
│ └──────────────────────────────────────────────────────────────┘   │
│          ↓                                                           │
│ ┌──────────────────────────────────────────────────────────────┐   │
│ │ CACHE: Store results in persistent storage                   │   │
│ │ Capability: storage [pipeline/cache/*] [read/write, 5GB quota]  │
│ └──────────────────────────────────────────────────────────────┘   │
│          ↓                                                           │
│ ┌──────────────────────────────────────────────────────────────┐   │
│ │ OUTPUT: Write final results                                   │   │
│ │ Capability: filesystem [/data/output/**/*.json] [write]       │   │
│ └──────────────────────────────────────────────────────────────┘   │
│          ↓                                                           │
│ ┌──────────────────────────────────────────────────────────────┐   │
│ │ NOTIFY: Send completion webhook                              │   │
│ │ Capability: network [https://webhook.company.com/*] [connect]  │
│ └──────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

## Execution Flow with Capability Checks

### Step 1: Read Input Data

```rust
// Host function invoked: read_csv("/data/input/2025-12-20/sales.csv")
// Capability check sequence:

1. Normalize path: /data/input/2025-12-20/sales.csv
2. Pattern match: matches /data/input/**/*.csv ✓
3. Permission check: "read" in ["read"] ✓
4. Audit: GRANTED
5. Return file contents

Result: ✓ GRANTED
Audit: 2025-12-20T15:30:00.001Z | pipeline | /data/input/2025-12-20/sales.csv | read | granted | -
```

### Step 2: Enrich with API Data

```rust
// Host function invoked: connect("https://api.data-provider.com/v1/enrich")
// Capability check sequence:

1. Parse endpoint URL: https://api.data-provider.com/v1/enrich
2. Pattern match: matches https://api.data-provider.com/v1/* ✓
3. Permission check: "connect" in ["connect"] ✓
4. Audit: GRANTED
5. Establish connection

Result: ✓ GRANTED
Audit: 2025-12-20T15:30:01.000Z | pipeline | https://api.data-provider.com/v1/enrich | connect | granted | -
```

### Step 3: Query Database

```rust
// Host function invoked: query_database("SELECT * FROM analytics.sales")
// Capability check sequence:

1. Custom capability type: database
2. Action check: "query_read" in ["query_read", "query_write"] ✓
3. Metadata validation: query_time < 30 seconds ✓
4. Environment check: production ✓
5. Audit: GRANTED
6. Execute query

Result: ✓ GRANTED
Audit: 2025-12-20T15:30:02.000Z | pipeline | database:query | query_read | granted | -
```

### Step 4: Create Temporary Files

```rust
// Host function invoked: write_temp("/tmp/pipeline-process-001/intermediate.json")
// Capability check sequence:

1. Normalize path: /tmp/pipeline-process-001/intermediate.json
2. Pattern match: matches /tmp/pipeline-process-*/ ✓
3. Permission check: "write" in ["read", "write"] ✓
4. Filesystem quota: file_size fits in allocation ✓
5. Audit: GRANTED
6. Write temporary data

Result: ✓ GRANTED
Audit: 2025-12-20T15:30:03.000Z | pipeline | /tmp/pipeline-process-001/intermediate.json | write | granted | -
```

### Step 5: Cache Processing State

```rust
// Host function invoked: store_state("pipeline/cache/batch-001", state_data)
// Capability check sequence:

1. Namespace check: matches pipeline/cache/* ✓
2. Permission check: "write" in ["read", "write"] ✓
3. Storage quota: 2.5MB fits in 5GB (usage: 2.5MB/5GB) ✓
4. Audit: GRANTED
5. Store in persistent storage

Result: ✓ GRANTED
Audit: 2025-12-20T15:30:04.000Z | pipeline | pipeline/cache/batch-001 | write | granted | usage: 2.5MB/5GB
```

### Step 6: Write Output Results

```rust
// Host function invoked: write_output("/data/output/2025-12-20/results.json")
// Capability check sequence:

1. Normalize path: /data/output/2025-12-20/results.json
2. Pattern match: matches /data/output/**/*.json ✓
3. Permission check: "write" in ["write"] ✓
4. Audit: GRANTED
5. Write results

Result: ✓ GRANTED
Audit: 2025-12-20T15:30:05.000Z | pipeline | /data/output/2025-12-20/results.json | write | granted | -
```

### Step 7: Send Notification

```rust
// Host function invoked: connect("https://webhook.company.com/pipeline/complete")
// Capability check sequence:

1. Parse endpoint URL: https://webhook.company.com/pipeline/complete
2. Pattern match: matches https://webhook.company.com/pipeline/* ✓
3. Permission check: "connect" in ["connect"] ✓
4. Audit: GRANTED
5. Send webhook notification

Result: ✓ GRANTED
Audit: 2025-12-20T15:30:06.000Z | pipeline | https://webhook.company.com/pipeline/complete | connect | granted | -
```

## Security Features

### ✓ Read/Write Separation

Component has different permissions per path:
- Inputs: read-only (cannot modify input data)
- Outputs: write-only (cannot read back output)
- Temporary: read/write (for processing)

### ✓ Isolated Temporary Space

Temporary files are in uniquely-named directory:
```
/tmp/pipeline-process-001/  ← component instance A
/tmp/pipeline-process-002/  ← component instance B

Each instance cannot read other's temp files!
Each instance sees unique process ID
```

### ✓ Quota-Limited Storage

```
5GB quota across all pipeline storage namespaces
- pipeline/state/*
- pipeline/cache/*
- pipeline/archive/*

Component cannot exceed 5GB total
Prevents resource exhaustion
Enables multi-tenancy (multiple instances)
```

### ✓ Custom Capability Metadata

Database queries validated with metadata:
```
resource_type = "database"
database = "analytics"        ← Enforced
environment = "production"    ← Enforced
max_query_time_seconds = 30   ← Enforced

Host function can validate:
- Is this the analytics database? YES
- Is this production? YES
- Is query < 30 seconds? Check during execution
```

### ✓ Multi-Endpoint Network Control

Component can access multiple endpoints but not others:
```
Allowed:
  https://api.data-provider.com/v1/enrich
  https://backup-api.data-provider.com/fallback
  https://webhook.company.com/pipeline/complete

Blocked:
  https://attacker.com/steal-data
  https://api.data-provider.com/admin
  http://insecure.provider.com/data
```

## Attack Prevention Examples

### Attack 1: Unauthorized Data Exfiltration

```
Malicious code: Connect to https://attacker.com/exfil

Capability check:
  - Pattern matches: https://webhook.company.com/pipeline/* ?
  - NO: https://attacker.com/exfil does not match
  - Result: DENIED

Audit: 2025-12-20T15:30:10.000Z | pipeline | https://attacker.com/exfil | connect | denied | Not in whitelist
```

### Attack 2: Modifying Input Files

```
Malicious code: Write to /data/input/malicious.csv

Capability check:
  - Permission: "write" in [read] ?
  - NO: Only read permission declared
  - Result: DENIED

Audit: 2025-12-20T15:30:11.000Z | pipeline | /data/input/malicious.csv | write | denied | write not in [read]
```

### Attack 3: Excessive Storage Usage

```
Malicious code: Write 10GB to pipeline/cache/

Capability check:
  - Storage quota: current (4.5GB) + request (10GB) <= limit (5GB) ?
  - NO: 14.5GB > 5GB
  - Result: DENIED

Audit: 2025-12-20T15:30:12.000Z | pipeline | pipeline/cache/huge-file | write | denied | Quota exceeded (14.5GB > 5GB)
```

### Attack 4: Long-Running Database Query

```
Malicious code: SELECT * FROM huge_table (takes 120 seconds)

Capability check (in host function):
  - Query timeout: query_time > 30 seconds ?
  - YES: Query exceeded max_query_time_seconds
  - Result: TERMINATED

Audit: 2025-12-20T15:30:13.000Z | pipeline | database:query | query_read | denied | Query timeout exceeded (120s > 30s max)
```

## Complete Audit Trail

A full execution session produces comprehensive logs:

```
2025-12-20T15:30:00.001Z | pipeline | /data/input/2025-12-20/sales.csv | read | granted | -
2025-12-20T15:30:01.000Z | pipeline | https://api.data-provider.com/v1/enrich | connect | granted | -
2025-12-20T15:30:02.000Z | pipeline | database:query | query_read | granted | -
2025-12-20T15:30:03.000Z | pipeline | /tmp/pipeline-process-001/intermediate.json | write | granted | -
2025-12-20T15:30:04.000Z | pipeline | pipeline/cache/batch-001 | write | granted | usage: 2.5MB/5GB
2025-12-20T15:30:05.000Z | pipeline | /data/output/2025-12-20/results.json | write | granted | -
2025-12-20T15:30:06.000Z | pipeline | https://webhook.company.com/pipeline/complete | connect | granted | -
2025-12-20T15:30:07.000Z | pipeline | pipeline/state/completion | write | granted | usage: 2.51MB/5GB

No denials (normal operation).
All operations properly authorized and logged.
```

## Production Deployment Checklist

- [ ] **Capability Completeness**: Component has all needed capabilities
- [ ] **Principle of Least Privilege**: No unnecessary permissions granted
- [ ] **Quota Sizing**: 5GB quota is appropriate for expected workload
- [ ] **Error Handling**: Component handles permission denials gracefully
- [ ] **Monitoring**: Audit logs are collected and monitored
- [ ] **Quota Alerts**: Alert if usage exceeds 80% (4GB)
- [ ] **Network Allowlist**: All needed endpoints are whitelisted
- [ ] **Temporary Cleanup**: Temp files are cleaned up after processing
- [ ] **State Retention**: Important state is saved to persistent storage
- [ ] **Performance**: Capability checks don't create bottleneck (<5μs)

## Performance Considerations

### Capability Check Overhead

Each operation incurs ~3-4μs overhead:
```
Read input file: 3-4μs check + file I/O (~10ms)
API call: 3-4μs check + network (~100ms)
Database query: 3-4μs check + query execution (~50ms)
Storage operation: 3-4μs check + storage I/O (~5ms)
```

Overhead is negligible compared to actual operation time.

### Optimization

```rust
// Implement connection pooling to reduce checks
struct ConnectionPool {
    connections: HashMap<String, Vec<Connection>>,
    // Checks happen once per pool creation, not per request
}

// Cache database query plans
struct QueryCache {
    plans: HashMap<String, QueryPlan>,
    // Validated once, reused multiple times
}

// Batch temporary file operations
fn batch_write_temp_files(files: Vec<(Path, Data)>) {
    for (path, data) in files {
        // 3-4μs × N operations, but amortized through batching
        write_temp_file(path, data)?;
    }
}
```

## Next Steps

- Review [capability-declaration-guide.md](../capability-declaration-guide.md) for complete syntax
- Study [security-best-practices.md](../security-best-practices.md) for design patterns
- See [security-architecture.md](../security-architecture.md) for implementation details
- Check [host-integration-guide.md](../host-integration-guide.md) for host function patterns
