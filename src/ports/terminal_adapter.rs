use anyhow::Result;

pub trait TerminalAdapter {
    fn set_tab_color(&self, hex: &str) -> Result<()>;
    fn reset_tab_color(&self) -> Result<()>;
    fn set_tab_title(&self, title: &str) -> Result<()>;
    fn reset_tab_title(&self) -> Result<()>;
}
