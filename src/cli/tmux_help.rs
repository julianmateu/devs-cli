pub fn run() {
    print!("{TMUX_REFERENCE}");
}

const TMUX_REFERENCE: &str = "\
tmux quick reference (default prefix: Ctrl-b)

Panes
  %            split vertically (left/right)
  \"            split horizontally (top/bottom)
  arrow        move to pane in direction
  o            cycle to next pane
  x            close current pane
  z            toggle zoom (fullscreen pane)
  {            swap pane left
  }            swap pane right
  Ctrl-arrow   resize pane in direction
  !            break pane into its own window
  q            show pane numbers (press number to jump)

Windows
  c            create new window
  n            next window
  p            previous window
  0-9          switch to window by number
  ,            rename current window
  &            close current window
  w            list windows (interactive picker)
  l            toggle to last active window

Sessions
  d            detach from session
  s            list sessions (interactive picker)
  $            rename current session
  (            switch to previous session
  )            switch to next session

Copy mode (enter with prefix + [)
  [            enter copy mode
  q            exit copy mode
  /            search forward
  ?            search backward
  n            next search match
  N            previous search match
  Space        start selection
  Enter        copy selection and exit copy mode
  g            go to top
  G            go to bottom
";
