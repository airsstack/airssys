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

use crate::core::{ComponentId, WasmError};
use crate::actor::SupervisorConfig;
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use std::time::Duration;

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
pub struct ComponentSupervisor {
    /// Supervision handles for all supervised components
    supervision_handles: HashMap<ComponentId, SupervisionHandle>,
}

impl ComponentSupervisor {
    /// Create new ComponentSupervisor.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let supervisor = ComponentSupervisor::new();
    /// ```
    pub fn new() -> Self {
        Self {
            supervision_handles: HashMap::new(),
        }
    }

    /// Register a component under supervision.
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
    /// supervisor.supervise(component_id, config)?;
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
}
