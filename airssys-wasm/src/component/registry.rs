//! Component registry for tracking loaded components and their actor addresses.
//!
//! Provides thread-safe mapping of ComponentId -> ActorAddress for message routing.
//!
//! # Architecture
//!
//! ComponentRegistry is part of Layer 3A (component/ module). It:
//! - Uses types from `core/component/` (ComponentId)
//! - Uses types from `airssys-rt` (ActorAddress)
//! - Enables message routing between WASM component actors
//!
//! # Module Boundary Rules
//!
//! - CAN import: `core/`, `airssys-rt`
//! - CANNOT import: `runtime/`, `security/`, `system/`
//!
//! # Error Handling
//!
//! All methods that access the RwLock return `Result<T, RegistryError>` to handle
//! potential lock poisoning. This follows workspace policy of denying `unwrap_used`.
//!
//! # References
//!
//! - ADR-WASM-031: Component & Messaging Module Design
//! - ADR-WASM-023: Module Boundary Enforcement

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::fmt;
use std::sync::RwLock;

// Layer 2: Third-party crate imports
use airssys_rt::ActorAddress;
use thiserror::Error;

// Layer 3: Internal module imports
use crate::core::component::id::ComponentId;

/// Errors that can occur during registry operations.
#[derive(Debug, Clone, Error)]
pub enum RegistryError {
    /// The RwLock has been poisoned by a panic in another thread.
    #[error("Registry lock poisoned: {0}")]
    LockPoisoned(String),
}

/// Registry for tracking loaded components and their actor addresses.
///
/// Thread-safe registry using RwLock for concurrent read access
/// with minimal write contention. Maps ComponentId to ActorAddress
/// for message routing between WASM component actors.
///
/// # Thread Safety
///
/// Uses `RwLock<HashMap>` for interior mutability:
/// - Multiple concurrent reads allowed
/// - Exclusive access for writes
/// - No async operations (synchronous lock acquisition)
///
/// # Error Handling
///
/// All methods return `Result<T, RegistryError>` to handle lock poisoning.
/// Lock poisoning occurs when a thread panics while holding the lock.
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_wasm::component::registry::ComponentRegistry;
/// use airssys_wasm::core::component::id::ComponentId;
/// use airssys_rt::ActorAddress;
///
/// let registry = ComponentRegistry::new();
/// let id = ComponentId::new("system", "database", "prod");
/// let address = ActorAddress::named("database_actor");
///
/// // Register a component
/// registry.register(id.clone(), address.clone())?;
///
/// // Lookup by ID
/// assert_eq!(registry.get(&id)?, Some(address));
/// ```
pub struct ComponentRegistry {
    /// Maps ComponentId to ActorAddress for message routing
    components: RwLock<HashMap<ComponentId, ActorAddress>>,
}

impl ComponentRegistry {
    /// Creates a new empty registry.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::component::registry::ComponentRegistry;
    ///
    /// let registry = ComponentRegistry::new();
    /// assert_eq!(registry.count()?, 0);
    /// ```
    pub fn new() -> Self {
        Self {
            components: RwLock::new(HashMap::new()),
        }
    }

    /// Registers a component with its actor address.
    ///
    /// If the component ID already exists, the address is updated.
    ///
    /// # Arguments
    ///
    /// * `id` - The component identifier
    /// * `address` - The actor address for this component
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::LockPoisoned` if the lock has been poisoned.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::component::registry::ComponentRegistry;
    /// use airssys_wasm::core::component::id::ComponentId;
    /// use airssys_rt::ActorAddress;
    ///
    /// let registry = ComponentRegistry::new();
    /// let id = ComponentId::new("system", "database", "prod");
    /// let address = ActorAddress::named("db_actor");
    ///
    /// registry.register(id, address)?;
    /// ```
    pub fn register(&self, id: ComponentId, address: ActorAddress) -> Result<(), RegistryError> {
        let mut components = self
            .components
            .write()
            .map_err(|e| RegistryError::LockPoisoned(e.to_string()))?;
        components.insert(id, address);
        Ok(())
    }

    /// Unregisters a component and returns its actor address.
    ///
    /// # Arguments
    ///
    /// * `id` - The component identifier to remove
    ///
    /// # Returns
    ///
    /// The actor address if the component was registered, `None` otherwise.
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::LockPoisoned` if the lock has been poisoned.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::component::registry::ComponentRegistry;
    /// use airssys_wasm::core::component::id::ComponentId;
    /// use airssys_rt::ActorAddress;
    ///
    /// let registry = ComponentRegistry::new();
    /// let id = ComponentId::new("system", "database", "prod");
    /// let address = ActorAddress::named("db_actor");
    ///
    /// registry.register(id.clone(), address)?;
    /// let removed = registry.unregister(&id)?;
    /// assert!(removed.is_some());
    /// ```
    pub fn unregister(&self, id: &ComponentId) -> Result<Option<ActorAddress>, RegistryError> {
        let mut components = self
            .components
            .write()
            .map_err(|e| RegistryError::LockPoisoned(e.to_string()))?;
        Ok(components.remove(id))
    }

    /// Gets the actor address for a component.
    ///
    /// # Arguments
    ///
    /// * `id` - The component identifier to lookup
    ///
    /// # Returns
    ///
    /// The actor address if found, `None` otherwise.
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::LockPoisoned` if the lock has been poisoned.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::component::registry::ComponentRegistry;
    /// use airssys_wasm::core::component::id::ComponentId;
    ///
    /// let registry = ComponentRegistry::new();
    /// let id = ComponentId::new("system", "database", "prod");
    ///
    /// assert_eq!(registry.get(&id)?, None);
    /// ```
    pub fn get(&self, id: &ComponentId) -> Result<Option<ActorAddress>, RegistryError> {
        let components = self
            .components
            .read()
            .map_err(|e| RegistryError::LockPoisoned(e.to_string()))?;
        Ok(components.get(id).cloned())
    }

    /// Checks if a component is registered.
    ///
    /// # Arguments
    ///
    /// * `id` - The component identifier to check
    ///
    /// # Returns
    ///
    /// `true` if the component is registered, `false` otherwise.
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::LockPoisoned` if the lock has been poisoned.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::component::registry::ComponentRegistry;
    /// use airssys_wasm::core::component::id::ComponentId;
    ///
    /// let registry = ComponentRegistry::new();
    /// let id = ComponentId::new("system", "database", "prod");
    ///
    /// assert!(!registry.contains(&id)?);
    /// ```
    pub fn contains(&self, id: &ComponentId) -> Result<bool, RegistryError> {
        let components = self
            .components
            .read()
            .map_err(|e| RegistryError::LockPoisoned(e.to_string()))?;
        Ok(components.contains_key(id))
    }

    /// Lists all registered component IDs.
    ///
    /// # Returns
    ///
    /// A vector of all registered ComponentIds. Order is not guaranteed.
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::LockPoisoned` if the lock has been poisoned.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::component::registry::ComponentRegistry;
    ///
    /// let registry = ComponentRegistry::new();
    /// let list = registry.list()?;
    /// assert!(list.is_empty());
    /// ```
    pub fn list(&self) -> Result<Vec<ComponentId>, RegistryError> {
        let components = self
            .components
            .read()
            .map_err(|e| RegistryError::LockPoisoned(e.to_string()))?;
        Ok(components.keys().cloned().collect())
    }

    /// Returns the count of registered components.
    ///
    /// # Returns
    ///
    /// The number of registered components.
    ///
    /// # Errors
    ///
    /// Returns `RegistryError::LockPoisoned` if the lock has been poisoned.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::component::registry::ComponentRegistry;
    ///
    /// let registry = ComponentRegistry::new();
    /// assert_eq!(registry.count()?, 0);
    /// ```
    pub fn count(&self) -> Result<usize, RegistryError> {
        let components = self
            .components
            .read()
            .map_err(|e| RegistryError::LockPoisoned(e.to_string()))?;
        Ok(components.len())
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ComponentRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Attempt to read the component count for debug output
        // If the lock is poisoned, show that instead
        match self.components.read() {
            Ok(components) => f
                .debug_struct("ComponentRegistry")
                .field("component_count", &components.len())
                .finish(),
            Err(_) => f
                .debug_struct("ComponentRegistry")
                .field("status", &"<lock poisoned>")
                .finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================
    // Helper Functions
    // ========================================

    fn create_test_id(namespace: &str, name: &str, instance: &str) -> ComponentId {
        ComponentId::new(namespace, name, instance)
    }

    fn create_mock_address(name: &str) -> ActorAddress {
        ActorAddress::named(name)
    }

    // ========================================
    // Constructor Tests
    // ========================================

    #[test]
    fn test_registry_new() {
        let registry = ComponentRegistry::new();
        assert_eq!(registry.count().unwrap(), 0);
    }

    #[test]
    fn test_registry_default() {
        let registry = ComponentRegistry::default();
        assert_eq!(registry.count().unwrap(), 0);
    }

    // ========================================
    // Register Tests
    // ========================================

    #[test]
    fn test_register_single() {
        let registry = ComponentRegistry::new();
        let id = create_test_id("system", "database", "prod");
        let address = create_mock_address("db_actor");

        registry.register(id.clone(), address.clone()).unwrap();

        assert!(registry.contains(&id).unwrap());
        assert_eq!(registry.count().unwrap(), 1);
    }

    #[test]
    fn test_register_multiple() {
        let registry = ComponentRegistry::new();
        let id1 = create_test_id("system", "database", "prod");
        let id2 = create_test_id("user", "cache", "dev");
        let addr1 = create_mock_address("db_actor");
        let addr2 = create_mock_address("cache_actor");

        registry.register(id1.clone(), addr1).unwrap();
        registry.register(id2.clone(), addr2).unwrap();

        assert_eq!(registry.count().unwrap(), 2);
        assert!(registry.contains(&id1).unwrap());
        assert!(registry.contains(&id2).unwrap());
    }

    #[test]
    fn test_register_overwrites_existing() {
        let registry = ComponentRegistry::new();
        let id = create_test_id("system", "database", "prod");
        let addr1 = create_mock_address("old_actor");
        let addr2 = create_mock_address("new_actor");

        registry.register(id.clone(), addr1).unwrap();
        assert_eq!(registry.count().unwrap(), 1);

        // Register same ID with different address
        registry.register(id.clone(), addr2.clone()).unwrap();
        assert_eq!(registry.count().unwrap(), 1); // Still 1 component

        // Verify new address is stored
        let stored = registry.get(&id).unwrap();
        assert!(stored.is_some());
        assert_eq!(stored.as_ref().and_then(|a| a.name()), Some("new_actor"));
    }

    // ========================================
    // Unregister Tests
    // ========================================

    #[test]
    fn test_unregister_existing() {
        let registry = ComponentRegistry::new();
        let id = create_test_id("system", "database", "prod");
        let address = create_mock_address("db_actor");

        registry.register(id.clone(), address).unwrap();
        assert_eq!(registry.count().unwrap(), 1);

        let removed = registry.unregister(&id).unwrap();
        assert!(removed.is_some());
        assert_eq!(removed.as_ref().and_then(|a| a.name()), Some("db_actor"));
        assert_eq!(registry.count().unwrap(), 0);
        assert!(!registry.contains(&id).unwrap());
    }

    #[test]
    fn test_unregister_nonexistent() {
        let registry = ComponentRegistry::new();
        let id = create_test_id("system", "nonexistent", "v1");

        let removed = registry.unregister(&id).unwrap();
        assert_eq!(removed, None);
    }

    // ========================================
    // Get Tests
    // ========================================

    #[test]
    fn test_get_existing() {
        let registry = ComponentRegistry::new();
        let id = create_test_id("system", "database", "prod");
        let address = create_mock_address("db_actor");

        registry.register(id.clone(), address).unwrap();

        let retrieved = registry.get(&id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.as_ref().and_then(|a| a.name()), Some("db_actor"));
    }

    #[test]
    fn test_get_nonexistent() {
        let registry = ComponentRegistry::new();
        let id = create_test_id("system", "nonexistent", "v1");

        assert_eq!(registry.get(&id).unwrap(), None);
    }

    // ========================================
    // Contains Tests
    // ========================================

    #[test]
    fn test_contains_existing() {
        let registry = ComponentRegistry::new();
        let id = create_test_id("system", "database", "prod");
        let address = create_mock_address("db_actor");

        registry.register(id.clone(), address).unwrap();

        assert!(registry.contains(&id).unwrap());
    }

    #[test]
    fn test_contains_nonexistent() {
        let registry = ComponentRegistry::new();
        let id = create_test_id("system", "nonexistent", "v1");

        assert!(!registry.contains(&id).unwrap());
    }

    // ========================================
    // List Tests
    // ========================================

    #[test]
    fn test_list_empty() {
        let registry = ComponentRegistry::new();
        let list = registry.list().unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn test_list_multiple() {
        let registry = ComponentRegistry::new();
        let id1 = create_test_id("system", "database", "prod");
        let id2 = create_test_id("user", "cache", "dev");
        let id3 = create_test_id("app", "auth", "v1");

        registry
            .register(id1.clone(), create_mock_address("actor1"))
            .unwrap();
        registry
            .register(id2.clone(), create_mock_address("actor2"))
            .unwrap();
        registry
            .register(id3.clone(), create_mock_address("actor3"))
            .unwrap();

        let list = registry.list().unwrap();
        assert_eq!(list.len(), 3);
        assert!(list.contains(&id1));
        assert!(list.contains(&id2));
        assert!(list.contains(&id3));
    }

    // ========================================
    // Count Tests
    // ========================================

    #[test]
    fn test_count_operations() {
        let registry = ComponentRegistry::new();
        assert_eq!(registry.count().unwrap(), 0);

        registry
            .register(
                create_test_id("system", "c1", "v1"),
                create_mock_address("a1"),
            )
            .unwrap();
        assert_eq!(registry.count().unwrap(), 1);

        registry
            .register(
                create_test_id("system", "c2", "v1"),
                create_mock_address("a2"),
            )
            .unwrap();
        assert_eq!(registry.count().unwrap(), 2);

        registry
            .unregister(&create_test_id("system", "c1", "v1"))
            .unwrap();
        assert_eq!(registry.count().unwrap(), 1);
    }

    // ========================================
    // Thread Safety Tests
    // ========================================

    #[test]
    fn test_send_sync_bounds() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<ComponentRegistry>();
        assert_sync::<ComponentRegistry>();
    }

    // ========================================
    // Error Type Tests
    // ========================================

    #[test]
    fn test_registry_error_display() {
        let error = RegistryError::LockPoisoned("test error".to_string());
        assert!(error.to_string().contains("Registry lock poisoned"));
        assert!(error.to_string().contains("test error"));
    }

    #[test]
    fn test_registry_error_clone() {
        let error = RegistryError::LockPoisoned("test error".to_string());
        let cloned = error.clone();
        assert_eq!(error.to_string(), cloned.to_string());
    }

    // ========================================
    // Debug Trait Tests
    // ========================================

    #[test]
    fn test_registry_debug_empty() {
        let registry = ComponentRegistry::new();
        let debug_str = format!("{:?}", registry);
        assert!(debug_str.contains("ComponentRegistry"));
        assert!(debug_str.contains("component_count"));
        assert!(debug_str.contains("0"));
    }

    #[test]
    fn test_registry_debug_with_components() {
        let registry = ComponentRegistry::new();
        registry
            .register(
                create_test_id("system", "test", "v1"),
                create_mock_address("actor1"),
            )
            .unwrap();
        registry
            .register(
                create_test_id("system", "test2", "v1"),
                create_mock_address("actor2"),
            )
            .unwrap();

        let debug_str = format!("{:?}", registry);
        assert!(debug_str.contains("ComponentRegistry"));
        assert!(debug_str.contains("component_count"));
        assert!(debug_str.contains("2"));
    }
}
