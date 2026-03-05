use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::domain::project::ProjectConfig;
use crate::ports::project_repository::ProjectRepository;

pub struct TomlProjectRepository {
    config_dir: PathBuf,
}

impl TomlProjectRepository {
    pub fn new(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }

    fn project_path(&self, name: &str) -> PathBuf {
        self.config_dir
            .join("projects")
            .join(format!("{name}.toml"))
    }
}

impl ProjectRepository for TomlProjectRepository {
    fn save(&self, config: &ProjectConfig) -> Result<()> {
        let path = self.project_path(&config.project.name);
        fs::create_dir_all(path.parent().unwrap())?;
        let content = toml::to_string(config).context("failed to serialize project config")?;
        fs::write(&path, content).with_context(|| format!("failed to write {}", path.display()))?;
        Ok(())
    }

    fn load(&self, name: &str) -> Result<ProjectConfig> {
        let path = self.project_path(name);
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read project '{name}'"))?;
        let config = toml::from_str(&content)
            .with_context(|| format!("failed to parse config for project '{name}'"))?;
        Ok(config)
    }

    fn list(&self) -> Result<Vec<String>> {
        let projects_dir = self.config_dir.join("projects");
        if !projects_dir.exists() {
            return Ok(vec![]);
        }
        let mut names: Vec<String> = fs::read_dir(&projects_dir)
            .with_context(|| {
                format!(
                    "failed to read project directory {}",
                    projects_dir.display()
                )
            })?
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if path.extension()?.to_str()? == "toml" {
                    return Some(String::from(path.file_stem()?.to_str()?));
                }
                None
            })
            .collect();
        names.sort();
        Ok(names)
    }

    fn delete(&self, name: &str) -> Result<()> {
        let path = self.project_path(name);
        fs::remove_file(&path).with_context(|| format!("failed to delete project '{name}'"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::project::ProjectMetadata;
    use crate::domain::test_helpers::dt;
    use tempfile::tempdir;

    fn sample_config(name: &str) -> ProjectConfig {
        ProjectConfig {
            project: ProjectMetadata {
                name: name.to_string(),
                path: "/some/path".to_string(),
                color: None,
                created_at: dt("2026-01-01T00:00:00Z"),
            },
            layout: None,
            claude_sessions: vec![],
            notes: vec![],
            last_state: None,
        }
    }

    #[test]
    fn save_creates_file_with_correct_content() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        let config = sample_config("test-project");

        repo.save(&config).unwrap();

        let file_path = dir.path().join("projects/test-project.toml");
        assert!(file_path.exists());

        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("name = \"test-project\""));
        assert!(content.contains("path = \"/some/path\""));
    }

    #[test]
    fn save_creates_directory_if_missing() {
        let dir = tempdir().unwrap();
        let projects_dir = dir.path().join("projects");
        assert!(!projects_dir.exists());

        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        repo.save(&sample_config("my-project")).unwrap();

        assert!(projects_dir.exists());
    }

    #[test]
    fn delete_removes_file() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        repo.save(&sample_config("doomed")).unwrap();

        let file_path = dir.path().join("projects/doomed.toml");
        assert!(file_path.exists());

        repo.delete("doomed").unwrap();
        assert!(!file_path.exists());
    }

    #[test]
    fn delete_nonexistent_returns_error() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());

        let result = repo.delete("ghost");
        assert!(result.is_err());
    }

    #[test]
    fn load_returns_saved_config() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        let config = sample_config("roundtrip");

        repo.save(&config).unwrap();
        let loaded = repo.load("roundtrip").unwrap();

        assert_eq!(loaded, config);
    }

    #[test]
    fn load_nonexistent_returns_error() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());

        let result = repo.load("ghost");
        assert!(result.is_err());
    }

    #[test]
    fn list_returns_sorted_project_names() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());

        repo.save(&sample_config("zebra")).unwrap();
        repo.save(&sample_config("alpha")).unwrap();
        repo.save(&sample_config("middle")).unwrap();

        let names = repo.list().unwrap();
        assert_eq!(names, vec!["alpha", "middle", "zebra"]);
    }

    #[test]
    fn list_empty_when_no_projects() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());

        let names = repo.list().unwrap();
        assert!(names.is_empty());
    }

    #[test]
    fn list_ignores_non_toml_files() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        repo.save(&sample_config("real-project")).unwrap();

        // Create a non-toml file that should be ignored
        let junk = dir.path().join("projects/notes.txt");
        fs::write(&junk, "not a project").unwrap();

        let names = repo.list().unwrap();
        assert_eq!(names, vec!["real-project"]);
    }
}
