use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{env, error::Error, fs, path::Path};
use uor_r4_integer::{
    discrete_d11::{ExactCopyState, IntegerDiscreteModel, ROLE_COPY_STOPS},
    paired_role::{RequestedEligibility, RoleSpan},
};
use uor_r4_tokenizer::{
    dialogue::{DialogueProtocol, Message},
    ByteBpeTokenizer,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
const PARENT: &str = "02b922d37308593de956bf9c7c8ee0dbb259d2152528d6993eb9c89be84f65e3";
const TOKENIZER: &str = "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377";
const MAX_TOKENS: usize = 16;
const EOS: u32 = 1;

#[derive(Serialize)]
struct Replay {
    generated_ids: Vec<u32>,
    copy_step_words: Vec<[u32; 8]>,
}

fn replay(model: &IntegerDiscreteModel, original: &[u32], source: Option<u32>) -> Result<Replay> {
    let context_limit = model.shape().context as usize;
    let mut context = original.to_vec();
    let mut cursor = ExactCopyState::new(source, original.len().try_into()?);
    let mut trace = Replay {
        generated_ids: Vec::new(),
        copy_step_words: Vec::new(),
    };
    for _ in 0..MAX_TOKENS {
        let trial = cursor
            .write(original, 0, ROLE_COPY_STOPS)?
            .finish_with_eos(true, EOS);
        let step = if trial.copied != 0 || trial.completed_span() {
            trial
        } else {
            let prediction = model.evaluate_prefix(&context)?.prediction;
            cursor
                .write(original, prediction, ROLE_COPY_STOPS)?
                .finish_with_eos(true, EOS)
        };
        cursor = step.state;
        trace.generated_ids.push(step.emitted);
        trace.copy_step_words.push(step.words());
        if step.emitted == EOS {
            break;
        }
        if context.len() == context_limit {
            context.remove(0);
        }
        context.push(step.emitted);
    }
    Ok(trace)
}

fn grade_span(value: &Value) -> Option<RoleSpan> {
    Some(RoleSpan {
        start: value.get(0)?.as_u64()? as usize,
        end: value.get(1)?.as_u64()? as usize,
    })
}

fn last_supported(
    model: &IntegerDiscreteModel,
    ids: &[u32],
    content: std::ops::Range<usize>,
    original_boundary: Option<usize>,
) -> Result<Value> {
    let native = model.frozen_role_replay()?;
    let context = native
        .requested_context
        .ok_or("requested context missing")?;
    for end in content
        .clone()
        .rev()
        .filter(|&end| native.delimiters.contains(&ids[end]))
    {
        let bounded = content.start..end;
        let trace = native.roles.trace_requested_occurrence(
            ids,
            bounded.clone(),
            0..ids.len(),
            native.delimiters,
            native.assertion_classes,
            context,
        )?;
        if trace.query_boundary == original_boundary
            || !trace
                .candidates
                .iter()
                .any(|candidate| candidate.eligibility == RequestedEligibility::Eligible)
        {
            continue;
        }
        let roles = native.roles.predict_requested_occurrence(
            ids,
            bounded,
            0..ids.len(),
            native.delimiters,
            native.assertion_classes,
            context,
        )?;
        let (decision, assertions) = model.paired_copy_witness(ids, roles, None)?;
        return Ok(json!({
            "content_end": end, "trace": trace, "requested": roles.requested,
            "decision": decision, "assertions": assertions,
            "writer": replay(model, ids, decision.choice.source)?,
        }));
    }
    Ok(Value::Null)
}

fn trace(
    model: &IntegerDiscreteModel,
    tokenizer: &ByteBpeTokenizer,
    ids: &[u32],
    gold_query: Option<RoleSpan>,
    gold_source: Option<u32>,
    compare_last_supported: bool,
) -> Result<Value> {
    let native = model.frozen_role_replay()?;
    let framing = native
        .requested_framing
        .ok_or("requested framing missing")?;
    let content = framing.content(ids)?;
    let roles = model.paired_roles_diagnostic(ids, 0..ids.len(), false)?;
    let trace = native.roles.trace_requested_occurrence(
        ids,
        content.clone(),
        0..ids.len(),
        native.delimiters,
        native.assertion_classes,
        native
            .requested_context
            .ok_or("requested context missing")?,
    )?;
    let mut bytes = Vec::with_capacity(ids.len() + 1);
    let mut offset = 0;
    bytes.push(offset);
    for &id in ids {
        offset += tokenizer.decode_bytes(&[id]).len();
        bytes.push(offset);
    }
    let mut candidates = Vec::new();
    for candidate in trace.candidates {
        let [start, end] = candidate.span;
        let keys = candidate.feature_keys;
        let counts = keys.map(|key| {
            let cells = native.roles.cells();
            cells
                .binary_search_by_key(&key, |cell| cell[0])
                .ok()
                .map(|index| cells[index])
        });
        candidates.push(json!({
            "candidate": candidate,
            "bytes": [bytes[start], bytes[end]],
            "literal": tokenizer.decode(&ids[start..end]),
            "ids": ids[start..end],
            "assertion_classes": ids[start..end].iter()
                .map(|&id| native.assertion_classes[id as usize]).collect::<Vec<_>>(),
            "cells": counts,
        }));
    }
    let (ordinary, assertions) = model.paired_copy_witness(ids, roles, None)?;
    let product = model
        .paired_copy_decision(ids, true)?
        .ok_or("ordinary paired decision missing")?;
    if json!(product) != json!(ordinary) {
        return Err("diagnostic differs from ordinary entry point".into());
    }
    let supplied = gold_query
        .map(|span| -> Result<Value> {
            let (decision, assertions) = model.paired_copy_witness(ids, roles, Some(span))?;
            Ok(json!({
                "decision": decision, "assertions": assertions,
                "writer": replay(model, ids, decision.choice.source)?,
            }))
        })
        .transpose()?;
    let forced = gold_source
        .map(|source| replay(model, ids, Some(source)))
        .transpose()?;
    let fallback = if compare_last_supported && roles.requested.is_none() {
        last_supported(model, ids, content.clone(), trace.query_boundary)?
    } else {
        Value::Null
    };
    Ok(json!({
        "ids": ids, "text": tokenizer.decode(ids),
        "content": trace.content,
        "delimiters": native.delimiters,
        "query_boundary": trace.query_boundary.map(|p| json!({
            "token": p, "byte": bytes[p], "id": ids[p], "literal": tokenizer.decode(&ids[p..p+1]),
            "following": tokenizer.decode(&ids[p+1..content.end]),
        })),
        "gold_query": gold_query, "gold_source": gold_source,
        "candidates": candidates, "role_words": roles.words(),
        "requested": roles.requested, "requested_score": roles.requested_score,
        "requested_runner_up": roles.requested_runner_up,
        "ordinary": ordinary, "ordinary_assertions": assertions,
        "ordinary_writer": replay(model, ids, ordinary.choice.source)?,
        "supplied_query": supplied, "forced_source_writer": forced,
        "last_supported_clause": fallback,
    }))
}

fn grade_writer(tokenizer: &ByteBpeTokenizer, replay: &Value, expected: &[u32]) -> Result<Value> {
    let tape = serde_json::from_value::<Vec<u32>>(replay["generated_ids"].clone())?;
    let visible = tape.strip_suffix(&[EOS]).unwrap_or(&tape);
    Ok(json!({
        "text": tokenizer.decode(visible),
        "ends_eos": tape.ends_with(&[EOS]),
        "exact_tape": tape == expected,
        "first_mismatch": (0..expected.len().max(tape.len()))
            .find(|&i| expected.get(i) != tape.get(i))
            .map(|i| json!({"offset": i, "expected_id": expected.get(i), "generated_id": tape.get(i)})),
    }))
}

fn main() -> Result<()> {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    let compare_last_supported = args.last().is_some_and(|arg| arg == "--last-supported");
    if compare_last_supported {
        args.pop();
    }
    let [model_file, tokenizer_file, grade_file, ordinary_file, clients_dir] = args.as_slice()
    else {
        return Err(
            "usage: d26-ordinary-role-trace PARENT TOKENIZER GRADE ORDINARY CLIENTS_OR_DASH [--last-supported]".into(),
        );
    };
    let model = IntegerDiscreteModel::load(Path::new(model_file))?;
    let tokenizer_bytes = fs::read(tokenizer_file)?;
    let fitted_recurrence = model.artifact_schema() == "uor-r4.lut-discrete-contextual-source/2"
        && model.contextual_source_bindings().is_some_and(|state| {
            state.parent_sha256 == PARENT
                && state.train_sha256 == "77115522738096384bbb8e4eb1213e1ea9ca0b3759a2c88bc0e41f3a5877445b"
                && state.bank_sha256 == "019f76d5cf9dd47d50b41b43a81acdbb3d0a122f196b850802c241060f64d2e5"
        });
    if (model.artifact_sha256() != PARENT && !fitted_recurrence)
        || model.tokenizer_sha256() != Some(TOKENIZER)
        || format!("{:x}", Sha256::digest(&tokenizer_bytes)) != TOKENIZER
    {
        return Err("parent or tokenizer pin mismatch".into());
    }
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes).ok_or("invalid tokenizer")?;
    let protocol = DialogueProtocol::literal_roles_v2(&tokenizer)?;
    let encoder = protocol.bind(&tokenizer)?;
    let encode = |content: &str| {
        encoder
            .encode_assistant_prefix(&[Message {
                role: "user",
                content,
            }])
            .tokens
    };
    if grade_file == "controls" {
        let mut rows = Vec::new();
        for line in fs::read_to_string(ordinary_file)?.lines() {
            let fields = line.splitn(4, '\t').collect::<Vec<_>>();
            let [row, group, gold_text, prompt] = fields.as_slice() else {
                return Err("invalid control row".into());
            };
            let ids = encode(prompt);
            let names = gold_text.split(',').collect::<Vec<_>>();
            let native = model.frozen_role_replay()?;
            let all_candidates = native.roles.trace_where(&ids, 0..ids.len(), |_| true)?;
            let complete_name = |name: &str| {
                all_candidates.iter().rev().find_map(|candidate| {
                    let [start, end] = candidate.span;
                    (tokenizer.decode(&ids[start..end]).trim() == name)
                        .then_some(RoleSpan { start, end })
                })
            };
            let spans = names
                .iter()
                .map(|name| complete_name(name))
                .collect::<Vec<_>>();
            let result = trace(
                &model,
                &tokenizer,
                &ids,
                spans.first().copied().flatten(),
                None,
                compare_last_supported,
            )?;
            rows.push(json!({
                "row": row.parse::<u32>()?, "group": group, "gold_names": names,
                "gold_query_spans": spans, "exact": result,
            }));
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "parent": PARENT, "tokenizer": TOKENIZER, "rows": rows,
            }))?
        );
        return Ok(());
    }
    let grades = serde_json::from_slice::<Vec<Value>>(&fs::read(grade_file)?)?;
    let prompts = serde_json::from_slice::<Vec<Value>>(&fs::read(ordinary_file)?)?;
    let mut rows = Vec::new();
    for grade in grades {
        let row = grade["row"].as_u64().ok_or("row missing")?;
        let prompt = prompts
            .iter()
            .find(|item| item["row"] == row)
            .ok_or("prompt missing")?;
        let text = prompt["messages"][0]["content"]
            .as_str()
            .ok_or("text missing")?;
        let ids = encode(text);
        if json!(ids) != grade["original_ids"] {
            return Err(format!("row {row} original IDs differ").into());
        }
        let span = grade_span(&grade["fact_and_query"][1]);
        let source = grade["gold"].as_u64().map(|id| id as u32);
        let result = trace(
            &model,
            &tokenizer,
            &ids,
            span,
            source,
            compare_last_supported,
        )?;
        let expected = if let Some(source) = source {
            serde_json::from_value::<Vec<u32>>(
                grade["plausible_writer_ids"][source.to_string()].clone(),
            )?
        } else {
            let reply = grade["expected_reply"]
                .as_str()
                .ok_or("refusal text missing")?;
            let mut expected = tokenizer.encode(&format!(" {reply}"));
            expected.push(EOS);
            expected
        };
        let ordinary_reply = grade_writer(&tokenizer, &result["ordinary_writer"], &expected)?;
        if source.is_some()
            && grade_writer(&tokenizer, &result["forced_source_writer"], &expected)?["exact_tape"]
                != true
        {
            return Err(
                format!("row {row} forced writer does not match the frozen gold tape").into(),
            );
        }
        let fallback_reply = if result["last_supported_clause"].is_null() {
            ordinary_reply.clone()
        } else {
            grade_writer(
                &tokenizer,
                &result["last_supported_clause"]["writer"],
                &expected,
            )?
        };
        if clients_dir != "-" {
            let client_path = Path::new(clients_dir).join(format!("{row}.json"));
            let client = serde_json::from_slice::<Value>(&fs::read(client_path)?)?;
            if client["artifact_sha256"] != model.artifact_sha256()
                || client["tokenizer_sha256"] != TOKENIZER
                || client["model_schema"] != model.artifact_schema()
                || client["prompt_ids"] != result["ids"]
                || client["paired"] != result["ordinary"]
                || client["generated_ids"] != result["ordinary_writer"]["generated_ids"]
                || client["copy_step_words"] != result["ordinary_writer"]["copy_step_words"]
                || client["stop"]
                    != if ordinary_reply["ends_eos"] == true {
                        "eos"
                    } else {
                        "max_tokens"
                    }
                || !client["read_enabled"].as_bool().unwrap_or(false)
            {
                return Err(format!("row {row} differs from sealed ordinary client").into());
            }
        }
        let first_question = text
            .split_once('?')
            .map(|(question, _)| format!("{question}?"));
        let diagnostic = first_question
            .filter(|_| !compare_last_supported)
            .map(|question| trace(&model, &tokenizer, &encode(&question), span, source, false))
            .transpose()?;
        rows.push(json!({
            "row": row, "expected_reply": grade["expected_reply"],
            "exact": result, "first_question_only": diagnostic,
            "ordinary_reply": ordinary_reply,
            "last_supported_reply": fallback_reply,
        }));
    }
    let mut record = json!({
        "parent": PARENT, "tokenizer": TOKENIZER,
        "sealed_client_compared": clients_dir != "-", "rows": rows,
    });
    if fitted_recurrence {
        record["recurrence_carrier_sha256"] = json!(model.artifact_sha256());
    }
    println!("{}", serde_json::to_string_pretty(&record)?);
    Ok(())
}
