use anyhow::Result;

use crate::ports::terminal_adapter::TerminalAdapter;

pub struct NoopTerminalAdapter;

impl TerminalAdapter for NoopTerminalAdapter {
    fn set_tab_color(&self, _hex: &str) -> Result<()> {
        Ok(())
    }

    fn reset_tab_color(&self) -> Result<()> {
        Ok(())
    }

    fn set_tab_title(&self, _title: &str) -> Result<()> {
        Ok(())
    }

    fn reset_tab_title(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_adapter_succeeds() {
        let adapter = NoopTerminalAdapter;
        assert!(adapter.set_tab_color("#ff0000").is_ok());
        assert!(adapter.reset_tab_color().is_ok());
        assert!(adapter.set_tab_title("test").is_ok());
        assert!(adapter.reset_tab_title().is_ok());
    }
}
