use anyhow::Result;

use crate::ports::project_repository::ProjectRepository;

pub fn run(repo: &dyn ProjectRepository, name: &str) -> Result<()> {
    let config = repo.load(name)?;
    let toml = toml::to_string(&config)?;
    print!("{toml}");
    Ok(())
}
