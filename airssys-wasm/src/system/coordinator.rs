//! # SystemCoordinator - Composition Root
//!
//! The top-level orchestrator for the airssys-wasm system. Receives pre-built
//! dependencies, wires them together following the Dependency Inversion Principle,
//! and manages the full component lifecycle.
//!
//! # Architecture
//!
//! SystemCoordinator is part of Layer 4 (system/ module). As the composition root,
//! it is the ONLY module that may import from ALL lower layers:
//! - `core/` (Layer 0)
//! - `security/` (Layer 2A)
//! - `runtime/` (Layer 2B)
//! - `component/` (Layer 3A)
//! - `messaging/` (Layer 3B)
//!
//! # S6.2 Compliance
//!
//! Uses full static dispatch via generics `<E, L, V, A, B>`. Zero `dyn` usage
//! in library code, consistent with the pattern used throughout the codebase.
//!
//! # References
//!
//! - ADR-WASM-032: System Module Design
//! - ADR-WASM-023: Module Boundary Enforcement (Layer 4)
//! - KNOWLEDGE-WASM-037: Dependency Inversion Principle

// Layer 1: Standard library imports
use std::fmt;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use airssys_rt::broker::MessageBroker;
use airssys_rt::system::{ActorSystem, SystemConfig};
use airssys_rt::SystemError as RtSystemError;
use chrono::{DateTime, Utc};
use thiserror::Error;

// Layer 3: Internal module imports
use crate::component::registry::{ComponentRegistry, RegistryError};
use crate::component::spawner::{ComponentSpawner, SpawnerError};
use crate::component::wrapper::ComponentActorMessage;
use crate::core::component::id::ComponentId;
use crate::core::messaging::errors::MessagingError as CoreMessagingError;
use crate::core::runtime::traits::{ComponentLoader, RuntimeEngine};
use crate::core::security::traits::{SecurityAuditLogger, SecurityValidator};
use crate::messaging::correlation::CorrelationTrackerImpl;
use crate::messaging::subscriber::ComponentSubscriber;

// ============================================================================
// SystemError
// ============================================================================

/// Errors that can occur during system coordination operations.
#[derive(Debug, Error)]
pub enum SystemError {
    /// System is already running (start() called twice).
    #[error("System already running")]
    AlreadyRunning,

    /// System is not running (operation requires started system).
    #[error("System not running")]
    NotRunning,

    /// System has been shut down and cannot be restarted.
    #[error("System already shut down")]
    AlreadyShutDown,

    /// System shutdown failed.
    #[error("Shutdown failed: {0}")]
    ShutdownFailed(#[source] RtSystemError),

    /// Component operation failed (spawner error).
    #[error("Component error: {0}")]
    ComponentSpawner(#[source] SpawnerError),

    /// Component registry operation failed.
    #[error("Registry error: {0}")]
    ComponentRegistry(#[source] RegistryError),

    /// Messaging operation failed.
    #[error("Messaging error: {0}")]
    Messaging(#[source] CoreMessagingError),

    /// System initialization failed.
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
}

impl From<SpawnerError> for SystemError {
    fn from(err: SpawnerError) -> Self {
        SystemError::ComponentSpawner(err)
    }
}

impl From<RegistryError> for SystemError {
    fn from(err: RegistryError) -> Self {
        SystemError::ComponentRegistry(err)
    }
}

impl From<CoreMessagingError> for SystemError {
    fn from(err: CoreMessagingError) -> Self {
        SystemError::Messaging(err)
    }
}

impl From<RtSystemError> for SystemError {
    fn from(err: RtSystemError) -> Self {
        SystemError::ShutdownFailed(err)
    }
}

// ============================================================================
// SystemCoordinator
// ============================================================================

/// System coordinator - the composition root for airssys-wasm.
///
/// Wires all dependencies together following the Dependency Inversion Principle.
/// As the composition root (Layer 4), this is the ONLY module that imports
/// from ALL layers. Uses full static dispatch (generics) consistent with
/// the S6.2 pattern used throughout the codebase.
///
/// # Architecture
///
/// SystemCoordinator receives pre-built dependencies and:
/// - Creates the ActorSystem (airssys-rt)
/// - Wires ComponentSpawner with the engine, loader, and registry
/// - Manages component lifecycle (load/unload)
/// - Provides accessors for internal components
///
/// # Generic Parameters
///
/// * `E` - WASM runtime engine implementation (e.g., `WasmtimeEngine`)
/// * `L` - Component binary loader implementation (e.g., `FileComponentLoader`)
/// * `V` - Security capability validator (e.g., `CapabilityValidator`)
/// * `A` - Security audit logger (e.g., `ConsoleSecurityAuditLogger`)
/// * `B` - Message broker for the actor system (e.g., `InMemoryMessageBroker`)
///
/// # References
///
/// - ADR-WASM-032: System Module Design
/// - ADR-WASM-023: Module Boundary Enforcement (Layer 4)
/// - KNOWLEDGE-WASM-037: Dependency Inversion Principle
pub struct SystemCoordinator<E, L, V, A, B>
where
    E: RuntimeEngine + 'static,
    L: ComponentLoader + 'static,
    V: SecurityValidator,
    A: SecurityAuditLogger,
    B: MessageBroker<ComponentActorMessage> + Clone + Send + Sync + 'static,
{
    // Injected dependencies (static dispatch via generics, S6.2)
    engine: Arc<E>,
    loader: Arc<L>,
    security_validator: Arc<V>,
    audit_logger: Arc<A>,

    // Internal components (created by coordinator)
    registry: Arc<ComponentRegistry>,
    spawner: ComponentSpawner<E, L>,
    subscriber: Arc<ComponentSubscriber>,
    correlation_tracker: Arc<CorrelationTrackerImpl>,

    // Actor system (from airssys-rt)
    actor_system: ActorSystem<ComponentActorMessage, B>,

    // State
    is_running: bool,
    is_shutdown: bool,
    shutdown_failed_stops: usize,

    // Timestamps
    created_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
}

impl<E, L, V, A, B> SystemCoordinator<E, L, V, A, B>
where
    E: RuntimeEngine + 'static,
    L: ComponentLoader + 'static,
    V: SecurityValidator,
    A: SecurityAuditLogger,
    B: MessageBroker<ComponentActorMessage> + Clone + Send + Sync + 'static,
{
    /// Creates a new SystemCoordinator with injected dependencies.
    ///
    /// This is the primary constructor used by SystemBuilder (WASM-TASK-049).
    /// All dependencies must be pre-constructed by the caller.
    ///
    /// # Arguments
    ///
    /// * `engine` - WASM runtime engine
    /// * `loader` - Component binary loader
    /// * `security_validator` - Capability validator
    /// * `audit_logger` - Security audit logger
    /// * `actor_system_config` - Configuration for the airssys-rt ActorSystem
    /// * `broker` - Message broker for the actor system
    pub fn new(
        engine: Arc<E>,
        loader: Arc<L>,
        security_validator: Arc<V>,
        audit_logger: Arc<A>,
        actor_system_config: SystemConfig,
        broker: B,
    ) -> Self {
        // Create internal components
        let registry = Arc::new(ComponentRegistry::new());
        let subscriber = Arc::new(ComponentSubscriber::new());
        let correlation_tracker = Arc::new(CorrelationTrackerImpl::new());

        // Create actor system (ready immediately, no start() needed)
        let actor_system = ActorSystem::new(actor_system_config, broker);

        // Create spawner with same generic types (static dispatch)
        let spawner = ComponentSpawner::new(
            Arc::clone(&engine),
            Arc::clone(&loader),
            Arc::clone(&registry),
        );

        Self {
            engine,
            loader,
            security_validator,
            audit_logger,
            registry,
            spawner,
            subscriber,
            correlation_tracker,
            actor_system,
            is_running: false,
            is_shutdown: false,
            shutdown_failed_stops: 0,
            created_at: Utc::now(),
            started_at: None,
        }
    }

    // ========================================================================
    // Lifecycle Methods
    // ========================================================================

    /// Start the system coordinator.
    ///
    /// Marks the system as running. The ActorSystem is already operational
    /// after construction (no separate start needed for airssys-rt).
    ///
    /// # Errors
    ///
    /// - `SystemError::AlreadyShutDown` if the system was previously shut down.
    /// - `SystemError::AlreadyRunning` if the system is already started.
    pub fn start(&mut self) -> Result<(), SystemError> {
        if self.is_shutdown {
            return Err(SystemError::AlreadyShutDown);
        }
        if self.is_running {
            return Err(SystemError::AlreadyRunning);
        }

        self.is_running = true;
        self.started_at = Some(Utc::now());
        Ok(())
    }

    /// Gracefully shutdown the system coordinator.
    ///
    /// Stops all registered components, cleans up subscriber mailboxes,
    /// and shuts down the underlying actor system.
    ///
    /// If the system is not running, this is a no-op (returns Ok).
    ///
    /// # Errors
    ///
    /// Returns `SystemError::ShutdownFailed` if the actor system shutdown fails.
    pub async fn shutdown(&mut self) -> Result<(), SystemError> {
        if !self.is_running {
            return Ok(());
        }

        // Step 1: Get list of all registered components
        let component_ids = self.registry.list()?;

        // Step 2: Stop each component (best-effort, count failures)
        let mut failed_stops: usize = 0;
        for id in &component_ids {
            if self.spawner.stop(id).is_err() {
                failed_stops += 1;
            }
            let _ = self.subscriber.unregister_mailbox(id);
        }

        // Step 3: Shutdown the actor system
        self.actor_system.shutdown().await?;

        self.is_running = false;
        self.is_shutdown = true;
        self.shutdown_failed_stops = failed_stops;
        Ok(())
    }

    // ========================================================================
    // Component Management
    // ========================================================================

    /// Load and spawn a component in the actor system.
    ///
    /// Delegates to ComponentSpawner to load the WASM binary, create
    /// a ComponentWrapper actor, spawn it in the ActorSystem, and
    /// register it in the ComponentRegistry.
    ///
    /// # Arguments
    ///
    /// * `id` - The component identifier to load
    ///
    /// # Errors
    ///
    /// - `SystemError::NotRunning` if the system has not been started
    /// - `SystemError::ComponentError` if spawning fails (load, validation, etc.)
    pub async fn load_component(&self, id: ComponentId) -> Result<(), SystemError> {
        if !self.is_running {
            return Err(SystemError::NotRunning);
        }

        self.spawner.spawn(&self.actor_system, id).await?;
        Ok(())
    }

    /// Unload a component from the system.
    ///
    /// Removes the component from the registry and cleans up its
    /// subscriber mailbox registration.
    ///
    /// # Arguments
    ///
    /// * `id` - The component identifier to unload
    ///
    /// # Errors
    ///
    /// - `SystemError::NotRunning` if the system has not been started
    /// - `SystemError::ComponentError` if the component is not found
    pub fn unload_component(&self, id: &ComponentId) -> Result<(), SystemError> {
        if !self.is_running {
            return Err(SystemError::NotRunning);
        }

        // Step 1: Unregister from spawner/registry
        self.spawner.stop(id)?;

        // Step 2: Clean up subscriber mailbox (best-effort)
        let _ = self.subscriber.unregister_mailbox(id);

        Ok(())
    }

    // ========================================================================
    // Accessor Methods
    // ========================================================================

    /// Returns a reference to the runtime engine.
    pub fn engine(&self) -> &Arc<E> {
        &self.engine
    }

    /// Returns a reference to the component loader.
    pub fn loader(&self) -> &Arc<L> {
        &self.loader
    }

    /// Returns a reference to the security validator.
    pub fn security_validator(&self) -> &Arc<V> {
        &self.security_validator
    }

    /// Returns a reference to the audit logger.
    pub fn audit_logger(&self) -> &Arc<A> {
        &self.audit_logger
    }

    /// Returns a reference to the component registry.
    pub fn registry(&self) -> &Arc<ComponentRegistry> {
        &self.registry
    }

    /// Returns a reference to the component subscriber.
    pub fn subscriber(&self) -> &Arc<ComponentSubscriber> {
        &self.subscriber
    }

    /// Returns a reference to the correlation tracker.
    pub fn correlation_tracker(&self) -> &Arc<CorrelationTrackerImpl> {
        &self.correlation_tracker
    }

    /// Returns whether the system is currently running.
    pub fn is_running(&self) -> bool {
        self.is_running
    }

    /// Returns when the coordinator was created.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Returns when the coordinator was started, if it has been started.
    pub fn started_at(&self) -> Option<DateTime<Utc>> {
        self.started_at
    }

    /// Returns the number of component stops that failed during the last shutdown.
    pub fn shutdown_failed_stops(&self) -> usize {
        self.shutdown_failed_stops
    }

    /// Returns the number of currently registered components.
    pub fn component_count(&self) -> Result<usize, SystemError> {
        Ok(self.registry.count()?)
    }
}

// ============================================================================
// Debug Implementation
// ============================================================================

impl<E, L, V, A, B> fmt::Debug for SystemCoordinator<E, L, V, A, B>
where
    E: RuntimeEngine + 'static,
    L: ComponentLoader + 'static,
    V: SecurityValidator,
    A: SecurityAuditLogger,
    B: MessageBroker<ComponentActorMessage> + Clone + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SystemCoordinator")
            .field("is_running", &self.is_running)
            .field("is_shutdown", &self.is_shutdown)
            .field("created_at", &self.created_at)
            .field("started_at", &self.started_at)
            .field("registry", &self.registry)
            .field("subscriber", &self.subscriber)
            .finish_non_exhaustive()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    use airssys_rt::broker::InMemoryMessageBroker;

    use crate::core::component::handle::ComponentHandle;
    use crate::core::component::message::{ComponentMessage, MessagePayload};
    use crate::core::runtime::errors::WasmError;
    use crate::core::security::capability::Capability;
    use crate::core::security::errors::SecurityError;
    use crate::core::security::traits::SecurityEvent;

    // ========================================
    // Mock RuntimeEngine
    // ========================================
    struct MockRuntimeEngine;

    impl RuntimeEngine for MockRuntimeEngine {
        fn load_component(
            &self,
            id: &ComponentId,
            _bytes: &[u8],
        ) -> Result<ComponentHandle, WasmError> {
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
    // Mock ComponentLoader
    // ========================================
    struct MockComponentLoader;

    impl ComponentLoader for MockComponentLoader {
        fn load_bytes(&self, _id: &ComponentId) -> Result<Vec<u8>, WasmError> {
            Ok(vec![0u8; 100])
        }

        fn validate(&self, _bytes: &[u8]) -> Result<(), WasmError> {
            Ok(())
        }
    }

    // ========================================
    // Mock SecurityValidator
    // ========================================
    struct MockSecurityValidator;

    impl SecurityValidator for MockSecurityValidator {
        fn validate_capability(
            &self,
            _component: &ComponentId,
            _capability: &Capability,
        ) -> Result<(), SecurityError> {
            Ok(())
        }

        fn can_send_to(
            &self,
            _sender: &ComponentId,
            _target: &ComponentId,
        ) -> Result<(), SecurityError> {
            Ok(())
        }
    }

    // ========================================
    // Mock SecurityAuditLogger
    // ========================================
    struct MockAuditLogger;

    impl SecurityAuditLogger for MockAuditLogger {
        fn log_event(&self, _event: SecurityEvent) {}
    }

    // ========================================
    // Type alias for test coordinator
    // ========================================
    type TestCoordinator = SystemCoordinator<
        MockRuntimeEngine,
        MockComponentLoader,
        MockSecurityValidator,
        MockAuditLogger,
        InMemoryMessageBroker<ComponentActorMessage>,
    >;

    // ========================================
    // Helper: create coordinator with mocks
    // ========================================
    fn create_test_coordinator() -> TestCoordinator {
        let engine = Arc::new(MockRuntimeEngine);
        let loader = Arc::new(MockComponentLoader);
        let validator = Arc::new(MockSecurityValidator);
        let logger = Arc::new(MockAuditLogger);
        let broker = InMemoryMessageBroker::<ComponentActorMessage>::new();

        SystemCoordinator::new(
            engine,
            loader,
            validator,
            logger,
            SystemConfig::default(),
            broker,
        )
    }

    fn create_test_id(name: &str) -> ComponentId {
        ComponentId::new("test", name, "v1")
    }

    // ========================================
    // 1. Constructor Tests
    // ========================================

    #[tokio::test]
    async fn test_coordinator_creation() {
        let coordinator = create_test_coordinator();
        assert!(!coordinator.is_running());
        coordinator.actor_system.force_shutdown().await;
    }

    #[tokio::test]
    async fn test_coordinator_created_at_is_set() {
        let before = Utc::now();
        let coordinator = create_test_coordinator();
        let after = Utc::now();

        assert!(coordinator.created_at() >= before);
        assert!(coordinator.created_at() <= after);
        coordinator.actor_system.force_shutdown().await;
    }

    #[tokio::test]
    async fn test_coordinator_started_at_is_none_initially() {
        let coordinator = create_test_coordinator();
        assert!(coordinator.started_at().is_none());
        coordinator.actor_system.force_shutdown().await;
    }

    // ========================================
    // 2. Start/Stop Lifecycle Tests
    // ========================================

    #[tokio::test]
    async fn test_start_succeeds() {
        let mut coordinator = create_test_coordinator();
        let result = coordinator.start();
        assert!(result.is_ok());
        assert!(coordinator.is_running());
        coordinator.actor_system.force_shutdown().await;
    }

    #[tokio::test]
    async fn test_start_sets_started_at() {
        let mut coordinator = create_test_coordinator();
        let before = Utc::now();
        coordinator.start().unwrap();
        let after = Utc::now();

        let started_at = coordinator.started_at().unwrap();
        assert!(started_at >= before);
        assert!(started_at <= after);
        coordinator.actor_system.force_shutdown().await;
    }

    #[tokio::test]
    async fn test_start_when_already_running_returns_error() {
        let mut coordinator = create_test_coordinator();
        coordinator.start().unwrap();

        let result = coordinator.start();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already running"));
        coordinator.actor_system.force_shutdown().await;
    }

    #[tokio::test]
    async fn test_shutdown_when_not_running_is_noop() {
        let mut coordinator = create_test_coordinator();
        // Not started, shutdown should be a no-op
        let result = coordinator.shutdown().await;
        assert!(result.is_ok());
        assert!(!coordinator.is_running());
        coordinator.actor_system.force_shutdown().await;
    }

    #[tokio::test]
    async fn test_shutdown_sets_not_running() {
        let mut coordinator = create_test_coordinator();
        coordinator.start().unwrap();
        assert!(coordinator.is_running());

        let result = coordinator.shutdown().await;
        assert!(result.is_ok());
        assert!(!coordinator.is_running());
    }

    #[tokio::test]
    async fn test_start_after_shutdown_returns_error() {
        let mut coordinator = create_test_coordinator();
        coordinator.start().unwrap();
        coordinator.shutdown().await.unwrap();
        assert!(!coordinator.is_running());

        // Cannot restart after shutdown — actor system is no longer usable.
        let result = coordinator.start();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("already shut down"));
    }

    // ========================================
    // 3. Component Load/Unload Tests
    // ========================================

    #[tokio::test]
    async fn test_load_component_when_not_running_returns_error() {
        let coordinator = create_test_coordinator();
        let id = create_test_id("test-comp");

        let result = coordinator.load_component(id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not running"));
    }

    #[tokio::test]
    async fn test_load_component_succeeds() {
        let mut coordinator = create_test_coordinator();
        coordinator.start().unwrap();

        let id = create_test_id("my-component");
        let result = coordinator.load_component(id.clone()).await;
        assert!(result.is_ok());

        // Verify component is in the registry
        assert!(coordinator.registry().contains(&id).unwrap());

        coordinator.actor_system.force_shutdown().await;
    }

    #[tokio::test]
    async fn test_load_multiple_components() {
        let mut coordinator = create_test_coordinator();
        coordinator.start().unwrap();

        let id1 = create_test_id("comp-1");
        let id2 = create_test_id("comp-2");
        let id3 = create_test_id("comp-3");

        coordinator.load_component(id1.clone()).await.unwrap();
        coordinator.load_component(id2.clone()).await.unwrap();
        coordinator.load_component(id3.clone()).await.unwrap();

        assert_eq!(coordinator.registry().count().unwrap(), 3);
        assert!(coordinator.registry().contains(&id1).unwrap());
        assert!(coordinator.registry().contains(&id2).unwrap());
        assert!(coordinator.registry().contains(&id3).unwrap());

        coordinator.actor_system.force_shutdown().await;
    }

    #[tokio::test]
    async fn test_unload_component_succeeds() {
        let mut coordinator = create_test_coordinator();
        coordinator.start().unwrap();

        let id = create_test_id("unload-me");
        coordinator.load_component(id.clone()).await.unwrap();
        assert!(coordinator.registry().contains(&id).unwrap());

        let result = coordinator.unload_component(&id);
        assert!(result.is_ok());
        assert!(!coordinator.registry().contains(&id).unwrap());

        coordinator.actor_system.force_shutdown().await;
    }

    #[tokio::test]
    async fn test_unload_component_when_not_running_returns_error() {
        let coordinator = create_test_coordinator();
        let id = create_test_id("test-comp");

        let result = coordinator.unload_component(&id);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not running"));
        coordinator.actor_system.force_shutdown().await;
    }

    #[tokio::test]
    async fn test_unload_nonexistent_component_returns_error() {
        let mut coordinator = create_test_coordinator();
        coordinator.start().unwrap();

        let id = create_test_id("nonexistent");
        let result = coordinator.unload_component(&id);
        assert!(result.is_err());
        // SpawnerError::NotSpawned wraps into SystemError::ComponentSpawner
        assert!(result.unwrap_err().to_string().contains("Component error"));

        coordinator.actor_system.force_shutdown().await;
    }

    // ========================================
    // 4. Accessor Tests
    // ========================================

    #[tokio::test]
    async fn test_accessor_registry() {
        let coordinator = create_test_coordinator();
        let registry = coordinator.registry();
        assert_eq!(registry.count().unwrap(), 0);
        coordinator.actor_system.force_shutdown().await;
    }

    #[tokio::test]
    async fn test_accessor_subscriber() {
        let coordinator = create_test_coordinator();
        let subscriber = coordinator.subscriber();
        assert_eq!(subscriber.count().unwrap(), 0);
        coordinator.actor_system.force_shutdown().await;
    }

    #[tokio::test]
    async fn test_accessor_correlation_tracker() {
        let coordinator = create_test_coordinator();
        let _tracker = coordinator.correlation_tracker();
        // CorrelationTrackerImpl exists and is accessible
        coordinator.actor_system.force_shutdown().await;
    }

    #[tokio::test]
    async fn test_accessor_is_running() {
        let mut coordinator = create_test_coordinator();
        assert!(!coordinator.is_running());

        coordinator.start().unwrap();
        assert!(coordinator.is_running());
        coordinator.actor_system.force_shutdown().await;
    }

    #[tokio::test]
    async fn test_component_count_empty() {
        let coordinator = create_test_coordinator();
        assert_eq!(coordinator.component_count().unwrap(), 0);
        coordinator.actor_system.force_shutdown().await;
    }

    #[tokio::test]
    async fn test_component_count_after_load() {
        let mut coordinator = create_test_coordinator();
        coordinator.start().unwrap();

        coordinator
            .load_component(create_test_id("c1"))
            .await
            .unwrap();
        assert_eq!(coordinator.component_count().unwrap(), 1);

        coordinator
            .load_component(create_test_id("c2"))
            .await
            .unwrap();
        assert_eq!(coordinator.component_count().unwrap(), 2);

        coordinator.actor_system.force_shutdown().await;
    }

    #[tokio::test]
    async fn test_shutdown_failed_stops_initially_zero() {
        let coordinator = create_test_coordinator();
        assert_eq!(coordinator.shutdown_failed_stops(), 0);
        coordinator.actor_system.force_shutdown().await;
    }

    // ========================================
    // 5. Error Type Tests
    // ========================================

    #[test]
    fn test_system_error_display_already_running() {
        let err = SystemError::AlreadyRunning;
        assert_eq!(err.to_string(), "System already running");
    }

    #[test]
    fn test_system_error_display_not_running() {
        let err = SystemError::NotRunning;
        assert_eq!(err.to_string(), "System not running");
    }

    #[test]
    fn test_system_error_display_already_shut_down() {
        let err = SystemError::AlreadyShutDown;
        assert_eq!(err.to_string(), "System already shut down");
    }

    #[test]
    fn test_system_error_display_shutdown_failed() {
        let rt_err = RtSystemError::ShuttingDown;
        let err = SystemError::ShutdownFailed(rt_err);
        assert!(err.to_string().contains("Shutdown failed"));
    }

    #[test]
    fn test_system_error_display_component_spawner() {
        let spawner_err = SpawnerError::NotSpawned("test/comp/v1".to_string());
        let err = SystemError::ComponentSpawner(spawner_err);
        assert!(err.to_string().contains("Component error"));
    }

    #[test]
    fn test_system_error_display_component_registry() {
        let reg_err = RegistryError::LockPoisoned("test".to_string());
        let err = SystemError::ComponentRegistry(reg_err);
        assert!(err.to_string().contains("Registry error"));
    }

    #[test]
    fn test_system_error_from_spawner_error() {
        let spawner_err = SpawnerError::NotSpawned("test/comp/v1".to_string());
        let system_err: SystemError = spawner_err.into();
        assert!(system_err.to_string().contains("Component error"));
        // Source error is preserved (not stringified)
        assert!(std::error::Error::source(&system_err).is_some());
    }

    #[test]
    fn test_system_error_from_registry_error() {
        let reg_err = RegistryError::LockPoisoned("test".to_string());
        let system_err: SystemError = reg_err.into();
        assert!(system_err.to_string().contains("Registry error"));
        assert!(std::error::Error::source(&system_err).is_some());
    }

    #[test]
    fn test_system_error_from_messaging_error() {
        let messaging_err = CoreMessagingError::DeliveryFailed("connection lost".to_string());
        let system_err: SystemError = messaging_err.into();
        assert!(system_err.to_string().contains("Messaging error"));
        assert!(std::error::Error::source(&system_err).is_some());
    }

    #[test]
    fn test_system_error_from_rt_system_error() {
        let rt_err = RtSystemError::ShuttingDown;
        let system_err: SystemError = rt_err.into();
        assert!(system_err.to_string().contains("Shutdown failed"));
        assert!(std::error::Error::source(&system_err).is_some());
    }

    // ========================================
    // 6. Debug Trait Test
    // ========================================

    #[tokio::test]
    async fn test_coordinator_debug_format() {
        let coordinator = create_test_coordinator();
        let debug_str = format!("{:?}", coordinator);
        assert!(debug_str.contains("SystemCoordinator"));
        assert!(debug_str.contains("is_running"));
        assert!(debug_str.contains("is_shutdown"));
        assert!(debug_str.contains("created_at"));
        // finish_non_exhaustive() appends ".." to indicate omitted fields
        assert!(debug_str.contains(".."));
        coordinator.actor_system.force_shutdown().await;
    }

    // ========================================
    // 7. Thread Safety Test
    // ========================================

    #[test]
    fn test_coordinator_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<TestCoordinator>();
    }
}
