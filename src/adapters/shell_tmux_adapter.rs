use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::domain::saved_state::SavedPane;
use crate::ports::tmux_adapter::TmuxAdapter;

pub struct ShellTmuxAdapter;

fn normalize_command(raw_args: &str) -> String {
    if raw_args.is_empty() {
        return String::new();
    }

    let (first, rest) = raw_args.split_once(' ').unwrap_or((raw_args, ""));

    // Detect node wrappers: node /path/to/npm-cli.js, npx-cli.js, yarn-*.cjs
    if first == "node" || first.ends_with("/node") {
        if let Some(script) = rest.split_whitespace().next() {
            let filename = script.rsplit('/').next().unwrap_or(script);
            let trailing = rest.split_once(' ').map(|(_, t)| t).unwrap_or("");
            if filename.starts_with("npm-cli") {
                return format!("npm {trailing}").trim().to_string();
            }
            if filename.starts_with("npx-cli") {
                return format!("npx {trailing}").trim().to_string();
            }
            if filename.starts_with("yarn-") && filename.ends_with(".cjs") {
                return format!("yarn {trailing}").trim().to_string();
            }
        }
        // Plain node script — normalize the node path itself
        if first.contains('/') {
            if rest.is_empty() {
                return "node".to_string();
            }
            return format!("node {rest}");
        }
        return raw_args.to_string();
    }

    // Strip absolute path from first arg
    if first.contains('/') {
        let basename = first.rsplit('/').next().unwrap_or(first);
        if rest.is_empty() {
            return basename.to_string();
        }
        return format!("{basename} {rest}");
    }

    raw_args.to_string()
}

/// Pure logic: given optional pgrep output and a function to get ps args for a pid,
/// determine the best command string for a pane.
fn resolve_from_children(
    child_pids: Option<&str>,
    ps_args_for_pid: impl Fn(&str) -> Option<String>,
    fallback: &str,
) -> String {
    let pids: Vec<&str> = child_pids.unwrap_or("").split_whitespace().collect();

    if pids.is_empty() {
        return fallback.to_string();
    }

    match ps_args_for_pid(pids[0]) {
        Some(raw) if !raw.is_empty() => normalize_command(&raw),
        _ => fallback.to_string(),
    }
}

fn resolve_pane_command(shell_pid: u32, fallback_command: &str) -> String {
    let pgrep = Command::new("pgrep")
        .args(["-P", &shell_pid.to_string()])
        .output();

    let pgrep_output = match pgrep {
        Ok(output) if output.status.success() => {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
        _ => None,
    };

    resolve_from_children(
        pgrep_output.as_deref(),
        |pid| {
            let ps = Command::new("ps")
                .args(["-o", "args=", "-p", pid])
                .output()
                .ok()?;
            if ps.status.success() {
                Some(String::from_utf8_lossy(&ps.stdout).trim().to_string())
            } else {
                None
            }
        },
        fallback_command,
    )
}

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

    fn split_window(
        &self,
        target: &str,
        horizontal: bool,
        size: Option<&str>,
        path: Option<&str>,
    ) -> Result<()> {
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
            "#{pane_index}\t#{pane_current_path}\t#{pane_current_command}\t#{pane_pid}",
        ])?;
        let panes = output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                match parts.len() {
                    4 => {
                        let index = parts[0].parse().ok()?;
                        let path = parts[1].to_string();
                        let fallback = parts[2];
                        let pid: u32 = parts[3].parse().ok()?;
                        let command = resolve_pane_command(pid, fallback);
                        Some(SavedPane {
                            index,
                            path,
                            command,
                        })
                    }
                    3 => Some(SavedPane {
                        index: parts[0].parse().ok()?,
                        path: parts[1].to_string(),
                        command: parts[2].to_string(),
                    }),
                    _ => None,
                }
            })
            .collect();
        Ok(panes)
    }

    fn apply_layout(&self, name: &str, layout_string: &str) -> Result<()> {
        self.run_tmux(&["select-layout", "-t", name, layout_string])
    }

    fn kill_session(&self, name: &str) -> Result<()> {
        self.run_tmux(&["kill-session", "-t", name])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_npm() {
        assert_eq!(
            normalize_command("node /usr/local/lib/node_modules/npm/bin/npm-cli.js run dev"),
            "npm run dev"
        );
    }

    #[test]
    fn normalize_npx() {
        assert_eq!(
            normalize_command("node /usr/local/lib/node_modules/npm/bin/npx-cli.js some-tool"),
            "npx some-tool"
        );
    }

    #[test]
    fn normalize_yarn() {
        assert_eq!(
            normalize_command("node /home/user/.yarn/releases/yarn-4.0.cjs dev"),
            "yarn dev"
        );
    }

    #[test]
    fn normalize_plain_node_script() {
        assert_eq!(normalize_command("node server.js"), "node server.js");
    }

    #[test]
    fn normalize_absolute_node_binary() {
        assert_eq!(
            normalize_command("/usr/local/bin/node server.js"),
            "node server.js"
        );
        assert_eq!(normalize_command("/usr/local/bin/node"), "node");
    }

    #[test]
    fn normalize_non_node_passthrough() {
        assert_eq!(
            normalize_command("cargo watch -x test"),
            "cargo watch -x test"
        );
    }

    #[test]
    fn normalize_absolute_path() {
        assert_eq!(normalize_command("/usr/local/bin/nvim"), "nvim");
    }

    #[test]
    fn normalize_empty() {
        assert_eq!(normalize_command(""), "");
    }

    #[test]
    fn resolve_no_children_returns_fallback() {
        assert_eq!(resolve_from_children(None, |_| None, "zsh"), "zsh");
        assert_eq!(resolve_from_children(Some(""), |_| None, "zsh"), "zsh");
    }

    #[test]
    fn resolve_one_child_returns_normalized() {
        let result = resolve_from_children(
            Some("12345"),
            |pid| {
                assert_eq!(pid, "12345");
                Some("node /usr/lib/npm/bin/npm-cli.js run dev".to_string())
            },
            "zsh",
        );
        assert_eq!(result, "npm run dev");
    }

    #[test]
    fn resolve_multiple_children_uses_first() {
        let result = resolve_from_children(
            Some("111\n222\n333"),
            |pid| {
                if pid == "111" {
                    Some("cargo watch -x test".to_string())
                } else {
                    panic!("should only query first pid");
                }
            },
            "zsh",
        );
        assert_eq!(result, "cargo watch -x test");
    }

    #[test]
    fn resolve_ps_failure_returns_fallback() {
        let result = resolve_from_children(Some("12345"), |_| None, "bash");
        assert_eq!(result, "bash");
    }

    #[test]
    fn resolve_ps_empty_output_returns_fallback() {
        let result = resolve_from_children(Some("12345"), |_| Some(String::new()), "bash");
        assert_eq!(result, "bash");
    }
}
