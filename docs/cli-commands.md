# CLI Commands

## Project management

### `devs new <name>`

Register a new project.

```bash
devs new rmbs-tool --path ~/src/rmbs-tool --color "#e06c75"
devs new my-api --path ~/src/my-api
```

| Flag | Required | Description |
|------|----------|-------------|
| `--path <path>` | yes | Absolute path to the project directory |
| `--color <hex>` | no | Tab color (`"#rrggbb"` or `"rrggbb"`) |
| `--from <project>` | no | Copy layout from an existing project |
| `--from-session <name>` | no | Capture layout from a live tmux session (conflicts with `--from`) |
| `--session <LABEL:ID>` | no | Pre-populate a Claude session (repeatable) |

```bash
devs new fork --path ~/src/fork --from rmbs-tool --session "implement:abc123"
devs new captured --path ~/src/captured --from-session my-api
```

Creates `~/.config/devs/projects/<name>.toml` with default layout.
When `--from` is specified, copies the layout from the source project (not notes or saved state).
When `--from-session` is specified, captures the exact pane geometry and commands from a live tmux session.
When `--session` is specified, creates active Claude sessions with the given label and ID.
Fails if a project with that name already exists.


### `devs list`

List all registered projects.

```bash
devs list
```

Output:
```
rmbs-tool    ~/src/rmbs-tool
my-api       ~/src/my-api
playground   ~/src/playground
```


### `devs status`

Show all projects with live status information.

```bash
devs status
```

Output:
```
PROJECT        PATH                        TMUX    CLAUDE   LAST NOTE
rmbs-tool      ~/src/rmbs-tool             alive   2 active "implement step 4"
my-api         ~/src/my-api                dead    0 active "waiting on PR review"
playground     ~/src/playground            alive   1 active --
```

Checks tmux session liveness via `tmux has-session -t <name>`.
Counts active (not done) Claude sessions.
Shows the most recent note (truncated).


### `devs remove <name>`

Remove a project from tracking.

```bash
devs remove rmbs-tool --force
devs remove rmbs-tool --force --kill    # also kill tmux session if alive
```

| Flag | Required | Description |
|------|----------|-------------|
| `--force` | yes | Confirm deletion |
| `--kill` | no | Kill the tmux session if alive before removing |

Deletes the TOML file. Requires `--force` to confirm.


### `devs edit <name>`

Open the project's TOML config in `$EDITOR`.

```bash
devs edit rmbs-tool
```


### `devs config <name>`

Print the project's current config to stdout.

```bash
devs config rmbs-tool
```


## Session management

### `devs open <name>`

Open (or attach to) a project's tmux session.

```bash
devs open rmbs-tool              # auto: use saved state if available, else default
devs open rmbs-tool --default    # always use declarative layout
devs open rmbs-tool --saved      # always use saved state (fail if none)
```

Behavior:
1. If tmux session `<name>` already exists → attach to it
2. If saved state exists → use saved state (unless `--default`/`--saved`)
3. Create tmux session from chosen layout
4. Set tab color via escape sequences
5. Print active Claude session hints
6. Attach to the session

**`claude:<label>` expansion**: In layout pane commands, `claude` and `claude:<label>` are expanded automatically:
- `claude` → starts a new Claude session with label `"default"`, or resumes if one exists
- `claude:brainstorm` → starts/resumes a Claude session with label `"brainstorm"`

This allows declarative layouts to include Claude sessions that persist across `devs open` invocations.

### `devs save <name>`

Snapshot the current tmux state for a project.

```bash
devs save rmbs-tool              # save to [last_state] (runtime snapshot)
devs save rmbs-tool --as-default # save as the declarative [layout] default
```

| Flag | Required | Description |
|------|----------|-------------|
| `--as-default` | no | Write captured layout as the declarative `[layout]` default |

Without `--as-default`: captures to `[last_state]` in the project's TOML file (runtime snapshot, overwrites any previous saved state).

With `--as-default`: captures the current pane geometry and commands and writes them as the declarative `[layout]` section. This replaces any existing layout. The `layout_string` field preserves exact pane geometry, while pane commands remain human-editable.


### `devs close <name>`

Close a project's tmux session. Optionally saves the layout before closing.

```bash
devs close rmbs-tool              # kill session, reset tab color
devs close rmbs-tool --save       # save layout, then kill session
```

| Flag | Required | Description |
|------|----------|-------------|
| `--save` | no | Save the current layout before closing |

Behavior:
1. Verify the project exists
2. Verify the tmux session is alive (error if not)
3. If `--save`: capture the current layout (same as `devs save`)
4. Kill the tmux session
5. Reset the tab color


### `devs reset <name>`

Discard saved tmux state, reverting to the declarative layout.

```bash
devs reset rmbs-tool
```

Removes the `[last_state]` section from the TOML file.


## Claude session tracking

### `devs claude <name> <label>`

Launch a new Claude Code session within a project.

```bash
devs claude rmbs-tool "brainstorm architecture"
```

1. Generates a session ID (or captures it from Claude)
2. Records it in the project's TOML with the label and status `active`
3. Launches `claude` in the current terminal

### `devs claude <name> --resume <id>`

Resume an existing Claude Code session.

```bash
devs claude rmbs-tool --resume abc123
```

Launches `claude --resume <id>`.


### `devs claudes <name>`

List Claude sessions for a project.

```bash
devs claudes rmbs-tool           # active sessions only
devs claudes rmbs-tool --all     # include done sessions
```

Output:
```
ID        LABEL                      STATUS   STARTED
abc123    brainstorm architecture    active   2026-03-01 10:00
def456    implement step 4           active   2026-03-02 14:30
```


### `devs claude-done <name> <session-id>`

Mark a Claude session as done.

```bash
devs claude-done rmbs-tool abc123
```

Sets `status = "done"` and `finished_at` to the current timestamp.


## Notes

### `devs note <name> <message>`

Add a timestamped note.

```bash
devs note rmbs-tool "picking up from step 4 of the migration"
```


### `devs notes <name>`

View notes for a project.

```bash
devs notes rmbs-tool             # last 20 notes
devs notes rmbs-tool --all       # all notes
devs notes rmbs-tool --since 2d  # notes from last 2 days
devs notes rmbs-tool --clear     # delete all notes
```

Output:
```
2026-03-03 14:30  blocked on API key, asked Sarah
2026-03-03 10:15  picking up from step 4 of the migration plan
```

Notes are displayed newest-first.


## Global

### `devs completions <shell>`

Generate shell completions for the given shell (bash, zsh, fish, elvish, powershell).

A one-line hint is printed to stderr after generation, so piping to a file works cleanly (`devs completions zsh > _devs`). Run `devs completions --help` to see full setup instructions.

#### Oh My Zsh

```bash
mkdir -p ${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/plugins/devs
devs completions zsh > ${ZSH_CUSTOM:-~/.oh-my-zsh/custom}/plugins/devs/_devs
```

Add `devs` to the `plugins=(...)` list in `~/.zshrc`, then restart your shell.

#### Vanilla zsh

```bash
mkdir -p ~/.zfunc
devs completions zsh > ~/.zfunc/_devs
```

Add to `~/.zshrc` (before `compinit`):

```zsh
fpath=(~/.zfunc $fpath)
autoload -Uz compinit && compinit
```

#### Bash

```bash
mkdir -p ~/.local/share/bash-completion/completions
devs completions bash > ~/.local/share/bash-completion/completions/devs
```

Completions are loaded automatically on the next shell start.

#### Fish

```bash
devs completions fish > ~/.config/fish/completions/devs.fish
```

Fish loads completions from this directory automatically.

### `devs --version`

Print version.

### `devs --help`

Print help for all commands.

### `devs <command> --help`

Print help for a specific command.
