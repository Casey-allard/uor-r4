//! End-to-end chat replay for the `uor-chat --stack` serving path.
//!
//! The fixture is a recorded `geometric-stack lut-chat` transcript
//! (`uor-r4.geometric-stack-lut-chat/1`: the D10 integer engine, temperature 0,
//! the literal-role protocol) of a dialogue-trained `UORLUT01` artifact. Every
//! reply id in it must come back from `uor_r4_integer::stack::StackChat` on the
//! same artifact, tokenizer and protocol, and the history this path builds must
//! equal the recorded one. The D10 and D11 engines are bit-exact on exported
//! stacks (`uor-r4-training`'s `stack_d11_oracle` tests, `d11-evaluate`), so
//! this test compares the chat front-end — history handling, the protocol
//! encoding, the stop rules — not the kernels.
//!
//! Fixture-dependent: ignored by default, and fails when run without the local
//! dialogue artifact and its recorded transcript. Run it explicitly:
//!
//! ```text
//! cargo test -p uor-r4-integer --release --test stack_chat_reference -- --ignored --nocapture
//! ```

use serde_json::Value;
use std::fs;
use std::path::Path;
use uor_r4_integer::stack::{IntegerStackModel, StackChat, StackStop};
use uor_r4_integer::IntegerError;

/// The dialogue-trained stack artifact (`uor-r4.lut-stack/1`, width 288,
/// pattern `rrarra`, Lorentz read, context 256) and its recorded chat.
const ARTIFACT_PATH: &str =
    "/Users/casey.allard/uor-r4-local/workspace/uor-r4-lab/anti-gravity-d4/s2_dialogue_qat_1/export/model.lut";
const REFERENCE_PATH: &str =
    "/Users/casey.allard/uor-r4-local/workspace/uor-r4-lab/anti-gravity-d4/s2_dialogue_qat_1/chat/chat.json";
/// The tokenizer whose sha256 the recorded transcript binds
/// (`d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`).
const TOKENIZER_PATH: &str =
    "/Users/casey.allard/uor-r4-local/workspace/uor-r4-lab/claude-t4-1433-resume/bundle-learned-1/tokenizer.json";

fn invalid(message: impl Into<String>) -> IntegerError {
    IntegerError::Invalid(message.into())
}

fn require(path: &Path) -> Result<(), IntegerError> {
    if path.exists() {
        Ok(())
    } else {
        Err(invalid(format!(
            "required fixture {} is missing; this ignored test needs the local dialogue stack \
             artifact and its recorded lut-chat transcript",
            path.display()
        )))
    }
}

fn ids(value: &Value, what: &str) -> Result<Vec<u32>, IntegerError> {
    value
        .as_array()
        .ok_or_else(|| invalid(format!("{what} is not an array")))?
        .iter()
        .map(|entry| {
            entry
                .as_u64()
                .and_then(|id| u32::try_from(id).ok())
                .ok_or_else(|| invalid(format!("{what} holds a non-u32 id")))
        })
        .collect()
}

/// The recorded stop of a turn, as a comparable name and period.
fn recorded_stop(value: &Value) -> (&'static str, Option<usize>) {
    match value {
        Value::String(name) if name == "eos" => ("eos", None),
        Value::String(name) if name == "max_new_tokens" => ("max_new_tokens", None),
        value => (
            "short_cycle",
            value
                .get("short_cycle")
                .and_then(Value::as_u64)
                .and_then(|period| usize::try_from(period).ok()),
        ),
    }
}

fn served_stop(stop: StackStop) -> (&'static str, Option<usize>) {
    match stop {
        StackStop::Eos => ("eos", None),
        StackStop::ShortCycle { period } => ("short_cycle", Some(period)),
        StackStop::MaximumNewTokens => ("max_new_tokens", None),
    }
}

#[test]
#[ignore = "requires the local dialogue stack artifact and its recorded chat; run with --ignored"]
fn stack_chat_reproduces_the_recorded_greedy_transcript() -> Result<(), Box<dyn std::error::Error>>
{
    let (artifact, reference, tokenizer) = (
        Path::new(ARTIFACT_PATH),
        Path::new(REFERENCE_PATH),
        Path::new(TOKENIZER_PATH),
    );
    require(artifact)?;
    require(reference)?;
    require(tokenizer)?;
    let recorded: Value = serde_json::from_slice(&fs::read(reference)?)?;
    let version = match recorded["protocol"]["schema"].as_str() {
        Some(schema) if schema.ends_with("/2") => 2,
        _ => 1,
    };
    let max_new_tokens = recorded["record"]["max_new_tokens"]
        .as_u64()
        .and_then(|tokens| usize::try_from(tokens).ok())
        .ok_or_else(|| invalid("the recorded max_new_tokens is missing"))?;
    let model = IntegerStackModel::load(artifact)?;
    let mut chat = StackChat::from_tokenizer_path(&model, tokenizer, version)?;
    let eos = chat.protocol().eos_id;
    let context = chat.context();
    let rows = recorded["record"]["rows"]
        .as_array()
        .ok_or_else(|| invalid("the recorded transcript has no rows"))?;
    let (mut rows_compared, mut turns_compared) = (0usize, 0usize);
    let mut ids_compared = 0usize;
    for row in rows {
        let row_id = row["id"].as_str().unwrap_or("<unnamed>");
        let turns = row["turns"]
            .as_array()
            .ok_or_else(|| invalid(format!("row {row_id} has no turns")))?;
        let mut history = vec![chat.protocol().bos_id];
        for (index, turn) in turns.iter().enumerate() {
            let user = turn["user"]
                .as_str()
                .ok_or_else(|| invalid(format!("row {row_id} turn {} has no user", index + 1)))?;
            let expected = ids(&turn["reply_ids"], &format!("row {row_id} reply ids"))?;
            // `reply_panel`'s own history rule: the turn's prefix, then the reply.
            let prefix = chat
                .protocol()
                .bind(chat.tokenizer())?
                .encode_user_prefix(user, index != 0);
            history.extend_from_slice(&prefix.tokens);
            assert!(
                history.len() + max_new_tokens <= context,
                "row {row_id} turn {}: the recorded history does not fit the context",
                index + 1
            );
            let reply = chat.reply_history(&history, max_new_tokens)?;
            assert_eq!(
                reply.ids,
                expected,
                "row {row_id} turn {} ({user:?})",
                index + 1
            );
            assert_eq!(
                served_stop(reply.stop),
                recorded_stop(&turn["stop"]),
                "row {row_id} turn {} ({user:?}) stopped differently",
                index + 1
            );
            ids_compared += reply.ids.len();
            turns_compared += 1;
            history.extend_from_slice(&reply.ids);
            if reply.stop != StackStop::Eos && index + 1 < turns.len() {
                history.push(eos);
            }
        }
        assert_eq!(
            history,
            ids(&row["history_ids"], &format!("row {row_id} history"))?,
            "row {row_id}: the reconstructed history differs from the recorded one"
        );
        rows_compared += 1;
    }
    println!(
        "stack chat replay: {rows_compared} rows, {turns_compared} turns, {ids_compared} reply ids, \
         artifact sha256 {}",
        model.artifact_sha256()
    );
    Ok(())
}
