//! M-world corpora and evaluation (Lab 1; ROADMAP restart packet, Stage 1, R1
//! and A1). `world=v1` (the default) is `uor_r4_training::milestone_world`, R1's
//! sealed instrument; `world=v2` is `uor_r4_training::milestone_world_v2`, the
//! retrieval instrument (open value pools, MQAR and copy episodes, the #1516
//! oracle fixes).
//!
//! ```text
//! m-world corpus [world=v1] out=NEW_REPORT_ROOT tokenizer=TOKENIZER.json chat=CHAT_V0_TRAIN_DIR \
//!   exclude=REQUESTS.json [conversations=5600] [seed=1]
//! m-world corpus world=v2 out=NEW_REPORT_ROOT tokenizer=TOKENIZER.json [chat=CHAT_V0_TRAIN_DIR] \
//!   [exclude=REQUESTS.json] [conversations=5600] [seed=1] \
//!   [mqar_share=0.35] [copy_share=0.10] [relation_share=0.30] [other_share=0.25] \
//!   [recall=off|oracle|sieve] [recall_at=reply|query] [context=256] [protocol=1|2]
//! m-world evaluate [world=v1] out=NEW_REPORT_ROOT model=MODEL_DIR tokenizer=TOKENIZER.json \
//!   [split=development|train] [conversations=300] [seed=9101] [max_new_tokens=32] \
//!   [panel=REQUESTS.json] [select=none|flock:W:K] [pointer_select=none|flock:W:K|top:K] \
//!   [pointer_route=none|prime:W|prime-ranked:W|ngram:W|ngram-ranked:W]
//! m-world evaluate world=v2 out=NEW_REPORT_ROOT model=MODEL_DIR tokenizer=TOKENIZER.json \
//!   [split=development|train] [conversations=300] [seed=9101] [max_new_tokens=32] \
//!   [panel=REQUESTS.json] [mqar_share=..] [copy_share=..] [relation_share=..] [other_share=..] \
//!   [select=none|flock:W:K] [pointer_select=none|flock:W:K|top:K] \
//!   [pointer_route=none|prime:W|prime-ranked:W|ngram:W|ngram-ranked:W] [recall=off|oracle|sieve|route] \
//!   [recall_at=reply|query] [route_paraphrases=A.jsonl[,B.jsonl...]] [route_acts=fact|any] [route_trunk=MODEL_DIR] \
//!   [protocol=1|2]
//! m-world rejudge out=NEW_REPORT_ROOT report=OLD_ROOT/m_world_evaluation.json [tokenizer=T.json]
//! m-world evaluate-cells [world=v2] out=NEW_REPORT_ROOT model=MODEL_DIR tokenizer=TOKENIZER.json \
//!   [conversations=300] [seed=9101] [max_new_tokens=32] [teacher_forced=true] \
//!   [mqar_share=..] [copy_share=..] [relation_share=..] [other_share=..] \
//!   [select=none|flock:W:K] [pointer_select=none|flock:W:K|top:K] \
//!   [pointer_route=none|prime:W|prime-ranked:W|ngram:W|ngram-ranked:W]
//! m-world baselines world=v2 out=NEW_REPORT_ROOT tokenizer=TOKENIZER.json \
//!   [split=development|train] [conversations=2000] [seed=9101] [cells=true] \
//!   [mqar_share=..] [copy_share=..] [relation_share=..] [other_share=..]
//! m-world route world=v2 out=NEW_REPORT_ROOT tokenizer=TOKENIZER.json \
//!   [split=development|train] [conversations=2000] [seed=9101] [cells=true] \
//!   [mqar_share=..] [copy_share=..] [relation_share=..] [other_share=..]
//! m-world compiler world=v2 out=NEW_REPORT_ROOT model=MODEL_DIR tokenizer=TOKENIZER.json \
//!   [train_conversations=2000] [eval_conversations=1000] [seed=9101] [steps=400] [rate=0.5] [l2=0.0001] \
//!   [lexical=true|false] [paraphrases=A.jsonl[,B.jsonl...]]
//! m-world probe out=NEW_REPORT_ROOT model=MODEL_DIR tokenizer=TOKENIZER.json [max_new_tokens=48] \
//!   [select=none|flock:W:K] [pointer_select=none|flock:W:K|top:K] \
//!   [pointer_route=none|prime:W|prime-ranked:W|ngram:W|ngram-ranked:W]
//! m-world probe-static out=NEW_REPORT_ROOT tokenizer=TOKENIZER.json [context=256]
//! ```
//!
//! `corpus` writes a prepared dialogue split (`uor-r4-chat-corpus/v1`: a UORT
//! token store, its response mask and its manifest) under `OUT/train/`. With
//! `chat=`, the whole chat-v0 training split comes first, unchanged (v2: minus
//! any document whose user turns share an 8-word n-gram with the sealed
//! probe, counted in the manifest's `composition.chat_v0.probe_screen`), then
//! `conversations` M-world training conversations as one more source. None of
//! their user turns equals (ignoring case and punctuation) a user turn of the
//! `exclude` panel. v1 requires `chat=` and `exclude=`; v2 makes both
//! optional (without `chat=` the split is the M-world source alone).
//! `dialogue-train` reads it with `train_tokens=OUT/train/tokens.u16
//! train_mask=OUT/train/response_mask.u8 train_manifest=OUT/train/manifest.json`.
//!
//! `evaluate` (v1) answers M-world conversations of `split` with the saved
//! stack's greedy replies, each turn after the model's own earlier replies, as
//! `stack_dialogue::reply_panel` does. It judges every turn with the frozen
//! oracle and reports per category and per intent. A conversation whose
//! history cannot fit the context with every reply at `max_new_tokens` is
//! skipped and counted. With `panel=`, it also answers that request panel and
//! scores its ten memory requests (`stack_memory_replies::score_memory`) into
//! `panel_replies.json`, after the M-world report is written.
//!
//! `evaluate world=v2` answers each turn after the episode's REFERENCE
//! history (the world's own earlier replies, not the model's), so an MQAR
//! item is asked at exactly its recorded token distance and every episode
//! fits the context. It judges with the v2 oracle and reports accuracy per
//! category and intent, MQAR recall by distance and by N, copy exact match,
//! relation recall on open and closed pools (abstentions apart), and the A1
//! gate: MQAR recall >= 0.9 at every distance AND open-relation recall >= 0.9,
//! computed on this report's split (the gate counts on `split=development`).
//!
//! `protocol=2` (`corpus` and `evaluate`, world=v2) uses literal-role dialogue
//! version 2 (`uor_r4_tokenizer::dialogue::SCHEMA_V2`): the space after a role
//! marker belongs to the message, so a reply's first word is the token it is
//! mid-sentence and can be copied from context. A version 2 corpus has no
//! `chat=` part (chat-v0 is prepared in version 1) and records
//! `dialogue_protocol` in its manifest; evaluate a model in the version it was
//! trained in. Replies are scored without the leading space.
//!
//! `rejudge` re-judges an evaluation's saved replies with this build's oracle;
//! a v2 report needs `tokenizer=` (its MQAR distances depend on it).
//!
//! `select=` and `pointer_select=` (`evaluate`, `evaluate-cells` and `probe`)
//! override the loaded model's selections without training: the reads' flock
//! (sink at position 0, the last W positions and the K best of the rest) and
//! the pointer head's own selection (`top:K` keeps the K best sources alone, so
//! `top:1` is the single-source pointer; `none` keeps every source). The
//! weights are unchanged, so a window x k sweep is one set of weights scored
//! under several selections. Both are parsed and validated with the other
//! arguments, before the report root is claimed (a malformed one claims
//! nothing); `pointer_select=` still needs a model with a pointer head (`none`
//! excepted), which is known only once the model is loaded. The report's
//! `selection_override` records what was given, what the saved model had and
//! what applied; it is null without an override.
//! `pointer_route=prime:W` likewise replaces the pointer head's learned scores
//! by the exact prime route (`geometric_stack::PrimeRoute`) on the loaded
//! weights (`prime-ranked:W`: the same admission, ranked by the learned score
//! plus the route's; `ngram:W` and `ngram-ranked:W`: admission by the longest
//! ordered n-let match); it excludes a pointer selection (`none` clears a saved
//! route).
//!
//! The council's A1 amendments (issue 1511), all world=v2:
//!
//! - `evaluate-cells` draws every retrieval item under each of the four
//!   (phrasing split x value split) cells from the same seed and reports
//!   accuracy per cell, category, MQAR distance and pool, with the mean NLL of
//!   the gold reply and of the gold value beside it (teacher-forced, through the
//!   pointer mixture for a pointer model). The A1 gate is computed on
//!   `dev_phrasing x dev_value` alone, exactly as `evaluate` computes it, and
//!   `train_phrasing x dev_value` is reported as the pure-retrieval cell. It has
//!   its own parser; `evaluate`'s is untouched. It draws 300 conversations per
//!   cell by default, as `evaluate` does.
//! - `baselines` runs the untrained rules R-recency (the latest open value) and
//!   R-nlet (the continuation of the latest earlier occurrence of the query's
//!   last two words) on the same episodes and judges them with the v2 oracle.
//!   `instrument_freeze_ok` is true only if both are below 0.6 on every gated
//!   cell (MQAR per distance and open-relation recall on the development
//!   cell); a leak names its query templates. No model is loaded. Only the
//!   `reference` row (the world's own replies) carries an `a1_gate` verdict; a
//!   rule row carries its scores and whether it is below the freeze limit.
//! - `probe` answers the sealed 40-item English retrieval probe
//!   (`data/a1-english-probe.json`, SHA-256 pinned in
//!   `milestone_world_v2_probe`) greedily and teacher-forced; `probe-static`
//!   reports its token counts, distances, fit and the two rules, without a model.
//!   `corpus world=v2` redraws any conversation that shares an 8-word user-turn
//!   n-gram with the probe, and drops any `chat=` document whose user turns do.
//!
//! The log-sieve design (`docs/integration/log-sieve-retrieval-design-2026-10-01.md`,
//! D18 outcome D), without training:
//!
//! - `route` (E1) scores R-sieve, the untrained identity channel, and the
//!   reference over the episodes `baselines` draws. It is not a freeze rule.
//! - `evaluate world=v2 recall=oracle|sieve` (E2) puts one system turn, "Memory:
//!   {value}." or "Memory: none.", before each MQAR and relation query's reply:
//!   the oracle's value, or R-sieve's. A query whose line leaves no room to
//!   reply is answered empty and counted in `recall.overflow`. `recall=off`
//!   (the default) leaves the report unchanged. `recall_at=query` puts the
//!   line just before the query's own user turn instead, so that the question
//!   stays the last turn.
//!
//! A saved transport snap is restored; a saved served representation is
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
use uor_r4_tokenizer::dialogue::{DialogueEncoder, DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::flock::FlockSelect;
use uor_r4_training::geometric_stack::{
    parse_flock_select, parse_pointer_route, parse_pointer_select, PointerSelect, PrimeRoute,
    StackModel,
};
use uor_r4_training::milestone_world::{judge, normalized, Category, MWorld, Split};
use uor_r4_training::milestone_world_v2::{
    judge_v2, render, Conversation2, Kind, MWorld2, Mix, Pool, Scorecard, Turn2, CONTEXT,
};
use uor_r4_training::stack_dialogue::{
    check_panel, episode_contract, episode_contract_for, greedy_reply, load_requests, reply_panel,
    DialogueSplit, Reply, Request,
};
use uor_r4_training::stack_memory_replies::score_memory;
use uor_r4_training::stack_tracking::Rng;
use uor_r4_training::{sha256_file, Result, TrainingError};

// The council's A1 amendments: cells, rule baselines, the sealed English probe.
use uor_r4_training::milestone_world_v2::{
    freeze_report, is_retrieval, oracle_recall, recall_line, relation_names, reserved_words,
    run_route, run_rules, sieve_value, takes_recall, Category2, Cell, CellScores, Meter, RuleRun,
    Tag, FREEZE_LIMIT, REFERENCE, REVISION,
};
// E3 of the log-sieve design: the relation compiler.
use uor_r4_training::milestone_world_v2_probe::{
    answer_layout, conversation_excluding_probe, history_messages, probe, probe_ngrams,
    probe_sha256, shares_ngram, static_report, teacher_forced, NllCard, ProbeGroup, Rejected,
    EXCLUSION_NGRAM,
};
use uor_r4_training::relation_compiler::{
    collect, label, op_text, paraphrase_examples, score, trunk_features, ActRule, CompilerSettings,
    Example, Lexicon, OpPolicy, RelationMode, RelationRoute, SavedCompiler, Softmax, SparseSoftmax,
    Trunk, ACTS, COMPILE_PROMPT, NONE as RC_NONE,
};
use uor_r4_training::stack_checkpoint::{
    save_checkpoint, sealed_manifest_sha256, CheckpointIdentity, DataIdentity,
};
use uor_r4_training::stack_grounded_session::{
    CompiledAction, ContextPolicy, GroundedSession, RecallDisposition, SessionLimits, SessionScope,
    SourceSpan, TurnCompiler, TurnControls, TurnOutcome,
};
use uor_r4_training::stack_store::StackStore;
use uor_r4_training::temporal_compiler::GroundedCompiler;

fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
}

/// `select=` and `pointer_select=` as given and as parsed: the post-hoc
/// override of a loaded model's selections. Parsed and validated with the
/// arguments (see [`Args::parse`]), before any report root is claimed or model
/// is loaded, and applied to the model by [`Selection::apply`].
#[derive(Clone, Debug, Default)]
struct Selection {
    /// The reads' flock: the text given and its parse (`none` parses to `None`).
    select: Option<(String, Option<FlockSelect>)>,
    /// The pointer head's own selection, likewise.
    pointer_select: Option<(String, Option<PointerSelect>)>,
    /// The pointer head's prime route, likewise.
    pointer_route: Option<(String, Option<PrimeRoute>)>,
}

impl Selection {
    fn parse(pairs: &BTreeMap<String, String>) -> Result<Self> {
        let select = match pairs.get("select") {
            Some(text) => Some((text.clone(), parse_flock_select(text)?)),
            None => None,
        };
        let pointer_select = match pairs.get("pointer_select") {
            Some(text) => Some((text.clone(), parse_pointer_select(text)?)),
            None => None,
        };
        let pointer_route = match pairs.get("pointer_route") {
            Some(text) => Some((text.clone(), parse_pointer_route(text)?)),
            None => None,
        };
        Ok(Self {
            select,
            pointer_select,
            pointer_route,
        })
    }

    /// Apply the override to the loaded model, post hoc: the reads' flock and
    /// the pointer head's own selection replace the saved ones (the weights do
    /// not change). The record gives, for each one given, the text given, the
    /// saved selection and the one that applies; it is `null` when neither was
    /// given.
    fn apply(&self, model: &mut StackModel) -> Result<Value> {
        if self.select.is_none() && self.pointer_select.is_none() && self.pointer_route.is_none() {
            return Ok(Value::Null);
        }
        let mut record = serde_json::Map::new();
        record.insert("weights_unchanged".into(), json!(true));
        if let Some((text, parsed)) = &self.select {
            let saved = model.config.select;
            model.set_select(*parsed)?;
            record.insert(
                "select".into(),
                json!({"given": text, "saved": saved, "effective": model.config.select}),
            );
        }
        if let Some((text, parsed)) = &self.pointer_select {
            let saved = model.config.pointer.and_then(|pointer| pointer.select);
            model.set_pointer_select(*parsed)?;
            record.insert(
                "pointer_select".into(),
                json!({
                    "given": text, "saved": saved,
                    "effective": model.config.pointer.and_then(|pointer| pointer.select),
                }),
            );
        }
        if let Some((text, parsed)) = &self.pointer_route {
            let saved = model.config.pointer.and_then(|pointer| pointer.route);
            model.set_pointer_route(*parsed)?;
            record.insert(
                "pointer_route".into(),
                json!({
                    "given": text, "saved": saved,
                    "effective": model.config.pointer.and_then(|pointer| pointer.route),
                }),
            );
        }
        Ok(Value::Object(record))
    }
}

struct Args {
    pairs: BTreeMap<String, String>,
    /// `select=` and `pointer_select=`, parsed with the arguments; empty when
    /// neither was given (and always empty for a mode that does not allow them).
    selection: Selection,
}

impl Args {
    /// The `key=value` arguments, each key one of `allowed`, and the model
    /// selection they give, parsed and validated here: this runs before a mode
    /// claims its report root, so a malformed `select=` or `pointer_select=`
    /// costs no root.
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
        let selection = Selection::parse(&pairs)?;
        Ok(Self { pairs, selection })
    }

    fn required(&self, key: &str) -> Result<String> {
        self.pairs
            .get(key)
            .cloned()
            .ok_or_else(|| invalid(format!("missing {key}=")))
    }

    fn optional(&self, key: &str) -> Option<String> {
        self.pairs.get(key).cloned()
    }

    fn number<T: std::str::FromStr>(&self, key: &str, default: T) -> Result<T> {
        match self.pairs.get(key) {
            None => Ok(default),
            Some(value) => value
                .parse()
                .map_err(|_| invalid(format!("invalid {key}={value}"))),
        }
    }
}

/// Which world a corpus or evaluation uses.
#[derive(Clone, Copy, PartialEq, Eq)]
enum World {
    V1,
    V2,
}

/// `recall=` of `evaluate world=v2`: the emission of the log-sieve design
/// (`docs/integration/log-sieve-retrieval-design-2026-10-01.md` §2.4, E2).
/// Before each MQAR and relation query's reply, one system turn states a
/// value: none (`off`, the default), the oracle's own (`oracle`), or what the
/// untrained identity channel finds (`sieve`, [`sieve_value`]); "none" when
/// there is none to state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Recall {
    Off,
    Oracle,
    Sieve,
    /// The whole route: the identity channel, then the relation channel
    /// (`uor_r4_training::relation_compiler::RelationRoute`).
    Route,
}

/// `recall_at=` of `evaluate world=v2`: where the recall line goes, just
/// before the reply (`reply`, the default) or just before the query's own
/// user turn (`query`), so that the question stays the last turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecallAt {
    Reply,
    Query,
}

/// `route_acts=` of `evaluate recall=route`: whether the route's lookup needs
/// a statement the table names an assert or update (`fact`, the default), or
/// only one that holds a value (`any`). Returns whether `fact` applies.
fn route_acts_of(args: &Args) -> Result<bool> {
    match args.optional("route_acts").as_deref() {
        None | Some("fact") => Ok(true),
        Some("any") => Ok(false),
        Some(other) => Err(invalid(format!("unknown route_acts={other}: fact or any"))),
    }
}

fn recall_at_of(args: &Args) -> Result<RecallAt> {
    match args.optional("recall_at").as_deref() {
        None | Some("reply") => Ok(RecallAt::Reply),
        Some("query") => Ok(RecallAt::Query),
        Some(other) => Err(invalid(format!(
            "unknown recall_at={other}: reply or query"
        ))),
    }
}

/// `protocol=1|2` (`corpus` and `evaluate` with world=v2): the literal-role
/// dialogue version a corpus is encoded and an evaluation prompts in (default
/// 1). Version 2 puts the space after a role marker into the message, so a
/// reply's first word is the token it is mid-sentence; a model trained on one
/// version is evaluated in it.
fn protocol_version_of(args: &Args) -> Result<u8> {
    match args.optional("protocol").as_deref() {
        None | Some("1") => Ok(1),
        Some("2") => Ok(2),
        Some(other) => Err(invalid(format!("invalid protocol={other} (1 or 2)"))),
    }
}

fn recall_of(args: &Args) -> Result<Recall> {
    match args.optional("recall").as_deref() {
        None | Some("off") => Ok(Recall::Off),
        Some("oracle") => Ok(Recall::Oracle),
        Some("sieve") => Ok(Recall::Sieve),
        Some("route") => Ok(Recall::Route),
        Some(other) => Err(invalid(format!(
            "unknown recall={other}: off, oracle, sieve or route"
        ))),
    }
}

/// The arguments only world=v2 reads.
const V2_ONLY: [&str; 4] = ["mqar_share", "copy_share", "relation_share", "other_share"];

fn world_of(args: &Args) -> Result<World> {
    match args.optional("world").as_deref() {
        None | Some("v1") => {
            for key in V2_ONLY {
                if args.pairs.contains_key(key) {
                    return Err(invalid(format!("{key}= applies to world=v2 only")));
                }
            }
            Ok(World::V1)
        }
        Some("v2") => Ok(World::V2),
        Some(other) => Err(invalid(format!("unknown world={other}; use v1 or v2"))),
    }
}

/// The v2 mix from the shares given (the defaults otherwise).
fn mix_of(args: &Args) -> Result<Mix> {
    let defaults = Mix::default();
    Ok(Mix {
        mqar: args.number("mqar_share", defaults.mqar)?,
        copy: args.number("copy_share", defaults.copy)?,
        relation: args.number("relation_share", defaults.relation)?,
        other: args.number("other_share", defaults.other)?,
        closed: defaults.closed,
    })
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

fn rate_json((pass, of): (usize, usize)) -> Value {
    json!({"pass": pass, "of": of, "rate": if of == 0 { 0.0 } else { pass as f64 / of as f64 }})
}

/// A prepared chat-v0 split, validated against the tokenizer.
struct Chat {
    manifest: Value,
    manifest_path: PathBuf,
    reader: MmapCorpusReader,
    mask: Vec<u8>,
}

fn load_chat(chat: &Path, tokenizer_path: &Path) -> Result<Chat> {
    let chat_tokens_path = chat.join("tokens.u16");
    let chat_mask_path = chat.join("response_mask.u8");
    let manifest_path = chat.join("manifest.json");
    let manifest: Value = serde_json::from_slice(&fs::read(&manifest_path)?)?;
    if manifest["schema"] != "uor-r4-chat-corpus/v1"
        || manifest["drops"]["special_token_occurrences"] != 0
        || manifest["tokenizer"]["sha256"] != json!(sha256_file(tokenizer_path)?)
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
        if manifest[key] != json!(sha256_file(path)?) {
            return Err(invalid(format!(
                "{} does not match the chat manifest's {key}",
                path.display()
            )));
        }
    }
    let reader = MmapCorpusReader::open(&chat_tokens_path)
        .map_err(|e| invalid(format!("chat token store: {e}")))?;
    let mask = fs::read(&chat_mask_path)?;
    if mask.len() != reader.as_slice().len() {
        return Err(invalid("chat tokens and mask differ in length"));
    }
    Ok(Chat {
        manifest,
        manifest_path,
        reader,
        mask,
    })
}

fn corpus(args: &Args, out: &Path) -> Result<()> {
    let started = Instant::now();
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let chat_dir = PathBuf::from(args.required("chat")?);
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
    let Chat {
        manifest: chat_manifest,
        manifest_path: chat_manifest_path,
        reader,
        mut mask,
    } = load_chat(&chat_dir, &tokenizer_path)?;
    let vocab = reader.vocab_size();
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

/// The chat-v0 manifest's rules, for a split with no chat source (verbatim).
const MASK_SCHEMA: &str = "uor-r4-response-mask/u8/v1";
const MASK_RULE: &str = "1 = each token of an assistant turn's response content and its terminating <|eos|>; 0 = <|bos|>, all role markers, turn separators, system/user content, and an unmasked document-terminal <|eos|>.";
const TEMPLATE_RULE: &str = "<|bos|> then turns joined by a single '\\n' separator (placed before every turn after the first); a turn is '<marker><content>' with markers 'System: ', 'User: ', 'Assistant: '; every assistant turn ends with <|eos|>; a document-terminal <|eos|> is appended only when the final emitted turn is not assistant. Content is \\r\\n/\\r-normalised and trimmed; interior whitespace preserved.";

/// The messages of `turns[..=upto]`: every exchange before `upto` complete,
/// then the user turn at `upto` alone.
fn messages_through(turns: &[Turn2], upto: usize) -> Vec<Message<'_>> {
    let mut messages = Vec::with_capacity(2 * upto + 1);
    for (i, turn) in turns.iter().enumerate().take(upto + 1) {
        messages.push(Message {
            role: "user",
            content: &turn.user,
        });
        if i < upto {
            messages.push(Message {
                role: "assistant",
                content: &turn.reply,
            });
        }
    }
    messages
}

/// A whole episode's messages with each turn's recall line (`lines[i]` for
/// turn `i`) as a system turn just before its reply or just before its user
/// turn, as `evaluate world=v2 recall=` places it.
fn recalled_messages<'a>(
    turns: &'a [Turn2],
    lines: &'a [Option<String>],
    at: RecallAt,
) -> Vec<Message<'a>> {
    let mut messages = Vec::with_capacity(3 * turns.len());
    for (turn, line) in turns.iter().zip(lines) {
        let system = line.as_deref().map(|content| Message {
            role: "system",
            content,
        });
        if at == RecallAt::Query {
            messages.extend(system);
        }
        messages.push(Message {
            role: "user",
            content: &turn.user,
        });
        if at == RecallAt::Reply {
            messages.extend(system);
        }
        messages.push(Message {
            role: "assistant",
            content: &turn.reply,
        });
    }
    messages
}

/// The user turns of a decoded literal-role document ("User: ..." lines, with
/// "Assistant: " and "System: " turns between them): a line that begins (after
/// optional spaces) with a role marker opens a turn of that role, and any other
/// line continues the turn before it. The text does not say where a turn ends,
/// so a line inside a user turn that itself begins with `Assistant:` or
/// `System:` ends that turn early (an n-gram after it is not seen), and one
/// inside another turn that begins with `User:` counts as a user turn (the
/// safe side: the document is dropped).
fn user_turns_of(text: &str) -> Vec<String> {
    let mut turns: Vec<(bool, String)> = Vec::new();
    for line in text.split('\n') {
        let head = line.trim_start();
        let opened = if let Some(rest) = head.strip_prefix("User:") {
            Some((true, rest))
        } else if let Some(rest) = head.strip_prefix("Assistant:") {
            Some((false, rest))
        } else {
            head.strip_prefix("System:").map(|rest| (false, rest))
        };
        match opened {
            Some((is_user, rest)) => turns.push((is_user, rest.to_owned())),
            None => {
                if let Some((_, content)) = turns.last_mut() {
                    content.push('\n');
                    content.push_str(line);
                }
            }
        }
    }
    turns
        .into_iter()
        .filter_map(|(is_user, content)| is_user.then(|| content.trim().to_owned()))
        .collect()
}

/// A chat user turn as a probe-exclusion candidate: [`shares_ngram`] reads the
/// user text of a turn and nothing else.
fn user_only(user: String) -> Turn2 {
    Turn2 {
        intent: "chat_user".to_owned(),
        category: Category2::Responsive,
        user,
        reply: String::new(),
        checks: Vec::new(),
        tag: Tag::default(),
    }
}

/// `object[key] -= by` for a count the chat manifest keeps per source; a key
/// the manifest does not have is left out.
fn reduce_count(object: &mut serde_json::Map<String, Value>, key: &str, by: usize) -> Result<()> {
    if let Some(value) = object.get_mut(key) {
        let current = value
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .ok_or_else(|| invalid(format!("chat source field {key} is not a count")))?;
        let reduced = current.checked_sub(by).ok_or_else(|| {
            invalid(format!(
                "chat source field {key} is {current}, below the {by} the probe screen dropped"
            ))
        })?;
        *value = json!(reduced);
    }
    Ok(())
}

/// The chat-v0 split after the probe screen: the documents none of whose user
/// turns shares an 8-word n-gram with the probe, in store order, and what the
/// screen dropped.
struct ScreenedChat {
    /// The retained documents' tokens and response mask.
    tokens: Vec<u16>,
    mask: Vec<u8>,
    /// The chat manifest's `files`, each source's counts reduced by what was
    /// dropped from it (unchanged for a source that lost nothing).
    files: Vec<Value>,
    /// The record for the corpus manifest (`composition.chat_v0.probe_screen`).
    record: Value,
    documents: usize,
    excluded_documents: usize,
    excluded_tokens: usize,
}

/// Screen the chat-v0 split against the sealed probe by the rule the world's
/// conversations are redrawn by ([`shares_ngram`]: any user turn sharing an
/// [`EXCLUSION_NGRAM`]-word n-gram with the probe's user turns); see
/// [`screen_documents`].
fn screen_chat(
    chat: &Chat,
    tokenizer: &ByteBpeTokenizer,
    protocol: &DialogueProtocol,
    grams: &BTreeSet<Vec<String>>,
) -> Result<ScreenedChat> {
    let files = chat.manifest["files"]
        .as_array()
        .cloned()
        .ok_or_else(|| invalid("the chat manifest has no files"))?;
    screen_documents(
        chat.reader.as_slice(),
        &chat.mask,
        files,
        (protocol.bos_id, protocol.eos_id, protocol.unk_id),
        &|ids: &[u32]| tokenizer.decode(ids),
        grams,
    )
}

/// The screen itself, over a token store `tokens` with its response `mask` and
/// the manifest's `files` (each source's count of tokens, in store order); the
/// specials are the protocol's (BOS, EOS, UNK) ids and `decode` turns token ids
/// into text. A document is the tokens from one BOS to the next; it is decoded
/// without its specials, its user turns read off, and it is dropped whole when
/// the rule fires. The counts of a source that lost a document (`tokens`,
/// `response_tokens`, `rows_used`) are reduced by exactly what it lost; with no
/// hit the tokens, mask and sources come back exactly as they went in. A source
/// that would lose every document is refused (the split loader takes no empty
/// source), before anything is written. The rule is n-gram based: a probe user
/// turn of fewer than [`EXCLUSION_NGRAM`] words has no window, so it is not
/// screened.
fn screen_documents(
    tokens: &[u16],
    mask: &[u8],
    mut files: Vec<Value>,
    specials: (u32, u32, u32),
    decode: &dyn Fn(&[u32]) -> String,
    grams: &BTreeSet<Vec<String>>,
) -> Result<ScreenedChat> {
    let (bos, eos, unk) = specials;
    // Each source's span of the token store, in store order.
    let mut spans: Vec<(usize, usize)> = Vec::with_capacity(files.len());
    let mut start = 0usize;
    for file in &files {
        let length = file["tokens"]
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .ok_or_else(|| invalid("a chat source has no token count"))?;
        let end = start
            .checked_add(length)
            .ok_or_else(|| invalid("the chat source lengths overflow"))?;
        spans.push((start, end));
        start = end;
    }
    if start != tokens.len() || mask.len() != tokens.len() {
        return Err(invalid(
            "the chat sources do not cover the chat token store",
        ));
    }
    // Every BOS starts a document.
    let mut starts: Vec<usize> = Vec::new();
    for (at, &id) in tokens.iter().enumerate() {
        if u32::from(id) == bos {
            starts.push(at);
        }
    }
    if !tokens.is_empty() && starts.first() != Some(&0) {
        return Err(invalid("the chat split does not begin with a document"));
    }
    let mut kept_tokens: Vec<u16> = Vec::with_capacity(tokens.len());
    let mut kept_mask: Vec<u8> = Vec::with_capacity(tokens.len());
    // Per source: (documents, documents dropped, tokens dropped, response
    // tokens dropped).
    let mut per_source = vec![(0usize, 0usize, 0usize, 0usize); files.len()];
    for (index, &from) in starts.iter().enumerate() {
        let to = starts.get(index + 1).copied().unwrap_or(tokens.len());
        let source = spans
            .iter()
            .position(|&(begin, end)| begin <= from && from < end)
            .ok_or_else(|| invalid("a chat document lies outside every source"))?;
        if to > spans[source].1 {
            return Err(invalid("a chat document straddles two sources"));
        }
        let ids: Vec<u32> = tokens[from..to]
            .iter()
            .map(|&id| u32::from(id))
            .filter(|&id| id != bos && id != eos && id != unk)
            .collect();
        let users: Vec<Turn2> = user_turns_of(&decode(ids.as_slice()))
            .into_iter()
            .map(user_only)
            .collect();
        let row = &mut per_source[source];
        row.0 += 1;
        if shares_ngram(&users, grams) {
            row.1 += 1;
            row.2 += to - from;
            row.3 += mask[from..to].iter().filter(|&&m| m == 1).count();
        } else {
            kept_tokens.extend_from_slice(&tokens[from..to]);
            kept_mask.extend_from_slice(&mask[from..to]);
        }
    }
    let mut by_source: Vec<Value> = Vec::with_capacity(files.len());
    let (mut documents, mut excluded_documents) = (0usize, 0usize);
    let (mut excluded_tokens, mut excluded_response_tokens) = (0usize, 0usize);
    for (file, &(seen, dropped, dropped_tokens, dropped_response)) in
        files.iter_mut().zip(&per_source)
    {
        documents += seen;
        excluded_documents += dropped;
        excluded_tokens += dropped_tokens;
        excluded_response_tokens += dropped_response;
        by_source.push(json!({
            "label": file["label"],
            "documents": seen,
            "excluded_documents": dropped,
            "excluded_tokens": dropped_tokens,
            "excluded_response_tokens": dropped_response,
        }));
        if dropped == 0 {
            continue;
        }
        if dropped == seen {
            return Err(invalid(format!(
                "the probe screen would drop every document of chat source {}",
                file["label"].as_str().unwrap_or("?")
            )));
        }
        let object = file
            .as_object_mut()
            .ok_or_else(|| invalid("a chat source is not an object"))?;
        reduce_count(object, "tokens", dropped_tokens)?;
        reduce_count(object, "response_tokens", dropped_response)?;
        reduce_count(object, "rows_used", dropped)?;
        object.insert("probe_excluded_documents".into(), json!(dropped));
    }
    let record = json!({
        "rule": "a chat document is dropped when any of its user turns shares an 8-word n-gram with the sealed probe's user turns",
        "ngram_words": EXCLUSION_NGRAM,
        "probe_sha256": probe_sha256(),
        "probe_ngrams": grams.len(),
        "documents": documents,
        "excluded_documents": excluded_documents,
        "excluded_tokens": excluded_tokens,
        "excluded_response_tokens": excluded_response_tokens,
        "tokens_source": tokens.len(),
        "tokens_kept": kept_tokens.len(),
        "by_source": by_source,
    });
    Ok(ScreenedChat {
        tokens: kept_tokens,
        mask: kept_mask,
        files,
        record,
        documents,
        excluded_documents,
        excluded_tokens,
    })
}

/// v2 corpus: M-world v2 training conversations, one document each, measured
/// in this tokenizer's real tokens, optionally after the chat-v0 split.
fn corpus_v2(args: &Args, out: &Path) -> Result<()> {
    let started = Instant::now();
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let conversations: usize = args.number("conversations", 5_600)?;
    let seed: u64 = args.number("seed", 1)?;
    let mix = mix_of(args)?;
    // The log-sieve emission curriculum: each retrieval turn's recall line in
    // the training episode, which the reply then answers from.
    let recall = recall_of(args)?;
    let recall_at = recall_at_of(args)?;
    let context: usize = args.number("context", CONTEXT)?;
    if context < CONTEXT {
        return Err(invalid(format!("context must be at least {CONTEXT}")));
    }
    let mut recall_lines = 0usize;
    let tokenizer = load_tokenizer(&tokenizer_path)?;
    let version = protocol_version_of(args)?;
    if version != 1 && args.optional("chat").is_some() {
        return Err(invalid(
            "chat= documents are prepared in protocol 1; a protocol=2 corpus is M-world alone",
        ));
    }
    let protocol = DialogueProtocol::literal_roles_version(&tokenizer, version)
        .map_err(|e| invalid(format!("protocol: {e}")))?;
    let encoder = protocol
        .bind(&tokenizer)
        .map_err(|e| invalid(format!("protocol: {e}")))?;
    let exclude_path = args.optional("exclude").map(PathBuf::from);
    let excluded = match &exclude_path {
        Some(path) => panel_turns(path)?,
        None => BTreeSet::new(),
    };
    let chat = match args.optional("chat") {
        Some(dir) => Some(load_chat(Path::new(&dir), &tokenizer_path)?),
        None => None,
    };
    let vocab = match &chat {
        Some(chat) => chat.reader.vocab_size(),
        None => u32::try_from(tokenizer.vocab_size())
            .map_err(|_| invalid("the tokenizer's vocabulary does not fit the token store"))?,
    };
    let count = |text: &str| tokenizer.encode(text).len();
    let spaced_meter = uor_r4_training::milestone_world_v2::Meter::spaced(&count);
    let mut world = MWorld2::new(&count, mix)?;
    let mut rng = Rng::new(seed);
    // Training excludes the sealed English probe by an overlap rule: a
    // conversation that shares an 8-word user-turn n-gram with it is drawn
    // again. Shorter user turns escape the rule; it is not complete
    // decontamination.
    let probe_items = probe()?;
    let probe_grams = probe_ngrams(&probe_items);
    // The chat-v0 split is screened by the same rule, whichever source a
    // document comes from.
    let screened = match &chat {
        Some(chat) => Some(screen_chat(chat, &tokenizer, &protocol, &probe_grams)?),
        None => None,
    };
    let mut rejections = Rejected::default();
    let (mut world_tokens, mut world_mask) = (Vec::new(), Vec::new());
    let mut responses = 0usize;
    // (episodes, tokens, response tokens) per kind; (turns, tokens, response
    // tokens) per category; achieved distances per distance bucket.
    let mut per_kind: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new();
    let mut per_category: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new();
    let mut achieved: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    let mut n_matrix: BTreeMap<String, usize> = BTreeMap::new();
    let mut length_histogram: BTreeMap<usize, usize> = BTreeMap::new();
    let (mut longest, mut sample) = (0usize, Vec::new());
    let mut examples: BTreeMap<&str, Value> = BTreeMap::new();
    for index in 0..conversations {
        let conversation = conversation_excluding_probe(
            &mut world,
            &mut rng,
            Split::Train,
            &excluded,
            &probe_grams,
            &mut rejections,
        )?;
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
                "M-world v2 conversation {index} did not encode"
            )));
        }
        // Conversations are drawn under version 1's meter, so both versions
        // encode the same conversations; a version 2 document is checked
        // against version 2's layout.
        let metered = if version == 1 {
            conversation.tokens
        } else {
            spaced_meter.document(&conversation.turns)
        };
        if encoded.tokens.len() != metered {
            return Err(invalid(format!(
                "M-world v2 conversation {index}: the meter counted {} tokens, the protocol {}",
                metered,
                encoded.tokens.len()
            )));
        }
        // The meter checks the plain episode; with recall lines the corpus
        // holds the episode with each retrieval turn's line added.
        let encoded = if recall == Recall::Off {
            encoded
        } else {
            let lines: Vec<Option<String>> = conversation
                .turns
                .iter()
                .enumerate()
                .map(|(i, turn)| match recall {
                    _ if !takes_recall(turn) => None,
                    // Refused for a corpus before the root is claimed.
                    Recall::Off | Recall::Route => None,
                    Recall::Oracle => Some(recall_line(oracle_recall(turn))),
                    Recall::Sieve => Some(recall_line(
                        sieve_value(&conversation.turns[..i], turn).as_deref(),
                    )),
                })
                .collect();
            recall_lines += lines.iter().flatten().count();
            let messages = recalled_messages(&conversation.turns, &lines, recall_at);
            let recalled = encoder.encode_document(&messages);
            if recalled.emitted_turns != messages.len() || recalled.special_token_occurrences != 0 {
                return Err(invalid(format!(
                    "M-world v2 conversation {index} with recall lines did not encode"
                )));
            }
            recalled
        };
        if encoded.tokens.len() > context {
            return Err(invalid(format!(
                "M-world v2 conversation {index} has {} tokens, over the {context}-token context",
                encoded.tokens.len()
            )));
        }
        for &id in &encoded.tokens {
            if id >= vocab {
                return Err(invalid("an M-world token is outside the vocabulary"));
            }
            world_tokens.push(id as u16);
        }
        let response_tokens = encoded.response_mask.iter().filter(|&&m| m == 1).count();
        world_mask.extend(&encoded.response_mask);
        longest = longest.max(encoded.tokens.len());
        *length_histogram
            .entry(encoded.tokens.len() / 32 * 32)
            .or_default() += 1;
        let kind = per_kind
            .entry(format!("{:?}", conversation.kind))
            .or_default();
        *kind = (
            kind.0 + 1,
            kind.1 + encoded.tokens.len(),
            kind.2 + response_tokens,
        );
        responses += conversation.turns.len();
        for (i, turn) in conversation.turns.iter().enumerate() {
            let cell = per_category
                .entry(format!("{:?}", turn.category))
                .or_default();
            *cell = (
                cell.0 + 1,
                cell.1 + world.meter().exchange(i, &turn.user, &turn.reply),
                cell.2 + world.meter().text(&turn.reply) + 1,
            );
            if let Some(mqar) = &turn.tag.mqar {
                achieved
                    .entry(mqar.target_distance)
                    .or_default()
                    .push(mqar.distance);
                *n_matrix
                    .entry(format!(
                        "D{}xN{}->N{}",
                        mqar.target_distance, mqar.n_requested, mqar.n
                    ))
                    .or_default() += 1;
            }
        }
        let open_relation = conversation.kind == Kind::Relation
            && conversation
                .turns
                .last()
                .is_some_and(|t| t.tag.pool == Some(Pool::Open) && !t.tag.abstain);
        let label = match conversation.kind {
            Kind::Mqar => Some("mqar"),
            Kind::Copy => Some("copy"),
            Kind::Relation if open_relation => Some("open_relation"),
            _ => None,
        };
        if let Some(label) = label {
            examples.entry(label).or_insert_with(
                || json!({"tokens": conversation.tokens, "episode": render(&conversation.turns)}),
            );
        }
        if index < 24 {
            sample.push(json!(conversation));
        }
    }
    let rejected = rejections.total();
    let train = out.join("train");
    fs::create_dir_all(&train)?;
    let tokens_path = train.join("tokens.u16");
    let mut writer = CorpusWriter::create(&tokens_path, vocab)
        .map_err(|e| invalid(format!("token store: {e}")))?;
    if let Some(screened) = &screened {
        writer
            .write_tokens(&screened.tokens)
            .map_err(|e| invalid(format!("token store: {e}")))?;
    }
    writer
        .write_tokens(&world_tokens)
        .map_err(|e| invalid(format!("token store: {e}")))?;
    let total = writer
        .finish()
        .map_err(|e| invalid(format!("token store: {e}")))?;
    let mut mask = screened.as_ref().map_or_else(Vec::new, |s| s.mask.clone());
    mask.extend(&world_mask);
    let mask_path = train.join("response_mask.u8");
    fs::write(&mask_path, &mask)?;
    let world_response_tokens = world_mask.iter().filter(|&&m| m == 1).count();
    let world_file = json!({
        "label": "m-world-v2.train",
        "path": "generated: uor_r4_training::milestone_world_v2 (Split::Train)",
        "rows_total": conversations,
        "rows_used": conversations,
        "tokens": world_tokens.len(),
        "response_tokens": world_response_tokens,
        "special_token_occurrences": 0,
    });
    let world_input = json!({"label": "m-world-v2.train", "path": "generated"});
    let (mut files, mut inputs) = (Vec::new(), Vec::new());
    if let (Some(chat), Some(screened)) = (&chat, &screened) {
        // The chat sources, their counts reduced by what the probe screen dropped.
        files.extend(screened.files.iter().cloned());
        inputs.extend(
            chat.manifest["inputs"]
                .as_array()
                .cloned()
                .unwrap_or_default(),
        );
    }
    files.push(world_file);
    inputs.push(world_input);
    let tokenizer_entry = match &chat {
        Some(chat) => chat.manifest["tokenizer"].clone(),
        None => json!({
            "bos_id": protocol.bos_id,
            "eos_id": protocol.eos_id,
            "unk_id": protocol.unk_id,
            "path": tokenizer_path.display().to_string(),
            "sha256": sha256_file(&tokenizer_path)?,
            "tokenizer_cid": protocol.tokenizer_cid,
            "vocab_size": tokenizer.vocab_size(),
        }),
    };
    let pick = |key: &str, fallback: Value| match &chat {
        Some(chat) => chat.manifest[key].clone(),
        None => fallback,
    };
    let distance_report: BTreeMap<String, Value> = achieved
        .iter()
        .map(|(target, seen)| {
            let mean = seen.iter().sum::<usize>() as f64 / seen.len().max(1) as f64;
            (
                target.to_string(),
                json!({
                    "episodes": seen.len(),
                    "achieved_min": seen.iter().min(),
                    "achieved_max": seen.iter().max(),
                    "achieved_mean": mean,
                }),
            )
        })
        .collect();
    let table = |cells: &BTreeMap<String, (usize, usize, usize)>, unit: &str| -> Value {
        json!(cells
            .iter()
            .map(|(k, v)| (
                k.clone(),
                json!({unit: v.0, "tokens": v.1, "response_tokens": v.2})
            ))
            .collect::<BTreeMap<_, _>>())
    };
    let composition = json!({
        "chat_v0": chat.as_ref().zip(screened.as_ref()).map(|(chat, screened)| json!({
            "manifest": chat.manifest_path.display().to_string(),
            "manifest_sha256": sha256_file(&chat.manifest_path).ok(),
            "tokens_sha256": chat.manifest["tokens_sha256"],
            "tokens_source": chat.reader.as_slice().len(),
            "tokens": screened.tokens.len(),
            "probe_screen": screened.record,
        })),
        "m_world": {
            "version": "m-world-v2",
            "world_digest": MWorld2::digest(),
            "split": "train",
            "seed": seed,
            "mix": world.mix(),
            "conversations": conversations,
            "responses": responses,
            "tokens": world_tokens.len(),
            "response_tokens": world_response_tokens,
            "rejected_draws": rejected,
            "excluded_panel": exclude_path.as_ref().map(|p| p.display().to_string()),
            "excluded_turns": excluded.len(),
            "probe_exclusion": {
                "probe_sha256": probe_sha256(),
                "ngram_words": EXCLUSION_NGRAM,
                "probe_ngrams": probe_grams.len(),
                "rejected_by_panel": rejections.panel,
                "rejected_by_probe": rejections.probe,
            },
            "episodes_per_kind": table(&per_kind, "episodes"),
            "turns_per_category": table(&per_category, "turns"),
            "mqar_achieved_distance_by_bucket": distance_report,
            "mqar_n_requested_to_achieved": n_matrix,
            "episode_tokens_max": longest,
            "recall": (recall != Recall::Off).then(|| json!({
                "mode": format!("{recall:?}").to_lowercase(),
                "at": format!("{recall_at:?}").to_lowercase(),
                "lines": recall_lines,
                "context": context,
                "rule": "one system turn per MQAR and relation query, \"Memory: {value}.\" or \
                         \"Memory: none.\"; the plain episode is the one the meter checks",
            })),
            "episode_tokens_histogram_by_32": length_histogram
                .iter()
                .map(|(bucket, n)| (format!("{bucket}-{}", bucket + 31), *n))
                .collect::<BTreeMap<_, _>>(),
        },
    });
    let manifest = json!({
        "schema": "uor-r4-chat-corpus/v1",
        "mask_schema": pick("mask_schema", json!(MASK_SCHEMA)),
        "split": "train",
        "mask_rule": pick("mask_rule", json!(MASK_RULE)),
        "template_rule": pick("template_rule", json!(TEMPLATE_RULE)),
        "tokenizer": tokenizer_entry,
        "max_tokens": pick("max_tokens", json!(CONTEXT)),
        "inputs": inputs,
        "files": files,
        "drops": pick("drops", json!({
            "messages_skipped_empty": 0, "messages_skipped_unknown_role": 0,
            "rows_dropped_empty": 0, "rows_dropped_malformed": 0, "rows_dropped_no_messages": 0,
            "rows_dropped_no_response": 0, "rows_dropped_oversized": 0,
            "special_token_occurrences": 0,
        })),
        "tokens": total,
        "response_tokens": mask.iter().filter(|&&m| m == 1).count(),
        "tokens_bytes": fs::metadata(&tokens_path)?.len(),
        "mask_bytes": mask.len(),
        "tokens_sha256": sha256_file(&tokens_path)?,
        "mask_sha256": sha256_file(&mask_path)?,
        "composition": composition,
    });
    let mut manifest = manifest;
    // Recorded only for version 2, so version 1 manifests are unchanged.
    if version != 1 {
        manifest["dialogue_protocol"] = json!(protocol.schema);
    }
    let manifest_path = train.join("manifest.json");
    fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;
    // The split must load and index exactly as `dialogue-train` will read it.
    let (_, contract) = episode_contract_for(&tokenizer, vocab as usize, CONTEXT, version)?;
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
            "schema": "uor-r4.m-world-corpus/2",
            "executable_sha256": sha256_file(&executable)?,
            "world": "m-world-v2",
            "world_digest": MWorld2::digest(),
            "v1_world_digest": MWorld::digest(),
            "mix": world.mix(),
            "tokenizer_sha256": sha256_file(&tokenizer_path)?,
            "protocol_identity": protocol.identity().map_err(|e| invalid(format!("protocol: {e}")))?,
            "composition": manifest["composition"],
            "eligible_responses": index.episodes().len(),
            "sources": sources,
            "examples": examples,
            "sample": sample,
            "wall_seconds": started.elapsed().as_secs_f64(),
        }))?,
    )?;
    println!(
        "{total} tokens: M-world v2 {} ({conversations} conversations, {responses} responses, \
         {rejected} draws rejected), longest episode {longest} tokens",
        world_tokens.len()
    );
    if let Some(screened) = &screened {
        println!(
            "chat-v0 probe screen: {} of {} documents dropped ({} tokens)",
            screened.excluded_documents, screened.documents, screened.excluded_tokens
        );
    }
    println!("episodes per kind (episodes, tokens, response tokens): {per_kind:?}");
    println!("turns per category (turns, tokens, response tokens): {per_category:?}");
    println!(
        "MQAR achieved distance by bucket: {}",
        serde_json::to_string(&composition["m_world"]["mqar_achieved_distance_by_bucket"])?
    );
    for (label, example) in &examples {
        println!(
            "--- example: {label} ({} tokens)\n{}",
            example["tokens"],
            example["episode"].as_str().unwrap_or_default()
        );
    }
    Ok(())
}

/// The model of `directory`, its file identity, and the record of the
/// selection override `selection` gives it ([`Selection::apply`]). The
/// selection was parsed with the arguments, before the report root was
/// claimed; only its application to this model happens here.
fn load_model(
    directory: &Path,
    device: &Device,
    selection: &Selection,
) -> Result<(StackModel, Value, Value)> {
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
    // The identity is of the saved files; the override changes no weight.
    let selection_override = selection.apply(&mut model)?;
    Ok((
        model,
        json!({"files_sha256": files, "transport_snap": format!("{snap:?}")}),
        selection_override,
    ))
}

fn split_of(args: &Args) -> Result<Split> {
    match args.optional("split").as_deref() {
        None | Some("development") => Ok(Split::Development),
        Some("train") => Ok(Split::Train),
        Some(other) => Err(invalid(format!("unknown split={other}"))),
    }
}

/// Answer the request panel of `panel=` and score its ten memory requests
/// into `panel_replies.json`, after the M-world report is written.
#[allow(clippy::too_many_arguments)]
fn answer_panel(
    path: &Path,
    out: &Path,
    encoder: &DialogueEncoder<'_>,
    protocol: &DialogueProtocol,
    context: usize,
    max_new_tokens: usize,
    decode: &dyn Fn(&[u32]) -> String,
    reply: &mut dyn FnMut(&[u32], usize) -> Result<Reply>,
    model_identity: &Value,
    executable_sha256: &str,
) -> Result<()> {
    let requests = load_requests(path)?;
    let replies = reply_panel(
        encoder,
        protocol,
        &requests,
        context,
        max_new_tokens,
        decode,
        reply,
    )?;
    let memory = score_memory(&replies)?;
    println!("panel memory: {}/{}", memory["correct"], memory["of"]);
    fs::write(
        out.join("panel_replies.json"),
        serde_json::to_vec_pretty(&json!({
            "schema": "uor-r4.m-world-panel/1",
            "executable_sha256": executable_sha256,
            "model_identity": model_identity,
            "requests": path.display().to_string(),
            "requests_sha256": sha256_file(path)?,
            "max_new_tokens": max_new_tokens,
            "memory": memory,
            "replies": replies,
        }))?,
    )?;
    Ok(())
}

fn evaluate(args: &Args, out: &Path) -> Result<()> {
    let started = Instant::now();
    let model_dir = PathBuf::from(args.required("model")?);
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let split = split_of(args)?;
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
    let (model, identity, selection_override) = load_model(&model_dir, &device, &args.selection)?;
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
        "selection_override": selection_override,
        "tokenizer_sha256": sha256_file(&tokenizer_path)?,
        "protocol_identity": protocol.identity().map_err(|e| invalid(format!("protocol: {e}")))?,
        "dialogue_protocol": protocol.schema,
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
        answer_panel(
            Path::new(&path),
            out,
            &encoder,
            &protocol,
            context,
            max_new_tokens,
            &decode,
            &mut reply,
            &report["model_identity"],
            &executable_sha256,
        )?;
    }
    Ok(())
}

/// v2 evaluation: each turn is answered after the episode's reference
/// history, so MQAR items are asked at exactly their recorded distance.
fn evaluate_v2(args: &Args, out: &Path) -> Result<()> {
    let started = Instant::now();
    let model_dir = PathBuf::from(args.required("model")?);
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let split = split_of(args)?;
    let conversations: usize = args.number("conversations", 300)?;
    let seed: u64 = args.number("seed", 9_101)?;
    let max_new_tokens: usize = args.number("max_new_tokens", 32)?;
    let mix = mix_of(args)?;
    let tokenizer = load_tokenizer(&tokenizer_path)?;
    let protocol = DialogueProtocol::literal_roles_version(&tokenizer, protocol_version_of(args)?)
        .map_err(|e| invalid(format!("protocol: {e}")))?;
    let encoder = protocol
        .bind(&tokenizer)
        .map_err(|e| invalid(format!("protocol: {e}")))?;
    let device = Device::Cpu;
    let (model, identity, selection_override) = load_model(&model_dir, &device, &args.selection)?;
    let context = model.config.context;
    let recall = recall_of(args)?;
    let recall_at = recall_at_of(args)?;
    let route_fact_acts = route_acts_of(args)?;
    let decode = |ids: &[u32]| tokenizer.decode(ids);
    let mut reply =
        |history: &[u32], cap: usize| greedy_reply(&model, history, cap, protocol.eos_id);
    let count = |text: &str| tokenizer.encode(text).len();
    // recall=route fits the relation channel's word table first, from the
    // training-phrasing draws `compiler` uses (and any teacher paraphrases),
    // with `route_trunk=`'s features of each turn beside its words.
    let route_trunk = match args.optional("route_trunk") {
        Some(directory) => {
            let (trunk, identity, _) =
                load_model(Path::new(&directory), &device, &Selection::default())?;
            Some((
                trunk,
                json!({"model": directory, "model_identity": identity}),
            ))
        }
        None => None,
    };
    let trunk_cache: std::cell::RefCell<BTreeMap<String, Vec<f64>>> = Default::default();
    let trunk_of = |text: &str| -> Result<Option<Vec<f64>>> {
        let Some((trunk, _)) = &route_trunk else {
            return Ok(None);
        };
        if let Some(features) = trunk_cache.borrow().get(text) {
            return Ok(Some(features.clone()));
        }
        let features = trunk_features(trunk, &encoder, protocol.bos_id, text)?;
        trunk_cache
            .borrow_mut()
            .insert(text.to_owned(), features.clone());
        Ok(Some(features))
    };
    let (route, route_record) = if recall == Recall::Route {
        let features =
            |text: &str| -> Result<Vec<f64>> { trunk_of(text)?.ok_or_else(|| invalid("no trunk")) };
        let trunk = route_trunk.as_ref().map(|(_, identity)| {
            (
                &features as &dyn Fn(&str) -> Result<Vec<f64>>,
                identity.clone(),
            )
        });
        let (route, record) = fit_route(&count, args.optional("route_paraphrases"), trunk)?;
        (Some(route), Some(record))
    } else {
        (None, None)
    };
    let reserved = reserved_words();
    // Relation queries the route's table named correctly, of those evaluated.
    let mut route_named = (0usize, 0usize);
    let mut world = MWorld2::new(&count, mix)?;
    let mut rng = Rng::new(seed);
    let mut card = Scorecard::default();
    let (mut whole, mut judged) = (0usize, Vec::with_capacity(conversations));
    // Queries whose recall line left no room to reply (failed, not skipped).
    let mut recall_overflow = 0usize;
    for index in 0..conversations {
        let conversation = world.conversation(&mut rng, split)?;
        let mut all = true;
        let mut turns = Vec::with_capacity(conversation.turns.len());
        for (t, turn) in conversation.turns.iter().enumerate() {
            let history = &conversation.turns[..t];
            let line = match (recall, &route) {
                (Recall::Off, _) => None,
                _ if !takes_recall(turn) => None,
                (Recall::Oracle, _) => Some(recall_line(oracle_recall(turn))),
                (Recall::Sieve, _) => Some(recall_line(sieve_value(history, turn).as_deref())),
                (Recall::Route, Some(route)) => {
                    if turn.category == Category2::Relation {
                        let dense = trunk_of(&turn.user)?;
                        route_named.1 += 1;
                        route_named.0 += usize::from(
                            route.classify(&turn.user, dense.as_deref())?.0 == label(turn).0,
                        );
                    }
                    let value = match sieve_value(history, turn) {
                        Some(value) => Some(value),
                        None => {
                            let mut trunk = |text: &str| trunk_of(text);
                            route.value(history, turn, reserved, route_fact_acts, &mut trunk)?
                        }
                    };
                    Some(recall_line(value.as_deref()))
                }
                (Recall::Route, None) => return Err(invalid("recall=route without a route")),
            };
            let mut messages = messages_through(&conversation.turns, t);
            if let Some(line) = &line {
                let system = Message {
                    role: "system",
                    content: line,
                };
                match recall_at {
                    RecallAt::Reply => messages.push(system),
                    // Before the query's own user turn, the last message.
                    RecallAt::Query => messages.insert(messages.len() - 1, system),
                }
            }
            let prefix = encoder.encode_assistant_prefix(&messages);
            if prefix.emitted_turns != messages.len() || prefix.special_token_occurrences != 0 {
                return Err(invalid(format!("mw2-{index:04}: turn {t} did not encode")));
            }
            let overflow = prefix.tokens.len() >= context;
            let (text, stop) = if overflow {
                // Only a recall line can push an episode past the context.
                recall_overflow += 1;
                (String::new(), json!("recall_overflow"))
            } else {
                let cap = max_new_tokens.min(context - prefix.tokens.len());
                let generated = greedy_reply(&model, &prefix.tokens, cap, protocol.eos_id)?;
                let text_ids: Vec<u32> = generated
                    .ids
                    .iter()
                    .copied()
                    .filter(|&id| id != protocol.eos_id)
                    .collect();
                (
                    protocol.reply_text(&decode(&text_ids)).to_owned(),
                    generated.stop_record(),
                )
            };
            let pass = judge_v2(&turn.checks, &turn.user, &text);
            all &= pass;
            card.record(turn, pass);
            let mut row = json!({
                "intent": turn.intent, "category": turn.category, "user": turn.user,
                "reference_reply": turn.reply, "reply": text, "pass": pass,
                "checks": turn.checks, "tag": turn.tag, "stop": stop,
                "history_tokens": prefix.tokens.len(),
            });
            if let Some(line) = &line {
                row["recall"] = json!(line);
            }
            turns.push(row);
        }
        whole += usize::from(all);
        judged.push(json!({
            "id": format!("mw2-{index:04}"), "kind": conversation.kind,
            "tokens": conversation.tokens, "all_pass": all, "turns": turns,
        }));
    }
    let scores = card.to_json();
    for (category, cell) in scores["by_category"].as_object().into_iter().flatten() {
        println!(
            "{category}: {}/{} ({:.3})",
            cell["pass"],
            cell["of"],
            cell["rate"].as_f64().unwrap_or(0.0)
        );
    }
    for (distance, cell) in scores["mqar"]["by_distance"]
        .as_object()
        .into_iter()
        .flatten()
    {
        println!(
            "MQAR D={distance}: {}/{} ({:.3})",
            cell["pass"],
            cell["of"],
            cell["rate"].as_f64().unwrap_or(0.0)
        );
    }
    for (n, cell) in scores["mqar"]["by_n"].as_object().into_iter().flatten() {
        println!(
            "MQAR N={n}: {}/{} ({:.3})",
            cell["pass"],
            cell["of"],
            cell["rate"].as_f64().unwrap_or(0.0)
        );
    }
    println!(
        "copy: {}/{}; open relation: {}/{}; closed relation: {}/{}; a1_gate: {}",
        scores["copy"]["pass"],
        scores["copy"]["of"],
        scores["relation"]["open"]["pass"],
        scores["relation"]["open"]["of"],
        scores["relation"]["closed"]["pass"],
        scores["relation"]["closed"]["of"],
        scores["a1_gate"]
    );
    let executable = std::env::current_exe()?;
    let executable_sha256 = sha256_file(&executable)?;
    let mut report = json!({
        "schema": "uor-r4.m-world-evaluation/2",
        "executable_sha256": executable_sha256,
        "world": "m-world-v2",
        "world_digest": MWorld2::digest(),
        "model": model_dir.display().to_string(),
        "model_identity": identity,
        "selection_override": selection_override,
        "tokenizer_sha256": sha256_file(&tokenizer_path)?,
        "protocol_identity": protocol.identity().map_err(|e| invalid(format!("protocol: {e}")))?,
        "split": split,
        "seed": seed,
        "mix": world.mix(),
        "conversations_drawn": conversations,
        "history": "reference: every turn is answered after the episode's own earlier replies",
        "context": context,
        "max_new_tokens": max_new_tokens,
        "a1_gate_scope": if split == Split::Development {
            "development: this is the A1 gate"
        } else {
            "train: informational; the A1 gate is decided on split=development"
        },
        "conversations_all_pass": rate_json((whole, conversations)),
        "conversations": judged,
        "wall_seconds": started.elapsed().as_secs_f64(),
    });
    if let (Some(report), Some(scores)) = (report.as_object_mut(), scores.as_object()) {
        report.extend(scores.clone());
    }
    // Only when given, so other evaluations' reports are unchanged.
    if recall != Recall::Off {
        println!("recall={recall:?}: {recall_overflow} queries had no room to reply");
        report["recall"] = json!({
            "mode": format!("{recall:?}").to_lowercase(),
            "at": format!("{recall_at:?}").to_lowercase(),
            "line": "one system turn per MQAR and relation query, \"Memory: {value}.\" or \
                     \"Memory: none.\", just before the reply (at=reply) or just before the \
                     query's own user turn (at=query)",
            "overflow": recall_overflow,
            "overflow_rule": "a query whose recall line leaves no room to reply is \
                              answered empty and fails",
        });
        if let Some(record) = route_record {
            report["recall"]["route"] = record;
            report["recall"]["route"]["lookup_acts"] =
                json!(if route_fact_acts { "fact" } else { "any" });
            report["recall"]["route"]["relation_queries_named"] =
                rate_json((route_named.0, route_named.1));
            println!(
                "route: the word table named the relation of {}/{} relation queries",
                route_named.0, route_named.1
            );
        }
    }
    // Written before the panel, so a panel error cannot discard it.
    fs::write(
        out.join("m_world_evaluation.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    if let Some(path) = args.optional("panel") {
        answer_panel(
            Path::new(&path),
            out,
            &encoder,
            &protocol,
            context,
            max_new_tokens,
            &decode,
            &mut reply,
            &report["model_identity"],
            &executable_sha256,
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
    if old["schema"] == "uor-r4.m-world-evaluation/2" {
        return rejudge_v2(args, out, &report_path, &old);
    }
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

/// The v2 counterpart: regenerate the report's conversations with the same
/// tokenizer and mix, require every saved user turn to match, and judge the
/// saved replies with this build's v2 oracle.
fn rejudge_v2(args: &Args, out: &Path, report_path: &Path, old: &Value) -> Result<()> {
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    if old["tokenizer_sha256"] != json!(sha256_file(&tokenizer_path)?) {
        return Err(invalid(
            "tokenizer= is not the tokenizer the report was evaluated with",
        ));
    }
    let split: Split = serde_json::from_value(old["split"].clone())?;
    let seed = old["seed"]
        .as_u64()
        .ok_or_else(|| invalid("the report has no seed"))?;
    let drawn = old["conversations_drawn"]
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| invalid("the report has no conversation count"))?;
    let mix: Mix = serde_json::from_value(old["mix"].clone())?;
    let tokenizer = load_tokenizer(&tokenizer_path)?;
    let count = |text: &str| tokenizer.encode(text).len();
    let mut world = MWorld2::new(&count, mix)?;
    let mut rng = Rng::new(seed);
    let conversations: Vec<Conversation2> = (0..drawn)
        .map(|_| world.conversation(&mut rng, split))
        .collect::<Result<_>>()?;
    let rows = old["conversations"]
        .as_array()
        .ok_or_else(|| invalid("the report has no conversations"))?;
    let mut card = Scorecard::default();
    let (mut whole, mut changed) = (0usize, Vec::new());
    for row in rows {
        let id = row["id"].as_str().unwrap_or_default();
        let index: usize = id
            .strip_prefix("mw2-")
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
            let pass = judge_v2(&turn.checks, &turn.user, reply);
            all &= pass;
            card.record(turn, pass);
            if saved["pass"].as_bool() != Some(pass) {
                changed.push(json!({
                    "id": id, "turn": number + 1, "intent": turn.intent, "user": turn.user,
                    "reply": reply, "before": saved["pass"], "after": pass,
                }));
            }
        }
        whole += usize::from(all);
    }
    let scores = card.to_json();
    println!(
        "{} turns changed; a1_gate {} (was {})",
        changed.len(),
        scores["a1_gate"],
        old["a1_gate"]
    );
    let executable = std::env::current_exe()?;
    let mut report = json!({
        "schema": "uor-r4.m-world-rejudge/2",
        "world_digest": MWorld2::digest(),
        "source_world_digest": old["world_digest"],
        "executable_sha256": sha256_file(&executable)?,
        "source_report": report_path.display().to_string(),
        "source_report_sha256": sha256_file(report_path)?,
        "split": split,
        "seed": seed,
        "mix": mix,
        "a1_gate_before": old["a1_gate"],
        "conversations_all_pass": rate_json((whole, rows.len())),
        "changed_turns": changed,
    });
    if let (Some(report), Some(scores)) = (report.as_object_mut(), scores.as_object()) {
        report.extend(scores.clone());
    }
    fs::write(
        out.join("m_world_rejudged.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}

// ---------------------------------------------------------------------------
// The council's A1 amendments: cross cells, untrained rule baselines, the sealed
// English probe and teacher-forced answer-span scores. Each mode has its own
// parser and claims its report root like the modes above.

/// The stack under `model=`, as these modes load it (one place to follow
/// `load_model`), with the `select=` and `pointer_select=` the arguments gave
/// (`selection`, parsed before the root was claimed) applied post hoc and
/// recorded exactly as `evaluate` records them.
fn open_model(
    directory: &Path,
    device: &Device,
    selection: &Selection,
) -> Result<(StackModel, Value, Value)> {
    load_model(directory, device, selection)
}

/// These modes are world=v2 only; `world=v2` may be spelled out.
fn require_v2(args: &Args) -> Result<()> {
    match args.optional("world").as_deref() {
        None | Some("v2") => Ok(()),
        Some(other) => Err(invalid(format!(
            "this mode is world=v2 only, not world={other}"
        ))),
    }
}

/// A greedy reply to one turn, after the episode's reference history.
struct Answer {
    text: String,
    stop: Value,
    history_tokens: usize,
}

#[allow(clippy::too_many_arguments)]
fn answer_turn(
    model: &StackModel,
    encoder: &DialogueEncoder<'_>,
    protocol: &DialogueProtocol,
    decode: &dyn Fn(&[u32]) -> String,
    turns: &[Turn2],
    index: usize,
    max_new_tokens: usize,
    context: usize,
) -> Result<Answer> {
    let messages = history_messages(turns, index);
    let prefix = encoder.encode_assistant_prefix(&messages);
    if prefix.emitted_turns != messages.len() || prefix.special_token_occurrences != 0 {
        return Err(invalid(format!("turn {index} did not encode")));
    }
    let cap = max_new_tokens.min(context.saturating_sub(prefix.tokens.len()));
    let generated = greedy_reply(model, &prefix.tokens, cap, protocol.eos_id)?;
    let ids: Vec<u32> = generated
        .ids
        .iter()
        .copied()
        .filter(|&id| id != protocol.eos_id)
        .collect();
    Ok(Answer {
        text: decode(&ids),
        stop: generated.stop_record(),
        history_tokens: prefix.tokens.len(),
    })
}

/// `evaluate-cells`: every retrieval turn (MQAR queries, relation queries and
/// abstentions, copy) of the four (phrasing x value) cells, each drawn from
/// the same seed, answered after its reference history, judged by the v2 oracle
/// and, unless `teacher_forced=false`, teacher-forced. The A1 gate is decided on
/// `dev_phrasing x dev_value`, the development split, as before.
fn evaluate_cells(args: &Args, out: &Path) -> Result<()> {
    let started = Instant::now();
    require_v2(args)?;
    let model_dir = PathBuf::from(args.required("model")?);
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let conversations: usize = args.number("conversations", 300)?;
    let seed: u64 = args.number("seed", 9_101)?;
    let max_new_tokens: usize = args.number("max_new_tokens", 32)?;
    let forced: bool = args.number("teacher_forced", true)?;
    let mix = mix_of(args)?;
    let tokenizer = load_tokenizer(&tokenizer_path)?;
    let protocol = DialogueProtocol::literal_roles_v1(&tokenizer)
        .map_err(|e| invalid(format!("protocol: {e}")))?;
    let encoder = protocol
        .bind(&tokenizer)
        .map_err(|e| invalid(format!("protocol: {e}")))?;
    let device = Device::Cpu;
    let (model, identity, selection_override) = open_model(&model_dir, &device, &args.selection)?;
    let context = model.config.context;
    let decode = |ids: &[u32]| tokenizer.decode(ids);
    let count = |text: &str| tokenizer.encode(text).len();
    let mut scores = CellScores::default();
    let mut nll: BTreeMap<Cell, NllCard> = BTreeMap::new();
    let mut items: Vec<Value> = Vec::new();
    let mut unscored = 0usize;
    for cell in Cell::ALL {
        // A fresh world and stream per cell: each cell cycles the same MQAR
        // (distance, N) sequence from the same seed.
        let mut world = MWorld2::new(&count, mix)?;
        let mut rng = Rng::new(seed);
        for index in 0..conversations {
            let conversation = world.conversation_in(&mut rng, cell)?;
            for (t, turn) in conversation.turns.iter().enumerate() {
                if !is_retrieval(turn) {
                    continue;
                }
                let answer = answer_turn(
                    &model,
                    &encoder,
                    &protocol,
                    &decode,
                    &conversation.turns,
                    t,
                    max_new_tokens,
                    context,
                )?;
                let pass = judge_v2(&turn.checks, &turn.user, &answer.text);
                scores.record(cell, turn, pass);
                let mut record = json!({
                    "cell": cell.key(),
                    "id": format!("mw2-{index:04}"),
                    "turn": t,
                    "kind": conversation.kind,
                    "intent": turn.intent,
                    "category": turn.category,
                    "user": turn.user,
                    "reference_reply": turn.reply,
                    "reply": answer.text,
                    "pass": pass,
                    "checks": turn.checks,
                    "tag": turn.tag,
                    "stop": answer.stop,
                    "history_tokens": answer.history_tokens,
                });
                if forced {
                    let layout = answer_layout(&encoder, &tokenizer, &conversation.turns, t)?;
                    match teacher_forced(&model, &layout)? {
                        Some(scored) => {
                            nll.entry(cell).or_default().record(turn, &scored);
                            record["teacher_forced"] = json!({
                                "reply_tokens": scored.reply_tokens,
                                "reply_nll": scored.reply_nll,
                                "answer_tokens": scored.answer_tokens,
                                "answer_nll": scored.answer_nll,
                            });
                        }
                        None => unscored += 1,
                    }
                }
                items.push(record);
            }
        }
    }
    // The teacher-forced scores sit beside each cell's accuracy.
    let mut scores_json = scores.to_json();
    for (cell, card) in &nll {
        scores_json["cells"][cell.key()]["teacher_forced"] = card.to_json();
    }
    scores_json["pure_retrieval"]["teacher_forced"] =
        json!(nll.get(&Cell::PURE_RETRIEVAL).map(NllCard::to_json));
    // The pure-retrieval cell first, then the development cell the gate reads.
    let order = [
        Cell::PURE_RETRIEVAL,
        Cell::GATED,
        Cell::new(Split::Train, Split::Train),
        Cell::new(Split::Development, Split::Train),
    ];
    for cell in order {
        let row = &scores_json["matrix"][cell.key()];
        let rate = |key: &str| row[key]["rate"].as_f64().unwrap_or(0.0);
        let note = if cell == Cell::PURE_RETRIEVAL {
            "  <- pure retrieval"
        } else if cell == Cell::GATED {
            "  <- the A1 gate is decided here"
        } else {
            ""
        };
        println!(
            "{}: MQAR D16 {:.3} D64 {:.3} D200 {:.3}; open relation {:.3}, closed {:.3}; copy {:.3}{note}",
            cell.key(),
            rate("mqar/distance/16"),
            rate("mqar/distance/64"),
            rate("mqar/distance/200"),
            rate("relation/open/recall"),
            rate("relation/closed/recall"),
            rate("copy"),
        );
    }
    println!(
        "a1_gate (decided on {}): {}",
        Cell::GATED.key(),
        scores_json["a1_gate"]
    );
    let executable = std::env::current_exe()?;
    let mut report = json!({
        "schema": "uor-r4.m-world-cells/1",
        "executable_sha256": sha256_file(&executable)?,
        "world": "m-world-v2",
        "world_digest": MWorld2::digest(),
        "revision": REVISION,
        "model": model_dir.display().to_string(),
        "model_identity": identity,
        "selection_override": selection_override,
        "tokenizer_sha256": sha256_file(&tokenizer_path)?,
        "protocol_identity": protocol.identity().map_err(|e| invalid(format!("protocol: {e}")))?,
        "seed": seed,
        "mix": mix,
        "conversations_per_cell": conversations,
        "history": "reference: every retrieval turn is answered after the episode's own earlier replies",
        "scope": "retrieval turns only (MQAR queries, relation queries and abstentions, copy); the A1 gate is computed on the development cell alone, and train_phrasing x dev_value is the pure-retrieval cell",
        "context": context,
        "max_new_tokens": max_new_tokens,
        "teacher_forced": forced,
        "teacher_forced_unscored": unscored,
        "items": items,
        "wall_seconds": started.elapsed().as_secs_f64(),
    });
    if let (Some(report), Some(scores)) = (report.as_object_mut(), scores_json.as_object()) {
        report.extend(scores.clone());
    }
    fs::write(
        out.join("m_world_cells.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}

/// One row of the baselines report for one cell. The [`REFERENCE`] row (the
/// world's own replies through the oracle) stands in for a model, so it keeps
/// its scorecard's `a1_gate`. An untrained rule is not a model and takes no
/// gate: its row is its scores, the freeze limit and whether it is below that
/// limit on every gated key of this cell (the freeze itself is decided on the
/// development cell alone, in `freeze` and `instrument_freeze_ok`).
fn baseline_row(name: &str, run: &RuleRun) -> Value {
    let mut row = run.card.to_json();
    if name != REFERENCE {
        if let Some(object) = row.as_object_mut() {
            object.remove("a1_gate");
            object.remove("a1_gate_rule");
            object.insert(
                "freeze_limit".into(),
                json!(FREEZE_LIMIT.0 as f64 / FREEZE_LIMIT.1 as f64),
            );
            object.insert(
                "below_freeze_limit".into(),
                json!(run.card.below_freeze_limit()),
            );
        }
    }
    row
}

/// `baselines`: the two untrained rules (R-recency, R-nlet) and the reference
/// replies over the same episodes, judged by the v2 oracle, per cell, category
/// and distance. The instrument freezes only if both rules are below 0.6 on
/// every gated cell (MQAR per distance and open-relation recall on the
/// development cell); otherwise the report names the leaking templates.
fn baselines(args: &Args, out: &Path) -> Result<()> {
    let started = Instant::now();
    if args.optional("world").as_deref() != Some("v2") {
        return Err(invalid("baselines needs world=v2"));
    }
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let split = split_of(args)?;
    let conversations: usize = args.number("conversations", 2_000)?;
    let seed: u64 = args.number("seed", 9_101)?;
    let cells: bool = args.number("cells", true)?;
    let mix = mix_of(args)?;
    let tokenizer = load_tokenizer(&tokenizer_path)?;
    let count = |text: &str| tokenizer.encode(text).len();
    let primary = Cell::same(split);
    let to_run: Vec<Cell> = if cells {
        Cell::ALL.to_vec()
    } else {
        vec![primary]
    };
    let mut by_cell: BTreeMap<Cell, BTreeMap<&'static str, RuleRun>> = BTreeMap::new();
    for cell in &to_run {
        let mut world = MWorld2::new(&count, mix)?;
        let mut rng = Rng::new(seed);
        by_cell.insert(
            *cell,
            run_rules(&mut world, &mut rng, *cell, conversations)?,
        );
    }
    let cells_json: BTreeMap<&str, Value> = by_cell
        .iter()
        .map(|(cell, runs)| {
            let rows: BTreeMap<&str, Value> = runs
                .iter()
                .map(|(name, run)| (*name, baseline_row(name, run)))
                .collect();
            (cell.key(), json!(rows))
        })
        .collect();
    // The freeze is decided on the development cell alone.
    let freeze = by_cell.get(&Cell::GATED).map(freeze_report);
    match &freeze {
        Some(freeze) => {
            println!("instrument_freeze_ok: {}", freeze["instrument_freeze_ok"]);
            for (rule, rows) in freeze["gated"].as_object().into_iter().flatten() {
                for (key, row) in rows.as_object().into_iter().flatten() {
                    println!(
                        "{rule} {key}: {}/{} ({:.3}){}",
                        row["pass"],
                        row["of"],
                        row["rate"].as_f64().unwrap_or(0.0),
                        if row["below_limit"] == json!(true) {
                            ""
                        } else {
                            "  LEAK"
                        }
                    );
                }
            }
        }
        None => println!(
            "instrument_freeze_ok: not decided (the development cell was not run; use cells=true or split=development)"
        ),
    }
    let executable = std::env::current_exe()?;
    let report = json!({
        "schema": "uor-r4.m-world-baselines/1",
        "executable_sha256": sha256_file(&executable)?,
        "world": "m-world-v2",
        "world_digest": MWorld2::digest(),
        "revision": REVISION,
        "tokenizer_sha256": sha256_file(&tokenizer_path)?,
        "split": split,
        "seed": seed,
        "mix": mix,
        "conversations_per_cell": conversations,
        "cells_run": to_run.iter().map(|c| c.key()).collect::<Vec<_>>(),
        "history": "reference: every retrieval turn is answered after the episode's own earlier replies",
        "rules": {
            "R-recency": "the most recent open-pool value the generator recorded in the history",
            "R-nlet": "the words after the latest earlier occurrence of the query's last two words, up to the clause end; otherwise \"I don't know.\"",
            "reference": "the world's own reference replies through the oracle: 1.0 unless the harness is broken",
        },
        "row_fields": "the reference row is a model row and carries its a1_gate; a rule row is not a model and carries its scores, freeze_limit and below_freeze_limit (every gated key of that cell below the limit; the freeze itself is decided on the development cell, in freeze and instrument_freeze_ok) and no a1_gate",
        "instrument_freeze_ok": freeze.as_ref().map(|f| f["instrument_freeze_ok"].clone()),
        "freeze_scope": "decided on dev_phrasing x dev_value (the development split); null when that cell was not run",
        "freeze": freeze,
        "cells": cells_json,
        "wall_seconds": started.elapsed().as_secs_f64(),
    });
    fs::write(
        out.join("m_world_baselines.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}

/// `route`: the untrained retrieval routes ([`Rule::ROUTES`]: R-sieve, the
/// identity channel of the log-sieve design, E1) and the reference replies
/// over the same episodes `baselines` draws, judged by the v2 oracle, per cell.
/// A route is not a freeze rule and no freeze is computed; its `a1_gate`
/// reads the keys a model's does. It tests the instrument (whether its keys
/// repeat verbatim), not a model.
fn route(args: &Args, out: &Path) -> Result<()> {
    let started = Instant::now();
    if args.optional("world").as_deref() != Some("v2") {
        return Err(invalid("route needs world=v2"));
    }
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let split = split_of(args)?;
    let conversations: usize = args.number("conversations", 2_000)?;
    let seed: u64 = args.number("seed", 9_101)?;
    let cells: bool = args.number("cells", true)?;
    let mix = mix_of(args)?;
    let tokenizer = load_tokenizer(&tokenizer_path)?;
    let count = |text: &str| tokenizer.encode(text).len();
    let to_run: Vec<Cell> = if cells {
        Cell::ALL.to_vec()
    } else {
        vec![Cell::same(split)]
    };
    let mut cells_json: BTreeMap<&str, Value> = BTreeMap::new();
    for cell in &to_run {
        let mut world = MWorld2::new(&count, mix)?;
        let mut rng = Rng::new(seed);
        let runs = run_route(&mut world, &mut rng, *cell, conversations)?;
        let rows: BTreeMap<&str, Value> = runs
            .iter()
            .map(|(name, run)| (*name, run.card.to_json()))
            .collect();
        if let Some(sieve) = rows.get("R-sieve") {
            println!(
                "{} R-sieve: MQAR d16 {} d64 {} d200 {}; open relation {}; a1_gate {}",
                cell.key(),
                sieve["mqar"]["by_distance"]["16"]["rate"],
                sieve["mqar"]["by_distance"]["64"]["rate"],
                sieve["mqar"]["by_distance"]["200"]["rate"],
                sieve["relation"]["open"]["rate"],
                sieve["a1_gate"]
            );
        }
        cells_json.insert(cell.key(), json!(rows));
    }
    let report = json!({
        "schema": "uor-r4.m-world-route/1",
        "executable_sha256": sha256_file(&std::env::current_exe()?)?,
        "world": "m-world-v2",
        "world_digest": MWorld2::digest(),
        "revision": REVISION,
        "tokenizer_sha256": sha256_file(&tokenizer_path)?,
        "split": split,
        "seed": seed,
        "mix": mix,
        "conversations_per_cell": conversations,
        "cells_run": to_run.iter().map(|c| c.key()).collect::<Vec<_>>(),
        "history": "reference: every retrieval turn is answered after the episode's own earlier replies",
        "routes": {
            "R-sieve": "the identity channel of docs/integration/log-sieve-retrieval-design-2026-10-01.md §2.2, untrained: the latest user clause sharing the most query content words (outside the world's fixed vocabulary) gives the words after the first shared one, as \"It's {value}.\"; otherwise \"I don't know.\"",
            "reference": "the world's own reference replies through the oracle: 1.0 unless the harness is broken",
        },
        "scope": "an untrained route over the instrument: it tests whether the instrument's keys repeat verbatim, not a model; no freeze is computed",
        "cells": cells_json,
        "wall_seconds": started.elapsed().as_secs_f64(),
    });
    fs::write(
        out.join("m_world_route.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}

/// Teacher paraphrases from a comma-separated list of `teacher-paraphrase`
/// files, filled with `train`'s values (`paraphrase_examples`), with each
/// file's SHA-256 and counts.
fn load_paraphrases(list: &str, train: &[Example]) -> Result<(Vec<Example>, Value)> {
    let (mut all, mut files) = (Vec::new(), Vec::new());
    for path in list.split(',').filter(|p| !p.trim().is_empty()) {
        let path = PathBuf::from(path.trim());
        let (examples, skipped) = paraphrase_examples(&fs::read_to_string(&path)?, train)?;
        files.push(json!({
            "path": path.display().to_string(),
            "sha256": sha256_file(&path)?,
            "examples": examples.len(),
            "skipped_without_a_value": skipped,
        }));
        all.extend(examples);
    }
    if files.is_empty() {
        return Err(invalid("paraphrases= names no file"));
    }
    let record = json!({"files": files, "examples": all.len()});
    Ok((all, record))
}

/// The relation channel's word table for `evaluate recall=route`: fitted on
/// every user turn of relation-heavy draws with training phrasings (both value
/// splits, 2,000 conversations each, seed 9,101: `compiler`'s draw), plus any
/// teacher paraphrases, by 400 sparse gradient steps (rate 0.5, l2 1e-4).
fn fit_route(
    count: &dyn Fn(&str) -> usize,
    paraphrases: Option<String>,
    trunk: Option<(&dyn Fn(&str) -> Result<Vec<f64>>, Value)>,
) -> Result<(RelationRoute, Value)> {
    let mix = Mix {
        mqar: 0.15,
        copy: 0.05,
        relation: 0.60,
        other: 0.20,
        ..Mix::default()
    };
    let mut train = Vec::new();
    for value in [Split::Train, Split::Development] {
        let mut world = MWorld2::new(count, mix)?;
        let mut rng = Rng::new(9_101);
        train.extend(collect(
            &mut world,
            &mut rng,
            Cell::new(Split::Train, value),
            2_000,
        )?);
    }
    let turns = train.len();
    let paraphrase_record = match paraphrases {
        Some(list) => {
            let (examples, record) = load_paraphrases(&list, &train)?;
            train.extend(examples);
            Some(record)
        }
        None => None,
    };
    let (route, trunk_record) = match trunk {
        None => (RelationRoute::fit(&train, 400, 0.5, 1e-4)?, Value::Null),
        Some((features, identity)) => {
            let x: Vec<Vec<f64>> = train
                .iter()
                .map(|e| features(&e.text))
                .collect::<Result<_>>()?;
            (
                RelationRoute::fit_with(&train, Some(&x), 400, 0.5, 1e-4)?,
                identity,
            )
        }
    };
    let record = json!({
        "trunk": trunk_record,
        "channels": "identity (R-sieve) first; then the relation channel: the latest earlier user turn the word table names as asserting or updating the asked relation, and its words outside the world's fixed vocabulary",
        "table": "sparse softmax over the words of a turn, 400 full-batch steps, rate 0.5, l2 1e-4",
        "training_turns": turns,
        "draw": "relation-heavy mix (MQAR .15, copy .05, relation .60, other .20), training phrasings, both value splits, 2,000 conversations each, seed 9101",
        "paraphrases": paraphrase_record,
    });
    Ok((route, record))
}

/// `compiler` (E3 of the log-sieve design): softmax heads that name a user
/// turn's relation and act, fitted on turns with training phrasings and scored
/// on turns with development phrasings, from the frozen trunk's states and,
/// as a control, from the turn's words (`uor_r4_training::relation_compiler`).
/// Relation accuracy is over the turns that state, update or ask a relation;
/// act accuracy is over every user turn.
fn compiler(args: &Args, out: &Path) -> Result<()> {
    let started = Instant::now();
    if args.optional("world").as_deref() != Some("v2") {
        return Err(invalid("compiler needs world=v2"));
    }
    let model_dir = PathBuf::from(args.required("model")?);
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let train_conversations: usize = args.number("train_conversations", 2_000)?;
    let eval_conversations: usize = args.number("eval_conversations", 1_000)?;
    let seed: u64 = args.number("seed", 9_101)?;
    let steps: usize = args.number("steps", 400)?;
    let rate: f64 = args.number("rate", 0.5)?;
    let l2: f64 = args.number("l2", 1e-4)?;
    let lexical_head: bool = args.number("lexical", true)?;
    let tokenizer = load_tokenizer(&tokenizer_path)?;
    let protocol = DialogueProtocol::literal_roles_v1(&tokenizer)
        .map_err(|e| invalid(format!("protocol: {e}")))?;
    let encoder = protocol
        .bind(&tokenizer)
        .map_err(|e| invalid(format!("protocol: {e}")))?;
    let device = Device::Cpu;
    let (model, identity, _) = load_model(&model_dir, &device, &args.selection)?;
    let count = |text: &str| tokenizer.encode(text).len();
    // Relation-heavy episodes: more relation turns per draw, the same templates.
    let mix = Mix {
        mqar: 0.15,
        copy: 0.05,
        relation: 0.60,
        other: 0.20,
        ..Mix::default()
    };
    let draw = |cell: Cell, conversations: usize, seed: u64| -> Result<Vec<Example>> {
        let mut world = MWorld2::new(&count, mix)?;
        let mut rng = Rng::new(seed);
        collect(&mut world, &mut rng, cell, conversations)
    };
    let mut train = Vec::new();
    for value in [Split::Train, Split::Development] {
        train.extend(draw(
            Cell::new(Split::Train, value),
            train_conversations,
            seed,
        )?);
    }
    // Teacher paraphrases (`teacher-paraphrase`), filled with training values,
    // join the training turns; the development turns are untouched.
    let paraphrases = match args.optional("paraphrases") {
        Some(list) => {
            let (examples, record) = load_paraphrases(&list, &train)?;
            train.extend(examples);
            Some(record)
        }
        None => None,
    };
    let tests: Vec<(Cell, Vec<Example>)> = [Split::Development, Split::Train]
        .into_iter()
        .map(|value| {
            let cell = Cell::new(Split::Development, value);
            Ok((cell, draw(cell, eval_conversations, seed + 1)?))
        })
        .collect::<Result<_>>()?;
    let relations: Vec<String> = relation_names()
        .iter()
        .map(|name| (*name).to_owned())
        .chain([RC_NONE.to_owned()])
        .collect();
    let acts: Vec<String> = ACTS.iter().map(|act| (*act).to_owned()).collect();
    let relation_index = |example: &Example| {
        relations
            .iter()
            .position(|r| *r == example.relation)
            .unwrap_or(relations.len() - 1)
    };
    let act_index = |example: &Example| {
        ACTS.iter()
            .position(|a| *a == example.act)
            .unwrap_or(ACTS.len() - 1)
    };
    let none = relations.len() - 1;
    let lexicon = Lexicon::fit(train.iter().map(|e| e.text.as_str()));
    let trunk = |examples: &[Example]| -> Result<Vec<Vec<f64>>> {
        examples
            .iter()
            .map(|e| trunk_features(&model, &encoder, protocol.bos_id, &e.text))
            .collect()
    };
    let lexical = |examples: &[Example]| -> Vec<Vec<f64>> {
        examples.iter().map(|e| lexicon.features(&e.text)).collect()
    };
    let train_relation: Vec<usize> = train.iter().map(relation_index).collect();
    let train_act: Vec<usize> = train.iter().map(act_index).collect();
    // The dense heads: the trunk's states; and, unless `lexical=false` (the
    // word heads depend on the turns alone, not on the trunk), the turn's
    // words and both together. The sparse `table` head is the route's word
    // table (`recall=route`): present words at their standardized scale.
    let dense = |name: &str, trunk_x: &[Vec<f64>], examples: &[Example]| -> Vec<Vec<f64>> {
        match name {
            "trunk" => trunk_x.to_vec(),
            "lexical" => lexical(examples),
            _ => trunk_x
                .iter()
                .zip(lexical(examples))
                .map(|(t, w)| t.iter().copied().chain(w).collect())
                .collect(),
        }
    };
    let dense_names: &[&str] = if lexical_head {
        &["trunk", "lexical", "combined"]
    } else {
        &["trunk"]
    };
    let train_trunk = trunk(&train)?;
    let mut heads = BTreeMap::new();
    for name in dense_names {
        let x = dense(name, &train_trunk, &train);
        let relation_head = Softmax::fit(&x, &train_relation, relations.len(), steps, rate, l2)?;
        let act_head = Softmax::fit(&x, &train_act, acts.len(), steps, rate, l2)?;
        heads.insert(*name, (relation_head, act_head));
    }
    let table_rows: Vec<Vec<(usize, f64)>> =
        train.iter().map(|e| lexicon.scaled(&e.text)).collect();
    let dim = lexicon.len().max(1);
    let table = (
        SparseSoftmax::fit(
            &table_rows,
            &train_relation,
            relations.len(),
            dim,
            steps,
            rate,
            l2,
        )?,
        SparseSoftmax::fit(&table_rows, &train_act, acts.len(), dim, steps, rate, l2)?,
    );
    let mut cells_json = BTreeMap::new();
    for (cell, examples) in &tests {
        let truth_relation: Vec<usize> = examples.iter().map(relation_index).collect();
        let truth_act: Vec<usize> = examples.iter().map(act_index).collect();
        let test_trunk = trunk(examples)?;
        let mut predictions: Vec<(&str, Vec<usize>, Vec<usize>)> = Vec::new();
        for (name, (relation_head, act_head)) in &heads {
            let x = dense(name, &test_trunk, examples);
            predictions.push((
                *name,
                x.iter().map(|row| relation_head.predict(row)).collect(),
                x.iter().map(|row| act_head.predict(row)).collect(),
            ));
        }
        let rows_x: Vec<Vec<(usize, f64)>> =
            examples.iter().map(|e| lexicon.scaled(&e.text)).collect();
        predictions.push((
            "table",
            rows_x.iter().map(|row| table.0.predict(row)).collect(),
            rows_x.iter().map(|row| table.1.predict(row)).collect(),
        ));
        let mut rows = BTreeMap::new();
        for (name, predicted_relation, predicted_act) in predictions {
            let relation_turns = score(&relations, &truth_relation, &predicted_relation, |t| {
                t != none
            });
            let all_turns = score(&relations, &truth_relation, &predicted_relation, |_| true);
            let act = score(&acts, &truth_act, &predicted_act, |_| true);
            println!(
                "{} {name}: relation {}/{} ({:.3}) on relation turns; act {}/{} ({:.3})",
                cell.key(),
                relation_turns["pass"],
                relation_turns["of"],
                relation_turns["rate"].as_f64().unwrap_or(0.0),
                act["pass"],
                act["of"],
                act["rate"].as_f64().unwrap_or(0.0),
            );
            rows.insert(
                (*name).to_owned(),
                json!({
                    "relation_on_relation_turns": relation_turns,
                    "relation_on_all_turns": all_turns,
                    "act_on_all_turns": act,
                }),
            );
        }
        cells_json.insert(cell.key(), json!(rows));
    }
    let gate_row = &cells_json[Cell::GATED.key()]["trunk"];
    let gate = gate_row["relation_on_relation_turns"]["rate"]
        .as_f64()
        .unwrap_or(0.0)
        >= 0.9
        && gate_row["act_on_all_turns"]["rate"].as_f64().unwrap_or(0.0) >= 0.95;
    println!("E3 gate (trunk, dev x dev: relation >= 0.9 and act >= 0.95): {gate}");
    let report = json!({
        "schema": "uor-r4.m-world-compiler/1",
        "executable_sha256": sha256_file(&std::env::current_exe()?)?,
        "world": "m-world-v2",
        "world_digest": MWorld2::digest(),
        "revision": REVISION,
        "model": model_dir.display().to_string(),
        "model_identity": identity,
        "tokenizer_sha256": sha256_file(&tokenizer_path)?,
        "mix": mix,
        "seed": seed,
        "train": {
            "cells": ["train_phrasing_train_value", "train_phrasing_dev_value"],
            "conversations_per_cell": train_conversations,
            "turns": train.len(),
            "relation_turns": train_relation.iter().filter(|&&r| r != none).count(),
            "paraphrases": paraphrases,
            "lexicon_words": lexicon.len(),
        },
        "eval_conversations_per_cell": eval_conversations,
        "fit": {"steps": steps, "rate": rate, "l2": l2, "lexical_head": lexical_head, "features": {
            "trunk": "final normalized state at the assistant marker and its mean over the turn read alone (2 x width)",
            "lexical": "binary words of the training turns (a control)",
            "combined": "trunk and lexical features together (dense, standardized)",
            "table": "the route's sparse word table: present words at their standardized scale sqrt((1-p)/p), fitted without standardizing absent ones",
        }},
        "labels": {"relations": relations, "acts": acts},
        "gate": {
            "rule": "trunk head on dev_phrasing x dev_value: relation accuracy >= 0.9 on relation turns and act accuracy >= 0.95 on all turns",
            "met": gate,
        },
        "cells": cells_json,
        "wall_seconds": started.elapsed().as_secs_f64(),
    });
    fs::write(
        out.join("m_world_compiler.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}

/// `probe`: the sealed 40-item English retrieval probe, answered greedily after
/// each item's reference history and teacher-forced.
fn probe_evaluate(args: &Args, out: &Path) -> Result<()> {
    let started = Instant::now();
    let model_dir = PathBuf::from(args.required("model")?);
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let max_new_tokens: usize = args.number("max_new_tokens", 48)?;
    let items = probe()?;
    let tokenizer = load_tokenizer(&tokenizer_path)?;
    let protocol = DialogueProtocol::literal_roles_v1(&tokenizer)
        .map_err(|e| invalid(format!("protocol: {e}")))?;
    let encoder = protocol
        .bind(&tokenizer)
        .map_err(|e| invalid(format!("protocol: {e}")))?;
    let device = Device::Cpu;
    let (model, identity, selection_override) = open_model(&model_dir, &device, &args.selection)?;
    let context = model.config.context;
    let decode = |ids: &[u32]| tokenizer.decode(ids);
    let count = |text: &str| tokenizer.encode(text).len();
    let meter = Meter::new(&count);
    let mut tallies: BTreeMap<ProbeGroup, (usize, usize)> = BTreeMap::new();
    let mut nll: BTreeMap<ProbeGroup, NllCard> = BTreeMap::new();
    let mut rows: Vec<Value> = Vec::with_capacity(items.len());
    let (mut passed, mut skipped, mut unscored) = (0usize, 0usize, 0usize);
    for item in &items {
        let conversation = item.conversation(&meter)?;
        let Some(last) = conversation.turns.len().checked_sub(1) else {
            continue;
        };
        let turn = &conversation.turns[last];
        let distance = (last >= 1).then(|| meter.distance(&conversation.turns));
        if conversation.tokens > context {
            skipped += 1;
            rows.push(json!({
                "id": item.id,
                "group": item.group,
                "tokens": conversation.tokens,
                "skipped": "the item's document is longer than the model's context",
            }));
            continue;
        }
        let answer = answer_turn(
            &model,
            &encoder,
            &protocol,
            &decode,
            &conversation.turns,
            last,
            max_new_tokens,
            context,
        )?;
        let pass = judge_v2(&turn.checks, &turn.user, &answer.text);
        passed += usize::from(pass);
        let tally = tallies.entry(item.group).or_default();
        tally.0 += usize::from(pass);
        tally.1 += 1;
        let layout = answer_layout(&encoder, &tokenizer, &conversation.turns, last)?;
        let mut row = json!({
            "id": item.id,
            "group": item.group,
            "tokens": conversation.tokens,
            "distance": distance,
            "user": turn.user,
            "reference_reply": turn.reply,
            "reply": answer.text,
            "pass": pass,
            "stop": answer.stop,
            "history_tokens": answer.history_tokens,
        });
        match teacher_forced(&model, &layout)? {
            Some(scored) => {
                nll.entry(item.group).or_default().record(turn, &scored);
                row["teacher_forced"] = json!({
                    "reply_tokens": scored.reply_tokens,
                    "reply_nll": scored.reply_nll,
                    "answer_tokens": scored.answer_tokens,
                    "answer_nll": scored.answer_nll,
                });
            }
            None => unscored += 1,
        }
        rows.push(row);
    }
    let scored_items = items.len() - skipped;
    let by_group: BTreeMap<&str, Value> = tallies
        .iter()
        .map(|(group, &tally)| (group.name(), rate_json(tally)))
        .collect();
    let teacher_forced_by_group: BTreeMap<&str, Value> = nll
        .iter()
        .map(|(group, card)| (group.name(), card.to_json()))
        .collect();
    for (group, cell) in &by_group {
        println!(
            "probe {group}: {}/{} ({:.3})",
            cell["pass"],
            cell["of"],
            cell["rate"].as_f64().unwrap_or(0.0)
        );
    }
    println!("probe: {passed}/{scored_items} pass ({skipped} skipped for the context)");
    let executable = std::env::current_exe()?;
    let report = json!({
        "schema": "uor-r4.m-world-probe-evaluation/1",
        "executable_sha256": sha256_file(&executable)?,
        "probe_sha256": probe_sha256(),
        "world_digest": MWorld2::digest(),
        "model": model_dir.display().to_string(),
        "model_identity": identity,
        "selection_override": selection_override,
        "tokenizer_sha256": sha256_file(&tokenizer_path)?,
        "protocol_identity": protocol.identity().map_err(|e| invalid(format!("protocol: {e}")))?,
        "history": "reference: the scored turn is answered after the item's own earlier replies",
        "scope": "a sealed English retrieval probe, excluded from training by the 8-word user-turn overlap screen (not complete decontamination); a result on it is not chat quality",
        "context": context,
        "max_new_tokens": max_new_tokens,
        "items": items.len(),
        "scored": scored_items,
        "skipped_context": skipped,
        "teacher_forced_unscored": unscored,
        "pass": rate_json((passed, scored_items)),
        "by_group": by_group,
        "teacher_forced_by_group": teacher_forced_by_group,
        "results": rows,
        "wall_seconds": started.elapsed().as_secs_f64(),
    });
    fs::write(
        out.join("m_world_probe.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}

/// `probe-static`: the probe's token counts, whether each item fits the
/// context, its distance, and the two untrained rules' replies on it. Needs the
/// tokenizer and no model.
fn probe_static(args: &Args, out: &Path) -> Result<()> {
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let context: usize = args.number("context", CONTEXT)?;
    let items = probe()?;
    let tokenizer = load_tokenizer(&tokenizer_path)?;
    let count = |text: &str| tokenizer.encode(text).len();
    let meter = Meter::new(&count);
    let mut report = static_report(&items, &meter, context)?;
    report["tokenizer_sha256"] = json!(sha256_file(&tokenizer_path)?);
    for (group, cell) in report["by_group"].as_object().into_iter().flatten() {
        println!(
            "{group}: {} items, {} fit {context} tokens; rules {}",
            cell["items"], cell["fit_context"], cell["rules"]
        );
    }
    fs::write(
        out.join("m_world_probe_static.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}

/// One of the amendment modes, if `mode` is one: parse its arguments, claim its
/// report root, run, seal and verify.
/// The action a user turn should compile to, from its typed intent and
/// template (evaluator knowledge, used only to score).
fn gold_action(compiler: &SavedCompiler, turn_text: &str, turn: &Turn2) -> Result<CompiledAction> {
    let (relation, act) = label(turn);
    if relation == RC_NONE {
        return Ok(CompiledAction::Unresolved {
            reason: "no relation".into(),
        });
    }
    let id = compiler
        .relation_id(&relation)
        .ok_or_else(|| invalid(format!("relation {relation} has no store ID")))?;
    let example = Example {
        text: turn_text.to_owned(),
        relation,
        act,
        template: turn.tag.template.clone(),
    };
    Ok(match (act, example.slot_span()) {
        ("query", _) => CompiledAction::QueryCurrent { relation: id },
        ("assert", Some((start, end))) => CompiledAction::Assert {
            relation: id,
            span: SourceSpan { start, end },
        },
        ("update", Some((start, end))) => CompiledAction::Correct {
            relation: id,
            span: SourceSpan { start, end },
        },
        _ => CompiledAction::Unresolved {
            reason: "the statement's slot cannot be recovered".into(),
        },
    })
}

fn action_kind(action: &CompiledAction) -> &'static str {
    match action {
        CompiledAction::Assert { .. } => "assert",
        CompiledAction::Correct { .. } => "correct",
        CompiledAction::QueryCurrent { .. } | CompiledAction::Query { .. } => "query",
        CompiledAction::Unresolved { .. } => "unresolved",
    }
}

/// Exact-action agreement: the same kind, relation and value span, any
/// unresolved reason.
fn same_action(a: &CompiledAction, b: &CompiledAction) -> bool {
    match (a, b) {
        (CompiledAction::Unresolved { .. }, CompiledAction::Unresolved { .. }) => true,
        _ => a == b,
    }
}

/// `compiler-save`: fit the saved compiler of the grounded session (the
/// relation/act table of `recall=route` plus the value-span head), write its
/// artifact `compiler.json`, and score its actions on development phrasings.
/// `values=train` (default) fits on training values only; `values=both` is
/// `recall=route`'s draw (both value splits, so development values are seen).
fn compiler_save(args: &Args, out: &Path) -> Result<()> {
    let started = Instant::now();
    if args.optional("world").as_deref() != Some("v2") {
        return Err(invalid("compiler-save needs world=v2"));
    }
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let train_conversations: usize = args.number("train_conversations", 2_000)?;
    let eval_conversations: usize = args.number("eval_conversations", 1_000)?;
    let seed: u64 = args.number("seed", 9_101)?;
    let values = args.optional("values").unwrap_or_else(|| "train".into());
    let settings = CompilerSettings {
        act_rule: ActRule::parse(&args.optional("act_rule").unwrap_or_else(|| "table".into()))?,
        relation_mode: RelationMode::parse(
            &args
                .optional("relation_mode")
                .unwrap_or_else(|| "table".into()),
        )?,
        ..CompilerSettings::default()
    };
    let tokenizer = load_tokenizer(&tokenizer_path)?;
    let tokenizer_sha256 = sha256_file(&tokenizer_path)?;
    let count = |text: &str| tokenizer.encode(text).len();
    let mix = Mix {
        mqar: 0.15,
        copy: 0.05,
        relation: 0.60,
        other: 0.20,
        ..Mix::default()
    };
    let draw = |cell: Cell, conversations: usize, seed: u64| -> Result<Vec<Example>> {
        let mut world = MWorld2::new(&count, mix)?;
        let mut rng = Rng::new(seed);
        collect(&mut world, &mut rng, cell, conversations)
    };
    let mut train = Vec::new();
    let draws: Vec<(Split, usize)> = match values.as_str() {
        "train" => vec![(Split::Train, 2 * train_conversations)],
        "both" => vec![
            (Split::Train, train_conversations),
            (Split::Development, train_conversations),
        ],
        other => return Err(invalid(format!("unknown values={other}"))),
    };
    for &(value, conversations) in &draws {
        train.extend(draw(Cell::new(Split::Train, value), conversations, seed)?);
    }
    let turns = train.len();
    let paraphrases = match args.optional("paraphrases") {
        Some(list) => {
            let (examples, record) = load_paraphrases(&list, &train)?;
            train.extend(examples);
            record
        }
        None => Value::Null,
    };
    let training = json!({
        "world": "m-world-v2",
        "world_digest": MWorld2::digest(),
        "mix": "relation-heavy: mqar .15, copy .05, relation .60, other .20",
        "phrasing": "train",
        "values": values,
        "draws": draws
            .iter()
            .map(|(value, conversations)| json!({
                "value": format!("{value:?}").to_lowercase(),
                "conversations": conversations,
                "seed": seed,
            }))
            .collect::<Vec<_>>(),
        "world_turns": turns,
        "paraphrases": paraphrases,
        "examples": train.len(),
    });
    let fit_started = Instant::now();
    let trunk = match args.optional("trunk") {
        Some(directory) => Some(Trunk::load(
            Path::new(&directory),
            &fs::read(&tokenizer_path)?,
            &Device::Cpu,
        )?),
        None => None,
    };
    let saved =
        SavedCompiler::fit_with(&train, &tokenizer_sha256, training.clone(), settings, trunk)?;
    let fit_seconds = fit_started.elapsed().as_secs_f64();
    fs::write(out.join("compiler.json"), saved.bytes())?;
    let mut cells = serde_json::Map::new();
    for value in [Split::Development, Split::Train] {
        let cell = Cell::new(Split::Development, value);
        let examples = draw(cell, eval_conversations, seed + 1)?;
        // (pass, of) tallies.
        let mut tally: BTreeMap<String, (usize, usize)> = BTreeMap::new();
        let mut add = |key: String, pass: bool| {
            let entry = tally.entry(key).or_default();
            entry.0 += usize::from(pass);
            entry.1 += 1;
        };
        let mut misses = Vec::new();
        // One compile per turn, in parallel; tallies stay sequential.
        let predictions = parallel_map(&examples, |example| {
            let action = saved.action(&example.text)?;
            let classified = if saved.relation_mode() == RelationMode::OpModel {
                classified_of(&saved, &action)
            } else {
                let (relation, act) = saved.classify(&example.text)?;
                (relation.to_owned(), act)
            };
            Ok((classified, action))
        })?;
        for (example, ((relation, act), predicted)) in examples.iter().zip(predictions) {
            if example.relation != RC_NONE {
                add("relation".into(), relation == example.relation);
            }
            add("act".into(), act == example.act);
            let gold_span = example.slot_span();
            let decoded = saved.span().decode(&example.text);
            let writes = example.relation != RC_NONE && matches!(example.act, "assert" | "update");
            if writes {
                if let Some(gold) = gold_span {
                    add("span/exact".into(), decoded == Some(gold));
                }
            } else {
                add("span/no_write".into(), decoded.is_none());
            }
            let gold = gold_of(&saved, example)?;
            let pass = same_action(&predicted, &gold);
            add(format!("action/{}", action_kind(&gold)), pass);
            if example.relation != RC_NONE {
                add(format!("action/relation/{}", example.relation), pass);
            }
            if !pass && misses.len() < 40 {
                misses.push(json!({
                    "text": example.text, "gold": gold, "predicted": predicted,
                }));
            }
        }
        let rates: serde_json::Map<String, Value> = tally
            .iter()
            .map(|(key, &(pass, of))| (key.clone(), rate_json((pass, of))))
            .collect();
        println!(
            "{:?} phrasing x {:?} values: {}",
            cell.phrasing,
            cell.value,
            ["relation", "act", "span/exact", "span/no_write"]
                .iter()
                .map(|key| {
                    let (pass, of) = tally.get(*key).copied().unwrap_or_default();
                    format!("{key} {pass}/{of}")
                })
                .collect::<Vec<_>>()
                .join(", ")
        );
        cells.insert(
            format!("{:?}_x_{:?}", cell.phrasing, cell.value).to_lowercase(),
            json!({"turns": examples.len(), "rates": rates, "first_misses": misses}),
        );
    }
    let executable = std::env::current_exe()?;
    let report = json!({
        "schema": "uor-r4.m-world-compiler-save/1",
        "executable_sha256": sha256_file(&executable)?,
        "artifact": "compiler.json",
        "artifact_sha256": saved.identity().artifact_sha256,
        "identity": saved.identity(),
        "training": training,
        "settings": format!("{settings:?}"),
        "fit_seconds": fit_seconds,
        "evaluation": {
            "draw": format!("relation-heavy mix, development phrasings, {eval_conversations} conversations per value split, seed {}", seed + 1),
            "scope": "development phrasings; relation over turns naming a relation; act over every turn; span/exact over statements with a recoverable slot; span/no_write over queries and turns naming no relation; action is the exact compiled action (kind, relation and value span)",
            "cells": cells,
        },
        "wall_seconds": started.elapsed().as_secs_f64(),
    });
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    Ok(())
}

/// `map` over `items` on all available cores, results in order. Each worker
/// takes a contiguous chunk; the first error stops the collection.
fn parallel_map<T: Sync, R: Send>(
    items: &[T],
    map: impl Fn(&T) -> Result<R> + Sync,
) -> Result<Vec<R>> {
    let workers = std::thread::available_parallelism()
        .map_or(4, |n| n.get())
        .clamp(1, 8);
    let chunk = items.len().div_ceil(workers).max(1);
    let map = &map;
    std::thread::scope(|scope| {
        let handles: Vec<_> = items
            .chunks(chunk)
            .map(|part| scope.spawn(move || part.iter().map(map).collect::<Result<Vec<R>>>()))
            .collect();
        let mut out = Vec::with_capacity(items.len());
        for handle in handles {
            out.extend(
                handle
                    .join()
                    .map_err(|_| invalid("an evaluation worker panicked"))??,
            );
        }
        Ok(out)
    })
}

/// The relation and act an action implies (none for an unresolved turn).
fn classified_of(compiler: &SavedCompiler, action: &CompiledAction) -> (String, &'static str) {
    let name = |id: &u32| {
        compiler
            .identity()
            .relations
            .iter()
            .find(|label| label.id == *id)
            .map_or(RC_NONE.to_owned(), |label| label.name.clone())
    };
    match action {
        CompiledAction::Assert { relation, .. } => (name(relation), "assert"),
        CompiledAction::Correct { relation, .. } => (name(relation), "update"),
        CompiledAction::QueryCurrent { relation } | CompiledAction::Query { relation, .. } => {
            (name(relation), "query")
        }
        CompiledAction::Unresolved { .. } => (RC_NONE.to_owned(), RC_NONE),
    }
}

/// The gold action of a labelled example (see [`gold_action`]).
fn gold_of(compiler: &SavedCompiler, example: &Example) -> Result<CompiledAction> {
    let turn = Turn2 {
        intent: format!("{}_{}", example.relation, example.act),
        category: Category2::Relation,
        user: example.text.clone(),
        reply: String::new(),
        checks: Vec::new(),
        tag: Tag {
            template: example.template.clone(),
            ..Tag::default()
        },
    };
    gold_action(compiler, &example.text, &turn)
}

fn disposition_name(recall: &RecallDisposition) -> &'static str {
    match recall {
        RecallDisposition::NotRequested => "not_requested",
        RecallDisposition::Value => "value",
        RecallDisposition::Absent => "absent",
        RecallDisposition::Disabled => "disabled",
        RecallDisposition::Unsupported { .. } => "unsupported",
    }
}

/// `session`: M-world development conversations through the actual grounded
/// session (#962): the saved compiler (`compiler=`) predicts each user turn's
/// action, the exact store applies it, and the emitter (`model_root=`'s
/// model, saved as an inference checkpoint with an empty store) replies after
/// its own generated history. Arms: `default` (read and write), `no_read`
/// (no recall line enters the emitter) and `no_write` (statements are not
/// stored). The first `reload=` conversations of the default arm are also run
/// with a save and load at their middle turn, and must match.
fn session(args: &Args, out: &Path) -> Result<()> {
    let started = Instant::now();
    if args.optional("world").as_deref() != Some("v2") {
        return Err(invalid("session needs world=v2"));
    }
    let model_root = PathBuf::from(args.required("model_root")?);
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let compiler_path = PathBuf::from(args.required("compiler")?);
    let conversations: usize = args.number("conversations", 300)?;
    let seed: u64 = args.number("seed", 9_101)?;
    let max_new_tokens: usize = args.number("max_new_tokens", 32)?;
    let reload: usize = args.number("reload", 5)?;
    let arms: Vec<String> = args
        .optional("arms")
        .unwrap_or_else(|| "default,no_read,no_write".into())
        .split(',')
        .map(str::to_owned)
        .collect();
    let controls_of = |arm: &str| -> Result<TurnControls> {
        match arm {
            "default" => Ok(TurnControls::default()),
            "no_read" => Ok(TurnControls {
                read: false,
                write: true,
            }),
            "no_write" => Ok(TurnControls {
                read: true,
                write: false,
            }),
            other => Err(invalid(format!("unknown arm {other}"))),
        }
    };
    for arm in &arms {
        controls_of(arm)?;
    }
    let tokenizer_json = fs::read(&tokenizer_path)?;
    let tokenizer = load_tokenizer(&tokenizer_path)?;
    let compiler_bytes = fs::read(&compiler_path)?;
    let trunk_directory = args.optional("trunk").map(PathBuf::from);
    // How an op-model compiler combines its op model with its saved table
    // (a load-time choice, recorded in the report).
    let op_policy = OpPolicy::parse(args.optional("op_policy").as_deref().unwrap_or("op"))?;
    if op_policy != OpPolicy::Op && trunk_directory.is_none() {
        return Err(invalid("op_policy= needs the op model's trunk="));
    }
    // A combined compiler loads only with the trunk it binds; otherwise the
    // artifact's own schema chooses the grounded compiler.
    let load_compiler = || -> Result<GroundedCompiler> {
        match &trunk_directory {
            Some(directory) => Ok(GroundedCompiler::Legacy(
                SavedCompiler::load(
                    compiler_bytes.clone(),
                    Some(Trunk::load(directory, &tokenizer_json, &Device::Cpu)?),
                )?
                .with_op_policy(op_policy)?,
            )),
            None => GroundedCompiler::from_bytes(compiler_bytes.clone()),
        }
    };
    let compiler = load_compiler()?;
    let device = Device::Cpu;
    // The emitter as a sealed inference checkpoint with an empty store. Its
    // data identities are the training report's recorded inputs, bound by
    // the report's sealed manifest.
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
    let identity = CheckpointIdentity::from_tokenizer(
        &tokenizer_json,
        data,
        sealed_manifest_sha256(&model_root).map_err(|e| invalid(e.to_string()))?,
    )
    .map_err(|e| invalid(e.to_string()))?;
    let (model, model_identity, _) =
        load_model(&model_root.join("model"), &device, &Selection::default())?;
    let checkpoint = out.join("checkpoint");
    let store = StackStore::new(1, 8).map_err(|e| invalid(e.to_string()))?;
    save_checkpoint(&checkpoint, &model, &identity, Some(&store))
        .map_err(|e| invalid(e.to_string()))?;
    drop(model);
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
    let open = |compiler: GroundedCompiler| {
        GroundedSession::from_checkpoint_path(
            &checkpoint,
            tokenizer_json.clone(),
            compiler,
            scope.clone(),
            limits.clone(),
            &device,
        )
        .map_err(|e| invalid(e.to_string()))
    };
    let count = |text: &str| tokenizer.encode(text).len();
    let mut world = MWorld2::new(&count, Mix::default())?;
    let mut rng = Rng::new(seed);
    let drawn: Vec<Conversation2> = (0..conversations)
        .map(|_| world.conversation(&mut rng, Split::Development))
        .collect::<Result<_>>()?;
    let mut arm_reports = serde_json::Map::new();
    let mut default_outcomes: Vec<Vec<Option<TurnOutcome>>> = Vec::new();
    for arm in &arms {
        let controls = controls_of(arm)?;
        let mut card = Scorecard::default();
        let mut tally: BTreeMap<String, (usize, usize)> = BTreeMap::new();
        let mut add = |key: String, pass: bool| {
            let entry = tally.entry(key).or_default();
            entry.0 += usize::from(pass);
            entry.1 += 1;
        };
        let mut dispositions: BTreeMap<&str, usize> = BTreeMap::new();
        let (mut errors, mut error_examples) = (0usize, Vec::new());
        let (mut windowed, mut whole) = (0usize, 0usize);
        let mut judged = Vec::with_capacity(conversations);
        for (index, conversation) in drawn.iter().enumerate() {
            let mut session = open(compiler.clone())?;
            let mut all = true;
            let mut rows = Vec::new();
            let mut outcomes = Vec::new();
            for turn in &conversation.turns {
                let gold = gold_action(compiler.base(), &turn.user, turn)?;
                let (pass, row, outcome) = match session.turn_with_controls(&turn.user, controls) {
                    Ok(outcome) => {
                        let pass = judge_v2(&turn.checks, &turn.user, &outcome.reply_text);
                        *dispositions
                            .entry(disposition_name(&outcome.recall))
                            .or_default() += 1;
                        windowed += usize::from(outcome.retained_from_turn > 0);
                        let exact = same_action(&outcome.action, &gold);
                        add(format!("action/{}", action_kind(&gold)), exact);
                        let row = json!({
                            "intent": turn.intent, "category": turn.category,
                            "user": turn.user, "reply": outcome.reply_text, "pass": pass,
                            "action": outcome.action, "gold_action": gold,
                            "recall": disposition_name(&outcome.recall),
                            "retained_from_turn": outcome.retained_from_turn,
                            "stop": outcome.stop,
                        });
                        (pass, row, Some(outcome))
                    }
                    Err(error) => {
                        errors += 1;
                        if error_examples.len() < 20 {
                            error_examples.push(json!({
                                "conversation": index, "user": turn.user,
                                "error": error.to_string(),
                            }));
                        }
                        let row = json!({
                            "intent": turn.intent, "category": turn.category,
                            "user": turn.user, "pass": false, "gold_action": gold,
                            "error": error.to_string(),
                        });
                        (false, row, None)
                    }
                };
                all &= pass;
                card.record(turn, pass);
                rows.push(row);
                outcomes.push(outcome);
            }
            whole += usize::from(all);
            judged.push(json!({
                "id": format!("mw2-{index:04}"), "kind": conversation.kind,
                "all_pass": all, "turns": rows,
            }));
            if arm == "default" {
                default_outcomes.push(outcomes);
            }
        }
        let scores = card.to_json();
        println!(
            "arm {arm}: MQAR {}/{}; open relation {}/{}; closed relation {}/{}; \
             open abstain {}/{}; errors {errors}",
            scores["mqar"]["all"]["pass"],
            scores["mqar"]["all"]["of"],
            scores["relation"]["open"]["pass"],
            scores["relation"]["open"]["of"],
            scores["relation"]["closed"]["pass"],
            scores["relation"]["closed"]["of"],
            scores["relation"]["open_abstain"]["pass"],
            scores["relation"]["open_abstain"]["of"],
        );
        let mut report = json!({
            "controls": controls,
            "conversations_all_pass": rate_json((whole, conversations)),
            "compiler_actions": tally
                .iter()
                .map(|(key, &(pass, of))| (key.clone(), rate_json((pass, of))))
                .collect::<serde_json::Map<_, _>>(),
            "recall_dispositions": dispositions,
            "turns_with_dropped_history": windowed,
            "turn_errors": errors,
            "first_turn_errors": error_examples,
            "conversations": judged,
        });
        if let (Some(report), Some(scores)) = (report.as_object_mut(), scores.as_object()) {
            report.extend(scores.clone());
        }
        arm_reports.insert(arm.clone(), report);
    }
    // Save/load continuity: the same turns with a save and a load from disk
    // at the middle turn give the same outcomes.
    let mut continuity = Vec::new();
    for (index, conversation) in drawn.iter().enumerate().take(reload) {
        let Some(expected) = default_outcomes.get(index) else {
            break;
        };
        let middle = conversation.turns.len() / 2;
        let mut first = open(compiler.clone())?;
        let mut outcomes = Vec::new();
        for turn in &conversation.turns[..middle] {
            outcomes.push(first.turn(&turn.user).ok());
        }
        let root = out.join(format!("reload-{index:04}"));
        first.save(&root).map_err(|e| invalid(e.to_string()))?;
        drop(first);
        let reloaded = load_compiler()?;
        let mut second =
            GroundedSession::load(&root, reloaded, &device).map_err(|e| invalid(e.to_string()))?;
        for turn in &conversation.turns[middle..] {
            outcomes.push(second.turn(&turn.user).ok());
        }
        let equal = &outcomes == expected;
        println!("reload mw2-{index:04} at turn {middle}: equal={equal}");
        continuity.push(json!({
            "id": format!("mw2-{index:04}"), "save_after_turn": middle,
            "turns": conversation.turns.len(), "equal": equal,
        }));
    }
    let executable = std::env::current_exe()?;
    let report = json!({
        "schema": "uor-r4.m-world-session/1",
        "executable_sha256": sha256_file(&executable)?,
        "world": "m-world-v2",
        "world_digest": MWorld2::digest(),
        "split": "development phrasings and values (evaluate's default cell)",
        "seed": seed,
        "conversations": conversations,
        "mix": world.mix(),
        "history": "generated: each turn follows the session's own earlier replies and recall lines",
        "model_root": model_root.display().to_string(),
        "model_identity": model_identity,
        "checkpoint": "checkpoint",
        "compiler": compiler_path.display().to_string(),
        "compiler_identity": compiler.identity(),
        "trunk": trunk_directory.as_ref().map(|d| d.display().to_string()),
        "op_policy": format!("{op_policy:?}"),
        "tokenizer_sha256": sha256_file(&tokenizer_path)?,
        "limits": limits,
        "scope": "MQAR keys have no channel in the one-entity session: MQAR turns compile to unresolved and are scored without recall",
        "arms": arm_reports,
        "reload": {
            "rule": "the default arm's first conversations, saved after the middle turn into a new sealed envelope and loaded from disk with a compiler reloaded from its bytes, in this process",
            "conversations": continuity,
        },
        "wall_seconds": started.elapsed().as_secs_f64(),
    });
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    Ok(())
}

/// `compile-corpus`: training documents that teach a stack to compile a user
/// turn into its memory operation (`relation_mode=op_model`). Each document is
/// `System: Compile.` / `User: <turn>` / `Assistant: <op>`, where the op comes
/// from the turn's typed intent and template slot (`relation_compiler::op_text`).
/// Only the op is supervised. `train/` draws training phrasings with training
/// values (plus any teacher paraphrases); `dev/` draws training phrasings with
/// development values, so checkpoint selection never sees development phrasings.
fn compile_corpus(args: &Args, out: &Path) -> Result<()> {
    if args.optional("world").as_deref() != Some("v2") {
        return Err(invalid("compile-corpus needs world=v2"));
    }
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let conversations: usize = args.number("conversations", 4_000)?;
    let dev_conversations: usize = args.number("dev_conversations", 300)?;
    let seed: u64 = args.number("seed", 9_101)?;
    let tokenizer = load_tokenizer(&tokenizer_path)?;
    let protocol = DialogueProtocol::literal_roles_v1(&tokenizer)
        .map_err(|e| invalid(format!("protocol: {e}")))?;
    let encoder = protocol
        .bind(&tokenizer)
        .map_err(|e| invalid(format!("protocol: {e}")))?;
    let vocab = u32::try_from(tokenizer.vocab_size())
        .map_err(|_| invalid("the tokenizer's vocabulary does not fit the token store"))?;
    let count = |text: &str| tokenizer.encode(text).len();
    let mix = Mix {
        mqar: 0.15,
        copy: 0.05,
        relation: 0.60,
        other: 0.20,
        ..Mix::default()
    };
    let draw = |value: Split, conversations: usize, seed: u64| -> Result<Vec<Example>> {
        let mut world = MWorld2::new(&count, mix)?;
        let mut rng = Rng::new(seed);
        collect(
            &mut world,
            &mut rng,
            Cell::new(Split::Train, value),
            conversations,
        )
    };
    let mut train = draw(Split::Train, conversations, seed)?;
    let paraphrases = match args.optional("paraphrases") {
        Some(list) => {
            let (examples, record) = load_paraphrases(&list, &train)?;
            train.extend(examples);
            record
        }
        None => Value::Null,
    };
    let dev = draw(Split::Development, dev_conversations, seed + 1)?;
    let mut splits = serde_json::Map::new();
    for (name, examples) in [("train", &train), ("dev", &dev)] {
        let (mut tokens, mut mask) = (Vec::new(), Vec::new());
        let (mut documents, mut skipped) = (0usize, 0usize);
        let mut by_op: BTreeMap<String, usize> = BTreeMap::new();
        for example in examples.iter() {
            let Some(op) = op_text(example) else {
                skipped += 1;
                continue;
            };
            let messages = [
                Message {
                    role: "system",
                    content: COMPILE_PROMPT,
                },
                Message {
                    role: "user",
                    content: &example.text,
                },
                Message {
                    role: "assistant",
                    content: &op,
                },
            ];
            let encoded = encoder.encode_document(&messages);
            if encoded.emitted_turns != 3 || encoded.special_token_occurrences != 0 {
                skipped += 1;
                continue;
            }
            if encoded.tokens.len() > CONTEXT {
                return Err(invalid("a compile document is longer than the context"));
            }
            for &id in &encoded.tokens {
                tokens.push(u16::try_from(id).map_err(|_| invalid("a token ID exceeds u16"))?);
            }
            mask.extend(&encoded.response_mask);
            documents += 1;
            *by_op
                .entry(op.split_whitespace().nth(1).unwrap_or("?").to_owned())
                .or_default() += 1;
        }
        let directory = out.join(name);
        fs::create_dir_all(&directory)?;
        let mut writer = CorpusWriter::create(&directory.join("tokens.u16"), vocab)
            .map_err(|e| invalid(format!("token store: {e}")))?;
        writer
            .write_tokens(&tokens)
            .map_err(|e| invalid(format!("token store: {e}")))?;
        writer
            .finish()
            .map_err(|e| invalid(format!("token store: {e}")))?;
        fs::write(directory.join("response_mask.u8"), &mask)?;
        let manifest = json!({
            "schema": "uor-r4.m-world-compile-corpus/1",
            "files": [{
                "label": format!("m-world-v2.compile.{name}"),
                "tokens": tokens.len(),
                "special_token_occurrences": 0,
            }],
            "drops": {"special_token_occurrences": 0},
            "documents": documents,
            "skipped_without_a_recoverable_op": skipped,
            "documents_by_op": by_op,
        });
        fs::write(
            directory.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest)?,
        )?;
        println!(
            "{name}: {documents} documents, {} tokens, {skipped} skipped",
            tokens.len()
        );
        splits.insert(name.to_owned(), manifest);
    }
    let report = json!({
        "schema": "uor-r4.m-world-compile-corpus-report/1",
        "executable_sha256": sha256_file(&std::env::current_exe()?)?,
        "world_digest": MWorld2::digest(),
        "tokenizer_sha256": sha256_file(&tokenizer_path)?,
        "prompt": COMPILE_PROMPT,
        "op_format": "Op: none | Op: query <relation> | Op: assert <relation> <value> | Op: update <relation> <value>",
        "train_draw": format!("relation-heavy mix, training phrasings x training values, {conversations} conversations, seed {seed}"),
        "dev_draw": format!("relation-heavy mix, training phrasings x development values, {dev_conversations} conversations, seed {}", seed + 1),
        "paraphrases": paraphrases,
        "splits": splits,
    });
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    Ok(())
}

fn run_v2_extras(mode: &str, rest: &[String]) -> Option<Result<()>> {
    let cells: &[&str] = &[
        "out",
        "world",
        "model",
        "tokenizer",
        "conversations",
        "seed",
        "max_new_tokens",
        "teacher_forced",
        "select",
        "pointer_select",
        "pointer_route",
        "mqar_share",
        "copy_share",
        "relation_share",
        "other_share",
    ];
    let baseline: &[&str] = &[
        "out",
        "world",
        "tokenizer",
        "split",
        "conversations",
        "seed",
        "cells",
        "mqar_share",
        "copy_share",
        "relation_share",
        "other_share",
    ];
    let evaluate_probe: &[&str] = &[
        "out",
        "model",
        "tokenizer",
        "max_new_tokens",
        "select",
        "pointer_select",
        "pointer_route",
    ];
    let static_probe: &[&str] = &["out", "tokenizer", "context"];
    let relation_compiler: &[&str] = &[
        "out",
        "world",
        "model",
        "tokenizer",
        "train_conversations",
        "eval_conversations",
        "seed",
        "steps",
        "rate",
        "l2",
        "lexical",
        "paraphrases",
        "select",
        "pointer_select",
        "pointer_route",
    ];
    let compile_corpus_keys: &[&str] = &[
        "out",
        "world",
        "tokenizer",
        "conversations",
        "dev_conversations",
        "seed",
        "paraphrases",
    ];
    let compiler_save_keys: &[&str] = &[
        "out",
        "world",
        "tokenizer",
        "train_conversations",
        "eval_conversations",
        "seed",
        "values",
        "paraphrases",
        "act_rule",
        "relation_mode",
        "trunk",
    ];
    let session_keys: &[&str] = &[
        "out",
        "world",
        "model_root",
        "tokenizer",
        "compiler",
        "conversations",
        "seed",
        "max_new_tokens",
        "reload",
        "arms",
        "trunk",
        "op_policy",
    ];
    match mode {
        "compiler" => Some(claimed(rest, relation_compiler, compiler)),
        "compile-corpus" => Some(claimed(rest, compile_corpus_keys, compile_corpus)),
        "compiler-save" => Some(claimed(rest, compiler_save_keys, compiler_save)),
        "session" => Some(claimed(rest, session_keys, session)),
        "evaluate-cells" => Some(claimed(rest, cells, evaluate_cells)),
        "baselines" => Some(claimed(rest, baseline, baselines)),
        "route" => Some(claimed(rest, baseline, route)),
        "probe" => Some(claimed(rest, evaluate_probe, probe_evaluate)),
        "probe-static" => Some(claimed(rest, static_probe, probe_static)),
        _ => None,
    }
}

fn claimed(rest: &[String], allowed: &[&str], run: fn(&Args, &Path) -> Result<()>) -> Result<()> {
    let args = Args::parse(rest, allowed)?;
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

fn main() -> Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let (mode, rest) = arguments
        .split_first()
        .ok_or_else(|| invalid("usage: m-world corpus|evaluate|rejudge key=value..."))?;
    if let Some(result) = run_v2_extras(mode, rest) {
        return result;
    }
    let args = match mode.as_str() {
        "corpus" => Args::parse(
            rest,
            &[
                "out",
                "world",
                "tokenizer",
                "chat",
                "exclude",
                "conversations",
                "seed",
                "mqar_share",
                "copy_share",
                "relation_share",
                "other_share",
                "recall",
                "recall_at",
                "context",
                "protocol",
            ],
        )?,
        "evaluate" => Args::parse(
            rest,
            &[
                "out",
                "world",
                "model",
                "tokenizer",
                "split",
                "conversations",
                "seed",
                "max_new_tokens",
                "panel",
                "mqar_share",
                "copy_share",
                "relation_share",
                "other_share",
                "select",
                "pointer_select",
                "pointer_route",
                "recall",
                "recall_at",
                "route_paraphrases",
                "route_acts",
                "route_trunk",
                "protocol",
            ],
        )?,
        "rejudge" => Args::parse(rest, &["out", "report", "tokenizer"])?,
        other => return Err(invalid(format!("unknown mode {other}"))),
    };
    // `recall=` and `recall_at=` are checked before the root is claimed: a
    // malformed value, or one given to world=v1, claims nothing. So is
    // `protocol=`, which only world=v2 encodes, and never with chat=.
    if mode == "corpus" || mode == "evaluate" {
        let version = protocol_version_of(&args)?;
        if version != 1 && world_of(&args)? != World::V2 {
            return Err(invalid("protocol=2 needs world=v2"));
        }
        if version != 1 && args.optional("chat").is_some() {
            return Err(invalid(
                "chat= documents are prepared in protocol 1; a protocol=2 corpus is M-world alone",
            ));
        }
    }
    if mode == "corpus" {
        let recall = recall_of(&args)?;
        recall_at_of(&args)?;
        let v1 = world_of(&args)? == World::V1;
        if v1
            && (recall != Recall::Off
                || args.optional("recall_at").is_some()
                || args.optional("context").is_some())
        {
            return Err(invalid("recall=, recall_at= and context= need world=v2"));
        }
        if recall == Recall::Off && args.optional("recall_at").is_some() {
            return Err(invalid("recall_at= needs recall=oracle or recall=sieve"));
        }
        if recall == Recall::Route {
            return Err(invalid(
                "a corpus takes recall=oracle or recall=sieve; route is for evaluate",
            ));
        }
        if args.number("context", CONTEXT)? < CONTEXT {
            return Err(invalid(format!("context must be at least {CONTEXT}")));
        }
    }
    if mode == "evaluate" {
        let recall = recall_of(&args)?;
        recall_at_of(&args)?;
        if recall != Recall::Off && world_of(&args)? != World::V2 {
            return Err(invalid("recall= needs world=v2"));
        }
        if recall == Recall::Off && args.optional("recall_at").is_some() {
            return Err(invalid("recall_at= needs recall=oracle or recall=sieve"));
        }
        if recall != Recall::Route
            && (args.optional("route_paraphrases").is_some()
                || args.optional("route_acts").is_some())
        {
            return Err(invalid(
                "route_paraphrases=, route_acts= and route_trunk= need recall=route",
            ));
        }
        route_acts_of(&args)?;
    }
    let world = if mode == "rejudge" {
        World::V1
    } else {
        world_of(&args)?
    };
    let out = PathBuf::from(args.required("out")?);
    report_output::claim(&out)?;
    let result = match (mode.as_str(), world) {
        ("corpus", World::V1) => corpus(&args, &out),
        ("corpus", World::V2) => corpus_v2(&args, &out),
        ("rejudge", _) => rejudge(&args, &out),
        (_, World::V1) => evaluate(&args, &out),
        (_, World::V2) => evaluate_v2(&args, &out),
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

#[cfg(test)]
mod tests {
    use super::*;

    const MODEL_KEYS: &[&str] = &[
        "out",
        "model",
        "tokenizer",
        "select",
        "pointer_select",
        "pointer_route",
    ];

    fn parse(arguments: &[&str], allowed: &[&str]) -> Result<Args> {
        let owned: Vec<String> = arguments.iter().map(|a| (*a).to_owned()).collect();
        Args::parse(&owned, allowed)
    }

    /// `select=` and `pointer_select=` are read with the other arguments, which
    /// every mode parses before it claims its report root: a malformed one
    /// fails here and costs no root.
    #[test]
    fn a_selection_is_parsed_and_validated_with_the_arguments() {
        let args = parse(
            &["out=r", "select=flock:16:4", "pointer_select=top:3"],
            MODEL_KEYS,
        )
        .expect("well-formed selections");
        let (text, parsed) = args.selection.select.as_ref().expect("a select");
        assert_eq!(text, "flock:16:4");
        assert_eq!(parsed.map(|s| (s.sink, s.window, s.k)), Some((0, 16, 4)));
        let (text, parsed) = args
            .selection
            .pointer_select
            .as_ref()
            .expect("a pointer select");
        assert_eq!(text, "top:3");
        assert_eq!(*parsed, Some(PointerSelect::TopK(3)));
        // `none` is a selection (it clears the saved one); no argument is none.
        let none = parse(&["out=r", "select=none"], MODEL_KEYS).expect("none parses");
        let (text, parsed) = none.selection.select.as_ref().expect("a select");
        assert_eq!((text.as_str(), parsed.is_none()), ("none", true));
        let absent = parse(&["out=r"], MODEL_KEYS).expect("no selection");
        assert!(absent.selection.select.is_none() && absent.selection.pointer_select.is_none());
        // A malformed one is refused with the arguments.
        for bad in [
            "select=flock:0:4",
            "select=flock:16:0",
            "select=flock:16",
            "select=top:3",
            "pointer_select=top:0",
            "pointer_select=flock:x:2",
            "pointer_select=sideways",
            "pointer_route=prime:0",
            "pointer_route=prime:9",
            "pointer_route=cosine:2",
        ] {
            assert!(parse(&["out=r", bad], MODEL_KEYS).is_err(), "{bad}");
        }
        let routed = parse(&["out=r", "pointer_route=prime:4"], MODEL_KEYS).expect("a route");
        let (text, parsed) = routed.selection.pointer_route.as_ref().expect("a route");
        assert_eq!(text, "prime:4");
        assert_eq!(*parsed, Some(PrimeRoute::exact(4)));
        // A mode that does not take them refuses them as unknown arguments.
        assert!(parse(&["out=r", "select=none"], &["out", "tokenizer"]).is_err());
    }

    #[test]
    fn the_protocol_version_is_one_or_two() {
        let keys = ["out", "protocol"];
        let version =
            |given: &[&str]| parse(given, &keys).and_then(|args| protocol_version_of(&args));
        assert_eq!(version(&["out=r"]).expect("default"), 1);
        assert_eq!(version(&["out=r", "protocol=1"]).expect("one"), 1);
        assert_eq!(version(&["out=r", "protocol=2"]).expect("two"), 2);
        for bad in ["protocol=3", "protocol=v2", "protocol="] {
            assert!(version(&["out=r", bad]).is_err(), "{bad}");
        }
    }

    #[test]
    fn only_the_reference_row_of_the_baselines_report_carries_a_gate() {
        let run = RuleRun::default();
        let rule = baseline_row("R-recency", &run);
        assert!(rule.get("a1_gate").is_none() && rule.get("a1_gate_rule").is_none());
        assert_eq!(rule["freeze_limit"], json!(0.6));
        // An empty card has no item below the limit.
        assert_eq!(rule["below_freeze_limit"], json!(false));
        assert!(rule.get("by_category").is_some() && rule.get("mqar").is_some());
        let reference = baseline_row(REFERENCE, &run);
        assert_eq!(reference["a1_gate"], json!(false));
        assert!(reference.get("a1_gate_rule").is_some());
        assert!(reference.get("below_freeze_limit").is_none());
    }

    #[test]
    fn user_turns_are_read_off_a_decoded_document() {
        let text = "User: Hello there.\nAssistant: Hi!\nUser: Two\nlines here.\n\
                    System: be brief\nAssistant: ok";
        assert_eq!(
            user_turns_of(text),
            vec!["Hello there.".to_owned(), "Two\nlines here.".to_owned()]
        );
        assert!(user_turns_of("Assistant: only me").is_empty());
        assert!(user_turns_of("").is_empty());
        assert_eq!(
            user_turns_of("  User: indented"),
            vec!["indented".to_owned()]
        );
    }

    #[test]
    fn a_source_count_is_reduced_by_exactly_what_was_dropped() {
        let mut object = serde_json::Map::new();
        object.insert("tokens".into(), json!(10));
        reduce_count(&mut object, "tokens", 4).expect("reduces");
        assert_eq!(object["tokens"], json!(6));
        reduce_count(&mut object, "rows_used", 1).expect("a missing key is left out");
        assert!(!object.contains_key("rows_used"));
        assert!(reduce_count(&mut object, "tokens", 7).is_err());
        assert_eq!(object["tokens"], json!(6));
    }

    /// A document as the literal-role protocol lays it out, with characters as
    /// tokens (BOS 1, EOS 2): "User: {user}\nAssistant: {assistant}", the
    /// response mask over the assistant text and its EOS.
    fn document(user: &str, assistant: &str) -> (Vec<u16>, Vec<u8>) {
        let mut tokens = vec![1u16];
        let mut mask = vec![0u8];
        for c in format!("User: {user}\nAssistant: ").chars() {
            tokens.push(c as u16);
            mask.push(0);
        }
        for c in assistant.chars() {
            tokens.push(c as u16);
            mask.push(1);
        }
        tokens.push(2);
        mask.push(1);
        (tokens, mask)
    }

    #[test]
    fn the_probe_screen_drops_whole_documents_and_keeps_every_count_exact() {
        let decode =
            |ids: &[u32]| -> String { ids.iter().filter_map(|&id| char::from_u32(id)).collect() };
        // One eight-word window of the probe's user turns.
        let window: Vec<String> = "the quick brown fox jumps over the lazy"
            .split(' ')
            .map(str::to_owned)
            .collect();
        let grams: BTreeSet<Vec<String>> = [window].into_iter().collect();
        let phrase = "the quick brown fox jumps over the lazy dog";
        let docs = [
            document("Hello there", "Hi"),
            // Its user turn holds the window: dropped whole.
            document(&format!("Tell me: {phrase}"), "No"),
            // Only its assistant turn does: the rule reads user turns alone.
            document("Say hello", phrase),
            document("Bye", "See you"),
        ];
        let mut tokens: Vec<u16> = Vec::new();
        let mut mask: Vec<u8> = Vec::new();
        let mut sizes: Vec<(usize, usize)> = Vec::new();
        for (t, m) in &docs {
            sizes.push((t.len(), m.iter().filter(|&&x| x == 1).count()));
            tokens.extend(t);
            mask.extend(m);
        }
        // Two sources: the first two documents, then the last two.
        let source = |label: &str, range: std::ops::Range<usize>| -> Value {
            let part = &sizes[range];
            json!({
                "label": label,
                "tokens": part.iter().map(|s| s.0).sum::<usize>(),
                "response_tokens": part.iter().map(|s| s.1).sum::<usize>(),
                "rows_used": part.len(),
                "special_token_occurrences": 0,
            })
        };
        let files = vec![source("a", 0..2), source("b", 2..4)];
        let screened = screen_documents(&tokens, &mask, files.clone(), (1, 2, 0), &decode, &grams)
            .expect("a screen");
        assert_eq!((screened.documents, screened.excluded_documents), (4, 1));
        assert_eq!(screened.excluded_tokens, sizes[1].0);
        // The kept store is documents 0, 2 and 3, in order.
        let mut kept_tokens: Vec<u16> = Vec::new();
        let mut kept_mask: Vec<u8> = Vec::new();
        for index in [0usize, 2, 3] {
            kept_tokens.extend(&docs[index].0);
            kept_mask.extend(&docs[index].1);
        }
        assert_eq!(screened.tokens, kept_tokens);
        assert_eq!(screened.mask, kept_mask);
        // The first source lost exactly that document; the second is untouched.
        let first = &screened.files[0];
        assert_eq!(first["tokens"], json!(sizes[0].0));
        assert_eq!(first["response_tokens"], json!(sizes[0].1));
        assert_eq!(first["rows_used"], json!(1));
        assert_eq!(first["probe_excluded_documents"], json!(1));
        assert_eq!(screened.files[1], files[1]);
        // The sources still cover the kept store exactly, as the split loader requires.
        let covered: u64 = screened
            .files
            .iter()
            .map(|f| f["tokens"].as_u64().unwrap_or(0))
            .sum();
        assert_eq!(covered as usize, screened.tokens.len());
        assert_eq!(screened.record["excluded_documents"], json!(1));
        assert_eq!(
            screened.record["by_source"][0]["excluded_documents"],
            json!(1)
        );
        assert_eq!(
            screened.record["by_source"][1]["excluded_documents"],
            json!(0)
        );
        // With nothing to match, the store and the sources come back unchanged.
        let clean = screen_documents(
            &tokens,
            &mask,
            files.clone(),
            (1, 2, 0),
            &decode,
            &BTreeSet::new(),
        )
        .expect("a screen");
        assert_eq!(clean.excluded_documents, 0);
        assert_eq!(clean.tokens, tokens);
        assert_eq!(clean.mask, mask);
        assert_eq!(clean.files, files);
        // A source the screen would empty is refused before anything is written.
        let emptied = vec![source("a", 0..1), source("b", 1..2), source("c", 2..4)];
        assert!(screen_documents(&tokens, &mask, emptied, (1, 2, 0), &decode, &grams).is_err());
        // Sources that do not cover the store, and a store that does not begin
        // with a document, are refused.
        let mut short = files.clone();
        short[1]["tokens"] = json!(1);
        assert!(screen_documents(&tokens, &mask, short, (1, 2, 0), &decode, &grams).is_err());
        let mut headless = tokens.clone();
        headless[0] = u16::from(b'x');
        assert!(screen_documents(&headless, &mask, files, (1, 2, 0), &decode, &grams).is_err());
    }
}
