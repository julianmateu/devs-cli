# devs

[![CI](https://github.com/julianmateu/devs-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/julianmateu/devs-cli/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/devs-cli.svg)](https://crates.io/crates/devs-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

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
- **Simple storage** -- TOML files in `~/.config/devs/`, split into portable config (syncable across machines) and machine-local data

## Installation

### Prerequisites

- [tmux](https://github.com/tmux/tmux) (3.2+ recommended for tab color passthrough)
- iTerm2 (optional -- tab colors are silently ignored in other terminals)

### Homebrew (macOS and Linux)

```bash
brew tap julianmateu/devs
brew install devs
```

### Cargo (any platform with Rust)

```bash
cargo install devs-cli
```

### Build from source

```bash
git clone https://github.com/julianmateu/devs-cli.git
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
path = "~/src/my-api"
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

Most project-scoped commands accept an optional project name. When omitted, the project is **inferred from your current directory** by matching against registered project paths. This works from the project root or any subdirectory.

### Project management

| Command | Description |
|---------|-------------|
| `devs new <name> [--path <path>] [--color <hex>]` | Register a new project |
| `devs new <name> --from <project>` | Copy layout from an existing project |
| `devs new <name> --from-session <session>` | Capture layout from a live tmux session |
| `devs init [name]` | Export project config to a shareable `.devs.toml` |
| `devs list` | List all registered projects |
| `devs status` | Show all projects with live tmux/Claude status |
| `devs config [name]` | Print merged config (portable + local) to stdout |
| `devs edit [name]` | Open the portable config in `$EDITOR` |
| `devs remove [name] --force [--kill]` | Remove a project (`--kill` to also kill tmux session) |

`devs new` accepts `--session LABEL:ID` (repeatable) to pre-populate Claude sessions. `--path` defaults to the current directory and expands `~` to `$HOME`. If a `.devs.toml` file exists in the project directory, its color and layout are picked up automatically (explicit flags override).

```bash
devs new frontend --path ~/src/frontend --color "#e06c75"
cd ~/src/frontend && devs new frontend     # --path defaults to CWD
devs new fork --from frontend --session "main:abc123"
devs new captured --path ~/src/captured --from-session frontend
devs init frontend                         # export config to .devs.toml
devs list
devs status
devs config frontend
devs edit frontend
devs remove frontend --force --kill
```

### Session management

| Command | Description |
|---------|-------------|
| `devs open [name]` | Create or attach to a tmux session |
| `devs open [name] --default` | Always use the declarative layout |
| `devs open [name] --saved` | Always use the saved state (error if none) |
| `devs save [name]` | Snapshot the current tmux layout |
| `devs save [name] --as-default` | Save current layout as the declarative default |
| `devs close [name] [--save]` | Kill tmux session and reset tab color |
| `devs reset [name]` | Discard saved state, revert to declarative layout |

`devs open` is idempotent: if the tmux session already exists, it attaches to it. If saved state exists and no flags are given, it restores the saved layout. Use `--default` to force the declarative layout instead.

In layout pane commands, `claude` and `claude:<label>` are expanded automatically -- `claude:brainstorm` starts or resumes a Claude session with label "brainstorm".

```bash
devs open my-api
devs save my-api
devs save my-api --as-default   # capture current layout as the declarative default
devs close my-api --save        # save layout, kill session, reset tab color
devs open my-api --default   # ignore saved state, use config layout
devs reset my-api            # discard saved state entirely
```

### Claude Code session tracking

| Command | Description |
|---------|-------------|
| `devs claude [name] <label>` | Launch a new Claude session with a label |
| `devs claude [name] --resume <label>` | Resume an existing Claude session |
| `devs claudes [name]` | List active Claude sessions for a project |
| `devs claudes [name] --all` | Include completed sessions |
| `devs claude-done [name] <label>` | Mark a Claude session as done |

When you run `devs open`, active Claude sessions are printed as hints so you know what to resume.

```bash
devs claude my-api "implement auth middleware"
devs claudes my-api
devs claude my-api --resume "implement auth middleware"
devs claude-done my-api "implement auth middleware"
```

### Notes

| Command | Description |
|---------|-------------|
| `devs note [name] <message>` | Add a timestamped note |
| `devs notes [name]` | Show last 20 notes |
| `devs notes [name] --all` | Show all notes |
| `devs notes [name] --since 2d` | Filter by time |
| `devs notes [name] --clear` | Delete all notes |

Notes are fleeting breadcrumbs, not tasks. They help you remember where you left off after a context switch.

```bash
devs note my-api "picking up from step 4 of the migration"
devs note my-api "blocked on API key, asked Sarah"
devs notes my-api
```

### Global

| Command | Description |
|---------|-------------|
| `devs completions <shell>` | Generate shell completions |
| `devs tmux-help` | Print tmux quick reference |
| `devs generate-man <output-dir>` | Generate man pages |
| `devs --version` | Print version |
| `devs --help` | Print help for all commands |
| `devs <command> --help` | Print help for a specific command |

### Shell completions

Dynamic completions complete subcommands, flags, **and project names**. Add one line to your shell config. Run `devs completions --help` to see these instructions at any time.

#### Dynamic setup (recommended)

**Zsh** — add to `~/.zshrc`:

```zsh
source <(COMPLETE=zsh devs)
```

**Bash** — add to `~/.bashrc`:

```bash
source <(COMPLETE=bash devs)
```

**Fish** — add to `~/.config/fish/config.fish`:

```fish
source (COMPLETE=fish devs | psub)
```

#### Static fallback

If dynamic completions don't work on your system, you can generate static completions (subcommands and flags only, no project names):

<details>
<summary>Static setup instructions</summary>

**Oh My Zsh:**

```bash
mkdir -p ${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/plugins/devs
devs completions zsh > ${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/plugins/devs/_devs
```

Then add `devs` to the `plugins=(...)` list in your `~/.zshrc` and restart your shell.

**Vanilla zsh:**

```bash
mkdir -p ~/.zfunc
devs completions zsh > ~/.zfunc/_devs
```

Add to your `~/.zshrc` (before `compinit`):

```zsh
fpath=(~/.zfunc $fpath)
autoload -Uz compinit && compinit
```

**Bash:**

```bash
mkdir -p ~/.local/share/bash-completion/completions
devs completions bash > ~/.local/share/bash-completion/completions/devs
```

**Fish:**

```bash
devs completions fish > ~/.config/fish/completions/devs.fish
```

</details>

## Shareable config (`.devs.toml`)

Place a `.devs.toml` file in a project's root directory to define a shareable layout and color. When `devs new` is run from that directory (or with `--path` pointing to it), the settings are picked up automatically.

```toml
# .devs.toml
color = "#61afef"

[layout.main]
cmd = "nvim"

[[layout.panes]]
split = "right"
cmd = "claude"
size = "40%"

[[layout.panes]]
split = "bottom-right"
```

Explicit flags (`--color`, `--from`, `--from-session`) override `.devs.toml` values.

To export an existing project's config to `.devs.toml`:

```bash
devs init my-api
```

This writes the project's color and layout to `.devs.toml` in the project's directory, so team members can pick it up with `devs new`.

## Configuration

Config is split into **portable** (syncable) and **machine-local** files under `~/.config/devs/`.

### Portable config (`projects/<name>.toml`)

Metadata, layout, and notes. Paths under `$HOME` use tilde form (`~/...`).

```toml
[project]
name = "my-api"
path = "~/src/my-api"
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

[[notes]]
content = "picking up from step 4 of the migration"
created_at = "2026-03-03T10:15:00Z"
```

### Machine-local config (`local/<name>.toml`)

Claude sessions and saved tmux state. Not synced.

```toml
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

### Multi-machine sync

The portable config directory can be synced via git. Machine-local data is auto-gitignored.

```bash
cd ~/.config/devs
git init && git add -A && git remote add origin <your-repo> && git push
```

See [docs/data-model.md](docs/data-model.md) for full details.

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
