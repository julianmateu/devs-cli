use anyhow::{Result, bail};

use crate::ports::project_repository::ProjectRepository;

pub fn run(repo: &dyn ProjectRepository, name: &str, force: bool) -> Result<()> {
    if !force {
        bail!("removing project '{name}' will delete its config. Use --force to confirm.");
    }
    repo.delete(name)?;
    println!("Removed project '{name}'.");
    Ok(())
}
