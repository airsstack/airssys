# ADR-WASM-015: WIT Package Structure Organization

**Status:** Accepted (with 2025-10-26 constraint resolution update)  
**Date:** 2025-10-25  
**Updated:** 2025-10-26 (Discovery of Component Model v0.1 cross-interface type reuse)  
**Category:** Interface Design & Organization  
**Related:** WASM-TASK-003 Phase 1, ADR-WASM-011 (Module Structure), KNOWLEDGE-WASM-004 (WIT Management), DEBT-WASM-003  

## Context

During WASM-TASK-003 Phase 1 implementation, a discrepancy was identified between the original plan (6 separate WIT files) and the delivered structure (2 packages with consolidated interfaces). This ADR establishes the definitive WIT package structure organization to resolve this discrepancy and provide clear guidance for future WIT interface development.

### Update 2025-10-26: Constraint Resolution

**Discovery**: Component Model v0.1 *does* support cross-interface type reuse within packages using `use` statements, enabling:
- Single-package multi-file organization with clean type imports
- Zero type duplication via `use types.{...}` statements  
- Proper interface isolation with clear dependency declarations
- Clean architecture while supporting existing v0.1 constraints

**Current Implementation (Phase 2 Task 2.1)**:
- Single package: `airssys:core@1.0.0`
- Four focused files: types.wit, capabilities.wit, component-lifecycle.wit, host-services.wit
- Uses: `use types.{component-id, request-id, ...}` for cross-interface type imports
- Result: 92 lines of duplication eliminated, clean architecture established

**7-Package Design Status**:
- ✅ Still valid as design preference (not technical constraint)
- ⚠️  Remains blocked by cross-package type import limitations in v0.1
- 🔄 Deferred to Task 2.2 decision: separate packages vs. consolidated packages for extensions
- 📋 See DEBT-WASM-003 for migration path when v0.2 available

## Decision

### Current Implementation (Phase 2 Task 2.1 - 2025-10-26)

**Adopt single-package multi-file organization with cross-interface type imports:**

```
wit/
├── core/
│   ├── types.wit                → Layer 0: Foundation types (source of truth)
│   ├── capabilities.wit         → Layer 1: Permissions (uses types.{...})
│   ├── component-lifecycle.wit  → Layer 2: Lifecycle (uses types.{...})
│   ├── host-services.wit        → Layer 3: Host services (uses types.{...})
│   └── deps.toml                → Package: airssys:core@1.0.0
└── [future extensions...]
```

**Benefits of this approach:**
- ✅ Single source of truth for types (types.wit)
- ✅ Clean dependencies via `use types.{component-id, ...}` statements
- ✅ Zero type duplication (eliminated 92 lines)
- ✅ Proper interface isolation maintained
- ✅ Clear layered architecture
- ✅ Practical working solution for v0.1 constraints

### Original 7-Package Design (Future Consideration)

The original directory-based package structure design remains valid for future implementation:

```
wit/
├── core/
│   ├── types.wit          → package: airssys:core-types@1.0.0
│   ├── component.wit       → package: airssys:core-component@1.0.0
│   ├── capabilities.wit    → package: airssys:core-capabilities@1.0.0
│   └── host.wit           → package: airssys:core-host@1.0.0
├── ext/
│   ├── filesystem.wit      → package: airssys:ext-filesystem@1.0.0
│   ├── network.wit         → package: airssys:ext-network@1.0.0
│   └── process.wit        → package: airssys:ext-process@1.0.0
└── deps.toml
```

**When to migrate to 7-package design:**
- Component Model v0.2 adds cross-package type imports support
- Cross-package `use` statements work properly in wasm-tools 1.300.0+
- Design preference favors separate packages for independent versioning

**Decision for Task 2.2:**
- Implement extension packages (filesystem, network, process) as separate packages OR
- Follow current pattern with single airssys:extensions@1.0.0 package
- See DEBT-WASM-003 for guidance on package organization strategy

### Naming Convention

**Format:** `airssys:{directory}-{type}@{version}`

**Core Packages (`core/` directory):**
- `airssys:core-types@1.0.0` - Fundamental types, errors, and data structures
- `airssys:core-component@1.0.0` - Component lifecycle contracts and interfaces
- `airssys:core-capabilities@1.0.0` - Permission and capability type definitions
- `airssys:core-host@1.0.0` - Essential host services (logging, messaging, timing)

**Extension Packages (`ext/` directory):**
- `airssys:ext-filesystem@1.0.0` - File system operations and access
- `airssys:ext-network@1.0.0` - Network operations and HTTP client
- `airssys:ext-process@1.0.0` - Process spawning and environment access

## Rationale

### 1. Semantic Clarity
- **Directory Mapping**: Package names directly map to directory structure for immediate understanding
- **Purpose Indication**: `core-` prefix indicates required interfaces, `ext-` prefix indicates optional capabilities
- **Type Identification**: Second part of name clearly identifies package purpose (types, component, host, etc.)

### 2. Consistency and Predictability
- **Uniform Pattern**: All packages follow identical naming convention
- **Easy Discovery**: Developers can predict package names from directory structure
- **Scalability**: New packages follow same pattern without special cases

### 3. Granular Versioning
- **Independent Evolution**: Each package can version independently based on its own API changes
- **Dependency Management**: Fine-grained control over which package versions to use
- **Backward Compatibility**: Easier to maintain compatibility when only specific packages change

### 4. WebAssembly Component Model Alignment
- **Package-Based Organization**: Follows Component Model best practices for package organization
- **Cross-Package Dependencies**: Clean dependency management via `deps.toml`
- **Interface Boundaries**: Clear separation between different functional domains

## Implementation Strategy

### Phase 1: Migration from Current Structure
**Current State:** 2 packages (`airssys:component-core`, `airssys:host-core`) with consolidated interfaces

**Target State:** 7 packages with granular interface separation

**Migration Steps:**
1. **Split Current Interfaces**
   - Extract types from `types.wit` into `core-types` package
   - Extract component lifecycle from `component.wit` into `core-component` package
   - Extract capabilities from `types.wit` into `core-capabilities` package
   - Extract host services from `host.wit` into `core-host` package

2. **Create Extension Packages**
   - Create `ext-filesystem` with file operation interfaces
   - Create `ext-network` with network operation interfaces
   - Create `ext-process` with process operation interfaces

3. **Update Dependencies**
   - Update `deps.toml` with new package references
   - Update cross-package imports in WIT files
   - Validate all dependencies resolve correctly

### Phase 2: Integration with Build System
1. **Update build.rs** to handle multiple package generation
2. **Update Cargo.toml** dependencies for generated bindings
3. **Update documentation** to reflect new structure
4. **Update examples** to use new package imports

## Impact Assessment

### Positive Impacts
- **Clearer Organization**: Functional domains are explicitly separated
- **Better Developer Experience**: Easier to find and understand specific interfaces
- **Improved Maintainability**: Changes to one domain don't affect others
- **Enhanced Scalability**: Easy to add new functional domains

### Migration Costs
- **Breaking Change**: Existing code using current package names will need updates
- **Documentation Updates**: All references to current package structure need revision
- **Build System Changes**: wit-bindgen configuration needs adjustment
- **Testing**: Comprehensive testing required to ensure migration correctness

## Dependencies

### Technical Dependencies
- **WIT Specification**: Package structure must comply with Component Model standards
- **wit-bindgen**: Build system must support multi-package binding generation
- **Cargo Integration**: Generated bindings must integrate cleanly with Rust build system

### Project Dependencies
- **WASM-TASK-003**: This ADR directly impacts Phase 1 implementation completion
- **WASM-TASK-004**: Actor System Integration depends on stable WIT interfaces
- **WASM-TASK-010**: SDK development depends on final WIT package structure

## Future Considerations

### Extensibility
- **New Extensions**: Additional `ext/` packages can be added following same pattern
- **Version Evolution**: Each package can follow semantic versioning independently
- **Cross-Language Support**: Package structure supports multi-language binding generation

### Compatibility
- **Backward Compatibility**: Consider maintaining compatibility layer during transition
- **Documentation Migration**: Plan for gradual migration of examples and tutorials
- **Community Communication**: Clear communication about package structure changes

## Decision Record

**Accepted By:** Project Architecture Team  
**Date:** 2025-10-25  
**Implementation Priority:** High - Required for WASM-TASK-003 Phase 1 completion  
**Review Date:** 2026-01-25 (3-month review)

## Related Documentation

### ADRs
- **ADR-WASM-011**: Module Structure Organization - Overall code organization patterns
- **ADR-WASM-005**: Capability-Based Security Model - Permission system integration
- **ADR-WASM-009**: Component Communication Model - Messaging interface requirements

### Knowledge Documentation
- **KNOWLEDGE-WASM-004**: WIT Management Architecture - Primary reference for interface design
- **KNOWLEDGE-WASM-012**: Module Structure Architecture - Overall module organization

### Task Documentation
- **WASM-TASK-003**: WIT Interface System - Direct implementation task
- **WASM-TASK-003 Phase 1**: Current implementation requiring structure correction

## Implementation Status

**Current Status:** Accepted - Ready for implementation  
**Next Action:** Update WASM-TASK-003 Phase 1 to implement this package structure  
**Estimated Effort:** 2-3 days for migration and testing  
**Target Completion:** 2025-10-28

---

**Document Version:** 1.0.0  
**Last Updated:** 2025-10-25  
**Maintained By:** AirsSys Architecture Team