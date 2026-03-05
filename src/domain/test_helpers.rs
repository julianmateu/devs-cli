use std::cell::RefCell;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Serialize};

use crate::domain::saved_state::SavedPane;
use crate::ports::terminal_adapter::TerminalAdapter;
use crate::ports::tmux_adapter::TmuxAdapter;

pub fn assert_toml_roundtrip<T>(value: &T, expected_toml: &str)
where
    T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let serialized = toml::to_string(&value).unwrap();
    assert_eq!(serialized, expected_toml);

    let deserialized: T = toml::from_str(&serialized).unwrap();
    assert_eq!(&deserialized, value);
}

pub fn dt(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s).unwrap().to_utc()
}

pub struct MockTmuxAdapter {
    pub has_session: bool,
    pub layout_string: String,
    pub panes: Vec<SavedPane>,
    calls: RefCell<Vec<String>>,
}

impl MockTmuxAdapter {
    pub fn with_session(layout_string: &str, panes: Vec<SavedPane>) -> Self {
        Self {
            has_session: true,
            layout_string: layout_string.to_string(),
            panes,
            calls: RefCell::new(vec![]),
        }
    }

    pub fn no_session() -> Self {
        Self {
            has_session: false,
            layout_string: String::new(),
            panes: vec![],
            calls: RefCell::new(vec![]),
        }
    }

    pub fn calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }
}

impl TmuxAdapter for MockTmuxAdapter {
    fn has_session(&self, _name: &str) -> bool {
        self.has_session
    }

    fn create_session(&self, name: &str, path: &str) -> Result<()> {
        self.calls
            .borrow_mut()
            .push(format!("create_session({name}, {path})"));
        Ok(())
    }

    fn attach(&self, name: &str) -> Result<()> {
        self.calls.borrow_mut().push(format!("attach({name})"));
        Ok(())
    }

    fn split_window(&self, target: &str, horizontal: bool, size: Option<&str>, path: Option<&str>) -> Result<()> {
        let dir = if horizontal { "horizontal" } else { "vertical" };
        let size = size.unwrap_or("-");
        let path = path.unwrap_or("-");
        self.calls
            .borrow_mut()
            .push(format!("split_window({target}, {dir}, {size}, {path})"));
        Ok(())
    }

    fn send_keys(&self, target: &str, keys: &str) -> Result<()> {
        self.calls
            .borrow_mut()
            .push(format!("send_keys({target}, {keys})"));
        Ok(())
    }

    fn select_pane(&self, target: &str) -> Result<()> {
        self.calls
            .borrow_mut()
            .push(format!("select_pane({target})"));
        Ok(())
    }

    fn get_layout(&self, _name: &str) -> Result<String> {
        Ok(self.layout_string.clone())
    }

    fn get_panes(&self, _name: &str) -> Result<Vec<SavedPane>> {
        Ok(self.panes.clone())
    }

    fn apply_layout(&self, name: &str, layout_string: &str) -> Result<()> {
        self.calls
            .borrow_mut()
            .push(format!("apply_layout({name}, {layout_string})"));
        Ok(())
    }
}

pub struct MockTerminalAdapter {
    calls: RefCell<Vec<String>>,
}

impl MockTerminalAdapter {
    pub fn new() -> Self {
        Self {
            calls: RefCell::new(vec![]),
        }
    }

    pub fn calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }
}

impl TerminalAdapter for MockTerminalAdapter {
    fn set_tab_color(&self, hex: &str) -> Result<()> {
        self.calls
            .borrow_mut()
            .push(format!("set_tab_color({hex})"));
        Ok(())
    }

    fn reset_tab_color(&self) -> Result<()> {
        self.calls
            .borrow_mut()
            .push("reset_tab_color()".to_string());
        Ok(())
    }
}
