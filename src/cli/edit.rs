use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

pub fn run(name: &str, config_dir: &Path) -> Result<()> {
    let path = config_dir.join("projects").join(format!("{name}.toml"));
    if !path.exists() {
        bail!("project '{name}' not found");
    }
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    Command::new(&editor)
        .arg(&path)
        .status()
        .with_context(|| format!("failed to launch editor '{editor}'"))?;
    Ok(())
}
