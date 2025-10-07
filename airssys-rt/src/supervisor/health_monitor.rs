//! Background health monitoring utilities for supervisors.
//!
//! This module provides helper functions and utilities for implementing
//! automatic background health checking of supervised children.
//!
//! # Design Philosophy
//!
//! Rather than tightly coupling background health checking into SupervisorNode,
//! this module provides composable utilities that users can integrate as needed.
//! This follows YAGNI principles while still providing production-ready patterns.

// Layer 1: Standard library imports
use std::sync::Arc;
use std::time::Duration;

// Layer 2: Third-party crate imports
use tokio::sync::Mutex;
use tokio::time::interval;

// Layer 3: Internal module imports
use super::node::SupervisorNode;
use super::traits::{Child, SupervisionStrategy};
use crate::monitoring::{Monitor, SupervisionEvent};

/// Background health monitoring task for a supervisor.
///
/// This function spawns a background task that periodically checks the health
/// of all children in a supervisor and triggers automatic restarts on failures.
///
/// # Parameters
///
/// - `supervisor`: Arc-wrapped Mutex-protected supervisor instance
/// - `check_interval`: How often to check child health
///
/// # Returns
///
/// A tuple of `(JoinHandle, oneshot::Sender)` for task control:
/// - `JoinHandle`: Handle to the background task
/// - `Sender`: Send `()` to this channel to gracefully stop health checking
///
/// # Examples
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use std::time::Duration;
/// use tokio::sync::Mutex;
/// use airssys_rt::supervisor::{SupervisorNode, OneForOne, health_monitor::spawn_health_monitor};
/// use airssys_rt::monitoring::InMemoryMonitor;
///
/// # use airssys_rt::supervisor::Child;
/// # use async_trait::async_trait;
/// # struct MyWorker;
/// # #[derive(Debug)]
/// # struct MyError;
/// # impl std::fmt::Display for MyError {
/// #     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { Ok(()) }
/// # }
/// # impl std::error::Error for MyError {}
/// # #[async_trait]
/// # impl Child for MyWorker {
/// #     type Error = MyError;
/// #     async fn start(&mut self) -> Result<(), Self::Error> { Ok(()) }
/// #     async fn stop(&mut self, _: Duration) -> Result<(), Self::Error> { Ok(()) }
/// # }
/// # async fn example() {
/// let monitor = InMemoryMonitor::new(Default::default());
/// let mut supervisor = SupervisorNode::<OneForOne, MyWorker, _>::new(OneForOne, monitor);
///
/// // Enable health monitoring
/// supervisor.enable_health_checks(
///     Duration::from_secs(30),
///     Duration::from_secs(5),
///     3,
/// );
///
/// // Wrap in Arc<Mutex<>> and spawn background health checker
/// let supervisor_arc = Arc::new(Mutex::new(supervisor));
/// let (task_handle, shutdown_tx) = spawn_health_monitor(
///     supervisor_arc.clone(),
///     Duration::from_secs(30),
/// );
///
/// // Later, to stop health checking:
/// let _ = shutdown_tx.send(());
/// let _ = task_handle.await;
/// # }
/// ```
pub fn spawn_health_monitor<S, C, M>(
    supervisor: Arc<Mutex<SupervisorNode<S, C, M>>>,
    check_interval: Duration,
) -> (
    tokio::task::JoinHandle<()>,
    tokio::sync::oneshot::Sender<()>,
)
where
    S: SupervisionStrategy + Send + Sync + 'static,
    C: Child + Send + Sync + 'static,
    M: Monitor<SupervisionEvent> + Send + Sync + 'static,
{
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();

    let task_handle = tokio::spawn(async move {
        let mut check_interval = interval(check_interval);

        loop {
            tokio::select! {
                _ = &mut shutdown_rx => {
                    // Shutdown signal received
                    break;
                }
                _ = check_interval.tick() => {
                    // Perform health checks
                    let mut sup = supervisor.lock().await;

                    // Check if health monitoring is still enabled
                    if !sup.is_health_monitoring_enabled() {
                        break;
                    }

                    // Get list of child IDs (clone to avoid holding lock during checks)
                    let child_ids: Vec<_> = sup.child_ids().to_vec();

                    // Check each child
                    for child_id in child_ids {
                        if let Err(e) = sup.check_child_health(&child_id).await {
                            // Health check failed - error already logged via monitoring
                            eprintln!("Health check error for child {child_id}: {e}");
                        }
                    }
                }
            }
        }
    });

    (task_handle, shutdown_tx)
}
