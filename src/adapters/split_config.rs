use serde::{Deserialize, Serialize};

use crate::domain::claude_session::ClaudeSession;
use crate::domain::layout::Layout;
use crate::domain::note::Note;
use crate::domain::project::{ProjectConfig, ProjectMetadata};
use crate::domain::saved_state::SavedState;

/// Portable config written to `projects/<name>.toml`.
/// Contains metadata, layout, and notes — everything safe to sync across machines.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortableConfig {
    pub project: ProjectMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<Layout>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<Note>,
}

/// Machine-local config written to `local/<name>.toml`.
/// Contains claude sessions and saved tmux state — machine-specific data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MachineLocalConfig {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub claude_sessions: Vec<ClaudeSession>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_state: Option<SavedState>,
}

impl MachineLocalConfig {
    pub fn is_empty(&self) -> bool {
        self.claude_sessions.is_empty() && self.last_state.is_none()
    }
}

/// Split a full ProjectConfig into portable and machine-local halves.
pub fn split(config: &ProjectConfig) -> (PortableConfig, MachineLocalConfig) {
    let portable = PortableConfig {
        project: config.project.clone(),
        layout: config.layout.clone(),
        notes: config.notes.clone(),
    };

    let local = MachineLocalConfig {
        claude_sessions: config.claude_sessions.clone(),
        last_state: config.last_state.clone(),
    };

    (portable, local)
}

/// Merge a portable config with machine-local data into a full ProjectConfig.
#[cfg(test)]
pub fn merge(portable: PortableConfig, local: MachineLocalConfig) -> ProjectConfig {
    ProjectConfig {
        project: portable.project,
        layout: portable.layout,
        claude_sessions: local.claude_sessions,
        notes: portable.notes,
        last_state: local.last_state,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::claude_session::ClaudeSessionStatus;
    use crate::domain::layout::{Layout, MainPane, SplitDirection, SplitPane};
    use crate::domain::saved_state::SavedPane;
    use crate::domain::test_helpers::{assert_toml_roundtrip, dt};

    fn full_config() -> ProjectConfig {
        ProjectConfig {
            project: ProjectMetadata {
                name: "test-proj".to_string(),
                path: "~/src/test".to_string(),
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
                    size: Some("40%".to_string()),
                }],
                layout_string: None,
            }),
            claude_sessions: vec![ClaudeSession {
                id: "sess_abc".to_string(),
                label: "brainstorm".to_string(),
                started_at: dt("2026-03-01T10:00:00Z"),
                status: ClaudeSessionStatus::Active,
            }],
            notes: vec![Note {
                content: "a note".to_string(),
                created_at: dt("2026-03-03T10:15:00Z"),
            }],
            last_state: Some(SavedState {
                captured_at: dt("2026-03-03T16:00:00Z"),
                layout_string: "5aed,176x79,0,0".to_string(),
                panes: vec![SavedPane {
                    index: 0,
                    path: "/home/user/src/test-proj".to_string(),
                    command: "nvim".to_string(),
                }],
            }),
        }
    }

    #[test]
    fn portable_config_roundtrip() {
        let portable = PortableConfig {
            project: ProjectMetadata {
                name: "test-proj".to_string(),
                path: "~/src/test".to_string(),
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
                    size: Some("40%".to_string()),
                }],
                layout_string: None,
            }),
            notes: vec![Note {
                content: "a note".to_string(),
                created_at: dt("2026-03-03T10:15:00Z"),
            }],
        };

        let expected = r##"[project]
name = "test-proj"
path = "~/src/test"
color = "#e06c75"
created_at = "2026-03-03T10:00:00Z"

[layout.main]
cmd = "nvim"

[[layout.panes]]
split = "right"
cmd = "claude"
size = "40%"

[[notes]]
content = "a note"
created_at = "2026-03-03T10:15:00Z"
"##;

        assert_toml_roundtrip(&portable, expected);
    }

    #[test]
    fn machine_local_config_roundtrip() {
        let local = MachineLocalConfig {
            claude_sessions: vec![ClaudeSession {
                id: "sess_abc".to_string(),
                label: "brainstorm".to_string(),
                started_at: dt("2026-03-01T10:00:00Z"),
                status: ClaudeSessionStatus::Active,
            }],
            last_state: Some(SavedState {
                captured_at: dt("2026-03-03T16:00:00Z"),
                layout_string: "5aed,176x79,0,0".to_string(),
                panes: vec![SavedPane {
                    index: 0,
                    path: "/home/user/src/test-proj".to_string(),
                    command: "nvim".to_string(),
                }],
            }),
        };

        let expected = r#"[[claude_sessions]]
id = "sess_abc"
label = "brainstorm"
started_at = "2026-03-01T10:00:00Z"
status = "active"

[last_state]
captured_at = "2026-03-03T16:00:00Z"
layout_string = "5aed,176x79,0,0"

[[last_state.panes]]
index = 0
path = "/home/user/src/test-proj"
command = "nvim"
"#;

        assert_toml_roundtrip(&local, expected);
    }

    #[test]
    fn split_separates_portable_from_local() {
        let config = full_config();
        let (portable, local) = split(&config);

        // Portable gets metadata, layout, notes
        assert_eq!(portable.project, config.project);
        assert_eq!(portable.layout, config.layout);
        assert_eq!(portable.notes, config.notes);

        // Local gets sessions and state
        assert_eq!(local.claude_sessions, config.claude_sessions);
        assert_eq!(local.last_state, config.last_state);
    }

    #[test]
    fn split_then_merge_roundtrips() {
        let config = full_config();
        let (portable, local) = split(&config);
        let merged = merge(portable, local);
        assert_eq!(merged, config);
    }

    #[test]
    fn machine_local_is_empty_when_no_data() {
        let local = MachineLocalConfig {
            claude_sessions: vec![],
            last_state: None,
        };
        assert!(local.is_empty());
    }

    #[test]
    fn machine_local_not_empty_with_sessions() {
        let local = MachineLocalConfig {
            claude_sessions: vec![ClaudeSession {
                id: "s".to_string(),
                label: "l".to_string(),
                started_at: dt("2026-01-01T00:00:00Z"),
                status: ClaudeSessionStatus::Active,
            }],
            last_state: None,
        };
        assert!(!local.is_empty());
    }

    #[test]
    fn machine_local_not_empty_with_state() {
        let local = MachineLocalConfig {
            claude_sessions: vec![],
            last_state: Some(SavedState {
                captured_at: dt("2026-01-01T00:00:00Z"),
                layout_string: "layout".to_string(),
                panes: vec![],
            }),
        };
        assert!(!local.is_empty());
    }

    #[test]
    fn empty_local_config_roundtrip() {
        let local = MachineLocalConfig {
            claude_sessions: vec![],
            last_state: None,
        };
        let expected = "";
        assert_toml_roundtrip(&local, expected);
    }

    #[test]
    fn minimal_portable_config_roundtrip() {
        let portable = PortableConfig {
            project: ProjectMetadata {
                name: "minimal".to_string(),
                path: "/some/path".to_string(),
                color: None,
                created_at: dt("2026-01-01T00:00:00Z"),
            },
            layout: None,
            notes: vec![],
        };

        let expected = r#"[project]
name = "minimal"
path = "/some/path"
created_at = "2026-01-01T00:00:00Z"
"#;

        assert_toml_roundtrip(&portable, expected);
    }
}
