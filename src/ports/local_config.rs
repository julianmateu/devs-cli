use anyhow::Result;

use crate::domain::local_config::LocalConfig;

pub trait LocalConfigReader {
    fn read(&self, project_path: &str) -> Result<Option<LocalConfig>>;
}

pub trait LocalConfigWriter {
    fn write(&self, project_path: &str, config: &LocalConfig) -> Result<()>;
}
