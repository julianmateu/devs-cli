use clap::{Parser, Subcommand};

pub mod config;
pub mod edit;
pub mod list;
pub mod new;
pub mod remove;

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
}
