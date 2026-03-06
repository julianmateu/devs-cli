use anyhow::{Result, bail};

use crate::domain::saved_state::SavedState;
use crate::ports::project_repository::ProjectRepository;
use crate::ports::tmux_adapter::TmuxAdapter;

pub fn save_layout(repo: &dyn ProjectRepository, tmux: &dyn TmuxAdapter, name: &str) -> Result<()> {
    let mut config = repo.load(name)?;
    let layout_string = tmux.get_layout(name)?;
    let panes = tmux.get_panes(name)?;
    config.last_state = Some(SavedState {
        captured_at: chrono::Utc::now(),
        layout_string,
        panes,
    });
    repo.save(&config)?;
    Ok(())
}

pub fn run(repo: &dyn ProjectRepository, tmux: &dyn TmuxAdapter, name: &str) -> Result<()> {
    if !tmux.has_session(name) {
        bail!("no active tmux session for '{name}'")
    }
    save_layout(repo, tmux, name)?;
    println!("Saved layout for '{name}'.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::toml_project_repository::TomlProjectRepository;
    use crate::domain::saved_state::SavedPane;
    use crate::domain::test_helpers::MockTmuxAdapter;
    use tempfile::tempdir;

    #[test]
    fn save_captures_tmux_state() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        crate::cli::new::run(&repo, "myproject", "/some/path", None, None, &[]).unwrap();

        let tmux = MockTmuxAdapter::with_session(
            "5aed,176x79,0,0",
            vec![
                SavedPane {
                    index: 0,
                    path: "/some/path".to_string(),
                    command: "nvim".to_string(),
                },
                SavedPane {
                    index: 1,
                    path: "/some/path".to_string(),
                    command: "zsh".to_string(),
                },
            ],
        );

        run(&repo, &tmux, "myproject").unwrap();

        let config = repo.load("myproject").unwrap();
        let state = config.last_state.expect("last_state should be set");
        assert_eq!(state.layout_string, "5aed,176x79,0,0");
        assert_eq!(state.panes.len(), 2);
        assert_eq!(state.panes[0].command, "nvim");
        assert_eq!(state.panes[1].command, "zsh");
    }

    #[test]
    fn save_fails_when_no_tmux_session() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        crate::cli::new::run(&repo, "myproject", "/some/path", None, None, &[]).unwrap();

        let tmux = MockTmuxAdapter::no_session();

        let result = run(&repo, &tmux, "myproject");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("no active tmux session")
        );
    }

    #[test]
    fn save_overwrites_previous_saved_state() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        crate::cli::new::run(&repo, "myproject", "/some/path", None, None, &[]).unwrap();

        // First save
        let tmux = MockTmuxAdapter::with_session("old-layout", vec![]);
        run(&repo, &tmux, "myproject").unwrap();

        // Second save with different layout
        let tmux = MockTmuxAdapter::with_session(
            "new-layout",
            vec![SavedPane {
                index: 0,
                path: "/some/path".to_string(),
                command: "cargo watch".to_string(),
            }],
        );
        run(&repo, &tmux, "myproject").unwrap();

        let config = repo.load("myproject").unwrap();
        let state = config.last_state.expect("last_state should be set");
        assert_eq!(state.layout_string, "new-layout");
        assert_eq!(state.panes.len(), 1);
        assert_eq!(state.panes[0].command, "cargo watch");
    }
}
