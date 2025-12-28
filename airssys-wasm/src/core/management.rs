//! Component registry and management abstractions.
//!
//! This module provides traits and types for component registration, querying,
//! and metadata management. These abstractions are used by Block 8 (Component
//! Manager) to maintain the component registry.
//!
//! # Architecture
//!
//! The management abstractions follow a repository pattern:
//!
//! - `ComponentRegistry` trait defines the contract for registry implementations
//! - `InstallationMetadata` tracks complete installation state
//! - `ComponentQuery` enables flexible filtering and search
//! - `RegistryOperation` enumerates all registry operations for auditing
//!
//! # Design Principles
//!
//! - **Trait-Based**: Registry behavior defined via traits for testability
//! - **Async First**: All operations are async for non-blocking I/O
//! - **Query Flexibility**: ComponentQuery supports multiple filter criteria
//! - **Complete Metadata**: InstallationMetadata captures all relevant state
//!
//! # References
//!
//! - **ADR-WASM-012**: Comprehensive Core Abstractions Strategy
//! - **Block 8**: Component Manager implementation
//! - **Workspace Standards**: §6.2 (Avoid dyn patterns - use generic constraints)

use std::path::PathBuf;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::core::lifecycle::{LifecycleState, VersionInfo};
use crate::core::{Capability, ComponentId, ComponentMetadata, InstallationSource, WasmResult};

/// Component registry trait for component management.
///
/// Defines the contract for component registry implementations. The registry
/// maintains the authoritative list of installed components and their metadata.
///
/// # Implementation Notes
///
/// Implementations should:
/// - Persist registry state across restarts
/// - Support concurrent access (use appropriate locking)
/// - Validate component IDs before registration
/// - Emit events for all registry modifications
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::management::ComponentRegistry;
/// use airssys_wasm::core::{ComponentId, ComponentMetadata};
///
/// // Example usage (requires an implementation)
/// async fn register_component<R: ComponentRegistry>(
///     registry: &mut R,
///     id: ComponentId,
///     metadata: ComponentMetadata,
/// ) {
///     registry.register(id, metadata).await.expect("Registration failed");
/// }
/// ```
#[async_trait]
pub trait ComponentRegistry: Send + Sync {
    /// Register a new component in the registry.
    ///
    /// # Errors
    ///
    /// Returns `WasmError::ComponentAlreadyExists` if a component with the
    /// same ID is already registered.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let id = ComponentId::new("image-processor");
    /// let metadata = ComponentMetadata { /* ... */ };
    /// registry.register(id, metadata).await?;
    /// ```
    async fn register(
        &mut self,
        component_id: ComponentId,
        metadata: ComponentMetadata,
    ) -> WasmResult<()>;

    /// Unregister a component from the registry.
    ///
    /// # Errors
    ///
    /// Returns `WasmError::ComponentNotFound` if the component doesn't exist.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let id = ComponentId::new("image-processor");
    /// registry.unregister(&id).await?;
    /// ```
    async fn unregister(&mut self, component_id: &ComponentId) -> WasmResult<()>;

    /// Get component metadata by ID.
    ///
    /// Returns `None` if the component is not found.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let id = ComponentId::new("image-processor");
    /// if let Some(metadata) = registry.get_metadata(&id).await? {
    ///     println!("Found component: {}", metadata.name);
    /// }
    /// ```
    async fn get_metadata(
        &self,
        component_id: &ComponentId,
    ) -> WasmResult<Option<ComponentMetadata>>;

    /// Query components by filter criteria.
    ///
    /// Returns all components matching the query criteria. An empty query
    /// returns all registered components.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use airssys_wasm::core::management::ComponentQuery;
    /// use airssys_wasm::core::lifecycle::LifecycleState;
    ///
    /// let query = ComponentQuery {
    ///     state: Some(LifecycleState::Running),
    ///     ..Default::default()
    /// };
    /// let running = registry.query(query).await?;
    /// println!("Found {} running components", running.len());
    /// ```
    async fn query(&self, query: ComponentQuery) -> WasmResult<Vec<ComponentMetadata>>;

    /// Update component metadata.
    ///
    /// Replaces the existing metadata for the component with the provided metadata.
    ///
    /// # Errors
    ///
    /// Returns `WasmError::ComponentNotFound` if the component doesn't exist.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let id = ComponentId::new("image-processor");
    /// let updated_metadata = ComponentMetadata { /* ... */ };
    /// registry.update_metadata(&id, updated_metadata).await?;
    /// ```
    async fn update_metadata(
        &mut self,
        component_id: &ComponentId,
        metadata: ComponentMetadata,
    ) -> WasmResult<()>;

    /// List all registered component IDs.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let all_ids = registry.list_component_ids().await?;
    /// for id in all_ids {
    ///     println!("Component: {}", id.as_str());
    /// }
    /// ```
    async fn list_component_ids(&self) -> WasmResult<Vec<ComponentId>>;
}

/// Installation metadata for installed components.
///
/// Captures complete state about a component installation including version,
/// source, installation time, filesystem location, and current lifecycle state.
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::management::InstallationMetadata;
/// use airssys_wasm::core::{ComponentId, InstallationSource};
/// use airssys_wasm::core::lifecycle::{VersionInfo, LifecycleState};
/// use std::path::PathBuf;
///
/// let metadata = InstallationMetadata {
///     component_id: ComponentId::new("my-component"),
///     version: VersionInfo::new("1.0.0", "abc123"),
///     source: InstallationSource::File {
///         path: PathBuf::from("/path/to/component.wasm"),
///     },
///     installed_at: chrono::Utc::now(),
///     install_path: PathBuf::from("/var/lib/components/my-component"),
///     state: LifecycleState::Installed,
/// };
///
/// assert_eq!(metadata.component_id.as_str(), "my-component");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationMetadata {
    /// Component identifier
    pub component_id: ComponentId,

    /// Version information with hash and signature
    pub version: VersionInfo,

    /// Installation source (Git, File, or URL)
    pub source: InstallationSource,

    /// When the component was installed
    pub installed_at: DateTime<Utc>,

    /// Filesystem path where component is installed
    pub install_path: PathBuf,

    /// Current lifecycle state
    pub state: LifecycleState,
}

impl InstallationMetadata {
    /// Create new installation metadata.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::management::InstallationMetadata;
    /// use airssys_wasm::core::{ComponentId, InstallationSource};
    /// use airssys_wasm::core::lifecycle::{VersionInfo, LifecycleState};
    /// use std::path::PathBuf;
    ///
    /// let metadata = InstallationMetadata::new(
    ///     ComponentId::new("test"),
    ///     VersionInfo::new("1.0.0", "hash123"),
    ///     InstallationSource::File { path: PathBuf::from("/test.wasm") },
    ///     PathBuf::from("/install"),
    /// );
    ///
    /// assert_eq!(metadata.state, LifecycleState::Installed);
    /// ```
    pub fn new(
        component_id: ComponentId,
        version: VersionInfo,
        source: InstallationSource,
        install_path: PathBuf,
    ) -> Self {
        Self {
            component_id,
            version,
            source,
            installed_at: Utc::now(),
            install_path,
            state: LifecycleState::Installed,
        }
    }

    /// Update the lifecycle state.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::management::InstallationMetadata;
    /// use airssys_wasm::core::{ComponentId, InstallationSource};
    /// use airssys_wasm::core::lifecycle::{VersionInfo, LifecycleState};
    /// use std::path::PathBuf;
    ///
    /// let mut metadata = InstallationMetadata::new(
    ///     ComponentId::new("test"),
    ///     VersionInfo::new("1.0.0", "hash123"),
    ///     InstallationSource::File { path: PathBuf::from("/test.wasm") },
    ///     PathBuf::from("/install"),
    /// );
    ///
    /// metadata.set_state(LifecycleState::Running);
    /// assert_eq!(metadata.state, LifecycleState::Running);
    /// ```
    pub fn set_state(&mut self, state: LifecycleState) {
        self.state = state;
    }

    /// Check if the component is currently active.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::management::InstallationMetadata;
    /// use airssys_wasm::core::{ComponentId, InstallationSource};
    /// use airssys_wasm::core::lifecycle::{VersionInfo, LifecycleState};
    /// use std::path::PathBuf;
    ///
    /// let mut metadata = InstallationMetadata::new(
    ///     ComponentId::new("test"),
    ///     VersionInfo::new("1.0.0", "hash123"),
    ///     InstallationSource::File { path: PathBuf::from("/test.wasm") },
    ///     PathBuf::from("/install"),
    /// );
    ///
    /// assert!(!metadata.is_active());
    /// metadata.set_state(LifecycleState::Running);
    /// assert!(metadata.is_active());
    /// ```
    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }
}

/// Component query for filtering components.
///
/// Supports filtering by name pattern, lifecycle state, installation time,
/// and required capabilities. All criteria are combined with AND logic.
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::management::ComponentQuery;
/// use airssys_wasm::core::lifecycle::LifecycleState;
/// use airssys_wasm::core::Capability;
///
/// // Query for running components installed in the last hour
/// let query = ComponentQuery {
///     name_pattern: None,
///     state: Some(LifecycleState::Running),
///     installed_after: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
///     has_capability: None,
/// };
/// ```
#[derive(Debug, Clone, Default)]
pub struct ComponentQuery {
    /// Filter by name pattern (glob-style matching)
    ///
    /// Examples: "image-*", "*-processor", "exact-name"
    pub name_pattern: Option<String>,

    /// Filter by lifecycle state
    pub state: Option<LifecycleState>,

    /// Filter by installation time (components installed after this timestamp)
    pub installed_after: Option<DateTime<Utc>>,

    /// Filter by required capability
    pub has_capability: Option<Capability>,
}

impl ComponentQuery {
    /// Create an empty query (matches all components).
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::management::ComponentQuery;
    ///
    /// let query = ComponentQuery::new();
    /// assert!(query.name_pattern.is_none());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by name pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::management::ComponentQuery;
    ///
    /// let query = ComponentQuery::new().with_name_pattern("image-*");
    /// assert_eq!(query.name_pattern, Some("image-*".to_string()));
    /// ```
    pub fn with_name_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.name_pattern = Some(pattern.into());
        self
    }

    /// Filter by lifecycle state.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::management::ComponentQuery;
    /// use airssys_wasm::core::lifecycle::LifecycleState;
    ///
    /// let query = ComponentQuery::new().with_state(LifecycleState::Running);
    /// assert_eq!(query.state, Some(LifecycleState::Running));
    /// ```
    pub fn with_state(mut self, state: LifecycleState) -> Self {
        self.state = Some(state);
        self
    }

    /// Filter by installation time.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::management::ComponentQuery;
    /// use chrono::{Utc, Duration};
    ///
    /// let one_hour_ago = Utc::now() - Duration::hours(1);
    /// let query = ComponentQuery::new().with_installed_after(one_hour_ago);
    /// assert!(query.installed_after.is_some());
    /// ```
    pub fn with_installed_after(mut self, timestamp: DateTime<Utc>) -> Self {
        self.installed_after = Some(timestamp);
        self
    }

    /// Filter by required capability.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::management::ComponentQuery;
    /// use airssys_wasm::core::{Capability, PathPattern};
    ///
    /// let capability = Capability::FileRead(PathPattern::new("/data/*"));
    /// let query = ComponentQuery::new().with_capability(capability.clone());
    /// assert_eq!(query.has_capability, Some(capability));
    /// ```
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.has_capability = Some(capability);
        self
    }

    /// Check if this query would match all components (no filters).
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::management::ComponentQuery;
    /// use airssys_wasm::core::lifecycle::LifecycleState;
    ///
    /// let empty = ComponentQuery::new();
    /// assert!(empty.is_empty());
    ///
    /// let filtered = ComponentQuery::new().with_state(LifecycleState::Running);
    /// assert!(!filtered.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.name_pattern.is_none()
            && self.state.is_none()
            && self.installed_after.is_none()
            && self.has_capability.is_none()
    }
}

/// Registry operation types for auditing.
///
/// Enumerates all possible registry operations with their associated data.
/// Used for audit logging and operation tracking.
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::management::RegistryOperation;
/// use airssys_wasm::core::ComponentId;
///
/// let op = RegistryOperation::Unregister(ComponentId::new("test"));
/// match op {
///     RegistryOperation::Unregister(id) => {
///         println!("Unregistering component: {}", id.as_str());
///     }
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone)]
pub enum RegistryOperation {
    /// Register a new component
    Register(ComponentId, ComponentMetadata),

    /// Unregister an existing component
    Unregister(ComponentId),

    /// Update component metadata
    Update(ComponentId, ComponentMetadata),

    /// Query components
    Query(ComponentQuery),
}

impl RegistryOperation {
    /// Get the component ID associated with this operation, if applicable.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::management::RegistryOperation;
    /// use airssys_wasm::core::ComponentId;
    ///
    /// let id = ComponentId::new("test");
    /// let op = RegistryOperation::Unregister(id.clone());
    /// assert_eq!(op.component_id(), Some(&id));
    ///
    /// let query_op = RegistryOperation::Query(Default::default());
    /// assert_eq!(query_op.component_id(), None);
    /// ```
    pub fn component_id(&self) -> Option<&ComponentId> {
        match self {
            RegistryOperation::Register(id, _)
            | RegistryOperation::Unregister(id)
            | RegistryOperation::Update(id, _) => Some(id),
            RegistryOperation::Query(_) => None,
        }
    }

    /// Get a human-readable description of this operation.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::management::RegistryOperation;
    /// use airssys_wasm::core::ComponentId;
    ///
    /// let op = RegistryOperation::Unregister(ComponentId::new("test"));
    /// assert_eq!(op.description(), "Unregister component");
    /// ```
    pub fn description(&self) -> &str {
        match self {
            RegistryOperation::Register(_, _) => "Register component",
            RegistryOperation::Unregister(_) => "Unregister component",
            RegistryOperation::Update(_, _) => "Update component metadata",
            RegistryOperation::Query(_) => "Query components",
        }
    }
}

#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic, clippy::indexing_slicing, clippy::too_many_arguments, clippy::type_complexity, reason = "test code")]
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_installation_metadata_creation() {
        let id = ComponentId::new("test-component");
        let version = VersionInfo::new("1.0.0", "abc123");
        let source = InstallationSource::File {
            path: PathBuf::from("/test.wasm"),
        };
        let install_path = PathBuf::from("/var/lib/components/test");

        let metadata = InstallationMetadata::new(id.clone(), version, source, install_path);

        assert_eq!(metadata.component_id, id);
        assert_eq!(metadata.version.version, "1.0.0");
        assert_eq!(metadata.state, LifecycleState::Installed);
        assert!(!metadata.is_active());
    }

    #[test]
    fn test_installation_metadata_state_changes() {
        let id = ComponentId::new("test");
        let version = VersionInfo::new("1.0.0", "hash");
        let source = InstallationSource::File {
            path: PathBuf::from("/test.wasm"),
        };
        let install_path = PathBuf::from("/install");

        let mut metadata = InstallationMetadata::new(id, version, source, install_path);

        assert!(!metadata.is_active());

        metadata.set_state(LifecycleState::Running);
        assert_eq!(metadata.state, LifecycleState::Running);
        assert!(metadata.is_active());

        metadata.set_state(LifecycleState::Stopped);
        assert!(!metadata.is_active());
    }

    #[test]
    fn test_component_query_empty() {
        let query = ComponentQuery::new();
        assert!(query.is_empty());
        assert!(query.name_pattern.is_none());
        assert!(query.state.is_none());
    }

    #[test]
    fn test_component_query_builder() {
        let query = ComponentQuery::new()
            .with_name_pattern("image-*")
            .with_state(LifecycleState::Running);

        assert!(!query.is_empty());
        assert_eq!(query.name_pattern, Some("image-*".to_string()));
        assert_eq!(query.state, Some(LifecycleState::Running));
    }

    #[test]
    fn test_component_query_with_time_filter() {
        let timestamp = Utc::now();
        let query = ComponentQuery::new().with_installed_after(timestamp);

        assert!(!query.is_empty());
        assert_eq!(query.installed_after, Some(timestamp));
    }

    #[test]
    fn test_registry_operation_component_id() {
        let id = ComponentId::new("test");

        let unregister = RegistryOperation::Unregister(id.clone());
        assert_eq!(unregister.component_id(), Some(&id));

        let query = RegistryOperation::Query(ComponentQuery::new());
        assert_eq!(query.component_id(), None);
    }

    #[test]
    fn test_registry_operation_description() {
        let id = ComponentId::new("test");

        let unregister = RegistryOperation::Unregister(id);
        assert_eq!(unregister.description(), "Unregister component");

        let query = RegistryOperation::Query(ComponentQuery::new());
        assert_eq!(query.description(), "Query components");
    }
}
