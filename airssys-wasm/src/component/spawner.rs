//! Component spawner for creating and managing component actors.
//!
//! Orchestrates the full component lifecycle:
//! - Load component bytes via [`ComponentLoader`]
//! - Validate component binary
//! - Create [`ComponentWrapper`] actor
//! - Spawn actor in [`ActorSystem`] via builder pattern
//! - Register in [`ComponentRegistry`]
//!
//! # Architecture
//!
//! ComponentSpawner is part of Layer 3A (component/ module). It:
//! - Uses traits from `core/runtime/` (RuntimeEngine, ComponentLoader)
//! - Uses types from `core/component/` (ComponentId)
//! - Integrates with `airssys-rt` (ActorSystem, ActorAddress)
//! - Receives concrete implementations via generic type parameters (S6.2)
//!
//! # S6.2 Compliance
//!
//! `ComponentSpawner<E, L>` uses generics for static dispatch:
//! - `E: RuntimeEngine` for WASM execution
//! - `L: ComponentLoader` for loading WASM binaries
//!
//! This follows PROJECTS_STANDARD.md S6.2:
//! > Hierarchy: 1. Concrete types first -> 2. Generics with constraints -> 3. `dyn` only as last resort
//!
//! # Module Boundary Rules
//!
//! - CAN import: `core/`, `airssys-rt`
//! - CANNOT import: `runtime/`, `security/`, `system/`
//!
//! # Known Limitations
//!
//! - Actors are spawned without supervision (WASM-TASK-040 will add this)
//! - `stop()` only unregisters from registry; sending shutdown messages
//!   requires system/ module integration with the message broker
//!
//! # References
//!
//! - ADR-WASM-031: Component & Messaging Module Design
//! - ADR-WASM-023: Module Boundary Enforcement
//! - PROJECTS_STANDARD.md S6.2: Avoid `dyn` Patterns

// Layer 1: Standard library imports
use std::fmt;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use airssys_rt::broker::MessageBroker;
use airssys_rt::system::ActorSystem;
use airssys_rt::util::ActorAddress;
use airssys_rt::SystemError;
use thiserror::Error;

// Layer 3: Internal module imports
use crate::core::component::id::ComponentId;
use crate::core::runtime::errors::WasmError;
use crate::core::runtime::traits::{ComponentLoader, RuntimeEngine};

use super::registry::{ComponentRegistry, RegistryError};
use super::wrapper::{ComponentActorMessage, ComponentWrapper};

/// Errors that can occur during component spawning operations.
///
/// Each variant maps to a specific failure mode in the spawn lifecycle.
#[derive(Debug, Error)]
pub enum SpawnerError {
    /// Failed to load component bytes from storage.
    #[error("Failed to load component '{0}': {1}")]
    LoadFailed(String, WasmError),

    /// Component validation failed (invalid WASM binary).
    #[error("Component validation failed for '{0}': {1}")]
    ValidationFailed(String, WasmError),

    /// Actor system failed to spawn the actor.
    #[error("Failed to spawn actor for component '{0}': {1}")]
    SpawnFailed(String, SystemError),

    /// Registry operation failed (e.g., lock poisoned).
    #[error("Registry error: {0}")]
    RegistryError(#[from] RegistryError),

    /// Component is already spawned (duplicate spawn attempt).
    #[error("Component already spawned: {0}")]
    AlreadySpawned(String),

    /// Component is not spawned (stop called on unknown component).
    #[error("Component not spawned: {0}")]
    NotSpawned(String),
}

/// Spawns and manages component actors in the airssys-rt actor system.
///
/// `ComponentSpawner<E, L>` orchestrates the full lifecycle of WASM component actors:
///
/// 1. **Load**: Fetches component bytes via [`ComponentLoader`]
/// 2. **Validate**: Validates the WASM binary structure
/// 3. **Wrap**: Creates a [`ComponentWrapper<E>`] actor instance
/// 4. **Spawn**: Spawns the actor in the [`ActorSystem`]
/// 5. **Register**: Records the component in [`ComponentRegistry`]
///
/// # S6.2 Compliance
///
/// Uses generic type parameters for static dispatch (no `dyn` trait objects):
/// - `E: RuntimeEngine` - for WASM execution
/// - `L: ComponentLoader` - for loading WASM bytes
///
/// # Thread Safety
///
/// `ComponentSpawner<E, L>` is `Send + Sync` when `E` and `L` are `Send + Sync`
/// (which they are, since both traits require `Send + Sync`).
/// All internal state is managed through `Arc` references.
///
/// # Examples
///
/// ```rust,ignore
/// use std::sync::Arc;
/// use airssys_wasm::component::spawner::ComponentSpawner;
///
/// let spawner = ComponentSpawner::new(
///     Arc::new(wasmtime_engine),
///     Arc::new(file_loader),
///     Arc::new(registry),
/// );
///
/// // Spawn a component
/// let id = ComponentId::new("system", "database", "prod");
/// spawner.spawn(&actor_system, id).await?;
/// ```
pub struct ComponentSpawner<E: RuntimeEngine, L: ComponentLoader> {
    /// Runtime engine for WASM execution (static dispatch per S6.2)
    engine: Arc<E>,

    /// Component loader for fetching and validating WASM bytes (static dispatch per S6.2)
    loader: Arc<L>,

    /// Registry for tracking spawned components (shared)
    registry: Arc<ComponentRegistry>,
}

impl<E: RuntimeEngine, L: ComponentLoader> ComponentSpawner<E, L> {
    /// Creates a new ComponentSpawner with injected dependencies.
    ///
    /// # Arguments
    ///
    /// * `engine` - Runtime engine for executing WASM components
    /// * `loader` - Component loader for loading and validating WASM binaries
    /// * `registry` - Component registry for tracking spawned components
    pub fn new(engine: Arc<E>, loader: Arc<L>, registry: Arc<ComponentRegistry>) -> Self {
        Self {
            engine,
            loader,
            registry,
        }
    }

    /// Spawns a new component actor in the given actor system.
    ///
    /// Performs the full spawn lifecycle:
    /// 1. Check if component is already spawned (reject duplicates)
    /// 2. Load component bytes via ComponentLoader
    /// 3. Validate WASM binary via ComponentLoader::validate()
    /// 4. Create ComponentWrapper<E> with injected RuntimeEngine
    /// 5. Spawn actor in ActorSystem using builder pattern
    /// 6. Register the component in ComponentRegistry
    ///
    /// # Arguments
    ///
    /// * `actor_system` - The actor system to spawn the component in
    /// * `id` - The component identifier
    ///
    /// # Returns
    ///
    /// The `ActorAddress` of the spawned component actor.
    ///
    /// # Errors
    ///
    /// - [`SpawnerError::AlreadySpawned`] - Component already registered
    /// - [`SpawnerError::LoadFailed`] - Cannot load component bytes
    /// - [`SpawnerError::ValidationFailed`] - WASM binary validation failed
    /// - [`SpawnerError::SpawnFailed`] - Actor system spawn failed
    /// - [`SpawnerError::RegistryError`] - Registry operation failed
    ///
    /// # Type Parameters
    ///
    /// * `B` - The message broker type used by the actor system
    pub async fn spawn<B>(
        &self,
        actor_system: &ActorSystem<ComponentActorMessage, B>,
        id: ComponentId,
    ) -> Result<ActorAddress, SpawnerError>
    where
        E: 'static,
        B: MessageBroker<ComponentActorMessage> + Clone + Send + Sync + 'static,
    {
        // Step 1: Check if already spawned
        let already_exists = self.registry.contains(&id)?;
        if already_exists {
            return Err(SpawnerError::AlreadySpawned(id.to_string()));
        }

        // Step 2: Load component bytes
        let id_str = id.to_string();
        let bytes = self
            .loader
            .load_bytes(&id)
            .map_err(|e| SpawnerError::LoadFailed(id_str.clone(), e))?;

        // Step 3: Validate WASM binary
        self.loader
            .validate(&bytes)
            .map_err(|e| SpawnerError::ValidationFailed(id_str.clone(), e))?;

        // Step 4: Create ComponentWrapper<E> actor (static dispatch)
        let wrapper = ComponentWrapper::new(id.clone(), Arc::clone(&self.engine), bytes);

        // Step 5: Spawn actor via builder pattern
        let actor_name = format!("wasm-component-{}", id_str);
        let address = actor_system
            .spawn()
            .with_name(actor_name)
            .spawn(wrapper)
            .await
            .map_err(|e| SpawnerError::SpawnFailed(id_str, e))?;

        // Step 6: Register in registry
        self.registry.register(id, address.clone())?;

        Ok(address)
    }

    /// Stops a component by unregistering it from the registry.
    ///
    /// # Arguments
    ///
    /// * `id` - The component identifier to stop
    ///
    /// # Returns
    ///
    /// The `ActorAddress` of the stopped component (for caller to handle shutdown
    /// messaging if needed).
    ///
    /// # Errors
    ///
    /// - [`SpawnerError::NotSpawned`] - Component not found in registry
    /// - [`SpawnerError::RegistryError`] - Registry operation failed
    ///
    /// # Known Limitation
    ///
    /// This method only unregisters the component from the registry.
    /// It does NOT send a `ComponentActorMessage::Shutdown` message to the actor,
    /// because ActorAddress is an identifier only and cannot send messages directly.
    /// Sending shutdown messages requires integration with the message broker,
    /// which is the responsibility of the system/ module (Layer 4).
    pub fn stop(&self, id: &ComponentId) -> Result<ActorAddress, SpawnerError> {
        let removed = self.registry.unregister(id)?;
        match removed {
            Some(address) => Ok(address),
            None => Err(SpawnerError::NotSpawned(id.to_string())),
        }
    }

    /// Checks whether a component is currently spawned (registered).
    ///
    /// # Arguments
    ///
    /// * `id` - The component identifier to check
    ///
    /// # Errors
    ///
    /// - [`SpawnerError::RegistryError`] - Registry operation failed
    pub fn is_spawned(&self, id: &ComponentId) -> Result<bool, SpawnerError> {
        let result = self.registry.contains(id)?;
        Ok(result)
    }

    /// Returns the number of currently spawned components.
    ///
    /// # Errors
    ///
    /// - [`SpawnerError::RegistryError`] - Registry operation failed
    pub fn spawned_count(&self) -> Result<usize, SpawnerError> {
        let count = self.registry.count()?;
        Ok(count)
    }

    /// Returns a reference to the component registry.
    pub fn registry(&self) -> &Arc<ComponentRegistry> {
        &self.registry
    }
}

// Manual Debug implementation - engine and loader use opaque display
// (RuntimeEngine and ComponentLoader traits do not require Debug)
impl<E: RuntimeEngine, L: ComponentLoader> fmt::Debug for ComponentSpawner<E, L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ComponentSpawner")
            .field("engine", &"<RuntimeEngine>")
            .field("loader", &"<ComponentLoader>")
            .field("registry", &self.registry)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::component::handle::ComponentHandle;
    use crate::core::component::message::{ComponentMessage, MessagePayload};
    use std::sync::atomic::{AtomicBool, Ordering};

    // ========================================
    // Mock RuntimeEngine for Testing
    // ========================================

    struct MockRuntimeEngine {
        should_fail_load: AtomicBool,
    }

    impl MockRuntimeEngine {
        fn new() -> Self {
            Self {
                should_fail_load: AtomicBool::new(false),
            }
        }
    }

    impl RuntimeEngine for MockRuntimeEngine {
        fn load_component(
            &self,
            id: &ComponentId,
            _bytes: &[u8],
        ) -> Result<ComponentHandle, WasmError> {
            if self.should_fail_load.load(Ordering::SeqCst) {
                return Err(WasmError::InstantiationFailed(
                    "Mock load failure".to_string(),
                ));
            }
            Ok(ComponentHandle::new(id.clone(), 1))
        }

        fn unload_component(&self, _handle: &ComponentHandle) -> Result<(), WasmError> {
            Ok(())
        }

        fn call_handle_message(
            &self,
            _handle: &ComponentHandle,
            _msg: &ComponentMessage,
        ) -> Result<Option<MessagePayload>, WasmError> {
            Ok(None)
        }

        fn call_handle_callback(
            &self,
            _handle: &ComponentHandle,
            _msg: &ComponentMessage,
        ) -> Result<(), WasmError> {
            Ok(())
        }
    }

    // ========================================
    // Mock ComponentLoader for Testing
    // ========================================

    struct MockComponentLoader {
        should_fail_load: bool,
        should_fail_validate: bool,
    }

    impl MockComponentLoader {
        fn new() -> Self {
            Self {
                should_fail_load: false,
                should_fail_validate: false,
            }
        }

        fn with_load_failure() -> Self {
            Self {
                should_fail_load: true,
                should_fail_validate: false,
            }
        }

        fn with_validation_failure() -> Self {
            Self {
                should_fail_load: false,
                should_fail_validate: true,
            }
        }
    }

    impl ComponentLoader for MockComponentLoader {
        fn load_bytes(&self, _id: &ComponentId) -> Result<Vec<u8>, WasmError> {
            if self.should_fail_load {
                Err(WasmError::ComponentNotFound("mock load error".to_string()))
            } else {
                Ok(vec![0u8; 100])
            }
        }

        fn validate(&self, _bytes: &[u8]) -> Result<(), WasmError> {
            if self.should_fail_validate {
                Err(WasmError::InvalidComponent(
                    "mock validation failure".to_string(),
                ))
            } else {
                Ok(())
            }
        }
    }

    // ========================================
    // Helper Functions (all using concrete types - S6.2)
    // ========================================

    fn create_spawner(
        loader: MockComponentLoader,
    ) -> ComponentSpawner<MockRuntimeEngine, MockComponentLoader> {
        let engine = Arc::new(MockRuntimeEngine::new());
        let registry = Arc::new(ComponentRegistry::new());
        ComponentSpawner::new(engine, Arc::new(loader), registry)
    }

    fn create_test_id(name: &str) -> ComponentId {
        ComponentId::new("test", name, "v1")
    }

    // ========================================
    // Constructor Tests
    // ========================================

    #[test]
    fn test_spawner_creation() {
        let spawner = create_spawner(MockComponentLoader::new());

        let count = spawner.spawned_count();
        assert!(count.is_ok());
        assert_eq!(count.unwrap(), 0);
    }

    #[test]
    fn test_spawner_debug_impl() {
        let spawner = create_spawner(MockComponentLoader::new());

        let debug_str = format!("{:?}", spawner);
        assert!(debug_str.contains("ComponentSpawner"));
        assert!(debug_str.contains("<RuntimeEngine>"));
        assert!(debug_str.contains("<ComponentLoader>"));
    }

    #[test]
    fn test_spawner_registry_accessor() {
        let spawner = create_spawner(MockComponentLoader::new());

        let registry = spawner.registry();
        assert_eq!(registry.count().unwrap(), 0);
    }

    // ========================================
    // is_spawned Tests
    // ========================================

    #[test]
    fn test_is_spawned_returns_false_for_unregistered() {
        let spawner = create_spawner(MockComponentLoader::new());
        let id = create_test_id("nonexistent");

        assert!(!spawner.is_spawned(&id).unwrap());
    }

    // ========================================
    // stop() Tests
    // ========================================

    #[test]
    fn test_stop_returns_error_for_unregistered() {
        let spawner = create_spawner(MockComponentLoader::new());
        let id = create_test_id("nonexistent");

        let result = spawner.stop(&id);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("not spawned"));
    }

    #[test]
    fn test_stop_removes_registered_component() {
        let spawner = create_spawner(MockComponentLoader::new());
        let id = create_test_id("test-component");

        // Manually register a component to simulate a spawned state
        let mock_address = ActorAddress::named("test-actor");
        spawner
            .registry()
            .register(id.clone(), mock_address)
            .unwrap();

        assert!(spawner.is_spawned(&id).unwrap());

        // Stop should succeed and remove from registry
        let result = spawner.stop(&id);
        assert!(result.is_ok());
        assert!(!spawner.is_spawned(&id).unwrap());
    }

    #[test]
    fn test_stop_returns_actor_address() {
        let spawner = create_spawner(MockComponentLoader::new());
        let id = create_test_id("test-component");

        let mock_address = ActorAddress::named("my-actor");
        spawner
            .registry()
            .register(id.clone(), mock_address)
            .unwrap();

        let result = spawner.stop(&id);
        assert!(result.is_ok());
        let address = result.unwrap();
        assert_eq!(address.name(), Some("my-actor"));
    }

    // ========================================
    // spawn() - Load Failure Tests
    // ========================================

    #[tokio::test]
    async fn test_spawn_fails_on_load_error() {
        let spawner = create_spawner(MockComponentLoader::with_load_failure());

        let id = create_test_id("fail-load");
        let load_result = spawner.loader.load_bytes(&id);
        assert!(load_result.is_err());

        // Verify the SpawnerError wrapping
        let wasm_err = load_result.unwrap_err();
        let spawner_err = SpawnerError::LoadFailed(id.to_string(), wasm_err);
        assert!(spawner_err.to_string().contains("Failed to load"));
        assert!(spawner_err.to_string().contains("test/fail-load/v1"));
    }

    // ========================================
    // spawn() - Validation Failure Tests
    // ========================================

    #[test]
    fn test_spawn_fails_on_validation_error() {
        let spawner = create_spawner(MockComponentLoader::with_validation_failure());

        // Test the loader validation path
        let bytes = vec![0u8; 100];
        let validate_result = spawner.loader.validate(&bytes);
        assert!(validate_result.is_err());

        // Verify the SpawnerError wrapping
        let wasm_err = validate_result.unwrap_err();
        let id = create_test_id("fail-validate");
        let spawner_err = SpawnerError::ValidationFailed(id.to_string(), wasm_err);
        assert!(spawner_err
            .to_string()
            .contains("Component validation failed"));
    }

    // ========================================
    // SpawnerError Tests
    // ========================================

    #[test]
    fn test_spawner_error_load_failed_display() {
        let wasm_err = WasmError::ComponentNotFound("test.wasm".to_string());
        let err = SpawnerError::LoadFailed("test/comp/v1".to_string(), wasm_err);
        let display = err.to_string();
        assert!(display.contains("Failed to load"));
        assert!(display.contains("test/comp/v1"));
        assert!(display.contains("Component not found"));
    }

    #[test]
    fn test_spawner_error_validation_failed_display() {
        let wasm_err = WasmError::InvalidComponent("bad magic".to_string());
        let err = SpawnerError::ValidationFailed("test/comp/v1".to_string(), wasm_err);
        let display = err.to_string();
        assert!(display.contains("validation failed"));
        assert!(display.contains("test/comp/v1"));
        assert!(display.contains("Invalid component"));
    }

    #[test]
    fn test_spawner_error_already_spawned_display() {
        let err = SpawnerError::AlreadySpawned("test/comp/v1".to_string());
        let display = err.to_string();
        assert!(display.contains("already spawned"));
        assert!(display.contains("test/comp/v1"));
    }

    #[test]
    fn test_spawner_error_not_spawned_display() {
        let err = SpawnerError::NotSpawned("test/comp/v1".to_string());
        let display = err.to_string();
        assert!(display.contains("not spawned"));
        assert!(display.contains("test/comp/v1"));
    }

    #[test]
    fn test_spawner_error_from_registry_error() {
        let registry_err = RegistryError::LockPoisoned("test".to_string());
        let spawner_err: SpawnerError = registry_err.into();
        assert!(spawner_err.to_string().contains("Registry"));
    }

    #[test]
    fn test_spawner_error_debug_impl() {
        let err = SpawnerError::AlreadySpawned("test".to_string());
        let debug = format!("{:?}", err);
        assert!(debug.contains("AlreadySpawned"));
    }

    // ========================================
    // Send + Sync Bounds Tests
    // ========================================

    #[test]
    fn test_spawner_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<ComponentSpawner<MockRuntimeEngine, MockComponentLoader>>();
        assert_sync::<ComponentSpawner<MockRuntimeEngine, MockComponentLoader>>();
    }

    #[test]
    fn test_spawner_error_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<SpawnerError>();
        assert_sync::<SpawnerError>();
    }

    // ========================================
    // Integration Tests with ActorSystem
    // ========================================

    #[tokio::test]
    async fn test_spawn_full_lifecycle() {
        use airssys_rt::broker::InMemoryMessageBroker;
        use airssys_rt::system::SystemConfig;

        let broker = InMemoryMessageBroker::<ComponentActorMessage>::new();
        let system = ActorSystem::new(SystemConfig::default(), broker);

        let engine = Arc::new(MockRuntimeEngine::new());
        let loader = Arc::new(MockComponentLoader::new());
        let registry = Arc::new(ComponentRegistry::new());

        let spawner = ComponentSpawner::new(engine, loader, registry);

        let id = create_test_id("lifecycle-test");

        // Spawn component
        let result = spawner.spawn(&system, id.clone()).await;
        assert!(result.is_ok());
        let address = result.unwrap();
        assert!(address.name().is_some());

        // Verify component is registered
        assert!(spawner.is_spawned(&id).unwrap());
        assert_eq!(spawner.spawned_count().unwrap(), 1);

        // Stop component
        let stop_result = spawner.stop(&id);
        assert!(stop_result.is_ok());
        assert!(!spawner.is_spawned(&id).unwrap());
        assert_eq!(spawner.spawned_count().unwrap(), 0);

        // Cleanup
        system.force_shutdown().await;
    }

    #[tokio::test]
    async fn test_spawn_duplicate_rejected() {
        use airssys_rt::broker::InMemoryMessageBroker;
        use airssys_rt::system::SystemConfig;

        let broker = InMemoryMessageBroker::<ComponentActorMessage>::new();
        let system = ActorSystem::new(SystemConfig::default(), broker);

        let engine = Arc::new(MockRuntimeEngine::new());
        let loader = Arc::new(MockComponentLoader::new());
        let registry = Arc::new(ComponentRegistry::new());

        let spawner = ComponentSpawner::new(engine, loader, registry);

        let id = create_test_id("duplicate-test");

        // First spawn succeeds
        let result1 = spawner.spawn(&system, id.clone()).await;
        assert!(result1.is_ok());

        // Second spawn of same ID fails
        let result2 = spawner.spawn(&system, id).await;
        assert!(result2.is_err());
        let err = result2.unwrap_err();
        assert!(err.to_string().contains("already spawned"));

        // Cleanup
        system.force_shutdown().await;
    }

    #[tokio::test]
    async fn test_spawn_load_failure_does_not_register() {
        use airssys_rt::broker::InMemoryMessageBroker;
        use airssys_rt::system::SystemConfig;

        let broker = InMemoryMessageBroker::<ComponentActorMessage>::new();
        let system = ActorSystem::new(SystemConfig::default(), broker);

        let engine = Arc::new(MockRuntimeEngine::new());
        let loader = Arc::new(MockComponentLoader::with_load_failure());
        let registry = Arc::new(ComponentRegistry::new());

        let spawner = ComponentSpawner::new(engine, loader, registry);

        let id = create_test_id("load-fail");

        let result = spawner.spawn(&system, id.clone()).await;
        assert!(result.is_err());

        // Should NOT be registered
        assert!(!spawner.is_spawned(&id).unwrap());
        assert_eq!(spawner.spawned_count().unwrap(), 0);

        system.force_shutdown().await;
    }

    #[tokio::test]
    async fn test_spawn_validation_failure_does_not_register() {
        use airssys_rt::broker::InMemoryMessageBroker;
        use airssys_rt::system::SystemConfig;

        let broker = InMemoryMessageBroker::<ComponentActorMessage>::new();
        let system = ActorSystem::new(SystemConfig::default(), broker);

        let engine = Arc::new(MockRuntimeEngine::new());
        let loader = Arc::new(MockComponentLoader::with_validation_failure());
        let registry = Arc::new(ComponentRegistry::new());

        let spawner = ComponentSpawner::new(engine, loader, registry);

        let id = create_test_id("validate-fail");

        let result = spawner.spawn(&system, id.clone()).await;
        assert!(result.is_err());

        // Should NOT be registered
        assert!(!spawner.is_spawned(&id).unwrap());

        system.force_shutdown().await;
    }

    #[tokio::test]
    async fn test_spawn_multiple_components() {
        use airssys_rt::broker::InMemoryMessageBroker;
        use airssys_rt::system::SystemConfig;

        let broker = InMemoryMessageBroker::<ComponentActorMessage>::new();
        let system = ActorSystem::new(SystemConfig::default(), broker);

        let engine = Arc::new(MockRuntimeEngine::new());
        let loader = Arc::new(MockComponentLoader::new());
        let registry = Arc::new(ComponentRegistry::new());

        let spawner = ComponentSpawner::new(engine, loader, registry);

        let id1 = create_test_id("comp-1");
        let id2 = create_test_id("comp-2");
        let id3 = create_test_id("comp-3");

        let result1 = spawner.spawn(&system, id1.clone()).await;
        assert!(result1.is_ok());

        let result2 = spawner.spawn(&system, id2.clone()).await;
        assert!(result2.is_ok());

        let result3 = spawner.spawn(&system, id3.clone()).await;
        assert!(result3.is_ok());

        assert_eq!(spawner.spawned_count().unwrap(), 3);
        assert!(spawner.is_spawned(&id1).unwrap());
        assert!(spawner.is_spawned(&id2).unwrap());
        assert!(spawner.is_spawned(&id3).unwrap());

        system.force_shutdown().await;
    }

    #[tokio::test]
    async fn test_spawn_actor_name_format() {
        use airssys_rt::broker::InMemoryMessageBroker;
        use airssys_rt::system::SystemConfig;

        let broker = InMemoryMessageBroker::<ComponentActorMessage>::new();
        let system = ActorSystem::new(SystemConfig::default(), broker);

        let engine = Arc::new(MockRuntimeEngine::new());
        let loader = Arc::new(MockComponentLoader::new());
        let registry = Arc::new(ComponentRegistry::new());

        let spawner = ComponentSpawner::new(engine, loader, registry);

        let id = ComponentId::new("system", "database", "prod");
        let result = spawner.spawn(&system, id).await;
        assert!(result.is_ok());

        let address = result.unwrap();
        // Actor name should be "wasm-component-system/database/prod"
        assert!(address.name().unwrap().starts_with("wasm-component-"));

        system.force_shutdown().await;
    }
}
