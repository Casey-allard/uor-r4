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
//! `--place-rows 1` (claude-352) needs an artifact with a place read and adds `"place"` to each
//! reply line: one `[held, offered]` pair a generated id, each a number or `null`, read from
//! `IntegerStackSession::last_place` right after the `serve_step` that chose that id. That is
//! the place read's register after that step, in `last_place`'s own words: "the place it holds,
//! if it is set, and the place whose token that step's readout was offered, if its row was
//! open". Pair `r` is so the register after stepping the token before reply id `r` (the prompt's
//! last id for `r = 0`, reply id `r - 1` after), the step whose readout chose id `r`. A place is
//! a session position, which is the index into the row's prompt: a row's session starts empty.
//!
//! `--places KIT` (needs `--place-rows 1`) reads a place kit, the
//! `[reply start, place, value start, value length]` labels `train-in-space --turn-census
//! --places-out` writes in corpus positions, and for each row with a `position` (the corpus
//! position of the reply's first target, which is a kit reply start) adds `"value_row"`, the
//! reply row at which the value's first token is chosen (value start − `position`), and
//! `"true_place"`, the labelled place as an index into the prompt (place − (`position` − prompt
//! length)). A reply the kit does not label gets `null` for both; a place before the prompt's
//! first token gets `"true_place": null` and `"true_place_outside": true`. The receipt gains
//! `place_rows`: the rows with a `position`, those with a true place, those whose place is
//! outside the prompt, and the kit's hash and the corpus hash it names. Without `--kit-tokens`
//! the kit's corpus hash is recorded and not checked (`"kit_tokens_checked": false`): the
//! reader has no corpus. `--kit-tokens TOKENS.u16` (needs `--places`) gives it the corpus: the
//! file's SHA-256 must be the hash the kit names, and each row's `reference` must be the file's
//! ids at the row's `position`, or the run is refused before anything is written.
//!
//! `--copy-rule NAME` (claude-355) serves the artifact's copy heads under another rule than its
//! own: `mean`, `latest-exact`, `latest-nearest`, `longest-exact` or `longest-after-opening`
//! (claude-356: the latest exact match at the row that chooses a reply's first token, the longest
//! match at every other row; with `--mark-reply 0` no row is marked and it is `longest-exact`).
//! It needs an artifact with a Hamming read. The replies are then no longer what the artifact was exported as; the receipt
//! gains `copy_rule` with the rule served and the artifact's own. Without the flag the artifact's
//! own rule is served and the receipt is as before.
//!
//! `--place-asks FILE` (claude-359) serves the artifact's place read from a fitted asks table
//! (`place-asks fit`) in place of its ask heads' finds: a place the asks find sets the register,
//! and a fact they find was never told is answered with their stored reply. `--place-speak 0|1`
//! (default 1) says whether an open row serves the token at its place outright. It needs an
//! artifact with a place read, fitted marks and vocabulary equal to its own. The receipt gains
//! `place_asks`: the file's SHA-256, `speak`, how many ids a row's place spoke, and how many
//! replies started on the stored reply. Without the flag the receipt is as before.
//!
//! `--serve-again 0|1` (claude-363, default 0, needs `--place-asks`) serves a reply again when
//! the register held a place right after the step that chose its first id and no step that
//! chose a reply id was offered a place: a fresh session reads the same prompt, the reply start
//! is opened (`IntegerStackSession::set_open_reply_start`) before the prompt's last token, and
//! the reply is served with the same loop and limits. The second reply stands: its line gains
//! `"again": true` and `"first"`, the first reply's ids, and the row's counters count it alone.
//! The receipt gains `serve_again` with `replies_served_again`. It is refused with
//! `--place-rows 1`, `--time-steps 1`, `--count-visits 1` and `--trace-control 1`, which read a
//! row's one serve. With the flag 0 the replies and the receipt are as before.
//!
//! `--whole-prompt TOKENS.u16 --bos ID` (claude-359) reads each row's prompt as a chat session
//! holds it: the whole conversation up to the reply. A float score's row holds only the model's
//! window, so a fact told before the window is not in the row at all, while a session keeps
//! every token and the place read looks through all of them. `TOKENS.u16` is the corpus the
//! rows were cut from; each row needs a `position`, and its own prompt must be the corpus ids
//! that end there. The prompt served is the corpus from the latest `ID` before the position up
//! to it: longer than the row's when the window had lost the conversation's start, and shorter
//! when the window began in the text before it. The receipt gains `whole_prompt`: the corpus's
//! SHA-256, `bos`, how many rows were lengthened, how many were shortened and the longest
//! prompt. It is refused with `--place-rows 1`, whose places are read as rows of the window.
//!
//! Every argument is a `--name value` pair; a name with no value is refused.
//!
//! It is an instrument: it trains nothing, changes no artifact and moves no bar.
//!
//! ```text
//! raw-id-replies --stack ART.uorlut --samples ROWS.json --out REPLIES.jsonl \
//!   --mark-reply 0|1 [--max-tokens 48] [--count-visits 0|1] [--time-steps 0|1] \
//!   [--trace-control 0|1 --trace-out TRACE.jsonl] \
//!   [--place-rows 0|1 [--places KIT.json [--kit-tokens TOKENS.u16]]] \
//!   [--copy-rule mean|latest-exact|latest-nearest|longest-exact|longest-after-opening] \
//!   [--place-asks ASKS.json [--place-speak 0|1]] \
//!   [--whole-prompt TOKENS.u16 --bos ID]
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
use uor_r4_integer::place_asks::PlaceAsks;
use uor_r4_integer::stack::{
    read_hamming_uorlut01, CopyRule, HammingStepRead, IntegerStackModel, IntegerStackSession,
};

const SCHEMA: &str = "uor-r4.raw-id-replies/1";
const EOS: u32 = 1;
const USAGE: &str = "usage: raw-id-replies --stack FILE --samples FILE --out FILE --mark-reply 0|1 [--max-tokens N] [--count-visits 0|1] [--time-steps 0|1] [--trace-control 0|1 --trace-out FILE] [--place-rows 0|1 [--places FILE [--kit-tokens FILE]]] [--copy-rule mean|latest-exact|latest-nearest|longest-exact|longest-after-opening] [--place-asks FILE [--place-speak 0|1] [--serve-again 0|1]] [--whole-prompt FILE --bos ID]";
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

/// One serve of a row: the reply and the row's counter increments, added to the run's only for
/// the serve that stands.
#[derive(Default)]
struct Served {
    ids: Vec<u32>,
    /// One `[held, offered]` a generated id, kept with `--place-rows 1`.
    registers: Vec<(Option<usize>, Option<usize>)>,
    /// The reply ended on the end mark.
    on_eos: bool,
    /// Reply ids a row's place spoke.
    spoke_place: u64,
    /// Replies that started the stored reply.
    told_replies: u64,
    /// The register held a place right after the step that chose the reply's first id.
    first_held: bool,
    /// Some step that chose a reply id was offered a place: its row was open.
    any_offered: bool,
}

/// `--serve-again 1` (claude-363): a reply is served again with its start open when the
/// register held a place after the step that chose its first id and no step that chose a reply
/// id was offered one: the asks found a place the reply never opened to.
fn serve_again_due(first_held: bool, any_offered: bool) -> bool {
    first_held && !any_offered
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

/// A place kit as `turn_census::place_kit` writes it (claude-351); the reader keeps its labels.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PlaceKitFile {
    tokens_sha256: String,
    #[serde(rename = "marks")]
    _marks: Value,
    #[serde(rename = "stop")]
    _stop: Vec<u32>,
    places: Vec<[u32; 4]>,
}

/// The kit's labels keyed by reply start: `(place, value start)`, in corpus positions.
struct PlaceLabels {
    by_reply: std::collections::BTreeMap<usize, (usize, usize)>,
    tokens_sha256: String,
    sha256: String,
}

fn place_labels(bytes: &[u8], path: &str) -> Result<PlaceLabels> {
    let kit: PlaceKitFile = serde_json::from_slice(bytes).map_err(|e| format!("{path}: {e}"))?;
    let mut by_reply = std::collections::BTreeMap::new();
    for [reply, place, value, _] in kit.places {
        let (reply, place, value) = (reply as usize, place as usize, value as usize);
        // The kit's own check: a place stands before its reply and its value inside it.
        if place >= reply || value < reply {
            return Err(format!("{path}: label at reply {reply} has place {place} and value start {value}"));
        }
        if by_reply.insert(reply, (place, value)).is_some() {
            return Err(format!("{path}: two labels for the reply at {reply}"));
        }
    }
    Ok(PlaceLabels { by_reply, tokens_sha256: kit.tokens_sha256, sha256: sha256_hex(bytes) })
}

/// Where a row's labelled place stands: its value row and its place as a prompt index.
#[derive(Debug, PartialEq, Eq)]
enum TruePlace {
    Unlabelled,
    Inside { value_row: usize, place: usize },
    Outside { value_row: usize },
}

/// The kit's label for the reply whose first target is at corpus `position`, read against a
/// prompt of `prompt_len` ids that ends there.
fn true_place(labels: &PlaceLabels, position: usize, prompt_len: usize) -> Result<TruePlace> {
    let start = position
        .checked_sub(prompt_len)
        .ok_or_else(|| format!("position {position} is before a prompt of {prompt_len} ids"))?;
    Ok(match labels.by_reply.get(&position) {
        None => TruePlace::Unlabelled,
        Some(&(place, value)) if place >= start => {
            TruePlace::Inside { value_row: value - position, place: place - start }
        }
        Some(&(_, value)) => TruePlace::Outside { value_row: value - position },
    })
}

#[derive(Default)]
struct PlaceTally {
    rows: usize,
    true_place: usize,
    outside: usize,
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

/// `--kit-tokens` (claude-352): binds the kit, its corpus and the rows.
/// - `bytes` is the corpus file the kit was written from: a `UORT` file, a 64-byte header and
///   then the ids, two little-endian bytes each.
/// - Its SHA-256 must be `kit_sha256`, the hash the kit names.
/// - Each row's `reference` must be the file's ids from the row's `position` on, at least one.
/// - Each row's `prompt` must be the file's ids that end at the row's `position`: the true
///   place is read as a row of the prompt, so a prompt with an id more or less at its front
///   would move every true place by one.
///
/// A kit labels by corpus position alone, so a kit written from another corpus, or rows cut
/// from another corpus, would give places for other text and no error.
fn check_kit_tokens(bytes: &[u8], path: &str, kit_sha256: &str, rows: &[Value]) -> Result<()> {
    if sha256_hex(bytes) != kit_sha256 {
        return Err(format!("{path}: its SHA-256 is not the kit's tokens_sha256: the kit was written from another corpus"));
    }
    if bytes.len() < 64 || !bytes.starts_with(b"UORT") {
        return Err(format!("{path}: not a UORT corpus file"));
    }
    // The corpus id at a position; none past the file's end or past what a position can be.
    let id_at = |position: usize| {
        let at = position.checked_mul(2)?.checked_add(64)?;
        let pair = bytes.get(at..at.checked_add(2)?)?;
        Some(u64::from(u16::from_le_bytes([pair[0], pair[1]])))
    };
    let equals = |start: usize, ids: &[Value]| {
        ids.iter().enumerate().all(|(j, id)| {
            let corpus = start.checked_add(j).and_then(id_at);
            corpus.is_some() && corpus == id.as_u64()
        })
    };
    for (i, row) in rows.iter().enumerate() {
        let position = row
            .get("position")
            .and_then(Value::as_u64)
            .and_then(|p| usize::try_from(p).ok())
            .ok_or_else(|| format!("row {i}: --kit-tokens needs a position on every row"))?;
        let reference = row
            .get("reference")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("row {i}: --kit-tokens needs a reference on every row"))?;
        if reference.is_empty() || !equals(position, reference) {
            return Err(format!("{path}: row {i}'s reference is not the corpus at position {position}: the rows are not this corpus's"));
        }
        let prompt = row
            .get("prompt")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("row {i}: --kit-tokens needs a prompt on every row"))?;
        let start = position.checked_sub(prompt.len());
        if !start.is_some_and(|start| equals(start, prompt)) {
            return Err(format!("{path}: row {i}'s prompt is not the corpus up to position {position}: its true place would be read against other text"));
        }
    }
    Ok(())
}

/// `--whole-prompt` (claude-359): a row's prompt as a chat session holds it.
/// - `bytes` is the corpus the rows were cut from, a `UORT` file.
/// - `own` is the row's prompt; it must be the corpus ids that end at the row's `position`, or
///   the row is another corpus's and the longer prompt would be other text.
/// - The result is the corpus from the latest `bos` before `position` up to it: the whole
///   conversation. `own` is its end when the window lost the start, and holds the end of the
///   text before the conversation when the window began there, which the result leaves out.
fn whole_prompt(bytes: &[u8], path: &str, bos: u32, row: &Value, index: usize, own: &[u32]) -> Result<Vec<u32>> {
    if bytes.len() < 64 || !bytes.starts_with(b"UORT") {
        return Err(format!("{path}: not a UORT corpus file"));
    }
    let id_at = |position: usize| {
        let at = position.checked_mul(2)?.checked_add(64)?;
        let pair = bytes.get(at..at.checked_add(2)?)?;
        Some(u32::from(u16::from_le_bytes([pair[0], pair[1]])))
    };
    let position = row
        .get("position")
        .and_then(Value::as_u64)
        .and_then(|p| usize::try_from(p).ok())
        .ok_or_else(|| format!("row {index}: --whole-prompt needs a position on every row"))?;
    // A position past the corpus names no reply of it, and the search back from it would not end.
    if position > (bytes.len() - 64) / 2 {
        return Err(format!("{path}: row {index}: position {position} is past the corpus"));
    }
    let start = position.checked_sub(own.len());
    let cut_here = start.is_some_and(|start| own.iter().enumerate().all(|(j, &id)| id_at(start + j) == Some(id)));
    if !cut_here {
        return Err(format!("{path}: row {index}'s prompt is not the corpus up to position {position}: the rows are not this corpus's"));
    }
    let from = (0..position)
        .rev()
        .find(|&at| id_at(at) == Some(bos))
        .ok_or_else(|| format!("{path}: row {index}: no id {bos} before position {position}"))?;
    Ok((from..position).filter_map(id_at).collect())
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
    println!("{}", run_with_args(std::env::args().skip(1))?);
    Ok(())
}

fn run_with_args(args: impl Iterator<Item = String>) -> Result<Value> {
    let args: Vec<String> = args.collect();
    // A name with no value would read as the flag left out: `--place-rows` last ran without
    // the register and said nothing (claude-352's review).
    if let Some(odd) = args.chunks(2).find(|pair| pair.len() != 2 || !pair[0].starts_with("--")) {
        return Err(format!("{}: every argument is a `--name value` pair\n{USAGE}", odd[0]));
    }
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
    let place_rows = match flag(&args, "--place-rows").as_deref() {
        None | Some("0") => false,
        Some("1") => true,
        Some(other) => return Err(format!("--place-rows takes 0 or 1, not {other}")),
    };
    let places_path = flag(&args, "--places");
    if places_path.is_some() && !place_rows {
        return Err("--places needs --place-rows 1".to_owned());
    }
    let kit_tokens_path = flag(&args, "--kit-tokens");
    if kit_tokens_path.is_some() && places_path.is_none() {
        return Err("--kit-tokens needs --places".to_owned());
    }
    let copy_rule = match flag(&args, "--copy-rule") {
        Some(name) => Some(CopyRule::parse(&name).ok_or_else(|| {
            format!("--copy-rule takes mean, latest-exact, latest-nearest, longest-exact or longest-after-opening, not {name}")
        })?),
        None => None,
    };
    // claude-363: a reply served again with its start open, when the asks found a place the
    // reply never opened to.
    let serve_again = match flag(&args, "--serve-again").as_deref() {
        None | Some("0") => false,
        Some("1") => true,
        Some(other) => return Err(format!("--serve-again takes 0 or 1, not {other}")),
    };
    if serve_again {
        if flag(&args, "--place-asks").is_none() {
            return Err("--serve-again 1 needs --place-asks".to_owned());
        }
        let one_serve = [
            (place_rows, "--place-rows"),
            (time_steps, "--time-steps"),
            (count_visits, "--count-visits"),
            (trace_control, "--trace-control"),
        ];
        if let Some((_, name)) = one_serve.iter().find(|(on, _)| *on) {
            return Err(format!(
                "--serve-again 1 is refused with {name} 1: it reads a row's one serve"
            ));
        }
    }

    let artifact = fs::read(&stack_path).map_err(|e| format!("{stack_path}: {e}"))?;
    let samples = fs::read(&samples_path).map_err(|e| format!("{samples_path}: {e}"))?;
    let mut model = IntegerStackModel::parse(&artifact).map_err(|e| format!("artifact: {e}"))?;
    // The artifact's own rule, read before any switch.
    let own_copy_rule = model.copy_rule();
    if let Some(rule) = copy_rule {
        if !model.set_copy_rule(rule) {
            // Without a Hamming read there is no copy head to set: the run would be a plain one.
            return Err("--copy-rule needs an artifact with a Hamming read".to_owned());
        }
    }
    // The asks (claude-359): a fitted table the place read is served from, set only here.
    let place_speak = match flag(&args, "--place-speak").as_deref() {
        None => None,
        Some("0") => Some(false),
        Some("1") => Some(true),
        Some(other) => return Err(format!("--place-speak takes 0 or 1, not {other}")),
    };
    let place_asks = match (flag(&args, "--place-asks"), place_speak) {
        (Some(path), speak) => {
            let bytes = fs::read(&path).map_err(|e| format!("{path}: {e}"))?;
            let asks: PlaceAsks = serde_json::from_slice(&bytes).map_err(|e| format!("{path}: {e}"))?;
            let speak = speak.unwrap_or(true);
            model.set_place_asks(&asks, speak).map_err(|e| format!("--place-asks: {e}"))?;
            Some((sha256_hex(&bytes), speak))
        }
        (None, Some(_)) => return Err("--place-speak needs --place-asks".to_owned()),
        (None, None) => None,
    };
    // The whole conversation as each row's prompt (claude-359): the corpus and its start mark.
    let whole = match (flag(&args, "--whole-prompt"), flag(&args, "--bos")) {
        (Some(_), _) if place_rows => {
            return Err("--whole-prompt is refused with --place-rows 1: its places are rows of the window".to_owned())
        }
        (Some(path), Some(bos)) => {
            let bos: u32 = bos.parse().map_err(|_| format!("--bos takes a token id, not {bos}"))?;
            let bytes = fs::read(&path).map_err(|e| format!("{path}: {e}"))?;
            Some((path, bytes, bos))
        }
        (Some(_), None) => return Err("--whole-prompt needs --bos".to_owned()),
        (None, Some(_)) => return Err("--bos needs --whole-prompt".to_owned()),
        (None, None) => None,
    };
    // Rows whose prompt grew, rows whose prompt shrank, and the longest prompt served.
    let (mut lengthened, mut shortened, mut longest_prompt) = (0usize, 0usize, 0usize);
    // Reply ids a row's place spoke, and replies that started the stored reply.
    let (mut spoke_place, mut told_replies) = (0u64, 0u64);
    let has_turn_memory = model.session().has_turn_memory();
    if mark && !has_turn_memory {
        // A marked run on a model without a turn memory would be an unmarked run by another name.
        return Err("--mark-reply 1 needs an artifact with a turn memory".to_owned());
    }
    if place_rows {
        let image = read_hamming_uorlut01(&artifact).map_err(|e| format!("artifact: {e}"))?;
        if image.and_then(|image| image.place).is_none() {
            // Without a place read the register is never set: every pair would be null.
            return Err("--place-rows 1 needs an artifact with a place read".to_owned());
        }
    }
    let labels = places_path
        .as_deref()
        .map(|path| {
            let bytes = fs::read(path).map_err(|e| format!("{path}: {e}"))?;
            place_labels(&bytes, path)
        })
        .transpose()?;
    let mut place_tally = PlaceTally::default();
    let rows: Vec<Value> =
        serde_json::from_slice(&samples).map_err(|e| format!("{samples_path}: {e}"))?;
    if let (Some(path), Some(labels)) = (kit_tokens_path.as_deref(), &labels) {
        let bytes = fs::read(path).map_err(|e| format!("{path}: {e}"))?;
        check_kit_tokens(&bytes, path, &labels.tokens_sha256, &rows)?;
    }
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
    // Rows whose reply was served again with its start open (`--serve-again 1`).
    let mut replies_served_again = 0usize;
    // One serve of a row: a fresh session reads the prompt and serves the reply. The second
    // serve of `--serve-again 1` runs this same code with `open_start` on. The row's counters
    // come back in `Served` and are added once it is known which serve stands; the clock, the
    // visit count and the trace are refused with `--serve-again 1`, so a row's one serve is all
    // that reaches them.
    let serve = |open_start: bool,
                 prompt: &[u32],
                 index: usize,
                 clock: &mut StepClock,
                 visits: &mut Visits,
                 trace_tally: &mut TraceTally,
                 trace: &mut Option<fs::File>|
     -> Result<Served> {
        let mut session = model.session();
        session.clear_reply_start();
        let mut pick = EOS;
        let mut reply = Served { ids: Vec::with_capacity(max_tokens), ..Served::default() };
        for (at, &token) in prompt.iter().enumerate() {
            if mark && at + 1 == prompt.len() {
                session.mark_reply_start();
            }
            if open_start && at + 1 == prompt.len() {
                session.set_open_reply_start(true);
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
            let (held, offered) = session.last_place();
            // This step chose the reply's first id.
            reply.first_held = held.is_some();
            reply.any_offered |= offered.is_some();
            if place_rows {
                reply.registers.push((held, offered));
            }
            match session.last_spoken() {
                Some((_, None)) => reply.spoke_place += 1,
                Some((_, Some(0))) => reply.told_replies += 1,
                _ => {}
            }
            if let (Some(table), Some(query)) = (table, session.last_query_q12()) {
                visits.count(table, query, session.last_native_token());
            }
        }
        loop {
            reply.ids.push(pick);
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
                        reply_position: reply.ids.len() - 1,
                        served: pick,
                        hamming: reads,
                        ids: &reply.ids,
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
                reply.on_eos = true;
                break;
            }
            if reply.ids.len() == max_tokens {
                break;
            }
            let started = Instant::now();
            let (served, _) = session
                .serve_step(pick)
                .map_err(|e| format!("row {index}, reply id {}: {e}", reply.ids.len()))?;
            clock.serve_ns += started.elapsed().as_nanos();
            clock.serve_steps += 1;
            pick = served;
            let (held, offered) = session.last_place();
            reply.any_offered |= offered.is_some();
            if place_rows {
                reply.registers.push((held, offered));
            }
            match session.last_spoken() {
                Some((_, None)) => reply.spoke_place += 1,
                Some((_, Some(0))) => reply.told_replies += 1,
                _ => {}
            }
            if let (Some(table), Some(query)) = (table, session.last_query_q12()) {
                visits.count(table, query, session.last_native_token());
            }
        }
        if trace_control {
            let replay = replay_reply(&model, prompt, mark, max_tokens)?;
            if replay != reply.ids {
                return Err(format!(
                    "row {index}: traced reply differs from untraced product replay"
                ));
            }
        }
        Ok(reply)
    };
    for (index, row) in rows.iter().enumerate() {
        let prompt = prompt_ids(row, index)?;
        let prompt = match &whole {
            Some((path, bytes, bos)) => {
                let all = whole_prompt(bytes, path, *bos, row, index, &prompt)?;
                lengthened += usize::from(all.len() > prompt.len());
                shortened += usize::from(all.len() < prompt.len());
                all
            }
            None => prompt,
        };
        longest_prompt = longest_prompt.max(prompt.len());
        let first = serve(false, &prompt, index, &mut clock, &mut visits, &mut trace_tally, &mut trace)?;
        // The reply that stands, and the first reply's ids when it was served again.
        let (reply, first_ids) = if serve_again && serve_again_due(first.first_held, first.any_offered) {
            let again = serve(true, &prompt, index, &mut clock, &mut visits, &mut trace_tally, &mut trace)?;
            replies_served_again += 1;
            (again, Some(first.ids))
        } else {
            (first, None)
        };
        stopped_on_eos += usize::from(reply.on_eos);
        spoke_place += reply.spoke_place;
        told_replies += reply.told_replies;
        let (ids, registers) = (reply.ids, reply.registers);
        let mut line = json!({"i": index, "ids": ids});
        if let Some(first_ids) = first_ids {
            line["again"] = json!(true);
            line["first"] = json!(first_ids);
        }
        if place_rows {
            let pairs: Vec<Value> =
                registers.iter().map(|(held, offered)| json!([held, offered])).collect();
            line["place"] = Value::Array(pairs);
        }
        if let Some(labels) = &labels {
            if let Some(position) = row.get("position") {
                let position = position
                    .as_u64()
                    .and_then(|p| usize::try_from(p).ok())
                    .ok_or_else(|| format!("row {index}: position is not a corpus index"))?;
                place_tally.rows += 1;
                match true_place(labels, position, prompt.len())
                    .map_err(|e| format!("row {index}: {e}"))?
                {
                    TruePlace::Unlabelled => {
                        line["value_row"] = Value::Null;
                        line["true_place"] = Value::Null;
                    }
                    TruePlace::Inside { value_row, place } => {
                        place_tally.true_place += 1;
                        line["value_row"] = json!(value_row);
                        line["true_place"] = json!(place);
                    }
                    TruePlace::Outside { value_row } => {
                        place_tally.outside += 1;
                        line["value_row"] = json!(value_row);
                        line["true_place"] = Value::Null;
                        line["true_place_outside"] = json!(true);
                    }
                }
            }
        }
        writeln!(out, "{line}").map_err(|e| format!("{out_path}: {e}"))?;
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
    if let Some((sha256, speak)) = &place_asks {
        receipt["place_asks"] = json!({
            "sha256": sha256,
            "speak": speak,
            "ids_spoken_from_a_place": spoke_place,
            "replies_started_on_the_stored_reply": told_replies,
        });
    }
    if serve_again {
        receipt["serve_again"] = json!({"replies_served_again": replies_served_again});
    }
    if let Some((_, bytes, bos)) = &whole {
        receipt["whole_prompt"] = json!({
            "tokens_sha256": sha256_hex(bytes),
            "bos": bos,
            "rows_lengthened": lengthened,
            "rows_shortened": shortened,
            "longest_prompt": longest_prompt,
        });
    }
    if copy_rule.is_some() {
        // What the session serves after the switch, beside the artifact's own rule.
        receipt["copy_rule"] = json!({
            "served": model.copy_rule().map(CopyRule::name),
            "own": own_copy_rule.map(CopyRule::name),
        });
    }
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
    if let Some(labels) = &labels {
        receipt["place_rows"] = json!({
            "rows": place_tally.rows,
            "true_place": place_tally.true_place,
            "true_place_outside": place_tally.outside,
            "places_sha256": labels.sha256,
            "kit_tokens_sha256": labels.tokens_sha256,
            "kit_tokens_checked": kit_tokens_path.is_some(),
        });
    }
    Ok(receipt)
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

    /// claude-355: `--copy-rule` sets the copy heads' rule by name and the receipt records what
    /// is served; the artifact's own rule by name changes no reply; an unknown name and an
    /// artifact with no Hamming read are refused.
    #[test]
    fn the_copy_rule_is_set_by_name_and_recorded() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("uor-reply-rule-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&root).expect("temp directory");
        let stack = root.join("synthetic.uorlut");
        let bare = root.join("bare.uorlut");
        let samples = root.join("samples.json");
        fs::write(&stack, trace_artifact()).expect("artifact");
        let mut snap = synthetic_fbs_trunk(17);
        snap.readout_nearest = true;
        fs::write(&bare, export_fbs_trunk_uorlut01(&snap).expect("bare artifact")).expect("bare");
        fs::write(&samples, r#"[{"prompt":[2,3,2,3,2]}]"#).expect("samples");
        let run = |stack: &Path, out: &str, mark: &str, rule: Option<&str>| {
            let mut args = vec![
                "--stack".to_owned(),
                stack.display().to_string(),
                "--samples".to_owned(),
                samples.display().to_string(),
                "--out".to_owned(),
                root.join(out).display().to_string(),
                "--mark-reply".to_owned(),
                mark.to_owned(),
                "--max-tokens".to_owned(),
                "4".to_owned(),
            ];
            if let Some(rule) = rule {
                args.extend(["--copy-rule".to_owned(), rule.to_owned()]);
            }
            run_with_args(args.into_iter())
        };

        let plain = run(&stack, "plain.jsonl", "1", None).expect("plain run");
        assert!(plain.get("copy_rule").is_none(), "{plain}");
        let own = IntegerStackModel::parse(&trace_artifact())
            .expect("artifact")
            .copy_rule()
            .expect("a Hamming read")
            .name();
        let same = run(&stack, "own.jsonl", "1", Some(own)).expect("own rule");
        assert_eq!(same["copy_rule"], json!({"served": own, "own": own}));
        assert_eq!(
            fs::read(root.join("plain.jsonl")).expect("plain replies"),
            fs::read(root.join("own.jsonl")).expect("own-rule replies")
        );
        let longest = run(&stack, "longest.jsonl", "1", Some("longest-exact")).expect("longest");
        assert_eq!(longest["copy_rule"], json!({"served": "longest-exact", "own": own}));
        // claude-356: the rule that spares a reply's opening is served by its name, and with no
        // reply marked it reads every row as the longest rule does.
        let spared = run(&stack, "spared.jsonl", "1", Some("longest-after-opening")).expect("spared");
        assert_eq!(spared["copy_rule"], json!({"served": "longest-after-opening", "own": own}));
        run(&stack, "longest-unmarked.jsonl", "0", Some("longest-exact")).expect("longest, unmarked");
        run(&stack, "spared-unmarked.jsonl", "0", Some("longest-after-opening")).expect("spared, unmarked");
        assert_eq!(
            fs::read(root.join("longest-unmarked.jsonl")).expect("longest replies"),
            fs::read(root.join("spared-unmarked.jsonl")).expect("spared replies")
        );

        let error = run(&stack, "bad.jsonl", "1", Some("longest")).unwrap_err();
        assert!(error.contains("--copy-rule takes mean, latest-exact"), "{error}");
        assert!(!root.join("bad.jsonl").exists());
        let error = run(&bare, "bare.jsonl", "0", Some("longest-exact")).unwrap_err();
        assert!(error.contains("--copy-rule needs an artifact with a Hamming read"), "{error}");
        assert!(!root.join("bare.jsonl").exists());
        let _ = fs::remove_dir_all(root);
    }

    // The integer crate's place-read conversation (`stack/tests.rs`, claude-351): a user turn
    // states the value `30 31` at positions 4 and 5, a later one asks for it with `PLACE_ASK`,
    // and the reply opens it with `PLACE_OPENS`, says it and stops on `PLACE_DOT`.
    const PLACE_BOS: u32 = 500;
    const PLACE_EOS: u32 = 501;
    const PLACE_SEP: u32 = 502;
    const PLACE_USER: u32 = 504;
    const PLACE_MODEL: u32 = 505;
    const PLACE_BEFORE: u32 = 21;
    const PLACE_ASK: u32 = 50;
    const PLACE_OPENS: u32 = 60;
    const PLACE_DOT: u32 = 32;
    const PLACE_TALK: [u32; 22] = [
        PLACE_BOS, PLACE_USER, 20, PLACE_BEFORE, 30, 31, PLACE_DOT, PLACE_EOS, PLACE_SEP,
        PLACE_MODEL, 40, PLACE_EOS, PLACE_SEP, PLACE_USER, 20, PLACE_ASK, PLACE_SEP, PLACE_MODEL,
        PLACE_OPENS, 30, 31, PLACE_DOT,
    ];

    /// A hand-built artifact with a place read, built as the integer crate's `place_trunk` builds
    /// it through the public API: a synthetic trunk with copy reads at depths 1 and 3, an ask
    /// table whose `PLACE_ASK` row finds the token after `PLACE_BEFORE`, and the classes stop
    /// (`PLACE_DOT`), arm (`PLACE_ASK`) and open (`PLACE_OPENS`). Returned with the artifact it
    /// was built on, which asks and has no place read.
    fn place_artifacts() -> (Vec<u8>, Vec<u8>) {
        let (asked, placed, _) = place_artifacts_with_image();
        (asked, placed)
    }

    /// [`place_artifacts`] with the place image the second artifact carries: its marks and its
    /// vocabulary are what a table for it must name.
    fn place_artifacts_with_image() -> (Vec<u8>, Vec<u8>, uor_r4_integer::stack::PlaceImage) {
        use uor_r4_integer::stack::synthetic::{
            synthetic_fbs_trunk_with, FbsSynth, FBS_SYNTH_NON_SQUARE,
        };
        use uor_r4_integer::stack::{
            add_hamming_ask_uorlut01, add_hamming_place_uorlut01, served_copy_codes_uorlut01,
            AskImage, PlaceImage, PlaceMarks, PLACE_ARM, PLACE_OPEN, PLACE_STOP,
        };
        let shape = FbsSynth {
            context: 40,
            ..FBS_SYNTH_NON_SQUARE
        };
        let mut snap = synthetic_fbs_trunk_with(20261029, shape);
        let (vocab, width, layers) = (snap.vocab, snap.width, snap.layers.len());
        let mut read = synthetic_hamming(width, layers, 4, 194);
        let depths = [1, 3];
        let all = std::mem::take(&mut read.depths);
        read.reads = all
            .iter()
            .zip(std::mem::take(&mut read.reads))
            .filter(|(depth, _)| depths.contains(depth))
            .map(|(_, r)| r)
            .collect();
        read.depths = depths.to_vec();
        for r in &mut read.reads {
            // The copy heads at full weight and the type heads off, as `place_trunk` has them.
            r.weight[width..].fill(Default::default());
        }
        snap.hamming = Some(read);
        let bytes = export_fbs_trunk_uorlut01(&snap).expect("export with the reads");
        let table = served_copy_codes_uorlut01(&bytes)
            .expect("parse")
            .expect("reads")
            .table;
        let row = |v: u32| &table[v as usize * width..][..width];
        let mut ask = vec![0i16; vocab * width];
        for (k, a) in ask[PLACE_ASK as usize * width..][..width].iter_mut().enumerate() {
            *a = row(PLACE_BEFORE)[k] - row(PLACE_ASK)[k];
        }
        let asks = [None, Some(AskImage { table: ask, carry: None })];
        let asked = add_hamming_ask_uorlut01(&bytes, &asks).expect("the readout read asks");
        let marks = PlaceMarks {
            bos: PLACE_BOS,
            eos: PLACE_EOS,
            sep: vec![PLACE_SEP],
            system: vec![503],
            user: vec![PLACE_USER],
            assistant: vec![PLACE_MODEL],
        };
        let mut class = vec![0i16; vocab];
        class[PLACE_DOT as usize] = PLACE_STOP;
        class[PLACE_ASK as usize] = PLACE_ARM;
        class[PLACE_OPENS as usize] = PLACE_OPEN;
        let place = PlaceImage::checked(marks, class, vocab).expect("the classes");
        let placed = add_hamming_place_uorlut01(&asked, &place).expect("the place read is added");
        (asked, placed, place)
    }

    /// claude-352, check I-l: `--place-rows 1` prints the register after each step that chose a
    /// reply id, and `--places` the kit's true place and value row in the prompt's terms.
    #[test]
    fn place_rows_print_the_register_and_the_true_place() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("uor-place-rows-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&root).expect("temp directory");
        let stack = root.join("placed.uorlut");
        let samples = root.join("samples.json");
        let kit = root.join("places.json");
        let placed_out = root.join("placed.jsonl");
        let plain_out = root.join("plain.jsonl");
        let (_, artifact) = place_artifacts();
        fs::write(&stack, artifact).expect("artifact");
        // Each prompt is the conversation up to a reply row, at corpus positions from 1000:
        // the reply start (the register set, not open), the open row, and the row after the
        // stop; then the reply start again at another corpus place, with no kit label.
        let base = 1000usize;
        let rows: Vec<Value> = [18usize, 19, 22, 18]
            .iter()
            .enumerate()
            .map(|(i, &end)| {
                let position = if i == 3 { 5000 + end } else { base + end };
                json!({"prompt": PLACE_TALK[..end].to_vec(), "position": position})
            })
            .collect();
        fs::write(&samples, serde_json::to_vec(&rows).expect("rows")).expect("samples");
        // The labels: the reply at 1018 reads the value stated at 1004 and says it from its
        // first target; the reply at 1019 is labelled with a place before its prompt's first
        // token; the reply at 1022 by the value stated at 1005, said one row later.
        let kit_json = json!({
            "tokens_sha256": "00",
            "marks": {},
            "stop": [PLACE_DOT],
            "places": [[1018, 1004, 1018, 2], [1019, 990, 1019, 1], [1022, 1005, 1023, 1]],
        });
        fs::write(&kit, serde_json::to_vec(&kit_json).expect("kit")).expect("kit file");
        let args = |out: &Path, place: bool| {
            let mut args = vec![
                "--stack".to_owned(),
                stack.display().to_string(),
                "--samples".to_owned(),
                samples.display().to_string(),
                "--out".to_owned(),
                out.display().to_string(),
                "--mark-reply".to_owned(),
                "0".to_owned(),
                "--max-tokens".to_owned(),
                "3".to_owned(),
            ];
            if place {
                args.extend([
                    "--place-rows".to_owned(),
                    "1".to_owned(),
                    "--places".to_owned(),
                    kit.display().to_string(),
                ]);
            }
            args
        };
        let receipt = run_with_args(args(&placed_out, true).into_iter()).expect("place rows");
        let plain_receipt = run_with_args(args(&plain_out, false).into_iter()).expect("plain rows");
        let read = |path: &Path| -> Vec<Value> {
            fs::read_to_string(path)
                .expect("replies")
                .lines()
                .map(|line| serde_json::from_str(line).expect("reply JSON"))
                .collect()
        };
        let (placed, plain) = (read(&placed_out), read(&plain_out));
        let _ = fs::remove_dir_all(&root);
        // Row 0 is served from `PLACE_MODEL`: the find set the register at the value's first
        // token, position 4, and the row is not open; the replies 40 and 486 are neither open
        // nor stop tokens, so it stays. Row 1 is served from `PLACE_OPENS`: the row is open and
        // offered place 4; the reply 118 is not the offered token 30, so the next row closes
        // and the register keeps place 4. Row 2 is served from `PLACE_DOT`, which cleared it.
        // Row 3 is row 0's prompt at a corpus place the kit does not label.
        let want = [
            json!({"i": 0, "ids": [40, 486, 82], "place": [[4, null], [4, null], [4, null]],
                "value_row": 0, "true_place": 4}),
            json!({"i": 1, "ids": [118, 182, 191], "place": [[4, 4], [4, null], [4, null]],
                "value_row": 0, "true_place": null, "true_place_outside": true}),
            json!({"i": 2, "ids": [447, 43, 219], "place": [[null, null], [null, null], [null, null]],
                "value_row": 1, "true_place": 5}),
            json!({"i": 3, "ids": [40, 486, 82], "place": [[4, null], [4, null], [4, null]],
                "value_row": null, "true_place": null}),
        ];
        assert_eq!(placed, want);
        // Off, every line is `{"i", "ids"}` with the same ids, and the receipt has no place rows.
        for (plain, placed) in plain.iter().zip(&placed) {
            assert_eq!(*plain, json!({"i": placed["i"], "ids": placed["ids"]}));
        }
        assert_eq!(plain.len(), placed.len());
        assert!(plain_receipt.get("place_rows").is_none());
        let counts = &receipt["place_rows"];
        assert_eq!(
            [&counts["rows"], &counts["true_place"], &counts["true_place_outside"]],
            [&json!(4), &json!(2), &json!(1)]
        );
        // The kit and the register refuse what they cannot read.
        let kit_only = args(&root.join("unused.jsonl"), true);
        let without_rows: Vec<String> = kit_only
            .iter()
            .filter(|arg| !matches!(arg.as_str(), "--place-rows" | "1"))
            .cloned()
            .collect();
        let error = run_with_args(without_rows.into_iter()).unwrap_err();
        assert!(error.contains("--places needs --place-rows 1"), "{error}");
    }

    #[test]
    fn true_place_reads_the_kit_in_prompt_terms() {
        let labels = place_labels(
            br#"{"tokens_sha256":"00","marks":{},"stop":[],"places":[[20,12,23,2],[40,5,40,1]]}"#,
            "kit",
        )
        .expect("kit");
        // The prompt of 10 ids covers corpus 10..20: place 12 is prompt index 2, the value's
        // first token is chosen at reply row 3.
        assert_eq!(
            true_place(&labels, 20, 10).unwrap(),
            TruePlace::Inside { value_row: 3, place: 2 }
        );
        assert_eq!(true_place(&labels, 40, 10).unwrap(), TruePlace::Outside { value_row: 0 });
        assert_eq!(true_place(&labels, 21, 10).unwrap(), TruePlace::Unlabelled);
        assert!(true_place(&labels, 5, 10).is_err(), "a prompt longer than its position");
        let after = br#"{"tokens_sha256":"00","marks":{},"stop":[],"places":[[20,20,23,2]]}"#;
        assert!(place_labels(after, "kit").is_err(), "a place stands before its reply");
        let twice = br#"{"tokens_sha256":"00","marks":{},"stop":[],"places":[[20,1,23,2],[20,2,24,1]]}"#;
        assert!(place_labels(twice, "kit").is_err(), "one label a reply");
        let unknown = br#"{"tokens_sha256":"00","marks":{},"stop":[],"places":[],"gain":1}"#;
        assert!(place_labels(unknown, "kit").is_err(), "the kit's own shape");
    }

    /// claude-352's review: a name with no value read as the flag left out, so `--place-rows`
    /// as the last argument ran without the register and said nothing.
    #[test]
    fn a_name_without_its_value_is_refused() {
        let args = ["--stack", "a", "--samples", "b", "--out", "c", "--mark-reply", "0", "--place-rows"];
        let error = run_with_args(args.iter().map(|arg| (*arg).to_owned())).unwrap_err();
        assert!(error.contains("--place-rows: every argument is a `--name value` pair"), "{error}");
        let stray = ["--stack", "a", "b", "--samples"];
        let error = run_with_args(stray.iter().map(|arg| (*arg).to_owned())).unwrap_err();
        assert!(error.contains("b: every argument"), "{error}");
    }

    /// claude-363: a reply is served again only when the register held a place after its first
    /// id was chosen and no choosing step was offered one.
    #[test]
    fn a_reply_is_served_again_only_on_a_held_place_never_offered() {
        assert!(serve_again_due(true, false));
        assert!(!serve_again_due(true, true));
        assert!(!serve_again_due(false, false));
        assert!(!serve_again_due(false, true));
    }

    /// claude-363: `--serve-again` takes 0 or 1, needs `--place-asks`, and is refused with each
    /// flag that reads a row's one serve; all before any file is read.
    #[test]
    fn serve_again_is_parsed_and_refused_by_name() {
        let run = |extra: &[&str]| {
            let fixed = ["--stack", "/nonexistent/a", "--samples", "b", "--out", "c", "--mark-reply", "0"];
            run_with_args(fixed.iter().chain(extra).map(|arg| (*arg).to_owned())).unwrap_err()
        };
        let asks = ["--place-asks", "/nonexistent/asks.json", "--serve-again", "1"];
        let with = |more: [&'static str; 2]| -> Vec<&str> { asks.iter().copied().chain(more).collect() };
        assert!(run(&["--serve-again", "2"]).contains("--serve-again takes 0 or 1, not 2"));
        assert!(run(&["--serve-again", "1"]).contains("--serve-again 1 needs --place-asks"));
        for name in ["--place-rows", "--time-steps", "--count-visits"] {
            let error = run(&with([name, "1"]));
            assert!(error.contains(&format!("--serve-again 1 is refused with {name} 1")), "{error}");
        }
        let traced: Vec<&str> = with(["--trace-control", "1"]).into_iter().chain(["--trace-out", "t"]).collect();
        let error = run(&traced);
        assert!(error.contains("--serve-again 1 is refused with --trace-control 1"), "{error}");
        // Past the flags the run goes on to read the artifact.
        for extra in [&asks[..], &["--serve-again", "0"][..]] {
            let error = run(extra);
            assert!(error.starts_with("/nonexistent/a:"), "{error}");
        }
    }

    /// claude-359: `--whole-prompt` gives the conversation from its start mark, and refuses a
    /// row cut from other text, a row with no position and a corpus with no start mark.
    #[test]
    fn the_whole_prompt_is_the_conversation_from_its_start_mark() {
        // Two conversations: ids 7 20 21 22 | 7 30 31 32 33, start mark 7.
        let mut corpus = b"UORT".to_vec();
        corpus.resize(64, 0);
        for id in [7u16, 20, 21, 22, 7, 30, 31, 32, 33] {
            corpus.extend(id.to_le_bytes());
        }
        let row = json!({"position": 8, "prompt": [31, 32]});
        assert_eq!(whole_prompt(&corpus, "tokens", 7, &row, 0, &[31, 32]).unwrap(), [7, 30, 31, 32]);
        // A window that already holds the start comes back as it is.
        let all = json!({"position": 3, "prompt": [7, 20, 21]});
        assert_eq!(whole_prompt(&corpus, "tokens", 7, &all, 0, &[7, 20, 21]).unwrap(), [7, 20, 21]);
        let error = whole_prompt(&corpus, "tokens", 7, &row, 0, &[30, 32]).unwrap_err();
        assert!(error.contains("not the corpus"), "{error}");
        let error = whole_prompt(&corpus, "tokens", 7, &json!({"prompt": [31, 32]}), 0, &[31, 32]).unwrap_err();
        assert!(error.contains("position"), "{error}");
        let error = whole_prompt(&corpus, "tokens", 9, &row, 0, &[31, 32]).unwrap_err();
        assert!(error.contains("no id 9"), "{error}");
        assert!(whole_prompt(&corpus[..40], "tokens", 7, &row, 0, &[31, 32]).is_err(), "a file with no header");
        // A window that began in the text before the conversation comes back without that text.
        let early = json!({"position": 6, "prompt": [22, 7, 30]});
        assert_eq!(whole_prompt(&corpus, "tokens", 7, &early, 0, &[22, 7, 30]).unwrap(), [7, 30]);
        // A position past the corpus is refused, whatever the prompt: an empty one would pass
        // the check of the row and leave a search with no end.
        let far = json!({"position": u64::MAX, "prompt": []});
        let error = whole_prompt(&corpus, "tokens", 7, &far, 0, &[]).unwrap_err();
        assert!(error.contains("past the corpus"), "{error}");
    }

    /// claude-359, review round 1: the asks and the whole prompt through the run. The receipt
    /// says what was served, and each flag that needs another is refused by name.
    #[test]
    fn the_asks_and_the_whole_prompt_are_served_and_told_in_the_receipt() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("uor-asks-run-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&root).expect("temp directory");
        let file = |name: &str, bytes: &[u8]| {
            let path = root.join(name);
            fs::write(&path, bytes).expect("a file of the test");
            path.display().to_string()
        };
        let (_, placed, place) = place_artifacts_with_image();
        let stack = file("placed.uorlut", &placed);
        // A corpus of 1,100 ids with the conversation from position 1000.
        let mut corpus = b"UORT".to_vec();
        corpus.resize(64, 0);
        for at in 0..1100usize {
            let id = at.checked_sub(1000).and_then(|k| PLACE_TALK.get(k)).copied().unwrap_or(0);
            corpus.extend(u16::try_from(id).expect("an id of the test").to_le_bytes());
        }
        let tokens = file("tokens.u16", &corpus);
        // Three rows that end at the reply's first row: the conversation as it is, a window that
        // lost its first four ids, and a window that holds two ids of the text before it.
        let before: Vec<u32> = [0, 0].into_iter().chain(PLACE_TALK[..18].iter().copied()).collect();
        let rows = json!([
            {"prompt": PLACE_TALK[..18].to_vec(), "position": 1018},
            {"prompt": PLACE_TALK[4..18].to_vec(), "position": 1018},
            {"prompt": before, "position": 1018},
        ]);
        let samples = file("samples.json", &serde_json::to_vec(&rows).expect("rows"));
        // A table that finds the position after id 30 for a question holding `PLACE_ASK`, and
        // opens the role marker's row, so a reply's first id is the id at that place, 31.
        let asks = PlaceAsks {
            tokens_sha256: "test".to_owned(),
            marks: place.marks.clone(),
            vocab: place.class.len() as u32,
            link: vec![30],
            shared: Vec::new(),
            unseen: vec![[0, 0]; 8],
            pairs: vec![[PLACE_ASK as i32, 1, 30, 100]],
            floor: 50,
            floor_other: 50,
            margin: 0,
            opens: vec![[PLACE_SEP as i32, PLACE_MODEL as i32, 1]],
            asking: vec![[PLACE_ASK as i32, 10]],
            asking_prior: 0,
            asking_floor: 5,
            not_told: vec![41, 42, PLACE_EOS],
            forms: vec![vec![41, 42, PLACE_EOS]],
            repeat: Vec::new(),
            repeat3: Vec::new(),
            ask_floors: Vec::new(),
        };
        let table_bytes = serde_json::to_vec(&asks).expect("the asks");
        let table = file("asks.json", &table_bytes);
        let run = |name: &str, extra: &[&str]| {
            let out = root.join(name).display().to_string();
            let fixed = [
                "--stack", stack.as_str(), "--samples", samples.as_str(), "--out", out.as_str(),
                "--mark-reply", "0", "--max-tokens", "3",
            ];
            run_with_args(fixed.iter().chain(extra).map(|arg| (*arg).to_owned()))
        };
        let (asks_flag, whole_flag) = (["--place-asks", table.as_str()], ["--whole-prompt", tokens.as_str(), "--bos", "500"]);
        let both: Vec<&str> = asks_flag.iter().chain(&whole_flag).copied().collect();
        let quiet: Vec<&str> = asks_flag.iter().copied().chain(["--place-speak", "0"]).collect();
        let rows_too: Vec<&str> = whole_flag.iter().copied().chain(["--place-rows", "1"]).collect();
        let plain = run("plain.jsonl", &[]).expect("no new flag");
        let served = run("served.jsonl", &both).expect("the asks on the whole prompt");
        let unspoken = run("unspoken.jsonl", &quiet).expect("the asks with speak off");
        let first_ids = |name: &str| -> Vec<Value> {
            let text = fs::read_to_string(root.join(name)).expect("reply rows");
            text.lines().map(|line| serde_json::from_str::<Value>(line).expect("a row")["ids"][0].clone()).collect()
        };
        let (served_first, unspoken_first) = (first_ids("served.jsonl"), first_ids("unspoken.jsonl"));
        let refused = [
            run("a.jsonl", &["--bos", "500"]).unwrap_err(),
            run("b.jsonl", &["--whole-prompt", tokens.as_str()]).unwrap_err(),
            run("c.jsonl", &rows_too).unwrap_err(),
            run("d.jsonl", &["--place-speak", "1"]).unwrap_err(),
        ];
        let _ = fs::remove_dir_all(&root);
        assert!(plain.get("place_asks").is_none() && plain.get("whole_prompt").is_none(), "{plain}");
        // Every row is served the one conversation, and the open row speaks the id at the place.
        assert_eq!(served_first, vec![json!(31); 3]);
        assert_eq!(
            served["place_asks"],
            json!({"sha256": sha256_hex(&table_bytes), "speak": true, "ids_spoken_from_a_place": 3, "replies_started_on_the_stored_reply": 0})
        );
        assert_eq!(
            served["whole_prompt"],
            json!({"tokens_sha256": sha256_hex(&corpus), "bos": 500, "rows_lengthened": 1, "rows_shortened": 1, "longest_prompt": 18})
        );
        // With speak off no id is spoken, and the receipt says so.
        assert_eq!(unspoken["place_asks"]["speak"], false);
        assert_eq!(unspoken["place_asks"]["ids_spoken_from_a_place"], 0);
        assert_ne!(unspoken_first, served_first);
        for (error, want) in refused.iter().zip(["--bos needs --whole-prompt", "--whole-prompt needs --bos", "refused with --place-rows 1", "--place-speak needs --place-asks"]) {
            assert!(error.contains(want), "{error}");
        }
    }

    /// claude-352's review: `--kit-tokens` refuses a kit written from another corpus and rows
    /// that are not this corpus's, and passes the corpus the kit names.
    #[test]
    fn the_kit_tokens_bind_the_kit_and_the_rows_to_one_corpus() {
        // A UORT file: a 64-byte header, then ids 10, 11, 12, 13 at positions 0 to 3.
        let mut corpus = b"UORT".to_vec();
        corpus.resize(64, 0);
        for id in [10u16, 11, 12, 13] {
            corpus.extend(id.to_le_bytes());
        }
        let sha = sha256_hex(&corpus);
        let rows = vec![json!({"position": 1, "prompt": [10], "reference": [11, 12]})];
        assert!(check_kit_tokens(&corpus, "tokens", &sha, &rows).is_ok());
        let refused = |row: Value, word: &str| {
            let error = check_kit_tokens(&corpus, "tokens", &sha, &[row]).unwrap_err();
            assert!(error.contains(word), "{error}");
        };
        // A prompt with an id more at its front, a prompt of other ids, and no prompt.
        refused(json!({"position": 1, "prompt": [9, 10], "reference": [11]}), "row 0's prompt is not");
        refused(json!({"position": 2, "prompt": [10, 12], "reference": [12]}), "row 0's prompt is not");
        refused(json!({"position": 1, "reference": [11]}), "needs a prompt");
        // A reference of no ids binds nothing, at any position.
        refused(json!({"position": 1, "prompt": [10], "reference": []}), "row 0's reference is not");
        // A position past what the file can hold is refused, not wrapped around.
        refused(json!({"position": u64::MAX, "prompt": [10], "reference": [10]}), "row 0's reference is not");
        // A file with the kit's hash and no `UORT` header.
        let mut bare_file = corpus.clone();
        bare_file[0] = b'X';
        let error = check_kit_tokens(&bare_file, "tokens", &sha256_hex(&bare_file), &rows).unwrap_err();
        assert!(error.contains("not a UORT corpus file"), "{error}");
        let error = check_kit_tokens(&corpus, "tokens", "00", &rows).unwrap_err();
        assert!(error.contains("another corpus"), "{error}");
        let moved = vec![json!({"position": 2, "prompt": [10], "reference": [11, 12]})];
        let error = check_kit_tokens(&corpus, "tokens", &sha, &moved).unwrap_err();
        assert!(error.contains("row 0's reference is not the corpus at position 2"), "{error}");
        let past = vec![json!({"position": 3, "prompt": [10], "reference": [13, 14]})];
        assert!(check_kit_tokens(&corpus, "tokens", &sha, &past).is_err(), "a reference past the file's end");
        let bare = vec![json!({"prompt": [10]})];
        assert!(check_kit_tokens(&corpus, "tokens", &sha, &bare).is_err(), "a row with no position");
    }

    /// claude-352, review round 1: `--kit-tokens` on the whole run. The receipt says the kit's
    /// corpus was checked only when it was, and the check changes no reply row. A corpus the
    /// kit was not written from, `--kit-tokens` without its kit, and `--place-rows 1` on an
    /// artifact without a place read are each refused by name, before a row is written.
    #[test]
    fn the_kit_tokens_are_checked_through_the_run() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("uor-kit-tokens-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&root).expect("temp directory");
        let file = |name: &str, bytes: &[u8]| {
            let path = root.join(name);
            fs::write(&path, bytes).expect("a file of the test");
            path.display().to_string()
        };
        let (asked, placed) = place_artifacts();
        let (stack, bare_stack) = (file("placed.uorlut", &placed), file("asked.uorlut", &asked));
        // A corpus of 1,100 ids with the conversation from position 1000.
        let mut ids = vec![0u16; 1100];
        for (k, &id) in PLACE_TALK.iter().enumerate() {
            ids[1000 + k] = u16::try_from(id).expect("an id of the test");
        }
        let mut corpus = b"UORT".to_vec();
        corpus.resize(64, 0);
        for id in &ids {
            corpus.extend(id.to_le_bytes());
        }
        let tokens = file("tokens.u16", &corpus);
        let mut other = corpus.clone();
        *other.last_mut().expect("bytes") ^= 1;
        let other_tokens = file("other.u16", &other);
        let rows = json!([{
            "prompt": PLACE_TALK[..18].to_vec(),
            "position": 1018,
            "reference": [ids[1018], ids[1019]],
        }]);
        let samples = file("samples.json", &serde_json::to_vec(&rows).expect("rows"));
        let kit_json = json!({
            "tokens_sha256": sha256_hex(&corpus),
            "marks": {},
            "stop": [PLACE_DOT],
            "places": [[1018, 1004, 1018, 2]],
        });
        let kit = file("places.json", &serde_json::to_vec(&kit_json).expect("kit"));
        let run = |name: &str, stack: &str, extra: &[&str]| {
            let out = root.join(name).display().to_string();
            let fixed = [
                "--stack", stack, "--samples", samples.as_str(), "--out", out.as_str(),
                "--mark-reply", "0", "--max-tokens", "3",
            ];
            run_with_args(fixed.iter().chain(extra).map(|arg| (*arg).to_owned()))
        };
        let place = ["--place-rows", "1", "--places", kit.as_str()];
        let [rows_name, on, kit_name, kit_path] = place;
        let bound = [rows_name, on, kit_name, kit_path, "--kit-tokens", tokens.as_str()];
        let other_bound = [rows_name, on, kit_name, kit_path, "--kit-tokens", other_tokens.as_str()];
        let unchecked = run("unchecked.jsonl", &stack, &place).expect("the kit alone");
        let checked = run("checked.jsonl", &stack, &bound).expect("the kit and its corpus");
        let rows_of = |name: &str| fs::read(root.join(name)).expect("reply rows");
        let same = rows_of("unchecked.jsonl") == rows_of("checked.jsonl");
        let another = run("another.jsonl", &stack, &other_bound).unwrap_err();
        let written = root.join("another.jsonl").exists();
        let no_kit = run("no-kit.jsonl", &stack, &["--kit-tokens", tokens.as_str()]).unwrap_err();
        let no_place = run("no-place.jsonl", &bare_stack, &place).unwrap_err();
        let _ = fs::remove_dir_all(&root);
        assert_eq!(unchecked["place_rows"]["kit_tokens_checked"], false);
        assert_eq!(checked["place_rows"]["kit_tokens_checked"], true);
        assert!(same, "the check changes no reply row");
        assert!(another.contains("another corpus"), "{another}");
        assert!(!written, "a refused run writes no rows");
        assert!(no_kit.contains("--kit-tokens needs --places"), "{no_kit}");
        assert!(no_place.contains("needs an artifact with a place read"), "{no_place}");
    }
}
