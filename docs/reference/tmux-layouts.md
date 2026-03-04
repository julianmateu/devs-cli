# tmux Layout Mechanics Reference

Reference documentation for tmux layout management, pane state capture, and programmatic session creation/restoration.

## 1. Split Commands

### `split-window` (alias: `splitw`)

Splits the current active pane into two panes. The new pane runs a new shell (or an optional command).

#### Direction Flags

| Flag | Direction | New Pane Position | Default |
|------|-----------|-------------------|---------|
| `-v` | Vertical split | Below the current pane | Yes (default if no flag) |
| `-h` | Horizontal split | Right of the current pane |  |

Note: The naming is counterintuitive. `-h` creates a **horizontal divider line** visually, but tmux calls it "horizontal" because panes are arranged **left-right**. `-v` creates a **vertical divider line**, arranging panes **top-bottom**.

#### Size Flags

| Flag | Description | Example |
|------|-------------|---------|
| `-l N` | Exact size in lines (vertical) or columns (horizontal) | `splitw -l 20` (20-line pane below) |
| `-l N%` | Percentage of available space (tmux 3.1+) | `splitw -l 30%` |
| `-p N` | Percentage of available space | `splitw -p 25` (25% of current pane) |

#### Placement Flags

| Flag | Description | Example |
|------|-------------|---------|
| `-b` | Place new pane before (above/left) instead of after (below/right) | `splitw -hb` (new pane on the left) |
| `-f` | Full-width/height split spanning the entire window | `splitw -f` (full-width pane at bottom) |
| `-d` | Do not make the new pane the active pane | `splitw -d` (focus stays on original) |

#### Target and Directory

| Flag | Description | Example |
|------|-------------|---------|
| `-t target` | Split a specific pane instead of the active one | `splitw -t mysession:1.2` |
| `-c path` | Set the working directory of the new pane | `splitw -c /home/user/project` |

#### Practical Examples

```bash
# Split active pane vertically, new pane gets 30% of the space
tmux split-window -v -p 30

# Split pane 0 horizontally, new pane gets 40 columns
tmux split-window -h -l 40 -t 0

# Full-width pane at the top of the window
tmux split-window -bf

# Split and run a command in the new pane (without changing focus)
tmux split-window -d -c /var/log 'tail -f syslog'
```

Flags can be combined: `splitw -hl 20` is equivalent to `splitw -h -l 20`.


## 2. Built-in Layouts

tmux provides five preset layouts. Apply them with `select-layout`:

```bash
tmux select-layout even-horizontal
```

Or cycle through them with `Prefix + Space`.

### Layout Descriptions

**`even-horizontal`** -- All panes side by side, equal widths.

```
+-------+-------+-------+
|       |       |       |
|   0   |   1   |   2   |
|       |       |       |
+-------+-------+-------+
```

**`even-vertical`** -- All panes stacked, equal heights.

```
+---------------------+
|         0           |
+---------------------+
|         1           |
+---------------------+
|         2           |
+---------------------+
```

**`main-horizontal`** -- One large pane on top, remaining panes arranged in a row below.

```
+---------------------+
|                     |
|         0           |
|                     |
+------+------+-------+
|  1   |  2   |   3   |
+------+------+-------+
```

Configure the main pane height:

```bash
tmux set-window-option main-pane-height 30
```

**`main-vertical`** -- One large pane on the left, remaining panes stacked on the right.

```
+------------+--------+
|            |   1    |
|            +--------+
|     0      |   2    |
|            +--------+
|            |   3    |
+------------+--------+
```

Configure the main pane width:

```bash
tmux set-window-option main-pane-width 100
```

**`tiled`** -- Panes distributed as evenly as possible in a grid.

```
+----------+----------+
|    0     |    1     |
|          |          |
+----------+----------+
|    2     |    3     |
|          |          |
+----------+----------+
```


## 3. Layout Strings

tmux represents window layouts internally as compact strings. These are the strings you see in `list-windows` output and can pass to `select-layout` to restore an exact pane arrangement.

### Capturing a Layout String

```bash
# Show layout for all windows in current session
tmux list-windows -F '#{window_index}: #{window_layout}'

# Get layout of the active window only
tmux list-windows -F '#{window_active} #{window_layout}' | grep '^1' | cut -d' ' -f2

# Get layout for a specific window
tmux list-windows -t mysession -F '#{window_index} #{window_layout}' | grep '^0 ' | cut -d' ' -f2
```

### Restoring a Layout String

```bash
tmux select-layout 'bb62,159x48,0,0{79x48,0,0,0,79x48,80,0,1}'
```

The target window must already have the correct number of panes. `select-layout` only rearranges geometry; it does not create or destroy panes.

### Format Specification

A layout string has the structure:

```
CHECKSUM,WxH,xoff,yoff[,pane_id | CONTAINER]
```

#### Top-level Example

```
bb62,159x48,0,0{79x48,0,0,0,79x48,80,0,1}
```

Breaking this down:

| Part | Meaning |
|------|---------|
| `bb62` | 16-bit checksum (4 hex chars) |
| `159x48` | Total window dimensions: 159 columns x 48 rows |
| `0,0` | Window offset (always 0,0 for the root) |
| `{...}` | Container with horizontal arrangement (left-right) |

#### Container Types

| Notation | Layout Type | Meaning |
|----------|-------------|---------|
| `{...}` | `LAYOUT_LEFTRIGHT` | Children arranged side by side (horizontal split) |
| `[...]` | `LAYOUT_TOPBOTTOM` | Children stacked vertically (vertical split) |

#### Cell Format

Each cell (leaf pane or nested container) has the format:

```
WxH,xoff,yoff,pane_id
```

| Field | Description |
|-------|-------------|
| `WxH` | Cell dimensions (width x height in character cells) |
| `xoff,yoff` | Position offset from the top-left of the window |
| `pane_id` | Numeric pane identifier (leaf nodes only) |

#### Nested Example

```
5aed,176x79,0,0[176x59,0,0,0,176x19,0,60{87x19,0,60,1,88x19,88,60,2}]
```

Parsed:

```
5aed                        -- checksum
176x79,0,0                  -- root: 176 cols x 79 rows at (0,0)
[                           -- top-bottom container
  176x59,0,0,0              -- pane 0: 176x59 at (0,0)  [top pane]
  176x19,0,60               -- nested container at (0,60)
  {                         -- left-right container
    87x19,0,60,1            -- pane 1: 87x19 at (0,60)   [bottom-left]
    88x19,88,60,2           -- pane 2: 88x19 at (88,60)  [bottom-right]
  }
]
```

Visual representation:

```
+----------------------------------+
|                                  |
|           pane 0                 |
|          176x59                  |
|                                  |
+----------------+-----------------+
|    pane 1      |     pane 2      |
|    87x19       |     88x19       |
+----------------+-----------------+
```

#### Checksum Algorithm

The checksum is a 16-bit value computed over the layout string (everything after the checksum and comma). The algorithm from tmux source (`layout-custom.c`):

```bash
tmux_layout_checksum() {
    local layout="$1"
    local csum=0
    for (( i=0; i<${#layout}; i++ )); do
        csum=$(( (csum >> 1) + ((csum & 1) << 15) ))
        csum=$(( csum + $(LC_CTYPE=C printf '%d' "'${layout:$i:1}") ))
    done
    printf '%04x' $(( csum & 0xffff ))
}
```

Usage:

```bash
layout="176x79,0,0[176x59,0,0,0,176x19,0,60{87x19,0,60,1,88x19,88,60,2}]"
checksum=$(tmux_layout_checksum "$layout")
tmux select-layout "${checksum},${layout}"
```

The checksum must be correct or tmux will reject the layout string.


## 4. Pane State Capture

### Key Format Variables

Use `list-panes -F` to query pane state. The most useful format variables for session save/restore:

#### Identity and Position

| Variable | Description |
|----------|-------------|
| `#{pane_id}` | Unique pane ID (e.g., `%0`, `%5`). Stable for server lifetime. |
| `#{pane_index}` | Pane index within the window (0-based, can change on pane close) |
| `#{pane_active}` | `1` if this is the active pane in its window |
| `#{pane_width}` | Pane width in columns |
| `#{pane_height}` | Pane height in rows |
| `#{pane_left}` | Left edge column |
| `#{pane_top}` | Top edge row |
| `#{pane_right}` | Right edge column |
| `#{pane_bottom}` | Bottom edge row |

#### State (for save/restore)

| Variable | Description |
|----------|-------------|
| `#{pane_current_path}` | Working directory of the pane's foreground process |
| `#{pane_current_command}` | Name of the foreground command (e.g., `zsh`, `vim`, `node`) |
| `#{pane_start_command}` | Command the pane was started with |
| `#{pane_pid}` | PID of the first process in the pane |
| `#{pane_tty}` | Pseudo-terminal device path |
| `#{pane_dead}` | `1` if the pane's process has exited |
| `#{pane_title}` | Pane title (set by escape sequences) |

#### Window and Session Context

| Variable | Description |
|----------|-------------|
| `#{session_name}` | Session name |
| `#{window_index}` | Window index |
| `#{window_name}` | Window name |
| `#{window_layout}` | Layout string for the window (see Section 3) |
| `#{window_active}` | `1` if this is the active window |

### Practical Capture Commands

```bash
# Full snapshot of all panes across all sessions
tmux list-panes -a -F '#{session_name}\t#{window_index}\t#{window_name}\t#{window_layout}\t#{pane_index}\t#{pane_active}\t#{pane_current_path}\t#{pane_current_command}'

# Panes in the current window
tmux list-panes -F '#{pane_index}: [#{pane_current_command}] #{pane_current_path}'

# Panes in a specific session and window
tmux list-panes -t mysession:0 -F '#{pane_index} #{pane_current_path} #{pane_current_command}'
```

### Capture Pane Contents

To capture what is displayed in a pane's terminal buffer:

```bash
# Print pane contents to stdout
tmux capture-pane -t %0 -p

# Capture full scrollback history
tmux capture-pane -t %0 -p -S -

# Save to file
tmux capture-pane -t %0 -p > /tmp/pane-contents.txt
```


## 5. Session Save/Restore Workflow

### Save (Snapshot)

The full sequence to capture a session's state:

```bash
#!/bin/bash
SESSION="$1"
OUTFILE="$2"

# 1. Capture window-level data (one line per window)
tmux list-windows -t "$SESSION" -F \
  '#{window_index}|#{window_name}|#{window_layout}|#{window_active}' \
  > "${OUTFILE}.windows"

# 2. Capture pane-level data (one line per pane)
tmux list-panes -t "$SESSION" -a -F \
  '#{window_index}|#{pane_index}|#{pane_active}|#{pane_current_path}|#{pane_current_command}' \
  | grep "^" > "${OUTFILE}.panes"

# 3. Optionally capture pane contents
while IFS='|' read -r widx pidx pactive ppath pcmd; do
    tmux capture-pane -t "${SESSION}:${widx}.${pidx}" -p \
      > "${OUTFILE}.pane-${widx}-${pidx}.txt" 2>/dev/null
done < "${OUTFILE}.panes"
```

### Restore (Recreate)

The sequence to recreate a session from saved state:

```bash
#!/bin/bash
SESSION="$1"
OUTFILE="$2"

# 1. Create the session (first window is created automatically)
FIRST_WINDOW=true
while IFS='|' read -r widx wname wlayout wactive; do
    if $FIRST_WINDOW; then
        tmux new-session -d -s "$SESSION" -n "$wname" -x 200 -y 50
        FIRST_WINDOW=false
    else
        tmux new-window -t "${SESSION}:${widx}" -n "$wname"
    fi
done < "${OUTFILE}.windows"

# 2. Create panes in each window
#    Read panes grouped by window index. The first pane (index 0) already
#    exists from window creation. Additional panes need split-window.
while IFS='|' read -r widx pidx pactive ppath pcmd; do
    TARGET="${SESSION}:${widx}"
    if [ "$pidx" -eq 0 ]; then
        # First pane already exists, just set its directory
        tmux send-keys -t "${TARGET}.0" "cd '$ppath'" C-m
    else
        # Create additional panes by splitting
        tmux split-window -t "${TARGET}" -c "$ppath"
    fi
done < "${OUTFILE}.panes"

# 3. Apply saved layouts (this fixes the geometry after splits)
while IFS='|' read -r widx wname wlayout wactive; do
    tmux select-layout -t "${SESSION}:${widx}" "$wlayout"
done < "${OUTFILE}.windows"

# 4. Send commands to restart processes (optional, application-specific)
while IFS='|' read -r widx pidx pactive ppath pcmd; do
    TARGET="${SESSION}:${widx}.${pidx}"
    case "$pcmd" in
        vim|nvim) tmux send-keys -t "$TARGET" "$pcmd" C-m ;;
        node)     tmux send-keys -t "$TARGET" "node" C-m ;;
        # Add more process restoration rules as needed
    esac
done < "${OUTFILE}.panes"

# 5. Select the correct active windows and panes
while IFS='|' read -r widx wname wlayout wactive; do
    if [ "$wactive" = "1" ]; then
        tmux select-window -t "${SESSION}:${widx}"
    fi
done < "${OUTFILE}.windows"

while IFS='|' read -r widx pidx pactive ppath pcmd; do
    if [ "$pactive" = "1" ]; then
        tmux select-pane -t "${SESSION}:${widx}.${pidx}"
    fi
done < "${OUTFILE}.panes"

# 6. Attach
tmux attach-session -t "$SESSION"
```

### Critical Ordering

The restore sequence must follow this order:

1. **Create session** (`new-session -d`) -- creates the first window and pane
2. **Create additional windows** (`new-window`) -- one per saved window
3. **Create panes** (`split-window`) -- one per additional pane in each window
4. **Apply layouts** (`select-layout`) -- must happen after all panes exist
5. **Send commands** (`send-keys`) -- restore running processes
6. **Set active state** (`select-window`, `select-pane`) -- restore focus

Layout strings encode pane IDs from the original session. When restoring, pane IDs will differ, but `select-layout` maps them by position (left-to-right, top-to-bottom order matches pane index order). The number of panes must match what the layout string expects.


## 6. Common Patterns (tmuxinator / tmuxp / scripts)

### The Standard Programmatic Session Creation Pattern

All tmux session managers (tmuxinator, tmuxp) and custom scripts follow the same fundamental sequence:

```bash
#!/bin/bash
SESSION="myproject"

# 1. Check if session exists; create if not
tmux has-session -t $SESSION 2>/dev/null
if [ $? != 0 ]; then

    # 2. Create detached session with first window
    tmux new-session -d -s $SESSION -n editor -c ~/projects/myapp

    # 3. Send initial command to the first pane
    tmux send-keys -t $SESSION:editor 'vim .' C-m

    # 4. Create additional panes by splitting
    tmux split-window -v -p 30 -t $SESSION:editor -c ~/projects/myapp
    tmux send-keys -t $SESSION:editor.1 'npm run dev' C-m

    # 5. Create additional windows
    tmux new-window -t $SESSION -n server -c ~/projects/myapp
    tmux send-keys -t $SESSION:server 'npm start' C-m

    # 6. Split the server window
    tmux split-window -h -p 50 -t $SESSION:server -c ~/projects/myapp
    tmux send-keys -t $SESSION:server.1 'tail -f logs/app.log' C-m

    # 7. Apply layout (built-in or custom string)
    tmux select-layout -t $SESSION:server even-horizontal

    # 8. Select starting window and pane
    tmux select-window -t $SESSION:editor
    tmux select-pane -t $SESSION:editor.0
fi

# 9. Attach
tmux attach-session -t $SESSION
```

### tmuxinator YAML Pattern

tmuxinator translates YAML configs into the same tmux command sequence:

```yaml
name: myproject
root: ~/projects/myapp
windows:
  - editor:
      layout: main-vertical
      panes:
        - vim .
        - npm run dev
  - server:
      layout: even-horizontal
      panes:
        - npm start
        - tail -f logs/app.log
  - console:
      panes:
        - rails console
```

Under the hood, tmuxinator generates and runs shell commands equivalent to:

```bash
tmux new-session -d -s myproject -n editor
tmux send-keys -t myproject:1 'cd ~/projects/myapp' C-m
tmux send-keys -t myproject:1 'vim .' C-m
tmux split-window -c ~/projects/myapp -t myproject:1
tmux select-layout -t myproject:1 main-vertical
tmux send-keys -t myproject:1.1 'npm run dev' C-m
# ... and so on for each window and pane
```

### Key Patterns

**Detached creation**: Always use `new-session -d` so the script can set everything up before attaching.

**Directory via `-c`**: Use `split-window -c PATH` and `new-window -c PATH` to set the working directory directly, rather than sending `cd` commands.

**Layout after splits**: Apply `select-layout` after all panes in a window are created. Applying it earlier may produce unexpected results because the layout distributes space among existing panes.

**`send-keys` with `C-m`**: The `C-m` at the end simulates pressing Enter. Without it the command text appears in the pane but does not execute.

**Idempotent startup**: Use `has-session` to check if the session already exists before creating it. This allows the script to be run multiple times safely.

**`wait-for` synchronization**: For commands that must complete before subsequent steps:

```bash
tmux send-keys -t work 'vagrant up; tmux wait-for -S vagrant-ready' C-m
tmux wait-for vagrant-ready
# Now vagrant is up, continue with dependent setup
```


## 7. Pane Identification

### ID Types

tmux uses three levels of identification, each with a prefix character:

| Object | Prefix | Example | Stability |
|--------|--------|---------|-----------|
| Session | `$` | `$3` | Stable for session lifetime |
| Window | `@` | `@1` | Stable for window lifetime |
| Pane | `%` | `%5` | Stable for pane lifetime (unique across server) |

These IDs are assigned by the tmux server and never reused within a server's lifetime. Pane IDs are set in the `TMUX_PANE` environment variable inside each pane.

### Pane Index vs Pane ID

| Concept | Format | Behavior |
|---------|--------|----------|
| **Pane ID** (`%N`) | `%0`, `%1`, `%5` | Globally unique, never reused, stable |
| **Pane index** (`N`) | `0`, `1`, `2` | Per-window, 0-based, renumbered when panes are closed |

### Target Syntax: `session:window.pane`

The `-t` flag accepts targets in this format:

```
[session_name]:[window_index|window_name][.pane_index]
```

| Target | Meaning |
|--------|---------|
| `mysession:2.1` | Pane 1 of window 2 in session "mysession" |
| `:2.1` | Pane 1 of window 2 in the current session |
| `.1` | Pane 1 in the current window |
| `2` | Pane 2 in the current window (or window 2 if no such pane) |
| `%5` | Pane with ID %5 (regardless of session/window) |
| `mysession:` | Current window of session "mysession" |

Special index modifiers:

| Modifier | Meaning |
|----------|---------|
| `+` / `-` | Next / previous pane index |
| `{last}` | Last (previously active) pane |
| `{top}`, `{bottom}`, `{left}`, `{right}` | Pane in the given direction |

### Discovering Pane IDs and Indices

```bash
# Show pane indices briefly on screen
tmux display-panes  # (or Prefix + q)

# List all panes with IDs and indices
tmux list-panes -F '#{pane_index} #{pane_id} #{pane_current_command} #{pane_current_path}'

# List all panes across all sessions
tmux list-panes -a -F '#{session_name}:#{window_index}.#{pane_index} (#{pane_id}) #{pane_current_command}'

# Get the current pane ID
tmux display-message -p '#{pane_id}'
```

### Pane Index Assignment

Pane indices are assigned based on position: top-to-bottom, then left-to-right. When a pane is closed, remaining panes are renumbered. This means pane indices are not stable identifiers for long-lived references -- use pane IDs (`%N`) for that.


## Appendix: Existing Session Save/Restore Tools

### tmux-resurrect

A tmux plugin that serializes the full environment to `~/.local/share/tmux/resurrect/` (or `~/.tmux/resurrect/`). Saves sessions, windows, panes, layouts, working directories, and optionally running programs and pane contents. Triggered manually with `Prefix + Ctrl-s` (save) and `Prefix + Ctrl-r` (restore).

### tmux-continuum

Companion to tmux-resurrect. Automatically saves every 15 minutes and can auto-restore on tmux server start.

### tmuxp

Python-based session manager. Uses YAML/JSON config files. Can freeze the current session state: `tmuxp freeze SESSION_NAME` produces a config file that can recreate the session.

### tmuxinator

Ruby-based session manager. Uses YAML config files. Does not capture existing sessions -- configs are authored manually. Generates shell scripts that run the standard tmux command sequence.
