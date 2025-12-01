# WASM-TASK-003 Phase 2: Implementation Foundation - Comprehensive Task Plan

**Generated:** 2025-10-26  
**Status:** Ready for Execution  
**Duration:** Days 4-6 (18 hours total)  
**Prerequisites:** ✅ ALL MET - Phase 1 Complete

---

## Executive Summary

### Phase 2 Objective

Implement all WIT packages with complete validation, building on Phase 1's research and design:

**Core Package (Task 2.1) - ✅ COMPLETE:**
- `airssys:core@1.0.0` - Single package with 4 multi-file interfaces
  - types.wit (Foundation types - Layer 0)
  - capabilities.wit (Permissions - Layer 1)
  - component-lifecycle.wit (Lifecycle management - Layer 2)
  - host-services.wit (Host services - Layer 3)

**Extension Packages (Task 2.2) - PLANNED:**
- `airssys:ext-filesystem@1.0.0` - Filesystem operations
- `airssys:ext-network@1.0.0` - Network operations
- `airssys:ext-process@1.0.0` - Process operations

**Architecture Decision:**
Due to Component Model v0.1 limitations (no cross-package type imports), core packages were consolidated into a single `airssys:core@1.0.0` package with 4 logically-separated interfaces via multi-file organization. This pragmatic approach maintains clean code organization without the complexity of nested single-file subdirectories. Extension packages will follow the same pattern or be similarly consolidated. See DEBT-WASM-003 for v0.2 migration strategy.

### Phase 1 Achievement Summary

✅ **All Prerequisites Met:**
- WIT ecosystem thoroughly researched (wasm-tools 1.240.0)
- 7-package structure fully designed
- Acyclic dependency graph validated
- ~42 WIT interfaces designed
- Build system integration strategy proven
- 25 comprehensive documents (6,500+ lines)
- 100% evidence-based approach

### Success Criteria

- ✅ All 7 packages validate individually with wasm-tools
- ✅ Complete wit/ directory validates as system
- ✅ All cross-package imports resolve correctly
- ✅ Zero circular dependencies
- ✅ 100% compliance with ADR-WASM-015 and workspace standards

---

## Task 2.1: Core Package Implementation (Day 4, 6 hours)

### Overview

Implement 4 core WIT packages in topological dependency order. Each package must validate independently before proceeding to dependent packages.

**Topological Order (MUST FOLLOW):**
```
Level 0: core-types           (90 min) - Foundation, no dependencies
Level 1: core-capabilities    (90 min) - Depends on types only
         core-component       (90 min) - Depends on types only (parallel with capabilities)
Level 2: core-host            (90 min) - Depends on types + capabilities
```

---

### Subtask 2.1.1: core-types Package Implementation (90 minutes)

**Objective:** Implement foundation package providing common types, errors, and data structures.

**Dependencies:** None (foundation package)

#### Step 1: Create Directory Structure (5 min)

```bash
cd /Users/hiraq/Projects/airsstack/airssys/airssys-wasm
mkdir -p wit/core/types
```

**Verification:**
```bash
ls -la wit/core/types  # Should show empty directory
```

#### Step 2: Create types.wit (60 min)

**File:** `wit/core/types/types.wit`

**Reference:** `docs/src/wit/package_content_design.md` (Package 1, lines 20-128)

**Content Structure:**
- Package declaration: `package airssys:core-types@1.0.0;`
- Interface: `types`
- Component Identity types (component-id, request-id)
- Timestamp types
- Error variants (component-error, execution-error, file-error, network-error, process-error)
- Status enums (health-status, log-level, execution-status)

**Key Types (Total: ~18 types):**
```wit
// Foundation types
record component-id { namespace, name, version }
type request-id = string
record timestamp { seconds, nanoseconds }

// Error variants (5 total)
variant component-error { ... }
variant execution-error { ... }
variant file-error { ... }
variant network-error { ... }
variant process-error { ... }

// Status enums (3 total)
enum health-status { healthy, degraded, unhealthy, unknown }
enum log-level { trace, debug, info, warn, error }
variant execution-status { success, failed, timeout, cancelled }
```

**Implementation Checklist:**
- [ ] Package declaration with correct name format
- [ ] Component identity types (component-id, request-id, timestamp)
- [ ] All 5 error variant types with string payloads
- [ ] All 3 status enum types
- [ ] Clean WIT syntax (semicolons, braces, commas)
- [ ] No imports (foundation package)

#### Step 3: Create deps.toml (5 min)

**File:** `wit/core/types/deps.toml`

**Content:**
```toml
# airssys:core-types@1.0.0
# Foundation package with no dependencies

[dependencies]
# (none - foundation package)
```

**Reference:** `wit/deps.toml.template` (core-types section)

#### Step 4: Validate Package (20 min)

```bash
# Validate WIT syntax
wasm-tools component wit wit/core/types/

# Expected output: Package definition printed without errors
# Verify package name: airssys:core-types@1.0.0
# Verify all types listed in interface
```

**Success Criteria:**
- ✅ No syntax errors
- ✅ Package name correct: `airssys:core-types@1.0.0`
- ✅ All ~18 types defined and exported
- ✅ Clean output (no warnings)

**Common Issues & Solutions:**
- Error "unexpected token" → Check semicolons after variant cases and record fields
- Error "invalid package name" → Verify `airssys:core-types@1.0.0` format
- Error "unknown type" → Check for typos in type names (use kebab-case)

**Validation Checkpoint:** ✅ MUST PASS before proceeding to Level 1

---

### Subtask 2.1.2: core-capabilities Package Implementation (90 minutes)

**Objective:** Implement permission and capability system types.

**Dependencies:** `airssys:core-types@1.0.0`

#### Step 1: Create Directory Structure (5 min)

```bash
mkdir -p wit/core/capabilities
```

#### Step 2: Create capabilities.wit (60 min)

**File:** `wit/core/capabilities/capabilities.wit`

**Reference:** `docs/src/wit/package_content_design.md` (Package 3, lines 252-343)

**Content Structure:**
- Package declaration: `package airssys:core-capabilities@1.0.0;`
- Import: `use airssys:core-types@1.0.0.{component-error};`
- Interface: `capabilities`
- Filesystem permissions (filesystem-permission record, filesystem-action enum)
- Network permissions (network-permission record, network-action enum)
- Process permissions (process-permission record, process-action enum)
- Permission aggregation (requested-permissions record, permission-result variant)

**Key Types (Total: ~8 types):**
```wit
// Filesystem
record filesystem-permission { action, path-pattern }
enum filesystem-action { read, write, delete, list }

// Network
record network-permission { action, host-pattern, port }
enum network-action { outbound, inbound }

// Process
record process-permission { action, command-pattern }
enum process-action { spawn, kill, signal }

// Aggregation
record requested-permissions { filesystem, network, process }
variant permission-result { granted, denied(string) }
```

**Implementation Checklist:**
- [ ] Package declaration
- [ ] Correct import from core-types
- [ ] Filesystem permission types (2 types)
- [ ] Network permission types (2 types)
- [ ] Process permission types (2 types)
- [ ] Aggregation types (2 types)

#### Step 3: Create deps.toml (5 min)

**File:** `wit/core/capabilities/deps.toml`

**Content:**
```toml
# airssys:core-capabilities@1.0.0
# Depends on: core-types

[dependencies]
types = { path = "../types" }
```

**Critical:** Verify relative path `../types` points to sibling directory.

#### Step 4: Validate Package (20 min)

```bash
wasm-tools component wit wit/core/capabilities/

# Verify:
# - use airssys:core-types@1.0.0 resolves
# - All permission types defined
# - No circular dependencies
```

**Success Criteria:**
- ✅ Package validates without errors
- ✅ Import from core-types resolves
- ✅ All 8 permission types defined
- ✅ No dependency warnings

**Validation Checkpoint:** ✅ MUST PASS before core-host

---

### Subtask 2.1.3: core-component Package Implementation (90 minutes)

**Objective:** Implement component lifecycle interface (THE contract all components must implement).

**Dependencies:** `airssys:core-types@1.0.0`

**Note:** Can be implemented **in parallel** with core-capabilities (both depend only on core-types).

#### Step 1: Create Directory Structure (5 min)

```bash
mkdir -p wit/core/component
```

#### Step 2: Create component.wit (60 min)

**File:** `wit/core/component/component.wit`

**Reference:** `docs/src/wit/package_content_design.md` (Package 2, lines 133-253)

**Content Structure:**
- Package declaration: `package airssys:core-component@1.0.0;`
- Imports from core-types (component-id, component-error, execution-error, health-status, request-id)
- Interface: `component-lifecycle`
- Configuration types (component-config, resource-limits)
- Execution context types (execution-context, caller-info)
- Metadata types (component-metadata, memory-requirements)
- Lifecycle functions (7 functions: init, execute, handle-message, handle-callback, metadata, health, shutdown)

**Key Types (Total: ~6 records + 7 functions):**
```wit
// Configuration
record component-config { env-vars, config-data, resource-limits }
record resource-limits { max-memory-bytes, max-cpu-time-ms, max-execution-time-ms }

// Execution Context
record execution-context { request-id, timeout-ms, caller-info }
record caller-info { component-id, external-source }

// Metadata
record component-metadata { name, version, description, author, supported-operations, memory-requirements }
record memory-requirements { min-memory-bytes, max-memory-bytes, preferred-memory-bytes }

// Lifecycle Functions
init: func(config: component-config) -> result<_, component-error>;
execute: func(operation: list<u8>, context: execution-context) -> result<list<u8>, execution-error>;
handle-message: func(sender: component-id, message: list<u8>) -> result<_, component-error>;
handle-callback: func(request-id: request-id, result: result<list<u8>, string>) -> result<_, component-error>;
metadata: func() -> component-metadata;
health: func() -> health-status;
shutdown: func() -> result<_, component-error>;
```

**Implementation Checklist:**
- [ ] Package declaration
- [ ] All 5 imports from core-types
- [ ] Configuration types (2 records)
- [ ] Execution context types (2 records)
- [ ] Metadata types (2 records)
- [ ] All 7 lifecycle functions with correct signatures

#### Step 3: Create deps.toml (5 min)

**File:** `wit/core/component/deps.toml`

**Content:**
```toml
# airssys:core-component@1.0.0
# Depends on: core-types

[dependencies]
types = { path = "../types" }
```

#### Step 4: Validate Package (20 min)

```bash
wasm-tools component wit wit/core/component/

# Verify all imports resolve and functions are well-formed
```

**Success Criteria:**
- ✅ Package validates
- ✅ All 5 type imports resolve
- ✅ All 7 functions defined
- ✅ Result types use correct error variants

**Validation Checkpoint:** ✅ MUST PASS before proceeding

---

### Subtask 2.1.4: core-host Package Implementation (90 minutes)

**Objective:** Implement essential host services available to ALL components.

**Dependencies:** `airssys:core-types@1.0.0`, `airssys:core-capabilities@1.0.0`

**Prerequisite:** ✅ core-capabilities MUST be validated first.

#### Step 1: Create Directory Structure (5 min)

```bash
mkdir -p wit/core/host
```

#### Step 2: Create host.wit (60 min)

**File:** `wit/core/host/host.wit`

**Reference:** `docs/src/wit/package_content_design.md` (Package 4, lines 346-443)

**Content Structure:**
- Package declaration: `package airssys:core-host@1.0.0;`
- Imports from core-types (component-id, request-id, component-error, log-level, timestamp)
- Import from core-capabilities (permission-result)
- Interface: `host-services`
- Messaging types (messaging-error variant, component-metadata record)
- Functions (8 functions: log, send-message, send-request, cancel-request, current-time-millis, sleep-millis, list-components, get-component-metadata)

**Key Types & Functions:**
```wit
// Messaging types
variant messaging-error { component-not-found, send-failed, timeout, invalid-message }
record component-metadata { name, version, description, status }

// Logging (1 function)
log: func(level: log-level, message: string, context: option<list<tuple<string, string>>>);

// Messaging (3 functions)
send-message: func(target: component-id, message: list<u8>) -> result<_, messaging-error>;
send-request: func(target: component-id, request: list<u8>, timeout-ms: u64) -> result<request-id, messaging-error>;
cancel-request: func(request-id: request-id) -> result<_, messaging-error>;

// Timing (2 functions)
current-time-millis: func() -> u64;
sleep-millis: func(duration-ms: u64);

// Introspection (2 functions)
list-components: func() -> list<component-id>;
get-component-metadata: func(id: component-id) -> result<component-metadata, component-error>;
```

**Implementation Checklist:**
- [ ] Package declaration
- [ ] All 6 imports (5 from core-types, 1 from core-capabilities)
- [ ] Messaging error types (2 types)
- [ ] All 8 host service functions with correct signatures
- [ ] Correct use of imported types in signatures

#### Step 3: Create deps.toml (5 min)

**File:** `wit/core/host/deps.toml`

**Content:**
```toml
# airssys:core-host@1.0.0
# Depends on: core-types, core-capabilities

[dependencies]
types = { path = "../types" }
capabilities = { path = "../capabilities" }
```

**Critical:** Two dependencies with sibling paths.

#### Step 4: Validate Package (20 min)

```bash
wasm-tools component wit wit/core/host/

# Verify both imports resolve and all functions defined
```

#### Step 5: Validate All Core Packages Together (CRITICAL)

```bash
# Validate entire core/ directory as system
wasm-tools component wit wit/core/

# Expected: All 4 packages validate together
# All cross-package imports resolve
# Zero circular dependencies
```

**Success Criteria:**
- ✅ core-host validates individually
- ✅ **ALL 4 core packages validate together**
- ✅ All cross-package imports resolve
- ✅ Zero dependency warnings
- ✅ Zero circular dependency errors

**Validation Checkpoint:** ✅ **PHASE 1 GATE - All 4 core packages must validate before Task 2.2**

---

### Task 2.1 Completion Checklist

- [ ] ✅ wit/core/types/ directory created
- [ ] ✅ wit/core/types/types.wit implemented (~18 types)
- [ ] ✅ wit/core/types/deps.toml created (empty dependencies)
- [ ] ✅ core-types validates successfully
- [ ] ✅ wit/core/capabilities/ directory created
- [ ] ✅ wit/core/capabilities/capabilities.wit implemented (~8 types)
- [ ] ✅ wit/core/capabilities/deps.toml created (depends on types)
- [ ] ✅ core-capabilities validates successfully
- [ ] ✅ wit/core/component/ directory created
- [ ] ✅ wit/core/component/component.wit implemented (~6 records + 7 functions)
- [ ] ✅ wit/core/component/deps.toml created (depends on types)
- [ ] ✅ core-component validates successfully
- [ ] ✅ wit/core/host/ directory created
- [ ] ✅ wit/core/host/host.wit implemented (~2 types + 8 functions)
- [ ] ✅ wit/core/host/deps.toml created (depends on types + capabilities)
- [ ] ✅ core-host validates successfully
- [ ] ✅ **All 4 core packages validate together as system**

**Task 2.1 Success:** 4 core packages complete, individually validated, and system-validated.

**Time Estimate:** 6 hours (90 min × 4 packages)

---

## Task 2.2: Extension Package Implementation (Day 5, 6 hours)

### Overview

Implement 3 extension WIT packages. All extensions can be implemented **in parallel** as they share the same dependencies (core-types, core-capabilities) and have no interdependencies.

**Implementation Order (Parallel Capable):**
```
Level 3: ext-filesystem    (2 hours) - Parallel with ext-network and ext-process
         ext-network       (2 hours) - Parallel with ext-filesystem and ext-process
         ext-process       (2 hours) - Parallel with ext-filesystem and ext-network
```

**Sequential Strategy:** Implement in any order (recommend alphabetical for simplicity).

---

### Subtask 2.2.1: ext-filesystem Package Implementation (2 hours)

**Objective:** Implement filesystem operation interfaces with capability-gated file I/O.

**Dependencies:** `airssys:core-types@1.0.0`, `airssys:core-capabilities@1.0.0`

#### Step 1: Create Directory Structure (5 min)

```bash
mkdir -p wit/ext/filesystem
```

#### Step 2: Create filesystem.wit (90 min)

**File:** `wit/ext/filesystem/filesystem.wit`

**Reference:** `docs/src/wit/package_content_design.md` (Package 5, lines 447-531)

**Content Structure:**
- Package declaration: `package airssys:ext-filesystem@1.0.0;`
- Imports from core-types (file-error, timestamp)
- Imports from core-capabilities (filesystem-permission, filesystem-action)
- Interface: `filesystem`
- File metadata types (file-stat record, dir-entry record, file-type enum)
- File operations (4 functions: read-file, write-file, delete-file, file-exists)
- File metadata (1 function: stat)
- Directory operations (3 functions: list-directory, create-directory, remove-directory)

**Key Types & Functions:**
```wit
// Metadata types
record file-stat { size, is-directory, is-file, is-symlink, created-at, modified-at, accessed-at }
record dir-entry { name, path, file-type }
enum file-type { file, directory, symlink, unknown }

// File operations (4 functions)
read-file: func(path: string) -> result<list<u8>, file-error>;
write-file: func(path: string, data: list<u8>) -> result<_, file-error>;
delete-file: func(path: string) -> result<_, file-error>;
file-exists: func(path: string) -> result<bool, file-error>;

// Metadata (1 function)
stat: func(path: string) -> result<file-stat, file-error>;

// Directory operations (3 functions)
list-directory: func(path: string) -> result<list<dir-entry>, file-error>;
create-directory: func(path: string) -> result<_, file-error>;
remove-directory: func(path: string, recursive: bool) -> result<_, file-error>;
```

**Implementation Checklist:**
- [ ] Package declaration
- [ ] Cross-tier imports from core-types (2 types)
- [ ] Cross-tier imports from core-capabilities (2 types)
- [ ] File metadata types (3 types)
- [ ] All 8 filesystem functions

#### Step 3: Create deps.toml (Cross-Tier Paths!) (5 min)

**File:** `wit/ext/filesystem/deps.toml`

**Content:**
```toml
# airssys:ext-filesystem@1.0.0
# Depends on: core-types, core-capabilities

[dependencies]
types = { path = "../../core/types" }
capabilities = { path = "../../core/capabilities" }
```

**CRITICAL:** Note `../../` path for cross-tier reference (ext → core).

#### Step 4: Validate Package (20 min)

```bash
wasm-tools component wit wit/ext/filesystem/

# Verify cross-tier imports resolve correctly
```

**Success Criteria:**
- ✅ Package validates
- ✅ Cross-tier imports resolve (../../core/*)
- ✅ All 3 metadata types + 8 functions defined
- ✅ No dependency errors

**Validation Checkpoint:** ✅ MUST PASS individually

---

### Subtask 2.2.2: ext-network Package Implementation (2 hours)

**Objective:** Implement network operation interfaces with capability-gated HTTP and socket operations.

**Dependencies:** `airssys:core-types@1.0.0`, `airssys:core-capabilities@1.0.0`

#### Step 1: Create Directory Structure (5 min)

```bash
mkdir -p wit/ext/network
```

#### Step 2: Create network.wit (90 min)

**File:** `wit/ext/network/network.wit`

**Reference:** `docs/src/wit/package_content_design.md` (Package 6, lines 534-618)

**Content Structure:**
- Package declaration: `package airssys:ext-network@1.0.0;`
- Imports from core-types (network-error)
- Imports from core-capabilities (network-permission, network-action)
- Interface: `network`
- HTTP types (http-request record, http-method enum, http-response record)
- Network address (network-address record - future use)
- HTTP client function (1 function: http-request)

**Key Types & Functions:**
```wit
// HTTP Request types
record http-request { method, url, headers, body, timeout-ms }
enum http-method { get, post, put, delete, patch, head, options }

// HTTP Response
record http-response { status-code, headers, body }

// Network Address (for future socket operations)
record network-address { host, port }

// HTTP Client (1 function)
http-request: func(request: http-request) -> result<http-response, network-error>;
```

**Implementation Checklist:**
- [ ] Package declaration
- [ ] Cross-tier imports (3 types total from core packages)
- [ ] HTTP request types (2 types: record + enum)
- [ ] HTTP response type (1 record)
- [ ] Network address type (1 record)
- [ ] HTTP client function (1 function)

#### Step 3: Create deps.toml (5 min)

**File:** `wit/ext/network/deps.toml`

**Content:**
```toml
# airssys:ext-network@1.0.0
# Depends on: core-types, core-capabilities

[dependencies]
types = { path = "../../core/types" }
capabilities = { path = "../../core/capabilities" }
```

#### Step 4: Validate Package (20 min)

```bash
wasm-tools component wit wit/ext/network/
```

**Success Criteria:**
- ✅ Package validates
- ✅ Cross-tier imports resolve
- ✅ All 4 network types + 1 function defined
- ✅ HTTP types properly structured

**Validation Checkpoint:** ✅ MUST PASS individually

---

### Subtask 2.2.3: ext-process Package Implementation (2 hours)

**Objective:** Implement process operation interfaces with capability-gated process spawning and management.

**Dependencies:** `airssys:core-types@1.0.0`, `airssys:core-capabilities@1.0.0`

#### Step 1: Create Directory Structure (5 min)

```bash
mkdir -p wit/ext/process
```

#### Step 2: Create process.wit (90 min)

**File:** `wit/ext/process/process.wit`

**Reference:** `docs/src/wit/package_content_design.md` (Package 7, lines 621-710)

**Content Structure:**
- Package declaration: `package airssys:ext-process@1.0.0;`
- Imports from core-types (process-error)
- Imports from core-capabilities (process-permission, process-action)
- Interface: `process`
- Process spawn types (process-config record)
- Process handle (process-handle resource - advanced WIT feature)
- Process status (process-status record, process-signal enum)
- Process functions (4 functions: spawn-process, wait-process, kill-process, get-environment-variable)

**Key Types & Functions:**
```wit
// Process configuration
record process-config { command, args, env, working-dir, timeout-ms }

// Process handle (resource type)
resource process-handle {
    // Resource type for managing spawned process
}

// Process status
record process-status { exit-code, running, stdout, stderr }
enum process-signal { term, kill, int, hup }

// Process functions (4 functions)
spawn-process: func(config: process-config) -> result<process-handle, process-error>;
wait-process: func(handle: process-handle, timeout-ms: u64) -> result<process-status, process-error>;
kill-process: func(handle: process-handle) -> result<_, process-error>;
get-environment-variable: func(name: string) -> option<string>;
```

**Implementation Checklist:**
- [ ] Package declaration
- [ ] Cross-tier imports (3 types from core packages)
- [ ] Process config type (1 record)
- [ ] Process handle resource (1 resource - advanced WIT)
- [ ] Process status types (2 types: record + enum)
- [ ] All 4 process management functions

**Special Note:** `resource` type is advanced WIT feature - verify syntax carefully.

#### Step 3: Create deps.toml (5 min)

**File:** `wit/ext/process/deps.toml`

**Content:**
```toml
# airssys:ext-process@1.0.0
# Depends on: core-types, core-capabilities

[dependencies]
types = { path = "../../core/types" }
capabilities = { path = "../../core/capabilities" }
```

#### Step 4: Validate Package (20 min)

```bash
wasm-tools component wit wit/ext/process/

# Pay special attention to resource type validation
```

**Success Criteria:**
- ✅ Package validates
- ✅ Cross-tier imports resolve
- ✅ Resource type validates correctly
- ✅ All 4 process functions defined

**Validation Checkpoint:** ✅ MUST PASS individually

---

### Task 2.2 Completion Checklist

- [ ] ✅ wit/ext/filesystem/ complete and validated
- [ ] ✅ wit/ext/network/ complete and validated
- [ ] ✅ wit/ext/process/ complete and validated
- [ ] ✅ All extensions use correct cross-tier paths (`../../core/`)
- [ ] ✅ All 3 extensions validate individually
- [ ] ✅ **All extensions + core packages validate together**

**Task 2.2 Success:** 3 extension packages complete and validated.

**Time Estimate:** 6 hours (2 hours × 3 packages)

---

## Task 2.3: Complete System Validation and Documentation (Day 6, 6 hours)

### Overview

Configure complete dependency graph, validate entire WIT package system, create documentation, and prepare handoff to Phase 3.

**Activities:**
1. Complete system validation (all 7 packages together)
2. Generate dependency resolution graph
3. Create wit/ directory documentation
4. Document validation procedures
5. Create phase completion report

---

### Subtask 2.3.1: Complete System Validation (2 hours)

**Objective:** Validate all 7 packages as complete integrated system.

#### Validation 1: Individual Package Validation (30 min)

**Script:**
```bash
#!/bin/bash
# Validate each package independently

echo "=== Individual Package Validation ===" > validation_log.txt

for pkg in wit/core/{types,component,capabilities,host} wit/ext/{filesystem,network,process}; do
    echo "Validating $pkg..." | tee -a validation_log.txt
    if wasm-tools component wit "$pkg/" >> validation_log.txt 2>&1; then
        echo "✅ PASS: $pkg" | tee -a validation_log.txt
    else
        echo "❌ FAIL: $pkg" | tee -a validation_log.txt
        exit 1
    fi
    echo "---" >> validation_log.txt
done

echo "✅ All 7 packages validate individually" | tee -a validation_log.txt
```

**Success Criteria:**
- ✅ All 7 packages validate without errors
- ✅ Zero syntax errors
- ✅ Zero import resolution errors

#### Validation 2: Complete wit/ Directory Validation (30 min)

**Script:**
```bash
#!/bin/bash
# Validate entire structure as integrated system

echo "=== Complete System Validation ===" >> validation_log.txt

if wasm-tools component wit wit/ >> validation_log.txt 2>&1; then
    echo "✅ PASS: Complete wit/ directory validates" | tee -a validation_log.txt
else
    echo "❌ FAIL: System validation failed" | tee -a validation_log.txt
    exit 1
fi

# Count packages in output
pkg_count=$(wasm-tools component wit wit/ 2>&1 | grep -c "package airssys:")
if [ "$pkg_count" -eq 7 ]; then
    echo "✅ All 7 packages present in resolution" | tee -a validation_log.txt
else
    echo "⚠️  Warning: Expected 7 packages, found $pkg_count" | tee -a validation_log.txt
fi
```

**Success Criteria:**
- ✅ Complete wit/ directory validates as system
- ✅ All 7 packages resolved
- ✅ Zero dependency resolution errors
- ✅ Zero circular dependency errors

#### Validation 3: Generate Resolution Graph (30 min)

**Script:**
```bash
#!/bin/bash
# Generate complete resolution graph for inspection

mkdir -p wit-validated
wasm-tools component wit wit/ --out-dir wit-validated/

# Inspect generated structure
echo "=== Resolution Graph Generated ===" >> validation_log.txt
tree wit-validated/ >> validation_log.txt 2>/dev/null || ls -R wit-validated/ >> validation_log.txt

# Verify all packages present
for pkg in core-types core-component core-capabilities core-host ext-filesystem ext-network ext-process; do
    if ls wit-validated/*${pkg}* >/dev/null 2>&1; then
        echo "✅ Found resolved: $pkg" | tee -a validation_log.txt
    else
        echo "⚠️  Missing resolved: $pkg" | tee -a validation_log.txt
    fi
done
```

**Success Criteria:**
- ✅ Resolution graph generated successfully
- ✅ All 7 packages present in output
- ✅ Dependency relationships visible

#### Validation 4: Dependency Graph Verification (30 min)

**Checklist:**
```bash
# Manual dependency verification

echo "=== Dependency Graph Verification ===" >> validation_log.txt

# Level 0: core-types (no dependencies)
echo "Level 0: core-types (foundation)" >> validation_log.txt

# Level 1: core-component, core-capabilities (depend on types only)
echo "Level 1: core-component, core-capabilities" >> validation_log.txt

# Level 2: core-host (depends on types + capabilities)
echo "Level 2: core-host" >> validation_log.txt

# Level 3: All extensions (depend on types + capabilities)
echo "Level 3: ext-filesystem, ext-network, ext-process" >> validation_log.txt

echo "✅ Topological ordering verified" >> validation_log.txt
```

**Success Criteria:**
- ✅ Topological ordering matches Phase 1 design
- ✅ Zero circular dependencies confirmed
- ✅ All cross-package imports resolve

---

### Subtask 2.3.2: Documentation Creation (2 hours)

**Objective:** Create comprehensive documentation for wit/ directory.

#### Document 1: wit/README.md (60 min)

**File:** `airssys-wasm/wit/README.md`

**Content Structure:**
```markdown
# AirsSys WASM WIT Packages

## Overview

Complete WIT (WebAssembly Interface Types) package structure for airssys-wasm framework.

**Total Packages:** 7 (4 core + 3 extension)
**Version:** All @1.0.0
**Status:** Phase 2 Implementation Complete

## Package Structure

### Core Packages (Required)

| Package | Purpose | Dependencies |
|---------|---------|--------------|
| `airssys:core-types@1.0.0` | Foundation types and errors | None |
| `airssys:core-component@1.0.0` | Component lifecycle interface | core-types |
| `airssys:core-capabilities@1.0.0` | Permission types | core-types |
| `airssys:core-host@1.0.0` | Host services | core-types, core-capabilities |

### Extension Packages (Optional)

| Package | Purpose | Dependencies |
|---------|---------|--------------|
| `airssys:ext-filesystem@1.0.0` | Filesystem operations | core-types, core-capabilities |
| `airssys:ext-network@1.0.0` | Network operations | core-types, core-capabilities |
| `airssys:ext-process@1.0.0` | Process operations | core-types, core-capabilities |

## Directory Structure

```
wit/
├── core/
│   ├── types/          → airssys:core-types@1.0.0
│   ├── component/      → airssys:core-component@1.0.0
│   ├── capabilities/   → airssys:core-capabilities@1.0.0
│   └── host/           → airssys:core-host@1.0.0
└── ext/
    ├── filesystem/     → airssys:ext-filesystem@1.0.0
    ├── network/        → airssys:ext-network@1.0.0
    └── process/        → airssys:ext-process@1.0.0
```

## Validation

See `VALIDATION.md` for validation procedures.

**Quick Validation:**
```bash
# Validate all packages
wasm-tools component wit wit/
```

## Build Order

Follow topological dependency order:

1. core-types (foundation)
2. core-component, core-capabilities (parallel)
3. core-host
4. ext-filesystem, ext-network, ext-process (parallel)

## References

- **Design Document:** `docs/src/wit/package_content_design.md`
- **Dependency Graph:** `docs/src/wit/reference/dependency_graph.md`
- **ADR-WASM-015:** WIT Package Structure Organization
```

#### Document 2: wit/VALIDATION.md (60 min)

**File:** `airssys-wasm/wit/VALIDATION.md`

**Content Structure:**
```markdown
# WIT Package Validation Procedures

## Quick Validation

```bash
# Validate complete structure
wasm-tools component wit wit/
```

## Individual Package Validation

```bash
# Core packages
wasm-tools component wit wit/core/types/
wasm-tools component wit wit/core/component/
wasm-tools component wit wit/core/capabilities/
wasm-tools component wit wit/core/host/

# Extension packages
wasm-tools component wit wit/ext/filesystem/
wasm-tools component wit wit/ext/network/
wasm-tools component wit wit/ext/process/
```

## Common Issues and Solutions

### Issue: "Package not found"

**Cause:** Incorrect path in deps.toml

**Solution:**
```toml
# Same tier (core → core)
types = { path = "../types" }

# Cross tier (ext → core)
types = { path = "../../core/types" }
```

### Issue: "Circular dependency detected"

**Cause:** Package A depends on B, B depends on A

**Solution:** Review dependency graph - our design has zero circular dependencies.

### Issue: "Unknown type in interface"

**Cause:** Type not imported or not defined

**Solution:**
```wit
// Add missing import
use airssys:core-types@1.0.0.{missing-type};
```

## Expected Validation Output

```
package airssys:core-types@1.0.0;
interface types { ... }

package airssys:core-component@1.0.0;
interface component-lifecycle { ... }

... (all 7 packages listed)
```

## Troubleshooting

**No output:** Check wasm-tools installation (`wasm-tools --version`)
**Syntax errors:** Review WIT syntax (semicolons, braces, commas)
**Import errors:** Verify deps.toml paths are correct
```

---

### Subtask 2.3.3: Phase Completion Report (2 hours)

**Objective:** Create comprehensive completion report documenting Phase 2 achievements.

#### Document: Complete Validation Report (120 min)

**File:** `docs/src/wit/phase_2_completion_report.md`

**Content Structure:**
```markdown
# WASM-TASK-003 Phase 2 Completion Report

**Completion Date:** [Date]
**Phase:** Implementation Foundation (Days 4-6)
**Duration:** [Actual hours]
**Quality:** [Assessment]

## Executive Summary

✅ **ALL PHASE 2 OBJECTIVES COMPLETE**

- 7 WIT packages implemented (4 core + 3 extension)
- All packages validate individually
- Complete system validates as integrated graph
- Zero circular dependencies
- 100% compliance with ADR-WASM-015

## Deliverables Summary

### Core Packages (4)

| Package | Status | Types | Functions | Validation |
|---------|--------|-------|-----------|------------|
| core-types | ✅ Complete | ~18 | 0 | ✅ PASS |
| core-component | ✅ Complete | ~6 | 7 | ✅ PASS |
| core-capabilities | ✅ Complete | ~8 | 0 | ✅ PASS |
| core-host | ✅ Complete | ~2 | 8 | ✅ PASS |

### Extension Packages (3)

| Package | Status | Types | Functions | Validation |
|---------|--------|-------|-----------|------------|
| ext-filesystem | ✅ Complete | ~3 | 8 | ✅ PASS |
| ext-network | ✅ Complete | ~4 | 1 | ✅ PASS |
| ext-process | ✅ Complete | ~4 | 4 | ✅ PASS |

### Total Statistics

- **Total Packages:** 7
- **Total Types:** ~45
- **Total Functions:** ~28
- **Total Lines:** [Count from actual files]
- **Validation Status:** ✅ 100% PASS

## Validation Results

[Include validation_log.txt output]

## Quality Metrics

### Code Quality
- ✅ Zero syntax errors
- ✅ Zero warnings
- ✅ 100% WIT spec compliance
- ✅ Consistent naming conventions

### Standards Compliance
- ✅ ADR-WASM-015 compliance
- ✅ Workspace standards (§2.1-§6.3)
- ✅ Package naming conventions
- ✅ deps.toml format compliance

## Phase 2 Success Criteria - ALL MET ✅

- ✅ All 7 packages implemented
- ✅ All packages validate individually
- ✅ Complete system validates
- ✅ All cross-package imports resolve
- ✅ Zero circular dependencies
- ✅ Documentation complete

## Next Steps: Phase 3 (Days 7-9)

**Ready for:** Build System Integration

**Phase 3 Tasks:**
1. Task 3.1: wit-bindgen Build Configuration
2. Task 3.2: Permission System Integration
3. Task 3.3: End-to-End Validation

**Prerequisites:** ✅ ALL MET
```

---

### Task 2.3 Completion Checklist

- [ ] ✅ All 7 packages validate individually (Subtask 2.3.1)
- [ ] ✅ Complete wit/ directory validates (Subtask 2.3.1)
- [ ] ✅ Resolution graph generated successfully (Subtask 2.3.1)
- [ ] ✅ Dependency graph verified against Phase 1 design (Subtask 2.3.1)
- [ ] ✅ wit/README.md created (Subtask 2.3.2)
- [ ] ✅ wit/VALIDATION.md created (Subtask 2.3.2)
- [ ] ✅ Phase 2 completion report documented (Subtask 2.3.3)
- [ ] ✅ Zero circular dependencies confirmed
- [ ] ✅ All cross-package imports resolve
- [ ] ✅ Ready for Phase 3 (Build System Integration)

**Task 2.3 Success:** Complete WIT package system validated and documented.

**Time Estimate:** 6 hours (2 hours validation + 2 hours docs + 2 hours report)

---

## Phase 2 Overall Success Criteria

### Package Implementation ✅

- All 7 packages implemented
- All `.wit` files syntactically valid
- All `deps.toml` files correctly configured
- All package names follow ADR-WASM-015 conventions

### Validation ✅

- All individual packages validate with wasm-tools
- Complete structure validates as system
- All cross-package imports resolve
- Zero circular dependencies
- Zero warnings or errors

### Documentation ✅

- README and VALIDATION guides created
- Complete validation report documented
- Any issues and resolutions recorded
- Handoff to Phase 3 prepared

### Quality Standards ✅

- 100% compliance with ADR-WASM-015
- 100% compliance with workspace standards
- Evidence-based implementation (no assumptions)
- Professional documentation quality

---

## Risk Mitigation Strategies

### Risk 1: WIT Syntax Errors

**Probability:** Medium  
**Impact:** Medium (blocks progress)

**Mitigation:**
- Validate after each package implementation
- Use Phase 1 templates as reference
- Check semicolons, braces, commas carefully
- Reference `docs/research/wit_specification_constraints.md`

**Contingency:** If errors persist, consult WIT specification or wasm-tools error messages.

### Risk 2: Dependency Resolution Failures

**Probability:** Low  
**Impact:** High (breaks system validation)

**Mitigation:**
- Double-check deps.toml paths (../ for same tier, ../../ for cross-tier)
- Validate dependencies exist before referencing
- Follow topological order strictly
- Test individual packages before system validation

**Contingency:** Use validation_log.txt to identify specific dependency failures.

### Risk 3: Cross-Package Import Issues

**Probability:** Low  
**Impact:** Medium (requires rework)

**Mitigation:**
- Reference `docs/src/wit/reference/import_patterns.md`
- Use exact type names from core-types package
- Verify import statements match exported types
- Test imports immediately after adding

**Contingency:** wasm-tools error messages will specify unresolved types.

### Risk 4: Time Overruns

**Probability:** Medium  
**Impact:** Low (flexible timeline)

**Mitigation:**
- Implement packages sequentially if needed
- Prioritize core packages (block extensions)
- Use parallel implementation only if confident
- Allow buffer time for validation issues

**Contingency:** Extension packages can be deferred if time-constrained.

### Risk 5: Resource Type Validation (ext-process)

**Probability:** Low  
**Impact:** Medium (advanced WIT feature)

**Mitigation:**
- Reference WASI Preview 2 resource examples
- Validate resource syntax carefully
- Test process package last (after filesystem and network)
- Consult WIT spec for resource type syntax

**Contingency:** If resource validation fails, simplify to record type temporarily.

---

## Timeline Estimates

### Task 2.1: Core Package Implementation
- **Subtask 2.1.1:** core-types (90 min)
- **Subtask 2.1.2:** core-capabilities (90 min)
- **Subtask 2.1.3:** core-component (90 min) - Can parallel with 2.1.2
- **Subtask 2.1.4:** core-host (90 min)
- **Total:** 6 hours (Day 4)

### Task 2.2: Extension Package Implementation
- **Subtask 2.2.1:** ext-filesystem (2 hours)
- **Subtask 2.2.2:** ext-network (2 hours)
- **Subtask 2.2.3:** ext-process (2 hours)
- **Total:** 6 hours (Day 5)

### Task 2.3: Complete Validation
- **Subtask 2.3.1:** System validation (2 hours)
- **Subtask 2.3.2:** Documentation (2 hours)
- **Subtask 2.3.3:** Completion report (2 hours)
- **Total:** 6 hours (Day 6)

### Phase 2 Total: 18 hours (3 days)

---

## Dependencies and Prerequisites

### Phase 1 Prerequisites (ALL MET ✅)

- ✅ WIT ecosystem research complete
- ✅ wasm-tools 1.240.0 validation workflow established
- ✅ 7-package structure fully designed
- ✅ Acyclic dependency graph validated
- ✅ Build system integration strategy proven
- ✅ ~42 WIT interfaces designed
- ✅ deps.toml template created
- ✅ Complete handoff materials available

### External Dependencies

- ✅ wasm-tools 1.240.0 installed and accessible
- ✅ Git repository for tracking changes
- ✅ Text editor with WIT syntax support (optional but helpful)

### Knowledge Prerequisites

- ✅ Familiarity with WIT syntax (from Phase 1 research)
- ✅ Understanding of package dependencies (from Phase 1 design)
- ✅ ADR-WASM-015 specification knowledge

---

## References

### Phase 1 Deliverables

**Core Design Documents:**
- `docs/src/wit/validation/structure_plan.md` - Directory organization
- `docs/src/wit/package_content_design.md` - Interface specifications
- `docs/src/wit/reference/dependency_graph.md` - Dependency analysis
- `docs/src/wit/implementation_guide.md` - Step-by-step guide

**Research Documents:**
- `docs/research/tooling_versions.md` - wasm-tools 1.240.0 documentation
- `docs/research/wasm_tools_commands_reference.md` - 420-line command reference
- `docs/research/wit_specification_constraints.md` - 540-line specification guide
- `docs/research/wasm_tools_validation_guide.md` - 412-line validation workflow

**Templates:**
- `wit/deps.toml.template` - Dependency configuration template
- `build.rs.template` - Build script template (Phase 3 use)

### ADRs and Knowledge Base

- **ADR-WASM-015:** WIT Package Structure Organization (authoritative source)
- **KNOWLEDGE-WASM-004:** WIT Management Architecture
- **WASI Preview 2:** Reference examples for interface patterns

### Workspace Standards

- **§2.1:** 3-Layer import organization
- **§4.3:** Module architecture (mod.rs pattern)
- **§5.1:** Dependency management
- **§6.1:** YAGNI principles
- **§7.2:** Documentation quality standards

---

## Memory Bank Updates Required

After Phase 2 completion, update these memory bank files:

1. `.memory-bank/sub_projects/airssys-wasm/tasks/task_003_block_2_wit_interface_system.md`
   - Update progress tracking (Phase 2 complete)
   - Document deliverables and metrics
   - Record any issues encountered

2. `.memory-bank/sub_projects/airssys-wasm/progress.md`
   - Update overall task progress (66% complete - Phase 2 of 3)
   - Add Phase 2 completion entry
   - Update next steps to Phase 3

3. `.memory-bank/sub_projects/airssys-wasm/active_context.md`
   - Update current focus to Phase 3
   - Document Phase 2 achievements
   - Update context for next session

---

## Conclusion

This comprehensive task plan provides:

✅ **Complete task breakdown** - 3 main tasks, 9 subtasks, step-by-step instructions  
✅ **Evidence-based approach** - All decisions backed by Phase 1 research  
✅ **Clear success criteria** - Validation checkpoints at every step  
✅ **Risk mitigation** - 5 identified risks with contingency plans  
✅ **Time estimates** - 18 hours total (6 hours per day × 3 days)  
✅ **Quality assurance** - Validation and documentation at every level  
✅ **Handoff preparation** - Complete documentation for Phase 3

**Phase 2 Ready:** All prerequisites met, comprehensive plan in place, clear path to success.

**Next Action:** Begin Task 2.1 (Core Package Implementation) - Subtask 2.1.1 (core-types package).

---

**Document Version:** 1.0.0  
**Created:** 2025-10-26  
**Author:** AI Agent (task-plans)  
**Status:** Ready for Execution  
**Estimated Duration:** 18 hours (Days 4-6)
