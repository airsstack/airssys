//! Actor spawn builder with fluent API.

// Layer 1: Standard library
use std::marker::PhantomData;

// Layer 2: Third-party
// (minimal)

// Layer 3: Internal
use crate::actor::Actor;
use crate::broker::MessageBroker;
use crate::message::Message;
use crate::util::ActorAddress;

use super::{ActorSystem, SystemError};

/// Fluent builder for spawning actors.
///
/// Provides ergonomic API for configuring and spawning actors with
/// compile-time type safety (§6.3 M-DI-HIERARCHY).
///
/// Generic over message type M and broker type B for dependency injection (ADR-006).
///
/// # Type Parameters
///
/// * `M` - The message type used by the actor system
/// * `B` - The broker implementation (injected via ActorSystem)
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_rt::*;
///
/// let address = system.spawn()
///     .with_name("worker")
///     .with_mailbox_capacity(500)
///     .spawn(MyActor::new())
///     .await?;
/// ```
pub struct ActorSpawnBuilder<M: Message, B: MessageBroker<M>> {
    system: ActorSystem<M, B>,
    name: Option<String>,
    mailbox_capacity: Option<usize>,
    supervisor: Option<ActorAddress>, // Reserved for RT-TASK-007
    _marker: PhantomData<M>,
}

impl<M: Message + serde::Serialize, B: MessageBroker<M> + 'static> ActorSpawnBuilder<M, B> {
    /// Create new builder (internal, called by ActorSystem).
    ///
    /// # Arguments
    ///
    /// * `system` - The actor system that will spawn the actor
    pub(crate) fn new(system: ActorSystem<M, B>) -> Self {
        Self {
            system,
            name: None,
            mailbox_capacity: None,
            supervisor: None,
            _marker: PhantomData,
        }
    }

    /// Set actor name for identification.
    ///
    /// Named actors can be looked up by name in the registry.
    ///
    /// # Arguments
    ///
    /// * `name` - The actor name (converted to String)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// system.spawn().with_name("worker-1").spawn(actor).await?;
    /// ```
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set mailbox capacity (creates bounded mailbox).
    ///
    /// Defaults to system config if not specified (typically 1000).
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of messages in mailbox
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// // High-priority actor with larger mailbox
    /// system.spawn().with_mailbox_capacity(5000).spawn(actor).await?;
    /// ```
    pub fn with_mailbox_capacity(mut self, capacity: usize) -> Self {
        self.mailbox_capacity = Some(capacity);
        self
    }

    /// Set supervisor for fault tolerance (reserved for RT-TASK-007).
    ///
    /// NOTE: Supervision will be implemented in RT-TASK-007 Supervisor Framework.
    /// This method exists for API completeness but supervisor assignment is not
    /// yet functional.
    ///
    /// # Arguments
    ///
    /// * `supervisor` - Address of the supervising actor
    pub fn under_supervisor(mut self, supervisor: ActorAddress) -> Self {
        self.supervisor = Some(supervisor);
        self
    }

    /// Spawn the actor with configured settings.
    ///
    /// Returns ActorAddress for communicating with the spawned actor.
    ///
    /// # Type Parameters
    ///
    /// * `A` - The actor type implementing Actor<Message = M>
    ///
    /// # Arguments
    ///
    /// * `actor` - The actor instance to spawn
    ///
    /// # Errors
    ///
    /// Returns `SystemError::ShuttingDown` if system is shutting down.
    /// Returns `SystemError::ActorLimitExceeded` if max actors reached.
    /// Returns `SystemError::SpawnFailed` if actor initialization fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_rt::*;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), SystemError> {
    ///     let system = ActorSystem::<MyMessage>::new(SystemConfig::default());
    ///     
    ///     let address = system.spawn()
    ///         .with_name("worker")
    ///         .with_mailbox_capacity(1000)
    ///         .spawn(MyActor::new())
    ///         .await?;
    ///     
    ///     println!("Spawned actor: {:?}", address);
    ///     Ok(())
    /// }
    /// ```
    pub async fn spawn<A>(self, actor: A) -> Result<ActorAddress, SystemError>
    where
        A: Actor<Message = M> + Send + 'static,
    {
        // Use builder config or system defaults
        let capacity = self
            .mailbox_capacity
            .unwrap_or(self.system.config().default_mailbox_capacity);

        // Call system's internal spawn method (3 parameters: actor, name, capacity)
        let address = self
            .system
            .spawn_actor_internal(actor, self.name.clone(), capacity)
            .await?;

        Ok(address)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::actor::ActorContext;
    use crate::broker::in_memory::InMemoryMessageBroker;
    use crate::message::Message;
    use crate::system::SystemConfig;
    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestMessage;

    impl Message for TestMessage {
        const MESSAGE_TYPE: &'static str = "test";
    }

    struct TestActor;

    #[async_trait]
    impl Actor for TestActor {
        type Message = TestMessage;
        type Error = std::io::Error;

        async fn handle_message<B: crate::broker::MessageBroker<Self::Message>>(
            &mut self,
            _msg: Self::Message,
            _ctx: &mut ActorContext<Self::Message, B>,
        ) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_builder_default_spawn() {
        let config = SystemConfig::default();
        let broker = InMemoryMessageBroker::<TestMessage>::new();
        let system = ActorSystem::new(config, broker);

        let result = system.spawn().spawn(TestActor).await;

        assert!(result.is_ok());
        assert_eq!(system.actor_count(), 1);
    }

    #[tokio::test]
    async fn test_builder_with_name() {
        let config = SystemConfig::default();
        let broker = InMemoryMessageBroker::<TestMessage>::new();
        let system = ActorSystem::new(config, broker);

        let result = system
            .spawn()
            .with_name("test-actor")
            .spawn(TestActor)
            .await;

        assert!(result.is_ok());
        assert_eq!(system.actor_count(), 1);
    }

    #[tokio::test]
    async fn test_builder_with_capacity() {
        let config = SystemConfig::default();
        let broker = InMemoryMessageBroker::<TestMessage>::new();
        let system = ActorSystem::new(config, broker);

        let result = system
            .spawn()
            .with_mailbox_capacity(500)
            .spawn(TestActor)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_builder_with_supervisor() {
        let config = SystemConfig::default();
        let broker = InMemoryMessageBroker::<TestMessage>::new();
        let system = ActorSystem::new(config, broker);

        let supervisor_addr = ActorAddress::named("supervisor");

        let result = system
            .spawn()
            .under_supervisor(supervisor_addr)
            .spawn(TestActor)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_builder_full_configuration() {
        let config = SystemConfig::default();
        let broker = InMemoryMessageBroker::<TestMessage>::new();
        let system = ActorSystem::new(config, broker);

        let result = system
            .spawn()
            .with_name("full-config")
            .with_mailbox_capacity(2000)
            .spawn(TestActor)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_builder_during_shutdown() {
        let config = SystemConfig::default();
        let broker = InMemoryMessageBroker::<TestMessage>::new();
        let system = ActorSystem::new(config, broker);

        // Initiate shutdown
        let _ = system.shutdown().await;

        let result = system
            .spawn()
            .with_name("should-fail")
            .spawn(TestActor)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SystemError::ShuttingDown));
    }

    #[tokio::test]
    async fn test_builder_actor_limit() {
        let config = SystemConfig::builder().with_max_actors(1).build().unwrap();
        let broker = InMemoryMessageBroker::<TestMessage>::new();
        let system = ActorSystem::new(config, broker);

        // First spawn should succeed
        let result1 = system.spawn().spawn(TestActor).await;
        assert!(result1.is_ok());

        // Second spawn should fail
        let result2 = system.spawn().spawn(TestActor).await;
        assert!(result2.is_err());
        assert!(matches!(
            result2.unwrap_err(),
            SystemError::ActorLimitExceeded { .. }
        ));
    }

    #[tokio::test]
    async fn test_builder_chaining() {
        let config = SystemConfig::default();
        let broker = InMemoryMessageBroker::<TestMessage>::new();
        let system = ActorSystem::new(config, broker);

        // Test fluent API chaining
        let builder = system
            .spawn()
            .with_name("chained")
            .with_mailbox_capacity(100);

        let result = builder.spawn(TestActor).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_builder_multiple_actors() {
        let config = SystemConfig::default();
        let broker = InMemoryMessageBroker::<TestMessage>::new();
        let system = ActorSystem::new(config, broker);

        // Spawn multiple actors
        let _addr1 = system
            .spawn()
            .with_name("actor-1")
            .spawn(TestActor)
            .await
            .unwrap();
        let _addr2 = system
            .spawn()
            .with_name("actor-2")
            .spawn(TestActor)
            .await
            .unwrap();
        let _addr3 = system
            .spawn()
            .with_name("actor-3")
            .spawn(TestActor)
            .await
            .unwrap();

        assert_eq!(system.actor_count(), 3);
    }
}
