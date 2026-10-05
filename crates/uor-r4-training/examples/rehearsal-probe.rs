//! Step 0c (barrier assessment 2026-10-05, #820): teacher-forced rehearsal
//! probe of the D19 MQAR rows, log recall off, the network alone. Training-free.
//!
//! ```text
//! rehearsal-probe out=NEW_REPORT_ROOT model_root=RUN_ROOT tokenizer=TOKENIZER.json \
//!   compiler=COMPILER.json [trunk=OP_MODEL_DIR] [op_policy=unless_query] \
//!   [conversations=300] [seed=9101] [max_new_tokens=32]
//! ```
//!
//! The prompt of every MQAR query is the one the grounded session serves
//! (`m-world session world=v2 log_recall=off`): the same development draw
//! (seed, conversations), the same compiler, trunk and op policy, the same
//! limits (whole completed turns, `max_new_tokens`), and each turn after the
//! session's own generated history. Only MQAR conversations are run through the
//! session (the draw still walks all of them, so the RNG matches). At the MQAR
//! query the session's recorded emitter input is the prompt, and its greedy
//! reply is the free run (i). On the same prompt and the same float weights
//! (pointer mixture included, as `greedy_reply` scores) the probe then forces,
//! BPE-exactly, the start of an assistant reply and reads the greedy next
//! pieces:
//!
//! - `is` (ii): "{K} is" from the gold rehearsal "{K} is {v}." (K capitalized
//!   as the world writes it);
//! - `eq`, `colon` (iii): the copula swapped, "{K} = {v}." and "{K}: {v}.";
//! - `is_lc` (extra): "{k} is" with the key lowercase, so that its pieces are
//!   exactly the assertion's (the world capitalizes the reply's first word,
//!   whose pieces then differ from the assertion's);
//! - `bare` (extra): "It's", the world's bare reply form, with no key cue.
//!
//! This tokenizer writes a numeral as a bare " " piece and then digits, so
//! `first` (the raw first value piece) is trivially the space for a numeric
//! value; `first_content` also forces the value's leading whitespace-only gold
//! pieces and scores the next one, the value's first content piece.
//!
//! "BPE-exactly": the whole gold reply is encoded as the protocol encodes an
//! assistant message (one segment, a leading space under protocol 2) and cut at
//! the piece boundary before the value; a reply whose pieces do not split
//! there is counted as `boundary_unclean` and not scored. A forced arm scores
//! `first`: the greedy next piece is the value's first gold piece, and `full`:
//! the next greedy pieces are exactly the value's gold pieces. The free run
//! scores the session's own judge (`judge_v2`) and whether the value appears,
//! and splits by reply form (rehearse, bare, other).
//!
//! Every root is claimed before anything is loaded and sealed at the end. Set
//! RAYON_NUM_THREADS to bound the threads.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::Device;
use serde_json::{json, Value};
use uor_r4_core::report_output;
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::geometric_stack::StackModel;
use uor_r4_training::milestone_world::Split;
use uor_r4_training::milestone_world_v2::{judge_v2, Category2, Kind, MWorld2, Mix};
use uor_r4_training::relation_compiler::{OpPolicy, SavedCompiler, Trunk};
use uor_r4_training::stack_checkpoint::{
    save_checkpoint, sealed_manifest_sha256, CheckpointIdentity, DataIdentity,
};
use uor_r4_training::stack_dialogue::greedy_reply;
use uor_r4_training::stack_grounded_session::{
    ContextPolicy, GroundedSession, RecallDisposition, SessionLimits, SessionScope, TurnControls,
};
use uor_r4_training::stack_store::StackStore;
use uor_r4_training::stack_tracking::Rng;
use uor_r4_training::temporal_compiler::GroundedCompiler;
use uor_r4_training::{sha256_file, Result, TrainingError};

fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
}

const ALLOWED: &[&str] = &[
    "out",
    "model_root",
    "tokenizer",
    "compiler",
    "trunk",
    "op_policy",
    "conversations",
    "seed",
    "max_new_tokens",
];

/// The forced arms: name, and the gold reply text whose pieces before the
/// value are forced (`{K}` the capitalized key, `{v}` the value).
const ARMS: &[(&str, &str)] = &[
    ("is", "{K} is {v}."),
    ("is_lc", "{k} is {v}."),
    ("eq", "{K} = {v}."),
    ("colon", "{K}: {v}."),
    ("bare", "It's {v}."),
];

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// The key of a query from its unfilled template ("Remind me what {k} was.")
/// and its text (the template filled, first letter capitalized).
fn key_from_query(template: &str, query: &str) -> Option<String> {
    let (before, after) = template.split_once("{k}")?;
    if query.len() < before.len() + after.len() {
        return None;
    }
    let head = &query[..before.len()];
    let tail = &query[query.len() - after.len()..];
    if !head.eq_ignore_ascii_case(before) || tail != after {
        return None;
    }
    let key = &query[before.len()..query.len() - after.len()];
    (!key.is_empty()).then(|| key.to_owned())
}

/// A gold reply cut at the value: the forced pieces, the value's gold pieces,
/// and whether the reply's pieces split exactly there.
#[derive(Debug, PartialEq, Eq)]
struct Cut {
    forced: Vec<u32>,
    value: Vec<u32>,
}

/// Cut `reply` (encoded as one segment, as the dialogue protocol encodes an
/// assistant message) before `value`, whose first occurrence after `prefix`
/// starts the value. `None` when no piece boundary falls exactly at the
/// value's start and end.
fn cut(
    tokenizer: &ByteBpeTokenizer,
    spaced: bool,
    prefix: &str,
    value: &str,
) -> Option<(Cut, String)> {
    let reply = format!("{prefix} {value}.");
    let segment = if spaced {
        format!(" {reply}")
    } else {
        reply.clone()
    };
    let ids = tokenizer.encode(&segment);
    let forced_text = if spaced {
        format!(" {prefix}")
    } else {
        prefix.to_owned()
    };
    let value_end_text = format!("{forced_text} {value}");
    let mut forced_len = None;
    let mut value_len = None;
    for n in 0..=ids.len() {
        let text = tokenizer.decode(&ids[..n]);
        if text == forced_text {
            forced_len = Some(n);
        }
        if text == value_end_text {
            value_len = Some(n);
        }
    }
    let (f, v) = (forced_len?, value_len?);
    if v <= f {
        return None;
    }
    Some((
        Cut {
            forced: ids[..f].to_vec(),
            value: ids[f..v].to_vec(),
        },
        reply,
    ))
}

/// The free run's reply form.
fn reply_form(reply: &str, key: &str) -> &'static str {
    let lower = reply.trim_start().to_lowercase();
    let key = key.to_lowercase();
    if lower.starts_with(&format!("{key} is"))
        || lower.starts_with(&format!("{key} was"))
        || lower.starts_with(&format!("{key}:"))
        || lower.starts_with(&format!("{key} ="))
    {
        "rehearse"
    } else if ["it's", "that's", "it is", "that is", "it was"]
        .iter()
        .any(|lead| lower.starts_with(lead))
    {
        "bare"
    } else {
        "other"
    }
}

/// Whether `needle` occurs in `hay` as a contiguous run.
fn contains_run(hay: &[u32], needle: &[u32]) -> bool {
    !needle.is_empty() && hay.windows(needle.len()).any(|w| w == needle)
}

/// The greedy token of a score vector: the highest, the lowest id on a tie
/// (as `greedy_reply` picks it).
fn argmax(scores: &[f32]) -> (u32, usize) {
    let mut best = 0usize;
    for (i, v) in scores.iter().enumerate() {
        if *v > scores[best] {
            best = i;
        }
    }
    (best as u32, best)
}

/// Rank (0 = greedy) of `id` under `scores`, ties broken as `argmax` does.
fn rank(scores: &[f32], id: u32) -> usize {
    let target = scores[id as usize];
    scores
        .iter()
        .enumerate()
        .filter(|(i, v)| **v > target || (**v == target && (*i as u32) < id))
        .count()
}

struct Args(BTreeMap<String, String>);

impl Args {
    fn parse(arguments: &[String]) -> Result<Self> {
        let mut pairs = BTreeMap::new();
        for argument in arguments {
            let (key, value) = argument
                .split_once('=')
                .ok_or_else(|| invalid(format!("arguments are key=value, got {argument}")))?;
            if !ALLOWED.contains(&key) {
                return Err(invalid(format!("unknown argument {key}=")));
            }
            pairs.insert(key.to_owned(), value.to_owned());
        }
        Ok(Self(pairs))
    }
    fn required(&self, key: &str) -> Result<String> {
        self.0
            .get(key)
            .cloned()
            .ok_or_else(|| invalid(format!("missing {key}=")))
    }
    fn optional(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }
    fn number<T: std::str::FromStr>(&self, key: &str, default: T) -> Result<T> {
        match self.0.get(key) {
            None => Ok(default),
            Some(text) => text
                .parse()
                .map_err(|_| invalid(format!("{key}= is not a number: {text}"))),
        }
    }
}

#[derive(Default)]
struct Tally(BTreeMap<String, (usize, usize)>);

impl Tally {
    fn add(&mut self, key: String, pass: bool) {
        let entry = self.0.entry(key).or_default();
        entry.0 += usize::from(pass);
        entry.1 += 1;
    }
    fn to_json(&self) -> Value {
        Value::Object(
            self.0
                .iter()
                .map(|(key, &(pass, of))| {
                    (
                        key.clone(),
                        json!({"pass": pass, "of": of,
                               "rate": if of == 0 { 0.0 } else { pass as f64 / of as f64 }}),
                    )
                })
                .collect(),
        )
    }
}

fn run(args: &Args, out: &Path) -> Result<()> {
    let started = Instant::now();
    let model_root = PathBuf::from(args.required("model_root")?);
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let compiler_path = PathBuf::from(args.required("compiler")?);
    let trunk_directory = args.optional("trunk").map(PathBuf::from);
    let op_policy_text = args.optional("op_policy").unwrap_or_else(|| "op".into());
    let op_policy = OpPolicy::parse(&op_policy_text)?;
    if op_policy != OpPolicy::Op && trunk_directory.is_none() {
        return Err(invalid("op_policy= needs the op model's trunk="));
    }
    let conversations: usize = args.number("conversations", 300)?;
    let seed: u64 = args.number("seed", 9_101)?;
    let max_new_tokens: usize = args.number("max_new_tokens", 32)?;

    let tokenizer_json = fs::read(&tokenizer_path)?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_json)
        .ok_or_else(|| invalid("tokenizer JSON is not a supported byte-level BPE"))?;
    let compiler_bytes = fs::read(&compiler_path)?;
    let compiler = match &trunk_directory {
        Some(directory) => GroundedCompiler::Legacy(
            SavedCompiler::load(
                compiler_bytes.clone(),
                Some(Trunk::load(directory, &tokenizer_json, &Device::Cpu)?),
            )?
            .with_op_policy(op_policy)?,
        ),
        None => GroundedCompiler::from_bytes(compiler_bytes.clone())?,
    };
    let device = Device::Cpu;

    // The emitter as a sealed inference checkpoint with an empty store, as
    // `m-world session` builds it.
    let training_report: Value =
        serde_json::from_slice(&fs::read(model_root.join("report.json"))?)?;
    let mut data = Vec::new();
    for input in training_report["inputs"]["train"]
        .as_array()
        .ok_or_else(|| invalid("the model's report records no training inputs"))?
    {
        let path = PathBuf::from(input["path"].as_str().unwrap_or_default());
        let label = match (path.parent().and_then(Path::file_name), path.file_name()) {
            (Some(parent), Some(name)) => {
                format!("{}/{}", parent.to_string_lossy(), name.to_string_lossy())
            }
            _ => return Err(invalid("a training input has no file name")),
        };
        data.push(DataIdentity {
            label,
            bytes: input["bytes"]
                .as_u64()
                .ok_or_else(|| invalid("a training input has no byte count"))?,
            sha256: input["sha256"]
                .as_str()
                .ok_or_else(|| invalid("a training input has no digest"))?
                .to_owned(),
        });
    }
    let protocol_version = match training_report["settings"]["protocol"].as_u64() {
        None => 1u8,
        Some(2) => 2,
        Some(other) => {
            return Err(invalid(format!(
                "the emitter's training protocol {other} is unknown"
            )))
        }
    };
    let spaced = protocol_version == 2;
    let identity = CheckpointIdentity::from_tokenizer_version(
        &tokenizer_json,
        protocol_version,
        data,
        sealed_manifest_sha256(&model_root).map_err(|e| invalid(e.to_string()))?,
    )
    .map_err(|e| invalid(e.to_string()))?;
    let eos = identity.protocol.eos_id;
    let model_dir = model_root.join("model");
    if StackModel::saved_served_representation(&model_dir)?.is_some() {
        return Err(invalid("a served representation is refused"));
    }
    let mut model = StackModel::load(&model_dir, &device)?;
    let snap = StackModel::saved_transport_snap(&model_dir)?;
    model.set_transport_snap(snap)?;
    let mut model_files = serde_json::Map::new();
    for name in ["config.json", "model.safetensors", "transport.json"] {
        let path = model_dir.join(name);
        if path.exists() {
            model_files.insert(name.into(), json!(sha256_file(&path)?));
        }
    }
    let checkpoint = out.join("checkpoint");
    let store = StackStore::new(1, 8).map_err(|e| invalid(e.to_string()))?;
    save_checkpoint(&checkpoint, &model, &identity, Some(&store))
        .map_err(|e| invalid(e.to_string()))?;
    let scope = SessionScope {
        scope: b"m-world-v2".to_vec(),
        entity: tokenizer.encode("user"),
    };
    let limits = SessionLimits {
        max_new_tokens,
        max_turns: 64,
        max_source_bytes: 1 << 16,
        max_history_tokens: 1 << 20,
        max_store_records: 4_096,
        context_policy: ContextPolicy::WholeCompletedTurns,
    };
    let context = model.config.context;

    let count = |text: &str| tokenizer.encode(text).len();
    let mut world = MWorld2::new(&count, Mix::default())?;
    let mut rng = Rng::new(seed);
    let drawn: Vec<_> = (0..conversations)
        .map(|_| world.conversation(&mut rng, Split::Development))
        .collect::<Result<_>>()?;

    let mut rows = Vec::new();
    let mut tally = Tally::default();
    let mut session_errors = Vec::new();
    for (index, conversation) in drawn.iter().enumerate() {
        if conversation.kind != Kind::Mqar {
            continue;
        }
        let mut session = GroundedSession::from_checkpoint_path(
            &checkpoint,
            tokenizer_json.clone(),
            compiler.clone(),
            scope.clone(),
            limits.clone(),
            &device,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let assertion = conversation
            .turns
            .iter()
            .find(|t| t.intent == "mqar_assert")
            .map(|t| t.user.clone())
            .unwrap_or_default();
        for turn in &conversation.turns {
            let outcome = match session.turn_with_controls(&turn.user, TurnControls::default()) {
                Ok(outcome) => outcome,
                Err(error) => {
                    session_errors.push(json!({
                        "conversation": index, "user": turn.user, "error": error.to_string(),
                    }));
                    if turn.category == Category2::Mqar {
                        tally.add("session_error".into(), true);
                    }
                    continue;
                }
            };
            if turn.category != Category2::Mqar {
                continue;
            }
            let Some(mqar) = turn.tag.mqar.as_ref() else {
                continue;
            };
            let value = turn
                .tag
                .answer
                .clone()
                .ok_or_else(|| invalid("an MQAR query has no answer"))?;
            let template = turn.tag.template.clone().unwrap_or_default();
            let key = key_from_query(&template, &turn.user)
                .ok_or_else(|| invalid(format!("no key in {:?}", turn.user)))?;
            let key_asserted = assertion.contains(&format!("{key} is {value}"));
            let prompt = outcome.emitter_input_ids.clone();
            let recall = match &outcome.recall {
                RecallDisposition::NotRequested => "not_requested",
                RecallDisposition::Value => "value",
                RecallDisposition::LogValue => "log_value",
                RecallDisposition::Absent => "absent",
                RecallDisposition::Disabled => "disabled",
                RecallDisposition::Unsupported { .. } => "unsupported",
            };
            // (i) the free run: the session's own greedy reply.
            let reply = outcome.reply_text.clone();
            let pass = judge_v2(&turn.checks, &turn.user, &reply);
            let appears = reply.to_lowercase().contains(&value.to_lowercase());
            let form = reply_form(&reply, &key);
            let key_cap = capitalize(&key);
            let key_lower_pieces = tokenizer.encode(&format!(" {key}"));
            let key_cap_pieces = tokenizer.encode(&format!(" {key_cap}"));
            let distance = mqar.target_distance;
            let kp = key_cap_pieces.len().min(4);
            let kp_label = if kp >= 4 {
                "4+".to_owned()
            } else {
                kp.to_string()
            };
            for (scope_name, scope_key) in [
                ("all", String::new()),
                ("distance", format!("/{distance}")),
                ("key_pieces", format!("/{kp_label}")),
            ] {
                let prefix = format!("free/{scope_name}{scope_key}");
                tally.add(format!("{prefix}/judge_pass"), pass);
                tally.add(format!("{prefix}/value_appears"), appears);
                for f in ["rehearse", "bare", "other"] {
                    tally.add(format!("{prefix}/form_{f}"), form == f);
                }
            }
            tally.add(format!("free/by_form/{form}/judge_pass"), pass);
            tally.add(format!("free/by_form/{form}/value_appears"), appears);

            let mut arms = serde_json::Map::new();
            for (arm, pattern) in ARMS {
                let prefix_text = pattern
                    .replace("{K}", &key_cap)
                    .replace("{k}", &key)
                    .replace(" {v}.", "")
                    .replace("{v}.", "");
                let Some((piece_cut, gold_reply)) = cut(&tokenizer, spaced, &prefix_text, &value)
                else {
                    tally.add(format!("{arm}/boundary_unclean"), true);
                    arms.insert((*arm).into(), json!({"boundary_unclean": true}));
                    continue;
                };
                tally.add(format!("{arm}/boundary_unclean"), false);
                let mut input = prompt.clone();
                input.extend(&piece_cut.forced);
                let room = context.saturating_sub(input.len());
                let cap = (piece_cut.value.len() + 4).min(room);
                if cap < piece_cut.value.len() {
                    tally.add(format!("{arm}/no_room"), true);
                    arms.insert((*arm).into(), json!({"no_room": true}));
                    continue;
                }
                let scores = model.next_scores(&input)?;
                let (top, _) = argmax(&scores);
                let gold_first = piece_cut.value[0];
                let first = top == gold_first;
                let gold_rank = rank(&scores, gold_first);
                // The value's first content piece: leading whitespace-only gold
                // pieces (a numeral's " " under this tokenizer) are forced too.
                let lead = piece_cut
                    .value
                    .iter()
                    .take_while(|id| tokenizer.decode(&[**id]).trim().is_empty())
                    .count()
                    .min(piece_cut.value.len() - 1);
                let mut content_input = input.clone();
                content_input.extend(&piece_cut.value[..lead]);
                let content_scores = model.next_scores(&content_input)?;
                let (content_top, _) = argmax(&content_scores);
                let gold_content = piece_cut.value[lead];
                let first_content = content_top == gold_content;
                let content_rank = rank(&content_scores, gold_content);
                let continuation = greedy_reply(&model, &input, cap, eos)?;
                let full = continuation.ids.len() >= piece_cut.value.len()
                    && continuation.ids[..piece_cut.value.len()] == piece_cut.value[..];
                let text_end = continuation.ids.len()
                    - usize::from(continuation.eos && !continuation.ids.is_empty());
                let continued = tokenizer.decode(&continuation.ids[..text_end]);
                let text_value = continued
                    .trim_start()
                    .to_lowercase()
                    .starts_with(&value.to_lowercase());
                let value_pieces = piece_cut.value.len().min(3);
                let vp_label = if value_pieces >= 3 {
                    "3+".to_owned()
                } else {
                    value_pieces.to_string()
                };
                for (scope_name, scope_key) in [
                    ("all", String::new()),
                    ("distance", format!("/{distance}")),
                    ("key_pieces", format!("/{kp_label}")),
                    ("value_pieces", format!("/{vp_label}")),
                    ("free_form", format!("/{form}")),
                ] {
                    let p = format!("{arm}/{scope_name}{scope_key}");
                    tally.add(format!("{p}/first"), first);
                    tally.add(format!("{p}/first_content"), first_content);
                    tally.add(format!("{p}/content_top5"), content_rank < 5);
                    tally.add(format!("{p}/full"), full);
                    tally.add(format!("{p}/text_value"), text_value);
                    tally.add(format!("{p}/gold_top5"), gold_rank < 5);
                }
                arms.insert(
                    (*arm).into(),
                    json!({
                        "gold_reply": gold_reply,
                        "forced_ids": piece_cut.forced,
                        "forced_text": tokenizer.decode(&piece_cut.forced),
                        "value_ids": piece_cut.value,
                        "greedy_first_id": top,
                        "greedy_first_text": tokenizer.decode(&[top]),
                        "gold_first_rank": gold_rank,
                        "forced_whitespace_value_pieces": lead,
                        "greedy_content_id": content_top,
                        "greedy_content_text": tokenizer.decode(&[content_top]),
                        "gold_content_rank": content_rank,
                        "first": first, "first_content": first_content,
                        "full": full, "text_value": text_value,
                        "continuation": continued,
                    }),
                );
            }
            println!(
                "mw2-{index:04} D{distance} n{} key {key:?} value {value:?}: free {:?} pass={pass} | content-first is={} is_lc={} eq={} colon={}",
                mqar.n,
                reply,
                arms.get("is").and_then(|a| a.get("first_content")).cloned().unwrap_or(Value::Null),
                arms.get("is_lc").and_then(|a| a.get("first_content")).cloned().unwrap_or(Value::Null),
                arms.get("eq").and_then(|a| a.get("first_content")).cloned().unwrap_or(Value::Null),
                arms.get("colon").and_then(|a| a.get("first_content")).cloned().unwrap_or(Value::Null),
            );
            rows.push(json!({
                "id": format!("mw2-{index:04}"),
                "n": mqar.n, "target_distance": distance, "distance": mqar.distance,
                "queried": mqar.queried,
                "query": turn.user, "template": template, "key": key, "value": value,
                "key_asserted_verbatim": key_asserted,
                "key_pieces_reply_form": key_cap_pieces.len(),
                "key_pieces_assertion_form": key_lower_pieces.len(),
                "key_reply_tuple_in_prompt": contains_run(&prompt, &key_cap_pieces),
                "key_assertion_tuple_in_prompt": contains_run(&prompt, &key_lower_pieces),
                "prompt_tokens": prompt.len(),
                "retained_from_turn": outcome.retained_from_turn,
                "recall": recall,
                "free": {"reply": reply, "judge_pass": pass, "value_appears": appears, "form": form},
                "forced": arms,
            }));
        }
    }
    let executable = std::env::current_exe()?;
    let report = json!({
        "schema": "uor-r4.step0c-rehearsal-probe/1",
        "card": "Step 0c teacher-forced rehearsal probe, #820",
        "executable_sha256": sha256_file(&executable)?,
        "world": "m-world-v2",
        "world_digest": MWorld2::digest(),
        "split": "development",
        "seed": seed,
        "conversations": conversations,
        "log_recall": "off",
        "model_root": model_root.display().to_string(),
        "model_files_sha256": model_files,
        "transport_snap": format!("{snap:?}"),
        "context": context,
        "dialogue_protocol_version": protocol_version,
        "tokenizer_sha256": sha256_file(&tokenizer_path)?,
        "compiler": compiler_path.display().to_string(),
        "compiler_sha256": sha256_file(&compiler_path)?,
        "trunk": trunk_directory.as_ref().map(|d| d.display().to_string()),
        "op_policy": op_policy_text,
        "limits": limits,
        "arms": ARMS.iter().map(|(name, pattern)| json!({"name": name, "gold_reply": pattern})).collect::<Vec<_>>(),
        "mqar_rows": rows.len(),
        "tally": tally.to_json(),
        "session_errors": session_errors,
        "rows": rows,
        "wall_seconds": started.elapsed().as_secs_f64(),
    });
    fs::write(
        out.join("rehearsal_probe.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!(
        "{} MQAR rows in {:.0} s",
        report["mqar_rows"],
        started.elapsed().as_secs_f64()
    );
    Ok(())
}

fn main() -> Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let args = Args::parse(&arguments)?;
    let out = PathBuf::from(args.required("out")?);
    report_output::claim(&out)?;
    let result = run(&args, &out);
    if let Err(error) = &result {
        fs::write(
            out.join("error.json"),
            serde_json::to_vec_pretty(&json!({"error": error.to_string()}))?,
        )?;
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_is_recovered_from_every_template_shape() {
        assert_eq!(
            key_from_query("Remind me what {k} was.", "Remind me what zorbi was."),
            Some("zorbi".into())
        );
        assert_eq!(
            key_from_query("{k}?", "Zorbi?"),
            Some("Zorbi".into()),
            "a leading slot keeps the capitalized text"
        );
        assert_eq!(key_from_query("What is {k}?", "What was x?"), None);
    }

    #[test]
    fn reply_forms_split_rehearse_bare_and_other() {
        assert_eq!(reply_form("Zorbi is 42.", "zorbi"), "rehearse");
        assert_eq!(reply_form("It's 42.", "zorbi"), "bare");
        assert_eq!(reply_form("That's 42.", "zorbi"), "bare");
        assert_eq!(reply_form("I don't know.", "zorbi"), "other");
    }

    #[test]
    fn rank_and_argmax_break_ties_toward_the_lowest_id() {
        let scores = [0.5, 2.0, 2.0, -1.0];
        assert_eq!(argmax(&scores).0, 1);
        assert_eq!(rank(&scores, 1), 0);
        assert_eq!(rank(&scores, 2), 1);
        assert_eq!(rank(&scores, 0), 2);
        assert_eq!(rank(&scores, 3), 3);
    }

    #[test]
    fn runs_are_found_only_when_contiguous() {
        assert!(contains_run(&[1, 2, 3, 4], &[2, 3]));
        assert!(!contains_run(&[1, 2, 3, 4], &[2, 4]));
        assert!(!contains_run(&[1, 2], &[]));
    }

    /// A byte-level tokenizer with no merges: every byte is one piece, so the
    /// value always starts on a piece boundary.
    fn byte_tokenizer() -> ByteBpeTokenizer {
        let mut vocab = serde_json::Map::new();
        for (id, surface) in ["<|bos|>", "<|eos|>", "<|unk|>"].iter().enumerate() {
            vocab.insert((*surface).to_owned(), json!(id));
        }
        // GPT-2's byte-to-character alphabet.
        let mut extra = 0;
        for byte in 0u32..256 {
            let printable = (u32::from(b'!')..=u32::from(b'~')).contains(&byte)
                || (0xA1..=0xAC).contains(&byte)
                || (0xAE..=0xFF).contains(&byte);
            let ch = if printable {
                char::from_u32(byte)
            } else {
                extra += 1;
                char::from_u32(255 + extra)
            }
            .expect("a valid alphabet character");
            vocab.insert(ch.to_string(), json!(byte + 3));
        }
        ByteBpeTokenizer::from_tokenizer_json_bytes(
            json!({
                "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
                "added_tokens": [{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}],
                "model": {"type": "BPE", "vocab": vocab, "merges": []}
            })
            .to_string()
            .as_bytes(),
        )
        .expect("fixture tokenizer")
    }

    #[test]
    fn the_cut_forces_the_prefix_and_keeps_the_value_pieces_exact() {
        let tokenizer = byte_tokenizer();
        let (piece_cut, reply) = cut(&tokenizer, true, "Zo is", "42").expect("a clean cut");
        assert_eq!(reply, "Zo is 42.");
        assert_eq!(tokenizer.decode(&piece_cut.forced), " Zo is");
        assert_eq!(tokenizer.decode(&piece_cut.value), " 42");
        let (unspaced, _) = cut(&tokenizer, false, "It's", "ab").expect("a clean cut");
        assert_eq!(tokenizer.decode(&unspaced.forced), "It's");
        assert_eq!(tokenizer.decode(&unspaced.value), " ab");
    }
}
