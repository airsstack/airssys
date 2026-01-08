// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)
// None needed for this module

// Layer 2: Third-party crate imports (per PROJECTS_STANDARD.md §2.1)
// None needed for this module

// Layer 3: Internal module imports (per PROJECTS_STANDARD.md §2.1)
use super::id::ComponentId;

/// Message payload placeholder.
///
/// This is a temporary placeholder type for MessagePayload.
/// The real MessagePayload will be implemented in `core/messaging/payload.rs`
/// (future task: WASM-TASK-0XX).
///
/// For now, we use `Vec<u8>` as a simple placeholder to enable
/// ComponentMessage to compile and be tested.
///
/// # Architecture Note
///
/// The placeholder allows `core/component/` to be implemented without
/// waiting for `core/messaging/` module. Once `core/messaging/payload.rs`
/// is created, this type alias will be replaced with the real MessagePayload
/// type.
pub type MessagePayload = Vec<u8>;

/// Metadata for a component message.
///
/// MessageMetadata contains optional information about a message that supports
/// request-response correlation, reply routing, and message type identification.
///
/// # Fields
///
/// - `correlation_id`: Optional correlation identifier for request-response patterns
/// - `reply_to`: Optional ComponentId to which responses should be routed
/// - `timestamp_ms`: Message creation timestamp in milliseconds since Unix epoch
/// - `content_type`: Optional MIME type or content identifier for message payload
///
/// # Architecture Note
///
/// MessageMetadata lives in `core/component/` (Layer 1) as a pure data structure.
/// It is used by all messaging-related modules to provide consistent message metadata.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::component::MessageMetadata;
///
/// // Default metadata (all fields None or 0)
/// let metadata = MessageMetadata::default();
/// assert!(metadata.correlation_id.is_none());
/// assert!(metadata.reply_to.is_none());
/// assert_eq!(metadata.timestamp_ms, 0);
/// assert!(metadata.content_type.is_none());
/// ```
#[derive(Debug, Clone)]
pub struct MessageMetadata {
    /// Optional correlation identifier for request-response patterns
    pub correlation_id: Option<String>,
    /// Optional component ID to which responses should be routed
    pub reply_to: Option<ComponentId>,
    /// Message creation timestamp in milliseconds since Unix epoch
    pub timestamp_ms: u64,
    /// Optional MIME type or content identifier for payload
    pub content_type: Option<String>,
}

impl Default for MessageMetadata {
    /// Creates default MessageMetadata with all optional fields set to None
    /// and timestamp_ms set to 0.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::component::MessageMetadata;
    ///
    /// let metadata = MessageMetadata::default();
    /// assert!(metadata.correlation_id.is_none());
    /// assert!(metadata.reply_to.is_none());
    /// assert_eq!(metadata.timestamp_ms, 0);
    /// assert!(metadata.content_type.is_none());
    /// ```
    fn default() -> Self {
        Self {
            correlation_id: None,
            reply_to: None,
            timestamp_ms: 0,
            content_type: None,
        }
    }
}

/// Complete message envelope for component communication.
///
/// ComponentMessage wraps a payload with metadata and sender information,
/// forming a complete message envelope that can be routed between components.
///
/// # Fields
///
/// - `sender`: ComponentId of the message sender
/// - `payload`: Message payload (placeholder: Vec<u8>)
/// - `metadata`: Message metadata (correlation, reply routing, timestamp, content type)
///
/// # Architecture Note
///
/// ComponentMessage lives in `core/component/` (Layer 1) as a pure data structure.
/// It is the foundational message type used by all inter-component communication.
/// The `payload` field will be updated to use real MessagePayload type once
/// `core/messaging/payload.rs` is implemented.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::component::{ComponentId, ComponentMessage, MessageMetadata};
///
/// let sender = ComponentId::new("system", "database", "prod");
/// let payload = vec![1, 2, 3, 4];
/// let metadata = MessageMetadata::default();
///
/// let message = ComponentMessage::new(sender, payload, metadata);
///
/// assert_eq!(message.sender.to_string_id(), "system/database/prod");
/// assert_eq!(message.payload, vec![1, 2, 3, 4]);
/// ```
#[derive(Debug, Clone)]
pub struct ComponentMessage {
    /// ComponentId of the message sender
    pub sender: ComponentId,
    /// Message payload (placeholder: Vec<u8>, will be MessagePayload)
    pub payload: MessagePayload,
    /// Message metadata
    pub metadata: MessageMetadata,
}

impl ComponentMessage {
    /// Creates a new ComponentMessage with sender, payload, and metadata.
    ///
    /// # Arguments
    ///
    /// * `sender` - ComponentId of the message sender
    /// * `payload` - Message payload (Vec<u8> placeholder)
    /// * `metadata` - Message metadata
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::component::{ComponentId, ComponentMessage, MessageMetadata};
    ///
    /// let sender = ComponentId::new("system", "database", "prod");
    /// let payload = vec![1, 2, 3];
    /// let metadata = MessageMetadata::default();
    ///
    /// let message = ComponentMessage::new(sender, payload, metadata);
    /// ```
    pub fn new(sender: ComponentId, payload: MessagePayload, metadata: MessageMetadata) -> Self {
        Self {
            sender,
            payload,
            metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_metadata_default_initialization() {
        let metadata = MessageMetadata::default();

        assert!(metadata.correlation_id.is_none());
        assert!(metadata.reply_to.is_none());
        assert_eq!(metadata.timestamp_ms, 0);
        assert!(metadata.content_type.is_none());
    }

    #[test]
    fn test_component_message_creation_with_all_fields() {
        let sender = ComponentId::new("system", "database", "prod");
        let payload = vec![1, 2, 3, 4];
        let metadata = MessageMetadata {
            correlation_id: Some("corr-123".to_string()),
            reply_to: Some(ComponentId::new("system", "cache", "dev")),
            timestamp_ms: 1234567890,
            content_type: Some("application/json".to_string()),
        };

        let message = ComponentMessage::new(sender.clone(), payload.clone(), metadata.clone());

        assert_eq!(message.sender, sender);
        assert_eq!(message.payload, payload);
        assert_eq!(
            message.metadata.correlation_id,
            Some("corr-123".to_string())
        );
        assert_eq!(
            message.metadata.reply_to.unwrap().to_string_id(),
            "system/cache/dev"
        );
        assert_eq!(message.metadata.timestamp_ms, 1234567890);
        assert_eq!(
            message.metadata.content_type,
            Some("application/json".to_string())
        );
    }

    #[test]
    fn test_default_sets_correlation_id_to_none() {
        let metadata = MessageMetadata::default();
        assert!(metadata.correlation_id.is_none());
    }

    #[test]
    fn test_default_sets_reply_to_to_none() {
        let metadata = MessageMetadata::default();
        assert!(metadata.reply_to.is_none());
    }

    #[test]
    fn test_default_sets_timestamp_ms_to_zero() {
        let metadata = MessageMetadata::default();
        assert_eq!(metadata.timestamp_ms, 0);
    }

    #[test]
    fn test_default_sets_content_type_to_none() {
        let metadata = MessageMetadata::default();
        assert!(metadata.content_type.is_none());
    }

    #[test]
    fn test_message_with_various_metadata_combinations() {
        let sender = ComponentId::new("test", "comp", "1");
        let payload = vec![];

        // Test with only correlation_id
        let metadata1 = MessageMetadata {
            correlation_id: Some("corr-1".to_string()),
            ..Default::default()
        };
        let message1 = ComponentMessage::new(sender.clone(), payload.clone(), metadata1);
        assert_eq!(message1.metadata.correlation_id, Some("corr-1".to_string()));

        // Test with only reply_to
        let metadata2 = MessageMetadata {
            reply_to: Some(ComponentId::new("test", "comp", "2")),
            ..Default::default()
        };
        let message2 = ComponentMessage::new(sender.clone(), payload.clone(), metadata2);
        assert_eq!(
            message2.metadata.reply_to.unwrap().to_string_id(),
            "test/comp/2"
        );

        // Test with only content_type
        let metadata3 = MessageMetadata {
            content_type: Some("text/plain".to_string()),
            ..Default::default()
        };
        let message3 = ComponentMessage::new(sender, payload, metadata3);
        assert_eq!(
            message3.metadata.content_type,
            Some("text/plain".to_string())
        );
    }

    #[test]
    fn test_message_clone_creates_independent_copy() {
        let sender = ComponentId::new("test", "comp", "1");
        let payload = vec![1, 2, 3];
        let metadata = MessageMetadata::default();

        let message1 = ComponentMessage::new(sender.clone(), payload.clone(), metadata.clone());
        let message2 = message1.clone();

        assert_eq!(message1.sender, message2.sender);
        assert_eq!(message1.payload, message2.payload);
        assert_eq!(
            message1.metadata.correlation_id,
            message2.metadata.correlation_id
        );

        // Verify independence by modifying original's payload
        let mut message1_mut = message1;
        message1_mut.payload.push(4);

        assert_eq!(message2.payload.len(), 3); // Clone was not affected
        assert_eq!(message1_mut.payload.len(), 4);
    }

    #[test]
    fn test_message_with_empty_payload() {
        let sender = ComponentId::new("test", "comp", "1");
        let payload: Vec<u8> = vec![];
        let metadata = MessageMetadata::default();

        let message = ComponentMessage::new(sender, payload, metadata);

        assert!(message.payload.is_empty());
    }
}
