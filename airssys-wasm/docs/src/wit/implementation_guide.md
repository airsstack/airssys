# WIT Package Implementation Guide

**Version:** 1.0.0

---

## Overview

This guide provides step-by-step instructions for implementing the 7-package WIT structure. Follow this guide to create all packages with validated `deps.toml` configuration and complete cross-package dependency resolution.

**Prerequisites:** Review `package_structure_design.md` to understand the architecture

**Implementation Timeline:** Approximately 18 hours total
- Phase 1: Core packages (6 hours)
- Phase 2: Extension packages (6 hours)
- Phase 3: Validation and integration (6 hours)

---

## Implementation Order (Topological)

### Critical Path

**Must Follow This Order (Dependency-Based):**

```
Level 0: core-types           (Implement FIRST)
         ↓
Level 1: core-component       (Parallel with core-capabilities)
         core-capabilities    (Parallel with core-component)
         ↓
Level 2: core-host           (After both Level 1 packages)
         ↓
Level 3: ext-filesystem      (Parallel with ext-network and ext-process)
         ext-network         (Parallel with ext-filesystem and ext-process)
         ext-process         (Parallel with ext-filesystem and ext-network)
```

**Why This Order:**
- Dependencies must exist before dependent packages
- Topological ordering ensures clean builds
- Parallelization opportunities at Level 1 and Level 3

---

## Phase 1: Core Packages

### Hour 1: core-types Package (90 minutes)

#### Step 1.1: Create Directory Structure (5 min)

```bash
cd airssys-wasm
mkdir -p wit/core/types
```

**Verification:**
```bash
ls -la wit/core/types  # Should show empty directory
```

#### Step 1.2: Create types.wit (60 min)

**File:** `wit/core/types/types.wit`

**Content Template:**
```wit
package airssys:core-types@1.0.0;

interface types {
    // ═══════════════════════════════════════════════════════════
    // COMPONENT IDENTITY
    // ═══════════════════════════════════════════════════════════
    
    record component-id {
        namespace: string,
        name: string,
        version: string,
    }
    
    type request-id = string;
    
    // ═══════════════════════════════════════════════════════════
    // TIMESTAMPS
    // ═══════════════════════════════════════════════════════════
    
    record timestamp {
        seconds: u64,
        nanoseconds: u32,
    }
    
    // ═══════════════════════════════════════════════════════════
    // ERROR TYPES
    // ═══════════════════════════════════════════════════════════
    
    variant component-error {
        initialization-failed(string),
        configuration-invalid(string),
        resource-exhausted(string),
        internal-error(string),
    }
    
    variant execution-error {
        invalid-input(string),
        processing-failed(string),
        timeout(string),
        resource-limit-exceeded(string),
    }
    
    variant file-error {
        not-found(string),
        permission-denied(string),
        already-exists(string),
        io-error(string),
    }
    
    variant network-error {
        connection-failed(string),
        timeout(string),
        invalid-url(string),
        protocol-error(string),
    }
    
    variant process-error {
        spawn-failed(string),
        not-found(string),
        permission-denied(string),
        timeout(string),
    }
    
    // ═══════════════════════════════════════════════════════════
    // STATUS ENUMS
    // ═══════════════════════════════════════════════════════════
    
    enum health-status {
        healthy,
        degraded,
        unhealthy,
        unknown,
    }
    
    enum log-level {
        trace,
        debug,
        info,
        warn,
        error,
    }
    
    variant execution-status {
        success,
        failed,
        timeout,
        cancelled,
    }
}
```

**Design Reference:** `package_content_design.md` (Package 1 section)

#### Step 1.3: Create deps.toml (5 min)

**File:** `wit/core/types/deps.toml`

**Content:**
```toml
# airssys:core-types@1.0.0
# Foundation package with no dependencies

[dependencies]
# (none - foundation package)
```

**Template Reference:** `wit/deps.toml.template` (core-types section)

#### Step 1.4: Validate Package (20 min)

```bash
# Validate WIT syntax
wasm-tools component wit wit/core/types/

# Expected output: Package definition printed without errors
# Look for:
#   package airssys:core-types@1.0.0;
#   interface types { ... }
```

**Success Criteria:**
- ✅ No syntax errors
- ✅ Package name correct: `airssys:core-types@1.0.0`
- ✅ All types defined
- ✅ Clean output (no warnings)

**Troubleshooting:**
- Error "unexpected token" → Check WIT syntax (semicolons, braces)
- Error "invalid package name" → Verify naming format
- Error "unknown type" → Check for typos in type names

---

### Hour 2-3: core-capabilities and core-component (3 hours, parallel)

#### Parallel Implementation Strategy

**Option A: Implement Sequentially**
1. core-capabilities (1.5 hours)
2. core-component (1.5 hours)

**Option B: Implement in Parallel (if supported)**
1. Create both directories simultaneously
2. Implement both `.wit` files
3. Validate both together

#### Step 2.1: core-capabilities Package (1.5 hours)

**Step 2.1.1: Create Directory (5 min)**
```bash
mkdir -p wit/core/capabilities
```

**Step 2.1.2: Create capabilities.wit (60 min)**

**File:** `wit/core/capabilities/capabilities.wit`

**Content:**
```wit
package airssys:core-capabilities@1.0.0;

use airssys:core-types@1.0.0.{component-error};

interface capabilities {
    // ═══════════════════════════════════════════════════════════
    // FILESYSTEM PERMISSIONS
    // ═══════════════════════════════════════════════════════════
    
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
    
    // ═══════════════════════════════════════════════════════════
    // NETWORK PERMISSIONS
    // ═══════════════════════════════════════════════════════════
    
    record network-permission {
        action: network-action,
        host-pattern: string,
        port: option<u16>,
    }
    
    enum network-action {
        outbound,
        inbound,
    }
    
    // ═══════════════════════════════════════════════════════════
    // PROCESS PERMISSIONS
    // ═══════════════════════════════════════════════════════════
    
    record process-permission {
        action: process-action,
        command-pattern: string,
    }
    
    enum process-action {
        spawn,
        kill,
        signal,
    }
    
    // ═══════════════════════════════════════════════════════════
    // PERMISSION AGGREGATION
    // ═══════════════════════════════════════════════════════════
    
    record requested-permissions {
        filesystem: list<filesystem-permission>,
        network: list<network-permission>,
        process: list<process-permission>,
    }
    
    variant permission-result {
        granted,
        denied(string),
    }
}
```

**Design Reference:** `package_content_design.md` (Package 3 section)

**Step 2.1.3: Create deps.toml (5 min)**

**File:** `wit/core/capabilities/deps.toml`

**Content:**
```toml
# airssys:core-capabilities@1.0.0
# Depends on: core-types

[dependencies]
types = { path = "../types" }
```

**Step 2.1.4: Validate Package (20 min)**

```bash
wasm-tools component wit wit/core/capabilities/

# Verify:
# - use airssys:core-types@1.0.0 resolves
# - All permission types defined
# - No circular dependencies
```

---

#### Step 2.2: core-component Package (1.5 hours)

**Step 2.2.1: Create Directory (5 min)**
```bash
mkdir -p wit/core/component
```

**Step 2.2.2: Create component.wit (60 min)**

**File:** `wit/core/component/component.wit`

**Content:**
```wit
package airssys:core-component@1.0.0;

use airssys:core-types@1.0.0.{
    component-id,
    component-error,
    execution-error,
    health-status,
    request-id
};

interface component-lifecycle {
    // ═══════════════════════════════════════════════════════════
    // CONFIGURATION TYPES
    // ═══════════════════════════════════════════════════════════
    
    record component-config {
        env-vars: list<tuple<string, string>>,
        config-data: option<list<u8>>,
        resource-limits: resource-limits,
    }
    
    record resource-limits {
        max-memory-bytes: u64,
        max-cpu-time-ms: u64,
        max-execution-time-ms: u64,
    }
    
    // ═══════════════════════════════════════════════════════════
    // EXECUTION CONTEXT
    // ═══════════════════════════════════════════════════════════
    
    record execution-context {
        request-id: request-id,
        timeout-ms: u64,
        caller-info: option<caller-info>,
    }
    
    record caller-info {
        component-id: option<component-id>,
        external-source: option<string>,
    }
    
    // ═══════════════════════════════════════════════════════════
    // COMPONENT METADATA
    // ═══════════════════════════════════════════════════════════
    
    record component-metadata {
        name: string,
        version: string,
        description: string,
        author: string,
        supported-operations: list<string>,
        memory-requirements: memory-requirements,
    }
    
    record memory-requirements {
        min-memory-bytes: u64,
        max-memory-bytes: u64,
        preferred-memory-bytes: u64,
    }
    
    // ═══════════════════════════════════════════════════════════
    // LIFECYCLE FUNCTIONS
    // ═══════════════════════════════════════════════════════════
    
    init: func(config: component-config) -> result<_, component-error>;
    
    execute: func(
        operation: list<u8>,
        context: execution-context
    ) -> result<list<u8>, execution-error>;
    
    handle-message: func(
        sender: component-id,
        message: list<u8>
    ) -> result<_, component-error>;
    
    handle-callback: func(
        request-id: request-id,
        result: result<list<u8>, string>
    ) -> result<_, component-error>;
    
    metadata: func() -> component-metadata;
    health: func() -> health-status;
    shutdown: func() -> result<_, component-error>;
}
```

**Design Reference:** `package_content_design.md` (Package 2 section)

**Step 2.2.3: Create deps.toml (5 min)**

**File:** `wit/core/component/deps.toml`

**Content:**
```toml
# airssys:core-component@1.0.0
# Depends on: core-types

[dependencies]
types = { path = "../types" }
```

**Step 2.2.4: Validate Package (20 min)**

```bash
wasm-tools component wit wit/core/component/
```

---

### Hour 4: core-host Package (90 minutes)

#### Step 4.1: Create Directory (5 min)

```bash
mkdir -p wit/core/host
```

#### Step 4.2: Create host.wit (60 min)

**File:** `wit/core/host/host.wit`

**Content:**
```wit
package airssys:core-host@1.0.0;

use airssys:core-types@1.0.0.{
    component-id,
    request-id,
    component-error,
    log-level,
    timestamp
};

use airssys:core-capabilities@1.0.0.{permission-result};

interface host-services {
    // ═══════════════════════════════════════════════════════════
    // MESSAGING TYPES
    // ═══════════════════════════════════════════════════════════
    
    variant messaging-error {
        component-not-found(string),
        send-failed(string),
        timeout(string),
        invalid-message(string),
    }
    
    record component-metadata {
        name: string,
        version: string,
        description: string,
        status: string,
    }
    
    // ═══════════════════════════════════════════════════════════
    // LOGGING
    // ═══════════════════════════════════════════════════════════
    
    log: func(
        level: log-level,
        message: string,
        context: option<list<tuple<string, string>>>
    );
    
    // ═══════════════════════════════════════════════════════════
    // MESSAGING
    // ═══════════════════════════════════════════════════════════
    
    send-message: func(
        target: component-id,
        message: list<u8>
    ) -> result<_, messaging-error>;
    
    send-request: func(
        target: component-id,
        request: list<u8>,
        timeout-ms: u64
    ) -> result<request-id, messaging-error>;
    
    cancel-request: func(
        request-id: request-id
    ) -> result<_, messaging-error>;
    
    // ═══════════════════════════════════════════════════════════
    // TIMING
    // ═══════════════════════════════════════════════════════════
    
    current-time-millis: func() -> u64;
    sleep-millis: func(duration-ms: u64);
    
    // ═══════════════════════════════════════════════════════════
    // INTROSPECTION
    // ═══════════════════════════════════════════════════════════
    
    list-components: func() -> list<component-id>;
    get-component-metadata: func(id: component-id) -> result<component-metadata, component-error>;
}
```

**Design Reference:** `package_content_design.md` (Package 4 section)

#### Step 4.3: Create deps.toml (5 min)

**File:** `wit/core/host/deps.toml`

**Content:**
```toml
# airssys:core-host@1.0.0
# Depends on: core-types, core-capabilities

[dependencies]
types = { path = "../types" }
capabilities = { path = "../capabilities" }
```

#### Step 4.4: Validate Package (20 min)

```bash
wasm-tools component wit wit/core/host/
```

#### Step 4.5: Validate All Core Packages Together

```bash
# Validate entire core/ directory
wasm-tools component wit wit/core/

# Expected: All 4 packages validate together
# All cross-package imports resolve
```

---

### Phase 1 Completion Checklist

- [ ] ✅ wit/core/types/ directory created
- [ ] ✅ wit/core/types/types.wit implemented
- [ ] ✅ wit/core/types/deps.toml created
- [ ] ✅ core-types validates successfully
- [ ] ✅ wit/core/capabilities/ directory created
- [ ] ✅ wit/core/capabilities/capabilities.wit implemented
- [ ] ✅ wit/core/capabilities/deps.toml created
- [ ] ✅ core-capabilities validates successfully
- [ ] ✅ wit/core/component/ directory created
- [ ] ✅ wit/core/component/component.wit implemented
- [ ] ✅ wit/core/component/deps.toml created
- [ ] ✅ core-component validates successfully
- [ ] ✅ wit/core/host/ directory created
- [ ] ✅ wit/core/host/host.wit implemented
- [ ] ✅ wit/core/host/deps.toml created
- [ ] ✅ core-host validates successfully
- [ ] ✅ All core packages validate together

**Phase 1 Success:** 4 core packages complete and validated

---

## Phase 2: Extension Packages

### Extension Package Template

**All extension packages follow the same pattern:**
1. Create directory (5 min)
2. Create `.wit` file (60 min)
3. Create `deps.toml` with cross-tier paths (5 min)
4. Validate (20 min)

**Total per package:** ~90 minutes × 3 packages = 4.5 hours + validation time

---

### Hour 5-6: ext-filesystem Package (2 hours)

#### Step 5.1: Create Directory

```bash
mkdir -p wit/ext/filesystem
```

#### Step 5.2: Create filesystem.wit

**File:** `wit/ext/filesystem/filesystem.wit`

**Content:**
```wit
package airssys:ext-filesystem@1.0.0;

use airssys:core-types@1.0.0.{file-error, timestamp};
use airssys:core-capabilities@1.0.0.{filesystem-permission, filesystem-action};

interface filesystem {
    // ═══════════════════════════════════════════════════════════
    // FILE METADATA TYPES
    // ═══════════════════════════════════════════════════════════
    
    record file-stat {
        size: u64,
        is-directory: bool,
        is-file: bool,
        is-symlink: bool,
        created-at: option<timestamp>,
        modified-at: option<timestamp>,
        accessed-at: option<timestamp>,
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
        unknown,
    }
    
    // ═══════════════════════════════════════════════════════════
    // FILE OPERATIONS
    // ═══════════════════════════════════════════════════════════
    
    read-file: func(path: string) -> result<list<u8>, file-error>;
    write-file: func(path: string, data: list<u8>) -> result<_, file-error>;
    delete-file: func(path: string) -> result<_, file-error>;
    file-exists: func(path: string) -> result<bool, file-error>;
    
    // ═══════════════════════════════════════════════════════════
    // FILE METADATA
    // ═══════════════════════════════════════════════════════════
    
    stat: func(path: string) -> result<file-stat, file-error>;
    
    // ═══════════════════════════════════════════════════════════
    // DIRECTORY OPERATIONS
    // ═══════════════════════════════════════════════════════════
    
    list-directory: func(path: string) -> result<list<dir-entry>, file-error>;
    create-directory: func(path: string) -> result<_, file-error>;
    remove-directory: func(path: string, recursive: bool) -> result<_, file-error>;
}
```

**Design Reference:** `package_content_design.md` (Package 5 section)

#### Step 5.3: Create deps.toml (Cross-Tier Paths!)

**File:** `wit/ext/filesystem/deps.toml`

**Content:**
```toml
# airssys:ext-filesystem@1.0.0
# Depends on: core-types, core-capabilities

[dependencies]
types = { path = "../../core/types" }
capabilities = { path = "../../core/capabilities" }
```

**Critical:** Note `../../` path (cross-tier reference)

#### Step 5.4: Validate Package

```bash
wasm-tools component wit wit/ext/filesystem/
```

---

### Hour 7-8: ext-network and ext-process (2 hours)

#### ext-network Package (60 min)

**Directory:**
```bash
mkdir -p wit/ext/network
```

**File:** `wit/ext/network/network.wit`

**Content:** See `package_content_design.md` (Package 6 section)

**Key Types:** `http-request`, `http-response`, `http-method`, `network-address`

**deps.toml:**
```toml
[dependencies]
types = { path = "../../core/types" }
capabilities = { path = "../../core/capabilities" }
```

#### ext-process Package (60 min)

**Directory:**
```bash
mkdir -p wit/ext/process
```

**File:** `wit/ext/process/process.wit`

**Content:** See `package_content_design.md` (Package 7 section)

**Key Types:** `process-config`, `process-handle` (resource), `process-status`, `process-signal`

**deps.toml:**
```toml
[dependencies]
types = { path = "../../core/types" }
capabilities = { path = "../../core/capabilities" }
```

---

### Phase 2 Completion Checklist

- [ ] ✅ wit/ext/filesystem/ complete and validated
- [ ] ✅ wit/ext/network/ complete and validated
- [ ] ✅ wit/ext/process/ complete and validated
- [ ] ✅ All extension packages use correct cross-tier paths (`../../core/`)
- [ ] ✅ All extensions validate individually
- [ ] ✅ All extensions validate together with core packages

**Phase 2 Success:** 3 extension packages complete and validated

---

## Phase 3: Complete Validation and Integration

### Hour 9: Complete Structure Validation (2 hours)

#### Validation 1: Individual Package Validation

```bash
# Validate each package independently
for pkg in wit/core/{types,component,capabilities,host} wit/ext/{filesystem,network,process}; do
    echo "Validating $pkg..."
    wasm-tools component wit "$pkg/" || echo "FAILED: $pkg"
done
```

**Expected:** All 7 packages validate successfully

#### Validation 2: Complete wit/ Directory Validation

```bash
# Validate entire structure
wasm-tools component wit wit/

# Expected output: All packages printed with resolved dependencies
```

#### Validation 3: Generate Resolution Graph

```bash
# Generate complete resolution graph
wasm-tools component wit wit/ --out-dir wit-validated/

# Inspect generated files
ls -R wit-validated/
```

**Expected:** All 7 packages with resolved cross-references

---

### Hour 10: Documentation and README (2 hours)

#### Create wit/README.md

**Content:**
- Package structure overview
- Build order explanation
- Validation instructions
- Cross-reference to design documents

#### Create wit/VALIDATION.md

**Content:**
- Validation procedures
- Common issues and solutions
- Troubleshooting guide
- Expected outputs

---

### Hour 11-12: Final Validation Report (2 hours)

#### Create Complete Validation Report

**File:** `docs/src/wit/complete_structure_validation.md`

**Content:**
- Validation results for all 7 packages
- Dependency resolution confirmation
- Cross-package import verification
- Any issues encountered and resolutions
- Final sign-off

---

### Phase 3 Completion Checklist

- [ ] ✅ All 7 packages validate individually
- [ ] ✅ Complete wit/ directory validates
- [ ] ✅ Resolution graph generated successfully
- [ ] ✅ wit/README.md created
- [ ] ✅ wit/VALIDATION.md created
- [ ] ✅ Complete validation report documented
- [ ] ✅ Zero circular dependencies confirmed
- [ ] ✅ All cross-package imports resolve
- [ ] ✅ Ready for Phase 3 (Build System Integration)

**Phase 3 Success:** Complete WIT package system validated and ready

---

## Success Criteria

### Package Implementation

- ✅ All 7 packages implemented
- ✅ All `.wit` files syntactically valid
- ✅ All `deps.toml` files correctly configured
- ✅ All package names follow ADR-WASM-015 conventions

### Validation

- ✅ All individual packages validate with wasm-tools
- ✅ Complete structure validates as system
- ✅ All cross-package imports resolve
- ✅ Zero circular dependencies
- ✅ Zero warnings or errors

### Documentation

- ✅ README and VALIDATION guides created
- ✅ Complete validation report documented
- ✅ Any issues and resolutions recorded

### Next Steps

- ✅ Package structure ready for binding generation
- ✅ Dependencies configured for tool integration
- ✅ Documentation of validation and integration

---

## Troubleshooting Common Issues

### Issue: "Package not found"

**Cause:** Incorrect path in deps.toml

**Solution:**
```toml
# Wrong:
types = { path = "types" }

# Correct:
types = { path = "../types" }          # Same tier
types = { path = "../../core/types" }  # Cross tier
```

### Issue: "Circular dependency detected"

**Cause:** Package A depends on Package B, Package B depends on Package A

**Solution:** Review dependency graph (our design has zero circular dependencies, so this shouldn't occur if design is followed)

### Issue: "Unknown type in interface"

**Cause:** Type not imported or not defined

**Solution:**
```wit
// Add import for missing type
use airssys:core-types@1.0.0.{missing-type};
```

---

## Quality Checklist Before Phase 3

- [ ] All 7 packages exist with correct directory structure
- [ ] All package names match `airssys:{tier}-{type}@1.0.0` pattern
- [ ] All `.wit` files have package declaration
- [ ] All `.wit` files have correct imports
- [ ] All `deps.toml` files have correct paths
- [ ] wasm-tools validates all packages without errors
- [ ] Complete validation report shows zero issues
- [ ] Documentation (README, VALIDATION) complete
- [ ] Ready for Phase 3 wit-bindgen integration

---

## References

### Design Documents (From Task 1.2)

- `validation/structure_plan.md` - Directory organization
- `package_content_design.md` - Interface specifications
- `reference/dependency_graph.md` - Dependency analysis
- `../../wit/deps.toml.template` - Configuration template
- `reference/import_patterns.md` - Import syntax examples
- `reference/type_sharing_strategy.md` - Type placement rules

### Validation Documents

- `validation/validation_checklist.md` - Quality assurance checklist
- Task 1.1 research documents - WIT constraints and wasm-tools commands

---

**Document Version:** 1.0.0  
**Status:** Complete  
**Next:** Building and integration with your development environment
