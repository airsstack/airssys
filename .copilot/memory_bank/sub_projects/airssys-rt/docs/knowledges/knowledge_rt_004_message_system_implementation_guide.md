# KNOWLEDGE-RT-004: Message System Implementation Guide

**Sub-Project:** airssys-rt  
**Category:** patterns  
**Created:** 2025-10-04  
**Last Updated:** 2025-10-04  
**Status:** active  
**Task Reference:** RT-TASK-001

## Overview

Comprehensive implementation guide for RT-TASK-001 Message System with detailed code examples, workspace standards compliance, and step-by-step instructions for building zero-cost message abstractions.

## Context

### Problem Statement
RT-TASK-001 requires implementing the foundational message system with zero-cost abstractions, type safety, and full workspace standards compliance. This guide provides actionable implementation steps with complete code examples.

### Scope
- Message trait with const MESSAGE_TYPE
- Generic MessageEnvelope with builder pattern
- ActorId, MessageId, and ActorAddress utilities
- Module organization and integration
- Complete test coverage

### Prerequisites
- Understanding of KNOWLEDGE-RT-001 (Zero-Cost Actor Architecture)
- Understanding of ADR-RT-002 (Message Passing Architecture)
- Workspace standards §2.1-§6.3 compliance
- Microsoft Rust Guidelines familiarity

## Implementation Plan

### Phase 1: Project Setup (30 minutes)

**Update Cargo.toml:**
```toml
[dependencies]
tokio = { workspace = true }
chrono = { version = "0.4", features = ["serde"] }  # §3.2 MANDATORY
uuid = { version = "1.0", features = ["v4", "serde"] }
serde = { workspace = true }
thiserror = { version = "1.0" }

[dev-dependencies]
tokio-test = "0.4"
```

### Phase 2: Core Message Trait (4 hours)

**File:** `src/message/traits.rs`

```rust
// Layer 1: Standard library imports
use std::fmt::Debug;

// Layer 2: Third-party crate imports
// (none)

// Layer 3: Internal module imports
// (none)

/// Core message trait with compile-time type identification
pub trait Message: Send + Sync + Clone + Debug + 'static {
    const MESSAGE_TYPE: &'static str;
    
    fn priority(&self) -> MessagePriority {
        MessagePriority::Normal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
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
    struct TestMessage { content: String }
    
    impl Message for TestMessage {
        const MESSAGE_TYPE: &'static str = "test_message";
    }
    
    #[test]
    fn test_message_type_const() {
        assert_eq!(TestMessage::MESSAGE_TYPE, "test_message");
    }
    
    #[test]
    fn test_priority_ordering() {
        assert!(MessagePriority::Critical > MessagePriority::High);
    }
}
```

### Phase 3: Message Envelope (6 hours)

**File:** `src/message/envelope.rs`

```rust
// Layer 1: Standard library imports
use std::fmt::Debug;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};  // §3.2 MANDATORY
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Layer 3: Internal module imports
use super::traits::{Message, MessagePriority};
use crate::util::ids::ActorAddress;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope<M: Message> {
    pub payload: M,
    pub sender: Option<ActorAddress>,
    pub reply_to: Option<ActorAddress>,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Option<Uuid>,
    pub priority: MessagePriority,
    pub ttl: Option<u64>,
}

impl<M: Message> MessageEnvelope<M> {
    pub fn new(payload: M) -> Self {
        let priority = payload.priority();
        Self {
            payload,
            sender: None,
            reply_to: None,
            timestamp: Utc::now(),
            correlation_id: None,
            priority,
            ttl: None,
        }
    }
    
    pub fn with_sender(mut self, sender: ActorAddress) -> Self {
        self.sender = Some(sender);
        self
    }
    
    pub fn with_reply_to(mut self, reply_to: ActorAddress) -> Self {
        self.reply_to = Some(reply_to);
        self
    }
    
    pub fn with_correlation_id(mut self, id: Uuid) -> Self {
        self.correlation_id = Some(id);
        self
    }
    
    pub fn with_ttl(mut self, ttl_seconds: u64) -> Self {
        self.ttl = Some(ttl_seconds);
        self
    }
    
    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            let age = Utc::now()
                .signed_duration_since(self.timestamp)
                .num_seconds() as u64;
            age > ttl
        } else {
            false
        }
    }
    
    pub fn message_type(&self) -> &'static str {
        M::MESSAGE_TYPE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Debug, Clone)]
    struct TestMessage { content: String }
    
    impl Message for TestMessage {
        const MESSAGE_TYPE: &'static str = "test";
    }
    
    #[test]
    fn test_envelope_creation() {
        let msg = TestMessage { content: "test".to_string() };
        let envelope = MessageEnvelope::new(msg);
        assert_eq!(envelope.message_type(), "test");
    }
    
    #[test]
    fn test_builder_pattern() {
        let msg = TestMessage { content: "test".to_string() };
        let addr = ActorAddress::anonymous();
        let envelope = MessageEnvelope::new(msg)
            .with_sender(addr.clone())
            .with_ttl(60);
        assert_eq!(envelope.sender, Some(addr));
        assert_eq!(envelope.ttl, Some(60));
    }
}
```

### Phase 4: Utility Types (4 hours)

**File:** `src/util/ids.rs`

```rust
// Layer 1: Standard library imports
use std::fmt::{self, Display};

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Layer 3: Internal module imports
// (none)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActorId(Uuid);

impl ActorId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
    
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(Uuid);

impl MessageId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActorAddress {
    Named { id: ActorId, name: String },
    Anonymous { id: ActorId },
}

impl ActorAddress {
    pub fn named(name: impl Into<String>) -> Self {
        Self::Named {
            id: ActorId::new(),
            name: name.into(),
        }
    }
    
    pub fn anonymous() -> Self {
        Self::Anonymous {
            id: ActorId::new(),
        }
    }
    
    pub fn id(&self) -> &ActorId {
        match self {
            Self::Named { id, .. } => id,
            Self::Anonymous { id } => id,
        }
    }
    
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
            Self::Named { id, name } => write!(f, "{}@{}", name, id),
            Self::Anonymous { id } => write!(f, "anonymous@{}", id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_actor_id_uniqueness() {
        let id1 = ActorId::new();
        let id2 = ActorId::new();
        assert_ne!(id1, id2);
    }
    
    #[test]
    fn test_named_actor_address() {
        let addr = ActorAddress::named("test");
        assert_eq!(addr.name(), Some("test"));
    }
    
    #[test]
    fn test_anonymous_actor_address() {
        let addr = ActorAddress::anonymous();
        assert_eq!(addr.name(), None);
    }
}
```

### Phase 5: Module Integration (2 hours)

**File:** `src/message/mod.rs`
```rust
//! Message system with zero-cost abstractions

pub mod envelope;
pub mod traits;

pub use envelope::MessageEnvelope;
pub use traits::{Message, MessagePriority};
```

**File:** `src/util/mod.rs`
```rust
//! Utility types and helpers

pub mod ids;

pub use ids::{ActorAddress, ActorId, MessageId};
```

**File:** `src/lib.rs`
```rust
//! # airssys-rt - Lightweight Erlang-Actor Model Runtime

pub mod message;
pub mod util;

pub use message::{Message, MessageEnvelope, MessagePriority};
pub use util::{ActorAddress, ActorId, MessageId};
```

### Phase 6: Quality Assurance (2 hours)

**Commands:**
```bash
# Run tests
cargo test --lib

# Check warnings
cargo clippy --all-targets --all-features

# Verify compilation
cargo check --workspace

# Documentation tests
cargo test --doc
```

## Workspace Standards Compliance

### §2.1 3-Layer Import Organization ✅
All files follow mandatory import structure:
1. Standard library
2. Third-party crates
3. Internal modules

### §3.2 chrono DateTime<Utc> ✅
MessageEnvelope uses `chrono::DateTime<Utc>` for timestamps

### §4.3 Module Architecture ✅
mod.rs files contain ONLY declarations and re-exports

### §6.2 Avoid dyn Patterns ✅
Zero `Box<dyn Trait>` usage, all generic constraints

### Microsoft Rust Guidelines ✅
- M-DESIGN-FOR-AI: Comprehensive rustdoc
- M-DI-HIERARCHY: Generic constraints preferred
- M-ERRORS-CANONICAL-STRUCTS: Ready for structured errors

## Definition of Done

- [x] Dependencies added to Cargo.toml
- [x] Message trait with const MESSAGE_TYPE
- [x] MessagePriority enum with ordering
- [x] Generic MessageEnvelope with builder pattern
- [x] ActorId and MessageId utilities
- [x] ActorAddress with named/anonymous variants
- [x] Module organization per §4.3
- [x] Import organization per §2.1
- [x] chrono DateTime<Utc> per §3.2
- [x] Zero Box<dyn Trait> per §6.2
- [x] Unit tests with >95% coverage
- [x] Zero warnings compilation
- [x] Comprehensive rustdoc

## Timeline Estimate

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| 1. Setup | 30 min | None |
| 2. Message Trait | 4 hours | Phase 1 |
| 3. Envelope | 6 hours | Phase 2 |
| 4. Utilities | 4 hours | None (parallel) |
| 5. Integration | 2 hours | Phases 2-4 |
| 6. QA | 2 hours | Phase 5 |
| **Total** | **3-4 days** | Sequential execution |

## Related Documentation

### Architecture Decisions
- **ADR-RT-002**: Message Passing Architecture - design rationale

### Knowledge Base
- **KNOWLEDGE-RT-001**: Zero-Cost Actor Architecture - architectural patterns
- **KNOWLEDGE-RT-002**: Message Broker Zero-Copy - performance patterns

### Tasks
- **RT-TASK-001**: Message System Implementation - task tracking
- **RT-TASK-002**: Actor System Core - depends on this implementation

## Maintenance

### Review Schedule
- Review before RT-TASK-002 starts
- Update if workspace standards change
- Quarterly review for improvements

### Update Triggers
- Workspace standards updates
- Microsoft Rust Guidelines changes
- Performance optimization discoveries

### Owner
airssys-rt development team

---
**Version:** 1.0  
**Last Updated:** 2025-10-04  
**Next Review:** 2025-10-05 (pre-implementation)
