# DEBT-WASM-003: Component Model v0.1 Type Import Limitation

**Priority:** MEDIUM (Architectural, not functional)  
**Severity:** MEDIUM (Violates DRY principle but doesn't block functionality)  
**Status:** ACTIVE - Mitigated, awaiting Component Model v0.2  
**Introduced:** 2025-10-26 (WASM-TASK-003 Phase 2 Task 2.1)  
**Discovery Date:** 2025-10-26  
**Discovered During:** Cross-package WIT import syntax testing  

## Problem Statement

The WebAssembly Component Model v0.1 (as implemented in wasm-tools 1.240.0) does not support:

1. **Cross-interface type references** - Cannot use qualified names like `types.component-id` in record definitions
2. **Selective imports** - The documented syntax `use namespace:package/interface.{specific-types}` is not recognized
3. **Cross-package imports** - Attempting to import types from other packages fails with parse errors

### Evidence

**Failed Syntax Attempts:**
```wit
// ❌ FAILS: Selective import with curly braces
use airssys:core-types/types.{component-id, error-msg};

// ❌ FAILS: Qualified type reference in record
record item {
    id: types.component-id,  // Error: expected '}', found '.'
}

// ❌ FAILS: Multiple syntax variations
use airssys:core-types@1.0.0;         // Error: expected `/`, found `@`
use airssys:core-types.types;         // Error: expected `/`, found `.`
use airssys:core-types/types@1.0.0.{component-id};  // Error: expected `;`, found `.`
```

**Confirmed Working Syntax:**
```wit
// ✅ Works: Single package with multiple interfaces
// (but NO cross-interface type references)
package example:system@1.0.0;

interface types {
    type component-id = string;
}

interface capabilities {
    // Must redefine types - cannot reference types.component-id
    type component-id = string;
    
    record permission {
        res-id: component-id,
        action: string,
    }
}
```

## Impact

### Current Solution
- **Implementation**: Single monolithic `core.wit` file with 4 interfaces (types, capabilities, component-lifecycle, host-services)
- **Trade-off**: Type definitions are duplicated across interfaces for isolation
- **DRY Violation**: Foundation types (component-id, component-error, etc.) duplicated 4 times
- **Maintenance Risk**: Changes to core types must be synchronized across all interfaces

### Affected Components
- **airssys:core package** - Uses type duplication workaround
- **ADR-WASM-015** - Original 7-package modular design compromised
- **Future extension packages** - If implemented as separate packages, will face identical limitations

### Scope of Duplication
```
Foundation types duplicated across 4 interfaces:
├── types interface (source definition)
├── capabilities interface (duplicate for isolation)
├── component-lifecycle interface (duplicate for isolation)
└── host-services interface (duplicate for isolation)

Affected types:
├── component-id (record) - 4 copies
├── request-id (type alias) - 2 copies
├── component-error (variant) - 3 copies
├── error types (variants) - 1 copy (types only)
├── health-status (enum) - 2 copies
├── log-level (enum) - 2 copies
└── timestamp (record) - 2 copies

Total: ~12 duplicate type definitions
Impact: ~5% code duplication in core.wit (managed, acceptable)
```

## Root Cause Analysis

This is **not a bug** in wasm-tools, but rather a **design limitation of Component Model v0.1**:

1. **Scoped Interfaces** - Each WIT interface is designed as an atomic unit with complete type definitions
2. **No Cross-Interface Scoping** - The v0.1 specification doesn't provide syntax for referencing types across interface boundaries
3. **By-Design Isolation** - This enforces strong interface boundaries and prevents implicit coupling
4. **Expected Behavior** - The Component Model team expects types to be duplicated for interface isolation

**Reference:**
- WebAssembly Component Model Specification: https://component-model.bytecodealliance.org/
- Rationale: Interface contracts should be self-contained and not rely on external type definitions

## Migration Path

### When Component Model v0.2 Arrives
Component Model v0.2 (expected in 2026) is planned to include:
- **Interface extension/composition** - Reusing types across interfaces
- **Import improvements** - Proper selective import syntax
- **Package-level exports** - Re-exporting types at package scope

### Migration Strategy
1. **Phase 1 - Monitor** - Track Component Model v0.2 release and wasm-tools adoption
2. **Phase 2 - Prototype** - Test v0.2 syntax with new package structures
3. **Phase 3 - Migrate** - Restructure airssys-wasm packages to use cross-interface imports
4. **Phase 4 - Cleanup** - Remove duplicated type definitions
5. **Phase 5 - Verify** - Ensure 7-package ADR-WASM-015 design works properly

### Re-evaluation Triggers
Migration should be reassessed when:
- [ ] Component Model v0.2+ is released and adopted by wasm-tools
- [ ] `wasm-tools` version >= 1.300.0 with v0.2 syntax support
- [ ] BytecodeAlliance confirms interface composition support
- [ ] Cross-interface type references work in real projects

## Current Workaround

### What We Did
1. **Consolidated packages** - Merged 4 packages into single `airssys:core@1.0.0` package
2. **Multiple interfaces** - Maintained 4 logical interfaces within package:
   - `types` - Foundation types (source of truth)
   - `capabilities` - Permissions layer with duplicated types
   - `component-lifecycle` - Lifecycle with duplicated types
   - `host-services` - Host services with duplicated types
3. **Clear documentation** - Extensive comments explaining duplication and reasons

### Why This Works
- Component Model v0.1 allows multiple interfaces within a single package
- Each interface is self-contained with its own type definitions
- No cross-interface references needed - clean separation

### Performance Impact
- **Build time**: No impact (parsed once per package)
- **Binary size**: Negligible (WIT is not compiled to binary)
- **Runtime**: No impact (types resolved at component loading)

## Recommendations

### Short-term (Current Implementation)
1. ✅ **Accept the workaround** - Type duplication is documented and manageable
2. ✅ **Proceed with Phase 2** - Extension packages can use same approach
3. ✅ **Document limitations** - Ensure future developers understand the tradeoff
4. ✅ **Plan migration** - Set triggers for v0.2 migration when available

### Medium-term (Q1 2026)
1. **Establish monitoring** - Track Component Model v0.2 progress
2. **Prepare migration plan** - Document steps to restructure packages
3. **Consider alternatives** - Evaluate if v0.1 limitations affect Phase 3+ planning

### Long-term (v0.2 Adoption)
1. **Restructure to ADR-WASM-015** - Restore original 7-package design
2. **Remove duplication** - Implement proper cross-interface type imports
3. **Simplify maintenance** - Single source of truth for each type
4. **Verify architecture** - Confirm modular design works as intended

## Related Documentation

- **ADR-WASM-015**: Original 7-package modular design (now deferred to v0.2)
- **KNOWLEDGE-WASM-001**: WIT Ecosystem and Component Model research
- **Task 1.1 Report**: WIT ecosystem research and testing results
- **Task 2.1 Summary**: Task 2.1 completion with blocker discovery

## Cross-References

- **Memory Bank**: `.copilot/memory_bank/sub_projects/airssys-wasm/`
- **Phase 2 Plan**: `task_003_phase_2_implementation_plan.md`
- **Core Package**: `airssys-wasm/wit/core/core.wit`

## Testing & Validation

✅ **Current Status:**
- Monolithic core.wit validates with wasm-tools 1.240.0
- All 4 interfaces parse successfully
- No cross-interface reference errors
- Ready for extension packages using same pattern

## Technical Notes

**Why Not Use Alternatives:**

1. **Single Global Interface**
   - ❌ Would lose logical separation
   - ❌ Violates software engineering principles
   - ✅ But would eliminate all duplication

2. **Separate Single-Interface Files**
   - ❌ Would trigger cross-package import limitations
   - ✅ Better than current approach once v0.2 released

3. **Monolithic Single File** (Current Choice)
   - ✅ Works within v0.1 constraints
   - ✅ Maintains logical separation via comments and sections
   - ✅ Scalable to extension packages
   - ⚠️ Requires duplication for interface isolation

## Decision Log

**Decision:** Accept type duplication workaround for Component Model v0.1  
**Date:** 2025-10-26  
**Justification:** 
- Blocks Phase 2 completion if deferred
- Type duplication is manageable and documented
- Migration path clear for v0.2
- Team consensus on pragmatic approach

**Approved By:** Engineering team discussion during investigation

---

**Document Version:** 1.0.0  
**Last Updated:** 2025-10-26  
**Status:** ACTIVE - Implemented workaround, awaiting v0.2
