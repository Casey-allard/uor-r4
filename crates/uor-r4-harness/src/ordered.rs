//! A JSON value that remembers object key order.
//!
//! `serde_json::Value` is a `BTreeMap` behind the scenes, so it **sorts** keys on parse. That is the
//! right default for most of this crate: several Python writers pass `sort_keys=True`, and it is the
//! sorting that makes the bytes match. It is wrong for the panel files, where `panel show --json`
//! prints each bar as the file's OWN object and the file's order is
//! `id, label, current, target, of, unit, direction, note`.
//!
//! **Scope is the point of this type.** Turning on `serde_json`'s `preserve_order` feature would change
//! every `Value` in the crate, including the paths that rely on the sorting, and would silently
//! reorder them. This is a separate representation, used only where document order is part of the
//! output.

use std::fmt;

use serde::de::{Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};

/// A JSON value with its object keys in document order.
#[derive(Debug, Clone, PartialEq)]
pub enum Ordered {
    Null,
    Bool(bool),
    Num(serde_json::Number),
    Str(String),
    Arr(Vec<Ordered>),
    Obj(Vec<(String, Ordered)>),
}

impl Ordered {
    /// Parse a document, keeping every object's key order.
    pub fn parse(text: &str) -> Result<Ordered, String> {
        serde_json::from_str::<Ordered>(text).map_err(|e| e.to_string())
    }

    pub fn get(&self, key: &str) -> Option<&Ordered> {
        match self {
            Ordered::Obj(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Ordered::Str(s) => Some(s),
            _ => None,
        }
    }

    /// A string field, or the empty string - the shape most of the registry reads take.
    pub fn str_or_empty(&self, key: &str) -> String {
        self.get(key)
            .and_then(Ordered::as_str)
            .unwrap_or("")
            .to_string()
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Ordered::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Ordered]> {
        match self {
            Ordered::Arr(items) => Some(items),
            _ => None,
        }
    }

    /// The pairs of an object, in document order.
    pub fn as_object(&self) -> Option<&[(String, Ordered)]> {
        match self {
            Ordered::Obj(pairs) => Some(pairs),
            _ => None,
        }
    }

    /// This value as `serde_json::Value`, losing the order - for callers that only need the data.
    pub fn to_value(&self) -> serde_json::Value {
        match self {
            Ordered::Null => serde_json::Value::Null,
            Ordered::Bool(b) => serde_json::Value::Bool(*b),
            Ordered::Num(n) => serde_json::Value::Number(n.clone()),
            Ordered::Str(s) => serde_json::Value::String(s.clone()),
            Ordered::Arr(items) => {
                serde_json::Value::Array(items.iter().map(Ordered::to_value).collect())
            }
            Ordered::Obj(pairs) => {
                let mut map = serde_json::Map::new();
                for (k, v) in pairs {
                    map.insert(k.clone(), v.to_value());
                }
                serde_json::Value::Object(map)
            }
        }
    }

    /// Python's compact `json.dumps(value)`: `", "` between items, `": "` after a key.
    pub fn to_json(&self) -> String {
        let mut out = String::new();
        self.write(&mut out, false, 0);
        out
    }

    /// Python's `json.dumps(value, indent=2)`.
    pub fn to_pretty(&self) -> String {
        let mut out = String::new();
        self.write(&mut out, true, 0);
        out
    }

    fn write(&self, out: &mut String, pretty: bool, depth: usize) {
        match self {
            Ordered::Null => out.push_str("null"),
            Ordered::Bool(true) => out.push_str("true"),
            Ordered::Bool(false) => out.push_str("false"),
            Ordered::Num(n) => out.push_str(&n.to_string()),
            // `ensure_ascii=False`: JSON's own escaping only, non-ASCII left alone.
            Ordered::Str(s) => out.push_str(&serde_json::Value::String(s.clone()).to_string()),
            Ordered::Arr(items) => {
                if items.is_empty() {
                    out.push_str("[]");
                    return;
                }
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        separator(out, pretty);
                    }
                    newline(out, pretty, depth + 1);
                    item.write(out, pretty, depth + 1);
                }
                newline(out, pretty, depth);
                out.push(']');
            }
            Ordered::Obj(pairs) => {
                if pairs.is_empty() {
                    out.push_str("{}");
                    return;
                }
                out.push('{');
                for (i, (k, v)) in pairs.iter().enumerate() {
                    if i > 0 {
                        separator(out, pretty);
                    }
                    newline(out, pretty, depth + 1);
                    out.push_str(&serde_json::Value::String(k.clone()).to_string());
                    out.push_str(": ");
                    v.write(out, pretty, depth + 1);
                }
                newline(out, pretty, depth);
                out.push('}');
            }
        }
    }
}

/// Python's separators: `", "` compact, `","` between an indented item and its newline.
fn separator(out: &mut String, pretty: bool) {
    out.push(',');
    if !pretty {
        out.push(' ');
    }
}

fn newline(out: &mut String, pretty: bool, depth: usize) {
    if pretty {
        out.push('\n');
        out.push_str(&"  ".repeat(depth));
    }
}

impl fmt::Display for Ordered {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_json())
    }
}

impl<'de> Deserialize<'de> for Ordered {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Ordered;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("any JSON value")
            }
            fn visit_bool<E>(self, v: bool) -> Result<Ordered, E> {
                Ok(Ordered::Bool(v))
            }
            fn visit_i64<E>(self, v: i64) -> Result<Ordered, E> {
                Ok(Ordered::Num(v.into()))
            }
            fn visit_u64<E>(self, v: u64) -> Result<Ordered, E> {
                Ok(Ordered::Num(v.into()))
            }
            fn visit_f64<E>(self, v: f64) -> Result<Ordered, E> {
                Ok(serde_json::Number::from_f64(v)
                    .map(Ordered::Num)
                    .unwrap_or(Ordered::Null))
            }
            fn visit_str<E>(self, v: &str) -> Result<Ordered, E> {
                Ok(Ordered::Str(v.to_string()))
            }
            fn visit_string<E>(self, v: String) -> Result<Ordered, E> {
                Ok(Ordered::Str(v))
            }
            fn visit_unit<E>(self) -> Result<Ordered, E> {
                Ok(Ordered::Null)
            }
            fn visit_none<E>(self) -> Result<Ordered, E> {
                Ok(Ordered::Null)
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Ordered, A::Error> {
                let mut items = Vec::new();
                while let Some(item) = seq.next_element::<Ordered>()? {
                    items.push(item);
                }
                Ok(Ordered::Arr(items))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Ordered, A::Error> {
                let mut pairs = Vec::new();
                while let Some((k, v)) = map.next_entry::<String, Ordered>()? {
                    // **Push, never insert into a sorted map**: the order IS the data here.
                    pairs.push((k, v));
                }
                Ok(Ordered::Obj(pairs))
            }
        }
        d.deserialize_any(V)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The order IS the data.** A `serde_json::Value` would answer `[("b", ..), ("a", ..)]` here and
    /// `panel show --json` would then print a bar the file never wrote.
    #[test]
    fn object_keys_keep_their_document_order() {
        let v = Ordered::parse(r#"{"id": "x", "label": "y", "current": 1, "note": "n"}"#)
            .expect("parse");
        let keys: Vec<&str> = v
            .as_object()
            .expect("object")
            .iter()
            .map(|(k, _)| k.as_str())
            .collect();
        assert_eq!(keys, vec!["id", "label", "current", "note"]);
        assert_eq!(v.str_or_empty("label"), "y");
        assert_eq!(v.str_or_empty("missing"), "");
    }

    #[test]
    fn nested_objects_and_arrays_keep_their_order_too() {
        let v =
            Ordered::parse(r#"{"panels": {"p": {"bars": [{"b": 1, "a": 2}]}}}"#).expect("parse");
        let bars = v
            .get("panels")
            .and_then(|p| p.get("p"))
            .and_then(|p| p.get("bars"))
            .and_then(Ordered::as_array)
            .expect("bars");
        let keys: Vec<&str> = bars[0]
            .as_object()
            .expect("bar")
            .iter()
            .map(|(k, _)| k.as_str())
            .collect();
        assert_eq!(keys, vec!["b", "a"]);
    }

    /// Python's compact form has a space after `:` and after `,`; its pretty form has neither.
    #[test]
    fn the_two_renderings_are_pythons() {
        let v =
            Ordered::parse(r#"{"a": 1, "b": [1, 2], "c": {"d": null}, "e": []}"#).expect("parse");
        assert_eq!(
            v.to_json(),
            r#"{"a": 1, "b": [1, 2], "c": {"d": null}, "e": []}"#
        );
        assert_eq!(
            v.to_pretty(),
            "{\n  \"a\": 1,\n  \"b\": [\n    1,\n    2\n  ],\n  \"c\": {\n    \"d\": null\n  },\n  \"e\": []\n}"
        );
    }

    /// `ensure_ascii=False`: the em dash stays an em dash, where `serde_json`'s default would escape it.
    #[test]
    fn non_ascii_is_left_alone() {
        let v = Ordered::parse(r#"{"note": "a — b"}"#).expect("parse");
        assert!(v.to_json().contains('—'), "{}", v.to_json());
    }

    #[test]
    fn numbers_survive_the_round_trip() {
        let v = Ordered::parse(r#"{"i": 437, "f": 2.5, "z": 0}"#).expect("parse");
        assert_eq!(v.to_json(), r#"{"i": 437, "f": 2.5, "z": 0}"#);
    }
}
