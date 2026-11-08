//! reply-cap: writes a copy of one SFT split (`manifest.json`, `tokens.u16`,
//! `response_mask.u8`) in which each EOS-ending reply sequence keeps its mask for its first `k`
//! occurrences in file order and is masked to 0 after that. A reply is a maximal run of
//! consecutive `mask == 1` positions; it ends on EOS when its last token is `eos_id`. Integer
//! arithmetic only; every inconsistency is a refusal.

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader;

const KEYS: [&str; 3] = ["split", "out", "k"];

type R<T> = Result<T, String>;

/// Entry point for `main`: `key=value` arguments; returns the `reply_cap` object as one JSON line.
pub fn run(args: &[String]) -> R<String> {
    let mut kv: BTreeMap<&str, &str> = BTreeMap::new();
    for item in args {
        let (k, v) = item
            .split_once('=')
            .ok_or(format!("argument {item:?}: expected key=value"))?;
        if !KEYS.contains(&k) {
            return Err(format!("unknown argument {k}="));
        }
        if kv.insert(k, v).is_some() {
            return Err(format!("{k}= given more than once"));
        }
    }
    let need = |k: &str| kv.get(k).copied().ok_or(format!("{k}= is required"));
    let (dir, out, kstr) = (PathBuf::from(need("split")?), PathBuf::from(need("out")?), need("k")?);
    let k = kstr
        .parse::<u64>()
        .ok()
        .filter(|&k| k >= 1)
        .ok_or(format!("k={kstr}: expected a positive integer"))?;
    if out.exists() {
        return Err(format!("{}: out already exists", out.display()));
    }

    let s = Split::load(&dir)?;
    if s.manifest.contains_key("reply_cap") {
        return Err(format!(
            "{}: manifest already has reply_cap",
            dir.join("manifest.json").display()
        ));
    }
    let (mask, cap) = cap_mask(s.tokens(), &s.mask, s.eos, k);
    let mask_sha = sha_bytes(&mask);
    let summary = json!({
        "k": k,
        "input_tokens_sha256": s.tokens_sha,
        "input_mask_sha256": s.mask_sha,
        "replies": cap.replies,
        "eos_replies": cap.eos_replies,
        "eos_replies_kept": cap.eos_replies_kept,
        "eos_replies_masked": cap.eos_replies_masked,
        "distinct_eos_replies": cap.distinct_eos_replies,
        "scored_before": cap.scored_before,
        "scored_after": cap.scored_after,
    });
    let mut manifest = s.manifest.clone();
    let tag = format!(" (reply-cap k={k})");
    for key in ["source", "revision"] {
        if let Some(Value::String(v)) = manifest.get_mut(key) {
            v.push_str(&tag);
        }
    }
    // The tokens file is copied byte for byte, so its sha256 is the input's.
    manifest.insert("tokens_sha256".into(), json!(s.tokens_sha));
    manifest.insert("mask_sha256".into(), json!(mask_sha));
    manifest.insert("reply_cap".into(), summary.clone());

    if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    fs::create_dir(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    write(&out.join("tokens.u16"), &s.tokens_bytes)?;
    write(&out.join("response_mask.u8"), &mask)?;
    let mut m = serde_json::to_vec_pretty(&Value::Object(manifest)).expect("json value serialises");
    m.push(b'\n');
    write(&out.join("manifest.json"), &m)?;
    Ok(serde_json::to_string(&summary).expect("json value serialises"))
}

/// One split on disk, validated.
pub struct Split {
    reader: MmapCorpusReader,
    tokens_bytes: Vec<u8>,
    mask: Vec<u8>,
    eos: u16,
    manifest: serde_json::Map<String, Value>,
    tokens_sha: String,
    mask_sha: String,
}

fn read(path: &Path) -> R<Vec<u8>> {
    fs::read(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn write(path: &Path, bytes: &[u8]) -> R<()> {
    fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))
}

fn sha_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

impl Split {
    pub fn load(dir: &Path) -> R<Split> {
        let mp = dir.join("manifest.json");
        let manifest: Value = serde_json::from_slice(&read(&mp)?)
            .map_err(|e| format!("{}: not JSON: {e}", mp.display()))?;
        let manifest = match manifest {
            Value::Object(m) => m,
            _ => return Err(format!("{}: not a JSON object", mp.display())),
        };
        let eos = match manifest.get("eos_id") {
            None | Some(Value::Null) => return Err(format!("{}: no eos_id", mp.display())),
            Some(v) => v
                .as_u64()
                .filter(|&x| x <= u16::MAX as u64)
                .map(|x| x as u16)
                .ok_or(format!("{}: eos_id {v} is not a u16", mp.display()))?,
        };

        let tp = dir.join("tokens.u16");
        let tokens_bytes = read(&tp)?;
        let tokens_sha = sha_bytes(&tokens_bytes);
        let reader =
            MmapCorpusReader::open(&tp).map_err(|e| format!("{}: {e}", tp.display()))?;
        let kp = dir.join("response_mask.u8");
        let mask = read(&kp)?;
        let mask_sha = sha_bytes(&mask);
        if mask.len() != reader.total_tokens() {
            return Err(format!(
                "{}: {} bytes, but {} holds {} tokens",
                kp.display(),
                mask.len(),
                tp.display(),
                reader.total_tokens()
            ));
        }
        if let Some(p) = mask.iter().position(|&b| b > 1) {
            return Err(format!(
                "{}: byte {} at position {p} is not 0 or 1",
                kp.display(),
                mask[p]
            ));
        }
        for (key, actual) in [("tokens_sha256", &tokens_sha), ("mask_sha256", &mask_sha)] {
            if let Some(v) = manifest.get(key) {
                let declared = v
                    .as_str()
                    .ok_or(format!("{}: {key} is not a string", mp.display()))?;
                if !declared.eq_ignore_ascii_case(actual) {
                    return Err(format!(
                        "{}: {key} {declared} does not match the file's {actual}",
                        mp.display()
                    ));
                }
            }
        }
        Ok(Split {
            reader,
            tokens_bytes,
            mask,
            eos,
            manifest,
            tokens_sha,
            mask_sha,
        })
    }

    pub fn tokens(&self) -> &[u16] {
        self.reader.as_slice()
    }
}

/// Half-open `[start, end)` ranges of the maximal runs of `mask == 1`.
pub fn reply_runs(mask: &[u8]) -> Vec<(usize, usize)> {
    let mut runs = Vec::new();
    let mut start = None;
    for (i, &m) in mask.iter().enumerate() {
        match (m, start) {
            (1, None) => start = Some(i),
            (0, Some(s)) => {
                runs.push((s, i));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        runs.push((s, mask.len()));
    }
    runs
}

#[derive(Debug, PartialEq, Eq)]
pub struct Cap {
    pub replies: u64,
    pub eos_replies: u64,
    pub eos_replies_kept: u64,
    pub eos_replies_masked: u64,
    pub distinct_eos_replies: u64,
    pub scored_before: u64,
    pub scored_after: u64,
}

/// The capped mask. `tokens` and `mask` have equal length and the mask holds only 0/1 (checked
/// at load). The j-th occurrence (1-based, file order) of an EOS-ending reply keeps its mask when
/// `j <= k`; otherwise every byte of its run becomes 0.
pub fn cap_mask(tokens: &[u16], mask: &[u8], eos: u16, k: u64) -> (Vec<u8>, Cap) {
    let mut out = mask.to_vec();
    let runs = reply_runs(mask);
    let mut seen: BTreeMap<&[u16], u64> = BTreeMap::new();
    let (mut eos_replies, mut kept, mut masked) = (0u64, 0u64, 0u64);
    for &(s, e) in &runs {
        let r = &tokens[s..e];
        if r[r.len() - 1] != eos {
            continue;
        }
        eos_replies += 1;
        let j = seen.entry(r).or_insert(0);
        *j += 1;
        if *j <= k {
            kept += 1;
        } else {
            masked += 1;
            out[s..e].fill(0);
        }
    }
    let ones = |m: &[u8]| m.iter().filter(|&&b| b == 1).count() as u64;
    let cap = Cap {
        replies: runs.len() as u64,
        eos_replies,
        eos_replies_kept: kept,
        eos_replies_masked: masked,
        distinct_eos_replies: seen.len() as u64,
        scored_before: ones(mask),
        scored_after: ones(&out),
    };
    (out, cap)
}

/// The trainer's usable-window count (claude-321). It restates `build_document_windows` in
/// `crates/uor-r4-float-train/src/hamilton_sheet.rs` and the filter in `train_windows`
/// (`crates/uor-r4-float-train/src/in_space_long.rs`). Each `bos`-delimited document's target
/// positions `doc_start + 1 .. doc_end` are cut into chunks of at most `context`. A chunk is a
/// window when at least one target is scored (`tokens[p] != bos && mask[p] == 1`), and usable when
/// at least `min_scored` are. Positions before the first `bos` belong to no document.
/// Returns `(documents, windows, usable)`.
pub fn usable_windows(
    tokens: &[u16],
    mask: &[u8],
    bos: u16,
    context: usize,
    min_scored: usize,
) -> (u64, u64, u64) {
    let starts: Vec<usize> = (0..tokens.len()).filter(|&i| tokens[i] == bos).collect();
    let (mut windows, mut usable) = (0u64, 0u64);
    for (d, &start) in starts.iter().enumerate() {
        let end = starts.get(d + 1).copied().unwrap_or(tokens.len());
        let mut cursor = start + 1;
        while cursor < end {
            let stop = (cursor + context).min(end);
            let n = (cursor..stop)
                .filter(|&p| tokens[p] != bos && mask[p] == 1)
                .count();
            if n > 0 {
                windows += 1;
            }
            if n >= min_scored {
                usable += 1;
            }
            cursor = stop;
        }
    }
    (starts.len() as u64, windows, usable)
}

/// The trainer's filter: a window is usable with at least this many scored targets.
pub const MIN_SCORED: usize = 4;

/// `window-count split=DIR context=N`: the trainer's usable TRAIN windows for one split, with
/// `bos` 0 (the sheet's BOS id). The split loads as for `reply-cap`, so its manifest needs
/// `eos_id`. Returns one JSON line.
pub fn run_window_count(args: &[String]) -> R<String> {
    let mut kv: BTreeMap<&str, &str> = BTreeMap::new();
    for item in args {
        let (k, v) = item
            .split_once('=')
            .ok_or(format!("argument {item:?}: expected key=value"))?;
        if !["split", "context"].contains(&k) {
            return Err(format!("unknown argument {k}="));
        }
        if kv.insert(k, v).is_some() {
            return Err(format!("{k}= given more than once"));
        }
    }
    let need = |k: &str| kv.get(k).copied().ok_or(format!("{k}= is required"));
    let dir = PathBuf::from(need("split")?);
    let cstr = need("context")?;
    let context = cstr
        .parse::<usize>()
        .ok()
        .filter(|&c| c >= 1)
        .ok_or(format!("context={cstr}: expected a positive integer"))?;
    let s = Split::load(&dir)?;
    if let Some(b) = s.manifest.get("bos_id") {
        if b.as_u64() != Some(0) {
            return Err(format!("{}: bos_id {b} is not 0", dir.join("manifest.json").display()));
        }
    }
    let tokens = s.tokens();
    if tokens.first().copied() != Some(0) {
        return Err(format!("{}: the first token is not BOS (0)", dir.join("tokens.u16").display()));
    }
    let (documents, windows, usable) = usable_windows(tokens, &s.mask, 0, context, MIN_SCORED);
    let v = json!({
        "split_tokens_sha256": s.tokens_sha,
        "split_mask_sha256": s.mask_sha,
        "context": context,
        "min_scored": MIN_SCORED,
        "documents": documents,
        "windows": windows,
        "usable_windows": usable,
    });
    Ok(serde_json::to_string(&v).expect("json value serialises"))
}
