//! Greedy replies of a served `UORLUT01` artifact to raw token-id prompts, with or without the
//! turn memory's reply mark.
//!
//! `uor-chat --prompt-ids` steps raw ids and never says where the reply begins, so a model with
//! a turn memory answers there with the memory off. This reader steps the same ids through
//! `IntegerStackSession::serve_step`, the product path, and with `--mark-reply 1` marks the reply
//! start before the prompt's last token, as `StackChat::feed` does for a text turn. With
//! `--mark-reply 0` its replies must equal `uor-chat --prompt-ids` id for id; that is the
//! reader's own control.
//!
//! It is an instrument: it trains nothing, changes no artifact and moves no bar.
//!
//! ```text
//! raw-id-replies --stack ART.uorlut --samples ROWS.json --out REPLIES.jsonl \
//!   --mark-reply 0|1 [--max-tokens 48]
//! ```
//!
//! `ROWS.json` is a JSON array of objects with a `prompt` array of token ids (a float score's
//! `samples-*.json`). `REPLIES.jsonl` gets one `{"i", "ids"}` line a row: the greedy ids up to
//! and including the first end-of-reply id, or `--max-tokens` of them. The output file must not
//! exist. A receipt is printed on stdout.

use std::fs::{self, OpenOptions};
use std::io::Write;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_integer::stack::IntegerStackModel;

const SCHEMA: &str = "uor-r4.raw-id-replies/1";
const EOS: u32 = 1;
const USAGE: &str = "usage: raw-id-replies --stack FILE --samples FILE --out FILE --mark-reply 0|1 [--max-tokens N]";

type Result<T> = std::result::Result<T, String>;

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
            // Destructured: the float scan (D32) reads a tuple index that starts a line as a
            // float literal.
            let (served, _) = session
                .serve_step(token)
                .map_err(|e| format!("row {index}, prompt id {at}: {e}"))?;
            pick = served;
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
            let (served, _) = session
                .serve_step(pick)
                .map_err(|e| format!("row {index}, reply id {}: {e}", ids.len()))?;
            pick = served;
        }
        writeln!(out, "{}", json!({"i": index, "ids": ids}))
            .map_err(|e| format!("{out_path}: {e}"))?;
    }
    out.flush().map_err(|e| format!("{out_path}: {e}"))?;

    println!(
        "{}",
        json!({
            "schema": SCHEMA,
            "rows": rows.len(),
            "stopped_on_eos": stopped_on_eos,
            "mark_reply": mark,
            "max_tokens": max_tokens,
            "has_turn_memory": has_turn_memory,
            "stack_sha256": sha256_hex(&artifact),
            "samples_sha256": sha256_hex(&samples),
        })
    );
    Ok(())
}
