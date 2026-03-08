use std::io::Write;

use anyhow::Result;

use crate::domain::claude_session::ClaudeSessionStatus;
use crate::ports::project_repository::ProjectRepository;
use crate::ports::tmux_adapter::TmuxAdapter;

use super::format::abbreviate_home;

struct ProjectRow {
    name: String,
    path: String,
    tmux: String,
    claude: String,
    last_note: String,
}

pub fn run(
    repo: &dyn ProjectRepository,
    tmux: &dyn TmuxAdapter,
    out: &mut dyn Write,
) -> Result<()> {
    let names = repo.list()?;
    if names.is_empty() {
        writeln!(out, "No projects registered.")?;
        return Ok(());
    }

    let rows: Vec<ProjectRow> = names
        .iter()
        .filter_map(|name| {
            let config = repo.load(name).ok()?;
            let alive = tmux.has_session(name);
            let active_count = config
                .claude_sessions
                .iter()
                .filter(|s| matches!(s.status, ClaudeSessionStatus::Active))
                .count();
            let last_note = config
                .notes
                .last()
                .map(|n| {
                    let content = &n.content;
                    let char_count = content.chars().count();
                    if char_count > 40 {
                        let truncated: String = content.chars().take(37).collect();
                        format!("\"{truncated}...\"")
                    } else {
                        format!("\"{content}\"")
                    }
                })
                .unwrap_or_else(|| "--".to_string());

            Some(ProjectRow {
                name: name.clone(),
                path: abbreviate_home(&config.project.path),
                tmux: if alive { "alive" } else { "dead" }.to_string(),
                claude: format!("{active_count} active"),
                last_note,
            })
        })
        .collect();

    if rows.is_empty() {
        writeln!(out, "No projects could be loaded.")?;
        return Ok(());
    }

    let w_name = rows.iter().map(|r| r.name.len()).max().unwrap_or(0).max(7);
    let w_path = rows.iter().map(|r| r.path.len()).max().unwrap_or(0).max(4);
    let w_tmux = 5; // "alive" / "dead"
    let w_claude = rows
        .iter()
        .map(|r| r.claude.len())
        .max()
        .unwrap_or(0)
        .max(6);

    writeln!(
        out,
        "{:<w_name$}   {:<w_path$}   {:<w_tmux$}   {:<w_claude$}   LAST NOTE",
        "PROJECT", "PATH", "TMUX", "CLAUDE"
    )?;

    for row in &rows {
        writeln!(
            out,
            "{:<w_name$}   {:<w_path$}   {:<w_tmux$}   {:<w_claude$}   {}",
            row.name, row.path, row.tmux, row.claude, row.last_note
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::toml_project_repository::TomlProjectRepository;
    use crate::test_support::MockTmuxAdapter;
    use tempfile::tempdir;

    #[test]
    fn status_no_projects() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        let tmux = MockTmuxAdapter::no_session();
        let mut out = Vec::new();

        assert!(run(&repo, &tmux, &mut out).is_ok());
    }

    #[test]
    fn status_single_project_alive() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams::new("proj", "/some/path"),
        )
        .unwrap();
        crate::cli::note::run(&repo, "proj", "implement step 4").unwrap();

        let tmux = MockTmuxAdapter::with_session("", vec![]);
        let mut out = Vec::new();

        assert!(run(&repo, &tmux, &mut out).is_ok());
    }

    #[test]
    fn status_dead_session_no_notes() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams::new("proj", "/some/path"),
        )
        .unwrap();

        let tmux = MockTmuxAdapter::no_session();
        let mut out = Vec::new();

        assert!(run(&repo, &tmux, &mut out).is_ok());
    }

    #[test]
    fn status_multiple_projects() {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams::new("alpha", "/some/alpha"),
        )
        .unwrap();
        crate::cli::new::run(
            &repo,
            crate::cli::new::NewProjectParams::new("beta", "/some/beta"),
        )
        .unwrap();
        crate::cli::note::run(&repo, "alpha", "note for alpha").unwrap();

        let tmux = MockTmuxAdapter::no_session();
        let mut out = Vec::new();

        assert!(run(&repo, &tmux, &mut out).is_ok());
    }
}
