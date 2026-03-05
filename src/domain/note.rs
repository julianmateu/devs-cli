use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    content: String,
    created_at: DateTime<Utc>, //
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_constructor() {
        let note = Note {
            content: String::from("test content"),
            created_at: DateTime::parse_from_rfc3339("2020-10-01T01:47:12.746202562+00:00")
                .unwrap()
                .to_utc(),
        };

        let expected_note_toml = r#"content = "test content"
created_at = "2020-10-01T01:47:12.746202562Z"
"#;

        let serialized_note = toml::to_string(&note).unwrap();
        assert_eq!(serialized_note, expected_note_toml);

        let deserialized_note: Note = toml::from_str(&serialized_note).unwrap();
        assert_eq!(deserialized_note, note);
    }
}
