use anyhow::{Result, bail};
use uuid::Uuid;

use crate::domain::claude_session::{ClaudeSession, ClaudeSessionStatus};
use crate::ports::process_launcher::ProcessLauncher;
use crate::ports::project_repository::ProjectRepository;

pub fn start(
    repo: &dyn ProjectRepository,
    launcher: &dyn ProcessLauncher,
    name: &str,
    label: &str,
) -> Result<()> {
    let mut config = repo.load(name)?;
    let session_id = Uuid::new_v4().to_string();
    let session = ClaudeSession {
        id: session_id,
        label: label.to_string(),
        started_at: chrono::Utc::now(),
        status: ClaudeSessionStatus::Active,
    };
    config.claude_sessions.push(session);
    repo.save(&config)?;

    let path = config.project.path.clone();
    launcher.launch_claude(&[], &path)?;
    Ok(())
}

pub fn resume(
    repo: &dyn ProjectRepository,
    launcher: &dyn ProcessLauncher,
    name: &str,
    label: &str,
) -> Result<()> {
    let config = repo.load(name)?;
    let session = config.claude_sessions.iter().find(|s| s.label == label);

    match session {
        Some(s) => {
            let path = config.project.path.clone();
            launcher.launch_claude(&["--resume", &s.id], &path)?;
            Ok(())
        }
        None => bail!("no session '{label}' found in project '{name}'"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::toml_project_repository::TomlProjectRepository;
    use crate::domain::claude_session::ClaudeSessionStatus;
    use crate::domain::test_helpers::MockProcessLauncher;
    use tempfile::tempdir;

    fn test_repo() -> (TomlProjectRepository, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        (repo, dir)
    }

    fn create_project(repo: &TomlProjectRepository, name: &str) {
        crate::cli::new::run(repo, name, "/some/path", None).unwrap();
    }

    // --- start tests ---

    #[test]
    fn start_records_session_and_launches_claude() {
        let (repo, _dir) = test_repo();
        create_project(&repo, "myproject");
        let launcher = MockProcessLauncher::new();

        start(&repo, &launcher, "myproject", "brainstorm").unwrap();

        // Session is recorded
        let config = repo.load("myproject").unwrap();
        assert_eq!(config.claude_sessions.len(), 1);
        let session = &config.claude_sessions[0];
        assert_eq!(session.label, "brainstorm");
        assert_eq!(session.status, ClaudeSessionStatus::Active);

        // Claude was launched in the project directory with no extra args
        let calls = launcher.calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], "launch_claude([], /some/path)");
    }

    #[test]
    fn start_generates_unique_ids() {
        let (repo, _dir) = test_repo();
        create_project(&repo, "myproject");
        let launcher = MockProcessLauncher::new();

        start(&repo, &launcher, "myproject", "session-a").unwrap();
        start(&repo, &launcher, "myproject", "session-b").unwrap();

        let config = repo.load("myproject").unwrap();
        assert_eq!(config.claude_sessions.len(), 2);
        assert_ne!(config.claude_sessions[0].id, config.claude_sessions[1].id);
    }

    #[test]
    fn start_fails_for_missing_project() {
        let (repo, _dir) = test_repo();
        let launcher = MockProcessLauncher::new();

        let result = start(&repo, &launcher, "nonexistent", "label");
        assert!(result.is_err());
        // Claude should NOT have been launched
        assert!(launcher.calls().is_empty());
    }

    // --- resume tests ---

    #[test]
    fn resume_finds_session_by_label_and_launches_claude() {
        let (repo, _dir) = test_repo();
        create_project(&repo, "myproject");
        let launcher = MockProcessLauncher::new();

        start(&repo, &launcher, "myproject", "brainstorm").unwrap();

        // Get the stored session ID
        let config = repo.load("myproject").unwrap();
        let session_id = config.claude_sessions[0].id.clone();

        let resume_launcher = MockProcessLauncher::new();
        resume(&repo, &resume_launcher, "myproject", "brainstorm").unwrap();

        let calls = resume_launcher.calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(
            calls[0],
            format!("launch_claude([--resume, {session_id}], /some/path)")
        );
    }

    #[test]
    fn resume_fails_for_unknown_label() {
        let (repo, _dir) = test_repo();
        create_project(&repo, "myproject");
        let launcher = MockProcessLauncher::new();

        let result = resume(&repo, &launcher, "myproject", "nonexistent");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("no session 'nonexistent'")
        );
        assert!(launcher.calls().is_empty());
    }

    #[test]
    fn resume_fails_for_missing_project() {
        let (repo, _dir) = test_repo();
        let launcher = MockProcessLauncher::new();

        let result = resume(&repo, &launcher, "nonexistent", "some-label");
        assert!(result.is_err());
        assert!(launcher.calls().is_empty());
    }
}
