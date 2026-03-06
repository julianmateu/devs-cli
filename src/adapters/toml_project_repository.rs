use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::adapters::split_config::{self, MachineLocalConfig};
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

    fn local_path(&self, name: &str) -> PathBuf {
        self.config_dir.join("local").join(format!("{name}.toml"))
    }
}

impl ProjectRepository for TomlProjectRepository {
    fn save(&self, config: &ProjectConfig) -> Result<()> {
        let (portable, local) = split_config::split(config);

        // Write local file first (crash-safe: if we crash after local but before
        // portable, the next load still works — portable file has old data with
        // serde defaults for missing fields)
        if !local.is_empty() {
            let local_path = self.local_path(&config.project.name);
            fs::create_dir_all(local_path.parent().unwrap())?;
            let content = toml::to_string(&local).context("failed to serialize local config")?;
            fs::write(&local_path, content)
                .with_context(|| format!("failed to write {}", local_path.display()))?;
        } else {
            // Clean up local file if it exists but is now empty
            let local_path = self.local_path(&config.project.name);
            if local_path.exists() {
                let _ = fs::remove_file(&local_path);
            }
        }

        // Write portable file
        let project_path = self.project_path(&config.project.name);
        fs::create_dir_all(project_path.parent().unwrap())?;
        let content = toml::to_string(&portable).context("failed to serialize project config")?;
        fs::write(&project_path, content)
            .with_context(|| format!("failed to write {}", project_path.display()))?;

        Ok(())
    }

    fn load(&self, name: &str) -> Result<ProjectConfig> {
        let project_path = self.project_path(name);
        let content = fs::read_to_string(&project_path)
            .with_context(|| format!("failed to read project '{name}'"))?;

        // Deserialize as full ProjectConfig — this handles both v1 (all-in-one)
        // and v2 (portable-only) formats, since serde defaults handle missing fields.
        let mut config: ProjectConfig = toml::from_str(&content)
            .with_context(|| format!("failed to parse config for project '{name}'"))?;

        // If a local file exists, override machine-specific fields
        let local_path = self.local_path(name);
        if local_path.exists() {
            let local_content = fs::read_to_string(&local_path)
                .with_context(|| format!("failed to read local config for '{name}'"))?;
            let local: MachineLocalConfig = toml::from_str(&local_content)
                .with_context(|| format!("failed to parse local config for '{name}'"))?;
            config.claude_sessions = local.claude_sessions;
            config.last_state = local.last_state;
        }

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
        let project_path = self.project_path(name);
        fs::remove_file(&project_path)
            .with_context(|| format!("failed to delete project '{name}'"))?;

        // Also remove local file if it exists (ignore errors — it may not exist)
        let local_path = self.local_path(name);
        let _ = fs::remove_file(&local_path);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::split_config::PortableConfig;
    use crate::domain::claude_session::{ClaudeSession, ClaudeSessionStatus};
    use crate::domain::project::ProjectMetadata;
    use crate::domain::saved_state::{SavedPane, SavedState};
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

    fn config_with_local_data(name: &str) -> ProjectConfig {
        ProjectConfig {
            project: ProjectMetadata {
                name: name.to_string(),
                path: "/some/path".to_string(),
                color: Some("#e06c75".to_string()),
                created_at: dt("2026-01-01T00:00:00Z"),
            },
            layout: None,
            claude_sessions: vec![ClaudeSession {
                id: "sess_abc".to_string(),
                label: "brainstorm".to_string(),
                started_at: dt("2026-03-01T10:00:00Z"),
                status: ClaudeSessionStatus::Active,
            }],
            notes: vec![],
            last_state: Some(SavedState {
                captured_at: dt("2026-03-03T16:00:00Z"),
                layout_string: "5aed,176x79,0,0".to_string(),
                panes: vec![SavedPane {
                    index: 0,
                    path: "/some/path".to_string(),
                    command: "nvim".to_string(),
                }],
            }),
        }
    }

    // --- Existing tests (must still pass) ---

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

    // --- New split tests ---

    #[test]
    fn save_creates_both_files() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());

        repo.save(&config_with_local_data("myproj")).unwrap();

        assert!(dir.path().join("projects/myproj.toml").exists());
        assert!(dir.path().join("local/myproj.toml").exists());
    }

    #[test]
    fn save_portable_file_excludes_sessions_and_state() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());

        repo.save(&config_with_local_data("myproj")).unwrap();

        let content = fs::read_to_string(dir.path().join("projects/myproj.toml")).unwrap();
        assert!(
            !content.contains("claude_sessions"),
            "portable file should not contain claude_sessions"
        );
        assert!(
            !content.contains("last_state"),
            "portable file should not contain last_state"
        );
        assert!(content.contains("name = \"myproj\""));
    }

    #[test]
    fn save_local_file_contains_sessions_and_state() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());

        repo.save(&config_with_local_data("myproj")).unwrap();

        let content = fs::read_to_string(dir.path().join("local/myproj.toml")).unwrap();
        assert!(content.contains("claude_sessions"));
        assert!(content.contains("sess_abc"));
        assert!(content.contains("last_state"));
        assert!(content.contains("5aed,176x79,0,0"));
    }

    #[test]
    fn save_skips_local_when_empty() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());

        // sample_config has no sessions or state
        repo.save(&sample_config("minimal")).unwrap();

        assert!(dir.path().join("projects/minimal.toml").exists());
        assert!(
            !dir.path().join("local/minimal.toml").exists(),
            "should not create local file when there's no local data"
        );
    }

    #[test]
    fn save_removes_local_file_when_data_becomes_empty() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());

        // Save with local data
        repo.save(&config_with_local_data("myproj")).unwrap();
        assert!(dir.path().join("local/myproj.toml").exists());

        // Save again without local data
        repo.save(&sample_config("myproj")).unwrap();
        assert!(
            !dir.path().join("local/myproj.toml").exists(),
            "local file should be removed when local data becomes empty"
        );
    }

    #[test]
    fn load_merges_both_files() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        let config = config_with_local_data("myproj");

        repo.save(&config).unwrap();
        let loaded = repo.load("myproj").unwrap();

        assert_eq!(loaded, config);
    }

    #[test]
    fn load_without_local_returns_empty_sessions() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());

        // Write only a portable file (no local file)
        let portable = PortableConfig {
            project: ProjectMetadata {
                name: "myproj".to_string(),
                path: "/some/path".to_string(),
                color: None,
                created_at: dt("2026-01-01T00:00:00Z"),
            },
            layout: None,
            notes: vec![],
        };
        let project_path = dir.path().join("projects/myproj.toml");
        fs::create_dir_all(project_path.parent().unwrap()).unwrap();
        fs::write(&project_path, toml::to_string(&portable).unwrap()).unwrap();

        let loaded = repo.load("myproj").unwrap();
        assert!(loaded.claude_sessions.is_empty());
        assert!(loaded.last_state.is_none());
    }

    #[test]
    fn load_v1_format_still_works() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());

        // Write a v1-style all-in-one file directly
        let v1_content = r#"[project]
name = "legacy"
path = "/old/path"
created_at = "2026-01-01T00:00:00Z"

[[claude_sessions]]
id = "old_sess"
label = "old-label"
started_at = "2026-01-01T00:00:00Z"
status = "active"
"#;
        let project_path = dir.path().join("projects/legacy.toml");
        fs::create_dir_all(project_path.parent().unwrap()).unwrap();
        fs::write(&project_path, v1_content).unwrap();

        let loaded = repo.load("legacy").unwrap();
        assert_eq!(loaded.project.name, "legacy");
        assert_eq!(loaded.claude_sessions.len(), 1);
        assert_eq!(loaded.claude_sessions[0].id, "old_sess");
    }

    #[test]
    fn delete_removes_both_files() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());

        repo.save(&config_with_local_data("doomed")).unwrap();
        assert!(dir.path().join("projects/doomed.toml").exists());
        assert!(dir.path().join("local/doomed.toml").exists());

        repo.delete("doomed").unwrap();
        assert!(!dir.path().join("projects/doomed.toml").exists());
        assert!(!dir.path().join("local/doomed.toml").exists());
    }

    #[test]
    fn delete_succeeds_when_no_local_file() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());

        repo.save(&sample_config("minimal")).unwrap();
        assert!(!dir.path().join("local/minimal.toml").exists());

        // Should not error even though there's no local file
        repo.delete("minimal").unwrap();
        assert!(!dir.path().join("projects/minimal.toml").exists());
    }

    #[test]
    fn save_creates_local_directory() {
        let dir = tempdir().unwrap();
        let local_dir = dir.path().join("local");
        assert!(!local_dir.exists());

        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        repo.save(&config_with_local_data("myproj")).unwrap();

        assert!(local_dir.exists());
    }
}
