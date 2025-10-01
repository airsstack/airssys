//! Console-based activity logger implementation.
//!
//! This module provides a logger that outputs activity logs to the console
//! with optional pretty-printing for development and debugging.

// Layer 1: Standard library imports
use std::io::{self, Write};

// Layer 2: Third-party crate imports
use async_trait::async_trait;
use serde_json;

// Layer 3: Internal module imports
use crate::middleware::logger::activity::{ActivityLog, ActivityLogger};
use crate::middleware::logger::config::LogFormat;
use crate::middleware::logger::error::LogError;

/// Console activity logger with configurable formatting.
///
/// Outputs activity logs directly to stdout with configurable formatting
/// for development, debugging, and production console scenarios.
///
/// # Features
///
/// - **Multiple formats**: JSON, pretty-printed, or compact text
/// - **Color support**: Optional colored output for better readability
/// - **Output destination**: Choose between stdout and stderr
/// - **Async-safe**: Safe to use in async contexts without blocking
///
/// # Examples
///
/// ```rust,ignore
/// use airssys_osl::middleware::logger::loggers::ConsoleActivityLogger;
/// use airssys_osl::middleware::logger::config::LogFormat;
///
/// // Create a console logger with pretty printing
/// let logger = ConsoleActivityLogger::new()
///     .with_format(LogFormat::Pretty)
///     .with_colors(true);
/// ```
#[derive(Debug, Clone)]
pub struct ConsoleActivityLogger {
    format: LogFormat,
    use_colors: bool,
    use_stderr: bool,
}

impl Default for ConsoleActivityLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsoleActivityLogger {
    /// Create a new console logger with default settings.
    ///
    /// Defaults to JSON format with no colors and stdout output.
    pub fn new() -> Self {
        Self {
            format: LogFormat::Json,
            use_colors: false,
            use_stderr: false,
        }
    }

    /// Configure the output format for log entries.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use airssys_osl::middleware::logger::config::LogFormat;
    ///
    /// let logger = ConsoleActivityLogger::new()
    ///     .with_format(LogFormat::Pretty);
    /// ```
    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    /// Enable or disable colored output.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let logger = ConsoleActivityLogger::new()
    ///     .with_colors(true);
    /// ```
    pub fn with_colors(mut self, use_colors: bool) -> Self {
        self.use_colors = use_colors;
        self
    }

    /// Configure whether to use stderr instead of stdout.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let logger = ConsoleActivityLogger::new()
    ///     .with_stderr(true);
    /// ```
    pub fn with_stderr(mut self, use_stderr: bool) -> Self {
        self.use_stderr = use_stderr;
        self
    }

    /// Format an activity log entry according to the configured format.
    fn format_log(&self, log: &ActivityLog) -> String {
        match self.format {
            LogFormat::Json => {
                // Serialize to JSON - use to_string() to avoid unwrap
                serde_json::to_string(log).unwrap_or_else(|_| {
                    format!(
                        r#"{{"error":"Failed to serialize log","operation_id":"{}"}}"#,
                        log.operation_id
                    )
                })
            }
            LogFormat::Pretty => {
                // Pretty-printed format for development
                let color_reset = if self.use_colors { "\x1b[0m" } else { "" };
                let (status_color, level_prefix) = if log.result.starts_with("Error") {
                    (if self.use_colors { "\x1b[31m" } else { "" }, "ERROR")
                } else {
                    (if self.use_colors { "\x1b[32m" } else { "" }, "INFO ")
                };

                format!(
                    "{}{}{} [{}] {} ({}) - {} ({}ms){}",
                    status_color,
                    level_prefix,
                    color_reset,
                    log.timestamp.format("%H:%M:%S%.3f"),
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
                // Compact single-line format
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
                    log.result,
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
impl ActivityLogger for ConsoleActivityLogger {
    async fn log_activity(&self, log: ActivityLog) -> Result<(), LogError> {
        // Format the log entry
        let formatted = self.format_log(&log);

        // Write to stdout or stderr based on configuration
        // Note: println!/eprintln! macros handle errors internally, so we don't get io::Error
        if self.use_stderr {
            eprintln!("{formatted}");
        } else {
            println!("{formatted}");
        }

        // Flush immediately for console output to ensure visibility
        let flush_result = if self.use_stderr {
            io::stderr().flush()
        } else {
            io::stdout().flush()
        };

        flush_result.map_err(|e| LogError::io("flush", "console", e))
    }

    async fn flush(&self) -> Result<(), LogError> {
        // Flush both stdout and stderr to be safe
        io::stdout()
            .flush()
            .and_then(|_| io::stderr().flush())
            .map_err(|e| LogError::io("flush", "console", e))
    }
}
