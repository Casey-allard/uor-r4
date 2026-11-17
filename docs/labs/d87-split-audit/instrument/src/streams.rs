//! The sealed per-split token streams (claude-420).
//!
//! For each split this writes the UORT token stream, the response mask and the segment tags of
//! every kept record. Records run in the plan's order (shard paths sorted, then line order),
//! byte for byte what claude-403's `Ingest` writer makes from the same records. It also writes:
//! - for each DEV position, its novel-target class under claude-411's 13-gram rule;
//! - the per-id byte table that is the BPB denominator.
//!
//! TRAIN is written as per-shard parts in parallel, then joined in shard order, so it is never
//! held whole in memory.

use super::*;
use m2_student_corpus_ingest::VOCAB;
use std::fs::File;
use std::io::{BufWriter, Write};

/// A DEV target whose 13 preceding tokens (one record) occur as a 13-gram in TRAIN.
pub const SEEN: u8 = 0;
/// A DEV target whose 13 preceding tokens do not occur in TRAIN.
pub const NOVEL: u8 = 1;
/// A DEV target with fewer than 13 tokens before it in its record.
pub const SHORT: u8 = 2;
/// A BOS position, never a target.
pub const FRAME: u8 = 3;
const CLASSES: [&str; 3] = ["seen", "novel", "short_context"];

/// The UORT header that `Ingest::finish` writes for a stream of `tokens` tokens.
pub fn uort_header(tokens: u64) -> [u8; UORT_HEADER] {
    let mut h = [0u8; UORT_HEADER];
    h[0..4].copy_from_slice(b"UORT");
    h[4..8].copy_from_slice(&1u32.to_le_bytes());
    h[8..16].copy_from_slice(&tokens.to_le_bytes());
    h[16..20].copy_from_slice(&VOCAB.to_le_bytes());
    h
}

/// The decoded UTF-8 length of every id below `VOCAB`. BOS, EOS and UNK are 0. An id past the
/// vocabulary decodes to nothing.
pub fn id_bytes(coder: &Coder) -> Result<Vec<u8>, String> {
    let special: Vec<u32> = ["<|bos|>", "<|eos|>", "<|unk|>"]
        .iter()
        .filter_map(|n| coder.tok.token_id(n))
        .collect();
    (0..VOCAB)
        .map(|id| {
            if special.contains(&id) {
                return Ok(0);
            }
            let n = coder.tok.decode_bytes(&[id]).len();
            u8::try_from(n).map_err(|_| format!("token {id} decodes to {n} bytes, over 255"))
        })
        .collect()
}

/// The bytes of a record's targets (every position after BOS). Refused unless they equal the
/// record's text bytes minus its dropped bytes.
pub(crate) fn target_bytes(t: &Tok, table: &[u8]) -> Result<u64, String> {
    let sum: u64 = t.tokens[1..]
        .iter()
        .map(|&id| table[id as usize] as u64)
        .sum();
    let text: u64 = t.seg_bytes.iter().sum();
    if sum + t.dropped_bytes != text {
        return Err(format!(
            "record {}: its targets decode to {sum} bytes, but its text holds {text} bytes with {} dropped",
            t.id, t.dropped_bytes
        ));
    }
    Ok(sum)
}

/// The novel-target class of every position of one DEV record.
pub fn novel_of(t: &Tok, train_hits: &GramSet) -> Vec<u8> {
    (0..t.tokens.len())
        .map(|e| match e {
            0 => FRAME,
            e if e < GRAM => SHORT,
            e if train_hits.hit_of(&pack(&t.tokens[e - GRAM..e])) => SEEN,
            _ => NOVEL,
        })
        .collect()
}

/// Counts of one split. `class` holds, per class, [targets, bytes, response targets, response
/// bytes].
#[derive(Clone, Copy, Default)]
struct Sums {
    records: u64,
    tokens: u64,
    target_bytes: u64,
    response_targets: u64,
    response_bytes: u64,
    text_bytes: u64,
    dropped_bytes: u64,
    class: [[u64; 4]; 3],
}

impl Sums {
    fn add(&mut self, t: &Tok, table: &[u8], novel: Option<&[u8]>) -> Result<(), String> {
        self.target_bytes += target_bytes(t, table)?;
        self.records += 1;
        self.tokens += t.tokens.len() as u64;
        self.text_bytes += t.seg_bytes.iter().sum::<u64>();
        self.dropped_bytes += t.dropped_bytes;
        for e in 1..t.tokens.len() {
            let b = table[t.tokens[e] as usize] as u64;
            let r = (t.mask[e] == 1) as u64;
            self.response_targets += r;
            self.response_bytes += r * b;
            if let Some(nv) = novel {
                let c = self
                    .class
                    .get_mut(nv[e] as usize)
                    .ok_or(format!("record {}: position {e} is no target class", t.id))?;
                c[0] += 1;
                c[1] += b;
                c[2] += r;
                c[3] += r * b;
            }
        }
        Ok(())
    }

    fn merge(&mut self, o: &Sums) {
        self.records += o.records;
        self.tokens += o.tokens;
        self.target_bytes += o.target_bytes;
        self.response_targets += o.response_targets;
        self.response_bytes += o.response_bytes;
        self.text_bytes += o.text_bytes;
        self.dropped_bytes += o.dropped_bytes;
        for (a, b) in self.class.iter_mut().zip(&o.class) {
            for (x, y) in a.iter_mut().zip(b) {
                *x += y;
            }
        }
    }
}

/// A file written once, front to back, with its sha256 taken on the way.
struct Sink {
    name: String,
    w: BufWriter<File>,
    h: Sha256,
    n: u64,
}

impl Sink {
    fn create(dir: &Path, name: String) -> Result<Self, String> {
        let p = dir.join(&name);
        let f = File::create(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        Ok(Self {
            name,
            w: BufWriter::with_capacity(1 << 20, f),
            h: Sha256::new(),
            n: 0,
        })
    }

    fn put(&mut self, b: &[u8]) -> Result<(), String> {
        self.h.update(b);
        self.n += b.len() as u64;
        self.w
            .write_all(b)
            .map_err(|e| format!("{}: {e}", self.name))
    }

    fn done(self) -> Result<Value, String> {
        let Sink { name, mut w, h, n } = self;
        w.flush().map_err(|e| format!("{name}: {e}"))?;
        Ok(json!({"file": name, "bytes": n, "sha256": hex(&h.finalize())}))
    }
}

fn token_bytes(t: &Tok) -> Vec<u8> {
    t.tokens
        .iter()
        .flat_map(|&id| (id as u16).to_le_bytes())
        .collect()
}

/// Write one DEV split from its records in plan order.
fn write_dev(
    out: &Path,
    s: Split,
    toks: &[Tok],
    hits: &GramSet,
    table: &[u8],
) -> Result<(Sums, Vec<Value>), String> {
    let name = s.name();
    let mut sinks = Vec::new();
    for ext in ["tokens.u16", "mask.u8", "segment.u8", "novel.u8"] {
        sinks.push(Sink::create(out, format!("{name}.{ext}"))?);
    }
    let tokens: u64 = toks.iter().map(|t| t.tokens.len() as u64).sum();
    sinks[0].put(&uort_header(tokens))?;
    let mut sums = Sums::default();
    for t in toks {
        let nv = novel_of(t, hits);
        sums.add(t, table, Some(&nv))?;
        sinks[0].put(&token_bytes(t))?;
        sinks[1].put(&t.mask)?;
        sinks[2].put(&t.segments)?;
        sinks[3].put(&nv)?;
    }
    let files = sinks
        .into_iter()
        .map(Sink::done)
        .collect::<Result<_, _>>()?;
    Ok((sums, files))
}

/// The summary of one split.
fn split_json(
    s: Split,
    sums: &Sums,
    files: Vec<Value>,
    dev: bool,
    tokenizer_sha: &str,
    table_sha: &str,
) -> Value {
    let novel = dev.then(|| {
        let classes: serde_json::Map<String, Value> = CLASSES
            .iter()
            .zip(&sums.class)
            .map(|(&n, c)| {
                let [targets, bytes, response_targets, response_bytes] = *c;
                (n.to_string(), json!({"targets": targets, "bytes": bytes,
                    "response_targets": response_targets, "response_bytes": response_bytes}))
            })
            .collect();
        json!({
            "rule": "per position: 0 seen (the 13 preceding tokens of one record form a 13-gram in TRAIN), 1 novel (they do not), 2 short context (under 13 preceding tokens), 3 BOS (never a target); claude-411's rule",
            "classes": classes,
        })
    });
    json!({
        "unit": "claude-420", "split": s.name(),
        "order": "shard paths sorted, then line order (claude-411's plan); each record framed BOS ... EOS, never across records",
        "tokenizer_sha256": tokenizer_sha, "id_bytes_sha256": table_sha,
        "byte_rule": "a target's bytes are its id's decoded UTF-8 length (BOS, EOS, UNK: 0); for every record they sum to its text bytes minus its dropped bytes (bytes the vocabulary cannot emit), checked record by record",
        "records": sums.records, "tokens": sums.tokens, "targets": sums.tokens - sums.records,
        "target_bytes": sums.target_bytes, "response_targets": sums.response_targets,
        "response_bytes": sums.response_bytes, "text_bytes": sums.text_bytes,
        "dropped_bytes": sums.dropped_bytes, "files": files, "novel": novel,
    })
}

/// Write the three splits' streams, DEV novel masks, `id-bytes.u8` and one `<split>.json` each
/// into `out`. Returns the summaries in `SPLITS` order.
pub fn write_streams(
    plan: &Plan,
    coder: &Coder,
    threads: usize,
    bloom_log2: u32,
    out: &Path,
    tokenizer_sha: &str,
) -> Result<Vec<Value>, String> {
    if !(6..=34).contains(&bloom_log2) {
        return Err(format!("bloom_log2 {bloom_log2} is outside 6..=34"));
    }
    std::fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let canon = |p: &Path| {
        p.canonicalize()
            .map_err(|e| format!("{}: {e}", p.display()))
    };
    let out_canon = canon(out)?;
    for (p, _) in &plan.shards {
        if canon(p)?.starts_with(&out_canon) {
            return Err(format!("{} is inside --out", p.display()));
        }
    }
    // A fresh parts folder: a stale part from an earlier run would be joined or left behind.
    let parts = out.join("parts");
    if parts.exists() {
        return Err(format!(
            "{} exists; remove it, or use a new --out",
            parts.display()
        ));
    }
    std::fs::create_dir(&parts).map_err(|e| format!("{}: {e}", parts.display()))?;
    let table = id_bytes(coder)?;
    let table_sha = hex(&sha(&[&table]));
    std::fs::write(out.join("id-bytes.u8"), &table).map_err(|e| e.to_string())?;
    let paths: Vec<PathBuf> = plan.shards.iter().map(|(p, _)| p.clone()).collect();
    let index_of = |p: &Path| paths.iter().position(|q| q == p).unwrap();

    // Pass 1: DEV records kept whole (they are small), and their grams.
    let dev = par_shards(&paths, threads, |p| {
        let fate = &plan.fate[index_of(p)];
        let (mut i, mut grams, mut toks) = (0usize, [vec![], vec![]], [vec![], vec![]]);
        read_shard(p, &mut |line| {
            let f = fate[i];
            i += 1;
            let k = match f {
                Fate::Keep(Split::DevTune) => 0,
                Fate::Keep(Split::DevTest) => 1,
                _ => return Ok(()),
            };
            let t = tokenize(coder, line)?;
            each_gram(&t, |g, s| grams[k].push((g, s)));
            toks[k].push(t);
            Ok(())
        })?;
        Ok((grams, toks))
    })?;
    let (mut tune, mut test, mut dev_toks) = (vec![], vec![], [vec![], vec![]]);
    for ([gt, gs], [tt, ts]) in dev {
        tune.extend(gt);
        test.extend(gs);
        dev_toks[0].extend(tt);
        dev_toks[1].extend(ts);
    }
    let sets = [GramSet::new(tune), GramSet::new(test)];
    let mut bloom = vec![0u64; 1usize << bloom_log2.saturating_sub(6)];
    let shift = 64 - bloom_log2;
    for set in &sets {
        for g in &set.uniq {
            let b = (mix(g) >> shift) as usize;
            bloom[b / 64] |= 1 << (b % 64);
        }
    }

    // Pass 2: TRAIN records into per-shard parts, marking DEV grams on the way.
    let train = par_shards(&paths, threads, |p| {
        let ix = index_of(p);
        let fate = &plan.fate[ix];
        let mut part = Vec::new();
        for ext in ["tokens", "mask", "segment"] {
            part.push(Sink::create(&parts, format!("train-{ix:05}.{ext}"))?);
        }
        let (mut i, mut sums) = (0usize, Sums::default());
        read_shard(p, &mut |line| {
            let f = fate[i];
            i += 1;
            if f != Fate::Keep(Split::Train) {
                return Ok(());
            }
            let t = tokenize(coder, line)?;
            sums.add(&t, &table, None)?;
            each_gram(&t, |g, _| {
                let b = (mix(&g) >> shift) as usize;
                if bloom[b / 64] >> (b % 64) & 1 == 1 {
                    sets[0].mark(&g);
                    sets[1].mark(&g);
                }
            });
            part[0].put(&token_bytes(&t))?;
            part[1].put(&t.mask)?;
            part[2].put(&t.segments)
        })?;
        for s in part {
            s.done()?;
        }
        Ok(sums)
    })?;
    let mut train_sums = Sums::default();
    for s in &train {
        train_sums.merge(s);
    }

    // Join TRAIN's parts in shard order.
    let mut files = vec![];
    for (ext, out_ext) in [
        ("tokens", "tokens.u16"),
        ("mask", "mask.u8"),
        ("segment", "segment.u8"),
    ] {
        let mut sink = Sink::create(out, format!("train.{out_ext}"))?;
        if ext == "tokens" {
            sink.put(&uort_header(train_sums.tokens))?;
        }
        for ix in 0..paths.len() {
            let p = parts.join(format!("train-{ix:05}.{ext}"));
            sink.put(&std::fs::read(&p).map_err(|e| format!("{}: {e}", p.display()))?)?;
            std::fs::remove_file(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        }
        files.push(sink.done()?);
    }
    std::fs::remove_dir(&parts).map_err(|e| format!("{}: {e}", parts.display()))?;

    let mut summaries = vec![split_json(
        Split::Train,
        &train_sums,
        files,
        false,
        tokenizer_sha,
        &table_sha,
    )];
    for (k, s) in [Split::DevTune, Split::DevTest].into_iter().enumerate() {
        let (sums, files) = write_dev(out, s, &dev_toks[k], &sets[k], &table)?;
        summaries.push(split_json(s, &sums, files, true, tokenizer_sha, &table_sha));
    }
    for (s, v) in SPLITS.iter().zip(&summaries) {
        let text = serde_json::to_string_pretty(v).map_err(|e| e.to_string())? + "\n";
        std::fs::write(out.join(format!("{}.json", s.name())), text).map_err(|e| e.to_string())?;
    }
    Ok(summaries)
}
