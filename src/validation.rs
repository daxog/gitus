use colored::Colorize;
use inquire::Text;
use validator::ValidateEmail;

use crate::{error::AppError, GitUserProfile, BACK_OPTION};

/// Maximum length for Git username
const MAX_USERNAME_LENGTH: usize = 30;
/// Maximum length for Git email address
const MAX_EMAIL_LENGTH: usize = 100;
/// Maximum length for user alias
const MAX_ALIAS_LENGTH: usize = 30;

/// Prompts user for input until valid input is provided
pub fn prompt_until_valid<F>(prompt_message: &str, input_validation: F) -> Result<String, AppError>
where
    F: Fn(&str) -> Result<(), AppError>,
{
    loop {
        let input: String = Text::new(prompt_message).prompt()?;
        match input_validation(&input) {
            Ok(_) => break Ok(input),
            Err(AppError::Validation(msg)) => println!("{}", msg.red()),
            Err(e) => return Err(e), 
        }
    }
}

// Validate input helper functions

/// Validates username input
pub fn validate_input_username(name: &str, existing_users: &[GitUserProfile]) -> Result<(), AppError> {
    if name.is_empty() {
        Err(AppError::Validation("Username cannot be empty".to_string()))
    } else if name.len() > MAX_USERNAME_LENGTH {
        Err(AppError::Validation(format!("username too long, max {} characters)", MAX_USERNAME_LENGTH)))
    } else if existing_users.iter().any(|user| user.git_username == name) {
        Err(AppError::Validation("Username already exists".to_string()))
    } else {
        Ok(())
    }
}

/// Validates email input
pub fn validate_input_email(email: &str, existing_users: &[GitUserProfile]) -> Result<(), AppError> {
    if email.is_empty() {
        Err(AppError::Validation("Email cannot be empty".to_string()))
    } else if email.len() > MAX_EMAIL_LENGTH {
        Err(AppError::Validation(format!("email too long, max {} characters",MAX_EMAIL_LENGTH)))
    } else if !email.validate_email() {
        Err(AppError::Validation("Invalid email format".to_string()))
    } else if existing_users.iter().any(|user| user.git_email == email) {
        Err(AppError::Validation("Email already exists".to_string()))
    } else {
        Ok(())
    }
}

/// Validates an alias input
pub fn validate_input_alias(alias: &str, existing_users: &[GitUserProfile]) -> Result<(), AppError> {
    if alias.is_empty() {
        Err(AppError::Validation("Alias cannot be empty".to_string()))
    } else if alias.len() > MAX_ALIAS_LENGTH {
        Err(AppError::Validation(format!("Alias too long (max {} characters)",MAX_ALIAS_LENGTH)))
    } else if alias == BACK_OPTION {
        Err(AppError::Validation("Alias cannot be 'back'".to_string()))
    } else if existing_users.iter().any(|user| user.user_alias == alias) {
        Err(AppError::Validation("Alias already exists".to_string()))
    } else {
        Ok(())
    }
}