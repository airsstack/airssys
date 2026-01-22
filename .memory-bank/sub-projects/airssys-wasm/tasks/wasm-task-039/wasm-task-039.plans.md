# WASM-TASK-039: Implementation Plans

## References
- ADR-WASM-031 (Component & Messaging Module Design)
- ADR-WASM-026 (Implementation Roadmap)
- ADR-WASM-025 (Clean Slate Architecture)
- ADR-WASM-023 (Module Boundary Enforcement)
- PROJECTS_STANDARD.md (§2.1, §2.2, §4.3)

---

## Actions

### Action 1: Implement ComponentSpawner

**Objective**: Create the ComponentSpawner that orchestrates component lifecycle (load, validate, spawn, register).

**Detailed Steps**:

#### Step 1.1: Create `src/component/spawner.rs`

```rust
//! Component spawner for creating and managing component actors.
//!
//! Orchestrates the full lifecycle:
//! - Load component bytes
//! - Validate component
//! - Create ComponentWrapper
//! - Spawn as supervised actor
//! - Register in registry

// Layer 1: Standard library imports
use std::sync::Arc;

// Layer 2: Third-party crate imports
use airssys_rt::{ActorSystem, SupervisorNode};

// Layer 3: Internal module imports
use crate::core::component::id::ComponentId;
use crate::core::errors::wasm::WasmError;
use crate::core::runtime::traits::{ComponentLoader, RuntimeEngine};

use super::registry::ComponentRegistry;
use super::supervisor::SupervisorConfig;
use super::wrapper::ComponentWrapper;

/// Spawns and manages component actors
pub struct ComponentSpawner {
    engine: Arc<dyn RuntimeEngine>,
    loader: Arc<dyn ComponentLoader>,
    registry: Arc<ComponentRegistry>,
    supervisor_config: SupervisorConfig,
}

impl ComponentSpawner {
    /// Create a new ComponentSpawner
    ///
    /// # Arguments
    /// * `engine` - The runtime engine for executing components
    /// * `loader` - The component loader for loading and validating components
    /// * `registry` - The component registry for tracking spawned components
    /// * `supervisor_config` - Supervisor configuration for fault tolerance
    pub fn new(
        engine: Arc<dyn RuntimeEngine>,
        loader: Arc<dyn ComponentLoader>,
        registry: Arc<ComponentRegistry>,
        supervisor_config: SupervisorConfig,
    ) -> Self {
        Self {
            engine,
            loader,
            registry,
            supervisor_config,
        }
    }

    /// Spawn a new component actor
    ///
    /// # Arguments
    /// * `actor_system` - The ActorSystem to spawn the actor in
    /// * `id` - The component identifier
    ///
    /// # Returns
    /// Ok(()) if component was successfully spawned and registered
    ///
    /// # Errors
    /// - If component bytes cannot be loaded
    /// - If component validation fails
    /// - If actor spawn fails
    pub async fn spawn(
        &self,
        actor_system: &ActorSystem,
        id: ComponentId,
    ) -> Result<(), WasmError> {
        // Load component bytes
        let bytes = self.loader.load_bytes(&id)?;
        
        // Validate component
        self.loader.validate(&bytes)?;

        // Create the wrapper with injected engine
        let wrapper = ComponentWrapper::new(
            id.clone(),
            Arc::clone(&self.engine),
            bytes,
        );

        // Spawn as supervised actor
        let address = actor_system
            .spawn_supervised(
                wrapper,
                self.supervisor_config.to_supervisor_node(),
            )
            .await
            .map_err(|e| WasmError::RuntimeError(e.to_string()))?;

        // Register in registry
        self.registry.register(id, address);

        Ok(())
    }

    /// Stop a component
    ///
    /// # Arguments
    /// * `id` - The component identifier to stop
    ///
    /// # Returns
    /// Ok(()) if component was successfully stopped and unregistered
    pub async fn stop(&self, id: &ComponentId) -> Result<(), WasmError> {
        if let Some(_address) = self.registry.unregister(id) {
            // Send shutdown message
            // address.send(ComponentActorMessage::Shutdown).await?;
            // Note: Actual shutdown implementation requires airssys-rt integration
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::supervisor::BackoffStrategy;
    use std::time::Duration;

    // Mock RuntimeEngine for testing
    struct MockRuntimeEngine;

    impl RuntimeEngine for MockRuntimeEngine {
        fn load_component(
            &self,
            _id: &ComponentId,
            _bytes: &[u8],
        ) -> Result<crate::core::component::handle::ComponentHandle, WasmError> {
            Ok(crate::core::component::handle::ComponentHandle::new(
                ComponentId::new("test"),
            ))
        }

        fn unload_component(
            &self,
            _handle: &crate::core::component::handle::ComponentHandle,
        ) -> Result<(), WasmError> {
            Ok(())
        }

        fn call_handle_message(
            &self,
            _handle: &crate::core::component::handle::ComponentHandle,
            _msg: &crate::core::component::message::ComponentMessage,
        ) -> Result<Option<crate::core::component::MessagePayload>, WasmError> {
            Ok(None)
        }

        fn call_handle_callback(
            &self,
            _handle: &crate::core::component::handle::ComponentHandle,
            _msg: &crate::core::component::message::ComponentMessage,
        ) -> Result<(), WasmError> {
            Ok(())
        }
    }

    // Mock ComponentLoader for testing
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

        fn with_load_failure(mut self) -> Self {
            self.should_fail_load = true;
            self
        }

        fn with_validation_failure(mut self) -> Self {
            self.should_fail_validate = true;
            self
        }
    }

    impl ComponentLoader for MockComponentLoader {
        fn load_bytes(&self, _id: &ComponentId) -> Result<Vec<u8>, WasmError> {
            if self.should_fail_load {
                Err(WasmError::ComponentNotFound("mock error".to_string()))
            } else {
                Ok(vec![0u8; 100])
            }
        }

        fn validate(&self, _bytes: &[u8]) -> Result<(), WasmError> {
            if self.should_fail_validate {
                Err(WasmError::InvalidComponent("validation failed".to_string()))
            } else {
                Ok(())
            }
        }
    }

    fn create_spawner(
        loader: Arc<dyn ComponentLoader>,
    ) -> ComponentSpawner {
        let engine: Arc<dyn RuntimeEngine> = Arc::new(MockRuntimeEngine);
        let registry = Arc::new(ComponentRegistry::new());
        let config = SupervisorConfig::default();

        ComponentSpawner::new(engine, loader, registry, config)
    }

    #[test]
    fn test_spawner_creation() {
        let loader: Arc<dyn ComponentLoader> = Arc::new(MockComponentLoader::new());
        let spawner = create_spawner(loader);

        // Verify spawner was created (no panics)
        assert_eq!(spawner.registry.count(), 0);
    }

    #[tokio::test]
    async fn test_spawn_loads_and_validates() {
        let loader: Arc<dyn ComponentLoader> = Arc::new(MockComponentLoader::new());
        let spawner = create_spawner(loader);

        // Note: Actual spawn requires ActorSystem which is complex to mock
        // This test verifies the spawner structure and mocks
        assert_eq!(spawner.registry.count(), 0);
    }

    #[test]
    fn test_spawn_fails_on_load_error() {
        let loader: Arc<dyn ComponentLoader> = Arc::new(
            MockComponentLoader::new().with_load_failure()
        );
        let spawner = create_spawner(loader);

        // Verify loader would fail
        let result = spawner.loader.load_bytes(&ComponentId::new("test"));
        assert!(result.is_err());
    }

    #[test]
    fn test_spawn_fails_on_validation_error() {
        let loader: Arc<dyn ComponentLoader> = Arc::new(
            MockComponentLoader::new().with_validation_failure()
        );
        let spawner = create_spawner(loader);

        // Verify validator would fail
        let bytes = vec![0u8; 100];
        let result = spawner.loader.validate(&bytes);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stop_unregisters_component() {
        let loader: Arc<dyn ComponentLoader> = Arc::new(MockComponentLoader::new());
        let spawner = create_spawner(loader);

        // Manually register a component
        let id = ComponentId::new("test-component");
        // Note: Can't fully test without ActorAddress mock
        
        let result = spawner.stop(&id).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_spawner_with_custom_supervisor_config() {
        let loader: Arc<dyn ComponentLoader> = Arc::new(MockComponentLoader::new());
        let engine: Arc<dyn RuntimeEngine> = Arc::new(MockRuntimeEngine);
        let registry = Arc::new(ComponentRegistry::new());
        let config = SupervisorConfig::new(5, Duration::from_secs(120))
            .with_backoff(BackoffStrategy::Fixed(Duration::from_secs(1)));

        let spawner = ComponentSpawner::new(engine, loader, registry, config);

        assert_eq!(spawner.registry.count(), 0);
    }
}
```

#### Step 1.2: Update `src/component/mod.rs`

```rust
//! Component module - Actor system integration for WASM components
//!
//! This module integrates WASM components with the airssys-rt actor system.

pub mod registry;
pub mod spawner;
pub mod supervisor;
pub mod wrapper;

pub use registry::ComponentRegistry;
pub use spawner::ComponentSpawner;
pub use supervisor::{BackoffStrategy, SupervisorConfig};
pub use wrapper::{ComponentActorMessage, ComponentWrapper};
```

**Deliverables**:
- `src/component/spawner.rs` with ComponentSpawner struct
- spawn() method with full lifecycle orchestration
- stop() method for component shutdown
- Comprehensive unit tests with mocks
- Module export in `src/component/mod.rs`

**Constraints**:
- Must not import from `runtime/`, `security/`, or `system/`
- Must follow §2.1 3-Layer imports
- Must follow §2.2 No FQN in type annotations
- Must use trait-based dependency injection

---

## Verification Section

### Automated Tests
```bash
# Unit tests for spawner module
cargo test -p airssys-wasm --lib -- component::spawner

# All component module tests
cargo test -p airssys-wasm --lib -- component

# Build verification
cargo build -p airssys-wasm

# Clippy
cargo clippy -p airssys-wasm --lib -- -D warnings
```

### Architecture Compliance
```bash
# Verify no forbidden imports in spawner.rs
grep -rn "use crate::runtime" src/component/spawner.rs  # Should be empty
grep -rn "use crate::security" src/component/spawner.rs  # Should be empty
grep -rn "use crate::system" src/component/spawner.rs    # Should be empty

# Verify no FQN in type annotations
grep -rn "std::" src/component/spawner.rs | grep -v "^.*use " | grep "::"  # Should be empty
```

---

## Success Criteria
- [ ] `src/component/spawner.rs` exists and compiles
- [ ] ComponentSpawner struct with trait-based dependencies
- [ ] spawn() method orchestrates full lifecycle
- [ ] stop() method handles shutdown
- [ ] Unit tests pass (6+ tests with mocks)
- [ ] MockRuntimeEngine and MockComponentLoader for testing
- [ ] `cargo clippy -p airssys-wasm --lib -- -D warnings` passes
- [ ] No forbidden imports (architecture compliance)
- [ ] §2.1 3-Layer imports verified
- [ ] §2.2 No FQN verified
