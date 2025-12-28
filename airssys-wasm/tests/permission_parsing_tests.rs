#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

//! Permission parsing tests
//!
//! Tests for Component.toml permission parsing and validation.

use airssys_wasm::core::manifest::ComponentManifest;
use airssys_wasm::core::permission::{NetworkEndpoint, PermissionManifest};

#[test]
fn test_parse_empty_permissions() {
    let toml = r#"
[package]
name = "test-component"
version = "1.0.0"
"#;

    let manifest = ComponentManifest::from_toml_str(toml).unwrap();
    assert!(!manifest.permissions.has_any_permissions());
    assert_eq!(manifest.permissions.total_permission_count(), 0);
}

#[test]
fn test_parse_filesystem_read_permissions() {
    let toml = r#"
[package]
name = "test-component"
version = "1.0.0"

[permissions.filesystem]
read = ["/data/**", "/config/*.json", "/etc/myapp/app.toml"]
"#;

    let manifest = ComponentManifest::from_toml_str(toml).unwrap();
    assert_eq!(manifest.permissions.filesystem.read.len(), 3);
    assert!(manifest
        .permissions
        .filesystem
        .read
        .contains(&"/data/**".to_string()));
    assert!(manifest
        .permissions
        .filesystem
        .read
        .contains(&"/config/*.json".to_string()));
}

#[test]
fn test_parse_filesystem_write_permissions() {
    let toml = r#"
[package]
name = "test-component"
version = "1.0.0"

[permissions.filesystem]
write = ["/output/**", "/tmp/cache/*"]
"#;

    let manifest = ComponentManifest::from_toml_str(toml).unwrap();
    assert_eq!(manifest.permissions.filesystem.write.len(), 2);
}

#[test]
fn test_parse_filesystem_all_actions() {
    let toml = r#"
[package]
name = "test-component"
version = "1.0.0"

[permissions.filesystem]
read = ["/data/**"]
write = ["/output/**"]
delete = ["/tmp/cache/*"]
list = ["/data"]
"#;

    let manifest = ComponentManifest::from_toml_str(toml).unwrap();
    assert_eq!(manifest.permissions.filesystem.read.len(), 1);
    assert_eq!(manifest.permissions.filesystem.write.len(), 1);
    assert_eq!(manifest.permissions.filesystem.delete.len(), 1);
    assert_eq!(manifest.permissions.filesystem.list.len(), 1);
}

#[test]
fn test_parse_network_outbound_permissions() {
    let toml = r#"
[package]
name = "test-component"
version = "1.0.0"

[permissions.network]
outbound = [
    { host = "api.example.com", port = 443 },
    { host = "*.cdn.example.com", port = 443 },
    { host = "192.168.1.100", port = 8080 }
]
"#;

    let manifest = ComponentManifest::from_toml_str(toml).unwrap();
    assert_eq!(manifest.permissions.network.outbound.len(), 3);

    // Check exact match
    assert!(manifest
        .permissions
        .network
        .outbound
        .contains(&NetworkEndpoint {
            host: "api.example.com".to_string(),
            port: 443,
        }));

    // Check wildcard
    assert!(manifest
        .permissions
        .network
        .outbound
        .contains(&NetworkEndpoint {
            host: "*.cdn.example.com".to_string(),
            port: 443,
        }));

    // Check IP address
    assert!(manifest
        .permissions
        .network
        .outbound
        .contains(&NetworkEndpoint {
            host: "192.168.1.100".to_string(),
            port: 8080,
        }));
}

#[test]
fn test_parse_network_inbound_permissions() {
    let toml = r#"
[package]
name = "test-component"
version = "1.0.0"

[permissions.network]
inbound = [8080, 9000]
"#;

    let manifest = ComponentManifest::from_toml_str(toml).unwrap();
    assert_eq!(manifest.permissions.network.inbound.len(), 2);
    assert!(manifest.permissions.network.inbound.contains(&8080));
    assert!(manifest.permissions.network.inbound.contains(&9000));
}

#[test]
fn test_parse_storage_permissions() {
    let toml = r#"
[package]
name = "test-component"
version = "1.0.0"

[permissions.storage]
namespaces = ["myapp:cache", "myapp:config", "shared:public"]
max_size_mb = 100
"#;

    let manifest = ComponentManifest::from_toml_str(toml).unwrap();
    assert_eq!(manifest.permissions.storage.namespaces.len(), 3);
    assert_eq!(manifest.permissions.storage.max_size_mb, 100);
    assert!(manifest.permissions.storage.has_namespace("myapp:cache"));
    assert!(manifest.permissions.storage.has_namespace("shared:public"));
}

#[test]
fn test_parse_complete_permissions() {
    let toml = r#"
[package]
name = "data-processor"
version = "1.0.0"
description = "Comprehensive test component"

[permissions.filesystem]
read = ["/data/**", "/config/*.json"]
write = ["/output/**"]
delete = ["/tmp/cache/*"]
list = ["/data", "/config"]

[permissions.network]
outbound = [
    { host = "api.example.com", port = 443 },
    { host = "*.cdn.example.com", port = 443 }
]
inbound = [8080]

[permissions.storage]
namespaces = ["myapp:cache", "myapp:config"]
max_size_mb = 100
"#;

    let manifest = ComponentManifest::from_toml_str(toml).unwrap();

    // Verify filesystem
    assert_eq!(manifest.permissions.filesystem.read.len(), 2);
    assert_eq!(manifest.permissions.filesystem.write.len(), 1);
    assert_eq!(manifest.permissions.filesystem.delete.len(), 1);
    assert_eq!(manifest.permissions.filesystem.list.len(), 2);

    // Verify network
    assert_eq!(manifest.permissions.network.outbound.len(), 2);
    assert_eq!(manifest.permissions.network.inbound.len(), 1);

    // Verify storage
    assert_eq!(manifest.permissions.storage.namespaces.len(), 2);
    assert_eq!(manifest.permissions.storage.max_size_mb, 100);

    // Verify total count (2 read + 1 write + 1 delete + 2 list + 2 outbound + 1 inbound + 2 namespaces = 11)
    assert_eq!(manifest.permissions.total_permission_count(), 11);
}

#[test]
fn test_invalid_empty_filesystem_pattern() {
    let toml = r#"
[package]
name = "test-component"
version = "1.0.0"

[permissions.filesystem]
read = [""]
"#;

    let manifest = ComponentManifest::from_toml_str(toml).unwrap();
    assert!(manifest.validate().is_err());
}

#[test]
fn test_invalid_network_port_zero() {
    let toml = r#"
[package]
name = "test-component"
version = "1.0.0"

[permissions.network]
outbound = [
    { host = "api.example.com", port = 0 }
]
"#;

    let manifest = ComponentManifest::from_toml_str(toml).unwrap();
    assert!(manifest.validate().is_err());
}

#[test]
fn test_invalid_storage_namespace_format() {
    let toml = r#"
[package]
name = "test-component"
version = "1.0.0"

[permissions.storage]
namespaces = ["invalid-namespace"]
"#;

    let manifest = ComponentManifest::from_toml_str(toml).unwrap();
    assert!(manifest.validate().is_err());
}

#[test]
fn test_manifest_has_any_permissions() {
    let mut manifest = PermissionManifest::new();
    assert!(!manifest.has_any_permissions());

    manifest.filesystem.read.push("/data/**".to_string());
    assert!(manifest.has_any_permissions());
}

#[test]
fn test_filesystem_all_patterns_dedup() {
    let mut perms = PermissionManifest::new();
    perms.filesystem.read.push("/data/**".to_string());
    perms.filesystem.write.push("/data/**".to_string()); // Duplicate
    perms.filesystem.write.push("/output/**".to_string());

    let patterns = perms.filesystem.all_patterns();
    assert_eq!(patterns.len(), 2); // Deduplicated
    assert!(patterns.contains("/data/**"));
    assert!(patterns.contains("/output/**"));
}

#[test]
fn test_network_outbound_hosts() {
    let mut perms = PermissionManifest::new();
    perms.network.outbound.push(NetworkEndpoint {
        host: "api.example.com".to_string(),
        port: 443,
    });
    perms.network.outbound.push(NetworkEndpoint {
        host: "api.example.com".to_string(),
        port: 80, // Same host, different port
    });
    perms.network.outbound.push(NetworkEndpoint {
        host: "*.cdn.example.com".to_string(),
        port: 443,
    });

    let hosts = perms.network.outbound_hosts();
    assert_eq!(hosts.len(), 2); // Only unique hosts
    assert!(hosts.contains("api.example.com"));
    assert!(hosts.contains("*.cdn.example.com"));
}

#[test]
fn test_glob_pattern_syntax() {
    let toml = r#"
[package]
name = "test-component"
version = "1.0.0"

[permissions.filesystem]
read = [
    "/exact/path.txt",
    "/dir/*.json",
    "/recursive/**",
    "/single?char",
    "/range/[abc]*.txt"
]
"#;

    let manifest = ComponentManifest::from_toml_str(toml).unwrap();
    assert!(manifest.validate().is_ok());
    assert_eq!(manifest.permissions.filesystem.read.len(), 5);
}
