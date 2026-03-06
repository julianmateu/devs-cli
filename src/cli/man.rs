use std::path::Path;

use anyhow::{Context, Result};
use clap::CommandFactory;

pub fn run(output_dir: &Path) -> Result<()> {
    let cmd = super::Cli::command();
    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("failed to create directory '{}'", output_dir.display()))?;
    clap_mangen::generate_to(cmd, output_dir).context("failed to generate man pages")?;
    eprintln!(
        "# Man pages generated in '{}'. See 'devs generate-man --help' for installation instructions.",
        output_dir.display()
    );
    Ok(())
}
