//! Check every file `case::load` reads against the sha256 the saved case records.
//!
//! Ported from claude-50's `saved.rs` (`becd57e9c`). Two receipts cover the 384 files:
//!
//! - `coupled-gradient-receipt.json` `files[]`: exactly one entry for each of the four
//!   master and gradient files (other entries, such as the per-term files, are ignored);
//! - `protected-margin-receipt.json` `terms[i]`: `file` is `protected-margin-{i:03}`
//!   and `sha256` its hash, one term per guard row, in guard-index order.
//!
//! A missing receipt, a missing or duplicate entry, a misnamed term or a hash that
//! differs from the file's bytes is refused, naming the file.

use std::path::Path;

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::case::{BITS_SUFFIX, FAMILIES, GUARDS};

/// Receipt naming the master and gradient files.
pub const GRADIENT_RECEIPT: &str = "coupled-gradient-receipt.json";
/// Receipt naming the guard rows.
pub const MARGIN_RECEIPT: &str = "protected-margin-receipt.json";

/// One file whose bytes matched its receipt's sha256.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedHash {
    pub file: String,
    pub sha256: String,
}

/// Lower-case hex sha256 of `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn read_json(dir: &Path, file: &str) -> Result<Value, String> {
    let bytes = std::fs::read(dir.join(file)).map_err(|e| format!("receipt {file}: {e}"))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("receipt {file}: {e}"))
}

fn str_field<'a>(v: &'a Value, key: &str, receipt: &str, ctx: &str) -> Result<&'a str, String> {
    v.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("receipt {receipt}: {ctx}: missing string field `{key}`"))
}

/// The receipt sha256 for `file` in `coupled-gradient-receipt.json` `files[]`.
fn gradient_hash(receipt: &Value, file: &str) -> Result<String, String> {
    let entries = receipt
        .get("files")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("receipt {GRADIENT_RECEIPT}: missing array `files`"))?;
    let mut found = None;
    for e in entries {
        if e.get("file").and_then(Value::as_str) == Some(file) {
            if found.is_some() {
                return Err(format!(
                    "receipt {GRADIENT_RECEIPT}: {file} is listed more than once"
                ));
            }
            found = Some(str_field(e, "sha256", GRADIENT_RECEIPT, file)?.to_string());
        }
    }
    found.ok_or_else(|| format!("receipt {GRADIENT_RECEIPT}: {file} is not listed in `files`"))
}

fn check(
    dir: &Path,
    file: &str,
    expected: &str,
    out: &mut Vec<VerifiedHash>,
) -> Result<(), String> {
    let bytes = std::fs::read(dir.join(file)).map_err(|e| format!("cannot read {file}: {e}"))?;
    let actual = sha256_hex(&bytes);
    if actual != expected {
        return Err(format!(
            "sha256 mismatch for {file}: receipt says {expected}, file hashes to {actual}"
        ));
    }
    out.push(VerifiedHash {
        file: file.to_string(),
        sha256: actual,
    });
    Ok(())
}

/// Verify the 4 master/gradient files and the `GUARDS` guard rows against their receipts.
/// Returns one entry per verified file: the four family files first, then the guard rows.
pub fn verify(dir: &Path) -> Result<Vec<VerifiedHash>, String> {
    let mut verified = Vec::with_capacity(2 * FAMILIES.len() + GUARDS);

    let grad_receipt = read_json(dir, GRADIENT_RECEIPT)?;
    for (stem, _) in FAMILIES {
        for kind in ["initial-master", "gradient"] {
            let file = format!("coupled-{stem}-{kind}.{BITS_SUFFIX}");
            let expected = gradient_hash(&grad_receipt, &file)?;
            check(dir, &file, &expected, &mut verified)?;
        }
    }

    let margin_receipt = read_json(dir, MARGIN_RECEIPT)?;
    let terms = margin_receipt
        .get("terms")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("receipt {MARGIN_RECEIPT}: missing array `terms`"))?;
    if terms.len() != GUARDS {
        return Err(format!(
            "receipt {MARGIN_RECEIPT}: {} terms, expected one per guard row ({GUARDS})",
            terms.len()
        ));
    }
    for (i, term) in terms.iter().enumerate() {
        let file = format!("protected-margin-{i:03}.{BITS_SUFFIX}");
        let ctx = format!("terms[{i}]");
        let listed = str_field(term, "file", MARGIN_RECEIPT, &ctx)?;
        if listed != file {
            return Err(format!(
                "receipt {MARGIN_RECEIPT}: {ctx} names {listed}, expected {file}"
            ));
        }
        let expected = str_field(term, "sha256", MARGIN_RECEIPT, &ctx)?;
        check(dir, &file, expected, &mut verified)?;
    }
    Ok(verified)
}
