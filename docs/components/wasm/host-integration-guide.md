# Host Function Integration Guide

## Overview

**Host functions** are the bridge between WASM components and the host system. They handle resource access (filesystem, network, storage) while enforcing security checks.

This guide explains how to implement host functions with proper capability checking, audit logging, and error handling.

---

## Architecture Overview

```
WASM Component
    ↓
Host Function Call
    ↓
Capability Check
    ├─ Lookup component security context
    ├─ Match resource against capabilities
    ├─ Verify permission matches request
    └─ Return Allow/Deny
        ↓
    [ALLOW]
        ↓
    Audit Log: GRANTED
        ↓
    Perform Operation
        ↓
    Return Result
    
    [DENY]
        ↓
    Audit Log: DENIED
        ↓
    Return Error to Component
```

---

## Security Requirements for Host Functions

All host functions MUST:

1. **Check Capability First**
   - Before accessing any resource
   - Even if component is trusted

2. **Normalize Paths**
   - Resolve `..` and `.` components
   - Canonicalize to absolute path
   - Prevent path traversal attacks

3. **Audit Log All Operations**
   - Grant: Log successful access
   - Deny: Log rejected access with reason
   - Quota: Log usage after operation

4. **Handle Errors Gracefully**
   - Return meaningful error messages
   - Don't expose system details
   - Log security incidents

5. **Respect Quotas**
   - Check quota before operation
   - Track usage after operation
   - Return quota exceeded error if needed

---

## Basic Host Function Pattern

### Template Structure

```rust
pub fn host_operation(
    component_id: &str,
    resource: &str,
    permission: &str,
) -> Result<OperationResult> {
    // 1. Validate input parameters
    let resource = validate_resource_path(resource)?;
    
    // 2. Check capability
    check_capability(component_id, &resource, permission)?;
    
    // 3. Verify resource exists/is valid
    verify_resource_exists(&resource)?;
    
    // 4. Check quotas
    check_quota(component_id, &resource, permission)?;
    
    // 5. Perform operation
    let result = perform_operation(&resource, permission)?;
    
    // 6. Update quotas
    update_quota(component_id, &resource, &result)?;
    
    // 7. Audit log
    audit_logger.log_access_granted(component_id, &resource, permission);
    
    // 8. Return result
    Ok(result)
}
```

---

## Filesystem Host Functions

### File Read Operation

```rust
/// Host function: Read file contents
pub fn host_read_file(
    component_id: &str,
    path: &str,
) -> Result<Vec<u8>, HostError> {
    // 1. Normalize the path
    let normalized = match std::fs::canonicalize(path) {
        Ok(p) => p,
        Err(e) => {
            audit_logger.log_error(
                component_id,
                "read",
                path,
                &format!("Path normalization failed: {}", e),
            );
            return Err(HostError::InvalidPath(path.to_string()));
        }
    };
    
    let normalized_str = normalized.to_string_lossy();
    
    // 2. Check capability
    match check_capability(component_id, &normalized_str, "read") {
        Ok(_) => {},
        Err(e) => {
            audit_logger.log_access_denied(
                component_id,
                &normalized_str,
                "read",
                &format!("{:?}", e),
            );
            return Err(HostError::CapabilityDenied(
                format!("Cannot read {}: {}", path, e)
            ));
        }
    }
    
    // 3. Check file exists
    if !normalized.exists() {
        audit_logger.log_error(
            component_id,
            "read",
            &normalized_str,
            "File not found",
        );
        return Err(HostError::NotFound(path.to_string()));
    }
    
    // 4. Check it's a regular file (not directory)
    if !normalized.is_file() {
        audit_logger.log_error(
            component_id,
            "read",
            &normalized_str,
            "Not a regular file",
        );
        return Err(HostError::NotAFile(path.to_string()));
    }
    
    // 5. Read the file
    let data = match std::fs::read(&normalized) {
        Ok(d) => d,
        Err(e) => {
            audit_logger.log_error(
                component_id,
                "read",
                &normalized_str,
                &format!("Read failed: {}", e),
            );
            return Err(HostError::Io(e));
        }
    };
    
    // 6. Audit success
    audit_logger.log_access_granted(
        component_id,
        &normalized_str,
        "read",
    );
    
    Ok(data)
}
```

### File Write Operation

```rust
/// Host function: Write file contents
pub fn host_write_file(
    component_id: &str,
    path: &str,
    data: &[u8],
) -> Result<u64, HostError> {
    // 1. Normalize path
    let normalized = match std::fs::canonicalize_parent(path) {
        Ok(p) => p,
        Err(e) => {
            audit_logger.log_error(
                component_id,
                "write",
                path,
                &format!("Path normalization failed: {}", e),
            );
            return Err(HostError::InvalidPath(path.to_string()));
        }
    };
    
    let normalized_str = normalized.to_string_lossy();
    
    // 2. Check capability
    match check_capability(component_id, &normalized_str, "write") {
        Ok(_) => {},
        Err(e) => {
            audit_logger.log_access_denied(
                component_id,
                &normalized_str,
                "write",
                &format!("{:?}", e),
            );
            return Err(HostError::CapabilityDenied(
                format!("Cannot write {}: {}", path, e)
            ));
        }
    }
    
    // 3. Check quota
    let data_size = data.len() as u64;
    match check_quota(component_id, data_size) {
        Ok(_) => {},
        Err(e) => {
            audit_logger.log_quota_exceeded(
                component_id,
                &normalized_str,
                data_size,
            );
            return Err(HostError::QuotaExceeded(format!(
                "Write would exceed quota: {}",
                e
            )));
        }
    }
    
    // 4. Create parent directory if needed
    if let Some(parent) = normalized.parent() {
        if !parent.exists() {
            match std::fs::create_dir_all(parent) {
                Ok(_) => {},
                Err(e) => {
                    audit_logger.log_error(
                        component_id,
                        "write",
                        &normalized_str,
                        &format!("Failed to create parent: {}", e),
                    );
                    return Err(HostError::Io(e));
                }
            }
        }
    }
    
    // 5. Write file
    match std::fs::write(&normalized, data) {
        Ok(_) => {},
        Err(e) => {
            audit_logger.log_error(
                component_id,
                "write",
                &normalized_str,
                &format!("Write failed: {}", e),
            );
            return Err(HostError::Io(e));
        }
    }
    
    // 6. Update quota
    update_quota(component_id, data_size)?;
    
    // 7. Audit success (including usage)
    audit_logger.log_access_granted_with_quota(
        component_id,
        &normalized_str,
        "write",
        data_size,
    );
    
    Ok(data_size)
}
```

---

## Network Host Functions

### Network Connection

```rust
/// Host function: Establish network connection
pub fn host_connect(
    component_id: &str,
    endpoint: &str,
    timeout_ms: u32,
) -> Result<NetworkConnection, HostError> {
    // 1. Parse endpoint URL
    let url = match Url::parse(endpoint) {
        Ok(u) => u,
        Err(e) => {
            audit_logger.log_error(
                component_id,
                "connect",
                endpoint,
                &format!("Invalid URL: {}", e),
            );
            return Err(HostError::InvalidUrl(endpoint.to_string()));
        }
    };
    
    // 2. Check capability
    match check_capability(component_id, endpoint, "connect") {
        Ok(_) => {},
        Err(e) => {
            audit_logger.log_access_denied(
                component_id,
                endpoint,
                "connect",
                &format!("{:?}", e),
            );
            return Err(HostError::CapabilityDenied(
                format!("Cannot connect to {}: {}", endpoint, e)
            ));
        }
    }
    
    // 3. Check network quota
    match check_network_quota(component_id) {
        Ok(_) => {},
        Err(e) => {
            audit_logger.log_quota_exceeded(
                component_id,
                endpoint,
                1,
            );
            return Err(HostError::QuotaExceeded(format!(
                "Network quota exceeded: {}",
                e
            )));
        }
    }
    
    // 4. Validate URL scheme
    match url.scheme() {
        "https" => {},  // Allow HTTPS
        "http" => {
            // Warn but allow HTTP (consider blocking in production)
            audit_logger.log_warning(
                component_id,
                "connect",
                endpoint,
                "Insecure HTTP connection",
            );
        }
        _ => {
            audit_logger.log_error(
                component_id,
                "connect",
                endpoint,
                &format!("Unsupported scheme: {}", url.scheme()),
            );
            return Err(HostError::InvalidScheme(url.scheme().to_string()));
        }
    }
    
    // 5. Establish connection with timeout
    let timeout = Duration::from_millis(timeout_ms as u64);
    match tokio::time::timeout(timeout, establish_connection(&url)) {
        Ok(Ok(conn)) => {
            // 6. Audit success
            audit_logger.log_access_granted(
                component_id,
                endpoint,
                "connect",
            );
            
            Ok(conn)
        }
        Ok(Err(e)) => {
            audit_logger.log_error(
                component_id,
                "connect",
                endpoint,
                &format!("Connection failed: {}", e),
            );
            Err(HostError::ConnectionFailed(e.to_string()))
        }
        Err(_) => {
            audit_logger.log_error(
                component_id,
                "connect",
                endpoint,
                &format!("Connection timeout ({}ms)", timeout_ms),
            );
            Err(HostError::Timeout)
        }
    }
}
```

---

## Storage Host Functions

### Storage Write

```rust
/// Host function: Write to persistent storage
pub fn host_storage_write(
    component_id: &str,
    namespace: &str,
    key: &str,
    data: &[u8],
) -> Result<u64, HostError> {
    // 1. Build full path
    let full_path = format!("{}/{}", namespace, key);
    
    // 2. Validate namespace/key
    if namespace.is_empty() || key.is_empty() {
        return Err(HostError::InvalidNamespace(
            "Namespace and key cannot be empty".to_string()
        ));
    }
    
    // 3. Check capability
    match check_capability(component_id, &full_path, "write") {
        Ok(_) => {},
        Err(e) => {
            audit_logger.log_access_denied(
                component_id,
                &full_path,
                "write",
                &format!("{:?}", e),
            );
            return Err(HostError::CapabilityDenied(format!(
                "Cannot write to {}: {}",
                full_path, e
            )));
        }
    }
    
    // 4. Check storage quota
    let data_size = data.len() as u64;
    match check_storage_quota(component_id, data_size) {
        Ok(_) => {},
        Err(e) => {
            audit_logger.log_quota_exceeded(
                component_id,
                &full_path,
                data_size,
            );
            return Err(HostError::QuotaExceeded(format!(
                "Write would exceed quota: {}",
                e
            )));
        }
    }
    
    // 5. Write to storage backend
    match storage_backend.write(&full_path, data) {
        Ok(_) => {},
        Err(e) => {
            audit_logger.log_error(
                component_id,
                "write",
                &full_path,
                &format!("Storage write failed: {}", e),
            );
            return Err(HostError::StorageError(e.to_string()));
        }
    }
    
    // 6. Update quota tracking
    update_storage_quota(component_id, data_size)?;
    
    // 7. Get current quota usage for audit log
    let current_usage = get_storage_usage(component_id)?;
    let quota_limit = get_storage_quota_limit(component_id)?;
    
    // 8. Audit success with quota information
    audit_logger.log_access_granted_with_quota(
        component_id,
        &full_path,
        "write",
        current_usage,
        quota_limit,
    );
    
    Ok(data_size)
}
```

### Storage Read

```rust
/// Host function: Read from persistent storage
pub fn host_storage_read(
    component_id: &str,
    namespace: &str,
    key: &str,
) -> Result<Vec<u8>, HostError> {
    // 1. Build full path
    let full_path = format!("{}/{}", namespace, key);
    
    // 2. Validate namespace/key
    if namespace.is_empty() || key.is_empty() {
        return Err(HostError::InvalidNamespace(
            "Namespace and key cannot be empty".to_string()
        ));
    }
    
    // 3. Check capability
    match check_capability(component_id, &full_path, "read") {
        Ok(_) => {},
        Err(e) => {
            audit_logger.log_access_denied(
                component_id,
                &full_path,
                "read",
                &format!("{:?}", e),
            );
            return Err(HostError::CapabilityDenied(format!(
                "Cannot read from {}: {}",
                full_path, e
            )));
        }
    }
    
    // 4. Check storage exists
    if !storage_backend.exists(&full_path)? {
        audit_logger.log_error(
            component_id,
            "read",
            &full_path,
            "Storage entry not found",
        );
        return Err(HostError::NotFound(full_path));
    }
    
    // 5. Read from storage backend
    match storage_backend.read(&full_path) {
        Ok(data) => {
            // 6. Audit success
            audit_logger.log_access_granted(
                component_id,
                &full_path,
                "read",
            );
            
            Ok(data)
        }
        Err(e) => {
            audit_logger.log_error(
                component_id,
                "read",
                &full_path,
                &format!("Storage read failed: {}", e),
            );
            Err(HostError::StorageError(e.to_string()))
        }
    }
}
```

---

## Error Handling Patterns

### Safe Error Messages

```rust
// ❌ BAD: Exposes system details
fn host_read_file_bad(component_id: &str, path: &str) -> Result<Vec<u8>> {
    std::fs::read(path)
        .map_err(|e| format!("Failed to read {}: {}", path, e.kind()))
}

// ✓ GOOD: Sanitized error message
fn host_read_file_good(component_id: &str, path: &str) -> Result<Vec<u8>> {
    check_capability(component_id, path, "read")?;
    
    std::fs::read(path)
        .map_err(|_| HostError::AccessDenied(
            "Cannot read file".to_string()
        ))
}
```

### Error Recovery

```rust
// Implement retry logic for transient failures
pub fn host_operation_with_retry<F, T>(
    component_id: &str,
    max_retries: u32,
    mut operation: F,
) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    for attempt in 0..max_retries {
        match operation() {
            Ok(result) => return Ok(result),
            Err(e) if should_retry(&e) && attempt < max_retries - 1 => {
                let backoff_ms = 100 * 2_u32.pow(attempt);
                std::thread::sleep(Duration::from_millis(backoff_ms as u64));
                continue;
            }
            Err(e) => return Err(e),
        }
    }
    
    Err(HostError::MaxRetriesExceeded)
}

fn should_retry(error: &HostError) -> bool {
    matches!(error, HostError::Timeout | HostError::ConnectionFailed(_))
}
```

---

## Audit Logging Patterns

### Complete Audit Log Entry

```rust
pub fn log_complete_operation(
    component_id: &str,
    resource: &str,
    operation: &str,
    result: &str,
    quota_usage: Option<(u64, u64)>,  // (current, limit)
) {
    let entry = OperationLog {
        timestamp: Utc::now(),
        component_id: component_id.to_string(),
        resource: resource.to_string(),
        operation: operation.to_string(),
        result: result.to_string(),
        details: match quota_usage {
            Some((current, limit)) => {
                format!("usage: {}MB/{}MB", current / 1024 / 1024, limit / 1024 / 1024)
            }
            None => "-".to_string(),
        },
    };
    
    audit_logger.log(&entry);
}
```

### Structured Logging

```rust
// ✓ GOOD: Structured, parseable logs
log_entry {
    timestamp: "2025-12-20T15:30:00.001Z",
    component_id: "my-component",
    resource: "/app/data/file.json",
    operation: "read",
    result: "granted",
    details: "-"
}

// ❌ BAD: Unstructured, hard to parse
"2025-12-20 15:30:00 - my-component read /app/data/file.json - success"
```

---

## Performance Optimization

### Capability Check Caching

```rust
// Cache recent capability checks (time-based TTL)
struct CapabilityCache {
    cache: Arc<DashMap<String, (bool, Instant)>>,
    ttl: Duration,
}

impl CapabilityCache {
    pub fn check_with_cache(
        &self,
        component_id: &str,
        resource: &str,
        perm: &str,
    ) -> Result<bool> {
        let key = format!("{}:{}:{}", component_id, resource, perm);
        
        // Check cache first
        if let Some(entry) = self.cache.get(&key) {
            let (result, timestamp) = entry.value();
            if timestamp.elapsed() < self.ttl {
                return Ok(*result);
            }
        }
        
        // Cache miss or expired: re-check
        let result = check_capability(component_id, resource, perm).is_ok();
        self.cache.insert(key, (result, Instant::now()));
        
        Ok(result)
    }
}
```

### Batch Operations

```rust
// Batch multiple operations for efficiency
pub fn host_write_batch(
    component_id: &str,
    writes: Vec<(String, Vec<u8>)>,
) -> Result<Vec<u64>> {
    let mut results = Vec::new();
    
    // Single capability check for all writes
    let total_size: u64 = writes.iter().map(|(_, data)| data.len() as u64).sum();
    check_quota(component_id, total_size)?;
    
    // Perform all writes
    for (path, data) in writes {
        let size = host_write_file(component_id, &path, &data)?;
        results.push(size);
    }
    
    Ok(results)
}
```

---

## Best Practices

### ✓ Always Normalize Paths

```rust
// CRITICAL: Always normalize before checking
let normalized = std::fs::canonicalize(path)?;
check_capability(component_id, &normalized.to_string_lossy(), "read")?;
```

### ✓ Check Before Operating

```rust
// Check FIRST, operate SECOND
check_capability(component_id, resource, permission)?;
perform_operation(resource)?;
```

### ✓ Separate Concerns

```rust
// Separate validation, capability, quota, operation
validate_input()?;
check_capability()?;
check_quota()?;
perform_operation()?;
update_quota()?;
audit_log()?;
```

### ✓ Comprehensive Error Context

```rust
// Return errors with context
Err(HostError::CapabilityDenied(format!(
    "Component {} cannot {} {} ({})",
    component_id, permission, resource, reason
)))
```

### ✓ Complete Audit Trails

```rust
// Log all decisions
audit.log_granted(component_id, resource, permission, quota);
audit.log_denied(component_id, resource, permission, reason);
audit.log_quota(component_id, current, limit);
```

---

## References

- **Architecture**: [security-architecture.md](security-architecture.md)
- **Capability Declaration**: [capability-declaration-guide.md](capability-declaration-guide.md)
- **Best Practices**: [security-best-practices.md](security-best-practices.md)
- **Troubleshooting**: [troubleshooting-security.md](troubleshooting-security.md)
