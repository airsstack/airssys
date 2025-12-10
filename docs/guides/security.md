# Security Guide

Security best practices for AirsSys components.

## OSL Security Model

### Deny-by-Default

All operations are **denied by default** unless explicitly allowed by security policies.

```rust
// Without policy - operation denied
let result = read_file("/data/secret.txt", "user").await;
// Error: Access denied

// With policy - operation allowed
let acl = AccessControlList::new()
    .add_entry(AclEntry::new(
        "user".to_string(),
        "/data/*".to_string(),
        vec!["read".to_string()],
        AclPolicy::Allow,
    ));
```

### Access Control Lists (ACL)

Path-based access control with glob patterns:

```rust
use airssys_osl::middleware::security::*;

let acl = AccessControlList::new()
    // Allow read/write to /tmp
    .add_entry(AclEntry::new(
        "alice".to_string(),
        "/tmp/*".to_string(),
        vec!["read".to_string(), "write".to_string()],
        AclPolicy::Allow,
    ))
    // Allow read-only to /data
    .add_entry(AclEntry::new(
        "alice".to_string(),
        "/data/**".to_string(),
        vec!["read".to_string()],
        AclPolicy::Allow,
    ))
    // Explicitly deny sensitive files
    .add_entry(AclEntry::new(
        "alice".to_string(),
        "/data/sensitive/*".to_string(),
        vec!["read".to_string(), "write".to_string()],
        AclPolicy::Deny,  // Deny takes precedence
    ));
```

### Role-Based Access Control (RBAC)

Role hierarchies with permission inheritance:

```rust
let rbac = RoleBasedAccessControl::new()
    // Define roles
    .add_role("admin", vec!["read", "write", "delete", "execute"])
    .add_role("developer", vec!["read", "write", "execute"])
    .add_role("operator", vec!["read", "execute"])
    .add_role("viewer", vec!["read"])
    
    // Define role hierarchy (inheritance)
    .add_role_hierarchy("admin", "developer")
    .add_role_hierarchy("developer", "operator")
    .add_role_hierarchy("operator", "viewer");

// Assign user to role
rbac.assign_user("alice", "developer")?;

// alice inherits: read, write, execute (from developer)
//                 + read (from operator via hierarchy)
//                 + read (from viewer via hierarchy)
```

### Combining Policies

Multiple policies can be combined:

```rust
let security = SecurityMiddlewareBuilder::new()
    .add_policy(Box::new(acl))
    .add_policy(Box::new(rbac))
    .add_policy(Box::new(rate_limiter))
    .build()?;

// All policies must allow the operation
```

## RT Security Model

### Actor Isolation

Actors have private state enforced by Rust ownership:

```rust
struct BankAccount {
    balance: i64,  // Private, cannot be accessed by other actors
}

// Other actors CANNOT:
// - Read balance directly
// - Modify balance directly
// - Share references to balance

// Other actors CAN:
// - Send messages to request balance
// - Send messages to request transfers
```

### Message Validation

Validate messages before processing:

```rust
#[async_trait]
impl Actor for SecureActor {
    async fn handle_message<B: MessageBroker<Self::Message>>(
        &mut self,
        msg: Self::Message,
        _ctx: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        // Validate message
        if !self.validate_message(&msg) {
            return Err("Invalid message".into());
        }
        
        // Process message
        // ...
        Ok(())
    }
}
```

### Supervisor Security

Supervisors enforce fault isolation:

```rust
// Failed actor is isolated and restarted
// Failure does NOT propagate to siblings
let supervisor = SupervisorBuilder::new()
    .with_strategy(RestartStrategy::OneForOne)  // Isolate failures
    .with_max_restarts(3, Duration::from_secs(60))
    .build();
```

## Audit Logging

### OSL Audit Trails

All operations are logged:

```json
{
  "timestamp": "2025-12-10T10:30:00Z",
  "principal": "alice",
  "operation": "FileRead",
  "resource": "/data/customer_data.csv",
  "result": "allowed",
  "policy": "acl:data-read",
  "duration_ms": 15
}
```

Configure logging:

```rust
use airssys_osl::middleware::logger::*;

let logger = FileActivityLogger::new("/var/log/airssys/audit.log").await?;
let middleware = LoggerMiddleware::with_config(
    logger,
    LoggerConfig {
        log_success: true,
        log_failures: true,
        include_principal: true,
        include_duration: true,
    }
);
```

### RT Event Monitoring

Monitor actor events:

```rust
use airssys_rt::monitoring::*;

let monitor = InMemoryMonitor::new();

// Events logged:
// - Actor spawned
// - Actor stopped
// - Message sent
// - Message received
// - Supervision events
// - Health check results
```

## Security Best Practices

### Principle of Least Privilege

Grant minimum required permissions:

```rust
// ❌ Bad: Overly permissive
let acl = AccessControlList::new()
    .add_entry(AclEntry::new(
        "app".to_string(),
        "/**".to_string(),  // Access to everything!
        vec!["read".to_string(), "write".to_string(), "delete".to_string()],
        AclPolicy::Allow,
    ));

// ✅ Good: Specific permissions
let acl = AccessControlList::new()
    .add_entry(AclEntry::new(
        "app".to_string(),
        "/app/data/*".to_string(),  // Only app data
        vec!["read".to_string(), "write".to_string()],  // No delete
        AclPolicy::Allow,
    ));
```

### Defense in Depth

Layer multiple security mechanisms:

```rust
let security = SecurityMiddlewareBuilder::new()
    .add_policy(Box::new(acl))        // Path-based control
    .add_policy(Box::new(rbac))       // Role-based control
    .add_policy(Box::new(rate_limit)) // Rate limiting
    .add_policy(Box::new(time_based)) // Time-based access
    .build()?;
```

### Secure Defaults

Use secure defaults:

```rust
// Default security included in helpers
read_file(path, principal).await?;  // ✅ Security checked

// Explicit security for custom use
read_file_with_middleware(path, principal, security).await?;  // ✅ Custom policy
```

### Input Validation

Validate all inputs:

```rust
fn validate_path(path: &str) -> Result<(), Error> {
    // Check path traversal
    if path.contains("..") {
        return Err("Path traversal detected");
    }
    
    // Check absolute path
    if !path.starts_with("/") {
        return Err("Absolute path required");
    }
    
    // Check allowed directory
    if !path.starts_with("/app/data") {
        return Err("Path not in allowed directory");
    }
    
    Ok(())
}
```

### Error Handling

Don't leak sensitive information in errors:

```rust
// ❌ Bad: Leaks path details
return Err(format!("Failed to read {}", full_system_path));

// ✅ Good: Generic error
return Err("File operation failed");

// ✅ Good: Log details, return generic error
log::error!("Failed to read {}", full_system_path);
return Err("File operation failed");
```

## Compliance Considerations

### SOC 2 / HIPAA / GDPR

AirsSys provides features for compliance:

- **Audit logs**: Complete operation history
- **Access control**: Granular permissions
- **Data isolation**: Actor-based separation
- **Encryption**: Use with file encryption middleware
- **Retention**: Configurable log retention

Example compliance configuration:

```rust
// Audit logging for compliance
let audit_logger = FileActivityLogger::new("/audit/operations.log").await?;
let audit_middleware = LoggerMiddleware::with_config(
    audit_logger,
    LoggerConfig {
        log_success: true,
        log_failures: true,
        include_principal: true,
        include_duration: true,
    }
);

// Access control for data protection
let acl = AccessControlList::new()
    .add_entry(AclEntry::new(
        "medical_staff".to_string(),
        "/medical/records/**".to_string(),
        vec!["read".to_string()],
        AclPolicy::Allow,
    ));

// Combine for compliance
let security = SecurityMiddlewareBuilder::new()
    .add_middleware(audit_middleware)
    .add_policy(Box::new(acl))
    .build()?;
```

## Security Checklist

### Before Deployment

- [ ] Security policies configured
- [ ] Audit logging enabled
- [ ] Log files secured (permissions)
- [ ] Error messages don't leak sensitive data
- [ ] Input validation implemented
- [ ] Least privilege principle applied
- [ ] Security policies tested
- [ ] Failure scenarios tested

### During Operation

- [ ] Monitor audit logs
- [ ] Review access patterns
- [ ] Update security policies as needed
- [ ] Rotate log files
- [ ] Review and respond to security events

## Next Steps

- [Integration Guide](integration.md)
- [Performance Guide](performance.md)
- [OSL Security Reference](../components/osl/reference/security-practices.md)
