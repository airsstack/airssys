# KNOWLEDGE-RT-005: Actor System Core Implementation Guide

**Sub-Project:** airssys-rt  
**Category:** patterns  
**Created:** 2025-10-04  
**Last Updated:** 2025-10-04  
**Status:** active  
**Task Reference:** RT-TASK-002

## Overview

Comprehensive implementation guide for RT-TASK-002 Actor System Core with detailed code examples, workspace standards compliance, and step-by-step instructions for building zero-cost actor abstractions.

## Context

### Problem Statement
RT-TASK-002 requires implementing the core actor system with generic Actor trait, ActorContext, and lifecycle management. This builds on RT-TASK-001 message system foundation.

### Scope
- Generic Actor trait with associated Message and Error types
- ActorContext<M: Message> with zero-cost abstraction
- Actor lifecycle management (states and transitions)
- ErrorAction enum for supervision decisions
- Module organization and integration

### Prerequisites
- RT-TASK-001 completed (Message system)
- KNOWLEDGE-RT-001 (Zero-Cost Actor Architecture)
- Workspace standards §2.1-§6.3 compliance
- async-trait for async trait methods

## Implementation Plan

### Phase 1: Dependencies & Actor Trait (4-6 hours)

**Update Cargo.toml:**
```toml
[dependencies]
async-trait = { workspace = true }
```

**File:** `src/actor/traits.rs`

```rust
// Layer 1: Standard library imports
use std::error::Error;

// Layer 2: Third-party crate imports
use async_trait::async_trait;

// Layer 3: Internal module imports
use super::context::ActorContext;
use crate::message::Message;

/// Core Actor trait with generic constraints
#[async_trait]
pub trait Actor: Send + Sync + 'static {
    type Message: Message;
    type Error: Error + Send + Sync + 'static;
    
    async fn handle_message(
        &mut self,
        message: Self::Message,
        context: &mut ActorContext<Self::Message>,
    ) -> Result<(), Self::Error>;
    
    async fn pre_start(&mut self, _context: &mut ActorContext<Self::Message>) -> Result<(), Self::Error> {
        Ok(())
    }
    
    async fn post_stop(&mut self, _context: &mut ActorContext<Self::Message>) -> Result<(), Self::Error> {
        Ok(())
    }
    
    async fn on_error(
        &mut self,
        _error: Self::Error,
        _context: &mut ActorContext<Self::Message>,
    ) -> ErrorAction {
        ErrorAction::Stop
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorAction {
    Stop,
    Resume,
    Restart,
    Escalate,
}

impl Default for ErrorAction {
    fn default() -> Self {
        Self::Stop
    }
}
```

### Phase 2: Actor Context (6-8 hours)

**File:** `src/actor/context.rs`

```rust
// Layer 1: Standard library imports
use std::marker::PhantomData;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};  // §3.2 MANDATORY

// Layer 3: Internal module imports
use crate::message::Message;
use crate::util::{ActorAddress, ActorId};

pub struct ActorContext<M: Message> {
    address: ActorAddress,
    id: ActorId,
    created_at: DateTime<Utc>,
    _marker: PhantomData<M>,
}

impl<M: Message> ActorContext<M> {
    pub fn new(address: ActorAddress) -> Self {
        Self {
            id: *address.id(),
            address,
            created_at: Utc::now(),  // §3.2
            _marker: PhantomData,
        }
    }
    
    pub fn address(&self) -> &ActorAddress {
        &self.address
    }
    
    pub fn id(&self) -> &ActorId {
        &self.id
    }
    
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
```

### Phase 3: Lifecycle Management (4 hours)

**File:** `src/actor/lifecycle.rs`

```rust
// Layer 1: Standard library imports
// (none)

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};  // §3.2 MANDATORY

// Layer 3: Internal module imports
// (none)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorState {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

#[derive(Debug, Clone)]
pub struct ActorLifecycle {
    state: ActorState,
    last_state_change: DateTime<Utc>,
    restart_count: u32,
}

impl ActorLifecycle {
    pub fn new() -> Self {
        Self {
            state: ActorState::Starting,
            last_state_change: Utc::now(),  // §3.2
            restart_count: 0,
        }
    }
    
    pub fn transition_to(&mut self, new_state: ActorState) {
        self.state = new_state;
        self.last_state_change = Utc::now();  // §3.2
        
        if new_state == ActorState::Starting {
            self.restart_count += 1;
        }
    }
    
    pub fn state(&self) -> ActorState {
        self.state
    }
    
    pub fn restart_count(&self) -> u32 {
        self.restart_count
    }
}

impl Default for ActorLifecycle {
    fn default() -> Self {
        Self::new()
    }
}
```

### Phase 4: Module Integration (2 hours)

**File:** `src/actor/mod.rs`
```rust
//! Actor system core with zero-cost abstractions

pub mod context;
pub mod lifecycle;
pub mod traits;

pub use context::ActorContext;
pub use lifecycle::{ActorLifecycle, ActorState};
pub use traits::{Actor, ErrorAction};
```

**File:** `src/lib.rs`
```rust
pub mod actor;
pub mod message;
pub mod util;

pub use actor::{Actor, ActorContext, ActorState, ErrorAction};
pub use message::{Message, MessageEnvelope, MessagePriority};
pub use util::{ActorAddress, ActorId, MessageId};
```

### Phase 5: Quality Assurance (2 hours)

**Commands:**
```bash
cargo test --lib
cargo clippy --all-targets --all-features
cargo check --workspace
cargo doc --no-deps
```

## Workspace Standards Compliance

### §2.1 3-Layer Import Organization ✅
All files follow mandatory import structure

### §3.2 chrono DateTime<Utc> ✅
ActorContext and ActorLifecycle use chrono

### §4.3 Module Architecture ✅
mod.rs contains ONLY declarations/re-exports

### §6.2 Avoid dyn Patterns ✅
Zero Box<dyn Trait>, all generic constraints

### Microsoft Rust Guidelines ✅
- M-DESIGN-FOR-AI: Comprehensive rustdoc
- M-DI-HIERARCHY: Generic constraints preferred

## Definition of Done

- [x] Generic Actor trait with associated types
- [x] ErrorAction enum (Stop, Resume, Restart, Escalate)
- [x] Lifecycle methods (pre_start, post_stop, on_error)
- [x] ActorContext<M> with PhantomData
- [x] ActorLifecycle state machine
- [x] ActorState enum
- [x] Module structure (§4.3 compliant)
- [x] Unit tests >95% coverage
- [x] Zero warnings
- [x] Documentation with examples

## Timeline Estimate

| Phase | Duration |
|-------|----------|
| 1. Actor Trait | 4-6 hours |
| 2. ActorContext | 6-8 hours |
| 3. Lifecycle | 4 hours |
| 4. Integration | 2 hours |
| 5. QA | 2 hours |
| **Total** | **18-22 hours** |

## Related Documentation

### Knowledge Base
- **KNOWLEDGE-RT-001**: Zero-Cost Actor Architecture
- **KNOWLEDGE-RT-004**: Message System Implementation Guide

### Tasks
- **RT-TASK-001**: Message System (dependency)
- **RT-TASK-002**: Actor System Core (this guide)
- **RT-TASK-004**: Message Broker (future dependency)

---
**Version:** 1.0  
**Last Updated:** 2025-10-04  
**Next Review:** 2025-10-05
