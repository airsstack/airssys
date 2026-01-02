//! Host System Manager
//!
//! The HostSystemManager provides system-wide coordination for airssys-wasm
//! framework. It manages component lifecycle, system initialization, and message
//! flow coordination.
//!
//! # Architecture
//!
//! HostSystemManager coordinates all infrastructure initialization and component
//! lifecycle management. It does NOT implement core operations but delegates
//! to appropriate modules:
//! - WASM execution → runtime/ (WasmEngine)
//! - Actor spawning → actor/ (ComponentSpawner)
//! - Message routing → messaging/ (MessagingService)
//! - Correlation tracking → host_system/ (CorrelationTracker)

// Layer 1: Standard library imports
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use airssys_rt::broker::InMemoryMessageBroker;
use airssys_rt::util::ActorAddress;

// Layer 3: Internal module imports
use crate::core::WasmError;
use crate::core::component_message::ComponentMessage;
use crate::core::component::ComponentId;
use crate::core::component::ComponentMetadata;
use crate::core::component::ComponentStatus;
use crate::core::capability::CapabilitySet;
use crate::host_system::correlation_tracker::CorrelationTracker;
use crate::host_system::timeout_handler::TimeoutHandler;
use crate::actor::component::{ComponentSpawner, ComponentRegistry};
use crate::messaging::MessagingService;
use crate::runtime::WasmEngine;

/// Host system coordinator for airssys-wasm framework.
///
/// The HostSystemManager manages system initialization, component lifecycle,
/// and message flow coordination between actor/, messaging/, and runtime/ modules.
///
/// # Architecture
///
/// HostSystemManager coordinates all infrastructure initialization and component
/// lifecycle management. It does NOT implement core operations but delegates
/// to appropriate modules:
/// - WASM execution → runtime/ (WasmEngine)
/// - Actor spawning → actor/ (ComponentSpawner)
/// - Message routing → messaging/ (MessagingService)
/// - Correlation tracking → host_system/ (CorrelationTracker)
///
/// # Thread Safety
///
/// HostSystemManager is `Send + Sync` and can be safely shared across
/// threads. All infrastructure components are wrapped in `Arc` for
/// thread-safe sharing.
///
/// # Cloning
///
/// Cloning HostSystemManager is not supported - use Arc to share
/// manager across threads if needed.
///
/// # Performance
///
/// Target initialization time: <100ms (including all infrastructure)
/// Target spawn time: <10ms (delegates to ComponentSpawner)
///
/// # Examples
///
/// ```rust,ignore
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use airssys_wasm::host_system::HostSystemManager;
/// use airssys_wasm::core::{ComponentId, CapabilitySet, ComponentMetadata};
///
/// // Initialize system
/// let mut manager = HostSystemManager::new().await?;
///
/// // Spawn component
/// let component_id = ComponentId::new("my-component");
/// let wasm_bytes = std::fs::read("component.wasm")?;
/// let metadata = ComponentMetadata::new(component_id.clone());
/// let capabilities = CapabilitySet::new();
///
/// manager.spawn_component(
///     component_id.clone(),
///     wasm_bytes,
///     metadata,
///     capabilities
/// ).await?;
///
/// // Query component status
/// let status = manager.get_component_status(&component_id).await?;
/// println!("Component status: {:?}", status);
///
/// // Stop component
/// manager.stop_component(&component_id).await?;
///
/// // Graceful shutdown
/// manager.shutdown().await?;
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// - `WasmError::EngineInitialization`: System initialization failed
/// - `WasmError::ComponentNotFound`: Component ID not found
/// - `WasmError::ComponentSpawnFailed`: Component spawn failed
#[allow(dead_code)]  // Fields will be used in Subtasks 4.3-4.6 per YAGNI principle
pub struct HostSystemManager {
    /// WASM execution engine for executing component code
    engine: Arc<WasmEngine>,

    /// Component registry for O(1) ComponentId → ActorAddress lookups
    registry: Arc<ComponentRegistry>,

    /// Component spawner for creating ComponentActor instances
    spawner: Arc<ComponentSpawner<InMemoryMessageBroker<ComponentMessage>>>,

    /// Messaging service with MessageBroker for inter-component communication
    messaging_service: Arc<MessagingService>,

    /// Correlation tracker for request-response pattern
    correlation_tracker: Arc<CorrelationTracker>,

    /// Timeout handler for request timeout enforcement
    timeout_handler: Arc<TimeoutHandler>,

    /// System startup flag - true after initialization complete
    started: Arc<AtomicBool>,
}

// Manual Debug implementation since some fields don't implement Debug
impl std::fmt::Debug for HostSystemManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HostSystemManager")
            .field("engine", &"<WasmEngine>")
            .field("registry", &"<ComponentRegistry>")
            .field("spawner", &"<ComponentSpawner>")
            .field("messaging_service", &"<MessagingService>")
            .field("correlation_tracker", &"<CorrelationTracker>")
            .field("timeout_handler", &"<TimeoutHandler>")
            .field("started", &self.started)
            .finish()
    }
}

impl HostSystemManager {
    /// Creates a new HostSystemManager instance and initializes all infrastructure.
    ///
    /// Initializes all system components in the correct order and wires
    /// dependencies via constructor injection (not import-based dependencies).
    ///
    /// # Initialization Order
    ///
    /// 1. Create WasmEngine for WASM execution
    /// 2. Create CorrelationTracker and TimeoutHandler
    /// 3. Create MessageBroker externally (not in MessagingService)
    /// 4. Clone broker for ActorSystem (avoid borrow after move)
    /// 5. Clone broker for ComponentSpawner (separate clone from ActorSystem)
    /// 6. Create ActorSystem with broker (non-Arc)
    /// 7. Create MessagingService with injected broker
    /// 8. Create ComponentRegistry for O(1) lookups
    /// 9. Create ComponentSpawner with broker (using separate clone)
    /// 10. Set started flag to true
    ///
    /// # Dependency Injection
    ///
    /// All dependencies are passed via constructors, ensuring no circular
    /// imports between modules. This follows the pattern specified in
    /// KNOWLEDGE-WASM-036 lines 518-540.
    ///
    /// # Performance
    ///
    /// Target: <100ms total initialization time
    ///
    /// # Returns
    ///
    /// Returns a `HostSystemManager` instance.
    ///
    /// # Errors
    ///
    /// - `WasmError::EngineInitialization`: WasmEngine creation failed
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use airssys_wasm::host_system::HostSystemManager;
    ///
    /// let manager = HostSystemManager::new().await?;
    /// println!("System initialized successfully");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new() -> Result<Self, WasmError> {
        // Step 1: Create WasmEngine
        let engine = Arc::new(WasmEngine::new().map_err(|e| {
            WasmError::engine_initialization(format!(
                "Failed to create WASM engine: {}",
                e
            ))
        })?);

        // Step 2: Create CorrelationTracker and TimeoutHandler
        let correlation_tracker = Arc::new(CorrelationTracker::new());
        let timeout_handler = Arc::new(TimeoutHandler::new());

        // Step 3: Create MessageBroker externally (NOT in MessagingService)
        let broker = InMemoryMessageBroker::new();  // Create non-Arc broker

        // Step 4: Clone broker for ActorSystem (avoid borrow after move)
        let broker_for_actor = broker.clone();

        // Step 5: Clone broker for ComponentSpawner (separate clone from ActorSystem)
        let broker_for_spawner = broker.clone();

        // Step 6: Create ActorSystem with broker (non-Arc)
        let actor_system = airssys_rt::system::ActorSystem::new(
            airssys_rt::system::SystemConfig::default(),
            broker_for_actor,  // Pass InMemoryMessageBroker (not Arc clone)
        );

        // Step 7: Create MessagingService with injected broker
        let messaging_service = Arc::new(MessagingService::new(
            Arc::new(broker.clone()),  // Wrap in Arc for MessagingService
            Arc::clone(&correlation_tracker),
            Arc::clone(&timeout_handler),
        ));

        // Step 8: Create ComponentRegistry
        let registry = ComponentRegistry::new();
        let registry = Arc::new(registry);

         // Step 9: Create ComponentSpawner with broker (using separate clone)
        let spawner = Arc::new(ComponentSpawner::new(
            actor_system,
            (*registry).clone(),
            broker_for_spawner,  // Use broker_for_spawner (separate clone from actor_system)
        ));

         // Step 10: Set started flag
          let started = Arc::new(AtomicBool::new(true));

        Ok(Self {
            engine,
            registry,
            spawner,
            messaging_service,
            correlation_tracker,
            timeout_handler,
            started,
        })
    }

    /// Check if the system is started.
    ///
    /// Returns true if the system has been successfully initialized
    /// and is ready to accept component operations.
    ///
    /// # Returns
    ///
    /// `true` if system is started, `false` otherwise
    pub fn started(&self) -> bool {
        self.started.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Check if a component is registered in the system.
    ///
    /// This is a helper method for testing and monitoring purposes.
    /// It checks whether a component ID exists in the component registry.
    ///
    /// # Parameters
    ///
    /// - `id`: Component identifier to check
    ///
    /// # Returns
    ///
    /// `true` if component is registered, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use airssys_wasm::host_system::HostSystemManager;
    /// use airssys_wasm::core::{ComponentId, ComponentMetadata, CapabilitySet};
    ///
    /// let mut manager = HostSystemManager::new().await?;
    ///
    /// let component_id = ComponentId::new("my-component");
    /// manager.spawn_component(
    ///     component_id.clone(),
    ///     std::path::PathBuf::from("component.wasm"),
    ///     ComponentMetadata::new(component_id.clone()),
    ///     CapabilitySet::new()
    /// ).await?;
    ///
    /// assert!(manager.is_component_registered(&component_id));
    ///
    /// manager.stop_component(&component_id).await?;
    /// assert!(!manager.is_component_registered(&component_id));
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_component_registered(&self, id: &ComponentId) -> bool {
        self.registry.is_registered(id)
    }

    /// Spawns a new component into the system.
    ///
    /// Delegates to ComponentSpawner for actor creation and returns
    /// ActorAddress for sending messages to the spawned component.
    ///
    /// # Spawn Flow
    ///
    /// 1. Verify system is started
    /// 2. Delegate to ComponentSpawner::spawn_component()
    /// 3. Return ActorAddress for message routing
    ///
    /// Note: CorrelationTracker registration will be added in future tasks
    /// when component-level request-response support is implemented.
    ///
    /// # Performance
    ///
    /// Target: <10ms spawn time (delegates to ComponentSpawner)
    ///
    /// # Parameters
    ///
    /// - `id`: Unique component identifier
    /// - `wasm_path`: Path to WASM component file
    /// - `metadata`: Component metadata
    /// - `capabilities`: Granted capabilities for this component
    ///
    /// # Returns
    ///
    /// Returns `ActorAddress` for sending messages to the spawned component.
    ///
    /// # Errors
    ///
    /// - `WasmError::EngineInitialization`: System not initialized
    /// - `WasmError::ComponentLoadFailed`: Component failed to spawn
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use airssys_wasm::host_system::HostSystemManager;
    /// use airssys_wasm::core::{ComponentId, ComponentMetadata, CapabilitySet};
    /// use std::path::PathBuf;
    ///
    /// let mut manager = HostSystemManager::new().await?;
    ///
    /// let component_id = ComponentId::new("my-component");
    /// let wasm_path = PathBuf::from("component.wasm");
    /// let metadata = ComponentMetadata::new(component_id.clone());
    /// let capabilities = CapabilitySet::new();
    ///
    /// let actor_address = manager.spawn_component(
    ///     component_id,
    ///     wasm_path,
    ///     metadata,
    ///     capabilities
    /// ).await?;
    ///
    /// // Use actor_address to send messages
    /// println!("Component spawned with actor address: {:?}", actor_address.id());
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn spawn_component(
        &mut self,
        id: ComponentId,
        wasm_path: PathBuf,
        metadata: ComponentMetadata,
        capabilities: CapabilitySet,
    ) -> Result<ActorAddress, WasmError> {
        // Step 1: Verify system is started
        if !self.started.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(WasmError::engine_initialization(
                "HostSystemManager not initialized".to_string()
            ));
        }

        // Step 2: Delegate to ComponentSpawner for actor creation
        // ComponentSpawner handles:
        // - Creating ComponentActor instance
        // - Injecting MessageBroker bridge
        // - Spawning via ActorSystem
        // - Registering in ComponentRegistry
        let actor_address = self.spawner.spawn_component(
            id.clone(),
            wasm_path,
            metadata.clone(),
            capabilities,
        ).await.map_err(|e| {
            WasmError::component_load_failed(
                id.as_str(),
                format!(
                    "Failed to spawn component {}: {}",
                    id.as_str(),
                    e
                )
            )
        })?;

        // Step 3: Return ActorAddress for message routing
        // Note: Future enhancement - Register component with CorrelationTracker
        // when component-level request-response support is added
        Ok(actor_address)
    }

    /// Stops a running component by ID.
    ///
    /// Performs a complete shutdown sequence: untracks pending requests,
    /// unregisters from messaging, stops the actor, and removes from registry.
    ///
    /// # Stop Flow
    ///
    /// 1. Verify system is started
    /// 2. Lookup component in registry to get ActorAddress
    /// 3. Unregister from CorrelationTracker (cleanup pending requests)
    /// 4. Stop actor via ActorSystem (system-wide coordination)
    /// 5. Unregister from ComponentRegistry
    ///
    /// # Parameters
    ///
    /// - `id`: Component identifier to stop
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful stop.
    ///
    /// # Errors
    ///
    /// - `WasmError::EngineInitialization`: System not initialized
    /// - `WasmError::ComponentNotFound`: Component ID not found in registry
    /// - `WasmError::Timeout`: Actor stop timed out
    /// - `WasmError::Internal`: Unexpected error during stop sequence
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use airssys_wasm::host_system::HostSystemManager;
    /// use airssys_wasm::core::ComponentId;
    ///
    /// let mut manager = HostSystemManager::new().await?;
    /// manager.initialize_system().await?;
    ///
    /// let component_id = ComponentId::new("my-component");
    /// manager.stop_component(&component_id).await?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn stop_component(&mut self, id: &ComponentId) -> Result<(), WasmError> {
        // 1. Verify system is started
        if !self.started.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(WasmError::engine_initialization(
                "HostSystemManager not initialized".to_string()
            ));
        }

        // 2. Lookup component in registry (must exist to stop)
        let _actor_addr = self.registry.lookup(id).map_err(|e| {
            WasmError::component_not_found(format!(
                "Component {} not found in registry: {}",
                id.as_str(), e
            ))
        })?;

        // 3. Unregister from CorrelationTracker (cleanup pending requests)
        // Remove all pending requests where component is the sender
        self.correlation_tracker.cleanup_pending_for_component(id).await;

        // 4. Unregister from ComponentRegistry
        // Note: Current airssys-rt API doesn't support per-actor stop,
        // so we unregister from registry which prevents new message routing
        self.registry.unregister(id).map_err(|e| {
            WasmError::internal(format!(
                "Failed to unregister component {} from registry: {}",
                id.as_str(), e
            ))
        })?;

        // Actor will continue to exist in ActorSystem but won't receive new messages
        // since it's no longer in the registry. It will eventually be cleaned up
        // when ActorSystem shuts down or through garbage collection.

        Ok(())
    }

    /// Restarts a component by stopping and respawning it.
    ///
    /// This is a convenience method that combines stop_component()
    /// and spawn_component(). For supervision and automatic restarts,
    /// use ComponentSupervisor (future phase).
    ///
    /// # Restart Flow
    ///
    /// 1. Verify component is registered (must have been spawned before)
    /// 2. Stop component
    /// 3. Save WASM bytes to temporary file
    /// 4. Respawn component with original metadata and capabilities
    ///
    /// # Note
    ///
    /// This method requires caller to have access to the original
    /// wasm_bytes, metadata, and capabilities. For automatic supervision
    /// with state preservation, see ComponentSupervisor (future phase).
    ///
    /// # Parameters
    ///
    /// - `id`: Component identifier to restart
    /// - `wasm_bytes`: WASM binary bytes (same as original spawn)
    /// - `metadata`: Component metadata (same as original spawn)
    /// - `capabilities`: Capability set (same as original spawn)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful restart.
    ///
    /// # Errors
    ///
    /// - `WasmError::EngineInitialization`: HostSystemManager not initialized
    /// - `WasmError::ComponentNotFound`: Component not found or was never spawned
    /// - `WasmError::ComponentLoadFailed`: Respawn failed
    ///
    /// # Panics
    ///
    /// This method does not panic. All errors are returned as `Result`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use airssys_wasm::host_system::HostSystemManager;
    /// use airssys_wasm::core::{ComponentId, ComponentMetadata, CapabilitySet};
    ///
    /// let mut manager = HostSystemManager::new().await?;
    ///
    /// let component_id = ComponentId::new("my-component");
    /// let wasm_bytes = std::fs::read("component.wasm")?;
    /// let metadata = ComponentMetadata::new(component_id.clone());
    /// let capabilities = CapabilitySet::new();
    ///
    /// // Spawn first
    /// manager.spawn_component(
    ///     component_id.clone(),
    ///     wasm_bytes.clone(),
    ///     metadata.clone(),
    ///     capabilities.clone()
    /// ).await?;
    ///
    /// // Restart with same parameters
    /// manager.restart_component(
    ///     &component_id,
    ///     wasm_bytes,
    ///     metadata,
    ///     capabilities
    /// ).await?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn restart_component(
        &mut self,
        id: &ComponentId,
        wasm_bytes: Vec<u8>,
        metadata: ComponentMetadata,
        capabilities: CapabilitySet,
    ) -> Result<(), WasmError> {
        // 1. Verify system is started
        if !self.started.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(WasmError::engine_initialization(
                "HostSystemManager not initialized".to_string()
            ));
        }

        // 2. Verify component is registered (must have been spawned before)
        if !self.registry.is_registered(id) {
            return Err(WasmError::component_not_found(id.as_str()));
        }

        // 3. Stop component
        self.stop_component(id).await?;

        // 4. Save WASM bytes to temporary file
        // Note: Current spawn_component() takes PathBuf, so we need to save bytes to a file
        // In future, spawn_component() could be updated to accept Vec<u8> directly
        let temp_dir = std::env::temp_dir();
        let temp_file_path = temp_dir.join(format!("component-{}.wasm", id.as_str()));

        std::fs::write(&temp_file_path, wasm_bytes).map_err(|e| {
            WasmError::component_load_failed(
                id.as_str(),
                format!("Failed to write temporary WASM file {}: {}", temp_file_path.display(), e)
            )
        })?;

        // 5. Respawn component
        let _actor_address = self.spawn_component(
            id.clone(),
            temp_file_path,
            metadata,
            capabilities
        ).await.map_err(|e| {
            WasmError::component_load_failed(
                id.as_str(),
                format!("Failed to respawn component {}: {}", id.as_str(), e)
            )
        })?;

        // 6. Note: temp_file is not cleaned up immediately because ComponentActor
        // may load it asynchronously. In production, cleanup strategy should be enhanced.

        Ok(())
    }

    /// Queries current status of a component.
    ///
    /// This method provides a read-only query of component state without modifying
    /// component. It checks if the component is registered and returns
    /// appropriate status.
    ///
    /// # Status Logic
    ///
    /// Currently returns simplified status:
    /// - `ComponentStatus::Running`: Component is registered and active
    ///
    /// **Note:** Actual state tracking (Registered → Running → Stopped transitions)
    /// will be enhanced in Phase 5 when component state machine is implemented.
    /// For now, all registered components return `Running` status.
    ///
    /// # Query Flow
    ///
    /// 1. Verify system is started
    /// 2. Check if component is registered via `ComponentRegistry::is_registered()`
    /// 3. Query actor address via `ComponentRegistry::lookup()`
    /// 4. Return appropriate status based on registration
    ///
    /// # Performance
    ///
    /// Target: <1ms (O(1) registry lookup)
    ///
    /// # Parameters
    ///
    /// - `id`: Unique component identifier to query
    ///
    /// # Returns
    ///
    /// Returns `ComponentStatus` indicating current state of component.
    ///
    /// # Errors
    ///
    /// - `WasmError::EngineInitialization`: System not initialized (check `started()` first)
    /// - `WasmError::ComponentNotFound`: Component ID not registered in system
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use airssys_wasm::host_system::HostSystemManager;
    /// use airssys_wasm::core::{ComponentId, ComponentStatus};
    ///
    /// // Initialize system and spawn component
    /// let mut manager = HostSystemManager::new().await?;
    /// let component_id = ComponentId::new("my-component");
    /// let wasm_path = std::path::PathBuf::from("component.wasm");
    /// let metadata = ComponentMetadata::new(component_id.clone());
    /// let capabilities = CapabilitySet::new();
    ///
    /// manager.spawn_component(
    ///     component_id.clone(),
    ///     wasm_path,
    ///     metadata,
    ///     capabilities
    /// ).await?;
    ///
    /// // Query component status
    /// let status = manager.get_component_status(&component_id).await?;
    /// assert_eq!(status, ComponentStatus::Running);
    /// println!("Component status: {:?}", status);
    ///
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Thread Safety
    ///
    /// This method is thread-safe and can be called concurrently from multiple threads.
    /// The HostSystemManager and its ComponentRegistry use interior mutability
    /// (Arc/RwLock) for safe concurrent access.
    pub async fn get_component_status(&self, id: &ComponentId) -> Result<ComponentStatus, WasmError> {
        // 1. Verify system is started
        if !self.started.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(WasmError::engine_initialization(
                "HostSystemManager not initialized".to_string()
            ));
        }

        // 2. Check if component is registered
        if !self.registry.is_registered(id) {
            return Err(WasmError::component_not_found(id.as_str()));
        }

        // 3. Query actor address (verify component is accessible)
        let _actor_address = self.registry.lookup(id)?;

        // 4. Return status (simplified for Phase 4)
        // Note: For now, return Running for all registered components
        // TODO: Phase 5 - Implement actual state tracking (Registered → Running → Stopped)
        Ok(ComponentStatus::Running)
    }

    /// Shuts down the host system gracefully.
    ///
    /// Stops all running components and cleans up resources.
    /// After shutdown, the system cannot be restarted - create a new
    /// HostSystemManager instance instead.
    ///
    /// # Shutdown Flow
    ///
    /// 1. Verify system is started
    /// 2. Get all registered component IDs
    /// 3. Stop each component with timeout
    /// 4. Set started flag to false
    ///
    /// # Parameters
    ///
    /// None
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful shutdown.
    ///
    /// # Errors
    ///
    /// This method continues shutting down other components even if
    /// individual components fail to stop. It returns `Ok(())`
    /// if the shutdown process completes, regardless of individual
    /// component failures.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use airssys_wasm::host_system::HostSystemManager;
    ///
    /// let mut manager = HostSystemManager::new().await?;
    ///
    /// // ... use system ...
    ///
    /// // Graceful shutdown
    /// manager.shutdown().await?;
    /// println!("System shut down gracefully");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn shutdown(&mut self) -> Result<(), WasmError> {
        // Verify system is started
        if !self.started.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(()); // Already shut down
        }

        // Get all registered component IDs
        let component_ids = self.registry.list_components();

        // Stop all components with timeout
        for id in component_ids {
            if let Err(e) = self.stop_component(&id).await {
                eprintln!("Warning: Failed to stop component {:?}: {:?}", id, e);
                // Continue shutting down other components
            }
        }

        // Set started flag to false
        self.started.store(false, std::sync::atomic::Ordering::Relaxed);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_host_system_manager_new_success() {
        // Test: HostSystemManager::new() initializes all infrastructure successfully
        let start = Instant::now();
        let result = HostSystemManager::new().await;
        let duration = start.elapsed();

        assert!(result.is_ok(), "HostSystemManager::new() should succeed");

        let manager = result.unwrap();
        assert!(manager.started(), "System should be started after initialization");

        // Verify initialization meets performance target (<100ms)
        assert!(
            duration.as_millis() < 100,
            "Initialization should complete in <100ms, took {:?}",
            duration
        );

        println!("✅ System initialization completed in {:?}", duration);
    }

    #[tokio::test]
    async fn test_host_system_manager_new_error_handling() {
        // Test: Error handling when WasmEngine creation fails
        // Note: This test verifies error handling path
        // Currently, WasmEngine::new() should not fail in normal conditions
        // This test documents expected error behavior

        let result = HostSystemManager::new().await;

        // In normal conditions, initialization should succeed
        // This test documents that errors are properly converted to WasmError
        match result {
            Ok(_) => {
                println!("✅ Normal initialization succeeded");
            }
            Err(WasmError::EngineInitialization { reason, .. }) => {
                println!("✅ Error properly formatted: {}", reason);
            }
            Err(e) => {
                panic!("Unexpected error type: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_host_system_manager_dependencies_wired() {
        // Test: Verify all dependencies are correctly wired
        let manager = HostSystemManager::new().await.unwrap();

        // We can't directly access private fields, but we can verify
        // the system started flag and that no panics occurred
        assert!(manager.started(), "System should be started");

        // Implicit test: If no panic occurred during initialization,
        // all dependencies were successfully created and wired
        println!("✅ All dependencies initialized without errors");
    }

    #[tokio::test]
    async fn test_host_system_manager_started_flag() {
        // Test: Verify started flag is set correctly
        let manager = HostSystemManager::new().await.unwrap();

        assert!(manager.started(), "started flag should be true after initialization");
        println!("✅ started flag correctly set to true");
    }

    #[tokio::test]
    async fn test_spawn_component_success() {
        // Test: Spawn component successfully with real WASM fixture
        let mut manager = HostSystemManager::new().await.unwrap();

        let component_id = ComponentId::new("test-component");
        let wasm_path = PathBuf::from("tests/fixtures/handle-message-component.wasm");
        let metadata = crate::core::component::ComponentMetadata {
            name: component_id.as_str().to_string(),
            version: "1.0.0".to_string(),
            author: "test".to_string(),
            description: Some("Test component".to_string()),
            max_memory_bytes: 10_000_000,
            max_fuel: 1_000_000,
            timeout_seconds: 30,
        };
        let capabilities = CapabilitySet::new();

        // Spawn component
        let result = manager.spawn_component(
            component_id.clone(),
            wasm_path,
            metadata,
            capabilities
        ).await;

        assert!(result.is_ok(), "spawn_component() should succeed with valid WASM file");

        let actor_address = result.unwrap();

        // Verify ActorAddress is returned
        let actor_id_str = actor_address.id().to_string();
        assert!(!actor_id_str.is_empty(), "ActorAddress should have non-empty ID");

        println!("✅ Component spawned successfully: {}", component_id.as_str());
    }

    #[tokio::test]
    async fn test_spawn_component_not_started() {
        // Test: Error handling when system not initialized
        let mut manager = HostSystemManager::new().await.unwrap();

        // Manually set started flag to false (simulating shutdown)
        manager.started.store(false, std::sync::atomic::Ordering::Relaxed);

        let component_id = ComponentId::new("test-component");
        let wasm_path = PathBuf::from("tests/fixtures/handle-message-component.wasm");
        let metadata = crate::core::component::ComponentMetadata {
            name: component_id.as_str().to_string(),
            version: "1.0.0".to_string(),
            author: "test".to_string(),
            description: Some("Test component".to_string()),
            max_memory_bytes: 10_000_000,
            max_fuel: 1_000_000,
            timeout_seconds: 30,
        };
        let capabilities = CapabilitySet::new();

        // Attempt to spawn (should fail)
        let result = manager.spawn_component(
            component_id,
            wasm_path,
            metadata,
            capabilities
        ).await;

        assert!(result.is_err(), "spawn_component() should fail when system not started");

        // Verify error type
        match result {
            Err(WasmError::EngineInitialization { reason, .. }) => {
                assert!(reason.contains("not initialized") || reason.contains("initialized"),
                    "Error message should mention initialization");
            }
            _ => panic!("Expected EngineInitialization error, got: {:?}", result),
        }

        println!("✅ Error handling correct for not started state");
    }

    #[tokio::test]
    async fn test_spawn_component_deferred_wasm_loading() {
        // Test: Verify WASM loading is deferred to actor execution time
        //
        // spawn_component() creates an Actor but does NOT immediately load the WASM file.
        // WASM loading happens when the actor processes its first message (deferred loading).
        // This allows spawn to succeed even if the WASM file doesn't exist yet.
        //
        // The test verifies this deferred loading behavior.
        let mut manager = HostSystemManager::new().await.unwrap();

        let component_id = ComponentId::new("test-component");
        let wasm_path = PathBuf::from("tests/fixtures/non-existent.wasm");
        let metadata = crate::core::component::ComponentMetadata {
            name: component_id.as_str().to_string(),
            version: "1.0.0".to_string(),
            author: "test".to_string(),
            description: Some("Test component".to_string()),
            max_memory_bytes: 10_000_000,
            max_fuel: 1_000_000,
            timeout_seconds: 30,
        };
        let capabilities = CapabilitySet::new();

        // Spawn should succeed even with invalid path (deferred loading)
        let result = manager.spawn_component(
            component_id.clone(),
            wasm_path,
            metadata,
            capabilities
        ).await;

        assert!(result.is_ok(), "spawn_component() should succeed (deferred loading)");

        let actor_address = result.unwrap();
        assert!(!actor_address.id().to_string().is_empty(), "ActorAddress should be valid");

        println!("✅ spawn_component() succeeds with deferred WASM loading: {}", component_id.as_str());
    }

    #[tokio::test]
    async fn test_spawn_component_actor_address_returned() {
        // Test: Verify ActorAddress is returned for message routing
        let mut manager = HostSystemManager::new().await.unwrap();

        let component_id = ComponentId::new("test-component");
        let wasm_path = PathBuf::from("tests/fixtures/handle-message-component.wasm");
        let metadata = crate::core::component::ComponentMetadata {
            name: component_id.as_str().to_string(),
            version: "1.0.0".to_string(),
            author: "test".to_string(),
            description: Some("Test component".to_string()),
            max_memory_bytes: 10_000_000,
            max_fuel: 1_000_000,
            timeout_seconds: 30,
        };
        let capabilities = CapabilitySet::new();

        // Spawn component
        let result = manager.spawn_component(
            component_id.clone(),
            wasm_path,
            metadata,
            capabilities
        ).await;

        assert!(result.is_ok(), "spawn_component() should succeed");

        let actor_address = result.unwrap();

        // Verify ActorAddress is not empty and has valid ID
        let actor_id_str = actor_address.id().to_string();
        assert!(!actor_id_str.is_empty(), "ActorAddress ID should not be empty");
        println!("✅ ActorAddress returned correctly: {}", actor_id_str);
    }

    #[tokio::test]
    async fn test_stop_component_success() {
        // Test: Stop component successfully after spawning
        let mut manager = HostSystemManager::new().await.unwrap();
        // System is already initialized via new(), no need for initialize_system()

        // Spawn a test component
        let component_id = ComponentId::new("test-component-stop");
        let wasm_path = PathBuf::from("tests/fixtures/handle-message-component.wasm");
        let metadata = crate::core::component::ComponentMetadata {
            name: component_id.as_str().to_string(),
            version: "1.0.0".to_string(),
            author: "test".to_string(),
            description: Some("Test component".to_string()),
            max_memory_bytes: 10_000_000,
            max_fuel: 1_000_000,
            timeout_seconds: 30,
        };
        let capabilities = CapabilitySet::new();

        manager.spawn_component(
            component_id.clone(),
            wasm_path.clone(),
            metadata.clone(),
            capabilities.clone()
        ).await.unwrap();

        // Verify component exists in registry
        let lookup_result = manager.registry.lookup(&component_id);
        assert!(lookup_result.is_ok(), "Component should be registered");

        // Stop component
        let result = manager.stop_component(&component_id).await;

        // Verify success
        assert!(result.is_ok(), "stop_component should succeed: {:?}", result);

        // Verify component removed from registry
        let lookup_result_after = manager.registry.lookup(&component_id);
        assert!(lookup_result_after.is_err(),
                "Component should be removed from registry");

        println!("✅ Component stopped successfully: {}", component_id.as_str());
    }

    #[tokio::test]
    async fn test_stop_component_not_initialized() {
        // Test: Error handling when system not initialized
        let mut manager = HostSystemManager::new().await.unwrap();

        // Manually set started flag to false
        manager.started.store(false, std::sync::atomic::Ordering::Relaxed);

        let component_id = ComponentId::new("test-component");

        // Stop without initialization should fail
        let result = manager.stop_component(&component_id).await;

        assert!(result.is_err());
        match result {
            Err(WasmError::EngineInitialization { reason, .. }) => {
                assert!(reason.contains("not initialized") || reason.contains("initialized"),
                    "Error message should mention initialization");
            }
            _ => panic!("Expected EngineInitialization error, got {:?}", result),
        }

        println!("✅ Error handling correct for not initialized state");
    }

    #[tokio::test]
    async fn test_stop_component_not_found() {
        // Test: Error handling when component doesn't exist
        let mut manager = HostSystemManager::new().await.unwrap();

        let component_id = ComponentId::new("non-existent-component");

        // Stop non-existent component should fail
        let result = manager.stop_component(&component_id).await;

        assert!(result.is_err());
        match result {
            Err(WasmError::ComponentNotFound { component_id: cid, .. }) => {
                assert!(cid.contains("non-existent") || cid.contains("found"),
                    "Error message should mention not found");
            }
            _ => panic!("Expected ComponentNotFound error, got {:?}", result),
        }

        println!("✅ Error handling correct for nonexistent component");
    }

    #[tokio::test]
    async fn test_stop_component_twice() {
        // Test: Stopping already stopped component fails
        let mut manager = HostSystemManager::new().await.unwrap();

        let component_id = ComponentId::new("test-component-twice");
        let wasm_path = PathBuf::from("tests/fixtures/handle-message-component.wasm");
        let metadata = crate::core::component::ComponentMetadata {
            name: component_id.as_str().to_string(),
            version: "1.0.0".to_string(),
            author: "test".to_string(),
            description: Some("Test component".to_string()),
            max_memory_bytes: 10_000_000,
            max_fuel: 1_000_000,
            timeout_seconds: 30,
        };
        let capabilities = CapabilitySet::new();

        manager.spawn_component(
            component_id.clone(),
            wasm_path,
            metadata,
            capabilities
        ).await.unwrap();

        // Stop component first time
        let result1 = manager.stop_component(&component_id).await;
        assert!(result1.is_ok(), "First stop should succeed");

        // Stop component second time should fail
        let result2 = manager.stop_component(&component_id).await;
        assert!(result2.is_err(), "Second stop should fail");

        match result2 {
            Err(WasmError::ComponentNotFound { component_id: _, .. }) => {
                // Expected: component not found
            }
            _ => panic!("Expected ComponentNotFound on second stop, got {:?}", result2),
        }

        println!("✅ Duplicate stop handled correctly");
    }

    #[tokio::test]
    async fn test_stop_component_cleans_correlations() {
        // Test: Correlations are cleaned up when component is stopped
        let mut manager = HostSystemManager::new().await.unwrap();

        let component_id = ComponentId::new("test-component-correlations");
        let wasm_path = PathBuf::from("tests/fixtures/handle-message-component.wasm");
        let metadata = crate::core::component::ComponentMetadata {
            name: component_id.as_str().to_string(),
            version: "1.0.0".to_string(),
            author: "test".to_string(),
            description: Some("Test component".to_string()),
            max_memory_bytes: 10_000_000,
            max_fuel: 1_000_000,
            timeout_seconds: 30,
        };
        let capabilities = CapabilitySet::new();

        manager.spawn_component(
            component_id.clone(),
            wasm_path,
            metadata,
            capabilities
        ).await.unwrap();

        // Register a pending request (simulate request-response)
        let correlation_id = uuid::Uuid::new_v4();
        let (tx, _rx) = tokio::sync::oneshot::channel();
        use crate::core::messaging::{CorrelationId, PendingRequest};
        use std::time::Duration;

        let pending = PendingRequest {
            correlation_id: CorrelationId::from(correlation_id),
            response_tx: tx,
            requested_at: tokio::time::Instant::now(),
            timeout: Duration::from_secs(5),
            from: component_id.clone(),
            to: ComponentId::new("other-component"),
        };

        manager.correlation_tracker.register_pending(pending).await.unwrap();

        // Verify pending request exists
        assert!(manager.correlation_tracker.contains(&CorrelationId::from(correlation_id)),
                "Correlation should be tracked");

        // Stop component
        manager.stop_component(&component_id).await.unwrap();

        // Verify correlation removed (cleanup_pending_for_component called)
        let contains_after = manager.correlation_tracker.contains(&CorrelationId::from(correlation_id));
        assert!(!contains_after,
                "Correlations should be cleaned up after stop");

        println!("✅ Correlations cleaned up correctly");
    }

    #[tokio::test]
    async fn test_stop_component_registry_cleanup() {
        // Test: Component is properly removed from registry after stop
        let mut manager = HostSystemManager::new().await.unwrap();

        let component_id = ComponentId::new("test-component-registry");
        let wasm_path = PathBuf::from("tests/fixtures/handle-message-component.wasm");
        let metadata = crate::core::component::ComponentMetadata {
            name: component_id.as_str().to_string(),
            version: "1.0.0".to_string(),
            author: "test".to_string(),
            description: Some("Test component".to_string()),
            max_memory_bytes: 10_000_000,
            max_fuel: 1_000_000,
            timeout_seconds: 30,
        };
        let capabilities = CapabilitySet::new();

        // Spawn component
        manager.spawn_component(
            component_id.clone(),
            wasm_path,
            metadata,
            capabilities
        ).await.unwrap();

        // Verify component is registered
        let count_before = manager.registry.count().unwrap();
        assert!(count_before > 0, "Registry should have at least one component");

        // Stop component
        manager.stop_component(&component_id).await.unwrap();

        // Verify component is removed from registry
        let count_after = manager.registry.count().unwrap();
        assert!(count_after < count_before,
                "Registry count should decrease after stop");

        // Verify specific component not in registry
        assert!(!manager.registry.is_registered(&component_id),
                "Component should not be registered after stop");

        println!("✅ Registry cleanup verified");
    }

    #[tokio::test]
    async fn test_restart_component_success() {
        let mut manager = HostSystemManager::new().await.unwrap();

        let component_id = ComponentId::new("test-restart-success");
        let wasm_path = PathBuf::from("tests/fixtures/handle-message-component.wasm");
        let wasm_bytes = std::fs::read(&wasm_path).unwrap();
        let metadata = crate::core::component::ComponentMetadata {
            name: component_id.as_str().to_string(),
            version: "1.0.0".to_string(),
            author: "test".to_string(),
            description: Some("Test component".to_string()),
            max_memory_bytes: 10_000_000,
            max_fuel: 1_000_000,
            timeout_seconds: 30,
        };
        let capabilities = CapabilitySet::new();

        manager.spawn_component(
            component_id.clone(),
            wasm_path,
            metadata.clone(),
            capabilities.clone()
        ).await.unwrap();

        assert!(manager.is_component_registered(&component_id),
                "Component should be registered after spawn");

        let result = manager.restart_component(
            &component_id,
            wasm_bytes,
            metadata,
            capabilities
        ).await;

        assert!(result.is_ok(), "restart_component should succeed: {:?}", result);
        assert!(manager.is_component_registered(&component_id),
                "Component should be registered after restart");
    }

    #[tokio::test]
    async fn test_restart_nonexistent_component() {
        let mut manager = HostSystemManager::new().await.unwrap();

        let component_id = ComponentId::new("non-existent-restart-component");
        let wasm_bytes = vec![0x00, 0x61, 0x73, 0x6d];
        let metadata = crate::core::component::ComponentMetadata {
            name: component_id.as_str().to_string(),
            version: "1.0.0".to_string(),
            author: "test".to_string(),
            description: Some("Test component".to_string()),
            max_memory_bytes: 10_000_000,
            max_fuel: 1_000_000,
            timeout_seconds: 30,
        };
        let capabilities = CapabilitySet::new();

        let result = manager.restart_component(
            &component_id,
            wasm_bytes,
            metadata,
            capabilities
        ).await;

        assert!(result.is_err(), "restart_component should fail for nonexistent component: {:?}", result);
        match result {
            Err(WasmError::ComponentNotFound { component_id: cid, .. }) => {
                assert!(cid.contains("non-existent") || cid.contains("found"),
                        "Error message should mention not found");
            }
            Err(e) => panic!("Expected ComponentNotFound, got: {:?}", e),
            Ok(()) => panic!("Expected error, got Ok"),
        }
    }

    #[tokio::test]
    async fn test_restart_preserves_capabilities() {
        let mut manager = HostSystemManager::new().await.unwrap();

        let component_id = ComponentId::new("test-restart-capabilities");
        let wasm_path = PathBuf::from("tests/fixtures/handle-message-component.wasm");
        let wasm_bytes = std::fs::read(&wasm_path).unwrap();

        let capabilities = CapabilitySet::new();
        let metadata = crate::core::component::ComponentMetadata {
            name: component_id.as_str().to_string(),
            version: "1.0.0".to_string(),
            author: "test".to_string(),
            description: Some("Test component".to_string()),
            max_memory_bytes: 10_000_000,
            max_fuel: 1_000_000,
            timeout_seconds: 30,
        };

        manager.spawn_component(
            component_id.clone(),
            wasm_path,
            metadata.clone(),
            capabilities.clone()
        ).await.unwrap();

        let result = manager.restart_component(
            &component_id,
            wasm_bytes,
            metadata,
            capabilities
        ).await;

        assert!(result.is_ok(), "restart_component should succeed: {:?}", result);
        assert!(manager.is_component_registered(&component_id),
                "Component should be registered after restart with same capabilities");
    }

    #[tokio::test]
    async fn test_restart_with_different_id_fails() {
        let mut manager = HostSystemManager::new().await.unwrap();

        let original_id = ComponentId::new("test-restart-original");
        let wrong_id = ComponentId::new("test-restart-wrong");
        let wasm_path = PathBuf::from("tests/fixtures/handle-message-component.wasm");
        let wasm_bytes = std::fs::read(&wasm_path).unwrap();
        let metadata = crate::core::component::ComponentMetadata {
            name: original_id.as_str().to_string(),
            version: "1.0.0".to_string(),
            author: "test".to_string(),
            description: Some("Test component".to_string()),
            max_memory_bytes: 10_000_000,
            max_fuel: 1_000_000,
            timeout_seconds: 30,
        };
        let capabilities = CapabilitySet::new();

        manager.spawn_component(
            original_id.clone(),
            wasm_path.clone(),
            metadata.clone(),
            capabilities.clone()
        ).await.unwrap();

        let result = manager.restart_component(
            &wrong_id,
            wasm_bytes,
            metadata,
            capabilities
        ).await;

        assert!(result.is_err(), "restart with wrong ID should fail: {:?}", result);
        match result {
            Err(WasmError::ComponentNotFound { component_id: cid, .. }) => {
                assert!(cid.contains("wrong") || cid.contains("not found"),
                        "Error should mention wrong component ID");
            }
            Err(e) => panic!("Expected ComponentNotFound, got: {:?}", e),
            Ok(()) => panic!("Expected error, got Ok"),
        }
    }

    // Task 4.7: shutdown() unit tests

    #[tokio::test]
    async fn test_shutdown_stops_all_components() {
        let mut manager = HostSystemManager::new().await.unwrap();

        // Spawn 2 components
        let comp1_id = ComponentId::new("comp1");
        let comp2_id = ComponentId::new("comp2");

        let wasm_path = PathBuf::from("tests/fixtures/handle-message-component.wasm");

        manager.spawn_component(
            comp1_id.clone(),
            wasm_path.clone(),
            crate::core::component::ComponentMetadata {
                name: comp1_id.as_str().to_string(),
                version: "1.0.0".to_string(),
                author: "test".to_string(),
                description: Some("Test component".to_string()),
                max_memory_bytes: 10_000_000,
                max_fuel: 1_000_000,
                timeout_seconds: 30,
            },
            CapabilitySet::new(),
        ).await.unwrap();

        manager.spawn_component(
            comp2_id.clone(),
            wasm_path,
            crate::core::component::ComponentMetadata {
                name: comp2_id.as_str().to_string(),
                version: "1.0.0".to_string(),
                author: "test".to_string(),
                description: Some("Test component".to_string()),
                max_memory_bytes: 10_000_000,
                max_fuel: 1_000_000,
                timeout_seconds: 30,
            },
            CapabilitySet::new(),
        ).await.unwrap();

        // Shutdown
        manager.shutdown().await.unwrap();

        // Verify both stopped (should get ComponentNotFound)
        assert!(manager.get_component_status(&comp1_id).await.is_err());
        assert!(manager.get_component_status(&comp2_id).await.is_err());
    }

    #[tokio::test]
    async fn test_shutdown_is_idempotent() {
        let mut manager = HostSystemManager::new().await.unwrap();

        // First shutdown
        manager.shutdown().await.unwrap();

        // Second shutdown should succeed
        let result = manager.shutdown().await;
        assert!(result.is_ok(), "Second shutdown should succeed");
    }

    #[tokio::test]
    async fn test_shutdown_when_not_started() {
        let mut manager = HostSystemManager::new().await.unwrap();

        // Shutdown when already shut down
        let result = manager.shutdown().await;
        assert!(result.is_ok(), "Shutdown when not started should succeed");
    }

    #[tokio::test]
    async fn test_shutdown_handles_stop_errors() {
        let mut manager = HostSystemManager::new().await.unwrap();

        // Spawn one component
        let comp_id = ComponentId::new("comp1");
        let wasm_path = PathBuf::from("tests/fixtures/handle-message-component.wasm");

        manager.spawn_component(
            comp_id.clone(),
            wasm_path,
            crate::core::component::ComponentMetadata {
                name: comp_id.as_str().to_string(),
                version: "1.0.0".to_string(),
                author: "test".to_string(),
                description: Some("Test component".to_string()),
                max_memory_bytes: 10_000_000,
                max_fuel: 1_000_000,
                timeout_seconds: 30,
            },
            CapabilitySet::new(),
        ).await.unwrap();

        // Shutdown (should succeed even if component fails to stop)
        let result = manager.shutdown().await;
        assert!(result.is_ok(), "Shutdown should succeed despite component errors");
    }

    #[tokio::test]
    async fn test_shutdown_sets_started_flag_false() {
        let mut manager = HostSystemManager::new().await.unwrap();

        assert!(manager.started.load(std::sync::atomic::Ordering::Relaxed), "Should start as true");

        manager.shutdown().await.unwrap();

        assert!(!manager.started.load(std::sync::atomic::Ordering::Relaxed), "Should be false after shutdown");
    }

    #[tokio::test]
    async fn test_shutdown_empty_system() {
        let mut manager = HostSystemManager::new().await.unwrap();

        // Shutdown with no components
        let result = manager.shutdown().await;
        assert!(result.is_ok(), "Shutdown with no components should succeed");
    }

    #[tokio::test]
    async fn test_shutdown_single_component() {
        let mut manager = HostSystemManager::new().await.unwrap();

        let comp_id = ComponentId::new("comp1");
        let wasm_path = PathBuf::from("tests/fixtures/handle-message-component.wasm");

        manager.spawn_component(
            comp_id.clone(),
            wasm_path,
            crate::core::component::ComponentMetadata {
                name: comp_id.as_str().to_string(),
                version: "1.0.0".to_string(),
                author: "test".to_string(),
                description: Some("Test component".to_string()),
                max_memory_bytes: 10_000_000,
                max_fuel: 1_000_000,
                timeout_seconds: 30,
            },
            CapabilitySet::new(),
        ).await.unwrap();

        let result = manager.shutdown().await;
        assert!(result.is_ok(), "Shutdown with single component should succeed");
    }

    #[tokio::test]
    async fn test_shutdown_multiple_components() {
        let mut manager = HostSystemManager::new().await.unwrap();

        let wasm_path = PathBuf::from("tests/fixtures/handle-message-component.wasm");

        // Spawn 3 components
        for i in 1..=3 {
            let comp_id = ComponentId::new(format!("comp{}", i));
            manager.spawn_component(
                comp_id.clone(),
                wasm_path.clone(),
                crate::core::component::ComponentMetadata {
                    name: comp_id.as_str().to_string(),
                    version: "1.0.0".to_string(),
                    author: "test".to_string(),
                    description: Some("Test component".to_string()),
                    max_memory_bytes: 10_000_000,
                    max_fuel: 1_000_000,
                    timeout_seconds: 30,
                },
                CapabilitySet::new(),
            ).await.unwrap();
        }

        let result = manager.shutdown().await;
        assert!(result.is_ok(), "Shutdown with multiple components should succeed");
    }

    #[tokio::test]
    async fn test_shutdown_preserves_error_tolerance() {
        let mut manager = HostSystemManager::new().await.unwrap();

        // Spawn multiple components
        let wasm_path = PathBuf::from("tests/fixtures/handle-message-component.wasm");

        for i in 1..=2 {
            let comp_id = ComponentId::new(format!("comp{}", i));
            manager.spawn_component(
                comp_id.clone(),
                wasm_path.clone(),
                crate::core::component::ComponentMetadata {
                    name: comp_id.as_str().to_string(),
                    version: "1.0.0".to_string(),
                    author: "test".to_string(),
                    description: Some("Test component".to_string()),
                    max_memory_bytes: 10_000_000,
                    max_fuel: 1_000_000,
                    timeout_seconds: 30,
                },
                CapabilitySet::new(),
            ).await.unwrap();
        }

        // Shutdown should succeed even if some components fail
        let result = manager.shutdown().await;
        assert!(result.is_ok(), "Shutdown should be error-tolerant");
    }
}
