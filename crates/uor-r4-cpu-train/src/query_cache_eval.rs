//! D84 evaluator for immutable teacher-forced FBS query caches.
//!
//! The GPU scorer captures exact Q12 queries and decisions once. This module
//! verifies that evidence, streams TRAIN into candidate and matched-control
//! output tables, and evaluates both against frozen panel queries on CPU.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{invalid, CpuTrainError};

type Result<T> = std::result::Result<T, CpuTrainError>;

pub const QUERY_CACHE_SCHEMA: &str = "uor-r4.fbs-query-cache/1";
const QUERY_CACHE_SEAL_SCHEMA: &str = "uor-r4.fbs-query-cache-seal/1";
const EVALUATION_SCHEMA: &str = "uor-r4.cpu-query-cache-evaluation/1";
const ROW_MAGIC: &[u8; 8] = b"UORQROW1";
const Q12_SCALE: f32 = 4096.0;
const PINNED_TOKENIZER_SHA256: &str =
    "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89";
const QUERY_BLOCK_ROWS: usize = 64;

pub const REGISTERED_MINIMUM_OBSERVATIONS: usize = 8;
pub const REGISTERED_MIN_FITTED_ROWS: usize = 100;
pub const REGISTERED_MIN_FIT_OBSERVATIONS: usize = 10_000;
pub const REGISTERED_EXPECTED_ROWS: usize = 1_460_697;
pub const REGISTERED_TRAIN_ROWS: usize = 27_416_554;
pub const REGISTERED_WIDTH: usize = 256;
pub const REGISTERED_CONTEXT: usize = 192;
pub const REGISTERED_TRAIN_SPLIT: &str = "train";
pub const REGISTERED_PANEL_SPLITS: [&str; 4] = ["oasst2", "wiki-00000", "wiki-00020", "wiki-00040"];
pub const REGISTERED_TRAIN_TOKENS_SHA256: &str =
    "ed8f25374e99bd65cd509b5cfcdffb6a8476401a973ea02aaaaa0ea76e35ee52";
pub const REGISTERED_TRAIN_MASK_SHA256: &str =
    "ec6e520c7a67aaf6efc7aaf42bf7513b3e697239fb18b73a1e89f130eacd4875";
pub const REGISTERED_POOLED_ROW_IDENTITY: &str =
    "5f88a5b3295ba5d684a8c75206787642fd65a2377d2dbb268992ec5c71a621eb";
const REGISTERED_PANEL_INPUTS: [(&str, &str, &str); 4] = [
    (
        "oasst2",
        "02a04e97bbb8936c993bf5eff0f29c171ebeae88988217a4100fdc77701f169e",
        "9c5bb6567c6a4cded2dfb2280ae4acdc798e283a3beae8aaebc695656e1b4833",
    ),
    (
        "wiki-00000",
        "82b3a40c3ffe2cabd9164f636d9c2710fb054e171913eca944ebf819b70209f7",
        "34fe6b68e0d039b914bb1cde789223ef463b37ea77af7f3d77ab90eb0977386f",
    ),
    (
        "wiki-00020",
        "9b7f1432b105f161ed8b54b01900d8fa791efc6dbfd5ad1d62a8d4d94a86d640",
        "a6803ab3de3fae60c382416639ffa583c26ac822db78a37afd2c2d718767e00d",
    ),
    (
        "wiki-00040",
        "c927243b540bccdda8af8fb5a11aafa8b9041d4af30dd614f23d2ebb9aa5962d",
        "9d95ea6de0eba07d4cdbc258b5fa6ae88def952f062d166ba417d783724fa625",
    ),
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryCacheManifest {
    pub schema: String,
    pub checkpoint: String,
    pub data_root: String,
    pub tokenizer_sha256: String,
    pub checkpoint_files: BTreeMap<String, String>,
    pub run_config: Option<String>,
    pub run_config_sha256: Option<String>,
    pub table_file: String,
    pub table_sha256: String,
    pub vocab: usize,
    pub width: usize,
    pub query_space: String,
    pub splits: Vec<QueryCacheSplit>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryCacheSplit {
    pub split: String,
    pub label: String,
    pub memory: bool,
    pub context: usize,
    pub rows: usize,
    pub tokens_sha256: String,
    pub mask_sha256: String,
    pub query_file: String,
    pub query_sha256: String,
    pub rows_file: String,
    pub rows_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct QueryCacheSeal {
    schema: String,
    manifest_sha256: String,
}

#[derive(Clone, Debug)]
struct CachedRows {
    targets: Vec<u32>,
    positions: Vec<u32>,
    nearest: Vec<u32>,
    writer: Vec<u32>,
    opens: Vec<u8>,
    copies: Vec<u8>,
}

#[derive(Debug)]
struct QueryCache {
    root: PathBuf,
    manifest: QueryCacheManifest,
    manifest_sha256: String,
    table: Vec<i16>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ReadoutCounts {
    pub rows: usize,
    pub top1_correct: usize,
    pub top8_correct: usize,
    pub changed_from_frozen: usize,
    pub decision_sha256: String,
}

#[derive(Clone, Debug)]
struct Decisions {
    counts: ReadoutCounts,
    top1_rows: Vec<bool>,
    top8_rows: Vec<bool>,
    tokens: Vec<u32>,
}

#[derive(Clone, Debug)]
struct FittedTables {
    centroid: Vec<i16>,
    shuffled: Vec<i16>,
    fitted_rows: usize,
    observations: usize,
    control_strata: usize,
    singleton_control_targets: usize,
    changed_coordinates: u64,
    displacement_l1: u64,
    shuffled_changed_targets: usize,
    shuffled_changed_coordinates: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct CentroidEvaluationRequest {
    pub seed: Option<u8>,
    pub train_split: String,
    pub panel_splits: Vec<String>,
    pub memory: bool,
    pub minimum_observations: usize,
    pub minimum_fitted_rows: usize,
    pub minimum_fit_observations: usize,
    pub expected_train_tokens_sha256: Option<String>,
    pub expected_train_mask_sha256: Option<String>,
    pub expected_rows: Option<usize>,
    pub expected_row_identity: Option<String>,
    pub checkpoint_root: Option<PathBuf>,
}

impl QueryCache {
    fn open(root: &Path) -> Result<Self> {
        let root = fs::canonicalize(root)
            .map_err(|error| invalid(format!("{}: {error}", root.display())))?;
        let manifest_bytes = fs::read(root.join("manifest.json"))
            .map_err(|error| invalid(format!("query cache manifest: {error}")))?;
        let seal_bytes = fs::read(root.join("seal.json"))
            .map_err(|error| invalid(format!("query cache seal: {error}")))?;
        let seal: QueryCacheSeal = serde_json::from_slice(&seal_bytes)
            .map_err(|error| invalid(format!("query cache seal: {error}")))?;
        let manifest_sha256 = sha256(&manifest_bytes);
        if seal.schema != QUERY_CACHE_SEAL_SCHEMA || seal.manifest_sha256 != manifest_sha256 {
            return Err(invalid("query cache manifest seal differs"));
        }
        let manifest: QueryCacheManifest = serde_json::from_slice(&manifest_bytes)
            .map_err(|error| invalid(format!("query cache manifest: {error}")))?;
        if manifest.schema != QUERY_CACHE_SCHEMA
            || manifest.query_space != "q12-f32"
            || manifest.tokenizer_sha256 != PINNED_TOKENIZER_SHA256
            || manifest.checkpoint.is_empty()
            || manifest.data_root.is_empty()
            || manifest.table_file.is_empty()
            || manifest.vocab == 0
            || manifest.width == 0
            || manifest.width > i32::MAX as usize / u16::MAX as usize
            || manifest.splits.is_empty()
        {
            return Err(invalid("query cache manifest identity differs"));
        }
        safe_relative(&manifest.table_file)?;
        let mut split_keys = BTreeSet::new();
        for split in &manifest.splits {
            safe_relative(&split.query_file)?;
            safe_relative(&split.rows_file)?;
            if split.context == 0
                || split.rows == 0
                || !split_keys.insert((&split.split, split.memory))
            {
                return Err(invalid("query cache split inventory differs"));
            }
        }
        let table_bytes = read_checked(&root, &manifest.table_file, &manifest.table_sha256)?;
        let table_f32 = decode_f32(&table_bytes)?;
        if table_f32.len() != manifest.vocab.saturating_mul(manifest.width) {
            return Err(invalid("query cache table shape differs"));
        }
        let table = table_f32
            .into_iter()
            .map(f32_to_q12)
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            root,
            manifest,
            manifest_sha256,
            table,
        })
    }

    fn split(&self, name: &str, memory: bool) -> Result<(&QueryCacheSplit, CachedRows)> {
        let metadata = self
            .manifest
            .splits
            .iter()
            .find(|split| split.split == name && split.memory == memory)
            .ok_or_else(|| {
                invalid(format!(
                    "query cache has no {name} split with memory={memory}"
                ))
            })?;
        let row_bytes = read_checked(&self.root, &metadata.rows_file, &metadata.rows_sha256)?;
        let rows = decode_cached_rows(&row_bytes, metadata.rows, self.manifest.vocab)?;
        Ok((metadata, rows))
    }

    fn verify_manifest_files(
        &self,
        selected: &BTreeSet<(String, bool)>,
        checkpoint_root: Option<&Path>,
    ) -> Result<()> {
        for split in &self.manifest.splits {
            if selected.contains(&(split.split.clone(), split.memory)) {
                continue;
            }
            verify_file_checked(&self.root.join(&split.query_file), &split.query_sha256)?;
            verify_file_checked(&self.root.join(&split.rows_file), &split.rows_sha256)?;
        }

        let checkpoint_root = checkpoint_root
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from(&self.manifest.checkpoint));
        if self.manifest.checkpoint_files.is_empty() {
            return Err(invalid("query cache checkpoint inventory is empty"));
        }
        for (file, expected) in &self.manifest.checkpoint_files {
            safe_relative(file)?;
            verify_file_checked(&checkpoint_root.join(file), expected)?;
        }
        match (
            self.manifest.run_config.as_deref(),
            self.manifest.run_config_sha256.as_deref(),
        ) {
            (None, None) => {}
            (Some(original), Some(expected)) => {
                let original = Path::new(original);
                let candidates = [
                    Some(original.to_path_buf()),
                    Some(checkpoint_root.join("config.json")),
                    checkpoint_root
                        .parent()
                        .and_then(Path::parent)
                        .map(|run| run.join("config.json")),
                ];
                let mut matched = false;
                for candidate in candidates.into_iter().flatten() {
                    if candidate.is_file()
                        && file_sha256(&candidate).is_ok_and(|actual| actual == expected)
                    {
                        matched = true;
                        break;
                    }
                }
                if !matched {
                    return Err(invalid("query cache run config hash differs"));
                }
            }
            _ => return Err(invalid("query cache run config inventory differs")),
        }
        Ok(())
    }
}

pub fn evaluate_query_cache(
    root: &Path,
    request: &CentroidEvaluationRequest,
) -> Result<serde_json::Value> {
    evaluate_query_cache_inner(root, request, false)
}

pub fn evaluate_registered_query_cache(
    root: &Path,
    request: &CentroidEvaluationRequest,
) -> Result<serde_json::Value> {
    validate_registered_request(request)?;
    evaluate_query_cache_inner(root, request, true)
}

fn evaluate_query_cache_inner(
    root: &Path,
    request: &CentroidEvaluationRequest,
    registered: bool,
) -> Result<serde_json::Value> {
    validate_request(request)?;
    let cache = QueryCache::open(root)?;
    if registered {
        validate_registered_cache(&cache, request)?;
    }
    let selected = request
        .panel_splits
        .iter()
        .chain(std::iter::once(&request.train_split))
        .map(|split| (split.clone(), request.memory))
        .collect::<BTreeSet<_>>();
    cache.verify_manifest_files(&selected, request.checkpoint_root.as_deref())?;
    let (train_metadata, train_rows) = cache.split(&request.train_split, request.memory)?;
    let train_identity_matches = request
        .expected_train_tokens_sha256
        .as_ref()
        .is_none_or(|expected| expected == &train_metadata.tokens_sha256)
        && request
            .expected_train_mask_sha256
            .as_ref()
            .is_none_or(|expected| expected == &train_metadata.mask_sha256);

    let started = std::time::Instant::now();
    let fitted = fit_streamed_tables(&cache, train_metadata, &train_rows, request)?;
    let power_ok = fitted.fitted_rows >= request.minimum_fitted_rows
        && fitted.observations >= request.minimum_fit_observations;
    let shuffle_ok = fitted.shuffled_changed_targets > 0 && fitted.shuffled_changed_coordinates > 0;

    let mut panels = Vec::new();
    let mut identities = Vec::new();
    let mut original_top1 = Vec::new();
    let mut centroid_top1 = Vec::new();
    let mut shuffled_top1 = Vec::new();
    let mut original_top8 = Vec::new();
    let mut centroid_top8 = Vec::new();
    let mut shuffled_top8 = Vec::new();
    let mut pooled_original_tokens = Vec::new();
    let mut pooled_centroid_tokens = Vec::new();
    let mut pooled_shuffled_tokens = Vec::new();
    let mut identity_exact = true;

    for name in &request.panel_splits {
        let (metadata, rows) = cache.split(name, request.memory)?;
        let queries = load_q12_queries(&cache.root, metadata, cache.manifest.width)?;
        let identity = row_identity(&rows);
        let original = evaluate_table(
            &queries,
            &rows.targets,
            &rows,
            &cache.table,
            cache.manifest.vocab,
            cache.manifest.width,
        )?;
        let centroid = evaluate_table(
            &queries,
            &rows.targets,
            &rows,
            &fitted.centroid,
            cache.manifest.vocab,
            cache.manifest.width,
        )?;
        let shuffled = evaluate_table(
            &queries,
            &rows.targets,
            &rows,
            &fitted.shuffled,
            cache.manifest.vocab,
            cache.manifest.width,
        )?;
        let panel_identity_exact = original
            .tokens
            .iter()
            .zip(&rows.nearest)
            .all(|(left, right)| left == right);
        identity_exact &= panel_identity_exact;
        original_top1.extend_from_slice(&original.top1_rows);
        centroid_top1.extend_from_slice(&centroid.top1_rows);
        shuffled_top1.extend_from_slice(&shuffled.top1_rows);
        original_top8.extend_from_slice(&original.top8_rows);
        centroid_top8.extend_from_slice(&centroid.top8_rows);
        shuffled_top8.extend_from_slice(&shuffled.top8_rows);
        pooled_original_tokens.extend_from_slice(&original.tokens);
        pooled_centroid_tokens.extend_from_slice(&centroid.tokens);
        pooled_shuffled_tokens.extend_from_slice(&shuffled.tokens);
        identities.push((name.clone(), identity));
        panels.push(serde_json::json!({
            "split": name,
            "memory": metadata.memory,
            "context": metadata.context,
            "rows": rows.targets.len(),
            "tokens_sha256": &metadata.tokens_sha256,
            "mask_sha256": &metadata.mask_sha256,
            "row_identity_sha256": hex(&identities.last().expect("panel identity").1),
            "opening_rows": rows.opens.iter().filter(|&&flag| flag == 1).count(),
            "copy_rows": rows.copies.iter().filter(|&&flag| flag == 1).count(),
            "writer_differs_from_nearest": rows.writer.iter().zip(&rows.nearest).filter(|(left, right)| left != right).count(),
            "identity": {
                "exact": panel_identity_exact,
                "decision_sha256": original.counts.decision_sha256,
            },
            "original": original.counts,
            "centroid": centroid.counts,
            "exact_count_shuffled_target": shuffled.counts,
            "centroid_vs_original": paired(&centroid.top1_rows, &original.top1_rows),
            "centroid_vs_exact_count_shuffled": paired(&centroid.top1_rows, &shuffled.top1_rows),
            "centroid_top8_not_below_original": centroid.counts.top8_correct >= original.counts.top8_correct,
        }));
    }

    let pooled_identity = pooled_identity(&identities);
    let identity_registered = request
        .expected_row_identity
        .as_ref()
        .is_none_or(|expected| expected == &pooled_identity);
    let rows_registered = request
        .expected_rows
        .is_none_or(|expected| expected == pooled_original_tokens.len());
    let paired_original = paired(&centroid_top1, &original_top1);
    let paired_shuffled = paired(&centroid_top1, &shuffled_top1);
    let centroid_top1_correct = centroid_top1.iter().filter(|&&hit| hit).count();
    let original_top1_correct = original_top1.iter().filter(|&&hit| hit).count();
    let shuffled_top1_correct = shuffled_top1.iter().filter(|&&hit| hit).count();
    let original_top8_correct = original_top8.iter().filter(|&&hit| hit).count();
    let centroid_top8_correct = centroid_top8.iter().filter(|&&hit| hit).count();
    let shuffled_top8_correct = shuffled_top8.iter().filter(|&&hit| hit).count();
    let top8_holds = centroid_top8_correct >= original_top8_correct;
    let bar_passes = identity_exact
        && train_identity_matches
        && identity_registered
        && rows_registered
        && power_ok
        && shuffle_ok
        && top8_holds
        && centroid_top1_correct > original_top1_correct
        && centroid_top1_correct > shuffled_top1_correct
        && paired_original["passes"].as_bool() == Some(true)
        && paired_shuffled["passes"].as_bool() == Some(true);
    let status = if !identity_exact
        || !train_identity_matches
        || !identity_registered
        || !rows_registered
        || !shuffle_ok
    {
        "CONTROL_FAULT"
    } else if !power_ok {
        "INSUFFICIENT_POWER"
    } else if bar_passes {
        "KEEP"
    } else {
        "REJECT"
    };

    Ok(serde_json::json!({
        "schema": EVALUATION_SCHEMA,
        "evaluation_profile": if registered {
            "n3s-registered-v1"
        } else {
            "diagnostic"
        },
        "status": status,
        "cache": {
            "root": cache.root,
            "manifest_sha256": cache.manifest_sha256,
            "checkpoint": cache.manifest.checkpoint,
            "checkpoint_files": &cache.manifest.checkpoint_files,
            "run_config": &cache.manifest.run_config,
            "run_config_sha256": &cache.manifest.run_config_sha256,
            "data_root": &cache.manifest.data_root,
            "tokenizer_sha256": &cache.manifest.tokenizer_sha256,
            "table_sha256": &cache.manifest.table_sha256,
            "vocab": cache.manifest.vocab,
            "width": cache.manifest.width,
            "query_space": &cache.manifest.query_space,
            "splits": &cache.manifest.splits,
        },
        "request": request,
        "train": {
            "split": &request.train_split,
            "memory": request.memory,
            "rows": train_rows.targets.len(),
            "context": train_metadata.context,
            "tokens_sha256": &train_metadata.tokens_sha256,
            "mask_sha256": &train_metadata.mask_sha256,
            "expected_tokens_sha256": request.expected_train_tokens_sha256,
            "expected_mask_sha256": request.expected_train_mask_sha256,
            "identity_matches": train_identity_matches,
        },
        "workers": rayon::current_num_threads(),
        "wall_ms": started.elapsed().as_millis(),
        "fit": {
            "fitted_rows": fitted.fitted_rows,
            "observations": fitted.observations,
            "minimum_fitted_rows": request.minimum_fitted_rows,
            "minimum_fit_observations": request.minimum_fit_observations,
            "power_ok": power_ok,
            "control_strategy": "exact-count-stratified-cycle",
            "control_strata": fitted.control_strata,
            "singleton_control_targets": fitted.singleton_control_targets,
            "changed_coordinates": fitted.changed_coordinates,
            "displacement_l1": fitted.displacement_l1,
            "shuffled_changed_targets": fitted.shuffled_changed_targets,
            "shuffled_changed_coordinates": fitted.shuffled_changed_coordinates,
        },
        "pooled": {
            "rows": pooled_original_tokens.len(),
            "row_identity_sha256": pooled_identity,
            "expected_rows": request.expected_rows,
            "expected_row_identity": request.expected_row_identity,
            "rows_match": rows_registered,
            "identity_matches": identity_registered,
            "original_top1": original_top1_correct,
            "centroid_top1": centroid_top1_correct,
            "exact_count_shuffled_top1": shuffled_top1_correct,
            "original_top8": original_top8_correct,
            "centroid_top8": centroid_top8_correct,
            "exact_count_shuffled_top8": shuffled_top8_correct,
            "top8_holds": top8_holds,
            "centroid_vs_original": paired_original,
            "centroid_vs_exact_count_shuffled": paired_shuffled,
            "decision_sha256": sha256(
                &pooled_original_tokens
                    .iter()
                    .chain(&pooled_centroid_tokens)
                    .chain(&pooled_shuffled_tokens)
                    .flat_map(|token| token.to_le_bytes())
                    .collect::<Vec<_>>(),
            ),
            "bar_passes": bar_passes,
        },
        "panels": panels,
    }))
}

pub fn aggregate_query_cache_reports(reports: &[serde_json::Value]) -> Result<serde_json::Value> {
    aggregate_query_cache_reports_with_identity(reports, REGISTERED_POOLED_ROW_IDENTITY)
}

fn aggregate_query_cache_reports_with_identity(
    reports: &[serde_json::Value],
    expected_pooled_identity: &str,
) -> Result<serde_json::Value> {
    if reports.len() != 2 {
        return Err(invalid(
            "query cache aggregate needs exactly two seed reports",
        ));
    }
    let mut manifests = BTreeSet::new();
    let mut seeds = Vec::<(u8, serde_json::Value)>::new();
    let mut seed_ids = BTreeSet::new();
    let mut statuses = Vec::new();
    let mut binding = None;
    for report in reports {
        if report["schema"] != EVALUATION_SCHEMA {
            return Err(invalid("query cache aggregate report schema differs"));
        }
        if report["evaluation_profile"] != "n3s-registered-v1" {
            return Err(invalid("query cache aggregate report is not registered"));
        }
        let status = report["status"]
            .as_str()
            .ok_or_else(|| invalid("query cache aggregate report has no status"))?;
        if !matches!(
            status,
            "KEEP" | "REJECT" | "INSUFFICIENT_POWER" | "CONTROL_FAULT"
        ) {
            return Err(invalid("query cache aggregate report status differs"));
        }
        let manifest = report["cache"]["manifest_sha256"]
            .as_str()
            .ok_or_else(|| invalid("query cache aggregate manifest identity is absent"))?;
        if !is_sha256(manifest) {
            return Err(invalid("query cache aggregate manifest identity differs"));
        }
        if !manifests.insert(manifest.to_owned()) {
            return Err(invalid("query cache aggregate repeats a cache"));
        }
        if report["pooled"]["bar_passes"].as_bool() != Some(status == "KEEP") {
            return Err(invalid("query cache aggregate status and bar differ"));
        }
        let request = &report["request"];
        let seed = request["seed"]
            .as_u64()
            .filter(|seed| matches!(seed, 1 | 2))
            .ok_or_else(|| invalid("query cache aggregate seed is absent or invalid"))?
            as u8;
        if !seed_ids.insert(seed) {
            return Err(invalid("query cache aggregate repeats a seed role"));
        }
        let panels = report["panels"]
            .as_array()
            .ok_or_else(|| invalid("query cache aggregate panels are absent"))?;
        let panel_names = panels
            .iter()
            .map(|panel| {
                panel["split"]
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| invalid("query cache aggregate panel name is absent"))
            })
            .collect::<Result<Vec<_>>>()?;
        let registered_panels = REGISTERED_PANEL_SPLITS
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if panel_names != registered_panels {
            return Err(invalid("query cache aggregate panel inventory differs"));
        }
        if request["panel_splits"].as_array().is_none_or(|splits| {
            !splits
                .iter()
                .map(serde_json::Value::as_str)
                .eq(REGISTERED_PANEL_SPLITS.into_iter().map(Some))
        }) {
            return Err(invalid("query cache aggregate requested panels differ"));
        }
        let rows = request["expected_rows"]
            .as_u64()
            .ok_or_else(|| invalid("query cache aggregate expected rows are absent"))?;
        let identity = request["expected_row_identity"]
            .as_str()
            .ok_or_else(|| invalid("query cache aggregate expected identity is absent"))?
            .to_owned();
        let train_tokens = request["expected_train_tokens_sha256"]
            .as_str()
            .ok_or_else(|| invalid("query cache aggregate TRAIN token pin is absent"))?
            .to_owned();
        let train_mask = request["expected_train_mask_sha256"]
            .as_str()
            .ok_or_else(|| invalid("query cache aggregate TRAIN mask pin is absent"))?
            .to_owned();
        let train_split = request["train_split"]
            .as_str()
            .ok_or_else(|| invalid("query cache aggregate TRAIN split is absent"))?;
        let minimum_observations = request["minimum_observations"]
            .as_u64()
            .ok_or_else(|| invalid("query cache aggregate minimum observations are absent"))?;
        let minimum_fitted_rows = request["minimum_fitted_rows"]
            .as_u64()
            .ok_or_else(|| invalid("query cache aggregate minimum fitted rows are absent"))?;
        let minimum_fit_observations = request["minimum_fit_observations"]
            .as_u64()
            .ok_or_else(|| invalid("query cache aggregate minimum fit observations are absent"))?;
        if rows != REGISTERED_EXPECTED_ROWS as u64
            || identity != expected_pooled_identity
            || train_tokens != REGISTERED_TRAIN_TOKENS_SHA256
            || train_mask != REGISTERED_TRAIN_MASK_SHA256
            || train_split != REGISTERED_TRAIN_SPLIT
            || request["memory"].as_bool() != Some(false)
            || minimum_observations != REGISTERED_MINIMUM_OBSERVATIONS as u64
            || minimum_fitted_rows != REGISTERED_MIN_FITTED_ROWS as u64
            || minimum_fit_observations != REGISTERED_MIN_FIT_OBSERVATIONS as u64
            || report["fit"]["minimum_fitted_rows"] != REGISTERED_MIN_FITTED_ROWS
            || report["fit"]["minimum_fit_observations"] != REGISTERED_MIN_FIT_OBSERVATIONS
            || report["fit"]["control_strategy"] != "exact-count-stratified-cycle"
            || report["cache"]["width"] != REGISTERED_WIDTH
            || report["cache"]["query_space"] != "q12-f32"
            || report["cache"]["tokenizer_sha256"] != PINNED_TOKENIZER_SHA256
        {
            return Err(invalid(
                "query cache aggregate report differs from the registered evaluation",
            ));
        }
        let cache = &report["cache"];
        let cache_splits = cache["splits"]
            .as_array()
            .ok_or_else(|| invalid("query cache aggregate cache inventory is absent"))?;
        if cache_splits.len() != 1 + REGISTERED_PANEL_SPLITS.len() {
            return Err(invalid("query cache aggregate cache inventory differs"));
        }
        if cache_splits.iter().any(|split| {
            split["query_file"].as_str().is_none_or(str::is_empty)
                || split["rows_file"].as_str().is_none_or(str::is_empty)
                || split["query_sha256"]
                    .as_str()
                    .is_none_or(|hash| !is_sha256(hash))
                || split["rows_sha256"]
                    .as_str()
                    .is_none_or(|hash| !is_sha256(hash))
                || split["tokens_sha256"]
                    .as_str()
                    .is_none_or(|hash| !is_sha256(hash))
                || split["mask_sha256"]
                    .as_str()
                    .is_none_or(|hash| !is_sha256(hash))
        }) {
            return Err(invalid("query cache aggregate split pins differ"));
        }
        let checkpoint_files = cache["checkpoint_files"]
            .as_object()
            .filter(|files| !files.is_empty())
            .ok_or_else(|| invalid("query cache aggregate checkpoint inventory is absent"))?;
        if checkpoint_files
            .values()
            .any(|hash| hash.as_str().is_none_or(|hash| !is_sha256(hash)))
            || cache["table_sha256"]
                .as_str()
                .is_none_or(|hash| !is_sha256(hash))
            || cache["run_config"].as_str().is_none()
            || cache["run_config_sha256"]
                .as_str()
                .is_none_or(|hash| !is_sha256(hash))
        {
            return Err(invalid("query cache aggregate cache pins differ"));
        }
        let train_cache = registered_cache_split(cache_splits, REGISTERED_TRAIN_SPLIT)?;
        if train_cache["tokens_sha256"] != REGISTERED_TRAIN_TOKENS_SHA256
            || train_cache["mask_sha256"] != REGISTERED_TRAIN_MASK_SHA256
            || train_cache["context"] != REGISTERED_CONTEXT
            || train_cache["rows"] != REGISTERED_TRAIN_ROWS
        {
            return Err(invalid("query cache aggregate TRAIN inventory differs"));
        }
        for (name, tokens, mask) in REGISTERED_PANEL_INPUTS {
            let split = registered_cache_split(cache_splits, name)?;
            if split["tokens_sha256"] != tokens
                || split["mask_sha256"] != mask
                || split["context"] != REGISTERED_CONTEXT
            {
                return Err(invalid(format!(
                    "query cache aggregate {name} inventory differs"
                )));
            }
        }

        let train = &report["train"];
        let train_checks = [
            train["split"].as_str() == Some(REGISTERED_TRAIN_SPLIT),
            train["memory"].as_bool() == Some(false),
            train["rows"].as_u64() == Some(REGISTERED_TRAIN_ROWS as u64),
            train["context"].as_u64() == Some(REGISTERED_CONTEXT as u64),
            train["tokens_sha256"].as_str() == Some(REGISTERED_TRAIN_TOKENS_SHA256),
            train["mask_sha256"].as_str() == Some(REGISTERED_TRAIN_MASK_SHA256),
            train["expected_tokens_sha256"].as_str() == Some(REGISTERED_TRAIN_TOKENS_SHA256),
            train["expected_mask_sha256"].as_str() == Some(REGISTERED_TRAIN_MASK_SHA256),
        ];
        let train_identity = train_checks.into_iter().all(|check| check);
        if train["identity_matches"].as_bool() != Some(train_identity) {
            return Err(invalid("query cache aggregate TRAIN identity differs"));
        }

        let mut panel_rows = 0u64;
        let mut cache_panel_rows = 0u64;
        let mut panel_identity_exact = true;
        let mut original_top1 = 0u64;
        let mut centroid_top1 = 0u64;
        let mut shuffled_top1 = 0u64;
        let mut original_top8 = 0u64;
        let mut centroid_top8 = 0u64;
        let mut shuffled_top8 = 0u64;
        let mut original_candidate_only = 0u64;
        let mut original_control_only = 0u64;
        let mut shuffled_candidate_only = 0u64;
        let mut shuffled_control_only = 0u64;
        let mut panel_identities = Vec::with_capacity(REGISTERED_PANEL_SPLITS.len());
        for (panel, (name, tokens, mask)) in panels.iter().zip(REGISTERED_PANEL_INPUTS) {
            let split = registered_cache_split(cache_splits, name)?;
            if panel["tokens_sha256"] != tokens
                || panel["mask_sha256"] != mask
                || panel["context"] != REGISTERED_CONTEXT
                || panel["memory"] != false
                || panel["tokens_sha256"] != split["tokens_sha256"]
                || panel["mask_sha256"] != split["mask_sha256"]
                || panel["context"] != split["context"]
                || panel["memory"] != split["memory"]
            {
                return Err(invalid(format!(
                    "query cache aggregate {name} panel inventory differs"
                )));
            }
            let rows = panel["rows"]
                .as_u64()
                .ok_or_else(|| invalid("query cache aggregate panel rows are absent"))?;
            if split["rows"] != rows {
                return Err(invalid(format!(
                    "query cache aggregate {name} cache rows differ"
                )));
            }
            if rows == 0 {
                return Err(invalid("query cache aggregate panel is empty"));
            }
            let identity = panel["row_identity_sha256"]
                .as_str()
                .and_then(parse_sha256)
                .ok_or_else(|| invalid("query cache aggregate panel identity differs"))?;
            let decision_hashes = [
                &panel["identity"]["decision_sha256"],
                &panel["original"]["decision_sha256"],
                &panel["centroid"]["decision_sha256"],
                &panel["exact_count_shuffled_target"]["decision_sha256"],
            ];
            if decision_hashes
                .into_iter()
                .any(|hash| hash.as_str().is_none_or(|hash| !is_sha256(hash)))
                || panel["identity"]["decision_sha256"] != panel["original"]["decision_sha256"]
            {
                return Err(invalid("query cache aggregate panel pins differ"));
            }
            let changed = panel["original"]["changed_from_frozen"]
                .as_u64()
                .filter(|changed| *changed <= rows)
                .ok_or_else(|| invalid("query cache aggregate panel changed count differs"))?;
            let identity_exact = panel["identity"]["exact"].as_bool() == Some(true);
            if identity_exact != (changed == 0) {
                return Err(invalid("query cache aggregate panel exact flag differs"));
            }
            panel_rows = panel_rows
                .checked_add(rows)
                .ok_or_else(|| invalid("query cache aggregate panel rows overflow"))?;
            cache_panel_rows = cache_panel_rows
                .checked_add(
                    split["rows"]
                        .as_u64()
                        .ok_or_else(|| invalid("query cache aggregate cache rows are absent"))?,
                )
                .ok_or_else(|| invalid("query cache aggregate cache rows overflow"))?;
            panel_identity_exact &= identity_exact;
            panel_identities.push((name.to_owned(), identity));
            let mut panel_original_top1 = 0u64;
            let mut panel_centroid_top1 = 0u64;
            let mut panel_shuffled_top1 = 0u64;
            let mut panel_original_top8 = 0u64;
            let mut panel_centroid_top8 = 0u64;
            for (arm, value) in [
                ("original", &panel["original"]),
                ("centroid", &panel["centroid"]),
                (
                    "exact_count_shuffled_target",
                    &panel["exact_count_shuffled_target"],
                ),
            ] {
                if value["rows"].as_u64() != Some(rows) {
                    return Err(invalid(format!(
                        "query cache aggregate {name} {arm} rows differ"
                    )));
                }
                let top1 = value["top1_correct"]
                    .as_u64()
                    .ok_or_else(|| invalid("query cache aggregate panel top-1 is absent"))?;
                let top8 = value["top8_correct"]
                    .as_u64()
                    .ok_or_else(|| invalid("query cache aggregate panel top-8 is absent"))?;
                if top1 > top8 || top8 > rows {
                    return Err(invalid(format!(
                        "query cache aggregate {name} {arm} counts differ"
                    )));
                }
                if value["changed_from_frozen"]
                    .as_u64()
                    .is_none_or(|changed| changed > rows)
                {
                    return Err(invalid(format!(
                        "query cache aggregate {name} {arm} changed count differs"
                    )));
                }
                match arm {
                    "original" => {
                        original_top1 += top1;
                        original_top8 += top8;
                        panel_original_top1 = top1;
                        panel_original_top8 = top8;
                    }
                    "centroid" => {
                        centroid_top1 += top1;
                        centroid_top8 += top8;
                        panel_centroid_top1 = top1;
                        panel_centroid_top8 = top8;
                    }
                    _ => {
                        shuffled_top1 += top1;
                        shuffled_top8 += top8;
                        panel_shuffled_top1 = top1;
                    }
                }
            }
            let original_pair = reported_pair(&panel["centroid_vs_original"], rows)?;
            let shuffled_pair = reported_pair(&panel["centroid_vs_exact_count_shuffled"], rows)?;
            if original_pair.candidate_sum != panel_centroid_top1
                || original_pair.control_sum != panel_original_top1
                || shuffled_pair.candidate_sum != panel_centroid_top1
                || shuffled_pair.control_sum != panel_shuffled_top1
                || panel["centroid_top8_not_below_original"].as_bool()
                    != Some(panel_centroid_top8 >= panel_original_top8)
            {
                return Err(invalid(format!(
                    "query cache aggregate {name} paired evidence differs"
                )));
            }
            original_candidate_only = original_candidate_only
                .checked_add(original_pair.candidate_only)
                .ok_or_else(|| invalid("query cache aggregate candidate-only count overflows"))?;
            original_control_only = original_control_only
                .checked_add(original_pair.control_only)
                .ok_or_else(|| invalid("query cache aggregate control-only count overflows"))?;
            shuffled_candidate_only = shuffled_candidate_only
                .checked_add(shuffled_pair.candidate_only)
                .ok_or_else(|| invalid("query cache aggregate candidate-only count overflows"))?;
            shuffled_control_only = shuffled_control_only
                .checked_add(shuffled_pair.control_only)
                .ok_or_else(|| invalid("query cache aggregate control-only count overflows"))?;
        }
        let panel_rows_match = panel_rows == rows && cache_panel_rows == rows;
        let computed_pooled_identity = pooled_identity(&panel_identities);
        let pooled = &report["pooled"];
        let fit = &report["fit"];
        if pooled["decision_sha256"]
            .as_str()
            .is_none_or(|hash| !is_sha256(hash))
        {
            return Err(invalid("query cache aggregate pooled decision pin differs"));
        }
        for (name, expected) in [
            ("rows", rows),
            ("original_top1", original_top1),
            ("centroid_top1", centroid_top1),
            ("exact_count_shuffled_top1", shuffled_top1),
            ("original_top8", original_top8),
            ("centroid_top8", centroid_top8),
            ("exact_count_shuffled_top8", shuffled_top8),
        ] {
            if pooled[name].as_u64() != Some(expected) {
                return Err(invalid(format!(
                    "query cache aggregate pooled {name} differs"
                )));
            }
        }
        let pooled_rows_match = panel_rows_match
            && pooled["rows"].as_u64() == Some(rows)
            && pooled["row_identity_sha256"].as_str() == Some(identity.as_str());
        if pooled["row_identity_sha256"].as_str() != Some(computed_pooled_identity.as_str()) {
            return Err(invalid(
                "query cache aggregate pooled identity rollup differs",
            ));
        }
        let pooled_identity_matches = computed_pooled_identity == expected_pooled_identity
            && identity == expected_pooled_identity;
        if pooled["rows_match"].as_bool() != Some(pooled_rows_match)
            || pooled["identity_matches"].as_bool() != Some(pooled_identity_matches)
        {
            return Err(invalid("query cache aggregate pooled identity differs"));
        }
        let fitted_rows = fit["fitted_rows"]
            .as_u64()
            .ok_or_else(|| invalid("query cache aggregate fitted rows are absent"))?;
        let observations = fit["observations"]
            .as_u64()
            .ok_or_else(|| invalid("query cache aggregate observations are absent"))?;
        let power_ok = fitted_rows <= REGISTERED_TRAIN_ROWS as u64
            && observations <= REGISTERED_TRAIN_ROWS as u64
            && observations >= fitted_rows
            && fitted_rows >= REGISTERED_MIN_FITTED_ROWS as u64
            && observations >= REGISTERED_MIN_FIT_OBSERVATIONS as u64;
        let shuffle_ok = fit["shuffled_changed_targets"]
            .as_u64()
            .is_some_and(|changed| changed > 0)
            && fit["shuffled_changed_coordinates"]
                .as_u64()
                .is_some_and(|changed| changed > 0);
        let top8_holds = centroid_top8 >= original_top8;
        if pooled["top8_holds"].as_bool() != Some(top8_holds) {
            return Err(invalid("query cache aggregate top-8 flag differs"));
        }
        let original_pair = reported_pair(&pooled["centroid_vs_original"], rows)?;
        let shuffled_pair = reported_pair(&pooled["centroid_vs_exact_count_shuffled"], rows)?;
        if original_pair.candidate_sum != centroid_top1
            || original_pair.control_sum != original_top1
            || shuffled_pair.candidate_sum != centroid_top1
            || shuffled_pair.control_sum != shuffled_top1
            || original_pair.candidate_only != original_candidate_only
            || original_pair.control_only != original_control_only
            || shuffled_pair.candidate_only != shuffled_candidate_only
            || shuffled_pair.control_only != shuffled_control_only
        {
            return Err(invalid("query cache aggregate paired totals differ"));
        }
        let mechanism_bar = top8_holds
            && centroid_top1 > original_top1
            && centroid_top1 > shuffled_top1
            && original_pair.passes
            && shuffled_pair.passes;
        let bar_passes = train_identity
            && panel_identity_exact
            && panel_rows_match
            && pooled_rows_match
            && pooled_identity_matches
            && power_ok
            && shuffle_ok
            && mechanism_bar;
        let recomputed_status = if !train_identity
            || !panel_identity_exact
            || !panel_rows_match
            || !pooled_rows_match
            || !pooled_identity_matches
            || !shuffle_ok
        {
            "CONTROL_FAULT"
        } else if !power_ok {
            "INSUFFICIENT_POWER"
        } else if bar_passes {
            "KEEP"
        } else {
            "REJECT"
        };
        if fit["power_ok"].as_bool() != Some(power_ok)
            || pooled["bar_passes"].as_bool() != Some(bar_passes)
            || status != recomputed_status
        {
            return Err(invalid(
                "query cache aggregate report contradicts its own metrics",
            ));
        }
        if report["pooled"]["rows"].as_u64() != Some(rows)
            || report["pooled"]["row_identity_sha256"].as_str() != Some(identity.as_str())
        {
            return Err(invalid(
                "query cache aggregate pooled request binding differs",
            ));
        }
        let current = (
            train_split.to_owned(),
            rows,
            identity,
            train_tokens,
            train_mask,
            minimum_observations,
            minimum_fitted_rows,
            minimum_fit_observations,
            panel_names,
        );
        if binding.as_ref().is_some_and(|binding| binding != &current) {
            return Err(invalid("query cache aggregate seed bindings differ"));
        }
        binding = Some(current);
        seeds.push((
            seed,
            serde_json::json!({
                "seed": seed,
                "cache_manifest_sha256": manifest,
                "checkpoint": report["cache"]["checkpoint"],
                "status": status,
                "fit": report["fit"],
                "pooled": report["pooled"],
            }),
        ));
        statuses.push(status);
    }
    if seed_ids != BTreeSet::from([1, 2]) {
        return Err(invalid(
            "query cache aggregate needs one seed-1 and one seed-2 report",
        ));
    }
    seeds.sort_by_key(|(seed, _)| *seed);
    let seeds = seeds
        .into_iter()
        .map(|(_, report)| report)
        .collect::<Vec<_>>();
    let status = if statuses.contains(&"CONTROL_FAULT") {
        "CONTROL_FAULT"
    } else if statuses.contains(&"INSUFFICIENT_POWER") {
        "INSUFFICIENT_POWER"
    } else if statuses.iter().all(|status| *status == "KEEP") {
        "KEEP_ON_DEVELOPMENT_PANEL"
    } else {
        "REJECT"
    };
    let payload = serde_json::json!({
        "schema": "uor-r4.cpu-query-cache-aggregate/1",
        "status": status,
        "seeds": seeds,
    });
    let semantic = serde_json::to_vec(&payload)
        .map_err(|error| invalid(format!("query cache aggregate: {error}")))?;
    Ok(serde_json::json!({
        "schema": payload["schema"],
        "status": status,
        "seeds": payload["seeds"],
        "decision_sha256": sha256(&semantic),
    }))
}

struct ReportedPair {
    candidate_only: u64,
    control_only: u64,
    candidate_sum: u64,
    control_sum: u64,
    passes: bool,
}

fn reported_pair(value: &serde_json::Value, rows: u64) -> Result<ReportedPair> {
    let field = |name: &str| {
        value[name]
            .as_u64()
            .ok_or_else(|| invalid(format!("query cache aggregate paired {name} is absent")))
    };
    let candidate_only = field("candidate_only")?;
    let control_only = field("control_only")?;
    let candidate_sum = field("candidate_sum")?;
    let control_sum = field("control_sum")?;
    let discordant = field("discordant")?;
    let threshold = field("threshold")?;
    let expected_discordant = candidate_only
        .checked_add(control_only)
        .ok_or_else(|| invalid("query cache aggregate paired discordance overflows"))?;
    let expected_threshold = ceil_sqrt(expected_discordant.saturating_mul(4)).max(2);
    let passes = candidate_sum > control_sum && candidate_only >= expected_threshold;
    if candidate_only > candidate_sum
        || control_only > control_sum
        || candidate_sum > rows
        || control_sum > rows
        || candidate_sum.checked_add(control_only) != control_sum.checked_add(candidate_only)
        || expected_discordant > rows
        || discordant != expected_discordant
        || threshold != expected_threshold
        || value["passes"].as_bool() != Some(passes)
    {
        return Err(invalid("query cache aggregate paired metrics differ"));
    }
    Ok(ReportedPair {
        candidate_only,
        control_only,
        candidate_sum,
        control_sum,
        passes,
    })
}

fn registered_cache_split<'a>(
    splits: &'a [serde_json::Value],
    name: &str,
) -> Result<&'a serde_json::Value> {
    let mut matches = splits.iter().filter(|split| split["split"] == name);
    let split = matches
        .next()
        .ok_or_else(|| invalid(format!("query cache aggregate cache has no {name} split")))?;
    if matches.next().is_some() || split["memory"] != false || split["label"] != "no-memory" {
        return Err(invalid(format!(
            "query cache aggregate {name} cache split differs"
        )));
    }
    Ok(split)
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn parse_sha256(value: &str) -> Option<[u8; 32]> {
    if !is_sha256(value) {
        return None;
    }
    let mut bytes = [0u8; 32];
    for (byte, pair) in bytes.iter_mut().zip(value.as_bytes().chunks_exact(2)) {
        let high = (pair[0] as char).to_digit(16)?;
        let low = (pair[1] as char).to_digit(16)?;
        *byte = ((high << 4) | low) as u8;
    }
    Some(bytes)
}

fn validate_request(request: &CentroidEvaluationRequest) -> Result<()> {
    if request.panel_splits.is_empty()
        || request.minimum_observations == 0
        || request.minimum_fitted_rows == 0
        || request.minimum_fit_observations == 0
    {
        return Err(invalid("query cache evaluation request is empty"));
    }
    if request.panel_splits.iter().collect::<BTreeSet<_>>().len() != request.panel_splits.len() {
        return Err(invalid("query cache evaluation repeats a panel"));
    }
    if request
        .panel_splits
        .iter()
        .any(|panel| panel == &request.train_split)
    {
        return Err(invalid("query cache evaluation uses train as a panel"));
    }
    Ok(())
}

fn validate_registered_request(request: &CentroidEvaluationRequest) -> Result<()> {
    if !matches!(request.seed, Some(1 | 2))
        || request.train_split != REGISTERED_TRAIN_SPLIT
        || request.memory
        || !request
            .panel_splits
            .iter()
            .map(String::as_str)
            .eq(REGISTERED_PANEL_SPLITS)
        || request.minimum_observations != REGISTERED_MINIMUM_OBSERVATIONS
        || request.minimum_fitted_rows != REGISTERED_MIN_FITTED_ROWS
        || request.minimum_fit_observations != REGISTERED_MIN_FIT_OBSERVATIONS
        || request.expected_train_tokens_sha256.as_deref() != Some(REGISTERED_TRAIN_TOKENS_SHA256)
        || request.expected_train_mask_sha256.as_deref() != Some(REGISTERED_TRAIN_MASK_SHA256)
        || request.expected_rows != Some(REGISTERED_EXPECTED_ROWS)
        || request.expected_row_identity.as_deref() != Some(REGISTERED_POOLED_ROW_IDENTITY)
    {
        return Err(invalid(
            "query cache request does not match the registered n3S evaluation",
        ));
    }
    Ok(())
}

fn validate_registered_cache(
    cache: &QueryCache,
    request: &CentroidEvaluationRequest,
) -> Result<()> {
    if cache.manifest.width != REGISTERED_WIDTH
        || cache.manifest.splits.len() != 1 + REGISTERED_PANEL_SPLITS.len()
        || cache
            .manifest
            .splits
            .iter()
            .any(|split| split.memory || split.label != "no-memory")
    {
        return Err(invalid(
            "query cache does not match the registered n3S inventory",
        ));
    }
    let train = registered_split(cache, REGISTERED_TRAIN_SPLIT)?;
    if train.context != REGISTERED_CONTEXT
        || train.tokens_sha256 != REGISTERED_TRAIN_TOKENS_SHA256
        || train.mask_sha256 != REGISTERED_TRAIN_MASK_SHA256
    {
        return Err(invalid("query cache TRAIN split differs from registration"));
    }
    for (name, tokens, mask) in REGISTERED_PANEL_INPUTS {
        let split = registered_split(cache, name)?;
        if split.context != REGISTERED_CONTEXT
            || split.tokens_sha256 != tokens
            || split.mask_sha256 != mask
        {
            return Err(invalid(format!(
                "query cache {name} split differs from registration"
            )));
        }
    }
    if cache.manifest.run_config.is_none() || cache.manifest.run_config_sha256.is_none() {
        return Err(invalid("query cache run config is not registered"));
    }
    if request.checkpoint_root.is_some()
        && !request.checkpoint_root.as_deref().is_some_and(Path::is_dir)
    {
        return Err(invalid("query cache checkpoint root is not a directory"));
    }
    Ok(())
}

fn registered_split<'a>(cache: &'a QueryCache, name: &str) -> Result<&'a QueryCacheSplit> {
    cache
        .manifest
        .splits
        .iter()
        .find(|split| split.split == name && !split.memory)
        .ok_or_else(|| invalid(format!("query cache has no registered {name} split")))
}

fn fit_streamed_tables(
    cache: &QueryCache,
    metadata: &QueryCacheSplit,
    rows: &CachedRows,
    request: &CentroidEvaluationRequest,
) -> Result<FittedTables> {
    let vocab = cache.manifest.vocab;
    let width = cache.manifest.width;
    let mut counts = vec![0usize; vocab];
    for &target in &rows.targets {
        let target = target as usize;
        if target >= vocab {
            return Err(invalid("query cache TRAIN target is outside the table"));
        }
        counts[target] += 1;
    }
    let fitted_tokens = (0..vocab)
        .filter(|&token| counts[token] >= request.minimum_observations)
        .collect::<Vec<_>>();
    if fitted_tokens.is_empty() {
        return Err(invalid("query cache centroid has no eligible rows"));
    }
    let fitted_observations = fitted_tokens.iter().map(|&token| counts[token]).sum();
    let mut strata = BTreeMap::<usize, Vec<usize>>::new();
    for &token in &fitted_tokens {
        strata.entry(counts[token]).or_default().push(token);
    }
    let control_strata = strata.len();
    let singleton_control_targets = strata.values().filter(|targets| targets.len() == 1).count();
    let mut control_targets = vec![None; vocab];
    for targets in strata.values() {
        for (index, &target) in targets.iter().enumerate() {
            control_targets[target] = Some(targets[(index + 1) % targets.len()]);
        }
    }
    let shuffled_changed_targets = control_targets
        .iter()
        .enumerate()
        .filter(|(token, control)| control.is_some_and(|control| *token != control))
        .count();

    let mut candidate_sums = vec![0f64; vocab.saturating_mul(width)];
    let mut control_sums = vec![0f64; vocab.saturating_mul(width)];
    let query_path = cache.root.join(&metadata.query_file);
    let expected_bytes = rows
        .targets
        .len()
        .checked_mul(width)
        .and_then(|values| values.checked_mul(4))
        .ok_or_else(|| invalid("query cache TRAIN query length overflows"))?;
    let actual_bytes = fs::metadata(&query_path)
        .map_err(|error| invalid(format!("{}: {error}", query_path.display())))?
        .len();
    if actual_bytes != expected_bytes as u64 {
        return Err(invalid("query cache TRAIN query length differs"));
    }
    let mut reader = BufReader::new(
        File::open(&query_path)
            .map_err(|error| invalid(format!("{}: {error}", query_path.display())))?,
    );
    let mut hasher = Sha256::new();
    let row_bytes = width
        .checked_mul(4)
        .ok_or_else(|| invalid("query cache row width overflows"))?;
    let mut buffer = vec![0u8; row_bytes];
    for (row, &target) in rows.targets.iter().enumerate() {
        reader
            .read_exact(&mut buffer)
            .map_err(|error| invalid(format!("query cache TRAIN row {row}: {error}")))?;
        hasher.update(&buffer);
        let target = target as usize;
        let candidate = target.saturating_mul(width);
        let control_target = control_targets[target].map(|target| target.saturating_mul(width));
        for coordinate in 0..width {
            let at = coordinate * 4;
            let value =
                f32::from_le_bytes([buffer[at], buffer[at + 1], buffer[at + 2], buffer[at + 3]]);
            let value = f64::from(f32_to_q12(value)?);
            if let Some(control_target) = control_target {
                candidate_sums[candidate + coordinate] += value;
                control_sums[control_target + coordinate] += value;
            }
        }
    }
    let mut tail = [0u8; 1];
    if reader
        .read(&mut tail)
        .map_err(|error| invalid(error.to_string()))?
        != 0
        || format!("{:x}", hasher.finalize()) != metadata.query_sha256
    {
        return Err(invalid("query cache TRAIN query hash or length differs"));
    }

    let mut centroid = cache.table.clone();
    let mut shuffled = cache.table.clone();
    let mut changed_coordinates = 0u64;
    let mut displacement_l1 = 0u64;
    let mut shuffled_changed_coordinates = 0u64;
    for &token in &fitted_tokens {
        let base = token.saturating_mul(width);
        for coordinate in 0..width {
            let candidate = q12_mean(candidate_sums[base + coordinate], counts[token])?;
            let control = q12_mean(control_sums[base + coordinate], counts[token])?;
            changed_coordinates += u64::from(candidate != cache.table[base + coordinate]);
            displacement_l1 += u64::from(cache.table[base + coordinate].abs_diff(candidate));
            centroid[base + coordinate] = candidate;
            shuffled[base + coordinate] = control;
            shuffled_changed_coordinates += u64::from(candidate != control);
        }
    }
    Ok(FittedTables {
        centroid,
        shuffled,
        fitted_rows: fitted_tokens.len(),
        observations: fitted_observations,
        control_strata,
        singleton_control_targets,
        changed_coordinates,
        displacement_l1,
        shuffled_changed_targets,
        shuffled_changed_coordinates,
    })
}

fn load_q12_queries(root: &Path, metadata: &QueryCacheSplit, width: usize) -> Result<Vec<i16>> {
    let bytes = read_checked(root, &metadata.query_file, &metadata.query_sha256)?;
    let expected = metadata
        .rows
        .checked_mul(width)
        .and_then(|values| values.checked_mul(4))
        .ok_or_else(|| invalid("query cache query length overflows"))?;
    if bytes.len() != expected {
        return Err(invalid("query cache query row count differs"));
    }
    let mut queries = Vec::with_capacity(metadata.rows.saturating_mul(width));
    for chunk in bytes.chunks_exact(4) {
        let value = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        queries.push(f32_to_q12(value)?);
    }
    Ok(queries)
}

fn evaluate_table(
    queries: &[i16],
    targets: &[u32],
    rows: &CachedRows,
    table: &[i16],
    vocab: usize,
    width: usize,
) -> Result<Decisions> {
    let block_rows = QUERY_BLOCK_ROWS;
    let decoded = queries
        .par_chunks(width.saturating_mul(block_rows))
        .enumerate()
        .map(|(block, block_queries)| {
            let rows_in_block = block_queries.len() / width;
            let mut best = vec![Vec::<(i32, u32)>::with_capacity(8); rows_in_block];
            for token in 0..vocab {
                let candidate = &table[token * width..(token + 1) * width];
                for (row, query) in block_queries.chunks_exact(width).enumerate() {
                    let distance = query
                        .iter()
                        .zip(candidate)
                        .fold(0i32, |sum, (&left, &right)| {
                            sum.saturating_add(i32::from(left.abs_diff(right)))
                        });
                    insert_best(&mut best[row], distance, token as u32);
                }
            }
            (block, best)
        })
        .collect::<Vec<_>>();
    let decoded = decoded
        .into_iter()
        .flat_map(|(block, best)| {
            let start = block * block_rows;
            best.into_iter().enumerate().map(move |(row, candidates)| {
                let index = start + row;
                let top1 = candidates[0].1;
                (
                    top1,
                    top1 == targets[index],
                    candidates.iter().any(|&(_, token)| token == targets[index]),
                )
            })
        })
        .collect::<Vec<_>>();
    let mut tokens = Vec::with_capacity(decoded.len());
    let mut top1_rows = Vec::with_capacity(decoded.len());
    let mut top8_rows = Vec::with_capacity(decoded.len());
    let mut bytes = Vec::with_capacity(decoded.len() * 4);
    for (token, top1, top8) in decoded {
        tokens.push(token);
        top1_rows.push(top1);
        top8_rows.push(top8);
        bytes.extend_from_slice(&token.to_le_bytes());
    }
    let top1_correct = top1_rows.iter().filter(|&&hit| hit).count();
    let top8_correct = top8_rows.iter().filter(|&&hit| hit).count();
    let changed_from_frozen = tokens
        .iter()
        .zip(&rows.nearest)
        .filter(|(left, right)| left != right)
        .count();
    Ok(Decisions {
        counts: ReadoutCounts {
            rows: tokens.len(),
            top1_correct,
            top8_correct,
            changed_from_frozen,
            decision_sha256: sha256(&bytes),
        },
        top1_rows,
        top8_rows,
        tokens,
    })
}

fn insert_best(best: &mut Vec<(i32, u32)>, distance: i32, token: u32) {
    let candidate = (distance, token);
    let position = best
        .iter()
        .position(|current| {
            candidate
                .0
                .cmp(&current.0)
                .then(candidate.1.cmp(&current.1))
                .is_lt()
        })
        .unwrap_or(best.len());
    if position < 8 {
        best.insert(position, candidate);
        best.truncate(8);
    }
}

fn paired(candidate: &[bool], control: &[bool]) -> serde_json::Value {
    if candidate.len() != control.len() {
        return serde_json::json!({"error": "paired decisions differ"});
    }
    let candidate_only = candidate
        .iter()
        .zip(control)
        .filter(|(left, right)| **left && !**right)
        .count() as u64;
    let control_only = candidate
        .iter()
        .zip(control)
        .filter(|(left, right)| !**left && **right)
        .count() as u64;
    let candidate_sum = candidate.iter().filter(|&&hit| hit).count() as u64;
    let control_sum = control.iter().filter(|&&hit| hit).count() as u64;
    let discordant = candidate_only + control_only;
    let threshold = ceil_sqrt(discordant.saturating_mul(4)).max(2);
    serde_json::json!({
        "candidate_only": candidate_only,
        "control_only": control_only,
        "candidate_sum": candidate_sum,
        "control_sum": control_sum,
        "discordant": discordant,
        "threshold": threshold,
        "passes": candidate_sum > control_sum && candidate_only >= threshold,
    })
}

fn ceil_sqrt(value: u64) -> u64 {
    if value <= 1 {
        return value;
    }
    let mut low = 1u64;
    let mut high = value.min(1u64 << 32);
    while low < high {
        let middle = low + (high - low) / 2;
        if middle.saturating_mul(middle) >= value {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    low
}

fn decode_cached_rows(bytes: &[u8], expected_rows: usize, vocab: usize) -> Result<CachedRows> {
    let minimum = 16usize
        .checked_add(expected_rows.saturating_mul(18))
        .ok_or_else(|| invalid("query cache row inventory overflows"))?;
    if bytes.len() != minimum || &bytes[..8] != ROW_MAGIC {
        return Err(invalid("query cache row file header or length differs"));
    }
    let stored = u64::from_le_bytes(bytes[8..16].try_into().unwrap_or([0; 8]));
    if stored != expected_rows as u64 {
        return Err(invalid("query cache row count differs"));
    }
    let mut at = 16usize;
    let mut read_u32 = |count: usize| -> Result<Vec<u32>> {
        let end = at
            .checked_add(count.saturating_mul(4))
            .ok_or_else(|| invalid("query cache row offset overflows"))?;
        let values = bytes
            .get(at..end)
            .ok_or_else(|| invalid("query cache row file is truncated"))?
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();
        at = end;
        Ok(values)
    };
    let targets = read_u32(expected_rows)?;
    let positions = read_u32(expected_rows)?;
    let nearest = read_u32(expected_rows)?;
    let writer = read_u32(expected_rows)?;
    if targets
        .iter()
        .chain(&nearest)
        .chain(&writer)
        .any(|&token| token as usize >= vocab)
    {
        return Err(invalid("query cache decision token is outside the table"));
    }
    if positions.iter().copied().collect::<BTreeSet<_>>().len() != expected_rows {
        return Err(invalid("query cache split has duplicate positions"));
    }
    let opens = bytes[at..at + expected_rows].to_vec();
    at += expected_rows;
    let copies = bytes[at..at + expected_rows].to_vec();
    if opens.iter().chain(&copies).any(|&flag| flag > 1) {
        return Err(invalid("query cache row flag differs"));
    }
    Ok(CachedRows {
        targets,
        positions,
        nearest,
        writer,
        opens,
        copies,
    })
}

fn read_checked(root: &Path, file: &str, expected: &str) -> Result<Vec<u8>> {
    safe_relative(file)?;
    let path = root.join(file);
    let bytes = fs::read(&path).map_err(|error| invalid(format!("{}: {error}", path.display())))?;
    if sha256(&bytes) != expected {
        return Err(invalid(format!("query cache file {file} hash differs")));
    }
    Ok(bytes)
}

fn verify_file_checked(path: &Path, expected: &str) -> Result<()> {
    let actual = file_sha256(path)?;
    if actual != expected {
        return Err(invalid(format!(
            "query cache file {} hash differs",
            path.display()
        )));
    }
    Ok(())
}

fn file_sha256(path: &Path) -> Result<String> {
    let mut reader = BufReader::new(
        File::open(path).map_err(|error| invalid(format!("{}: {error}", path.display())))?,
    );
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 1024 * 1024];
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|error| invalid(format!("{}: {error}", path.display())))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn safe_relative(file: &str) -> Result<()> {
    let path = Path::new(file);
    if file.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(invalid("query cache file path escapes its root"));
    }
    Ok(())
}

fn decode_f32(bytes: &[u8]) -> Result<Vec<f32>> {
    if !bytes.len().is_multiple_of(4) {
        return Err(invalid("query cache F32 file is not aligned"));
    }
    Ok(bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect())
}

fn f32_to_q12(value: f32) -> Result<i16> {
    if !value.is_finite() {
        return Err(invalid("query cache F32 value is not finite"));
    }
    let scaled = value * Q12_SCALE;
    let rounded = scaled.round();
    let integer = i16::try_from(rounded as i32)
        .map_err(|_| invalid("query cache Q12 value is outside i16"))?;
    if f32::from(integer) / Q12_SCALE != value {
        return Err(invalid("query cache query or table is not on the Q12 grid"));
    }
    Ok(integer)
}

fn q12_mean(sum: f64, count: usize) -> Result<i16> {
    if count == 0 || !sum.is_finite() {
        return Err(invalid("query cache centroid count or sum differs"));
    }
    let value = sum / count as f64;
    let rounded = if value >= 0.0 {
        value.round()
    } else {
        -(-value).round()
    };
    i16::try_from(rounded as i32).map_err(|_| invalid("query cache centroid is outside i16"))
}

fn row_identity(rows: &CachedRows) -> [u8; 32] {
    let mut hasher = Sha256::new();
    for (&position, &target) in rows.positions.iter().zip(&rows.targets) {
        hasher.update(u64::from(position).to_le_bytes());
        hasher.update(target.to_le_bytes());
    }
    hasher.finalize().into()
}

fn pooled_identity(panels: &[(String, [u8; 32])]) -> String {
    let mut hasher = Sha256::new();
    for (name, identity) in panels {
        hasher.update((name.len() as u64).to_le_bytes());
        hasher.update(name.as_bytes());
        hasher.update(identity);
    }
    format!("{:x}", hasher.finalize())
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q12(values: &[i16]) -> Vec<f32> {
        values
            .iter()
            .map(|&value| f32::from(value) / Q12_SCALE)
            .collect()
    }

    fn f32_bytes(values: &[f32]) -> Vec<u8> {
        values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect()
    }

    fn row_bytes(rows: &CachedRows) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(ROW_MAGIC);
        bytes.extend_from_slice(&(rows.targets.len() as u64).to_le_bytes());
        for values in [&rows.targets, &rows.positions, &rows.nearest, &rows.writer] {
            for value in values {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        bytes.extend_from_slice(&rows.opens);
        bytes.extend_from_slice(&rows.copies);
        bytes
    }

    fn fixture(name: &str) -> (PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "uor-r4-cpu-query-cache-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let checkpoint = root.join("checkpoint");
        fs::create_dir_all(&checkpoint).unwrap();
        let checkpoint_config = b"{}";
        fs::write(checkpoint.join("config.json"), checkpoint_config).unwrap();
        let cache_root = root.join("cache");
        fs::create_dir_all(cache_root.join("no-memory")).unwrap();

        let table = q12(&[0, 10, 20]);
        let table_bytes = f32_bytes(&table);
        fs::write(cache_root.join("table.f32"), &table_bytes).unwrap();
        let train_rows = CachedRows {
            targets: vec![0, 0, 0, 1, 1, 1, 2],
            positions: vec![1, 2, 3, 4, 5, 6, 7],
            nearest: vec![0, 0, 0, 1, 1, 1, 2],
            writer: vec![0, 0, 0, 1, 1, 1, 2],
            opens: vec![0; 7],
            copies: vec![0; 7],
        };
        let train_queries = f32_bytes(&q12(&[0, 2, 4, 14, 16, 18, 22]));
        fs::write(
            cache_root.join("no-memory/train.queries.f32"),
            &train_queries,
        )
        .unwrap();
        let train_rows_bytes = row_bytes(&train_rows);
        fs::write(
            cache_root.join("no-memory/train.rows.bin"),
            &train_rows_bytes,
        )
        .unwrap();
        let panel_rows = CachedRows {
            targets: vec![0, 1, 2],
            positions: vec![1, 2, 3],
            nearest: vec![0, 1, 2],
            writer: vec![0, 1, 2],
            opens: vec![0; 3],
            copies: vec![0; 3],
        };
        let panel_queries = f32_bytes(&q12(&[1, 14, 20]));
        fs::write(cache_root.join("no-memory/p.queries.f32"), &panel_queries).unwrap();
        let panel_rows_bytes = row_bytes(&panel_rows);
        fs::write(cache_root.join("no-memory/p.rows.bin"), &panel_rows_bytes).unwrap();

        let manifest = QueryCacheManifest {
            schema: QUERY_CACHE_SCHEMA.to_owned(),
            checkpoint: checkpoint.display().to_string(),
            data_root: root.display().to_string(),
            tokenizer_sha256: PINNED_TOKENIZER_SHA256.to_owned(),
            checkpoint_files: BTreeMap::from([(
                "config.json".to_owned(),
                sha256(checkpoint_config),
            )]),
            run_config: None,
            run_config_sha256: None,
            table_file: "table.f32".to_owned(),
            table_sha256: sha256(&table_bytes),
            vocab: 3,
            width: 1,
            query_space: "q12-f32".to_owned(),
            splits: vec![
                QueryCacheSplit {
                    split: "train".to_owned(),
                    label: "no-memory".to_owned(),
                    memory: false,
                    context: 4,
                    rows: 7,
                    tokens_sha256: "train-tokens".to_owned(),
                    mask_sha256: "train-mask".to_owned(),
                    query_file: "no-memory/train.queries.f32".to_owned(),
                    query_sha256: sha256(&train_queries),
                    rows_file: "no-memory/train.rows.bin".to_owned(),
                    rows_sha256: sha256(&train_rows_bytes),
                },
                QueryCacheSplit {
                    split: "p".to_owned(),
                    label: "no-memory".to_owned(),
                    memory: false,
                    context: 4,
                    rows: 3,
                    tokens_sha256: "panel-tokens".to_owned(),
                    mask_sha256: "panel-mask".to_owned(),
                    query_file: "no-memory/p.queries.f32".to_owned(),
                    query_sha256: sha256(&panel_queries),
                    rows_file: "no-memory/p.rows.bin".to_owned(),
                    rows_sha256: sha256(&panel_rows_bytes),
                },
            ],
        };
        let manifest_bytes = serde_json::to_vec_pretty(&manifest).unwrap();
        fs::write(cache_root.join("manifest.json"), &manifest_bytes).unwrap();
        let seal = QueryCacheSeal {
            schema: QUERY_CACHE_SEAL_SCHEMA.to_owned(),
            manifest_sha256: sha256(&manifest_bytes),
        };
        fs::write(
            cache_root.join("seal.json"),
            serde_json::to_vec_pretty(&seal).unwrap(),
        )
        .unwrap();
        (root, cache_root)
    }

    fn rewrite_manifest(cache_root: &Path, update: impl FnOnce(&mut QueryCacheManifest)) {
        let path = cache_root.join("manifest.json");
        let mut manifest: QueryCacheManifest =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        update(&mut manifest);
        let bytes = serde_json::to_vec_pretty(&manifest).unwrap();
        fs::write(&path, &bytes).unwrap();
        fs::write(
            cache_root.join("seal.json"),
            serde_json::to_vec_pretty(&QueryCacheSeal {
                schema: QUERY_CACHE_SEAL_SCHEMA.to_owned(),
                manifest_sha256: sha256(&bytes),
            })
            .unwrap(),
        )
        .unwrap();
    }

    fn request() -> CentroidEvaluationRequest {
        CentroidEvaluationRequest {
            seed: None,
            train_split: "train".to_owned(),
            panel_splits: vec!["p".to_owned()],
            memory: false,
            minimum_observations: 1,
            minimum_fitted_rows: 3,
            minimum_fit_observations: 7,
            expected_train_tokens_sha256: Some("train-tokens".to_owned()),
            expected_train_mask_sha256: Some("train-mask".to_owned()),
            expected_rows: Some(3),
            expected_row_identity: Some(
                "4bcb985dd6bfd61c3bf7451f03f8f3dbe03643b09ac903da46db0e8015957274".to_owned(),
            ),
            checkpoint_root: None,
        }
    }

    fn registered_request() -> CentroidEvaluationRequest {
        CentroidEvaluationRequest {
            seed: Some(1),
            train_split: REGISTERED_TRAIN_SPLIT.to_owned(),
            panel_splits: REGISTERED_PANEL_SPLITS
                .into_iter()
                .map(str::to_owned)
                .collect(),
            memory: false,
            minimum_observations: REGISTERED_MINIMUM_OBSERVATIONS,
            minimum_fitted_rows: REGISTERED_MIN_FITTED_ROWS,
            minimum_fit_observations: REGISTERED_MIN_FIT_OBSERVATIONS,
            expected_train_tokens_sha256: Some(REGISTERED_TRAIN_TOKENS_SHA256.to_owned()),
            expected_train_mask_sha256: Some(REGISTERED_TRAIN_MASK_SHA256.to_owned()),
            expected_rows: Some(REGISTERED_EXPECTED_ROWS),
            expected_row_identity: Some(REGISTERED_POOLED_ROW_IDENTITY.to_owned()),
            checkpoint_root: None,
        }
    }

    #[test]
    fn q12_round_trip_and_ties_are_exact() {
        assert_eq!(f32_to_q12(1.0 / Q12_SCALE).unwrap(), 1);
        assert_eq!(f32_to_q12(-1.0 / Q12_SCALE).unwrap(), -1);
        assert!(f32_to_q12(1.0 / 8192.0).is_err());
        assert_eq!(q12_mean(1.0, 2).unwrap(), 1);
        assert_eq!(q12_mean(-1.0, 2).unwrap(), -1);
        assert_eq!(q12_mean(-3.0, 2).unwrap(), -2);
        let mut best = Vec::new();
        for token in [11, 1, 10, 2, 9, 3, 8, 4, 7, 5, 6, 12] {
            insert_best(&mut best, 3, token);
        }
        assert_eq!(
            best.iter().map(|candidate| candidate.1).collect::<Vec<_>>(),
            [1, 2, 3, 4, 5, 6, 7, 8]
        );
        insert_best(&mut best, 3, 9);
        assert_eq!(best.len(), 8);
        assert_eq!(best.last().map(|candidate| candidate.1), Some(8));
    }

    #[test]
    fn top8_membership_is_exact_across_parallel_blocks() {
        let rows = QUERY_BLOCK_ROWS * 2 + 1;
        let queries = vec![0i16; rows];
        let targets = (0..rows)
            .map(|row| if row.is_multiple_of(2) { 7 } else { 8 })
            .collect::<Vec<_>>();
        let cached = CachedRows {
            targets: targets.clone(),
            positions: (1..=rows as u32).collect(),
            nearest: vec![0; rows],
            writer: vec![0; rows],
            opens: vec![0; rows],
            copies: vec![0; rows],
        };
        let table = (0i16..12).collect::<Vec<_>>();
        let decisions = evaluate_table(&queries, &targets, &cached, &table, 12, 1).unwrap();
        let expected = targets
            .iter()
            .map(|target| *target == 7)
            .collect::<Vec<_>>();
        assert_eq!(decisions.top1_rows, vec![false; rows]);
        assert_eq!(decisions.top8_rows, expected);
        assert_eq!(decisions.counts.top8_correct, rows.div_ceil(2));
    }

    #[test]
    fn exact_count_shuffle_uses_every_eligible_row_and_is_not_vacuous() {
        let (root, cache_root) = fixture("matched");
        let report = evaluate_query_cache(&cache_root, &request()).unwrap();
        assert_eq!(report["status"], "REJECT");
        assert_eq!(report["fit"]["fitted_rows"], 3);
        assert_eq!(report["fit"]["observations"], 7);
        assert_eq!(report["fit"]["power_ok"], true);
        assert_eq!(
            report["fit"]["control_strategy"],
            "exact-count-stratified-cycle"
        );
        assert_eq!(report["fit"]["control_strata"], 2);
        assert_eq!(report["fit"]["singleton_control_targets"], 1);
        assert_eq!(report["fit"]["shuffled_changed_targets"], 2);
        assert!(
            report["fit"]["shuffled_changed_coordinates"]
                .as_u64()
                .unwrap()
                > 0
        );
        assert_eq!(report["pooled"]["rows"], 3);
        assert_eq!(report["pooled"]["identity_matches"], true);
        assert_eq!(report["panels"][0]["identity"]["exact"], true);

        let registered = request();
        let one = rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .build()
            .unwrap()
            .install(|| evaluate_query_cache(&cache_root, &registered).unwrap());
        let two = rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .build()
            .unwrap()
            .install(|| evaluate_query_cache(&cache_root, &registered).unwrap());
        let normalize = |mut value: serde_json::Value| {
            value["wall_ms"] = 0.into();
            value["workers"] = 0.into();
            value
        };
        assert_eq!(normalize(one), normalize(two));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn worker_order_is_stable_across_multiple_parallel_blocks() {
        let (root, cache_root) = fixture("worker-order");
        let rows = QUERY_BLOCK_ROWS * 2 + 1;
        let targets = (0..rows).map(|row| (row % 3) as u32).collect::<Vec<_>>();
        let panel_rows = CachedRows {
            targets: targets.clone(),
            positions: (1..=rows as u32).collect(),
            nearest: targets.clone(),
            writer: targets.clone(),
            opens: vec![0; rows],
            copies: vec![0; rows],
        };
        let query_values = targets
            .iter()
            .map(|target| match target {
                0 => 0,
                1 => 10,
                _ => 20,
            })
            .collect::<Vec<_>>();
        let queries = f32_bytes(&q12(&query_values));
        let rows_bytes = row_bytes(&panel_rows);
        fs::write(cache_root.join("no-memory/p.queries.f32"), &queries).unwrap();
        fs::write(cache_root.join("no-memory/p.rows.bin"), &rows_bytes).unwrap();
        rewrite_manifest(&cache_root, |manifest| {
            let panel = manifest
                .splits
                .iter_mut()
                .find(|split| split.split == "p")
                .unwrap();
            panel.rows = rows;
            panel.query_sha256 = sha256(&queries);
            panel.rows_sha256 = sha256(&rows_bytes);
        });
        let pooled = pooled_identity(&[("p".to_owned(), row_identity(&panel_rows))]);
        let mut registered = request();
        registered.expected_rows = Some(rows);
        registered.expected_row_identity = Some(pooled);

        let one = rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .build()
            .unwrap()
            .install(|| evaluate_query_cache(&cache_root, &registered).unwrap());
        let two = rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .build()
            .unwrap()
            .install(|| evaluate_query_cache(&cache_root, &registered).unwrap());
        let normalize = |mut value: serde_json::Value| {
            value["wall_ms"] = 0.into();
            value["workers"] = 0.into();
            value
        };
        assert_eq!(normalize(one), normalize(two));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn singleton_count_strata_use_their_own_query_in_both_arms() {
        let (root, cache_root) = fixture("singleton-control");
        let cache = QueryCache::open(&cache_root).unwrap();
        let (metadata, rows) = cache.split("train", false).unwrap();
        let fitted = fit_streamed_tables(&cache, metadata, &rows, &request()).unwrap();

        assert_eq!(fitted.centroid[0], 2);
        assert_eq!(fitted.shuffled[0], 16);
        assert_eq!(fitted.centroid[2], 22);
        assert_eq!(fitted.shuffled[2], 22);
        assert_eq!(fitted.singleton_control_targets, 1);
        assert_eq!(fitted.shuffled_changed_targets, 2);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn every_train_query_must_be_on_the_q12_grid_even_when_ineligible() {
        let (root, cache_root) = fixture("ineligible-q12");
        let query = cache_root.join("no-memory/train.queries.f32");
        let mut bytes = fs::read(&query).unwrap();
        let last = bytes.len() - 4;
        bytes[last..].copy_from_slice(&(22.5 / Q12_SCALE).to_le_bytes());
        fs::write(&query, &bytes).unwrap();
        rewrite_manifest(&cache_root, |manifest| {
            let train = manifest
                .splits
                .iter_mut()
                .find(|split| split.split == "train")
                .unwrap();
            train.query_sha256 = sha256(&bytes);
        });

        let mut ineligible = request();
        ineligible.minimum_observations = 3;
        let error = evaluate_query_cache(&cache_root, &ineligible)
            .unwrap_err()
            .to_string();
        assert!(error.contains("Q12"), "{error}");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn checkpoint_inventory_hashes_are_verified() {
        let (root, cache_root) = fixture("checkpoint-hash");
        fs::write(root.join("checkpoint/config.json"), b"[]").unwrap();
        let error = evaluate_query_cache(&cache_root, &request())
            .unwrap_err()
            .to_string();
        assert!(error.contains("hash differs"), "{error}");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn every_manifest_split_and_run_config_hash_is_verified() {
        let (root, cache_root) = fixture("manifest-inventory");
        let panel_query = fs::read(cache_root.join("no-memory/p.queries.f32")).unwrap();
        let panel_rows = fs::read(cache_root.join("no-memory/p.rows.bin")).unwrap();
        fs::write(
            cache_root.join("no-memory/unused.queries.f32"),
            &panel_query,
        )
        .unwrap();
        fs::write(cache_root.join("no-memory/unused.rows.bin"), &panel_rows).unwrap();
        let run_config = root.join("run/config.json");
        fs::create_dir_all(run_config.parent().unwrap()).unwrap();
        let run_config_bytes = br#"{"steps":8192}"#;
        fs::write(&run_config, run_config_bytes).unwrap();
        rewrite_manifest(&cache_root, |manifest| {
            let mut unused = manifest
                .splits
                .iter()
                .find(|split| split.split == "p")
                .unwrap()
                .clone();
            unused.split = "unused".to_owned();
            unused.query_file = "no-memory/unused.queries.f32".to_owned();
            unused.query_sha256 = sha256(&panel_query);
            unused.rows_file = "no-memory/unused.rows.bin".to_owned();
            unused.rows_sha256 = sha256(&panel_rows);
            manifest.splits.push(unused);
            manifest.run_config = Some(run_config.display().to_string());
            manifest.run_config_sha256 = Some(sha256(run_config_bytes));
        });

        evaluate_query_cache(&cache_root, &request()).unwrap();
        fs::write(&run_config, br#"{"steps":1}"#).unwrap();
        let error = evaluate_query_cache(&cache_root, &request())
            .unwrap_err()
            .to_string();
        assert!(error.contains("run config hash differs"), "{error}");
        fs::write(&run_config, run_config_bytes).unwrap();
        fs::write(cache_root.join("no-memory/unused.queries.f32"), b"tampered").unwrap();
        let error = evaluate_query_cache(&cache_root, &request())
            .unwrap_err()
            .to_string();
        assert!(error.contains("hash differs"), "{error}");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn widths_that_cannot_hold_exact_l1_are_refused() {
        let (root, cache_root) = fixture("unsupported-width");
        rewrite_manifest(&cache_root, |manifest| manifest.width = 32_769);
        let error = QueryCache::open(&cache_root).unwrap_err().to_string();
        assert!(error.contains("identity differs"), "{error}");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn registered_gate_cannot_be_relaxed_or_left_unpinned() {
        let registered = registered_request();
        validate_registered_request(&registered).unwrap();

        let mut lowered = registered.clone();
        lowered.minimum_observations = REGISTERED_MINIMUM_OBSERVATIONS - 1;
        assert!(validate_registered_request(&lowered).is_err());

        let mut unpinned = registered;
        unpinned.expected_row_identity = None;
        assert!(validate_registered_request(&unpinned).is_err());
    }

    #[test]
    fn registered_power_identity_and_tamper_gates_fail_closed() {
        let (root, cache_root) = fixture("gates");
        let mut underpowered = request();
        underpowered.minimum_fitted_rows = 4;
        let report = evaluate_query_cache(&cache_root, &underpowered).unwrap();
        assert_eq!(report["status"], "INSUFFICIENT_POWER");

        let mut wrong_identity = request();
        wrong_identity.expected_row_identity = Some("00".repeat(32));
        let report = evaluate_query_cache(&cache_root, &wrong_identity).unwrap();
        assert_eq!(report["status"], "CONTROL_FAULT");

        let mut wrong_train = request();
        wrong_train.expected_train_tokens_sha256 = Some("not-the-train".to_owned());
        let report = evaluate_query_cache(&cache_root, &wrong_train).unwrap();
        assert_eq!(report["status"], "CONTROL_FAULT");
        assert_eq!(report["train"]["identity_matches"], false);

        let query = cache_root.join("no-memory/p.queries.f32");
        let mut bytes = fs::read(&query).unwrap();
        bytes[0] ^= 1;
        fs::write(&query, bytes).unwrap();
        assert!(evaluate_query_cache(&cache_root, &request()).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn two_seed_aggregate_has_fixed_precedence() {
        let panel_hashes = [
            "01".repeat(32),
            "02".repeat(32),
            "03".repeat(32),
            "04".repeat(32),
        ];
        let expected_identity = pooled_identity(
            &REGISTERED_PANEL_SPLITS
                .into_iter()
                .zip(&panel_hashes)
                .map(|(name, hash)| {
                    (
                        name.to_owned(),
                        parse_sha256(hash).expect("test panel identity"),
                    )
                })
                .collect::<Vec<_>>(),
        );
        let aggregate = |reports: &[serde_json::Value]| {
            aggregate_query_cache_reports_with_identity(reports, &expected_identity)
        };
        let report = |status: &str, seed: u8, manifest: &str, bar: bool| {
            let control_fault = status == "CONTROL_FAULT";
            let insufficient_power = status == "INSUFFICIENT_POWER";
            let mechanism_pass = status != "REJECT";
            let power_ok = !insufficient_power;
            let fitted_rows = if power_ok {
                REGISTERED_MIN_FITTED_ROWS
            } else {
                REGISTERED_MIN_FITTED_ROWS - 1
            };
            let observations = if power_ok {
                REGISTERED_MIN_FIT_OBSERVATIONS
            } else {
                REGISTERED_MIN_FIT_OBSERVATIONS - 1
            };
            let panel_rows = [365_174_u64, 365_174, 365_174, 365_175];
            let split_hash = "0".repeat(64);
            let mut cache_splits = vec![serde_json::json!({
                "split": REGISTERED_TRAIN_SPLIT,
                "label": "no-memory",
                "memory": false,
                "context": REGISTERED_CONTEXT,
                "rows": REGISTERED_TRAIN_ROWS,
                "tokens_sha256": REGISTERED_TRAIN_TOKENS_SHA256,
                "mask_sha256": REGISTERED_TRAIN_MASK_SHA256,
                "query_file": "no-memory/train.queries.f32",
                "query_sha256": split_hash,
                "rows_file": "no-memory/train.rows.bin",
                "rows_sha256": split_hash,
            })];
            let pair = |candidate_sum: u64, control_sum: u64| {
                let candidate_only = candidate_sum.saturating_sub(control_sum);
                let control_only = control_sum.saturating_sub(candidate_sum);
                let discordant = candidate_only + control_only;
                let threshold = ceil_sqrt(discordant.saturating_mul(4)).max(2);
                let passes = candidate_sum > control_sum && candidate_only >= threshold;
                serde_json::json!({
                    "candidate_only": candidate_only,
                    "control_only": control_only,
                    "candidate_sum": candidate_sum,
                    "control_sum": control_sum,
                    "discordant": discordant,
                    "threshold": threshold,
                    "passes": passes,
                })
            };
            let panels = REGISTERED_PANEL_SPLITS
                .into_iter()
                .zip(REGISTERED_PANEL_INPUTS)
                .zip(panel_rows)
                .zip(&panel_hashes)
                .map(|(((split, (_, tokens, mask)), rows), identity)| {
                    cache_splits.push(serde_json::json!({
                        "split": split,
                        "label": "no-memory",
                        "memory": false,
                        "context": REGISTERED_CONTEXT,
                        "rows": rows,
                        "tokens_sha256": tokens,
                        "mask_sha256": mask,
                        "query_file": format!("no-memory/{split}.queries.f32"),
                        "query_sha256": split_hash,
                        "rows_file": format!("no-memory/{split}.rows.bin"),
                        "rows_sha256": split_hash,
                    }));
                    let original_top1 = 25;
                    let centroid_top1 = if mechanism_pass { 26 } else { 24 };
                    let shuffled_top1 = 24;
                    let original_top8 = 30;
                    let centroid_top8 = if mechanism_pass { 30 } else { 25 };
                    let shuffled_top8 = 30;
                    serde_json::json!({
                        "split": split,
                        "memory": false,
                        "context": REGISTERED_CONTEXT,
                        "rows": rows,
                        "tokens_sha256": tokens,
                        "mask_sha256": mask,
                        "row_identity_sha256": identity,
                        "identity": {
                            "exact": true,
                            "decision_sha256": split_hash,
                        },
                        "original": {
                            "rows": rows,
                            "top1_correct": original_top1,
                            "top8_correct": original_top8,
                            "changed_from_frozen": 0,
                            "decision_sha256": split_hash,
                        },
                        "centroid": {
                            "rows": rows,
                            "top1_correct": centroid_top1,
                            "top8_correct": centroid_top8,
                            "changed_from_frozen": 0,
                            "decision_sha256": split_hash,
                        },
                        "exact_count_shuffled_target": {
                            "rows": rows,
                            "top1_correct": shuffled_top1,
                            "top8_correct": shuffled_top8,
                            "changed_from_frozen": 0,
                            "decision_sha256": split_hash,
                        },
                        "centroid_vs_original": pair(centroid_top1, original_top1),
                        "centroid_vs_exact_count_shuffled": pair(centroid_top1, shuffled_top1),
                        "centroid_top8_not_below_original": centroid_top8 >= original_top8,
                    })
                })
                .collect::<Vec<_>>();
            let original_top1 = 100;
            let centroid_top1 = if mechanism_pass { 104 } else { 96 };
            let shuffled_top1 = 96;
            let original_top8 = 120;
            let centroid_top8 = if mechanism_pass { 120 } else { 100 };
            let shuffled_top8 = 120;
            serde_json::json!({
                "schema": EVALUATION_SCHEMA,
                "evaluation_profile": "n3s-registered-v1",
                "status": status,
                "cache": {
                    "manifest_sha256": manifest,
                    "checkpoint": "/runs/s1/checkpoint",
                    "checkpoint_files": {"model.safetensors": "a".repeat(64)},
                    "run_config": "/runs/s1/config.json",
                    "run_config_sha256": "b".repeat(64),
                    "data_root": "/data",
                    "tokenizer_sha256": PINNED_TOKENIZER_SHA256,
                    "table_sha256": "c".repeat(64),
                    "vocab": 16_384,
                    "width": REGISTERED_WIDTH,
                    "query_space": "q12-f32",
                    "splits": cache_splits,
                },
                "request": {
                    "seed": seed,
                    "train_split": REGISTERED_TRAIN_SPLIT,
                    "panel_splits": REGISTERED_PANEL_SPLITS,
                    "expected_rows": REGISTERED_EXPECTED_ROWS,
                    "expected_row_identity": expected_identity,
                    "expected_train_tokens_sha256": REGISTERED_TRAIN_TOKENS_SHA256,
                    "expected_train_mask_sha256": REGISTERED_TRAIN_MASK_SHA256,
                    "memory": false,
                    "minimum_observations": REGISTERED_MINIMUM_OBSERVATIONS,
                    "minimum_fitted_rows": REGISTERED_MIN_FITTED_ROWS,
                    "minimum_fit_observations": REGISTERED_MIN_FIT_OBSERVATIONS,
                },
                "fit": {
                    "fitted_rows": fitted_rows,
                    "observations": observations,
                    "minimum_fitted_rows": REGISTERED_MIN_FITTED_ROWS,
                    "minimum_fit_observations": REGISTERED_MIN_FIT_OBSERVATIONS,
                    "power_ok": power_ok,
                    "control_strategy": "exact-count-stratified-cycle",
                    "shuffled_changed_targets": if control_fault { 0 } else { 1 },
                    "shuffled_changed_coordinates": if control_fault { 0 } else { 1 },
                },
                "train": {
                    "split": REGISTERED_TRAIN_SPLIT,
                    "memory": false,
                    "rows": REGISTERED_TRAIN_ROWS,
                    "context": REGISTERED_CONTEXT,
                    "tokens_sha256": REGISTERED_TRAIN_TOKENS_SHA256,
                    "mask_sha256": REGISTERED_TRAIN_MASK_SHA256,
                    "expected_tokens_sha256": REGISTERED_TRAIN_TOKENS_SHA256,
                    "expected_mask_sha256": REGISTERED_TRAIN_MASK_SHA256,
                    "identity_matches": true,
                },
                "pooled": {
                    "bar_passes": bar,
                    "rows": REGISTERED_EXPECTED_ROWS,
                    "rows_match": true,
                    "identity_matches": true,
                    "row_identity_sha256": expected_identity,
                    "decision_sha256": split_hash,
                    "original_top1": original_top1,
                    "centroid_top1": centroid_top1,
                    "exact_count_shuffled_top1": shuffled_top1,
                    "original_top8": original_top8,
                    "centroid_top8": centroid_top8,
                    "exact_count_shuffled_top8": shuffled_top8,
                    "top8_holds": mechanism_pass,
                    "centroid_vs_original": pair(centroid_top1, original_top1),
                    "centroid_vs_exact_count_shuffled": pair(centroid_top1, shuffled_top1),
                },
                "panels": panels,
            })
        };
        let keep = aggregate(&[
            report("KEEP", 2, &"22".repeat(32), true),
            report("KEEP", 1, &"11".repeat(32), true),
        ])
        .unwrap();
        assert_eq!(keep["status"], "KEEP_ON_DEVELOPMENT_PANEL");
        assert_eq!(keep["seeds"][0]["seed"], 1);
        assert_eq!(keep["seeds"][1]["seed"], 2);
        let reject = aggregate(&[
            report("KEEP", 1, &"11".repeat(32), true),
            report("REJECT", 2, &"22".repeat(32), false),
        ])
        .unwrap();
        assert_eq!(reject["status"], "REJECT");
        let fault = aggregate(&[
            report("INSUFFICIENT_POWER", 1, &"11".repeat(32), false),
            report("CONTROL_FAULT", 2, &"22".repeat(32), false),
        ])
        .unwrap();
        assert_eq!(fault["status"], "CONTROL_FAULT");

        let mut lowered = report("KEEP", 1, &"11".repeat(32), true);
        lowered["request"]["minimum_observations"] = 7.into();
        let error = aggregate(&[lowered, report("KEEP", 2, &"22".repeat(32), true)])
            .unwrap_err()
            .to_string();
        assert!(error.contains("registered evaluation"), "{error}");

        let mut diagnostic = report("KEEP", 1, &"11".repeat(32), true);
        diagnostic["evaluation_profile"] = "diagnostic".into();
        let error = aggregate(&[diagnostic, report("KEEP", 2, &"22".repeat(32), true)])
            .unwrap_err()
            .to_string();
        assert!(error.contains("not registered"), "{error}");

        let duplicate_seed = aggregate(&[
            report("KEEP", 1, &"11".repeat(32), true),
            report("KEEP", 1, &"22".repeat(32), true),
        ])
        .unwrap_err()
        .to_string();
        assert!(
            duplicate_seed.contains("repeats a seed"),
            "{duplicate_seed}"
        );

        let mut panel_fault = report("CONTROL_FAULT", 1, &"11".repeat(32), false);
        panel_fault["fit"]["shuffled_changed_targets"] = 1.into();
        panel_fault["fit"]["shuffled_changed_coordinates"] = 1.into();
        panel_fault["panels"][0]["original"]["changed_from_frozen"] = 1.into();
        panel_fault["panels"][0]["identity"]["exact"] = false.into();
        let fault = aggregate(&[panel_fault, report("KEEP", 2, &"22".repeat(32), true)]).unwrap();
        assert_eq!(fault["status"], "CONTROL_FAULT");

        let mut bad_identity = report("KEEP", 1, &"11".repeat(32), true);
        bad_identity["panels"][0]["row_identity_sha256"] = "ff".repeat(32).into();
        let error = aggregate(&[bad_identity, report("KEEP", 2, &"22".repeat(32), true)])
            .unwrap_err()
            .to_string();
        assert!(error.contains("identity rollup"), "{error}");

        let mut bad_exact = report("KEEP", 1, &"11".repeat(32), true);
        bad_exact["panels"][0]["original"]["changed_from_frozen"] = 1.into();
        let error = aggregate(&[bad_exact, report("KEEP", 2, &"22".repeat(32), true)])
            .unwrap_err()
            .to_string();
        assert!(error.contains("exact flag"), "{error}");

        let mut bad_top8 = report("KEEP", 1, &"11".repeat(32), true);
        for panel in bad_top8["panels"].as_array_mut().unwrap() {
            panel["centroid"]["top8_correct"] = 29.into();
            panel["centroid_top8_not_below_original"] = false.into();
        }
        bad_top8["pooled"]["centroid_top8"] = 116.into();
        let error = aggregate(&[bad_top8, report("KEEP", 2, &"22".repeat(32), true)])
            .unwrap_err()
            .to_string();
        assert!(error.contains("top-8 flag"), "{error}");

        let mut bad_pair = report("KEEP", 1, &"11".repeat(32), true);
        bad_pair["pooled"]["centroid_vs_original"] = serde_json::json!({
            "candidate_only": 0,
            "control_only": 0,
            "candidate_sum": 104,
            "control_sum": 100,
            "discordant": 0,
            "threshold": 2,
            "passes": true,
        });
        let error = aggregate(&[bad_pair, report("KEEP", 2, &"22".repeat(32), true)])
            .unwrap_err()
            .to_string();
        assert!(error.contains("paired metrics"), "{error}");

        let impossible_pair = serde_json::json!({
            "candidate_only": 10,
            "control_only": 10,
            "candidate_sum": 10,
            "control_sum": 10,
            "discordant": 20,
            "threshold": 9,
            "passes": false,
        });
        assert!(reported_pair(&impossible_pair, 10).is_err());

        let mut missing_hashes = report("KEEP", 1, &"11".repeat(32), true);
        for panel in missing_hashes["panels"].as_array_mut().unwrap() {
            panel["identity"]["decision_sha256"] = serde_json::Value::Null;
            panel["original"]["decision_sha256"] = serde_json::Value::Null;
            panel["centroid"]["decision_sha256"] = serde_json::Value::Null;
            panel["exact_count_shuffled_target"]["decision_sha256"] = serde_json::Value::Null;
        }
        let error = aggregate(&[missing_hashes, report("KEEP", 2, &"22".repeat(32), true)])
            .unwrap_err()
            .to_string();
        assert!(error.contains("panel pins"), "{error}");

        let mut bad_train = report("KEEP", 1, &"11".repeat(32), true);
        bad_train["train"]["tokens_sha256"] = "f".repeat(64).into();
        let error = aggregate(&[bad_train, report("KEEP", 2, &"22".repeat(32), true)])
            .unwrap_err()
            .to_string();
        assert!(error.contains("TRAIN identity"), "{error}");

        let error = aggregate_query_cache_reports(&[
            report("KEEP", 1, &"11".repeat(32), true),
            report("KEEP", 2, &"22".repeat(32), true),
        ])
        .unwrap_err()
        .to_string();
        assert!(error.contains("registered evaluation"), "{error}");
    }
}
