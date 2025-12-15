//! Component supervision implementation for automatic restart management.
//!
//! This module implements ComponentSupervisor, which manages the lifecycle
//! and restart behavior of supervised ComponentActor instances.
//!
//! # Overview
//!
//! ComponentSupervisor tracks components under supervision and makes restart
//! decisions based on configured policies, backoff strategies, and restart limits.
//!
//! # Architecture
//!
//! The supervisor tracks:
//! - **Restart history**: Timeline of all restart attempts
//! - **Restart count**: Number of restarts in current time window
//! - **Supervision state**: Current state (Initializing, Running, Restarting, etc.)
//! - **Parent-child relationships**: For hierarchical supervision trees
//!
//! # Example
//!
//! ```rust,ignore
//! use airssys_wasm::actor::{ComponentSupervisor, SupervisorConfig, RestartPolicy};
//! use std::time::Duration;
//!
//! let supervisor = ComponentSupervisor::new(actor_system, registry);
//!
//! // Register a component under supervision
//! let config = SupervisorConfig::permanent()
//!     .with_max_restarts(3);
//! supervisor.supervise(component_id, spec, config)?;
//!
//! // Handle failure
//! let decision = supervisor.handle_component_failure(&component_id, "Connection timeout")?;
//! // Decision will indicate whether to restart and with what delay
//! ```

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

// Layer 3: Internal module imports
use crate::actor::{ComponentActor, SupervisorConfig, SupervisorNodeBridge};
use crate::core::{ComponentId, WasmError};

/// Handle to a supervised component with restart tracking.
///
/// Tracks all metadata needed to manage a component's lifecycle under supervision,
/// including restart history, current state, and configuration.
#[derive(Debug, Clone)]
pub struct SupervisionHandle {
    /// Component being supervised
    pub component_id: ComponentId,

    /// Parent component (for hierarchical supervision trees)
    pub parent_id: Option<ComponentId>,

    /// Number of restart attempts made
    pub restart_count: u32,

    /// Timestamps and error status of recent restart attempts
    /// Each tuple: (timestamp, is_error)
    pub restart_history: Vec<(DateTime<Utc>, bool)>,

    /// Supervision configuration
    pub config: SupervisorConfig,

    /// When this supervision was created
    pub created_at: DateTime<Utc>,

    /// When the component was last restarted (if ever)
    pub last_restart: Option<DateTime<Utc>>,

    /// Current state of the supervised component
    pub state: SupervisionState,
}

/// Supervision state machine for supervised components.
///
/// Represents the current lifecycle state of a supervised component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupervisionState {
    /// Component created, waiting to start
    Initializing,

    /// Component running normally
    Running,

    /// Component failed, scheduling restart
    SchedulingRestart,

    /// Component restart attempt in progress
    Restarting,

    /// Component stopped normally
    Stopped,

    /// Component hit restart limit, no more restarts
    RestartLimitExceeded,

    /// Component permanently failed (unrecoverable error)
    Terminated,
}

impl std::fmt::Display for SupervisionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupervisionState::Initializing => write!(f, "Initializing"),
            SupervisionState::Running => write!(f, "Running"),
            SupervisionState::SchedulingRestart => write!(f, "SchedulingRestart"),
            SupervisionState::Restarting => write!(f, "Restarting"),
            SupervisionState::Stopped => write!(f, "Stopped"),
            SupervisionState::RestartLimitExceeded => write!(f, "RestartLimitExceeded"),
            SupervisionState::Terminated => write!(f, "Terminated"),
        }
    }
}

/// Decision on whether to restart a component after failure.
///
/// Returned by restart decision methods to indicate whether the component
/// should be restarted, and if so, with what delay.
#[derive(Debug, Clone)]
pub enum RestartDecision {
    /// Component should restart with delay
    Scheduled {
        /// Duration to wait before restart attempt
        delay: Duration,
        /// Restart attempt number (1-based)
        attempt: u32,
    },

    /// Component should not restart (policy forbids)
    Denied(String),

    /// Component hit restart limit
    LimitExceeded(String),
}

/// Supervision statistics for all components under supervision.
#[derive(Debug, Clone)]
pub struct SupervisionStatistics {
    /// Total number of supervised components
    pub total_supervised: usize,
    /// Number currently in Running state
    pub currently_running: usize,
    /// Number in failed states (Terminated or RestartLimitExceeded)
    pub failed_components: usize,
    /// Total restart attempts across all components
    pub total_restart_attempts: u32,
}

/// Supervises ComponentActor instances with automatic restart.
///
/// Manages the supervision lifecycle for multiple components, tracking
/// restart history and making restart decisions based on configured policies.
///
/// # Architecture (ADR-WASM-018)
///
/// ComponentSupervisor operates at Layer 1 (WASM Configuration):
/// - **Tracks** restart history and policy decisions
/// - **Delegates** actual supervision execution to SupervisorNode (Layer 3) via bridge
/// - **Maintains** layer separation through SupervisorNodeBridge abstraction
pub struct ComponentSupervisor {
    /// Supervision handles for all supervised components
    supervision_handles: HashMap<ComponentId, SupervisionHandle>,

    /// Bridge to SupervisorNode (Layer 3) for supervision execution
    supervisor_bridge: Option<Arc<RwLock<dyn SupervisorNodeBridge>>>,
}

impl ComponentSupervisor {
    /// Create new ComponentSupervisor without bridge (policy tracking only).
    ///
    /// For full supervision with automatic restart, use `with_bridge()`.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let supervisor = ComponentSupervisor::new();
    /// ```
    pub fn new() -> Self {
        Self {
            supervision_handles: HashMap::new(),
            supervisor_bridge: None,
        }
    }

    /// Create new ComponentSupervisor with SupervisorNode bridge.
    ///
    /// This enables full supervision with automatic restart through Layer 3.
    ///
    /// # Parameters
    ///
    /// * `bridge` - Bridge to SupervisorNode for execution
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::SupervisorNodeWrapper;
    ///
    /// let bridge = Arc::new(RwLock::new(SupervisorNodeWrapper::new()));
    /// let supervisor = ComponentSupervisor::with_bridge(bridge);
    /// ```
    pub fn with_bridge(bridge: Arc<RwLock<dyn SupervisorNodeBridge>>) -> Self {
        Self {
            supervision_handles: HashMap::new(),
            supervisor_bridge: Some(bridge),
        }
    }

    /// Register a component under supervision.
    ///
    /// If a bridge is configured, this will also register the component with
    /// SupervisorNode for automatic restart execution.
    ///
    /// # Parameters
    ///
    /// * `component_id` - ID of component to supervise
    /// * `config` - Supervision configuration
    ///
    /// # Returns
    ///
    /// - `Ok(SupervisionHandle)` on success
    /// - `Err(WasmError)` if component already supervised
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let config = SupervisorConfig::permanent();
    /// supervisor.supervise(&component_id, config)?;
    /// ```
    pub fn supervise(
        &mut self,
        component_id: &ComponentId,
        config: SupervisorConfig,
    ) -> Result<SupervisionHandle, WasmError> {
        if self.supervision_handles.contains_key(component_id) {
            return Err(WasmError::internal(
                format!("Component {:?} already supervised", component_id),
            ));
        }

        let handle = SupervisionHandle {
            component_id: component_id.clone(),
            parent_id: None,
            restart_count: 0,
            restart_history: Vec::new(),
            config,
            created_at: Utc::now(),
            last_restart: None,
            state: SupervisionState::Initializing,
        };

        self.supervision_handles
            .insert(component_id.clone(), handle.clone());

        Ok(handle)
    }

    /// Register a component with both supervisor and bridge for full supervision.
    ///
    /// This method combines policy tracking (Layer 1) with execution registration
    /// (Layer 3 via bridge). Use this for complete supervision with automatic restart.
    ///
    /// # Parameters
    ///
    /// * `component_id` - ID of component to supervise
    /// * `actor` - ComponentActor instance to supervise
    /// * `config` - Supervision configuration
    ///
    /// # Returns
    ///
    /// - `Ok(SupervisionHandle)` on success
    /// - `Err(WasmError)` if component already supervised or bridge not configured
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let actor = ComponentActor::new(/* ... */)?;
    /// let config = SupervisorConfig::permanent();
    /// let handle = supervisor.supervise_with_actor(component_id, actor, config).await?;
    /// ```
    pub async fn supervise_with_actor(
        &mut self,
        component_id: ComponentId,
        actor: ComponentActor,
        config: SupervisorConfig,
    ) -> Result<SupervisionHandle, WasmError> {
        // Create local supervision handle
        let handle = self.supervise(&component_id, config.clone())?;

        // Register with SupervisorNode via bridge (if available)
        if let Some(bridge) = &self.supervisor_bridge {
            let mut bridge_guard = bridge.write().await;
            bridge_guard
                .register_component(component_id, actor, config)
                .await?;
        }

        Ok(handle)
    }

    /// Start a supervised component via the bridge.
    ///
    /// Delegates to SupervisorNode to start the component. Updates local state.
    ///
    /// # Parameters
    ///
    /// * `component_id` - ID of component to start
    ///
    /// # Returns
    ///
    /// - `Ok(())` on success
    /// - `Err(WasmError)` if bridge not configured or component not found
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// supervisor.start_component(&component_id).await?;
    /// ```
    pub async fn start_component(&mut self, component_id: &ComponentId) -> Result<(), WasmError> {
        // Update local state
        self.mark_running(component_id)?;

        // Start via bridge (if available)
        if let Some(bridge) = &self.supervisor_bridge {
            let mut bridge_guard = bridge.write().await;
            bridge_guard.start_component(component_id).await?;
        }

        Ok(())
    }

    /// Stop a supervised component via the bridge.
    ///
    /// Delegates to SupervisorNode to stop the component gracefully.
    ///
    /// # Parameters
    ///
    /// * `component_id` - ID of component to stop
    /// * `timeout` - Maximum time to wait for graceful shutdown
    ///
    /// # Returns
    ///
    /// - `Ok(())` on success
    /// - `Err(WasmError)` if bridge not configured or component not found
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// supervisor.stop_component(&component_id, Duration::from_secs(5)).await?;
    /// ```
    pub async fn stop_component(
        &mut self,
        component_id: &ComponentId,
        timeout: Duration,
    ) -> Result<(), WasmError> {
        // Stop via bridge (if available)
        if let Some(bridge) = &self.supervisor_bridge {
            let mut bridge_guard = bridge.write().await;
            bridge_guard.stop_component(component_id, timeout).await?;
        }

        // Update local state
        if let Some(handle) = self.get_handle_mut(component_id) {
            handle.state = SupervisionState::Stopped;
        }

        Ok(())
    }

    /// Query component supervision state from SupervisorNode.
    ///
    /// Returns the current lifecycle state as tracked by Layer 3 supervision.
    ///
    /// # Parameters
    ///
    /// * `component_id` - ID of component to query
    ///
    /// # Returns
    ///
    /// - `Some(ComponentSupervisionState)` if component supervised
    /// - `None` if component not found or bridge not configured
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if let Some(state) = supervisor.query_component_state(&component_id) {
    ///     println!("Component state: {:?}", state);
    /// }
    /// ```
    pub fn query_component_state(
        &self,
        component_id: &ComponentId,
    ) -> Option<crate::actor::ComponentSupervisionState> {
        let bridge = self.supervisor_bridge.as_ref()?;
        let bridge_guard = bridge.blocking_read();
        bridge_guard.get_component_state(component_id)
    }

    /// Start all supervised components.
    ///
    /// # Returns
    ///
    /// - `Ok(())` on success
    /// - `Err(WasmError)` if bridge not configured or any start fails
    pub async fn start_all(&mut self) -> Result<(), WasmError> {
        if let Some(bridge) = &self.supervisor_bridge {
            let mut bridge_guard = bridge.write().await;
            bridge_guard.start_all().await?;

            // Update local state for all components
            for handle in self.supervision_handles.values_mut() {
                handle.state = SupervisionState::Running;
            }
        }

        Ok(())
    }

    /// Stop all supervised components.
    ///
    /// # Parameters
    ///
    /// * `timeout` - Maximum time to wait for each component shutdown
    ///
    /// # Returns
    ///
    /// - `Ok(())` on success
    /// - `Err(WasmError)` if bridge not configured or any stop fails
    pub async fn stop_all(&mut self, timeout: Duration) -> Result<(), WasmError> {
        if let Some(bridge) = &self.supervisor_bridge {
            let mut bridge_guard = bridge.write().await;
            bridge_guard.stop_all(timeout).await?;

            // Update local state for all components
            for handle in self.supervision_handles.values_mut() {
                handle.state = SupervisionState::Stopped;
            }
        }

        Ok(())
    }

    /// Remove a component from supervision.
    ///
    /// # Parameters
    ///
    /// * `component_id` - ID of component to remove from supervision
    ///
    /// # Returns
    ///
    /// - `Ok(())` on success
    /// - `Err(WasmError::ComponentNotFound)` if component not supervised
    pub fn unsupervise(&mut self, component_id: &ComponentId) -> Result<(), WasmError> {
        self.supervision_handles.remove(component_id);
        Ok(())
    }

    /// Get supervision handle for component (immutable).
    ///
    /// # Parameters
    ///
    /// * `component_id` - ID of component
    ///
    /// # Returns
    ///
    /// - `Some(&SupervisionHandle)` if component supervised
    /// - `None` if component not found
    pub fn get_handle(&self, component_id: &ComponentId) -> Option<&SupervisionHandle> {
        self.supervision_handles.get(component_id)
    }

    /// Get supervision handle for component (mutable).
    ///
    /// # Parameters
    ///
    /// * `component_id` - ID of component
    ///
    /// # Returns
    ///
    /// - `Some(&mut SupervisionHandle)` if component supervised
    /// - `None` if component not found
    pub fn get_handle_mut(
        &mut self,
        component_id: &ComponentId,
    ) -> Option<&mut SupervisionHandle> {
        self.supervision_handles.get_mut(component_id)
    }

    /// Record component failure and determine restart decision.
    ///
    /// Updates restart history and makes a restart decision based on the
    /// component's supervision configuration.
    ///
    /// # Parameters
    ///
    /// * `component_id` - ID of failed component
    /// * `error` - Error message for logging
    ///
    /// # Returns
    ///
    /// - `Ok(RestartDecision::Scheduled { .. })` if restart should happen
    /// - `Ok(RestartDecision::Denied(_))` if restart policy forbids
    /// - `Ok(RestartDecision::LimitExceeded(_))` if max restarts exceeded
    /// - `Err(WasmError::ComponentNotFound)` if component not supervised
    pub async fn handle_component_failure(
        &mut self,
        component_id: &ComponentId,
        _error: &str,
    ) -> Result<RestartDecision, WasmError> {
        let handle = self
            .get_handle_mut(component_id)
            .ok_or_else(|| WasmError::component_not_found(format!("{:?}", component_id)))?;

        handle.restart_history.push((Utc::now(), true));
        handle.restart_count += 1;

        // Check restart policy
        if !handle.config.restart_policy.should_restart(true) {
            handle.state = SupervisionState::Stopped;
            return Ok(RestartDecision::Denied(
                "Restart policy forbids restart".to_string(),
            ));
        }

        // Check restart limit
        if handle.config.check_restart_limit(&handle.restart_history) {
            handle.state = SupervisionState::RestartLimitExceeded;
            return Ok(RestartDecision::LimitExceeded(format!(
                "Max restarts {} exceeded in {:?}",
                handle.config.max_restarts, handle.config.time_window
            )));
        }

        // Calculate delay
        let delay = handle
            .config
            .calculate_next_restart_delay(handle.restart_count - 1);

        handle.state = SupervisionState::SchedulingRestart;

        Ok(RestartDecision::Scheduled {
            delay,
            attempt: handle.restart_count,
        })
    }

    /// Record component normal exit and determine restart decision.
    ///
    /// Updates restart history for normal exit and makes a restart decision.
    ///
    /// # Parameters
    ///
    /// * `component_id` - ID of component that exited
    ///
    /// # Returns
    ///
    /// - `Ok(RestartDecision::Scheduled { .. })` if restart should happen
    /// - `Ok(RestartDecision::Denied(_))` if restart policy forbids
    /// - `Err(WasmError::ComponentNotFound)` if component not supervised
    pub async fn handle_component_exit(
        &mut self,
        component_id: &ComponentId,
    ) -> Result<RestartDecision, WasmError> {
        let handle = self
            .get_handle_mut(component_id)
            .ok_or_else(|| WasmError::component_not_found(format!("{:?}", component_id)))?;

        handle.restart_history.push((Utc::now(), false));

        // Check restart policy for normal exit
        if !handle.config.restart_policy.should_restart(false) {
            handle.state = SupervisionState::Stopped;
            return Ok(RestartDecision::Denied(
                "Normal exit, restart policy forbids restart".to_string(),
            ));
        }

        let delay = handle
            .config
            .calculate_next_restart_delay(handle.restart_count);
        handle.state = SupervisionState::SchedulingRestart;

        Ok(RestartDecision::Scheduled {
            delay,
            attempt: handle.restart_count,
        })
    }

    /// Record successful component start.
    ///
    /// Marks component as running after successful startup.
    ///
    /// # Parameters
    ///
    /// * `component_id` - ID of component that started
    pub fn mark_running(&mut self, component_id: &ComponentId) -> Result<(), WasmError> {
        let handle = self
            .get_handle_mut(component_id)
            .ok_or_else(|| WasmError::component_not_found(format!("{:?}", component_id)))?;
        handle.state = SupervisionState::Running;
        Ok(())
    }

    /// Record restart attempt start.
    ///
    /// Marks component as restarting.
    ///
    /// # Parameters
    ///
    /// * `component_id` - ID of component being restarted
    pub fn mark_restarting(&mut self, component_id: &ComponentId) -> Result<(), WasmError> {
        let handle = self
            .get_handle_mut(component_id)
            .ok_or_else(|| WasmError::component_not_found(format!("{:?}", component_id)))?;
        handle.state = SupervisionState::Restarting;
        handle.last_restart = Some(Utc::now());
        Ok(())
    }

    /// Get all supervision handles.
    ///
    /// # Returns
    ///
    /// Vector of all SupervisionHandle objects
    pub fn get_all_handles(&self) -> Vec<SupervisionHandle> {
        self.supervision_handles.values().cloned().collect()
    }

    /// Get supervision statistics.
    ///
    /// # Returns
    ///
    /// SupervisionStatistics with counts for all states
    pub fn get_statistics(&self) -> SupervisionStatistics {
        let total = self.supervision_handles.len();
        let running = self
            .supervision_handles
            .iter()
            .filter(|(_, h)| h.state == SupervisionState::Running)
            .count();
        let failed = self
            .supervision_handles
            .iter()
            .filter(|(_, h)| {
                h.state == SupervisionState::Terminated
                    || h.state == SupervisionState::RestartLimitExceeded
            })
            .count();

        SupervisionStatistics {
            total_supervised: total,
            currently_running: running,
            failed_components: failed,
            total_restart_attempts: self
                .supervision_handles
                .values()
                .map(|h| h.restart_count)
                .sum(),
        }
    }

    /// Get restart statistics for a supervised component.
    ///
    /// Queries the SupervisorNode bridge for detailed restart metrics including
    /// total count, recent rate, and last restart time.
    ///
    /// # Parameters
    ///
    /// * `component_id` - Component to query
    ///
    /// # Returns
    ///
    /// Restart statistics if component is supervised and bridge is available, None otherwise
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if let Some(stats) = supervisor.get_restart_stats(&component_id) {
    ///     println!("Total restarts: {}", stats.total_restarts);
    ///     println!("Recent rate: {:.2} restarts/sec", stats.recent_rate);
    /// }
    /// ```
    pub fn get_restart_stats(
        &self,
        component_id: &ComponentId,
    ) -> Option<crate::actor::supervisor_wrapper::RestartStats> {
        // Check if component is supervised
        if !self.supervision_handles.contains_key(component_id) {
            return None;
        }

        // Query bridge if available
        let bridge = self.supervisor_bridge.as_ref()?;
        
        
        bridge.blocking_read().get_restart_stats(component_id)
    }

    /// Reset restart tracking for a supervised component.
    ///
    /// Clears restart history and counters for the component. Useful after
    /// successful recovery or manual intervention.
    ///
    /// # Parameters
    ///
    /// * `component_id` - Component to reset
    ///
    /// # Returns
    ///
    /// - `Ok(())` on success
    /// - `Err(WasmError::ComponentNotFound)` if component not supervised
    /// - `Err(WasmError::BridgeNotConfigured)` if bridge not available
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// supervisor.reset_restart_tracking(&component_id)?;
    /// println!("Restart tracking reset for component");
    /// ```
    pub fn reset_restart_tracking(&mut self, component_id: &ComponentId) -> Result<(), WasmError> {
        // Check if component is supervised
        if !self.supervision_handles.contains_key(component_id) {
            return Err(WasmError::component_not_found(format!("{:?}", component_id)));
        }

        // Reset local tracking
        if let Some(handle) = self.supervision_handles.get_mut(component_id) {
            handle.restart_count = 0;
            handle.restart_history.clear();
            handle.last_restart = None;
        }

        // Reset bridge tracking
        if let Some(bridge) = &mut self.supervisor_bridge {
            bridge.blocking_write().reset_restart_tracking(component_id);
        }

        Ok(())
    }

    /// Set parent-child relationship in supervision tree.
    ///
    /// Builds hierarchy for advanced supervision tree management.
    ///
    /// # Parameters
    ///
    /// * `child_id` - ID of child component
    /// * `parent_id` - ID of parent component
    ///
    /// # Returns
    ///
    /// - `Ok(())` on success
    /// - `Err(WasmError::ComponentNotFound)` if child not supervised
    pub fn set_parent(
        &mut self,
        child_id: &ComponentId,
        parent_id: ComponentId,
    ) -> Result<(), WasmError> {
        let child = self
            .get_handle_mut(child_id)
            .ok_or_else(|| WasmError::component_not_found(format!("{:?}", child_id)))?;
        child.parent_id = Some(parent_id);
        Ok(())
    }

    /// Get all children of a supervisor.
    ///
    /// # Parameters
    ///
    /// * `parent_id` - ID of parent component
    ///
    /// # Returns
    ///
    /// Vector of child component IDs
    pub fn get_children(&self, parent_id: &ComponentId) -> Vec<ComponentId> {
        self.supervision_handles
            .iter()
            .filter(|(_, handle)| handle.parent_id.as_ref() == Some(parent_id))
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get all ancestors of a component in the supervision tree.
    ///
    /// # Parameters
    ///
    /// * `component_id` - ID of component
    ///
    /// # Returns
    ///
    /// Vector of ancestor component IDs (from immediate parent to root)
    pub fn get_ancestors(&self, component_id: &ComponentId) -> Vec<ComponentId> {
        let mut ancestors = Vec::new();
        let mut current = component_id.clone();

        while let Some(handle) = self.supervision_handles.get(&current) {
            match &handle.parent_id {
                Some(parent) => {
                    ancestors.push(parent.clone());
                    current = parent.clone();
                }
                None => break,
            }
        }

        ancestors
    }

    /// Get supervision tree as hierarchical structure.
    ///
    /// # Returns
    ///
    /// SupervisionTree with hierarchical representation
    pub fn get_tree_structure(&self) -> SupervisionTree {
        let root_nodes: Vec<_> = self
            .supervision_handles
            .iter()
            .filter(|(_, h)| h.parent_id.is_none())
            .map(|(id, _)| id.clone())
            .collect();

        let mut tree_nodes = HashMap::new();

        for root_id in root_nodes {
            let node = self.build_tree_node(&root_id);
            tree_nodes.insert(root_id, node);
        }

        SupervisionTree { nodes: tree_nodes }
    }

    fn build_tree_node(&self, node_id: &ComponentId) -> SupervisionTreeNode {
        let children = self.get_children(node_id);
        let child_nodes = children
            .into_iter()
            .map(|child_id| self.build_tree_node(&child_id))
            .collect();

        let handle = self.supervision_handles.get(node_id).cloned();

        SupervisionTreeNode {
            component_id: node_id.clone(),
            handle,
            children: child_nodes,
        }
    }

    /// Print supervision tree for debugging.
    pub fn print_tree(&self) {
        let tree = self.get_tree_structure();
        tree.print();
    }
}

impl Default for ComponentSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

/// Hierarchical supervision tree representation.
#[derive(Debug, Clone)]
pub struct SupervisionTree {
    /// Root nodes of the supervision tree
    nodes: HashMap<ComponentId, SupervisionTreeNode>,
}

impl SupervisionTree {
    /// Print the entire supervision tree.
    pub fn print(&self) {
        for node in self.nodes.values() {
            Self::print_node(node, 0);
        }
    }

    fn print_node(node: &SupervisionTreeNode, depth: usize) {
        let indent = "  ".repeat(depth);
        let state = node
            .handle
            .as_ref()
            .map(|h| format!("{}", h.state))
            .unwrap_or_else(|| "Unknown".to_string());
        println!("{}├─ {:?} [{}]", indent, node.component_id, state);

        for child in &node.children {
            Self::print_node(child, depth + 1);
        }
    }
}

/// Node in a hierarchical supervision tree.
#[derive(Debug, Clone)]
pub struct SupervisionTreeNode {
    /// ID of this component
    pub component_id: ComponentId,
    /// Supervision handle for this component
    pub handle: Option<SupervisionHandle>,
    /// Child nodes in the tree
    pub children: Vec<SupervisionTreeNode>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supervision_handle_creation() {
        let config = SupervisorConfig::default();
        let handle = SupervisionHandle {
            component_id: ComponentId::new("test-component"),
            parent_id: None,
            restart_count: 0,
            restart_history: Vec::new(),
            config: config.clone(),
            created_at: Utc::now(),
            last_restart: None,
            state: SupervisionState::Initializing,
        };

        assert_eq!(handle.restart_count, 0);
        assert_eq!(handle.state, SupervisionState::Initializing);
        assert!(handle.parent_id.is_none());
    }

    #[test]
    fn test_supervision_state_transitions() {
        let mut handle = SupervisionHandle {
            component_id: ComponentId::new("test-component"),
            parent_id: None,
            restart_count: 0,
            restart_history: Vec::new(),
            config: SupervisorConfig::default(),
            created_at: Utc::now(),
            last_restart: None,
            state: SupervisionState::Initializing,
        };

        // Transition to Running
        handle.state = SupervisionState::Running;
        assert_eq!(handle.state, SupervisionState::Running);

        // Transition to SchedulingRestart
        handle.state = SupervisionState::SchedulingRestart;
        assert_eq!(handle.state, SupervisionState::SchedulingRestart);

        // Transition to Stopped
        handle.state = SupervisionState::Stopped;
        assert_eq!(handle.state, SupervisionState::Stopped);
    }

    #[test]
    fn test_supervision_state_display() {
        assert_eq!(SupervisionState::Initializing.to_string(), "Initializing");
        assert_eq!(SupervisionState::Running.to_string(), "Running");
        assert_eq!(SupervisionState::SchedulingRestart.to_string(), "SchedulingRestart");
        assert_eq!(SupervisionState::Restarting.to_string(), "Restarting");
        assert_eq!(SupervisionState::Stopped.to_string(), "Stopped");
        assert_eq!(SupervisionState::RestartLimitExceeded.to_string(), "RestartLimitExceeded");
        assert_eq!(SupervisionState::Terminated.to_string(), "Terminated");
    }

    #[test]
    fn test_component_supervisor_new() {
        let supervisor = ComponentSupervisor::new();
        assert_eq!(supervisor.get_all_handles().len(), 0);
    }

    #[test]
    fn test_component_supervisor_supervise() {
        let mut supervisor = ComponentSupervisor::new();
        let component_id = ComponentId::new("test-component");
        let config = SupervisorConfig::permanent();

        let result = supervisor.supervise(&component_id, config);
        assert!(result.is_ok());

        let handle = supervisor.get_handle(&component_id);
        assert!(handle.is_some());
    }

    #[test]
    fn test_component_supervisor_supervise_duplicate() {
        let mut supervisor = ComponentSupervisor::new();
        let component_id = ComponentId::new("test-component");
        let config = SupervisorConfig::permanent();

        supervisor.supervise(&component_id, config.clone()).ok();

        // Try to supervise again - should fail
        let result = supervisor.supervise(&component_id, config);
        assert!(result.is_err());
    }

    #[test]
    fn test_component_supervisor_unsupervise() {
        let mut supervisor = ComponentSupervisor::new();
        let component_id = ComponentId::new("test-component");
        let config = SupervisorConfig::permanent();

        supervisor.supervise(&component_id, config).ok();
        assert!(supervisor.get_handle(&component_id).is_some());

        supervisor.unsupervise(&component_id).ok();
        assert!(supervisor.get_handle(&component_id).is_none());
    }

    #[test]
    fn test_get_statistics_empty() {
        let supervisor = ComponentSupervisor::new();
        let stats = supervisor.get_statistics();

        assert_eq!(stats.total_supervised, 0);
        assert_eq!(stats.currently_running, 0);
        assert_eq!(stats.failed_components, 0);
        assert_eq!(stats.total_restart_attempts, 0);
    }

    #[test]
    fn test_get_statistics_with_components() {
        let mut supervisor = ComponentSupervisor::new();

        // Add 3 components
        for i in 0..3 {
            let component_id = ComponentId::new(format!("component-{}", i));
            let config = SupervisorConfig::permanent();
            supervisor.supervise(&component_id, config).ok();
        }

        let stats = supervisor.get_statistics();
        assert_eq!(stats.total_supervised, 3);
        assert_eq!(stats.currently_running, 0); // All in Initializing state
    }

    #[test]
    fn test_mark_running() {
        let mut supervisor = ComponentSupervisor::new();
        let component_id = ComponentId::new("test-component");
        let config = SupervisorConfig::permanent();

        supervisor.supervise(&component_id, config).ok();
        supervisor.mark_running(&component_id).ok();

        let handle = supervisor.get_handle(&component_id);
        assert!(handle.is_some());
        if let Some(h) = handle {
            assert_eq!(h.state, SupervisionState::Running);
        }
    }

    #[test]
    fn test_mark_restarting() {
        let mut supervisor = ComponentSupervisor::new();
        let component_id = ComponentId::new("test-component");
        let config = SupervisorConfig::permanent();

        supervisor.supervise(&component_id, config).ok();
        supervisor.mark_restarting(&component_id).ok();

        let handle = supervisor.get_handle(&component_id);
        assert!(handle.is_some());
        if let Some(h) = handle {
            assert_eq!(h.state, SupervisionState::Restarting);
            assert!(h.last_restart.is_some());
        }
    }

    #[test]
    fn test_set_parent_child_relationship() {
        let mut supervisor = ComponentSupervisor::new();

        let parent_id = ComponentId::new("parent");
        let child_id = ComponentId::new("child");

        supervisor.supervise(&parent_id, SupervisorConfig::default()).ok();
        supervisor.supervise(&child_id, SupervisorConfig::default()).ok();

        supervisor.set_parent(&child_id, parent_id.clone()).ok();

        let child_handle = supervisor.get_handle(&child_id);
        assert!(child_handle.is_some());
        if let Some(h) = child_handle {
            assert_eq!(h.parent_id.as_ref(), Some(&parent_id));
        }
    }

    #[test]
    fn test_get_children() {
        let mut supervisor = ComponentSupervisor::new();

        let parent_id = ComponentId::new("parent");
        let child1_id = ComponentId::new("child1");
        let child2_id = ComponentId::new("child2");

        supervisor.supervise(&parent_id, SupervisorConfig::default()).ok();
        supervisor.supervise(&child1_id, SupervisorConfig::default()).ok();
        supervisor.supervise(&child2_id, SupervisorConfig::default()).ok();

        supervisor.set_parent(&child1_id, parent_id.clone()).ok();
        supervisor.set_parent(&child2_id, parent_id.clone()).ok();

        let children = supervisor.get_children(&parent_id);
        assert_eq!(children.len(), 2);
        assert!(children.contains(&child1_id));
        assert!(children.contains(&child2_id));
    }

    #[test]
    fn test_get_ancestors() {
        let mut supervisor = ComponentSupervisor::new();

        let root_id = ComponentId::new("root");
        let parent_id = ComponentId::new("parent");
        let child_id = ComponentId::new("child");

        supervisor.supervise(&root_id, SupervisorConfig::default()).ok();
        supervisor.supervise(&parent_id, SupervisorConfig::default()).ok();
        supervisor.supervise(&child_id, SupervisorConfig::default()).ok();

        supervisor.set_parent(&parent_id, root_id.clone()).ok();
        supervisor.set_parent(&child_id, parent_id.clone()).ok();

        let ancestors = supervisor.get_ancestors(&child_id);
        assert_eq!(ancestors.len(), 2);
        assert_eq!(ancestors[0], parent_id);
        assert_eq!(ancestors[1], root_id);
    }

    // ========================================================================
    // STEP 3.2.3: Bridge Integration Tests
    // ========================================================================

    #[test]
    fn test_component_supervisor_with_bridge() {
        use crate::actor::SupervisorNodeWrapper;

        let bridge = Arc::new(RwLock::new(SupervisorNodeWrapper::new()));
        let supervisor = ComponentSupervisor::with_bridge(bridge);

        assert!(supervisor.supervisor_bridge.is_some());
        assert_eq!(supervisor.get_all_handles().len(), 0);
    }

    #[test]
    fn test_component_supervisor_without_bridge() {
        let supervisor = ComponentSupervisor::new();
        assert!(supervisor.supervisor_bridge.is_none());
    }

    #[test]
    fn test_query_component_state_without_bridge() {
        let supervisor = ComponentSupervisor::new();
        let component_id = ComponentId::new("test-component");

        // Should return None when no bridge configured
        let state = supervisor.query_component_state(&component_id);
        assert!(state.is_none());
    }

    #[tokio::test]
    async fn test_supervise_basic_without_actor() {
        let mut supervisor = ComponentSupervisor::new();
        let component_id = ComponentId::new("test-component");
        let config = SupervisorConfig::permanent();

        // Basic supervise without actor (policy tracking only)
        let result = supervisor.supervise(&component_id, config);
        assert!(result.is_ok());

        let handle = supervisor.get_handle(&component_id);
        assert!(handle.is_some());
    }

    #[tokio::test]
    async fn test_start_component_without_bridge() {
        let mut supervisor = ComponentSupervisor::new();
        let component_id = ComponentId::new("test-component");
        let config = SupervisorConfig::permanent();

        supervisor.supervise(&component_id, config).ok();

        // Start should succeed even without bridge (updates local state only)
        let result = supervisor.start_component(&component_id).await;
        assert!(result.is_ok());

        let handle = supervisor.get_handle(&component_id);
        assert!(handle.is_some());
        if let Some(h) = handle {
            assert_eq!(h.state, SupervisionState::Running);
        }
    }

    #[tokio::test]
    async fn test_stop_component_without_bridge() {
        let mut supervisor = ComponentSupervisor::new();
        let component_id = ComponentId::new("test-component");
        let config = SupervisorConfig::permanent();

        supervisor.supervise(&component_id, config).ok();
        supervisor.mark_running(&component_id).ok();

        // Stop should succeed even without bridge (updates local state only)
        let result = supervisor.stop_component(&component_id, Duration::from_secs(5)).await;
        assert!(result.is_ok());

        let handle = supervisor.get_handle(&component_id);
        assert!(handle.is_some());
        if let Some(h) = handle {
            assert_eq!(h.state, SupervisionState::Stopped);
        }
    }
}
