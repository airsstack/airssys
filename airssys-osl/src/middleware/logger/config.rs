//! Configuration types for logger middleware.
//!
//! This module defines configuration structures and enums for customizing
//! logger behavior, formats, and performance characteristics.

// Layer 1: Standard library imports
// (none for this module)

// Layer 2: Third-party crate imports
use serde::{Deserialize, Serialize};

// Layer 3: Internal module imports
// (none for this module)

/// Configuration for logger middleware behavior.
///
/// Controls various aspects of logging including output format, buffering,
/// and performance characteristics. Provides sensible defaults for common
/// use cases while allowing fine-tuned customization.
///
/// # Examples
///
/// ```rust
/// use airssys_osl::middleware::logger::{LoggerConfig, LogLevel, LogFormat};
///
/// let config = LoggerConfig {
///     level: LogLevel::Info,
///     format: LogFormat::Json,
///     buffer_size: 1000,
///     flush_interval_ms: 5000,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggerConfig {
    /// Minimum log level to record
    pub level: LogLevel,
    
    /// Output format for log entries
    pub format: LogFormat,
    
    /// Buffer size for batching log entries (0 = unbuffered)
    pub buffer_size: usize,
    
    /// Automatic flush interval in milliseconds (0 = manual flush only)
    pub flush_interval_ms: u64,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            format: LogFormat::Json,
            buffer_size: 1000,
            flush_interval_ms: 5000,
        }
    }
}

// LoggerConfig uses Default trait for sensible universal defaults
// Users can customize with struct update syntax: LoggerConfig { level: LogLevel::Debug, ..Default::default() }

/// Log level enumeration for filtering log entries.
///
/// Determines which activity logs should be recorded based on their
/// importance level. Higher levels include all lower levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    /// Only log errors and critical failures
    Error,
    
    /// Log warnings and errors
    Warn,
    
    /// Log informational messages, warnings, and errors (default)
    Info,
    
    /// Log debug information and all higher levels
    Debug,
    
    /// Log all activity including detailed tracing information
    Trace,
}

impl LogLevel {
    /// Check if this level should log messages at the given level.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::middleware::logger::LogLevel;
    ///
    /// assert!(LogLevel::Debug.should_log(LogLevel::Info));
    /// assert!(!LogLevel::Info.should_log(LogLevel::Debug));
    /// ```
    pub fn should_log(self, level: LogLevel) -> bool {
        level <= self
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Trace => write!(f, "TRACE"),
        }
    }
}

/// Output format options for log entries.
///
/// Determines how activity logs are formatted for output. Different formats
/// are suitable for different use cases (development, production, monitoring).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogFormat {
    /// Structured JSON format for machine processing
    Json,
    
    /// Human-readable pretty format for development
    Pretty,
    
    /// Compact single-line format for space efficiency
    Compact,
}

impl std::fmt::Display for LogFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogFormat::Json => write!(f, "json"),
            LogFormat::Pretty => write!(f, "pretty"),
            LogFormat::Compact => write!(f, "compact"),
        }
    }
}