# Message Delivery Mechanism - airssys-wasm

**Document Type:** Knowledge Documentation  
**Document ID:** KNOWLEDGE-WASM-025  
**Created:** 2025-12-21  
**Updated:** 2025-12-21  
**Status:** ⚠️ SUPERSEDED BY KNOWLEDGE-WASM-026  
**Priority:** Low - Historical reference only  
**Related:** ADR-WASM-009, KNOWLEDGE-WASM-026

---

## ⚠️ THIS DOCUMENT IS SUPERSEDED

**This document is superseded by [KNOWLEDGE-WASM-026: Message Delivery Architecture - Final Decision](knowledge-wasm-026-message-delivery-architecture-final.md).**

The solution proposed in this document (extending `ComponentRegistry` to store `MailboxSender`) was **rejected** after architectural review.

### Reason for Rejection

1. **Violates Single Responsibility:** ComponentRegistry should be pure identity lookup
2. **Mixing Concerns:** Addressing + delivery in one component  
3. **ADR-WASM-009 Violation:** ADR defines ComponentRegistry for identity, not delivery

### Correct Solution (KNOWLEDGE-WASM-026)

> **`ComponentRegistry` stays pure (identity lookup only).**  
> **`ActorSystemSubscriber` owns message delivery (has `MailboxSender` references).**

---

## Historical Content (For Reference Only)

The content below is preserved for historical context. **Do not implement this design.**

---

## The Problem: ActorAddress Cannot Send Messages

### What ActorAddress Is

From `airssys-rt/src/util/ids.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActorAddress {
    Named { id: ActorId, name: String },
    Anonymous { id: ActorId },
}

impl ActorAddress {
    pub fn named(name: impl Into<String>) -> Self { ... }
    pub fn anonymous() -> Self { ... }
    pub fn id(&self) -> &ActorId { ... }
    pub fn name(&self) -> Option<&str> { ... }
    // NO send() method!
}
```

**Key Insight**: `ActorAddress` is an **identifier**, not a **sender**. It identifies which actor to send to, but provides no mechanism to actually send messages.

---

## ~~Solution Options~~ (REJECTED)

### ~~Option 2: Pragmatic Simplification (REJECTED)~~

**This was originally marked as "CHOSEN" but was subsequently REJECTED.**

~~**Architecture**:~~
```text
ComponentRegistry (EXTENDED)  ← REJECTED
    └── ComponentId → (ActorAddress, MailboxSender<ComponentMessage>)
    └── send_to(component_id, message) → Result<(), WasmError>
```

**Why this was rejected:**
- ⚠️ `ComponentRegistry` should NOT have delivery responsibility
- ⚠️ Deviates from ADR-WASM-009 design principles
- ⚠️ Mixing concerns (addressing + delivery)

---

## Correct Solution

See **[KNOWLEDGE-WASM-026: Message Delivery Architecture - Final Decision](knowledge-wasm-026-message-delivery-architecture-final.md)** for the correct architecture:

- **`ActorSystemSubscriber`** owns `mailbox_senders: HashMap<ComponentId, MailboxSender>`
- **`ComponentRegistry`** remains pure identity lookup
- When ComponentActor spawns, MailboxSender is registered with ActorSystemSubscriber
- ActorSystemSubscriber uses this map to deliver messages

---

## Version History

| Date | Version | Changes |
|------|---------|---------|
| 2025-12-21 | 1.0 | Initial document - delivery mechanism decision |
| 2025-12-21 | 2.0 | SUPERSEDED by KNOWLEDGE-WASM-026 - solution rejected |

---

**End of KNOWLEDGE-WASM-025 (SUPERSEDED)**
