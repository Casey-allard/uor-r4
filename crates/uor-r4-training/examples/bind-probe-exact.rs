//! Exact-prefix binding probe driver for P1 of the 2026-10-08 cross-turn
//! binding review (#820). Inference only; reads the same `read_span_probe`
//! observation the `binding-probe` binary uses, but builds the window itself
//! so that turn 1's assistant reply can be pinned with the exact piece-wise
//! protocol encoding (including its EOS).
//!
//! ```text
//! bind-probe-exact model=DIR tokenizer=tokenizer.json rows=ROWS.json out=FILE.json
//!   [device=cpu|cuda|metal] [protocol=2] [max_new_tokens=40]
//!   [pointer_gate_floor=F] [pointer_copy_stop=MIN_SPAN]
//!   [pointer_copy_stop_identity=1] [copy_trace=1]
//!   [pointer_span_extract=0|1|2|3] [pointer_span_min=N]
//! ```
//!
//! `pointer_gate_floor=F` (default 0) is the serving-time copy-gate floor of
//! [`StackModel::set_pointer_gate_floor`]: the identity mixture uses
//! `max(gate, F)`. It is a read-out change only, and `0` leaves every reply,
//! mixture probability and gate exactly as an unfloored probe reports them.
//!
//! `pointer_copy_stop=MIN_SPAN` (default 0 = off) is the serving-time copy-stop
//! rule of [`StackModel::set_pointer_copy_stop`]: the reply ends once an
//! emitted id breaks a run of at least `MIN_SPAN` ids copied from the window
//! the pointer itself selected (the source it attends most at each step, then
//! the positions after it). Every row records `copy_stop` (the matched span)
//! and `reply_stop` (how the reply ended). The default is the historical
//! decoder bit for bit.
//!
//! `pointer_copy_stop_identity=1` (default 0 = the run-length rule above)
//! switches `pointer_copy_stop` to the TOKEN-IDENTITY rule of
//! [`CopyStop::identity`]: the reply ends at the last id that reproduces the
//! window span the pointer's own argmax source opened, and the first emitted
//! id that is not the span's next window id is dropped. `0` with
//! `pointer_copy_stop=0` leaves every reply and every record exactly as the
//! historical decoder does.
//!
//! `copy_trace=1` (default 0 = off) records the pointer's own selection at
//! every step of the reply (`reply_trace`) through
//! [`StackModel::set_pointer_copy_trace`].
//!
//! ROWS.json is an array of
//! `{"id","premise","question","expected":["two"],"forbid":["five"],"turn1":"Sure."}`
//! where `turn1` is a string to pin turn 1's reply, or `null` to generate it
//! greedily exactly as `reply_panel` does.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use candle_core::Device;
use serde_json::{json, Value};
use uor_r4_tokenizer::dialogue::DialogueProtocol;
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::binding_probe::{
    disjoint, phrase_positions, token_byte_ranges, words, Evidence, SPAN_KINDS, TAU,
};
use uor_r4_training::geometric_stack::{CopyStop, SpanExtract, StackModel};
use uor_r4_training::stack_dialogue::greedy_reply;

type Error = Box<dyn std::error::Error>;

fn arguments(list: &[String], keys: &[&str]) -> Result<BTreeMap<String, String>, Error> {
    let mut out = BTreeMap::new();
    for argument in list {
        let (key, value) = argument
            .split_once('=')
            .ok_or_else(|| format!("expected key=value, got {argument}"))?;
        if !keys.contains(&key) || out.insert(key.to_owned(), value.to_owned()).is_some() {
            return Err(format!("unknown or repeated argument {key}").into());
        }
    }
    Ok(out)
}

/// `binding-probe`'s spelling rule: the first token of the lower, capitalised,
/// spaced and unspaced spellings of a term.
fn first_token_candidates(tokenizer: &ByteBpeTokenizer, phrase: &[String]) -> Vec<u32> {
    let plain = phrase.join(" ");
    let mut capital = plain.clone();
    if let Some(first) = capital.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    let mut ids = BTreeSet::new();
    for variant in [
        format!(" {plain}"),
        plain.clone(),
        format!(" {capital}"),
        capital,
    ] {
        if let Some(&id) = tokenizer.encode(&variant).first() {
            ids.insert(id);
        }
    }
    ids.into_iter().collect()
}

fn window_text(tokenizer: &ByteBpeTokenizer, ids: &[u32]) -> (String, Vec<std::ops::Range<usize>>) {
    let pieces: Vec<Vec<u8>> = ids
        .iter()
        .map(|&id| tokenizer.decode_bytes(&[id]))
        .collect();
    let ranges = token_byte_ranges(&pieces);
    let bytes = pieces.concat();
    (String::from_utf8_lossy(&bytes).into_owned(), ranges)
}

fn run() -> Result<(), Error> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let args = arguments(
        &raw,
        &[
            "model",
            "tokenizer",
            "rows",
            "out",
            "device",
            "protocol",
            "max_new_tokens",
            "pointer_gate_floor",
            "pointer_copy_stop",
            "pointer_copy_stop_identity",
            "copy_trace",
            "pointer_span_extract",
            "pointer_span_min",
        ],
    )?;
    let required = |key: &str| -> Result<String, Error> {
        args.get(key)
            .cloned()
            .ok_or_else(|| format!("missing {key}=").into())
    };
    let model_dir = PathBuf::from(required("model")?);
    let tokenizer_path = PathBuf::from(required("tokenizer")?);
    let rows_path = PathBuf::from(required("rows")?);
    let out = PathBuf::from(required("out")?);
    if out.exists() {
        return Err(format!("{} exists", out.display()).into());
    }
    let version: u8 = args.get("protocol").map_or(Ok(2), |v| v.parse())?;
    let max_new_tokens: usize = args.get("max_new_tokens").map_or(Ok(40), |v| v.parse())?;
    let gate_floor: f64 = args
        .get("pointer_gate_floor")
        .map_or(Ok(0.0), |v| v.parse())?;
    let copy_stop_min_span: usize = args.get("pointer_copy_stop").map_or(Ok(0), |v| v.parse())?;
    let copy_stop_identity: u8 = args
        .get("pointer_copy_stop_identity")
        .map_or(Ok(0), |v| v.parse())?;
    let copy_trace: u8 = args.get("copy_trace").map_or(Ok(0), |v| v.parse())?;
    let span_extract: u8 = args
        .get("pointer_span_extract")
        .map_or(Ok(0), |v| v.parse())?;
    let span_min: usize = args.get("pointer_span_min").map_or(Ok(2), |v| v.parse())?;
    let device_name = args.get("device").map(String::as_str).unwrap_or("cpu");
    let device: Device = uor_r4_training::baseline_protocol::device(device_name)?;
    candle_core::cuda::set_gemm_reduced_precision_f32(false);

    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(&tokenizer_path)?)
        .ok_or("not a supported tokenizer.json")?;
    let protocol = DialogueProtocol::literal_roles_version(&tokenizer, version)?;
    let encoder = protocol.bind(&tokenizer)?;
    let mut model = StackModel::load(&model_dir, &device)?;
    model.set_pointer_gate_floor(gate_floor)?;
    model.set_pointer_copy_trace(copy_trace > 0);
    if copy_stop_min_span >= 2 {
        let rule = if copy_stop_identity > 0 {
            CopyStop::identity(copy_stop_min_span)
        } else {
            CopyStop::new(copy_stop_min_span)
        };
        model.set_pointer_copy_stop(Some(rule))?;
    }
    // `pointer_span_extract`: 0 off (the default), 1 mid-reply runs with the
    // anchor kept (the headline), 2 mid-reply runs with the anchor dropped
    // (the fitted variant), 3 any run with the anchor kept (the broad
    // control). A rule is a read-out change only; flag 0 sets nothing and adds
    // no field to any record, so a rule-off run is the sealed schema exactly.
    if span_extract > 0 {
        let rule = SpanExtract::from_flag(span_extract, span_min).ok_or(
            "unknown pointer_span_extract mode (0 off, 1 mid/kept, 2 mid/dropped, 3 any/kept)",
        )?;
        model.set_pointer_span_extract(Some(rule))?;
    }
    let rows: Vec<Value> = serde_json::from_slice(&fs::read(&rows_path)?)?;
    let eos = protocol.eos_id;

    let mut records = Vec::new();
    for row in &rows {
        let id = row["id"].as_str().ok_or("row without id")?.to_owned();
        let premise = row["premise"].as_str().ok_or("row without premise")?;
        let question = row["question"].as_str().ok_or("row without question")?;
        let expected: Vec<Vec<String>> = row["expected"]
            .as_array()
            .ok_or("row without expected")?
            .iter()
            .map(|t| words(t.as_str().unwrap_or_default()))
            .collect();
        let forbid: Vec<Vec<String>> = row["forbid"]
            .as_array()
            .ok_or("row without forbid")?
            .iter()
            .map(|t| words(t.as_str().unwrap_or_default()))
            .collect();
        let pinned = row["turn1"].as_str().map(str::to_owned);

        // The window `reply_panel` hands the last turn's reply: BOS, turn 1's
        // prefix, turn 1's reply ids (pinned or generated) closed with EOS,
        // then turn 2's prefix ending in the assistant marker.
        let mut window: Vec<u32> = vec![protocol.bos_id];
        window.extend(&encoder.encode_user_prefix(premise, false).tokens);
        let turn1_ids: Vec<u32> = match &pinned {
            Some(text) => {
                // The corpus renders an assistant turn as one `format!(" {content}")`
                // piece; `reply_panel` closes a non-terminal reply with EOS.
                let mut ids = tokenizer.encode(&format!(" {text}"));
                ids.push(eos);
                ids
            }
            None => {
                let reply = greedy_reply(&model, &window, max_new_tokens, eos)?;
                let mut ids = reply.ids.clone();
                if !reply.eos {
                    ids.push(eos);
                }
                ids
            }
        };
        window.extend(&turn1_ids);
        window.extend(&encoder.encode_user_prefix(question, true).tokens);
        if window.len() > model.config.context {
            return Err(format!("row {id}: window outgrew the context").into());
        }

        let (text, tokens) = window_text(&tokenizer, &window);
        let all = 0..window.len();
        let union = |terms: &[Vec<String>]| -> BTreeSet<usize> {
            terms
                .iter()
                .flat_map(|t| phrase_positions(&text, &tokens, all.clone(), t))
                .collect()
        };
        let spans = disjoint([
            union(&expected),
            union(&forbid),
            BTreeSet::new(),
            BTreeSet::new(),
            BTreeSet::new(),
        ]);
        let mut sets: Vec<Vec<usize>> = spans.to_vec();
        sets.push(all.clone().collect());
        let probe = model.read_span_probe(&window, &sets)?;

        let kinds = SPAN_KINDS.len();
        let mut read_expected = 0f64;
        let mut read_distractor = 0f64;
        let mut reads = Vec::new();
        for read in &probe.reads {
            let mut heads = Vec::new();
            for masses in &read.heads {
                read_expected = read_expected.max(masses[0]);
                read_distractor = read_distractor.max(masses[1]);
                heads.push(json!({
                    "expected": masses[0], "distractor": masses[1],
                    "other": masses[kinds] - masses[..kinds].iter().sum::<f64>(),
                    "no_read": 1.0 - masses[kinds],
                }));
            }
            reads.push(json!({"layer": read.layer, "heads": heads}));
        }
        let (pointer, pointer_expected, pointer_distractor) = match &probe.pointer {
            None => (Value::Null, None, None),
            Some(head) => {
                let mass = |set: &[usize]| set.iter().map(|&j| head.attention[j]).sum::<f64>();
                let masses: Vec<f64> = spans.iter().map(|s| mass(s)).collect();
                (
                    json!({"gate": head.gate, "expected": masses[0], "distractor": masses[1]}),
                    Some(masses[0]),
                    Some(masses[1]),
                )
            }
        };
        let outputs = |terms: &[Vec<String>]| -> (Vec<Value>, f64) {
            let mut best = 0f64;
            let records = terms
                .iter()
                .map(|term| {
                    let ids = first_token_candidates(&tokenizer, term);
                    let p_mix = ids
                        .iter()
                        .map(|&i| probe.mixture[i as usize])
                        .fold(0.0, f64::max);
                    let p_gen = ids
                        .iter()
                        .map(|&i| probe.generator[i as usize])
                        .fold(0.0, f64::max);
                    best = best.max(p_mix);
                    json!({"term": term.join(" "), "first_token_ids": ids,
                           "p_mixture": p_mix, "p_generator": p_gen})
                })
                .collect();
            (records, best)
        };
        let (expected_out, out_expected) = outputs(&expected);
        let (distractor_out, out_distractor) = outputs(&forbid);
        let greedy = probe
            .mixture
            .iter()
            .enumerate()
            .fold(
                0usize,
                |best, (i, &p)| {
                    if p > probe.mixture[best] {
                        i
                    } else {
                        best
                    }
                },
            );
        let evidence = Evidence {
            read_expected,
            read_distractor,
            pointer_expected,
            pointer_distractor,
            out_expected,
            out_distractor,
        };
        let reply = greedy_reply(&model, &window, max_new_tokens, eos)?;
        // The rule's own account of the span it stopped at, if it fired, and
        // the pointer's selection at every step when a trace was asked for.
        let copy_stop = reply.copy_stop;
        let span_report = reply.span_extract;
        let trace_value = if copy_trace > 0 {
            serde_json::to_value(&reply.trace)?
        } else {
            Value::Null
        };
        let reply_text = tokenizer.decode(
            &reply
                .ids
                .iter()
                .copied()
                .filter(|&i| i != eos)
                .collect::<Vec<u32>>(),
        );
        let mut record = json!({
            "id": id,
            "turn1_reply": pinned,
            "turn1_ids": turn1_ids,
            "window_tokens": window.len(),
            "window_ids": window,
            "window_text": text,
            "span_sizes": SPAN_KINDS.iter().zip(&spans).map(|(k, s)| (k.to_string(), json!(s.len()))).collect::<serde_json::Map<_, _>>(),
            "span_positions": {"expected": spans[0], "distractor": spans[1]},
            "reads": reads,
            "pointer": pointer,
            "evidence": {
                "read_expected_max": evidence.read_expected,
                "read_distractor_max": evidence.read_distractor,
                "pointer_expected": evidence.pointer_expected,
                "pointer_distractor": evidence.pointer_distractor,
                "out_expected": evidence.out_expected,
                "out_distractor": evidence.out_distractor,
            },
            "pattern": evidence.pattern(TAU).name(),
            "miss_class": evidence.miss_class(TAU).name(),
            "marker": {
                "expected": expected_out, "distractor": distractor_out,
                "greedy_token": {"id": greedy, "text": tokenizer.decode(&[greedy as u32]), "p_mixture": probe.mixture[greedy]},
            },
            "reply": reply_text,
            "reply_ids": reply.ids,
            "reply_eos": reply.eos,
            "reply_stop": reply.stop_record(),
            "reply_stopped_at": reply.stopped_at,
            "reply_trace": trace_value,
            "copy_stop": copy_stop.map(|report| json!({
                "window_index": report.window_index,
                "copied": report.copied,
                "emitted": report.emitted,
                "dropped": report.dropped,
                "attention": report.attention,
                "gate": report.gate,
                "raw_gate": report.raw_gate,
            })),
        });
        // The field is written only when the rule is enabled, so a rule-off
        // record is the sealed schema exactly, with nothing added.
        if span_extract > 0 {
            record["span_extract"] = span_report
                .map(|report| {
                    json!({
                        "window_index": report.window_index,
                        "anchor": report.anchor,
                        "extracted": report.extracted,
                        "run_start_step": report.run_start_step,
                        "run_steps": report.run_steps,
                        "dropped_prefix": report.dropped_prefix,
                    })
                })
                .unwrap_or(Value::Null);
        }
        records.push(record);
    }
    let mut head = json!({
        "schema": "uor-r4.bind-probe-exact/1",
        "model": model_dir.display().to_string(),
        "parameters": model.parameter_count(),
        "rows_path": rows_path.display().to_string(),
        "tau": TAU,
        "device": device_name,
        "protocol": version,
        "max_new_tokens": max_new_tokens,
        "pointer_gate_floor": gate_floor,
        "pointer_copy_stop": if copy_stop_min_span >= 2 { json!(copy_stop_min_span) } else { Value::Null },
        "pointer_copy_stop_identity": if copy_stop_min_span >= 2 && copy_stop_identity > 0 { json!(1) } else { Value::Null },
        "copy_trace": copy_trace > 0,
        "records": records,
    });
    // The rule is named in the head only when it was enabled, so a rule-off
    // file keeps the sealed head as well as the sealed record schema.
    if span_extract > 0 {
        head["span_extract"] = json!(span_extract);
        head["span_extract_min"] = json!(span_min);
    }
    fs::write(&out, serde_json::to_vec_pretty(&head)?)?;
    println!("{} rows -> {}", rows.len(), out.display());
    Ok(())
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("bind-probe-exact: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
