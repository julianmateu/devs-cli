use std::io::{self, Write};

use anyhow::{Context, Result};

use crate::ports::terminal_adapter::TerminalAdapter;

pub struct ItermTerminalAdapter;

impl ItermTerminalAdapter {
    pub fn new() -> Self {
        Self
    }
}

fn in_tmux() -> bool {
    std::env::var("TMUX").is_ok_and(|v| !v.is_empty())
}

fn emit_osc(payload: &str) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    if in_tmux() {
        write!(stdout, "\x1bPtmux;\x1b\x1b]{payload}\x07\x1b\\")?;
    } else {
        write!(stdout, "\x1b]{payload}\x07")?;
    }
    stdout.flush()
}

impl TerminalAdapter for ItermTerminalAdapter {
    fn set_tab_color(&self, hex: &str) -> Result<()> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        emit_osc(&format!("1337;SetColors=tab={hex}"))
            .context("failed to set tab color")
    }

    fn reset_tab_color(&self) -> Result<()> {
        emit_osc("1337;SetColors=tab=default").context("failed to reset tab color")
    }
}
