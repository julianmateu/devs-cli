# Implementation Plan

## 1. Man pages (`devs generate-man`) ✅

Implemented in commit 8e2ce3d. Generates man pages from clap metadata using `clap_mangen`.
3 integration tests (files created, valid roff, creates output directory). Docs updated.


## 2. tmux help (`devs tmux-help`)

Print a quick reference of common tmux operations. Pure text output, no tmux interaction.

### Design
- Static text embedded in the binary
- Handler in `src/cli/tmux_help.rs` -- prints to stdout
- Content: prefix key, pane navigation, window management, session management, copy mode

### Files
- `src/cli/tmux_help.rs` -- handler with static content
- `src/cli/mod.rs` -- add variant + module
- `src/main.rs` -- add match arm
- `tests/cli_tests.rs` -- integration test

### Tests
- Integration: output contains key sections (panes, windows, sessions)


## 3. Shareable `.devs.toml`

Allow a `.devs.toml` file in the project root that defines layout and color. When `devs new` is run with just a path, it picks up the local config.

### Design
- Domain: new struct `LocalConfig` with optional layout + color
- Port: extend `ProjectRepository` or add `LocalConfigReader` trait
- Adapter: `TomlLocalConfigReader` reads `.devs.toml` from project path
- CLI: `devs new <name> --path <path>` checks for `.devs.toml` and merges
- Explicit flags (`--color`, `--from`) take priority over `.devs.toml`

### Files
- `src/domain/local_config.rs` -- `LocalConfig` struct
- `src/ports/local_config_reader.rs` -- trait
- `src/adapters/toml_local_config_reader.rs` -- implementation
- `src/cli/new.rs` -- merge logic
- `src/main.rs` -- wire adapter

### Tests
- Unit: `LocalConfig` parsing
- Unit: merge logic (explicit flags win)
- Integration: `devs new` with `.devs.toml` present picks up layout
- Integration: `--from` overrides `.devs.toml` layout

### Open questions
- Should `.devs.toml` support `claude_sessions`? Probably not -- those are per-user.
<!-- REVIEWER: No -->
- Should `devs init` create a `.devs.toml` from the current project config? Nice to have.
<!-- REVIEWER: Yes -->
