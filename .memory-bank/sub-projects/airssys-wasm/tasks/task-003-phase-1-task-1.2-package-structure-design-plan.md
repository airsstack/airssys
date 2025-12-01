# WASM-TASK-003 Phase 1 Task 1.2: Package Structure Design - Implementation Plan

**Status:** 📋 READY FOR EXECUTION  
**Created:** 2025-10-25  
**Task Duration:** 6 hours (Day 2 of 9-day Phase 1 rework)  
**Prerequisites:** Task 1.1 (WIT Ecosystem Research) COMPLETE  
**Priority:** CRITICAL - Blocks Phase 2 implementation

---

## Executive Summary

This is the detailed implementation plan for **Task 1.2: Package Structure Design Based on Evidence**, the second task in WASM-TASK-003 Phase 1. This design phase translates the research findings from Task 1.1 into a complete, validated 7-package structure blueprint ready for Phase 2 implementation.

### Task Objective

Design the complete 7-package WIT structure per ADR-WASM-015, with validated `deps.toml` configuration, cross-package dependency graph, and import patterns. Deliver a blueprint ready for Phase 2 implementation.

### Key Deliverables

1. **Package Structure Design** - Complete directory and file organization
2. **deps.toml Configuration** - Template for all 7 packages
3. **Dependency Graph** - Validated acyclic dependency graph
4. **Import Patterns** - Cross-package import strategy
5. **Type Sharing Strategy** - Type reuse guidelines
6. **Phase 2 Handoff** - Implementation guide

---

## Context from Task 1.1

### Evidence Available (✅ COMPLETE)

**Research Documentation:**
- `docs/research/tooling_versions.md` - wasm-tools 1.240.0 validated
- `docs/research/wasm_tools_commands_reference.md` - Complete command reference
- `docs/research/wit_specification_constraints.md` - WIT specification rules documented
- `docs/research/wasm_tools_validation_guide.md` - Validation workflow proven

**Test Packages:**
- `tests/wit_validation/minimal_package/` - Working test packages

**Key Research Findings:**
1. Package naming format validated: `airssys:core-types@1.0.0` ✅
2. Validation workflow proven: `wasm-tools component wit <file>` ✅
3. WIT specification constraints documented with evidence ✅
4. Naming patterns confirmed (lowercase, hyphens, semantic versioning) ✅
5. WASI Preview 2 examples as reference patterns ✅

---

## ADR-WASM-015: 7-Package Structure

### Core Packages (4)

1. **`airssys:core-types@1.0.0`** - Common types and errors
2. **`airssys:core-component@1.0.0`** - Component lifecycle interfaces
3. **`airssys:core-capabilities@1.0.0`** - Capability and permission types
4. **`airssys:core-host@1.0.0`** - Host service interfaces

### Extension Packages (3)

5. **`airssys:ext-filesystem@1.0.0`** - Filesystem operation interfaces
6. **`airssys:ext-network@1.0.0`** - Network operation interfaces
7. **`airssys:ext-process@1.0.0`** - Process operation interfaces

---

## Task 1.2 Breakdown: 6 Hours Total

### Hour 1-2: Directory Structure and Package Organization Design (2 hours)

#### Hour 1 (60 min): Core Package Structure Design

**Activity 1.1 (20 min): Design `wit/core/` directory structure**

**Actions:**
```bash
# Design directory structure for 4 core packages
# Each package will be a subdirectory with WIT files

wit/
├── core/
│   ├── types/
│   │   └── types.wit          # airssys:core-types@1.0.0
│   ├── component/
│   │   └── component.wit      # airssys:core-component@1.0.0
│   ├── capabilities/
│   │   └── capabilities.wit   # airssys:core-capabilities@1.0.0
│   └── host/
│       └── host.wit           # airssys:core-host@1.0.0
```

**Design Decisions:**
- Each package gets its own subdirectory
- Main WIT file named after package concept
- Clean separation of concerns per ADR-WASM-015

**Activity 1.2 (20 min): Design `wit/ext/` directory structure**

**Actions:**
```bash
# Design directory structure for 3 extension packages

wit/
├── ext/
│   ├── filesystem/
│   │   └── filesystem.wit     # airssys:ext-filesystem@1.0.0
│   ├── network/
│   │   └── network.wit        # airssys:ext-network@1.0.0
│   └── process/
│       └── process.wit        # airssys:ext-process@1.0.0
```

**Design Decisions:**
- Extension packages mirror core structure
- Clear separation: core (required) vs ext (optional)
- Each extension is independent and focused

**Activity 1.3 (20 min): Create directory structure blueprint document**

**Actions:**
1. Document complete `wit/` directory tree
2. Define package-to-directory mapping
3. Create README.md explaining organization
4. Document file naming conventions

**Deliverable:** `wit/structure_plan.md`

**Content:**
- Complete directory tree visualization
- Package-to-directory mapping table
- File naming conventions
- Directory organization rationale
- Cross-reference to ADR-WASM-015

---

#### Hour 2 (60 min): Package Content Design

**Activity 2.1 (30 min): Design core package interfaces**

**For Each Core Package, Define:**

**`core/types/types.wit`:**
- Common error types (component-error, execution-error, etc.)
- Shared data structures (component-id, request-id, etc.)
- Result types and status enums
- Documentation comments

**`core/component/component.wit`:**
- Component lifecycle interfaces (init, execute, shutdown)
- Component metadata structures
- Health check interfaces
- Lifecycle callback definitions

**`core/capabilities/capabilities.wit`:**
- Permission types (filesystem-permission, network-permission, etc.)
- Capability request structures
- Permission action enums
- Security-related types

**`core/host/host.wit`:**
- Host service interfaces (logging, messaging, time)
- Host function signatures
- Service discovery interfaces
- Introspection capabilities

**Activity 2.2 (30 min): Design extension package interfaces**

**For Each Extension Package, Define:**

**`ext/filesystem/filesystem.wit`:**
- File operation interfaces (read, write, delete, list)
- Path and directory types
- File metadata structures
- Filesystem-specific errors

**`ext/network/network.wit`:**
- Network operation interfaces (TCP, UDP, HTTP)
- Socket and connection types
- Network address structures
- Network-specific errors

**`ext/process/process.wit`:**
- Process operation interfaces (spawn, kill, signal)
- Process handle types
- Environment and argument structures
- Process-specific errors

**Deliverable:** `wit/package_content_design.md`

**Content:**
- Interface outline for each package
- Type definitions per package
- Function signatures per package
- Documentation standards
- Cross-package type usage

---

### Hour 3-4: Dependency Graph and deps.toml Design (2 hours)

#### Hour 3 (60 min): Dependency Graph Design

**Activity 3.1 (20 min): Analyze cross-package dependencies**

**Dependency Analysis:**

**Core Packages:**
- `core-types`: No dependencies (foundation)
- `core-component`: Depends on `core-types` (uses error types)
- `core-capabilities`: Depends on `core-types` (uses common types)
- `core-host`: Depends on `core-types`, `core-capabilities` (uses types and permissions)

**Extension Packages:**
- `ext-filesystem`: Depends on `core-types`, `core-capabilities` (uses errors and permissions)
- `ext-network`: Depends on `core-types`, `core-capabilities` (uses errors and permissions)
- `ext-process`: Depends on `core-types`, `core-capabilities` (uses errors and permissions)

**Activity 3.2 (20 min): Create dependency graph**

**ASCII Dependency Graph:**
```
Level 0 (Foundation):
  core-types
      ↓
Level 1 (Base Abstractions):
  ├── core-component
  └── core-capabilities
      ↓
Level 2 (Host Services):
  core-host
      ↓
Level 3 (Extensions):
  ├── ext-filesystem
  ├── ext-network
  └── ext-process
```

**Topological Ordering:**
1. `core-types` (no dependencies)
2. `core-component`, `core-capabilities` (parallel, both depend on core-types)
3. `core-host` (depends on core-types, core-capabilities)
4. `ext-*` packages (parallel, all depend on core-types, core-capabilities)

**Validation:**
- ✅ No circular dependencies
- ✅ Clear dependency layers
- ✅ Foundation → Abstractions → Services → Extensions

**Activity 3.3 (20 min): Document dependency rationale**

**For Each Dependency, Document:**
- Why the dependency exists
- What types/interfaces are imported
- Alternative designs considered
- Future evolution considerations

**Deliverable:** `wit/dependency_graph.md`

**Content:**
- ASCII dependency diagram
- Topological ordering
- Dependency rationale table
- Circular dependency validation
- Import requirements per package

---

#### Hour 4 (60 min): deps.toml Configuration Design

**Activity 4.1 (30 min): Research deps.toml format**

**Actions:**
1. Review WASI Preview 2 deps.toml examples
2. Study wasm-tools deps.toml documentation
3. Understand path-based dependency syntax
4. Document configuration patterns

**Research Questions:**
- [ ] What is exact deps.toml syntax?
- [ ] How are relative paths specified?
- [ ] How are versions referenced?
- [ ] Can dependencies be optional?

**Research Sources:**
- WASI Preview 2 WIT packages
- wasm-tools documentation
- Component Model specification

**Activity 4.2 (30 min): Design deps.toml structure**

**For Each Package, Design deps.toml:**

**Example: `wit/core/component/deps.toml`**
```toml
# Dependencies for airssys:core-component@1.0.0

[dependencies]
"airssys:core-types" = { path = "../types" }
```

**Example: `wit/core/host/deps.toml`**
```toml
# Dependencies for airssys:core-host@1.0.0

[dependencies]
"airssys:core-types" = { path = "../types" }
"airssys:core-capabilities" = { path = "../capabilities" }
```

**Example: `wit/ext/filesystem/deps.toml`**
```toml
# Dependencies for airssys:ext-filesystem@1.0.0

[dependencies]
"airssys:core-types" = { path = "../../core/types" }
"airssys:core-capabilities" = { path = "../../core/capabilities" }
```

**Template Design Principles:**
- Relative paths from package directory
- Clear dependency declarations
- Comments explaining purpose
- Version consistency

**Deliverable:** `wit/deps.toml.template`

**Content:**
- deps.toml template for each of 7 packages
- Path resolution documentation
- Dependency declaration examples
- Configuration validation notes

**Deliverable:** `docs/wit/dependency_resolution_strategy.md`

**Content:**
- deps.toml format specification
- Path resolution rules
- Version management strategy
- Dependency update procedures
- Validation with wasm-tools

---

### Hour 5: Cross-Package Import Pattern Design (1 hour)

#### Activity 5.1 (30 min): Import Syntax Design

**WIT Import Syntax:**
```wit
// In core/component/component.wit
package airssys:core-component@1.0.0;

// Import types from core-types package
use airssys:core-types@1.0.0.{component-error, execution-error, component-id};

interface component {
    // Use imported types
    metadata: func() -> result<component-metadata, component-error>;
}
```

**Import Pattern Examples:**

**Pattern 1: Core package importing from core-types**
```wit
// core/host/host.wit
use airssys:core-types@1.0.0.{component-id, log-level};
use airssys:core-capabilities@1.0.0.{filesystem-permission};
```

**Pattern 2: Extension package importing from core**
```wit
// ext/filesystem/filesystem.wit
use airssys:core-types@1.0.0.{file-error, request-id};
use airssys:core-capabilities@1.0.0.{filesystem-permission, filesystem-action};
```

**Pattern 3: Multiple imports from same package**
```wit
use airssys:core-types@1.0.0.{
    component-error,
    execution-error,
    component-id,
    request-id
};
```

**Activity 5.2 (30 min): Type Sharing Strategy**

**Type Sharing Principles:**

1. **Foundation Types in core-types**
   - All error types
   - All ID types
   - Common result structures
   - Shared enums

2. **Domain-Specific Types in Domain Packages**
   - Filesystem types in ext-filesystem
   - Network types in ext-network
   - Process types in ext-process

3. **Import, Don't Duplicate**
   - Never redefine types
   - Always import shared types
   - Use consistent naming

4. **Version Consistency**
   - All packages use @1.0.0
   - Coordinated version updates
   - Breaking change strategy

**Deliverable:** `wit/import_patterns.md`

**Content:**
- Import syntax examples for all dependency combinations
- Pattern templates for common imports
- Import organization guidelines
- Version reference requirements

**Deliverable:** `wit/type_sharing_strategy.md`

**Content:**
- Type placement guidelines (which types in which packages)
- Type reuse patterns
- Import vs duplication rules
- Version compatibility strategy
- Type evolution guidelines

---

### Hour 6: Validation and Documentation (1 hour)

#### Activity 6.1 (30 min): Design Validation

**Validation Checklist:**

**Structural Validation:**
- [ ] ✅ All 7 packages defined with correct names
- [ ] ✅ Directory structure matches ADR-WASM-015
- [ ] ✅ File naming follows conventions
- [ ] ✅ Package organization is clear

**Dependency Validation:**
- [ ] ✅ Dependency graph is acyclic (no cycles)
- [ ] ✅ deps.toml paths are correct (relative paths valid)
- [ ] ✅ All dependencies are in topological order
- [ ] ✅ No missing dependencies

**Naming Validation:**
- [ ] ✅ Package names match `airssys:{type}@1.0.0` pattern
- [ ] ✅ All names follow lowercase-with-hyphens convention
- [ ] ✅ Semantic versioning applied consistently
- [ ] ✅ Names validated against Task 1.1 constraints

**Import Pattern Validation:**
- [ ] ✅ Import syntax matches WIT specification
- [ ] ✅ All imported types exist in dependency packages
- [ ] ✅ Version references are consistent
- [ ] ✅ No circular imports

**ADR-WASM-015 Compliance:**
- [ ] ✅ 4 core packages as specified
- [ ] ✅ 3 extension packages as specified
- [ ] ✅ Package purposes match ADR rationale
- [ ] ✅ Organization supports capability system

**KNOWLEDGE-WASM-004 Compliance:**
- [ ] ✅ WIT management patterns followed
- [ ] ✅ Interface organization best practices applied
- [ ] ✅ Documentation standards met

**Activity 6.2 (30 min): Create comprehensive design summary**

**Design Summary Content:**

1. **Executive Summary**
   - 7-package structure overview
   - Key design decisions
   - Validation results

2. **Package Organization**
   - Directory structure
   - File-to-package mapping
   - Naming conventions

3. **Dependency Architecture**
   - Dependency graph
   - deps.toml configuration
   - Import patterns

4. **Type Sharing Strategy**
   - Type placement rules
   - Import guidelines
   - Version management

5. **Phase 2 Handoff**
   - Implementation priorities
   - Validation requirements
   - Open questions

6. **Evidence Traceability**
   - Task 1.1 findings used
   - ADR-WASM-015 compliance
   - WIT specification compliance

**Deliverable:** `docs/wit/task_1.2_design_summary.md`

**Deliverable:** `docs/wit/phase_2_implementation_guide.md`

**Content:**
- Step-by-step implementation instructions for Phase 2
- Package implementation order (topological)
- Validation checklist per package
- Integration testing approach
- Success criteria for Phase 2

**Deliverable:** `wit/validation_checklist.md`

**Content:**
- Pre-implementation validation checks
- Per-package validation requirements
- Cross-package validation steps
- wasm-tools validation commands
- Success criteria checklist

---

## Success Criteria for Task 1.2

### Must Complete

- ✅ 7-package structure clearly defined with file-to-package mapping
- ✅ All naming follows ADR-WASM-015 conventions (`airssys:core-types@1.0.0` format)
- ✅ Dependency graph documented with no circular dependencies
- ✅ deps.toml template created with all 7 package references
- ✅ Cross-package import patterns documented
- ✅ Type sharing strategy defined
- ✅ Clear rationale for every organizational decision
- ✅ Phase 2 implementation guide ready

### Quality Gates

- ✅ All package names validate against WIT naming constraints (Task 1.1 evidence)
- ✅ Dependency graph is acyclic (topological sort possible)
- ✅ deps.toml syntax follows WASI Preview 2 patterns
- ✅ Import patterns match WIT specification
- ✅ Complete traceability: ADR-WASM-015 → design → implementation plan

### Evidence-Based Validation

- ✅ Package names match `namespace:name@version` pattern from Task 1.1
- ✅ File organization follows proven WIT patterns (WASI Preview 2 reference)
- ✅ No assumptions - all design decisions backed by Task 1.1 research

---

## Deliverables Summary

### Documentation (7 files)

1. **`wit/structure_plan.md`** - Complete directory and package organization blueprint
2. **`wit/package_content_design.md`** - Interface design for all 7 packages
3. **`wit/dependency_graph.md`** - Dependency graph with ASCII diagram and rationale
4. **`wit/import_patterns.md`** - Cross-package import pattern guide
5. **`wit/type_sharing_strategy.md`** - Type reuse and sharing guidelines
6. **`docs/wit/task_1.2_design_summary.md`** - Complete design summary
7. **`docs/wit/phase_2_implementation_guide.md`** - Phase 2 handoff document

### Templates (2 files)

1. **`wit/deps.toml.template`** - deps.toml configuration template with examples
2. **`wit/validation_checklist.md`** - Implementation validation checklist

### Research Documentation (1 file)

1. **`docs/wit/dependency_resolution_strategy.md`** - Dependency management guidelines

**Total:** 10 deliverables

---

## Critical Questions Answered

### Q1: Does deps.toml configuration happen in Task 1.2?

**A:** ✅ **YES** - deps.toml **template design** happens in Task 1.2 Hour 4

- **Hour 4 Activity 4.2**: Create deps.toml template with all 7 package references
- **Deliverable**: `wit/deps.toml.template` with configuration for each package
- **Note**: Actual implementation and validation happens in Phase 2 Task 2.3 (Day 6)

**Clear Separation:**
- **Task 1.2 delivers**: Template and configuration strategy
- **Phase 2 delivers**: Working deps.toml files validated with wasm-tools

### Q2: Does cross-package dependency testing happen in Task 1.2?

**A:** ❌ **NO** - Testing happens in **Phase 2 Task 2.3** (Day 6)

- **Task 1.2 delivers**: Design, templates, and import patterns (pure design)
- **Phase 2 Task 2.3 delivers**: Actual implementation and validation with wasm-tools
- **Clear separation**: Design (Task 1.2) vs Implementation (Phase 2)

### Q3: What's the handoff to Phase 2?

**A:** Complete design blueprint with:
- Directory structure plan
- deps.toml template for all 7 packages
- Dependency graph (ASCII diagram with rationale)
- Import pattern examples
- Type sharing strategy
- Implementation validation checklist
- **No code implementation** - pure design deliverables

---

## Risk Mitigation

### Risk 1: deps.toml format assumptions incorrect

**Probability:** Medium  
**Impact:** High (Phase 2 implementation would fail)  
**Mitigation:**
- Study WASI Preview 2 examples thoroughly
- Document format from proven sources
- Test syntax assumptions with minimal example
- Validate during Phase 2 early

### Risk 2: Circular dependencies in design

**Probability:** Low  
**Impact:** High (would block implementation)  
**Mitigation:**
- Create dependency graph early in Hour 3
- Validate topological ordering
- Review against ADR-WASM-015 rationale
- Ask user if uncertain about dependency direction

### Risk 3: Package granularity too fine/coarse

**Probability:** Low  
**Impact:** Medium (would require redesign)  
**Mitigation:**
- Reference ADR-WASM-015 rationale closely
- Validate against KNOWLEDGE-WASM-004 patterns
- Consider future evolution requirements
- Review with user before finalizing

### Risk 4: Import pattern incompatibility

**Probability:** Low  
**Impact:** Medium (Phase 2 validation would fail)  
**Mitigation:**
- Reference Task 1.1 WIT specification constraints
- Use WASI Preview 2 as proven examples
- Document exact syntax with sources
- Validate import syntax format early

---

## Integration with Phase 1

### Dependencies

**Upstream:**
- **Task 1.1** (WIT Ecosystem Research) - ✅ COMPLETE

**Downstream:**
- **Task 1.3** (Build System Integration Research) - Will use this package structure
- **Phase 2** (Implementation Foundation) - Will implement this design

### Handoff Requirements

**To Task 1.3 (Day 3):**
- Package structure overview
- Package naming conventions
- Directory organization

**To Phase 2 (Days 4-6):**
1. ✅ Complete directory structure plan
2. ✅ deps.toml template for all packages
3. ✅ Dependency graph validated (no cycles)
4. ✅ Import pattern examples
5. ✅ Type sharing strategy
6. ✅ Implementation validation checklist
7. ✅ Clear implementation priorities

---

## Evidence-Based Approach

### All Design Decisions Backed By

**From Task 1.1 Research:**
- Package naming: `airssys:core-types@1.0.0` format validated
- WIT specification constraints: Documented in `wit_specification_constraints.md`
- Validation workflow: Proven with test packages
- Naming conventions: Lowercase-with-hyphens confirmed

**From ADR-WASM-015:**
- 7-package structure rationale
- Core vs extension separation
- Package responsibilities
- Capability system integration

**From KNOWLEDGE-WASM-004:**
- WIT management patterns
- Interface organization best practices
- Documentation standards

**From WASI Preview 2:**
- Multi-package organization examples
- deps.toml configuration patterns
- Import syntax examples
- Proven dependency structures

### No Assumptions Policy

- ✅ Package names: Validated against Task 1.1 constraints
- ✅ deps.toml format: Researched from WASI examples
- ✅ Import syntax: Validated against WIT specification
- ✅ Dependency graph: Designed with topological validation
- ✅ Type sharing: Based on proven WASI patterns

---

## Next Steps After Task 1.2

### Immediate Next Task

**Task 1.3: Build System Integration Research** (Day 3, 6 hours)
- Will use Task 1.2 package structure design
- Will research wit-bindgen integration for 7-package structure
- Will prepare for Phase 3 build system implementation

### Phase 2 Preparation

**Phase 2: Implementation Foundation** (Days 4-6)
- Receives complete design blueprint from Task 1.2
- Implements 7 packages based on structure plan
- Validates with wasm-tools at every step
- Uses deps.toml template for configuration

---

**Document Version:** 1.0.0  
**Created:** 2025-10-25  
**Task Owner:** AI Development Agent  
**User Approval Required:** Yes (before execution)  
**Estimated Completion:** 2025-10-25 (Day 2 of Phase 1)

---

**READY FOR USER REVIEW AND APPROVAL** ✅
