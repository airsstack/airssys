# WIT Package Structure Plan - ADR-WASM-015 Implementation

**Version:** 1.0.0

---

## Executive Summary

This document defines the complete WIT package structure for the airssys-wasm project, implementing the 7-package design specified in ADR-WASM-015. The structure consists of 4 core packages (foundation and required interfaces) and 3 extension packages (optional capabilities).

### Key Decisions
- **Directory-based organization**: Each package in its own subdirectory
- **Semantic naming**: `airssys:{directory}-{type}@1.0.0` pattern
- **Clear separation**: Core (required) vs Extensions (optional)
- **Evidence-based**: All decisions backed by Task 1.1 research and WASI Preview 2 patterns

---

## Complete Directory Tree

```
airssys-wasm/wit/
│
├── core/                                 # Core Packages (Required)
│   ├── types/
│   │   ├── types.wit                     # package: airssys:core-types@1.0.0
│   │   └── deps.toml                     # No dependencies (foundation)
│   │
│   ├── component/
│   │   ├── component.wit                 # package: airssys:core-component@1.0.0
│   │   └── deps.toml                     # depends on: core-types
│   │
│   ├── capabilities/
│   │   ├── capabilities.wit              # package: airssys:core-capabilities@1.0.0
│   │   └── deps.toml                     # depends on: core-types
│   │
│   └── host/
│       ├── host.wit                      # package: airssys:core-host@1.0.0
│       └── deps.toml                     # depends on: core-types, core-capabilities
│
├── ext/                                  # Extension Packages (Optional)
│   ├── filesystem/
│   │   ├── filesystem.wit                # package: airssys:ext-filesystem@1.0.0
│   │   └── deps.toml                     # depends on: core-types, core-capabilities
│   │
│   ├── network/
│   │   ├── network.wit                   # package: airssys:ext-network@1.0.0
│   │   └── deps.toml                     # depends on: core-types, core-capabilities
│   │
│   └── process/
│       ├── process.wit                   # package: airssys:ext-process@1.0.0
│       └── deps.toml                     # depends on: core-types, core-capabilities
│
├── README.md                              # Structure overview and usage guide
├── VALIDATION.md                          # Validation procedures (Phase 2)
└── .gitignore                             # Ignore generated artifacts
```

---

## Package-to-Directory Mapping

### Core Packages (4)

| Package Name | Directory Path | WIT File | Primary Purpose |
|--------------|---------------|----------|-----------------|
| `airssys:core-types@1.0.0` | `wit/core/types/` | `types.wit` | Foundation types, errors, IDs, result types |
| `airssys:core-component@1.0.0` | `wit/core/component/` | `component.wit` | Component lifecycle interfaces, metadata, health |
| `airssys:core-capabilities@1.0.0` | `wit/core/capabilities/` | `capabilities.wit` | Permission types, capability system, security |
| `airssys:core-host@1.0.0` | `wit/core/host/` | `host.wit` | Host services (logging, messaging, time) |

### Extension Packages (3)

| Package Name | Directory Path | WIT File | Primary Purpose |
|--------------|---------------|----------|-----------------|
| `airssys:ext-filesystem@1.0.0` | `wit/ext/filesystem/` | `filesystem.wit` | File operations (read, write, list, delete) |
| `airssys:ext-network@1.0.0` | `wit/ext/network/` | `network.wit` | Network operations (HTTP, TCP, UDP) |
| `airssys:ext-process@1.0.0` | `wit/ext/process/` | `process.wit` | Process operations (spawn, kill, environment) |

---

## Directory Organization Rationale

### Two-Tier Structure: core/ and ext/

**Design Decision:** Separate required core interfaces from optional extension capabilities.

**Rationale:**
1. **Clear Intent**: Developers immediately understand which packages are required vs optional
2. **Dependency Management**: Core packages form stable foundation, extensions evolve independently
3. **Component Flexibility**: Components can import only the extensions they need
4. **WASI Alignment**: Mirrors WASI Preview 2's separation of core and optional interfaces

**Evidence:** WASI Preview 2 uses similar pattern (e.g., `wasi:io` vs `wasi:http`)

### Package-per-Directory Pattern

**Design Decision:** Each package gets its own dedicated directory.

**Rationale:**
1. **Isolation**: Package boundaries clearly visible in filesystem
2. **Independent deps.toml**: Each package declares its own dependencies
3. **Scalability**: Easy to add new packages without restructuring
4. **Validation**: wasm-tools can validate individual packages

**Evidence:** Proven pattern from Task 1.1 minimal package validation

### One WIT File per Package

**Design Decision:** Each package contains single primary `.wit` file.

**Rationale:**
1. **Simplicity**: No multi-file coordination complexity within package
2. **Clarity**: Package content immediately visible in single file
3. **YAGNI Principle**: Start simple, add multiple files only if needed
4. **Validation**: Straightforward validation with wasm-tools

**Evidence:** WASI Preview 2 packages often use single-file pattern

---

## File Naming Conventions

### WIT File Naming

**Pattern:** `{concept}.wit` (lowercase, matches package concept)

**Examples:**
```
core/types/types.wit         ← Matches concept: types
core/component/component.wit  ← Matches concept: component
ext/filesystem/filesystem.wit ← Matches concept: filesystem
```

**Rationale:**
- **Consistency**: File name reflects package purpose
- **Predictability**: Easy to find relevant file
- **Convention**: Matches WASI Preview 2 naming patterns

### deps.toml Naming

**Pattern:** Always `deps.toml` (standard name)

**Rationale:**
- **wasm-tools Convention**: Expected filename for dependencies
- **Tooling Support**: Standard name recognized by ecosystem tools
- **Clarity**: No ambiguity about purpose

**Evidence:** Task 1.1 research showed deps.toml is standard name

---

## Package Naming Convention

### Format

```
airssys:{directory}-{type}@{version}
```

### Breakdown

**Namespace:** `airssys`
- All airssys-wasm packages use this namespace
- Distinguishes from WASI and other frameworks
- Lowercase identifier (per WIT specification)

**Name Part 1:** `{directory}` - `core` or `ext`
- Indicates package tier (required vs optional)
- Maps directly to directory location

**Name Part 2:** `{type}` - Functional purpose
- `types`, `component`, `capabilities`, `host` (core)
- `filesystem`, `network`, `process` (extensions)

**Separator:** Hyphen (`-`)
- Connects directory and type parts
- Standard separator in WIT naming

**Version:** `@1.0.0`
- Semantic versioning (major.minor.patch)
- All packages start at 1.0.0 (stable foundation)
- Independent versioning per package

### Complete Package Names

**Core Packages:**
```
airssys:core-types@1.0.0
airssys:core-component@1.0.0
airssys:core-capabilities@1.0.0
airssys:core-host@1.0.0
```

**Extension Packages:**
```
airssys:ext-filesystem@1.0.0
airssys:ext-network@1.0.0
airssys:ext-process@1.0.0
```

### Naming Validation

**Validation Against Task 1.1 Constraints:**
- ✅ Namespace: Lowercase (`airssys`)
- ✅ Name: Lowercase with hyphens (`core-types`, `ext-filesystem`)
- ✅ Version: Semantic versioning (`1.0.0`)
- ✅ Format: `namespace:name@version` pattern
- ✅ Separator: Colon (`:`) between namespace and name
- ✅ Terminator: Semicolon (`;`) in package declaration (in `.wit` files)

---

## Namespace Hierarchy

### Conceptual Organization

```
airssys                                    # Namespace root
├── core-*                                 # Required core packages
│   ├── types                              # Foundation (no dependencies)
│   ├── component                          # Component abstraction
│   ├── capabilities                       # Security system
│   └── host                               # Host services
└── ext-*                                  # Optional extension packages
    ├── filesystem                         # File I/O capability
    ├── network                            # Network capability
    └── process                            # Process capability
```

### Import Examples

**Component Imports Core Type:**
```wit
// In core/component/component.wit
package airssys:core-component@1.0.0;

use airssys:core-types@1.0.0.{component-id, component-error};
```

**Extension Imports Multiple Core Packages:**
```wit
// In ext/filesystem/filesystem.wit
package airssys:ext-filesystem@1.0.0;

use airssys:core-types@1.0.0.{file-error, request-id};
use airssys:core-capabilities@1.0.0.{filesystem-permission, filesystem-action};
```

---

## Directory Structure Design Principles

### 1. Locality of Concern

**Principle:** Everything related to a package is in its directory.

**Application:**
- Package WIT definition → `{package}/package-name.wit`
- Package dependencies → `{package}/deps.toml`
- Package-specific documentation → `{package}/README.md` (future)

**Benefit:** Self-contained packages, easy to understand and maintain

### 2. Hierarchical Organization

**Principle:** Top-level directories represent conceptual boundaries.

**Application:**
- `core/` → Required, stable, foundation interfaces
- `ext/` → Optional, evolving, capability interfaces

**Benefit:** Clear separation of required vs optional

### 3. Flat Within Tiers

**Principle:** No deep nesting within core/ or ext/.

**Application:**
- `core/types/` NOT `core/primitives/types/`
- `ext/filesystem/` NOT `ext/io/filesystem/`

**Benefit:** Simple navigation, avoid artificial hierarchy

### 4. Predictable Paths

**Principle:** Package location derivable from package name.

**Application:**
```
airssys:core-types@1.0.0      → wit/core/types/
airssys:ext-filesystem@1.0.0  → wit/ext/filesystem/
```

**Benefit:** Easy to locate package files

---

## Cross-Reference to ADR-WASM-015

### Alignment Verification

| ADR-WASM-015 Requirement | Implementation | Status |
|-------------------------|----------------|--------|
| 7-package structure | 4 core + 3 ext packages | ✅ Complete |
| Semantic naming | `airssys:{dir}-{type}@{version}` | ✅ Complete |
| Directory mapping | Package-per-directory | ✅ Complete |
| Core packages: types, component, capabilities, host | Implemented in `core/` | ✅ Complete |
| Extension packages: filesystem, network, process | Implemented in `ext/` | ✅ Complete |
| Independent versioning | Each package has own version | ✅ Complete |
| deps.toml dependency management | Each package has deps.toml | ✅ Complete |

**Conclusion:** Structure fully implements ADR-WASM-015 specification.

---

## File Organization Example

### Package: core-types

```
wit/core/types/
├── types.wit           # Package definition
│   ├── package airssys:core-types@1.0.0;
│   ├── interface types { ... }
│   └── (exported types used by other packages)
└── deps.toml           # Empty (no dependencies)
```

### Package: core-component (with dependency)

```
wit/core/component/
├── component.wit       # Package definition
│   ├── package airssys:core-component@1.0.0;
│   ├── use airssys:core-types@1.0.0.{component-error};
│   └── interface component { ... }
└── deps.toml           # Lists dependency on core-types
    └── [dependencies]
        └── types = { path = "../types" }
```

### Package: ext-filesystem (with multiple dependencies)

```
wit/ext/filesystem/
├── filesystem.wit      # Package definition
│   ├── package airssys:ext-filesystem@1.0.0;
│   ├── use airssys:core-types@1.0.0.{file-error};
│   ├── use airssys:core-capabilities@1.0.0.{filesystem-permission};
│   └── interface filesystem { ... }
└── deps.toml           # Lists dependencies on core packages
    └── [dependencies]
        ├── types = { path = "../../core/types" }
        └── capabilities = { path = "../../core/capabilities" }
```

---

## Implementation Checklist (Phase 2)

### Phase 2 Task 2.1: Core Package Implementation

- [ ] Create `wit/core/types/` directory
- [ ] Create `wit/core/types/types.wit` with package declaration
- [ ] Create `wit/core/types/deps.toml` (empty, no dependencies)
- [ ] Create `wit/core/component/` directory
- [ ] Create `wit/core/component/component.wit` with package declaration
- [ ] Create `wit/core/component/deps.toml` with types dependency
- [ ] Create `wit/core/capabilities/` directory
- [ ] Create `wit/core/capabilities/capabilities.wit` with package declaration
- [ ] Create `wit/core/capabilities/deps.toml` with types dependency
- [ ] Create `wit/core/host/` directory
- [ ] Create `wit/core/host/host.wit` with package declaration
- [ ] Create `wit/core/host/deps.toml` with types and capabilities dependencies

### Phase 2 Task 2.2: Extension Package Implementation

- [ ] Create `wit/ext/filesystem/` directory
- [ ] Create `wit/ext/filesystem/filesystem.wit` with package declaration
- [ ] Create `wit/ext/filesystem/deps.toml` with core dependencies
- [ ] Create `wit/ext/network/` directory
- [ ] Create `wit/ext/network/network.wit` with package declaration
- [ ] Create `wit/ext/network/deps.toml` with core dependencies
- [ ] Create `wit/ext/process/` directory
- [ ] Create `wit/ext/process/process.wit` with package declaration
- [ ] Create `wit/ext/process/deps.toml` with core dependencies

### Phase 2 Task 2.3: Validation and Documentation

- [ ] Validate all packages individually with wasm-tools
- [ ] Validate complete structure with cross-package imports
- [ ] Create `wit/README.md` with structure overview
- [ ] Create `wit/VALIDATION.md` with validation procedures
- [ ] Document any validation issues and resolutions

---

## Validation Strategy

### Individual Package Validation

```bash
# Validate each package independently
wasm-tools component wit wit/core/types/
wasm-tools component wit wit/core/component/
wasm-tools component wit wit/core/capabilities/
wasm-tools component wit wit/core/host/
wasm-tools component wit wit/ext/filesystem/
wasm-tools component wit wit/ext/network/
wasm-tools component wit wit/ext/process/
```

**Expected Result:** Each package validates without errors

### Complete Structure Validation

```bash
# Validate entire wit/ directory (all packages with dependencies)
wasm-tools component wit wit/

# Generate resolution graph for inspection
wasm-tools component wit wit/ --out-dir wit-validated/
```

**Expected Result:** All packages resolve correctly, no circular dependencies

---

## Future Extensibility

### Adding New Core Package

**Example:** Adding `core-storage` package

```bash
# 1. Create directory
mkdir -p wit/core/storage

# 2. Create package WIT file
cat > wit/core/storage/storage.wit <<'EOF'
package airssys:core-storage@1.0.0;

interface storage {
    // Storage interface definition
}
EOF

# 3. Create deps.toml
cat > wit/core/storage/deps.toml <<'EOF'
[dependencies]
types = { path = "../types" }
EOF

# 4. Validate
wasm-tools component wit wit/core/storage/
```

### Adding New Extension Package

**Example:** Adding `ext-database` package

```bash
# 1. Create directory
mkdir -p wit/ext/database

# 2. Create package WIT file
cat > wit/ext/database/database.wit <<'EOF'
package airssys:ext-database@1.0.0;

use airssys:core-types@1.0.0.{database-error};
use airssys:core-capabilities@1.0.0.{database-permission};

interface database {
    // Database interface definition
}
EOF

# 3. Create deps.toml
cat > wit/ext/database/deps.toml <<'EOF'
[dependencies]
types = { path = "../../core/types" }
capabilities = { path = "../../core/capabilities" }
EOF

# 4. Validate
wasm-tools component wit wit/ext/database/
```

**Pattern:** New packages follow exact same structure as existing packages

---

## ASCII Visualization

### Complete Structure

```
wit/
│
├─ core/ ━━━━━━━━━━━━━━━━━━ Required Packages (Foundation)
│  │
│  ├─ types/ ─────────────── airssys:core-types@1.0.0
│  │  ├─ types.wit          (No dependencies - Foundation)
│  │  └─ deps.toml
│  │
│  ├─ component/ ─────────── airssys:core-component@1.0.0
│  │  ├─ component.wit      (Depends: core-types)
│  │  └─ deps.toml
│  │
│  ├─ capabilities/ ──────── airssys:core-capabilities@1.0.0
│  │  ├─ capabilities.wit   (Depends: core-types)
│  │  └─ deps.toml
│  │
│  └─ host/ ──────────────── airssys:core-host@1.0.0
│     ├─ host.wit           (Depends: core-types, core-capabilities)
│     └─ deps.toml
│
└─ ext/ ━━━━━━━━━━━━━━━━━━━ Optional Packages (Capabilities)
   │
   ├─ filesystem/ ────────── airssys:ext-filesystem@1.0.0
   │  ├─ filesystem.wit     (Depends: core-types, core-capabilities)
   │  └─ deps.toml
   │
   ├─ network/ ──────────────  airssys:ext-network@1.0.0
   │  ├─ network.wit        (Depends: core-types, core-capabilities)
   │  └─ deps.toml
   │
   └─ process/ ────────────── airssys:ext-process@1.0.0
      ├─ process.wit        (Depends: core-types, core-capabilities)
      └─ deps.toml
```

---

## Success Criteria

### Structure Completeness
- ✅ All 7 packages have dedicated directories
- ✅ Each package has primary `.wit` file
- ✅ Each package has `deps.toml` configuration
- ✅ Directory structure matches ADR-WASM-015

### Naming Compliance
- ✅ All package names follow `airssys:{dir}-{type}@{version}` pattern
- ✅ All file names follow lowercase-with-hyphens convention
- ✅ All versions use semantic versioning

### Organization Principles
- ✅ Core packages in `core/` directory
- ✅ Extension packages in `ext/` directory
- ✅ Package-per-directory pattern applied consistently
- ✅ Flat hierarchy within core/ and ext/

### Evidence-Based Design
- ✅ Package naming validated against Task 1.1 constraints
- ✅ Directory patterns follow WASI Preview 2 examples
- ✅ Structure compatible with wasm-tools validation
- ✅ All decisions backed by documented sources

---

## References

### Task Documents
- **Task 1.1 Deliverables**: WIT specification constraints, wasm-tools commands, validation guide
- **Task 1.2 Plan**: Detailed implementation plan for this task
- **ADR-WASM-015**: WIT Package Structure Organization (authoritative source)

### Knowledge Base
- **KNOWLEDGE-WASM-004**: WIT Management Architecture
- **WASI Preview 2**: Reference examples for package patterns

### Tool Documentation
- **wasm-tools 1.240.0**: Command reference and validation workflows
- **WIT Specification**: Component Model interface definition language

---

**Document Version:** 1.0.0  
**Created:** 2025-10-25  
**Status:** Complete - Ready for Phase 2 Implementation  
**Next Action:** Create `package_content_design.md` with interface specifications
