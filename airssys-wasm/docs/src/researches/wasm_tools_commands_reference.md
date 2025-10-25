# wasm-tools Commands Reference - WIT Ecosystem Research

**Date:** 2025-10-25  
**Task:** WASM-TASK-003 Phase 1 Task 1.1  
**Tool Version:** wasm-tools 1.240.0  
**Purpose:** Complete reference for wasm-tools command usage in WIT development

---

## Core Commands Overview

### Main Command Structure
```bash
wasm-tools <COMMAND> [OPTIONS] [ARGUMENTS]
```

### Available Top-Level Commands
- `parse` - Parse WebAssembly text format
- `validate` - Validate WebAssembly binary
- `print` - Print textual form of Wasm binary
- `component` - **Primary WIT tooling** (our focus)
- `compose` - WebAssembly component composer
- `metadata` - Manipulate metadata
- `wit-smith` - WIT test case generator
- And others (see `wasm-tools --help` for complete list)

---

## Component Command (Primary Focus)

### Component Subcommands
```bash
wasm-tools component <COMMAND>
```

**Available Subcommands:**
- `new` - Encode component from core wasm binary
- `wit` - **Work with WIT text format** (our primary tool)
- `embed` - Embed metadata in core wasm module
- `targets` - Verify component conforms to world
- `link` - Link dynamic library modules
- `semver-check` - Verify world semver compatibility
- `unbundle` - Unbundle core wasm from component

---

## WIT Subcommand (Critical for Task 1.1)

### Command Syntax
```bash
wasm-tools component wit [OPTIONS] [INPUT]
```

### Input Types Supported
The `wit` subcommand intelligently detects input type:

1. **Single WIT file** (`*.wit`)
   - Parsed as single-document package
   - Example: `wasm-tools component wit types.wit`

2. **Directory**
   - Parsed as WIT package (all `*.wit` files)
   - Example: `wasm-tools component wit ./wit/core/`

3. **Binary WIT package** (`*.wat` or `*.wasm`)
   - Binary representation of WIT package
   - Example: `wasm-tools component wit package.wasm`

4. **Component binary**
   - Extract interface from component
   - Example: `wasm-tools component wit component.wasm`

5. **stdin** (when INPUT is `-` or omitted)
   - Read from standard input
   - Example: `cat types.wit | wasm-tools component wit -`

---

## Key Options for WIT Validation

### Output Control
```bash
# Print to stdout (default)
wasm-tools component wit input.wit

# Write to file
wasm-tools component wit input.wit -o output.wit

# Write to directory (entire resolution graph)
wasm-tools component wit ./wit/ --out-dir ./generated/

# Binary WebAssembly output
wasm-tools component wit input.wit --wasm -o output.wasm

# WebAssembly text format output
wasm-tools component wit input.wit --wat -o output.wat
```

### Validation Control
```bash
# Normal validation (default)
wasm-tools component wit input.wit

# Skip validation (when using --wasm or --wat)
wasm-tools component wit input.wit --wat --skip-validation

# Verbose output for debugging
wasm-tools component wit input.wit -v     # info level
wasm-tools component wit input.wit -vv    # debug level
wasm-tools component wit input.wit -vvv   # trace level
```

### Feature Flags
```bash
# Enable specific unstable feature
wasm-tools component wit input.wit --features foo

# Enable all unstable features
wasm-tools component wit input.wit --all-features
```

### Documentation Control
```bash
# Include doc comments (default)
wasm-tools component wit input.wit

# Exclude doc comments
wasm-tools component wit input.wit --no-docs
```

### JSON Output
```bash
# Output WIT as JSON representation
wasm-tools component wit input.wit --json
```

---

## Advanced Usage Patterns

### Importize World (Critical for Understanding)
```bash
# Generate world that imports a component's exports
wasm-tools component wit component.wasm --importize

# Importize specific world from WIT package
wasm-tools component wit ./wit/ --importize-world calculator

# Specify output world name
wasm-tools component wit ./wit/ --importize-world calculator \
  --importize-out-world-name calculator-import
```

### Semver-Based Import Deduplication
```bash
# Deduplicate world imports based on semver
wasm-tools component wit ./wit/ --merge-world-imports-based-on-semver myworld
```

---

## Common Validation Workflows

### Workflow 1: Validate Single WIT File
```bash
# Step 1: Parse and print (validates syntax)
wasm-tools component wit types.wit

# Step 2: Generate binary (validates semantics)
wasm-tools component wit types.wit --wasm -o types.wasm

# Step 3: Verify binary converts back
wasm-tools component wit types.wasm
```

**Expected Output:** If valid, prints WIT package definition

**Error Indicators:**
- Syntax errors: Line/column with error message
- Semantic errors: Package/interface validation failures
- Missing dependencies: Unresolved type references

### Workflow 2: Validate Multi-File WIT Package
```bash
# Step 1: Validate entire directory
wasm-tools component wit ./wit/core/

# Step 2: Generate full resolution graph
wasm-tools component wit ./wit/ --out-dir ./validated/

# Step 3: Inspect individual files
ls ./validated/
```

**Expected Output:** Directory structure with all resolved WIT files

### Workflow 3: Validate Cross-Package Dependencies
```bash
# Assuming directory structure:
# wit/
#   core/
#     types.wit
#     component.wit
#   deps.toml

# Validate with dependency resolution
wasm-tools component wit ./wit/
```

**Expected Behavior:**
- Resolves deps.toml references
- Validates cross-package imports
- Reports missing or circular dependencies

---

## Error Message Patterns

### Syntax Errors
```
error: unexpected token
  --> types.wit:5:12
   |
 5 |     record result {
   |            ^^^^^^ expected identifier
```

**Meaning:** WIT syntax violation at specific location

### Semantic Errors
```
error: package `airssys:core-types` not found
  --> component.wit:3:6
   |
 3 | use airssys:core-types@1.0.0.{types};
   |      ^^^^^^^^^^^^^^^^^^^^^ package not found
```

**Meaning:** Missing dependency or incorrect package reference

### Validation Errors
```
error: duplicate interface name
  --> types.wit:10:1
   |
10 | interface types {
   | ^^^^^^^^^^^^^^^^^ interface `types` defined multiple times
```

**Meaning:** Semantic constraint violation (duplicate definitions)

---

## Validation Success Indicators

### Single File Validation Success
```bash
$ wasm-tools component wit types.wit
package airssys:core-types@1.0.0;

interface types {
  record component-id {
    namespace: string,
    name: string,
    version: string,
  }
}
```

**Success Criteria:**
- ✅ No error messages
- ✅ Clean WIT output printed
- ✅ Exit code 0

### Directory Validation Success
```bash
$ wasm-tools component wit ./wit/core/
package airssys:core-types@1.0.0;
// ... full package definition printed
```

**Success Criteria:**
- ✅ All files parsed successfully
- ✅ Cross-references resolved
- ✅ Complete package printed

### Binary Generation Success
```bash
$ wasm-tools component wit types.wit --wasm -o types.wasm
$ echo $?
0
```

**Success Criteria:**
- ✅ No error output
- ✅ Binary file created
- ✅ Exit code 0

---

## Best Practices for AirsSys Development

### 1. Always Validate Before Commit
```bash
# Validate all WIT packages
wasm-tools component wit ./wit/core/
wasm-tools component wit ./wit/ext/
```

### 2. Use Verbose Mode for Debugging
```bash
# When validation fails, use verbose output
wasm-tools component wit ./wit/ -vv
```

### 3. Generate Binary for CI Validation
```bash
# Create reproducible binary artifacts
wasm-tools component wit ./wit/core/types.wit --wasm -o artifacts/core-types.wasm
```

### 4. Extract Interfaces from Components
```bash
# When analyzing third-party components
wasm-tools component wit third-party.wasm --out-dir ./analyzed/
```

### 5. Test Dependency Resolution
```bash
# Validate deps.toml works correctly
wasm-tools component wit ./wit/ --out-dir ./resolved/
diff -r ./wit/ ./resolved/  # Check for unexpected changes
```

---

## Validation Checklist for WASM-TASK-003

### Single Package Validation
- [ ] ✅ WIT syntax validates (`wasm-tools component wit <file>`)
- [ ] ✅ Binary generation succeeds (`--wasm -o output.wasm`)
- [ ] ✅ Binary round-trip works (`wasm-tools component wit output.wasm`)
- [ ] ✅ No warnings in verbose mode (`-vv`)

### Multi-Package Validation
- [ ] ✅ Directory validation succeeds (`wasm-tools component wit ./wit/`)
- [ ] ✅ deps.toml resolves correctly
- [ ] ✅ Cross-package imports validate
- [ ] ✅ Full resolution graph generates (`--out-dir`)

### Integration Validation
- [ ] ✅ All 7 ADR-WASM-015 packages validate individually
- [ ] ✅ Cross-dependencies resolve correctly
- [ ] ✅ No circular dependencies detected
- [ ] ✅ Semver constraints respected

---

## Command Quick Reference

```bash
# Basic validation
wasm-tools component wit <INPUT>

# Validate with verbose output
wasm-tools component wit <INPUT> -vv

# Generate binary
wasm-tools component wit <INPUT> --wasm -o <OUTPUT>

# Validate directory (package)
wasm-tools component wit ./wit/

# Generate full resolution
wasm-tools component wit ./wit/ --out-dir ./output/

# Extract interface from component
wasm-tools component wit component.wasm --out-dir ./extracted/

# JSON representation
wasm-tools component wit <INPUT> --json

# Importize component
wasm-tools component wit component.wasm --importize
```

---

## Research Questions Answered

### Q1: What version of wasm-tools are we using?
**A:** 1.240.0 (documented in `tooling_versions.md`)

### Q2: What subcommands are available for WIT validation?
**A:** `wasm-tools component wit` is the primary command with options:
- Text output (default)
- Binary output (`--wasm`)
- WAT output (`--wat`)
- JSON output (`--json`)
- Directory resolution (`--out-dir`)

### Q3: What flags/options are relevant to package validation?
**A:** Critical options:
- `-v, -vv, -vvv` - Verbose output levels
- `--skip-validation` - Skip validation (for debugging)
- `--features` / `--all-features` - Enable unstable features
- `--out-dir` - Generate full resolution graph

### Q4: How do we validate a single WIT file vs. a package directory?
**A:** 
- **Single file:** `wasm-tools component wit file.wit`
- **Directory (package):** `wasm-tools component wit ./directory/`
- Tool auto-detects input type and processes accordingly

---

**Document Version:** 1.0.0  
**Last Updated:** 2025-10-25  
**Research Status:** Complete - Commands documented and tested  
**Next Action:** Activity 1.1.2 - WIT Validation Command Study
