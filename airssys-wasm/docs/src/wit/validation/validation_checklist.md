# WIT Package Validation Checklist - Phase 2 Quality Assurance

**Task Reference:** WASM-TASK-003 Phase 1 Task 1.2  
**Version:** 1.0.0

---

## Overview

This checklist provides comprehensive validation criteria for all 7 WIT packages. Use this checklist during and after Phase 2 implementation to ensure quality, correctness, and ADR-WASM-015 compliance.

**Usage:**
- Check each item as you complete implementation
- Validate continuously (don't wait until the end)
- Document any issues in validation report
- All items must be ✅ before proceeding to Phase 3

---

## Pre-Implementation Validation

### Design Documents Ready

- [ ] `wit/structure_plan.md` exists and reviewed
- [ ] `wit/package_content_design.md` exists and reviewed
- [ ] `wit/dependency_graph.md` exists and reviewed
- [ ] `wit/deps.toml.template` exists and reviewed
- [ ] `wit/import_patterns.md` exists and reviewed
- [ ] `wit/type_sharing_strategy.md` exists and reviewed
- [ ] `docs/src/wit/phase_2_implementation_guide.md` exists and reviewed

### Environment Ready

- [ ] wasm-tools 1.240.0 installed and verified
- [ ] Working directory is `airssys-wasm/`
- [ ] Git repository clean (no uncommitted changes blocking work)
- [ ] Task 1.2 design documents committed

---

## Per-Package Validation

### Package 1: core-types

#### Directory Structure

- [ ] `wit/core/types/` directory exists
- [ ] `wit/core/types/types.wit` file exists
- [ ] `wit/core/types/deps.toml` file exists (can be empty)
- [ ] No extraneous files in package directory

#### WIT File Content

- [ ] Package declaration: `package airssys:core-types@1.0.0;`
- [ ] Interface declaration: `interface types {` ... `}`
- [ ] `component-id` record defined
- [ ] `request-id` type defined
- [ ] `timestamp` record defined
- [ ] All error variants defined (component-error, execution-error, file-error, network-error, process-error)
- [ ] All status enums defined (health-status, log-level, execution-status)
- [ ] No imports (foundation package)
- [ ] All types use lowercase-with-hyphens naming

#### deps.toml Content

- [ ] File contains `[dependencies]` section
- [ ] `# (none - foundation package)` comment or similar
- [ ] No dependency entries (this is the foundation)

#### Validation

- [ ] `wasm-tools component wit wit/core/types/` succeeds
- [ ] Package name correct in output
- [ ] All types present in output
- [ ] Zero errors
- [ ] Zero warnings

---

### Package 2: core-component

#### Directory Structure

- [ ] `wit/core/component/` directory exists
- [ ] `wit/core/component/component.wit` file exists
- [ ] `wit/core/component/deps.toml` file exists
- [ ] No extraneous files in package directory

#### WIT File Content

- [ ] Package declaration: `package airssys:core-component@1.0.0;`
- [ ] Import statement for core-types with correct types
- [ ] Interface declaration: `interface component-lifecycle {` ... `}`
- [ ] `component-config` record defined
- [ ] `resource-limits` record defined
- [ ] `execution-context` record defined
- [ ] `component-metadata` record defined
- [ ] `memory-requirements` record defined
- [ ] `init` function declared
- [ ] `execute` function declared
- [ ] `handle-message` function declared
- [ ] `handle-callback` function declared
- [ ] `metadata` function declared
- [ ] `health` function declared
- [ ] `shutdown` function declared
- [ ] All imported types used correctly

#### deps.toml Content

- [ ] `[dependencies]` section present
- [ ] `types = { path = "../types" }` entry present
- [ ] Path is relative (`../types` not absolute)
- [ ] No extra dependencies

#### Validation

- [ ] `wasm-tools component wit wit/core/component/` succeeds
- [ ] Import resolves to core-types
- [ ] All functions present in output
- [ ] Zero errors
- [ ] Zero warnings

---

### Package 3: core-capabilities

#### Directory Structure

- [ ] `wit/core/capabilities/` directory exists
- [ ] `wit/core/capabilities/capabilities.wit` file exists
- [ ] `wit/core/capabilities/deps.toml` file exists
- [ ] No extraneous files in package directory

#### WIT File Content

- [ ] Package declaration: `package airssys:core-capabilities@1.0.0;`
- [ ] Import statement for core-types (component-error)
- [ ] Interface declaration: `interface capabilities {` ... `}`
- [ ] `filesystem-permission` record defined
- [ ] `filesystem-action` enum defined
- [ ] `network-permission` record defined
- [ ] `network-action` enum defined
- [ ] `process-permission` record defined
- [ ] `process-action` enum defined
- [ ] `requested-permissions` record defined
- [ ] `permission-result` variant defined

#### deps.toml Content

- [ ] `[dependencies]` section present
- [ ] `types = { path = "../types" }` entry present
- [ ] Path is relative and correct

#### Validation

- [ ] `wasm-tools component wit wit/core/capabilities/` succeeds
- [ ] Import resolves correctly
- [ ] All permission types present
- [ ] Zero errors
- [ ] Zero warnings

---

### Package 4: core-host

#### Directory Structure

- [ ] `wit/core/host/` directory exists
- [ ] `wit/core/host/host.wit` file exists
- [ ] `wit/core/host/deps.toml` file exists
- [ ] No extraneous files in package directory

#### WIT File Content

- [ ] Package declaration: `package airssys:core-host@1.0.0;`
- [ ] Import statement for core-types with multiple types
- [ ] Import statement for core-capabilities
- [ ] Interface declaration: `interface host-services {` ... `}`
- [ ] `messaging-error` variant defined
- [ ] `component-metadata` record defined (local)
- [ ] `log` function declared
- [ ] `send-message` function declared
- [ ] `send-request` function declared
- [ ] `cancel-request` function declared
- [ ] `current-time-millis` function declared
- [ ] `sleep-millis` function declared
- [ ] `list-components` function declared
- [ ] `get-component-metadata` function declared

#### deps.toml Content

- [ ] `[dependencies]` section present
- [ ] `types = { path = "../types" }` entry present
- [ ] `capabilities = { path = "../capabilities" }` entry present
- [ ] Both paths are relative and correct

#### Validation

- [ ] `wasm-tools component wit wit/core/host/` succeeds
- [ ] Both imports resolve correctly
- [ ] All functions present
- [ ] Zero errors
- [ ] Zero warnings

---

### Package 5: ext-filesystem

#### Directory Structure

- [ ] `wit/ext/filesystem/` directory exists
- [ ] `wit/ext/filesystem/filesystem.wit` file exists
- [ ] `wit/ext/filesystem/deps.toml` file exists
- [ ] No extraneous files in package directory

#### WIT File Content

- [ ] Package declaration: `package airssys:ext-filesystem@1.0.0;`
- [ ] Import statement for core-types (file-error, timestamp)
- [ ] Import statement for core-capabilities (filesystem-permission, filesystem-action)
- [ ] Interface declaration: `interface filesystem {` ... `}`
- [ ] `file-stat` record defined
- [ ] `dir-entry` record defined
- [ ] `file-type` enum defined
- [ ] `read-file` function declared
- [ ] `write-file` function declared
- [ ] `delete-file` function declared
- [ ] `file-exists` function declared
- [ ] `stat` function declared
- [ ] `list-directory` function declared
- [ ] `create-directory` function declared
- [ ] `remove-directory` function declared

#### deps.toml Content

- [ ] `[dependencies]` section present
- [ ] `types = { path = "../../core/types" }` entry present
- [ ] `capabilities = { path = "../../core/capabilities" }` entry present
- [ ] Both paths use cross-tier reference (`../../core/`)

#### Validation

- [ ] `wasm-tools component wit wit/ext/filesystem/` succeeds
- [ ] Cross-tier imports resolve correctly
- [ ] All filesystem functions present
- [ ] Zero errors
- [ ] Zero warnings

---

### Package 6: ext-network

#### Directory Structure

- [ ] `wit/ext/network/` directory exists
- [ ] `wit/ext/network/network.wit` file exists
- [ ] `wit/ext/network/deps.toml` file exists
- [ ] No extraneous files in package directory

#### WIT File Content

- [ ] Package declaration: `package airssys:ext-network@1.0.0;`
- [ ] Import statement for core-types (network-error)
- [ ] Import statement for core-capabilities (network-permission, network-action)
- [ ] Interface declaration: `interface network {` ... `}`
- [ ] `http-request` record defined
- [ ] `http-method` enum defined
- [ ] `http-response` record defined
- [ ] `network-address` record defined
- [ ] `http-request` function declared

#### deps.toml Content

- [ ] `[dependencies]` section present
- [ ] `types = { path = "../../core/types" }` entry present
- [ ] `capabilities = { path = "../../core/capabilities" }` entry present
- [ ] Both paths use cross-tier reference

#### Validation

- [ ] `wasm-tools component wit wit/ext/network/` succeeds
- [ ] Cross-tier imports resolve
- [ ] All network types present
- [ ] Zero errors
- [ ] Zero warnings

---

### Package 7: ext-process

#### Directory Structure

- [ ] `wit/ext/process/` directory exists
- [ ] `wit/ext/process/process.wit` file exists
- [ ] `wit/ext/process/deps.toml` file exists
- [ ] No extraneous files in package directory

#### WIT File Content

- [ ] Package declaration: `package airssys:ext-process@1.0.0;`
- [ ] Import statement for core-types (process-error)
- [ ] Import statement for core-capabilities (process-permission, process-action)
- [ ] Interface declaration: `interface process {` ... `}`
- [ ] `process-config` record defined
- [ ] `process-handle` resource defined
- [ ] `process-status` record defined
- [ ] `process-signal` enum defined (optional/future)
- [ ] `spawn-process` function declared
- [ ] `wait-process` function declared
- [ ] `kill-process` function declared
- [ ] `get-environment-variable` function declared

#### deps.toml Content

- [ ] `[dependencies]` section present
- [ ] `types = { path = "../../core/types" }` entry present
- [ ] `capabilities = { path = "../../core/capabilities" }` entry present
- [ ] Both paths use cross-tier reference

#### Validation

- [ ] `wasm-tools component wit wit/ext/process/` succeeds
- [ ] Cross-tier imports resolve
- [ ] All process types present
- [ ] Zero errors
- [ ] Zero warnings

---

## Cross-Package Validation

### Dependency Resolution

- [ ] core-types has no dependencies
- [ ] core-component depends only on core-types
- [ ] core-capabilities depends only on core-types
- [ ] core-host depends on core-types and core-capabilities
- [ ] ext-filesystem depends on core-types and core-capabilities
- [ ] ext-network depends on core-types and core-capabilities
- [ ] ext-process depends on core-types and core-capabilities

### Import Resolution

- [ ] All `use` statements in all packages resolve correctly
- [ ] All imported types exist in source packages
- [ ] All imported types are used in importing package
- [ ] No unused imports

### Circular Dependency Check

- [ ] Zero circular dependencies detected
- [ ] Topological ordering possible
- [ ] Build order clear (types → capabilities/component → host → extensions)

---

## Complete Structure Validation

### Full Directory Validation

- [ ] `wasm-tools component wit wit/` succeeds
- [ ] All 7 packages recognized
- [ ] All cross-package references resolve
- [ ] Complete package definition output

### Resolution Graph Generation

- [ ] `wasm-tools component wit wit/ --out-dir wit-validated/` succeeds
- [ ] `wit-validated/` directory created
- [ ] All 7 packages present in output
- [ ] Cross-references properly resolved

### Naming Convention Compliance

- [ ] All packages use `airssys:{tier}-{type}@1.0.0` pattern
- [ ] All core packages prefixed `core-`
- [ ] All extension packages prefixed `ext-`
- [ ] All package names lowercase-with-hyphens
- [ ] All versions are `1.0.0`

---

## Documentation Validation

### Required Documentation

- [ ] `wit/README.md` exists
- [ ] `wit/README.md` explains package structure
- [ ] `wit/VALIDATION.md` exists
- [ ] `wit/VALIDATION.md` documents validation procedures
- [ ] `docs/src/wit/complete_structure_validation.md` exists
- [ ] Validation report documents all checks performed
- [ ] Any issues encountered are documented with resolutions

### Documentation Quality

- [ ] All documentation uses professional tone
- [ ] No hyperbolic language (per documentation_terminology_standards.md)
- [ ] All claims backed by evidence
- [ ] All examples tested and validated
- [ ] Cross-references work correctly

---

## ADR-WASM-015 Compliance

### Structure Compliance

- [ ] Exactly 7 packages (4 core + 3 ext)
- [ ] Directory structure matches ADR-WASM-015
- [ ] Package naming matches ADR-WASM-015
- [ ] Core packages in `wit/core/`
- [ ] Extension packages in `wit/ext/`

### Dependency Compliance

- [ ] Core packages depend only on other core packages
- [ ] Extension packages depend only on core packages
- [ ] No ext→ext dependencies
- [ ] Foundation pattern (types as base)

### Type Organization Compliance

- [ ] Common types in core-types
- [ ] Permission types in core-capabilities
- [ ] Lifecycle types in core-component
- [ ] Domain types in domain packages

---

## Phase 3 Readiness

### Build System Integration Prerequisites

- [ ] All packages validate successfully
- [ ] Package structure ready for wit-bindgen
- [ ] Dependencies configured correctly
- [ ] World definitions prepared (or deferred to Phase 3)
- [ ] Clear handoff documentation exists

### Quality Metrics

- [ ] Zero errors in any validation
- [ ] Zero warnings in any validation
- [ ] 100% package naming compliance
- [ ] 100% dependency graph correctness
- [ ] 100% import resolution success

---

## Final Sign-Off

### Implementation Complete

- [ ] All 7 packages implemented
- [ ] All validation checks passed
- [ ] All documentation complete
- [ ] Ready for Phase 3

### Quality Assurance

- [ ] No assumptions made (all evidence-based)
- [ ] No placeholder content remaining
- [ ] No TODO items unresolved
- [ ] All cross-references valid

### Commit Readiness

- [ ] All files saved
- [ ] Git status clean for new commit
- [ ] Commit message prepared
- [ ] Ready to commit Phase 2 completion

---

## Validation Command Reference

### Individual Package Validation

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

### Complete Structure Validation

```bash
# Validate all packages together
wasm-tools component wit wit/

# Generate resolution graph
wasm-tools component wit wit/ --out-dir wit-validated/

# Verbose validation (if issues)
wasm-tools component wit wit/ -vv
```

### Automated Validation Script

```bash
#!/bin/bash
# validate-all.sh - Validate all packages

echo "Validating individual packages..."
for pkg in wit/core/{types,component,capabilities,host} wit/ext/{filesystem,network,process}; do
    echo "  Checking $pkg..."
    wasm-tools component wit "$pkg/" > /dev/null 2>&1
    if [ $? -eq 0 ]; then
        echo "    ✅ $pkg OK"
    else
        echo "    ❌ $pkg FAILED"
        wasm-tools component wit "$pkg/"  # Show error
        exit 1
    fi
done

echo "Validating complete structure..."
wasm-tools component wit wit/ > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "  ✅ Complete structure OK"
else
    echo "  ❌ Complete structure FAILED"
    wasm-tools component wit wit/  # Show error
    exit 1
fi

echo "✅ All validation checks passed!"
```

---

## Troubleshooting Reference

### Common Issues

**Issue:** `error: package not found`
- Check deps.toml path correctness
- Verify relative path calculation
- Ensure target package exists

**Issue:** `error: unresolved type`
- Check import statements
- Verify type exists in source package
- Ensure correct spelling

**Issue:** `error: circular dependency`
- Review dependency graph
- Check for A→B→A cycles
- Our design has zero cycles (shouldn't occur)

**Issue:** `error: invalid package name`
- Verify naming format
- Check for uppercase (should be lowercase)
- Ensure correct separators (`:` for namespace, `-` in names)

---

## Success Criteria Summary

### All Must Be ✅

- ✅ All 7 packages exist with correct structure
- ✅ All packages validate individually
- ✅ Complete structure validates
- ✅ Zero circular dependencies
- ✅ All imports resolve
- ✅ All naming conventions followed
- ✅ All documentation complete
- ✅ Ready for Phase 3

**Phase 2 Complete:** When all items in this checklist are ✅

---

**Document Version:** 1.0.0  
**Created:** 2025-10-25  
**Status:** Complete - Ready for Phase 2 quality assurance  
**Usage:** Check items during Phase 2 implementation for quality assurance
