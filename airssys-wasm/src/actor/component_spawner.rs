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

// Layer 2: Third-party crate imports
// (none)

// Layer 3: Internal module imports
use super::component_actor::{ComponentActor, ComponentMessage};
use super::component_registry::ComponentRegistry;
use super::message_router::MessageRouter;
use crate::core::{ComponentId, ComponentMetadata, CapabilitySet, WasmError};
use airssys_rt::broker::MessageBroker;
use airssys_rt::system::ActorSystem;
use airssys_rt::util::ActorAddress;

/// Spawner for WASM component actors.
///
/// ComponentSpawner manages the lifecycle of spawning ComponentActor instances
/// through the airssys-rt ActorSystem, ensuring proper initialization and
/// registration.
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
    pub fn new(actor_system: ActorSystem<ComponentMessage, B>, registry: ComponentRegistry, broker: B) -> Self {
        Self { actor_system, registry, broker }
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
        let actor = ComponentActor::new(component_id.clone(), metadata, capabilities);

        // 2. Spawn via ActorSystem (NOT tokio::spawn)
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
                    component_id.as_str(), e
                ))
            })?;

        // 3. Register component in registry for routing
        self.registry
            .register(component_id.clone(), actor_ref.clone())
            .map_err(|e| {
                WasmError::internal(format!(
                    "Failed to register component {} in registry: {}",
                    component_id.as_str(), e
                ))
            })?;

        // 4. Return ActorAddress for message routing
        Ok(actor_ref)
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "expect is acceptable in test code for clear error messages")]
#[expect(clippy::panic, reason = "panic is acceptable in test code for assertion failures")]
#[expect(clippy::unwrap_used, reason = "unwrap is acceptable in test code")]
mod tests {
    use super::*;
    use airssys_rt::broker::InMemoryMessageBroker;
    use airssys_rt::system::SystemConfig;
    use airssys_rt::util::ActorAddress;
    use crate::core::ResourceLimits;

    fn create_test_metadata() -> ComponentMetadata {
        ComponentMetadata {
            name: "test-component".to_string(),
            version: "1.0.0".to_string(),
            author: "Test".to_string(),
            description: None,
            required_capabilities: vec![],
            resource_limits: ResourceLimits {
                max_memory_bytes: 64 * 1024 * 1024,
                max_fuel: 1_000_000,
                max_execution_ms: 5000,
                max_storage_bytes: 10 * 1024 * 1024,
            },
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
        
        assert!(actor_ref.is_ok(), "Failed to spawn component: {:?}", actor_ref.err());
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
        assert!(lookup_result.is_ok(), "Component should be registered in registry");
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

            assert!(result.is_ok(), "Failed to spawn component {}: {:?}", i, result.err());
            
            // Verify registration
            assert!(registry.lookup(&component_id).is_ok(), "Component {} should be registered", i);
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
}
