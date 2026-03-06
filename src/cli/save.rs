use anyhow::{Result, bail};

use crate::domain::layout::Layout;
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

fn save_as_default(repo: &dyn ProjectRepository, tmux: &dyn TmuxAdapter, name: &str) -> Result<()> {
    let mut config = repo.load(name)?;
    let layout_string = tmux.get_layout(name)?;
    let panes = tmux.get_panes(name)?;
    config.layout = Some(Layout::from_snapshot(layout_string, &panes));
    repo.save(&config)?;
    Ok(())
}

pub fn run(
    repo: &dyn ProjectRepository,
    tmux: &dyn TmuxAdapter,
    name: &str,
    as_default: bool,
) -> Result<()> {
    if !tmux.has_session(name) {
        bail!("no active tmux session for '{name}'")
    }
    if as_default {
        save_as_default(repo, tmux, name)?;
        println!("Saved current layout as default for '{name}'.");
    } else {
        save_layout(repo, tmux, name)?;
        println!("Saved layout for '{name}'.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::toml_project_repository::TomlProjectRepository;
    use crate::domain::layout::{Layout, MainPane, SplitDirection, SplitPane};
    use crate::domain::saved_state::SavedPane;
    use crate::domain::test_helpers::MockTmuxAdapter;
    use tempfile::tempdir;

    #[test]
    fn save_captures_tmux_state() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        crate::cli::new::run(&repo, "myproject", "/some/path", None, None, None, &[]).unwrap();

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

        run(&repo, &tmux, "myproject", false).unwrap();

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
        crate::cli::new::run(&repo, "myproject", "/some/path", None, None, None, &[]).unwrap();

        let tmux = MockTmuxAdapter::no_session();

        let result = run(&repo, &tmux, "myproject", false);
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
        crate::cli::new::run(&repo, "myproject", "/some/path", None, None, None, &[]).unwrap();

        // First save
        let tmux = MockTmuxAdapter::with_session("old-layout", vec![]);
        run(&repo, &tmux, "myproject", false).unwrap();

        // Second save with different layout
        let tmux = MockTmuxAdapter::with_session(
            "new-layout",
            vec![SavedPane {
                index: 0,
                path: "/some/path".to_string(),
                command: "cargo watch".to_string(),
            }],
        );
        run(&repo, &tmux, "myproject", false).unwrap();

        let config = repo.load("myproject").unwrap();
        let state = config.last_state.expect("last_state should be set");
        assert_eq!(state.layout_string, "new-layout");
        assert_eq!(state.panes.len(), 1);
        assert_eq!(state.panes[0].command, "cargo watch");
    }

    #[test]
    fn save_as_default_writes_layout() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        crate::cli::new::run(&repo, "myproject", "/some/path", None, None, None, &[]).unwrap();

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
                SavedPane {
                    index: 2,
                    path: "/some/path".to_string(),
                    command: "cargo watch".to_string(),
                },
            ],
        );

        run(&repo, &tmux, "myproject", true).unwrap();

        let config = repo.load("myproject").unwrap();
        let layout = config.layout.expect("layout should be set");
        assert_eq!(layout.layout_string, Some("5aed,176x79,0,0".to_string()));
        assert_eq!(layout.main.cmd, Some("nvim".to_string()));
        assert_eq!(layout.panes.len(), 2);
        assert_eq!(layout.panes[0].cmd, None); // zsh is a shell
        assert_eq!(layout.panes[1].cmd, Some("cargo watch".to_string()));
    }

    #[test]
    fn save_as_default_replaces_existing_layout() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        crate::cli::new::run(&repo, "myproject", "/some/path", None, None, None, &[]).unwrap();

        // Set an existing declarative layout
        let mut config = repo.load("myproject").unwrap();
        config.layout = Some(Layout {
            main: MainPane {
                cmd: Some("old-editor".to_string()),
            },
            panes: vec![SplitPane {
                split: SplitDirection::Bottom,
                cmd: Some("old-cmd".to_string()),
                size: Some("50%".to_string()),
            }],
            layout_string: None,
        });
        repo.save(&config).unwrap();

        let tmux = MockTmuxAdapter::with_session(
            "new-layout-string",
            vec![SavedPane {
                index: 0,
                path: "/some/path".to_string(),
                command: "nvim".to_string(),
            }],
        );

        run(&repo, &tmux, "myproject", true).unwrap();

        let config = repo.load("myproject").unwrap();
        let layout = config.layout.expect("layout should be set");
        assert_eq!(layout.layout_string, Some("new-layout-string".to_string()));
        assert_eq!(layout.main.cmd, Some("nvim".to_string()));
        assert!(layout.panes.is_empty());
    }

    #[test]
    fn save_as_default_does_not_touch_last_state() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        crate::cli::new::run(&repo, "myproject", "/some/path", None, None, None, &[]).unwrap();

        let tmux = MockTmuxAdapter::with_session("layout", vec![]);

        run(&repo, &tmux, "myproject", true).unwrap();

        let config = repo.load("myproject").unwrap();
        assert!(config.last_state.is_none());
    }

    #[test]
    fn save_as_default_fails_when_no_tmux_session() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        crate::cli::new::run(&repo, "myproject", "/some/path", None, None, None, &[]).unwrap();

        let tmux = MockTmuxAdapter::no_session();

        let result = run(&repo, &tmux, "myproject", true);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("no active tmux session")
        );
    }
}
