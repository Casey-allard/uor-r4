//! Derivation of a smaller byte-level BPE `tokenizer.json` from a larger one.
//!
//! The real-text experiments need a 4,096-token vocabulary "matching the shipped model's resolution".
//! Rather than re-train a tokenizer, they truncate the source tokenizer's dense id prefix: the base
//! byte tokens plus the first `vocab - 256` merges. This module holds that derivation so the count
//! tool and the coverage instrument cannot drift apart.
//!
//! The derived JSON is fed back through
//! [`HfBpeTokenizer::from_tokenizer_json_bytes`](crate::transformerless::hf_bpe::HfBpeTokenizer::from_tokenizer_json_bytes)
//! rather than to a hand-written encoder, so the derived vocabulary and the runtime tokenizer are
//! checked by the same code.

use crate::transformerless::hf_bpe::HfBpeTokenizer;

/// Derive a dense `vocab`-sized byte-level BPE `tokenizer.json` from a larger one.
///
/// The derived vocabulary is the dense id prefix `0..vocab`. Merges are kept in their original rank
/// order but only when their product token survives that prefix, which is exactly the truncation a
/// fresh BPE at this size would produce.
///
/// Errors when the source prefix is not dense, or when `vocab` exceeds the source: a silently aliased
/// or gapped vocabulary would corrupt every downstream count.
pub fn derive_tokenizer_json(original: &[u8], vocab: usize) -> Result<Vec<u8>, String> {
    let mut root: serde_json::Value =
        serde_json::from_slice(original).map_err(|e| format!("tokenizer.json: {e}"))?;

    {
        let model = root
            .get_mut("model")
            .ok_or("tokenizer.json has no `model`")?;
        let vocab_map = model
            .get("vocab")
            .and_then(serde_json::Value::as_object)
            .ok_or("model.vocab is not an object")?;

        let mut kept = serde_json::Map::with_capacity(vocab);
        for (piece, id) in vocab_map {
            let id = id.as_u64().ok_or("model.vocab id is not an integer")?;
            if (id as usize) < vocab {
                kept.insert(piece.clone(), serde_json::Value::from(id));
            }
        }
        if kept.len() != vocab {
            return Err(format!(
                "derived vocabulary is {} tokens, expected a dense prefix of {vocab}",
                kept.len()
            ));
        }
        let present: std::collections::HashSet<u64> = kept
            .values()
            .filter_map(serde_json::Value::as_u64)
            .collect();
        for id in 0..vocab as u64 {
            if !present.contains(&id) {
                return Err(format!("derived vocabulary is not dense: id {id} missing"));
            }
        }

        // Keep merges in rank order, dropping any whose product did not survive the truncation.
        let merges = model
            .get("merges")
            .and_then(serde_json::Value::as_array)
            .ok_or("model.merges is not an array")?;
        let mut kept_merges: Vec<serde_json::Value> = Vec::with_capacity(merges.len());
        for entry in merges {
            let (left, right) = match entry {
                serde_json::Value::String(pair) => {
                    let mut it = pair.split(' ');
                    match (it.next(), it.next(), it.next()) {
                        (Some(l), Some(r), None) => (l.to_string(), r.to_string()),
                        _ => return Err(format!("malformed merge string: {pair:?}")),
                    }
                }
                serde_json::Value::Array(pair) => match (
                    pair.first().and_then(serde_json::Value::as_str),
                    pair.get(1).and_then(serde_json::Value::as_str),
                    pair.len(),
                ) {
                    (Some(l), Some(r), 2) => (l.to_string(), r.to_string()),
                    _ => return Err(format!("malformed merge array: {pair:?}")),
                },
                _ => return Err("malformed merge entry".into()),
            };
            if kept.contains_key(&format!("{left}{right}")) {
                kept_merges.push(entry.clone());
            }
        }
        model["vocab"] = serde_json::Value::Object(kept.clone());
        model["merges"] = serde_json::Value::Array(kept_merges);
    }

    // Drop added tokens outside the derived prefix so the parser sees a dense vocabulary.
    if let Some(added) = root
        .get_mut("added_tokens")
        .and_then(serde_json::Value::as_array_mut)
    {
        added.retain(|e| {
            e.get("id")
                .and_then(serde_json::Value::as_u64)
                .is_some_and(|id| (id as usize) < vocab)
        });
    }

    serde_json::to_vec(&root).map_err(|e| format!("re-serialize tokenizer: {e}"))
}

/// Load the derived tokenizer, or report why the derivation was rejected.
pub fn derive_tokenizer(original: &[u8], vocab: usize) -> Result<HfBpeTokenizer, String> {
    let bytes = derive_tokenizer_json(original, vocab)?;
    HfBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| "derived tokenizer.json is not a valid byte-level BPE tokenizer".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A synthetic 1000-token byte-level BPE `tokenizer.json` whose merges all produce tokens present
    /// in the vocabulary, so truncation is exactly observable.
    fn synthetic_tokenizer(total: usize) -> Vec<u8> {
        let base = 256usize;
        let mut vocab = serde_json::Map::new();
        for k in 0..base {
            vocab.insert(format!("a{k}"), serde_json::Value::from(k as u64));
        }
        for k in base..total {
            vocab.insert(
                format!("m{j}m{j}", j = k - base),
                serde_json::Value::from(k as u64),
            );
        }
        let merges: Vec<serde_json::Value> = (0..total - base)
            .map(|j| {
                serde_json::Value::Array(vec![
                    serde_json::Value::from(format!("m{j}")),
                    serde_json::Value::from(format!("m{j}")),
                ])
            })
            .collect();
        let root = serde_json::json!({
            "model": { "type": "BPE", "vocab": vocab, "merges": merges },
            "added_tokens": [],
        });
        serde_json::to_vec(&root).expect("serialize")
    }

    fn inspect(json: &[u8]) -> (usize, usize) {
        let v: serde_json::Value = serde_json::from_slice(json).expect("parse");
        let model = v.get("model").expect("model");
        (
            model
                .get("vocab")
                .and_then(serde_json::Value::as_object)
                .expect("vocab")
                .len(),
            model
                .get("merges")
                .and_then(serde_json::Value::as_array)
                .expect("merges")
                .len(),
        )
    }

    #[test]
    fn truncation_keeps_exactly_the_dense_prefix_and_its_merges() {
        let original = synthetic_tokenizer(1000);
        let derived = derive_tokenizer_json(&original, 500).expect("derive");
        let (vocab, merges) = inspect(&derived);
        assert_eq!(vocab, 500, "vocabulary must be the requested dense prefix");
        // Products surviving id < 500 are ids 256..=499, i.e. 244 of the 744 merges.
        assert_eq!(
            merges, 244,
            "only merges whose product survives may be kept"
        );
    }

    #[test]
    fn a_larger_request_than_the_source_is_rejected() {
        let original = synthetic_tokenizer(1000);
        assert!(derive_tokenizer_json(&original, 5000).is_err());
    }

    #[test]
    fn a_non_dense_prefix_is_rejected() {
        let original = synthetic_tokenizer(1000);
        let mut v: serde_json::Value = serde_json::from_slice(&original).expect("parse");
        let vocab = v["model"]["vocab"].as_object_mut().expect("vocab");
        vocab.retain(|_, id| id.as_u64() != Some(400));
        let holed = serde_json::to_vec(&v).expect("serialize");
        assert!(
            derive_tokenizer_json(&holed, 1000).is_err(),
            "a vocabulary with a hole must not be accepted as a dense prefix"
        );
    }
}
