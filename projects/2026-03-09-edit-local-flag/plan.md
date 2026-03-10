# devs edit --local flag

**Status: COMPLETED** (commit `13da6fc`)

## Goal

Allow `devs edit` to open either the portable config or the machine-local config file.

## Behavior

- `devs edit <name>` → opens portable config (unchanged)
- `devs edit <name> --local` → opens `~/.config/devs/local/<name>.toml` (machine-local config)
- Creates the `local/` parent directory if it doesn't exist (editors handle new files fine)

## Changes made

- `src/cli/mod.rs` — added `#[arg(long)] local: bool` to `Commands::Edit`
- `src/cli/edit.rs` — updated `run()` to accept `local: bool`, branch path logic, `create_dir_all` for local dir
- `src/main.rs` — destructure and pass `local`

## Tests added

- `edit_local_fails_for_missing_project` — validates project must exist even with `--local`
- `edit_local_creates_parent_directory` — verifies `local/` dir is created on demand
