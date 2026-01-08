# WASM-TASK-017: Create core/component/ Submodule

**Status:** pending  
**Added:** 2026-01-08  
**Updated:** 2026-01-08  
**Priority:** high  
**Estimated Duration:** 2-3 hours  
**Phase:** Phase 3 - Core Module (Layer 1)

## Original Request
Create the `core/component/` submodule containing foundation types for component identity, handles, and messages per ADR-WASM-028.

## Thought Process
This is the first task of Phase 3 that establishes the component-related core types. These types will be used by all other modules. The component submodule includes:
- `ComponentId` - Unique identifier for component instances
- `ComponentHandle` - Opaque handle to loaded components
- `ComponentMessage` - Message envelope for component communication
- `MessageMetadata` - Metadata for messages
- `ComponentLifecycle` trait - Lifecycle management abstraction

## Deliverables
- [ ] `core/component/mod.rs` created with module declarations
- [ ] `core/component/id.rs` with `ComponentId` struct
- [ ] `core/component/handle.rs` with `ComponentHandle` struct
- [ ] `core/component/message.rs` with `ComponentMessage` and `MessageMetadata`
- [ ] `core/component/traits.rs` with `ComponentLifecycle` trait
- [ ] `core/mod.rs` updated to export component submodule

## Success Criteria
- [ ] `cargo build -p airssys-wasm` succeeds
- [ ] `cargo clippy -p airssys-wasm --all-targets -- -D warnings` passes
- [ ] `core/component/` imports only `std` (no external crates)
- [ ] All types properly documented with rustdoc
- [ ] Types align with ADR-WASM-028 specifications

## Progress Tracking
**Overall Status:** 0% complete

## Progress Log
*(No progress yet)*


## Related Documentation

### Knowledge Documents
- **KNOWLEDGE-WASM-038:** Component Module Responsibility and Architecture (two-layer distinction, core/component/ vs component/)
- **KNOWLEDGE-WASM-037:** Rebuild Architecture - Clean Slate Design
- **KNOWLEDGE-WASM-031:** Foundational Architecture
## Standards Compliance Checklist
- [ ] **§2.1 3-Layer Import Organization** - Only std imports
- [ ] **§4.3 Module Architecture Patterns** - mod.rs only declarations
- [ ] **ADR-WASM-028** - Core module structure compliance
- [ ] **ADR-WASM-025** - Clean-slate rebuild architecture
- [ ] **KNOWLEDGE-WASM-037** - Technical reference alignment

## Dependencies
- **Upstream:** WASM-TASK-016 (Update lib.rs exports) ✅ COMPLETE
- **Downstream:** WASM-TASK-018, 019, 020, 021, 022, 023 (other core submodules)

## Definition of Done
- [ ] All deliverables complete
- [ ] All success criteria met
- [ ] Build passes with zero warnings
- [ ] Core types ready for use by other modules
