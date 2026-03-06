use anyhow::Result;

use crate::cli::format::expand_home;
use crate::domain::local_config::LocalConfig;
use crate::ports::local_config::LocalConfigWriter;
use crate::ports::project_repository::ProjectRepository;

pub fn run(repo: &dyn ProjectRepository, writer: &dyn LocalConfigWriter, name: &str) -> Result<()> {
    let config = repo.load(name)?;
    let expanded_path = expand_home(&config.project.path);

    let local_config = LocalConfig {
        color: config.project.color,
        layout: config.layout.map(|mut l| {
            l.layout_string = None;
            l
        }),
    };

    writer.write(&expanded_path, &local_config)?;

    println!("Wrote .devs.toml to {expanded_path}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::toml_local_config::TomlLocalConfig;
    use crate::adapters::toml_project_repository::TomlProjectRepository;
    use crate::domain::layout::{Layout, MainPane, SplitDirection, SplitPane};
    use crate::ports::local_config::LocalConfigReader;
    use tempfile::tempdir;

    fn setup() -> (TomlProjectRepository, tempfile::TempDir, tempfile::TempDir) {
        let config_dir = tempdir().unwrap();
        let project_dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(config_dir.path().to_path_buf());
        (repo, config_dir, project_dir)
    }

    #[test]
    fn exports_color_and_layout() {
        let (repo, _config_dir, project_dir) = setup();
        let project_path = project_dir.path().to_str().unwrap();

        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams {
                color: Some("#e06c75"),
                ..crate::cli::new::NewProjectParams::new("myproj", project_path)
            },
        )
        .unwrap();

        // Add a layout
        let mut config = repo.load("myproj").unwrap();
        config.layout = Some(Layout {
            main: MainPane {
                cmd: Some("nvim".to_string()),
            },
            panes: vec![SplitPane {
                split: SplitDirection::Right,
                cmd: Some("claude".to_string()),
                size: Some("40%".to_string()),
            }],
            layout_string: None,
        });
        repo.save(&config).unwrap();

        let writer = TomlLocalConfig;
        run(&repo, &writer, "myproj").unwrap();

        let reader = TomlLocalConfig;
        let local_config = reader.read(project_path).unwrap().expect("should exist");
        assert_eq!(local_config.color, Some("#e06c75".to_string()));
        let layout = local_config.layout.expect("should have layout");
        assert_eq!(layout.main.cmd, Some("nvim".to_string()));
        assert_eq!(layout.panes.len(), 1);
    }

    #[test]
    fn exports_with_no_color() {
        let (repo, _config_dir, project_dir) = setup();
        let project_path = project_dir.path().to_str().unwrap();

        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams::new("myproj", project_path),
        )
        .unwrap();

        let writer = TomlLocalConfig;
        run(&repo, &writer, "myproj").unwrap();

        let reader = TomlLocalConfig;
        let local_config = reader.read(project_path).unwrap().expect("should exist");
        assert!(local_config.color.is_none());
    }

    #[test]
    fn exports_with_no_layout() {
        let (repo, _config_dir, project_dir) = setup();
        let project_path = project_dir.path().to_str().unwrap();

        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams {
                color: Some("#aabbcc"),
                ..crate::cli::new::NewProjectParams::new("myproj", project_path)
            },
        )
        .unwrap();

        let writer = TomlLocalConfig;
        run(&repo, &writer, "myproj").unwrap();

        let reader = TomlLocalConfig;
        let local_config = reader.read(project_path).unwrap().expect("should exist");
        assert!(local_config.layout.is_none());
        assert_eq!(local_config.color, Some("#aabbcc".to_string()));
    }

    #[test]
    fn fails_for_missing_project() {
        let (repo, _config_dir, _project_dir) = setup();
        let writer = TomlLocalConfig;

        let result = run(&repo, &writer, "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn written_file_roundtrips() {
        let (repo, _config_dir, project_dir) = setup();
        let project_path = project_dir.path().to_str().unwrap();

        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams {
                color: Some("#61afef"),
                ..crate::cli::new::NewProjectParams::new("myproj", project_path)
            },
        )
        .unwrap();

        let mut config = repo.load("myproj").unwrap();
        config.layout = Some(Layout {
            main: MainPane {
                cmd: Some("nvim".to_string()),
            },
            panes: vec![
                SplitPane {
                    split: SplitDirection::Right,
                    cmd: Some("claude".to_string()),
                    size: Some("40%".to_string()),
                },
                SplitPane {
                    split: SplitDirection::BottomRight,
                    cmd: None,
                    size: None,
                },
            ],
            layout_string: None,
        });
        repo.save(&config).unwrap();

        let adapter = TomlLocalConfig;
        run(&repo, &adapter, "myproj").unwrap();

        // Read back and verify it matches what we expect
        let local_config = adapter.read(project_path).unwrap().expect("should exist");
        assert_eq!(local_config.color, Some("#61afef".to_string()));
        assert_eq!(local_config.layout, config.layout);
    }

    #[test]
    fn strips_layout_string_from_export() {
        let (repo, _config_dir, project_dir) = setup();
        let project_path = project_dir.path().to_str().unwrap();

        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams::new("myproj", project_path),
        )
        .unwrap();

        let mut config = repo.load("myproj").unwrap();
        config.layout = Some(Layout {
            main: MainPane {
                cmd: Some("nvim".to_string()),
            },
            panes: vec![],
            layout_string: Some("5aed,176x79,0,0".to_string()),
        });
        repo.save(&config).unwrap();

        let adapter = TomlLocalConfig;
        run(&repo, &adapter, "myproj").unwrap();

        let local_config = adapter.read(project_path).unwrap().expect("should exist");
        let layout = local_config.layout.expect("should have layout");
        assert!(
            layout.layout_string.is_none(),
            "layout_string should be stripped from export"
        );
        assert_eq!(layout.main.cmd, Some("nvim".to_string()));
    }
}
