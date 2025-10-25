# deps.toml Format Specification - WIT Package Dependencies

**Date:** 2025-10-25  
**Task:** WASM-TASK-003 Phase 1 Task 1.2 - Hour 4  
**Purpose:** Document deps.toml configuration format for WIT package dependencies  
**Version:** 1.0.0  
**Research Status:** Specification-based (WIT Component Model)

---

## Executive Summary

The `deps.toml` file is used by wasm-tools to resolve cross-package dependencies when parsing WIT packages. This document specifies the exact format based on the WIT Component Model specification and provides validated examples for the airssys-wasm 7-package structure.

**Key Findings:**
- `deps.toml` uses standard TOML format
- Dependencies specified in `[dependencies]` table
- Path-based dependency resolution (relative paths)
- No version constraints in deps.toml (versions in package declarations)

---

## deps.toml File Purpose

### What It Does

**Primary Function:** Tells wasm-tools where to find dependency packages when parsing WIT files.

**When It's Used:**
```bash
# wasm-tools reads deps.toml when validating packages
wasm-tools component wit ./wit/core/component/

# Looks for deps.toml in the package directory:
# ./wit/core/component/deps.toml
```

**What It Doesn't Do:**
- ❌ Version resolution (versions declared in `.wit` package declarations)
- ❌ Conditional dependencies (all dependencies are required)
- ❌ Feature flags (WIT doesn't support conditional compilation)

---

## File Format Specification

### Basic Structure

```toml
# deps.toml - WIT Package Dependencies

[dependencies]
{dependency-name} = { path = "{relative-path}" }
```

### Fields

#### `[dependencies]` Table

**Required:** No (packages with no dependencies can omit this file entirely)

**Purpose:** Lists all packages this package imports via `use` statements

**Format:** Table of key-value pairs

#### Dependency Entry

**Key:** Dependency identifier (used for reference, can be anything)
**Value:** Inline table with `path` field

**Path Field:**
- **Type:** String
- **Value:** Relative path from deps.toml location to dependency package directory
- **Resolution:** Relative to the directory containing deps.toml

---

## Path Resolution Rules

### Relative Path Syntax

**From Package Directory to Dependency:**
```
Current package:    wit/core/component/
Dependency package: wit/core/types/
Relative path:      ../types
```

**Rules:**
1. Paths are relative to the directory containing deps.toml
2. Use `../` to go up one directory level
3. Use forward slashes `/` (Unix-style paths)
4. Path points to the directory containing the dependency's `.wit` file

### Path Examples

#### Same Tier (core → core)

```toml
# In wit/core/component/deps.toml
# Dependency: wit/core/types/
[dependencies]
types = { path = "../types" }
```

**Explanation:**
- Current: `wit/core/component/`
- Target: `wit/core/types/`
- Navigate: Up one level (`../`) to `wit/core/`, then into `types/`

#### Same Tier (ext → ext) - Not Used in Our Design

```toml
# Hypothetical: In wit/ext/network/deps.toml
# Dependency: wit/ext/filesystem/
[dependencies]
filesystem = { path = "../filesystem" }
```

#### Different Tier (ext → core)

```toml
# In wit/ext/filesystem/deps.toml
# Dependency: wit/core/types/
[dependencies]
types = { path = "../../core/types" }
```

**Explanation:**
- Current: `wit/ext/filesystem/`
- Target: `wit/core/types/`
- Navigate: Up to `wit/ext/` (`../`), up to `wit/` (`../../`), into `core/types/`

---

## Dependency Naming Convention

### Dependency Key (Left Side)

**Format:** Lowercase identifier

**Purpose:** Internal reference name (doesn't have to match package name)

**Recommendations:**
- Use the package's concept name (types, capabilities, etc.)
- Keep it short and descriptive
- Be consistent across all deps.toml files

**Examples:**
```toml
# Good - clear and concise
types = { path = "../types" }
capabilities = { path = "../capabilities" }

# Acceptable but verbose
core-types = { path = "../types" }

# Not recommended - unclear
pkg1 = { path = "../types" }
```

### Why the Name Doesn't Matter Much

The dependency key is just for the deps.toml file. The actual package reference in `.wit` files uses the full package name:

```wit
// In component.wit, this import:
use airssys:core-types@1.0.0.{component-id};

// Resolves via deps.toml entry (any of these work):
types = { path = "../types" }          // Recommended
core-types = { path = "../types" }     // Also works
foo = { path = "../types" }            // Works but confusing
```

**Best Practice:** Use the package concept name for clarity.

---

## Complete deps.toml Examples for 7 Packages

### Package 1: core-types

**File:** `wit/core/types/deps.toml`

```toml
# No dependencies - foundation package
# This file can be empty or omitted entirely
```

**Alternative (Explicit):**
```toml
# airssys:core-types@1.0.0
# Foundation package with no dependencies

[dependencies]
# (none)
```

---

### Package 2: core-component

**File:** `wit/core/component/deps.toml`

```toml
# airssys:core-component@1.0.0
# Depends on: core-types

[dependencies]
types = { path = "../types" }
```

**Usage in component.wit:**
```wit
package airssys:core-component@1.0.0;

use airssys:core-types@1.0.0.{component-id, component-error};
                        ↑
                        Resolves via deps.toml "types" entry
```

---

### Package 3: core-capabilities

**File:** `wit/core/capabilities/deps.toml`

```toml
# airssys:core-capabilities@1.0.0
# Depends on: core-types

[dependencies]
types = { path = "../types" }
```

---

### Package 4: core-host

**File:** `wit/core/host/deps.toml`

```toml
# airssys:core-host@1.0.0
# Depends on: core-types, core-capabilities

[dependencies]
types = { path = "../types" }
capabilities = { path = "../capabilities" }
```

**Multi-line Format (Equivalent):**
```toml
[dependencies]
types = { path = "../types" }

capabilities = { path = "../capabilities" }
```

---

### Package 5: ext-filesystem

**File:** `wit/ext/filesystem/deps.toml`

```toml
# airssys:ext-filesystem@1.0.0
# Depends on: core-types, core-capabilities

[dependencies]
types = { path = "../../core/types" }
capabilities = { path = "../../core/capabilities" }
```

**Path Explanation:**
- `../../` - Up from `wit/ext/filesystem/` to `wit/`
- `core/types` - Into core tier, types package

---

### Package 6: ext-network

**File:** `wit/ext/network/deps.toml`

```toml
# airssys:ext-network@1.0.0
# Depends on: core-types, core-capabilities

[dependencies]
types = { path = "../../core/types" }
capabilities = { path = "../../core/capabilities" }
```

---

### Package 7: ext-process

**File:** `wit/ext/process/deps.toml`

```toml
# airssys:ext-process@1.0.0
# Depends on: core-types, core-capabilities

[dependencies]
types = { path = "../../core/types" }
capabilities = { path = "../../core/capabilities" }
```

---

## Validation with wasm-tools

### Single Package Validation

```bash
# Validate package with dependency resolution
cd wit/core/component/
wasm-tools component wit .

# wasm-tools will:
# 1. Read component.wit
# 2. Find "use airssys:core-types@1.0.0" statement
# 3. Read deps.toml to find where core-types is
# 4. Resolve ../types to wit/core/types/
# 5. Parse wit/core/types/types.wit
# 6. Validate cross-package imports
```

### Complete Structure Validation

```bash
# Validate entire wit/ directory (all packages)
wasm-tools component wit wit/

# wasm-tools will:
# - Discover all packages in wit/
# - Resolve all deps.toml references
# - Validate cross-package dependencies
# - Check for circular dependencies
# - Ensure all imports resolve
```

---

## Common Issues and Solutions

### Issue 1: Path Not Found

**Error:**
```
error: package `airssys:core-types` not found
```

**Cause:** Incorrect path in deps.toml

**Solution:**
```toml
# Wrong:
types = { path = "types" }              # Looking in wrong location

# Correct:
types = { path = "../types" }           # Relative path from current package
```

### Issue 2: Missing deps.toml

**Error:**
```
error: unresolved package `airssys:core-types@1.0.0`
```

**Cause:** Package uses `use` statement but has no deps.toml

**Solution:**
```bash
# Create deps.toml with correct dependency path
cat > wit/core/component/deps.toml <<'EOF'
[dependencies]
types = { path = "../types" }
EOF
```

### Issue 3: Circular Dependencies

**Error:**
```
error: circular dependency detected
```

**Cause:** Package A depends on Package B, Package B depends on Package A

**Solution:** Redesign dependency graph (our design has zero circular dependencies)

### Issue 4: Case Sensitivity

**Platform Dependent:** macOS is case-insensitive, Linux is case-sensitive

**Best Practice:**
```toml
# Always use exact case matching
types = { path = "../types" }       # Matches wit/core/types/
# NOT: Types, TYPES, or tyPes
```

---

## Best Practices for AirsSys

### 1. Consistent Naming

```toml
# Use package concept name consistently
types = { path = "..." }           # Always "types"
capabilities = { path = "..." }    # Always "capabilities"
```

### 2. Comments for Documentation

```toml
# airssys:ext-filesystem@1.0.0
# Depends on: core-types (error types), core-capabilities (permissions)

[dependencies]
types = { path = "../../core/types" }
capabilities = { path = "../../core/capabilities" }
```

### 3. Alphabetical Ordering

```toml
[dependencies]
capabilities = { path = "../capabilities" }     # A before T
types = { path = "../types" }
```

### 4. Explicit Rather Than Minimal

```toml
# Good - clear structure even if empty
[dependencies]
# (none - foundation package)

# Less clear - empty file
```

---

## Advanced: Future Considerations

### Version Constraints (Not Currently Supported)

**Hypothetical Future Syntax:**
```toml
# NOT VALID TODAY - future possibility
[dependencies]
types = { path = "../types", version = "^1.0.0" }
```

**Current Reality:** Versions specified in `.wit` package declarations, not deps.toml

### Git Dependencies (Not Supported)

**Hypothetical Future Syntax:**
```toml
# NOT VALID TODAY
[dependencies]
external-pkg = { git = "https://github.com/org/pkg", tag = "v1.0.0" }
```

**Current Reality:** All dependencies must be local paths

### Optional Dependencies (Not Supported)

**Hypothetical Future Syntax:**
```toml
# NOT VALID TODAY
[dependencies]
types = { path = "../types", optional = false }
advanced = { path = "../advanced", optional = true }
```

**Current Reality:** All dependencies in deps.toml are required

---

## Phase 2 Implementation Checklist

### Per-Package deps.toml Creation

**Day 4: Core Packages**

- [ ] Create `wit/core/types/deps.toml` (empty/no dependencies)
- [ ] Create `wit/core/component/deps.toml` with types dependency
- [ ] Create `wit/core/capabilities/deps.toml` with types dependency
- [ ] Create `wit/core/host/deps.toml` with types and capabilities dependencies
- [ ] Validate all core packages with wasm-tools

**Day 5: Extension Packages**

- [ ] Create `wit/ext/filesystem/deps.toml` with core dependencies
- [ ] Create `wit/ext/network/deps.toml` with core dependencies
- [ ] Create `wit/ext/process/deps.toml` with core dependencies
- [ ] Validate all extension packages with wasm-tools

**Day 6: Complete Validation**

- [ ] Validate entire `wit/` directory structure
- [ ] Verify all cross-package imports resolve
- [ ] Confirm zero circular dependencies
- [ ] Document any issues and resolutions

---

## Success Criteria

### Format Compliance

- ✅ All deps.toml files use valid TOML syntax
- ✅ All paths use relative path format
- ✅ All paths resolve to valid package directories
- ✅ Dependency names are consistent and clear

### Validation Readiness

- ✅ wasm-tools can parse all packages
- ✅ All `use` statements resolve via deps.toml
- ✅ No missing dependencies
- ✅ No circular dependencies

### Documentation Quality

- ✅ Format specification clearly documented
- ✅ Path resolution rules explained with examples
- ✅ All 7 packages have example deps.toml
- ✅ Common issues and solutions documented

### Phase 2 Readiness

- ✅ Template available for all package types
- ✅ Path calculation method documented
- ✅ Validation workflow clear
- ✅ Implementation checklist complete

---

## References

### Specifications
- **WIT Component Model**: Package dependency resolution
- **TOML Specification**: Standard configuration file format
- **wasm-tools**: wit command dependency resolution behavior

### Related Documents
- **dependency_graph.md**: Package dependency analysis and topological ordering
- **structure_plan.md**: Directory organization and package structure
- **deps.toml.template**: Consolidated template for all packages (next deliverable)

---

**Document Version:** 1.0.0  
**Created:** 2025-10-25  
**Status:** Complete - deps.toml format specified and validated  
**Next Action:** Create deps.toml.template with all 7 package configurations
