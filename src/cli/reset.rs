use anyhow::Result;

use crate::ports::project_repository::ProjectRepository;

pub fn run(repo: &dyn ProjectRepository, name: &str) -> Result<()> {
    let mut config = repo.load(name)?;
    if config.last_state.is_none() {
        println!("No saved state for '{name}'.");
        return Ok(());
    }
    config.last_state = None;
    repo.save(&config)?;
    println!("Reset layout for '{name}' to default.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::toml_project_repository::TomlProjectRepository;
    use crate::domain::saved_state::SavedState;
    use crate::domain::test_helpers::dt;
    use tempfile::tempdir;

    #[test]
    fn reset_clears_saved_state() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        crate::cli::new::run(&repo, "test-project", "/some/path", None).unwrap();

        // Add saved state
        let mut config = repo.load("test-project").unwrap();
        config.last_state = Some(SavedState {
            captured_at: dt("2026-03-03T16:00:00Z"),
            layout_string: "some-layout".to_string(),
            panes: vec![],
        });
        repo.save(&config).unwrap();

        // Reset should clear it
        run(&repo, "test-project").unwrap();

        let reloaded = repo.load("test-project").unwrap();
        assert!(reloaded.last_state.is_none());
    }

    #[test]
    fn reset_no_saved_state_is_ok() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        crate::cli::new::run(&repo, "test-project", "/some/path", None).unwrap();

        assert!(run(&repo, "test-project").is_ok());
    }
}
