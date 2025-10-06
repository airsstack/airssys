//! Core Actor trait with generic constraints for zero-cost abstractions.
//!
//! This module provides the foundational `Actor` trait that all actors must implement,
//! along with the `ErrorAction` enum for supervision decisions.
//!
//! # Design Philosophy
//!
//! - **Zero-cost abstractions**: Generic constraints instead of trait objects (§6.2)
//! - **Type safety**: Associated types for Message and Error
//! - **Supervision**: ErrorAction enum for fault tolerance decisions
//! - **Lifecycle hooks**: pre_start, post_stop, on_error methods
//!
//! # Example
//!
//! ```rust
//! use airssys_rt::{Actor, ActorContext, ErrorAction, Message};
//! use async_trait::async_trait;
//! use std::fmt;
//!
//! #[derive(Debug, Clone)]
//! struct PingMessage;
//!
//! impl Message for PingMessage {
//!     const MESSAGE_TYPE: &'static str = "ping";
//! }
//!
//! struct PingActor {
//!     count: u32,
//! }
//!
//! #[derive(Debug)]
//! struct PingError;
//!
//! impl fmt::Display for PingError {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         write!(f, "Ping error")
//!     }
//! }
//!
//! impl std::error::Error for PingError {}
//!
//! #[async_trait]
//! impl Actor for PingActor {
//!     type Message = PingMessage;
//!     type Error = PingError;
//!
//!     async fn handle_message(
//!         &mut self,
//!         _message: Self::Message,
//!         _context: &mut ActorContext<Self::Message>,
//!     ) -> Result<(), Self::Error> {
//!         self.count += 1;
//!         Ok(())
//!     }
//! }
//! ```

// Layer 1: Standard library imports
use std::error::Error;

// Layer 2: Third-party crate imports
use async_trait::async_trait;

// Layer 3: Internal module imports
use super::context::ActorContext;
use crate::message::Message;

/// Core Actor trait with generic constraints for zero-cost abstractions.
///
/// All actors must implement this trait to participate in the actor system.
/// The trait uses associated types for `Message` and `Error` to enable
/// compile-time type checking and zero-cost abstractions (§6.2).
///
/// # Associated Types
///
/// - `Message`: The type of messages this actor can handle
/// - `Error`: The error type returned by actor operations
///
/// # Lifecycle Methods
///
/// - `handle_message`: Process incoming messages (REQUIRED)
/// - `pre_start`: Initialize actor before receiving messages (optional)
/// - `post_stop`: Cleanup when actor stops (optional)
/// - `on_error`: Handle errors and return supervision decision (optional)
///
/// # Examples
///
/// ```rust
/// use airssys_rt::{Actor, ActorContext, ErrorAction, Message};
/// use async_trait::async_trait;
/// use std::fmt;
///
/// #[derive(Debug, Clone)]
/// struct CounterMessage { delta: i32 }
///
/// impl Message for CounterMessage {
///     const MESSAGE_TYPE: &'static str = "counter";
/// }
///
/// struct CounterActor { value: i32 }
///
/// #[derive(Debug)]
/// struct CounterError;
///
/// impl fmt::Display for CounterError {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "Counter error")
///     }
/// }
///
/// impl std::error::Error for CounterError {}
///
/// #[async_trait]
/// impl Actor for CounterActor {
///     type Message = CounterMessage;
///     type Error = CounterError;
///
///     async fn handle_message(
///         &mut self,
///         message: Self::Message,
///         _context: &mut ActorContext<Self::Message>,
///     ) -> Result<(), Self::Error> {
///         self.value += message.delta;
///         Ok(())
///     }
///
///     async fn pre_start(
///         &mut self,
///         _context: &mut ActorContext<Self::Message>,
///     ) -> Result<(), Self::Error> {
///         println!("CounterActor starting with value: {}", self.value);
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait Actor: Send + Sync + 'static {
    /// The type of messages this actor can handle.
    type Message: Message;

    /// The error type returned by actor operations.
    type Error: Error + Send + Sync + 'static;

    /// Handle an incoming message.
    ///
    /// This is the core method that processes messages sent to the actor.
    /// Implement your actor's business logic here.
    ///
    /// Generic over broker type B to support dependency injection (ADR-006).
    ///
    /// # Arguments
    ///
    /// * `message` - The message to process
    /// * `context` - Mutable reference to actor context for metadata access
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message processed successfully
    /// * `Err(Self::Error)` - Error occurred, supervisor will call `on_error`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airssys_rt::{Actor, ActorContext, Message};
    /// # use async_trait::async_trait;
    /// # use std::fmt;
    /// #
    /// # #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    /// # struct MyMessage;
    /// # impl Message for MyMessage {
    /// #     const MESSAGE_TYPE: &'static str = "my_message";
    /// # }
    /// #
    /// # #[derive(Debug)]
    /// # struct MyError;
    /// # impl fmt::Display for MyError {
    /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    /// #         write!(f, "My error")
    /// #     }
    /// # }
    /// # impl std::error::Error for MyError {}
    /// #
    /// struct MyActor;
    ///
    /// #[async_trait]
    /// impl Actor for MyActor {
    ///     type Message = MyMessage;
    ///     type Error = MyError;
    ///
    ///     async fn handle_message<B: airssys_rt::broker::MessageBroker<Self::Message>>(
    ///         &mut self,
    ///         _message: Self::Message,
    ///         context: &mut ActorContext<Self::Message, B>,
    ///     ) -> Result<(), Self::Error> {
    ///         println!("Received message at actor: {:?}", context.address());
    ///         Ok(())
    ///     }
    /// }
    /// ```
    async fn handle_message<B: crate::broker::MessageBroker<Self::Message>>(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error>;

    /// Lifecycle hook called before the actor starts receiving messages.
    ///
    /// Use this to initialize resources, establish connections, or perform
    /// any setup required before message processing begins.
    ///
    /// Default implementation does nothing and returns `Ok(())`.
    ///
    /// # Arguments
    ///
    /// * `context` - Mutable reference to actor context
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Initialization successful, actor will start
    /// * `Err(Self::Error)` - Initialization failed, actor will not start
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airssys_rt::{Actor, ActorContext, Message};
    /// # use async_trait::async_trait;
    /// # use std::fmt;
    /// #
    /// # #[derive(Debug, Clone)]
    /// # struct MyMessage;
    /// # impl Message for MyMessage {
    /// #     const MESSAGE_TYPE: &'static str = "my_message";
    /// # }
    /// #
    /// # #[derive(Debug)]
    /// # struct MyError;
    /// # impl fmt::Display for MyError {
    /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    /// #         write!(f, "My error")
    /// #     }
    /// # }
    /// # impl std::error::Error for MyError {}
    /// #
    /// struct DatabaseActor {
    ///     connected: bool,
    /// }
    ///
    /// #[async_trait]
    /// impl Actor for DatabaseActor {
    ///     type Message = MyMessage;
    ///     type Error = MyError;
    ///
    ///     async fn handle_message(
    ///         &mut self,
    ///         _message: Self::Message,
    ///         _context: &mut ActorContext<Self::Message>,
    ///     ) -> Result<(), Self::Error> {
    ///         Ok(())
    ///     }
    ///
    ///     async fn pre_start<B: airssys_rt::broker::MessageBroker<Self::Message>>(
    ///         &mut self,
    ///         _context: &mut ActorContext<Self::Message, B>,
    ///     ) -> Result<(), Self::Error> {
    ///         // Simulate connecting to database
    ///         self.connected = true;
    ///         println!("Database connection established");
    ///         Ok(())
    ///     }
    /// }
    /// ```
    async fn pre_start<B: crate::broker::MessageBroker<Self::Message>>(
        &mut self,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Lifecycle hook called when the actor is stopping.
    ///
    /// Use this to cleanup resources, close connections, or perform
    /// any teardown required when the actor shuts down.
    ///
    /// Default implementation does nothing and returns `Ok(())`.
    ///
    /// Generic over broker type B to support dependency injection (ADR-006).
    ///
    /// # Arguments
    ///
    /// * `context` - Mutable reference to actor context
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Cleanup successful
    /// * `Err(Self::Error)` - Cleanup failed (actor still stops)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airssys_rt::{Actor, ActorContext, Message};
    /// # use async_trait::async_trait;
    /// # use std::fmt;
    /// #
    /// # #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    /// # struct MyMessage;
    /// # impl Message for MyMessage {
    /// #     const MESSAGE_TYPE: &'static str = "my_message";
    /// # }
    /// #
    /// # #[derive(Debug)]
    /// # struct MyError;
    /// # impl fmt::Display for MyError {
    /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    /// #         write!(f, "My error")
    /// #     }
    /// # }
    /// # impl std::error::Error for MyError {}
    /// #
    /// struct FileActor {
    ///     file_open: bool,
    /// }
    ///
    /// #[async_trait]
    /// impl Actor for FileActor {
    ///     type Message = MyMessage;
    ///     type Error = MyError;
    ///
    ///     async fn handle_message<B: airssys_rt::broker::MessageBroker<Self::Message>>(
    ///         &mut self,
    ///         _message: Self::Message,
    ///         _context: &mut ActorContext<Self::Message, B>,
    ///     ) -> Result<(), Self::Error> {
    ///         Ok(())
    ///     }
    ///
    ///     async fn post_stop<B: airssys_rt::broker::MessageBroker<Self::Message>>(
    ///         &mut self,
    ///         _context: &mut ActorContext<Self::Message, B>,
    ///     ) -> Result<(), Self::Error> {
    ///         // Simulate closing file handle
    ///         self.file_open = false;
    ///         println!("File handle closed");
    ///         Ok(())
    ///     }
    /// }
    /// ```
    async fn post_stop<B: crate::broker::MessageBroker<Self::Message>>(
        &mut self,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Handle errors and return a supervision decision.
    ///
    /// This method is called by the supervisor when `handle_message` returns an error.
    /// The returned `ErrorAction` determines how the supervisor handles the failure.
    ///
    /// Default implementation returns `ErrorAction::Stop`.
    ///
    /// Generic over broker type B to support dependency injection (ADR-006).
    ///
    /// # Arguments
    ///
    /// * `error` - The error that occurred
    /// * `context` - Mutable reference to actor context
    ///
    /// # Returns
    ///
    /// An `ErrorAction` indicating how the supervisor should respond:
    /// - `Stop` - Stop the actor permanently
    /// - `Resume` - Continue processing (ignore the error)
    /// - `Restart` - Restart the actor (call pre_start again)
    /// - `Escalate` - Propagate error to parent supervisor
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use airssys_rt::{Actor, ActorContext, ErrorAction, Message};
    /// # use async_trait::async_trait;
    /// # use std::fmt;
    /// #
    /// # #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    /// # struct MyMessage;
    /// # impl Message for MyMessage {
    /// #     const MESSAGE_TYPE: &'static str = "my_message";
    /// # }
    /// #
    /// # #[derive(Debug)]
    /// # struct MyError { recoverable: bool }
    /// # impl fmt::Display for MyError {
    /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    /// #         write!(f, "My error")
    /// #     }
    /// # }
    /// # impl std::error::Error for MyError {}
    /// #
    /// struct ResilientActor {
    ///     retry_count: u32,
    /// }
    ///
    /// #[async_trait]
    /// impl Actor for ResilientActor {
    ///     type Message = MyMessage;
    ///     type Error = MyError;
    ///
    ///     async fn handle_message<B: airssys_rt::broker::MessageBroker<Self::Message>>(
    ///         &mut self,
    ///         _message: Self::Message,
    ///         _context: &mut ActorContext<Self::Message, B>,
    ///     ) -> Result<(), Self::Error> {
    ///         Ok(())
    ///     }
    ///
    ///     async fn on_error<B: airssys_rt::broker::MessageBroker<Self::Message>>(
    ///         &mut self,
    ///         error: Self::Error,
    ///         _context: &mut ActorContext<Self::Message, B>,
    ///     ) -> ErrorAction {
    ///         if error.recoverable && self.retry_count < 3 {
    ///             self.retry_count += 1;
    ///             ErrorAction::Restart
    ///         } else if error.recoverable {
    ///             ErrorAction::Resume
    ///         } else {
    ///             ErrorAction::Escalate
    ///         }
    ///     }
    /// }
    /// ```
    async fn on_error<B: crate::broker::MessageBroker<Self::Message>>(
        &mut self,
        _error: Self::Error,
        _context: &mut ActorContext<Self::Message, B>,
    ) -> ErrorAction {
        ErrorAction::Stop
    }
}

/// Supervision decision returned by the `Actor::on_error` method.
///
/// Determines how the supervisor should handle an actor failure.
///
/// # Variants
///
/// - `Stop` - Permanently stop the actor
/// - `Resume` - Continue processing, ignore the error
/// - `Restart` - Restart the actor (call `pre_start` again)
/// - `Escalate` - Propagate error to parent supervisor
///
/// # Examples
///
/// ```rust
/// use airssys_rt::ErrorAction;
///
/// // Default supervision strategy is to stop
/// assert_eq!(ErrorAction::default(), ErrorAction::Stop);
///
/// // Decide based on error severity
/// fn handle_error(severity: u8) -> ErrorAction {
///     match severity {
///         0..=2 => ErrorAction::Resume,   // Low severity: continue
///         3..=5 => ErrorAction::Restart,  // Medium: restart
///         6..=8 => ErrorAction::Stop,     // High: stop
///         _ => ErrorAction::Escalate,     // Critical: escalate
///     }
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorAction {
    /// Stop the actor permanently.
    ///
    /// The actor will call `post_stop` and then terminate.
    /// No further messages will be processed.
    Stop,

    /// Resume processing, ignoring the error.
    ///
    /// The actor continues running and can process the next message.
    /// Use this for non-critical errors that can be safely ignored.
    Resume,

    /// Restart the actor.
    ///
    /// The actor will call `post_stop`, then `pre_start`, and resume processing.
    /// Use this to recover from transient failures.
    Restart,

    /// Escalate the error to the parent supervisor.
    ///
    /// The parent supervisor will decide how to handle this failure.
    /// Use this for critical errors that require higher-level intervention.
    Escalate,
}

impl Default for ErrorAction {
    /// Default supervision strategy is to stop the actor.
    ///
    /// This conservative default prevents cascading failures by
    /// stopping actors that encounter errors.
    fn default() -> Self {
        Self::Stop
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::broker::InMemoryMessageBroker;
    use crate::util::ActorAddress;
    use serde::{Deserialize, Serialize};
    use std::fmt;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestMessage {
        #[allow(dead_code)]
        content: String,
    }

    impl Message for TestMessage {
        const MESSAGE_TYPE: &'static str = "test";
    }

    #[derive(Debug)]
    struct TestError {
        #[allow(dead_code)]
        message: String,
    }

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "Test error: {}", self.message)
        }
    }

    impl Error for TestError {}

    struct TestActor {
        message_count: u32,
        should_fail: bool,
    }

    #[async_trait]
    impl Actor for TestActor {
        type Message = TestMessage;
        type Error = TestError;

        async fn handle_message<B: crate::broker::MessageBroker<Self::Message>>(
            &mut self,
            _message: Self::Message,
            _context: &mut ActorContext<Self::Message, B>,
        ) -> Result<(), Self::Error> {
            if self.should_fail {
                return Err(TestError {
                    message: "Intentional failure".to_string(),
                });
            }
            self.message_count += 1;
            Ok(())
        }

        async fn pre_start<B: crate::broker::MessageBroker<Self::Message>>(
            &mut self,
            _context: &mut ActorContext<Self::Message, B>,
        ) -> Result<(), Self::Error> {
            self.message_count = 0;
            Ok(())
        }

        async fn post_stop<B: crate::broker::MessageBroker<Self::Message>>(
            &mut self,
            _context: &mut ActorContext<Self::Message, B>,
        ) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn on_error<B: crate::broker::MessageBroker<Self::Message>>(
            &mut self,
            _error: Self::Error,
            _context: &mut ActorContext<Self::Message, B>,
        ) -> ErrorAction {
            if self.message_count < 3 {
                ErrorAction::Restart
            } else {
                ErrorAction::Stop
            }
        }
    }

    #[tokio::test]
    async fn test_actor_handle_message_success() {
        let mut actor = TestActor {
            message_count: 0,
            should_fail: false,
        };
        let address = ActorAddress::anonymous();
        let broker = InMemoryMessageBroker::<TestMessage>::new();
        let mut context = ActorContext::new(address, broker);

        let message = TestMessage {
            content: "test".to_string(),
        };

        let result = actor.handle_message(message, &mut context).await;
        assert!(result.is_ok());
        assert_eq!(actor.message_count, 1);
    }

    #[tokio::test]
    async fn test_actor_handle_message_failure() {
        let mut actor = TestActor {
            message_count: 0,
            should_fail: true,
        };
        let address = ActorAddress::anonymous();
        let broker = InMemoryMessageBroker::<TestMessage>::new();
        let mut context = ActorContext::new(address, broker);

        let message = TestMessage {
            content: "test".to_string(),
        };

        let result = actor.handle_message(message, &mut context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_actor_pre_start() {
        let mut actor = TestActor {
            message_count: 42,
            should_fail: false,
        };
        let address = ActorAddress::anonymous();
        let broker = InMemoryMessageBroker::<TestMessage>::new();
        let mut context = ActorContext::new(address, broker);

        let result = actor.pre_start(&mut context).await;
        assert!(result.is_ok());
        assert_eq!(actor.message_count, 0);
    }

    #[tokio::test]
    async fn test_actor_post_stop() {
        let mut actor = TestActor {
            message_count: 0,
            should_fail: false,
        };
        let address = ActorAddress::anonymous();
        let broker = InMemoryMessageBroker::<TestMessage>::new();
        let mut context = ActorContext::new(address, broker);

        let result = actor.post_stop(&mut context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_actor_on_error_restart() {
        let mut actor = TestActor {
            message_count: 1,
            should_fail: false,
        };
        let address = ActorAddress::anonymous();
        let broker = InMemoryMessageBroker::<TestMessage>::new();
        let mut context = ActorContext::new(address, broker);

        let error = TestError {
            message: "test error".to_string(),
        };

        let action = actor.on_error(error, &mut context).await;
        assert_eq!(action, ErrorAction::Restart);
    }

    #[tokio::test]
    async fn test_actor_on_error_stop() {
        let mut actor = TestActor {
            message_count: 5,
            should_fail: false,
        };
        let address = ActorAddress::anonymous();
        let broker = InMemoryMessageBroker::<TestMessage>::new();
        let mut context = ActorContext::new(address, broker);

        let error = TestError {
            message: "test error".to_string(),
        };

        let action = actor.on_error(error, &mut context).await;
        assert_eq!(action, ErrorAction::Stop);
    }

    #[test]
    fn test_error_action_default() {
        assert_eq!(ErrorAction::default(), ErrorAction::Stop);
    }

    #[test]
    fn test_error_action_equality() {
        assert_eq!(ErrorAction::Stop, ErrorAction::Stop);
        assert_eq!(ErrorAction::Resume, ErrorAction::Resume);
        assert_eq!(ErrorAction::Restart, ErrorAction::Restart);
        assert_eq!(ErrorAction::Escalate, ErrorAction::Escalate);

        assert_ne!(ErrorAction::Stop, ErrorAction::Resume);
        assert_ne!(ErrorAction::Restart, ErrorAction::Escalate);
    }

    #[test]
    fn test_error_action_clone() {
        let action = ErrorAction::Restart;
        let cloned = action;
        assert_eq!(action, cloned);
    }

    #[test]
    fn test_error_action_copy() {
        let action = ErrorAction::Resume;
        let copied = action;
        assert_eq!(action, copied);
    }
}
