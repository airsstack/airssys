//! High-level convenience functions for common OS operations.
//!
//! This module provides **three API levels** for different use cases:
//!
//! # Level 1: Simple Functions (Recommended for Most Users)
//!
//! Quick, one-line operations with default security enforcement:
//!
//! ```rust,no_run
//! use airssys_osl::helpers::*;
//!
//! # async fn example() -> airssys_osl::core::result::OSResult<()> {
//! // Simple, secure by default
//! let data = read_file("/etc/hosts", "admin").await?;
//! write_file("/tmp/output.txt", data, "admin").await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Level 2: Custom Middleware (Advanced Users)
//!
//! Full control over security policies and custom middleware:
//!
//! ```rust,no_run
//! use airssys_osl::helpers::*;
//! use airssys_osl::middleware::security::*;
//!
//! # async fn example() -> airssys_osl::core::result::OSResult<()> {
//! // Custom ACL policy
//! let acl = AccessControlList::new()
//!     .add_entry(AclEntry::new(
//!         "alice".to_string(),
//!         "/data/*".to_string(),
//!         vec!["read".to_string()],
//!         AclPolicy::Allow
//!     ));
//!
//! let security = SecurityMiddlewareBuilder::new()
//!     .add_policy(Box::new(acl))
//!     .build();
//!
//! // Use with custom middleware (Phase 2-4: Implementation pending)
//! // let data = read_file_with_middleware("/data/secret.txt", "alice", security).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Level 3: Trait-Based Composition (Future - Phase 8-10)
//!
//! Type-safe builder pattern for complex operation chains:
//!
//! ```rust,ignore
//! use airssys_osl::helpers::*;
//!
//! // Future API - not yet implemented
//! FileOperation::read("/etc/hosts")
//!     .with_user("admin")
//!     .with_acl(acl_policy)
//!     .with_rbac(rbac_policy)
//!     .execute()
//!     .await?;
//! ```
//!
//! # Security Model
//!
//! All helper functions enforce **deny-by-default** security:
//!
//! - **Default SecurityMiddleware**: ACL + RBAC policies
//! - **ACL (Access Control List)**: Path-based access control
//! - **RBAC (Role-Based Access Control)**: Role-based permissions
//! - **Audit Logging**: All operations logged for security compliance
//!
//! ## Default Security Behavior
//!
//! ```rust,no_run
//! use airssys_osl::helpers::*;
//!
//! # async fn example() -> airssys_osl::core::result::OSResult<()> {
//! // Automatically uses default_security_middleware()
//! // which includes:
//! // - default_acl_policy() → PathACL with deny-by-default
//! // - default_rbac_policy() → PathRBAC with deny-by-default
//! let data = read_file("/etc/hosts", "admin").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Custom Security Policies
//!
//! Advanced users can bypass defaults with `*_with_middleware()` variants:
//!
//! ```rust,no_run
//! use airssys_osl::helpers::*;
//! use airssys_osl::middleware::security::*;
//!
//! # async fn example() -> airssys_osl::core::result::OSResult<()> {
//! // Create custom ACL policy
//! let acl = AccessControlList::new()
//!     .add_entry(AclEntry::new(
//!         "alice".to_string(),
//!         "/data/*".to_string(),
//!         vec!["read".to_string()],
//!         AclPolicy::Allow
//!     ))
//!     .add_entry(AclEntry::new(
//!         "bob".to_string(),
//!         "/data/secret/*".to_string(),
//!         vec!["read".to_string()],
//!         AclPolicy::Deny
//!     ));
//!
//! // Create custom RBAC policy
//! use airssys_osl::middleware::security::rbac::{Role, Permission};
//!
//! let read_permission = Permission::new(
//!     "read_file".to_string(),
//!     "Read File".to_string(),
//!     "Allows reading files".to_string()
//! );
//!
//! let data_reader_role = Role::new("data_reader".to_string(), "Data Reader".to_string())
//!     .with_permission("read_file".to_string());
//!
//! let rbac = RoleBasedAccessControl::new()
//!     .add_permission(read_permission)
//!     .add_role(data_reader_role)
//!     .assign_roles("alice".to_string(), vec!["data_reader".to_string()]);
//!
//! let security = SecurityMiddlewareBuilder::new()
//!     .add_policy(Box::new(acl))
//!     .add_policy(Box::new(rbac))
//!     .build();
//!
//! // Use custom middleware (Phase 2-4: Implementation pending)
//! // let data = read_file_with_middleware("/data/file.txt", "alice", security).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Available Operations
//!
//! ## Filesystem Operations
//! - [`read_file`] / `read_file_with_middleware` - Read file contents
//! - [`write_file`] / `write_file_with_middleware` - Write file contents
//! - [`delete_file`] / `delete_file_with_middleware` - Delete file
//! - [`create_directory`] / `create_directory_with_middleware` - Create directory
//!
//! ## Process Operations
//! - [`spawn_process`] / `spawn_process_with_middleware` - Spawn new process
//! - [`kill_process`] / `kill_process_with_middleware` - Kill process by PID
//! - [`send_signal`] / `send_signal_with_middleware` - Send signal to process
//!
//! ## Network Operations
//! - [`network_connect`] / `network_connect_with_middleware` - Connect to remote endpoint
//! - [`network_listen`] / `network_listen_with_middleware` - Listen on local endpoint
//! - [`create_socket`] / `create_socket_with_middleware` - Create network socket
//!
//! # Implementation Status
//!
//! - ✅ **Phase 1.1-1.2**: Module structure and middleware factories (COMPLETE)
//! - 🚧 **Phase 1.3-1.4**: Documentation and KNOW-013 alignment (IN PROGRESS)
//! - ⏳ **Phase 2-4**: Simple helper implementations with `*_with_middleware()` variants
//! - ⏳ **Phase 5-7**: Integration tests and custom middleware examples
//! - ⏳ **Phase 8-10**: Trait-based composition layer (Future)
//! - ⏳ **Phase 11**: Final validation and production readiness
//!
//! # References
//!
//! - **Task**: `.copilot/memory_bank/sub_projects/airssys-osl/tasks/detail/OSL-TASK-010.md`
//! - **Knowledge Base**: `.copilot/memory_bank/sub_projects/airssys-osl/docs/knowledges/KNOW-013.md`
//! - **Architecture**: `.copilot/memory_bank/sub_projects/airssys-osl/docs/adr/`
//!
//! [`Middleware<O>`]: crate::core::middleware::Middleware
//! [`OSResult<T>`]: crate::core::result::OSResult
//! [`OSError`]: crate::core::result::OSError
//! [`SecurityMiddleware`]: crate::middleware::security::SecurityMiddleware
//! [`Middleware`]: crate::core::middleware::Middleware
//! [`OSExecutor`]: crate::core::executor::OSExecutor
//! [`ExecutorExt`]: crate::middleware::ext::ExecutorExt
//! [`read_file`]: simple::read_file
//! [`write_file`]: simple::write_file
//! [`delete_file`]: simple::delete_file
//! [`create_directory`]: simple::create_directory
//! [`spawn_process`]: simple::spawn_process
//! [`kill_process`]: simple::kill_process
//! [`send_signal`]: simple::send_signal
//! [`network_connect`]: simple::network_connect
//! [`network_listen`]: simple::network_listen
//! [`create_socket`]: simple::create_socket

// ============================================================================
// Module Declarations (§4.3 Module Architecture - MANDATORY)
// ============================================================================

pub(crate) mod context; // Security context building utilities
pub(crate) mod factories;

// Module declarations for simple helpers and composition
pub(crate) mod simple; // Phase 2-4: Simple helper functions
pub mod composition; // Phase 8: Trait-based composition layer

// ============================================================================
// Re-exports (will be populated in later phases)
// ============================================================================

// Re-export simple helpers (Level 1 & 2)
pub use self::simple::*;

// Re-export composition layer (Level 3) - Phase 8
pub use self::composition::{
    ComposedHelper, FileHelper, HelperPipeline, NetworkHelper, ProcessHelper,
};
