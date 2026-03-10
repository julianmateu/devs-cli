# Dynamic shell completions

## Status: COMPLETED (2026-03-10)

## What was implemented

Dynamic project name completions using `clap_complete`'s `CompleteEnv` + `ArgValueCandidates`.

### How it works

1. User adds `source <(COMPLETE=zsh devs)` to `.zshrc`
2. On tab-press, the shell calls back into `devs` with special args
3. `CompleteEnv::complete()` intercepts this, calls `ArgValueCandidates` closures
4. Project names are loaded from disk and returned as completion candidates

### Approach chosen: CompleteEnv + ArgValueCandidates (Option A)

- `unstable-ext` feature on clap, `unstable-dynamic` feature on clap_complete
- `complete_command()` factory in `main.rs` auto-discovers subcommands with a `name` arg
- No architecture violations — all adapter construction stays in `main.rs`

### Commits

- `ee84193` — Add dynamic project name completions via CompleteEnv
- `a53339e` — Update completions docs and add integration tests for dynamic completions

### Files changed

| File | Change |
|------|--------|
| `Cargo.toml` | Added `unstable-ext` to clap, `unstable-dynamic` to clap_complete |
| `src/main.rs` | Added `complete_command()` factory, `CompleteEnv` call before `Cli::parse()` |
| `src/cli/completions.rs` | Updated stderr message to mention static vs dynamic |
| `src/cli/mod.rs` | Updated `Completions` help text with dynamic setup instructions |
| `README.md` | Rewrote shell completions section (dynamic recommended, static as fallback) |
| `tests/cli_tests.rs` | Added 2 integration tests for dynamic completions |

## Out of scope

- Session label completions for `--resume` (clap_complete doesn't support context-dependent completions yet)
- Fish/PowerShell testing (should work via CompleteEnv but untested)
