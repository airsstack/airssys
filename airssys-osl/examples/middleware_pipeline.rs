//! Middleware pipeline integration examples.
//!
//! This example demonstrates how to integrate logger middleware into
//! processing pipelines and basic logger usage patterns.

#![allow(clippy::unwrap_used, clippy::expect_used)] // Allow in examples for clarity

use airssys_osl::middleware::logger::{
    ActivityLog, ActivityLogger, ConsoleActivityLogger, FileActivityLogger, LogFormat,
    LoggerConfig, LoggerMiddleware, TracingActivityLogger,
};
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 AirssysOSL Middleware Pipeline Integration Examples\n");

    // Initialize tracing for examples
    tracing_subscriber::fmt::init();

    // 1. Basic Logger Usage
    basic_logger_usage().await?;

    // 2. Logger Middleware Examples
    logger_middleware_examples().await?;

    // 3. Configuration Examples
    logger_configuration_examples().await?;

    // 4. Concurrent Logging Examples
    concurrent_logging_examples().await?;

    println!("✅ All middleware pipeline examples completed successfully!");
    Ok(())
}

/// Demonstrates basic logger usage patterns
async fn basic_logger_usage() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Basic Logger Usage Examples");
    println!("===============================\n");

    // Console logger with different formats
    println!("1. Console Logger with JSON format:");
    let json_logger = ConsoleActivityLogger::new().with_format(LogFormat::Json);
    let activity = create_sample_activity("JSON Example", "demonstration")?;
    json_logger.log_activity(activity).await?;

    println!("\n2. Console Logger with Pretty format:");
    let pretty_logger = ConsoleActivityLogger::new().with_format(LogFormat::Pretty);
    let activity = create_sample_activity("Pretty Example", "demonstration")?;
    pretty_logger.log_activity(activity).await?;

    println!("\n3. Console Logger with Compact format:");
    let compact_logger = ConsoleActivityLogger::new().with_format(LogFormat::Compact);
    let activity = create_sample_activity("Compact Example", "demonstration")?;
    compact_logger.log_activity(activity).await?;

    println!("\n✅ Basic logger usage examples completed\n");
    Ok(())
}

/// Demonstrates logger middleware creation and usage
async fn logger_middleware_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Logger Middleware Examples");
    println!("==============================\n");

    // Create temporary directory for file logging
    let temp_dir = TempDir::new()?;

    // Create different logger implementations
    let console_logger = ConsoleActivityLogger::new().with_format(LogFormat::Pretty);
    let file_logger = FileActivityLogger::new(temp_dir.path().join("middleware.log")).await?;
    let tracing_logger = TracingActivityLogger::new();

    // Create middleware instances
    let console_middleware = LoggerMiddleware::with_default_config(console_logger);
    let file_middleware = LoggerMiddleware::with_default_config(file_logger);
    let tracing_middleware = LoggerMiddleware::with_default_config(tracing_logger);

    // Create a sample activity
    let activity = create_sample_activity("Middleware Demo", "pipeline_processing")?;

    println!("Processing activity through different middleware loggers...");

    // Log through console middleware
    println!("1. Console middleware:");
    console_middleware
        .logger()
        .log_activity(activity.clone())
        .await?;
    println!("   → Logged to console");

    // Log through file middleware
    println!("2. File middleware:");
    file_middleware
        .logger()
        .log_activity(activity.clone())
        .await?;
    println!(
        "   → Logged to file: {:?}",
        temp_dir.path().join("middleware.log")
    );

    // Log through tracing middleware
    println!("3. Tracing middleware:");
    tracing_middleware.logger().log_activity(activity).await?;
    println!("   → Logged through tracing");

    println!("✅ Logger middleware examples completed\n");
    Ok(())
}

/// Demonstrates various logger configuration patterns
async fn logger_configuration_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚙️ Logger Configuration Examples");
    println!("=================================\n");

    // Example 1: Default configuration
    println!("1. Default LoggerConfig:");
    let config1 = LoggerConfig::default();
    println!("   → Format: {:?}", config1.format);
    println!("   → Level: {:?}", config1.level);
    println!("   → Buffer size: {}", config1.buffer_size);
    println!("   → Flush interval: {}ms", config1.flush_interval_ms);

    // Example 2: Custom configuration
    println!("\n2. Custom LoggerConfig:");
    let config2 = LoggerConfig {
        format: LogFormat::Json,
        buffer_size: 500,
        flush_interval_ms: 1000,
        ..LoggerConfig::default()
    };
    println!("   → Format: {:?}", config2.format);
    println!("   → Buffer size: {}", config2.buffer_size);
    println!("   → Flush interval: {}ms", config2.flush_interval_ms);

    // Example 3: File logger with custom configuration
    println!("\n3. File Logger with Custom Config:");
    let temp_dir = TempDir::new()?;
    let file_logger = FileActivityLogger::new(temp_dir.path().join("config.log"))
        .await?
        .with_format(LogFormat::Pretty);

    let config_activity = create_complex_activity("Config Demo", "configuration_test")?;
    file_logger.log_activity(config_activity).await?;
    file_logger.flush().await?;
    println!("   → File logged with pretty format");

    println!("✅ Logger configuration examples completed\n");
    Ok(())
}

/// Demonstrates concurrent logging scenarios
async fn concurrent_logging_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Concurrent Logging Examples");
    println!("===============================\n");

    // Create shared logger instances
    let temp_dir = TempDir::new()?;
    let file_logger = std::sync::Arc::new(
        FileActivityLogger::new(temp_dir.path().join("concurrent.log"))
            .await?
            .with_format(LogFormat::Json),
    );

    // Spawn multiple concurrent logging tasks
    let mut handles = Vec::new();
    for i in 0..5 {
        let logger = std::sync::Arc::clone(&file_logger);
        let handle = tokio::spawn(async move {
            for j in 0..3 {
                let activity = create_sample_activity(
                    &format!("Concurrent Task {i}"),
                    &format!("operation_{j}"),
                )
                .unwrap();
                logger.log_activity(activity).await.unwrap();
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    file_logger.flush().await?;
    println!("   → Completed 15 concurrent logging operations");

    // Read and display some of the logged content
    let content = tokio::fs::read_to_string(temp_dir.path().join("concurrent.log")).await?;
    let lines: Vec<&str> = content.lines().take(3).collect();
    println!("   → Sample logged entries (first 3):");
    for line in lines {
        println!("     {line}");
    }

    println!("✅ Concurrent logging examples completed\n");
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
        150,
    )
    .with_metadata(
        "operation_name".to_string(),
        serde_json::Value::String(operation_name.to_string()),
    )
    .with_metadata(
        "example_data".to_string(),
        serde_json::Value::String("sample_value".to_string()),
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
        serde_json::Value::String("req789".to_string()),
    )
    .with_metadata(
        "client_ip".to_string(),
        serde_json::Value::String("192.168.1.100".to_string()),
    )
    .with_metadata(
        "user_agent".to_string(),
        serde_json::Value::String("AirssysClient/1.0".to_string()),
    )
    .with_metadata(
        "processing_time_ms".to_string(),
        serde_json::Value::Number(serde_json::Number::from(250)),
    )
    .with_metadata(
        "data_size_bytes".to_string(),
        serde_json::Value::Number(serde_json::Number::from(1024)),
    ))
}
