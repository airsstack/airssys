# WASM-TASK-003: Block 2 - WIT Interface System
## Comprehensive Implementation Plans

**Task ID:** WASM-TASK-003  
**Status:** not-started  
**Created:** 2025-10-24  
**Duration:** 3-4 weeks (6 phases)  
**Priority:** Critical Path - Foundation Layer

---

## Executive Summary

This document provides comprehensive, actionable implementation plans for WASM-TASK-003 (Block 2: WIT Interface System), building on the completed Block 1 (WASM Runtime Layer). The WIT interface system establishes the type-safe contract between WASM components and the host runtime, enabling language-agnostic component development with fine-grained capability-based security.

**Task Context:**
- **Duration**: 3-4 weeks (6 phases)
- **Dependencies**: ✅ WASM-TASK-002 (Block 1) complete - 288 tests passing, zero warnings
- **Status**: Ready to begin - all prerequisites met
- **Priority**: Critical Path - Foundation Layer

**Key Objectives:**
1. Design and implement comprehensive WIT interfaces for all host services
2. Establish capability permission system with annotations
3. Integrate wit-bindgen for Rust binding generation
4. Implement interface validation and versioning
5. Create complete documentation and examples

---

## 1. Phase-by-Phase Implementation Plan

### Phase 1: WIT Interface Design and Structure (Week 1)

#### Overview
Establish the foundational WIT interface structure, directory organization, and core interface definitions that all components will implement.

---

#### Task 1.1: WIT Project Structure Setup
**Duration**: Day 1 (6 hours)

**Deliverables:**
- Create `airssys-wasm/wit/` directory structure
- Setup package naming conventions
- Define world files organization
- Establish version control strategy for WIT files

**Directory Structure to Create:**
```
airssys-wasm/
├── wit/
│   ├── core/                          # Required base interfaces
│   │   ├── component.wit              # Universal component lifecycle
│   │   ├── host.wit                   # Core host services
│   │   ├── types.wit                  # Common types and errors
│   │   └── capabilities.wit           # Capability types
│   ├── extensions/                     # Optional domain extensions (Phase 3)
│   │   ├── filesystem.wit
│   │   ├── network.wit
│   │   └── process.wit
│   ├── examples/                       # Reference implementations
│   │   └── basic-component.wit
│   └── README.md                       # WIT documentation index
```

**Implementation Steps:**
1. Create directory structure as above
2. Add `wit/README.md` documenting:
   - Package naming conventions (`airssys:component-core@1.0.0`)
   - Interface organization philosophy
   - Version strategy (semantic versioning)
   - Integration with wit-bindgen
3. Document WIT file structure standards
4. Setup git tracking for WIT files

**Success Criteria:**
- ✅ Directory structure created with proper organization
- ✅ README.md documents conventions and patterns
- ✅ Package naming follows `airssys:{category}-{type}@{version}` format
- ✅ Version control strategy documented (WIT files tracked in git)

**Testing:**
```bash
# Verify directory structure
ls -R airssys-wasm/wit/

# Validate structure matches standard
```

---

#### Task 1.2: Core Host Service Interface Definitions
**Duration**: Days 2-3 (12 hours)

**Deliverables:**
- `core/types.wit` - Common types, errors, component metadata
- `core/capabilities.wit` - Capability types and permission structures
- `core/host.wit` - Host services interface (logging, messaging, time, introspection)
- Interface design documentation

---

**File: `wit/core/types.wit`**
```wit
package airssys:component-core@1.0.0;

/// Component unique identifier
type component-id = string;

/// Request identifier for tracking async operations
type request-id = string;

/// Common error types for operations
variant component-error {
    initialization-failed(string),
    invalid-configuration(string),
    resource-exhausted(string),
    internal-error(string),
}

variant execution-error {
    invalid-input(string),
    timeout-exceeded,
    capability-denied(string),
    internal-error(string),
}

variant messaging-error {
    component-not-found(component-id),
    message-too-large,
    timeout-exceeded,
    delivery-failed(string),
}

variant file-error {
    not-found(string),
    permission-denied(string),
    io-error(string),
}

variant http-error {
    invalid-url(string),
    connection-failed(string),
    timeout-exceeded,
    permission-denied(string),
}

/// Execution context for operations
record execution-context {
    request-id: request-id,
    timeout-ms: u64,
}

/// Component metadata returned by metadata() function
record component-metadata {
    // Identity
    name: string,
    version: string,
    description: string,
    author: string,
    license: string,
    
    // Runtime requirements
    requested-permissions: requested-permissions,
    supported-operations: list<string>,
    
    // Runtime characteristics
    language: string,
    memory-requirements: memory-requirements,
    timeout-ms: option<u64>,
    stateful: bool,
    
    // API version
    api-version: string,
    
    // Discovery metadata
    homepage: option<string>,
    repository: option<string>,
    tags: list<string>,
}

record memory-requirements {
    min-memory-bytes: u64,
    max-memory-bytes: u64,
    preferred-memory-bytes: u64,
}

/// Health status for monitoring
record health-status {
    healthy: bool,
    reason: option<string>,
}

/// Logging levels
enum log-level {
    trace,
    debug,
    info,
    warn,
    error,
}

/// Component configuration during initialization
record component-config {
    /// Environment variables specific to this component
    env: list<tuple<string, string>>,
    
    /// Configuration data from Component.toml
    config-data: option<list<u8>>,
}

/// Requested permissions for host capabilities
record requested-permissions {
    filesystem: list<filesystem-permission>,
    network: list<network-permission>,
    storage: list<storage-permission>,
}
```

---

**File: `wit/core/capabilities.wit`**
```wit
package airssys:component-core@1.0.0;

/// Filesystem permission declaration
record filesystem-permission {
    action: filesystem-action,
    path-pattern: string,  // Glob pattern (e.g., "/data/**", "/output/*.txt")
}

enum filesystem-action {
    read,
    write,
    delete,
    list,
}

/// Network permission declaration
record network-permission {
    action: network-action,
    host-pattern: string,  // Wildcard pattern (e.g., "api.example.com", "*.github.com")
    port: option<u16>,
}

enum network-action {
    outbound,
    inbound,
}

/// Storage permission declaration
record storage-permission {
    namespace-pattern: string,  // e.g., "myapp:config", "shared:*"
    action: storage-action,
}

enum storage-action {
    read,
    write,
    delete,
    list,
}
```

---

**File: `wit/core/host.wit`**
```wit
package airssys:host-core@1.0.0;

use airssys:component-core/types.{
    component-id, 
    request-id, 
    component-metadata, 
    component-error, 
    messaging-error, 
    log-level
};

/// Core services always available to ALL components
interface host-services {
    /// Structured logging with context
    /// 
    /// Always available - no capability required
    log: func(
        level: log-level,
        message: string,
        context: option<list<tuple<string, string>>>
    );
    
    /// Send one-way message to another component (fire-and-forget)
    /// 
    /// Delivery: Best-effort async delivery, no response expected
    /// Use case: Event notifications, status updates
    /// Permission: Requires messaging capability
    send-message: func(
        target: component-id,
        message: list<u8>  // Encoded message payload
    ) -> result<_, messaging-error>;
    
    /// Send request expecting callback response (request-response pattern)
    /// 
    /// Flow: Host delivers request to target component via handle-message,
    ///       target responds via host, host routes response to handle-callback
    /// Use case: RPC calls, data queries
    /// Permission: Requires messaging capability
    send-request: func(
        target: component-id,
        request: list<u8>,  // Encoded request payload
        timeout-ms: u64
    ) -> result<request-id, messaging-error>;
    
    /// Cancel pending request before timeout
    /// 
    /// Best-effort cancellation - may still receive callback if already processed
    cancel-request: func(
        request-id: request-id
    ) -> result<_, messaging-error>;
    
    /// Time and timing services
    /// 
    /// Always available - no capability required
    current-time-millis: func() -> u64;
    sleep-millis: func(duration: u64);
    
    /// Component introspection
    /// 
    /// Always available - no capability required
    list-components: func() -> list<component-id>;
    get-component-metadata: func(id: component-id) -> result<component-metadata, component-error>;
}
```

---

**Implementation Steps:**
1. Create `wit/core/types.wit` with common types
2. Create `wit/core/capabilities.wit` with permission structures
3. Create `wit/core/host.wit` with host service interface
4. Document each interface with comprehensive comments
5. Add examples for each type and function

**Success Criteria:**
- ✅ All core types defined with proper WIT syntax
- ✅ Capability structures match ADR-WASM-005 security model
- ✅ Host services interface complete with all functions
- ✅ Every type/function has documentation comments
- ✅ Zero WIT syntax errors (validate with `wasm-tools component wit`)

**Testing:**
```bash
# Validate WIT syntax
wasm-tools component wit airssys-wasm/wit/core/

# Expected output: No errors, lists interfaces and types
```

---

#### Task 1.3: Component Contract Interface
**Duration**: Days 4-5 (10 hours)

**Deliverables:**
- `core/component.wit` - Component lifecycle interface (exports)
- Component world definition (what components must export/import)
- Contract documentation and examples

---

**File: `wit/core/component.wit`**
```wit
package airssys:component-core@1.0.0;

use types.{
    component-config, 
    component-error, 
    execution-context, 
    execution-error, 
    messaging-error, 
    component-metadata, 
    health-status, 
    component-id
};

/// Universal component lifecycle interface - ALL components MUST implement
interface component-lifecycle {
    /// Initialize component with configuration
    /// 
    /// Called once during component loading after instantiation
    /// Must complete successfully for component to become active
    /// Use case: Setup resources, validate config, prepare state
    init: func(config: component-config) -> result<_, component-error>;
    
    /// Handle external RPC requests (synchronous request-response)
    /// 
    /// Flow: External entity → Host RPC gateway → Component execute()
    /// Data: Self-describing with multicodec prefix
    /// Use case: HTTP API endpoints, gRPC services, CLI commands
    /// Timeout: Must complete within execution-context.timeout-ms
    execute: func(
        operation: list<u8>,  // Encoded operation data
        context: execution-context
    ) -> result<list<u8>, execution-error>;
    
    /// Handle internal component messages (asynchronous push delivery)
    /// 
    /// Flow: Component A → send-message() → Host → Component B handle-message()
    /// Delivery: Host pushes messages when they arrive (actor model)
    /// No polling: Components don't poll for messages
    /// Use case: Inter-component events, notifications, pub-sub
    handle-message: func(
        sender: component-id,
        message: list<u8>  // Encoded message
    ) -> result<_, messaging-error>;
    
    /// Handle callback responses from request-response operations
    /// 
    /// Flow: Component A → send-request() → Host → Component B handle-message()
    ///       → Component B logic → Host → Component A handle-callback()
    /// Optional: Only implement if component uses send-request
    /// Use case: Async RPC responses, distributed queries
    handle-callback: func(
        request-id: string,
        result: result<list<u8>, string>
    ) -> result<_, messaging-error>;
    
    /// Component metadata and capabilities
    /// 
    /// Called during component discovery and introspection
    /// Returns static metadata about component capabilities
    /// Always available - can be called before init()
    metadata: func() -> component-metadata;
    
    /// Health check for monitoring
    /// 
    /// Called periodically by host for health monitoring
    /// Should be lightweight and fast (<10ms target)
    /// Use case: Load balancers, orchestrators, monitoring systems
    health: func() -> health-status;
    
    /// Graceful shutdown and cleanup
    /// 
    /// Called before component unload or system shutdown
    /// Should cleanup resources, flush state, close connections
    /// Timeout: Must complete within configured shutdown timeout
    shutdown: func() -> result<_, component-error>;
}

/// Component world definition - what ALL components import/export
world component {
    /// Components MUST export lifecycle interface
    export component-lifecycle;
    
    /// Components MUST import host services
    import airssys:host-core/host-services.{host-services};
}
```

---

**Implementation Steps:**
1. Create `wit/core/component.wit` with lifecycle interface
2. Define component world with required imports/exports
3. Document each lifecycle method with:
   - Purpose and use case
   - When it's called
   - Expected behavior
   - Performance expectations
4. Add examples of typical implementations
5. Validate world definition compiles

**Success Criteria:**
- ✅ Component lifecycle interface complete with all 7 methods
- ✅ Component world defines required imports/exports
- ✅ Every method thoroughly documented
- ✅ Actor model integration clear (handle-message, handle-callback)
- ✅ WIT syntax validates successfully

**Testing:**
```bash
# Validate component world definition
wasm-tools component wit airssys-wasm/wit/core/component.wit

# Should output world definition without errors
```

---

### Phase 2: Capability Permission System (Week 1-2)

#### Overview
Design and implement the capability permission annotation system that allows components to declare required permissions and host functions to specify permission requirements.

---

#### Task 2.1: Capability Annotation Design
**Duration**: Days 6-7 (8 hours)

**Deliverables:**
- Component.toml permission declaration schema
- Permission pattern specification (glob for filesystem, wildcards for network)
- Annotation validation rules
- Permission documentation with examples

**Component.toml Permission Schema:**
```toml
# Component manifest with capability declarations
[component]
name = "data-processor"
version = "1.0.0"
api-version = "1.0.0"

# Filesystem permissions with glob patterns
[capabilities.filesystem]
read = [
    "/etc/myapp/config.toml",    # Exact file
    "/var/data/input/**",         # Recursive glob
    "/tmp/cache/*.json"           # Wildcard glob
]
write = [
    "/var/data/output/**",
    "/tmp/myapp/cache/*"
]

# Network permissions with domain wildcards
[[capabilities.network]]
action = "outbound"
host = "api.example.com"
port = 443

[[capabilities.network]]
action = "outbound"
host = "*.cdn.example.com"        # Wildcard domain
port = 443

# Storage permissions with namespace patterns
[capabilities.storage]
namespaces = [
    "myapp:config",               # Exact namespace
    "myapp:cache",
    "shared:*"                    # Wildcard namespace
]
max-size = "100MB"
```

**Permission Pattern Specification:**

**Filesystem Patterns (Glob Syntax):**
- `/path/to/file.txt` - Exact file match
- `/dir/*` - All files in directory (non-recursive)
- `/dir/**` - All files in directory tree (recursive)
- `/dir/*.{txt,json}` - Multiple extensions
- `/dir/**/config.toml` - Named file anywhere in tree

**Network Patterns (Domain Wildcards):**
- `api.example.com` - Exact domain
- `*.example.com` - All subdomains
- `*.github.com:443` - With port specification
- `10.0.0.*` - IP wildcard (use cautiously)

**Storage Patterns (Namespace Matching):**
- `myapp:config` - Exact namespace
- `myapp:*` - All namespaces with prefix
- `shared:*` - Shared namespace wildcard

**Validation Rules:**
1. All patterns must be well-formed (validated at component load)
2. Filesystem patterns must be absolute paths (no relative paths)
3. Network patterns must be valid domains or IPs
4. Storage namespaces must follow `category:subcategory` format
5. Wildcard usage must be explicit (no implicit wildcards)

**Implementation Steps:**
1. Define Component.toml schema structure
2. Document permission pattern syntax with examples
3. Specify validation rules for each permission type
4. Create permission declaration examples for common use cases
5. Document security implications of wildcards

**Success Criteria:**
- ✅ Component.toml schema clear and complete
- ✅ Pattern syntax documented with examples
- ✅ Validation rules specified
- ✅ Security implications documented
- ✅ Common use case examples provided

---

#### Task 2.2: Permission Parsing and Validation
**Duration**: Days 8-10 (12 hours)

**Deliverables:**
- Component.toml parser for permission declarations
- Pattern matching implementation (glob, wildcard)
- Permission validation logic
- Unit tests for parsing and matching

**New Rust Module: `src/core/permissions.rs`**

**Module Structure:**
```rust
// src/core/permissions.rs

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Permission-related errors
#[derive(Debug, Error)]
pub enum PermissionError {
    #[error("Invalid filesystem pattern: {0}")]
    InvalidFilesystemPattern(String),
    
    #[error("Invalid network pattern: {0}")]
    InvalidNetworkPattern(String),
    
    #[error("Invalid storage namespace: {0}")]
    InvalidStorageNamespace(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

/// Filesystem permission declaration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FilesystemPermission {
    pub action: FilesystemAction,
    pub path_pattern: String,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FilesystemAction {
    Read,
    Write,
    Delete,
    List,
}

/// Network permission declaration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkPermission {
    pub action: NetworkAction,
    pub host_pattern: String,
    pub port: Option<u16>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkAction {
    Outbound,
    Inbound,
}

/// Storage permission declaration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StoragePermission {
    pub namespace_pattern: String,
    pub action: StorageAction,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageAction {
    Read,
    Write,
    Delete,
    List,
}

/// Complete capability manifest from Component.toml
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CapabilityManifest {
    #[serde(default)]
    pub filesystem: FilesystemCapabilities,
    
    #[serde(default)]
    pub network: Vec<NetworkPermission>,
    
    #[serde(default)]
    pub storage: StorageCapabilities,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct FilesystemCapabilities {
    #[serde(default)]
    pub read: Vec<String>,
    
    #[serde(default)]
    pub write: Vec<String>,
    
    #[serde(default)]
    pub delete: Vec<String>,
    
    #[serde(default)]
    pub list: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct StorageCapabilities {
    #[serde(default)]
    pub namespaces: Vec<String>,
    
    pub max_size: Option<String>,
}

/// Permission validator
pub struct PermissionValidator {
    manifest: CapabilityManifest,
}

impl PermissionValidator {
    pub fn new(manifest: CapabilityManifest) -> Result<Self, PermissionError> {
        // Validate all patterns at construction
        Self::validate_manifest(&manifest)?;
        Ok(Self { manifest })
    }
    
    /// Validate entire manifest
    fn validate_manifest(manifest: &CapabilityManifest) -> Result<(), PermissionError> {
        // Validate filesystem patterns
        for pattern in &manifest.filesystem.read {
            Self::validate_filesystem_pattern(pattern)?;
        }
        for pattern in &manifest.filesystem.write {
            Self::validate_filesystem_pattern(pattern)?;
        }
        for pattern in &manifest.filesystem.delete {
            Self::validate_filesystem_pattern(pattern)?;
        }
        for pattern in &manifest.filesystem.list {
            Self::validate_filesystem_pattern(pattern)?;
        }
        
        // Validate network patterns
        for perm in &manifest.network {
            Self::validate_network_pattern(&perm.host_pattern)?;
        }
        
        // Validate storage namespaces
        for namespace in &manifest.storage.namespaces {
            Self::validate_storage_namespace(namespace)?;
        }
        
        Ok(())
    }
    
    /// Validate filesystem pattern (glob syntax)
    fn validate_filesystem_pattern(pattern: &str) -> Result<(), PermissionError> {
        // Must be absolute path
        if !pattern.starts_with('/') {
            return Err(PermissionError::InvalidFilesystemPattern(
                format!("Pattern must be absolute path: {}", pattern)
            ));
        }
        
        // Validate glob pattern compiles
        glob::Pattern::new(pattern)
            .map_err(|e| PermissionError::InvalidFilesystemPattern(
                format!("Invalid glob pattern '{}': {}", pattern, e)
            ))?;
        
        Ok(())
    }
    
    /// Validate network pattern (domain wildcards)
    fn validate_network_pattern(pattern: &str) -> Result<(), PermissionError> {
        // Basic validation - must not be empty
        if pattern.is_empty() {
            return Err(PermissionError::InvalidNetworkPattern(
                "Network pattern cannot be empty".to_string()
            ));
        }
        
        // Wildcard must be at start of domain segment
        if pattern.contains('*') && !pattern.starts_with("*.") {
            return Err(PermissionError::InvalidNetworkPattern(
                format!("Wildcard must be at start of domain segment: {}", pattern)
            ));
        }
        
        Ok(())
    }
    
    /// Validate storage namespace pattern
    fn validate_storage_namespace(namespace: &str) -> Result<(), PermissionError> {
        // Must follow category:subcategory format
        if !namespace.contains(':') {
            return Err(PermissionError::InvalidStorageNamespace(
                format!("Namespace must follow 'category:subcategory' format: {}", namespace)
            ));
        }
        
        Ok(())
    }
    
    /// Check if filesystem access is permitted
    pub fn check_filesystem_access(
        &self,
        path: &str,
        action: FilesystemAction,
    ) -> Result<(), PermissionError> {
        let patterns = match action {
            FilesystemAction::Read => &self.manifest.filesystem.read,
            FilesystemAction::Write => &self.manifest.filesystem.write,
            FilesystemAction::Delete => &self.manifest.filesystem.delete,
            FilesystemAction::List => &self.manifest.filesystem.list,
        };
        
        for pattern in patterns {
            if glob::Pattern::new(pattern)
                .expect("validated pattern")
                .matches(path)
            {
                return Ok(());
            }
        }
        
        Err(PermissionError::PermissionDenied(
            format!("Filesystem {:?} access denied for path: {}", action, path)
        ))
    }
    
    /// Check if network access is permitted
    pub fn check_network_access(
        &self,
        host: &str,
        port: u16,
        action: NetworkAction,
    ) -> Result<(), PermissionError> {
        for perm in &self.manifest.network {
            if perm.action as u8 != action as u8 {
                continue;
            }
            
            // Check port match
            if let Some(allowed_port) = perm.port {
                if allowed_port != port {
                    continue;
                }
            }
            
            // Check host match with wildcard support
            if Self::matches_domain_pattern(&perm.host_pattern, host) {
                return Ok(());
            }
        }
        
        Err(PermissionError::PermissionDenied(
            format!("Network {:?} access denied for {}:{}", action, host, port)
        ))
    }
    
    /// Check if storage access is permitted
    pub fn check_storage_access(
        &self,
        namespace: &str,
    ) -> Result<(), PermissionError> {
        for pattern in &self.manifest.storage.namespaces {
            if Self::matches_namespace_pattern(pattern, namespace) {
                return Ok(());
            }
        }
        
        Err(PermissionError::PermissionDenied(
            format!("Storage access denied for namespace: {}", namespace)
        ))
    }
    
    /// Match domain pattern with wildcard support
    fn matches_domain_pattern(pattern: &str, host: &str) -> bool {
        if pattern == host {
            return true;
        }
        
        // Wildcard matching: *.example.com matches api.example.com
        if let Some(suffix) = pattern.strip_prefix("*.") {
            return host.ends_with(suffix) || host == suffix;
        }
        
        false
    }
    
    /// Match namespace pattern with wildcard support
    fn matches_namespace_pattern(pattern: &str, namespace: &str) -> bool {
        if pattern == namespace {
            return true;
        }
        
        // Wildcard matching: myapp:* matches myapp:config
        if let Some(prefix) = pattern.strip_suffix("*") {
            return namespace.starts_with(prefix);
        }
        
        false
    }
}
```

**Implementation Steps:**
1. Create `src/core/permissions.rs` module
2. Implement permission data structures
3. Implement pattern validation logic
4. Implement pattern matching logic (glob, wildcard)
5. Add unit tests for all validation and matching
6. Update `src/core/mod.rs` to export permissions module

**File: `src/core/mod.rs` (modification)**
```rust
pub mod config;
pub mod error;
pub mod permissions;  // Add this line
pub mod result;
pub mod types;
```

**Success Criteria:**
- ✅ Permission structures defined with proper types
- ✅ Component.toml deserializes to CapabilityManifest
- ✅ All pattern validation implemented
- ✅ Pattern matching works correctly (glob, wildcard)
- ✅ >90% test coverage for permission logic

**Unit Tests: `src/core/permissions.rs`**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_filesystem_permission_validation() {
        // Valid patterns
        assert!(PermissionValidator::validate_filesystem_pattern("/data/**").is_ok());
        assert!(PermissionValidator::validate_filesystem_pattern("/tmp/*.txt").is_ok());
        
        // Invalid patterns (relative paths)
        assert!(PermissionValidator::validate_filesystem_pattern("data/**").is_err());
        assert!(PermissionValidator::validate_filesystem_pattern("../etc/passwd").is_err());
    }
    
    #[test]
    fn test_filesystem_glob_matching() {
        let manifest = CapabilityManifest {
            filesystem: FilesystemCapabilities {
                read: vec!["/data/**".to_string(), "/tmp/*.json".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        
        let validator = PermissionValidator::new(manifest).unwrap();
        
        // Should match recursive glob
        assert!(validator.check_filesystem_access("/data/input/file.txt", FilesystemAction::Read).is_ok());
        assert!(validator.check_filesystem_access("/data/nested/deep/file.txt", FilesystemAction::Read).is_ok());
        
        // Should match wildcard
        assert!(validator.check_filesystem_access("/tmp/data.json", FilesystemAction::Read).is_ok());
        
        // Should NOT match
        assert!(validator.check_filesystem_access("/etc/passwd", FilesystemAction::Read).is_err());
        assert!(validator.check_filesystem_access("/tmp/data.txt", FilesystemAction::Read).is_err());
    }
    
    #[test]
    fn test_network_domain_matching() {
        let manifest = CapabilityManifest {
            network: vec![
                NetworkPermission {
                    action: NetworkAction::Outbound,
                    host_pattern: "api.example.com".to_string(),
                    port: Some(443),
                },
                NetworkPermission {
                    action: NetworkAction::Outbound,
                    host_pattern: "*.github.com".to_string(),
                    port: Some(443),
                },
            ],
            ..Default::default()
        };
        
        let validator = PermissionValidator::new(manifest).unwrap();
        
        // Should match exact domain
        assert!(validator.check_network_access("api.example.com", 443, NetworkAction::Outbound).is_ok());
        
        // Should match wildcard subdomain
        assert!(validator.check_network_access("api.github.com", 443, NetworkAction::Outbound).is_ok());
        assert!(validator.check_network_access("raw.github.com", 443, NetworkAction::Outbound).is_ok());
        
        // Should NOT match different domain
        assert!(validator.check_network_access("malicious.com", 443, NetworkAction::Outbound).is_err());
        
        // Should NOT match different port
        assert!(validator.check_network_access("api.example.com", 80, NetworkAction::Outbound).is_err());
    }
    
    #[test]
    fn test_storage_namespace_matching() {
        let manifest = CapabilityManifest {
            storage: StorageCapabilities {
                namespaces: vec![
                    "myapp:config".to_string(),
                    "myapp:cache".to_string(),
                    "shared:*".to_string(),
                ],
                max_size: None,
            },
            ..Default::default()
        };
        
        let validator = PermissionValidator::new(manifest).unwrap();
        
        // Should match exact namespace
        assert!(validator.check_storage_access("myapp:config").is_ok());
        assert!(validator.check_storage_access("myapp:cache").is_ok());
        
        // Should match wildcard namespace
        assert!(validator.check_storage_access("shared:data").is_ok());
        assert!(validator.check_storage_access("shared:temp").is_ok());
        
        // Should NOT match
        assert!(validator.check_storage_access("otherapp:config").is_err());
        assert!(validator.check_storage_access("myapp:secrets").is_err());
    }
}
```

---

#### Task 2.3: Integrate Permission Validation into Component Loading
**Duration**: Days 11-12 (8 hours)

**Deliverables:**
- Hook permission validation into component loading pipeline
- Parse Component.toml capabilities section
- Store PermissionValidator with component instance
- Integration tests for permission enforcement

**File Modification: `src/runtime/component.rs`**

Add permission validation to component loading:

```rust
// src/runtime/component.rs

use crate::core::permissions::{CapabilityManifest, PermissionValidator};

pub struct Component {
    // ... existing fields ...
    
    /// Permission validator for capability checks
    permission_validator: Arc<PermissionValidator>,
}

impl Component {
    pub async fn load(
        engine: &Engine,
        component_path: &Path,
        config: ComponentConfig,
    ) -> Result<Self, WasmError> {
        // ... existing loading logic ...
        
        // Parse capabilities from Component.toml
        let manifest = Self::parse_capability_manifest(&config)?;
        
        // Validate and create permission validator
        let permission_validator = Arc::new(
            PermissionValidator::new(manifest)
                .map_err(|e| WasmError::ConfigError(format!("Invalid permissions: {}", e)))?
        );
        
        Ok(Self {
            // ... existing fields ...
            permission_validator,
        })
    }
    
    /// Parse capability manifest from Component.toml
    fn parse_capability_manifest(config: &ComponentConfig) -> Result<CapabilityManifest, WasmError> {
        // Check if config has capabilities section
        let capabilities_str = config.metadata.get("capabilities")
            .ok_or_else(|| WasmError::ConfigError("Missing capabilities section".to_string()))?;
        
        // Parse TOML capabilities section
        let manifest: CapabilityManifest = toml::from_str(capabilities_str)
            .map_err(|e| WasmError::ConfigError(format!("Invalid capabilities TOML: {}", e)))?;
        
        Ok(manifest)
    }
    
    /// Get permission validator for capability checks
    pub fn permission_validator(&self) -> &PermissionValidator {
        &self.permission_validator
    }
}
```

**Implementation Steps:**
1. Add permission_validator field to Component struct
2. Parse Component.toml capabilities section
3. Create PermissionValidator during component load
4. Store validator with component instance
5. Add getter method for accessing validator
6. Add integration tests

**Success Criteria:**
- ✅ Permission validator created during component load
- ✅ Invalid permissions cause load failure with clear error
- ✅ Permission validator accessible for capability checks
- ✅ Integration tests verify enforcement

**Integration Test: `tests/permission_validation_tests.rs`**
```rust
// tests/permission_validation_tests.rs

use airssys_wasm::prelude::*;

#[tokio::test]
async fn test_component_with_valid_permissions_loads() {
    let engine = Engine::default();
    
    let config = ComponentConfig {
        // ... component with valid filesystem permissions ...
    };
    
    let component = Component::load(&engine, Path::new("test.wasm"), config).await;
    assert!(component.is_ok());
}

#[tokio::test]
async fn test_component_with_invalid_permissions_fails_load() {
    let engine = Engine::default();
    
    let config = ComponentConfig {
        // ... component with invalid glob pattern ...
    };
    
    let component = Component::load(&engine, Path::new("test.wasm"), config).await;
    assert!(component.is_err());
    
    let err = component.unwrap_err();
    assert!(matches!(err, WasmError::ConfigError(_)));
}

#[tokio::test]
async fn test_permission_validator_accessible() {
    let engine = Engine::default();
    
    let config = ComponentConfig {
        // ... valid config ...
    };
    
    let component = Component::load(&engine, Path::new("test.wasm"), config).await.unwrap();
    let validator = component.permission_validator();
    
    // Validator should enforce declared permissions
    assert!(validator.check_filesystem_access("/data/input.txt", FilesystemAction::Read).is_ok());
    assert!(validator.check_filesystem_access("/etc/passwd", FilesystemAction::Read).is_err());
}
```

---

### Phase 3: Advanced Host Service Interfaces (Week 2-3)

#### Overview
Define extended host service interfaces for filesystem, network, and process operations with capability annotations.

---

#### Task 3.1: Filesystem Host Interface
**Duration**: Days 13-15 (12 hours)

**Deliverables:**
- `extensions/filesystem.wit` - File operations interface
- Async file operations (read, write, stat, delete, list)
- Path permission integration
- Filesystem usage examples

**File: `wit/extensions/filesystem.wit`**
```wit
package airssys:host-extensions@1.0.0;

use airssys:component-core/types.{file-error};

/// Filesystem operations - requires filesystem capability
interface filesystem {
    /// Read entire file contents
    /// 
    /// Capability: filesystem.read for path
    /// Performance: <1ms for small files (<1MB), streaming for large files
    read-file: func(path: string) -> result<list<u8>, file-error>;
    
    /// Write entire file contents (overwrites existing)
    /// 
    /// Capability: filesystem.write for path
    /// Atomicity: Write to temp file, then rename
    write-file: func(path: string, data: list<u8>) -> result<_, file-error>;
    
    /// Get file metadata
    /// 
    /// Capability: filesystem.read or filesystem.list for path
    stat-file: func(path: string) -> result<file-metadata, file-error>;
    
    /// Delete file
    /// 
    /// Capability: filesystem.delete for path
    delete-file: func(path: string) -> result<_, file-error>;
    
    /// List directory contents
    /// 
    /// Capability: filesystem.list for path
    list-directory: func(path: string) -> result<list<directory-entry>, file-error>;
    
    /// Create directory (and parents if needed)
    /// 
    /// Capability: filesystem.write for path
    create-directory: func(path: string) -> result<_, file-error>;
}

/// File metadata
record file-metadata {
    size: u64,
    is-directory: bool,
    is-symlink: bool,
    created: u64,      // Unix timestamp millis
    modified: u64,     // Unix timestamp millis
    readonly: bool,
}

/// Directory entry
record directory-entry {
    name: string,
    is-directory: bool,
    size: u64,
}
```

**Implementation Steps:**
1. Create `wit/extensions/filesystem.wit`
2. Define filesystem interface with all operations
3. Document capability requirements for each function
4. Define supporting types (file-metadata, directory-entry)
5. Create filesystem usage examples
6. Validate WIT syntax

**Success Criteria:**
- ✅ Complete filesystem API defined
- ✅ All operations have capability annotations in comments
- ✅ Async operations designed for large files
- ✅ Safe path handling specified
- ✅ WIT validates successfully

---

#### Task 3.2: Network Host Interface
**Duration**: Days 16-18 (12 hours)

**Deliverables:**
- `extensions/network.wit` - Network operations interface
- HTTP client interface
- TCP socket interface (basic)
- Network capability annotations
- Network usage examples

**File: `wit/extensions/network.wit`**
```wit
package airssys:host-extensions@1.0.0;

use airssys:component-core/types.{http-error};

/// Network operations - requires network capability
interface network {
    /// HTTP request (simple interface)
    /// 
    /// Capability: network.outbound for host:port
    /// Timeout: Specified in request or default 30s
    http-request: func(request: http-request) -> result<http-response, http-error>;
}

/// HTTP request
record http-request {
    method: http-method,
    url: string,
    headers: list<tuple<string, string>>,
    body: option<list<u8>>,
    timeout-ms: option<u64>,
}

/// HTTP response
record http-response {
    status: u16,
    headers: list<tuple<string, string>>,
    body: list<u8>,
}

/// HTTP methods
enum http-method {
    get,
    post,
    put,
    delete,
    patch,
    head,
    options,
}
```

**Implementation Steps:**
1. Create `wit/extensions/network.wit`
2. Define HTTP client interface
3. Document capability requirements
4. Define HTTP types (request, response, method)
5. Create network usage examples
6. Validate WIT syntax

**Success Criteria:**
- ✅ HTTP client interface complete
- ✅ Capability annotations documented
- ✅ Timeout handling specified
- ✅ Standard HTTP semantics followed
- ✅ WIT validates successfully

---

#### Task 3.3: Process Host Interface
**Duration**: Days 19-20 (8 hours)

**Deliverables:**
- `extensions/process.wit` - Process operations interface
- Spawn/wait/kill functions
- Environment variable access
- Process capability annotations
- Process usage examples

**File: `wit/extensions/process.wit`**
```wit
package airssys:host-extensions@1.0.0;

use airssys:component-core/types.{component-error};

/// Process operations - requires process capability
interface process {
    /// Spawn new process
    /// 
    /// Capability: process.spawn for command
    spawn-process: func(
        command: string,
        args: list<string>,
        env: option<list<tuple<string, string>>>
    ) -> result<process-handle, component-error>;
    
    /// Wait for process completion
    /// 
    /// Capability: Requires previous spawn permission
    wait-process: func(handle: process-handle, timeout-ms: u64) -> result<process-result, component-error>;
    
    /// Get environment variable
    /// 
    /// Capability: Always allowed (component's own env only)
    get-env: func(name: string) -> option<string>;
}

/// Process handle
type process-handle = u64;

/// Process result
record process-result {
    exit-code: s32,
    stdout: list<u8>,
    stderr: list<u8>,
}
```

**Implementation Steps:**
1. Create `wit/extensions/process.wit`
2. Define process operations interface
3. Document capability requirements
4. Define process types (handle, result)
5. Create process usage examples
6. Validate WIT syntax

**Success Criteria:**
- ✅ Process operations interface complete
- ✅ Safe process isolation designed
- ✅ Environment access controlled
- ✅ Capability requirements documented
- ✅ WIT validates successfully

---

### Phase 4: Rust Binding Generation (Week 3)

#### Overview
Integrate wit-bindgen for automatic Rust binding generation from WIT interfaces.

---

#### Task 4.1: wit-bindgen Integration
**Duration**: Days 21-23 (12 hours)

**Deliverables:**
- Cargo build integration for binding generation
- Build script (build.rs)
- Generated code location strategy
- Binding regeneration documentation

**Add Dependency: `Cargo.toml`**
```toml
[dependencies]
# ... existing dependencies ...

# WIT binding generation
wit-bindgen = { version = "0.16", default-features = false, features = ["realloc"] }

[build-dependencies]
wit-bindgen = "0.16"
```

**Build Script: `build.rs`**
```rust
// build.rs

use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let wit_dir = PathBuf::from("wit");
    
    // Generate bindings for host interfaces
    wit_bindgen_rust::generate!({
        path: wit_dir.join("core"),
        world: "component",
        exports: {
            world: true,
        },
        with: {
            "airssys:component-core": generate,
            "airssys:host-core": generate,
        },
    });
    
    // Trigger rebuild if WIT files change
    println!("cargo:rerun-if-changed=wit/");
}
```

**Generated Bindings Module: `src/bindings.rs`**
```rust
// src/bindings.rs

// Include generated bindings from build.rs
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Re-export for convenience
pub use airssys::component_core::*;
pub use airssys::host_core::*;
```

**Implementation Steps:**
1. Add wit-bindgen dependencies to Cargo.toml
2. Create build.rs script
3. Configure binding generation for all WIT packages
4. Create src/bindings.rs to include generated code
5. Test binding generation with `cargo build`
6. Document regeneration process

**Success Criteria:**
- ✅ Bindings generate automatically on `cargo build`
- ✅ Generated code compiles successfully
- ✅ Generated traits accessible from src/bindings.rs
- ✅ Rebuild triggered by WIT file changes
- ✅ Documentation explains regeneration process

**Testing:**
```bash
# Clean and rebuild to test generation
cargo clean
cargo build

# Verify bindings generated
ls -la target/debug/build/airssys-wasm-*/out/

# Should see bindings.rs with generated traits
```

---

#### Task 4.2: Rust Host Implementation Stubs
**Duration**: Days 24-25 (10 hours)

**Deliverables:**
- Host trait implementations (stubs)
- Type conversions for WIT types
- Error mappings (WIT errors → Rust errors)
- Host trait documentation

**New Module: `src/runtime/host_impl.rs`**
```rust
// src/runtime/host_impl.rs

use crate::bindings::*;
use crate::core::error::WasmError;
use crate::core::permissions::{PermissionValidator, FilesystemAction};
use std::sync::Arc;

/// Host services implementation
pub struct HostServicesImpl {
    permission_validator: Arc<PermissionValidator>,
    // ... other runtime services ...
}

impl airssys::host_core::HostServices for HostServicesImpl {
    fn log(
        &mut self,
        level: LogLevel,
        message: String,
        context: Option<Vec<(String, String)>>,
    ) {
        // Implementation in Block 5
        todo!("Implement logging")
    }
    
    fn send_message(
        &mut self,
        target: String,
        message: Vec<u8>,
    ) -> Result<(), MessagingError> {
        // Implementation in Block 6
        todo!("Implement messaging")
    }
    
    fn send_request(
        &mut self,
        target: String,
        request: Vec<u8>,
        timeout_ms: u64,
    ) -> Result<String, MessagingError> {
        // Implementation in Block 6
        todo!("Implement request-response")
    }
    
    fn cancel_request(
        &mut self,
        request_id: String,
    ) -> Result<(), MessagingError> {
        // Implementation in Block 6
        todo!("Implement request cancellation")
    }
    
    fn current_time_millis(&mut self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
    
    fn sleep_millis(&mut self, duration: u64) {
        // Implementation requires async context
        todo!("Implement sleep")
    }
    
    fn list_components(&mut self) -> Vec<String> {
        // Implementation in Block 3
        todo!("Implement component listing")
    }
    
    fn get_component_metadata(
        &mut self,
        id: String,
    ) -> Result<ComponentMetadata, ComponentError> {
        // Implementation in Block 3
        todo!("Implement metadata retrieval")
    }
}

/// Filesystem implementation with capability checks
pub struct FilesystemImpl {
    permission_validator: Arc<PermissionValidator>,
}

impl airssys::host_extensions::Filesystem for FilesystemImpl {
    fn read_file(&mut self, path: String) -> Result<Vec<u8>, FileError> {
        // Check permission
        self.permission_validator
            .check_filesystem_access(&path, FilesystemAction::Read)
            .map_err(|e| FileError::PermissionDenied(e.to_string()))?;
        
        // Implementation in Block 8
        todo!("Implement file reading")
    }
    
    fn write_file(&mut self, path: String, data: Vec<u8>) -> Result<(), FileError> {
        // Check permission
        self.permission_validator
            .check_filesystem_access(&path, FilesystemAction::Write)
            .map_err(|e| FileError::PermissionDenied(e.to_string()))?;
        
        // Implementation in Block 8
        todo!("Implement file writing")
    }
    
    // ... other filesystem methods ...
}
```

**Implementation Steps:**
1. Create src/runtime/host_impl.rs
2. Implement stub traits for all host interfaces
3. Add permission checks before operations
4. Map WIT errors to Rust errors
5. Document each stub implementation
6. Add unit tests for permission checking

**Success Criteria:**
- ✅ All host interfaces have stub implementations
- ✅ Permission validation integrated
- ✅ Type conversions correct
- ✅ Error mappings complete
- ✅ Stubs compile successfully

---

#### Task 4.3: Rust Component SDK Foundation
**Duration**: Days 26-27 (10 hours)

**Deliverables:**
- Component trait definitions from WIT
- Component macro for entry point generation
- Example component using WIT interfaces
- Component SDK documentation

**New Crate: `airssys-wasm-component-sdk/`**

**Create SDK crate structure:**
```
airssys-wasm-component-sdk/
├── src/
│   ├── lib.rs              # SDK re-exports
│   ├── component.rs        # Component trait and helpers
│   └── macros.rs           # Component macros
├── examples/
│   ├── hello_world.rs      # Basic component example
│   └── echo_service.rs     # RPC component example
├── Cargo.toml
└── README.md
```

**SDK Crate: `airssys-wasm-component-sdk/Cargo.toml`**
```toml
[package]
name = "airssys-wasm-component-sdk"
version = "0.1.0"
edition = "2021"

[dependencies]
wit-bindgen = { version = "0.16", default-features = false, features = ["realloc"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[build-dependencies]
wit-bindgen = "0.16"
```

**SDK: `airssys-wasm-component-sdk/src/component.rs`**
```rust
// airssys-wasm-component-sdk/src/component.rs

use crate::bindings::*;

/// Component trait that users implement
pub trait Component: Sized {
    /// Initialize component
    fn init(config: ComponentConfig) -> Result<Self, String>;
    
    /// Handle RPC execute call
    fn execute(&mut self, operation: Vec<u8>, context: ExecutionContext) -> Result<Vec<u8>, String>;
    
    /// Handle incoming message
    fn handle_message(&mut self, sender: String, message: Vec<u8>) -> Result<(), String>;
    
    /// Handle callback response (optional)
    fn handle_callback(&mut self, request_id: String, result: Result<Vec<u8>, String>) -> Result<(), String> {
        Ok(()) // Default: no-op
    }
    
    /// Component metadata
    fn metadata() -> ComponentMetadata;
    
    /// Health check
    fn health(&self) -> HealthStatus {
        HealthStatus {
            healthy: true,
            reason: None,
        }
    }
    
    /// Shutdown cleanup
    fn shutdown(self) -> Result<(), String> {
        Ok(()) // Default: no cleanup
    }
}

/// Export component implementation as WASM exports
#[macro_export]
macro_rules! export_component {
    ($component:ty) => {
        static mut COMPONENT: Option<$component> = None;
        
        #[no_mangle]
        pub extern "C" fn init(config: ComponentConfig) -> Result<(), ComponentError> {
            unsafe {
                COMPONENT = Some(<$component>::init(config)?);
            }
            Ok(())
        }
        
        #[no_mangle]
        pub extern "C" fn execute(
            operation: Vec<u8>,
            context: ExecutionContext,
        ) -> Result<Vec<u8>, ExecutionError> {
            unsafe {
                COMPONENT.as_mut()
                    .ok_or_else(|| ExecutionError::InternalError("Component not initialized".to_string()))?
                    .execute(operation, context)
            }
        }
        
        #[no_mangle]
        pub extern "C" fn handle_message(
            sender: String,
            message: Vec<u8>,
        ) -> Result<(), MessagingError> {
            unsafe {
                COMPONENT.as_mut()
                    .ok_or_else(|| MessagingError::DeliveryFailed("Component not initialized".to_string()))?
                    .handle_message(sender, message)
            }
        }
        
        // ... other exports ...
    };
}
```

**Example Component: `airssys-wasm-component-sdk/examples/hello_world.rs`**
```rust
// examples/hello_world.rs

use airssys_wasm_component_sdk::prelude::*;

struct HelloWorldComponent;

impl Component for HelloWorldComponent {
    fn init(config: ComponentConfig) -> Result<Self, String> {
        Ok(Self)
    }
    
    fn execute(&mut self, operation: Vec<u8>, context: ExecutionContext) -> Result<Vec<u8>, String> {
        Ok(b"Hello, World!".to_vec())
    }
    
    fn handle_message(&mut self, sender: String, message: Vec<u8>) -> Result<(), String> {
        // Log message received
        Ok(())
    }
    
    fn metadata() -> ComponentMetadata {
        ComponentMetadata {
            name: "hello-world".to_string(),
            version: "1.0.0".to_string(),
            description: "Simple hello world component".to_string(),
            // ... other fields ...
        }
    }
}

export_component!(HelloWorldComponent);
```

**Implementation Steps:**
1. Create airssys-wasm-component-sdk crate
2. Implement Component trait
3. Implement export_component! macro
4. Create example components
5. Document SDK usage
6. Test example components compile to WASM

**Success Criteria:**
- ✅ Component trait clear and minimal
- ✅ export_component! macro generates correct exports
- ✅ Example components compile to WASM
- ✅ Type-safe boundaries enforced
- ✅ Clear SDK documentation

**Testing:**
```bash
# Build example component to WASM
cd airssys-wasm-component-sdk
cargo build --target wasm32-wasi --example hello_world

# Verify WASM output
ls -la target/wasm32-wasi/debug/examples/hello_world.wasm
```

---

### Phase 5: Interface Validation and Versioning (Week 3-4)

#### Overview
Implement interface compatibility checking and versioning strategy for WIT interface evolution.

---

#### Task 5.1: Interface Compatibility Checking
**Duration**: Days 28-30 (12 hours)

**Deliverables:**
- Interface version validation logic
- Compatibility matrix checking
- Breaking change detection
- Clear error messages for mismatches

**New Module: `src/core/interface_validation.rs`**
```rust
// src/core/interface_validation.rs

use crate::core::error::WasmError;
use semver::{Version, VersionReq};

/// Interface version validator
pub struct InterfaceValidator {
    /// Supported interface versions by the runtime
    supported_versions: Vec<SupportedInterface>,
}

#[derive(Debug, Clone)]
pub struct SupportedInterface {
    pub name: String,
    pub version_requirement: VersionReq,
}

impl InterfaceValidator {
    pub fn new() -> Self {
        Self {
            supported_versions: vec![
                SupportedInterface {
                    name: "airssys:component-core".to_string(),
                    version_requirement: VersionReq::parse("^1.0.0").unwrap(),
                },
                SupportedInterface {
                    name: "airssys:host-core".to_string(),
                    version_requirement: VersionReq::parse("^1.0.0").unwrap(),
                },
                SupportedInterface {
                    name: "airssys:host-extensions".to_string(),
                    version_requirement: VersionReq::parse("^1.0.0").unwrap(),
                },
            ],
        }
    }
    
    /// Validate component's required interfaces
    pub fn validate_component_interfaces(
        &self,
        component_interfaces: &[ComponentInterface],
    ) -> Result<(), WasmError> {
        for interface in component_interfaces {
            self.validate_single_interface(interface)?;
        }
        Ok(())
    }
    
    /// Validate single interface compatibility
    fn validate_single_interface(
        &self,
        interface: &ComponentInterface,
    ) -> Result<(), WasmError> {
        // Find supported interface
        let supported = self.supported_versions
            .iter()
            .find(|s| s.name == interface.name)
            .ok_or_else(|| WasmError::InterfaceError(
                format!("Unsupported interface: {}", interface.name)
            ))?;
        
        // Parse component's required version
        let component_version = Version::parse(&interface.version)
            .map_err(|e| WasmError::InterfaceError(
                format!("Invalid version '{}': {}", interface.version, e)
            ))?;
        
        // Check compatibility
        if !supported.version_requirement.matches(&component_version) {
            return Err(WasmError::InterfaceError(
                format!(
                    "Interface version mismatch: {} requires version {}, but runtime supports {}",
                    interface.name,
                    interface.version,
                    supported.version_requirement
                )
            ));
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ComponentInterface {
    pub name: String,
    pub version: String,
}
```

**Implementation Steps:**
1. Create src/core/interface_validation.rs
2. Implement InterfaceValidator
3. Add semantic versioning support (semver crate)
4. Implement compatibility checking logic
5. Add clear error messages
6. Integrate into component loading

**Cargo.toml Addition:**
```toml
[dependencies]
# ... existing ...
semver = "1.0"
```

**Success Criteria:**
- ✅ Interface version validation implemented
- ✅ Semantic versioning support (^1.0.0, ~1.2.3, etc.)
- ✅ Clear error messages for version mismatches
- ✅ Integration with component loading
- ✅ Unit tests for all validation scenarios

---

#### Task 5.2: Interface Evolution Strategy Documentation
**Duration**: Days 31-32 (8 hours)

**Deliverables:**
- Semantic versioning guidelines
- Backward compatibility rules
- Deprecation patterns
- Migration path documentation

**Document: `wit/VERSIONING.md`**
```markdown
# WIT Interface Versioning Strategy

## Semantic Versioning

All WIT interfaces follow semantic versioning: `MAJOR.MINOR.PATCH`

### Version Components

- **MAJOR**: Breaking changes (incompatible API changes)
- **MINOR**: Backward-compatible new features
- **PATCH**: Backward-compatible bug fixes

### Breaking Changes (MAJOR bump)

Breaking changes require MAJOR version increment:
- Removing functions or interfaces
- Removing or renaming parameters
- Changing parameter types (incompatible)
- Changing function semantics
- Removing enum variants
- Renaming record fields

### Backward-Compatible Changes (MINOR bump)

Non-breaking changes require MINOR version increment:
- Adding new functions
- Adding new interfaces
- Adding new optional parameters
- Adding new enum variants (at end)
- Adding new record fields (with defaults)

### Bug Fixes (PATCH bump)

Non-breaking bug fixes require PATCH version increment:
- Documentation corrections
- Implementation bug fixes (no API change)
- Performance improvements

## Compatibility Requirements

### Runtime Support

Runtime MUST support:
- Exact version match (1.2.3)
- Minor version range (^1.2 = 1.2.x, 1.3.x, ..., but not 2.x)
- Patch version range (~1.2.3 = 1.2.3, 1.2.4, ..., but not 1.3.x)

### Component Requirements

Components declare version requirements using semver:
```toml
[dependencies]
"airssys:component-core" = "^1.0.0"  # 1.x.x compatible
"airssys:host-core" = "~1.2.0"       # 1.2.x compatible
```

## Deprecation Process

### Step 1: Mark as Deprecated (MINOR bump)
Add `@deprecated` annotation with replacement:
```wit
/// @deprecated Use new-function instead
old-function: func() -> result<_, error>;
```

### Step 2: Warning Period (minimum 2 MINOR versions)
Runtime warns on usage of deprecated features:
```
WARNING: Component 'my-component' uses deprecated function 'old-function'
         Use 'new-function' instead. Support will be removed in version 2.0.0
```

### Step 3: Removal (MAJOR bump)
Remove deprecated function in next MAJOR version.

## Migration Paths

### MAJOR Version Migration

Provide clear migration guide:
1. List all breaking changes
2. Provide before/after code examples
3. Document workarounds for complex migrations
4. Offer compatibility shim if possible

### Example Migration Guide (1.x → 2.x)

**Breaking Change**: `execute` parameter changed from `data: list<u8>` to `operation: operation-request`

**Migration**:
```rust
// Before (1.x)
fn execute(&mut self, data: Vec<u8>) -> Result<Vec<u8>, String>

// After (2.x)
fn execute(&mut self, operation: OperationRequest) -> Result<Vec<u8>, String>

// Workaround for gradual migration
impl Component for MyComponent {
    fn execute(&mut self, operation: OperationRequest) -> Result<Vec<u8>, String> {
        let data = operation.payload; // Extract data
        // Rest of old implementation unchanged
    }
}
```

## Version Declaration

### WIT Package Version
```wit
package airssys:component-core@1.2.3;
```

### Component Version Requirement (Component.toml)
```toml
[component]
api-version = "1.2.3"  # Exact version component was built against

[dependencies]
"airssys:component-core" = "^1.2.0"  # Compatible range
```

## Testing Strategy

### Compatibility Testing

Test matrix for interface versions:
- Component built with 1.0.0 runs on runtime 1.0.0 ✅
- Component built with 1.0.0 runs on runtime 1.5.0 ✅
- Component built with 1.5.0 runs on runtime 1.0.0 ❌ (use newer APIs)
- Component built with 1.x runs on runtime 2.x ❌ (breaking changes)

### Automated Testing

CI pipeline validates:
- No breaking changes in MINOR/PATCH bumps
- Deprecated features still work
- Migration guides accurate
```

**Implementation Steps:**
1. Create wit/VERSIONING.md
2. Document semantic versioning rules
3. Define breaking vs non-breaking changes
4. Document deprecation process
5. Provide migration guide templates
6. Document testing strategy

**Success Criteria:**
- ✅ Clear versioning policy documented
- ✅ Breaking changes defined
- ✅ Deprecation process specified
- ✅ Migration patterns provided
- ✅ Testing strategy documented

---

#### Task 5.3: Interface Testing Framework
**Duration**: Days 33-34 (8 hours)

**Deliverables:**
- Interface validation test suite
- Compatibility test cases
- Version mismatch tests
- Invalid interface tests

**Integration Tests: `tests/interface_validation_tests.rs`**
```rust
// tests/interface_validation_tests.rs

use airssys_wasm::prelude::*;
use airssys_wasm::core::interface_validation::*;

#[test]
fn test_compatible_interface_version_accepted() {
    let validator = InterfaceValidator::new();
    
    let interfaces = vec![
        ComponentInterface {
            name: "airssys:component-core".to_string(),
            version: "1.0.0".to_string(),
        },
    ];
    
    assert!(validator.validate_component_interfaces(&interfaces).is_ok());
}

#[test]
fn test_minor_version_upgrade_compatible() {
    let validator = InterfaceValidator::new();
    
    // Component built with 1.0.0 should work with runtime supporting ^1.0
    let interfaces = vec![
        ComponentInterface {
            name: "airssys:component-core".to_string(),
            version: "1.5.0".to_string(),  // Newer minor version
        },
    ];
    
    assert!(validator.validate_component_interfaces(&interfaces).is_ok());
}

#[test]
fn test_major_version_mismatch_rejected() {
    let validator = InterfaceValidator::new();
    
    // Component built with 2.0.0 incompatible with runtime supporting ^1.0
    let interfaces = vec![
        ComponentInterface {
            name: "airssys:component-core".to_string(),
            version: "2.0.0".to_string(),  // Breaking version
        },
    ];
    
    let result = validator.validate_component_interfaces(&interfaces);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(matches!(err, WasmError::InterfaceError(_)));
    assert!(err.to_string().contains("version mismatch"));
}

#[test]
fn test_unsupported_interface_rejected() {
    let validator = InterfaceValidator::new();
    
    let interfaces = vec![
        ComponentInterface {
            name: "airssys:unknown-interface".to_string(),
            version: "1.0.0".to_string(),
        },
    ];
    
    let result = validator.validate_component_interfaces(&interfaces);
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Unsupported interface"));
}

#[test]
fn test_invalid_version_format_rejected() {
    let validator = InterfaceValidator::new();
    
    let interfaces = vec![
        ComponentInterface {
            name: "airssys:component-core".to_string(),
            version: "invalid.version".to_string(),
        },
    ];
    
    let result = validator.validate_component_interfaces(&interfaces);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid version"));
}
```

**Implementation Steps:**
1. Create tests/interface_validation_tests.rs
2. Test compatible version acceptance
3. Test minor version compatibility
4. Test major version rejection
5. Test unsupported interface rejection
6. Test invalid version format rejection
7. Achieve >90% coverage

**Success Criteria:**
- ✅ Comprehensive test coverage (>90%)
- ✅ All validation paths tested
- ✅ Invalid cases caught and reported
- ✅ Clear error messages validated
- ✅ All tests pass

---

### Phase 6: Documentation and Examples (Week 4)

#### Overview
Create comprehensive documentation, developer guides, and example components for WIT interfaces.

---

#### Task 6.1: WIT Interface Reference Documentation
**Duration**: Days 35-37 (12 hours)

**Deliverables:**
- Complete API reference for all WIT interfaces
- Function-level documentation
- Type reference documentation
- Capability requirement reference
- mdBook integration

**Create Documentation Structure:**
```
airssys-wasm/docs/src/
├── wit-reference/
│   ├── README.md                   # WIT reference index
│   ├── core/
│   │   ├── component.md            # component-lifecycle interface
│   │   ├── host.md                 # host-services interface
│   │   ├── types.md                # Common types
│   │   └── capabilities.md         # Capability types
│   ├── extensions/
│   │   ├── filesystem.md           # Filesystem interface
│   │   ├── network.md              # Network interface
│   │   └── process.md              # Process interface
│   └── versioning.md               # Versioning strategy
```

**Document: `docs/src/wit-reference/core/component.md`**
```markdown
# Component Lifecycle Interface

Package: `airssys:component-core@1.0.0`

## Interface: component-lifecycle

All components MUST implement this interface to be loadable by the runtime.

### init

```wit
init: func(config: component-config) -> result<_, component-error>;
```

Initialize component with configuration.

**Parameters:**
- `config`: Component configuration including environment variables and config data

**Returns:**
- `Ok(())`: Component initialized successfully
- `Err(component-error)`: Initialization failed

**When Called:**
Called once during component loading after WASM instantiation. Must complete successfully for component to become active.

**Use Cases:**
- Setup internal resources
- Validate configuration
- Initialize state
- Connect to external services (if permitted)

**Performance:**
Should complete within 1 second. Exceeding timeout causes load failure.

**Example:**
```rust
impl Component for MyComponent {
    fn init(config: ComponentConfig) -> Result<Self, String> {
        // Validate required config
        let api_key = config.env.iter()
            .find(|(k, _)| k == "API_KEY")
            .map(|(_, v)| v)
            .ok_or("Missing API_KEY")?;
        
        Ok(Self {
            api_key: api_key.clone(),
        })
    }
}
```

---

### execute

```wit
execute: func(
    operation: list<u8>,
    context: execution-context
) -> result<list<u8>, execution-error>;
```

Handle external RPC requests (synchronous request-response).

**Parameters:**
- `operation`: Encoded operation data (self-describing with multicodec prefix)
- `context`: Execution context with request ID and timeout

**Returns:**
- `Ok(Vec<u8>)`: Operation result (encoded response)
- `Err(execution-error)`: Operation failed

**Flow:**
```
External Entity → Host RPC Gateway → Component.execute()
```

**Use Cases:**
- HTTP API endpoints
- gRPC services
- CLI commands
- REST API handlers

**Timeout:**
Must complete within `context.timeout-ms`. Exceeding timeout returns error to caller.

**Data Format:**
Operation data uses multicodec prefixes for self-describing encoding:
- JSON: `0x0200` prefix + UTF-8 JSON
- MessagePack: `0x0201` prefix + MessagePack bytes
- Protobuf: Custom prefix + Protobuf bytes

**Example:**
```rust
impl Component for ApiComponent {
    fn execute(&mut self, operation: Vec<u8>, context: ExecutionContext) -> Result<Vec<u8>, String> {
        // Decode operation (assuming JSON)
        let request: ApiRequest = serde_json::from_slice(&operation[2..])
            .map_err(|e| format!("Invalid JSON: {}", e))?;
        
        // Process request
        let response = self.handle_api_request(request)?;
        
        // Encode response with multicodec prefix
        let mut result = vec![0x02, 0x00]; // JSON prefix
        serde_json::to_writer(&mut result, &response)
            .map_err(|e| format!("Failed to encode response: {}", e))?;
        
        Ok(result)
    }
}
```

--- (continue for all functions) ---
```

**Implementation Steps:**
1. Create docs/src/wit-reference/ structure
2. Document each interface with:
   - Function signatures
   - Parameter descriptions
   - Return value descriptions
   - When called / flow
   - Use cases
   - Performance expectations
   - Code examples
3. Document all types and enums
4. Document capability requirements
5. Integrate into mdBook (update SUMMARY.md)
6. Generate searchable documentation

**Success Criteria:**
- ✅ All interfaces fully documented
- ✅ Every function has examples
- ✅ Capability requirements clear
- ✅ Performance expectations documented
- ✅ Searchable documentation (mdBook)

---

#### Task 6.2: Component Development Guide
**Duration**: Days 38-39 (10 hours)

**Deliverables:**
- "Getting Started with WIT" tutorial
- Interface usage patterns
- Capability declaration guide
- Common pitfalls and solutions
- Best practices guide

**Document: `docs/src/guides/component-development.md`**
```markdown
# Component Development Guide

This guide walks you through developing WASM components using the WIT interface system.

## Prerequisites

- Rust 1.70+ with `wasm32-wasi` target
- airssys-wasm-component-sdk
- Basic understanding of WebAssembly

## Quick Start

### 1. Create New Component Project

```bash
cargo new --lib my-component
cd my-component
```

### 2. Configure Cargo.toml

```toml
[package]
name = "my-component"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
airssys-wasm-component-sdk = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Enable link-time optimization
codegen-units = 1   # Better optimization
```

### 3. Implement Component

```rust
// src/lib.rs

use airssys_wasm_component_sdk::prelude::*;
use serde::{Deserialize, Serialize};

// Your component state
struct MyComponent {
    counter: u32,
}

// Implement Component trait
impl Component for MyComponent {
    fn init(config: ComponentConfig) -> Result<Self, String> {
        Ok(Self { counter: 0 })
    }
    
    fn execute(&mut self, operation: Vec<u8>, _context: ExecutionContext) -> Result<Vec<u8>, String> {
        // Decode operation (JSON format)
        let request: Request = serde_json::from_slice(&operation[2..])
            .map_err(|e| format!("Invalid request: {}", e))?;
        
        // Process request
        let response = match request.action.as_str() {
            "increment" => {
                self.counter += 1;
                Response { result: self.counter }
            }
            "get" => Response { result: self.counter },
            _ => return Err(format!("Unknown action: {}", request.action)),
        };
        
        // Encode response with JSON prefix
        let mut result = vec![0x02, 0x00];
        serde_json::to_writer(&mut result, &response)
            .map_err(|e| format!("Encoding failed: {}", e))?;
        
        Ok(result)
    }
    
    fn handle_message(&mut self, _sender: String, _message: Vec<u8>) -> Result<(), String> {
        Ok(()) // No message handling needed
    }
    
    fn metadata() -> ComponentMetadata {
        ComponentMetadata {
            name: "my-component".to_string(),
            version: "0.1.0".to_string(),
            description: "Simple counter component".to_string(),
            author: "Your Name".to_string(),
            license: "MIT".to_string(),
            api_version: "1.0.0".to_string(),
            // ... other fields ...
        }
    }
}

#[derive(Deserialize)]
struct Request {
    action: String,
}

#[derive(Serialize)]
struct Response {
    result: u32,
}

// Export component
export_component!(MyComponent);
```

### 4. Declare Capabilities (Component.toml)

```toml
[component]
name = "my-component"
version = "0.1.0"
api-version = "1.0.0"

# No special capabilities needed for this simple component
[capabilities]
# filesystem = []
# network = []
```

### 5. Build Component

```bash
# Build WASM component
cargo build --target wasm32-wasi --release

# Output: target/wasm32-wasi/release/my_component.wasm
```

### 6. Test Component

```bash
# Use airssys-wasm-cli to test (Block 9)
airssys-wasm-cli test target/wasm32-wasi/release/my_component.wasm

# Or load in runtime
```

## Interface Usage Patterns

### Handling RPC Requests (execute)

The `execute` method handles synchronous request-response operations from external entities:

```rust
fn execute(&mut self, operation: Vec<u8>, context: ExecutionContext) -> Result<Vec<u8>, String> {
    // 1. Decode operation (check multicodec prefix)
    let codec = u16::from_be_bytes([operation[0], operation[1]]);
    let payload = &operation[2..];
    
    let request = match codec {
        0x0200 => serde_json::from_slice(payload)?,  // JSON
        0x0201 => rmp_serde::from_slice(payload)?,   // MessagePack
        _ => return Err(format!("Unsupported codec: {:#x}", codec)),
    };
    
    // 2. Process request
    let response = self.process_request(request)?;
    
    // 3. Encode response with same codec
    let mut result = vec![(codec >> 8) as u8, codec as u8];
    match codec {
        0x0200 => serde_json::to_writer(&mut result, &response)?,
        0x0201 => rmp_serde::encode::write(&mut result, &response)?,
        _ => unreachable!(),
    }
    
    Ok(result)
}
```

### Handling Messages (handle-message)

The `handle-message` method receives asynchronous messages from other components:

```rust
fn handle_message(&mut self, sender: String, message: Vec<u8>) -> Result<(), String> {
    // Decode message
    let event: Event = serde_json::from_slice(&message[2..])?;
    
    // Process event
    match event.event_type.as_str() {
        "user_created" => self.on_user_created(event.data)?,
        "user_deleted" => self.on_user_deleted(event.data)?,
        _ => return Err(format!("Unknown event: {}", event.event_type)),
    }
    
    Ok(())
}
```

### Sending Messages to Other Components

```rust
fn execute(&mut self, operation: Vec<u8>, context: ExecutionContext) -> Result<Vec<u8>, String> {
    // Process operation...
    
    // Send notification to another component
    let notification = Event {
        event_type: "operation_completed".to_string(),
        data: serde_json::to_value(&result)?,
    };
    
    let mut message = vec![0x02, 0x00]; // JSON prefix
    serde_json::to_writer(&mut message, &notification)?;
    
    // Fire-and-forget message
    host::send_message("notification-service", message)
        .map_err(|e| format!("Failed to send notification: {:?}", e))?;
    
    Ok(response)
}
```

### Request-Response Pattern (send-request + handle-callback)

```rust
// Send request to another component
fn execute(&mut self, operation: Vec<u8>, context: ExecutionContext) -> Result<Vec<u8>, String> {
    // Create request
    let request = DataRequest { query: "SELECT * FROM users".to_string() };
    let mut request_bytes = vec![0x02, 0x00];
    serde_json::to_writer(&mut request_bytes, &request)?;
    
    // Send request expecting callback
    let request_id = host::send_request("database-service", request_bytes, 5000)
        .map_err(|e| format!("Request failed: {:?}", e))?;
    
    // Store request ID for callback handling
    self.pending_requests.insert(request_id.clone(), context.request_id);
    
    // Return immediately (async pattern)
    Ok(vec![])
}

// Handle callback response
fn handle_callback(&mut self, request_id: String, result: Result<Vec<u8>, String>) -> Result<(), String> {
    // Retrieve original request context
    let original_request = self.pending_requests.remove(&request_id)
        .ok_or("Unknown request ID")?;
    
    // Process response
    match result {
        Ok(data) => {
            let response: DataResponse = serde_json::from_slice(&data[2..])?;
            // Handle successful response
            self.handle_data_response(original_request, response)?;
        }
        Err(error) => {
            // Handle error response
            self.handle_data_error(original_request, error)?;
        }
    }
    
    Ok(())
}
```

## Capability Declaration Guide

### Filesystem Capabilities

```toml
[capabilities.filesystem]
read = [
    "/etc/myapp/config.toml",     # Exact file
    "/var/data/input/**",          # Recursive directory
    "/tmp/cache/*.json",           # Wildcard pattern
]
write = [
    "/var/data/output/**",
    "/var/log/myapp.log",
]
delete = [
    "/tmp/myapp/**",
]
```

**Best Practices:**
- Use most specific patterns possible
- Avoid overly broad patterns like `/**`
- Document why each permission is needed
- Test with minimal permissions first

### Network Capabilities

```toml
[[capabilities.network]]
action = "outbound"
host = "api.example.com"
port = 443

[[capabilities.network]]
action = "outbound"
host = "*.cdn.example.com"  # Wildcard subdomain
port = 443
```

**Best Practices:**
- Specify exact domains when possible
- Use wildcards only when necessary
- Always specify ports explicitly
- Document external service dependencies

### Storage Capabilities

```toml
[capabilities.storage]
namespaces = [
    "myapp:config",      # Exact namespace
    "myapp:cache",
    "shared:public",     # Shared namespace
]
max-size = "100MB"
```

**Best Practices:**
- Use component-specific namespaces
- Set reasonable size limits
- Document data stored in each namespace
- Use `shared:*` only when necessary

## Common Pitfalls and Solutions

### Pitfall 1: Forgetting Multicodec Prefixes

**Problem:**
```rust
// ❌ Wrong - raw JSON without prefix
serde_json::to_vec(&response)
```

**Solution:**
```rust
// ✅ Correct - add multicodec prefix
let mut result = vec![0x02, 0x00]; // JSON prefix
serde_json::to_writer(&mut result, &response)?;
```

### Pitfall 2: Blocking Operations in execute()

**Problem:**
```rust
// ❌ Wrong - blocking sleep
std::thread::sleep(Duration::from_secs(5));
```

**Solution:**
```rust
// ✅ Correct - use host sleep or return quickly
host::sleep_millis(5000); // If blocking is necessary
// Or better: return immediately and use messaging
```

### Pitfall 3: Insufficient Capability Declarations

**Problem:**
Component tries to access `/var/data/file.txt` but only declared `/var/data/*.json`.

**Solution:**
Update Component.toml with broader pattern or additional specific paths:
```toml
read = [
    "/var/data/**",  # Matches all files in /var/data/
]
```

### Pitfall 4: Not Handling Capability Denials

**Problem:**
```rust
// ❌ No error handling for capability denial
let data = host::filesystem::read_file("/etc/passwd")?;
```

**Solution:**
```rust
// ✅ Handle permission errors gracefully
match host::filesystem::read_file("/etc/passwd") {
    Ok(data) => process_data(data),
    Err(FileError::PermissionDenied(path)) => {
        return Err(format!("Access denied: {}", path));
    }
    Err(e) => return Err(format!("File error: {:?}", e)),
}
```

## Best Practices

### Component Design
- Keep components small and focused
- Follow single responsibility principle
- Design for composability
- Use messaging for inter-component communication

### Performance
- Minimize component size (<1MB target)
- Optimize for startup time (<100ms target)
- Use efficient encoding (MessagePack for performance, JSON for debugging)
- Implement health() lightweight (<10ms)

### Security
- Request minimum necessary capabilities
- Validate all inputs thoroughly
- Never trust external data
- Use structured error types, avoid leaking sensitive info

### Testing
- Unit test component logic in native Rust
- Integration test as WASM component
- Test capability enforcement
- Test error scenarios

## Next Steps

- Read [WIT Interface Reference](../wit-reference/) for complete API
- Explore [Component Examples](../examples/) for patterns
- Review [Security Best Practices](../security/) for hardening
- Learn [Deployment Patterns](../deployment/) for production

--- (continue with more sections) ---
```

**Implementation Steps:**
1. Create docs/src/guides/component-development.md
2. Write quick start tutorial
3. Document interface usage patterns
4. Create capability declaration guide
5. Document common pitfalls with solutions
6. Document best practices
7. Integrate into mdBook

**Success Criteria:**
- ✅ Complete tutorial from start to deployment
- ✅ Interface patterns clearly explained
- ✅ Capability system understandable
- ✅ Common pitfalls documented with solutions
- ✅ Best practices guide comprehensive

---

#### Task 6.3: Interface Examples and Templates
**Duration**: Days 40-42 (12 hours)

**Deliverables:**
- Example component using storage interface
- Example component using messaging interface
- Example component using network interface
- Component project templates
- Example documentation

**Create Examples Directory:**
```
airssys-wasm-component-sdk/examples/
├── hello_world.rs          # Basic component
├── echo_service.rs         # RPC component
├── counter_service.rs      # Stateful component
├── message_processor.rs    # Messaging component
├── http_client.rs          # Network component
├── file_processor.rs       # Filesystem component
└── request_response.rs     # Request-response pattern
```

**Example: `examples/message_processor.rs`**
```rust
// examples/message_processor.rs

use airssys_wasm_component_sdk::prelude::*;
use serde::{Deserialize, Serialize};

/// Message processor that receives events and forwards them
struct MessageProcessor {
    processed_count: u64,
}

impl Component for MessageProcessor {
    fn init(_config: ComponentConfig) -> Result<Self, String> {
        host::log(LogLevel::Info, "MessageProcessor initialized", None);
        Ok(Self { processed_count: 0 })
    }
    
    fn execute(&mut self, operation: Vec<u8>, _context: ExecutionContext) -> Result<Vec<u8>, String> {
        // Handle status query
        let request: StatusRequest = serde_json::from_slice(&operation[2..])?;
        
        let response = StatusResponse {
            processed_count: self.processed_count,
            status: "running".to_string(),
        };
        
        let mut result = vec![0x02, 0x00];
        serde_json::to_writer(&mut result, &response)?;
        Ok(result)
    }
    
    fn handle_message(&mut self, sender: String, message: Vec<u8>) -> Result<(), String> {
        // Decode event
        let event: Event = serde_json::from_slice(&message[2..])?;
        
        host::log(
            LogLevel::Info,
            &format!("Received event: {:?} from {}", event.event_type, sender),
            None,
        );
        
        // Process event
        self.process_event(&event)?;
        
        // Forward to downstream component
        self.forward_event(&event)?;
        
        self.processed_count += 1;
        Ok(())
    }
    
    fn metadata() -> ComponentMetadata {
        ComponentMetadata {
            name: "message-processor".to_string(),
            version: "1.0.0".to_string(),
            description: "Processes and forwards events".to_string(),
            author: "AirsSys Team".to_string(),
            license: "MIT".to_string(),
            api_version: "1.0.0".to_string(),
            supported_operations: vec!["status".to_string()],
            // ... other fields ...
        }
    }
}

impl MessageProcessor {
    fn process_event(&mut self, event: &Event) -> Result<(), String> {
        // Process event logic
        match event.event_type.as_str() {
            "user_action" => {
                // Handle user action
                host::log(LogLevel::Debug, "Processing user action", None);
            }
            "system_event" => {
                // Handle system event
                host::log(LogLevel::Debug, "Processing system event", None);
            }
            _ => {
                host::log(
                    LogLevel::Warn,
                    &format!("Unknown event type: {}", event.event_type),
                    None,
                );
            }
        }
        Ok(())
    }
    
    fn forward_event(&mut self, event: &Event) -> Result<(), String> {
        // Forward to downstream component
        let mut message = vec![0x02, 0x00];
        serde_json::to_writer(&mut message, event)?;
        
        host::send_message("downstream-processor", message)
            .map_err(|e| format!("Failed to forward event: {:?}", e))?;
        
        Ok(())
    }
}

#[derive(Deserialize)]
struct StatusRequest {
    _action: String,
}

#[derive(Serialize)]
struct StatusResponse {
    processed_count: u64,
    status: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Event {
    event_type: String,
    timestamp: u64,
    data: serde_json::Value,
}

export_component!(MessageProcessor);
```

**Example: `examples/http_client.rs`**
```rust
// examples/http_client.rs

use airssys_wasm_component_sdk::prelude::*;
use serde::{Deserialize, Serialize};

/// HTTP client component that fetches data from external APIs
struct HttpClient;

impl Component for HttpClient {
    fn init(_config: ComponentConfig) -> Result<Self, String> {
        Ok(Self)
    }
    
    fn execute(&mut self, operation: Vec<u8>, _context: ExecutionContext) -> Result<Vec<u8>, String> {
        // Decode request
        let request: ApiRequest = serde_json::from_slice(&operation[2..])?;
        
        // Make HTTP request
        let http_request = HttpRequest {
            method: HttpMethod::Get,
            url: request.url.clone(),
            headers: vec![
                ("User-Agent".to_string(), "AirsSys-Component/1.0".to_string()),
                ("Accept".to_string(), "application/json".to_string()),
            ],
            body: None,
            timeout_ms: Some(5000),
        };
        
        let http_response = host::network::http_request(http_request)
            .map_err(|e| format!("HTTP request failed: {:?}", e))?;
        
        // Parse response
        let api_data: serde_json::Value = serde_json::from_slice(&http_response.body)
            .map_err(|e| format!("Invalid JSON response: {}", e))?;
        
        // Create response
        let response = ApiResponse {
            status: http_response.status,
            data: api_data,
        };
        
        let mut result = vec![0x02, 0x00];
        serde_json::to_writer(&mut result, &response)?;
        Ok(result)
    }
    
    fn handle_message(&mut self, _sender: String, _message: Vec<u8>) -> Result<(), String> {
        Ok(())
    }
    
    fn metadata() -> ComponentMetadata {
        ComponentMetadata {
            name: "http-client".to_string(),
            version: "1.0.0".to_string(),
            description: "HTTP client for external API calls".to_string(),
            author: "AirsSys Team".to_string(),
            license: "MIT".to_string(),
            api_version: "1.0.0".to_string(),
            // ... other fields ...
        }
    }
}

#[derive(Deserialize)]
struct ApiRequest {
    url: String,
}

#[derive(Serialize)]
struct ApiResponse {
    status: u16,
    data: serde_json::Value,
}

export_component!(HttpClient);
```

**Component.toml for http_client:**
```toml
[component]
name = "http-client"
version = "1.0.0"
api-version = "1.0.0"

# Network permissions for external APIs
[[capabilities.network]]
action = "outbound"
host = "api.github.com"
port = 443

[[capabilities.network]]
action = "outbound"
host = "*.example.com"
port = 443
```

**Project Template: `templates/component-template/`**
```
templates/component-template/
├── Cargo.toml
├── Component.toml
├── src/
│   └── lib.rs
├── tests/
│   └── integration_test.rs
└── README.md
```

**Implementation Steps:**
1. Create airssys-wasm-component-sdk/examples/ directory
2. Implement example components:
   - hello_world.rs (basic)
   - echo_service.rs (RPC)
   - counter_service.rs (stateful)
   - message_processor.rs (messaging)
   - http_client.rs (network)
   - file_processor.rs (filesystem)
   - request_response.rs (request-response)
3. Create Component.toml for each example
4. Create project template structure
5. Document each example
6. Test all examples compile to WASM

**Success Criteria:**
- ✅ Working examples for all major interfaces
- ✅ Examples demonstrate best practices
- ✅ All examples compile to WASM
- ✅ Templates ready for copy-paste
- ✅ Clear documentation for each example

**Testing:**
```bash
# Build all examples
cd airssys-wasm-component-sdk
for example in examples/*.rs; do
    name=$(basename "$example" .rs)
    cargo build --target wasm32-wasi --example "$name" --release
done

# Verify all compiled successfully
ls -la target/wasm32-wasi/release/examples/
```

---

## 2. File Creation and Modification Plan

### New Files to Create

**WIT Interface Files:**
1. `airssys-wasm/wit/README.md` - WIT documentation index
2. `airssys-wasm/wit/core/types.wit` - Common types
3. `airssys-wasm/wit/core/capabilities.wit` - Capability types
4. `airssys-wasm/wit/core/host.wit` - Host services interface
5. `airssys-wasm/wit/core/component.wit` - Component lifecycle interface
6. `airssys-wasm/wit/extensions/filesystem.wit` - Filesystem interface
7. `airssys-wasm/wit/extensions/network.wit` - Network interface
8. `airssys-wasm/wit/extensions/process.wit` - Process interface
9. `airssys-wasm/wit/examples/basic-component.wit` - Example interface

**Rust Implementation Files:**
10. `airssys-wasm/src/core/permissions.rs` - Permission system
11. `airssys-wasm/src/core/interface_validation.rs` - Interface validation
12. `airssys-wasm/src/bindings.rs` - Generated bindings wrapper
13. `airssys-wasm/src/runtime/host_impl.rs` - Host implementation stubs
14. `airssys-wasm/build.rs` - Build script for wit-bindgen

**SDK Files:**
15. `airssys-wasm-component-sdk/` - New crate directory
16. `airssys-wasm-component-sdk/Cargo.toml` - SDK crate config
17. `airssys-wasm-component-sdk/src/lib.rs` - SDK exports
18. `airssys-wasm-component-sdk/src/component.rs` - Component trait
19. `airssys-wasm-component-sdk/src/macros.rs` - Component macros
20. `airssys-wasm-component-sdk/examples/hello_world.rs` - Basic example
21. `airssys-wasm-component-sdk/examples/echo_service.rs` - RPC example
22. `airssys-wasm-component-sdk/examples/message_processor.rs` - Messaging example
23. `airssys-wasm-component-sdk/examples/http_client.rs` - Network example
24. `airssys-wasm-component-sdk/examples/file_processor.rs` - Filesystem example

**Test Files:**
25. `airssys-wasm/tests/permission_validation_tests.rs` - Permission tests
26. `airssys-wasm/tests/interface_validation_tests.rs` - Interface validation tests
27. `airssys-wasm/tests/wit_binding_tests.rs` - Binding generation tests

**Documentation Files:**
28. `airssys-wasm/wit/VERSIONING.md` - Versioning strategy
29. `airssys-wasm/docs/src/wit-reference/` - API reference directory
30. `airssys-wasm/docs/src/wit-reference/core/component.md` - Component interface docs
31. `airssys-wasm/docs/src/wit-reference/core/host.md` - Host services docs
32. `airssys-wasm/docs/src/wit-reference/extensions/filesystem.md` - Filesystem docs
33. `airssys-wasm/docs/src/wit-reference/extensions/network.md` - Network docs
34. `airssys-wasm/docs/src/guides/component-development.md` - Development guide

### Existing Files to Modify

**Cargo Configuration:**
1. `airssys-wasm/Cargo.toml` - Add wit-bindgen, semver, glob dependencies
2. `Cargo.toml` (workspace root) - Add airssys-wasm-component-sdk member

**Module Declarations:**
3. `airssys-wasm/src/core/mod.rs` - Export permissions module
4. `airssys-wasm/src/runtime/mod.rs` - Export host_impl module
5. `airssys-wasm/src/lib.rs` - Export bindings module

**Runtime Integration:**
6. `airssys-wasm/src/runtime/component.rs` - Add permission validation to component loading
7. `airssys-wasm/src/core/error.rs` - Add interface validation errors, permission errors

**Configuration:**
8. `airssys-wasm/src/core/config.rs` - Add capability manifest parsing

**Documentation:**
9. `airssys-wasm/docs/src/SUMMARY.md` - Add WIT reference and guides sections

---

## 3. Dependencies and Prerequisites

### External Dependencies to Add

**Cargo.toml (airssys-wasm):**
```toml
[dependencies]
# ... existing dependencies ...

# WIT binding generation
wit-bindgen = { version = "0.16", default-features = false, features = ["realloc"] }

# Permission pattern matching
glob = "0.3"

# Semantic versioning
semver = "1.0"

[build-dependencies]
wit-bindgen = "0.16"
```

**Cargo.toml (airssys-wasm-component-sdk):**
```toml
[dependencies]
wit-bindgen = { version = "0.16", default-features = false, features = ["realloc"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[build-dependencies]
wit-bindgen = "0.16"
```

### Knowledge/Research Required

**Component Model Specification:**
- WIT IDL syntax and semantics
- World definitions and composition
- Resource handles and lifecycle
- Interface versioning best practices

**wit-bindgen:**
- Rust binding generation configuration
- Build integration patterns
- Generated code structure
- Host vs guest bindings

**Pattern Matching:**
- Glob pattern library for Rust (glob crate)
- Wildcard domain matching algorithms
- Performance optimization for pattern matching

**Semantic Versioning:**
- semver crate usage
- Version requirement syntax (^, ~, =)
- Compatibility checking algorithms

---

## 4. Integration Points with Existing Code

### Block 1 Runtime Integration

**Component Loading (`src/runtime/component.rs`):**
```rust
// Hook 1: Parse capability manifest during load
let manifest = Self::parse_capability_manifest(&config)?;

// Hook 2: Create permission validator
let permission_validator = Arc::new(PermissionValidator::new(manifest)?);

// Hook 3: Validate WIT interfaces
let interface_validator = InterfaceValidator::new();
interface_validator.validate_component_interfaces(&component.interfaces())?;

// Hook 4: Store validator with component
self.permission_validator = permission_validator;
```

**Host Function Dispatch:**
```rust
// Before calling host function, check permission
fn call_host_function(&self, func_name: &str, args: &[Val]) -> Result<Vec<Val>, WasmError> {
    // Check permission based on function name
    if func_name.starts_with("filesystem::") {
        let path = extract_path_arg(args)?;
        self.permission_validator.check_filesystem_access(path, action)?;
    }
    
    // Call actual host function
    self.invoke_host_function(func_name, args)
}
```

### Error Handling Integration

**Add New Error Types (`src/core/error.rs`):**
```rust
#[derive(Debug, Error)]
pub enum WasmError {
    // ... existing variants ...
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Interface error: {0}")]
    InterfaceError(String),
    
    #[error("Invalid capability manifest: {0}")]
    CapabilityError(String),
}
```

### Configuration Integration

**Extend ComponentConfig (`src/core/config.rs`):**
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct ComponentConfig {
    // ... existing fields ...
    
    /// Capability manifest from Component.toml
    pub capabilities: Option<CapabilityManifest>,
}
```

---

## 5. Testing Strategy

### Unit Testing Plan

**Coverage Target:** >90%

**Permission Module Tests (`src/core/permissions.rs`):**
- Filesystem glob pattern validation
- Filesystem glob pattern matching
- Network domain wildcard validation
- Network domain wildcard matching
- Storage namespace pattern validation
- Storage namespace pattern matching
- Invalid pattern detection
- Permission denial scenarios

**Interface Validation Tests (`src/core/interface_validation.rs`):**
- Compatible version acceptance
- Minor version upgrade compatibility
- Major version mismatch rejection
- Unsupported interface rejection
- Invalid version format rejection
- Multiple interface validation

**WIT Binding Tests (`tests/wit_binding_tests.rs`):**
- Bindings generate successfully
- Generated code compiles
- Type conversions correct
- Error mappings accurate

### Integration Testing Plan

**Permission Enforcement Tests (`tests/permission_validation_tests.rs`):**
- Component with valid permissions loads
- Component with invalid permissions fails load
- Permission validator accessible from component
- Filesystem access enforcement
- Network access enforcement
- Storage access enforcement

**Component Lifecycle Tests:**
- Load component with WIT validation
- Call host functions through bindings
- Capability checks enforce permissions
- Interface version mismatch caught

**Multi-Component Tests:**
- Component A sends message to Component B
- Component A requests data from Component B
- Callback routing works correctly
- Permission isolation maintained

### Performance Testing Plan

**Benchmarks (`benches/interface_benchmarks.rs`):**
```rust
// Interface validation benchmark
fn bench_interface_validation(c: &mut Criterion) {
    let validator = InterfaceValidator::new();
    let interfaces = create_test_interfaces();
    
    c.bench_function("interface_validation", |b| {
        b.iter(|| validator.validate_component_interfaces(&interfaces))
    });
}

// Permission check benchmark
fn bench_permission_check(c: &mut Criterion) {
    let validator = create_permission_validator();
    
    c.bench_function("filesystem_permission_check", |b| {
        b.iter(|| validator.check_filesystem_access("/data/file.txt", FilesystemAction::Read))
    });
}

// Binding overhead benchmark
fn bench_binding_overhead(c: &mut Criterion) {
    // Measure overhead of calling host function through WIT bindings
    c.bench_function("host_function_call", |b| {
        b.iter(|| host::current_time_millis())
    });
}
```

**Performance Targets:**
- Interface validation: <1ms per component load
- Permission check: <5μs per check
- Binding overhead: <1μs per host function call

### Documentation Testing

**Code Example Compilation:**
```bash
# Extract and compile all code examples from documentation
cargo test --doc

# Test all example components
cd airssys-wasm-component-sdk
cargo test --examples
```

**Link Validation:**
```bash
# Use markdown link checker
markdown-link-check docs/**/*.md
```

---

## 6. Success Criteria and Validation

### Phase Completion Checklist

**Phase 1 Complete When:**
- ✅ WIT directory structure created
- ✅ Core interfaces defined (types.wit, capabilities.wit, host.wit, component.wit)
- ✅ All WIT files validate successfully
- ✅ Documentation complete for each interface

**Phase 2 Complete When:**
- ✅ Permission structures implemented
- ✅ Component.toml parsing works
- ✅ Pattern matching implemented (glob, wildcard)
- ✅ Permission validation integrated into component loading
- ✅ >90% test coverage for permission logic

**Phase 3 Complete When:**
- ✅ Extended interfaces defined (filesystem.wit, network.wit, process.wit)
- ✅ All interfaces follow consistent patterns
- ✅ Capability requirements documented
- ✅ All WIT files validate

**Phase 4 Complete When:**
- ✅ wit-bindgen integration working
- ✅ Bindings generate automatically
- ✅ Host trait stubs implemented
- ✅ Component SDK foundation ready
- ✅ Example components compile to WASM

**Phase 5 Complete When:**
- ✅ Interface validation implemented
- ✅ Versioning strategy documented
- ✅ Compatibility testing complete
- ✅ >90% test coverage

**Phase 6 Complete When:**
- ✅ Complete API reference documentation
- ✅ Component development guide written
- ✅ All examples working
- ✅ Templates available
- ✅ Documentation accurate and searchable

### Overall Task Completion Criteria

**Definition of Done:**
1. ✅ All WIT interfaces defined and validated
2. ✅ Rust bindings generate successfully
3. ✅ Capability system operational
4. ✅ Interface validation working
5. ✅ >90% test coverage achieved
6. ✅ Documentation complete and accurate
7. ✅ Performance targets met (<1ms validation, <1μs binding, <5μs permission check)
8. ✅ Zero warnings
9. ✅ All examples compile and run
10. ✅ SDK ready for external component development

**Validation Commands:**
```bash
# Code quality
cargo clippy --workspace --all-targets --all-features
cargo test --workspace

# WIT validation
wasm-tools component wit airssys-wasm/wit/

# Performance benchmarks
cargo bench -p airssys-wasm

# Documentation
mdbook test airssys-wasm/docs
mdbook build airssys-wasm/docs

# Examples
cd airssys-wasm-component-sdk
cargo build --target wasm32-wasi --examples --release
```

---

## 7. Timeline and Milestones

### Week 1 Milestones (Days 1-7)

**End of Day 1:**
- ✅ WIT directory structure created
- ✅ Package naming conventions established

**End of Day 3:**
- ✅ Core WIT interfaces defined (types.wit, capabilities.wit, host.wit)
- ✅ WIT syntax validated

**End of Day 5:**
- ✅ Component lifecycle interface complete (component.wit)
- ✅ Component world definition working

**End of Day 7:**
- ✅ Permission structures implemented
- ✅ Component.toml parsing works

**Week 1 Checkpoint:**
- Core WIT interfaces complete and validated
- Permission system foundation ready
- Ready to implement pattern matching

---

### Week 2 Milestones (Days 8-14)

**End of Day 10:**
- ✅ Permission validation integrated
- ✅ Pattern matching working (glob, wildcard)

**End of Day 12:**
- ✅ Permission system fully tested (>90% coverage)
- ✅ Integration with component loading complete

**End of Day 14:**
- ✅ Extended interfaces defined (filesystem.wit)
- ✅ Capability annotations documented

**Week 2 Checkpoint:**
- Capability permission system operational
- Filesystem interface complete
- Ready for network and process interfaces

---

### Week 3 Milestones (Days 15-21)

**End of Day 17:**
- ✅ Network interface complete (network.wit)
- ✅ Process interface complete (process.wit)

**End of Day 19:**
- ✅ All extended interfaces validated
- ✅ Interface consistency verified

**End of Day 21:**
- ✅ wit-bindgen integration working
- ✅ Bindings generate automatically

**Week 3 Checkpoint:**
- All WIT interfaces complete
- Rust binding generation working
- Ready for host implementation stubs

---

### Week 4 Milestones (Days 22-28+)

**End of Day 24:**
- ✅ Host trait stubs implemented
- ✅ Permission checks integrated

**End of Day 27:**
- ✅ Component SDK foundation ready
- ✅ Example components working

**End of Day 30:**
- ✅ Interface validation complete
- ✅ Versioning strategy documented

**End of Day 34:**
- ✅ Interface testing framework complete
- ✅ >90% test coverage achieved

**End of Day 37:**
- ✅ API reference documentation complete
- ✅ mdBook integration working

**End of Day 40:**
- ✅ Component development guide complete
- ✅ All examples working

**End of Day 42 (Final):**
- ✅ All deliverables complete
- ✅ Task ready for completion

**Week 4 Checkpoint:**
- All phases complete
- Documentation comprehensive
- Examples working
- Ready for Block 3

---

## 8. Risk Mitigation Strategies

### Risk 1: WIT Specification Evolution

**Mitigation:**
- Follow Component Model spec closely on GitHub
- Subscribe to specification change notifications
- Pin WIT version initially (wit-bindgen = "0.16")
- Abstract WIT details behind Rust types for easier migration
- Test against specification examples regularly

**Contingency Plan:**
- If breaking changes occur, update incrementally
- Maintain compatibility layer for old versions
- Document migration path for components

---

### Risk 2: Permission Annotation Complexity

**Mitigation:**
- Keep annotation syntax simple (TOML-based)
- Provide extensive examples in documentation
- Create validation tooling with clear error messages
- Document common patterns (templates)
- Collect feedback from early users

**Contingency Plan:**
- If too complex, simplify to predefined permission sets
- Provide configuration generator tool
- Offer consulting/support for complex use cases

---

### Risk 3: Binding Generation Issues

**Mitigation:**
- Test binding generation early (Phase 4 Day 1)
- Have fallback manual bindings ready
- Monitor wit-bindgen issues on GitHub
- Contribute fixes upstream if needed
- Document binding generation process thoroughly

**Contingency Plan:**
- If wit-bindgen fails, implement manual bindings
- Use wasmtime API directly as fallback
- Delay SDK release until bindings stable

---

### Risk 4: Interface Design Mistakes

**Mitigation:**
- Review against KNOWLEDGE-WASM-004 patterns
- Study existing WASM interface designs (WASI Preview 2)
- Iterate on interfaces early (Phase 1)
- Collect feedback from stakeholders
- Plan for versioning and evolution from Day 1

**Contingency Plan:**
- If major design flaw found, create v2 interfaces
- Provide migration tooling
- Support old interfaces during transition period
- Document lessons learned

---

### Risk 5: Multi-Language Foundation Gaps

**Mitigation:**
- Design interfaces without Rust-specific assumptions
- Consult multi-language examples (wit-bindgen docs)
- Document language-neutral patterns
- Avoid Rust-specific WIT features
- Plan for future language bindings

**Contingency Plan:**
- If gaps found, create compatibility layer
- Add language-specific extensions in Phase 2
- Maintain core interfaces language-neutral

---

## 9. Follow-Up Tasks

### Immediate Follow-Up (WASM-TASK-004: Block 3)

**What Block 2 Provides for Block 3:**
- Complete WIT interfaces for actor integration
- Component lifecycle interface with messaging
- Host services for component discovery
- Messaging patterns (send-message, send-request)
- Permission system for actor isolation

**Integration Points Prepared:**
- `handle-message` interface ready for actor message delivery
- `handle-callback` interface ready for request-response pattern
- `list-components` / `get-component-metadata` for actor discovery
- Component permission isolation enforced

**Documentation Handoff:**
- Component development guide includes messaging patterns
- Actor model integration documented
- Inter-component communication examples

---

### Documentation to Update

**Memory Bank Updates:**

1. **progress.md:**
   - Mark Block 2 (WASM-TASK-003) complete (100%)
   - Update overall completion percentage
   - Note Block 3 dependencies satisfied

2. **tasks/_index.md:**
   - Move WASM-TASK-003 to completed tasks section
   - Update completion statistics (4 of 13 complete)

3. **Create Completion Summary:**
   - File: `tasks/task_003_completion_summary.md`
   - Document all deliverables
   - Include test coverage metrics
   - Include performance benchmark results
   - Document lessons learned

4. **Knowledge Documentation:**
   - File: `docs/knowledges/knowledge_wasm_020_wit_interface_patterns.md`
   - Document WIT interface design patterns discovered
   - Document permission system architecture insights
   - Document binding generation best practices

5. **ADRs (if architectural decisions made):**
   - Create ADR if significant architectural decisions emerged
   - Example: ADR for permission pattern syntax choice

---

### External Documentation

**README Updates:**

1. **airssys-wasm/README.md:**
   - Add WIT interface system section
   - Link to API reference documentation
   - Add component development quick start

2. **airssys-wasm-component-sdk/README.md:**
   - Complete SDK documentation
   - Getting started guide
   - Example components reference
   - Capability declaration guide

**Public Documentation:**

1. **Website/GitHub Pages:**
   - Publish mdBook documentation
   - API reference searchable
   - Component development tutorials

2. **Repository:**
   - Update repository description
   - Add component development badges
   - Link to published documentation

---

## 10. Additional Notes

### Development Environment Setup

**Required Tools:**
```bash
# Rust with WASM target
rustup target add wasm32-wasi

# WIT tooling
cargo install wasm-tools

# mdBook for documentation
cargo install mdbook

# Component testing (future)
# cargo install airssys-wasm-cli  (Block 9)
```

### Code Quality Standards

**Pre-Commit Checklist:**
```bash
# Format code
cargo fmt --all

# Lint code
cargo clippy --workspace --all-targets --all-features

# Run tests
cargo test --workspace

# Validate WIT
wasm-tools component wit airssys-wasm/wit/

# Build documentation
mdbook build airssys-wasm/docs

# Test documentation
mdbook test airssys-wasm/docs
```

### Continuous Integration

**CI Pipeline Requirements:**
```yaml
# .github/workflows/wasm-task-003.yml
name: WASM-TASK-003 WIT Interface System

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-wasi
      
      # Validate WIT files
      - name: Install wasm-tools
        run: cargo install wasm-tools
      
      - name: Validate WIT
        run: wasm-tools component wit airssys-wasm/wit/
      
      # Run tests
      - name: Run tests
        run: cargo test --workspace
      
      # Build examples
      - name: Build SDK examples
        run: |
          cd airssys-wasm-component-sdk
          cargo build --target wasm32-wasi --examples --release
      
      # Build documentation
      - name: Build docs
        run: |
          cargo install mdbook
          mdbook build airssys-wasm/docs
      
      # Run benchmarks
      - name: Run benchmarks
        run: cargo bench -p airssys-wasm --no-run
```

### Performance Monitoring

**Baseline Metrics to Track:**
- Interface validation time (<1ms target)
- Permission check time (<5μs target)
- Binding overhead (<1μs target)
- Component loading time with validation
- Memory overhead per component
- WASM binary size for SDK examples

**Regression Detection:**
- Run benchmarks in CI
- Compare against baseline metrics
- Alert on >10% performance regression
- Document performance changes in PRs

---

## Summary

This comprehensive implementation plan provides:

- **Detailed phase breakdowns** with actionable sub-tasks (42 days / 3-4 weeks)
- **Concrete WIT interface specifications** ready to implement
- **Complete Rust implementation strategy** with permission system
- **Integration points** with Block 1 runtime
- **Comprehensive testing approach** (>90% coverage target)
- **Risk mitigation strategies** for all identified risks
- **Clear success validation** criteria and completion checklist
- **Timeline with weekly milestones** and checkpoints
- **Follow-up task preparation** for Block 3

**Implementation can begin immediately** with clear understanding of:
- What to build (WIT interfaces, permission system, SDK)
- How to build it (step-by-step implementation)
- Where to integrate it (Block 1 component loading)
- How to test it (unit, integration, performance)
- When it's done (completion criteria and validation)

**Next Step:** Begin Phase 1, Task 1.1 - WIT Project Structure Setup.
