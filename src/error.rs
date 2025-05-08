use thiserror::Error;

/// Application error types
#[derive(Error, Debug)]
pub enum AppError {
    /// Error during file I/O operations
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    /// Error during JSON serialization or deserialization
    #[error("json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    /// Error when user input fails.
    #[error("inquire error: {0}")]
    Inquire(#[from] inquire::InquireError),
    /// Error when executing Git commands
    #[error("git command failed: {0}")]
    GitCommand(String),
    /// Error when current directory is not a Git repository
    # [error("not in git repository")]
    NotInGitRepository,
    /// Error during input validation.
    #[error("validation error: {0}")]
    Validation(String),
    /// Error when specific user alias is not found.
    #[error("user alias not found: '{0}'")]
    UserNotFound(String),
    /// Error during UTF-8 conversion.
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}