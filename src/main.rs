mod adapters;
mod cli;
mod domain;
mod ports;

use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::Parser;

use adapters::iterm_terminal_adapter::ItermTerminalAdapter;
use adapters::os_process_launcher::OsProcessLauncher;
use adapters::shell_tmux_adapter::ShellTmuxAdapter;
use adapters::toml_project_repository::TomlProjectRepository;
use cli::{Cli, Commands};
use domain::layout::Layout;
use ports::tmux_adapter::TmuxAdapter;

fn config_dir() -> PathBuf {
    dirs::home_dir()
        .expect("could not determine home directory")
        .join(".config/devs")
}

fn capture_layout_from_session(
    tmux: &dyn TmuxAdapter,
    from_session: Option<&str>,
) -> Result<Option<Layout>> {
    let session = match from_session {
        Some(s) => s,
        None => return Ok(None),
    };
    if !tmux.has_session(session) {
        bail!("no active tmux session for '{session}'");
    }
    let layout_string = tmux.get_layout(session)?;
    let panes = tmux.get_panes(session)?;
    Ok(Some(Layout::from_snapshot(layout_string, &panes)))
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_dir = config_dir();
    let repo = TomlProjectRepository::new(config_dir.clone());
    let tmux = ShellTmuxAdapter;
    let terminal = ItermTerminalAdapter::new();
    let launcher = OsProcessLauncher;

    match cli.command {
        Commands::New {
            name,
            path,
            color,
            from,
            from_session,
            sessions,
        } => {
            let captured_layout = capture_layout_from_session(&tmux, from_session.as_deref())?;
            cli::new::run(
                &repo,
                &name,
                &path,
                color.as_deref(),
                from.as_deref(),
                captured_layout,
                &sessions,
            )?
        }
        Commands::List => cli::list::run(&repo)?,
        Commands::Status => cli::status::run(&repo, &tmux)?,
        Commands::Config { name } => cli::config::run(&repo, &name)?,
        Commands::Edit { name } => cli::edit::run(&name, &config_dir)?,
        Commands::Remove { name, force, kill } => {
            cli::remove::run(&repo, &tmux, &name, force, kill)?
        }
        Commands::Close { name, save } => cli::close::run(&repo, &tmux, &terminal, &name, save)?,
        Commands::Open {
            name,
            default,
            saved,
        } => cli::open::run(&repo, &tmux, &terminal, &name, default, saved)?,
        Commands::Save { name, as_default } => cli::save::run(&repo, &tmux, &name, as_default)?,
        Commands::Reset { name } => cli::reset::run(&repo, &name)?,
        Commands::Claude {
            name,
            label,
            resume,
        } => match resume {
            Some(label) => cli::claude::resume(&repo, &launcher, &name, &label)?,
            None => {
                let label = label.ok_or_else(|| {
                    anyhow::anyhow!("label is required when starting a new session")
                })?;
                cli::claude::start(&repo, &launcher, &name, &label)?
            }
        },
        Commands::Claudes { name, all } => cli::claudes::run(&repo, &name, all)?,
        Commands::ClaudeDone { name, label } => cli::claude_done::run(&repo, &name, &label)?,
        Commands::Note { name, message } => cli::note::run(&repo, &name, &message)?,
        Commands::Notes {
            name,
            all,
            since,
            clear,
        } => cli::notes::run(&repo, &name, all, since.as_deref(), clear)?,
        Commands::Completions { shell } => cli::completions::run(shell),
        Commands::GenerateMan { output_dir } => cli::man::run(&output_dir)?,
    }

    Ok(())
}
