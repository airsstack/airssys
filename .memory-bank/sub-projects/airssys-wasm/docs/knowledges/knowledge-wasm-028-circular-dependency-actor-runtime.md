# KNOWLEDGE-WASM-028: Circular Dependency Between actor/ and runtime/

**Document ID:** KNOWLEDGE-WASM-028  
**Created:** 2025-12-21  
**Updated:** 2025-12-21  
**Category:** Architecture / Fatal Errors / Module Dependencies  
**Maturity:** Stable  
**Severity:** 🔴 **CRITICAL**

## Overview

This document records a **critical architectural violation** where the `runtime/` module incorrectly imports types from `actor/` module, creating a circular dependency. The correct dependency direction is **one-way**: `actor/ → runtime/ → core/`. This violation breaks layer separation and indicates confused module responsibilities.

## Context

### Problem Statement

The module dependency architecture should be:

```
┌─────────────────────────────────────────────────────────────┐
│                   CORRECT DEPENDENCY FLOW                    │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│    actor/                                                    │
│       │                                                      │
│       │ imports (depends on)                                 │
│       ▼                                                      │
│    runtime/                                                  │
│       │                                                      │
│       │ imports (depends on)                                 │
│       ▼                                                      │
│    core/                                                     │
│                                                              │
│    ONE-WAY FLOW: actor/ → runtime/ → core/                  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

However, the current codebase has:

```
┌─────────────────────────────────────────────────────────────┐
│                   ACTUAL DEPENDENCY (WRONG!)                 │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│    actor/  ←──────────────────────┐                         │
│       │                           │                          │
│       │ imports                   │ imports (WRONG!)         │
│       ▼                           │                          │
│    runtime/ ──────────────────────┘                         │
│       │                                                      │
│       │ imports                                              │
│       ▼                                                      │
│    core/                                                     │
│                                                              │
│    CIRCULAR: actor/ ↔ runtime/ (VIOLATION!)                 │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Scope

This violation affects:
- Module compilation order
- Code organization clarity
- Testability (can't test runtime/ without actor/)
- Refactorability (changes cascade in both directions)

## Technical Content

### Evidence of Circular Imports

**Files in `runtime/` that incorrectly import from `actor/`:**

| File | Line | Wrong Import |
|------|------|--------------|
| `src/runtime/async_host.rs` | 52 | `use crate::actor::ComponentMessage` |
| `src/runtime/messaging.rs` | 76 | `use crate::actor::ComponentMessage` |
| `src/runtime/messaging_subscription.rs` | 108 | `use crate::actor::component::ComponentRegistry` |
| `src/runtime/messaging_subscription.rs` | 109 | `use crate::actor::component::ActorSystemSubscriber` |
| `src/runtime/messaging_subscription.rs` | 109 | `use crate::actor::component::SubscriberManager` |

### Why This Is Wrong

#### 1. Module Responsibilities

| Module | Purpose | Should Import From |
|--------|---------|-------------------|
| `core/` | Types, traits, errors | Nothing internal (leaf module) |
| `runtime/` | WASM execution engine | `core/` only |
| `actor/` | airssys-rt integration | `core/`, `runtime/` |

#### 2. Layer Architecture (ADR-WASM-018)

```
Layer 3: airssys-rt (external crate)
    ↑ used by
Layer 2: actor/ (integration layer)
    ↑ uses  
Layer 2: runtime/ (WASM execution)
    ↑ uses
Layer 1: core/ (types and traits)
```

`runtime/` is a **lower layer** than `actor/`. Lower layers should NEVER import from higher layers.

#### 3. The Specific Problem

**`ComponentMessage`** is currently defined in `actor/` but used by `runtime/`.

This suggests `ComponentMessage` is in the **wrong location**. It should be in `core/` because:
- It's a data type (struct), not integration logic
- Multiple modules need it
- It has no dependencies on actor-specific code

**`ComponentRegistry`** and **`ActorSystemSubscriber`** are imported by `runtime/messaging_subscription.rs`. This suggests:
- Either these types belong in a shared location
- Or `messaging_subscription.rs` has logic that belongs in `actor/`

### Root Cause Analysis

#### Why Did This Happen?

1. **Convenience over Architecture**: Developer needed `ComponentMessage` in `runtime/`, and it already existed in `actor/`, so they imported it

2. **Type Location Confusion**: `ComponentMessage` was initially created in `actor/` because that's where message handling happens, but it's actually a **data type** that should be in `core/`

3. **Mixed Responsibilities**: `messaging_subscription.rs` in `runtime/` does subscription management that touches actor registration - this logic may be in the wrong module

4. **No Dependency Linting**: No tooling enforces the correct import direction

### Impact

| Aspect | Impact |
|--------|--------|
| **Compilation** | Works (Rust allows it), but conceptually wrong |
| **Testability** | Can't unit test `runtime/` without `actor/` |
| **Maintainability** | Changes cascade unpredictably |
| **Refactoring** | High risk - touching either module affects the other |
| **Architecture** | Violates ADR-WASM-018 three-layer design |

## Remediation

### Step 1: Move ComponentMessage to core/

**Current location:** `src/actor/component/message.rs` (or similar)  
**Target location:** `src/core/component_message.rs`

```rust
// src/core/component_message.rs (NEW)

use crate::core::ComponentId;

/// Message passed between components
#[derive(Debug, Clone)]
pub struct ComponentMessage {
    pub sender: ComponentId,
    pub recipient: ComponentId,
    pub payload: Vec<u8>,
    pub correlation_id: Option<String>,
    // ... other fields
}

// src/core/mod.rs - add export
pub mod component_message;
pub use component_message::ComponentMessage;
```

### Step 2: Update Imports in runtime/

```rust
// BEFORE (WRONG):
use crate::actor::ComponentMessage;

// AFTER (CORRECT):
use crate::core::ComponentMessage;
```

Files to update:
- `src/runtime/async_host.rs`
- `src/runtime/messaging.rs`

### Step 3: Analyze messaging_subscription.rs

This file imports `ComponentRegistry` and `ActorSystemSubscriber` from `actor/`. Two options:

**Option A: Move the logic to actor/**
If `messaging_subscription.rs` primarily does actor-level integration, move it to `actor/`.

**Option B: Extract shared types to core/**
If these types are needed by multiple modules, consider extracting interfaces to `core/`.

**Recommended:** Option A - the subscription logic is integration work that belongs in `actor/`.

### Step 4: Verify No runtime/ → actor/ Imports

After remediation, run:

```bash
# Should return NO results
grep -r "use crate::actor" src/runtime/
```

## Correct Architecture After Fix

```
┌─────────────────────────────────────────────────────────────┐
│                   AFTER REMEDIATION                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│    actor/                                                    │
│       │                                                      │
│       │ imports                                              │
│       ▼                                                      │
│    runtime/                                                  │
│       │                                                      │
│       │ imports                                              │
│       ▼                                                      │
│    core/                                                     │
│       └── ComponentMessage (MOVED HERE)                     │
│                                                              │
│    ONE-WAY: actor/ → runtime/ → core/ ✅                    │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Prevention

### Guidelines for Future Development

✅ **DO:**
- Put shared data types in `core/`
- Check import direction before adding new imports
- Ask: "Is this higher-layer or lower-layer?"

❌ **DON'T:**
- Import from `actor/` in `runtime/`
- Import from `runtime/` in `core/`
- Create "convenience" imports that violate layer boundaries

### Automated Enforcement (Recommended)

Add a CI check:

```bash
#!/bin/bash
# check-layer-dependencies.sh

# runtime/ should not import from actor/
if grep -r "use crate::actor" src/runtime/; then
    echo "ERROR: runtime/ imports from actor/ - violates layer architecture"
    exit 1
fi

# core/ should not import from runtime/ or actor/
if grep -r "use crate::runtime\|use crate::actor" src/core/; then
    echo "ERROR: core/ imports from higher layers - violates layer architecture"
    exit 1
fi

echo "Layer dependencies OK"
```

## References

### Related ADRs
- **ADR-WASM-018:** Three-Layer Architecture (defines correct layer boundaries)
- **ADR-WASM-011:** Module Structure Organization
- **ADR-WASM-021:** Duplicate Runtime Remediation (related issue)
- **ADR-WASM-022:** Circular Dependency Remediation (to be created)

### Related Knowledge
- **KNOWLEDGE-WASM-027:** Duplicate WASM Runtime (related violation)
- **KNOWLEDGE-WASM-012:** Module Structure Architecture

## History

### Version History
- **2025-12-21:** v1.0 - Initial documentation of circular dependency violation

### Discovery
- **Discovered:** 2025-12-21 during architecture analysis
- **Reporter:** AI Assistant during code review
- **Verified By:** grep analysis of import statements

---
**Severity:** 🔴 CRITICAL  
**Status:** Requires remediation as part of ADR-WASM-021 Phase 1  
**Template Version:** 1.0
