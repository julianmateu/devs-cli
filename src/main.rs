mod adapters;
mod cli;
mod domain;
mod ports;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use adapters::iterm_terminal_adapter::ItermTerminalAdapter;
use adapters::shell_tmux_adapter::ShellTmuxAdapter;
use adapters::toml_project_repository::TomlProjectRepository;
use cli::{Cli, Commands};

fn config_dir() -> PathBuf {
    dirs::home_dir()
        .expect("could not determine home directory")
        .join(".config/devs")
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_dir = config_dir();
    let repo = TomlProjectRepository::new(config_dir.clone());
    let tmux = ShellTmuxAdapter;
    let terminal = ItermTerminalAdapter::new();

    match cli.command {
        Commands::New { name, path, color } => {
            cli::new::run(&repo, &name, &path, color.as_deref())?
        }
        Commands::List => cli::list::run(&repo)?,
        Commands::Config { name } => cli::config::run(&repo, &name)?,
        Commands::Edit { name } => cli::edit::run(&name, &config_dir)?,
        Commands::Remove { name, force } => cli::remove::run(&repo, &name, force)?,
        Commands::Open {
            name,
            default,
            saved,
        } => cli::open::run(&repo, &tmux, &terminal, &name, default, saved)?,
        Commands::Save { name } => cli::save::run(&repo, &tmux, &name)?,
        Commands::Reset { name } => cli::reset::run(&repo, &name)?,
        Commands::Claude {
            name,
            label,
            resume,
        } => match resume {
            Some(id) => cli::claude::resume(&repo, &name, &id)?,
            None => {
                let label = label.ok_or_else(|| {
                    anyhow::anyhow!("label is required when starting a new session")
                })?;
                cli::claude::start(&repo, &name, &label)?
            }
        },
    }

    Ok(())
}
