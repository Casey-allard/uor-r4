//! Lawful M-world v2 training-corpus writer for the oracle-memory comparison.
//!
//! The historical `examples/m-world.rs` remains source evidence and is not a
//! Cargo target after D29. This writer imports only the admitted M-world and
//! tokenizer modules. It emits the same prepared-store shape as
//! `m-world corpus world=v2`, without the historical evaluator, compiler or
//! model code:
//!
//! ```text
//! m-world-recall-corpus out=NEW_ROOT tokenizer=TOKENIZER.json \
//!   [world=v2|v2c] [conversations=5600] [seed=1] \
//!   [mqar_share=0.35] [copy_share=0.10] [relation_share=0.30] \
//!   [other_share=0.25] [closed_share=0.20] [recall=off|oracle] \
//!   [recall_at=reply|query] [context=384] [protocol=1|2] [exclude=PANEL.json] \
//!   [chain=0]
//! ```
//!
//! `chain=TOKENS` (at least twice the episode context) writes long chained
//! conversations of at most that many tokens under the chosen protocol
//! (`MWorld2::chained`) instead of single episodes. It needs `context=` of at
//! least the budget and `recall=off`, and the manifest gains a `chain`
//! census. The default `chain=0` writes today's corpus, byte for byte.
//!
//! `out/train/` is a `uor-r4-chat-corpus/v1` split. It can be passed directly
//! to `train-in-space` with its token and mask SHA-256 values, or composed with
//! the pinned chat corpus through `mix-chat-corpus`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use uor_r4_core::native_geometric::mmap_corpus::CorpusWriter;
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::{DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::milestone_world::{normalized, Split};
use uor_r4_training::milestone_world_v2::{
    oracle_recall, recall_line, takes_recall, Cell, ChainRedraws, Conversation2, MWorld2, Meter,
    Mix, Turn2, Variant, CHAIN_QUERIES, CONTEXT, REVISION,
};

type Result<T> = std::result::Result<T, String>;

const MASK_SCHEMA: &str = "uor-r4-response-mask/u8/v1";
const MASK_RULE: &str = "1 = each token of an assistant turn's response content and its terminating <|eos|>; 0 = <|bos|>, all role markers, turn separators, system/user content, and an unmasked document-terminal <|eos|>.";
const TEMPLATE_RULE: &str = "<|bos|> then turns joined by a single '\\n' separator (placed before every turn after the first); a turn is '<marker><content>' with markers 'System: ', 'User: ', 'Assistant: '; every assistant turn ends with <|eos|>; a document-terminal <|eos|> is appended only when the final emitted turn is not assistant. Content is \\r\\n/\\r-normalised and trimmed; interior whitespace preserved.";

const ALLOWED: &[&str] = &[
    "out",
    "tokenizer",
    "world",
    "conversations",
    "seed",
    "mqar_share",
    "copy_share",
    "relation_share",
    "other_share",
    "closed_share",
    "recall",
    "recall_at",
    "context",
    "protocol",
    "exclude",
    "chain",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Recall {
    Off,
    Oracle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecallAt {
    Reply,
    Query,
}

#[derive(Clone)]
struct Options {
    out: PathBuf,
    tokenizer: PathBuf,
    world: Variant,
    conversations: usize,
    seed: u64,
    mix: Mix,
    recall: Recall,
    recall_at: RecallAt,
    context: usize,
    protocol: u8,
    exclude: Option<PathBuf>,
    /// The chained conversations' token budget; 0 writes single episodes.
    chain: usize,
}

impl Options {
    fn parse(arguments: &[String]) -> Result<Self> {
        let mut pairs = BTreeMap::new();
        for argument in arguments {
            let (key, value) = argument
                .split_once('=')
                .ok_or_else(|| format!("expected key=value, found {argument:?}"))?;
            if !ALLOWED.contains(&key) {
                return Err(format!("unknown argument {key}="));
            }
            if pairs.insert(key.to_owned(), value.to_owned()).is_some() {
                return Err(format!("duplicate argument {key}="));
            }
        }
        let required = |key: &str| {
            pairs
                .get(key)
                .cloned()
                .ok_or_else(|| format!("missing {key}="))
        };
        let optional = |key: &str| pairs.get(key).map(String::as_str);
        let number = |key: &str, default: &str| -> Result<u64> {
            optional(key)
                .unwrap_or(default)
                .parse()
                .map_err(|_| format!("{key}= must be a nonnegative integer"))
        };
        let share = |key: &str, default: &str| -> Result<f64> {
            let value = optional(key)
                .unwrap_or(default)
                .parse::<f64>()
                .map_err(|_| format!("{key}= must be a number"))?;
            if !value.is_finite() || value < 0.0 || value > 1.0 {
                return Err(format!("{key}= must be in [0, 1]"));
            }
            Ok(value)
        };
        let world = match optional("world") {
            None | Some("v2") => Variant::V2,
            Some("v2c") => Variant::V2c,
            Some(other) => return Err(format!("unknown world={other}: v2 or v2c")),
        };
        let recall = match optional("recall") {
            None | Some("off") => Recall::Off,
            Some("oracle") => Recall::Oracle,
            Some(other) => return Err(format!("unknown recall={other}: off or oracle")),
        };
        let recall_at = match optional("recall_at") {
            None | Some("reply") => RecallAt::Reply,
            Some("query") => RecallAt::Query,
            Some(other) => return Err(format!("unknown recall_at={other}: reply or query")),
        };
        if recall == Recall::Off && optional("recall_at").is_some() {
            return Err("recall_at= needs recall=oracle".to_owned());
        }
        let protocol = number("protocol", "2")?;
        if !(1..=2).contains(&protocol) {
            return Err("protocol= must be 1 or 2".to_owned());
        }
        let context = usize::try_from(number("context", "384")?)
            .map_err(|_| "context= does not fit this machine".to_owned())?;
        if context < CONTEXT {
            return Err(format!("context= must be at least {CONTEXT}"));
        }
        let chain = usize::try_from(number("chain", "0")?)
            .map_err(|_| "chain= does not fit this machine".to_owned())?;
        if chain != 0 {
            if chain < 2 * CONTEXT {
                return Err(format!("chain= must be 0 or at least {}", 2 * CONTEXT));
            }
            if context < chain {
                return Err(format!("context= must be at least chain= ({chain})"));
            }
            if recall != Recall::Off {
                return Err("chain= needs recall=off".to_owned());
            }
        }
        let conversations = usize::try_from(number("conversations", "5600")?)
            .map_err(|_| "conversations= does not fit this machine".to_owned())?;
        if conversations == 0 {
            return Err("conversations= must be positive".to_owned());
        }
        let mix = Mix {
            mqar: share("mqar_share", "0.35")?,
            copy: share("copy_share", "0.10")?,
            relation: share("relation_share", "0.30")?,
            other: share("other_share", "0.25")?,
            closed: share("closed_share", "0.20")?,
        };
        if (mix.mqar + mix.copy + mix.relation + mix.other - 1.0).abs() > 1e-9 {
            return Err(
                "mqar_share, copy_share, relation_share and other_share must sum to 1".into(),
            );
        }
        Ok(Self {
            out: PathBuf::from(required("out")?),
            tokenizer: PathBuf::from(required("tokenizer")?),
            world,
            conversations,
            seed: number("seed", "1")?,
            mix,
            recall,
            recall_at,
            context,
            protocol: protocol as u8,
            exclude: optional("exclude").map(PathBuf::from),
            chain,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Receipt {
    conversations: usize,
    turns: usize,
    tokens: usize,
    response_tokens: usize,
    recall_lines: usize,
    recall_value_lines: usize,
    recall_none_lines: usize,
    longest: usize,
}

fn load_tokenizer(path: &Path) -> Result<(ByteBpeTokenizer, String)> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let sha256 = uor_r4_training::sha256_bytes(&bytes);
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| "tokenizer JSON is not a supported byte-level BPE".to_owned())?;
    Ok((tokenizer, sha256))
}

fn load_exclusions(path: &Path) -> Result<(BTreeSet<String>, String)> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let sha256 = uor_r4_training::sha256_bytes(&bytes);
    let rows: Value =
        serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    let rows = rows
        .as_array()
        .ok_or_else(|| format!("{}: panel must be a JSON array", path.display()))?;
    let mut turns = BTreeSet::new();
    for row in rows {
        let user_turns = row["user_turns"]
            .as_array()
            .ok_or_else(|| format!("{}: a row has no user_turns array", path.display()))?;
        for turn in user_turns {
            let turn = turn
                .as_str()
                .ok_or_else(|| format!("{}: a user turn is not a string", path.display()))?;
            turns.insert(normalized(turn));
        }
    }
    Ok((turns, sha256))
}

fn messages<'a>(turns: &'a [Turn2], lines: &'a [Option<String>], at: RecallAt) -> Vec<Message<'a>> {
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

fn lines_for(conversation: &Conversation2, recall: Recall) -> (Vec<Option<String>>, usize, usize) {
    let mut values = 0;
    let mut none = 0;
    let lines = conversation
        .turns
        .iter()
        .map(|turn| {
            if recall == Recall::Off || !takes_recall(turn) {
                return None;
            }
            let value = oracle_recall(turn);
            if value.is_some() {
                values += 1;
            } else {
                none += 1;
            }
            Some(recall_line(value))
        })
        .collect();
    (lines, values, none)
}

/// What a chained corpus holds: its documents' lengths and episodes, its
/// notes and queries, how far back each query asks, and how often a user
/// turn repeats an earlier one of the same document.
#[derive(Default)]
struct ChainCensus {
    tokens: Vec<usize>,
    episodes: Vec<usize>,
    notes: usize,
    distances: Vec<usize>,
    user_turns: usize,
    repeated_user_turns: usize,
    metered_mismatches: usize,
}

impl ChainCensus {
    fn add(&mut self, conversation: &Conversation2, encoded: usize) {
        self.tokens.push(encoded);
        self.metered_mismatches += usize::from(encoded != conversation.tokens);
        self.notes += conversation
            .turns
            .iter()
            .filter(|t| t.intent == "chain_note")
            .count();
        let chains: Vec<_> = conversation
            .turns
            .iter()
            .filter_map(|t| t.tag.chain.as_ref())
            .collect();
        self.episodes.extend(chains.first().map(|c| c.episodes));
        self.distances.extend(chains.iter().map(|c| c.distance));
        let mut seen = BTreeSet::new();
        for turn in &conversation.turns {
            self.user_turns += 1;
            self.repeated_user_turns += usize::from(!seen.insert(normalized(&turn.user)));
        }
    }

    fn record(&self, budget: usize, redraws: ChainRedraws) -> Value {
        let spread = |values: &[usize]| {
            json!({
                "min": values.iter().min(),
                "max": values.iter().max(),
                "total": values.iter().sum::<usize>(),
            })
        };
        let past = |floor: usize| self.distances.iter().filter(|&&d| d > floor).count();
        let mut histogram: BTreeMap<String, usize> = BTreeMap::new();
        for &distance in &self.distances {
            *histogram
                .entry(format!("{:05}", distance / 256 * 256))
                .or_default() += 1;
        }
        json!({
            "budget": budget,
            "documents": self.tokens.len(),
            "document_tokens": spread(&self.tokens),
            "episodes": spread(&self.episodes),
            "notes": self.notes,
            "queries": self.distances.len(),
            "queries_max_per_document": CHAIN_QUERIES,
            "query_distance": spread(&self.distances),
            "query_distance_past": {"192": past(192), "512": past(512), "1024": past(1_024)},
            "query_distance_by_256": histogram,
            "user_turns": self.user_turns,
            "repeated_user_turns": self.repeated_user_turns,
            "metered_mismatches": self.metered_mismatches,
            "redraws": redraws,
            "rule": "MWorld2::chained: episodes back to back, a note before the first and before about one in three of the rest, then queries for up to four notes; distances in the generator's meter of the chosen protocol",
        })
    }
}

fn generate(options: &Options) -> Result<Receipt> {
    let (tokenizer, tokenizer_sha256) = load_tokenizer(&options.tokenizer)?;
    let protocol = DialogueProtocol::literal_roles_version(&tokenizer, options.protocol)
        .map_err(|e| format!("protocol: {e}"))?;
    let encoder = protocol
        .bind(&tokenizer)
        .map_err(|e| format!("protocol: {e}"))?;
    let count = |text: &str| tokenizer.encode(text).len();
    let mut world = MWorld2::new(&count, options.mix)
        .map_err(|e| e.to_string())?
        .with_variant(options.world);
    if options.chain != 0 && options.protocol == 2 {
        world = world.with_spaced_meter();
    }
    let mut rng = uor_r4_training::milestone_world::Rng::new(options.seed);
    let (excluded, excluded_sha256) = match &options.exclude {
        Some(path) => {
            let (turns, sha256) = load_exclusions(path)?;
            (turns, Some(sha256))
        }
        None => (BTreeSet::new(), None),
    };
    let vocab = u32::try_from(tokenizer.vocab_size())
        .map_err(|_| "tokenizer vocabulary does not fit the token store".to_owned())?;
    let mut token_ids = Vec::new();
    let mut response_mask = Vec::new();
    let mut rejected = 0usize;
    let mut turns = 0usize;
    let mut recall_lines = 0usize;
    let mut recall_value_lines = 0usize;
    let mut recall_none_lines = 0usize;
    let mut longest = 0usize;
    let mut by_kind: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new();
    let mut by_category: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new();
    let episode_meter = if options.protocol == 1 {
        Meter::new(&count)
    } else {
        Meter::spaced(&count)
    };

    let mut census = ChainCensus::default();
    let mut redraws = ChainRedraws::default();

    for index in 0..options.conversations {
        let conversation = if options.chain == 0 {
            world.conversation_excluding(&mut rng, Split::Train, &excluded, &mut rejected)
        } else {
            world.chained(
                &mut rng,
                Cell::same(Split::Train),
                options.chain,
                &excluded,
                &mut redraws,
            )
        }
        .map_err(|e| e.to_string())?;
        let (lines, values, none) = lines_for(&conversation, options.recall);
        recall_lines += lines.iter().flatten().count();
        recall_value_lines += values;
        recall_none_lines += none;
        let encoded =
            encoder.encode_document(&messages(&conversation.turns, &lines, options.recall_at));
        if encoded.emitted_turns != conversation.turns.len() * 2 + lines.iter().flatten().count()
            || encoded.special_token_occurrences != 0
        {
            return Err(format!("M-world v2 conversation {index} did not encode"));
        }
        if encoded.tokens.len() > options.context {
            return Err(format!(
                "M-world v2 conversation {index} has {} tokens, over the {}-token context",
                encoded.tokens.len(),
                options.context
            ));
        }
        for &id in &encoded.tokens {
            if id >= vocab {
                return Err("an M-world token is outside the vocabulary".to_owned());
            }
            token_ids.push(id as u16);
        }
        let response = encoded.response_mask.iter().filter(|&&m| m == 1).count();
        response_mask.extend(&encoded.response_mask);
        longest = longest.max(encoded.tokens.len());
        if options.chain != 0 {
            census.add(&conversation, encoded.tokens.len());
        }
        let kind = by_kind
            .entry(format!("{:?}", conversation.kind))
            .or_default();
        *kind = (kind.0 + 1, kind.1 + encoded.tokens.len(), kind.2 + response);
        turns += conversation.turns.len();
        for (turn_index, turn) in conversation.turns.iter().enumerate() {
            let category = by_category
                .entry(format!("{:?}", turn.category))
                .or_default();
            *category = (
                category.0 + 1,
                category.1 + episode_meter.exchange(turn_index, &turn.user, &turn.reply),
                category.2 + episode_meter.text(&turn.reply) + 1,
            );
        }
    }

    // A chained corpus counts its excluded episodes with its other redraws. An
    // excluded note or query drops its whole draw, so it counts under the
    // census's `redraws.dropped`, not here.
    rejected += redraws.excluded;

    if options.recall == Recall::Oracle && recall_value_lines == 0 {
        return Err("recall=oracle emitted no value-bearing memory lines".to_owned());
    }

    let train = options.out.join("train");
    fs::create_dir_all(&train).map_err(|e| e.to_string())?;
    let tokens_path = train.join("tokens.u16");
    let mut writer =
        CorpusWriter::create(&tokens_path, vocab).map_err(|e| format!("token store: {e}"))?;
    writer
        .write_tokens(&token_ids)
        .map_err(|e| format!("token store: {e}"))?;
    let written = writer.finish().map_err(|e| format!("token store: {e}"))?;
    let mask_path = train.join("response_mask.u8");
    fs::write(&mask_path, &response_mask).map_err(|e| e.to_string())?;
    let response_tokens = response_mask.iter().filter(|&&mask| mask == 1).count();

    let table = |cells: &BTreeMap<String, (usize, usize, usize)>, unit: &str| {
        json!(cells
            .iter()
            .map(|(key, value)| (
                key,
                json!({unit: value.0, "tokens": value.1, "response_tokens": value.2})
            ))
            .collect::<BTreeMap<_, _>>())
    };
    let recall = (options.recall == Recall::Oracle).then(|| {
        json!({
            "mode": "oracle",
            "at": match options.recall_at {
                RecallAt::Reply => "reply",
                RecallAt::Query => "query",
            },
            "lines": recall_lines,
            "value_lines": recall_value_lines,
            "none_lines": recall_none_lines,
            "context": options.context,
            "rule": "one system turn per MQAR and relation query, \"Memory: {value}.\" or \"Memory: none.\"; the plain episode is the one MWorld2 meters",
        })
    });
    let excluded_record = options.exclude.as_ref().map(|path| {
        json!({
            "path": path,
            "sha256": excluded_sha256,
            "turns": excluded.len(),
            "rejected_draws": rejected,
        })
    });
    let mut manifest = json!({
        "schema": "uor-r4-chat-corpus/v1",
        "mask_schema": MASK_SCHEMA,
        "split": "train",
        "mask_rule": MASK_RULE,
        "template_rule": TEMPLATE_RULE,
        "dialogue_protocol": protocol.schema,
        "tokenizer": {
            "bos_id": protocol.bos_id,
            "eos_id": protocol.eos_id,
            "unk_id": protocol.unk_id,
            "path": options.tokenizer,
            "sha256": tokenizer_sha256,
            "tokenizer_cid": protocol.tokenizer_cid,
            "vocab_size": tokenizer.vocab_size(),
        },
        "max_tokens": options.context,
        "inputs": [{"label": "m-world-v2.train", "path": "generated"}],
        "files": [{
            "label": "m-world-v2.train",
            "path": "generated: uor_r4_training::milestone_world_v2 (Split::Train)",
            "rows_total": options.conversations,
            "rows_used": options.conversations,
            "tokens": token_ids.len(),
            "response_tokens": response_tokens,
            "special_token_occurrences": 0,
        }],
        "drops": {
            "messages_skipped_empty": 0,
            "messages_skipped_unknown_role": 0,
            "rows_dropped_empty": 0,
            "rows_dropped_malformed": 0,
            "rows_dropped_no_messages": 0,
            "rows_dropped_no_response": 0,
            "rows_dropped_oversized": 0,
            "special_token_occurrences": 0,
        },
        "rows_used": options.conversations,
        "tokens": written,
        "response_tokens": response_tokens,
        "tokens_bytes": fs::metadata(&tokens_path).map_err(|e| e.to_string())?.len(),
        "mask_bytes": response_mask.len(),
        "tokens_sha256": uor_r4_training::sha256_file(&tokens_path).map_err(|e| e.to_string())?,
        "mask_sha256": uor_r4_training::sha256_file(&mask_path).map_err(|e| e.to_string())?,
        "composition": {
            "m_world": {
                "version": options.world.world_name(),
                "revision": REVISION,
                "world_digest": MWorld2::digest_for(options.world),
                "split": "train",
                "seed": options.seed,
                "mix": options.mix,
                "protocol": options.protocol,
                "conversations": options.conversations,
                "turns": turns,
                "tokens": token_ids.len(),
                "response_tokens": response_tokens,
                "episode_tokens_max": longest,
                "episodes_per_kind": table(&by_kind, "episodes"),
                "turns_per_category": table(&by_category, "turns"),
                "recall": recall,
                "exclude": excluded_record,
            }
        }
    });
    // Only a chained corpus has the key, so a default manifest is unchanged.
    if options.chain != 0 {
        manifest["composition"]["m_world"]["chain"] = census.record(options.chain, redraws);
    }
    let manifest_path = train.join("manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        options.out.join("corpus.json"),
        serde_json::to_vec_pretty(&json!({
            "schema": "uor-r4.m-world-recall-corpus/1",
            "executable_sha256": uor_r4_training::sha256_file(
                &std::env::current_exe().map_err(|e| e.to_string())?
            ).ok(),
            "train_manifest_sha256": uor_r4_training::sha256_file(&manifest_path)
                .map_err(|e| e.to_string())?,
            "receipt": {
                "conversations": options.conversations,
                "turns": turns,
                "tokens": written,
                "response_tokens": response_tokens,
                "recall_lines": recall_lines,
                "recall_value_lines": recall_value_lines,
                "recall_none_lines": recall_none_lines,
                "episode_tokens_max": longest,
            },
            "lawful_scope": "imports the admitted M-world v2, tokenizer and UORT writer only; no evaluator, compiler, model, probability operation or dense contraction",
        }))
        .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    Ok(Receipt {
        conversations: options.conversations,
        turns,
        tokens: written as usize,
        response_tokens,
        recall_lines,
        recall_value_lines,
        recall_none_lines,
        longest,
    })
}

fn execute(options: &Options) -> Result<Receipt> {
    report_output::claim(&options.out).map_err(|e| e.to_string())?;
    let result = generate(options);
    if let Err(error) = &result {
        fs::write(
            options.out.join("error.json"),
            serde_json::to_vec_pretty(&json!({"error": error})).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }
    report_output::seal(&options.out).map_err(|e| e.to_string())?;
    report_output::verify(&options.out).map_err(|e| e.to_string())?;
    result
}

fn run() -> Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let options = Options::parse(&arguments)?;
    let receipt = execute(&options)?;
    println!(
        "{} conversations, {} turns, {} tokens, {} response tokens; recall lines {} ({} values, {} none); longest {}",
        receipt.conversations,
        receipt.turns,
        receipt.tokens,
        receipt.response_tokens,
        receipt.recall_lines,
        receipt.recall_value_lines,
        receipt.recall_none_lines,
        receipt.longest
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;
    use uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader;

    const PINNED_TOKENIZER_SHA256: &str =
        "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89";

    fn byte_tokenizer_json() -> Value {
        let mut printable: Vec<u32> = (u32::from(b'!')..=u32::from(b'~')).collect();
        printable.extend(0xA1..=0xAC);
        printable.extend(0xAE..=0xFF);
        let mut vocab = serde_json::Map::new();
        let specials = ["<|bos|>", "<|eos|>", "<|unk|>"];
        for (id, surface) in specials.iter().enumerate() {
            vocab.insert((*surface).to_owned(), json!(id));
        }
        let mut extra = 0;
        for byte in 0u32..256 {
            let ch = if printable.contains(&byte) {
                char::from_u32(byte)
            } else {
                extra += 1;
                char::from_u32(255 + extra)
            }
            .expect("a valid char");
            vocab.insert(ch.to_string(), json!(byte + 3));
        }
        let added: Vec<Value> = specials
            .iter()
            .enumerate()
            .map(|(id, surface)| json!({"id": id, "content": surface}))
            .collect();
        json!({
            "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
            "added_tokens": added,
            "model": {"type": "BPE", "vocab": vocab, "merges": []},
        })
    }

    fn byte_tokenizer() -> ByteBpeTokenizer {
        ByteBpeTokenizer::from_tokenizer_json_bytes(
            serde_json::to_vec(&byte_tokenizer_json())
                .expect("JSON")
                .as_slice(),
        )
        .expect("the byte tokenizer parses")
    }

    fn byte_tokenizer_path() -> PathBuf {
        static PATH: OnceLock<PathBuf> = OnceLock::new();
        PATH.get_or_init(|| {
            let path = std::env::temp_dir().join(format!(
                "m-world-recall-tokenizer-{}.json",
                std::process::id()
            ));
            fs::write(
                &path,
                serde_json::to_vec(&byte_tokenizer_json()).expect("JSON"),
            )
            .expect("tokenizer");
            path
        })
        .clone()
    }

    fn options(root: &Path, recall: Recall) -> Options {
        Options {
            out: root.to_owned(),
            tokenizer: byte_tokenizer_path(),
            world: Variant::V2,
            conversations: 24,
            seed: 73,
            mix: Mix {
                mqar: 0.0,
                copy: 0.0,
                relation: 0.75,
                other: 0.25,
                closed: 0.20,
            },
            recall,
            recall_at: RecallAt::Reply,
            context: 384,
            protocol: 2,
            exclude: None,
            chain: 0,
        }
    }

    fn pinned_options(root: &Path, tokenizer: PathBuf, recall: Recall) -> Options {
        Options {
            out: root.to_owned(),
            tokenizer,
            world: Variant::V2,
            conversations: 128,
            seed: 73,
            mix: Mix::default(),
            recall,
            recall_at: RecallAt::Reply,
            context: 384,
            protocol: 2,
            exclude: None,
            chain: 0,
        }
    }

    fn clean(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "m-world-recall-corpus-{}-{name}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("test root");
        root
    }

    fn unclaimed(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "m-world-recall-corpus-{}-{name}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        root
    }

    fn read_with(root: &Path, tokenizer: &ByteBpeTokenizer) -> (Vec<u8>, Vec<u8>, Vec<u8>, String) {
        let train = root.join("train");
        let token_bytes = fs::read(train.join("tokens.u16")).expect("tokens");
        let mask_bytes = fs::read(train.join("response_mask.u8")).expect("mask");
        let manifest_bytes = fs::read(train.join("manifest.json")).expect("manifest");
        let reader = MmapCorpusReader::open(train.join("tokens.u16")).expect("token store");
        let ids: Vec<u32> = reader.as_slice().iter().map(|&id| u32::from(id)).collect();
        (
            token_bytes,
            mask_bytes,
            manifest_bytes,
            tokenizer.decode(&ids),
        )
    }

    fn read(root: &Path) -> (Vec<u8>, Vec<u8>, Vec<u8>, String) {
        read_with(root, &byte_tokenizer())
    }

    fn table_total(manifest: &Value, table: &str, field: &str) -> u64 {
        manifest["composition"]["m_world"][table]
            .as_object()
            .expect("table")
            .values()
            .map(|row| row[field].as_u64().expect("integer count"))
            .sum()
    }

    #[test]
    fn oracle_lines_are_counted_and_off_is_a_matched_control() {
        let root = clean("oracle");
        let receipt = generate(&options(&root, Recall::Oracle)).expect("oracle corpus");
        assert_eq!(
            receipt,
            Receipt {
                conversations: 24,
                turns: 66,
                tokens: 4_793,
                response_tokens: 1_853,
                recall_lines: 10,
                recall_value_lines: 8,
                recall_none_lines: 2,
                longest: 281,
            }
        );
        assert!(receipt.recall_value_lines > 0);
        assert_eq!(
            receipt.recall_lines,
            receipt.recall_value_lines + receipt.recall_none_lines
        );
        let (_, mask, manifest, text) = read(&root);
        assert_eq!(mask.len(), receipt.tokens);
        assert_eq!(text.matches("Memory:").count(), receipt.recall_lines);
        assert!(text.contains("Memory:"));
        let manifest: Value = serde_json::from_slice(&manifest).expect("JSON");
        assert_eq!(
            manifest["composition"]["m_world"]["recall"]["value_lines"],
            json!(receipt.recall_value_lines)
        );
        assert_eq!(
            table_total(&manifest, "episodes_per_kind", "tokens"),
            receipt.tokens as u64
        );
        assert_eq!(
            table_total(&manifest, "turns_per_category", "response_tokens"),
            receipt.response_tokens as u64
        );
        let _ = fs::remove_dir_all(root);

        let root = clean("off");
        let control = generate(&options(&root, Recall::Off)).expect("control corpus");
        assert_eq!(
            (control.conversations, control.turns),
            (receipt.conversations, receipt.turns)
        );
        let (_, _, manifest, text) = read(&root);
        assert!(!text.contains("Memory:"));
        let manifest: Value = serde_json::from_slice(&manifest).expect("JSON");
        assert!(manifest["composition"]["m_world"]["recall"].is_null());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    #[ignore = "needs the pinned issue-1017 tokenizer JSON"]
    fn pinned_tokenizer_default_mix_is_deterministic_and_teaches_memory() {
        let tokenizer_path = std::env::var_os("UOR_R4_PINNED_TOKENIZER")
            .map(PathBuf::from)
            .expect("set UOR_R4_PINNED_TOKENIZER to the d36d3e87 tokenizer JSON");
        assert_eq!(
            uor_r4_training::sha256_file(&tokenizer_path).expect("pinned tokenizer"),
            PINNED_TOKENIZER_SHA256
        );

        let first_root = clean("pinned-oracle-first");
        let second_root = clean("pinned-oracle-second");
        let first = generate(&pinned_options(
            &first_root,
            tokenizer_path.clone(),
            Recall::Oracle,
        ))
        .expect("first pinned corpus");
        let second = generate(&pinned_options(
            &second_root,
            tokenizer_path.clone(),
            Recall::Oracle,
        ))
        .expect("second pinned corpus");
        assert_eq!(
            first,
            Receipt {
                conversations: 128,
                turns: 446,
                tokens: 16_214,
                response_tokens: 4_903,
                recall_lines: 72,
                recall_value_lines: 64,
                recall_none_lines: 8,
                longest: 256,
            }
        );
        assert_eq!(first, second);
        let first_bytes = read(&first_root);
        let second_bytes = read(&second_root);
        assert_eq!(
            (&first_bytes.0, &first_bytes.1, &first_bytes.2),
            (&second_bytes.0, &second_bytes.1, &second_bytes.2)
        );

        let manifest: Value = serde_json::from_slice(&first_bytes.2).expect("JSON");
        assert_eq!(
            table_total(&manifest, "episodes_per_kind", "tokens"),
            first.tokens as u64
        );
        assert_eq!(
            table_total(&manifest, "turns_per_category", "response_tokens"),
            first.response_tokens as u64
        );
        let train = first_root.join("train");
        let (tokenizer, _) = load_tokenizer(&tokenizer_path).expect("pinned tokenizer");
        let reader = MmapCorpusReader::open(train.join("tokens.u16")).expect("token store");
        let ids: Vec<u32> = reader.as_slice().iter().map(|&id| u32::from(id)).collect();
        let text = tokenizer.decode(&ids);
        assert_eq!(text.matches("Memory:").count(), first.recall_lines);
        assert_eq!(
            text.matches("Memory: none.").count(),
            first.recall_none_lines
        );

        let control_root = clean("pinned-off");
        let control = generate(&pinned_options(
            &control_root,
            tokenizer_path.clone(),
            Recall::Off,
        ))
        .expect("pinned control");
        assert_eq!(
            control,
            Receipt {
                conversations: 128,
                turns: 446,
                tokens: 15_134,
                response_tokens: 4_903,
                recall_lines: 0,
                recall_value_lines: 0,
                recall_none_lines: 0,
                longest: 241,
            }
        );
        let (_, _, control_manifest, control_text) = read_with(&control_root, &tokenizer);
        assert!(!control_text.contains("Memory:"));
        let control_manifest: Value =
            serde_json::from_slice(&control_manifest).expect("control JSON");
        assert!(control_manifest["composition"]["m_world"]["recall"].is_null());

        let _ = fs::remove_dir_all(first_root);
        let _ = fs::remove_dir_all(second_root);
        let _ = fs::remove_dir_all(control_root);
    }

    #[test]
    fn oracle_corpora_are_byte_deterministic() {
        let first_root = clean("oracle-first");
        let second_root = clean("oracle-second");
        generate(&options(&first_root, Recall::Oracle)).expect("first corpus");
        generate(&options(&second_root, Recall::Oracle)).expect("second corpus");
        let first = read(&first_root);
        let second = read(&second_root);
        assert_eq!((first.0, first.1, first.2), (second.0, second.1, second.2));
        let _ = fs::remove_dir_all(first_root);
        let _ = fs::remove_dir_all(second_root);
    }

    #[test]
    fn the_command_claims_seals_and_verifies_its_output_root() {
        let root = unclaimed("sealed");
        let mut args = options(&root, Recall::Off);
        args.out = root.clone();
        execute(&args).expect("sealed corpus");
        report_output::verify(&root).expect("verified root");
        assert!(root.join("manifest.json").is_file());
        assert!(root.join("train/manifest.json").is_file());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn exclusion_panel_identity_is_the_hash_of_its_parsed_bytes() {
        let root = clean("excluded");
        let panel = root.join("exclude.json");
        fs::write(
            &panel,
            serde_json::to_vec(&json!([{
                "id": "excluded-1",
                "category": "memory",
                "user_turns": ["This exact turn is not in the seeded training stream."]
            }]))
            .expect("JSON"),
        )
        .expect("panel");
        let mut args = options(&root, Recall::Off);
        args.exclude = Some(panel.clone());
        generate(&args).expect("excluded corpus");
        let manifest: Value =
            serde_json::from_slice(&fs::read(root.join("train/manifest.json")).expect("manifest"))
                .expect("JSON");
        assert_eq!(
            manifest["composition"]["m_world"]["exclude"]["sha256"],
            json!(uor_r4_training::sha256_file(&panel).expect("panel hash"))
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn arguments_are_strict_and_keep_off_as_the_default() {
        let parsed =
            Options::parse(&["out=test".to_owned(), "tokenizer=tokenizer.json".to_owned()])
                .expect("minimal arguments");
        assert_eq!(parsed.recall, Recall::Off);
        assert_eq!(parsed.protocol, 2);
        assert_eq!(parsed.context, 384);
        assert!(Options::parse(&[
            "out=test".to_owned(),
            "tokenizer=tokenizer.json".to_owned(),
            "recall=oracle".to_owned(),
            "recall_at=reply".to_owned(),
        ])
        .is_ok());
        assert!(Options::parse(&[
            "out=test".to_owned(),
            "tokenizer=tokenizer.json".to_owned(),
            "recall_at=reply".to_owned(),
        ])
        .is_err());
        assert!(Options::parse(&[
            "out=test".to_owned(),
            "tokenizer=tokenizer.json".to_owned(),
            "unknown=1".to_owned(),
        ])
        .is_err());
    }

    #[test]
    fn chain_is_off_by_default_and_checked_when_on() {
        let parse = |extra: &[&str]| {
            let mut arguments = vec!["out=test".to_owned(), "tokenizer=tokenizer.json".to_owned()];
            arguments.extend(extra.iter().map(|a| (*a).to_owned()));
            Options::parse(&arguments)
        };
        assert_eq!(parse(&[]).expect("minimal arguments").chain, 0);
        assert_eq!(
            parse(&["chain=2048", "context=2048"])
                .expect("a chained corpus")
                .chain,
            2_048
        );
        // Twice the episode context is the least budget.
        assert_eq!(
            parse(&["chain=512", "context=512"])
                .expect("the least chained budget")
                .chain,
            512
        );
        // Under twice the episode context, past the context, and with memory lines.
        assert!(parse(&["chain=511", "context=2048"]).is_err());
        assert!(parse(&["chain=2048"]).is_err());
        assert!(parse(&["chain=2048", "context=2048", "recall=oracle"]).is_err());
    }

    #[test]
    fn the_default_corpus_records_no_chain() {
        let root = clean("no-chain");
        generate(&options(&root, Recall::Off)).expect("a corpus");
        let manifest: Value =
            serde_json::from_slice(&fs::read(root.join("train").join("manifest.json")).unwrap())
                .unwrap();
        assert!(manifest["composition"]["m_world"].get("chain").is_none());
        assert!(manifest["composition"]["m_world"]["episodes_per_kind"]
            .get("Chain")
            .is_none());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn chained_corpora_stay_within_their_budget_under_either_protocol() {
        for protocol in [1u8, 2] {
            let root = clean(&format!("chain-p{protocol}"));
            // The byte tokenizer fits no MQAR episode: options()'s mix has none.
            let mut chained = options(&root, Recall::Off);
            chained.conversations = 6;
            chained.chain = 1_024;
            chained.context = 1_024;
            chained.protocol = protocol;
            let receipt = generate(&chained).expect("a chained corpus");
            assert!(
                receipt.longest <= 1_024 && receipt.longest > CONTEXT,
                "{receipt:?}"
            );
            let manifest: Value = serde_json::from_slice(
                &fs::read(root.join("train").join("manifest.json")).unwrap(),
            )
            .unwrap();
            let m_world = &manifest["composition"]["m_world"];
            let record = &m_world["chain"];
            assert_eq!(record["budget"], 1_024);
            assert_eq!(record["documents"], 6);
            assert_eq!(record["metered_mismatches"], 0, "{record}");
            assert!(record["queries"].as_u64().unwrap() >= 6, "{record}");
            assert_eq!(record["document_tokens"]["max"], receipt.longest);
            assert_eq!(m_world["episodes_per_kind"]["Chain"]["episodes"], 6);
            let again = clean(&format!("chain-p{protocol}-again"));
            chained.out = again.clone();
            assert_eq!(generate(&chained).expect("the same corpus"), receipt);
            assert_eq!(read(&root), read(&again), "byte deterministic");
            let _ = fs::remove_dir_all(root);
            let _ = fs::remove_dir_all(again);
        }
    }
}
