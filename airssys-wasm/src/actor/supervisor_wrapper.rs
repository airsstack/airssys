//! SupervisorNode wrapper for ComponentActor supervision.
//!
//! This module provides the concrete integration between airssys-wasm's
//! ComponentSupervisor (Layer 1) and airssys-rt's SupervisorNode (Layer 3).
//!
//! # Architecture Role (ADR-WASM-018)
//!
//! - **Consumes**: SupervisorConfig from Layer 1
//! - **Uses**: SupervisorNode from Layer 3
//! - **Manages**: ComponentId ↔ ChildId mapping
//!
//! # Supervision Strategy
//!
//! Currently uses OneForOne strategy (each component supervised independently).
//! Future: Support OneForAll and RestForOne for component groups.
//!
//! # Performance
//!
//! - Registration: O(1) hashmap insertion + SupervisorNode overhead
//! - State query: O(1) hashmap lookup
//! - Start/Stop: Direct SupervisorNode delegation
//!
//! # Example
//!
//! ```rust,ignore
//! use airssys_wasm::actor::{SupervisorNodeWrapper, ComponentActor, SupervisorConfig};
//! use airssys_wasm::core::ComponentId;
//!
//! async fn create_wrapper() {
//!     let mut wrapper = SupervisorNodeWrapper::new();
//!     
//!     let component_id = ComponentId::new("my-component");
//!     let actor = ComponentActor::new(/* ... */);
//!     let config = SupervisorConfig::permanent();
//!     
//!     wrapper.register_component(component_id, actor, config).await.unwrap();
//! }
//! ```

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use tokio::sync::RwLock;

use airssys_rt::monitoring::{NoopMonitor, SupervisionEvent};
use airssys_rt::supervisor::{
    ChildId as RtChildId, ChildSpec as RtChildSpec, ChildState as RtChildState, OneForOne,
    RestartPolicy as RtRestartPolicy, ShutdownPolicy, Supervisor, SupervisorNode,
};

// Layer 3: Internal module imports
use crate::actor::{
    ComponentActor, ComponentSupervisionState, RestartPolicy, SupervisorConfig, SupervisorNodeBridge,
};
use crate::core::{ComponentId, WasmError};

/// Wrapper around airssys-rt SupervisorNode for ComponentActor supervision.
///
/// This struct bridges Layer 1 (WASM configuration) with Layer 3 (actor supervision),
/// maintaining clear ownership boundaries per ADR-WASM-018.
///
/// # Architecture Role
///
/// - **Consumes**: SupervisorConfig from Layer 1
/// - **Uses**: SupervisorNode from Layer 3
/// - **Manages**: ComponentId ↔ ChildId mapping
///
/// # Supervision Strategy
///
/// Currently uses OneForOne strategy (each component supervised independently).
/// Future: Support OneForAll and RestForOne for component groups.
///
/// # Performance
///
/// - Registration: O(1) hashmap insertion + SupervisorNode overhead
/// - State query: O(1) hashmap lookup
/// - Start/Stop: Direct SupervisorNode delegation
pub struct SupervisorNodeWrapper {
    /// Underlying airssys-rt SupervisorNode (OneForOne strategy)
    supervisor: Arc<RwLock<SupervisorNode<OneForOne, ComponentActor, NoopMonitor<SupervisionEvent>>>>,

    /// Mapping: ComponentId → ChildId (for SupervisorNode)
    component_to_child: Arc<RwLock<HashMap<ComponentId, String>>>,

    /// Reverse mapping: ChildId → ComponentId
    child_to_component: Arc<RwLock<HashMap<String, ComponentId>>>,
}

impl SupervisorNodeWrapper {
    /// Create a new SupervisorNodeWrapper.
    ///
    /// # Returns
    ///
    /// Wrapper with OneForOne supervision strategy (default for independent components).
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::SupervisorNodeWrapper;
    ///
    /// let wrapper = SupervisorNodeWrapper::new();
    /// ```
    pub fn new() -> Self {
        let supervisor_node = SupervisorNode::<OneForOne, ComponentActor, _>::new(
            OneForOne,
            NoopMonitor::<SupervisionEvent>::new(),
        );

        Self {
            supervisor: Arc::new(RwLock::new(supervisor_node)),
            component_to_child: Arc::new(RwLock::new(HashMap::new())),
            child_to_component: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with custom SupervisorNode instance.
    ///
    /// Allows testing with mock SupervisorNode or custom configuration.
    ///
    /// # Parameters
    ///
    /// - `supervisor`: Custom SupervisorNode instance
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::SupervisorNodeWrapper;
    /// use airssys_rt::supervisor::{SupervisorNode, OneForOne};
    /// use airssys_rt::monitoring::InMemoryMonitor;
    ///
    /// let monitor = InMemoryMonitor::new(Default::default());
    /// let supervisor = SupervisorNode::new(OneForOne, monitor);
    /// let wrapper = SupervisorNodeWrapper::with_supervisor(supervisor);
    /// ```
    pub fn with_supervisor(
        supervisor: SupervisorNode<OneForOne, ComponentActor, NoopMonitor<SupervisionEvent>>,
    ) -> Self {
        Self {
            supervisor: Arc::new(RwLock::new(supervisor)),
            component_to_child: Arc::new(RwLock::new(HashMap::new())),
            child_to_component: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Convert Layer 1 RestartPolicy to Layer 3 RestartPolicy.
    ///
    /// Maps WASM-specific restart policies to airssys-rt restart policies.
    ///
    /// # Parameters
    ///
    /// - `policy`: Layer 1 restart policy
    ///
    /// # Returns
    ///
    /// Equivalent Layer 3 restart policy
    fn convert_restart_policy(policy: RestartPolicy) -> RtRestartPolicy {
        match policy {
            RestartPolicy::Permanent => RtRestartPolicy::Permanent,
            RestartPolicy::Transient => RtRestartPolicy::Transient,
            RestartPolicy::Temporary => RtRestartPolicy::Temporary,
        }
    }

    /// Convert Layer 3 ChildState to Layer 1 ComponentSupervisionState.
    ///
    /// Maps airssys-rt child states to WASM component supervision states.
    ///
    /// # Parameters
    ///
    /// - `state`: Layer 3 child state
    ///
    /// # Returns
    ///
    /// Equivalent Layer 1 supervision state
    fn convert_child_state(state: RtChildState) -> ComponentSupervisionState {
        match state {
            RtChildState::Starting => ComponentSupervisionState::Starting,
            RtChildState::Running => ComponentSupervisionState::Running,
            RtChildState::Restarting => ComponentSupervisionState::Restarting,
            RtChildState::Stopping => ComponentSupervisionState::Stopped, // Treat stopping as stopped
            RtChildState::Stopped => ComponentSupervisionState::Stopped,
            RtChildState::Failed => ComponentSupervisionState::Failed,
        }
    }
}

#[async_trait]
impl SupervisorNodeBridge for SupervisorNodeWrapper {
    async fn register_component(
        &mut self,
        component_id: ComponentId,
        actor: ComponentActor,
        config: SupervisorConfig,
    ) -> Result<(), WasmError> {
        // Generate ChildId from ComponentId
        let child_id = component_id.as_str().to_string();

        // Check for duplicates
        {
            let mappings = self.component_to_child.read().await;
            if mappings.contains_key(&component_id) {
                return Err(WasmError::internal(format!(
                    "Component {} already supervised",
                    component_id.as_str()
                )));
            }
        }

        // Create ChildSpec for SupervisorNode
        // Wrap actor in Arc to allow the factory closure to be Fn instead of FnOnce
        let actor_cell = Arc::new(RwLock::new(Some(actor)));
        let child_spec = RtChildSpec {
            id: child_id.clone(),
            factory: move || {
                // Take the actor from the cell (only works for first call)
                let mut cell = actor_cell.blocking_write();
                cell.take().unwrap_or_else(|| {
                    // This should never happen as SupervisorNode only calls factory once
                    // Using unreachable! as this is a logic error if it ever occurs
                    unreachable!("Factory called multiple times - ComponentActor can only be created once")
                })
            },
            restart_policy: Self::convert_restart_policy(config.restart_policy),
            shutdown_policy: ShutdownPolicy::Graceful(config.shutdown_timeout),
            start_timeout: Duration::from_secs(30), // Default start timeout
            shutdown_timeout: config.shutdown_timeout,
        };

        // Register with SupervisorNode (Layer 3)
        {
            let mut supervisor = self.supervisor.write().await;
            supervisor
                .start_child(child_spec)
                .await
                .map_err(|e| WasmError::internal(format!("Supervision failed: {}", e)))?;
        }

        // Update mappings
        {
            let mut comp_to_child = self.component_to_child.write().await;
            let mut child_to_comp = self.child_to_component.write().await;
            comp_to_child.insert(component_id.clone(), child_id.clone());
            child_to_comp.insert(child_id, component_id);
        }

        Ok(())
    }

    async fn start_component(&mut self, component_id: &ComponentId) -> Result<(), WasmError> {
        // Look up ChildId
        let _child_id = {
            let mappings = self.component_to_child.read().await;
            mappings
                .get(component_id)
                .cloned()
                .ok_or_else(|| WasmError::component_not_found(component_id.as_str()))?
        };

        // Note: SupervisorNode starts children automatically when registered
        // This method is a no-op for already-registered components
        // In a full implementation, this would call a SupervisorNode start method if available

        Ok(())
    }

    async fn stop_component(
        &mut self,
        component_id: &ComponentId,
        _timeout: Duration,
    ) -> Result<(), WasmError> {
        // Look up ChildId
        let child_id = {
            let mappings = self.component_to_child.read().await;
            mappings
                .get(component_id)
                .cloned()
                .ok_or_else(|| WasmError::component_not_found(component_id.as_str()))?
        };

        // Stop via SupervisorNode
        let mut supervisor = self.supervisor.write().await;
        supervisor
            .stop_child(&RtChildId::from(
                uuid::Uuid::parse_str(&child_id)
                    .map_err(|e| WasmError::internal(format!("Invalid child ID: {}", e)))?,
            ))
            .await
            .map_err(|e| WasmError::internal(format!("Stop failed: {}", e)))?;

        // Remove from mappings
        {
            let mut comp_to_child = self.component_to_child.write().await;
            let mut child_to_comp = self.child_to_component.write().await;
            comp_to_child.remove(component_id);
            child_to_comp.remove(&child_id);
        }

        Ok(())
    }

    fn get_component_state(
        &self,
        component_id: &ComponentId,
    ) -> Option<ComponentSupervisionState> {
        // Non-async read for synchronous state query
        let mappings = self.component_to_child.blocking_read();
        let child_id = mappings.get(component_id)?;

        let supervisor = self.supervisor.blocking_read();

        // Parse child_id string to ChildId
        let child_id_uuid = uuid::Uuid::parse_str(child_id).ok()?;
        let rt_child_id = RtChildId::from(child_id_uuid);

        let child_handle = supervisor.get_child(&rt_child_id)?;
        let child_state = *child_handle.state();

        Some(Self::convert_child_state(child_state))
    }

    async fn start_all(&mut self) -> Result<(), WasmError> {
        // SupervisorNode doesn't have a start_all method in current implementation
        // Components are started automatically when registered
        Ok(())
    }

    async fn stop_all(&mut self, timeout: Duration) -> Result<(), WasmError> {
        // Collect all component IDs
        let component_ids: Vec<ComponentId> = {
            let mappings = self.component_to_child.read().await;
            mappings.keys().cloned().collect()
        };

        // Stop each component
        for component_id in component_ids {
            self.stop_component(&component_id, timeout).await?;
        }

        Ok(())
    }
}

impl Default for SupervisorNodeWrapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor::RestartPolicy as WasmRestartPolicy;

    #[test]
    fn test_restart_policy_conversion() {
        assert_eq!(
            SupervisorNodeWrapper::convert_restart_policy(WasmRestartPolicy::Permanent),
            RtRestartPolicy::Permanent
        );
        assert_eq!(
            SupervisorNodeWrapper::convert_restart_policy(WasmRestartPolicy::Transient),
            RtRestartPolicy::Transient
        );
        assert_eq!(
            SupervisorNodeWrapper::convert_restart_policy(WasmRestartPolicy::Temporary),
            RtRestartPolicy::Temporary
        );
    }

    #[test]
    fn test_child_state_conversion() {
        assert_eq!(
            SupervisorNodeWrapper::convert_child_state(RtChildState::Starting),
            ComponentSupervisionState::Starting
        );
        assert_eq!(
            SupervisorNodeWrapper::convert_child_state(RtChildState::Running),
            ComponentSupervisionState::Running
        );
        assert_eq!(
            SupervisorNodeWrapper::convert_child_state(RtChildState::Restarting),
            ComponentSupervisionState::Restarting
        );
        assert_eq!(
            SupervisorNodeWrapper::convert_child_state(RtChildState::Stopped),
            ComponentSupervisionState::Stopped
        );
        assert_eq!(
            SupervisorNodeWrapper::convert_child_state(RtChildState::Failed),
            ComponentSupervisionState::Failed
        );
    }

    #[tokio::test]
    async fn test_wrapper_creation() {
        let wrapper = SupervisorNodeWrapper::new();
        assert!(wrapper.component_to_child.read().await.is_empty());
        assert!(wrapper.child_to_component.read().await.is_empty());
    }

    #[tokio::test]
    async fn test_wrapper_default() {
        let wrapper = SupervisorNodeWrapper::default();
        assert!(wrapper.component_to_child.read().await.is_empty());
    }

    #[test]
    fn test_wrapper_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<SupervisorNodeWrapper>();
        assert_sync::<SupervisorNodeWrapper>();
    }
}
