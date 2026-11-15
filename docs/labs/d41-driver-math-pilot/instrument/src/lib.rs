//! The D41/D86 donor pilot's client: prompt set, corpus records, receipt hashes and
//! the readback check. It holds no model: the donor runs in an external engine.

pub mod prompts;

use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

/// The most answer texts a prompt may keep (D86: "up to four").
pub const MAX_ANSWERS: usize = 4;

pub fn sha256_hex(bytes: &[u8]) -> String {
    let d = Sha256::digest(bytes);
    d.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_bytes(h: &str) -> Result<Vec<u8>, String> {
    if !h.len().is_multiple_of(2) {
        return Err(format!("odd hex length: {h}"));
    }
    (0..h.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&h[i..i + 2], 16).map_err(|e| format!("bad hex {h}: {e}")))
        .collect()
}

/// The Merkle root over shard hashes, in shard order. A parent is
/// `sha256(left_bytes || right_bytes)`; an unpaired last node moves up unchanged.
/// One leaf is its own root.
pub fn merkle_root(leaves_hex: &[String]) -> Result<String, String> {
    if leaves_hex.is_empty() {
        return Err("no leaves".into());
    }
    let mut level: Vec<Vec<u8>> = leaves_hex
        .iter()
        .map(|h| hex_bytes(h))
        .collect::<Result<_, _>>()?;
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        for pair in level.chunks(2) {
            if pair.len() == 2 {
                let mut cat = pair[0].clone();
                cat.extend_from_slice(&pair[1]);
                next.push(Sha256::digest(&cat).to_vec());
            } else {
                next.push(pair[0].clone());
            }
        }
        level = next;
    }
    Ok(level[0].iter().map(|b| format!("{b:02x}")).collect())
}

/// The answers a prompt keeps: distinct non-empty trimmed texts, at most four, in byte
/// order. Byte order means a text's position carries no donor preference (no rank).
pub fn canonical_answers(texts: &[String]) -> Vec<String> {
    let set: BTreeSet<String> = texts
        .iter()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect();
    set.into_iter().take(MAX_ANSWERS).collect()
}

/// An answer text the engine returned with its channel markup still in it: the harmony
/// parse failed, so the text may hold reasoning, and it is not kept.
pub fn has_markup(text: &str) -> bool {
    text.contains("<|") || text.contains("|>")
}

/// The structural consumption check on one corpus record: exactly the keys `id`, `kind`,
/// `messages`, `answers`; messages hold only `role` and `content` strings; answers are one
/// to four distinct strings in byte order. Any other field (a logit, probability, score or
/// rank among them) fails.
pub fn check_record(v: &serde_json::Value) -> Result<(), String> {
    let m = v.as_object().ok_or("record is not an object")?;
    let keys: BTreeSet<&str> = m.keys().map(|k| k.as_str()).collect();
    let want: BTreeSet<&str> = ["answers", "id", "kind", "messages"].into_iter().collect();
    if keys != want {
        return Err(format!("record keys {keys:?}, want {want:?}"));
    }
    for k in ["id", "kind"] {
        if !m[k].is_string() {
            return Err(format!("{k} is not a string"));
        }
    }
    let msgs = m["messages"].as_array().ok_or("messages is not an array")?;
    if msgs.is_empty() {
        return Err("no messages".into());
    }
    for msg in msgs {
        let o = msg.as_object().ok_or("message is not an object")?;
        let mk: BTreeSet<&str> = o.keys().map(|k| k.as_str()).collect();
        if mk != ["content", "role"].into_iter().collect::<BTreeSet<_>>() {
            return Err(format!("message keys {mk:?}"));
        }
        if !o["role"].is_string() || !o["content"].is_string() {
            return Err("message role or content is not a string".into());
        }
    }
    let ans = m["answers"].as_array().ok_or("answers is not an array")?;
    if ans.is_empty() || ans.len() > MAX_ANSWERS {
        return Err(format!("{} answers", ans.len()));
    }
    let texts: Vec<String> = ans
        .iter()
        .map(|a| {
            a.as_str()
                .map(str::to_string)
                .ok_or("an answer is not a string")
        })
        .collect::<Result<_, _>>()?;
    if canonical_answers(&texts) != texts {
        return Err("answers are not distinct, trimmed and in byte order".into());
    }
    if texts.iter().any(|t| has_markup(t)) {
        return Err("an answer holds channel markup".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn merkle_root_of_known_leaves() {
        let a = sha256_hex(b"a");
        let b = sha256_hex(b"b");
        let c = sha256_hex(b"c");
        assert_eq!(merkle_root(std::slice::from_ref(&a)).unwrap(), a);
        let ab = {
            let mut cat = hex_bytes(&a).unwrap();
            cat.extend(hex_bytes(&b).unwrap());
            sha256_hex(&cat)
        };
        assert_eq!(merkle_root(&[a.clone(), b.clone()]).unwrap(), ab);
        let abc = {
            let mut cat = hex_bytes(&ab).unwrap();
            cat.extend(hex_bytes(&c).unwrap());
            sha256_hex(&cat)
        };
        assert_eq!(
            merkle_root(&[a.clone(), b.clone(), c.clone()]).unwrap(),
            abc
        );
        assert_ne!(merkle_root(&[b, a]).unwrap(), ab);
        assert!(merkle_root(&[]).is_err());
    }

    #[test]
    fn answers_are_distinct_capped_and_in_byte_order() {
        let t = |xs: &[&str]| xs.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(
            canonical_answers(&t(&["b", " a ", "b", ""])),
            t(&["a", "b"])
        );
        assert_eq!(canonical_answers(&t(&["e", "d", "c", "b", "a"])).len(), 4);
    }

    #[test]
    fn a_record_with_any_score_field_fails() {
        let good = json!({"id": "p0", "kind": "open", "messages": [{"role": "user", "content": "Hi"}],
                          "answers": ["Hello!", "Hi there."]});
        assert!(check_record(&good).is_ok());
        for extra in ["logprobs", "probs", "score", "rank", "top_logprobs"] {
            let mut bad = good.clone();
            bad[extra] = json!([5]);
            assert!(check_record(&bad).is_err(), "{extra} passed");
        }
        let mut bad = good.clone();
        bad["messages"][0]["logprob"] = json!(-1);
        assert!(check_record(&bad).is_err());
        let mut bad = good.clone();
        bad["answers"] = json!(["b", "a"]);
        assert!(check_record(&bad).is_err());
        let mut bad = good.clone();
        bad["answers"] = json!(["<|channel|>analysis"]);
        assert!(check_record(&bad).is_err());
        let mut bad = good;
        bad["answers"] = json!([]);
        assert!(check_record(&bad).is_err());
    }
}
