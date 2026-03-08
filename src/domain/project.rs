use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::claude_session::ClaudeSession;
use super::layout::Layout;
use super::note::Note;
use super::saved_state::SavedState;

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("invalid project name '{name}': {reason}")]
    InvalidName { name: String, reason: String },

    #[error("invalid color '{color}': must be hex format #rrggbb or rrggbb")]
    InvalidColor { color: String },

    #[error("project path must not be empty")]
    EmptyPath,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub name: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
}

const INVALID_NAME_CHARS: &[char] = &['.', ':', ' '];

pub fn validate_hex_color(color: &str) -> Result<(), ProjectError> {
    let hex = color.strip_prefix('#').unwrap_or(color);
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ProjectError::InvalidColor {
            color: color.to_string(),
        });
    }
    Ok(())
}

impl ProjectMetadata {
    pub fn validate(&self) -> Result<(), ProjectError> {
        if self.name.is_empty() {
            return Err(ProjectError::InvalidName {
                name: self.name.clone(),
                reason: String::from("is empty"),
            });
        }
        if let Some(c) = self.name.chars().find(|c| INVALID_NAME_CHARS.contains(c)) {
            return Err(ProjectError::InvalidName {
                name: self.name.clone(),
                reason: format!("contains invalid character '{c}'"),
            });
        }
        if let Some(color) = &self.color {
            validate_hex_color(color)?;
        }
        if self.path.is_empty() {
            return Err(ProjectError::EmptyPath);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub project: ProjectMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<Layout>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub claude_sessions: Vec<ClaudeSession>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<Note>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_state: Option<SavedState>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::claude_session::ClaudeSessionStatus;
    use crate::domain::layout::{Layout, MainPane, SplitDirection, SplitPane};
    use crate::domain::saved_state::SavedPane;
    use crate::domain::test_helpers::{assert_toml_roundtrip, dt};

    #[test]
    fn minimal_project_config_roundtrip() {
        let config = ProjectConfig {
            project: ProjectMetadata {
                name: "test-project".to_string(),
                path: "/some/path".to_string(),
                color: None,
                created_at: dt("2026-03-03T10:00:00Z"),
            },
            layout: None,
            claude_sessions: vec![],
            notes: vec![],
            last_state: None,
        };

        let expected = r#"[project]
name = "test-project"
path = "/some/path"
created_at = "2026-03-03T10:00:00Z"
"#;

        assert_toml_roundtrip(&config, expected);
    }

    #[test]
    fn full_project_config_roundtrip() {
        let config = ProjectConfig {
            project: ProjectMetadata {
                name: "my-project".to_string(),
                path: "/home/user/src/my-project".to_string(),
                color: Some("#e06c75".to_string()),
                created_at: dt("2026-03-03T10:00:00Z"),
            },
            layout: Some(Layout {
                main: MainPane {
                    cmd: Some("nvim".to_string()),
                },
                panes: vec![SplitPane {
                    split: SplitDirection::Right,
                    cmd: Some("claude".to_string()),
                    size: None,
                }],
                layout_string: None,
            }),
            claude_sessions: vec![ClaudeSession {
                id: "session_abc".to_string(),
                label: "brainstorm".to_string(),
                started_at: dt("2026-03-01T10:00:00Z"),
                status: ClaudeSessionStatus::Active,
            }],
            notes: vec![Note {
                content: "picking up from step 4".to_string(),
                created_at: dt("2026-03-03T10:15:00Z"),
            }],
            last_state: Some(SavedState {
                captured_at: dt("2026-03-03T16:00:00Z"),
                layout_string: "5aed,176x79,0,0".to_string(),
                panes: vec![SavedPane {
                    index: 0,
                    path: "/home/user/src/my-project".to_string(),
                    command: "nvim".to_string(),
                }],
            }),
        };

        // Note: toml crate serializes tables before arrays of tables.
        // The exact output order depends on serde field order + toml rules.
        // This test will reveal the actual format.
        let expected = r##"[project]
name = "my-project"
path = "/home/user/src/my-project"
color = "#e06c75"
created_at = "2026-03-03T10:00:00Z"

[layout.main]
cmd = "nvim"

[[layout.panes]]
split = "right"
cmd = "claude"

[[claude_sessions]]
id = "session_abc"
label = "brainstorm"
started_at = "2026-03-01T10:00:00Z"
status = "active"

[[notes]]
content = "picking up from step 4"
created_at = "2026-03-03T10:15:00Z"

[last_state]
captured_at = "2026-03-03T16:00:00Z"
layout_string = "5aed,176x79,0,0"

[[last_state.panes]]
index = 0
path = "/home/user/src/my-project"
command = "nvim"
"##;

        assert_toml_roundtrip(&config, expected);
    }

    fn valid_metadata() -> ProjectMetadata {
        ProjectMetadata {
            name: "my-project".to_string(),
            path: "/some/path".to_string(),
            color: None,
            created_at: dt("2026-01-01T00:00:00Z"),
        }
    }

    #[test]
    fn validate_accepts_valid_metadata() {
        let meta = valid_metadata();
        assert!(meta.validate().is_ok());
    }

    #[test]
    fn validate_accepts_valid_color_with_hash() {
        let meta = ProjectMetadata {
            color: Some("#e06c75".to_string()),
            ..valid_metadata()
        };
        assert!(meta.validate().is_ok());
    }

    #[test]
    fn validate_accepts_valid_color_without_hash() {
        let meta = ProjectMetadata {
            color: Some("e06c75".to_string()),
            ..valid_metadata()
        };
        assert!(meta.validate().is_ok());
    }

    #[test]
    fn validate_rejects_name_with_dot() {
        let meta = ProjectMetadata {
            name: "my.project".to_string(),
            ..valid_metadata()
        };
        assert!(matches!(
            meta.validate(),
            Err(ProjectError::InvalidName { .. })
        ));
    }

    #[test]
    fn validate_rejects_name_with_colon() {
        let meta = ProjectMetadata {
            name: "my:project".to_string(),
            ..valid_metadata()
        };
        assert!(matches!(
            meta.validate(),
            Err(ProjectError::InvalidName { .. })
        ));
    }

    #[test]
    fn validate_rejects_name_with_space() {
        let meta = ProjectMetadata {
            name: "my project".to_string(),
            ..valid_metadata()
        };
        assert!(matches!(
            meta.validate(),
            Err(ProjectError::InvalidName { .. })
        ));
    }

    #[test]
    fn validate_rejects_empty_name() {
        let meta = ProjectMetadata {
            name: "".to_string(),
            ..valid_metadata()
        };
        assert!(matches!(
            meta.validate(),
            Err(ProjectError::InvalidName { .. })
        ));
    }

    #[test]
    fn validate_rejects_invalid_color() {
        let meta = ProjectMetadata {
            color: Some("not-a-color".to_string()),
            ..valid_metadata()
        };
        assert!(matches!(
            meta.validate(),
            Err(ProjectError::InvalidColor { .. })
        ));
    }

    #[test]
    fn validate_rejects_empty_path() {
        let meta = ProjectMetadata {
            path: "".to_string(),
            ..valid_metadata()
        };
        assert!(matches!(meta.validate(), Err(ProjectError::EmptyPath)));
    }
}
