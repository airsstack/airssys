# Security Framework

This section documents the consolidated security middleware in AirsSys OSL.

## Current Status
**Implementation Phase**: ✅ **Implemented**  
**Module Location**: `middleware/security/` (consolidated approach)  
**RustDoc**: Run `cargo doc --open` in `airssys-osl` for complete API documentation

The security middleware provides comprehensive security controls through a consolidated architecture.

## Security Integration Pattern

All security concerns are handled within the security middleware:
- No separate `SecurityPolicy` trait - integrated into `SecurityMiddleware`
- Security middleware processes all operations before execution
- Unified security decision-making point

## Module Structure

```
middleware/security/
├── mod.rs          # Security middleware exports and orchestration
├── policy.rs       # Policy evaluation
├── acl.rs          # Access Control Lists implementation
├── rbac.rs         # Role-Based Access Control implementation
├── middleware.rs   # Core security middleware implementation
└── audit.rs        # Security audit logging
```

## Security Middleware

The main security middleware implementation:

```rust
use airssys_osl::middleware::security::SecurityMiddleware;

// Create ACL-based security
let acl_security = SecurityMiddleware::with_acl(acl_policy);

// Create RBAC-based security
let rbac_security = SecurityMiddleware::with_rbac(rbac_policy);

// Use with executor
let executor = FilesystemExecutor::new()
    .with_middleware(acl_security);
```

## Access Control Lists (ACL)

Fine-grained permission management:

```rust
use airssys_osl::middleware::security::acl::{AclPolicy, Permission};

let mut acl = AclPolicy::new();
acl.grant_permission("user1", "/data", Permission::Read);
acl.grant_permission("admin", "/data", Permission::ReadWrite);
```

## Role-Based Access Control (RBAC)

Role-based permission management:

```rust
use airssys_osl::middleware::security::rbac::{RbacPolicy, Role};

let mut rbac = RbacPolicy::new();
rbac.assign_role("user1", Role::Reader);
rbac.assign_role("admin", Role::Administrator);
```

## Security Audit

Comprehensive audit logging:

```rust
use airssys_osl::middleware::security::audit::AuditLogger;

let audit = AuditLogger::new();
// All security decisions are automatically logged
```

## Security Principles

The implementation follows these principles:
- **Deny by default**: All operations require explicit permission
- **Comprehensive auditing**: All system operations logged
- **Policy enforcement**: Runtime security policy validation
- **Threat detection**: Built-in detection of suspicious activities

## Examples

See the security examples:
- `examples/security_acl.rs` - ACL-based security
- `examples/security_rbac.rs` - RBAC-based security
- Security guide: [Security Setup Guide](../guides/security-setup.md)

For complete API documentation with all methods and configuration options, see the generated RustDoc (`cargo doc --open`).
