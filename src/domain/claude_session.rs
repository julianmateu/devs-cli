use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", content = "finished_at")]
#[serde(rename_all = "lowercase")]
enum ClaudeSessionStatus {
    Active,
    Done(DateTime<Utc>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClaudeSession {
    id: String,
    label: String,
    started_at: DateTime<Utc>,
    #[serde(flatten)]
    status: ClaudeSessionStatus,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::test_helpers::assert_toml_roundtrip;

    #[test]
    fn claude_session_active() {
        let active_session = ClaudeSession {
            id: String::from("session_1234"),
            label: String::from("my-test-session"),
            started_at: DateTime::parse_from_rfc3339("2000-01-01T00:00:00Z")
                .unwrap()
                .to_utc(),
            status: ClaudeSessionStatus::Active,
        };

        let expected_session_toml = r#"id = "session_1234"
label = "my-test-session"
started_at = "2000-01-01T00:00:00Z"
status = "active"
"#;

        assert_toml_roundtrip(&active_session, expected_session_toml);
    }

    #[test]
    fn claude_session_done() {
        let done_session = ClaudeSession {
            id: String::from("session_1234"),
            label: String::from("my-test-session"),
            started_at: DateTime::parse_from_rfc3339("2000-01-01T00:00:00Z")
                .unwrap()
                .to_utc(),
            status: ClaudeSessionStatus::Done(
                DateTime::parse_from_rfc3339("2020-01-05T12:00:00Z")
                    .unwrap()
                    .to_utc(),
            ),
        };
        let expected_session_toml = r#"id = "session_1234"
label = "my-test-session"
started_at = "2000-01-01T00:00:00Z"
status = "done"
finished_at = "2020-01-05T12:00:00Z"
"#;

        assert_toml_roundtrip(&done_session, expected_session_toml);
    }
}
