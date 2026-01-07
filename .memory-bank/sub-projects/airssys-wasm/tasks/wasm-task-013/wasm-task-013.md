# WASM-TASK-013: Rename actor/ to component/

**Status:** pending  
**Added:** 2026-01-07  
**Updated:** 2026-01-07  
**Priority:** high  
**Estimated Duration:** 1 hour  
**Phase:** Phase 2 - Project Restructuring

## Original Request
Rename the `actor/` directory to `component/` to align with WASM Component Model terminology per ADR-WASM-025 and ADR-WASM-026.

## Thought Process
The clean-slate rebuild architecture uses "component" terminology instead of "actor" to align with:
1. WASM Component Model naming conventions
2. WIT interface terminology (component-lifecycle.wit, ComponentId, etc.)
3. Industry-standard WebAssembly terminology

This is a simple rename operation that maintains the same layer position (Layer 3A).

## Deliverables
- [ ] `src/actor/` renamed to `src/component/`
- [ ] `src/component/mod.rs` documentation updated to reflect new name
- [ ] All references to "actor" in module documentation updated to "component"

## Success Criteria
- [ ] `cargo build -p airssys-wasm` succeeds
- [ ] `cargo clippy -p airssys-wasm --all-targets -- -D warnings` passes
- [ ] No references to "actor" module in `lib.rs` after update
- [ ] Module documentation reflects "component" terminology

## Progress Tracking
**Overall Status:** 0% complete

## Progress Log
*(No entries yet)*

## Standards Compliance Checklist
- [ ] **§4.3 Module Architecture Patterns** - mod.rs contains only declarations
- [ ] **ADR-WASM-025** - Clean-slate rebuild architecture compliance
- [ ] **ADR-WASM-026** - Phase 2 task compliance
- [ ] **KNOWLEDGE-WASM-037** - Component terminology alignment

## Dependencies
- **Upstream:** WASM-TASK-012 (wit-bindgen integration) ✅ Complete
- **Downstream:** WASM-TASK-014 (Create system/ module)

## Definition of Done
- [ ] All deliverables complete
- [ ] All success criteria met
- [ ] Build passes with zero warnings
- [ ] Directory renamed and documented
