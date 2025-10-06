//! Main actor system implementation with pub-sub architecture.

// Layer 1: Standard library
use std::collections::HashMap;
use std::sync::Arc;

// Layer 2: Third-party
use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::task::{spawn, JoinHandle};
use tokio::time::{sleep, timeout};

// Layer 3: Internal
use super::{builder::ActorSpawnBuilder, SystemConfig, SystemError};
use crate::actor::{Actor, ActorContext, ErrorAction};
use crate::broker::MessageBroker;
use crate::message::{Message, MessageEnvelope};
use crate::util::{ActorAddress, ActorId};

/// System state enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SystemState {
    /// System is running normally
    Running,
    /// Graceful shutdown in progress
    ShuttingDown,
    /// System has stopped
    Stopped,
}

/// Internal actor metadata.
#[allow(dead_code)] // Fields reserved for future actor management features
struct ActorMetadata<M: Message> {
    id: ActorId,
    address: ActorAddress,
    name: Option<String>,
    spawned_at: DateTime<Utc>,
    mailbox_sender: UnboundedSender<MessageEnvelope<M>>,
    task_handle: JoinHandle<()>,
}

/// Main actor system managing actor lifecycle.
///
/// The system is generic over the message type, mailbox sender, and broker
/// implementation, following dependency injection pattern (ADR-006).
///
/// # Architecture: Pub-Sub Integration (ADR-006)
///
/// ```text
/// Actor → ActorContext.send() → Broker.publish() → ActorSystem (subscriber)
///                                                        ↓
///                                                  Routes to actor
///                                                        ↓
///                                                  Mailbox → Actor
/// ```
///
/// # Type Parameters
///
/// * `M` - The message type used by all actors in this system
/// * `B` - The message broker implementation (injected via dependency injection)
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_rt::*;
///
/// #[tokio::main]
/// async fn main() -> Result<(), SystemError> {
///     // Create broker and inject into system
///     let broker = InMemoryMessageBroker::new();
///     let system = ActorSystem::new(SystemConfig::default(), broker);
///     
///     // All actors use the injected broker
///     let addr = system.spawn()
///         .with_name("worker")
///         .spawn(my_actor)
///         .await?;
///     
///     system.shutdown().await?;
///     Ok(())
/// }
/// ```
pub struct ActorSystem<M: Message, B: MessageBroker<M>> {
    pub(crate) inner: Arc<ActorSystemInner<M, B>>,
}

pub(crate) struct ActorSystemInner<M: Message, B: MessageBroker<M>> {
    pub(crate) config: SystemConfig,
    pub(crate) broker: B, // Dependency injection (ADR-006 §6.2 compliance)
    actors: RwLock<HashMap<ActorAddress, ActorMetadata<M>>>,
    pub(crate) state: RwLock<SystemState>,
    router_handle: RwLock<Option<JoinHandle<()>>>,
}

impl<M: Message + serde::Serialize, B: MessageBroker<M> + Clone + Send + Sync + 'static>
    ActorSystem<M, B>
{
    /// Get the system configuration.
    pub fn config(&self) -> &SystemConfig {
        &self.inner.config
    }

    /// Create a new actor system with dependency injection.
    ///
    /// # Arguments
    ///
    /// * `config` - System configuration
    /// * `broker` - Message broker implementation (injected)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let broker = InMemoryMessageBroker::new();
    /// let system = ActorSystem::new(SystemConfig::default(), broker);
    /// ```
    pub fn new(config: SystemConfig, broker: B) -> Self {
        let inner = Arc::new(ActorSystemInner {
            config,
            broker,
            actors: RwLock::new(HashMap::new()),
            state: RwLock::new(SystemState::Running),
            router_handle: RwLock::new(None),
        });

        // Start router task
        let inner_clone = Arc::clone(&inner);
        let router_handle = spawn(async move {
            Self::router_task(inner_clone).await;
        });

        *inner.router_handle.write() = Some(router_handle);

        Self { inner }
    }

    /// Router task: subscribes to broker and routes messages to actors.
    async fn router_task(inner: Arc<ActorSystemInner<M, B>>) {
        // Subscribe to broker
        let mut stream = match inner.broker.subscribe().await {
            Ok(s) => s,
            Err(_) => return,
        };

        // Route messages to actors
        while let Some(envelope) = stream.recv().await {
            // Check if system is shutting down
            if *inner.state.read() != SystemState::Running {
                break;
            }

            // Route to target actor
            if let Some(target) = &envelope.reply_to {
                let actors = inner.actors.read();
                if let Some(metadata) = actors.get(target) {
                    // Send to actor's mailbox (ignore if mailbox closed)
                    let _ = metadata.mailbox_sender.send(envelope);
                }
                // If actor not found, message is dropped (dead letter in future)
            }
        }
    }

    /// Get the number of active actors.
    pub fn actor_count(&self) -> usize {
        self.inner.actors.read().len()
    }

    /// Check if system is shutting down.
    pub fn is_shutting_down(&self) -> bool {
        *self.inner.state.read() != SystemState::Running
    }

    /// Gracefully shutdown the system.
    ///
    /// Waits for all actors to finish processing before returning.
    pub async fn shutdown(&self) -> Result<(), SystemError> {
        // Set shutting down state
        {
            let mut state = self.inner.state.write();
            if *state != SystemState::Running {
                return Err(SystemError::ShuttingDown);
            }
            *state = SystemState::ShuttingDown;
        }

        // Stop router task
        if let Some(handle) = self.inner.router_handle.write().take() {
            handle.abort();
        }

        // Wait for all actors to finish (with timeout)
        let timeout_duration = self.inner.config.shutdown_timeout;
        let result = timeout(timeout_duration, self.wait_for_actors()).await;

        match result {
            Ok(()) => {
                *self.inner.state.write() = SystemState::Stopped;
                Ok(())
            }
            Err(_) => Err(SystemError::ShutdownTimeout(timeout_duration)),
        }
    }

    /// Wait for all actors to finish.
    async fn wait_for_actors(&self) {
        loop {
            let actor_count = self.inner.actors.read().len();

            if actor_count == 0 {
                break;
            }

            // Wait a bit
            sleep(std::time::Duration::from_millis(10)).await;
        }
    }

    /// Force immediate shutdown without waiting.
    pub async fn force_shutdown(&self) {
        *self.inner.state.write() = SystemState::Stopped;

        // Stop router
        if let Some(handle) = self.inner.router_handle.write().take() {
            handle.abort();
        }

        // Abort all actor tasks
        let mut actors = self.inner.actors.write();
        for metadata in actors.values() {
            metadata.task_handle.abort();
        }
        actors.clear();
    }

    /// Internal: Spawn actor with full configuration.
    ///
    /// Called by ActorSpawnBuilder.
    pub(crate) async fn spawn_actor_internal<A>(
        &self,
        actor: A,
        name: Option<String>,
        _mailbox_capacity: usize,
    ) -> Result<ActorAddress, SystemError>
    where
        A: Actor<Message = M> + Send + 'static,
    {
        // Check if shutting down
        if self.is_shutting_down() {
            return Err(SystemError::ShuttingDown);
        }

        // Check actor limit
        if self.inner.config.max_actors > 0 {
            let current = self.actor_count();
            if current >= self.inner.config.max_actors {
                return Err(SystemError::ActorLimitExceeded {
                    current,
                    max: self.inner.config.max_actors,
                });
            }
        }

        // Create actor ID and address
        let actor_id = ActorId::new();
        let address = if let Some(ref n) = name {
            ActorAddress::named(n)
        } else {
            ActorAddress::Anonymous { id: actor_id }
        };

        // Create unbounded mailbox (bounded not yet supported in pub-sub)
        let (mailbox_sender, mailbox_receiver) = unbounded_channel();

        // Create actor context
        let context = ActorContext::new(address.clone(), self.inner.broker.clone());

        // Spawn actor task
        let task_handle = self.spawn_actor_task(actor, mailbox_receiver, context);

        // Store metadata
        let metadata = ActorMetadata {
            id: actor_id,
            address: address.clone(),
            name,
            spawned_at: Utc::now(),
            mailbox_sender,
            task_handle,
        };

        self.inner.actors.write().insert(address.clone(), metadata);

        Ok(address)
    }

    /// Spawn the actor task.
    fn spawn_actor_task<A>(
        &self,
        mut actor: A,
        mut mailbox_receiver: UnboundedReceiver<MessageEnvelope<M>>,
        mut context: ActorContext<M, B>,
    ) -> JoinHandle<()>
    where
        A: Actor<Message = M> + Send + 'static,
    {
        spawn(async move {
            // Call pre_start lifecycle hook
            if let Err(error) = actor.pre_start(&mut context).await {
                let action = actor.on_error(error, &mut context).await;
                match action {
                    ErrorAction::Stop | ErrorAction::Restart => return,
                    ErrorAction::Escalate => return, // TODO: escalate to supervisor
                    ErrorAction::Resume => {}        // Continue with message processing
                }
            }

            // Actor message loop
            while let Some(envelope) = mailbox_receiver.recv().await {
                let message = envelope.payload;

                match actor.handle_message(message, &mut context).await {
                    Ok(()) => {
                        // Message handled successfully
                    }
                    Err(error) => {
                        let action = actor.on_error(error, &mut context).await;
                        match action {
                            ErrorAction::Stop => {
                                // Actor requested stop
                                break;
                            }
                            ErrorAction::Restart => {
                                // TODO: Implement restart logic with supervisor
                                break;
                            }
                            ErrorAction::Escalate => {
                                // TODO: Implement escalation to supervisor
                                break;
                            }
                            ErrorAction::Resume => {
                                // Continue processing next message
                                continue;
                            }
                        }
                    }
                }
            }

            // Call post_stop lifecycle hook
            let _ = actor.post_stop(&mut context).await;
        })
    }

    /// Create a builder for spawning actors.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let address = system.spawn()
    ///     .with_name("worker")
    ///     .with_mailbox_capacity(1000)
    ///     .spawn(my_actor)
    ///     .await?;
    /// ```
    pub fn spawn(&self) -> ActorSpawnBuilder<M, B> {
        ActorSpawnBuilder::new(self.clone())
    }
}

impl<M: Message, B: MessageBroker<M>> Clone for ActorSystem<M, B> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::broker::in_memory::InMemoryMessageBroker;
    use crate::message::MessagePriority;

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    struct TestMessage {
        data: String,
    }

    impl Message for TestMessage {
        const MESSAGE_TYPE: &'static str = "test";

        fn priority(&self) -> MessagePriority {
            MessagePriority::Normal
        }
    }

    struct TestActor;

    #[async_trait::async_trait]
    impl Actor for TestActor {
        type Message = TestMessage;
        type Error = std::io::Error;

        async fn handle_message<B: crate::broker::MessageBroker<Self::Message>>(
            &mut self,
            _message: Self::Message,
            _context: &mut ActorContext<Self::Message, B>,
        ) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_actor_system_creation() {
        let broker = InMemoryMessageBroker::<TestMessage>::new();
        let system = ActorSystem::new(SystemConfig::default(), broker);
        assert_eq!(system.actor_count(), 0);
        assert!(!system.is_shutting_down());
    }

    #[tokio::test]
    async fn test_actor_spawn() {
        let broker = InMemoryMessageBroker::<TestMessage>::new();
        let system = ActorSystem::new(SystemConfig::default(), broker);

        let address = system
            .spawn_actor_internal(TestActor, Some("test".to_string()), 100)
            .await
            .unwrap();

        assert_eq!(system.actor_count(), 1);

        if let ActorAddress::Named { name, .. } = address {
            assert_eq!(name, "test");
        } else {
            panic!("Expected named address");
        }
    }

    #[tokio::test]
    async fn test_system_shutdown() {
        let broker = InMemoryMessageBroker::<TestMessage>::new();
        let system = ActorSystem::new(SystemConfig::default(), broker);

        // Test shutdown without actors (should succeed immediately)
        assert!(system.shutdown().await.is_ok());
        assert!(system.is_shutting_down());
    }

    #[tokio::test]
    async fn test_force_shutdown() {
        let broker = InMemoryMessageBroker::<TestMessage>::new();
        let system = ActorSystem::new(SystemConfig::default(), broker);

        let _addr = system
            .spawn_actor_internal(TestActor, None, 100)
            .await
            .unwrap();

        // Force shutdown should complete immediately
        system.force_shutdown().await;
        assert_eq!(system.actor_count(), 0);
    }
}
