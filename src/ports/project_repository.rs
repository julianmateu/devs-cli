use anyhow::Result;

use crate::domain::project::ProjectConfig;

pub trait ProjectRepository {
    fn load(&self, name: &str) -> Result<ProjectConfig>;
    fn save(&self, config: &ProjectConfig) -> Result<()>;
    fn list(&self) -> Result<Vec<String>>;
    fn delete(&self, name: &str) -> Result<()>;
}
