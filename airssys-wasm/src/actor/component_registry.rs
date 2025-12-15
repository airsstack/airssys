//! Component registry for tracking component instances.
//!
//! This module provides `ComponentRegistry`, a thread-safe registry for tracking
//! ComponentActor instances by ComponentId with O(1) lookup performance.
//!
//! # Architecture
//!
//! ```text
//! ComponentRegistry
//!     ↓
//! Arc<RwLock<HashMap<ComponentId, ActorAddress>>>
//!     ↓
//! O(1) lookup via ComponentId → ActorAddress
//! ```
//!
//! # Performance
//!
//! - **Lookup**: O(1) - HashMap lookup
//! - **Registration**: O(1) - HashMap insert
//! - **Unregister**: O(1) - HashMap remove
//! - **Thread Safety**: RwLock allows concurrent reads
//!
//! # Usage
//!
//! ```rust,ignore
//! use airssys_wasm::actor::ComponentRegistry;
//! use airssys_wasm::core::ComponentId;
//! use airssys_rt::util::ActorAddress;
//!
//! let registry = ComponentRegistry::new();
//!
//! // Register component
//! registry.register(component_id.clone(), actor_ref.clone())?;
//!
//! // Lookup component (O(1))
//! let actor_ref = registry.lookup(&component_id)?;
//!
//! // Unregister component
//! registry.unregister(&component_id)?;
//! ```
//!
//! # References
//!
//! - **WASM-TASK-004 Phase 2 Task 2.2**: ComponentRegistry Implementation
//! - **ADR-WASM-006**: Actor-based Component Isolation

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// Layer 2: Third-party crate imports
// (none)

// Layer 3: Internal module imports
use crate::core::{ComponentId, WasmError};
use airssys_rt::util::ActorAddress;

/// ComponentRegistry for tracking component instances.
///
/// ComponentRegistry provides thread-safe O(1) lookup of component instances
/// by ComponentId. It uses `Arc<RwLock<HashMap>>` for concurrent access with
/// multiple readers or single writer.
///
/// # Thread Safety
///
/// - **Concurrent reads**: Multiple threads can lookup simultaneously
/// - **Exclusive writes**: Registration/unregistration requires exclusive lock
/// - **Poisoning**: Lock poisoning is handled as internal error
///
/// # Cloning
///
/// Cloning a `ComponentRegistry` creates a new handle to the same underlying
/// storage (Arc clone). All clones share the same registry data. This allows
/// the registry to be safely passed across threads while maintaining a single
/// source of truth.
///
/// # Performance
///
/// Target: <1μs lookup time (O(1) HashMap access + RwLock overhead)
///
/// Note: Pure HashMap lookup is <100ns, but RwLock acquisition adds ~700ns overhead
/// in debug builds. Release builds with optimizations should approach the target.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::actor::ComponentRegistry;
/// use airssys_wasm::core::ComponentId;
/// use airssys_rt::util::ActorAddress;
///
/// let registry = ComponentRegistry::new();
/// assert_eq!(registry.count().unwrap(), 0);
///
/// // Register components
/// let component_id = ComponentId::new("test");
/// let actor_addr = ActorAddress::named("test");
/// registry.register(component_id.clone(), actor_addr.clone()).unwrap();
///
/// // Lookup (O(1))
/// let found = registry.lookup(&component_id).unwrap();
/// assert_eq!(found, actor_addr);
///
/// // Unregister
/// registry.unregister(&component_id).unwrap();
/// assert_eq!(registry.count().unwrap(), 0);
/// ```
#[derive(Clone, Debug)]
pub struct ComponentRegistry {
    instances: Arc<RwLock<HashMap<ComponentId, ActorAddress>>>,
}

impl ComponentRegistry {
    /// Create a new component registry.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::actor::ComponentRegistry;
    ///
    /// let registry = ComponentRegistry::new();
    /// assert_eq!(registry.count().unwrap(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            instances: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a component instance.
    ///
    /// Adds a component to the registry, mapping ComponentId to ActorAddress.
    /// If the component already exists, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `component_id` - Unique component identifier
    /// * `actor_addr` - ActorAddress for sending messages to component
    ///
    /// # Errors
    ///
    /// Returns `WasmError::Internal` if registry lock is poisoned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::actor::ComponentRegistry;
    /// use airssys_wasm::core::ComponentId;
    /// use airssys_rt::util::ActorAddress;
    ///
    /// let registry = ComponentRegistry::new();
    /// let component_id = ComponentId::new("worker");
    /// let actor_addr = ActorAddress::named("worker");
    ///
    /// registry.register(component_id, actor_addr).unwrap();
    /// assert_eq!(registry.count().unwrap(), 1);
    /// ```
    pub fn register(
        &self,
        component_id: ComponentId,
        actor_addr: ActorAddress,
    ) -> Result<(), WasmError> {
        let mut instances = self
            .instances
            .write()
            .map_err(|e| WasmError::internal(format!("Registry lock poisoned during register: {}", e)))?;

        instances.insert(component_id, actor_addr);
        Ok(())
    }

    /// Lookup a component by ID (O(1)).
    ///
    /// Returns the ActorAddress for the specified component, or an error if
    /// the component is not registered.
    ///
    /// # Arguments
    ///
    /// * `component_id` - Component identifier to lookup
    ///
    /// # Returns
    ///
    /// Returns cloned ActorAddress if found.
    ///
    /// # Errors
    ///
    /// * `WasmError::ComponentNotFound` - Component not in registry
    /// * `WasmError::Internal` - Registry lock poisoned
    ///
    /// # Performance
    ///
    /// Target: <1μs (O(1) HashMap lookup + RwLock read overhead)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::actor::ComponentRegistry;
    /// use airssys_wasm::core::ComponentId;
    /// use airssys_rt::util::ActorAddress;
    ///
    /// let registry = ComponentRegistry::new();
    /// let component_id = ComponentId::new("test");
    /// let actor_addr = ActorAddress::named("test");
    ///
    /// registry.register(component_id.clone(), actor_addr.clone()).unwrap();
    ///
    /// let found = registry.lookup(&component_id).unwrap();
    /// assert_eq!(found, actor_addr);
    /// ```
    pub fn lookup(&self, component_id: &ComponentId) -> Result<ActorAddress, WasmError> {
        let instances = self
            .instances
            .read()
            .map_err(|e| WasmError::internal(format!("Registry lock poisoned during lookup: {}", e)))?;

        instances
            .get(component_id)
            .cloned()
            .ok_or_else(|| {
                WasmError::component_not_found(format!("Component {} not found", component_id.as_str()))
            })
    }

    /// Remove a component from the registry.
    ///
    /// Unregisters a component, removing it from the registry. If the component
    /// is not registered, this operation succeeds silently.
    ///
    /// # Arguments
    ///
    /// * `component_id` - Component identifier to unregister
    ///
    /// # Errors
    ///
    /// Returns `WasmError::Internal` if registry lock is poisoned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::actor::ComponentRegistry;
    /// use airssys_wasm::core::ComponentId;
    /// use airssys_rt::util::ActorAddress;
    ///
    /// let registry = ComponentRegistry::new();
    /// let component_id = ComponentId::new("test");
    /// let actor_addr = ActorAddress::named("test");
    ///
    /// registry.register(component_id.clone(), actor_addr).unwrap();
    /// assert_eq!(registry.count().unwrap(), 1);
    ///
    /// registry.unregister(&component_id).unwrap();
    /// assert_eq!(registry.count().unwrap(), 0);
    /// ```
    pub fn unregister(&self, component_id: &ComponentId) -> Result<(), WasmError> {
        let mut instances = self
            .instances
            .write()
            .map_err(|e| WasmError::internal(format!("Registry lock poisoned during unregister: {}", e)))?;

        instances.remove(component_id);
        Ok(())
    }

    /// Get the count of registered components.
    ///
    /// Returns the number of components currently tracked by the registry.
    ///
    /// # Returns
    ///
    /// Number of registered components.
    ///
    /// # Errors
    ///
    /// Returns `WasmError::Internal` if registry lock is poisoned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::actor::ComponentRegistry;
    /// use airssys_wasm::core::ComponentId;
    /// use airssys_rt::util::ActorAddress;
    ///
    /// let registry = ComponentRegistry::new();
    /// assert_eq!(registry.count().unwrap(), 0);
    ///
    /// registry.register(
    ///     ComponentId::new("test"),
    ///     ActorAddress::named("test")
    /// ).unwrap();
    /// assert_eq!(registry.count().unwrap(), 1);
    /// ```
    pub fn count(&self) -> Result<usize, WasmError> {
        let instances = self
            .instances
            .read()
            .map_err(|e| WasmError::internal(format!("Registry lock poisoned during count: {}", e)))?;
        
        Ok(instances.len())
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "expect is acceptable in test code for clear error messages")]
#[expect(clippy::panic, reason = "panic is acceptable in test code for assertion failures")]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ComponentRegistry::new();
        assert_eq!(registry.count().expect("Failed to get count"), 0);
    }

    #[test]
    fn test_register_component() {
        let registry = ComponentRegistry::new();
        let component_id = ComponentId::new("test-component");
        let actor_addr = ActorAddress::named("test-component");

        let result = registry.register(component_id.clone(), actor_addr.clone());
        assert!(result.is_ok(), "Failed to register component: {:?}", result.err());
        assert_eq!(registry.count().expect("Failed to get count"), 1);
    }

    #[test]
    fn test_lookup_component() {
        let registry = ComponentRegistry::new();
        let component_id = ComponentId::new("test-component");
        let actor_addr = ActorAddress::named("test-component");

        let result = registry.register(component_id.clone(), actor_addr.clone());
        assert!(result.is_ok(), "Failed to register component: {:?}", result.err());

        let found = registry.lookup(&component_id);
        assert!(found.is_ok(), "Failed to lookup component: {:?}", found.err());
        assert_eq!(found.expect("lookup returned Ok but unwrap failed"), actor_addr);
    }

    #[test]
    fn test_lookup_nonexistent_component() {
        let registry = ComponentRegistry::new();
        let component_id = ComponentId::new("nonexistent");

        let result = registry.lookup(&component_id);
        assert!(result.is_err());

        match result {
            Err(WasmError::ComponentNotFound { .. }) => {}
            _ => panic!("Expected ComponentNotFound error"),
        }
    }

    #[test]
    fn test_unregister_component() {
        let registry = ComponentRegistry::new();
        let component_id = ComponentId::new("test-component");
        let actor_addr = ActorAddress::named("test-component");

        let result = registry.register(component_id.clone(), actor_addr);
        assert!(result.is_ok(), "Failed to register component: {:?}", result.err());
        assert_eq!(registry.count().expect("Failed to get count"), 1);

        let result = registry.unregister(&component_id);
        assert!(result.is_ok(), "Failed to unregister component: {:?}", result.err());
        assert_eq!(registry.count().expect("Failed to get count"), 0);

        // Verify component is gone
        let result = registry.lookup(&component_id);
        assert!(result.is_err(), "Expected error for missing component");
    }

    #[test]
    fn test_unregister_nonexistent_component() {
        let registry = ComponentRegistry::new();
        let component_id = ComponentId::new("nonexistent");

        // Should succeed silently
        let result = registry.unregister(&component_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_multiple_components() {
        let registry = ComponentRegistry::new();

        for i in 0..10 {
            let component_id = ComponentId::new(format!("component-{}", i));
            let actor_addr = ActorAddress::named(format!("component-{}", i));
            let result = registry.register(component_id, actor_addr);
            assert!(result.is_ok(), "Failed to register component {}: {:?}", i, result.err());
        }

        assert_eq!(registry.count().expect("Failed to get count"), 10);
    }

    #[test]
    fn test_register_overwrites_existing() {
        let registry = ComponentRegistry::new();
        let component_id = ComponentId::new("test");
        let addr1 = ActorAddress::named("test-1");
        let addr2 = ActorAddress::named("test-2");

        // Register first time
        let result = registry.register(component_id.clone(), addr1);
        assert!(result.is_ok(), "Failed to register component first time: {:?}", result.err());
        assert_eq!(registry.count().expect("Failed to get count"), 1);

        // Register again with different address
        let result = registry.register(component_id.clone(), addr2.clone());
        assert!(result.is_ok(), "Failed to register component second time: {:?}", result.err());
        assert_eq!(registry.count().expect("Failed to get count"), 1);

        // Verify new address
        let found = registry.lookup(&component_id);
        assert!(found.is_ok(), "Failed to lookup component: {:?}", found.err());
        assert_eq!(found.expect("lookup returned Ok but unwrap failed"), addr2);
    }

    #[test]
    fn test_registry_clone() {
        let registry1 = ComponentRegistry::new();
        let component_id = ComponentId::new("test");
        let actor_addr = ActorAddress::named("test");

        let result = registry1.register(component_id.clone(), actor_addr.clone());
        assert!(result.is_ok(), "Failed to register component: {:?}", result.err());

        // Clone registry (Arc clone, shares data)
        let registry2 = registry1.clone();

        // Both registries see the same data
        assert_eq!(registry1.count().expect("Failed to get count from registry1"), 1);
        assert_eq!(registry2.count().expect("Failed to get count from registry2"), 1);

        let found = registry2.lookup(&component_id);
        assert!(found.is_ok(), "Failed to lookup component: {:?}", found.err());
        assert_eq!(found.expect("lookup returned Ok but unwrap failed"), actor_addr);
    }

    #[tokio::test]
    async fn test_concurrent_lookups() {
        use tokio::task;

        let registry = ComponentRegistry::new();
        let component_id = ComponentId::new("test");
        let actor_addr = ActorAddress::named("test");

        let result = registry.register(component_id.clone(), actor_addr.clone());
        assert!(result.is_ok(), "Failed to register component: {:?}", result.err());

        // Spawn multiple concurrent readers
        let mut handles = vec![];
        for _ in 0..10 {
            let registry_clone = registry.clone();
            let id_clone = component_id.clone();
            let addr_clone = actor_addr.clone();

            let handle = task::spawn(async move {
                let found = registry_clone.lookup(&id_clone);
                assert!(found.is_ok(), "Failed to lookup component: {:?}", found.err());
                assert_eq!(found.expect("lookup returned Ok but unwrap failed"), addr_clone);
            });

            handles.push(handle);
        }

        // Wait for all readers to complete
        for handle in handles {
            handle.await.expect("Task panicked");
        }
    }

    #[test]
    fn test_default_implementation() {
        let registry = ComponentRegistry::default();
        assert_eq!(registry.count().expect("Failed to get count"), 0);
    }
}
