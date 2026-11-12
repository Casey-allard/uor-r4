//! `row-picks` (claude-354): the integer reader's choice at every scored row of a panel, at the
//! windows a float scoring pass cuts, beside that pass's saved choices.
//!
//! A reply read token by token mixes one early difference with everything that follows from it.
//! Here every row is read with the true text before it, as the float pass reads it, so each row
//! is its own comparison.
//!
//! ```text
//! row-picks --stack FILE.uorlut --tokens tokens.u16 --mask response_mask.u8 --context N
//!           --mark-replies 0|1 --out ROWS.json [--float-rows rows-p-no-memory.json]
//! ```
//!
//! - `--tokens` is a `UORT` corpus file (a 64-byte header, then the ids, two little-endian bytes
//!   each) and `--mask` its response mask, one byte a token.
//! - The windows are the float trainer's `build_document_windows` at `--context`: a document is
//!   cut into back-to-back blocks of `context` targets, each read from at most `context` tokens
//!   back, never past the document's first token.
//! - `--mark-replies 1` marks each reply's start as the chat does (the turn memory moves); `0`
//!   never marks one, the float pass's `no-memory` label.
//! - A row is `carried` where its reply opened before its window's first token. A session cannot
//!   be told of such a start, while the float pass still reads its reply table there, by a key
//!   cut at the window. With `--mark-replies 1` these rows are written but not compared: of the
//!   receipt's counts only `rows`, `carried` and `carried_differ` hold them.
//! - `--float-rows` is the float pass's `rows-*.json`. Its `positions` and `targets` must equal
//!   this program's, or nothing is written: the windows would not be the same rows.
//!
//! `ROWS.json` holds, a scored row: `positions`, `targets`, `nearest` (the reader's choice),
//! `margins` (the L1 distance of the second nearest table row less the nearest's), `steps` (how
//! many reply tokens stand before the row: 0 at a reply's first token), `copies` (1 where the
//! target follows the same token earlier in the window's prompt text) and `carried`. With
//! `--float-rows`, `differ` lists each row whose choice is not the float pass's: the float's
//! token, its rank under the reader's query, how much farther it is than the reader's choice,
//! and whether the row is carried. The output file must not exist. A receipt is printed on
//! stdout.
//!
//! A second mode reads the token at which a decoded reply first leaves the float pass's:
//!
//! ```text
//! row-picks --stack FILE.uorlut --samples ROWS.json --float-replies F.jsonl
//!           --integer-replies I.jsonl --mark-replies 0|1 --out LEFT.json
//! ```
//!
//! - `ROWS.json` is a float score's samples (a `prompt` array a row); the two reply files are
//!   `raw-id-replies` files, one `{"i", "ids"}` line a row. A float reply is read up to and
//!   including its first end-of-reply id.
//! - For each row whose two replies differ, the prompt and the ids both replies share are fed
//!   again. The reader's choice must be the saved integer reply's next id and the table's
//!   nearest row, or nothing is written.
//! - `LEFT.json` holds a row: the step, both ids, the float id's rank under the reader's query,
//!   how much farther it is than the reader's choice, and the reader's margin there.

use std::fs::{self, OpenOptions};
use std::io::Write;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_integer::stack::IntegerStackModel;

const SCHEMA: &str = "uor-r4.row-picks/1";
const BOS: u32 = 0;
const HEADER: usize = 64;
const EOS: u32 = 1;
const USAGE: &str = "usage: row-picks --stack FILE --tokens FILE --mask FILE --context N --mark-replies 0|1 --out FILE [--float-rows FILE]\n   or: row-picks --stack FILE --samples FILE --float-replies FILE --integer-replies FILE --mark-replies 0|1 --out FILE";

type Result<T> = std::result::Result<T, String>;

/// One forward of the float scoring pass: the tokens read from `start`, `inputs` of them, and the
/// corpus positions scored in it.
#[derive(Debug, PartialEq)]
struct Window {
    start: usize,
    inputs: usize,
    scored: Vec<usize>,
}

/// The float trainer's `build_document_windows`, by corpus position.
/// - A document opens at each `bos`.
/// - Its targets are cut into blocks of `context`; a block is read from `context` tokens before
///   its last target, or from the document's first token where that is nearer.
/// - A target is scored where its mask byte is 1 and it is not `bos`; a block with none is left
///   out.
fn windows(tokens: &[u32], mask: &[u8], context: usize, bos: u32) -> Vec<Window> {
    let starts: Vec<usize> = (0..tokens.len()).filter(|&i| tokens[i] == bos).collect();
    let mut out = Vec::new();
    for (doc, &doc_start) in starts.iter().enumerate() {
        let doc_end = starts.get(doc + 1).copied().unwrap_or(tokens.len());
        let mut cursor = doc_start + 1;
        while cursor < doc_end {
            let end = (cursor + context).min(doc_end);
            let start = (end - 1).saturating_sub(context).max(doc_start);
            let scored: Vec<usize> = (cursor..end)
                .filter(|&p| tokens[p] != bos && mask[p] == 1)
                .collect();
            if !scored.is_empty() {
                out.push(Window { start, inputs: end - 1 - start, scored });
            }
            cursor = end;
        }
    }
    out
}

/// The L1 distance between a query and a table row.
fn distance(query: &[i32], row: &[i16]) -> u64 {
    query
        .iter()
        .zip(row)
        .map(|(&x, &t)| u64::from(x.abs_diff(i32::from(t))))
        .sum()
}

/// The nearest table row to `query` (the smaller id on a tie), its distance and the second
/// nearest's distance.
fn two_nearest(table: &[i16], query: &[i32]) -> (u32, u64, u64) {
    let (mut token, mut best, mut second) = (u32::MAX, u64::MAX, u64::MAX);
    for (id, row) in table.chunks_exact(query.len()).enumerate() {
        let sum = distance(query, row);
        if sum < best {
            second = best;
            (token, best) = (id as u32, sum);
        } else if sum < second {
            second = sum;
        }
    }
    (token, best, second)
}

/// `token`'s rank under `query` (1 is the nearest; a tie goes to the smaller id) and its distance.
fn rank_of(table: &[i16], query: &[i32], token: u32) -> (usize, u64) {
    let width = query.len();
    let token = token as usize;
    let own = distance(query, &table[token * width..(token + 1) * width]);
    let better = table
        .chunks_exact(width)
        .enumerate()
        .filter(|(id, row)| {
            let other = distance(query, row);
            *id != token && (other < own || (other == own && *id < token))
        })
        .count();
    (better + 1, own)
}

/// How many reply tokens stand directly before `position`: 0 at a reply's first token.
fn reply_step(mask: &[u8], position: usize) -> usize {
    (0..position).rev().take_while(|&p| mask[p] == 1).count()
}

/// Whether the target at `position` follows the token before it somewhere earlier in the
/// window's prompt text (the float pass's `copies_prompt`).
fn copies(tokens: &[u32], mask: &[u8], start: usize, position: usize) -> bool {
    let (previous, target) = (tokens[position - 1], tokens[position]);
    (start..(position - 1).saturating_sub(1)).any(|j| {
        tokens[j] == previous && tokens[j + 1] == target && mask[j] == 0 && mask[j + 1] == 0
    })
}

/// What a session is told before the input at `at` is stepped, as the chat feeds a turn: nothing
/// inside a reply (`None`); at a prompt token the mark is cleared, and set again where the next
/// token opens a reply (`Some(true)`).
fn turn(mask: &[u8], at: usize) -> Option<bool> {
    (mask[at] == 0).then(|| mask[at + 1] == 1)
}

/// Whether the reply holding `position` opened before the window's first token at `start`: no
/// prompt token stands between them, so [`turn`] marks no start for the row.
fn carried(mask: &[u8], start: usize, position: usize) -> bool {
    (start..position).all(|at| mask[at] == 1)
}

/// How many of `rows` fall in each band, and how many of those are flagged. A row is its value
/// and its flag; a band runs from its edge up to the next edge, the last band without end.
fn bands(rows: &[(u64, bool)], edges: &[u64]) -> Value {
    let cells: Vec<Value> = edges
        .iter()
        .enumerate()
        .map(|(at, &from)| {
            let below = edges.get(at + 1).copied().unwrap_or(u64::MAX);
            let inside: Vec<bool> = rows
                .iter()
                .filter(|(value, _)| *value >= from && *value < below)
                .map(|&(_, flagged)| flagged)
                .collect();
            json!({
                "from": from,
                "rows": inside.len(),
                "flagged": inside.iter().filter(|flagged| **flagged).count(),
            })
        })
        .collect();
    json!(cells)
}

/// The middle value (the lower of two), or none of an empty list.
fn median(mut values: Vec<u64>) -> Option<u64> {
    values.sort_unstable();
    values.get(values.len().saturating_sub(1) / 2).copied()
}

/// The bands of a rank (second, third to fifth, sixth to twentieth, beyond), of a reply step
/// (the first token, the next three, the rest) and of a margin or gap in L1 units at Q12.
const RANK_EDGES: [u64; 4] = [2, 3, 6, 21];
const STEP_EDGES: [u64; 3] = [0, 1, 4];
const MARGIN_EDGES: [u64; 5] = [0, 1000, 4000, 16000, 64000];

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|index| args.get(index + 1))
        .cloned()
}

/// A `UORT` file's ids.
fn corpus_ids(bytes: &[u8], path: &str) -> Result<Vec<u32>> {
    if bytes.len() < HEADER || !bytes.starts_with(b"UORT") || (bytes.len() - HEADER) % 2 != 0 {
        return Err(format!("{path}: not a UORT corpus file"));
    }
    Ok(bytes[HEADER..]
        .chunks_exact(2)
        .map(|pair| u32::from(u16::from_le_bytes([pair[0], pair[1]])))
        .collect())
}

fn u64s(value: &Value, name: &str, path: &str) -> Result<Vec<u64>> {
    value
        .get(name)
        .and_then(Value::as_array)
        .and_then(|ids| ids.iter().map(Value::as_u64).collect::<Option<Vec<u64>>>())
        .ok_or_else(|| format!("{path}: no `{name}` array of whole numbers"))
}

fn u32s(value: Option<&Value>, what: &str) -> Result<Vec<u32>> {
    value
        .and_then(Value::as_array)
        .and_then(|ids| {
            ids.iter()
                .map(|id| id.as_u64().and_then(|id| u32::try_from(id).ok()))
                .collect::<Option<Vec<u32>>>()
        })
        .ok_or_else(|| format!("{what}: not an array of ids"))
}

/// A `raw-id-replies` file's replies, in row order.
fn replies(path: &str) -> Result<Vec<Vec<u32>>> {
    let text = fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .enumerate()
        .map(|(index, line)| {
            let row: Value = serde_json::from_str(line).map_err(|e| format!("{path}: {e}"))?;
            if row.get("i").and_then(Value::as_u64) != Some(index as u64) {
                return Err(format!("{path}: line {index} is not row {index}"));
            }
            u32s(row.get("ids"), &format!("{path}: row {index}"))
        })
        .collect()
}

/// The step at which the reader's reply first leaves the float pass's, the float reply read up
/// to and including its first end-of-reply id; `None` where neither leaves the other.
fn first_leave(trained: &[u32], integer: &[u32]) -> Option<usize> {
    let end = trained.iter().position(|&id| id == EOS).map_or(trained.len(), |at| at + 1);
    let trained = &trained[..end];
    (0..trained.len().min(integer.len())).find(|&step| trained[step] != integer[step])
}

fn zero_or_one(value: &str, name: &str) -> Result<bool> {
    match value {
        "0" => Ok(false),
        "1" => Ok(true),
        other => Err(format!("{name} takes 0 or 1, not {other}")),
    }
}

/// The `--samples` mode: each differing reply's first differing token, read again.
fn leaves(args: &[String]) -> Result<Value> {
    let need = |name: &str| flag(args, name).ok_or_else(|| format!("missing {name}\n{USAGE}"));
    let (stack_path, samples_path, out_path) = (need("--stack")?, need("--samples")?, need("--out")?);
    let (float_path, integer_path) = (need("--float-replies")?, need("--integer-replies")?);
    let mark = zero_or_one(&need("--mark-replies")?, "--mark-replies")?;
    let artifact = fs::read(&stack_path).map_err(|e| format!("{stack_path}: {e}"))?;
    let samples = fs::read(&samples_path).map_err(|e| format!("{samples_path}: {e}"))?;
    let rows: Vec<Value> =
        serde_json::from_slice(&samples).map_err(|e| format!("{samples_path}: {e}"))?;
    let (trained, integer) = (replies(&float_path)?, replies(&integer_path)?);
    if trained.len() != rows.len() || integer.len() != rows.len() {
        return Err(format!(
            "{} rows, {} float replies, {} integer replies",
            rows.len(),
            trained.len(),
            integer.len()
        ));
    }
    let model = IntegerStackModel::parse(&artifact).map_err(|e| format!("artifact: {e}"))?;
    if mark && !model.session().has_turn_memory() {
        return Err("--mark-replies 1 needs an artifact with a turn memory".to_owned());
    }
    let table = match model.fbs_writer_table() {
        Some(table) if model.shape().head_nearest() => table,
        _ => return Err("row-picks needs an artifact with a writer table and the nearest head".to_owned()),
    };
    let mut left = Vec::new();
    for (index, row) in rows.iter().enumerate() {
        let Some(step) = first_leave(&trained[index], &integer[index]) else {
            continue;
        };
        let prompt = u32s(row.get("prompt"), &format!("row {index}'s prompt"))?;
        if prompt.is_empty() {
            return Err(format!("row {index}: empty prompt"));
        }
        let mut session = model.session();
        session.clear_reply_start();
        let mut pick = u32::MAX;
        // As `raw-id-replies` decodes: the prompt's ids but the last are read, the last and each
        // reply id are served.
        for (at, &token) in prompt.iter().chain(&integer[index][..step]).enumerate() {
            if mark && at + 1 == prompt.len() {
                session.mark_reply_start();
            }
            if at + 1 < prompt.len() {
                session
                    .read_step(token)
                    .map_err(|e| format!("row {index}, id {at}: {e}"))?;
            } else {
                (pick, _) = session
                    .serve_step(token)
                    .map_err(|e| format!("row {index}, id {at}: {e}"))?;
            }
        }
        let (ours, theirs) = (integer[index][step], trained[index][step]);
        if pick != ours {
            return Err(format!(
                "row {index}, step {step}: the reader chose {pick}, the saved reply holds {ours}: not this artifact's replies"
            ));
        }
        let query = session
            .last_query_q12()
            .ok_or_else(|| format!("row {index}: the step left no query"))?;
        if theirs as usize >= table.len() / query.len() {
            return Err(format!("row {index}: float id {theirs} is outside the table"));
        }
        let (scan, best, second) = two_nearest(table, query);
        if scan != pick {
            return Err(format!(
                "row {index}, step {step}: the reader chose {pick}, the table's nearest is {scan}"
            ));
        }
        let (rank, own) = rank_of(table, query, theirs);
        left.push(json!({
            "i": index,
            "step": step,
            "integer": ours,
            "float": theirs,
            "rank": rank,
            "gap": own - best,
            "margin": second - best,
        }));
    }
    let of = |name: &str| -> Vec<u64> { left.iter().filter_map(|row| row[name].as_u64()).collect() };
    let all = |values: Vec<u64>| -> Vec<(u64, bool)> { values.into_iter().map(|v| (v, true)).collect() };
    let receipt = json!({
        "schema": SCHEMA,
        "stack_sha256": sha256_hex(&artifact),
        "samples_sha256": sha256_hex(&samples),
        "float_replies_sha256": sha256_hex(&fs::read(&float_path).map_err(|e| format!("{float_path}: {e}"))?),
        "integer_replies_sha256": sha256_hex(&fs::read(&integer_path).map_err(|e| format!("{integer_path}: {e}"))?),
        "mark_replies": mark,
        "rows": rows.len(),
        "left": left.len(),
        "by_rank": bands(&all(of("rank")), &RANK_EDGES),
        "by_step": bands(&all(of("step")), &STEP_EDGES),
        "gap_median": median(of("gap")),
        "margin_median": median(of("margin")),
    });
    let mut body = receipt.clone();
    body["leaves"] = json!(left);
    let mut out = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&out_path)
        .map_err(|e| format!("{out_path}: {e}"))?;
    out.write_all(body.to_string().as_bytes())
        .map_err(|e| format!("{out_path}: {e}"))?;
    Ok(receipt)
}

fn main() {
    match run(std::env::args().skip(1).collect()) {
        Ok(receipt) => println!("{receipt}"),
        Err(error) => {
            eprintln!("row-picks: {error}");
            std::process::exit(1);
        }
    }
}

fn run(args: Vec<String>) -> Result<Value> {
    if let Some(odd) = args.chunks(2).find(|pair| pair.len() != 2 || !pair[0].starts_with("--")) {
        return Err(format!("{}: every argument is a `--name value` pair\n{USAGE}", odd[0]));
    }
    if flag(&args, "--samples").is_some() {
        return leaves(&args);
    }
    let need = |name: &str| flag(&args, name).ok_or_else(|| format!("missing {name}\n{USAGE}"));
    let (stack_path, tokens_path, mask_path) = (need("--stack")?, need("--tokens")?, need("--mask")?);
    let out_path = need("--out")?;
    let context = need("--context")?
        .parse::<usize>()
        .ok()
        .filter(|n| *n > 0)
        .ok_or_else(|| "--context takes a positive integer".to_owned())?;
    let mark = zero_or_one(&need("--mark-replies")?, "--mark-replies")?;
    let float_path = flag(&args, "--float-rows");

    let artifact = fs::read(&stack_path).map_err(|e| format!("{stack_path}: {e}"))?;
    let token_bytes = fs::read(&tokens_path).map_err(|e| format!("{tokens_path}: {e}"))?;
    let mask = fs::read(&mask_path).map_err(|e| format!("{mask_path}: {e}"))?;
    let tokens = corpus_ids(&token_bytes, &tokens_path)?;
    if mask.len() != tokens.len() {
        return Err(format!("{mask_path}: {} mask bytes for {} tokens", mask.len(), tokens.len()));
    }
    let model = IntegerStackModel::parse(&artifact).map_err(|e| format!("artifact: {e}"))?;
    if mark && !model.session().has_turn_memory() {
        return Err("--mark-replies 1 needs an artifact with a turn memory".to_owned());
    }
    let table = match model.fbs_writer_table() {
        Some(table) if model.shape().head_nearest() => table,
        _ => return Err("row-picks needs an artifact with a writer table and the nearest head".to_owned()),
    };
    let trained = float_path
        .as_deref()
        .map(|path| -> Result<(Vec<u64>, Vec<u64>, Vec<u64>, String)> {
            let bytes = fs::read(path).map_err(|e| format!("{path}: {e}"))?;
            let rows: Value = serde_json::from_slice(&bytes).map_err(|e| format!("{path}: {e}"))?;
            Ok((
                u64s(&rows, "positions", path)?,
                u64s(&rows, "targets", path)?,
                u64s(&rows, "nearest", path)?,
                sha256_hex(&bytes),
            ))
        })
        .transpose()?;

    let cut = windows(&tokens, &mask, context, BOS);
    let (mut positions, mut targets, mut nearest) = (Vec::new(), Vec::new(), Vec::new());
    let (mut margins, mut steps, mut copied, mut differ) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    // A row's flag: whether its choice is not the float pass's. `kept` is whether the row is
    // compared, `late` whether it is carried; `ranks` holds the compared differing rows.
    let (mut flags, mut kept, mut late, mut ranks) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let (mut integer_right, mut float_right) = (0usize, 0usize);
    for window in &cut {
        let mut session = model.session();
        session.clear_reply_start();
        let mut scored = window.scored.iter().copied().peekable();
        for offset in 0..window.inputs {
            let at = window.start + offset;
            let position = at + 1;
            if mark {
                if let Some(opens) = turn(&mask, at) {
                    session.clear_reply_start();
                    if opens {
                        session.mark_reply_start();
                    }
                }
            }
            if scored.peek() != Some(&position) {
                session
                    .read_step(tokens[at])
                    .map_err(|e| format!("position {at}: {e}"))?;
                continue;
            }
            scored.next();
            let (pick, _) = session
                .serve_step(tokens[at])
                .map_err(|e| format!("position {at}: {e}"))?;
            let query = session
                .last_query_q12()
                .ok_or_else(|| format!("position {at}: the step left no query"))?;
            let (scan, best, second) = two_nearest(table, query);
            if scan != pick {
                return Err(format!("position {at}: the reader chose {pick}, the table's nearest is {scan}"));
            }
            let is_carried = carried(&mask, window.start, position);
            // A carried row of a marked run is read with no reply start: not the float's row.
            let compared = !(mark && is_carried);
            let mut differs = false;
            if let Some((_, _, float_nearest, _)) = &trained {
                let theirs = float_nearest.get(positions.len()).copied().unwrap_or(u64::MAX);
                if theirs != u64::from(pick) {
                    if theirs >= (table.len() / query.len()) as u64 {
                        return Err(format!("position {position}: the float rows hold no id of the table"));
                    }
                    let (rank, own) = rank_of(table, query, theirs as u32);
                    differ.push(json!({
                        "position": position,
                        "target": tokens[position],
                        "integer": pick,
                        "float": theirs,
                        "rank": rank,
                        "gap": own - best,
                        "margin": second - best,
                        "carried": u8::from(is_carried),
                    }));
                    differs = true;
                    if compared {
                        ranks.push((rank as u64, true));
                        integer_right += usize::from(pick == tokens[position]);
                        float_right += usize::from(theirs == u64::from(tokens[position]));
                    }
                }
            }
            flags.push(differs);
            kept.push(compared);
            late.push(u8::from(is_carried));
            positions.push(position as u64);
            targets.push(u64::from(tokens[position]));
            nearest.push(u64::from(pick));
            margins.push(second - best);
            steps.push(reply_step(&mask, position));
            copied.push(u8::from(copies(&tokens, &mask, window.start, position)));
        }
    }
    if let Some((float_positions, float_targets, float_nearest, _)) = &trained {
        if *float_positions != positions || *float_targets != targets || float_nearest.len() != positions.len() {
            return Err(format!(
                "--float-rows: its rows are not this panel's at context {context}: {} rows and {} choices against {}",
                float_positions.len(),
                float_nearest.len(),
                positions.len()
            ));
        }
    }
    // Every count below is over the compared rows.
    let right = |choices: &[u64]| -> usize {
        choices
            .iter()
            .zip(&targets)
            .zip(&kept)
            .filter(|((choice, target), &compared)| compared && choice == target)
            .count()
    };
    let flagged = |values: Vec<u64>| -> Vec<(u64, bool)> {
        values
            .into_iter()
            .zip(flags.iter().copied())
            .zip(&kept)
            .filter(|(_, &compared)| compared)
            .map(|(row, _)| row)
            .collect()
    };
    let of_differ = |name: &str| -> Vec<u64> {
        differ
            .iter()
            .filter(|row| !(mark && row["carried"] == 1))
            .filter_map(|row| row[name].as_u64())
            .collect()
    };
    // With `--float-rows`: the rows and the differing rows of each band.
    let breakdown = trained.as_ref().map(|_| {
        json!({
            "differ_by_rank": bands(&ranks, &RANK_EDGES),
            "by_margin": bands(&flagged(margins.clone()), &MARGIN_EDGES),
            "by_step": bands(&flagged(steps.iter().map(|&s| s as u64).collect()), &STEP_EDGES),
            "by_copy": bands(&flagged(copied.iter().map(|&c| u64::from(c)).collect()), &[0, 1]),
            "differ_integer_right": integer_right,
            "differ_float_right": float_right,
            "differ_gap_median": median(of_differ("gap")),
            "differ_margin_median": median(of_differ("margin")),
            "margin_median": median(flagged(margins.clone()).into_iter().map(|(margin, _)| margin).collect()),
        })
    });
    let receipt = json!({
        "breakdown": breakdown,
        "schema": SCHEMA,
        "stack_sha256": sha256_hex(&artifact),
        "tokens_sha256": sha256_hex(&token_bytes),
        "mask_sha256": sha256_hex(&mask),
        "context": context,
        "mark_replies": mark,
        "windows": cut.len(),
        "turn_memory": model
            .turn_memory_table_lengths()
            .map(|(opening, reply)| json!({"opening": opening, "reply": reply})),
        "rows": positions.len(),
        "carried": late.iter().filter(|&&c| c == 1).count(),
        "compared": kept.iter().filter(|&&compared| compared).count(),
        "nearest_hits": right(&nearest),
        "float_rows_sha256": trained.as_ref().map(|(_, _, _, sha)| sha.clone()),
        "float_nearest_hits": trained.as_ref().map(|(_, _, choices, _)| right(choices)),
        "differ_from_float": trained.as_ref().map(|_| ranks.len()),
        "carried_differ": trained
            .as_ref()
            .map(|_| differ.iter().filter(|row| row["carried"] == 1).count()),
    });
    let mut body = receipt.clone();
    for (name, value) in [
        ("positions", json!(positions)),
        ("targets", json!(targets)),
        ("nearest", json!(nearest)),
        ("margins", json!(margins)),
        ("steps", json!(steps)),
        ("copies", json!(copied)),
        ("carried", json!(late)),
        ("differ", json!(differ)),
    ] {
        body[name] = value;
    }
    let mut out = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&out_path)
        .map_err(|e| format!("{out_path}: {e}"))?;
    out.write_all(body.to_string().as_bytes())
        .map_err(|e| format!("{out_path}: {e}"))?;
    Ok(receipt)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two documents at context 4: a short one in one window, and one of six targets whose last
    /// block, two targets long, slides back to read a full window.
    #[test]
    fn the_windows_are_the_float_passs() {
        // Document 1 is the first four tokens, document 2 the seven after them.
        let tokens = [0, 5, 6, 7, 0, 5, 6, 7, 8, 9, 5];
        let mask = [0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1];
        let cut = windows(&tokens, &mask, 4, 0);
        assert_eq!(
            cut,
            vec![
                Window { start: 0, inputs: 3, scored: vec![2, 3] },
                // The first four targets, read from the document's first token.
                Window { start: 4, inputs: 4, scored: vec![7, 8] },
                // The last two targets: read from four tokens before the last, not from the
                // block's own first.
                Window { start: 6, inputs: 4, scored: vec![9, 10] },
            ]
        );
        // A block with no scored target is left out.
        let quiet = [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1];
        assert_eq!(windows(&tokens, &quiet, 4, 0).len(), 1);
    }

    #[test]
    fn a_row_is_labelled_by_its_reply_step_and_its_copy() {
        let tokens = [0, 5, 6, 9, 5, 6, 7];
        let mask = [0, 0, 0, 0, 1, 1, 1];
        assert_eq!(reply_step(&mask, 4), 0);
        assert_eq!(reply_step(&mask, 6), 2);
        // Position 5's target 6 follows a 5 in the prompt (positions 1, 2).
        assert!(copies(&tokens, &mask, 0, 5));
        // Position 6's target 7 follows no earlier 6 in the prompt.
        assert!(!copies(&tokens, &mask, 0, 6));
        // A token doubled at the prompt's end is no copy: the pair read ends before the row's
        // own input.
        assert!(!copies(&[0, 5, 5, 5], &[0, 0, 0, 1], 0, 3));
    }

    /// A reply cut by a block: the rows of its second block are carried, and they are the rows
    /// no mark reaches.
    #[test]
    fn a_row_is_carried_where_no_reply_start_lies_in_its_window() {
        let tokens = [0, 5, 6, 7, 8, 9, 5, 6];
        let mask = [0, 0, 1, 1, 1, 1, 0, 1];
        // At context 2 the second window is read from position 2, inside the reply.
        let cut = windows(&tokens, &mask, 2, 0);
        assert_eq!(cut[1], Window { start: 2, inputs: 2, scored: vec![3, 4] });
        assert!(!carried(&mask, 0, 2));
        assert!(carried(&mask, 2, 3) && carried(&mask, 2, 4));
        // What the session is told before each input: nothing inside a reply.
        assert_eq!(
            (0..7).map(|at| turn(&mask, at)).collect::<Vec<_>>(),
            vec![Some(false), Some(true), None, None, None, None, Some(true)]
        );
        // A reply row is carried exactly where the marks leave the session unmarked.
        for start in 0..mask.len() {
            for position in start + 1..mask.len() {
                let mut marked = false;
                for at in start..position {
                    if let Some(opens) = turn(&mask, at) {
                        marked = opens;
                    }
                }
                if mask[position] == 1 {
                    assert_eq!(carried(&mask, start, position), !marked, "{start} to {position}");
                }
            }
        }
    }

    #[test]
    fn rows_are_counted_in_their_band_with_their_flags() {
        let rows = [(0, false), (999, true), (1000, true), (70000, false), (3, true)];
        assert_eq!(
            bands(&rows, &[0, 1000, 64000]),
            json!([
                {"from": 0, "rows": 3, "flagged": 2},
                {"from": 1000, "rows": 1, "flagged": 1},
                {"from": 64000, "rows": 1, "flagged": 0},
            ])
        );
        assert_eq!(median(vec![9, 1, 5, 7]), Some(5));
        assert_eq!(median(Vec::new()), None);
    }

    #[test]
    fn a_reply_leaves_at_its_first_other_id_before_the_floats_end() {
        // The float reply ends at its first end-of-reply id; what stands after it is not read.
        assert_eq!(first_leave(&[5, 6, EOS, 9, 9], &[5, 6, EOS]), None);
        assert_eq!(first_leave(&[5, 6, EOS, 9, 9], &[5, 7, EOS]), Some(1));
        assert_eq!(first_leave(&[5, 6, 7], &[5, 6, EOS]), Some(2));
        assert_eq!(first_leave(&[5, 6, EOS], &[5, 6, 7, 8]), Some(2));
    }

    #[test]
    fn the_scan_names_the_nearest_the_second_and_a_rank() {
        // Three rows of width 2; the query is nearest row 1, then row 0, then row 2.
        let table = [0i16, 0, 10, 10, 40, 40];
        let query = [9i32, 9];
        assert_eq!(two_nearest(&table, &query), (1, 2, 18));
        assert_eq!(rank_of(&table, &query, 1), (1, 2));
        assert_eq!(rank_of(&table, &query, 0), (2, 18));
        assert_eq!(rank_of(&table, &query, 2), (3, 62));
    }
}
