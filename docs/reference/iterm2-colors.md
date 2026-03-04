# iTerm2 Tab Color Escape Sequences

Reference documentation for setting iTerm2 tab colors via escape sequences, including
tmux passthrough and cross-terminal compatibility notes.

## Method 1: OSC 6 (Per-Channel RGB)

This is the most commonly used method. Each RGB channel is set with a separate escape
sequence. The value range is 0-255 per channel.

### Format

```
ESC ] 6 ; 1 ; bg ; red ; brightness ; N BEL
ESC ] 6 ; 1 ; bg ; green ; brightness ; N BEL
ESC ] 6 ; 1 ; bg ; blue ; brightness ; N BEL
```

Where:
- `ESC` = `\x1b` (hex `0x1b`)
- `]` = literal `]` (this makes `ESC ]` the OSC introducer)
- `6` = the OSC number (iTerm2 proprietary, window/tab chrome)
- `1` = undocumented constant (always `1` in practice)
- `bg` = background (this controls the tab bar chrome, not the terminal background)
- `red` / `green` / `blue` = channel name
- `brightness` = literal keyword
- `N` = decimal integer 0-255
- `BEL` = `\x07` (alternatively, `ESC \` i.e. `\x1b\x5c` can be used as the ST terminator)

### Reset to Default

```
ESC ] 6 ; 1 ; bg ; * ; default BEL
```

This removes the custom tab color and restores the profile/system default.

### Raw Bytes

Setting tab to `#ff6600` (orange):

| Sequence | Hex Bytes |
|---|---|
| Set red=255 | `1b 5d 36 3b 31 3b 62 67 3b 72 65 64 3b 62 72 69 67 68 74 6e 65 73 73 3b 32 35 35 07` |
| Set green=102 | `1b 5d 36 3b 31 3b 62 67 3b 67 72 65 65 6e 3b 62 72 69 67 68 74 6e 65 73 73 3b 31 30 32 07` |
| Set blue=0 | `1b 5d 36 3b 31 3b 62 67 3b 62 6c 75 65 3b 62 72 69 67 68 74 6e 65 73 73 3b 30 07` |
| Reset | `1b 5d 36 3b 31 3b 62 67 3b 2a 3b 64 65 66 61 75 6c 74 07` |

### Shell Example

```bash
# Set tab to purple (#ff00ff)
printf '\033]6;1;bg;red;brightness;255\a'
printf '\033]6;1;bg;green;brightness;0\a'
printf '\033]6;1;bg;blue;brightness;255\a'

# Reset to default
printf '\033]6;1;bg;*;default\a'
```

## Method 2: OSC 1337 SetColors

This is iTerm2's general-purpose color-setting escape code. It accepts a hex color value
directly (no per-channel splitting needed).

### Format

```
ESC ] 1337 ; SetColors=tab=RRGGBB BEL
```

Where:
- `RRGGBB` = 6-digit hex color (e.g. `ff6600`), **no `#` prefix**
- Also accepts 3-digit shorthand `RGB` (e.g. `f60`)
- Optionally prefixed with a color space: `srgb:RRGGBB`, `p3:RRGGBB`, or `rgb:RRGGBB`
  - `srgb` = standard sRGB (the default if no prefix is given)
  - `p3` = Apple Display P3 wide gamut
  - `rgb` = Apple's legacy device-independent color space

### Reset to Default

```
ESC ] 1337 ; SetColors=tab=default BEL
```

### Shell Example

```bash
# Set tab to orange
printf '\033]1337;SetColors=tab=ff6600\a'

# Set tab with P3 wide gamut color
printf '\033]1337;SetColors=tab=p3:ff6600\a'

# Reset
printf '\033]1337;SetColors=tab=default\a'
```

### Other Settable Keys

The `SetColors` command also accepts these keys (useful for full theming, not just tabs):

```
fg bg bold link selbg selfg curbg curfg underline tab
black red green yellow blue magenta cyan white
br_black br_red br_green br_yellow br_blue br_magenta br_cyan br_white
```

Multiple can be set in one sequence: `SetColors=tab=ff0000=bg=000000`.

## tmux Passthrough

tmux intercepts and discards unrecognized escape sequences by default. There are two
approaches to get iTerm2 escape sequences through to the outer terminal.

### Approach 1: DCS Passthrough Wrapping

Wrap the escape sequence in a DCS (Device Control String) envelope. This works with
tmux 3.2+ when `allow-passthrough` is enabled.

**Format:**

```
ESC P tmux; ESCAPED_SEQUENCE ESC \
```

Where:
- `ESC P` = DCS introducer (`\x1b\x50`)
- `tmux;` = literal prefix that tells tmux to pass through
- `ESCAPED_SEQUENCE` = the original OSC sequence with **every `ESC` byte doubled**
- `ESC \` = ST terminator for the DCS (`\x1b\x5c`)

**The doubling rule**: Every `\x1b` inside the wrapped sequence must become `\x1b\x1b`.

**Example** -- wrapping `\033]6;1;bg;red;brightness;255\a`:

```
\033Ptmux;\033\033]6;1;bg;red;brightness;255\a\033\\
```

Broken down:
```
\033P          DCS start
tmux;          passthrough prefix
\033           doubled ESC (original \033 from OSC)
\033]          the ] that follows (now the inner ESC is doubled)
6;1;bg;red;brightness;255
\a             BEL terminator for inner sequence
\033\\         DCS end (ESC \)
```

**Wait** -- more precisely, the inner `\033]` becomes `\033\033]` (the ESC is doubled).
The BEL (`\a` / `\x07`) does NOT need doubling since only ESC bytes are doubled.

### Approach 2: `allow-passthrough` Option

Starting with tmux 3.3a, the `allow-passthrough` option can be configured to let
applications send escape sequences directly without DCS wrapping.

```bash
# In ~/.tmux.conf
set -g allow-passthrough on

# Or for all panes including invisible ones:
set -g allow-passthrough all
```

Values:
- `off` (default) -- passthrough sequences are silently discarded
- `on` -- passthrough works for visible panes only
- `all` -- passthrough works for all panes, including invisible/background ones

With `allow-passthrough on`, the DCS wrapping is still required. The option controls
whether tmux honors the `\033Ptmux;...\033\\` envelope at all.

**Important**: Even with `allow-passthrough`, you still need the DCS wrapper. The option
does not make tmux transparently forward raw OSC sequences. It only controls whether
the `\033Ptmux;` DCS envelope is processed or discarded.

### Shell Example (tmux-aware)

```bash
set_tab_color() {
    local r=$1 g=$2 b=$3

    if [ -n "$TMUX" ]; then
        # Inside tmux: wrap in DCS passthrough
        printf '\033Ptmux;\033\033]6;1;bg;red;brightness;%d\a\033\\' "$r"
        printf '\033Ptmux;\033\033]6;1;bg;green;brightness;%d\a\033\\' "$g"
        printf '\033Ptmux;\033\033]6;1;bg;blue;brightness;%d\a\033\\' "$b"
    else
        # Direct to iTerm2
        printf '\033]6;1;bg;red;brightness;%d\a' "$r"
        printf '\033]6;1;bg;green;brightness;%d\a' "$g"
        printf '\033]6;1;bg;blue;brightness;%d\a' "$b"
    fi
}

reset_tab_color() {
    if [ -n "$TMUX" ]; then
        printf '\033Ptmux;\033\033]6;1;bg;*;default\a\033\\'
    else
        printf '\033]6;1;bg;*;default\a'
    fi
}
```

## Cross-Terminal Compatibility

### WezTerm

- **OSC 6 tab color**: Listed as "Ignored" in WezTerm's escape sequence documentation.
  WezTerm recognizes the sequence but does not act on it.
- **OSC 1337 SetColors**: Not implemented for tab colors. WezTerm only supports OSC 1337
  for inline images (iTerm2 image protocol).
- **Tab colors**: WezTerm does not currently support setting tab colors via escape
  sequences. Tab appearance is controlled through Lua configuration only.

### Kitty

- **No escape sequence support** for tab colors. Kitty does not implement OSC 6 or
  OSC 1337 SetColors.
- **Remote control protocol**: Kitty supports `kitten @ set-tab-color` via its
  socket-based remote control protocol. This requires `allow_remote_control` and
  `listen_on` to be configured in `kitty.conf`. It is not an escape sequence -- it
  communicates over a Unix socket.
  ```bash
  kitten @ set-tab-color active_bg=#ff6600
  ```
- **OSC 21**: Kitty has its own color-control escape (OSC 21) for terminal foreground,
  background, cursor, etc., but this does not cover tab bar colors.

### Ghostty

- **No known support** for tab color escape sequences as of early 2026.
- Ghostty supports many standard escape sequences but iTerm2 proprietary codes for
  tab colors are not implemented.

### Terminal.app (macOS)

- Does not support OSC 6 or OSC 1337. These sequences are silently ignored.

### Summary Table

| Terminal | OSC 6 (tab) | OSC 1337 SetColors=tab | Other mechanism |
|---|---|---|---|
| iTerm2 | Yes | Yes | -- |
| WezTerm | Ignored | No | Lua config only |
| Kitty | No | No | `kitten @ set-tab-color` (socket) |
| Ghostty | No | No | None known |
| Terminal.app | No | No | None |

## Gotchas and Edge Cases

1. **Three sequences for OSC 6**: The color is not applied until all three channels
   (red, green, blue) are sent. If you only send red, the green and blue channels
   retain their previous values. There is a brief visual flash as channels are set
   sequentially. The OSC 1337 method avoids this by setting the color atomically.

2. **Hex format for OSC 1337**: Must be 6 digits (`ff6600`), not 3 digits shorthand.
   The 3-digit form (`f60`) is documented as accepted but some older builds may not
   parse it correctly. Use 6 digits for safety. Do NOT include a `#` prefix.

3. **Theme dependency**: The visual prominence of the tab color depends on the iTerm2
   theme setting (Minimal, Compact, Light, Dark). Minimal theme shows the most
   dramatic color fill. Light/Dark themes show a subtle tint.

4. **tmux detection**: Check `$TMUX` environment variable (not `$TERM`). The `$TERM`
   variable inside tmux is typically `screen-256color` or `tmux-256color`, but
   checking `$TMUX` is more reliable for detecting whether DCS wrapping is needed.

5. **tmux version requirements**: DCS passthrough requires tmux 3.2+. The
   `allow-passthrough` option was added in tmux 3.2 and defaults to `off` since
   tmux 3.3a. Older tmux versions silently discard the sequences.

6. **BEL vs ST terminator**: Both `\x07` (BEL) and `\x1b\x5c` (ESC \\) work as
   the string terminator. Inside DCS passthrough, using BEL for the inner sequence
   is simpler because it avoids ambiguity with the outer ESC \\ terminator.

7. **No persistence**: Tab colors are per-session and ephemeral. They are lost when
   the tab/session is closed or when iTerm2 is restarted. They are not saved in the
   profile.

8. **Multiple tabs**: The escape sequence affects whichever tab the emitting process
   is running in. You cannot set the color of a different tab via escape sequences.

9. **SSH sessions**: Tab colors set inside an SSH session work fine (the escape
   sequences pass through the SSH channel to the local terminal). But if you are also
   inside tmux on the remote host, you need DCS wrapping on the remote tmux too.

## Rust Implementation

### Simple Implementation (no tmux)

```rust
use std::io::{self, Write};

/// Set the iTerm2 tab color using OSC 6 (per-channel).
fn set_tab_color_osc6(r: u8, g: u8, b: u8) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    write!(stdout, "\x1b]6;1;bg;red;brightness;{r}\x07")?;
    write!(stdout, "\x1b]6;1;bg;green;brightness;{g}\x07")?;
    write!(stdout, "\x1b]6;1;bg;blue;brightness;{b}\x07")?;
    stdout.flush()
}

/// Set the iTerm2 tab color using OSC 1337 SetColors (atomic).
fn set_tab_color_osc1337(r: u8, g: u8, b: u8) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    write!(stdout, "\x1b]1337;SetColors=tab={r:02x}{g:02x}{b:02x}\x07")?;
    stdout.flush()
}

/// Reset the tab color to the profile default.
fn reset_tab_color() -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    write!(stdout, "\x1b]6;1;bg;*;default\x07")?;
    stdout.flush()
}
```

### tmux-Aware Implementation

```rust
use std::io::{self, Write};

/// Returns true if running inside tmux.
fn in_tmux() -> bool {
    std::env::var("TMUX").is_ok_and(|v| !v.is_empty())
}

/// Emit an OSC escape sequence, wrapping in DCS passthrough if inside tmux.
fn emit_osc(payload: &str) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    if in_tmux() {
        // DCS passthrough: ESC P tmux; ESC ESC ] payload BEL ESC backslash
        write!(stdout, "\x1bPtmux;\x1b\x1b]{payload}\x07\x1b\\")?;
    } else {
        write!(stdout, "\x1b]{payload}\x07")?;
    }
    stdout.flush()
}

/// Set the iTerm2 tab color (r, g, b each 0-255).
///
/// Uses the OSC 1337 SetColors method for atomic color changes (no flicker).
/// Automatically wraps in DCS passthrough when running inside tmux.
pub fn set_tab_color(r: u8, g: u8, b: u8) -> io::Result<()> {
    emit_osc(&format!("1337;SetColors=tab={r:02x}{g:02x}{b:02x}"))
}

/// Reset the tab color to the iTerm2 profile default.
pub fn reset_tab_color() -> io::Result<()> {
    emit_osc("1337;SetColors=tab=default")
}
```

### Full Implementation with Hex Parsing

```rust
use std::io::{self, Write};

/// An RGB color for terminal tab chrome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl TabColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parse a hex color string. Accepts formats:
    /// - `"ff6600"` (6-digit)
    /// - `"#ff6600"` (6-digit with hash)
    /// - `"f60"` (3-digit shorthand)
    /// - `"#f60"` (3-digit shorthand with hash)
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|e| format!("invalid red: {e}"))?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|e| format!("invalid green: {e}"))?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|e| format!("invalid blue: {e}"))?;
                Ok(Self { r, g, b })
            }
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16)
                    .map_err(|e| format!("invalid red: {e}"))?;
                let g = u8::from_str_radix(&hex[1..2], 16)
                    .map_err(|e| format!("invalid green: {e}"))?;
                let b = u8::from_str_radix(&hex[2..3], 16)
                    .map_err(|e| format!("invalid blue: {e}"))?;
                // Expand 3-digit to 6-digit: 0xf -> 0xff
                Ok(Self {
                    r: r << 4 | r,
                    g: g << 4 | g,
                    b: b << 4 | b,
                })
            }
            _ => Err(format!(
                "expected 3 or 6 hex digits, got {} chars: {hex:?}",
                hex.len()
            )),
        }
    }

    /// Returns the 6-digit hex representation (no `#` prefix).
    pub fn to_hex(self) -> String {
        format!("{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

/// Terminal environment detection.
fn in_tmux() -> bool {
    std::env::var("TMUX").is_ok_and(|v| !v.is_empty())
}

/// Write an OSC sequence to stdout, with DCS passthrough wrapping for tmux.
fn emit_osc(payload: &str) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    if in_tmux() {
        write!(stdout, "\x1bPtmux;\x1b\x1b]{payload}\x07\x1b\\")?;
    } else {
        write!(stdout, "\x1b]{payload}\x07")?;
    }
    stdout.flush()
}

/// Set the iTerm2 tab color.
pub fn set_tab_color(color: TabColor) -> io::Result<()> {
    emit_osc(&format!("1337;SetColors=tab={}", color.to_hex()))
}

/// Reset the iTerm2 tab color to the profile default.
pub fn reset_tab_color() -> io::Result<()> {
    emit_osc("1337;SetColors=tab=default")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_6_digit_hex() {
        let c = TabColor::from_hex("ff6600").unwrap();
        assert_eq!(c, TabColor::new(255, 102, 0));
    }

    #[test]
    fn parse_6_digit_hex_with_hash() {
        let c = TabColor::from_hex("#1a2b3c").unwrap();
        assert_eq!(c, TabColor::new(0x1a, 0x2b, 0x3c));
    }

    #[test]
    fn parse_3_digit_hex() {
        let c = TabColor::from_hex("f60").unwrap();
        assert_eq!(c, TabColor::new(0xff, 0x66, 0x00));
    }

    #[test]
    fn parse_3_digit_hex_with_hash() {
        let c = TabColor::from_hex("#abc").unwrap();
        assert_eq!(c, TabColor::new(0xaa, 0xbb, 0xcc));
    }

    #[test]
    fn roundtrip_hex() {
        let c = TabColor::new(255, 102, 0);
        assert_eq!(c.to_hex(), "ff6600");
        assert_eq!(TabColor::from_hex(&c.to_hex()).unwrap(), c);
    }

    #[test]
    fn reject_invalid_hex() {
        assert!(TabColor::from_hex("gg0000").is_err());
        assert!(TabColor::from_hex("ff00").is_err());
        assert!(TabColor::from_hex("").is_err());
    }
}
```

## References

- [iTerm2 Proprietary Escape Codes](https://iterm2.com/documentation-escape-codes.html) -- official documentation for OSC 6 and OSC 1337
- [tmux FAQ -- Passthrough](https://github.com/tmux/tmux/wiki/FAQ) -- DCS passthrough format
- [tmux allow-passthrough](https://tmuxai.dev/tmux-allow-passthrough/) -- configuration guide
- [WezTerm Escape Sequences](https://wezterm.org/escape-sequences.html) -- OSC 6 listed as "Ignored"
- [Kitty Color Control](https://sw.kovidgoyal.net/kitty/color-stack/) -- OSC 21 (no tab support)
- [Kitty set-tab-color](https://www.mankier.com/1/kitten-@-set-tab-color) -- socket-based remote control
- [iterm2-tab-color (bash)](https://github.com/connordelacruz/iterm2-tab-color) -- reference shell implementation
- [it2setcolor (shell)](https://github.com/fabioy/shell/blob/master/.iterm2/it2setcolor) -- iTerm2's bundled utility
