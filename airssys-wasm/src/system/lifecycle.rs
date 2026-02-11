//! # LifecycleManager - System Lifecycle State Machine
//!
//! Manages the system lifecycle phases with event notifications. Operates as
//! a state machine tracking phase transitions:
//!
//! ```text
//! Created -> Starting -> Running -> Stopping -> Stopped
//! ```
//!
//! Notifies registered listeners of lifecycle events and provides convenience
//! methods that delegate start/stop operations to the [`SystemCoordinator`].
//!
//! # Architecture
//!
//! LifecycleManager is part of Layer 4 (system/ module). As part of the
//! composition root, it is allowed to use `Box<dyn LifecycleListener>` for
//! heterogeneous listener storage (S6.2 exception for Layer 4).
//!
//! The struct itself is NOT generic. Its `start()` and `stop()` methods are
//! generic over the SystemCoordinator's type parameters `<E, L, V, A, B>`,
//! keeping the API simple while supporting any coordinator instantiation.
//!
//! # Event Design (ADR-WASM-032 Evolution)
//!
//! The ADR reference design uses simple label variants (`Starting`, `Started`,
//! `Stopping`, `Stopped`, `Error(String)`). This implementation evolves the
//! design to use structured variants with richer content:
//!
//! - `PhaseChanged { from, to }` replaces individual phase events
//! - `StartFailed { error }` and `StopFailed { error }` distinguish failure origin
//! - `ComponentLoaded` / `ComponentUnloaded` retained from ADR
//!
//! # References
//!
//! - ADR-WASM-032: System Module Design
//! - ADR-WASM-023: Module Boundary Enforcement (Layer 4)
//! - KNOWLEDGE-WASM-037: Dependency Inversion Principle

// Layer 1: Standard library imports
use std::fmt;

// Layer 2: Third-party crate imports
use airssys_rt::broker::MessageBroker;
use thiserror::Error;

// Layer 3: Internal module imports
use super::coordinator::{SystemCoordinator, SystemError};
use crate::component::wrapper::ComponentActorMessage;
use crate::core::runtime::traits::{ComponentLoader, RuntimeEngine};
use crate::core::security::traits::{SecurityAuditLogger, SecurityValidator};

// ============================================================================
// LifecyclePhase
// ============================================================================

/// Phases of the system lifecycle.
///
/// Transitions are strictly linear:
/// `Created -> Starting -> Running -> Stopping -> Stopped`
///
/// Any other transition is rejected with [`LifecycleError::InvalidTransition`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecyclePhase {
    /// System has been created but not yet started.
    Created,
    /// System is in the process of starting.
    Starting,
    /// System is fully operational.
    Running,
    /// System is in the process of stopping.
    Stopping,
    /// System has been fully stopped.
    Stopped,
}

impl fmt::Display for LifecyclePhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created => write!(f, "Created"),
            Self::Starting => write!(f, "Starting"),
            Self::Running => write!(f, "Running"),
            Self::Stopping => write!(f, "Stopping"),
            Self::Stopped => write!(f, "Stopped"),
        }
    }
}

// ============================================================================
// LifecycleEvent
// ============================================================================

/// Events emitted during lifecycle transitions and operations.
///
/// Listeners receive these events to react to system state changes.
///
/// # Design Note (ADR-WASM-032 Evolution)
///
/// The ADR reference design uses simple label variants (`Starting`, `Started`,
/// etc.). This implementation uses structured variants with richer content:
///
/// - `PhaseChanged { from, to }` captures ALL phase transitions with both
///   source and target phases in a single variant.
/// - `StartFailed` / `StopFailed` distinguish failure origin so listeners
///   can react differently.
/// - Error messages are stored as `String` (not `SystemError`) because
///   `LifecycleEvent` must be `Clone` and `SystemError` is not `Clone`.
#[derive(Debug, Clone)]
pub enum LifecycleEvent {
    /// A phase transition occurred.
    PhaseChanged {
        /// The phase before the transition.
        from: LifecyclePhase,
        /// The phase after the transition.
        to: LifecyclePhase,
    },
    /// System start failed. Phase was rolled back.
    StartFailed {
        /// The stringified error from the coordinator.
        error: String,
    },
    /// System stop failed. Phase was rolled back.
    StopFailed {
        /// The stringified error from the coordinator.
        error: String,
    },
    /// A component was loaded into the system.
    ComponentLoaded(String),
    /// A component was unloaded from the system.
    ComponentUnloaded(String),
}

// ============================================================================
// LifecycleError
// ============================================================================

/// Errors that can occur during lifecycle operations.
#[derive(Debug, Error)]
pub enum LifecycleError {
    /// An invalid phase transition was attempted.
    #[error("Invalid transition from {from} to {to}")]
    InvalidTransition {
        /// The current phase.
        from: LifecyclePhase,
        /// The requested target phase.
        to: LifecyclePhase,
    },

    /// The coordinator start operation failed.
    #[error("Start failed: {0}")]
    StartFailed(#[source] SystemError),

    /// The coordinator shutdown operation failed.
    #[error("Stop failed: {0}")]
    StopFailed(#[source] SystemError),
}

// ============================================================================
// LifecycleListener
// ============================================================================

/// Trait for receiving lifecycle event notifications.
///
/// Implementations must be thread-safe (`Send + Sync`) to support
/// concurrent access from the system coordinator.
pub trait LifecycleListener: Send + Sync {
    /// Called when a lifecycle event occurs.
    ///
    /// # Arguments
    ///
    /// * `event` - Reference to the lifecycle event (not owned, to avoid cloning).
    fn on_event(&self, event: &LifecycleEvent);
}

// ============================================================================
// LifecycleManager
// ============================================================================

/// Manages the system lifecycle phases with event notifications.
///
/// Operates as a state machine tracking phase transitions and notifying
/// registered listeners. Provides convenience methods that delegate start/stop
/// operations to a [`SystemCoordinator`].
///
/// # Non-Generic Design
///
/// LifecycleManager itself is NOT generic. Its `start()` and `stop()` methods
/// are generic over the SystemCoordinator's type parameters, keeping the struct
/// simple while supporting any coordinator instantiation.
///
/// # Listener Storage
///
/// Uses `Box<dyn LifecycleListener>` for heterogeneous listener storage.
/// This is acceptable in Layer 4 (composition root) per S6.2 exception.
pub struct LifecycleManager {
    phase: LifecyclePhase,
    listeners: Vec<Box<dyn LifecycleListener>>,
}

impl LifecycleManager {
    /// Creates a new LifecycleManager in the [`LifecyclePhase::Created`] phase.
    pub fn new() -> Self {
        Self {
            phase: LifecyclePhase::Created,
            listeners: Vec::new(),
        }
    }

    /// Returns the current lifecycle phase.
    pub fn current_phase(&self) -> LifecyclePhase {
        self.phase
    }

    /// Adds a lifecycle listener that will be notified of events.
    pub fn add_listener(&mut self, listener: Box<dyn LifecycleListener>) {
        self.listeners.push(listener);
    }

    /// Returns the number of registered listeners.
    pub fn listener_count(&self) -> usize {
        self.listeners.len()
    }

    /// Notifies listeners that a component has been loaded.
    ///
    /// Emits a [`LifecycleEvent::ComponentLoaded`] event to all listeners.
    pub fn notify_component_loaded(&self, component_id: &str) {
        self.notify(&LifecycleEvent::ComponentLoaded(component_id.to_string()));
    }

    /// Notifies listeners that a component has been unloaded.
    ///
    /// Emits a [`LifecycleEvent::ComponentUnloaded`] event to all listeners.
    pub fn notify_component_unloaded(&self, component_id: &str) {
        self.notify(&LifecycleEvent::ComponentUnloaded(component_id.to_string()));
    }

    /// Starts the system by delegating to the coordinator.
    ///
    /// Transitions through `Created -> Starting -> Running`. If the coordinator
    /// start fails, the phase is rolled back to `Created` and a
    /// [`LifecycleEvent::StartFailed`] event is emitted.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::InvalidTransition`] if not in `Created` phase
    /// - [`LifecycleError::StartFailed`] if coordinator start fails
    pub fn start<E, L, V, A, B>(
        &mut self,
        coordinator: &mut SystemCoordinator<E, L, V, A, B>,
    ) -> Result<(), LifecycleError>
    where
        E: RuntimeEngine + 'static,
        L: ComponentLoader + 'static,
        V: SecurityValidator,
        A: SecurityAuditLogger,
        B: MessageBroker<ComponentActorMessage> + Clone + Send + Sync + 'static,
    {
        // Step 1: Validate we can transition to Starting
        self.validate_transition(LifecyclePhase::Starting)?;

        // Step 2: Transition to Starting and notify
        let from = self.phase;
        self.phase = LifecyclePhase::Starting;
        self.notify(&LifecycleEvent::PhaseChanged {
            from,
            to: LifecyclePhase::Starting,
        });

        // Step 3: Delegate to coordinator
        match coordinator.start() {
            Ok(()) => {
                // Step 4a: Success - transition to Running
                self.phase = LifecyclePhase::Running;
                self.notify(&LifecycleEvent::PhaseChanged {
                    from: LifecyclePhase::Starting,
                    to: LifecyclePhase::Running,
                });
                Ok(())
            }
            Err(err) => {
                // Step 4b: Failure - roll back to Created, notify error
                self.phase = from;
                self.notify(&LifecycleEvent::StartFailed {
                    error: err.to_string(),
                });
                Err(LifecycleError::StartFailed(err))
            }
        }
    }

    /// Stops the system by delegating to the coordinator.
    ///
    /// Transitions through `Running -> Stopping -> Stopped`. If the coordinator
    /// shutdown fails, the phase is rolled back to `Running` and a
    /// [`LifecycleEvent::StopFailed`] event is emitted.
    ///
    /// # Errors
    ///
    /// - [`LifecycleError::InvalidTransition`] if not in `Running` phase
    /// - [`LifecycleError::StopFailed`] if coordinator shutdown fails
    pub async fn stop<E, L, V, A, B>(
        &mut self,
        coordinator: &mut SystemCoordinator<E, L, V, A, B>,
    ) -> Result<(), LifecycleError>
    where
        E: RuntimeEngine + 'static,
        L: ComponentLoader + 'static,
        V: SecurityValidator,
        A: SecurityAuditLogger,
        B: MessageBroker<ComponentActorMessage> + Clone + Send + Sync + 'static,
    {
        // Step 1: Validate we can transition to Stopping
        self.validate_transition(LifecyclePhase::Stopping)?;

        // Step 2: Transition to Stopping and notify
        let from = self.phase;
        self.phase = LifecyclePhase::Stopping;
        self.notify(&LifecycleEvent::PhaseChanged {
            from,
            to: LifecyclePhase::Stopping,
        });

        // Step 3: Delegate to coordinator (async)
        match coordinator.shutdown().await {
            Ok(()) => {
                // Step 4a: Success - transition to Stopped
                self.phase = LifecyclePhase::Stopped;
                self.notify(&LifecycleEvent::PhaseChanged {
                    from: LifecyclePhase::Stopping,
                    to: LifecyclePhase::Stopped,
                });
                Ok(())
            }
            Err(err) => {
                // Step 4b: Failure - roll back to Running, notify error
                self.phase = from;
                self.notify(&LifecycleEvent::StopFailed {
                    error: err.to_string(),
                });
                Err(LifecycleError::StopFailed(err))
            }
        }
    }

    // ========================================================================
    // Private Methods
    // ========================================================================

    /// Validates that the requested phase transition is valid.
    fn validate_transition(&self, to: LifecyclePhase) -> Result<(), LifecycleError> {
        let valid = matches!(
            (self.phase, to),
            (LifecyclePhase::Created, LifecyclePhase::Starting)
                | (LifecyclePhase::Starting, LifecyclePhase::Running)
                | (LifecyclePhase::Running, LifecyclePhase::Stopping)
                | (LifecyclePhase::Stopping, LifecyclePhase::Stopped)
        );

        if valid {
            Ok(())
        } else {
            Err(LifecycleError::InvalidTransition {
                from: self.phase,
                to,
            })
        }
    }

    /// Notifies all registered listeners of an event.
    fn notify(&self, event: &LifecycleEvent) {
        for listener in &self.listeners {
            listener.on_event(event);
        }
    }
}

// ============================================================================
// Debug Implementation
// ============================================================================

impl fmt::Debug for LifecycleManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LifecycleManager")
            .field("phase", &self.phase)
            .field("listener_count", &self.listeners.len())
            .finish()
    }
}

// ============================================================================
// Default Implementation
// ============================================================================

impl Default for LifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Layer 1: Standard library imports
    use std::sync::{Arc, Mutex};

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
    // Type alias for test coordinator
    // ========================================
    type TestCoordinator = SystemCoordinator<
        MockRuntimeEngine,
        MockComponentLoader,
        MockSecurityValidator,
        MockAuditLogger,
        InMemoryMessageBroker<ComponentActorMessage>,
    >;

    // ========================================
    // Helper: create coordinator with mocks
    // ========================================
    fn create_test_coordinator() -> TestCoordinator {
        let engine = Arc::new(MockRuntimeEngine);
        let loader = Arc::new(MockComponentLoader);
        let validator = Arc::new(MockSecurityValidator);
        let logger = Arc::new(MockAuditLogger);
        let broker = InMemoryMessageBroker::<ComponentActorMessage>::new();

        SystemCoordinator::new(
            engine,
            loader,
            validator,
            logger,
            SystemConfig::default(),
            broker,
        )
    }

    // ========================================
    // RecordingListener - captures events
    // ========================================

    /// Structured representation of a recorded event for test assertions.
    #[derive(Debug, Clone)]
    #[expect(
        dead_code,
        reason = "StopFailed.error field exists for structural parity with LifecycleEvent but stop failure cannot be triggered with mock types"
    )]
    enum RecordedEvent {
        PhaseChanged {
            from: LifecyclePhase,
            to: LifecyclePhase,
        },
        StartFailed {
            error: String,
        },
        StopFailed {
            error: String,
        },
        ComponentLoaded(String),
        ComponentUnloaded(String),
    }

    struct RecordingListener {
        events: Arc<Mutex<Vec<RecordedEvent>>>,
    }

    impl RecordingListener {
        fn new() -> (Self, Arc<Mutex<Vec<RecordedEvent>>>) {
            let events = Arc::new(Mutex::new(Vec::new()));
            let listener = Self {
                events: Arc::clone(&events),
            };
            (listener, events)
        }
    }

    impl LifecycleListener for RecordingListener {
        fn on_event(&self, event: &LifecycleEvent) {
            let recorded = match event {
                LifecycleEvent::PhaseChanged { from, to } => RecordedEvent::PhaseChanged {
                    from: *from,
                    to: *to,
                },
                LifecycleEvent::StartFailed { error } => RecordedEvent::StartFailed {
                    error: error.clone(),
                },
                LifecycleEvent::StopFailed { error } => RecordedEvent::StopFailed {
                    error: error.clone(),
                },
                LifecycleEvent::ComponentLoaded(id) => RecordedEvent::ComponentLoaded(id.clone()),
                LifecycleEvent::ComponentUnloaded(id) => {
                    RecordedEvent::ComponentUnloaded(id.clone())
                }
            };
            if let Ok(mut events) = self.events.lock() {
                events.push(recorded);
            }
        }
    }

    // ========================================
    // Group 1: Phase Transition Tests
    // ========================================

    #[test]
    fn test_valid_transition_created_to_starting() {
        let manager = LifecycleManager::new();
        assert!(manager
            .validate_transition(LifecyclePhase::Starting)
            .is_ok());
    }

    #[test]
    fn test_valid_transition_starting_to_running() {
        let mut manager = LifecycleManager::new();
        manager.phase = LifecyclePhase::Starting;
        assert!(manager.validate_transition(LifecyclePhase::Running).is_ok());
    }

    #[test]
    fn test_valid_transition_running_to_stopping() {
        let mut manager = LifecycleManager::new();
        manager.phase = LifecyclePhase::Running;
        assert!(manager
            .validate_transition(LifecyclePhase::Stopping)
            .is_ok());
    }

    #[test]
    fn test_valid_transition_stopping_to_stopped() {
        let mut manager = LifecycleManager::new();
        manager.phase = LifecyclePhase::Stopping;
        assert!(manager.validate_transition(LifecyclePhase::Stopped).is_ok());
    }

    #[test]
    fn test_invalid_transition_created_to_running() {
        let manager = LifecycleManager::new();
        let result = manager.validate_transition(LifecyclePhase::Running);
        assert!(result.is_err());

        let err = result.unwrap_err();
        match err {
            LifecycleError::InvalidTransition { from, to } => {
                assert_eq!(from, LifecyclePhase::Created);
                assert_eq!(to, LifecyclePhase::Running);
            }
            _ => panic!("Expected InvalidTransition error"),
        }
    }

    // ========================================
    // Group 2: LifecycleManager Basic Tests
    // ========================================

    #[test]
    fn test_new_creates_in_created_phase() {
        let manager = LifecycleManager::new();
        assert_eq!(manager.current_phase(), LifecyclePhase::Created);
        assert_eq!(manager.listener_count(), 0);
    }

    #[test]
    fn test_add_listener_increments_count() {
        let mut manager = LifecycleManager::new();
        assert_eq!(manager.listener_count(), 0);

        let (listener, _events) = RecordingListener::new();
        manager.add_listener(Box::new(listener));
        assert_eq!(manager.listener_count(), 1);

        let (listener2, _events2) = RecordingListener::new();
        manager.add_listener(Box::new(listener2));
        assert_eq!(manager.listener_count(), 2);
    }

    #[test]
    fn test_default_is_same_as_new() {
        let from_new = LifecycleManager::new();
        let from_default = LifecycleManager::default();

        assert_eq!(from_new.current_phase(), from_default.current_phase());
        assert_eq!(from_new.listener_count(), from_default.listener_count());
    }

    #[test]
    fn test_debug_format_includes_phase() {
        let manager = LifecycleManager::new();
        let debug_str = format!("{:?}", manager);
        assert!(debug_str.contains("LifecycleManager"));
        assert!(debug_str.contains("phase"));
        assert!(debug_str.contains("Created"));
        assert!(debug_str.contains("listener_count"));
        assert!(debug_str.contains("0"));
    }

    // ========================================
    // Group 3: Start/Stop Delegation Tests
    // ========================================

    #[tokio::test]
    async fn test_start_transitions_to_running() {
        let mut coordinator = create_test_coordinator();
        let mut manager = LifecycleManager::new();

        let result = manager.start(&mut coordinator);
        assert!(result.is_ok());
        assert_eq!(manager.current_phase(), LifecyclePhase::Running);

        coordinator.shutdown().await.unwrap_or_default();
    }

    #[tokio::test]
    async fn test_stop_transitions_to_stopped() {
        let mut coordinator = create_test_coordinator();
        let mut manager = LifecycleManager::new();

        manager.start(&mut coordinator).unwrap();
        assert_eq!(manager.current_phase(), LifecyclePhase::Running);

        let result = manager.stop(&mut coordinator).await;
        assert!(result.is_ok());
        assert_eq!(manager.current_phase(), LifecyclePhase::Stopped);
    }

    #[tokio::test]
    async fn test_start_when_not_created_returns_error() {
        let mut coordinator = create_test_coordinator();
        let mut manager = LifecycleManager::new();

        // Start successfully first (now in Running)
        manager.start(&mut coordinator).unwrap();
        assert_eq!(manager.current_phase(), LifecyclePhase::Running);

        // Attempting start again from Running should fail
        let result = manager.start(&mut coordinator);
        assert!(result.is_err());
        match result.unwrap_err() {
            LifecycleError::InvalidTransition { from, to } => {
                assert_eq!(from, LifecyclePhase::Running);
                assert_eq!(to, LifecyclePhase::Starting);
            }
            _ => panic!("Expected InvalidTransition error"),
        }

        coordinator.shutdown().await.unwrap_or_default();
    }

    #[tokio::test]
    async fn test_stop_when_not_running_returns_error() {
        let mut coordinator = create_test_coordinator();
        let mut manager = LifecycleManager::new();

        // Attempting stop from Created should fail
        let result = manager.stop(&mut coordinator).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            LifecycleError::InvalidTransition { from, to } => {
                assert_eq!(from, LifecyclePhase::Created);
                assert_eq!(to, LifecyclePhase::Stopping);
            }
            _ => panic!("Expected InvalidTransition error"),
        }

        coordinator.shutdown().await.unwrap_or_default();
    }

    // ========================================
    // Group 4: Listener Notification Tests
    // ========================================

    #[tokio::test]
    async fn test_start_notifies_listeners_with_phase_changed_events() {
        let mut coordinator = create_test_coordinator();
        let mut manager = LifecycleManager::new();

        let (listener, events) = RecordingListener::new();
        manager.add_listener(Box::new(listener));

        manager.start(&mut coordinator).unwrap();

        {
            let events = events.lock().unwrap();
            assert_eq!(events.len(), 2);

            // First event: Created -> Starting
            match &events[0] {
                RecordedEvent::PhaseChanged { from, to } => {
                    assert_eq!(*from, LifecyclePhase::Created);
                    assert_eq!(*to, LifecyclePhase::Starting);
                }
                other => panic!("Expected PhaseChanged, got {:?}", other),
            }

            // Second event: Starting -> Running
            match &events[1] {
                RecordedEvent::PhaseChanged { from, to } => {
                    assert_eq!(*from, LifecyclePhase::Starting);
                    assert_eq!(*to, LifecyclePhase::Running);
                }
                other => panic!("Expected PhaseChanged, got {:?}", other),
            }
        }

        coordinator.shutdown().await.unwrap_or_default();
    }

    #[tokio::test]
    async fn test_stop_notifies_listeners_with_phase_changed_events() {
        let mut coordinator = create_test_coordinator();
        let mut manager = LifecycleManager::new();

        manager.start(&mut coordinator).unwrap();

        // Add listener AFTER start so we only capture stop events
        let (listener, events) = RecordingListener::new();
        manager.add_listener(Box::new(listener));

        manager.stop(&mut coordinator).await.unwrap();

        let events = events.lock().unwrap();
        assert_eq!(events.len(), 2);

        // First event: Running -> Stopping
        match &events[0] {
            RecordedEvent::PhaseChanged { from, to } => {
                assert_eq!(*from, LifecyclePhase::Running);
                assert_eq!(*to, LifecyclePhase::Stopping);
            }
            other => panic!("Expected PhaseChanged, got {:?}", other),
        }

        // Second event: Stopping -> Stopped
        match &events[1] {
            RecordedEvent::PhaseChanged { from, to } => {
                assert_eq!(*from, LifecyclePhase::Stopping);
                assert_eq!(*to, LifecyclePhase::Stopped);
            }
            other => panic!("Expected PhaseChanged, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_multiple_listeners_all_notified() {
        let mut coordinator = create_test_coordinator();
        let mut manager = LifecycleManager::new();

        let (listener1, events1) = RecordingListener::new();
        let (listener2, events2) = RecordingListener::new();
        manager.add_listener(Box::new(listener1));
        manager.add_listener(Box::new(listener2));

        manager.start(&mut coordinator).unwrap();

        {
            let events1 = events1.lock().unwrap();
            let events2 = events2.lock().unwrap();

            // Both listeners receive the same 2 events
            assert_eq!(events1.len(), 2);
            assert_eq!(events2.len(), 2);

            // Verify both got Created->Starting as first event
            match (&events1[0], &events2[0]) {
                (
                    RecordedEvent::PhaseChanged { from: f1, to: t1 },
                    RecordedEvent::PhaseChanged { from: f2, to: t2 },
                ) => {
                    assert_eq!(*f1, LifecyclePhase::Created);
                    assert_eq!(*t1, LifecyclePhase::Starting);
                    assert_eq!(*f2, LifecyclePhase::Created);
                    assert_eq!(*t2, LifecyclePhase::Starting);
                }
                (other1, other2) => panic!(
                    "Expected PhaseChanged for both, got {:?} and {:?}",
                    other1, other2
                ),
            }
        }

        coordinator.shutdown().await.unwrap_or_default();
    }

    #[test]
    fn test_component_loaded_notification_content() {
        let mut manager = LifecycleManager::new();
        let (listener, events) = RecordingListener::new();
        manager.add_listener(Box::new(listener));

        manager.notify_component_loaded("test/echo/v1");

        let events = events.lock().unwrap();
        assert_eq!(events.len(), 1);
        match &events[0] {
            RecordedEvent::ComponentLoaded(id) => {
                assert_eq!(id, "test/echo/v1");
            }
            other => panic!("Expected ComponentLoaded, got {:?}", other),
        }
    }

    #[test]
    fn test_component_unloaded_notification_content() {
        let mut manager = LifecycleManager::new();
        let (listener, events) = RecordingListener::new();
        manager.add_listener(Box::new(listener));

        manager.notify_component_unloaded("test/echo/v1");

        let events = events.lock().unwrap();
        assert_eq!(events.len(), 1);
        match &events[0] {
            RecordedEvent::ComponentUnloaded(id) => {
                assert_eq!(id, "test/echo/v1");
            }
            other => panic!("Expected ComponentUnloaded, got {:?}", other),
        }
    }

    // ========================================
    // Group 5: Failure-Rollback Tests
    // ========================================

    #[tokio::test]
    async fn test_start_failure_rolls_back_phase() {
        let mut coordinator = create_test_coordinator();

        // Pre-start the coordinator so it is already running
        coordinator.start().unwrap();

        let mut manager = LifecycleManager::new();
        let result = manager.start(&mut coordinator);

        // coordinator.start() returns Err(AlreadyRunning)
        assert!(result.is_err());
        // Phase must roll back to Created (not stuck at Starting)
        assert_eq!(manager.current_phase(), LifecyclePhase::Created);

        coordinator.shutdown().await.unwrap_or_default();
    }

    #[tokio::test]
    async fn test_start_failure_emits_start_failed_event() {
        let mut coordinator = create_test_coordinator();

        // Pre-start the coordinator so it is already running
        coordinator.start().unwrap();

        let mut manager = LifecycleManager::new();
        let (listener, events) = RecordingListener::new();
        manager.add_listener(Box::new(listener));

        let _result = manager.start(&mut coordinator);

        {
            let events = events.lock().unwrap();
            // Expected: 1 PhaseChanged (Created->Starting) + 1 StartFailed
            assert_eq!(events.len(), 2);

            match &events[0] {
                RecordedEvent::PhaseChanged { from, to } => {
                    assert_eq!(*from, LifecyclePhase::Created);
                    assert_eq!(*to, LifecyclePhase::Starting);
                }
                other => panic!("Expected PhaseChanged, got {:?}", other),
            }

            match &events[1] {
                RecordedEvent::StartFailed { error } => {
                    assert!(
                        error.contains("already running"),
                        "Expected error to contain 'already running', got: {}",
                        error
                    );
                }
                other => panic!("Expected StartFailed, got {:?}", other),
            }
        }

        coordinator.shutdown().await.unwrap_or_default();
    }

    // Note on stop() failure rollback:
    // The stop() rollback logic is structurally identical to start():
    //   match coordinator.shutdown().await {
    //       Ok(()) => { /* advance phase */ }
    //       Err(err) => { self.phase = from; self.notify(StopFailed{..}); }
    //   }
    // coordinator.shutdown() cannot be made to fail with mock types:
    //   1. shutdown() when not running is a no-op (returns Ok)
    //   2. The ActorSystem gracefully handles double-shutdown
    // The start failure tests (test_start_failure_rolls_back_phase and
    // test_start_failure_emits_start_failed_event) prove the rollback
    // mechanism works for the identical code pattern.

    #[tokio::test]
    async fn test_stop_success_transitions_phases() {
        let mut coordinator = create_test_coordinator();
        let mut manager = LifecycleManager::new();

        manager.start(&mut coordinator).unwrap();
        assert_eq!(manager.current_phase(), LifecyclePhase::Running);

        let result = manager.stop(&mut coordinator).await;
        assert!(result.is_ok());
        assert_eq!(manager.current_phase(), LifecyclePhase::Stopped);
    }

    #[tokio::test]
    async fn test_stop_success_emits_phase_changed_events() {
        let mut coordinator = create_test_coordinator();
        let mut manager = LifecycleManager::new();

        manager.start(&mut coordinator).unwrap();

        let (listener, events) = RecordingListener::new();
        manager.add_listener(Box::new(listener));

        manager.stop(&mut coordinator).await.unwrap();

        let events = events.lock().unwrap();
        assert_eq!(events.len(), 2);

        match &events[0] {
            RecordedEvent::PhaseChanged { from, to } => {
                assert_eq!(*from, LifecyclePhase::Running);
                assert_eq!(*to, LifecyclePhase::Stopping);
            }
            other => panic!("Expected PhaseChanged, got {:?}", other),
        }

        match &events[1] {
            RecordedEvent::PhaseChanged { from, to } => {
                assert_eq!(*from, LifecyclePhase::Stopping);
                assert_eq!(*to, LifecyclePhase::Stopped);
            }
            other => panic!("Expected PhaseChanged, got {:?}", other),
        }
    }

    // ========================================
    // Group 6: Error Types and Trait Tests
    // ========================================

    #[test]
    fn test_lifecycle_error_display_invalid_transition() {
        let err = LifecycleError::InvalidTransition {
            from: LifecyclePhase::Created,
            to: LifecyclePhase::Running,
        };
        let display = err.to_string();
        assert!(
            display.contains("Created"),
            "Expected 'Created' in: {}",
            display
        );
        assert!(
            display.contains("Running"),
            "Expected 'Running' in: {}",
            display
        );
        assert!(
            display.contains("Invalid transition"),
            "Expected 'Invalid transition' in: {}",
            display
        );
    }

    #[test]
    fn test_lifecycle_phase_display() {
        assert_eq!(LifecyclePhase::Created.to_string(), "Created");
        assert_eq!(LifecyclePhase::Starting.to_string(), "Starting");
        assert_eq!(LifecyclePhase::Running.to_string(), "Running");
        assert_eq!(LifecyclePhase::Stopping.to_string(), "Stopping");
        assert_eq!(LifecyclePhase::Stopped.to_string(), "Stopped");
    }

    #[test]
    fn test_lifecycle_manager_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<LifecycleManager>();
    }
}
