# Dynamic shell completions

## Goal

Tab-completion for project names and claude session labels.

## Current behavior

`devs completions <shell>` generates static completions via `clap_complete::generate()`. Only subcommands and flags are completed.

## Desired behavior

- Tab-completing after `devs open ` shows registered project names
- Tab-completing after `devs claude <project> --resume ` shows session labels for that project
- Works for zsh, bash, and fish

## Research needed

- Check current `clap_complete` version in `Cargo.toml`
- Evaluate `clap_complete`'s `CompleteEnv` (clap 4.5+) vs custom approach
- Determine if `ValueHint::Other` with a custom completer is viable

## Implementation approaches

### Option A: clap_complete custom completer (preferred if available)
- Use `clap_complete::engine::ArgValueCompleter` or similar
- Register a callback that calls `repo.list()` at completion time
- Requires the binary to be invoked for completions (standard pattern)

### Option B: Hidden `--complete` subcommand
- `devs _complete projects` → prints project names one per line
- `devs _complete sessions <project>` → prints session labels
- Shell completion scripts call these subcommands
- More shell-specific scripting needed

### Option C: Static generation with dynamic wrapper
- Generate base completions with clap_complete
- Wrap with shell functions that inject dynamic values

## Files to change

- `Cargo.toml` — possibly update clap_complete version
- `src/cli/mod.rs` — add value hints or custom completers
- `src/cli/completions.rs` — update generation logic
- `src/main.rs` — wire up repository access for completions

## Status

Needs research on clap_complete version and capabilities before detailed planning.
