// Layer 1: Standard library imports
use std::fmt::Debug;

// Layer 2: Third-party crate imports
// (none for this module)

// Layer 3: Internal module imports
// (none yet)

/// Core message trait with compile-time type identification
/// 
/// # Zero-Cost Abstraction
/// Uses const MESSAGE_TYPE instead of runtime reflection for maximum performance.
/// All message types are resolved at compile time.
///
/// # Design Principles
/// - **Type Safety**: Compile-time message type verification
/// - **Zero Overhead**: No runtime type checking or reflection
/// - **Flexibility**: Support for custom priority levels per message type
///
/// # Example
/// ```rust
/// use airssys_rt::message::{Message, MessagePriority};
/// 
/// #[derive(Debug, Clone)]
/// struct MyMessage {
///     data: String,
/// }
///
/// impl Message for MyMessage {
///     const MESSAGE_TYPE: &'static str = "my_message";
///     
///     fn priority(&self) -> MessagePriority {
///         MessagePriority::High
///     }
/// }
/// ```
pub trait Message: Send + Sync + Clone + Debug + 'static {
    /// Unique message type identifier (compile-time constant)
    /// 
    /// This const allows message type identification without runtime reflection,
    /// enabling zero-cost message routing and handling.
    const MESSAGE_TYPE: &'static str;
    
    /// Message routing priority (default: Normal)
    /// 
    /// Override this method to provide custom priority levels for specific
    /// message types. Higher priority messages are processed before lower
    /// priority messages in the actor mailbox.
    fn priority(&self) -> MessagePriority {
        MessagePriority::Normal
    }
}

/// Message priority levels for routing and processing
/// 
/// Defines the relative importance of messages for mailbox processing order.
/// Messages are processed in priority order, with higher priority messages
/// being handled before lower priority ones.
///
/// # Priority Ordering
/// Critical > High > Normal > Low
///
/// # Example
/// ```rust
/// use airssys_rt::message::MessagePriority;
/// 
/// assert!(MessagePriority::Critical > MessagePriority::High);
/// assert!(MessagePriority::High > MessagePriority::Normal);
/// assert!(MessagePriority::Normal > MessagePriority::Low);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MessagePriority {
    /// Background processing (lowest priority)
    /// 
    /// Use for non-critical maintenance tasks, cleanup operations,
    /// or analytics that can be deferred.
    Low = 0,
    
    /// Default priority for normal messages
    /// 
    /// Standard priority for routine business logic and operations.
    Normal = 1,
    
    /// High priority for important messages
    /// 
    /// Use for time-sensitive operations or user-facing requests
    /// that should be handled promptly.
    High = 2,
    
    /// Highest priority for critical system messages
    /// 
    /// Reserved for system-critical operations like shutdown signals,
    /// supervisor commands, or health check responses.
    Critical = 3,
}

impl Default for MessagePriority {
    fn default() -> Self {
        Self::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    struct TestMessage {
        content: String,
    }
    
    impl Message for TestMessage {
        const MESSAGE_TYPE: &'static str = "test_message";
    }
    
    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    struct HighPriorityMessage {
        data: u64,
    }
    
    impl Message for HighPriorityMessage {
        const MESSAGE_TYPE: &'static str = "high_priority_message";
        
        fn priority(&self) -> MessagePriority {
            MessagePriority::High
        }
    }
    
    #[test]
    fn test_message_type_const() {
        assert_eq!(TestMessage::MESSAGE_TYPE, "test_message");
        assert_eq!(HighPriorityMessage::MESSAGE_TYPE, "high_priority_message");
    }
    
    #[test]
    fn test_default_priority() {
        let msg = TestMessage { 
            content: "test".to_string() 
        };
        assert_eq!(msg.priority(), MessagePriority::Normal);
    }
    
    #[test]
    fn test_custom_priority() {
        let msg = HighPriorityMessage { data: 42 };
        assert_eq!(msg.priority(), MessagePriority::High);
    }
    
    #[test]
    fn test_priority_ordering() {
        assert!(MessagePriority::Critical > MessagePriority::High);
        assert!(MessagePriority::High > MessagePriority::Normal);
        assert!(MessagePriority::Normal > MessagePriority::Low);
    }
    
    #[test]
    fn test_priority_default() {
        assert_eq!(MessagePriority::default(), MessagePriority::Normal);
    }
    
    #[test]
    fn test_priority_equality() {
        assert_eq!(MessagePriority::Normal, MessagePriority::Normal);
        assert_ne!(MessagePriority::High, MessagePriority::Low);
    }
    
    #[test]
    fn test_priority_ordering_transitive() {
        // Verify transitive property: if A > B and B > C, then A > C
        assert!(MessagePriority::Critical > MessagePriority::Normal);
        assert!(MessagePriority::High > MessagePriority::Low);
    }
    
    #[test]
    fn test_message_trait_bounds() {
        // Verify that Message trait enforces required bounds
        fn assert_message<M: Message>() {}
        
        assert_message::<TestMessage>();
        assert_message::<HighPriorityMessage>();
    }
}
