use serde::{Deserialize, Serialize};

/// Represents a Git user profile stored in the profiles file
#[derive(Serialize, Deserialize, Debug)]
pub struct GitUserProfile {
    /// Git username (user.name)
    pub git_username: String,
    /// Git email address (user.email)
    pub git_email: String,
    /// Unique user alias
    pub user_alias: String,
}