use std::path::Path;

use anyhow::{Result, bail};

use crate::domain::path::expand_home;
use crate::ports::project_repository::ProjectRepository;

pub fn resolve_project_name(
    name: Option<&str>,
    cwd: &Path,
    home_dir: Option<&str>,
    repo: &dyn ProjectRepository,
) -> Result<String> {
    if let Some(n) = name {
        return Ok(n.to_string());
    }

    let names = repo.list()?;
    let mut matches: Vec<(String, usize)> = Vec::new();

    for project_name in &names {
        let config = repo.load(project_name)?;
        let expanded = expand_home(&config.project.path, home_dir);
        let canonical = Path::new(&expanded)
            .canonicalize()
            .unwrap_or_else(|_| expanded.into());
        if cwd.starts_with(&canonical) {
            matches.push((project_name.clone(), canonical.as_os_str().len()));
        }
    }

    match matches.len() {
        0 => bail!(
            "no project found for directory '{}'. Register one with 'devs new <name>'",
            cwd.display()
        ),
        1 => Ok(matches.into_iter().next().unwrap().0),
        _ => {
            let max_depth = matches.iter().map(|(_, d)| *d).max().unwrap();
            let deepest: Vec<String> = matches
                .into_iter()
                .filter(|(_, d)| *d == max_depth)
                .map(|(name, _)| name)
                .collect();

            if deepest.len() == 1 {
                Ok(deepest.into_iter().next().unwrap())
            } else {
                bail!(
                    "multiple projects match directory '{}': {}",
                    cwd.display(),
                    deepest.join(", ")
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::project::{ProjectConfig, ProjectMetadata};
    use crate::domain::test_helpers::dt;
    use crate::test_support::InMemoryProjectRepository;

    fn make_project(name: &str, path: &str) -> ProjectConfig {
        ProjectConfig {
            project: ProjectMetadata {
                name: name.to_string(),
                path: path.to_string(),
                color: None,
                created_at: dt("2026-01-01T00:00:00Z"),
            },
            layout: None,
            claude_sessions: vec![],
            notes: vec![],
            last_state: None,
        }
    }

    #[test]
    fn explicit_name_passes_through() {
        let repo = InMemoryProjectRepository::new();
        let result = resolve_project_name(Some("my-project"), Path::new("/any/where"), None, &repo);
        assert_eq!(result.unwrap(), "my-project");
    }

    #[test]
    fn cwd_exact_match_returns_project_name() {
        let repo = InMemoryProjectRepository::new();
        repo.save(&make_project("web-app", "/home/user/src/web-app"))
            .unwrap();

        let result = resolve_project_name(None, Path::new("/home/user/src/web-app"), None, &repo);
        assert_eq!(result.unwrap(), "web-app");
    }

    #[test]
    fn cwd_in_subdirectory_returns_project_name() {
        let repo = InMemoryProjectRepository::new();
        repo.save(&make_project("web-app", "/home/user/src/web-app"))
            .unwrap();

        let result = resolve_project_name(
            None,
            Path::new("/home/user/src/web-app/src/components"),
            None,
            &repo,
        );
        assert_eq!(result.unwrap(), "web-app");
    }

    #[test]
    fn no_match_returns_actionable_error() {
        let repo = InMemoryProjectRepository::new();
        repo.save(&make_project("web-app", "/home/user/src/web-app"))
            .unwrap();

        let result = resolve_project_name(None, Path::new("/home/user/other"), None, &repo);
        let err = result.unwrap_err().to_string();
        assert!(err.contains("no project found"), "got: {err}");
        assert!(err.contains("devs new"), "got: {err}");
    }

    #[test]
    fn deepest_match_wins() {
        let repo = InMemoryProjectRepository::new();
        repo.save(&make_project("parent", "/home/user/src"))
            .unwrap();
        repo.save(&make_project("child", "/home/user/src/web-app"))
            .unwrap();

        let result =
            resolve_project_name(None, Path::new("/home/user/src/web-app/src"), None, &repo);
        assert_eq!(result.unwrap(), "child");
    }

    #[test]
    fn ambiguous_same_path_returns_error() {
        let repo = InMemoryProjectRepository::new();
        repo.save(&make_project("alpha", "/home/user/src/app"))
            .unwrap();
        repo.save(&make_project("beta", "/home/user/src/app"))
            .unwrap();

        let result = resolve_project_name(None, Path::new("/home/user/src/app"), None, &repo);
        let err = result.unwrap_err().to_string();
        assert!(err.contains("multiple projects match"), "got: {err}");
        assert!(err.contains("alpha"), "got: {err}");
        assert!(err.contains("beta"), "got: {err}");
    }

    #[test]
    fn tilde_expansion_matches_absolute_cwd() {
        let repo = InMemoryProjectRepository::new();
        repo.save(&make_project("web-app", "~/src/web-app"))
            .unwrap();

        let result = resolve_project_name(
            None,
            Path::new("/Users/testuser/src/web-app"),
            Some("/Users/testuser"),
            &repo,
        );
        assert_eq!(result.unwrap(), "web-app");
    }

    #[test]
    fn empty_repo_returns_error() {
        let repo = InMemoryProjectRepository::new();

        let result = resolve_project_name(None, Path::new("/home/user/src"), None, &repo);
        let err = result.unwrap_err().to_string();
        assert!(err.contains("no project found"), "got: {err}");
    }
}
