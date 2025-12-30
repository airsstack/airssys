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
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use airssys_rt::broker::InMemoryMessageBroker;

// Layer 3: Internal module imports
use crate::core::WasmError;
use crate::core::component_message::ComponentMessage;
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
}
