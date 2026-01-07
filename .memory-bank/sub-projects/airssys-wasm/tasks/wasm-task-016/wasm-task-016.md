# WASM-TASK-016: Update lib.rs Exports

**Status:** pending  
**Added:** 2026-01-07  
**Updated:** 2026-01-07  
**Priority:** high  
**Estimated Duration:** 1 hour  
**Phase:** Phase 2 - Project Restructuring

## Original Request
Update `lib.rs` to export the new module structure (component/, messaging/, system/) per ADR-WASM-026.

## Thought Process
This is the final task of Phase 2 that:
1. Updates lib.rs to declare all new modules
2. Updates module documentation to reflect new architecture
3. Updates the dependency diagram in documentation
4. Ensures all modules are properly exported

## Deliverables
- [ ] `lib.rs` updated to declare `component` instead of `actor`
- [ ] `lib.rs` updated to declare new `messaging` module
- [ ] `lib.rs` updated to declare new `system` module
- [ ] Module documentation/diagram updated to reflect 6-module structure
- [ ] Prelude updated if necessary

## Success Criteria
- [ ] `cargo build -p airssys-wasm` succeeds
- [ ] `cargo clippy -p airssys-wasm --all-targets -- -D warnings` passes
- [ ] All 6 modules declared: core, security, runtime, component, messaging, system
- [ ] Documentation diagram reflects new dependency structure
- [ ] `cargo doc -p airssys-wasm` generates correct module documentation

## Progress Tracking
**Overall Status:** 0% complete

## Progress Log
*(No entries yet)*

## Standards Compliance Checklist
- [ ] **§2.1 3-Layer Import Organization**
- [ ] **§4.3 Module Architecture Patterns**
- [ ] **ADR-WASM-025** - Clean-slate rebuild architecture compliance
- [ ] **ADR-WASM-026** - Phase 2 task compliance
- [ ] **KNOWLEDGE-WASM-037** - Module structure alignment

## Dependencies
- **Upstream:** WASM-TASK-015 (Create messaging/ module)
- **Downstream:** WASM-TASK-017 (Phase 3: Core module implementation)

## Definition of Done
- [ ] All deliverables complete
- [ ] All success criteria met
- [ ] Build passes with zero warnings
- [ ] Phase 2 complete, ready for Phase 3
