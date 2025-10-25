# WIT Package Dependency Graph - Topological Analysis

**Version:** 1.0.0

---

## Executive Summary

This document provides complete dependency analysis for the 7 packages in ADR-WASM-015. All dependencies are **acyclic** (zero circular dependencies), allowing for clear topological ordering and safe incremental implementation.

**Key Findings:**
- ✅ **Zero circular dependencies** - Clean acyclic graph
- ✅ **4 dependency levels** - Clear implementation order
- ✅ **Foundation pattern** - core-types as universal foundation
- ✅ **Parallel opportunities** - Extensions can build in parallel

---

## Complete Dependency Matrix

### 7x7 Dependency Table

| Package ↓ / Depends on → | core-types | core-component | core-capabilities | core-host | ext-filesystem | ext-network | ext-process |
|--------------------------|------------|----------------|-------------------|-----------|----------------|-------------|-------------|
| **core-types**           | -          | ❌              | ❌                 | ❌         | ❌              | ❌           | ❌           |
| **core-component**       | ✅ YES      | -              | ❌                 | ❌         | ❌              | ❌           | ❌           |
| **core-capabilities**    | ✅ YES      | ❌              | -                 | ❌         | ❌              | ❌           | ❌           |
| **core-host**            | ✅ YES      | ❌              | ✅ YES             | -         | ❌              | ❌           | ❌           |
| **ext-filesystem**       | ✅ YES      | ❌              | ✅ YES             | ❌         | -              | ❌           | ❌           |
| **ext-network**          | ✅ YES      | ❌              | ✅ YES             | ❌         | ❌              | -           | ❌           |
| **ext-process**          | ✅ YES      | ❌              | ✅ YES             | ❌         | ❌              | ❌           | -           |

**Legend:**
- ✅ YES: Package depends on this dependency
- ❌: No dependency
- `-`: Self (cannot depend on self)

### Dependency Count Summary

| Package | Incoming (Depended On By) | Outgoing (Depends On) |
|---------|---------------------------|----------------------|
| **core-types** | 6 packages | 0 (foundation) |
| **core-component** | 0 packages | 1 (types) |
| **core-capabilities** | 4 packages | 1 (types) |
| **core-host** | 0 packages | 2 (types, capabilities) |
| **ext-filesystem** | 0 packages | 2 (types, capabilities) |
| **ext-network** | 0 packages | 2 (types, capabilities) |
| **ext-process** | 0 packages | 2 (types, capabilities) |

**Key Insight:** `core-types` and `core-capabilities` are the most depended-upon packages.

---

## ASCII Dependency Graph

### Visual Representation

```
Level 0 (Foundation - No Dependencies):
┌─────────────────┐
│  core-types     │  ← Foundation package (component-id, errors, etc.)
└────────┬────────┘
         │
         ├──────────────────────────────┬──────────────────────┐
         │                              │                      │
         ↓                              ↓                      ↓
Level 1 (Base Abstractions - Depend on types only):
┌─────────────────┐           ┌─────────────────┐
│ core-component  │           │core-capabilities│  ← Permission types
└─────────────────┘           └────────┬────────┘
                                       │
         ┌─────────────────────────────┴──────────────────────┐
         │                             │                       │
         ↓                             ↓                       ↓
Level 2 (Host Services - Depend on types + capabilities):
┌─────────────────┐
│   core-host     │  ← Host services (logging, messaging, etc.)
└─────────────────┘

Level 3 (Extensions - Depend on types + capabilities):
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│ ext-filesystem  │     │  ext-network    │     │  ext-process    │
└─────────────────┘     └─────────────────┘     └─────────────────┘
         ↑                       ↑                       ↑
         └───────────────────────┴───────────────────────┘
                  (All depend on types + capabilities)
```

### Dependency Flow Direction

```
                         ╔═══════════════╗
                         ║  core-types   ║ (Foundation)
                         ╚═══════╤═══════╝
                                 │
                 ┌───────────────┼───────────────┐
                 │               │               │
                 ↓               ↓               ↓
         ╔═══════════════╗  ╔═══════════════╗
         ║core-component ║  ║core-capabilities║
         ╚═══════════════╝  ╚═══════╤═══════╝
                                    │
         ┌──────────────────────────┴──────────────────────┐
         │                          │                       │
         ↓                          ↓                       ↓
  ╔═══════════════╗          ╔═══════════════╗      ╔═══════════════╗
  ║ core-host     ║          ║ext-filesystem ║      ║ ext-network   ║
  ╚═══════════════╝          ╚═══════════════╝      ╚═══════════════╝
                                                            
                                     ╔═══════════════╗
                                     ║ ext-process   ║
                                     ╚═══════════════╝
```

**Flow Direction:** Top → Bottom (foundation → abstractions → services/extensions)

---

## Topological Ordering

### Complete Build Order

**Definition:** A linear ordering of packages where each package appears after all its dependencies.

**Topological Sort Result:**

```
Level 0:  core-types                    (Build first - no dependencies)
          ↓
Level 1:  core-component                (Parallel with core-capabilities)
          core-capabilities             (Parallel with core-component)
          ↓
Level 2:  core-host                     (Depends on types + capabilities)
          ↓
Level 3:  ext-filesystem                (Parallel - all 3 can build together)
          ext-network                   (Parallel - all 3 can build together)
          ext-process                   (Parallel - all 3 can build together)
```

### Phase 2 Implementation Order

**Day 4: Core Packages (Sequential)**

```bash
# Hour 1: Foundation (90 minutes)
1. Implement core-types           # No dependencies

# Hour 2: Base Abstractions (3 hours total, can parallelize)
2. Implement core-capabilities    # Depends: types (READY after #1)
3. Implement core-component       # Depends: types (READY after #1)
   # Note: #2 and #3 can be done in parallel

# Hour 3-4: Host Services (90 minutes)
4. Implement core-host            # Depends: types, capabilities (READY after #2)
```

**Day 5: Extension Packages (Parallel)**

```bash
# All extensions can build in parallel (same dependencies)
5. Implement ext-filesystem       # Depends: types, capabilities
6. Implement ext-network          # Depends: types, capabilities
7. Implement ext-process          # Depends: types, capabilities
```

**Critical Path:** core-types → core-capabilities → core-host → validation

---

## Detailed Dependency Analysis

### Package 1: core-types

**Dependencies:** None (foundation package)

**Depended On By:**
- core-component
- core-capabilities
- core-host
- ext-filesystem
- ext-network
- ext-process

**Total Incoming Dependencies:** 6 packages

**Rationale:** Foundation package providing common types used everywhere.

**Implementation Priority:** **FIRST** (blocks everything else)

---

### Package 2: core-component

**Dependencies:**
- `airssys:core-types@1.0.0`

**Depended On By:** None (components implement this, packages don't import it)

**Rationale:** Defines component contract. Not imported by other packages because it's the interface components *implement*, not consume.

**Implementation Priority:** Level 1 (after core-types)

**Specific Imports:**
```wit
use airssys:core-types@1.0.0.{
    component-id,
    component-error,
    execution-error,
    health-status,
    request-id
};
```

---

### Package 3: core-capabilities

**Dependencies:**
- `airssys:core-types@1.0.0`

**Depended On By:**
- core-host
- ext-filesystem
- ext-network
- ext-process

**Total Incoming Dependencies:** 4 packages

**Rationale:** Permission types used by host services and all extensions for capability-based security.

**Implementation Priority:** Level 1 (after core-types, before host/extensions)

**Specific Imports:**
```wit
use airssys:core-types@1.0.0.{component-error};
```

---

### Package 4: core-host

**Dependencies:**
- `airssys:core-types@1.0.0`
- `airssys:core-capabilities@1.0.0`

**Depended On By:** None (host services are imported by components at runtime, not in WIT)

**Rationale:** Host services depend on both foundation types and capability types for permission checks.

**Implementation Priority:** Level 2 (after types and capabilities)

**Specific Imports:**
```wit
use airssys:core-types@1.0.0.{
    component-id,
    request-id,
    component-error,
    log-level,
    timestamp
};
use airssys:core-capabilities@1.0.0.{permission-result};  // Future use
```

---

### Package 5: ext-filesystem

**Dependencies:**
- `airssys:core-types@1.0.0`
- `airssys:core-capabilities@1.0.0`

**Depended On By:** None (optional extension)

**Rationale:** File operations need error types and filesystem permissions.

**Implementation Priority:** Level 3 (parallel with other extensions)

**Specific Imports:**
```wit
use airssys:core-types@1.0.0.{file-error, timestamp};
use airssys:core-capabilities@1.0.0.{filesystem-permission, filesystem-action};
```

---

### Package 6: ext-network

**Dependencies:**
- `airssys:core-types@1.0.0`
- `airssys:core-capabilities@1.0.0`

**Depended On By:** None (optional extension)

**Rationale:** Network operations need error types and network permissions.

**Implementation Priority:** Level 3 (parallel with other extensions)

**Specific Imports:**
```wit
use airssys:core-types@1.0.0.{network-error};
use airssys:core-capabilities@1.0.0.{network-permission, network-action};
```

---

### Package 7: ext-process

**Dependencies:**
- `airssys:core-types@1.0.0`
- `airssys:core-capabilities@1.0.0`

**Depended On By:** None (optional extension)

**Rationale:** Process operations need error types and process permissions.

**Implementation Priority:** Level 3 (parallel with other extensions)

**Specific Imports:**
```wit
use airssys:core-types@1.0.0.{process-error};
use airssys:core-capabilities@1.0.0.{process-permission, process-action};
```

---

## Circular Dependency Check

### Analysis Methodology

**Algorithm:** Depth-First Search (DFS) cycle detection

**Starting Points:** All 7 packages

**Result:** ✅ **ZERO circular dependencies detected**

### Verification by Package

```
Check 1: core-types
  Dependencies: [] (none)
  Result: ✅ No cycles possible (leaf node)

Check 2: core-component
  Dependencies: [core-types]
  Path: core-component → core-types → (end)
  Result: ✅ No cycles

Check 3: core-capabilities
  Dependencies: [core-types]
  Path: core-capabilities → core-types → (end)
  Result: ✅ No cycles

Check 4: core-host
  Dependencies: [core-types, core-capabilities]
  Path 1: core-host → core-types → (end)
  Path 2: core-host → core-capabilities → core-types → (end)
  Result: ✅ No cycles

Check 5: ext-filesystem
  Dependencies: [core-types, core-capabilities]
  Path 1: ext-filesystem → core-types → (end)
  Path 2: ext-filesystem → core-capabilities → core-types → (end)
  Result: ✅ No cycles

Check 6: ext-network
  Dependencies: [core-types, core-capabilities]
  Path 1: ext-network → core-types → (end)
  Path 2: ext-network → core-capabilities → core-types → (end)
  Result: ✅ No cycles

Check 7: ext-process
  Dependencies: [core-types, core-capabilities]
  Path 1: ext-process → core-types → (end)
  Path 2: ext-process → core-capabilities → core-types → (end)
  Result: ✅ No cycles
```

**Conclusion:** Graph is **Directed Acyclic Graph (DAG)** ✅

---

## Dependency Rationale

### Why core-types is Foundation

**Decision:** All packages depend on core-types (except core-types itself)

**Rationale:**
1. **Common Error Types**: All packages need consistent error handling
2. **Shared IDs**: component-id, request-id used everywhere
3. **Status Enums**: health-status, log-level are cross-cutting
4. **No Circular Risk**: Types don't depend on anything, safe foundation

**Alternative Considered:** Each package defines own types
**Rejected Because:** Type duplication, inconsistent error handling, cross-package type mapping complexity

**Evidence:** WASI Preview 2 uses same pattern (wasi:io/error is foundation)

---

### Why core-capabilities Depends Only on core-types

**Decision:** Capabilities package has minimal dependencies

**Rationale:**
1. **Permission Types are Data**: No logic, just type definitions
2. **Wide Reuse**: Used by host and all extensions
3. **Stability**: Few dependencies = fewer breaking changes
4. **Independence**: Permission system evolves separately from services

**Alternative Considered:** Capabilities in each extension package
**Rejected Because:** Permission type duplication, inconsistent security model

**Evidence:** ADR-WASM-015 specifies separate capabilities package

---

### Why Extensions All Depend on capabilities

**Decision:** All ext-* packages import core-capabilities

**Rationale:**
1. **Permission Enforcement**: Each extension needs permission types
2. **Security Integration**: Capability-based security is universal
3. **Documentation**: Permission requirements visible in interface
4. **Runtime Checks**: Host validates permissions using these types

**Alternative Considered:** Permission checks external to WIT
**Rejected Because:** Security not visible in interface definition

**Evidence:** KNOWLEDGE-WASM-004 permission-based security model

---

### Why core-component Stands Alone

**Decision:** core-component has no incoming dependencies (other packages don't import it)

**Rationale:**
1. **Implementation Interface**: Components implement this, don't consume it
2. **Runtime Contract**: Between component and host, not between packages
3. **No Shared Types**: Component-specific types (component-config, etc.)
4. **Independence**: Component interface evolves separately

**Alternative Considered:** Host packages import component types
**Rejected Because:** Unnecessary coupling, component types are internal

**Evidence:** WIT component model - worlds export interfaces, don't import component definitions

---

### Why core-host Depends on Both core Packages

**Decision:** core-host imports types and capabilities

**Rationale:**
1. **Type Foundation**: Needs component-id, log-level, errors (from types)
2. **Permission Integration**: Future capability query functions (from capabilities)
3. **Service Layer**: Host services bridge types and capabilities
4. **Complete Context**: Host needs both to provide services

**Alternative Considered:** Host has no capability dependency
**Rejected Because:** Future permission query APIs need capability types

**Evidence:** Host services design in KNOWLEDGE-WASM-004

---

## Optional Dependencies (None Currently)

### Current Design: All Dependencies Required

**Decision:** All dependencies in deps.toml are required (no optional dependencies)

**Rationale:**
1. **Simplicity**: Required dependencies are easier to reason about
2. **WIT Limitation**: WIT spec doesn't support optional import syntax
3. **YAGNI**: No current use case for optional dependencies
4. **Build Clarity**: Deterministic builds, no conditional compilation

### Future: If Optional Dependencies Needed

**Hypothetical Example:**
```toml
# Future syntax (not currently valid WIT)
[dependencies]
types = { path = "../types", required = true }
advanced-types = { path = "../advanced", optional = true }
```

**Current Status:** Not needed, not implemented

**Trigger for Re-evaluation:** If extension packages need selective core imports

---

## Dependency Version Strategy

### Current: All @1.0.0

**Decision:** All packages use version `@1.0.0`

**Rationale:**
1. **Foundation Phase**: Initial stable release
2. **Synchronized Start**: All packages mature together
3. **Simplicity**: No version compatibility matrix yet
4. **Breaking Changes**: Major version bump for all packages together

### Future: Independent Versioning

**When:** After initial stable release (post-Phase 1)

**Strategy:**
- Core packages: Conservative versioning (stability critical)
- Extension packages: Independent evolution (can change freely)
- Compatibility: Maintain backward compatibility in core packages
- Breaking Changes: Coordinated major version bumps

**Example Future State:**
```
core-types@1.2.0           ← Stable, minor updates only
core-capabilities@1.1.0    ← Stable, small additions
ext-filesystem@2.0.0       ← Major update, new features
ext-network@1.5.0          ← Active development
```

---

## Dependency Graph Validation Checklist

### Structural Validation

- [x] ✅ All 7 packages represented in graph
- [x] ✅ All dependencies explicitly declared
- [x] ✅ No missing dependencies
- [x] ✅ No undeclared dependencies

### Acyclic Validation

- [x] ✅ Zero circular dependencies detected
- [x] ✅ Topological sort produces valid ordering
- [x] ✅ All dependency paths terminate
- [x] ✅ DAG structure confirmed

### ADR-WASM-015 Compliance

- [x] ✅ Core packages depend on core packages only
- [x] ✅ Extension packages depend on core packages only
- [x] ✅ No ext→ext dependencies
- [x] ✅ Foundation pattern (types as base)

### Implementation Readiness

- [x] ✅ Clear build order defined
- [x] ✅ Parallelization opportunities identified
- [x] ✅ Critical path documented
- [x] ✅ No blocking circular dependencies

---

## Implementation Impact

### Parallel Build Opportunities

**Level 1 Parallelization:**
- core-component and core-capabilities can build simultaneously (both depend only on core-types)
- **Time Savings:** 50% reduction at Level 1 (2 packages in parallel vs sequential)

**Level 3 Parallelization:**
- All 3 extension packages can build simultaneously
- **Time Savings:** 67% reduction at Level 3 (3 packages in parallel vs sequential)

**Total Parallelization Benefit:**
- Sequential: 7 packages × 1.5 hours = 10.5 hours
- Parallel: 4 levels × 1.5 hours = 6 hours
- **Savings: ~43% faster implementation**

### Critical Path

**Longest Dependency Chain:**
```
core-types → core-capabilities → core-host
```

**Total Depth:** 3 levels

**Critical Path Time:** 4.5 hours (3 packages × 1.5 hours)

**Bottleneck:** core-types (blocks everything), core-capabilities (blocks host + all extensions)

---

## Phase 2 Build Script (Pseudocode)

### Topological Build Order

```bash
#!/bin/bash
# Phase 2 Task 2.1-2.2: Package Implementation

# Level 0: Foundation
echo "Building Level 0: Foundation..."
build_package "core-types"                # No dependencies
validate_package "core-types"

# Level 1: Base Abstractions (parallel)
echo "Building Level 1: Base Abstractions..."
build_package "core-capabilities" &       # Depends: types
build_package "core-component" &          # Depends: types
wait  # Wait for parallel builds
validate_package "core-capabilities"
validate_package "core-component"

# Level 2: Host Services
echo "Building Level 2: Host Services..."
build_package "core-host"                 # Depends: types, capabilities
validate_package "core-host"

# Level 3: Extensions (parallel)
echo "Building Level 3: Extensions..."
build_package "ext-filesystem" &          # Depends: types, capabilities
build_package "ext-network" &             # Depends: types, capabilities
build_package "ext-process" &             # Depends: types, capabilities
wait  # Wait for parallel builds
validate_package "ext-filesystem"
validate_package "ext-network"
validate_package "ext-process"

# Complete validation
echo "Validating complete dependency graph..."
wasm-tools component wit wit/
```

---

## Success Criteria

### Dependency Analysis Complete

- [x] ✅ All 7 packages analyzed
- [x] ✅ All dependencies identified and documented
- [x] ✅ Dependency rationale provided for each
- [x] ✅ No assumptions - all backed by ADR-WASM-015

### Graph Properties Validated

- [x] ✅ Acyclic graph confirmed (zero cycles)
- [x] ✅ Topological ordering computed
- [x] ✅ Implementation priorities clear
- [x] ✅ Parallelization opportunities identified

### Documentation Quality

- [x] ✅ Visual ASCII diagrams provided
- [x] ✅ 7x7 dependency matrix complete
- [x] ✅ Per-package dependency analysis detailed
- [x] ✅ Build order and critical path documented

### ADR-WASM-015 Alignment

- [x] ✅ Core package dependencies match specification
- [x] ✅ Extension package dependencies match specification
- [x] ✅ No violations of dependency rules
- [x] ✅ Foundation pattern implemented correctly

---

## References

### Source Documents
- **ADR-WASM-015**: WIT Package Structure Organization - Dependency rules
- **structure_plan.md**: Package organization and directory structure
- **package_content_design.md**: Interface definitions and type imports

### Related Documents
- **deps.toml.template**: Configuration template (Hour 4 deliverable)
- **import_patterns.md**: Import syntax examples (Hour 5 deliverable)

---

**Document Version:** 1.0.0  
**Created:** 2025-10-25  
**Status:** Complete - Dependency graph validated, zero circular dependencies  
**Next Action:** Research deps.toml format and create configuration template (Hour 4)
