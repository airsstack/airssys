# ADR-WASM-024: Refactor Messaging from Runtime to Top-Level Module

**ADR ID:** ADR-WASM-024  
**Created:** 2025-12-26  
**Updated:** 2025-12-26  
**Status:** Accepted  
**Deciders:** @hiraq (Project Architect)  

## Title

Move messaging infrastructure from `src/runtime/messaging.rs` to a new top-level `src/messaging/` module to fix module architecture violation and eliminate circular dependency risk.

## Context

### Problem Statement

**Critical Architectural Violation Discovered:**

1. **Misplaced Code**: Messaging infrastructure currently in `src/runtime/messaging.rs` (1,313 lines)
2. **Wrong Responsibility**: `runtime/` should only handle WASM execution (Block 1)
3. **Circular Dependency Risk**: `runtime/messaging.rs` imports from `actor/message/`
4. **Missing Module**: No top-level `messaging/` module exists (should be Block 5)

**Impact:**
- Violates ADR-WASM-018 three-layer architecture
- Creates confusion about module boundaries
- Makes code harder to navigate
- Risk of circular dependencies (runtime → actor)
- Blocks proper Block 5 (Inter-Component Communication) development

### Business Context

The `airssys-wasm` framework aims to provide a clean, maintainable architecture for building pluggable systems. Module architecture violations increase maintenance burden and make onboarding difficult for new contributors. Fixing this violation is critical for:
- Long-term maintainability
- Clear module boundaries
- Easier navigation for developers
- Prevention of future architectural violations

### Technical Context

**Current State:**
- `src/runtime/messaging.rs` contains: MessagingService, ResponseRouter, MessageReceptionMetrics
- `runtime/messaging.rs` line 76: `use crate::actor::message::{CorrelationId, CorrelationTracker, RequestError, ResponseMessage}`
- No top-level `messaging/` module exists

**Correct Architecture (from KNOWLEDGE-WASM-012):**
```
runtime/ (Block 1: WASM Execution)
  ↓
actor/ (Block 3: Actor System Integration)
  ↓
messaging/ (Block 5: Inter-Component Communication) ← MISSING
```

### Stakeholders

- **Developers**: Need clear module structure for effective development
- **Maintainers**: Need architecture compliance to reduce long-term maintenance burden
- **Contributors**: Need intuitive code navigation and clear boundaries

## Decision

### Summary

**Move all messaging infrastructure from `src/runtime/messaging.rs` to a new top-level `src/messaging/` module.**

**Key Changes:**
1. Create `src/messaging/` module (declaration-only `mod.rs`)
2. Move MessagingService, ResponseRouter, and metrics to `src/messaging/messaging_service.rs`
3. Update all imports from `runtime::MessagingService` to `messaging::MessagingService`
4. Remove `src/runtime/messaging.rs`
5. Add CI checks to prevent future violations

### Rationale

**Why This Decision:**

1. **Correct Module Responsibility** (KNOWLEDGE-WASM-012)
   - `runtime/` is for WASM execution only (Block 1)
   - Messaging is separate responsibility (Block 5)
   - Each module should have clear, distinct purpose

2. **Eliminates Circular Dependency Risk**
   - Current: `runtime/messaging.rs` → `actor/message`
   - Future: Both depend on shared `messaging/` module
   - Enforces one-way dependency chain

3. **Aligns with ADR-WASM-018 Three-Layer Architecture**
   - `core/` → `runtime/` → `actor/` → `messaging/`
   - One-way dependencies only
   - Clear architectural boundaries

4. **Improves Navigation**
   - All messaging code in one place
   - Developers can find messaging infrastructure quickly
   - Reduces cognitive load

5. **Enables Future Block 5 Development**
   - Provides proper foundation for inter-component communication
   - Separates concerns clearly
   - Makes Block 5 development straightforward

### Assumptions

- No existing code depends on `runtime::MessagingService` (can update imports)
- Migration can be done incrementally without breaking builds
- CI checks will prevent future violations
- No performance impact (code organization only)

## Considered Options

### Option 1: Move Messaging to Top-Level Module ✅ **SELECTED**

**Description:** 
- Create new `src/messaging/` module
- Move all code from `runtime/messaging.rs` to `messaging/`
- Update all import statements
- Remove `runtime/messaging.rs`

**Pros:**
- ✅ Corrects architectural violation completely
- ✅ Aligns with ADR-WASM-018 three-layer architecture
- ✅ Eliminates circular dependency risk
- ✅ Clear module boundaries
- ✅ Improves code navigation
- ✅ Enables future Block 5 development
- ✅ Follows KNOWLEDGE-WASM-012 module structure

**Cons:**
- ⚠️ Breaking change: Import paths change from `runtime::` to `messaging/`
- ⚠️ Requires updating all imports (but straightforward search-and-replace)
- ⚠️ Effort: 4.5-5.5 weeks

**Implementation Effort:** Medium  
**Risk Level:** Low

### Option 2: Keep Messaging in Runtime with Better Separation

**Description:**
- Keep `runtime/messaging.rs` in place
- Add documentation explaining it's "runtime-level messaging"
- Accept the architectural violation

**Pros:**
- ✅ No breaking changes
- ✅ Minimal effort
- ✅ No import updates needed

**Cons:**
- ❌ Violates ADR-WASM-018 three-layer architecture
- ❌ Keeps circular dependency risk
- ❌ Confusing module boundaries
- ❌ Blocks proper Block 5 development
- ❌ Long-term maintenance burden
- ❌ Violates KNOWLEDGE-WASM-012 module structure

**Implementation Effort:** None  
**Risk Level:** High

### Option 3: Split Messaging Between runtime/ and actor/

**Description:**
- Keep some messaging code in `runtime/messaging.rs`
- Move other messaging code to `actor/message/`
- Keep both locations with shared types

**Pros:**
- ✅ Reduces breaking changes
- ✅ Can move incrementally

**Cons:**
- ❌ Still violates clear module boundaries
- ❌ Still has circular dependency risk
- ❌ Confusing split responsibility
- ❌ Harder to navigate (messaging scattered)
- ❌ Violates ADR-WASM-018 three-layer architecture
- ❌ Doesn't solve root problem

**Implementation Effort:** Medium  
**Risk Level:** High

### Option 4: Create Internal messaging/ in Both runtime/ and actor/

**Description:**
- Create `runtime/messaging.rs` and `actor/messaging.rs` as internal modules
- Use re-exports at top level
- Keep both locations separate

**Pros:**
- ✅ No breaking changes at top level
- ✅ Can maintain separation

**Cons:**
- ❌ Still violates one-way dependency chain
- ❌ Runtime messaging can still import from actor
- ❌ Confusing duplicate module names
- ❌ Harder to understand which messaging code is where
- ❌ Violates ADR-WASM-018

**Implementation Effort:** High  
**Risk Level:** High

## Implementation

### Implementation Plan

**Phase 1: Create Top-Level messaging/ Module (Days 1-2)**
1. Create `src/messaging/mod.rs` with module declarations only
2. Create `src/messaging/messaging_service.rs` with code from runtime/messaging.rs
3. Create remaining messaging submodules (router, fire_and_forget, etc.)
4. Update `src/lib.rs` to declare new messaging module
5. Add messaging types to `src/prelude.rs`

**Phase 2: Update All Import Statements (Days 2-3)**
1. Update imports in `actor/message/` modules
2. Update imports in `runtime/` modules  
3. Update imports in all integration tests
4. Update imports in examples
5. Search for all `use airssys_wasm::runtime::Messaging*` and replace

**Phase 3: Remove runtime/messaging.rs (Days 3-4)**
1. Verify all imports updated (cargo build succeeds)
2. Delete `src/runtime/messaging.rs`
3. Remove messaging from `src/runtime/mod.rs` re-exports
4. Run all tests to ensure nothing broken

**Phase 4: Add Architecture Compliance Checks (Days 4-5)**
1. Create CI script to check layer dependencies
2. Add script to GitHub Actions workflow
3. Create architecture compliance tests
4. Verify no circular dependencies (grep checks)

**Phase 5: Comprehensive Testing (Days 5-7)**
1. Run all existing tests
2. Add integration tests for messaging module
3. Verify end-to-end message flow
4. Performance benchmark validation
5. Documentation updates

### Timeline

**Total Effort**: 4.5-5.5 weeks (per WASM-TASK-HOTFIX-001)

### Resources Required

- Developer time: ~40 hours
- CI infrastructure: Scripts and workflow updates
- Testing: Integration tests and benchmarks
- Documentation: Migration guide and updated examples

### Dependencies

- **KNOWLEDGE-WASM-012**: Module Structure Architecture (must follow)
- **ADR-WASM-018**: Three-Layer Architecture (must enforce)
- **ADR-WASM-023**: Module Boundary Enforcement (must comply)
- **WASM-TASK-HOTFIX-001**: Implementation task

## Implications

### System Impact

**Positive:**
- ✅ Fixes critical architectural violation
- ✅ Eliminates circular dependency risk
- ✅ Improves code navigation and maintainability
- ✅ Enables proper Block 5 development
- ✅ Clearer module boundaries
- ✅ Aligns with architecture documents

**Negative:**
- ⚠️ Breaking change: Import paths change
- ⚠️ Requires updating external code references (if any exist)
- ⚠️ Short-term effort for migration

### Performance Impact

**Zero Performance Impact:**
- Code organization change only
- No behavioral changes
- Identical runtime performance
- Same code, different location

### Security Impact

**No Security Impact:**
- Security behavior unchanged
- Capability checks still enforced
- Same security model

### Scalability Impact

**Positive:**
- Easier to add messaging features
- Clear module boundaries for future enhancements
- Better separation of concerns

### Maintainability Impact

**Positive:**
- Clearer code organization
- Easier navigation for developers
- Reduced cognitive load
- Easier onboarding for new contributors

## Compliance

### Workspace Standards

**Standards Applied:**
- **§4.3**: Module Architecture (mod.rs declaration-only pattern)
- **§4.3**: Three-layer import organization (one-way dependencies)
- **§6.1**: YAGNI Principles (simple, correct structure)

**Compliance Impact:**
- ✅ Fixes ADR-WASM-018 three-layer architecture violation
- ✅ Aligns with KNOWLEDGE-WASM-012 module structure
- ✅ Enforces ADR-WASM-023 module boundary requirements
- ✅ Provides clean architecture foundation

### Technical Debt

**Debt Resolved:**
- ✅ DEBT-WASM-027: Duplicate WASM Runtime Fatal Architecture Violation (partial fix)
- ✅ DEBT-WASM-028: Circular Dependency Actor Runtime (partial fix)
- ✅ DEBT-WASM-004: Message Delivery Runtime Glue Missing (enables proper fix)

**Debt Created:**
- None - This is a remediation, not new debt

## Monitoring and Validation

### Success Criteria

**Phase 1 Complete:**
- ✅ `src/messaging/` module created with all code moved
- ✅ `src/lib.rs` updated with messaging module
- ✅ `src/prelude.rs` updated with messaging exports
- ✅ Code compiles (`cargo build` succeeds)

**Phase 2 Complete:**
- ✅ All imports updated from `runtime::Messaging*` to `messaging::Messaging*`
- ✅ No remaining imports of `runtime::MessagingService`
- ✅ All code compiles after import updates

**Phase 3 Complete:**
- ✅ `src/runtime/messaging.rs` deleted
- ✅ `src/runtime/mod.rs` no longer exports messaging types
- ✅ All tests pass after removal

**Phase 4 Complete:**
- ✅ CI layer dependency check script created
- ✅ Script integrated into GitHub Actions workflow
- ✅ Architecture compliance tests created and passing
- ✅ No circular dependencies (grep checks return nothing)

**Phase 5 Complete:**
- ✅ All tests pass (cargo test)
- ✅ Zero compiler warnings
- ✅ Zero clippy warnings
- ✅ Integration tests for messaging module pass
- ✅ End-to-end messaging verified
- ✅ Documentation updated

### Key Metrics

**Pre-Refactoring:**
- Module architecture violation present
- `runtime/messaging.rs`: 1,313 lines
- Circular dependency risk
- No top-level messaging/ module

**Post-Refactoring:**
- Module architecture compliant with ADR-WASM-018
- `src/messaging/`: ~1,300 lines (same code, better location)
- No circular dependencies
- Clear module boundaries

### Review Schedule

**Review After:** Phase 5 completion (post-implementation)

**Review Points:**
- Module architecture compliance verified
- All imports updated correctly
- No circular dependencies
- Documentation accurate
- Tests comprehensive

## Risks and Mitigations

### Identified Risks

| Risk | Likelihood | Impact | Mitigation |
|-------|-------------|---------|-------------|
| **Breaking import paths** | High | Medium | Comprehensive search for all imports; provide deprecation warnings; update documentation |
| **Missed imports during update** | Medium | Medium | Use grep to find all references; run all tests; incremental verification |
| **External code references** | Low | Medium | Document breaking changes; provide migration guide; semantic version bump |
| **Circular dependency reintroduced** | Low | High | CI enforcement checks; architecture compliance tests; code review process |

### Contingency Plans

**Plan A (Primary):** Execute refactoring as planned in 4.5-5.5 weeks

**Plan B (Fallback):** If blocking issues arise, defer messaging/ submodules to future phase, focus only on moving core MessagingService

**Plan C (Rollback):** Keep re-exports in runtime/ as deprecated aliases, document migration path clearly

## References

### Related Documents

**ADRs:**
- **ADR-WASM-018**: Three-Layer Architecture (PRIMARY REFERENCE - must follow)
- **ADR-WASM-023**: Module Boundary Enforcement (enforcement rules)
- **ADR-WASM-011**: Module Structure Organization

**Knowledge:**
- **KNOWLEDGE-WASM-002**: High-Level Overview
- **KNOWLEDGE-WASM-003**: Core Architecture Design
- **KNOWLEDGE-WASM-012**: Module Structure Architecture (PRIMARY REFERENCE - lines 506-596)
- **KNOWLEDGE-WASM-024**: Component Messaging Clarifications
- **KNOWLEDGE-WASM-029**: Messaging Patterns
- **KNOWLEDGE-WASM-034**: Module Architecture Violation - Messaging in Runtime

**Technical Debt:**
- **DEBT-WASM-004**: Message Delivery Runtime Glue Missing
- **DEBT-WASM-027**: Duplicate WASM Runtime Fatal Architecture Violation
- **DEBT-WASM-028**: Circular Dependency Actor Runtime

**Task:**
- **WASM-TASK-HOTFIX-001**: Messaging Module Architecture Refactoring (implementation)

### External References
- Workspace Standards: `.aiassisted/instructions/multi-project-memory-bank.instructions.md`
- Rust Book - Modules: https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html

## History

### Status Changes
- **[2025-12-26]:** Status set to Accepted - Architectural decision made

### Updates
- **[2025-12-26]:** Initial creation - Complete ADR with all sections filled

### Reviews
- **[2025-12-26]:** Accepted by @hiraq - Architecture decision aligns with ADR-WASM-018 and KNOWLEDGE-WASM-012

---

**Template Version:** 1.0  
**Last Updated:** 2025-12-26
