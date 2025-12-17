//! Basic Trust Level System Example
//!
//! This example demonstrates the core trust-level classification system for WASM components.
//!
//! # What this demonstrates:
//! - Creating a trust registry with configuration
//! - Determining trust levels for different component sources
//! - Trusted vs Unknown classification
//!
//! # Run with:
//! ```bash
//! cargo run --example security_trust_basic
//! ```

// Layer 1: Standard library imports
use std::io::Write;

// Layer 2: Third-party crate imports
use tempfile::NamedTempFile;

// Layer 3: Internal module imports
use airssys_wasm::security::trust::{ComponentSource, TrustLevel, TrustRegistry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== WASM Trust Level System - Basic Example ===\n");

    // Create sample trust configuration
    let config_content = r#"
[trust]
dev_mode = false

[[trust.sources]]
type = "git"
url_pattern = "https://github.com/mycompany/*"
branch = "main"
description = "Internal company repositories"

[[trust.sources]]
type = "signing_key"
public_key = "ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIJbpYR+abc123"
signer = "security-team@mycompany.com"
description = "Security team signing key"

[[trust.sources]]
type = "local"
path_pattern = "/opt/verified-components/*"
description = "Pre-verified local components"
    "#;

    // Write to temporary file
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(config_content.as_bytes())?;
    temp_file.flush()?;

    println!("1️⃣  Loading trust configuration...");
    let registry = TrustRegistry::from_config(temp_file.path()).await?;
    println!("   ✅ Loaded {} trusted sources\n", registry.list_trusted_sources().len());

    // Test Case 1: Trusted Git repository (matches pattern)
    println!("2️⃣  Testing trusted Git repository:");
    let trusted_git = ComponentSource::Git {
        url: "https://github.com/mycompany/my-component".to_string(),
        branch: "main".to_string(),
        commit: "abc123".to_string(),
    };
    let level = registry.determine_trust_level("my-component", &trusted_git);
    println!("   Source: {}", trusted_git.identifier());
    println!("   Trust Level: {:?}", level);
    println!("   Requires Approval: {}", level.requires_approval());
    println!("   ✅ Result: Instant install (no review needed)\n");
    assert_eq!(level, TrustLevel::Trusted);

    // Test Case 2: Unknown Git repository (doesn't match pattern)
    println!("3️⃣  Testing unknown Git repository:");
    let unknown_git = ComponentSource::Git {
        url: "https://github.com/external-org/suspicious-component".to_string(),
        branch: "main".to_string(),
        commit: "xyz789".to_string(),
    };
    let level = registry.determine_trust_level("suspicious-component", &unknown_git);
    println!("   Source: {}", unknown_git.identifier());
    println!("   Trust Level: {:?}", level);
    println!("   Requires Approval: {}", level.requires_approval());
    println!("   ⏳ Result: Manual review required\n");
    assert_eq!(level, TrustLevel::Unknown);

    // Test Case 3: Branch mismatch (trusted repo but wrong branch)
    println!("4️⃣  Testing branch restriction:");
    let wrong_branch = ComponentSource::Git {
        url: "https://github.com/mycompany/my-component".to_string(),
        branch: "dev".to_string(), // Not 'main'
        commit: "def456".to_string(),
    };
    let level = registry.determine_trust_level("my-component-dev", &wrong_branch);
    println!("   Source: {}", wrong_branch.identifier());
    println!("   Trust Level: {:?}", level);
    println!("   ⚠️  Result: Branch 'dev' not in trusted list (only 'main' allowed)\n");
    assert_eq!(level, TrustLevel::Unknown);

    // Test Case 4: Trusted signed component
    println!("5️⃣  Testing signed component:");
    let signed_component = ComponentSource::Signed {
        signature: "ed25519:sig123...".to_string(),
        public_key: "ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIJbpYR+abc123".to_string(),
    };
    let level = registry.determine_trust_level("signed-component", &signed_component);
    println!("   Source: {}", signed_component.identifier());
    println!("   Trust Level: {:?}", level);
    println!("   ✅ Result: Signature verified - instant install\n");
    assert_eq!(level, TrustLevel::Trusted);

    // Test Case 5: Local verified component
    println!("6️⃣  Testing local verified component:");
    let local_component = ComponentSource::Local {
        path: std::path::PathBuf::from("/opt/verified-components/my-component.wasm"),
    };
    let level = registry.determine_trust_level("local-component", &local_component);
    println!("   Source: {}", local_component.identifier());
    println!("   Trust Level: {:?}", level);
    println!("   ✅ Result: Pre-verified path - instant install\n");
    assert_eq!(level, TrustLevel::Trusted);

    println!("=== Summary ===");
    println!("✅ Trusted sources: Instant install (no approval delay)");
    println!("⏳ Unknown sources: Manual review required");
    println!("🔒 Security posture: Deny-by-default");

    Ok(())
}
