//! Batch child builder for spawning multiple children with shared configuration.
//!
//! This module provides the [`ChildrenBatchBuilder`] type for efficiently spawning
//! multiple child processes with shared default configuration while allowing
//! per-child customization.

use std::collections::HashMap;
use std::time::Duration;

use crate::monitoring::traits::Monitor;
use crate::monitoring::types::SupervisionEvent;
use crate::supervisor::builder::constants::{
    DEFAULT_RESTART_POLICY, DEFAULT_SHUTDOWN_POLICY, DEFAULT_SHUTDOWN_TIMEOUT,
    DEFAULT_START_TIMEOUT,
};
use crate::supervisor::builder::customizer::BatchChildCustomizer;
use crate::supervisor::node::SupervisorNode;
use crate::supervisor::traits::{Child, SupervisionStrategy, Supervisor};
use crate::supervisor::types::{ChildId, RestartPolicy, ShutdownPolicy};
use crate::supervisor::ChildSpec;
use crate::supervisor::SupervisorError;

/// Internal struct representing a child spec with possible overrides
pub(super) struct BatchChildSpec<C> {
    pub(super) id: String,
    pub(super) factory: Box<dyn Fn() -> C + Send + Sync + 'static>,
    pub(super) restart_policy: Option<RestartPolicy>,
    pub(super) shutdown_policy: Option<ShutdownPolicy>,
    pub(super) start_timeout: Option<Duration>,
    pub(super) shutdown_timeout: Option<Duration>,
}

impl<C> BatchChildSpec<C> {
    fn new(id: String, factory: Box<dyn Fn() -> C + Send + Sync + 'static>) -> Self {
        Self {
            id,
            factory,
            restart_policy: None,
            shutdown_policy: None,
            start_timeout: None,
            shutdown_timeout: None,
        }
    }
}

/// Builder for batching multiple children with shared defaults
pub struct ChildrenBatchBuilder<'a, S, C, M>
where
    S: SupervisionStrategy,
    C: Child,
    M: Monitor<SupervisionEvent>,
{
    supervisor: &'a mut SupervisorNode<S, C, M>,
    shared_restart_policy: RestartPolicy,
    shared_shutdown_policy: ShutdownPolicy,
    shared_start_timeout: Duration,
    shared_shutdown_timeout: Duration,
    pub(super) children: Vec<BatchChildSpec<C>>,
}

impl<'a, S, C, M> ChildrenBatchBuilder<'a, S, C, M>
where
    S: SupervisionStrategy + Send + Sync,
    C: Child + Send + Sync,
    M: Monitor<SupervisionEvent> + Send + Sync + 'static,
{
    pub fn new(supervisor: &'a mut SupervisorNode<S, C, M>) -> Self {
        Self {
            supervisor,
            shared_restart_policy: DEFAULT_RESTART_POLICY,
            shared_shutdown_policy: DEFAULT_SHUTDOWN_POLICY,
            shared_start_timeout: DEFAULT_START_TIMEOUT,
            shared_shutdown_timeout: DEFAULT_SHUTDOWN_TIMEOUT,
            children: Vec::new(),
        }
    }

    pub fn restart_policy(mut self, policy: RestartPolicy) -> Self {
        self.shared_restart_policy = policy;
        self
    }
    pub fn shutdown_policy(mut self, policy: ShutdownPolicy) -> Self {
        self.shared_shutdown_policy = policy;
        self
    }
    pub fn start_timeout(mut self, timeout: Duration) -> Self {
        self.shared_start_timeout = timeout;
        self
    }
    pub fn shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.shared_shutdown_timeout = timeout;
        self
    }
    /// Add a child with shared defaults
    pub fn child<F>(mut self, id: impl Into<String>, factory: F) -> Self
    where
        F: Fn() -> C + Send + Sync + 'static,
    {
        let spec = BatchChildSpec::new(id.into(), Box::new(factory));
        self.children.push(spec);
        self
    }
    /// Add a child with per-child customization
    pub fn child_with<F>(
        mut self,
        id: impl Into<String>,
        factory: F,
    ) -> BatchChildCustomizer<'a, S, C, M>
    where
        F: Fn() -> C + Send + Sync + 'static,
    {
        let spec = BatchChildSpec::new(id.into(), Box::new(factory));
        self.children.push(spec);
        let child_index = self.children.len() - 1;
        BatchChildCustomizer::new(self, child_index)
    }
    /// Spawn all children, fail-fast on error
    pub async fn spawn_all(self) -> Result<Vec<ChildId>, SupervisorError> {
        let mut ids = Vec::new();
        for child in self.children {
            let spec = ChildSpec {
                id: child.id,
                factory: child.factory,
                restart_policy: child.restart_policy.unwrap_or(self.shared_restart_policy),
                shutdown_policy: child.shutdown_policy.unwrap_or(self.shared_shutdown_policy),
                start_timeout: child.start_timeout.unwrap_or(self.shared_start_timeout),
                shutdown_timeout: child
                    .shutdown_timeout
                    .unwrap_or(self.shared_shutdown_timeout),
            };
            let id = self.supervisor.start_child(spec).await?;
            ids.push(id);
        }
        Ok(ids)
    }

    /// Spawn all children, return HashMap of id to ChildId
    pub async fn spawn_all_map(self) -> Result<HashMap<String, ChildId>, SupervisorError> {
        let mut map = HashMap::new();
        for child in self.children {
            let id = child.id.clone();
            let spec = ChildSpec {
                id: child.id,
                factory: child.factory,
                restart_policy: child.restart_policy.unwrap_or(self.shared_restart_policy),
                shutdown_policy: child.shutdown_policy.unwrap_or(self.shared_shutdown_policy),
                start_timeout: child.start_timeout.unwrap_or(self.shared_start_timeout),
                shutdown_timeout: child
                    .shutdown_timeout
                    .unwrap_or(self.shared_shutdown_timeout),
            };
            let child_id = self.supervisor.start_child(spec).await?;
            map.insert(id, child_id);
        }
        Ok(map)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::monitoring::noop::NoopMonitor;
    use crate::supervisor::strategy::OneForOne;
    use async_trait::async_trait;

    #[derive(Clone)]
    struct TestChild;

    impl TestChild {
        fn new(_id: &str) -> Self {
            Self
        }
    }

    #[async_trait]
    impl Child for TestChild {
        type Error = std::io::Error;

        async fn start(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn stop(&mut self, _timeout: Duration) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_batch_builder_creation() {
        let mut supervisor: SupervisorNode<OneForOne, TestChild, NoopMonitor<SupervisionEvent>> =
            SupervisorNode::new(OneForOne, NoopMonitor::new());
        let builder = supervisor.children();

        assert_eq!(builder.children.len(), 0);
        assert_eq!(builder.shared_restart_policy, DEFAULT_RESTART_POLICY);
        assert_eq!(builder.shared_shutdown_policy, DEFAULT_SHUTDOWN_POLICY);
    }

    #[tokio::test]
    async fn test_batch_shared_defaults() {
        let mut supervisor: SupervisorNode<OneForOne, TestChild, NoopMonitor<SupervisionEvent>> =
            SupervisorNode::new(OneForOne, NoopMonitor::new());

        let ids = supervisor
            .children()
            .restart_policy(RestartPolicy::Temporary)
            .child("child-1", || TestChild::new("child-1"))
            .child("child-2", || TestChild::new("child-2"))
            .child("child-3", || TestChild::new("child-3"))
            .spawn_all()
            .await
            .unwrap();

        assert_eq!(ids.len(), 3);
        assert_eq!(supervisor.child_count(), 3);
    }

    #[tokio::test]
    async fn test_batch_spawn_all_returns_vec() {
        let mut supervisor: SupervisorNode<OneForOne, TestChild, NoopMonitor<SupervisionEvent>> =
            SupervisorNode::new(OneForOne, NoopMonitor::new());

        let ids = supervisor
            .children()
            .child("worker-1", || TestChild::new("worker-1"))
            .child("worker-2", || TestChild::new("worker-2"))
            .spawn_all()
            .await
            .unwrap();

        assert_eq!(ids.len(), 2);
        for id in &ids {
            assert!(supervisor.get_child(id).is_some());
        }
    }

    #[tokio::test]
    async fn test_batch_spawn_all_map_returns_hashmap() {
        let mut supervisor: SupervisorNode<OneForOne, TestChild, NoopMonitor<SupervisionEvent>> =
            SupervisorNode::new(OneForOne, NoopMonitor::new());

        let map = supervisor
            .children()
            .child("worker-1", || TestChild::new("worker-1"))
            .child("worker-2", || TestChild::new("worker-2"))
            .spawn_all_map()
            .await
            .unwrap();

        assert_eq!(map.len(), 2);
        assert!(map.contains_key("worker-1"));
        assert!(map.contains_key("worker-2"));

        let child_id = map.get("worker-1").unwrap();
        assert!(supervisor.get_child(child_id).is_some());
    }

    #[tokio::test]
    async fn test_batch_with_customizer() {
        let mut supervisor: SupervisorNode<OneForOne, TestChild, NoopMonitor<SupervisionEvent>> =
            SupervisorNode::new(OneForOne, NoopMonitor::new());

        let ids = supervisor
            .children()
            .restart_policy(RestartPolicy::Permanent)
            .child("regular-1", || TestChild::new("regular-1"))
            .child_with("custom", || TestChild::new("custom"))
            .restart_policy(RestartPolicy::Temporary)
            .done()
            .child("regular-2", || TestChild::new("regular-2"))
            .spawn_all()
            .await
            .unwrap();

        assert_eq!(ids.len(), 3);
    }

    #[tokio::test]
    async fn test_batch_all_policies() {
        let mut supervisor: SupervisorNode<OneForOne, TestChild, NoopMonitor<SupervisionEvent>> =
            SupervisorNode::new(OneForOne, NoopMonitor::new());

        let ids = supervisor
            .children()
            .restart_policy(RestartPolicy::Transient)
            .shutdown_policy(ShutdownPolicy::Immediate)
            .start_timeout(Duration::from_secs(60))
            .shutdown_timeout(Duration::from_secs(20))
            .child("worker-1", || TestChild::new("worker-1"))
            .child("worker-2", || TestChild::new("worker-2"))
            .spawn_all()
            .await
            .unwrap();

        assert_eq!(ids.len(), 2);
    }

    #[tokio::test]
    async fn test_batch_empty_spawn() {
        let mut supervisor: SupervisorNode<OneForOne, TestChild, NoopMonitor<SupervisionEvent>> =
            SupervisorNode::new(OneForOne, NoopMonitor::new());

        let ids = supervisor.children().spawn_all().await.unwrap();

        assert_eq!(ids.len(), 0);
    }

    #[tokio::test]
    async fn test_batch_customizer_chaining() {
        let mut supervisor: SupervisorNode<OneForOne, TestChild, NoopMonitor<SupervisionEvent>> =
            SupervisorNode::new(OneForOne, NoopMonitor::new());

        let ids = supervisor
            .children()
            .child_with("custom-1", || TestChild::new("custom-1"))
            .restart_policy(RestartPolicy::Temporary)
            .shutdown_policy(ShutdownPolicy::Immediate)
            .start_timeout(Duration::from_secs(45))
            .shutdown_timeout(Duration::from_secs(15))
            .done()
            .child("regular", || TestChild::new("regular"))
            .spawn_all()
            .await
            .unwrap();

        assert_eq!(ids.len(), 2);
    }

    #[tokio::test]
    async fn test_batch_multiple_customizers() {
        let mut supervisor: SupervisorNode<OneForOne, TestChild, NoopMonitor<SupervisionEvent>> =
            SupervisorNode::new(OneForOne, NoopMonitor::new());

        let ids = supervisor
            .children()
            .restart_policy(RestartPolicy::Permanent)
            .child_with("custom-1", || TestChild::new("custom-1"))
            .restart_policy(RestartPolicy::Temporary)
            .done()
            .child("regular", || TestChild::new("regular"))
            .child_with("custom-2", || TestChild::new("custom-2"))
            .restart_policy(RestartPolicy::Transient)
            .done()
            .spawn_all()
            .await
            .unwrap();

        assert_eq!(ids.len(), 3);
    }

    #[tokio::test]
    async fn test_batch_spawn_all_map_empty() {
        let mut supervisor: SupervisorNode<OneForOne, TestChild, NoopMonitor<SupervisionEvent>> =
            SupervisorNode::new(OneForOne, NoopMonitor::new());

        let map = supervisor.children().spawn_all_map().await.unwrap();

        assert_eq!(map.len(), 0);
    }

    #[tokio::test]
    async fn test_batch_large_batch() {
        let mut supervisor: SupervisorNode<OneForOne, TestChild, NoopMonitor<SupervisionEvent>> =
            SupervisorNode::new(OneForOne, NoopMonitor::new());

        let mut builder = supervisor.children();
        for i in 0..10 {
            builder = builder.child(format!("worker-{i}"), move || {
                TestChild::new(&format!("worker-{i}"))
            });
        }

        let ids = builder.spawn_all().await.unwrap();

        assert_eq!(ids.len(), 10);
        assert_eq!(supervisor.child_count(), 10);
    }

    #[tokio::test]
    async fn test_batch_mixed_defaults_and_custom() {
        let mut supervisor: SupervisorNode<OneForOne, TestChild, NoopMonitor<SupervisionEvent>> =
            SupervisorNode::new(OneForOne, NoopMonitor::new());

        let map = supervisor
            .children()
            .restart_policy(RestartPolicy::Permanent)
            .shutdown_policy(ShutdownPolicy::Graceful(Duration::from_secs(10)))
            .child("default-1", || TestChild::new("default-1"))
            .child_with("custom", || TestChild::new("custom"))
            .restart_policy(RestartPolicy::Temporary)
            .shutdown_timeout(Duration::from_secs(5))
            .done()
            .child("default-2", || TestChild::new("default-2"))
            .spawn_all_map()
            .await
            .unwrap();

        assert_eq!(map.len(), 3);
        assert!(map.contains_key("default-1"));
        assert!(map.contains_key("custom"));
        assert!(map.contains_key("default-2"));
    }
}
