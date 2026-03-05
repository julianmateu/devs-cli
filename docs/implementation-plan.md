# Implementation Plan

## Crate dependencies

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
toml = "0.8"
chrono = { version = "0.4", features = ["serde"] }
dirs = "6"                    # XDG paths (~/.config)
uuid = { version = "1", features = ["v4"] }  # session IDs
anyhow = "1"                  # error handling
```

No async needed — all operations are synchronous CLI + tmux commands.


## Module structure (Clean Architecture)

The codebase follows ports-and-adapters (hexagonal) architecture. Domain logic is pure and has no I/O dependencies. Infrastructure concerns are behind traits.

```
src/
├── main.rs                          # Composition root: wire adapters, run CLI
│
├── domain/                          # Pure types and business logic. No I/O.
│   ├── mod.rs
│   ├── project.rs                   # Project, ClaudeSession, Note structs + validation
│   └── layout.rs                    # Layout, PaneConfig, SplitDirection types
│
├── ports/                           # Trait definitions (interfaces)
│   ├── mod.rs
│   ├── project_repository.rs        # trait ProjectRepository { load, save, list, delete }
│   ├── tmux_adapter.rs              # trait TmuxAdapter { has_session, create, attach, ... }
│   └── terminal_adapter.rs          # trait TerminalAdapter { set_tab_color, reset }
│
├── adapters/                        # Infrastructure implementations
│   ├── mod.rs
│   ├── toml_project_repository.rs   # TOML file read/write in ~/.config/devs/
│   ├── shell_tmux_adapter.rs        # Shells out to `tmux` commands
│   └── iterm_terminal_adapter.rs    # OSC escape sequences for tab colors
│
└── cli/                             # clap definitions + command handlers
    ├── mod.rs                       # Clap derive structs (Cli, Commands enum)
    ├── new.rs                       # devs new
    ├── list.rs                      # devs list
    ├── status.rs                    # devs status
    ├── open.rs                      # devs open
    ├── save.rs                      # devs save
    ├── claude.rs                    # devs claude, claudes, claude-done
    └── notes.rs                     # devs note, notes
```

**Dependency rules:**
- `domain/` → nothing (pure Rust + serde)
- `ports/` → `domain/` (traits reference domain types)
- `adapters/` → `domain/` + `ports/` (implement traits using domain types)
- `cli/` → `domain/` + `ports/` (handlers receive `dyn Trait`, never concrete adapters)
- `main.rs` → everything (constructs concrete adapters, wires them into CLI handlers)

**Testing strategy:**
- Domain logic: unit tests with plain structs, no mocks needed
- CLI handlers: unit tests with in-memory trait implementations (e.g., `InMemoryProjectRepository`)
- Adapters: integration tests against temp directories (`tempfile` crate) or manual testing (tmux)
- End-to-end: `assert_cmd` crate for CLI smoke tests


## Phase 1: Project scaffolding + CRUD (COMPLETE)

**Goal**: `devs new`, `devs list`, `devs status`, `devs config`, `devs edit`, `devs remove`

### Steps

1. **Scaffold the Rust project**
   - `cargo init devs-cli`
   - Add dependencies to `Cargo.toml`
   - Set up module structure (`domain/`, `ports/`, `adapters/`, `cli/`)
   - Set up `main.rs` with clap derive

2. **Domain types** (`domain/project.rs`) — TDD
   - `Project` struct (name, path, color, created_at) with `Debug, Clone, PartialEq, Serialize, Deserialize`
   - `ClaudeSession` struct (id, label, status, started_at, finished_at)
   - `Note` struct (content, created_at)
   - `ProjectConfig` struct (project, layout, claude_sessions, notes, last_state)
   - Validation: project name (valid tmux session name), color format, path exists
   - Tests: struct construction, validation rules, serde round-trip (serialize to TOML and back)

3. **Repository port** (`ports/project_repository.rs`)
   - `trait ProjectRepository { load, save, list, delete }`
   - Takes/returns domain types

4. **TOML adapter** (`adapters/toml_project_repository.rs`) — TDD
   - Implements `ProjectRepository` against `~/.config/devs/projects/`
   - Config dir path injected via constructor (for testability — tests use temp dirs)
   - Tests: write + read round-trip, list, delete, missing file error, invalid TOML error

5. **CLI commands** (`cli/`, `main.rs`)
   - Clap derive structs for `new`, `list`, `config`, `edit`, `remove`
   - Handlers receive `&dyn ProjectRepository`
   - `main.rs` constructs `TomlProjectRepository` with real config dir, passes to handlers

6. **Basic `devs status`** (`cli/status.rs`)
   - List projects from repository
   - Check `tmux has-session` for each (this touches tmux — use the `TmuxAdapter` trait even if the full implementation comes in Phase 2, so the architecture is clean from the start)
   - Show last note and active Claude session count


## Phase 2: tmux session management (COMPLETE)

**Goal**: `devs open`, `devs save`, `devs reset`

### Steps

1. **Implement tmux wrappers** (`tmux.rs`)
   - `has_session(name) -> bool`
   - `attach(name)`
   - `create_session(name, path)` — `tmux new-session -d -s <name> -c <path>`
   - `split_pane(target, direction, size, path, cmd)` — `tmux split-window ...`
   - `send_keys(target, keys)` — `tmux send-keys ... C-m`
   - `get_layout(name) -> String` — `tmux list-windows -F '#{window_layout}'`
   - `get_panes(name) -> Vec<PaneInfo>` — `tmux list-panes -F ...`
   - `apply_layout(name, layout_string)` — `tmux select-layout`

2. **Implement `devs open`** — the core command
   - Check if session exists → attach
   - Check for `last_state` → prompt or use flags
   - Create from declarative layout:
     - First pane: `new-session -d -s <name> -c <path>`
     - Subsequent panes: iterate `layout.panes`, run `split-window` for each
     - Send `cmd` via `send-keys` for each pane with a command
   - Create from saved state:
     - Create session with enough panes (N-1 splits)
     - Apply saved layout string
     - Send saved commands
   - Set tab color
   - Print Claude session hints
   - Attach

3. **Implement `devs save`**
   - Call tmux to capture layout string and pane states
   - Write to `[last_state]` in the project TOML

4. **Implement `devs reset`**
   - Remove `[last_state]` section from TOML

5. **Implement tab color** (`tab_color.rs`)
   - `set_tab_color(hex)` — parse hex, emit OSC 1337 with tmux passthrough
   - Called during `devs open` if project has a color


## Phase 3: Claude session tracking (COMPLETE)

**Goal**: `devs claude`, `devs claudes`, `devs claude-done`

### Steps

1. **Implement Claude session CRUD** (`claude.rs`)
   - `add_session(project, id, label)` — append to `claude_sessions` in TOML
   - `list_sessions(project, include_done) -> Vec<ClaudeSession>`
   - `mark_done(project, session_id)` — set status and finished_at

2. **Implement `devs claude <name> <label>`**
   - Generate a session ID (UUID v4 or let Claude assign one)
   - Record in project TOML
   - Exec `claude` (replace current process or spawn child)
   - Challenge: capturing Claude's own session ID if we don't control it
     - Option A: generate our own ID, pass `--session-id <id>` to Claude (if supported)
     - Option B: launch Claude, capture output to find session ID (fragile)
     - Option C: let user manually register sessions — simplest for v1

3. **Implement `devs claude <name> --resume <id>`**
   - Exec `claude --resume <id>`

4. **Implement `devs claudes <name>`**
   - Load project TOML, filter by status, format as table

5. **Implement `devs claude-done <name> <id>`**
   - Load TOML, find session, update status and finished_at, save


## Phase 4: Notes (COMPLETE)

**Goal**: `devs note`, `devs notes`

### Steps

1. **Implement notes** (`notes.rs`)
   - `add_note(project, content)` — append to `notes` in TOML
   - `list_notes(project, limit, since) -> Vec<Note>`
   - `clear_notes(project)` — remove all notes from TOML

2. **Wire up CLI commands**
   - `devs note <name> <message>`
   - `devs notes <name> [--all] [--since <duration>] [--clear]`


## Phase 5: Polish (COMPLETE)

**Goal**: Error handling, edge cases, remaining v1 features

**Additions since initial plan:**
- `devs new --from <project>`: copy layout from existing project
- `devs new --session LABEL:ID`: pre-populate Claude sessions
- `devs open`: `claude:<label>` expansion in layout pane commands
- `devs remove --kill`: kill tmux session before removing
- `devs status`: live dashboard with tmux liveness, active Claude count, last note
- `devs list`: shows project paths alongside names

### Steps

1. **Error handling**
   - Friendly error messages for: tmux not installed, project not found, invalid config
   - Validate project path exists on `devs new`
   - Validate color format on `devs new`

2. **Edge cases**
   - Project name validation (no spaces, no special chars — must be valid tmux session name)
   - Handle `~` expansion in paths
   - Handle missing `$EDITOR` for `devs edit`

3. **README and publishing**
   - Write README.md for the repo (install, usage, examples)
   - Add LICENSE
   - Publish to GitHub


## Open questions

1. **Claude session ID capture**: How does Claude Code assign session IDs? Can we pass `--session-id` to control it? If not, what's the best way to capture it? For v1, we may need to let the user register sessions manually (`devs claude-register rmbs-tool <id> "label"`).

2. **Multiple windows**: The current design assumes one tmux window per project. Should we support multiple windows in the layout config? tmuxinator does this. For v1, one window is sufficient.

3. **Binary name**: `devs` is short and memorable. Check for conflicts on crates.io and PATH. Alternatives: `hq`, `bench`, `sess`, `proj`.

4. **Shell completions**: clap can generate shell completions. Worth adding in v1 for a good developer experience.
