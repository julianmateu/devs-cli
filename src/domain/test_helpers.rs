use chrono::{DateTime, Utc};
use serde::{Serialize, de::DeserializeOwned};

pub fn assert_toml_roundtrip<T>(value: &T, expected_toml: &str)
where
    T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let serialized = toml::to_string(&value).unwrap();
    assert_eq!(serialized, expected_toml);

    let deserialized: T = toml::from_str(&serialized).unwrap();
    assert_eq!(&deserialized, value);
}

pub fn dt(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s).unwrap().to_utc()
}
