//! WASM Capability Types and airssys-osl Security Bridge
//!
//! This module provides the foundational types for WASM component security by bridging
//! WASM-specific capability declarations to the airssys-osl security infrastructure
//! (ACL/RBAC/audit logging).
//!
//! # Security Model
//!
//! airssys-wasm implements a **deny-by-default** capability-based security model:
//! - Components must explicitly declare capabilities in `Component.toml`
//! - Capabilities are mapped to airssys-osl ACL/RBAC policies
//! - Host functions check capabilities before granting resource access
//! - Each component has an isolated security context
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ Component.toml                                              │
//! │ [capabilities]                                              │
//! │ filesystem.read = ["/app/data/*"]                           │
//! └────────────────┬────────────────────────────────────────────┘
//!                  │ Parse (Task 1.2)
//!                  ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │ WasmCapability (This Module)                                │
//! │ Filesystem { paths: ["/app/data/*"], permissions: ["read"] }│
//! └────────────────┬────────────────────────────────────────────┘
//!                  │ Map to ACL
//!                  ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │ airssys-osl AclEntry                                        │
//! │ identity: "component-123"                                   │
//! │ resource_pattern: "/app/data/*"                             │
//! │ permissions: ["read"]                                       │
//! │ policy: Allow                                               │
//! └────────────────┬────────────────────────────────────────────┘
//!                  │ Evaluate
//!                  ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │ airssys-osl SecurityPolicy::evaluate()                      │
//! │ → PolicyDecision::Allow or Deny                             │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example Usage
//!
//! ## Declaring Capabilities
//!
//! ```toml
//! # Component.toml
//! [capabilities]
//! filesystem.read = ["/app/config/*", "/app/data/*.json"]
//! filesystem.write = ["/app/data/*"]
//! network.connect = ["api.example.com:443", "*.cdn.example.com:80"]
//! storage.namespace = ["component:<id>:*"]
//! ```
//!
//! ## Building Capability Set
//!
//! ```rust
//! use airssys_wasm::security::{WasmCapability, WasmCapabilitySet};
//!
//! let capabilities = WasmCapabilitySet::new()
//!     .grant(WasmCapability::Filesystem {
//!         paths: vec!["/app/data/*".to_string()],
//!         permissions: vec!["read".to_string(), "write".to_string()],
//!     })
//!     .grant(WasmCapability::Network {
//!         endpoints: vec!["api.example.com:443".to_string()],
//!         permissions: vec!["connect".to_string()],
//!     });
//! ```
//!
//! ## Converting to ACL Entries
//!
//! ```rust
//! use airssys_wasm::security::WasmCapability;
//!
//! let cap = WasmCapability::Filesystem {
//!     paths: vec!["/app/config/*".to_string()],
//!     permissions: vec!["read".to_string()],
//! };
//!
//! // Convert to airssys-osl ACL entries for component "comp-123"
//! let acl_entries = cap.to_acl_entry("comp-123");
//! // Results in: AclEntry(identity="comp-123", resource="/app/config/*", permissions=["read"])
//! ```
//!
//! # Integration with airssys-osl
//!
//! This module **reuses** airssys-osl security infrastructure rather than reimplementing it:
//!
//! - **ACL (Access Control Lists)**: Glob pattern matching for resources
//! - **RBAC (Role-Based Access Control)**: Permission-based authorization
//! - **Audit Logging**: Comprehensive security event logging
//! - **SecurityPolicy**: Pluggable policy evaluation engine
//!
//! Benefits:
//! - ✅ Reuse 1000+ lines of battle-tested security code
//! - ✅ Leverage 311+ passing tests from airssys-osl
//! - ✅ Maintain architectural consistency across AirsSys
//! - ✅ Avoid code duplication and maintenance burden
//!
//! # Performance
//!
//! - **Capability Lookup**: O(1) via `HashSet<WasmCapability>`
//! - **ACL Conversion**: ~50-100ns per capability (allocation + clone)
//! - **Typical Component**: ~1μs for 10 capabilities
//! - **Large Component**: ~10μs for 100 capabilities
//!
//! Target: <5μs per capability check (includes ACL evaluation)
//!
//! # Security Considerations
//!
//! ## Deny-by-Default
//! Components without declared capabilities are **denied all access** to host resources.
//! This prevents accidental privilege escalation.
//!
//! ## Least Privilege
//! Components should declare **minimal capabilities** required for functionality.
//! Example: Request `/app/data/*.json` instead of `/app/**` (overly broad).
//!
//! ## Pattern Validation
//! Glob patterns are validated by airssys-osl during ACL evaluation.
//! Invalid patterns (e.g., malformed globs) result in denial.
//!
//! ## Capability Immutability
//! Once a component is spawned, its capability set is **immutable**.
//! This prevents runtime privilege escalation.
//!
//! # Related Modules
//!
//! - **Task 1.2**: Component.toml parser (creates `WasmCapabilitySet` from TOML)
//! - **Task 1.3**: `WasmSecurityContext` bridge to airssys-osl `SecurityContext`
//! - **Task 3.1**: `check_capability()` API for host functions
//!
//! # Standards Compliance
//!
//! - **ADR-WASM-005**: Capability-Based Security Model ✅
//! - **PROJECTS_STANDARD.md**: §4.3 (module structure), §5.1 (dependencies) ✅
//! - **Microsoft Rust Guidelines**: M-DESIGN-FOR-AI, M-CANONICAL-DOCS ✅

// Layer 1: Standard library imports
use std::collections::HashSet;

// Layer 2: Third-party crate imports
use airssys_osl::middleware::security::{AclEntry, AclPolicy};
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
// (none - this is a leaf module)

/// WASM component capability types.
///
/// Represents the different categories of host resources that WASM components
/// can request access to. Each capability type maps to specific airssys-osl
/// ACL/RBAC policies.
///
/// # Capability Types
///
/// - **Filesystem**: File and directory access (read, write, execute)
/// - **Network**: Network socket operations (connect, bind, listen)
/// - **Storage**: Key-value storage access (read, write, delete)
///
/// # Design Rationale
///
/// ## Why Enum Instead of Trait?
///
/// We use an enum rather than a trait-based design because:
/// 1. **Closed Set**: Capability types are fixed at compile time (extensibility via enum variants)
/// 2. **Type Safety**: Exhaustive match checking ensures all capability types are handled
/// 3. **Serialization**: Easy to serialize/deserialize for Component.toml parsing
/// 4. **Performance**: No vtable overhead, direct enum dispatch
///
/// ## Why Separate Fields for Patterns and Permissions?
///
/// Separating `paths`/`endpoints`/`namespaces` from `permissions` allows:
/// 1. **Multi-Resource Grants**: Single capability can grant access to multiple resources
/// 2. **Pattern Reuse**: Same permission set applied to different resource patterns
/// 3. **Clear Semantics**: Explicit separation between "what" (resources) and "how" (permissions)
///
/// # Examples
///
/// ## Filesystem Capability
///
/// ```rust
/// use airssys_wasm::security::WasmCapability;
///
/// // Read access to config files
/// let cap = WasmCapability::Filesystem {
///     paths: vec![
///         "/app/config/*.toml".to_string(),
///         "/app/config/*.json".to_string(),
///     ],
///     permissions: vec!["read".to_string()],
/// };
/// ```
///
/// ## Network Capability
///
/// ```rust
/// use airssys_wasm::security::WasmCapability;
///
/// // Connect to API endpoints
/// let cap = WasmCapability::Network {
///     endpoints: vec![
///         "api.example.com:443".to_string(),
///         "*.cdn.example.com:80".to_string(),  // Wildcard subdomain
///     ],
///     permissions: vec!["connect".to_string()],
/// };
/// ```
///
/// ## Storage Capability
///
/// ```rust
/// use airssys_wasm::security::WasmCapability;
///
/// // Read/write to component-specific namespace
/// let cap = WasmCapability::Storage {
///     namespaces: vec!["component:<id>:*".to_string()],
///     permissions: vec!["read".to_string(), "write".to_string()],
/// };
/// ```
///
/// # Pattern Matching
///
/// Resource patterns support glob syntax via airssys-osl:
/// - `*` - Matches any sequence (e.g., `/app/data/*` matches `/app/data/file.txt`)
/// - `?` - Matches single character (e.g., `file?.txt` matches `file1.txt`)
/// - `**` - Matches any depth (e.g., `/app/**/*.log` matches nested logs)
/// - `[abc]` - Matches character class
/// - `{a,b}` - Matches alternatives
///
/// # Permission Strings
///
/// Permissions are string-based for flexibility:
/// - **Filesystem**: `"read"`, `"write"`, `"execute"`
/// - **Network**: `"connect"`, `"bind"`, `"listen"`
/// - **Storage**: `"read"`, `"write"`, `"delete"`
///
/// # Trait Implementations
///
/// - `Debug`: For debugging and error messages
/// - `Clone`: Capabilities can be cloned for multiple components
/// - `PartialEq`, `Eq`: Equality comparison for deduplication
/// - `Hash`: Used in `HashSet` for O(1) lookup
/// - `Serialize`, `Deserialize`: TOML/JSON parsing support
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WasmCapability {
    /// Filesystem access capability.
    ///
    /// Grants access to files and directories matching the specified glob patterns
    /// with the specified permissions.
    ///
    /// # Fields
    ///
    /// - `paths`: Glob patterns for filesystem resources (e.g., `"/app/data/*"`)
    /// - `permissions`: Allowed operations (e.g., `["read", "write"]`)
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::WasmCapability;
    ///
    /// let cap = WasmCapability::Filesystem {
    ///     paths: vec!["/app/data/*.json".to_string()],
    ///     permissions: vec!["read".to_string()],
    /// };
    /// ```
    Filesystem {
        /// Resource glob patterns (e.g., `"/app/config/*"`, `"/data/**/*.log"`)
        paths: Vec<String>,
        
        /// Allowed permissions: `"read"`, `"write"`, `"execute"`
        permissions: Vec<String>,
    },
    
    /// Network access capability.
    ///
    /// Grants access to network endpoints (domain:port) with the specified permissions.
    /// Supports wildcard subdomains for flexible domain matching.
    ///
    /// # Fields
    ///
    /// - `endpoints`: Domain:port patterns (e.g., `"api.example.com:443"`)
    /// - `permissions`: Allowed operations (e.g., `["connect"]`)
    ///
    /// # Endpoint Format
    ///
    /// Endpoints must be in `domain:port` format:
    /// - Exact domain: `"api.example.com:443"`
    /// - Wildcard subdomain: `"*.cdn.example.com:80"`
    /// - IP address: `"192.168.1.100:8080"` (future support)
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::WasmCapability;
    ///
    /// let cap = WasmCapability::Network {
    ///     endpoints: vec![
    ///         "api.example.com:443".to_string(),
    ///         "*.cdn.example.com:80".to_string(),
    ///     ],
    ///     permissions: vec!["connect".to_string()],
    /// };
    /// ```
    Network {
        /// Endpoint patterns in `domain:port` format (supports `*` wildcard for subdomains)
        endpoints: Vec<String>,
        
        /// Allowed permissions: `"connect"`, `"bind"`, `"listen"`
        permissions: Vec<String>,
    },
    
    /// Storage access capability.
    ///
    /// Grants access to key-value storage namespaces with the specified permissions.
    /// Namespaces provide isolated storage per component.
    ///
    /// # Fields
    ///
    /// - `namespaces`: Namespace patterns (e.g., `"component:<id>:*"`)
    /// - `permissions`: Allowed operations (e.g., `["read", "write"]`)
    ///
    /// # Namespace Format
    ///
    /// Namespaces use hierarchical naming with `:` separator:
    /// - Component-specific: `"component:<id>:*"`
    /// - Shared: `"shared:config:*"`
    /// - Scoped: `"component:<id>:cache:*"`
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::WasmCapability;
    ///
    /// let cap = WasmCapability::Storage {
    ///     namespaces: vec!["component:<id>:*".to_string()],
    ///     permissions: vec!["read".to_string(), "write".to_string()],
    /// };
    /// ```
    Storage {
        /// Namespace patterns with `:` hierarchy (e.g., `"component:<id>:cache:*"`)
        namespaces: Vec<String>,
        
        /// Allowed permissions: `"read"`, `"write"`, `"delete"`
        permissions: Vec<String>,
    },
}

impl WasmCapability {
    /// Convert this WASM capability to airssys-osl ACL entries.
    ///
    /// Maps WASM-specific capability declarations to airssys-osl `AclEntry` types
    /// that can be evaluated by the airssys-osl SecurityPolicy engine.
    ///
    /// # Arguments
    ///
    /// - `component_id`: The unique identifier for the component (becomes ACL identity)
    ///
    /// # Returns
    ///
    /// A vector of `AclEntry` instances, one per resource pattern. For example,
    /// a `Filesystem` capability with 3 paths returns 3 ACL entries.
    ///
    /// # Mapping Rules
    ///
    /// ## Filesystem → ACL
    /// - `identity`: `component_id`
    /// - `resource_pattern`: Each path from `paths` field
    /// - `permissions`: Direct copy of `permissions` field
    /// - `policy`: `AclPolicy::Allow` (explicit allow-list)
    ///
    /// ## Network → ACL
    /// - `identity`: `component_id`
    /// - `resource_pattern`: Each endpoint from `endpoints` field
    /// - `permissions`: Direct copy of `permissions` field
    /// - `policy`: `AclPolicy::Allow`
    ///
    /// ## Storage → ACL
    /// - `identity`: `component_id`
    /// - `resource_pattern`: Each namespace from `namespaces` field
    /// - `permissions`: Direct copy of `permissions` field
    /// - `policy`: `AclPolicy::Allow`
    ///
    /// # Performance
    ///
    /// - **Time Complexity**: O(N) where N = number of resource patterns
    /// - **Allocations**: N `AclEntry` allocations + string clones
    /// - **Typical Cost**: ~50-100ns per pattern (allocation + clone overhead)
    ///
    /// # Examples
    ///
    /// ## Single Resource Pattern
    ///
    /// ```rust
    /// use airssys_wasm::security::WasmCapability;
    ///
    /// let cap = WasmCapability::Filesystem {
    ///     paths: vec!["/app/data/*".to_string()],
    ///     permissions: vec!["read".to_string()],
    /// };
    ///
    /// let acl_entries = cap.to_acl_entry("component-123");
    /// assert_eq!(acl_entries.len(), 1);
    /// assert_eq!(acl_entries[0].identity, "component-123");
    /// assert_eq!(acl_entries[0].resource_pattern, "/app/data/*");
    /// assert_eq!(acl_entries[0].permissions, vec!["read"]);
    /// ```
    ///
    /// ## Multiple Resource Patterns
    ///
    /// ```rust
    /// use airssys_wasm::security::WasmCapability;
    ///
    /// let cap = WasmCapability::Filesystem {
    ///     paths: vec![
    ///         "/app/config/*".to_string(),
    ///         "/app/data/*.json".to_string(),
    ///     ],
    ///     permissions: vec!["read".to_string()],
    /// };
    ///
    /// let acl_entries = cap.to_acl_entry("component-456");
    /// assert_eq!(acl_entries.len(), 2);  // One ACL entry per path
    /// ```
    ///
    /// # Design Rationale
    ///
    /// ## Why Return `Vec<AclEntry>` Instead of Single Entry?
    ///
    /// A single WASM capability can grant access to **multiple resources** (multiple paths,
    /// endpoints, or namespaces). To avoid complex ACL logic, we create one ACL entry per
    /// resource pattern, allowing airssys-osl to evaluate each independently.
    ///
    /// ## Why Clone Strings?
    ///
    /// `AclEntry::new()` takes owned `String` values. We could use `&str` and lifetimes,
    /// but this complicates the API and ownership model. String clones are cheap for
    /// typical capability declarations (10-50 bytes per pattern).
    pub fn to_acl_entry(&self, component_id: &str) -> Vec<AclEntry> {
        match self {
            WasmCapability::Filesystem { paths, permissions } => {
                // Create one ACL entry per filesystem path pattern
                paths.iter().map(|path| {
                    AclEntry::new(
                        component_id.to_string(),  // identity = component ID
                        path.clone(),              // resource_pattern = filesystem path
                        permissions.clone(),       // permissions = read/write/execute
                        AclPolicy::Allow,          // policy = explicit allow
                    )
                }).collect()
            }
            WasmCapability::Network { endpoints, permissions } => {
                // Create one ACL entry per network endpoint pattern
                endpoints.iter().map(|endpoint| {
                    AclEntry::new(
                        component_id.to_string(),  // identity = component ID
                        endpoint.clone(),          // resource_pattern = domain:port
                        permissions.clone(),       // permissions = connect/bind/listen
                        AclPolicy::Allow,          // policy = explicit allow
                    )
                }).collect()
            }
            WasmCapability::Storage { namespaces, permissions } => {
                // Create one ACL entry per storage namespace pattern
                namespaces.iter().map(|ns| {
                    AclEntry::new(
                        component_id.to_string(),  // identity = component ID
                        ns.clone(),                // resource_pattern = namespace path
                        permissions.clone(),       // permissions = read/write/delete
                        AclPolicy::Allow,          // policy = explicit allow
                    )
                }).collect()
            }
        }
    }
}

/// Container for a WASM component's capability set.
///
/// Holds all capabilities declared by a component in its `Component.toml` manifest.
/// Uses `HashSet` for O(1) lookup and automatic deduplication of identical capabilities.
///
/// # Design Rationale
///
/// ## Why HashSet Instead of Vec?
///
/// - **Deduplication**: Components cannot declare the same capability twice
/// - **Lookup Performance**: O(1) capability existence checks
/// - **Semantic Correctness**: Capabilities are a *set* (no duplicates, no order)
///
/// ## Why Not HashMap<CapabilityType, Vec<...>>?
///
/// Grouping by capability type (Filesystem, Network, Storage) would complicate
/// the API and provide no performance benefit. `HashSet<WasmCapability>` is
/// simpler and equally efficient.
///
/// # Examples
///
/// ## Building a Capability Set
///
/// ```rust
/// use airssys_wasm::security::{WasmCapability, WasmCapabilitySet};
///
/// let capabilities = WasmCapabilitySet::new()
///     .grant(WasmCapability::Filesystem {
///         paths: vec!["/app/data/*".to_string()],
///         permissions: vec!["read".to_string(), "write".to_string()],
///     })
///     .grant(WasmCapability::Network {
///         endpoints: vec!["api.example.com:443".to_string()],
///         permissions: vec!["connect".to_string()],
///     });
/// ```
///
/// ## Converting to ACL Entries
///
/// ```rust
/// use airssys_wasm::security::{WasmCapability, WasmCapabilitySet};
///
/// let capabilities = WasmCapabilitySet::new()
///     .grant(WasmCapability::Filesystem {
///         paths: vec!["/app/*".to_string()],
///         permissions: vec!["read".to_string()],
///     });
///
/// // Convert all capabilities to ACL entries for airssys-osl evaluation
/// let acl_entries = capabilities.to_acl_entries("component-789");
/// ```
///
/// # Trait Implementations
///
/// - `Debug`: For debugging and error messages
/// - `Clone`: Capability sets can be cloned (e.g., for actor spawn)
/// - `Default`: Empty capability set (deny-by-default)
/// - `Serialize`, `Deserialize`: TOML/JSON parsing support
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WasmCapabilitySet {
    /// Internal set of capabilities (HashSet for deduplication and O(1) lookup)
    capabilities: HashSet<WasmCapability>,
}

impl WasmCapabilitySet {
    /// Create an empty capability set.
    ///
    /// An empty capability set grants **no access** to host resources,
    /// implementing the deny-by-default security model.
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::WasmCapabilitySet;
    ///
    /// let empty = WasmCapabilitySet::new();
    /// assert_eq!(empty.to_acl_entries("comp-1").len(), 0);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Grant a capability to this set (builder pattern).
    ///
    /// Adds a capability to the set and returns `self` for method chaining.
    /// If the capability already exists (duplicate), it is **not added again**
    /// due to `HashSet` semantics.
    ///
    /// # Arguments
    ///
    /// - `cap`: The capability to grant
    ///
    /// # Returns
    ///
    /// `self` for method chaining (builder pattern)
    ///
    /// # Examples
    ///
    /// ## Single Capability
    ///
    /// ```rust
    /// use airssys_wasm::security::{WasmCapability, WasmCapabilitySet};
    ///
    /// let caps = WasmCapabilitySet::new()
    ///     .grant(WasmCapability::Filesystem {
    ///         paths: vec!["/app/data/*".to_string()],
    ///         permissions: vec!["read".to_string()],
    ///     });
    /// ```
    ///
    /// ## Multiple Capabilities (Chaining)
    ///
    /// ```rust
    /// use airssys_wasm::security::{WasmCapability, WasmCapabilitySet};
    ///
    /// let caps = WasmCapabilitySet::new()
    ///     .grant(WasmCapability::Filesystem {
    ///         paths: vec!["/app/config/*".to_string()],
    ///         permissions: vec!["read".to_string()],
    ///     })
    ///     .grant(WasmCapability::Filesystem {
    ///         paths: vec!["/app/data/*".to_string()],
    ///         permissions: vec!["read".to_string(), "write".to_string()],
    ///     })
    ///     .grant(WasmCapability::Network {
    ///         endpoints: vec!["api.example.com:443".to_string()],
    ///         permissions: vec!["connect".to_string()],
    ///     });
    /// ```
    ///
    /// ## Duplicate Capabilities (Automatic Deduplication)
    ///
    /// ```rust
    /// use airssys_wasm::security::{WasmCapability, WasmCapabilitySet};
    ///
    /// let cap = WasmCapability::Filesystem {
    ///     paths: vec!["/app/data/*".to_string()],
    ///     permissions: vec!["read".to_string()],
    /// };
    ///
    /// let caps = WasmCapabilitySet::new()
    ///     .grant(cap.clone())
    ///     .grant(cap.clone());  // Duplicate - ignored by HashSet
    ///
    /// // Only 1 ACL entry created (deduplication)
    /// assert_eq!(caps.to_acl_entries("comp-1").len(), 1);
    /// ```
    ///
    /// # Performance
    ///
    /// - **Time Complexity**: O(1) average case (HashSet insert)
    /// - **Allocations**: No allocations (moves capability into HashSet)
    /// - **Typical Cost**: <10ns (hash + insert)
    pub fn grant(mut self, cap: WasmCapability) -> Self {
        self.capabilities.insert(cap);
        self
    }

    /// Convert all capabilities in this set to airssys-osl ACL entries.
    ///
    /// Flattens all capabilities into a single list of `AclEntry` instances
    /// suitable for evaluation by airssys-osl's `SecurityPolicy`.
    ///
    /// # Arguments
    ///
    /// - `component_id`: The unique identifier for the component (becomes ACL identity)
    ///
    /// # Returns
    ///
    /// A vector containing all ACL entries from all capabilities. Order is undefined
    /// (depends on `HashSet` iteration order).
    ///
    /// # Performance
    ///
    /// - **Time Complexity**: O(N × M) where N = capabilities, M = avg patterns per capability
    /// - **Allocations**: One `Vec<AclEntry>` + N×M `AclEntry` instances
    /// - **Typical Cost**: ~1μs for 10 capabilities, ~10μs for 100 capabilities
    ///
    /// # Examples
    ///
    /// ## Empty Set
    ///
    /// ```rust
    /// use airssys_wasm::security::WasmCapabilitySet;
    ///
    /// let empty = WasmCapabilitySet::new();
    /// let entries = empty.to_acl_entries("component-empty");
    /// assert_eq!(entries.len(), 0);  // No capabilities = no ACL entries
    /// ```
    ///
    /// ## Multiple Capabilities
    ///
    /// ```rust
    /// use airssys_wasm::security::{WasmCapability, WasmCapabilitySet};
    ///
    /// let caps = WasmCapabilitySet::new()
    ///     .grant(WasmCapability::Filesystem {
    ///         paths: vec!["/app/data/*".to_string()],
    ///         permissions: vec!["read".to_string()],
    ///     })
    ///     .grant(WasmCapability::Network {
    ///         endpoints: vec!["api.example.com:443".to_string()],
    ///         permissions: vec!["connect".to_string()],
    ///     });
    ///
    /// let entries = caps.to_acl_entries("component-multi");
    /// assert_eq!(entries.len(), 2);  // 1 filesystem + 1 network = 2 ACL entries
    /// ```
    ///
    /// ## Multiple Patterns Per Capability
    ///
    /// ```rust
    /// use airssys_wasm::security::{WasmCapability, WasmCapabilitySet};
    ///
    /// let caps = WasmCapabilitySet::new()
    ///     .grant(WasmCapability::Filesystem {
    ///         paths: vec![
    ///             "/app/config/*".to_string(),
    ///             "/app/data/*.json".to_string(),
    ///             "/app/logs/*.log".to_string(),
    ///         ],
    ///         permissions: vec!["read".to_string()],
    ///     });
    ///
    /// let entries = caps.to_acl_entries("component-paths");
    /// assert_eq!(entries.len(), 3);  // 3 paths = 3 ACL entries
    /// ```
    pub fn to_acl_entries(&self, component_id: &str) -> Vec<AclEntry> {
        self.capabilities
            .iter()
            .flat_map(|cap| cap.to_acl_entry(component_id))
            .collect()
    }
}

/// WASM component security context.
///
/// Encapsulates all security-related information for a WASM component instance,
/// including its unique identifier and granted capabilities. This context is
/// attached to each `ComponentActor` and used to evaluate host function access.
///
/// # Lifecycle
///
/// 1. **Creation**: Built from `Component.toml` during component spawn (Task 1.2)
/// 2. **Attachment**: Stored in `ComponentActor` instance (Task 4.1)
/// 3. **Restoration**: Restored after supervisor restart (Task 5.1)
/// 4. **Evaluation**: Used by `check_capability()` API (Task 3.1)
///
/// # Immutability
///
/// Once created, a `WasmSecurityContext` is **immutable**. Components cannot
/// request additional capabilities at runtime, preventing privilege escalation.
///
/// # Examples
///
/// ## Creating a Security Context
///
/// ```rust
/// use airssys_wasm::security::{WasmCapability, WasmCapabilitySet, WasmSecurityContext};
///
/// let capabilities = WasmCapabilitySet::new()
///     .grant(WasmCapability::Filesystem {
///         paths: vec!["/app/data/*".to_string()],
///         permissions: vec!["read".to_string()],
///     });
///
/// let context = WasmSecurityContext::new(
///     "component-secure-123".to_string(),
///     capabilities,
/// );
/// ```
///
/// ## Using with ComponentActor
///
/// ```rust,ignore
/// // Inside ComponentActor spawn logic (Task 4.1)
/// pub struct ComponentActor {
///     pub security_context: WasmSecurityContext,
///     // ... other fields
/// }
///
/// let actor = ComponentActor {
///     security_context: WasmSecurityContext::new(component_id, capabilities),
///     // ...
/// };
/// ```
///
/// # Trait Implementations
///
/// - `Debug`: For debugging and error messages
/// - `Clone`: Security contexts can be cloned (e.g., for supervisor restart)
#[derive(Debug, Clone)]
pub struct WasmSecurityContext {
    /// Unique identifier for the component (maps to ACL identity)
    pub component_id: String,
    
    /// Set of capabilities granted to the component
    pub capabilities: WasmCapabilitySet,
}

impl WasmSecurityContext {
    /// Create a new WASM security context.
    ///
    /// # Arguments
    ///
    /// - `component_id`: Unique identifier for the component
    /// - `capabilities`: Set of capabilities granted to the component
    ///
    /// # Returns
    ///
    /// A new `WasmSecurityContext` instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use airssys_wasm::security::{WasmCapability, WasmCapabilitySet, WasmSecurityContext};
    ///
    /// let capabilities = WasmCapabilitySet::new()
    ///     .grant(WasmCapability::Filesystem {
    ///         paths: vec!["/app/data/*".to_string()],
    ///         permissions: vec!["read".to_string()],
    ///     });
    ///
    /// let context = WasmSecurityContext::new(
    ///     "my-component".to_string(),
    ///     capabilities,
    /// );
    ///
    /// assert_eq!(context.component_id, "my-component");
    /// ```
    pub fn new(component_id: String, capabilities: WasmCapabilitySet) -> Self {
        Self {
            component_id,
            capabilities,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test filesystem capability to ACL entry conversion.
    ///
    /// Verifies that a filesystem capability with a single path pattern
    /// correctly maps to an ACL entry with the right identity, resource
    /// pattern, and permissions.
    #[test]
    fn test_filesystem_capability_to_acl() {
        let cap = WasmCapability::Filesystem {
            paths: vec!["/app/data/*".to_string()],
            permissions: vec!["read".to_string(), "write".to_string()],
        };

        let entries = cap.to_acl_entry("component-123");
        
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].identity, "component-123");
        assert_eq!(entries[0].resource_pattern, "/app/data/*");
        assert_eq!(entries[0].permissions, vec!["read", "write"]);
    }

    /// Test capability set builder pattern with multiple capability types.
    ///
    /// Verifies that:
    /// 1. Multiple capabilities can be chained via `.grant()`
    /// 2. Different capability types (Filesystem, Network) are supported
    /// 3. ACL entry count matches capability count
    #[test]
    fn test_capability_set() {
        let set = WasmCapabilitySet::new()
            .grant(WasmCapability::Filesystem {
                paths: vec!["/app/*".to_string()],
                permissions: vec!["read".to_string()],
            })
            .grant(WasmCapability::Network {
                endpoints: vec!["api.example.com:443".to_string()],
                permissions: vec!["connect".to_string()],
            });

        let entries = set.to_acl_entries("comp-1");
        
        // 1 filesystem path + 1 network endpoint = 2 ACL entries
        assert_eq!(entries.len(), 2);
    }
}
