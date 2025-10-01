//! Log formatting and output utilities.
//!
//! This module provides utilities for formatting ActivityLog entries
//! into different output formats (JSON, pretty-printed, compact).

// Layer 1: Standard library imports
// (none for this module)

// Layer 2: Third-party crate imports
use serde_json;

// Layer 3: Internal module imports
use super::activity::ActivityLog;
use super::config::LogFormat;
use super::error::LogError;

/// Formats ActivityLog entries into specified output formats.
///
/// Provides different formatting strategies for activity logs based on
/// the intended use case (development, production, monitoring).
///
/// # Examples
///
/// ```rust
/// use airssys_osl::middleware::logger::{LogFormatter, LogFormat, ActivityLog};
/// use chrono::Utc;
/// use std::collections::HashMap;
///
/// let log = ActivityLog {
///     timestamp: Utc::now(),
///     operation_id: "op_123".to_string(),
///     operation_type: "file_read".to_string(),
///     user_context: Some("user123".to_string()),
///     result: "Success".to_string(),
///     duration_ms: 150,
///     metadata: HashMap::new(),
///     security_relevant: true,
/// };
///
/// let formatter = LogFormatter::new(LogFormat::Json);
/// let output = formatter.format(&log).unwrap();
/// ```
pub struct LogFormatter {
    format: LogFormat,
}

impl LogFormatter {
    /// Create a new formatter with the specified format.
    pub fn new(format: LogFormat) -> Self {
        Self { format }
    }

    /// Format an activity log entry into a string.
    pub fn format(&self, log: &ActivityLog) -> Result<String, LogError> {
        match self.format {
            LogFormat::Json => self.format_json(log),
            LogFormat::Pretty => self.format_pretty(log),
            LogFormat::Compact => self.format_compact(log),
        }
    }

    /// Format as structured JSON.
    fn format_json(&self, log: &ActivityLog) -> Result<String, LogError> {
        serde_json::to_string_pretty(log).map_err(|e| LogError::serialization(&log.operation_id, e))
    }

    /// Format as human-readable pretty format.
    fn format_pretty(&self, log: &ActivityLog) -> Result<String, LogError> {
        let security_marker = if log.security_relevant {
            " [SECURITY]"
        } else {
            ""
        };
        let user_info = log
            .user_context
            .as_ref()
            .map(|u| format!(" user={u}"))
            .unwrap_or_default();

        Ok(format!(
            "[{}] {} {}{}{} ({}ms)\n  Operation: {}\n  Result: {}{}",
            log.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            log.operation_type,
            log.operation_id,
            user_info,
            security_marker,
            log.duration_ms,
            log.operation_type,
            log.result,
            if log.metadata.is_empty() {
                String::new()
            } else {
                format!("\n  Metadata: {:?}", log.metadata)
            }
        ))
    }

    /// Format as compact single-line format.
    fn format_compact(&self, log: &ActivityLog) -> Result<String, LogError> {
        let security_marker = if log.security_relevant { "S" } else { "" };
        let user_info = log
            .user_context
            .as_ref()
            .map(|u| format!("u={u}"))
            .unwrap_or_else(|| "u=_".to_string());

        Ok(format!(
            "{} {} {} {} {} {}ms{}",
            log.timestamp.format("%H:%M:%S"),
            log.operation_type,
            log.operation_id,
            user_info,
            log.result,
            log.duration_ms,
            security_marker
        ))
    }
}

impl std::fmt::Debug for LogFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LogFormatter")
            .field("format", &self.format)
            .finish()
    }
}
