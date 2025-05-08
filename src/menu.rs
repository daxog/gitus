use colored::Colorize;
use inquire::Select;

use crate::{add_user, check_if_users_exist, delete_user, error::AppError, list_all_users, show_current_user, storage::load_users, switch_user, validation::{prompt_until_valid, validate_input_alias, validate_input_email, validate_input_username}, GitUserProfile, BACK_OPTION};

/// Runs interactive menu interface
pub fn run_menu() -> Result<(), AppError> {
    loop {
        let actions: Vec<&'static str> = vec![
            "switch user", 
            "add user", 
            "delete user", 
            "show current user",
            "show all users", 
            "quit"
        ];

        let action_selected: &'static str = Select::new(&format!("{}", "select action".blue()), actions)
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

/// Menu for switching users
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

/// Menu for adding a new user
fn menu_add_user() -> Result<(), AppError> {
    let users: Vec<GitUserProfile> = load_users()?;

    // Input validation
    let username: String = prompt_until_valid(
        &format!("{}", "enter git username:".blue()),
        |input| validate_input_username(input, &users),
    )?;

    let email: String = prompt_until_valid(
        &format!("{}", "enter git email:".blue()), 
        |input| validate_input_email(input, &users)
    )?;

    let alias: String = prompt_until_valid(
        &format!("{}", "enter alias:".blue()), 
        |input| validate_input_alias(input, &users)
    )?;
    
    add_user(&username, &email, &alias)?;
    Ok(())
}


/// Menu for deleting a user
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

/// Builds list of user aliases for menu to display
pub fn build_alias_list(users: &[GitUserProfile]) -> Vec<String> {
    let mut user_aliases: Vec<String> = users.iter()
        .map(|user| user.user_alias.clone())
        .collect();
    user_aliases.push("back".to_string());
    user_aliases
}