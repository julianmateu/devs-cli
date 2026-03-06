use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::domain::local_config::LocalConfig;
use crate::ports::local_config::{LocalConfigReader, LocalConfigWriter};

const FILE_NAME: &str = ".devs.toml";

pub struct TomlLocalConfig;

impl LocalConfigReader for TomlLocalConfig {
    fn read(&self, project_path: &str) -> Result<Option<LocalConfig>> {
        let file_path = Path::new(project_path).join(FILE_NAME);
        if !file_path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(&file_path)
            .with_context(|| format!("failed to read {}", file_path.display()))?;
        let config: LocalConfig = toml::from_str(&content)
            .with_context(|| format!("failed to parse {}", file_path.display()))?;
        Ok(Some(config))
    }
}

impl LocalConfigWriter for TomlLocalConfig {
    fn write(&self, project_path: &str, config: &LocalConfig) -> Result<()> {
        let file_path = Path::new(project_path).join(FILE_NAME);
        let content =
            toml::to_string(config).context("failed to serialize local config to TOML")?;
        fs::write(&file_path, content)
            .with_context(|| format!("failed to write {}", file_path.display()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::layout::{Layout, MainPane, SplitDirection, SplitPane};
    use tempfile::tempdir;

    #[test]
    fn read_missing_file_returns_none() {
        let dir = tempdir().unwrap();
        let adapter = TomlLocalConfig;

        let result = adapter.read(dir.path().to_str().unwrap()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn read_valid_file_returns_config() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(FILE_NAME);
        fs::write(
            &file_path,
            r##"color = "#61afef"

[layout.main]
cmd = "nvim"
"##,
        )
        .unwrap();

        let adapter = TomlLocalConfig;
        let result = adapter.read(dir.path().to_str().unwrap()).unwrap();

        let config = result.expect("should return Some");
        assert_eq!(config.color, Some("#61afef".to_string()));
        let layout = config.layout.expect("should have layout");
        assert_eq!(layout.main.cmd, Some("nvim".to_string()));
    }

    #[test]
    fn read_invalid_toml_returns_error() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(FILE_NAME);
        fs::write(&file_path, "this is not valid toml [[[").unwrap();

        let adapter = TomlLocalConfig;
        let result = adapter.read(dir.path().to_str().unwrap());

        assert!(result.is_err());
    }

    #[test]
    fn write_creates_file() {
        let dir = tempdir().unwrap();
        let adapter = TomlLocalConfig;

        let config = LocalConfig {
            color: Some("#e06c75".to_string()),
            layout: None,
        };

        adapter
            .write(dir.path().to_str().unwrap(), &config)
            .unwrap();

        let file_path = dir.path().join(FILE_NAME);
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains(r##"color = "#e06c75""##));
    }

    #[test]
    fn write_color_only_omits_layout() {
        let dir = tempdir().unwrap();
        let adapter = TomlLocalConfig;

        let config = LocalConfig {
            color: Some("#ff0000".to_string()),
            layout: None,
        };

        adapter
            .write(dir.path().to_str().unwrap(), &config)
            .unwrap();

        let content = fs::read_to_string(dir.path().join(FILE_NAME)).unwrap();
        assert!(!content.contains("layout"));
    }

    #[test]
    fn roundtrip_write_then_read() {
        let dir = tempdir().unwrap();
        let adapter = TomlLocalConfig;

        let config = LocalConfig {
            color: Some("#61afef".to_string()),
            layout: Some(Layout {
                main: MainPane {
                    cmd: Some("nvim".to_string()),
                },
                panes: vec![SplitPane {
                    split: SplitDirection::Right,
                    cmd: Some("claude".to_string()),
                    size: Some("40%".to_string()),
                }],
                layout_string: None,
            }),
        };

        let path_str = dir.path().to_str().unwrap();
        adapter.write(path_str, &config).unwrap();
        let loaded = adapter.read(path_str).unwrap().expect("should return Some");

        assert_eq!(loaded, config);
    }
}
