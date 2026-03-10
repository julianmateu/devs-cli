# Skeptical Review: Dynamic Shell Completions

Reviewed: 2026-03-10
Commits: ee84193, a53339e, b2fa6be (HEAD~3..HEAD)

---

## Critical

None.

---

## High

### 1. `complete_command()` applies project-name completions to `devs new`

**File:** `src/main.rs`, lines 66-70

```rust
let subcmd_names: Vec<String> = cmd
    .get_subcommands()
    .filter(|s| s.get_arguments().any(|a| a.get_id() == "name"))
    .map(|s| s.get_name().to_string())
    .collect();
```

The filter matches every subcommand with an argument whose clap ID is `"name"`. The `new` subcommand has `name: String` — a positional arg for the name of a *new* project that doesn't exist yet. This causes `devs new <TAB>` to suggest existing project names, which is wrong: the user is registering a *new* name, not selecting an existing one.

Every other subcommand with `name: Option<String>` looks up an existing project, so completions are correct there. `new` is the only offender, but it is a concrete UX bug: a user pressing Tab after `devs new` will see a list of projects they already have.

**Fix:** filter the `new` subcommand out explicitly, or (better) introduce a marker that distinguishes "lookup existing" from "new name" at the arg level.

---

### 2. `dynamic_completions_includes_project_names` does not test what its name claims

**File:** `tests/cli_tests.rs`, lines 68-86

```rust
fn dynamic_completions_includes_project_names() {
    // ...sets up a home dir with a project registered as "test-proj"...
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("complete"),
        "Expected bash completion script"
    );
}
```

The test name says it verifies project names appear in completions. The assertion only checks that the word `"complete"` appears anywhere in the bash registration script. This is identical to what `dynamic_completions_outputs_shell_script` already checks. The `setup_project_home()` call is dead work — it creates a project that is never observed in the output.

Why it cannot test what it claims: `COMPLETE=bash devs` outputs a *shell registration script*, not a list of candidates. The actual candidate list is emitted when the shell calls back into `devs` at tab-press time, with additional env vars (e.g. `COMP_LINE`, `COMP_POINT` for bash, or `_CLAP_IFS`/`_CLAP_COMPLETE_INDEX` for clap internals). That invocation path is completely untested.

This is a test that passes vacuously and provides false confidence. The feature's core value proposition — that pressing Tab returns project names — has no automated test coverage.

---

## Observations (Low / Informational)

### 3. `unstable-ext` and `unstable-dynamic` are production dependencies

Both features are documented by clap/clap_complete as unstable, meaning their APIs can change in any minor release without a SemVer break. The Cargo.toml pins `clap = "4"` and `clap_complete = "4"`, which allows any `4.x` update to silently break the completion logic.

This is an accepted trade-off (the plan.md acknowledges it) but worth flagging: if clap 4.x updates these unstable APIs, the build may silently degrade rather than fail to compile. A more defensive approach would be to pin the exact minor version for these dependencies while they remain unstable.

### 4. `ProjectRepository` import in `main.rs` is used only in `complete_command()`

**File:** `src/main.rs`, line 27

```rust
use ports::project_repository::ProjectRepository;
```

This import is needed because `repo.list()` is called via the trait. That is fine and architecturally correct — `main.rs` is the composition root and is allowed to use both ports and adapters. However, `complete_command()` also constructs `TomlProjectRepository` directly (line 72), so the import chain is: concrete adapter + port trait, both in main.rs. This is the right place for it. No violation, just noting it for clarity.

### 5. `complete_command()` silently swallows `repo.list()` errors

**File:** `src/main.rs`, line 73

```rust
let names: Vec<String> = repo.list().unwrap_or_default();
```

On error (e.g. corrupted TOML, missing config directory), completion silently returns an empty list. This is reasonable for a completion path — crashing on Tab would be worse — but it means a misconfigured installation gives the impression completions are working (the script loads, Tab works) while returning no candidates. There is no warning emitted. This is acceptable behaviour, but a debug/trace log would help users diagnose "why are no projects completing?".

### 6. Fish completions instruction in `src/cli/mod.rs` uses process substitution

**File:** `src/cli/mod.rs`, line 183

```
source (COMPLETE=fish devs | psub)
```

The `psub` command is Fish-specific process substitution. This is correct Fish syntax. The instruction is accurate, but Fish support is explicitly marked as "untested" in the plan. The README says "(should work via CompleteEnv but untested)". Consider noting this caveat in the `--help` text too, not just in the project docs.

---

## What's Done Well

- **Architecture is clean.** `complete_command()` lives entirely in `main.rs` (the composition root). No port traits or domain types are polluted with completion concerns. The `cli/` handlers remain unaware of completions.
- **No duplication.** The function re-uses `Cli::command()` and mutates it with candidates rather than duplicating the command tree.
- **Graceful degradation.** Static completions are preserved as a fallback; the `completions` subcommand still works and now has better help text.
- **Docs updated consistently.** README, `--help` text, and project docs all reflect the new flow. The static setup instructions are preserved under a collapsible section.
- **Cargo.lock committed.** The additional transitive dependencies (`is_executable`, `shlex`, new `windows-sys` pin) are tracked.
- **All tests pass, clippy clean, no warnings.**

---

## Summary

No critical issues. One concrete UX bug (High #1): project name completions are incorrectly attached to `devs new`, which accepts a *new* name and should not suggest existing ones. One misleading test (High #2): `dynamic_completions_includes_project_names` asserts nothing about project names and duplicates the assertion already made by `dynamic_completions_outputs_shell_script`; the real completion path (candidate generation at tab-press time) is untested.

Both High issues should be addressed before this is considered complete. The rest are informational.
