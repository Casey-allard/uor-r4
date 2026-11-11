//! Deterministic, aligned dialogue-scenario corpus generation.
//!
//! A scenario id binds one fact family to its statements, question wording,
//! answer value, control condition, reference reply and expected EOS. The
//! output's `tokens.u16` and `response_mask.u8` use the existing chat-corpus
//! ABI, while `scenarios.jsonl` keeps the semantic labels available to tests and
//! diagnostics.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uor_r4_core::native_geometric::mmap_corpus::CorpusWriter;
use uor_r4_tokenizer::dialogue::{DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;

use crate::milestone_world::Rng;

pub const SCENARIO_SCHEMA: &str = "uor-r4.scenario-corpus/1";
pub const TRAIN_MANIFEST_SCHEMA: &str = "uor-r4-chat-corpus/v1";
pub const MASK_SCHEMA: &str = "uor-r4-response-mask/u8/v1";
pub const GENERATOR: &str = "scenario-corpus/1";

const ATTRIBUTES: &[&str] = &[
    "favorite color",
    "lucky number",
    "favorite fruit",
    "pet name",
    "home city",
    "project code",
    "meeting room",
    "signal word",
    "unlock phrase",
    "route number",
];
const ADJECTIVES: &[&str] = &[
    "amber", "brisk", "clear", "deep", "gentle", "hidden", "quiet", "silver", "steady", "violet",
];
const NOUNS: &[&str] = &[
    "beacon", "cedar", "delta", "harbor", "lantern", "meadow", "orchard", "river", "signal",
    "stone",
];
const GENERAL_REPLIES: &[&str] = &[
    "Hello! How can I help?",
    "I am here and ready to help.",
    "Good to hear from you.",
    "I can help with that.",
];
const DECLINES: &[&str] = &[
    "I don't know that yet.",
    "I don't have that information.",
    "You haven't told me that.",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScenarioSplit {
    Train,
    Development,
    Fresh,
}

impl ScenarioSplit {
    pub const ALL: [Self; 3] = [Self::Train, Self::Development, Self::Fresh];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Train => "train",
            Self::Development => "development",
            Self::Fresh => "fresh",
        }
    }

    pub const fn code(self) -> &'static str {
        match self {
            Self::Train => "tr",
            Self::Development => "dv",
            Self::Fresh => "fr",
        }
    }

    pub const fn value_base(self) -> usize {
        match self {
            Self::Train => 0,
            Self::Development => 1_000_000,
            Self::Fresh => 2_000_000,
        }
    }

    pub const fn seed_salt(self) -> u64 {
        match self {
            Self::Train => 0x10,
            Self::Development => 0x20,
            Self::Fresh => 0x30,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScenarioArchetype {
    Memorize,
    Paraphrase,
    Selection,
    Update,
    Abstention,
    Copy,
    General,
    NoMemoryControl,
    WrongMemoryControl,
    StaleMemoryControl,
}

impl ScenarioArchetype {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Memorize => "memorize",
            Self::Paraphrase => "paraphrase",
            Self::Selection => "selection",
            Self::Update => "update",
            Self::Abstention => "abstention",
            Self::Copy => "copy",
            Self::General => "general",
            Self::NoMemoryControl => "no-memory-control",
            Self::WrongMemoryControl => "wrong-memory-control",
            Self::StaleMemoryControl => "stale-memory-control",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioTurn {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioRow {
    pub scenario_id: String,
    pub family_id: String,
    pub split: ScenarioSplit,
    pub archetype: ScenarioArchetype,
    pub question_family: String,
    pub entity: Option<String>,
    pub attribute: Option<String>,
    pub value: Option<String>,
    pub distractor_value: Option<String>,
    pub scored_question: bool,
    pub expected_eos: bool,
    pub reference_reply: String,
    pub turns: Vec<ScenarioTurn>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioCorpusConfig {
    pub seed: u64,
    pub train: usize,
    pub development: usize,
    pub fresh: usize,
    pub context: usize,
}

impl Default for ScenarioCorpusConfig {
    fn default() -> Self {
        Self {
            seed: 1,
            train: 20_000,
            development: 1_024,
            fresh: 1_024,
            context: 256,
        }
    }
}

impl ScenarioCorpusConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.train < 32 || self.development < 32 || self.fresh < 32 {
            return Err("each split needs at least 32 scenarios".to_owned());
        }
        if self.context < 64 {
            return Err("context must be at least 64 tokens".to_owned());
        }
        Ok(())
    }

    fn count(&self, split: ScenarioSplit) -> usize {
        match split {
            ScenarioSplit::Train => self.train,
            ScenarioSplit::Development => self.development,
            ScenarioSplit::Fresh => self.fresh,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitReceipt {
    pub rows: usize,
    pub question_rows: usize,
    pub question_permille: u32,
    pub tokens: u64,
    pub response_tokens: usize,
    pub archetypes: BTreeMap<String, usize>,
    pub tokens_sha256: String,
    pub mask_sha256: String,
    pub manifest_sha256: String,
    pub scenarios_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioCorpusReceipt {
    pub schema: String,
    pub generator: String,
    pub config: ScenarioCorpusConfig,
    pub tokenizer: Value,
    pub splits: BTreeMap<String, SplitReceipt>,
    pub holdouts: Vec<String>,
}

pub fn generate_scenarios(config: &ScenarioCorpusConfig) -> Result<Vec<ScenarioRow>, String> {
    config.validate()?;
    let mut rows = Vec::with_capacity(config.train + config.development + config.fresh);
    for split in ScenarioSplit::ALL {
        let mut rng = Rng::new(config.seed ^ split.seed_salt());
        for index in 0..config.count(split) {
            rows.push(make_scenario(split, index, &mut rng)?);
        }
    }
    validate_rows(&rows)?;
    Ok(rows)
}

pub fn write_corpus(
    config: &ScenarioCorpusConfig,
    tokenizer_path: &Path,
    out: &Path,
) -> Result<ScenarioCorpusReceipt, String> {
    let tokenizer_bytes =
        fs::read(tokenizer_path).map_err(|e| format!("{}: {e}", tokenizer_path.display()))?;
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes).ok_or_else(|| {
            format!(
                "{} is not a byte-level BPE tokenizer",
                tokenizer_path.display()
            )
        })?;
    let protocol = DialogueProtocol::literal_roles_v2(&tokenizer)
        .map_err(|e| format!("dialogue protocol: {e}"))?;
    let encoder = protocol
        .bind(&tokenizer)
        .map_err(|e| format!("dialogue protocol: {e}"))?;
    let tokenizer_sha256 = crate::sha256_file(tokenizer_path).map_err(|e| e.to_string())?;
    let rows = generate_scenarios(config)?;
    fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let context = WriteContext {
        config,
        out,
        tokenizer_path,
        tokenizer_sha256: &tokenizer_sha256,
        tokenizer: &tokenizer,
        protocol: &protocol,
        encoder: &encoder,
    };

    let mut splits = BTreeMap::new();
    for split in ScenarioSplit::ALL {
        let split_rows = rows
            .iter()
            .filter(|row| row.split == split)
            .collect::<Vec<_>>();
        let receipt = write_split(&context, split, &split_rows)?;
        splits.insert(split.as_str().to_owned(), receipt);
    }

    let receipt = ScenarioCorpusReceipt {
        schema: SCENARIO_SCHEMA.to_owned(),
        generator: GENERATOR.to_owned(),
        config: config.clone(),
        tokenizer: json!({
            "path": tokenizer_path,
            "sha256": tokenizer_sha256,
            "tokenizer_cid": tokenizer.address(),
            "vocab_size": tokenizer.vocab_size(),
            "bos_id": protocol.bos_id,
            "eos_id": protocol.eos_id,
            "unk_id": protocol.unk_id,
            "dialogue_schema": protocol.schema,
        }),
        splits,
        holdouts: vec![
            "scenario-id".to_owned(),
            "entity-name".to_owned(),
            "fact-value-tuple".to_owned(),
            "question-template-family".to_owned(),
            "composition".to_owned(),
        ],
    };
    fs::write(
        out.join("scenario-manifest.json"),
        serde_json::to_vec_pretty(&receipt).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(receipt)
}

pub fn check_corpus(
    config: &ScenarioCorpusConfig,
    tokenizer_path: &Path,
    existing: &Path,
) -> Result<(), String> {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let generated = std::env::temp_dir().join(format!(
        "uor-scenario-corpus-check-{}-{nonce}",
        std::process::id()
    ));
    let checked = write_corpus(config, tokenizer_path, &generated).and_then(|_| {
        let files = [
            "scenario-manifest.json",
            "train/tokens.u16",
            "train/response_mask.u8",
            "train/manifest.json",
            "train/scenarios.jsonl",
            "development/tokens.u16",
            "development/response_mask.u8",
            "development/manifest.json",
            "development/scenarios.jsonl",
            "fresh/tokens.u16",
            "fresh/response_mask.u8",
            "fresh/manifest.json",
            "fresh/scenarios.jsonl",
        ];
        for relative in files {
            let expected = fs::read(generated.join(relative))
                .map_err(|e| format!("generated {relative}: {e}"))?;
            let actual = fs::read(existing.join(relative))
                .map_err(|e| format!("{}: {e}", existing.join(relative).display()))?;
            if expected != actual {
                return Err(format!("{relative} differs from regeneration"));
            }
        }
        Ok(())
    });
    let _ = fs::remove_dir_all(&generated);
    checked
}

struct WriteContext<'a, 'tokenizer> {
    config: &'a ScenarioCorpusConfig,
    out: &'a Path,
    tokenizer_path: &'a Path,
    tokenizer_sha256: &'a str,
    tokenizer: &'a ByteBpeTokenizer,
    protocol: &'a DialogueProtocol,
    encoder: &'a uor_r4_tokenizer::dialogue::DialogueEncoder<'tokenizer>,
}

fn write_split(
    context: &WriteContext<'_, '_>,
    split: ScenarioSplit,
    rows: &[&ScenarioRow],
) -> Result<SplitReceipt, String> {
    let dir = context.out.join(split.as_str());
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let vocab = u32::try_from(context.tokenizer.vocab_size())
        .map_err(|_| "tokenizer vocabulary does not fit u32")?;
    let mut token_ids = Vec::new();
    let mut response_mask = Vec::new();
    let mut archetypes = BTreeMap::new();
    let mut question_rows = 0usize;
    let mut scenarios = Vec::new();
    for row in rows {
        let messages = row
            .turns
            .iter()
            .map(|turn| Message {
                role: &turn.role,
                content: &turn.content,
            })
            .collect::<Vec<_>>();
        let encoded = context.encoder.encode_document(&messages);
        if encoded.emitted_turns != row.turns.len()
            || encoded.skipped_empty != 0
            || encoded.skipped_unknown_role != 0
            || encoded.special_token_occurrences != 0
        {
            return Err(format!("{} did not encode exactly", row.scenario_id));
        }
        if encoded.tokens.len() > context.config.context {
            return Err(format!(
                "{} has {} tokens, over context {}",
                row.scenario_id,
                encoded.tokens.len(),
                context.config.context
            ));
        }
        if encoded.response_mask.len() != encoded.tokens.len()
            || encoded.response_mask.iter().all(|&mask| mask == 0)
            || encoded.tokens.last().copied() != Some(context.protocol.eos_id)
            || encoded.response_mask.last().copied() != Some(1)
        {
            return Err(format!("{} has an invalid response mask", row.scenario_id));
        }
        let masked_ids = encoded
            .tokens
            .iter()
            .zip(&encoded.response_mask)
            .filter_map(|(&id, &mask)| (mask == 1).then_some(id))
            .collect::<Vec<_>>();
        let decoded = context.tokenizer.decode(&masked_ids);
        for turn in row.turns.iter().filter(|turn| turn.role == "assistant") {
            if !decoded.contains(&turn.content) {
                return Err(format!(
                    "{} does not fully mask assistant reply {:?}",
                    row.scenario_id, turn.content
                ));
            }
        }
        if let Some(value) = &row.value {
            if !decoded.contains(value) {
                return Err(format!(
                    "{}'s masked response does not contain answer {value:?}",
                    row.scenario_id
                ));
            }
        }
        token_ids.extend(
            encoded
                .tokens
                .into_iter()
                .map(|id| {
                    u16::try_from(id).map_err(|_| format!("{} token is over u16", row.scenario_id))
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
        response_mask.extend(encoded.response_mask);
        if row.scored_question {
            question_rows += 1;
        }
        *archetypes
            .entry(row.archetype.as_str().to_owned())
            .or_insert(0) += 1;
        scenarios.push(serde_json::to_vec(row).map_err(|e| e.to_string())?);
    }

    let tokens_path = dir.join("tokens.u16");
    let mut writer = CorpusWriter::create(&tokens_path, vocab)
        .map_err(|e| format!("{}: {e}", tokens_path.display()))?;
    writer
        .write_tokens(&token_ids)
        .map_err(|e| format!("{}: {e}", tokens_path.display()))?;
    let tokens = writer
        .finish()
        .map_err(|e| format!("{}: {e}", tokens_path.display()))?;
    let mask_path = dir.join("response_mask.u8");
    fs::write(&mask_path, &response_mask).map_err(|e| format!("{}: {e}", mask_path.display()))?;
    let scenarios_path = dir.join("scenarios.jsonl");
    fs::write(&scenarios_path, scenarios.join(&b'\n'))
        .map_err(|e| format!("{}: {e}", scenarios_path.display()))?;
    let response_tokens = response_mask.iter().filter(|&&mask| mask == 1).count();
    let question_permille = ((question_rows as u64 * 1000) / rows.len().max(1) as u64) as u32;
    let manifest = json!({
        "schema": TRAIN_MANIFEST_SCHEMA,
        "mask_schema": MASK_SCHEMA,
        "split": split.as_str(),
        "mask_rule": "1 = each assistant response token and its <|eos|>; 0 = BOS, role markers, separators and user content",
        "template_rule": "literal-role dialogue v2; BOS then turns joined by newline; every assistant turn ends with EOS; the final responder is always Assistant",
        "dialogue_protocol": context.protocol.schema,
        "max_tokens": context.config.context,
        "tokenizer": {
            "path": context.tokenizer_path,
            "sha256": context.tokenizer_sha256,
            "bos_id": context.protocol.bos_id,
            "eos_id": context.protocol.eos_id,
            "unk_id": context.protocol.unk_id,
            "tokenizer_cid": context.protocol.tokenizer_cid,
            "vocab_size": context.tokenizer.vocab_size(),
        },
        "inputs": [{"label": format!("scenario-corpus.{}", split.as_str()), "path": "generated"}],
        "files": [{
            "label": format!("scenario-corpus.{}", split.as_str()),
            "path": "generated: uor_r4_training::scenario_corpus",
            "rows_total": rows.len(),
            "rows_used": rows.len(),
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
        "rows_used": rows.len(),
        "tokens": tokens,
        "response_tokens": response_tokens,
        "tokens_bytes": fs::metadata(&tokens_path).map_err(|e| e.to_string())?.len(),
        "mask_bytes": response_mask.len(),
        "tokens_sha256": crate::sha256_file(&tokens_path).map_err(|e| e.to_string())?,
        "mask_sha256": crate::sha256_file(&mask_path).map_err(|e| e.to_string())?,
        "composition": {
            "scenario_corpus": {
                "schema": SCENARIO_SCHEMA,
                "generator": GENERATOR,
                "config": context.config,
                "question_rows": question_rows,
                "question_permille": question_permille,
                "archetypes": archetypes,
                "scenarios_path": "scenarios.jsonl",
            }
        }
    });
    let manifest_path = dir.join("manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| format!("{}: {e}", manifest_path.display()))?;
    Ok(SplitReceipt {
        rows: rows.len(),
        question_rows,
        question_permille,
        tokens,
        response_tokens,
        archetypes,
        tokens_sha256: crate::sha256_file(&tokens_path).map_err(|e| e.to_string())?,
        mask_sha256: crate::sha256_file(&mask_path).map_err(|e| e.to_string())?,
        manifest_sha256: crate::sha256_file(&manifest_path).map_err(|e| e.to_string())?,
        scenarios_sha256: crate::sha256_file(&scenarios_path).map_err(|e| e.to_string())?,
    })
}

fn make_scenario(split: ScenarioSplit, index: usize, rng: &mut Rng) -> Result<ScenarioRow, String> {
    let serial = split.value_base() + index + 1;
    let entity = entity_name(split, serial);
    let alternate = format!("{}-alt", entity);
    let attribute = ATTRIBUTES[index % ATTRIBUTES.len()].to_owned();
    let alternate_attribute = ATTRIBUTES[(index + 3) % ATTRIBUTES.len()].to_owned();
    let value = value_name(serial);
    let next_value = value_name(serial + 500_000);
    let distractor = value_name(serial + 900_000);
    let family_id = format!("{}-{serial:07}", split.code());
    let question_family_index = question_family(split, index + rng.below(2));
    let question_family = format!("{}-{}", split.as_str(), question_family_index);
    let archetype = if index.is_multiple_of(32) {
        ScenarioArchetype::General
    } else {
        [
            ScenarioArchetype::Memorize,
            ScenarioArchetype::Paraphrase,
            ScenarioArchetype::Selection,
            ScenarioArchetype::Update,
            ScenarioArchetype::Abstention,
            ScenarioArchetype::Copy,
            ScenarioArchetype::NoMemoryControl,
            ScenarioArchetype::WrongMemoryControl,
            ScenarioArchetype::StaleMemoryControl,
        ][index % 9]
    };
    let question =
        |variant: usize| ask(split, &entity, &attribute, variant + question_family_index);
    let answer = |value: &str| match split {
        ScenarioSplit::Train => format!("{value}."),
        ScenarioSplit::Development => format!("It is {value}."),
        ScenarioSplit::Fresh => format!("The answer is {value}."),
    };
    let decline = DECLINES[index % DECLINES.len()].to_owned();
    let mut turns = Vec::new();
    let mut scored_question = true;
    let expected_value = match archetype {
        ScenarioArchetype::Memorize => {
            turns.push(user(format!("{entity}'s {attribute} is {value}.")));
            turns.push(assistant("I will remember that."));
            turns.push(user(question(0)));
            turns.push(assistant(answer(&value)));
            Some(value.clone())
        }
        ScenarioArchetype::Paraphrase => {
            turns.push(user(format!(
                "Please remember that {entity}'s {attribute} is {value}."
            )));
            turns.push(assistant("Understood."));
            turns.push(user(question(0)));
            turns.push(assistant(answer(&value)));
            turns.push(user(question(1)));
            turns.push(assistant(answer(&value)));
            Some(value.clone())
        }
        ScenarioArchetype::Selection => {
            turns.push(user(format!(
                "{entity}'s {attribute} is {value}; {alternate}'s {attribute} is {distractor}."
            )));
            turns.push(assistant("I have both."));
            turns.push(user(question(0)));
            turns.push(assistant(answer(&value)));
            Some(value.clone())
        }
        ScenarioArchetype::Update => {
            turns.push(user(format!("{entity}'s {attribute} was {distractor}.")));
            turns.push(assistant("I noted the old value."));
            turns.push(user(format!(
                "Update it: {entity}'s {attribute} is now {next_value}."
            )));
            turns.push(assistant("The new value is recorded."));
            turns.push(user(question(0)));
            turns.push(assistant(answer(&next_value)));
            Some(next_value)
        }
        ScenarioArchetype::Abstention => {
            turns.push(user(format!(
                "{entity}'s {alternate_attribute} is {distractor}."
            )));
            turns.push(assistant("I will remember that."));
            turns.push(user(question(0)));
            turns.push(assistant(decline));
            None
        }
        ScenarioArchetype::Copy => {
            let phrase = value_name(serial + 750_000);
            turns.push(user(format!("Can you repeat exactly: {phrase}?")));
            turns.push(assistant(format!("{phrase}.")));
            Some(phrase)
        }
        ScenarioArchetype::General => {
            scored_question = false;
            turns.push(user(general_user(split, index)));
            turns.push(assistant(
                GENERAL_REPLIES[index % GENERAL_REPLIES.len()].to_owned(),
            ));
            None
        }
        ScenarioArchetype::NoMemoryControl => {
            turns.push(user(question(0)));
            turns.push(assistant(decline));
            None
        }
        ScenarioArchetype::WrongMemoryControl => {
            turns.push(user(format!("{alternate}'s {attribute} is {distractor}.")));
            turns.push(assistant("I will remember that."));
            turns.push(user(question(0)));
            turns.push(assistant(DECLINES[(index + 1) % DECLINES.len()].to_owned()));
            None
        }
        ScenarioArchetype::StaleMemoryControl => {
            turns.push(user(format!("{entity}'s {attribute} is {distractor}.")));
            turns.push(assistant("I recorded the old value."));
            turns.push(user(format!("Change it to {next_value}.")));
            turns.push(assistant("The value is changed."));
            turns.push(user(question(0)));
            turns.push(assistant(answer(&next_value)));
            Some(next_value)
        }
    };
    let reference_reply = turns
        .last()
        .filter(|turn| turn.role == "assistant")
        .map(|turn| turn.content.clone())
        .ok_or_else(|| format!("{family_id} has no final assistant reply"))?;
    Ok(ScenarioRow {
        scenario_id: format!("{family_id}-{}", archetype.as_str()),
        family_id,
        split,
        archetype,
        question_family,
        entity: Some(entity),
        attribute: Some(attribute),
        value: expected_value,
        distractor_value: Some(distractor),
        scored_question,
        expected_eos: true,
        reference_reply,
        turns,
    })
}

fn user(content: String) -> ScenarioTurn {
    ScenarioTurn {
        role: "user".to_owned(),
        content,
    }
}

fn assistant(content: impl Into<String>) -> ScenarioTurn {
    ScenarioTurn {
        role: "assistant".to_owned(),
        content: content.into(),
    }
}

fn general_user(split: ScenarioSplit, index: usize) -> String {
    match split {
        ScenarioSplit::Train => format!("Hello, session {index}."),
        ScenarioSplit::Development => format!("Hi there, development {index}."),
        ScenarioSplit::Fresh => format!("Good evening, fresh {index}."),
    }
}

fn value_name(serial: usize) -> String {
    format!(
        "{} {} {}",
        ADJECTIVES[serial % ADJECTIVES.len()],
        NOUNS[(serial / ADJECTIVES.len()) % NOUNS.len()],
        serial
    )
}

fn entity_name(split: ScenarioSplit, serial: usize) -> String {
    format!(
        "{}{}{}-{}",
        split.code(),
        ["ra", "ve", "lo", "mi", "sa", "tu"][serial % 6],
        ["n", "k", "l", "s"][(serial / 6) % 4],
        serial
    )
}

fn question_family(split: ScenarioSplit, index: usize) -> usize {
    match split {
        ScenarioSplit::Train => index % 3,
        ScenarioSplit::Development => 3 + index % 2,
        ScenarioSplit::Fresh => 5 + index % 2,
    }
}

fn ask(split: ScenarioSplit, entity: &str, attribute: &str, variant: usize) -> String {
    match split {
        ScenarioSplit::Train => match variant % 3 {
            0 => format!("What is {entity}'s {attribute}?"),
            1 => format!("Remind me: what is {entity}'s {attribute}?"),
            _ => format!("Tell me, what is {entity}'s {attribute}?"),
        },
        ScenarioSplit::Development => match variant % 2 {
            0 => format!("Do you remember {entity}'s {attribute}?"),
            _ => format!("What did I say {entity}'s {attribute} was?"),
        },
        ScenarioSplit::Fresh => match variant % 2 {
            0 => format!("Can you state the {attribute} for {entity}?"),
            _ => format!("For {entity}, what is the {attribute}?"),
        },
    }
}

fn validate_rows(rows: &[ScenarioRow]) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    let mut entities: BTreeMap<ScenarioSplit, BTreeSet<&str>> = BTreeMap::new();
    let mut values: BTreeMap<ScenarioSplit, BTreeSet<(&str, &str, &str)>> = BTreeMap::new();
    let mut templates: BTreeMap<ScenarioSplit, BTreeSet<&str>> = BTreeMap::new();
    for row in rows {
        if !ids.insert(row.scenario_id.as_str()) {
            return Err(format!("duplicate scenario id {}", row.scenario_id));
        }
        if row.turns.last().map(|turn| turn.role.as_str()) != Some("assistant") || !row.expected_eos
        {
            return Err(format!(
                "{} does not end in the expected assistant EOS",
                row.scenario_id
            ));
        }
        let has_value = matches!(
            row.archetype,
            ScenarioArchetype::Memorize
                | ScenarioArchetype::Paraphrase
                | ScenarioArchetype::Selection
                | ScenarioArchetype::Update
                | ScenarioArchetype::Copy
                | ScenarioArchetype::StaleMemoryControl
        );
        if row.value.is_some() != has_value {
            return Err(format!(
                "{} has the wrong expected-value shape for {:?}",
                row.scenario_id, row.archetype
            ));
        }
        if row.scored_question == matches!(row.archetype, ScenarioArchetype::General) {
            return Err(format!("{} has the wrong scoreability", row.scenario_id));
        }
        if row.scored_question
            && !row
                .turns
                .iter()
                .any(|turn| turn.role == "user" && turn.content.contains('?'))
        {
            return Err(format!("{} has no scoreable question", row.scenario_id));
        }
        if let Some(entity) = &row.entity {
            entities
                .entry(row.split)
                .or_default()
                .insert(entity.as_str());
        }
        if let (Some(entity), Some(attribute), Some(value)) =
            (&row.entity, &row.attribute, &row.value)
        {
            values.entry(row.split).or_default().insert((
                entity.as_str(),
                attribute.as_str(),
                value.as_str(),
            ));
            if !row.reference_reply.contains(value) {
                return Err(format!(
                    "{} does not state its expected value",
                    row.scenario_id
                ));
            }
        }
        templates
            .entry(row.split)
            .or_default()
            .insert(row.question_family.as_str());
    }
    for pair in [
        (ScenarioSplit::Train, ScenarioSplit::Development),
        (ScenarioSplit::Train, ScenarioSplit::Fresh),
        (ScenarioSplit::Development, ScenarioSplit::Fresh),
    ] {
        if !entities
            .get(&pair.0)
            .into_iter()
            .flatten()
            .collect::<BTreeSet<_>>()
            .is_disjoint(
                &entities
                    .get(&pair.1)
                    .into_iter()
                    .flatten()
                    .collect::<BTreeSet<_>>(),
            )
        {
            return Err(format!(
                "entity names overlap {:?} and {:?}",
                pair.0, pair.1
            ));
        }
        if !templates
            .get(&pair.0)
            .into_iter()
            .flatten()
            .collect::<BTreeSet<_>>()
            .is_disjoint(
                &templates
                    .get(&pair.1)
                    .into_iter()
                    .flatten()
                    .collect::<BTreeSet<_>>(),
            )
        {
            return Err(format!(
                "question families overlap {:?} and {:?}",
                pair.0, pair.1
            ));
        }
        if !values
            .get(&pair.0)
            .into_iter()
            .flatten()
            .collect::<BTreeSet<_>>()
            .is_disjoint(
                &values
                    .get(&pair.1)
                    .into_iter()
                    .flatten()
                    .collect::<BTreeSet<_>>(),
            )
        {
            return Err(format!(
                "fact value tuples overlap {:?} and {:?}",
                pair.0, pair.1
            ));
        }
    }
    Ok(())
}

pub fn manifest_path(root: &Path) -> PathBuf {
    root.join("scenario-manifest.json")
}
