//! Token counts for the open reply panel, with the engine `chat-grade` uses.
//!
//! Two modes, both CPU-only and model-free:
//!
//! * `mode=text` (default): `IN.json` is a JSON array of strings; `OUT.json` is a
//!   JSON array of `{"tokens": n, "decode_roundtrip": bool, "ids": [...]}`.
//!   This is the instrument for one question: whether a recorded reply ran to
//!   the generation cap or stopped on its own. `chat-grade`'s `Reply::stop_with`
//!   ends a reply on EOS or on a short repeated cycle, and `reply_panel` stores
//!   `decode(ids without EOS)`; the sealed grade report keeps only that text,
//!   not the ids. Re-encoding the text gives `t = |encode(decode(ids))|`, and
//!   rank-ordered BPE is the minimal segmentation of the decoded bytes, so
//!   `t <= len(ids) <= cap`. `t == cap` therefore proves the cap was reached.
//! * `mode=panel`: `IN.json` is a panel (rows with `id` and `user_turns`);
//!   `OUT.json` is `[{"id", "worst_case_positions"}]` computed exactly as
//!   `chat-grade check`'s `check_panel` does. This exists to validate the
//!   instrument against worst cases already recorded on main for `context=384`
//!   (`conversational-v3`: 336, `conversational-v4`: 347).
//!
//! ```text
//! cargo run --release -p uor-r4-tokenizer --example token-counts -- \
//!   TOKENIZER.json IN.json OUT.json [mode=text|panel] [protocol=2] [max_new_tokens=64]
//! ```
//!
//! No model is loaded and nothing is generated.

use std::fs;
use std::process::ExitCode;

use serde_json::{json, Value};
use uor_r4_tokenizer::dialogue::DialogueProtocol;
use uor_r4_tokenizer::ByteBpeTokenizer;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 3 {
        eprintln!(
            "usage: token-counts TOKENIZER.json IN.json OUT.json \
                   [mode=text|panel] [protocol=2] [max_new_tokens=64]"
        );
        return ExitCode::from(2);
    }
    let mut mode = "text";
    let mut protocol_version = 2u8;
    let mut max_new_tokens = 64usize;
    for option in &args[3..] {
        match option.split_once('=') {
            Some(("mode", value)) => mode = if value == "panel" { "panel" } else { "text" },
            Some(("protocol", value)) => protocol_version = value.parse().unwrap_or(2),
            Some(("max_new_tokens", value)) => max_new_tokens = value.parse().unwrap_or(64),
            _ => {
                eprintln!("unknown option {option}");
                return ExitCode::from(2);
            }
        }
    }
    let tokenizer_bytes = match fs::read(&args[0]) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("read {}: {error}", args[0]);
            return ExitCode::from(2);
        }
    };
    let Some(tokenizer) = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes) else {
        eprintln!("{} is not a supported byte-level BPE", args[0]);
        return ExitCode::from(2);
    };
    let Ok(input) = fs::read_to_string(&args[1]) else {
        eprintln!("read {}", args[1]);
        return ExitCode::from(2);
    };

    let rows: Vec<Value> = if mode == "panel" {
        let Ok(requests) = serde_json::from_str::<Vec<Value>>(&input) else {
            eprintln!("{} is not a JSON panel", args[1]);
            return ExitCode::from(2);
        };
        let Ok(protocol) = DialogueProtocol::literal_roles_version(&tokenizer, protocol_version)
        else {
            eprintln!("protocol {protocol_version} is not supported by this tokenizer");
            return ExitCode::from(2);
        };
        let Ok(encoder) = protocol.bind(&tokenizer) else {
            eprintln!("the protocol does not bind to this tokenizer");
            return ExitCode::from(2);
        };
        requests
            .iter()
            .map(|request| {
                let turns = request["user_turns"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default();
                let mut longest = 1usize; // the leading BOS, as check_panel counts it
                for (turn, user) in turns.iter().enumerate() {
                    let user = user.as_str().unwrap_or("");
                    longest +=
                        encoder.encode_user_prefix(user, turn != 0).tokens.len() + max_new_tokens;
                    if turn + 1 < turns.len() {
                        longest += 1;
                    }
                }
                json!({
                    "id": request["id"].as_str().unwrap_or(""),
                    "worst_case_positions": longest,
                })
            })
            .collect()
    } else {
        let Ok(texts) = serde_json::from_str::<Vec<String>>(&input) else {
            eprintln!("{} is not a JSON array of strings", args[1]);
            return ExitCode::from(2);
        };
        texts
            .iter()
            .map(|text| {
                let ids = tokenizer.encode(text);
                let roundtrip = tokenizer.decode(&ids) == *text;
                json!({"tokens": ids.len(), "decode_roundtrip": roundtrip, "ids": ids})
            })
            .collect()
    };

    if let Err(error) = fs::write(
        &args[2],
        serde_json::to_vec_pretty(&rows).unwrap_or_default(),
    ) {
        eprintln!("write {}: {error}", args[2]);
        return ExitCode::from(1);
    }
    if mode == "panel" {
        let worst = rows
            .iter()
            .max_by_key(|row| row["worst_case_positions"].as_u64().unwrap_or(0))
            .cloned()
            .unwrap_or(Value::Null);
        println!("{} rows, worst case {}", rows.len(), worst);
    } else {
        println!(
            "{} texts, {} tokens, tokenizer {}",
            rows.len(),
            rows.iter()
                .map(|row| row["tokens"].as_u64().unwrap_or(0))
                .sum::<u64>(),
            tokenizer.address()
        );
    }
    ExitCode::SUCCESS
}
