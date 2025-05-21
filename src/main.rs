//! Simple tool for quickly switching between multiple Git users.
//! User information is stored in a JSON file in the user's home directory.

mod cli;
mod error;
mod git;
mod menu;
mod profile;
mod storage;
mod validation;

use clap::Parser;
use cli::{Cli, Commands};
use colored::Colorize;
use error::AppError;
use git::{get_git_user, is_inside_git_repo, set_git_config};
use menu::run_menu;
use profile::GitUserProfile;
use storage::{check_if_users_exist, load_users, save_users};
use validation::{validate_input_alias, validate_input_email, validate_input_username};


/// Option text to return back to main menu
const BACK_OPTION: &str = "back";

/// Entry point for application
fn main() -> Result<(), AppError>  {
    if !is_inside_git_repo()? {
        return Err(AppError::NotInGitRepository);
    }

    if let Err(e) = run_app() {
        eprintln!("{}: {}", "error running app".red(), e);
    }

    Ok(())
}

/// Main application logic for command execution
fn run_app() -> Result<(), AppError> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Switch { user_alias }) => switch_user(&user_alias),
        Some(Commands::Add {
            git_username,
            git_email,
            user_alias,
        }) => add_user(&git_username, &git_email, &user_alias),
        Some(Commands::Delete { user_alias }) => delete_user(&user_alias),
        Some(Commands::Current) => show_current_user(),
        Some(Commands::List) => list_all_users(),
        None => run_menu(),
    }
}

// Switches current Git user to selected user profile
pub fn switch_user(user_alias: &str) -> Result<(), AppError> {
    let users: Vec<GitUserProfile> = load_users()?;
    check_if_users_exist(&users)?;

    if user_alias == BACK_OPTION {
        return Err(AppError::Validation("invalid alias for switching".to_string()));
    }

    if let Some(user) = users.iter().find(|user| user.user_alias == user_alias) {
        set_git_config("user.name", &user.git_username)?;
        set_git_config("user.email", &user.git_email)?;
        println!("{} {}", "switched to user:".green(), user.user_alias);
        Ok(())
    } else {
        Err(AppError::UserNotFound(user_alias.to_string()))
    }
}

/// Adds a new user profile to the stored profiles
fn add_user(git_username: &str, git_email: &str, user_alias: &str) -> Result<(), AppError> {
    let mut users: Vec<GitUserProfile> = load_users()?;

    // Input validation
    validate_input_username(git_username, &users)?;
    validate_input_email(git_email, &users)?;
    validate_input_alias(user_alias, &users)?;

    users.push(GitUserProfile {
        git_username: git_username.to_string(),
        git_email: git_email.to_string(),
        user_alias: user_alias.to_string(),
    });

    save_users(&users)?;
    print_success("added user");
    Ok(())
}

/// Deletes selected user profile from storage
fn delete_user(user_alias: &str) -> Result<(), AppError> {
    let mut users: Vec<GitUserProfile> = load_users()?;
    check_if_users_exist(&users)?;

    let initial_len = users.len();
    users.retain(|user| user.user_alias != user_alias);

    if users.len() == initial_len {
        return Err(AppError::UserNotFound(user_alias.to_string()));
    } 

    save_users(&users)?;
    print_success("deleted user");
    Ok(())
}

/// Shows current git user
fn show_current_user() -> Result<(), AppError> {
    let current_git_username: String = get_git_user("user.name")?;
    let current_git_email: String = get_git_user("user.email")?;

    println!("{} {} <{}>", "current user:".blue(), current_git_username.trim(), current_git_email.trim());
    Ok(())
}

/// List all users in storage file
fn list_all_users() -> Result<(), AppError> {
    let users: Vec<GitUserProfile> = load_users()?;
    check_if_users_exist(&users)?;

    for user in users {
        println!("{:?}", user);
    }
    Ok(())
}

/// Prints success message in green color
fn print_success(msg: &str) {
    println!("{}", msg.green());
}