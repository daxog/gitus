use std::process::{Command, Output};

use crate::error::AppError;

/// Executes Git config get command
///
/// # Arguments
/// * `key` - Git config key (user.name or user.email)
pub fn get_git_user(key: &str) -> Result<String, AppError> {
    let git_command_output: Output = Command::new("git")
        .args(["config", "--get", key])
        .output()?;

    if !git_command_output.status.success() {
        return Err(AppError::GitCommand(
            String::from_utf8(git_command_output.stderr)?.trim().to_string(),
        ));
    }

    let value = String::from_utf8_lossy(&git_command_output.stdout).to_string();
    Ok(value)
}

/// Executes a Git config set command
///
/// # Arguments
/// * `key` - Git config key to set (user.name or user.email)
/// * `value` - Value to set for key (username or email)
pub fn set_git_config(key: &str, value: &str) -> Result<(), AppError> {
    let git_command_output: Output = Command::new("git").args(["config", key, value]).output()?;

    if !git_command_output.status.success() {
        return Err(AppError::GitCommand(
            String::from_utf8(git_command_output.stderr)?.trim().to_string(),
        ));
    }

    Ok(())  
}

/// Checks if current directory is in a Git repository for executing Git commands
pub fn is_inside_git_repo() -> Result<bool, AppError> {
    let git_command_output: Output = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()?;

    if !git_command_output.status.success() {
        return Err(AppError::GitCommand(
            String::from_utf8(git_command_output.stderr)?.trim().to_string(),
        ));
    }

    let value = String::from_utf8_lossy(&git_command_output.stdout).to_string();
    Ok(value.trim() == "true")
}
