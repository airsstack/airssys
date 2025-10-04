// Layer 1: Standard library imports
use std::fmt::{self, Display};

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Layer 3: Internal module imports
// (none)

/// Unique identifier for actors in the system
///
/// # Performance
/// Uses UUID v4 for globally unique identifiers with excellent collision resistance.
/// Implements cheap cloning via Copy trait.
///
/// # Example
/// ```rust
/// use airssys_rt::util::ActorId;
///
/// let id1 = ActorId::new();
/// let id2 = ActorId::new();
/// assert_ne!(id1, id2); // Globally unique
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActorId(Uuid);

impl ActorId {
    /// Generate a new random ActorId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create ActorId from existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for ActorId {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for ActorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for messages in the system
///
/// # Example
/// ```rust
/// use airssys_rt::util::MessageId;
///
/// let id = MessageId::new();
/// println!("Message ID: {}", id);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(Uuid);

impl MessageId {
    /// Generate a new random MessageId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create MessageId from existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Actor address for message routing
///
/// # Design
/// Supports both named and anonymous actors for flexible routing patterns.
/// Named actors can be discovered by name, while anonymous actors are
/// accessible only via their unique ID.
///
/// # Example
/// ```rust
/// use airssys_rt::util::ActorAddress;
///
/// // Create named actor
/// let supervisor = ActorAddress::named("main_supervisor");
/// assert_eq!(supervisor.name(), Some("main_supervisor"));
///
/// // Create anonymous actor
/// let worker = ActorAddress::anonymous();
/// assert_eq!(worker.name(), None);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActorAddress {
    /// Named actor with string identifier
    Named { id: ActorId, name: String },
    /// Anonymous actor with only ID
    Anonymous { id: ActorId },
}

impl ActorAddress {
    /// Create a new named actor address
    pub fn named(name: impl Into<String>) -> Self {
        Self::Named {
            id: ActorId::new(),
            name: name.into(),
        }
    }

    /// Create a new anonymous actor address
    pub fn anonymous() -> Self {
        Self::Anonymous { id: ActorId::new() }
    }

    /// Get the actor ID
    pub fn id(&self) -> &ActorId {
        match self {
            Self::Named { id, .. } => id,
            Self::Anonymous { id } => id,
        }
    }

    /// Get the actor name if available
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Named { name, .. } => Some(name),
            Self::Anonymous { .. } => None,
        }
    }
}

impl Display for ActorAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Named { id, name } => write!(f, "{name}@{id}"),
            Self::Anonymous { id } => write!(f, "anonymous@{id}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_id_creation() {
        let id1 = ActorId::new();
        let id2 = ActorId::new();

        assert_ne!(id1, id2); // Should be unique
    }

    #[test]
    fn test_actor_id_from_uuid() {
        let uuid = Uuid::new_v4();
        let id = ActorId::from_uuid(uuid);

        assert_eq!(id.as_uuid(), &uuid);
    }

    #[test]
    fn test_actor_id_default() {
        let id = ActorId::default();
        assert_eq!(id.as_uuid().get_version_num(), 4);
    }

    #[test]
    fn test_actor_id_display() {
        let id = ActorId::new();
        let display = format!("{id}");

        assert!(!display.is_empty());
        assert!(display.contains('-')); // UUID format
    }

    #[test]
    fn test_message_id_creation() {
        let id1 = MessageId::new();
        let id2 = MessageId::new();

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_message_id_default() {
        let id = MessageId::default();
        assert_eq!(id.as_uuid().get_version_num(), 4);
    }

    #[test]
    fn test_named_actor_address() {
        let addr = ActorAddress::named("test_actor");

        assert_eq!(addr.name(), Some("test_actor"));
        assert_eq!(addr.id().as_uuid().get_version_num(), 4);
    }

    #[test]
    fn test_anonymous_actor_address() {
        let addr = ActorAddress::anonymous();

        assert_eq!(addr.name(), None);
        assert_eq!(addr.id().as_uuid().get_version_num(), 4);
    }

    #[test]
    fn test_actor_address_display_named() {
        let addr = ActorAddress::named("my_actor");
        let display = format!("{addr}");

        assert!(display.contains("my_actor@"));
    }

    #[test]
    fn test_actor_address_display_anonymous() {
        let addr = ActorAddress::anonymous();
        let display = format!("{addr}");

        assert!(display.contains("anonymous@"));
    }

    #[test]
    fn test_actor_address_equality() {
        let addr1 = ActorAddress::named("actor");
        let addr2 = ActorAddress::named("actor");

        // Different IDs even with same name
        assert_ne!(addr1, addr2);
    }
}
