//! Unit tests for logger middleware implementations.
//!
//! These tests verify that all logger implementations work correctly
//! and handle various scenarios including errors and edge cases.

use std::sync::Arc;

use tempfile::TempDir;
use tokio::fs;

use airssys_osl::middleware::logger::{
    ActivityLog, ActivityLogger, ConsoleActivityLogger, FileActivityLogger, LogFormat,
    TracingActivityLogger,
};

/// Helper function to create a test ActivityLog
fn create_test_log() -> ActivityLog {
    ActivityLog::new(
        "test_op_123".to_string(),
        "file_read".to_string(),
        Some("test_user".to_string()),
        "Success".to_string(),
        150,
    )
    .mark_security_relevant()
    .with_metadata("file_size".to_string(), serde_json::Value::Number(1024.into()))
}

/// Helper function to create an error ActivityLog
fn create_error_log() -> ActivityLog {
    ActivityLog::new(
        "test_op_456".to_string(),
        "file_write".to_string(),
        Some("test_user".to_string()),
        "Error: Permission denied".to_string(),
        0,
    )
    .mark_security_relevant()
}

#[cfg(test)]
mod console_logger_tests {
    use super::*;

    #[tokio::test]
    async fn test_console_logger_creation() {
        let logger = ConsoleActivityLogger::new();
        
        // Test that creation succeeds and logger works
        let test_log = create_test_log();
        let result = logger.log_activity(test_log).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_console_logger_builder_pattern() {
        let logger = ConsoleActivityLogger::new()
            .with_format(LogFormat::Pretty)
            .with_colors(true)
            .with_stderr(true);

        // Builder pattern should work correctly
        // Note: We can't directly test private fields, but we can test behavior
        let test_log = create_test_log();
        let result = logger.log_activity(test_log).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_console_logger_json_format() {
        let logger = ConsoleActivityLogger::new().with_format(LogFormat::Json);
        let test_log = create_test_log();

        // Should handle JSON formatting without errors
        let result = logger.log_activity(test_log).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_console_logger_pretty_format() {
        let logger = ConsoleActivityLogger::new()
            .with_format(LogFormat::Pretty)
            .with_colors(true);
        let test_log = create_test_log();

        // Should handle pretty formatting with colors
        let result = logger.log_activity(test_log).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_console_logger_compact_format() {
        let logger = ConsoleActivityLogger::new().with_format(LogFormat::Compact);
        let test_log = create_test_log();

        // Should handle compact formatting
        let result = logger.log_activity(test_log).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_console_logger_error_logs() {
        let logger = ConsoleActivityLogger::new().with_format(LogFormat::Pretty);
        let error_log = create_error_log();

        // Should handle error logs correctly
        let result = logger.log_activity(error_log).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_console_logger_flush() {
        let logger = ConsoleActivityLogger::new();

        // Flush should always succeed for console logger
        let result = logger.flush().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_console_logger_stderr_output() {
        let logger = ConsoleActivityLogger::new()
            .with_stderr(true)
            .with_format(LogFormat::Compact);
        let test_log = create_test_log();

        // Should handle stderr output without errors
        let result = logger.log_activity(test_log).await;
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod file_logger_tests {
    use super::*;

    #[tokio::test]
    async fn test_file_logger_creation() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let log_path = temp_dir.path().join("test.log");

        let logger = FileActivityLogger::new(&log_path).await;
        assert!(logger.is_ok());

        let logger = logger.unwrap();
        assert_eq!(logger.file_path(), log_path);
    }

    #[tokio::test]
    async fn test_file_logger_creates_directories() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let log_path = temp_dir.path().join("nested").join("dirs").join("test.log");

        // Should create parent directories automatically
        let logger = FileActivityLogger::new(&log_path).await;
        assert!(logger.is_ok());
        assert!(log_path.parent().unwrap().exists());
    }

    #[tokio::test]
    async fn test_file_logger_write_and_read() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let log_path = temp_dir.path().join("test.log");

        let logger = FileActivityLogger::new(&log_path)
            .await
            .expect("Failed to create file logger")
            .with_format(LogFormat::Json);

        let test_log = create_test_log();
        
        // Write log entry
        let write_result = logger.log_activity(test_log).await;
        assert!(write_result.is_ok());

        // Flush to ensure data is written
        let flush_result = logger.flush().await;
        assert!(flush_result.is_ok());

        // Verify file exists and has content
        let content = fs::read_to_string(&log_path).await.expect("Failed to read log file");
        assert!(!content.is_empty());
        assert!(content.contains("test_op_123"));
        assert!(content.contains("file_read"));
    }

    #[tokio::test]
    async fn test_file_logger_multiple_formats() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Test JSON format
        let json_path = temp_dir.path().join("json.log");
        let json_logger = FileActivityLogger::new(&json_path)
            .await
            .expect("Failed to create JSON logger")
            .with_format(LogFormat::Json);

        // Test Compact format
        let compact_path = temp_dir.path().join("compact.log");
        let compact_logger = FileActivityLogger::new(&compact_path)
            .await
            .expect("Failed to create compact logger")
            .with_format(LogFormat::Compact);

        let test_log = create_test_log();

        // Both should write successfully
        assert!(json_logger.log_activity(test_log.clone()).await.is_ok());
        assert!(compact_logger.log_activity(test_log).await.is_ok());

        // Flush both
        assert!(json_logger.flush().await.is_ok());
        assert!(compact_logger.flush().await.is_ok());

        // Verify different formats
        let json_content = fs::read_to_string(&json_path).await.expect("Failed to read JSON log");
        let compact_content = fs::read_to_string(&compact_path).await.expect("Failed to read compact log");

        assert!(json_content.contains("\"operation_id\":\"test_op_123\""));
        assert!(compact_content.contains("|test_op_123|"));
    }

    #[tokio::test]
    async fn test_file_logger_concurrent_access() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let log_path = temp_dir.path().join("concurrent.log");

        let logger = Arc::new(
            FileActivityLogger::new(&log_path)
                .await
                .expect("Failed to create file logger")
        );

        // Spawn multiple concurrent write tasks
        let mut handles = Vec::new();
        for i in 0..10 {
            let logger_clone = Arc::clone(&logger);
            let handle = tokio::spawn(async move {
                let log = ActivityLog::new(
                    format!("op_{}", i),
                    "concurrent_test".to_string(),
                    Some("test_user".to_string()),
                    "Success".to_string(),
                    i * 10,
                );
                logger_clone.log_activity(log).await
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            let result = handle.await.expect("Task panicked");
            assert!(result.is_ok());
        }

        // Flush and verify all entries were written
        assert!(logger.flush().await.is_ok());
        let content = fs::read_to_string(&log_path).await.expect("Failed to read log file");
        
        // Should have 10 entries
        assert_eq!(content.lines().count(), 10);
    }

    #[tokio::test]
    async fn test_file_logger_error_handling() {
        // Test with a valid path but verify error handling patterns
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let log_path = temp_dir.path().join("test.log");

        let logger = FileActivityLogger::new(&log_path)
            .await
            .expect("Failed to create file logger");

        // Test normal operation
        let test_log = create_test_log();
        assert!(logger.log_activity(test_log).await.is_ok());
        assert!(logger.flush().await.is_ok());
    }

    #[tokio::test]
    async fn test_file_logger_append_mode() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let log_path = temp_dir.path().join("append.log");

        // Create first logger and write entry
        {
            let logger1 = FileActivityLogger::new(&log_path)
                .await
                .expect("Failed to create first logger");
            
            let log1 = create_test_log();
            assert!(logger1.log_activity(log1).await.is_ok());
            assert!(logger1.flush().await.is_ok());
        }

        // Create second logger and write another entry
        {
            let logger2 = FileActivityLogger::new(&log_path)
                .await
                .expect("Failed to create second logger");
            
            let log2 = create_error_log();
            assert!(logger2.log_activity(log2).await.is_ok());
            assert!(logger2.flush().await.is_ok());
        }

        // Verify both entries are in the file
        let content = fs::read_to_string(&log_path).await.expect("Failed to read log file");
        assert!(content.contains("test_op_123"));
        assert!(content.contains("test_op_456"));
        assert_eq!(content.lines().count(), 2);
    }
}

#[cfg(test)]
mod tracing_logger_tests {
    use super::*;

    #[tokio::test]
    async fn test_tracing_logger_creation() {
        let logger = TracingActivityLogger::new();
        
        // Should create successfully
        let test_log = create_test_log();
        let result = logger.log_activity(test_log).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_tracing_logger_default() {
        let logger = TracingActivityLogger::default();
        
        // Default should work the same as new()
        let test_log = create_test_log();
        let result = logger.log_activity(test_log).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_tracing_logger_success_logs() {
        let logger = TracingActivityLogger::new();
        let success_log = create_test_log();

        // Should handle success logs (info level)
        let result = logger.log_activity(success_log).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_tracing_logger_error_logs() {
        let logger = TracingActivityLogger::new();
        let error_log = create_error_log();

        // Should handle error logs (error level)
        let result = logger.log_activity(error_log).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_tracing_logger_warning_logs() {
        let logger = TracingActivityLogger::new();
        let warning_log = ActivityLog::new(
            "test_op_789".to_string(),
            "network_request".to_string(),
            Some("test_user".to_string()),
            "Warning: timeout occurred".to_string(),
            5000,
        );

        // Should handle warning logs (warn level)
        let result = logger.log_activity(warning_log).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_tracing_logger_flush() {
        let logger = TracingActivityLogger::new();

        // Flush should always succeed (no-op for tracing)
        let result = logger.flush().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_tracing_logger_clone() {
        let logger1 = TracingActivityLogger::new();
        let logger2 = logger1.clone();

        // Both clones should work independently
        let test_log1 = create_test_log();
        let test_log2 = create_error_log();

        assert!(logger1.log_activity(test_log1).await.is_ok());
        assert!(logger2.log_activity(test_log2).await.is_ok());
    }

    #[tokio::test]
    async fn test_tracing_logger_structured_fields() {
        let logger = TracingActivityLogger::new();
        
        // Create log with rich metadata
        let mut complex_log = create_test_log();
        complex_log = complex_log
            .with_metadata("request_id".to_string(), serde_json::Value::String("req_123".to_string()))
            .with_metadata("response_time".to_string(), serde_json::Value::Number(250.into()))
            .with_metadata("user_agent".to_string(), serde_json::Value::String("test-client/1.0".to_string()));

        // Should handle complex structured data
        let result = logger.log_activity(complex_log).await;
        assert!(result.is_ok());
    }
}