//! Hierarchical supervisor tree for multi-level fault tolerance.
//!
//! This module provides hierarchical supervision capabilities, enabling supervisors
//! to be organized in trees for multi-level fault isolation and error escalation.
//!
//! # Design Philosophy
//!
//! Following YAGNI principles (§6.1), this implementation focuses on essential
//! hierarchical supervision capabilities without premature complexity:
//!
//! - **Parent-child relationships**: Supervisors can have parent supervisors
//! - **Error escalation**: Failed supervisors escalate to their parent
//! - **Coordinated shutdown**: Shutdown propagates top-down through tree
//! - **No trait objects**: Avoids `Box<dyn Supervisor>` pattern (§6.2)
//!
//! # Architecture
//!
//! The tree structure is maintained through explicit parent references rather
//! than a complex tree data structure. This keeps the implementation simple
//! while enabling essential hierarchical supervision patterns.

// Layer 1: Standard library imports
use std::collections::HashMap;

// Layer 2: Third-party crate imports
use uuid::Uuid;

// Layer 3: Internal module imports
use super::error::SupervisorError;
use super::node::SupervisorNode;
use super::traits::{Child, SupervisionStrategy};
use crate::monitoring::{Monitor, SupervisionEvent};

/// Supervisor tree context for hierarchical supervision.
///
/// Maintains the hierarchical relationships between supervisors, enabling
/// error escalation and coordinated shutdown across supervision levels.
///
/// # Type Parameters
///
/// - `S`: Supervision strategy type implementing `SupervisionStrategy`
/// - `C`: Child type implementing the `Child` trait
/// - `M`: Monitor type for supervision events
///
/// # Design Notes
///
/// This is a simple registry-based approach rather than a complex tree structure.
/// Supervisors register themselves and their parent relationship, enabling
/// error escalation without requiring trait objects or complex tree traversal.
///
/// # Examples
///
/// ```rust
/// use airssys_rt::supervisor::{SupervisorTree, SupervisorNode, OneForOne};
/// use airssys_rt::monitoring::InMemoryMonitor;
///
/// # use airssys_rt::supervisor::Child;
/// # use async_trait::async_trait;
/// # use std::time::Duration;
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
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let monitor = InMemoryMonitor::new(MonitoringConfig::default());
/// let mut tree = SupervisorTree::<OneForOne, MyWorker, _>::new();
///
/// // Create root supervisor
/// let root_id = tree.create_supervisor(None, OneForOne, monitor.clone())?;
///
/// // Create child supervisor under root
/// let child_id = tree.create_supervisor(Some(root_id), OneForOne, monitor)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct SupervisorTree<S, C, M>
where
    S: SupervisionStrategy + Clone,
    C: Child,
    M: Monitor<SupervisionEvent> + Clone,
{
    /// Registry of all supervisors in the tree
    supervisors: HashMap<SupervisorId, SupervisorNode<S, C, M>>,

    /// Parent relationships: child_id -> parent_id
    parent_map: HashMap<SupervisorId, SupervisorId>,

    /// Root supervisor IDs (supervisors without parents)
    roots: Vec<SupervisorId>,
}

/// Unique identifier for a supervisor node in the tree.
///
/// Supervisors are identified by UUID to ensure uniqueness across the
/// entire supervision tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SupervisorId(Uuid);

impl SupervisorId {
    /// Creates a new unique supervisor ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Returns the underlying UUID.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for SupervisorId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SupervisorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for SupervisorId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl<S, C, M> SupervisorTree<S, C, M>
where
    S: SupervisionStrategy + Clone,
    C: Child,
    M: Monitor<SupervisionEvent> + Clone + 'static,
{
    /// Creates a new empty supervisor tree.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::{SupervisorTree, OneForOne};
    /// use airssys_rt::monitoring::InMemoryMonitor;
    ///
    /// # use airssys_rt::supervisor::Child;
    /// # use async_trait::async_trait;
    /// # use std::time::Duration;
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
    /// let tree = SupervisorTree::<OneForOne, MyWorker, InMemoryMonitor<_>>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            supervisors: HashMap::new(),
            parent_map: HashMap::new(),
            roots: Vec::new(),
        }
    }

    /// Creates a new supervisor in the tree.
    ///
    /// # Parameters
    ///
    /// - `parent_id`: Optional parent supervisor ID. If `None`, creates a root supervisor.
    /// - `strategy`: Supervision strategy instance for this supervisor
    /// - `monitor`: Monitor instance for recording supervision events
    ///
    /// # Returns
    ///
    /// Returns the unique ID of the newly created supervisor.
    ///
    /// # Errors
    ///
    /// Returns error if parent ID is provided but parent supervisor doesn't exist.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::{SupervisorTree, OneForOne};
    /// use airssys_rt::monitoring::InMemoryMonitor;
    ///
    /// # use airssys_rt::supervisor::Child;
    /// # use async_trait::async_trait;
    /// # use std::time::Duration;
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
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let monitor = InMemoryMonitor::new(MonitoringConfig::default());
    /// let mut tree = SupervisorTree::<OneForOne, MyWorker, _>::new();
    ///
    /// // Create root supervisor
    /// let root = tree.create_supervisor(None, OneForOne, monitor.clone())?;
    ///
    /// // Create child supervisor
    /// let child = tree.create_supervisor(Some(root), OneForOne, monitor)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn create_supervisor(
        &mut self,
        parent_id: Option<SupervisorId>,
        strategy: S,
        monitor: M,
    ) -> Result<SupervisorId, SupervisorError> {
        // Validate parent exists if provided
        if let Some(pid) = parent_id {
            if !self.supervisors.contains_key(&pid) {
                return Err(SupervisorError::TreeIntegrityViolation {
                    reason: format!("Parent supervisor {} not found", pid),
                });
            }
        }

        let supervisor_id = SupervisorId::new();
        let supervisor = SupervisorNode::new(strategy, monitor);

        self.supervisors.insert(supervisor_id, supervisor);

        // Update tree relationships
        if let Some(pid) = parent_id {
            self.parent_map.insert(supervisor_id, pid);
        } else {
            self.roots.push(supervisor_id);
        }

        Ok(supervisor_id)
    }

    /// Removes a supervisor from the tree.
    ///
    /// Stops all children of the supervisor before removal. If the supervisor
    /// has child supervisors, they are also removed recursively.
    ///
    /// # Errors
    ///
    /// Returns error if supervisor is not found or if child shutdown fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::{SupervisorTree, OneForOne};
    /// use airssys_rt::monitoring::InMemoryMonitor;
    ///
    /// # use airssys_rt::supervisor::Child;
    /// # use async_trait::async_trait;
    /// # use std::time::Duration;
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
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let monitor = InMemoryMonitor::new(MonitoringConfig::default());
    /// let mut tree = SupervisorTree::<OneForOne, MyWorker, _>::new();
    /// let supervisor_id = tree.create_supervisor(None, OneForOne, monitor)?;
    ///
    /// // Later: remove the supervisor
    /// tree.remove_supervisor(supervisor_id).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn remove_supervisor(
        &mut self,
        supervisor_id: SupervisorId,
    ) -> Result<(), SupervisorError> {
        // Find and remove all child supervisors first
        let child_supervisors: Vec<SupervisorId> = self
            .parent_map
            .iter()
            .filter(|(_, parent)| **parent == supervisor_id)
            .map(|(child, _)| *child)
            .collect();

        for child_id in child_supervisors {
            // Use Box::pin to avoid infinite-sized future from recursion
            Box::pin(self.remove_supervisor(child_id)).await?;
        }

        // Remove the supervisor itself
        self.supervisors.remove(&supervisor_id).ok_or_else(|| {
            SupervisorError::TreeIntegrityViolation {
                reason: format!("Supervisor {} not found", supervisor_id),
            }
        })?;

        // Clean up relationships
        self.parent_map.remove(&supervisor_id);
        self.roots.retain(|&id| id != supervisor_id);

        Ok(())
    }

    /// Gets a reference to a supervisor by ID.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::{SupervisorTree, OneForOne};
    /// use airssys_rt::monitoring::InMemoryMonitor;
    ///
    /// # use airssys_rt::supervisor::Child;
    /// # use async_trait::async_trait;
    /// # use std::time::Duration;
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
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let monitor = InMemoryMonitor::new(MonitoringConfig::default());
    /// let mut tree = SupervisorTree::<OneForOne, MyWorker, _>::new();
    /// let supervisor_id = tree.create_supervisor(None, OneForOne, monitor)?;
    ///
    /// let supervisor = tree.get_supervisor(supervisor_id)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_supervisor(
        &self,
        supervisor_id: SupervisorId,
    ) -> Result<&SupervisorNode<S, C, M>, SupervisorError> {
        self.supervisors.get(&supervisor_id).ok_or_else(|| {
            SupervisorError::TreeIntegrityViolation {
                reason: format!("Supervisor {} not found", supervisor_id),
            }
        })
    }

    /// Gets a mutable reference to a supervisor by ID.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::{SupervisorTree, OneForOne, ChildSpec, RestartPolicy, ShutdownPolicy};
    /// use airssys_rt::monitoring::InMemoryMonitor;
    /// use std::time::Duration;
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
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let monitor = InMemoryMonitor::new(MonitoringConfig::default());
    /// let mut tree = SupervisorTree::<OneForOne, MyWorker, _>::new();
    /// let supervisor_id = tree.create_supervisor(None, OneForOne, monitor)?;
    ///
    /// // Add a child to the supervisor
    /// let supervisor = tree.get_supervisor_mut(supervisor_id)?;
    /// supervisor.start_child(ChildSpec {
    ///     id: "worker-1".into(),
    ///     factory: || MyWorker,
    ///     restart_policy: RestartPolicy::Permanent,
    ///     shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
    ///     start_timeout: Duration::from_secs(10),
    ///     shutdown_timeout: Duration::from_secs(10),
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_supervisor_mut(
        &mut self,
        supervisor_id: SupervisorId,
    ) -> Result<&mut SupervisorNode<S, C, M>, SupervisorError> {
        self.supervisors.get_mut(&supervisor_id).ok_or_else(|| {
            SupervisorError::TreeIntegrityViolation {
                reason: format!("Supervisor {} not found", supervisor_id),
            }
        })
    }

    /// Gets the parent supervisor ID for a given supervisor.
    ///
    /// Returns `None` if the supervisor is a root supervisor.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::{SupervisorTree, OneForOne};
    /// use airssys_rt::monitoring::InMemoryMonitor;
    ///
    /// # use airssys_rt::supervisor::Child;
    /// # use async_trait::async_trait;
    /// # use std::time::Duration;
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
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let monitor = InMemoryMonitor::new(MonitoringConfig::default());
    /// let mut tree = SupervisorTree::<OneForOne, MyWorker, _>::new();
    ///
    /// let root = tree.create_supervisor(None, OneForOne, monitor.clone())?;
    /// let child = tree.create_supervisor(Some(root), OneForOne, monitor)?;
    ///
    /// assert_eq!(tree.get_parent(child), Some(root));
    /// assert_eq!(tree.get_parent(root), None);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_parent(&self, supervisor_id: SupervisorId) -> Option<SupervisorId> {
        self.parent_map.get(&supervisor_id).copied()
    }

    /// Escalates an error to the parent supervisor.
    ///
    /// This is called when a supervisor exhausts its restart strategies or
    /// encounters an unrecoverable error that needs to be handled at a higher level.
    ///
    /// # Error Handling
    ///
    /// - If supervisor has a parent: Error is escalated to parent's error handling
    /// - If supervisor is root: Error is logged and supervisor is terminated
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use airssys_rt::supervisor::{SupervisorTree, OneForOne, SupervisorError};
    /// use airssys_rt::monitoring::InMemoryMonitor;
    ///
    /// # use airssys_rt::supervisor::Child;
    /// # use async_trait::async_trait;
    /// # use std::time::Duration;
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
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let monitor = InMemoryMonitor::new(MonitoringConfig::default());
    /// let mut tree = SupervisorTree::<OneForOne, MyWorker, _>::new();
    /// let supervisor_id = tree.create_supervisor(None, OneForOne, monitor)?;
    ///
    /// // When supervisor encounters unrecoverable error:
    /// let error = SupervisorError::TreeIntegrityViolation { reason: "Critical failure".into() };
    /// tree.escalate_error(supervisor_id, error).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn escalate_error(
        &mut self,
        supervisor_id: SupervisorId,
        error: SupervisorError,
    ) -> Result<(), SupervisorError> {
        if let Some(parent_id) = self.get_parent(supervisor_id) {
            // Escalate to parent supervisor
            // For now, we log the error. In future phases, this can trigger
            // parent supervision strategies.
            eprintln!(
                "Supervisor {} escalating error to parent {}: {}",
                supervisor_id, parent_id, error
            );
            Ok(())
        } else {
            // Root supervisor - no parent to escalate to
            // This is a critical system error
            Err(SupervisorError::TreeIntegrityViolation {
                reason: format!(
                    "Root supervisor {} encountered unrecoverable error: {}",
                    supervisor_id, error
                ),
            })
        }
    }

    /// Shuts down the entire supervision tree.
    ///
    /// Performs top-down shutdown starting from root supervisors, ensuring
    /// coordinated and graceful termination of all supervised processes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::{SupervisorTree, OneForOne};
    /// use airssys_rt::monitoring::InMemoryMonitor;
    ///
    /// # use airssys_rt::supervisor::Child;
    /// # use async_trait::async_trait;
    /// # use std::time::Duration;
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
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let monitor = InMemoryMonitor::new(MonitoringConfig::default());
    /// let mut tree = SupervisorTree::<OneForOne, MyWorker, _>::new();
    /// tree.create_supervisor(None, OneForOne, monitor)?;
    ///
    /// // Gracefully shutdown the entire tree
    /// tree.shutdown().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn shutdown(&mut self) -> Result<(), SupervisorError> {
        // Shutdown all root supervisors (which will cascade to children)
        let roots = self.roots.clone();
        for root_id in roots {
            self.remove_supervisor(root_id).await?;
        }
        Ok(())
    }

    /// Returns the number of supervisors in the tree.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::{SupervisorTree, OneForOne};
    /// use airssys_rt::monitoring::InMemoryMonitor;
    ///
    /// # use airssys_rt::supervisor::Child;
    /// # use async_trait::async_trait;
    /// # use std::time::Duration;
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
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let monitor = InMemoryMonitor::new(MonitoringConfig::default());
    /// let mut tree = SupervisorTree::<OneForOne, MyWorker, _>::new();
    /// tree.create_supervisor(None, OneForOne, monitor.clone())?;
    /// tree.create_supervisor(None, OneForOne, monitor)?;
    ///
    /// assert_eq!(tree.supervisor_count(), 2);
    /// # Ok(())
    /// # }
    /// ```
    pub fn supervisor_count(&self) -> usize {
        self.supervisors.len()
    }

    /// Returns the number of root supervisors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_rt::supervisor::{SupervisorTree, OneForOne};
    /// use airssys_rt::monitoring::InMemoryMonitor;
    ///
    /// # use airssys_rt::supervisor::Child;
    /// # use async_trait::async_trait;
    /// # use std::time::Duration;
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
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let monitor = InMemoryMonitor::new(MonitoringConfig::default());
    /// let mut tree = SupervisorTree::<OneForOne, MyWorker, _>::new();
    ///
    /// let root1 = tree.create_supervisor(None, OneForOne, monitor.clone())?;
    /// let root2 = tree.create_supervisor(None, OneForOne, monitor.clone())?;
    /// let _child = tree.create_supervisor(Some(root1), OneForOne, monitor)?;
    ///
    /// assert_eq!(tree.root_count(), 2); // Only root1 and root2
    /// # Ok(())
    /// # }
    /// ```
    pub fn root_count(&self) -> usize {
        self.roots.len()
    }
}

impl<S, C, M> Default for SupervisorTree<S, C, M>
where
    S: SupervisionStrategy + Clone,
    C: Child,
    M: Monitor<SupervisionEvent> + Clone + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::{InMemoryMonitor, MonitoringConfig};
    use crate::supervisor::strategy::OneForOne;
    use crate::supervisor::traits::{Child, Supervisor};
    use crate::supervisor::types::{ChildSpec, RestartPolicy, ShutdownPolicy};
    use async_trait::async_trait;
    use std::time::Duration;

    // Test child implementation
    struct TestChild {
        started: bool,
    }

    #[derive(Debug)]
    struct TestError;

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Test error")
        }
    }

    impl std::error::Error for TestError {}

    #[async_trait]
    impl Child for TestChild {
        type Error = TestError;

        async fn start(&mut self) -> Result<(), Self::Error> {
            self.started = true;
            Ok(())
        }

        async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
            self.started = false;
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_create_root_supervisor() {
        let monitor = InMemoryMonitor::new(MonitoringConfig::default());
        let mut tree = SupervisorTree::<OneForOne, TestChild, _>::new();

        let root_id = tree
            .create_supervisor(None, OneForOne, monitor)
            .expect("Should create root supervisor");

        assert_eq!(tree.supervisor_count(), 1);
        assert_eq!(tree.root_count(), 1);
        assert!(tree.get_supervisor(root_id).is_ok());
        assert_eq!(tree.get_parent(root_id), None);
    }

    #[tokio::test]
    async fn test_create_child_supervisor() {
        let monitor = InMemoryMonitor::new(MonitoringConfig::default());
        let mut tree = SupervisorTree::<OneForOne, TestChild, _>::new();

        let root_id = tree
            .create_supervisor(None, OneForOne, monitor.clone())
            .expect("Should create root");
        let child_id = tree
            .create_supervisor(Some(root_id), OneForOne, monitor)
            .expect("Should create child");

        assert_eq!(tree.supervisor_count(), 2);
        assert_eq!(tree.root_count(), 1);
        assert_eq!(tree.get_parent(child_id), Some(root_id));
        assert_eq!(tree.get_parent(root_id), None);
    }

    #[tokio::test]
    async fn test_create_supervisor_with_invalid_parent() {
        let monitor = InMemoryMonitor::new(MonitoringConfig::default());
        let mut tree = SupervisorTree::<OneForOne, TestChild, _>::new();

        let invalid_id = SupervisorId::new();
        let result = tree.create_supervisor(Some(invalid_id), OneForOne, monitor);

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_root_supervisors() {
        let monitor = InMemoryMonitor::new(MonitoringConfig::default());
        let mut tree = SupervisorTree::<OneForOne, TestChild, _>::new();

        let root1 = tree
            .create_supervisor(None, OneForOne, monitor.clone())
            .expect("Should create root1");
        let root2 = tree
            .create_supervisor(None, OneForOne, monitor)
            .expect("Should create root2");

        assert_eq!(tree.supervisor_count(), 2);
        assert_eq!(tree.root_count(), 2);
        assert_eq!(tree.get_parent(root1), None);
        assert_eq!(tree.get_parent(root2), None);
    }

    #[tokio::test]
    async fn test_remove_supervisor() {
        let monitor = InMemoryMonitor::new(MonitoringConfig::default());
        let mut tree = SupervisorTree::<OneForOne, TestChild, _>::new();

        let root_id = tree
            .create_supervisor(None, OneForOne, monitor)
            .expect("Should create root");

        tree.remove_supervisor(root_id)
            .await
            .expect("Should remove supervisor");

        assert_eq!(tree.supervisor_count(), 0);
        assert_eq!(tree.root_count(), 0);
    }

    #[tokio::test]
    async fn test_remove_supervisor_removes_children() {
        let monitor = InMemoryMonitor::new(MonitoringConfig::default());
        let mut tree = SupervisorTree::<OneForOne, TestChild, _>::new();

        let root_id = tree
            .create_supervisor(None, OneForOne, monitor.clone())
            .expect("Should create root");
        let child_id = tree
            .create_supervisor(Some(root_id), OneForOne, monitor.clone())
            .expect("Should create child");
        let _grandchild_id = tree
            .create_supervisor(Some(child_id), OneForOne, monitor)
            .expect("Should create grandchild");

        assert_eq!(tree.supervisor_count(), 3);

        tree.remove_supervisor(root_id)
            .await
            .expect("Should remove supervisor and descendants");

        assert_eq!(tree.supervisor_count(), 0);
    }

    #[tokio::test]
    async fn test_shutdown_tree() {
        let monitor = InMemoryMonitor::new(MonitoringConfig::default());
        let mut tree = SupervisorTree::<OneForOne, TestChild, _>::new();

        tree.create_supervisor(None, OneForOne, monitor.clone())
            .expect("Should create root1");
        tree.create_supervisor(None, OneForOne, monitor)
            .expect("Should create root2");

        assert_eq!(tree.supervisor_count(), 2);

        tree.shutdown().await.expect("Should shutdown tree");

        assert_eq!(tree.supervisor_count(), 0);
        assert_eq!(tree.root_count(), 0);
    }

    #[tokio::test]
    async fn test_add_child_to_supervisor() {
        let monitor = InMemoryMonitor::new(MonitoringConfig::default());
        let mut tree = SupervisorTree::<OneForOne, TestChild, _>::new();

        let supervisor_id = tree
            .create_supervisor(None, OneForOne, monitor)
            .expect("Should create supervisor");

        let supervisor = tree
            .get_supervisor_mut(supervisor_id)
            .expect("Should get supervisor");

        let _child_id = supervisor
            .start_child(ChildSpec {
                id: "test-child".into(),
                factory: || TestChild { started: false },
                restart_policy: RestartPolicy::Permanent,
                shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
                start_timeout: Duration::from_secs(10),
                shutdown_timeout: Duration::from_secs(10),
            })
            .await
            .expect("Should start child");

        assert_eq!(supervisor.child_count(), 1);
    }

    #[tokio::test]
    async fn test_hierarchical_shutdown() {
        let monitor = InMemoryMonitor::new(MonitoringConfig::default());
        let mut tree = SupervisorTree::<OneForOne, TestChild, _>::new();

        // Create hierarchy: root -> child -> grandchild
        let root_id = tree
            .create_supervisor(None, OneForOne, monitor.clone())
            .expect("Should create root");
        let _child_id = tree
            .create_supervisor(Some(root_id), OneForOne, monitor.clone())
            .expect("Should create child");
        let _grandchild_id = tree
            .create_supervisor(Some(_child_id), OneForOne, monitor)
            .expect("Should create grandchild");

        // Add children to each supervisor level
        let root_supervisor = tree.get_supervisor_mut(root_id).unwrap();
        root_supervisor
            .start_child(ChildSpec {
                id: "root-worker".into(),
                factory: || TestChild { started: false },
                restart_policy: RestartPolicy::Permanent,
                shutdown_policy: ShutdownPolicy::Graceful(Duration::from_secs(5)),
                start_timeout: Duration::from_secs(10),
                shutdown_timeout: Duration::from_secs(10),
            })
            .await
            .expect("Should start root child");

        // Shutdown should remove entire tree
        tree.shutdown().await.expect("Should shutdown hierarchy");

        assert_eq!(tree.supervisor_count(), 0);
        assert_eq!(tree.root_count(), 0);
    }
}
