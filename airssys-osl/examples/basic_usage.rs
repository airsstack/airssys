//! Basic usage example for airssys-osl
//!
//! This example demonstrates the basic usage patterns of the OSL framework.
//! Run with: cargo run --example basic_usage

use airssys_osl::core::{
    context::{ExecutionContext, SecurityContext},
    operation::OperationType,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AirsSys OSL Basic Usage Example ===\n");

    // Create a security context
    let security_context = SecurityContext::new("demo-user".to_string());
    println!(
        "Created security context for: {}",
        security_context.principal
    );

    // Create execution context with metadata
    let exec_context = ExecutionContext::new(security_context)
        .with_metadata("example".to_string(), "basic_usage".to_string())
        .with_metadata("version".to_string(), "1.0".to_string());

    println!("Execution context metadata:");
    if let Some(example) = exec_context.get_metadata("example") {
        println!("  - Example: {}", example);
    }
    if let Some(version) = exec_context.get_metadata("version") {
        println!("  - Version: {}", version);
    }

    // Demonstrate operation types
    println!("\nSupported operation types:");
    let operation_types = [
        OperationType::Filesystem,
        OperationType::Process,
        OperationType::Network,
        OperationType::Utility,
    ];

    for op_type in &operation_types {
        println!(
            "  - {}: privileged={}",
            op_type.as_str(),
            op_type.is_privileged()
        );
    }

    println!("\n=== Example completed successfully! ===");
    Ok(())
}
