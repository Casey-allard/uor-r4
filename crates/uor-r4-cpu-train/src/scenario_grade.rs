//! Aligned scenario scoring over the exact CPU served-reply path.
//!
//! This is development research under D84. It binds the generated scenario
//! labels back to the tokenizer and corpus receipt, then generates responses
//! through the same CPU/artifact path used by the other development drivers.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Instant;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use uor_r4_discrete::corpus::sha256;
use uor_r4_integer::stack::{IntegerStackModel, StackChat, StackReply};
use uor_r4_tokenizer::dialogue::{DialogueProtocol, Message};
use uor_r4_tokenizer::ByteBpeTokenizer;

use crate::{invalid, validate_corpus_manifest, CpuCorpusIdentity, CpuTrainError};

pub const SCENARIO_GRADE_SCHEMA: &str = "uor-r4.cpu-scenario-grade/1";
const SCENARIO_SCHEMA: &str = "uor-r4.scenario-corpus/1";
const MAX_REPLY_TOKENS: usize = 64;

#[derive(Clone, Debug)]
pub struct ScenarioGradeConfig {
    pub split: String,
    pub max_cases: usize,
    pub reply_tokens: usize,
}

impl ScenarioGradeConfig {
    fn validate(&self) -> Result<(), CpuTrainError> {
        if !matches!(self.split.as_str(), "train" | "development" | "fresh") {
            return Err(invalid(
                "scenario split must be train, development or fresh",
            ));
        }
        if self.max_cases == 0 || self.reply_tokens == 0 {
            return Err(invalid("scenario case and reply caps must be positive"));
        }
        if self.reply_tokens > MAX_REPLY_TOKENS {
            return Err(invalid("scenario reply cap exceeds the served-path limit"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioTurn {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioCase {
    pub scenario_id: String,
    pub family_id: String,
    pub split: String,
    pub archetype: String,
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
pub struct ScenarioCorpusIdentity {
    pub schema: String,
    pub split: String,
    pub tokens_sha256: String,
    pub mask_sha256: String,
    pub tokenizer_sha256: String,
    pub tokens: usize,
    pub token_bytes: usize,
    pub mask_bytes: usize,
    pub response_targets: usize,
    pub max_tokens: usize,
    pub vocab: usize,
    pub bos: u32,
}

impl From<CpuCorpusIdentity> for ScenarioCorpusIdentity {
    fn from(identity: CpuCorpusIdentity) -> Self {
        Self {
            schema: identity.schema,
            split: identity.split,
            tokens_sha256: identity.tokens_sha256,
            mask_sha256: identity.mask_sha256,
            tokenizer_sha256: identity.tokenizer_sha256,
            tokens: identity.tokens,
            token_bytes: identity.token_bytes,
            mask_bytes: identity.mask_bytes,
            response_targets: identity.response_targets,
            max_tokens: identity.max_tokens,
            vocab: identity.vocab,
            bos: identity.bos,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioGradeCounts {
    pub cases: usize,
    pub prompt_ineligible: usize,
    pub complete: usize,
    pub eos_stops: usize,
    pub short_cycle_stops: usize,
    pub cap_stops: usize,
    pub exact_reference: usize,
    pub asked_value_present: usize,
    pub correct_direct_answer: usize,
    pub distractor_present: usize,
    pub abstained: usize,
    pub repeat_4gram: usize,
    pub generated_tokens: usize,
    pub distinct_replies: usize,
}

impl ScenarioGradeCounts {
    fn empty() -> Self {
        Self {
            cases: 0,
            prompt_ineligible: 0,
            complete: 0,
            eos_stops: 0,
            short_cycle_stops: 0,
            cap_stops: 0,
            exact_reference: 0,
            asked_value_present: 0,
            correct_direct_answer: 0,
            distractor_present: 0,
            abstained: 0,
            repeat_4gram: 0,
            generated_tokens: 0,
            distinct_replies: 0,
        }
    }

    fn add(&mut self, row: &ScenarioGradeRow) {
        self.cases += 1;
        self.complete += usize::from(row.complete);
        self.eos_stops += usize::from(row.stop == "eos");
        self.short_cycle_stops += usize::from(row.stop == "short_cycle");
        self.cap_stops += usize::from(row.stop == "max_new_tokens");
        self.exact_reference += usize::from(row.exact_reference);
        self.asked_value_present += usize::from(row.asked_value_present);
        self.correct_direct_answer += usize::from(row.correct_direct_answer);
        self.distractor_present += usize::from(row.distractor_present);
        self.abstained += usize::from(row.abstained);
        self.repeat_4gram += usize::from(row.repeat_4gram);
        self.generated_tokens += row.generated_tokens;
    }

    fn finish(&mut self, rows: &[ScenarioGradeRow]) {
        self.distinct_replies = rows
            .iter()
            .filter(|row| !row.reply_text.is_empty())
            .map(|row| row.reply_text.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .len();
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioGradeRow {
    pub scenario_id: String,
    pub family_id: String,
    pub archetype: String,
    pub scored_question: bool,
    pub prompt_tokens: usize,
    pub generated_tokens: usize,
    pub stop: String,
    pub reply_text: String,
    pub reference_reply: String,
    pub asked_value: Option<String>,
    pub distractor_value: Option<String>,
    pub complete: bool,
    pub exact_reference: bool,
    pub asked_value_present: bool,
    pub correct_direct_answer: bool,
    pub distractor_present: bool,
    pub abstained: bool,
    pub repeat_4gram: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioGradeReport {
    pub schema: String,
    pub substrate: String,
    pub qualification: String,
    pub claim: String,
    pub split: String,
    pub artifact_sha256: String,
    pub tokenizer_sha256: String,
    pub scenarios_sha256: String,
    pub corpus: ScenarioCorpusIdentity,
    pub source_rows: usize,
    pub eligible_rows: usize,
    pub prompt_ineligible_rows: usize,
    pub selected_rows: usize,
    pub selected_ranks: Vec<usize>,
    pub reply_tokens: usize,
    pub elapsed_us: u64,
    pub counts: ScenarioGradeCounts,
    pub by_archetype: BTreeMap<String, ScenarioGradeCounts>,
    pub rows: Vec<ScenarioGradeRow>,
}

#[derive(Debug, Deserialize)]
struct ScenarioReceipt {
    schema: String,
    config: ReceiptConfig,
    tokenizer: ReceiptTokenizer,
    splits: BTreeMap<String, ReceiptSplit>,
}

#[derive(Debug, Deserialize)]
struct ReceiptConfig {
    context: usize,
}

#[derive(Debug, Deserialize)]
struct ReceiptTokenizer {
    sha256: String,
    vocab_size: usize,
    bos_id: u32,
    dialogue_schema: String,
}

#[derive(Debug, Deserialize)]
struct ReceiptSplit {
    rows: usize,
    tokens_sha256: String,
    mask_sha256: String,
    manifest_sha256: String,
    scenarios_sha256: String,
}

struct ScenarioCandidate {
    case: ScenarioCase,
    prompt: Vec<u32>,
}

pub fn grade_scenarios(
    artifact: &[u8],
    corpus_root: &Path,
    tokenizer_path: &Path,
    config: ScenarioGradeConfig,
) -> Result<ScenarioGradeReport, CpuTrainError> {
    config.validate()?;
    let started = Instant::now();
    let receipt_bytes = read_path(&corpus_root.join("scenario-manifest.json"))?;
    let receipt = parse_receipt(&receipt_bytes)?;
    let split_root = corpus_root.join(&config.split);
    let token_bytes = read_path(&split_root.join("tokens.u16"))?;
    let mask = read_path(&split_root.join("response_mask.u8"))?;
    let manifest_bytes = read_path(&split_root.join("manifest.json"))?;
    let corpus = validate_corpus_manifest(&manifest_bytes, &token_bytes, &mask)?;
    let split_receipt = receipt
        .splits
        .get(&config.split)
        .ok_or_else(|| invalid("scenario receipt has no requested split"))?;
    if corpus.split != config.split
        || corpus.vocab != receipt.tokenizer.vocab_size
        || corpus.bos != receipt.tokenizer.bos_id
        || corpus.tokenizer_sha256 != receipt.tokenizer.sha256
        || corpus.max_tokens != receipt.config.context
        || corpus.tokens_sha256 != split_receipt.tokens_sha256
        || corpus.mask_sha256 != split_receipt.mask_sha256
        || sha256(&manifest_bytes) != split_receipt.manifest_sha256
    {
        return Err(invalid("scenario split identity differs from its receipt"));
    }
    let scenarios_bytes = read_path(&split_root.join("scenarios.jsonl"))?;
    let scenarios_sha256 = validate_scenarios_sha(&receipt, &config.split, &scenarios_bytes)?;

    let tokenizer_bytes = read_path(tokenizer_path)?;
    let tokenizer_sha256 = sha256(&tokenizer_bytes);
    if tokenizer_sha256 != receipt.tokenizer.sha256 {
        return Err(invalid("scenario tokenizer differs from its receipt"));
    }
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or_else(|| invalid("scenario tokenizer is not byte-level BPE"))?;
    if tokenizer.vocab_size() != corpus.vocab {
        return Err(invalid("scenario tokenizer vocabulary differs"));
    }
    let protocol = DialogueProtocol::literal_roles_v2(&tokenizer)
        .map_err(|error| invalid(format!("scenario dialogue protocol: {error}")))?;
    let encoder = protocol
        .bind(&tokenizer)
        .map_err(|error| invalid(format!("scenario dialogue encoder: {error}")))?;

    let model = IntegerStackModel::parse(artifact)
        .map_err(|error| invalid(format!("scenario artifact: {error}")))?;
    if model.shape().vocab != corpus.vocab {
        return Err(invalid("scenario artifact vocabulary differs"));
    }
    model
        .require_tokenizer_sha256(&tokenizer_sha256)
        .map_err(|error| invalid(format!("scenario artifact tokenizer binding: {error}")))?;

    let cases = parse_cases(&scenarios_bytes, &config.split)?;
    let source_rows = cases.len();
    if source_rows != split_receipt.rows {
        return Err(invalid("scenario label row count differs from its receipt"));
    }
    let mut candidates = Vec::new();
    let mut prompt_ineligible_rows = 0usize;
    let mut ineligible_by_archetype = BTreeMap::<String, usize>::new();
    for case in cases {
        let Some((last, history)) = case.turns.split_last() else {
            return Err(invalid("scenario case has no turns"));
        };
        if last.role != "assistant" {
            return Err(invalid("scenario case does not end in an assistant turn"));
        }
        let messages = history
            .iter()
            .map(|turn| Message {
                role: &turn.role,
                content: &turn.content,
            })
            .collect::<Vec<_>>();
        let encoded = encoder.encode_assistant_prefix(&messages);
        if encoded.emitted_turns != history.len()
            || encoded.skipped_empty != 0
            || encoded.skipped_unknown_role != 0
            || encoded.special_token_occurrences != 0
        {
            return Err(invalid(format!(
                "scenario {} prompt does not encode exactly",
                case.scenario_id
            )));
        }
        if encoded.tokens.is_empty() || encoded.tokens.len() >= model.shape().context {
            prompt_ineligible_rows += 1;
            *ineligible_by_archetype.entry(case.archetype).or_default() += 1;
            continue;
        }
        candidates.push(ScenarioCandidate {
            case,
            prompt: encoded.tokens,
        });
    }
    let eligible_rows = candidates.len();
    if eligible_rows == 0 {
        return Err(invalid("scenario split has no prompt-eligible cases"));
    }
    let selected_rows = config.max_cases.min(eligible_rows);
    let selected = (0..selected_rows)
        .map(|rank| even_rank(rank, eligible_rows, selected_rows))
        .collect::<Vec<_>>();
    let selected_ranks = selected.clone();

    let rows = selected
        .par_iter()
        .map(|&index| {
            let candidate = &candidates[index];
            let mut chat = StackChat::from_tokenizer_json(&model, &tokenizer_bytes, 2)
                .map_err(|error| invalid(format!("scenario chat: {error}")))?;
            let cap = config
                .reply_tokens
                .min(model.shape().context - candidate.prompt.len());
            let reply = chat
                .reply_history(&candidate.prompt, cap)
                .map_err(|error| invalid(format!("scenario reply: {error}")))?;
            Ok(score_row(&candidate.case, &candidate.prompt, &reply))
        })
        .collect::<Result<Vec<_>, CpuTrainError>>()?;

    let mut counts = ScenarioGradeCounts::empty();
    let mut by_archetype = BTreeMap::<String, Vec<ScenarioGradeRow>>::new();
    for row in &rows {
        counts.add(row);
        by_archetype
            .entry(row.archetype.clone())
            .or_default()
            .push(row.clone());
    }
    counts.finish(&rows);
    counts.prompt_ineligible = prompt_ineligible_rows;
    for archetype in ineligible_by_archetype.keys() {
        by_archetype.entry(archetype.clone()).or_default();
    }
    let by_archetype = by_archetype
        .into_iter()
        .map(|(archetype, rows)| {
            let mut counts = ScenarioGradeCounts::empty();
            for row in &rows {
                counts.add(row);
            }
            counts.finish(&rows);
            counts.prompt_ineligible = ineligible_by_archetype
                .get(&archetype)
                .copied()
                .unwrap_or(0);
            (archetype, counts)
        })
        .collect();

    Ok(ScenarioGradeReport {
        schema: SCENARIO_GRADE_SCHEMA.to_owned(),
        substrate: "cpu-development-research".to_owned(),
        qualification: "gpu-required".to_owned(),
        claim: "aligned scenario diagnostic over the exact StackChat CPU served-reply path; not GPU qualification, useful-chat evidence or a capability bar".to_owned(),
        split: config.split,
        artifact_sha256: sha256(artifact),
        tokenizer_sha256,
        scenarios_sha256,
        corpus: corpus.into(),
        source_rows,
        eligible_rows,
        prompt_ineligible_rows,
        selected_rows,
        selected_ranks,
        reply_tokens: config.reply_tokens,
        elapsed_us: started.elapsed().as_micros().try_into().unwrap_or(u64::MAX),
        counts,
        by_archetype,
        rows,
    })
}

fn parse_receipt(bytes: &[u8]) -> Result<ScenarioReceipt, CpuTrainError> {
    let receipt: ScenarioReceipt = serde_json::from_slice(bytes)
        .map_err(|error| invalid(format!("scenario receipt: {error}")))?;
    if receipt.schema != SCENARIO_SCHEMA {
        return Err(invalid("scenario receipt schema differs"));
    }
    if receipt.tokenizer.dialogue_schema != "uor-r4.literal-role-dialogue/2" {
        return Err(invalid("scenario dialogue protocol version differs"));
    }
    Ok(receipt)
}

fn validate_scenarios_sha(
    receipt: &ScenarioReceipt,
    split: &str,
    scenarios: &[u8],
) -> Result<String, CpuTrainError> {
    let split_receipt = receipt
        .splits
        .get(split)
        .ok_or_else(|| invalid("scenario receipt has no requested split"))?;
    let actual = sha256(scenarios);
    if actual != split_receipt.scenarios_sha256 {
        return Err(invalid("scenario labels differ from their receipt"));
    }
    Ok(actual)
}

fn parse_cases(bytes: &[u8], split: &str) -> Result<Vec<ScenarioCase>, CpuTrainError> {
    let mut cases = Vec::new();
    for (index, line) in bytes.split(|byte| *byte == b'\n').enumerate() {
        if line.is_empty() {
            continue;
        }
        let case: ScenarioCase = serde_json::from_slice(line)
            .map_err(|error| invalid(format!("scenario line {}: {error}", index + 1)))?;
        if case.split != split {
            return Err(invalid(format!(
                "scenario {} belongs to {}, not {split}",
                case.scenario_id, case.split
            )));
        }
        cases.push(case);
    }
    if cases.is_empty() {
        return Err(invalid("scenario labels are empty"));
    }
    Ok(cases)
}

fn score_row(case: &ScenarioCase, prompt: &[u32], generated: &StackReply) -> ScenarioGradeRow {
    score_decoded(case, prompt.len(), generated, generated.text.clone())
}

fn score_decoded(
    case: &ScenarioCase,
    prompt_tokens: usize,
    generated: &StackReply,
    reply_text: String,
) -> ScenarioGradeRow {
    let reference_reply = case.reference_reply.trim().to_owned();
    let asked_value_present = case
        .value
        .as_ref()
        .is_some_and(|value| reply_text.contains(value));
    let distractor_present = case
        .distractor_value
        .as_ref()
        .is_some_and(|value| reply_text.contains(value));
    let abstained = case.scored_question
        && case.value.is_none()
        && !distractor_present
        && is_decline(&reply_text);
    ScenarioGradeRow {
        scenario_id: case.scenario_id.clone(),
        family_id: case.family_id.clone(),
        archetype: case.archetype.clone(),
        scored_question: case.scored_question,
        prompt_tokens,
        generated_tokens: generated.ids.len(),
        stop: generated.stop.name().to_owned(),
        exact_reference: reply_text == reference_reply,
        reply_text,
        reference_reply,
        asked_value: case.value.clone(),
        distractor_value: case.distractor_value.clone(),
        complete: generated.stop.name() == "eos" && case.expected_eos,
        asked_value_present,
        correct_direct_answer: asked_value_present && !distractor_present,
        distractor_present,
        abstained,
        repeat_4gram: has_repeated_4gram(&generated.ids),
    }
}

fn is_decline(text: &str) -> bool {
    let folded = text.to_ascii_lowercase();
    folded.contains("don't know")
        || folded.contains("don't have that information")
        || folded.contains("haven't told me")
}

fn has_repeated_4gram(tokens: &[u32]) -> bool {
    if tokens.len() < 8 {
        return false;
    }
    let mut seen = std::collections::BTreeSet::new();
    for gram in tokens.windows(4) {
        if !seen.insert(gram) {
            return true;
        }
    }
    false
}

fn even_rank(rank: usize, total: usize, selected: usize) -> usize {
    usize::try_from(rank as u128 * total as u128 / selected as u128).unwrap_or(usize::MAX)
}

fn read_path(path: &Path) -> Result<Vec<u8>, CpuTrainError> {
    std::fs::read(path).map_err(|error| invalid(format!("{}: {error}", path.display())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};
    use uor_r4_integer::stack::StackStop;
    use uor_r4_integer::stack::{
        export_fbs_trunk_uorlut01_with_provenance, FbsExportProvenance, FbsExportSnaps,
        StackContractProvenance,
    };

    fn case(value: Option<&str>, distractor: Option<&str>, scored: bool) -> ScenarioCase {
        ScenarioCase {
            scenario_id: "dv-1-test".to_owned(),
            family_id: "dv-1".to_owned(),
            split: "development".to_owned(),
            archetype: "test".to_owned(),
            question_family: "development-1".to_owned(),
            entity: Some("entity".to_owned()),
            attribute: Some("color".to_owned()),
            value: value.map(str::to_owned),
            distractor_value: distractor.map(|value| value.to_owned()),
            scored_question: scored,
            expected_eos: true,
            reference_reply: "It is amber.".to_owned(),
            turns: vec![ScenarioTurn {
                role: "assistant".to_owned(),
                content: "It is amber.".to_owned(),
            }],
        }
    }

    fn fixture_tokenizer() -> Vec<u8> {
        let mut vocab = serde_json::Map::new();
        for (id, surface) in ["<|bos|>", "<|eos|>", "<|unk|>"].iter().enumerate() {
            vocab.insert((*surface).to_owned(), serde_json::Value::from(id));
        }
        for byte in 0u32..256 {
            let character = if (0x21..=0x7e).contains(&byte)
                || (0xa1..=0xac).contains(&byte)
                || (0xae..=0xff).contains(&byte)
            {
                char::from_u32(byte).expect("ASCII byte is a character")
            } else {
                let skipped = (0u32..byte)
                    .filter(|value| {
                        !((0x21..=0x7e).contains(value)
                            || (0xa1..=0xac).contains(value)
                            || (0xae..=0xff).contains(value))
                    })
                    .count() as u32;
                char::from_u32(0x100 + skipped).expect("test byte code point is valid")
            };
            vocab.insert(character.to_string(), serde_json::Value::from(byte + 3));
        }
        for id in 259u32..4096 {
            vocab.insert(format!("<filler-{id}>"), serde_json::Value::from(id));
        }
        serde_json::to_vec(&serde_json::json!({
            "model": {
                "type": "BPE",
                "vocab": vocab,
                "merges": []
            },
            "pre_tokenizer": {
                "type": "ByteLevel",
                "add_prefix_space": false
            },
            "added_tokens": [
                {"id": 0, "content": "<|bos|>"},
                {"id": 1, "content": "<|eos|>"},
                {"id": 2, "content": "<|unk|>"}
            ]
        }))
        .unwrap()
    }

    fn uort(tokens: &[u16], vocab: u32) -> Vec<u8> {
        let mut bytes = vec![0u8; 64];
        bytes[..4].copy_from_slice(b"UORT");
        bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
        bytes[8..16].copy_from_slice(&(tokens.len() as u64).to_le_bytes());
        bytes[16..20].copy_from_slice(&vocab.to_le_bytes());
        for token in tokens {
            bytes.extend_from_slice(&token.to_le_bytes());
        }
        bytes
    }

    fn scenario_root() -> (PathBuf, PathBuf) {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "uor-scenario-grade-test-{}-{nonce}",
            std::process::id()
        ));
        let split = root.join("development");
        fs::create_dir_all(&split).unwrap();

        let tokenizer = fixture_tokenizer();
        let tokenizer_path = root.join("tokenizer.json");
        fs::write(&tokenizer_path, &tokenizer).unwrap();
        let tokenizer_sha256 = sha256(&tokenizer);

        let tokens = uort(&[0, 3, 4, 1], 4096);
        let mask = vec![0, 0, 0, 1];
        fs::write(split.join("tokens.u16"), &tokens).unwrap();
        fs::write(split.join("response_mask.u8"), &mask).unwrap();

        let case = serde_json::to_vec(&serde_json::json!({
            "scenario_id": "dv-test-copy",
            "family_id": "dv-test",
            "split": "development",
            "archetype": "copy",
            "question_family": "development-1",
            "entity": null,
            "attribute": null,
            "value": "Hi",
            "distractor_value": "Bye",
            "scored_question": true,
            "expected_eos": true,
            "reference_reply": "Hi",
            "turns": [
                {"role": "user", "content": "Hi"},
                {"role": "assistant", "content": "Hi"}
            ]
        }))
        .unwrap();
        fs::write(split.join("scenarios.jsonl"), &case).unwrap();
        let scenarios_sha256 = sha256(&case);

        let manifest = serde_json::to_vec(&serde_json::json!({
            "schema": "uor-r4-chat-corpus/v1",
            "mask_schema": "uor-r4-response-mask/u8/v1",
            "split": "development",
            "tokenizer": {
                "sha256": tokenizer_sha256,
                "vocab_size": 4096,
                "bos_id": 0
            },
            "tokens": 4,
            "tokens_bytes": tokens.len(),
            "mask_bytes": mask.len(),
            "response_tokens": 1,
            "max_tokens": 64,
            "tokens_sha256": sha256(&tokens),
            "mask_sha256": sha256(&mask)
        }))
        .unwrap();
        fs::write(split.join("manifest.json"), &manifest).unwrap();

        let receipt = serde_json::to_vec(&serde_json::json!({
            "schema": SCENARIO_SCHEMA,
            "config": {"context": 64},
            "tokenizer": {
                "sha256": tokenizer_sha256,
                "vocab_size": 4096,
                "bos_id": 0,
                "dialogue_schema": "uor-r4.literal-role-dialogue/2"
            },
            "splits": {
                "development": {
                    "rows": 1,
                    "tokens_sha256": sha256(&tokens),
                    "mask_sha256": sha256(&mask),
                    "manifest_sha256": sha256(&manifest),
                    "scenarios_sha256": scenarios_sha256
                }
            }
        }))
        .unwrap();
        fs::write(root.join("scenario-manifest.json"), receipt).unwrap();
        (root, tokenizer_path)
    }

    #[test]
    fn grade_scenarios_generates_a_non_vacuous_reply_through_stack_chat() {
        let (root, tokenizer_path) = scenario_root();
        let tokenizer_sha256 = sha256(&fs::read(&tokenizer_path).unwrap());
        let snapshot = synthetic_fbs_trunk_with(
            272,
            FbsSynth {
                vocab: 4096,
                d: 64,
                mlp: 64,
                layers: 1,
                context: 64,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.25,
                decay_step: 0.0625,
            },
        );
        let artifact = export_fbs_trunk_uorlut01_with_provenance(
            &snapshot,
            FbsExportSnaps::SNAPPED,
            None,
            0,
            Some(&FbsExportProvenance {
                stack_contract: StackContractProvenance {
                    source: "scenario-grade-test".to_owned(),
                    sha256: "0".repeat(64),
                    semantic_blake3: "0".repeat(64),
                },
                tokenizer_sha256,
            }),
        )
        .unwrap();
        let report = grade_scenarios(
            &artifact,
            &root,
            &tokenizer_path,
            ScenarioGradeConfig {
                split: "development".to_owned(),
                max_cases: 1,
                reply_tokens: 4,
            },
        )
        .unwrap();
        assert_eq!(report.source_rows, 1);
        assert_eq!(report.eligible_rows, 1);
        assert_eq!(report.selected_rows, 1);
        assert_eq!(report.rows.len(), 1);
        assert!(report.rows[0].prompt_tokens > 0);
        assert!(report.rows[0].generated_tokens > 0);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn value_and_distractor_scoring_are_separate() {
        let generated = StackReply {
            ids: b"It is amber."
                .iter()
                .map(|&byte| u32::from(byte))
                .collect(),
            text: "It is amber.".to_owned(),
            stop: StackStop::Eos,
        };
        let row = score_decoded(
            &case(Some("amber"), Some("violet"), true),
            2,
            &generated,
            "It is amber.".to_owned(),
        );
        assert!(row.complete);
        assert!(row.exact_reference);
        assert!(row.asked_value_present);
        assert!(row.correct_direct_answer);
        assert!(!row.distractor_present);
        assert!(!row.abstained);
    }

    #[test]
    fn abstention_requires_a_decline_and_no_distractor() {
        let generated = StackReply {
            ids: b"I don't know that yet."
                .iter()
                .map(|&byte| u32::from(byte))
                .collect(),
            text: "I don't know that yet.".to_owned(),
            stop: StackStop::Eos,
        };
        let row = score_decoded(
            &case(None, Some("violet"), true),
            2,
            &generated,
            "I don't know that yet.".to_owned(),
        );
        assert!(row.abstained);
        assert!(!row.correct_direct_answer);
        assert!(!row.distractor_present);
    }

    #[test]
    fn distractor_presence_does_not_count_as_the_asked_value() {
        let generated = StackReply {
            ids: b"It is violet."
                .iter()
                .map(|&byte| u32::from(byte))
                .collect(),
            text: "It is violet.".to_owned(),
            stop: StackStop::Eos,
        };
        let row = score_decoded(
            &case(Some("amber"), Some("violet"), true),
            2,
            &generated,
            "It is violet.".to_owned(),
        );
        assert!(!row.asked_value_present);
        assert!(!row.correct_direct_answer);
        assert!(row.distractor_present);
    }

    #[test]
    fn repeated_four_grams_are_detected() {
        assert!(has_repeated_4gram(&[1, 2, 3, 4, 1, 2, 3, 4]));
        assert!(!has_repeated_4gram(&[1, 2, 3, 4, 5, 6, 7, 8]));
    }

    #[test]
    fn selection_is_deterministic_and_bounded() {
        let selected = (0..5)
            .map(|rank| even_rank(rank, 11, 5))
            .collect::<Vec<_>>();
        assert_eq!(selected, [0, 2, 4, 6, 8]);
    }

    #[test]
    fn case_parser_refuses_a_different_split() {
        let line = serde_json::to_vec(&case(Some("amber"), None, true)).unwrap();
        let error = parse_cases(&line, "fresh").unwrap_err();
        assert!(error.to_string().contains("belongs to development"));
    }

    #[test]
    fn receipt_refuses_a_changed_scenario_hash() {
        let receipt_bytes = serde_json::to_vec(&serde_json::json!({
            "schema": SCENARIO_SCHEMA,
            "config": {
                "context": 256
            },
            "tokenizer": {
                "sha256": "0".repeat(64),
                "vocab_size": 4096,
                "bos_id": 0,
                "dialogue_schema": "uor-r4.literal-role-dialogue/2"
            },
            "splits": {
                "development": {
                    "rows": 1,
                    "tokens_sha256": "0".repeat(64),
                    "mask_sha256": "0".repeat(64),
                    "manifest_sha256": "0".repeat(64),
                    "scenarios_sha256": "0".repeat(64)
                }
            }
        }))
        .unwrap();
        let receipt = parse_receipt(&receipt_bytes).unwrap();
        let error = validate_scenarios_sha(&receipt, "development", b"changed").unwrap_err();
        assert!(error.to_string().contains("differ from their receipt"));
    }
}
