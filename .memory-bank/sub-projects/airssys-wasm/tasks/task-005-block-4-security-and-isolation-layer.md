# [WASM-TASK-005] - Block 4: Security & Isolation Layer

**Status:** not-started  
**Added:** 2025-10-20  
**Updated:** 2025-12-13  
**Priority:** 🔒 CRITICAL PATH - Security Layer + Task 1.3 Deferred Work  
**Layer:** 2 - Core Services  
**Block:** 4 of 11  
**Estimated Effort:** 5-6 weeks + 16-20 hours (Task 1.3 deferred work)

**⚠️ CRITICAL PREREQUISITES - MUST COMPLETE BEFORE STARTING ⚠️**

Before starting Block 4, you MUST complete:
- **DEBT-WASM-004 Item #3**: Capability Enforcement in InterComponent Messages
  - Location: `src/actor/actor_impl.rs` lines 223-228
  - Estimated: 16-20 hours
  - **SECURITY CRITICAL** - System vulnerable until implemented
  - See: `.memory-bank/sub-projects/airssys-wasm/docs/technical-debt/debt-wasm-004-task-1.3-deferred-implementation.md`

**📋 Mandatory Checklist:**
- [ ] Review DEBT-WASM-004 Item #3 requirements
- [ ] Implement capability checking in actor_impl.rs InterComponent handler
- [ ] Remove "FUTURE WORK" comments from lines 223-228
- [ ] Security tests achieve ≥95% coverage
- [ ] Security audit passed
- [ ] Sign-off obtained (see DEBT-WASM-004)

## Overview

Implement the multi-layered security and isolation system that protects the host from malicious or buggy components through fine-grained capability-based permissions, pattern matching for resources, trust-level workflows, actor isolation, and supervision trees. This block establishes the security foundation that all host function access depends upon.

## Context

**Current State:**
- Architecture complete: ADR-WASM-005 (Capability-Based Security Model)
- Foundation ready: Block 3 (Actor System Integration) provides ComponentActor isolation
- Block 1 provides: WASM memory sandboxing (512KB-4MB isolated linear memory)
- Security model: Deny-by-default, least privilege, explicit declaration

**Problem Statement:**
Components require access to host resources (filesystem, network, storage) but:
1. **Threat: Resource Abuse** - Component declares "read config.toml" but reads /etc/passwd
2. **Threat: Network Abuse** - Component declares "call api.example.com" but exfiltrates data
3. **Threat: Resource Exhaustion** - Component declares "100MB storage" but writes 10GB
4. **Challenge: Developer Experience** - Security shouldn't block legitimate development

The framework needs multi-layered defense:
- **Layer 1**: Capability-based security (permission checks at host functions)
- **Layer 2**: WASM linear memory sandbox (bounds checking)
- **Layer 3**: Actor isolation (private mailbox, message passing only)
- **Layer 4**: Supervision trees (automatic restart, health monitoring)

**Why This Block Matters:**
Without this security layer:
- Components could access unauthorized files/network/storage
- No defense against malicious third-party components
- Host system vulnerable to compromise
- No production-ready trust model

This block enables safe execution of untrusted third-party components.

## Objectives

### Primary Objective
Implement fine-grained capability-based security with pattern matching (filesystem globs, network domains, storage namespaces), trust-level workflows (trusted instant install, unknown review), and integration with actor isolation and supervision to create defense-in-depth protection.

### Secondary Objectives
- Achieve <1-5μs capability check overhead (0.1% of typical operation)
- Implement trust-level system for developer-friendly workflows
- Integrate with airssys-osl RBAC/ACL for layered security
- Create comprehensive security audit logging
- Establish dev mode for rapid iteration with warnings

## Scope

### In Scope
1. **Capability Pattern Matching** - Filesystem globs, network domains, storage namespaces
2. **Component.toml Declaration** - Capability manifest parsing and validation
3. **Trust-Level System** - Trusted instant, unknown review, dev mode bypass
4. **Host Function Enforcement** - Entry point capability checks
5. **Actor Isolation Patterns** - ComponentActor security boundaries
6. **Supervision Trees** - Automatic restart and health monitoring
7. **Security Audit Logging** - All access attempts logged
8. **airssys-osl Integration** - Layered RBAC/ACL integration

### Out of Scope
- Actual host function implementations (Block 8)
- Storage backend implementation (Block 6)
- Network operations (Block 8)
- Component installation workflow (Block 7)
- Custom capability types (Phase 2+)

## Implementation Plan

### Phase 1: Capability System Foundation (Week 1-2)

#### Task 1.1: Capability Data Structures
**Deliverables:**
- `Capability` enum (Filesystem, Network, Storage, Process, etc.)
- `FilesystemCapability` struct (read/write patterns)
- `NetworkCapability` struct (inbound/outbound patterns)
- `StorageCapability` struct (namespace patterns)
- Capability set data structure
- Capability documentation

**Success Criteria:**
- All capability types defined
- Data structures support pattern matching
- Efficient storage and lookup
- Clear API for capability checking

#### Task 1.2: Pattern Matching Engine
**Deliverables:**
- Glob pattern parser for filesystem (`*`, `**`, `?`, `[abc]`)
- Domain pattern matcher for network (wildcards, exact match)
- Namespace pattern matcher for storage (prefix matching)
- Pattern validation (reject invalid patterns)
- Pattern matching tests
- Performance benchmarks (<1μs per check)

**Success Criteria:**
- Filesystem globs work correctly (`/etc/**`, `/tmp/*.txt`)
- Domain wildcards work (`*.example.com`)
- Storage namespaces work (`component:<id>:*`)
- Invalid patterns rejected with clear errors
- Pattern matching <1μs average

#### Task 1.3: Component.toml Capability Parser
**Deliverables:**
- Parse `[capabilities]` section from Component.toml
- Capability declaration validation
- Required vs optional capabilities distinction
- Capability set construction from manifest
- Parsing error handling with clear messages
- Parser tests

**Success Criteria:**
- Component.toml capabilities parsed correctly
- Invalid declarations rejected
- Required capabilities distinguished from optional
- Clear error messages for malformed declarations
- Comprehensive test coverage

---

### Phase 2: Trust-Level System (Week 2-3)

#### Task 2.1: Trust Level Implementation
**Deliverables:**
- `TrustLevel` enum (Trusted, Unknown, DevMode)
- Trust level determination logic
- Trust source registry (Git repos, signing keys)
- Trust level configuration system
- Trust level documentation

**Success Criteria:**
- Three trust levels implemented
- Trust sources configurable
- Clear trust determination rules
- Documentation for trust management

#### Task 2.2: Approval Workflow Engine
**Deliverables:**
- Approval workflow state machine
- Trusted source auto-approval (instant install)
- Unknown source review queue
- DevMode capability bypass with warnings
- Approval decision persistence
- Workflow tests

**Success Criteria:**
- Trusted sources install instantly
- Unknown sources enter review queue
- DevMode bypasses with logged warnings
- Approval decisions persist across restarts
- Clear workflow documentation

#### Task 2.3: Trust Configuration System
**Deliverables:**
- Trust configuration file format
- Trusted Git repository configuration
- Trusted signing key configuration
- DevMode enable/disable controls
- Configuration validation
- Configuration documentation

**Success Criteria:**
- Trust sources configurable via file
- Git repos and signing keys supported
- DevMode configurable
- Configuration errors caught early
- Clear configuration documentation

---

### Phase 3: Host Function Enforcement (Week 3-4)

#### Task 3.1: Capability Check API
**Deliverables:**
- `check_capability()` function for host functions
- Capability context (component ID, requested resource)
- Check result types (Granted, Denied, reasons)
- Performance optimized checks (<5μs)
- Capability check documentation

**Success Criteria:**
- Host functions can check capabilities easily
- Check includes component context
- Clear granted/denied results with reasons
- Performance target met (<5μs)
- Easy to use API

#### Task 3.2: Host Function Entry Points
**Deliverables:**
- Capability check integration points
- Filesystem host function checks (read/write/stat)
- Network host function checks (connect/bind)
- Storage host function checks (get/set/delete)
- Check failure error responses
- Integration tests

**Success Criteria:**
- All host function categories covered
- Checks enforce capability patterns
- Denied access returns clear errors
- No bypass vulnerabilities
- Comprehensive integration tests

#### Task 3.3: Audit Logging Integration
**Deliverables:**
- Security audit log format
- Log all capability checks (granted + denied)
- Log component context (ID, capability, resource)
- Log trust level and approval status
- Structured logging (JSON or similar)
- Audit log documentation

**Success Criteria:**
- All access attempts logged
- Logs include full context
- Structured format for analysis
- Performance overhead minimal
- Clear audit trail

---

### Phase 4: Actor Isolation Integration (Week 4)

#### Task 4.1: ComponentActor Security Boundaries
**Deliverables:**
- Security context per ComponentActor
- Capability set per component instance
- Isolation verification tests
- Actor-to-actor security boundaries
- Security boundary documentation

**Success Criteria:**
- Each ComponentActor has isolated capability set
- Components cannot access each other's resources
- Security boundaries enforced by actor system
- Isolation verified through testing
- Clear security documentation

#### Task 4.2: Message Passing Security
**Deliverables:**
- Message authorization checks
- Topic subscription capability requirements
- Message content validation
- Quota enforcement for messaging
- Message security tests

**Success Criteria:**
- Message sending requires capabilities
- Topic subscriptions authorized
- Message quotas prevent spam
- No message-based bypass vulnerabilities
- Comprehensive security tests

#### Task 4.3: Resource Quota System
**Deliverables:**
- Storage quota tracking per component
- Message rate limiting per component
- Network bandwidth quotas (Phase 2)
- Quota enforcement in host functions
- Quota configuration and monitoring

**Success Criteria:**
- Storage quotas enforced
- Message rate limits enforced
- Quota violations handled gracefully
- Quotas configurable per component
- Quota monitoring available

---

### Phase 5: Supervision and Health Monitoring (Week 5)

#### Task 5.1: SupervisorNode Security Integration
**Deliverables:**
- Supervisor restart policies with security context
- Security state restoration on restart
- Failed component cleanup
- Restart limits based on security violations
- Supervision security documentation

**Success Criteria:**
- Components restart with same capabilities
- Security violations trigger stricter restart policies
- Failed components cleaned up properly
- Restart storms prevented
- Clear supervision security model

#### Task 5.2: Health Monitoring with Security
**Deliverables:**
- Health checks with security context
- Security violation detection in health checks
- Automatic response to security issues
- Health status security reporting
- Health monitoring documentation

**Success Criteria:**
- Health checks include security status
- Security violations detected and reported
- Automatic remediation (restart/isolate)
- Clear health-security integration
- Comprehensive monitoring

#### Task 5.3: Graceful Degradation
**Deliverables:**
- Capability reduction on security violations
- Component isolation escalation
- Security incident response automation
- Recovery procedures
- Degradation documentation

**Success Criteria:**
- Security violations trigger capability reduction
- Repeated violations increase isolation
- Automated incident response
- Recovery path clear
- Well-documented procedures

---

### Phase 6: airssys-osl Integration and Testing (Week 5-6)

#### Task 6.1: Layered Security Integration
**Deliverables:**
- airssys-osl RBAC/ACL integration
- Multi-layer security enforcement (capabilities → RBAC → OS)
- Permission translation between layers
- Integration error handling
- Layered security documentation

**Success Criteria:**
- Capabilities layer works with RBAC/ACL
- Three-layer defense operational
- Permission translation correct
- No security bypasses between layers
- Clear integration documentation

#### Task 6.2: Comprehensive Security Testing
**Deliverables:**
- Security test suite (positive and negative tests)
- Bypass attempt tests (malicious component scenarios)
- Pattern matching edge cases
- Performance benchmarks (<5μs overhead)
- Penetration testing framework

**Success Criteria:**
- All capability patterns tested
- Bypass attempts detected and blocked
- Edge cases covered
- Performance targets met
- No security vulnerabilities found

#### Task 6.3: Security Documentation and Examples
**Deliverables:**
- Capability declaration guide
- Trust level configuration guide
- Security best practices documentation
- Example secure components
- Security troubleshooting guide

**Success Criteria:**
- Complete security documentation
- Clear capability declaration examples
- Best practices actionable
- Examples demonstrate security patterns
- Troubleshooting guide comprehensive

---

## Success Criteria

### Definition of Done
This task is complete when:

1. ✅ **Capability System Operational**
   - Pattern matching works (filesystem globs, network domains, storage namespaces)
   - Component.toml capabilities parsed and validated
   - Capability checks <5μs overhead

2. ✅ **Trust-Level System Working**
   - Trusted sources install instantly (auto-approval)
   - Unknown sources enter review queue
   - DevMode enables rapid development with warnings
   - Trust configuration functional

3. ✅ **Host Function Enforcement Active**
   - All host function categories have capability checks
   - Denied access returns clear errors
   - Security audit logging operational
   - No bypass vulnerabilities

4. ✅ **Actor Isolation Integrated**
   - ComponentActor security boundaries enforced
   - Message passing security functional
   - Resource quotas enforced (storage, messaging)
   - Isolation verified through testing

5. ✅ **Supervision with Security**
   - SupervisorNode restarts with security context
   - Health monitoring includes security status
   - Graceful degradation on violations
   - Automatic incident response

6. ✅ **airssys-osl Integration Complete**
   - Layered security operational (capabilities → RBAC → OS)
   - Permission translation correct
   - Defense-in-depth verified
   - Integration tested thoroughly

7. ✅ **Testing & Documentation Complete**
   - Security test suite passing (>95% coverage)
   - Bypass attempts blocked
   - Performance benchmarks met
   - Complete security documentation

## Dependencies

### Upstream Dependencies
- ✅ WASM-TASK-002: WASM Runtime Layer (Block 1) - **REQUIRED** for memory sandbox
- ✅ WASM-TASK-004: Actor System Integration (Block 3) - **REQUIRED** for ComponentActor isolation
- ✅ ADR-WASM-005: Capability-Based Security Model - **COMPLETE**
- ✅ ADR-WASM-006: Component Isolation and Sandboxing - **COMPLETE**

### Downstream Dependencies (Blocks This Task)
- WASM-TASK-006: Inter-Component Communication (Block 5) - needs message security
- WASM-TASK-007: Persistent Storage (Block 6) - needs storage capability enforcement
- WASM-TASK-008: Component Lifecycle (Block 7) - needs trust-level workflows
- WASM-TASK-009: AirsSys-OSL Bridge (Block 8) - needs capability checks for host functions

### External Dependencies
- airssys-osl RBAC/ACL system (external, currently 85% complete)
- glob pattern matching library (consider using `globset` crate)

## Risks and Mitigations

### Risk 1: Pattern Matching Performance
**Impact:** High - Slow checks could bottleneck all operations  
**Probability:** Medium - Pattern matching can be expensive  
**Mitigation:**
- Use compiled glob patterns (one-time compilation cost)
- Cache capability check results where possible
- Benchmark extensively and optimize hot paths
- Target <5μs includes pattern compilation

### Risk 2: Bypass Vulnerabilities
**Impact:** Critical - Security bypass defeats entire system  
**Probability:** Medium - Complex systems have edge cases  
**Mitigation:**
- Extensive security testing with bypass attempts
- Code review by security experts
- Penetration testing with malicious components
- Bug bounty program consideration for Phase 2

### Risk 3: Developer Experience Friction
**Impact:** High - Poor UX could hinder adoption  
**Probability:** Medium - Security often conflicts with ease of use  
**Mitigation:**
- Trust-level system enables instant install for trusted sources
- DevMode for rapid development iteration
- Clear error messages for capability denials
- Comprehensive documentation with examples

### Risk 4: airssys-osl Integration Complexity
**Impact:** Medium - Integration issues could delay Block 8  
**Probability:** Low - airssys-osl has stable RBAC/ACL API  
**Mitigation:**
- Design capability layer to be independent
- Abstract airssys-osl integration behind trait
- Test capability layer without airssys-osl first
- Clear integration interface contract

### Risk 5: Quota System Overhead
**Impact:** Medium - Quota tracking could impact performance  
**Probability:** Low - Simple counters are fast  
**Mitigation:**
- Use atomic counters for lock-free updates
- Batch quota checks where possible
- Optimize quota storage (in-memory with periodic persistence)
- Benchmark quota overhead separately

## Progress Tracking

**Overall Status:** not-started - 0%

### Phase Breakdown
| Phase | Description | Status | Estimated Duration | Notes |
|-------|-------------|--------|-------------------|-------|
| 1 | Capability System Foundation | not-started | Week 1-2 | Pattern matching core |
| 2 | Trust-Level System | not-started | Week 2-3 | Developer experience |
| 3 | Host Function Enforcement | not-started | Week 3-4 | Security enforcement |
| 4 | Actor Isolation Integration | not-started | Week 4 | Defense-in-depth |
| 5 | Supervision and Health Monitoring | not-started | Week 5 | Security resilience |
| 6 | airssys-osl Integration and Testing | not-started | Week 5-6 | Layered security |

### Subtasks
| ID | Description | Status | Updated | Notes |
|----|-------------|--------|---------|-------|
| 1.1 | Capability Data Structures | not-started | - | Foundation |
| 1.2 | Pattern Matching Engine | not-started | - | Performance critical |
| 1.3 | Component.toml Capability Parser | not-started | - | Declaration system |
| 2.1 | Trust Level Implementation | not-started | - | UX foundation |
| 2.2 | Approval Workflow Engine | not-started | - | Trust automation |
| 2.3 | Trust Configuration System | not-started | - | Configuration |
| 3.1 | Capability Check API | not-started | - | Enforcement API |
| 3.2 | Host Function Entry Points | not-started | - | Integration points |
| 3.3 | Audit Logging Integration | not-started | - | Security visibility |
| 4.1 | ComponentActor Security Boundaries | not-started | - | Isolation layer |
| 4.2 | Message Passing Security | not-started | - | Communication security |
| 4.3 | Resource Quota System | not-started | - | Abuse prevention |
| 5.1 | SupervisorNode Security Integration | not-started | - | Resilience |
| 5.2 | Health Monitoring with Security | not-started | - | Monitoring |
| 5.3 | Graceful Degradation | not-started | - | Incident response |
| 6.1 | Layered Security Integration | not-started | - | Defense-in-depth |
| 6.2 | Comprehensive Security Testing | not-started | - | Validation |
| 6.3 | Security Documentation and Examples | not-started | - | Developer guide |

## Progress Log

*No progress yet - task just created*

## Related Documentation

### ADRs
- **ADR-WASM-005: Capability-Based Security Model** - Primary reference for all security decisions
- **ADR-WASM-006: Component Isolation and Sandboxing** - Multi-layer defense architecture
- **ADR-WASM-002: WASM Runtime Engine Selection** - Memory sandbox integration
- **ADR-WASM-009: Component Communication Model** - Message security requirements

### Knowledge Documentation
- **KNOWLEDGE-WASM-001: Component Framework Architecture** - Security context in architecture
- **KNOWLEDGE-WASM-004: WIT Management Architecture** - Capability declarations in WIT
- **KNOWLEDGE-WASM-009: Component Installation Architecture** - Ed25519 ownership and trust

### External References
- [Capability-Based Security (Wikipedia)](https://en.wikipedia.org/wiki/Capability-based_security)
- [WASM Sandboxing Model](https://webassembly.org/docs/security/)
- [Pattern Matching (globset crate)](https://docs.rs/globset/)
- [Security Audit Logging Best Practices](https://owasp.org/www-project-application-security-verification-standard/)

## Notes

**Multi-Layer Defense:**
This block implements Layer 1 (Capabilities) of the 4-layer security model:
- Layer 1: Capability-based security (this block)
- Layer 2: WASM linear memory sandbox (Block 1)
- Layer 3: Actor isolation (Block 3)
- Layer 4: Supervision trees (Block 3)

**Performance Critical:**
Capability checks are in the hot path of EVERY host function call. <5μs overhead is non-negotiable.

**Trust-Level is Key to UX:**
Without trust levels, every component would require manual approval. Trust system enables:
- Internal components: instant install
- Known publishers: instant install
- Unknown sources: review required
- Development: rapid iteration

**Pattern Matching Security:**
Glob patterns are powerful but must be implemented correctly:
- `**` must not escape filesystem boundaries
- Symlink handling must be secure
- Path canonicalization required

**Deny-by-Default:**
Components start with ZERO permissions. All capabilities must be explicitly declared and granted.

**Developer Documentation Critical:**
Security is only effective if developers understand it. Comprehensive examples and troubleshooting essential.

**Audit Logging Non-Optional:**
All capability checks (granted AND denied) must be logged for security analysis.

**Integration with Block 3:**
This block heavily depends on ComponentActor isolation from Block 3. Actor boundaries are security boundaries.

**Phase 2 Enhancements:**
- Custom capability types (domain-specific permissions)
- Advanced quota systems (network bandwidth, CPU time)
- Machine learning-based anomaly detection
- Integration with external security services
