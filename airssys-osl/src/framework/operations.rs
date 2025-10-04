//! Operation builders for framework ergonomic APIs.
//!
//! This module contains operation builders that provide fluent interfaces for
//! constructing and executing operations through the framework.
//!
//! Phase 1 provides the foundational structure. Full implementation with
//! operation construction and execution will be completed in Phase 3.

use super::OSLFramework;

/// Builder for filesystem operations.
///
/// Provides a fluent interface for constructing filesystem operations like
/// reading and writing files. Full implementation coming in Phase 3.
///
/// # Examples
///
/// ```no_run
/// use airssys_osl::prelude::*;
///
/// # async fn example() -> OSResult<()> {
/// let osl = OSLFramework::builder()
///     .with_default_security()
///     .build().await?;
///
/// let fs_builder = osl.filesystem();
/// // Phase 3 will implement: fs_builder.read_file("/path").execute().await?
/// # Ok(())
/// # }
/// ```
pub struct FilesystemBuilder<'a> {
    _framework: &'a OSLFramework,
}

impl<'a> FilesystemBuilder<'a> {
    /// Create a new filesystem builder.
    ///
    /// This is typically called through `OSLFramework::filesystem()` rather
    /// than directly.
    pub(crate) fn new(framework: &'a OSLFramework) -> Self {
        Self {
            _framework: framework,
        }
    }

    // TODO Phase 3: Implement operation construction methods:
    // - read_file()
    // - write_file()
    // - create_directory()
    // - delete_file()
    // - with_permissions()
    // - execute()
}

/// Builder for process operations.
///
/// Provides a fluent interface for constructing process operations like
/// spawning and managing processes. Full implementation coming in Phase 3.
///
/// # Examples
///
/// ```no_run
/// use airssys_osl::prelude::*;
///
/// # async fn example() -> OSResult<()> {
/// let osl = OSLFramework::builder()
///     .with_default_security()
///     .build().await?;
///
/// let proc_builder = osl.process();
/// // Phase 3 will implement: proc_builder.spawn("command").execute().await?
/// # Ok(())
/// # }
/// ```
pub struct ProcessBuilder<'a> {
    _framework: &'a OSLFramework,
}

impl<'a> ProcessBuilder<'a> {
    /// Create a new process builder.
    ///
    /// This is typically called through `OSLFramework::process()` rather
    /// than directly.
    pub(crate) fn new(framework: &'a OSLFramework) -> Self {
        Self {
            _framework: framework,
        }
    }

    // TODO Phase 3: Implement operation construction methods:
    // - spawn()
    // - kill()
    // - with_environment()
    // - with_working_directory()
    // - execute()
}

/// Builder for network operations.
///
/// Provides a fluent interface for constructing network operations like
/// creating sockets and connections. Full implementation coming in Phase 3.
///
/// # Examples
///
/// ```no_run
/// use airssys_osl::prelude::*;
///
/// # async fn example() -> OSResult<()> {
/// let osl = OSLFramework::builder()
///     .with_default_security()
///     .build().await?;
///
/// let net_builder = osl.network();
/// // Phase 3 will implement: net_builder.connect("host:port").execute().await?
/// # Ok(())
/// # }
/// ```
pub struct NetworkBuilder<'a> {
    _framework: &'a OSLFramework,
}

impl<'a> NetworkBuilder<'a> {
    /// Create a new network builder.
    ///
    /// This is typically called through `OSLFramework::network()` rather
    /// than directly.
    pub(crate) fn new(framework: &'a OSLFramework) -> Self {
        Self {
            _framework: framework,
        }
    }

    // TODO Phase 3: Implement operation construction methods:
    // - connect()
    // - listen()
    // - with_timeout()
    // - execute()
}

// TODO Phase 3: The following will be completed:
// - Full implementation of operation builders with fluent APIs
// - Execute methods for each builder type
// - Integration with middleware pipeline
// - Automatic security context propagation
