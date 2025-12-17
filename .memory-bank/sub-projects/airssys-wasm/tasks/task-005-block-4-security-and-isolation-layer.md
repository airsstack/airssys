# [WASM-TASK-005] - Block 4: Security & Isolation Layer (REVISED)

**Status:** ready-to-start  
**Added:** 2025-10-20  
**Updated:** 2025-12-17 (MAJOR REVISION - airssys-osl Integration)  
**Priority:** 🔒 CRITICAL PATH - Security Layer  
**Layer:** 2 - Core Services  
**Block:** 4 of 11  
**Estimated Effort:** 3-4 weeks (reduced from 5-6 weeks due to airssys-osl reuse)

---

## 🚨 CRITICAL ARCHITECTURAL DECISION (2025-12-17)

**REVISED APPROACH:** Instead of building security infrastructure from scratch, this task now **LEVERAGES airssys-osl** which already provides:

✅ **Complete Security Framework:**
- ACL (Access Control Lists) with glob pattern matching
- RBAC (Role-Based Access Control) with role inheritance  
- SecurityPolicy trait for extensibility
- SecurityMiddleware with priority 100
- Deny-by-default security model
- Comprehensive audit logging (SecurityAuditLogger)
- Pattern matching (glob patterns, wildcards)
- 311+ tests passing, production-ready

**What Changed:**
- ~~Phase 1: Build capability data structures from scratch~~ → **Phase 1: Bridge WASM to airssys-osl**
- ~~Phase 1-2: Build pattern matching engine~~ → **REUSE airssys-osl glob patterns**
- ~~Phase 3: Build audit logging~~ → **REUSE airssys-osl SecurityAuditLogger**
- ~~Phase 6: Integrate airssys-osl~~ → **START with airssys-osl integration**

**Benefits:**
- ✅ Reuse 1000+ lines of battle-tested security code
- ✅ Leverage existing ACL/RBAC patterns and tests
- ✅ Maintain architectural consistency across AirsSys
- ✅ Reduce implementation time by ~40% (3-4 weeks vs 5-6 weeks)
- ✅ Avoid code duplication and maintenance burden

---

## Overview

Implement the WASM component security layer by **integrating airssys-osl security infrastructure** (ACL/RBAC/audit) with WASM-specific extensions (Component.toml parsing, trust-level system, component isolation). This creates a multi-layered defense protecting the host from malicious or buggy components.

## Context

**Current State:**
- ✅ **airssys-osl**: Complete security framework (ACL, RBAC, audit logging, glob patterns)
- ✅ **WASM-TASK-004**: ComponentActor isolation (Block 3 complete, 589 tests passing)
- ✅ **Architecture**: ADR-WASM-005 (Capability-Based Security Model)
- ✅ **Foundation**: WASM memory sandboxing (512KB-4MB isolated linear memory)

**Problem Statement:**
Components require access to host resources (filesystem, network, storage) but:
1. **Threat: Resource Abuse** - Component declares "read config.toml" but reads /etc/passwd
2. **Threat: Network Abuse** - Component declares "call api.example.com" but exfiltrates data  
3. **Threat: Resource Exhaustion** - Component declares "100MB storage" but writes 10GB
4. **Challenge: Developer Experience** - Security shouldn't block legitimate development

**Multi-Layered Defense:**
- **Layer 1**: airssys-osl ACL/RBAC (permission checks at host functions) ← **REUSE**
- **Layer 2**: WASM linear memory sandbox (bounds checking) ← **EXISTING**
- **Layer 3**: ComponentActor isolation (private mailbox, message passing) ← **EXISTING**
- **Layer 4**: Supervision trees (automatic restart, health monitoring) ← **EXISTING**
- **Layer 5**: Trust-level system (trusted/unknown/dev sources) ← **NEW - THIS TASK**

## Objectives

### Primary Objective
Create a **WASM-to-OSL security bridge** that maps WASM component capabilities (declared in Component.toml) to airssys-osl security policies (ACL/RBAC), adds WASM-specific trust-level workflows, and integrates with ComponentActor isolation.

### Secondary Objectives
- Achieve <1-5μs capability check overhead (leverage airssys-osl performance)
- Implement trust-level system for developer-friendly workflows (trusted/unknown/dev)
- Map Component.toml capabilities to ACL/RBAC policies
- Reuse airssys-osl audit logging for all security events
- Establish dev mode for rapid iteration with warnings

## Scope

### In Scope
1. **WASM-OSL Security Bridge** - Map WASM capabilities to ACL/RBAC policies
2. **Component.toml Parser** - Parse capabilities and map to airssys-osl SecurityContext
3. **Trust-Level System** - Trusted instant, unknown review, dev mode (WASM-specific)
4. **ComponentActor Security Context** - Per-component capability isolation
5. **Capability-to-Policy Mapping** - Filesystem, network, storage capability translation
6. **airssys-osl Integration** - Use existing ACL/RBAC/audit infrastructure
7. **Security Testing** - Verify WASM-OSL bridge correctness

### Out of Scope
- ~~Building ACL/RBAC from scratch~~ → **REUSE airssys-osl**
- ~~Building pattern matching engine~~ → **REUSE airssys-osl glob patterns**
- ~~Building audit logging~~ → **REUSE airssys-osl SecurityAuditLogger**
- Actual host function implementations (Block 8)
- Storage backend implementation (Block 6)
- Component installation workflow (Block 7)

---

## Implementation Plan (REVISED)

### Phase 1: WASM-OSL Security Bridge (Week 1)

#### Task 1.1: WASM Capability Types and OSL Mapping
**Objective:** Define WASM capability types that map cleanly to airssys-osl ACL/RBAC.

**Deliverables:**
- `WasmCapability` enum (Filesystem, Network, Storage, Process)
- `WasmCapabilitySet` container (wraps capabilities)
- Mapping functions: `WasmCapability` → `airssys_osl::middleware::security::AclEntry`
- Mapping functions: `WasmCapability` → `airssys_osl::middleware::security::RoleBasedAccessControl`
- Unit tests for capability mapping
- Documentation explaining WASM → OSL translation

**Success Criteria:**
- All WASM capability types map to ACL/RBAC policies
- Filesystem capabilities map to ACL resource patterns (glob)
- Network capabilities map to ACL resource patterns (domains)
- Storage capabilities map to ACL resource patterns (namespaces)
- Process capabilities map to RBAC permissions
- Zero capability-to-policy translation errors
- Clear documentation with examples

**Example Mapping:**
```rust
// WASM Capability
WasmCapability::Filesystem {
    paths: vec!["/app/data/*"],
    permissions: vec![Read, Write],
}

// Maps to airssys-osl ACL
AclEntry::new(
    component_id,          // identity = component ID
    "/app/data/*",         // resource_pattern (glob)
    vec!["read", "write"], // permissions
    AclPolicy::Allow,
)
```

**Estimated Effort:** 1-2 days

---

#### Task 1.2: Component.toml Capability Parser
**Objective:** Parse Component.toml `[capabilities]` section and build `WasmCapabilitySet`.

**Deliverables:**
- Component.toml TOML parser for `[capabilities]` section
- Capability declaration validation (correct syntax, valid patterns)
- Required vs optional capabilities distinction
- Parser error handling with clear error messages
- Parser tests (valid declarations, invalid declarations, edge cases)
- Documentation for Component.toml capability syntax

**Example Component.toml:**
```toml
[capabilities]
# Filesystem capabilities
filesystem.read = ["/app/config/*", "/app/data/*.json"]
filesystem.write = ["/app/data/*"]

# Network capabilities  
network.connect = ["api.example.com:443", "*.cdn.example.com:80"]

# Storage capabilities
storage.namespace = ["component:<id>:*"]

# Process capabilities (optional)
process.spawn = false  # Denied by default
```

**Success Criteria:**
- Component.toml capabilities parsed correctly
- Invalid patterns rejected with clear errors
- Required capabilities distinguished from optional
- Malformed declarations produce helpful error messages
- Comprehensive test coverage (20+ test cases)
- Clear documentation with examples

**Estimated Effort:** 2-3 days

---

#### Task 1.3: SecurityContext Bridge
**Objective:** Bridge WASM component context to airssys-osl SecurityContext.

**Deliverables:**
- `WasmSecurityContext` struct (component ID, capability set, trust level)
- `WasmSecurityContext` → `airssys_osl::core::context::SecurityContext` converter
- Integration with ComponentActor (attach security context per actor)
- Security context lifecycle (creation, restoration after restart)
- Unit tests for context conversion
- Documentation for security context usage

**Implementation:**
```rust
pub struct WasmSecurityContext {
    pub component_id: ComponentId,
    pub capabilities: WasmCapabilitySet,
    pub trust_level: TrustLevel,
}

impl WasmSecurityContext {
    /// Convert to airssys-osl SecurityContext with ACL attributes
    pub fn to_osl_context(&self, resource: &str, permission: &str) -> SecurityContext {
        let mut attributes = HashMap::new();
        attributes.insert("acl.resource".to_string(), resource.to_string());
        attributes.insert("acl.permission".to_string(), permission.to_string());
        
        SecurityContext {
            principal: self.component_id.to_string(),
            session_id: Uuid::new_v4(),
            established_at: Utc::now(),
            attributes,
        }
    }
}
```

**Success Criteria:**
- WasmSecurityContext converts to airssys-osl SecurityContext
- Component ID maps to SecurityContext principal
- Capabilities map to ACL/RBAC attributes
- Context attached to ComponentActor instances
- Context survives actor restarts
- Clear API and documentation

**Estimated Effort:** 1-2 days

---

### Phase 2: Trust-Level System (Week 2)

#### Task 2.1: Trust Level Implementation
**Objective:** Implement WASM-specific trust-level system (trusted/unknown/dev).

**Deliverables:**
- `TrustLevel` enum (Trusted, Unknown, DevMode)
- `TrustSource` registry (trusted Git repos, signing keys)
- Trust determination logic (check component source against registry)
- Trust level configuration file format
- Trust level documentation

**Trust Levels:**
- **Trusted**: Component from known trusted source → instant approval
- **Unknown**: Component from unknown source → requires manual review
- **DevMode**: Development mode → bypass with logged warnings

**Success Criteria:**
- Three trust levels implemented
- Trust sources configurable (Git repos, signing keys)
- Clear trust determination algorithm
- Trust configuration file parsed correctly
- Documentation for trust management

**Estimated Effort:** 2 days

---

#### Task 2.2: Approval Workflow Engine
**Objective:** Implement approval workflow for unknown components.

**Deliverables:**
- Approval workflow state machine (Pending → Approved/Rejected)
- Trusted source auto-approval (instant install)
- Unknown source review queue (manual approval)
- DevMode capability bypass with warnings
- Approval decision persistence (store approvals)
- Workflow tests (state transitions, edge cases)

**Success Criteria:**
- Trusted sources install instantly (no approval needed)
- Unknown sources enter review queue (manual approval required)
- DevMode bypasses approval with logged warnings
- Approval decisions persist across restarts
- Clear workflow state machine
- Comprehensive documentation

**Estimated Effort:** 2-3 days

---

#### Task 2.3: Trust Configuration System
**Objective:** Create configuration system for trust sources.

**Deliverables:**
- Trust configuration file format (TOML or JSON)
- Trusted Git repository configuration
- Trusted signing key configuration (public keys)
- DevMode enable/disable controls
- Configuration validation (reject invalid configs)
- Configuration documentation with examples

**Example Configuration:**
```toml
[trust]
dev_mode = false  # Disable dev mode in production

[[trust.sources]]
type = "git"
url = "https://github.com/trusted-org/wasm-components"
branch = "main"

[[trust.sources]]
type = "signing_key"
public_key = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJbpYR..."
```

**Success Criteria:**
- Trust configuration file parsed correctly
- Git repos and signing keys configured
- DevMode enable/disable works
- Invalid configurations rejected with clear errors
- Clear documentation with examples

**Estimated Effort:** 1-2 days

---

### Phase 3: Capability Enforcement (Week 2-3)

#### Task 3.1: Capability Check API
**Objective:** Create API for host functions to check capabilities via airssys-osl.

**Deliverables:**
- `check_capability()` function (component ID, resource, permission)
- Build airssys-osl SecurityContext from WASM context
- Call airssys-osl SecurityPolicy::evaluate()
- Return CapabilityCheckResult (Granted/Denied with reason)
- Performance optimization (<5μs per check, leverage airssys-osl)
- API documentation with examples

**Implementation:**
```rust
pub fn check_capability(
    component_id: &ComponentId,
    resource: &str,
    permission: &str,
) -> CapabilityCheckResult {
    // 1. Get WasmSecurityContext for component
    let wasm_ctx = get_component_security_context(component_id)?;
    
    // 2. Convert to airssys-osl SecurityContext
    let osl_ctx = wasm_ctx.to_osl_context(resource, permission);
    
    // 3. Build ACL from component capabilities
    let acl = build_acl_from_capabilities(&wasm_ctx.capabilities);
    
    // 4. Evaluate using airssys-osl SecurityPolicy
    match acl.evaluate(&osl_ctx) {
        PolicyDecision::Allow => CapabilityCheckResult::Granted,
        PolicyDecision::Deny(reason) => CapabilityCheckResult::Denied(reason),
    }
}
```

**Success Criteria:**
- Host functions can check capabilities easily
- Check includes component context
- Clear granted/denied results with reasons
- Performance target met (<5μs)
- Reuses airssys-osl SecurityPolicy evaluation
- Clear API and documentation

**Estimated Effort:** 2 days

---

#### Task 3.2: Host Function Integration Points
**Objective:** Define integration points for host function capability checks.

**Deliverables:**
- Capability check macro for host functions (reduce boilerplate)
- Integration pattern for filesystem host functions
- Integration pattern for network host functions
- Integration pattern for storage host functions
- Check failure error responses (WIT error types)
- Integration tests (mock host functions with checks)

**Example Integration:**
```rust
#[host_function]
fn filesystem_read(path: String) -> Result<Vec<u8>, Error> {
    // Automatic capability check via macro
    check_capability!(Filesystem, path, Read)?;
    
    // Actual implementation (Block 8)
    // ...
}
```

**Success Criteria:**
- All host function categories covered (filesystem, network, storage)
- Checks enforce capability patterns from Component.toml
- Denied access returns clear WIT errors
- No bypass vulnerabilities
- Comprehensive integration tests (30+ test cases)

**Estimated Effort:** 2-3 days

---

#### Task 3.3: Audit Logging Integration
**Objective:** Use airssys-osl SecurityAuditLogger for all WASM security events.

**Deliverables:**
- Integration with airssys-osl SecurityAuditLogger
- Log all capability checks (granted + denied)
- Log component context (ID, capability, resource, trust level)
- Log trust level and approval workflow events
- Structured audit log format (JSON)
- Audit log documentation

**Example Audit Log:**
```json
{
  "timestamp": "2025-12-17T10:30:45.123456Z",
  "event_type": "CapabilityCheckDenied",
  "component_id": "wasm-component-abc123",
  "resource": "/etc/passwd",
  "permission": "read",
  "trust_level": "Unknown",
  "decision": "Denied: Component declared /app/data/* but requested /etc/passwd"
}
```

**Success Criteria:**
- All capability checks logged via airssys-osl
- Logs include full context (component, resource, permission)
- Trust level included in audit logs
- Structured format for analysis (JSON)
- Performance overhead minimal (<100ns per log)
- Clear audit trail for forensics

**Estimated Effort:** 1-2 days

---

### Phase 4: ComponentActor Security Integration (Week 3)

#### Task 4.1: ComponentActor Security Context Attachment
**Objective:** Attach WasmSecurityContext to each ComponentActor instance.

**Deliverables:**
- Add `security_context: WasmSecurityContext` field to ComponentActor
- Initialize security context during component spawn
- Capability set isolation (each component has separate capabilities)
- Security context restoration after supervisor restart
- Isolation verification tests (components cannot access each other's resources)
- Security boundary documentation

**Success Criteria:**
- Each ComponentActor has isolated WasmSecurityContext
- Components cannot access each other's resources
- Security context survives actor restarts
- Isolation verified through testing (20+ test cases)
- Clear security boundary documentation

**Estimated Effort:** 1-2 days

---

#### Task 4.2: Message Passing Security (Already Implemented ✅)
**Status:** ✅ **COMPLETE** (2025-12-17, DEBT-WASM-004 Item #3)

**What's Already Done:**
- ✅ Message authorization checks in `actor_impl.rs` lines 326-416
- ✅ Sender authorization (3-layer enforcement)
- ✅ Payload size limits (configurable)
- ✅ Rate limiting per component
- ✅ 16/16 security tests passing (100% pass rate)
- ✅ 554 ns overhead (9x faster than 5μs target)
- ✅ Security audit logging integrated

**No Additional Work Required:** Message passing security is production-ready.

**Estimated Effort:** 0 days (already complete)

---

#### Task 4.3: Resource Quota System
**Objective:** Implement per-component resource quotas (storage, messages).

**Deliverables:**
- `ResourceQuota` struct (storage bytes, message rate, network bandwidth)
- Quota tracking per ComponentActor
- Quota enforcement in capability checks
- Quota violation error responses
- Quota configuration (default + per-component override)
- Quota monitoring API
- Quota tests (enforcement, violations, monitoring)

**Success Criteria:**
- Storage quotas enforced (100MB default)
- Message rate limits enforced (1000 msg/sec default)
- Quota violations handled gracefully (return error, log event)
- Quotas configurable per component
- Quota monitoring available (current usage)
- Comprehensive tests (15+ test cases)

**Estimated Effort:** 2 days

---

### Phase 5: Testing & Documentation (Week 4)

#### Task 5.1: Security Integration Testing
**Objective:** Comprehensive security testing of WASM-OSL bridge.

**Deliverables:**
- Security test suite (positive and negative tests)
- Bypass attempt tests (malicious component scenarios)
- Trust level workflow tests (trusted/unknown/dev)
- Capability mapping tests (WASM → ACL/RBAC)
- Pattern matching tests (glob patterns, edge cases)
- Performance benchmarks (<5μs capability check)
- Penetration testing framework

**Success Criteria:**
- All capability patterns tested (50+ test cases)
- Bypass attempts detected and blocked (20+ threat scenarios)
- Edge cases covered (invalid patterns, circular dependencies)
- Performance targets met (<5μs per check)
- No security vulnerabilities found
- Comprehensive test suite (100+ tests total)

**Estimated Effort:** 3 days

---

#### Task 5.2: Security Documentation
**Objective:** Complete security documentation for WASM-OSL integration.

**Deliverables:**
- Component.toml capability declaration guide
- Trust level configuration guide
- WASM-OSL security architecture documentation
- Security best practices guide
- Example secure components (3-5 examples)
- Security troubleshooting guide
- Integration guide for host functions

**Success Criteria:**
- Complete security documentation (2000+ lines)
- Clear capability declaration examples (10+ examples)
- Best practices actionable (do's and don'ts)
- Examples demonstrate security patterns
- Troubleshooting guide comprehensive (common issues + solutions)
- Diátaxis framework compliance

**Estimated Effort:** 2-3 days

---

#### Task 5.3: Production Readiness Checklist
**Objective:** Verify production readiness of security layer.

**Deliverables:**
- Production readiness checklist (security, performance, docs)
- Security audit report (vulnerabilities, mitigations, recommendations)
- Performance benchmark report (<5μs capability checks verified)
- Test coverage report (>95% coverage target)
- Final integration verification (all layers working together)
- Sign-off document (stakeholder approval)

**Success Criteria:**
- All checklist items verified ✅
- Zero critical security vulnerabilities
- Performance targets met (<5μs capability checks)
- Test coverage >95% (all critical paths)
- All documentation complete
- Stakeholder sign-off obtained

**Estimated Effort:** 1-2 days

---

## Success Criteria

### Definition of Done
This task is complete when:

1. **WASM-OSL Bridge Complete:**
   - ✅ WasmCapability types map to airssys-osl ACL/RBAC policies
   - ✅ Component.toml parser builds capability sets
   - ✅ WasmSecurityContext converts to airssys-osl SecurityContext
   - ✅ All mappings tested and documented

2. **Trust-Level System Complete:**
   - ✅ TrustLevel enum implemented (Trusted/Unknown/DevMode)
   - ✅ Approval workflow engine functional
   - ✅ Trust configuration system operational
   - ✅ All workflows tested and documented

3. **Capability Enforcement Complete:**
   - ✅ check_capability() API functional
   - ✅ Host function integration points defined
   - ✅ airssys-osl SecurityAuditLogger integrated
   - ✅ All checks tested and documented

4. **ComponentActor Integration Complete:**
   - ✅ Security context attached to each ComponentActor
   - ✅ Message passing security verified (already complete)
   - ✅ Resource quota system operational
   - ✅ Isolation tested and documented

5. **Testing & Documentation Complete:**
   - ✅ 100+ security tests passing (>95% coverage)
   - ✅ Performance benchmarks met (<5μs per check)
   - ✅ Complete documentation (2000+ lines)
   - ✅ Production readiness verified
   - ✅ Zero critical vulnerabilities
   - ✅ Stakeholder sign-off obtained

6. **Quality Gates:**
   - ✅ Zero compiler warnings (cargo clippy)
   - ✅ Zero rustdoc warnings
   - ✅ All tests passing (cargo test)
   - ✅ Benchmarks meet targets (cargo bench)
   - ✅ Code review complete
   - ✅ Security audit complete

---

## Architecture Alignment

### ADR Compliance
- **ADR-WASM-005**: Capability-Based Security Model ✅
- **ADR-WASM-006**: ComponentActor Pattern ✅
- **ADR-WASM-010**: Implementation Strategy (reuse airssys-osl) ✅

### Integration Points
1. **airssys-osl**: ACL/RBAC/audit infrastructure (REUSE)
2. **ComponentActor**: Security context attachment (EXTEND)
3. **Supervision**: Security context restoration (INTEGRATE)
4. **Host Functions**: Capability check entry points (NEW)

### Performance Targets
- Capability check: <5μs (leverage airssys-osl performance)
- Audit logging: <100ns per log (airssys-osl overhead)
- Context conversion: <1μs (WasmSecurityContext → SecurityContext)
- Total overhead: <10μs per host function call

---

## Risk Assessment

### Technical Risks
1. **airssys-osl API Changes** (Low) - OSL is stable, minimal changes expected
2. **Pattern Matching Performance** (Low) - airssys-osl glob patterns already optimized
3. **Trust Source Verification** (Medium) - Git/signing key verification complexity

### Mitigation Strategies
1. Version pin airssys-osl dependency
2. Reuse airssys-osl pattern matching (battle-tested)
3. Start simple (URL matching), add signing keys in Phase 2

---

## Timeline Summary

**Total Duration:** 3-4 weeks (reduced from 5-6 weeks)

- **Week 1**: Phase 1 - WASM-OSL Security Bridge (Tasks 1.1-1.3)
- **Week 2**: Phase 2 - Trust-Level System (Tasks 2.1-2.3) + Phase 3 Start
- **Week 3**: Phase 3 - Capability Enforcement (Tasks 3.1-3.3) + Phase 4 (Tasks 4.1, 4.3)
- **Week 4**: Phase 5 - Testing & Documentation (Tasks 5.1-5.3)

**Note:** Task 4.2 (Message Passing Security) already complete, saving ~1 week.

---

## Dependencies

### Completed Dependencies ✅
- ✅ airssys-osl (100% complete, production-ready)
- ✅ WASM-TASK-004 Block 3 (ComponentActor system, 589 tests passing)
- ✅ DEBT-WASM-004 Item #3 (Message passing security, 16/16 tests)

### Blocking Tasks
- None - all prerequisites complete

### Unblocked Tasks
- ✅ WASM-TASK-006: Block 5 - Component Lifecycle (unblocked by this task)
- ✅ WASM-TASK-007: Block 6 - State Management (unblocked by this task)
- ✅ WASM-TASK-008: Block 7 - Host Functions (unblocked by this task)

---

## References

### Related Documentation
- **ADR-WASM-005**: Capability-Based Security Model
- **ADR-WASM-006**: ComponentActor Pattern
- **airssys-osl README**: Security middleware overview
- **airssys-osl ACL/RBAC**: `src/middleware/security/`
- **DEBT-WASM-004**: Message passing security (complete)

### Code Locations
- **airssys-osl security**: `airssys-osl/src/middleware/security/`
- **ComponentActor**: `airssys-wasm/src/actor/`
- **Security bridge**: `airssys-wasm/src/security/` (to be created)

### Standards References
- **PROJECTS_STANDARD.md**: Project standards (§4.3, §5.1, §6.1)
- **Microsoft Rust Guidelines**: Security patterns (M-ERRORS-CANONICAL)
- **Diátaxis Framework**: Documentation structure

---

## Revision History

| Date | Version | Changes |
|------|---------|---------|
| 2025-10-20 | 1.0 | Initial plan (build from scratch) |
| 2025-12-17 | 2.0 | **MAJOR REVISION**: Leverage airssys-osl, reduce scope/timeline |

