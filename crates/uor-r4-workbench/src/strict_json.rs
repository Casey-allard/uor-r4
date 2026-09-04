//! Strict JSON intake for the workbench wire protocol.
//!
//! `serde_json::Value` keeps the last occurrence of a repeated object key.
//! That behavior is not admissible for configuration, HTTP, or IPC records
//! whose exact fields carry identity and lifecycle authority.  Every typed
//! parse therefore makes a bounded validation pass before deserializing the
//! requested type.

use serde::de::{DeserializeOwned, DeserializeSeed, Error as _, MapAccess, SeqAccess, Visitor};
use std::collections::BTreeSet;
use std::fmt;

/// Maximum nested JSON array/object containers accepted by #1105.
pub const JSON_MAX_NESTING: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StrictJsonError {
    message: String,
}

impl StrictJsonError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for StrictJsonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for StrictJsonError {}

#[derive(Clone, Copy)]
struct UniqueSeed {
    container_depth: usize,
    max_nesting: usize,
}

impl UniqueSeed {
    fn child(self) -> Self {
        Self {
            container_depth: self.container_depth + 1,
            max_nesting: self.max_nesting,
        }
    }

    fn enter_container<E: serde::de::Error>(self) -> Result<(), E> {
        if self.container_depth >= self.max_nesting {
            return Err(E::custom(format!(
                "JSON nesting exceeds {} containers",
                self.max_nesting
            )));
        }
        Ok(())
    }
}

impl<'de> DeserializeSeed<'de> for UniqueSeed {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(UniqueVisitor { seed: self })
    }
}

struct UniqueVisitor {
    seed: UniqueSeed,
}

impl<'de> Visitor<'de> for UniqueVisitor {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("JSON with unique object keys and bounded nesting")
    }

    fn visit_bool<E>(self, _: bool) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_i64<E>(self, _: i64) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_u64<E>(self, _: u64) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_f64<E>(self, _: f64) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_char<E>(self, _: char) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_str<E>(self, _: &str) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_borrowed_str<E>(self, _: &'de str) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_string<E>(self, _: String) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.seed.deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        self.seed.enter_container()?;
        while sequence.next_element_seed(self.seed.child())?.is_some() {}
        Ok(())
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        self.seed.enter_container()?;
        let mut keys = BTreeSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !keys.insert(key.clone()) {
                return Err(A::Error::custom(format!("duplicate JSON field {key:?}")));
            }
            map.next_value_seed(self.seed.child())?;
        }
        Ok(())
    }
}

/// Verify uniqueness, nesting, syntax, and the absence of trailing data.
pub fn validate_document(bytes: &[u8]) -> Result<(), StrictJsonError> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    UniqueSeed {
        container_depth: 0,
        max_nesting: JSON_MAX_NESTING,
    }
    .deserialize(&mut deserializer)
    .map_err(|error| StrictJsonError::new(format!("malformed JSON: {error}")))?;
    deserializer
        .end()
        .map_err(|error| StrictJsonError::new(format!("malformed JSON: {error}")))
}

/// Strictly validate a document and then deserialize its exact typed schema.
pub fn from_slice<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, StrictJsonError> {
    validate_document(bytes)?;
    serde_json::from_slice(bytes)
        .map_err(|error| StrictJsonError::new(format!("invalid JSON schema: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    #[serde(deny_unknown_fields)]
    struct Exact {
        value: u64,
    }

    #[test]
    fn rejects_duplicate_keys_at_every_depth() {
        let top = from_slice::<serde_json::Value>(br#"{"a":1,"a":2}"#)
            .expect_err("top-level duplicate must fail");
        assert!(top.to_string().contains("duplicate JSON field \"a\""));

        let nested = from_slice::<serde_json::Value>(br#"{"a":{"b":1,"b":2}}"#)
            .expect_err("nested duplicate must fail");
        assert!(nested.to_string().contains("duplicate JSON field \"b\""));
    }

    #[test]
    fn enforces_thirty_two_container_nesting_limit() {
        let admitted = format!("{}0{}", "[".repeat(32), "]".repeat(32));
        validate_document(admitted.as_bytes()).expect("32 containers are admitted");

        let rejected = format!("{}0{}", "[".repeat(33), "]".repeat(33));
        let error = validate_document(rejected.as_bytes()).expect_err("33 containers must fail");
        assert!(error.to_string().contains("exceeds 32 containers"));
    }

    #[test]
    fn typed_pass_rejects_unknown_missing_and_wrong_type_fields() {
        assert_eq!(
            from_slice::<Exact>(br#"{"value":7}"#).expect("exact object"),
            Exact { value: 7 }
        );
        assert!(from_slice::<Exact>(br#"{"value":7,"extra":false}"#).is_err());
        assert!(from_slice::<Exact>(br#"{}"#).is_err());
        assert!(from_slice::<Exact>(br#"{"value":"7"}"#).is_err());
        assert!(from_slice::<Exact>(br#"{"value":7} null"#).is_err());
    }
}
