//! Construction-only HELM-D learned-manifold R4 decision harness for #973.
//!
//! The ignored freeze test commits a fresh 16-fit/8-validation population
//! without serializing any next-token target. The ignored decision test opens
//! fit inputs first, fits and independently replays two equal-capacity arms,
//! publishes an exclusive checkpoint, and only then materializes validation.

use std::collections::{BTreeMap, HashSet};
use std::env;
use std::error::Error;
use std::fs;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use uor_r4_core::helm_d_r4_attention::{
    canonical_registered_h4_spin_frames, helm_d_learned_manifold_centroid,
    helm_d_learned_manifold_logit, helm_d_learned_manifold_value_readout,
    helm_d_lorentz_causal_row, HelmDLearnedManifoldEvidence, HelmDLearnedManifoldIntervention,
    HelmDLearnedManifoldMetric, HelmDLearnedManifoldParameters, HelmDLearnedManifoldR4Transport,
    HelmDLearnedManifoldValueReadout, HelmDLorentzReferenceConfig, R4AffineAdapter,
    R4SpinCausalAttentionTransport, R4SpinFrameAtlas, R4SpinTransportEvidence,
    R4SpinTransportIntervention, HELM_D_LEARNED_EUCLIDEAN_LORENTZ_CENTROID_R4_LOCALIZATION_POLICY,
    HELM_D_LEARNED_EUCLIDEAN_R4_CONTROL_POLICY, HELM_D_LEARNED_LORENTZ_R4_CONSTRUCTION_POLICY,
    HELM_D_LEARNED_LORENTZ_TANGENT_R4_LOCALIZATION_POLICY, HELM_D_R4_GAUGE_SOFTMAX_POLICY,
    HELM_D_UPSTREAM_COMMIT,
};
use uor_r4_core::transformerless::scenarios::Tokenizer;
use uor_r4_model_source::attention::{
    head_attention_value_aggregate, standard_head_attention_weights, CausalAttentionHeadContext,
    CausalAttentionLayerSelection, CausalAttentionProjectionAudit,
    CausalAttentionProjectionContext, CausalAttentionSourceContext, CausalAttentionTransport,
    CausalAttentionTransportAudit,
};
use uor_r4_model_source::{
    HuggingFaceLlamaOracle, TeacherExecutionConfig, TeacherExecutionPreparation,
    TeacherExecutionSnapshot,
};

const MODEL_ENV: &str = "UOR_R4_973_HELM_D_MANIFOLD_MODEL";
const TOKENIZER_ENV: &str = "UOR_R4_973_HELM_D_MANIFOLD_TOKENIZER";
const CORPUS_ENV: &str = "UOR_R4_973_HELM_D_MANIFOLD_CORPUS";
const PARTITION_OUTPUT_ENV: &str = "UOR_R4_973_HELM_D_MANIFOLD_PARTITION_OUTPUT";
const PARTITION_ENV: &str = "UOR_R4_973_HELM_D_MANIFOLD_PARTITION";
const CHECKPOINT_ENV: &str = "UOR_R4_973_HELM_D_MANIFOLD_CHECKPOINT";
const RESULT_OUTPUT_ENV: &str = "UOR_R4_973_HELM_D_MANIFOLD_OUTPUT";
const LOCALIZATION_PARTITION_ENV: &str = "UOR_R4_973_SCORE_CENTROID_PARTITION";
const LOCALIZATION_CHECKPOINT_ENV: &str = "UOR_R4_973_SCORE_CENTROID_CHECKPOINT";
const LOCALIZATION_OUTPUT_ENV: &str = "UOR_R4_973_SCORE_CENTROID_OUTPUT";
const LOCALIZATION_TARGET_OUTPUT_ENV: &str = "UOR_R4_973_SCORE_CENTROID_TARGET_COMMITMENT_OUTPUT";
const LOCALIZATION_TARGET_ENV: &str = "UOR_R4_973_SCORE_CENTROID_TARGET_COMMITMENT";
const CANONICAL_DETERMINISTIC_ENV: &str = "TLESS_CANONICAL_DETERMINISTIC";

const DEFAULT_MODEL: &str = "/Users/casey.allard/uor-r4/.uor-models/sources/smollm2-135m-instruct";
const DEFAULT_TOKENIZER: &str =
    "/Users/casey.allard/uor-r4/.uor-models/compiled/smollm2-135m-instruct/tokenizer.bin";
const DEFAULT_CORPUS: &str =
    "/Users/casey.allard/uor-r4/.uor-models/corpora/simple-wiki-20231101/articles.jsonl";

const CORPUS_CID: &str = "blake3:194db0eebf2d49823ece01ee935447a0cc9edeaf018454ceea480ce7590132cf";
const DONOR_CID: &str = "blake3:12d2cd8a877ef2cdcf785b3d4d1f373e0419074cc884aeaff06fc059686a5ba5";
const PREDECESSOR_PARTITION_CID: &str =
    "blake3:cad3dfd17159fdacc5c40e38753109c11764117e3c960f42b9b198d5731272a1";
const CORPUS_DOCUMENTS: usize = 3_000;
const PARITY_DOCUMENT_ID: &str = "12";
const FIT_DOCUMENTS: usize = 16;
const VALIDATION_DOCUMENTS: usize = 8;
const REQUIRED_TOKENS: usize = 17;
const INPUT_POSITIONS: usize = 16;
const SCORE_START: usize = 8;
const SCORE_POSITIONS: usize = 8;
const SHARDS: usize = 8;
const ADAM_STEPS: usize = 128;
const LOCALIZATION_ADAM_STEPS: usize = 32;
const LOCALIZATION_FIT_DOCUMENTS: usize = 8;
const LOCALIZATION_AUDIT_DOCUMENTS: usize = 8;
const LOCALIZATION_PREFLIGHT_DOCUMENTS: usize = 2;
const LEARNING_RATE: f64 = 1.0e-3;
const ADAM_BETA1: f64 = 0.9;
const ADAM_BETA2: f64 = 0.999;
const ADAM_EPSILON: f64 = 1.0e-8;
const RIDGE: f64 = 1.0e-6;
const SCALE_FLOOR: f64 = 1.0e-6;
const INITIAL_SCALE: f64 = 24.0;
const MAX_CANARY_SECONDS: f64 = 2.0 * 60.0 * 60.0;
const EXPECTED_LAYERS: usize = 30;
const EXPECTED_QUERY_HEADS: usize = 9;
const EXPECTED_KV_HEADS: usize = 3;
const HEAD_WIDTH: usize = 64;
const R4_WIDTH: usize = 4;
const BLOCKS_PER_HEAD: usize = HEAD_WIDTH / R4_WIDTH;
const PARAMETER_SCALARS: usize = 144_060;
const PARTITION_SCHEMA: &str = "uor-r4.helm-d-learned-manifold-r4-construction-partition/2";
const CHECKPOINT_SCHEMA: &str = "uor-r4.helm-d-learned-manifold-r4-construction-checkpoint/2";
const RESULT_SCHEMA: &str = "uor-r4.helm-d-learned-manifold-r4-construction-result/2";
const LOCALIZATION_CHECKPOINT_SCHEMA: &str =
    "uor-r4.helm-d-score-centroid-localization-r4-checkpoint/1";
const LOCALIZATION_RESULT_SCHEMA: &str = "uor-r4.helm-d-score-centroid-localization-r4-result/1";
const LOCALIZATION_TARGET_CORPUS_STATUS: &str =
    "MANIFEST_ONLY_WITH_COMMITTED_TARGET_SPANS_VERIFIED";
const LOCALIZATION_PARTITION_CID: &str =
    "blake3:5c5a7dab9d7a0fbc9d176faafd49b42094ef89138cc32699dfc1b4fe937d1bde";
const LOCALIZATION_PREDECESSOR_RESULT_CID: &str =
    "blake3:9144913380c6ebdeebb5848138bc8e6642c1e7020d8e7a097aa3cd73cb829020";
const LOCALIZATION_PREDECESSOR_CHECKPOINT_CID: &str =
    "blake3:dd04bfc1cf15e5dd2c6c8be5afa363ecb452386f54632cd120bce56018444789";
const LOCALIZATION_PREDECESSOR_MANIFEST_CID: &str =
    "blake3:359b3270aaa0d3ac157280c9206ad820d18ee93932e0530552ebbb7935ac6410";

const LOCALIZATION_FIT_IDS: [&str; LOCALIZATION_FIT_DOCUMENTS] = [
    "8503", "7754", "3956", "7315", "476", "4749", "7525", "8141",
];
const LOCALIZATION_AUDIT_IDS: [&str; LOCALIZATION_AUDIT_DOCUMENTS] = [
    "6309", "271", "6749", "7604", "8384", "7183", "3621", "3799",
];

const PASS_TERMINAL: &str = "PASS_HELM_D_LEARNED_MANIFOLD_R4_CONSTRUCTION_AUTHORIZE_HELDOUT_FREEZE";
const RETAIN_TERMINAL: &str = "RETAIN_HELM_D_MANIFOLD_FUNCTIONAL_PARITY_NO_CURVATURE_ADVANTAGE";
const FAIL_TERMINAL: &str =
    "FAIL_HELM_D_MANIFOLD_CONSTRUCTION_REVISE_PROJECTION_SCORE_CENTROID_OR_TRAINING";
const UNAVAILABLE_TERMINAL: &str = "UNAVAILABLE_HELM_D_MANIFOLD_CONSTRUCTION_EVIDENCE";
const LOCALIZATION_SELECT_TANGENT_TERMINAL: &str =
    "SELECT_TANGENT_VALUE_READOUT_FOR_FRESH_CONSTRUCTION";
const LOCALIZATION_REJECT_TANGENT_PREFLIGHT_TERMINAL: &str =
    "REJECT_TANGENT_READOUT_SELECT_SCORE_PREFLIGHT";
const LOCALIZATION_SELECT_SCORE_TERMINAL: &str = "SELECT_FIXED_CURVATURE_SCORE_CONTINUATION";
const LOCALIZATION_REVISE_TERMINAL: &str = "REVISE_PROJECTION_OR_FITTER";
const LOCALIZATION_UNAVAILABLE_TERMINAL: &str = "UNAVAILABLE_OPERATOR_LOCALIZATION_EVIDENCE";

const COMPILED_CONTRACT: &[u8] =
    include_bytes!("../../../docs/helm_d_learned_manifold_r4_construction_973.md");
const COMPILED_LOCALIZATION_CONTRACT: &[u8] =
    include_bytes!("../../../docs/helm_d_score_centroid_localization_973.md");
const COMPILED_LOCALIZATION_RUNNER: &[u8] =
    include_bytes!("../../../scripts/run_helm_d_score_centroid_localization_973.sh");
const COMPILED_PREDECESSOR_PARTITION: &[u8] =
    include_bytes!("../../../docs/intrinsic_lorentz_r4_attention_partition_973.json");
const COMPILED_PREDECESSOR_RESULT: &[u8] = include_bytes!(
    "../../../docs/helm_d_learned_manifold_r4_construction_attempt_02_result_973.json"
);
const COMPILED_PREDECESSOR_CHECKPOINT: &[u8] = include_bytes!(
    "../../../docs/helm_d_learned_manifold_r4_construction_attempt_02_checkpoint_973.json"
);
const COMPILED_CORE_SOURCE: &[u8] = include_bytes!("../src/helm_d_r4_attention.rs");
const COMPILED_HARNESS_SOURCE: &[u8] =
    include_bytes!("helm_d_learned_manifold_r4_construction_973.rs");
const COMPILED_MODEL_ATTENTION_SOURCE: &[u8] =
    include_bytes!("../../uor-r4-model-source/src/attention.rs");
const COMPILED_MODEL_SOURCE: &[u8] = include_bytes!("../../uor-r4-model-source/src/lib.rs");

type TestResult<T = ()> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Debug, Deserialize)]
struct Article {
    id: String,
    title: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct CorpusManifest {
    article_count: usize,
    corpus_cid: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FrozenDocumentCommitment {
    id: String,
    selection_digest: String,
    title_cid: String,
    input_cid: String,
    predecessor_domain_input_cid: String,
    corpus_byte_offset: u64,
    corpus_byte_length: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FrozenExclusionSet {
    predecessor_partition_cid: String,
    document_ids: Vec<String>,
    input_cids: Vec<String>,
    exclusion_cid: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FrozenPartition {
    schema: String,
    issue: u32,
    selection_policy: String,
    selection_policy_cid: String,
    corpus_cid: String,
    corpus_documents: usize,
    donor_cid: String,
    tokenizer_cid: String,
    upstream_source_commit: String,
    required_tokens_per_document: usize,
    input_positions: usize,
    scored_positions: Vec<usize>,
    exclusions: FrozenExclusionSet,
    construction_fit: Vec<FrozenDocumentCommitment>,
    construction_validation: Vec<FrozenDocumentCommitment>,
    partition_cid: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FrozenPartitionEnvelope {
    manifest_cid: String,
    manifest: FrozenPartition,
}

#[derive(Debug)]
struct Candidate {
    selection_digest: [u8; 32],
    commitment: FrozenDocumentCommitment,
}

#[derive(Clone, Debug)]
struct FrozenDocument {
    id: String,
    tokens: Vec<u32>,
}

#[derive(Debug, Deserialize)]
struct PredecessorEnvelope {
    manifest: PredecessorManifest,
}

#[derive(Debug, Deserialize)]
struct PredecessorManifest {
    partition_cid: String,
    construction_fit: Vec<PredecessorDocument>,
    construction_validation: Vec<PredecessorDocument>,
    #[serde(rename = "d3_heldout")]
    predecessor_heldout: Vec<PredecessorDocument>,
}

#[derive(Debug, Deserialize)]
struct PredecessorDocument {
    id: String,
    input_cid: String,
}

fn path_from_env(name: &str, default: &str) -> PathBuf {
    env::var_os(name).map_or_else(|| PathBuf::from(default), PathBuf::from)
}

fn required_path_from_env(name: &str) -> TestResult<PathBuf> {
    env::var_os(name)
        .map(PathBuf::from)
        .ok_or_else(|| format!("required path environment variable {name} is unset").into())
}

fn cid_bytes(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

fn file_cid(path: &Path) -> TestResult<String> {
    let mut input = fs::File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

fn token_cid(domain: &[u8], tokens: &[u32]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(
        &u64::try_from(tokens.len())
            .unwrap_or(u64::MAX)
            .to_le_bytes(),
    );
    for token in tokens {
        hasher.update(&token.to_le_bytes());
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn string_cid(domain: &[u8], value: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(&u64::try_from(value.len()).unwrap_or(u64::MAX).to_le_bytes());
    hasher.update(value.as_bytes());
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn selection_digest(id: &str) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4.helm-d-learned-manifold-r4-construction/2\0");
    hasher.update(id.as_bytes());
    *hasher.finalize().as_bytes()
}

fn reserved_partition_id(id: &str) -> bool {
    blake3::hash(id.as_bytes()).as_bytes()[0].is_multiple_of(5)
}

fn canonical_json_bytes(value: &impl Serialize) -> TestResult<Vec<u8>> {
    let mut value = serde_json::to_value(value)?;
    sort_json_keys(&mut value);
    Ok(serde_json::to_vec(&value)?)
}

fn sort_json_keys(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Array(values) => values.iter_mut().for_each(sort_json_keys),
        serde_json::Value::Object(object) => {
            let mut entries = std::mem::take(object).into_iter().collect::<Vec<_>>();
            entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));
            for (key, mut value) in entries {
                sort_json_keys(&mut value);
                object.insert(key, value);
            }
        }
        _ => {}
    }
}

fn canonical_json_cid(value: &impl Serialize) -> TestResult<String> {
    Ok(cid_bytes(&canonical_json_bytes(value)?))
}

fn write_pretty_json(path: &Path, value: &impl Serialize) -> TestResult {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}

fn write_pretty_json_exclusive(path: &Path, value: &impl Serialize) -> TestResult {
    let parent = path.parent().ok_or("exclusive path has no parent")?;
    fs::create_dir_all(parent)?;
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    output.write_all(&bytes)?;
    output.sync_all()?;
    fs::File::open(parent)?.sync_all()?;
    Ok(())
}

fn verify_corpus_manifest(path: &Path) -> TestResult {
    let manifest_path = path.with_file_name("manifest.json");
    let manifest: CorpusManifest = serde_json::from_slice(&fs::read(manifest_path)?)?;
    if manifest.article_count != CORPUS_DOCUMENTS || manifest.corpus_cid != CORPUS_CID {
        return Err("SimpleWiki manifest identity mismatch".into());
    }
    Ok(())
}

fn verify_complete_corpus(path: &Path) -> TestResult {
    verify_corpus_manifest(path)?;
    if file_cid(path)? != CORPUS_CID {
        return Err("SimpleWiki byte identity mismatch".into());
    }
    Ok(())
}

fn predecessor_exclusions() -> TestResult<FrozenExclusionSet> {
    let predecessor: PredecessorEnvelope = serde_json::from_slice(COMPILED_PREDECESSOR_PARTITION)?;
    if predecessor.manifest.partition_cid != PREDECESSOR_PARTITION_CID {
        return Err("predecessor partition CID does not match the frozen exclusion source".into());
    }
    let documents = predecessor
        .manifest
        .construction_fit
        .iter()
        .chain(&predecessor.manifest.construction_validation)
        .chain(&predecessor.manifest.predecessor_heldout)
        .collect::<Vec<_>>();
    let mut document_ids = documents
        .iter()
        .map(|document| document.id.clone())
        .collect::<Vec<_>>();
    let mut input_cids = documents
        .iter()
        .map(|document| document.input_cid.clone())
        .collect::<Vec<_>>();
    document_ids.sort_unstable_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    input_cids.sort_unstable();
    if document_ids.len() != 28
        || input_cids.len() != 28
        || document_ids.windows(2).any(|pair| pair[0] == pair[1])
        || input_cids.windows(2).any(|pair| pair[0] == pair[1])
    {
        return Err("predecessor exclusion set is incomplete or duplicated".into());
    }
    let mut exclusion = FrozenExclusionSet {
        predecessor_partition_cid: PREDECESSOR_PARTITION_CID.to_owned(),
        document_ids,
        input_cids,
        exclusion_cid: String::new(),
    };
    exclusion.exclusion_cid = canonical_json_cid(&exclusion)?;
    Ok(exclusion)
}

fn partition_cid(partition: &FrozenPartition) -> TestResult<String> {
    let mut commitment = partition.clone();
    commitment.partition_cid.clear();
    canonical_json_cid(&commitment)
}

fn validate_partition(partition: &FrozenPartition) -> TestResult {
    if partition.schema != PARTITION_SCHEMA
        || partition.issue != 973
        || partition.corpus_cid != CORPUS_CID
        || partition.corpus_documents != CORPUS_DOCUMENTS
        || partition.donor_cid != DONOR_CID
        || partition.upstream_source_commit != HELM_D_UPSTREAM_COMMIT
        || partition.required_tokens_per_document != REQUIRED_TOKENS
        || partition.input_positions != INPUT_POSITIONS
        || partition.scored_positions != (SCORE_START..INPUT_POSITIONS).collect::<Vec<_>>()
        || partition.construction_fit.len() != FIT_DOCUMENTS
        || partition.construction_validation.len() != VALIDATION_DOCUMENTS
        || partition.exclusions != predecessor_exclusions()?
        || partition.selection_policy_cid
            != string_cid(b"uor-r4.selection-policy/1", &partition.selection_policy)
        || partition.partition_cid != partition_cid(partition)?
    {
        return Err("frozen construction partition header or identity is invalid".into());
    }
    let excluded_ids = partition
        .exclusions
        .document_ids
        .iter()
        .collect::<HashSet<_>>();
    let excluded_inputs = partition
        .exclusions
        .input_cids
        .iter()
        .collect::<HashSet<_>>();
    let mut selected_ids = HashSet::new();
    let mut selected_inputs = HashSet::new();
    let mut ordered = partition
        .construction_fit
        .iter()
        .chain(&partition.construction_validation);
    let mut prior: Option<([u8; 32], Vec<u8>)> = None;
    for document in ordered.by_ref() {
        let digest = selection_digest(&document.id);
        let key = (digest, document.id.as_bytes().to_vec());
        if document.id == PARITY_DOCUMENT_ID
            || reserved_partition_id(&document.id)
            || excluded_ids.contains(&document.id)
            || excluded_inputs.contains(&document.predecessor_domain_input_cid)
            || document.selection_digest != format!("blake3:{}", hex::encode(digest))
            || !document.title_cid.starts_with("blake3:")
            || !document.input_cid.starts_with("blake3:")
            || !document.predecessor_domain_input_cid.starts_with("blake3:")
            || document.corpus_byte_length == 0
            || document
                .corpus_byte_offset
                .checked_add(document.corpus_byte_length)
                .is_none()
            || !selected_ids.insert(document.id.clone())
            || !selected_inputs.insert(document.input_cid.clone())
            || prior.as_ref().is_some_and(|prior| prior >= &key)
        {
            return Err(format!(
                "selected document {} violates the frozen policy",
                document.id
            )
            .into());
        }
        prior = Some(key);
    }
    Ok(())
}

fn parse_partition(bytes: &[u8]) -> TestResult<FrozenPartitionEnvelope> {
    let envelope: FrozenPartitionEnvelope = serde_json::from_slice(bytes)?;
    validate_partition(&envelope.manifest)?;
    if envelope.manifest_cid != canonical_json_cid(&envelope.manifest)? {
        return Err("partition envelope CID mismatch".into());
    }
    Ok(envelope)
}

#[test]
#[ignore = "freezes a fresh 16-fit/8-validation construction-only population"]
fn freeze_helm_d_learned_manifold_r4_construction_partition() -> TestResult {
    let tokenizer_path = path_from_env(TOKENIZER_ENV, DEFAULT_TOKENIZER);
    let corpus_path = path_from_env(CORPUS_ENV, DEFAULT_CORPUS);
    verify_complete_corpus(&corpus_path)?;
    let tokenizer = Tokenizer::try_load(&tokenizer_path)?;
    let exclusions = predecessor_exclusions()?;
    let excluded_ids = exclusions.document_ids.iter().collect::<HashSet<_>>();
    let excluded_inputs = exclusions.input_cids.iter().collect::<HashSet<_>>();
    let mut observed_ids = HashSet::new();
    let mut observed_documents = 0_usize;
    let mut candidates = Vec::new();
    let mut corpus = BufReader::new(fs::File::open(&corpus_path)?);
    let mut line = Vec::new();
    let mut offset = 0_u64;
    loop {
        line.clear();
        let count = corpus.read_until(b'\n', &mut line)?;
        if count == 0 {
            break;
        }
        let line_offset = offset;
        let line_length = u64::try_from(count)?;
        offset = offset
            .checked_add(line_length)
            .ok_or("corpus offset overflow")?;
        if line.iter().all(u8::is_ascii_whitespace) {
            continue;
        }
        observed_documents += 1;
        let article: Article = serde_json::from_slice(&line)?;
        if !observed_ids.insert(article.id.clone()) {
            return Err(format!("duplicate corpus document {}", article.id).into());
        }
        if article.id == PARITY_DOCUMENT_ID
            || reserved_partition_id(&article.id)
            || excluded_ids.contains(&article.id)
        {
            continue;
        }
        let tokens = tokenizer.encode(&format!("{}\n\n{}", article.title, article.text));
        if tokens.len() < REQUIRED_TOKENS {
            continue;
        }
        let input_cid = token_cid(
            b"uor-r4.helm-d-learned-manifold.inputs/2",
            &tokens[..INPUT_POSITIONS],
        );
        let predecessor_domain_input_cid =
            token_cid(b"uor-r4.intrinsic.inputs/1", &tokens[..INPUT_POSITIONS]);
        if excluded_inputs.contains(&predecessor_domain_input_cid) {
            continue;
        }
        let digest = selection_digest(&article.id);
        candidates.push(Candidate {
            selection_digest: digest,
            commitment: FrozenDocumentCommitment {
                id: article.id,
                selection_digest: format!("blake3:{}", hex::encode(digest)),
                title_cid: string_cid(b"uor-r4.helm-d-learned-manifold.title/2", &article.title),
                input_cid,
                predecessor_domain_input_cid,
                corpus_byte_offset: line_offset,
                corpus_byte_length: line_length,
            },
        });
    }
    if observed_documents != CORPUS_DOCUMENTS {
        return Err(format!(
            "corpus count mismatch: expected {CORPUS_DOCUMENTS}, observed {observed_documents}"
        )
        .into());
    }
    candidates.sort_by(|left, right| {
        left.selection_digest
            .cmp(&right.selection_digest)
            .then_with(|| {
                left.commitment
                    .id
                    .as_bytes()
                    .cmp(right.commitment.id.as_bytes())
            })
    });
    if candidates.len() < FIT_DOCUMENTS + VALIDATION_DOCUMENTS {
        return Err("insufficient fresh eligible construction documents".into());
    }
    let selected = candidates
        .into_iter()
        .take(FIT_DOCUMENTS + VALIDATION_DOCUMENTS)
        .map(|candidate| candidate.commitment)
        .collect::<Vec<_>>();
    let selection_policy = concat!(
        "BLAKE3(domain-nul-UTF8(id)); exclude reserved digest class, id 12, and every ",
        "predecessor id/input CID; eligible encoded length >=17; sort complete digest then UTF8 id; ",
        "first 16 fit, next 8 validation; commit id/title CID/input CID/byte span only"
    )
    .to_owned();
    let mut partition = FrozenPartition {
        schema: PARTITION_SCHEMA.to_owned(),
        issue: 973,
        selection_policy_cid: string_cid(b"uor-r4.selection-policy/1", &selection_policy),
        selection_policy,
        corpus_cid: CORPUS_CID.to_owned(),
        corpus_documents: CORPUS_DOCUMENTS,
        donor_cid: DONOR_CID.to_owned(),
        tokenizer_cid: file_cid(&tokenizer_path)?,
        upstream_source_commit: HELM_D_UPSTREAM_COMMIT.to_owned(),
        required_tokens_per_document: REQUIRED_TOKENS,
        input_positions: INPUT_POSITIONS,
        scored_positions: (SCORE_START..INPUT_POSITIONS).collect(),
        exclusions,
        construction_fit: selected[..FIT_DOCUMENTS].to_vec(),
        construction_validation: selected[FIT_DOCUMENTS..].to_vec(),
        partition_cid: String::new(),
    };
    partition.partition_cid = partition_cid(&partition)?;
    validate_partition(&partition)?;
    let envelope = FrozenPartitionEnvelope {
        manifest_cid: canonical_json_cid(&partition)?,
        manifest: partition,
    };
    let output = required_path_from_env(PARTITION_OUTPUT_ENV)?;
    write_pretty_json(&output, &envelope)?;
    eprintln!(
        "frozen HELM-D learned-manifold construction partition: {}",
        envelope.manifest_cid
    );
    Ok(())
}

fn materialize_documents(
    corpus_path: &Path,
    tokenizer: &Tokenizer,
    commitments: &[FrozenDocumentCommitment],
    include_next_token: bool,
) -> TestResult<Vec<FrozenDocument>> {
    let mut corpus = fs::File::open(corpus_path)?;
    let corpus_len = corpus.metadata()?.len();
    let mut documents = Vec::with_capacity(commitments.len());
    for commitment in commitments {
        if reserved_partition_id(&commitment.id) || commitment.id == PARITY_DOCUMENT_ID {
            return Err(format!(
                "materialization rejected reserved document {}",
                commitment.id
            )
            .into());
        }
        let end = commitment
            .corpus_byte_offset
            .checked_add(commitment.corpus_byte_length)
            .filter(|end| commitment.corpus_byte_length > 0 && *end <= corpus_len)
            .ok_or("committed corpus span is invalid")?;
        let mut bytes = vec![0_u8; usize::try_from(commitment.corpus_byte_length)?];
        corpus.seek(SeekFrom::Start(commitment.corpus_byte_offset))?;
        corpus.read_exact(&mut bytes)?;
        if corpus.stream_position()? != end {
            return Err("committed corpus span ended at the wrong offset".into());
        }
        let article: Article = serde_json::from_slice(&bytes)?;
        if article.id != commitment.id
            || string_cid(b"uor-r4.helm-d-learned-manifold.title/2", &article.title)
                != commitment.title_cid
        {
            return Err(format!("committed document {} identity mismatch", commitment.id).into());
        }
        let mut tokens = tokenizer.encode(&format!("{}\n\n{}", article.title, article.text));
        if tokens.len() < REQUIRED_TOKENS {
            return Err(
                format!("committed document {} is no longer eligible", commitment.id).into(),
            );
        }
        if token_cid(
            b"uor-r4.helm-d-learned-manifold.inputs/2",
            &tokens[..INPUT_POSITIONS],
        ) != commitment.input_cid
            || token_cid(b"uor-r4.intrinsic.inputs/1", &tokens[..INPUT_POSITIONS])
                != commitment.predecessor_domain_input_cid
        {
            return Err(format!("committed document {} input CID mismatch", commitment.id).into());
        }
        tokens.truncate(if include_next_token {
            REQUIRED_TOKENS
        } else {
            INPUT_POSITIONS
        });
        documents.push(FrozenDocument {
            id: article.id,
            tokens,
        });
    }
    Ok(documents)
}

#[derive(Clone, Debug, PartialEq)]
struct ProjectionTrace {
    query: Vec<f32>,
    key: Vec<f32>,
    value: Vec<f32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct HeadTrace {
    post_rope_query: Vec<f32>,
    post_rope_current_key: Vec<f32>,
    current_value: Vec<f32>,
    donor_weights: Vec<f32>,
    donor_aggregate: Vec<f32>,
}

#[derive(Clone, Debug, Default)]
struct DonorTraceState {
    projections: BTreeMap<(usize, usize), ProjectionTrace>,
    heads: BTreeMap<(usize, usize, usize), HeadTrace>,
}

#[derive(Clone, Debug)]
struct CapturedDocument {
    document: FrozenDocument,
    projections: Vec<Vec<ProjectionTrace>>,
    heads: Vec<Vec<Vec<HeadTrace>>>,
    decoder_audit: CausalAttentionTransportAudit,
    projection_audit: CausalAttentionProjectionAudit,
    trace_cid: String,
}

struct DonorTracingTransport {
    state: Arc<Mutex<DonorTraceState>>,
}

impl DonorTracingTransport {
    fn with_head(&self, context: CausalAttentionHeadContext, update: impl FnOnce(&mut HeadTrace)) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        update(
            state
                .heads
                .entry((context.query_position, context.layer, context.head))
                .or_default(),
        );
    }
}

impl CausalAttentionTransport for DonorTracingTransport {
    fn policy_identity(&self) -> &str {
        "uor-r4.donor-pre-rope-qkv-attention-trace/2"
    }

    fn begin_position(&mut self, _token: usize, _position: usize) {}

    fn transform_projected_qkv_before_rope(
        &mut self,
        context: CausalAttentionProjectionContext,
        query: &mut [f32],
        key: &mut [f32],
        value: &mut [f32],
    ) {
        let replaced = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .projections
            .insert(
                (context.query_position, context.layer),
                ProjectionTrace {
                    query: query.to_vec(),
                    key: key.to_vec(),
                    value: value.to_vec(),
                },
            );
        assert!(replaced.is_none(), "projection hook invoked more than once");
    }

    fn transform_query(
        &mut self,
        context: CausalAttentionHeadContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        self.with_head(context, |head| head.post_rope_query = input.to_vec());
        output.copy_from_slice(input);
    }

    fn transport_key(
        &mut self,
        context: CausalAttentionSourceContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        if context.source_position == context.query_position {
            self.with_head(
                CausalAttentionHeadContext {
                    layer: context.layer,
                    head: context.head,
                    query_position: context.query_position,
                },
                |head| head.post_rope_current_key = input.to_vec(),
            );
        }
        output.copy_from_slice(input);
    }

    fn transport_value(
        &mut self,
        context: CausalAttentionSourceContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        if context.source_position == context.query_position {
            self.with_head(
                CausalAttentionHeadContext {
                    layer: context.layer,
                    head: context.head,
                    query_position: context.query_position,
                },
                |head| head.current_value = input.to_vec(),
            );
        }
        output.copy_from_slice(input);
    }

    fn score_and_normalize(
        &mut self,
        context: CausalAttentionHeadContext,
        query: &[f32],
        packed_keys: &[f32],
        output_weights: &mut [f32],
        canonical_math: bool,
    ) {
        standard_head_attention_weights(
            output_weights,
            query,
            packed_keys,
            0,
            query.len(),
            canonical_math,
        );
        self.with_head(context, |head| head.donor_weights = output_weights.to_vec());
    }

    fn weighted_value_centroid(
        &mut self,
        context: CausalAttentionHeadContext,
        weights: &[f32],
        packed_values: &[f32],
        output: &mut [f32],
    ) {
        let width = output.len();
        head_attention_value_aggregate(output, weights, packed_values, 0, width);
        self.with_head(context, |head| head.donor_aggregate = output.to_vec());
    }

    fn output_to_model_frame(
        &mut self,
        _context: CausalAttentionHeadContext,
        input: &[f32],
        output: &mut [f32],
    ) {
        output.copy_from_slice(input);
    }
}

fn capture_document(
    oracle: &HuggingFaceLlamaOracle,
    document: &FrozenDocument,
) -> TestResult<CapturedDocument> {
    if document.tokens.len() != INPUT_POSITIONS {
        return Err("fit trace must not contain a next-token target".into());
    }
    let config = oracle.cfg();
    let shared = Arc::new(Mutex::new(DonorTraceState::default()));
    let mut traced = oracle.new_causal_attention_transport_session(
        Box::new(DonorTracingTransport {
            state: shared.clone(),
        }),
        CausalAttentionLayerSelection::All,
        INPUT_POSITIONS,
    )?;
    let mut donor = oracle.new_state_bounded(INPUT_POSITIONS)?;
    let mut traced_logits = vec![0.0_f32; config.vocab];
    let mut donor_logits = vec![0.0_f32; config.vocab];
    let mut logits = Vec::with_capacity(INPUT_POSITIONS);
    for (position, token) in document.tokens.iter().copied().enumerate() {
        oracle.step_state(&mut donor, token as usize, position, &mut donor_logits)?;
        oracle.step_causal_attention_transport(
            &mut traced,
            token as usize,
            position,
            &mut traced_logits,
        )?;
        if donor_logits
            .iter()
            .zip(&traced_logits)
            .any(|(left, right)| left.to_bits() != right.to_bits())
        {
            return Err(format!("identity trace changed donor logits for {}", document.id).into());
        }
        logits.push(traced_logits.clone());
    }
    if donor.persistent_state_cid() != traced.persistent_state_cid() {
        return Err("identity tracing path changed persistent decoder state".into());
    }
    let raw = shared
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let mut projections = Vec::with_capacity(INPUT_POSITIONS);
    let mut heads = Vec::with_capacity(INPUT_POSITIONS);
    for position in 0..INPUT_POSITIONS {
        let mut position_projections = Vec::with_capacity(config.n_layers);
        let mut position_heads = Vec::with_capacity(config.n_layers);
        for layer in 0..config.n_layers {
            position_projections.push(
                raw.projections
                    .get(&(position, layer))
                    .cloned()
                    .ok_or("missing pre-RoPE projection trace")?,
            );
            let mut layer_heads = Vec::with_capacity(config.n_heads);
            for head in 0..config.n_heads {
                let trace = raw
                    .heads
                    .get(&(position, layer, head))
                    .cloned()
                    .ok_or("missing donor attention row trace")?;
                if trace.post_rope_query.len() != HEAD_WIDTH
                    || trace.post_rope_current_key.len() != HEAD_WIDTH
                    || trace.current_value.len() != HEAD_WIDTH
                    || trace.donor_weights.len() != position + 1
                    || trace.donor_aggregate.len() != HEAD_WIDTH
                {
                    return Err("donor attention row trace shape mismatch".into());
                }
                layer_heads.push(trace);
            }
            position_heads.push(layer_heads);
        }
        projections.push(position_projections);
        heads.push(position_heads);
    }
    let decoder_audit = traced.audit();
    let projection_audit = traced.pre_rope_projection_audit();
    let trace_cid = donor_trace_cid(&projections, &heads, &logits);
    Ok(CapturedDocument {
        document: document.clone(),
        projections,
        heads,
        decoder_audit,
        projection_audit,
        trace_cid,
    })
}

fn donor_trace_cid(
    projections: &[Vec<ProjectionTrace>],
    heads: &[Vec<Vec<HeadTrace>>],
    logits: &[Vec<f32>],
) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4.helm-d-learned-manifold.donor-trace/2\0");
    for projection in projections.iter().flatten() {
        for values in [&projection.query, &projection.key, &projection.value] {
            for value in values {
                hasher.update(&value.to_bits().to_le_bytes());
            }
        }
    }
    for head in heads.iter().flatten().flatten() {
        for values in [
            &head.post_rope_query,
            &head.post_rope_current_key,
            &head.current_value,
            &head.donor_weights,
            &head.donor_aggregate,
        ] {
            for value in values {
                hasher.update(&value.to_bits().to_le_bytes());
            }
        }
    }
    for value in logits.iter().flatten() {
        hasher.update(&value.to_bits().to_le_bytes());
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn capture_fit_documents(
    oracle: &HuggingFaceLlamaOracle,
    documents: &[FrozenDocument],
) -> TestResult<Vec<CapturedDocument>> {
    documents
        .iter()
        .map(|document| capture_document(oracle, document))
        .collect()
}

#[derive(Clone, Debug, Default)]
struct AffineGradient {
    matrix: [[f64; R4_WIDTH]; R4_WIDTH],
    bias: [f64; R4_WIDTH],
}

#[derive(Clone, Debug)]
struct ParameterGradient {
    query: Vec<AffineGradient>,
    key: Vec<AffineGradient>,
    value: Vec<AffineGradient>,
    scale: Vec<f64>,
    bias: Vec<f64>,
}

impl ParameterGradient {
    fn zero(parameters: &HelmDLearnedManifoldParameters) -> Self {
        Self {
            query: vec![AffineGradient::default(); parameters.query_adapters().len()],
            key: vec![AffineGradient::default(); parameters.key_adapters().len()],
            value: vec![AffineGradient::default(); parameters.value_adapters().len()],
            scale: vec![0.0; parameters.layers()],
            bias: vec![0.0; parameters.layers()],
        }
    }

    fn add_assign(&mut self, other: &Self) {
        for (target, source) in self.query.iter_mut().zip(&other.query) {
            add_affine_gradient(target, source);
        }
        for (target, source) in self.key.iter_mut().zip(&other.key) {
            add_affine_gradient(target, source);
        }
        for (target, source) in self.value.iter_mut().zip(&other.value) {
            add_affine_gradient(target, source);
        }
        for (target, source) in self.scale.iter_mut().zip(&other.scale) {
            *target += source;
        }
        for (target, source) in self.bias.iter_mut().zip(&other.bias) {
            *target += source;
        }
    }

    fn scale_by(&mut self, factor: f64) {
        for gradient in self
            .query
            .iter_mut()
            .chain(&mut self.key)
            .chain(&mut self.value)
        {
            for value in gradient
                .matrix
                .iter_mut()
                .flatten()
                .chain(&mut gradient.bias)
            {
                *value *= factor;
            }
        }
        for value in self.scale.iter_mut().chain(&mut self.bias) {
            *value *= factor;
        }
    }

    fn all_finite(&self) -> bool {
        self.query
            .iter()
            .chain(&self.key)
            .chain(&self.value)
            .flat_map(|gradient| gradient.matrix.iter().flatten().chain(&gradient.bias))
            .chain(&self.scale)
            .chain(&self.bias)
            .all(|value| value.is_finite())
    }

    fn bind_identity_and_maximum(&self, hasher: &mut blake3::Hasher, maximum_absolute: &mut f64) {
        for gradient in self.query.iter().chain(&self.key).chain(&self.value) {
            for value in gradient.matrix.iter().flatten().chain(&gradient.bias) {
                hasher.update(&value.to_bits().to_le_bytes());
                *maximum_absolute = maximum_absolute.max(value.abs());
            }
        }
        for value in self.scale.iter().chain(&self.bias) {
            hasher.update(&value.to_bits().to_le_bytes());
            *maximum_absolute = maximum_absolute.max(value.abs());
        }
    }
}

fn add_affine_gradient(target: &mut AffineGradient, source: &AffineGradient) {
    for (target, source) in target
        .matrix
        .iter_mut()
        .flatten()
        .zip(source.matrix.iter().flatten())
    {
        *target += source;
    }
    for (target, source) in target.bias.iter_mut().zip(source.bias) {
        *target += source;
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct WorkLedger {
    workers: usize,
    full_batch_steps: usize,
    full_batch_evaluations: usize,
    rows_per_shard_per_evaluation: Vec<u64>,
    source_pairs_per_shard_per_evaluation: Vec<u64>,
    total_row_evaluations: u64,
    total_source_pair_evaluations: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct GradientAudit {
    schema: String,
    gradient_evaluations: usize,
    optimizer_gradient_evaluations: usize,
    diagnostic_gradient_evaluations: usize,
    maximum_absolute_gradient: f64,
    ordered_evaluation_cids: Vec<String>,
}

#[derive(Clone, Debug)]
struct ShardEvaluation {
    loss: f64,
    rows: u64,
    source_pairs: u64,
    gradient: ParameterGradient,
}

#[derive(Clone, Debug)]
struct DatasetEvaluation {
    objective: f64,
    gradient: ParameterGradient,
    rows_per_shard: Vec<u64>,
    source_pairs_per_shard: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct FitReport {
    metric: HelmDLearnedManifoldMetric,
    optimizer: String,
    steps: usize,
    workers: usize,
    fit_documents: usize,
    fit_rows: u64,
    initial_objective: f64,
    final_objective: f64,
    parameter_scalars: usize,
    parameter_cid: String,
    donor_trace_cid: String,
    gradient_audit: GradientAudit,
    gradient_audit_cid: String,
    work: WorkLedger,
    report_cid: String,
}

fn fit_report_cid(report: &FitReport) -> TestResult<String> {
    let mut commitment = report.clone();
    commitment.report_cid.clear();
    canonical_json_cid(&commitment)
}

#[derive(Clone, Debug)]
struct FittedArm {
    parameters: HelmDLearnedManifoldParameters,
    parameter_bytes: Vec<u8>,
    report: FitReport,
}

fn adapter_index(layer: usize, head: usize, block: usize, heads: usize) -> usize {
    (layer * heads + head) * BLOCKS_PER_HEAD + block
}

fn apply_adapters(
    adapters: &[R4AffineAdapter],
    layer: usize,
    head: usize,
    heads: usize,
    input: &[f32],
) -> TestResult<Vec<f64>> {
    if input.len() != HEAD_WIDTH {
        return Err("adapter input is not one complete head".into());
    }
    let mut output = vec![0.0; HEAD_WIDTH];
    for block in 0..BLOCKS_PER_HEAD {
        let adapter = adapters
            .get(adapter_index(layer, head, block, heads))
            .ok_or("adapter index is unavailable")?;
        for row in 0..R4_WIDTH {
            let mut value = adapter.bias[row];
            for column in 0..R4_WIDTH {
                value += adapter.matrix[row][column] * f64::from(input[block * R4_WIDTH + column]);
            }
            if !value.is_finite() {
                return Err("adapter produced a non-finite value".into());
            }
            output[block * R4_WIDTH + row] = value;
        }
    }
    Ok(output)
}

fn rope_coefficients(position: usize, frequency_index: usize, rope_theta: f32) -> (f64, f64) {
    let frequency =
        1.0_f32 / libm::powf(rope_theta, (2 * frequency_index) as f32 / HEAD_WIDTH as f32);
    let angle = position as f32 * frequency;
    (f64::from(libm::cosf(angle)), f64::from(libm::sinf(angle)))
}

fn apply_rope_f64(values: &mut [f64], position: usize, rope_theta: f32, interleaved: bool) {
    if interleaved {
        for pair in 0..HEAD_WIDTH / 2 {
            let (cos, sin) = rope_coefficients(position, pair, rope_theta);
            let first = values[2 * pair];
            let second = values[2 * pair + 1];
            values[2 * pair] = first * cos - second * sin;
            values[2 * pair + 1] = second * cos + first * sin;
        }
    } else {
        let half = HEAD_WIDTH / 2;
        for lane in 0..half {
            let (cos, sin) = rope_coefficients(position, lane, rope_theta);
            let first = values[lane];
            let second = values[lane + half];
            values[lane] = first * cos - second * sin;
            values[lane + half] = second * cos + first * sin;
        }
    }
}

fn apply_rope_f32(values: &mut [f32], position: usize, rope_theta: f32, interleaved: bool) {
    if interleaved {
        for pair in 0..HEAD_WIDTH / 2 {
            let (cos, sin) = rope_coefficients(position, pair, rope_theta);
            let cos = cos as f32;
            let sin = sin as f32;
            let first = values[2 * pair];
            let second = values[2 * pair + 1];
            values[2 * pair] = first * cos - second * sin;
            values[2 * pair + 1] = second * cos + first * sin;
        }
    } else {
        let half = HEAD_WIDTH / 2;
        for lane in 0..half {
            let (cos, sin) = rope_coefficients(position, lane, rope_theta);
            let cos = cos as f32;
            let sin = sin as f32;
            let first = values[lane];
            let second = values[lane + half];
            values[lane] = first * cos - second * sin;
            values[lane + half] = second * cos + first * sin;
        }
    }
}

fn rope_gradient_to_input(
    gradient: &[f64],
    position: usize,
    rope_theta: f32,
    interleaved: bool,
) -> Vec<f64> {
    let mut output = vec![0.0; HEAD_WIDTH];
    if interleaved {
        for pair in 0..HEAD_WIDTH / 2 {
            let (cos, sin) = rope_coefficients(position, pair, rope_theta);
            let first = gradient[2 * pair];
            let second = gradient[2 * pair + 1];
            output[2 * pair] = first * cos + second * sin;
            output[2 * pair + 1] = -first * sin + second * cos;
        }
    } else {
        let half = HEAD_WIDTH / 2;
        for lane in 0..half {
            let (cos, sin) = rope_coefficients(position, lane, rope_theta);
            let first = gradient[lane];
            let second = gradient[lane + half];
            output[lane] = first * cos + second * sin;
            output[lane + half] = -first * sin + second * cos;
        }
    }
    output
}

fn stable_softmax_f64(logits: &[f64]) -> TestResult<Vec<f64>> {
    if logits.is_empty() || logits.iter().any(|value| !value.is_finite()) {
        return Err("softmax received empty or non-finite logits".into());
    }
    let maximum = logits.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mut weights = logits
        .iter()
        .map(|logit| libm::exp(*logit - maximum))
        .collect::<Vec<_>>();
    let sum = weights.iter().sum::<f64>();
    if !sum.is_finite() || sum <= 0.0 {
        return Err("softmax denominator is invalid".into());
    }
    for weight in &mut weights {
        *weight /= sum;
    }
    Ok(weights)
}

fn score_numerator_and_gradients(
    metric: HelmDLearnedManifoldMetric,
    query: &[f64],
    key: &[f64],
) -> TestResult<(f64, Vec<f64>, Vec<f64>)> {
    match metric {
        HelmDLearnedManifoldMetric::Euclidean => {
            let mut numerator = 0.0;
            let mut query_gradient = vec![0.0; query.len()];
            let mut key_gradient = vec![0.0; key.len()];
            for lane in 0..query.len() {
                let difference = query[lane] - key[lane];
                numerator -= difference * difference;
                query_gradient[lane] = -2.0 * difference;
                key_gradient[lane] = 2.0 * difference;
            }
            Ok((numerator, query_gradient, key_gradient))
        }
        HelmDLearnedManifoldMetric::Lorentz => {
            let query_norm = query.iter().map(|value| value * value).sum::<f64>();
            let key_norm = key.iter().map(|value| value * value).sum::<f64>();
            let query_time = libm::sqrt(1.0 + query_norm);
            let key_time = libm::sqrt(1.0 + key_norm);
            let dot = query
                .iter()
                .zip(key)
                .map(|(left, right)| left * right)
                .sum::<f64>();
            let numerator = 2.0 + 2.0 * (-query_time * key_time + dot);
            let query_gradient = query
                .iter()
                .zip(key)
                .map(|(query, key)| 2.0 * (*key - key_time * *query / query_time))
                .collect();
            let key_gradient = query
                .iter()
                .zip(key)
                .map(|(query, key)| 2.0 * (*query - query_time * *key / key_time))
                .collect();
            Ok((numerator, query_gradient, key_gradient))
        }
    }
}

type CentroidBackward = (Vec<f64>, Vec<f64>, Vec<Vec<f64>>);

fn centroid_forward_backward(
    metric: HelmDLearnedManifoldMetric,
    values: &[Vec<f64>],
    weights: &[f64],
    output_gradient: &[f64],
) -> TestResult<CentroidBackward> {
    let width = output_gradient.len();
    if values.is_empty()
        || values.len() != weights.len()
        || values.iter().any(|value| value.len() != width)
    {
        return Err("centroid differentiation shape mismatch".into());
    }
    if metric == HelmDLearnedManifoldMetric::Euclidean {
        let mut output = vec![0.0; width];
        for (weight, value) in weights.iter().zip(values) {
            for lane in 0..width {
                output[lane] += *weight * value[lane];
            }
        }
        let weight_gradients = values
            .iter()
            .map(|value| value.iter().zip(output_gradient).map(|(v, g)| v * g).sum())
            .collect();
        let value_gradients = weights
            .iter()
            .map(|weight| {
                output_gradient
                    .iter()
                    .map(|gradient| *weight * *gradient)
                    .collect()
            })
            .collect();
        return Ok((output, weight_gradients, value_gradients));
    }

    let times = values
        .iter()
        .map(|value| libm::sqrt(1.0 + value.iter().map(|lane| lane * lane).sum::<f64>()))
        .collect::<Vec<_>>();
    let time_sum = weights.iter().zip(&times).map(|(w, t)| w * t).sum::<f64>();
    let mut spatial_sum = vec![0.0; width];
    for (weight, value) in weights.iter().zip(values) {
        for lane in 0..width {
            spatial_sum[lane] += *weight * value[lane];
        }
    }
    let spatial_norm = libm::sqrt(spatial_sum.iter().map(|value| value * value).sum());
    let denominator = libm::sqrt((time_sum - spatial_norm) * (time_sum + spatial_norm));
    if !denominator.is_finite() || denominator <= 0.0 {
        return Err("Lorentz centroid derivative is not future timelike".into());
    }
    let output = spatial_sum
        .iter()
        .map(|coordinate| *coordinate / denominator)
        .collect::<Vec<_>>();
    let contraction = output_gradient
        .iter()
        .zip(&spatial_sum)
        .map(|(left, right)| left * right)
        .sum::<f64>();
    let denominator_cubed = denominator * denominator * denominator;
    let spatial_gradient = output_gradient
        .iter()
        .zip(&spatial_sum)
        .map(|(gradient, spatial)| {
            gradient / denominator + contraction * spatial / denominator_cubed
        })
        .collect::<Vec<_>>();
    let time_gradient = -contraction * time_sum / denominator_cubed;
    let weight_gradients = values
        .iter()
        .zip(&times)
        .map(|(value, time)| {
            value
                .iter()
                .zip(&spatial_gradient)
                .map(|(value, gradient)| value * gradient)
                .sum::<f64>()
                + time_gradient * time
        })
        .collect::<Vec<_>>();
    let value_gradients = values
        .iter()
        .zip(weights)
        .zip(&times)
        .map(|((value, weight), time)| {
            value
                .iter()
                .zip(&spatial_gradient)
                .map(|(value, spatial)| *weight * (*spatial + time_gradient * *value / *time))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    Ok((output, weight_gradients, value_gradients))
}

fn accumulate_adapter_gradient(
    target: &mut AffineGradient,
    input: &[f32],
    output_gradient: &[f64],
    block: usize,
) {
    for row in 0..R4_WIDTH {
        let gradient = output_gradient[block * R4_WIDTH + row];
        target.bias[row] += gradient;
        for column in 0..R4_WIDTH {
            target.matrix[row][column] += gradient * f64::from(input[block * R4_WIDTH + column]);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn evaluate_row(
    parameters: &HelmDLearnedManifoldParameters,
    metric: HelmDLearnedManifoldMetric,
    document: &CapturedDocument,
    layer: usize,
    head: usize,
    position: usize,
    rope_theta: f32,
    rope_interleaved: bool,
    gradient: &mut ParameterGradient,
) -> TestResult<(f64, u64)> {
    let projection = &document.projections[position][layer];
    let query_input = &projection.query[head * HEAD_WIDTH..(head + 1) * HEAD_WIDTH];
    let mut query = apply_adapters(
        parameters.query_adapters(),
        layer,
        head,
        EXPECTED_QUERY_HEADS,
        query_input,
    )?;
    apply_rope_f64(&mut query, position, rope_theta, rope_interleaved);

    let kv_head = head / (EXPECTED_QUERY_HEADS / EXPECTED_KV_HEADS);
    let mut keys = Vec::with_capacity(position + 1);
    let mut values = Vec::with_capacity(position + 1);
    let mut key_inputs = Vec::with_capacity(position + 1);
    let mut value_inputs = Vec::with_capacity(position + 1);
    for source in 0..=position {
        let source_projection = &document.projections[source][layer];
        let key_input =
            source_projection.key[kv_head * HEAD_WIDTH..(kv_head + 1) * HEAD_WIDTH].to_vec();
        let value_input =
            source_projection.value[kv_head * HEAD_WIDTH..(kv_head + 1) * HEAD_WIDTH].to_vec();
        let mut key = apply_adapters(
            parameters.key_adapters(),
            layer,
            kv_head,
            EXPECTED_KV_HEADS,
            &key_input,
        )?;
        apply_rope_f64(&mut key, source, rope_theta, rope_interleaved);
        let value = apply_adapters(
            parameters.value_adapters(),
            layer,
            kv_head,
            EXPECTED_KV_HEADS,
            &value_input,
        )?;
        key_inputs.push(key_input);
        value_inputs.push(value_input);
        keys.push(key);
        values.push(value);
    }

    let scale = parameters.learned_scale(layer)?;
    let bias = parameters.learned_bias(layer)?;
    let mut numerators = Vec::with_capacity(position + 1);
    let mut query_numerator_gradients = Vec::with_capacity(position + 1);
    let mut key_numerator_gradients = Vec::with_capacity(position + 1);
    let mut logits = Vec::with_capacity(position + 1);
    for key in &keys {
        let (numerator, query_gradient, key_gradient) =
            score_numerator_and_gradients(metric, &query, key)?;
        let public_logit = helm_d_learned_manifold_logit(metric, &query, key, scale, bias)?;
        let logit = numerator / scale + bias;
        if (public_logit - logit).abs() > 1.0e-12 {
            return Err("analytic score drifted from the public construction operator".into());
        }
        numerators.push(numerator);
        query_numerator_gradients.push(query_gradient);
        key_numerator_gradients.push(key_gradient);
        logits.push(logit);
    }
    let weights = stable_softmax_f64(&logits)?;
    let donor_weights = document.heads[position][layer][head]
        .donor_weights
        .iter()
        .map(|weight| f64::from(*weight))
        .collect::<Vec<_>>();
    let donor_weight_sum = donor_weights.iter().sum::<f64>();
    let cross_entropy = donor_weights
        .iter()
        .zip(&weights)
        .map(|(donor, learned)| -*donor * libm::log(*learned))
        .sum::<f64>();

    let learned_aggregate = helm_d_learned_manifold_centroid(metric, &values, &weights)?;
    let donor_aggregate = &document.heads[position][layer][head].donor_aggregate;
    let donor_norm = libm::sqrt(
        donor_aggregate
            .iter()
            .map(|value| f64::from(*value) * f64::from(*value))
            .sum::<f64>(),
    );
    let normalization = donor_norm.max(1.0);
    let mut aggregate_gradient = vec![0.0; HEAD_WIDTH];
    let mut aggregate_loss = 0.0;
    for lane in 0..HEAD_WIDTH {
        let difference = learned_aggregate[lane] - f64::from(donor_aggregate[lane]);
        aggregate_loss += difference * difference / (normalization * normalization);
        aggregate_gradient[lane] =
            2.0 * difference / (HEAD_WIDTH as f64 * normalization * normalization);
    }
    aggregate_loss /= HEAD_WIDTH as f64;
    let (differentiated_aggregate, weight_gradient, value_gradients) =
        centroid_forward_backward(metric, &values, &weights, &aggregate_gradient)?;
    if learned_aggregate
        .iter()
        .zip(&differentiated_aggregate)
        .any(|(left, right)| (*left - *right).abs() > 2.0e-12 * (1.0 + left.abs()))
    {
        return Err("analytic centroid drifted from the public construction operator".into());
    }

    let weighted_weight_gradient = weights
        .iter()
        .zip(&weight_gradient)
        .map(|(weight, gradient)| weight * gradient)
        .sum::<f64>();
    let mut logit_gradients = Vec::with_capacity(weights.len());
    for source in 0..weights.len() {
        let cross_entropy_gradient = weights[source] * donor_weight_sum - donor_weights[source];
        let aggregate_weight_gradient =
            weights[source] * (weight_gradient[source] - weighted_weight_gradient);
        logit_gradients.push(cross_entropy_gradient + aggregate_weight_gradient);
    }

    let mut query_post_gradient = vec![0.0; HEAD_WIDTH];
    for source in 0..=position {
        let logit_gradient = logit_gradients[source];
        for lane in 0..HEAD_WIDTH {
            query_post_gradient[lane] +=
                logit_gradient * query_numerator_gradients[source][lane] / scale;
        }
        let key_post_gradient = key_numerator_gradients[source]
            .iter()
            .map(|gradient| logit_gradient * gradient / scale)
            .collect::<Vec<_>>();
        let key_adapter_gradient =
            rope_gradient_to_input(&key_post_gradient, source, rope_theta, rope_interleaved);
        let key_base = adapter_index(layer, kv_head, 0, EXPECTED_KV_HEADS);
        let value_base = adapter_index(layer, kv_head, 0, EXPECTED_KV_HEADS);
        for block in 0..BLOCKS_PER_HEAD {
            accumulate_adapter_gradient(
                &mut gradient.key[key_base + block],
                &key_inputs[source],
                &key_adapter_gradient,
                block,
            );
            accumulate_adapter_gradient(
                &mut gradient.value[value_base + block],
                &value_inputs[source],
                &value_gradients[source],
                block,
            );
        }
        gradient.scale[layer] += logit_gradient * -numerators[source] / (scale * scale);
        gradient.bias[layer] += logit_gradient;
    }
    let query_adapter_gradient =
        rope_gradient_to_input(&query_post_gradient, position, rope_theta, rope_interleaved);
    let query_base = adapter_index(layer, head, 0, EXPECTED_QUERY_HEADS);
    for block in 0..BLOCKS_PER_HEAD {
        accumulate_adapter_gradient(
            &mut gradient.query[query_base + block],
            query_input,
            &query_adapter_gradient,
            block,
        );
    }
    let loss = cross_entropy + aggregate_loss;
    if !loss.is_finite() || !gradient.all_finite() {
        return Err("construction objective or gradient is non-finite".into());
    }
    Ok((loss, u64::try_from(position + 1)?))
}

#[allow(clippy::too_many_arguments)]
fn evaluate_shard(
    parameters: &HelmDLearnedManifoldParameters,
    metric: HelmDLearnedManifoldMetric,
    documents: &[CapturedDocument],
    document_limit: usize,
    shard: usize,
    rope_theta: f32,
    rope_interleaved: bool,
) -> TestResult<ShardEvaluation> {
    let mut evaluation = ShardEvaluation {
        loss: 0.0,
        rows: 0,
        source_pairs: 0,
        gradient: ParameterGradient::zero(parameters),
    };
    // Frozen order inside a shard: layer, head, query position, document,
    // source position, block, lane. Shards are reduced separately in 0..8.
    for layer in 0..EXPECTED_LAYERS {
        for head in 0..EXPECTED_QUERY_HEADS {
            for position in SCORE_START..INPUT_POSITIONS {
                for (document_index, document) in documents[..document_limit].iter().enumerate() {
                    let ordinal = (((document_index * EXPECTED_LAYERS + layer)
                        * EXPECTED_QUERY_HEADS
                        + head)
                        * SCORE_POSITIONS)
                        + (position - SCORE_START);
                    if ordinal % SHARDS != shard {
                        continue;
                    }
                    let (loss, source_pairs) = evaluate_row(
                        parameters,
                        metric,
                        document,
                        layer,
                        head,
                        position,
                        rope_theta,
                        rope_interleaved,
                        &mut evaluation.gradient,
                    )?;
                    evaluation.loss += loss;
                    evaluation.rows += 1;
                    evaluation.source_pairs += source_pairs;
                }
            }
        }
    }
    Ok(evaluation)
}

fn ridge_objective_and_gradient(
    parameters: &HelmDLearnedManifoldParameters,
    gradient: &mut ParameterGradient,
) -> f64 {
    let mut objective = 0.0;
    for (adapters, gradients) in [
        (parameters.query_adapters(), &mut gradient.query),
        (parameters.key_adapters(), &mut gradient.key),
        (parameters.value_adapters(), &mut gradient.value),
    ] {
        for (adapter, target) in adapters.iter().zip(gradients.iter_mut()) {
            for row in 0..R4_WIDTH {
                for column in 0..R4_WIDTH {
                    let expected = if row == column { 1.0 } else { 0.0 };
                    let difference = adapter.matrix[row][column] - expected;
                    objective += RIDGE * difference * difference;
                    target.matrix[row][column] += 2.0 * RIDGE * difference;
                }
                objective += RIDGE * adapter.bias[row] * adapter.bias[row];
                target.bias[row] += 2.0 * RIDGE * adapter.bias[row];
            }
        }
    }
    objective
}

fn evaluate_dataset(
    parameters: &HelmDLearnedManifoldParameters,
    metric: HelmDLearnedManifoldMetric,
    documents: &[CapturedDocument],
    document_limit: usize,
    rope_theta: f32,
    rope_interleaved: bool,
) -> TestResult<DatasetEvaluation> {
    if document_limit == 0 || document_limit > documents.len() {
        return Err("dataset evaluation document limit is invalid".into());
    }
    let shards = std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(SHARDS);
        for shard in 0..SHARDS {
            handles.push(scope.spawn(move || {
                evaluate_shard(
                    parameters,
                    metric,
                    documents,
                    document_limit,
                    shard,
                    rope_theta,
                    rope_interleaved,
                )
            }));
        }
        handles
            .into_iter()
            .map(|handle| handle.join().map_err(|_| "construction shard panicked")?)
            .collect::<TestResult<Vec<_>>>()
    })?;
    let mut loss = 0.0;
    let mut rows = 0_u64;
    let mut gradient = ParameterGradient::zero(parameters);
    let mut rows_per_shard = Vec::with_capacity(SHARDS);
    let mut source_pairs_per_shard = Vec::with_capacity(SHARDS);
    for shard in &shards {
        loss += shard.loss;
        rows += shard.rows;
        gradient.add_assign(&shard.gradient);
        rows_per_shard.push(shard.rows);
        source_pairs_per_shard.push(shard.source_pairs);
    }
    let expected_rows =
        u64::try_from(document_limit * SCORE_POSITIONS * EXPECTED_LAYERS * EXPECTED_QUERY_HEADS)?;
    if rows != expected_rows || rows_per_shard.contains(&0) {
        return Err("eight-shard work ledger is incomplete".into());
    }
    let reciprocal_rows = 1.0 / rows as f64;
    loss *= reciprocal_rows;
    gradient.scale_by(reciprocal_rows);
    loss += ridge_objective_and_gradient(parameters, &mut gradient);
    if !loss.is_finite() || !gradient.all_finite() {
        return Err("reduced construction objective or gradient is non-finite".into());
    }
    Ok(DatasetEvaluation {
        objective: loss,
        gradient,
        rows_per_shard,
        source_pairs_per_shard,
    })
}

fn update_scalar(
    parameter: &mut f64,
    gradient: f64,
    first_moment: &mut f64,
    second_moment: &mut f64,
    step: usize,
) {
    *first_moment = ADAM_BETA1 * *first_moment + (1.0 - ADAM_BETA1) * gradient;
    *second_moment = ADAM_BETA2 * *second_moment + (1.0 - ADAM_BETA2) * gradient * gradient;
    let first_hat = *first_moment / (1.0 - libm::pow(ADAM_BETA1, step as f64));
    let second_hat = *second_moment / (1.0 - libm::pow(ADAM_BETA2, step as f64));
    *parameter -= LEARNING_RATE * first_hat / (libm::sqrt(second_hat) + ADAM_EPSILON);
}

fn adam_step(
    parameters: &mut HelmDLearnedManifoldParameters,
    gradient: &ParameterGradient,
    first: &mut ParameterGradient,
    second: &mut ParameterGradient,
    step: usize,
) -> TestResult {
    fn update_adapters(
        adapters: &mut [R4AffineAdapter],
        gradients: &[AffineGradient],
        first: &mut [AffineGradient],
        second: &mut [AffineGradient],
        step: usize,
    ) {
        for (((adapter, gradient), first), second) in
            adapters.iter_mut().zip(gradients).zip(first).zip(second)
        {
            for row in 0..R4_WIDTH {
                for column in 0..R4_WIDTH {
                    update_scalar(
                        &mut adapter.matrix[row][column],
                        gradient.matrix[row][column],
                        &mut first.matrix[row][column],
                        &mut second.matrix[row][column],
                        step,
                    );
                }
                update_scalar(
                    &mut adapter.bias[row],
                    gradient.bias[row],
                    &mut first.bias[row],
                    &mut second.bias[row],
                    step,
                );
            }
        }
    }
    update_adapters(
        parameters.query_adapters_mut(),
        &gradient.query,
        &mut first.query,
        &mut second.query,
        step,
    );
    update_adapters(
        parameters.key_adapters_mut(),
        &gradient.key,
        &mut first.key,
        &mut second.key,
        step,
    );
    update_adapters(
        parameters.value_adapters_mut(),
        &gradient.value,
        &mut first.value,
        &mut second.value,
        step,
    );
    for layer in 0..parameters.layers() {
        update_scalar(
            &mut parameters.learned_scales_mut()[layer],
            gradient.scale[layer],
            &mut first.scale[layer],
            &mut second.scale[layer],
            step,
        );
        parameters.learned_scales_mut()[layer] =
            parameters.learned_scales()[layer].max(SCALE_FLOOR);
        update_scalar(
            &mut parameters.learned_biases_mut()[layer],
            gradient.bias[layer],
            &mut first.bias[layer],
            &mut second.bias[layer],
            step,
        );
    }
    parameters.validate()?;
    Ok(())
}

fn aggregate_trace_cid(documents: &[CapturedDocument]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4.helm-d-learned-manifold.fit-traces/2\0");
    for document in documents {
        hasher.update(document.document.id.as_bytes());
        hasher.update(document.trace_cid.as_bytes());
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn evaluation_gradient_identity(
    evaluation: &DatasetEvaluation,
    phase: &str,
    ordinal: usize,
) -> (String, f64) {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4.helm-d-learned-manifold.gradient-evaluation/2\0");
    hasher.update(&u64::try_from(phase.len()).unwrap_or(u64::MAX).to_le_bytes());
    hasher.update(phase.as_bytes());
    hasher.update(&u64::try_from(ordinal).unwrap_or(u64::MAX).to_le_bytes());
    hasher.update(&evaluation.objective.to_bits().to_le_bytes());
    for rows in &evaluation.rows_per_shard {
        hasher.update(&rows.to_le_bytes());
    }
    for pairs in &evaluation.source_pairs_per_shard {
        hasher.update(&pairs.to_le_bytes());
    }
    let mut maximum_absolute = 0.0_f64;
    evaluation
        .gradient
        .bind_identity_and_maximum(&mut hasher, &mut maximum_absolute);
    (
        format!("blake3:{}", hasher.finalize().to_hex()),
        maximum_absolute,
    )
}

fn public_parameter_identity(parameters: &HelmDLearnedManifoldParameters) -> TestResult<String> {
    Ok(parameters.parameter_identity()?)
}

fn fit_arm(
    metric: HelmDLearnedManifoldMetric,
    documents: &[CapturedDocument],
    document_limit: usize,
    rope_theta: f32,
    rope_interleaved: bool,
    steps: usize,
) -> TestResult<FittedArm> {
    let mut parameters = HelmDLearnedManifoldParameters::identity(
        EXPECTED_LAYERS,
        EXPECTED_QUERY_HEADS,
        EXPECTED_KV_HEADS,
        BLOCKS_PER_HEAD,
        INITIAL_SCALE,
    )?;
    if parameters.scalar_parameter_count()? != PARAMETER_SCALARS {
        return Err("learned parameter capacity differs from 144,060 scalars".into());
    }
    let initial = evaluate_dataset(
        &parameters,
        metric,
        documents,
        document_limit,
        rope_theta,
        rope_interleaved,
    )?;
    let mut first = ParameterGradient::zero(&parameters);
    let mut second = ParameterGradient::zero(&parameters);
    let rows_per_shard = initial.rows_per_shard.clone();
    let source_pairs_per_shard = initial.source_pairs_per_shard.clone();
    let (initial_gradient_cid, mut maximum_absolute_gradient) =
        evaluation_gradient_identity(&initial, "initial-diagnostic", 0);
    let mut ordered_evaluation_cids = vec![initial_gradient_cid];
    for step in 1..=steps {
        let evaluation = evaluate_dataset(
            &parameters,
            metric,
            documents,
            document_limit,
            rope_theta,
            rope_interleaved,
        )?;
        if evaluation.rows_per_shard != rows_per_shard
            || evaluation.source_pairs_per_shard != source_pairs_per_shard
        {
            return Err("deterministic shard schedule changed during fitting".into());
        }
        let (gradient_cid, maximum) =
            evaluation_gradient_identity(&evaluation, "optimizer-step", step);
        ordered_evaluation_cids.push(gradient_cid);
        maximum_absolute_gradient = maximum_absolute_gradient.max(maximum);
        adam_step(
            &mut parameters,
            &evaluation.gradient,
            &mut first,
            &mut second,
            step,
        )?;
    }
    let final_evaluation = evaluate_dataset(
        &parameters,
        metric,
        documents,
        document_limit,
        rope_theta,
        rope_interleaved,
    )?;
    if final_evaluation.rows_per_shard != rows_per_shard
        || final_evaluation.source_pairs_per_shard != source_pairs_per_shard
    {
        return Err("deterministic shard schedule changed during final evaluation".into());
    }
    let (final_gradient_cid, final_maximum) =
        evaluation_gradient_identity(&final_evaluation, "final-diagnostic", steps + 1);
    ordered_evaluation_cids.push(final_gradient_cid);
    maximum_absolute_gradient = maximum_absolute_gradient.max(final_maximum);
    let parameter_bytes = canonical_json_bytes(&parameters)?;
    let parameter_cid = public_parameter_identity(&parameters)?;
    let rows_per_step = rows_per_shard.iter().sum::<u64>();
    let source_pairs_per_step = source_pairs_per_shard.iter().sum::<u64>();
    let full_batch_evaluations = steps.checked_add(2).ok_or("evaluation count overflow")?;
    let gradient_audit = GradientAudit {
        schema: "uor-r4.helm-d-learned-manifold-gradient-audit/2".to_owned(),
        gradient_evaluations: full_batch_evaluations,
        optimizer_gradient_evaluations: steps,
        diagnostic_gradient_evaluations: 2,
        maximum_absolute_gradient,
        ordered_evaluation_cids,
    };
    if !gradient_audit.maximum_absolute_gradient.is_finite()
        || gradient_audit.ordered_evaluation_cids.len() != full_batch_evaluations
    {
        return Err("gradient audit is incomplete or non-finite".into());
    }
    let gradient_audit_cid = canonical_json_cid(&gradient_audit)?;
    let mut report = FitReport {
        metric,
        optimizer: format!(
            "full-batch-f64-adam(lr={LEARNING_RATE},beta1={ADAM_BETA1},beta2={ADAM_BETA2},epsilon={ADAM_EPSILON},ridge={RIDGE},scale_floor={SCALE_FLOOR})"
        ),
        steps,
        workers: SHARDS,
        fit_documents: document_limit,
        fit_rows: rows_per_step,
        initial_objective: initial.objective,
        final_objective: final_evaluation.objective,
        parameter_scalars: parameters.scalar_parameter_count()?,
        parameter_cid,
        donor_trace_cid: aggregate_trace_cid(&documents[..document_limit]),
        gradient_audit,
        gradient_audit_cid,
        work: WorkLedger {
            workers: SHARDS,
            full_batch_steps: steps,
            full_batch_evaluations,
            rows_per_shard_per_evaluation: rows_per_shard,
            source_pairs_per_shard_per_evaluation: source_pairs_per_shard,
            total_row_evaluations: rows_per_step.saturating_mul(full_batch_evaluations as u64),
            total_source_pair_evaluations: source_pairs_per_step
                .saturating_mul(full_batch_evaluations as u64),
        },
        report_cid: String::new(),
    };
    report.report_cid = fit_report_cid(&report)?;
    Ok(FittedArm {
        parameters,
        parameter_bytes,
        report,
    })
}

#[derive(Clone, Debug)]
struct SyntheticGradient {
    query: Vec<f64>,
    keys: Vec<Vec<f64>>,
    values: Vec<Vec<f64>>,
    scale: f64,
}

fn synthetic_objective_and_gradient(
    metric: HelmDLearnedManifoldMetric,
    query: &[f64],
    keys: &[Vec<f64>],
    values: &[Vec<f64>],
    scale: f64,
) -> TestResult<(f64, SyntheticGradient)> {
    let donor_weights = [0.61_f64, 0.39];
    let donor_aggregate = [0.13_f64, -0.07, 0.23, 0.31];
    let mut numerators = Vec::new();
    let mut query_partials = Vec::new();
    let mut key_partials = Vec::new();
    let mut logits = Vec::new();
    for key in keys {
        let (numerator, query_partial, key_partial) =
            score_numerator_and_gradients(metric, query, key)?;
        numerators.push(numerator);
        query_partials.push(query_partial);
        key_partials.push(key_partial);
        logits.push(numerator / scale);
    }
    let weights = stable_softmax_f64(&logits)?;
    let cross_entropy = donor_weights
        .iter()
        .zip(&weights)
        .map(|(donor, learned)| -*donor * libm::log(*learned))
        .sum::<f64>();
    let aggregate = helm_d_learned_manifold_centroid(metric, values, &weights)?;
    let mut aggregate_gradient = vec![0.0; query.len()];
    let mut aggregate_loss = 0.0;
    for lane in 0..query.len() {
        let difference = aggregate[lane] - donor_aggregate[lane];
        aggregate_loss += difference * difference / query.len() as f64;
        aggregate_gradient[lane] = 2.0 * difference / query.len() as f64;
    }
    let (_, weight_gradient, value_gradient) =
        centroid_forward_backward(metric, values, &weights, &aggregate_gradient)?;
    let mean_weight_gradient = weights
        .iter()
        .zip(&weight_gradient)
        .map(|(weight, gradient)| weight * gradient)
        .sum::<f64>();
    let mut logit_gradient = Vec::new();
    for source in 0..weights.len() {
        logit_gradient.push(
            weights[source] - donor_weights[source]
                + weights[source] * (weight_gradient[source] - mean_weight_gradient),
        );
    }
    let mut query_gradient = vec![0.0; query.len()];
    let mut key_gradient = vec![vec![0.0; query.len()]; keys.len()];
    let mut scale_gradient = 0.0;
    for source in 0..keys.len() {
        for lane in 0..query.len() {
            query_gradient[lane] += logit_gradient[source] * query_partials[source][lane] / scale;
            key_gradient[source][lane] =
                logit_gradient[source] * key_partials[source][lane] / scale;
        }
        scale_gradient += logit_gradient[source] * -numerators[source] / (scale * scale);
    }
    Ok((
        cross_entropy + aggregate_loss,
        SyntheticGradient {
            query: query_gradient,
            keys: key_gradient,
            values: value_gradient,
            scale: scale_gradient,
        },
    ))
}

fn finite_difference_gradient_preflight() -> TestResult<f64> {
    const STEP: f64 = 1.0e-6;
    const TOLERANCE: f64 = 3.0e-5;
    let query = vec![0.17, -0.31, 0.23, 0.41];
    let keys = vec![vec![0.11, -0.19, 0.29, 0.37], vec![-0.43, 0.07, 0.13, 0.31]];
    let values = vec![vec![0.27, 0.17, -0.09, 0.33], vec![-0.21, 0.41, 0.05, 0.19]];
    let scale = 2.7;
    let mut maximum_error = 0.0_f64;
    for metric in [
        HelmDLearnedManifoldMetric::Lorentz,
        HelmDLearnedManifoldMetric::Euclidean,
    ] {
        let (_, analytic) =
            synthetic_objective_and_gradient(metric, &query, &keys, &values, scale)?;
        for lane in 0..query.len() {
            let mut plus = query.clone();
            let mut minus = query.clone();
            plus[lane] += STEP;
            minus[lane] -= STEP;
            let plus = synthetic_objective_and_gradient(metric, &plus, &keys, &values, scale)?.0;
            let minus = synthetic_objective_and_gradient(metric, &minus, &keys, &values, scale)?.0;
            maximum_error =
                maximum_error.max(((plus - minus) / (2.0 * STEP) - analytic.query[lane]).abs());
        }
        for source in 0..keys.len() {
            for lane in 0..query.len() {
                let mut plus = keys.clone();
                let mut minus = keys.clone();
                plus[source][lane] += STEP;
                minus[source][lane] -= STEP;
                let plus =
                    synthetic_objective_and_gradient(metric, &query, &plus, &values, scale)?.0;
                let minus =
                    synthetic_objective_and_gradient(metric, &query, &minus, &values, scale)?.0;
                maximum_error = maximum_error
                    .max(((plus - minus) / (2.0 * STEP) - analytic.keys[source][lane]).abs());
            }
        }
        for source in 0..values.len() {
            for lane in 0..query.len() {
                let mut plus = values.clone();
                let mut minus = values.clone();
                plus[source][lane] += STEP;
                minus[source][lane] -= STEP;
                let plus = synthetic_objective_and_gradient(metric, &query, &keys, &plus, scale)?.0;
                let minus =
                    synthetic_objective_and_gradient(metric, &query, &keys, &minus, scale)?.0;
                maximum_error = maximum_error
                    .max(((plus - minus) / (2.0 * STEP) - analytic.values[source][lane]).abs());
            }
        }
        let plus =
            synthetic_objective_and_gradient(metric, &query, &keys, &values, scale + STEP)?.0;
        let minus =
            synthetic_objective_and_gradient(metric, &query, &keys, &values, scale - STEP)?.0;
        maximum_error = maximum_error.max(((plus - minus) / (2.0 * STEP) - analytic.scale).abs());
    }
    if maximum_error > TOLERANCE {
        return Err(format!(
            "central finite-difference gradient error {maximum_error} exceeds {TOLERANCE}"
        )
        .into());
    }
    Ok(maximum_error)
}

fn identity_projection_ordering_preflight(
    captures: &[CapturedDocument],
    rope_theta: f32,
    rope_interleaved: bool,
) -> TestResult<u64> {
    let mut compared_lanes = 0_u64;
    let expected_projection_calls = u64::try_from(INPUT_POSITIONS * EXPECTED_LAYERS)?;
    for document in captures {
        if document.projection_audit.hook_calls != expected_projection_calls
            || document.projection_audit.query_vectors
                != u64::try_from(INPUT_POSITIONS * EXPECTED_LAYERS * EXPECTED_QUERY_HEADS)?
            || document.projection_audit.key_vectors
                != u64::try_from(INPUT_POSITIONS * EXPECTED_LAYERS * EXPECTED_KV_HEADS)?
            || document.projection_audit.value_vectors
                != u64::try_from(INPUT_POSITIONS * EXPECTED_LAYERS * EXPECTED_KV_HEADS)?
            || document.decoder_audit.future_reads != 0
        {
            return Err("identity projection trace audit is incomplete or noncausal".into());
        }
        for position in 0..INPUT_POSITIONS {
            for layer in 0..EXPECTED_LAYERS {
                let projection = &document.projections[position][layer];
                for head in 0..EXPECTED_QUERY_HEADS {
                    let mut query =
                        projection.query[head * HEAD_WIDTH..(head + 1) * HEAD_WIDTH].to_vec();
                    apply_rope_f32(&mut query, position, rope_theta, rope_interleaved);
                    let kv_head = head / (EXPECTED_QUERY_HEADS / EXPECTED_KV_HEADS);
                    let mut key =
                        projection.key[kv_head * HEAD_WIDTH..(kv_head + 1) * HEAD_WIDTH].to_vec();
                    apply_rope_f32(&mut key, position, rope_theta, rope_interleaved);
                    let value = &projection.value[kv_head * HEAD_WIDTH..(kv_head + 1) * HEAD_WIDTH];
                    let observed = &document.heads[position][layer][head];
                    if query
                        .iter()
                        .zip(&observed.post_rope_query)
                        .chain(key.iter().zip(&observed.post_rope_current_key))
                        .chain(value.iter().zip(&observed.current_value))
                        .any(|(expected, actual)| expected.to_bits() != actual.to_bits())
                    {
                        return Err(
                            "identity adapter trace does not prove pre-RoPE ordering".into()
                        );
                    }
                    compared_lanes += u64::try_from(3 * HEAD_WIDTH)?;
                }
            }
        }
    }
    Ok(compared_lanes)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct GeometryPreflight {
    golden_maximum_error: f64,
    finite_difference_maximum_error: f64,
    identity_ordering_compared_lanes: u64,
    registered_frames: usize,
    maximum_score_covariance_error: f64,
    maximum_weight_covariance_error: f64,
    maximum_hyperboloid_residual: f64,
    maximum_centroid_covariance_error: f64,
    source_frame_permutation_live: bool,
}

fn pinned_golden_preflight() -> TestResult<f64> {
    let trace = helm_d_lorentz_causal_row(
        &[0.25, -0.5, 0.75],
        &[vec![0.1, -0.2, 0.3], vec![-0.4, 0.2, 0.6]],
        &[vec![0.7, 0.1, -0.2], vec![-0.3, 0.8, 0.4]],
        HelmDLorentzReferenceConfig {
            curvature: 1.0,
            learned_scale: 2.5,
            bias: -0.125,
        },
    )?;
    let expected_logits = [-0.214_615_321_377_075_56, -0.493_210_510_118_965_55];
    let expected_weights = [0.569_201_782_148_292_1, 0.430_798_217_851_707_9];
    let expected_centroid = [
        1.078_716_185_780_169_5,
        0.223_617_725_820_237_98,
        0.333_562_632_088_915_3,
        0.048_576_667_619_514_846,
    ];
    let maximum = trace
        .logits
        .iter()
        .zip(expected_logits)
        .map(|(actual, expected)| (*actual - expected).abs())
        .chain(
            trace
                .weights
                .iter()
                .zip(expected_weights)
                .map(|(actual, expected)| (*actual - expected).abs()),
        )
        .chain(
            trace
                .centroid
                .iter()
                .zip(expected_centroid)
                .map(|(actual, expected)| (*actual - expected).abs()),
        )
        .fold(0.0_f64, f64::max);
    if maximum > 1.0e-15 {
        return Err("pinned HELM-D golden vector drifted".into());
    }
    Ok(maximum)
}

fn hyperboloid_residual(spatial: &[f64]) -> f64 {
    let time = libm::sqrt(1.0 + spatial.iter().map(|value| value * value).sum::<f64>());
    (-time * time + spatial.iter().map(|value| value * value).sum::<f64>() + 1.0).abs()
}

fn registered_frame_covariance_preflight(
    maximum_token_id: u32,
) -> TestResult<(usize, f64, f64, f64, f64, bool)> {
    let frames = canonical_registered_h4_spin_frames()?;
    if frames.len() != 120
        || frames
            .iter()
            .enumerate()
            .any(|(offset, frame)| usize::from(frame.h4_table_offset()) != offset)
    {
        return Err("canonical H4 registry did not enumerate all 120 frames in table order".into());
    }
    let query = (0..HEAD_WIDTH)
        .map(|lane| (lane as f64 - 31.5) / 37.0)
        .collect::<Vec<_>>();
    let keys = [
        (0..HEAD_WIDTH)
            .map(|lane| (lane as f64 - 17.0) / 41.0)
            .collect::<Vec<_>>(),
        (0..HEAD_WIDTH)
            .map(|lane| (23.0 - lane as f64) / 43.0)
            .collect::<Vec<_>>(),
    ];
    let values = [
        (0..HEAD_WIDTH)
            .map(|lane| (lane as f64 - 7.0) / 47.0)
            .collect::<Vec<_>>(),
        (0..HEAD_WIDTH)
            .map(|lane| (11.0 - lane as f64) / 53.0)
            .collect::<Vec<_>>(),
    ];
    let baseline_logits = keys
        .iter()
        .map(|key| {
            helm_d_learned_manifold_logit(
                HelmDLearnedManifoldMetric::Lorentz,
                &query,
                key,
                2.5,
                -0.125,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let baseline_weights = stable_softmax_f64(&baseline_logits)?;
    let baseline_centroid = helm_d_learned_manifold_centroid(
        HelmDLearnedManifoldMetric::Lorentz,
        &values,
        &baseline_weights,
    )?;
    let mut maximum_score = 0.0_f64;
    let mut maximum_weight = 0.0_f64;
    let mut maximum_centroid = 0.0_f64;
    let mut maximum_output = 0.0_f64;
    for frame in &frames {
        let encode = |vector: &[f64]| -> TestResult<Vec<f64>> {
            let mut encoded = Vec::with_capacity(HEAD_WIDTH);
            for block in vector.chunks_exact(R4_WIDTH) {
                let block = [block[0], block[1], block[2], block[3]];
                encoded.extend_from_slice(&frame.encode_model_block(block)?);
            }
            Ok(encoded)
        };
        let encoded_query = encode(&query)?;
        let encoded_keys = keys
            .iter()
            .map(|key| encode(key))
            .collect::<TestResult<Vec<_>>>()?;
        let encoded_values = values
            .iter()
            .map(|value| encode(value))
            .collect::<TestResult<Vec<_>>>()?;
        let encoded_logits = encoded_keys
            .iter()
            .map(|key| {
                helm_d_learned_manifold_logit(
                    HelmDLearnedManifoldMetric::Lorentz,
                    &encoded_query,
                    key,
                    2.5,
                    -0.125,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let encoded_weights = stable_softmax_f64(&encoded_logits)?;
        let encoded_centroid = helm_d_learned_manifold_centroid(
            HelmDLearnedManifoldMetric::Lorentz,
            &encoded_values,
            &encoded_weights,
        )?;
        let mut decoded = Vec::with_capacity(HEAD_WIDTH);
        for block in encoded_centroid.chunks_exact(R4_WIDTH) {
            decoded.extend_from_slice(
                &frame.decode_local_block([block[0], block[1], block[2], block[3]])?,
            );
        }
        maximum_score = maximum_score.max(
            baseline_logits
                .iter()
                .zip(&encoded_logits)
                .map(|(left, right)| (*left - *right).abs())
                .fold(0.0, f64::max),
        );
        maximum_weight = maximum_weight.max(
            baseline_weights
                .iter()
                .zip(&encoded_weights)
                .map(|(left, right)| (*left - *right).abs())
                .fold(0.0, f64::max),
        );
        maximum_centroid = maximum_centroid.max(
            hyperboloid_residual(&encoded_centroid).max(hyperboloid_residual(&baseline_centroid)),
        );
        maximum_output = maximum_output.max(
            baseline_centroid
                .iter()
                .zip(&decoded)
                .map(|(left, right)| (*left - *right).abs())
                .fold(0.0, f64::max),
        );
    }
    if maximum_score > 1.0e-8
        || maximum_weight > 1.0e-8
        || maximum_centroid > 1.0e-8
        || maximum_output > 1.0e-8
    {
        return Err("registered-frame Lorentz covariance exceeds the frozen bound".into());
    }

    // The exhaustive covariance census above is deliberately synthetic. Keep
    // destructive-control liveness on a separate, naturally reached causal
    // atlas so direct registry enumeration cannot make the intervention live.
    let mut atlas = R4SpinFrameAtlas::new(maximum_token_id, 3)?;
    for (position, token) in [5, 9, 2].into_iter().enumerate() {
        atlas.begin_position(token, position)?;
    }
    let source_position = 0;
    let query_position = 2;
    let block = [0.25, -0.5, 0.75, 0.125];
    let local = atlas.encode_model_block(source_position, block)?;
    let coherent = atlas.transport_local_block(
        source_position,
        query_position,
        local,
        R4SpinTransportIntervention::Coherent,
        false,
    )?;
    let permuted = atlas.transport_local_block(
        source_position,
        query_position,
        local,
        R4SpinTransportIntervention::SourceFramePermuted,
        false,
    )?;
    let permutation_live = coherent
        .iter()
        .zip(permuted)
        .any(|(left, right)| (*left - right).abs() > 1.0e-8);
    if !permutation_live {
        return Err("source-frame permutation is not live".into());
    }
    Ok((
        frames.len(),
        maximum_score,
        maximum_weight,
        maximum_centroid,
        maximum_output,
        true,
    ))
}

fn construction_preflight(
    captures: &[CapturedDocument],
    rope_theta: f32,
    rope_interleaved: bool,
    registered_frame_preflight: (usize, f64, f64, f64, f64, bool),
) -> TestResult<GeometryPreflight> {
    let golden_maximum_error = pinned_golden_preflight()?;
    let finite_difference_maximum_error = finite_difference_gradient_preflight()?;
    let identity_ordering_compared_lanes =
        identity_projection_ordering_preflight(captures, rope_theta, rope_interleaved)?;
    let (registered_frames, score, weight, centroid, output, source_frame_permutation_live) =
        registered_frame_preflight;
    Ok(GeometryPreflight {
        golden_maximum_error,
        finite_difference_maximum_error,
        identity_ordering_compared_lanes,
        registered_frames,
        maximum_score_covariance_error: score,
        maximum_weight_covariance_error: weight,
        maximum_hyperboloid_residual: centroid,
        maximum_centroid_covariance_error: output,
        source_frame_permutation_live,
    })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct CanaryReport {
    lorentz_initial_objective: f64,
    lorentz_after_one_step: f64,
    euclidean_initial_objective: f64,
    euclidean_after_one_step: f64,
    rows_per_shard: Vec<u64>,
    extrapolated_complete_seconds: f64,
    passed: bool,
}

fn objective_canary(
    captures: &[CapturedDocument],
    rope_theta: f32,
    rope_interleaved: bool,
) -> TestResult<CanaryReport> {
    let started = Instant::now();
    let lorentz = fit_arm(
        HelmDLearnedManifoldMetric::Lorentz,
        captures,
        2,
        rope_theta,
        rope_interleaved,
        1,
    )?;
    let euclidean = fit_arm(
        HelmDLearnedManifoldMetric::Euclidean,
        captures,
        2,
        rope_theta,
        rope_interleaved,
        1,
    )?;
    let elapsed = started.elapsed().as_secs_f64();
    // Each one-step canary evaluates its dataset three times (initial, update,
    // final). The complete fit evaluates each arm 130 times, has eight times
    // as many documents, and repeats both independent arms for replay. Ten
    // percent is then reserved for admitted validation and result I/O.
    let extrapolated_complete_seconds = elapsed * (130.0 / 3.0) * 8.0 * 2.0 * 1.1;
    let passed = lorentz.report.final_objective < lorentz.report.initial_objective
        && euclidean.report.final_objective < euclidean.report.initial_objective
        && lorentz.report.work.rows_per_shard_per_evaluation.len() == SHARDS
        && lorentz
            .report
            .work
            .rows_per_shard_per_evaluation
            .iter()
            .all(|rows| *rows > 0)
        && lorentz.report.work.rows_per_shard_per_evaluation
            == euclidean.report.work.rows_per_shard_per_evaluation
        && extrapolated_complete_seconds <= MAX_CANARY_SECONDS;
    let report = CanaryReport {
        lorentz_initial_objective: lorentz.report.initial_objective,
        lorentz_after_one_step: lorentz.report.final_objective,
        euclidean_initial_objective: euclidean.report.initial_objective,
        euclidean_after_one_step: euclidean.report.final_objective,
        rows_per_shard: lorentz.report.work.rows_per_shard_per_evaluation,
        extrapolated_complete_seconds,
        passed,
    };
    if !report.passed {
        return Err(format!("two-document objective/runtime canary failed: {report:?}").into());
    }
    Ok(report)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ImplementationIdentity {
    contract_cid: String,
    predecessor_partition_bytes_cid: String,
    implementation_source_cid: String,
    executable_cid: String,
    git_revision: String,
    git_tracked_tree_clean: bool,
}

fn repository_root() -> TestResult<PathBuf> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    Ok(manifest
        .parent()
        .and_then(Path::parent)
        .ok_or("core crate is not inside the expected workspace")?
        .to_path_buf())
}

fn verified_git_revision() -> TestResult<String> {
    let root = repository_root()?;
    let revision = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&root)
        .output()?;
    if !revision.status.success() {
        return Err("could not resolve the implementation Git revision".into());
    }
    let revision = String::from_utf8(revision.stdout)?.trim().to_owned();
    if revision.len() != 40 || !revision.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return Err("implementation Git revision is not a complete commit identity".into());
    }
    let status = Command::new("git")
        .args(["status", "--porcelain", "--untracked-files=no"])
        .current_dir(root)
        .output()?;
    if !status.status.success() || !status.stdout.is_empty() {
        return Err("decision execution requires a clean tracked Git tree".into());
    }
    Ok(revision)
}

fn implementation_identity() -> TestResult<ImplementationIdentity> {
    let mut source = blake3::Hasher::new();
    source.update(b"uor-r4.helm-d-learned-manifold.implementation/2\0");
    for bytes in [
        COMPILED_CORE_SOURCE,
        COMPILED_HARNESS_SOURCE,
        COMPILED_MODEL_ATTENTION_SOURCE,
        COMPILED_MODEL_SOURCE,
    ] {
        source.update(&u64::try_from(bytes.len())?.to_le_bytes());
        source.update(bytes);
    }
    let executable = env::current_exe()?;
    let git_revision = verified_git_revision()?;
    Ok(ImplementationIdentity {
        contract_cid: cid_bytes(COMPILED_CONTRACT),
        predecessor_partition_bytes_cid: cid_bytes(COMPILED_PREDECESSOR_PARTITION),
        implementation_source_cid: format!("blake3:{}", source.finalize().to_hex()),
        executable_cid: file_cid(&executable)?,
        git_revision,
        git_tracked_tree_clean: true,
    })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct FitReplayReport {
    lorentz_parameter_bytes_identical: bool,
    euclidean_parameter_bytes_identical: bool,
    lorentz_report_identical: bool,
    euclidean_report_identical: bool,
    replay_cid: String,
}

fn fit_replay_cid(report: &FitReplayReport) -> TestResult<String> {
    let mut commitment = report.clone();
    commitment.replay_cid.clear();
    canonical_json_cid(&commitment)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct FitCheckpoint {
    schema: String,
    issue: u32,
    contract_cid: String,
    population_cid: String,
    partition_cid: String,
    manifest_cid: String,
    donor_cid: String,
    tokenizer_cid: String,
    upstream_source_commit: String,
    lorentz_policy: String,
    euclidean_policy: String,
    optimizer_specification: String,
    implementation: ImplementationIdentity,
    exact_workers: usize,
    execution_preparation: TeacherExecutionPreparation,
    fit_trace_cid: String,
    preflight: GeometryPreflight,
    canary: CanaryReport,
    lorentz_parameters: HelmDLearnedManifoldParameters,
    lorentz_parameter_cid: String,
    lorentz_fit_report: FitReport,
    euclidean_parameters: HelmDLearnedManifoldParameters,
    euclidean_parameter_cid: String,
    euclidean_fit_report: FitReport,
    replay: FitReplayReport,
    validation_materialized: bool,
    future_position_reads: u64,
    target_as_input_reads: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct FitCheckpointEnvelope {
    checkpoint_cid: String,
    checkpoint: FitCheckpoint,
}

#[derive(Debug)]
struct ValidationAdmission {
    checkpoint_cid: String,
    partition_cid: String,
    manifest_cid: String,
}

fn validate_fit_report(
    report: &FitReport,
    parameters: &HelmDLearnedManifoldParameters,
    metric: HelmDLearnedManifoldMetric,
) -> TestResult {
    let rows_per_evaluation =
        u64::try_from(FIT_DOCUMENTS * SCORE_POSITIONS * EXPECTED_LAYERS * EXPECTED_QUERY_HEADS)?;
    let source_pairs_per_evaluation = u64::try_from(
        FIT_DOCUMENTS
            * EXPECTED_LAYERS
            * EXPECTED_QUERY_HEADS
            * ((SCORE_START + 1)..=INPUT_POSITIONS).sum::<usize>(),
    )?;
    let evaluations = ADAM_STEPS + 2;
    let rows_per_shard = &report.work.rows_per_shard_per_evaluation;
    let source_pairs_per_shard = &report.work.source_pairs_per_shard_per_evaluation;
    if report.metric != metric
        || report.steps != ADAM_STEPS
        || report.workers != SHARDS
        || report.fit_documents != FIT_DOCUMENTS
        || report.fit_rows != rows_per_evaluation
        || !report.initial_objective.is_finite()
        || !report.final_objective.is_finite()
        || report.parameter_scalars != PARAMETER_SCALARS
        || report.parameter_cid != public_parameter_identity(parameters)?
        || report.gradient_audit.schema != "uor-r4.helm-d-learned-manifold-gradient-audit/2"
        || report.gradient_audit.gradient_evaluations != evaluations
        || report.gradient_audit.optimizer_gradient_evaluations != ADAM_STEPS
        || report.gradient_audit.diagnostic_gradient_evaluations != 2
        || !report.gradient_audit.maximum_absolute_gradient.is_finite()
        || report.gradient_audit.maximum_absolute_gradient <= 0.0
        || report.gradient_audit.ordered_evaluation_cids.len() != evaluations
        || report
            .gradient_audit
            .ordered_evaluation_cids
            .iter()
            .any(|cid| !cid.starts_with("blake3:"))
        || report.gradient_audit_cid != canonical_json_cid(&report.gradient_audit)?
        || report.work.workers != SHARDS
        || report.work.full_batch_steps != ADAM_STEPS
        || report.work.full_batch_evaluations != evaluations
        || rows_per_shard.len() != SHARDS
        || source_pairs_per_shard.len() != SHARDS
        || rows_per_shard.contains(&0)
        || source_pairs_per_shard.contains(&0)
        || rows_per_shard.iter().sum::<u64>() != rows_per_evaluation
        || source_pairs_per_shard.iter().sum::<u64>() != source_pairs_per_evaluation
        || report.work.total_row_evaluations
            != rows_per_evaluation.saturating_mul(evaluations as u64)
        || report.work.total_source_pair_evaluations
            != source_pairs_per_evaluation.saturating_mul(evaluations as u64)
        || report.report_cid != fit_report_cid(report)?
    {
        return Err("fit report gradient/work/parameter evidence is invalid".into());
    }
    Ok(())
}

fn write_and_read_checkpoint(
    path: &Path,
    checkpoint: FitCheckpoint,
) -> TestResult<(FitCheckpointEnvelope, ValidationAdmission)> {
    let preflight_scalars = [
        checkpoint.preflight.golden_maximum_error,
        checkpoint.preflight.finite_difference_maximum_error,
        checkpoint.preflight.maximum_score_covariance_error,
        checkpoint.preflight.maximum_weight_covariance_error,
        checkpoint.preflight.maximum_hyperboloid_residual,
        checkpoint.preflight.maximum_centroid_covariance_error,
        checkpoint.canary.extrapolated_complete_seconds,
    ];
    if checkpoint.validation_materialized
        || checkpoint.schema != CHECKPOINT_SCHEMA
        || checkpoint.issue != 973
        || checkpoint.contract_cid != checkpoint.implementation.contract_cid
        || checkpoint.population_cid != CORPUS_CID
        || checkpoint.donor_cid != DONOR_CID
        || checkpoint.upstream_source_commit != HELM_D_UPSTREAM_COMMIT
        || checkpoint.lorentz_policy != HELM_D_LEARNED_LORENTZ_R4_CONSTRUCTION_POLICY
        || checkpoint.euclidean_policy != HELM_D_LEARNED_EUCLIDEAN_R4_CONTROL_POLICY
        || checkpoint.exact_workers != SHARDS
        || checkpoint.fit_trace_cid != checkpoint.lorentz_fit_report.donor_trace_cid
        || checkpoint.fit_trace_cid != checkpoint.euclidean_fit_report.donor_trace_cid
        || checkpoint.implementation.git_revision.len() != 40
        || !checkpoint.implementation.git_tracked_tree_clean
        || checkpoint.preflight.registered_frames != 120
        || checkpoint.preflight.golden_maximum_error > 1.0e-15
        || checkpoint.preflight.finite_difference_maximum_error > 3.0e-5
        || checkpoint.preflight.maximum_score_covariance_error > 1.0e-8
        || checkpoint.preflight.maximum_weight_covariance_error > 1.0e-8
        || checkpoint.preflight.maximum_hyperboloid_residual > 1.0e-8
        || checkpoint.preflight.maximum_centroid_covariance_error > 1.0e-8
        || preflight_scalars.iter().any(|value| !value.is_finite())
        || !checkpoint.preflight.source_frame_permutation_live
        || !checkpoint.canary.passed
        || checkpoint.future_position_reads != 0
        || checkpoint.target_as_input_reads != 0
        || !checkpoint.replay.lorentz_parameter_bytes_identical
        || !checkpoint.replay.euclidean_parameter_bytes_identical
        || !checkpoint.replay.lorentz_report_identical
        || !checkpoint.replay.euclidean_report_identical
        || checkpoint.replay.replay_cid != fit_replay_cid(&checkpoint.replay)?
        || checkpoint.lorentz_parameter_cid != checkpoint.lorentz_fit_report.parameter_cid
        || checkpoint.euclidean_parameter_cid != checkpoint.euclidean_fit_report.parameter_cid
        || checkpoint
            .lorentz_fit_report
            .work
            .rows_per_shard_per_evaluation
            != checkpoint
                .euclidean_fit_report
                .work
                .rows_per_shard_per_evaluation
        || checkpoint
            .lorentz_fit_report
            .work
            .source_pairs_per_shard_per_evaluation
            != checkpoint
                .euclidean_fit_report
                .work
                .source_pairs_per_shard_per_evaluation
    {
        return Err("fit checkpoint does not seal construction-only replay evidence".into());
    }
    validate_fit_report(
        &checkpoint.lorentz_fit_report,
        &checkpoint.lorentz_parameters,
        HelmDLearnedManifoldMetric::Lorentz,
    )?;
    validate_fit_report(
        &checkpoint.euclidean_fit_report,
        &checkpoint.euclidean_parameters,
        HelmDLearnedManifoldMetric::Euclidean,
    )?;
    let envelope = FitCheckpointEnvelope {
        checkpoint_cid: canonical_json_cid(&checkpoint)?,
        checkpoint,
    };
    write_pretty_json_exclusive(path, &envelope)?;
    let readback: FitCheckpointEnvelope = serde_json::from_slice(&fs::read(path)?)?;
    if readback != envelope
        || readback.checkpoint_cid != canonical_json_cid(&readback.checkpoint)?
        || canonical_json_bytes(&readback)? != canonical_json_bytes(&envelope)?
    {
        return Err("exclusive fit checkpoint readback is not byte-identical".into());
    }
    let admission = ValidationAdmission {
        checkpoint_cid: readback.checkpoint_cid.clone(),
        partition_cid: readback.checkpoint.partition_cid.clone(),
        manifest_cid: readback.checkpoint.manifest_cid.clone(),
    };
    Ok((readback, admission))
}

fn materialize_validation_documents(
    admission: &ValidationAdmission,
    partition: &FrozenPartitionEnvelope,
    corpus_path: &Path,
    tokenizer: &Tokenizer,
) -> TestResult<Vec<FrozenDocument>> {
    if admission.checkpoint_cid.is_empty()
        || admission.partition_cid != partition.manifest.partition_cid
        || admission.manifest_cid != partition.manifest_cid
    {
        return Err("typed validation admission is not bound to this partition".into());
    }
    // The whole-file identity scan is deliberately admission-gated because it
    // streams bytes belonging to the sealed validation ranges.
    verify_complete_corpus(corpus_path)?;
    materialize_documents(
        corpus_path,
        tokenizer,
        &partition.manifest.construction_validation,
        true,
    )
}

#[derive(Clone, Copy)]
enum ArmSpec<'a> {
    Donor,
    Gauge,
    Learned {
        parameters: &'a HelmDLearnedManifoldParameters,
        metric: HelmDLearnedManifoldMetric,
        intervention: HelmDLearnedManifoldIntervention,
    },
    Localized {
        parameters: &'a HelmDLearnedManifoldParameters,
        metric: HelmDLearnedManifoldMetric,
        value_readout: HelmDLearnedManifoldValueReadout,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ArmReport {
    name: String,
    policy_identity: String,
    positions: usize,
    mean_next_token_nll: f64,
    top1_correct: usize,
    top1_token_ids: Vec<u32>,
    decoded_snippets: Vec<String>,
    logits_cid: String,
    replay_identical: bool,
    target_as_input_reads: u64,
    decoder_audits: Vec<CausalAuditRecord>,
    projection_audits: Vec<ProjectionAuditRecord>,
    implementation_evidence: Vec<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct CausalAuditRecord {
    positions: u64,
    layers: u64,
    heads: u64,
    query_transforms: u64,
    key_transports: u64,
    value_transports: u64,
    output_transforms: u64,
    future_reads: u64,
    maximum_query_position: Option<usize>,
    maximum_source_position: Option<usize>,
}

impl From<CausalAttentionTransportAudit> for CausalAuditRecord {
    fn from(audit: CausalAttentionTransportAudit) -> Self {
        Self {
            positions: audit.positions,
            layers: audit.layers,
            heads: audit.heads,
            query_transforms: audit.query_transforms,
            key_transports: audit.key_transports,
            value_transports: audit.value_transports,
            output_transforms: audit.output_transforms,
            future_reads: audit.future_reads,
            maximum_query_position: audit.maximum_query_position,
            maximum_source_position: audit.maximum_source_position,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ProjectionAuditRecord {
    hook_calls: u64,
    query_vectors: u64,
    key_vectors: u64,
    value_vectors: u64,
    query_lanes: u64,
    key_lanes: u64,
    value_lanes: u64,
}

impl From<CausalAttentionProjectionAudit> for ProjectionAuditRecord {
    fn from(audit: CausalAttentionProjectionAudit) -> Self {
        Self {
            hook_calls: audit.hook_calls,
            query_vectors: audit.query_vectors,
            key_vectors: audit.key_vectors,
            value_vectors: audit.value_vectors,
            query_lanes: audit.query_lanes,
            key_lanes: audit.key_lanes,
            value_lanes: audit.value_lanes,
        }
    }
}

#[derive(Clone, Debug)]
struct ArmExecution {
    report: ArmReport,
    scored_logits: Vec<Vec<f32>>,
    top1_tokens: Vec<usize>,
}

fn expected_transport_source_pairs() -> u64 {
    u64::try_from(EXPECTED_LAYERS * EXPECTED_QUERY_HEADS * (1..=INPUT_POSITIONS).sum::<usize>())
        .unwrap_or(u64::MAX)
}

fn expected_nonidentity_source_frame_blocks() -> u64 {
    u64::try_from(
        EXPECTED_LAYERS
            * EXPECTED_QUERY_HEADS
            * (1..INPUT_POSITIONS)
                .map(|position| position + 1)
                .sum::<usize>()
            * BLOCKS_PER_HEAD
            * 2,
    )
    .unwrap_or(u64::MAX)
}

fn expected_nonidentity_row_permutations() -> u64 {
    u64::try_from(
        EXPECTED_LAYERS
            * EXPECTED_QUERY_HEADS
            * (1..INPUT_POSITIONS)
                .map(|position| position + 1)
                .sum::<usize>(),
    )
    .unwrap_or(u64::MAX)
}

fn validate_transport_audit(
    audit: &uor_r4_core::helm_d_r4_attention::R4SpinTransportAudit,
    expected_source_frame_permutations: u64,
) -> bool {
    let head_rows =
        u64::try_from(INPUT_POSITIONS * EXPECTED_LAYERS * EXPECTED_QUERY_HEADS).unwrap_or(u64::MAX);
    let source_pairs = expected_transport_source_pairs();
    let blocks = u64::try_from(BLOCKS_PER_HEAD).unwrap_or(u64::MAX);
    audit.positions_prepared == INPUT_POSITIONS as u64
        && audit.r4_blocks_encoded
            == head_rows
                .saturating_mul(blocks)
                .saturating_add(source_pairs.saturating_mul(blocks).saturating_mul(2))
        && audit.key_blocks_transported == source_pairs.saturating_mul(blocks)
        && audit.value_blocks_transported == source_pairs.saturating_mul(blocks)
        && audit.output_blocks_decoded == head_rows.saturating_mul(blocks)
        && audit.future_position_reads == 0
        && audit.source_frame_permutations == expected_source_frame_permutations
}

fn validate_gauge_evidence(evidence: &R4SpinTransportEvidence, actual_policy: &str) -> TestResult {
    if evidence.schema != "uor-r4.r4-spin-transport-evidence/1"
        || evidence.policy_identity != actual_policy
        || actual_policy != HELM_D_R4_GAUGE_SOFTMAX_POLICY
        || evidence.intervention != R4SpinTransportIntervention::Coherent
        || evidence.frame_table_offsets.len() != INPUT_POSITIONS
        || evidence
            .frame_table_offsets
            .iter()
            .any(|offset| *offset >= 120)
        || !validate_transport_audit(&evidence.audit, 0)
    {
        return Err("gauge implementation evidence does not match exact frozen work".into());
    }
    Ok(())
}

fn validate_learned_evidence(
    evidence: &HelmDLearnedManifoldEvidence,
    actual_policy: &str,
    expected_parameter_identity: &str,
    metric: HelmDLearnedManifoldMetric,
    value_readout: HelmDLearnedManifoldValueReadout,
    intervention: HelmDLearnedManifoldIntervention,
) -> TestResult {
    let expected_policy = match (metric, value_readout) {
        (
            HelmDLearnedManifoldMetric::Lorentz,
            HelmDLearnedManifoldValueReadout::NormalizedLorentzCentroid,
        ) => HELM_D_LEARNED_LORENTZ_R4_CONSTRUCTION_POLICY,
        (
            HelmDLearnedManifoldMetric::Euclidean,
            HelmDLearnedManifoldValueReadout::TransportedTangentArithmeticSum,
        ) => HELM_D_LEARNED_EUCLIDEAN_R4_CONTROL_POLICY,
        (
            HelmDLearnedManifoldMetric::Lorentz,
            HelmDLearnedManifoldValueReadout::TransportedTangentArithmeticSum,
        ) => HELM_D_LEARNED_LORENTZ_TANGENT_R4_LOCALIZATION_POLICY,
        (
            HelmDLearnedManifoldMetric::Euclidean,
            HelmDLearnedManifoldValueReadout::NormalizedLorentzCentroid,
        ) => HELM_D_LEARNED_EUCLIDEAN_LORENTZ_CENTROID_R4_LOCALIZATION_POLICY,
    };
    let head_rows = u64::try_from(INPUT_POSITIONS * EXPECTED_LAYERS * EXPECTED_QUERY_HEADS)?;
    let source_pairs = expected_transport_source_pairs();
    let expected_source_frame_permutations =
        if intervention == HelmDLearnedManifoldIntervention::SourceFramePermuted {
            expected_nonidentity_source_frame_blocks()
        } else {
            0
        };
    let expected_value_permutations =
        if intervention == HelmDLearnedManifoldIntervention::ValuePermuted {
            expected_nonidentity_row_permutations()
        } else {
            0
        };
    let expected_order_key_permutations =
        if intervention == HelmDLearnedManifoldIntervention::OrderKeyShuffled {
            expected_nonidentity_row_permutations()
        } else {
            0
        };
    let audit = evidence.learned_manifold_audit;
    if evidence.schema != "uor-r4.helm-d-learned-manifold-r4-evidence/2"
        || evidence.policy_identity != actual_policy
        || actual_policy != expected_policy
        || evidence.parameter_identity != expected_parameter_identity
        || evidence.scalar_parameter_count != PARAMETER_SCALARS
        || evidence.metric != metric
        || evidence.value_readout
            != (value_readout != metric.default_value_readout()).then_some(value_readout)
        || evidence.intervention != intervention
        || evidence.frame_table_offsets.len() != INPUT_POSITIONS
        || evidence
            .frame_table_offsets
            .iter()
            .any(|offset| *offset >= 120)
        || !validate_transport_audit(
            &evidence.transport_audit,
            expected_source_frame_permutations,
        )
        || audit.projection_tuples != u64::try_from(INPUT_POSITIONS * EXPECTED_LAYERS)?
        || audit.projected_query_lanes
            != u64::try_from(INPUT_POSITIONS * EXPECTED_LAYERS * EXPECTED_QUERY_HEADS * HEAD_WIDTH)?
        || audit.projected_key_lanes
            != u64::try_from(INPUT_POSITIONS * EXPECTED_LAYERS * EXPECTED_KV_HEADS * HEAD_WIDTH)?
        || audit.projected_value_lanes
            != u64::try_from(INPUT_POSITIONS * EXPECTED_LAYERS * EXPECTED_KV_HEADS * HEAD_WIDTH)?
        || audit.score_rows != head_rows
        || audit.compatibility_pairs != source_pairs
        || audit.centroid_rows != head_rows
        || audit.centroid_source_pairs != source_pairs
        || audit.source_frame_permutations != expected_source_frame_permutations
        || audit.value_permutations != expected_value_permutations
        || audit.order_key_permutations != expected_order_key_permutations
        || audit.arithmetic_failures != 0
    {
        return Err("learned implementation evidence does not match exact frozen work".into());
    }
    Ok(())
}

fn bind_actual_policy(observed: &mut Option<String>, actual: &str) -> TestResult {
    if observed
        .as_ref()
        .is_some_and(|established| established != actual)
    {
        return Err("arm transport policy changed between validation documents".into());
    }
    if observed.is_none() {
        *observed = Some(actual.to_owned());
    }
    Ok(())
}

fn argmax(values: &[f32]) -> TestResult<usize> {
    values
        .iter()
        .enumerate()
        .max_by(|(left_index, left), (right_index, right)| {
            left.total_cmp(right)
                .then_with(|| right_index.cmp(left_index))
        })
        .map(|(index, _)| index)
        .ok_or_else(|| "argmax received an empty row".into())
}

fn next_token_nll(logits: &[f32], target: usize) -> TestResult<f64> {
    if target >= logits.len() || logits.is_empty() || logits.iter().any(|value| !value.is_finite())
    {
        return Err("next-token NLL received invalid logits or target".into());
    }
    let maximum = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let denominator = logits
        .iter()
        .map(|logit| libm::exp(f64::from(*logit - maximum)))
        .sum::<f64>();
    Ok(libm::log(denominator) + f64::from(maximum - logits[target]))
}

fn scored_logits_cid(logits: &[Vec<f32>]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"uor-r4.helm-d-learned-manifold.validation-logits/2\0");
    for row in logits {
        hasher.update(&u64::try_from(row.len()).unwrap_or(u64::MAX).to_le_bytes());
        for value in row {
            hasher.update(&value.to_bits().to_le_bytes());
        }
    }
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn run_arm_once(
    oracle: &HuggingFaceLlamaOracle,
    tokenizer: &Tokenizer,
    documents: &[FrozenDocument],
    spec: ArmSpec<'_>,
    name: &str,
) -> TestResult<ArmExecution> {
    let maximum_token_id = u32::try_from(oracle.cfg().vocab.checked_sub(1).ok_or("empty vocab")?)?;
    let mut scored_logits = Vec::with_capacity(documents.len() * SCORE_POSITIONS);
    let mut top1_tokens = Vec::with_capacity(documents.len() * SCORE_POSITIONS);
    let mut nll_sum = 0.0;
    let mut top1_correct = 0_usize;
    let mut target_reads = 0_u64;
    let mut decoder_audits = Vec::new();
    let mut projection_audits = Vec::new();
    let mut evidence = Vec::new();
    let mut observed_policy = match spec {
        ArmSpec::Donor => Some("smollm2-135m-ordinary-causal-softmax".to_owned()),
        ArmSpec::Gauge | ArmSpec::Learned { .. } | ArmSpec::Localized { .. } => None,
    };
    let expected_parameter_identity = match spec {
        ArmSpec::Learned { parameters, .. } | ArmSpec::Localized { parameters, .. } => {
            Some(public_parameter_identity(parameters)?)
        }
        ArmSpec::Donor | ArmSpec::Gauge => None,
    };
    for document in documents {
        if document.tokens.len() != REQUIRED_TOKENS {
            return Err("validation arm requires exactly 16 inputs plus one target token".into());
        }
        let mut logits = vec![0.0_f32; oracle.cfg().vocab];
        match spec {
            ArmSpec::Donor => {
                let mut state = oracle.new_state_bounded(INPUT_POSITIONS)?;
                for position in 0..INPUT_POSITIONS {
                    oracle.step_state(
                        &mut state,
                        document.tokens[position] as usize,
                        position,
                        &mut logits,
                    )?;
                    if position >= SCORE_START {
                        let target = document.tokens[position + 1] as usize;
                        target_reads += 1;
                        nll_sum += next_token_nll(&logits, target)?;
                        let top1 = argmax(&logits)?;
                        top1_correct += usize::from(top1 == target);
                        top1_tokens.push(top1);
                        scored_logits.push(logits.clone());
                    }
                }
            }
            ArmSpec::Gauge => {
                let transport = R4SpinCausalAttentionTransport::new(
                    maximum_token_id,
                    INPUT_POSITIONS,
                    R4SpinTransportIntervention::Coherent,
                )?;
                let mut session = oracle.new_causal_attention_transport_session(
                    Box::new(transport),
                    CausalAttentionLayerSelection::All,
                    INPUT_POSITIONS,
                )?;
                let actual_policy = session.policy_identity().to_owned();
                bind_actual_policy(&mut observed_policy, &actual_policy)?;
                for position in 0..INPUT_POSITIONS {
                    oracle.step_causal_attention_transport(
                        &mut session,
                        document.tokens[position] as usize,
                        position,
                        &mut logits,
                    )?;
                    if position >= SCORE_START {
                        let target = document.tokens[position + 1] as usize;
                        target_reads += 1;
                        nll_sum += next_token_nll(&logits, target)?;
                        let top1 = argmax(&logits)?;
                        top1_correct += usize::from(top1 == target);
                        top1_tokens.push(top1);
                        scored_logits.push(logits.clone());
                    }
                }
                session.transport_status()?;
                decoder_audits.push(session.audit().into());
                projection_audits.push(session.pre_rope_projection_audit().into());
                let value = session
                    .transport_implementation_evidence()?
                    .ok_or("gauge arm omitted implementation evidence")?;
                let parsed: R4SpinTransportEvidence = serde_json::from_str(&value)?;
                validate_gauge_evidence(&parsed, &actual_policy)?;
                evidence.push(serde_json::to_value(parsed)?);
            }
            ArmSpec::Learned {
                parameters,
                metric,
                intervention,
            } => {
                let transport = HelmDLearnedManifoldR4Transport::new(
                    maximum_token_id,
                    INPUT_POSITIONS,
                    parameters.clone(),
                    metric,
                    intervention,
                )?;
                let mut session = oracle.new_causal_attention_transport_session(
                    Box::new(transport),
                    CausalAttentionLayerSelection::All,
                    INPUT_POSITIONS,
                )?;
                let actual_policy = session.policy_identity().to_owned();
                bind_actual_policy(&mut observed_policy, &actual_policy)?;
                for position in 0..INPUT_POSITIONS {
                    oracle.step_causal_attention_transport(
                        &mut session,
                        document.tokens[position] as usize,
                        position,
                        &mut logits,
                    )?;
                    if position >= SCORE_START {
                        let target = document.tokens[position + 1] as usize;
                        target_reads += 1;
                        nll_sum += next_token_nll(&logits, target)?;
                        let top1 = argmax(&logits)?;
                        top1_correct += usize::from(top1 == target);
                        top1_tokens.push(top1);
                        scored_logits.push(logits.clone());
                    }
                }
                session.transport_status()?;
                decoder_audits.push(session.audit().into());
                projection_audits.push(session.pre_rope_projection_audit().into());
                let value = session
                    .transport_implementation_evidence()?
                    .ok_or("learned arm omitted implementation evidence")?;
                let parsed: HelmDLearnedManifoldEvidence = serde_json::from_str(&value)?;
                validate_learned_evidence(
                    &parsed,
                    &actual_policy,
                    expected_parameter_identity
                        .as_deref()
                        .ok_or("learned arm parameter identity is unavailable")?,
                    metric,
                    metric.default_value_readout(),
                    intervention,
                )?;
                evidence.push(serde_json::to_value(parsed)?);
            }
            ArmSpec::Localized {
                parameters,
                metric,
                value_readout,
            } => {
                let transport = HelmDLearnedManifoldR4Transport::new_with_value_readout(
                    maximum_token_id,
                    INPUT_POSITIONS,
                    parameters.clone(),
                    metric,
                    value_readout,
                    HelmDLearnedManifoldIntervention::Coherent,
                )?;
                let mut session = oracle.new_causal_attention_transport_session(
                    Box::new(transport),
                    CausalAttentionLayerSelection::All,
                    INPUT_POSITIONS,
                )?;
                let actual_policy = session.policy_identity().to_owned();
                bind_actual_policy(&mut observed_policy, &actual_policy)?;
                for position in 0..INPUT_POSITIONS {
                    oracle.step_causal_attention_transport(
                        &mut session,
                        document.tokens[position] as usize,
                        position,
                        &mut logits,
                    )?;
                    if position >= SCORE_START {
                        let target = document.tokens[position + 1] as usize;
                        target_reads += 1;
                        nll_sum += next_token_nll(&logits, target)?;
                        let top1 = argmax(&logits)?;
                        top1_correct += usize::from(top1 == target);
                        top1_tokens.push(top1);
                        scored_logits.push(logits.clone());
                    }
                }
                session.transport_status()?;
                decoder_audits.push(session.audit().into());
                projection_audits.push(session.pre_rope_projection_audit().into());
                let value = session
                    .transport_implementation_evidence()?
                    .ok_or("localized learned arm omitted implementation evidence")?;
                let parsed: HelmDLearnedManifoldEvidence = serde_json::from_str(&value)?;
                validate_learned_evidence(
                    &parsed,
                    &actual_policy,
                    expected_parameter_identity
                        .as_deref()
                        .ok_or("localized learned arm parameter identity is unavailable")?,
                    metric,
                    value_readout,
                    HelmDLearnedManifoldIntervention::Coherent,
                )?;
                evidence.push(serde_json::to_value(parsed)?);
            }
        }
    }
    let positions = documents.len() * SCORE_POSITIONS;
    if scored_logits.len() != positions || target_reads != u64::try_from(positions)? {
        return Err("arm scored work does not match the frozen 64-position ledger".into());
    }
    let top1_token_ids = top1_tokens
        .iter()
        .copied()
        .map(u32::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    let decoded_snippets = top1_token_ids
        .chunks_exact(SCORE_POSITIONS)
        .map(|tokens| tokenizer.decode(tokens))
        .collect::<Vec<_>>();
    if top1_token_ids.len() != positions || decoded_snippets.len() != documents.len() {
        return Err("bounded top-1 diagnostics are incomplete".into());
    }
    Ok(ArmExecution {
        report: ArmReport {
            name: name.to_owned(),
            policy_identity: observed_policy.ok_or("arm did not bind an actual policy identity")?,
            positions,
            mean_next_token_nll: nll_sum / positions as f64,
            top1_correct,
            top1_token_ids,
            decoded_snippets,
            logits_cid: scored_logits_cid(&scored_logits),
            replay_identical: false,
            target_as_input_reads: 0,
            decoder_audits,
            projection_audits,
            implementation_evidence: evidence,
        },
        scored_logits,
        top1_tokens,
    })
}

fn run_arm(
    oracle: &HuggingFaceLlamaOracle,
    tokenizer: &Tokenizer,
    documents: &[FrozenDocument],
    spec: ArmSpec<'_>,
    name: &str,
) -> TestResult<ArmExecution> {
    let mut first = run_arm_once(oracle, tokenizer, documents, spec, name)?;
    let second = run_arm_once(oracle, tokenizer, documents, spec, name)?;
    let identical = first.report == second.report
        && first.report.logits_cid == second.report.logits_cid
        && first
            .scored_logits
            .iter()
            .flatten()
            .zip(second.scored_logits.iter().flatten())
            .all(|(left, right)| left.to_bits() == right.to_bits())
        && first.top1_tokens == second.top1_tokens
        && first.report.top1_token_ids == second.report.top1_token_ids
        && first.report.decoded_snippets == second.report.decoded_snippets
        && first.report.decoder_audits == second.report.decoder_audits
        && first.report.projection_audits == second.report.projection_audits
        && first.report.implementation_evidence == second.report.implementation_evidence;
    first.report.replay_identical = identical;
    Ok(first)
}

fn audit_is_exact(report: &ArmReport) -> bool {
    if report.name == "donor" {
        return report.decoder_audits.is_empty()
            && report.projection_audits.is_empty()
            && report.implementation_evidence.is_empty()
            && report.target_as_input_reads == 0;
    }
    let expected_layers = u64::try_from(INPUT_POSITIONS * EXPECTED_LAYERS).unwrap_or(u64::MAX);
    let expected_heads =
        u64::try_from(INPUT_POSITIONS * EXPECTED_LAYERS * EXPECTED_QUERY_HEADS).unwrap_or(u64::MAX);
    let expected_source_pairs = u64::try_from(
        EXPECTED_LAYERS * EXPECTED_QUERY_HEADS * (1..=INPUT_POSITIONS).sum::<usize>(),
    )
    .unwrap_or(u64::MAX);
    report.decoder_audits.len() == VALIDATION_DOCUMENTS
        && report.projection_audits.len() == VALIDATION_DOCUMENTS
        && report.decoder_audits.iter().all(|audit| {
            audit.positions == INPUT_POSITIONS as u64
                && audit.layers == expected_layers
                && audit.heads == expected_heads
                && audit.query_transforms == expected_heads
                && audit.key_transports == expected_source_pairs
                && audit.value_transports == expected_source_pairs
                && audit.output_transforms == expected_heads
                && audit.future_reads == 0
                && audit.maximum_query_position == Some(INPUT_POSITIONS - 1)
                && audit.maximum_source_position == Some(INPUT_POSITIONS - 1)
        })
        && report.projection_audits.iter().all(|audit| {
            audit.hook_calls == expected_layers
                && audit.query_vectors == expected_heads
                && audit.key_vectors
                    == u64::try_from(INPUT_POSITIONS * EXPECTED_LAYERS * EXPECTED_KV_HEADS)
                        .unwrap_or(u64::MAX)
                && audit.value_vectors
                    == u64::try_from(INPUT_POSITIONS * EXPECTED_LAYERS * EXPECTED_KV_HEADS)
                        .unwrap_or(u64::MAX)
                && audit.query_lanes == expected_heads * HEAD_WIDTH as u64
                && audit.key_lanes
                    == u64::try_from(
                        INPUT_POSITIONS * EXPECTED_LAYERS * EXPECTED_KV_HEADS * HEAD_WIDTH,
                    )
                    .unwrap_or(u64::MAX)
                && audit.value_lanes
                    == u64::try_from(
                        INPUT_POSITIONS * EXPECTED_LAYERS * EXPECTED_KV_HEADS * HEAD_WIDTH,
                    )
                    .unwrap_or(u64::MAX)
        })
        && report.implementation_evidence.len() == VALIDATION_DOCUMENTS
        && report.target_as_input_reads == 0
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct DecisionComparisons {
    donor_gauge_top1_matches: usize,
    gauge_minus_donor_nll: f64,
    donor_gauge_mean_absolute_logit_delta: f64,
    donor_gauge_maximum_relative_bound_excess: f64,
    functional_retention: bool,
    matched_learned_parity: bool,
    lorentz_euclidean_advantage: bool,
    source_frame_control_separates: bool,
    value_control_separates: bool,
    order_key_control_separates: bool,
    all_arm_replay: bool,
    causal_work_valid: bool,
}

fn compare_decision_arms(
    donor: &ArmExecution,
    gauge: &ArmExecution,
    lorentz: &ArmExecution,
    euclidean: &ArmExecution,
    source_frame: &ArmExecution,
    value: &ArmExecution,
    order_key: &ArmExecution,
) -> TestResult<DecisionComparisons> {
    let positions = VALIDATION_DOCUMENTS * SCORE_POSITIONS;
    let donor_gauge_top1_matches = donor
        .top1_tokens
        .iter()
        .zip(&gauge.top1_tokens)
        .filter(|(left, right)| left == right)
        .count();
    let mut absolute_sum = 0.0;
    let mut lanes = 0_u64;
    let mut maximum_excess = 0.0_f64;
    for (donor_row, gauge_row) in donor.scored_logits.iter().zip(&gauge.scored_logits) {
        if donor_row.len() != gauge_row.len() {
            return Err("donor/gauge logit shapes differ".into());
        }
        for (donor, gauge) in donor_row.iter().zip(gauge_row) {
            let delta = f64::from((*gauge - *donor).abs());
            absolute_sum += delta;
            lanes += 1;
            let bound = 0.02 + 0.001 * f64::from(donor.abs().max(gauge.abs()));
            maximum_excess = maximum_excess.max(delta - bound);
        }
    }
    let all_arm_replay = [
        donor,
        gauge,
        lorentz,
        euclidean,
        source_frame,
        value,
        order_key,
    ]
    .iter()
    .all(|arm| arm.report.replay_identical);
    let causal_work_valid = audit_is_exact(&donor.report)
        && audit_is_exact(&gauge.report)
        && audit_is_exact(&lorentz.report)
        && audit_is_exact(&euclidean.report)
        && audit_is_exact(&source_frame.report)
        && audit_is_exact(&value.report)
        && audit_is_exact(&order_key.report);
    Ok(DecisionComparisons {
        donor_gauge_top1_matches,
        gauge_minus_donor_nll: gauge.report.mean_next_token_nll - donor.report.mean_next_token_nll,
        donor_gauge_mean_absolute_logit_delta: absolute_sum / lanes as f64,
        donor_gauge_maximum_relative_bound_excess: maximum_excess,
        functional_retention: lorentz.report.mean_next_token_nll
            <= donor.report.mean_next_token_nll + 0.05,
        matched_learned_parity: lorentz.report.mean_next_token_nll
            <= euclidean.report.mean_next_token_nll + 0.05,
        lorentz_euclidean_advantage: lorentz.report.mean_next_token_nll
            <= euclidean.report.mean_next_token_nll - 0.01,
        source_frame_control_separates: source_frame.report.mean_next_token_nll
            >= lorentz.report.mean_next_token_nll + 0.02,
        value_control_separates: value.report.mean_next_token_nll
            >= lorentz.report.mean_next_token_nll + 0.02,
        order_key_control_separates: order_key.report.mean_next_token_nll
            >= lorentz.report.mean_next_token_nll + 0.02,
        all_arm_replay,
        causal_work_valid: causal_work_valid && donor_gauge_top1_matches <= positions && lanes > 0,
    })
}

fn terminal_for(comparisons: &DecisionComparisons) -> &'static str {
    let donor_gauge = comparisons.donor_gauge_top1_matches
        == VALIDATION_DOCUMENTS * SCORE_POSITIONS
        && comparisons.gauge_minus_donor_nll <= 0.002
        && comparisons.donor_gauge_mean_absolute_logit_delta <= 0.002
        && comparisons.donor_gauge_maximum_relative_bound_excess <= 0.0;
    let destructive = comparisons.source_frame_control_separates
        && comparisons.value_control_separates
        && comparisons.order_key_control_separates;
    let shared = donor_gauge
        && comparisons.functional_retention
        && comparisons.matched_learned_parity
        && destructive
        && comparisons.all_arm_replay
        && comparisons.causal_work_valid;
    if shared && comparisons.lorentz_euclidean_advantage {
        PASS_TERMINAL
    } else if shared {
        RETAIN_TERMINAL
    } else {
        FAIL_TERMINAL
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct TimingReport {
    fit_trace_seconds: f64,
    preflight_and_canary_seconds: f64,
    fit_and_replay_seconds: f64,
    validation_seconds: f64,
    total_seconds: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ResultPayload {
    schema: String,
    issue: u32,
    terminal: String,
    partition_cid: String,
    checkpoint_cid: String,
    implementation: ImplementationIdentity,
    preflight: GeometryPreflight,
    canary: CanaryReport,
    comparisons: DecisionComparisons,
    donor: ArmReport,
    gauge: ArmReport,
    learned_lorentz: ArmReport,
    learned_euclidean: ArmReport,
    source_frame_permuted: ArmReport,
    value_permuted: ArmReport,
    order_key_shuffled: ArmReport,
    execution_snapshot: TeacherExecutionSnapshot,
    timing: TimingReport,
    validation_materialized_after_checkpoint: bool,
    d3_status: String,
    result_cid: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct UnavailablePayload {
    schema: String,
    issue: u32,
    terminal: String,
    reason: String,
    validation_materialized: bool,
    checkpoint_exists: bool,
    d3_status: String,
    result_cid: String,
}

trait SelfCommittedResult: Clone + DeserializeOwned + PartialEq + Serialize {
    fn result_cid(&self) -> &str;
    fn result_cid_mut(&mut self) -> &mut String;
}

impl SelfCommittedResult for ResultPayload {
    fn result_cid(&self) -> &str {
        &self.result_cid
    }

    fn result_cid_mut(&mut self) -> &mut String {
        &mut self.result_cid
    }
}

impl SelfCommittedResult for UnavailablePayload {
    fn result_cid(&self) -> &str {
        &self.result_cid
    }

    fn result_cid_mut(&mut self) -> &mut String {
        &mut self.result_cid
    }
}

fn result_payload_cid<T: SelfCommittedResult>(payload: &T) -> TestResult<String> {
    let mut commitment = payload.clone();
    commitment.result_cid_mut().clear();
    canonical_json_cid(&commitment)
}

fn write_result<T: SelfCommittedResult>(path: &Path, payload: &T) -> TestResult<String> {
    let cid = result_payload_cid(payload)?;
    if payload.result_cid() != cid {
        return Err("result self-CID does not commit the empty result_cid convention".into());
    }
    write_pretty_json_exclusive(path, payload)?;
    let readback: T = serde_json::from_slice(&fs::read(path)?)?;
    if readback != *payload
        || readback.result_cid() != cid
        || result_payload_cid(&readback)? != cid
        || canonical_json_bytes(&readback)? != canonical_json_bytes(payload)?
    {
        return Err("exclusive result readback failed its self-CID convention".into());
    }
    Ok(cid)
}

fn write_unavailable(
    path: &Path,
    reason: &str,
    validation_materialized: bool,
    checkpoint_exists: bool,
) -> TestResult<String> {
    let mut payload = UnavailablePayload {
        schema: RESULT_SCHEMA.to_owned(),
        issue: 973,
        terminal: UNAVAILABLE_TERMINAL.to_owned(),
        reason: reason.to_owned(),
        validation_materialized,
        checkpoint_exists,
        d3_status: "NOT_RUN".to_owned(),
        result_cid: String::new(),
    };
    payload.result_cid = result_payload_cid(&payload)?;
    write_result(path, &payload)
}

#[test]
fn helm_d_learned_manifold_r4_construction_partition_policy_and_exclusions_are_sealed() -> TestResult
{
    let exclusions = predecessor_exclusions()?;
    if exclusions.document_ids.len() != 28 || exclusions.input_cids.len() != 28 {
        return Err("focused exclusion census is not the frozen 28-document predecessor".into());
    }
    let mut exclusion_commitment = exclusions.clone();
    exclusion_commitment.exclusion_cid.clear();
    if exclusions.exclusion_cid != canonical_json_cid(&exclusion_commitment)? {
        return Err("exclusion self-commitment drifted".into());
    }
    let predecessor: PredecessorEnvelope = serde_json::from_slice(COMPILED_PREDECESSOR_PARTITION)?;
    for document in predecessor
        .manifest
        .construction_fit
        .iter()
        .chain(&predecessor.manifest.construction_validation)
        .chain(&predecessor.manifest.predecessor_heldout)
    {
        if !exclusions.document_ids.contains(&document.id)
            || !exclusions.input_cids.contains(&document.input_cid)
        {
            return Err("a predecessor identity escaped the exclusion seal".into());
        }
    }
    let sample_tokens = (0..INPUT_POSITIONS as u32).collect::<Vec<_>>();
    let new_domain_input_cid =
        token_cid(b"uor-r4.helm-d-learned-manifold.inputs/2", &sample_tokens);
    let predecessor_domain_input_cid = token_cid(b"uor-r4.intrinsic.inputs/1", &sample_tokens);
    if new_domain_input_cid == predecessor_domain_input_cid {
        return Err("versioned input CID domains unexpectedly alias".into());
    }
    let commitment = FrozenDocumentCommitment {
        id: "focused-policy-fixture".to_owned(),
        selection_digest: format!(
            "blake3:{}",
            hex::encode(selection_digest("focused-policy-fixture"))
        ),
        title_cid: string_cid(b"uor-r4.helm-d-learned-manifold.title/2", "fixture"),
        input_cid: new_domain_input_cid,
        predecessor_domain_input_cid,
        corpus_byte_offset: 7,
        corpus_byte_length: 11,
    };
    let serialized = serde_json::to_value(commitment)?;
    let object = serialized
        .as_object()
        .ok_or("commitment did not serialize as an object")?;
    if object
        .keys()
        .any(|key| key.contains("target") || key.contains("token"))
        || object.len() != 7
    {
        return Err("partition commitment serialized a target/token identity".into());
    }
    let reserved_count = (0..100)
        .filter(|id| reserved_partition_id(&id.to_string()))
        .count();
    if reserved_count == 0 || reserved_count == 100 || PARITY_DOCUMENT_ID != "12" {
        return Err("reserved partition policy is not live".into());
    }
    Ok(())
}

#[test]
fn helm_d_learned_manifold_r4_construction_analytic_gradients_match_finite_difference() -> TestResult
{
    let maximum_error = finite_difference_gradient_preflight()?;
    if maximum_error > 3.0e-5 {
        return Err("analytic gradient exceeded the frozen finite-difference bound".into());
    }
    Ok(())
}

#[test]
fn helm_d_learned_manifold_r4_construction_golden_operator_is_pinned() -> TestResult {
    if pinned_golden_preflight()? > 1.0e-15 {
        return Err("HELM-D golden operator exceeded the frozen error bound".into());
    }
    Ok(())
}

#[test]
fn helm_d_learned_manifold_r4_construction_registered_frame_preflight_passes() -> TestResult {
    let (frames, score, weight, residual, centroid, permutation_live) =
        registered_frame_covariance_preflight(9)?;
    if frames != 120
        || score > 1.0e-8
        || weight > 1.0e-8
        || residual > 1.0e-8
        || centroid > 1.0e-8
        || !permutation_live
    {
        return Err("repaired registered-frame covariance/liveness preflight failed".into());
    }
    Ok(())
}

#[test]
fn helm_d_learned_manifold_r4_construction_public_identities_and_work_counts_are_pinned(
) -> TestResult {
    let parameters = HelmDLearnedManifoldParameters::identity(
        EXPECTED_LAYERS,
        EXPECTED_QUERY_HEADS,
        EXPECTED_KV_HEADS,
        BLOCKS_PER_HEAD,
        INITIAL_SCALE,
    )?;
    let parameter_cid = public_parameter_identity(&parameters)?;
    let transport = HelmDLearnedManifoldR4Transport::new(
        0,
        1,
        parameters.clone(),
        HelmDLearnedManifoldMetric::Lorentz,
        HelmDLearnedManifoldIntervention::Coherent,
    )?;
    if transport.parameter_identity()? != parameter_cid
        || !parameter_cid.starts_with("blake3:")
        || expected_transport_source_pairs() != 36_720
        || expected_nonidentity_source_frame_blocks() != 1_166_400
        || expected_nonidentity_row_permutations() != 36_450
    {
        return Err("public parameter identity or exact intervention work count drifted".into());
    }
    let mut unavailable = UnavailablePayload {
        schema: RESULT_SCHEMA.to_owned(),
        issue: 973,
        terminal: UNAVAILABLE_TERMINAL.to_owned(),
        reason: "focused self-CID fixture".to_owned(),
        validation_materialized: false,
        checkpoint_exists: false,
        d3_status: "NOT_RUN".to_owned(),
        result_cid: String::new(),
    };
    unavailable.result_cid = result_payload_cid(&unavailable)?;
    if unavailable.result_cid != result_payload_cid(&unavailable)? {
        return Err("result empty-field self-CID convention drifted".into());
    }
    Ok(())
}

#[test]
fn helm_d_learned_manifold_r4_construction_validation_replay_failure_is_a_failed_result() {
    let comparisons = DecisionComparisons {
        donor_gauge_top1_matches: VALIDATION_DOCUMENTS * SCORE_POSITIONS,
        gauge_minus_donor_nll: 0.0,
        donor_gauge_mean_absolute_logit_delta: 0.0,
        donor_gauge_maximum_relative_bound_excess: 0.0,
        functional_retention: true,
        matched_learned_parity: true,
        lorentz_euclidean_advantage: true,
        source_frame_control_separates: true,
        value_control_separates: true,
        order_key_control_separates: true,
        all_arm_replay: false,
        causal_work_valid: true,
    };
    assert_eq!(terminal_for(&comparisons), FAIL_TERMINAL);
}

#[test]
#[ignore = "fits and evaluates the sealed 16-fit/8-validation HELM-D construction"]
fn helm_d_learned_manifold_r4_construction_decision() -> TestResult {
    let result_path = required_path_from_env(RESULT_OUTPUT_ENV)?;
    let checkpoint_path = required_path_from_env(CHECKPOINT_ENV)?;
    let mut validation_materialized = false;
    match run_helm_d_learned_manifold_r4_construction(
        &result_path,
        &checkpoint_path,
        &mut validation_materialized,
    ) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = write_unavailable(
                &result_path,
                &error.to_string(),
                validation_materialized,
                checkpoint_path.is_file(),
            );
            Err(error)
        }
    }
}

fn run_helm_d_learned_manifold_r4_construction(
    result_path: &Path,
    checkpoint_path: &Path,
    validation_materialized: &mut bool,
) -> TestResult {
    let total_started = Instant::now();
    if env::var(CANONICAL_DETERMINISTIC_ENV).as_deref() != Ok("1") {
        return Err(format!("{CANONICAL_DETERMINISTIC_ENV}=1 is required").into());
    }
    if result_path.exists() || checkpoint_path.exists() {
        return Err("exclusive result or checkpoint path already exists".into());
    }
    let partition_path = required_path_from_env(PARTITION_ENV)?;
    let partition_envelope = parse_partition(&fs::read(&partition_path)?)?;
    let partition = &partition_envelope.manifest;
    let tokenizer_path = path_from_env(TOKENIZER_ENV, DEFAULT_TOKENIZER);
    let corpus_path = path_from_env(CORPUS_ENV, DEFAULT_CORPUS);
    let model_path = path_from_env(MODEL_ENV, DEFAULT_MODEL);
    // Before the checkpoint, only the adjacent manifest and the individually
    // committed fit ranges may be read. Whole-corpus hashing is admission-gated.
    verify_corpus_manifest(&corpus_path)?;
    if file_cid(&tokenizer_path)? != partition.tokenizer_cid {
        return Err("tokenizer identity differs from the frozen partition".into());
    }
    let tokenizer = Tokenizer::try_load(&tokenizer_path)?;
    let workers = NonZeroUsize::new(SHARDS).ok_or("eight workers must be nonzero")?;
    let oracle = HuggingFaceLlamaOracle::load_with_execution(
        &model_path,
        TeacherExecutionConfig::fixed_workers(workers),
    )?;
    if oracle.source_cid() != DONOR_CID || oracle.source_cid() != partition.donor_cid {
        return Err("donor identity differs from the frozen partition".into());
    }
    let config = oracle.cfg();
    if config.n_layers != EXPECTED_LAYERS
        || config.n_heads != EXPECTED_QUERY_HEADS
        || config.n_kv_heads != EXPECTED_KV_HEADS
        || config.dim / config.n_heads != HEAD_WIDTH
        || config.r4_attention
        || config.seq_len < INPUT_POSITIONS
    {
        return Err("donor model geometry differs from the frozen construction operator".into());
    }
    let rope_theta = config.rope_theta;
    let rope_interleaved = config.rope_interleaved;
    let maximum_token_id = u32::try_from(config.vocab.checked_sub(1).ok_or("empty vocab")?)?;
    // This registry-wide structural gate is independent of donor traces. Run
    // it before the expensive construction-fit capture and carry its result
    // forward so a static census defect cannot consume the fit-trace budget.
    let registered_frame_started = Instant::now();
    let registered_frame_preflight = registered_frame_covariance_preflight(maximum_token_id)?;
    let registered_frame_preflight_seconds = registered_frame_started.elapsed().as_secs_f64();
    let preparation = oracle.prepare_exact_execution(1)?;
    if preparation.workers_observed != SHARDS || !preparation.backend_exercised {
        return Err("exact eight-worker donor preparation was not exercised".into());
    }
    let implementation = implementation_identity()?;

    let fit_trace_started = Instant::now();
    let fit_documents =
        materialize_documents(&corpus_path, &tokenizer, &partition.construction_fit, false)?;
    if fit_documents
        .iter()
        .any(|document| document.tokens.len() != INPUT_POSITIONS)
    {
        return Err("fit materialization retained a future target".into());
    }
    let captures = capture_fit_documents(&oracle, &fit_documents)?;
    let fit_trace_seconds = fit_trace_started.elapsed().as_secs_f64();

    let preflight_started = Instant::now();
    let preflight = construction_preflight(
        &captures,
        rope_theta,
        rope_interleaved,
        registered_frame_preflight,
    )?;
    let canary = objective_canary(&captures, rope_theta, rope_interleaved)?;
    let preflight_and_canary_seconds =
        registered_frame_preflight_seconds + preflight_started.elapsed().as_secs_f64();

    let fit_started = Instant::now();
    let lorentz = fit_arm(
        HelmDLearnedManifoldMetric::Lorentz,
        &captures,
        FIT_DOCUMENTS,
        rope_theta,
        rope_interleaved,
        ADAM_STEPS,
    )?;
    let euclidean = fit_arm(
        HelmDLearnedManifoldMetric::Euclidean,
        &captures,
        FIT_DOCUMENTS,
        rope_theta,
        rope_interleaved,
        ADAM_STEPS,
    )?;
    let lorentz_replay = fit_arm(
        HelmDLearnedManifoldMetric::Lorentz,
        &captures,
        FIT_DOCUMENTS,
        rope_theta,
        rope_interleaved,
        ADAM_STEPS,
    )?;
    let euclidean_replay = fit_arm(
        HelmDLearnedManifoldMetric::Euclidean,
        &captures,
        FIT_DOCUMENTS,
        rope_theta,
        rope_interleaved,
        ADAM_STEPS,
    )?;
    let mut replay = FitReplayReport {
        lorentz_parameter_bytes_identical: lorentz.parameter_bytes
            == lorentz_replay.parameter_bytes,
        euclidean_parameter_bytes_identical: euclidean.parameter_bytes
            == euclidean_replay.parameter_bytes,
        lorentz_report_identical: lorentz.report == lorentz_replay.report,
        euclidean_report_identical: euclidean.report == euclidean_replay.report,
        replay_cid: String::new(),
    };
    replay.replay_cid = fit_replay_cid(&replay)?;
    if !replay.lorentz_parameter_bytes_identical
        || !replay.euclidean_parameter_bytes_identical
        || !replay.lorentz_report_identical
        || !replay.euclidean_report_identical
    {
        return Err("independent fit replay is not byte-identical".into());
    }
    let fit_and_replay_seconds = fit_started.elapsed().as_secs_f64();
    let fit_trace_cid = aggregate_trace_cid(&captures);
    let checkpoint = FitCheckpoint {
        schema: CHECKPOINT_SCHEMA.to_owned(),
        issue: 973,
        contract_cid: implementation.contract_cid.clone(),
        population_cid: CORPUS_CID.to_owned(),
        partition_cid: partition.partition_cid.clone(),
        manifest_cid: partition_envelope.manifest_cid.clone(),
        donor_cid: DONOR_CID.to_owned(),
        tokenizer_cid: partition.tokenizer_cid.clone(),
        upstream_source_commit: HELM_D_UPSTREAM_COMMIT.to_owned(),
        lorentz_policy: HELM_D_LEARNED_LORENTZ_R4_CONSTRUCTION_POLICY.to_owned(),
        euclidean_policy: HELM_D_LEARNED_EUCLIDEAN_R4_CONTROL_POLICY.to_owned(),
        optimizer_specification: format!(
            "128 full-batch f64 Adam steps; lr={LEARNING_RATE}; beta1={ADAM_BETA1}; beta2={ADAM_BETA2}; epsilon={ADAM_EPSILON}; ridge={RIDGE}; scale-floor={SCALE_FLOOR}; eight ordered shards"
        ),
        implementation: implementation.clone(),
        exact_workers: SHARDS,
        execution_preparation: preparation,
        fit_trace_cid,
        preflight: preflight.clone(),
        canary: canary.clone(),
        lorentz_parameters: lorentz.parameters.clone(),
        lorentz_parameter_cid: lorentz.report.parameter_cid.clone(),
        lorentz_fit_report: lorentz.report.clone(),
        euclidean_parameters: euclidean.parameters.clone(),
        euclidean_parameter_cid: euclidean.report.parameter_cid.clone(),
        euclidean_fit_report: euclidean.report.clone(),
        replay,
        validation_materialized: false,
        future_position_reads: captures
            .iter()
            .map(|capture| capture.decoder_audit.future_reads)
            .sum(),
        target_as_input_reads: 0,
    };
    let (checkpoint, validation_admission) =
        write_and_read_checkpoint(checkpoint_path, checkpoint)?;

    // This is the first operation permitted to materialize validation text,
    // tokens, or targets. The exclusive checkpoint has already been read back.
    let validation_started = Instant::now();
    // Mark the seal opened before even the admitted whole-corpus identity scan
    // so an unavailable result cannot claim validation stayed sealed after any
    // validation-range bytes may have been streamed.
    *validation_materialized = true;
    let validation_documents = materialize_validation_documents(
        &validation_admission,
        &partition_envelope,
        &corpus_path,
        &tokenizer,
    )?;
    let donor = run_arm(
        &oracle,
        &tokenizer,
        &validation_documents,
        ArmSpec::Donor,
        "donor",
    )?;
    let gauge = run_arm(
        &oracle,
        &tokenizer,
        &validation_documents,
        ArmSpec::Gauge,
        "gauge",
    )?;
    let learned_lorentz = run_arm(
        &oracle,
        &tokenizer,
        &validation_documents,
        ArmSpec::Learned {
            parameters: &checkpoint.checkpoint.lorentz_parameters,
            metric: HelmDLearnedManifoldMetric::Lorentz,
            intervention: HelmDLearnedManifoldIntervention::Coherent,
        },
        "learned_lorentz",
    )?;
    let learned_euclidean = run_arm(
        &oracle,
        &tokenizer,
        &validation_documents,
        ArmSpec::Learned {
            parameters: &checkpoint.checkpoint.euclidean_parameters,
            metric: HelmDLearnedManifoldMetric::Euclidean,
            intervention: HelmDLearnedManifoldIntervention::Coherent,
        },
        "learned_euclidean",
    )?;
    let source_frame_permuted = run_arm(
        &oracle,
        &tokenizer,
        &validation_documents,
        ArmSpec::Learned {
            parameters: &checkpoint.checkpoint.lorentz_parameters,
            metric: HelmDLearnedManifoldMetric::Lorentz,
            intervention: HelmDLearnedManifoldIntervention::SourceFramePermuted,
        },
        "source_frame_permuted",
    )?;
    let value_permuted = run_arm(
        &oracle,
        &tokenizer,
        &validation_documents,
        ArmSpec::Learned {
            parameters: &checkpoint.checkpoint.lorentz_parameters,
            metric: HelmDLearnedManifoldMetric::Lorentz,
            intervention: HelmDLearnedManifoldIntervention::ValuePermuted,
        },
        "value_permuted",
    )?;
    let order_key_shuffled = run_arm(
        &oracle,
        &tokenizer,
        &validation_documents,
        ArmSpec::Learned {
            parameters: &checkpoint.checkpoint.lorentz_parameters,
            metric: HelmDLearnedManifoldMetric::Lorentz,
            intervention: HelmDLearnedManifoldIntervention::OrderKeyShuffled,
        },
        "order_key_shuffled",
    )?;
    let comparisons = compare_decision_arms(
        &donor,
        &gauge,
        &learned_lorentz,
        &learned_euclidean,
        &source_frame_permuted,
        &value_permuted,
        &order_key_shuffled,
    )?;
    let terminal = terminal_for(&comparisons).to_owned();
    let validation_seconds = validation_started.elapsed().as_secs_f64();
    let mut payload = ResultPayload {
        schema: RESULT_SCHEMA.to_owned(),
        issue: 973,
        terminal,
        partition_cid: partition.partition_cid.clone(),
        checkpoint_cid: checkpoint.checkpoint_cid,
        implementation,
        preflight,
        canary,
        comparisons,
        donor: donor.report,
        gauge: gauge.report,
        learned_lorentz: learned_lorentz.report,
        learned_euclidean: learned_euclidean.report,
        source_frame_permuted: source_frame_permuted.report,
        value_permuted: value_permuted.report,
        order_key_shuffled: order_key_shuffled.report,
        execution_snapshot: oracle.execution_snapshot(),
        timing: TimingReport {
            fit_trace_seconds,
            preflight_and_canary_seconds,
            fit_and_replay_seconds,
            validation_seconds,
            total_seconds: total_started.elapsed().as_secs_f64(),
        },
        validation_materialized_after_checkpoint: true,
        d3_status: "NOT_RUN".to_owned(),
        result_cid: String::new(),
    };
    payload.result_cid = result_payload_cid(&payload)?;
    let result_cid = write_result(result_path, &payload)?;
    eprintln!(
        "HELM-D learned-manifold construction terminal={} result_cid={result_cid}",
        payload.terminal
    );
    Ok(())
}

// -------------------------------------------------------------------------
// Frozen score-by-readout localization (HelmDScoreCentroidLocalizationR4V1)
// -------------------------------------------------------------------------

const LOCALIZATION_TRAINABLE_PARAMETER_SCALARS: usize = 115_230;
const LOCALIZATION_FROZEN_PARAMETER_SCALARS: usize = 28_830;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalizationTargetCommitment {
    document_id: String,
    input_cid: String,
    article_span_bytes_cid: String,
    input_plus_target_cid: String,
    target_cid: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalizationTargetManifest {
    schema: String,
    issue: u32,
    partition_cid: String,
    corpus_cid: String,
    corpus_verification_status: String,
    tokenizer_cid: String,
    generator_implementation: LocalizationImplementationIdentity,
    document_ids: Vec<String>,
    commitments: Vec<LocalizationTargetCommitment>,
    manifest_cid: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalizationTargetEnvelope {
    artifact_cid: String,
    manifest: LocalizationTargetManifest,
}

fn localization_target_manifest_cid(manifest: &LocalizationTargetManifest) -> TestResult<String> {
    let mut commitment = manifest.clone();
    commitment.manifest_cid.clear();
    canonical_json_cid(&commitment)
}

fn committed_corpus_span_bytes_cid(
    corpus_path: &Path,
    commitment: &FrozenDocumentCommitment,
) -> TestResult<String> {
    let mut corpus = fs::File::open(corpus_path)?;
    let corpus_len = corpus.metadata()?.len();
    let end = commitment
        .corpus_byte_offset
        .checked_add(commitment.corpus_byte_length)
        .filter(|end| commitment.corpus_byte_length > 0 && *end <= corpus_len)
        .ok_or("committed target corpus span is invalid")?;
    let mut bytes = vec![0_u8; usize::try_from(commitment.corpus_byte_length)?];
    corpus.seek(SeekFrom::Start(commitment.corpus_byte_offset))?;
    corpus.read_exact(&mut bytes)?;
    if corpus.stream_position()? != end {
        return Err("committed target corpus span ended at the wrong offset".into());
    }
    Ok(cid_bytes(&bytes))
}

fn validate_localization_target_envelope(
    envelope: &LocalizationTargetEnvelope,
    partition: &FrozenPartition,
    implementation: &LocalizationImplementationIdentity,
) -> TestResult {
    let expected_ids = LOCALIZATION_AUDIT_IDS.map(str::to_owned).to_vec();
    if envelope.manifest.schema != "uor-r4.helm-d-score-centroid-localization-target-commitments/1"
        || envelope.manifest.issue != 973
        || envelope.manifest.partition_cid != LOCALIZATION_PARTITION_CID
        || envelope.manifest.partition_cid != partition.partition_cid
        || envelope.manifest.corpus_cid != CORPUS_CID
        || envelope.manifest.corpus_verification_status != LOCALIZATION_TARGET_CORPUS_STATUS
        || envelope.manifest.tokenizer_cid != partition.tokenizer_cid
        || &envelope.manifest.generator_implementation != implementation
        || envelope.manifest.document_ids != expected_ids
        || envelope.manifest.commitments.len() != LOCALIZATION_AUDIT_DOCUMENTS
        || envelope.manifest.manifest_cid != localization_target_manifest_cid(&envelope.manifest)?
        || envelope.artifact_cid != canonical_json_cid(&envelope.manifest)?
    {
        return Err("localization target commitment envelope header is invalid".into());
    }
    for (commitment, partition_document) in envelope
        .manifest
        .commitments
        .iter()
        .zip(&partition.construction_fit[LOCALIZATION_FIT_DOCUMENTS..])
    {
        if commitment.document_id != partition_document.id
            || commitment.input_cid != partition_document.input_cid
            || !commitment.article_span_bytes_cid.starts_with("blake3:")
            || !commitment.input_plus_target_cid.starts_with("blake3:")
            || !commitment.target_cid.starts_with("blake3:")
        {
            return Err("localization target commitment does not bind its audit input".into());
        }
    }
    Ok(())
}

fn validate_localization_materialized_targets(
    documents: &[FrozenDocument],
    envelope: &LocalizationTargetEnvelope,
    corpus_path: &Path,
    partition: &FrozenPartition,
) -> TestResult {
    if documents.len() != LOCALIZATION_AUDIT_DOCUMENTS
        || documents.len() != envelope.manifest.commitments.len()
        || documents.len() != partition.construction_fit[LOCALIZATION_FIT_DOCUMENTS..].len()
    {
        return Err("materialized localization target count differs from its commitment".into());
    }
    for ((document, commitment), partition_document) in documents
        .iter()
        .zip(&envelope.manifest.commitments)
        .zip(&partition.construction_fit[LOCALIZATION_FIT_DOCUMENTS..])
    {
        if document.id != commitment.document_id
            || document.tokens.len() != REQUIRED_TOKENS
            || committed_corpus_span_bytes_cid(corpus_path, partition_document)?
                != commitment.article_span_bytes_cid
            || token_cid(
                b"uor-r4.helm-d-score-centroid-localization.input-plus-target/1",
                &document.tokens,
            ) != commitment.input_plus_target_cid
            || token_cid(
                b"uor-r4.helm-d-score-centroid-localization.target/1",
                &document.tokens[INPUT_POSITIONS..REQUIRED_TOKENS],
            ) != commitment.target_cid
        {
            return Err(format!(
                "materialized decoder target for {} differs from the post-freeze commitment",
                document.id
            )
            .into());
        }
    }
    Ok(())
}

#[test]
#[ignore = "freezes locally held target commitments for the eight localization-audit documents"]
fn freeze_helm_d_score_centroid_localization_r4_v1_targets() -> TestResult {
    let output = required_path_from_env(LOCALIZATION_TARGET_OUTPUT_ENV)?;
    let partition_path = required_path_from_env(LOCALIZATION_PARTITION_ENV)?;
    let partition_envelope = parse_partition(&fs::read(partition_path)?)?;
    let partition = &partition_envelope.manifest;
    if partition.partition_cid != LOCALIZATION_PARTITION_CID {
        return Err("target freeze received the wrong construction partition".into());
    }
    let tokenizer_path = path_from_env(TOKENIZER_ENV, DEFAULT_TOKENIZER);
    let corpus_path = path_from_env(CORPUS_ENV, DEFAULT_CORPUS);
    verify_corpus_manifest(&corpus_path)?;
    if file_cid(&tokenizer_path)? != partition.tokenizer_cid {
        return Err("target freeze tokenizer differs from the partition".into());
    }
    let implementation = localization_implementation_identity()?;
    let tokenizer = Tokenizer::try_load(tokenizer_path)?;
    let documents = materialize_documents(
        &corpus_path,
        &tokenizer,
        &partition.construction_fit[LOCALIZATION_FIT_DOCUMENTS..],
        true,
    )?;
    let commitments = documents
        .iter()
        .zip(&partition.construction_fit[LOCALIZATION_FIT_DOCUMENTS..])
        .map(|(document, partition_document)| {
            Ok(LocalizationTargetCommitment {
                document_id: document.id.clone(),
                input_cid: partition_document.input_cid.clone(),
                article_span_bytes_cid: committed_corpus_span_bytes_cid(
                    &corpus_path,
                    partition_document,
                )?,
                input_plus_target_cid: token_cid(
                    b"uor-r4.helm-d-score-centroid-localization.input-plus-target/1",
                    &document.tokens,
                ),
                target_cid: token_cid(
                    b"uor-r4.helm-d-score-centroid-localization.target/1",
                    &document.tokens[INPUT_POSITIONS..REQUIRED_TOKENS],
                ),
            })
        })
        .collect::<TestResult<Vec<_>>>()?;
    let mut manifest = LocalizationTargetManifest {
        schema: "uor-r4.helm-d-score-centroid-localization-target-commitments/1".to_owned(),
        issue: 973,
        partition_cid: partition.partition_cid.clone(),
        corpus_cid: CORPUS_CID.to_owned(),
        corpus_verification_status: LOCALIZATION_TARGET_CORPUS_STATUS.to_owned(),
        tokenizer_cid: partition.tokenizer_cid.clone(),
        generator_implementation: implementation.clone(),
        document_ids: LOCALIZATION_AUDIT_IDS.map(str::to_owned).to_vec(),
        commitments,
        manifest_cid: String::new(),
    };
    manifest.manifest_cid = localization_target_manifest_cid(&manifest)?;
    let envelope = LocalizationTargetEnvelope {
        artifact_cid: canonical_json_cid(&manifest)?,
        manifest,
    };
    validate_localization_target_envelope(&envelope, partition, &implementation)?;
    write_pretty_json_exclusive(&output, &envelope)?;
    eprintln!(
        "HELM-D score-centroid target commitments artifact_cid={}",
        envelope.artifact_cid
    );
    Ok(())
}

fn localization_parameters_are_frozen(parameters: &HelmDLearnedManifoldParameters) -> bool {
    parameters
        .value_adapters()
        .iter()
        .all(|adapter| *adapter == R4AffineAdapter::identity())
        && parameters
            .learned_biases()
            .iter()
            .all(|bias| bias.to_bits() == 0)
}

#[allow(clippy::too_many_arguments)]
fn evaluate_localization_ce_row(
    parameters: &HelmDLearnedManifoldParameters,
    metric: HelmDLearnedManifoldMetric,
    document: &CapturedDocument,
    layer: usize,
    head: usize,
    position: usize,
    rope_theta: f32,
    rope_interleaved: bool,
    gradient: &mut ParameterGradient,
) -> TestResult<(f64, u64)> {
    if !localization_parameters_are_frozen(parameters) {
        return Err("localization value adapters or uniform biases escaped their freeze".into());
    }
    let projection = &document.projections[position][layer];
    let query_input = &projection.query[head * HEAD_WIDTH..(head + 1) * HEAD_WIDTH];
    let mut query = apply_adapters(
        parameters.query_adapters(),
        layer,
        head,
        EXPECTED_QUERY_HEADS,
        query_input,
    )?;
    apply_rope_f64(&mut query, position, rope_theta, rope_interleaved);

    let kv_head = head / (EXPECTED_QUERY_HEADS / EXPECTED_KV_HEADS);
    let mut keys = Vec::with_capacity(position + 1);
    let mut key_inputs = Vec::with_capacity(position + 1);
    for source in 0..=position {
        let source_projection = &document.projections[source][layer];
        let key_input =
            source_projection.key[kv_head * HEAD_WIDTH..(kv_head + 1) * HEAD_WIDTH].to_vec();
        let mut key = apply_adapters(
            parameters.key_adapters(),
            layer,
            kv_head,
            EXPECTED_KV_HEADS,
            &key_input,
        )?;
        apply_rope_f64(&mut key, source, rope_theta, rope_interleaved);
        key_inputs.push(key_input);
        keys.push(key);
    }

    let scale = parameters.learned_scale(layer)?;
    let mut numerators = Vec::with_capacity(position + 1);
    let mut query_numerator_gradients = Vec::with_capacity(position + 1);
    let mut key_numerator_gradients = Vec::with_capacity(position + 1);
    let mut logits = Vec::with_capacity(position + 1);
    for key in &keys {
        let (numerator, query_gradient, key_gradient) =
            score_numerator_and_gradients(metric, &query, key)?;
        let public_logit = helm_d_learned_manifold_logit(metric, &query, key, scale, 0.0)?;
        let logit = numerator / scale;
        if (public_logit - logit).abs() > 1.0e-12 {
            return Err("localization analytic score drifted from the public operator".into());
        }
        numerators.push(numerator);
        query_numerator_gradients.push(query_gradient);
        key_numerator_gradients.push(key_gradient);
        logits.push(logit);
    }
    let weights = stable_softmax_f64(&logits)?;
    let donor_weights = document.heads[position][layer][head]
        .donor_weights
        .iter()
        .map(|weight| f64::from(*weight))
        .collect::<Vec<_>>();
    let donor_weight_sum = donor_weights.iter().sum::<f64>();
    let cross_entropy = donor_weights
        .iter()
        .zip(&weights)
        .map(|(donor, learned)| -*donor * libm::log(*learned))
        .sum::<f64>();
    let mut query_post_gradient = vec![0.0; HEAD_WIDTH];
    for source in 0..=position {
        let logit_gradient = weights[source] * donor_weight_sum - donor_weights[source];
        for lane in 0..HEAD_WIDTH {
            query_post_gradient[lane] +=
                logit_gradient * query_numerator_gradients[source][lane] / scale;
        }
        let key_post_gradient = key_numerator_gradients[source]
            .iter()
            .map(|gradient| logit_gradient * gradient / scale)
            .collect::<Vec<_>>();
        let key_adapter_gradient =
            rope_gradient_to_input(&key_post_gradient, source, rope_theta, rope_interleaved);
        let key_base = adapter_index(layer, kv_head, 0, EXPECTED_KV_HEADS);
        for block in 0..BLOCKS_PER_HEAD {
            accumulate_adapter_gradient(
                &mut gradient.key[key_base + block],
                &key_inputs[source],
                &key_adapter_gradient,
                block,
            );
        }
        gradient.scale[layer] += logit_gradient * -numerators[source] / (scale * scale);
    }
    let query_adapter_gradient =
        rope_gradient_to_input(&query_post_gradient, position, rope_theta, rope_interleaved);
    let query_base = adapter_index(layer, head, 0, EXPECTED_QUERY_HEADS);
    for block in 0..BLOCKS_PER_HEAD {
        accumulate_adapter_gradient(
            &mut gradient.query[query_base + block],
            query_input,
            &query_adapter_gradient,
            block,
        );
    }
    if !cross_entropy.is_finite() || !gradient.all_finite() {
        return Err("localization CE objective or gradient is non-finite".into());
    }
    Ok((cross_entropy, u64::try_from(position + 1)?))
}

#[allow(clippy::too_many_arguments)]
fn evaluate_localization_ce_shard(
    parameters: &HelmDLearnedManifoldParameters,
    metric: HelmDLearnedManifoldMetric,
    documents: &[CapturedDocument],
    document_limit: usize,
    shard: usize,
    rope_theta: f32,
    rope_interleaved: bool,
) -> TestResult<ShardEvaluation> {
    let mut evaluation = ShardEvaluation {
        loss: 0.0,
        rows: 0,
        source_pairs: 0,
        gradient: ParameterGradient::zero(parameters),
    };
    for layer in 0..EXPECTED_LAYERS {
        for head in 0..EXPECTED_QUERY_HEADS {
            for position in SCORE_START..INPUT_POSITIONS {
                for (document_index, document) in documents[..document_limit].iter().enumerate() {
                    let ordinal = (((document_index * EXPECTED_LAYERS + layer)
                        * EXPECTED_QUERY_HEADS
                        + head)
                        * SCORE_POSITIONS)
                        + (position - SCORE_START);
                    if ordinal % SHARDS != shard {
                        continue;
                    }
                    let (loss, source_pairs) = evaluate_localization_ce_row(
                        parameters,
                        metric,
                        document,
                        layer,
                        head,
                        position,
                        rope_theta,
                        rope_interleaved,
                        &mut evaluation.gradient,
                    )?;
                    evaluation.loss += loss;
                    evaluation.rows += 1;
                    evaluation.source_pairs += source_pairs;
                }
            }
        }
    }
    Ok(evaluation)
}

fn localization_ridge_objective_and_gradient(
    parameters: &HelmDLearnedManifoldParameters,
    gradient: &mut ParameterGradient,
) -> f64 {
    let mut objective = 0.0;
    for (adapters, gradients) in [
        (parameters.query_adapters(), &mut gradient.query),
        (parameters.key_adapters(), &mut gradient.key),
    ] {
        for (adapter, target) in adapters.iter().zip(gradients.iter_mut()) {
            for row in 0..R4_WIDTH {
                for column in 0..R4_WIDTH {
                    let expected = if row == column { 1.0 } else { 0.0 };
                    let difference = adapter.matrix[row][column] - expected;
                    objective += RIDGE * difference * difference;
                    target.matrix[row][column] += 2.0 * RIDGE * difference;
                }
                objective += RIDGE * adapter.bias[row] * adapter.bias[row];
                target.bias[row] += 2.0 * RIDGE * adapter.bias[row];
            }
        }
    }
    objective
}

fn localization_frozen_gradient_is_zero(gradient: &ParameterGradient) -> bool {
    gradient.value.iter().all(|adapter| {
        adapter
            .matrix
            .iter()
            .flatten()
            .chain(&adapter.bias)
            .all(|value| value.to_bits() == 0)
    }) && gradient.bias.iter().all(|value| value.to_bits() == 0)
}

fn evaluate_localization_ce_dataset(
    parameters: &HelmDLearnedManifoldParameters,
    metric: HelmDLearnedManifoldMetric,
    documents: &[CapturedDocument],
    document_limit: usize,
    rope_theta: f32,
    rope_interleaved: bool,
) -> TestResult<DatasetEvaluation> {
    if document_limit == 0 || document_limit > documents.len() {
        return Err("localization dataset document limit is invalid".into());
    }
    let shards = std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(SHARDS);
        for shard in 0..SHARDS {
            handles.push(scope.spawn(move || {
                evaluate_localization_ce_shard(
                    parameters,
                    metric,
                    documents,
                    document_limit,
                    shard,
                    rope_theta,
                    rope_interleaved,
                )
            }));
        }
        handles
            .into_iter()
            .map(|handle| handle.join().map_err(|_| "localization shard panicked")?)
            .collect::<TestResult<Vec<_>>>()
    })?;
    let mut loss = 0.0;
    let mut rows = 0_u64;
    let mut gradient = ParameterGradient::zero(parameters);
    let mut rows_per_shard = Vec::with_capacity(SHARDS);
    let mut source_pairs_per_shard = Vec::with_capacity(SHARDS);
    for shard in &shards {
        loss += shard.loss;
        rows += shard.rows;
        gradient.add_assign(&shard.gradient);
        rows_per_shard.push(shard.rows);
        source_pairs_per_shard.push(shard.source_pairs);
    }
    let expected_rows =
        u64::try_from(document_limit * SCORE_POSITIONS * EXPECTED_LAYERS * EXPECTED_QUERY_HEADS)?;
    if rows != expected_rows || rows_per_shard.contains(&0) {
        return Err("localization eight-shard work ledger is incomplete".into());
    }
    let reciprocal_rows = 1.0 / rows as f64;
    loss *= reciprocal_rows;
    gradient.scale_by(reciprocal_rows);
    loss += localization_ridge_objective_and_gradient(parameters, &mut gradient);
    if !loss.is_finite()
        || !gradient.all_finite()
        || !localization_frozen_gradient_is_zero(&gradient)
    {
        return Err("localization reduced CE gradient changed a frozen parameter".into());
    }
    Ok(DatasetEvaluation {
        objective: loss,
        gradient,
        rows_per_shard,
        source_pairs_per_shard,
    })
}

fn localization_adam_step(
    parameters: &mut HelmDLearnedManifoldParameters,
    gradient: &ParameterGradient,
    first: &mut ParameterGradient,
    second: &mut ParameterGradient,
    step: usize,
) -> TestResult {
    fn update_adapters(
        adapters: &mut [R4AffineAdapter],
        gradients: &[AffineGradient],
        first: &mut [AffineGradient],
        second: &mut [AffineGradient],
        step: usize,
    ) {
        for (((adapter, gradient), first), second) in
            adapters.iter_mut().zip(gradients).zip(first).zip(second)
        {
            for row in 0..R4_WIDTH {
                for column in 0..R4_WIDTH {
                    update_scalar(
                        &mut adapter.matrix[row][column],
                        gradient.matrix[row][column],
                        &mut first.matrix[row][column],
                        &mut second.matrix[row][column],
                        step,
                    );
                }
                update_scalar(
                    &mut adapter.bias[row],
                    gradient.bias[row],
                    &mut first.bias[row],
                    &mut second.bias[row],
                    step,
                );
            }
        }
    }
    if !localization_frozen_gradient_is_zero(gradient) {
        return Err("localization optimizer received a nonzero frozen gradient".into());
    }
    update_adapters(
        parameters.query_adapters_mut(),
        &gradient.query,
        &mut first.query,
        &mut second.query,
        step,
    );
    update_adapters(
        parameters.key_adapters_mut(),
        &gradient.key,
        &mut first.key,
        &mut second.key,
        step,
    );
    for layer in 0..parameters.layers() {
        update_scalar(
            &mut parameters.learned_scales_mut()[layer],
            gradient.scale[layer],
            &mut first.scale[layer],
            &mut second.scale[layer],
            step,
        );
        parameters.learned_scales_mut()[layer] =
            parameters.learned_scales()[layer].max(SCALE_FLOOR);
    }
    if !localization_parameters_are_frozen(parameters) {
        return Err("localization optimizer mutated value adapters or uniform biases".into());
    }
    parameters.validate()?;
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LocalizationFitReport {
    schema: String,
    metric: HelmDLearnedManifoldMetric,
    objective: String,
    optimizer: String,
    steps: usize,
    workers: usize,
    fit_documents: usize,
    fit_rows: u64,
    initial_objective: f64,
    final_objective: f64,
    total_parameter_scalars: usize,
    trainable_parameter_scalars: usize,
    frozen_parameter_scalars: usize,
    value_adapters_identity: bool,
    uniform_bias_zero: bool,
    parameter_cid: String,
    donor_trace_cid: String,
    gradient_audit: GradientAudit,
    gradient_audit_cid: String,
    work: WorkLedger,
    report_cid: String,
}

fn localization_fit_report_cid(report: &LocalizationFitReport) -> TestResult<String> {
    let mut commitment = report.clone();
    commitment.report_cid.clear();
    canonical_json_cid(&commitment)
}

#[derive(Clone, Debug)]
struct LocalizationFittedScore {
    parameters: HelmDLearnedManifoldParameters,
    parameter_bytes: Vec<u8>,
    report: LocalizationFitReport,
}

fn fit_localization_score(
    metric: HelmDLearnedManifoldMetric,
    documents: &[CapturedDocument],
    document_limit: usize,
    rope_theta: f32,
    rope_interleaved: bool,
) -> TestResult<LocalizationFittedScore> {
    let mut parameters = HelmDLearnedManifoldParameters::identity(
        EXPECTED_LAYERS,
        EXPECTED_QUERY_HEADS,
        EXPECTED_KV_HEADS,
        BLOCKS_PER_HEAD,
        INITIAL_SCALE,
    )?;
    if parameters.scalar_parameter_count()? != PARAMETER_SCALARS
        || PARAMETER_SCALARS
            != LOCALIZATION_TRAINABLE_PARAMETER_SCALARS + LOCALIZATION_FROZEN_PARAMETER_SCALARS
        || !localization_parameters_are_frozen(&parameters)
    {
        return Err("localization parameter capacity or freeze differs from contract".into());
    }
    let initial = evaluate_localization_ce_dataset(
        &parameters,
        metric,
        documents,
        document_limit,
        rope_theta,
        rope_interleaved,
    )?;
    let mut first = ParameterGradient::zero(&parameters);
    let mut second = ParameterGradient::zero(&parameters);
    let rows_per_shard = initial.rows_per_shard.clone();
    let source_pairs_per_shard = initial.source_pairs_per_shard.clone();
    let (initial_gradient_cid, mut maximum_absolute_gradient) =
        evaluation_gradient_identity(&initial, "localization-initial-diagnostic", 0);
    let mut ordered_evaluation_cids = vec![initial_gradient_cid];
    for step in 1..=LOCALIZATION_ADAM_STEPS {
        let evaluation = evaluate_localization_ce_dataset(
            &parameters,
            metric,
            documents,
            document_limit,
            rope_theta,
            rope_interleaved,
        )?;
        if evaluation.rows_per_shard != rows_per_shard
            || evaluation.source_pairs_per_shard != source_pairs_per_shard
        {
            return Err("localization deterministic shard schedule changed during fitting".into());
        }
        let (gradient_cid, maximum) =
            evaluation_gradient_identity(&evaluation, "localization-optimizer-step", step);
        ordered_evaluation_cids.push(gradient_cid);
        maximum_absolute_gradient = maximum_absolute_gradient.max(maximum);
        localization_adam_step(
            &mut parameters,
            &evaluation.gradient,
            &mut first,
            &mut second,
            step,
        )?;
    }
    let final_evaluation = evaluate_localization_ce_dataset(
        &parameters,
        metric,
        documents,
        document_limit,
        rope_theta,
        rope_interleaved,
    )?;
    if final_evaluation.rows_per_shard != rows_per_shard
        || final_evaluation.source_pairs_per_shard != source_pairs_per_shard
    {
        return Err("localization deterministic shard schedule changed at final evaluation".into());
    }
    let (final_gradient_cid, final_maximum) = evaluation_gradient_identity(
        &final_evaluation,
        "localization-final-diagnostic",
        LOCALIZATION_ADAM_STEPS + 1,
    );
    ordered_evaluation_cids.push(final_gradient_cid);
    maximum_absolute_gradient = maximum_absolute_gradient.max(final_maximum);
    let parameter_bytes = canonical_json_bytes(&parameters)?;
    let parameter_cid = public_parameter_identity(&parameters)?;
    let rows_per_evaluation = rows_per_shard.iter().sum::<u64>();
    let source_pairs_per_evaluation = source_pairs_per_shard.iter().sum::<u64>();
    let evaluations = LOCALIZATION_ADAM_STEPS + 2;
    let gradient_audit = GradientAudit {
        schema: "uor-r4.helm-d-score-centroid-localization-gradient-audit/1".to_owned(),
        gradient_evaluations: evaluations,
        optimizer_gradient_evaluations: LOCALIZATION_ADAM_STEPS,
        diagnostic_gradient_evaluations: 2,
        maximum_absolute_gradient,
        ordered_evaluation_cids,
    };
    let gradient_audit_cid = canonical_json_cid(&gradient_audit)?;
    let mut report = LocalizationFitReport {
        schema: "uor-r4.helm-d-score-centroid-localization-fit/1".to_owned(),
        metric,
        objective: "donor-attention-cross-entropy-plus-qk-only-ridge".to_owned(),
        optimizer: format!(
            "full-batch-f64-adam(lr={LEARNING_RATE},beta1={ADAM_BETA1},beta2={ADAM_BETA2},epsilon={ADAM_EPSILON},ridge={RIDGE},scale_floor={SCALE_FLOOR})"
        ),
        steps: LOCALIZATION_ADAM_STEPS,
        workers: SHARDS,
        fit_documents: document_limit,
        fit_rows: rows_per_evaluation,
        initial_objective: initial.objective,
        final_objective: final_evaluation.objective,
        total_parameter_scalars: PARAMETER_SCALARS,
        trainable_parameter_scalars: LOCALIZATION_TRAINABLE_PARAMETER_SCALARS,
        frozen_parameter_scalars: LOCALIZATION_FROZEN_PARAMETER_SCALARS,
        value_adapters_identity: localization_parameters_are_frozen(&parameters),
        uniform_bias_zero: parameters
            .learned_biases()
            .iter()
            .all(|bias| bias.to_bits() == 0),
        parameter_cid,
        donor_trace_cid: aggregate_trace_cid(&documents[..document_limit]),
        gradient_audit,
        gradient_audit_cid,
        work: WorkLedger {
            workers: SHARDS,
            full_batch_steps: LOCALIZATION_ADAM_STEPS,
            full_batch_evaluations: evaluations,
            rows_per_shard_per_evaluation: rows_per_shard,
            source_pairs_per_shard_per_evaluation: source_pairs_per_shard,
            total_row_evaluations: rows_per_evaluation.saturating_mul(evaluations as u64),
            total_source_pair_evaluations: source_pairs_per_evaluation
                .saturating_mul(evaluations as u64),
        },
        report_cid: String::new(),
    };
    report.report_cid = localization_fit_report_cid(&report)?;
    Ok(LocalizationFittedScore {
        parameters,
        parameter_bytes,
        report,
    })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LocalizationQuantiles {
    p0: f64,
    p50: f64,
    p95: f64,
    p100: f64,
}

fn localization_quantiles(mut values: Vec<f64>) -> TestResult<LocalizationQuantiles> {
    if values.is_empty() || values.iter().any(|value| !value.is_finite()) {
        return Err("localization quantiles require finite nonempty values".into());
    }
    values.sort_by(f64::total_cmp);
    let nearest_rank = |percent: usize| -> usize {
        percent
            .saturating_mul(values.len())
            .div_ceil(100)
            .saturating_sub(1)
    };
    Ok(LocalizationQuantiles {
        p0: values[0],
        p50: values[nearest_rank(50)],
        p95: values[nearest_rank(95)],
        p100: values[values.len() - 1],
    })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LocalizationAttentionArmReport {
    name: String,
    metric: HelmDLearnedManifoldMetric,
    value_readout: HelmDLearnedManifoldValueReadout,
    rows: u64,
    source_pairs: u64,
    donor_attention_cross_entropy: f64,
    normalized_aggregate_mse: f64,
    mean_attention_entropy: f64,
    query_radial_norm_quantiles: LocalizationQuantiles,
    key_radial_norm_quantiles: LocalizationQuantiles,
    lorentz_denominator_quantiles: Option<LocalizationQuantiles>,
    lorentz_reciprocal_multiplier_quantiles: Option<LocalizationQuantiles>,
    logits_cid: String,
    weights_cid: String,
}

struct LocalizationArmAccumulator {
    name: &'static str,
    metric: HelmDLearnedManifoldMetric,
    value_readout: HelmDLearnedManifoldValueReadout,
    rows: u64,
    source_pairs: u64,
    cross_entropy_sum: f64,
    aggregate_mse_sum: f64,
    entropy_sum: f64,
    query_radial_norms: Vec<f64>,
    key_radial_norms: Vec<f64>,
    normalization_factors: Vec<f64>,
    reciprocal_multipliers: Vec<f64>,
    logits_hasher: blake3::Hasher,
    weights_hasher: blake3::Hasher,
}

impl LocalizationArmAccumulator {
    fn new(
        name: &'static str,
        metric: HelmDLearnedManifoldMetric,
        value_readout: HelmDLearnedManifoldValueReadout,
    ) -> Self {
        let mut logits_hasher = blake3::Hasher::new();
        logits_hasher.update(b"uor-r4.helm-d-score-centroid-localization.logits/1\0");
        let mut weights_hasher = blake3::Hasher::new();
        weights_hasher.update(b"uor-r4.helm-d-score-centroid-localization.weights/1\0");
        Self {
            name,
            metric,
            value_readout,
            rows: 0,
            source_pairs: 0,
            cross_entropy_sum: 0.0,
            aggregate_mse_sum: 0.0,
            entropy_sum: 0.0,
            query_radial_norms: Vec::new(),
            key_radial_norms: Vec::new(),
            normalization_factors: Vec::new(),
            reciprocal_multipliers: Vec::new(),
            logits_hasher,
            weights_hasher,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn push(
        &mut self,
        logits: &[f64],
        weights: &[f64],
        cross_entropy: f64,
        aggregate_mse: f64,
        query_norm: f64,
        key_norms: &[f64],
        normalization_factor: Option<f64>,
    ) -> TestResult {
        if !cross_entropy.is_finite()
            || !aggregate_mse.is_finite()
            || !query_norm.is_finite()
            || key_norms.iter().any(|value| !value.is_finite())
            || normalization_factor.is_some_and(|value| !value.is_finite())
        {
            return Err("localization attention metric is non-finite".into());
        }
        self.rows = self.rows.saturating_add(1);
        self.source_pairs = self
            .source_pairs
            .saturating_add(u64::try_from(weights.len())?);
        self.cross_entropy_sum += cross_entropy;
        self.aggregate_mse_sum += aggregate_mse;
        self.entropy_sum += weights
            .iter()
            .filter(|weight| **weight > 0.0)
            .map(|weight| -*weight * libm::log(*weight))
            .sum::<f64>();
        self.query_radial_norms.push(query_norm);
        self.key_radial_norms.extend_from_slice(key_norms);
        if let Some(factor) = normalization_factor {
            self.normalization_factors.push(factor);
            self.reciprocal_multipliers.push(factor.recip());
        }
        self.logits_hasher
            .update(&u64::try_from(logits.len())?.to_le_bytes());
        self.weights_hasher
            .update(&u64::try_from(weights.len())?.to_le_bytes());
        for value in logits {
            self.logits_hasher.update(&value.to_bits().to_le_bytes());
        }
        for value in weights {
            self.weights_hasher.update(&value.to_bits().to_le_bytes());
        }
        Ok(())
    }

    fn finish(self) -> TestResult<LocalizationAttentionArmReport> {
        if self.rows == 0
            || self.query_radial_norms.len() != usize::try_from(self.rows)?
            || (self.value_readout == HelmDLearnedManifoldValueReadout::NormalizedLorentzCentroid
                && self.normalization_factors.len() != usize::try_from(self.rows)?)
            || (self.value_readout
                == HelmDLearnedManifoldValueReadout::TransportedTangentArithmeticSum
                && !self.normalization_factors.is_empty())
            || self.normalization_factors.len() != self.reciprocal_multipliers.len()
        {
            return Err("localization attention accumulator is incomplete".into());
        }
        let reciprocal_rows = 1.0 / self.rows as f64;
        let (normalization, reciprocal) = if self.normalization_factors.is_empty() {
            (None, None)
        } else {
            (
                Some(localization_quantiles(self.normalization_factors)?),
                Some(localization_quantiles(self.reciprocal_multipliers)?),
            )
        };
        Ok(LocalizationAttentionArmReport {
            name: self.name.to_owned(),
            metric: self.metric,
            value_readout: self.value_readout,
            rows: self.rows,
            source_pairs: self.source_pairs,
            donor_attention_cross_entropy: self.cross_entropy_sum * reciprocal_rows,
            normalized_aggregate_mse: self.aggregate_mse_sum * reciprocal_rows,
            mean_attention_entropy: self.entropy_sum * reciprocal_rows,
            query_radial_norm_quantiles: localization_quantiles(self.query_radial_norms)?,
            key_radial_norm_quantiles: localization_quantiles(self.key_radial_norms)?,
            lorentz_denominator_quantiles: normalization,
            lorentz_reciprocal_multiplier_quantiles: reciprocal,
            logits_cid: format!("blake3:{}", self.logits_hasher.finalize().to_hex()),
            weights_cid: format!("blake3:{}", self.weights_hasher.finalize().to_hex()),
        })
    }
}

struct LocalizationRow {
    query: Vec<f64>,
    keys: Vec<Vec<f64>>,
    values: Vec<Vec<f64>>,
    logits: Vec<f64>,
    weights: Vec<f64>,
    cross_entropy: f64,
}

#[allow(clippy::too_many_arguments)]
fn localization_attention_row(
    parameters: &HelmDLearnedManifoldParameters,
    metric: HelmDLearnedManifoldMetric,
    document: &CapturedDocument,
    layer: usize,
    head: usize,
    position: usize,
    rope_theta: f32,
    rope_interleaved: bool,
) -> TestResult<LocalizationRow> {
    if !localization_parameters_are_frozen(parameters) {
        return Err("localization attention evaluation observed an unfrozen value or bias".into());
    }
    let projection = &document.projections[position][layer];
    let query_input = &projection.query[head * HEAD_WIDTH..(head + 1) * HEAD_WIDTH];
    let mut query = apply_adapters(
        parameters.query_adapters(),
        layer,
        head,
        EXPECTED_QUERY_HEADS,
        query_input,
    )?;
    apply_rope_f64(&mut query, position, rope_theta, rope_interleaved);
    let kv_head = head / (EXPECTED_QUERY_HEADS / EXPECTED_KV_HEADS);
    let mut keys = Vec::with_capacity(position + 1);
    let mut values = Vec::with_capacity(position + 1);
    for source in 0..=position {
        let source_projection = &document.projections[source][layer];
        let key_input = &source_projection.key[kv_head * HEAD_WIDTH..(kv_head + 1) * HEAD_WIDTH];
        let value_input =
            &source_projection.value[kv_head * HEAD_WIDTH..(kv_head + 1) * HEAD_WIDTH];
        let mut key = apply_adapters(
            parameters.key_adapters(),
            layer,
            kv_head,
            EXPECTED_KV_HEADS,
            key_input,
        )?;
        apply_rope_f64(&mut key, source, rope_theta, rope_interleaved);
        let value = apply_adapters(
            parameters.value_adapters(),
            layer,
            kv_head,
            EXPECTED_KV_HEADS,
            value_input,
        )?;
        keys.push(key);
        values.push(value);
    }
    let scale = parameters.learned_scale(layer)?;
    let logits = keys
        .iter()
        .map(|key| helm_d_learned_manifold_logit(metric, &query, key, scale, 0.0))
        .collect::<Result<Vec<_>, _>>()?;
    let weights = stable_softmax_f64(&logits)?;
    let donor_weights = &document.heads[position][layer][head].donor_weights;
    let cross_entropy = donor_weights
        .iter()
        .zip(&weights)
        .map(|(donor, learned)| -f64::from(*donor) * libm::log(*learned))
        .sum::<f64>();
    Ok(LocalizationRow {
        query,
        keys,
        values,
        logits,
        weights,
        cross_entropy,
    })
}

fn localization_normalization_factor(values: &[Vec<f64>], weights: &[f64]) -> TestResult<f64> {
    fn compensated_add(sum: &mut f64, correction: &mut f64, value: f64) {
        let next = *sum + value;
        if sum.abs() >= value.abs() {
            *correction += (*sum - next) + value;
        } else {
            *correction += (value - next) + *sum;
        }
        *sum = next;
    }
    fn compensated_sum(values: impl IntoIterator<Item = f64>) -> f64 {
        let mut sum = 0.0;
        let mut correction = 0.0;
        for value in values {
            compensated_add(&mut sum, &mut correction, value);
        }
        sum + correction
    }
    let weight_sum = compensated_sum(weights.iter().copied());
    if !weight_sum.is_finite() || weight_sum <= 0.0 {
        return Err("localization normalization received invalid weights".into());
    }
    let mut time_sum = 0.0;
    let mut time_correction = 0.0;
    let mut spatial_sum = vec![0.0; HEAD_WIDTH];
    let mut spatial_correction = vec![0.0; HEAD_WIDTH];
    for (weight, value) in weights.iter().zip(values) {
        let normalized_weight = *weight / weight_sum;
        let time = libm::sqrt(1.0 + compensated_sum(value.iter().map(|lane| lane * lane)));
        compensated_add(
            &mut time_sum,
            &mut time_correction,
            normalized_weight * time,
        );
        for ((target, correction), coordinate) in spatial_sum
            .iter_mut()
            .zip(&mut spatial_correction)
            .zip(value)
        {
            compensated_add(target, correction, normalized_weight * *coordinate);
        }
    }
    time_sum += time_correction;
    for (target, correction) in spatial_sum.iter_mut().zip(spatial_correction) {
        *target += correction;
    }
    let spatial_norm = libm::sqrt(compensated_sum(spatial_sum.iter().map(|lane| lane * lane)));
    let factor = libm::sqrt((time_sum - spatial_norm) * (time_sum + spatial_norm));
    if !factor.is_finite() || factor < 1.0 - 1.0e-12 {
        return Err(
            format!("Lorentz normalization factor {factor} is below the frozen bound").into(),
        );
    }
    Ok(factor)
}

fn localization_normalized_aggregate_mse(learned: &[f64], donor: &[f32]) -> TestResult<f64> {
    if learned.len() != HEAD_WIDTH || donor.len() != HEAD_WIDTH {
        return Err("localization aggregate MSE shape mismatch".into());
    }
    let donor_norm = libm::sqrt(
        donor
            .iter()
            .map(|value| f64::from(*value) * f64::from(*value))
            .sum::<f64>(),
    );
    let normalization = donor_norm.max(1.0);
    Ok(learned
        .iter()
        .zip(donor)
        .map(|(left, right)| {
            let difference = *left - f64::from(*right);
            difference * difference / (normalization * normalization)
        })
        .sum::<f64>()
        / HEAD_WIDTH as f64)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LocalizationAttentionDocumentReport {
    document_id: String,
    lorentz_normalized: LocalizationAttentionArmReport,
    lorentz_tangent: LocalizationAttentionArmReport,
    euclidean_normalized: LocalizationAttentionArmReport,
    euclidean_tangent: LocalizationAttentionArmReport,
    lorentz_score_pair_bit_identical: bool,
    euclidean_score_pair_bit_identical: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct LocalizationAttentionWork {
    documents: usize,
    rows_per_document: u64,
    source_pairs_per_document: u64,
    total_rows_per_arm: u64,
    total_source_pairs_per_arm: u64,
    future_position_reads: u64,
    target_as_input_reads: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LocalizationAttentionEvaluation {
    schema: String,
    offline_evaluation_frame: String,
    offline_atlas_transport_status: String,
    documents: Vec<LocalizationAttentionDocumentReport>,
    pooled: Vec<LocalizationAttentionArmReport>,
    score_paired_logits_and_weights_bit_identical: bool,
    all_normalization_factors_at_least_one_minus_tolerance: bool,
    work: LocalizationAttentionWork,
    replay_identical: bool,
    evaluation_cid: String,
}

fn localization_attention_evaluation_cid(
    evaluation: &LocalizationAttentionEvaluation,
) -> TestResult<String> {
    let mut commitment = evaluation.clone();
    commitment.evaluation_cid.clear();
    canonical_json_cid(&commitment)
}

fn push_localization_row(
    accumulator: &mut LocalizationArmAccumulator,
    row: &LocalizationRow,
    readout: HelmDLearnedManifoldValueReadout,
    donor_aggregate: &[f32],
) -> TestResult {
    let aggregate = helm_d_learned_manifold_value_readout(readout, &row.values, &row.weights)?;
    let aggregate_mse = localization_normalized_aggregate_mse(&aggregate, donor_aggregate)?;
    let query_norm = libm::sqrt(row.query.iter().map(|lane| lane * lane).sum::<f64>());
    let key_norms = row
        .keys
        .iter()
        .map(|key| libm::sqrt(key.iter().map(|lane| lane * lane).sum::<f64>()))
        .collect::<Vec<_>>();
    let normalization_factor = (readout
        == HelmDLearnedManifoldValueReadout::NormalizedLorentzCentroid)
        .then(|| localization_normalization_factor(&row.values, &row.weights))
        .transpose()?;
    accumulator.push(
        &row.logits,
        &row.weights,
        row.cross_entropy,
        aggregate_mse,
        query_norm,
        &key_norms,
        normalization_factor,
    )
}

fn evaluate_localization_attention_once(
    documents: &[CapturedDocument],
    lorentz_parameters: &HelmDLearnedManifoldParameters,
    euclidean_parameters: &HelmDLearnedManifoldParameters,
    rope_theta: f32,
    rope_interleaved: bool,
) -> TestResult<LocalizationAttentionEvaluation> {
    if documents.is_empty() {
        return Err("localization attention evaluation requires documents".into());
    }
    let mut pooled_lm = LocalizationArmAccumulator::new(
        "L-M",
        HelmDLearnedManifoldMetric::Lorentz,
        HelmDLearnedManifoldValueReadout::NormalizedLorentzCentroid,
    );
    let mut pooled_lt = LocalizationArmAccumulator::new(
        "L-T",
        HelmDLearnedManifoldMetric::Lorentz,
        HelmDLearnedManifoldValueReadout::TransportedTangentArithmeticSum,
    );
    let mut pooled_em = LocalizationArmAccumulator::new(
        "E-M",
        HelmDLearnedManifoldMetric::Euclidean,
        HelmDLearnedManifoldValueReadout::NormalizedLorentzCentroid,
    );
    let mut pooled_et = LocalizationArmAccumulator::new(
        "E-T",
        HelmDLearnedManifoldMetric::Euclidean,
        HelmDLearnedManifoldValueReadout::TransportedTangentArithmeticSum,
    );
    let mut document_reports = Vec::with_capacity(documents.len());
    let mut all_pairs_identical = true;
    let mut future_position_reads = 0_u64;
    for document in documents {
        if document.document.tokens.len() != INPUT_POSITIONS
            || document.decoder_audit.future_reads != 0
        {
            return Err("localization attention capture is not strict causal input".into());
        }
        future_position_reads =
            future_position_reads.saturating_add(document.decoder_audit.future_reads);
        let mut document_lm = LocalizationArmAccumulator::new(
            "L-M",
            HelmDLearnedManifoldMetric::Lorentz,
            HelmDLearnedManifoldValueReadout::NormalizedLorentzCentroid,
        );
        let mut document_lt = LocalizationArmAccumulator::new(
            "L-T",
            HelmDLearnedManifoldMetric::Lorentz,
            HelmDLearnedManifoldValueReadout::TransportedTangentArithmeticSum,
        );
        let mut document_em = LocalizationArmAccumulator::new(
            "E-M",
            HelmDLearnedManifoldMetric::Euclidean,
            HelmDLearnedManifoldValueReadout::NormalizedLorentzCentroid,
        );
        let mut document_et = LocalizationArmAccumulator::new(
            "E-T",
            HelmDLearnedManifoldMetric::Euclidean,
            HelmDLearnedManifoldValueReadout::TransportedTangentArithmeticSum,
        );
        let mut lorentz_identical = true;
        let mut euclidean_identical = true;
        for layer in 0..EXPECTED_LAYERS {
            for head in 0..EXPECTED_QUERY_HEADS {
                for position in SCORE_START..INPUT_POSITIONS {
                    let lorentz_first = localization_attention_row(
                        lorentz_parameters,
                        HelmDLearnedManifoldMetric::Lorentz,
                        document,
                        layer,
                        head,
                        position,
                        rope_theta,
                        rope_interleaved,
                    )?;
                    let lorentz_second = localization_attention_row(
                        lorentz_parameters,
                        HelmDLearnedManifoldMetric::Lorentz,
                        document,
                        layer,
                        head,
                        position,
                        rope_theta,
                        rope_interleaved,
                    )?;
                    let euclidean_first = localization_attention_row(
                        euclidean_parameters,
                        HelmDLearnedManifoldMetric::Euclidean,
                        document,
                        layer,
                        head,
                        position,
                        rope_theta,
                        rope_interleaved,
                    )?;
                    let euclidean_second = localization_attention_row(
                        euclidean_parameters,
                        HelmDLearnedManifoldMetric::Euclidean,
                        document,
                        layer,
                        head,
                        position,
                        rope_theta,
                        rope_interleaved,
                    )?;
                    lorentz_identical &= lorentz_first
                        .logits
                        .iter()
                        .zip(&lorentz_second.logits)
                        .all(|(left, right)| left.to_bits() == right.to_bits())
                        && lorentz_first
                            .weights
                            .iter()
                            .zip(&lorentz_second.weights)
                            .all(|(left, right)| left.to_bits() == right.to_bits());
                    euclidean_identical &= euclidean_first
                        .logits
                        .iter()
                        .zip(&euclidean_second.logits)
                        .all(|(left, right)| left.to_bits() == right.to_bits())
                        && euclidean_first
                            .weights
                            .iter()
                            .zip(&euclidean_second.weights)
                            .all(|(left, right)| left.to_bits() == right.to_bits());
                    let donor_aggregate = &document.heads[position][layer][head].donor_aggregate;
                    for accumulator in [&mut document_lm, &mut pooled_lm] {
                        push_localization_row(
                            accumulator,
                            &lorentz_first,
                            HelmDLearnedManifoldValueReadout::NormalizedLorentzCentroid,
                            donor_aggregate,
                        )?;
                    }
                    for accumulator in [&mut document_lt, &mut pooled_lt] {
                        push_localization_row(
                            accumulator,
                            &lorentz_second,
                            HelmDLearnedManifoldValueReadout::TransportedTangentArithmeticSum,
                            donor_aggregate,
                        )?;
                    }
                    for accumulator in [&mut document_em, &mut pooled_em] {
                        push_localization_row(
                            accumulator,
                            &euclidean_first,
                            HelmDLearnedManifoldValueReadout::NormalizedLorentzCentroid,
                            donor_aggregate,
                        )?;
                    }
                    for accumulator in [&mut document_et, &mut pooled_et] {
                        push_localization_row(
                            accumulator,
                            &euclidean_second,
                            HelmDLearnedManifoldValueReadout::TransportedTangentArithmeticSum,
                            donor_aggregate,
                        )?;
                    }
                }
            }
        }
        let lorentz_normalized = document_lm.finish()?;
        let lorentz_tangent = document_lt.finish()?;
        let euclidean_normalized = document_em.finish()?;
        let euclidean_tangent = document_et.finish()?;
        lorentz_identical &= lorentz_normalized.logits_cid == lorentz_tangent.logits_cid
            && lorentz_normalized.weights_cid == lorentz_tangent.weights_cid;
        euclidean_identical &= euclidean_normalized.logits_cid == euclidean_tangent.logits_cid
            && euclidean_normalized.weights_cid == euclidean_tangent.weights_cid;
        all_pairs_identical &= lorentz_identical && euclidean_identical;
        document_reports.push(LocalizationAttentionDocumentReport {
            document_id: document.document.id.clone(),
            lorentz_normalized,
            lorentz_tangent,
            euclidean_normalized,
            euclidean_tangent,
            lorentz_score_pair_bit_identical: lorentz_identical,
            euclidean_score_pair_bit_identical: euclidean_identical,
        });
    }
    let pooled = vec![
        pooled_lm.finish()?,
        pooled_lt.finish()?,
        pooled_em.finish()?,
        pooled_et.finish()?,
    ];
    all_pairs_identical &= pooled[0].logits_cid == pooled[1].logits_cid
        && pooled[0].weights_cid == pooled[1].weights_cid
        && pooled[2].logits_cid == pooled[3].logits_cid
        && pooled[2].weights_cid == pooled[3].weights_cid;
    let expected_rows_per_document =
        u64::try_from(SCORE_POSITIONS * EXPECTED_LAYERS * EXPECTED_QUERY_HEADS)?;
    let expected_source_pairs_per_document = u64::try_from(
        EXPECTED_LAYERS
            * EXPECTED_QUERY_HEADS
            * ((SCORE_START + 1)..=INPUT_POSITIONS).sum::<usize>(),
    )?;
    let total_rows = expected_rows_per_document.saturating_mul(documents.len() as u64);
    let total_pairs = expected_source_pairs_per_document.saturating_mul(documents.len() as u64);
    if pooled
        .iter()
        .any(|arm| arm.rows != total_rows || arm.source_pairs != total_pairs)
    {
        return Err("localization attention work does not match the exact ledger".into());
    }
    let normalization_factors_valid = pooled.iter().all(|arm| {
        arm.lorentz_denominator_quantiles
            .as_ref()
            .is_none_or(|quantiles| quantiles.p0 >= 1.0 - 1.0e-12)
            && arm
                .lorentz_reciprocal_multiplier_quantiles
                .as_ref()
                .is_none_or(|quantiles| quantiles.p0.is_finite() && quantiles.p0 > 0.0)
    });
    let mut evaluation = LocalizationAttentionEvaluation {
        schema: "uor-r4.helm-d-score-centroid-localization-attention-evaluation/1".to_owned(),
        offline_evaluation_frame: "canonical_model_frame_gauge_equivalent".to_owned(),
        offline_atlas_transport_status: "NOT_EXECUTED_COVARIANCE_REDUCED".to_owned(),
        documents: document_reports,
        pooled,
        score_paired_logits_and_weights_bit_identical: all_pairs_identical,
        all_normalization_factors_at_least_one_minus_tolerance: normalization_factors_valid,
        work: LocalizationAttentionWork {
            documents: documents.len(),
            rows_per_document: expected_rows_per_document,
            source_pairs_per_document: expected_source_pairs_per_document,
            total_rows_per_arm: total_rows,
            total_source_pairs_per_arm: total_pairs,
            future_position_reads,
            target_as_input_reads: 0,
        },
        replay_identical: false,
        evaluation_cid: String::new(),
    };
    evaluation.evaluation_cid = localization_attention_evaluation_cid(&evaluation)?;
    Ok(evaluation)
}

fn evaluate_localization_attention(
    documents: &[CapturedDocument],
    lorentz_parameters: &HelmDLearnedManifoldParameters,
    euclidean_parameters: &HelmDLearnedManifoldParameters,
    rope_theta: f32,
    rope_interleaved: bool,
) -> TestResult<LocalizationAttentionEvaluation> {
    let mut first = evaluate_localization_attention_once(
        documents,
        lorentz_parameters,
        euclidean_parameters,
        rope_theta,
        rope_interleaved,
    )?;
    let second = evaluate_localization_attention_once(
        documents,
        lorentz_parameters,
        euclidean_parameters,
        rope_theta,
        rope_interleaved,
    )?;
    first.replay_identical = {
        let mut expected = first.clone();
        expected.replay_identical = false;
        expected.evaluation_cid = localization_attention_evaluation_cid(&expected)?;
        expected == second
    };
    first.evaluation_cid = localization_attention_evaluation_cid(&first)?;
    if !first.replay_identical {
        return Err("localization attention evaluation replay is not byte-identical".into());
    }
    Ok(first)
}

fn localization_tangent_registered_frame_covariance() -> TestResult<(usize, f64)> {
    let frames = canonical_registered_h4_spin_frames()?;
    if frames.len() != 120 {
        return Err("localization tangent covariance did not enumerate 120 H4 frames".into());
    }
    let values = [
        (0..HEAD_WIDTH)
            .map(|lane| (lane as f64 - 13.0) / 43.0)
            .collect::<Vec<_>>(),
        (0..HEAD_WIDTH)
            .map(|lane| (19.0 - lane as f64) / 47.0)
            .collect::<Vec<_>>(),
        (0..HEAD_WIDTH)
            .map(|lane| (lane as f64 - 29.0) / 53.0)
            .collect::<Vec<_>>(),
    ];
    let weights = [0.2, 0.3, 0.5];
    let baseline = helm_d_learned_manifold_value_readout(
        HelmDLearnedManifoldValueReadout::TransportedTangentArithmeticSum,
        &values,
        &weights,
    )?;
    let mut maximum_error = 0.0_f64;
    for frame in frames.iter().copied() {
        let encoded = values
            .iter()
            .map(|value| {
                let mut output = Vec::with_capacity(HEAD_WIDTH);
                for block in value.chunks_exact(R4_WIDTH) {
                    output.extend_from_slice(
                        &frame.encode_model_block([block[0], block[1], block[2], block[3]])?,
                    );
                }
                Ok(output)
            })
            .collect::<TestResult<Vec<_>>>()?;
        let tangent = helm_d_learned_manifold_value_readout(
            HelmDLearnedManifoldValueReadout::TransportedTangentArithmeticSum,
            &encoded,
            &weights,
        )?;
        let mut decoded = Vec::with_capacity(HEAD_WIDTH);
        for block in tangent.chunks_exact(R4_WIDTH) {
            decoded.extend_from_slice(
                &frame.decode_local_block([block[0], block[1], block[2], block[3]])?,
            );
        }
        maximum_error = maximum_error.max(
            baseline
                .iter()
                .zip(decoded)
                .map(|(left, right)| (*left - right).abs())
                .fold(0.0, f64::max),
        );
    }
    if maximum_error > 1.0e-8 {
        return Err("tangent value readout exceeds the 120-frame covariance tolerance".into());
    }
    Ok((frames.len(), maximum_error))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LocalizationMixedCovarianceReport {
    registered_frames: usize,
    lorentz_score_maximum_error: f64,
    lorentz_weight_maximum_error: f64,
    euclidean_score_maximum_error: f64,
    euclidean_weight_maximum_error: f64,
    lorentz_normalized_readout_maximum_error: f64,
    lorentz_tangent_readout_maximum_error: f64,
    euclidean_normalized_readout_maximum_error: f64,
    euclidean_tangent_readout_maximum_error: f64,
}

fn localization_mixed_registered_frame_covariance() -> TestResult<LocalizationMixedCovarianceReport>
{
    let frames = canonical_registered_h4_spin_frames()?;
    if frames.len() != 120 {
        return Err("mixed localization covariance did not enumerate 120 H4 frames".into());
    }
    let query = (0..HEAD_WIDTH)
        .map(|lane| (lane as f64 - 31.5) / 37.0)
        .collect::<Vec<_>>();
    let keys = [
        (0..HEAD_WIDTH)
            .map(|lane| (lane as f64 - 17.0) / 41.0)
            .collect::<Vec<_>>(),
        (0..HEAD_WIDTH)
            .map(|lane| (23.0 - lane as f64) / 43.0)
            .collect::<Vec<_>>(),
    ];
    let values = [
        (0..HEAD_WIDTH)
            .map(|lane| (lane as f64 - 7.0) / 47.0)
            .collect::<Vec<_>>(),
        (0..HEAD_WIDTH)
            .map(|lane| (11.0 - lane as f64) / 53.0)
            .collect::<Vec<_>>(),
    ];
    let mut report = LocalizationMixedCovarianceReport {
        registered_frames: frames.len(),
        lorentz_score_maximum_error: 0.0,
        lorentz_weight_maximum_error: 0.0,
        euclidean_score_maximum_error: 0.0,
        euclidean_weight_maximum_error: 0.0,
        lorentz_normalized_readout_maximum_error: 0.0,
        lorentz_tangent_readout_maximum_error: 0.0,
        euclidean_normalized_readout_maximum_error: 0.0,
        euclidean_tangent_readout_maximum_error: 0.0,
    };
    for metric in [
        HelmDLearnedManifoldMetric::Lorentz,
        HelmDLearnedManifoldMetric::Euclidean,
    ] {
        let baseline_logits = keys
            .iter()
            .map(|key| helm_d_learned_manifold_logit(metric, &query, key, 2.5, -0.125))
            .collect::<Result<Vec<_>, _>>()?;
        let baseline_weights = stable_softmax_f64(&baseline_logits)?;
        let baseline_normalized = helm_d_learned_manifold_value_readout(
            HelmDLearnedManifoldValueReadout::NormalizedLorentzCentroid,
            &values,
            &baseline_weights,
        )?;
        let baseline_tangent = helm_d_learned_manifold_value_readout(
            HelmDLearnedManifoldValueReadout::TransportedTangentArithmeticSum,
            &values,
            &baseline_weights,
        )?;
        for frame in frames.iter().copied() {
            let encode = |vector: &[f64]| -> TestResult<Vec<f64>> {
                let mut output = Vec::with_capacity(HEAD_WIDTH);
                for block in vector.chunks_exact(R4_WIDTH) {
                    output.extend_from_slice(
                        &frame.encode_model_block([block[0], block[1], block[2], block[3]])?,
                    );
                }
                Ok(output)
            };
            let encoded_query = encode(&query)?;
            let encoded_keys = keys
                .iter()
                .map(|key| encode(key))
                .collect::<TestResult<Vec<_>>>()?;
            let encoded_values = values
                .iter()
                .map(|value| encode(value))
                .collect::<TestResult<Vec<_>>>()?;
            let encoded_logits = encoded_keys
                .iter()
                .map(|key| helm_d_learned_manifold_logit(metric, &encoded_query, key, 2.5, -0.125))
                .collect::<Result<Vec<_>, _>>()?;
            let encoded_weights = stable_softmax_f64(&encoded_logits)?;
            let score_error = baseline_logits
                .iter()
                .zip(&encoded_logits)
                .map(|(left, right)| (*left - *right).abs())
                .fold(0.0, f64::max);
            let weight_error = baseline_weights
                .iter()
                .zip(&encoded_weights)
                .map(|(left, right)| (*left - *right).abs())
                .fold(0.0, f64::max);
            let encoded_normalized = helm_d_learned_manifold_value_readout(
                HelmDLearnedManifoldValueReadout::NormalizedLorentzCentroid,
                &encoded_values,
                &encoded_weights,
            )?;
            let encoded_tangent = helm_d_learned_manifold_value_readout(
                HelmDLearnedManifoldValueReadout::TransportedTangentArithmeticSum,
                &encoded_values,
                &encoded_weights,
            )?;
            let decode = |vector: &[f64]| -> TestResult<Vec<f64>> {
                let mut output = Vec::with_capacity(HEAD_WIDTH);
                for block in vector.chunks_exact(R4_WIDTH) {
                    output.extend_from_slice(
                        &frame.decode_local_block([block[0], block[1], block[2], block[3]])?,
                    );
                }
                Ok(output)
            };
            let decoded_normalized = decode(&encoded_normalized)?;
            let decoded_tangent = decode(&encoded_tangent)?;
            let normalized_error = baseline_normalized
                .iter()
                .zip(decoded_normalized)
                .map(|(left, right)| (*left - right).abs())
                .fold(0.0, f64::max);
            let tangent_error = baseline_tangent
                .iter()
                .zip(decoded_tangent)
                .map(|(left, right)| (*left - right).abs())
                .fold(0.0, f64::max);
            match metric {
                HelmDLearnedManifoldMetric::Lorentz => {
                    report.lorentz_score_maximum_error =
                        report.lorentz_score_maximum_error.max(score_error);
                    report.lorentz_weight_maximum_error =
                        report.lorentz_weight_maximum_error.max(weight_error);
                    report.lorentz_normalized_readout_maximum_error = report
                        .lorentz_normalized_readout_maximum_error
                        .max(normalized_error);
                    report.lorentz_tangent_readout_maximum_error = report
                        .lorentz_tangent_readout_maximum_error
                        .max(tangent_error);
                }
                HelmDLearnedManifoldMetric::Euclidean => {
                    report.euclidean_score_maximum_error =
                        report.euclidean_score_maximum_error.max(score_error);
                    report.euclidean_weight_maximum_error =
                        report.euclidean_weight_maximum_error.max(weight_error);
                    report.euclidean_normalized_readout_maximum_error = report
                        .euclidean_normalized_readout_maximum_error
                        .max(normalized_error);
                    report.euclidean_tangent_readout_maximum_error = report
                        .euclidean_tangent_readout_maximum_error
                        .max(tangent_error);
                }
            }
        }
    }
    if [
        report.lorentz_score_maximum_error,
        report.lorentz_weight_maximum_error,
        report.euclidean_score_maximum_error,
        report.euclidean_weight_maximum_error,
        report.lorentz_normalized_readout_maximum_error,
        report.lorentz_tangent_readout_maximum_error,
        report.euclidean_normalized_readout_maximum_error,
        report.euclidean_tangent_readout_maximum_error,
    ]
    .iter()
    .any(|error| !error.is_finite() || *error > 1.0e-8)
    {
        return Err("one score/readout pair exceeds the 120-frame covariance tolerance".into());
    }
    Ok(report)
}

fn localization_lm_lt_mse_passes(
    evaluation: &LocalizationAttentionEvaluation,
    require_pooled_ten_percent: bool,
) -> TestResult<bool> {
    let per_document = evaluation.documents.iter().all(|document| {
        localization_document_mse_passes(
            require_pooled_ten_percent,
            document.lorentz_normalized.normalized_aggregate_mse,
            document.lorentz_tangent.normalized_aggregate_mse,
        )
    });
    let pooled_lm = evaluation
        .pooled
        .iter()
        .find(|arm| arm.name == "L-M")
        .ok_or("localization pooled L-M report is missing")?;
    let pooled_lt = evaluation
        .pooled
        .iter()
        .find(|arm| arm.name == "L-T")
        .ok_or("localization pooled L-T report is missing")?;
    Ok(per_document
        && (!require_pooled_ten_percent
            || pooled_lt.normalized_aggregate_mse <= 0.9 * pooled_lm.normalized_aggregate_mse))
}

fn localization_document_mse_passes(full_audit: bool, normalized: f64, tangent: f64) -> bool {
    if full_audit {
        tangent < normalized
    } else {
        tangent <= 0.9 * normalized
    }
}

fn localization_euclidean_score_passes(
    evaluation: &LocalizationAttentionEvaluation,
) -> TestResult<bool> {
    let per_document = evaluation.documents.iter().all(|document| {
        document.euclidean_normalized.donor_attention_cross_entropy
            <= document.lorentz_normalized.donor_attention_cross_entropy - 0.01
    });
    let pooled_l = evaluation
        .pooled
        .iter()
        .find(|arm| arm.name == "L-M")
        .ok_or("localization pooled L-M report is missing")?;
    let pooled_e = evaluation
        .pooled
        .iter()
        .find(|arm| arm.name == "E-M")
        .ok_or("localization pooled E-M report is missing")?;
    Ok(per_document
        && pooled_e.donor_attention_cross_entropy <= pooled_l.donor_attention_cross_entropy - 0.01)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LocalizationPreflightReport {
    schema: String,
    document_ids: Vec<String>,
    untouched_parameter_cid: String,
    identity_qkv: bool,
    layer_scale: f64,
    uniform_bias: f64,
    registered_frames: usize,
    normalized_centroid_covariance_maximum_error: f64,
    tangent_covariance_maximum_error: f64,
    mixed_score_readout_covariance: LocalizationMixedCovarianceReport,
    natural_atlas_source_permutation_live: bool,
    identity_projection_ordering_compared_lanes: u64,
    attention: LocalizationAttentionEvaluation,
    tangent_mse_ten_percent_better_on_each_document: bool,
    infrastructure_passed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct LocalizationForwardLedger {
    preflight_trace_forwards: u64,
    complete_trace_forwards: u64,
    paired_decoder_forwards: u64,
    total_forwards: u64,
    baseline_forward_calls: u64,
    measured_forward_calls: u64,
    measured_streams_started: u64,
    measured_streams_completed: u64,
    measured_multiworker_forward_calls: u64,
    requested_workers: usize,
    effective_workers: usize,
    maximum_active_workers: usize,
    active_workers_at_snapshot: usize,
    active_streams_at_snapshot: usize,
}

fn localization_forward_ledger(
    baseline: TeacherExecutionSnapshot,
    current: TeacherExecutionSnapshot,
    complete_trace_forwards: u64,
    paired_decoder_forwards: u64,
) -> TestResult<LocalizationForwardLedger> {
    let measured_forward_calls = current
        .forward_calls
        .checked_sub(baseline.forward_calls)
        .ok_or("localization forward counter moved backwards")?;
    let measured_streams_started = current
        .streams_started
        .checked_sub(baseline.streams_started)
        .ok_or("localization started-stream counter moved backwards")?;
    let measured_streams_completed = current
        .streams_completed
        .checked_sub(baseline.streams_completed)
        .ok_or("localization completed-stream counter moved backwards")?;
    let measured_multiworker_forward_calls = current
        .multiworker_forward_calls
        .checked_sub(baseline.multiworker_forward_calls)
        .ok_or("localization multiworker-forward counter moved backwards")?;
    let expected_total = complete_trace_forwards.saturating_add(paired_decoder_forwards);
    if measured_forward_calls != expected_total
        || measured_streams_started != expected_total
        || measured_streams_completed != expected_total
        || measured_multiworker_forward_calls != expected_total
        || current.requested_workers != SHARDS
        || current.effective_workers != SHARDS
        || current.max_active_workers < 2
        || current.forward_max_active_workers < 2
        || current.active_workers != 0
        || current.active_streams != 0
    {
        return Err("measured exact-forward counters differ from the frozen ledger".into());
    }
    Ok(LocalizationForwardLedger {
        preflight_trace_forwards: 64,
        complete_trace_forwards,
        paired_decoder_forwards,
        total_forwards: expected_total,
        baseline_forward_calls: baseline.forward_calls,
        measured_forward_calls,
        measured_streams_started,
        measured_streams_completed,
        measured_multiworker_forward_calls,
        requested_workers: current.requested_workers,
        effective_workers: current.effective_workers,
        maximum_active_workers: current.max_active_workers,
        active_workers_at_snapshot: current.active_workers,
        active_streams_at_snapshot: current.active_streams,
    })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LocalizationFitReplayReport {
    lorentz_primary_parameter_cid: String,
    lorentz_replay_parameter_cid: String,
    euclidean_primary_parameter_cid: String,
    euclidean_replay_parameter_cid: String,
    lorentz_primary_report_cid: String,
    lorentz_replay_report_cid: String,
    euclidean_primary_report_cid: String,
    euclidean_replay_report_cid: String,
    lorentz_parameter_bytes_identical: bool,
    euclidean_parameter_bytes_identical: bool,
    lorentz_report_identical: bool,
    euclidean_report_identical: bool,
    replay_cid: String,
}

fn localization_fit_replay_cid(report: &LocalizationFitReplayReport) -> TestResult<String> {
    let mut commitment = report.clone();
    commitment.replay_cid.clear();
    canonical_json_cid(&commitment)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct LocalizationPredecessorIdentity {
    result_bytes_cid: String,
    result_self_cid: String,
    checkpoint_bytes_cid: String,
    checkpoint_self_cid: String,
    partition_cid: String,
    manifest_cid: String,
}

fn localization_predecessor_identity() -> TestResult<LocalizationPredecessorIdentity> {
    let mut result: serde_json::Value = serde_json::from_slice(COMPILED_PREDECESSOR_RESULT)?;
    let result_claimed = result
        .get("result_cid")
        .and_then(serde_json::Value::as_str)
        .ok_or("predecessor result omitted result_cid")?
        .to_owned();
    let result_checkpoint_cid = result
        .get("checkpoint_cid")
        .and_then(serde_json::Value::as_str)
        .ok_or("predecessor result omitted checkpoint_cid")?
        .to_owned();
    let result_partition_cid = result
        .get("partition_cid")
        .and_then(serde_json::Value::as_str)
        .ok_or("predecessor result omitted partition_cid")?
        .to_owned();
    *result
        .get_mut("result_cid")
        .ok_or("predecessor result_cid is unavailable")? = serde_json::Value::String(String::new());
    let result_self_cid = canonical_json_cid(&result)?;
    let checkpoint: serde_json::Value = serde_json::from_slice(COMPILED_PREDECESSOR_CHECKPOINT)?;
    let checkpoint_claimed = checkpoint
        .get("checkpoint_cid")
        .and_then(serde_json::Value::as_str)
        .ok_or("predecessor checkpoint omitted checkpoint_cid")?
        .to_owned();
    let checkpoint_payload = checkpoint
        .get("checkpoint")
        .ok_or("predecessor checkpoint omitted payload")?;
    let checkpoint_self_cid = canonical_json_cid(checkpoint_payload)?;
    let checkpoint_partition_cid = checkpoint_payload
        .get("partition_cid")
        .and_then(serde_json::Value::as_str)
        .ok_or("predecessor checkpoint payload omitted partition_cid")?
        .to_owned();
    let checkpoint_manifest_cid = checkpoint_payload
        .get("manifest_cid")
        .and_then(serde_json::Value::as_str)
        .ok_or("predecessor checkpoint payload omitted manifest_cid")?
        .to_owned();
    if result_claimed != result_self_cid
        || result_self_cid != LOCALIZATION_PREDECESSOR_RESULT_CID
        || checkpoint_claimed != checkpoint_self_cid
        || checkpoint_self_cid != LOCALIZATION_PREDECESSOR_CHECKPOINT_CID
        || result_checkpoint_cid != checkpoint_claimed
        || result_partition_cid != checkpoint_partition_cid
        || checkpoint_partition_cid != LOCALIZATION_PARTITION_CID
        || checkpoint_manifest_cid != LOCALIZATION_PREDECESSOR_MANIFEST_CID
    {
        return Err("predecessor result/checkpoint self-CID no longer resolves".into());
    }
    Ok(LocalizationPredecessorIdentity {
        result_bytes_cid: cid_bytes(COMPILED_PREDECESSOR_RESULT),
        result_self_cid,
        checkpoint_bytes_cid: cid_bytes(COMPILED_PREDECESSOR_CHECKPOINT),
        checkpoint_self_cid,
        partition_cid: checkpoint_partition_cid,
        manifest_cid: checkpoint_manifest_cid,
    })
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct LocalizationImplementationIdentity {
    contract_cid: String,
    predecessor_partition_cid: String,
    predecessor: LocalizationPredecessorIdentity,
    implementation_source_cid: String,
    executable_cid: String,
    git_revision: String,
    git_tracked_tree_clean: bool,
}

fn localization_implementation_identity() -> TestResult<LocalizationImplementationIdentity> {
    let mut source = blake3::Hasher::new();
    source.update(b"uor-r4.helm-d-score-centroid-localization.implementation/1\0");
    for bytes in [
        COMPILED_LOCALIZATION_CONTRACT,
        COMPILED_LOCALIZATION_RUNNER,
        COMPILED_CORE_SOURCE,
        COMPILED_HARNESS_SOURCE,
        COMPILED_MODEL_ATTENTION_SOURCE,
        COMPILED_MODEL_SOURCE,
    ] {
        source.update(&u64::try_from(bytes.len())?.to_le_bytes());
        source.update(bytes);
    }
    Ok(LocalizationImplementationIdentity {
        contract_cid: cid_bytes(COMPILED_LOCALIZATION_CONTRACT),
        predecessor_partition_cid: LOCALIZATION_PARTITION_CID.to_owned(),
        predecessor: localization_predecessor_identity()?,
        implementation_source_cid: format!("blake3:{}", source.finalize().to_hex()),
        executable_cid: file_cid(&env::current_exe()?)?,
        git_revision: verified_git_revision()?,
        git_tracked_tree_clean: true,
    })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LocalizationCheckpoint {
    schema: String,
    issue: u32,
    phase: String,
    contract_cid: String,
    population_cid: String,
    partition_cid: String,
    manifest_cid: String,
    donor_cid: String,
    tokenizer_cid: String,
    target_commitment_artifact_cid: String,
    target_commitment_manifest_cid: String,
    upstream_source_commit: String,
    implementation: LocalizationImplementationIdentity,
    exact_workers: usize,
    execution_preparation: TeacherExecutionPreparation,
    preflight: LocalizationPreflightReport,
    score_fit_document_ids: Vec<String>,
    localization_audit_document_ids: Vec<String>,
    lorentz_parameters: Option<HelmDLearnedManifoldParameters>,
    lorentz_fit_report: Option<LocalizationFitReport>,
    euclidean_parameters: Option<HelmDLearnedManifoldParameters>,
    euclidean_fit_report: Option<LocalizationFitReport>,
    fit_replay: Option<LocalizationFitReplayReport>,
    fit_population_attention: Option<LocalizationAttentionEvaluation>,
    audit_population_attention: Option<LocalizationAttentionEvaluation>,
    forward_ledger: LocalizationForwardLedger,
    target_materialized: bool,
    d3_status: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LocalizationCheckpointEnvelope {
    checkpoint_cid: String,
    checkpoint: LocalizationCheckpoint,
}

fn write_localization_checkpoint(
    path: &Path,
    checkpoint: LocalizationCheckpoint,
) -> TestResult<LocalizationCheckpointEnvelope> {
    let full = checkpoint.phase == "full_attention_audit";
    let optional_full_fields = [
        checkpoint.lorentz_parameters.is_some(),
        checkpoint.lorentz_fit_report.is_some(),
        checkpoint.euclidean_parameters.is_some(),
        checkpoint.euclidean_fit_report.is_some(),
        checkpoint.fit_replay.is_some(),
        checkpoint.fit_population_attention.is_some(),
        checkpoint.audit_population_attention.is_some(),
    ];
    if checkpoint.schema != LOCALIZATION_CHECKPOINT_SCHEMA
        || checkpoint.issue != 973
        || !matches!(
            checkpoint.phase.as_str(),
            "preflight_rejected" | "full_attention_audit"
        )
        || checkpoint.contract_cid != checkpoint.implementation.contract_cid
        || checkpoint.population_cid != CORPUS_CID
        || checkpoint.partition_cid != LOCALIZATION_PARTITION_CID
        || checkpoint.donor_cid != DONOR_CID
        || checkpoint.upstream_source_commit != HELM_D_UPSTREAM_COMMIT
        || !checkpoint
            .target_commitment_artifact_cid
            .starts_with("blake3:")
        || !checkpoint
            .target_commitment_manifest_cid
            .starts_with("blake3:")
        || checkpoint.exact_workers != SHARDS
        || checkpoint.target_materialized
        || checkpoint.d3_status != "NOT_RUN"
        || !checkpoint.preflight.infrastructure_passed
        || checkpoint.preflight.attention.offline_evaluation_frame
            != "canonical_model_frame_gauge_equivalent"
        || checkpoint
            .preflight
            .attention
            .offline_atlas_transport_status
            != "NOT_EXECUTED_COVARIANCE_REDUCED"
        || checkpoint
            .preflight
            .mixed_score_readout_covariance
            .registered_frames
            != 120
        || !checkpoint.preflight.natural_atlas_source_permutation_live
        || [
            checkpoint
                .preflight
                .mixed_score_readout_covariance
                .lorentz_score_maximum_error,
            checkpoint
                .preflight
                .mixed_score_readout_covariance
                .lorentz_weight_maximum_error,
            checkpoint
                .preflight
                .mixed_score_readout_covariance
                .euclidean_score_maximum_error,
            checkpoint
                .preflight
                .mixed_score_readout_covariance
                .euclidean_weight_maximum_error,
            checkpoint
                .preflight
                .mixed_score_readout_covariance
                .lorentz_normalized_readout_maximum_error,
            checkpoint
                .preflight
                .mixed_score_readout_covariance
                .lorentz_tangent_readout_maximum_error,
            checkpoint
                .preflight
                .mixed_score_readout_covariance
                .euclidean_normalized_readout_maximum_error,
            checkpoint
                .preflight
                .mixed_score_readout_covariance
                .euclidean_tangent_readout_maximum_error,
        ]
        .iter()
        .any(|error| !error.is_finite() || *error > 1.0e-8)
        || checkpoint.forward_ledger.preflight_trace_forwards != 64
        || checkpoint.forward_ledger.paired_decoder_forwards != 0
        || checkpoint.forward_ledger.total_forwards
            != checkpoint.forward_ledger.complete_trace_forwards
        || checkpoint.forward_ledger.measured_forward_calls
            != checkpoint.forward_ledger.total_forwards
        || checkpoint.forward_ledger.measured_streams_started
            != checkpoint.forward_ledger.total_forwards
        || checkpoint.forward_ledger.measured_streams_completed
            != checkpoint.forward_ledger.total_forwards
        || checkpoint.forward_ledger.measured_multiworker_forward_calls
            != checkpoint.forward_ledger.total_forwards
        || checkpoint.forward_ledger.requested_workers != SHARDS
        || checkpoint.forward_ledger.effective_workers != SHARDS
        || checkpoint.forward_ledger.maximum_active_workers < 2
        || checkpoint.forward_ledger.active_workers_at_snapshot != 0
        || checkpoint.forward_ledger.active_streams_at_snapshot != 0
        || optional_full_fields.iter().any(|present| *present != full)
        || checkpoint
            .preflight
            .tangent_mse_ten_percent_better_on_each_document
            != full
        || (full
            && (checkpoint.score_fit_document_ids
                != LOCALIZATION_FIT_IDS.map(str::to_owned).to_vec()
                || checkpoint.localization_audit_document_ids
                    != LOCALIZATION_AUDIT_IDS.map(str::to_owned).to_vec()
                || checkpoint.forward_ledger.complete_trace_forwards != 512))
        || (!full
            && (!checkpoint.score_fit_document_ids.is_empty()
                || !checkpoint.localization_audit_document_ids.is_empty()
                || checkpoint.forward_ledger.complete_trace_forwards != 64))
    {
        return Err("localization checkpoint violates its frozen phase contract".into());
    }
    if full {
        let lorentz = checkpoint
            .lorentz_parameters
            .as_ref()
            .ok_or("full checkpoint omitted Lorentz parameters")?;
        let euclidean = checkpoint
            .euclidean_parameters
            .as_ref()
            .ok_or("full checkpoint omitted Euclidean parameters")?;
        let lorentz_report = checkpoint
            .lorentz_fit_report
            .as_ref()
            .ok_or("full checkpoint omitted Lorentz fit report")?;
        let euclidean_report = checkpoint
            .euclidean_fit_report
            .as_ref()
            .ok_or("full checkpoint omitted Euclidean fit report")?;
        validate_localization_fit_report(
            lorentz_report,
            lorentz,
            HelmDLearnedManifoldMetric::Lorentz,
        )?;
        validate_localization_fit_report(
            euclidean_report,
            euclidean,
            HelmDLearnedManifoldMetric::Euclidean,
        )?;
        let replay = checkpoint
            .fit_replay
            .as_ref()
            .ok_or("full checkpoint omitted fit replay")?;
        if !replay.lorentz_parameter_bytes_identical
            || !replay.euclidean_parameter_bytes_identical
            || !replay.lorentz_report_identical
            || !replay.euclidean_report_identical
            || replay.replay_cid != localization_fit_replay_cid(replay)?
            || replay.lorentz_primary_parameter_cid != lorentz_report.parameter_cid
            || replay.euclidean_primary_parameter_cid != euclidean_report.parameter_cid
            || replay.lorentz_primary_parameter_cid != replay.lorentz_replay_parameter_cid
            || replay.euclidean_primary_parameter_cid != replay.euclidean_replay_parameter_cid
            || replay.lorentz_primary_report_cid != lorentz_report.report_cid
            || replay.euclidean_primary_report_cid != euclidean_report.report_cid
            || replay.lorentz_primary_report_cid != replay.lorentz_replay_report_cid
            || replay.euclidean_primary_report_cid != replay.euclidean_replay_report_cid
            || !checkpoint
                .fit_population_attention
                .as_ref()
                .is_some_and(|evaluation| {
                    evaluation.replay_identical
                        && evaluation.offline_evaluation_frame
                            == "canonical_model_frame_gauge_equivalent"
                        && evaluation.offline_atlas_transport_status
                            == "NOT_EXECUTED_COVARIANCE_REDUCED"
                })
            || !checkpoint
                .audit_population_attention
                .as_ref()
                .is_some_and(|evaluation| {
                    evaluation.replay_identical
                        && evaluation.offline_evaluation_frame
                            == "canonical_model_frame_gauge_equivalent"
                        && evaluation.offline_atlas_transport_status
                            == "NOT_EXECUTED_COVARIANCE_REDUCED"
                })
        {
            return Err("localization checkpoint replay evidence is incomplete".into());
        }
    }
    let envelope = LocalizationCheckpointEnvelope {
        checkpoint_cid: canonical_json_cid(&checkpoint)?,
        checkpoint,
    };
    write_pretty_json_exclusive(path, &envelope)?;
    let readback: LocalizationCheckpointEnvelope = serde_json::from_slice(&fs::read(path)?)?;
    if readback != envelope
        || readback.checkpoint_cid != canonical_json_cid(&readback.checkpoint)?
        || canonical_json_bytes(&readback)? != canonical_json_bytes(&envelope)?
    {
        return Err("localization checkpoint readback is not byte-identical".into());
    }
    Ok(readback)
}

fn validate_localization_fit_report(
    report: &LocalizationFitReport,
    parameters: &HelmDLearnedManifoldParameters,
    metric: HelmDLearnedManifoldMetric,
) -> TestResult {
    let rows = u64::try_from(
        LOCALIZATION_FIT_DOCUMENTS * SCORE_POSITIONS * EXPECTED_LAYERS * EXPECTED_QUERY_HEADS,
    )?;
    let pairs = u64::try_from(
        LOCALIZATION_FIT_DOCUMENTS
            * EXPECTED_LAYERS
            * EXPECTED_QUERY_HEADS
            * ((SCORE_START + 1)..=INPUT_POSITIONS).sum::<usize>(),
    )?;
    let evaluations = LOCALIZATION_ADAM_STEPS + 2;
    if report.schema != "uor-r4.helm-d-score-centroid-localization-fit/1"
        || report.metric != metric
        || report.objective != "donor-attention-cross-entropy-plus-qk-only-ridge"
        || report.steps != LOCALIZATION_ADAM_STEPS
        || report.workers != SHARDS
        || report.fit_documents != LOCALIZATION_FIT_DOCUMENTS
        || report.fit_rows != rows
        || !report.initial_objective.is_finite()
        || !report.final_objective.is_finite()
        || report.total_parameter_scalars != PARAMETER_SCALARS
        || report.trainable_parameter_scalars != LOCALIZATION_TRAINABLE_PARAMETER_SCALARS
        || report.frozen_parameter_scalars != LOCALIZATION_FROZEN_PARAMETER_SCALARS
        || !report.value_adapters_identity
        || !report.uniform_bias_zero
        || !localization_parameters_are_frozen(parameters)
        || report.parameter_cid != public_parameter_identity(parameters)?
        || report.gradient_audit.schema
            != "uor-r4.helm-d-score-centroid-localization-gradient-audit/1"
        || report.gradient_audit.gradient_evaluations != evaluations
        || report.gradient_audit.optimizer_gradient_evaluations != LOCALIZATION_ADAM_STEPS
        || report.gradient_audit.diagnostic_gradient_evaluations != 2
        || report.gradient_audit.ordered_evaluation_cids.len() != evaluations
        || report.gradient_audit_cid != canonical_json_cid(&report.gradient_audit)?
        || report.work.workers != SHARDS
        || report.work.full_batch_steps != LOCALIZATION_ADAM_STEPS
        || report.work.full_batch_evaluations != evaluations
        || report
            .work
            .rows_per_shard_per_evaluation
            .iter()
            .sum::<u64>()
            != rows
        || report
            .work
            .source_pairs_per_shard_per_evaluation
            .iter()
            .sum::<u64>()
            != pairs
        || report.work.total_row_evaluations != rows.saturating_mul(evaluations as u64)
        || report.work.total_source_pair_evaluations != pairs.saturating_mul(evaluations as u64)
        || report.report_cid != localization_fit_report_cid(report)?
    {
        return Err("localization fit report fails its exact work/freeze schema".into());
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LocalizationDecoderComparison {
    lorentz_normalized: ArmReport,
    lorentz_tangent: ArmReport,
    tangent_minus_normalized_nll: f64,
    tangent_nll_improves_by_at_least_005: bool,
    exact_causal_work: bool,
    replay_identical: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LocalizationTiming {
    preflight_trace_seconds: f64,
    preflight_evaluation_seconds: f64,
    remaining_trace_seconds: f64,
    fit_and_replay_seconds: f64,
    attention_evaluation_seconds: f64,
    decoder_seconds: f64,
    total_seconds: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LocalizationResultPayload {
    schema: String,
    issue: u32,
    terminal: String,
    partition_cid: String,
    checkpoint_cid: String,
    target_commitment_artifact_cid: String,
    implementation: LocalizationImplementationIdentity,
    preflight: LocalizationPreflightReport,
    fit_population_attention: Option<LocalizationAttentionEvaluation>,
    audit_population_attention: Option<LocalizationAttentionEvaluation>,
    decoder: Option<LocalizationDecoderComparison>,
    forward_ledger: LocalizationForwardLedger,
    execution_snapshot: TeacherExecutionSnapshot,
    timing: LocalizationTiming,
    audit_targets_materialized_after_checkpoint: bool,
    v2_validation_status: String,
    d3_status: String,
    result_cid: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LocalizationUnavailablePayload {
    schema: String,
    issue: u32,
    terminal: String,
    reason: String,
    checkpoint_exists: bool,
    audit_targets_materialized: bool,
    v2_validation_status: String,
    d3_status: String,
    result_cid: String,
}

impl SelfCommittedResult for LocalizationResultPayload {
    fn result_cid(&self) -> &str {
        &self.result_cid
    }

    fn result_cid_mut(&mut self) -> &mut String {
        &mut self.result_cid
    }
}

impl SelfCommittedResult for LocalizationUnavailablePayload {
    fn result_cid(&self) -> &str {
        &self.result_cid
    }

    fn result_cid_mut(&mut self) -> &mut String {
        &mut self.result_cid
    }
}

fn write_localization_unavailable(
    path: &Path,
    reason: &str,
    checkpoint_exists: bool,
    audit_targets_materialized: bool,
) -> TestResult<String> {
    let mut payload = LocalizationUnavailablePayload {
        schema: LOCALIZATION_RESULT_SCHEMA.to_owned(),
        issue: 973,
        terminal: LOCALIZATION_UNAVAILABLE_TERMINAL.to_owned(),
        reason: reason.to_owned(),
        checkpoint_exists,
        audit_targets_materialized,
        v2_validation_status: "NOT_OPENED_OR_RESCORED".to_owned(),
        d3_status: "NOT_RUN".to_owned(),
        result_cid: String::new(),
    };
    payload.result_cid = result_payload_cid(&payload)?;
    write_result(path, &payload)
}

fn localization_terminal_for(
    preflight_tangent_passed: bool,
    audit_tangent_passed: Option<bool>,
    euclidean_score_passed: Option<bool>,
    decoder_tangent_passed: Option<bool>,
) -> &'static str {
    if !preflight_tangent_passed {
        return LOCALIZATION_REJECT_TANGENT_PREFLIGHT_TERMINAL;
    }
    match (
        audit_tangent_passed,
        euclidean_score_passed,
        decoder_tangent_passed,
    ) {
        (Some(true), _, Some(true)) => LOCALIZATION_SELECT_TANGENT_TERMINAL,
        (Some(false), Some(true), None) => LOCALIZATION_SELECT_SCORE_TERMINAL,
        (Some(_), Some(_), _) => LOCALIZATION_REVISE_TERMINAL,
        _ => LOCALIZATION_UNAVAILABLE_TERMINAL,
    }
}

#[test]
fn helm_d_score_centroid_localization_r4_v1_schema_and_decisions_are_pinned() -> TestResult {
    if LOCALIZATION_CHECKPOINT_SCHEMA != "uor-r4.helm-d-score-centroid-localization-r4-checkpoint/1"
        || LOCALIZATION_RESULT_SCHEMA != "uor-r4.helm-d-score-centroid-localization-r4-result/1"
        || LOCALIZATION_FIT_IDS.len() != 8
        || LOCALIZATION_AUDIT_IDS.len() != 8
        || LOCALIZATION_FIT_IDS
            .iter()
            .any(|id| LOCALIZATION_AUDIT_IDS.contains(id))
        || localization_terminal_for(false, None, None, None)
            != LOCALIZATION_REJECT_TANGENT_PREFLIGHT_TERMINAL
        || localization_terminal_for(true, Some(true), Some(true), Some(true))
            != LOCALIZATION_SELECT_TANGENT_TERMINAL
        || localization_terminal_for(true, Some(false), Some(true), None)
            != LOCALIZATION_SELECT_SCORE_TERMINAL
        || localization_terminal_for(true, Some(true), Some(true), Some(false))
            != LOCALIZATION_REVISE_TERMINAL
        || localization_document_mse_passes(true, 1.0, 1.0)
        || !localization_document_mse_passes(true, 1.0, 1.0 - f64::EPSILON)
        || !localization_document_mse_passes(false, 1.0, 0.9)
        || localization_document_mse_passes(false, 1.0, 0.9 + f64::EPSILON)
    {
        return Err("localization schema or exclusive terminal mapping drifted".into());
    }
    let quantiles = localization_quantiles((1..=20).map(|value| value as f64).collect())?;
    if quantiles
        != (LocalizationQuantiles {
            p0: 1.0,
            p50: 10.0,
            p95: 19.0,
            p100: 20.0,
        })
    {
        return Err("nearest-rank localization quantiles drifted".into());
    }
    Ok(())
}

#[test]
fn helm_d_score_centroid_localization_r4_v1_predecessor_and_covariance_are_sealed() -> TestResult {
    let predecessor = localization_predecessor_identity()?;
    let (frames, tangent_error) = localization_tangent_registered_frame_covariance()?;
    let mixed = localization_mixed_registered_frame_covariance()?;
    if predecessor.result_self_cid != LOCALIZATION_PREDECESSOR_RESULT_CID
        || predecessor.checkpoint_self_cid != LOCALIZATION_PREDECESSOR_CHECKPOINT_CID
        || frames != 120
        || mixed.registered_frames != 120
        || tangent_error > 1.0e-8
    {
        return Err("localization predecessor or tangent covariance seal drifted".into());
    }
    Ok(())
}

#[test]
#[ignore = "runs the frozen two-document gate and admitted 8/8 score-readout localization"]
fn helm_d_score_centroid_localization_r4_v1_decision() -> TestResult {
    let result_path = required_path_from_env(LOCALIZATION_OUTPUT_ENV)?;
    let checkpoint_path = required_path_from_env(LOCALIZATION_CHECKPOINT_ENV)?;
    let mut audit_targets_materialized = false;
    match run_helm_d_score_centroid_localization_r4_v1(
        &result_path,
        &checkpoint_path,
        &mut audit_targets_materialized,
    ) {
        Ok(()) => Ok(()),
        Err(error) => {
            if !result_path.exists() {
                let _ = write_localization_unavailable(
                    &result_path,
                    &error.to_string(),
                    checkpoint_path.is_file(),
                    audit_targets_materialized,
                );
            }
            Err(error)
        }
    }
}

fn run_helm_d_score_centroid_localization_r4_v1(
    result_path: &Path,
    checkpoint_path: &Path,
    audit_targets_materialized: &mut bool,
) -> TestResult {
    let total_started = Instant::now();
    if env::var(CANONICAL_DETERMINISTIC_ENV).as_deref() != Ok("1") {
        return Err(format!("{CANONICAL_DETERMINISTIC_ENV}=1 is required").into());
    }
    if result_path.exists() || checkpoint_path.exists() {
        return Err("exclusive localization result or checkpoint path already exists".into());
    }
    let partition_path = required_path_from_env(LOCALIZATION_PARTITION_ENV)?;
    let partition_envelope = parse_partition(&fs::read(partition_path)?)?;
    let partition = &partition_envelope.manifest;
    let implementation = localization_implementation_identity()?;
    let target_commitment_path = required_path_from_env(LOCALIZATION_TARGET_ENV)?;
    let target_envelope: LocalizationTargetEnvelope =
        serde_json::from_slice(&fs::read(target_commitment_path)?)?;
    validate_localization_target_envelope(&target_envelope, partition, &implementation)?;
    let fit_ids = partition.construction_fit[..LOCALIZATION_FIT_DOCUMENTS]
        .iter()
        .map(|document| document.id.as_str())
        .collect::<Vec<_>>();
    let audit_ids = partition.construction_fit
        [LOCALIZATION_FIT_DOCUMENTS..LOCALIZATION_FIT_DOCUMENTS + LOCALIZATION_AUDIT_DOCUMENTS]
        .iter()
        .map(|document| document.id.as_str())
        .collect::<Vec<_>>();
    if partition.partition_cid != LOCALIZATION_PARTITION_CID
        || fit_ids != LOCALIZATION_FIT_IDS
        || audit_ids != LOCALIZATION_AUDIT_IDS
    {
        return Err(
            "localization partition does not expose the frozen 8/8 construction split".into(),
        );
    }
    let tokenizer_path = path_from_env(TOKENIZER_ENV, DEFAULT_TOKENIZER);
    let corpus_path = path_from_env(CORPUS_ENV, DEFAULT_CORPUS);
    let model_path = path_from_env(MODEL_ENV, DEFAULT_MODEL);
    verify_corpus_manifest(&corpus_path)?;
    if file_cid(&tokenizer_path)? != partition.tokenizer_cid {
        return Err("localization tokenizer identity differs from the frozen partition".into());
    }
    let tokenizer = Tokenizer::try_load(&tokenizer_path)?;
    let workers = NonZeroUsize::new(SHARDS).ok_or("eight workers must be nonzero")?;
    let oracle = HuggingFaceLlamaOracle::load_with_execution(
        &model_path,
        TeacherExecutionConfig::fixed_workers(workers),
    )?;
    if oracle.source_cid() != DONOR_CID || oracle.source_cid() != partition.donor_cid {
        return Err("localization donor identity differs from the frozen partition".into());
    }
    let config = oracle.cfg();
    if config.n_layers != EXPECTED_LAYERS
        || config.n_heads != EXPECTED_QUERY_HEADS
        || config.n_kv_heads != EXPECTED_KV_HEADS
        || config.dim / config.n_heads != HEAD_WIDTH
        || config.r4_attention
        || config.seq_len < INPUT_POSITIONS
    {
        return Err("localization donor geometry differs from the frozen operator".into());
    }
    let rope_theta = config.rope_theta;
    let rope_interleaved = config.rope_interleaved;
    let maximum_token_id = u32::try_from(config.vocab.checked_sub(1).ok_or("empty vocab")?)?;
    let preparation = oracle.prepare_exact_execution(1)?;
    if preparation.workers_observed != SHARDS || !preparation.backend_exercised {
        return Err("localization exact eight-worker donor preparation was not exercised".into());
    }
    let execution_baseline = oracle.execution_snapshot();
    let preflight_trace_started = Instant::now();
    let preflight_documents = materialize_documents(
        &corpus_path,
        &tokenizer,
        &partition.construction_fit[..LOCALIZATION_PREFLIGHT_DOCUMENTS],
        false,
    )?;
    let mut captures = capture_fit_documents(&oracle, &preflight_documents)?;
    let preflight_trace_seconds = preflight_trace_started.elapsed().as_secs_f64();
    let preflight_evaluation_started = Instant::now();
    let untouched = HelmDLearnedManifoldParameters::identity(
        EXPECTED_LAYERS,
        EXPECTED_QUERY_HEADS,
        EXPECTED_KV_HEADS,
        BLOCKS_PER_HEAD,
        INITIAL_SCALE,
    )?;
    if !localization_parameters_are_frozen(&untouched)
        || untouched
            .query_adapters()
            .iter()
            .chain(untouched.key_adapters())
            .any(|adapter| *adapter != R4AffineAdapter::identity())
        || untouched
            .learned_scales()
            .iter()
            .any(|scale| scale.to_bits() != INITIAL_SCALE.to_bits())
    {
        return Err("localization preflight initialization is not untouched".into());
    }
    let preflight_attention = evaluate_localization_attention(
        &captures,
        &untouched,
        &untouched,
        rope_theta,
        rope_interleaved,
    )?;
    let (registered_frames, _, _, _, normalized_covariance, natural_atlas_liveness) =
        registered_frame_covariance_preflight(maximum_token_id)?;
    let (tangent_frames, tangent_covariance) = localization_tangent_registered_frame_covariance()?;
    let mixed_covariance = localization_mixed_registered_frame_covariance()?;
    let compared_lanes =
        identity_projection_ordering_preflight(&captures, rope_theta, rope_interleaved)?;
    let preflight_tangent_passed = localization_lm_lt_mse_passes(&preflight_attention, false)?;
    let preflight_infrastructure = preflight_attention
        .score_paired_logits_and_weights_bit_identical
        && preflight_attention.all_normalization_factors_at_least_one_minus_tolerance
        && preflight_attention.replay_identical
        && preflight_attention.work.future_position_reads == 0
        && preflight_attention.work.target_as_input_reads == 0
        && registered_frames == 120
        && tangent_frames == 120
        && mixed_covariance.registered_frames == 120
        && natural_atlas_liveness
        && normalized_covariance <= 1.0e-8
        && tangent_covariance <= 1.0e-8;
    if !preflight_infrastructure {
        return Err(
            "localization preflight identity/covariance/causal/work/replay gate failed".into(),
        );
    }
    let preflight = LocalizationPreflightReport {
        schema: "uor-r4.helm-d-score-centroid-localization-preflight/1".to_owned(),
        document_ids: LOCALIZATION_FIT_IDS[..LOCALIZATION_PREFLIGHT_DOCUMENTS]
            .iter()
            .map(ToString::to_string)
            .collect(),
        untouched_parameter_cid: public_parameter_identity(&untouched)?,
        identity_qkv: true,
        layer_scale: INITIAL_SCALE,
        uniform_bias: 0.0,
        registered_frames,
        normalized_centroid_covariance_maximum_error: normalized_covariance,
        tangent_covariance_maximum_error: tangent_covariance,
        mixed_score_readout_covariance: mixed_covariance,
        natural_atlas_source_permutation_live: natural_atlas_liveness,
        identity_projection_ordering_compared_lanes: compared_lanes,
        attention: preflight_attention,
        tangent_mse_ten_percent_better_on_each_document: preflight_tangent_passed,
        infrastructure_passed: true,
    };
    let preflight_evaluation_seconds = preflight_evaluation_started.elapsed().as_secs_f64();

    if !preflight_tangent_passed {
        let execution_snapshot = oracle.execution_snapshot();
        let forward_ledger =
            localization_forward_ledger(execution_baseline, execution_snapshot, 64, 0)?;
        let checkpoint = write_localization_checkpoint(
            checkpoint_path,
            LocalizationCheckpoint {
                schema: LOCALIZATION_CHECKPOINT_SCHEMA.to_owned(),
                issue: 973,
                phase: "preflight_rejected".to_owned(),
                contract_cid: implementation.contract_cid.clone(),
                population_cid: CORPUS_CID.to_owned(),
                partition_cid: partition.partition_cid.clone(),
                manifest_cid: partition_envelope.manifest_cid.clone(),
                donor_cid: DONOR_CID.to_owned(),
                tokenizer_cid: partition.tokenizer_cid.clone(),
                target_commitment_artifact_cid: target_envelope.artifact_cid.clone(),
                target_commitment_manifest_cid: target_envelope.manifest.manifest_cid.clone(),
                upstream_source_commit: HELM_D_UPSTREAM_COMMIT.to_owned(),
                implementation: implementation.clone(),
                exact_workers: SHARDS,
                execution_preparation: preparation,
                preflight: preflight.clone(),
                score_fit_document_ids: Vec::new(),
                localization_audit_document_ids: Vec::new(),
                lorentz_parameters: None,
                lorentz_fit_report: None,
                euclidean_parameters: None,
                euclidean_fit_report: None,
                fit_replay: None,
                fit_population_attention: None,
                audit_population_attention: None,
                forward_ledger: forward_ledger.clone(),
                target_materialized: false,
                d3_status: "NOT_RUN".to_owned(),
            },
        )?;
        let mut payload = LocalizationResultPayload {
            schema: LOCALIZATION_RESULT_SCHEMA.to_owned(),
            issue: 973,
            terminal: LOCALIZATION_REJECT_TANGENT_PREFLIGHT_TERMINAL.to_owned(),
            partition_cid: partition.partition_cid.clone(),
            checkpoint_cid: checkpoint.checkpoint_cid,
            target_commitment_artifact_cid: target_envelope.artifact_cid.clone(),
            implementation,
            preflight,
            fit_population_attention: None,
            audit_population_attention: None,
            decoder: None,
            forward_ledger,
            execution_snapshot,
            timing: LocalizationTiming {
                preflight_trace_seconds,
                preflight_evaluation_seconds,
                remaining_trace_seconds: 0.0,
                fit_and_replay_seconds: 0.0,
                attention_evaluation_seconds: 0.0,
                decoder_seconds: 0.0,
                total_seconds: total_started.elapsed().as_secs_f64(),
            },
            audit_targets_materialized_after_checkpoint: false,
            v2_validation_status: "NOT_OPENED_OR_RESCORED".to_owned(),
            d3_status: "NOT_RUN".to_owned(),
            result_cid: String::new(),
        };
        payload.result_cid = result_payload_cid(&payload)?;
        let result_cid = write_result(result_path, &payload)?;
        eprintln!(
            "HELM-D score-centroid localization terminal={} result_cid={result_cid}",
            payload.terminal
        );
        return Ok(());
    }

    let remaining_trace_started = Instant::now();
    let remaining_documents = materialize_documents(
        &corpus_path,
        &tokenizer,
        &partition.construction_fit[LOCALIZATION_PREFLIGHT_DOCUMENTS..],
        false,
    )?;
    captures.extend(capture_fit_documents(&oracle, &remaining_documents)?);
    if captures.len() != FIT_DOCUMENTS
        || captures
            .iter()
            .map(|capture| capture.document.id.as_str())
            .collect::<Vec<_>>()
            != partition
                .construction_fit
                .iter()
                .map(|document| document.id.as_str())
                .collect::<Vec<_>>()
    {
        return Err("localization complete trace order differs from the frozen split".into());
    }
    let remaining_trace_seconds = remaining_trace_started.elapsed().as_secs_f64();
    let fit_started = Instant::now();
    let lorentz = fit_localization_score(
        HelmDLearnedManifoldMetric::Lorentz,
        &captures[..LOCALIZATION_FIT_DOCUMENTS],
        LOCALIZATION_FIT_DOCUMENTS,
        rope_theta,
        rope_interleaved,
    )?;
    let euclidean = fit_localization_score(
        HelmDLearnedManifoldMetric::Euclidean,
        &captures[..LOCALIZATION_FIT_DOCUMENTS],
        LOCALIZATION_FIT_DOCUMENTS,
        rope_theta,
        rope_interleaved,
    )?;
    let lorentz_replay = fit_localization_score(
        HelmDLearnedManifoldMetric::Lorentz,
        &captures[..LOCALIZATION_FIT_DOCUMENTS],
        LOCALIZATION_FIT_DOCUMENTS,
        rope_theta,
        rope_interleaved,
    )?;
    let euclidean_replay = fit_localization_score(
        HelmDLearnedManifoldMetric::Euclidean,
        &captures[..LOCALIZATION_FIT_DOCUMENTS],
        LOCALIZATION_FIT_DOCUMENTS,
        rope_theta,
        rope_interleaved,
    )?;
    let mut fit_replay = LocalizationFitReplayReport {
        lorentz_primary_parameter_cid: lorentz.report.parameter_cid.clone(),
        lorentz_replay_parameter_cid: lorentz_replay.report.parameter_cid.clone(),
        euclidean_primary_parameter_cid: euclidean.report.parameter_cid.clone(),
        euclidean_replay_parameter_cid: euclidean_replay.report.parameter_cid.clone(),
        lorentz_primary_report_cid: lorentz.report.report_cid.clone(),
        lorentz_replay_report_cid: lorentz_replay.report.report_cid.clone(),
        euclidean_primary_report_cid: euclidean.report.report_cid.clone(),
        euclidean_replay_report_cid: euclidean_replay.report.report_cid.clone(),
        lorentz_parameter_bytes_identical: lorentz.parameter_bytes
            == lorentz_replay.parameter_bytes,
        euclidean_parameter_bytes_identical: euclidean.parameter_bytes
            == euclidean_replay.parameter_bytes,
        lorentz_report_identical: lorentz.report == lorentz_replay.report,
        euclidean_report_identical: euclidean.report == euclidean_replay.report,
        replay_cid: String::new(),
    };
    fit_replay.replay_cid = localization_fit_replay_cid(&fit_replay)?;
    if !fit_replay.lorentz_parameter_bytes_identical
        || !fit_replay.euclidean_parameter_bytes_identical
        || !fit_replay.lorentz_report_identical
        || !fit_replay.euclidean_report_identical
    {
        return Err("localization score refits are not byte-identical".into());
    }
    let fit_and_replay_seconds = fit_started.elapsed().as_secs_f64();
    let attention_started = Instant::now();
    let fit_population_attention = evaluate_localization_attention(
        &captures[..LOCALIZATION_FIT_DOCUMENTS],
        &lorentz.parameters,
        &euclidean.parameters,
        rope_theta,
        rope_interleaved,
    )?;
    let audit_population_attention = evaluate_localization_attention(
        &captures[LOCALIZATION_FIT_DOCUMENTS..],
        &lorentz.parameters,
        &euclidean.parameters,
        rope_theta,
        rope_interleaved,
    )?;
    if !fit_population_attention.score_paired_logits_and_weights_bit_identical
        || !audit_population_attention.score_paired_logits_and_weights_bit_identical
        || !fit_population_attention.all_normalization_factors_at_least_one_minus_tolerance
        || !audit_population_attention.all_normalization_factors_at_least_one_minus_tolerance
        || fit_population_attention.work.future_position_reads != 0
        || audit_population_attention.work.future_position_reads != 0
    {
        return Err("localization full attention identity/normalization/causal gate failed".into());
    }
    let attention_evaluation_seconds = attention_started.elapsed().as_secs_f64();
    let audit_tangent_passed = localization_lm_lt_mse_passes(&audit_population_attention, true)?;
    let euclidean_score_passed = localization_euclidean_score_passes(&audit_population_attention)?;
    let full_forward_ledger =
        localization_forward_ledger(execution_baseline, oracle.execution_snapshot(), 512, 0)?;
    let checkpoint = write_localization_checkpoint(
        checkpoint_path,
        LocalizationCheckpoint {
            schema: LOCALIZATION_CHECKPOINT_SCHEMA.to_owned(),
            issue: 973,
            phase: "full_attention_audit".to_owned(),
            contract_cid: implementation.contract_cid.clone(),
            population_cid: CORPUS_CID.to_owned(),
            partition_cid: partition.partition_cid.clone(),
            manifest_cid: partition_envelope.manifest_cid.clone(),
            donor_cid: DONOR_CID.to_owned(),
            tokenizer_cid: partition.tokenizer_cid.clone(),
            target_commitment_artifact_cid: target_envelope.artifact_cid.clone(),
            target_commitment_manifest_cid: target_envelope.manifest.manifest_cid.clone(),
            upstream_source_commit: HELM_D_UPSTREAM_COMMIT.to_owned(),
            implementation: implementation.clone(),
            exact_workers: SHARDS,
            execution_preparation: preparation,
            preflight: preflight.clone(),
            score_fit_document_ids: LOCALIZATION_FIT_IDS.map(str::to_owned).to_vec(),
            localization_audit_document_ids: LOCALIZATION_AUDIT_IDS.map(str::to_owned).to_vec(),
            lorentz_parameters: Some(lorentz.parameters.clone()),
            lorentz_fit_report: Some(lorentz.report),
            euclidean_parameters: Some(euclidean.parameters.clone()),
            euclidean_fit_report: Some(euclidean.report),
            fit_replay: Some(fit_replay),
            fit_population_attention: Some(fit_population_attention.clone()),
            audit_population_attention: Some(audit_population_attention.clone()),
            forward_ledger: full_forward_ledger.clone(),
            target_materialized: false,
            d3_status: "NOT_RUN".to_owned(),
        },
    )?;

    let mut decoder = None;
    let mut decoder_seconds = 0.0;
    let terminal = if audit_tangent_passed {
        let decoder_started = Instant::now();
        *audit_targets_materialized = true;
        let audit_documents = materialize_documents(
            &corpus_path,
            &tokenizer,
            &partition.construction_fit[LOCALIZATION_FIT_DOCUMENTS..],
            true,
        )?;
        validate_localization_materialized_targets(
            &audit_documents,
            &target_envelope,
            &corpus_path,
            partition,
        )?;
        let lm = run_arm(
            &oracle,
            &tokenizer,
            &audit_documents,
            ArmSpec::Localized {
                parameters: checkpoint
                    .checkpoint
                    .lorentz_parameters
                    .as_ref()
                    .ok_or("checkpoint omitted admitted Lorentz parameters")?,
                metric: HelmDLearnedManifoldMetric::Lorentz,
                value_readout: HelmDLearnedManifoldValueReadout::NormalizedLorentzCentroid,
            },
            "localization_l_m",
        )?;
        let lt = run_arm(
            &oracle,
            &tokenizer,
            &audit_documents,
            ArmSpec::Localized {
                parameters: checkpoint
                    .checkpoint
                    .lorentz_parameters
                    .as_ref()
                    .ok_or("checkpoint omitted admitted Lorentz parameters")?,
                metric: HelmDLearnedManifoldMetric::Lorentz,
                value_readout: HelmDLearnedManifoldValueReadout::TransportedTangentArithmeticSum,
            },
            "localization_l_t",
        )?;
        let exact_causal_work = audit_is_exact(&lm.report) && audit_is_exact(&lt.report);
        let replay_identical = lm.report.replay_identical && lt.report.replay_identical;
        if !exact_causal_work || !replay_identical {
            return Err("localization paired decoder causal work or replay failed".into());
        }
        let nll_delta = lt.report.mean_next_token_nll - lm.report.mean_next_token_nll;
        let decoder_passed = nll_delta <= -0.05;
        decoder = Some(LocalizationDecoderComparison {
            lorentz_normalized: lm.report,
            lorentz_tangent: lt.report,
            tangent_minus_normalized_nll: nll_delta,
            tangent_nll_improves_by_at_least_005: decoder_passed,
            exact_causal_work,
            replay_identical,
        });
        decoder_seconds = decoder_started.elapsed().as_secs_f64();
        localization_terminal_for(
            true,
            Some(true),
            Some(euclidean_score_passed),
            Some(decoder_passed),
        )
    } else {
        localization_terminal_for(true, Some(false), Some(euclidean_score_passed), None)
    };
    let execution_snapshot = oracle.execution_snapshot();
    let forward_ledger = localization_forward_ledger(
        execution_baseline,
        execution_snapshot,
        512,
        if decoder.is_some() { 512 } else { 0 },
    )?;
    let mut payload = LocalizationResultPayload {
        schema: LOCALIZATION_RESULT_SCHEMA.to_owned(),
        issue: 973,
        terminal: terminal.to_owned(),
        partition_cid: partition.partition_cid.clone(),
        checkpoint_cid: checkpoint.checkpoint_cid,
        target_commitment_artifact_cid: target_envelope.artifact_cid,
        implementation,
        preflight,
        fit_population_attention: Some(fit_population_attention),
        audit_population_attention: Some(audit_population_attention),
        decoder,
        forward_ledger,
        execution_snapshot,
        timing: LocalizationTiming {
            preflight_trace_seconds,
            preflight_evaluation_seconds,
            remaining_trace_seconds,
            fit_and_replay_seconds,
            attention_evaluation_seconds,
            decoder_seconds,
            total_seconds: total_started.elapsed().as_secs_f64(),
        },
        audit_targets_materialized_after_checkpoint: *audit_targets_materialized,
        v2_validation_status: "NOT_OPENED_OR_RESCORED".to_owned(),
        d3_status: "NOT_RUN".to_owned(),
        result_cid: String::new(),
    };
    payload.result_cid = result_payload_cid(&payload)?;
    let result_cid = write_result(result_path, &payload)?;
    eprintln!(
        "HELM-D score-centroid localization terminal={} result_cid={result_cid}",
        payload.terminal
    );
    Ok(())
}
