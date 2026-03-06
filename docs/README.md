# devs-cli

A project-aware tmux session manager with Claude Code session tracking.

## Problem

When working across multiple projects simultaneously, each with its own tmux session (neovim, terminals, multiple Claude Code sessions), restarting the machine means losing:

- Which projects were active and their folder paths
- The tmux pane layouts for each project
- Which Claude Code sessions were relevant to each project (and what they were about)
- The mental context of "what was I doing in each project"

Existing tools (tmuxinator, tmuxp, tmux-resurrect) handle layout save/restore but don't understand **projects** as a first-class concept, and none of them track Claude Code sessions.

## What devs-cli does

`devs` is a CLI that wraps tmux and Claude Code to provide project-level orchestration:

- **Register projects** with a folder path, tab color, and default pane layout
- **Open projects** with a single command that creates (or attaches to) a tmux session
- **Track Claude Code sessions** per project, with labels and active/done status
- **Save and restore** tmux layouts (both declarative configs and runtime snapshots)
- **Keep fleeting notes** per project ("picking up from step 4", "waiting on PR review")
- **Set iTerm2 tab colors** automatically via escape sequences

## Key design principles

- **Additive, not a cage**: direct tmux interaction is never blocked. `devs` adds project awareness on top of tmux, it doesn't replace it.
- **Two-layer layout**: declarative config (human-editable baseline) + runtime snapshots (exact state capture). Either can be used to restore.
- **Separate concerns**: pane layout (visual) and Claude sessions (logical) are tracked independently. Claude sessions are not tied to specific panes.
- **Simple storage**: one TOML file per project in `~/.config/devs/projects/`. No database.
- **iTerm2-optional**: tab colors use standard escape sequences. They work in iTerm2, are silently ignored elsewhere.

## Tech stack

- **Language**: Rust
- **CLI framework**: clap
- **Config format**: TOML (serde + toml crate)
- **Session management**: tmux (shelling out to tmux commands)
- **Tab colors**: OSC escape sequences (no iTerm2 API dependency)

## Documentation

- [Design decisions](design.md) — architecture and rationale for key choices
- [CLI commands](cli-commands.md) — full command reference
- [Data model](data-model.md) — TOML config structure and storage layout
- [Reference: iTerm2 colors](reference/iterm2-colors.md) — escape sequence reference
- [Reference: tmux layouts](reference/tmux-layouts.md) — layout mechanics reference
