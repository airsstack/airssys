//! File-based activity logger implementation.
//!
//! This module provides a logger that outputs activity logs to files
//! with async I/O for production logging scenarios.

// Layer 1: Standard library imports
use std::path::{Path, PathBuf};

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use serde_json;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::Mutex;

// Layer 3: Internal module imports
use crate::middleware::logger::activity::{ActivityLog, ActivityLogger};
use crate::middleware::logger::config::LogFormat;
use crate::middleware::logger::error::LogError;

/// File-based activity logger with async I/O and buffering.
///
/// Outputs activity logs to specified files with async I/O for production
/// logging scenarios. Uses buffered writing for performance and proper
/// error handling for file operations.
///
/// # Features
///
/// - **Async I/O**: Non-blocking file operations using tokio
/// - **Buffered Writing**: Efficient batched writes for performance
/// - **Configurable Format**: JSON or compact text output
/// - **Error Handling**: Comprehensive file operation error handling
/// - **Thread Safety**: Safe concurrent access with async mutex
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_osl::middleware::logger::loggers::FileActivityLogger;
/// use airssys_osl::middleware::logger::config::LogFormat;
///
/// // Create a file logger with JSON output
/// let logger = FileActivityLogger::new("/var/log/activity.log")
///     .await
///     .expect("Failed to create file logger")
///     .with_format(LogFormat::Json);
/// ```
#[derive(Debug)]
pub struct FileActivityLogger {
    file_path: PathBuf,
    format: LogFormat,
    writer: Mutex<BufWriter<File>>,
}

impl FileActivityLogger {
    /// Create a new file logger that writes to the specified path.
    ///
    /// Creates the file if it doesn't exist, or appends to existing file.
    /// Uses JSON format by default.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let logger = FileActivityLogger::new("/var/log/activity.log").await?;
    /// ```
    pub async fn new<P: AsRef<Path>>(file_path: P) -> Result<Self, LogError> {
        let path = file_path.as_ref().to_path_buf();

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| LogError::io("create_dir", parent.display().to_string(), e))?;
        }

        // Open file for append, create if doesn't exist
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await
            .map_err(|e| LogError::io("open", path.display().to_string(), e))?;

        let writer = BufWriter::new(file);

        Ok(Self {
            file_path: path,
            format: LogFormat::Json,
            writer: Mutex::new(writer),
        })
    }

    /// Configure the output format for log entries.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let logger = logger.with_format(LogFormat::Compact);
    /// ```
    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    /// Get the file path being written to.
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    /// Format an activity log entry according to the configured format.
    fn format_log(&self, log: &ActivityLog) -> String {
        match self.format {
            LogFormat::Json => {
                // Single-line JSON for easy parsing
                serde_json::to_string(log).unwrap_or_else(|_| {
                    format!(
                        r#"{{"error":"Failed to serialize log","operation_id":"{}"}}"#,
                        log.operation_id
                    )
                })
            }
            LogFormat::Pretty => {
                // Pretty format not ideal for files, but support it anyway
                format!(
                    "[{}] {} ({}) - {} ({}ms){}",
                    log.timestamp.to_rfc3339(),
                    log.operation_id,
                    log.user_context.as_deref().unwrap_or("system"),
                    log.result,
                    log.duration_ms,
                    if log.metadata.is_empty() {
                        String::new()
                    } else {
                        format!(" | metadata: {:?}", log.metadata)
                    }
                )
            }
            LogFormat::Compact => {
                // Space-efficient format for high-volume logging
                format!(
                    "{}|{}|{}|{}|{}|{}ms{}",
                    log.timestamp.to_rfc3339(),
                    if log.result.starts_with("Error") {
                        "ERR"
                    } else {
                        "OK"
                    },
                    log.operation_id,
                    log.user_context.as_deref().unwrap_or("sys"),
                    log.result.replace('|', "_"), // Escape pipe characters
                    log.duration_ms,
                    if log.metadata.is_empty() {
                        String::new()
                    } else {
                        format!("|meta:{}", log.metadata.len())
                    }
                )
            }
        }
    }
}

#[async_trait]
impl ActivityLogger for FileActivityLogger {
    async fn log_activity(&self, log: ActivityLog) -> Result<(), LogError> {
        let formatted = self.format_log(&log);
        let line = format!("{formatted}\n");

        // Get exclusive access to the writer
        let mut writer = self.writer.lock().await;

        // Write the log line
        writer
            .write_all(line.as_bytes())
            .await
            .map_err(|e| LogError::io("write", self.file_path.display().to_string(), e))?;

        // Note: We don't flush on every write for performance
        // flush() will be called explicitly when needed
        Ok(())
    }

    async fn flush(&self) -> Result<(), LogError> {
        let mut writer = self.writer.lock().await;

        // Flush the buffer to ensure all data is written
        writer
            .flush()
            .await
            .map_err(|e| LogError::io("flush", self.file_path.display().to_string(), e))
    }
}
