# WASM-TASK-010: Create world.wit

**Status:** pending  
**Added:** 2026-01-05  
**Updated:** 2026-01-05  
**Priority:** high  
**Estimated Duration:** 0.5 days

## Original Request

Create the `world.wit` file that ties together all imports and exports into the component world definition (Phase 1, Step 10).

## Thought Process

`world.wit` defines the complete component world - the contract that all guest components implement. It specifies what interfaces the host provides (imports) and what interfaces guests must export. This is the culmination of all previous interface definitions.

## Deliverables

- [ ] `wit/core/world.wit` file created
- [ ] Package declaration
- [ ] World `component` defined
- [ ] All host-provided interfaces imported (host-messaging, host-services, storage)
- [ ] Guest export interface defined (component-lifecycle)
- [ ] File validated with `wasm-tools component wit`

## Success Criteria

- [ ] File content matches ADR-WASM-027 specification (lines 462-477)
- [ ] World definition correct
- [ ] All interface references valid
- [ ] WIT validation passes

## Progress Tracking

**Overall Status:** 0% complete

## Progress Log

*No progress yet*

## Standards Compliance Checklist

- [ ] **ADR-WASM-027** - WIT Interface Design (world.wit specification)
- [ ] **KNOWLEDGE-WASM-013** - Core WIT Package Structure

## Definition of Done

- [ ] All deliverables complete
- [ ] All success criteria met
- [ ] Validation commands pass
- [ ] Ready for WASM-TASK-011 (Validate WIT package)
