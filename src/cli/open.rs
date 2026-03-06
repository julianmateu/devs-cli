use anyhow::{Result, bail};

use crate::domain::claude_session::{ClaudeSession, ClaudeSessionStatus};
use crate::domain::layout::{Layout, SHELL_COMMANDS, SplitDirection};
use crate::domain::saved_state::SavedState;
use crate::ports::project_repository::ProjectRepository;
use crate::ports::terminal_adapter::TerminalAdapter;
use crate::ports::tmux_adapter::TmuxAdapter;

pub fn run(
    repo: &dyn ProjectRepository,
    tmux: &dyn TmuxAdapter,
    terminal: &dyn TerminalAdapter,
    name: &str,
    use_default: bool,
    use_saved: bool,
) -> Result<()> {
    if use_default && use_saved {
        bail!("cannot use both --default and --saved");
    }

    let config = repo.load(name)?;

    if tmux.has_session(name) {
        terminal.set_tab_title(name)?;
        if let Some(color) = &config.project.color {
            terminal.set_tab_color(color)?;
        }
        tmux.attach(name)?;
        return Ok(());
    }

    let use_saved_state = if use_default {
        false
    } else if use_saved {
        if config.last_state.is_none() {
            bail!("no saved state for '{name}'");
        }
        true
    } else {
        config.last_state.is_some()
    };

    if use_saved_state {
        create_from_saved_state(
            tmux,
            name,
            &config.project.path,
            config.last_state.as_ref().unwrap(),
        )?;
    } else {
        create_from_declarative_layout(
            tmux,
            name,
            &config.project.path,
            config.layout.as_ref(),
            &config.claude_sessions,
        )?;
    }

    terminal.set_tab_title(name)?;
    if let Some(color) = &config.project.color {
        terminal.set_tab_color(color)?;
    }

    for session in &config.claude_sessions {
        if matches!(session.status, ClaudeSessionStatus::Active) {
            println!("  devs claude {name} --resume {}", session.label);
        }
    }

    tmux.attach(name)?;
    Ok(())
}

fn expand_cmd(cmd: &str, name: &str, sessions: &[ClaudeSession]) -> String {
    let label = if cmd == "claude" {
        "default"
    } else if let Some(label) = cmd.strip_prefix("claude:") {
        label
    } else {
        return cmd.to_string();
    };

    let has_active_session = sessions
        .iter()
        .any(|s| s.label == label && matches!(s.status, ClaudeSessionStatus::Active));

    if has_active_session {
        format!("devs claude {name} --resume {label}")
    } else {
        format!("devs claude {name} {label}")
    }
}

fn create_from_declarative_layout(
    tmux: &dyn TmuxAdapter,
    name: &str,
    path: &str,
    layout: Option<&Layout>,
    sessions: &[ClaudeSession],
) -> Result<()> {
    tmux.create_session(name, path)?;

    let layout = match layout {
        Some(l) => l,
        None => return Ok(()),
    };

    if let Some(cmd) = &layout.main.cmd {
        let expanded = expand_cmd(cmd, name, sessions);
        tmux.send_keys(&format!("{name}:0.0"), &expanded)?;
    }

    let mut right_pane: Option<u32> = None;
    let mut pane_count: u32 = 1;

    for pane in &layout.panes {
        let target = format!("{name}:0");
        match pane.split {
            SplitDirection::Right => {
                tmux.split_window(&target, true, pane.size.as_deref(), Some(path))?;
                right_pane = Some(pane_count);
                pane_count += 1;
            }
            SplitDirection::Bottom => {
                tmux.split_window(&target, false, pane.size.as_deref(), Some(path))?;
                pane_count += 1;
            }
            SplitDirection::BottomRight => {
                if let Some(right) = right_pane {
                    tmux.select_pane(&format!("{name}:0.{right}"))?;
                }
                tmux.split_window(&target, false, pane.size.as_deref(), Some(path))?;
                pane_count += 1;
            }
        }
        if let Some(cmd) = &pane.cmd {
            let expanded = expand_cmd(cmd, name, sessions);
            tmux.send_keys(&format!("{name}:0"), &expanded)?;
        }
    }

    if !layout.panes.is_empty() {
        tmux.select_pane(&format!("{name}:0.0"))?;
    }

    if let Some(ls) = &layout.layout_string {
        tmux.apply_layout(name, ls)?;
    }

    Ok(())
}

fn create_from_saved_state(
    tmux: &dyn TmuxAdapter,
    name: &str,
    path: &str,
    state: &SavedState,
) -> Result<()> {
    tmux.create_session(name, path)?;

    for pane in state.panes.iter().skip(1) {
        tmux.split_window(&format!("{name}:0"), true, None, Some(&pane.path))?;
    }

    tmux.apply_layout(name, &state.layout_string)?;

    for pane in &state.panes {
        if !SHELL_COMMANDS.contains(&pane.command.as_str()) {
            tmux.send_keys(&format!("{name}:0.{}", pane.index), &pane.command)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::toml_project_repository::TomlProjectRepository;
    use crate::domain::claude_session::{ClaudeSession, ClaudeSessionStatus};
    use crate::domain::layout::{Layout, MainPane, SplitDirection, SplitPane};
    use crate::domain::saved_state::{SavedPane, SavedState};
    use crate::domain::test_helpers::{MockTerminalAdapter, MockTmuxAdapter, dt};
    use tempfile::tempdir;

    fn setup_repo() -> (tempfile::TempDir, TomlProjectRepository) {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        (dir, repo)
    }

    #[test]
    fn attaches_to_existing_session_with_color() {
        let (_dir, repo) = setup_repo();
        crate::cli::new::run(
            &repo,
            "myproj",
            "/some/path",
            Some("#e06c75"),
            None,
            None,
            &[],
        )
        .unwrap();

        let tmux = MockTmuxAdapter::with_session("", vec![]);
        let terminal = MockTerminalAdapter::new();

        run(&repo, &tmux, &terminal, "myproj", false, false).unwrap();

        assert_eq!(
            terminal.calls(),
            vec!["set_tab_title(myproj)", "set_tab_color(#e06c75)"]
        );
        assert_eq!(tmux.calls(), vec!["attach(myproj)"]);
    }

    #[test]
    fn errors_with_both_flags() {
        let (_dir, repo) = setup_repo();
        crate::cli::new::run(&repo, "myproj", "/some/path", None, None, None, &[]).unwrap();

        let tmux = MockTmuxAdapter::no_session();
        let terminal = MockTerminalAdapter::new();

        let result = run(&repo, &tmux, &terminal, "myproj", true, true);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("cannot use both --default and --saved")
        );
    }

    #[test]
    fn saved_flag_errors_without_saved_state() {
        let (_dir, repo) = setup_repo();
        crate::cli::new::run(&repo, "myproj", "/some/path", None, None, None, &[]).unwrap();

        let tmux = MockTmuxAdapter::no_session();
        let terminal = MockTerminalAdapter::new();

        let result = run(&repo, &tmux, &terminal, "myproj", false, true);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no saved state"));
    }

    #[test]
    fn creates_minimal_session_without_layout() {
        let (_dir, repo) = setup_repo();
        crate::cli::new::run(&repo, "myproj", "/some/path", None, None, None, &[]).unwrap();

        let tmux = MockTmuxAdapter::no_session();
        let terminal = MockTerminalAdapter::new();

        run(&repo, &tmux, &terminal, "myproj", false, false).unwrap();

        assert_eq!(
            tmux.calls(),
            vec!["create_session(myproj, /some/path)", "attach(myproj)"]
        );
        assert_eq!(terminal.calls(), vec!["set_tab_title(myproj)"]);
    }

    #[test]
    fn creates_declarative_layout_with_splits() {
        let (_dir, repo) = setup_repo();
        crate::cli::new::run(&repo, "myproj", "/some/path", None, None, None, &[]).unwrap();

        // Add a layout with main cmd + right split + bottom-right split
        let mut config = repo.load("myproj").unwrap();
        config.layout = Some(Layout {
            main: MainPane {
                cmd: Some("nvim".to_string()),
            },
            panes: vec![
                SplitPane {
                    split: SplitDirection::Right,
                    cmd: Some("claude".to_string()),
                    size: Some("40%".to_string()),
                },
                SplitPane {
                    split: SplitDirection::BottomRight,
                    cmd: None,
                    size: None,
                },
            ],
            layout_string: None,
        });
        repo.save(&config).unwrap();

        let tmux = MockTmuxAdapter::no_session();
        let terminal = MockTerminalAdapter::new();

        run(&repo, &tmux, &terminal, "myproj", false, false).unwrap();

        assert_eq!(
            tmux.calls(),
            vec![
                "create_session(myproj, /some/path)",
                "send_keys(myproj:0.0, nvim)",
                "split_window(myproj:0, horizontal, 40%, /some/path)",
                "send_keys(myproj:0, devs claude myproj default)",
                "select_pane(myproj:0.1)",
                "split_window(myproj:0, vertical, -, /some/path)",
                "select_pane(myproj:0.0)",
                "attach(myproj)",
            ]
        );
    }

    #[test]
    fn default_flag_uses_declarative_over_saved() {
        let (_dir, repo) = setup_repo();
        crate::cli::new::run(&repo, "myproj", "/some/path", None, None, None, &[]).unwrap();

        // Add both a layout and saved state
        let mut config = repo.load("myproj").unwrap();
        config.layout = Some(Layout {
            main: MainPane { cmd: None },
            panes: vec![],
            layout_string: None,
        });
        config.last_state = Some(SavedState {
            captured_at: dt("2026-03-03T16:00:00Z"),
            layout_string: "saved-layout".to_string(),
            panes: vec![SavedPane {
                index: 0,
                path: "/some/path".to_string(),
                command: "nvim".to_string(),
            }],
        });
        repo.save(&config).unwrap();

        let tmux = MockTmuxAdapter::no_session();
        let terminal = MockTerminalAdapter::new();

        run(&repo, &tmux, &terminal, "myproj", true, false).unwrap();

        // Should NOT contain apply_layout (saved state path)
        let calls = tmux.calls();
        assert!(calls.contains(&"create_session(myproj, /some/path)".to_string()));
        assert!(!calls.iter().any(|c| c.starts_with("apply_layout")));
    }

    #[test]
    fn prefers_saved_state_when_no_flags() {
        let (_dir, repo) = setup_repo();
        crate::cli::new::run(&repo, "myproj", "/some/path", None, None, None, &[]).unwrap();

        let mut config = repo.load("myproj").unwrap();
        config.layout = Some(Layout {
            main: MainPane {
                cmd: Some("nvim".to_string()),
            },
            panes: vec![],
            layout_string: None,
        });
        config.last_state = Some(SavedState {
            captured_at: dt("2026-03-03T16:00:00Z"),
            layout_string: "saved-layout".to_string(),
            panes: vec![SavedPane {
                index: 0,
                path: "/some/path".to_string(),
                command: "nvim".to_string(),
            }],
        });
        repo.save(&config).unwrap();

        let tmux = MockTmuxAdapter::no_session();
        let terminal = MockTerminalAdapter::new();

        run(&repo, &tmux, &terminal, "myproj", false, false).unwrap();

        // Should use saved state path (apply_layout), not declarative
        let calls = tmux.calls();
        assert!(calls.contains(&"apply_layout(myproj, saved-layout)".to_string()));
    }

    #[test]
    fn creates_from_saved_state_skipping_shells() {
        let (_dir, repo) = setup_repo();
        crate::cli::new::run(&repo, "myproj", "/some/path", None, None, None, &[]).unwrap();

        let mut config = repo.load("myproj").unwrap();
        config.last_state = Some(SavedState {
            captured_at: dt("2026-03-03T16:00:00Z"),
            layout_string: "5aed,176x79,0,0".to_string(),
            panes: vec![
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
        });
        repo.save(&config).unwrap();

        let tmux = MockTmuxAdapter::no_session();
        let terminal = MockTerminalAdapter::new();

        run(&repo, &tmux, &terminal, "myproj", false, true).unwrap();

        assert_eq!(
            tmux.calls(),
            vec![
                "create_session(myproj, /some/path)",
                "split_window(myproj:0, horizontal, -, /some/path)", // pane 1
                "split_window(myproj:0, horizontal, -, /some/path)", // pane 2
                "apply_layout(myproj, 5aed,176x79,0,0)",
                "send_keys(myproj:0.0, nvim)", // nvim is not a shell
                "send_keys(myproj:0.2, cargo watch)", // cargo watch is not a shell
                // zsh (pane 1) is skipped
                "attach(myproj)",
            ]
        );
    }

    #[test]
    fn sets_tab_color_on_new_session() {
        let (_dir, repo) = setup_repo();
        crate::cli::new::run(
            &repo,
            "myproj",
            "/some/path",
            Some("#e06c75"),
            None,
            None,
            &[],
        )
        .unwrap();

        let tmux = MockTmuxAdapter::no_session();
        let terminal = MockTerminalAdapter::new();

        run(&repo, &tmux, &terminal, "myproj", false, false).unwrap();

        assert_eq!(
            terminal.calls(),
            vec!["set_tab_title(myproj)", "set_tab_color(#e06c75)"]
        );
    }

    #[test]
    fn expand_cmd_with_existing_session_returns_resume() {
        let sessions = vec![ClaudeSession {
            id: "sess_1".to_string(),
            label: "main-session".to_string(),
            started_at: dt("2026-03-01T10:00:00Z"),
            status: ClaudeSessionStatus::Active,
        }];

        assert_eq!(
            expand_cmd("claude:main-session", "myproj", &sessions),
            "devs claude myproj --resume main-session"
        );
    }

    #[test]
    fn expand_cmd_without_existing_session_returns_start() {
        let sessions: Vec<ClaudeSession> = vec![];

        assert_eq!(
            expand_cmd("claude:new-label", "myproj", &sessions),
            "devs claude myproj new-label"
        );
    }

    #[test]
    fn expand_cmd_done_session_returns_start() {
        let sessions = vec![ClaudeSession {
            id: "sess_1".to_string(),
            label: "old".to_string(),
            started_at: dt("2026-03-01T10:00:00Z"),
            status: ClaudeSessionStatus::Done(dt("2026-03-02T10:00:00Z")),
        }];

        assert_eq!(
            expand_cmd("claude:old", "myproj", &sessions),
            "devs claude myproj old"
        );
    }

    #[test]
    fn expand_cmd_bare_claude_uses_default_label() {
        let sessions: Vec<ClaudeSession> = vec![];

        assert_eq!(
            expand_cmd("claude", "myproj", &sessions),
            "devs claude myproj default"
        );
    }

    #[test]
    fn expand_cmd_non_claude_passes_through() {
        let sessions: Vec<ClaudeSession> = vec![];

        assert_eq!(expand_cmd("nvim", "myproj", &sessions), "nvim");
    }

    #[test]
    fn open_expands_claude_shorthand_in_layout() {
        let (_dir, repo) = setup_repo();
        crate::cli::new::run(&repo, "myproj", "/some/path", None, None, None, &[]).unwrap();

        let mut config = repo.load("myproj").unwrap();
        config.layout = Some(Layout {
            main: MainPane {
                cmd: Some("nvim".to_string()),
            },
            panes: vec![SplitPane {
                split: SplitDirection::Right,
                cmd: Some("claude:code-review".to_string()),
                size: None,
            }],
            layout_string: None,
        });
        config.claude_sessions = vec![ClaudeSession {
            id: "sess_1".to_string(),
            label: "code-review".to_string(),
            started_at: dt("2026-03-01T10:00:00Z"),
            status: ClaudeSessionStatus::Active,
        }];
        repo.save(&config).unwrap();

        let tmux = MockTmuxAdapter::no_session();
        let terminal = MockTerminalAdapter::new();

        run(&repo, &tmux, &terminal, "myproj", false, false).unwrap();

        let calls = tmux.calls();
        assert!(
            calls.contains(
                &"send_keys(myproj:0, devs claude myproj --resume code-review)".to_string()
            )
        );
    }

    #[test]
    fn handles_active_claude_sessions() {
        let (_dir, repo) = setup_repo();
        crate::cli::new::run(&repo, "myproj", "/some/path", None, None, None, &[]).unwrap();

        let mut config = repo.load("myproj").unwrap();
        config.claude_sessions = vec![
            ClaudeSession {
                id: "sess_1".to_string(),
                label: "brainstorm".to_string(),
                started_at: dt("2026-03-01T10:00:00Z"),
                status: ClaudeSessionStatus::Active,
            },
            ClaudeSession {
                id: "sess_2".to_string(),
                label: "done-session".to_string(),
                started_at: dt("2026-03-01T10:00:00Z"),
                status: ClaudeSessionStatus::Done(dt("2026-03-02T10:00:00Z")),
            },
        ];
        repo.save(&config).unwrap();

        let tmux = MockTmuxAdapter::no_session();
        let terminal = MockTerminalAdapter::new();

        // Should not error — active sessions are printed but we don't capture stdout here
        run(&repo, &tmux, &terminal, "myproj", false, false).unwrap();
    }

    #[test]
    fn open_applies_layout_string_from_declarative() {
        let (_dir, repo) = setup_repo();
        crate::cli::new::run(&repo, "myproj", "/some/path", None, None, None, &[]).unwrap();

        let mut config = repo.load("myproj").unwrap();
        config.layout = Some(Layout {
            main: MainPane {
                cmd: Some("nvim".to_string()),
            },
            panes: vec![SplitPane {
                split: SplitDirection::Right,
                cmd: Some("cargo watch".to_string()),
                size: None,
            }],
            layout_string: Some("5aed,176x79,0,0".to_string()),
        });
        repo.save(&config).unwrap();

        let tmux = MockTmuxAdapter::no_session();
        let terminal = MockTerminalAdapter::new();

        run(&repo, &tmux, &terminal, "myproj", false, false).unwrap();

        let calls = tmux.calls();
        assert!(calls.contains(&"apply_layout(myproj, 5aed,176x79,0,0)".to_string()));
    }
}
