//! Host instrument for the W2 code-utilization diagnostic.
//!
//! Implements the registered sheet
//! `docs/harness/prep/m1-w2-code-utilization-2026-10-25.md` (FIT, round 2) on the
//! stored W2 capture. Exact integers only: every count is a `u64`, every
//! comparison is an integer comparison, and every ratio is reported as a
//! numerator and a denominator. It reads `uor-r4-discrete` and never edits it.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uor_r4_discrete::w2_capacity as cap;

pub const SCHEMA: &str = "uor-r4.w2-code-utilization/1";
pub const CAPTURE_SCHEMA: &str = "uor-r4.w2-capture/1";
pub const DEV_ROWS: usize = 14_287;
pub const TRAIN_ROWS: usize = 1_049_856;
pub const VOCAB: usize = 4_096;
pub const CELLS: usize = 256;
pub const SLOTS: usize = 16;
pub const ABSENT: u16 = u16::MAX;
/// The M1 T2 bar and W2's M = max(U, G).
pub const T2_BAR: u64 = 6_300;
/// W2's F² = max(576, 4(b+c)) over its co-primary pairs: 4 · (3,729 + 843).
pub const H_LAW_F2: u128 = 18_288;
/// QC's coverage guard: 10% of the DEV panel with a seen joint key.
pub const QC_MIN_JOINT_ROWS: u64 = 1_429;
pub const SUBSAMPLES: usize = 5;

/// Pins, copied from `docs/labs/m1-w2-capacity-capture/README.md`.
pub const TRAIN_STREAM_SHA256: &str =
    "70a587670edb2f5c26f8c2ba299c6bed58c437baf5b0ccc469ed8f316dc69c57";
pub const QUERY_SIDECAR_SHA256: &str =
    "6d5d4f5dd8693b44aa69989a371b592b1361d82d328c700c574ab00228a2d7fe";
pub const CAPTURE_SHA256: &str = "d22a4eb20643fb8b72871d40e44478a5f1892a5f3322fcd56cfbbbe3a2bb8eb1";
pub const RECORD_SHA256: &str = "69717971a6f4e6f4a6dede6ad5eee630c4abdbda13e4c8dd434ef126a74c1abc";

/// The W2 numbers precedence 0 must reproduce (the sheet's §5, row 0).
pub const W2_T: u64 = 3_414;
pub const W2_G: u64 = 6_300;
pub const W2_U: u64 = 1_226;
pub const W2_W: u64 = 149;
pub const W2_A_W: u64 = 1_920;
pub const W2_RESTRICTED_TABLE: u64 = 1_138;
pub const W2_RESTRICTED_BIGRAM: u64 = 1_482;
pub const W2_COVERAGE: u64 = 14_175;
pub const W2_CODES_GE_2: u64 = 41_806;
pub const W2_ADDRESS_ONLY: u64 = 1_675;
pub const W2_INTERIOR_ONLY: u64 = 1_628;
pub const W2_SUM_K: u64 = 114_296;

/// Every mismatch found, so one run names them all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fault(pub Vec<String>);

impl std::fmt::Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "INSTRUMENT_FAULT: {}", self.0.join("; "))
    }
}

pub type Res<T> = Result<T, Fault>;

fn fault<T>(message: impl Into<String>) -> Res<T> {
    Err(Fault(vec![message.into()]))
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

// ---------------------------------------------------------------- inputs

#[derive(Deserialize)]
struct CaptureFile {
    schema: String,
    dev_rows: Vec<CaptureRow>,
    token_cells: Vec<u16>,
}

#[derive(Deserialize)]
struct CaptureRow {
    code: u16,
    query_token: u32,
    target_token: u32,
    admitted_cell_slots: Vec<u16>,
    top8: Vec<u32>,
    top1_correct: u64,
    top8_correct: u64,
    admitted_cells: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dev {
    pub code: u16,
    pub query: u16,
    pub target: u16,
    pub slots: [u16; SLOTS],
    pub k: usize,
    pub writer_top1: u16,
    pub writer_hit1: bool,
    pub writer_hit8: bool,
}

impl Dev {
    pub fn admits_cell(&self, cell: u16) -> bool {
        self.slots.iter().any(|&s| s != ABSENT && s == cell)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Train {
    pub code: u16,
    pub query: u16,
    pub target: u16,
}

#[derive(Clone, Debug, Serialize)]
pub struct FilePin {
    pub name: String,
    pub sha256: String,
    pub bytes: u64,
}

pub struct Inputs {
    pub dev: Vec<Dev>,
    pub train: Vec<Train>,
    pub cells: Vec<u16>,
    pub pins: Vec<FilePin>,
}

fn read_pinned(dir: &Path, name: &str, sha: &str, len: Option<usize>) -> Res<(Vec<u8>, FilePin)> {
    let path = dir.join(name);
    let bytes = fs::read(&path).map_err(|e| Fault(vec![format!("{}: {e}", path.display())]))?;
    let got = sha256_hex(&bytes);
    let mut problems = Vec::new();
    if got != sha {
        problems.push(format!("{name}: sha256 {got} != pin {sha}"));
    }
    if let Some(want) = len {
        if bytes.len() != want {
            problems.push(format!("{name}: {} bytes != {want}", bytes.len()));
        }
    }
    if !problems.is_empty() {
        return Err(Fault(problems));
    }
    let pin = FilePin {
        name: name.to_string(),
        sha256: got,
        bytes: bytes.len() as u64,
    };
    Ok((bytes, pin))
}

fn word(bytes: &[u8], i: usize) -> u16 {
    u16::from_le_bytes([bytes[2 * i], bytes[2 * i + 1]])
}

/// Authenticate and load the capture (sheet §6 checks 1 and 3) and the W2 record.
pub fn load(capture_dir: &Path, record_path: &Path) -> Res<(Inputs, Value)> {
    let (stream, stream_pin) = read_pinned(
        capture_dir,
        "train-stream.bin",
        TRAIN_STREAM_SHA256,
        Some(4 * TRAIN_ROWS),
    )?;
    let (sidecar, sidecar_pin) = read_pinned(
        capture_dir,
        "query-sidecar.bin",
        QUERY_SIDECAR_SHA256,
        Some(2 * TRAIN_ROWS),
    )?;
    let (capture_bytes, capture_pin) =
        read_pinned(capture_dir, "capture.json", CAPTURE_SHA256, None)?;
    let record_dir = record_path
        .parent()
        .ok_or_else(|| Fault(vec!["record path has no parent".into()]))?;
    let record_name = record_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Fault(vec!["record path has no file name".into()]))?;
    let (record_bytes, record_pin) = read_pinned(record_dir, record_name, RECORD_SHA256, None)?;

    let capture: CaptureFile = serde_json::from_slice(&capture_bytes)
        .map_err(|e| Fault(vec![format!("capture.json: {e}")]))?;
    let record: Value = serde_json::from_slice(&record_bytes)
        .map_err(|e| Fault(vec![format!("record.json: {e}")]))?;

    let mut problems = Vec::new();
    if capture.schema != CAPTURE_SCHEMA {
        problems.push(format!("capture schema {}", capture.schema));
    }
    if capture.dev_rows.len() != DEV_ROWS {
        problems.push(format!("{} DEV rows != {DEV_ROWS}", capture.dev_rows.len()));
    }
    if capture.token_cells.len() != VOCAB {
        problems.push(format!(
            "{} token cells != {VOCAB}",
            capture.token_cells.len()
        ));
    }
    let bad_cells = capture
        .token_cells
        .iter()
        .filter(|&&c| usize::from(c) >= CELLS)
        .count();
    if bad_cells != 0 {
        problems.push(format!("{bad_cells} token cells >= {CELLS}"));
    }

    let mut dev = Vec::with_capacity(capture.dev_rows.len());
    for (i, row) in capture.dev_rows.iter().enumerate() {
        let in_vocab = |t: u32| (t as usize) < VOCAB;
        if !in_vocab(row.query_token) || !in_vocab(row.target_token) {
            problems.push(format!("DEV row {i}: token >= {VOCAB}"));
            continue;
        }
        if row.admitted_cell_slots.len() != SLOTS || row.top8.len() != 8 {
            problems.push(format!("DEV row {i}: slot or top-8 width"));
            continue;
        }
        let mut slots = [ABSENT; SLOTS];
        slots.copy_from_slice(&row.admitted_cell_slots);
        let k = slots.iter().filter(|&&s| s != ABSENT).count();
        if slots
            .iter()
            .any(|&s| s != ABSENT && usize::from(s) >= CELLS)
        {
            problems.push(format!("DEV row {i}: admitted cell >= {CELLS}"));
        }
        let mut present: Vec<u16> = slots.iter().copied().filter(|&s| s != ABSENT).collect();
        present.sort_unstable();
        present.dedup();
        if present.len() != k {
            problems.push(format!("DEV row {i}: admitted cells repeat"));
        }
        if k as u64 != row.admitted_cells {
            problems.push(format!("DEV row {i}: {k} slots != admitted_cells"));
        }
        if row.top1_correct > 1 || row.top8_correct > 1 {
            problems.push(format!("DEV row {i}: hit flag not 0 or 1"));
        }
        let writer_top1 = row.top8[0];
        if !in_vocab(writer_top1) {
            problems.push(format!("DEV row {i}: writer top-1 >= {VOCAB}"));
            continue;
        }
        let writer_hit1 = row.top1_correct == 1;
        if writer_hit1 != (writer_top1 == row.target_token) {
            problems.push(format!("DEV row {i}: top1_correct disagrees with top8[0]"));
        }
        dev.push(Dev {
            code: row.code,
            query: row.query_token as u16,
            target: row.target_token as u16,
            slots,
            k,
            writer_top1: writer_top1 as u16,
            writer_hit1,
            writer_hit8: row.top8_correct == 1,
        });
    }

    let train: Vec<Train> = (0..TRAIN_ROWS)
        .map(|i| Train {
            code: word(&stream, 2 * i),
            query: word(&sidecar, i),
            target: word(&stream, 2 * i + 1),
        })
        .collect();
    let bad_train = train
        .iter()
        .filter(|r| usize::from(r.query) >= VOCAB || usize::from(r.target) >= VOCAB)
        .count();
    if bad_train != 0 {
        problems.push(format!("{bad_train} TRAIN rows with a token >= {VOCAB}"));
    }
    if !problems.is_empty() {
        return Err(Fault(problems));
    }
    Ok((
        Inputs {
            dev,
            train,
            cells: capture.token_cells,
            pins: vec![stream_pin, sidecar_pin, capture_pin, record_pin],
        },
        record,
    ))
}

// ---------------------------------------------------------------- count tables

/// One (token, count) entry of a key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub token: u16,
    pub count: u64,
}

/// The sheet's tie rule: higher count wins, then the lower id.
pub fn better(a: Entry, b: Entry) -> bool {
    a.count > b.count || (a.count == b.count && a.token < b.token)
}

#[derive(Clone, Debug)]
pub struct KeyRow {
    /// Sorted by token.
    pub entries: Vec<Entry>,
    pub total: u64,
    pub top: Entry,
    pub second: Option<Entry>,
}

/// Sparse `count[key][token]`, deterministic.
#[derive(Clone, Debug, Default)]
pub struct Table {
    pub keys: BTreeMap<u32, KeyRow>,
    pub total: u64,
}

impl Table {
    pub fn build(pairs: impl Iterator<Item = [u32; 2]>) -> Table {
        let mut v: Vec<[u32; 2]> = pairs.collect();
        v.sort_unstable();
        let mut keys = BTreeMap::new();
        let total = v.len() as u64;
        let mut i = 0;
        while i < v.len() {
            let key = v[i][0];
            let mut entries: Vec<Entry> = Vec::new();
            while i < v.len() && v[i][0] == key {
                let token = v[i][1] as u16;
                match entries.last_mut() {
                    Some(e) if e.token == token => e.count += 1,
                    _ => entries.push(Entry { token, count: 1 }),
                }
                i += 1;
            }
            let row_total = entries.iter().map(|e| e.count).sum();
            let mut top = entries[0];
            let mut second: Option<Entry> = None;
            for &e in &entries[1..] {
                if better(e, top) {
                    second = Some(top);
                    top = e;
                } else if second.map_or(true, |s| better(e, s)) {
                    second = Some(e);
                }
            }
            keys.insert(
                key,
                KeyRow {
                    entries,
                    total: row_total,
                    top,
                    second,
                },
            );
        }
        Table { keys, total }
    }

    pub fn key_total(&self, key: u32) -> u64 {
        self.keys.get(&key).map_or(0, |r| r.total)
    }

    pub fn argmax(&self, key: u32) -> Option<u16> {
        self.keys.get(&key).map(|r| r.top.token)
    }

    pub fn count(&self, key: u32, token: u16) -> u64 {
        self.keys.get(&key).map_or(0, |r| {
            r.entries
                .binary_search_by_key(&token, |e| e.token)
                .map_or(0, |i| r.entries[i].count)
        })
    }

    /// Leave-one-out top-1 for a row with this key and target: `None` when the
    /// key has no other row (the caller falls back), else whether `y` still wins.
    pub fn loo(&self, key: u32, y: u16) -> Option<bool> {
        let row = self.keys.get(&key)?;
        if row.total <= 1 {
            return None;
        }
        let own = Entry {
            token: y,
            count: self.count(key, y).saturating_sub(1),
        };
        let other = if row.top.token != y {
            Some(row.top)
        } else {
            row.second
        };
        Some(match other {
            None => own.count > 0,
            Some(o) => better(own, o),
        })
    }

    /// Σ over keys of the largest count: the oracle (resubstitution) top-1.
    pub fn oracle(&self) -> u64 {
        self.keys.values().map(|r| r.top.count).sum()
    }

    pub fn single_row_keys(&self) -> u64 {
        self.keys.values().filter(|r| r.total == 1).count() as u64
    }

    /// Collision agreement: Σ_κ Σ_t n(n−1) over Σ_κ n_κ(n_κ−1).
    pub fn collision(&self) -> Ratio {
        let mut num = 0u128;
        let mut den = 0u128;
        for r in self.keys.values() {
            for e in &r.entries {
                num += u128::from(e.count) * u128::from(e.count.saturating_sub(1));
            }
            den += u128::from(r.total) * u128::from(r.total.saturating_sub(1));
        }
        Ratio { num, den }
    }
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub struct Ratio {
    pub num: u128,
    pub den: u128,
}

// ---------------------------------------------------------------- floors and bands

/// Paired discordance of arm X against arm Y: b = Y hits and X misses, c = X hits
/// and Y misses.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub struct Pair {
    pub b: u64,
    pub c: u64,
}

impl Pair {
    pub fn of(x: &[bool], y: &[bool]) -> Pair {
        let mut p = Pair { b: 0, c: 0 };
        for (&xh, &yh) in x.iter().zip(y) {
            if yh && !xh {
                p.b += 1;
            }
            if xh && !yh {
                p.c += 1;
            }
        }
        p
    }
    pub fn reversed(self) -> Pair {
        Pair {
            b: self.c,
            c: self.b,
        }
    }
}

/// W2's paired floor in squares: X beats Y iff c > b and (c − b)² > max(576, 4(b + c)).
pub fn beats(p: Pair) -> bool {
    if p.c <= p.b {
        return false;
    }
    let d = u128::from(p.c - p.b);
    let floor = std::cmp::max(576u128, 4 * u128::from(p.b + p.c));
    d * d > floor
}

/// Precedence 1: b + c < 100 and the exact two-sided McNemar test does not reject.
pub fn underpowered(p: Pair) -> bool {
    p.b + p.c < 100 && !cap::exact_mcnemar_rejects_005(cap::Discordance { b: p.b, c: p.c })
}

/// QA's band. `tw` is A_T against A_W; `tu` is A_T against A_U, decisive only when
/// A_T beats A_W.
pub fn qa_band(tw: Pair, tu: Pair) -> &'static str {
    if underpowered(tw) {
        return "UNDERPOWERED";
    }
    if beats(tw) {
        if underpowered(tu) {
            "A1u"
        } else if beats(tu) {
            "A1a"
        } else {
            "A1b"
        }
    } else if beats(tw.reversed()) {
        "A3"
    } else {
        "A2"
    }
}

/// QB: d_T counts only when T(N_0) beats T(N_1); B1 iff T_proj crosses W2's H_law line.
pub fn qb_band(t0: u64, t1_pair: Pair, f2: u128) -> QbBand {
    let d_t = if beats(t1_pair) {
        t1_pair.c - t1_pair.b
    } else {
        0
    };
    let t_proj = t0 + 2 * d_t;
    let band = if t_proj > T2_BAR && {
        let e = u128::from(t_proj - T2_BAR);
        e * e > f2
    } {
        "B1"
    } else {
        "B2"
    };
    QbBand { d_t, t_proj, band }
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub struct QbBand {
    pub d_t: u64,
    pub t_proj: u64,
    pub band: &'static str,
}

/// QC's band: J against G, with the joint-key coverage guard.
pub fn qc_band(joint_seen_rows: u64, jg: Pair) -> &'static str {
    if joint_seen_rows < QC_MIN_JOINT_ROWS || underpowered(jg) {
        "UNDERPOWERED"
    } else if beats(jg) {
        "C1"
    } else if beats(jg.reversed()) {
        "C3"
    } else {
        "C2"
    }
}

// ---------------------------------------------------------------- helpers

/// Rank all cells by (score desc, unigram cell mass desc, cell id asc).
pub fn rank_cells(score: &[u64; CELLS], mass: &[u64; CELLS]) -> Vec<u16> {
    let mut idx: Vec<u16> = (0..CELLS as u16).collect();
    idx.sort_by(|&a, &b| {
        let (ua, ub) = (usize::from(a), usize::from(b));
        score[ub]
            .cmp(&score[ua])
            .then(mass[ub].cmp(&mass[ua]))
            .then(a.cmp(&b))
    });
    idx
}

/// Rank cells for one key of a TRAIN table: its target counts summed per cell
/// (A_T for a code table, A_G for the bigram), ties by unigram cell mass, then id.
/// An unseen key ranks by mass alone.
pub fn ranked_for(table: &Table, key: u32, cells: &[u16], mass: &[u64; CELLS]) -> Vec<u16> {
    let mut s = [0u64; CELLS];
    if let Some(r) = table.keys.get(&key) {
        for e in &r.entries {
            s[usize::from(cells[usize::from(e.token)])] += e.count;
        }
    }
    rank_cells(&s, mass)
}

/// Whether `cell` is among the first `k` ranked cells (k = the row's admitted count).
pub fn in_top(ranked: &[u16], k: usize, cell: u16) -> bool {
    ranked.iter().take(k).any(|&c| c == cell)
}

/// J's prediction: the joint (code, query) key when it was seen at least `min_count`
/// times, else G's prediction for the query, else the unigram's.
pub fn joint_prediction(
    joint: &Table,
    bigram: &Table,
    uni_top: u16,
    code: u16,
    query: u16,
    min_count: u64,
) -> u16 {
    let key = joint_key(code, query);
    if joint.key_total(key) >= min_count {
        if let Some(t) = joint.argmax(key) {
            return t;
        }
    }
    bigram.argmax(u32::from(query)).unwrap_or(uni_top)
}

struct CodeGroup {
    k: usize,
    same_k: bool,
    a_t: u64,
    a_omega: u64,
}

/// Minimum, quartiles and maximum by the lower nearest rank: index ⌊n·j/4⌋, capped at n − 1.
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
pub struct Spread {
    pub min: u64,
    pub q1: u64,
    pub median: u64,
    pub q3: u64,
    pub max: u64,
}

pub fn spread(mut v: Vec<u64>) -> Spread {
    if v.is_empty() {
        return Spread {
            min: 0,
            q1: 0,
            median: 0,
            q3: 0,
            max: 0,
        };
    }
    v.sort_unstable();
    let n = v.len();
    let at = |j: usize| v[std::cmp::min(n * j / 4, n - 1)];
    Spread {
        min: v[0],
        q1: at(1),
        median: at(2),
        q3: at(3),
        max: v[n - 1],
    }
}

fn hits(n: &[bool]) -> u64 {
    n.iter().filter(|&&h| h).count() as u64
}

fn bits(n: &[bool]) -> String {
    n.iter().map(|&h| if h { '1' } else { '0' }).collect()
}

fn joint_key(code: u16, query: u16) -> u32 {
    (u32::from(code) << 16) | u32::from(query)
}

/// The code with block `k`'s root removed: the other three roots, 12 bits.
fn drop_block(code: u16, k: usize) -> u32 {
    let roots = cap::unpack_code(code);
    roots
        .iter()
        .enumerate()
        .filter(|&(b, _)| b != k)
        .fold(0u32, |acc, (_, &r)| (acc << 4) | u32::from(r))
}

fn pair_key(code: u16, j: usize, k: usize) -> u32 {
    let roots = cap::unpack_code(code);
    (u32::from(roots[j]) << 4) | u32::from(roots[k])
}

/// A keyed table on TRAIN with a unigram fallback, scored on DEV.
fn keyed_hits(
    train: &[Train],
    dev: &[Dev],
    key: impl Fn(u16, u16) -> u32,
    fallback: u16,
) -> (Vec<bool>, Table) {
    let table = Table::build(
        train
            .iter()
            .map(|r| [key(r.code, r.query), u32::from(r.target)]),
    );
    let h = dev
        .iter()
        .map(|d| table.argmax(key(d.code, d.query)).unwrap_or(fallback) == d.target)
        .collect();
    (h, table)
}

struct Basic {
    unigram: Table,
    uni_top: u16,
    code: Table,
    bigram: Table,
    t_hits: Vec<bool>,
    g_hits: Vec<bool>,
}

fn basic(train: &[Train], dev: &[Dev]) -> Res<Basic> {
    let unigram = Table::build(train.iter().map(|r| [0, u32::from(r.target)]));
    let uni_top = match unigram.argmax(0) {
        Some(t) => t,
        None => return fault("empty TRAIN subsample"),
    };
    let (t_hits, code) = keyed_hits(train, dev, |c, _| u32::from(c), uni_top);
    let bigram = Table::build(
        train
            .iter()
            .map(|r| [u32::from(r.query), u32::from(r.target)]),
    );
    let g_hits = dev
        .iter()
        .map(|d| bigram.argmax(u32::from(d.query)).unwrap_or(uni_top) == d.target)
        .collect();
    Ok(Basic {
        unigram,
        uni_top,
        code,
        bigram,
        t_hits,
        g_hits,
    })
}

// ---------------------------------------------------------------- record

#[derive(Clone, Debug, Serialize)]
pub struct Check {
    pub name: &'static str,
    pub expected: u64,
    pub w2_function: u64,
    pub record: u64,
    pub instrument: Option<u64>,
    pub ok: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct Precedence0 {
    pub checks: Vec<Check>,
    pub sum_k: u64,
    pub f2: u128,
    pub row_mismatches: BTreeMap<String, u64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Qa {
    pub a_w: u64,
    pub a_t: u64,
    pub a_u: u64,
    pub a_g: u64,
    pub a_omega: u64,
    pub pair_t_w: Pair,
    pub pair_t_u: Pair,
    pub band: &'static str,
    pub a_u_beats_a_w: bool,
    pub t_adm: u64,
    pub t_non: u64,
    pub cell_split: CellSplit,
    pub candidate_load: BTreeMap<String, Spread>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CellSplit {
    pub writer_hit1: u64,
    pub writer_right_cell: u64,
    pub table_hit1: u64,
    pub table_right_cell: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ScalePoint {
    pub rows: u64,
    pub t: u64,
    pub g: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Qb {
    pub stride: Vec<ScalePoint>,
    pub prefix: Vec<ScalePoint>,
    pub pair_t0_t1: Pair,
    pub result: QbBand,
    pub g1: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct Qc {
    pub j: u64,
    pub g: u64,
    pub joint_seen_rows: u64,
    pub pair_j_g: Pair,
    pub band: &'static str,
    pub j_m2: u64,
    pub pair_j_m2_g: Pair,
}

#[derive(Clone, Debug, Serialize)]
pub struct Oracle {
    pub top1: u64,
    pub single_row_keys: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct InSample {
    pub resubstitution: u64,
    pub leave_one_out: u64,
    pub loo_without_resub_hit: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct Descriptive {
    pub omega_code: Oracle,
    pub omega_query: Oracle,
    pub omega_joint: Oracle,
    pub omega_code_at_most_t2_bar: bool,
    pub t12_drop_block: [u64; 4],
    pub t8_pairs: BTreeMap<String, u64>,
    pub gc_query_cell: u64,
    pub distinct_codes_train: u64,
    pub distinct_codes_dev: u64,
    pub dev_rows_unseen_code: u64,
    pub block_root_histograms: [[u64; 16]; 4],
    pub roots_used_per_block: [u64; 4],
    pub collision_effective_codes: Ratio,
    pub query_determinacy_train: u64,
    pub query_determinacy_dev: u64,
    pub in_sample_train: BTreeMap<String, InSample>,
    pub collision_agreement: BTreeMap<String, Ratio>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ArmR {
    pub status: &'static str,
    pub reason: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct Record {
    pub schema: &'static str,
    pub sheet: &'static str,
    pub discrete_commit: String,
    pub inputs: Vec<FilePin>,
    pub precedence_0: Precedence0,
    pub qa: Qa,
    pub qb: Qb,
    pub qc: Qc,
    pub descriptive: Descriptive,
    pub arm_r: ArmR,
    pub dev_hits: BTreeMap<String, String>,
}

fn record_u64(record: &Value, path: &str) -> u64 {
    path.split('.')
        .try_fold(record, |v, k| v.get(k))
        .and_then(Value::as_u64)
        .unwrap_or(u64::MAX)
}

// ---------------------------------------------------------------- the run

/// Run the sheet once. Precedence 0 must pass before any band is read.
pub fn run(inputs: &Inputs, record: &Value, discrete_commit: &str) -> Res<Record> {
    let dev = &inputs.dev;
    let train = &inputs.train;
    let cells = &inputs.cells;
    let cell_of = |t: u16| cells[usize::from(t)];

    // W2's own public functions, for precedence 0.
    let w2_train: Vec<cap::TrainRow> = train
        .iter()
        .map(|r| cap::TrainRow {
            code: r.code,
            query_token: r.query,
            target_token: r.target,
        })
        .collect();
    let w2_dev: Vec<cap::DevRow> = dev
        .iter()
        .map(|d| cap::DevRow {
            code: d.code,
            query_token: d.query,
            target_token: d.target,
            writer_hit1: d.writer_hit1,
            writer_hit8: d.writer_hit8,
        })
        .collect();
    let err = |e: String| Fault(vec![e]);
    let tables = cap::build_tables(&w2_train).map_err(|e| err(format!("build_tables: {e:?}")))?;
    let scores = cap::score_dev(&tables, &w2_dev).map_err(|e| err(format!("score_dev: {e:?}")))?;
    let agg = cap::aggregate(&tables, &scores);
    let actions = cap::symmetry_root_actions().map_err(err)?;
    let sym_keys = cap::symmetry_keys(&actions);
    let sym = cap::build_sym_table(&w2_train, &sym_keys);
    let slot_vecs: Vec<Vec<u16>> = dev.iter().map(|d| d.slots.to_vec()).collect();
    let admitted = cap::score_admitted(&tables, &sym, &sym_keys, &w2_dev, cells, &slot_vecs);
    let adm = cap::admission_matched(&admitted, &w2_dev);

    // The instrument's own tables on N_0.
    let b0 = basic(train, dev)?;
    let u_hits: Vec<bool> = dev.iter().map(|d| b0.uni_top == d.target).collect();
    let w_hits: Vec<bool> = dev.iter().map(|d| d.writer_hit1).collect();
    let aw_hits: Vec<bool> = dev
        .iter()
        .map(|d| d.admits_cell(cell_of(d.target)))
        .collect();
    let (ao_hits, _) = keyed_hits(train, dev, |c, _| pair_key(c, 0, 3), b0.uni_top);
    let (io_hits, _) = keyed_hits(train, dev, |c, _| pair_key(c, 1, 2), b0.uni_top);

    let mut row_mismatches = BTreeMap::new();
    let mut mismatch = |name: &str, n: u64| {
        row_mismatches.insert(name.to_string(), n);
    };
    let top1 = |p: &cap::Prediction| p.top8.first().copied();
    mismatch(
        "table",
        dev.iter()
            .zip(&scores)
            .zip(&b0.t_hits)
            .filter(|&((d, s), &h)| (top1(&s.table) == Some(d.target)) != h)
            .count() as u64,
    );
    mismatch(
        "bigram",
        dev.iter()
            .zip(&scores)
            .zip(&b0.g_hits)
            .filter(|&((d, s), &h)| (top1(&s.bigram) == Some(d.target)) != h)
            .count() as u64,
    );
    mismatch(
        "unigram",
        scores
            .iter()
            .zip(&u_hits)
            .filter(|&(s, &h)| s.unigram_hit1 != h)
            .count() as u64,
    );
    mismatch(
        "address_only",
        dev.iter()
            .zip(&scores)
            .zip(&ao_hits)
            .filter(|&((d, s), &h)| (top1(&s.address_only) == Some(d.target)) != h)
            .count() as u64,
    );
    mismatch(
        "interior_only",
        dev.iter()
            .zip(&scores)
            .zip(&io_hits)
            .filter(|&((d, s), &h)| (top1(&s.interior_only) == Some(d.target)) != h)
            .count() as u64,
    );
    mismatch(
        "target_admitted",
        admitted
            .iter()
            .zip(&aw_hits)
            .filter(|&(a, &h)| a.target_admitted != h)
            .count() as u64,
    );

    let desc = "aggregates.descriptive";
    let check = |name: &'static str, expected: u64, w2: u64, rec: &str, own: Option<u64>| {
        let record = record_u64(record, rec);
        Check {
            name,
            expected,
            w2_function: w2,
            record,
            instrument: own,
            ok: w2 == expected && record == expected && own.map_or(true, |o| o == expected),
        }
    };
    let checks = vec![
        check(
            "T",
            W2_T,
            agg.t_top1,
            "aggregates.T_table_matched_top1",
            Some(hits(&b0.t_hits)),
        ),
        check(
            "G",
            W2_G,
            agg.g_top1,
            "aggregates.G_bigram_matched_top1",
            Some(hits(&b0.g_hits)),
        ),
        check(
            "U",
            W2_U,
            agg.u_top1,
            "aggregates.U_unigram_matched_top1",
            Some(hits(&u_hits)),
        ),
        check(
            "W",
            W2_W,
            agg.w_top1,
            "aggregates.W_writer_top1",
            Some(hits(&w_hits)),
        ),
        check(
            "A_W",
            W2_A_W,
            adm.target_admitted_rows,
            &format!("{desc}.admission_matched.target_admitted_rows"),
            Some(hits(&aw_hits)),
        ),
        check(
            "restricted_table",
            W2_RESTRICTED_TABLE,
            adm.all[0].hit1,
            &format!("{desc}.admission_matched.all_rows.table.hit1"),
            None,
        ),
        check(
            "restricted_bigram",
            W2_RESTRICTED_BIGRAM,
            adm.all[2].hit1,
            &format!("{desc}.admission_matched.all_rows.bigram.hit1"),
            None,
        ),
        check(
            "coverage",
            W2_COVERAGE,
            agg.coverage_rows,
            "evaluability.coverage_rows",
            Some(
                dev.iter()
                    .filter(|d| b0.code.key_total(u32::from(d.code)) > 0)
                    .count() as u64,
            ),
        ),
        check(
            "codes_with_count_ge_2",
            W2_CODES_GE_2,
            agg.codes_with_count_ge_2,
            "evaluability.codes_with_count_ge_2",
            Some(b0.code.keys.values().filter(|r| r.total >= 2).count() as u64),
        ),
        check(
            "address_only",
            W2_ADDRESS_ONLY,
            agg.address_only_top1,
            &format!("{desc}.address_only_top1"),
            Some(hits(&ao_hits)),
        ),
        check(
            "interior_only",
            W2_INTERIOR_ONLY,
            agg.interior_only_top1,
            &format!("{desc}.interior_only_top1"),
            Some(hits(&io_hits)),
        ),
    ];
    let sum_k: u64 = dev.iter().map(|d| d.k as u64).sum();
    let f2 = [&agg.d_u, &agg.d_g]
        .iter()
        .map(|d| 4 * u128::from(d.b + d.c))
        .fold(576u128, std::cmp::max);
    let mut problems: Vec<String> = checks
        .iter()
        .filter(|c| !c.ok)
        .map(|c| {
            format!(
                "{}: expected {}, W2 function {}, record {}, instrument {:?}",
                c.name, c.expected, c.w2_function, c.record, c.instrument
            )
        })
        .collect();
    for (name, &n) in &row_mismatches {
        if n != 0 {
            problems.push(format!("{name}: {n} DEV rows differ from W2's own scoring"));
        }
    }
    if sum_k != W2_SUM_K {
        problems.push(format!("sum k_r {sum_k} != {W2_SUM_K}"));
    }
    if f2 != H_LAW_F2 {
        problems.push(format!("F² {f2} != {H_LAW_F2}"));
    }
    if agg.m_top1() != T2_BAR {
        problems.push(format!("M = max(U, G) {} != {T2_BAR}", agg.m_top1()));
    }
    let conserved = |t: &Table, n: usize| {
        t.total == n as u64 && t.keys.values().map(|r| r.total).sum::<u64>() == n as u64
    };
    if !conserved(&b0.code, TRAIN_ROWS)
        || !conserved(&b0.bigram, TRAIN_ROWS)
        || !conserved(&b0.unigram, TRAIN_ROWS)
    {
        problems.push("count-table conservation fails on N_0".into());
    }

    // ---- QA
    let mut mass = [0u64; CELLS];
    for e in &b0.unigram.keys[&0].entries {
        mass[usize::from(cell_of(e.token))] += e.count;
    }
    if mass.iter().sum::<u64>() != TRAIN_ROWS as u64 {
        problems.push("cell masses do not sum to N_0".into());
    }
    let mut tokens_in_cell = [0u64; CELLS];
    for &c in cells.iter() {
        tokens_in_cell[usize::from(c)] += 1;
    }
    let ranked_u = rank_cells(&[0u64; CELLS], &mass);
    let mut by_code: BTreeMap<u16, Vec<u16>> = BTreeMap::new();
    let mut by_query: BTreeMap<u16, Vec<u16>> = BTreeMap::new();
    let mut dev_code_cells: BTreeMap<u16, [u64; CELLS]> = BTreeMap::new();
    for d in dev {
        by_code
            .entry(d.code)
            .or_insert_with(|| ranked_for(&b0.code, u32::from(d.code), cells, &mass));
        by_query
            .entry(d.query)
            .or_insert_with(|| ranked_for(&b0.bigram, u32::from(d.query), cells, &mass));
        dev_code_cells.entry(d.code).or_insert([0u64; CELLS])[usize::from(cell_of(d.target))] += 1;
    }
    let by_omega: BTreeMap<u16, Vec<u16>> = dev_code_cells
        .iter()
        .map(|(&c, s)| (c, rank_cells(s, &mass)))
        .collect();
    let load = |ranked: &[u16], k: usize| -> u64 {
        ranked
            .iter()
            .take(k)
            .map(|&c| tokens_in_cell[usize::from(c)])
            .sum()
    };
    let mut at_hits = Vec::with_capacity(dev.len());
    let mut au_hits = Vec::with_capacity(dev.len());
    let mut ag_hits = Vec::with_capacity(dev.len());
    let mut ao_adm_hits = Vec::with_capacity(dev.len());
    let mut loads: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    for d in dev {
        let cell = cell_of(d.target);
        let rc = &by_code[&d.code];
        let rq = &by_query[&d.query];
        let ro = &by_omega[&d.code];
        at_hits.push(in_top(rc, d.k, cell));
        au_hits.push(in_top(&ranked_u, d.k, cell));
        ag_hits.push(in_top(rq, d.k, cell));
        ao_adm_hits.push(in_top(ro, d.k, cell));
        let s_load: u64 = d
            .slots
            .iter()
            .filter(|&&s| s != ABSENT)
            .map(|&s| tokens_in_cell[usize::from(s)])
            .sum();
        loads.entry("writer_S".into()).or_default().push(s_load);
        loads.entry("A_T".into()).or_default().push(load(rc, d.k));
        loads
            .entry("A_U".into())
            .or_default()
            .push(load(&ranked_u, d.k));
        loads
            .entry("A_Omega".into())
            .or_default()
            .push(load(ro, d.k));
    }
    let pair_t_w = Pair::of(&at_hits, &aw_hits);
    let pair_t_u = Pair::of(&at_hits, &au_hits);
    let t_adm = dev
        .iter()
        .zip(&b0.t_hits)
        .filter(|&(d, &h)| h && d.admits_cell(cell_of(d.target)))
        .count() as u64;
    let t_total = hits(&b0.t_hits);
    if t_adm > W2_RESTRICTED_TABLE || t_total.saturating_sub(t_adm) < W2_T - W2_RESTRICTED_TABLE {
        problems.push(format!("T_adm {t_adm} breaks the W2 bounds"));
    }
    let a_t = hits(&at_hits);
    let a_omega = hits(&ao_adm_hits);
    // A_T ≤ A_Ω holds per code wherever k_r is constant within that code.
    let mut groups: BTreeMap<u16, CodeGroup> = BTreeMap::new();
    for ((d, &at), &ao) in dev.iter().zip(&at_hits).zip(&ao_adm_hits) {
        let g = groups.entry(d.code).or_insert(CodeGroup {
            k: d.k,
            same_k: true,
            a_t: 0,
            a_omega: 0,
        });
        g.same_k &= g.k == d.k;
        g.a_t += u64::from(at);
        g.a_omega += u64::from(ao);
    }
    let broken = groups
        .values()
        .filter(|g| g.same_k && g.a_t > g.a_omega)
        .count();
    if broken != 0 {
        problems.push(format!("A_T > A_Omega within {broken} codes"));
    }
    let table_pred = |d: &Dev| b0.code.argmax(u32::from(d.code)).unwrap_or(b0.uni_top);
    let cell_split = CellSplit {
        writer_hit1: hits(&w_hits),
        writer_right_cell: dev
            .iter()
            .filter(|d| cell_of(d.writer_top1) == cell_of(d.target))
            .count() as u64,
        table_hit1: t_total,
        table_right_cell: dev
            .iter()
            .filter(|d| cell_of(table_pred(d)) == cell_of(d.target))
            .count() as u64,
    };
    let qa = Qa {
        a_w: hits(&aw_hits),
        a_t,
        a_u: hits(&au_hits),
        a_g: hits(&ag_hits),
        a_omega,
        pair_t_w,
        pair_t_u,
        band: qa_band(pair_t_w, pair_t_u),
        a_u_beats_a_w: beats(Pair::of(&au_hits, &aw_hits)),
        t_adm,
        t_non: t_total - t_adm,
        cell_split,
        candidate_load: loads.into_iter().map(|(k, v)| (k, spread(v))).collect(),
    };

    // ---- QB
    let mut stride = Vec::new();
    let mut prefix = Vec::new();
    let mut t_by_stride: Vec<Vec<bool>> = Vec::new();
    let mut extra_hits: BTreeMap<String, String> = BTreeMap::new();
    for j in 0..SUBSAMPLES {
        let step = 1usize << j;
        let rows: Vec<Train> = train.iter().step_by(step).copied().collect();
        let n = rows.len();
        let bj = if j == 0 {
            None
        } else {
            Some(basic(&rows, dev)?)
        };
        let bj = bj.as_ref().unwrap_or(&b0);
        if !conserved(&bj.code, n) || !conserved(&bj.bigram, n) || !conserved(&bj.unigram, n) {
            problems.push(format!("count-table conservation fails on N_{j}"));
        }
        stride.push(ScalePoint {
            rows: n as u64,
            t: hits(&bj.t_hits),
            g: hits(&bj.g_hits),
        });
        if j > 0 {
            extra_hits.insert(format!("T_N{j}"), bits(&bj.t_hits));
            extra_hits.insert(format!("G_N{j}"), bits(&bj.g_hits));
            let bp = basic(&train[..n], dev)?;
            prefix.push(ScalePoint {
                rows: n as u64,
                t: hits(&bp.t_hits),
                g: hits(&bp.g_hits),
            });
        }
        t_by_stride.push(bj.t_hits.clone());
    }
    let pair_t0_t1 = Pair::of(&t_by_stride[0], &t_by_stride[1]);
    let result = qb_band(t_total, pair_t0_t1, f2);
    let qb = Qb {
        stride,
        prefix,
        pair_t0_t1,
        result,
        g1: if result.band == "B1" {
            "REGISTER under its own pre-run review"
        } else {
            "DECLINED"
        },
    };

    // ---- QC
    let joint = Table::build(
        train
            .iter()
            .map(|r| [joint_key(r.code, r.query), u32::from(r.target)]),
    );
    let mut j_hits = Vec::with_capacity(dev.len());
    let mut jm2_hits = Vec::with_capacity(dev.len());
    let mut joint_seen_rows = 0u64;
    for d in dev {
        if joint.key_total(joint_key(d.code, d.query)) > 0 {
            joint_seen_rows += 1;
        }
        let j = joint_prediction(&joint, &b0.bigram, b0.uni_top, d.code, d.query, 1);
        let jm2 = joint_prediction(&joint, &b0.bigram, b0.uni_top, d.code, d.query, 2);
        j_hits.push(j == d.target);
        jm2_hits.push(jm2 == d.target);
    }
    let pair_j_g = Pair::of(&j_hits, &b0.g_hits);
    let qc = Qc {
        j: hits(&j_hits),
        g: hits(&b0.g_hits),
        joint_seen_rows,
        pair_j_g,
        band: qc_band(joint_seen_rows, pair_j_g),
        j_m2: hits(&jm2_hits),
        pair_j_m2_g: Pair::of(&jm2_hits, &b0.g_hits),
    };
    if joint.total != TRAIN_ROWS as u64 {
        problems.push("joint table total != N_0".into());
    }

    // ---- descriptive
    let dev_table =
        |key: &dyn Fn(&Dev) -> u32| Table::build(dev.iter().map(|d| [key(d), u32::from(d.target)]));
    let oracle = |t: &Table| Oracle {
        top1: t.oracle(),
        single_row_keys: t.single_row_keys(),
    };
    let omega_code = oracle(&dev_table(&|d| u32::from(d.code)));
    let omega_query = oracle(&dev_table(&|d| u32::from(d.query)));
    let omega_joint = oracle(&dev_table(&|d| joint_key(d.code, d.query)));
    if t_total > omega_code.top1 {
        problems.push("T > Omega_code".into());
    }
    let mut t12 = [0u64; 4];
    for (k, slot) in t12.iter_mut().enumerate() {
        let (h, _) = keyed_hits(train, dev, |c, _| drop_block(c, k), b0.uni_top);
        *slot = hits(&h);
    }
    let mut t8 = BTreeMap::new();
    for j in 0..4 {
        for k in (j + 1)..4 {
            let (h, _) = keyed_hits(train, dev, |c, _| pair_key(c, j, k), b0.uni_top);
            let n = hits(&h);
            t8.insert(format!("blocks_{j}_{k}"), n);
        }
    }
    let (gc_hits, _) = keyed_hits(train, dev, |_, q| u32::from(cell_of(q)), b0.uni_top);
    let gc = hits(&gc_hits);
    let mut hist = [[0u64; 16]; 4];
    for r in train {
        for (b, &root) in cap::unpack_code(r.code).iter().enumerate() {
            hist[b][usize::from(root & 0xF)] += 1;
        }
    }
    let mut roots_used = [0u64; 4];
    for (b, h) in hist.iter().enumerate() {
        roots_used[b] = h.iter().filter(|&&n| n > 0).count() as u64;
    }
    let sq: u128 = b0
        .code
        .keys
        .values()
        .map(|r| u128::from(r.total) * u128::from(r.total))
        .sum();
    let n0 = TRAIN_ROWS as u128;
    let code_query = Table::build(
        train
            .iter()
            .map(|r| [u32::from(r.code), u32::from(r.query)]),
    );
    let dev_code_query = Table::build(dev.iter().map(|d| [u32::from(d.code), u32::from(d.query)]));
    let distinct_dev: std::collections::BTreeSet<u16> = dev.iter().map(|d| d.code).collect();

    let uni_loo = |y: u16| b0.unigram.loo(0, y).unwrap_or(false);
    let g_loo = |q: u16, y: u16| b0.bigram.loo(u32::from(q), y).unwrap_or_else(|| uni_loo(y));
    let mut in_sample = BTreeMap::new();
    let mut tally = |name: &str, resub: &dyn Fn(&Train) -> bool, loo: &dyn Fn(&Train) -> bool| {
        let mut s = InSample {
            resubstitution: 0,
            leave_one_out: 0,
            loo_without_resub_hit: 0,
        };
        for r in train {
            let (rh, lh) = (resub(r), loo(r));
            s.resubstitution += u64::from(rh);
            s.leave_one_out += u64::from(lh);
            s.loo_without_resub_hit += u64::from(lh && !rh);
        }
        in_sample.insert(name.to_string(), s);
    };
    tally("U", &|r| b0.uni_top == r.target, &|r| uni_loo(r.target));
    tally(
        "G",
        &|r| b0.bigram.argmax(u32::from(r.query)) == Some(r.target),
        &|r| g_loo(r.query, r.target),
    );
    tally(
        "T",
        &|r| b0.code.argmax(u32::from(r.code)) == Some(r.target),
        &|r| {
            b0.code
                .loo(u32::from(r.code), r.target)
                .unwrap_or_else(|| uni_loo(r.target))
        },
    );
    tally(
        "J",
        &|r| joint.argmax(joint_key(r.code, r.query)) == Some(r.target),
        &|r| {
            joint
                .loo(joint_key(r.code, r.query), r.target)
                .unwrap_or_else(|| g_loo(r.query, r.target))
        },
    );
    for (name, s) in &in_sample {
        if s.loo_without_resub_hit != 0 {
            problems.push(format!(
                "{name}: leave-one-out hit without a resubstitution hit"
            ));
        }
    }
    let mut collision = BTreeMap::new();
    collision.insert("unigram".to_string(), b0.unigram.collision());
    collision.insert("query".to_string(), b0.bigram.collision());
    collision.insert("code".to_string(), b0.code.collision());
    collision.insert("joint".to_string(), joint.collision());

    let descriptive = Descriptive {
        omega_code_at_most_t2_bar: omega_code.top1 <= T2_BAR,
        omega_code,
        omega_query,
        omega_joint,
        t12_drop_block: t12,
        t8_pairs: t8,
        gc_query_cell: gc,
        distinct_codes_train: b0.code.keys.len() as u64,
        distinct_codes_dev: distinct_dev.len() as u64,
        dev_rows_unseen_code: dev
            .iter()
            .filter(|d| b0.code.key_total(u32::from(d.code)) == 0)
            .count() as u64,
        block_root_histograms: hist,
        roots_used_per_block: roots_used,
        collision_effective_codes: Ratio {
            num: n0 * n0,
            den: sq,
        },
        query_determinacy_train: code_query.oracle(),
        query_determinacy_dev: dev_code_query.oracle(),
        in_sample_train: in_sample,
        collision_agreement: collision,
    };

    if !problems.is_empty() {
        return Err(Fault(problems));
    }

    let mut dev_hits = extra_hits;
    for (name, h) in [
        ("T", &b0.t_hits),
        ("G", &b0.g_hits),
        ("U", &u_hits),
        ("W", &w_hits),
        ("A_W", &aw_hits),
        ("A_T", &at_hits),
        ("A_U", &au_hits),
        ("A_G", &ag_hits),
        ("A_Omega", &ao_adm_hits),
        ("J", &j_hits),
        ("J_m2", &jm2_hits),
    ] {
        dev_hits.insert(name.to_string(), bits(h));
    }

    Ok(Record {
        schema: SCHEMA,
        sheet: "docs/harness/prep/m1-w2-code-utilization-2026-10-25.md",
        discrete_commit: discrete_commit.to_string(),
        inputs: inputs.pins.clone(),
        precedence_0: Precedence0 {
            checks,
            sum_k,
            f2,
            row_mismatches,
        },
        qa,
        qb,
        qc,
        descriptive,
        arm_r: ArmR {
            status: "NOT_RUN",
            reason: "not attempted by this tool: its inputs, the Q1-Q3 checkpoints (a5a22dce.., dde889aa.., 102beed4..), were not found by the lab's search of the HF store's local-archive/2026-10-20 file list and archive/storage (claude-95 REPORT); the sheet makes R NOT_RUN without its inputs, never a substitute",
        },
        dev_hits,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(b: u64, c: u64) -> Pair {
        Pair { b, c }
    }

    #[test]
    fn beats_uses_the_squared_floor() {
        // b + c small: the 576 floor rules, so c − b must exceed 24.
        assert!(!beats(p(0, 24)));
        assert!(beats(p(0, 25)));
        // b + c = 200: floor 800, so c − b = 28 (784) fails and 29 (841) passes.
        assert!(!beats(p(86, 114)));
        assert!(beats(p(85, 114)));
        assert!(!beats(p(10, 10)));
        assert!(!beats(p(30, 0)));
    }

    #[test]
    fn underpowered_needs_small_n_and_no_rejection() {
        assert!(underpowered(p(5, 6)));
        assert!(!underpowered(p(60, 60)));
        assert!(!underpowered(p(0, 30)));
    }

    #[test]
    fn qa_band_covers_every_row() {
        let strong = p(100, 400);
        let weak = p(400, 100);
        let tie = p(300, 310);
        let small_tie = p(5, 6);
        assert_eq!(qa_band(strong, strong), "A1a");
        assert_eq!(qa_band(strong, tie), "A1b");
        assert_eq!(qa_band(strong, weak), "A1b");
        assert_eq!(qa_band(strong, small_tie), "A1u");
        assert_eq!(qa_band(tie, strong), "A2");
        assert_eq!(qa_band(weak, strong), "A3");
        assert_eq!(qa_band(small_tie, strong), "UNDERPOWERED");
        // An underpowered A_T-against-A_U pair cannot void A2 or A3.
        assert_eq!(qa_band(weak, small_tie), "A3");
        assert_eq!(qa_band(tie, small_tie), "A2");
    }

    #[test]
    fn qb_band_needs_t_n1_at_most_1903() {
        // T(N_0) = 3,414: B1 needs d_T >= 1,511.
        let t0 = 3_414;
        let b1 = qb_band(t0, p(0, 1_511), H_LAW_F2);
        assert_eq!((b1.t_proj, b1.band), (6_436, "B1"));
        let b2 = qb_band(t0, p(0, 1_510), H_LAW_F2);
        assert_eq!((b2.t_proj, b2.band), (6_434, "B2"));
        // A noise-level gain counts as d_T = 0.
        let flat = qb_band(t0, p(100, 110), H_LAW_F2);
        assert_eq!((flat.d_t, flat.band), (0, "B2"));
    }

    #[test]
    fn qc_band_and_its_guard() {
        assert_eq!(qc_band(5_000, p(100, 400)), "C1");
        assert_eq!(qc_band(5_000, p(300, 310)), "C2");
        assert_eq!(qc_band(5_000, p(400, 100)), "C3");
        assert_eq!(qc_band(1_428, p(100, 400)), "UNDERPOWERED");
        assert_eq!(qc_band(5_000, p(5, 6)), "UNDERPOWERED");
    }

    #[test]
    fn table_tie_rule_and_leave_one_out() {
        // key 1: token 7 ×2, token 3 ×2, token 9 ×1 → top is 3 (lower id on a tie).
        let t = Table::build(
            [[1, 7], [1, 7], [1, 3], [1, 3], [1, 9], [2, 5]]
                .iter()
                .copied(),
        );
        assert_eq!(t.argmax(1), Some(3));
        assert_eq!(t.count(1, 7), 2);
        assert_eq!(t.key_total(1), 5);
        // Removing one 3 leaves 7 ×2 against 3 ×1: 3 loses.
        assert_eq!(t.loo(1, 3), Some(false));
        // Removing one 7 leaves 3 ×2 against 7 ×1: 7 loses.
        assert_eq!(t.loo(1, 7), Some(false));
        // A single-row key falls back.
        assert_eq!(t.loo(2, 5), None);
        let u = Table::build([[0, 4], [0, 4], [0, 4], [0, 2]].iter().copied());
        assert_eq!(u.loo(0, 4), Some(true));
        assert_eq!(u.loo(0, 2), Some(false));
        assert_eq!(u.oracle(), 3);
        let c = u.collision();
        assert_eq!((c.num, c.den), (6, 12));
    }

    #[test]
    fn leave_one_out_edge_cases() {
        // y is the only token, total 2: still wins after removal.
        let only = Table::build([[1, 4], [1, 4]].iter().copied());
        assert_eq!(only.loo(1, 4), Some(true));
        // After removal y ties the runner-up: the lower id wins.
        let low = Table::build([[1, 2], [1, 2], [1, 5]].iter().copied());
        assert_eq!(low.loo(1, 2), Some(true));
        let high = Table::build([[1, 5], [1, 5], [1, 2]].iter().copied());
        assert_eq!(high.loo(1, 5), Some(false));
    }

    #[test]
    fn in_top_respects_the_row_k() {
        let mut ranked: Vec<u16> = vec![5, 9, 2];
        ranked.extend((0..CELLS as u16).filter(|c| ![5, 9, 2].contains(c)));
        assert!(in_top(&ranked, 2, 9));
        assert!(!in_top(&ranked, 2, 2));
        assert!(!in_top(&ranked, 1, 9));
        assert!(!in_top(&ranked, 0, 5));
        assert!(in_top(&ranked, 3, 2));
    }

    #[test]
    fn ranked_for_sums_targets_per_cell_and_breaks_ties_by_mass() {
        let mut cells = vec![0u16; VOCAB];
        cells[10] = 7;
        cells[11] = 7;
        cells[12] = 3;
        // Key 42: cell 7 gets 1 + 1, cell 3 gets 2: a score tie.
        let t = Table::build([[42, 10], [42, 11], [42, 12], [42, 12]].iter().copied());
        let mut mass = [0u64; CELLS];
        mass[7] = 5;
        mass[3] = 1;
        assert_eq!(&ranked_for(&t, 42, &cells, &mass)[..2], &[7, 3]);
        mass[3] = 9;
        assert_eq!(&ranked_for(&t, 42, &cells, &mass)[..2], &[3, 7]);
        // An unseen key ranks by mass alone.
        mass[200] = 50;
        assert_eq!(ranked_for(&t, 99, &cells, &mass)[0], 200);
    }

    #[test]
    fn joint_prediction_falls_back_by_min_count() {
        let key = joint_key(3, 8);
        let joint = Table::build([[key, 77]].iter().copied());
        let bigram = Table::build([[8, 55], [8, 55]].iter().copied());
        assert_eq!(joint_prediction(&joint, &bigram, 1, 3, 8, 1), 77);
        assert_eq!(joint_prediction(&joint, &bigram, 1, 3, 8, 2), 55);
        assert_eq!(joint_prediction(&joint, &bigram, 1, 4, 9, 1), 1);
    }

    #[test]
    fn rank_cells_breaks_ties_by_mass_then_id() {
        let mut score = [0u64; CELLS];
        let mut mass = [0u64; CELLS];
        score[9] = 5;
        score[4] = 5;
        mass[4] = 1;
        mass[200] = 9;
        let r = rank_cells(&score, &mass);
        assert_eq!(&r[..4], &[4, 9, 200, 0]);
    }

    #[test]
    fn spread_uses_lower_nearest_rank() {
        let s = spread(vec![5, 1, 4, 2, 3]);
        assert_eq!((s.min, s.q1, s.median, s.q3, s.max), (1, 2, 3, 4, 5));
    }

    #[test]
    fn projections_keep_the_named_blocks() {
        let code = cap::pack_code([1, 2, 3, 4]);
        assert_eq!(drop_block(code, 0), 0x234);
        assert_eq!(drop_block(code, 3), 0x123);
        assert_eq!(pair_key(code, 0, 3), 0x14);
        assert_eq!(pair_key(code, 1, 2), 0x23);
        assert_eq!(joint_key(0xABCD, 0x0123), 0xABCD_0123);
    }
}
