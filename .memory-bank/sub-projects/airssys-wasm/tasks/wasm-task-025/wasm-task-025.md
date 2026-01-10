# WASM-TASK-025: Create security/capability/ Submodule

**Status:** pending
**Added:** 2026-01-10
**Updated:** 2026-01-10
**Priority:** high
**Estimated Duration:** 2-3 hours
**Phase:** Phase 4 - Security Module (Layer 2A)

## Original Request
Create the `security/capability/` submodule containing capability types, CapabilitySet, and CapabilityGrant per ADR-WASM-029.

## Thought Process
This task implements capability management types that extend the core capability definitions. The security module (Layer 2A) depends on core/ types from WASM-TASK-020 (core/security/) and implements concrete permission management.

Key components:
- PatternMatcher for glob-style pattern matching
- CapabilitySet for managing component permissions
- Permission structs for each capability type (Messaging, Storage, Filesystem, Network)
- CapabilityGrant for permission grants

## Deliverables
- [ ] `security/capability/mod.rs` created with module declarations
- [ ] `security/capability/types.rs` with PatternMatcher and core re-exports
- [ ] `security/capability/set.rs` with CapabilitySet and permission structs
- [ ] `security/capability/grant.rs` with CapabilityGrant
- [ ] `security/mod.rs` updated (or created) with capability submodule declaration
- [ ] Unit tests for PatternMatcher and CapabilitySet

## Success Criteria
- [ ] `cargo build -p airssys-wasm` succeeds
- [ ] `cargo clippy -p airssys-wasm --all-targets -- -D warnings` passes
- [ ] PatternMatcher correctly matches glob patterns
- [ ] CapabilitySet permission checks work correctly
- [ ] Types align with ADR-WASM-029 specifications

## Progress Tracking
**Overall Status:** 0% complete

## Progress Log
*(No entries yet)*

## Standards Compliance Checklist
- [ ] §2.1 3-Layer Import Organization
- [ ] §4.3 Module Architecture Patterns (mod.rs only declarations)
- [ ] §6.1 YAGNI Principles
- [ ] §6.4 Quality Gates (zero warnings)
- [ ] ADR-WASM-029 Security Module Design
- [ ] ADR-WASM-023 Module Boundary Enforcement

## Dependencies
- **Upstream:**
  - WASM-TASK-024 (Core unit tests) ✅
  - WASM-TASK-020 (core/security/) - for core capability types ✅
- **Downstream:** WASM-TASK-026, WASM-TASK-027, WASM-TASK-028, WASM-TASK-029

## Definition of Done
- [ ] All deliverables complete
- [ ] All success criteria met
- [ ] Build passes with zero warnings
- [ ] Clippy passes with zero warnings
- [ ] Unit tests pass
- [ ] Architecture verification passed
