# UX Improvements Investigation

## 1. `devs edit` — Support both local and project config

**Current state:** `devs edit <name>` only opens `~/.config/devs/projects/<name>.toml` (the portable config). There's no way to edit:
- `~/.config/devs/local/<name>.toml` (machine-local config: claude sessions, saved state)
- `.devs.toml` in the project directory (local config: color, layout)

**`devs config` comparison:** `devs config <name>` prints the *merged* portable + machine-local config. It does NOT include `.devs.toml` from the project dir.

**Proposal:** Add a `--local` flag to `devs edit`:
- `devs edit <name>` → opens `~/.config/devs/projects/<name>.toml` (portable, as today)
- `devs edit <name> --local` → opens `~/.config/devs/local/<name>.toml` (machine-local)
- `.devs.toml` is a separate concern (it lives in the project dir, not in ~/.config/devs)

**Key files:** `src/cli/edit.rs`, `src/cli/mod.rs` (Commands::Edit), `src/main.rs`

---

## 2. Infer project name from CWD

**Current state:** Every command that needs a project requires an explicit `name` argument. No CWD-based inference exists.

**How paths are stored:** Project paths are stored abbreviated (`~/src/foo`). `expand_home()` converts to absolute. CWD from `std::env::current_dir()` gives an absolute path.

**Proposal:** Add a `resolve_project_name()` function:
1. If `name` is provided as argument, use it (current behavior).
2. If `name` is omitted, get CWD and compare against all project paths (after expanding `~`).
3. Match if CWD equals or is a subdirectory of a project's path.
4. Error if no match or multiple matches.

**Impact:** This touches every command variant in `src/cli/mod.rs` — all `name: String` fields become `name: Option<String>`. The resolution logic goes in `main.rs` before dispatching.

**Key files:** `src/cli/mod.rs`, `src/main.rs`, `src/domain/path.rs`, `src/cli/format.rs`

---

## 3. Dynamic shell completions

**Current state:** `devs completions <shell>` generates static completions via `clap_complete::generate()`. Only completes subcommand names and flags, not project names or session labels.

**Options for dynamic completions:**
- **clap_complete with `CompleteEnv`** (clap 4.5+): Supports runtime completions via a callback. The binary itself provides completions when invoked with a special env var. This is the modern recommended approach.
- **Custom shell functions**: Write shell-specific completion scripts that call `devs list` etc.

**Proposal:** Use `clap_complete`'s `CompleteEnv` or custom value hints + a hidden `complete` subcommand:
- Project names: `devs list` already exists and returns sorted names.
- Claude session labels: Would need a new output mode or hidden subcommand.

**Key files:** `src/cli/completions.rs`, `src/cli/mod.rs`, `Cargo.toml` (clap_complete version)

---

## 4. Docs / man pages / help text improvements

**Current state:**
- Help comes entirely from Clap `///` doc comments in `src/cli/mod.rs`
- Man pages auto-generated from the same Clap definitions
- `devs tmux-help` prints a hardcoded tmux reference

**Key files:** `src/cli/mod.rs` (all doc comments), `src/cli/man.rs`, `src/cli/tmux_help.rs`, `docs/`
