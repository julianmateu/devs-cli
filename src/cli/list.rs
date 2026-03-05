use anyhow::Result;

use crate::ports::project_repository::ProjectRepository;

pub fn run(repo: &dyn ProjectRepository) -> Result<()> {
    let names = repo.list()?;
    if names.is_empty() {
        println!("No projects registered.");
    } else {
        for name in names {
            println!("{name}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::toml_project_repository::TomlProjectRepository;
    use tempfile::tempdir;

    #[test]
    fn list_succeeds_when_empty() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());

        assert!(run(&repo).is_ok());
    }

    #[test]
    fn list_succeeds_with_projects() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        crate::cli::new::run(&repo, "test-project", "/some/path", None).unwrap();

        assert!(run(&repo).is_ok());
    }
}
