//! Example: DevMode Security Bypass Workflow
#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used, reason = "example code")]//!
//! This example demonstrates the DevMode security bypass workflow that allows
//! rapid local development without security friction. DevMode should **NEVER**
//! be enabled in production environments!
//!
//! # What This Demonstrates
//!
//! - DevMode configuration and activation
//! - Security bypass with prominent warnings
//! - Audit trail for DevMode usage
//! - Performance optimization for local development
//!
//! # ⚠️  WARNING ⚠️
//!
//! DevMode disables security checks and should **ONLY** be used in local
//! development environments. NEVER enable DevMode in production, staging,
//! or any shared environment!
//!
//! # Run With
//!
//! ```bash
//! cargo run --example security_approval_devmode
//! ```

// Layer 1: Standard library imports
use std::path::PathBuf;
use std::sync::Arc;

// Layer 2: Third-party imports
use tempfile::TempDir;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

// Layer 3: Internal imports
use airssys_wasm::security::approval::{ApprovalDecision, ApprovalStore, ApprovalWorkflow};
use airssys_wasm::security::trust::{ComponentSource, TrustRegistry};
use airssys_wasm::security::WasmCapabilitySet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber for logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::WARN) // Show warnings for DevMode
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    warn!("⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️");
    warn!("DEVELOPMENT MODE EXAMPLE");
    warn!("Security checks will be BYPASSED!");
    warn!("DO NOT use DevMode in production!");
    warn!("⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️");
    warn!("");

    // Create temporary directory for this example
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("trust-config.toml");

    // Step 1: Create trust configuration with DevMode ENABLED
    info!("📝 Step 1: Creating trust configuration with DevMode...");

    let config_content = r#"
[trust]
# ⚠️  WARNING: DevMode enabled! ⚠️
# Security checks are bypassed!
# ONLY for local development!
dev_mode = true
"#;

    tokio::fs::write(&config_path, config_content).await?;
    warn!("⚠️  DevMode configuration created");
    warn!("   Security checks: DISABLED");
    warn!("   Approval workflow: BYPASSED");
    warn!("");

    // Step 2: Initialize approval workflow
    info!("🔧 Step 2: Initializing approval workflow with DevMode...");

    let trust_registry = Arc::new(TrustRegistry::from_config(&config_path).await?);
    let approval_store = Arc::new(ApprovalStore::new(temp_dir.path().join("approvals"))?);
    let workflow = ApprovalWorkflow::new(trust_registry, approval_store);

    info!("✅ Approval workflow initialized (DevMode active)");
    info!("");

    // Step 3: Test DevMode bypass for local component
    info!("🧪 Step 3: Testing DevMode bypass for local component...");

    let source = ComponentSource::Local {
        path: PathBuf::from("/home/dev/my-local-component"),
    };

    let capabilities = WasmCapabilitySet::new();

    let start = std::time::Instant::now();
    let decision = workflow
        .request_approval("my-local-component", &source, &capabilities)
        .await?;
    let elapsed = start.elapsed();

    match &decision {
        ApprovalDecision::Bypassed { devmode } => {
            warn!("⚠️  Security bypassed (DevMode)!");
            info!("   Component: my-local-component");
            info!("   DevMode: {}", devmode);
            info!("   Latency: {:?}", elapsed);
            info!("   Can proceed: {}", decision.can_proceed());
        }
        _ => {
            panic!("Expected Bypassed, got: {:?}", decision);
        }
    }
    info!("");

    // Step 4: Test DevMode bypass for unknown remote component
    info!("🧪 Step 4: Testing DevMode bypass for unknown remote component...");

    let source = ComponentSource::Git {
        url: "https://github.com/untrusted/unknown-tool".to_string(),
        branch: "main".to_string(),
        commit: "abc123".to_string(),
    };

    let decision = workflow
        .request_approval("unknown-tool", &source, &capabilities)
        .await?;

    match &decision {
        ApprovalDecision::Bypassed { devmode } => {
            warn!("⚠️  Security bypassed for unknown component!");
            warn!("   Component: unknown-tool");
            warn!("   Source: Untrusted Git repository");
            warn!("   DevMode: {}", devmode);
            warn!("   Risk: Malicious code could execute without review!");
        }
        _ => {
            panic!("Expected Bypassed, got: {:?}", decision);
        }
    }
    info!("");

    // Step 5: Test DevMode bypass for potentially malicious component
    info!("🧪 Step 5: Testing DevMode bypass for potentially malicious component...");

    let source = ComponentSource::Git {
        url: "https://github.com/sketchy/ransomware".to_string(),
        branch: "main".to_string(),
        commit: "bad123".to_string(),
    };

    let decision = workflow
        .request_approval("ransomware", &source, &capabilities)
        .await?;

    match &decision {
        ApprovalDecision::Bypassed { .. } => {
            warn!("⚠️  ⚠️  ⚠️  CRITICAL SECURITY RISK ⚠️  ⚠️  ⚠️");
            warn!("   Malicious component allowed to execute!");
            warn!("   Component: ransomware");
            warn!("   Source: Untrusted repository");
            warn!("   DevMode bypassed security checks!");
            warn!("   This would be BLOCKED in production!");
        }
        _ => {
            panic!("Expected Bypassed, got: {:?}", decision);
        }
    }
    info!("");

    // Step 6: Performance comparison
    info!("📊 Step 6: Performance comparison (DevMode vs Production)...");

    let iterations = 100;
    let mut total_elapsed = std::time::Duration::ZERO;

    for i in 0..iterations {
        let source = ComponentSource::Local {
            path: PathBuf::from(format!("/dev/component-{}", i)),
        };

        let start = std::time::Instant::now();
        workflow
            .request_approval(&format!("component-{}", i), &source, &capabilities)
            .await?;
        total_elapsed += start.elapsed();
    }

    let avg_latency = total_elapsed / iterations;
    info!("✅ Performance test complete:");
    info!("   Iterations: {}", iterations);
    info!("   Total time: {:?}", total_elapsed);
    info!("   Average latency: {:?}", avg_latency);
    info!("   Expected: <1ms (no approval queue overhead)");
    info!("");

    // Summary
    warn!("📊 Summary:");
    warn!("   ⚠️  All security checks BYPASSED");
    warn!("   ⚠️  Unknown components allowed without review");
    warn!("   ⚠️  Malicious components could execute");
    warn!("   ✅ Fast iteration for local development");
    warn!("   ✅ All bypasses logged to audit trail");
    warn!("");
    warn!("🚨 Security Implications:");
    warn!("   - DevMode disables the approval workflow");
    warn!("   - No administrator review required");
    warn!("   - No capability enforcement (future feature)");
    warn!("   - No sandboxing verification");
    warn!("   - Malicious code could execute freely");
    warn!("");
    warn!("✅ Appropriate Use Cases:");
    warn!("   - Local development on trusted machine");
    warn!("   - Rapid component iteration");
    warn!("   - Testing and debugging");
    warn!("   - Developer productivity");
    warn!("");
    warn!("❌ NEVER Use DevMode For:");
    warn!("   - Production environments");
    warn!("   - Staging environments");
    warn!("   - CI/CD pipelines");
    warn!("   - Shared development servers");
    warn!("   - Any environment with sensitive data");
    warn!("");
    warn!("⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️");
    warn!("Remember: With great power comes great responsibility!");
    warn!("DevMode is a development tool. Use it wisely.");
    warn!("⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️  ⚠️");

    Ok(())
}
