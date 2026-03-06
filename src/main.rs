mod adapters;
mod cli;
mod domain;
mod ports;

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::Parser;

use adapters::iterm_terminal_adapter::ItermTerminalAdapter;
use adapters::migration;
use adapters::os_process_launcher::OsProcessLauncher;
use adapters::shell_tmux_adapter::ShellTmuxAdapter;
use adapters::toml_local_config::TomlLocalConfig;
use adapters::toml_project_repository::TomlProjectRepository;
use cli::{Cli, Commands};
use domain::layout::Layout;
use ports::local_config::LocalConfigReader;
use ports::tmux_adapter::TmuxAdapter;

fn config_dir() -> PathBuf {
    dirs::home_dir()
        .expect("could not determine home directory")
        .join(".config/devs")
}

fn resolve_path(path: Option<&str>) -> Result<String> {
    match path {
        Some(p) => Ok(cli::format::expand_home(p)),
        None => {
            let cwd = std::env::current_dir().context("failed to determine current directory")?;
            Ok(cwd.to_string_lossy().to_string())
        }
    }
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
    migration::migrate_if_needed(&config_dir)?;
    let repo = TomlProjectRepository::new(config_dir.clone());
    let tmux = ShellTmuxAdapter;
    let terminal = ItermTerminalAdapter::new();
    let launcher = OsProcessLauncher;
    let local_config_adapter = TomlLocalConfig;

    match cli.command {
        Commands::New {
            name,
            path,
            color,
            from,
            from_session,
            sessions,
        } => {
            let resolved_path = resolve_path(path.as_deref())?;
            let local_config = local_config_adapter.read(&resolved_path)?;
            let captured_layout = capture_layout_from_session(&tmux, from_session.as_deref())?;
            let storage_path = cli::format::abbreviate_home(&resolved_path);
            cli::new::run(
                &repo,
                cli::new::NewProjectParams {
                    color: color.as_deref(),
                    from: from.as_deref(),
                    from_layout: captured_layout,
                    sessions: &sessions,
                    local_config,
                    ..cli::new::NewProjectParams::new(&name, &storage_path)
                },
            )?
        }
        Commands::Init { name } => cli::init::run(&repo, &local_config_adapter, &name)?,
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
        Commands::TmuxHelp => cli::tmux_help::run(),
        Commands::GenerateMan { output_dir } => cli::man::run(&output_dir)?,
    }

    Ok(())
}
