use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::ports::process_launcher::ProcessLauncher;

pub struct OsProcessLauncher;

impl ProcessLauncher for OsProcessLauncher {
    fn launch_claude(&self, args: &[&str], working_dir: &str) -> Result<()> {
        let status = Command::new("claude")
            .args(args)
            .current_dir(working_dir)
            .status()
            .context("failed to launch claude — is it installed?")?;
        if !status.success() {
            bail!("claude exited with {}", status);
        }
        Ok(())
    }
}
