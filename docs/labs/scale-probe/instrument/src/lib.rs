//! P5 scale-probe laptop side: nested training mixes (`mix`), count-table grading of one
//! checkpoint's rows (`grade`), the TRAIN 8-gram context census of the rows (`seen`) and the
//! paired-band comparison of two cells (`pair`).
//! Counting only; integer arithmetic only; every inconsistency is a refusal.

use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use uor_r4_core::native_geometric::mmap_corpus::{CorpusWriter, MmapCorpusReader};

/// Token id bound for rows and count tables.
pub const VOCAB: usize = 4096;
pub const MIX_SCHEMA: &str = "uor-r4-scale-probe-mix/v1";
pub const GRADE_SCHEMA: &str = "uor-r4-scale-probe-grade/v1";
pub const PAIR_SCHEMA: &str = "uor-r4-scale-probe-pair/v1";
pub const HITS_FILE: &str = "nearest-hits.bin";
pub const T2_HITS_FILE: &str = "t2-hits.bin";
pub const UNIGRAM_HITS_FILE: &str = "unigram-hits.bin";
pub const SEEN_SCHEMA: &str = "uor-r4-scale-probe-seen/v1";
pub const EXCLUDED_FILE: &str = "excluded.bin";
/// Context length of the TRAIN n-gram census.
pub const NGRAM: usize = 8;

type R<T> = Result<T, String>;

/// Entry point for `main`: `args[0]` is the command.
pub fn run(args: &[String]) -> R<String> {
    let (cmd, rest) = args
        .split_first()
        .ok_or("usage: scale-probe mix|grade|seen|pair key=value...")?;
    let a = Args::parse(rest)?;
    match cmd.as_str() {
        "mix" => run_mix(&a),
        "grade" => run_grade(&a),
        "seen" => run_seen(&a),
        "pair" => run_pair(&a),
        other => Err(format!("unknown command {other:?}")),
    }
}

/// `key=value` arguments, repeated keys kept in order.
pub struct Args(Vec<(String, String)>);

impl Args {
    pub fn parse(items: &[String]) -> R<Self> {
        let mut v = Vec::new();
        for item in items {
            let (k, val) = item
                .split_once('=')
                .ok_or(format!("argument {item:?}: expected key=value"))?;
            v.push((k.to_string(), val.to_string()));
        }
        Ok(Args(v))
    }
    fn known(&self, keys: &[&str]) -> R<()> {
        match self.0.iter().find(|(k, _)| !keys.contains(&k.as_str())) {
            Some((k, _)) => Err(format!("unknown argument {k}=")),
            None => Ok(()),
        }
    }
    fn all(&self, key: &str) -> Vec<&str> {
        self.0
            .iter()
            .filter(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
            .collect()
    }
    fn opt(&self, key: &str) -> R<Option<&str>> {
        match self.all(key).as_slice() {
            [] => Ok(None),
            [v] => Ok(Some(v)),
            _ => Err(format!("{key}= given more than once")),
        }
    }
    fn one(&self, key: &str) -> R<&str> {
        self.opt(key)?.ok_or(format!("{key}= is required"))
    }
}

fn sha_bytes(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn read(path: &Path) -> R<Vec<u8>> {
    fs::read(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn write(path: &Path, bytes: &[u8]) -> R<()> {
    fs::write(path, bytes).map_err(|e| format!("{}: {e}", path.display()))
}

fn pretty(v: &Value) -> Vec<u8> {
    let mut s = serde_json::to_string_pretty(v).expect("json value serialises");
    s.push('\n');
    s.into_bytes()
}

fn refuse_existing(out: &Path) -> R<()> {
    if out.exists() {
        return Err(format!(
            "{}: out already exists; never overwritten",
            out.display()
        ));
    }
    Ok(())
}

fn create_out_dir(out: &Path) -> R<()> {
    if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    fs::create_dir(out).map_err(|e| format!("{}: {e}", out.display()))
}

/// `DIR:TOKSHA:MASKSHA` (the dir may itself hold `:`).
fn split_spec(spec: &str) -> R<(PathBuf, String, String)> {
    let mut it = spec.rsplitn(3, ':');
    let (mask, tok, dir) = (it.next(), it.next(), it.next());
    match (dir, tok, mask) {
        (Some(d), Some(t), Some(m)) if !d.is_empty() => Ok((
            PathBuf::from(d),
            t.to_ascii_lowercase(),
            m.to_ascii_lowercase(),
        )),
        _ => Err(format!("{spec:?}: expected DIR:TOKSHA:MASKSHA")),
    }
}

/// `NAME:DIR:TOKSHA:MASKSHA`.
fn named_spec(spec: &str) -> R<(String, PathBuf, String, String)> {
    let (name, rest) = spec
        .split_once(':')
        .filter(|(n, _)| !n.is_empty())
        .ok_or(format!("{spec:?}: expected NAME:DIR:TOKSHA:MASKSHA"))?;
    let (dir, t, m) = split_spec(rest)?;
    Ok((name.to_string(), dir, t, m))
}

/// One split directory, sha-checked against the caller's pins.
pub struct Split {
    pub dir: PathBuf,
    pub tokens: Vec<u16>,
    pub mask: Vec<u8>,
    pub vocab: u32,
    pub manifest: Option<Value>,
    pub tokens_sha256: String,
    pub mask_sha256: String,
}

/// Loads `dir/{tokens.u16,response_mask.u8,manifest.json}` and refuses any inconsistency.
pub fn load_split(dir: &Path, tok_sha: &str, mask_sha: &str, need_manifest: bool) -> R<Split> {
    let tp = dir.join("tokens.u16");
    let mp = dir.join("response_mask.u8");
    let tokens_sha256 = sha_bytes(&read(&tp)?);
    if tokens_sha256 != tok_sha {
        return Err(format!(
            "{}: sha256 {tokens_sha256} != given {tok_sha}",
            tp.display()
        ));
    }
    let mask = read(&mp)?;
    let mask_sha256 = sha_bytes(&mask);
    if mask_sha256 != mask_sha {
        return Err(format!(
            "{}: sha256 {mask_sha256} != given {mask_sha}",
            mp.display()
        ));
    }
    let mf = dir.join("manifest.json");
    let manifest = if mf.exists() {
        let v: Value = serde_json::from_slice(&read(&mf)?)
            .map_err(|e| format!("{}: unreadable json: {e}", mf.display()))?;
        for (key, given) in [("tokens_sha256", tok_sha), ("mask_sha256", mask_sha)] {
            if v.get(key).and_then(Value::as_str) != Some(given) {
                return Err(format!(
                    "{}: {key} {} disagrees with given {given}",
                    mf.display(),
                    v.get(key).unwrap_or(&Value::Null)
                ));
            }
        }
        if v.get("bos_id").and_then(Value::as_u64) != Some(0) {
            return Err(format!(
                "{}: bos_id {} is not 0",
                mf.display(),
                v.get("bos_id").unwrap_or(&Value::Null)
            ));
        }
        Some(v)
    } else if need_manifest {
        return Err(format!(
            "{}: missing; a mix root needs its manifest",
            mf.display()
        ));
    } else {
        None
    };
    let reader = MmapCorpusReader::open(&tp).map_err(|e| format!("{}: {e}", tp.display()))?;
    let tokens = reader.as_slice().to_vec();
    let vocab = reader.vocab_size();
    if tokens.len() != mask.len() {
        return Err(format!(
            "{}: {} tokens but {} holds {} mask bytes",
            tp.display(),
            tokens.len(),
            mp.display(),
            mask.len()
        ));
    }
    if tokens.first() != Some(&0) {
        return Err(format!(
            "{}: tokens[0] is {:?}, not BOS 0",
            tp.display(),
            tokens.first()
        ));
    }
    if let Some(i) = tokens.iter().position(|&t| t as usize >= VOCAB) {
        return Err(format!(
            "{}: token {i} is {}, not below the vocabulary of {VOCAB}",
            tp.display(),
            tokens[i]
        ));
    }
    if let Some(i) = mask.iter().position(|&m| m > 1) {
        return Err(format!(
            "{}: mask[{i}] = {} is not 0 or 1",
            mp.display(),
            mask[i]
        ));
    }
    Ok(Split {
        dir: dir.to_path_buf(),
        tokens,
        mask,
        vocab,
        manifest,
        tokens_sha256,
        mask_sha256,
    })
}

/// A scored position: not the first, not BOS, response mask 1.
pub fn is_scored(tokens: &[u16], mask: &[u8], p: usize) -> bool {
    p >= 1 && tokens[p] != 0 && mask[p] == 1
}

fn count_scored(tokens: &[u16], mask: &[u8], end: usize) -> u64 {
    (1..end).filter(|&p| is_scored(tokens, mask, p)).count() as u64
}

/// `N/D` with `1 <= N <= D <= 64`.
pub fn parse_fraction(s: &str) -> R<(u64, u64)> {
    let bad = || format!("fraction {s:?}: expected N/D with 1 <= N <= D <= 64");
    let (n, d) = s.split_once('/').ok_or_else(bad)?;
    let (n, d): (u64, u64) = (n.parse().map_err(|_| bad())?, d.parse().map_err(|_| bad())?);
    if !(1 <= n && n <= d && d <= 64) {
        return Err(bad());
    }
    Ok((n, d))
}

/// The shortest document-boundary prefix `[0, L)` with `L * D >= total * N`.
pub fn prefix_len(tokens: &[u16], n: u64, d: u64) -> usize {
    let need = tokens.len() as u128 * u128::from(n);
    for (i, &t) in tokens.iter().enumerate() {
        if t == 0 && i as u128 * u128::from(d) >= need {
            return i;
        }
    }
    tokens.len()
}

fn run_mix(a: &Args) -> R<String> {
    a.known(&["out", "fraction", "root"])?;
    let out = PathBuf::from(a.one("out")?);
    refuse_existing(&out)?;
    let frac = a.one("fraction")?;
    let (n, d) = parse_fraction(frac)?;
    let specs = a.all("root");
    if specs.is_empty() {
        return Err("at least one root= is required".into());
    }
    let mut names = BTreeSet::new();
    let mut roots: Vec<(String, Split)> = Vec::new();
    for spec in specs {
        let (name, dir, t, m) = named_spec(spec)?;
        if !names.insert(name.clone()) {
            return Err(format!("root {name}: duplicate root name"));
        }
        let s = load_split(&dir, &t, &m, true)?;
        if let Some((first, f)) = roots.first() {
            if f.vocab != s.vocab {
                return Err(format!(
                    "{}: vocab_size {} differs from root {first}'s {}",
                    s.dir.join("tokens.u16").display(),
                    s.vocab,
                    f.vocab
                ));
            }
        }
        roots.push((name, s));
    }
    let mut root_rows = Vec::new();
    let (mut tot_tokens, mut tot_docs, mut tot_scored) = (0u64, 0u64, 0u64);
    for (name, s) in &roots {
        let mf = s.manifest.as_ref().expect("mix roots carry a manifest");
        let mut row = json!({
            "name": name,
            "tokens_sha256": s.tokens_sha256,
            "mask_sha256": s.mask_sha256,
        });
        for key in ["source", "revision", "licence", "attribution"] {
            let v = mf.get(key).ok_or(format!(
                "{}: manifest lacks {key}",
                s.dir.join("manifest.json").display()
            ))?;
            row[key] = v.clone();
        }
        let l = prefix_len(&s.tokens, n, d);
        let docs_total = s.tokens.iter().filter(|&&t| t == 0).count() as u64;
        let docs_taken = s.tokens[..l].iter().filter(|&&t| t == 0).count() as u64;
        let scored = count_scored(&s.tokens, &s.mask, l);
        row["documents_total"] = json!(docs_total);
        row["documents_taken"] = json!(docs_taken);
        row["tokens_total"] = json!(s.tokens.len() as u64);
        row["tokens_taken"] = json!(l as u64);
        row["scored_taken"] = json!(scored);
        tot_tokens += l as u64;
        tot_docs += docs_taken;
        tot_scored += scored;
        root_rows.push(row);
    }
    create_out_dir(&out)?;
    let tp = out.join("tokens.u16");
    let first = &roots[0];
    let mut w =
        CorpusWriter::create(&tp, first.1.vocab).map_err(|e| format!("{}: {e}", tp.display()))?;
    let mut mask = Vec::with_capacity(tot_tokens as usize);
    for ((_, s), row) in roots.iter().zip(&root_rows) {
        let l = row["tokens_taken"].as_u64().expect("set above") as usize;
        w.write_tokens(&s.tokens[..l])
            .map_err(|e| format!("{}: {e}", tp.display()))?;
        mask.extend_from_slice(&s.mask[..l]);
    }
    w.finish().map_err(|e| format!("{}: {e}", tp.display()))?;
    let mp = out.join("response_mask.u8");
    write(&mp, &mask)?;
    let manifest = json!({
        "schema": MIX_SCHEMA,
        "fraction": format!("{n}/{d}"),
        "roots": root_rows,
        "tokens": tot_tokens,
        "documents": tot_docs,
        "scored": tot_scored,
        "tokens_sha256": sha_bytes(&read(&tp)?),
        "mask_sha256": sha_bytes(&read(&mp)?),
        "bos_id": 0,
    });
    write(&out.join("manifest.json"), &pretty(&manifest))?;
    let mut s = format!(
        "mix {} fraction={frac}: tokens={tot_tokens} documents={tot_docs} scored={tot_scored}\n",
        out.display()
    );
    for r in &root_rows {
        s += &format!(
            "  {}: tokens {}/{} documents {}/{}\n",
            r["name"].as_str().unwrap_or(""),
            r["tokens_taken"],
            r["tokens_total"],
            r["documents_taken"],
            r["documents_total"]
        );
    }
    Ok(s)
}

/// Rank rule top-1: highest count, ties to the lower id, zero counts excluded.
pub fn top1<T: Copy + Ord + Default>(counts: &[T]) -> Option<u16> {
    let mut best: Option<(usize, T)> = None;
    for (i, &c) in counts.iter().enumerate() {
        if c == T::default() {
            continue;
        }
        if best.is_none_or(|(_, b)| c > b) {
            best = Some((i, c));
        }
    }
    best.map(|(i, _)| i as u16)
}

/// TRAIN count tables: unigram and the previous-token bigram (T2), top-1 only.
pub struct Tables {
    pub unigram_top1: u16,
    pub bigram_top1: Vec<Option<u16>>,
    pub scored_targets: u64,
    pub bigram_keys_seen: u64,
}

impl Tables {
    pub fn build(tokens: &[u16], mask: &[u8]) -> R<Self> {
        if tokens.len() > u32::MAX as usize {
            return Err(format!(
                "train: {} tokens could overflow a u32 bigram count",
                tokens.len()
            ));
        }
        let mut uni = vec![0u64; VOCAB];
        let mut bi = vec![0u32; VOCAB * VOCAB];
        let mut scored = 0u64;
        for p in 1..tokens.len() {
            if !is_scored(tokens, mask, p) {
                continue;
            }
            let (q, t) = (tokens[p - 1] as usize, tokens[p] as usize);
            if q >= VOCAB || t >= VOCAB {
                return Err(format!("train position {p}: token id >= {VOCAB}"));
            }
            uni[t] += 1;
            bi[q * VOCAB + t] += 1;
            scored += 1;
        }
        let unigram_top1 = top1(&uni).ok_or("train: no scored positions")?;
        let bigram_top1: Vec<Option<u16>> = bi.chunks_exact(VOCAB).map(top1).collect();
        let bigram_keys_seen = bigram_top1.iter().filter(|t| t.is_some()).count() as u64;
        Ok(Tables {
            unigram_top1,
            bigram_top1,
            scored_targets: scored,
            bigram_keys_seen,
        })
    }
    /// T2 top-1 for previous token `q`; an unseen key falls back to the unigram top-1.
    pub fn t2(&self, q: u16) -> u16 {
        self.bigram_top1
            .get(q as usize)
            .copied()
            .flatten()
            .unwrap_or(self.unigram_top1)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RowsFile {
    split: String,
    memory: bool,
    positions: Vec<usize>,
    targets: Vec<u32>,
    nearest: Vec<u32>,
    writer: Vec<u32>,
}

#[derive(Default, Clone, Copy)]
struct Counts {
    rows: u64,
    model: u64,
    writer: u64,
    unigram: u64,
    t2: u64,
    vs_uni: [u64; 2],
    vs_t2: [u64; 2],
}

impl Counts {
    fn add_row(&mut self, model: bool, writer: bool, uni: bool, t2: bool) {
        self.rows += 1;
        self.model += u64::from(model);
        self.writer += u64::from(writer);
        self.unigram += u64::from(uni);
        self.t2 += u64::from(t2);
        self.vs_uni[0] += u64::from(model && !uni);
        self.vs_uni[1] += u64::from(!model && uni);
        self.vs_t2[0] += u64::from(model && !t2);
        self.vs_t2[1] += u64::from(!model && t2);
    }
    fn add(&mut self, o: &Counts) {
        self.rows += o.rows;
        self.model += o.model;
        self.writer += o.writer;
        self.unigram += o.unigram;
        self.t2 += o.t2;
        for i in 0..2 {
            self.vs_uni[i] += o.vs_uni[i];
            self.vs_t2[i] += o.vs_t2[i];
        }
    }
    fn fill(&self, v: &mut Value) {
        v["rows"] = json!(self.rows);
        v["model_hits"] = json!(self.model);
        v["writer_hits"] = json!(self.writer);
        v["unigram_hits"] = json!(self.unigram);
        v["t2_hits"] = json!(self.t2);
        v["vs_unigram"] = json!({"b": self.vs_uni[0], "c": self.vs_uni[1]});
        v["vs_t2"] = json!({"b": self.vs_t2[0], "c": self.vs_t2[1]});
    }
}

/// One panel split with its rows, every row check passed.
struct Panel {
    name: String,
    split: Split,
    /// None for `rows=NAME:all`.
    rows_file_sha256: Option<String>,
    rows: RowsFile,
    /// SHA-256 over the split's `(position u64 LE, target u32 LE)` pairs.
    id: [u8; 32],
}

/// The inputs `grade` and `seen` share: `out`, TRAIN and the checked panels in order.
struct Inputs {
    out: PathBuf,
    train: Split,
    panels: Vec<Panel>,
}

/// `allow_all`: `rows=NAME:all` takes every scored panel position, target `tokens[p]`.
fn load_inputs(a: &Args, allow_all: bool) -> R<Inputs> {
    a.known(&["out", "train", "panel", "rows"])?;
    let out = PathBuf::from(a.one("out")?);
    refuse_existing(&out)?;
    let (tdir, tt, tm) = split_spec(a.one("train")?)?;
    let specs: Vec<_> = a
        .all("panel")
        .into_iter()
        .map(named_spec)
        .collect::<R<_>>()?;
    if specs.is_empty() {
        return Err("at least one panel= is required".into());
    }
    let mut rows: Vec<(String, PathBuf)> = Vec::new();
    for spec in a.all("rows") {
        let (n, f) = spec
            .split_once(':')
            .ok_or(format!("rows={spec:?}: expected NAME:FILE"))?;
        if rows.iter().any(|(m, _)| m == n) {
            return Err(format!("rows {n}: given more than once"));
        }
        rows.push((n.to_string(), PathBuf::from(f)));
    }
    let mut names = BTreeSet::new();
    for (name, ..) in &specs {
        if !names.insert(name.as_str()) {
            return Err(format!("panel {name}: given more than once"));
        }
        if !rows.iter().any(|(n, _)| n == name) {
            return Err(format!("panel {name}: no rows file with that name"));
        }
    }
    if let Some((n, _)) = rows.iter().find(|(n, _)| !names.contains(n.as_str())) {
        return Err(format!("rows {n}: no panel with that name"));
    }
    let train = load_split(&tdir, &tt, &tm, false)?;
    let mut panels = Vec::new();
    for (name, dir, t, m) in &specs {
        let p = load_split(dir, t, m, false)?;
        let (_, rp) = rows.iter().find(|(n, _)| n == name).expect("matched above");
        let (rf, rows_file_sha256) = if allow_all && rp.as_path() == Path::new("all") {
            let positions: Vec<usize> = (1..p.tokens.len())
                .filter(|&q| is_scored(&p.tokens, &p.mask, q))
                .collect();
            let targets = positions.iter().map(|&q| u32::from(p.tokens[q])).collect();
            let rf = RowsFile {
                split: name.clone(),
                memory: false,
                positions,
                targets,
                nearest: Vec::new(),
                writer: Vec::new(),
            };
            (rf, None)
        } else {
            if rp.as_path() == Path::new("all") {
                return Err(format!(
                    "rows={name}:all is for seen only; this command grades a rows file"
                ));
            }
            let bytes = read(rp)?;
            let rf: RowsFile = serde_json::from_slice(&bytes)
                .map_err(|e| format!("{}: not a rows file: {e}", rp.display()))?;
            check_rows(&rf, &p, name, rp)?;
            (rf, Some(sha_bytes(&bytes)))
        };
        let mut id = Sha256::new();
        for (&pos, &target) in rf.positions.iter().zip(&rf.targets) {
            id.update((pos as u64).to_le_bytes());
            id.update(target.to_le_bytes());
        }
        panels.push(Panel {
            name: name.clone(),
            split: p,
            rows_file_sha256,
            rows: rf,
            id: id.finalize().into(),
        });
    }
    Ok(Inputs { out, train, panels })
}

/// Every row check of a rows file against its panel.
fn check_rows(rf: &RowsFile, p: &Split, name: &str, rp: &Path) -> R<()> {
    if rf.split != *name {
        return Err(format!(
            "{}: split {:?} != rows name {name}",
            rp.display(),
            rf.split
        ));
    }
    let k = rf.positions.len();
    if [rf.targets.len(), rf.nearest.len(), rf.writer.len()] != [k; 3] {
        return Err(format!(
            "{}: split {name}: unequal list lengths",
            rp.display()
        ));
    }
    let mut used = vec![false; p.tokens.len()];
    for i in 0..k {
        let (pos, target) = (rf.positions[i], rf.targets[i]);
        let bad = |why: String| format!("split {name} row {i}: position {pos}: {why}");
        if pos == 0 {
            return Err(bad("position 0 is never scored".into()));
        }
        if pos >= p.tokens.len() {
            return Err(bad(format!("beyond panel length {}", p.tokens.len())));
        }
        for (what, v) in [
            ("target", target),
            ("nearest", rf.nearest[i]),
            ("writer", rf.writer[i]),
        ] {
            if v as usize >= VOCAB {
                return Err(bad(format!("{what} id {v} >= {VOCAB}")));
            }
        }
        if u32::from(p.tokens[pos]) != target {
            return Err(bad(format!(
                "panel token {} != target {target}",
                p.tokens[pos]
            )));
        }
        if p.tokens[pos] == 0 {
            return Err(bad("target is BOS".into()));
        }
        if p.mask[pos] != 1 {
            return Err(bad("mask is 0".into()));
        }
        if std::mem::replace(&mut used[pos], true) {
            return Err(bad("duplicate position".into()));
        }
    }
    Ok(())
}

/// The pooled row identity: SHA-256 over, per split in order, the name length (u64 LE), the
/// name bytes and the split's 32-byte identity.
fn pooled_identity(panels: &[Panel]) -> String {
    let mut h = Sha256::new();
    for p in panels {
        h.update((p.name.len() as u64).to_le_bytes());
        h.update(p.name.as_bytes());
        h.update(p.id);
    }
    hex::encode(h.finalize())
}

/// One bit per row in pooled row order, LSB-first, zero-padded to whole bytes.
fn pack_bits(bits: &[bool]) -> Vec<u8> {
    let mut v = vec![0u8; bits.len().div_ceil(8)];
    for (i, &b) in bits.iter().enumerate() {
        v[i / 8] |= u8::from(b) << (i % 8);
    }
    v
}

fn run_grade(a: &Args) -> R<String> {
    let inp = load_inputs(a, false)?;
    let tables = Tables::build(&inp.train.tokens, &inp.train.mask)?;
    let mut splits = Vec::new();
    let mut pooled = Counts::default();
    let (mut near_bits, mut t2_bits, mut uni_bits) = (Vec::new(), Vec::new(), Vec::new());
    for pn in &inp.panels {
        let (p, rf, name) = (&pn.split, &pn.rows, &pn.name);
        let mut c = Counts::default();
        for i in 0..rf.positions.len() {
            let (pos, target) = (rf.positions[i], rf.targets[i]);
            let model = rf.nearest[i] == target;
            let uni = u32::from(tables.unigram_top1) == target;
            let t2 = u32::from(tables.t2(p.tokens[pos - 1])) == target;
            c.add_row(model, rf.writer[i] == target, uni, t2);
            near_bits.push(model);
            t2_bits.push(t2);
            uni_bits.push(uni);
        }
        pooled.add(&c);
        let mut v = json!({
            "name": name,
            "tokens_sha256": p.tokens_sha256,
            "mask_sha256": p.mask_sha256,
            "rows_file_sha256": pn.rows_file_sha256,
            "memory": rf.memory,
            "panel_scored_positions": count_scored(&p.tokens, &p.mask, p.tokens.len()),
            "row_identity_sha256": hex::encode(pn.id),
        });
        c.fill(&mut v);
        if let Some(mf) = &p.manifest {
            let mfp = p.dir.join("manifest.json");
            let get = |k: &str| {
                mf.get(k)
                    .and_then(Value::as_u64)
                    .ok_or(format!("{}: {k} is not an integer", mfp.display()))
            };
            let (cb, pt) = (get("content_bytes")?, get("tokens")?);
            if pt != p.tokens.len() as u64 {
                return Err(format!(
                    "{}: tokens {pt} != {} read",
                    mfp.display(),
                    p.tokens.len()
                ));
            }
            v["content_bytes"] = json!(cb);
            v["panel_tokens"] = json!(pt);
        }
        splits.push(v);
    }
    let (hits, t2h, unih) = (
        pack_bits(&near_bits),
        pack_bits(&t2_bits),
        pack_bits(&uni_bits),
    );
    let mut pv = json!({"row_identity_sha256": pooled_identity(&inp.panels)});
    pooled.fill(&mut pv);
    let grade = json!({
        "schema": GRADE_SCHEMA,
        "train": {
            "tokens_sha256": inp.train.tokens_sha256,
            "mask_sha256": inp.train.mask_sha256,
            "scored_targets": tables.scored_targets,
            "bigram_keys_seen": tables.bigram_keys_seen,
        },
        "unigram_top1": tables.unigram_top1,
        "splits": splits,
        "pooled": pv,
        "hits_file": HITS_FILE,
        "hits_sha256": sha_bytes(&hits),
        "t2_hits_file": T2_HITS_FILE,
        "t2_hits_sha256": sha_bytes(&t2h),
        "unigram_hits_file": UNIGRAM_HITS_FILE,
        "unigram_hits_sha256": sha_bytes(&unih),
    });
    let out = &inp.out;
    create_out_dir(out)?;
    write(&out.join(HITS_FILE), &hits)?;
    write(&out.join(T2_HITS_FILE), &t2h)?;
    write(&out.join(UNIGRAM_HITS_FILE), &unih)?;
    write(&out.join("grade.json"), &pretty(&grade))?;
    Ok(format!(
        "grade {}: rows={} model={} t2={} unigram={}\n",
        out.display(),
        pooled.rows,
        pooled.model,
        pooled.t2,
        pooled.unigram
    ))
}

/// Packs `NGRAM` ids (each < 4096, 12 bits) into one key, the first id in the high bits.
pub fn ngram_key(w: &[u16]) -> u128 {
    w.iter().fold(0u128, |k, &t| (k << 12) | u128::from(t))
}

/// Sorted distinct keys of every run `tokens[i..i + NGRAM]` inside one document: no BOS at
/// offsets `1..NGRAM` (a BOS at offset 0 opens the document). `tokens[0]` is BOS.
pub fn train_ngrams(tokens: &[u16]) -> R<Vec<u128>> {
    if let Some(p) = tokens.iter().position(|&t| t as usize >= VOCAB) {
        return Err(format!("train position {p}: token id >= {VOCAB}"));
    }
    let mut keys = Vec::with_capacity(tokens.len().saturating_sub(NGRAM - 1));
    let mut last_bos = 0usize;
    for j in 0..tokens.len() {
        if tokens[j] == 0 {
            last_bos = j;
        }
        if j + 1 >= NGRAM {
            let i = j + 1 - NGRAM;
            if last_bos <= i {
                keys.push(ngram_key(&tokens[i..=j]));
            }
        }
    }
    keys.sort_unstable();
    keys.dedup();
    Ok(keys)
}

fn seen_counts(mut v: Value, c: [u64; 3]) -> Value {
    v["rows"] = json!(c[0] + c[1] + c[2]);
    v["seen"] = json!(c[0]);
    v["short"] = json!(c[1]);
    v["unseen"] = json!(c[2]);
    v
}

/// Per row at `pos` in the document opened by BOS `s`: `short` if `pos - s < NGRAM`, else
/// `seen` iff `tokens[pos - NGRAM..pos]` is a TRAIN key, else `unseen`. Short or seen rows
/// get excluded bit 1. `pos - s < NGRAM` iff a BOS lies in `tokens[pos - NGRAM + 1..pos]`
/// (clipped at 0, where `tokens[0]` is BOS), so only `NGRAM - 1` tokens are read per row.
fn run_seen(a: &Args) -> R<String> {
    const SEEN: usize = 0;
    const SHORT: usize = 1;
    const UNSEEN: usize = 2;
    let inp = load_inputs(a, true)?;
    let keys = train_ngrams(&inp.train.tokens)?;
    let mut bits = Vec::new();
    let mut splits = Vec::new();
    let mut pooled = [0u64; 3];
    for pn in &inp.panels {
        let t = &pn.split.tokens;
        let mut c = [0u64; 3];
        for (i, &pos) in pn.rows.positions.iter().enumerate() {
            let class = if t[pos.saturating_sub(NGRAM - 1)..pos].contains(&0) {
                SHORT
            } else {
                let w = &t[pos - NGRAM..pos];
                if let Some(x) = w.iter().find(|&&x| x as usize >= VOCAB) {
                    return Err(format!(
                        "split {} row {i}: position {pos}: context id {x} >= {VOCAB}",
                        pn.name
                    ));
                }
                if keys.binary_search(&ngram_key(w)).is_ok() {
                    SEEN
                } else {
                    UNSEEN
                }
            };
            c[class] += 1;
            bits.push(class != UNSEEN);
        }
        for (p, x) in pooled.iter_mut().zip(c) {
            *p += x;
        }
        let v = json!({
            "name": pn.name,
            "rows_file_sha256": pn.rows_file_sha256,
            "row_identity_sha256": hex::encode(pn.id),
        });
        splits.push(seen_counts(v, c));
    }
    let excluded = pack_bits(&bits);
    let pv = json!({"row_identity_sha256": pooled_identity(&inp.panels)});
    let doc = json!({
        "schema": SEEN_SCHEMA,
        "ngram": NGRAM,
        "train": {
            "tokens_sha256": inp.train.tokens_sha256,
            "mask_sha256": inp.train.mask_sha256,
        },
        "train_ngrams_distinct": keys.len() as u64,
        "splits": splits,
        "pooled": seen_counts(pv, pooled),
        "excluded_file": EXCLUDED_FILE,
        "excluded_sha256": sha_bytes(&excluded),
    });
    create_out_dir(&inp.out)?;
    write(&inp.out.join(EXCLUDED_FILE), &excluded)?;
    write(&inp.out.join("seen.json"), &pretty(&doc))?;
    let mut s = format!(
        "seen {}: train_ngrams_distinct={} rows={} seen={} short={} unseen={}\n",
        inp.out.display(),
        keys.len(),
        bits.len(),
        pooled[SEEN],
        pooled[SHORT],
        pooled[UNSEEN]
    );
    for v in &splits {
        s += &format!(
            "  {}: rows {} seen {} short {} unseen {}\n",
            v["name"].as_str().unwrap_or(""),
            v["rows"],
            v["seen"],
            v["short"],
            v["unseen"]
        );
    }
    Ok(s)
}

/// `⌈2·√n⌉`: the smallest `k >= 0` with `k * k >= 4n`.
pub fn mcnemar_floor(n: u64) -> u64 {
    let target = n.saturating_mul(4);
    let mut k = 0u64;
    while k.saturating_mul(k) < target {
        k += 1;
    }
    k
}

/// One seed's paired reading.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seed {
    pub b: u64,
    pub c: u64,
    pub d: i64,
    pub band: u64,
    pub position: &'static str,
}

/// The band rule over two or three seeds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assessment {
    pub seeds: Vec<Seed>,
    pub spread: u64,
    pub verdict: &'static str,
}

/// `band(s) = max(⌈2·√(b(s)+c(s))⌉, max_s d − min_s d)` over every seed given; ahead iff
/// `d(s) > band(s)`, behind iff `d(s) < −band(s)`; the verdict needs every seed to agree, else
/// `mixed`.
pub fn assess(b: impl AsRef<[u64]>, c: impl AsRef<[u64]>) -> Assessment {
    let (b, c) = (b.as_ref(), c.as_ref());
    assert!(b.len() == c.len() && !b.is_empty(), "one (b, c) per seed");
    let d: Vec<i64> = b
        .iter()
        .zip(c)
        .map(|(&x, &y)| x as i64 - y as i64)
        .collect();
    let spread = (d.iter().max().unwrap() - d.iter().min().unwrap()) as u64;
    let seed = |s: usize| {
        let band = mcnemar_floor(b[s] + c[s]).max(spread);
        let position = if d[s] > band as i64 {
            "ahead"
        } else if d[s] < -(band as i64) {
            "behind"
        } else {
            "inside"
        };
        Seed {
            b: b[s],
            c: c[s],
            d: d[s],
            band,
            position,
        }
    };
    let seeds: Vec<Seed> = (0..b.len()).map(seed).collect();
    let verdict = if seeds.iter().all(|s| s.position == seeds[0].position) {
        seeds[0].position
    } else {
        "mixed"
    };
    Assessment {
        seeds,
        spread,
        verdict,
    }
}

fn assessment_json(a: &Assessment) -> Value {
    let seeds: Vec<Value> = a
        .seeds
        .iter()
        .map(|s| json!({"b": s.b, "c": s.c, "d": s.d, "band": s.band, "position": s.position}))
        .collect();
    json!({"seeds": seeds, "spread": a.spread, "verdict": a.verdict})
}

/// Row `i` of an LSB-first hit bitset.
pub fn hit(bits: &[u8], i: usize) -> bool {
    bits[i / 8] >> (i % 8) & 1 == 1
}

/// `(b, c)` over rows `[start, start + len)`: X right & Y wrong, X wrong & Y right.
pub fn discordant(x: &[u8], y: &[u8], start: usize, len: usize) -> (u64, u64) {
    discordant_where(x, y, start, len, None)
}

/// [`discordant`] over only the rows whose `excluded` bit is 0 (every row if `None`).
pub fn discordant_where(
    x: &[u8],
    y: &[u8],
    start: usize,
    len: usize,
    excluded: Option<&[u8]>,
) -> (u64, u64) {
    let (mut b, mut c) = (0u64, 0u64);
    for i in start..start + len {
        if excluded.is_some_and(|e| hit(e, i)) {
            continue;
        }
        match (hit(x, i), hit(y, i)) {
            (true, false) => b += 1,
            (false, true) => c += 1,
            _ => {}
        }
    }
    (b, c)
}

struct Grade {
    dir: PathBuf,
    sha256: String,
    v: Value,
    hits: Vec<u8>,
    /// `t2-hits.bin` and `unigram-hits.bin`; None in a grade written before them.
    t2: Option<Vec<u8>>,
    unigram: Option<Vec<u8>>,
    /// `(name, row_identity_sha256, rows)` per split, in order.
    splits: Vec<(String, String, u64)>,
}

/// A side bitset recorded as `{key}_file` / `{key}_sha256`, or None if the grade lacks it.
fn side_bits(dir: &Path, v: &Value, key: &str, file: &str, len: usize) -> R<Option<Vec<u8>>> {
    let gp = dir.join("grade.json");
    match v.get(format!("{key}_file")) {
        None => Ok(None),
        Some(f) if f == file => {
            let p = dir.join(file);
            let bits = read(&p)?;
            let got = sha_bytes(&bits);
            if v[format!("{key}_sha256")].as_str() != Some(got.as_str()) {
                return Err(format!(
                    "{}: sha256 {got} != {key}_sha256 {}",
                    p.display(),
                    v[format!("{key}_sha256")]
                ));
            }
            if bits.len() != len {
                return Err(format!("{}: {} bytes, not {len}", p.display(), bits.len()));
            }
            Ok(Some(bits))
        }
        Some(f) => Err(format!("{}: {key}_file {f} is not {file}", gp.display())),
    }
}

fn load_grade(dir: &Path) -> R<Grade> {
    let gp = dir.join("grade.json");
    let bytes = read(&gp)?;
    let v: Value = serde_json::from_slice(&bytes)
        .map_err(|e| format!("{}: unreadable json: {e}", gp.display()))?;
    let bad = |what: &str| format!("{}: {what}", gp.display());
    if v["schema"] != GRADE_SCHEMA || v["hits_file"] != HITS_FILE {
        return Err(bad("not a scale-probe grade"));
    }
    let hp = dir.join(HITS_FILE);
    let hits = read(&hp)?;
    let got = sha_bytes(&hits);
    if v["hits_sha256"].as_str() != Some(got.as_str()) {
        return Err(format!(
            "{}: sha256 {got} != hits_sha256 {}",
            hp.display(),
            v["hits_sha256"]
        ));
    }
    let mut splits = Vec::new();
    for s in v["splits"]
        .as_array()
        .ok_or_else(|| bad("splits is not a list"))?
    {
        let name = s["name"]
            .as_str()
            .ok_or_else(|| bad("split without a name"))?;
        let id = s["row_identity_sha256"]
            .as_str()
            .ok_or_else(|| bad("split without identity"))?;
        let rows = s["rows"]
            .as_u64()
            .ok_or_else(|| bad("split without rows"))?;
        splits.push((name.to_string(), id.to_string(), rows));
    }
    let total: u64 = splits.iter().map(|s| s.2).sum();
    if v["pooled"]["rows"].as_u64() != Some(total) || hits.len() as u64 != total.div_ceil(8) {
        return Err(format!(
            "{}: length disagrees with {total} rows",
            hp.display()
        ));
    }
    let len = total.div_ceil(8) as usize;
    let t2 = side_bits(dir, &v, "t2_hits", T2_HITS_FILE, len)?;
    let unigram = side_bits(dir, &v, "unigram_hits", UNIGRAM_HITS_FILE, len)?;
    let sha256 = sha_bytes(&bytes);
    Ok(Grade {
        dir: dir.to_path_buf(),
        sha256,
        v,
        hits,
        t2,
        unigram,
        splits,
    })
}

struct SeenDir {
    dir: PathBuf,
    sha256: String,
    v: Value,
    excluded: Vec<u8>,
}

fn load_seen(dir: &Path) -> R<SeenDir> {
    let sp = dir.join("seen.json");
    let bytes = read(&sp)?;
    let v: Value = serde_json::from_slice(&bytes)
        .map_err(|e| format!("{}: unreadable json: {e}", sp.display()))?;
    if v["schema"] != SEEN_SCHEMA || v["excluded_file"] != EXCLUDED_FILE {
        return Err(format!("{}: not a scale-probe seen", sp.display()));
    }
    let ep = dir.join(EXCLUDED_FILE);
    let excluded = read(&ep)?;
    let got = sha_bytes(&excluded);
    if v["excluded_sha256"].as_str() != Some(got.as_str()) {
        return Err(format!(
            "{}: sha256 {got} != excluded_sha256 {}",
            ep.display(),
            v["excluded_sha256"]
        ));
    }
    Ok(SeenDir {
        dir: dir.to_path_buf(),
        sha256: sha_bytes(&bytes),
        v,
        excluded,
    })
}

fn run_pair(a: &Args) -> R<String> {
    a.known(&["out", "a1", "a2", "a3", "b1", "b2", "b3", "table", "clean"])?;
    let out = PathBuf::from(a.one("out")?);
    refuse_existing(&out)?;
    let (b1, b2, table) = (a.opt("b1")?, a.opt("b2")?, a.opt("table")?);
    let (a3, b3) = (a.opt("a3")?, a.opt("b3")?);
    let table = match (b1, b2, table) {
        (Some(_), Some(_), None) => None,
        (None, None, Some(t @ ("t2" | "unigram"))) => Some(t),
        (None, None, Some(t)) => return Err(format!("table={t}: expected t2 or unigram")),
        _ => return Err("give exactly one of b1= b2= (both) or table=".into()),
    };
    match (table, a3, b3) {
        (None, Some(_), None) => return Err("a3= needs b3=: both cells take a third seed".into()),
        (_, None, Some(_)) => return Err("b3= needs a3=: both cells take a third seed".into()),
        (Some(_), _, Some(_)) => return Err("b3= is for cell mode; a table has no seed".into()),
        _ => {}
    }
    // `k` seeds per cell: cell a's grades are g[0..k], cell b's (cell mode) g[k..2k].
    let k = if a3.is_some() { 3 } else { 2 };
    let mut names: Vec<&str> = vec![a.one("a1")?, a.one("a2")?];
    names.extend(a3);
    if let (Some(x), Some(y)) = (b1, b2) {
        names.extend([x, y]);
        names.extend(b3);
    }
    let g = names
        .iter()
        .map(|p| load_grade(Path::new(p)))
        .collect::<R<Vec<Grade>>>()?;
    for x in &g[1..] {
        if x.v["pooled"]["row_identity_sha256"] != g[0].v["pooled"]["row_identity_sha256"] {
            return Err(format!(
                "{}: pooled row_identity_sha256 differs from a1",
                x.dir.display()
            ));
        }
        let names = |y: &Grade| y.splits.iter().map(|s| s.0.clone()).collect::<Vec<_>>();
        if names(x) != names(&g[0]) {
            return Err(format!(
                "{}: split names/order differ from a1",
                x.dir.display()
            ));
        }
        if let Some((p, _)) = x.splits.iter().zip(&g[0].splits).find(|(p, q)| p.1 != q.1) {
            return Err(format!(
                "{}: split {} row_identity_sha256 differs from a1",
                x.dir.display(),
                p.0
            ));
        }
        if let Some((p, q)) = x.splits.iter().zip(&g[0].splits).find(|(p, q)| p.2 != q.2) {
            return Err(format!(
                "{}: split {} holds {} rows, a1's {}",
                x.dir.display(),
                p.0,
                p.2,
                q.2
            ));
        }
    }
    let train = |y: &Grade| y.v["train"]["tokens_sha256"].clone();
    let same_train = (1..k)
        .map(|s| (0, s, "a"))
        .chain((1..k).map(|s| (k, k + s, "b")));
    for (p, q, cell) in same_train {
        if q < g.len() && train(&g[p]) != train(&g[q]) {
            return Err(format!(
                "{}: train.tokens_sha256 differs from {}; cell {cell}'s seeds share TRAIN",
                g[q].dir.display(),
                g[p].dir.display()
            ));
        }
    }
    let seen = a
        .opt("clean")?
        .map(|d| load_seen(Path::new(d)))
        .transpose()?;
    let total: u64 = g[0].splits.iter().map(|x| x.2).sum();
    if let Some(sn) = &seen {
        let bad = |what: &str| {
            format!(
                "{}: {what} differs from a1's grade",
                sn.dir.join("seen.json").display()
            )
        };
        if sn.v["train"]["tokens_sha256"] != g[0].v["train"]["tokens_sha256"] {
            return Err(bad("train.tokens_sha256"));
        }
        if sn.v["pooled"]["row_identity_sha256"] != g[0].v["pooled"]["row_identity_sha256"] {
            return Err(bad("pooled row_identity_sha256"));
        }
        let ids: Vec<(Option<&str>, Option<&str>)> = sn.v["splits"]
            .as_array()
            .map(|v| v.as_slice())
            .unwrap_or_default()
            .iter()
            .map(|s| (s["name"].as_str(), s["row_identity_sha256"].as_str()))
            .collect();
        let want: Vec<(Option<&str>, Option<&str>)> = g[0]
            .splits
            .iter()
            .map(|s| (Some(s.0.as_str()), Some(s.1.as_str())))
            .collect();
        if ids != want {
            return Err(bad("split names/order or row_identity_sha256"));
        }
        if sn.excluded.len() as u64 != total.div_ceil(8) {
            return Err(bad("excluded.bin length"));
        }
    }
    let mut y: Vec<&[u8]> = Vec::new();
    for s in 0..k {
        y.push(match table {
            None => &g[k + s].hits,
            Some(t) => {
                let bits = if t == "t2" { &g[s].t2 } else { &g[s].unigram };
                bits.as_deref().ok_or(format!(
                    "{}: grade has no {t} hits bitset; an older grade cannot pair against \
                     a table",
                    g[s].dir.display()
                ))?
            }
        });
    }
    let n = g[0].splits.len();
    let names: Vec<&str> = g[0].splits.iter().map(|x| x.0.as_str()).collect();
    let lens: Vec<u64> = g[0].splits.iter().map(|x| x.2).collect();
    let range = |split: Option<usize>| -> (usize, usize) {
        match split {
            Some(i) => (lens[..i].iter().sum::<u64>() as usize, lens[i] as usize),
            None => (0, total as usize),
        }
    };
    if let Some(k) = table.map(|t| if t == "t2" { "vs_t2" } else { "vs_unigram" }) {
        for (s, ys) in y.iter().enumerate() {
            for split in (0..n).map(Some).chain([None]) {
                let (start, len) = range(split);
                let (b, c) = discordant(&g[s].hits, ys, start, len);
                let (v, what) = match split {
                    Some(i) => (&g[s].v["splits"][i][k], names[i]),
                    None => (&g[s].v["pooled"][k], "pooled"),
                };
                if (v["b"].as_u64(), v["c"].as_u64()) != (Some(b), Some(c)) {
                    return Err(format!(
                        "{}: {what} {k} {v} disagrees with b={b} c={c} from the hit bitsets",
                        g[s].dir.join("grade.json").display()
                    ));
                }
            }
        }
    }
    let reading = |split: Option<usize>, excl: Option<&[u8]>| -> Assessment {
        let (start, len) = range(split);
        let (b, c): (Vec<u64>, Vec<u64>) = (0..k)
            .map(|s| discordant_where(&g[s].hits, y[s], start, len, excl))
            .unzip();
        assess(b, c)
    };
    let section = |excl: Option<&[u8]>| -> (Assessment, Value) {
        let pooled = reading(None, excl);
        let splits: Vec<Value> = (0..n)
            .map(|i| {
                let mut v = assessment_json(&reading(Some(i), excl));
                v["name"] = json!(names[i]);
                v
            })
            .collect();
        (pooled, json!(splits))
    };
    let (pooled, splits) = section(None);
    let mut inputs = json!({});
    for s in 0..k {
        inputs[format!("a{}", s + 1)] = json!(g[s].sha256);
        if table.is_none() {
            inputs[format!("b{}", s + 1)] = json!(g[k + s].sha256);
        }
    }
    if let Some(t) = table {
        inputs["table"] = json!(t);
    }
    let mut doc = json!({
        "schema": PAIR_SCHEMA,
        "pooled": assessment_json(&pooled),
        "splits": splits,
    });
    let mut summary = format!(
        "pair {}: pooled verdict {}\n",
        out.display(),
        pooled.verdict
    );
    if let Some(sn) = &seen {
        inputs["clean_seen_sha256"] = json!(sn.sha256);
        let (up, us) = section(Some(&sn.excluded));
        doc["unseen"] = json!({"pooled": assessment_json(&up), "splits": us});
        summary += &format!("  unseen rows: pooled verdict {}\n", up.verdict);
    }
    doc["inputs_grade_sha256"] = inputs;
    if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&out)
        .map_err(|e| format!("{}: {e}", out.display()))?;
    std::io::Write::write_all(&mut f, &pretty(&doc))
        .map_err(|e| format!("{}: {e}", out.display()))?;
    Ok(summary)
}
