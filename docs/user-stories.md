# User Stories

## Core workflows

### 1. Register a new project

> As a developer, I want to register a project with its folder path so that `devs` knows about it.

```bash
devs new rmbs-tool --path ~/src/rmbs-tool --color "#e06c75"
```

Creates `~/.config/devs/projects/rmbs-tool.toml` with:
- Project name and path
- Optional tab color
- Default layout (can be customized later)


### 2. Open a project session

> As a developer, I want to open a project with a single command that creates a tmux session with my preferred pane layout.

```bash
devs open rmbs-tool
```

If tmux session `rmbs-tool` exists: attach to it.
If not: create it from the declarative layout config, set tab color, cd to project path.

With layout preference:
```bash
devs open rmbs-tool --default    # always use declarative layout
devs open rmbs-tool --saved      # always use last saved state (if available)
```


### 3. Check project status

> As a developer, I want to see all my projects and which ones have active tmux sessions.

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


### 4. Add a fleeting note

> As a developer, I want to jot down what I'm doing or what to do next, so I remember after context switches.

```bash
devs note rmbs-tool "picking up from step 4 of the migration plan"
devs note rmbs-tool "blocked on API key, asked Sarah"
```

View notes:
```bash
devs notes rmbs-tool             # last 20
devs notes rmbs-tool --all       # everything
devs notes rmbs-tool --since 2d  # last 2 days
devs notes rmbs-tool --clear     # wipe all
```


### 5. Launch and track a Claude session

> As a developer, I want to launch Claude Code within a project and have the session tracked, so I can find and resume it later.

```bash
devs claude rmbs-tool "brainstorm architecture"
```

This:
1. Records a new Claude session entry with a generated or captured session ID and the label
2. Launches `claude` in the current terminal (or a specified pane)

Resume an existing session:
```bash
devs claude rmbs-tool --resume abc123
```


### 6. View Claude sessions for a project

> As a developer, I want to see which Claude sessions are active for a project.

```bash
devs claudes rmbs-tool
```

Output:
```
ID        LABEL                      STATUS   STARTED
abc123    brainstorm architecture    active   2026-03-01 10:00
def456    implement step 4           active   2026-03-02 14:30
```

Show all (including done):
```bash
devs claudes rmbs-tool --all
```


### 7. Mark a Claude session as done

> As a developer, when I finish a line of work with a Claude session, I want to mark it done so it doesn't clutter my active list.

```bash
devs claude-done rmbs-tool abc123
```


### 8. Save current tmux state

> As a developer, after customizing my pane layout during a session, I want to save the current state so it can be restored exactly.

```bash
devs save rmbs-tool
```

Captures:
- tmux layout string (exact pane geometry)
- Each pane's working directory and current command
- Timestamp


### 9. Restore after reboot

> As a developer, after a machine restart, I want to get back to where I was with a single command.

```bash
devs open rmbs-tool
```

If saved state exists, offers to restore it:
```
Saved state from 2026-03-03 14:30 available. Restore? [Y/n/default]
  Y = restore saved layout
  n = use default layout
```

After creating the tmux session, prints Claude session hints:
```
Active Claude sessions:
  devs claude rmbs-tool --resume def456   "implement step 4"
  devs claude rmbs-tool --resume ghi789   "fix test suite"
```


### 10. Edit project config

> As a developer, I want to edit a project's layout, color, or other settings.

```bash
devs edit rmbs-tool          # opens TOML in $EDITOR
devs config rmbs-tool        # print current config to stdout
```


### 11. Reset to default layout

> As a developer, I want to discard my saved tmux state and go back to the declarative layout.

```bash
devs reset rmbs-tool
```


### 12. Remove a project

> As a developer, when I'm done with a project, I want to remove it from tracking.

```bash
devs remove rmbs-tool        # removes the TOML file
devs remove rmbs-tool --kill # also kills the tmux session if alive
```


## Stretch workflows (post-v1)

### 13. Save layout from current session as the new default

> As a developer, I want to capture my current tmux layout and make it the new declarative default.

```bash
devs save rmbs-tool --as-default
```

This captures the current layout and overwrites the `[[layout.panes]]` section in the TOML.


### 14. Create project from current tmux session

> As a developer, I already have a tmux session set up the way I like. I want to register it as a project.

```bash
devs new rmbs-tool --from-session    # captures current tmux session
```


### 15. List all projects

> As a developer, I want a quick list of all registered projects.

```bash
devs list
```

Output:
```
rmbs-tool    ~/src/rmbs-tool
my-api       ~/src/my-api
playground   ~/src/playground
```
