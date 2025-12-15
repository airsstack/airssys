//! Bridge trait for integrating ComponentSupervisor with airssys-rt SupervisorNode.
//!
//! This module defines the abstraction layer that connects Layer 1 (WASM configuration)
//! with Layer 3 (actor system supervision), maintaining clear ownership boundaries
//! per ADR-WASM-018.
//!
//! # Architecture Context (ADR-WASM-018)
//!
//! Per the Three-Layer Architecture:
//! - **Layer 1** (WASM Config): ComponentSupervisor uses this trait
//! - **Layer 3** (Actor Runtime): SupervisorNodeWrapper implements this trait
//! - **Boundary**: This trait is the integration point
//!
//! # Responsibilities
//!
//! The bridge handles:
//! - Registering ComponentActor instances for supervision
//! - Starting supervised components
//! - Stopping supervised components
//! - Querying supervision state
//!
//! # Performance
//!
//! Bridge operations should add <10μs overhead vs. direct SupervisorNode calls.
//!
//! # Example
//!
//! ```rust,ignore
//! use airssys_wasm::actor::{SupervisorNodeBridge, ComponentActor, SupervisorConfig};
//! use airssys_wasm::core::ComponentId;
//!
//! async fn supervise_component(
//!     bridge: &mut dyn SupervisorNodeBridge,
//!     component_id: ComponentId,
//!     actor: ComponentActor,
//!     config: SupervisorConfig,
//! ) -> Result<(), WasmError> {
//!     // Register component for supervision
//!     bridge.register_component(component_id.clone(), actor, config).await?;
//!     
//!     // Start the supervised component
//!     bridge.start_component(&component_id).await?;
//!     
//!     Ok(())
//! }
//! ```

// Layer 1: Standard library imports
use std::time::Duration;

// Layer 2: Third-party crate imports
use async_trait::async_trait;

// Layer 3: Internal module imports
use crate::actor::{ComponentActor, SupervisorConfig};
use crate::core::{ComponentId, WasmError};

/// Bridge trait for SupervisorNode integration.
///
/// This trait abstracts the airssys-rt SupervisorNode operations, allowing
/// ComponentSupervisor to coordinate with the supervision infrastructure without
/// directly depending on Layer 3 implementation details.
///
/// # Architecture Context
///
/// Per ADR-WASM-018:
/// - **Layer 1** (WASM Config): ComponentSupervisor uses this trait
/// - **Layer 3** (Actor Runtime): SupervisorNodeWrapper implements this trait
/// - **Boundary**: This trait is the integration point
///
/// # Responsibilities
///
/// The bridge handles:
/// - Registering ComponentActor instances for supervision
/// - Starting supervised components
/// - Stopping supervised components
/// - Querying supervision state
///
/// # Performance
///
/// Bridge operations should add <10μs overhead vs. direct SupervisorNode calls.
#[async_trait]
pub trait SupervisorNodeBridge: Send + Sync {
    /// Register a ComponentActor for supervision.
    ///
    /// Creates a ChildSpec and adds the component to the SupervisorNode.
    /// The SupervisorNode will automatically restart the component according
    /// to the configured RestartPolicy.
    ///
    /// # Parameters
    ///
    /// - `component_id`: Unique identifier for tracking
    /// - `actor`: ComponentActor instance to supervise
    /// - `config`: Supervision configuration (restart policy, backoff, etc.)
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Component registered successfully
    /// - `Err(WasmError)`: Registration failed (e.g., duplicate ID)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_wasm::actor::{SupervisorNodeBridge, ComponentActor, SupervisorConfig};
    /// use airssys_wasm::core::ComponentId;
    ///
    /// async fn register_example(bridge: &mut dyn SupervisorNodeBridge) -> Result<(), WasmError> {
    ///     let component_id = ComponentId::new("my-component");
    ///     let actor = ComponentActor::new(/* ... */)?;
    ///     let config = SupervisorConfig::permanent();
    ///     
    ///     bridge.register_component(component_id, actor, config).await?;
    ///     Ok(())
    /// }
    /// ```
    async fn register_component(
        &mut self,
        component_id: ComponentId,
        actor: ComponentActor,
        config: SupervisorConfig,
    ) -> Result<(), WasmError>;

    /// Start a supervised component.
    ///
    /// Calls Child::start() on the ComponentActor through SupervisorNode.
    /// If start fails, SupervisorNode will handle restart according to policy.
    ///
    /// # Parameters
    ///
    /// - `component_id`: Component to start
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Component started successfully
    /// - `Err(WasmError)`: Component not found or start failed
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// async fn start_example(
    ///     bridge: &mut dyn SupervisorNodeBridge,
    ///     component_id: &ComponentId,
    /// ) -> Result<(), WasmError> {
    ///     bridge.start_component(component_id).await?;
    ///     Ok(())
    /// }
    /// ```
    async fn start_component(&mut self, component_id: &ComponentId) -> Result<(), WasmError>;

    /// Stop a supervised component gracefully.
    ///
    /// Calls Child::stop() with configured timeout. Component will be
    /// removed from supervision (no automatic restart).
    ///
    /// # Parameters
    ///
    /// - `component_id`: Component to stop
    /// - `timeout`: Maximum time to wait for graceful shutdown
    ///
    /// # Returns
    ///
    /// - `Ok(())`: Component stopped successfully
    /// - `Err(WasmError)`: Component not found or stop timeout
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use std::time::Duration;
    ///
    /// async fn stop_example(
    ///     bridge: &mut dyn SupervisorNodeBridge,
    ///     component_id: &ComponentId,
    /// ) -> Result<(), WasmError> {
    ///     bridge.stop_component(component_id, Duration::from_secs(5)).await?;
    ///     Ok(())
    /// }
    /// ```
    async fn stop_component(
        &mut self,
        component_id: &ComponentId,
        timeout: Duration,
    ) -> Result<(), WasmError>;

    /// Query the current state of a supervised component.
    ///
    /// Returns component lifecycle state as tracked by SupervisorNode.
    ///
    /// # Parameters
    ///
    /// - `component_id`: Component to query
    ///
    /// # Returns
    ///
    /// - `Some(ComponentSupervisionState)`: Component is supervised
    /// - `None`: Component not found
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// async fn query_example(
    ///     bridge: &dyn SupervisorNodeBridge,
    ///     component_id: &ComponentId,
    /// ) {
    ///     match bridge.get_component_state(component_id) {
    ///         Some(state) => println!("Component state: {:?}", state),
    ///         None => println!("Component not found"),
    ///     }
    /// }
    /// ```
    fn get_component_state(&self, component_id: &ComponentId)
        -> Option<ComponentSupervisionState>;

    /// Start all supervised components.
    ///
    /// Convenience method to start all registered components.
    ///
    /// # Returns
    ///
    /// - `Ok(())`: All components started successfully
    /// - `Err(WasmError)`: One or more components failed to start
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// async fn start_all_example(bridge: &mut dyn SupervisorNodeBridge) -> Result<(), WasmError> {
    ///     bridge.start_all().await?;
    ///     Ok(())
    /// }
    /// ```
    async fn start_all(&mut self) -> Result<(), WasmError>;

    /// Stop all supervised components.
    ///
    /// Gracefully stops all components with configured timeout.
    ///
    /// # Parameters
    ///
    /// - `timeout`: Maximum time to wait for each component shutdown
    ///
    /// # Returns
    ///
    /// - `Ok(())`: All components stopped successfully
    /// - `Err(WasmError)`: One or more components failed to stop
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use std::time::Duration;
    ///
    /// async fn stop_all_example(bridge: &mut dyn SupervisorNodeBridge) -> Result<(), WasmError> {
    ///     bridge.stop_all(Duration::from_secs(10)).await?;
    ///     Ok(())
    /// }
    /// ```
    /// Get restart statistics for a supervised component.
    ///
    /// Returns restart tracking data including total restarts, recent rate,
    /// and last restart timestamp.
    ///
    /// # Parameters
    ///
    /// - `component_id`: Component to query
    ///
    /// # Returns
    ///
    /// - `Some(RestartStats)`: Component is supervised and has tracking data
    /// - `None`: Component not found or not supervised
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// async fn stats_example(
    ///     bridge: &dyn SupervisorNodeBridge,
    ///     component_id: &ComponentId,
    /// ) {
    ///     if let Some(stats) = bridge.get_restart_stats(component_id) {
    ///         println!("Total restarts: {}", stats.total_restarts);
    ///         println!("Recent rate: {}/sec", stats.recent_rate);
    ///     }
    /// }
    /// ```
    fn get_restart_stats(&self, component_id: &ComponentId) -> Option<crate::actor::RestartStats>;

    /// Reset restart tracking for a supervised component.
    ///
    /// Clears restart history, resets counters, and resets backoff state.
    /// Useful after successful recovery or manual intervention.
    ///
    /// # Parameters
    ///
    /// - `component_id`: Component to reset
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// async fn reset_example(
    ///     bridge: &mut dyn SupervisorNodeBridge,
    ///     component_id: &ComponentId,
    /// ) {
    ///     bridge.reset_restart_tracking(component_id);
    ///     println!("Restart tracking reset for component");
    /// }
    /// ```
    fn reset_restart_tracking(&mut self, component_id: &ComponentId);

    /// Query restart history for a supervised component.
    ///
    /// Returns up to `limit` most recent restart records (newest first).
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
    /// # Examples
    ///
    /// ```rust,ignore
    /// async fn history_example(
    ///     bridge: &dyn SupervisorNodeBridge,
    ///     component_id: &ComponentId,
    /// ) {
    ///     let history = bridge.query_restart_history(component_id, 5);
    ///     for record in history {
    ///         println!("Restart at {:?}: {:?}", record.timestamp, record.reason);
    ///     }
    /// }
    /// ```
    fn query_restart_history(
        &self,
        component_id: &ComponentId,
        limit: usize,
    ) -> Vec<crate::actor::RestartRecord>;
    async fn stop_all(&mut self, timeout: Duration) -> Result<(), WasmError>;
}

/// Component state as viewed through supervision.
///
/// Represents the lifecycle state of a supervised component as tracked
/// by the SupervisorNode.
///
/// # State Transitions
///
/// ```text
/// Registered → Starting → Running → Restarting → Running
///                    ↓        ↓           ↓
///                  Failed   Stopped     Failed
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentSupervisionState {
    /// Component registered but not started
    Registered,

    /// Component is starting (Child::start() in progress)
    Starting,

    /// Component is running normally
    Running,

    /// Component failed, restart scheduled
    Restarting,

    /// Component stopped normally
    Stopped,

    /// Component hit restart limit
    Failed,
}

impl ComponentSupervisionState {
    /// Returns true if the component is in a running state.
    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Returns true if the component is in a failed state.
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed)
    }

    /// Returns true if the component is stopped.
    pub fn is_stopped(&self) -> bool {
        matches!(self, Self::Stopped)
    }

    /// Returns true if the component is transitioning (starting or restarting).
    pub fn is_transitioning(&self) -> bool {
        matches!(self, Self::Starting | Self::Restarting)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supervision_state_is_running() {
        assert!(ComponentSupervisionState::Running.is_running());
        assert!(!ComponentSupervisionState::Stopped.is_running());
        assert!(!ComponentSupervisionState::Failed.is_running());
    }

    #[test]
    fn test_supervision_state_is_failed() {
        assert!(ComponentSupervisionState::Failed.is_failed());
        assert!(!ComponentSupervisionState::Running.is_failed());
        assert!(!ComponentSupervisionState::Stopped.is_failed());
    }

    #[test]
    fn test_supervision_state_is_stopped() {
        assert!(ComponentSupervisionState::Stopped.is_stopped());
        assert!(!ComponentSupervisionState::Running.is_stopped());
        assert!(!ComponentSupervisionState::Failed.is_stopped());
    }

    #[test]
    fn test_supervision_state_is_transitioning() {
        assert!(ComponentSupervisionState::Starting.is_transitioning());
        assert!(ComponentSupervisionState::Restarting.is_transitioning());
        assert!(!ComponentSupervisionState::Running.is_transitioning());
        assert!(!ComponentSupervisionState::Stopped.is_transitioning());
    }

    #[test]
    fn test_supervision_state_equality() {
        assert_eq!(
            ComponentSupervisionState::Running,
            ComponentSupervisionState::Running
        );
        assert_ne!(
            ComponentSupervisionState::Running,
            ComponentSupervisionState::Stopped
        );
    }

    #[test]
    fn test_supervision_state_debug() {
        let state = ComponentSupervisionState::Running;
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("Running"));
    }
}
