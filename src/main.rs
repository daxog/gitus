use std::{fs::{self, OpenOptions}, io::Read, path::Path, process::{Command, Output}};

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

#[derive(Serialize, Deserialize, Debug)]
struct GitUserProfile {
    git_username: String,
    git_email: String,
    user_alias: String,
}

fn main() {
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
            "switch user" => switch_user(),
            "add user" => add_user(),
            "delete user" => delete_user(),
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

fn switch_user() {
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

fn add_user() {
    let mut users: Vec<GitUserProfile> = load_users();

    // Input validation
    let username: String = loop {
        let name_input: String = Text::new(&format!("{}", "enter git username:".blue()))
            .prompt()
            .expect("failed to get username");
        if name_input.is_empty() {
            println!("{}", "empty name inputted".red());
        } else if name_input.len() > MAX_USERNAME_LENGTH {
            println!("{}", "name too long, must be 30 characters or less".red())
        } else if users.iter().any(|user| user.git_username == name_input) {
            println!("{}", "username already exists".red());
        }
        break name_input;
    };

    let email: String = loop {
        let email_input: String = Text::new(&format!("{}", "enter git email:".blue()))
           .prompt()
           .expect("failed to get email");
        if email_input.is_empty() {
            println!("{}", "empty email inputted".red());
        } else if email_input.len() > MAX_EMAIL_LENGTH {
            println!("{}", "email too long, must be 100 characters or less".red());
        } else if !email_input.validate_email() {
            println!("{}", "incorrect email format".red());
        } else if users.iter().any(|user| user.git_email == email_input) {
            println!("{}", "email already exists".red());
        }
        break email_input;
    };

    let alias: String = loop {
        let alias_input: String = Text::new(&format!("{}", "enter alias:".blue()))
          .prompt()
          .expect("failed to get alias");
        if alias_input.is_empty() {
            println!("{}", "empty alias inputted".red());
        } else if alias_input.len() > MAX_ALIAS_LENGTH {
            println!("{}", "alias too long, must be 30 characters or less".red());
        } else if alias_input == BACK_OPTION {
            println!("{}", "alias cannot be 'back'".red());
        } else if users.iter().any(|user| user.user_alias == alias_input) {
            println!("{}", "alias already exists".red());
        } 
        break alias_input;
    };
    
    users.push(GitUserProfile {
        git_username: username,
        git_email: email,
        user_alias: alias,
    });
    save_users(&users);
    println!("{}", "added user".green());
}

fn delete_user() {
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
