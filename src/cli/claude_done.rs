use anyhow::{Result, bail};

use crate::domain::claude_session::ClaudeSessionStatus;
use crate::ports::project_repository::ProjectRepository;

pub fn run(repo: &dyn ProjectRepository, name: &str, label: &str) -> Result<()> {
    let mut config = repo.load(name)?;
    let session = config.claude_sessions.iter_mut().find(|s| s.label == label);
    match session {
        Some(session) => {
            if let ClaudeSessionStatus::Done(_) = session.status {
                bail!("session '{label}' is already done");
            }
            session.status = ClaudeSessionStatus::Done(chrono::Utc::now());
            repo.save(&config)?;
            println!("Marked session '{label}' as done.");
            Ok(())
        }
        None => bail!("session '{label}' not found in project '{name}'"),
    }
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
    fn done_marks_active_session_as_done() {
        let (repo, _dir) = test_repo();
        create_project_with_sessions(
            &repo,
            vec![ClaudeSession {
                id: "session-1".to_string(),
                label: "brainstorm".to_string(),
                started_at: dt("2026-03-01T10:00:00Z"),
                status: ClaudeSessionStatus::Active,
            }],
        );

        run(&repo, "myproject", "brainstorm").unwrap();

        let config = repo.load("myproject").unwrap();
        assert!(matches!(
            config.claude_sessions[0].status,
            ClaudeSessionStatus::Done(_)
        ));
    }

    #[test]
    fn done_fails_for_nonexistent_session() {
        let (repo, _dir) = test_repo();
        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams::new("myproject", "/some/path"),
        )
        .unwrap();

        let result = run(&repo, "myproject", "nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn done_fails_for_already_done_session() {
        let (repo, _dir) = test_repo();
        create_project_with_sessions(
            &repo,
            vec![ClaudeSession {
                id: "session-1".to_string(),
                label: "brainstorm".to_string(),
                started_at: dt("2026-03-01T10:00:00Z"),
                status: ClaudeSessionStatus::Done(dt("2026-03-01T12:00:00Z")),
            }],
        );

        let result = run(&repo, "myproject", "brainstorm");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already done"));
    }

    #[test]
    fn done_fails_for_missing_project() {
        let (repo, _dir) = test_repo();

        let result = run(&repo, "nonexistent", "some-label");
        assert!(result.is_err());
    }

    #[test]
    fn done_only_marks_specified_session() {
        let (repo, _dir) = test_repo();
        create_project_with_sessions(
            &repo,
            vec![
                ClaudeSession {
                    id: "session-1".to_string(),
                    label: "first".to_string(),
                    started_at: dt("2026-03-01T10:00:00Z"),
                    status: ClaudeSessionStatus::Active,
                },
                ClaudeSession {
                    id: "session-2".to_string(),
                    label: "second".to_string(),
                    started_at: dt("2026-03-02T10:00:00Z"),
                    status: ClaudeSessionStatus::Active,
                },
            ],
        );

        run(&repo, "myproject", "first").unwrap();

        let config = repo.load("myproject").unwrap();
        assert!(matches!(
            config.claude_sessions[0].status,
            ClaudeSessionStatus::Done(_)
        ));
        assert_eq!(
            config.claude_sessions[1].status,
            ClaudeSessionStatus::Active
        );
    }
}
