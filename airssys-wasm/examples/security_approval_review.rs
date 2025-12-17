//! Example: Manual Review Workflow for Unknown Sources
//!
//! This example demonstrates the manual review workflow for WASM components from
//! unknown/untrusted sources. Unknown components enter a review queue and await
//! administrator approval before installation.
//!
//! # What This Demonstrates
//!
//! - Review queue for unknown components
//! - Prior approval caching (avoids re-prompting)
//! - Manual approval/denial operations
//! - Administrator review workflow
//! - Persistent decision storage
//!
//! # Run With
//!
//! ```bash
//! cargo run --example security_approval_review
//! ```

// Layer 1: Standard library imports
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
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🚀 Example: Manual Review Workflow for Unknown Sources");
    info!("");

    // Create temporary directory for this example
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("trust-config.toml");

    // Step 1: Create trust configuration with NO trusted sources
    info!("📝 Step 1: Creating trust configuration (no trusted sources)...");
    
    let config_content = r#"
[trust]
dev_mode = false

# No trusted sources configured
# All components will enter review queue
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

    // Step 3: Request approval for unknown component (enters queue)
    info!("🧪 Step 3: Requesting approval for unknown component...");
    
    let source = ComponentSource::Git {
        url: "https://github.com/external/untrusted-tool".to_string(),
        branch: "main".to_string(),
        commit: "xyz789abc123".to_string(),
    };
    
    let capabilities = WasmCapabilitySet::new();
    
    let decision = workflow.request_approval(
        "untrusted-tool",
        &source,
        &capabilities,
    ).await?;
    
    match &decision {
        ApprovalDecision::PendingReview { request_id, queue_position } => {
            info!("⏳ Component pending review!");
            info!("   Request ID: {}", request_id);
            info!("   Queue position: {}", queue_position);
            info!("   Status: Waiting for administrator approval");
        }
        _ => {
            panic!("Expected PendingReview, got: {:?}", decision);
        }
    }
    info!("");

    // Step 4: Administrator lists pending requests
    info!("👤 Step 4: Administrator reviews pending requests...");
    
    let review_queue = workflow.review_queue();
    let pending = review_queue.list_pending()?;
    
    info!("📋 Pending requests: {}", pending.len());
    for (i, request) in pending.iter().enumerate() {
        info!("   {}. Component: {}", i + 1, request.component_id);
        info!("      Source: {:?}", request.source);
        info!("      Trust level: {:?}", request.trust_level);
        info!("      State: {}", request.state);
    }
    info!("");

    // Step 5: Administrator starts review
    info!("🔍 Step 5: Administrator starts reviewing component...");
    
    let reviewed = review_queue.start_review("untrusted-tool", "admin@example.com")?;
    info!("✅ Review started!");
    info!("   Reviewer: admin@example.com");
    info!("   State: {}", reviewed.state);
    info!("");

    // Step 6: Administrator approves component
    info!("✅ Step 6: Administrator approves component...");
    
    review_queue.approve(
        "untrusted-tool",
        "admin@example.com",
        None, // Use original capabilities
        Some("Code review complete. Looks safe.".to_string()),
    ).await?;
    
    info!("✅ Component approved!");
    info!("   Approver: admin@example.com");
    info!("   Reason: Code review complete. Looks safe.");
    info!("   Decision persisted to disk");
    info!("");

    // Step 7: Request same component again (should use cached approval)
    info!("🔄 Step 7: Requesting same component again (cached approval)...");
    
    let decision = workflow.request_approval(
        "untrusted-tool",
        &source,
        &capabilities,
    ).await?;
    
    match &decision {
        ApprovalDecision::Approved { approver, .. } => {
            info!("✅ Component auto-approved from cache!");
            info!("   Approver: {}", approver);
            info!("   No re-prompting required");
        }
        _ => {
            panic!("Expected Approved (cached), got: {:?}", decision);
        }
    }
    info!("");

    // Step 8: Test denial workflow
    info!("❌ Step 8: Testing denial workflow for malicious component...");
    
    let malicious_source = ComponentSource::Git {
        url: "https://github.com/malicious/ransomware".to_string(),
        branch: "main".to_string(),
        commit: "bad123".to_string(),
    };
    
    let decision = workflow.request_approval(
        "ransomware",
        &malicious_source,
        &capabilities,
    ).await?;
    
    // Should enter queue
    if let ApprovalDecision::PendingReview { .. } = decision {
        info!("⏳ Malicious component entered queue");
        
        // Administrator reviews and denies
        review_queue.start_review("ransomware", "admin@example.com")?;
        review_queue.deny(
            "ransomware",
            "admin@example.com",
            "Security risk detected: Suspicious file operations",
        ).await?;
        
        warn!("❌ Component denied!");
        warn!("   Denier: admin@example.com");
        warn!("   Reason: Security risk detected: Suspicious file operations");
    }
    info!("");

    // Step 9: Request denied component again (should use cached denial)
    info!("🔄 Step 9: Requesting denied component again (cached denial)...");
    
    let decision = workflow.request_approval(
        "ransomware",
        &malicious_source,
        &capabilities,
    ).await?;
    
    match &decision {
        ApprovalDecision::Denied { reason, .. } => {
            warn!("❌ Component denied from cache!");
            warn!("   Reason: {}", reason);
            warn!("   Installation blocked");
        }
        _ => {
            panic!("Expected Denied (cached), got: {:?}", decision);
        }
    }
    info!("");

    // Summary
    info!("📊 Summary:");
    info!("   ✅ Unknown components enter review queue");
    info!("   ✅ Administrator approval workflow functional");
    info!("   ✅ Denial workflow functional");
    info!("   ✅ Prior approvals cached (no re-prompting)");
    info!("   ✅ Prior denials cached (persistent blocking)");
    info!("   ✅ All decisions persisted to disk");
    info!("");
    info!("🎉 Example completed successfully!");

    Ok(())
}
