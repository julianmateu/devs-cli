# Data Model

## Storage layout

```
~/.config/devs/
├── config.toml              # global defaults (optional)
└── projects/
    ├── rmbs-tool.toml       # one file per project
    └── my-api.toml
```

## Global config (`config.toml`)

Optional file for defaults that apply to all projects.

```toml
# Default layout used when a project doesn't specify its own
[[default_layout.panes]]
cmd = "nvim"
split = "main"

[[default_layout.panes]]
split = "right"

# Default tab color assignment strategy (future)
# color_palette = ["#e06c75", "#98c379", "#61afef", "#c678dd", "#e5c07b"]
```

## Project config (`projects/<name>.toml`)

### Full example

```toml
[project]
name = "rmbs-tool"
path = "/Users/julian/src/rmbs-tool"
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

# Claude Code sessions
[[claude_sessions]]
id = "session_abc123"
label = "brainstorm architecture"
status = "active"               # "active" or "done"
started_at = "2026-03-01T10:00:00Z"
finished_at = ""                # empty string when active

[[claude_sessions]]
id = "session_def456"
label = "implement step 4 of migration"
status = "active"
started_at = "2026-03-02T14:30:00Z"
finished_at = ""

[[claude_sessions]]
id = "session_old789"
label = "initial exploration"
status = "done"
started_at = "2026-02-28T09:00:00Z"
finished_at = "2026-02-28T17:00:00Z"

# Fleeting notes (append-only)
[[notes]]
content = "picking up from step 4 of the migration plan"
created_at = "2026-03-03T10:15:00Z"

[[notes]]
content = "blocked on API key, asked Sarah"
created_at = "2026-03-03T14:30:00Z"

# Saved tmux state (captured by `devs save`)
# This section is optional — only present if the user has run `devs save`
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


## Field reference

### `[project]`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | yes | Project identifier (used as tmux session name) |
| `path` | string | yes | Absolute path to the project directory |
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

### `[[claude_sessions]]`

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

### `[last_state]`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `captured_at` | string | yes | ISO 8601 timestamp of when the state was saved |
| `layout_string` | string | yes | tmux layout string (from `list-windows -F '#{window_layout}'`) |

### `[[last_state.panes]]`

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `index` | integer | yes | Pane index within the window |
| `path` | string | yes | Working directory of the pane |
| `command` | string | yes | Foreground command running in the pane |


## Rust struct mapping

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct ProjectConfig {
    project: Project,
    layout: Option<Layout>,
    #[serde(default)]
    claude_sessions: Vec<ClaudeSession>,
    #[serde(default)]
    notes: Vec<Note>,
    last_state: Option<SavedState>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Project {
    name: String,
    path: String,
    color: Option<String>,
    created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Layout {
    main: MainPane,
    panes: Vec<SplitPane>,
    layout_string: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MainPane {
    cmd: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SplitPane {
    split: SplitDirection,   // "right", "bottom", "bottom-right"
    cmd: Option<String>,
    size: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeSession {
    id: String,
    label: String,
    status: String,      // "active" or "done"
    started_at: String,
    finished_at: String,  // "" if active
}

#[derive(Debug, Serialize, Deserialize)]
struct Note {
    content: String,
    created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SavedState {
    captured_at: String,
    layout_string: String,
    panes: Vec<SavedPane>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SavedPane {
    index: u32,
    path: String,
    command: String,
}
```
