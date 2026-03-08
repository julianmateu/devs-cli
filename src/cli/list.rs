use std::io::Write;

use anyhow::Result;

use crate::ports::project_repository::ProjectRepository;

use super::format::abbreviate_home;

pub fn run(repo: &dyn ProjectRepository, out: &mut dyn Write) -> Result<()> {
    let names = repo.list()?;
    if names.is_empty() {
        writeln!(out, "No projects registered.")?;
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
        writeln!(out, "{name:<w_name$}   {path}")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::InMemoryProjectRepository;

    fn output_string(out: &[u8]) -> String {
        String::from_utf8(out.to_vec()).unwrap()
    }

    #[test]
    fn list_empty_shows_message() {
        let repo = InMemoryProjectRepository::new();
        let mut out = Vec::new();

        run(&repo, &mut out).unwrap();

        assert_eq!(output_string(&out), "No projects registered.\n");
    }

    #[test]
    fn list_shows_project_names_and_paths() {
        let repo = InMemoryProjectRepository::new();
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
        let mut out = Vec::new();

        run(&repo, &mut out).unwrap();

        let text = output_string(&out);
        assert!(
            text.contains("alpha"),
            "should contain project name 'alpha'"
        );
        assert!(text.contains("beta"), "should contain project name 'beta'");
        assert!(
            text.contains("/usr/local/alpha"),
            "should contain path for alpha"
        );
        assert!(
            text.contains("/usr/local/beta"),
            "should contain path for beta"
        );
    }

    #[test]
    fn list_column_aligned() {
        let repo = InMemoryProjectRepository::new();
        crate::cli::new::run(&repo, crate::cli::new::NewProjectParams::new("short", "/a")).unwrap();
        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams::new("longername", "/b"),
        )
        .unwrap();
        let mut out = Vec::new();

        run(&repo, &mut out).unwrap();

        let text = output_string(&out);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2);
        // Both lines should have the same column offset for the path
        let path_offset_0 = lines[0].find("/b").unwrap();
        let path_offset_1 = lines[1].find("/a").unwrap();
        assert_eq!(path_offset_0, path_offset_1, "paths should be aligned");
    }
}
