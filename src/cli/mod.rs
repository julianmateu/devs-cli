use clap::{Parser, Subcommand};
use clap_complete::Shell;

pub mod claude;
pub mod claude_done;
pub mod claudes;
pub mod close;
pub mod completions;
pub mod config;
pub mod edit;
pub mod format;
pub mod list;
pub mod new;
pub mod note;
pub mod notes;
pub mod open;
pub mod remove;
pub mod reset;
pub mod save;
pub mod status;

#[derive(Parser)]
#[command(name = "devs", version, about = "Project-aware tmux session manager")]
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
        /// Copy layout from an existing project
        #[arg(long)]
        from: Option<String>,
        /// Capture layout from a live tmux session
        #[arg(long, conflicts_with = "from")]
        from_session: Option<String>,
        /// Pre-populate a Claude session (format: LABEL:ID, repeatable)
        #[arg(long = "session", value_name = "LABEL:ID")]
        sessions: Vec<String>,
    },
    /// List all registered projects
    List,
    /// Show all projects with live status
    Status,
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
        /// Kill the tmux session if alive
        #[arg(long)]
        kill: bool,
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
    /// Close a project's tmux session (optionally saving layout first)
    Close {
        /// Project name
        name: String,
        /// Save the current layout before closing
        #[arg(long)]
        save: bool,
    },
    /// Save the current tmux layout for a project
    Save {
        /// Project name
        name: String,
        /// Write the captured layout as the declarative default in [layout]
        #[arg(long)]
        as_default: bool,
    },
    /// Reset a project's layout to its declarative default
    Reset {
        /// Project name
        name: String,
    },
    /// Start or resume a Claude Code session
    Claude {
        /// Project name
        name: String,
        /// Session label (required when starting a new session)
        label: Option<String>,
        /// Resume a session by label
        #[arg(long)]
        resume: Option<String>,
    },
    /// List Claude Code sessions for a project
    Claudes {
        /// Project name
        name: String,
        /// Show all sessions including done ones
        #[arg(long)]
        all: bool,
    },
    /// Mark a Claude Code session as done
    ClaudeDone {
        /// Project name
        name: String,
        /// Session label
        label: String,
    },
    /// Add a timestamped note to a project
    Note {
        /// Project name
        name: String,
        /// Note message
        message: String,
    },
    /// List notes for a project
    Notes {
        /// Project name
        name: String,
        /// Show all notes (default: last 20)
        #[arg(long)]
        all: bool,
        /// Filter notes by time (e.g., "2d", "1h", "30m")
        #[arg(long)]
        since: Option<String>,
        /// Delete all notes
        #[arg(long)]
        clear: bool,
    },
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}
