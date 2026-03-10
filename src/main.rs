mod adapters;
mod cli;
mod domain;
mod ports;

#[cfg(test)]
pub(crate) mod test_support;

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::{CommandFactory, Parser};
use clap_complete::CompletionCandidate;
use clap_complete::engine::ArgValueCandidates;
use clap_complete::env::CompleteEnv;

use adapters::iterm_terminal_adapter::ItermTerminalAdapter;
use adapters::migration;
use adapters::noop_terminal_adapter::NoopTerminalAdapter;
use adapters::os_process_launcher::OsProcessLauncher;
use adapters::shell_tmux_adapter::ShellTmuxAdapter;
use adapters::toml_local_config::TomlLocalConfig;
use adapters::toml_project_repository::TomlProjectRepository;
use cli::{Cli, Commands};
use domain::layout::Layout;
use ports::local_config::LocalConfigReader;
use ports::project_repository::ProjectRepository;
use ports::terminal_adapter::TerminalAdapter;
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

fn complete_command() -> clap::Command {
    let mut cmd = Cli::command();

    // "new" takes a name for a project that doesn't exist yet — skip it.
    let subcmd_names: Vec<String> = cmd
        .get_subcommands()
        .filter(|s| s.get_name() != "new")
        .filter(|s| s.get_arguments().any(|a| a.get_id() == "name"))
        .map(|s| s.get_name().to_string())
        .collect();

    let repo = TomlProjectRepository::new(config_dir());
    let names: Vec<String> = repo.list().unwrap_or_default();

    for subcmd_name in subcmd_names {
        let names = names.clone();
        cmd = cmd.mut_subcommand(subcmd_name, |subcmd| {
            let names = names.clone();
            subcmd.mut_arg("name", |arg| {
                arg.add(ArgValueCandidates::new(move || {
                    names
                        .iter()
                        .map(|n| CompletionCandidate::new(n.as_str()))
                        .collect()
                }))
            })
        });
    }

    cmd
}

fn main() -> Result<()> {
    CompleteEnv::with_factory(complete_command).complete();

    let cli = Cli::parse();
    let config_dir = config_dir();
    migration::migrate_if_needed(&config_dir)?;
    let repo = TomlProjectRepository::new(config_dir.clone());
    let tmux = ShellTmuxAdapter;
    let terminal: Box<dyn TerminalAdapter> =
        if std::env::var("TERM_PROGRAM").as_deref() == Ok("iTerm.app") {
            Box::new(ItermTerminalAdapter::new())
        } else {
            Box::new(NoopTerminalAdapter)
        };
    let launcher = OsProcessLauncher;
    let local_config_adapter = TomlLocalConfig;

    let cwd = std::env::current_dir().context("failed to determine current directory")?;
    let home_dir = dirs::home_dir().map(|p| p.to_string_lossy().into_owned());
    let resolve = |name: Option<String>| -> Result<String> {
        cli::resolve::resolve_project_name(name.as_deref(), &cwd, home_dir.as_deref(), &repo)
    };

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
        Commands::Init { name } => {
            let name = resolve(name)?;
            cli::init::run(&repo, &local_config_adapter, &name)?
        }
        Commands::List => cli::list::run(&repo, &mut std::io::stdout())?,
        Commands::Status => cli::status::run(&repo, &tmux, &mut std::io::stdout())?,
        Commands::Config { name } => {
            let name = resolve(name)?;
            cli::config::run(&repo, &name)?
        }
        Commands::Edit { name, local } => {
            let name = resolve(name)?;
            cli::edit::run(&repo, &name, &config_dir, local)?
        }
        Commands::Remove { name, force, kill } => {
            let name = resolve(name)?;
            cli::remove::run(&repo, &tmux, &name, force, kill)?
        }
        Commands::Close { name, save } => {
            let name = resolve(name)?;
            cli::close::run(&repo, &tmux, terminal.as_ref(), &name, save)?
        }
        Commands::Open {
            name,
            default,
            saved,
        } => {
            let name = resolve(name)?;
            cli::open::run(&repo, &tmux, terminal.as_ref(), &name, default, saved)?
        }
        Commands::Save { name, as_default } => {
            let name = resolve(name)?;
            cli::save::run(&repo, &tmux, &name, as_default)?
        }
        Commands::Reset { name } => {
            let name = resolve(name)?;
            cli::reset::run(&repo, &name)?
        }
        Commands::Claude {
            name,
            label,
            resume,
        } => {
            // When one positional is given, Clap assigns it to `name`.
            // If label is None and resume is None, the single arg is the label.
            let (name, label) = match (name, label) {
                (Some(n), Some(l)) => (Some(n), Some(l)),
                (Some(v), None) if resume.is_none() => (None, Some(v)),
                (n, l) => (n, l),
            };
            let name = resolve(name)?;
            match resume {
                Some(label) => cli::claude::resume(&repo, &launcher, &name, &label)?,
                None => {
                    let label = label.ok_or_else(|| {
                        anyhow::anyhow!("label is required when starting a new session")
                    })?;
                    cli::claude::start(&repo, &launcher, &name, &label)?
                }
            }
        }
        Commands::Claudes { name, all } => {
            let name = resolve(name)?;
            cli::claudes::run(&repo, &name, all, &mut std::io::stdout())?
        }
        Commands::ClaudeDone { name, label } => {
            // When one positional is given, Clap assigns it to `name`.
            // If label is None, the single arg is actually the label.
            let (name, label) = match (name, label) {
                (Some(n), Some(l)) => (Some(n), l),
                (Some(v), None) => (None, v),
                (None, _) => bail!("session label is required"),
            };
            let name = resolve(name)?;
            cli::claude_done::run(&repo, &name, &label)?
        }
        Commands::Note { name, message } => {
            // When one positional is given, Clap assigns it to `name`.
            // If message is None, the single arg is actually the message.
            let (name, message) = match (name, message) {
                (Some(n), Some(m)) => (Some(n), m),
                (Some(v), None) => (None, v),
                (None, _) => bail!("note message is required"),
            };
            let name = resolve(name)?;
            cli::note::run(&repo, &name, &message)?
        }
        Commands::Notes {
            name,
            all,
            since,
            clear,
            force,
        } => {
            let name = resolve(name)?;
            cli::notes::run(
                &repo,
                &name,
                all,
                since.as_deref(),
                clear,
                force,
                &mut std::io::stdout(),
            )?
        }
        Commands::Completions { shell } => cli::completions::run(shell),
        Commands::TmuxHelp => cli::tmux_help::run(),
        Commands::GenerateMan { output_dir } => cli::man::run(&output_dir)?,
    }

    Ok(())
}
