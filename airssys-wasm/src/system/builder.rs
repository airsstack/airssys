//! # SystemBuilder - Builder Pattern for SystemCoordinator
//!
//! Provides an ergonomic builder API for constructing a fully-configured
//! [`SystemCoordinator`]. All dependencies are injected via the constructor,
//! with optional configuration available through setter methods.
//!
//! # Architecture
//!
//! SystemBuilder is part of Layer 4 (system/ module). As the composition root's
//! entry point, it is allowed to import from ALL lower layers.
//!
//! # S6.2 Compliance
//!
//! Uses full static dispatch via generics `<E, L, V, A, B>`. Zero `dyn` usage,
//! consistent with the pattern used throughout the codebase:
//! - `ComponentSpawner<E, L>`
//! - `ComponentWrapper<E>`
//! - `SystemCoordinator<E, L, V, A, B>`
//!
//! # References
//!
//! - ADR-WASM-032: System Module Design (evolved: generics instead of dyn)
//! - ADR-WASM-023: Module Boundary Enforcement (Layer 4)
//! - KNOWLEDGE-WASM-037: Dependency Inversion Principle
//! - PROJECTS_STANDARD.md S6.2: Avoid dyn Patterns

// Layer 1: Standard library imports
use std::fmt;
use std::sync::Arc;

// Layer 2: Third-party crate imports
use airssys_rt::broker::MessageBroker;
use airssys_rt::system::SystemConfig;

// Layer 3: Internal module imports
use super::coordinator::SystemCoordinator;
use crate::component::wrapper::ComponentActorMessage;
use crate::core::runtime::traits::{ComponentLoader, RuntimeEngine};
use crate::core::security::traits::{SecurityAuditLogger, SecurityValidator};

/// Builder for constructing a fully-configured [`SystemCoordinator`].
///
/// Uses the builder pattern with method chaining for ergonomic configuration.
/// All dependencies must be provided at construction time via `new()`.
/// Optional configuration (e.g., actor system config) can be set via
/// setter methods before calling `build()`.
///
/// # Generic Parameters
///
/// * `E` - WASM runtime engine implementation (e.g., `WasmtimeEngine`)
/// * `L` - Component binary loader implementation (e.g., `FileComponentLoader`)
/// * `V` - Security capability validator (e.g., `CapabilityValidator`)
/// * `A` - Security audit logger (e.g., `ConsoleSecurityAuditLogger`)
/// * `B` - Message broker for the actor system (e.g., `InMemoryMessageBroker`)
///
/// # Example
///
/// ```rust,ignore
/// let coordinator = SystemBuilder::new(
///     Arc::new(engine),
///     Arc::new(loader),
///     Arc::new(validator),
///     Arc::new(logger),
///     broker,
/// )
/// .with_actor_system_config(custom_config)
/// .build();
/// ```
///
/// # S6.2 Compliance
///
/// Uses full static dispatch via generics. Zero `dyn` usage, consistent with
/// the pattern used throughout the codebase.
///
/// # References
///
/// - ADR-WASM-032: System Module Design
/// - ADR-WASM-023: Module Boundary Enforcement (Layer 4)
/// - KNOWLEDGE-WASM-037: Dependency Inversion Principle
pub struct SystemBuilder<E, L, V, A, B>
where
    E: RuntimeEngine + 'static,
    L: ComponentLoader + 'static,
    V: SecurityValidator,
    A: SecurityAuditLogger,
    B: MessageBroker<ComponentActorMessage> + Clone + Send + Sync + 'static,
{
    // Required dependencies (static dispatch via generics, S6.2)
    engine: Arc<E>,
    loader: Arc<L>,
    security_validator: Arc<V>,
    audit_logger: Arc<A>,
    broker: B,

    // Optional configuration
    actor_system_config: SystemConfig,
}

impl<E, L, V, A, B> SystemBuilder<E, L, V, A, B>
where
    E: RuntimeEngine + 'static,
    L: ComponentLoader + 'static,
    V: SecurityValidator,
    A: SecurityAuditLogger,
    B: MessageBroker<ComponentActorMessage> + Clone + Send + Sync + 'static,
{
    /// Creates a new SystemBuilder with all required dependencies.
    ///
    /// All dependencies must be provided upfront. Optional configuration
    /// can be set via setter methods before calling `build()`.
    ///
    /// # Arguments
    ///
    /// * `engine` - WASM runtime engine (e.g., `WasmtimeEngine`)
    /// * `loader` - Component binary loader (e.g., `FileComponentLoader`)
    /// * `security_validator` - Capability validator (e.g., `CapabilityValidator`)
    /// * `audit_logger` - Security audit logger (e.g., `ConsoleSecurityAuditLogger`)
    /// * `broker` - Message broker for the actor system (e.g., `InMemoryMessageBroker`)
    pub fn new(
        engine: Arc<E>,
        loader: Arc<L>,
        security_validator: Arc<V>,
        audit_logger: Arc<A>,
        broker: B,
    ) -> Self {
        Self {
            engine,
            loader,
            security_validator,
            audit_logger,
            broker,
            actor_system_config: SystemConfig::default(),
        }
    }

    /// Sets the actor system configuration.
    ///
    /// If not called, defaults to `SystemConfig::default()`.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the airssys-rt ActorSystem
    pub fn with_actor_system_config(mut self, config: SystemConfig) -> Self {
        self.actor_system_config = config;
        self
    }

    /// Builds the SystemCoordinator with the configured dependencies.
    ///
    /// Consumes the builder and delegates to `SystemCoordinator::new()` to
    /// wire all dependencies together.
    ///
    /// This method is infallible because all required dependencies are
    /// guaranteed to be present (enforced by the type system at `new()`).
    pub fn build(self) -> SystemCoordinator<E, L, V, A, B> {
        SystemCoordinator::new(
            self.engine,
            self.loader,
            self.security_validator,
            self.audit_logger,
            self.actor_system_config,
            self.broker,
        )
    }
}

// ============================================================================
// Debug Implementation
// ============================================================================

impl<E, L, V, A, B> fmt::Debug for SystemBuilder<E, L, V, A, B>
where
    E: RuntimeEngine + 'static,
    L: ComponentLoader + 'static,
    V: SecurityValidator,
    A: SecurityAuditLogger,
    B: MessageBroker<ComponentActorMessage> + Clone + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SystemBuilder")
            .field("actor_system_config", &self.actor_system_config)
            .finish_non_exhaustive()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Layer 1: Standard library imports
    use std::time::Duration;

    // Layer 2: Third-party crate imports
    use airssys_rt::broker::InMemoryMessageBroker;
    use airssys_rt::system::SystemConfig;

    // Layer 3: Internal module imports
    use crate::core::component::handle::ComponentHandle;
    use crate::core::component::id::ComponentId;
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
    // Type aliases for tests
    // ========================================
    type TestBroker = InMemoryMessageBroker<ComponentActorMessage>;
    type TestBuilder = SystemBuilder<
        MockRuntimeEngine,
        MockComponentLoader,
        MockSecurityValidator,
        MockAuditLogger,
        TestBroker,
    >;
    type TestCoordinator = SystemCoordinator<
        MockRuntimeEngine,
        MockComponentLoader,
        MockSecurityValidator,
        MockAuditLogger,
        TestBroker,
    >;

    // ========================================
    // Helper: create builder with mocks
    // ========================================
    fn create_test_builder() -> TestBuilder {
        SystemBuilder::new(
            Arc::new(MockRuntimeEngine),
            Arc::new(MockComponentLoader),
            Arc::new(MockSecurityValidator),
            Arc::new(MockAuditLogger),
            InMemoryMessageBroker::<ComponentActorMessage>::new(),
        )
    }

    fn create_test_id(name: &str) -> ComponentId {
        ComponentId::new("test", name, "v1")
    }

    // ========================================
    // 1. Constructor Tests
    // ========================================

    #[tokio::test]
    async fn test_builder_creation() {
        let builder = create_test_builder();
        // Builder was created without panic -- type system ensures all deps present
        let mut coordinator = builder.build();
        assert!(!coordinator.is_running());
        coordinator.shutdown().await.unwrap_or_default();
    }

    #[tokio::test]
    async fn test_builder_default_actor_system_config() {
        let builder = create_test_builder();
        let default_config = SystemConfig::default();

        // Build and verify the coordinator uses default config
        // We verify indirectly through the Debug output of the builder
        let debug_str = format!("{:?}", builder);
        let expected_debug = format!("{:?}", default_config);
        assert!(
            debug_str.contains(&expected_debug),
            "Builder debug should contain default SystemConfig debug representation"
        );

        let mut coordinator = builder.build();
        coordinator.shutdown().await.unwrap_or_default();
    }

    #[tokio::test]
    async fn test_builder_accepts_all_dependency_types() {
        // Verifies that all 5 generic type parameters compile correctly
        let engine = Arc::new(MockRuntimeEngine);
        let loader = Arc::new(MockComponentLoader);
        let validator = Arc::new(MockSecurityValidator);
        let logger = Arc::new(MockAuditLogger);
        let broker = InMemoryMessageBroker::<ComponentActorMessage>::new();

        let builder: TestBuilder = SystemBuilder::new(engine, loader, validator, logger, broker);
        let mut coordinator = builder.build();
        assert!(!coordinator.is_running());
        coordinator.shutdown().await.unwrap_or_default();
    }

    // ========================================
    // 2. Configuration Setter Tests
    // ========================================

    #[tokio::test]
    async fn test_with_actor_system_config() {
        let custom_config = SystemConfig::builder()
            .with_mailbox_capacity(500)
            .with_spawn_timeout(Duration::from_secs(10))
            .build()
            .unwrap();

        let builder = create_test_builder().with_actor_system_config(custom_config.clone());

        // Verify the custom config is stored via Debug output
        let debug_str = format!("{:?}", builder);
        assert!(
            debug_str.contains("500"),
            "Builder debug should reflect custom mailbox capacity of 500"
        );

        let mut coordinator = builder.build();
        coordinator.shutdown().await.unwrap_or_default();
    }

    #[tokio::test]
    async fn test_with_actor_system_config_chaining() {
        // Verifies method chaining returns Self (can chain further)
        let config1 = SystemConfig::builder()
            .with_mailbox_capacity(100)
            .build()
            .unwrap();

        let config2 = SystemConfig::builder()
            .with_mailbox_capacity(200)
            .build()
            .unwrap();

        // Chain two with_actor_system_config calls -- second overrides first
        let builder = create_test_builder()
            .with_actor_system_config(config1)
            .with_actor_system_config(config2);

        let debug_str = format!("{:?}", builder);
        assert!(
            debug_str.contains("200"),
            "Second config (mailbox 200) should override first (100)"
        );

        let mut coordinator = builder.build();
        coordinator.shutdown().await.unwrap_or_default();
    }

    #[tokio::test]
    async fn test_builder_method_chaining_fluent() {
        // Verifies the full fluent chain: new().with_config().build()
        let config = SystemConfig::builder()
            .with_mailbox_capacity(750)
            .build()
            .unwrap();

        let mut coordinator: TestCoordinator = SystemBuilder::new(
            Arc::new(MockRuntimeEngine),
            Arc::new(MockComponentLoader),
            Arc::new(MockSecurityValidator),
            Arc::new(MockAuditLogger),
            InMemoryMessageBroker::<ComponentActorMessage>::new(),
        )
        .with_actor_system_config(config)
        .build();

        assert!(!coordinator.is_running());
        coordinator.shutdown().await.unwrap_or_default();
    }

    // ========================================
    // 3. Build Tests
    // ========================================

    #[tokio::test]
    async fn test_build_produces_coordinator() {
        let builder = create_test_builder();
        let mut coordinator: TestCoordinator = builder.build();
        // Coordinator was produced successfully (type assertion above)
        assert!(!coordinator.is_running());
        coordinator.shutdown().await.unwrap_or_default();
    }

    #[tokio::test]
    async fn test_build_coordinator_not_running() {
        let mut coordinator = create_test_builder().build();
        assert!(!coordinator.is_running());
        coordinator.shutdown().await.unwrap_or_default();
    }

    #[tokio::test]
    async fn test_build_coordinator_registry_empty() {
        let mut coordinator = create_test_builder().build();
        assert_eq!(coordinator.component_count().unwrap(), 0);
        coordinator.shutdown().await.unwrap_or_default();
    }

    #[tokio::test]
    async fn test_build_coordinator_can_start() {
        let mut coordinator = create_test_builder().build();
        let result = coordinator.start();
        assert!(result.is_ok());
        assert!(coordinator.is_running());
        coordinator.shutdown().await.unwrap_or_default();
    }

    // ========================================
    // 4. Integration Tests (build + lifecycle)
    // ========================================

    #[tokio::test]
    async fn test_build_and_start_and_load_component() {
        let mut coordinator = create_test_builder().build();
        coordinator.start().unwrap();

        let id = create_test_id("echo");
        let result = coordinator.load_component(id.clone()).await;
        assert!(result.is_ok());
        assert!(coordinator.registry().contains(&id).unwrap());
        assert_eq!(coordinator.component_count().unwrap(), 1);

        coordinator.shutdown().await.unwrap_or_default();
    }

    #[tokio::test]
    async fn test_build_and_start_and_shutdown() {
        let mut coordinator = create_test_builder().build();
        coordinator.start().unwrap();
        assert!(coordinator.is_running());

        let result = coordinator.shutdown().await;
        assert!(result.is_ok());
        assert!(!coordinator.is_running());
    }

    #[tokio::test]
    async fn test_build_multiple_coordinators() {
        // Build two separate coordinators from two separate builders
        let mut coordinator1 = create_test_builder().build();
        let mut coordinator2 = create_test_builder().build();

        coordinator1.start().unwrap();
        coordinator2.start().unwrap();

        assert!(coordinator1.is_running());
        assert!(coordinator2.is_running());

        // Load a component only in coordinator1
        let id = create_test_id("only-in-c1");
        coordinator1.load_component(id.clone()).await.unwrap();

        assert_eq!(coordinator1.component_count().unwrap(), 1);
        assert_eq!(coordinator2.component_count().unwrap(), 0);

        coordinator1.shutdown().await.unwrap_or_default();
        coordinator2.shutdown().await.unwrap_or_default();
    }

    // ========================================
    // 5. Debug Trait Test
    // ========================================

    #[test]
    fn test_builder_debug_format() {
        let builder = create_test_builder();
        let debug_str = format!("{:?}", builder);
        assert!(
            debug_str.contains("SystemBuilder"),
            "Debug output should contain 'SystemBuilder'"
        );
        assert!(
            debug_str.contains("actor_system_config"),
            "Debug output should contain 'actor_system_config'"
        );
        // finish_non_exhaustive() appends ".." to indicate omitted fields
        assert!(
            debug_str.contains(".."),
            "Debug output should contain '..' from finish_non_exhaustive()"
        );
    }

    // ========================================
    // 6. Thread Safety Test
    // ========================================

    #[test]
    fn test_builder_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<TestBuilder>();
    }
}
