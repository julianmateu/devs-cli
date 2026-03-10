use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

fn devs_cmd() -> assert_cmd::Command {
    cargo_bin_cmd!("devs")
}

#[test]
fn version_flag_prints_version() {
    let version = env!("CARGO_PKG_VERSION");
    devs_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(version));
}

#[test]
fn from_session_conflicts_with_from() {
    devs_cmd()
        .args([
            "new",
            "test",
            "--path",
            "/tmp/test",
            "--from",
            "source",
            "--from-session",
            "live",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn completions_generates_output() {
    devs_cmd()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("devs"))
        .stderr(predicate::str::contains("setup instructions"));
}

#[test]
fn completions_generates_bash_output() {
    devs_cmd()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("devs"))
        .stderr(predicate::str::contains("setup instructions"));
}

#[test]
fn generate_man_creates_files() {
    let dir = tempfile::tempdir().unwrap();
    devs_cmd()
        .args(["generate-man", dir.path().to_str().unwrap()])
        .assert()
        .success()
        .stderr(predicate::str::contains("Man pages generated"));

    assert!(dir.path().join("devs.1").exists());
    assert!(dir.path().join("devs-open.1").exists());
}

#[test]
fn generate_man_produces_valid_roff() {
    let dir = tempfile::tempdir().unwrap();
    devs_cmd()
        .args(["generate-man", dir.path().to_str().unwrap()])
        .assert()
        .success();

    let content = fs::read_to_string(dir.path().join("devs.1")).unwrap();
    assert!(content.contains(".TH"));
    assert!(content.contains("devs"));
}

#[test]
fn generate_man_creates_output_directory() {
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("a").join("b").join("man1");

    devs_cmd()
        .args(["generate-man", nested.to_str().unwrap()])
        .assert()
        .success();

    assert!(nested.join("devs.1").exists());
}

#[test]
fn tmux_help_prints_reference() {
    devs_cmd()
        .args(["tmux-help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Panes"))
        .stdout(predicate::str::contains("Windows"))
        .stdout(predicate::str::contains("Sessions"))
        .stdout(predicate::str::contains("Copy mode"));
}

#[test]
fn new_path_is_optional_in_help() {
    // --path should be listed under [OPTIONS], not as a required argument
    devs_cmd()
        .args(["new", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[OPTIONS]"))
        .stdout(predicate::str::contains("--path"))
        .stdout(predicate::str::contains("defaults to current directory"));
}

#[test]
fn init_without_name_outside_project_shows_error() {
    devs_cmd()
        .args(["init"])
        .current_dir(std::env::temp_dir())
        .assert()
        .failure()
        .stderr(predicate::str::contains("no project found"));
}

#[test]
fn init_help_shows_description() {
    devs_cmd()
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(".devs.toml"));
}

/// Set up a temp home with a registered project for CWD inference tests.
/// Returns (home_dir, project_dir) — both are TempDirs that must be kept alive.
fn setup_project_home() -> (tempfile::TempDir, tempfile::TempDir) {
    let home = tempfile::tempdir().unwrap();
    let project_dir = tempfile::tempdir().unwrap();

    let config_dir = home.path().join(".config/devs");
    fs::create_dir_all(config_dir.join("projects")).unwrap();
    fs::create_dir_all(config_dir.join("local")).unwrap();

    // Write version 2 to skip migration
    fs::write(config_dir.join("config.toml"), "version = 2\n").unwrap();

    // Write a project config pointing to the temp project dir.
    // Use canonicalize to resolve macOS /private symlinks.
    let canonical_project = project_dir.path().canonicalize().unwrap();
    let project_toml = format!(
        "[project]\nname = \"test-proj\"\npath = \"{}\"\ncreated_at = \"2026-01-01T00:00:00Z\"\n",
        canonical_project.display()
    );
    fs::write(config_dir.join("projects/test-proj.toml"), project_toml).unwrap();

    (home, project_dir)
}

#[test]
fn config_infers_project_from_cwd() {
    let (home, project_dir) = setup_project_home();

    devs_cmd()
        .args(["config"])
        .env("HOME", home.path())
        .current_dir(project_dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("test-proj"));
}

#[test]
fn config_infers_project_from_subdirectory() {
    let (home, project_dir) = setup_project_home();
    let subdir = project_dir.path().join("src");
    fs::create_dir(&subdir).unwrap();

    devs_cmd()
        .args(["config"])
        .env("HOME", home.path())
        .current_dir(&subdir)
        .assert()
        .success()
        .stdout(predicate::str::contains("test-proj"));
}

#[test]
fn config_with_explicit_name_ignores_cwd() {
    let (home, _project_dir) = setup_project_home();

    devs_cmd()
        .args(["config", "test-proj"])
        .env("HOME", home.path())
        .current_dir(std::env::temp_dir())
        .assert()
        .success()
        .stdout(predicate::str::contains("test-proj"));
}

#[test]
fn docs_mention_all_subcommands() {
    let output = devs_cmd()
        .arg("--help")
        .output()
        .expect("failed to run devs --help");

    let help_text = String::from_utf8(output.stdout).unwrap();

    // Parse subcommand names from help output: lines like "  open   Description..."
    let subcommands: Vec<&str> = help_text
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            // Subcommand lines are indented and start with a word
            if line.starts_with("  ") && !line.starts_with("    ") && !trimmed.is_empty() {
                trimmed.split_whitespace().next()
            } else {
                None
            }
        })
        // Skip clap built-ins and flags
        .filter(|name| *name != "help" && !name.starts_with('-'))
        .collect();

    assert!(
        !subcommands.is_empty(),
        "Failed to parse any subcommands from devs --help"
    );

    let doc_files: Vec<(&str, String)> = vec![(
        "README.md",
        fs::read_to_string(Path::new("README.md")).expect("Failed to read README.md"),
    )];

    let mut missing: Vec<String> = Vec::new();

    for cmd in &subcommands {
        let pattern = format!("devs {cmd}");
        for (filename, content) in &doc_files {
            if !content.contains(&pattern) {
                missing.push(format!("`{pattern}` not found in {filename}"));
            }
        }
    }

    assert!(
        missing.is_empty(),
        "Documentation is out of sync with CLI subcommands:\n  - {}",
        missing.join("\n  - ")
    );
}
