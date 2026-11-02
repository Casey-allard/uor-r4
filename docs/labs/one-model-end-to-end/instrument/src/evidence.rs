//! CPU evidence for distinguishing referenced TRAIN material from new model behaviour.
//!
//! This instrument evaluates the discrete integer artifact through the same
//! `IntegerDiscreteModel` used by native serving. It reports next-token
//! admission/top-1/top-8, checks the independent CPU writer on the serving
//! query where its fixed width-four codebook applies, and stratifies panel rows
//! by whole-run overlap with a supplied TRAIN or control split. It does not
//! call generated text "complete" and does not infer generated-reply novelty
//! when no generated-reply file is supplied.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uor_r4_cpu_train::{
    CpuDiscreteProposalSample, CpuDiscreteProposalScorer, CpuWriterCore, ABSENT_CELL, READ_WIDTH,
    VOCABULARY,
};
use uor_r4_integer::discrete_d11::IntegerDiscreteModel;
use uor_r4_tokenizer::ByteBpeTokenizer;

type Result<T> = std::result::Result<T, String>;

const SCHEMA: &str = "uor-r4.m2-evidence/1";
const OPENING_KEY_TOKENS: usize = 8;
const CENSUS_MASK_KS: [usize; 2] = [256, 1_024];
const SHARE_PARTS_PER_MILLION: u64 = 1_000_000;
const CENSUS_MASK: u32 = u32::MAX;

#[derive(Clone, Debug, Serialize, Default, PartialEq, Eq)]
pub struct Reading {
    pub targets: u64,
    pub admitted: u64,
    pub top1: u64,
    pub top8: u64,
}

impl Reading {
    fn observe(&mut self, admitted: bool, top1: bool, top8: bool) {
        self.targets += 1;
        self.admitted += u64::from(admitted);
        self.top1 += u64::from(top1);
        self.top8 += u64::from(top8);
    }

    fn merge(&mut self, other: &Self) {
        self.targets += other.targets;
        self.admitted += other.admitted;
        self.top1 += other.top1;
        self.top8 += other.top8;
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ArtifactIdentity {
    pub path: String,
    pub sha256: String,
    pub model_sha256: String,
    pub schema: String,
    pub tables: usize,
    pub vocabulary: u32,
    pub context: u32,
    pub blocks: u32,
    pub kept_cells: u32,
    pub read_mode: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct CorpusIdentity {
    pub path: String,
    pub manifest_sha256: String,
    pub split: String,
    pub cell_phrasing: Option<String>,
    pub cell_value: Option<String>,
    pub seed: Option<u64>,
    pub conversations: Option<usize>,
    pub tokenizer_sha256: String,
    pub tokens_sha256: String,
    pub mask_sha256: String,
    pub documents: usize,
    pub tokens: usize,
    pub response_targets: usize,
    pub prompt_runs: usize,
    pub reply_runs: usize,
    pub max_tokens: usize,
    pub reference_role: Option<ReferenceRole>,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceRole {
    CanonicalTrain,
    GeneratedControl,
}

impl ReferenceRole {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "canonical_train" => Ok(Self::CanonicalTrain),
            "generated_control" => Ok(Self::GeneratedControl),
            _ => Err(format!(
                "train-role= must be canonical_train or generated_control, not {value:?}"
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorpusFiles {
    pub tokens: PathBuf,
    pub mask: PathBuf,
    pub manifest: PathBuf,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ReplyOverlap {
    pub reference_replies: u64,
    pub reference_replies_seen_in_reference: u64,
    pub reference_prompts_seen_in_reference: u64,
    pub reference_prompts_keyed_in_reference: u64,
    pub reference_reply_new_4grams: u64,
    pub reference_reply_total_4grams: u64,
    pub reference_reply_new_8grams: u64,
    pub reference_reply_total_8grams: u64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct WriterParity {
    pub rows: u64,
    pub admitted_cell_mismatches: u64,
    pub top8_mismatches: u64,
    pub token_mismatches: u64,
    pub query_source: String,
    pub trunk_query_scope: String,
}

impl WriterParity {
    fn unavailable(reason: impl Into<String>) -> Self {
        Self {
            rows: 0,
            admitted_cell_mismatches: 0,
            top8_mismatches: 0,
            token_mismatches: 0,
            query_source: "unavailable".to_owned(),
            trunk_query_scope: reason.into(),
        }
    }

    fn is_exact(&self) -> bool {
        self.rows > 0
            && self.admitted_cell_mismatches == 0
            && self.top8_mismatches == 0
            && self.token_mismatches == 0
    }

    fn from_serving_query() -> Self {
        Self {
            rows: 0,
            admitted_cell_mismatches: 0,
            top8_mismatches: 0,
            token_mismatches: 0,
            query_source: "integer_serving_trunk_q12".to_owned(),
            trunk_query_scope:
                "serving trunk Q12 query; cpu_engine_parity records whether a fully independent \
                 CPU query is available for this artifact family"
                    .to_owned(),
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct CpuEngineParity {
    pub rows: u64,
    pub query_coordinates_checked: u64,
    pub query_coordinate_mismatches: u64,
    pub admitted_cell_mismatches: u64,
    pub top8_mismatches: u64,
    pub token_mismatches: u64,
    pub unavailable: Option<String>,
}

impl CpuEngineParity {
    fn from_reading(reading: uor_r4_cpu_train::CpuDiscretePanelExportReading) -> Self {
        Self {
            rows: reading.rows,
            query_coordinates_checked: reading.query_coordinates_checked,
            query_coordinate_mismatches: reading.query_coordinate_mismatches,
            admitted_cell_mismatches: reading.admitted_cell_mismatches,
            top8_mismatches: reading.top8_mismatches,
            token_mismatches: reading.token_mismatches,
            unavailable: None,
        }
    }

    fn unavailable(reason: impl Into<String>) -> Self {
        Self {
            rows: 0,
            query_coordinates_checked: 0,
            query_coordinate_mismatches: 0,
            admitted_cell_mismatches: 0,
            top8_mismatches: 0,
            token_mismatches: 0,
            unavailable: Some(reason.into()),
        }
    }

    fn is_exact(&self) -> bool {
        self.unavailable.is_none()
            && self.rows > 0
            && self.query_coordinates_checked > 0
            && self.query_coordinate_mismatches == 0
            && self.admitted_cell_mismatches == 0
            && self.top8_mismatches == 0
            && self.token_mismatches == 0
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct TokenizerIdentity {
    pub path: String,
    pub sha256: String,
    pub vocabulary: usize,
    pub digit_tokens: usize,
    pub mask_ks: Vec<usize>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct TemplateShare {
    pub k: usize,
    pub runs: u64,
    pub seen: u64,
    pub share_parts_per_million: u32,
}

impl TemplateShare {
    fn new(k: usize, runs: u64, seen: u64) -> Self {
        let share_parts_per_million = if runs == 0 {
            0
        } else {
            u32::try_from(
                u128::from(seen)
                    .saturating_mul(u128::from(SHARE_PARTS_PER_MILLION))
                    .checked_div(u128::from(runs))
                    .unwrap_or(0),
            )
            .unwrap_or(u32::MAX)
        };
        Self {
            k,
            runs,
            seen,
            share_parts_per_million,
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct TemplateOverlap {
    pub prompts: u64,
    pub replies: u64,
    pub prompt_template: Vec<TemplateShare>,
    pub reply_template: Vec<TemplateShare>,
    pub new_prompt_templates_at_1024: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct EvidenceReport {
    pub schema: String,
    pub artifact: ArtifactIdentity,
    pub tokenizer: TokenizerIdentity,
    pub panel: CorpusIdentity,
    pub train: Option<CorpusIdentity>,
    pub measured_targets: usize,
    pub overall: Reading,
    pub strata: BTreeMap<String, Reading>,
    pub reply_overlap: Option<ReplyOverlap>,
    pub template_overlap: Option<TemplateOverlap>,
    pub writer_parity: WriterParity,
    pub cpu_engine_parity: CpuEngineParity,
    pub unavailable: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
struct Run {
    ordinal: usize,
    document: usize,
    mask: u8,
    range: Range<usize>,
}

#[derive(Clone, Debug)]
struct Corpus {
    path: PathBuf,
    tokens: Vec<u32>,
    mask: Vec<u8>,
    documents: Vec<Range<usize>>,
    runs: Vec<Run>,
    tokens_sha256: String,
    mask_sha256: String,
    manifest_sha256: String,
    split: String,
    cell_phrasing: Option<String>,
    cell_value: Option<String>,
    seed: Option<u64>,
    conversations: Option<usize>,
    tokenizer_sha256: String,
    max_tokens: usize,
}

#[derive(Clone, Copy, Debug, Default)]
struct Metadata {
    prompt_seen: Option<bool>,
    keyed: Option<bool>,
    reply_seen: Option<bool>,
    prompt_template_seen: Option<bool>,
    opening: bool,
}

#[derive(Clone, Debug)]
struct TokenizerCensus {
    identity: TokenizerIdentity,
    digits: HashSet<u32>,
}

#[derive(Clone, Debug)]
struct TemplateIndex {
    k: usize,
    keep: HashSet<u32>,
    prompts: RunIndex,
    replies: RunIndex,
}

#[derive(Clone, Debug)]
struct Target {
    prefix: Vec<u32>,
    target: u32,
    target_cell: u32,
    metadata: Metadata,
}

/// A compact index of variable-length token runs. Records point into one
/// contiguous arena and are sorted by the run's token sequence.
#[derive(Clone, Debug)]
struct RunIndex {
    data: Vec<u32>,
    records: Vec<(usize, usize)>,
}

impl RunIndex {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            records: Vec::new(),
        }
    }

    fn push(&mut self, tokens: &[u32]) -> usize {
        let ordinal = self.records.len();
        let offset = self.data.len();
        self.data.extend_from_slice(tokens);
        self.records.push((offset, tokens.len()));
        ordinal
    }

    fn finish(&mut self) {
        let data = &self.data;
        self.records.sort_unstable_by(|left, right| {
            let left = &data[left.0..left.0 + left.1];
            let right = &data[right.0..right.0 + right.1];
            left.cmp(right)
        });
    }

    fn contains(&self, tokens: &[u32]) -> bool {
        self.records
            .binary_search_by(|(offset, len)| self.data[*offset..*offset + *len].cmp(tokens))
            .is_ok()
    }
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn artifact_header(bytes: &[u8]) -> Result<serde_json::Value> {
    if bytes.len() < 16 || &bytes[..8] != b"UORLUT01" {
        return Err("artifact is not a canonical UORLUT01 file".to_owned());
    }
    let header_len = u64::from_le_bytes(
        bytes[8..16]
            .try_into()
            .map_err(|_| "artifact header length differs".to_owned())?,
    );
    let header_end = usize::try_from(header_len)
        .ok()
        .and_then(|length| length.checked_add(16))
        .filter(|&end| end <= bytes.len())
        .ok_or_else(|| "artifact header length is out of bounds".to_owned())?;
    serde_json::from_slice(&bytes[16..header_end])
        .map_err(|error| format!("artifact header: {error}"))
}

fn artifact_identity(
    path: &Path,
    bytes: &[u8],
    model: &IntegerDiscreteModel,
) -> Result<ArtifactIdentity> {
    let header = artifact_header(bytes)?;
    let schema = header["schema"]
        .as_str()
        .ok_or_else(|| "artifact schema differs".to_owned())?
        .to_owned();
    let tables = header["tables"]
        .as_array()
        .ok_or_else(|| "artifact tables differ".to_owned())?
        .len();
    let shape = model.shape();
    Ok(ArtifactIdentity {
        path: path.display().to_string(),
        sha256: sha256(bytes),
        model_sha256: model.artifact_sha256().to_owned(),
        schema,
        tables,
        vocabulary: shape.vocabulary,
        context: shape.context,
        blocks: shape.blocks,
        kept_cells: shape.kept_cells,
        read_mode: shape.read_mode.clone(),
    })
}

fn codebook_from_artifact(bytes: &[u8]) -> Result<Vec<i32>> {
    let header = artifact_header(bytes)?;
    let header_len = u64::from_le_bytes(
        bytes[8..16]
            .try_into()
            .map_err(|_| "artifact header length differs".to_owned())?,
    );
    let unaligned = 16usize
        .checked_add(
            usize::try_from(header_len)
                .map_err(|_| "artifact header length exceeds usize".to_owned())?,
        )
        .ok_or_else(|| "artifact data offset overflows".to_owned())?;
    let data_start = unaligned
        .checked_add(63)
        .ok_or_else(|| "artifact alignment overflows".to_owned())?
        / 64
        * 64;
    let table = header["tables"]
        .as_array()
        .and_then(|tables| {
            tables
                .iter()
                .find(|table| table["name"].as_str() == Some("discrete.table"))
        })
        .ok_or_else(|| "artifact has no discrete.table".to_owned())?;
    if table["kind"].as_str() != Some("i32")
        || table["len"].as_u64() != Some((VOCABULARY * READ_WIDTH) as u64)
    {
        return Err("artifact discrete.table is not the 4096 x 32 i32 codebook".to_owned());
    }
    let offset = usize::try_from(
        table["span"]["offset"]
            .as_u64()
            .ok_or_else(|| "artifact table offset differs".to_owned())?,
    )
    .map_err(|_| "artifact table offset exceeds usize".to_owned())?;
    let expected_bytes = VOCABULARY * READ_WIDTH * 4;
    if table["span"]["bytes"].as_u64() != Some(expected_bytes as u64) {
        return Err("artifact discrete.table byte span differs".to_owned());
    }
    let start = data_start
        .checked_add(offset)
        .ok_or_else(|| "artifact table start overflows".to_owned())?;
    let end = start
        .checked_add(expected_bytes)
        .ok_or_else(|| "artifact table end overflows".to_owned())?;
    let section = bytes
        .get(start..end)
        .ok_or_else(|| "artifact discrete.table span is out of bounds".to_owned())?;
    Ok(section
        .chunks_exact(4)
        .map(|word| i32::from_le_bytes(word.try_into().expect("four-byte chunk")))
        .collect())
}

#[derive(Deserialize)]
struct ManifestToken {
    vocab_size: usize,
    bos_id: u32,
    sha256: String,
}

#[derive(Deserialize)]
struct Manifest {
    schema: String,
    mask_schema: String,
    tokens: usize,
    tokens_bytes: usize,
    tokens_sha256: String,
    mask_bytes: usize,
    mask_sha256: String,
    response_tokens: usize,
    max_tokens: usize,
    split: String,
    #[serde(default)]
    cell: Option<serde_json::Value>,
    #[serde(default)]
    seed: Option<u64>,
    #[serde(default)]
    conversations: Option<usize>,
    tokenizer: ManifestToken,
}

fn parse_uort(token_bytes: &[u8], mask: &[u8], expected_vocabulary: usize) -> Result<Vec<u32>> {
    if token_bytes.len() < 64 || &token_bytes[..4] != b"UORT" {
        return Err("tokens.u16 is not a canonical UORT file".to_owned());
    }
    let version = u32::from_le_bytes(
        token_bytes[4..8]
            .try_into()
            .map_err(|_| "UORT version differs".to_owned())?,
    );
    let count = u64::from_le_bytes(
        token_bytes[8..16]
            .try_into()
            .map_err(|_| "UORT count differs".to_owned())?,
    );
    let vocabulary = u32::from_le_bytes(
        token_bytes[16..20]
            .try_into()
            .map_err(|_| "UORT vocabulary differs".to_owned())?,
    );
    if version != 1
        || count == 0
        || token_bytes[20..64].iter().any(|&byte| byte != 0)
        || usize::try_from(count)
            .ok()
            .and_then(|count| count.checked_mul(2))
            .and_then(|bytes| bytes.checked_add(64))
            != Some(token_bytes.len())
        || usize::try_from(count).ok() != Some(mask.len())
        || usize::try_from(vocabulary).ok() != Some(expected_vocabulary)
    {
        return Err("UORT header or payload identity differs".to_owned());
    }
    Ok(token_bytes[64..]
        .chunks_exact(2)
        .map(|bytes| u32::from(u16::from_le_bytes([bytes[0], bytes[1]])))
        .collect())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn read_tokenizer(path: &Path) -> Result<TokenizerCensus> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes).is_none() {
        return Err("tokenizer JSON is not a supported byte-level BPE tokenizer".to_owned());
    }
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|error| format!("tokenizer: {error}"))?;
    let vocab = value["model"]["vocab"]
        .as_object()
        .ok_or_else(|| "tokenizer JSON has no model.vocab".to_owned())?;
    let digits: HashSet<u32> = vocab
        .iter()
        .filter(|(piece, _)| {
            let bare = piece.strip_prefix('\u{120}').unwrap_or(piece);
            bare.len() == 1 && bare.as_bytes()[0].is_ascii_digit()
        })
        .filter_map(|(_, id)| id.as_u64().and_then(|id| u32::try_from(id).ok()))
        .collect();
    Ok(TokenizerCensus {
        identity: TokenizerIdentity {
            path: path.display().to_string(),
            sha256: sha256(&bytes),
            vocabulary: vocab.len(),
            digit_tokens: digits.len(),
            mask_ks: CENSUS_MASK_KS.to_vec(),
        },
        digits,
    })
}

fn bind_tokenizer(corpus: &Corpus, tokenizer: &TokenizerCensus) -> Result<()> {
    if corpus.tokenizer_sha256 != tokenizer.identity.sha256 {
        return Err(format!(
            "{}: manifest tokenizer sha256 differs from {}",
            corpus.path.display(),
            tokenizer.identity.path
        ));
    }
    Ok(())
}

fn validate_reference_role(corpus: &Corpus, role: ReferenceRole) -> Result<()> {
    if role == ReferenceRole::CanonicalTrain && corpus.split != "train" {
        return Err(format!(
            "canonical_train reference manifest split is {:?}, not \"train\"",
            corpus.split
        ));
    }
    Ok(())
}

fn read_corpus(files: &CorpusFiles, context: usize, vocabulary: usize, bos: u32) -> Result<Corpus> {
    let token_bytes =
        fs::read(&files.tokens).map_err(|e| format!("{}: {e}", files.tokens.display()))?;
    let mask = fs::read(&files.mask).map_err(|e| format!("{}: {e}", files.mask.display()))?;
    let manifest =
        fs::read(&files.manifest).map_err(|e| format!("{}: {e}", files.manifest.display()))?;
    let identity: Manifest =
        serde_json::from_slice(&manifest).map_err(|error| format!("manifest: {error}"))?;
    if identity.schema != "uor-r4-chat-corpus/v1"
        || identity.mask_schema != "uor-r4-response-mask/u8/v1"
        || identity.split.is_empty()
        || identity.tokenizer.vocab_size != vocabulary
        || identity.tokenizer.bos_id != bos
        || identity.tokens_bytes != token_bytes.len()
        || identity.mask_bytes != mask.len()
        || identity.response_tokens != mask.iter().filter(|&&m| m == 1).count()
        || identity.tokens_sha256 != sha256(&token_bytes)
        || identity.mask_sha256 != sha256(&mask)
        || !valid_sha256(&identity.tokenizer.sha256)
    {
        return Err(format!(
            "{}: corpus manifest identity or artifact vocabulary/BOS differs",
            files.manifest.display()
        ));
    }
    let tokens = parse_uort(&token_bytes, &mask, vocabulary)?;
    if identity.tokens != tokens.len()
        || identity.tokens == 0
        || identity.max_tokens < context
        || mask.iter().any(|&value| value > 1)
        || tokens.iter().any(|&token| token as usize >= vocabulary)
    {
        return Err("corpus manifest inventory differs from UORT payload".to_owned());
    }
    let starts: Vec<usize> = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, &token)| (token == bos).then_some(index))
        .collect();
    if starts.is_empty() {
        return Err("corpus has no BOS-delimited documents".to_owned());
    }
    if starts[0] != 0 {
        return Err("corpus does not begin with BOS".to_owned());
    }
    let documents: Vec<Range<usize>> = starts
        .iter()
        .enumerate()
        .map(|(index, &start)| start..starts.get(index + 1).copied().unwrap_or(tokens.len()))
        .collect();
    let runs = collect_runs(&tokens, &mask, &documents, bos);
    if !runs.iter().any(|run| run.mask == 0) {
        return Err(format!(
            "{}: corpus has no prompt runs",
            files.manifest.display()
        ));
    }
    if !runs.iter().any(|run| run.mask == 1) {
        return Err(format!(
            "{}: corpus has no response runs",
            files.manifest.display()
        ));
    }
    if identity
        .conversations
        .is_some_and(|conversations| conversations != documents.len())
    {
        return Err("corpus manifest conversation count differs".to_owned());
    }
    if !runs.iter().any(|run| run.mask == 1) {
        return Err("corpus has no response runs".to_owned());
    }
    let (cell_phrasing, cell_value) = identity
        .cell
        .as_ref()
        .map(|cell| {
            (
                cell["phrasing"].as_str().map(str::to_owned),
                cell["value"].as_str().map(str::to_owned),
            )
        })
        .unwrap_or((None, None));
    Ok(Corpus {
        path: files
            .manifest
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf(),
        tokens,
        mask,
        documents,
        runs,
        tokens_sha256: identity.tokens_sha256,
        mask_sha256: identity.mask_sha256,
        manifest_sha256: sha256(&manifest),
        split: identity.split,
        cell_phrasing,
        cell_value,
        seed: identity.seed,
        conversations: identity.conversations,
        tokenizer_sha256: identity.tokenizer.sha256,
        max_tokens: identity.max_tokens,
    })
}

fn collect_runs(tokens: &[u32], mask: &[u8], documents: &[Range<usize>], bos: u32) -> Vec<Run> {
    let mut runs = Vec::new();
    for (document, span) in documents.iter().enumerate() {
        let mut start = span.start + usize::from(tokens.get(span.start) == Some(&bos));
        while start < span.end {
            let value = mask[start];
            let mut end = start + 1;
            while end < span.end && mask[end] == value {
                end += 1;
            }
            runs.push(Run {
                ordinal: runs.len(),
                document,
                mask: value,
                range: start..end,
            });
            start = end;
        }
    }
    runs
}

fn corpus_identity(corpus: &Corpus, reference_role: Option<ReferenceRole>) -> CorpusIdentity {
    CorpusIdentity {
        path: corpus.path.display().to_string(),
        manifest_sha256: corpus.manifest_sha256.clone(),
        split: corpus.split.clone(),
        cell_phrasing: corpus.cell_phrasing.clone(),
        cell_value: corpus.cell_value.clone(),
        seed: corpus.seed,
        conversations: corpus.conversations,
        tokenizer_sha256: corpus.tokenizer_sha256.clone(),
        tokens_sha256: corpus.tokens_sha256.clone(),
        mask_sha256: corpus.mask_sha256.clone(),
        documents: corpus.documents.len(),
        tokens: corpus.tokens.len(),
        response_targets: corpus.mask.iter().filter(|&&m| m == 1).count(),
        prompt_runs: corpus.runs.iter().filter(|run| run.mask == 0).count(),
        reply_runs: corpus.runs.iter().filter(|run| run.mask == 1).count(),
        max_tokens: corpus.max_tokens,
        reference_role,
    }
}

fn prompt_run_before(corpus: &Corpus, reply: usize) -> Option<&Run> {
    let reply = corpus.runs.get(reply)?;
    corpus.runs[..reply.ordinal]
        .iter()
        .rev()
        .find(|run| run.document == reply.document && run.mask == 0)
}

fn run_tokens<'a>(corpus: &'a Corpus, run: &Run) -> &'a [u32] {
    &corpus.tokens[run.range.clone()]
}

fn run_index(corpus: &Corpus, mask: u8) -> RunIndex {
    let mut index = RunIndex::new();
    for run in corpus.runs.iter().filter(|run| run.mask == mask) {
        index.push(run_tokens(corpus, run));
    }
    index.finish();
    index
}

fn top_tokens(corpus: &Corpus, k: usize) -> HashSet<u32> {
    let mut counts = HashMap::<u32, usize>::new();
    for &token in corpus.tokens.iter().filter(|&&token| token != 0) {
        *counts.entry(token).or_default() += 1;
    }
    let mut ranked: Vec<(u32, usize)> = counts.into_iter().collect();
    ranked.sort_unstable_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(&right.0)));
    ranked.into_iter().take(k).map(|(token, _)| token).collect()
}

fn value_mask(run: &[u32], keep: &HashSet<u32>, digits: &HashSet<u32>) -> Vec<u32> {
    let mut masked = Vec::with_capacity(run.len());
    for &token in run {
        let id = if keep.contains(&token) && !digits.contains(&token) {
            token
        } else {
            CENSUS_MASK
        };
        if id != CENSUS_MASK || masked.last() != Some(&CENSUS_MASK) {
            masked.push(id);
        }
    }
    masked
}

fn masked_template_seen(
    index: &RunIndex,
    tokenizer: &TokenizerCensus,
    tokens: &[u32],
    keep: &HashSet<u32>,
) -> bool {
    index.contains(&value_mask(tokens, keep, &tokenizer.digits))
}

fn template_indexes(corpus: &Corpus, tokenizer: &TokenizerCensus) -> Vec<TemplateIndex> {
    CENSUS_MASK_KS
        .into_iter()
        .map(|k| {
            let keep = top_tokens(corpus, k);
            let build = |mask: u8| {
                let mut index = RunIndex::new();
                for run in corpus.runs.iter().filter(|run| run.mask == mask) {
                    index.push(&value_mask(
                        run_tokens(corpus, run),
                        &keep,
                        &tokenizer.digits,
                    ));
                }
                index.finish();
                index
            };
            let prompts = build(0);
            let replies = build(1);
            TemplateIndex {
                k,
                keep,
                prompts,
                replies,
            }
        })
        .collect()
}

fn template_overlap(
    panel: &Corpus,
    templates: &[TemplateIndex],
    tokenizer: &TokenizerCensus,
) -> TemplateOverlap {
    let prompt_runs: Vec<&Run> = panel.runs.iter().filter(|run| run.mask == 0).collect();
    let reply_runs: Vec<&Run> = panel.runs.iter().filter(|run| run.mask == 1).collect();
    let mut prompt_template = Vec::with_capacity(templates.len());
    let mut reply_template = Vec::with_capacity(templates.len());
    for template in templates {
        let prompts_seen = prompt_runs
            .iter()
            .filter(|run| {
                masked_template_seen(
                    &template.prompts,
                    tokenizer,
                    run_tokens(panel, run),
                    &template.keep,
                )
            })
            .count();
        let replies_seen = reply_runs
            .iter()
            .filter(|run| {
                masked_template_seen(
                    &template.replies,
                    tokenizer,
                    run_tokens(panel, run),
                    &template.keep,
                )
            })
            .count();
        prompt_template.push(TemplateShare::new(
            template.k,
            prompt_runs.len() as u64,
            prompts_seen as u64,
        ));
        reply_template.push(TemplateShare::new(
            template.k,
            reply_runs.len() as u64,
            replies_seen as u64,
        ));
    }
    let new_prompt_templates_at_1024 = prompt_template
        .iter()
        .find(|reading| reading.k == 1_024)
        .is_some_and(|reading| {
            reading
                .seen
                .checked_mul(2)
                .is_some_and(|seen_twice| seen_twice <= reading.runs)
        });
    TemplateOverlap {
        prompts: prompt_runs.len() as u64,
        replies: reply_runs.len() as u64,
        prompt_template,
        reply_template,
        new_prompt_templates_at_1024,
    }
}

fn metadata_for(
    panel: &Corpus,
    reply: &Run,
    prompts: Option<&RunIndex>,
    keys: Option<&RunIndex>,
    replies: Option<&RunIndex>,
    templates: Option<(&TemplateIndex, &TokenizerCensus)>,
) -> Metadata {
    let prompt = prompt_run_before(panel, reply.ordinal);
    let prompt_tokens = prompt.map(|run| run_tokens(panel, run));
    let key_len = prompt_tokens
        .map(|tokens| tokens.len().min(OPENING_KEY_TOKENS))
        .unwrap_or(0);
    Metadata {
        prompt_seen: prompts
            .zip(prompt_tokens)
            .map(|(index, tokens)| index.contains(tokens)),
        keyed: keys
            .zip(prompt_tokens)
            .map(|(index, tokens)| index.contains(&tokens[..key_len])),
        reply_seen: replies.map(|index| index.contains(run_tokens(panel, reply))),
        prompt_template_seen: templates
            .zip(prompt_tokens)
            .map(|((index, tokenizer), tokens)| {
                masked_template_seen(&index.prompts, tokenizer, tokens, &index.keep)
            }),
        opening: true,
    }
}

fn targets_in(
    corpus: &Corpus,
    context: usize,
    cells: &[u32],
    reply_metadata: &[Metadata],
) -> Vec<Target> {
    let mut targets = Vec::with_capacity(corpus.mask.iter().filter(|&&mask| mask == 1).count());
    for (reply_ordinal, run) in corpus.runs.iter().filter(|run| run.mask == 1).enumerate() {
        let document = &corpus.documents[run.document];
        for target_position in run.range.clone() {
            if corpus.tokens[target_position] == 0 {
                continue;
            }
            let mut metadata = reply_metadata
                .get(reply_ordinal)
                .copied()
                .unwrap_or_default();
            metadata.opening = target_position == run.range.start;
            let prefix_start = target_position.saturating_sub(context).max(document.start);
            targets.push(Target {
                prefix: corpus.tokens[prefix_start..target_position].to_vec(),
                target: corpus.tokens[target_position],
                target_cell: cells[corpus.tokens[target_position] as usize],
                metadata,
            });
        }
    }
    targets
}

fn evenly_cap<T: Clone>(values: Vec<T>, maximum: Option<usize>) -> Vec<T> {
    let Some(maximum) = maximum else {
        return values;
    };
    if values.len() <= maximum || maximum == 0 {
        return values.into_iter().take(maximum).collect();
    }
    (0..maximum)
        .map(|index| values[index * values.len() / maximum].clone())
        .collect()
}

fn add_stratum(strata: &mut BTreeMap<String, Reading>, name: impl Into<String>, reading: &Reading) {
    strata.entry(name.into()).or_default().merge(reading);
}

fn observe_writer_parity(
    writer: &CpuWriterCore,
    evaluation: &uor_r4_integer::discrete_d11::DiscretePrefixEvaluation,
    writer_parity: &mut WriterParity,
) -> Result<()> {
    let query: &[i32; READ_WIDTH] = evaluation.read_vector.as_slice().try_into().map_err(|_| {
        format!(
            "serving query has {} coordinates, not {READ_WIDTH}",
            evaluation.read_vector.len()
        )
    })?;
    let decision = writer.decide(query);
    let admitted_cells = decision.admitted[..decision.admitted_count]
        .iter()
        .copied()
        .filter(|&cell| cell != ABSENT_CELL)
        .map(u32::from)
        .collect::<Vec<_>>();
    let mut served_top8 = [u32::MAX; 8];
    for (slot, &token) in evaluation.top8.iter().take(8).enumerate() {
        served_top8[slot] = token;
    }
    writer_parity.rows += 1;
    writer_parity.admitted_cell_mismatches +=
        u64::from(admitted_cells != evaluation.admitted_cells);
    writer_parity.top8_mismatches += u64::from(decision.top8 != served_top8);
    writer_parity.token_mismatches += u64::from(decision.top8[0] != evaluation.prediction);
    Ok(())
}

fn observe_cpu_engine_parity(
    artifact: &ArtifactIdentity,
    artifact_bytes: &[u8],
    targets: &[Target],
) -> Result<CpuEngineParity> {
    if artifact.schema != "uor-r4.lut-discrete/1" || artifact.tables != 4 {
        return Ok(CpuEngineParity::unavailable(format!(
            "artifact schema {} with {} tables is outside the four-table CPU discrete read implementation",
            artifact.schema, artifact.tables
        )));
    }
    let samples = targets
        .iter()
        .map(|target| CpuDiscreteProposalSample {
            prefix: target.prefix.clone(),
            target: target.target,
        })
        .collect::<Vec<_>>();
    let scorer = CpuDiscreteProposalScorer::from_bytes(artifact_bytes)
        .map_err(|error| format!("independent CPU discrete engine: {error}"))?;
    let reading = scorer
        .export_parity(artifact_bytes, &samples)
        .map_err(|error| format!("independent CPU discrete parity: {error}"))?;
    let parity = CpuEngineParity::from_reading(reading);
    if !parity.is_exact() {
        return Err(format!(
            "independent CPU discrete engine differs from integer serving: {parity:?}"
        ));
    }
    Ok(parity)
}

fn observe_targets<F>(
    artifact: &ArtifactIdentity,
    targets: &[Target],
    writer: Option<&CpuWriterCore>,
    writer_parity: &mut WriterParity,
    mut predict: F,
) -> Result<(Reading, BTreeMap<String, Reading>)>
where
    F: FnMut(&[u32]) -> Result<uor_r4_integer::discrete_d11::DiscretePrefixEvaluation>,
{
    let mut overall = Reading::default();
    let mut strata = BTreeMap::new();
    for (index, sample) in targets.iter().enumerate() {
        let evaluation = predict(&sample.prefix)?;
        let admitted = evaluation.admitted_cells.contains(&sample.target_cell);
        let top1 = evaluation.prediction == sample.target;
        let top8 = evaluation.top8.contains(&sample.target);
        let mut row = Reading::default();
        row.observe(admitted, top1, top8);
        overall.merge(&row);
        add_stratum(
            &mut strata,
            if sample.metadata.opening {
                "reply-position:opening"
            } else {
                "reply-position:continuation"
            },
            &row,
        );
        for (name, value) in [
            ("prompt-overlap", sample.metadata.prompt_seen),
            ("prompt-template", sample.metadata.prompt_template_seen),
            ("memory-key", sample.metadata.keyed),
            ("reference-reply-overlap", sample.metadata.reply_seen),
        ] {
            let value = match value {
                Some(true) => "present",
                Some(false) => "absent",
                None => "unavailable",
            };
            add_stratum(&mut strata, format!("{name}:{value}"), &row);
        }
        if let Some(writer) = writer {
            observe_writer_parity(writer, &evaluation, writer_parity)?;
        }
        if (index + 1) % 1000 == 0 {
            eprintln!(
                "{index} of {} targets evaluated for {}",
                targets.len(),
                artifact.sha256
            );
        }
    }
    Ok((overall, strata))
}

#[allow(clippy::too_many_arguments)]
fn evaluate_model(
    artifact_bytes: &[u8],
    artifact: &ArtifactIdentity,
    model: &IntegerDiscreteModel,
    corpus: &Corpus,
    train: Option<&Corpus>,
    tokenizer: &TokenizerCensus,
    reference_role: Option<ReferenceRole>,
    max_targets: Option<usize>,
) -> Result<EvidenceReport> {
    if max_targets == Some(0) {
        return Err("max-targets must be positive".to_owned());
    }
    let shape = model.shape();
    let prompt_index = train.map(|train| run_index(train, 0));
    let reply_index = train.map(|train| run_index(train, 1));
    let key_index = train.map(|train| {
        let mut index = RunIndex::new();
        for run in train.runs.iter().filter(|run| run.mask == 0) {
            let tokens = run_tokens(train, run);
            index.push(&tokens[..tokens.len().min(OPENING_KEY_TOKENS)]);
        }
        index.finish();
        index
    });
    let templates = train.map(|train| template_indexes(train, tokenizer));
    let template_1024 = templates
        .as_ref()
        .and_then(|templates| templates.iter().find(|template| template.k == 1_024));

    let reply_runs: Vec<&Run> = corpus.runs.iter().filter(|run| run.mask == 1).collect();
    let metadata: Vec<Metadata> = reply_runs
        .iter()
        .map(|run| {
            metadata_for(
                corpus,
                run,
                prompt_index.as_ref(),
                key_index.as_ref(),
                reply_index.as_ref(),
                template_1024.map(|template| (template, tokenizer)),
            )
        })
        .collect();
    let targets = evenly_cap(
        targets_in(corpus, shape.context as usize, model.cells(), &metadata),
        max_targets,
    );
    if targets.is_empty() {
        return Err("panel has no measured response targets".to_owned());
    }
    let cpu_engine_parity = observe_cpu_engine_parity(artifact, artifact_bytes, &targets)?;
    let (writer, mut writer_parity) = if shape.vocabulary as usize == VOCABULARY
        && shape.blocks as usize == READ_WIDTH / 8
        && (1..=16).contains(&shape.kept_cells)
    {
        let codebook = codebook_from_artifact(artifact_bytes)?;
        let writer = CpuWriterCore::from_codebook(&codebook, shape.kept_cells as usize)
            .map_err(|error| format!("independent CPU writer: {error}"))?;
        (Some(writer), WriterParity::from_serving_query())
    } else {
        (
            None,
            WriterParity::unavailable(format!(
                "unsupported CPU writer shape: vocabulary {}, blocks {}, kept {}",
                shape.vocabulary, shape.blocks, shape.kept_cells
            )),
        )
    };
    let (overall, strata) = observe_targets(
        artifact,
        &targets,
        writer.as_ref(),
        &mut writer_parity,
        |prefix| model.evaluate_prefix(prefix).map_err(|e| e.to_string()),
    )?;
    if writer_parity.rows > 0 && !writer_parity.is_exact() {
        return Err(format!(
            "independent CPU writer differs from integer serving: {writer_parity:?}"
        ));
    }

    let mut unavailable = BTreeMap::new();
    unavailable.insert(
        "generated_complete_reply".to_owned(),
        "requires a sealed/generated reply file; corpus rows are teacher-forced targets".to_owned(),
    );
    unavailable.insert(
        "generated_reply_novelty".to_owned(),
        "requires model-generated replies; the report measures reference and panel overlap only"
            .to_owned(),
    );
    if let Some(reason) = &cpu_engine_parity.unavailable {
        unavailable.insert("independent_cpu_trunk_query".to_owned(), reason.clone());
    }
    unavailable.insert(
        "count_table_control".to_owned(),
        "no count-table arm is supplied to this evidence pass".to_owned(),
    );
    unavailable.insert(
        "memory_off_control".to_owned(),
        "no serving-session memory-off arm is supplied to this evidence pass".to_owned(),
    );
    unavailable.insert(
        "no_read_control".to_owned(),
        "no model read-ablation arm is supplied to this evidence pass".to_owned(),
    );
    let reply_overlap = train.map(|train| reply_overlap(corpus, train));
    let template_overlap = train
        .zip(templates.as_ref())
        .map(|(_, templates)| template_overlap(corpus, templates, tokenizer));
    Ok(EvidenceReport {
        schema: SCHEMA.to_owned(),
        artifact: artifact.clone(),
        tokenizer: tokenizer.identity.clone(),
        panel: corpus_identity(corpus, None),
        train: train.map(|train| corpus_identity(train, reference_role)),
        measured_targets: targets.len(),
        overall,
        strata,
        reply_overlap,
        template_overlap,
        writer_parity,
        cpu_engine_parity,
        unavailable,
    })
}

/// Evaluate a panel. `artifact` is a discrete UORLUT01 file. Every corpus names
/// its `tokens.u16`, `response_mask.u8` and `manifest.json` explicitly. When a
/// reference corpus is supplied, `reference_role` must distinguish canonical
/// TRAIN from a generated control.
pub fn evaluate(
    artifact_path: &Path,
    tokenizer_path: &Path,
    panel: &CorpusFiles,
    train: Option<&CorpusFiles>,
    reference_role: Option<ReferenceRole>,
    max_targets: Option<usize>,
) -> Result<EvidenceReport> {
    if train.is_some() != reference_role.is_some() {
        return Err("train inputs and train-role must be supplied together".to_owned());
    }
    if max_targets == Some(0) {
        return Err("max-targets must be positive".to_owned());
    }
    let artifact_bytes =
        fs::read(artifact_path).map_err(|e| format!("{}: {e}", artifact_path.display()))?;
    let model = IntegerDiscreteModel::from_bytes(&artifact_bytes).map_err(|e| e.to_string())?;
    let shape = model.shape();
    let tokenizer = read_tokenizer(tokenizer_path)?;
    if tokenizer.identity.vocabulary != shape.vocabulary as usize {
        return Err(format!(
            "tokenizer vocabulary {} differs from artifact vocabulary {}",
            tokenizer.identity.vocabulary, shape.vocabulary
        ));
    }
    let manifest =
        fs::read(&panel.manifest).map_err(|e| format!("{}: {e}", panel.manifest.display()))?;
    let parsed: serde_json::Value =
        serde_json::from_slice(&manifest).map_err(|e| format!("manifest: {e}"))?;
    let bos = parsed["tokenizer"]["bos_id"]
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| "manifest tokenizer.bos_id differs".to_owned())?;
    let corpus = read_corpus(
        panel,
        shape.context as usize,
        shape.vocabulary as usize,
        bos,
    )?;
    bind_tokenizer(&corpus, &tokenizer)?;
    let identity = artifact_identity(artifact_path, &artifact_bytes, &model)?;
    let train = train
        .map(|files| {
            read_corpus(
                files,
                shape.context as usize,
                shape.vocabulary as usize,
                bos,
            )
        })
        .transpose()?;
    if let Some(train) = &train {
        bind_tokenizer(train, &tokenizer)?;
        validate_reference_role(train, reference_role.expect("checked with train"))?;
    }
    let report = evaluate_model(
        &artifact_bytes,
        &identity,
        &model,
        &corpus,
        train.as_ref(),
        &tokenizer,
        reference_role,
        max_targets,
    )?;
    Ok(report)
}

fn ngram_index(corpus: &Corpus, n: usize) -> RunIndex {
    let mut index = RunIndex::new();
    for run in corpus.runs.iter().filter(|run| run.mask == 1) {
        let tokens = run_tokens(corpus, run);
        for window in tokens.windows(n) {
            index.push(window);
        }
    }
    index.finish();
    index
}

fn reply_overlap(panel: &Corpus, train: &Corpus) -> ReplyOverlap {
    let prompts = run_index(train, 0);
    let replies = run_index(train, 1);
    let mut keys = RunIndex::new();
    for run in train.runs.iter().filter(|run| run.mask == 0) {
        let tokens = run_tokens(train, run);
        keys.push(&tokens[..tokens.len().min(OPENING_KEY_TOKENS)]);
    }
    keys.finish();
    let n4 = ngram_index(train, 4);
    let n8 = ngram_index(train, 8);
    let mut reading = ReplyOverlap {
        reference_replies: 0,
        reference_replies_seen_in_reference: 0,
        reference_prompts_seen_in_reference: 0,
        reference_prompts_keyed_in_reference: 0,
        reference_reply_new_4grams: 0,
        reference_reply_total_4grams: 0,
        reference_reply_new_8grams: 0,
        reference_reply_total_8grams: 0,
    };
    for reply in panel.runs.iter().filter(|run| run.mask == 1) {
        let reply_tokens = run_tokens(panel, reply);
        reading.reference_replies += 1;
        reading.reference_replies_seen_in_reference += u64::from(replies.contains(reply_tokens));
        if let Some(prompt) = prompt_run_before(panel, reply.ordinal) {
            let prompt_tokens = run_tokens(panel, prompt);
            reading.reference_prompts_seen_in_reference +=
                u64::from(prompts.contains(prompt_tokens));
            reading.reference_prompts_keyed_in_reference += u64::from(
                keys.contains(&prompt_tokens[..prompt_tokens.len().min(OPENING_KEY_TOKENS)]),
            );
        }
        for window in reply_tokens.windows(4) {
            reading.reference_reply_total_4grams += 1;
            reading.reference_reply_new_4grams += u64::from(!n4.contains(window));
        }
        for window in reply_tokens.windows(8) {
            reading.reference_reply_total_8grams += 1;
            reading.reference_reply_new_8grams += u64::from(!n8.contains(window));
        }
    }
    reading
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEMP: AtomicU64 = AtomicU64::new(1);

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "uor-m2-evidence-{name}-{}-{}",
                std::process::id(),
                NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn uort(tokens: &[u32], vocabulary: u32) -> Vec<u8> {
        let mut bytes = vec![0u8; 64];
        bytes[..4].copy_from_slice(b"UORT");
        bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
        bytes[8..16].copy_from_slice(&(tokens.len() as u64).to_le_bytes());
        bytes[16..20].copy_from_slice(&vocabulary.to_le_bytes());
        for token in tokens {
            bytes.extend(u16::try_from(*token).unwrap().to_le_bytes());
        }
        bytes
    }

    fn written_corpus(name: &str, tokens: &[u32], mask: &[u8]) -> (TempDir, CorpusFiles) {
        let directory = TempDir::new(name);
        let token_bytes = uort(tokens, 4096);
        let manifest = serde_json::json!({
            "schema": "uor-r4-chat-corpus/v1",
            "mask_schema": "uor-r4-response-mask/u8/v1",
            "split": "train",
            "cell": {"phrasing": "train", "value": "train"},
            "seed": 7,
            "conversations": 1,
            "tokens": tokens.len(),
            "tokens_bytes": token_bytes.len(),
            "tokens_sha256": sha256(&token_bytes),
            "mask_bytes": mask.len(),
            "mask_sha256": sha256(mask),
            "response_tokens": mask.iter().filter(|&&value| value == 1).count(),
            "max_tokens": 192,
            "tokenizer": {
                "vocab_size": 4096,
                "bos_id": 0,
                "sha256": "ab".repeat(32),
            },
        });
        let files = CorpusFiles {
            tokens: directory.0.join("tokens.u16"),
            mask: directory.0.join("response_mask.u8"),
            manifest: directory.0.join("manifest.json"),
        };
        fs::write(&files.tokens, token_bytes).unwrap();
        fs::write(&files.mask, mask).unwrap();
        fs::write(
            &files.manifest,
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        (directory, files)
    }

    fn written_tokenizer(name: &str) -> (TempDir, TokenizerCensus) {
        let directory = TempDir::new(name);
        let path = directory.0.join("tokenizer.json");
        let tokenizer = serde_json::json!({
            "model": {
                "type": "BPE",
                "vocab": {
                    "<|bos|>": 0,
                    "a": 1,
                    "b": 2,
                    "hello": 3,
                    "0": 4,
                    "\u{120}1": 5
                },
                "merges": []
            },
            "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": true}
        });
        fs::write(&path, serde_json::to_vec(&tokenizer).unwrap()).unwrap();
        let census = read_tokenizer(&path).unwrap();
        (directory, census)
    }

    fn corpus(tokens: &[u32], mask: &[u8]) -> Corpus {
        let documents: Vec<Range<usize>> = tokens
            .iter()
            .enumerate()
            .filter_map(|(index, &token)| (token == 0).then_some(index))
            .enumerate()
            .map(|(index, start)| {
                let starts: Vec<_> = tokens
                    .iter()
                    .enumerate()
                    .filter_map(|(index, &token)| (token == 0).then_some(index))
                    .collect();
                start..starts.get(index + 1).copied().unwrap_or(tokens.len())
            })
            .collect();
        let runs = collect_runs(tokens, mask, &documents, 0);
        Corpus {
            path: PathBuf::from("fixture"),
            tokens: tokens.to_vec(),
            mask: mask.to_vec(),
            documents,
            runs,
            tokens_sha256: "00".repeat(32),
            mask_sha256: "11".repeat(32),
            manifest_sha256: "22".repeat(32),
            split: "train".to_owned(),
            cell_phrasing: Some("train".to_owned()),
            cell_value: Some("train".to_owned()),
            seed: Some(7),
            conversations: Some(1),
            tokenizer_sha256: "33".repeat(32),
            max_tokens: 32,
        }
    }

    #[test]
    fn run_indices_are_exact_and_lexicographic() {
        let mut index = RunIndex::new();
        index.push(&[3, 4]);
        index.push(&[1, 2, 3]);
        index.push(&[1, 2]);
        index.finish();
        assert!(index.contains(&[1, 2]));
        assert!(index.contains(&[1, 2, 3]));
        assert!(index.contains(&[3, 4]));
        assert!(!index.contains(&[2]));
        assert!(!index.contains(&[3]));
    }

    #[test]
    fn reply_and_prompt_metadata_use_the_immediately_prior_prompt() {
        let panel = corpus(
            &[0, 10, 11, 12, 20, 21, 22, 0, 30, 31, 32],
            &[0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0],
        );
        let prompts = run_index(&panel, 0);
        let replies = run_index(&panel, 1);
        let reply = panel.runs.iter().find(|run| run.mask == 1).unwrap();
        let prompt = panel.runs.iter().find(|run| run.mask == 0).unwrap();
        assert_eq!(run_tokens(&panel, prompt), [10, 11, 12]);
        let metadata = metadata_for(
            &panel,
            reply,
            Some(&prompts),
            Some(&prompts),
            Some(&replies),
            None,
        );
        assert_eq!(metadata.prompt_seen, Some(true));
        assert_eq!(metadata.keyed, Some(true));
        assert_eq!(metadata.reply_seen, Some(true));

        let successor = corpus(
            &[0, 10, 11, 20, 21, 30, 31, 40, 41],
            &[0, 0, 0, 1, 1, 0, 0, 1, 1],
        );
        let prompts = successor
            .runs
            .iter()
            .filter(|run| run.mask == 0)
            .map(|run| run_tokens(&successor, run).to_vec())
            .collect::<Vec<_>>();
        assert_eq!(prompts, vec![vec![10, 11], vec![30, 31]]);
    }

    #[test]
    fn target_cap_is_even_and_deterministic() {
        let values = (0..10).collect::<Vec<_>>();
        assert_eq!(evenly_cap(values.clone(), Some(4)), vec![0, 2, 5, 7]);
        assert_eq!(evenly_cap(values.clone(), Some(0)), Vec::<i32>::new());
        assert_eq!(evenly_cap(values, None).len(), 10);
    }

    #[test]
    fn reply_overlap_separates_seen_and_novel_windows() {
        let train = corpus(
            &[0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20],
            &[0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        );
        let panel = corpus(
            &[0, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20],
            &[0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        );
        let reading = reply_overlap(&panel, &train);
        assert_eq!(reading.reference_replies_seen_in_reference, 1);
        assert_eq!(reading.reference_reply_new_4grams, 0);
        assert_eq!(reading.reference_reply_new_8grams, 0);
        assert_eq!(reading.reference_prompts_keyed_in_reference, 1);
    }

    #[test]
    fn value_masks_follow_the_census_rule_and_collapse_adjacent_values() {
        let (_directory, tokenizer) = written_tokenizer("value-mask");
        assert_eq!(tokenizer.digits, HashSet::from([4, 5]));
        assert_eq!(
            value_mask(
                &[1, 4, 5, 2, 6, 7, 1],
                &HashSet::from([1, 2]),
                &tokenizer.digits,
            ),
            vec![1, CENSUS_MASK, 2, CENSUS_MASK, 1]
        );
    }

    #[test]
    fn value_masked_templates_match_reworded_values_but_not_other_orders() {
        let (_directory, tokenizer) = written_tokenizer("template-mask");
        let keep = HashSet::from([10, 20]);
        let mut prompts = RunIndex::new();
        prompts.push(&value_mask(&[10, 100, 20], &keep, &tokenizer.digits));
        prompts.finish();
        assert!(masked_template_seen(
            &prompts,
            &tokenizer,
            &[10, 999, 20],
            &keep
        ));
        assert!(!masked_template_seen(
            &prompts,
            &tokenizer,
            &[10, 20, 999],
            &keep
        ));
    }

    #[test]
    fn tokenizer_must_exist_be_bpe_style_and_match_both_manifests() {
        let missing = std::env::temp_dir().join(format!(
            "missing-tokenizer-{}-{}",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        assert!(read_tokenizer(&missing).is_err());

        let directory = TempDir::new("tokenizer-no-vocab");
        let path = directory.0.join("tokenizer.json");
        fs::write(&path, br#"{"model":{}}"#).unwrap();
        assert!(read_tokenizer(&path)
            .unwrap_err()
            .contains("supported byte-level BPE"));

        let (_directory, tokenizer) = written_tokenizer("tokenizer-binding");
        let mut panel = corpus(&[0, 10, 11, 20, 21], &[0, 0, 0, 1, 1]);
        assert!(bind_tokenizer(&panel, &tokenizer).is_err());
        panel.tokenizer_sha256 = tokenizer.identity.sha256.clone();
        bind_tokenizer(&panel, &tokenizer).unwrap();
    }

    #[test]
    fn canonical_train_role_requires_a_train_manifest() {
        let mut reference = corpus(&[0, 10, 20], &[0, 0, 1]);
        validate_reference_role(&reference, ReferenceRole::CanonicalTrain).unwrap();
        reference.split = "dev".to_owned();
        assert!(
            validate_reference_role(&reference, ReferenceRole::CanonicalTrain)
                .unwrap_err()
                .contains("not \"train\"")
        );
        validate_reference_role(&reference, ReferenceRole::GeneratedControl).unwrap();
    }

    #[test]
    fn malformed_uort_and_manifest_identity_are_refused() {
        assert!(parse_uort(b"not-uort", &[], 4096).is_err());
        let empty = uort(&[], 4096);
        assert!(parse_uort(&empty, &[], 4096).is_err());
        let wrong_vocabulary = uort(&[0, 1], 4095);
        assert!(parse_uort(&wrong_vocabulary, &[0, 1], 4096).is_err());

        let (_directory, files) = written_corpus("manifest-mismatch", &[0, 10, 20], &[0, 0, 1]);
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(&files.manifest).unwrap()).unwrap();
        manifest["mask_sha256"] = serde_json::json!("00".repeat(32));
        fs::write(&files.manifest, serde_json::to_vec(&manifest).unwrap()).unwrap();
        assert!(read_corpus(&files, 8, 4096, 0)
            .unwrap_err()
            .contains("manifest identity"));
    }

    #[test]
    fn vocabulary_bos_and_empty_response_inputs_fail_closed() {
        let (_directory, files) = written_corpus("artifact-mismatch", &[0, 10, 20], &[0, 0, 1]);
        assert!(read_corpus(&files, 8, 4097, 0)
            .unwrap_err()
            .contains("vocabulary/BOS"));
        assert!(read_corpus(&files, 8, 4096, 1)
            .unwrap_err()
            .contains("vocabulary/BOS"));

        let (_directory, empty) = written_corpus("empty-response", &[0, 10], &[0, 0]);
        assert!(read_corpus(&empty, 8, 4096, 0)
            .unwrap_err()
            .contains("no response"));
        assert!(evenly_cap((0..10).collect::<Vec<_>>(), Some(0)).is_empty());
        assert!(codebook_from_artifact(b"not-an-artifact").is_err());
    }

    #[test]
    fn absent_reference_fields_become_explicit_unavailable_strata() {
        let panel = corpus(&[0, 10, 11, 20, 21], &[0, 0, 0, 1, 1]);
        let reply = panel.runs.iter().find(|run| run.mask == 1).unwrap();
        let metadata = metadata_for(&panel, reply, None, None, None, None);
        assert_eq!(metadata.prompt_seen, None);
        assert_eq!(metadata.keyed, None);
        assert_eq!(metadata.reply_seen, None);
        assert_eq!(metadata.prompt_template_seen, None);

        let mut strata = BTreeMap::new();
        let row = Reading {
            targets: 1,
            admitted: 1,
            top1: 1,
            top8: 1,
        };
        for (name, value) in [
            ("prompt-overlap", metadata.prompt_seen),
            ("prompt-template", metadata.prompt_template_seen),
            ("memory-key", metadata.keyed),
            ("reference-reply-overlap", metadata.reply_seen),
        ] {
            let value = match value {
                Some(true) => "present",
                Some(false) => "absent",
                None => "unavailable",
            };
            add_stratum(&mut strata, format!("{name}:{value}"), &row);
        }
        assert!(strata.contains_key("prompt-overlap:unavailable"));
        assert!(strata.contains_key("prompt-template:unavailable"));
        assert!(strata.contains_key("memory-key:unavailable"));
        assert!(strata.contains_key("reference-reply-overlap:unavailable"));
    }

    #[test]
    fn planted_writer_metric_error_is_counted() {
        let roots = vec![[0u8; 4]; VOCABULARY];
        let writer = CpuWriterCore::from_roots(&roots, 2).unwrap();
        let query = *writer.token_vector(0).unwrap();
        let decision = writer.decide(&query);
        let mut served = uor_r4_integer::discrete_d11::DiscretePrefixEvaluation {
            read_vector: query.to_vec(),
            admitted_cells: decision.admitted[..decision.admitted_count]
                .iter()
                .copied()
                .filter(|&cell| cell != ABSENT_CELL)
                .map(u32::from)
                .collect(),
            top8: decision.top8.to_vec(),
            prediction: decision.top8[0],
            relation_scores: None,
        };
        let mut parity = WriterParity::from_serving_query();
        observe_writer_parity(&writer, &served, &mut parity).unwrap();
        assert!(parity.is_exact());

        served.prediction = decision.top8[1];
        served.top8.reverse();
        served.admitted_cells.clear();
        observe_writer_parity(&writer, &served, &mut parity).unwrap();
        assert_eq!(parity.admitted_cell_mismatches, 1);
        assert_eq!(parity.top8_mismatches, 1);
        assert_eq!(parity.token_mismatches, 1);
        assert!(!parity.is_exact());
    }

    #[test]
    fn report_serialization_is_byte_deterministic() {
        let report = EvidenceReport {
            schema: SCHEMA.to_owned(),
            artifact: ArtifactIdentity {
                path: "a".to_owned(),
                sha256: "00".repeat(32),
                model_sha256: "11".repeat(32),
                schema: "uor-r4.lut-discrete/1".to_owned(),
                tables: 4,
                vocabulary: 4096,
                context: 192,
                blocks: 4,
                kept_cells: 8,
                read_mode: "relation".to_owned(),
            },
            tokenizer: TokenizerIdentity {
                path: "t".to_owned(),
                sha256: "22".repeat(32),
                vocabulary: 4096,
                digit_tokens: 10,
                mask_ks: CENSUS_MASK_KS.to_vec(),
            },
            panel: corpus_identity(
                &corpus(&[0, 10, 20], &[0, 0, 1]),
                Some(ReferenceRole::GeneratedControl),
            ),
            train: None,
            measured_targets: 1,
            overall: Reading {
                targets: 1,
                admitted: 1,
                top1: 1,
                top8: 1,
            },
            strata: BTreeMap::from([(
                "reply-position:opening".to_owned(),
                Reading {
                    targets: 1,
                    admitted: 1,
                    top1: 1,
                    top8: 1,
                },
            )]),
            reply_overlap: None,
            template_overlap: None,
            writer_parity: WriterParity::from_serving_query(),
            cpu_engine_parity: CpuEngineParity::unavailable("fixture"),
            unavailable: BTreeMap::new(),
        };
        assert_eq!(
            serde_json::to_vec_pretty(&report).unwrap(),
            serde_json::to_vec_pretty(&report).unwrap()
        );
    }
}
