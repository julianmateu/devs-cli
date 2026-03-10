# devs-cli

A project-aware tmux session manager with Claude Code session tracking, written in Rust.

## Project documentation

Design decisions, data model, and CLI command reference are in `docs/`. Read them before making changes.

## Development principles

### TDD (Test-Driven Development)

- **Red-Green-Refactor**: Write a failing test first, make it pass with minimal code, then refactor.
- Never trust a test you didn't see fail. If a test passes on the first run, verify it's actually testing what you think.
- Test the behavior, not the implementation. Tests should describe *what* the code does, not *how*.
- Test happy paths, error cases, and edge cases.
- Keep tests fast — no real filesystem or tmux calls in unit tests. Use trait-based test doubles.

### Clean Architecture

The codebase follows a ports-and-adapters (hexagonal) architecture. Domain logic must never depend on infrastructure.

```
src/
├── domain/           # Pure business logic and types. No I/O.
│   ├── project.rs    # ProjectConfig, ProjectMetadata
│   ├── layout.rs     # Layout, MainPane, SplitPane, SplitDirection
│   ├── claude_session.rs  # ClaudeSession, ClaudeSessionStatus
│   ├── note.rs            # Note
│   ├── saved_state.rs     # SavedState, SavedPane
│   ├── local_config.rs    # LocalConfig (.devs.toml model)
│   ├── duration.rs        # Duration parsing (2d, 1h, 30m)
│   ├── path.rs            # abbreviate_home, expand_home
│   └── test_helpers.rs    # Shared test fixtures
│
├── ports/            # Trait definitions (interfaces).
│   ├── project_repository.rs   # trait ProjectRepository
│   ├── tmux_adapter.rs         # trait TmuxAdapter
│   ├── terminal_adapter.rs     # trait TerminalAdapter
│   ├── process_launcher.rs     # trait ProcessLauncher
│   └── local_config.rs         # traits LocalConfigReader, LocalConfigWriter
│
├── adapters/         # Infrastructure implementations.
│   ├── toml_project_repository.rs  # Split TOML files (projects/ + local/)
│   ├── shell_tmux_adapter.rs       # Shell tmux commands
│   ├── iterm_terminal_adapter.rs   # OSC escape sequences
│   ├── noop_terminal_adapter.rs    # No-op for non-iTerm terminals
│   ├── os_process_launcher.rs      # std::process
│   ├── toml_local_config.rs        # .devs.toml reader/writer
│   ├── split_config.rs             # PortableConfig/MachineLocalConfig
│   ├── config_version.rs           # Config version tracking
│   └── migration.rs                # v1 → v2 auto-migration
│
├── cli/              # Command handlers (receive traits, not concrete types).
│   ├── mod.rs        # Clap derive structs
│   ├── new.rs, init.rs, list.rs, status.rs, config.rs, edit.rs, remove.rs
│   ├── open.rs, close.rs, save.rs, reset.rs
│   ├── claude.rs, claudes.rs, claude_done.rs
│   ├── note.rs, notes.rs
│   ├── resolve.rs    # CWD-based project name resolution
│   ├── completions.rs, tmux_help.rs, man.rs
│   └── format.rs     # Re-exports domain::path helpers
│
└── main.rs           # Composition root + dynamic completions (CompleteEnv)
```

**Key rules:**
- `domain/` imports nothing from `ports/`, `adapters/`, or `cli/`. It's pure Rust with serde derives.
- `ports/` defines traits. It may reference `domain/` types in trait signatures.
- `adapters/` implements port traits. It depends on `domain/` and `ports/`, never on `cli/`.
- `cli/` handlers receive port traits (not concrete adapters). They orchestrate domain logic.
- `main.rs` is the composition root: it constructs concrete adapters and passes them to CLI handlers.
- Tests use in-memory or mock implementations of the port traits.

### Clean Code

- Functions should do one thing. If a function has "and" in its description, split it.
- Names should reveal intent. No abbreviations unless universally understood (`id`, `cmd`, `config`).
- No comments explaining *what* the code does — the code should be self-explanatory. Comments explain *why* when the reason isn't obvious.
- No dead code. No commented-out code. Delete it; git has history.
- Error messages should be actionable: say what went wrong *and* what the user can do about it.

### SOLID

- **SRP**: Each struct/module has one reason to change. `ProjectRepository` handles persistence, not validation. `Project` handles validation, not persistence.
- **OCP**: New behaviors (e.g., a new storage backend) are added by implementing a trait, not modifying existing code. Avoid match/if-else chains that grow with new variants.
- **LSP**: All implementations of a trait must be substitutable. An `InMemoryProjectRepository` must behave identically to `TomlProjectRepository` from the caller's perspective.
- **ISP**: Keep traits focused. Don't put `set_tab_color` on `TmuxAdapter`. Terminal concerns and tmux concerns are separate traits.
- **DIP**: CLI handlers depend on `dyn ProjectRepository`, not `TomlProjectRepository`. Constructing the concrete type happens in `main.rs` only.

## Rust conventions

- Use `thiserror` for domain error types, `anyhow` for CLI-level error propagation.
- Prefer `&str` over `String` in function parameters where ownership isn't needed.
- Use `impl Into<String>` or generics sparingly — only when it genuinely improves the API.
- Derive `Debug`, `Clone`, `PartialEq` on domain types for testability.
- Use `#[cfg(test)]` modules in the same file for unit tests. Integration tests go in `tests/`.

## Testing strategy

- **Unit tests**: domain logic, validation, serialization/deserialization. Use in-memory trait implementations.
- **Integration tests**: end-to-end CLI commands against a temp directory (no real tmux). Use `assert_cmd` and `tempfile` crates.
- **Manual tests**: tmux integration, tab colors, Claude session launching. These can't be easily automated.

## Git hooks

Pre-commit hooks are tracked in `.githooks/` and activated via `git config core.hooksPath .githooks`. The pre-commit hook runs:

1. `cargo fmt --check`
2. `cargo clippy -- -D warnings`
3. `cargo test --quiet`

All three must pass before a commit is accepted.

## Commit conventions

- Commit messages: imperative mood, concise (`Add project CRUD`, `Implement devs open command`)
- One logical change per commit
- All tests must pass before committing (enforced by pre-commit hook)
