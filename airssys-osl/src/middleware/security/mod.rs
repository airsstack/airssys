//! Security middleware module for comprehensive security validation and policy enforcement.
//!
//! This module provides a complete security middleware implementation with policy-based
//! access control, security validation, and comprehensive audit logging. All security
//! enforcement happens in this middleware layer before operations reach executors.
//!
//! # Architecture
//!
//! The security middleware follows a layered architecture:
//!
//! - **Policy Framework** - Abstract policy evaluation engine with ACL and RBAC support
//! - **Access Control** - ACL (Access Control Lists) and RBAC (Role-Based Access Control)
//! - **Audit Logging** - Comprehensive security event logging and audit trails
//! - **Middleware Integration** - SecurityMiddleware implementing the Middleware trait
//!
//! # Security Model
//!
//! ## Deny-by-Default Philosophy
//!
//! The security middleware implements a **deny-by-default** model where all operations are
//! denied unless explicitly allowed by at least one security policy. This is a foundational
//! security principle that prevents unauthorized access by default.
//!
//! **Key Principles:**
//! - **Explicit Allow Required**: Operations must be explicitly permitted by policy
//! - **Deny-Wins Semantics**: If ANY policy denies, the entire operation is denied
//! - **No Implicit Trust**: Empty policies result in denial, not allowance
//! - **Defense in Depth**: Multiple policies can enforce different security aspects
//!
//! ## Policy Evaluation Flow
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ 1. SecurityMiddleware::before_execution()                   │
//! │    - Receives Operation + ExecutionContext                  │
//! │    - Extracts security attributes (principal, resource, etc)│
//! └────────────────────────┬────────────────────────────────────┘
//!                          │
//!                          ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │ 2. For Each SecurityPolicy (ACL, RBAC, Custom)              │
//! │    - policy.evaluate(context, attributes)                   │
//! │    - Returns PolicyDecision { allowed, reason }             │
//! └────────────────────────┬────────────────────────────────────┘
//!                          │
//!                          ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │ 3. Aggregate Policy Decisions                               │
//! │    - ANY Deny = Overall Deny (deny-wins)                    │
//! │    - ALL Allow = Overall Allow                              │
//! │    - NO Decisions = Deny (deny-by-default)                  │
//! └────────────────────────┬────────────────────────────────────┘
//!                          │
//!                          ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │ 4. Audit Logging                                            │
//! │    - Log security decision (allow/deny)                     │
//! │    - Include principal, resource, policies evaluated        │
//! │    - Record timestamp and context metadata                  │
//! └────────────────────────┬────────────────────────────────────┘
//!                          │
//!                          ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │ 5. Decision Enforcement                                     │
//! │    - Allow: Continue to next middleware/executor            │
//! │    - Deny: Return MiddlewareError::SecurityPolicyViolation  │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Priority 100: First in Middleware Pipeline
//!
//! The security middleware runs with **priority 100** (highest), ensuring security validation
//! occurs BEFORE any other middleware or operation execution. This prevents:
//!
//! - Resource consumption by unauthorized operations
//! - Information leakage through error messages from later middleware
//! - Time-of-check-time-of-use (TOCTOU) vulnerabilities
//! - Bypass through middleware ordering manipulation
//!
//! # Quick Start
//!
//! ## Basic Setup with ACL
//!
//! ```rust,no_run
//! use airssys_osl::middleware::security::{
//!     SecurityMiddleware, SecurityMiddlewareBuilder,
//!     AccessControlList, AclEntry, AclPolicy,
//! };
//! use airssys_osl::core::security::SecurityConfig;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Configure ACL for file access control
//! let acl = AccessControlList::new()
//!     .add_entry(AclEntry::new(
//!         "user_alice".to_string(),           // Principal
//!         "/home/alice/*".to_string(),         // Resource pattern
//!         vec!["file:read".to_string()],       // Permissions
//!         AclPolicy::Allow,                    // Policy decision
//!     ))
//!     .add_entry(AclEntry::new(
//!         "admin".to_string(),
//!         "/etc/*".to_string(),
//!         vec!["*".to_string()],               // All permissions
//!         AclPolicy::Allow,
//!     ));
//!
//! // Build security middleware with ACL policy
//! let security = SecurityMiddlewareBuilder::new()
//!     .with_config(SecurityConfig::default())
//!     .add_policy(Box::new(acl))
//!     .build()?;
//!
//! // Use in middleware pipeline (priority 100 - runs first)
//! // framework.middleware_pipeline().add(security);
//! # Ok(())
//! # }
//! ```
//!
//! ## Setup with RBAC
//!
//! ```rust,no_run
//! use airssys_osl::middleware::security::{
//!     SecurityMiddlewareBuilder, RoleBasedAccessControl, Role,
//! };
//! use airssys_osl::core::security::SecurityConfig;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Configure RBAC with role hierarchy
//! let mut rbac = RoleBasedAccessControl::new();
//!
//! // Define roles with permissions
//! rbac = rbac.add_role(
//!     Role::new("admin".to_string(), "System Administrator".to_string())
//!         .with_permission("system:*".to_string())
//! );
//! rbac = rbac.add_role(
//!     Role::new("operator".to_string(), "System Operator".to_string())
//!         .with_permission("process:spawn".to_string())
//!         .with_permission("process:kill".to_string())
//!         .inherits_from("user".to_string())  // Role inheritance
//! );
//! rbac = rbac.add_role(
//!     Role::new("user".to_string(), "Regular User".to_string())
//!         .with_permission("file:read".to_string())
//! );
//!
//! // Assign roles to principals
//! rbac = rbac.assign_roles("alice".to_string(), vec!["admin".to_string()]);
//! rbac = rbac.assign_roles("bob".to_string(), vec!["operator".to_string()]);
//!
//! // Build security middleware with RBAC policy
//! let security = SecurityMiddlewareBuilder::new()
//!     .with_config(SecurityConfig::default())
//!     .add_policy(Box::new(rbac))
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Combined ACL + RBAC
//!
//! ```rust,no_run
//! use airssys_osl::middleware::security::{
//!     SecurityMiddlewareBuilder, AccessControlList, AclEntry, AclPolicy,
//!     RoleBasedAccessControl, Role,
//! };
//! use airssys_osl::core::security::SecurityConfig;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Configure ACL for resource-based access
//! let acl = AccessControlList::new()
//!     .add_entry(AclEntry::new(
//!         "alice".to_string(),
//!         "/sensitive/*".to_string(),
//!         vec!["file:read".to_string()],
//!         AclPolicy::Deny,  // Explicit deny
//!     ));
//!
//! // Configure RBAC for permission-based access
//! let mut rbac = RoleBasedAccessControl::new();
//! rbac = rbac.add_role(
//!     Role::new("admin".to_string(), "Admin".to_string())
//!         .with_permission("system:*".to_string())
//! );
//! rbac = rbac.assign_roles("alice".to_string(), vec!["admin".to_string()]);
//!
//! // Build with BOTH policies (deny-wins: ACL deny overrides RBAC allow)
//! let security = SecurityMiddlewareBuilder::new()
//!     .with_config(SecurityConfig::default())
//!     .add_policy(Box::new(acl))    // Evaluated first
//!     .add_policy(Box::new(rbac))   // Evaluated second
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! # Security Audit Logging
//!
//! ## Audit Log Format
//!
//! All security decisions are logged with comprehensive context for compliance and forensics:
//!
//! ```json
//! {
//!   "timestamp": "2025-01-15T10:30:45.123456Z",
//!   "event_type": "AccessDenied",
//!   "principal": "user_alice",
//!   "resource": "/etc/shadow",
//!   "permission": "file:read",
//!   "decision": "Deny",
//!   "reason": "ACL policy denied access to /etc/shadow",
//!   "policies_evaluated": ["ACL", "RBAC"],
//!   "metadata": {
//!     "operation_type": "FileReadOperation",
//!     "source_ip": "192.168.1.100",
//!     "session_id": "sess_abc123"
//!   }
//! }
//! ```
//!
//! ## Audit Log Fields
//!
//! - **timestamp**: ISO 8601 UTC timestamp of the security event
//! - **event_type**: Type of security event (AccessGranted, AccessDenied, PolicyError, etc.)
//! - **principal**: Identity attempting the operation (from SecurityContext)
//! - **resource**: Resource being accessed (file path, network address, etc.)
//! - **permission**: Permission requested (file:read, process:spawn, etc.)
//! - **decision**: Final decision (Allow/Deny)
//! - **reason**: Human-readable explanation of the decision
//! - **policies_evaluated**: List of policies that evaluated the request
//! - **metadata**: Additional context (operation type, session info, etc.)
//!
//! ## Audit Log Consumption
//!
//! Audit logs are automatically generated during security policy evaluation.
//! The logs can be consumed from the standard output or integrated with
//! external logging systems.
//!
//! Example audit log output:
//! ```text
//! [SECURITY AUDIT] {
//!   "timestamp": "2025-10-10T10:30:45.123456Z",
//!   "event_type": "AccessDenied",
//!   "operation_id": "filesystem:abc123",
//!   "principal": "bob",
//!   "session_id": "sess_xyz",
//!   "decision": "Deny: ACL default policy denies access",
//!   "policy_applied": "Access Control List (ACL) Policy",
//!   "metadata": null
//! }
//! ```
//!
//! # Threat Model and Security Boundaries
//!
//! ## Threats Mitigated
//!
//! The security middleware protects against the following threat classes:
//!
//! ### 1. Unauthorized Access
//! - **Permission Escalation**: Users attempting to access resources beyond their privileges
//! - **Resource Bypass**: Accessing resources not covered by security policies
//! - **Role Violations**: Operations requiring roles not assigned to the principal
//! - **Identity Spoofing**: Empty or invalid principal identities
//!
//! ### 2. Policy Exploitation
//! - **Wildcard Pattern Abuse**: Exploiting glob patterns to access unintended resources
//! - **Permission String Manipulation**: Attempting to manipulate permission identifiers
//! - **Multi-Policy Conflicts**: Hoping one policy allows when another denies (deny-wins enforced)
//! - **Default Policy Bypass**: Attempting operations without configured policies (deny-by-default)
//!
//! ### 3. System Integrity
//! - **Circular Role Dependencies**: DoS through role inheritance loops (detected and denied)
//! - **Process Privilege Escalation**: Regular users spawning privileged processes
//! - **Network Authorization**: Unauthorized network socket operations
//!
//! ### 4. Configuration Attacks
//! - **ACL Default Policy Override**: Explicit denies always win over default allows
//! - **Permission Wildcard Confusion**: Wildcard permissions require explicit ACL configuration
//!
//! ## Threats Out of Scope
//!
//! The following threats are NOT mitigated by this middleware and require other security controls:
//!
//! - **Cryptographic Attacks**: No encryption/decryption or key management
//! - **Network Protocol Attacks**: No DDoS protection, no TLS/SSL termination
//! - **Memory Safety**: Relies on Rust's memory safety guarantees
//! - **Side-Channel Attacks**: No timing attack mitigation or constant-time operations
//! - **Physical Attacks**: No protection against physical access to systems
//! - **Social Engineering**: No user behavior analysis or anomaly detection
//! - **Supply Chain**: No validation of external dependencies or code integrity
//!
//! ## Security Assumptions
//!
//! The security middleware operates under these assumptions:
//!
//! 1. **Trusted Runtime Environment**: The Rust runtime and OS kernel are trusted
//! 2. **Correct Policy Configuration**: Security policies are correctly configured by administrators
//! 3. **Secure Principal Identity**: Principal identities are securely established (e.g., via authentication)
//! 4. **Immutable Policies**: Policies are not modified during security evaluation
//! 5. **Audit Log Integrity**: Audit logs are stored in tamper-resistant storage
//!
//! ## Attack Surface Analysis
//!
//! ### Trust Boundaries
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ Untrusted: User Input, External Operations                  │
//! ├─────────────────────────────────────────────────────────────┤
//! │ SecurityMiddleware (Trust Boundary Enforcement)             │
//! ├─────────────────────────────────────────────────────────────┤
//! │ Trusted: Executors, System Calls, Internal Logic            │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ### Critical Components
//!
//! - **Policy Evaluation Engine**: Must correctly implement deny-by-default and deny-wins
//! - **Pattern Matching (glob)**: Must not allow path traversal or pattern exploitation
//! - **Role Inheritance**: Must detect circular dependencies and prevent infinite loops
//! - **Audit Logger**: Must reliably record all security decisions for forensics
//!
//! # Security Testing Guidelines
//!
//! ## Writing Security Tests
//!
//! Security tests should validate both positive and negative scenarios:
//!
//! ```rust,no_run
//! #[tokio::test]
//! async fn test_security_scenario() {
//!     // 1. Setup: Configure security policies
//!     let acl = AccessControlList::new()
//!         .add_entry(AclEntry::new(
//!             "allowed_user".to_string(),
//!             "/public/*".to_string(),
//!             vec!["file:read".to_string()],
//!             AclPolicy::Allow,
//!         ));
//!
//!     let middleware = SecurityMiddlewareBuilder::new()
//!         .with_config(SecurityConfig::default())
//!         .add_policy(Box::new(acl))
//!         .build()
//!         .expect("Middleware build failed");
//!
//!     // 2. Positive Test: Allowed access should succeed
//!     let operation = FileReadOperation::new("/public/data.txt".to_string());
//!     let mut context = ExecutionContext::new(
//!         SecurityContext::new("allowed_user".to_string())
//!     );
//!     context.security_context.attributes.insert(
//!         "resource".to_string(), "/public/data.txt".to_string()
//!     );
//!     context.security_context.attributes.insert(
//!         "permission".to_string(), "file:read".to_string()
//!     );
//!
//!     let result = middleware.before_execution(operation, &context).await;
//!     assert!(result.is_ok(), "Allowed access should succeed");
//!
//!     // 3. Negative Test: Unauthorized access should be denied
//!     let operation2 = FileReadOperation::new("/public/data.txt".to_string());
//!     let mut context2 = ExecutionContext::new(
//!         SecurityContext::new("unauthorized_user".to_string())
//!     );
//!     context2.security_context.attributes.insert(
//!         "resource".to_string(), "/public/data.txt".to_string()
//!     );
//!     context2.security_context.attributes.insert(
//!         "permission".to_string(), "file:read".to_string()
//!     );
//!
//!     let result2 = middleware.before_execution(operation2, &context2).await;
//!     assert!(result2.is_err(), "Unauthorized access should be denied");
//! }
//! ```
//!
//! ## Threat Modeling Approach
//!
//! Follow this systematic approach for security threat modeling:
//!
//! ### 1. Identify Assets
//! - What resources need protection? (files, processes, network sockets)
//! - What operations are security-sensitive? (read, write, execute, listen)
//!
//! ### 2. Identify Threats
//! - **Spoofing**: Can an attacker impersonate a legitimate user?
//! - **Tampering**: Can an attacker modify data or policies?
//! - **Repudiation**: Can an attacker deny performing an action?
//! - **Information Disclosure**: Can an attacker access unauthorized information?
//! - **Denial of Service**: Can an attacker prevent legitimate access?
//! - **Elevation of Privilege**: Can an attacker gain higher privileges?
//!
//! ### 3. Determine Countermeasures
//! - Which security policies apply? (ACL, RBAC, custom)
//! - What is the expected security decision? (allow/deny)
//! - What should be logged? (all security decisions)
//!
//! ### 4. Write Threat Tests
//! - Create test for each identified threat scenario
//! - Validate both the deny decision and audit logging
//! - Test edge cases (empty principals, circular roles, etc.)
//!
//! ## Penetration Testing Preparation
//!
//! Before penetration testing, ensure:
//!
//! 1. **Complete Test Coverage**: All threat scenarios have corresponding tests
//! 2. **Audit Logging Enabled**: All security events are logged
//! 3. **Monitoring Setup**: Real-time alerts for suspicious patterns
//! 4. **Policy Review**: Security policies are correctly configured
//! 5. **Boundary Testing**: Trust boundaries are clearly defined and enforced
//!
//! # Custom SecurityPolicy Implementation
//!
//! Implement custom security policies for domain-specific requirements:
//!
//! ```rust,no_run
//! use airssys_osl::middleware::security::policy::{SecurityPolicy, PolicyDecision, PolicyScope};
//! use airssys_osl::core::context::SecurityContext;
//!
//! /// Time-based access control policy (e.g., business hours only)
//! #[derive(Debug)]
//! pub struct TimeBasedPolicy {
//!     allowed_hours: (u32, u32),  // (start_hour, end_hour)
//! }
//!
//! impl SecurityPolicy for TimeBasedPolicy {
//!     fn evaluate(&self, _context: &SecurityContext) -> PolicyDecision {
//!         use chrono::Timelike;
//!         let now = chrono::Utc::now();
//!         let hour = now.hour();
//!
//!         if hour >= self.allowed_hours.0 && hour < self.allowed_hours.1 {
//!             PolicyDecision::Allow
//!         } else {
//!             PolicyDecision::Deny(
//!                 format!("Access outside business hours (current: {}:00)", hour)
//!             )
//!         }
//!     }
//!
//!     fn description(&self) -> &str {
//!         "Time-Based Access Control Policy"
//!     }
//!
//!     fn scope(&self) -> PolicyScope {
//!         PolicyScope::All
//!     }
//! }
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Use custom policy with security middleware
//! use airssys_osl::middleware::security::SecurityMiddlewareBuilder;
//! use airssys_osl::core::security::SecurityConfig;
//!
//! let time_policy = TimeBasedPolicy {
//!     allowed_hours: (9, 17),  // 9 AM to 5 PM
//! };
//!
//! let security = SecurityMiddlewareBuilder::new()
//!     .with_config(SecurityConfig::default())
//!     .add_policy(Box::new(time_policy))
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!

//!
//! # Module Organization
//!
//! This module follows workspace standard §4.3 - mod.rs files contain ONLY
//! module declarations and re-exports, with implementation in separate files:
//!
//! - [`policy`] - Security policy framework and evaluation engine
//! - [`acl`] - Access Control Lists implementation
//! - [`rbac`] - Role-Based Access Control implementation
//! - [`audit`] - Security audit logging framework
//! - [`middleware`] - SecurityMiddleware implementation
//!
//! # See Also
//!
//! - **Integration Tests**: `tests/security_middleware_tests.rs` - SecurityMiddleware integration
//! - **Threat Model Tests**: `tests/security_threat_tests.rs` - Comprehensive threat scenarios
//! - **ACL Tests**: `tests/acl_tests.rs` - Access Control List validation
//! - **RBAC Tests**: `tests/rbac_tests.rs` - Role-Based Access Control validation
//! - **Microsoft Rust Guidelines**: Security patterns following M-MOCKABLE-SYSCALLS, M-ERRORS-CANONICAL-STRUCTS

// Layer 3: Internal module imports
// (none for this module - only declarations)

// Module declarations (§4.3 - ONLY declarations in mod.rs)
pub mod acl;
pub mod audit;
pub mod middleware;
pub mod policy;
pub mod rbac;

// Re-export primary types for ergonomic imports
pub use acl::{AccessControlList, AclEntry, AclPolicy};
pub use audit::{SecurityAuditLog, SecurityAuditLogger, SecurityEventType};
pub use middleware::{SecurityMiddleware, SecurityMiddlewareBuilder};
pub use policy::{AuthRequirement, PolicyDecision, PolicyScope, SecurityPolicy};
pub use rbac::{Permission, PermissionId, Role, RoleBasedAccessControl, RoleId, UserId};
