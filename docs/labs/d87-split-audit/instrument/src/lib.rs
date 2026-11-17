//! The D87 donor corpus split by prompt, deduplicated and audited (claude-411).
//!
//! Split key: a keyed SHA-256 of the bank's `normalize` of a record's FIRST user message.
//! D87 records carry no prompt-id field; the bank (claude-400) derives each id from that
//! same normalized text, so every alternative or re-generation of one prompt shares the key.
//! The bucket is the key's first 8 bytes (big-endian) modulo the ratios' common denominator.
//!
//! Duplicates, judged in a fixed order (shard paths sorted, then line order), first kept:
//! exact (SHA-256 of the whole line), repeated id, and near (SHA-256 of the normalized
//! answer text of every turn, when it is at least `near_floor` bytes long).
//!
//! Audit: every 13-gram of a record's token stream (BOS..EOS, never across records) is packed
//! exactly, 13 ids of 12 bits in three u64 words, so a match is an exact 13-gram compare.
//! Each DEV position (the last token of a 13-gram) counts once if its gram occurs anywhere in
//! TRAIN. All counts are integers; ratios are [numerator, denominator] pairs.

use std::collections::{BTreeMap, HashSet};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;

use m2_student_corpus_ingest::{hex, parse_record, read_shard, Coder, Ingest, Segment, SEGMENTS, UORT_HEADER};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

/// The n of the overlap census.
pub const GRAM: usize = 13;
/// Bits per token id in a packed gram (ids are below 4,096).
const ID_BITS: u32 = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Split {
    Train,
    DevTune,
    DevTest,
}
pub const SPLITS: [Split; 3] = [Split::Train, Split::DevTune, Split::DevTest];
impl Split {
    pub fn name(self) -> &'static str {
        match self {
            Split::Train => "train",
            Split::DevTune => "dev-tune",
            Split::DevTest => "dev-test",
        }
    }
    fn ix(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dup {
    Exact,
    Id,
    Near,
}
const DUPS: [Dup; 3] = [Dup::Exact, Dup::Id, Dup::Near];
impl Dup {
    fn name(self) -> &'static str {
        match self {
            Dup::Exact => "exact",
            Dup::Id => "id",
            Dup::Near => "near",
        }
    }
}

/// The bank's prompt normalization (`docs/labs/m2-donor-corpus/instrument/src/prompts.rs`),
/// copied: alphanumerics lowercased, everything else a space, whitespace runs collapsed.
pub fn normalize(s: &str) -> String {
    let mapped: String = s
        .chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { ' ' })
        .collect();
    mapped.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The template key (claude-411 regroup): `normalize`, then every run of digits replaced by
/// `#`, so one question asked with different numbers is one group.
pub fn template_key(s: &str) -> String {
    let mut out = String::new();
    let mut in_run = false;
    for c in normalize(s).chars() {
        if c.is_numeric() {
            if !in_run {
                out.push('#');
            }
            in_run = true;
        } else {
            out.push(c);
            in_run = false;
        }
    }
    out
}

/// What the split key groups: the normalized first user message, or its template key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    Prompt,
    Template,
}
impl Group {
    pub fn name(self) -> &'static str {
        match self {
            Group::Prompt => "prompt",
            Group::Template => "template",
        }
    }
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "prompt" => Ok(Group::Prompt),
            "template" => Ok(Group::Template),
            _ => Err(format!("group {s} is not prompt or template")),
        }
    }
}

fn sha(parts: &[&[u8]]) -> [u8; 32] {
    let mut h = Sha256::new();
    for p in parts {
        h.update(p);
    }
    h.finalize().into()
}

/// Split parameters. Ratios are integer fractions of prompts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub key: String,
    pub dev_tune: (u64, u64),
    pub dev_test: (u64, u64),
    pub near_floor: usize,
    pub group: Group,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            key: "claude-411/d87-split/v1".into(),
            dev_tune: (1, 200),
            dev_test: (1, 100),
            near_floor: 32,
            group: Group::Prompt,
        }
    }
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// A validated config: the bucket modulus and the two cut points.
#[derive(Clone, Debug)]
pub struct Splitter {
    pub cfg: Config,
    pub modulus: u64,
    test_cut: u64,
    tune_cut: u64,
}

impl Splitter {
    pub fn new(cfg: Config) -> Result<Self, String> {
        let (tn, td) = cfg.dev_tune;
        let (sn, sd) = cfg.dev_test;
        if td == 0 || sd == 0 || tn > td || sn > sd {
            return Err("ratios must be N/D with 0 <= N <= D and D > 0".into());
        }
        let modulus = td / gcd(td, sd) * sd;
        let test_cut = sn * (modulus / sd);
        let tune_cut = test_cut + tn * (modulus / td);
        if tune_cut > modulus {
            return Err("DEV-tune plus DEV-test exceed all prompts".into());
        }
        Ok(Self { cfg, modulus, test_cut, tune_cut })
    }

    /// The prompt key and split of a record whose first user message is `first_user`.
    pub fn split_of(&self, first_user: &str) -> ([u8; 32], Split) {
        let text = match self.cfg.group {
            Group::Prompt => normalize(first_user),
            Group::Template => template_key(first_user),
        };
        let key = sha(&[self.cfg.key.as_bytes(), &[0], text.as_bytes()]);
        let bucket = u64::from_be_bytes(key[..8].try_into().unwrap()) % self.modulus;
        let split = if bucket < self.test_cut {
            Split::DevTest
        } else if bucket < self.tune_cut {
            Split::DevTune
        } else {
            Split::Train
        };
        (key, split)
    }

    pub fn config_json(&self) -> Value {
        let c = &self.cfg;
        let mut v = json!({"key": c.key, "dev_tune": [c.dev_tune.0, c.dev_tune.1],
               "dev_test": [c.dev_test.0, c.dev_test.1], "near_floor": c.near_floor,
               "bucket_modulus": self.modulus});
        // The prompt group writes no field, so a first-run split-summary.json still reproduces.
        if c.group != Group::Prompt {
            v["group"] = c.group.name().into();
        }
        v
    }

    pub fn from_config_json(v: &Value) -> Result<Self, String> {
        let pair = |k: &str| -> Result<(u64, u64), String> {
            let a = v[k].as_array().ok_or(format!("config.{k}"))?;
            match (a.first().and_then(Value::as_u64), a.get(1).and_then(Value::as_u64)) {
                (Some(n), Some(d)) if a.len() == 2 => Ok((n, d)),
                _ => Err(format!("config.{k} is not [N, D]")),
            }
        };
        Self::new(Config {
            key: v["key"].as_str().ok_or("config.key")?.to_string(),
            dev_tune: pair("dev_tune")?,
            dev_test: pair("dev_test")?,
            near_floor: v["near_floor"].as_u64().ok_or("config.near_floor")? as usize,
            group: match v.get("group") {
                None => Group::Prompt,
                Some(g) => Group::parse(g.as_str().ok_or("config.group")?)?,
            },
        })
    }
}

/// What the split needs from one record line.
#[derive(Clone, Debug)]
pub struct Info {
    pub id: String,
    pub kind: String,
    pub source: String,
    pub prompt: [u8; 32],
    pub split: Split,
    line: [u8; 32],
    answer: Option<[u8; 32]>,
}

/// Validate one line as a D87 record (through student-ingest's parser) and key it.
pub fn scan_line(sp: &Splitter, line: &str) -> Result<Info, String> {
    let rec = parse_record(line)?;
    let v: Value = serde_json::from_str(line).map_err(|e| e.to_string())?;
    let id = rec.id.clone();
    let source = v["source"].as_str().ok_or(format!("record {id}: not a D87 record"))?;
    let first = v["messages"][0]["content"].as_str().ok_or(format!("record {id}: no first message"))?;
    let turns = v["turns"].as_array().ok_or(format!("record {id}: no turns"))?;
    let answers: Vec<&str> = turns.iter().filter_map(|t| t["answer"].as_str()).collect();
    let norm = normalize(&answers.join(" "));
    let (prompt, split) = sp.split_of(first);
    Ok(Info {
        id,
        kind: rec.kind,
        source: source.to_string(),
        prompt,
        split,
        line: sha(&[line.as_bytes()]),
        answer: (norm.len() >= sp.cfg.near_floor).then(|| sha(&[norm.as_bytes()])),
    })
}

/// Run `f` over shards on `threads` threads; results come back in shard order.
fn par_shards<T: Send>(
    paths: &[PathBuf],
    threads: usize,
    f: impl Fn(&Path) -> Result<T, String> + Sync,
) -> Result<Vec<T>, String> {
    let next = AtomicUsize::new(0);
    let slots: Vec<Mutex<Option<Result<T, String>>>> = paths.iter().map(|_| Mutex::new(None)).collect();
    std::thread::scope(|s| {
        for _ in 0..threads.clamp(1, paths.len().max(1)) {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= paths.len() {
                    break;
                }
                let r = f(&paths[i]).map_err(|e| format!("{}: {e}", paths[i].display()));
                *slots[i].lock().unwrap() = Some(r);
            });
        }
    });
    slots.into_iter().map(|m| m.into_inner().unwrap().unwrap()).collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fate {
    Keep(Split),
    Drop(Split, Dup),
}

#[derive(Default)]
struct SplitCount {
    ids: Vec<String>,
    prompts: HashSet<[u8; 32]>,
    kinds: BTreeMap<String, u64>,
    sources: BTreeMap<String, u64>,
    dropped: [u64; 3],
}

/// The split of a set of shards: per shard its sha256 and the fate of every line.
pub struct Plan {
    pub splitter: Splitter,
    pub shards: Vec<(PathBuf, String)>,
    pub fate: Vec<Vec<Fate>>,
    counts: Vec<SplitCount>,
}

pub fn plan(sp: &Splitter, paths: &[PathBuf], threads: usize) -> Result<Plan, String> {
    let mut paths = paths.to_vec();
    paths.sort();
    if paths.windows(2).any(|w| w[0] == w[1]) {
        return Err("a shard is given twice".into());
    }
    let scanned = par_shards(&paths, threads, |p| {
        let mut infos = vec![];
        let sha = read_shard(p, &mut |line| {
            infos.push(scan_line(sp, line)?);
            Ok(())
        })?;
        Ok((sha, infos))
    })?;
    let (mut lines, mut ids, mut answers) = (HashSet::new(), HashSet::new(), HashSet::new());
    let mut counts: Vec<SplitCount> = SPLITS.iter().map(|_| SplitCount::default()).collect();
    let (mut shards, mut fate) = (vec![], vec![]);
    for (p, (sha, infos)) in paths.into_iter().zip(scanned) {
        let mut f = Vec::with_capacity(infos.len());
        for info in infos {
            let dup = if !lines.insert(info.line) {
                Some(Dup::Exact)
            } else if !ids.insert(info.id.clone()) {
                Some(Dup::Id)
            } else if info.answer.is_some_and(|a| !answers.insert(a)) {
                Some(Dup::Near)
            } else {
                None
            };
            let c = &mut counts[info.split.ix()];
            if let Some(d) = dup {
                c.dropped[d as usize] += 1;
                f.push(Fate::Drop(info.split, d));
                continue;
            }
            c.prompts.insert(info.prompt);
            *c.kinds.entry(info.kind).or_insert(0) += 1;
            *c.sources.entry(info.source).or_insert(0) += 1;
            c.ids.push(info.id);
            f.push(Fate::Keep(info.split));
        }
        shards.push((p, sha));
        fate.push(f);
    }
    for c in &mut counts {
        c.ids.sort();
    }
    Ok(Plan { splitter: sp.clone(), shards, fate, counts })
}

impl Plan {
    /// The sorted-id manifest text of one split.
    pub fn manifest(&self, s: Split) -> String {
        self.counts[s.ix()].ids.iter().map(|i| format!("{i}\n")).collect()
    }

    pub fn summary(&self) -> Value {
        let total_records: u64 = self.counts.iter().map(|c| c.ids.len() as u64).sum();
        let total_prompts: u64 = self.counts.iter().map(|c| c.prompts.len() as u64).sum();
        let mut splits = serde_json::Map::new();
        let mut dropped_total = [0u64; 3];
        for s in SPLITS {
            let c = &self.counts[s.ix()];
            let dropped: serde_json::Map<String, Value> =
                DUPS.iter().map(|&d| (d.name().to_string(), json!(c.dropped[d as usize]))).collect();
            for d in DUPS {
                dropped_total[d as usize] += c.dropped[d as usize];
            }
            splits.insert(s.name().into(), json!({
                "records": c.ids.len(), "prompts": c.prompts.len(), "kinds": c.kinds,
                "sources": c.sources, "dropped": dropped,
                "prompt_share": [c.prompts.len(), total_prompts],
                "ids_sha256": hex(&sha(&[self.manifest(s).as_bytes()])),
            }));
        }
        let inputs: Vec<Value> = self.shards.iter().zip(&self.fate)
            .map(|((p, h), f)| json!({"path": p.display().to_string(), "sha256": h, "lines": f.len()}))
            .collect();
        let lines: u64 = self.fate.iter().map(|f| f.len() as u64).sum();
        let dropped: serde_json::Map<String, Value> =
            DUPS.iter().map(|&d| (d.name().to_string(), json!(dropped_total[d as usize]))).collect();
        let split_key = match self.splitter.cfg.group {
            Group::Prompt => "sha256(key || 0x00 || normalize(first user message)); bucket = first 8 bytes big-endian mod bucket_modulus; bucket < dev_test cut is dev-test, then dev-tune, else train",
            Group::Template => "sha256(key || 0x00 || template_key(first user message)), template_key = normalize with every digit run replaced by #; bucket = first 8 bytes big-endian mod bucket_modulus; bucket < dev_test cut is dev-test, then dev-tune, else train; prompts counts template keys",
        };
        json!({
            "unit": "claude-411",
            "config": self.splitter.config_json(),
            "rules": {
                "split_key": split_key,
                "order": "shard paths sorted, then line order; the first record is kept",
                "exact": "sha256 of the whole line", "id": "a repeated id",
                "near": "sha256 of normalize(every turn's answer joined by a space), when at least near_floor bytes",
            },
            "inputs": inputs, "lines": lines, "records": total_records, "prompts": total_prompts,
            "dropped": dropped, "splits": splits,
            "ratios": "every pair is an exact integer [numerator, denominator]",
        })
    }

    /// Write `<split>.ids` and `split-summary.json` into `out`.
    pub fn write(&self, out: &Path) -> Result<String, String> {
        for (p, _) in &self.shards {
            if p.parent() == Some(out) {
                return Err(format!("{} is inside --out", p.display()));
            }
        }
        std::fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
        for s in SPLITS {
            std::fs::write(out.join(format!("{}.ids", s.name())), self.manifest(s)).map_err(|e| e.to_string())?;
        }
        let text = serde_json::to_string_pretty(&self.summary()).map_err(|e| e.to_string())? + "\n";
        std::fs::write(out.join("split-summary.json"), &text).map_err(|e| e.to_string())?;
        Ok(text)
    }
}

/// One tokenized record: student-ingest's stream for it, with segment tags.
pub struct Tok {
    pub id: String,
    pub tokens: Vec<u32>,
    pub segments: Vec<u8>,
    pub mask: Vec<u8>,
    pub response: u64,
    pub seg_bytes: [u64; 4],
    pub seg_tokens: [u64; 4],
    /// Text bytes the vocabulary cannot emit, so no token holds them (claude-420).
    pub dropped_bytes: u64,
}

pub fn tokenize(coder: &Coder, line: &str) -> Result<Tok, String> {
    let rec = parse_record(line)?;
    let mut ing = Ingest::new(Cursor::new(vec![]), vec![], vec![]).map_err(|e| e.to_string())?;
    ing.add(coder, &rec)?;
    let (t, mask, segments, stats) = ing.finish().map_err(|e| e.to_string())?;
    let t = t.into_inner();
    let tokens = t[UORT_HEADER..].chunks_exact(2).map(|b| u16::from_le_bytes([b[0], b[1]]) as u32).collect();
    let (mut seg_bytes, mut seg_tokens, mut dropped_bytes) = ([0u64; 4], [0u64; 4], 0u64);
    for s in SEGMENTS {
        let c = &stats.per_segment[&s];
        seg_bytes[s as usize] = c.text_bytes;
        seg_tokens[s as usize] = c.tokens;
        dropped_bytes += c.dropped_bytes;
    }
    let response = mask.iter().filter(|&&m| m == 1).count() as u64;
    Ok(Tok { id: rec.id, tokens, segments, mask, response, seg_bytes, seg_tokens, dropped_bytes })
}

/// The unpacked ids of a packed 13-gram.
pub fn unpack(w: &[u64; 3]) -> Vec<u32> {
    (0..GRAM).map(|i| (w[i / 5] >> (ID_BITS * (i % 5) as u32) & 0xfff) as u32).collect()
}

/// The long-span lengths of the contamination census, and the one the gate reads.
pub const SPANS: [usize; 3] = [32, 50, 100];
pub const GATE_SPAN: usize = 50;

/// Polynomial hashing modulo the Mersenne prime 2^61 - 1. A hash only finds candidates;
/// every candidate is compared token for token before it counts.
const P61: u64 = (1 << 61) - 1;
const BASE: u64 = 0x0d6e_8fe1_43a5_9b37 % P61;

fn mulmod(a: u64, b: u64) -> u64 {
    let x = a as u128 * b as u128;
    let r = (x as u64 & P61) + (x >> 61) as u64;
    let r = (r & P61) + (r >> 61);
    if r >= P61 {
        r - P61
    } else {
        r
    }
}

fn powmod(mut b: u64, mut e: usize) -> u64 {
    let mut r = 1;
    while e > 0 {
        if e & 1 == 1 {
            r = mulmod(r, b);
        }
        b = mulmod(b, b);
        e >>= 1;
    }
    r
}

/// Prefix hashes: `buf[i]` hashes `tokens[..i]`.
fn prefix(tokens: &[u32], buf: &mut Vec<u64>) {
    buf.clear();
    buf.push(0);
    let mut h = 0u64;
    for &t in tokens {
        h = mulmod(h, BASE) + t as u64 + 1;
        if h >= P61 {
            h -= P61;
        }
        buf.push(h);
    }
}

/// The hash of `tokens[s..e]` from prefix hashes, with `pow` = BASE^(e - s).
fn window(buf: &[u64], pow: u64, s: usize, e: usize) -> u64 {
    let sub = mulmod(buf[s], pow);
    if buf[e] >= sub {
        buf[e] - sub
    } else {
        buf[e] + P61 - sub
    }
}

/// A token is in the contamination census's text when it is reasoning or answer.
fn in_response_text(seg: u8) -> bool {
    seg == Segment::Reasoning as u8 || seg == Segment::Answer as u8
}

/// The `n`-token spans of DEV-test reasoning+answer text, exact: each distinct span once, with
/// the records that hold it, and a bit for "found in TRAIN".
pub struct SpanSet {
    n: usize,
    pow: u64,
    /// (hash, record, offset) of each distinct span, sorted by hash then tokens.
    uniq: Vec<(u64, u32, u32)>,
    /// (distinct span, record) for every eligible DEV window.
    owners: Vec<(u32, u32)>,
    hit: Vec<AtomicU64>,
    bloom: Vec<u64>,
    shift: u32,
    /// Per distinct span, the number of distinct TRAIN records that hold it (claude-414). Kept
    /// only by a counted set: counting compares every candidate, found already or not.
    mult: Option<Vec<AtomicU32>>,
}

/// The largest TRAIN multiplicity at which a shared span is copied content; from 3 it is
/// stock wording (claude-414).
pub const COPIED_MAX: u32 = 2;
/// The multiplicity buckets of the span classification.
pub const MULT_BUCKETS: [&str; 5] = ["1", "2", "3-9", "10-99", ">=100"];

fn bucket(m: u32) -> usize {
    match m {
        0..=1 => 0,
        2 => 1,
        3..=9 => 2,
        10..=99 => 3,
        _ => 4,
    }
}

/// One shared span of the classification's top list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopSpan {
    /// Distinct DEV-test records that hold the span.
    pub dev_records: u64,
    /// Distinct TRAIN records that hold the span.
    pub train_records: u32,
    pub copied: bool,
    pub tokens: Vec<u32>,
}

impl SpanSet {
    pub fn new(dev: &[Tok], n: usize, bloom_log2: u32) -> Self {
        Self::build(dev, n, bloom_log2, false)
    }

    /// A set that also counts, per distinct span, the distinct TRAIN records holding it.
    pub fn counted(dev: &[Tok], n: usize, bloom_log2: u32) -> Self {
        Self::build(dev, n, bloom_log2, true)
    }

    fn build(dev: &[Tok], n: usize, bloom_log2: u32, counted: bool) -> Self {
        let pow = powmod(BASE, n);
        let mut entries: Vec<(u64, u32, u32)> = vec![];
        let mut buf = vec![];
        for (r, t) in dev.iter().enumerate() {
            prefix(&t.tokens, &mut buf);
            let mut run = 0usize;
            for (i, &seg) in t.segments.iter().enumerate() {
                run = if in_response_text(seg) { run + 1 } else { 0 };
                // The span check: a window counts only when all n of its tokens are text.
                if run >= n {
                    let s = i + 1 - n;
                    entries.push((window(&buf, pow, s, i + 1), r as u32, s as u32));
                }
            }
        }
        let span = |r: u32, s: u32| &dev[r as usize].tokens[s as usize..s as usize + n];
        entries.sort_unstable_by(|a, b| a.0.cmp(&b.0).then_with(|| span(a.1, a.2).cmp(span(b.1, b.2))));
        let (mut uniq, mut owners): (Vec<(u64, u32, u32)>, Vec<(u32, u32)>) = (vec![], vec![]);
        for (h, r, s) in entries {
            let same = uniq.last().is_some_and(|&(uh, ur, us)| uh == h && span(ur, us) == span(r, s));
            if !same {
                uniq.push((h, r, s));
            }
            owners.push((uniq.len() as u32 - 1, r));
        }
        let mut bloom = vec![0u64; 1usize << bloom_log2.saturating_sub(6)];
        let shift = 64 - bloom_log2;
        for &(h, _, _) in &uniq {
            let b = (mix(&[h, 0, 0]) >> shift) as usize;
            bloom[b / 64] |= 1 << (b % 64);
        }
        let hit = (0..uniq.len().div_ceil(64)).map(|_| AtomicU64::new(0)).collect();
        let mult = counted.then(|| (0..uniq.len()).map(|_| AtomicU32::new(0)).collect());
        Self { n, pow, uniq, owners, hit, bloom, shift, mult }
    }

    fn is_hit(&self, i: usize) -> bool {
        self.hit[i / 64].load(Ordering::Relaxed) >> (i % 64) & 1 == 1
    }

    /// Mark every distinct DEV span that equals one of `train`'s n-windows. `train` is one
    /// whole TRAIN record; a counted set adds one to each distinct span the record holds,
    /// however often the record repeats it. Adds commute, so the counts do not depend on which
    /// thread marks which record.
    pub fn mark(&self, dev: &[Tok], train: &[u32], buf: &[u64]) {
        let n = self.n;
        let counting = self.mult.is_some();
        let mut held: Vec<u32> = vec![];
        for e in n..=train.len() {
            let h = window(buf, self.pow, e - n, e);
            let b = (mix(&[h, 0, 0]) >> self.shift) as usize;
            if self.bloom[b / 64] >> (b % 64) & 1 == 0 {
                continue;
            }
            let mut i = self.uniq.partition_point(|u| u.0 < h);
            while self.uniq.get(i).is_some_and(|entry| entry.0 == h) {
                let (_, r, s) = self.uniq[i];
                if (counting || !self.is_hit(i)) && dev[r as usize].tokens[s as usize..s as usize + n] == train[e - n..e] {
                    self.hit[i / 64].fetch_or(1 << (i % 64), Ordering::Relaxed);
                    if counting {
                        held.push(i as u32);
                    }
                    break;
                }
                i += 1;
            }
        }
        if let Some(mult) = &self.mult {
            // One count per TRAIN record: a span repeated inside the record counts once.
            held.sort_unstable();
            held.dedup();
            for &i in &held {
                mult[i as usize].fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Does the TRAIN record `train` hold any of the set's DEV spans (claude-416)? Every
    /// candidate is compared token for token, spans already marked by other records included.
    pub fn holds(&self, dev: &[Tok], train: &[u32], buf: &[u64]) -> bool {
        let n = self.n;
        (n..=train.len()).any(|e| {
            let h = window(buf, self.pow, e - n, e);
            let b = (mix(&[h, 0, 0]) >> self.shift) as usize;
            if self.bloom[b / 64] >> (b % 64) & 1 == 0 {
                return false;
            }
            let i = self.uniq.partition_point(|u| u.0 < h);
            self.uniq[i..].iter().take_while(|u| u.0 == h)
                .any(|&(_, r, s)| dev[r as usize].tokens[s as usize..s as usize + n] == train[e - n..e])
        })
    }

    /// Does the TRAIN record `train` hold a copied span of this counted set: one whose TRAIN
    /// multiplicity, from a finished TRAIN pass, is in `1..=COPIED_MAX` (claude-417)? Compared
    /// token for token like `holds`; false for a set that does not count.
    pub fn holds_copied(&self, dev: &[Tok], train: &[u32], buf: &[u64]) -> bool {
        let Some(mult) = &self.mult else { return false };
        let n = self.n;
        (n..=train.len()).any(|e| {
            let h = window(buf, self.pow, e - n, e);
            let b = (mix(&[h, 0, 0]) >> self.shift) as usize;
            if self.bloom[b / 64] >> (b % 64) & 1 == 0 {
                return false;
            }
            let i = self.uniq.partition_point(|u| u.0 < h);
            (i..self.uniq.len()).take_while(|&j| { let (k, _, _) = self.uniq[j]; k == h }).any(|j| {
                let (_, r, s) = self.uniq[j];
                (1..=COPIED_MAX).contains(&mult[j].load(Ordering::Relaxed))
                    && dev[r as usize].tokens[s as usize..s as usize + n] == train[e - n..e]
            })
        })
    }

    /// The classification of a counted set's shared spans: copied (TRAIN multiplicity at most
    /// `COPIED_MAX`) or stock; a contaminated record is copied-contaminated when any of its
    /// shared spans is copied. With the `k` shared spans held by the most DEV records. None for
    /// a set that does not count.
    pub fn classify(&self, dev: &[Tok], k: usize) -> Option<(Value, Vec<TopSpan>)> {
        let mult = self.mult.as_ref()?;
        let m = |i: usize| mult[i].load(Ordering::Relaxed);
        let (mut span_hist, mut copied_spans, mut stock_spans, mut most) = ([0u64; 5], 0u64, 0u64, 0u32);
        for i in 0..self.uniq.len() {
            let c = m(i);
            if c == 0 {
                continue;
            }
            span_hist[bucket(c)] += 1;
            if c <= COPIED_MAX {
                copied_spans += 1;
            } else {
                stock_spans += 1;
            }
            most = most.max(c);
        }
        // Per span, the distinct DEV records holding it; per record, its least multiplicity
        // over its shared spans (0: it shares none).
        let mut pairs = self.owners.clone();
        pairs.sort_unstable();
        pairs.dedup();
        let (records, mut holders) = (dev.len(), vec![0u64; self.uniq.len()]);
        let mut least = vec![0u32; records];
        for &(u, r) in &pairs {
            holders[u as usize] += 1;
            let c = m(u as usize);
            let slot = &mut least[r as usize];
            if c > 0 && (*slot == 0 || c < *slot) {
                *slot = c;
            }
        }
        let (mut rec_hist, mut contaminated, mut copied) = ([0u64; 5], 0u64, 0u64);
        for &c in &least {
            if c > 0 {
                contaminated += 1;
                rec_hist[bucket(c)] += 1;
                copied += (c <= COPIED_MAX) as u64;
            }
        }
        let mut order: Vec<usize> = (0..self.uniq.len()).filter(|&i| m(i) > 0).collect();
        order.sort_unstable_by(|&a, &b| holders[b].cmp(&holders[a]).then(a.cmp(&b)));
        order.truncate(k);
        let top: Vec<TopSpan> = order
            .iter()
            .map(|&i| {
                let (_, r, s) = self.uniq[i];
                let tokens = dev[r as usize].tokens[s as usize..s as usize + self.n].to_vec();
                TopSpan { dev_records: holders[i], train_records: m(i), copied: m(i) <= COPIED_MAX, tokens }
            })
            .collect();
        let hist = |h: &[u64; 5]| -> serde_json::Map<String, Value> {
            MULT_BUCKETS.iter().zip(h).map(|(b, &c)| (b.to_string(), json!(c))).collect()
        };
        let top_copied = top.iter().filter(|t| t.copied).count() as u64;
        let verdict = if contaminated == 0 {
            "NO-DATA"
        } else if copied * 10 > contaminated {
            "SPAN-DEDUP"
        } else {
            "SPLIT-V1-STANDS"
        };
        let borderline = contaminated > 0 && copied * 100 >= contaminated * 8 && copied * 100 <= contaminated * 12;
        let v = json!({
            "unit": "claude-414", "span": self.n,
            "rule": {
                "multiplicity": "a shared span's TRAIN multiplicity is the number of distinct TRAIN records holding it; a span repeated inside one TRAIN record counts once",
                "span_class": format!("copied when the multiplicity is 1..={COPIED_MAX}, stock when it is at least {}", COPIED_MAX + 1),
                "record_class": "a contaminated dev-test record is copied-contaminated when any of its shared spans is copied, else stock-only",
                "bar": "copied_contaminated x 10 > contaminated: span dedup; else split v1 stands; a share of 8..=12% is borderline",
            },
            "dev_test_records": records,
            "contaminated": contaminated,
            "copied_contaminated": copied,
            "stock_only": contaminated - copied,
            "copied_share": [copied, contaminated],
            "copied_share_per_mille": if contaminated == 0 { 0 } else { copied * 1000 / contaminated },
            "bar": {"verdict": verdict, "borderline": borderline},
            "shared_spans": copied_spans + stock_spans,
            "shared_spans_by_class": {"copied": copied_spans, "stock": stock_spans},
            "most_train_records_of_a_span": most,
            "span_multiplicity_histogram": hist(&span_hist),
            "record_least_multiplicity_histogram": hist(&rec_hist),
            "top": {"k": top.len(), "ranked_by": "distinct dev-test records holding the span, then span order",
                    "copied": top_copied, "stock": top.len() as u64 - top_copied,
                    "dev_records": top.iter().map(|t| t.dev_records).collect::<Vec<_>>(),
                    "train_records": top.iter().map(|t| t.train_records).collect::<Vec<_>>(),
                    "text": "decoded only to the laptop-side --classify-spans file, never committed"},
            "ratios": "every pair is an exact integer [numerator, denominator]",
        });
        Some((v, top))
    }

    /// Per DEV record: does any of its spans occur in TRAIN; and does it have any span.
    pub fn records(&self, records: usize) -> (Vec<bool>, Vec<bool>) {
        let (mut hit, mut any) = (vec![false; records], vec![false; records]);
        for &(u, r) in &self.owners {
            any[r as usize] = true;
            hit[r as usize] |= self.is_hit(u as usize);
        }
        (hit, any)
    }

    pub fn counts(&self) -> (u64, u64, u64) {
        let hits = (0..self.uniq.len()).filter(|&i| self.is_hit(i)).count() as u64;
        (self.owners.len() as u64, self.uniq.len() as u64, hits)
    }
}

/// Pack 13 ids (each below 4,096) exactly into three words.
pub fn pack(ids: &[u32]) -> [u64; 3] {
    assert_eq!(ids.len(), GRAM, "a 13-gram has {GRAM} ids");
    let mut w = [0u64; 3];
    for (i, &id) in ids.iter().enumerate() {
        // A hard check in release too: an id of 4,096 or more would spill into the next slot
        // and make false matches.
        assert!(id < 1 << ID_BITS, "token id {id} does not fit {ID_BITS} bits");
        w[i / 5] |= (id as u64) << (ID_BITS * (i % 5) as u32);
    }
    w
}

fn mix(g: &[u64; 3]) -> u64 {
    let mut z = g[0] ^ g[1].rotate_left(21) ^ g[2].rotate_left(42);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// Every DEV position of one split: its gram, sorted and deduplicated, with the number of
/// positions per gram by the segment of the gram's last token, and a hit bit per gram.
pub struct GramSet {
    uniq: Vec<[u64; 3]>,
    mult: Vec<[u32; 4]>,
    hit: Vec<AtomicU64>,
}

impl GramSet {
    pub fn new(mut v: Vec<([u64; 3], u8)>) -> Self {
        v.sort_unstable();
        let (mut uniq, mut mult): (Vec<[u64; 3]>, Vec<[u32; 4]>) = (vec![], vec![]);
        for (g, s) in v {
            if uniq.last() != Some(&g) {
                uniq.push(g);
                mult.push([0; 4]);
            }
            mult.last_mut().unwrap()[s as usize] += 1;
        }
        let hit = (0..uniq.len().div_ceil(64)).map(|_| AtomicU64::new(0)).collect();
        Self { uniq, mult, hit }
    }

    pub fn mark(&self, g: &[u64; 3]) {
        if let Ok(i) = self.uniq.binary_search(g) {
            self.hit[i / 64].fetch_or(1 << (i % 64), Ordering::Relaxed);
        }
    }

    /// Whether `g` is a DEV gram found in TRAIN.
    pub fn hit_of(&self, g: &[u64; 3]) -> bool {
        self.uniq.binary_search(g).is_ok_and(|i| self.hit[i / 64].load(Ordering::Relaxed) >> (i % 64) & 1 == 1)
    }

    /// The `k` grams found in TRAIN with the most DEV positions whose last token is in
    /// segment `seg`, most first (ties by gram), with those position counts.
    pub fn top_hits(&self, seg: usize, k: usize) -> Vec<(u64, [u64; 3])> {
        let mut v: Vec<(u64, [u64; 3])> = (0..self.uniq.len())
            .filter(|&i| self.hit[i / 64].load(Ordering::Relaxed) >> (i % 64) & 1 == 1 && self.mult[i][seg] > 0)
            .map(|i| (self.mult[i][seg] as u64, self.uniq[i]))
            .collect();
        v.sort_unstable_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        v.truncate(k);
        v
    }

    /// (positions, overlapped positions) per segment of the gram's last token.
    pub fn census(&self) -> ([u64; 4], [u64; 4], u64, u64) {
        let (mut pos, mut over, mut hits) = ([0u64; 4], [0u64; 4], 0u64);
        for (i, m) in self.mult.iter().enumerate() {
            let h = self.hit[i / 64].load(Ordering::Relaxed) >> (i % 64) & 1 == 1;
            hits += h as u64;
            for s in 0..4 {
                pos[s] += m[s] as u64;
                if h {
                    over[s] += m[s] as u64;
                }
            }
        }
        (pos, over, self.uniq.len() as u64, hits)
    }
}

/// Per-split token counts for the target-count reconciliation.
#[derive(Clone, Copy, Default, Debug)]
pub struct TokCount {
    pub records: u64,
    pub tokens: u64,
    pub response: u64,
    pub seg_bytes: [u64; 4],
    pub seg_tokens: [u64; 4],
}
impl TokCount {
    fn add(&mut self, t: &Tok) {
        self.records += 1;
        self.tokens += t.tokens.len() as u64;
        self.response += t.response;
        for s in 0..4 {
            self.seg_bytes[s] += t.seg_bytes[s];
            self.seg_tokens[s] += t.seg_tokens[s];
        }
    }
    fn merge(&mut self, o: &TokCount) {
        self.records += o.records;
        self.tokens += o.tokens;
        self.response += o.response;
        for s in 0..4 {
            self.seg_bytes[s] += o.seg_bytes[s];
            self.seg_tokens[s] += o.seg_tokens[s];
        }
    }
    fn json(&self) -> Value {
        let bytes: u64 = self.seg_bytes.iter().sum();
        let seg: serde_json::Map<String, Value> = SEGMENTS.iter().map(|&s| {
            let i = s as usize;
            (s.name().to_string(), json!({"text_bytes": self.seg_bytes[i], "tokens": self.seg_tokens[i]}))
        }).collect();
        let next = self.tokens - self.records;
        json!({"records": self.records, "tokens": self.tokens, "response_tokens": self.response,
               "next_token_targets": next, "text_bytes": bytes, "per_segment": seg,
               "bytes_per_token": [bytes, self.tokens], "bytes_per_response_token": [bytes, self.response],
               "bytes_per_next_token_target": [bytes, next]})
    }
}

fn each_gram(t: &Tok, mut f: impl FnMut([u64; 3], u8)) {
    for e in GRAM..=t.tokens.len() {
        f(pack(&t.tokens[e - GRAM..e]), t.segments[e - 1]);
    }
}

/// The census's text outputs, kept out of the summary: they hold corpus text or record ids.
pub struct Extras {
    /// The most frequent DEV-test prompt 13-grams found in TRAIN: (positions, ids).
    pub templates: Vec<(u64, Vec<u32>)>,
    /// Per span length in `SPANS`, the ids of the contaminated DEV-test records.
    pub contaminated: Vec<Vec<String>>,
    /// The `GATE_SPAN` classification (claude-414), when asked for: its summary and top spans.
    pub classify: Option<(Value, Vec<TopSpan>)>,
    /// With `AuditOptions::find_drop`, the sorted ids of the kept TRAIN records (outside the
    /// applied drop list) that hold a DEV-test span of that length (claude-416).
    pub drop_found: Vec<String>,
    /// With `AuditOptions::find_copied`, the sorted ids of the kept TRAIN records (outside the
    /// applied drop list) that hold a copied `GATE_SPAN` span (claude-417).
    pub copied_found: Vec<String>,
    /// The sorted ids of the DEV-test records contaminated at `GATE_SPAN` in this run (claude-417).
    pub residue: Vec<String>,
}

/// The span length whose TRAIN holders the span dedup drops (claude-414's decision item 2).
pub const DROP_SPAN: usize = 100;

/// What an audit does beyond the census (claude-414, claude-416).
#[derive(Default)]
pub struct AuditOptions {
    /// Classify the `GATE_SPAN` shared spans, decoding the `top` held by the most DEV records.
    pub classify_top: Option<usize>,
    /// Kept TRAIN record ids skipped in the TRAIN pass (split v2 = split v1 minus these).
    pub drop: HashSet<String>,
    /// List the TRAIN records that hold a DEV-test span of this length (one of `SPANS`).
    pub find_drop: Option<usize>,
    /// List the kept TRAIN records that hold a copied `GATE_SPAN` span (TRAIN multiplicity at
    /// most `COPIED_MAX`), by a second TRAIN pass after the counting one (claude-417).
    pub find_copied: bool,
    /// DEV-test record ids left out of DEV-test entirely: spans, grams and every count (claude-417).
    pub drop_dev_test: HashSet<String>,
}

/// The 13-gram census of a plan; see `audit_full`.
pub fn audit(plan: &Plan, coder: &Coder, threads: usize, bloom_log2: u32) -> Result<Value, String> {
    audit_full(plan, coder, threads, bloom_log2).map(|(v, _)| v)
}

/// The census of a plan: DEV grams and DEV-test spans collected, TRAIN streamed against them.
pub fn audit_full(plan: &Plan, coder: &Coder, threads: usize, bloom_log2: u32) -> Result<(Value, Extras), String> {
    audit_with(plan, coder, threads, bloom_log2, &AuditOptions::default())
}

/// `audit_full`, plus the classification of the `GATE_SPAN`-token shared spans by TRAIN
/// multiplicity, with the `top` spans held by the most DEV-test records (claude-414).
pub fn audit_classify(plan: &Plan, coder: &Coder, threads: usize, bloom_log2: u32, top: usize) -> Result<(Value, Extras), String> {
    audit_with(plan, coder, threads, bloom_log2, &AuditOptions { classify_top: Some(top), ..AuditOptions::default() })
}

/// The census under `opts`: a drop list applied to TRAIN, drop candidates listed, or both.
/// A drop id that is not a kept TRAIN record is refused.
pub fn audit_opts(plan: &Plan, coder: &Coder, threads: usize, bloom_log2: u32, opts: &AuditOptions) -> Result<(Value, Extras), String> {
    audit_with(plan, coder, threads, bloom_log2, opts)
}

fn audit_with(plan: &Plan, coder: &Coder, threads: usize, bloom_log2: u32, opts: &AuditOptions) -> Result<(Value, Extras), String> {
    if !(6..=40).contains(&bloom_log2) {
        return Err(format!("bloom_log2 {bloom_log2} is outside 6..=40"));
    }
    if opts.find_drop.is_some_and(|n| !SPANS.contains(&n)) {
        return Err(format!("find_drop must be one of {SPANS:?}"));
    }
    if opts.find_copied && !opts.drop.is_empty() {
        return Err("find_copied counts multiplicity over split v1, so it takes no TRAIN drop list".into());
    }
    let classify_top = opts.classify_top;
    let paths: Vec<PathBuf> = plan.shards.iter().map(|(p, _)| p.clone()).collect();
    let fate_of = |p: &Path| &plan.fate[paths.iter().position(|q| q == p).unwrap()];
    // Pass 1: DEV grams and DEV counts; DEV-test records kept whole for the span census.
    let dev = par_shards(&paths, threads, |p| {
        let fate = fate_of(p);
        let (mut i, mut grams, mut cnt) = (0usize, [vec![], vec![]], [TokCount::default(); 2]);
        let mut toks = vec![];
        let mut gone = TokCount::default();
        read_shard(p, &mut |line| {
            let f = fate[i];
            i += 1;
            let k = match f {
                Fate::Keep(Split::DevTune) => 0,
                Fate::Keep(Split::DevTest) => 1,
                _ => return Ok(()),
            };
            let t = tokenize(coder, line)?;
            if k == 1 && opts.drop_dev_test.contains(&t.id) {
                gone.add(&t);
                return Ok(());
            }
            cnt[k].add(&t);
            each_gram(&t, |g, s| grams[k].push((g, s)));
            if k == 1 {
                toks.push(t);
            }
            Ok(())
        })?;
        Ok((grams, cnt, toks, gone))
    })?;
    let mut counts = [TokCount::default(); 3];
    let (mut tune, mut test, mut dev_test, mut dev_gone) = (vec![], vec![], vec![], TokCount::default());
    for ([gt, gs], [ct, cs], tk, g) in dev {
        tune.extend(gt);
        test.extend(gs);
        counts[1].merge(&ct);
        counts[2].merge(&cs);
        dev_test.extend(tk);
        dev_gone.merge(&g);
    }
    if dev_gone.records != opts.drop_dev_test.len() as u64 {
        return Err(format!("{} of {} DEV-test drop ids are kept DEV-test records", dev_gone.records, opts.drop_dev_test.len()));
    }
    let counted = classify_top.is_some() || opts.find_copied;
    let spans: Vec<SpanSet> = SPANS
        .iter()
        .map(|&n| match counted {
            true if n == GATE_SPAN => SpanSet::counted(&dev_test, n, bloom_log2),
            _ => SpanSet::new(&dev_test, n, bloom_log2),
        })
        .collect();
    let sets = [GramSet::new(tune), GramSet::new(test)];
    let mut bloom = vec![0u64; 1usize << bloom_log2.saturating_sub(6)];
    let shift = 64 - bloom_log2;
    for set in &sets {
        for g in &set.uniq {
            let b = (mix(g) >> shift) as usize;
            bloom[b / 64] |= 1 << (b % 64);
        }
    }
    let finder = opts.find_drop.and_then(|n| spans.iter().find(|s| s.n == n));
    // Pass 2: TRAIN grams against the DEV sets; a dropped record is counted apart and skipped.
    let train = par_shards(&paths, threads, |p| {
        let fate = fate_of(p);
        let (mut i, mut cnt, mut buf) = (0usize, TokCount::default(), vec![]);
        let (mut dropped, mut found) = (TokCount::default(), vec![]);
        read_shard(p, &mut |line| {
            let f = fate[i];
            i += 1;
            if f != Fate::Keep(Split::Train) {
                return Ok(());
            }
            let t = tokenize(coder, line)?;
            if opts.drop.contains(&t.id) {
                dropped.add(&t);
                return Ok(());
            }
            cnt.add(&t);
            prefix(&t.tokens, &mut buf);
            if let Some(s) = finder {
                if s.holds(&dev_test, &t.tokens, &buf) {
                    found.push(t.id.clone());
                }
            }
            each_gram(&t, |g, _| {
                let b = (mix(&g) >> shift) as usize;
                if bloom[b / 64] >> (b % 64) & 1 == 1 {
                    sets[0].mark(&g);
                    sets[1].mark(&g);
                }
            });
            for s in &spans {
                s.mark(&dev_test, &t.tokens, &buf);
            }
            Ok(())
        })?;
        Ok((cnt, dropped, found))
    })?;
    let (mut dropped, mut drop_found) = (TokCount::default(), vec![]);
    for (c, d, f) in train {
        counts[0].merge(&c);
        dropped.merge(&d);
        drop_found.extend(f);
    }
    drop_found.sort_unstable();
    if dropped.records != opts.drop.len() as u64 {
        return Err(format!("{} of {} drop ids are kept TRAIN records", dropped.records, opts.drop.len()));
    }
    // Pass 3 (claude-417): multiplicities are final only after pass 2, so the holders of copied
    // spans are found by a second pass over the same kept TRAIN records.
    let mut copied_found = vec![];
    if opts.find_copied {
        let s = spans.iter().find(|s| s.n == GATE_SPAN).unwrap();
        let found = par_shards(&paths, threads, |p| {
            let fate = fate_of(p);
            let (mut i, mut buf, mut found) = (0usize, vec![], vec![]);
            read_shard(p, &mut |line| {
                let f = fate[i];
                i += 1;
                if f != Fate::Keep(Split::Train) {
                    return Ok(());
                }
                let t = tokenize(coder, line)?;
                if opts.drop.contains(&t.id) {
                    return Ok(());
                }
                prefix(&t.tokens, &mut buf);
                if s.holds_copied(&dev_test, &t.tokens, &buf) {
                    found.push(t.id);
                }
                Ok(())
            })?;
            Ok(found)
        })?;
        copied_found = found.into_iter().flatten().collect();
        copied_found.sort_unstable();
    }
    let mut total = TokCount::default();
    let mut splits = serde_json::Map::new();
    for s in SPLITS {
        total.merge(&counts[s.ix()]);
        splits.insert(s.name().into(), counts[s.ix()].json());
    }
    let mut overlap = serde_json::Map::new();
    let mut gate = json!(null);
    for (set, s) in sets.iter().zip([Split::DevTune, Split::DevTest]) {
        let (pos, over, uniq, hits) = set.census();
        let (p, o): (u64, u64) = (pos.iter().sum(), over.iter().sum());
        let seg: serde_json::Map<String, Value> = SEGMENTS.iter()
            .map(|&g| (g.name().to_string(), json!([over[g as usize], pos[g as usize]]))).collect();
        overlap.insert(s.name().into(), json!({"positions": p, "overlapped": o, "overlap": [o, p],
            "overlap_by_last_token_segment": seg, "unique_grams": uniq, "unique_grams_in_train": hits}));
        if s == Split::DevTest {
            let verdict = if p == 0 { "NO-DATA" } else if o * 100 <= p { "PASS" } else { "FAIL" };
            gate = json!({"rule": "dev-test overlapped x 100 <= dev-test positions (the first bar; a diagnostic since claude-411's regroup ruling)",
                          "overlap": [o, p], "verdict": verdict});
        }
    }
    // Long-span contamination of DEV-test records.
    let records = dev_test.len() as u64;
    let (mut contamination, mut contaminated, mut gate_span) = (serde_json::Map::new(), vec![], json!(null));
    for s in &spans {
        let (hit, any) = s.records(dev_test.len());
        let c = hit.iter().filter(|&&h| h).count() as u64;
        let (windows, uniq, uhits) = s.counts();
        contamination.insert(s.n.to_string(), json!({"contaminated": c, "share": [c, records],
            "records_with_a_span": any.iter().filter(|&&a| a).count(), "windows": windows,
            "unique_spans": uniq, "unique_spans_in_train": uhits}));
        if s.n == GATE_SPAN {
            let verdict = if records == 0 { "NO-DATA" } else if c * 100 <= records { "PASS" } else { "FAIL" };
            gate_span = json!({"rule": "dev-test records holding a >= 50-token reasoning+answer span found in TRAIN, x 100 <= dev-test records (at most 1%)",
                               "contaminated": [c, records], "verdict": verdict});
        }
        contaminated.push(dev_test.iter().zip(&hit).filter(|(_, &h)| h).map(|(t, _)| t.id.clone()).collect());
    }
    // Per record: answer 13-grams in TRAIN; per target: is its preceding 13-gram in TRAIN.
    let ans = Segment::Answer as u8;
    let (mut half, mut with_answer) = (0u64, 0u64);
    let (mut targets, mut ctx, mut novel, mut r_targets, mut r_ctx, mut r_novel) = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
    for t in &dev_test {
        let (mut ag, mut ah) = (0u64, 0u64);
        for e in 1..t.tokens.len() {
            let resp = t.mask[e] == 1;
            targets += 1;
            r_targets += resp as u64;
            if e < GRAM {
                continue;
            }
            let found = sets[1].hit_of(&pack(&t.tokens[e - GRAM..e]));
            ctx += 1;
            r_ctx += resp as u64;
            novel += !found as u64;
            r_novel += (resp && !found) as u64;
        }
        for e in GRAM..=t.tokens.len() {
            if t.segments[e - 1] == ans {
                ag += 1;
                ah += sets[1].hit_of(&pack(&t.tokens[e - GRAM..e])) as u64;
            }
        }
        with_answer += (ag > 0) as u64;
        half += (ag > 0 && ah * 2 >= ag) as u64;
    }
    let templates = sets[1].top_hits(Segment::Prompt as usize, 30).into_iter().map(|(c, g)| (c, unpack(&g))).collect();
    let mut v = json!({
        "unit": "claude-411", "gram": GRAM,
        "gram_compare": "exact: 13 ids x 12 bits packed in three u64 words, no hash; grams never cross a record",
        "bloom_log2": bloom_log2, "splits": splits, "total": total.json(), "overlap": overlap,
        "gate": gate_span,
        "overlap_13gram_diagnostic": gate,
        "contamination": {
            "rule": "a dev-test record is contaminated at n when n consecutive tokens of it, all reasoning or answer, equal n consecutive tokens of one TRAIN record; candidates by a mod 2^61-1 rolling hash, every candidate compared token for token",
            "dev_test_records": records, "by_span": contamination,
        },
        "answer_13gram_records": {
            "rule": "dev-test records whose answer 13-grams (last token in an answer) are at least half in TRAIN",
            "records": [half, records], "records_with_an_answer_gram": with_answer,
        },
        "novel_targets": {
            "rule": "dev-test next-token targets whose 13 preceding tokens (one record) form a gram not in TRAIN; targets with under 13 preceding tokens are counted apart",
            "all": {"targets": targets, "with_13_context": ctx, "novel": novel, "short_context": targets - ctx},
            "response_mask": {"targets": r_targets, "with_13_context": r_ctx, "novel": r_novel, "short_context": r_targets - r_ctx},
        },
        "ratios": "every pair is an exact integer [numerator, denominator]",
    });
    if !opts.drop.is_empty() || opts.find_drop.is_some() || opts.find_copied {
        let kept_tokens = counts[0].tokens;
        v["train_drop"] = json!({
            "rule": "split v2 = split v1 with the listed kept TRAIN records skipped; `splits.train` counts what remains (claude-416)",
            "applied": dropped.json(),
            "applied_token_share": [dropped.tokens, dropped.tokens + kept_tokens],
            "found": opts.find_drop.map(|n| json!({
                "rule": "kept TRAIN records outside the applied list that hold a DEV-test span of `span` reasoning+answer tokens, compared token for token",
                "span": n, "records": drop_found.len(),
            })),
        });
        if opts.find_copied {
            v["train_drop"]["copied"] = json!({
                "rule": "kept TRAIN records outside the applied list that hold a DEV-test span of `span` reasoning+answer tokens whose TRAIN multiplicity (distinct kept TRAIN records, from a full counting pass) is at most `max_multiplicity`; found by a second TRAIN pass, compared token for token (claude-417)",
                "span": GATE_SPAN, "max_multiplicity": COPIED_MAX, "records": copied_found.len(),
            });
        }
    }
    if !opts.drop_dev_test.is_empty() {
        v["dev_test_drop"] = json!({
            "rule": "split v3 DEV-test = split v1 DEV-test with the listed records left out of every DEV-test span, gram and count; `splits.dev-test` counts what remains (claude-417)",
            "applied": dev_gone.json(),
            "applied_token_share": [dev_gone.tokens, dev_gone.tokens + counts[2].tokens],
        });
    }
    let gate_ix = SPANS.iter().position(|&n| n == GATE_SPAN).unwrap();
    let mut residue: Vec<String> = Vec::clone(&contaminated[gate_ix]);
    residue.sort_unstable();
    let classify = classify_top.and_then(|k| spans.iter().find(|s| s.n == GATE_SPAN)?.classify(&dev_test, k));
    Ok((v, Extras { templates, contaminated, classify, drop_found, copied_found, residue }))
}

pub mod streams;

#[cfg(test)]
mod tests;
