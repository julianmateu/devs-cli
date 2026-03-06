use anyhow::{Result, bail};

use super::format::expand_home;
use crate::domain::claude_session::{ClaudeSession, ClaudeSessionStatus};
use crate::domain::project::{ProjectConfig, ProjectMetadata};
use crate::ports::project_repository::ProjectRepository;

pub fn run(
    repo: &dyn ProjectRepository,
    name: &str,
    path: &str,
    color: Option<&str>,
    from: Option<&str>,
    sessions: &[String],
) -> Result<()> {
    if repo.load(name).is_ok() {
        bail!("project '{name}' already exists");
    }

    let metadata = ProjectMetadata {
        name: String::from(name),
        path: expand_home(path),
        color: color.map(String::from),
        created_at: chrono::Utc::now(),
    };
    metadata.validate()?;

    let layout = if let Some(source_name) = from {
        let source = repo.load(source_name)?;
        source.layout
    } else {
        None
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

        run(&repo, "my-project", "/some/path", None, None, &[]).unwrap();

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
            "my-project",
            "/some/path",
            Some("#e06c75"),
            None,
            &[],
        )
        .unwrap();

        let config = repo.load("my-project").unwrap();
        assert_eq!(config.project.color, Some("#e06c75".to_string()));
    }

    #[test]
    fn new_rejects_duplicate_name() {
        let (repo, _dir) = test_repo();

        run(&repo, "my-project", "/some/path", None, None, &[]).unwrap();
        let result = run(&repo, "my-project", "/other/path", None, None, &[]);

        assert!(result.is_err());
    }

    #[test]
    fn new_rejects_invalid_name() {
        let (repo, _dir) = test_repo();

        let result = run(&repo, "bad.name", "/some/path", None, None, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn new_rejects_invalid_color() {
        let (repo, _dir) = test_repo();

        let result = run(
            &repo,
            "my-project",
            "/some/path",
            Some("not-hex"),
            None,
            &[],
        );
        assert!(result.is_err());
    }

    #[test]
    fn from_copies_layout() {
        let (repo, _dir) = test_repo();

        // Create source with a layout
        run(&repo, "source", "/src/path", None, None, &[]).unwrap();
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

        // Create new project from source
        run(&repo, "target", "/tgt/path", None, Some("source"), &[]).unwrap();

        let target = repo.load("target").unwrap();
        assert_eq!(target.layout, source_config.layout);
    }

    #[test]
    fn from_does_not_copy_notes_or_last_state() {
        let (repo, _dir) = test_repo();

        run(&repo, "source", "/src/path", None, None, &[]).unwrap();
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

        run(&repo, "target", "/tgt/path", None, Some("source"), &[]).unwrap();

        let target = repo.load("target").unwrap();
        assert!(target.notes.is_empty());
        assert!(target.last_state.is_none());
    }

    #[test]
    fn from_uses_new_name_path_color() {
        let (repo, _dir) = test_repo();

        run(&repo, "source", "/src/path", Some("#111111"), None, &[]).unwrap();

        run(
            &repo,
            "target",
            "/tgt/path",
            Some("#222222"),
            Some("source"),
            &[],
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

        run(&repo, "source", "/src/path", None, None, &[]).unwrap();

        let sessions = vec!["main:abc123".to_string()];
        run(
            &repo,
            "target",
            "/tgt/path",
            None,
            Some("source"),
            &sessions,
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

        run(&repo, "source", "/src/path", None, None, &[]).unwrap();

        let sessions = vec!["main:abc123".to_string(), "review:def456".to_string()];
        run(
            &repo,
            "target",
            "/tgt/path",
            None,
            Some("source"),
            &sessions,
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

        run(&repo, "source", "/src/path", None, None, &[]).unwrap();

        run(&repo, "target", "/tgt/path", None, Some("source"), &[]).unwrap();

        let target = repo.load("target").unwrap();
        assert!(target.claude_sessions.is_empty());
    }

    #[test]
    fn from_fails_if_source_missing() {
        let (repo, _dir) = test_repo();

        let result = run(&repo, "target", "/tgt/path", None, Some("nonexistent"), &[]);
        assert!(result.is_err());
    }

    #[test]
    fn from_with_invalid_session_format_fails() {
        let (repo, _dir) = test_repo();

        run(&repo, "source", "/src/path", None, None, &[]).unwrap();

        let sessions = vec!["no-colon-here".to_string()];
        let result = run(
            &repo,
            "target",
            "/tgt/path",
            None,
            Some("source"),
            &sessions,
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("LABEL:ID"));
    }

    #[test]
    fn new_expands_tilde_in_path() {
        let (repo, _dir) = test_repo();
        let home = dirs::home_dir().unwrap();

        run(&repo, "tilded", "~/some/project", None, None, &[]).unwrap();

        let config = repo.load("tilded").unwrap();
        let expected = format!("{}/some/project", home.display());
        assert_eq!(config.project.path, expected);
    }

    #[test]
    fn from_works_when_source_has_no_layout() {
        let (repo, _dir) = test_repo();

        run(&repo, "source", "/src/path", None, None, &[]).unwrap();

        run(&repo, "target", "/tgt/path", None, Some("source"), &[]).unwrap();

        let target = repo.load("target").unwrap();
        assert!(target.layout.is_none());
    }
}
