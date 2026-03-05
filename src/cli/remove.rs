use anyhow::{Result, bail};

use crate::ports::project_repository::ProjectRepository;

pub fn run(repo: &dyn ProjectRepository, name: &str, force: bool) -> Result<()> {
    if !force {
        bail!("removing project '{name}' will delete its config. Use --force to confirm.");
    }
    repo.delete(name)?;
    println!("Removed project '{name}'.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::toml_project_repository::TomlProjectRepository;
    use tempfile::tempdir;

    #[test]
    fn remove_deletes_project_with_force() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        crate::cli::new::run(&repo, "doomed", "/some/path", None).unwrap();

        assert!(run(&repo, "doomed", true).is_ok());
        assert!(repo.load("doomed").is_err());
    }

    #[test]
    fn remove_fails_without_force() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        crate::cli::new::run(&repo, "safe", "/some/path", None).unwrap();

        assert!(run(&repo, "safe", false).is_err());
        assert!(repo.load("safe").is_ok());
    }

    #[test]
    fn remove_fails_for_missing_project() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());

        assert!(run(&repo, "ghost", true).is_err());
    }
}
