use serde::{Deserialize, Serialize};

use crate::domain::layout::Layout;
use crate::domain::project::{ProjectError, validate_hex_color};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<Layout>,
}

impl LocalConfig {
    pub fn validate(&self) -> Result<(), ProjectError> {
        if let Some(color) = &self.color {
            validate_hex_color(color)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::layout::{MainPane, SplitDirection, SplitPane};
    use crate::domain::test_helpers::assert_toml_roundtrip;

    #[test]
    fn roundtrip_full_config() {
        let config = LocalConfig {
            color: Some("#61afef".to_string()),
            layout: Some(Layout {
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
            }),
        };

        let expected = r##"color = "#61afef"

[layout.main]
cmd = "nvim"

[[layout.panes]]
split = "right"
cmd = "claude"
size = "40%"

[[layout.panes]]
split = "bottom-right"
"##;

        assert_toml_roundtrip(&config, expected);
    }

    #[test]
    fn roundtrip_color_only() {
        let config = LocalConfig {
            color: Some("#e06c75".to_string()),
            layout: None,
        };

        let expected = "color = \"#e06c75\"\n";

        assert_toml_roundtrip(&config, expected);
    }

    #[test]
    fn roundtrip_layout_only() {
        let config = LocalConfig {
            color: None,
            layout: Some(Layout {
                main: MainPane {
                    cmd: Some("nvim".to_string()),
                },
                panes: vec![],
                layout_string: None,
            }),
        };

        let expected = "[layout.main]\ncmd = \"nvim\"\n";

        assert_toml_roundtrip(&config, expected);
    }

    #[test]
    fn deserialize_empty_toml() {
        let config: LocalConfig = toml::from_str("").unwrap();
        assert_eq!(config.color, None);
        assert_eq!(config.layout, None);
    }

    #[test]
    fn validate_accepts_valid_color() {
        let config = LocalConfig {
            color: Some("#61afef".to_string()),
            layout: None,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn validate_accepts_no_color() {
        let config = LocalConfig {
            color: None,
            layout: None,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn validate_rejects_invalid_color() {
        let config = LocalConfig {
            color: Some("banana".to_string()),
            layout: None,
        };
        assert!(matches!(
            config.validate(),
            Err(crate::domain::project::ProjectError::InvalidColor { .. })
        ));
    }
}
