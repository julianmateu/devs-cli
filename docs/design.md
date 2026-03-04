# Design Decisions

## 1. Storage: TOML files, not SQLite

**Decision**: One TOML file per project in `~/.config/devs/projects/`, not a SQLite database.

**Rationale**:
- Human-readable and hand-editable (users will want to tweak layouts)
- Debuggable: `cat ~/.config/devs/projects/my-project.toml`
- No dependency on rusqlite or migration tooling
- Comments supported (unlike JSON)
- No footguns (unlike YAML's implicit type coercion: `NO` -> `false`, `3.10` -> `3.1`)
- Native to the Rust ecosystem (Cargo.toml precedent)
- One file per project means no risk of corrupting all projects at once
- No concurrency concerns (single user, single CLI)

**Trade-off**: Less queryable than SQLite for cross-project queries. Acceptable because we'll have <50 projects.

**Migration path**: If we ever need SQLite, the data model maps directly — same structs, different backend.

### Why not JSON?
- No comments (TOML supports them)
- Trailing comma errors when hand-editing
- Noisier syntax (quoted keys, braces everywhere)
- Less readable for config-style data

### Why not YAML?
- Implicit type coercion footguns (`country: NO` -> `false`, `version: 3.10` -> `3.1`)
- Indentation-sensitive (error-prone when hand-editing)
- `serde_yaml` is less battle-tested than `toml` in the Rust ecosystem


## 2. Storage location: `$HOME`, not the project

**Decision**: All devs-cli data lives in `~/.config/devs/`, not in individual project repositories.

```
~/.config/devs/
├── config.toml              # global defaults
└── projects/
    ├── rmbs-tool.toml
    └── my-api.toml
```

**Rationale**:
- This is cross-project orchestration metadata — it doesn't belong in any single project
- tmux layouts, Claude session IDs, and tab colors are personal dev environment state, not project state
- Different machines may have different layouts for the same project
- Keeps project repos clean (no `.devs.toml` cluttering git)
- Follows XDG Base Directory convention

**Optional**: A future version could support an optional `.devs.toml` in the project root for shareable defaults (layout, color) that are committed to git. The `~/.config/devs/` file always takes precedence.


## 3. No explicit `close` command

**Decision**: No `devs close` command. tmux manages its own session lifecycle.

**Rationale**:
- tmux sessions survive terminal/tab closure and detach. That's tmux's whole point.
- `devs open` is idempotent: if the tmux session exists, attach to it. If not, create it.
- Claude session IDs are recorded when they're *launched*, not when they're "closed".
- The user can kill tmux sessions directly (`tmux kill-session -t name`) — we don't need to wrap that.

**Session lifecycle**:
```
devs new      -> register project (metadata + default layout)
devs open     -> create or attach tmux session
devs status   -> show projects + which have live tmux sessions
devs save     -> snapshot current tmux state for later restore
devs note     -> append a timestamped note
```

`devs status` checks tmux liveness via `tmux has-session -t <name>` at query time.


## 4. Two-layer layout system

**Decision**: Layouts have two representations that coexist:
1. **Declarative config** (human-editable, the baseline)
2. **Runtime snapshot** (captured from a live tmux session)

**Rationale**: There is a tension between:
- Wanting a clean, editable config ("always open my project with nvim + terminal + claude")
- Wanting to capture ad-hoc changes ("I split a new pane and resized things")

Both are valid. Rather than forcing one model, we support both:

- `devs open foo` uses the declarative layout to create a fresh session
- The user works, adds panes, resizes — tmux handles this natively
- `devs save foo` captures the current tmux state (layout string + pane commands) as `last_state`
- `devs open foo` (after reboot) asks: "Restore last session layout or use default?"
  - Or: use `--default` / `--saved` flags to skip the prompt
- `devs reset foo` discards the saved state, reverts to declarative config

**Declarative layout format** (in the TOML config):
```toml
[[layout.panes]]
cmd = "nvim"
split = "main"

[[layout.panes]]
cmd = "claude"
split = "right"

[[layout.panes]]
split = "bottom-right"
```

Split values for v1: `main` (first pane), `right` (vertical split), `bottom` (horizontal split of current pane), `bottom-right` (horizontal split of the rightmost pane). Percentages can be added later (`right:40%`).

**Runtime snapshot format** (captured by `devs save`):
```toml
[last_state]
layout_string = "5aed,176x79,0,0[176x59,0,0,0,176x19,0,60{87x19,0,60,1,88x19,88,60,2}]"
captured_at = "2026-03-03T14:30:00Z"

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


## 5. Claude sessions: separate from panes

**Decision**: Claude Code sessions are tracked as a flat list per project, not tied to specific panes.

**Rationale**: Coupling sessions to panes creates an impossible tracking problem:
- User starts Claude session A in pane 3
- User finishes session A, starts session B in the same pane
- User splits pane 3, moves to a new pane, starts session C
- User closes the pane entirely

Tracking all of this requires real-time process monitoring, which is fragile and complex.

Instead, Claude sessions and pane layouts are **independent concerns**:
- **Pane layout** = visual arrangement (tracked via tmux layout strings)
- **Claude sessions** = logical conversations (tracked by ID + label + status)

**Claude session lifecycle**:
- `devs claude foo "brainstorm architecture"` — launches Claude, records session ID with label
- `devs claudes foo` — lists active sessions
- `devs claude-done foo <id>` — marks a session as done (hidden from default list)
- `devs claudes foo --all` — shows all sessions including done

On restore, `devs open` prints active Claude session IDs as hints:
```
# Active Claude sessions for rmbs-tool:
#   devs claude rmbs-tool --resume abc123   "brainstorm architecture"
#   devs claude rmbs-tool --resume def456   "implement step 4"
```

The user decides which session to resume in which pane. This is simpler, more reliable, and actually more useful — after a reboot, you might want sessions in different panes.


## 6. Notes: append-only scratchpad

**Decision**: Notes are a simple append-only log with clear/filter. No status tracking.

**Rationale**: Notes in devs-cli are **fleeting mental breadcrumbs**, not tasks:
- "picking up from step 4 of the migration"
- "blocked on API key, asked Sarah"
- "need to check if the race condition is fixed"

The moment you add `--done` flags and status tracking, you're building a todo app and competing with purpose-built tools (GitHub Issues, Linear, etc.).

**Operations**:
- `devs note foo "message"` — append with timestamp
- `devs notes foo` — show last 20 notes
- `devs notes foo --all` — show all notes
- `devs notes foo --since 2d` — filter by time
- `devs notes foo --clear` — wipe all notes


## 7. Tab colors: escape sequences, not iTerm2 API

**Decision**: Set tab colors via OSC escape sequences, not the iTerm2 Python API.

**Rationale**:
- No dependency on iTerm2 — works with any terminal that supports OSC 1337 or OSC 6
- No Python runtime dependency
- Silently ignored by terminals that don't support it
- Simple: just write bytes to stdout
- Automatically wrapped in DCS passthrough when inside tmux (`$TMUX` check)

**Method**: OSC 1337 SetColors (atomic, no per-channel flicker):
```
\x1b]1337;SetColors=tab=RRGGBB\x07
```

See [reference/iterm2-colors.md](reference/iterm2-colors.md) for full details.
