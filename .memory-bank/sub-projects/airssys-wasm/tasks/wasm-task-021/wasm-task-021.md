# WASM-TASK-021: Create core/storage/ Submodule

**Status:** pending  
**Added:** 2026-01-08  
**Updated:** 2026-01-10  
**Priority:** high  
**Estimated Duration:** 1-2 hours  
**Phase:** Phase 3 - Core Module (Layer 1)

## Original Request
Create the `core/storage/` submodule containing storage abstractions per ADR-WASM-028.

## Thought Process
This task creates the storage-related core abstraction for component-isolated storage. The storage module follows the Dependency Inversion Principle - traits are defined in Layer 1 (`core/storage/`) and implementations will be provided via host functions or a storage system module.

**Key Design Decision:** Use dedicated `StorageValue` ADT instead of `MessagePayload` for domain boundary clarity. Engineers immediately know the type's purpose from its name.

Key types include:
- `StorageValue` ADT - Dedicated bytes wrapper for storage values
- `StorageError` enum - Co-located error types for storage operations
- `ComponentStorage` trait - Storage abstraction for key-value operations

## Deliverables
- [ ] `wit/core/storage.wit` updated with dedicated `storage-value` type
- [ ] `core/storage/value.rs` with `StorageValue` ADT (dedicated domain type)
- [ ] `core/storage/errors.rs` with `StorageError` enum (5 variants, WIT-aligned)
- [ ] `core/storage/traits.rs` with `ComponentStorage` trait (5 methods)
- [ ] `core/storage/mod.rs` with module declarations and documentation
- [ ] `core/mod.rs` updated to export storage submodule
- [ ] Unit tests for all new types

## Success Criteria
- [ ] `cargo build -p airssys-wasm` succeeds with zero warnings
- [ ] `cargo clippy -p airssys-wasm --all-targets -- -D warnings` passes
- [ ] `cargo test -p airssys-wasm --lib storage` passes (all unit tests)
- [ ] `ComponentStorage` trait uses `MessagePayload` from `core/component/`
- [ ] All types properly documented with rustdoc (summary sentence < 15 words)
- [ ] Types align with ADR-WASM-028 specifications (lines 74-77, 507-533)

## Progress Tracking
**Overall Status:** 0% complete

## Progress Log
*(No progress yet)*

## Standards Compliance Checklist

### Code Organization (PROJECTS_STANDARD.md)
- [ ] **§2.1 3-Layer Import Organization** - All files follow Layer 1/2/3 comments
- [ ] **§2.2 No FQN in Type Annotations** - All types imported, not qualified
- [ ] **§4.3 Module Architecture Patterns** - mod.rs only declarations, no re-exports
- [ ] **§6.2 Avoid `dyn` Patterns** - Trait uses `&self` not `Box<dyn>`

### Architecture Compliance (ADRs)
- [ ] **ADR-WASM-028** - Core module structure (co-located errors, Layer 1 compliance)
- [ ] **ADR-WASM-025** - Clean-slate rebuild architecture (new module from scratch)
- [ ] **KNOWLEDGE-WASM-037** - Technical reference alignment

### Documentation (Microsoft Rust Guidelines)
- [ ] **M-CANONICAL-DOCS** - Canonical sections (Summary, Examples, Errors)
- [ ] **M-FIRST-DOC-SENTENCE** - First sentence < 15 words
- [ ] **M-MODULE-DOCS** - Module documentation in mod.rs

## Verification Commands
```bash
# Build check
cargo build -p airssys-wasm

# Lint check
cargo clippy -p airssys-wasm --all-targets -- -D warnings

# Unit tests
cargo test -p airssys-wasm --lib storage

# Layer 1 compliance (should only show core/component imports)
grep -rn "use crate::" src/core/storage/

# Forbidden imports (should be empty)
grep -rn "use crate::runtime\|use crate::messaging\|use crate::security\|use crate::system" src/core/storage/
```

## Dependencies
- **Upstream:** 
  - WASM-TASK-017 (core/component/) - for `MessagePayload` (prerequisite)
  - ~~WASM-TASK-022 (core/errors/)~~ - **ABANDONED**: StorageError now co-located
- **Downstream:** WASM-TASK-024 (Core unit tests)

## Definition of Done
- [ ] All deliverables complete
- [ ] All success criteria met
- [ ] All standards compliance checks pass
- [ ] Build passes with zero warnings
- [ ] All unit tests pass
- [ ] Storage abstractions ready for implementation
