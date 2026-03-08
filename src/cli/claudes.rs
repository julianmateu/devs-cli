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

    struct Row {
        label: String,
        id: String,
        started: String,
        status: String,
    }

    let rows: Vec<Row> = sessions
        .iter()
        .map(|s| {
            let status = match &s.status {
                ClaudeSessionStatus::Active => "active".to_string(),
                ClaudeSessionStatus::Done(finished_at) => {
                    format!("done ({})", finished_at.format("%Y-%m-%d %H:%M"))
                }
            };
            Row {
                label: s.label.clone(),
                id: s.id.clone(),
                started: s.started_at.format("%Y-%m-%d %H:%M").to_string(),
                status,
            }
        })
        .collect();

    let w_label = rows.iter().map(|r| r.label.len()).max().unwrap_or(0).max(5);
    let w_id = rows.iter().map(|r| r.id.len()).max().unwrap_or(0).max(2);
    let w_started = 16; // "YYYY-MM-DD HH:MM"

    writeln!(
        out,
        "{:<w_label$}   {:<w_id$}   {:<w_started$}   STATUS",
        "LABEL", "ID", "STARTED"
    )?;

    for row in &rows {
        writeln!(
            out,
            "{:<w_label$}   {:<w_id$}   {:<w_started$}   {}",
            row.label, row.id, row.started, row.status
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
    fn list_shows_header_and_active_sessions() {
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
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines[0].contains("LABEL"), "first line should be header");
        assert!(lines[0].contains("ID"), "header should contain ID");
        assert!(lines[0].contains("STATUS"), "header should contain STATUS");
        assert!(text.contains("brainstorm"), "should show active label");
        assert!(text.contains("id-1"), "should show active ID");
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
