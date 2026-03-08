use std::io::Write;

use anyhow::Result;

use crate::domain::claude_session::ClaudeSessionStatus;
use crate::ports::project_repository::ProjectRepository;

pub fn run(repo: &dyn ProjectRepository, name: &str, all: bool, out: &mut dyn Write) -> Result<()> {
    let config = repo.load(name)?;
    let sessions: Vec<_> = config
        .claude_sessions
        .iter()
        .filter(|s| all || s.status == ClaudeSessionStatus::Active)
        .collect();

    if sessions.is_empty() {
        writeln!(out, "No Claude sessions for '{name}'.")?;
        return Ok(());
    }

    for session in sessions {
        let status = match &session.status {
            ClaudeSessionStatus::Active => "active".to_string(),
            ClaudeSessionStatus::Done(finished_at) => {
                format!("done ({})", finished_at.format("%Y-%m-%d %H:%M"))
            }
        };
        writeln!(
            out,
            "{}  {}  {}  [{}]",
            session.label,
            session.id,
            session.started_at.format("%Y-%m-%d %H:%M"),
            status,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::toml_project_repository::TomlProjectRepository;
    use crate::domain::claude_session::{ClaudeSession, ClaudeSessionStatus};
    use crate::domain::test_helpers::dt;
    use tempfile::tempdir;

    fn test_repo() -> (TomlProjectRepository, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        (repo, dir)
    }

    fn create_project_with_sessions(repo: &TomlProjectRepository, sessions: Vec<ClaudeSession>) {
        crate::cli::new::run(
            repo,
            crate::cli::new::NewProjectParams::new("myproject", "/some/path"),
        )
        .unwrap();
        let mut config = repo.load("myproject").unwrap();
        config.claude_sessions = sessions;
        repo.save(&config).unwrap();
    }

    #[test]
    fn list_shows_active_sessions_by_default() {
        let (repo, _dir) = test_repo();
        create_project_with_sessions(
            &repo,
            vec![
                ClaudeSession {
                    id: "id-1".to_string(),
                    label: "brainstorm".to_string(),
                    started_at: dt("2026-03-01T10:00:00Z"),
                    status: ClaudeSessionStatus::Active,
                },
                ClaudeSession {
                    id: "id-2".to_string(),
                    label: "refactor".to_string(),
                    started_at: dt("2026-03-02T10:00:00Z"),
                    status: ClaudeSessionStatus::Done(dt("2026-03-02T12:00:00Z")),
                },
            ],
        );
        let mut out = Vec::new();

        let result = run(&repo, "myproject", false, &mut out);
        assert!(result.is_ok());
    }

    #[test]
    fn list_with_all_includes_done_sessions() {
        let (repo, _dir) = test_repo();
        create_project_with_sessions(
            &repo,
            vec![
                ClaudeSession {
                    id: "id-1".to_string(),
                    label: "brainstorm".to_string(),
                    started_at: dt("2026-03-01T10:00:00Z"),
                    status: ClaudeSessionStatus::Active,
                },
                ClaudeSession {
                    id: "id-2".to_string(),
                    label: "refactor".to_string(),
                    started_at: dt("2026-03-02T10:00:00Z"),
                    status: ClaudeSessionStatus::Done(dt("2026-03-02T12:00:00Z")),
                },
            ],
        );
        let mut out = Vec::new();

        let result = run(&repo, "myproject", true, &mut out);
        assert!(result.is_ok());
    }

    #[test]
    fn list_empty_sessions() {
        let (repo, _dir) = test_repo();
        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams::new("myproject", "/some/path"),
        )
        .unwrap();
        let mut out = Vec::new();

        let result = run(&repo, "myproject", false, &mut out);
        assert!(result.is_ok());
    }

    #[test]
    fn list_fails_for_missing_project() {
        let (repo, _dir) = test_repo();
        let mut out = Vec::new();

        let result = run(&repo, "nonexistent", false, &mut out);
        assert!(result.is_err());
    }

    #[test]
    fn list_only_done_sessions_shows_empty_without_all() {
        let (repo, _dir) = test_repo();
        create_project_with_sessions(
            &repo,
            vec![ClaudeSession {
                id: "id-1".to_string(),
                label: "finished-work".to_string(),
                started_at: dt("2026-03-01T10:00:00Z"),
                status: ClaudeSessionStatus::Done(dt("2026-03-01T12:00:00Z")),
            }],
        );
        let mut out = Vec::new();

        let result = run(&repo, "myproject", false, &mut out);
        assert!(result.is_ok());
    }
}
