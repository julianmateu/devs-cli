use anyhow::{Result, bail};
use uuid::Uuid;

use crate::domain::claude_session::{ClaudeSession, ClaudeSessionStatus};
use crate::ports::project_repository::ProjectRepository;

pub fn start(repo: &dyn ProjectRepository, name: &str, label: &str) -> Result<()> {
    let mut config = repo.load(name)?;
    let session_id = Uuid::new_v4().to_string();
    let session = ClaudeSession {
        id: session_id.clone(),
        label: label.to_string(),
        started_at: chrono::Utc::now(),
        status: ClaudeSessionStatus::Active,
    };
    config.claude_sessions.push(session);
    repo.save(&config)?;
    println!("{session_id}");
    Ok(())
}

pub fn resume(repo: &dyn ProjectRepository, name: &str, id: &str) -> Result<()> {
    let config = repo.load(name)?;
    let session = config.claude_sessions.iter().find(|s| s.id == id);
    match session {
        Some(_) => {
            println!("claude --resume {id}");
            Ok(())
        }
        None => bail!("session '{id}' not found in project '{name}'"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::toml_project_repository::TomlProjectRepository;
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
    fn start_adds_active_session_to_project() {
        let (repo, _dir) = test_repo();
        create_project(&repo, "myproject");

        start(&repo, "myproject", "brainstorm").unwrap();

        let config = repo.load("myproject").unwrap();
        assert_eq!(config.claude_sessions.len(), 1);
        let session = &config.claude_sessions[0];
        assert_eq!(session.label, "brainstorm");
        assert_eq!(session.status, ClaudeSessionStatus::Active);
        assert!(!session.id.is_empty());
    }

    #[test]
    fn start_generates_unique_ids() {
        let (repo, _dir) = test_repo();
        create_project(&repo, "myproject");

        start(&repo, "myproject", "session-a").unwrap();
        start(&repo, "myproject", "session-b").unwrap();

        let config = repo.load("myproject").unwrap();
        assert_eq!(config.claude_sessions.len(), 2);
        assert_ne!(config.claude_sessions[0].id, config.claude_sessions[1].id);
    }

    #[test]
    fn start_fails_for_missing_project() {
        let (repo, _dir) = test_repo();

        let result = start(&repo, "nonexistent", "label");
        assert!(result.is_err());
    }

    // --- resume tests ---

    #[test]
    fn resume_prints_command_for_existing_session() {
        let (repo, _dir) = test_repo();
        create_project(&repo, "myproject");
        start(&repo, "myproject", "brainstorm").unwrap();

        let config = repo.load("myproject").unwrap();
        let session_id = &config.claude_sessions[0].id;

        let result = resume(&repo, "myproject", session_id);
        assert!(result.is_ok());
    }

    #[test]
    fn resume_fails_for_nonexistent_session() {
        let (repo, _dir) = test_repo();
        create_project(&repo, "myproject");

        let result = resume(&repo, "myproject", "nonexistent-id");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("session 'nonexistent-id' not found")
        );
    }

    #[test]
    fn resume_fails_for_missing_project() {
        let (repo, _dir) = test_repo();

        let result = resume(&repo, "nonexistent", "some-id");
        assert!(result.is_err());
    }
}
