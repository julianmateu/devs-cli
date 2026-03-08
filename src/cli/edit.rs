use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use crate::ports::project_repository::ProjectRepository;

pub fn run(repo: &dyn ProjectRepository, name: &str, config_dir: &Path) -> Result<()> {
    repo.load(name)?;
    let path = config_dir.join("projects").join(format!("{name}.toml"));
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .map_err(|_| {
            anyhow::anyhow!("no $EDITOR set; set EDITOR or VISUAL to your preferred text editor")
        })?;
    Command::new(&editor)
        .arg(&path)
        .status()
        .with_context(|| format!("failed to launch editor '{editor}'"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::InMemoryProjectRepository;

    #[test]
    fn edit_fails_for_missing_project() {
        let repo = InMemoryProjectRepository::new();
        let result = run(&repo, "nonexistent", Path::new("/tmp/config"));
        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("not found"),
            "error should mention project not found"
        );
    }

    #[test]
    fn edit_fails_when_no_editor_set() {
        let repo = InMemoryProjectRepository::new();
        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams::new("my-project", "/some/path"),
        )
        .unwrap();

        // SAFETY: test runs single-threaded; we save and restore env vars.
        unsafe {
            let old_visual = std::env::var("VISUAL").ok();
            let old_editor = std::env::var("EDITOR").ok();
            std::env::remove_var("VISUAL");
            std::env::remove_var("EDITOR");

            let result = run(&repo, "my-project", Path::new("/tmp/config"));

            if let Some(v) = old_visual {
                std::env::set_var("VISUAL", v);
            }
            if let Some(e) = old_editor {
                std::env::set_var("EDITOR", e);
            }

            assert!(result.is_err());
            assert!(
                result.unwrap_err().to_string().contains("EDITOR"),
                "error should mention EDITOR"
            );
        }
    }
}
