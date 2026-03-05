use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    content: String,
    created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::test_helpers::assert_toml_roundtrip;

    #[test]
    fn note_toml_roundtrip() {
        let note = Note {
            content: String::from("test content"),
            created_at: DateTime::parse_from_rfc3339("2020-10-01T01:47:12.746202562+00:00")
                .unwrap()
                .to_utc(),
        };

        let expected_note_toml = r#"content = "test content"
created_at = "2020-10-01T01:47:12.746202562Z"
"#;
        assert_toml_roundtrip(&note, expected_note_toml);
    }
}
