//! Step 0b of the 2026-10-05 barrier assessment: a training-free tokenizer
//! and format audit of the D19 grounded-session draw (#820).
//!
//! ```text
//! step0b-key-tuple-audit out=NEW_REPORT_ROOT tokenizer=TOKENIZER.json \
//!   [seed=9101] [conversations=300] [train_seed=1] [train_conversations=5600] \
//!   [sessions=SESSION_REPORT.json[,...]]
//! ```
//!
//! It draws the same conversations as `m-world session world=v2` (development
//! split, default mix, `seed`, `conversations`) and, for every MQAR query,
//! relation query and copy turn, takes the token pieces of the key and value
//! where they occur: (A) in the assertion turn, (Q) in the question turn and
//! (R) in the expected reply. It also takes the value in the recall line the
//! log sieve would add (`Memory: {v}.`, a system turn). Each content is
//! tokenized exactly as `DialogueProtocol` encodes it: version 2 (the chat
//! emitters' protocol) as `" " + content`, version 1 as the content alone.
//! The m-tuple of an occurrence is its first m pieces (m = 1, 2, 3), and a
//! mismatch between two occurrences is classified as leading space, casing,
//! both, punctuation attachment or other (same text, different segmentation).
//!
//! A second draw of the TRAINING split (`train_seed`, `train_conversations`)
//! gives the training distribution's reply-form shares. It is the corpus
//! defaults' draw, not the exact corpus a model trained on.
//!
//! `sessions=` reads existing `uor-r4.m-world-session/1` reports: their user
//! texts must equal this draw (a check that the draw is the D19 draw), and
//! the forms of the model's actual MQAR replies are counted.
//!
//! Training-free and model-free. The decision rule (frozen before the run):
//! if more than 10% of the MQAR key/value tuples mismatch between the
//! assertion and the reply start, the copy boundary fix (Step 4) must precede
//! chat lineage verdicts.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::report_output;
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::milestone_world::Split;
use uor_r4_training::milestone_world_v2::{Category2, Conversation2, MWorld2, Mix, Turn2};
use uor_r4_training::stack_tracking::Rng;

const USAGE: &str = "usage: step0b-key-tuple-audit out=NEW_REPORT_ROOT tokenizer=TOKENIZER.json \
[seed=9101] [conversations=300] [train_seed=1] [train_conversations=5600] [sessions=A.json,...]";

/// The decision threshold of Step 0b, frozen in the assessment.
const THRESHOLD: f64 = 0.10;
const MS: [usize; 3] = [1, 2, 3];

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("step0b-key-tuple-audit: {error}");
            ExitCode::FAILURE
        }
    }
}

struct Args {
    out: PathBuf,
    tokenizer: PathBuf,
    seed: u64,
    conversations: usize,
    train_seed: u64,
    train_conversations: usize,
    sessions: Vec<PathBuf>,
}

fn parse_args() -> Result<Args, String> {
    let mut map: BTreeMap<String, String> = BTreeMap::new();
    for argument in std::env::args().skip(1) {
        let (key, value) = argument.split_once('=').ok_or(USAGE)?;
        if !matches!(
            key,
            "out"
                | "tokenizer"
                | "seed"
                | "conversations"
                | "train_seed"
                | "train_conversations"
                | "sessions"
        ) || map.insert(key.to_owned(), value.to_owned()).is_some()
        {
            return Err(format!("unknown or repeated argument {key}; {USAGE}"));
        }
    }
    let number = |key: &str, default: u64| -> Result<u64, String> {
        map.get(key)
            .map(|v| v.parse::<u64>().map_err(|e| format!("{key}={v}: {e}")))
            .unwrap_or(Ok(default))
    };
    Ok(Args {
        out: PathBuf::from(map.get("out").ok_or(USAGE)?),
        tokenizer: PathBuf::from(map.get("tokenizer").ok_or(USAGE)?),
        seed: number("seed", 9_101)?,
        conversations: number("conversations", 300)? as usize,
        train_seed: number("train_seed", 1)?,
        train_conversations: number("train_conversations", 5_600)? as usize,
        sessions: map
            .get("sessions")
            .map(|s| {
                s.split(',')
                    .filter(|p| !p.is_empty())
                    .map(PathBuf::from)
                    .collect()
            })
            .unwrap_or_default(),
    })
}

/// One occurrence of an entity (key or value) in a content text.
#[derive(Clone, Debug)]
struct Occurrence {
    ids: Vec<u32>,
    pieces: Vec<String>,
    /// The last piece runs past the entity (punctuation attached).
    right_spill: bool,
    /// The first piece starts before the entity with more than one space.
    left_spill: bool,
}

/// The pieces of `content[start..start + len]` as `DialogueProtocol`
/// tokenizes the content: version 2 (`spaced`) as " " + trimmed content,
/// version 1 as the trimmed content.
fn occurrence(
    tokenizer: &ByteBpeTokenizer,
    content: &str,
    spaced: bool,
    start: usize,
    len: usize,
) -> Result<Occurrence, String> {
    let trimmed = content.trim();
    let shift = content.len() - content.trim_start().len();
    if start < shift || start + len > shift + trimmed.len() {
        return Err(format!(
            "entity span outside the trimmed content: {content:?}"
        ));
    }
    let (text, s) = if spaced {
        (format!(" {trimmed}"), start - shift + 1)
    } else {
        (trimmed.to_owned(), start - shift)
    };
    let e = s + len;
    let ids = tokenizer.encode(&text);
    let mut at = 0usize;
    let mut out = Occurrence {
        ids: Vec::new(),
        pieces: Vec::new(),
        right_spill: false,
        left_spill: false,
    };
    for &id in &ids {
        let bytes = tokenizer.decode_bytes(&[id]);
        let (begin, end) = (at, at + bytes.len());
        at = end;
        if end <= s || begin >= e {
            continue;
        }
        if out.ids.is_empty() && begin < s {
            let before = &text.as_bytes()[begin..s];
            out.left_spill = !(before == b" ");
        }
        if end > e {
            out.right_spill = true;
        }
        out.ids.push(id);
        out.pieces
            .push(String::from_utf8_lossy(&bytes).into_owned());
    }
    if at != text.len() {
        return Err(format!("token bytes do not tile {text:?}"));
    }
    if out.ids.is_empty() {
        return Err(format!("no pieces cover the entity in {text:?}"));
    }
    Ok(out)
}

/// Why two occurrences' pieces differ, from their whole decoded text.
fn cause(a: &Occurrence, b: &Occurrence) -> &'static str {
    if a.ids == b.ids {
        return "match";
    }
    let (da, db) = (a.pieces.concat(), b.pieces.concat());
    let (ta, tb) = (da.trim_start(), db.trim_start());
    let space_differs = da.starts_with(' ') != db.starts_with(' ');
    if da == db {
        if a.right_spill || b.right_spill || a.left_spill || b.left_spill {
            "punctuation"
        } else {
            "segmentation"
        }
    } else if ta == tb && space_differs {
        "leading_space"
    } else if ta.to_lowercase() == tb.to_lowercase() && !space_differs {
        "casing"
    } else if ta.to_lowercase() == tb.to_lowercase() {
        "space_and_casing"
    } else if a.right_spill || b.right_spill || a.left_spill || b.left_spill {
        "punctuation"
    } else {
        "other"
    }
}

fn prefix(o: &Occurrence, m: usize) -> &[u32] {
    &o.ids[..m.min(o.ids.len())]
}

/// Byte offset of `needle` in `haystack` at word boundaries (ASCII
/// case-insensitive when `fold`), the first occurrence at or after `from`.
fn find_word(haystack: &str, needle: &str, from: usize, fold: bool) -> Option<usize> {
    let (h, n) = if fold {
        (haystack.to_ascii_lowercase(), needle.to_ascii_lowercase())
    } else {
        (haystack.to_owned(), needle.to_owned())
    };
    let bytes = h.as_bytes();
    let mut at = from;
    while let Some(found) = h.get(at..).and_then(|rest| rest.find(&n)) {
        let start = at + found;
        let end = start + n.len();
        let left = start == 0 || !bytes[start - 1].is_ascii_alphanumeric();
        let right = end == bytes.len() || !bytes[end].is_ascii_alphanumeric();
        if left && right {
            return Some(start);
        }
        at = start + 1;
    }
    None
}

/// The MQAR reply form of an expected (or generated) reply.
fn reply_form(reply: &str, key: &str) -> &'static str {
    let lower = reply.trim().to_ascii_lowercase();
    if find_word(&lower, key, 0, true) == Some(0) {
        "rehearse"
    } else if lower.starts_with("it's ") {
        "bare_its"
    } else if lower.starts_with("that's ") {
        "bare_thats"
    } else if find_word(&lower, key, 0, true).is_some() {
        "key_later"
    } else {
        "other"
    }
}

/// The MQAR key, recovered from the query's recorded template.
fn mqar_key(query: &Turn2) -> Result<String, String> {
    let template = query
        .tag
        .template
        .as_deref()
        .ok_or("an MQAR query records no template")?;
    let (before, after) = template
        .split_once("{k}")
        .ok_or("an MQAR template has no {k}")?;
    let user = &query.user;
    if user.len() < before.len() + after.len()
        || !user.ends_with(after)
        || !user[..before.len()].eq_ignore_ascii_case(before)
    {
        return Err(format!("query {user:?} does not fit template {template:?}"));
    }
    Ok(user[before.len()..user.len() - after.len()].to_owned())
}

/// One position pair's tally: (mismatch counts at each m, of, causes).
#[derive(Default)]
struct Tally {
    of: usize,
    mismatch: [usize; 3],
    causes: BTreeMap<String, usize>,
}

impl Tally {
    fn add(&mut self, a: &Occurrence, b: &Occurrence) {
        self.of += 1;
        for (slot, m) in MS.iter().enumerate() {
            if prefix(a, *m) != prefix(b, *m) {
                self.mismatch[slot] += 1;
            }
        }
        *self.causes.entry(cause(a, b).to_owned()).or_default() += 1;
    }

    fn json(&self) -> Value {
        let rate = |n: usize| {
            if self.of == 0 {
                Value::Null
            } else {
                json!(n as f64 / self.of as f64)
            }
        };
        json!({
            "of": self.of,
            "mismatch": MS.iter().zip(self.mismatch).map(|(m, n)| {
                (format!("m{m}"), json!({"count": n, "rate": rate(n)}))
            }).collect::<serde_json::Map<_, _>>(),
            "causes": self.causes,
        })
    }
}

fn piece_json(o: &Occurrence) -> Value {
    json!({"ids": o.ids, "pieces": o.pieces, "right_spill": o.right_spill, "left_spill": o.left_spill})
}

fn draw(
    seed: u64,
    conversations: usize,
    split: Split,
    count: &dyn Fn(&str) -> usize,
) -> Result<Vec<Conversation2>, String> {
    let mut world = MWorld2::new(count, Mix::default()).map_err(|e| e.to_string())?;
    let mut rng = Rng::new(seed);
    (0..conversations)
        .map(|_| {
            world
                .conversation(&mut rng, split)
                .map_err(|e| e.to_string())
        })
        .collect()
}

/// The audit of one draw under one protocol: tallies and per-fact rows.
struct Audit {
    tallies: BTreeMap<String, Tally>,
    forms: BTreeMap<String, usize>,
    rows: Vec<Value>,
}

fn audit(
    tokenizer: &ByteBpeTokenizer,
    drawn: &[Conversation2],
    spaced: bool,
    keep_rows: bool,
) -> Result<Audit, String> {
    let mut out = Audit {
        tallies: BTreeMap::new(),
        forms: BTreeMap::new(),
        rows: Vec::new(),
    };
    let mut tally = |name: &str, a: &Occurrence, b: &Occurrence| {
        out.tallies.entry(name.to_owned()).or_default().add(a, b);
    };
    let at = |content: &str, word: &str, fold: bool| -> Result<Occurrence, String> {
        let start = find_word(content, word, 0, fold)
            .ok_or_else(|| format!("{word:?} not found in {content:?}"))?;
        occurrence(tokenizer, content, spaced, start, word.len())
    };
    for (index, conversation) in drawn.iter().enumerate() {
        for (t, turn) in conversation.turns.iter().enumerate() {
            match turn.category {
                Category2::Mqar => {
                    let key = mqar_key(turn)?;
                    let value = turn.tag.answer.clone().ok_or("MQAR query without answer")?;
                    let assertion = &conversation.turns[0].user;
                    // The queried pair "{k} is {v}" locates both entities.
                    let pair = format!("{key} is {value}");
                    let p = find_word(assertion, &pair, 0, false)
                        .ok_or_else(|| format!("pair {pair:?} not in {assertion:?}"))?;
                    let key_a = occurrence(tokenizer, assertion, spaced, p, key.len())?;
                    let value_a =
                        occurrence(tokenizer, assertion, spaced, p + key.len() + 4, value.len())?;
                    let key_q = at(&turn.user, &key, false)?;
                    let form = reply_form(&turn.reply, &key);
                    *out.forms.entry(form.to_owned()).or_default() += 1;
                    let value_r = at(&turn.reply, &value, false)?;
                    let memory = format!("Memory: {value}.");
                    let value_m = at(&memory, &value, false)?;
                    tally("mqar/key/A-Q", &key_a, &key_q);
                    tally("mqar/value/A-R", &value_a, &value_r);
                    tally("mqar/value/A-M", &value_a, &value_m);
                    let key_r = if form == "rehearse" {
                        let key_r = occurrence(tokenizer, &turn.reply, spaced, 0, key.len())?;
                        tally("mqar/key/A-R(rehearse)", &key_a, &key_r);
                        tally("mqar/key/Q-R(rehearse)", &key_q, &key_r);
                        // The rule's denominator: every key/value tuple that
                        // appears at the reply start.
                        tally("mqar/rule/A-R", &key_a, &key_r);
                        Some(key_r)
                    } else {
                        None
                    };
                    tally("mqar/rule/A-R", &value_a, &value_r);
                    if keep_rows {
                        out.rows.push(json!({
                            "conversation": index, "turn": t, "kind": "mqar",
                            "key": key, "value": value, "form": form,
                            "assertion": assertion, "query": turn.user, "reply": turn.reply,
                            "key_A": piece_json(&key_a), "key_Q": piece_json(&key_q),
                            "key_R": key_r.as_ref().map(piece_json),
                            "value_A": piece_json(&value_a), "value_R": piece_json(&value_r),
                            "value_M": piece_json(&value_m),
                            "key_cause_A_R": key_r.as_ref().map(|r| cause(&key_a, r)),
                            "value_cause_A_R": cause(&value_a, &value_r),
                        }));
                    }
                }
                Category2::Relation if !turn.tag.abstain && turn.intent.ends_with("_query") => {
                    let value = turn
                        .tag
                        .answer
                        .clone()
                        .ok_or("relation query without answer")?;
                    // The latest earlier user turn stating the value.
                    let Some(source) = conversation.turns[..t]
                        .iter()
                        .rev()
                        .find(|s| find_word(&s.user, &value, 0, false).is_some())
                    else {
                        // A closed value can be stated in another casing.
                        *out.forms
                            .entry("relation/unstated_exact".into())
                            .or_default() += 1;
                        continue;
                    };
                    let value_a = at(&source.user, &value, false)?;
                    let value_r = at(&turn.reply, &value, false)?;
                    let memory = format!("Memory: {value}.");
                    let value_m = at(&memory, &value, false)?;
                    let pool = match turn.tag.pool {
                        Some(uor_r4_training::milestone_world_v2::Pool::Closed) => "closed",
                        _ => "open",
                    };
                    tally(&format!("relation/{pool}/value/A-R"), &value_a, &value_r);
                    tally(&format!("relation/{pool}/value/A-M"), &value_a, &value_m);
                    if keep_rows {
                        out.rows.push(json!({
                            "conversation": index, "turn": t, "kind": "relation",
                            "intent": turn.intent, "pool": pool, "value": value,
                            "assertion": source.user, "query": turn.user, "reply": turn.reply,
                            "value_A": piece_json(&value_a), "value_R": piece_json(&value_r),
                            "value_cause_A_R": cause(&value_a, &value_r),
                        }));
                    }
                }
                Category2::Copy => {
                    let words = &turn.tag.values;
                    let mut from_user = turn
                        .user
                        .find(": ")
                        .map(|i| i + 2)
                        .ok_or("a copy turn without ': '")?;
                    let mut from_reply = 0;
                    let mut causes = Vec::new();
                    for (w, word) in words.iter().enumerate() {
                        let u = find_word(&turn.user, word, from_user, false)
                            .ok_or("copy word not in user")?;
                        let r = find_word(&turn.reply, word, from_reply, false)
                            .ok_or("copy word not in reply")?;
                        from_user = u + word.len();
                        from_reply = r + word.len();
                        let a = occurrence(tokenizer, &turn.user, spaced, u, word.len())?;
                        let b = occurrence(tokenizer, &turn.reply, spaced, r, word.len())?;
                        let name = if w == 0 {
                            "copy/first_word/U-R"
                        } else {
                            "copy/later_words/U-R"
                        };
                        tally(name, &a, &b);
                        causes.push(cause(&a, &b));
                    }
                    if keep_rows {
                        out.rows.push(json!({
                            "conversation": index, "turn": t, "kind": "copy",
                            "user": turn.user, "reply": turn.reply, "causes": causes,
                        }));
                    }
                }
                _ => {}
            }
        }
    }
    Ok(out)
}

fn audit_json(a: &Audit) -> Value {
    json!({
        "tallies": a.tallies.iter().map(|(k, t)| (k.clone(), t.json())).collect::<serde_json::Map<_, _>>(),
        "mqar_reply_forms": a.forms,
    })
}

/// The user texts and model replies of a session report's default arm.
fn session_check(path: &Path, drawn: &[Conversation2]) -> Result<Value, String> {
    let report: Value = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let conversations = report["arms"]["default"]["conversations"]
        .as_array()
        .ok_or("no arms.default.conversations")?;
    let (mut equal, mut compared) = (0usize, 0usize);
    let mut forms: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for (conversation, drawn) in conversations.iter().zip(drawn) {
        let turns = conversation["turns"].as_array().ok_or("no turns")?;
        for (row, turn) in turns.iter().zip(&drawn.turns) {
            compared += 1;
            equal += usize::from(row["user"].as_str() == Some(turn.user.as_str()));
            if turn.category == Category2::Mqar {
                let key = mqar_key(turn)?;
                let reply = row["reply"].as_str().unwrap_or_default();
                let entry = forms.entry(reply_form(reply, &key).to_owned()).or_default();
                entry.0 += 1;
                entry.1 += usize::from(row["pass"].as_bool() == Some(true));
            }
        }
    }
    Ok(json!({
        "path": path.display().to_string(),
        "sha256": sha256(&fs::read(path).map_err(|e| e.to_string())?),
        "conversations": conversations.len(),
        "user_turns_equal": equal, "user_turns_compared": compared,
        "model_mqar_reply_forms": forms.iter().map(|(k, (n, pass))| {
            (k.clone(), json!({"count": n, "pass": pass}))
        }).collect::<serde_json::Map<_, _>>(),
        "seed": report["seed"], "dialogue_protocol_version": report["dialogue_protocol_version"],
        "log_recall": report["log_recall"], "model_root": report["model_root"],
    }))
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn run() -> Result<(), String> {
    let args = parse_args()?;
    report_output::claim(&args.out).map_err(|e| e.to_string())?;
    let tokenizer_bytes = fs::read(&args.tokenizer).map_err(|e| format!("tokenizer: {e}"))?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or("tokenizer: not a supported byte-level BPE tokenizer.json")?;
    let count = |text: &str| tokenizer.encode(text).len();
    let development = draw(args.seed, args.conversations, Split::Development, &count)?;
    let train = draw(
        args.train_seed,
        args.train_conversations,
        Split::Train,
        &count,
    )?;

    let v2 = audit(&tokenizer, &development, true, true)?;
    let v1 = audit(&tokenizer, &development, false, false)?;
    let train_v2 = audit(&tokenizer, &train, true, false)?;

    let rule = &v2.tallies["mqar/rule/A-R"];
    let rates: Vec<f64> = rule
        .mismatch
        .iter()
        .map(|n| *n as f64 / rule.of.max(1) as f64)
        .collect();
    let fires = rates.iter().any(|r| *r > THRESHOLD);
    let decision = if fires {
        "copy boundary fix (Step 4) must precede chat lineage verdicts"
    } else {
        "tuple mismatch at or under 10%: no Step 4 precondition from 0b"
    };

    let sessions = args
        .sessions
        .iter()
        .map(|p| session_check(p, &development))
        .collect::<Result<Vec<_>, _>>()?;

    let mqar_rows = development
        .iter()
        .flat_map(|c| &c.turns)
        .filter(|t| t.category == Category2::Mqar)
        .count();
    let report = json!({
        "schema": "uor-r4.step0b-key-tuple-audit/1",
        "card": "Step 0b tokenizer/format audit #820 (barrier assessment 2026-10-05)",
        "training_free": true,
        "tokenizer": {
            "path": args.tokenizer.display().to_string(),
            "sha256": sha256(&tokenizer_bytes),
            "address": tokenizer.address(),
        },
        "world": {"version": "m-world-v2", "digest": MWorld2::digest(), "mix": Mix::default()},
        "draw": {
            "split": "development", "seed": args.seed, "conversations": args.conversations,
            "mqar_queries": mqar_rows,
            "as": "m-world session world=v2 (its default seed, conversations and split)",
        },
        "train_draw": {"split": "train", "seed": args.train_seed, "conversations": args.train_conversations},
        "protocol_v2": audit_json(&v2),
        "protocol_v1": audit_json(&v1),
        "train_protocol_v2": audit_json(&train_v2),
        "decision_rule": {
            "rule": "if more than 10% of MQAR key/value tuples mismatch between the assertion and the reply start (m = 1, 2 or 3), the copy boundary fix (Step 4) must precede chat lineage verdicts",
            "denominator": "protocol v2: every MQAR value tuple (A vs R) plus every key tuple that appears at the reply start (rehearsing replies)",
            "of": rule.of,
            "mismatch": rule.mismatch,
            "rates": rates,
            "threshold": THRESHOLD,
            "fires": fires,
            "outcome": decision,
        },
        "sessions": sessions,
        "executable_sha256": std::env::current_exe().ok().and_then(|p| fs::read(p).ok()).map(|b| sha256(&b)),
    });
    let write = |name: &str, bytes: &[u8]| -> Result<(), String> {
        let mut file = fs::File::create_new(args.out.join(name)).map_err(|e| e.to_string())?;
        file.write_all(bytes).map_err(|e| e.to_string())
    };
    let report_text = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
    write("report.json", report_text.as_bytes())?;
    let mut rows = String::new();
    for row in &v2.rows {
        rows.push_str(&row.to_string());
        rows.push('\n');
    }
    write("rows-v2.jsonl", rows.as_bytes())?;
    // A sha256 manifest of the outputs, then the shared BLAKE3 seal.
    let mut manifest = String::new();
    for name in [report_output::ATTEMPT_FILE, "report.json", "rows-v2.jsonl"] {
        let bytes = fs::read(args.out.join(name)).map_err(|e| e.to_string())?;
        manifest.push_str(&format!("{}  {name}\n", sha256(&bytes)));
    }
    write("sha256-manifest.txt", manifest.as_bytes())?;
    report_output::seal(&args.out).map_err(|e| e.to_string())?;
    report_output::verify(&args.out).map_err(|e| e.to_string())?;

    println!(
        "MQAR queries: {mqar_rows}; reply forms (v2 dev): {:?}",
        v2.forms
    );
    for (name, audit) in [("v2", &v2), ("v1", &v1), ("train-v2", &train_v2)] {
        for (key, t) in &audit.tallies {
            println!(
                "{name} {key}: of {} mismatch m1 {} m2 {} m3 {} causes {:?}",
                t.of, t.mismatch[0], t.mismatch[1], t.mismatch[2], t.causes
            );
        }
    }
    println!("decision: fires={fires} rates={rates:?} -> {decision}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_word_respects_boundaries_and_case() {
        assert_eq!(find_word("bolt is 4, bol is 5", "bol", 0, false), Some(11));
        assert_eq!(find_word("Bol is 5.", "bol", 0, true), Some(0));
        assert_eq!(find_word("Bol is 5.", "bol", 0, false), None);
    }

    #[test]
    fn reply_forms() {
        assert_eq!(reply_form("Bol is 47.", "bol"), "rehearse");
        assert_eq!(reply_form("It's 47.", "bol"), "bare_its");
        assert_eq!(reply_form("That's 47.", "bol"), "bare_thats");
        assert_eq!(reply_form("47.", "bol"), "other");
    }
}
