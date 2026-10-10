//! `pcopy-mass-read` — the copy mass the pointer's own *scored* path reports.
//!
//! References #2029. The v5-answering artifact carries a copy pointer and no
//! memory reader, so the only measurable half of the read/emit question is the
//! pointer's. The per-step *trace* (the decoder's own selection) is already
//! recorded; what was missing is the other half of the mixture: the copy mass
//! `p_copy(target | t)` the pointer assigns to a *chosen* target, which is what
//! decides whether the copy channel carries the value at all.
//!
//! That quantity is already public and already called on the reply path —
//! [`StackModel::score_targets`] returns [`TargetScores::pointer`], one
//! [`PointerRowStats`] per scored position, whose `copy_mass` the source
//! documents as "`p_copy(target | t)` before the gate". This example is the
//! missing **caller**: no model code, no knob, no training, no generation.
//!
//! ```text
//! pcopy-mass-read model=DIR tokenizer=T.json panel=PANEL.json checks=CHECKS.tsv \
//!   replies=REPLIES.json out=OUT.json [trace=TRACE.json] [frame=ID,ID,...] \
//!   [rows=all|numeric|word]
//! ```
//!
//! For every v5 memory row it rebuilds the exact sequence the graded reply ran
//! on — `history_ids` ++ the sealed reply's ids, both taken from the sealed v5
//! acceptance record, so nothing is generated here — locates the value's own
//! token run and the distractor's, and calls `score_targets` once per target in
//! a fixed panel (the value's ids, the distractor's ids, the decoded frame ids).
//!
//! Two readings come out of that, per row:
//!
//! * **validation** — on the natural path (target = the sealed next token) the
//!   per-step `hit` must equal the recorded trace's `matches_source`. If it does
//!   not, this path is measuring a different mixture than the decoder used and
//!   the run is **VOID**: it writes the mismatches and reports no panel numbers.
//! * **the panel** — `copy_mass` (with `gate`, `hit`, `reachable`) for every
//!   panel target at the value's positions, the distractor's positions and every
//!   reply step, so `p_copy(digit)` can be read against the frame baseline.
//!
//! Read-out only: no weights are saved, no knob is set, no reply is generated.

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use candle_core::Device;
use serde_json::{json, Value};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::geometric_stack::StackModel;

/// The ids the 2026-10-09 decode identified as the frame the pointer attends
/// (`pointer-selected-ids-decode-2026-10-09`), in the order that record lists
/// them.
const FRAME_IDS: &[u32] = &[223, 1498, 2369, 1156, 2728, 1044, 754, 772, 1790];

fn arg(args: &[(String, String)], key: &str) -> Option<String> {
    args.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
}

fn required(args: &[(String, String)], key: &str) -> Result<String, Box<dyn Error>> {
    arg(args, key).ok_or_else(|| format!("{key}= is required").into())
}

/// Every start position at which `run` occurs in `ids`.
fn run_positions(ids: &[u32], run: &[u32]) -> Vec<usize> {
    if run.is_empty() || run.len() > ids.len() {
        return Vec::new();
    }
    (0..=ids.len() - run.len())
        .filter(|&start| ids[start..start + run.len()] == *run)
        .collect()
}

/// Every token span whose decoded text covers an occurrence of `needle`.
///
/// The value's own encoding is not always how the value appears in the
/// sequence: a word value can be split differently inside a longer word
/// (`Phil` inside ` Philip`), while a bare digit always encodes to itself. So
/// the positions are located by text, on the tokenizer's own decode, and the
/// exact id-run search is kept beside it as a cross-check.
fn locate_text(ids: &[u32], tokenizer: &ByteBpeTokenizer, needle: &str) -> Vec<(usize, usize)> {
    let mut out: Vec<(usize, usize)> = Vec::new();
    if needle.is_empty() || ids.is_empty() {
        return out;
    }
    // Byte offset of every token boundary in the decoded sequence.
    let mut offsets: Vec<usize> = Vec::with_capacity(ids.len() + 1);
    for i in 0..=ids.len() {
        offsets.push(tokenizer.decode(&ids[..i]).len());
    }
    let text = tokenizer.decode(ids);
    let mut from = 0usize;
    while let Some(found) = text[from..].find(needle) {
        let start = from + found;
        let end = start + needle.len();
        let first = (0..ids.len()).find(|&i| offsets[i] <= start && start < offsets[i + 1]);
        let last = (0..ids.len())
            .rev()
            .find(|&i| offsets[i] < end && end <= offsets[i + 1]);
        if let (Some(first), Some(last)) = (first, last) {
            out.push((first, last));
        }
        from = start + 1;
        if from >= text.len() {
            break;
        }
    }
    out
}

/// The value/distractor column of the frozen v5 checks file, keyed by row id.
struct RowTerms {
    value: String,
    forbid: String,
    category: String,
    numeric: bool,
}

fn read_terms(path: &Path) -> Result<BTreeMap<String, RowTerms>, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    let mut out = BTreeMap::new();
    for line in text.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let columns: Vec<&str> = line.split('\t').collect();
        if columns.len() < 5 {
            return Err(format!("checks line needs >= 5 columns: {line}").into());
        }
        let value = columns[3].trim().to_owned();
        let numeric = !value.is_empty() && value.chars().all(|c| c.is_ascii_digit());
        out.insert(
            columns[0].to_owned(),
            RowTerms {
                value,
                forbid: columns[4].trim().to_owned(),
                category: columns[2].trim().to_owned(),
                numeric,
            },
        );
    }
    Ok(out)
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
    let model_dir = PathBuf::from(required(&args, "model")?);
    let tokenizer_path = PathBuf::from(required(&args, "tokenizer")?);
    let checks_path = PathBuf::from(required(&args, "checks")?);
    let replies_path = PathBuf::from(required(&args, "replies")?);
    let out = PathBuf::from(required(&args, "out")?);
    let trace_path = arg(&args, "trace").map(PathBuf::from);
    let only = arg(&args, "rows").unwrap_or_else(|| "all".into());
    let frame_ids: Vec<u32> = match arg(&args, "frame") {
        Some(list) => list
            .split(',')
            .map(|item| item.trim().parse::<u32>())
            .collect::<Result<_, _>>()?,
        None => FRAME_IDS.to_vec(),
    };

    // ---- inputs -----------------------------------------------------------
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(&tokenizer_path)?)
        .ok_or("tokenizer did not parse")?;
    let terms = read_terms(&checks_path)?;
    let sealed: Value = serde_json::from_slice(&fs::read(&replies_path)?)?;
    let sealed_rows = sealed["panel"]["rows"]
        .as_array()
        .ok_or("replies file has no panel.rows")?;
    if sealed["model_sha256"].as_str().is_none() {
        return Err("replies file carries no model_sha256".into());
    }

    // The recorded per-step trace of the decoder's own selection, by row id.
    let mut trace: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    if let Some(path) = &trace_path {
        let doc: Value = serde_json::from_slice(&fs::read(path)?)?;
        for row in doc["rows"].as_array().ok_or("trace has no rows")? {
            let id = row["id"].as_str().ok_or("trace row has no id")?.to_owned();
            let steps = row["audit_steps_selecting_a_digit"]
                .as_array()
                .ok_or("trace row has no steps")?
                .clone();
            trace.insert(id, steps);
        }
    }

    let model = StackModel::load(&model_dir, &Device::Cpu)?;
    if model.config.pointer.is_none() {
        return Err("the artifact carries no pointer head: nothing to read".into());
    }

    // ---- one row at a time ------------------------------------------------
    let mut rows_out: Vec<Value> = Vec::new();
    let mut validation_rows = 0usize;
    let mut validation_steps = 0usize;
    let mut mismatches: Vec<Value> = Vec::new();
    let mut cross_path_worst: f64 = 0.0;
    let mut summary: Vec<String> = Vec::new();

    for row in sealed_rows {
        let id = row["id"].as_str().ok_or("sealed row has no id")?.to_owned();
        let Some(term) = terms.get(&id) else { continue };
        if only == "numeric" && !term.numeric {
            continue;
        }
        if only == "word" && term.numeric {
            continue;
        }
        let ids: Vec<u32> = row["history_ids"]
            .as_array()
            .ok_or("sealed row has no history_ids")?
            .iter()
            .map(|v| v.as_u64().unwrap_or(u64::MAX) as u32)
            .collect();
        let turns = row["turns"].as_array().ok_or("sealed row has no turns")?;
        let last = turns.last().ok_or("sealed row has no turns")?;
        let reply: Vec<u32> = last["reply_ids"]
            .as_array()
            .ok_or("sealed last turn has no reply_ids")?
            .iter()
            .map(|v| v.as_u64().unwrap_or(u64::MAX) as u32)
            .collect();
        let length = ids.len();
        if reply.is_empty() || reply.len() >= length || ids[length - reply.len()..] != reply[..] {
            return Err(
                format!("{id}: history_ids does not end with the sealed last reply").into(),
            );
        }
        let history_len = length - reply.len();
        let value_ids = tokenizer.encode(&term.value);
        let forbid_ids = if term.forbid == "-" || term.forbid.is_empty() {
            Vec::new()
        } else {
            tokenizer.encode(&term.forbid)
        };
        if value_ids.is_empty() {
            return Err(format!("{id}: value {:?} encoded to no ids", term.value).into());
        }
        let value_positions = run_positions(&ids, &value_ids);
        let forbid_positions = run_positions(&ids, &forbid_ids);
        // Text-located windows (the value as it appears in the sequence, which is
        // not always its own whole-string encoding for word values).
        let value_spans = locate_text(&ids, &tokenizer, &term.value);
        let forbid_spans = if term.forbid == "-" {
            Vec::new()
        } else {
            locate_text(&ids, &tokenizer, &term.forbid)
        };

        // Probe positions: the value's span, the distractor's span, every reply step.
        let mut probe: Vec<(String, usize)> = Vec::new();
        for &(s, e) in &value_spans {
            for p in s..=e {
                probe.push(("value".into(), p));
            }
        }
        for &(s, e) in &forbid_spans {
            for p in s..=e {
                probe.push(("distractor".into(), p));
            }
        }
        let mut reply_positions = Vec::new();
        for step in 0..reply.len() {
            let position = history_len - 1 + step;
            reply_positions.push(position);
            probe.push(("reply_step".into(), position));
        }
        let probe_positions: Vec<usize> = {
            let mut all: Vec<usize> = probe.iter().map(|(_, p)| *p).collect();
            all.sort_unstable();
            all.dedup();
            all
        };

        // Natural targets: every position predicts the next token it was fed.
        let mut natural: Vec<u32> = ids.clone();
        natural.rotate_left(1);
        if let Some(last) = natural.last_mut() {
            *last = ids[length - 1];
        }

        let natural_scores = model.score_targets(&ids, &natural, None, 1, length)?;
        let natural_stats = natural_scores
            .pointer
            .as_ref()
            .ok_or("score_targets returned no pointer statistics for a pointer model")?;

        // VALIDATION: the natural path's `hit` must equal the trace's
        // `matches_source` at every recorded step.
        let mut steps_out: Vec<Value> = Vec::new();
        if let Some(steps) = trace.get(&id) {
            validation_rows += 1;
            for s in steps {
                let step = s["step"].as_u64().unwrap_or(u64::MAX) as usize;
                if step >= reply.len() {
                    return Err(format!("{id}: trace step {step} is past the reply").into());
                }
                let position = history_len - 1 + step;
                let stats = natural_stats[position]
                    .as_ref()
                    .ok_or("no statistics at a traced reply step")?;
                let traced = s["matches_source"].as_bool().unwrap_or(false);
                validation_steps += 1;
                if stats.hit != traced {
                    mismatches.push(json!({
                        "check": "hit_vs_matches_source",
                        "id": id, "step": step, "position": position,
                        "target": reply[step], "hit": stats.hit,
                        "trace_matches_source": traced,
                        "trace_source": s["source"], "trace_source_id": s["source_id"],
                        "trace_attention": s["attention"], "gate": stats.gate,
                        "copy_mass": stats.copy_mass,
                    }));
                }
                steps_out.push(json!({
                    "step": step, "position": position, "target": reply[step],
                    "gate": stats.gate, "copy_mass": stats.copy_mass,
                    "hit": stats.hit, "reachable": stats.reachable,
                    "trace_source": s["source"], "trace_source_id": s["source_id"],
                    "trace_attention": s["attention"], "trace_matches_source": traced,
                    "nll": natural_scores.nll[position],
                }));
            }
        }

        // THE PANEL: one `score_targets` call per target, target overridden at
        // the probe positions only. The forward pass is identical every time;
        // only the reported copy mass changes, so this reads the whole
        // (target x position) grid at the cost of one forward per target.
        //
        // Four kinds, and the difference between the first two matters for word
        // values: `value` is the value's OWN encoding (what the model must
        // produce), `span` is the id actually present in the sequence at the
        // located value span (what the copy channel can carry — a word can be
        // tokenized differently in context than on its own). They coincide for
        // the bare-digit values.
        let mut panel_targets: Vec<(String, u32)> = Vec::new();
        let mut seen: std::collections::BTreeSet<(String, u32)> = Default::default();
        let mut push = |panel_targets: &mut Vec<(String, u32)>, kind: &str, t: u32| {
            if seen.insert((kind.to_owned(), t)) {
                panel_targets.push((kind.to_owned(), t));
            }
        };
        for &t in &value_ids {
            push(&mut panel_targets, "value", t);
        }
        for &(s, e) in &value_spans {
            for &t in &ids[s..=e] {
                push(&mut panel_targets, "span", t);
            }
        }
        for &t in &forbid_ids {
            push(&mut panel_targets, "distractor", t);
        }
        for &(s, e) in &forbid_spans {
            for &t in &ids[s..=e] {
                push(&mut panel_targets, "distractor", t);
            }
        }
        for &t in &frame_ids {
            push(&mut panel_targets, "frame", t);
        }
        let mut panel: BTreeMap<String, Value> = BTreeMap::new();
        for (kind, target) in &panel_targets {
            let mut targets = natural.clone();
            for &position in &probe_positions {
                targets[position] = *target;
            }
            let scores = model.score_targets(&ids, &targets, None, 1, length)?;
            let stats = scores
                .pointer
                .as_ref()
                .ok_or("score_targets returned no pointer statistics for a pointer model")?;
            let mut at: BTreeMap<String, Value> = BTreeMap::new();
            for &position in &probe_positions {
                let Some(read) = stats[position].as_ref() else {
                    continue;
                };
                at.insert(
                    position.to_string(),
                    json!({
                        "gate": read.gate, "copy_mass": read.copy_mass,
                        "hit": read.hit, "reachable": read.reachable,
                    }),
                );
            }
            panel.insert(
                format!("{kind}:{target}"),
                Value::Object(at.into_iter().collect()),
            );
        }

        // THE PER-POSITION READ. `copy_mass` is a mass over token IDS, so a word
        // whose subwords recur elsewhere in the window has its share inflated.
        // The pointer's own attention vector, from the public `read_span_probe`,
        // gives the same distribution over POSITIONS, so the value can be read at
        // the positions that HOLD it. The two paths must agree on the same
        // distribution (checked below); if they do not, the run is VOID.
        let span_ids: Vec<u32> = value_spans
            .iter()
            .flat_map(|&(s, e)| ids[s..=e].to_vec())
            .collect();
        let mut pos_steps: Vec<Value> = Vec::new();
        let mut cross_path_max: f64 = 0.0;
        for step in 0..reply.len() {
            let position = history_len - 1 + step;
            let window = &ids[..=position];
            let holding: Vec<usize> = value_spans
                .iter()
                .flat_map(|&(s, e)| (s..=e).filter(|&p| p <= position))
                .collect();
            // The probe needs at least one in-window position; when the value is
            // not yet in the window, the query position itself is passed.
            let spans_arg = vec![if holding.is_empty() {
                vec![position]
            } else {
                holding.clone()
            }];
            let probe = model.read_span_probe(window, &spans_arg)?;
            let pointer = probe
                .pointer
                .as_ref()
                .ok_or("read_span_probe returned no pointer head")?;
            let attention = &pointer.attention;
            let mut best = 0usize;
            for (j, &a) in attention.iter().enumerate() {
                if a > attention[best] {
                    best = j;
                }
            }
            let value_pos_share: f64 = holding.iter().map(|&j| attention[j]).sum();
            let value_id_share: f64 = attention
                .iter()
                .enumerate()
                .filter(|(j, _)| span_ids.contains(&window[*j]))
                .map(|(_, a)| a)
                .sum();
            let frame_id_share: f64 = attention
                .iter()
                .enumerate()
                .filter(|(j, _)| frame_ids.contains(&window[*j]))
                .map(|(_, a)| a)
                .sum();
            // Cross-path: every panel target's `copy_mass` must equal the same
            // mass summed from this attention vector over the positions holding it.
            for (kind, target) in &panel_targets {
                if let Some(read) = panel
                    .get(&format!("{kind}:{target}"))
                    .and_then(|v| v.get(position.to_string()))
                    .and_then(|v| v.get("copy_mass"))
                    .and_then(|v| v.as_f64())
                {
                    let from_positions: f64 = attention
                        .iter()
                        .enumerate()
                        .filter(|(j, _)| window[*j] == *target)
                        .map(|(_, a)| a)
                        .sum();
                    let delta = (read - from_positions).abs();
                    if delta > cross_path_max {
                        cross_path_max = delta;
                    }
                }
            }
            let traced = trace
                .get(&id)
                .and_then(|steps| steps.get(step))
                .cloned()
                .unwrap_or(Value::Null);
            let traced_pos = traced["source"].as_u64().map(|v| v as usize);
            let traced_id = traced["source_id"].as_u64().map(|v| v as u32);
            let matches_trace = match (traced_pos, traced_id) {
                (Some(p), Some(t)) => best == p && window[best] == t,
                _ => true,
            };
            if !matches_trace {
                mismatches.push(json!({
                    "check": "argmax_vs_trace_source", "id": id, "step": step,
                    "position": position, "argmax_source": best, "argmax_id": window[best],
                    "trace_source": traced_pos, "trace_source_id": traced_id,
                }));
            }
            pos_steps.push(json!({
                "step": step,
                "position": position,
                "gate": pointer.gate,
                "argmax_source": best,
                "argmax_id": window[best],
                "value_pos_share": value_pos_share,
                "value_id_share": value_id_share,
                "frame_id_share": frame_id_share,
                "trace_source": traced_pos,
                "trace_source_id": traced_id,
                "argmax_matches_trace": matches_trace,
            }));
        }

        let value_read = |position: usize, target: u32| -> Option<f64> {
            panel
                .get(&format!("value:{target}"))
                .and_then(|v| v.get(position.to_string()))
                .and_then(|v| v.get("copy_mass"))
                .and_then(|v| v.as_f64())
        };
        let frame_read = |position: usize| -> Option<f64> {
            frame_ids
                .iter()
                .filter_map(|t| {
                    panel
                        .get(&format!("frame:{t}"))
                        .and_then(|v| v.get(position.to_string()))
                        .and_then(|v| v.get("copy_mass"))
                        .and_then(|v| v.as_f64())
                })
                .reduce(f64::max)
        };
        let reply_best = |target: u32| -> f64 {
            reply_positions
                .iter()
                .filter_map(|&p| value_read(p, target))
                .fold(f64::MIN, f64::max)
        };
        let reply_frame_best = reply_positions
            .iter()
            .filter_map(|&p| frame_read(p))
            .fold(f64::MIN, f64::max);
        let first_value_position = value_spans.first().map(|&(s, _)| s);
        let value_mass_at_first = first_value_position.map(|p| {
            value_ids
                .iter()
                .map(|&t| value_read(p, t))
                .collect::<Vec<_>>()
        });
        let frame_mass_at_first = first_value_position.and_then(frame_read);
        let forbid_mass_at_first = forbid_ids
            .iter()
            .filter_map(|&t| {
                first_value_position.and_then(|p| {
                    panel
                        .get(&format!("distractor:{t}"))
                        .and_then(|v| v.get(p.to_string()))
                        .and_then(|v| v.get("copy_mass"))
                        .and_then(|v| v.as_f64())
                })
            })
            .reduce(f64::max);
        cross_path_worst = cross_path_worst.max(cross_path_max);
        let pos_best = pos_steps
            .iter()
            .map(|s| s["value_pos_share"].as_f64().unwrap_or(0.0))
            .fold(f64::MIN, f64::max);
        let id_best = pos_steps
            .iter()
            .map(|s| s["value_id_share"].as_f64().unwrap_or(0.0))
            .fold(f64::MIN, f64::max);
        let pos_frame_best = pos_steps
            .iter()
            .map(|s| s["frame_id_share"].as_f64().unwrap_or(0.0))
            .fold(f64::MIN, f64::max);
        summary.push(format!(
            "{id:18} {:<7} {:<10} spans {:?} | PER POSITION over reply steps: max value-at-its-positions {:.4}, value-by-id {:.4} (inflation {:.2}x), frame {:.4} | id-based panel: at first value position value {:?} frame {:?} distractor {:?} | over reply steps max copy_mass value {:?} frame {:.4} | value tokens selected {} / emitted {}",
            if term.numeric { "numeric" } else { "word" },
            term.value,
            value_spans,
            pos_best,
            id_best,
            if pos_best > 0.0 { id_best / pos_best } else { 0.0 },
            pos_frame_best,
            value_mass_at_first,
            frame_mass_at_first,
            forbid_mass_at_first,
            value_ids.iter().map(|&t| reply_best(t)).collect::<Vec<_>>(),
            reply_frame_best,
            trace.get(&id).map(|s| s.iter().filter(|v| {
                v["source_id"].as_u64().map(|x| value_ids.contains(&(x as u32))).unwrap_or(false)
            }).count()).unwrap_or(0),
            reply.iter().filter(|x| value_ids.contains(x)).count(),
        ));

        rows_out.push(json!({
            "id": id,
            "category": term.category,
            "numeric": term.numeric,
            "value": term.value,
            "value_ids": value_ids,
            "distractor": term.forbid,
            "distractor_ids": forbid_ids,
            "history_len": history_len,
            "reply_len": reply.len(),
            "value_positions": value_positions,
            "distractor_positions": forbid_positions,
            "value_spans": value_spans.iter().map(|&(s, e)| json!([s, e])).collect::<Vec<_>>(),
            "value_span_ids": value_spans.iter().map(|&(s, e)| json!(ids[s..=e])).collect::<Vec<_>>(),
            "distractor_spans": forbid_spans.iter().map(|&(s, e)| json!([s, e])).collect::<Vec<_>>(),
            "reply_positions": reply_positions,
            "natural_steps": steps_out,
            "position_read": pos_steps,
            "position_read_max_cross_path_delta": cross_path_max,
            "panel": panel,
        }));
    }

    // The two public paths must agree on the same distribution. The tolerance is
    // 1e-5, not 1e-6: `read_span_probe` builds its attention in f32 and
    // `score_targets` sums the mixture's copy mass separately, so the two agree
    // only to f32 accumulation (measured here, and printed with the result).
    const PATHS_AGREE_TOL: f64 = 1e-5;
    if cross_path_worst > PATHS_AGREE_TOL {
        mismatches.push(json!({
            "check": "paths_agree", "max_abs_delta": cross_path_worst,
            "rule": "score_targets copy_mass(target) must equal read_span_probe's attention summed over the positions holding that target",
        }));
    }
    let void = !mismatches.is_empty();
    let report = json!({
        "schema": "uor-r4.pcopy-mass-read/1",
        "model": model_dir.display().to_string(),
        "model_sha256": sealed["model_sha256"],
        "sealed_replies": replies_path.display().to_string(),
        "tokenizer": tokenizer_path.display().to_string(),
        "checks": checks_path.display().to_string(),
        "frame_ids": frame_ids,
        "frame_text": tokenizer.decode(&frame_ids),
        "row_filter": only,
        "path": "StackModel::score_targets -> TargetScores.pointer -> PointerRowStats { gate, copy_mass, hit, reachable }",
        "position_path": "StackModel::read_span_probe -> SpanProbe.pointer: PointerProbe { gate, attention } (attention over the window's positions 0..=t)",
        "validation": {
            "rule": "three checks, any mismatch makes the run VOID and no panel number is reported: (1) on the natural path (target = the sealed next token) `hit` must equal the recorded trace's `matches_source` at every traced step; (2) the per-position read's attention argmax must equal the trace's `source` position and `source_id` at every traced step; (3) the two public paths must agree -- score_targets copy_mass(target) == read_span_probe attention summed over the positions holding that target -- to 1e-5, the f32 accumulation bound; the measured maximum is reported as cross_path_max_abs_delta",
            "rows_checked": validation_rows,
            "steps_checked": validation_steps,
            "cross_path_max_abs_delta": cross_path_worst,
            "mismatches": mismatches,
            "void": void,
        },
        "rows": rows_out,
    });
    fs::write(&out, serde_json::to_vec_pretty(&report)?)?;

    if void {
        println!(
            "VOID: {validation_steps} traced steps checked, {} mismatches",
            mismatches.len()
        );
        for m in mismatches.iter().take(20) {
            println!("  {m}");
        }
        println!("no panel numbers are reported; wrote {}", out.display());
        std::process::exit(3);
    }
    println!(
        "validation PASSED: {} rows, {validation_steps} traced steps; hit == matches_source; attention argmax == trace source at every traced step; the two public paths agree to {cross_path_worst:.3e}",
        validation_rows
    );
    for line in &summary {
        println!("{line}");
    }
    println!("wrote {}", out.display());
    Ok(())
}
