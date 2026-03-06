use anyhow::Result;

use crate::domain::note::Note;
use crate::ports::project_repository::ProjectRepository;

pub fn run(repo: &dyn ProjectRepository, name: &str, message: &str) -> Result<()> {
    let mut config = repo.load(name)?;
    let note = Note {
        content: message.to_string(),
        created_at: chrono::Utc::now(),
    };
    config.notes.push(note);
    repo.save(&config)?;
    println!("Added note to '{name}'.");
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
    fn note_adds_to_project() {
        let (repo, _dir) = test_repo();
        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams::new("myproject", "/some/path"),
        )
        .unwrap();

        run(&repo, "myproject", "picking up from step 4").unwrap();

        let config = repo.load("myproject").unwrap();
        assert_eq!(config.notes.len(), 1);
        assert_eq!(config.notes[0].content, "picking up from step 4");
    }

    #[test]
    fn note_appends_to_existing_notes() {
        let (repo, _dir) = test_repo();
        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams::new("myproject", "/some/path"),
        )
        .unwrap();

        run(&repo, "myproject", "first note").unwrap();
        run(&repo, "myproject", "second note").unwrap();

        let config = repo.load("myproject").unwrap();
        assert_eq!(config.notes.len(), 2);
        assert_eq!(config.notes[0].content, "first note");
        assert_eq!(config.notes[1].content, "second note");
    }

    #[test]
    fn note_fails_for_missing_project() {
        let (repo, _dir) = test_repo();

        let result = run(&repo, "nonexistent", "some note");
        assert!(result.is_err());
    }

    #[test]
    fn note_sets_created_at() {
        let (repo, _dir) = test_repo();
        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams::new("myproject", "/some/path"),
        )
        .unwrap();

        let before = chrono::Utc::now();
        run(&repo, "myproject", "timestamped").unwrap();
        let after = chrono::Utc::now();

        let config = repo.load("myproject").unwrap();
        assert!(config.notes[0].created_at >= before);
        assert!(config.notes[0].created_at <= after);
    }
}
