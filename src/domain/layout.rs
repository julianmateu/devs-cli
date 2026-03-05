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
}
