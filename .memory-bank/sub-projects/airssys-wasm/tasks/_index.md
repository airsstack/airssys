# airssys-wasm Tasks Index

**Sub-Project:** airssys-wasm  
**Last Updated:** 2025-12-26 18:00  
**Current Task:** TASK 1.1 - Move Messaging Types to Core/

---

## ⚠️ CRITICAL: READ THIS FIRST

**THE ONLY SOURCE OF TRUTH IS:** `tasks/TASK-1.1-HONEST-STATUS.md`

**Created:** 2025-12-26  
**Contains:**
- Actual current state (verified by grep commands)
- What's actually done (40% complete)
- What's not done (60% incomplete)
- All lies told in previous sessions
- All remaining work with step-by-step plan

**IGNORE:** All other task files that mention Task 1.1, 1.2, or 1.3
- They are outdated or confusing
- Do NOT match current reality

---

## Current Task

| Task ID | Name | Status | Priority | Time Remaining |
|---------|------|--------|----------|----------------|
| **TASK 1.1** | Move Messaging Types to Core/ | ⚠️ IN PROGRESS | CRITICAL | ~2 hours |

---

## Task 1.1 Status (from TASK-1.1-HONEST-STATUS.md)

### What's Complete (~40%)
- ✅ Types added to `core/messaging.rs` (CorrelationId, PendingRequest, ResponseMessage, RequestError)
- ✅ Types exported from `core/mod.rs`
- ✅ MessagingSubscriptionService moved to `actor/message/messaging_subscription.rs`

### What's Not Complete (~60%)
- ❌ ResponseRouter still in `runtime/messaging.rs` (should be in `actor/message/response_router.rs`)
- ❌ Circular dependency exists (runtime/ imports from actor/ 5 times)
- ❌ Build broken (Cargo.toml references deleted benchmark)

### Remaining Work
1. Create `src/actor/message/response_router.rs`
2. Remove ResponseRouter from `src/runtime/messaging.rs`
3. Fix circular dependency imports
4. Fix Cargo.toml
5. Verify with grep commands
6. Build and test

---

## Archived Tasks

All previous task files have been archived or are outdated. See `TASK-1.1-HONEST-STATUS.md` for current status.

---

**Truth File:** `tasks/TASK-1.1-HONEST-STATUS.md`  
**Last Verified:** 2025-12-26  
**Verification Method:** grep commands and code inspection

### WASM-TASK-HOTFIX-001: Messaging Module Architecture Refactoring
- **Status:** 🔴 CRITICAL / NOT STARTED
- **File:** [`task-hotfix-001-messaging-module-architecture-refactoring.md`](task-hotfix-001-messaging-module-architecture-refactoring.md)
- **Priority:** 🔴 BLOCKER
- **Estimated Effort:** 4.5-5.5 weeks
- **Description:** Refactor messaging infrastructure from `runtime/messaging.rs` to top-level `messaging/` module
- **Related Docs:**
  - [KNOWLEDGE-WASM-034](../docs/knowledges/knowledge-wasm-034-module-architecture-violation-messaging-in-runtime.md)
  - [ADR-WASM-024](../docs/adr/adr-wasm-024-refactor-messaging-from-runtime-to-top-level-module.md)
- **Dependencies:** ADR-WASM-018 (must follow)
- **Blocks:** All subsequent WASM-TASK-006+ development

