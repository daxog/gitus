use clap::{Parser, Subcommand};

/// CLI arguments parser using `clap`
#[derive(Parser, Debug)]
pub struct Cli {
    /// Subcommand chosen to execute
    #[command(subcommand)]
    pub command: Option<Commands>,
}

// Subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Switches current Git user to selected user
    Switch {
        /// Alias of user to switch to
        user_alias: String,
    },
    /// Adds a new user profile
    Add {
        /// Git username
        git_username: String,
        /// Git email
        git_email: String,
        /// Unique alias for the user
        user_alias: String,
    },
    /// Deletes a user profile
    Delete {
        /// Alias of user to delete
        user_alias: String,
    },
    /// Displays current Git user
    Current,
    /// Displays all users in stored JSON file
    List,
}