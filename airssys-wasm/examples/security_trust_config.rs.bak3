//! Trust Configuration Management Example
//!
//! This example demonstrates dynamic trust source management and configuration patterns.
//!
//! # What this demonstrates:
//! - Loading trust configuration from TOML
//! - Adding/removing trusted sources dynamically
//! - Listing trusted sources
//! - Configuration validation
//!
//! # Run with:
//! ```bash
//! cargo run --example security_trust_config
//! ```

// Layer 1: Standard library imports
use std::io::Write;

// Layer 2: Third-party crate imports
use tempfile::NamedTempFile;

// Layer 3: Internal module imports
use airssys_wasm::security::trust::{ComponentSource, TrustLevel, TrustRegistry, TrustSource};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== WASM Trust Configuration Management Example ===\n");

    // Create initial trust configuration
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
    "#;

    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(config_content.as_bytes())?;
    temp_file.flush()?;

    // Load initial configuration
    println!("1️⃣  Loading initial configuration:");
    let registry = TrustRegistry::from_config(temp_file.path()).await?;
    println!(
        "   ✅ Loaded {} trusted sources\n",
        registry.list_trusted_sources().len()
    );

    // List initial trusted sources
    println!("2️⃣  Initial trusted sources:");
    for (i, source) in registry.list_trusted_sources().iter().enumerate() {
        println!("   {}. Type: {}", i + 1, source.source_type());
        match source {
            TrustSource::GitRepository {
                url_pattern,
                branch,
                description,
            } => {
                println!("      URL Pattern: {}", url_pattern);
                if let Some(branch) = branch {
                    println!("      Branch: {}", branch);
                }
                println!("      Description: {}", description);
            }
            TrustSource::SigningKey {
                public_key,
                signer,
                description,
            } => {
                println!("      Public Key: {}...", &public_key[..30]);
                println!("      Signer: {}", signer);
                println!("      Description: {}", description);
            }
            TrustSource::LocalPath {
                path_pattern,
                description,
            } => {
                println!("      Path Pattern: {}", path_pattern);
                println!("      Description: {}", description);
            }
        }
    }
    println!();

    // Test component before adding new source
    println!("3️⃣  Testing external organization (before trust):");
    let external_component = ComponentSource::Git {
        url: "https://github.com/trusted-external-org/component".to_string(),
        branch: "stable".to_string(),
        commit: "xyz789".to_string(),
    };
    let level = registry.determine_trust_level("external-component", &external_component);
    println!("   Source: {}", external_component.identifier());
    println!("   Trust Level: {:?}", level);
    println!("   ⏳ Result: Unknown - review required\n");
    assert_eq!(level, TrustLevel::Unknown);

    // Add new trusted source dynamically
    println!("4️⃣  Adding new trusted source (requires admin permission):");
    let new_source = TrustSource::GitRepository {
        url_pattern: "https://github.com/trusted-external-org/*".to_string(),
        branch: Some("stable".to_string()),
        description: "Verified external organization (added dynamically)".to_string(),
    };
    registry.add_trusted_source(new_source)?;
    println!("   ✅ Added: https://github.com/trusted-external-org/*");
    println!(
        "   Total trusted sources: {}\n",
        registry.list_trusted_sources().len()
    );

    // Test same component after adding source
    println!("5️⃣  Testing external organization (after trust):");
    let level = registry.determine_trust_level("external-component", &external_component);
    println!("   Source: {}", external_component.identifier());
    println!("   Trust Level: {:?}", level);
    println!("   ✅ Result: Trusted - instant install\n");
    assert_eq!(level, TrustLevel::Trusted);

    // Add local path source
    println!("6️⃣  Adding local path source:");
    let local_source = TrustSource::LocalPath {
        path_pattern: "/home/developer/workspace/components/*".to_string(),
        description: "Developer workspace components".to_string(),
    };
    registry.add_trusted_source(local_source)?;
    println!("   ✅ Added: /home/developer/workspace/components/*");
    println!(
        "   Total trusted sources: {}\n",
        registry.list_trusted_sources().len()
    );

    // List all sources after additions
    println!("7️⃣  All trusted sources after additions:");
    for (i, source) in registry.list_trusted_sources().iter().enumerate() {
        println!(
            "   {}. Type: {} - {}",
            i + 1,
            source.source_type(),
            match source {
                TrustSource::GitRepository { url_pattern, .. } => url_pattern.clone(),
                TrustSource::SigningKey { signer, .. } => format!("Signer: {}", signer),
                TrustSource::LocalPath { path_pattern, .. } => path_pattern.clone(),
            }
        );
    }
    println!();

    // Remove a trusted source
    println!("8️⃣  Removing trusted source (index 2):");
    let sources_before = registry.list_trusted_sources().len();
    registry.remove_trusted_source(2)?;
    let sources_after = registry.list_trusted_sources().len();
    println!("   ✅ Removed");
    println!("   Sources: {} → {}\n", sources_before, sources_after);

    // Verify component now unknown after removal
    println!("9️⃣  Testing external organization (after removal):");
    let level = registry.determine_trust_level("external-component", &external_component);
    println!("   Source: {}", external_component.identifier());
    println!("   Trust Level: {:?}", level);
    println!("   ⏳ Result: Unknown again (trust revoked)\n");
    assert_eq!(level, TrustLevel::Unknown);

    // Configuration validation example
    println!("🔟  Configuration validation:");
    println!("   ✅ URL patterns cannot be empty");
    println!("   ✅ Public keys must start with 'ed25519:'");
    println!("   ✅ Descriptions required for all sources");
    println!("   ✅ No duplicate sources allowed\n");

    println!("=== Trust Configuration Best Practices ===");
    println!("✅ Use wildcard patterns for organization-wide trust");
    println!("✅ Restrict branches for production deployments");
    println!("✅ Maintain audit trail of trust additions/removals");
    println!("✅ Regular review of trusted sources");
    println!("✅ Principle of least privilege (narrow patterns)");
    println!("❌ Avoid overly broad patterns like '**'");
    println!("❌ Don't trust entire Git hosting platforms");

    Ok(())
}
