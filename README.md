# devs

A project-aware tmux session manager that remembers your layouts, tracks Claude Code sessions, and sets iTerm2 tab colors.

## What is this?

When you work across multiple projects, each with its own tmux session, editor, terminals, and Claude Code conversations, a machine restart wipes out all of that context. You lose the pane layouts, the Claude session IDs, and the mental breadcrumbs about where you left off.

`devs` fixes this. It treats **projects** as a first-class concept on top of tmux:

- Register a project once with its path and preferred layout
- Open it with a single command -- tmux session, pane layout, tab color, all set up
- Save and restore exact pane arrangements after a reboot
- Track Claude Code sessions per project so you can resume the right conversation
- Jot down fleeting notes so you remember what you were doing

It is additive, not a cage. Direct tmux interaction is never blocked. `devs` adds project awareness on top of tmux; it does not replace it.

## Features

- **Declarative layouts** -- define pane arrangements in TOML; `devs open` builds them
- **Save and restore** -- capture your live tmux layout and restore it exactly after a reboot
- **iTerm2 tab colors** -- each project gets a color, set automatically via escape sequences
- **Claude Code session tracking** -- record which Claude sessions belong to which project, with labels and active/done status
- **Fleeting notes** -- timestamped scratchpad per project for context breadcrumbs
- **Simple storage** -- one TOML file per project in `~/.config/devs/projects/`, human-readable and hand-editable

## Installation

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable)
- [tmux](https://github.com/tmux/tmux) (3.2+ recommended for tab color passthrough)
- iTerm2 (optional -- tab colors are silently ignored in other terminals)

### Build from source

```bash
git clone https://github.com/your-user/devs-cli.git
cd devs-cli
cargo install --path .
```

The binary is named `devs`.

### Development setup

After cloning, configure git to use the tracked hooks:

```bash
git config core.hooksPath .githooks
```

This enables the pre-commit hook which runs `cargo fmt --check`, `cargo clippy`, and `cargo test` before each commit.

## tmux configuration

For tab colors to work inside tmux, you need to enable escape sequence passthrough. Add this to your `~/.tmux.conf`:

```
set -g allow-passthrough on
```

Without this, tmux silently discards the escape sequences that set tab colors.

## Quick start

```bash
# 1. Register a project
devs new my-api --path ~/src/my-api --color "#61afef"

# 2. Edit the config to define your layout
devs edit my-api
```

The config file opens in `$EDITOR`. Add a layout:

```toml
[project]
name = "my-api"
path = "/Users/you/src/my-api"
color = "#61afef"
created_at = "2026-03-05T10:00:00Z"

[layout.main]
cmd = "nvim"

[[layout.panes]]
split = "right"
cmd = "claude"
size = "40%"

[[layout.panes]]
split = "bottom-right"
```

This creates three panes: nvim on the left, Claude Code on the top-right, and a shell on the bottom-right.

```bash
# 3. Open the project (creates tmux session + sets tab color)
devs open my-api

# 4. Work, resize panes, split new ones...

# 5. Save the current layout
devs save my-api

# 6. After a reboot, reopen -- saved layout is restored automatically
devs open my-api
```

## Commands

### Project management

| Command | Description |
|---------|-------------|
| `devs new <name> --path <path> [--color <hex>]` | Register a new project |
| `devs new <name> --from <project>` | Copy layout from an existing project |
| `devs list` | List all registered projects |
| `devs status` | Show all projects with live tmux/Claude status |
| `devs config <name>` | Print a project's TOML config to stdout |
| `devs edit <name>` | Open the project config in `$EDITOR` |
| `devs remove <name> --force [--kill]` | Remove a project (`--kill` to also kill tmux session) |

`devs new` accepts `--session LABEL:ID` (repeatable) to pre-populate Claude sessions, and `--path` expands `~` to `$HOME` automatically.

```bash
devs new frontend --path ~/src/frontend --color "#e06c75"
devs new fork --from frontend --session "main:abc123"
devs list
devs status
devs config frontend
devs edit frontend
devs remove frontend --force --kill
```

### Session management

| Command | Description |
|---------|-------------|
| `devs open <name>` | Create or attach to a tmux session |
| `devs open <name> --default` | Always use the declarative layout |
| `devs open <name> --saved` | Always use the saved state (error if none) |
| `devs save <name>` | Snapshot the current tmux layout |
| `devs close <name> [--save]` | Kill tmux session and reset tab color |
| `devs reset <name>` | Discard saved state, revert to declarative layout |

`devs open` is idempotent: if the tmux session already exists, it attaches to it. If saved state exists and no flags are given, it restores the saved layout. Use `--default` to force the declarative layout instead.

In layout pane commands, `claude` and `claude:<label>` are expanded automatically -- `claude:brainstorm` starts or resumes a Claude session with label "brainstorm".

```bash
devs open my-api
devs save my-api
devs close my-api --save     # save layout, kill session, reset tab color
devs open my-api --default   # ignore saved state, use config layout
devs reset my-api            # discard saved state entirely
```

### Claude Code session tracking

| Command | Description |
|---------|-------------|
| `devs claude <name> <label>` | Launch a new Claude session with a label |
| `devs claude <name> --resume <id>` | Resume an existing Claude session |
| `devs claudes <name>` | List active Claude sessions for a project |
| `devs claudes <name> --all` | Include completed sessions |
| `devs claude-done <name> <id>` | Mark a Claude session as done |

When you run `devs open`, active Claude session IDs are printed as hints so you know what to resume.

```bash
devs claude my-api "implement auth middleware"
devs claudes my-api
devs claude my-api --resume abc123
devs claude-done my-api abc123
```

### Notes

| Command | Description |
|---------|-------------|
| `devs note <name> <message>` | Add a timestamped note |
| `devs notes <name>` | Show last 20 notes |
| `devs notes <name> --all` | Show all notes |
| `devs notes <name> --since 2d` | Filter by time |
| `devs notes <name> --clear` | Delete all notes |

Notes are fleeting breadcrumbs, not tasks. They help you remember where you left off after a context switch.

```bash
devs note my-api "picking up from step 4 of the migration"
devs note my-api "blocked on API key, asked Sarah"
devs notes my-api
```

### Global

| Command | Description |
|---------|-------------|
| `devs --version` | Print version |
| `devs --help` | Print help for all commands |
| `devs <command> --help` | Print help for a specific command |

## Configuration

Projects are stored as individual TOML files in `~/.config/devs/projects/`. Here is a full example:

```toml
[project]
name = "my-api"
path = "/Users/you/src/my-api"
color = "#61afef"
created_at = "2026-03-05T10:00:00Z"

# Declarative layout (baseline)
# The main pane is the initial pane created with the session.
# Additional panes split relative to the active pane.
[layout.main]
cmd = "nvim"

[[layout.panes]]
split = "right"
cmd = "claude"
size = "40%"

[[layout.panes]]
split = "bottom-right"

# Claude Code sessions
[[claude_sessions]]
id = "session_abc123"
label = "implement auth middleware"
started_at = "2026-03-01T10:00:00Z"
status = "active"

[[claude_sessions]]
id = "session_def456"
label = "initial exploration"
started_at = "2026-02-28T09:00:00Z"
status = "done"
finished_at = "2026-02-28T17:00:00Z"

# Notes
[[notes]]
content = "picking up from step 4 of the migration"
created_at = "2026-03-03T10:15:00Z"

# Saved tmux state (written by `devs save`, not hand-edited)
[last_state]
captured_at = "2026-03-03T16:00:00Z"
layout_string = "5aed,176x79,0,0[176x59,0,0,0,176x19,0,60{87x19,0,60,1,88x19,88,60,2}]"

[[last_state.panes]]
index = 0
path = "/Users/you/src/my-api"
command = "nvim"

[[last_state.panes]]
index = 1
path = "/Users/you/src/my-api"
command = "claude"

[[last_state.panes]]
index = 2
path = "/Users/you/src/my-api"
command = "zsh"
```

### Layout split directions

| Value | Meaning |
|-------|---------|
| `right` | Vertical split to the right |
| `bottom` | Horizontal split below |
| `bottom-right` | Horizontal split below the rightmost pane |

The `main` pane is always the first pane, created with the tmux session. Additional panes are created in order, each splitting relative to the previous active pane.

## Architecture

`devs` follows a ports-and-adapters (hexagonal) architecture. Domain logic is pure and has no I/O dependencies. Infrastructure (TOML persistence, tmux commands, terminal escape sequences) is behind trait boundaries.

```
src/
  domain/       Pure business logic and types. No I/O.
  ports/        Trait definitions (ProjectRepository, TmuxAdapter, TerminalAdapter)
  adapters/     Implementations (TOML files, shell tmux commands, OSC escapes)
  cli/          Command handlers, receiving traits not concrete types
  main.rs       Composition root: wires concrete adapters into handlers
```

See `docs/` for detailed design decisions and the full data model.

## License

MIT
