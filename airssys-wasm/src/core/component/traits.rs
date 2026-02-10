// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)
// None needed for this module

// Layer 2: Third-party crate imports (per PROJECTS_STANDARD.md §2.1)
// None needed for this module

// Layer 3: Internal module imports (per PROJECTS_STANDARD.md §2.1)
use super::errors::ComponentError;
use super::id::ComponentId;

/// Trait for component lifecycle management.
///
/// ComponentLifecycle defines the lifecycle operations that components must
/// support: initialization, shutdown, and health checking.
///
/// Implementations of this trait are provided by concrete component types
/// in the `component/` module (Layer 3A), which wraps WASM components
/// as actors managed by airssys-rt.
///
/// # Trait Bounds
///
/// This trait requires `Send + Sync`, enabling it to be used in
/// concurrent contexts where components may be accessed from multiple threads.
///
/// # Architecture Note
///
/// ComponentLifecycle lives in `core/component/` (Layer 1) as a trait abstraction.
/// Concrete implementations are provided by the `component/` module (Layer 3A),
/// which integrates with airssys-rt's Actor system.
///
/// The trait uses generic placeholder types (ComponentHandle, ComponentMessage)
/// that are also defined in `core/component/`. The ComponentError type is
/// currently a placeholder that will be replaced with the real error type
/// from `core/errors/wasm.rs` in a future task.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::component::traits::ComponentLifecycle;
/// use airssys_wasm::core::component::errors::ComponentError;
///
/// struct MockComponent {
///     initialized: bool,
/// }
///
/// impl ComponentLifecycle for MockComponent {
///     fn initialize(&mut self) -> Result<(), ComponentError> {
///         self.initialized = true;
///         Ok(())
///     }
///
///     fn shutdown(&mut self) -> Result<(), ComponentError> {
///         self.initialized = false;
///         Ok(())
///     }
///
///     fn health_check(&self) -> bool {
///         self.initialized
///     }
/// }
/// ```
pub trait ComponentLifecycle: Send + Sync {
    /// Initialize the component.
    ///
    /// This method is called when a component is first loaded or created.
    /// Components should perform any necessary setup operations during
    /// initialization, such as:
    ///
    /// - Loading WASM binaries
    /// - Setting up internal state
    /// - Allocating resources
    /// - Preparing to receive messages
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails. The component should not
    /// be used if initialization fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::component::traits::ComponentLifecycle;
    /// use airssys_wasm::core::component::errors::ComponentError;
    ///
    /// struct MyComponent {
    ///     initialized: bool,
    /// }
    ///
    /// impl ComponentLifecycle for MyComponent {
    ///     fn initialize(&mut self) -> Result<(), ComponentError> {
    ///         self.initialized = true;
    ///         Ok(())
    ///     }
    /// #     fn shutdown(&mut self) -> Result<(), ComponentError> { Ok(()) }
    /// #     fn health_check(&self) -> bool { true }
    /// }
    /// ```
    fn initialize(&mut self) -> Result<(), ComponentError>;

    /// Shutdown the component.
    ///
    /// This method is called when a component is being unloaded or destroyed.
    /// Components should perform cleanup operations during shutdown, such as:
    ///
    /// - Releasing resources
    /// - Persisting state
    /// - Closing connections
    /// - Cleaning up internal structures
    ///
    /// # Errors
    ///
    /// Returns an error if shutdown fails. The component should still be
    /// considered unusable after shutdown is attempted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::component::traits::ComponentLifecycle;
    /// use airssys_wasm::core::component::errors::ComponentError;
    ///
    /// struct MyComponent {
    ///     initialized: bool,
    /// }
    ///
    /// impl ComponentLifecycle for MyComponent {
    /// #     fn initialize(&mut self) -> Result<(), ComponentError> { Ok(()) }
    ///     fn shutdown(&mut self) -> Result<(), ComponentError> {
    ///         self.initialized = false;
    ///         Ok(())
    ///     }
    /// #     fn health_check(&self) -> bool { true }
    /// }
    /// ```
    fn shutdown(&mut self) -> Result<(), ComponentError>;

    /// Check component health.
    ///
    /// This method is called to verify that the component is functioning
    /// correctly. Components should return `true` if they are healthy
    /// and able to process messages, or `false` if they are unhealthy.
    ///
    /// # Returns
    ///
    /// Returns `true` if the component is healthy, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::component::traits::ComponentLifecycle;
    /// use airssys_wasm::core::component::errors::ComponentError;
    ///
    /// struct MyComponent {
    ///     initialized: bool,
    /// }
    ///
    /// impl ComponentLifecycle for MyComponent {
    /// #     fn initialize(&mut self) -> Result<(), ComponentError> { Ok(()) }
    /// #     fn shutdown(&mut self) -> Result<(), ComponentError> { Ok(()) }
    ///     fn health_check(&self) -> bool {
    ///         self.initialized
    ///     }
    /// }
    /// ```
    fn health_check(&self) -> bool;
}

/// Trait for resolving whether a component exists in the system.
///
/// This trait provides the abstraction needed by the messaging layer
/// to verify target components before creating message envelopes.
/// It follows the Dependency Inversion Principle: `messaging/` depends
/// on this trait (in `core/`), not on the concrete `ComponentRegistry`
/// (in `component/`).
///
/// # Architecture
///
/// - **Defined in:** `core/component/traits.rs` (Layer 0)
/// - **Consumed by:** `messaging/router.rs` (Layer 3B)
/// - **Implemented by:** `component/registry.rs` (Layer 3A)
/// - **Wired by:** `system/` (Layer 4)
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` for use in concurrent messaging
/// contexts where multiple routers may check component existence simultaneously.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::component::traits::ComponentResolver;
/// use airssys_wasm::core::component::errors::ComponentError;
/// use airssys_wasm::core::component::id::ComponentId;
///
/// struct MockResolver {
///     registered: Vec<ComponentId>,
/// }
///
/// impl ComponentResolver for MockResolver {
///     fn contains(&self, id: &ComponentId) -> Result<bool, ComponentError> {
///         Ok(self.registered.iter().any(|r| r == id))
///     }
/// }
/// ```
pub trait ComponentResolver: Send + Sync {
    /// Check if a component with the given ID is currently registered.
    ///
    /// # Arguments
    ///
    /// * `id` - The component identifier to check
    ///
    /// # Returns
    ///
    /// `true` if the component exists, `false` otherwise.
    ///
    /// # Errors
    ///
    /// Returns `ComponentError` if the lookup cannot be performed
    /// (e.g., lock poisoning in the underlying storage).
    fn contains(&self, id: &ComponentId) -> Result<bool, ComponentError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock component for testing ComponentLifecycle trait.
    struct MockComponent {
        initialized: bool,
        shutdown_called: bool,
        fail_initialize: bool,
        fail_shutdown: bool,
    }

    impl MockComponent {
        fn new() -> Self {
            Self {
                initialized: false,
                shutdown_called: false,
                fail_initialize: false,
                fail_shutdown: false,
            }
        }

        fn with_failure_modes(fail_initialize: bool, fail_shutdown: bool) -> Self {
            Self {
                initialized: false,
                shutdown_called: false,
                fail_initialize,
                fail_shutdown,
            }
        }
    }

    impl ComponentLifecycle for MockComponent {
        fn initialize(&mut self) -> Result<(), ComponentError> {
            if self.fail_initialize {
                return Err(ComponentError::InitializationFailed(
                    "test failure".to_string(),
                ));
            }
            self.initialized = true;
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), ComponentError> {
            if self.fail_shutdown {
                return Err(ComponentError::ShutdownFailed("test failure".to_string()));
            }
            self.shutdown_called = true;
            self.initialized = false;
            Ok(())
        }

        fn health_check(&self) -> bool {
            self.initialized
        }
    }

    #[test]
    fn test_mock_component_implements_component_lifecycle() {
        let mut component = MockComponent::new();

        // Verify initial state
        assert!(!component.health_check());

        // Initialize
        assert!(component.initialize().is_ok());
        assert!(component.health_check());

        // Shutdown
        assert!(component.shutdown().is_ok());
        assert!(!component.health_check());
    }

    #[test]
    fn test_initialize_is_called_correctly() {
        let mut component = MockComponent::new();

        assert!(!component.health_check());

        let result = component.initialize();

        assert!(result.is_ok());
        assert!(component.health_check());
    }

    #[test]
    fn test_shutdown_is_called_correctly() {
        let mut component = MockComponent::new();

        component.initialize().unwrap();
        assert!(component.health_check());

        let result = component.shutdown();

        assert!(result.is_ok());
        assert!(!component.health_check());
    }

    #[test]
    fn test_health_check_returns_boolean() {
        let mut component = MockComponent::new();

        // Not initialized
        assert!(!component.health_check());

        // After initialization
        component.initialize().unwrap();
        assert!(component.health_check());

        // After shutdown
        component.shutdown().unwrap();
        assert!(!component.health_check());
    }

    #[test]
    fn test_trait_object_creation() {
        let mut component: Box<dyn ComponentLifecycle> = Box::new(MockComponent::new());

        assert!(!component.health_check());

        assert!(component.initialize().is_ok());
        assert!(component.health_check());

        assert!(component.shutdown().is_ok());
        assert!(!component.health_check());
    }

    #[test]
    fn test_trait_is_send_and_sync() {
        // This test verifies that ComponentLifecycle trait is Send + Sync
        // by ensuring it can be used in multi-threaded contexts

        use std::sync::{Arc, Mutex};
        use std::thread;

        let component = Arc::new(Mutex::new(MockComponent::new()));

        // Spawn multiple threads that interact with the component
        let handles: Vec<_> = (0..4)
            .map(|_| {
                let comp = Arc::clone(&component);
                thread::spawn(move || {
                    let mut comp = comp.lock().unwrap();
                    comp.initialize().unwrap();
                    comp.health_check();
                    comp.shutdown().unwrap();
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // Verify component is in expected state
        let comp = component.lock().unwrap();
        assert!(!comp.health_check());
    }

    #[test]
    fn test_initialize_returns_error_on_failure() {
        let mut component = MockComponent::with_failure_modes(true, false);

        let result = component.initialize();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ComponentError::InitializationFailed(_)
        ));
        assert!(!component.health_check());
    }

    #[test]
    fn test_shutdown_returns_error_on_failure() {
        let mut component = MockComponent::with_failure_modes(false, true);

        component.initialize().unwrap();

        let result = component.shutdown();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ComponentError::ShutdownFailed(_)
        ));
    }

    #[test]
    fn test_multiple_lifecycle_cycles() {
        let mut component = MockComponent::new();

        // Cycle 1
        component.initialize().unwrap();
        assert!(component.health_check());
        component.shutdown().unwrap();
        assert!(!component.health_check());

        // Cycle 2
        component.initialize().unwrap();
        assert!(component.health_check());
        component.shutdown().unwrap();
        assert!(!component.health_check());
    }

    // ---------------------------------------------------------------
    // ComponentResolver tests
    // ---------------------------------------------------------------

    struct MockResolver {
        registered: Vec<ComponentId>,
    }

    impl MockResolver {
        fn new() -> Self {
            Self {
                registered: Vec::new(),
            }
        }

        fn with_components(ids: Vec<ComponentId>) -> Self {
            Self { registered: ids }
        }
    }

    impl ComponentResolver for MockResolver {
        fn contains(&self, id: &ComponentId) -> Result<bool, ComponentError> {
            Ok(self.registered.iter().any(|r| r == id))
        }
    }

    #[test]
    fn test_component_resolver_contains_existing() {
        let id = ComponentId::new("app", "service", "v1");
        let resolver = MockResolver::with_components(vec![id.clone()]);

        assert!(resolver.contains(&id).unwrap());
    }

    #[test]
    fn test_component_resolver_contains_nonexistent() {
        let resolver = MockResolver::new();
        let id = ComponentId::new("app", "nonexistent", "v1");

        assert!(!resolver.contains(&id).unwrap());
    }

    #[test]
    fn test_component_resolver_is_send_sync() {
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}
        assert_send_sync::<dyn ComponentResolver>();
    }

    #[test]
    fn test_component_resolver_as_trait_object() {
        let resolver: Box<dyn ComponentResolver> = Box::new(MockResolver::new());
        let id = ComponentId::new("app", "test", "v1");
        assert!(!resolver.contains(&id).unwrap());
    }
}
