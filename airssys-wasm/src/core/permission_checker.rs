//! Permission checking and validation with pattern matching.
//!
//! This module implements runtime permission enforcement using glob pattern matching
//! for filesystem paths and wildcard matching for network domains.
//!
//! # Performance Characteristics
//!
//! - **Cached checks**: <50ns (LRU cache hit)
//! - **Uncached checks**: <5μs (pattern compilation + matching)
//! - **Cache size**: 1000 entries per component (configurable)
//!
//! # Permission Model
//!
//! Follows ADR-WASM-005 deny-by-default security:
//! - Component declares permissions in Component.toml
//! - Host loads and compiles permission patterns
//! - Runtime checks permissions at every host function call
//! - Cache improves performance for repeated checks
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust
//! use airssys_wasm::core::permission_checker::PermissionChecker;
//! use airssys_wasm::core::permission::PermissionManifest;
//! use airssys_wasm::core::component::ComponentId;
//!
//! // Create checker
//! let mut checker = PermissionChecker::new();
//!
//! // Load component permissions
//! let component_id = ComponentId::new("test-component");
//! let mut manifest = PermissionManifest::new();
//! manifest.filesystem.read.push("/data/**".to_string());
//!
//! checker.load_permissions(component_id.clone(), &manifest).unwrap();
//!
//! // Check permissions
//! assert!(checker.can_read_file(&component_id, "/data/input.txt").is_ok());
//! assert!(checker.can_read_file(&component_id, "/etc/passwd").is_err());
//! ```
//!
//! # References
//!
//! - **ADR-WASM-005**: Capability-Based Security Model (lines 459-536)
//! - **Performance Targets**: <50ns cached, <5μs uncached (ADR-WASM-005 line 540)

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Layer 2: Third-party crate imports
use glob::Pattern as GlobPattern;
use lru::LruCache;

// Layer 3: Internal module imports
use crate::core::component::ComponentId;
use crate::core::error::{WasmError, WasmResult};
use crate::core::permission::{NetworkEndpoint, PermissionManifest};

/// Permission checker with pattern matching and LRU caching.
///
/// This is the runtime enforcement mechanism for component permissions.
/// It compiles permission patterns at load time and checks them efficiently
/// at runtime using LRU caching for performance.
///
/// # Performance
///
/// - Pattern compilation: One-time cost at component load (<1ms)
/// - Cached permission checks: ~50ns (LRU cache hit)
/// - Uncached permission checks: ~1-5μs (pattern matching)
/// - Memory usage: ~100KB per component (patterns + cache)
///
/// # Thread Safety
///
/// PermissionChecker is thread-safe and can be shared across threads.
/// Internal cache uses Mutex for synchronization.
///
/// # Examples
///
/// ```rust
/// use airssys_wasm::core::permission_checker::PermissionChecker;
/// use airssys_wasm::core::permission::PermissionManifest;
/// use airssys_wasm::core::component::ComponentId;
///
/// let mut checker = PermissionChecker::new();
/// let id = ComponentId::new("my-component");
///
/// let mut perms = PermissionManifest::new();
/// perms.filesystem.read.push("/data/**".to_string());
/// perms.filesystem.write.push("/output/*.txt".to_string());
///
/// checker.load_permissions(id.clone(), &perms).unwrap();
///
/// // Filesystem checks
/// assert!(checker.can_read_file(&id, "/data/input.json").is_ok());
/// assert!(checker.can_write_file(&id, "/output/result.txt").is_ok());
/// assert!(checker.can_read_file(&id, "/etc/passwd").is_err());
/// ```
#[derive(Debug)]
pub struct PermissionChecker {
    /// Compiled permissions per component
    permissions: HashMap<ComponentId, CompiledPermissions>,

    /// LRU cache for permission checks (component_id + path → allowed)
    cache: Arc<Mutex<LruCache<CacheKey, bool>>>,

    /// Cache size (number of entries per component)
    cache_size: usize,
}

/// Compiled permission patterns for efficient matching.
///
/// Patterns are compiled once at component load time for better performance.
#[derive(Debug, Clone)]
struct CompiledPermissions {
    /// Compiled filesystem read patterns
    filesystem_read: Vec<GlobPattern>,

    /// Compiled filesystem write patterns
    filesystem_write: Vec<GlobPattern>,

    /// Compiled filesystem delete patterns
    filesystem_delete: Vec<GlobPattern>,

    /// Compiled filesystem list patterns
    filesystem_list: Vec<GlobPattern>,

    /// Network outbound endpoints (domain patterns + ports)
    network_outbound: Vec<NetworkEndpoint>,

    /// Network inbound ports
    network_inbound: Vec<u16>,

    /// Storage namespaces
    storage_namespaces: Vec<String>,

    /// Storage quota in bytes
    storage_quota: u64,
}

/// Cache key for LRU cache.
///
/// Combines component ID and resource path for efficient lookups.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    component_id: ComponentId,
    resource: String,
    action: PermissionAction,
}

/// Permission action types for cache differentiation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PermissionAction {
    FileRead,
    FileWrite,
    FileDelete,
    FileList,
    NetworkOutbound,
}

impl PermissionChecker {
    /// Create a new permission checker with default cache size.
    ///
    /// Default cache size: 1000 entries
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::permission_checker::PermissionChecker;
    ///
    /// let checker = PermissionChecker::new();
    /// ```
    pub fn new() -> Self {
        Self::with_cache_size(1000)
    }

    /// Create a permission checker with custom cache size.
    ///
    /// # Arguments
    ///
    /// * `cache_size` - Maximum number of cache entries (must be > 0)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::permission_checker::PermissionChecker;
    ///
    /// let checker = PermissionChecker::with_cache_size(5000);
    /// ```
    pub fn with_cache_size(cache_size: usize) -> Self {
        // Ensure cache_size is valid (> 0)
        let cache_size = cache_size.max(1);

        Self {
            permissions: HashMap::new(),
            cache: Arc::new(Mutex::new(LruCache::new(
                // SAFETY: cache_size is guaranteed to be >= 1 due to max(1) above
                unsafe { std::num::NonZeroUsize::new_unchecked(cache_size) },
            ))),
            cache_size,
        }
    }

    /// Get the configured cache size.
    ///
    /// Returns the maximum number of permission check results that can be cached.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::permission_checker::PermissionChecker;
    ///
    /// let checker = PermissionChecker::with_cache_size(5000);
    /// assert_eq!(checker.cache_size(), 5000);
    /// ```
    pub fn cache_size(&self) -> usize {
        self.cache_size
    }

    /// Load and compile permissions for a component.
    ///
    /// This should be called once during component loading.
    /// Patterns are compiled for efficient runtime checking.
    ///
    /// # Errors
    ///
    /// Returns error if any permission pattern is invalid.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::permission_checker::PermissionChecker;
    /// use airssys_wasm::core::permission::PermissionManifest;
    /// use airssys_wasm::core::component::ComponentId;
    ///
    /// let mut checker = PermissionChecker::new();
    /// let id = ComponentId::new("test-component");
    ///
    /// let mut perms = PermissionManifest::new();
    /// perms.filesystem.read.push("/data/**".to_string());
    ///
    /// checker.load_permissions(id, &perms).unwrap();
    /// ```
    pub fn load_permissions(
        &mut self,
        component_id: ComponentId,
        manifest: &PermissionManifest,
    ) -> WasmResult<()> {
        let compiled = self.compile_permissions(manifest)?;
        self.permissions.insert(component_id, compiled);
        Ok(())
    }

    /// Compile permission patterns for efficient matching.
    fn compile_permissions(
        &self,
        manifest: &PermissionManifest,
    ) -> WasmResult<CompiledPermissions> {
        // Compile filesystem patterns
        let filesystem_read = self.compile_glob_patterns(&manifest.filesystem.read)?;
        let filesystem_write = self.compile_glob_patterns(&manifest.filesystem.write)?;
        let filesystem_delete = self.compile_glob_patterns(&manifest.filesystem.delete)?;
        let filesystem_list = self.compile_glob_patterns(&manifest.filesystem.list)?;

        // Network permissions (no compilation needed, just clone)
        let network_outbound = manifest.network.outbound.clone();
        let network_inbound = manifest.network.inbound.clone();

        // Storage permissions
        let storage_namespaces = manifest.storage.namespaces.clone();
        let storage_quota = manifest.storage.max_size_mb * 1024 * 1024; // Convert MB to bytes

        Ok(CompiledPermissions {
            filesystem_read,
            filesystem_write,
            filesystem_delete,
            filesystem_list,
            network_outbound,
            network_inbound,
            storage_namespaces,
            storage_quota,
        })
    }

    /// Compile glob patterns from string patterns.
    fn compile_glob_patterns(&self, patterns: &[String]) -> WasmResult<Vec<GlobPattern>> {
        patterns
            .iter()
            .map(|pattern| {
                GlobPattern::new(pattern).map_err(|e| {
                    WasmError::component_validation_failed(format!(
                        "Invalid glob pattern '{pattern}': {e}"
                    ))
                })
            })
            .collect()
    }

    /// Check if component can read a file.
    ///
    /// # Errors
    ///
    /// Returns `CapabilityDenied` if component lacks permission.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::permission_checker::PermissionChecker;
    /// use airssys_wasm::core::permission::PermissionManifest;
    /// use airssys_wasm::core::component::ComponentId;
    ///
    /// let mut checker = PermissionChecker::new();
    /// let id = ComponentId::new("test");
    ///
    /// let mut perms = PermissionManifest::new();
    /// perms.filesystem.read.push("/data/**".to_string());
    /// checker.load_permissions(id.clone(), &perms).unwrap();
    ///
    /// assert!(checker.can_read_file(&id, "/data/input.txt").is_ok());
    /// assert!(checker.can_read_file(&id, "/etc/passwd").is_err());
    /// ```
    pub fn can_read_file(&self, component_id: &ComponentId, path: &str) -> WasmResult<()> {
        self.check_filesystem_permission(component_id, path, PermissionAction::FileRead, "read")
    }

    /// Check if component can write a file.
    ///
    /// # Errors
    ///
    /// Returns `CapabilityDenied` if component lacks permission.
    pub fn can_write_file(&self, component_id: &ComponentId, path: &str) -> WasmResult<()> {
        self.check_filesystem_permission(component_id, path, PermissionAction::FileWrite, "write")
    }

    /// Check if component can delete a file.
    ///
    /// # Errors
    ///
    /// Returns `CapabilityDenied` if component lacks permission.
    pub fn can_delete_file(&self, component_id: &ComponentId, path: &str) -> WasmResult<()> {
        self.check_filesystem_permission(component_id, path, PermissionAction::FileDelete, "delete")
    }

    /// Check if component can list a directory.
    ///
    /// # Errors
    ///
    /// Returns `CapabilityDenied` if component lacks permission.
    pub fn can_list_directory(&self, component_id: &ComponentId, path: &str) -> WasmResult<()> {
        self.check_filesystem_permission(component_id, path, PermissionAction::FileList, "list")
    }

    /// Check filesystem permission with caching.
    fn check_filesystem_permission(
        &self,
        component_id: &ComponentId,
        path: &str,
        action: PermissionAction,
        action_name: &str,
    ) -> WasmResult<()> {
        // Check cache first
        let cache_key = CacheKey {
            component_id: component_id.clone(),
            resource: path.to_string(),
            action,
        };

        {
            // Ignore poisoned mutex - cache is just a performance optimization
            if let Ok(mut cache) = self.cache.lock() {
                if let Some(&allowed) = cache.get(&cache_key) {
                    return if allowed {
                        Ok(())
                    } else {
                        Err(self.permission_denied_error(
                            component_id,
                            action_name,
                            path,
                            "cached denial",
                        ))
                    };
                }
            }
        }

        // Get component permissions
        let perms = self.permissions.get(component_id).ok_or_else(|| {
            WasmError::component_not_found(format!(
                "Component '{component_id:?}' not found in permission checker"
            ))
        })?;

        // Select patterns based on action
        let patterns = match action {
            PermissionAction::FileRead => &perms.filesystem_read,
            PermissionAction::FileWrite => &perms.filesystem_write,
            PermissionAction::FileDelete => &perms.filesystem_delete,
            PermissionAction::FileList => &perms.filesystem_list,
            _ => unreachable!(),
        };

        // Check if path matches any pattern
        let allowed = patterns.iter().any(|pattern| pattern.matches(path));

        // Cache result (ignore poisoned mutex - cache is just a performance optimization)
        if let Ok(mut cache) = self.cache.lock() {
            cache.put(cache_key, allowed);
        }

        if allowed {
            Ok(())
        } else {
            let pattern_count = patterns.len();
            Err(self.permission_denied_error(
                component_id,
                action_name,
                path,
                &format!("No matching pattern in {pattern_count} approved patterns"),
            ))
        }
    }

    /// Check if component can make outbound network connection.
    ///
    /// # Arguments
    ///
    /// * `component_id` - Component requesting permission
    /// * `host` - Target host domain or IP
    /// * `port` - Target port number
    ///
    /// # Errors
    ///
    /// Returns `CapabilityDenied` if component lacks permission.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::permission_checker::PermissionChecker;
    /// use airssys_wasm::core::permission::{PermissionManifest, NetworkEndpoint};
    /// use airssys_wasm::core::component::ComponentId;
    ///
    /// let mut checker = PermissionChecker::new();
    /// let id = ComponentId::new("test");
    ///
    /// let mut perms = PermissionManifest::new();
    /// perms.network.outbound.push(NetworkEndpoint {
    ///     host: "api.example.com".to_string(),
    ///     port: 443,
    /// });
    /// checker.load_permissions(id.clone(), &perms).unwrap();
    ///
    /// assert!(checker.can_connect_outbound(&id, "api.example.com", 443).is_ok());
    /// assert!(checker.can_connect_outbound(&id, "evil.com", 80).is_err());
    /// ```
    pub fn can_connect_outbound(
        &self,
        component_id: &ComponentId,
        host: &str,
        port: u16,
    ) -> WasmResult<()> {
        // Check cache
        let cache_key = CacheKey {
            component_id: component_id.clone(),
            resource: format!("{host}:{port}"),
            action: PermissionAction::NetworkOutbound,
        };

        {
            // Ignore poisoned mutex - cache is just a performance optimization
            if let Ok(mut cache) = self.cache.lock() {
                if let Some(&allowed) = cache.get(&cache_key) {
                    return if allowed {
                        Ok(())
                    } else {
                        Err(self.permission_denied_error(
                            component_id,
                            "network outbound",
                            &format!("{host}:{port}"),
                            "cached denial",
                        ))
                    };
                }
            }
        }

        // Get component permissions
        let perms = self.permissions.get(component_id).ok_or_else(|| {
            WasmError::component_not_found(format!(
                "Component '{component_id:?}' not found in permission checker"
            ))
        })?;

        // Check if endpoint matches any approved endpoint
        let allowed = perms.network_outbound.iter().any(|endpoint| {
            endpoint.port == port && self.matches_domain_pattern(&endpoint.host, host)
        });

        // Cache result (ignore poisoned mutex - cache is just a performance optimization)
        if let Ok(mut cache) = self.cache.lock() {
            cache.put(cache_key, allowed);
        }

        if allowed {
            Ok(())
        } else {
            let endpoint_count = perms.network_outbound.len();
            Err(self.permission_denied_error(
                component_id,
                "network outbound",
                &format!("{host}:{port}"),
                &format!("No matching endpoint in {endpoint_count} approved endpoints"),
            ))
        }
    }

    /// Check if host matches domain pattern.
    ///
    /// Supports:
    /// - Exact match: `"api.example.com"` matches `"api.example.com"`
    /// - Wildcard subdomain: `"*.example.com"` matches `"a.example.com"`, `"b.c.example.com"`
    fn matches_domain_pattern(&self, pattern: &str, host: &str) -> bool {
        if pattern == host {
            return true;
        }

        // Wildcard subdomain matching
        if let Some(base_domain) = pattern.strip_prefix("*.") {
            return host.ends_with(base_domain);
        }

        false
    }

    /// Check if component can accept inbound connection on port.
    ///
    /// # Arguments
    ///
    /// * `component_id` - Component requesting permission
    /// * `port` - Port number to listen on
    ///
    /// # Errors
    ///
    /// Returns `CapabilityDenied` if component lacks permission.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::permission_checker::PermissionChecker;
    /// use airssys_wasm::core::permission::PermissionManifest;
    /// use airssys_wasm::core::component::ComponentId;
    ///
    /// let mut checker = PermissionChecker::new();
    /// let id = ComponentId::new("test");
    ///
    /// let mut perms = PermissionManifest::new();
    /// perms.network.inbound.push(8080);
    /// checker.load_permissions(id.clone(), &perms).unwrap();
    ///
    /// assert!(checker.can_listen_inbound(&id, 8080).is_ok());
    /// assert!(checker.can_listen_inbound(&id, 80).is_err());
    /// ```
    pub fn can_listen_inbound(&self, component_id: &ComponentId, port: u16) -> WasmResult<()> {
        let perms = self.permissions.get(component_id).ok_or_else(|| {
            WasmError::component_not_found(format!(
                "Component '{component_id:?}' not found in permission checker"
            ))
        })?;

        if perms.network_inbound.contains(&port) {
            Ok(())
        } else {
            let port_count = perms.network_inbound.len();
            Err(self.permission_denied_error(
                component_id,
                "network inbound",
                &format!("port {port}"),
                &format!("No matching port in {port_count} approved ports"),
            ))
        }
    }

    /// Check if component can access storage namespace.
    ///
    /// # Errors
    ///
    /// Returns `CapabilityDenied` if component lacks permission.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_wasm::core::permission_checker::PermissionChecker;
    /// use airssys_wasm::core::permission::PermissionManifest;
    /// use airssys_wasm::core::component::ComponentId;
    ///
    /// let mut checker = PermissionChecker::new();
    /// let id = ComponentId::new("test");
    ///
    /// let mut perms = PermissionManifest::new();
    /// perms.storage.namespaces.push("myapp:cache".to_string());
    /// checker.load_permissions(id.clone(), &perms).unwrap();
    ///
    /// assert!(checker.can_access_storage(&id, "myapp:cache").is_ok());
    /// assert!(checker.can_access_storage(&id, "other:cache").is_err());
    /// ```
    pub fn can_access_storage(
        &self,
        component_id: &ComponentId,
        namespace: &str,
    ) -> WasmResult<()> {
        let perms = self.permissions.get(component_id).ok_or_else(|| {
            WasmError::component_not_found(format!(
                "Component '{component_id:?}' not found in permission checker"
            ))
        })?;

        if perms.storage_namespaces.iter().any(|ns| ns == namespace) {
            Ok(())
        } else {
            let namespace_count = perms.storage_namespaces.len();
            Err(self.permission_denied_error(
                component_id,
                "storage access",
                namespace,
                &format!("No matching namespace in {namespace_count} approved namespaces"),
            ))
        }
    }

    /// Get storage quota for component in bytes.
    ///
    /// Returns 0 if component has no storage permissions.
    pub fn storage_quota(&self, component_id: &ComponentId) -> u64 {
        self.permissions
            .get(component_id)
            .map(|p| p.storage_quota)
            .unwrap_or(0)
    }

    /// Clear cached permissions for a component.
    ///
    /// Useful when component is unloaded or permissions are updated.
    pub fn clear_cache(&self, _component_id: &ComponentId) {
        // Ignore poisoned mutex - cache is just a performance optimization
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }

    /// Helper to create permission denied error.
    fn permission_denied_error(
        &self,
        component_id: &ComponentId,
        operation: &str,
        resource: &str,
        reason: &str,
    ) -> WasmError {
        WasmError::capability_denied(
            crate::core::capability::Capability::FileRead(
                crate::core::capability::PathPattern::new(resource),
            ),
            format!(
                "Component '{component_id:?}' denied {operation} access to '{resource}': {reason}"
            ),
        )
    }
}

impl Default for PermissionChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Test code is allowed to use unwrap for clarity
mod tests {
    use super::*;
    use crate::core::permission::PermissionManifest;

    fn create_test_component_id() -> ComponentId {
        ComponentId::new("test-component")
    }

    #[test]
    fn test_filesystem_read_permission() {
        let mut checker = PermissionChecker::new();
        let id = create_test_component_id();

        let mut perms = PermissionManifest::new();
        perms.filesystem.read.push("/data/**".to_string());
        perms.filesystem.read.push("/config/*.json".to_string());

        checker.load_permissions(id.clone(), &perms).unwrap();

        // Should allow
        assert!(checker.can_read_file(&id, "/data/input.txt").is_ok());
        assert!(checker.can_read_file(&id, "/data/subdir/file.json").is_ok());
        assert!(checker.can_read_file(&id, "/config/app.json").is_ok());

        // Should deny
        assert!(checker.can_read_file(&id, "/etc/passwd").is_err());
        assert!(checker.can_read_file(&id, "/config/app.toml").is_err());
    }

    #[test]
    fn test_filesystem_write_permission() {
        let mut checker = PermissionChecker::new();
        let id = create_test_component_id();

        let mut perms = PermissionManifest::new();
        perms.filesystem.write.push("/output/**".to_string());

        checker.load_permissions(id.clone(), &perms).unwrap();

        assert!(checker.can_write_file(&id, "/output/result.txt").is_ok());
        assert!(checker
            .can_write_file(&id, "/output/subdir/data.json")
            .is_ok());
        assert!(checker.can_write_file(&id, "/etc/passwd").is_err());
    }

    #[test]
    fn test_network_outbound_permission() {
        let mut checker = PermissionChecker::new();
        let id = create_test_component_id();

        let mut perms = PermissionManifest::new();
        perms.network.outbound.push(NetworkEndpoint {
            host: "api.example.com".to_string(),
            port: 443,
        });
        perms.network.outbound.push(NetworkEndpoint {
            host: "*.cdn.example.com".to_string(),
            port: 443,
        });

        checker.load_permissions(id.clone(), &perms).unwrap();

        // Exact match
        assert!(checker
            .can_connect_outbound(&id, "api.example.com", 443)
            .is_ok());

        // Wildcard match
        assert!(checker
            .can_connect_outbound(&id, "a.cdn.example.com", 443)
            .is_ok());
        assert!(checker
            .can_connect_outbound(&id, "b.c.cdn.example.com", 443)
            .is_ok());

        // Should deny
        assert!(checker.can_connect_outbound(&id, "evil.com", 80).is_err());
        assert!(checker
            .can_connect_outbound(&id, "api.example.com", 80)
            .is_err());
    }

    #[test]
    fn test_network_inbound_permission() {
        let mut checker = PermissionChecker::new();
        let id = create_test_component_id();

        let mut perms = PermissionManifest::new();
        perms.network.inbound.push(8080);
        perms.network.inbound.push(9000);

        checker.load_permissions(id.clone(), &perms).unwrap();

        // Should allow
        assert!(checker.can_listen_inbound(&id, 8080).is_ok());
        assert!(checker.can_listen_inbound(&id, 9000).is_ok());

        // Should deny
        assert!(checker.can_listen_inbound(&id, 80).is_err());
        assert!(checker.can_listen_inbound(&id, 443).is_err());
    }

    #[test]
    fn test_storage_permission() {
        let mut checker = PermissionChecker::new();
        let id = create_test_component_id();

        let mut perms = PermissionManifest::new();
        perms.storage.namespaces.push("myapp:cache".to_string());
        perms.storage.namespaces.push("myapp:config".to_string());
        perms.storage.max_size_mb = 100;

        checker.load_permissions(id.clone(), &perms).unwrap();

        assert!(checker.can_access_storage(&id, "myapp:cache").is_ok());
        assert!(checker.can_access_storage(&id, "myapp:config").is_ok());
        assert!(checker.can_access_storage(&id, "other:cache").is_err());

        assert_eq!(checker.storage_quota(&id), 100 * 1024 * 1024);
    }

    #[test]
    fn test_permission_caching() {
        let mut checker = PermissionChecker::new();
        let id = create_test_component_id();

        let mut perms = PermissionManifest::new();
        perms.filesystem.read.push("/data/**".to_string());

        checker.load_permissions(id.clone(), &perms).unwrap();

        // First check (uncached)
        assert!(checker.can_read_file(&id, "/data/input.txt").is_ok());

        // Second check (cached) - should still work
        assert!(checker.can_read_file(&id, "/data/input.txt").is_ok());

        // Denial should also be cached
        assert!(checker.can_read_file(&id, "/etc/passwd").is_err());
        assert!(checker.can_read_file(&id, "/etc/passwd").is_err());
    }

    #[test]
    fn test_invalid_glob_pattern() {
        let mut checker = PermissionChecker::new();
        let id = create_test_component_id();

        let mut perms = PermissionManifest::new();
        perms.filesystem.read.push("[invalid".to_string()); // Invalid glob

        assert!(checker.load_permissions(id, &perms).is_err());
    }

    #[test]
    fn test_cache_size_configuration() {
        let checker_default = PermissionChecker::new();
        assert_eq!(checker_default.cache_size(), 1000);

        let checker_custom = PermissionChecker::with_cache_size(5000);
        assert_eq!(checker_custom.cache_size(), 5000);
    }
}
