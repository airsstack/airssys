# WIT Type Sharing Strategy - Type Placement and Reuse Guidelines

**Version:** 1.0.0

---

## Executive Summary

This document establishes the type sharing strategy for the airssys-wasm WIT package system. The strategy follows a **foundation-first** pattern where common types live in `core-types`, and domain-specific types live in their respective packages.

**Key Principles:**
- **Import, never duplicate** - Reuse types through imports, never redefine
- **Foundation types in core-types** - All cross-cutting types in single foundation package
- **Domain types in domain packages** - Package-specific types stay in that package
- **Version consistency** - All packages use same version (`@1.0.0`) for coordination

---

## Type Placement Guidelines

### Rule 1: Common Types in core-types

**Principle:** If a type is used by 2+ packages, it belongs in `core-types`.

**Examples:**
```wit
// In core-types
variant component-error { ... }       // Used by: component, host, capabilities
variant file-error { ... }            // Used by: filesystem
variant network-error { ... }         // Used by: network
type request-id = string;             // Used by: component, host
record timestamp { ... }              // Used by: filesystem, host
enum log-level { ... }                // Used by: host (could be used by others)
```

**Rationale:**
- Single source of truth
- Consistent error handling across packages
- No type duplication
- Easy to extend (add new error variants in one place)

---

### Rule 2: Domain-Specific Types in Domain Packages

**Principle:** If a type is unique to one package's domain, it stays in that package.

**Examples:**
```wit
// In ext-filesystem (filesystem-specific)
record file-stat {
    size: u64,
    is-directory: bool,
    modified-at: option<timestamp>,  // Uses timestamp from core-types
}

record dir-entry {
    name: string,
    path: string,
    file-type: file-type,
}

enum file-type {
    file,
    directory,
    symlink,
}

// These types only make sense in filesystem context
// Other packages don't need them
```

**Rationale:**
- Keeps packages focused
- Reduces coupling
- Types evolve with their domain
- Clear ownership

---

### Rule 3: Permission Types in core-capabilities

**Principle:** All permission and security-related types go in `core-capabilities`.

**Examples:**
```wit
// In core-capabilities
record filesystem-permission {
    action: filesystem-action,
    path-pattern: string,
}

enum filesystem-action {
    read,
    write,
    delete,
    list,
}

record network-permission {
    action: network-action,
    host-pattern: string,
    port: option<u16>,
}

enum network-action {
    outbound,
    inbound,
}
```

**Rationale:**
- Security types are cross-cutting
- Used by host runtime and all extensions
- Central location for permission system
- Easy to audit security model

---

### Rule 4: Lifecycle Types in core-component

**Principle:** Component lifecycle-specific types stay in `core-component`.

**Examples:**
```wit
// In core-component
record component-config {
    env-vars: list<tuple<string, string>>,
    config-data: option<list<u8>>,
    resource-limits: resource-limits,
}

record resource-limits {
    max-memory-bytes: u64,
    max-cpu-time-ms: u64,
}

record component-metadata {
    name: string,
    version: string,
    supported-operations: list<string>,
}
```

**Rationale:**
- These types define the component contract
- Only used by components and host (not by other packages)
- Component interface evolution independent of other packages

---

## Type Reuse Patterns

### Pattern 1: Import Foundation Types

**When:** Any package needs common error, ID, or status types

**How:**
```wit
// In ext-filesystem/filesystem.wit
package airssys:ext-filesystem@1.0.0;

// Import common types
use airssys:core-types@1.0.0.{file-error, timestamp};

interface filesystem {
    // Use imported types in function signatures
    read-file: func(path: string) -> result<list<u8>, file-error>;
    
    // Use imported types in record definitions
    record file-stat {
        size: u64,
        modified-at: option<timestamp>,  // ← Reused from core-types
    }
}
```

**Benefit:** Consistent types across entire framework

---

### Pattern 2: Import Permission Types

**When:** Extension packages need to reference permission types

**How:**
```wit
// In ext-network/network.wit
package airssys:ext-network@1.0.0;

use airssys:core-capabilities@1.0.0.{network-permission, network-action};

interface network {
    // Permission types used in documentation/comments
    // Actual enforcement happens in host runtime, not here
    http-request: func(request: http-request) -> result<http-response, network-error>;
}
```

**Benefit:** Permission system visible in interface definitions

---

### Pattern 3: Compose Domain Types with Foundation Types

**When:** Domain package defines complex types using foundation types

**How:**
```wit
// In ext-filesystem/filesystem.wit
use airssys:core-types@1.0.0.{file-error, timestamp};

interface filesystem {
    // Domain-specific record using foundation type
    record file-stat {
        size: u64,              // Domain-specific field
        is-directory: bool,     // Domain-specific field
        modified-at: option<timestamp>,  // Foundation type
    }
    
    // Function using both domain and foundation types
    stat: func(path: string) -> result<file-stat, file-error>;
    //                                   ^domain    ^foundation
}
```

**Benefit:** Best of both worlds - domain semantics with shared types

---

## Type Duplication: What to Avoid

### Anti-Pattern 1: Redefining Common Types

**❌ WRONG:**
```wit
// In ext-filesystem/filesystem.wit - DON'T DO THIS
variant file-error {
    not-found(string),
    permission-denied(string),
}

// In ext-network/network.wit - DON'T DO THIS
variant network-error {
    not-found(string),
    permission-denied(string),
}
```

**✅ CORRECT:**
```wit
// In core-types/types.wit - Define once
variant file-error {
    not-found(string),
    permission-denied(string),
    already-exists(string),
}

variant network-error {
    connection-failed(string),
    timeout(string),
}

// In ext-filesystem and ext-network - Import
use airssys:core-types@1.0.0.{file-error};
use airssys:core-types@1.0.0.{network-error};
```

---

### Anti-Pattern 2: Creating Package-Specific Versions of Common Concepts

**❌ WRONG:**
```wit
// In ext-filesystem
type filesystem-request-id = string;

// In ext-network
type network-request-id = string;
```

**✅ CORRECT:**
```wit
// In core-types - Single definition
type request-id = string;

// In all packages - Import and use
use airssys:core-types@1.0.0.{request-id};
```

---

### Anti-Pattern 3: Copying Types Instead of Importing

**❌ WRONG:**
```wit
// Copying timestamp definition into filesystem package
record timestamp {
    seconds: u64,
    nanoseconds: u32,
}
```

**✅ CORRECT:**
```wit
// Import from core-types
use airssys:core-types@1.0.0.{timestamp};
```

---

## Type Placement Decision Tree

```
Is this type used by multiple packages?
│
├─ YES → Place in core-types
│   │
│   └─ Is it security-related?
│       │
│       ├─ YES → Place in core-capabilities instead
│       │
│       └─ NO → core-types is correct
│
└─ NO → Is it used by only one package?
    │
    ├─ YES → Place in that package
    │   │
    │   └─ Is it component lifecycle-related?
    │       │
    │       ├─ YES → Place in core-component
    │       │
    │       └─ NO → Place in the domain package (ext-*)
    │
    └─ UNCERTAIN → Start in package, move to core-types if needed later
```

**Default Rule:** When uncertain, place in the package that uses it. Moving to core-types later is easier than the reverse.

---

## Type Evolution and Versioning

### Current Strategy: Synchronized Versioning

**All packages:** `@1.0.0` (synchronized)

**Rationale:**
- Initial release - all packages mature together
- Simple version management
- No compatibility matrix needed
- Breaking changes coordinated across all packages

### Future Strategy: Independent Versioning

**When:** After stable 1.0.0 release and ecosystem maturity

**Approach:**
```wit
// Hypothetical future scenario
core-types@1.2.0           // Stable, minor additions only
core-capabilities@1.1.0    // Stable, new permission type added
ext-filesystem@2.0.0       // Major update, breaking changes
ext-network@1.3.0          // Active development, backward compatible
```

**Compatibility Rules:**
- Core packages: Conservative versioning (major version changes are rare)
- Extension packages: More frequent evolution (independent major versions)
- Backward compatibility: Maintain across minor versions
- Breaking changes: Coordinate across dependent packages

---

## Type Sharing Across Package Boundaries

### Example 1: Error Type Sharing

**Definition (core-types):**
```wit
package airssys:core-types@1.0.0;

variant file-error {
    not-found(string),
    permission-denied(string),
    already-exists(string),
    io-error(string),
}
```

**Usage (ext-filesystem):**
```wit
package airssys:ext-filesystem@1.0.0;

use airssys:core-types@1.0.0.{file-error};

interface filesystem {
    read-file: func(path: string) -> result<list<u8>, file-error>;
    write-file: func(path: string, data: list<u8>) -> result<_, file-error>;
}
```

**Usage (component - if needed):**
```wit
package airssys:core-component@1.0.0;

use airssys:core-types@1.0.0.{file-error};

// Component can also use file-error if it does file operations
```

---

### Example 2: Timestamp Sharing

**Definition (core-types):**
```wit
record timestamp {
    seconds: u64,        // Unix timestamp
    nanoseconds: u32,    // Subsecond precision
}
```

**Usage 1 (ext-filesystem):**
```wit
use airssys:core-types@1.0.0.{timestamp};

record file-stat {
    size: u64,
    created-at: option<timestamp>,
    modified-at: option<timestamp>,
}
```

**Usage 2 (core-host):**
```wit
use airssys:core-types@1.0.0.{timestamp};

interface host-services {
    current-time: func() -> timestamp;
}
```

---

### Example 3: ID Type Sharing

**Definition (core-types):**
```wit
record component-id {
    namespace: string,
    name: string,
    version: string,
}

type request-id = string;
```

**Usage 1 (core-component):**
```wit
use airssys:core-types@1.0.0.{component-id, request-id};

interface component-lifecycle {
    handle-message: func(sender: component-id, message: list<u8>) -> result<_, component-error>;
}
```

**Usage 2 (core-host):**
```wit
use airssys:core-types@1.0.0.{component-id, request-id};

interface host-services {
    send-request: func(target: component-id, request: list<u8>) -> result<request-id, messaging-error>;
}
```

---

## Type Ownership Matrix

| Type Category | Owner Package | Importing Packages | Rationale |
|---------------|---------------|-------------------|-----------|
| Error types | core-types | All packages | Consistent error handling |
| ID types | core-types | component, host | Identity across system |
| Status enums | core-types | All packages | Common status values |
| Timestamps | core-types | filesystem, host | Shared time representation |
| Log levels | core-types | host | Logging system |
| Component config | core-component | (none) | Component implementation detail |
| Component metadata | core-component | (none) | Component contract |
| Filesystem permissions | core-capabilities | filesystem, host | Security model |
| Network permissions | core-capabilities | network, host | Security model |
| Process permissions | core-capabilities | process, host | Security model |
| File operations | ext-filesystem | (none) | Filesystem domain |
| HTTP types | ext-network | (none) | Network domain |
| Process types | ext-process | (none) | Process domain |

---

## Forward Compatibility Strategy

### Adding New Types to core-types

**Scenario:** Need to add new error type

**Approach:**
```wit
// Before (core-types@1.0.0)
variant component-error {
    initialization-failed(string),
    configuration-invalid(string),
}

// After (core-types@1.1.0) - Minor version bump
variant component-error {
    initialization-failed(string),
    configuration-invalid(string),
    dependency-missing(string),     // New variant added
}
```

**Impact:** Minor version bump (backward compatible)
- Old code: Still works, doesn't use new variant
- New code: Can use new variant
- All packages: Can upgrade to @1.1.0 independently

---

### Removing Types from core-types

**Scenario:** Need to remove unused type (rare)

**Approach:** **DON'T DO IT** - This is a breaking change

**Alternative:** Deprecate type
```wit
// Mark as deprecated in comments
/// @deprecated Use component-error instead
variant legacy-error {
    // ...
}
```

**Future:** Remove in major version (2.0.0)

---

### Changing Type Definitions

**Breaking Change Examples:**
```wit
// Before
record component-id {
    namespace: string,
    name: string,
}

// After - BREAKING (removed field)
record component-id {
    namespace: string,
    // name removed - BREAKS EXISTING CODE
}
```

**Backward Compatible Examples:**
```wit
// Before
record component-id {
    namespace: string,
    name: string,
}

// After - COMPATIBLE (added optional field)
record component-id {
    namespace: string,
    name: string,
    version: option<string>,  // New optional field - OK
}
```

---

## Type Sharing Checklist

### Before Adding a New Type

- [ ] Is this type used by multiple packages?
  - YES → Add to core-types
  - NO → Add to specific package

- [ ] Is this type security-related?
  - YES → Add to core-capabilities
  - NO → Continue to next check

- [ ] Is this type component lifecycle-related?
  - YES → Add to core-component
  - NO → Add to domain package

- [ ] Does a similar type already exist?
  - YES → Reuse existing type (import it)
  - NO → Proceed with new type

### Before Importing a Type

- [ ] Does the target package export this type?
- [ ] Is there a dependency path to the target package?
- [ ] Is the version compatible (@1.0.0 currently)?
- [ ] Is deps.toml configured correctly for this dependency?

### Before Changing an Existing Type

- [ ] Is this a breaking change?
  - YES → Requires major version bump, coordinate with users
  - NO → Proceed with minor/patch version

- [ ] How many packages use this type?
  - Document all affected packages
  - Test all affected packages after change

- [ ] Can this be done backward-compatibly?
  - Add optional fields instead of required
  - Add enum variants instead of removing
  - Extend, don't remove

---

## Success Criteria

### Strategy Completeness
- ✅ Type placement rules clearly defined
- ✅ Reuse patterns documented with examples
- ✅ Anti-patterns identified
- ✅ Decision tree provided

### Practical Guidance
- ✅ Type ownership matrix complete
- ✅ All 7 packages covered
- ✅ Examples for common scenarios
- ✅ Checklists for decision-making

### Forward Compatibility
- ✅ Versioning strategy defined
- ✅ Type evolution guidelines provided
- ✅ Breaking vs. compatible changes explained
- ✅ Migration paths documented

### Phase 2 Readiness
- ✅ Clear guidelines for type placement
- ✅ Examples ready for implementation
- ✅ Validation criteria defined
- ✅ Maintenance strategy documented

---

## References

### Source Documents
- **package_content_design.md**: Type definitions per package
- **dependency_graph.md**: Package dependency relationships
- **import_patterns.md**: Import syntax for type reuse
- **ADR-WASM-015**: Package organization rationale

### Related Documents
- **deps.toml.template**: Dependency configuration for imports
- **structure_plan.md**: Package directory organization

---

**Document Version:** 1.0.0  
**Created:** 2025-10-25  
**Status:** Complete - Type sharing strategy fully documented  
**Next Action:** Create final validation and handoff documents (Hour 6)
