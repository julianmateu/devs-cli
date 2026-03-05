use clap::{Parser, Subcommand};

pub mod config;
pub mod edit;
pub mod list;
pub mod new;
pub mod open;
pub mod remove;
pub mod reset;
pub mod save;

#[derive(Parser)]
#[command(name = "devs", about = "Project-aware tmux session manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Register a new project
    New {
        /// Project name (used as tmux session name)
        name: String,
        /// Absolute path to the project directory
        #[arg(long)]
        path: String,
        /// Tab color in hex format (#rrggbb or rrggbb)
        #[arg(long)]
        color: Option<String>,
    },
    /// List all registered projects
    List,
    /// Print a project's current config
    Config {
        /// Project name
        name: String,
    },
    /// Open a project's config in $EDITOR
    Edit {
        /// Project name
        name: String,
    },
    /// Remove a project from tracking
    Remove {
        /// Project name
        name: String,
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
    /// Create or attach to a tmux session for a project
    Open {
        /// Project name
        name: String,
        /// Use the declarative layout even if saved state exists
        #[arg(long)]
        default: bool,
        /// Use the saved layout (error if none)
        #[arg(long)]
        saved: bool,
    },
    /// Save the current tmux layout for a project
    Save {
        /// Project name
        name: String,
    },
    /// Reset a project's layout to its declarative default
    Reset {
        /// Project name
        name: String,
    },
}
