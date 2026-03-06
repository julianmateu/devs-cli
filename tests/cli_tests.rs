use assert_cmd::Command;
use predicates::prelude::*;

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
