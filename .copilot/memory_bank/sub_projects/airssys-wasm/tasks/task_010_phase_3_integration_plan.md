# Phase 3 Integration Plan: Build System Integration

**Task:** WASM-TASK-003 Phase 3 Planning  
**Document Type:** INTERNAL - Implementation Plan  
**Created:** 2025-10-26  
**Status:** Ready for Execution  
**Scope:** WASM-TASK-003 Phase 3 (Days 7-9) - Build System Integration  
**Prerequisites:** ✅ Phase 2 Complete - 13 WIT files validated

---

## Executive Summary

### Phase 3 Objective

Integrate the complete WIT package system (4 packages, 13 files, 1,627 lines) with wit-bindgen to generate Rust bindings, implement permission system parsing, and validate end-to-end component loading workflow.

**Duration:** 3 days (18 hours total)  
**Tasks:** 3 main tasks (wit-bindgen integration, permission system, end-to-end validation)

### What Phase 3 Will Deliver

**Primary Deliverables:**
1. ✅ Automatic Rust binding generation from WIT files
2. ✅ Component.toml permission parsing and validation
3. ✅ Complete end-to-end component loading workflow
4. ✅ Integration tests for generated bindings
5. ✅ Production-ready build system

**Success Criteria:**
- `cargo build` automatically generates bindings from WIT files
- Generated bindings compile without errors
- Permission system parses Component.toml and validates against WIT types
- End-to-end tests demonstrate complete workflow
- Zero warnings, zero errors, 100% test coverage

---

## Input Requirements

### What Phase 3 Will Consume

**From Phase 2 (Complete ✅):**
- ✅ 13 validated WIT files (core + 3 extensions)
- ✅ 4 deps.toml configuration files
- ✅ Complete validated package system (exit code 0)
- ✅ WIT System Architecture documentation
- ✅ Zero validation errors or warnings

**File Inventory:**
```
wit/
├── core/
│   ├── types.wit (112 lines)
│   ├── capabilities.wit (89 lines)
│   ├── component-lifecycle.wit (105 lines)
│   ├── host-services.wit (88 lines)
│   └── deps.toml
├── ext/
│   ├── filesystem/
│   │   ├── types.wit (140 lines)
│   │   ├── filesystem.wit (113 lines)
│   │   ├── metadata.wit (118 lines)
│   │   └── deps.toml
│   ├── network/
│   │   ├── types.wit (165 lines)
│   │   ├── socket.wit (133 lines)
│   │   ├── connection.wit (124 lines)
│   │   └── deps.toml
│   └── process/
│       ├── types.wit (145 lines)
│       ├── lifecycle.wit (140 lines)
│       ├── signals.wit (155 lines)
│       └── deps.toml
└── deps.toml.template
```

**External Dependencies:**
- wasm-tools 1.240.0 (installed and validated)
- wit-bindgen CLI (to be installed in Task 3.1)
- Rust toolchain with wasm32-wasi target
- Cargo build system

---

## wit-bindgen Integration

### Phase 3 Task 3.1: wit-bindgen Build Configuration (Day 7, 6 hours)

**Objective:** Configure wit-bindgen for automatic Rust binding generation on every build.

#### wit-bindgen CLI Approach

**Why CLI Instead of Rust Library:**
- **Wasmtime Host Compatibility:** wit-bindgen crate requires "wasmtime" feature for host bindings
- **wasm32 Target Compilation:** Components compile to wasm32-wasi target, CLI handles both
- **Multi-Package Support:** CLI can process multiple WIT packages in single invocation
- **Build Script Simplicity:** Cleaner build.rs without complex feature management

**Installation:**
```bash
cargo install wit-bindgen-cli
# Version: >=0.30.0 (Component Model v0.1 support)
```

#### Build Integration Workflow

**Step 1: build.rs Implementation**

```rust
// build.rs
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=wit/");
    
    // Detect WIT file changes
    let wit_dir = PathBuf::from("wit");
    if !wit_dir.exists() {
        panic!("WIT directory not found: {:?}", wit_dir);
    }
    
    // Generate bindings for all packages
    generate_bindings("wit/core", "src/bindings/core");
    generate_bindings("wit/ext/filesystem", "src/bindings/ext_filesystem");
    generate_bindings("wit/ext/network", "src/bindings/ext_network");
    generate_bindings("wit/ext/process", "src/bindings/ext_process");
    
    println!("cargo:warning=WIT bindings generated successfully");
}

fn generate_bindings(wit_path: &str, out_dir: &str) {
    let status = Command::new("wit-bindgen")
        .arg("rust")
        .arg("--out-dir")
        .arg(out_dir)
        .arg(wit_path)
        .status()
        .expect("Failed to execute wit-bindgen");
    
    if !status.success() {
        panic!("wit-bindgen failed for {}", wit_path);
    }
}
```

**Key Features:**
- Automatic regeneration when WIT files change
- Multi-package binding generation
- Clear error reporting
- Build warnings for visibility

**Step 2: Cargo.toml Configuration**

```toml
[package]
name = "airssys-wasm"
version = "1.0.0"
edition = "2021"

[dependencies]
# Workspace dependencies
airssys-osl = { workspace = true }
airssys-rt = { workspace = true }

# WASM runtime (from WASM-TASK-002)
wasmtime = { workspace = true, features = ["component-model"] }
wasmtime-wasi = { workspace = true }

# Core runtime
tokio = { workspace = true, features = ["full"] }
chrono = { workspace = true, features = ["serde"] }
serde = { workspace = true, features = ["derive"] }
thiserror = { workspace = true }

# Generated bindings support
wit-bindgen-rt = "0.30"  # Runtime support for generated bindings

[build-dependencies]
# No Rust dependencies - using CLI
```

**Note:** No wit-bindgen crate dependency - using CLI only

**Step 3: Generated Code Integration**

```rust
// src/bindings/mod.rs
//! Generated WIT bindings for airssys-wasm framework
//! 
//! DO NOT EDIT - Generated by wit-bindgen from WIT files

pub mod core {
    #![allow(clippy::all)]
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bindings/core/mod.rs"));
}

pub mod ext_filesystem {
    #![allow(clippy::all)]
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bindings/ext_filesystem/mod.rs"));
}

pub mod ext_network {
    #![allow(clippy::all)]
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bindings/ext_network/mod.rs"));
}

pub mod ext_process {
    #![allow(clippy::all)]
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bindings/ext_process/mod.rs"));
}
```

**Integration Features:**
- Generated code excluded from clippy checks
- Clear module organization
- Compile-time inclusion
- CI-friendly (generated code checked into git)

#### Multi-Package Binding Generation

**Challenge:** Generate bindings for 4 independent packages with different types.

**Solution:** Separate invocations with distinct output directories

**Directory Structure:**
```
src/bindings/
├── core/
│   ├── mod.rs
│   ├── types.rs
│   ├── capabilities.rs
│   ├── component_lifecycle.rs
│   └── host_services.rs
├── ext_filesystem/
│   ├── mod.rs
│   ├── types.rs
│   ├── filesystem.rs
│   └── metadata.rs
├── ext_network/
│   ├── mod.rs
│   ├── types.rs
│   ├── socket.rs
│   └── connection.rs
└── ext_process/
    ├── mod.rs
    ├── types.rs
    ├── lifecycle.rs
    └── signals.rs
```

**Namespace Management:**
- Each package gets dedicated module: `core::`, `ext_filesystem::`, `ext_network::`, `ext_process::`
- Type names automatically prefixed by module
- No name collisions (e.g., `core::types::ComponentId` vs. `ext_filesystem::types::Path`)

**Build Performance:**
- Estimated: ~2-3 seconds total generation time
- Incremental: Only regenerate on WIT file changes
- Parallel: Can generate packages in parallel (future optimization)

#### Validation Strategy

**Pre-Generation Validation:**
```bash
# Validate WIT files before binding generation
wasm-tools component wit wit/
# Expected: exit code 0
```

**Post-Generation Validation:**
```bash
# Verify generated bindings compile
cargo check

# Verify no clippy warnings in generated code
cargo clippy --all-targets --all-features
# Expected: 0 warnings (clippy::all disabled for generated code)
```

**Runtime Validation:**
```rust
#[cfg(test)]
mod binding_validation_tests {
    use super::*;
    
    #[test]
    fn test_core_types_exist() {
        use crate::bindings::core::types::*;
        
        // Verify ComponentId type exists and is usable
        let component_id = ComponentId {
            namespace: "test".into(),
            name: "component".into(),
            version: "1.0.0".into(),
        };
        
        assert_eq!(component_id.namespace, "test");
    }
    
    #[test]
    fn test_filesystem_types_exist() {
        use crate::bindings::ext_filesystem::types::*;
        
        // Verify filesystem types exist
        let _path: Path = "/test/path".into();
    }
    
    #[test]
    fn test_all_interfaces_generated() {
        // Verify all expected modules exist
        use crate::bindings::core::*;
        use crate::bindings::ext_filesystem::*;
        use crate::bindings::ext_network::*;
        use crate::bindings::ext_process::*;
        
        // If this compiles, all bindings were generated
    }
}
```

#### Deliverables

**Task 3.1 Complete When:**
- ✅ `build.rs` implemented and working
- ✅ Bindings generate automatically on `cargo build`
- ✅ All 4 packages generate bindings successfully
- ✅ Generated code compiles without errors
- ✅ Generated code checked into git
- ✅ Integration tests validate generated types
- ✅ Documentation explains binding generation process

---

## Permission System Implementation

### Phase 3 Task 3.2: Permission System Integration (Day 8, 6 hours)

**Objective:** Implement permission parsing from Component.toml and validation against WIT capability types.

#### Component.toml Permission Format

**Example Component.toml:**
```toml
[component]
name = "file-processor"
version = "1.0.0"
description = "Processes CSV files and generates reports"

[dependencies]
core = "1.0.0"
filesystem = "1.0.0"  # Component uses filesystem extension

[permissions]
# Filesystem permissions
filesystem = [
    { action = "read", path-pattern = "/data/input/**" },
    { action = "write", path-pattern = "/data/output/**" },
    { action = "list", path-pattern = "/data/input" }
]

# No network or process permissions needed
```

**Permission Schema:**
```toml
[permissions]
filesystem = [
    { action = "<filesystem-action>", path-pattern = "<glob-pattern>" }
]
network = [
    { action = "<network-action>", host-pattern = "<domain-pattern>", port = <u16> }
]
process = [
    { action = "<process-action>", command-pattern = "<command-glob>" }
]
```

#### Permission Data Structures

**Rust Types (matching WIT):**
```rust
// src/core/permissions.rs

use serde::{Deserialize, Serialize};

/// Component permission manifest from Component.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionManifest {
    pub filesystem: Vec<FilesystemPermission>,
    pub network: Vec<NetworkPermission>,
    pub process: Vec<ProcessPermission>,
}

/// Filesystem permission (matches WIT capabilities.wit)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemPermission {
    pub action: FilesystemAction,
    #[serde(rename = "path-pattern")]
    pub path_pattern: String,  // Glob pattern: /data/**, /config/*.json
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FilesystemAction {
    Read,
    Write,
    Delete,
    List,
}

/// Network permission (matches WIT capabilities.wit)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPermission {
    pub action: NetworkAction,
    #[serde(rename = "host-pattern")]
    pub host_pattern: String,  // Domain pattern: api.example.com, *.github.com
    pub port: Option<u16>,     // Optional port restriction
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkAction {
    Outbound,
    Inbound,
}

/// Process permission (matches WIT capabilities.wit)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessPermission {
    pub action: ProcessAction,
    #[serde(rename = "command-pattern")]
    pub command_pattern: String,  // Command glob: /usr/bin/python*, /bin/sh
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProcessAction {
    Spawn,
    Kill,
    Signal,
}

/// Permission validation result
#[derive(Debug, Clone)]
pub enum PermissionResult {
    Granted,
    Denied { reason: String },
}

impl PermissionResult {
    pub fn is_granted(&self) -> bool {
        matches!(self, PermissionResult::Granted)
    }
}
```

#### Component.toml Parser

**Parser Implementation:**
```rust
// src/core/component_config.rs

use std::path::Path;
use serde::Deserialize;
use crate::core::permissions::PermissionManifest;
use crate::core::error::WasmError;

/// Component configuration from Component.toml
#[derive(Debug, Clone, Deserialize)]
pub struct ComponentConfig {
    pub component: ComponentMetadata,
    pub dependencies: ComponentDependencies,
    pub permissions: PermissionManifest,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComponentMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComponentDependencies {
    pub core: String,  // "1.0.0"
    pub filesystem: Option<String>,
    pub network: Option<String>,
    pub process: Option<String>,
}

impl ComponentConfig {
    /// Parse Component.toml file
    pub fn from_file(path: &Path) -> Result<Self, WasmError> {
        let toml_content = std::fs::read_to_string(path)
            .map_err(|e| WasmError::InvalidConfiguration(format!("Failed to read Component.toml: {}", e)))?;
        
        Self::from_str(&toml_content)
    }
    
    /// Parse Component.toml from string
    pub fn from_str(content: &str) -> Result<Self, WasmError> {
        toml::from_str(content)
            .map_err(|e| WasmError::InvalidConfiguration(format!("Invalid Component.toml: {}", e)))
    }
    
    /// Validate permission patterns
    pub fn validate_permissions(&self) -> Result<(), WasmError> {
        // Validate filesystem patterns
        for perm in &self.permissions.filesystem {
            validate_glob_pattern(&perm.path_pattern)?;
        }
        
        // Validate network patterns
        for perm in &self.permissions.network {
            validate_domain_pattern(&perm.host_pattern)?;
            if let Some(port) = perm.port {
                if port == 0 {
                    return Err(WasmError::InvalidConfiguration("Invalid port: 0".into()));
                }
            }
        }
        
        // Validate process patterns
        for perm in &self.permissions.process {
            validate_command_pattern(&perm.command_pattern)?;
        }
        
        Ok(())
    }
}

/// Validate glob pattern syntax
fn validate_glob_pattern(pattern: &str) -> Result<(), WasmError> {
    if pattern.is_empty() {
        return Err(WasmError::InvalidConfiguration("Empty path pattern".into()));
    }
    
    // Basic validation: check for valid glob syntax
    if pattern.contains("***") {
        return Err(WasmError::InvalidConfiguration(format!("Invalid glob pattern: {}", pattern)));
    }
    
    Ok(())
}

/// Validate domain pattern syntax
fn validate_domain_pattern(pattern: &str) -> Result<(), WasmError> {
    if pattern.is_empty() {
        return Err(WasmError::InvalidConfiguration("Empty domain pattern".into()));
    }
    
    // Basic validation: check for valid domain characters
    if !pattern.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '*' || c == '-') {
        return Err(WasmError::InvalidConfiguration(format!("Invalid domain pattern: {}", pattern)));
    }
    
    Ok(())
}

/// Validate command pattern syntax
fn validate_command_pattern(pattern: &str) -> Result<(), WasmError> {
    if pattern.is_empty() {
        return Err(WasmError::InvalidConfiguration("Empty command pattern".into()));
    }
    
    // Basic validation: check for absolute path or glob
    if !pattern.starts_with('/') && !pattern.contains('*') {
        return Err(WasmError::InvalidConfiguration(format!("Command pattern must be absolute path or glob: {}", pattern)));
    }
    
    Ok(())
}
```

#### Permission Validation Logic

**Permission Matching:**
```rust
// src/core/permissions.rs

use glob::Pattern;

impl PermissionManifest {
    /// Check if requested permissions are subset of granted permissions
    pub fn validate_against(&self, granted: &PermissionManifest) -> PermissionResult {
        // Check filesystem permissions
        for requested in &self.filesystem {
            if !is_filesystem_permission_granted(requested, &granted.filesystem) {
                return PermissionResult::Denied {
                    reason: format!(
                        "Filesystem permission denied: {:?} on {}",
                        requested.action, requested.path_pattern
                    ),
                };
            }
        }
        
        // Check network permissions
        for requested in &self.network {
            if !is_network_permission_granted(requested, &granted.network) {
                return PermissionResult::Denied {
                    reason: format!(
                        "Network permission denied: {:?} to {}:{}",
                        requested.action, requested.host_pattern, requested.port.unwrap_or(0)
                    ),
                };
            }
        }
        
        // Check process permissions
        for requested in &self.process {
            if !is_process_permission_granted(requested, &granted.process) {
                return PermissionResult::Denied {
                    reason: format!(
                        "Process permission denied: {:?} on {}",
                        requested.action, requested.command_pattern
                    ),
                };
            }
        }
        
        PermissionResult::Granted
    }
}

fn is_filesystem_permission_granted(
    requested: &FilesystemPermission,
    granted: &[FilesystemPermission],
) -> bool {
    granted.iter().any(|g| {
        // Action must match
        if !matches!((requested.action, g.action),
            (FilesystemAction::Read, FilesystemAction::Read) |
            (FilesystemAction::Write, FilesystemAction::Write) |
            (FilesystemAction::Delete, FilesystemAction::Delete) |
            (FilesystemAction::List, FilesystemAction::List))
        {
            return false;
        }
        
        // Pattern must be subset of granted pattern
        is_pattern_subset(&requested.path_pattern, &g.path_pattern)
    })
}

fn is_network_permission_granted(
    requested: &NetworkPermission,
    granted: &[NetworkPermission],
) -> bool {
    granted.iter().any(|g| {
        // Action must match
        if !matches!((requested.action, g.action),
            (NetworkAction::Outbound, NetworkAction::Outbound) |
            (NetworkAction::Inbound, NetworkAction::Inbound))
        {
            return false;
        }
        
        // Host pattern must match
        if !is_domain_subset(&requested.host_pattern, &g.host_pattern) {
            return false;
        }
        
        // Port must match (if specified)
        match (requested.port, g.port) {
            (Some(r), Some(g)) => r == g,
            (Some(_), None) => true,  // Granted allows all ports
            (None, _) => true,        // Requested doesn't specify port
        }
    })
}

fn is_process_permission_granted(
    requested: &ProcessPermission,
    granted: &[ProcessPermission],
) -> bool {
    granted.iter().any(|g| {
        // Action must match
        if !matches!((requested.action, g.action),
            (ProcessAction::Spawn, ProcessAction::Spawn) |
            (ProcessAction::Kill, ProcessAction::Kill) |
            (ProcessAction::Signal, ProcessAction::Signal))
        {
            return false;
        }
        
        // Command pattern must be subset
        is_pattern_subset(&requested.command_pattern, &g.command_pattern)
    })
}

fn is_pattern_subset(requested: &str, granted: &str) -> bool {
    // Simple implementation: exact match or glob match
    if requested == granted {
        return true;
    }
    
    // If granted is wildcard pattern, check if requested matches
    if let Ok(pattern) = Pattern::new(granted) {
        return pattern.matches(requested);
    }
    
    false
}

fn is_domain_subset(requested: &str, granted: &str) -> bool {
    // Simple implementation: exact match or wildcard match
    if requested == granted {
        return true;
    }
    
    // If granted has wildcard, check match
    if granted.starts_with("*.") {
        let domain_suffix = &granted[2..];
        return requested.ends_with(domain_suffix) || requested == domain_suffix;
    }
    
    false
}
```

#### Integration Tests

**Permission Parsing Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_component_toml() {
        let toml = r#"
[component]
name = "test-component"
version = "1.0.0"
description = "Test component"

[dependencies]
core = "1.0.0"
filesystem = "1.0.0"

[permissions]
filesystem = [
    { action = "read", path-pattern = "/data/**" }
]
"#;
        
        let config = ComponentConfig::from_str(toml).unwrap();
        assert_eq!(config.component.name, "test-component");
        assert_eq!(config.permissions.filesystem.len(), 1);
        assert!(matches!(config.permissions.filesystem[0].action, FilesystemAction::Read));
    }
    
    #[test]
    fn test_permission_validation_granted() {
        let requested = PermissionManifest {
            filesystem: vec![
                FilesystemPermission {
                    action: FilesystemAction::Read,
                    path_pattern: "/data/input/file.txt".into(),
                }
            ],
            network: vec![],
            process: vec![],
        };
        
        let granted = PermissionManifest {
            filesystem: vec![
                FilesystemPermission {
                    action: FilesystemAction::Read,
                    path_pattern: "/data/**".into(),
                }
            ],
            network: vec![],
            process: vec![],
        };
        
        let result = requested.validate_against(&granted);
        assert!(result.is_granted());
    }
    
    #[test]
    fn test_permission_validation_denied() {
        let requested = PermissionManifest {
            filesystem: vec![
                FilesystemPermission {
                    action: FilesystemAction::Write,
                    path_pattern: "/etc/config".into(),
                }
            ],
            network: vec![],
            process: vec![],
        };
        
        let granted = PermissionManifest {
            filesystem: vec![
                FilesystemPermission {
                    action: FilesystemAction::Read,
                    path_pattern: "/data/**".into(),
                }
            ],
            network: vec![],
            process: vec![],
        };
        
        let result = requested.validate_against(&granted);
        assert!(!result.is_granted());
    }
    
    #[test]
    fn test_invalid_toml() {
        let toml = r#"
[component]
name = "test"
# Missing required fields
"#;
        
        let result = ComponentConfig::from_str(toml);
        assert!(result.is_err());
    }
}
```

#### Deliverables

**Task 3.2 Complete When:**
- ✅ Permission types implemented matching WIT
- ✅ Component.toml parser working
- ✅ Permission validation logic implemented
- ✅ Pattern matching (glob, domain, command) working
- ✅ >90% test coverage for permission logic
- ✅ Integration with component loading workflow
- ✅ Clear error messages for permission violations

---

## End-to-End Validation

### Phase 3 Task 3.3: End-to-End Validation (Day 9, 6 hours)

**Objective:** Validate complete workflow from WIT files → generated bindings → component loading → permission validation.

#### Complete Integration Test

**End-to-End Workflow Test:**
```rust
// tests/complete_wit_system_test.rs

use airssys_wasm::bindings::core::*;
use airssys_wasm::core::component_config::ComponentConfig;
use airssys_wasm::core::permissions::PermissionManifest;

#[test]
fn test_complete_component_loading_workflow() {
    // Step 1: Parse Component.toml
    let toml = r#"
[component]
name = "file-processor"
version = "1.0.0"
description = "CSV processor"

[dependencies]
core = "1.0.0"
filesystem = "1.0.0"

[permissions]
filesystem = [
    { action = "read", path-pattern = "/data/input/**" },
    { action = "write", path-pattern = "/data/output/**" }
]
"#;
    
    let config = ComponentConfig::from_str(toml)
        .expect("Failed to parse Component.toml");
    
    // Step 2: Validate permission syntax
    config.validate_permissions()
        .expect("Permission validation failed");
    
    // Step 3: Create security policy (host side)
    let granted = PermissionManifest {
        filesystem: vec![
            FilesystemPermission {
                action: FilesystemAction::Read,
                path_pattern: "/data/**".into(),
            },
            FilesystemPermission {
                action: FilesystemAction::Write,
                path_pattern: "/data/output/**".into(),
            },
        ],
        network: vec![],
        process: vec![],
    };
    
    // Step 4: Validate requested permissions against granted
    let result = config.permissions.validate_against(&granted);
    assert!(result.is_granted(), "Permission check failed");
    
    // Step 5: Verify generated bindings types match config types
    use airssys_wasm::bindings::core::types::ComponentId;
    
    let component_id = ComponentId {
        namespace: "test".into(),
        name: config.component.name.clone(),
        version: config.component.version.clone(),
    };
    
    assert_eq!(component_id.name, "file-processor");
    assert_eq!(component_id.version, "1.0.0");
}

#[test]
fn test_permission_denial_workflow() {
    // Component requests write permission to /etc
    let config_toml = r#"
[component]
name = "malicious-component"
version = "1.0.0"
description = "Tries to write to /etc"

[dependencies]
core = "1.0.0"
filesystem = "1.0.0"

[permissions]
filesystem = [
    { action = "write", path-pattern = "/etc/**" }
]
"#;
    
    let config = ComponentConfig::from_str(config_toml).unwrap();
    
    // Host only grants /data access
    let granted = PermissionManifest {
        filesystem: vec![
            FilesystemPermission {
                action: FilesystemAction::Read,
                path_pattern: "/data/**".into(),
            },
        ],
        network: vec![],
        process: vec![],
    };
    
    // Validation should deny
    let result = config.permissions.validate_against(&granted);
    assert!(!result.is_granted());
    
    if let PermissionResult::Denied { reason } = result {
        assert!(reason.contains("write"));
        assert!(reason.contains("/etc/**"));
    }
}

#[test]
fn test_generated_bindings_type_safety() {
    // Verify generated types are type-safe and usable
    use airssys_wasm::bindings::core::types::*;
    use airssys_wasm::bindings::core::capabilities::*;
    
    // Create filesystem permission using generated types
    let perm = FilesystemPermission {
        action: FilesystemAction::Read,
        path_pattern: "/data/test.txt".into(),
    };
    
    // Verify enum variant
    assert!(matches!(perm.action, FilesystemAction::Read));
    
    // Create component ID
    let id = ComponentId {
        namespace: "airssys".into(),
        name: "test-component".into(),
        version: "1.0.0".into(),
    };
    
    assert_eq!(id.namespace, "airssys");
}
```

#### Build System Validation

**Verify Automatic Regeneration:**
```bash
#!/bin/bash
# tests/validate_build_system.sh

set -e

echo "=== Build System Validation ==="

# Step 1: Clean build
echo "Clean build..."
cargo clean
rm -rf src/bindings/

# Step 2: Build (should trigger binding generation)
echo "Building (first time - should generate bindings)..."
cargo build 2>&1 | grep -q "WIT bindings generated successfully"

# Step 3: Verify bindings exist
echo "Verifying generated bindings..."
test -d src/bindings/core || (echo "Core bindings not generated" && exit 1)
test -d src/bindings/ext_filesystem || (echo "Filesystem bindings not generated" && exit 1)
test -d src/bindings/ext_network || (echo "Network bindings not generated" && exit 1)
test -d src/bindings/ext_process || (echo "Process bindings not generated" && exit 1)

# Step 4: Rebuild (should be no-op)
echo "Rebuilding (should skip generation)..."
cargo build 2>&1

# Step 5: Touch WIT file (should trigger regeneration)
echo "Touching WIT file..."
touch wit/core/types.wit

echo "Rebuilding (should regenerate bindings)..."
cargo build 2>&1 | grep -q "WIT bindings generated successfully"

echo "✅ Build system validation PASSED"
```

#### Documentation Completion

**Phase 3 Documentation:**

**1. Build System Integration Guide** (`docs/src/build/wit-bindgen-integration.md`)
- How binding generation works
- build.rs implementation details
- Troubleshooting binding generation issues

**2. Permission System Guide** (`docs/src/permissions/permission-system-integration.md`)
- Component.toml permission format
- Permission validation logic
- Security policy examples
- Common permission patterns

**3. Phase 3 Completion Report** (`docs/src/wit/phase-3-completion-report.md`)
- All deliverables summary
- Validation results
- Performance measurements
- Known issues and limitations

#### Final Validation Checklist

**Code Quality:**
- ✅ `cargo check` - No errors
- ✅ `cargo clippy --all-targets --all-features` - Zero warnings
- ✅ `cargo test` - All tests passing
- ✅ `cargo build --release` - Release build successful
- ✅ `cargo doc --no-deps` - Documentation builds

**Functional Validation:**
- ✅ WIT files validate with wasm-tools
- ✅ Bindings generate automatically on build
- ✅ Generated code compiles without errors
- ✅ Permission parsing works correctly
- ✅ Permission validation logic correct
- ✅ End-to-end tests demonstrate complete workflow

**Documentation Validation:**
- ✅ All Phase 3 documentation complete
- ✅ Build system integration guide
- ✅ Permission system guide
- ✅ API reference documentation
- ✅ Troubleshooting guides

#### Deliverables

**Task 3.3 Complete When:**
- ✅ Complete integration test suite passing
- ✅ Build system validation passing
- ✅ All code quality checks passing
- ✅ Documentation complete and accurate
- ✅ Phase 3 completion report documented
- ✅ Memory bank updated with completion status
- ✅ Ready for WASM-TASK-004 (Actor System Integration)

---

## Rust Code Organization

### Generated Bindings Structure

**Target Directory Layout:**
```
src/
├── bindings/               ← Generated from WIT (DO NOT EDIT)
│   ├── mod.rs             ← Integration module
│   ├── core/
│   │   ├── mod.rs
│   │   ├── types.rs       ← Generated from core/types.wit
│   │   ├── capabilities.rs ← Generated from core/capabilities.wit
│   │   ├── component_lifecycle.rs
│   │   └── host_services.rs
│   ├── ext_filesystem/
│   │   ├── mod.rs
│   │   ├── types.rs       ← Generated from ext/filesystem/types.wit
│   │   ├── filesystem.rs
│   │   └── metadata.rs
│   ├── ext_network/
│   │   ├── mod.rs
│   │   ├── types.rs
│   │   ├── socket.rs
│   │   └── connection.rs
│   └── ext_process/
│       ├── mod.rs
│       ├── types.rs
│       ├── lifecycle.rs
│       └── signals.rs
├── core/                   ← Core implementation (hand-written)
│   ├── mod.rs
│   ├── component_config.rs ← Component.toml parser
│   ├── permissions.rs     ← Permission validation
│   ├── error.rs           ← Error types (from WASM-TASK-000)
│   ├── component.rs       ← Component abstractions
│   └── ...
├── runtime/                ← WASM runtime (from WASM-TASK-002)
│   ├── mod.rs
│   ├── engine.rs
│   ├── limits.rs
│   └── ...
└── lib.rs
```

### Integration Between Generated and Hand-Written Code

**Generated Bindings → Core Implementation:**
```rust
// src/core/component_config.rs
use crate::bindings::core::capabilities::{
    FilesystemPermission as WitFilesystemPermission,
    FilesystemAction as WitFilesystemAction,
};

// Convert from TOML permission to WIT permission type
impl From<permissions::FilesystemPermission> for WitFilesystemPermission {
    fn from(perm: permissions::FilesystemPermission) -> Self {
        WitFilesystemPermission {
            action: match perm.action {
                permissions::FilesystemAction::Read => WitFilesystemAction::Read,
                permissions::FilesystemAction::Write => WitFilesystemAction::Write,
                permissions::FilesystemAction::Delete => WitFilesystemAction::Delete,
                permissions::FilesystemAction::List => WitFilesystemAction::List,
            },
            path_pattern: perm.path_pattern,
        }
    }
}
```

**Core Implementation → Runtime Integration:**
```rust
// src/runtime/component_loader.rs
use crate::core::component_config::ComponentConfig;
use crate::core::permissions::PermissionManifest;
use crate::runtime::engine::WasmEngine;

pub struct ComponentLoader {
    engine: WasmEngine,
}

impl ComponentLoader {
    pub fn load_component(&self, wasm_path: &Path, config_path: &Path) -> Result<LoadedComponent, WasmError> {
        // Parse Component.toml
        let config = ComponentConfig::from_file(config_path)?;
        
        // Validate permissions
        config.validate_permissions()?;
        
        // Load WASM binary
        let wasm_bytes = std::fs::read(wasm_path)?;
        let component = self.engine.load_component(&wasm_bytes)?;
        
        // Return loaded component with permissions
        Ok(LoadedComponent {
            component,
            config,
        })
    }
}
```

---

## Known Constraints

### From WIT Architecture

**1. Component Model v0.1 Limitations**
- ❌ No cross-package type imports
- ✅ Workaround: Extension packages have independent types.wit
- ✅ Future: Component Model v0.2 will support cross-package imports (DEBT-WASM-003)

**2. wit-bindgen CLI Requirements**
- Requires wit-bindgen-cli installed (`cargo install wit-bindgen-cli`)
- Version >=0.30.0 for Component Model v0.1 support
- Build fails if wit-bindgen not in PATH

**3. Generated Code Size**
- Estimated: ~3,000-4,000 lines of generated Rust code
- All generated code must be checked into git (CI requirement)
- Adds ~50-100KB to repository size

**4. Build Performance Overhead**
- ~2-3 seconds for complete binding generation
- Incremental builds skip generation if WIT unchanged
- CI builds always regenerate to ensure consistency

**5. Permission Pattern Matching Complexity**
- Glob patterns require glob crate dependency
- Domain wildcards need careful security review
- Command pattern matching platform-specific

**6. Type Name Collisions**
- Multiple packages have `types.wit`
- Namespace management critical: `core::types::` vs. `ext_filesystem::types::`
- Generated code must not conflict

---

## Success Criteria

### Phase 3 Complete When

**✅ Build System Integration:**
- build.rs implemented and working
- Bindings generate automatically on cargo build
- All 4 packages generate bindings successfully
- Generated code compiles without errors
- Incremental builds work correctly
- Build warnings clear and informative

**✅ Permission System:**
- Component.toml parser working correctly
- Permission validation logic implemented
- Pattern matching (glob, domain, command) functional
- >90% test coverage for permission logic
- Clear error messages for violations
- Integration with component loading workflow

**✅ End-to-End Validation:**
- Complete integration test suite passing
- Build system validation script passing
- All code quality checks passing (check, clippy, test, doc)
- Documentation complete and accurate
- Phase 3 completion report documented

**✅ Quality Standards:**
- Zero compiler warnings
- Zero clippy warnings (generated code excluded)
- All tests passing (unit + integration)
- 100% rustdoc coverage for hand-written code
- Professional documentation quality

---

## Timeline Estimates

**Task 3.1: wit-bindgen Build Configuration** - 6 hours (Day 7)
- Build.rs implementation: 2 hours
- Cargo.toml configuration: 1 hour
- Generated code integration: 1 hour
- Validation and testing: 1 hour
- Documentation: 1 hour

**Task 3.2: Permission System Integration** - 6 hours (Day 8)
- Permission data structures: 1.5 hours
- Component.toml parser: 1.5 hours
- Permission validation logic: 2 hours
- Integration tests: 1 hour

**Task 3.3: End-to-End Validation** - 6 hours (Day 9)
- Integration test suite: 2 hours
- Build system validation: 1 hour
- Documentation completion: 2 hours
- Final validation and reporting: 1 hour

**Phase 3 Total: 18 hours (3 days)**

---

## References

### Phase 2 Deliverables
- **WIT System Architecture:** `docs/src/reference/wit-system-architecture.md`
- **Complete WIT Files:** `wit/` directory (13 files, 1,627 lines)
- **Validation Results:** All packages validate with exit code 0

### Research Documents
- **wit-bindgen Integration Guide:** `docs/src/wit/research/wit_bindgen_integration_guide.md`
- **Build System Strategy:** `tasks/task_003_phase_3_build_system_strategy.md`
- **Cargo Configuration:** `tasks/task_003_phase_3_cargo_configuration.md`

### External References
- [wit-bindgen Documentation](https://github.com/bytecodealliance/wit-bindgen)
- [Component Model Specification](https://github.com/WebAssembly/component-model)
- [wasm-tools Documentation](https://github.com/bytecodealliance/wasm-tools)

---

**Document Version:** 1.0.0  
**Last Updated:** 2025-10-26  
**Author:** AI Agent (WASM-TASK-003 Phase 2 Task 2.3)  
**Status:** Ready for Execution  
**Prerequisites:** ✅ Phase 2 Complete - 13 WIT files validated
