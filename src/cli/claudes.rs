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
    use crate::domain::claude_session::{ClaudeSession, ClaudeSessionStatus};
    use crate::domain::test_helpers::dt;
    use crate::test_support::InMemoryProjectRepository;

    fn output_string(out: &[u8]) -> String {
        String::from_utf8(out.to_vec()).unwrap()
    }

    fn create_project_with_sessions(
        repo: &InMemoryProjectRepository,
        sessions: Vec<ClaudeSession>,
    ) {
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
        let repo = InMemoryProjectRepository::new();
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

        run(&repo, "myproject", false, &mut out).unwrap();

        let text = output_string(&out);
        assert!(
            text.contains("brainstorm"),
            "should show active session label"
        );
        assert!(text.contains("id-1"), "should show active session ID");
        assert!(!text.contains("refactor"), "should not show done session");
    }

    #[test]
    fn list_with_all_includes_done_sessions() {
        let repo = InMemoryProjectRepository::new();
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

        run(&repo, "myproject", true, &mut out).unwrap();

        let text = output_string(&out);
        assert!(text.contains("brainstorm"), "should show active session");
        assert!(text.contains("refactor"), "should show done session");
        assert!(text.contains("done"), "should show done status");
    }

    #[test]
    fn list_empty_sessions_shows_message() {
        let repo = InMemoryProjectRepository::new();
        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams::new("myproject", "/some/path"),
        )
        .unwrap();
        let mut out = Vec::new();

        run(&repo, "myproject", false, &mut out).unwrap();

        assert_eq!(output_string(&out), "No Claude sessions for 'myproject'.\n");
    }

    #[test]
    fn list_fails_for_missing_project() {
        let repo = InMemoryProjectRepository::new();
        let mut out = Vec::new();

        let result = run(&repo, "nonexistent", false, &mut out);
        assert!(result.is_err());
    }

    #[test]
    fn list_only_done_sessions_shows_empty_without_all() {
        let repo = InMemoryProjectRepository::new();
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

        run(&repo, "myproject", false, &mut out).unwrap();

        assert!(
            output_string(&out).contains("No Claude sessions"),
            "should show no sessions message"
        );
    }
}
