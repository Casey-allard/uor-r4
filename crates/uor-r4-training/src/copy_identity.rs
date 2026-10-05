//! Copy-identity table (Step 4 of the 2026-10-05 barrier assessment, #820).
//!
//! Step 0b (#1709) found that a rehearsed MQAR key at the start of a reply is
//! written in a casing that never occurs in the context: the assertion holds
//! `' l','u','k'` and the reply `' L','u','k'`. A pointer whose sources are
//! admitted by exact token identity (a routed pointer, [`PrimeRoute`]) then
//! cannot match the key. This module is the D11-legal answer on the match
//! side: a fixed table that maps every token id to a **canonical id** that
//! folds case and one leading space, used **only to decide which sources a
//! pointer key matches**. Emission is untouched: the pointer still copies the
//! token that stands at the matched source, in its own casing, and the
//! generated branch never sees the table.
//!
//! **Fold rule** ([`FOLD_RULE`]). A token whose decoded bytes are ASCII
//! letters, optionally after one leading space, belongs to the class of its
//! ASCII-lowercased letters; every other token (digits, punctuation, spaces,
//! non-ASCII, special tokens) is its own class. The canonical id of a class is
//! its least id. So `" Luk"`, `"luk"` and `" luk"` share a canonical id when
//! the vocabulary has them; `"<|im_start|>"` and `" 4"` stay themselves.
//! The table is idempotent (`c(c(i)) = c(i)`) and `c(i) <= i`. It does not
//! repair a key that BPE segments differently after capitalization (Step 0b:
//! 15 of the 41 D19 rehearsed keys); those need the generator fix (world=v2c).
//!
//! **D11 legality.** Serving the fold is one table read per token
//! (`vocab_size` u32 entries, fixed at export from the tokenizer) and integer
//! equality on the read values: no multiplier, no floating point, no learned
//! parameter. The ordered n-let admission ([`RouteAdmission::Ngram`]) compares
//! canonical ids instead of raw ids. Neither integer stack engine serves a
//! routed pointer yet (`stack_export` refuses one), so the table is not part
//! of an exported artifact today; [`CopyIdentity::to_json`] is the format a
//! port would carry, bound to the tokenizer by its address and digest.
//!
//! [`PrimeRoute`]: crate::geometric_stack::PrimeRoute
//! [`RouteAdmission::Ngram`]: crate::geometric_stack::RouteAdmission::Ngram

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uor_r4_tokenizer::ByteBpeTokenizer;

use crate::{invalid, Result};

/// The JSON schema of a saved table.
pub const COPY_IDENTITY_SCHEMA: &str = "uor-r4.copy-identity/1";
/// The fold the table implements (see the module documentation).
pub const FOLD_RULE: &str = "ascii-letters-case-and-one-leading-space/least-id";

/// A fixed map from each token id to its canonical id for pointer key
/// matching (see the module documentation).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CopyIdentity {
    canonical: Vec<u32>,
    /// The tokenizer's address (`ByteBpeTokenizer::address`), or empty for a
    /// table built from bare pieces.
    tokenizer: String,
}

/// The saved form.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Saved {
    schema: String,
    fold: String,
    tokenizer: String,
    digest: String,
    canonical: Vec<u32>,
}

/// The folded class key of one token's bytes, or `None` for a token that is
/// its own class.
fn fold_key(bytes: &[u8]) -> Option<Vec<u8>> {
    let letters = bytes.strip_prefix(b" ").unwrap_or(bytes);
    (!letters.is_empty() && letters.iter().all(u8::is_ascii_alphabetic))
        .then(|| letters.to_ascii_lowercase())
}

impl CopyIdentity {
    /// The table of a vocabulary whose token `i` decodes to `pieces[i]`.
    pub fn from_pieces(pieces: &[Vec<u8>]) -> Result<Self> {
        let size = u32::try_from(pieces.len())
            .map_err(|_| invalid("a copy-identity vocabulary must fit u32 ids"))?;
        let mut least: BTreeMap<Vec<u8>, u32> = BTreeMap::new();
        let mut canonical = Vec::with_capacity(pieces.len());
        for (id, piece) in (0..size).zip(pieces) {
            canonical.push(match fold_key(piece) {
                Some(key) => *least.entry(key).or_insert(id),
                None => id,
            });
        }
        Ok(Self {
            canonical,
            tokenizer: String::new(),
        })
    }

    /// The table of `tokenizer`'s whole id range, bound to its address.
    pub fn from_tokenizer(tokenizer: &ByteBpeTokenizer) -> Result<Self> {
        let size = u32::try_from(tokenizer.vocab_size())
            .map_err(|_| invalid("a copy-identity vocabulary must fit u32 ids"))?;
        let pieces: Vec<Vec<u8>> = (0..size).map(|id| tokenizer.decode_bytes(&[id])).collect();
        Ok(Self {
            tokenizer: tokenizer.address(),
            ..Self::from_pieces(&pieces)?
        })
    }

    /// Ids the table covers (the vocabulary size it was built from).
    pub fn len(&self) -> usize {
        self.canonical.len()
    }

    pub fn is_empty(&self) -> bool {
        self.canonical.is_empty()
    }

    /// The tokenizer address the table was built from (empty for bare pieces).
    pub fn tokenizer(&self) -> &str {
        &self.tokenizer
    }

    /// The canonical id of `id`; an id outside the table is its own.
    pub fn canonical(&self, id: u32) -> u32 {
        self.canonical.get(id as usize).copied().unwrap_or(id)
    }

    /// `ids` mapped through the table, for key matching only.
    pub fn fold(&self, ids: &[u32]) -> Vec<u32> {
        ids.iter().map(|&id| self.canonical(id)).collect()
    }

    /// How many ids map to another id.
    pub fn merged(&self) -> usize {
        (0u32..)
            .zip(&self.canonical)
            .filter(|(id, c)| *id != **c)
            .count()
    }

    /// SHA-256 over the schema, the fold rule and the table as little-endian
    /// u32s (the tokenizer address is recorded beside it, not hashed, so two
    /// tokenizers with one vocabulary have one table digest).
    pub fn digest(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(COPY_IDENTITY_SCHEMA.as_bytes());
        hasher.update([0u8]);
        hasher.update(FOLD_RULE.as_bytes());
        hasher.update([0u8]);
        for c in &self.canonical {
            hasher.update(c.to_le_bytes());
        }
        hex::encode(hasher.finalize())
    }

    fn validate(canonical: &[u32]) -> Result<()> {
        for (id, &c) in (0u32..).zip(canonical) {
            if c > id || canonical.get(c as usize) != Some(&c) {
                return Err(invalid(format!(
                    "copy-identity entry {id} -> {c} is not a least, idempotent class id"
                )));
            }
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(&Saved {
            schema: COPY_IDENTITY_SCHEMA.into(),
            fold: FOLD_RULE.into(),
            tokenizer: self.tokenizer.clone(),
            digest: self.digest(),
            canonical: self.canonical.clone(),
        })?)
    }

    /// A saved table, refused unless its schema, rule, digest and class
    /// structure all check.
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        let saved: Saved = serde_json::from_slice(bytes)?;
        if saved.schema != COPY_IDENTITY_SCHEMA || saved.fold != FOLD_RULE {
            return Err(invalid("not a copy-identity table of this schema and fold"));
        }
        Self::validate(&saved.canonical)?;
        let table = Self {
            canonical: saved.canonical,
            tokenizer: saved.tokenizer,
        };
        if table.digest() != saved.digest {
            return Err(invalid("copy-identity table digest mismatch"));
        }
        Ok(table)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        Ok(fs::write(path, self.to_json()?)?)
    }

    pub fn load(path: &Path) -> Result<Self> {
        Self::from_json(&fs::read(path)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pieces(texts: &[&str]) -> Vec<Vec<u8>> {
        texts.iter().map(|t| t.as_bytes().to_vec()).collect()
    }

    #[test]
    fn case_and_one_leading_space_fold_to_the_least_id() -> Result<()> {
        let vocab = pieces(&[
            "<|endoftext|>", // 0: special, its own
            " luk",          // 1
            " Luk",          // 2 -> 1
            "luk",           // 3 -> 1
            "LUK",           // 4 -> 1
            " 4",            // 5: digit, its own
            "4",             // 6: its own
            " ",             // 7: space only, its own
            "  luk",         // 8: two spaces, its own
            "é",             // 9: non-ASCII, its own
            ".",             // 10
            " is",           // 11
            "Is",            // 12 -> 11
            "<|im_start|>",  // 13
        ]);
        let table = CopyIdentity::from_pieces(&vocab)?;
        let expect = [0, 1, 1, 1, 1, 5, 6, 7, 8, 9, 10, 11, 11, 13];
        assert_eq!(table.fold(&(0..14).collect::<Vec<_>>()), expect);
        assert_eq!(table.merged(), 4);
        // An id outside the table is its own; folding is idempotent.
        assert_eq!(table.canonical(99), 99);
        assert_eq!(table.fold(&table.fold(&[2, 4, 12])), vec![1, 1, 11]);
        Ok(())
    }

    #[test]
    fn the_saved_table_round_trips_and_refuses_tampering() -> Result<()> {
        let table = CopyIdentity::from_pieces(&pieces(&["a", " A", "b", " B", "1"]))?;
        let bytes = table.to_json()?;
        let back = CopyIdentity::from_json(&bytes)?;
        assert_eq!(back, table);
        assert_eq!(back.digest(), table.digest());
        let mut saved: serde_json::Value = serde_json::from_slice(&bytes)?;
        saved["canonical"][1] = serde_json::json!(1);
        assert!(CopyIdentity::from_json(&serde_json::to_vec(&saved)?).is_err());
        saved["canonical"][1] = serde_json::json!(2);
        assert!(CopyIdentity::from_json(&serde_json::to_vec(&saved)?).is_err());
        Ok(())
    }

    #[test]
    fn a_byte_level_tokenizer_folds_a_capitalized_key_onto_the_asserted_one() -> Result<()> {
        // A tiny byte-level BPE whose vocabulary has " luk" and " Luk" whole.
        let json = byte_tokenizer_json(&["Ġluk", "ĠLuk"]);
        let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(json.as_bytes())
            .ok_or_else(|| invalid("test tokenizer"))?;
        let table = CopyIdentity::from_tokenizer(&tokenizer)?;
        assert_eq!(table.len(), tokenizer.vocab_size());
        assert_eq!(table.tokenizer(), tokenizer.address());
        let asserted = tokenizer.encode(" luk");
        let rehearsed = tokenizer.encode(" Luk");
        assert_eq!((asserted.len(), rehearsed.len()), (1, 1));
        assert_ne!(asserted, rehearsed);
        assert_eq!(table.fold(&asserted), table.fold(&rehearsed));
        // A digit and a punctuation byte keep their identity.
        let other = tokenizer.encode("4.");
        assert_eq!(table.fold(&other), other);
        // Special tokens are their own class.
        assert_eq!(table.fold(&[0, 1, 2]), vec![0, 1, 2]);
        // Single letters fold too, onto the class's least id: "L" (byte 76)
        // sits below "l" (byte 108).
        let (upper, lower) = (tokenizer.encode("L"), tokenizer.encode("l"));
        assert_eq!(table.fold(&lower), upper);
        Ok(())
    }

    /// A byte-level BPE tokenizer JSON: three special tokens, GPT-2's 256 byte
    /// symbols, then `words` (byte-level symbol strings) merged left to right
    /// from their bytes.
    fn byte_tokenizer_json(words: &[&str]) -> String {
        let mut vocab = serde_json::Map::new();
        for (id, surface) in ["<|bos|>", "<|eos|>", "<|unk|>"].iter().enumerate() {
            vocab.insert((*surface).to_owned(), serde_json::json!(id));
        }
        for (byte, ch) in (0u32..).zip(byte_symbols()) {
            vocab.insert(ch.to_string(), serde_json::json!(byte + 3));
        }
        let mut merges = Vec::new();
        for word in words {
            let chars: Vec<char> = word.chars().collect();
            let mut prefix = chars[0].to_string();
            for ch in &chars[1..] {
                merges.push(format!("{prefix} {ch}"));
                prefix.push(*ch);
                if !vocab.contains_key(&prefix) {
                    let id = vocab.len();
                    vocab.insert(prefix.clone(), serde_json::json!(id));
                }
            }
        }
        serde_json::json!({
            "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
            "added_tokens": [
                {"id": 0, "content": "<|bos|>"},
                {"id": 1, "content": "<|eos|>"},
                {"id": 2, "content": "<|unk|>"}
            ],
            "model": {"type": "BPE", "vocab": vocab, "merges": merges},
        })
        .to_string()
    }

    /// GPT-2's byte-to-symbol map, in byte order.
    fn byte_symbols() -> Vec<char> {
        let mut extra = 0u32;
        (0u32..256)
            .map(|byte| {
                let printable = (u32::from(b'!')..=u32::from(b'~')).contains(&byte)
                    || (0xA1..=0xAC).contains(&byte)
                    || (0xAE..=0xFF).contains(&byte);
                let code = if printable {
                    byte
                } else {
                    extra += 1;
                    255 + extra
                };
                char::from_u32(code).unwrap_or('?')
            })
            .collect()
    }
}
