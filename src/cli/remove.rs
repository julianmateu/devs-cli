use anyhow::{Result, bail};

use crate::ports::project_repository::ProjectRepository;
use crate::ports::tmux_adapter::TmuxAdapter;

pub fn run(
    repo: &dyn ProjectRepository,
    tmux: &dyn TmuxAdapter,
    name: &str,
    force: bool,
    kill: bool,
) -> Result<()> {
    if !force {
        bail!("removing project '{name}' will delete its config. Use --force to confirm.");
    }
    if kill && tmux.has_session(name) {
        tmux.kill_session(name)?;
        println!("Killed tmux session '{name}'.");
    }
    repo.delete(name)?;
    println!("Removed project '{name}'.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::toml_project_repository::TomlProjectRepository;
    use crate::domain::test_helpers::MockTmuxAdapter;
    use tempfile::tempdir;

    #[test]
    fn remove_deletes_project_with_force() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        let tmux = MockTmuxAdapter::no_session();
        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams::new("doomed", "/some/path"),
        )
        .unwrap();

        assert!(run(&repo, &tmux, "doomed", true, false).is_ok());
        assert!(repo.load("doomed").is_err());
    }

    #[test]
    fn remove_fails_without_force() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        let tmux = MockTmuxAdapter::no_session();
        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams::new("safe", "/some/path"),
        )
        .unwrap();

        assert!(run(&repo, &tmux, "safe", false, false).is_err());
        assert!(repo.load("safe").is_ok());
    }

    #[test]
    fn remove_fails_for_missing_project() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        let tmux = MockTmuxAdapter::no_session();

        assert!(run(&repo, &tmux, "ghost", true, false).is_err());
    }

    #[test]
    fn remove_kills_tmux_session_when_alive() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        let tmux = MockTmuxAdapter::with_session("", vec![]);
        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams::new("proj", "/some/path"),
        )
        .unwrap();

        assert!(run(&repo, &tmux, "proj", true, true).is_ok());
        assert!(tmux.calls().contains(&"kill_session(proj)".to_string()));
        assert!(repo.load("proj").is_err());
    }

    #[test]
    fn remove_kill_skips_when_session_dead() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        let tmux = MockTmuxAdapter::no_session();
        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams::new("proj", "/some/path"),
        )
        .unwrap();

        assert!(run(&repo, &tmux, "proj", true, true).is_ok());
        assert!(!tmux.calls().iter().any(|c| c.starts_with("kill_session")));
        assert!(repo.load("proj").is_err());
    }

    #[test]
    fn remove_kill_without_force_fails() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        let tmux = MockTmuxAdapter::with_session("", vec![]);
        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams::new("proj", "/some/path"),
        )
        .unwrap();

        assert!(run(&repo, &tmux, "proj", false, true).is_err());
        assert!(tmux.calls().is_empty());
        assert!(repo.load("proj").is_ok());
    }
}
