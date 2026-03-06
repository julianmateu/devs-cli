use anyhow::{Result, bail};

use crate::ports::project_repository::ProjectRepository;
use crate::ports::terminal_adapter::TerminalAdapter;
use crate::ports::tmux_adapter::TmuxAdapter;

pub fn run(
    repo: &dyn ProjectRepository,
    tmux: &dyn TmuxAdapter,
    terminal: &dyn TerminalAdapter,
    name: &str,
    save: bool,
) -> Result<()> {
    repo.load(name)?;

    if !tmux.has_session(name) {
        bail!("no active tmux session for '{name}'");
    }

    if save {
        super::save::save_layout(repo, tmux, name)?;
        println!("Saved layout for '{name}'.");
    }

    tmux.kill_session(name)?;
    terminal.reset_tab_color()?;
    terminal.reset_tab_title()?;
    println!("Closed session '{name}'.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::toml_project_repository::TomlProjectRepository;
    use crate::domain::saved_state::SavedPane;
    use crate::domain::test_helpers::{MockTerminalAdapter, MockTmuxAdapter};
    use tempfile::tempdir;

    fn setup() -> (TomlProjectRepository, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams::new("proj", "/some/path"),
        )
        .unwrap();
        (repo, dir)
    }

    #[test]
    fn close_kills_session_and_resets_color() {
        let (repo, _dir) = setup();
        let tmux = MockTmuxAdapter::with_session("layout", vec![]);
        let terminal = MockTerminalAdapter::new();

        run(&repo, &tmux, &terminal, "proj", false).unwrap();

        assert!(tmux.calls().contains(&"kill_session(proj)".to_string()));
        assert!(terminal.calls().contains(&"reset_tab_color()".to_string()));
        assert!(terminal.calls().contains(&"reset_tab_title()".to_string()));
    }

    #[test]
    fn close_with_save_saves_then_kills() {
        let (repo, _dir) = setup();
        let tmux = MockTmuxAdapter::with_session(
            "saved-layout",
            vec![SavedPane {
                index: 0,
                path: "/some/path".to_string(),
                command: "zsh".to_string(),
            }],
        );
        let terminal = MockTerminalAdapter::new();

        run(&repo, &tmux, &terminal, "proj", true).unwrap();

        let config = repo.load("proj").unwrap();
        let state = config.last_state.expect("last_state should be set");
        assert_eq!(state.layout_string, "saved-layout");
        assert_eq!(state.panes.len(), 1);

        assert!(tmux.calls().contains(&"kill_session(proj)".to_string()));
        assert!(terminal.calls().contains(&"reset_tab_color()".to_string()));
        assert!(terminal.calls().contains(&"reset_tab_title()".to_string()));
    }

    #[test]
    fn close_dead_session_errors() {
        let (repo, _dir) = setup();
        let tmux = MockTmuxAdapter::no_session();
        let terminal = MockTerminalAdapter::new();

        let result = run(&repo, &tmux, &terminal, "proj", false);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("no active tmux session")
        );
    }

    #[test]
    fn close_save_failure_does_not_kill_session() {
        let (repo, _dir) = setup();
        let mut tmux = MockTmuxAdapter::with_session("layout", vec![]);
        tmux.fail_on_get_layout = true;
        let terminal = MockTerminalAdapter::new();

        let result = run(&repo, &tmux, &terminal, "proj", true);
        assert!(result.is_err());
        assert!(!tmux.calls().iter().any(|c| c.starts_with("kill_session")));
        assert!(terminal.calls().is_empty());
    }

    #[test]
    fn close_missing_project_errors() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        let tmux = MockTmuxAdapter::with_session("layout", vec![]);
        let terminal = MockTerminalAdapter::new();

        let result = run(&repo, &tmux, &terminal, "ghost", false);
        assert!(result.is_err());
    }
}
