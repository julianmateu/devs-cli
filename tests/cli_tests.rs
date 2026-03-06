use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

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
