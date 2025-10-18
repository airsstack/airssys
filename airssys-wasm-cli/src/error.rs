use thiserror::Error;

pub type Result<T> = std::result::Result<T, CliError>;

#[derive(Error, Debug)]
#[allow(dead_code)] // Error variants will be used when commands are fully implemented
pub enum CliError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Component not found: {0}")]
    ComponentNotFound(String),

    #[error("Installation failed: {0}")]
    InstallationFailed(String),

    #[error("Signature verification failed: {0}")]
    SignatureVerificationFailed(String),

    #[error("Build failed: {0}")]
    BuildFailed(String),

    #[error("Git operation failed: {0}")]
    GitError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<serde_json::Error> for CliError {
    fn from(err: serde_json::Error) -> Self {
        CliError::Serialization(err.to_string())
    }
}

impl From<toml::de::Error> for CliError {
    fn from(err: toml::de::Error) -> Self {
        CliError::Serialization(err.to_string())
    }
}

impl From<git2::Error> for CliError {
    fn from(err: git2::Error) -> Self {
        CliError::GitError(err.to_string())
    }
}

impl From<reqwest::Error> for CliError {
    fn from(err: reqwest::Error) -> Self {
        CliError::Network(err.to_string())
    }
}
