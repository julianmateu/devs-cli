use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::domain::saved_state::SavedPane;
use crate::ports::tmux_adapter::TmuxAdapter;

pub struct ShellTmuxAdapter;

impl ShellTmuxAdapter {
    fn run_tmux(&self, args: &[&str]) -> Result<()> {
        let status = Command::new("tmux")
            .args(args)
            .status()
            .with_context(|| format!("failed to run tmux {}", args[0]))?;
        if !status.success() {
            bail!("tmux {} failed", args[0]);
        }
        Ok(())
    }

    fn run_tmux_output(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("tmux")
            .args(args)
            .output()
            .with_context(|| format!("failed to run tmux {}", args[0]))?;
        if !output.status.success() {
            bail!("tmux {} failed", args[0]);
        }
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }
}

impl TmuxAdapter for ShellTmuxAdapter {
    fn has_session(&self, name: &str) -> bool {
        Command::new("tmux")
            .args(["has-session", "-t", name])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn create_session(&self, name: &str, path: &str) -> Result<()> {
        self.run_tmux(&["new-session", "-d", "-s", name, "-c", path])
    }

    fn attach(&self, name: &str) -> Result<()> {
        let inside_tmux = std::env::var("TMUX").is_ok();
        let cmd = if inside_tmux {
            "switch-client"
        } else {
            "attach-session"
        };
        self.run_tmux(&[cmd, "-t", name])
    }

    fn split_window(&self, target: &str, horizontal: bool, size: Option<&str>, path: Option<&str>) -> Result<()> {
        let flag = if horizontal { "-h" } else { "-v" };
        let pct;
        let mut args = vec!["split-window", flag, "-t", target];
        if let Some(s) = size {
            pct = s.trim_end_matches('%').to_string();
            args.extend(["-p", &pct]);
        }
        if let Some(p) = path {
            args.extend(["-c", p]);
        }
        self.run_tmux(&args)
    }

    fn send_keys(&self, target: &str, keys: &str) -> Result<()> {
        self.run_tmux(&["send-keys", "-t", target, keys, "C-m"])
    }

    fn select_pane(&self, target: &str) -> Result<()> {
        self.run_tmux(&["select-pane", "-t", target])
    }

    fn get_layout(&self, name: &str) -> Result<String> {
        self.run_tmux_output(&["list-windows", "-t", name, "-F", "#{window_layout}"])
    }

    fn get_panes(&self, name: &str) -> Result<Vec<SavedPane>> {
        let output = self.run_tmux_output(&[
            "list-panes",
            "-t",
            name,
            "-F",
            "#{pane_index}\t#{pane_current_path}\t#{pane_current_command}",
        ])?;
        let panes = output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() == 3 {
                    Some(SavedPane {
                        index: parts[0].parse().ok()?,
                        path: parts[1].to_string(),
                        command: parts[2].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();
        Ok(panes)
    }

    fn apply_layout(&self, name: &str, layout_string: &str) -> Result<()> {
        self.run_tmux(&["select-layout", "-t", name, layout_string])
    }
}
