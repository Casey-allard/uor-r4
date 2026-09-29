//! M-world v1 corpora and evaluation (Lab 1; ROADMAP restart packet, Stage 1,
//! R1). The world is `uor_r4_training::milestone_world`.
//!
//! ```text
//! m-world corpus out=NEW_REPORT_ROOT tokenizer=TOKENIZER.json chat=CHAT_V0_TRAIN_DIR \
//!   exclude=REQUESTS.json [conversations=5600] [seed=1]
//! m-world evaluate out=NEW_REPORT_ROOT model=MODEL_DIR tokenizer=TOKENIZER.json \
//!   [split=development|train] [conversations=300] [seed=9101] [max_new_tokens=32] \
//!   [panel=REQUESTS.json]
//! m-world rejudge out=NEW_REPORT_ROOT report=OLD_ROOT/m_world_evaluation.json
//! ```
//!
//! `corpus` writes a prepared dialogue split (`uor-r4-chat-corpus/v1`: a UORT
//! token store, its response mask and its manifest) under `OUT/train/`. The
//! whole chat-v0 training split comes first, unchanged, then `conversations`
//! M-world training conversations as one more source. None of their user
//! turns equals (ignoring case and punctuation) a user turn of the `exclude`
//! panel. `dialogue-train` reads it with `train_tokens=OUT/train/tokens.u16
//! train_mask=OUT/train/response_mask.u8 train_manifest=OUT/train/manifest.json`.
//!
//! `evaluate` answers M-world conversations of `split` with the saved stack's
//! greedy replies, each turn after the model's own earlier replies, as
//! `stack_dialogue::reply_panel` does. It judges every turn with the frozen
//! oracle and reports per category and per intent. A conversation whose
//! history cannot fit the context with every reply at `max_new_tokens` is
//! skipped and counted. With `panel=`, it also answers that request panel and
//! scores its ten memory requests (`stack_memory_replies::score_memory`) into
//! `panel_replies.json`, after the M-world report is written. A
//! saved transport snap is restored; a saved served representation is
//! refused. Every root is claimed before anything is loaded and sealed at the
//! end. Set RAYON_NUM_THREADS to bound the threads.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::Device;
use serde_json::{json, Value};
use uor_r4_core::native_geometric::mmap_corpus::{CorpusWriter, MmapCorpusReader};
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::{DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::geometric_stack::StackModel;
use uor_r4_training::milestone_world::{judge, normalized, Category, MWorld, Split};
use uor_r4_training::stack_dialogue::{
    check_panel, episode_contract, greedy_reply, load_requests, reply_panel, DialogueSplit, Request,
};
use uor_r4_training::stack_memory_replies::score_memory;
use uor_r4_training::stack_tracking::Rng;
use uor_r4_training::{sha256_file, Result, TrainingError};

fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
}

struct Args(BTreeMap<String, String>);

impl Args {
    fn parse(arguments: &[String], allowed: &[&str]) -> Result<Self> {
        let mut pairs = BTreeMap::new();
        for argument in arguments {
            let (key, value) = argument
                .split_once('=')
                .ok_or_else(|| invalid(format!("arguments are key=value, got {argument}")))?;
            if !allowed.contains(&key) {
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
            Some(value) => value
                .parse()
                .map_err(|_| invalid(format!("invalid {key}={value}"))),
        }
    }
}

fn load_tokenizer(path: &Path) -> Result<ByteBpeTokenizer> {
    ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(path)?)
        .ok_or_else(|| invalid("tokenizer JSON is not a supported byte-level BPE"))
}

/// Every user turn of a request panel, normalized.
fn panel_turns(path: &Path) -> Result<BTreeSet<String>> {
    Ok(load_requests(path)?
        .iter()
        .flat_map(|r| r.user_turns.iter().map(|t| normalized(t)))
        .collect())
}

fn corpus(args: &Args, out: &Path) -> Result<()> {
    let started = Instant::now();
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let chat = PathBuf::from(args.required("chat")?);
    let exclude_path = PathBuf::from(args.required("exclude")?);
    let conversations: usize = args.number("conversations", 5_600)?;
    let seed: u64 = args.number("seed", 1)?;
    let tokenizer = load_tokenizer(&tokenizer_path)?;
    let protocol = DialogueProtocol::literal_roles_v1(&tokenizer)
        .map_err(|e| invalid(format!("protocol: {e}")))?;
    let encoder = protocol
        .bind(&tokenizer)
        .map_err(|e| invalid(format!("protocol: {e}")))?;
    let excluded = panel_turns(&exclude_path)?;
    let chat_tokens_path = chat.join("tokens.u16");
    let chat_mask_path = chat.join("response_mask.u8");
    let chat_manifest_path = chat.join("manifest.json");
    let chat_manifest: Value = serde_json::from_slice(&fs::read(&chat_manifest_path)?)?;
    if chat_manifest["schema"] != "uor-r4-chat-corpus/v1"
        || chat_manifest["drops"]["special_token_occurrences"] != 0
        || chat_manifest["tokenizer"]["sha256"] != json!(sha256_file(&tokenizer_path)?)
    {
        return Err(invalid(
            "chat= must be a uor-r4-chat-corpus/v1 split with no special-token text, \
             prepared with this tokenizer",
        ));
    }
    // The chat split's files must be the ones its manifest names.
    for (path, key) in [
        (&chat_tokens_path, "tokens_sha256"),
        (&chat_mask_path, "mask_sha256"),
    ] {
        if chat_manifest[key] != json!(sha256_file(path)?) {
            return Err(invalid(format!(
                "{} does not match the chat manifest's {key}",
                path.display()
            )));
        }
    }
    let reader = MmapCorpusReader::open(&chat_tokens_path)
        .map_err(|e| invalid(format!("chat token store: {e}")))?;
    let vocab = reader.vocab_size();
    let mut mask = fs::read(&chat_mask_path)?;
    if mask.len() != reader.as_slice().len() {
        return Err(invalid("chat tokens and mask differ in length"));
    }
    // M-world conversations, encoded exactly as corpus documents.
    let mut rng = Rng::new(seed);
    let (mut rejected, mut world_tokens, mut world_mask) = (0usize, Vec::new(), Vec::new());
    let (mut responses, mut per_category) = (0usize, BTreeMap::<String, usize>::new());
    let mut sample = Vec::new();
    for index in 0..conversations {
        let conversation =
            MWorld::conversation_excluding(&mut rng, Split::Train, &excluded, &mut rejected)?;
        let messages: Vec<Message<'_>> = conversation
            .turns
            .iter()
            .flat_map(|turn| {
                [
                    Message {
                        role: "user",
                        content: &turn.user,
                    },
                    Message {
                        role: "assistant",
                        content: &turn.reply,
                    },
                ]
            })
            .collect();
        let encoded = encoder.encode_document(&messages);
        if encoded.emitted_turns != messages.len() || encoded.special_token_occurrences != 0 {
            return Err(invalid(format!(
                "M-world conversation {index} did not encode"
            )));
        }
        for &id in &encoded.tokens {
            if id >= vocab {
                return Err(invalid("an M-world token is outside the vocabulary"));
            }
            world_tokens.push(id as u16);
        }
        world_mask.extend(&encoded.response_mask);
        responses += conversation.turns.len();
        for turn in &conversation.turns {
            *per_category
                .entry(format!("{:?}", turn.category))
                .or_default() += 1;
        }
        if index < 24 {
            sample.push(json!(conversation));
        }
    }
    let train = out.join("train");
    fs::create_dir_all(&train)?;
    let tokens_path = train.join("tokens.u16");
    let mut writer = CorpusWriter::create(&tokens_path, vocab)
        .map_err(|e| invalid(format!("token store: {e}")))?;
    writer
        .write_tokens(reader.as_slice())
        .and_then(|()| writer.write_tokens(&world_tokens))
        .map_err(|e| invalid(format!("token store: {e}")))?;
    let total = writer
        .finish()
        .map_err(|e| invalid(format!("token store: {e}")))?;
    mask.extend(&world_mask);
    let mask_path = train.join("response_mask.u8");
    fs::write(&mask_path, &mask)?;
    let world_response_tokens = world_mask.iter().filter(|&&m| m == 1).count();
    let mut files = chat_manifest["files"]
        .as_array()
        .cloned()
        .ok_or_else(|| invalid("the chat manifest has no files"))?;
    files.push(json!({
        "label": "m-world-v1.train",
        "path": "generated: uor_r4_training::milestone_world (Split::Train)",
        "rows_total": conversations,
        "rows_used": conversations,
        "tokens": world_tokens.len(),
        "response_tokens": world_response_tokens,
        "special_token_occurrences": 0,
    }));
    let mut inputs = chat_manifest["inputs"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    inputs.push(json!({"label": "m-world-v1.train", "path": "generated"}));
    let manifest = json!({
        "schema": "uor-r4-chat-corpus/v1",
        "mask_schema": chat_manifest["mask_schema"],
        "split": "train",
        "mask_rule": chat_manifest["mask_rule"],
        "template_rule": chat_manifest["template_rule"],
        "tokenizer": chat_manifest["tokenizer"],
        "max_tokens": chat_manifest["max_tokens"],
        "inputs": inputs,
        "files": files,
        "drops": chat_manifest["drops"],
        "tokens": total,
        "response_tokens": mask.iter().filter(|&&m| m == 1).count(),
        "tokens_bytes": fs::metadata(&tokens_path)?.len(),
        "mask_bytes": mask.len(),
        "tokens_sha256": sha256_file(&tokens_path)?,
        "mask_sha256": sha256_file(&mask_path)?,
        "composition": {
            "chat_v0": {
                "manifest": chat_manifest_path.display().to_string(),
                "manifest_sha256": sha256_file(&chat_manifest_path)?,
                "tokens_sha256": chat_manifest["tokens_sha256"],
                "tokens": reader.as_slice().len(),
            },
            "m_world": {
                "version": "m-world-v1",
                "split": "train",
                "seed": seed,
                "conversations": conversations,
                "responses": responses,
                "turns_per_category": per_category,
                "tokens": world_tokens.len(),
                "response_tokens": world_response_tokens,
                "rejected_draws": rejected,
                "excluded_panel": exclude_path.display().to_string(),
                "excluded_panel_sha256": sha256_file(&exclude_path)?,
                "excluded_turns": excluded.len(),
            },
        },
    });
    let manifest_path = train.join("manifest.json");
    fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;
    // The split must load and index exactly as `dialogue-train` will read it.
    let (_, contract) = episode_contract(&tokenizer, vocab as usize)?;
    let split = DialogueSplit::load(&tokens_path, &mask_path, &manifest_path)?;
    let index = split.index(contract)?;
    let sources: Vec<Value> = index
        .population()
        .sources
        .iter()
        .map(|s| json!({"label": s.label, "documents": s.documents, "eligible_responses": s.eligible_responses}))
        .collect();
    let executable = std::env::current_exe()?;
    fs::write(
        out.join("corpus.json"),
        serde_json::to_vec_pretty(&json!({
            "schema": "uor-r4.m-world-corpus/1",
            "executable_sha256": sha256_file(&executable)?,
            "world_digest": MWorld::digest(),
            "tokenizer_sha256": sha256_file(&tokenizer_path)?,
            "protocol_identity": protocol.identity().map_err(|e| invalid(format!("protocol: {e}")))?,
            "composition": manifest["composition"],
            "eligible_responses": index.episodes().len(),
            "sources": sources,
            "sample": sample,
            "wall_seconds": started.elapsed().as_secs_f64(),
        }))?,
    )?;
    println!(
        "{} tokens: chat-v0 {} + M-world {} ({conversations} conversations, {responses} responses, {rejected} draws rejected)",
        total,
        reader.as_slice().len(),
        world_tokens.len()
    );
    Ok(())
}

fn load_model(directory: &Path, device: &Device) -> Result<(StackModel, Value)> {
    if StackModel::saved_served_representation(directory)?.is_some() {
        return Err(invalid(
            "the model was saved with a served representation, which this tool does not reapply",
        ));
    }
    let mut model = StackModel::load(directory, device)?;
    let snap = StackModel::saved_transport_snap(directory)?;
    model.set_transport_snap(snap)?;
    let mut files = serde_json::Map::new();
    for name in ["config.json", "model.safetensors", "transport.json"] {
        let path = directory.join(name);
        if path.exists() {
            files.insert(name.into(), json!(sha256_file(&path)?));
        }
    }
    Ok((
        model,
        json!({"files_sha256": files, "transport_snap": format!("{snap:?}")}),
    ))
}

fn evaluate(args: &Args, out: &Path) -> Result<()> {
    let started = Instant::now();
    let model_dir = PathBuf::from(args.required("model")?);
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let split = match args.optional("split").as_deref() {
        None | Some("development") => Split::Development,
        Some("train") => Split::Train,
        Some(other) => return Err(invalid(format!("unknown split={other}"))),
    };
    let conversations: usize = args.number("conversations", 300)?;
    let seed: u64 = args.number("seed", 9_101)?;
    let max_new_tokens: usize = args.number("max_new_tokens", 32)?;
    let tokenizer = load_tokenizer(&tokenizer_path)?;
    let protocol = DialogueProtocol::literal_roles_v1(&tokenizer)
        .map_err(|e| invalid(format!("protocol: {e}")))?;
    let encoder = protocol
        .bind(&tokenizer)
        .map_err(|e| invalid(format!("protocol: {e}")))?;
    let device = Device::Cpu;
    let (model, identity) = load_model(&model_dir, &device)?;
    let context = model.config.context;
    let decode = |ids: &[u32]| tokenizer.decode(ids);
    let mut reply =
        |history: &[u32], cap: usize| greedy_reply(&model, history, cap, protocol.eos_id);
    // The world's conversations, and those whose history fits.
    let mut rng = Rng::new(seed);
    let (mut kept, mut requests, mut skipped) = (Vec::new(), Vec::new(), 0usize);
    for index in 0..conversations {
        let conversation = MWorld::conversation(&mut rng, split);
        let request = Request {
            id: format!("mw-{index:04}"),
            category: "m-world".into(),
            user_turns: conversation.turns.iter().map(|t| t.user.clone()).collect(),
        };
        if check_panel(
            &encoder,
            std::slice::from_ref(&request),
            context,
            max_new_tokens,
        )
        .is_err()
        {
            skipped += 1;
            continue;
        }
        kept.push(conversation);
        requests.push(request);
    }
    let answered = reply_panel(
        &encoder,
        &protocol,
        &requests,
        context,
        max_new_tokens,
        &decode,
        &mut reply,
    )?;
    let rows = answered["rows"]
        .as_array()
        .ok_or_else(|| invalid("the reply panel has rows"))?;
    let mut by_category: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut by_intent: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let (mut whole, mut judged) = (0usize, Vec::with_capacity(rows.len()));
    for (conversation, row) in kept.iter().zip(rows) {
        let replies = row["turns"]
            .as_array()
            .ok_or_else(|| invalid("a reply row has turns"))?;
        let mut all = true;
        let mut turns = Vec::with_capacity(replies.len());
        for (turn, answer) in conversation.turns.iter().zip(replies) {
            let text = answer["reply"]
                .as_str()
                .ok_or_else(|| invalid(format!("{}: a turn without a reply", row["id"])))?;
            let pass = judge(&turn.checks, &turn.user, text);
            all &= pass;
            for (key, table) in [
                (format!("{:?}", turn.category), &mut by_category),
                (turn.intent.clone(), &mut by_intent),
            ] {
                let cell = table.entry(key).or_default();
                cell.0 += usize::from(pass);
                cell.1 += 1;
            }
            turns.push(json!({
                "intent": turn.intent, "category": turn.category, "user": turn.user,
                "reply": text, "pass": pass, "checks": turn.checks,
                "stop": answer["stop"],
            }));
        }
        whole += usize::from(all);
        judged.push(json!({"id": row["id"], "all_pass": all, "turns": turns}));
    }
    let rate = |(pass, of): (usize, usize)| json!({"pass": pass, "of": of, "rate": if of == 0 { 0.0 } else { pass as f64 / of as f64 }});
    for category in [
        Category::Responsive,
        Category::Relation,
        Category::Instruction,
    ] {
        let cell = by_category
            .get(&format!("{category:?}"))
            .copied()
            .unwrap_or_default();
        println!(
            "{category:?}: {}/{} ({:.3})",
            cell.0,
            cell.1,
            if cell.1 == 0 {
                0.0
            } else {
                cell.0 as f64 / cell.1 as f64
            }
        );
    }
    let executable = std::env::current_exe()?;
    let executable_sha256 = sha256_file(&executable)?;
    let report = json!({
        "schema": "uor-r4.m-world-evaluation/1",
        "executable_sha256": executable_sha256,
        "world_digest": MWorld::digest(),
        "model": model_dir.display().to_string(),
        "model_identity": identity,
        "tokenizer_sha256": sha256_file(&tokenizer_path)?,
        "protocol_identity": protocol.identity().map_err(|e| invalid(format!("protocol: {e}")))?,
        "world": "m-world-v1",
        "split": split,
        "seed": seed,
        "conversations_drawn": conversations,
        "conversations_skipped_context": skipped,
        "max_new_tokens": max_new_tokens,
        "by_category": by_category.into_iter().map(|(k, v)| (k, rate(v))).collect::<BTreeMap<_, _>>(),
        "by_intent": by_intent.into_iter().map(|(k, v)| (k, rate(v))).collect::<BTreeMap<_, _>>(),
        "conversations_all_pass": rate((whole, kept.len())),
        "conversations": judged,
        "wall_seconds": started.elapsed().as_secs_f64(),
    });
    // Written before the panel, so a panel error cannot discard it.
    fs::write(
        out.join("m_world_evaluation.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    if let Some(path) = args.optional("panel") {
        let path = PathBuf::from(path);
        let requests = load_requests(&path)?;
        let replies = reply_panel(
            &encoder,
            &protocol,
            &requests,
            context,
            max_new_tokens,
            &decode,
            &mut reply,
        )?;
        let memory = score_memory(&replies)?;
        println!("panel memory: {}/{}", memory["correct"], memory["of"]);
        fs::write(
            out.join("panel_replies.json"),
            serde_json::to_vec_pretty(&json!({
                "schema": "uor-r4.m-world-panel/1",
                "executable_sha256": executable_sha256,
                "model_identity": report["model_identity"],
                "requests": path.display().to_string(),
                "requests_sha256": sha256_file(&path)?,
                "max_new_tokens": max_new_tokens,
                "memory": memory,
                "replies": replies,
            }))?,
        )?;
    }
    Ok(())
}

/// Re-judge an evaluation's saved replies with this build's oracle. The
/// conversations are regenerated from the report's seed and split, and every
/// saved user turn must equal its regenerated turn, so only the checks can
/// differ. Nothing is generated.
fn rejudge(args: &Args, out: &Path) -> Result<()> {
    let report_path = PathBuf::from(args.required("report")?);
    let old: Value = serde_json::from_slice(&fs::read(&report_path)?)?;
    if old["schema"] != "uor-r4.m-world-evaluation/1" {
        return Err(invalid("report= is not an M-world evaluation"));
    }
    let split: Split = serde_json::from_value(old["split"].clone())?;
    let seed = old["seed"]
        .as_u64()
        .ok_or_else(|| invalid("the report has no seed"))?;
    let drawn = old["conversations_drawn"]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| invalid("the report has no conversation count"))?;
    let mut rng = Rng::new(seed);
    let conversations: Vec<_> = (0..drawn)
        .map(|_| MWorld::conversation(&mut rng, split))
        .collect();
    let rows = old["conversations"]
        .as_array()
        .ok_or_else(|| invalid("the report has no conversations"))?;
    let mut by_category: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut by_intent: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let (mut whole, mut changed) = (0usize, Vec::new());
    for row in rows {
        let id = row["id"].as_str().unwrap_or_default();
        let index: usize = id
            .strip_prefix("mw-")
            .and_then(|n| n.parse().ok())
            .ok_or_else(|| invalid(format!("unexpected conversation id {id:?}")))?;
        let conversation = conversations
            .get(index)
            .ok_or_else(|| invalid(format!("{id} is beyond the drawn conversations")))?;
        let turns = row["turns"]
            .as_array()
            .ok_or_else(|| invalid(format!("{id} has no turns")))?;
        if turns.len() != conversation.turns.len() {
            return Err(invalid(format!("{id}: turn counts differ")));
        }
        let mut all = true;
        for (number, (saved, turn)) in turns.iter().zip(&conversation.turns).enumerate() {
            if saved["user"].as_str() != Some(turn.user.as_str()) {
                return Err(invalid(format!(
                    "{id} turn {}: the user turn differs",
                    number + 1
                )));
            }
            let reply = saved["reply"].as_str().unwrap_or_default();
            let pass = judge(&turn.checks, &turn.user, reply);
            all &= pass;
            if saved["pass"].as_bool() != Some(pass) {
                changed.push(json!({
                    "id": id, "turn": number + 1, "intent": turn.intent, "user": turn.user,
                    "reply": reply, "before": saved["pass"], "after": pass,
                }));
            }
            for (key, table) in [
                (format!("{:?}", turn.category), &mut by_category),
                (turn.intent.clone(), &mut by_intent),
            ] {
                let cell = table.entry(key).or_default();
                cell.0 += usize::from(pass);
                cell.1 += 1;
            }
        }
        whole += usize::from(all);
    }
    let rate = |(pass, of): (usize, usize)| json!({"pass": pass, "of": of, "rate": if of == 0 { 0.0 } else { pass as f64 / of as f64 }});
    for (category, cell) in &by_category {
        println!("{category}: {}/{}", cell.0, cell.1);
    }
    println!("{} turns changed", changed.len());
    let executable = std::env::current_exe()?;
    let report = json!({
        "schema": "uor-r4.m-world-rejudge/1",
        "world_digest": MWorld::digest(),
        "executable_sha256": sha256_file(&executable)?,
        "source_report": report_path.display().to_string(),
        "source_report_sha256": sha256_file(&report_path)?,
        "split": split,
        "seed": seed,
        "by_category_before": old["by_category"],
        "by_category": by_category.into_iter().map(|(k, v)| (k, rate(v))).collect::<BTreeMap<_, _>>(),
        "by_intent": by_intent.into_iter().map(|(k, v)| (k, rate(v))).collect::<BTreeMap<_, _>>(),
        "conversations_all_pass": rate((whole, rows.len())),
        "changed_turns": changed,
    });
    fs::write(
        out.join("m_world_rejudged.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}

fn main() -> Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let (mode, rest) = arguments
        .split_first()
        .ok_or_else(|| invalid("usage: m-world corpus|evaluate|rejudge key=value..."))?;
    let args = match mode.as_str() {
        "corpus" => Args::parse(
            rest,
            &[
                "out",
                "tokenizer",
                "chat",
                "exclude",
                "conversations",
                "seed",
            ],
        )?,
        "evaluate" => Args::parse(
            rest,
            &[
                "out",
                "model",
                "tokenizer",
                "split",
                "conversations",
                "seed",
                "max_new_tokens",
                "panel",
            ],
        )?,
        "rejudge" => Args::parse(rest, &["out", "report"])?,
        other => return Err(invalid(format!("unknown mode {other}"))),
    };
    let out = PathBuf::from(args.required("out")?);
    report_output::claim(&out)?;
    let result = match mode.as_str() {
        "corpus" => corpus(&args, &out),
        "rejudge" => rejudge(&args, &out),
        _ => evaluate(&args, &out),
    };
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
