# DEBT-WASM-003: Component Model v0.1 Type Import Limitation

**Priority:** MEDIUM (Architectural, not functional)  
**Severity:** MEDIUM (Violates DRY principle but doesn't block functionality)  
**Status:** ACTIVE - Mitigated, awaiting Component Model v0.2  
**Introduced:** 2025-10-26 (WASM-TASK-003 Phase 2 Task 2.1)  
**Discovery Date:** 2025-10-26  
**Discovered During:** Cross-package WIT import syntax testing  

## Problem Statement

**UPDATE (2025-10-26):** Component Model v0.1 *does* support cross-interface type reuse via `use` statements within the same package. See "Discovery & Resolution" section below.

The original investigation revealed limitations with:

1. **Qualified type references** - Cannot use qualified names like `types.component-id` in record definitions (remains true)
2. **Cross-package imports** - Importing types from other packages was attempted but failed (remains true)
3. **Selective imports** - Now confirmed working with `use interface.{type1, type2}` syntax within same package (RESOLVED)

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

## Impact & Resolution

### Implemented Solution (2025-10-26)
- **Implementation**: Multi-file `airssys:core@1.0.0` package with 4 interfaces in separate files
- **Type Reuse**: Using `use types.{type1, type2}` statements for cross-interface type imports
- **DRY Compliance**: ✅ Eliminated type duplication (removed 92 lines)
- **Clean Architecture**: ✅ Proper dependency declarations via `use` statements
- **Maintenance**: ✅ Single source of truth for each type definition

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

## Discovery & Resolution (2025-10-26)

### Root Cause of Initial Confusion
The documentation in the BytecodeAlliance Component Model guide clearly shows that `use` statements *do* work within packages:

```wit
interface types {
    type dimension = u32;
    record point {
        x: dimension,
        y: dimension,
    }
}

interface canvas {
    use types.{dimension, point};  // ← This works!
    draw-line: func(canvas: canvas-id, from: point, to: point);
}
```

The original investigation didn't properly test this syntax, instead focusing on cross-package imports which do have limitations.

### Implementation (Commit d193ded)
1. **Added `use` statements** to all dependent interfaces:
   - `capabilities.wit`: `use types.{component-id};`
   - `component-lifecycle.wit`: `use types.{component-id, request-id, component-error, execution-error, health-status};`
   - `host-services.wit`: `use types.{component-id, request-id, component-error, log-level, timestamp};`

2. **Validation Results**:
   - ✅ All interfaces validate individually within package context
   - ✅ Package validates as whole with `wasm-tools component wit wit/core/`
   - ✅ Exit code: 0 (success)
   - ✅ 92 lines of duplication removed

3. **Architecture**:
   - Multi-file structure: `types.wit`, `capabilities.wit`, `component-lifecycle.wit`, `host-services.wit`
   - Single package: `airssys:core@1.0.0`
   - Clean imports between interfaces
   - Proper type dependency declarations

## Previous Workaround (Now Superseded)

This section documents the investigation that led to the discovery. The monolithic approach was initially considered but is no longer used.

### What We Initially Considered
1. **Single monolithic file** - Merge all interfaces into `core.wit`
2. **Type duplication** - Re-define types in each interface for isolation
3. **Clear documentation** - Extensive comments explaining the limitation

### Performance Impact
- **Build time**: No impact (parsed once per package)
- **Binary size**: Negligible (WIT is not compiled to binary)
- **Runtime**: No impact (types resolved at component loading)

## Remaining Limitations

While cross-interface type reuse *within packages* is now resolved, the following v0.1 limitations remain:

1. **Qualified type references in definitions** - Cannot use `types.component-id` directly in record field types (must use short name)
2. **Cross-package type imports** - Cannot import types from external packages (only interfaces)
3. **Package-level exports** - Types cannot be re-exported at package scope

### Implications for Extension Packages

When implementing filesystem, network, and process extension packages (Task 2.2):
- ✅ Can import from core package interfaces (via `import airssys:core/types`)
- ⚠️ Each package must define its own types (cannot reuse core types directly)
- ✅ Can use internal `use` statements within extension packages (same pattern as core)

## Recommendations

### Current Status (✅ Implemented)
1. ✅ **Leverage `use` statements** - Eliminated type duplication in core package
2. ✅ **Multi-file structure** - Clean separation with shared types interface
3. ✅ **Proper documentation** - Comments explain import dependencies
4. ✅ **Ready for extension packages** - Pattern proven with core package

### For Task 2.2 (Extension Packages)
1. **Filesystem package**: Create separate `airssys:filesystem` with its own types
2. **Network package**: Create separate `airssys:network` with its own types
3. **Process package**: Create separate `airssys:process` with its own types
4. **Decision needed**: Implement as separate packages or consolidate like core?

### Long-term (Component Model v0.2+)
1. **Monitor v0.2 progress** - Track cross-package type imports support
2. **Plan migration** - When v0.2 available, could consolidate types into shared package
3. **Evaluate ADR-WASM-015** - Revisit original 7-package design with v0.2 capabilities

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
- Multi-file core package validates with wasm-tools 1.240.0
- All 4 interfaces in separate files parse successfully
- `use` statements properly resolve types across interfaces
- Cross-interface type reuse confirmed working (commit d193ded)
- 92 lines of type duplication removed
- Ready for Task 2.2 extension packages

## Technical Notes

**Architecture Decisions:**

1. **Single Global Interface**
   - ❌ Would lose logical separation
   - ❌ Violates software engineering principles
   - ✅ Would eliminate all duplication

2. **Separate Single-Interface Files with `use` Statements** (✅ Chosen)
   - ✅ Maintains logical separation across files
   - ✅ Leverages `use` statements for proper type imports
   - ✅ Clean dependency declarations
   - ✅ Scalable and maintainable
   - ✅ Eliminates all duplication

3. **Monolithic Single File** (Previously Considered)
   - ✅ Works within v0.1 constraints
   - ⚠️ Requires duplication for interface isolation
   - ❌ Less maintainable than multi-file approach

The multi-file approach with `use` statements is superior because it:
- Maintains logical interface separation
- Uses proper import declarations
- Eliminates type duplication
- Scales to larger packages
- Follows WIT best practices

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

---

**Document Version:** 2.0.0  
**Last Updated:** 2025-10-26 (Updated with `use` statement discovery and implementation)  
**Status:** RESOLVED for within-package imports - Implemented via `use` statements; cross-package limitations remain; awaiting v0.2 for full resolution
