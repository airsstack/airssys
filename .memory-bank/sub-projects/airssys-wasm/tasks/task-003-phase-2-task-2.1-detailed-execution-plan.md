# WASM-TASK-003 Phase 2 Task 2.1: Core Package Implementation - DETAILED EXECUTION PLAN

**Generated:** 2025-10-26  
**Status:** ✅ READY FOR EXECUTION  
**Duration:** 6 hours (Day 4 of Phase 2)  
**Working Directory:** `/Users/hiraq/Projects/airsstack/airssys/airssys-wasm`

---

## 🎯 Executive Summary

### Objective
Implement and validate 4 core WIT packages in topological dependency order, establishing the foundation for all other packages in the airssys-wasm WIT interface system.

### Success Metrics
- ✅ All 4 core packages implemented (`types`, `capabilities`, `component`, `host`)
- ✅ All packages validate individually with `wasm-tools component wit`
- ✅ All 4 packages validate together as complete core system
- ✅ Zero syntax errors, zero warnings
- ✅ All cross-package imports resolve correctly
- ✅ Complete documentation of validation results

### Critical Path
```
core-types (90 min) → core-capabilities (90 min) → core-host (90 min) → Complete Validation (90 min)
                    → core-component (90 min) ──────────┘
```

**Note:** `core-capabilities` and `core-component` can be implemented in parallel (both depend only on `core-types`).

---

## 📚 Prerequisites Check

### Required Knowledge
- ✅ Phase 1 Complete: WIT ecosystem researched, package structure designed
- ✅ `wasm-tools 1.240.0` installed and validated
- ✅ Package specifications documented in `docs/src/wit/package_content_design.md`
- ✅ Dependency graph validated in `docs/src/wit/reference/dependency_graph.md`
- ✅ Implementation guide available in `docs/src/wit/implementation_guide.md`

### Pre-Execution Verification
```bash
# Verify working directory
cd /Users/hiraq/Projects/airsstack/airssys/airssys-wasm

# Verify wasm-tools available
which wasm-tools
# Expected: /opt/homebrew/bin/wasm-tools (or similar)

# Verify version
wasm-tools --version
# Expected: wasm-tools 1.240.0 (or compatible)

# Verify git clean state (optional - recommended)
git status
# Recommended: Clean working tree or clear understanding of changes
```

---

## ⏱️ Time Allocation Breakdown

| Subtask | Package | Duration | Dependencies | Start Time |
|---------|---------|----------|--------------|------------|
| 2.1.1 | core-types | 90 min | None | Hour 0:00 |
| 2.1.2 | core-capabilities | 90 min | core-types | Hour 1:30 |
| 2.1.3 | core-component | 90 min | core-types | Hour 1:30 (parallel) |
| 2.1.4 | core-host | 90 min | types, capabilities | Hour 3:00 |
| 2.1.5 | Complete Validation | 90 min | All 4 packages | Hour 4:30 |

**Total:** 6 hours (360 minutes)

**Parallelization:** Subtasks 2.1.2 and 2.1.3 can run concurrently (saves 90 minutes if parallel execution supported).

---

## 🔨 SUBTASK 2.1.1: Core Types Package Implementation

**Package:** `airssys:core-types@1.0.0`  
**Duration:** 90 minutes  
**Dependencies:** None (foundation package)  
**Priority:** CRITICAL - Blocks all other packages

### Objective
Implement the foundation package containing all common types, errors, and data structures shared across the entire WIT interface system.

### Step 1.1: Create Directory Structure (5 minutes)

```bash
# Navigate to project root
cd /Users/hiraq/Projects/airsstack/airssys/airssys-wasm

# Create package directory
mkdir -p wit/core/types

# Verify creation
ls -la wit/core/types
# Expected: Empty directory
```

**Success Checkpoint:**
- ✅ Directory `wit/core/types/` exists
- ✅ Directory is empty and ready for files

**Troubleshooting:**
- Error "File exists": Directory already exists - verify it's empty before proceeding
- Permission denied: Check write permissions on `wit/` directory

---

### Step 1.2: Implement types.wit (60 minutes)

**File:** `wit/core/types/types.wit`  
**Lines:** ~120 lines  
**Content Source:** `docs/src/wit/package_content_design.md` (lines 20-128)

#### Complete File Content (Copy-Paste Ready)

```wit
package airssys:core-types@1.0.0;

interface types {
    // ═══════════════════════════════════════════════════════════
    // COMPONENT IDENTITY
    // ═══════════════════════════════════════════════════════════
    
    /// Component unique identifier
    record component-id {
        namespace: string,
        name: string,
        version: string,
    }
    
    /// Request correlation identifier
    type request-id = string;
    
    // ═══════════════════════════════════════════════════════════
    // TIMESTAMPS
    // ═══════════════════════════════════════════════════════════
    
    /// High-precision timestamp
    record timestamp {
        seconds: u64,
        nanoseconds: u32,
    }
    
    // ═══════════════════════════════════════════════════════════
    // ERROR TYPES
    // ═══════════════════════════════════════════════════════════
    
    /// Component lifecycle errors
    variant component-error {
        initialization-failed(string),
        configuration-invalid(string),
        resource-exhausted(string),
        internal-error(string),
    }
    
    /// Execution operation errors
    variant execution-error {
        invalid-input(string),
        processing-failed(string),
        timeout(string),
        resource-limit-exceeded(string),
    }
    
    /// Filesystem operation errors
    variant file-error {
        not-found(string),
        permission-denied(string),
        already-exists(string),
        io-error(string),
    }
    
    /// Network operation errors
    variant network-error {
        connection-failed(string),
        timeout(string),
        invalid-url(string),
        protocol-error(string),
    }
    
    /// Process operation errors
    variant process-error {
        spawn-failed(string),
        not-found(string),
        permission-denied(string),
        timeout(string),
    }
    
    // ═══════════════════════════════════════════════════════════
    // STATUS TYPES
    // ═══════════════════════════════════════════════════════════
    
    /// Component health status
    enum health-status {
        healthy,
        degraded,
        unhealthy,
        unknown,
    }
    
    /// Logging severity levels
    enum log-level {
        trace,
        debug,
        info,
        warn,
        error,
    }
    
    /// Execution result status
    variant execution-status {
        success,
        failed,
        timeout,
        cancelled,
    }
}
```

**Implementation Notes:**
- 18 total types: 2 records, 1 type alias, 5 error variants, 2 enums, 1 status variant
- All field names use kebab-case (WIT convention)
- All variant cases have string context for error details
- Comments use WIT triple-slash doc format (`///`)

**Success Checkpoint:**
- ✅ File `wit/core/types/types.wit` created
- ✅ Exactly 120 lines (including comments and spacing)
- ✅ Package declaration line 1: `package airssys:core-types@1.0.0;`
- ✅ Interface declaration line 3: `interface types {`
- ✅ All 18 types defined

---

### Step 1.3: Create deps.toml (5 minutes)

**File:** `wit/core/types/deps.toml`  
**Purpose:** Dependency configuration (empty for foundation package)

#### Complete File Content

```toml
# airssys:core-types@1.0.0
# Foundation package - No dependencies

[dependencies]
# (none - this is the foundation package all others depend on)
```

**Implementation Notes:**
- Empty `[dependencies]` section (no dependencies)
- Comment documents why no dependencies
- Follows template from `wit/deps.toml.template`

**Success Checkpoint:**
- ✅ File `wit/core/types/deps.toml` created
- ✅ Contains `[dependencies]` section
- ✅ No dependency entries (foundation package)

---

### Step 1.4: Validate Package (20 minutes)

#### Validation Command

```bash
# Validate package syntax and structure
wasm-tools component wit wit/core/types/

# Expected output format:
# package airssys:core-types@1.0.0;
# 
# interface types {
#   record component-id { ... }
#   type request-id = string;
#   ...
# }
```

**Expected Output:**
- Package definition printed without errors
- All types listed
- No "error:" or "warning:" messages

#### Success Criteria Checklist

- [ ] ✅ Command exits with code 0 (success)
- [ ] ✅ Package name correct: `airssys:core-types@1.0.0`
- [ ] ✅ Interface name correct: `types`
- [ ] ✅ All 18 types appear in output
- [ ] ✅ Zero syntax errors
- [ ] ✅ Zero warnings

#### Common Issues & Solutions

**Issue 1: "unexpected token ';'"**
- **Cause:** Missing semicolon after variant case or record field
- **Solution:** Check all record fields end with `,` and all variant cases end with `,`
- **Example Fix:**
  ```wit
  # Wrong:
  variant component-error {
      initialization-failed(string)  # Missing comma
  }
  
  # Correct:
  variant component-error {
      initialization-failed(string),
  }
  ```

**Issue 2: "invalid package name"**
- **Cause:** Package name doesn't follow format
- **Solution:** Verify line 1 exactly matches: `package airssys:core-types@1.0.0;`

**Issue 3: "unknown type"**
- **Cause:** Type name typo or incorrect kebab-case
- **Solution:** Verify all types use kebab-case (e.g., `component-id`, not `component_id`)

---

### Subtask 2.1.1 Completion Checklist

- [ ] ✅ Directory `wit/core/types/` created
- [ ] ✅ File `types.wit` implemented (120 lines)
- [ ] ✅ File `deps.toml` created (empty dependencies)
- [ ] ✅ Package validates successfully with `wasm-tools`
- [ ] ✅ All 18 types defined and validated
- [ ] ✅ Zero errors, zero warnings

**Duration Check:** Target 90 minutes  
**Handoff:** `core-types` package complete and ready for dependent packages

---

## 🔨 SUBTASK 2.1.2: Core Capabilities Package Implementation

**Package:** `airssys:core-capabilities@1.0.0`  
**Duration:** 90 minutes  
**Dependencies:** `airssys:core-types@1.0.0` ✅ (completed in 2.1.1)  
**Priority:** HIGH - Blocks core-host and all extension packages

### Objective
Implement the permission and capability system types used by all capability-gated operations.

### Step 2.1: Create Directory Structure (5 minutes)

```bash
cd /Users/hiraq/Projects/airsstack/airssys/airssys-wasm

mkdir -p wit/core/capabilities

ls -la wit/core/capabilities
# Expected: Empty directory
```

---

### Step 2.2: Implement capabilities.wit (60 minutes)

**File:** `wit/core/capabilities/capabilities.wit`  
**Lines:** ~90 lines  
**Content Source:** `docs/src/wit/package_content_design.md` (lines 252-343)

#### Complete File Content

```wit
package airssys:core-capabilities@1.0.0;

use airssys:core-types@1.0.0.{component-error};

interface capabilities {
    // ═══════════════════════════════════════════════════════════
    // FILESYSTEM PERMISSIONS
    // ═══════════════════════════════════════════════════════════
    
    /// Filesystem operation permission
    record filesystem-permission {
        action: filesystem-action,
        path-pattern: string,
    }
    
    /// Filesystem operation types
    enum filesystem-action {
        read,
        write,
        delete,
        list,
    }
    
    // ═══════════════════════════════════════════════════════════
    // NETWORK PERMISSIONS
    // ═══════════════════════════════════════════════════════════
    
    /// Network operation permission
    record network-permission {
        action: network-action,
        host-pattern: string,
        port: option<u16>,
    }
    
    /// Network operation types
    enum network-action {
        outbound,
        inbound,
    }
    
    // ═══════════════════════════════════════════════════════════
    // PROCESS PERMISSIONS
    // ═══════════════════════════════════════════════════════════
    
    /// Process operation permission
    record process-permission {
        action: process-action,
        command-pattern: string,
    }
    
    /// Process operation types
    enum process-action {
        spawn,
        kill,
        signal,
    }
    
    // ═══════════════════════════════════════════════════════════
    // PERMISSION AGGREGATION
    // ═══════════════════════════════════════════════════════════
    
    /// Complete set of component permissions
    record requested-permissions {
        filesystem: list<filesystem-permission>,
        network: list<network-permission>,
        process: list<process-permission>,
    }
    
    /// Permission check result
    variant permission-result {
        granted,
        denied(string),
    }
}
```

**Implementation Notes:**
- 8 total types: 4 records, 3 enums, 1 variant
- Imports `component-error` from `core-types` (currently unused but available for future)
- Permission patterns use glob-style strings
- All types support capability-based security model

---

### Step 2.3: Create deps.toml (5 minutes)

**File:** `wit/core/capabilities/deps.toml`

```toml
# airssys:core-capabilities@1.0.0
# Depends on: core-types

[dependencies]
types = { path = "../types" }
```

---

### Step 2.4: Validate Package (20 minutes)

```bash
wasm-tools component wit wit/core/capabilities/
```

**Success Criteria:**
- [ ] ✅ Command succeeds (exit code 0)
- [ ] ✅ Import from core-types resolves correctly
- [ ] ✅ All permission types defined
- [ ] ✅ Zero syntax errors
- [ ] ✅ Zero warnings

---

### Subtask 2.1.2 Completion Checklist

- [ ] ✅ Directory `wit/core/capabilities/` created
- [ ] ✅ File `capabilities.wit` implemented (90 lines)
- [ ] ✅ File `deps.toml` created (dependency on types)
- [ ] ✅ Package validates successfully
- [ ] ✅ Import from core-types resolves
- [ ] ✅ All 8 permission types defined
- [ ] ✅ Zero errors, zero warnings

**Duration Check:** Target 90 minutes

---

## 🔨 SUBTASK 2.1.3: Core Component Package Implementation

**Package:** `airssys:core-component@1.0.0`  
**Duration:** 90 minutes  
**Dependencies:** `airssys:core-types@1.0.0` ✅  
**Priority:** MEDIUM - Standalone (not imported by other packages)  
**NOTE:** Can be implemented in parallel with Subtask 2.1.2

### Step 3.2: Implement component.wit (60 minutes)

**File:** `wit/core/component/component.wit`

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
    
    /// Component initialization configuration
    record component-config {
        env-vars: list<tuple<string, string>>,
        config-data: option<list<u8>>,
        resource-limits: resource-limits,
    }
    
    /// Resource consumption limits
    record resource-limits {
        max-memory-bytes: u64,
        max-cpu-time-ms: u64,
        max-execution-time-ms: u64,
    }
    
    // ═══════════════════════════════════════════════════════════
    // EXECUTION CONTEXT
    // ═══════════════════════════════════════════════════════════
    
    /// Context for component execution
    record execution-context {
        request-id: request-id,
        timeout-ms: u64,
        caller-info: option<caller-info>,
    }
    
    /// Information about execution caller
    record caller-info {
        component-id: option<component-id>,
        external-source: option<string>,
    }
    
    // ═══════════════════════════════════════════════════════════
    // COMPONENT METADATA
    // ═══════════════════════════════════════════════════════════
    
    /// Component metadata and capabilities
    record component-metadata {
        name: string,
        version: string,
        description: string,
        author: string,
        supported-operations: list<string>,
        memory-requirements: memory-requirements,
    }
    
    /// Memory resource requirements
    record memory-requirements {
        min-memory-bytes: u64,
        max-memory-bytes: u64,
        preferred-memory-bytes: u64,
    }
    
    // ═══════════════════════════════════════════════════════════
    // LIFECYCLE FUNCTIONS
    // ═══════════════════════════════════════════════════════════
    
    /// Initialize component with configuration
    init: func(config: component-config) -> result<_, component-error>;
    
    /// Execute external RPC operation
    execute: func(
        operation: list<u8>,
        context: execution-context
    ) -> result<list<u8>, execution-error>;
    
    /// Handle inter-component message
    handle-message: func(
        sender: component-id,
        message: list<u8>
    ) -> result<_, component-error>;
    
    /// Handle async callback from request
    handle-callback: func(
        request-id: request-id,
        result: result<list<u8>, string>
    ) -> result<_, component-error>;
    
    /// Get component metadata
    metadata: func() -> component-metadata;
    
    /// Check component health status
    health: func() -> health-status;
    
    /// Gracefully shutdown component
    shutdown: func() -> result<_, component-error>;
}
```

---

## 🔨 SUBTASK 2.1.4: Core Host Package Implementation

**Package:** `airssys:core-host@1.0.0`  
**Duration:** 90 minutes  
**Dependencies:** `airssys:core-types@1.0.0` ✅, `airssys:core-capabilities@1.0.0` ✅

### Step 4.2: Implement host.wit (60 minutes)

**File:** `wit/core/host/host.wit`

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
    
    /// Inter-component messaging errors
    variant messaging-error {
        component-not-found(string),
        send-failed(string),
        timeout(string),
        invalid-message(string),
    }
    
    /// Component metadata for introspection
    record component-metadata {
        name: string,
        version: string,
        description: string,
        status: string,
    }
    
    // ═══════════════════════════════════════════════════════════
    // LOGGING
    // ═══════════════════════════════════════════════════════════
    
    /// Log message with severity and optional context
    log: func(
        level: log-level,
        message: string,
        context: option<list<tuple<string, string>>>
    );
    
    // ═══════════════════════════════════════════════════════════
    // MESSAGING
    // ═══════════════════════════════════════════════════════════
    
    /// Send fire-and-forget message to component
    send-message: func(
        target: component-id,
        message: list<u8>
    ) -> result<_, messaging-error>;
    
    /// Send request and get request ID for callback
    send-request: func(
        target: component-id,
        request: list<u8>,
        timeout-ms: u64
    ) -> result<request-id, messaging-error>;
    
    /// Cancel pending request by ID
    cancel-request: func(
        request-id: request-id
    ) -> result<_, messaging-error>;
    
    // ═══════════════════════════════════════════════════════════
    // TIMING
    // ═══════════════════════════════════════════════════════════
    
    /// Get current time in milliseconds since epoch
    current-time-millis: func() -> u64;
    
    /// Sleep for specified duration
    sleep-millis: func(duration-ms: u64);
    
    // ═══════════════════════════════════════════════════════════
    // INTROSPECTION
    // ═══════════════════════════════════════════════════════════
    
    /// List all loaded component IDs
    list-components: func() -> list<component-id>;
    
    /// Get metadata for specific component
    get-component-metadata: func(id: component-id) -> result<component-metadata, component-error>;
}
```

### Step 4.3: Create deps.toml (5 minutes)

```toml
# airssys:core-host@1.0.0
# Depends on: core-types, core-capabilities

[dependencies]
types = { path = "../types" }
capabilities = { path = "../capabilities" }
```

### Step 4.5: **CRITICAL GATE** - Validate Complete Core System (15 minutes)

```bash
# Validate ALL core packages together
wasm-tools component wit wit/core/
```

**Critical Success Criteria:**
- [ ] ✅ All 4 packages validate together
- [ ] ✅ All cross-package imports resolve correctly
- [ ] ✅ No circular dependency errors
- [ ] ✅ Zero syntax errors
- [ ] ✅ Zero warnings
- [ ] ✅ Complete core system operational

---

## 📊 TASK 2.1 FINAL CHECKLIST

### Phase 1 Gate Success Criteria (CRITICAL)

**Before proceeding to Task 2.2, ALL must be ✅:**

- [ ] ✅ All 4 core packages exist and are complete
- [ ] ✅ All 4 packages validate individually without errors
- [ ] ✅ All 4 packages validate together as system
- [ ] ✅ All cross-package imports resolve correctly
- [ ] ✅ Zero syntax errors across all packages
- [ ] ✅ Zero warnings across all packages
- [ ] ✅ Dependency graph verified as acyclic
- [ ] ✅ Complete validation documentation created
- [ ] ✅ Handoff materials prepared

**If ANY item above is ❌, DO NOT PROCEED to Task 2.2**

---

## 🚨 TROUBLESHOOTING GUIDE

### Common Issue 1: "Package not found"

**Solution:**
```toml
# Correct for same tier (e.g., capabilities importing types):
types = { path = "../types" }
```

### Common Issue 2: "Type not found in import"

**Solution:**
```wit
use airssys:core-types@1.0.0.{component-error};  // Correct
```

### Common Issue 3: "Unexpected token"

**Solution:**
```wit
# Correct - all commas present:
record example {
    field1: string,  // Comma required
    field2: u64,     // Comma required (even on last field)
}
```

---

**Document Version:** 1.0.0  
**Created:** 2025-10-26  
**Status:** Ready for Execution  
**Estimated Duration:** 6 hours (360 minutes)
