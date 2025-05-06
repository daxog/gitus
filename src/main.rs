use std::{fs::{self, OpenOptions}, io::Read, path::Path, process::{Command, Output}};

use clap::{command, Parser, Subcommand};
use colored::Colorize;
use inquire::{Select, Text};
use serde::{Deserialize, Serialize};
use validator::ValidateEmail;

// Constants
const GIT_PROFILES_PATH: &str = "user_profiles.json";
const MAX_USERNAME_LENGTH: usize = 30;
const MAX_EMAIL_LENGTH: usize = 100;
const MAX_ALIAS_LENGTH: usize = 30;
const BACK_OPTION: &str = "back";

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
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Switch { user_alias }) => cli_switch_user(&user_alias),
        Some(Commands::Add { git_username, git_email, user_alias }) => 
            cli_add_user(&git_username, &git_email, &user_alias),
        Some(Commands::Delete { user_alias }) => cli_delete_user(&user_alias),
        Some(Commands::Current) => show_current_user(),
        Some(Commands::List) => show_all_users(),
        None => run_menu(), 
    }
}

fn cli_switch_user(user_alias: &str) {
    let users: Vec<GitUserProfile> = load_users();
    if users.is_empty() {
        println!("{}", "no users to switch to".red());
        return;
    }

    if user_alias == BACK_OPTION {
        println!("{}", "invalid alias for switching".red());
        return;
    }

    if let Some(user) = users.iter().find(|user| user.user_alias == user_alias) {
        set_git_config("user.name", &user.git_username);
        set_git_config("user.email", &user.git_email);
        println!("{} {}", "switched to user:".green(), user.user_alias);
    } else {
        println!("{}", "user alias not found".red());
    }
}

fn cli_add_user(git_username: &str, git_email: &str, user_alias: &str) {
    let mut users: Vec<GitUserProfile> = load_users();

    // Input validation
    if let Err(err) = is_valid_username(git_username, &users) {
        println!("{}", err.red());
        return;
    }

    if let Err(err) = is_valid_email(git_email, &users) {
        println!("{}", err.red());
        return;
    }

    if let Err(err) = is_valid_alias(user_alias, &users) {
        println!("{}", err.red());
        return;
    }

    users.push(GitUserProfile {
        git_username: git_username.to_string(),
        git_email: git_email.to_string(),
        user_alias: user_alias.to_string(),
    });

    save_users(&users);
    println!("{}", "user added".green());

}

fn cli_delete_user(user_alias: &str) {
    let mut users = load_users();
    if users.is_empty() {
        println!("{}", "no users to delete".red());
        return;
    }

    if user_alias == BACK_OPTION {
        println!("{}", "invalid alias for deletion".red());
        return;
    }
    
    let initial_len = users.len();
    users.retain(|user| user.user_alias != user_alias);
    if users.len() == initial_len {
        println!("{}", "user alias not found".red());
    } else {
        save_users(&users);
        println!("{}", "user deleted".green());
    }
}

// Menu functions
fn run_menu() {
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
            .prompt()
            .expect("failed to select action");

        match action_selected {
            "switch user" => menu_switch_user(),
            "add user" => menu_add_user(),
            "delete user" => menu_delete_user(),
            "show current user" => show_current_user(),
            "show all users" => show_all_users(),
            "quit" => {
                println!("{}", "quitting".yellow());
                break;
            },
            _ => unreachable!("unexpected input"),
        }
    }
} 

fn menu_switch_user() {
    let users: Vec<GitUserProfile> = load_users();
    if users.is_empty() {
        println!("{}", "no users to switch to".red());
        return;
    }

    let mut user_aliases: Vec<String> = users.iter().map(|user| user.user_alias.clone()).collect();
    user_aliases.push(BACK_OPTION.to_string());

    let alias_to_switch: String = Select::new(&format!("{}", "select user to switch:".blue()), user_aliases)
        .prompt()
        .expect("failed to select alias");

    if alias_to_switch == BACK_OPTION {
        return;
    }
    
    if let Some(user) = users.iter().find(|user| user.user_alias == alias_to_switch) {
        set_git_config("user.name", &user.git_username);
        set_git_config("user.email", &user.git_email);
        println!("{} {}", "switched to user:".green(), user.user_alias);
    }
}

fn menu_add_user() {
    let mut users: Vec<GitUserProfile> = load_users();

    // Input validation
    let username: String = loop {
        let name_input: String = Text::new(&format!("{}", "enter git username:".blue()))
            .prompt()
            .expect("failed to get username");
        match is_valid_username(&name_input, &users) {
            Ok(_) => break name_input,
            Err(err_msg) => println!("{}", err_msg.red()),
        }
    };

    let email: String = loop {
        let email_input: String = Text::new(&format!("{}", "enter git email:".blue()))
            .prompt()
            .expect("failed to get email");
        match is_valid_email(&email_input, &users) {
            Ok(_) => break email_input,
            Err(msg) => println!("{}", msg.red()),
        }
    };

    let alias: String = loop {
        let alias_input: String = Text::new(&format!("{}", "enter alias:".blue()))
            .prompt()
            .expect("failed to get alias");
        match is_valid_alias(&alias_input, &users) {
            Ok(_) => break alias_input,
            Err(msg) => println!("{}", msg.red()),
        }
    };
    
    users.push(GitUserProfile {
        git_username: username,
        git_email: email,
        user_alias: alias,
    });
    save_users(&users);
    println!("{}", "added user".green());
}

fn menu_delete_user() {
    let mut users: Vec<GitUserProfile> = load_users();
    if users.is_empty() {
        println!("{}", "no users to show".red());
        return;
    }

    let mut user_aliases: Vec<String> = users.iter().map(|user| user.user_alias.clone()).collect();
    user_aliases.push(BACK_OPTION.to_string());

    let alias_to_delete: String = Select::new(&format!("{}", "select user to delete:".blue()), user_aliases)
        .prompt()
        .expect("failed to select alias");

    if alias_to_delete == BACK_OPTION {
        return;
    }
    
    users.retain(|user| user.user_alias != alias_to_delete);
    save_users(&users);
    println!("{}", "deleted user".green());
}

// Shows current git user
fn show_current_user() {
    let current_git_username: String = get_git_user("user.name");
    let current_git_email: String = get_git_user("user.email");

    println!("{} {} <{}>", "current user:".blue(), current_git_username.trim(), current_git_email.trim());
}

// List all users in json file
fn show_all_users() {
    let users: Vec<GitUserProfile> = load_users();
    if users.is_empty() {
        println!("{}", "no users to show".red());
        return;
    }

    for user in users {
        println!("{:?}", user);
    }
}

// Storage helper functions
fn load_users() -> Vec<GitUserProfile> {
    if !Path::new(GIT_PROFILES_PATH).exists() {
        return Vec::new();
    }

    let mut file = OpenOptions::new().read(true).open(GIT_PROFILES_PATH).expect("failed to open file");

    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents).expect("failed to read file");

    if file_contents.trim().is_empty() {
        return Vec::new();
    }

    serde_json::from_str(&file_contents).expect("failed to parse JSON")
}

fn save_users(users: &[GitUserProfile]) {
    let json: String = serde_json::to_string_pretty(users).expect("failed to serialize JSON to string");
    fs::write(GIT_PROFILES_PATH, json).expect("failed to write to file");
}

// Git commands
fn get_git_user(key: &str) -> String {
    let git_command_output: Output = Command::new("git")
        .args(["config", "--get", key])
        .output()
        .expect("failed to get git user");

    String::from_utf8_lossy(&git_command_output.stdout).to_string()
}

fn set_git_config(key: &str, value: &str) {
    Command::new("git")
        .args(["config", key, value])
        .output()
        .expect("failed to set git user");
}

// Validate input helper functions
fn is_valid_username(name: &str, existing_users: &[GitUserProfile]) -> Result<(), String> {
    if name.is_empty() {
        Err("empty name inputted".to_string())
    } else if name.len() > MAX_USERNAME_LENGTH {
        Err("name too long, must be 30 characters or less".to_string())
    } else if existing_users.iter().any(|user| user.git_username == name) {
        Err("username already exists".to_string())
    } else {
        Ok(())
    }
}

fn is_valid_email(email: &str, existing_users: &[GitUserProfile]) -> Result<(), String> {
    if email.is_empty() {
        Err("empty email inputted".to_string())
    } else if email.len() > MAX_EMAIL_LENGTH {
        Err("email too long, must be 100 characters or less".to_string())
    } else if !email.validate_email() {
        Err("incorrect email format".to_string())
    } else if existing_users.iter().any(|user| user.git_email == email) {
        Err("email already exists".to_string())
    } else {
        Ok(())
    }
}

fn is_valid_alias(alias: &str, existing_users: &[GitUserProfile]) -> Result<(), String> {
    if alias.is_empty() {
        Err("empty alias inputted".to_string())
    } else if alias.len() > MAX_ALIAS_LENGTH {
        Err("alias too long, must be 30 characters or less".to_string())
    } else if alias == BACK_OPTION {
        Err("alias cannot be 'back'".to_string())
    } else if existing_users.iter().any(|user| user.user_alias == alias) {
        Err("alias already exists".to_string())
    } else {
        Ok(())
    }
}