# airssys-wasm Technical Debt Index

**Sub-Project:** airssys-wasm  
**Last Updated:** 2025-12-22  
**Total Debt Items:** 4  
**Active Debt Items:** 4  

## Active Debt Items

### 🔴 DEBT-WASM-004: Message Delivery Runtime Glue Missing (CRITICAL)
- **File:** `debt-wasm-004-message-delivery-runtime-glue-missing.md`
- **Status:** Active
- **Category:** DEBT-ARCH
- **Priority:** CRITICAL (Business), HIGH (Technical)
- **Created:** 2025-12-22
- **Summary:** Message delivery system between host functions and actor mailboxes lacks critical runtime glue code. All components exist but are not wired together.
- **Impact:** Inter-component messaging completely non-functional
- **Affected Files:**
  - `actor/component/component_spawner.rs` - Missing mailbox creation/registration
  - `runtime/async_host.rs:692` - `response_rx` dropped immediately
  - Integration layer - `ActorSystemSubscriber::start()` never called
- **Estimated Fix:** 4-8 hours (~50-100 lines of glue code)
- **Re-evaluation:** Immediate - Next development sprint

### DEBT-WASM-003: Component Model v0.1 Type Import Limitation
- **File:** `debt-wasm-003-component-model-v0.1-type-import-limitation.md`
- **Status:** Resolved (within-package) / Active (cross-package)
- **Category:** DEBT-ARCH
- **Priority:** Medium (Technical), Low (Business)
- **Created:** 2025-10-26
- **Summary:** Component Model v0.1 has limited cross-package type import syntax
- **Impact:** Resolved for within-package imports via `use` statements; cross-package limitations remain
- **Re-evaluation:** When Component Model v0.2 releases

### DEBT-WASM-002: Epoch-Based Preemption Future Enhancement
- **File:** `debt-wasm-002-epoch-preemption-future-enhancement.md`
- **Status:** Active
- **Category:** DEBT-PERF
- **Priority:** Low (Business), Medium (Technical)
- **Created:** 2025-10-24
- **Summary:** Wasmtime epoch-based preemption not implemented; using tokio timeout wrapper instead for simplicity
- **Impact:** Cannot interrupt running WASM code mid-execution; acceptable for trusted components
- **Re-evaluation:** When malicious component handling becomes critical

### DEBT-WASM-001: Deferred WIT Interface Abstractions
- **File:** `debt-wasm-001-deferred-wit-interface-abstractions.md`
- **Status:** Active
- **Category:** DEBT-ARCH
- **Priority:** Low (Technical), Low (Business)
- **Created:** 2025-10-21
- **Summary:** Deferred runtime type abstractions (TypeDescriptor, InterfaceKind, BindingMetadata) following YAGNI analysis
- **Impact:** Positive maintainability (60% less code), zero functional impact
- **Re-evaluation:** Block 10 Phase 2+ (Q2 2026+) for multi-language support

## Priority Matrix

| ID | Category | Business Priority | Technical Priority | Status |
|----|----------|-------------------|-------------------|--------|
| DEBT-WASM-004 | DEBT-ARCH | 🔴 CRITICAL | 🔴 HIGH | Active |
| DEBT-WASM-003 | DEBT-ARCH | Low | Medium | Partial |
| DEBT-WASM-002 | DEBT-PERF | Low | Medium | Active |
| DEBT-WASM-001 | DEBT-ARCH | Low | Low | Active |

## Debt by Category

### DEBT-ARCH (Architectural)
- DEBT-WASM-004: Message Delivery Runtime Glue Missing (**CRITICAL**)
- DEBT-WASM-003: Component Model v0.1 Type Import Limitation
- DEBT-WASM-001: Deferred WIT Interface Abstractions

### DEBT-PERF (Performance)
- DEBT-WASM-002: Epoch-Based Preemption Future Enhancement

### DEBT-QUALITY (Code Quality)
- (None currently)

### DEBT-TEST (Testing)
- (None currently)

### DEBT-DOCS (Documentation)
- (None currently)

## Anticipated Debt Categories (Future)

### Expected DEBT-SECURITY
- Capability enforcement simplifications during prototype phase
- Security policy validation reductions for development speed
- Audit logging gaps during initial implementation
- Sandbox escape prevention measures deferred

### Expected DEBT-PERF
- Component instantiation optimizations postponed
- Memory management optimizations deferred
- Communication overhead optimizations delayed
- Resource pooling implementations simplified

### Debt Prevention Strategy
- Security-first development with comprehensive review process
- Performance benchmarking integrated into development workflow
- Regular architectural review for integration complexity
- Comprehensive testing including security and performance testing
- **Integration testing** between phases to catch wiring issues early

### High-Risk Debt Areas
- **Security**: Any security shortcuts could compromise entire system
- **Performance**: WASM execution performance critical for adoption
- **Integration**: Poor integration could fragment AirsSys ecosystem (⚠️ DEBT-WASM-004)
- **Component Model**: Incomplete implementation could limit functionality

---
**Note:** Debt tracking began 2025-10-21 during Phase 6 (WASM-TASK-000) implementation.  
**Last Updated:** 2025-12-22 - Added DEBT-WASM-004 (Critical priority)
