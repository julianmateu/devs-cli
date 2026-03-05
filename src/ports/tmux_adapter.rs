use anyhow::Result;

use crate::domain::saved_state::SavedPane;

pub trait TmuxAdapter {
    fn has_session(&self, name: &str) -> bool;
    fn create_session(&self, name: &str, path: &str) -> Result<()>;
    fn attach(&self, name: &str) -> Result<()>;
    fn split_window(
        &self,
        target: &str,
        horizontal: bool,
        size: Option<&str>,
        path: Option<&str>,
    ) -> Result<()>;
    fn send_keys(&self, target: &str, keys: &str) -> Result<()>;
    fn select_pane(&self, target: &str) -> Result<()>;
    fn get_layout(&self, name: &str) -> Result<String>;
    fn get_panes(&self, name: &str) -> Result<Vec<SavedPane>>;
    fn apply_layout(&self, name: &str, layout_string: &str) -> Result<()>;
}
