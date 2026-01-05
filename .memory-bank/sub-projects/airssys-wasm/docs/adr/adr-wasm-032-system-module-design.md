# ADR-WASM-032: System Module Design

**ADR ID:** ADR-WASM-032  
**Created:** 2026-01-05  
**Updated:** 2026-01-05  
**Status:** Accepted  
**Deciders:** Architecture Team  
**Category:** Module Design / System Coordinator  
**Parent:** [ADR-WASM-026](adr-wasm-026-implementation-roadmap-clean-slate-rebuild.md) (Phase 7)

---

## Title

System Module Design as Layer 4 Coordinator

---

## Context

The `system/` module is **Layer 4** of the architecture - the top-level coordinator. It:
- Receives pre-built dependencies from the caller via builder pattern
- Wires dependencies together following DIP
- Manages system lifecycle (init/shutdown)
- Is the ONLY module that knows all modules exist

**Import Rules:**
- ✅ Can import: ALL modules (`core/`, `security/`, `runtime/`, `component/`, `messaging/`)
- ✅ Provides: Central coordination point

### Design Principles

1. **Caller Creates Implementations**: The caller (application) creates concrete implementations
2. **Builder Accepts Traits**: `SystemBuilder` accepts `Arc<dyn Trait>` not concrete types
3. **Optional Defaults**: Builder provides defaults when not explicitly set
4. **Clear Naming**: Avoid confusion with `runtime/` module

### Naming Decision

| Term | Module | Purpose |
|------|--------|---------|
| `WasmtimeEngine` | `runtime/` | WASM execution |
| `SystemCoordinator` | `system/` | System orchestration |
| `SystemBuilder` | `system/` | Builder for coordinator |

### References

- [ADR-WASM-025](adr-wasm-025-clean-slate-rebuild-architecture.md): Clean-Slate Architecture
- [KNOWLEDGE-WASM-037](../knowledges/knowledge-wasm-037-rebuild-architecture-clean-slate.md): Technical Reference

---

## Decision

### System Module Structure

```
system/
├── mod.rs
├── coordinator.rs      # SystemCoordinator - main orchestrator
├── lifecycle.rs        # LifecycleManager - init/shutdown
└── builder.rs          # SystemBuilder - builder pattern
```

---

## Detailed Specifications

### system/builder.rs

```rust
use std::sync::Arc;
use std::time::Duration;

use crate::core::errors::wasm::WasmError;
use crate::core::runtime::traits::{ComponentLoader, RuntimeEngine};
use crate::core::security::traits::{SecurityValidator, SecurityAuditLogger};
use crate::core::messaging::traits::{MessageRouter, CorrelationTracker};

use crate::component::supervisor::{BackoffStrategy, SupervisorConfig};

use super::coordinator::SystemCoordinator;

/// Builder for SystemCoordinator with dependency injection
/// 
/// Caller creates concrete implementations and injects them via builder.
/// Builder uses core module trait abstractions as the contract.
pub struct SystemBuilder {
    // Injected dependencies (all optional - will use defaults if not set)
    engine: Option<Arc<dyn RuntimeEngine>>,
    loader: Option<Arc<dyn ComponentLoader>>,
    security_validator: Option<Arc<dyn SecurityValidator>>,
    audit_logger: Option<Arc<dyn SecurityAuditLogger>>,
    
    // Configuration
    component_path: String,
    supervisor_config: SupervisorConfig,
}

impl SystemBuilder {
    pub fn new() -> Self {
        Self {
            engine: None,
            loader: None,
            security_validator: None,
            audit_logger: None,
            component_path: "./components".to_string(),
            supervisor_config: SupervisorConfig::default(),
        }
    }

    // === Dependency Injection Methods ===

    /// Inject a RuntimeEngine implementation
    pub fn with_engine(mut self, engine: Arc<dyn RuntimeEngine>) -> Self {
        self.engine = Some(engine);
        self
    }

    /// Inject a ComponentLoader implementation  
    pub fn with_loader(mut self, loader: Arc<dyn ComponentLoader>) -> Self {
        self.loader = Some(loader);
        self
    }

    /// Inject a SecurityValidator implementation
    pub fn with_security_validator(mut self, validator: Arc<dyn SecurityValidator>) -> Self {
        self.security_validator = Some(validator);
        self
    }

    /// Inject a SecurityAuditLogger implementation
    pub fn with_audit_logger(mut self, logger: Arc<dyn SecurityAuditLogger>) -> Self {
        self.audit_logger = Some(logger);
        self
    }

    // === Configuration Methods ===

    /// Set the path where components are stored (used for default loader)
    pub fn component_path(mut self, path: impl Into<String>) -> Self {
        self.component_path = path.into();
        self
    }

    /// Set supervisor max restarts
    pub fn max_restarts(mut self, count: u32) -> Self {
        self.supervisor_config.max_restarts = count;
        self
    }

    /// Set supervisor restart window
    pub fn restart_window(mut self, duration: Duration) -> Self {
        self.supervisor_config.restart_window = duration;
        self
    }

    /// Set fixed backoff strategy
    pub fn fixed_backoff(mut self, delay: Duration) -> Self {
        self.supervisor_config.backoff_strategy = BackoffStrategy::Fixed(delay);
        self
    }

    /// Set exponential backoff strategy
    pub fn exponential_backoff(mut self, base: Duration, max: Duration) -> Self {
        self.supervisor_config.backoff_strategy = BackoffStrategy::Exponential { base, max };
        self
    }

    /// Build the SystemCoordinator
    /// 
    /// Uses provided dependencies or creates defaults.
    pub fn build(self) -> Result<SystemCoordinator, WasmError> {
        SystemCoordinator::from_builder(self)
    }

    // === Internal accessors for SystemCoordinator ===
    
    pub(super) fn take_engine(&mut self) -> Option<Arc<dyn RuntimeEngine>> {
        self.engine.take()
    }

    pub(super) fn take_loader(&mut self) -> Option<Arc<dyn ComponentLoader>> {
        self.loader.take()
    }

    pub(super) fn take_security_validator(&mut self) -> Option<Arc<dyn SecurityValidator>> {
        self.security_validator.take()
    }

    pub(super) fn take_audit_logger(&mut self) -> Option<Arc<dyn SecurityAuditLogger>> {
        self.audit_logger.take()
    }

    pub(super) fn component_path(&self) -> &str {
        &self.component_path
    }

    pub(super) fn supervisor_config(&self) -> &SupervisorConfig {
        &self.supervisor_config
    }
}

impl Default for SystemBuilder {
    fn default() -> Self {
        Self::new()
    }
}
```

---

### system/coordinator.rs

```rust
use std::sync::Arc;

use airssys_rt::ActorSystem;

use crate::core::component::id::ComponentId;
use crate::core::errors::wasm::WasmError;
use crate::core::runtime::traits::{ComponentLoader, RuntimeEngine};
use crate::core::security::traits::{SecurityValidator, SecurityAuditLogger};

use crate::component::registry::ComponentRegistry;
use crate::component::spawner::ComponentSpawner;
use crate::messaging::correlation::CorrelationTrackerImpl;
use crate::messaging::subscriber::ComponentSubscriber;

// Default implementations (only used when not injected)
use crate::runtime::engine::WasmtimeEngine;
use crate::runtime::loader::FileComponentLoader;
use crate::security::capability::validator::CapabilityValidator;
use crate::security::audit::ConsoleSecurityAuditLogger;

use super::builder::SystemBuilder;

/// System coordinator - orchestrates all components
/// 
/// This is the composition root where all dependencies are wired together.
/// Dependencies are injected via SystemBuilder; defaults used if not provided.
pub struct SystemCoordinator {
    // Injected dependencies (as trait objects)
    engine: Arc<dyn RuntimeEngine>,
    loader: Arc<dyn ComponentLoader>,
    security_validator: Arc<dyn SecurityValidator>,
    audit_logger: Arc<dyn SecurityAuditLogger>,
    
    // Internal components (created by coordinator)
    registry: Arc<ComponentRegistry>,
    spawner: ComponentSpawner,
    subscriber: Arc<ComponentSubscriber>,
    correlation_tracker: Arc<CorrelationTrackerImpl>,
    
    // Actor system (from airssys-rt)
    actor_system: ActorSystem,
    
    // State
    is_running: bool,
}

impl SystemCoordinator {
    /// Create from builder with injected or default dependencies
    pub(super) fn from_builder(mut builder: SystemBuilder) -> Result<Self, WasmError> {
        // Use injected dependencies or create defaults
        let engine: Arc<dyn RuntimeEngine> = builder.take_engine().unwrap_or_else(|| {
            Arc::new(WasmtimeEngine::new().expect("Failed to create default engine"))
        });

        let loader: Arc<dyn ComponentLoader> = builder.take_loader().unwrap_or_else(|| {
            Arc::new(FileComponentLoader::new(builder.component_path()))
        });

        let security_validator: Arc<dyn SecurityValidator> = builder
            .take_security_validator()
            .unwrap_or_else(|| Arc::new(CapabilityValidator::new()));

        let audit_logger: Arc<dyn SecurityAuditLogger> = builder
            .take_audit_logger()
            .unwrap_or_else(|| Arc::new(ConsoleSecurityAuditLogger::new()));

        // Create internal components
        let registry = Arc::new(ComponentRegistry::new());
        let subscriber = Arc::new(ComponentSubscriber::new());
        let correlation_tracker = Arc::new(CorrelationTrackerImpl::new());

        // Create actor system
        let actor_system = ActorSystem::new()
            .map_err(|e| WasmError::RuntimeError(e.to_string()))?;

        // Create spawner with trait-based dependencies
        let spawner = ComponentSpawner::new(
            Arc::clone(&engine),
            Arc::clone(&loader),
            Arc::clone(&registry),
            builder.supervisor_config().clone(),
        );

        Ok(Self {
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
        })
    }

    /// Start the system
    pub async fn start(&mut self) -> Result<(), WasmError> {
        if self.is_running {
            return Err(WasmError::RuntimeError("Already running".to_string()));
        }

        self.actor_system
            .start()
            .await
            .map_err(|e| WasmError::RuntimeError(e.to_string()))?;

        self.is_running = true;
        Ok(())
    }

    /// Shutdown the system
    pub async fn shutdown(&mut self) -> Result<(), WasmError> {
        if !self.is_running {
            return Ok(());
        }

        // Stop all components
        for id in self.registry.list() {
            let _ = self.spawner.stop(&id).await;
        }

        self.actor_system
            .shutdown()
            .await
            .map_err(|e| WasmError::RuntimeError(e.to_string()))?;

        self.is_running = false;
        Ok(())
    }

    /// Load and spawn a component
    pub async fn load_component(&self, id: ComponentId) -> Result<(), WasmError> {
        if !self.is_running {
            return Err(WasmError::RuntimeError("System not started".to_string()));
        }

        self.spawner.spawn(&self.actor_system, id).await
    }

    /// Unload a component
    pub async fn unload_component(&self, id: &ComponentId) -> Result<(), WasmError> {
        self.spawner.stop(id).await
    }

    // === Accessors (return trait objects, not concrete) ===

    pub fn engine(&self) -> &Arc<dyn RuntimeEngine> {
        &self.engine
    }

    pub fn security_validator(&self) -> &Arc<dyn SecurityValidator> {
        &self.security_validator
    }

    pub fn registry(&self) -> &Arc<ComponentRegistry> {
        &self.registry
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }
}
```

---

### system/lifecycle.rs

```rust
use crate::core::errors::wasm::WasmError;

use super::coordinator::SystemCoordinator;

/// System lifecycle phases
#[derive(Debug, Clone, PartialEq)]
pub enum LifecyclePhase {
    Created,
    Starting,
    Running,
    Stopping,
    Stopped,
}

/// Lifecycle events for observers
#[derive(Debug, Clone)]
pub enum LifecycleEvent {
    Starting,
    Started,
    Stopping,
    Stopped,
    ComponentLoaded(String),
    ComponentUnloaded(String),
    Error(String),
}

/// Lifecycle listener trait
pub trait LifecycleListener: Send + Sync {
    fn on_event(&self, event: LifecycleEvent);
}

/// Manages system lifecycle with event notifications
pub struct LifecycleManager {
    phase: LifecyclePhase,
    listeners: Vec<Box<dyn LifecycleListener>>,
}

impl LifecycleManager {
    pub fn new() -> Self {
        Self {
            phase: LifecyclePhase::Created,
            listeners: Vec::new(),
        }
    }

    pub fn add_listener(&mut self, listener: Box<dyn LifecycleListener>) {
        self.listeners.push(listener);
    }

    pub fn phase(&self) -> &LifecyclePhase {
        &self.phase
    }

    fn notify(&self, event: LifecycleEvent) {
        for listener in &self.listeners {
            listener.on_event(event.clone());
        }
    }

    pub async fn start(&mut self, coordinator: &mut SystemCoordinator) -> Result<(), WasmError> {
        self.phase = LifecyclePhase::Starting;
        self.notify(LifecycleEvent::Starting);

        coordinator.start().await?;

        self.phase = LifecyclePhase::Running;
        self.notify(LifecycleEvent::Started);

        Ok(())
    }

    pub async fn stop(&mut self, coordinator: &mut SystemCoordinator) -> Result<(), WasmError> {
        self.phase = LifecyclePhase::Stopping;
        self.notify(LifecycleEvent::Stopping);

        coordinator.shutdown().await?;

        self.phase = LifecyclePhase::Stopped;
        self.notify(LifecycleEvent::Stopped);

        Ok(())
    }
}

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## Usage Examples

### Basic Usage (with defaults)

```rust
// Uses default implementations for everything
let coordinator = SystemBuilder::new()
    .component_path("./my-components")
    .max_restarts(5)
    .build()?;
```

### Custom Implementations (dependency injection)

```rust
// Caller creates concrete implementations
let engine = Arc::new(WasmtimeEngine::new()?) as Arc<dyn RuntimeEngine>;
let loader = Arc::new(FileComponentLoader::new("./components")) as Arc<dyn ComponentLoader>;
let validator = Arc::new(CapabilityValidator::new()) as Arc<dyn SecurityValidator>;

// Inject via builder
let coordinator = SystemBuilder::new()
    .with_engine(engine)
    .with_loader(loader)
    .with_security_validator(validator)
    .max_restarts(10)
    .build()?;
```

### Testing (mock injection)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct MockEngine;
    impl RuntimeEngine for MockEngine {
        // Mock implementation...
    }

    #[tokio::test]
    async fn test_with_mock_engine() {
        let mock_engine = Arc::new(MockEngine) as Arc<dyn RuntimeEngine>;
        
        let coordinator = SystemBuilder::new()
            .with_engine(mock_engine)
            .build()
            .unwrap();
        
        // Test with mock...
    }
}
```

---

## Dependency Flow Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                      Caller (Application)                       │
│                                                                 │
│  // Create concrete implementations                             │
│  let engine = Arc::new(WasmtimeEngine::new()?);                │
│  let loader = Arc::new(FileComponentLoader::new("./"));        │
│  let validator = Arc::new(CapabilityValidator::new());         │
│                                                                 │
│  // Inject via builder                                          │
│  let coordinator = SystemBuilder::new()                         │
│      .with_engine(engine)       // Arc<dyn RuntimeEngine>       │
│      .with_loader(loader)       // Arc<dyn ComponentLoader>     │
│      .with_security_validator(validator)                        │
│      .build()?;                                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                 SystemCoordinator (system/)                     │
│                                                                 │
│  Receives: Arc<dyn RuntimeEngine>, Arc<dyn ComponentLoader>... │
│  Creates: ComponentSpawner, ComponentRegistry, etc.             │
│  Wires: Dependencies into internal components                   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│              component/ + messaging/ (Layer 3)                  │
│                                                                 │
│  ComponentSpawner receives: Arc<dyn RuntimeEngine>              │
│  ComponentWrapper receives: Arc<dyn RuntimeEngine>              │
│  (Never knows concrete type - pure DIP)                         │
└─────────────────────────────────────────────────────────────────┘
```

---

## Test Fixtures (Phase 7)

### Required Fixtures

| Fixture | Purpose | WIT Exports |
|---------|---------|-------------|
| `echo.wasm` | Basic message handling | `handle-message` returns input |
| `counter.wasm` | Stateful component | `handle-message` increments counter |
| `callback.wasm` | Request-response | Uses `host-messaging.request()` |

---

## History

| Date | Version | Change |
|------|---------|--------|
| 2026-01-05 | 1.0 | Initial system module design |
| 2026-01-05 | 2.0 | Refactored: RuntimeManager → SystemCoordinator, builder pattern for dependency injection |

---

**This ADR defines the system module structure for Phase 7 of the rebuild.**
