# KNOWLEDGE-WASM-031: Foundational Architecture

**Document ID:** KNOWLEDGE-WASM-031  
**Created:** 2025-12-22  
**Category:** Core Concepts / Foundational  
**Status:** 🔴 **CRITICAL - READ FIRST**  
**Priority:** This document should be read BEFORE any other knowledge document.

---

## What is airssys-wasm?

**airssys-wasm is a WASM-based Plugin/Extension Platform.**

Inspired by smart contract platforms like **NEAR** and **Polkadot** which use WASM for their smart contracts, airssys-wasm brings the same model to general-purpose plugin systems.

---

## The Two Root Entities

There are exactly **TWO** fundamental entities:

| Entity | Description |
|--------|-------------|
| **Host** | The system that installs and runs WASM plugins |
| **Plugin/Component** | A WASM module installed on the Host |

That's it. Everything else is implementation detail.

---

## Core Architecture Principles

### 1. Each Component = One Actor

Every installed WASM component is treated as a standalone **Actor**, managed through `airssys-rt` (the Actor-based Runtime).

```
┌─────────────────────────────────────────────────┐
│                    Host                          │
│                                                  │
│  ┌──────────────┐  ┌──────────────┐             │
│  │  Component A │  │  Component B │  ...        │
│  │   (Actor)    │  │   (Actor)    │             │
│  │   Mailbox    │  │   Mailbox    │             │
│  │   Storage    │  │   Storage    │             │
│  │   State      │  │   State      │             │
│  └──────────────┘  └──────────────┘             │
│                                                  │
│  ┌──────────────────────────────────────────┐   │
│  │           airssys-rt (Actor Runtime)      │   │
│  │     Manages actors, mailboxes, routing    │   │
│  └──────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
```

### 2. Isolated Storage Per Component

Each component has its **own persistent storage**, completely separated from other components.

- Component A cannot read Component B's storage
- Component A cannot write Component B's storage
- Just like smart contracts on NEAR/Polkadot

### 3. Private State Per Component

Each component has its **own private state**, not shared with others.

- Internal variables are private
- Memory is sandboxed (WASM guarantee)
- One component cannot inspect another's internal state

### 4. Communication via Actor Mailbox

Components communicate through the **Erlang-style Actor Mailbox** pattern, provided by `airssys-rt`.

```
Component A                     Component B
     │                               │
     │  send-message (fire-and-forget)
     │─────────────────────────────►│
     │                               │ handle-message
     │                               │
     │  send-request (request-response)
     │─────────────────────────────►│
     │                               │ handle-message
     │◄─────────────────────────────│ (return value)
     │  handle-callback              │
```

- **No direct function calls** between components
- **No shared memory** between components
- All communication is **message-passing**

---

## Comparison with Smart Contract Platforms

| Feature | NEAR/Polkadot | airssys-wasm |
|---------|---------------|--------------|
| Runtime | WASM | WASM |
| Deployment | Upload to blockchain | Install on Host |
| Isolation | Per-contract storage | Per-component storage |
| Communication | Cross-contract calls | Actor mailbox messages |
| State | Private per contract | Private per component |
| Language | Any → WASM | Any → WASM |

---

## The Key Integration: airssys-wasm + airssys-rt

| Layer | Crate | Responsibility |
|-------|-------|----------------|
| WASM Execution | `airssys-wasm` | Load, validate, execute WASM |
| Actor Management | `airssys-rt` | Actors, mailboxes, supervisors, message routing |

**airssys-wasm wraps WASM components as Actors managed by airssys-rt.**

This is NOT optional - it's the core design.

---

## What the End-to-End Flow Looks Like

### Installing a Component
```
1. Host receives WASM binary
2. Host validates and stores WASM
3. Host creates Actor (via airssys-rt) for this component
4. Actor gets its own Mailbox
5. Actor gets its own Storage namespace
6. Component is ready to receive messages
```

### Component A Sends Message to Component B
```
1. Component A calls send-message("B", data)
2. Host finds Actor B's mailbox
3. Host enqueues message to Actor B's mailbox
4. Actor B's handle-message is invoked with data
5. (If request-response) Return value routed back to A
6. (If request-response) Component A's handle-callback invoked
```

---

## Why This Matters

Understanding this foundational architecture prevents:

1. **Building unnecessary abstractions** - The actor model already exists in airssys-rt
2. **Violating isolation guarantees** - Storage and state MUST be per-component
3. **Wrong communication patterns** - ONLY mailbox messaging, no shared memory
4. **Duplicating runtime logic** - airssys-rt already provides what we need

---

## Summary

| Concept | Implementation |
|---------|----------------|
| **Host** | The system running airssys-wasm |
| **Plugin/Component** | WASM binary installed on Host |
| **Actor per Component** | Each component = one airssys-rt Actor |
| **Isolated Storage** | Per-component, like smart contracts |
| **Private State** | No sharing between components |
| **Communication** | Actor Mailbox (Erlang-style) |

---

## Related Documents

- **KNOWLEDGE-WASM-002**: High-Level Overview (conceptual)
- **KNOWLEDGE-WASM-005**: Messaging Architecture (detailed)
- **KNOWLEDGE-WASM-016**: Actor System Integration Guide
- **ADR-WASM-009**: Component Communication Model

---

**Remember:** airssys-wasm = WASM plugin platform where each plugin is an Actor with isolated storage, communicating via mailbox.
