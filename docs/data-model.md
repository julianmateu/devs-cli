# Data Model

## Storage layout (v2)

Config is split into **portable** (syncable) and **machine-local** files:

```
~/.config/devs/
├── config.toml              # version = 2
├── .gitignore               # excludes local/ and backup-v1/
├── projects/
│   ├── rmbs-tool.toml       # portable: metadata, layout, notes
│   └── my-api.toml
└── local/
    ├── rmbs-tool.toml       # machine-specific: claude sessions, saved tmux state
    └── my-api.toml
```

The `projects/` directory contains data safe to sync across machines (via git). The `local/` directory contains machine-specific data that should stay on this computer.

## Global config (`config.toml`)

Tracks the config format version. Created automatically on first run or migration.

```toml
version = 2
```

## Portable project config (`projects/<name>.toml`)

Contains metadata, layout, and notes.

### Full example

```toml
[project]
name = "rmbs-tool"
path = "~/src/rmbs-tool"
color = "#e06c75"
created_at = "2026-03-03T10:00:00Z"

# Declarative layout (baseline)
# The main pane is the initial pane created with the session.
# Additional panes split relative to the previous active pane.
# Optional layout_string preserves exact tmux geometry (written by `devs save --as-default`).
[layout]
# layout_string = "5aed,176x79,0,0[...]"   # optional, overrides geometry from splits

[layout.main]
cmd = "nvim"

[[layout.panes]]
cmd = "claude"
split = "right"
size = "40%"

[[layout.panes]]
split = "bottom-right"

# Fleeting notes (append-only)
[[notes]]
content = "picking up from step 4 of the migration plan"
created_at = "2026-03-03T10:15:00Z"

[[notes]]
content = "blocked on API key, asked Sarah"
created_at = "2026-03-03T14:30:00Z"
```

## Machine-local config (`local/<name>.toml`)

Contains claude sessions and saved tmux state. Only created when there is local data to store.

### Full example

```toml
[[claude_sessions]]
id = "session_abc123"
label = "brainstorm architecture"
status = "active"
started_at = "2026-03-01T10:00:00Z"

[[claude_sessions]]
id = "session_old789"
label = "initial exploration"
status = "done"
started_at = "2026-02-28T09:00:00Z"
finished_at = "2026-02-28T17:00:00Z"

# Saved tmux state (captured by `devs save`)
[last_state]
captured_at = "2026-03-03T16:00:00Z"
layout_string = "5aed,176x79,0,0[176x59,0,0,0,176x19,0,60{87x19,0,60,1,88x19,88,60,2}]"

[[last_state.panes]]
index = 0
path = "/Users/julian/src/rmbs-tool"
command = "nvim"

[[last_state.panes]]
index = 1
path = "/Users/julian/src/rmbs-tool"
command = "claude"

[[last_state.panes]]
index = 2
path = "/Users/julian/src/rmbs-tool"
command = "zsh"
```

## Path storage

Paths under `$HOME` are stored in tilde form (`~/src/foo`) and expanded at runtime. Paths outside `$HOME` (e.g., `/Volumes/external/project`) are stored as absolute paths.


## Field reference

### `[project]`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | yes | Project identifier (used as tmux session name) |
| `path` | string | yes | Path to the project directory (`~/...` or absolute) |
| `color` | string | no | Hex color for iTerm2 tab (`"#rrggbb"` or `"rrggbb"`) |
| `created_at` | string | yes | ISO 8601 timestamp |

### `[layout]`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `layout_string` | string | no | tmux layout string for exact geometry (written by `devs save --as-default`) |

When `layout_string` is present, pane geometry comes from `tmux select-layout` instead of split directions. Pane commands are still read from `[layout.main]` and `[[layout.panes]]`.

### `[layout.main]`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `cmd` | string | no | Command to run in the main pane (default: shell) |

### `[[layout.panes]]`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `cmd` | string | no | Command to run in the pane (default: shell) |
| `split` | string | yes | Split direction (see below) |
| `size` | string | no | Size as percentage (`"40%"`) |

**Split values**:

| Value | Meaning | tmux equivalent |
|-------|---------|-----------------|
| `right` | Vertical split to the right of the current pane | `split-window -h` |
| `bottom` | Horizontal split below the current pane | `split-window -v` |
| `bottom-right` | Horizontal split below the rightmost pane | `select-pane -t {right}` then `split-window -v` |

The `main` pane is the initial pane created with the tmux session. Additional panes are created in order, each splitting relative to the pane that was active after the previous split.

### `[[claude_sessions]]` (in `local/<name>.toml`)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | yes | Claude Code session ID |
| `label` | string | yes | Human-readable description of the session's purpose |
| `status` | string | yes | `"active"` or `"done"` |
| `started_at` | string | yes | ISO 8601 timestamp |
| `finished_at` | string | yes | ISO 8601 timestamp or `""` if active |

### `[[notes]]`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `content` | string | yes | The note text |
| `created_at` | string | yes | ISO 8601 timestamp |

### `[last_state]` (in `local/<name>.toml`)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `captured_at` | string | yes | ISO 8601 timestamp of when the state was saved |
| `layout_string` | string | yes | tmux layout string (from `list-windows -F '#{window_layout}'`) |

### `[[last_state.panes]]` (in `local/<name>.toml`)

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `index` | integer | yes | Pane index within the window |
| `path` | string | yes | Working directory of the pane |
| `command` | string | yes | Foreground command running in the pane |


## Syncing across machines

devs stores config in `~/.config/devs/`. To sync project layouts and notes across machines:

```bash
cd ~/.config/devs
git init
git add -A    # local/ is gitignored automatically
git remote add origin <your-repo>
git push
```

Machine-specific data (claude sessions, tmux snapshots) is in `local/` which is excluded by the auto-generated `.gitignore`.

On another machine, clone the repo and devs will use the synced layouts and notes while keeping local session data separate.


## Migration

When devs detects a v1 config (no `config.toml` or `version < 2`), it automatically migrates:

1. Backs up `projects/` to `backup-v1/`
2. Splits each project file into portable + local
3. Converts absolute paths under `$HOME` to tilde form
4. Creates `.gitignore` (excludes `local/` and `backup-v1/`)
5. Writes `config.toml` with `version = 2`

Migration is automatic and idempotent.
