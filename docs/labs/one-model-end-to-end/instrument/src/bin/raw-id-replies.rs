//! Greedy replies of a served `UORLUT01` artifact to raw token-id prompts, with or without the
//! turn memory's reply mark.
//!
//! `uor-chat --prompt-ids` steps raw ids and never says where the reply begins, so a model with
//! a turn memory answers there with the memory off. This reader steps the same ids through the
//! product path as `StackChat::feed` does for a text turn: every prompt id but the last through
//! `IntegerStackSession::read_step` (claude-348), the last and each reply id through
//! `serve_step`, and with `--mark-reply 1` the reply start marked before the prompt's last
//! token. With `--mark-reply 0` its replies must equal `uor-chat --prompt-ids` id for id; that
//! is the reader's own control.
//!
//! `--count-visits 1` (claude-348) also counts, at every step that chooses a token, how many
//! writer-table values an exact scan that stops a row early would read, and checks that scan's
//! answer, and the token the product's own scan chose, against the full scan's. It needs the
//! nearest head, adds `nearest_visit` to the receipt and changes no reply.
//!
//! `--time-steps 1` (claude-348) adds `step_time` to the receipt: how many prompt ids were read
//! and how many steps chose a token, and the nanoseconds spent in each kind. The count's own
//! scans are outside the clock.
//!
//! It is an instrument: it trains nothing, changes no artifact and moves no bar.
//!
//! ```text
//! raw-id-replies --stack ART.uorlut --samples ROWS.json --out REPLIES.jsonl \
//!   --mark-reply 0|1 [--max-tokens 48] [--count-visits 0|1] [--time-steps 0|1]
//! ```
//!
//! `ROWS.json` is a JSON array of objects with a `prompt` array of token ids (a float score's
//! `samples-*.json`). `REPLIES.jsonl` gets one `{"i", "ids"}` line a row: the greedy ids up to
//! and including the first end-of-reply id, or `--max-tokens` of them. The output file must not
//! exist. A receipt is printed on stdout.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::time::Instant;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_integer::stack::IntegerStackModel;

const SCHEMA: &str = "uor-r4.raw-id-replies/1";
const EOS: u32 = 1;
const USAGE: &str = "usage: raw-id-replies --stack FILE --samples FILE --out FILE --mark-reply 0|1 [--max-tokens N] [--count-visits 0|1] [--time-steps 0|1]";
/// How many values the early-stopping scan reads between two looks at its bound.
const BLOCK: usize = 16;

type Result<T> = std::result::Result<T, String>;

/// The L1 distance between a query block and a table block.
fn block_distance(query: &[i32], row: &[i16]) -> u64 {
    query
        .iter()
        .zip(row)
        .map(|(&x, &t)| u64::from(x.abs_diff(i32::from(t))))
        .sum()
}

/// The whole table's nearest row to `query`, every value read: the smaller id on a tie.
fn full_scan(table: &[i16], query: &[i32]) -> u32 {
    let (mut token, mut best) = (u32::MAX, u64::MAX);
    for (id, row) in table.chunks_exact(query.len()).enumerate() {
        let sum = block_distance(query, row);
        if sum < best {
            (token, best) = (id as u32, sum);
        }
    }
    token
}

/// The same answer by a scan that stops a row once its partial distance passes the best full
/// distance so far, and how many values it read. Rows go in id order and a later row replaces
/// the best only when strictly nearer, so a tie keeps the smaller id, as the full scan does.
fn early_stop_scan(table: &[i16], query: &[i32]) -> (u32, u64) {
    let (mut token, mut best, mut read) = (u32::MAX, u64::MAX, 0u64);
    for (id, row) in table.chunks_exact(query.len()).enumerate() {
        let mut sum = 0u64;
        let mut whole = true;
        for (xs, ts) in query.chunks(BLOCK).zip(row.chunks(BLOCK)) {
            sum += block_distance(xs, ts);
            read += xs.len() as u64;
            if sum > best {
                whole = false;
                break;
            }
        }
        if whole && sum < best {
            (token, best) = (id as u32, sum);
        }
    }
    (token, read)
}

/// The clock `--time-steps 1` keeps: steps of each kind and the nanoseconds spent in them.
#[derive(Default)]
struct StepClock {
    read_steps: u64,
    read_ns: u128,
    serve_steps: u64,
    serve_ns: u128,
}

/// The count `--count-visits 1` keeps over every choosing step.
#[derive(Default)]
struct Visits {
    queries: u64,
    read: u64,
    in_full_scans: u64,
    worst_per_mille: u64,
    disagree: u64,
    product_disagree: u64,
}

impl Visits {
    /// `chosen` is the token the product's own scan chose at this step
    /// (`IntegerStackSession::last_native_token`). A point key that names another token at a
    /// reply's opening would count as a product disagreement.
    fn count(&mut self, table: &[i16], query: &[i32], chosen: Option<u32>) {
        let (token, read) = early_stop_scan(table, query);
        let whole = table.len() as u64;
        let full = full_scan(table, query);
        self.queries += 1;
        self.read += read;
        self.in_full_scans += whole;
        self.worst_per_mille = self.worst_per_mille.max(read * 1000 / whole.max(1));
        if token != full {
            self.disagree += 1;
        }
        if chosen != Some(full) {
            self.product_disagree += 1;
        }
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("raw-id-replies: {error}");
        std::process::exit(1);
    }
}

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|index| args.get(index + 1))
        .cloned()
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn prompt_ids(row: &Value, index: usize) -> Result<Vec<u32>> {
    let ids = row
        .get("prompt")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("row {index}: no prompt array"))?;
    if ids.is_empty() {
        return Err(format!("row {index}: empty prompt"));
    }
    ids.iter()
        .map(|id| {
            id.as_u64()
                .and_then(|id| u32::try_from(id).ok())
                .ok_or_else(|| format!("row {index}: a prompt id is not a u32"))
        })
        .collect()
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let need = |name: &str| flag(&args, name).ok_or_else(|| format!("missing {name}\n{USAGE}"));
    let stack_path = need("--stack")?;
    let samples_path = need("--samples")?;
    let out_path = need("--out")?;
    let mark = match need("--mark-reply")?.as_str() {
        "0" => false,
        "1" => true,
        other => return Err(format!("--mark-reply takes 0 or 1, not {other}")),
    };
    let max_tokens = match flag(&args, "--max-tokens") {
        Some(value) => value
            .parse::<usize>()
            .ok()
            .filter(|n| *n > 0)
            .ok_or_else(|| format!("--max-tokens takes a positive integer, not {value}"))?,
        None => 48,
    };
    let count_visits = match flag(&args, "--count-visits").as_deref() {
        None | Some("0") => false,
        Some("1") => true,
        Some(other) => return Err(format!("--count-visits takes 0 or 1, not {other}")),
    };
    let time_steps = match flag(&args, "--time-steps").as_deref() {
        None | Some("0") => false,
        Some("1") => true,
        Some(other) => return Err(format!("--time-steps takes 0 or 1, not {other}")),
    };

    let artifact = fs::read(&stack_path).map_err(|e| format!("{stack_path}: {e}"))?;
    let samples = fs::read(&samples_path).map_err(|e| format!("{samples_path}: {e}"))?;
    let model = IntegerStackModel::parse(&artifact).map_err(|e| format!("artifact: {e}"))?;
    let has_turn_memory = model.session().has_turn_memory();
    if mark && !has_turn_memory {
        // A marked run on a model without a turn memory would be an unmarked run by another name.
        return Err("--mark-reply 1 needs an artifact with a turn memory".to_owned());
    }
    let rows: Vec<Value> =
        serde_json::from_slice(&samples).map_err(|e| format!("{samples_path}: {e}"))?;
    let mut out = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&out_path)
        .map_err(|e| format!("{out_path}: {e}"))?;

    // The product scans the whole table only under the nearest head; under the writer head the
    // count would describe a scan the product never runs.
    let table = match (count_visits, model.fbs_writer_table()) {
        (false, _) => None,
        (true, Some(table)) if model.shape().head_nearest() => Some(table),
        (true, _) => {
            return Err(
                "--count-visits 1 needs an artifact with a writer table and the nearest head"
                    .to_owned(),
            )
        }
    };
    let mut visits = Visits::default();
    let mut clock = StepClock::default();
    let mut stopped_on_eos = 0usize;
    for (index, row) in rows.iter().enumerate() {
        let prompt = prompt_ids(row, index)?;
        let mut session = model.session();
        session.clear_reply_start();
        let mut pick = EOS;
        for (at, &token) in prompt.iter().enumerate() {
            if mark && at + 1 == prompt.len() {
                session.mark_reply_start();
            }
            // As `StackChat::feed` steps a prompt since claude-348: every id but the last is
            // read without a choice, and the last is served.
            if at + 1 < prompt.len() {
                let started = Instant::now();
                session
                    .read_step(token)
                    .map_err(|e| format!("row {index}, prompt id {at}: {e}"))?;
                clock.read_ns += started.elapsed().as_nanos();
                clock.read_steps += 1;
                continue;
            }
            // Destructured: the float scan (D32) reads a tuple index that starts a line as a
            // float literal.
            let started = Instant::now();
            let (served, _) = session
                .serve_step(token)
                .map_err(|e| format!("row {index}, prompt id {at}: {e}"))?;
            clock.serve_ns += started.elapsed().as_nanos();
            clock.serve_steps += 1;
            pick = served;
            if let (Some(table), Some(query)) = (table, session.last_query_q12()) {
                visits.count(table, query, session.last_native_token());
            }
        }
        let mut ids = Vec::with_capacity(max_tokens);
        loop {
            ids.push(pick);
            if pick == EOS {
                stopped_on_eos += 1;
                break;
            }
            if ids.len() == max_tokens {
                break;
            }
            let started = Instant::now();
            let (served, _) = session
                .serve_step(pick)
                .map_err(|e| format!("row {index}, reply id {}: {e}", ids.len()))?;
            clock.serve_ns += started.elapsed().as_nanos();
            clock.serve_steps += 1;
            pick = served;
            if let (Some(table), Some(query)) = (table, session.last_query_q12()) {
                visits.count(table, query, session.last_native_token());
            }
        }
        writeln!(out, "{}", json!({"i": index, "ids": ids}))
            .map_err(|e| format!("{out_path}: {e}"))?;
    }
    out.flush().map_err(|e| format!("{out_path}: {e}"))?;

    let mut receipt = json!({
        "schema": SCHEMA,
        "rows": rows.len(),
        "stopped_on_eos": stopped_on_eos,
        "mark_reply": mark,
        "max_tokens": max_tokens,
        "has_turn_memory": has_turn_memory,
        "stack_sha256": sha256_hex(&artifact),
        "samples_sha256": sha256_hex(&samples),
    });
    if count_visits {
        receipt["nearest_visit"] = json!({
            "block": BLOCK,
            "queries": visits.queries,
            "values_read": visits.read,
            "values_in_full_scans": visits.in_full_scans,
            "per_mille": visits.read * 1000 / visits.in_full_scans.max(1),
            "worst_query_per_mille": visits.worst_per_mille,
            "disagree_with_full_scan": visits.disagree,
            "product_disagree_with_full_scan": visits.product_disagree,
        });
    }
    if time_steps {
        let whole = |ns: u128| u64::try_from(ns).unwrap_or(u64::MAX);
        receipt["step_time"] = json!({
            "read_steps": clock.read_steps,
            "read_ns": whole(clock.read_ns),
            "serve_steps": clock.serve_steps,
            "serve_ns": whole(clock.serve_ns),
        });
    }
    println!("{receipt}");
    Ok(())
}
