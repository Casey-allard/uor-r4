//! Hash-pinned regression over multiple trained ask-offset checkpoints.
//!
//! This is a host-only instrument. It reads archived float checkpoints and
//! compares the ask-selection geometry with stored expectations; it does not
//! train, export or serve a model.

use std::collections::BTreeSet;
use std::fs::File;
use std::io::Read;
use std::ops::Range;
use std::path::{Component, Path, PathBuf};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::ask_offset_audit::{AskOffsetRelationReading, TrainedAskOffsets};

pub const ASK_ARM_REGRESSION_MANIFEST_SCHEMA: &str = "uor-r4.cpu-ask-arm-regression-manifest/1";
pub const ASK_ARM_REGRESSION_REPORT_SCHEMA: &str = "uor-r4.cpu-ask-arm-regression/1";
pub const ASK_ARM_REGRESSION_DEFAULT_WORKERS: usize = 8;
pub const ASK_ARM_REGRESSION_MAX_WORKERS: usize = 32;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskArmRegressionManifest {
    pub schema: String,
    pub source_store: String,
    pub remote_root: String,
    pub archives: Vec<AskArmArchive>,
    pub settings: AskArmSettings,
    pub arms: Vec<AskArmCase>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskArmArchive {
    pub path: String,
    pub sha256: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskArmSettings {
    pub relation_start: u32,
    pub relation_end: u32,
    pub query_shift: u32,
}

impl AskArmSettings {
    fn relations(self) -> Result<Range<u32>, String> {
        if self.relation_start >= self.relation_end {
            return Err("relation_start must be less than relation_end".to_owned());
        }
        if self.relation_end.checked_add(self.query_shift).is_none() {
            return Err("query_shift overflows the relation range".to_owned());
        }
        Ok(self.relation_start..self.relation_end)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskArmCase {
    pub id: String,
    pub label: String,
    pub seed: u32,
    pub archive: String,
    pub checkpoint: String,
    pub model_sha256: String,
    pub hamming_sha256: String,
    pub expected: AskArmSummary,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskArmSummary {
    pub vocab: usize,
    pub width: usize,
    pub heads: usize,
    pub ask_heads: usize,
    pub depths: Vec<usize>,
    pub relations: usize,
    pub cells: usize,
    pub zero_exact_codes: usize,
    pub trained_exact_codes: usize,
    pub zero_total_bits: u64,
    pub trained_total_bits: u64,
    pub improvement_bits: i64,
    pub improved_cells: usize,
    pub regressed_cells: usize,
    pub zero_target_nearest: usize,
    pub trained_target_nearest: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskArmArchiveReading {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskArmReading {
    pub id: String,
    pub label: String,
    pub seed: u32,
    pub archive: String,
    pub checkpoint: String,
    pub model_sha256: String,
    pub hamming_sha256: String,
    pub observed: Option<AskArmSummary>,
    pub expected: AskArmSummary,
    pub mismatches: Vec<String>,
    pub error: Option<String>,
    pub passed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskArmRegressionReport {
    pub schema: String,
    pub manifest_sha256: String,
    pub executable_sha256: String,
    pub source_store: String,
    pub remote_root: String,
    pub settings: AskArmSettings,
    pub workers: usize,
    pub host: AskArmHost,
    pub archives: Vec<AskArmArchiveReading>,
    pub arms: Vec<AskArmReading>,
    pub passed: bool,
    pub claim: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AskArmHost {
    pub cpu: String,
    pub logical_cpus: usize,
    pub os: String,
}

impl AskArmRegressionReport {
    pub fn failures(&self) -> usize {
        self.arms.iter().filter(|arm| !arm.passed).count()
    }

    pub fn source_failures(&self) -> usize {
        self.arms.iter().filter(|arm| arm.error.is_some()).count()
    }

    pub fn regression_failures(&self) -> usize {
        self.arms
            .iter()
            .filter(|arm| arm.error.is_none() && !arm.mismatches.is_empty())
            .count()
    }
}

pub fn parse_manifest(bytes: &[u8]) -> Result<AskArmRegressionManifest, String> {
    let manifest: AskArmRegressionManifest =
        serde_json::from_slice(bytes).map_err(|error| format!("manifest JSON: {error}"))?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

pub fn run_ask_arm_regression(
    root: &Path,
    manifest_bytes: &[u8],
    workers: usize,
    executable_sha256: &str,
) -> Result<AskArmRegressionReport, String> {
    validate_sha256(executable_sha256, "executable")?;
    if workers == 0 || workers > ASK_ARM_REGRESSION_MAX_WORKERS {
        return Err(format!(
            "workers must be within 1..={ASK_ARM_REGRESSION_MAX_WORKERS}"
        ));
    }
    let manifest = parse_manifest(manifest_bytes)?;
    let canonical_root = root
        .canonicalize()
        .map_err(|error| format!("{}: {error}", root.display()))?;
    if !canonical_root.is_dir() {
        return Err(format!("{} is not a directory", canonical_root.display()));
    }

    let archives = manifest
        .archives
        .iter()
        .map(|archive| verify_archive(&canonical_root, archive))
        .collect::<Result<Vec<_>, _>>()?;
    let relations = manifest.settings.relations()?;
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(workers)
        .build()
        .map_err(|error| format!("worker pool: {error}"))?;
    let arms = pool.install(|| {
        manifest
            .arms
            .par_iter()
            .map(|case| {
                run_case(
                    &canonical_root,
                    relations.clone(),
                    manifest.settings.query_shift,
                    case,
                )
            })
            .collect::<Vec<_>>()
    });
    let passed = arms.iter().all(|arm| arm.passed);
    Ok(AskArmRegressionReport {
        schema: ASK_ARM_REGRESSION_REPORT_SCHEMA.to_owned(),
        manifest_sha256: hex_sha256(manifest_bytes),
        executable_sha256: executable_sha256.to_owned(),
        source_store: manifest.source_store,
        remote_root: manifest.remote_root,
        settings: manifest.settings,
        workers,
        host: host_identity(),
        archives,
        arms,
        passed,
        claim: "hash-pinned real-checkpoint ask-selection regression; not B2, carry, capability or serving evidence"
            .to_owned(),
    })
}

pub fn summarize_ask_offsets(
    offsets: &TrainedAskOffsets,
    relations: &[AskOffsetRelationReading],
) -> AskArmSummary {
    let cells = relations
        .iter()
        .flat_map(|relation| relation.cells.iter())
        .collect::<Vec<_>>();
    AskArmSummary {
        vocab: offsets.vocab(),
        width: offsets.width(),
        heads: offsets.heads(),
        ask_heads: offsets.ask_head_count(),
        depths: offsets.depths().collect(),
        relations: relations.len(),
        cells: cells.len(),
        zero_exact_codes: cells.iter().filter(|cell| cell.zero_bits == 0).count(),
        trained_exact_codes: cells.iter().filter(|cell| cell.trained_bits == 0).count(),
        zero_total_bits: cells.iter().map(|cell| u64::from(cell.zero_bits)).sum(),
        trained_total_bits: cells.iter().map(|cell| u64::from(cell.trained_bits)).sum(),
        improvement_bits: cells
            .iter()
            .map(|cell| i64::from(cell.improvement_bits))
            .sum(),
        improved_cells: cells
            .iter()
            .filter(|cell| cell.improvement_bits > 0)
            .count(),
        regressed_cells: cells
            .iter()
            .filter(|cell| cell.improvement_bits < 0)
            .count(),
        zero_target_nearest: cells.iter().filter(|cell| cell.zero_target_nearest).count(),
        trained_target_nearest: cells
            .iter()
            .filter(|cell| cell.trained_target_nearest)
            .count(),
    }
}

fn validate_manifest(manifest: &AskArmRegressionManifest) -> Result<(), String> {
    if manifest.schema != ASK_ARM_REGRESSION_MANIFEST_SCHEMA {
        return Err(format!(
            "manifest schema is {:?}, expected {ASK_ARM_REGRESSION_MANIFEST_SCHEMA:?}",
            manifest.schema
        ));
    }
    validate_store(&manifest.source_store)?;
    validate_store_path(&manifest.remote_root)?;
    manifest.settings.relations()?;
    if manifest.archives.is_empty() {
        return Err("manifest carries no archives".to_owned());
    }
    if manifest.arms.is_empty() {
        return Err("manifest carries no arms".to_owned());
    }

    let mut archive_paths = BTreeSet::new();
    for archive in &manifest.archives {
        validate_relative_path(&archive.path, "archive path")?;
        validate_sha256(&archive.sha256, "archive")?;
        if !archive_paths.insert(archive.path.as_str()) {
            return Err(format!("archive path {:?} is duplicated", archive.path));
        }
    }
    let mut ids = BTreeSet::new();
    let mut checkpoint_paths = BTreeSet::new();
    let mut checkpoint_pairs = BTreeSet::new();
    for arm in &manifest.arms {
        validate_id(&arm.id)?;
        if arm.label.trim().is_empty() {
            return Err(format!("arm {:?} has an empty label", arm.id));
        }
        if arm.seed == 0 {
            return Err(format!("arm {:?} has seed 0", arm.id));
        }
        if !ids.insert(arm.id.as_str()) {
            return Err(format!("arm id {:?} is duplicated", arm.id));
        }
        validate_relative_path(&arm.checkpoint, "checkpoint path")?;
        validate_sha256(&arm.model_sha256, "model")?;
        validate_sha256(&arm.hamming_sha256, "hamming")?;
        if !checkpoint_paths.insert(arm.checkpoint.as_str()) {
            return Err(format!(
                "checkpoint path {:?} is duplicated",
                arm.checkpoint
            ));
        }
        if !checkpoint_pairs.insert((arm.model_sha256.as_str(), arm.hamming_sha256.as_str())) {
            return Err(format!(
                "arm {:?} repeats checkpoint hashes model={} hamming={}",
                arm.id, arm.model_sha256, arm.hamming_sha256
            ));
        }
        if !archive_paths.contains(arm.archive.as_str()) {
            return Err(format!(
                "arm {:?} names unlisted archive {:?}",
                arm.id, arm.archive
            ));
        }
        if arm.expected.relations != manifest.settings.relations()?.len() {
            return Err(format!(
                "arm {:?} expects {} relations, settings name {}",
                arm.id,
                arm.expected.relations,
                manifest.settings.relations()?.len()
            ));
        }
    }
    Ok(())
}

fn verify_archive(root: &Path, archive: &AskArmArchive) -> Result<AskArmArchiveReading, String> {
    let path = resolve_under_root(root, &archive.path)?;
    let (sha256, bytes) = file_sha256(&path)?;
    if sha256 != archive.sha256 {
        return Err(format!(
            "{} archive SHA-256 differs: got {sha256}",
            archive.path
        ));
    }
    Ok(AskArmArchiveReading {
        path: archive.path.clone(),
        sha256,
        bytes,
    })
}

fn run_case(
    root: &Path,
    relations: Range<u32>,
    query_shift: u32,
    case: &AskArmCase,
) -> AskArmReading {
    let attempt = || -> Result<AskArmSummary, String> {
        let checkpoint = resolve_under_root(root, &case.checkpoint)?;
        let model_path = checkpoint.join("model.safetensors");
        let hamming_path = checkpoint.join("hamming_read.safetensors");
        let model = std::fs::read(&model_path)
            .map_err(|error| format!("{}: {error}", model_path.display()))?;
        let hamming = std::fs::read(&hamming_path)
            .map_err(|error| format!("{}: {error}", hamming_path.display()))?;
        let model_sha = hex_sha256(&model);
        if model_sha != case.model_sha256 {
            return Err(format!(
                "model.safetensors SHA-256 differs: got {model_sha}"
            ));
        }
        let hamming_sha = hex_sha256(&hamming);
        if hamming_sha != case.hamming_sha256 {
            return Err(format!(
                "hamming_read.safetensors SHA-256 differs: got {hamming_sha}"
            ));
        }
        let offsets = TrainedAskOffsets::from_bytes(&model, &hamming)?;
        let readings = offsets.audit_relations(relations, query_shift)?;
        Ok(summarize_ask_offsets(&offsets, &readings))
    };

    match attempt() {
        Ok(observed) => {
            let mismatches = compare_summary(&case.expected, &observed);
            AskArmReading {
                id: case.id.clone(),
                label: case.label.clone(),
                seed: case.seed,
                archive: case.archive.clone(),
                checkpoint: case.checkpoint.clone(),
                model_sha256: case.model_sha256.clone(),
                hamming_sha256: case.hamming_sha256.clone(),
                expected: case.expected.clone(),
                passed: mismatches.is_empty(),
                observed: Some(observed),
                mismatches,
                error: None,
            }
        }
        Err(error) => AskArmReading {
            id: case.id.clone(),
            label: case.label.clone(),
            seed: case.seed,
            archive: case.archive.clone(),
            checkpoint: case.checkpoint.clone(),
            model_sha256: case.model_sha256.clone(),
            hamming_sha256: case.hamming_sha256.clone(),
            expected: case.expected.clone(),
            observed: None,
            mismatches: Vec::new(),
            error: Some(error),
            passed: false,
        },
    }
}

fn compare_summary(expected: &AskArmSummary, observed: &AskArmSummary) -> Vec<String> {
    let mut mismatches = Vec::new();
    macro_rules! check {
        ($field:ident) => {
            if expected.$field != observed.$field {
                mismatches.push(format!(
                    "{} expected {:?}, observed {:?}",
                    stringify!($field),
                    expected.$field,
                    observed.$field
                ));
            }
        };
    }
    check!(vocab);
    check!(width);
    check!(heads);
    check!(ask_heads);
    check!(depths);
    check!(relations);
    check!(cells);
    check!(zero_exact_codes);
    check!(trained_exact_codes);
    check!(zero_total_bits);
    check!(trained_total_bits);
    check!(improvement_bits);
    check!(improved_cells);
    check!(regressed_cells);
    check!(zero_target_nearest);
    check!(trained_target_nearest);
    mismatches
}

fn resolve_under_root(root: &Path, relative: &str) -> Result<PathBuf, String> {
    validate_relative_path(relative, "input path")?;
    let path = root.join(relative);
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if !canonical.starts_with(root) {
        return Err(format!("{relative:?} resolves outside the supplied root"));
    }
    Ok(canonical)
}

fn file_sha256(path: &Path) -> Result<(String, u64), String> {
    let mut file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1 << 20];
    let mut bytes = 0_u64;
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        bytes = bytes
            .checked_add(read as u64)
            .ok_or_else(|| format!("{} byte count overflows", path.display()))?;
    }
    Ok((format!("{:x}", hasher.finalize()), bytes))
}

fn hex_sha256(bytes: &[u8]) -> String {
    uor_r4_discrete::corpus::sha256(bytes)
}

fn host_identity() -> AskArmHost {
    let cpu = std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                line.strip_prefix("model name\t: ")
                    .map(std::string::ToString::to_string)
            })
        })
        .unwrap_or_else(|| "unknown".to_owned());
    AskArmHost {
        cpu,
        logical_cpus: std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get),
        os: std::env::consts::OS.to_owned(),
    }
}

fn validate_store(value: &str) -> Result<(), String> {
    let parts = value.split('/').collect::<Vec<_>>();
    if parts.len() != 2
        || parts.iter().any(|part| part.is_empty())
        || value
            .bytes()
            .any(|byte| !byte.is_ascii_alphanumeric() && !matches!(byte, b'.' | b'_' | b'-' | b'/'))
    {
        return Err(format!("{value:?} is not an owner/name store"));
    }
    Ok(())
}

fn validate_store_path(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value.starts_with('/')
        || value.ends_with('/')
        || value.split('/').any(|part| part.is_empty() || part == "..")
    {
        return Err(format!("{value:?} is not a safe store path"));
    }
    Ok(())
}

fn validate_relative_path(value: &str, what: &str) -> Result<(), String> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("{what} {value:?} is not a plain relative path"));
    }
    Ok(())
}

fn validate_sha256(value: &str, what: &str) -> Result<(), String> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{what} SHA-256 is not 64 hexadecimal digits"));
    }
    Ok(())
}

fn validate_id(value: &str) -> Result<(), String> {
    if value.is_empty()
        || value
            .bytes()
            .any(|byte| !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && byte != b'-')
    {
        return Err(format!("arm id {value:?} is not lowercase kebab-case"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use safetensors::{serialize, tensor::TensorView, Dtype};

    const TEST_EXECUTABLE: &str =
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    struct TestDir(PathBuf);

    impl TestDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "uor-r4-ask-arm-regression-{name}-{}",
                std::process::id()
            ));
            if path.exists() {
                std::fs::remove_dir_all(&path).unwrap();
            }
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    fn f32_bytes(values: impl IntoIterator<Item = f32>) -> Vec<u8> {
        values.into_iter().flat_map(f32::to_le_bytes).collect()
    }

    fn u32_bytes(values: impl IntoIterator<Item = u32>) -> Vec<u8> {
        values.into_iter().flat_map(u32::to_le_bytes).collect()
    }

    fn file(tensors: Vec<(&str, Dtype, Vec<usize>, Vec<u8>)>) -> Vec<u8> {
        let tensors = tensors
            .into_iter()
            .map(|(name, dtype, shape, data)| (name.to_owned(), dtype, shape, data))
            .collect::<Vec<_>>();
        let views = tensors
            .iter()
            .map(|(name, dtype, shape, data)| {
                (
                    name.clone(),
                    TensorView::new(*dtype, shape.clone(), data).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        serialize(views, None).unwrap()
    }

    fn checkpoint(root: &Path) -> (String, String, AskArmSummary) {
        checkpoint_named(root, "checkpoint", 1.0)
    }

    fn checkpoint_named(
        root: &Path,
        name: &str,
        unused_token_value: f32,
    ) -> (String, String, AskArmSummary) {
        let (vocab, width, heads) = (64usize, 8usize, 2usize);
        let mut table = vec![1.0f32; vocab * width];
        table[..width].fill(unused_token_value);
        table[18 * width..19 * width].fill(-1.0);
        let model = file(vec![(
            "embedding.weight",
            Dtype::F32,
            vec![vocab, width],
            f32_bytes(table),
        )]);
        let rotation = (0..width / 4)
            .flat_map(|_| [1.0f32, 0.0, 0.0, 0.0])
            .collect::<Vec<_>>();
        let mut ask = vec![0.0f32; vocab * width];
        ask[33 * width..34 * width].fill(-2.0);
        let hamming = file(vec![
            ("heads", Dtype::U32, vec![1], u32_bytes([heads as u32])),
            (
                "d2.key_rotation",
                Dtype::F32,
                vec![width / 4, 4],
                f32_bytes(rotation),
            ),
            ("d2.ask", Dtype::F32, vec![vocab, width], f32_bytes(ask)),
        ]);
        let dir = root.join(name);
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("model.safetensors"), &model).unwrap();
        std::fs::write(dir.join("hamming_read.safetensors"), &hamming).unwrap();
        let offsets = TrainedAskOffsets::from_bytes(&model, &hamming).unwrap();
        let readings = offsets.audit_relations(16..32, 16).unwrap();
        (
            hex_sha256(&model),
            hex_sha256(&hamming),
            summarize_ask_offsets(&offsets, &readings),
        )
    }

    fn manifest(
        root: &Path,
        model_sha256: String,
        hamming_sha256: String,
        summary: AskArmSummary,
    ) -> Vec<u8> {
        let archive = b"sealed archive";
        std::fs::write(root.join("seed.tar.gz"), archive).unwrap();
        serde_json::to_vec(&serde_json::json!({
            "schema": ASK_ARM_REGRESSION_MANIFEST_SCHEMA,
            "source_store": "caseyallard/uor-r4-store",
            "remote_root": "results/claude/example",
            "archives": [{
                "path": "seed.tar.gz",
                "sha256": hex_sha256(archive)
            }],
            "settings": {
                "relation_start": 16,
                "relation_end": 32,
                "query_shift": 16
            },
            "arms": [{
                "id": "arm-seed-1",
                "label": "arm",
                "seed": 1,
                "archive": "seed.tar.gz",
                "checkpoint": "checkpoint",
                "model_sha256": model_sha256,
                "hamming_sha256": hamming_sha256,
                "expected": summary
            }]
        }))
        .unwrap()
    }

    #[test]
    fn real_shape_fixture_passes_and_repeats_deterministically() {
        let dir = TestDir::new("pass");
        let (model, hamming, summary) = checkpoint(&dir.0);
        let bytes = manifest(&dir.0, model, hamming, summary);
        let first = run_ask_arm_regression(&dir.0, &bytes, 2, TEST_EXECUTABLE).unwrap();
        let second = run_ask_arm_regression(&dir.0, &bytes, 2, TEST_EXECUTABLE).unwrap();
        assert!(first.passed);
        assert_eq!(first, second);
        assert_eq!(first.arms[0].observed, Some(first.arms[0].expected.clone()));
        assert!(run_ask_arm_regression(&dir.0, &bytes, 2, "not-a-sha").is_err());
    }

    #[test]
    fn wrong_expected_field_is_named() {
        let dir = TestDir::new("mismatch");
        let (model, hamming, mut summary) = checkpoint(&dir.0);
        summary.trained_target_nearest = summary.trained_target_nearest.saturating_sub(1);
        let bytes = manifest(&dir.0, model, hamming, summary);
        let report = run_ask_arm_regression(&dir.0, &bytes, 2, TEST_EXECUTABLE).unwrap();
        assert!(!report.passed);
        assert_eq!(report.failures(), 1);
        assert_eq!(report.source_failures(), 0);
        assert_eq!(report.regression_failures(), 1);
        assert!(report.arms[0]
            .mismatches
            .iter()
            .any(|line| line.starts_with("trained_target_nearest expected")));
    }

    #[test]
    fn wrong_checkpoint_hash_fails_the_case() {
        let dir = TestDir::new("hash");
        let (_model, hamming, summary) = checkpoint(&dir.0);
        let bytes = manifest(&dir.0, "0".repeat(64), hamming, summary);
        let report = run_ask_arm_regression(&dir.0, &bytes, 1, TEST_EXECUTABLE).unwrap();
        assert!(!report.passed);
        assert_eq!(report.source_failures(), 1);
        assert_eq!(report.regression_failures(), 0);
        assert!(report.arms[0]
            .error
            .as_deref()
            .is_some_and(|error| error.contains("model.safetensors SHA-256 differs")));
    }

    #[test]
    fn arms_keep_manifest_order_and_a_missing_file_is_a_case_error() {
        let dir = TestDir::new("multiple");
        let (model, hamming, summary) = checkpoint(&dir.0);
        let bytes = manifest(&dir.0, model, hamming, summary);
        let (second_model, second_hamming, second_summary) =
            checkpoint_named(&dir.0, "checkpoint-2", 2.0);
        let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let mut second = value["arms"][0].clone();
        second["id"] = serde_json::json!("arm-seed-2");
        second["seed"] = serde_json::json!(2);
        second["checkpoint"] = serde_json::json!("checkpoint-2");
        second["model_sha256"] = serde_json::json!(second_model);
        second["hamming_sha256"] = serde_json::json!(second_hamming);
        second["expected"] = serde_json::to_value(second_summary).unwrap();
        value["arms"] = serde_json::json!([value["arms"][0].clone(), second]);
        let report = run_ask_arm_regression(
            &dir.0,
            &serde_json::to_vec(&value).unwrap(),
            8,
            TEST_EXECUTABLE,
        )
        .unwrap();
        assert!(report.passed);
        assert_eq!(
            report
                .arms
                .iter()
                .map(|arm| arm.id.as_str())
                .collect::<Vec<_>>(),
            ["arm-seed-1", "arm-seed-2"]
        );

        std::fs::remove_file(dir.0.join("checkpoint/hamming_read.safetensors")).unwrap();
        std::fs::remove_file(dir.0.join("checkpoint-2/hamming_read.safetensors")).unwrap();
        let report = run_ask_arm_regression(
            &dir.0,
            &serde_json::to_vec(&value).unwrap(),
            2,
            TEST_EXECUTABLE,
        )
        .unwrap();
        assert!(!report.passed);
        assert_eq!(report.source_failures(), 2);
        assert_eq!(report.regression_failures(), 0);
        assert!(report.arms.iter().all(|arm| {
            arm.error
                .as_deref()
                .is_some_and(|error| error.contains("hamming_read.safetensors"))
        }));
    }

    #[test]
    fn wrong_archive_hash_is_refused_before_arm_work() {
        let dir = TestDir::new("archive");
        let (model, hamming, summary) = checkpoint(&dir.0);
        let bytes = manifest(&dir.0, model, hamming, summary);
        let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        value["archives"][0]["sha256"] = serde_json::json!("0".repeat(64));
        let error = run_ask_arm_regression(
            &dir.0,
            &serde_json::to_vec(&value).unwrap(),
            1,
            TEST_EXECUTABLE,
        )
        .unwrap_err();
        assert!(error.contains("archive SHA-256 differs"));
    }

    #[test]
    fn unsafe_and_duplicate_paths_are_refused() {
        let dir = TestDir::new("paths");
        let (model, hamming, summary) = checkpoint(&dir.0);
        let bytes = manifest(&dir.0, model, hamming, summary);
        for checkpoint in [
            "../checkpoint",
            "/checkpoint",
            "./checkpoint",
            "a/../checkpoint",
        ] {
            let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            value["arms"][0]["checkpoint"] = serde_json::json!(checkpoint);
            assert!(parse_manifest(&serde_json::to_vec(&value).unwrap())
                .unwrap_err()
                .contains("plain relative path"));
        }
        let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        value["arms"][0]["id"] = serde_json::json!("Not-Safe");
        assert!(parse_manifest(&serde_json::to_vec(&value).unwrap())
            .unwrap_err()
            .contains("lowercase kebab-case"));

        let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        value["arms"] = serde_json::json!([value["arms"][0].clone(), value["arms"][0].clone()]);
        assert!(parse_manifest(&serde_json::to_vec(&value).unwrap())
            .unwrap_err()
            .contains("is duplicated"));

        let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let mut second = value["arms"][0].clone();
        second["id"] = serde_json::json!("arm-seed-2");
        value["arms"] = serde_json::json!([value["arms"][0].clone(), second]);
        assert!(parse_manifest(&serde_json::to_vec(&value).unwrap())
            .unwrap_err()
            .contains("checkpoint path"));

        let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let mut second = value["arms"][0].clone();
        second["id"] = serde_json::json!("arm-seed-2");
        second["checkpoint"] = serde_json::json!("checkpoint-2");
        value["arms"] = serde_json::json!([value["arms"][0].clone(), second]);
        assert!(parse_manifest(&serde_json::to_vec(&value).unwrap())
            .unwrap_err()
            .contains("repeats checkpoint hashes"));

        let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        value["archives"] =
            serde_json::json!([value["archives"][0].clone(), value["archives"][0].clone()]);
        assert!(parse_manifest(&serde_json::to_vec(&value).unwrap())
            .unwrap_err()
            .contains("is duplicated"));
    }
}
