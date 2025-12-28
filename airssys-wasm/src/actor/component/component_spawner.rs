//! Component spawner for creating ComponentActor instances via ActorSystem.
//!
//! This module provides `ComponentSpawner`, which manages the spawning of WASM component
//! actors using the airssys-rt ActorSystem. ComponentSpawner ensures components are
//! properly initialized, registered, and integrated with the actor system.
//!
//! # Architecture
//!
//! ```text
//! ComponentSpawner
//!     ↓
//! ActorSystem::spawn() → ComponentActor (Actor trait)
//!     ↓                       ↓
//! Mailbox ← Messages    WASM Runtime (Child trait)
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use airssys_wasm::actor::ComponentSpawner;
//! use airssys_wasm::core::{ComponentId, SecurityConfig};
//! use airssys_rt::system::{ActorSystem, SystemConfig};
//! use airssys_rt::broker::InMemoryMessageBroker;
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create ActorSystem for ComponentMessage
//!     let broker = InMemoryMessageBroker::new();
//!     let actor_system = ActorSystem::new(SystemConfig::default(), broker);
//!     
//!     // Create ComponentSpawner
//!     let spawner = ComponentSpawner::new(actor_system);
//!     
//!     // Spawn a component
//!     let component_id = ComponentId::new("my-component");
//!     let wasm_path = PathBuf::from("/path/to/component.wasm");
//!     let security_config = SecurityConfig::default();
//!     
//!     let actor_ref = spawner
//!         .spawn_component(component_id, wasm_path, security_config)
//!         .await?;
//!     
//!     // Use actor_ref to send messages
//!     Ok(())
//! }
//! ```
//!
//! # Performance
//!
//! Target: <5ms spawn time (including WASM load)
//!
//! # References
//!
//! - **WASM-TASK-004 Phase 2 Task 2.1**: ComponentSpawner Implementation
//! - **ADR-WASM-006**: Actor-based Component Isolation

// Layer 1: Standard library imports
use std::path::PathBuf;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use tokio::sync::RwLock;

// Layer 3: Internal module imports
use super::component_actor::{ComponentActor, ComponentMessage};
use super::component_registry::ComponentRegistry;
use super::component_supervisor::ComponentSupervisor;
use crate::actor::message::MessageRouter;
use crate::actor::supervisor::SupervisorConfig;
use crate::actor::supervisor::SupervisorNodeWrapper;
use crate::core::{CapabilitySet, ComponentId, ComponentMetadata, WasmError};
use airssys_rt::broker::MessageBroker;
use airssys_rt::system::ActorSystem;
use airssys_rt::util::ActorAddress;

/// Spawner for WASM component actors.
///
/// ComponentSpawner manages the lifecycle of spawning ComponentActor instances
/// through the airssys-rt ActorSystem, ensuring proper initialization and
/// registration. Supports both unsupervised and supervised spawning.
///
/// # Type Parameters
///
/// * `B` - The message broker implementation (injected via ActorSystem)
///
/// # Examples
///
/// ```rust,ignore
/// let spawner = ComponentSpawner::new(actor_system, registry, broker);
/// let actor_ref = spawner.spawn_component(
///     ComponentId::new("worker"),
///     PathBuf::from("./worker.wasm"),
///     metadata,
///     capabilities
/// ).await?;
/// ```
pub struct ComponentSpawner<B: MessageBroker<ComponentMessage>> {
    actor_system: ActorSystem<ComponentMessage, B>,
    registry: ComponentRegistry,
    broker: B,
    supervisor: Option<Arc<RwLock<ComponentSupervisor>>>,
}

// Manual Debug implementation since ActorSystem doesn't implement Debug
impl<B: MessageBroker<ComponentMessage>> std::fmt::Debug for ComponentSpawner<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentSpawner")
            .field("actor_system", &"<ActorSystem>")
            .finish()
    }
}

impl<B: MessageBroker<ComponentMessage> + Clone + Send + Sync + 'static> ComponentSpawner<B> {
    /// Create new spawner with ActorSystem, ComponentRegistry, and MessageBroker.
    ///
    /// # Arguments
    ///
    /// * `actor_system` - The ActorSystem instance for spawning ComponentActors
    /// * `registry` - ComponentRegistry for tracking spawned components
    /// * `broker` - MessageBroker for message routing (cloned from ActorSystem)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_rt::system::{ActorSystem, SystemConfig};
    /// use airssys_rt::broker::InMemoryMessageBroker;
    /// use airssys_wasm::actor::ComponentRegistry;
    ///
    /// let broker = InMemoryMessageBroker::new();
    /// let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
    /// let registry = ComponentRegistry::new();
    /// let spawner = ComponentSpawner::new(actor_system, registry, broker);
    /// ```
    pub fn new(
        actor_system: ActorSystem<ComponentMessage, B>,
        registry: ComponentRegistry,
        broker: B,
    ) -> Self {
        Self {
            actor_system,
            registry,
            broker,
            supervisor: None,
        }
    }

    /// Create new spawner with supervision support.
    ///
    /// This enables supervised component spawning with automatic restart.
    ///
    /// # Arguments
    ///
    /// * `actor_system` - The ActorSystem instance for spawning ComponentActors
    /// * `registry` - ComponentRegistry for tracking spawned components
    /// * `broker` - MessageBroker for message routing
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let spawner = ComponentSpawner::with_supervision(actor_system, registry, broker);
    /// let component_id = spawner.spawn_supervised_component(
    ///     component_id,
    ///     wasm_path,
    ///     metadata,
    ///     capabilities,
    ///     SupervisorConfig::permanent()
    /// ).await?;
    /// ```
    pub fn with_supervision(
        actor_system: ActorSystem<ComponentMessage, B>,
        registry: ComponentRegistry,
        broker: B,
    ) -> Self {
        // Create SupervisorNodeBridge
        let bridge = Arc::new(RwLock::new(SupervisorNodeWrapper::new()));

        // Create ComponentSupervisor with bridge
        let supervisor = Arc::new(RwLock::new(ComponentSupervisor::with_bridge(bridge)));

        Self {
            actor_system,
            registry,
            broker,
            supervisor: Some(supervisor),
        }
    }

    /// Get reference to ComponentSupervisor (if supervision enabled).
    ///
    /// # Returns
    ///
    /// - `Some(Arc<RwLock<ComponentSupervisor>>)` if supervision enabled
    /// - `None` if supervision not enabled
    pub fn supervisor(&self) -> Option<Arc<RwLock<ComponentSupervisor>>> {
        self.supervisor.clone()
    }

    /// Get reference to underlying MessageBroker.
    ///
    /// Used for creating MessageRouter instances.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let broker = spawner.broker();
    /// // Use broker for manual message routing or router creation
    /// ```
    pub fn broker(&self) -> B {
        self.broker.clone()
    }

    /// Create MessageRouter with this spawner's registry and broker.
    ///
    /// Convenience method for creating router with correct dependencies.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let spawner = ComponentSpawner::new(actor_system, registry, broker);
    /// let router = spawner.create_router();
    /// ```
    pub fn create_router(&self) -> MessageRouter<B> {
        MessageRouter::new(
            self.registry.clone(),
            std::sync::Arc::new(self.broker.clone()),
        )
    }

    /// Spawn a component actor instance.
    ///
    /// Creates and spawns a ComponentActor via ActorSystem (NOT tokio::spawn),
    /// ensuring proper integration with the actor system's message routing
    /// and lifecycle management.
    ///
    /// # Arguments
    ///
    /// * `component_id` - Unique identifier for the component
    /// * `wasm_path` - Path to WASM component file
    /// * `metadata` - Component metadata (resource limits, capabilities, etc.)
    /// * `capabilities` - Security capabilities granted to component
    ///
    /// # Returns
    ///
    /// Returns `ActorAddress` for sending messages to the spawned component.
    ///
    /// # Errors
    ///
    /// * `WasmError::ActorError` - If actor spawning fails
    /// * `WasmError::InvalidConfiguration` - If metadata is invalid
    ///
    /// # Performance
    ///
    /// Target: <5ms average spawn time (including WASM load via Child::start)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let actor_ref = spawner.spawn_component(
    ///     ComponentId::new("image-processor"),
    ///     PathBuf::from("./image_processor.wasm"),
    ///     metadata,
    ///     capabilities
    /// ).await?;
    /// ```
    pub async fn spawn_component(
        &self,
        component_id: ComponentId,
        _wasm_path: PathBuf,
        metadata: ComponentMetadata,
        capabilities: CapabilitySet,
    ) -> Result<ActorAddress, WasmError> {
        // 1. Create ComponentActor instance
        let mut actor = ComponentActor::new(component_id.clone(), metadata, capabilities, ());

        // 2. Inject MessageBroker bridge
        // Create broker wrapper and inject into actor
        let broker_wrapper = Arc::new(crate::actor::message::MessageBrokerWrapper::new(
            self.broker.clone(),
        ));
        actor.set_broker(broker_wrapper as Arc<dyn crate::actor::message::MessageBrokerBridge>);

        // 3. Spawn via ActorSystem (NOT tokio::spawn)
        // Note: WASM loading happens later via Child::start() when supervised
        let actor_ref = self
            .actor_system
            .spawn()
            .with_name(component_id.as_str())
            .spawn(actor)
            .await
            .map_err(|e| {
                WasmError::actor_error(format!(
                    "Failed to spawn component {}: {}",
                    component_id.as_str(),
                    e
                ))
            })?;

        // 4. Register component in registry for routing
        self.registry
            .register(component_id.clone(), actor_ref.clone())
            .map_err(|e| {
                WasmError::internal(format!(
                    "Failed to register component {} in registry: {}",
                    component_id.as_str(),
                    e
                ))
            })?;

        // 5. Return ActorAddress for message routing
        Ok(actor_ref)
    }

    /// Spawn a supervised component with automatic restart.
    ///
    /// This method creates a ComponentActor and registers it with ComponentSupervisor
    /// for automatic restart based on the provided supervision configuration.
    ///
    /// # Arguments
    ///
    /// * `component_id` - Unique identifier for the component
    /// * `wasm_path` - Path to WASM component file
    /// * `metadata` - Component metadata (resource limits, capabilities, etc.)
    /// * `capabilities` - Security capabilities granted to component
    /// * `supervision_config` - Supervision configuration for restart behavior
    ///
    /// # Returns
    ///
    /// Returns `ComponentId` for tracking the supervised component.
    ///
    /// # Errors
    ///
    /// * `WasmError::Internal` - If supervision not enabled
    /// * `WasmError::ActorError` - If actor spawning fails
    /// * `WasmError::InvalidConfiguration` - If metadata is invalid
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let component_id = spawner.spawn_supervised_component(
    ///     ComponentId::new("image-processor"),
    ///     PathBuf::from("./image_processor.wasm"),
    ///     metadata,
    ///     capabilities,
    ///     SupervisorConfig::permanent()
    /// ).await?;
    /// ```
    pub async fn spawn_supervised_component(
        &self,
        component_id: ComponentId,
        _wasm_path: PathBuf,
        metadata: ComponentMetadata,
        capabilities: CapabilitySet,
        supervision_config: SupervisorConfig,
    ) -> Result<ComponentId, WasmError> {
        // Verify supervision is enabled
        let supervisor = self.supervisor.as_ref().ok_or_else(|| {
            WasmError::internal("Supervision not enabled - use with_supervision()")
        })?;

        // 1. Create ComponentActor instance
        let mut actor = ComponentActor::new(component_id.clone(), metadata, capabilities, ());

        // 2. Inject MessageBroker bridge
        let broker_wrapper = Arc::new(crate::actor::message::MessageBrokerWrapper::new(
            self.broker.clone(),
        ));
        actor.set_broker(broker_wrapper as Arc<dyn crate::actor::message::MessageBrokerBridge>);

        // 3. Register with ComponentSupervisor (which registers with SupervisorNode)
        {
            let mut supervisor_guard = supervisor.write().await;
            supervisor_guard
                .supervise_with_actor(component_id.clone(), actor, supervision_config)
                .await?;
        }

        // 4. Start the component via supervisor
        {
            let mut supervisor_guard = supervisor.write().await;
            supervisor_guard.start_component(&component_id).await?;
        }

        // 4. Return ComponentId for tracking
        Ok(component_id)
    }
}

#[allow(clippy::expect_used, clippy::unwrap_used, clippy::unwrap_err_used, clippy::expect_err_used, clippy::panic, clippy::unwrap_on_result, clippy::indexing_slicing, clippy::too_many_arguments, clippy::type_complexity, reason = "test code")]
#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "expect is acceptable in test code for clear error messages"
)]
#[expect(
    clippy::panic,
    reason = "panic is acceptable in test code for assertion failures"
)]
#[expect(clippy::unwrap_used, reason = "unwrap is acceptable in test code")]
mod tests {
    use super::*;
    use crate::core::ResourceLimits;
    use airssys_rt::broker::InMemoryMessageBroker;
    use airssys_rt::system::SystemConfig;
    use airssys_rt::util::ActorAddress;

    fn create_test_metadata() -> ComponentMetadata {
        ComponentMetadata {
            name: "test-component".to_string(),
            version: "1.0.0".to_string(),
            author: "Test".to_string(),
            description: None,
            max_memory_bytes: 64 * 1024 * 1024,
            max_fuel: 1_000_000,
            timeout_seconds: 5,
        }
    }

    #[tokio::test]
    async fn test_component_spawner_creation() {
        let broker = InMemoryMessageBroker::new();
        let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
        let registry = ComponentRegistry::new();
        let _spawner = ComponentSpawner::new(actor_system, registry, broker);
        // Test passes if ComponentSpawner can be created
    }

    #[tokio::test]
    async fn test_spawn_component_via_actor_system() {
        // 1. Create ActorSystem
        let broker = InMemoryMessageBroker::new();
        let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());

        // 2. Create ComponentSpawner with registry
        let registry = ComponentRegistry::new();
        let spawner = ComponentSpawner::new(actor_system, registry.clone(), broker);

        // 3. Spawn component
        let component_id = ComponentId::new("test-component");
        let wasm_path = PathBuf::from("./test.wasm");
        let metadata = create_test_metadata();
        let capabilities = CapabilitySet::new();

        let actor_ref = spawner
            .spawn_component(component_id.clone(), wasm_path, metadata, capabilities)
            .await;

        assert!(
            actor_ref.is_ok(),
            "Failed to spawn component: {:?}",
            actor_ref.err()
        );
        let actor_ref = actor_ref.expect("spawn_component returned Ok but unwrap failed");

        // 4. Verify ActorAddress returned
        match actor_ref {
            ActorAddress::Named { name, .. } => {
                assert_eq!(name, component_id.as_str());
            }
            other => panic!("Expected named ActorAddress, got: {:?}", other),
        }

        // 5. Verify component registered in registry
        let lookup_result = registry.lookup(&component_id);
        assert!(
            lookup_result.is_ok(),
            "Component should be registered in registry"
        );
    }

    #[tokio::test]
    async fn test_spawn_multiple_components() {
        let broker = InMemoryMessageBroker::new();
        let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
        let registry = ComponentRegistry::new();
        let spawner = ComponentSpawner::new(actor_system, registry.clone(), broker);

        // Spawn 3 components
        for i in 0..3 {
            let component_id = ComponentId::new(format!("test-component-{}", i));
            let wasm_path = PathBuf::from(format!("./test{}.wasm", i));
            let metadata = create_test_metadata();
            let capabilities = CapabilitySet::new();

            let result = spawner
                .spawn_component(component_id.clone(), wasm_path, metadata, capabilities)
                .await;

            assert!(
                result.is_ok(),
                "Failed to spawn component {}: {:?}",
                i,
                result.err()
            );

            // Verify registration
            assert!(
                registry.lookup(&component_id).is_ok(),
                "Component {} should be registered",
                i
            );
        }

        // Verify all components registered
        assert_eq!(registry.count().unwrap(), 3);
    }

    #[tokio::test]
    async fn test_create_router() {
        let broker = InMemoryMessageBroker::new();
        let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
        let registry = ComponentRegistry::new();
        let spawner = ComponentSpawner::new(actor_system, registry.clone(), broker);

        // Create router via spawner
        let router = spawner.create_router();

        // Verify router uses same registry
        assert_eq!(router.component_count().unwrap(), 0);

        // Register a component
        let component_id = ComponentId::new("test");
        let actor_addr = ActorAddress::named("test");
        registry.register(component_id.clone(), actor_addr).unwrap();

        // Verify router sees the component
        assert_eq!(router.component_count().unwrap(), 1);
        assert!(router.component_exists(&component_id));
    }

    // ========================================================================
    // STEP 3.2.4: Supervised Spawning Tests
    // ========================================================================

    #[tokio::test]
    async fn test_component_spawner_with_supervision() {
        let broker = InMemoryMessageBroker::new();
        let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
        let registry = ComponentRegistry::new();
        let spawner = ComponentSpawner::with_supervision(actor_system, registry, broker);

        assert!(spawner.supervisor().is_some());
    }

    #[tokio::test]
    async fn test_component_spawner_without_supervision() {
        let broker = InMemoryMessageBroker::new();
        let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
        let registry = ComponentRegistry::new();
        let spawner = ComponentSpawner::new(actor_system, registry, broker);

        assert!(spawner.supervisor().is_none());
    }

    #[tokio::test]
    async fn test_spawn_supervised_component_without_supervision_fails() {
        let broker = InMemoryMessageBroker::new();
        let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
        let registry = ComponentRegistry::new();
        let spawner = ComponentSpawner::new(actor_system, registry, broker);

        let component_id = ComponentId::new("test-component");
        let wasm_path = PathBuf::from("./test.wasm");
        let metadata = create_test_metadata();
        let capabilities = CapabilitySet::new();
        let supervision_config = SupervisorConfig::permanent();

        let result = spawner
            .spawn_supervised_component(
                component_id,
                wasm_path,
                metadata,
                capabilities,
                supervision_config,
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_supervisor_reference() {
        let broker = InMemoryMessageBroker::new();
        let actor_system = ActorSystem::new(SystemConfig::default(), broker.clone());
        let registry = ComponentRegistry::new();
        let spawner = ComponentSpawner::with_supervision(actor_system, registry, broker);

        let supervisor = spawner.supervisor();
        assert!(supervisor.is_some());

        // Verify we can access the supervisor
        if let Some(sup) = supervisor {
            let guard = sup.read().await;
            let stats = guard.get_statistics();
            assert_eq!(stats.total_supervised, 0);
        }
    }
}
