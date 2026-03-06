use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

#[test]
fn version_flag_prints_version() {
    Command::cargo_bin("devs")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn from_session_conflicts_with_from() {
    Command::cargo_bin("devs")
        .unwrap()
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
    Command::cargo_bin("devs")
        .unwrap()
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("devs"))
        .stderr(predicate::str::contains("setup instructions"));
}

#[test]
fn completions_generates_bash_output() {
    Command::cargo_bin("devs")
        .unwrap()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("devs"))
        .stderr(predicate::str::contains("setup instructions"));
}

#[test]
fn generate_man_creates_files() {
    let dir = tempfile::tempdir().unwrap();
    Command::cargo_bin("devs")
        .unwrap()
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
    Command::cargo_bin("devs")
        .unwrap()
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

    Command::cargo_bin("devs")
        .unwrap()
        .args(["generate-man", nested.to_str().unwrap()])
        .assert()
        .success();

    assert!(nested.join("devs.1").exists());
}

#[test]
fn tmux_help_prints_reference() {
    Command::cargo_bin("devs")
        .unwrap()
        .args(["tmux-help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Panes"))
        .stdout(predicate::str::contains("Windows"))
        .stdout(predicate::str::contains("Sessions"))
        .stdout(predicate::str::contains("Copy mode"));
}

#[test]
fn docs_mention_all_subcommands() {
    let output = Command::cargo_bin("devs")
        .unwrap()
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

    let doc_files: Vec<(&str, String)> = vec![
        (
            "README.md",
            fs::read_to_string(Path::new("README.md")).expect("Failed to read README.md"),
        ),
        (
            "docs/cli-commands.md",
            fs::read_to_string(Path::new("docs/cli-commands.md"))
                .expect("Failed to read docs/cli-commands.md"),
        ),
    ];

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
