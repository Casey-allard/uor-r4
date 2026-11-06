//! Immutable teacher-forced FBS query caches and CPU output-readout evaluation.
//!
//! The cache separates the geometric forward from output-table experiments. A writer stores
//! exactly the query rows the captured scorer saw, the table it read, and each row's target and
//! frozen decisions. A CPU evaluator can then compare output rows without rerunning the trunk or
//! changing the shared input table.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use candle_core::{DType, Tensor};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{Result, invalid, sha256_bytes};

/// The immutable query-cache schema.
pub const QUERY_CACHE_SCHEMA: &str = "uor-r4.fbs-query-cache/1";
/// The CPU evaluation-report schema.
pub const QUERY_CACHE_EVALUATION_SCHEMA: &str = "uor-r4.fbs-query-cache-evaluation/1";
const QUERY_CACHE_SEAL_SCHEMA: &str = "uor-r4.fbs-query-cache-seal/1";
const EVALUATION_QUERY_BLOCK: usize = 64;
const ROW_MAGIC: &[u8; 8] = b"UORQROW1";

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CachedRows {
    pub targets: Vec<u32>,
    pub positions: Vec<u32>,
    pub nearest: Vec<u32>,
    pub writer: Vec<u32>,
    pub opens: Vec<u8>,
    pub copies: Vec<u8>,
}

impl CachedRows {
    pub fn len(&self) -> usize {
        self.targets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }
}

pub struct CachedSplit {
    pub metadata: QueryCacheSplit,
    pub queries: Vec<f32>,
    pub rows: CachedRows,
}

pub struct QueryCacheWriter {
    root: PathBuf,
    manifest: QueryCacheManifest,
    started: BTreeSet<(String, String)>,
}

pub struct QuerySplitWriter {
    root: PathBuf,
    split: String,
    label: String,
    memory: bool,
    context: usize,
    width: usize,
    rows: usize,
    query_file: String,
    query_tmp: PathBuf,
    writer: Option<BufWriter<File>>,
}

impl QueryCacheWriter {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        root: &Path,
        checkpoint: &Path,
        data_root: &Path,
        table: &[f32],
        vocab: usize,
        width: usize,
        query_space: impl Into<String>,
    ) -> Result<Self> {
        let query_space = query_space.into();
        if !matches!(query_space.as_str(), "f32" | "q12-f32") {
            return Err(invalid("query cache query space differs"));
        }
        if table.len() != vocab.saturating_mul(width) {
            return Err(invalid("query cache table shape differs"));
        }
        if table.iter().any(|value| !value.is_finite()) {
            return Err(invalid("query cache table holds a non-finite value"));
        }
        fs::create_dir(root)?;
        let root = fs::canonicalize(root)?;
        let (run_config, run_config_sha256) = match run_config(checkpoint)? {
            Some(path) => (
                Some(fs::canonicalize(&path)?.display().to_string()),
                Some(file_sha256(&path)?),
            ),
            None => (None, None),
        };
        let table_file = "table.f32".to_owned();
        write_f32(&root.join(&table_file), table)?;
        Ok(Self {
            root,
            manifest: QueryCacheManifest {
                schema: QUERY_CACHE_SCHEMA.to_owned(),
                checkpoint: fs::canonicalize(checkpoint)?.display().to_string(),
                data_root: fs::canonicalize(data_root)?.display().to_string(),
                tokenizer_sha256: crate::hamilton_sheet::PINNED_TOKENIZER_SHA256.to_owned(),
                checkpoint_files: file_inventory(checkpoint)?,
                run_config,
                run_config_sha256,
                table_file,
                table_sha256: sha256_f32(table),
                vocab,
                width,
                query_space,
                splits: Vec::new(),
            },
            started: BTreeSet::new(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn start_split(
        &mut self,
        split: &str,
        label: &str,
        memory: bool,
        context: usize,
    ) -> Result<QuerySplitWriter> {
        safe_part(split, "split")?;
        safe_part(label, "label")?;
        let key = (split.to_owned(), label.to_owned());
        if !self.started.insert(key) {
            return Err(invalid(format!(
                "query cache split {split}/{label} is written twice"
            )));
        }
        let directory = self.root.join(label);
        if !directory.exists() {
            fs::create_dir(&directory)?;
        }
        let query_file = format!("{label}/{split}.queries.f32");
        let query_tmp = self.root.join(format!("{label}/.{split}.queries.f32.tmp"));
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&query_tmp)?;
        Ok(QuerySplitWriter {
            root: self.root.clone(),
            split: split.to_owned(),
            label: label.to_owned(),
            memory,
            context,
            width: self.manifest.width,
            rows: 0,
            query_file,
            query_tmp,
            writer: Some(BufWriter::new(file)),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn finish_split(
        &mut self,
        writer: QuerySplitWriter,
        rows: &CachedRows,
        tokens_sha256: String,
        mask_sha256: String,
    ) -> Result<()> {
        if writer.rows != rows.len() {
            return Err(invalid(format!(
                "query cache {} has {} query rows and {} decision rows",
                writer.split,
                writer.rows,
                rows.len()
            )));
        }
        writer.finish(&mut self.manifest, rows, tokens_sha256, mask_sha256)
    }

    pub fn finish(self) -> Result<PathBuf> {
        if self.manifest.splits.is_empty() {
            return Err(invalid("query cache has no splits"));
        }
        let path = self.root.join("manifest.json");
        let manifest = serde_json::to_vec_pretty(&self.manifest)?;
        let seal = serde_json::to_vec_pretty(&QueryCacheSeal {
            schema: QUERY_CACHE_SEAL_SCHEMA.to_owned(),
            manifest_sha256: sha256_bytes(&manifest),
        })?;
        write_new(&path, &manifest)?;
        write_new(&self.root.join("seal.json"), &seal)?;
        Ok(path)
    }
}

impl QuerySplitWriter {
    pub fn write_tensor(&mut self, query: &Tensor) -> Result<usize> {
        if query.dtype() != DType::F32 {
            return Err(invalid("query cache wants F32 query rows"));
        }
        let (rows, width) = query.dims2()?;
        if width != self.width {
            return Err(invalid("query cache query width differs"));
        }
        let values = query.to_vec2::<f32>()?;
        for row in &values {
            self.write_row(row)?;
        }
        Ok(rows)
    }

    fn write_row(&mut self, row: &[f32]) -> Result<()> {
        if row.len() != self.width || row.iter().any(|value| !value.is_finite()) {
            return Err(invalid(
                "query cache row is non-finite or has the wrong width",
            ));
        }
        let writer = self
            .writer
            .as_mut()
            .ok_or_else(|| invalid("query cache split is already closed"))?;
        for value in row {
            writer.write_all(&value.to_le_bytes())?;
        }
        self.rows += 1;
        Ok(())
    }

    fn finish(
        mut self,
        manifest: &mut QueryCacheManifest,
        rows: &CachedRows,
        tokens_sha256: String,
        mask_sha256: String,
    ) -> Result<()> {
        let mut writer = self
            .writer
            .take()
            .ok_or_else(|| invalid("query cache split is already closed"))?;
        writer.flush()?;
        drop(writer);
        let query_path = self.root.join(&self.query_file);
        fs::rename(&self.query_tmp, &query_path)?;
        let query_sha256 = file_sha256(&query_path)?;

        let rows_file = format!("{}/{}.rows.bin", self.label, self.split);
        let rows_tmp = self
            .root
            .join(format!("{}/.{}.rows.bin.tmp", self.label, self.split));
        let mut rows_writer = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&rows_tmp)?,
        );
        write_cached_rows(&mut rows_writer, rows)?;
        rows_writer.flush()?;
        drop(rows_writer);
        let rows_path = self.root.join(&rows_file);
        fs::rename(&rows_tmp, &rows_path)?;
        let rows_sha256 = file_sha256(&rows_path)?;

        manifest.splits.push(QueryCacheSplit {
            split: self.split,
            label: self.label,
            memory: self.memory,
            context: self.context,
            rows: self.rows,
            tokens_sha256,
            mask_sha256,
            query_file: self.query_file,
            query_sha256,
            rows_file,
            rows_sha256,
        });
        Ok(())
    }
}

pub struct QueryCache {
    pub root: PathBuf,
    pub manifest: QueryCacheManifest,
    pub manifest_sha256: String,
    table: Vec<f32>,
}

impl QueryCache {
    pub fn open(root: &Path) -> Result<Self> {
        let root = fs::canonicalize(root)?;
        let manifest_bytes = fs::read(root.join("manifest.json"))?;
        let seal_bytes = fs::read(root.join("seal.json"))?;
        let seal: QueryCacheSeal = serde_json::from_slice(&seal_bytes)?;
        if seal.schema != QUERY_CACHE_SEAL_SCHEMA
            || seal.manifest_sha256 != sha256_bytes(&manifest_bytes)
        {
            return Err(invalid("query cache manifest seal differs"));
        }
        let manifest: QueryCacheManifest = serde_json::from_slice(&manifest_bytes)?;
        if manifest.schema != QUERY_CACHE_SCHEMA
            || manifest.checkpoint.is_empty()
            || manifest.data_root.is_empty()
            || manifest.tokenizer_sha256 != crate::hamilton_sheet::PINNED_TOKENIZER_SHA256
            || manifest.vocab == 0
            || manifest.width == 0
            || manifest.table_file.is_empty()
            || manifest.splits.is_empty()
            || !matches!(manifest.query_space.as_str(), "f32" | "q12-f32")
        {
            return Err(invalid("query cache manifest identity differs"));
        }
        safe_relative(&manifest.table_file)?;
        let mut split_keys = BTreeSet::new();
        for split in &manifest.splits {
            safe_part(&split.split, "split")?;
            safe_part(&split.label, "label")?;
            if split.context == 0 || !split_keys.insert((&split.split, split.memory)) {
                return Err(invalid("query cache split inventory differs"));
            }
        }
        let table_bytes = fs::read(root.join(&manifest.table_file))?;
        let table = decode_f32(&table_bytes)?;
        if table.len() != manifest.vocab.saturating_mul(manifest.width)
            || table.iter().any(|value| !value.is_finite())
            || sha256_bytes(&table_bytes) != manifest.table_sha256
        {
            return Err(invalid("query cache table identity differs"));
        }
        Ok(Self {
            root,
            manifest,
            manifest_sha256: sha256_bytes(&manifest_bytes),
            table,
        })
    }

    pub fn split(&self, split: &str, memory: bool) -> Result<CachedSplit> {
        let metadata = self
            .manifest
            .splits
            .iter()
            .find(|entry| entry.split == split && entry.memory == memory)
            .ok_or_else(|| {
                invalid(format!(
                    "query cache has no {split} split with memory={memory}"
                ))
            })?
            .clone();
        let query_bytes = read_checked(&self.root, &metadata.query_file, &metadata.query_sha256)?;
        let row_bytes = read_checked(&self.root, &metadata.rows_file, &metadata.rows_sha256)?;
        let queries = decode_f32(&query_bytes)?;
        if queries.len() != metadata.rows.saturating_mul(self.manifest.width) {
            return Err(invalid("query cache query row count differs"));
        }
        let rows = decode_cached_rows(&row_bytes, metadata.rows, self.manifest.vocab)?;
        Ok(CachedSplit {
            metadata,
            queries,
            rows,
        })
    }

    pub fn table(&self) -> &[f32] {
        &self.table
    }
}

pub fn write_cached_rows(writer: &mut impl Write, rows: &CachedRows) -> Result<()> {
    let len = rows.len();
    if rows.positions.len() != len
        || rows.nearest.len() != len
        || rows.writer.len() != len
        || rows.opens.len() != len
        || rows.copies.len() != len
    {
        return Err(invalid(
            "query cache decision arrays have different lengths",
        ));
    }
    if rows
        .positions
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .len()
        != len
    {
        return Err(invalid("query cache split has duplicate positions"));
    }
    writer.write_all(ROW_MAGIC)?;
    writer.write_all(&(len as u64).to_le_bytes())?;
    for values in [&rows.targets, &rows.positions, &rows.nearest, &rows.writer] {
        for value in values {
            writer.write_all(&value.to_le_bytes())?;
        }
    }
    writer.write_all(&rows.opens)?;
    writer.write_all(&rows.copies)?;
    Ok(())
}

pub fn decode_cached_rows(bytes: &[u8], expected_rows: usize, vocab: usize) -> Result<CachedRows> {
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
    let opens = bytes[at..at + expected_rows].to_vec();
    at += expected_rows;
    let copies = bytes[at..at + expected_rows].to_vec();
    if opens.iter().chain(&copies).any(|&flag| flag > 1) {
        return Err(invalid("query cache row flag differs"));
    }
    if positions.iter().copied().collect::<BTreeSet<_>>().len() != expected_rows {
        return Err(invalid("query cache split has duplicate positions"));
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
    tokens: Vec<u32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CentroidFit {
    pub fitted_rows: usize,
    pub observations: usize,
    pub changed_rows: usize,
    pub changed_coordinates: usize,
    pub displacement_l1: f64,
}

pub fn evaluate_centroid_cache(
    root: &Path,
    train_split: &str,
    panel_splits: &[String],
    memory: bool,
    minimum_observations: usize,
) -> Result<serde_json::Value> {
    if panel_splits.is_empty() {
        return Err(invalid("query cache evaluation needs at least one panel"));
    }
    if minimum_observations == 0 {
        return Err(invalid("query cache minimum observations is zero"));
    }
    if panel_splits.iter().collect::<BTreeSet<_>>().len() != panel_splits.len() {
        return Err(invalid("query cache evaluation repeats a panel"));
    }
    if panel_splits.iter().any(|panel| panel == train_split) {
        return Err(invalid("query cache evaluation uses train as a panel"));
    }
    let cache = QueryCache::open(root)?;
    let train = cache.split(train_split, memory)?;
    if train.queries.is_empty() {
        return Err(invalid("query cache train split has no rows"));
    }
    let (centroid, shuffled, fit, shuffled_changed) = fit_centroid_and_control(
        cache.table(),
        cache.manifest.vocab,
        cache.manifest.width,
        &train,
        minimum_observations,
    )?;

    let mut panels = Vec::new();
    let mut all_identity_exact = true;
    for name in panel_splits {
        let panel = cache.split(name, memory)?;
        if panel.queries.is_empty() {
            return Err(invalid(format!("query cache panel {name} has no rows")));
        }
        let original = evaluate_table(
            cache.table(),
            cache.manifest.vocab,
            cache.manifest.width,
            &panel,
        )?;
        let identity_changed = original
            .tokens
            .iter()
            .zip(&panel.rows.nearest)
            .filter(|(left, right)| left != right)
            .count();
        let identity_exact = identity_changed == 0;
        all_identity_exact &= identity_exact;
        let centroid_eval = evaluate_table(
            &centroid,
            cache.manifest.vocab,
            cache.manifest.width,
            &panel,
        )?;
        let shuffled_eval = evaluate_table(
            &shuffled,
            cache.manifest.vocab,
            cache.manifest.width,
            &panel,
        )?;
        panels.push(serde_json::json!({
            "split": name,
            "rows": panel.rows.len(),
            "tokens_sha256": panel.metadata.tokens_sha256,
            "mask_sha256": panel.metadata.mask_sha256,
            "identity": {
                "exact": identity_exact,
                "changed_from_frozen": identity_changed,
                "decision_sha256": original.counts.decision_sha256,
            },
            "original": original.counts,
            "centroid": centroid_eval.counts,
            "shuffled_target": shuffled_eval.counts,
            "centroid_vs_original": paired(&centroid_eval.top1_rows, &original.top1_rows),
            "centroid_vs_shuffled": paired(&centroid_eval.top1_rows, &shuffled_eval.top1_rows),
            "centroid_top8_not_below_original":
                centroid_eval.counts.top8_correct >= original.counts.top8_correct,
        }));
    }
    Ok(serde_json::json!({
        "schema": QUERY_CACHE_EVALUATION_SCHEMA,
        "status": if all_identity_exact { "COMPLETE" } else { "CONTROL_FAULT" },
        "cache": {
            "root": cache.root,
            "manifest_sha256": cache.manifest_sha256,
            "checkpoint": cache.manifest.checkpoint,
            "checkpoint_files": cache.manifest.checkpoint_files,
            "run_config": cache.manifest.run_config,
            "run_config_sha256": cache.manifest.run_config_sha256,
            "table_sha256": cache.manifest.table_sha256,
            "vocab": cache.manifest.vocab,
            "width": cache.manifest.width,
            "query_space": cache.manifest.query_space,
        },
        "train": {
            "split": train_split,
            "memory": memory,
            "rows": train.rows.len(),
            "tokens_sha256": train.metadata.tokens_sha256,
            "mask_sha256": train.metadata.mask_sha256,
        },
        "minimum_observations": minimum_observations,
        "fit": fit,
        "shuffled_target": {
            "changed_rows": shuffled_changed,
        },
        "panels": panels,
    }))
}

fn fit_centroid_and_control(
    original: &[f32],
    vocab: usize,
    width: usize,
    train: &CachedSplit,
    minimum: usize,
) -> Result<(Vec<f32>, Vec<f32>, CentroidFit, usize)> {
    let mut counts = vec![0usize; vocab];
    let mut sums = vec![0f64; vocab.saturating_mul(width)];
    for (row, &target) in train.queries.chunks_exact(width).zip(&train.rows.targets) {
        let target = target as usize;
        counts[target] += 1;
        let base = target.saturating_mul(width);
        for (coordinate, &value) in row.iter().enumerate() {
            sums[base + coordinate] += f64::from(value);
        }
    }
    let fitted = (0..vocab)
        .filter(|&token| counts[token] >= minimum)
        .collect::<Vec<_>>();
    if fitted.is_empty() {
        return Err(invalid("query cache centroid has no eligible rows"));
    }
    let fitted_observations = fitted.iter().map(|&token| counts[token]).sum();
    let mut table = original.to_vec();
    let mut centroids = vec![Vec::<f32>::new(); vocab];
    let mut changed_rows = 0usize;
    let mut changed_coordinates = 0usize;
    let mut displacement_l1 = 0f64;
    for &token in &fitted {
        let base = token.saturating_mul(width);
        let mut row = Vec::with_capacity(width);
        let mut row_changed = false;
        for coordinate in 0..width {
            let value = (sums[base + coordinate] / counts[token] as f64) as f32;
            if !value.is_finite() {
                return Err(invalid("query cache centroid is non-finite"));
            }
            let old = table[base + coordinate];
            if old != value {
                changed_coordinates += 1;
                row_changed = true;
            }
            displacement_l1 += f64::from((old - value).abs());
            table[base + coordinate] = value;
            row.push(value);
        }
        changed_rows += usize::from(row_changed);
        centroids[token] = row;
    }

    let mut by_count = BTreeMap::<usize, Vec<usize>>::new();
    for &token in &fitted {
        by_count.entry(counts[token]).or_default().push(token);
    }
    let mut shuffled = original.to_vec();
    let mut shuffled_changed = 0usize;
    for tokens in by_count.values() {
        if tokens.len() < 2 {
            continue;
        }
        for (index, &target) in tokens.iter().enumerate() {
            let source = tokens[(index + 1) % tokens.len()];
            let target_base = target.saturating_mul(width);
            shuffled[target_base..target_base + width].copy_from_slice(&centroids[source]);
            shuffled_changed += 1;
        }
    }
    Ok((
        table,
        shuffled,
        CentroidFit {
            fitted_rows: fitted.len(),
            observations: fitted_observations,
            changed_rows,
            changed_coordinates,
            displacement_l1,
        },
        shuffled_changed,
    ))
}

fn evaluate_table(
    table: &[f32],
    vocab: usize,
    width: usize,
    split: &CachedSplit,
) -> Result<Decisions> {
    let blocks = split
        .queries
        .par_chunks(width.saturating_mul(EVALUATION_QUERY_BLOCK))
        .enumerate()
        .map(|(block, queries)| {
            let rows = queries.len() / width;
            let mut best = vec![Vec::<(f32, u32)>::with_capacity(8); rows];
            for token in 0..vocab {
                let candidate_row = &table[token * width..(token + 1) * width];
                for (row, query) in queries.chunks_exact(width).enumerate() {
                    let mut distance = 0f32;
                    for (&left, &right) in query.iter().zip(candidate_row) {
                        distance += (left - right).abs();
                    }
                    insert_best(&mut best[row], distance / width as f32, token as u32);
                }
            }
            (block, best)
        })
        .collect::<Vec<_>>();
    let decoded = blocks
        .into_iter()
        .flat_map(|(block, best)| {
            let start = block * EVALUATION_QUERY_BLOCK;
            best.into_iter().enumerate().map(move |(row, candidates)| {
                let index = start + row;
                let top1 = candidates[0].1;
                let target = split.rows.targets[index];
                (
                    top1,
                    top1 == target,
                    candidates.iter().any(|candidate| candidate.1 == target),
                )
            })
        })
        .collect::<Vec<_>>();
    let mut tokens = Vec::with_capacity(decoded.len());
    let mut top1_rows = Vec::with_capacity(decoded.len());
    let mut bytes = Vec::with_capacity(decoded.len() * 4);
    let mut top8_correct = 0usize;
    for (token, top1, top8) in decoded {
        tokens.push(token);
        top1_rows.push(top1);
        bytes.extend_from_slice(&token.to_le_bytes());
        top8_correct += usize::from(top8);
    }
    let top1_correct = top1_rows.iter().filter(|&&value| value).count();
    let changed_from_frozen = tokens
        .iter()
        .zip(&split.rows.nearest)
        .filter(|(left, right)| left != right)
        .count();
    Ok(Decisions {
        counts: ReadoutCounts {
            rows: tokens.len(),
            top1_correct,
            top8_correct,
            changed_from_frozen,
            decision_sha256: sha256_bytes(&bytes),
        },
        top1_rows,
        tokens,
    })
}

fn insert_best(best: &mut Vec<(f32, u32)>, distance: f32, token: u32) {
    let candidate = (distance, token);
    let position = best
        .iter()
        .position(|current| {
            candidate
                .0
                .total_cmp(&current.0)
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
    let candidate_sum = candidate.iter().filter(|&&value| value).count() as u64;
    let control_sum = control.iter().filter(|&&value| value).count() as u64;
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

fn safe_part(value: &str, what: &str) -> Result<()> {
    if value.is_empty()
        || value.starts_with('.')
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(invalid(format!(
            "query cache {what} is not one safe file part"
        )));
    }
    Ok(())
}

fn write_f32(path: &Path, values: &[f32]) -> Result<()> {
    let mut writer = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(path)?);
    for value in values {
        writer.write_all(&value.to_le_bytes())?;
    }
    writer.flush()?;
    Ok(())
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
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

fn sha256_f32(values: &[f32]) -> String {
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    sha256_bytes(&bytes)
}

fn read_checked(root: &Path, file: &str, expected: &str) -> Result<Vec<u8>> {
    safe_relative(file)?;
    let bytes = fs::read(root.join(file))?;
    if sha256_bytes(&bytes) != expected {
        return Err(invalid(format!("query cache file {file} hash differs")));
    }
    Ok(bytes)
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

fn file_sha256(path: &Path) -> Result<String> {
    Ok(sha256_bytes(&fs::read(path)?))
}

fn file_inventory(root: &Path) -> Result<BTreeMap<String, String>> {
    fn walk(root: &Path, relative: &Path, out: &mut BTreeMap<String, String>) -> Result<()> {
        let mut entries =
            fs::read_dir(root.join(relative))?.collect::<std::io::Result<Vec<_>>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let name = entry.file_name();
            let path = relative.join(&name);
            let metadata = fs::symlink_metadata(entry.path())?;
            if metadata.file_type().is_symlink() {
                return Err(invalid(format!(
                    "query cache checkpoint contains symlink {}",
                    path.display()
                )));
            }
            if metadata.is_dir() {
                walk(root, &path, out)?;
            } else if metadata.is_file() {
                out.insert(
                    path.to_string_lossy().replace('\\', "/"),
                    file_sha256(&entry.path())?,
                );
            }
        }
        Ok(())
    }
    let mut out = BTreeMap::new();
    walk(root, Path::new(""), &mut out)?;
    Ok(out)
}

fn run_config(checkpoint: &Path) -> Result<Option<PathBuf>> {
    let candidates = [
        Some(checkpoint.join("config.json")),
        checkpoint
            .parent()
            .and_then(Path::parent)
            .map(|run| run.join("config.json")),
    ];
    for path in candidates.into_iter().flatten() {
        if !path.is_file() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)
            .map_err(|error| invalid(format!("{}: {error}", path.display())))?;
        if value.get("steps").is_some() {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("uor-r4-query-cache-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        path
    }

    fn fixture(name: &str) -> (PathBuf, PathBuf, QueryCacheManifest) {
        let root = temp(name);
        let checkpoint = root.join("checkpoint");
        fs::create_dir_all(&checkpoint).unwrap();
        fs::write(checkpoint.join("config.json"), b"{}").unwrap();
        let cache_root = root.join("cache");
        let table = [0f32, 0., 10., 10., -10., -10.];
        let mut cache =
            QueryCacheWriter::new(&cache_root, &checkpoint, &root, &table, 3, 2, "f32").unwrap();
        let train_rows = CachedRows {
            targets: vec![0, 0, 1, 1, 2, 2],
            positions: vec![1, 2, 3, 4, 5, 6],
            nearest: vec![0, 0, 1, 1, 2, 2],
            writer: vec![0, 0, 1, 1, 2, 2],
            opens: vec![1, 0, 0, 0, 0, 0],
            copies: vec![0, 1, 1, 0, 0, 0],
        };
        let mut train = cache.start_split("train", "memory", true, 4).unwrap();
        for row in [
            [0.1, 0.2],
            [0.3, 0.1],
            [9.9, 10.1],
            [10.2, 10.0],
            [-9.9, -10.1],
            [-10.0, -10.2],
        ] {
            train.write_row(&row).unwrap();
        }
        cache
            .finish_split(
                train,
                &train_rows,
                "train-tokens".into(),
                "train-mask".into(),
            )
            .unwrap();
        let panel_rows = CachedRows {
            targets: vec![0, 1, 2],
            positions: vec![1, 2, 3],
            nearest: vec![0, 1, 2],
            writer: vec![0, 1, 2],
            opens: vec![1, 0, 0],
            copies: vec![0, 1, 0],
        };
        let mut panel = cache.start_split("p-test", "memory", true, 4).unwrap();
        for row in [[0.2, 0.2], [9.8, 9.8], [-10.2, -10.2]] {
            panel.write_row(&row).unwrap();
        }
        cache
            .finish_split(
                panel,
                &panel_rows,
                "panel-tokens".into(),
                "panel-mask".into(),
            )
            .unwrap();
        let manifest: QueryCacheManifest =
            serde_json::from_slice(&fs::read(cache.finish().unwrap()).unwrap()).unwrap();
        (root, cache_root, manifest)
    }

    #[test]
    fn cache_round_trips_and_reproduces_the_original_decisions() {
        let (root, cache_root, manifest) = fixture("roundtrip");
        assert_eq!(manifest.schema, QUERY_CACHE_SCHEMA);
        let report =
            evaluate_centroid_cache(&cache_root, "train", &["p-test".to_owned()], true, 2).unwrap();
        assert_eq!(report["status"], "COMPLETE");
        assert_eq!(report["panels"][0]["identity"]["exact"], true);
        assert_eq!(report["fit"]["fitted_rows"], 3);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn blocked_evaluation_preserves_panel_order_and_top_eight_ties() {
        let width = 1;
        let rows = EVALUATION_QUERY_BLOCK * 2 + 1;
        let queries = (0..rows)
            .map(|row| (row % 3) as f32 * 10.0)
            .collect::<Vec<_>>();
        let targets = (0..rows).map(|row| (row % 3) as u32).collect::<Vec<_>>();
        let nearest = targets.clone();
        let split = CachedSplit {
            metadata: QueryCacheSplit {
                split: "panel".into(),
                label: "memory".into(),
                memory: true,
                context: 2,
                rows,
                tokens_sha256: "tokens".into(),
                mask_sha256: "mask".into(),
                query_file: "queries".into(),
                query_sha256: "query".into(),
                rows_file: "rows".into(),
                rows_sha256: "rows".into(),
            },
            queries,
            rows: CachedRows {
                targets: targets.clone(),
                positions: (0..rows as u32).collect(),
                nearest,
                writer: targets,
                opens: vec![0; rows],
                copies: vec![0; rows],
            },
        };
        let decisions = evaluate_table(&[0.0, 10.0, 20.0], 3, width, &split).unwrap();
        assert_eq!(
            decisions.tokens,
            (0..rows).map(|row| (row % 3) as u32).collect::<Vec<_>>()
        );
        assert_eq!(decisions.counts.top1_correct, rows);
        assert_eq!(decisions.counts.changed_from_frozen, 0);

        let mut best = Vec::new();
        for token in [4, 2, 3, 1] {
            insert_best(&mut best, 1.0, token);
        }
        assert_eq!(
            best.iter().map(|candidate| candidate.1).collect::<Vec<_>>(),
            [1, 2, 3, 4]
        );
    }

    #[test]
    fn a_changed_query_bit_is_refused() {
        let (root, cache_root, _) = fixture("query-bit");
        let query = cache_root.join("memory/train.queries.f32");
        let mut bytes = fs::read(&query).unwrap();
        bytes[0] ^= 1;
        fs::write(&query, bytes).unwrap();
        assert!(
            QueryCache::open(&cache_root)
                .unwrap()
                .split("train", true)
                .is_err()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_changed_table_byte_is_refused() {
        let (root, cache_root, _) = fixture("table-bit");
        let table = cache_root.join("table.f32");
        let mut bytes = fs::read(&table).unwrap();
        bytes[0] ^= 1;
        fs::write(&table, bytes).unwrap();
        assert!(QueryCache::open(&cache_root).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_changed_decision_byte_is_refused() {
        let (root, cache_root, _) = fixture("decision-bit");
        let rows = cache_root.join("memory/p-test.rows.bin");
        let mut bytes = fs::read(&rows).unwrap();
        bytes[16] ^= 1;
        fs::write(&rows, bytes).unwrap();
        assert!(
            QueryCache::open(&cache_root)
                .unwrap()
                .split("p-test", true)
                .is_err()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_changed_manifest_is_refused_by_its_seal() {
        let (root, cache_root, _) = fixture("manifest-seal");
        let manifest = cache_root.join("manifest.json");
        let mut bytes = fs::read(&manifest).unwrap();
        let at = bytes
            .windows("train".len())
            .position(|window| window == b"train")
            .unwrap();
        bytes[at] = b'T';
        fs::write(&manifest, bytes).unwrap();
        assert!(QueryCache::open(&cache_root).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_split_captured_with_the_other_memory_mode_is_refused() {
        let (root, cache_root, _) = fixture("memory-mode");
        let cache = QueryCache::open(&cache_root).unwrap();
        assert!(cache.split("train", false).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn duplicate_positions_are_refused_before_the_split_is_written() {
        let (root, _, _) = fixture("duplicate");
        let checkpoint = root.join("checkpoint");
        let cache_root = root.join("duplicate-cache");
        let table = [0f32, 0., 1., 1.];
        let mut cache =
            QueryCacheWriter::new(&cache_root, &checkpoint, &root, &table, 2, 2, "f32").unwrap();
        let rows = CachedRows {
            targets: vec![0, 1],
            positions: vec![7, 7],
            nearest: vec![0, 1],
            writer: vec![0, 1],
            opens: vec![0, 0],
            copies: vec![0, 0],
        };
        let mut split = cache.start_split("train", "memory", true, 4).unwrap();
        split.write_row(&[0., 0.]).unwrap();
        split.write_row(&[1., 1.]).unwrap();
        assert!(
            cache
                .finish_split(split, &rows, "tokens".into(), "mask".into())
                .is_err()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn an_existing_cache_root_is_never_overwritten() {
        let (root, cache_root, _) = fixture("existing");
        let checkpoint = root.join("checkpoint");
        let table = [0f32, 0.];
        assert!(
            QueryCacheWriter::new(&cache_root, &checkpoint, &root, &table, 1, 2, "f32").is_err()
        );
        fs::remove_dir_all(root).unwrap();
    }
}
