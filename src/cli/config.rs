use anyhow::Result;

use crate::ports::project_repository::ProjectRepository;

pub fn run(repo: &dyn ProjectRepository, name: &str) -> Result<()> {
    let config = repo.load(name)?;
    let toml = toml::to_string(&config)?;
    print!("{toml}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::toml_project_repository::TomlProjectRepository;
    use tempfile::tempdir;

    #[test]
    fn config_prints_existing_project() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        crate::cli::new::run(&repo, "test-project", "/some/path", None).unwrap();

        assert!(run(&repo, "test-project").is_ok());
    }

    #[test]
    fn config_fails_for_missing_project() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());

        assert!(run(&repo, "ghost").is_err());
    }
}
