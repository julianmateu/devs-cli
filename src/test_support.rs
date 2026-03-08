use std::cell::RefCell;
use std::collections::HashMap;

use anyhow::{Result, bail};

use crate::domain::project::ProjectConfig;
use crate::domain::saved_state::SavedPane;
use crate::ports::process_launcher::ProcessLauncher;
use crate::ports::project_repository::ProjectRepository;
use crate::ports::terminal_adapter::TerminalAdapter;
use crate::ports::tmux_adapter::TmuxAdapter;

pub struct InMemoryProjectRepository {
    configs: RefCell<HashMap<String, ProjectConfig>>,
}

impl InMemoryProjectRepository {
    pub fn new() -> Self {
        Self {
            configs: RefCell::new(HashMap::new()),
        }
    }
}

impl ProjectRepository for InMemoryProjectRepository {
    fn load(&self, name: &str) -> Result<ProjectConfig> {
        self.configs
            .borrow()
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("project '{name}' not found"))
    }

    fn save(&self, config: &ProjectConfig) -> Result<()> {
        self.configs
            .borrow_mut()
            .insert(config.project.name.clone(), config.clone());
        Ok(())
    }

    fn list(&self) -> Result<Vec<String>> {
        let mut names: Vec<String> = self.configs.borrow().keys().cloned().collect();
        names.sort();
        Ok(names)
    }

    fn delete(&self, name: &str) -> Result<()> {
        if self.configs.borrow_mut().remove(name).is_none() {
            bail!("project '{name}' not found");
        }
        Ok(())
    }
}

pub struct MockTmuxAdapter {
    pub has_session: bool,
    pub layout_string: String,
    pub panes: Vec<SavedPane>,
    pub fail_on_get_layout: bool,
    calls: RefCell<Vec<String>>,
}

impl MockTmuxAdapter {
    pub fn with_session(layout_string: &str, panes: Vec<SavedPane>) -> Self {
        Self {
            has_session: true,
            layout_string: layout_string.to_string(),
            panes,
            fail_on_get_layout: false,
            calls: RefCell::new(vec![]),
        }
    }

    pub fn no_session() -> Self {
        Self {
            has_session: false,
            layout_string: String::new(),
            panes: vec![],
            fail_on_get_layout: false,
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

    fn split_window(
        &self,
        target: &str,
        horizontal: bool,
        size: Option<&str>,
        path: Option<&str>,
    ) -> Result<()> {
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
        if self.fail_on_get_layout {
            anyhow::bail!("mock: get_layout failed");
        }
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

    fn kill_session(&self, name: &str) -> Result<()> {
        self.calls
            .borrow_mut()
            .push(format!("kill_session({name})"));
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

    fn set_tab_title(&self, title: &str) -> Result<()> {
        self.calls
            .borrow_mut()
            .push(format!("set_tab_title({title})"));
        Ok(())
    }

    fn reset_tab_title(&self) -> Result<()> {
        self.calls
            .borrow_mut()
            .push("reset_tab_title()".to_string());
        Ok(())
    }
}

pub struct MockProcessLauncher {
    calls: RefCell<Vec<String>>,
}

impl MockProcessLauncher {
    pub fn new() -> Self {
        Self {
            calls: RefCell::new(vec![]),
        }
    }

    pub fn calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }
}

impl ProcessLauncher for MockProcessLauncher {
    fn launch_claude(&self, args: &[&str], working_dir: &str) -> Result<()> {
        let args_str = args.join(", ");
        self.calls
            .borrow_mut()
            .push(format!("launch_claude([{args_str}], {working_dir})"));
        Ok(())
    }
}
