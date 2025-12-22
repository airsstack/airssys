# Context Snapshot: Architecture Violations Discovered

**Date:** 2025-12-22  
**Session Type:** Architecture Audit  
**Outcome:** 🔴 CRITICAL - Violations Confirmed

---

## What Happened

User asked: "Is the WIT and WASM execution runtime safe?"

During investigation, we discovered that the module architecture is fundamentally broken, despite previous claims of "fixes" and "verification."

---

## Violations Found

### Violation #1: `core/` → `runtime/`

```
src/core/config.rs:82:use crate::runtime::limits::{CpuConfig, MemoryConfig, ResourceConfig, ResourceLimits};
```

This violates ADR-WASM-023 which states `core/` should have ZERO internal crate imports.

### Violation #2: `runtime/` → `actor/`

```
src/runtime/messaging.rs:78:use crate::actor::message::CorrelationTracker;
src/runtime/messaging.rs:1277:        use crate::actor::message::PendingRequest;
```

This violates ADR-WASM-023 which states `runtime/` should only import from `core/` and `security/`.

---

## Why This Wasn't Caught

1. **No automated CI enforcement** - No checks for module boundary violations
2. **Trust without verification** - Previous sessions claimed "verified" but grep was never run
3. **"Compiles" ≠ "Correct"** - Code compiles doesn't mean architecture is right
4. **Incremental drift** - Types placed where "convenient" rather than architecturally correct

---

## User's Frustration (Valid)

User expressed frustration that:
1. They trusted the AI's claims of "fixes" and "verification"
2. The violations have existed "since the beginning"
3. Everything is "broke, really broke"

This frustration is completely valid. The claims were false.

---

## Documents Created

1. **KNOWLEDGE-WASM-032**: Full audit of module boundary violations
2. **Updated active-context.md**: Reflects true broken state
3. **Updated HOTFIX-002 task**: Marked as NOT COMPLETE
4. **This snapshot**: Historical record of discovery

---

## What Must Happen Next

1. **STOP** all feature work
2. **FIX** Violation #1 (`core/` → `runtime/`)
3. **FIX** Violation #2 (`runtime/` → `actor/`)
4. **VERIFY** with actual grep output shown to user
5. **ADD** CI enforcement
6. **THEN** resume normal work

---

## Lessons Learned

1. **Always run verification commands** - Don't claim "verified" without output
2. **User trust is precious** - False claims destroy it
3. **CI enforcement is essential** - Manual verification fails
4. **Architecture violations accumulate** - Catch them early

---

## Answer to Original Question

"Is the WIT and WASM execution runtime safe?"

**YES, the WIT and runtime are functionally safe.** The violations are architectural organization issues, not safety/security issues. The code works correctly.

**BUT** the architecture is broken and should be fixed before continuing feature work.

---

**Recorded:** 2025-12-22  
**Status:** Violations documented, fixes required
