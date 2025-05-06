use std::{env, fs, io::Read, path::Path, process::{Command, Output}};

use clap::{command, Parser, Subcommand};
use colored::Colorize;
use inquire::{Select, Text};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use validator::ValidateEmail;

// Constants
const GLOBAL_GIT_PROFILES_PATH: &str = "user_profiles.json";
const MAX_USERNAME_LENGTH: usize = 30;
const MAX_EMAIL_LENGTH: usize = 100;
const MAX_ALIAS_LENGTH: usize = 30;
const BACK_OPTION: &str = "back";

// Error 
#[derive(Error, Debug)]
enum AppError {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("inquire error: {0}")]
    Inquire(#[from] inquire::InquireError),

    #[error("git command failed: {0}")]
    GitCommand(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("user alias not found: '{0}'")]
    UserNotFound(String),

    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

// Structs
#[derive(Serialize, Deserialize, Debug)]
struct GitUserProfile {
    git_username: String,
    git_email: String,
    user_alias: String,
}

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

// Subcommands
#[derive(Subcommand, Debug)]
enum Commands {
    Switch {
        user_alias: String,
    },
    Add {
        git_username: String,
        git_email: String,
        user_alias: String,
    },
    Delete {
        user_alias: String,
    },
    Current,
    List,
}

// Main
fn main() {
    if let Err(e) = run_app() {
        eprintln!("{}: {}", "error running app".red(), e);
    }
}

// Main run loop
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

fn switch_user(user_alias: &str) -> Result<(), AppError> {
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

// Menu functions
fn run_menu() -> Result<(), AppError> {
    loop {
        let actions = vec![
            "switch user", 
            "add user", 
            "delete user", 
            "show current user",
            "show all users", 
            "quit"
        ];

        let action_selected= Select::new(&format!("{}", "select action".blue()), actions)
            .prompt()?;

        match action_selected {
            "switch user" => menu_switch_user()?,
            "add user" => menu_add_user()?,
            "delete user" => menu_delete_user()?,
            "show current user" => show_current_user()?,
            "show all users" => list_all_users()?,
            "quit" => {
                println!("{}", "quitting".yellow());
                break Ok(());
            },
            _ => unreachable!("unexpected input"),
        }
    }
} 

fn menu_switch_user() -> Result<(), AppError>  {
    let users: Vec<GitUserProfile> = load_users()?;
    check_if_users_exist(&users)?;

    let user_aliases: Vec<String> = build_alias_list(&users);
    let alias_to_switch: String = Select::new(&format!("{}", "select user to switch:".blue()), user_aliases)
        .prompt()?;

    if alias_to_switch != BACK_OPTION {
        switch_user(&alias_to_switch)?;
    }
    
    Ok(())
}

fn menu_add_user() -> Result<(), AppError> {
    let users: Vec<GitUserProfile> = load_users()?;

    // Input validation
    let username: String = prompt_until_valid(
        "enter git username:",
        |input| validate_input_username(input, &users),
    )?;

    let email: String = prompt_until_valid(
        "enter git email:", 
        |input| validate_input_email(input, &users)
    )?;

    let alias: String = prompt_until_valid(
        "enter alias:", 
        |input| validate_input_alias(input, &users)
    )?;
    
    add_user(&username, &email, &alias)?;
    print_success("added user");
    Ok(())
}

fn menu_delete_user() -> Result<(), AppError> {
    let users: Vec<GitUserProfile> = load_users()?;
    check_if_users_exist(&users)?;

    let user_aliases: Vec<String> = build_alias_list(&users);
    let alias_to_delete: String = Select::new(&format!("{}", "select user to delete:".blue()), user_aliases)
        .prompt()?;

    if alias_to_delete != BACK_OPTION {
        delete_user(&alias_to_delete)?;
    }
    
    Ok(())
}

// Shows current git user
fn show_current_user() -> Result<(), AppError> {
    let current_git_username: String = get_git_user("user.name")?;
    let current_git_email: String = get_git_user("user.email")?;

    println!("{} {} <{}>", "current user:".blue(), current_git_username.trim(), current_git_email.trim());
    Ok(())
}

// List all users in json file
fn list_all_users() -> Result<(), AppError> {
    let users: Vec<GitUserProfile> = load_users()?;
    check_if_users_exist(&users)?;

    for user in users {
        println!("{:?}", user);
    }
    Ok(())
}

// Storage helper functions
fn get_global_profile_file_path() -> Result<String, AppError> {
    let home_dir: String = env::var("HOME").unwrap_or_else(|_| String::from("."));
    let global_profile_file = format!("{}/{}", home_dir, GLOBAL_GIT_PROFILES_PATH);
    Ok(global_profile_file)
}

fn load_users() -> Result<Vec<GitUserProfile>, AppError> {
    let profile_file_path: String = get_global_profile_file_path()?;

    if !Path::new(&profile_file_path).exists() {
        return Ok(Vec::new());
    }

    let mut file = fs::File::open(profile_file_path)?;

    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents).expect("failed to read file");

    if file_contents.trim().is_empty() {
        return Ok(Vec::new());
    }

    Ok(serde_json::from_str(&file_contents)?)
}

fn save_users(users: &[GitUserProfile]) -> Result<(), AppError>  {
    let profile_file_path: String = get_global_profile_file_path()?;
    let json: String = serde_json::to_string_pretty(users)?;
    fs::write(profile_file_path, json)?;
    Ok(())
}

// Git commands
fn get_git_user(key: &str) -> Result<String, AppError> {
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

fn set_git_config(key: &str, value: &str) -> Result<(), AppError> {
    let git_command_output: Output = Command::new("git").args(["config", key, value]).output()?;

    if !git_command_output.status.success() {
        return Err(AppError::GitCommand(
            String::from_utf8(git_command_output.stderr)?.trim().to_string(),
        ));
    }

    Ok(())  
}

// Build alias list for menu
fn build_alias_list(users: &[GitUserProfile]) -> Vec<String> {
    let mut user_aliases: Vec<String> = users.iter()
        .map(|user| user.user_alias.clone())
        .collect();
    user_aliases.push(BACK_OPTION.to_string());
    user_aliases
}

// Check if users exist
fn check_if_users_exist(users: &[GitUserProfile]) -> Result<(), AppError> {
    if users.is_empty() {
        return Err(AppError::Validation("no users found".to_string()));
    }
    Ok(())
}

// Helper function to repeatedly prompt to get valid input
fn prompt_until_valid<F>(prompt_message: &str, input_validation: F) -> Result<String, AppError>
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
fn validate_input_username(name: &str, existing_users: &[GitUserProfile]) -> Result<(), AppError> {
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

fn validate_input_email(email: &str, existing_users: &[GitUserProfile]) -> Result<(), AppError> {
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

fn validate_input_alias(alias: &str, existing_users: &[GitUserProfile]) -> Result<(), AppError> {
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

// Print helper functions
fn print_success(msg: &str) {
    println!("{}", msg.green());
}