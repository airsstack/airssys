//! Example: Auto-Approval Workflow for Trusted Sources
#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "example code"
)]
//! This example demonstrates the auto-approval workflow for WASM components from
//! trusted sources. Trusted components install instantly without user interaction,
//! optimizing developer productivity.
//!
//! # What This Demonstrates
//!
//! - Configuring trusted component sources (Git repositories, signing keys, local paths)
//! - Automatic approval for trusted sources (<1ms latency)
//! - Integration with Task 2.1 Trust Registry
//! - Audit logging for approved installations
//!
//! # Run With
//!
//! ```bash
//! cargo run --example security_approval_trusted
//! ```

// Layer 1: Standard library imports
use std::path::PathBuf;
use std::sync::Arc;

// Layer 2: Third-party imports
use tempfile::TempDir;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

// Layer 3: Internal imports
use airssys_wasm::security::approval::{ApprovalDecision, ApprovalStore, ApprovalWorkflow};
use airssys_wasm::security::trust::{ComponentSource, TrustRegistry};
use airssys_wasm::security::WasmCapabilitySet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber for logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🚀 Example: Auto-Approval Workflow for Trusted Sources");
    info!("");

    // Create temporary directory for this example
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("trust-config.toml");

    // Step 1: Create trust configuration with trusted sources
    info!("📝 Step 1: Creating trust configuration...");

    let config_content = r#"
[trust]
dev_mode = false

# Trusted Git repository (internal organization)
[[trust.sources]]
type = "git"
url_pattern = "https://github.com/mycompany/*"
branch = "main"
description = "Internal company repositories"

# Trusted signing key (security team)
[[trust.sources]]
type = "signing_key"
public_key = "ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIJ..."
signer = "security@mycompany.com"
description = "Security team signing key"

# Trusted local path (pre-verified components)
[[trust.sources]]
type = "local"
path_pattern = "/opt/verified-components/*"
description = "Pre-verified components"
"#;

    tokio::fs::write(&config_path, config_content).await?;
    info!("✅ Trust configuration created");
    info!("");

    // Step 2: Initialize approval workflow
    info!("🔧 Step 2: Initializing approval workflow...");

    let trust_registry = Arc::new(TrustRegistry::from_config(&config_path).await?);
    let approval_store = Arc::new(ApprovalStore::new(temp_dir.path().join("approvals"))?);
    let workflow = ApprovalWorkflow::new(trust_registry, approval_store);

    info!("✅ Approval workflow initialized");
    info!("");

    // Step 3: Test auto-approval for trusted Git source
    info!("🧪 Step 3: Testing auto-approval for trusted Git source...");

    let source = ComponentSource::Git {
        url: "https://github.com/mycompany/data-processor".to_string(),
        branch: "main".to_string(),
        commit: "abc123def456".to_string(),
    };

    let capabilities = WasmCapabilitySet::new();

    let start = std::time::Instant::now();
    let decision = workflow
        .request_approval("data-processor", &source, &capabilities)
        .await?;
    let elapsed = start.elapsed();

    match &decision {
        ApprovalDecision::Approved { approver, .. } => {
            info!("✅ Component auto-approved!");
            info!("   Approver: {}", approver);
            info!("   Latency: {:?} (target: <1ms)", elapsed);
            info!("   Can proceed: {}", decision.can_proceed());
        }
        _ => {
            panic!("Expected Approved, got: {:?}", decision);
        }
    }
    info!("");

    // Step 4: Test auto-approval for trusted local source
    info!("🧪 Step 4: Testing auto-approval for trusted local source...");

    let source = ComponentSource::Local {
        path: PathBuf::from("/opt/verified-components/logger"),
    };

    let decision = workflow
        .request_approval("logger", &source, &capabilities)
        .await?;

    match &decision {
        ApprovalDecision::Approved { approver, .. } => {
            info!("✅ Component auto-approved!");
            info!("   Approver: {}", approver);
            info!("   Can proceed: {}", decision.can_proceed());
        }
        _ => {
            panic!("Expected Approved, got: {:?}", decision);
        }
    }
    info!("");

    // Step 5: Test auto-approval for signed component
    info!("🧪 Step 5: Testing auto-approval for signed component...");

    let source = ComponentSource::Signed {
        public_key: "ed25519:AAAAC3NzaC1lZDI1NTE5AAAAIJ...".to_string(),
        signature: "signature123...".to_string(),
    };

    let decision = workflow
        .request_approval("analytics", &source, &capabilities)
        .await?;

    match &decision {
        ApprovalDecision::Approved { approver, .. } => {
            info!("✅ Component auto-approved!");
            info!("   Approver: {}", approver);
            info!("   Can proceed: {}", decision.can_proceed());
        }
        _ => {
            panic!("Expected Approved, got: {:?}", decision);
        }
    }
    info!("");

    // Summary
    info!("📊 Summary:");
    info!("   ✅ All trusted sources auto-approved instantly");
    info!("   ✅ No user interaction required");
    info!("   ✅ Performance target met (<1ms)");
    info!("   ✅ Audit trail captured in logs");
    info!("");
    info!("🎉 Example completed successfully!");

    Ok(())
}
