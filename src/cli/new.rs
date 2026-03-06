use anyhow::{Result, bail};

use crate::domain::claude_session::{ClaudeSession, ClaudeSessionStatus};
use crate::domain::layout::Layout;
use crate::domain::local_config::LocalConfig;
use crate::domain::project::{ProjectConfig, ProjectMetadata};
use crate::ports::project_repository::ProjectRepository;

pub struct NewProjectParams<'a> {
    pub name: &'a str,
    pub path: &'a str,
    pub color: Option<&'a str>,
    pub from: Option<&'a str>,
    pub from_layout: Option<Layout>,
    pub sessions: &'a [String],
    pub local_config: Option<LocalConfig>,
}

impl<'a> NewProjectParams<'a> {
    pub fn new(name: &'a str, path: &'a str) -> Self {
        Self {
            name,
            path,
            color: None,
            from: None,
            from_layout: None,
            sessions: &[],
            local_config: None,
        }
    }
}

pub fn run(repo: &dyn ProjectRepository, params: NewProjectParams) -> Result<()> {
    let NewProjectParams {
        name,
        path,
        color,
        from,
        from_layout,
        sessions,
        local_config,
    } = params;

    if repo.load(name).is_ok() {
        bail!("project '{name}' already exists");
    }

    let resolved_color = color
        .map(String::from)
        .or_else(|| local_config.as_ref().and_then(|lc| lc.color.clone()));

    let metadata = ProjectMetadata {
        name: String::from(name),
        path: String::from(path),
        color: resolved_color,
        created_at: chrono::Utc::now(),
    };
    metadata.validate()?;

    let had_local_config = local_config.is_some();

    let layout = if from_layout.is_some() {
        from_layout
    } else if let Some(source_name) = from {
        let source = repo.load(source_name)?;
        source.layout
    } else {
        local_config.and_then(|lc| lc.layout)
    };

    let claude_sessions = parse_sessions(sessions)?;

    let config = ProjectConfig {
        project: metadata,
        layout,
        claude_sessions,
        notes: vec![],
        last_state: None,
    };
    repo.save(&config)?;

    if let Some(source_name) = from {
        println!(
            "Created project '{name}' from '{source_name}'. Run 'devs edit {name}' to review the config."
        );
    } else if had_local_config {
        println!("Created project '{name}' (using .devs.toml).");
    } else {
        println!("Created project '{name}'.");
    }
    Ok(())
}

fn parse_sessions(raw: &[String]) -> Result<Vec<ClaudeSession>> {
    raw.iter()
        .map(|s| {
            let (label, id) = s.split_once(':').ok_or_else(|| {
                anyhow::anyhow!(
                    "invalid session format '{s}': expected LABEL:ID (e.g., main:abc123)"
                )
            })?;
            Ok(ClaudeSession {
                id: id.to_string(),
                label: label.to_string(),
                started_at: chrono::Utc::now(),
                status: ClaudeSessionStatus::Active,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::toml_project_repository::TomlProjectRepository;
    use crate::domain::layout::{Layout, MainPane, SplitDirection, SplitPane};
    use crate::domain::note::Note;
    use crate::domain::saved_state::{SavedPane, SavedState};
    use crate::domain::test_helpers::dt;
    use tempfile::tempdir;

    fn test_repo() -> (TomlProjectRepository, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        (repo, dir)
    }

    #[test]
    fn new_creates_project() {
        let (repo, _dir) = test_repo();

        run(&repo, NewProjectParams::new("my-project", "/some/path")).unwrap();

        let config = repo.load("my-project").unwrap();
        assert_eq!(config.project.name, "my-project");
        assert_eq!(config.project.path, "/some/path");
        assert!(config.project.color.is_none());
        assert!(config.layout.is_none());
        assert!(config.claude_sessions.is_empty());
        assert!(config.notes.is_empty());
    }

    #[test]
    fn new_with_color() {
        let (repo, _dir) = test_repo();

        run(
            &repo,
            NewProjectParams {
                color: Some("#e06c75"),
                ..NewProjectParams::new("my-project", "/some/path")
            },
        )
        .unwrap();

        let config = repo.load("my-project").unwrap();
        assert_eq!(config.project.color, Some("#e06c75".to_string()));
    }

    #[test]
    fn new_rejects_duplicate_name() {
        let (repo, _dir) = test_repo();

        run(&repo, NewProjectParams::new("my-project", "/some/path")).unwrap();
        let result = run(&repo, NewProjectParams::new("my-project", "/other/path"));

        assert!(result.is_err());
    }

    #[test]
    fn new_rejects_invalid_name() {
        let (repo, _dir) = test_repo();

        let result = run(&repo, NewProjectParams::new("bad.name", "/some/path"));
        assert!(result.is_err());
    }

    #[test]
    fn new_rejects_invalid_color() {
        let (repo, _dir) = test_repo();

        let result = run(
            &repo,
            NewProjectParams {
                color: Some("not-hex"),
                ..NewProjectParams::new("my-project", "/some/path")
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn from_copies_layout() {
        let (repo, _dir) = test_repo();

        run(&repo, NewProjectParams::new("source", "/src/path")).unwrap();
        let mut source_config = repo.load("source").unwrap();
        source_config.layout = Some(Layout {
            main: MainPane {
                cmd: Some("nvim".to_string()),
            },
            panes: vec![SplitPane {
                split: SplitDirection::Right,
                cmd: Some("claude:main".to_string()),
                size: Some("40%".to_string()),
            }],
            layout_string: None,
        });
        repo.save(&source_config).unwrap();

        run(
            &repo,
            NewProjectParams {
                from: Some("source"),
                ..NewProjectParams::new("target", "/tgt/path")
            },
        )
        .unwrap();

        let target = repo.load("target").unwrap();
        assert_eq!(target.layout, source_config.layout);
    }

    #[test]
    fn from_does_not_copy_notes_or_last_state() {
        let (repo, _dir) = test_repo();

        run(&repo, NewProjectParams::new("source", "/src/path")).unwrap();
        let mut source_config = repo.load("source").unwrap();
        source_config.notes = vec![Note {
            content: "some note".to_string(),
            created_at: dt("2026-03-01T10:00:00Z"),
        }];
        source_config.last_state = Some(SavedState {
            captured_at: dt("2026-03-03T16:00:00Z"),
            layout_string: "layout".to_string(),
            panes: vec![SavedPane {
                index: 0,
                path: "/src/path".to_string(),
                command: "zsh".to_string(),
            }],
        });
        repo.save(&source_config).unwrap();

        run(
            &repo,
            NewProjectParams {
                from: Some("source"),
                ..NewProjectParams::new("target", "/tgt/path")
            },
        )
        .unwrap();

        let target = repo.load("target").unwrap();
        assert!(target.notes.is_empty());
        assert!(target.last_state.is_none());
    }

    #[test]
    fn from_uses_new_name_path_color() {
        let (repo, _dir) = test_repo();

        run(
            &repo,
            NewProjectParams {
                color: Some("#111111"),
                ..NewProjectParams::new("source", "/src/path")
            },
        )
        .unwrap();

        run(
            &repo,
            NewProjectParams {
                color: Some("#222222"),
                from: Some("source"),
                ..NewProjectParams::new("target", "/tgt/path")
            },
        )
        .unwrap();

        let target = repo.load("target").unwrap();
        assert_eq!(target.project.name, "target");
        assert_eq!(target.project.path, "/tgt/path");
        assert_eq!(target.project.color, Some("#222222".to_string()));
    }

    #[test]
    fn from_with_session_creates_session() {
        let (repo, _dir) = test_repo();

        run(&repo, NewProjectParams::new("source", "/src/path")).unwrap();

        let sessions = vec!["main:abc123".to_string()];
        run(
            &repo,
            NewProjectParams {
                from: Some("source"),
                sessions: &sessions,
                ..NewProjectParams::new("target", "/tgt/path")
            },
        )
        .unwrap();

        let target = repo.load("target").unwrap();
        assert_eq!(target.claude_sessions.len(), 1);
        assert_eq!(target.claude_sessions[0].label, "main");
        assert_eq!(target.claude_sessions[0].id, "abc123");
        assert!(matches!(
            target.claude_sessions[0].status,
            ClaudeSessionStatus::Active
        ));
    }

    #[test]
    fn from_with_multiple_sessions() {
        let (repo, _dir) = test_repo();

        run(&repo, NewProjectParams::new("source", "/src/path")).unwrap();

        let sessions = vec!["main:abc123".to_string(), "review:def456".to_string()];
        run(
            &repo,
            NewProjectParams {
                from: Some("source"),
                sessions: &sessions,
                ..NewProjectParams::new("target", "/tgt/path")
            },
        )
        .unwrap();

        let target = repo.load("target").unwrap();
        assert_eq!(target.claude_sessions.len(), 2);
        assert_eq!(target.claude_sessions[0].label, "main");
        assert_eq!(target.claude_sessions[1].label, "review");
    }

    #[test]
    fn from_with_no_sessions_creates_none() {
        let (repo, _dir) = test_repo();

        run(&repo, NewProjectParams::new("source", "/src/path")).unwrap();
        run(
            &repo,
            NewProjectParams {
                from: Some("source"),
                ..NewProjectParams::new("target", "/tgt/path")
            },
        )
        .unwrap();

        let target = repo.load("target").unwrap();
        assert!(target.claude_sessions.is_empty());
    }

    #[test]
    fn from_fails_if_source_missing() {
        let (repo, _dir) = test_repo();

        let result = run(
            &repo,
            NewProjectParams {
                from: Some("nonexistent"),
                ..NewProjectParams::new("target", "/tgt/path")
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn from_with_invalid_session_format_fails() {
        let (repo, _dir) = test_repo();

        run(&repo, NewProjectParams::new("source", "/src/path")).unwrap();

        let sessions = vec!["no-colon-here".to_string()];
        let result = run(
            &repo,
            NewProjectParams {
                from: Some("source"),
                sessions: &sessions,
                ..NewProjectParams::new("target", "/tgt/path")
            },
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("LABEL:ID"));
    }

    #[test]
    fn from_works_when_source_has_no_layout() {
        let (repo, _dir) = test_repo();

        run(&repo, NewProjectParams::new("source", "/src/path")).unwrap();
        run(
            &repo,
            NewProjectParams {
                from: Some("source"),
                ..NewProjectParams::new("target", "/tgt/path")
            },
        )
        .unwrap();

        let target = repo.load("target").unwrap();
        assert!(target.layout.is_none());
    }

    #[test]
    fn new_from_layout_captures_layout() {
        let (repo, _dir) = test_repo();

        let captured = Layout::from_snapshot(
            "5aed,176x79,0,0".to_string(),
            &[
                SavedPane {
                    index: 0,
                    path: "/p".to_string(),
                    command: "nvim".to_string(),
                },
                SavedPane {
                    index: 1,
                    path: "/p".to_string(),
                    command: "zsh".to_string(),
                },
                SavedPane {
                    index: 2,
                    path: "/p".to_string(),
                    command: "cargo watch".to_string(),
                },
            ],
        );

        run(
            &repo,
            NewProjectParams {
                from_layout: Some(captured),
                ..NewProjectParams::new("my-project", "/some/path")
            },
        )
        .unwrap();

        let config = repo.load("my-project").unwrap();
        let layout = config.layout.expect("layout should be set");
        assert_eq!(layout.layout_string, Some("5aed,176x79,0,0".to_string()));
        assert_eq!(layout.main.cmd, Some("nvim".to_string()));
        assert_eq!(layout.panes.len(), 2);
        assert_eq!(layout.panes[0].cmd, None); // zsh is a shell
        assert_eq!(layout.panes[1].cmd, Some("cargo watch".to_string()));
    }

    #[test]
    fn new_from_layout_takes_priority_over_from() {
        let (repo, _dir) = test_repo();

        run(&repo, NewProjectParams::new("source", "/src/path")).unwrap();
        let mut source_config = repo.load("source").unwrap();
        source_config.layout = Some(Layout {
            main: MainPane {
                cmd: Some("emacs".to_string()),
            },
            panes: vec![],
            layout_string: None,
        });
        repo.save(&source_config).unwrap();

        let captured = Layout::from_snapshot(
            "captured-layout".to_string(),
            &[SavedPane {
                index: 0,
                path: "/p".to_string(),
                command: "nvim".to_string(),
            }],
        );

        run(
            &repo,
            NewProjectParams {
                from: Some("source"),
                from_layout: Some(captured),
                ..NewProjectParams::new("target", "/tgt/path")
            },
        )
        .unwrap();

        let config = repo.load("target").unwrap();
        let layout = config.layout.expect("layout should be set");
        assert_eq!(layout.layout_string, Some("captured-layout".to_string()));
        assert_eq!(layout.main.cmd, Some("nvim".to_string()));
    }

    // --- Local config merge tests ---

    fn sample_local_config() -> LocalConfig {
        LocalConfig {
            color: Some("#aabbcc".to_string()),
            layout: Some(Layout {
                main: MainPane {
                    cmd: Some("nvim".to_string()),
                },
                panes: vec![SplitPane {
                    split: SplitDirection::Right,
                    cmd: Some("claude".to_string()),
                    size: Some("40%".to_string()),
                }],
                layout_string: None,
            }),
        }
    }

    #[test]
    fn local_config_provides_color() {
        let (repo, _dir) = test_repo();

        run(
            &repo,
            NewProjectParams {
                local_config: Some(LocalConfig {
                    color: Some("#aabbcc".to_string()),
                    layout: None,
                }),
                ..NewProjectParams::new("my-project", "/some/path")
            },
        )
        .unwrap();

        let config = repo.load("my-project").unwrap();
        assert_eq!(config.project.color, Some("#aabbcc".to_string()));
    }

    #[test]
    fn flag_color_overrides_local_config() {
        let (repo, _dir) = test_repo();

        run(
            &repo,
            NewProjectParams {
                color: Some("#ffffff"),
                local_config: Some(LocalConfig {
                    color: Some("#aabbcc".to_string()),
                    layout: None,
                }),
                ..NewProjectParams::new("my-project", "/some/path")
            },
        )
        .unwrap();

        let config = repo.load("my-project").unwrap();
        assert_eq!(config.project.color, Some("#ffffff".to_string()));
    }

    #[test]
    fn local_config_provides_layout() {
        let (repo, _dir) = test_repo();
        let lc = sample_local_config();
        let expected_layout = lc.layout.clone();

        run(
            &repo,
            NewProjectParams {
                local_config: Some(lc),
                ..NewProjectParams::new("my-project", "/some/path")
            },
        )
        .unwrap();

        let config = repo.load("my-project").unwrap();
        assert_eq!(config.layout, expected_layout);
    }

    #[test]
    fn from_overrides_local_config_layout() {
        let (repo, _dir) = test_repo();

        run(&repo, NewProjectParams::new("source", "/src/path")).unwrap();
        let mut source_config = repo.load("source").unwrap();
        let source_layout = Layout {
            main: MainPane {
                cmd: Some("emacs".to_string()),
            },
            panes: vec![],
            layout_string: None,
        };
        source_config.layout = Some(source_layout.clone());
        repo.save(&source_config).unwrap();

        run(
            &repo,
            NewProjectParams {
                from: Some("source"),
                local_config: Some(sample_local_config()),
                ..NewProjectParams::new("target", "/tgt/path")
            },
        )
        .unwrap();

        let config = repo.load("target").unwrap();
        assert_eq!(config.layout, Some(source_layout));
    }

    #[test]
    fn from_without_layout_does_not_fallback_to_local_config() {
        let (repo, _dir) = test_repo();

        // Source project has no layout
        run(&repo, NewProjectParams::new("source", "/src/path")).unwrap();

        run(
            &repo,
            NewProjectParams {
                from: Some("source"),
                local_config: Some(sample_local_config()),
                ..NewProjectParams::new("target", "/tgt/path")
            },
        )
        .unwrap();

        let config = repo.load("target").unwrap();
        // --from fully overrides: even though local_config has a layout, source had None
        assert!(config.layout.is_none());
    }

    #[test]
    fn from_layout_overrides_local_config_layout() {
        let (repo, _dir) = test_repo();

        let captured = Layout::from_snapshot(
            "captured".to_string(),
            &[SavedPane {
                index: 0,
                path: "/p".to_string(),
                command: "nvim".to_string(),
            }],
        );

        run(
            &repo,
            NewProjectParams {
                from_layout: Some(captured.clone()),
                local_config: Some(sample_local_config()),
                ..NewProjectParams::new("my-project", "/some/path")
            },
        )
        .unwrap();

        let config = repo.load("my-project").unwrap();
        assert_eq!(config.layout, Some(captured));
    }

    #[test]
    fn local_config_none_unchanged() {
        let (repo, _dir) = test_repo();

        run(&repo, NewProjectParams::new("my-project", "/some/path")).unwrap();

        let config = repo.load("my-project").unwrap();
        assert!(config.project.color.is_none());
        assert!(config.layout.is_none());
    }

    #[test]
    fn local_config_color_and_layout_both_used() {
        let (repo, _dir) = test_repo();
        let lc = sample_local_config();
        let expected_layout = lc.layout.clone();

        run(
            &repo,
            NewProjectParams {
                local_config: Some(lc),
                ..NewProjectParams::new("my-project", "/some/path")
            },
        )
        .unwrap();

        let config = repo.load("my-project").unwrap();
        assert_eq!(config.project.color, Some("#aabbcc".to_string()));
        assert_eq!(config.layout, expected_layout);
    }
}
