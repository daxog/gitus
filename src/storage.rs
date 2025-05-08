use std::{fs, io::Read, path::{Path, PathBuf}};

use crate::{error::AppError, GitUserProfile};

/// User profiles file in user's home directory
const GLOBAL_GIT_PROFILES_FILE: &str = "user_profiles.json";

/// Gets the path to the profiles file
pub fn get_global_profile_path() -> Result<String, AppError> {
    let home_dir: PathBuf = dirs::home_dir().ok_or_else(|| {
        AppError::Validation("failed to find the home directory".to_string())
    })?;
    let profile_file_path: PathBuf = home_dir.join(GLOBAL_GIT_PROFILES_FILE);
    Ok(profile_file_path.to_string_lossy().into_owned())
}

/// Loads user profiles from the JSON file
pub fn load_users() -> Result<Vec<GitUserProfile>, AppError> {
    let profile_file_path: String = get_global_profile_path()?;

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

/// Saves user profiles to the JSON file
/// 
/// # Arguments
/// * `users` - Vector of user profiles to save
pub fn save_users(users: &[GitUserProfile]) -> Result<(), AppError>  {
    let profile_file_path: String = get_global_profile_path()?;
    let json: String = serde_json::to_string_pretty(users)?;
    fs::write(profile_file_path, json)?;
    Ok(())
}

/// Checks if any users exist in storage
///
/// # Arguments
/// * `users` - Vector of user profiles to check
pub fn check_if_users_exist(users: &[GitUserProfile]) -> Result<(), AppError> {
    if users.is_empty() {
        return Err(AppError::Validation("no users found".to_string()));
    }
    Ok(())
}