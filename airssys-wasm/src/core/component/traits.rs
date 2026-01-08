// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)
// None needed for this module

// Layer 2: Third-party crate imports (per PROJECTS_STANDARD.md §2.1)
// None needed for this module

// Layer 3: Internal module imports (per PROJECTS_STANDARD.md §2.1)
// Note: ComponentHandle and ComponentMessage will be used when real WasmError
// type is implemented in core/errors/wasm.rs (future task)

/// WasmError placeholder.
///
/// This is a temporary placeholder type for WasmError.
/// The real WasmError will be implemented in `core/errors/wasm.rs`
/// (future task: WASM-TASK-0XX).
///
/// For now, we use a simple String to enable ComponentLifecycle trait
/// to compile and be tested.
///
/// # Architecture Note
///
/// The placeholder allows `core/component/` to be implemented without
/// waiting for `core/errors/` module. Once `core/errors/wasm.rs`
/// is created, this type alias will be replaced with real WasmError enum.
pub type WasmError = String;

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
/// that are also defined in `core/component/`. The WasmError type is
/// currently a placeholder that will be replaced with the real error type
/// from `core/errors/wasm.rs` in a future task.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::component::{
///     ComponentLifecycle, ComponentHandle, ComponentMessage, WasmError,
/// };
///
/// struct MockComponent {
///     initialized: bool,
/// }
///
/// impl ComponentLifecycle for MockComponent {
///     fn initialize(&mut self) -> Result<(), WasmError> {
///         self.initialized = true;
///         Ok(())
///     }
///
///     fn shutdown(&mut self) -> Result<(), WasmError> {
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
    /// use airssys_wasm::core::component::{ComponentLifecycle, WasmError};
    ///
    /// struct MyComponent {
    ///     initialized: bool,
    /// }
    ///
    /// impl ComponentLifecycle for MyComponent {
    ///     fn initialize(&mut self) -> Result<(), WasmError> {
    ///         self.initialized = true;
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn initialize(&mut self) -> Result<(), WasmError>;

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
    /// use airssys_wasm::core::component::{ComponentLifecycle, WasmError};
    ///
    /// struct MyComponent {
    ///     initialized: bool,
    /// }
    ///
    /// impl ComponentLifecycle for MyComponent {
    ///     fn shutdown(&mut self) -> Result<(), WasmError> {
    ///         self.initialized = false;
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn shutdown(&mut self) -> Result<(), WasmError>;

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
    /// use airssys_wasm::core::component::ComponentLifecycle;
    ///
    /// struct MyComponent {
    ///     initialized: bool,
    /// }
    ///
    /// impl ComponentLifecycle for MyComponent {
    ///     fn health_check(&self) -> bool {
    ///         self.initialized
    ///     }
    /// }
    /// ```
    fn health_check(&self) -> bool;
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
        fn initialize(&mut self) -> Result<(), WasmError> {
            if self.fail_initialize {
                return Err("Initialize failed".to_string());
            }
            self.initialized = true;
            Ok(())
        }

        fn shutdown(&mut self) -> Result<(), WasmError> {
            if self.fail_shutdown {
                return Err("Shutdown failed".to_string());
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
        assert_eq!(result.unwrap_err(), "Initialize failed");
        assert!(!component.health_check());
    }

    #[test]
    fn test_shutdown_returns_error_on_failure() {
        let mut component = MockComponent::with_failure_modes(false, true);

        component.initialize().unwrap();

        let result = component.shutdown();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Shutdown failed");
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
}
