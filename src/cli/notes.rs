use std::io::Write;

use anyhow::Result;

use crate::domain::duration::parse_duration;
use crate::ports::project_repository::ProjectRepository;

pub fn run(
    repo: &dyn ProjectRepository,
    name: &str,
    all: bool,
    since: Option<&str>,
    clear: bool,
    force: bool,
    out: &mut dyn Write,
) -> Result<()> {
    let mut config = repo.load(name)?;

    if clear {
        let count = config.notes.len();
        if count > 0 && !force {
            anyhow::bail!("deleting {count} notes is destructive. Use --clear --force to confirm.");
        }
        config.notes.clear();
        repo.save(&config)?;
        writeln!(out, "Cleared {count} notes for '{name}'.")?;
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
        writeln!(out, "No notes for '{name}'.")?;
        return Ok(());
    }

    for note in &notes {
        let ts = note.created_at.format("%Y-%m-%d %H:%M");
        writeln!(out, "[{ts}] {}", note.content)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::note::Note;
    use crate::domain::test_helpers::dt;
    use crate::ports::project_repository::ProjectRepository;
    use crate::test_support::InMemoryProjectRepository;

    fn output_string(out: &[u8]) -> String {
        String::from_utf8(out.to_vec()).unwrap()
    }

    fn add_notes(repo: &dyn ProjectRepository, name: &str, notes: Vec<Note>) {
        let mut config = repo.load(name).unwrap();
        config.notes = notes;
        repo.save(&config).unwrap();
    }

    fn create_project(repo: &InMemoryProjectRepository) {
        crate::cli::new::run(
            repo,
            crate::cli::new::NewProjectParams::new("myproject", "/some/path"),
        )
        .unwrap();
    }

    #[test]
    fn notes_default_limit_is_20() {
        let repo = InMemoryProjectRepository::new();
        create_project(&repo);

        let notes: Vec<Note> = (0..25)
            .map(|i| Note {
                content: format!("note {i}"),
                created_at: dt(&format!("2026-03-01T{:02}:00:00Z", i % 24)),
            })
            .collect();
        add_notes(&repo, "myproject", notes);
        let mut out = Vec::new();

        run(&repo, "myproject", false, None, false, false, &mut out).unwrap();

        let text = output_string(&out);
        let line_count = text.lines().count();
        assert_eq!(line_count, 20, "should show exactly 20 notes");
        assert!(!text.contains("note 0"), "first notes should be skipped");
        assert!(!text.contains("note 4"), "early notes should be skipped");
        assert!(
            text.contains("note 5"),
            "note 5 should appear (25-20=5 skipped)"
        );
        assert!(text.contains("note 24"), "last note should appear");
    }

    #[test]
    fn notes_all_flag_shows_everything() {
        let repo = InMemoryProjectRepository::new();
        create_project(&repo);

        let notes: Vec<Note> = (0..25)
            .map(|i| Note {
                content: format!("note {i}"),
                created_at: dt(&format!("2026-03-01T{:02}:00:00Z", i % 24)),
            })
            .collect();
        add_notes(&repo, "myproject", notes);
        let mut out = Vec::new();

        run(&repo, "myproject", true, None, false, false, &mut out).unwrap();

        let text = output_string(&out);
        assert_eq!(text.lines().count(), 25, "should show all 25 notes");
        assert!(text.contains("note 0"), "first note should appear");
    }

    #[test]
    fn notes_since_filters_by_time() {
        let repo = InMemoryProjectRepository::new();
        create_project(&repo);

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
        let mut out = Vec::new();

        run(
            &repo,
            "myproject",
            false,
            Some("2d"),
            false,
            false,
            &mut out,
        )
        .unwrap();

        let text = output_string(&out);
        assert!(text.contains("recent note"), "should show recent note");
        assert!(!text.contains("old note"), "should filter out old note");
    }

    #[test]
    fn notes_since_rejects_invalid_duration() {
        let repo = InMemoryProjectRepository::new();
        create_project(&repo);
        let mut out = Vec::new();

        let result = run(
            &repo,
            "myproject",
            false,
            Some("bad"),
            false,
            false,
            &mut out,
        );
        assert!(result.is_err());
    }

    #[test]
    fn notes_clear_removes_all_notes() {
        let repo = InMemoryProjectRepository::new();
        create_project(&repo);

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
        let mut out = Vec::new();

        run(&repo, "myproject", false, None, true, true, &mut out).unwrap();

        let config = repo.load("myproject").unwrap();
        assert!(config.notes.is_empty());
        let text = output_string(&out);
        assert!(text.contains("Cleared 2 notes"), "should report count");
    }

    #[test]
    fn notes_empty_project_shows_no_notes_message() {
        let repo = InMemoryProjectRepository::new();
        create_project(&repo);
        let mut out = Vec::new();

        run(&repo, "myproject", false, None, false, false, &mut out).unwrap();

        assert_eq!(output_string(&out), "No notes for 'myproject'.\n");
    }

    #[test]
    fn notes_clear_without_force_fails() {
        let repo = InMemoryProjectRepository::new();
        create_project(&repo);

        add_notes(
            &repo,
            "myproject",
            vec![Note {
                content: "important".to_string(),
                created_at: dt("2026-03-03T10:00:00Z"),
            }],
        );
        let mut out = Vec::new();

        let result = run(&repo, "myproject", false, None, true, false, &mut out);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("--force"), "error should mention --force flag");

        let config = repo.load("myproject").unwrap();
        assert_eq!(config.notes.len(), 1, "notes should not be deleted");
    }

    #[test]
    fn notes_clear_empty_without_force_succeeds() {
        let repo = InMemoryProjectRepository::new();
        create_project(&repo);
        let mut out = Vec::new();

        run(&repo, "myproject", false, None, true, false, &mut out).unwrap();

        let text = output_string(&out);
        assert!(text.contains("Cleared 0 notes"), "should succeed for empty");
    }

    #[test]
    fn notes_fails_for_missing_project() {
        let repo = InMemoryProjectRepository::new();
        let mut out = Vec::new();

        let result = run(&repo, "nonexistent", false, None, false, false, &mut out);
        assert!(result.is_err());
    }
}
