# CWD-based project name inference

**Status: Completed** (2026-03-10)

## What was done

All 13 project-scoped commands now accept `name` as `Option<String>`. When omitted, the project is inferred by matching CWD against registered project paths.

### Implementation

- **`src/cli/resolve.rs`** — `resolve_project_name(name, cwd, home_dir, repo)` handles resolution:
  - Explicit name passes through (no repo access)
  - Otherwise iterates registered projects, expands tilde paths, matches CWD via `Path::starts_with`
  - Deepest match wins when paths are nested
  - Actionable errors for no match and ambiguous match
- **`src/cli/mod.rs`** — Changed `name: String` → `name: Option<String>` on 13 commands (Init, Config, Edit, Remove, Open, Close, Save, Reset, Claude, Claudes, ClaudeDone, Note, Notes)
- **`src/main.rs`** — Added `cwd`/`home_dir` computation and `resolve` closure, updated all 13 match arms
- **Disambiguation for multi-positional commands** — `ClaudeDone`, `Note`, and `Claude` have additional positional args. Made those also `Option<String>` at the Clap level. When only one positional is provided, it's treated as the non-name arg (label/message), and name is inferred from CWD.

### Tests

- 8 unit tests in `src/cli/resolve.rs` covering: pass-through, exact match, subdirectory match, no match, deepest match, ambiguous match, tilde expansion, empty repo
- 3 integration tests in `tests/cli_tests.rs`: CWD exact match, subdirectory match, explicit name ignores CWD
