use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct GlobalConfig {
    version: u32,
}

/// Read the config version from `config.toml` in the given directory.
/// Returns 1 if the file doesn't exist (implicit v1).
pub fn read_version(config_dir: &Path) -> Result<u32> {
    let path = config_dir.join("config.toml");
    if !path.exists() {
        return Ok(1);
    }
    let content =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let config: GlobalConfig =
        toml::from_str(&content).with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(config.version)
}

/// Write the config version to `config.toml` in the given directory.
pub fn write_version(config_dir: &Path, version: u32) -> Result<()> {
    let config = GlobalConfig { version };
    let content = toml::to_string(&config).context("failed to serialize config version")?;
    let path = config_dir.join("config.toml");
    fs::create_dir_all(config_dir)?;
    fs::write(&path, content).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn read_version_missing_file_returns_1() {
        let dir = tempdir().unwrap();
        assert_eq!(read_version(dir.path()).unwrap(), 1);
    }

    #[test]
    fn read_version_valid_file() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("config.toml"), "version = 2\n").unwrap();
        assert_eq!(read_version(dir.path()).unwrap(), 2);
    }

    #[test]
    fn write_then_read_roundtrips() {
        let dir = tempdir().unwrap();
        write_version(dir.path(), 3).unwrap();
        assert_eq!(read_version(dir.path()).unwrap(), 3);
    }

    #[test]
    fn write_creates_directory() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("nested/config");
        write_version(&nested, 2).unwrap();
        assert_eq!(read_version(&nested).unwrap(), 2);
    }
}
