use anyhow::Result;

pub trait ProcessLauncher {
    fn launch_claude(&self, args: &[&str], working_dir: &str) -> Result<()>;
}
