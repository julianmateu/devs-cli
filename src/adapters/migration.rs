use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::adapters::config_version;
use crate::adapters::split_config;
use crate::domain::project::ProjectConfig;

const CURRENT_VERSION: u32 = 2;

/// Run pending migrations if the config version is behind.
/// No-op if already at the current version.
pub fn migrate_if_needed(config_dir: &Path) -> Result<()> {
    let version = config_version::read_version(config_dir)?;
    if version >= CURRENT_VERSION {
        return Ok(());
    }

    if version < 2 {
        migrate_v1_to_v2(config_dir)?;
    }

    Ok(())
}

fn migrate_v1_to_v2(config_dir: &Path) -> Result<()> {
    let projects_dir = config_dir.join("projects");
    if !projects_dir.exists() {
        // Nothing to migrate — just write the version and gitignore
        config_version::write_version(config_dir, CURRENT_VERSION)?;
        write_gitignore(config_dir)?;
        return Ok(());
    }

    // Back up projects/ before modifying
    let backup_dir = config_dir.join("backup-v1");
    if !backup_dir.exists() {
        copy_dir(&projects_dir, &backup_dir)?;
        eprintln!("devs: backed up projects/ to {}", backup_dir.display());
    }

    // Migrate each project file
    let entries: Vec<_> = fs::read_dir(&projects_dir)
        .context("failed to read projects directory")?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "toml"))
        .collect();

    for entry in entries {
        let path = entry.path();
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();

        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let mut config: ProjectConfig = toml::from_str(&content)
            .with_context(|| format!("failed to parse {}", path.display()))?;

        // Abbreviate path
        let home_dir = dirs::home_dir().map(|p| p.to_string_lossy().into_owned());
        config.project.path =
            crate::domain::path::abbreviate_home(&config.project.path, home_dir.as_deref());

        // Split into portable + local
        let (portable, local) = split_config::split(&config);

        // Write local file if non-empty and not already present (crash-safe:
        // if a previous run wrote the local file but crashed before updating the
        // version, we don't overwrite the good local file with potentially
        // stale data from the now-stripped portable file)
        if !local.is_empty() {
            let local_path = config_dir.join("local").join(format!("{name}.toml"));
            if !local_path.exists() {
                fs::create_dir_all(local_path.parent().expect("config path has parent"))?;
                let local_content =
                    toml::to_string(&local).context("failed to serialize local config")?;
                fs::write(&local_path, &local_content)
                    .with_context(|| format!("failed to write {}", local_path.display()))?;
                eprintln!("devs: migrated local data for '{name}'");
            }
        }

        // Re-write portable file
        let portable_content =
            toml::to_string(&portable).context("failed to serialize portable config")?;
        fs::write(&path, &portable_content)
            .with_context(|| format!("failed to write {}", path.display()))?;

        eprintln!("devs: migrated '{name}' to v2 format");
    }

    // Write .gitignore
    write_gitignore(config_dir)?;

    // Write version
    config_version::write_version(config_dir, CURRENT_VERSION)?;

    eprintln!("devs: migration to v2 complete");
    Ok(())
}

fn write_gitignore(config_dir: &Path) -> Result<()> {
    let gitignore_path = config_dir.join(".gitignore");
    if !gitignore_path.exists() {
        fs::write(&gitignore_path, "local/\nbackup-v1/\n").context("failed to write .gitignore")?;
    }
    Ok(())
}

fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src).context("failed to read source directory")? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::config_version;
    use tempfile::tempdir;

    fn write_v1_project(config_dir: &Path, name: &str, content: &str) {
        let projects_dir = config_dir.join("projects");
        fs::create_dir_all(&projects_dir).unwrap();
        fs::write(projects_dir.join(format!("{name}.toml")), content).unwrap();
    }

    fn v1_full_project(name: &str, path: &str) -> String {
        format!(
            r#"[project]
name = "{name}"
path = "{path}"
created_at = "2026-01-01T00:00:00Z"

[[claude_sessions]]
id = "sess_1"
label = "main"
started_at = "2026-01-01T00:00:00Z"
status = "active"

[last_state]
captured_at = "2026-01-01T00:00:00Z"
layout_string = "5aed,176x79"

[[last_state.panes]]
index = 0
path = "{path}"
command = "nvim"
"#
        )
    }

    fn v1_minimal_project(name: &str) -> String {
        format!(
            r#"[project]
name = "{name}"
path = "/some/path"
created_at = "2026-01-01T00:00:00Z"
"#
        )
    }

    #[test]
    fn migrate_splits_v1_files() {
        let dir = tempdir().unwrap();
        let config_dir = dir.path();
        write_v1_project(
            config_dir,
            "myproj",
            &v1_full_project("myproj", "/some/path"),
        );

        migrate_if_needed(config_dir).unwrap();

        // Portable file should not contain sessions or state
        let portable = fs::read_to_string(config_dir.join("projects/myproj.toml")).unwrap();
        assert!(!portable.contains("claude_sessions"));
        assert!(!portable.contains("last_state"));
        assert!(portable.contains("name = \"myproj\""));

        // Local file should contain sessions and state
        let local = fs::read_to_string(config_dir.join("local/myproj.toml")).unwrap();
        assert!(local.contains("claude_sessions"));
        assert!(local.contains("sess_1"));
        assert!(local.contains("last_state"));
    }

    #[test]
    fn migrate_abbreviates_paths() {
        let dir = tempdir().unwrap();
        let config_dir = dir.path();
        let home = dirs::home_dir().unwrap();
        let abs_path = format!("{}/src/myproj", home.display());
        write_v1_project(config_dir, "myproj", &v1_full_project("myproj", &abs_path));

        migrate_if_needed(config_dir).unwrap();

        let portable = fs::read_to_string(config_dir.join("projects/myproj.toml")).unwrap();
        assert!(
            portable.contains("path = \"~/src/myproj\""),
            "path should be abbreviated: {portable}"
        );
    }

    #[test]
    fn migrate_creates_backup() {
        let dir = tempdir().unwrap();
        let config_dir = dir.path();
        write_v1_project(
            config_dir,
            "myproj",
            &v1_full_project("myproj", "/some/path"),
        );

        migrate_if_needed(config_dir).unwrap();

        let backup = config_dir.join("backup-v1/myproj.toml");
        assert!(backup.exists(), "backup file should exist");

        // Backup should contain original content (with sessions)
        let content = fs::read_to_string(&backup).unwrap();
        assert!(content.contains("claude_sessions"));
    }

    #[test]
    fn migrate_creates_gitignore() {
        let dir = tempdir().unwrap();
        let config_dir = dir.path();
        write_v1_project(config_dir, "myproj", &v1_minimal_project("myproj"));

        migrate_if_needed(config_dir).unwrap();

        let gitignore = fs::read_to_string(config_dir.join(".gitignore")).unwrap();
        assert!(gitignore.contains("local/"));
        assert!(gitignore.contains("backup-v1/"));
    }

    #[test]
    fn migrate_writes_version_2() {
        let dir = tempdir().unwrap();
        let config_dir = dir.path();
        write_v1_project(config_dir, "myproj", &v1_minimal_project("myproj"));

        migrate_if_needed(config_dir).unwrap();

        assert_eq!(config_version::read_version(config_dir).unwrap(), 2);
    }

    #[test]
    fn migrate_is_idempotent() {
        let dir = tempdir().unwrap();
        let config_dir = dir.path();
        write_v1_project(
            config_dir,
            "myproj",
            &v1_full_project("myproj", "/some/path"),
        );

        migrate_if_needed(config_dir).unwrap();
        let portable_after_first =
            fs::read_to_string(config_dir.join("projects/myproj.toml")).unwrap();

        // Run again — should be a no-op
        migrate_if_needed(config_dir).unwrap();
        let portable_after_second =
            fs::read_to_string(config_dir.join("projects/myproj.toml")).unwrap();

        assert_eq!(portable_after_first, portable_after_second);
    }

    #[test]
    fn migrate_handles_empty_projects_dir() {
        let dir = tempdir().unwrap();
        let config_dir = dir.path();

        // No projects dir at all
        migrate_if_needed(config_dir).unwrap();

        assert_eq!(config_version::read_version(config_dir).unwrap(), 2);
        assert!(config_dir.join(".gitignore").exists());
    }

    #[test]
    fn migrate_skips_projects_with_no_local_data() {
        let dir = tempdir().unwrap();
        let config_dir = dir.path();
        write_v1_project(config_dir, "minimal", &v1_minimal_project("minimal"));

        migrate_if_needed(config_dir).unwrap();

        assert!(
            !config_dir.join("local/minimal.toml").exists(),
            "should not create local file for projects without sessions/state"
        );
    }

    #[test]
    fn migrate_preserves_non_home_paths() {
        let dir = tempdir().unwrap();
        let config_dir = dir.path();
        write_v1_project(
            config_dir,
            "ext",
            &v1_minimal_project("ext").replace("/some/path", "/Volumes/ext/proj"),
        );

        migrate_if_needed(config_dir).unwrap();

        let portable = fs::read_to_string(config_dir.join("projects/ext.toml")).unwrap();
        assert!(
            portable.contains("path = \"/Volumes/ext/proj\""),
            "non-home paths should stay absolute: {portable}"
        );
    }
}
