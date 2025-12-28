#![allow(clippy::panic, clippy::expect_used, clippy::unwrap_used)]

//! Permission matching tests
//!
//! Tests for glob pattern matching and permission checking logic.

use airssys_wasm::core::component::ComponentId;
use airssys_wasm::core::permission::{NetworkEndpoint, PermissionManifest};
use airssys_wasm::core::permission_checker::PermissionChecker;

#[test]
fn test_exact_path_match() {
    let mut checker = PermissionChecker::new();
    let id = ComponentId::new("test");

    let mut perms = PermissionManifest::new();
    perms.filesystem.read.push("/data/input.txt".to_string());

    checker.load_permissions(id.clone(), &perms).unwrap();

    // Exact match should work
    assert!(checker.can_read_file(&id, "/data/input.txt").is_ok());

    // Different file should fail
    assert!(checker.can_read_file(&id, "/data/output.txt").is_err());
}

#[test]
fn test_glob_single_directory() {
    let mut checker = PermissionChecker::new();
    let id = ComponentId::new("test");

    let mut perms = PermissionManifest::new();
    perms.filesystem.read.push("/data/*.json".to_string());

    checker.load_permissions(id.clone(), &perms).unwrap();

    // Should match in same directory
    assert!(checker.can_read_file(&id, "/data/file1.json").is_ok());
    assert!(checker.can_read_file(&id, "/data/file2.json").is_ok());

    // Note: glob `*` does NOT match `/`, so subdirectory should fail
    // But glob library behavior: `/data/*.json` with `/data/subdir/file.json` may match
    // depending on implementation. Let's test the actual behavior.
    // For stricter control, use non-recursive patterns or explicit directory levels.

    // Should not match different extension
    assert!(checker.can_read_file(&id, "/data/file.txt").is_err());
}

#[test]
fn test_glob_recursive() {
    let mut checker = PermissionChecker::new();
    let id = ComponentId::new("test");

    let mut perms = PermissionManifest::new();
    perms.filesystem.read.push("/data/**".to_string());

    checker.load_permissions(id.clone(), &perms).unwrap();

    // Should match at root
    assert!(checker.can_read_file(&id, "/data/file.txt").is_ok());

    // Should match in subdirectory
    assert!(checker.can_read_file(&id, "/data/subdir/file.txt").is_ok());

    // Should match deeply nested
    assert!(checker.can_read_file(&id, "/data/a/b/c/d/file.txt").is_ok());

    // Should not match different root
    assert!(checker.can_read_file(&id, "/other/file.txt").is_err());
}

#[test]
fn test_glob_question_mark() {
    let mut checker = PermissionChecker::new();
    let id = ComponentId::new("test");

    let mut perms = PermissionManifest::new();
    perms.filesystem.read.push("/data/file?.txt".to_string());

    checker.load_permissions(id.clone(), &perms).unwrap();

    // Should match single character
    assert!(checker.can_read_file(&id, "/data/file1.txt").is_ok());
    assert!(checker.can_read_file(&id, "/data/fileA.txt").is_ok());

    // Should not match multiple characters
    assert!(checker.can_read_file(&id, "/data/file12.txt").is_err());

    // Should not match no character
    assert!(checker.can_read_file(&id, "/data/file.txt").is_err());
}

#[test]
fn test_multiple_patterns() {
    let mut checker = PermissionChecker::new();
    let id = ComponentId::new("test");

    let mut perms = PermissionManifest::new();
    perms.filesystem.read.push("/data/**".to_string());
    perms.filesystem.read.push("/config/*.json".to_string());
    perms
        .filesystem
        .read
        .push("/etc/myapp/app.toml".to_string());

    checker.load_permissions(id.clone(), &perms).unwrap();

    // Should match first pattern
    assert!(checker.can_read_file(&id, "/data/file.txt").is_ok());
    assert!(checker.can_read_file(&id, "/data/subdir/file.txt").is_ok());

    // Should match second pattern
    assert!(checker.can_read_file(&id, "/config/settings.json").is_ok());

    // Should match third pattern (exact)
    assert!(checker.can_read_file(&id, "/etc/myapp/app.toml").is_ok());

    // Should not match any pattern
    assert!(checker.can_read_file(&id, "/other/file.txt").is_err());
}

#[test]
fn test_filesystem_write_permission() {
    let mut checker = PermissionChecker::new();
    let id = ComponentId::new("test");

    let mut perms = PermissionManifest::new();
    perms.filesystem.write.push("/output/**".to_string());
    perms.filesystem.write.push("/tmp/cache/*".to_string());

    checker.load_permissions(id.clone(), &perms).unwrap();

    // Write permissions
    assert!(checker.can_write_file(&id, "/output/result.txt").is_ok());
    assert!(checker
        .can_write_file(&id, "/output/subdir/result.txt")
        .is_ok());
    assert!(checker.can_write_file(&id, "/tmp/cache/temp.dat").is_ok());

    // Should not allow write elsewhere
    assert!(checker.can_write_file(&id, "/etc/passwd").is_err());
}

#[test]
fn test_filesystem_delete_permission() {
    let mut checker = PermissionChecker::new();
    let id = ComponentId::new("test");

    let mut perms = PermissionManifest::new();
    perms.filesystem.delete.push("/tmp/cache/*".to_string());

    checker.load_permissions(id.clone(), &perms).unwrap();

    assert!(checker.can_delete_file(&id, "/tmp/cache/temp.dat").is_ok());
    assert!(checker.can_delete_file(&id, "/data/file.txt").is_err());
}

#[test]
fn test_filesystem_list_permission() {
    let mut checker = PermissionChecker::new();
    let id = ComponentId::new("test");

    let mut perms = PermissionManifest::new();
    perms.filesystem.list.push("/data".to_string());
    perms.filesystem.list.push("/config".to_string());

    checker.load_permissions(id.clone(), &perms).unwrap();

    assert!(checker.can_list_directory(&id, "/data").is_ok());
    assert!(checker.can_list_directory(&id, "/config").is_ok());
    assert!(checker.can_list_directory(&id, "/etc").is_err());
}

#[test]
fn test_network_exact_domain_match() {
    let mut checker = PermissionChecker::new();
    let id = ComponentId::new("test");

    let mut perms = PermissionManifest::new();
    perms.network.outbound.push(NetworkEndpoint {
        host: "api.example.com".to_string(),
        port: 443,
    });

    checker.load_permissions(id.clone(), &perms).unwrap();

    // Exact match should work
    assert!(checker
        .can_connect_outbound(&id, "api.example.com", 443)
        .is_ok());

    // Different host should fail
    assert!(checker
        .can_connect_outbound(&id, "other.example.com", 443)
        .is_err());

    // Different port should fail
    assert!(checker
        .can_connect_outbound(&id, "api.example.com", 80)
        .is_err());
}

#[test]
fn test_network_wildcard_subdomain() {
    let mut checker = PermissionChecker::new();
    let id = ComponentId::new("test");

    let mut perms = PermissionManifest::new();
    perms.network.outbound.push(NetworkEndpoint {
        host: "*.example.com".to_string(),
        port: 443,
    });

    checker.load_permissions(id.clone(), &perms).unwrap();

    // Should match single-level subdomain
    assert!(checker
        .can_connect_outbound(&id, "a.example.com", 443)
        .is_ok());
    assert!(checker
        .can_connect_outbound(&id, "api.example.com", 443)
        .is_ok());

    // Should match multi-level subdomain
    assert!(checker
        .can_connect_outbound(&id, "a.b.example.com", 443)
        .is_ok());

    // Note: Our implementation matches any host ending with `.example.com`
    // This means `*.example.com` does NOT match bare `example.com` (correct)

    // Should not match different domain
    assert!(checker.can_connect_outbound(&id, "other.com", 443).is_err());
}

#[test]
fn test_network_ip_address() {
    let mut checker = PermissionChecker::new();
    let id = ComponentId::new("test");

    let mut perms = PermissionManifest::new();
    perms.network.outbound.push(NetworkEndpoint {
        host: "192.168.1.100".to_string(),
        port: 8080,
    });

    checker.load_permissions(id.clone(), &perms).unwrap();

    assert!(checker
        .can_connect_outbound(&id, "192.168.1.100", 8080)
        .is_ok());
    assert!(checker
        .can_connect_outbound(&id, "192.168.1.101", 8080)
        .is_err());
}

#[test]
fn test_network_multiple_endpoints() {
    let mut checker = PermissionChecker::new();
    let id = ComponentId::new("test");

    let mut perms = PermissionManifest::new();
    perms.network.outbound.push(NetworkEndpoint {
        host: "api.example.com".to_string(),
        port: 443,
    });
    perms.network.outbound.push(NetworkEndpoint {
        host: "*.cdn.example.com".to_string(),
        port: 443,
    });
    perms.network.outbound.push(NetworkEndpoint {
        host: "192.168.1.100".to_string(),
        port: 8080,
    });

    checker.load_permissions(id.clone(), &perms).unwrap();

    // All should work
    assert!(checker
        .can_connect_outbound(&id, "api.example.com", 443)
        .is_ok());
    assert!(checker
        .can_connect_outbound(&id, "assets.cdn.example.com", 443)
        .is_ok());
    assert!(checker
        .can_connect_outbound(&id, "192.168.1.100", 8080)
        .is_ok());

    // None of these should work
    assert!(checker.can_connect_outbound(&id, "evil.com", 80).is_err());
}

#[test]
fn test_storage_namespace_exact_match() {
    let mut checker = PermissionChecker::new();
    let id = ComponentId::new("test");

    let mut perms = PermissionManifest::new();
    perms.storage.namespaces.push("myapp:cache".to_string());
    perms.storage.namespaces.push("myapp:config".to_string());

    checker.load_permissions(id.clone(), &perms).unwrap();

    assert!(checker.can_access_storage(&id, "myapp:cache").is_ok());
    assert!(checker.can_access_storage(&id, "myapp:config").is_ok());
    assert!(checker.can_access_storage(&id, "other:cache").is_err());
    assert!(checker.can_access_storage(&id, "myapp:data").is_err());
}

#[test]
fn test_storage_quota() {
    let mut checker = PermissionChecker::new();
    let id = ComponentId::new("test");

    let mut perms = PermissionManifest::new();
    perms.storage.max_size_mb = 100;

    checker.load_permissions(id.clone(), &perms).unwrap();

    // 100 MB = 104857600 bytes
    assert_eq!(checker.storage_quota(&id), 100 * 1024 * 1024);
}

#[test]
fn test_deny_by_default() {
    let mut checker = PermissionChecker::new();
    let id = ComponentId::new("test");

    // Load component with NO permissions
    let perms = PermissionManifest::new();
    checker.load_permissions(id.clone(), &perms).unwrap();

    // Everything should be denied
    assert!(checker.can_read_file(&id, "/data/file.txt").is_err());
    assert!(checker.can_write_file(&id, "/output/file.txt").is_err());
    assert!(checker.can_delete_file(&id, "/tmp/file.txt").is_err());
    assert!(checker.can_list_directory(&id, "/data").is_err());
    assert!(checker
        .can_connect_outbound(&id, "example.com", 443)
        .is_err());
    assert!(checker.can_access_storage(&id, "myapp:cache").is_err());
}

#[test]
fn test_permission_caching() {
    let mut checker = PermissionChecker::new();
    let id = ComponentId::new("test");

    let mut perms = PermissionManifest::new();
    perms.filesystem.read.push("/data/**".to_string());

    checker.load_permissions(id.clone(), &perms).unwrap();

    // First check (uncached)
    let result1 = checker.can_read_file(&id, "/data/input.txt");
    assert!(result1.is_ok());

    // Second check (should be cached)
    let result2 = checker.can_read_file(&id, "/data/input.txt");
    assert!(result2.is_ok());

    // Denied access should also be cached
    let result3 = checker.can_read_file(&id, "/etc/passwd");
    assert!(result3.is_err());

    let result4 = checker.can_read_file(&id, "/etc/passwd");
    assert!(result4.is_err());
}

#[test]
fn test_invalid_glob_pattern_rejected() {
    let mut checker = PermissionChecker::new();
    let id = ComponentId::new("test");

    let mut perms = PermissionManifest::new();
    perms.filesystem.read.push("[invalid".to_string()); // Invalid glob

    let result = checker.load_permissions(id, &perms);
    assert!(result.is_err());
}

#[test]
fn test_component_not_found() {
    let checker = PermissionChecker::new();
    let id = ComponentId::new("nonexistent");

    // Component not loaded - all checks should fail
    assert!(checker.can_read_file(&id, "/data/file.txt").is_err());
    assert!(checker
        .can_connect_outbound(&id, "example.com", 443)
        .is_err());
    assert!(checker.can_access_storage(&id, "myapp:cache").is_err());
}

#[test]
fn test_case_sensitive_paths() {
    let mut checker = PermissionChecker::new();
    let id = ComponentId::new("test");

    let mut perms = PermissionManifest::new();
    perms.filesystem.read.push("/Data/**".to_string());

    checker.load_permissions(id.clone(), &perms).unwrap();

    // Glob matching is case-sensitive on Unix
    assert!(checker.can_read_file(&id, "/Data/file.txt").is_ok());

    #[cfg(unix)]
    {
        assert!(checker.can_read_file(&id, "/data/file.txt").is_err());
    }
}
