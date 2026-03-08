use serde::{Deserialize, Serialize};

use crate::domain::saved_state::SavedPane;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SplitDirection {
    Right,
    Bottom,
    BottomRight,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MainPane {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SplitPane {
    pub split: SplitDirection,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Layout {
    pub main: MainPane,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub panes: Vec<SplitPane>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_string: Option<String>,
}

pub(crate) const SHELL_COMMANDS: &[&str] = &[
    "zsh", "bash", "sh", "fish", "dash", "ksh", "tcsh", "csh", "nu", "pwsh",
];

impl Layout {
    pub fn from_snapshot(layout_string: String, panes: &[SavedPane]) -> Self {
        let main_cmd = panes
            .first()
            .filter(|p| !SHELL_COMMANDS.contains(&p.command.as_str()))
            .map(|p| p.command.clone());

        let split_panes: Vec<SplitPane> = panes
            .iter()
            .skip(1)
            .map(|p| {
                let cmd = if SHELL_COMMANDS.contains(&p.command.as_str()) {
                    None
                } else {
                    Some(p.command.clone())
                };
                // SplitDirection::Right is a placeholder; when restoring a
                // snapshot the layout_string is the source of truth for pane
                // geometry, so the split direction here is not used.
                SplitPane {
                    split: SplitDirection::Right,
                    cmd,
                    size: None,
                }
            })
            .collect();

        Layout {
            main: MainPane { cmd: main_cmd },
            panes: split_panes,
            layout_string: Some(layout_string),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::test_helpers::assert_toml_roundtrip;

    #[test]
    fn layout_main_pane_only() {
        let layout = Layout {
            main: MainPane { cmd: None },
            panes: vec![],
            layout_string: None,
        };

        let expected = "[main]\n";

        assert_toml_roundtrip(&layout, expected);
    }

    #[test]
    fn layout_main_with_cmd_and_one_split() {
        let layout = Layout {
            main: MainPane {
                cmd: Some("nvim".to_string()),
            },
            panes: vec![SplitPane {
                split: SplitDirection::Right,
                cmd: Some("claude".to_string()),
                size: None,
            }],
            layout_string: None,
        };

        let expected = r#"[main]
cmd = "nvim"

[[panes]]
split = "right"
cmd = "claude"
"#;

        assert_toml_roundtrip(&layout, expected);
    }

    #[test]
    fn layout_multiple_splits_with_size() {
        let layout = Layout {
            main: MainPane {
                cmd: Some("nvim".to_string()),
            },
            panes: vec![
                SplitPane {
                    split: SplitDirection::Right,
                    cmd: Some("claude".to_string()),
                    size: Some("40%".to_string()),
                },
                SplitPane {
                    split: SplitDirection::BottomRight,
                    cmd: None,
                    size: None,
                },
            ],
            layout_string: None,
        };

        let expected = r#"[main]
cmd = "nvim"

[[panes]]
split = "right"
cmd = "claude"
size = "40%"

[[panes]]
split = "bottom-right"
"#;

        assert_toml_roundtrip(&layout, expected);
    }

    #[test]
    fn layout_all_split_directions() {
        let layout = Layout {
            main: MainPane { cmd: None },
            panes: vec![
                SplitPane {
                    split: SplitDirection::Right,
                    cmd: None,
                    size: None,
                },
                SplitPane {
                    split: SplitDirection::Bottom,
                    cmd: None,
                    size: None,
                },
                SplitPane {
                    split: SplitDirection::BottomRight,
                    cmd: None,
                    size: None,
                },
            ],
            layout_string: None,
        };

        let expected = r#"[main]

[[panes]]
split = "right"

[[panes]]
split = "bottom"

[[panes]]
split = "bottom-right"
"#;

        assert_toml_roundtrip(&layout, expected);
    }

    #[test]
    fn layout_with_layout_string_roundtrip() {
        let layout = Layout {
            main: MainPane {
                cmd: Some("nvim".to_string()),
            },
            panes: vec![SplitPane {
                split: SplitDirection::Right,
                cmd: Some("cargo watch".to_string()),
                size: None,
            }],
            layout_string: Some("5aed,176x79,0,0".to_string()),
        };

        let expected = r#"layout_string = "5aed,176x79,0,0"

[main]
cmd = "nvim"

[[panes]]
split = "right"
cmd = "cargo watch"
"#;

        assert_toml_roundtrip(&layout, expected);
    }

    #[test]
    fn from_snapshot_extracts_commands() {
        let panes = vec![
            SavedPane {
                index: 0,
                path: "/some/path".to_string(),
                command: "nvim".to_string(),
            },
            SavedPane {
                index: 1,
                path: "/some/path".to_string(),
                command: "zsh".to_string(),
            },
            SavedPane {
                index: 2,
                path: "/some/path".to_string(),
                command: "cargo watch".to_string(),
            },
        ];

        let layout = Layout::from_snapshot("5aed,176x79".to_string(), &panes);

        assert_eq!(layout.main.cmd, Some("nvim".to_string()));
        assert_eq!(layout.panes.len(), 2);
        assert_eq!(layout.panes[0].cmd, None); // zsh is a shell
        assert_eq!(layout.panes[1].cmd, Some("cargo watch".to_string()));
    }

    #[test]
    fn from_snapshot_skips_all_shell_commands() {
        let panes = vec![
            SavedPane {
                index: 0,
                path: "/p".to_string(),
                command: "bash".to_string(),
            },
            SavedPane {
                index: 1,
                path: "/p".to_string(),
                command: "fish".to_string(),
            },
        ];

        let layout = Layout::from_snapshot("layout".to_string(), &panes);

        assert_eq!(layout.main.cmd, None);
        assert_eq!(layout.panes.len(), 1);
        assert_eq!(layout.panes[0].cmd, None);
    }

    #[test]
    fn from_snapshot_single_pane() {
        let panes = vec![SavedPane {
            index: 0,
            path: "/p".to_string(),
            command: "nvim".to_string(),
        }];

        let layout = Layout::from_snapshot("layout".to_string(), &panes);

        assert_eq!(layout.main.cmd, Some("nvim".to_string()));
        assert!(layout.panes.is_empty());
    }

    #[test]
    fn from_snapshot_empty_panes() {
        let layout = Layout::from_snapshot("layout".to_string(), &[]);

        assert_eq!(layout.main.cmd, None);
        assert!(layout.panes.is_empty());
    }

    #[test]
    fn from_snapshot_with_full_command_strings() {
        let panes = vec![
            SavedPane {
                index: 0,
                path: "/project".to_string(),
                command: "nvim".to_string(),
            },
            SavedPane {
                index: 1,
                path: "/project".to_string(),
                command: "npm run dev".to_string(),
            },
            SavedPane {
                index: 2,
                path: "/project".to_string(),
                command: "claude:code-review".to_string(),
            },
        ];

        let layout = Layout::from_snapshot("layout-str".to_string(), &panes);

        assert_eq!(layout.main.cmd, Some("nvim".to_string()));
        assert_eq!(layout.panes.len(), 2);
        assert_eq!(layout.panes[0].cmd, Some("npm run dev".to_string()));
        assert_eq!(layout.panes[1].cmd, Some("claude:code-review".to_string()));
    }

    #[test]
    fn from_snapshot_recognizes_extended_shells() {
        let panes = vec![
            SavedPane {
                index: 0,
                path: "/p".to_string(),
                command: "nu".to_string(),
            },
            SavedPane {
                index: 1,
                path: "/p".to_string(),
                command: "pwsh".to_string(),
            },
            SavedPane {
                index: 2,
                path: "/p".to_string(),
                command: "dash".to_string(),
            },
        ];

        let layout = Layout::from_snapshot("layout".to_string(), &panes);

        assert_eq!(layout.main.cmd, None, "nu should be recognized as a shell");
        assert_eq!(
            layout.panes[0].cmd, None,
            "pwsh should be recognized as a shell"
        );
        assert_eq!(
            layout.panes[1].cmd, None,
            "dash should be recognized as a shell"
        );
    }

    #[test]
    fn from_snapshot_stores_layout_string() {
        let layout = Layout::from_snapshot("5aed,176x79,0,0".to_string(), &[]);

        assert_eq!(layout.layout_string, Some("5aed,176x79,0,0".to_string()));
    }
}
