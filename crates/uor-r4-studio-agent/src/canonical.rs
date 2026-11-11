//! `uor-r4.canonical-json/1` and event digests from spec §9.3.

use std::collections::HashSet;
use std::fmt;

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};

use crate::contract::AgentEvent;
use crate::error::{AgentError, AgentErrorCode};
use crate::ids::hex_lower;
use crate::MAX_SAFE_INTEGER;

const STRICT_PREFIX: &str = "uor-canonical-json: ";

pub fn canonical_json(value: &Value) -> Result<String, AgentError> {
    let mut out = String::new();
    write_canonical(value, &mut out)?;
    Ok(out)
}

pub fn parse_canonical_json(text: &str) -> Result<String, AgentError> {
    if contains_negative_zero(text) {
        return Err(AgentError::invalid_event(
            "negative zero is not a canonical number",
        ));
    }
    let mut deserializer = serde_json::Deserializer::from_str(text);
    let strict = StrictValue::deserialize(&mut deserializer).map_err(|error| {
        let message = error.to_string();
        if message.contains(STRICT_PREFIX) {
            AgentError::invalid_event(message)
        } else {
            AgentError::new(AgentErrorCode::MalformedEvent, message)
        }
    })?;
    deserializer.end().map_err(|error| {
        AgentError::new(
            AgentErrorCode::MalformedEvent,
            format!("trailing input after canonical value: {error}"),
        )
    })?;
    canonical_json(&strict.0)
}

pub fn digest_event(event: &AgentEvent) -> Result<String, AgentError> {
    let value = serde_json::to_value(event).map_err(|error| {
        AgentError::new(
            AgentErrorCode::Internal,
            format!("event serialization failed: {error}"),
        )
    })?;
    let canonical = canonical_json(&value)?;
    Ok(sha256_prefixed(canonical.as_bytes()))
}

pub(crate) fn sha256_prefixed(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("sha256:{}", hex_lower(digest.as_slice()))
}

pub(crate) fn is_digest_string(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(crate) fn stable_json(value: &Value) -> String {
    let mut out = String::new();
    write_stable(value, &mut out);
    out
}

fn write_canonical(value: &Value, out: &mut String) -> Result<(), AgentError> {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(number) => out.push_str(&canonical_number(number)?),
        Value::String(text) => escape_string(text, out),
        Value::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_canonical(item, out)?;
            }
            out.push(']');
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_unstable();
            out.push('{');
            for (index, key) in keys.iter().enumerate() {
                if !is_canonical_key(key) {
                    return Err(AgentError::invalid_event(format!(
                        "object key {key:?} is not in the canonical ASCII key set"
                    )));
                }
                if index > 0 {
                    out.push(',');
                }
                escape_string(key, out);
                out.push(':');
                let Some(item) = map.get(key.as_str()) else {
                    return Err(AgentError::new(
                        AgentErrorCode::Internal,
                        "object key disappeared during canonicalization",
                    ));
                };
                write_canonical(item, out)?;
            }
            out.push('}');
        }
    }
    Ok(())
}

fn canonical_number(number: &Number) -> Result<String, AgentError> {
    if let Some(value) = number.as_u64() {
        if value <= MAX_SAFE_INTEGER {
            return Ok(value.to_string());
        }
        return Err(AgentError::invalid_event(format!(
            "integer {value} exceeds the canonical safe integer range"
        )));
    }
    if let Some(value) = number.as_i64() {
        if value >= 0 && (value as u64) <= MAX_SAFE_INTEGER {
            return Ok(value.to_string());
        }
    }
    Err(AgentError::invalid_event(
        "only non-negative safe integers are canonical",
    ))
}

fn is_canonical_key(key: &str) -> bool {
    key.bytes().all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.' | b'-')
    })
}

fn escape_string(text: &str, out: &mut String) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    out.push('"');
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0}'..='\u{1f}' => {
                let code = character as u32;
                out.push_str("\\u00");
                out.push(HEX[((code >> 4) & 0x0f) as usize] as char);
                out.push(HEX[(code & 0x0f) as usize] as char);
            }
            _ => out.push(character),
        }
    }
    out.push('"');
}

fn write_stable(value: &Value, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(number) => out.push_str(&number.to_string()),
        Value::String(text) => escape_string(text, out),
        Value::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_stable(item, out);
            }
            out.push(']');
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_unstable();
            out.push('{');
            for (index, key) in keys.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                escape_string(key, out);
                out.push(':');
                if let Some(item) = map.get(key.as_str()) {
                    write_stable(item, out);
                }
            }
            out.push('}');
        }
    }
}

fn contains_negative_zero(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut in_string = false;
    let mut escaped = false;
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
        } else if byte == b'"' {
            in_string = true;
        } else if byte == b'-' && bytes.get(index + 1) == Some(&b'0') {
            let next = bytes.get(index + 2).copied().unwrap_or(b' ');
            if !(next.is_ascii_digit() || next == b'.' || next == b'e' || next == b'E') {
                return true;
            }
        }
        index += 1;
    }
    false
}

struct StrictValue(Value);

impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StrictValueVisitor)
    }
}

struct StrictValueVisitor;

impl<'de> Visitor<'de> for StrictValueVisitor {
    type Value = StrictValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a canonical JSON value")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value < 0 {
            return Err(E::custom(format!(
                "{STRICT_PREFIX}negative integers are not canonical"
            )));
        }
        self.visit_u64(value as u64)
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if value > MAX_SAFE_INTEGER {
            return Err(E::custom(format!(
                "{STRICT_PREFIX}integer {value} exceeds the canonical safe integer range"
            )));
        }
        Ok(StrictValue(Value::Number(Number::from(value))))
    }

    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Err(E::custom(format!(
            "{STRICT_PREFIX}floats are not canonical"
        )))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictValue(Value::String(value.to_string())))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::String(value)))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Null))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Null))
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut items = Vec::new();
        while let Some(item) = sequence.next_element::<StrictValue>()? {
            items.push(item.0);
        }
        Ok(StrictValue(Value::Array(items)))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        let mut seen = HashSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !seen.insert(key.clone()) {
                return Err(de::Error::custom(format!(
                    "{STRICT_PREFIX}duplicate object key {key:?}"
                )));
            }
            let value = map.next_value::<StrictValue>()?;
            values.insert(key, value.0);
        }
        Ok(StrictValue(Value::Object(values)))
    }
}
