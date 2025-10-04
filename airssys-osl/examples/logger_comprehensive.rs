//! Comprehensive logger usage examples.
//!
//! This example demonstrates all logger functionality including different
//! formats, configurations, and usage patterns.

#![allow(clippy::unwrap_used, clippy::expect_used)] // Allow in examples for clarity

use airssys_osl::middleware::logger::{
    ActivityLog, ActivityLogger, ConsoleActivityLogger, FileActivityLogger, LogFormat,
    LoggerConfig, TracingActivityLogger,
};
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 AirssysOSL Comprehensive Logger Examples\n");

    // Initialize tracing for examples
    tracing_subscriber::fmt::init();

    // 1. Console Logger Examples
    console_logger_examples().await?;

    // 2. File Logger Examples
    file_logger_examples().await?;

    // 3. Tracing Logger Examples
    tracing_logger_examples().await?;

    // 4. Configuration Examples
    configuration_examples().await?;

    println!("✅ All comprehensive logger examples completed successfully!");
    Ok(())
}

/// Demonstrates console logger with all formats
async fn console_logger_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("🖥️ Console Logger Examples");
    println!("============================\n");

    // Test all log formats
    let formats = vec![
        (LogFormat::Json, "JSON"),
        (LogFormat::Pretty, "Pretty"),
        (LogFormat::Compact, "Compact"),
    ];

    for (format, name) in formats {
        println!("{}. {} Format:", name.chars().next().unwrap(), name);
        let logger = ConsoleActivityLogger::new().with_format(format);
        let activity = create_sample_activity(&format!("{name} Example"), "console_demo")?;
        logger.log_activity(activity).await?;
        println!();
    }

    println!("✅ Console logger examples completed\n");
    Ok(())
}

/// Demonstrates file logger functionality
async fn file_logger_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("📁 File Logger Examples");
    println!("=========================\n");

    let temp_dir = TempDir::new()?;

    // Test different file formats
    let formats = vec![
        (LogFormat::Json, "json"),
        (LogFormat::Pretty, "pretty"),
        (LogFormat::Compact, "compact"),
    ];

    for (format, name) in formats {
        let file_path = temp_dir.path().join(format!("{name}.log"));
        println!("1. File logging with {name} format:");

        let file_logger = FileActivityLogger::new(file_path.clone())
            .await?
            .with_format(format);

        let activity = create_sample_activity(&format!("File {name} Example"), "file_demo")?;
        file_logger.log_activity(activity).await?;
        file_logger.flush().await?;

        // Read and display content
        let content = tokio::fs::read_to_string(&file_path).await?;
        println!("   → Logged to: {file_path:?}");
        println!(
            "   → Content preview: {}",
            content.lines().next().unwrap_or("(empty)")
        );
        println!();
    }

    println!("✅ File logger examples completed\n");
    Ok(())
}

/// Demonstrates tracing logger functionality  
async fn tracing_logger_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Tracing Logger Examples");
    println!("============================\n");

    let tracing_logger = TracingActivityLogger::new();

    // Log different types of activities
    println!("1. Success activity:");
    let success_activity = create_sample_activity("Success Operation", "tracing_demo")?;
    tracing_logger.log_activity(success_activity).await?;

    println!("\n2. Complex activity with metadata:");
    let complex_activity = create_complex_activity("Complex Operation", "tracing_demo")?;
    tracing_logger.log_activity(complex_activity).await?;

    println!("✅ Tracing logger examples completed\n");
    Ok(())
}

/// Demonstrates configuration options
async fn configuration_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚙️ Configuration Examples");
    println!("===========================\n");

    // Default configuration
    println!("1. Default LoggerConfig:");
    let default_config = LoggerConfig::default();
    println!("   → Format: {:?}", default_config.format);
    println!("   → Level: {:?}", default_config.level);
    println!("   → Buffer size: {}", default_config.buffer_size);
    println!(
        "   → Flush interval: {}ms",
        default_config.flush_interval_ms
    );

    // Custom configuration
    println!("\n2. Custom LoggerConfig:");
    let custom_config = LoggerConfig {
        format: LogFormat::Pretty,
        buffer_size: 2000,
        flush_interval_ms: 2000,
        ..LoggerConfig::default()
    };
    println!("   → Format: {:?}", custom_config.format);
    println!("   → Buffer size: {}", custom_config.buffer_size);
    println!("   → Flush interval: {}ms", custom_config.flush_interval_ms);

    // Demonstrate logger with custom configuration
    println!("\n3. Logger with custom configuration:");
    let temp_dir = TempDir::new()?;
    let file_logger = FileActivityLogger::new(temp_dir.path().join("custom_config.log"))
        .await?
        .with_format(LogFormat::Pretty);

    let activity = create_sample_activity("Custom Config Demo", "config_test")?;
    file_logger.log_activity(activity).await?;
    file_logger.flush().await?;
    println!("   → Logged with custom configuration");

    println!("\n✅ Configuration examples completed\n");
    Ok(())
}

/// Helper function to create sample activity logs
fn create_sample_activity(
    operation_name: &str,
    operation_type: &str,
) -> Result<ActivityLog, Box<dyn std::error::Error>> {
    Ok(ActivityLog::new(
        uuid::Uuid::new_v4().to_string(),
        operation_type.to_string(),
        Some("demo_user".to_string()),
        "Success".to_string(),
        100,
    )
    .with_metadata(
        "operation_name".to_string(),
        serde_json::Value::String(operation_name.to_string()),
    )
    .with_metadata(
        "component".to_string(),
        serde_json::Value::String("logger_demo".to_string()),
    ))
}

/// Helper function to create complex activity logs with rich metadata
fn create_complex_activity(
    operation_name: &str,
    operation_type: &str,
) -> Result<ActivityLog, Box<dyn std::error::Error>> {
    Ok(ActivityLog::new(
        uuid::Uuid::new_v4().to_string(),
        operation_type.to_string(),
        Some("admin_user".to_string()),
        "Success".to_string(),
        250,
    )
    .with_metadata(
        "operation_name".to_string(),
        serde_json::Value::String(operation_name.to_string()),
    )
    .with_metadata(
        "request_id".to_string(),
        serde_json::Value::String("req_123456".to_string()),
    )
    .with_metadata(
        "client_info".to_string(),
        serde_json::json!({
            "ip": "192.168.1.100",
            "user_agent": "AirssysClient/2.0",
            "session_id": "sess_789"
        }),
    )
    .with_metadata(
        "performance".to_string(),
        serde_json::json!({
            "cpu_usage": 15.5,
            "memory_mb": 64,
            "response_time_ms": 250
        }),
    ))
}
