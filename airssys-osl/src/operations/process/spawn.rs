//! Process spawn operation.

// Layer 1: Standard library imports
use std::collections::HashMap;
use std::fmt;

// Layer 2: Third-party crate imports
use chrono::{DateTime, Utc};
use uuid::Uuid;

// Layer 3: Internal module imports
use crate::core::operation::{Operation, OperationType, Permission};

/// Operation to spawn a new process.
///
/// Requires ProcessSpawn permission, which is an elevated privilege. This operation
/// supports command execution with arguments, environment variables, and working
/// directory configuration.
///
/// # Security
///
/// **This operation always requires elevated privileges** as it involves spawning
/// new system processes. The framework's security middleware will validate permissions
/// before execution.
///
/// # Examples
///
/// ```rust
/// use airssys_osl::operations::ProcessSpawnOperation;
///
/// // Basic command
/// let op = ProcessSpawnOperation::new("ls");
///
/// // Command with arguments
/// let op = ProcessSpawnOperation::new("echo")
///     .arg("Hello")
///     .arg("World");
///
/// // Command with environment and working directory
/// let op = ProcessSpawnOperation::new("cargo")
///     .arg("build")
///     .env("RUST_LOG", "debug")
///     .working_dir("/path/to/project");
/// ```
#[derive(Debug, Clone)]
pub struct ProcessSpawnOperation {
    /// Command to execute
    pub command: String,

    /// Command arguments
    pub args: Vec<String>,

    /// Environment variables (additions/overrides to inherited environment)
    pub env: HashMap<String, String>,

    /// Working directory (None = inherit from parent)
    pub working_dir: Option<String>,

    /// When this operation was created
    pub created_at: DateTime<Utc>,

    /// Optional operation ID
    pub operation_id: Option<String>,
}

impl ProcessSpawnOperation {
    /// Create a new process spawn operation.
    ///
    /// # Arguments
    ///
    /// * `command` - The command/program to execute
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::ProcessSpawnOperation;
    ///
    /// let op = ProcessSpawnOperation::new("ls");
    /// assert_eq!(op.command, "ls");
    /// assert!(op.args.is_empty());
    /// ```
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            env: HashMap::new(),
            working_dir: None,
            created_at: Utc::now(),
            operation_id: None,
        }
    }

    /// Set command arguments (replaces existing arguments).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::ProcessSpawnOperation;
    ///
    /// let op = ProcessSpawnOperation::new("ls")
    ///     .with_args(vec!["-la".to_string(), "/tmp".to_string()]);
    /// assert_eq!(op.args.len(), 2);
    /// ```
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Add a single argument.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::ProcessSpawnOperation;
    ///
    /// let op = ProcessSpawnOperation::new("echo")
    ///     .arg("Hello")
    ///     .arg("World");
    /// assert_eq!(op.args, vec!["Hello", "World"]);
    /// ```
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Set environment variables (replaces existing environment).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::ProcessSpawnOperation;
    /// use std::collections::HashMap;
    ///
    /// let mut env = HashMap::new();
    /// env.insert("RUST_LOG".to_string(), "debug".to_string());
    /// 
    /// let op = ProcessSpawnOperation::new("cargo")
    ///     .with_env(env);
    /// ```
    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = env;
        self
    }

    /// Add a single environment variable.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::ProcessSpawnOperation;
    ///
    /// let op = ProcessSpawnOperation::new("cargo")
    ///     .env("RUST_LOG", "debug")
    ///     .env("RUST_BACKTRACE", "1");
    /// assert_eq!(op.env.len(), 2);
    /// ```
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set working directory for the spawned process.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use airssys_osl::operations::ProcessSpawnOperation;
    ///
    /// let op = ProcessSpawnOperation::new("cargo")
    ///     .arg("build")
    ///     .working_dir("/path/to/project");
    /// assert_eq!(op.working_dir, Some("/path/to/project".to_string()));
    /// ```
    pub fn working_dir(mut self, dir: impl Into<String>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Create with explicit timestamp (for testing).
    pub fn with_timestamp(
        command: impl Into<String>,
        args: Vec<String>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            command: command.into(),
            args,
            env: HashMap::new(),
            working_dir: None,
            created_at,
            operation_id: None,
        }
    }

    /// Set custom operation ID.
    pub fn with_operation_id(mut self, id: impl Into<String>) -> Self {
        self.operation_id = Some(id.into());
        self
    }
}

impl Operation for ProcessSpawnOperation {
    fn operation_type(&self) -> OperationType {
        OperationType::Process
    }

    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::ProcessSpawn]
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn operation_id(&self) -> String {
        self.operation_id.clone().unwrap_or_else(|| {
            format!("{}:{}", self.operation_type().as_str(), Uuid::new_v4())
        })
    }

    fn requires_elevated_privileges(&self) -> bool {
        true // Process spawning always requires elevation
    }
}

impl fmt::Display for ProcessSpawnOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.args.is_empty() {
            write!(f, "ProcessSpawn({})", self.command)
        } else {
            write!(
                f,
                "ProcessSpawn({} [{}])",
                self.command,
                self.args.join(" ")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_spawn_operation_new() {
        let op = ProcessSpawnOperation::new("ls");
        assert_eq!(op.command, "ls");
        assert!(op.args.is_empty());
        assert!(op.env.is_empty());
        assert_eq!(op.working_dir, None);
    }

    #[test]
    fn test_process_spawn_with_args() {
        let op = ProcessSpawnOperation::new("echo")
            .arg("Hello")
            .arg("World");
        assert_eq!(op.command, "echo");
        assert_eq!(op.args, vec!["Hello", "World"]);
    }

    #[test]
    fn test_process_spawn_with_env() {
        let op = ProcessSpawnOperation::new("cargo")
            .env("RUST_LOG", "debug")
            .env("RUST_BACKTRACE", "1");
        assert_eq!(op.env.len(), 2);
        assert_eq!(op.env.get("RUST_LOG"), Some(&"debug".to_string()));
        assert_eq!(op.env.get("RUST_BACKTRACE"), Some(&"1".to_string()));
    }

    #[test]
    fn test_process_spawn_with_working_dir() {
        let op = ProcessSpawnOperation::new("cargo")
            .working_dir("/tmp/project");
        assert_eq!(op.working_dir, Some("/tmp/project".to_string()));
    }

    #[test]
    fn test_process_spawn_permissions() {
        let op = ProcessSpawnOperation::new("test");
        let permissions = op.required_permissions();
        assert_eq!(permissions.len(), 1);
        assert_eq!(permissions[0], Permission::ProcessSpawn);
    }

    #[test]
    fn test_process_spawn_requires_elevation() {
        let op = ProcessSpawnOperation::new("test");
        assert!(op.requires_elevated_privileges());
    }

    #[test]
    fn test_process_spawn_operation_type() {
        let op = ProcessSpawnOperation::new("test");
        assert_eq!(op.operation_type(), OperationType::Process);
    }
}
