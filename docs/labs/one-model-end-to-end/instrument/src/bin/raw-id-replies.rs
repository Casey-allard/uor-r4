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
//! `--trace-control 1 --trace-out FILE` writes one control row per generated token: nearest and
//! runner-up distance and margin, EOS rank, turn-memory/point-key moves, Hamming copy/type kept
//! positions, short-cycle state, reply position and the `opaque`, `reflective` or `permeable`
//! class. Each traced reply is also replayed untraced and must match id for id.
//!
//! It is an instrument: it trains nothing, changes no artifact and moves no bar.
//!
//! ```text
//! raw-id-replies --stack ART.uorlut --samples ROWS.json --out REPLIES.jsonl \
//!   --mark-reply 0|1 [--max-tokens 48] [--count-visits 0|1] [--time-steps 0|1] \
//!   [--trace-control 0|1 --trace-out TRACE.jsonl]
//! ```
//!
//! `ROWS.json` is a JSON array of objects with a `prompt` array of token ids (a float score's
//! `samples-*.json`). `REPLIES.jsonl` gets one `{"i", "ids"}` line a row: the greedy ids up to
//! and including the first end-of-reply id, or `--max-tokens` of them. The output file must not
//! exist. A receipt is printed on stdout.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::time::Instant;

use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_integer::generation::short_cycle;
use uor_r4_integer::stack::{HammingStepRead, IntegerStackModel, IntegerStackSession};

const SCHEMA: &str = "uor-r4.raw-id-replies/1";
const EOS: u32 = 1;
const USAGE: &str = "usage: raw-id-replies --stack FILE --samples FILE --out FILE --mark-reply 0|1 [--max-tokens N] [--count-visits 0|1] [--time-steps 0|1] [--trace-control 0|1 --trace-out FILE]";
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

#[derive(Serialize)]
struct ControlTraceRow {
    case: usize,
    reply_position: usize,
    token: u32,
    nearest: NearestTrace,
    eos: EosTrace,
    memory: MemoryTrace,
    reads: ReadTrace,
    short_cycle_period: Option<usize>,
    class: &'static str,
}

#[derive(Serialize)]
struct NearestTrace {
    token: u32,
    distance: u64,
    runner_up: u32,
    runner_up_distance: u64,
    margin: u64,
    selected_by_point_key: bool,
}

#[derive(Serialize)]
struct EosTrace {
    rank: usize,
    distance: u64,
}

#[derive(Serialize)]
struct MemoryTrace {
    turn_memory_present: bool,
    movement: Option<MemoryMoveTrace>,
    point_key: Option<PointKeyTrace>,
}

#[derive(Serialize)]
struct MemoryMoveTrace {
    key: Vec<u32>,
    stored: Option<u32>,
    opening: bool,
}

#[derive(Serialize)]
struct PointKeyTrace {
    token: u32,
    distance: i64,
}

#[derive(Default, Serialize)]
struct ReadTrace {
    reads: usize,
    copy_hits: usize,
    type_hits: usize,
    heads: Vec<ReadHeadTrace>,
}

#[derive(Serialize)]
struct ReadHeadTrace {
    read: usize,
    head: usize,
    kind: &'static str,
    kept: Vec<u32>,
}

#[derive(Default)]
struct TraceTally {
    rows: usize,
    opaque: usize,
    reflective: usize,
    permeable: usize,
    cycles: usize,
}

struct ControlStep<'a> {
    case: usize,
    reply_position: usize,
    served: u32,
    hamming: Vec<HammingStepRead>,
    ids: &'a [u32],
}

fn target_rank(table: &[i16], query: &[i32], target: u32) -> Result<(usize, u64)> {
    let width = query.len();
    if width == 0 || !table.len().is_multiple_of(width) || target as usize >= table.len() / width {
        return Err("EOS rank: query or target outside the writer table".to_owned());
    }
    let distance =
        |token: usize| -> u64 { block_distance(query, &table[token * width..(token + 1) * width]) };
    let target = target as usize;
    let own = distance(target);
    let better = (0..table.len() / width)
        .filter(|&token| token != target)
        .filter(|&token| {
            let other = distance(token);
            other < own || (other == own && token < target)
        })
        .count();
    Ok((better + 1, own))
}

fn read_trace(steps: Vec<HammingStepRead>) -> ReadTrace {
    let mut trace = ReadTrace {
        reads: steps.len(),
        ..ReadTrace::default()
    };
    for (read, step) in steps.into_iter().enumerate() {
        let copy_heads = step.heads.len() / 2;
        for (head, choice) in step.heads.into_iter().enumerate() {
            if choice.kept.is_empty() {
                continue;
            }
            let kind = if head < copy_heads {
                trace.copy_hits += 1;
                "copy"
            } else {
                trace.type_hits += 1;
                "type"
            };
            trace.heads.push(ReadHeadTrace {
                read,
                head,
                kind,
                kept: choice.kept,
            });
        }
    }
    trace
}

fn control_class(memory: &MemoryTrace, reads: &ReadTrace) -> &'static str {
    if memory.movement.as_ref().is_some_and(|m| m.stored.is_some()) || memory.point_key.is_some() {
        "permeable"
    } else if reads.copy_hits > 0 || reads.type_hits > 0 {
        "reflective"
    } else {
        "opaque"
    }
}

fn probe_selection(
    case: usize,
    reply_position: usize,
    served: u32,
    nearest: u32,
    point_key: Option<u32>,
) -> Result<u32> {
    let probe = point_key.unwrap_or(nearest);
    if probe != served {
        return Err(format!(
            "case {case}, reply {reply_position}: probe selected {probe}, served {served}"
        ));
    }
    Ok(probe)
}

fn control_row(
    model: &IntegerStackModel,
    session: &IntegerStackSession<'_>,
    table: &[i16],
    step: ControlStep<'_>,
) -> Result<ControlTraceRow> {
    let ControlStep {
        case,
        reply_position,
        served,
        hamming,
        ids,
    } = step;
    let query = session
        .last_query_q12()
        .ok_or("trace-control needs an FBS writer query")?;
    let [(nearest, nearest_score), (runner_up, runner_up_score)] = model
        .whole_table_nearest_two(query)
        .map_err(|e| format!("case {case}, reply {reply_position}: {e}"))?;
    let distance = |score: i64| -> u64 {
        score
            .checked_neg()
            .and_then(|distance| u64::try_from(distance).ok())
            .unwrap_or(u64::MAX)
    };
    let point_key = session.last_point_key();
    probe_selection(
        case,
        reply_position,
        served,
        nearest,
        point_key.map(|(token, _)| token),
    )?;
    let (eos_rank, eos_distance) = target_rank(table, query, EOS)?;
    let movement = session
        .last_memory_move_state()
        .map(|moved| MemoryMoveTrace {
            key: moved.key,
            stored: moved.stored,
            opening: moved.opening,
        });
    let memory = MemoryTrace {
        turn_memory_present: session.has_turn_memory(),
        movement,
        point_key: point_key.map(|(token, distance)| PointKeyTrace { token, distance }),
    };
    let reads = read_trace(hamming);
    let class = control_class(&memory, &reads);
    Ok(ControlTraceRow {
        case,
        reply_position,
        token: served,
        nearest: NearestTrace {
            token: nearest,
            distance: distance(nearest_score),
            runner_up,
            runner_up_distance: distance(runner_up_score),
            margin: distance(nearest_score).abs_diff(distance(runner_up_score)),
            selected_by_point_key: point_key.is_some(),
        },
        eos: EosTrace {
            rank: eos_rank,
            distance: eos_distance,
        },
        memory,
        reads,
        short_cycle_period: short_cycle(ids),
        class,
    })
}

fn replay_reply(
    model: &IntegerStackModel,
    prompt: &[u32],
    mark_reply: bool,
    max_tokens: usize,
) -> Result<Vec<u32>> {
    let mut session = model.session();
    session.clear_reply_start();
    let mut pick = EOS;
    for (at, &token) in prompt.iter().enumerate() {
        if mark_reply && at + 1 == prompt.len() {
            session.mark_reply_start();
        }
        if at + 1 < prompt.len() {
            session
                .read_step(token)
                .map_err(|e| format!("untraced prompt id {at}: {e}"))?;
        } else {
            let (selected, _) = session
                .serve_step(token)
                .map_err(|e| format!("untraced prompt id {at}: {e}"))?;
            pick = selected;
        }
    }
    let mut ids = Vec::with_capacity(max_tokens);
    loop {
        ids.push(pick);
        if pick == EOS || ids.len() == max_tokens {
            return Ok(ids);
        }
        let (selected, _) = session
            .serve_step(pick)
            .map_err(|e| format!("untraced reply id {}: {e}", ids.len()))?;
        pick = selected;
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
    run_with_args(std::env::args().skip(1))
}

fn run_with_args(args: impl Iterator<Item = String>) -> Result<()> {
    let args: Vec<String> = args.collect();
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
    let trace_control = match flag(&args, "--trace-control").as_deref() {
        None | Some("0") => false,
        Some("1") => true,
        Some(other) => return Err(format!("--trace-control takes 0 or 1, not {other}")),
    };
    let trace_path = flag(&args, "--trace-out");
    if trace_control != trace_path.is_some() {
        return Err("--trace-control 1 and --trace-out must be given together".to_owned());
    }
    if trace_path.as_deref() == Some(out_path.as_str()) {
        return Err("--trace-out must differ from --out".to_owned());
    }

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
    let mut trace = trace_path
        .as_deref()
        .map(|path| {
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .map_err(|e| format!("{path}: {e}"))
        })
        .transpose()?;

    // The product scans the whole table only under the nearest head; under the writer head the
    // count would describe a scan the product never runs.
    let needs_table = count_visits || trace_control;
    let table = match (needs_table, model.fbs_writer_table()) {
        (false, _) => None,
        (true, Some(table)) if model.shape().head_nearest() => Some(table),
        (true, _) => return Err(
            "nearest instrumentation needs an artifact with a writer table and the nearest head"
                .to_owned(),
        ),
    };
    let mut visits = Visits::default();
    let mut clock = StepClock::default();
    let mut trace_tally = TraceTally::default();
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
            if trace_control {
                session.set_hamming_trace(true);
            }
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
            if trace_control {
                let table = table.ok_or("--trace-control 1 needs the writer table")?;
                let reads = session
                    .take_hamming_trace()
                    .ok_or("Hamming trace was not enabled")?;
                let control = control_row(
                    &model,
                    &session,
                    table,
                    ControlStep {
                        case: index,
                        reply_position: ids.len() - 1,
                        served: pick,
                        hamming: reads,
                        ids: &ids,
                    },
                )?;
                trace_tally.rows += 1;
                match control.class {
                    "opaque" => trace_tally.opaque += 1,
                    "reflective" => trace_tally.reflective += 1,
                    "permeable" => trace_tally.permeable += 1,
                    _ => unreachable!("control_class returns one registered class"),
                }
                trace_tally.cycles += usize::from(control.short_cycle_period.is_some());
                let writer = trace
                    .as_mut()
                    .ok_or("--trace-control 1 needs --trace-out")?;
                serde_json::to_writer(&mut *writer, &control)
                    .map_err(|e| format!("{}: {e}", trace_path.as_deref().unwrap_or("trace")))?;
                writer
                    .write_all(b"\n")
                    .map_err(|e| format!("{}: {e}", trace_path.as_deref().unwrap_or("trace")))?;
            }
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
        if trace_control {
            let replay = replay_reply(&model, &prompt, mark, max_tokens)?;
            if replay != ids {
                return Err(format!(
                    "row {index}: traced reply differs from untraced product replay"
                ));
            }
        }
        writeln!(out, "{}", json!({"i": index, "ids": ids}))
            .map_err(|e| format!("{out_path}: {e}"))?;
    }
    out.flush().map_err(|e| format!("{out_path}: {e}"))?;
    if let Some(trace) = trace.as_mut() {
        trace
            .flush()
            .map_err(|e| format!("{}: {e}", trace_path.as_deref().unwrap_or("trace")))?;
    }

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
    if trace_control {
        receipt["reply_control"] = json!({
            "rows": trace_tally.rows,
            "opaque": trace_tally.opaque,
            "reflective": trace_tally.reflective,
            "permeable": trace_tally.permeable,
            "short_cycle_rows": trace_tally.cycles,
        });
    }
    println!("{receipt}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use uor_r4_integer::stack::{
        export_fbs_trunk_uorlut01, synthetic::synthetic_fbs_trunk, synthetic::synthetic_hamming,
        synthetic::synthetic_turn_memory, HammingStepHead,
    };

    fn trace_artifact() -> Vec<u8> {
        let seed = 17;
        let mut snap = synthetic_fbs_trunk(seed);
        snap.readout_nearest = true;
        snap.hamming = Some(synthetic_hamming(
            snap.width,
            snap.layers.len(),
            4,
            seed ^ 0x51a7,
        ));
        snap.memory = Some(synthetic_turn_memory(
            snap.vocab,
            snap.width,
            4,
            192,
            true,
            40,
            seed ^ 0x5eed,
        ));
        export_fbs_trunk_uorlut01(&snap).expect("synthetic artifact")
    }

    #[test]
    fn eos_rank_uses_distance_then_smaller_id() {
        let table = [0, 0, 1, 1, 2, 2, 1, 1];
        assert_eq!(target_rank(&table, &[1, 1], EOS).unwrap(), (1, 0));
        assert_eq!(target_rank(&table, &[1, 1], 3).unwrap(), (2, 0));
        assert_eq!(target_rank(&table, &[0, 0], EOS).unwrap(), (2, 2));
    }

    #[test]
    fn read_trace_splits_copy_and_type_heads() {
        let kept = |positions: &[u32]| HammingStepHead {
            query: 1,
            key: 2,
            kept: positions.to_vec(),
        };
        let trace = read_trace(vec![HammingStepRead {
            depth: 3,
            heads: vec![kept(&[2, 4]), kept(&[]), kept(&[5]), kept(&[])],
        }]);
        assert_eq!(trace.reads, 1);
        assert_eq!(trace.copy_hits, 1);
        assert_eq!(trace.type_hits, 1);
        assert_eq!(trace.heads.len(), 2);
        assert_eq!(trace.heads[0].kind, "copy");
        assert_eq!(trace.heads[1].kind, "type");
    }

    #[test]
    fn permeable_precedes_reflective_and_opaque() {
        let empty_reads = ReadTrace::default();
        let memory = MemoryTrace {
            turn_memory_present: true,
            movement: None,
            point_key: Some(PointKeyTrace {
                token: 7,
                distance: 3,
            }),
        };
        assert_eq!(control_class(&memory, &empty_reads), "permeable");

        let reflective = ReadTrace {
            copy_hits: 1,
            ..ReadTrace::default()
        };
        let no_memory = MemoryTrace {
            turn_memory_present: true,
            movement: None,
            point_key: None,
        };
        assert_eq!(control_class(&no_memory, &reflective), "reflective");
        assert_eq!(control_class(&no_memory, &empty_reads), "opaque");
    }

    #[test]
    fn probe_selection_refuses_a_different_served_token() {
        assert_eq!(probe_selection(0, 0, 4, 4, None).unwrap(), 4);
        assert_eq!(probe_selection(0, 0, 5, 4, Some(5)).unwrap(), 5);
        let error = probe_selection(3, 7, 5, 4, None).unwrap_err();
        assert!(error.contains("probe selected 4, served 5"), "{error}");
    }

    #[test]
    fn short_cycle_reports_the_shortest_terminal_period() {
        assert_eq!(short_cycle(&[9, 9, 9]), Some(1));
        assert_eq!(short_cycle(&[7, 8, 7, 8, 7, 8]), Some(2));
        assert_eq!(short_cycle(&[1, 2, 3, 4]), None);
    }

    #[test]
    fn traced_product_steps_select_and_replay_the_served_tokens() {
        let artifact = trace_artifact();
        let model = IntegerStackModel::parse(&artifact).expect("synthetic model");
        assert!(model.shape().head_nearest());
        let table = model.fbs_writer_table().expect("writer table");
        let prompt = [2, 3];
        let untraced = replay_reply(&model, &prompt, true, 4).expect("untraced reply");
        let mut session = model.session();
        session.clear_reply_start();
        let mut pick = EOS;
        for (at, &token) in prompt.iter().enumerate() {
            if at + 1 < prompt.len() {
                session.read_step(token).expect("read prompt");
                continue;
            }
            session.mark_reply_start();
            session.set_hamming_trace(true);
            let (selected, _) = session.serve_step(token).expect("serve prompt");
            pick = selected;
        }
        let mut traced = Vec::new();
        loop {
            traced.push(pick);
            let hamming = session.take_hamming_trace().expect("Hamming trace");
            let control = control_row(
                &model,
                &session,
                table,
                ControlStep {
                    case: 0,
                    reply_position: traced.len() - 1,
                    served: pick,
                    hamming,
                    ids: &traced,
                },
            )
            .expect("control row");
            assert_eq!(control.token, pick);
            if pick == EOS || traced.len() == 4 {
                break;
            }
            let (selected, _) = session.serve_step(pick).expect("serve reply");
            pick = selected;
        }
        assert_eq!(traced, untraced);
    }

    #[test]
    fn trace_cli_writes_control_rows_without_changing_replies() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("uor-reply-control-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&root).expect("temp directory");
        let stack = root.join("synthetic.uorlut");
        let samples = root.join("samples.json");
        let traced_out = root.join("traced.jsonl");
        let plain_out = root.join("plain.jsonl");
        let trace = root.join("control.jsonl");
        fs::write(&stack, trace_artifact()).expect("artifact");
        fs::write(&samples, r#"[{"prompt":[2,3]}]"#).expect("samples");
        let args = |out: &Path, trace: Option<&Path>| {
            let mut args = vec![
                "--stack".to_owned(),
                stack.display().to_string(),
                "--samples".to_owned(),
                samples.display().to_string(),
                "--out".to_owned(),
                out.display().to_string(),
                "--mark-reply".to_owned(),
                "1".to_owned(),
                "--max-tokens".to_owned(),
                "4".to_owned(),
            ];
            if let Some(trace) = trace {
                args.extend([
                    "--trace-control".to_owned(),
                    "1".to_owned(),
                    "--trace-out".to_owned(),
                    trace.display().to_string(),
                ]);
            }
            args
        };
        run_with_args(args(&traced_out, Some(&trace)).into_iter()).expect("traced CLI");
        run_with_args(args(&plain_out, None).into_iter()).expect("plain CLI");
        assert_eq!(
            fs::read(&traced_out).expect("traced replies"),
            fs::read(&plain_out).expect("plain replies")
        );
        let rows = fs::read_to_string(&trace).expect("control rows");
        assert!(!rows.is_empty());
        for line in rows.lines() {
            let row: Value = serde_json::from_str(line).expect("control JSON");
            for field in [
                "nearest",
                "eos",
                "memory",
                "reads",
                "short_cycle_period",
                "class",
            ] {
                assert!(row.get(field).is_some(), "missing {field}: {row}");
            }
        }
        let _ = fs::remove_dir_all(root);
    }
}
