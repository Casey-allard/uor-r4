//! The non-relation prose panel, graded on the GPU through the prime-sieve
//! frame compiler (`m3-nonrelation-prose-panel`, #2856).
//!
//! `controls --out DIR` runs the finite CUDA controls: hand-built frames and
//! turns with literal expected decisions. Any mismatch exits 1.
//!
//! `run --repo-root R --out DIR --expect-table SHA --expect-ptx SHA
//! [--seed N] [--values-per-phrasing N] [--device N]` compiles the frozen
//! panel and both recall controls in one launch, twice, and writes
//! `record.json` and `decisions.json`. A pin mismatch, a nondeterministic launch or a malformed
//! decision exits 1; the rates themselves never change the exit code.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde_json::{json, Value};
use uor_r4_training::milestone_world::Split;
use uor_r4_training::nonrelation_prose_panel::{
    grade_prose_panel, world_v2_relation_labels, ProseCategory, ProsePanel, ProseSplit,
};
use uor_r4_training::sieve_frame_compiler::device::{ptx_sha256, SieveDevice};
use uor_r4_training::sieve_frame_compiler::{
    world_v2_controls, ControlTurn, DecidedCompiler, Decision, DeviceFrames, DeviceTurns,
    FrameTable, MAX_VALUE_WORDS, REGISTERED_SEED, REGISTERED_VALUES_PER_PHRASING,
};
use uor_r4_training::{sha256_bytes, turn_action::CompiledAction};

type Result<T> = std::result::Result<T, String>;

fn main() {
    if let Err(error) = run(std::env::args().skip(1).collect()) {
        eprintln!("prose-panel-sieve: {error}");
        std::process::exit(1);
    }
}

fn run(args: Vec<String>) -> Result<()> {
    let (mode, rest) = args
        .split_first()
        .ok_or("usage: prose-panel-sieve controls|run ...")?;
    let mut flags = BTreeMap::new();
    for pair in rest.chunks(2) {
        match pair {
            [key, value] if key.starts_with("--") => {
                flags.insert(key.trim_start_matches("--").to_string(), value.clone());
            }
            _ => return Err(format!("expected --flag value pairs, got {pair:?}")),
        }
    }
    let get = |key: &str| flags.get(key).cloned();
    let number = |key: &str, default: u64| -> Result<u64> {
        get(key).map_or(Ok(default), |v| {
            v.parse().map_err(|e| format!("--{key}: {e}"))
        })
    };
    let out = PathBuf::from(get("out").ok_or("--out is required")?);
    fs::create_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    let device = SieveDevice::new(number("device", 0)? as usize).map_err(|e| e.to_string())?;
    let record = match mode.as_str() {
        "controls" => controls(&device)?,
        "run" => {
            let (record, decisions) = measure(
                &device,
                PathBuf::from(get("repo-root").ok_or("--repo-root is required")?),
                &get("expect-table").ok_or("--expect-table is required")?,
                &get("expect-ptx").ok_or("--expect-ptx is required")?,
                number("seed", REGISTERED_SEED)?,
                number("values-per-phrasing", REGISTERED_VALUES_PER_PHRASING as u64)? as usize,
            )?;
            fs::write(out.join("decisions.json"), decisions)
                .map_err(|e| format!("decisions.json: {e}"))?;
            record
        }
        other => return Err(format!("unknown mode {other:?}")),
    };
    let name = if mode == "run" {
        "record.json"
    } else {
        "controls.json"
    };
    let bytes = serde_json::to_vec_pretty(&record).map_err(|e| e.to_string())?;
    fs::write(out.join(name), &bytes).map_err(|e| format!("{name}: {e}"))?;
    println!(
        "{}",
        serde_json::to_string(&record["headline"]).map_err(|e| e.to_string())?
    );
    if record["instrument_ok"] != json!(true) {
        return Err("an instrument check failed; see the record".into());
    }
    Ok(())
}

/// One hand-built case: a turn's word IDs and the decision it must get.
struct Case {
    name: &'static str,
    turn: &'static [u32],
    want: Decision,
}

const fn decision(
    status: u32,
    frame: u32,
    hole_start: u32,
    hole_len: u32,
    survivors: u32,
) -> Decision {
    Decision {
        status,
        frame,
        hole_start,
        hole_len,
        survivors,
    }
}

fn controls(device: &SieveDevice) -> Result<Value> {
    // Words 1..=8, one 64-bit signature word. Frames:
    // 0: [1 2 3] {v} []          expert 14 (assert)
    // 1: [4 5] query             expert 15
    // 2: [6] {v} [7]             expert 21
    // 3: [1 2 8] {v} []          expert 22
    // 4: [1 2 8] {v} []          expert 33 (same words as 3, another expert)
    // 5: [6] {v} [7]             expert 21 (a duplicate of 2)
    let frames: [(&[u32], &[u32], bool, u64); 6] = [
        (&[1, 2, 3], &[], true, 14),
        (&[4, 5], &[], false, 15),
        (&[6], &[7], true, 21),
        (&[1, 2, 8], &[], true, 22),
        (&[1, 2, 8], &[], true, 33),
        (&[6], &[7], true, 21),
    ];
    let mut flat = DeviceFrames {
        sig_words: 1,
        ..DeviceFrames::default()
    };
    for (prefix, suffix, slot, expert) in frames {
        let ids: Vec<u32> = prefix.iter().chain(suffix).copied().collect();
        flat.signature.push(bits(&ids));
        flat.offset.push(flat.tokens.len() as u32);
        flat.tokens.extend(&ids);
        flat.prefix.push(prefix.len() as u32);
        flat.suffix.push(suffix.len() as u32);
        flat.slot.push(u32::from(slot));
        flat.expert.push(expert);
    }
    let cases = [
        Case {
            name: "assert frame, one-word value",
            turn: &[1, 2, 3, 0],
            want: decision(1, 0, 3, 1, 1),
        },
        Case {
            name: "assert frame, four-word value",
            turn: &[1, 2, 3, 0, 0, 0, 0],
            want: decision(1, 0, 3, 4, 1),
        },
        Case {
            name: "value past the cap",
            turn: &[1, 2, 3, 0, 0, 0, 0, 0],
            want: decision(0, 0, 0, 0, 0),
        },
        Case {
            name: "empty value",
            turn: &[1, 2, 3],
            want: decision(0, 0, 0, 0, 0),
        },
        Case {
            name: "sieve rejects a missing frame word",
            turn: &[1, 0, 3, 0],
            want: decision(0, 0, 0, 0, 0),
        },
        Case {
            name: "sieve passes, order fails",
            turn: &[2, 1, 3, 0],
            want: decision(0, 0, 0, 0, 0),
        },
        Case {
            name: "query exact",
            turn: &[4, 5],
            want: decision(1, 1, 0, 0, 1),
        },
        Case {
            name: "query with an extra word",
            turn: &[4, 5, 0],
            want: decision(0, 0, 0, 0, 0),
        },
        Case {
            name: "duplicate frames agree",
            turn: &[6, 0, 0, 7],
            want: decision(1, 2, 1, 2, 2),
        },
        Case {
            name: "two experts disagree",
            turn: &[1, 2, 8, 0],
            want: decision(2, 3, 3, 1, 2),
        },
        Case {
            name: "a value may hold frame words",
            turn: &[1, 2, 3, 4, 5],
            want: decision(1, 0, 3, 2, 1),
        },
        Case {
            name: "empty turn",
            turn: &[],
            want: decision(0, 0, 0, 0, 0),
        },
    ];
    let mut turns = DeviceTurns::default();
    for case in &cases {
        turns.signature.push(bits(case.turn));
        turns.offset.push(turns.tokens.len() as u32);
        turns.length.push(case.turn.len() as u32);
        turns.tokens.extend(case.turn);
    }
    let (got, micros) = device
        .compile(&flat, &turns, MAX_VALUE_WORDS)
        .map_err(|e| e.to_string())?;
    let mut rows = Vec::new();
    let mut failed = 0usize;
    for (case, got) in cases.iter().zip(&got) {
        let ok = *got == case.want;
        failed += usize::from(!ok);
        rows.push(json!({"case": case.name, "ok": ok, "want": case.want, "got": got}));
    }
    Ok(json!({
        "schema": "uor-r4.prose-panel-sieve.controls/1",
        "ptx_sha256": ptx_sha256(),
        "instrument_ok": failed == 0,
        "headline": {"controls": cases.len(), "failed": failed, "micros": micros},
        "cases": rows,
    }))
}

/// The squarefree prime product of `ids` (all below 64) as its one-word bitset.
fn bits(ids: &[u32]) -> u64 {
    ids.iter()
        .filter(|&&id| id != 0)
        .fold(0, |acc, &id| acc | 1u64 << (id - 1))
}

fn measure(
    device: &SieveDevice,
    repo_root: PathBuf,
    expect_table: &str,
    expect_ptx: &str,
    seed: u64,
    values_per_phrasing: usize,
) -> Result<(Value, Vec<u8>)> {
    let err = |e: &dyn std::fmt::Display| e.to_string();
    let table = FrameTable::world_v2().map_err(|e| err(&e))?;
    let artifact = table.canonical_bytes().map_err(|e| err(&e))?;
    let table_sha = sha256_bytes(&artifact);
    let ptx_sha = ptx_sha256();
    let pins = [
        ("frame_table_sha256", table_sha == expect_table),
        ("ptx_sha256", ptx_sha == expect_ptx),
        (
            "relation_labels",
            table.relations == world_v2_relation_labels(),
        ),
    ];
    if let Some((name, _)) = pins.iter().find(|(_, ok)| !ok) {
        return Err(format!("pin {name} does not match; nothing was run"));
    }
    let panel = ProsePanel::load_frozen(&repo_root).map_err(|e| err(&e))?;
    let train = world_v2_controls(Split::Train, values_per_phrasing, seed).map_err(|e| err(&e))?;
    let development =
        world_v2_controls(Split::Development, values_per_phrasing, seed).map_err(|e| err(&e))?;
    let sources: Vec<String> = panel
        .rows
        .iter()
        .map(|row| row.text.clone())
        .chain(train.iter().chain(&development).map(|c| c.source.clone()))
        .collect();
    let (split, turns) = table.encode_turns(&sources).map_err(|e| err(&e))?;
    let frames = table.device_frames().map_err(|e| err(&e))?;
    let (first, micros) = device
        .compile(&frames, &turns, table.max_value_words)
        .map_err(|e| err(&e))?;
    let (second, micros_again) = device
        .compile(&frames, &turns, table.max_value_words)
        .map_err(|e| err(&e))?;
    let deterministic = first == second;
    // Every turn's decision, in launch order: the artifact a second build is
    // compared on (timings excluded).
    let decision_rows: Vec<Value> = sources
        .iter()
        .zip(&first)
        .map(|(source, decision)| json!({"source": source, "decision": decision}))
        .collect();
    let decisions = serde_json::to_vec(&decision_rows).map_err(|e| err(&e))?;
    let decisions_sha = sha256_bytes(&decisions);
    let mut actions = Vec::with_capacity(sources.len());
    for (words, decision) in split.iter().zip(&first) {
        actions.push(table.decode(words, *decision).map_err(|e| err(&e))?);
    }
    let panel_actions: BTreeMap<String, CompiledAction> = panel
        .rows
        .iter()
        .zip(&actions)
        .map(|(row, action)| (row.text.clone(), action.clone()))
        .collect();
    let compiler = DecidedCompiler::new(&table, panel_actions).map_err(|e| err(&e))?;
    let report = grade_prose_panel(&panel, &compiler, true, true).map_err(|e| err(&e))?;
    let n = panel.rows.len();
    let train_score = recall(&train, &actions[n..n + train.len()]);
    let development_score = recall(&development, &actions[n + train.len()..]);
    let tokens: u64 = turns.length.iter().map(|&l| u64::from(l)).sum();
    let tokens_per_second = tokens * 1_000_000 / micros.max(1);
    let by_category = |map: &BTreeMap<ProseCategory, usize>| counts(map, |k| k.as_str());
    let by_split = |map: &BTreeMap<ProseSplit, usize>| counts(map, |k| k.as_str());
    let panel_turns: Vec<Value> = report
        .turns
        .iter()
        .zip(&first)
        .map(|(grade, decision)| {
            json!({
                "id": grade.id,
                "split": grade.split.as_str(),
                "category": grade.category.as_str(),
                "action": grade.action,
                "memory": grade.memory,
                "verdict": format!("{:?}", grade.verdict),
                "overwrote_stored_fact": grade.overwrote_stored_fact,
                "decision": decision,
            })
        })
        .collect();
    let floor_met = train_score.exact * 100 >= train_score.total * 95;
    let bar_met = report.false_overwrite_count == 0
        && report.false_assert_count == 0
        && report.clean_unresolved_count == n
        && floor_met;
    let record = json!({
        "schema": "uor-r4.prose-panel-sieve.record/1",
        "instrument_ok": deterministic,
        "headline": {
            "decisions_sha256": decisions_sha,
            "tokens_per_second": tokens_per_second,
            "turns": sources.len(),
            "tokens": tokens,
            "launch_micros": micros,
            "false_assert": [report.false_assert_count, n],
            "false_overwrite": [report.false_overwrite_count, n],
            "clean_unresolved": [report.clean_unresolved_count, n],
            "recall_train_phrasings": [train_score.exact, train_score.total],
            "recall_development_phrasings": [development_score.exact, development_score.total],
            "recall_floor_met": floor_met,
            "registered_bar_met": bar_met,
        },
        "pins": {
            "panel_sha256": report.panel_sha256,
            "frame_table_sha256": table_sha,
            "compiler_artifact_sha256": report.compiler_artifact_sha256,
            "ptx_sha256": ptx_sha,
            "frames": table.frames.len(),
            "lexicon_words": table.lexicon.len(),
            "max_value_words": table.max_value_words,
            "seed": seed,
            "values_per_phrasing": values_per_phrasing,
        },
        "instrument": {
            "deterministic_second_launch": deterministic,
            "second_launch_micros": micros_again,
        },
        "panel": {
            "false_correct_action": report.false_correct_action_count,
            "false_query": report.false_query_count,
            "per_category_false_assert": by_category(&report.per_category_false_assert),
            "per_category_false_overwrite": by_category(&report.per_category_false_overwrite),
            "per_split_false_assert": by_split(&report.per_split_false_assert),
            "per_split_false_overwrite": by_split(&report.per_split_false_overwrite),
            "turns": panel_turns,
        },
        "recall_train_phrasings": train_score.detail,
        "recall_development_phrasings": development_score.detail,
    });
    Ok((record, decisions))
}

/// A count map as a JSON object keyed by each key's name.
fn counts<K>(map: &BTreeMap<K, usize>, name: impl Fn(&K) -> &'static str) -> Value {
    Value::Object(
        map.iter()
            .map(|(k, v)| (name(k).to_string(), json!(v)))
            .collect(),
    )
}

struct Recall {
    exact: u64,
    total: u64,
    detail: Value,
}

/// Exact-match recall of `controls` against the decoded device actions.
fn recall(controls: &[ControlTurn], actions: &[CompiledAction]) -> Recall {
    let mut exact = 0u64;
    let mut misses = Vec::new();
    for (control, action) in controls.iter().zip(actions) {
        if *action == control.expected {
            exact += 1;
        } else {
            misses.push(json!({
                "source": control.source,
                "template": control.template,
                "expected": control.expected,
                "got": action,
            }));
        }
    }
    let total = controls.len() as u64;
    Recall {
        exact,
        total,
        detail: json!({"exact": exact, "total": total, "misses": misses}),
    }
}
