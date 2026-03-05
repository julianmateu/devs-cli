use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedPane {
    pub index: u32,
    pub path: String,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedState {
    pub captured_at: DateTime<Utc>,
    pub layout_string: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub panes: Vec<SavedPane>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::test_helpers::{assert_toml_roundtrip, dt};

    #[test]
    fn saved_state_roundtrip() {
        let state = SavedState {
            captured_at: dt("2026-03-03T16:00:00Z"),
            layout_string: "5aed,176x79,0,0".to_string(),
            panes: vec![
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
            ],
        };

        let expected = r#"captured_at = "2026-03-03T16:00:00Z"
layout_string = "5aed,176x79,0,0"

[[panes]]
index = 0
path = "/some/path"
command = "nvim"

[[panes]]
index = 1
path = "/some/path"
command = "zsh"
"#;

        assert_toml_roundtrip(&state, expected);
    }

    #[test]
    fn saved_state_no_panes() {
        let state = SavedState {
            captured_at: dt("2026-03-03T16:00:00Z"),
            layout_string: "5aed,176x79,0,0".to_string(),
            panes: vec![],
        };

        let expected = r#"captured_at = "2026-03-03T16:00:00Z"
layout_string = "5aed,176x79,0,0"
"#;

        assert_toml_roundtrip(&state, expected);
    }
}
