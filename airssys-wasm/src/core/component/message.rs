// Layer 1: Standard library imports (per PROJECTS_STANDARD.md §2.1)
// None needed for this module

// Layer 2: Third-party crate imports (per PROJECTS_STANDARD.md §2.1)
// None needed for this module

// Layer 3: Internal module imports (per PROJECTS_STANDARD.md §2.1)
use super::id::ComponentId;

/// Message payload wrapper for raw bytes.
///
/// `MessagePayload` wraps raw bytes for inter-component communication.
/// This type lives in `core/component/` because it is a fundamental
/// data type used by `ComponentMessage`, not a messaging behavior.
///
/// # Architecture Note
///
/// Per ADR-WASM-028 v1.1, `MessagePayload` is defined in `core/component/message.rs`
/// (not in `core/messaging/`) to avoid circular dependencies between
/// `core/component/` and `core/messaging/`.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::component::message::MessagePayload;
///
/// // Create from Vec<u8>
/// let payload = MessagePayload::new(vec![1, 2, 3, 4]);
/// assert_eq!(payload.len(), 4);
/// assert!(!payload.is_empty());
///
/// // Access bytes
/// assert_eq!(payload.as_bytes(), &[1, 2, 3, 4]);
///
/// // Convert back to Vec<u8>
/// let bytes = payload.into_bytes();
/// assert_eq!(bytes, vec![1, 2, 3, 4]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessagePayload(Vec<u8>);

impl MessagePayload {
    /// Creates a new `MessagePayload` from raw bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw bytes to wrap
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::component::message::MessagePayload;
    ///
    /// let payload = MessagePayload::new(vec![1, 2, 3]);
    /// assert_eq!(payload.len(), 3);
    /// ```
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    /// Returns the payload as a byte slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::component::message::MessagePayload;
    ///
    /// let payload = MessagePayload::new(vec![1, 2, 3]);
    /// assert_eq!(payload.as_bytes(), &[1, 2, 3]);
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Consumes the payload and returns the underlying bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::component::message::MessagePayload;
    ///
    /// let payload = MessagePayload::new(vec![1, 2, 3]);
    /// let bytes = payload.into_bytes();
    /// assert_eq!(bytes, vec![1, 2, 3]);
    /// ```
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }

    /// Returns the length of the payload in bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::component::message::MessagePayload;
    ///
    /// let payload = MessagePayload::new(vec![1, 2, 3, 4, 5]);
    /// assert_eq!(payload.len(), 5);
    /// ```
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if the payload is empty.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::component::message::MessagePayload;
    ///
    /// let empty = MessagePayload::new(vec![]);
    /// assert!(empty.is_empty());
    ///
    /// let non_empty = MessagePayload::new(vec![1]);
    /// assert!(!non_empty.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Vec<u8>> for MessagePayload {
    fn from(data: Vec<u8>) -> Self {
        Self::new(data)
    }
}

impl From<&[u8]> for MessagePayload {
    fn from(data: &[u8]) -> Self {
        Self::new(data.to_vec())
    }
}

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
/// - `payload`: Message payload wrapped in [`MessagePayload`]
/// - `metadata`: Message metadata (correlation, reply routing, timestamp, content type)
///
/// # Architecture Note
///
/// ComponentMessage lives in `core/component/` (Layer 1) as a pure data structure.
/// It is the foundational message type used by all inter-component communication.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::component::message::{ComponentMessage, MessageMetadata, MessagePayload};
/// use airssys_wasm::core::component::id::ComponentId;
///
/// let sender = ComponentId::new("system", "database", "prod");
/// let payload = MessagePayload::new(vec![1, 2, 3, 4]);
/// let metadata = MessageMetadata::default();
///
/// let message = ComponentMessage::new(sender, payload.clone(), metadata);
///
/// assert_eq!(message.sender.to_string_id(), "system/database/prod");
/// assert_eq!(message.payload, payload);
/// ```
#[derive(Debug, Clone)]
pub struct ComponentMessage {
    /// ComponentId of the message sender
    pub sender: ComponentId,
    /// Message payload for inter-component communication
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
    /// * `payload` - Message payload wrapped in [`MessagePayload`]
    /// * `metadata` - Message metadata
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::component::message::{ComponentMessage, MessageMetadata, MessagePayload};
    /// use airssys_wasm::core::component::id::ComponentId;
    ///
    /// let sender = ComponentId::new("system", "database", "prod");
    /// let payload = MessagePayload::new(vec![1, 2, 3]);
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
        let payload = MessagePayload::new(vec![1, 2, 3, 4]);
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
        let payload = MessagePayload::new(vec![]);

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
        let payload = MessagePayload::new(vec![1, 2, 3]);
        let metadata = MessageMetadata::default();

        let message1 = ComponentMessage::new(sender.clone(), payload.clone(), metadata.clone());
        let message2 = message1.clone();

        assert_eq!(message1.sender, message2.sender);
        assert_eq!(message1.payload, message2.payload);
        assert_eq!(
            message1.metadata.correlation_id,
            message2.metadata.correlation_id
        );

        // Verify independence - clones are equal but independent
        assert_eq!(message1.payload.len(), 3);
        assert_eq!(message2.payload.len(), 3);
    }

    #[test]
    fn test_message_with_empty_payload() {
        let sender = ComponentId::new("test", "comp", "1");
        let payload = MessagePayload::new(vec![]);
        let metadata = MessageMetadata::default();

        let message = ComponentMessage::new(sender, payload, metadata);

        assert!(message.payload.is_empty());
    }

    // MessagePayload-specific tests
    #[test]
    fn test_message_payload_new() {
        let payload = MessagePayload::new(vec![1, 2, 3, 4, 5]);
        assert_eq!(payload.len(), 5);
        assert!(!payload.is_empty());
    }

    #[test]
    fn test_message_payload_as_bytes() {
        let payload = MessagePayload::new(vec![10, 20, 30]);
        assert_eq!(payload.as_bytes(), &[10, 20, 30]);
    }

    #[test]
    fn test_message_payload_into_bytes() {
        let payload = MessagePayload::new(vec![1, 2, 3]);
        let bytes = payload.into_bytes();
        assert_eq!(bytes, vec![1, 2, 3]);
    }

    #[test]
    fn test_message_payload_from_vec() {
        let payload: MessagePayload = vec![1, 2, 3].into();
        assert_eq!(payload.len(), 3);
    }

    #[test]
    fn test_message_payload_from_slice() {
        let data: &[u8] = &[1, 2, 3, 4];
        let payload: MessagePayload = data.into();
        assert_eq!(payload.len(), 4);
    }

    #[test]
    fn test_message_payload_empty() {
        let payload = MessagePayload::new(vec![]);
        assert!(payload.is_empty());
        assert_eq!(payload.len(), 0);
    }

    #[test]
    fn test_message_payload_equality() {
        let p1 = MessagePayload::new(vec![1, 2, 3]);
        let p2 = MessagePayload::new(vec![1, 2, 3]);
        let p3 = MessagePayload::new(vec![3, 2, 1]);

        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
    }

    // Gap analysis tests

    #[test]
    fn test_message_payload_debug_format() {
        let payload = MessagePayload::new(vec![1, 2, 3]);
        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("MessagePayload"));
    }

    #[test]
    fn test_message_metadata_clone_creates_independent_copy() {
        let reply_to = ComponentId::new("test", "comp", "1");
        let metadata1 = MessageMetadata {
            correlation_id: Some("corr-1".to_string()),
            reply_to: Some(reply_to),
            timestamp_ms: 12345,
            content_type: Some("application/json".to_string()),
        };
        let metadata2 = metadata1.clone();

        // Verify independence
        assert_eq!(metadata1.correlation_id, metadata2.correlation_id);
        assert_eq!(metadata1.timestamp_ms, metadata2.timestamp_ms);
        assert_eq!(metadata1.content_type, metadata2.content_type);
    }

    #[test]
    fn test_component_message_debug_format() {
        let sender = ComponentId::new("test", "comp", "1");
        let payload = MessagePayload::new(vec![1, 2, 3]);
        let metadata = MessageMetadata::default();
        let message = ComponentMessage::new(sender, payload, metadata);

        let debug_str = format!("{:?}", message);
        assert!(debug_str.contains("ComponentMessage"));
        assert!(debug_str.contains("sender"));
    }

    #[test]
    fn test_message_payload_large_data() {
        let large_data = vec![0u8; 1024 * 1024]; // 1MB
        let payload = MessagePayload::new(large_data.clone());
        assert_eq!(payload.len(), 1024 * 1024);
        assert_eq!(payload.as_bytes().len(), 1024 * 1024);
    }
}
