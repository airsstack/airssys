# WASM-TASK-006: Create component-lifecycle.wit

**Status:** pending  
**Added:** 2026-01-05  
**Updated:** 2026-01-05  
**Priority:** high  
**Estimated Duration:** 1 day

## Original Request

Create the `component-lifecycle.wit` interface file defining the guest component contract (Phase 1, Step 6).

## Thought Process

`component-lifecycle.wit` is Layer 2 - the GUEST EXPORTS interface. This defines what guest components MUST implement: initialization, message handling, callbacks, metadata, health checks, and shutdown. This is the core contract between host runtime and guest components.

## Deliverables

- [ ] `wit/core/component-lifecycle.wit` file created
- [ ] Package and interface declarations
- [ ] `use` statements for types and errors
- [ ] All functions from ADR-WASM-027 lines 297-337 implemented
- [ ] File validated with `wasm-tools component wit`

## Success Criteria

- [ ] File content matches ADR-WASM-027 specification (lines 297-338)
- [ ] All lifecycle functions defined correctly
- [ ] Guest export semantics correct
- [ ] WIT validation passes

## Progress Tracking

**Overall Status:** 0% complete

## Progress Log

*No progress yet*

## Standards Compliance Checklist

- [ ] **ADR-WASM-027** - WIT Interface Design (component-lifecycle.wit specification)
- [ ] **KNOWLEDGE-WASM-013** - Core WIT Package Structure
- [ ] Guest export contract properly defined

## Definition of Done

- [ ] All deliverables complete
- [ ] All success criteria met
- [ ] Validation commands pass
- [ ] Ready for WASM-TASK-007 (host-messaging.wit)
