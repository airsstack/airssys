# Troubleshooting Security Issues

## Common Security Errors & Solutions

This reference guide covers 20+ common security issues, their causes, and solutions.

---

## Category 1: Capability Denials (8 errors)

### Error: "Access denied: pattern mismatch"

**Symptoms:**
- Host function returns error
- Component cannot access expected resource
- Audit log shows: `denied | Pattern mismatch`

**Common Causes:**
1. Path doesn't match declared pattern
2. Pattern uses wrong wildcard type
3. File is outside pattern boundary

**Diagnosis:**
```bash
# Check audit log
grep "denied.*Pattern mismatch" audit.log

# Example:
# Component requested: /app/data/subdir/file.log
# Pattern declared: /app/data/*.log (glob doesn't cross directories)
# Match: FAILED (glob stops at first /)
```

**Solutions:**

1. **Fix: Use recursive wildcard for nested directories**
   ```toml
   # ❌ WRONG: Glob doesn't cross / boundaries
   paths = ["/app/data/*.log"]
   
   # ✓ CORRECT: Recursive wildcard for nested
   paths = ["/app/data/**/*.log"]
   ```

2. **Fix: Verify path is in declared pattern**
   ```toml
   # Check what's actually requested vs declared
   # Requested: /var/log/myapp/2024/12/error.log
   # Declared patterns:
   #   - /var/log/myapp/*.log (WRONG - no nested)
   #   - /var/log/myapp/**/*.log (CORRECT)
   ```

3. **Fix: Add missing pattern**
   ```toml
   [[component.capabilities.filesystem]]
   paths = [
       "/app/data/*.json",          # Already declared
       "/app/config/prod.toml"      # NEW: Add exact file
   ]
   ```

---

### Error: "Permission denied: write not in [read]"

**Symptoms:**
- Component tries to modify files
- Returns permission denied error
- Component declared read-only

**Cause:**
- Declared only read permission
- Component attempting write operation

**Diagnosis:**
```bash
# Audit log shows:
# 2025-12-20T15:30:00Z | component | /app/config/file.json | write | denied | write not in [read]
```

**Solutions:**

1. **Fix: Add write permission if needed**
   ```toml
   [[component.capabilities.filesystem]]
   paths = ["/app/data/*"]
   permissions = ["read", "write"]  # Add write
   ```

2. **Fix: Don't request write if not needed**
   ```toml
   # ✓ CORRECT: Read-only (least privilege)
   [[component.capabilities.filesystem]]
   paths = ["/app/config/*"]
   permissions = ["read"]
   ```

3. **Fix: Separate read and write paths**
   ```toml
   # Read from config (read-only)
   [[component.capabilities.filesystem]]
   paths = ["/app/config/*"]
   permissions = ["read"]
   
   # Write to logs (write-only)
   [[component.capabilities.filesystem]]
   paths = ["/var/log/myapp/*"]
   permissions = ["write"]
   ```

---

### Error: "Access denied: endpoint not in whitelist"

**Symptoms:**
- Network connection fails
- Component tries to connect to service
- Audit: `denied | Not in whitelist`

**Common Causes:**
1. Endpoint not in declared network capabilities
2. Domain is correct but path is wrong
3. Using different API version

**Diagnosis:**
```bash
# Audit log:
# 2025-12-20T15:30:00Z | component | https://api.example.com/v3/users | connect | denied | Not in whitelist

# Check current declaration:
# endpoints = ["https://api.example.com/v1/*", "https://api.example.com/v2/*"]
# Issue: v3 not declared!
```

**Solutions:**

1. **Fix: Add endpoint to whitelist**
   ```toml
   [[component.capabilities.network]]
   endpoints = [
       "https://api.example.com/v1/*",
       "https://api.example.com/v2/*",
       "https://api.example.com/v3/*"  # NEW
   ]
   ```

2. **Fix: Update to correct API version**
   ```toml
   # If migrating from v1 to v2:
   endpoints = [
       "https://api.example.com/v2/*"  # Update version
   ]
   ```

3. **Fix: Add backup/failover endpoint**
   ```toml
   endpoints = [
       "https://api.primary.com/v1/*",
       "https://api.backup.com/v1/*"    # NEW: Failover
   ]
   ```

---

### Error: "Access denied: namespace not in capabilities"

**Symptoms:**
- Storage operation fails
- Component cannot read/write data
- Audit: `denied | Namespace not in capabilities`

**Cause:**
- Storage namespace not declared
- Multi-tenant component accessing wrong namespace

**Diagnosis:**
```bash
# Audit log:
# 2025-12-20T15:30:00Z | pipeline | tenant-456/data | read | denied | Namespace not in capabilities

# Current declaration:
# namespaces = ["tenant-123/*"]
# Issue: Only tenant-123 declared, not tenant-456
```

**Solutions:**

1. **Fix: Use glob pattern for multiple tenants**
   ```toml
   [[component.capabilities.storage]]
   namespaces = [
       "tenant-*/*"  # Matches all tenants
   ]
   ```

2. **Fix: Add specific namespace**
   ```toml
   namespaces = [
       "tenant-123/*",
       "tenant-456/*"  # NEW: Add tenant
   ]
   ```

3. **Fix: Verify tenant context**
   ```rust
   // In host function, validate tenant matches
   let expected_tenant = get_tenant_from_context()?;
   let actual_tenant = extract_tenant_from_namespace(&namespace)?;
   
   if expected_tenant != actual_tenant {
       return Err("Tenant mismatch");
   }
   ```

---

### Error: "Permission denied: read not in [write]"

**Symptoms:**
- Component tries to read data
- Returns permission denied
- Audit: `denied | read not in [write]`

**Cause:**
- Component declared write-only
- Component attempting read operation

**Solutions:**

1. **Fix: Add read permission if needed**
   ```toml
   [[component.capabilities.filesystem]]
   paths = ["/app/logs/*"]
   permissions = ["read", "write"]  # Add read
   ```

2. **Fix: Separate read and write by path**
   ```toml
   # Write-only logs
   [[component.capabilities.filesystem]]
   paths = ["/app/logs/*"]
   permissions = ["write"]
   
   # Read-only configuration
   [[component.capabilities.filesystem]]
   paths = ["/app/config/*"]
   permissions = ["read"]
   ```

---

### Error: "Access denied: operation not allowed for resource"

**Symptoms:**
- Filesystem operation fails
- Component tries unsupported operation
- Audit: `denied | Operation not allowed`

**Common Causes:**
1. Component tries execute without permission
2. Operation type not supported

**Solutions:**

1. **Fix: Add execute permission if needed**
   ```toml
   [[component.capabilities.filesystem]]
   paths = ["/app/bin/*"]
   permissions = ["read", "execute"]  # Add execute
   ```

2. **Fix: Don't execute unless necessary**
   ```toml
   # ✓ CORRECT: Only needed permissions
   [[component.capabilities.filesystem]]
   paths = ["/app/data/*"]
   permissions = ["read"]  # No execute needed
   ```

---

### Error: "Access denied: custom capability not found"

**Symptoms:**
- Custom capability check fails
- Component tries access with undeclared custom capability
- Audit: `denied | Custom capability not found`

**Cause:**
- Custom resource_type not declared

**Diagnosis:**
```bash
# Component attempted: database operation
# Error: resource_type "database" not found

# Check Component.toml:
# [[component.capabilities.custom]]
# resource_type = "payment_processor"  # WRONG resource_type
```

**Solutions:**

1. **Fix: Declare custom capability**
   ```toml
   [[component.capabilities.custom]]
   resource_type = "database"
   actions = ["query_read", "query_write"]
   ```

2. **Fix: Verify resource_type spelling**
   ```toml
   # ❌ WRONG: "databse" (typo)
   resource_type = "databse"
   
   # ✓ CORRECT: "database"
   resource_type = "database"
   ```

---

## Category 2: Trust & Approval Issues (5 errors)

### Error: "Component requires approval"

**Symptoms:**
- Component installation blocked
- Waiting for security team review
- Status: TrustLevel::Unknown

**Cause:**
- Component from unknown source
- Trust configuration doesn't match source

**Diagnosis:**
```
Component source: https://github.com/third-party/component
Trust configuration:
  - urls = ["https://github.com/myorg/*"]
Match: NO → TrustLevel::Unknown
```

**Solutions:**

1. **Fix: Add to trusted sources (if appropriate)**
   ```toml
   [[trust.sources]]
   type = "git"
   url_pattern = "https://github.com/third-party/*"
   description = "Third-party vendor"
   ```

2. **Fix: Request approval from security team**
   - Contact security-review@company.com
   - Provide Component.toml for review
   - Wait for approval decision

3. **Fix: Use locally developed alternative**
   - Create internal component
   - Add to trusted org repository
   - Instant deployment

---

### Error: "DevMode detected - security bypassed"

**Symptoms:**
- Warnings printed: "⚠️ ⚠️ ⚠️ DEVELOPMENT MODE ACTIVE"
- Security checks bypassed
- Message says: "DO NOT use in production!"

**Cause:**
- DevMode enabled in trust configuration
- Likely misconfiguration in production

**Diagnosis:**
```toml
# In production-config.toml (WRONG):
[trust]
dev_mode = true  # ⚠️ CRITICAL: Should be false!
```

**Solutions:**

1. **Fix: Disable DevMode in production**
   ```toml
   # production-config.toml
   [trust]
   dev_mode = false  # ✓ CORRECT for production
   ```

2. **Fix: Use separate config files**
   ```bash
   # development-config.toml
   [trust]
   dev_mode = true
   
   # production-config.toml
   [trust]
   dev_mode = false
   ```

3. **Fix: Verify correct config is loaded**
   ```bash
   # Check which config file is active
   $ env | grep TRUST_CONFIG
   TRUST_CONFIG=/etc/production-config.toml  # ✓ Correct
   
   # NOT:
   TRUST_CONFIG=/etc/development-config.toml  # ❌ Wrong!
   ```

---

### Error: "Approval expired"

**Symptoms:**
- Component previously approved, now blocked
- Error: `Approval expired: valid until 2025-12-20`
- Current date is past expiration

**Cause:**
- Approval has validity period
- Period has elapsed

**Diagnosis:**
```
Approval granted: 2024-12-20
Approval valid until: 2025-12-20
Current date: 2025-12-21
Status: EXPIRED
```

**Solutions:**

1. **Fix: Request re-approval**
   - Contact security team
   - Request new approval (usually quick if no changes)
   - Will be valid for another year

2. **Fix: Make component Trusted**
   ```toml
   [[trust.sources]]
   type = "git"
   url_pattern = "https://github.com/vendor/component"
   description = "Vendor component (approved for internal use)"
   ```

---

### Error: "Trust source revoked"

**Symptoms:**
- Component previously Trusted now Unknown
- Trust source removed from configuration
- Installation now requires approval

**Cause:**
- Trust source removed from trust-config.toml
- Organization decision to stop trusting source

**Diagnosis:**
```
Component source: https://github.com/old-vendor/component
Previous trust: YES
Current trust: NO (revoked)
Status: TrustLevel::Unknown
```

**Solutions:**

1. **Fix: Add source back if decision was wrong**
   ```toml
   [[trust.sources]]
   type = "git"
   url_pattern = "https://github.com/old-vendor/*"
   description = "Vendor (trust restored)"
   ```

2. **Fix: Request approval for revoked source**
   - If revocation was intentional
   - Component now requires case-by-case approval

---

### Error: "Multiple approvers required but only 1 approved"

**Symptoms:**
- Component awaiting approval
- One team member approved
- Component still blocked
- Message: "Awaiting approval from 1 more reviewer"

**Cause:**
- Approval workflow requires multiple approvals
- Only one person approved so far

**Solutions:**

1. **Fix: Get additional approvals**
   - Follow escalation path
   - Contact second required approver
   - They will approve/reject

2. **Fix: Lower approval requirement (if appropriate)**
   ```toml
   [trust.approval]
   require_approvals = 1  # Change from 2 to 1
   ```

---

## Category 3: Quota Issues (4 errors)

### Error: "Quota exceeded"

**Symptoms:**
- Storage write fails
- Error: `Quota exceeded: 1050MB > 1000MB`
- Component cannot continue

**Cause:**
- Operation would exceed allocated quota
- Storage already near limit

**Diagnosis:**
```
Allocated quota: 1GB
Current usage: 900MB
Requested: 200MB
Total if allowed: 1100MB > 1000MB ✗
```

**Solutions:**

1. **Fix: Increase quota**
   ```toml
   [[component.capabilities.storage]]
   namespaces = ["cache/*"]
   quota_gb = 2  # Increase from 1 to 2
   ```

2. **Fix: Implement cleanup in component**
   ```rust
   pub fn cleanup_old_data(days: u32) -> Result<u64> {
       let cutoff = SystemTime::now() - Duration::days(days);
       let mut freed = 0;
       
       for entry in list_storage()? {
           if entry.modified < cutoff {
               freed += delete_storage_entry(&entry)?;
           }
       }
       
       Ok(freed)
   }
   ```

3. **Fix: Use separate quota per tenant**
   ```toml
   # Instead of shared 5GB quota:
   # Create separate component instances per tenant
   # Each gets 5GB quota
   ```

---

### Error: "Rate limit exceeded"

**Symptoms:**
- Repeated denials for same resource
- Error: `Rate limit exceeded: 150 ops/sec > 100 ops/sec`
- Component requests too frequently

**Cause:**
- Component making excessive requests
- Exceeding configured rate limit

**Solutions:**

1. **Fix: Implement backoff in component**
   ```rust
   let mut retry_count = 0;
   loop {
       match check_capability() {
           Ok(_) => break,
           Err(RateLimitError) if retry_count < 3 => {
               let backoff_ms = 100 * 2_u32.pow(retry_count);
               tokio::time::sleep(Duration::from_millis(backoff_ms as u64)).await;
               retry_count += 1;
           }
           Err(e) => return Err(e),
       }
   }
   ```

2. **Fix: Batch operations**
   ```rust
   // Instead of 100 individual writes
   let mut batch = Vec::new();
   for item in items {
       batch.push(item);
       if batch.len() >= 10 {
           write_batch(&batch)?;
           batch.clear();
       }
   }
   ```

3. **Fix: Increase rate limit**
   ```toml
   [quotas]
   operations_per_second = 200  # Increase from 100
   ```

---

### Error: "Filesystem quota exceeded"

**Symptoms:**
- Filesystem write fails
- Error: `Filesystem quota exceeded: 5GB used, 5GB limit`
- Component cannot write more files

**Cause:**
- Too many files written to filesystem
- Exceeds quota allocation

**Solutions:**

1. **Fix: Increase filesystem quota**
   ```toml
   # Add filesystem quota if not present
   # Or increase existing quota
   ```

2. **Fix: Clean up old files**
   ```rust
   // Delete files older than 7 days
   let cutoff = SystemTime::now() - Duration::days(7);
   for entry in list_files(&pattern)? {
       if entry.modified < cutoff {
           delete_file(&entry)?;
       }
   }
   ```

3. **Fix: Move to storage namespace**
   ```toml
   # Instead of /tmp filesystem quota
   # Use storage namespace (persistent, quotaed separately)
   ```

---

### Error: "Network bandwidth exceeded"

**Symptoms:**
- Network operations slow or blocked
- Error: `Bandwidth quota exceeded: 100MB/min > 50MB/min`
- Component cannot send more data

**Cause:**
- Sending data too fast
- Exceeds network quota

**Solutions:**

1. **Fix: Implement rate limiting**
   ```rust
   struct BandwidthLimiter {
       bytes_sent: u64,
       window_start: Instant,
       limit_bytes_per_sec: u64,
   }
   ```

2. **Fix: Compress data before sending**
   ```rust
   // Reduce bandwidth usage through compression
   let compressed = compress_data(&data)?;
   send_network(&compressed)?;
   ```

3. **Fix: Batch API requests**
   ```rust
   // Send 100 items in 1 API call instead of 100 calls
   api_client.batch_create(items)?;
   ```

---

## Category 4: Pattern Matching Issues (2 errors)

### Error: "Invalid glob pattern"

**Symptoms:**
- Component.toml validation fails
- Error: `Invalid pattern: /app/[invalid].json`
- Component won't load

**Cause:**
- Malformed glob pattern syntax
- Invalid special characters

**Diagnosis:**
```toml
# Invalid patterns:
paths = ["/app/[a-z]/*.json"]  # Glob doesn't support ranges
paths = ["/app/{a,b}/*.json"]  # Doesn't support alternation
paths = ["/app/?.json"]        # ? is not supported
```

**Solutions:**

1. **Fix: Use supported glob syntax**
   ```toml
   # ✓ CORRECT: Only *, **, exact paths
   paths = [
       "/app/data/*.json",         # Glob
       "/app/logs/**/*.log",       # Recursive
       "/app/config/exact.toml"    # Exact
   ]
   ```

2. **Fix: List each path separately**
   ```toml
   # Instead of: [a-z]
   paths = [
       "/app/admin.json",
       "/app/api.json",
       "/app/auth.json",
       # ... etc
   ]
   ```

---

### Error: "Pattern too broad"

**Symptoms:**
- Component.toml rejected at validation
- Error: `Pattern too broad: / matches entire filesystem`
- Pattern flagged as security risk

**Cause:**
- Pattern is too general
- Could match system-critical paths

**Solutions:**

1. **Fix: Confine pattern to application directory**
   ```toml
   # ❌ WRONG: Matches entire filesystem
   paths = ["/"]
   
   # ✓ CORRECT: Limited to app
   paths = ["/app/*"]
   ```

2. **Fix: Use specific file extensions**
   ```toml
   # ❌ WRONG: Matches all files
   paths = ["/app/**"]
   
   # ✓ CORRECT: Specific types
   paths = ["/app/**/*.json", "/app/**/*.log"]
   ```

3. **Fix: Separate by directory**
   ```toml
   paths = [
       "/app/config/*.toml",
       "/app/data/*.csv",
       "/app/logs/*.log"
   ]
   ```

---

## Category 5: Configuration Issues (2 errors)

### Error: "Component.toml not found"

**Symptoms:**
- Component fails to load
- Error: `Component.toml not found in /path/to/component`
- No capabilities declared

**Cause:**
- Component.toml missing
- Wrong file path
- File misspelled

**Diagnosis:**
```bash
$ ls -la /path/to/component/
# Result: No Component.toml found

# Possible locations checked:
# - ./Component.toml
# - ./component.toml (wrong case)
# - ./config/component.toml (wrong location)
```

**Solutions:**

1. **Fix: Create Component.toml in component root**
   ```bash
   $ cat > Component.toml << 'EOF'
   [component]
   name = "my-component"
   version = "1.0.0"
   
   [[component.capabilities.filesystem]]
   paths = ["/app/data/*"]
   permissions = ["read"]
   EOF
   ```

2. **Fix: Verify file name is exactly "Component.toml"**
   ```bash
   # ❌ WRONG: Lower case
   component.toml
   
   # ✓ CORRECT: Title case
   Component.toml
   ```

---

### Error: "Syntax error in Component.toml"

**Symptoms:**
- Component fails to parse
- Error: `Syntax error at line 15: unexpected token`
- Component won't load

**Cause:**
- Invalid TOML syntax
- Missing quotes or brackets

**Diagnosis:**
```toml
# Line 15 has error:
[[component.capabilities.filesystem]]
paths = [/app/data/*.json]  # ❌ Missing quotes!
permissions = ["read"]
```

**Solutions:**

1. **Fix: Add missing quotes**
   ```toml
   # ❌ WRONG: No quotes
   paths = [/app/data/*.json]
   
   # ✓ CORRECT: Quoted strings
   paths = ["/app/data/*.json"]
   ```

2. **Fix: Validate TOML syntax**
   ```bash
   # Use online TOML validator or tool
   $ cat Component.toml | toml-lint
   # or use: https://toml.io/en/v1.0.0#valid
   ```

3. **Fix: Check for missing commas**
   ```toml
   # ❌ WRONG: Missing comma between entries
   [[component.capabilities.filesystem]]
   paths = ["/app/data/*"]
   [[component.capabilities.network]]  # Needs comma after previous section
   
   # ✓ CORRECT:
   [[component.capabilities.filesystem]]
   paths = ["/app/data/*"]
   
   [[component.capabilities.network]]
   endpoints = ["https://api.example.com/*"]
   ```

---

## Audit Log Interpretation

### Understanding Audit Entries

Each audit log entry has this format:
```
timestamp | component_id | resource | permission | result | details

Example:
2025-12-20T15:30:00.001Z | my-component | /app/data/file.json | read | granted | -
2025-12-20T15:30:01.000Z | my-component | /etc/passwd | read | denied | Pattern mismatch
```

**Fields:**
- **timestamp**: When the operation occurred (UTC)
- **component_id**: Component being checked
- **resource**: File path, endpoint, namespace, etc.
- **permission**: Operation type (read, write, connect, etc.)
- **result**: granted or denied
- **details**: Reason if denied, quota usage if relevant

### Reading Denials

```bash
# Find all denials
grep " denied " audit.log

# Find denials for specific component
grep "my-component.*denied" audit.log

# Find specific denial type
grep "denied.*pattern" audit.log
grep "denied.*permission" audit.log
grep "denied.*quota" audit.log
```

### Monitoring for Attacks

```bash
# Look for suspicious patterns
grep "/etc\|/root\|/home" audit.log | grep "granted"  # Unexpected system access

grep "denied.*traversal" audit.log                      # Path traversal attempts

grep "denied.*privilege" audit.log                      # Privilege escalation

grep "quota.*exceeded" audit.log                        # Resource exhaustion
```

---

## Getting Help

If your issue isn't covered here:

1. **Check audit logs**
   ```bash
   grep -i "component-name" audit.log | tail -50
   ```

2. **Enable debug logging**
   ```toml
   [logging]
   level = "debug"
   security_details = true
   ```

3. **Contact security team**
   - Email: security-review@company.com
   - Attach: audit logs, Component.toml, error messages

---

## References

- **Capability Declaration**: [capability-declaration-guide.md](capability-declaration-guide.md)
- **Trust Configuration**: [trust-configuration-guide.md](trust-configuration-guide.md)
- **Best Practices**: [security-best-practices.md](security-best-practices.md)
- **Architecture**: [security-architecture.md](security-architecture.md)
- **Host Integration**: [host-integration-guide.md](host-integration-guide.md)
