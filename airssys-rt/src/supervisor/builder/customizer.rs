//! Per-child customization API for batch child spawning.

use super::batch::ChildrenBatchBuilder;
use crate::monitoring::traits::Monitor;
use crate::monitoring::types::SupervisionEvent;
use crate::supervisor::traits::{Child, SupervisionStrategy};
use crate::supervisor::types::{RestartPolicy, ShutdownPolicy};
use std::time::Duration;

/// Builder for customizing individual child configuration within a batch.
pub struct BatchChildCustomizer<'a, S, C, M>
where
    S: SupervisionStrategy,
    C: Child,
    M: Monitor<SupervisionEvent>,
{
    parent: ChildrenBatchBuilder<'a, S, C, M>,
    child_index: usize,
}

impl<'a, S, C, M> BatchChildCustomizer<'a, S, C, M>
where
    S: SupervisionStrategy + Send + Sync,
    C: Child + Send + Sync,
    M: Monitor<SupervisionEvent> + Send + Sync + 'static,
{
    pub(super) fn new(parent: ChildrenBatchBuilder<'a, S, C, M>, child_index: usize) -> Self {
        Self {
            parent,
            child_index,
        }
    }

    #[must_use]
    pub fn restart_policy(mut self, policy: RestartPolicy) -> Self {
        self.parent.children[self.child_index].restart_policy = Some(policy);
        self
    }

    #[must_use]
    pub fn shutdown_policy(mut self, policy: ShutdownPolicy) -> Self {
        self.parent.children[self.child_index].shutdown_policy = Some(policy);
        self
    }

    #[must_use]
    pub fn start_timeout(mut self, timeout: Duration) -> Self {
        self.parent.children[self.child_index].start_timeout = Some(timeout);
        self
    }

    #[must_use]
    pub fn shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.parent.children[self.child_index].shutdown_timeout = Some(timeout);
        self
    }

    #[must_use]
    pub fn done(self) -> ChildrenBatchBuilder<'a, S, C, M> {
        self.parent
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::monitoring::noop::NoopMonitor;
    use crate::supervisor::node::SupervisorNode;
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
    async fn test_customizer_restart_policy_override() {
        let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::new());

        let ids = supervisor
            .children()
            .restart_policy(RestartPolicy::Permanent)
            .child_with("custom", || TestChild::new("custom"))
            .restart_policy(RestartPolicy::Temporary)
            .done()
            .spawn_all()
            .await
            .unwrap();

        assert_eq!(ids.len(), 1);
    }

    #[tokio::test]
    async fn test_customizer_shutdown_policy_override() {
        let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::new());

        let ids = supervisor
            .children()
            .shutdown_policy(ShutdownPolicy::Immediate)
            .child_with("custom", || TestChild::new("custom"))
            .shutdown_policy(ShutdownPolicy::Graceful(Duration::from_secs(10)))
            .done()
            .spawn_all()
            .await
            .unwrap();

        assert_eq!(ids.len(), 1);
    }

    #[tokio::test]
    async fn test_customizer_start_timeout_override() {
        let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::new());

        let ids = supervisor
            .children()
            .start_timeout(Duration::from_secs(30))
            .child_with("custom", || TestChild::new("custom"))
            .start_timeout(Duration::from_secs(60))
            .done()
            .spawn_all()
            .await
            .unwrap();

        assert_eq!(ids.len(), 1);
    }

    #[tokio::test]
    async fn test_customizer_shutdown_timeout_override() {
        let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::new());

        let ids = supervisor
            .children()
            .shutdown_timeout(Duration::from_secs(10))
            .child_with("custom", || TestChild::new("custom"))
            .shutdown_timeout(Duration::from_secs(5))
            .done()
            .spawn_all()
            .await
            .unwrap();

        assert_eq!(ids.len(), 1);
    }

    #[tokio::test]
    async fn test_customizer_done_returns_builder() {
        let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::new());

        let ids = supervisor
            .children()
            .child_with("custom", || TestChild::new("custom"))
            .restart_policy(RestartPolicy::Temporary)
            .done()
            .child("regular", || TestChild::new("regular"))
            .spawn_all()
            .await
            .unwrap();

        assert_eq!(ids.len(), 2);
    }

    #[tokio::test]
    async fn test_customizer_method_chaining() {
        let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::new());

        let ids = supervisor
            .children()
            .child_with("custom", || TestChild::new("custom"))
            .restart_policy(RestartPolicy::Temporary)
            .shutdown_policy(ShutdownPolicy::Immediate)
            .start_timeout(Duration::from_secs(45))
            .shutdown_timeout(Duration::from_secs(15))
            .done()
            .spawn_all()
            .await
            .unwrap();

        assert_eq!(ids.len(), 1);
    }

    #[tokio::test]
    async fn test_multiple_customizers_in_batch() {
        let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::new());

        let ids = supervisor
            .children()
            .restart_policy(RestartPolicy::Permanent)
            .child_with("custom-1", || TestChild::new("custom-1"))
            .restart_policy(RestartPolicy::Temporary)
            .done()
            .child_with("custom-2", || TestChild::new("custom-2"))
            .shutdown_policy(ShutdownPolicy::Immediate)
            .done()
            .child("regular", || TestChild::new("regular"))
            .spawn_all()
            .await
            .unwrap();

        assert_eq!(ids.len(), 3);
    }

    #[tokio::test]
    async fn test_customizer_partial_overrides() {
        let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::new());

        let ids = supervisor
            .children()
            .restart_policy(RestartPolicy::Permanent)
            .shutdown_policy(ShutdownPolicy::Graceful(Duration::from_secs(10)))
            .start_timeout(Duration::from_secs(30))
            .shutdown_timeout(Duration::from_secs(10))
            .child_with("partial", || TestChild::new("partial"))
            .restart_policy(RestartPolicy::Temporary)
            .done()
            .spawn_all()
            .await
            .unwrap();

        assert_eq!(ids.len(), 1);
    }

    #[tokio::test]
    async fn test_customizer_all_fields() {
        let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::new());

        let ids = supervisor
            .children()
            .child_with("all-custom", || TestChild::new("all-custom"))
            .restart_policy(RestartPolicy::Transient)
            .shutdown_policy(ShutdownPolicy::Immediate)
            .start_timeout(Duration::from_secs(90))
            .shutdown_timeout(Duration::from_secs(20))
            .done()
            .spawn_all()
            .await
            .unwrap();

        assert_eq!(ids.len(), 1);
    }

    #[tokio::test]
    async fn test_customizer_with_spawn_all_map() {
        let mut supervisor = SupervisorNode::new(OneForOne, NoopMonitor::new());

        let map = supervisor
            .children()
            .child_with("custom-1", || TestChild::new("custom-1"))
            .restart_policy(RestartPolicy::Temporary)
            .done()
            .child("regular", || TestChild::new("regular"))
            .child_with("custom-2", || TestChild::new("custom-2"))
            .shutdown_timeout(Duration::from_secs(5))
            .done()
            .spawn_all_map()
            .await
            .unwrap();

        assert_eq!(map.len(), 3);
        assert!(map.contains_key("custom-1"));
        assert!(map.contains_key("regular"));
        assert!(map.contains_key("custom-2"));
    }
}
