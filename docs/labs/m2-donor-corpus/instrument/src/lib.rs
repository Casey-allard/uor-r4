//! The D87 production donor pass's client (claude-400): it sends the prompt bank to a
//! vLLM OpenAI-compatible server serving the donor, keeps the teacher's reasoning text
//! and final answer text per turn as two hard-text fields, writes resumable shards, and
//! seals and verifies the corpus. It holds no model: the donor runs in an external
//! engine. No logits, probabilities, scores or ranks are requested or stored.

pub mod bank;
pub mod prompts;

use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub type Res<T> = Result<T, String>;
pub type Args = BTreeMap<String, String>;

const NOT_STORED: &str = "no logits, probabilities, scores or ranks are requested or stored";
const RATE_EVERY: Duration = Duration::from_secs(60);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(3600);

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

/// DeepSeek chat-template markup left in a text (U+FF5C fullwidth-bar tokens, think
/// tags, DSML): the server's reasoning parse failed, so the text is not kept.
pub fn has_markup(text: &str) -> bool {
    ["<\u{ff5c}", "\u{ff5c}>", "<think>", "</think>", "DSML"]
        .iter()
        .any(|m| text.contains(m))
}

fn keys_are(o: &Map<String, Value>, want: &[&str], what: &str) -> Res<()> {
    let have: BTreeSet<&str> = o.keys().map(|k| k.as_str()).collect();
    let want: BTreeSet<&str> = want.iter().copied().collect();
    if have != want {
        return Err(format!("{what} keys {have:?}, want {want:?}"));
    }
    Ok(())
}

/// The user-message count a prompt kind carries.
pub fn users_for_kind(kind: &str) -> Res<usize> {
    match kind {
        "single" => Ok(1),
        "two-turn" => Ok(2),
        other => Err(format!("unknown kind {other}")),
    }
}

fn check_user_messages(msgs: &Value, kind: &str) -> Res<()> {
    let msgs = msgs.as_array().ok_or("messages is not an array")?;
    let users = users_for_kind(kind)?;
    if msgs.len() != users {
        return Err(format!("{} messages for kind {kind}", msgs.len()));
    }
    for msg in msgs {
        let o = msg.as_object().ok_or("message is not an object")?;
        keys_are(o, &["content", "role"], "message")?;
        if o["role"].as_str() != Some("user") || !o["content"].is_string() {
            return Err("message is not a user string message".into());
        }
    }
    Ok(())
}

/// The structural consumption check on one corpus record: exactly the keys `id`, `kind`,
/// `source`, `messages`, `turns`; one or two user messages matching the kind; one turn
/// per user message holding exactly `reasoning` and `answer` strings, the answer
/// non-empty after trim and neither text holding markup. Any other field (a logit,
/// probability, score or rank among them) fails.
pub fn check_record(v: &Value) -> Res<()> {
    let m = v.as_object().ok_or("record is not an object")?;
    keys_are(m, &["id", "kind", "messages", "source", "turns"], "record")?;
    for k in ["id", "kind", "source"] {
        if !m[k].is_string() {
            return Err(format!("{k} is not a string"));
        }
    }
    let kind = m["kind"].as_str().unwrap_or("");
    check_user_messages(&m["messages"], kind)?;
    let turns = m["turns"].as_array().ok_or("turns is not an array")?;
    if turns.len() != users_for_kind(kind)? {
        return Err(format!("{} turns for kind {kind}", turns.len()));
    }
    for t in turns {
        let o = t.as_object().ok_or("turn is not an object")?;
        keys_are(o, &["answer", "reasoning"], "turn")?;
        let (Some(a), Some(r)) = (o["answer"].as_str(), o["reasoning"].as_str()) else {
            return Err("turn answer or reasoning is not a string".into());
        };
        if a.trim().is_empty() {
            return Err("empty answer".into());
        }
        if has_markup(a) || has_markup(r) {
            return Err("a turn holds markup".into());
        }
    }
    Ok(())
}

/// One parsed chat-completion reply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reply {
    pub reasoning: String,
    pub answer: String,
    pub finish_reason: String,
    pub completion_tokens: u64,
    pub prompt_tokens: u64,
    /// The served model the response names (`model`), empty when absent.
    pub model: String,
    /// DeepSeek's `usage.prompt_cache_hit_tokens`, 0 when absent (vLLM sends none).
    pub cache_hit_tokens: u64,
}

/// `choices[0].message.content` is the answer; the reasoning is `reasoning_content`, else
/// `reasoning`, else empty. Texts are kept exactly as returned.
pub fn parse_reply(v: &Value) -> Reply {
    let c = &v["choices"][0];
    let msg = &c["message"];
    let s = |x: &Value| x.as_str().map(str::to_string);
    Reply {
        reasoning: s(&msg["reasoning_content"])
            .or_else(|| s(&msg["reasoning"]))
            .unwrap_or_default(),
        answer: s(&msg["content"]).unwrap_or_default(),
        finish_reason: s(&c["finish_reason"]).unwrap_or_default(),
        completion_tokens: v["usage"]["completion_tokens"].as_u64().unwrap_or(0),
        prompt_tokens: v["usage"]["prompt_tokens"].as_u64().unwrap_or(0),
        model: s(&v["model"]).unwrap_or_default(),
        cache_hit_tokens: v["usage"]["prompt_cache_hit_tokens"].as_u64().unwrap_or(0),
    }
}

impl Reply {
    /// `None` when the turn is kept, else the drop reason.
    pub fn drop_reason(&self) -> Option<&'static str> {
        if self.finish_reason != "stop" {
            Some("length")
        } else if self.answer.trim().is_empty() {
            Some("empty")
        } else if has_markup(&self.answer) || has_markup(&self.reasoning) {
            Some("markup")
        } else {
            None
        }
    }
}

/// `seed = (base + 2*i + t) mod 2^32`.
pub fn turn_seed(base: u64, i: u64, t: u64) -> u32 {
    (base.wrapping_add(i.wrapping_mul(2)).wrapping_add(t) & 0xffff_ffff) as u32
}

/// Which server dialect a segment talks: a vLLM server, or a hosted OpenAI-format API
/// (DeepSeek's), which takes no `seed` and no `chat_template_kwargs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Api {
    Vllm,
    Openai,
}

impl Api {
    pub fn name(self) -> &'static str {
        match self {
            Api::Vllm => "vllm",
            Api::Openai => "openai",
        }
    }
}

/// The request settings shared by every turn of a segment.
#[derive(Debug, Clone)]
pub struct Request {
    pub model_name: String,
    pub max_tokens: u64,
    pub effort: u64,
    pub base_seed: u64,
    pub api: Api,
    pub extra: Option<Value>,
}

/// Refuses an `--extra-body` that could ask the server for a score: any key, at any depth, whose
/// name contains `logprob`, `score` or `rank` (hard text only; no logits, probabilities, scores
/// or ranks are requested).
pub fn check_extra_body(v: &Value) -> Res<()> {
    match v {
        Value::Object(o) => {
            for (k, x) in o {
                let l = k.to_ascii_lowercase();
                if ["logprob", "score", "rank"].iter().any(|b| l.contains(b)) {
                    return Err(format!(
                        "--extra-body: key {k} could request a score; refused"
                    ));
                }
                check_extra_body(x)?;
            }
            Ok(())
        }
        Value::Array(a) => a.iter().try_for_each(check_extra_body),
        _ => Ok(()),
    }
}

/// The chat-completion body (JSON integers only); `extra` top-level keys override. In
/// `openai` mode the body has no `seed` and no `chat_template_kwargs`.
pub fn request_body(r: &Request, messages: &Value, seed: u32) -> Value {
    let mut b = match r.api {
        Api::Vllm => json!({"model": r.model_name, "messages": messages,
            "max_tokens": r.max_tokens, "temperature": 1, "top_p": 1, "seed": seed,
            "chat_template_kwargs": {"thinking": true, "reasoning_effort": r.effort},
            "stream": false}),
        Api::Openai => json!({"model": r.model_name, "messages": messages,
            "max_tokens": r.max_tokens, "temperature": 1, "top_p": 1, "stream": false}),
    };
    if let Some(Value::Object(x)) = &r.extra {
        for (k, v) in x {
            b[k] = v.clone();
        }
    }
    b
}

/// The raw text of a top-level key's value in a one-line JSON object, so a record can
/// copy the bank's `messages` byte-for-byte.
pub fn raw_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let b = line.as_bytes();
    let (mut depth, mut i, mut in_str, mut str_start) = (0usize, 0usize, false, 0usize);
    let mut last_key: Option<(usize, usize)> = None;
    let mut val_start: Option<usize> = None;
    while i < b.len() {
        let c = b[i];
        if in_str {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == b'"' {
                in_str = false;
                if depth == 1 && val_start.is_none() {
                    last_key = Some((str_start, i + 1));
                }
            }
            i += 1;
            continue;
        }
        match c {
            b'"' => {
                in_str = true;
                str_start = i;
            }
            b'{' | b'[' => depth += 1,
            b'}' | b']' => {
                if depth == 1 {
                    return val_start.map(|s| line[s..i].trim());
                }
                depth = depth.saturating_sub(1);
            }
            b':' if depth == 1 && val_start.is_none() => {
                if let Some((a, z)) = last_key {
                    if &line[a + 1..z - 1] == key {
                        val_start = Some(i + 1);
                    }
                }
            }
            b',' if depth == 1 => {
                if let Some(s) = val_start {
                    return Some(line[s..i].trim());
                }
                last_key = None;
            }
            _ => {}
        }
        i += 1;
    }
    None
}

// ---------------------------------------------------------------- files and helpers

fn need<'a>(m: &'a Args, k: &str) -> Res<&'a str> {
    m.get(k)
        .map(|s| s.as_str())
        .ok_or_else(|| format!("missing --{k}"))
}

fn num<T: std::str::FromStr>(m: &Args, k: &str, default: Option<T>) -> Res<T> {
    match (m.get(k), default) {
        (None, Some(d)) => Ok(d),
        (None, None) => Err(format!("missing --{k}")),
        (Some(s), _) => s.parse().map_err(|_| format!("--{k}: not a number: {s}")),
    }
}

fn read(p: &Path) -> Res<Vec<u8>> {
    fs::read(p).map_err(|e| format!("read {}: {e}", p.display()))
}

fn write(p: &Path, bytes: &[u8]) -> Res<()> {
    fs::write(p, bytes).map_err(|e| format!("write {}: {e}", p.display()))
}

fn read_json(p: &Path) -> Res<Value> {
    serde_json::from_slice(&read(p)?).map_err(|e| format!("parse {}: {e}", p.display()))
}

fn append(p: &Path) -> Res<File> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(p)
        .map_err(|e| format!("open {}: {e}", p.display()))
}

fn put_line(f: &mut File, line: &str, what: &str) -> Res<()> {
    f.write_all(format!("{line}\n").as_bytes())
        .and_then(|_| f.flush())
        .map_err(|e| format!("write {what}: {e}"))
}

/// The `\n`-terminated lines of a file (a torn last line is left out).
fn complete_lines(bytes: &[u8]) -> Vec<&[u8]> {
    match bytes.iter().rposition(|&b| b == b'\n') {
        None => vec![],
        Some(end) => bytes[..end].split(|&b| b == b'\n').collect(),
    }
}

fn line_json(line: &[u8]) -> Option<Value> {
    serde_json::from_slice(line).ok()
}

fn line_id(line: &[u8]) -> Option<String> {
    line_json(line)?["id"].as_str().map(str::to_string)
}

/// UTC time as `YYYY-MM-DDTHH:MM:SSZ` (civil-from-days, Hinnant).
pub fn utc_now() -> String {
    let s = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as i64;
    let (days, rem) = (s.div_euclid(86_400), s.rem_euclid(86_400));
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if mo <= 2 { 1 } else { 0 };
    format!(
        "{y:04}-{mo:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

fn shard_name(n: u64) -> String {
    format!("shard-{n:05}.jsonl")
}

fn counts_name(n: u64) -> String {
    format!("counts-{n:05}.jsonl")
}

/// Final shard numbers in the directory, ascending.
fn final_shards(dir: &Path) -> Res<Vec<u64>> {
    let mut v = vec![];
    for e in fs::read_dir(dir).map_err(|e| format!("list {}: {e}", dir.display()))? {
        let name = e.map_err(|e| format!("list: {e}"))?.file_name();
        let name = name.to_string_lossy();
        if let Some(n) = name
            .strip_prefix("shard-")
            .and_then(|s| s.strip_suffix(".jsonl"))
        {
            v.push(n.parse().map_err(|_| format!("bad shard name {name}"))?);
        }
    }
    v.sort_unstable();
    Ok(v)
}

fn part_files(dir: &Path) -> Res<Vec<String>> {
    let mut v = vec![];
    for e in fs::read_dir(dir).map_err(|e| format!("list {}: {e}", dir.display()))? {
        let name = e.map_err(|e| format!("list: {e}"))?.file_name();
        let name = name.to_string_lossy().to_string();
        if name.ends_with(".part") {
            v.push(name);
        }
    }
    v.sort();
    Ok(v)
}

fn sync_dir(dir: &Path) {
    if let Ok(d) = File::open(dir) {
        let _ = d.sync_all();
    }
}

/// Writes `lines` to `part`, fsyncs it and renames it to `fin`.
fn finalize(part: &Path, fin: &Path, lines: &[&[u8]]) -> Res<()> {
    let mut bytes = Vec::new();
    for l in lines {
        bytes.extend_from_slice(l);
        bytes.push(b'\n');
    }
    let mut f = File::create(part).map_err(|e| format!("create {}: {e}", part.display()))?;
    f.write_all(&bytes)
        .and_then(|_| f.sync_all())
        .map_err(|e| format!("write {}: {e}", part.display()))?;
    fs::rename(part, fin).map_err(|e| format!("rename {}: {e}", part.display()))
}

/// Finalizes every `.part` shard and counts pair: keeps complete lines only, cuts both to
/// their common prefix of matching ids, and renames both to final names. A pair with no
/// common line is removed. Running it twice changes nothing.
fn recover_parts(dir: &Path) -> Res<()> {
    let mut nums = BTreeSet::new();
    for name in part_files(dir)? {
        let stem = name.trim_end_matches(".jsonl.part");
        let n = stem
            .strip_prefix("shard-")
            .or_else(|| stem.strip_prefix("counts-"));
        if let Some(n) = n.and_then(|n| n.parse::<u64>().ok()) {
            nums.insert(n);
        }
    }
    for n in nums {
        let (sf, cf) = (dir.join(shard_name(n)), dir.join(counts_name(n)));
        let (sp, cp) = (
            dir.join(shard_name(n) + ".part"),
            dir.join(counts_name(n) + ".part"),
        );
        let src = |p: &Path, f: &Path| -> Res<Vec<u8>> {
            if p.exists() {
                read(p)
            } else if f.exists() {
                read(f)
            } else {
                Ok(vec![])
            }
        };
        let (sb, cb) = (src(&sp, &sf)?, src(&cp, &cf)?);
        let (sl, cl) = (complete_lines(&sb), complete_lines(&cb));
        let mut k = 0;
        while k < sl.len().min(cl.len()) {
            match (line_id(sl[k]), line_id(cl[k])) {
                (Some(a), Some(b)) if a == b => k += 1,
                _ => break,
            }
        }
        if k == 0 {
            for p in [&sp, &cp, &sf, &cf] {
                if p.exists() {
                    fs::remove_file(p).map_err(|e| format!("remove {}: {e}", p.display()))?;
                }
            }
            continue;
        }
        finalize(&sp, &sf, &sl[..k])?;
        finalize(&cp, &cf, &cl[..k])?;
    }
    sync_dir(dir);
    Ok(())
}

// ---------------------------------------------------------------- the prompt bank

/// One bank line.
pub struct BankItem {
    pub id: String,
    pub kind: String,
    pub source: String,
    pub users: Vec<Value>,
    pub raw_messages: String,
}

pub fn load_bank(bytes: &[u8]) -> Res<Vec<BankItem>> {
    let text = std::str::from_utf8(bytes).map_err(|e| format!("bank is not UTF-8: {e}"))?;
    let mut seen = HashSet::new();
    let mut out = vec![];
    for (n, line) in text
        .lines()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty())
    {
        let v: Value =
            serde_json::from_str(line).map_err(|e| format!("bank line {}: {e}", n + 1))?;
        let o = v
            .as_object()
            .ok_or(format!("bank line {} is not an object", n + 1))?;
        keys_are(o, &["id", "kind", "messages", "source"], "bank line")?;
        let s = |k: &str| {
            o[k].as_str()
                .map(str::to_string)
                .ok_or(format!("bank line {}: {k} is not a string", n + 1))
        };
        let (id, kind, source) = (s("id")?, s("kind")?, s("source")?);
        check_user_messages(&o["messages"], &kind).map_err(|e| format!("bank {id}: {e}"))?;
        if !seen.insert(id.clone()) {
            return Err(format!("bank id {id} repeats"));
        }
        let raw = raw_value(line, "messages").ok_or(format!("bank {id}: no raw messages"))?;
        out.push(BankItem {
            users: o["messages"].as_array().cloned().unwrap_or_default(),
            raw_messages: raw.to_string(),
            id,
            kind,
            source,
        });
    }
    Ok(out)
}

// ---------------------------------------------------------------- generate

#[derive(Default)]
struct Totals {
    kept: u64,
    generated: u64,
    records: u64,
    turns: u64,
    empty_reasoning: u64,
    by_kind: BTreeMap<String, u64>,
    by_source: BTreeMap<String, u64>,
    dropped: BTreeMap<String, u64>,
}

impl Totals {
    fn add_record(&mut self, rec: &Value) {
        self.records += 1;
        *self
            .by_kind
            .entry(rec["kind"].as_str().unwrap_or("").into())
            .or_default() += 1;
        *self
            .by_source
            .entry(rec["source"].as_str().unwrap_or("").into())
            .or_default() += 1;
        for t in rec["turns"].as_array().into_iter().flatten() {
            self.turns += 1;
            if t["reasoning"].as_str().unwrap_or("").is_empty() {
                self.empty_reasoning += 1;
            }
        }
    }
}

struct Out {
    dir: PathBuf,
    shard_records: usize,
    next_shard: u64,
    open: Option<(u64, File, File, usize)>,
    dropped: File,
    t: Totals,
}

impl Out {
    fn close_shard(&mut self) -> Res<()> {
        if let Some((n, mut s, mut c, _)) = self.open.take() {
            s.sync_all()
                .and_then(|_| c.sync_all())
                .map_err(|e| format!("fsync shard {n}: {e}"))?;
            drop((s.flush(), c.flush()));
            for name in [shard_name(n), counts_name(n)] {
                let fin = self.dir.join(&name);
                fs::rename(self.dir.join(name + ".part"), &fin)
                    .map_err(|e| format!("rename {}: {e}", fin.display()))?;
            }
            sync_dir(&self.dir);
        }
        Ok(())
    }

    fn keep(&mut self, line: &str, counts: &str, rec: &Value, kept: u64, gen: u64) -> Res<()> {
        if self.open.is_none() {
            let n = self.next_shard;
            self.next_shard += 1;
            let s = File::create(self.dir.join(shard_name(n) + ".part"))
                .map_err(|e| format!("create shard {n}: {e}"))?;
            let c = File::create(self.dir.join(counts_name(n) + ".part"))
                .map_err(|e| format!("create counts {n}: {e}"))?;
            self.open = Some((n, s, c, 0));
        }
        let full = {
            let (_, s, c, k) = self.open.as_mut().unwrap();
            put_line(s, line, "shard")?;
            put_line(c, counts, "counts")?;
            *k += 1;
            *k >= self.shard_records
        };
        self.t.kept += kept;
        self.t.generated += gen;
        self.t.add_record(rec);
        if full {
            self.close_shard()?;
        }
        Ok(())
    }

    fn drop_prompt(&mut self, id: &str, reason: &str, gen: u64) -> Res<()> {
        put_line(
            &mut self.dropped,
            &json!({"id": id, "reason": reason}).to_string(),
            "dropped",
        )?;
        self.t.generated += gen;
        *self.t.dropped.entry(reason.into()).or_default() += 1;
        Ok(())
    }
}

/// Reads a directory's finished work: finalizes `.part` pairs, then collects every done
/// id and restores the totals from the final shards, counts files and `dropped.jsonl`.
fn resume(dir: &Path) -> Res<(HashSet<String>, Totals, u64)> {
    recover_parts(dir)?;
    let mut done = HashSet::new();
    let mut t = Totals::default();
    let shards = final_shards(dir)?;
    for &n in &shards {
        let sb = read(&dir.join(shard_name(n)))?;
        let cb = read(&dir.join(counts_name(n)))?;
        let (sl, cl) = (complete_lines(&sb), complete_lines(&cb));
        if sl.len() != cl.len() {
            return Err(format!(
                "shard {n}: {} records, {} counts",
                sl.len(),
                cl.len()
            ));
        }
        for (s, c) in sl.iter().zip(&cl) {
            let rec = line_json(s).ok_or(format!("shard {n}: bad record line"))?;
            let cnt = line_json(c).ok_or(format!("counts {n}: bad line"))?;
            let id = rec["id"].as_str().unwrap_or("").to_string();
            if cnt["id"].as_str() != Some(id.as_str()) || !done.insert(id.clone()) {
                return Err(format!("shard {n}: id {id} mismatched or repeated"));
            }
            t.kept += cnt["kept_tokens"].as_u64().unwrap_or(0);
            t.generated += cnt["generated_tokens"].as_u64().unwrap_or(0);
            t.add_record(&rec);
        }
    }
    let dp = dir.join("dropped.jsonl");
    if dp.exists() {
        let b = read(&dp)?;
        let lines = complete_lines(&b);
        let mut kept_bytes = 0;
        // An `http` drop is the server's refusal, not the prompt's: a later segment retries it
        // once. A second `http` drop of the same id makes it done, so prompts a server always
        // refuses cannot fill every resume's first requests and trip the refused stop.
        let mut http_seen: HashMap<String, u32> = HashMap::new();
        // Each id counts once in the drop totals, and not at all if a shard kept it later.
        let mut counted: HashSet<String> = HashSet::new();
        for l in &lines {
            kept_bytes += l.len() + 1;
            let v = line_json(l).ok_or("dropped.jsonl: bad line")?;
            let id = v["id"].as_str().unwrap_or("").to_string();
            let reason = v["reason"].as_str().unwrap_or("");
            let kept_later = done.contains(&id) && !counted.contains(&id);
            if reason == "http" {
                let n = http_seen.entry(id.clone()).or_default();
                *n += 1;
                if *n >= 2 {
                    done.insert(id.clone());
                }
            } else {
                done.insert(id.clone());
            }
            if !kept_later && counted.insert(id) {
                *t.dropped.entry(reason.into()).or_default() += 1;
            }
        }
        if kept_bytes != b.len() {
            write(&dp, &b[..kept_bytes])?;
        }
    }
    let next = shards.last().map(|n| n + 1).unwrap_or(0);
    Ok((done, t, next))
}

enum Outcome {
    Kept {
        line: String,
        counts: String,
        rec: Value,
        kept: u64,
    },
    Dropped {
        reason: &'static str,
        generated: u64,
    },
    /// The server stayed down: the prompt is left undone (not dropped) for a later resume.
    Abandoned,
    /// 401, 402 or 403: the account was refused; the prompt is left undone.
    Refused,
}

/// Why a request failed after its retries.
enum Fail {
    /// The server refused this request (a 4xx other than 401-403 and 429): drop the prompt.
    Prompt,
    /// 401, 402 or 403: the key, the balance or the access, not the prompt.
    Refused,
    /// 429 for longer than the server wait.
    Limited,
    /// Connection error, 5xx or an unreadable body: the server, not the prompt.
    Server,
}

/// Consecutive finished prompts dropped as `http` that stop a segment as `refused`.
const HTTP_DROP_STOP: u64 = 32;

/// The API key, read once at start. It has no `Debug` and is never written anywhere but
/// the `Authorization` header.
struct Bearer(String);

/// Reads the key from `--api-key-env NAME` or `--api-key-file PATH`, trimmed of leading
/// and trailing whitespace (a file's trailing newline among it). Returns the header value and the segment's `auth` field.
/// Errors name only the variable or the path.
fn read_key(m: &Args) -> Res<(Option<Bearer>, Value)> {
    let (key, auth) = match (m.get("api-key-env"), m.get("api-key-file")) {
        (Some(_), Some(_)) => {
            return Err("--api-key-env and --api-key-file are mutually exclusive".into())
        }
        (None, None) => return Ok((None, Value::Null)),
        (Some(name), None) => (
            std::env::var(name).map_err(|_| format!("--api-key-env {name}: not set"))?,
            json!(format!("env:{name}")),
        ),
        (None, Some(p)) => {
            let b = fs::read(p).map_err(|_| format!("--api-key-file {p}: unreadable"))?;
            let s = String::from_utf8(b).map_err(|_| format!("--api-key-file {p}: not UTF-8"))?;
            (s, json!("file"))
        }
    };
    let key = key.trim();
    let what = m
        .get("api-key-env")
        .map(|n| format!("--api-key-env {n}"))
        .unwrap_or_else(|| format!("--api-key-file {}", m["api-key-file"]));
    if key.is_empty() || key.chars().any(char::is_whitespace) {
        return Err(format!("{what}: the key is empty or holds whitespace"));
    }
    Ok((Some(Bearer(format!("Bearer {key}"))), auth))
}

/// The 429 backoff in ms: `min(60, 2^k)` s plus a 0-999 ms jitter from the prompt index.
fn backoff_ms(k: u32, i: usize) -> u64 {
    let secs = 1u64.checked_shl(k).unwrap_or(u64::MAX).min(60);
    let jitter = (i as u64)
        .wrapping_mul(7919)
        .wrapping_add(u64::from(k) * 104_729)
        % 1000;
    secs * 1000 + jitter
}

struct Run<'a> {
    url: String,
    health: String,
    server_wait: Duration,
    req: Request,
    bearer: Option<Bearer>,
    http_drops: AtomicU64,
    bank: &'a [BankItem],
    done: &'a HashSet<String>,
    out: Mutex<Out>,
    next: AtomicUsize,
    stop: AtomicBool,
    stop_why: Mutex<Option<&'static str>>,
    finished: AtomicBool,
    in_flight: AtomicU64,
    errors: AtomicU64,
    ms: Mutex<Vec<u64>>,
    budget: u64,
}

impl Run<'_> {
    fn set_stop(&self, why: &'static str) {
        let mut w = self.stop_why.lock().unwrap();
        if w.is_none() {
            *w = Some(why);
        }
        self.stop.store(true, Ordering::SeqCst);
    }

    /// Four tries against a server error; a 429 is retried with backoff, apart from those
    /// tries, until the server wait has passed since the request's first 429.
    fn post(&self, agent: &ureq::Agent, body: &Value, i: usize) -> Result<Value, Fail> {
        let mut attempt = 0u64;
        let mut limited: Option<(Instant, u32)> = None;
        loop {
            let t = Instant::now();
            let mut rq = agent.post(&self.url);
            if let Some(b) = &self.bearer {
                rq = rq.header("Authorization", b.0.as_str());
            }
            let res = rq.send_json(body);
            self.errors.fetch_add(1, Ordering::SeqCst);
            match res {
                Ok(mut r) => {
                    if let Ok(v) = r.body_mut().read_json::<Value>() {
                        self.errors.fetch_sub(1, Ordering::SeqCst);
                        self.ms.lock().unwrap().push(t.elapsed().as_millis() as u64);
                        return Ok(v);
                    }
                }
                Err(ureq::Error::StatusCode(429)) => {
                    let (first, k) = limited.get_or_insert((Instant::now(), 0));
                    if first.elapsed() >= self.server_wait {
                        return Err(Fail::Limited);
                    }
                    std::thread::sleep(Duration::from_millis(backoff_ms(*k, i)));
                    *k += 1;
                    continue;
                }
                Err(ureq::Error::StatusCode(401..=403)) => return Err(Fail::Refused),
                Err(ureq::Error::StatusCode(400..=499)) => return Err(Fail::Prompt),
                Err(_) => {}
            }
            attempt += 1;
            if attempt >= 4 {
                return Err(Fail::Server);
            }
            std::thread::sleep(Duration::from_secs(2 * attempt));
        }
    }

    /// Polls `/v1/models` every 10 s until it answers, the wait runs out or the run stops.
    fn wait_healthy(&self) -> bool {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(10)))
            .build()
            .into();
        let start = Instant::now();
        loop {
            if self.stop.load(Ordering::SeqCst) {
                return false;
            }
            let mut rq = agent.get(&self.health);
            if let Some(b) = &self.bearer {
                rq = rq.header("Authorization", b.0.as_str());
            }
            if rq.call().is_ok() {
                return true;
            }
            if start.elapsed() >= self.server_wait {
                return false;
            }
            std::thread::sleep(Duration::from_secs(10).min(self.server_wait));
        }
    }

    fn prompt(&self, agent: &ureq::Agent, i: usize) -> Outcome {
        let item = &self.bank[i];
        let mut msgs: Vec<Value> = vec![];
        let mut turns: Vec<Value> = vec![];
        let (mut kept, mut generated, mut prompt_tokens) = (0u64, 0u64, 0u64);
        let (mut cache_hit, mut served) = (0u64, String::new());
        for (t, user) in item.users.iter().enumerate() {
            msgs.push(user.clone());
            let seed = turn_seed(self.req.base_seed, i as u64, t as u64);
            let body = request_body(&self.req, &Value::Array(msgs.clone()), seed);
            let mut waits = 0;
            let v = loop {
                match self.post(agent, &body, i) {
                    Ok(v) => break v,
                    Err(Fail::Prompt) => {
                        return Outcome::Dropped {
                            reason: "http",
                            generated,
                        }
                    }
                    Err(Fail::Refused) => return Outcome::Refused,
                    Err(Fail::Limited) => return Outcome::Abandoned,
                    Err(Fail::Server) if waits < 3 && self.wait_healthy() => waits += 1,
                    Err(Fail::Server) => return Outcome::Abandoned,
                }
            };
            let r = parse_reply(&v);
            generated += r.completion_tokens;
            if let Some(reason) = r.drop_reason() {
                return Outcome::Dropped { reason, generated };
            }
            kept += r.completion_tokens;
            prompt_tokens += r.prompt_tokens;
            cache_hit += r.cache_hit_tokens;
            served = r.model.clone();
            msgs.push(json!({"role": "assistant", "content": r.answer}));
            turns.push(json!({"reasoning": r.reasoning, "answer": r.answer}));
        }
        let js = |s: &str| Value::String(s.into()).to_string();
        let line = format!(
            "{{\"id\":{},\"kind\":{},\"source\":{},\"messages\":{},\"turns\":{}}}",
            js(&item.id),
            js(&item.kind),
            js(&item.source),
            item.raw_messages,
            Value::Array(turns)
        );
        let counts = json!({"id": item.id, "kept_tokens": kept, "generated_tokens": generated,
            "prompt_tokens": prompt_tokens, "cache_hit_tokens": cache_hit, "model": served})
        .to_string();
        let rec: Value = serde_json::from_str(&line).unwrap_or(Value::Null);
        Outcome::Kept {
            line,
            counts,
            rec,
            kept,
        }
    }

    fn worker(&self) -> Res<()> {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(REQUEST_TIMEOUT))
            .build()
            .into();
        let limit = self.bank.len();
        loop {
            if self.stop.load(Ordering::SeqCst) {
                return Ok(());
            }
            let i = self.next.fetch_add(1, Ordering::SeqCst);
            if i >= limit {
                return Ok(());
            }
            if self.done.contains(&self.bank[i].id) {
                continue;
            }
            self.in_flight.fetch_add(1, Ordering::SeqCst);
            let outcome = self.prompt(&agent, i);
            self.in_flight.fetch_sub(1, Ordering::SeqCst);
            let mut out = self.out.lock().unwrap();
            match outcome {
                Outcome::Kept {
                    line,
                    counts,
                    rec,
                    kept,
                } => {
                    check_record(&rec).map_err(|e| format!("{}: {e}", self.bank[i].id))?;
                    self.http_drops.store(0, Ordering::SeqCst);
                    out.keep(&line, &counts, &rec, kept, kept)?
                }
                Outcome::Dropped { reason, generated } => {
                    if reason != "http" {
                        self.http_drops.store(0, Ordering::SeqCst);
                    } else if self.http_drops.fetch_add(1, Ordering::SeqCst) + 1 >= HTTP_DROP_STOP {
                        self.set_stop("refused");
                    }
                    out.drop_prompt(&self.bank[i].id, reason, generated)?
                }
                Outcome::Abandoned => self.set_stop("server"),
                Outcome::Refused => self.set_stop("refused"),
            }
            if out.t.kept >= self.budget {
                self.set_stop("budget");
            }
        }
    }

    fn rate_line(&self, start: Instant, last: &mut (u64, u64)) -> Value {
        let out = self.out.lock().unwrap();
        let t = &out.t;
        let dropped: u64 = t.dropped.values().sum();
        let v = json!({"utc": utc_now(), "segment_elapsed_ms": start.elapsed().as_millis() as u64,
            "kept_tokens": t.kept, "generated_tokens": t.generated, "records": t.records,
            "dropped": dropped, "in_flight": self.in_flight.load(Ordering::SeqCst),
            "errors": self.errors.load(Ordering::SeqCst),
            "window_kept_tokens": t.kept - last.0, "window_generated_tokens": t.generated - last.1});
        *last = (t.kept, t.generated);
        v
    }
}

fn pct(sorted: &[u64], p: usize) -> u64 {
    if sorted.is_empty() {
        0
    } else {
        sorted[(sorted.len() - 1) * p / 100]
    }
}

/// `generate`: one segment of the resumable run. Returns the segment line it appended.
pub fn cmd_generate(m: &Args) -> Res<Value> {
    let server = need(m, "server")?.trim_end_matches('/').to_string();
    let prompts_path = PathBuf::from(need(m, "prompts")?);
    let dir = PathBuf::from(need(m, "out")?);
    let budget: u64 = num(m, "budget", None)?;
    let commit = need(m, "instrument-commit")?.to_string();
    let parallel: usize = num(m, "parallel", Some(256))?;
    let max_seconds: Option<u64> = m
        .contains_key("max-seconds")
        .then(|| num(m, "max-seconds", Some(0)))
        .transpose()?;
    let shard_records: usize = num(m, "shard-records", Some(2048))?;
    let limit: usize = num(m, "limit", Some(usize::MAX))?;
    let extra = match m.get("extra-body") {
        None => None,
        Some(p) => {
            let v = read_json(Path::new(p))?;
            if !v.is_object() {
                return Err("--extra-body must hold a JSON object".into());
            }
            check_extra_body(&v)?;
            Some(v)
        }
    };
    let api = match m.get("api").map(String::as_str) {
        None | Some("vllm") => Api::Vllm,
        Some("openai") => Api::Openai,
        Some(other) => return Err(format!("--api: want vllm or openai, got {other}")),
    };
    if api == Api::Openai && m.contains_key("seed") {
        return Err("--seed does not apply to --api openai (the body carries no seed)".into());
    }
    let (bearer, auth) = read_key(m)?;
    let req = Request {
        api,
        model_name: m
            .get("model-name")
            .cloned()
            .unwrap_or_else(|| "donor".into()),
        max_tokens: num(m, "max-tokens", Some(16384))?,
        effort: num(m, "effort", Some(50))?,
        base_seed: num(m, "seed", Some(20261116))?,
        extra,
    };
    if parallel == 0 || shard_records == 0 {
        return Err("--parallel and --shard-records must be positive".into());
    }
    let bank_bytes = read(&prompts_path)?;
    let mut bank = load_bank(&bank_bytes)?;
    bank.truncate(limit);
    fs::create_dir_all(&dir).map_err(|e| format!("mkdir {}: {e}", dir.display()))?;
    let utc_start = utc_now();
    let start = Instant::now();
    let (done, totals, next_shard) = resume(&dir)?;
    let run = Run {
        url: format!("{server}/v1/chat/completions"),
        health: format!("{server}/v1/models"),
        server_wait: Duration::from_secs(num(m, "server-wait-seconds", Some(1800))?),
        req: req.clone(),
        bearer,
        http_drops: AtomicU64::new(0),
        bank: &bank,
        done: &done,
        out: Mutex::new(Out {
            dropped: append(&dir.join("dropped.jsonl"))?,
            dir: dir.clone(),
            shard_records,
            next_shard,
            open: None,
            t: totals,
        }),
        next: AtomicUsize::new(0),
        stop: AtomicBool::new(false),
        stop_why: Mutex::new(None),
        finished: AtomicBool::new(false),
        in_flight: AtomicU64::new(0),
        errors: AtomicU64::new(0),
        ms: Mutex::new(vec![]),
        budget,
    };
    if run.out.lock().unwrap().t.kept >= budget {
        run.set_stop("budget");
    }
    let mut rate = append(&dir.join("rate.jsonl"))?;
    let mut last = {
        let t = &run.out.lock().unwrap().t;
        (t.kept, t.generated)
    };
    let results: Vec<Res<()>> = std::thread::scope(|s| {
        let handles: Vec<_> = (0..parallel).map(|_| s.spawn(|| run.worker())).collect();
        let monitor = s.spawn(|| -> Res<()> {
            let mut last_rate = Instant::now();
            while !run.finished.load(Ordering::SeqCst) {
                if max_seconds.is_some_and(|t| start.elapsed() >= Duration::from_secs(t)) {
                    run.set_stop("time");
                }
                if dir.join("STOP").exists() {
                    run.set_stop("stop-file");
                }
                if last_rate.elapsed() >= RATE_EVERY {
                    last_rate = Instant::now();
                    put_line(
                        &mut rate,
                        &run.rate_line(start, &mut last).to_string(),
                        "rate",
                    )?;
                }
                std::thread::sleep(Duration::from_millis(200));
            }
            Ok(())
        });
        let mut r: Vec<Res<()>> = handles
            .into_iter()
            .map(|h| {
                let x = h.join().unwrap_or(Err("worker panicked".into()));
                if x.is_err() {
                    run.set_stop("exhausted");
                }
                x
            })
            .collect();
        run.finished.store(true, Ordering::SeqCst);
        r.push(monitor.join().unwrap_or(Err("monitor panicked".into())));
        r
    });
    run.out.lock().unwrap().close_shard()?;
    for r in results {
        r?;
    }
    put_line(
        &mut rate,
        &run.rate_line(start, &mut last).to_string(),
        "rate",
    )?;
    let stopped_by = run.stop_why.lock().unwrap().unwrap_or("exhausted");
    let out = run.out.lock().unwrap();
    let t = &out.t;
    let mut ms = run.ms.lock().unwrap().clone();
    ms.sort_unstable();
    let metrics = json!({"records": t.records, "turns": t.turns, "kept_tokens": t.kept,
        "generated_tokens": t.generated, "empty_reasoning_turns": t.empty_reasoning,
        "records_by_kind": t.by_kind, "records_by_source": t.by_source,
        "dropped": t.dropped.values().sum::<u64>(), "dropped_by_reason": t.dropped,
        "segment": {"requests": ms.len(), "errors": run.errors.load(Ordering::SeqCst),
            "elapsed_ms": start.elapsed().as_millis() as u64, "request_ms_p50": pct(&ms, 50),
            "request_ms_p95": pct(&ms, 95), "request_ms_max": ms.last().copied().unwrap_or(0)},
        "not_stored": NOT_STORED});
    write(
        &dir.join("metrics.json"),
        serde_json::to_string_pretty(&metrics)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )?;
    let seg = json!({"utc_start": utc_start, "utc_end": utc_now(), "instrument_commit": commit,
        "prompts_sha256": sha256_hex(&bank_bytes), "server": server, "model_name": req.model_name,
        "parallel": parallel, "max_tokens": req.max_tokens,
        "effort": (api == Api::Vllm).then_some(req.effort),
        "api": api.name(), "auth": auth,
        "base_seed": (api == Api::Vllm).then_some(req.base_seed), "budget": budget, "max_seconds": max_seconds,
        "extra_body": req.extra, "stopped_by": stopped_by, "kept_tokens_at_end": t.kept,
        "generated_tokens_at_end": t.generated, "records_at_end": t.records});
    put_line(
        &mut append(&dir.join("segments.jsonl"))?,
        &seg.to_string(),
        "segments",
    )?;
    Ok(seg)
}

// ---------------------------------------------------------------- seal and verify

const SEALED_FILES: [&str; 4] = [
    "dropped.jsonl",
    "segments.jsonl",
    "rate.jsonl",
    "metrics.json",
];

/// The hashes `seal` writes and `verify` recomputes.
fn digest(dir: &Path, prompts: &Path) -> Res<Value> {
    let parts = part_files(dir)?;
    if !parts.is_empty() {
        return Err(format!(
            "unfinished files {parts:?}: run generate to finalize them"
        ));
    }
    let mut shards = vec![];
    let mut counts = vec![];
    let mut leaves = vec![];
    let mut records = 0usize;
    for n in final_shards(dir)? {
        let b = read(&dir.join(shard_name(n)))?;
        let k = complete_lines(&b).len();
        records += k;
        leaves.push(sha256_hex(&b));
        shards.push(json!({"file": shard_name(n), "sha256": sha256_hex(&b), "records": k}));
        let c = read(&dir.join(counts_name(n)))?;
        counts.push(json!({"file": counts_name(n), "sha256": sha256_hex(&c)}));
    }
    let mut files = Map::new();
    for f in SEALED_FILES {
        files.insert(f.into(), sha256_hex(&read(&dir.join(f))?).into());
    }
    Ok(
        json!({"shards": shards, "records": records, "merkle_root": merkle_root(&leaves)?,
        "counts": counts, "files": files, "prompts_sha256": sha256_hex(&read(prompts)?)}),
    )
}

/// `seal`: writes `DIR/receipt.json`.
pub fn cmd_seal(m: &Args) -> Res<Value> {
    let dir = PathBuf::from(need(m, "dir")?);
    let mut r = digest(&dir, Path::new(need(m, "prompts")?))?;
    let mut commits: Vec<String> = vec![];
    for l in complete_lines(&read(&dir.join("segments.jsonl"))?) {
        let c = line_json(l).ok_or("segments.jsonl: bad line")?["instrument_commit"]
            .as_str()
            .unwrap_or("")
            .to_string();
        if !commits.contains(&c) {
            commits.push(c);
        }
    }
    r["instrument_commits"] = json!(commits);
    for k in ["model", "machine", "engine"] {
        r[k] = read_json(Path::new(need(m, k)?))?;
    }
    r["not_stored"] = NOT_STORED.into();
    write(
        &dir.join("receipt.json"),
        serde_json::to_string_pretty(&r)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )?;
    Ok(json!({"records": r["records"], "merkle_root": r["merkle_root"]}))
}

/// `balance`: GETs `{server}/user/balance` with the key from `--api-key-env` or
/// `--api-key-file` and returns the provider's JSON as it came (it holds no secret).
/// The spend watcher reads it; the key is read the same way as `generate` reads it.
pub fn cmd_balance(m: &Args) -> Res<Value> {
    let server = need(m, "server")?.trim_end_matches('/').to_string();
    let (Some(bearer), _) = read_key(m)? else {
        return Err("balance needs --api-key-env or --api-key-file".into());
    };
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .build()
        .into();
    let mut resp = agent
        .get(format!("{server}/user/balance"))
        .header("Authorization", bearer.0.as_str())
        .call()
        .map_err(|e| format!("balance: {e}"))?;
    resp.body_mut()
        .read_json::<Value>()
        .map_err(|e| format!("balance: unreadable body: {e}"))
}

/// `verify`: recomputes every hash and the root, checks every record against the bank
/// and the counts files, and fails on any mismatch.
pub fn cmd_verify(m: &Args) -> Res<Value> {
    let dir = PathBuf::from(need(m, "dir")?);
    let prompts = PathBuf::from(need(m, "prompts")?);
    let receipt = read_json(&dir.join("receipt.json"))?;
    let now = digest(&dir, &prompts)?;
    for k in [
        "shards",
        "records",
        "merkle_root",
        "counts",
        "files",
        "prompts_sha256",
    ] {
        if receipt[k] != now[k] {
            return Err(format!("{k} differs from the receipt"));
        }
    }
    let bank = load_bank(&read(&prompts)?)?;
    let by_id: BTreeMap<&str, &BankItem> = bank.iter().map(|b| (b.id.as_str(), b)).collect();
    let mut seen = HashSet::new();
    let (mut records, mut turns, mut kept) = (0u64, 0u64, 0u64);
    for n in final_shards(&dir)? {
        let sb = read(&dir.join(shard_name(n)))?;
        let cb = read(&dir.join(counts_name(n)))?;
        let (sl, cl) = (complete_lines(&sb), complete_lines(&cb));
        if sl.len() != cl.len() {
            return Err(format!(
                "shard {n}: {} records, {} counts",
                sl.len(),
                cl.len()
            ));
        }
        for (s, c) in sl.iter().zip(&cl) {
            let rec = line_json(s).ok_or(format!("shard {n}: bad record line"))?;
            check_record(&rec).map_err(|e| format!("shard {n}: {e}"))?;
            let id = rec["id"].as_str().unwrap_or("");
            let b = by_id
                .get(id)
                .ok_or(format!("shard {n}: id {id} not in the bank"))?;
            let bank_msgs: Value =
                serde_json::from_str(&b.raw_messages).map_err(|e| e.to_string())?;
            if rec["messages"] != bank_msgs || rec["kind"] != b.kind.as_str() {
                return Err(format!("shard {n}: {id} differs from its bank line"));
            }
            if !seen.insert(id.to_string()) {
                return Err(format!("id {id} repeats"));
            }
            let cnt = line_json(c).ok_or(format!("counts {n}: bad line"))?;
            if cnt["id"].as_str() != Some(id) {
                return Err(format!("counts {n}: id differs from shard line {id}"));
            }
            kept += cnt["kept_tokens"]
                .as_u64()
                .ok_or(format!("counts {n}: kept_tokens"))?;
            records += 1;
            turns += rec["turns"].as_array().map(|t| t.len() as u64).unwrap_or(0);
        }
    }
    Ok(
        json!({"records": records, "turns": turns, "kept_tokens": kept,
        "root": now["merkle_root"], "verified": true}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn raw_value_is_the_exact_slice() {
        let line = r#"{"id": "x,}", "messages" : [ {"role":"user", "content":"a\"]}"} ], "kind":"single"}"#;
        assert_eq!(
            raw_value(line, "messages"),
            Some(r#"[ {"role":"user", "content":"a\"]}"} ]"#)
        );
        assert_eq!(raw_value(line, "kind"), Some(r#""single""#));
        assert_eq!(raw_value(line, "none"), None);
    }

    #[test]
    fn seeds_wrap_at_two_to_the_32() {
        assert_eq!(turn_seed(20261116, 3, 1), 20261123);
        assert_eq!(turn_seed(0xffff_ffff, 0, 1), 0);
    }

    #[test]
    fn resume_retries_http_drops_only() {
        let dir = std::env::temp_dir().join(format!("m2-resume-http-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("dropped.jsonl"),
            concat!(
                "{\"id\":\"a\",\"reason\":\"http\"}\n",
                "{\"id\":\"b\",\"reason\":\"length\"}\n",
                "{\"id\":\"c\",\"reason\":\"http\"}\n",
                "{\"id\":\"c\",\"reason\":\"http\"}\n",
                "{\"id\":\"d\",\"reason\":\"http\"}\n",
                "{\"id\":\"d\",\"reason\":\"empty\"}\n",
                "{\"id\":\"e\",\"reason\":\"http\"}\n",
            ),
        )
        .unwrap();
        // `e` was refused once and then kept on retry: a shard holds it.
        std::fs::write(
            dir.join(shard_name(0)),
            "{\"id\":\"e\",\"kind\":\"single\",\"source\":\"s\"}\n",
        )
        .unwrap();
        std::fs::write(
            dir.join(counts_name(0)),
            "{\"id\":\"e\",\"kept_tokens\":5,\"generated_tokens\":6}\n",
        )
        .unwrap();
        let (done, t, _) = resume(&dir).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(!done.contains("a"), "an http drop must be retried");
        assert!(done.contains("b"), "a length drop stays done");
        assert!(
            done.contains("c"),
            "a second http drop of one id makes it done"
        );
        assert!(
            done.contains("d"),
            "an http drop retried into a content drop is done"
        );
        assert!(done.contains("e") && t.records == 1 && t.kept == 5);
        // Each id counts once, under its first reason, and a kept id not at all.
        assert_eq!(t.dropped.get("http"), Some(&3));
        assert_eq!(t.dropped.get("length"), Some(&1));
        assert_eq!(t.dropped.get("empty"), None);
        assert_eq!(t.dropped.values().sum::<u64>(), 4);
    }

    #[test]
    fn complete_lines_drop_a_torn_tail() {
        assert_eq!(complete_lines(b"a\nb\nc"), vec![&b"a"[..], &b"b"[..]]);
        assert!(complete_lines(b"abc").is_empty());
    }
}
