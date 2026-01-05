# WASM-TASK-003: Create types.wit

**Status:** pending  
**Added:** 2026-01-05  
**Updated:** 2026-01-05  
**Priority:** high  
**Estimated Duration:** 1 day

## Original Request

Create the `types.wit` interface file containing foundation types for all other WIT interfaces (Phase 1, Step 3).

## Thought Process

`types.wit` is Layer 0 - the foundation for all other interfaces. It defines core types like `component-id`, `component-message`, `message-payload`, timestamps, and enums. Every other interface will `use types.{...}` to import these types. Must be created before any other interface files.

## Deliverables

- [ ] `wit/core/types.wit` file created with all foundation types
- [ ] Package declaration: `package airssys:core@1.0.0;`
- [ ] Interface declaration: `interface types { ... }`
- [ ] All types from ADR-WASM-027 lines 60-143 implemented
- [ ] File validated with `wasm-tools component wit`

## Success Criteria

- [ ] File content matches ADR-WASM-027 specification (lines 58-144)
- [ ] All types compile without errors
- [ ] Package and interface declarations correct
- [ ] WIT validation passes

## Progress Tracking

**Overall Status:** 0% complete

## Progress Log

*No progress yet*

## Standards Compliance Checklist

- [ ] **ADR-WASM-027** - WIT Interface Design (types.wit specification)
- [ ] **KNOWLEDGE-WASM-013** - Core WIT Package Structure
- [ ] WIT syntax follows Component Model standards
- [ ] Documentation comments included for all types

## Definition of Done

- [ ] All deliverables complete
- [ ] All success criteria met
- [ ] Validation commands pass
- [ ] Ready for WASM-TASK-004 (errors.wit can reference types)
