//! Component lifecycle state machine and version management.
//!
//! This module provides abstractions for managing component lifecycle states,
//! versioning, and update strategies. These types are used by Block 7 (Lifecycle
//! Manager) to orchestrate component state transitions and updates.
//!
//! # Architecture
//!
//! The lifecycle abstractions follow a state machine pattern:
//!
//! ```text
//! Uninstalled → Installing → Installed → Starting → Running
//!                               ↓           ↓          ↓
//!                            Failed ← Stopping ← Updating
//!                                        ↓
//!                                    Stopped
//! ```
//!
//! # Design Principles
//!
//! - **Clear State Transitions**: Each state has well-defined entry/exit conditions
//! - **Versioning Support**: Complete version tracking with signatures
//! - **Update Strategies**: Support for different deployment patterns
//! - **Event Tracking**: All state transitions generate lifecycle events
//!
//! # References
//!
//! - **ADR-WASM-012**: Comprehensive Core Abstractions Strategy
//! - **Block 7**: Lifecycle Manager implementation
//! - **Workspace Standards**: §3.2 (DateTime<Utc> standard)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::core::ComponentId;

/// Component lifecycle state machine.
///
/// Represents the current state of a component in its lifecycle. State transitions
/// are managed by the Lifecycle Manager (Block 7) and must follow valid paths.
///
/// # State Transition Rules
///
/// - `Uninstalled` → `Installing` (installation starts)
/// - `Installing` → `Installed` (installation succeeds)
/// - `Installing` → `Failed` (installation fails)
/// - `Installed` → `Starting` (component activation begins)
/// - `Starting` → `Running` (component is active)
/// - `Starting` → `Failed` (activation fails)
/// - `Running` → `Updating` (update initiated)
/// - `Running` → `Stopping` (shutdown initiated)
/// - `Updating` → `Running` (update succeeds)
/// - `Updating` → `Failed` (update fails)
/// - `Stopping` → `Stopped` (shutdown complete)
/// - `Stopped` → `Starting` (restart)
/// - `Failed` → `Uninstalled` (cleanup after failure)
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::lifecycle::LifecycleState;
///
/// let state = LifecycleState::Uninstalled;
/// assert_eq!(state, LifecycleState::Uninstalled);
///
/// // Transition to installing
/// let state = LifecycleState::Installing;
/// assert_eq!(state, LifecycleState::Installing);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifecycleState {
    /// Component is not installed (initial state)
    Uninstalled,
    
    /// Component installation is in progress
    Installing,
    
    /// Component is installed but not running
    Installed,
    
    /// Component is starting up
    Starting,
    
    /// Component is running and can handle requests
    Running,
    
    /// Component is being updated (hot deployment)
    Updating,
    
    /// Component is shutting down
    Stopping,
    
    /// Component has stopped cleanly
    Stopped,
    
    /// Component is in a failed state (needs intervention)
    Failed,
}

impl LifecycleState {
    /// Check if the state represents a terminal state.
    ///
    /// Terminal states require external action to transition out.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::lifecycle::LifecycleState;
    ///
    /// assert!(LifecycleState::Failed.is_terminal());
    /// assert!(LifecycleState::Uninstalled.is_terminal());
    /// assert!(!LifecycleState::Running.is_terminal());
    /// ```
    pub fn is_terminal(&self) -> bool {
        matches!(self, LifecycleState::Failed | LifecycleState::Uninstalled)
    }

    /// Check if the state represents an active running state.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::lifecycle::LifecycleState;
    ///
    /// assert!(LifecycleState::Running.is_active());
    /// assert!(!LifecycleState::Stopped.is_active());
    /// ```
    pub fn is_active(&self) -> bool {
        matches!(self, LifecycleState::Running)
    }

    /// Check if the state represents a transitional state.
    ///
    /// Transitional states should resolve to stable states automatically.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::lifecycle::LifecycleState;
    ///
    /// assert!(LifecycleState::Installing.is_transitional());
    /// assert!(LifecycleState::Starting.is_transitional());
    /// assert!(!LifecycleState::Running.is_transitional());
    /// ```
    pub fn is_transitional(&self) -> bool {
        matches!(
            self,
            LifecycleState::Installing
                | LifecycleState::Starting
                | LifecycleState::Updating
                | LifecycleState::Stopping
        )
    }
}

/// Component version information.
///
/// Tracks version metadata including semantic version, content hash, optional
/// cryptographic signature, and installation timestamp.
///
/// # Security
///
/// The `signature` field should contain Ed25519 signatures when components are
/// installed from remote sources. The signature verifies the component hash
/// against a trusted public key.
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::lifecycle::VersionInfo;
/// use chrono::Utc;
///
/// let version = VersionInfo {
///     version: "1.2.3".to_string(),
///     hash: "abc123def456".to_string(),
///     signature: None,
///     installed_at: Utc::now(),
/// };
///
/// assert_eq!(version.version, "1.2.3");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Semantic version string (e.g., "1.2.3")
    pub version: String,
    
    /// Content hash (SHA-256) of component WASM binary
    pub hash: String,
    
    /// Optional Ed25519 signature for verification
    pub signature: Option<Vec<u8>>,
    
    /// Timestamp when this version was installed
    pub installed_at: DateTime<Utc>,
}

impl VersionInfo {
    /// Create a new version info without signature.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::lifecycle::VersionInfo;
    ///
    /// let version = VersionInfo::new("1.0.0", "abc123");
    /// assert_eq!(version.version, "1.0.0");
    /// assert!(version.signature.is_none());
    /// ```
    pub fn new(version: impl Into<String>, hash: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            hash: hash.into(),
            signature: None,
            installed_at: Utc::now(),
        }
    }

    /// Create a new version info with signature.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::lifecycle::VersionInfo;
    ///
    /// let signature = vec![0u8; 64]; // Mock Ed25519 signature
    /// let version = VersionInfo::with_signature("1.0.0", "abc123", signature.clone());
    /// assert_eq!(version.version, "1.0.0");
    /// assert_eq!(version.signature, Some(signature));
    /// ```
    pub fn with_signature(
        version: impl Into<String>,
        hash: impl Into<String>,
        signature: Vec<u8>,
    ) -> Self {
        Self {
            version: version.into(),
            hash: hash.into(),
            signature: Some(signature),
            installed_at: Utc::now(),
        }
    }

    /// Check if this version has a valid signature.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::lifecycle::VersionInfo;
    ///
    /// let unsigned = VersionInfo::new("1.0.0", "abc123");
    /// assert!(!unsigned.is_signed());
    ///
    /// let signed = VersionInfo::with_signature("1.0.0", "abc123", vec![0u8; 64]);
    /// assert!(signed.is_signed());
    /// ```
    pub fn is_signed(&self) -> bool {
        self.signature.is_some()
    }
}

/// Update strategy for component upgrades.
///
/// Defines different deployment patterns for updating running components.
/// The chosen strategy affects downtime, resource usage, and rollback complexity.
///
/// # Strategy Comparison
///
/// | Strategy   | Downtime | Resource Usage | Rollback    | Complexity |
/// |------------|----------|----------------|-------------|------------|
/// | StopStart  | Yes      | Low            | Simple      | Low        |
/// | BlueGreen  | No       | High (2x)      | Instant     | Medium     |
/// | Canary     | No       | High (2x)      | Gradual     | High       |
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::lifecycle::UpdateStrategy;
///
/// let strategy = UpdateStrategy::BlueGreen;
/// assert!(strategy.is_zero_downtime());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UpdateStrategy {
    /// Stop old version, start new version (simple, with downtime).
    ///
    /// This is the simplest update strategy but involves downtime during
    /// the transition. Best for non-critical components.
    StopStart,
    
    /// Start new version, switch traffic, stop old version (zero downtime).
    ///
    /// Also known as "blue-green deployment". The new version is started
    /// alongside the old, traffic is switched instantly, then the old version
    /// is stopped. Requires 2x resources temporarily but enables instant rollback.
    BlueGreen,
    
    /// Gradual traffic shift from old to new version (zero downtime, controlled).
    ///
    /// Also known as "canary deployment". Traffic is gradually shifted from
    /// the old version to the new version, allowing monitoring of the new
    /// version's behavior under real load. Enables safe rollback if issues detected.
    Canary,
}

impl UpdateStrategy {
    /// Check if this strategy provides zero-downtime updates.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::lifecycle::UpdateStrategy;
    ///
    /// assert!(!UpdateStrategy::StopStart.is_zero_downtime());
    /// assert!(UpdateStrategy::BlueGreen.is_zero_downtime());
    /// assert!(UpdateStrategy::Canary.is_zero_downtime());
    /// ```
    pub fn is_zero_downtime(&self) -> bool {
        matches!(self, UpdateStrategy::BlueGreen | UpdateStrategy::Canary)
    }

    /// Check if this strategy requires double resources during update.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::lifecycle::UpdateStrategy;
    ///
    /// assert!(!UpdateStrategy::StopStart.requires_double_resources());
    /// assert!(UpdateStrategy::BlueGreen.requires_double_resources());
    /// assert!(UpdateStrategy::Canary.requires_double_resources());
    /// ```
    pub fn requires_double_resources(&self) -> bool {
        matches!(self, UpdateStrategy::BlueGreen | UpdateStrategy::Canary)
    }
}

/// Lifecycle state transition event.
///
/// Represents a single lifecycle state transition with full context including
/// timestamps, reason, and involved states. Used for auditing and monitoring.
///
/// # Examples
///
/// ```
/// use airssys_wasm::core::lifecycle::{LifecycleEvent, LifecycleState};
/// use airssys_wasm::core::ComponentId;
/// use chrono::Utc;
///
/// let event = LifecycleEvent {
///     component_id: ComponentId::new("my-component"),
///     from_state: LifecycleState::Installed,
///     to_state: LifecycleState::Starting,
///     timestamp: Utc::now(),
///     reason: Some("User requested start".to_string()),
/// };
///
/// assert_eq!(event.from_state, LifecycleState::Installed);
/// assert_eq!(event.to_state, LifecycleState::Starting);
/// ```
#[derive(Debug, Clone)]
pub struct LifecycleEvent {
    /// Component that transitioned
    pub component_id: ComponentId,
    
    /// Previous lifecycle state
    pub from_state: LifecycleState,
    
    /// New lifecycle state
    pub to_state: LifecycleState,
    
    /// When the transition occurred
    pub timestamp: DateTime<Utc>,
    
    /// Optional reason for the transition
    pub reason: Option<String>,
}

impl LifecycleEvent {
    /// Create a new lifecycle event.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::lifecycle::{LifecycleEvent, LifecycleState};
    /// use airssys_wasm::core::ComponentId;
    ///
    /// let event = LifecycleEvent::new(
    ///     ComponentId::new("test"),
    ///     LifecycleState::Installed,
    ///     LifecycleState::Starting,
    /// );
    ///
    /// assert_eq!(event.from_state, LifecycleState::Installed);
    /// ```
    pub fn new(
        component_id: ComponentId,
        from_state: LifecycleState,
        to_state: LifecycleState,
    ) -> Self {
        Self {
            component_id,
            from_state,
            to_state,
            timestamp: Utc::now(),
            reason: None,
        }
    }

    /// Create a new lifecycle event with a reason.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::lifecycle::{LifecycleEvent, LifecycleState};
    /// use airssys_wasm::core::ComponentId;
    ///
    /// let event = LifecycleEvent::with_reason(
    ///     ComponentId::new("test"),
    ///     LifecycleState::Running,
    ///     LifecycleState::Failed,
    ///     "Out of memory",
    /// );
    ///
    /// assert_eq!(event.reason, Some("Out of memory".to_string()));
    /// ```
    pub fn with_reason(
        component_id: ComponentId,
        from_state: LifecycleState,
        to_state: LifecycleState,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            component_id,
            from_state,
            to_state,
            timestamp: Utc::now(),
            reason: Some(reason.into()),
        }
    }

    /// Check if this event represents a failure transition.
    ///
    /// # Examples
    ///
    /// ```
    /// use airssys_wasm::core::lifecycle::{LifecycleEvent, LifecycleState};
    /// use airssys_wasm::core::ComponentId;
    ///
    /// let failure = LifecycleEvent::new(
    ///     ComponentId::new("test"),
    ///     LifecycleState::Running,
    ///     LifecycleState::Failed,
    /// );
    /// assert!(failure.is_failure());
    ///
    /// let success = LifecycleEvent::new(
    ///     ComponentId::new("test"),
    ///     LifecycleState::Starting,
    ///     LifecycleState::Running,
    /// );
    /// assert!(!success.is_failure());
    /// ```
    pub fn is_failure(&self) -> bool {
        self.to_state == LifecycleState::Failed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_state_terminal() {
        assert!(LifecycleState::Failed.is_terminal());
        assert!(LifecycleState::Uninstalled.is_terminal());
        assert!(!LifecycleState::Running.is_terminal());
        assert!(!LifecycleState::Installed.is_terminal());
    }

    #[test]
    fn test_lifecycle_state_active() {
        assert!(LifecycleState::Running.is_active());
        assert!(!LifecycleState::Stopped.is_active());
        assert!(!LifecycleState::Starting.is_active());
    }

    #[test]
    fn test_lifecycle_state_transitional() {
        assert!(LifecycleState::Installing.is_transitional());
        assert!(LifecycleState::Starting.is_transitional());
        assert!(LifecycleState::Updating.is_transitional());
        assert!(LifecycleState::Stopping.is_transitional());
        assert!(!LifecycleState::Running.is_transitional());
        assert!(!LifecycleState::Failed.is_transitional());
    }

    #[test]
    fn test_version_info_creation() {
        let version = VersionInfo::new("1.0.0", "abc123");
        assert_eq!(version.version, "1.0.0");
        assert_eq!(version.hash, "abc123");
        assert!(!version.is_signed());
    }

    #[test]
    fn test_version_info_with_signature() {
        let sig = vec![1, 2, 3, 4];
        let version = VersionInfo::with_signature("1.0.0", "abc123", sig.clone());
        assert_eq!(version.version, "1.0.0");
        assert!(version.is_signed());
        assert_eq!(version.signature, Some(sig));
    }

    #[test]
    fn test_update_strategy_zero_downtime() {
        assert!(!UpdateStrategy::StopStart.is_zero_downtime());
        assert!(UpdateStrategy::BlueGreen.is_zero_downtime());
        assert!(UpdateStrategy::Canary.is_zero_downtime());
    }

    #[test]
    fn test_update_strategy_double_resources() {
        assert!(!UpdateStrategy::StopStart.requires_double_resources());
        assert!(UpdateStrategy::BlueGreen.requires_double_resources());
        assert!(UpdateStrategy::Canary.requires_double_resources());
    }

    #[test]
    fn test_lifecycle_event_creation() {
        let id = ComponentId::new("test-component");
        let event = LifecycleEvent::new(
            id.clone(),
            LifecycleState::Installed,
            LifecycleState::Starting,
        );

        assert_eq!(event.component_id, id);
        assert_eq!(event.from_state, LifecycleState::Installed);
        assert_eq!(event.to_state, LifecycleState::Starting);
        assert!(event.reason.is_none());
        assert!(!event.is_failure());
    }

    #[test]
    fn test_lifecycle_event_with_reason() {
        let id = ComponentId::new("test-component");
        let event = LifecycleEvent::with_reason(
            id,
            LifecycleState::Running,
            LifecycleState::Failed,
            "Out of memory",
        );

        assert_eq!(event.reason, Some("Out of memory".to_string()));
        assert!(event.is_failure());
    }

    #[test]
    fn test_lifecycle_event_failure_detection() {
        let id = ComponentId::new("test");
        
        let failure = LifecycleEvent::new(
            id.clone(),
            LifecycleState::Running,
            LifecycleState::Failed,
        );
        assert!(failure.is_failure());

        let success = LifecycleEvent::new(
            id,
            LifecycleState::Starting,
            LifecycleState::Running,
        );
        assert!(!success.is_failure());
    }
}
