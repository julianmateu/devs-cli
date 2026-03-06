use serde::{Deserialize, Serialize};

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

pub(crate) const SHELL_COMMANDS: &[&str] = &["zsh", "bash", "sh", "fish"];

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
}
