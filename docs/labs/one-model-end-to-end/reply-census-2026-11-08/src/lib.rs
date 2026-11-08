//! reply-census: counts the scored replies of one SFT split (`manifest.json`, `tokens.u16`,
//! `response_mask.u8`), optionally against a second split. A reply is a maximal run of
//! consecutive `mask == 1` positions. Counting only; integer arithmetic only; every
//! inconsistency is a refusal.

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader;

pub const SCHEMA: &str = "uor-r4-reply-census/v1";
/// Window length of the reply n-gram census; 8 x 16 bits pack exactly into a u128.
pub const NGRAM: usize = 8;
/// Ids shown per top reply.
pub const FIRST_IDS: usize = 16;
pub const DEFAULT_TOP: usize = 10;
const KEYS: [&str; 5] = ["split", "name", "against", "against_name", "top"];

type R<T> = Result<T, String>;

/// Entry point for `main`: `key=value` arguments; returns one JSON object.
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
    let (dir, name) = (PathBuf::from(need("split")?), need("name")?);
    let top = match kv.get("top") {
        None => DEFAULT_TOP,
        Some(v) => v
            .parse::<usize>()
            .map_err(|_| format!("top={v}: expected a non-negative integer"))?,
    };
    let against = match (kv.get("against"), kv.get("against_name")) {
        (Some(d), Some(n)) => Some((PathBuf::from(d), *n)),
        (None, None) => None,
        _ => return Err("against= and against_name= go together".into()),
    };

    let a = Split::load(&dir)?;
    let ca = census(a.tokens(), &a.mask, a.eos, a.bos);
    let mut out = json!({
        "schema": SCHEMA,
        "name": name,
        "split": dir.display().to_string(),
        "sha256": a.sha256(),
        "eos_id": a.eos,
        "bos_id": a.bos,
        "documents": ca.documents,
        "tokens": ca.tokens,
        "scored_targets": ca.scored_targets,
        "eos_targets": ca.eos_targets,
        "replies": ca.replies,
        "replies_ending_eos": ca.replies_ending_eos,
        "eos_not_at_reply_end": ca.eos_not_at_reply_end,
        "distinct_replies": ca.reply_counts.len(),
        "reply_length": {
            "median_lower": ca.len_median,
            "min": ca.len_min,
            "max": ca.len_max,
        },
        "eos_in_repeated": share(ca.eos_in_repeated, ca.replies_ending_eos),
        "eos_in_top1": share(ca.eos_in_top1, ca.replies_ending_eos),
        "reply_8grams": { "windows": ca.gram_windows, "distinct": ca.grams.len() },
        "top": top_replies(&ca, top),
        "against": null,
    });
    if let Some((bdir, bname)) = against {
        let b = Split::load(&bdir)?;
        if b.eos != a.eos {
            return Err(format!(
                "{}: eos_id {} differs from {}: eos_id {}",
                bdir.display(),
                b.eos,
                dir.display(),
                a.eos
            ));
        }
        let cb = census(b.tokens(), &b.mask, b.eos, b.bos);
        let (mut hit_replies, mut hit_distinct) = (0u64, 0u64);
        for (seq, &n) in &ca.reply_counts {
            if cb.reply_counts.contains_key(seq) {
                hit_replies += n;
                hit_distinct += 1;
            }
        }
        let hit_grams = ca
            .grams
            .iter()
            .filter(|g| cb.grams.binary_search(g).is_ok())
            .count() as u64;
        out["against"] = json!({
            "name": bname,
            "split": bdir.display().to_string(),
            "sha256": b.sha256(),
            "replies": cb.replies,
            "distinct_replies": cb.reply_counts.len(),
            "distinct_8grams": cb.grams.len(),
            "replies_in_against": share(hit_replies, ca.replies),
            "distinct_replies_in_against": share(hit_distinct, ca.reply_counts.len() as u64),
            "distinct_8grams_in_against": share(hit_grams, ca.grams.len() as u64),
        });
    }
    let mut s = serde_json::to_string_pretty(&out).expect("json value serialises");
    s.push('\n');
    Ok(s)
}

/// `count` of `of`, with parts per million rounded down (null when `of` is 0).
fn share(count: u64, of: u64) -> Value {
    let ppm = (of != 0).then(|| count as u128 * 1_000_000 / of as u128);
    json!({ "count": count, "of": of, "ppm": ppm })
}

fn top_replies(c: &Census, top: usize) -> Value {
    let mut v: Vec<(&[u16], u64)> = c.reply_counts.iter().map(|(s, &n)| (*s, n)).collect();
    v.sort_by(|x, y| y.1.cmp(&x.1).then_with(|| x.0.cmp(y.0)));
    v.iter()
        .take(top)
        .map(|(s, n)| {
            json!({
                "count": n,
                "length": s.len(),
                "first_ids": &s[..s.len().min(FIRST_IDS)],
            })
        })
        .collect()
}

/// One split on disk, validated.
pub struct Split {
    reader: MmapCorpusReader,
    mask: Vec<u8>,
    eos: u16,
    bos: Option<u16>,
    sha: [String; 3],
}

fn read(path: &Path) -> R<Vec<u8>> {
    fs::read(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn sha_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn id_field(m: &serde_json::Map<String, Value>, key: &str, path: &Path) -> R<Option<u16>> {
    match m.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => v
            .as_u64()
            .filter(|&x| x <= u16::MAX as u64)
            .map(|x| Some(x as u16))
            .ok_or(format!("{}: {key} {v} is not a u16", path.display())),
    }
}

impl Split {
    pub fn load(dir: &Path) -> R<Split> {
        let mp = dir.join("manifest.json");
        let mbytes = read(&mp)?;
        let manifest: Value = serde_json::from_slice(&mbytes)
            .map_err(|e| format!("{}: not JSON: {e}", mp.display()))?;
        let m = manifest
            .as_object()
            .ok_or(format!("{}: not a JSON object", mp.display()))?;
        let eos = id_field(m, "eos_id", &mp)?.ok_or(format!("{}: no eos_id", mp.display()))?;
        let bos = id_field(m, "bos_id", &mp)?;

        let tp = dir.join("tokens.u16");
        let tsha = sha_bytes(&read(&tp)?);
        let reader =
            MmapCorpusReader::open(&tp).map_err(|e| format!("{}: {e}", tp.display()))?;
        let kp = dir.join("response_mask.u8");
        let mask = read(&kp)?;
        let msha = sha_bytes(&mask);
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
        for (key, actual) in [("tokens_sha256", &tsha), ("mask_sha256", &msha)] {
            if let Some(v) = m.get(key) {
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
            mask,
            eos,
            bos,
            sha: [sha_bytes(&mbytes), tsha, msha],
        })
    }

    pub fn tokens(&self) -> &[u16] {
        self.reader.as_slice()
    }

    fn sha256(&self) -> Value {
        json!({
            "manifest.json": self.sha[0],
            "tokens.u16": self.sha[1],
            "response_mask.u8": self.sha[2],
        })
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

/// Eight ids packed exactly, 16 bits each, first id highest.
pub fn pack(w: &[u16]) -> u128 {
    w.iter().fold(0u128, |k, &t| (k << 16) | t as u128)
}

pub struct Census<'a> {
    pub tokens: u64,
    pub documents: Option<u64>,
    pub scored_targets: u64,
    pub eos_targets: u64,
    pub replies: u64,
    pub replies_ending_eos: u64,
    pub eos_not_at_reply_end: u64,
    pub reply_counts: BTreeMap<&'a [u16], u64>,
    pub eos_in_repeated: u64,
    pub eos_in_top1: u64,
    pub gram_windows: u64,
    /// Sorted, deduplicated packed reply 8-grams.
    pub grams: Vec<u128>,
    pub len_median: Option<u64>,
    pub len_min: Option<u64>,
    pub len_max: Option<u64>,
}

/// `tokens` and `mask` have equal length and the mask holds only 0/1 (checked at load).
pub fn census<'a>(tokens: &'a [u16], mask: &[u8], eos: u16, bos: Option<u16>) -> Census<'a> {
    let documents = bos.map(|b| tokens.iter().filter(|&&t| t == b).count() as u64);
    let scored_targets = mask.iter().filter(|&&m| m == 1).count() as u64;
    let eos_targets = tokens
        .iter()
        .zip(mask)
        .filter(|&(&t, &m)| m == 1 && t == eos)
        .count() as u64;
    let runs = reply_runs(mask);
    let mut reply_counts: BTreeMap<&[u16], u64> = BTreeMap::new();
    let (mut ending, mut inner, mut windows) = (0u64, 0u64, 0u64);
    let mut lens = Vec::with_capacity(runs.len());
    let mut grams = Vec::new();
    for &(s, e) in &runs {
        let r = &tokens[s..e];
        if r[r.len() - 1] == eos {
            ending += 1;
        }
        inner += r[..r.len() - 1].iter().filter(|&&t| t == eos).count() as u64;
        *reply_counts.entry(r).or_insert(0) += 1;
        lens.push(r.len() as u64);
        if r.len() >= NGRAM {
            windows += (r.len() - NGRAM + 1) as u64;
            grams.extend(r.windows(NGRAM).map(pack));
        }
    }
    grams.sort_unstable();
    grams.dedup();
    let (mut repeated, mut top1) = (0u64, 0u64);
    for (seq, &n) in &reply_counts {
        if seq.last() == Some(&eos) {
            if n >= 2 {
                repeated += n;
            }
            top1 = top1.max(n);
        }
    }
    lens.sort_unstable();
    let len_median = (!lens.is_empty()).then(|| lens[(lens.len() - 1) / 2]);
    Census {
        tokens: tokens.len() as u64,
        documents,
        scored_targets,
        eos_targets,
        replies: runs.len() as u64,
        replies_ending_eos: ending,
        eos_not_at_reply_end: inner,
        reply_counts,
        eos_in_repeated: repeated,
        eos_in_top1: top1,
        gram_windows: windows,
        grams,
        len_median,
        len_min: lens.first().copied(),
        len_max: lens.last().copied(),
    }
}
