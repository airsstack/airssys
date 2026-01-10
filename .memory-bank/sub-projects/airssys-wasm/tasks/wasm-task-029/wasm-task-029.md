# WASM-TASK-029: Create airssys-osl Bridge

**Status:** pending
**Added:** 2026-01-10
**Updated:** 2026-01-10
**Priority:** high
**Estimated Duration:** 2-3 hours
**Phase:** Phase 4 - Security Module (Layer 2A)

## Original Request
Create the bridge to airssys-osl SecurityContext for hierarchical security integration.

## Thought Process
This task integrates with the external airssys-osl crate for hierarchical security. The OslSecurityBridge:
- Wraps airssys-osl SecurityContext
- Provides OSL permission checking before capability validation
- Integrates with RBAC/ACL from airssys-osl

## Deliverables
- [ ] OslSecurityBridge struct in `security/mod.rs`
- [ ] Integration with airssys-osl SecurityContext
- [ ] check_osl_permission method
- [ ] Unit tests for OSL bridge (mocked if needed)

## Success Criteria
- [ ] `cargo build -p airssys-wasm` succeeds
- [ ] `cargo clippy -p airssys-wasm --all-targets -- -D warnings` passes
- [ ] OSL bridge correctly wraps SecurityContext
- [ ] Permission checking integrates with airssys-osl
- [ ] Types align with ADR-WASM-029 specifications

## Progress Tracking
**Overall Status:** 0% complete

## Progress Log
*(No entries yet)*

## Standards Compliance Checklist
- [ ] §2.1 3-Layer Import Organization
- [ ] §5.1 Dependency Management
- [ ] §6.4 Quality Gates (zero warnings)
- [ ] ADR-WASM-029 Security Module Design
- [ ] KNOWLEDGE-WASM-020 OSL Security Integration

## Dependencies
- **Upstream:**
  - WASM-TASK-026 (CapabilityValidator) - for security validation
  - airssys-osl crate (external dependency)
- **Downstream:** WASM-TASK-030 (security/ unit tests)

## Definition of Done
- [ ] All deliverables complete
- [ ] All success criteria met
- [ ] Build passes with zero warnings
- [ ] Unit tests pass
- [ ] OSL integration documented
