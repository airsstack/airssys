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
use crate::actor::component::ComponentActor;
use crate::actor::health::HealthMonitor;
use crate::actor::supervisor::{
    ComponentSupervisionState, ExponentialBackoff, ExponentialBackoffConfig, RestartPolicy,
    RestartRecord, RestartTracker, SlidingWindowConfig, SlidingWindowLimiter, SupervisorConfig,
    SupervisorNodeBridge,
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
    supervisor:
        Arc<RwLock<SupervisorNode<OneForOne, ComponentActor, NoopMonitor<SupervisionEvent>>>>,

    /// Mapping: ComponentId → ChildId (for SupervisorNode)
    component_to_child: Arc<RwLock<HashMap<ComponentId, String>>>,

    /// Reverse mapping: ChildId → ComponentId
    child_to_component: Arc<RwLock<HashMap<String, ComponentId>>>,

    /// Per-component exponential backoff trackers
    backoff_trackers: Arc<RwLock<HashMap<ComponentId, ExponentialBackoff>>>,

    /// Per-component restart history trackers
    restart_trackers: Arc<RwLock<HashMap<ComponentId, RestartTracker>>>,

    /// Per-component sliding window limiters
    window_limiters: Arc<RwLock<HashMap<ComponentId, SlidingWindowLimiter>>>,

    /// Per-component health monitors
    health_monitors: Arc<RwLock<HashMap<ComponentId, HealthMonitor>>>,
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
            backoff_trackers: Arc::new(RwLock::new(HashMap::new())),
            restart_trackers: Arc::new(RwLock::new(HashMap::new())),
            window_limiters: Arc::new(RwLock::new(HashMap::new())),
            health_monitors: Arc::new(RwLock::new(HashMap::new())),
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
            backoff_trackers: Arc::new(RwLock::new(HashMap::new())),
            restart_trackers: Arc::new(RwLock::new(HashMap::new())),
            window_limiters: Arc::new(RwLock::new(HashMap::new())),
            health_monitors: Arc::new(RwLock::new(HashMap::new())),
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

        // Initialize restart & backoff tracking systems
        {
            // Convert BackoffStrategy to ExponentialBackoffConfig
            let backoff_config = match &config.backoff_strategy {
                crate::actor::BackoffStrategy::Immediate => ExponentialBackoffConfig {
                    base_delay: Duration::ZERO,
                    max_delay: Duration::ZERO,
                    multiplier: 1.0,
                    jitter_factor: 0.0,
                },
                crate::actor::BackoffStrategy::Linear { base_delay } => ExponentialBackoffConfig {
                    base_delay: *base_delay,
                    max_delay: *base_delay * 10, // Cap at 10x base for linear
                    multiplier: 1.0,             // Linear = multiplier of 1.0
                    jitter_factor: 0.1,          // 10% jitter
                },
                crate::actor::BackoffStrategy::Exponential {
                    base_delay,
                    multiplier,
                    max_delay,
                } => ExponentialBackoffConfig {
                    base_delay: *base_delay,
                    max_delay: *max_delay,
                    multiplier: *multiplier as f64,
                    jitter_factor: 0.1, // 10% jitter
                },
            };

            // Create tracking systems
            let backoff = ExponentialBackoff::new(backoff_config);
            let tracker = RestartTracker::new();
            let window_config = SlidingWindowConfig {
                max_restarts: config.max_restarts,
                window_duration: config.time_window,
            };
            let limiter = SlidingWindowLimiter::new(window_config);
            let health_monitor = HealthMonitor::new(Duration::from_secs(10)); // Default 10s check interval

            // Store in wrapper
            let mut backoff_trackers = self.backoff_trackers.write().await;
            let mut restart_trackers = self.restart_trackers.write().await;
            let mut window_limiters = self.window_limiters.write().await;
            let mut health_monitors = self.health_monitors.write().await;

            backoff_trackers.insert(component_id.clone(), backoff);
            restart_trackers.insert(component_id.clone(), tracker);
            window_limiters.insert(component_id.clone(), limiter);
            health_monitors.insert(component_id.clone(), health_monitor);
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
                    unreachable!(
                        "Factory called multiple times - ComponentActor can only be created once"
                    )
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
            .stop_child(&RtChildId::from(uuid::Uuid::parse_str(&child_id).map_err(
                |e| WasmError::internal(format!("Invalid child ID: {}", e)),
            )?))
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

    fn get_component_state(&self, component_id: &ComponentId) -> Option<ComponentSupervisionState> {
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

    fn get_restart_stats(&self, component_id: &ComponentId) -> Option<crate::actor::RestartStats> {
        let trackers = self.restart_trackers.blocking_read();
        let tracker = trackers.get(component_id)?;

        let history = tracker.get_history(10); // Last 10 restarts
        let last_restart = history.first().map(|r| r.timestamp);

        Some(RestartStats {
            total_restarts: tracker.total_restarts(),
            recent_rate: tracker.recent_restart_rate(Duration::from_secs(60)),
            last_restart,
        })
    }

    fn reset_restart_tracking(&mut self, component_id: &ComponentId) {
        // Reset all tracking systems for this component
        if let Some(backoff) = self.backoff_trackers.blocking_write().get_mut(component_id) {
            backoff.reset();
        }
        if let Some(tracker) = self.restart_trackers.blocking_write().get_mut(component_id) {
            tracker.reset_on_recovery();
        }
        if let Some(limiter) = self.window_limiters.blocking_write().get_mut(component_id) {
            limiter.reset();
        }
        if let Some(monitor) = self.health_monitors.blocking_write().get_mut(component_id) {
            monitor.reset_on_recovery();
        }
    }

    fn query_restart_history(
        &self,
        component_id: &ComponentId,
        limit: usize,
    ) -> Vec<crate::actor::RestartRecord> {
        let trackers = self.restart_trackers.blocking_read();
        trackers
            .get(component_id)
            .map(|t| t.get_history(limit))
            .unwrap_or_default()
    }
}

impl Default for SupervisorNodeWrapper {
    fn default() -> Self {
        Self::new()
    }
}

// Additional helper methods for restart statistics
impl SupervisorNodeWrapper {
    /// Get restart statistics for a component.
    ///
    /// Returns comprehensive restart data including total count, recent rate,
    /// and last restart time.
    ///
    /// # Parameters
    ///
    /// - `component_id`: Component to query
    ///
    /// # Returns
    ///
    /// Restart statistics if component is supervised, None otherwise
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let stats = wrapper.get_restart_stats(&component_id);
    /// if let Some(stats) = stats {
    ///     println!("Total restarts: {}", stats.total_restarts);
    ///     println!("Recent rate: {} restarts/sec", stats.recent_rate);
    /// }
    /// ```
    pub fn get_restart_stats(&self, component_id: &ComponentId) -> Option<RestartStats> {
        let trackers = self.restart_trackers.blocking_read();
        let tracker = trackers.get(component_id)?;

        let history = tracker.get_history(10); // Last 10 restarts
        let last_restart = history.first().map(|r| r.timestamp);

        Some(RestartStats {
            total_restarts: tracker.total_restarts(),
            recent_rate: tracker.recent_restart_rate(Duration::from_secs(60)),
            last_restart,
        })
    }

    /// Reset restart tracking for a component.
    ///
    /// Clears restart history and resets counters. Useful after successful
    /// recovery or manual intervention.
    ///
    /// # Parameters
    ///
    /// - `component_id`: Component to reset
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// wrapper.reset_restart_tracking(&component_id);
    /// ```
    pub fn reset_restart_tracking(&mut self, component_id: &ComponentId) {
        // Reset all tracking systems for this component
        if let Some(backoff) = self.backoff_trackers.blocking_write().get_mut(component_id) {
            backoff.reset();
        }
        if let Some(tracker) = self.restart_trackers.blocking_write().get_mut(component_id) {
            tracker.reset_on_recovery();
        }
        if let Some(limiter) = self.window_limiters.blocking_write().get_mut(component_id) {
            limiter.reset();
        }
        if let Some(monitor) = self.health_monitors.blocking_write().get_mut(component_id) {
            monitor.reset_on_recovery();
        }
    }

    /// Query restart history for a component.
    ///
    /// Returns up to `limit` most recent restart records.
    ///
    /// # Parameters
    ///
    /// - `component_id`: Component to query
    /// - `limit`: Maximum number of records to return
    ///
    /// # Returns
    ///
    /// Vector of restart records (newest first), empty if component not found
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let history = wrapper.query_restart_history(&component_id, 5);
    /// for record in history {
    ///     println!("Restart at {:?}: {:?}", record.timestamp, record.reason);
    /// }
    /// ```
    pub fn query_restart_history(
        &self,
        component_id: &ComponentId,
        limit: usize,
    ) -> Vec<RestartRecord> {
        let trackers = self.restart_trackers.blocking_read();
        trackers
            .get(component_id)
            .map(|t| t.get_history(limit))
            .unwrap_or_default()
    }
}

/// Restart statistics for a supervised component.
///
/// Provides aggregated metrics about component restart behavior.
#[derive(Debug, Clone)]
pub struct RestartStats {
    /// Total number of restarts since component was registered
    pub total_restarts: u32,

    /// Recent restart rate (restarts per second in last 60s window)
    pub recent_rate: f64,

    /// Timestamp of most recent restart (if any)
    pub last_restart: Option<std::time::Instant>,
}

#[allow(clippy::expect_used, clippy::unwrap_used, clippy::unwrap_err_used, clippy::expect_err_used, clippy::panic, clippy::unwrap_on_result, clippy::indexing_slicing, clippy::too_many_arguments, clippy::type_complexity, reason = "test code")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor::supervisor::RestartPolicy as WasmRestartPolicy;

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
