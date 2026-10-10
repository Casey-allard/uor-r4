//! `numeric-pointer-trace` — does the copy pointer SELECT the stored digits?
//!
//! References #2029. The v5-answering artifact carries no memory reader, so the
//! only measurable half of the read/emit question is the pointer's: for each
//! numeric memory row, generate the reply with the per-step copy trace on and
//! compare, **as token ids**, what the pointer selected (`source_id`) with what
//! the decoder emitted (`id`).
//!
//! Class rules, fixed before the run:
//!   READER     — the stored value's digit token never appears as any step's
//!                `source_id`.
//!   EMITTER    — some step's `source_id` IS the stored digit token while that
//!                step's emitted `id` is not.
//!   PARTIAL    — the pointer selects one digit of a two-digit value but never
//!                the other (the run-level hypothesis this probe tests).
//!
//! Usage:
//!   numeric-pointer-trace model=DIR tokenizer=T.json requests=P.json \
//!     values=id<TAB>value.tsv out=OUT.json [cap=64] [protocol=2]

use std::error::Error;
use std::fs;
use std::path::PathBuf;

use candle_core::Device;
use serde_json::{json, Value};
use uor_r4_tokenizer::dialogue::DialogueProtocol;
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::geometric_stack::StackModel;
use uor_r4_training::stack_dialogue::{
    greedy_reply_with_copy_stop, load_requests, reply_panel, Request,
};

fn arg(args: &[(String, String)], key: &str) -> Option<String> {
    args.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
}

fn main() -> Result<(), Box<dyn Error>> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut args: Vec<(String, String)> = Vec::new();
    for item in &raw {
        let (k, v) = item
            .split_once('=')
            .ok_or_else(|| format!("expected key=value, got {item}"))?;
        args.push((k.to_owned(), v.to_owned()));
    }
    let model_dir = PathBuf::from(arg(&args, "model").ok_or("model= is required")?);

    let tokenizer_path = PathBuf::from(arg(&args, "tokenizer").ok_or("tokenizer= is required")?);
    let requests_path = PathBuf::from(arg(&args, "requests").ok_or("requests= is required")?);
    let values_path = PathBuf::from(arg(&args, "values").ok_or("values= is required")?);
    let out = PathBuf::from(arg(&args, "out").ok_or("out= is required")?);
    let cap: usize = arg(&args, "cap").unwrap_or_else(|| "64".into()).parse()?;
    let version: u8 = arg(&args, "protocol")
        .unwrap_or_else(|| "2".into())
        .parse()?;

    // id -> stored value, as a string of digits.
    let mut stored: Vec<(String, String)> = Vec::new();
    for line in fs::read_to_string(&values_path)?.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let (id, value) = line
            .split_once('\t')
            .ok_or("values line needs id<TAB>value")?;
        stored.push((id.to_owned(), value.trim().to_owned()));
    }

    let tokenizer_bytes = fs::read(&tokenizer_path)?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or("tokenizer did not parse")?;
    let protocol = DialogueProtocol::literal_roles_version(&tokenizer, version)?;
    let encoder = protocol.bind(&tokenizer)?;
    let requests: Vec<Request> = load_requests(&requests_path)?;
    let mut model = StackModel::load(&model_dir, &Device::Cpu)?;
    // The trace is the instrument: without it the decoder records no steps.
    model.set_pointer_copy_trace(true);

    // id -> (stored digit token ids, trace steps, emitted ids)
    let mut captured: Vec<(String, Vec<u32>, Vec<Value>, Vec<u32>)> = Vec::new();
    let panel = reply_panel(
        &encoder,
        &protocol,
        &requests,
        model.config.context,
        cap,
        &|ids| tokenizer.decode(ids),
        &mut |history, cap| {
            let reply = greedy_reply_with_copy_stop(&model, history, cap, protocol.eos_id, 3)?;
            let id = format!("row{}", captured.len());
            let steps: Vec<Value> = reply
                .trace
                .iter()
                .map(|s| {
                    json!({
                        "step": s.step, "id": s.id, "source": s.source,
                        "source_id": s.source_id, "matches_source": s.matches_source,
                        "attention": s.attention,
                    })
                })
                .collect();
            captured.push((id, reply.ids.clone(), steps, history.to_vec()));
            Ok(reply)
        },
    )?;

    // The panel rows are in request order.
    let rows = panel["rows"].as_array().cloned().unwrap_or_default();
    let mut results = Vec::new();
    let mut class_counts: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    for (index, (id, value)) in stored.iter().enumerate() {
        let (_, emitted, steps, history) = captured
            .get(index)
            .ok_or_else(|| format!("no trace captured for {id}"))?;
        // The stored value's own token ids, exactly as the tokenizer makes them.
        let digit_ids: Vec<u32> = tokenizer.encode(value);
        let selected: Vec<u32> = steps
            .iter()
            .filter_map(|s| s["source_id"].as_u64().map(|v| v as u32))
            .collect();
        let present = digit_ids.iter().filter(|d| selected.contains(d)).count();
        let emitted_present = digit_ids.iter().filter(|d| emitted.contains(d)).count();
        let class = if present == 0 {
            "READER"
        } else if emitted_present < present {
            "EMITTER"
        } else if present < digit_ids.len() {
            "PARTIAL"
        } else {
            "DELIVERED"
        };
        *class_counts.entry(class.to_owned()).or_insert(0) += 1;
        let audit: Vec<Value> = steps
            .iter()
            .filter(|s| {
                s["source_id"]
                    .as_u64()
                    .map(|v| digit_ids.contains(&(v as u32)))
                    .unwrap_or(false)
            })
            .take(6)
            .cloned()
            .collect();
        results.push(json!({
            "id": id, "value": value, "digit_ids": digit_ids,
            "class": class,
            "digit_tokens_selected": present,
            "digit_tokens_emitted": emitted_present,
            "emitted_ids": emitted,
            "history_len": history.len(),
            "steps_total": steps.len(),
            "audit_steps_selecting_a_digit": audit,
            "panel_row": rows.get(index).cloned(),
        }));
    }

    let out_json = json!({
        "schema": "uor-r4.numeric-pointer-trace/1",
        "model": model_dir.display().to_string(),
        "cap": cap,
        "protocol": version,
        "class_rule": "READER = no stored digit token ever selected; EMITTER = a stored digit \
                       token selected while not emitted at that step; PARTIAL = only some of the \
                       value's digit tokens ever selected; DELIVERED = every digit token selected \
                       and every one emitted. Compared as token ids, never strings.",
        "class_counts": class_counts,
        "rows": results,
    });
    fs::write(&out, serde_json::to_vec_pretty(&out_json)?)?;
    println!(
        "{} rows traced; classes {}",
        results.len(),
        serde_json::to_string(&out_json["class_counts"])?
    );
    Ok(())
}
