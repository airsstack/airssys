//! Actor context placeholder for Phase 1.
//!
//! This module will be fully implemented in Phase 2.

// Layer 1: Standard library imports
use std::marker::PhantomData;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc}; // §3.2 MANDATORY

// Layer 3: Internal module imports
use crate::message::Message;
use crate::util::{ActorAddress, ActorId};

/// Actor context with metadata (placeholder for Phase 2).
pub struct ActorContext<M: Message> {
    address: ActorAddress,
    id: ActorId,
    created_at: DateTime<Utc>,
    _marker: PhantomData<M>,
}

impl<M: Message> ActorContext<M> {
    /// Create a new actor context.
    pub fn new(address: ActorAddress) -> Self {
        Self {
            id: *address.id(),
            address,
            created_at: Utc::now(), // §3.2
            _marker: PhantomData,
        }
    }

    /// Get the actor's address.
    pub fn address(&self) -> &ActorAddress {
        &self.address
    }

    /// Get the actor's ID.
    pub fn id(&self) -> &ActorId {
        &self.id
    }

    /// Get the actor's creation timestamp.
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
