use anyhow::Result;

use crate::ports::project_repository::ProjectRepository;

pub fn run(repo: &dyn ProjectRepository) -> Result<()> {
    let names = repo.list()?;
    if names.is_empty() {
        println!("No projects registered.");
    } else {
        for name in names {
            println!("{name}");
        }
    }
    Ok(())
}
