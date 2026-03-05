use anyhow::Result;

use crate::domain::duration::parse_duration;
use crate::ports::project_repository::ProjectRepository;

pub fn run(
    repo: &dyn ProjectRepository,
    name: &str,
    all: bool,
    since: Option<&str>,
    clear: bool,
) -> Result<()> {
    let mut config = repo.load(name)?;

    if clear {
        let count = config.notes.len();
        config.notes.clear();
        repo.save(&config)?;
        println!("Cleared {count} notes for '{name}'.");
        return Ok(());
    }

    let now = chrono::Utc::now();
    let notes = if let Some(since_str) = since {
        let delta = parse_duration(since_str)?;
        let cutoff = now - delta;
        config
            .notes
            .iter()
            .filter(|n| n.created_at >= cutoff)
            .collect::<Vec<_>>()
    } else if all {
        config.notes.iter().collect()
    } else {
        let skip = config.notes.len().saturating_sub(20);
        config.notes.iter().skip(skip).collect()
    };

    if notes.is_empty() {
        println!("No notes for '{name}'.");
        return Ok(());
    }

    for note in &notes {
        let ts = note.created_at.format("%Y-%m-%d %H:%M");
        println!("[{ts}] {}", note.content);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::toml_project_repository::TomlProjectRepository;
    use crate::domain::note::Note;
    use crate::domain::test_helpers::dt;
    use tempfile::tempdir;

    fn test_repo() -> (TomlProjectRepository, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let repo = TomlProjectRepository::new(dir.path().to_path_buf());
        (repo, dir)
    }

    fn add_notes(repo: &TomlProjectRepository, name: &str, notes: Vec<Note>) {
        let mut config = repo.load(name).unwrap();
        config.notes = notes;
        repo.save(&config).unwrap();
    }

    #[test]
    fn notes_shows_last_20_by_default() {
        let (repo, _dir) = test_repo();
        crate::cli::new::run(&repo, "myproject", "/some/path", None, None, &[]).unwrap();

        let notes: Vec<Note> = (0..25)
            .map(|i| Note {
                content: format!("note {i}"),
                created_at: dt(&format!("2026-03-01T{:02}:00:00Z", i % 24)),
            })
            .collect();
        add_notes(&repo, "myproject", notes);

        run(&repo, "myproject", false, None, false).unwrap();
    }

    #[test]
    fn notes_all_flag_shows_everything() {
        let (repo, _dir) = test_repo();
        crate::cli::new::run(&repo, "myproject", "/some/path", None, None, &[]).unwrap();

        let notes: Vec<Note> = (0..25)
            .map(|i| Note {
                content: format!("note {i}"),
                created_at: dt(&format!("2026-03-01T{:02}:00:00Z", i % 24)),
            })
            .collect();
        add_notes(&repo, "myproject", notes);

        run(&repo, "myproject", true, None, false).unwrap();
    }

    #[test]
    fn notes_since_filters_by_time() {
        let (repo, _dir) = test_repo();
        crate::cli::new::run(&repo, "myproject", "/some/path", None, None, &[]).unwrap();

        let now = chrono::Utc::now();
        let old = now - chrono::TimeDelta::days(5);
        let recent = now - chrono::TimeDelta::hours(1);

        add_notes(
            &repo,
            "myproject",
            vec![
                Note {
                    content: "old note".to_string(),
                    created_at: old,
                },
                Note {
                    content: "recent note".to_string(),
                    created_at: recent,
                },
            ],
        );

        run(&repo, "myproject", false, Some("2d"), false).unwrap();
    }

    #[test]
    fn notes_since_rejects_invalid_duration() {
        let (repo, _dir) = test_repo();
        crate::cli::new::run(&repo, "myproject", "/some/path", None, None, &[]).unwrap();

        let result = run(&repo, "myproject", false, Some("bad"), false);
        assert!(result.is_err());
    }

    #[test]
    fn notes_clear_removes_all_notes() {
        let (repo, _dir) = test_repo();
        crate::cli::new::run(&repo, "myproject", "/some/path", None, None, &[]).unwrap();

        add_notes(
            &repo,
            "myproject",
            vec![
                Note {
                    content: "first".to_string(),
                    created_at: dt("2026-03-03T10:00:00Z"),
                },
                Note {
                    content: "second".to_string(),
                    created_at: dt("2026-03-03T11:00:00Z"),
                },
            ],
        );

        run(&repo, "myproject", false, None, true).unwrap();

        let config = repo.load("myproject").unwrap();
        assert!(config.notes.is_empty());
    }

    #[test]
    fn notes_clear_on_empty_notes() {
        let (repo, _dir) = test_repo();
        crate::cli::new::run(&repo, "myproject", "/some/path", None, None, &[]).unwrap();

        run(&repo, "myproject", false, None, true).unwrap();

        let config = repo.load("myproject").unwrap();
        assert!(config.notes.is_empty());
    }

    #[test]
    fn notes_empty_project_shows_message() {
        let (repo, _dir) = test_repo();
        crate::cli::new::run(&repo, "myproject", "/some/path", None, None, &[]).unwrap();

        run(&repo, "myproject", false, None, false).unwrap();
    }

    #[test]
    fn notes_fails_for_missing_project() {
        let (repo, _dir) = test_repo();

        let result = run(&repo, "nonexistent", false, None, false);
        assert!(result.is_err());
    }

    #[test]
    fn notes_default_limit_is_20() {
        let (repo, _dir) = test_repo();
        crate::cli::new::run(&repo, "myproject", "/some/path", None, None, &[]).unwrap();

        let notes: Vec<Note> = (0..25)
            .map(|i| Note {
                content: format!("note {i}"),
                created_at: dt("2026-03-03T10:00:00Z"),
            })
            .collect();
        add_notes(&repo, "myproject", notes);

        run(&repo, "myproject", false, None, false).unwrap();
    }
}
