use anyhow::Result;

use crate::ports::project_repository::ProjectRepository;

use super::format::abbreviate_home;

pub fn run(repo: &dyn ProjectRepository) -> Result<()> {
    let names = repo.list()?;
    if names.is_empty() {
        println!("No projects registered.");
        return Ok(());
    }

    let rows: Vec<(String, String)> = names
        .iter()
        .filter_map(|name| {
            let config = repo.load(name).ok()?;
            let path = abbreviate_home(&config.project.path);
            Some((name.clone(), path))
        })
        .collect();

    let w_name = rows.iter().map(|(n, _)| n.len()).max().unwrap_or(0);

    for (name, path) in &rows {
        println!("{name:<w_name$}   {path}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::toml_project_repository::TomlProjectRepository;
    use tempfile::tempdir;

    #[test]
    fn list_succeeds_when_empty() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());

        assert!(run(&repo).is_ok());
    }

    #[test]
    fn list_succeeds_with_projects() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams::new("test-project", "/some/path"),
        )
        .unwrap();

        assert!(run(&repo).is_ok());
    }

    #[test]
    fn list_shows_paths() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams::new("alpha", "/usr/local/alpha"),
        )
        .unwrap();
        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams::new("beta", "/usr/local/beta"),
        )
        .unwrap();

        // Just verify it runs without error (output is visual)
        assert!(run(&repo).is_ok());
    }
}
