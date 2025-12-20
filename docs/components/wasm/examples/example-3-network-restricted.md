# Example 3: Network-Restricted API Client Component

## Scenario

You're building an **API client component** that connects to external services. This component:
- Calls specific REST APIs in production
- Should only connect to authorized endpoints
- Must NOT connect to other services
- Uses endpoint whitelisting for security

## Component Configuration

**Component.toml:**
```toml
[package]
name = "api-client"
version = "1.0.0"
description = "Secure API client with endpoint whitelisting"

[component]
name = "api-client"
version = "1.0.0"

# Network capabilities: connect to specific endpoints only
[[component.capabilities.network]]
endpoints = [
    "https://api.example.com/v1/*",     # Production API
    "https://api.example.com/v2/*",     # API v2
    "https://webhook.example.com/*",    # Webhook receiver
    "https://health.example.com/status" # Health check endpoint
]
permissions = ["connect"]

# Filesystem: cache API responses
[[component.capabilities.filesystem]]
paths = ["/var/cache/api-client/*"]
permissions = ["read", "write"]

# Storage: persistent API state
[[component.capabilities.storage]]
namespaces = ["api-state/*"]
permissions = ["read", "write"]
quota_gb = 1
```

## Network Capability Explanation

### Endpoint Patterns

```toml
endpoints = [
    "https://api.example.com/v1/*",     # Glob pattern: all /v1 endpoints
    "https://api.example.com/v2/*",     # Glob pattern: all /v2 endpoints
    "https://webhook.example.com/*",    # Glob pattern: all webhook paths
    "https://health.example.com/status" # Exact endpoint match
]
```

### What This Allows

| Endpoint | Pattern | Result |
|---|---|---|
| `https://api.example.com/v1/users` | Matches `/v1/*` | ✓ GRANT |
| `https://api.example.com/v1/users/123` | Matches `/v1/*` | ✓ GRANT |
| `https://api.example.com/v2/products` | Matches `/v2/*` | ✓ GRANT |
| `https://webhook.example.com/events` | Matches `/*` | ✓ GRANT |
| `https://health.example.com/status` | Exact match | ✓ GRANT |

### What This Denies

| Endpoint | Reason | Result |
|---|---|---|
| `https://api.example.com/admin/users` | Not in `/v1/*` or `/v2/*` | ✗ DENY |
| `https://malicious.com/api/data` | Not in whitelist | ✗ DENY |
| `http://api.example.com/v1/users` | Wrong scheme (http not https) | ✗ DENY |
| `https://api.example.com/v3/data` | v3 not whitelisted | ✗ DENY |

## Host Function Implementation

When the component initiates network connections:

```rust
// Host function for network connections
pub fn host_connect(
    component_id: &str,
    endpoint: &str,
    timeout_ms: u32,
) -> Result<NetworkConnection> {
    // 1. Parse the endpoint URL
    let url = Url::parse(endpoint)?;
    
    // 2. Check capability
    check_capability(component_id, endpoint, "connect")?;
    
    // 3. Audit log the connection attempt
    audit_logger.log_network_connection_attempt(component_id, endpoint);
    
    // 4. Establish connection (with timeout)
    let connection = establish_connection(&url, timeout_ms)?;
    
    // 5. Log successful connection
    audit_logger.log_access_granted(component_id, endpoint, "connect");
    
    Ok(connection)
}
```

## Typical Operation

### Successful Workflow

```
1. Component needs user data
   - Calls: host_connect("api.example.com/v1/users/123", timeout)
   - Capability check: "https://api.example.com/v1/*" ✓
   - Connection established
   - Returns connection handle to component

2. Component sends request
   - Uses connection to send HTTP GET
   - Receives 200 OK with user data
   - Audit logged: connect GRANTED

3. Component caches result
   - Path: /var/cache/api-client/users/123.json
   - Capability check: /var/cache/api-client/* [write] ✓
   - Cache entry created

4. Component updates state
   - Namespace: api-state/user-cache
   - Capability check: api-state/* [write] ✓
   - Quota check: 150MB/1GB ✓
   - State updated
```

### Attack Prevention

```
Attack Scenario 1: Malicious code tries to phone home
   - Endpoint: https://attacker.com/steal-data
   - Capability check: does NOT match any pattern ✗
   - Result: Connection DENIED
   - Audit: 2025-12-20T15:30:00Z | api-client | https://attacker.com/steal-data | connect | denied | Not in whitelist

Attack Scenario 2: Lateral movement to admin API
   - Endpoint: https://api.example.com/admin/users
   - Capability check: does NOT match /v1/* or /v2/* ✗
   - Result: Connection DENIED
   - Audit: 2025-12-20T15:31:00Z | api-client | https://api.example.com/admin/users | connect | denied | Endpoint not authorized

Attack Scenario 3: Using insecure connection
   - Endpoint: http://api.example.com/v1/users (http not https)
   - Capability check: pattern requires https ✗
   - Result: Connection DENIED
   - Audit: 2025-12-20T15:32:00Z | api-client | http://api.example.com/v1/users | connect | denied | Scheme mismatch
```

## Domain Restriction Patterns

### Single Domain, Multiple Versions

```toml
[[component.capabilities.network]]
endpoints = [
    "https://api.example.com/v1/*",
    "https://api.example.com/v2/*"
]
permissions = ["connect"]
```

**Allows:**
- ✓ All v1 endpoints
- ✓ All v2 endpoints
- ✗ Admin endpoints
- ✗ v3 endpoints

### Multiple Domains

```toml
[[component.capabilities.network]]
endpoints = [
    "https://api.primary.com/*",
    "https://api.backup.com/*"
]
permissions = ["connect"]
```

**Allows:**
- ✓ All endpoints on primary and backup APIs
- ✗ Other domains

### Hierarchical Endpoints

```toml
[[component.capabilities.network]]
endpoints = [
    "https://api.example.com/*",        # All endpoints
    "https://webhook.example.com/*"     # Webhooks
]
permissions = ["connect"]
```

### Port-Specific Endpoints

```toml
[[component.capabilities.network]]
endpoints = [
    "https://api.example.com:8443/*",   # Custom HTTPS port
    "https://api.backup.com:9443/*"     # Alternative port
]
permissions = ["connect"]
```

## Cache Configuration

**Why cache locally?**
- Reduces API calls (faster, lower latency)
- Graceful degradation if API unavailable
- Quota controlled locally

```toml
[[component.capabilities.filesystem]]
paths = ["/var/cache/api-client/*"]
permissions = ["read", "write"]
```

**Cache operations:**
```
Read cached response:
  - Path: /var/cache/api-client/users/123.json
  - Check: /var/cache/api-client/* [read] ✓
  - Return cached data

Write cache entry:
  - Path: /var/cache/api-client/products/456.json
  - Check: /var/cache/api-client/* [write] ✓
  - Size: 50KB (respects filesystem quota)
  - Write cached data
```

## Storage for State

**Why persistent state?**
- Track last successful API call
- Rate limiting state
- Retry policy state
- User session information

```toml
[[component.capabilities.storage]]
namespaces = ["api-state/*"]
permissions = ["read", "write"]
quota_gb = 1
```

**State operations:**
```
Write last-sync timestamp:
  - Namespace: api-state/sync/users
  - Data: { last_sync: 2025-12-20T15:30:00Z }
  - Quota: 1KB (1GB quota available)

Read retry policy:
  - Namespace: api-state/retry-policy
  - Data: { max_retries: 3, backoff_ms: 1000 }
  - Quota: Already allocated

Cleanup old state:
  - Namespace: api-state/archive
  - Remove entries older than 90 days
  - Quota freed: 50MB
```

## Audit Trail

A typical operation session produces this audit trail:

```
2025-12-20T15:30:00.001Z | api-client | https://api.example.com/v1/users | connect | granted | -
2025-12-20T15:30:00.250Z | api-client | /var/cache/api-client/users.json | read | granted | -
2025-12-20T15:30:01.500Z | api-client | https://api.example.com/v1/products | connect | granted | -
2025-12-20T15:30:01.750Z | api-client | /var/cache/api-client/products.json | write | granted | -
2025-12-20T15:30:02.000Z | api-client | api-state/sync | write | granted | usage: 2MB/1GB
2025-12-20T15:30:02.100Z | api-client | https://webhook.example.com/completion | connect | granted | -
2025-12-20T15:30:02.350Z | api-client | https://health.example.com/status | connect | granted | -

No denials (expected behavior).
```

## Production Considerations

### Rate Limiting

Even with endpoint whitelisting, implement rate limiting:

```rust
// Component should track rate limits internally
struct RateLimiter {
    endpoints: HashMap<String, RateLimit>,
    // Track calls per minute per endpoint
}

// Example: 100 calls/minute per endpoint
impl RateLimiter {
    fn can_call(&mut self, endpoint: &str) -> bool {
        let limit = self.endpoints.entry(endpoint).or_insert_default();
        limit.can_call()
    }
}
```

### Connection Pooling

Reuse connections to reduce overhead:

```rust
struct ConnectionPool {
    connections: HashMap<String, Vec<NetworkConnection>>,
    max_per_endpoint: usize,
}

// Avoids frequent reconnections
// Respects capability checks (already done once per pool)
```

### Error Handling

Handle denied connections gracefully:

```rust
match host_connect(endpoint) {
    Ok(conn) => {
        // Use connection
    }
    Err(CapabilityDeniedError { reason }) => {
        // Log security incident
        log_security_alert("Unauthorized endpoint", endpoint, &reason);
        // Return error to caller
        return Err("Connection denied");
    }
    Err(NetworkError { .. }) => {
        // Network error (different from security denial)
        // Implement retry logic
    }
}
```

## Common Issues & Solutions

### Issue: "Endpoint not authorized" error

**Problem:** Component can't connect to needed API.

**Diagnosis:**
```
Error: https://api.example.com/v3/data - not in whitelist

Current configuration:
  endpoints = ["https://api.example.com/v1/*", "https://api.example.com/v2/*"]

Issue: API v3 not included
```

**Solution:** Update Component.toml:
```toml
[[component.capabilities.network]]
endpoints = [
    "https://api.example.com/v1/*",
    "https://api.example.com/v2/*",
    "https://api.example.com/v3/*"  # NEW
]
```

### Issue: Cache growing unbounded

**Problem:** Disk space consumed by cache.

**Solution 1:** Implement cache eviction
```rust
// Periodically clean old cache entries
fn evict_cache() {
    let cutoff = SystemTime::now() - Duration::days(7);
    // Remove entries older than 7 days
}
```

**Solution 2:** Add filesystem quota
```toml
# Ensure host function checks filesystem quota
# Prevents unbounded growth
```

### Issue: Storage quota exceeded

**Problem:** Component exceeds storage quota.

**Diagnosis:**
```
Error: Storage quota exceeded (1050MB > 1000MB)

Current usage:
  - api-state/sync: 100MB
  - api-state/cache: 800MB
  - api-state/archive: 150MB
```

**Solution:** Increase quota or clean up old state:
```toml
quota_gb = 2  # Increase to 2GB

# OR implement cleanup in component
fn cleanup_state() {
    // Delete api-state/archive entries older than 30 days
    // Frees ~100MB
}
```

## Next Steps

- Review [capability-declaration-guide.md](../capability-declaration-guide.md) for network patterns
- Read [security-best-practices.md](../security-best-practices.md) for endpoint design
- See [example-4-storage-isolated.md](example-4-storage-isolated.md) for storage patterns
