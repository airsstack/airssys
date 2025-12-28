//! Development Mode Trust Example
//!
//! This example demonstrates DevMode, which bypasses all security checks for rapid local development.
//!
//! # What this demonstrates:
//! - Enabling/disabling development mode
//! - Security bypass behavior
//! - Warning messages for audit trail
//!
//! # Security Warning:
//! **NEVER use DevMode in production!**
//!
//! # Run with:
//! ```bash
//! RUST_LOG=warn cargo run --example security_trust_devmode
//! ```

// Layer 1: Standard library imports
use std::io::Write;

// Layer 2: Third-party crate imports
use tempfile::NamedTempFile;

// Layer 3: Internal module imports
use airssys_wasm::security::trust::{ComponentSource, TrustLevel, TrustRegistry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing to see warnings
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .init();

    println!("=== WASM Trust Level System - DevMode Example ===\n");
    println!("⚠️  WARNING: This example demonstrates DEVELOPMENT MODE");
    println!("⚠️  Development mode BYPASSES ALL SECURITY CHECKS");
    println!("⚠️  NEVER use in production!\n");

    // Create minimal trust configuration
    let config_content = r#"
[trust]
dev_mode = false

[[trust.sources]]
type = "git"
url_pattern = "https://github.com/mycompany/*"
description = "Internal repos"
    "#;

    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(config_content.as_bytes())?;
    temp_file.flush()?;

    let registry = TrustRegistry::from_config(temp_file.path()).await?;

    // Test Case 1: Normal mode - unknown source requires approval
    println!("1️⃣  Normal Mode (Security Enabled):");
    println!("   DevMode: {}", registry.is_dev_mode());

    let unknown_source = ComponentSource::Git {
        url: "https://github.com/external/suspicious-component".to_string(),
        branch: "main".to_string(),
        commit: "abc123".to_string(),
    };

    let level = registry.determine_trust_level("suspicious-component", &unknown_source);
    println!("   Source: {}", unknown_source.identifier());
    println!("   Trust Level: {:?}", level);
    println!("   Security Posture: {}", level.security_posture());
    println!("   ⏳ Result: Review required (secure)\n");
    assert_eq!(level, TrustLevel::Unknown);

    // Test Case 2: Enable DevMode
    println!("2️⃣  Enabling Development Mode:");
    println!("   (Watch for warning messages below)\n");
    registry.set_dev_mode(true);
    println!("   DevMode: {}", registry.is_dev_mode());
    println!("   ⚠️  Security checks disabled!\n");

    // Test Case 3: DevMode - same unknown source now bypasses security
    println!("3️⃣  DevMode Enabled - Unknown Source:");
    let level = registry.determine_trust_level("suspicious-component", &unknown_source);
    println!("   Source: {}", unknown_source.identifier());
    println!("   Trust Level: {:?}", level);
    println!("   Security Posture: {}", level.security_posture());
    println!("   Bypasses Security: {}", level.bypasses_security());
    println!("   🔧 Result: Unrestricted install (INSECURE!)\n");
    assert_eq!(level, TrustLevel::DevMode);
    assert!(level.bypasses_security());

    // Test Case 4: DevMode takes precedence over trusted sources
    println!("4️⃣  DevMode Enabled - Even Trusted Sources:");
    let trusted_source = ComponentSource::Git {
        url: "https://github.com/mycompany/my-component".to_string(),
        branch: "main".to_string(),
        commit: "def456".to_string(),
    };

    let level = registry.determine_trust_level("my-component", &trusted_source);
    println!("   Source: {}", trusted_source.identifier());
    println!("   Trust Level: {:?} (not Trusted!)", level);
    println!("   Note: DevMode overrides even trusted sources\n");
    assert_eq!(level, TrustLevel::DevMode);

    // Test Case 5: Disable DevMode
    println!("5️⃣  Disabling Development Mode:");
    registry.set_dev_mode(false);
    println!("   DevMode: {}", registry.is_dev_mode());
    println!("   🔒 Security checks re-enabled\n");

    // Test Case 6: Verify security restored
    println!("6️⃣  Normal Mode Restored:");
    let level = registry.determine_trust_level("suspicious-component", &unknown_source);
    println!("   Source: {}", unknown_source.identifier());
    println!("   Trust Level: {:?}", level);
    println!("   ✅ Result: Review required (secure again)\n");
    assert_eq!(level, TrustLevel::Unknown);

    println!("=== DevMode Use Cases ===");
    println!("✅ Valid: Local development and testing");
    println!("✅ Valid: Rapid iteration without approval friction");
    println!("✅ Valid: CI/CD test environments");
    println!("❌ NEVER: Production deployments");
    println!("❌ NEVER: Public-facing services");
    println!("\n=== Audit Trail ===");
    println!("All DevMode usage is logged at WARNING level");
    println!("Review logs regularly for unexpected DevMode activation");

    Ok(())
}
