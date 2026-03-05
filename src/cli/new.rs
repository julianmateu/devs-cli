use anyhow::{Result, bail};

use crate::domain::project::{ProjectConfig, ProjectMetadata};
use crate::ports::project_repository::ProjectRepository;

pub fn run(
    repo: &dyn ProjectRepository,
    name: &str,
    path: &str,
    color: Option<&str>,
) -> Result<()> {
    if repo.load(name).is_ok() {
        bail!("project '{}' already exists", name);
    }
    let metadata = ProjectMetadata {
        name: String::from(name),
        path: String::from(path),
        color: color.map(String::from),
        created_at: chrono::Utc::now(),
    };
    metadata.validate()?;
    let config = ProjectConfig {
        project: metadata,
        layout: None,
        claude_sessions: vec![],
        notes: vec![],
        last_state: None,
    };
    repo.save(&config)?;
    println!("Created project '{name}'.");
    Ok(())
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

    #[test]
    fn new_creates_project() {
        let (repo, _dir) = test_repo();

        run(&repo, "my-project", "/some/path", None).unwrap();

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

        run(&repo, "my-project", "/some/path", Some("#e06c75")).unwrap();

        let config = repo.load("my-project").unwrap();
        assert_eq!(config.project.color, Some("#e06c75".to_string()));
    }

    #[test]
    fn new_rejects_duplicate_name() {
        let (repo, _dir) = test_repo();

        run(&repo, "my-project", "/some/path", None).unwrap();
        let result = run(&repo, "my-project", "/other/path", None);

        assert!(result.is_err());
    }

    #[test]
    fn new_rejects_invalid_name() {
        let (repo, _dir) = test_repo();

        let result = run(&repo, "bad.name", "/some/path", None);
        assert!(result.is_err());
    }

    #[test]
    fn new_rejects_invalid_color() {
        let (repo, _dir) = test_repo();

        let result = run(&repo, "my-project", "/some/path", Some("not-hex"));
        assert!(result.is_err());
    }
}
