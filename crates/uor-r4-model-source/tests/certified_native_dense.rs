//! #704 Choice-A dense arithmetic differential and cheap-gate harness.
//!
//! The checked controls preserve the binding whole-model PASS and remain the
//! production differential oracle. Every Conv1D candidate is compared bit-for-bit with one pinned
//! correctly-rounded dot plus one binary32 bias add; tied lm-head dots have no
//! bias. Workspace construction and source binding remain outside timing.
#![cfg(not(target_arch = "wasm32"))]

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use serde::Deserialize;
use uor_r4_model_source::gpt2::{
    Gpt2, Gpt2DenseCanaryCensus, Gpt2DenseCanaryMode, Gpt2DenseControlCensus, Gpt2DenseControlSite,
    Gpt2DenseLayerCensus, Gpt2State,
};
use uor_r4_model_source::{TraceCaptureRequest, TraceCaptureSinks};

const GPT2_REPOSITORY: &str = "openai-community/gpt2";
const GPT2_REVISION: &str = "607a30d783dfa663caf39e06633721c8d4cfcd7e";
const GPT2_MODEL_KAPPA: &str =
    "blake3:3bca1b7f6c327daecafc16e52d1319375299354e35413fb4e18d24e59b77ce06";
const GPT2_CONFIG_KAPPA: &str =
    "blake3:23e4471d412e06128072b559c031207de920b8a56d7108879d4b487c079a310c";
const GPT2_MODEL_BYTES: usize = 548_105_171;
const GPT2_CONFIG_BYTES: usize = 665;
const UOR_MATMUL_REV: &str = "b13c98449948174f590e337c4dc25dfc394a07d0";
const BASE_HEAD: &str = "6fbf718b4115859df6544545a5c43d7638a6ad0a";
const PINNED_RUSTC_RELEASE: &str = "1.97.1";
const PINNED_RUSTC_HOST: &str = "aarch64-apple-darwin";
const DENSE_ARITHMETIC_ID: &str = "gpt2-c-attn-exact-real-dot-certified-native-f64-sum-maxabsx-sumabsw-outward-cell-twosum-refine-or-pinned-uor-matmul-exact-fallback/1";
const LEGACY_C_ATTN_GPT2_SHA256: &str =
    "9c3435636638c63d2baac3545480dfb694db923c14b0c5ad8fde7c8c7dcff167";
const LEGACY_C_ATTN_HARNESS_SHA256: &str =
    "ea823d27a74ac4cd4168f0ab6d7d1da4c92678ecb178489c0be90d67f3082685";
const BINDING_WHOLE_GPT2_SHA256: &str =
    "1a945487fb9ec350fd8f670b8c04dacaf6b66e2339ee96b6b3883082e4de4bf8";
const BINDING_WHOLE_HARNESS_SHA256: &str =
    "144fd32d35292a6fd3e949bc7040b7e14c58563bdc31924b66d38bc1db547d89";
const BOOTSTRAP_ALGORITHM_ID: &str =
    "empirical-bootstrap-median-exact-order-statistic-n9-one-sided-upper-95/1";
const REAL_PAIRS: usize = 9;
const REAL_WARMUPS_PER_ARM: usize = 20;
const REAL_OUTPUT_LANES: usize = 2304;
const MAXIMUM_UPPER_RATIO: f64 = 4.0;
const WHOLE_DENSE_ARITHMETIC_ID: &str = "gpt2-all-dense-exact-dot-plus-f32-bias-certified-native-f64-sum-maxabsx-sumabsw-outward-cell-twosum-refine-or-pinned-uor-matmul-exact-fallback-tied-head-transpose/1";
const WHOLE_BOOTSTRAP_ALGORITHM_ID: &str = "whole-gpt2-11-step-3-story-paired-empirical-bootstrap-median-exact-order-statistic-n9-one-sided-upper-95/1";
const WHOLE_REAL_PAIRS: usize = 9;
const WHOLE_WARMUP_SUITES_PER_ARM: usize = 2;
const WHOLE_STEPS: usize = 11;
const WHOLE_OWNER_CALLS: usize = 539;
const WHOLE_LANES: usize = 1_465_211;
const WHOLE_MAXIMUM_UPPER_RATIO: f64 = 3.0;
const WHOLE_EXPECTED_GPT2_SHA256_ENV: &str = "UOR_DENSE_EXPECTED_GPT2_SHA256";
const WHOLE_EXPECTED_HARNESS_SHA256_ENV: &str = "UOR_DENSE_EXPECTED_HARNESS_SHA256";
const OUTPUT_POISON_BITS: u32 = 0x7fc0_704d;

#[derive(Clone, Copy)]
struct DenseGateContract {
    arithmetic_id: &'static str,
    bootstrap_id: &'static str,
    pairs: usize,
    warmups_per_arm: usize,
    threshold: f64,
    threshold_rule: &'static str,
}

struct CountingAllocator;

thread_local! {
    static COUNT_ALLOCATIONS: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

fn record_allocation(pointer: *mut u8) {
    if !pointer.is_null() {
        let _ = COUNT_ALLOCATIONS.try_with(|gate| {
            if gate.get() {
                let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
            }
        });
    }
}

// SAFETY: every request is forwarded unchanged to `System`; the thread-local
// cells are observational and never participate in allocation.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the backing allocator receives the original request.
        let pointer = unsafe { System.alloc(layout) };
        record_allocation(pointer);
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: the allocation came from `System` with this layout.
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the backing allocator receives the original request.
        let pointer = unsafe { System.alloc_zeroed(layout) };
        record_allocation(pointer);
        pointer
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        // SAFETY: the backing allocator receives the original request.
        let resized = unsafe { System.realloc(pointer, layout, size) };
        record_allocation(resized);
        resized
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: CountingAllocator = CountingAllocator;

struct AllocationMeasurement;

impl Drop for AllocationMeasurement {
    fn drop(&mut self) {
        COUNT_ALLOCATIONS.with(|gate| gate.set(false));
    }
}

fn counted_allocations<T>(operation: impl FnOnce() -> T) -> (T, usize) {
    ALLOCATIONS.with(|count| count.set(0));
    COUNT_ALLOCATIONS.with(|gate| {
        assert!(!gate.replace(true), "nested allocation measurement");
    });
    let measurement = AllocationMeasurement;
    let result = operation();
    drop(measurement);
    (result, ALLOCATIONS.with(Cell::get))
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/gpt2-tiny")
}

fn real_source() -> PathBuf {
    std::env::var_os("UOR_GPT2_SOURCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(".uor-models/sources/gpt2-124m")
        })
}

fn assert_bits(got: &[f32], expected: &[f32], context: &str) {
    assert_eq!(got.len(), expected.len(), "{context}: output length");
    for (lane, (&got, &expected)) in got.iter().zip(expected).enumerate() {
        assert_eq!(
            got.to_bits(),
            expected.to_bits(),
            "{context}: lane {lane}: got={got:?}, expected={expected:?}"
        );
    }
}

fn poison_output(output: &mut [f32]) {
    output.fill(f32::from_bits(OUTPUT_POISON_BITS));
}

fn assert_output_overwritten(output: &[f32], context: &str) {
    for (lane, value) in output.iter().enumerate() {
        assert_ne!(
            value.to_bits(),
            OUTPUT_POISON_BITS,
            "{context}: lane {lane} retained the output poison"
        );
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct DenseCensusTotal {
    calls: usize,
    lanes: usize,
    fast: usize,
    refined: usize,
    fallback_nonfinite: usize,
    fallback_zero: usize,
    fallback_overflow: usize,
    fallback_cell: usize,
}

impl DenseCensusTotal {
    fn observe_candidate(&mut self, census: Gpt2DenseCanaryCensus, expected_lanes: usize) {
        assert_eq!(census.lanes(), expected_lanes, "candidate lane count");
        assert_eq!(census.conventional(), 0, "candidate conventional count");
        assert_eq!(census.exact_control(), 0, "candidate exact-control count");
        assert_candidate_partition(census);
        self.calls += 1;
        self.lanes += census.lanes();
        self.fast += census.fast_certified();
        self.refined += census.refined_certified();
        self.fallback_nonfinite += census.fallback_nonfinite();
        self.fallback_zero += census.fallback_zero();
        self.fallback_overflow += census.fallback_overflow();
        self.fallback_cell += census.fallback_cell();
    }

    fn fallbacks(self) -> usize {
        self.fallback_nonfinite + self.fallback_zero + self.fallback_overflow + self.fallback_cell
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn sha256_file(path: &Path) -> String {
    let output = Command::new("shasum")
        .args(["-a", "256"])
        .arg(path)
        .output()
        .unwrap_or_else(|error| panic!("hash {}: {error}", path.display()));
    assert!(
        output.status.success(),
        "shasum failed for {}",
        path.display()
    );
    String::from_utf8(output.stdout)
        .expect("shasum output is UTF-8")
        .split_whitespace()
        .next()
        .expect("shasum reports a digest")
        .to_owned()
}

#[derive(Deserialize)]
struct SourceManifest {
    schema: String,
    repository: String,
    revision: String,
    files: Vec<SourceManifestFile>,
}

#[derive(Deserialize)]
struct SourceManifestFile {
    path: String,
    bytes: usize,
    kappa: String,
}

fn assert_manifest_file(
    manifest: &SourceManifest,
    path: &str,
    expected_bytes: usize,
    expected_kappa: &str,
) {
    let file = manifest
        .files
        .iter()
        .find(|file| file.path == path)
        .unwrap_or_else(|| panic!("source manifest is missing {path}"));
    assert_eq!(file.bytes, expected_bytes, "{path} manifest bytes");
    assert_eq!(file.kappa, expected_kappa, "{path} manifest kappa");
}

fn bind_dense_identity(path: &Path, model: &Gpt2, contract: DenseGateContract) {
    let root = workspace_root();
    let canonical = path.canonicalize().expect("canonicalize GPT-2 source");
    assert_eq!(model.attention_control_source_kappa(), GPT2_MODEL_KAPPA);
    assert_eq!(model.attention_control_source_bytes(), GPT2_MODEL_BYTES);

    let config = std::fs::read(path.join("config.json")).expect("read GPT-2 config");
    assert_eq!(config.len(), GPT2_CONFIG_BYTES);
    assert_eq!(
        format!("blake3:{}", blake3::hash(&config).to_hex()),
        GPT2_CONFIG_KAPPA
    );

    let manifest: SourceManifest = serde_json::from_slice(
        &std::fs::read(path.join("source_manifest.json")).expect("read source manifest"),
    )
    .expect("parse source manifest");
    assert_eq!(manifest.schema, "uor-r4-source-manifest/1");
    assert_eq!(manifest.repository, GPT2_REPOSITORY);
    assert_eq!(manifest.revision, GPT2_REVISION);
    assert_manifest_file(
        &manifest,
        "model.safetensors",
        GPT2_MODEL_BYTES,
        GPT2_MODEL_KAPPA,
    );
    assert_manifest_file(
        &manifest,
        "config.json",
        GPT2_CONFIG_BYTES,
        GPT2_CONFIG_KAPPA,
    );

    let model_source_manifest =
        std::fs::read_to_string(root.join("crates/uor-r4-model-source/Cargo.toml"))
            .expect("read model-source Cargo.toml");
    assert!(
        model_source_manifest.contains(&format!("rev = \"{UOR_MATMUL_REV}\"")),
        "model-source Cargo.toml does not pin the declared uor-matmul revision"
    );
    let lockfile = std::fs::read_to_string(root.join("Cargo.lock")).expect("read Cargo.lock");
    assert!(
        lockfile.contains(&format!("?rev={UOR_MATMUL_REV}#{UOR_MATMUL_REV}")),
        "Cargo.lock does not resolve the declared uor-matmul revision"
    );

    let head = Command::new("git")
        .current_dir(&root)
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("read dense canary HEAD");
    assert!(head.status.success(), "git rev-parse HEAD failed");
    let head = String::from_utf8(head.stdout).expect("git HEAD is UTF-8");
    assert_eq!(
        head.trim(),
        BASE_HEAD,
        "dense canary must run from its exact declared base"
    );
    let status = Command::new("git")
        .current_dir(&root)
        .args(["status", "--porcelain=v1", "--untracked-files=all"])
        .output()
        .expect("read dense canary worktree status");
    assert!(status.status.success(), "git status --porcelain failed");
    let status = String::from_utf8(status.stdout).expect("git status is UTF-8");
    let mut declared_paths = Vec::new();
    for line in status.lines() {
        assert!(line.len() >= 4, "malformed git porcelain line: {line:?}");
        let (code, path) = line.split_at(3);
        match path {
            "crates/uor-r4-model-source/src/gpt2.rs" => {
                assert_eq!(code.trim(), "M", "gpt2.rs must be the one modified path")
            }
            "crates/uor-r4-model-source/tests/certified_native_dense.rs" => {
                assert_eq!(code.trim(), "??", "dense harness must be untracked")
            }
            _ => panic!("undeclared dirty path invalidates dense timing identity: {line}"),
        }
        declared_paths.push(path);
    }
    declared_paths.sort_unstable();
    assert_eq!(
        declared_paths,
        [
            "crates/uor-r4-model-source/src/gpt2.rs",
            "crates/uor-r4-model-source/tests/certified_native_dense.rs",
        ],
        "dense timing requires exactly the two declared candidate paths"
    );

    let toolchain = std::fs::read_to_string(root.join("rust-toolchain.toml"))
        .expect("read pinned rust toolchain");
    assert!(toolchain
        .lines()
        .any(|line| line.trim() == format!("channel = \"{PINNED_RUSTC_RELEASE}\"")));
    let forbidden_build_overrides: Vec<(String, String)> = std::env::vars()
        .filter(|(key, value)| {
            !value.is_empty()
                && (key.contains("RUSTFLAGS")
                    || key.starts_with("CARGO_PROFILE_RELEASE_")
                    || matches!(
                        key.as_str(),
                        "RUSTC"
                            | "RUSTC_BOOTSTRAP"
                            | "RUSTC_WRAPPER"
                            | "RUSTC_WORKSPACE_WRAPPER"
                            | "CARGO_BUILD_TARGET"
                            | "CARGO_INCREMENTAL"
                    )
                    || (key.starts_with("CARGO_TARGET_")
                        && (key.ends_with("_LINKER") || key.ends_with("_RUNNER"))))
        })
        .collect();
    assert!(
        forbidden_build_overrides.is_empty(),
        "release canary forbids external codegen/release overrides: {forbidden_build_overrides:?}"
    );

    let rustc = Command::new("rustc")
        .current_dir(&root)
        .args(["--version", "--verbose"])
        .output()
        .expect("query rustc identity");
    assert!(rustc.status.success());
    let rustc_identity = String::from_utf8(rustc.stdout).expect("rustc identity is UTF-8");
    let rustc_release = rustc_identity
        .lines()
        .find_map(|line| line.strip_prefix("release: "))
        .expect("rustc -vV reports release");
    let rustc_host = rustc_identity
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .expect("rustc -vV reports host");
    assert_eq!(rustc_release, PINNED_RUSTC_RELEASE);
    assert_eq!(rustc_host, PINNED_RUSTC_HOST);
    assert_eq!(std::env::consts::ARCH, "aarch64");
    assert_eq!(std::env::consts::OS, "macos");

    eprintln!(
        "CERTIFIED_DENSE_IDENTITY repository={GPT2_REPOSITORY} revision={GPT2_REVISION} model_kappa={GPT2_MODEL_KAPPA} model_bytes={GPT2_MODEL_BYTES} config_kappa={GPT2_CONFIG_KAPPA} config_bytes={GPT2_CONFIG_BYTES} source_path={} arithmetic_id={} bootstrap_algorithm_id={} base_head={BASE_HEAD} uor_matmul_rev={UOR_MATMUL_REV} arch={} os={} rustc_release={PINNED_RUSTC_RELEASE} rustc_host={PINNED_RUSTC_HOST} build_overrides=none pairs={} warmups_per_arm={} threshold={:.1} threshold_rule={}",
        canonical.display(),
        contract.arithmetic_id,
        contract.bootstrap_id,
        std::env::consts::ARCH,
        std::env::consts::OS,
        contract.pairs,
        contract.warmups_per_arm,
        contract.threshold,
        contract.threshold_rule,
    );
    for line in rustc_identity.lines() {
        eprintln!("CERTIFIED_DENSE_RUSTC {line}");
    }
}

fn splitmix(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut value = *seed;
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn moderate_finite(seed: &mut u64) -> f32 {
    let bits = splitmix(seed);
    let sign = ((bits >> 63) as u32) << 31;
    let exponent = 112 + ((bits >> 32) as u32 % 31);
    let fraction = bits as u32 & 0x007f_ffff;
    f32::from_bits(sign | (exponent << 23) | fraction)
}

fn assert_candidate_partition(part: Gpt2DenseCanaryCensus) {
    // The public census intentionally exposes no mutable raw counters. These
    // totals are only the observations needed by this differential harness.
    assert_eq!(
        part.fast_certified()
            + part.refined_certified()
            + part.fallbacks().expect("fallback count fits usize"),
        part.lanes(),
        "candidate verdicts partition the output lanes"
    );
}

fn assert_state_bits(got: &Gpt2State, expected: &Gpt2State, context: &str) {
    assert!(
        got.dense_control_bit_identical(expected),
        "{context}: recurrent state differs bit-for-bit"
    );
}

fn assert_mode_census(
    census: Gpt2DenseCanaryCensus,
    expected_lanes: usize,
    mode: Gpt2DenseCanaryMode,
) {
    match mode {
        Gpt2DenseCanaryMode::Conventional => assert_conventional_census(census, expected_lanes),
        Gpt2DenseCanaryMode::Exact => assert_exact_control_census(census, expected_lanes),
        Gpt2DenseCanaryMode::CertifiedNative => {
            assert_eq!(census.lanes(), expected_lanes);
            assert_eq!(census.conventional(), 0);
            assert_eq!(census.exact_control(), 0);
            assert_candidate_partition(census);
        }
    }
}

fn assert_whole_census(
    census: Gpt2DenseControlCensus<'_>,
    model: &Gpt2,
    batch: usize,
    row_reuse_rows: usize,
    mode: Gpt2DenseCanaryMode,
) -> DenseCensusTotal {
    assert_whole_census_parts(
        census.layers(),
        census.lm_head(),
        model,
        batch,
        row_reuse_rows,
        mode,
    )
}

fn assert_whole_census_parts(
    layers: &[Gpt2DenseLayerCensus],
    lm_head: Gpt2DenseCanaryCensus,
    model: &Gpt2,
    batch: usize,
    row_reuse_rows: usize,
    mode: Gpt2DenseCanaryMode,
) -> DenseCensusTotal {
    assert_eq!(layers.len(), model.cfg.n_layer);
    let expected = [
        3 * model.cfg.n_embd,
        model.cfg.n_embd,
        model.cfg.n_inner,
        model.cfg.n_embd,
    ];
    let mut candidate = DenseCensusTotal::default();
    for layer in layers {
        for (part, lanes) in [
            (layer.c_attn(), expected[0]),
            (layer.attention_c_proj(), expected[1]),
            (layer.mlp_c_fc(), expected[2]),
            (layer.mlp_c_proj(), expected[3]),
        ] {
            let lanes = batch * lanes;
            assert_eq!(part.batch_rows(), row_reuse_rows);
            assert_mode_census(part, lanes, mode);
            if mode == Gpt2DenseCanaryMode::CertifiedNative {
                candidate.observe_candidate(part, lanes);
            }
        }
    }
    let head_lanes = batch * model.cfg.vocab;
    assert_eq!(lm_head.batch_rows(), row_reuse_rows);
    assert_mode_census(lm_head, head_lanes, mode);
    if mode == Gpt2DenseCanaryMode::CertifiedNative {
        candidate.observe_candidate(lm_head, head_lanes);
    }
    candidate
}

#[test]
fn checked_dense_canary_matches_exact_on_adversarial_and_random_inputs() {
    let model = Gpt2::load(fixture_dir(), None).expect("load tiny GPT-2 fixture");
    let d = model.cfg.n_embd;
    let out_dim = 3 * d;
    let mut workspace = model
        .dense_canary_workspace()
        .expect("valid model admits dense scratch");
    let mut exact = vec![0.0f32; out_dim];
    let mut candidate = vec![0.0f32; out_dim];
    let mut input = vec![0.0f32; d];
    let mut observed_fast = 0usize;
    let mut observed_refined = 0usize;

    model
        .layer0_c_attn_canary_input(3, 2, &mut input)
        .expect("real fixture activation extent is valid");
    for case in 0..96 {
        if case != 0 {
            let mut seed = 0x704d_e15e_0000_0000u64 ^ case as u64;
            for value in &mut input {
                *value = moderate_finite(&mut seed);
            }
        }
        let exact_census = model
            .layer0_c_attn_canary(
                &input,
                &mut exact,
                &mut workspace,
                Gpt2DenseCanaryMode::Exact,
            )
            .expect("checked exact control");
        assert_eq!(exact_census.exact_control(), out_dim);
        let census = model
            .layer0_c_attn_canary(
                &input,
                &mut candidate,
                &mut workspace,
                Gpt2DenseCanaryMode::CertifiedNative,
            )
            .expect("checked certified candidate");
        assert_bits(&candidate, &exact, &format!("finite case {case}"));
        assert_candidate_partition(census);
        observed_fast += census.fast_certified();
        observed_refined += census.refined_certified();
    }
    assert!(
        observed_fast > 0,
        "random corpus never certified a fast lane"
    );
    eprintln!("DENSE_RANDOM_CENSUS fast={observed_fast} refined={observed_refined}");

    // Exact cancellation is deliberately delegated, including signed-zero
    // selection, and the single f32 bias epilogue must still match exactly.
    input.fill(0.0);
    model
        .layer0_c_attn_canary(
            &input,
            &mut exact,
            &mut workspace,
            Gpt2DenseCanaryMode::Exact,
        )
        .expect("zero exact control");
    let zero = model
        .layer0_c_attn_canary(
            &input,
            &mut candidate,
            &mut workspace,
            Gpt2DenseCanaryMode::CertifiedNative,
        )
        .expect("zero certified candidate");
    assert_bits(&candidate, &exact, "zero fallback");
    assert_eq!(zero.fallback_zero(), out_dim);
}

#[test]
fn checked_dense_canary_is_failure_atomic_and_allocation_free() {
    let model = Gpt2::load(fixture_dir(), None).expect("load tiny GPT-2 fixture");
    let other = Gpt2::load(fixture_dir(), None).expect("load independent fixture model");
    let d = model.cfg.n_embd;
    let out_dim = 3 * d;
    let input = vec![0.25f32; d];
    let mut output = vec![f32::from_bits(0x7fc0_704d); out_dim];
    let mut workspace = model
        .dense_canary_workspace()
        .expect("valid model admits dense scratch");

    let output_before = output.clone();
    let workspace_before = workspace.clone();
    assert_eq!(
        other.layer0_c_attn_canary(
            &input,
            &mut output,
            &mut workspace,
            Gpt2DenseCanaryMode::CertifiedNative,
        ),
        None,
        "workspace from another model must be rejected"
    );
    assert_bits(
        &output,
        &output_before,
        "foreign-workspace failure atomicity",
    );
    assert_eq!(workspace, workspace_before);

    let mut untouched_output = output_before.clone();
    let workspace_before = workspace.clone();
    assert_eq!(
        model.layer0_c_attn_canary(
            &input[..d - 1],
            &mut untouched_output,
            &mut workspace,
            Gpt2DenseCanaryMode::CertifiedNative,
        ),
        None
    );
    assert_bits(
        &untouched_output,
        &output_before,
        "short-input failure atomicity",
    );
    assert_eq!(workspace, workspace_before);

    let mut short_output = vec![0.0f32; out_dim - 1];
    let short_before = short_output.clone();
    let workspace_before = workspace.clone();
    assert_eq!(
        model.layer0_c_attn_canary(
            &input,
            &mut short_output,
            &mut workspace,
            Gpt2DenseCanaryMode::CertifiedNative,
        ),
        None
    );
    assert_eq!(short_output, short_before);
    assert_eq!(workspace, workspace_before);

    let (census, allocations) = counted_allocations(|| {
        model
            .layer0_c_attn_canary(
                &input,
                &mut output,
                &mut workspace,
                Gpt2DenseCanaryMode::CertifiedNative,
            )
            .expect("valid checked candidate")
    });
    assert_eq!(allocations, 0, "dense candidate hot call allocated");
    assert_eq!(
        census.fast_certified()
            + census.refined_certified()
            + census.fallbacks().expect("fallback count fits usize"),
        out_dim
    );

    let zero_input = vec![0.0f32; d];
    let (fallback_census, fallback_allocations) = counted_allocations(|| {
        model
            .layer0_c_attn_canary(
                &zero_input,
                &mut output,
                &mut workspace,
                Gpt2DenseCanaryMode::CertifiedNative,
            )
            .expect("valid checked exact-fallback candidate")
    });
    assert_eq!(
        fallback_allocations, 0,
        "dense exact-fallback hot call allocated"
    );
    assert_eq!(fallback_census.fallback_zero(), out_dim);
}

#[test]
fn public_nonfinite_dense_inputs_use_bit_exact_allocation_free_fallback() {
    let model = Gpt2::load(fixture_dir(), None).expect("load tiny GPT-2 fixture");
    let d = model.cfg.n_embd;
    let out_dim = 3 * d;
    let mut workspace = model
        .dense_canary_workspace()
        .expect("valid model admits dense scratch");
    let mut exact = vec![0.0f32; out_dim];
    let mut candidate = vec![0.0f32; out_dim];

    for (case, nonfinite) in [
        ("quiet NaN", f32::from_bits(0x7fc0_0704)),
        ("positive infinity", f32::INFINITY),
        ("negative infinity", f32::NEG_INFINITY),
    ] {
        let mut input = vec![0.25f32; d];
        input[d / 2] = nonfinite;
        let input_before: Vec<u32> = input.iter().map(|value| value.to_bits()).collect();
        poison_output(&mut exact);
        model
            .layer0_c_attn_canary(
                &input,
                &mut exact,
                &mut workspace,
                Gpt2DenseCanaryMode::Exact,
            )
            .expect("checked pinned exact nonfinite control");
        assert_output_overwritten(&exact, &format!("{case} exact control"));

        poison_output(&mut candidate);
        let (census, allocations) = counted_allocations(|| {
            model
                .layer0_c_attn_canary(
                    &input,
                    &mut candidate,
                    &mut workspace,
                    Gpt2DenseCanaryMode::CertifiedNative,
                )
                .expect("checked nonfinite candidate")
        });
        assert_eq!(allocations, 0, "{case} exact fallback allocated");
        assert_output_overwritten(&candidate, &format!("{case} candidate"));
        assert_bits(&candidate, &exact, &format!("{case} pinned fallback"));
        assert_eq!(census.lanes(), out_dim);
        assert_eq!(census.fast_certified(), 0);
        assert_eq!(census.refined_certified(), 0);
        assert_eq!(census.fallback_nonfinite(), out_dim);
        assert_eq!(census.fallback_zero(), 0);
        assert_eq!(census.fallback_overflow(), 0);
        assert_eq!(census.fallback_cell(), 0);
        assert_eq!(
            input
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            input_before,
            "{case} mutated caller input"
        );
    }
}

#[test]
fn every_dense_owner_shape_matches_exact_for_random_zero_and_nonfinite_inputs() {
    let model = Gpt2::load(fixture_dir(), None).expect("load tiny GPT-2 fixture");
    let mut workspace = model
        .dense_control_workspace()
        .expect("prepare whole-model dense workspace");
    let d = model.cfg.n_embd;
    let mut observed_fast = 0usize;
    let mut seed = 0x704d_5eed_0000_0001u64;

    for layer in 0..model.cfg.n_layer {
        for (site, in_dim, out_dim) in [
            (Gpt2DenseControlSite::CAttn, d, 3 * d),
            (Gpt2DenseControlSite::AttentionCProj, d, d),
            (Gpt2DenseControlSite::MlpCFc, d, model.cfg.n_inner),
            (Gpt2DenseControlSite::MlpCProj, model.cfg.n_inner, d),
        ] {
            let mut input = vec![0.0f32; in_dim];
            for value in &mut input {
                *value = moderate_finite(&mut seed);
            }
            let input_before: Vec<u32> = input.iter().map(|value| value.to_bits()).collect();
            let mut exact = vec![0.0f32; out_dim];
            let mut candidate = vec![0.0f32; out_dim];
            model
                .dense_control_matrix_canary(
                    &mut workspace,
                    Some(layer),
                    site,
                    &input,
                    &mut exact,
                    Gpt2DenseCanaryMode::Exact,
                )
                .expect("valid exact matrix control");
            poison_output(&mut candidate);
            let (census, allocations) = counted_allocations(|| {
                model
                    .dense_control_matrix_canary(
                        &mut workspace,
                        Some(layer),
                        site,
                        &input,
                        &mut candidate,
                        Gpt2DenseCanaryMode::CertifiedNative,
                    )
                    .expect("valid candidate matrix control")
            });
            assert_eq!(allocations, 0, "{site:?} candidate allocated");
            assert_output_overwritten(&candidate, &format!("{site:?} random"));
            assert_bits(&candidate, &exact, &format!("layer {layer} {site:?}"));
            assert_candidate_partition(census);
            observed_fast += census.fast_certified();
            assert_eq!(
                input
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>(),
                input_before,
                "{site:?} mutated its input"
            );

            input.fill(0.0);
            model
                .dense_control_matrix_canary(
                    &mut workspace,
                    Some(layer),
                    site,
                    &input,
                    &mut exact,
                    Gpt2DenseCanaryMode::Exact,
                )
                .expect("zero exact matrix control");
            let zero = model
                .dense_control_matrix_canary(
                    &mut workspace,
                    Some(layer),
                    site,
                    &input,
                    &mut candidate,
                    Gpt2DenseCanaryMode::CertifiedNative,
                )
                .expect("zero candidate matrix control");
            assert_bits(&candidate, &exact, &format!("layer {layer} {site:?} zero"));
            assert_eq!(zero.fallback_zero(), out_dim);

            input.fill(0.25);
            input[in_dim / 2] = f32::from_bits(0x7fc0_704d);
            let input_before: Vec<u32> = input.iter().map(|value| value.to_bits()).collect();
            poison_output(&mut exact);
            model
                .dense_control_matrix_canary(
                    &mut workspace,
                    Some(layer),
                    site,
                    &input,
                    &mut exact,
                    Gpt2DenseCanaryMode::Exact,
                )
                .expect("nonfinite exact matrix control");
            assert_output_overwritten(&exact, "nonfinite exact matrix output");
            poison_output(&mut candidate);
            let (nonfinite, allocations) = counted_allocations(|| {
                model
                    .dense_control_matrix_canary(
                        &mut workspace,
                        Some(layer),
                        site,
                        &input,
                        &mut candidate,
                        Gpt2DenseCanaryMode::CertifiedNative,
                    )
                    .expect("nonfinite candidate matrix control")
            });
            assert_eq!(allocations, 0, "{site:?} nonfinite fallback allocated");
            assert_output_overwritten(&candidate, "nonfinite candidate matrix output");
            assert_bits(
                &candidate,
                &exact,
                &format!("layer {layer} {site:?} nonfinite"),
            );
            assert_eq!(nonfinite.fallback_nonfinite(), out_dim);
            assert_eq!(
                input
                    .iter()
                    .map(|value| value.to_bits())
                    .collect::<Vec<_>>(),
                input_before,
                "{site:?} nonfinite fallback mutated its input"
            );
        }
    }

    for case in 0..3 {
        let mut input = vec![0.0f32; d];
        for value in &mut input {
            *value = moderate_finite(&mut seed);
        }
        if case == 1 {
            input.fill(0.0);
        } else if case == 2 {
            input[d / 2] = f32::NEG_INFINITY;
        }
        let input_before: Vec<u32> = input.iter().map(|value| value.to_bits()).collect();
        let mut exact = vec![0.0f32; model.cfg.vocab];
        let mut candidate = vec![0.0f32; model.cfg.vocab];
        poison_output(&mut exact);
        model
            .dense_control_matrix_canary(
                &mut workspace,
                None,
                Gpt2DenseControlSite::LmHead,
                &input,
                &mut exact,
                Gpt2DenseCanaryMode::Exact,
            )
            .expect("valid exact lm-head control");
        assert_output_overwritten(&exact, "exact lm-head output");
        poison_output(&mut candidate);
        let (census, allocations) = counted_allocations(|| {
            model
                .dense_control_matrix_canary(
                    &mut workspace,
                    None,
                    Gpt2DenseControlSite::LmHead,
                    &input,
                    &mut candidate,
                    Gpt2DenseCanaryMode::CertifiedNative,
                )
                .expect("valid candidate lm-head control")
        });
        assert_eq!(allocations, 0, "lm-head case {case} allocated");
        assert_output_overwritten(&candidate, "candidate lm-head output");
        assert_bits(&candidate, &exact, &format!("lm-head case {case}"));
        assert_candidate_partition(census);
        observed_fast += census.fast_certified();
        if case == 1 {
            assert_eq!(census.fallback_zero(), model.cfg.vocab);
        }
        if case == 2 {
            assert_eq!(census.fallback_nonfinite(), model.cfg.vocab);
        }
        assert_eq!(
            input
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            input_before,
            "lm-head case {case} mutated its input"
        );
    }
    assert!(
        observed_fast > 0,
        "matrix sweep never certified a fast lane"
    );
}

#[test]
fn production_dense_is_recurrently_exact_and_allocation_free() {
    let model = Gpt2::load(fixture_dir(), None).expect("load tiny GPT-2 fixture");
    let mut conventional_state = Gpt2State::new(&model.cfg);
    let mut production_state = Gpt2State::new(&model.cfg);
    let mut exact_state = Gpt2State::new(&model.cfg);
    let mut candidate_state = Gpt2State::new(&model.cfg);
    let mut conventional_workspace = model
        .dense_control_workspace()
        .expect("prepare conventional workspace");
    let mut exact_workspace = model
        .dense_control_workspace()
        .expect("prepare exact workspace");
    let mut candidate_workspace = model
        .dense_control_workspace()
        .expect("prepare candidate workspace");

    let bytes = model
        .dense_control_workspace_bytes(&candidate_workspace)
        .expect("workspace byte census");
    assert_eq!(bytes.lm_head_transpose(), 32 * 24 * 4);
    assert_eq!(bytes.matrix_bounds(), 600 * 8);
    assert_eq!(bytes.f64_sum_scratch(), (600 + 128 + 1) * 8);
    assert_eq!(bytes.intermediate_scratch(), (368 + 376) * 4);
    assert!(bytes.metadata() > 0);
    assert_eq!(
        bytes.total(),
        bytes.lm_head_transpose()
            + bytes.matrix_bounds()
            + bytes.f64_sum_scratch()
            + bytes.intermediate_scratch()
            + bytes.metadata()
    );

    let mut aggregate = DenseCensusTotal::default();
    for (position, token) in [1usize, 3, 2].into_iter().enumerate() {
        model.forward(&mut production_state, token, position, &[], &mut |_, _| {});
        let conventional = model
            .forward_dense_control(
                &mut conventional_state,
                &mut conventional_workspace,
                token,
                position,
                Gpt2DenseCanaryMode::Conventional,
            )
            .expect("valid conventional whole step");
        assert_whole_census(
            conventional,
            &model,
            1,
            0,
            Gpt2DenseCanaryMode::Conventional,
        );
        exact_state.logits.fill(f32::from_bits(OUTPUT_POISON_BITS));
        exact_state.hidden.fill(f32::from_bits(OUTPUT_POISON_BITS));
        candidate_state
            .logits
            .fill(f32::from_bits(OUTPUT_POISON_BITS));
        candidate_state
            .hidden
            .fill(f32::from_bits(OUTPUT_POISON_BITS));
        let exact = model
            .forward_dense_control(
                &mut exact_state,
                &mut exact_workspace,
                token,
                position,
                Gpt2DenseCanaryMode::Exact,
            )
            .expect("valid exact whole step");
        assert_whole_census(exact, &model, 1, 0, Gpt2DenseCanaryMode::Exact);
        let candidate = model
            .forward_dense_control(
                &mut candidate_state,
                &mut candidate_workspace,
                token,
                position,
                Gpt2DenseCanaryMode::CertifiedNative,
            )
            .expect("valid candidate whole step");
        let observed = assert_whole_census(
            candidate,
            &model,
            1,
            0,
            Gpt2DenseCanaryMode::CertifiedNative,
        );
        aggregate.calls += observed.calls;
        aggregate.lanes += observed.lanes;
        aggregate.fast += observed.fast;
        aggregate.refined += observed.refined;
        aggregate.fallback_nonfinite += observed.fallback_nonfinite;
        aggregate.fallback_zero += observed.fallback_zero;
        aggregate.fallback_overflow += observed.fallback_overflow;
        aggregate.fallback_cell += observed.fallback_cell;
        assert_output_overwritten(&candidate_state.logits, "whole candidate logits");
        assert_output_overwritten(&candidate_state.hidden, "whole candidate hidden");
        assert_state_bits(
            &candidate_state,
            &exact_state,
            &format!("candidate/exact position {position}"),
        );
        assert_state_bits(
            &production_state,
            &exact_state,
            &format!("production/exact position {position}"),
        );
    }
    assert_eq!(aggregate.calls, 3 * (4 * model.cfg.n_layer + 1));
    assert_eq!(aggregate.lanes, 3 * 600);
    assert!(
        aggregate.fast > 0,
        "whole candidate never certified a fast lane"
    );

    let position = 3;
    let token = 4;
    exact_state.logits.fill(f32::from_bits(OUTPUT_POISON_BITS));
    candidate_state
        .logits
        .fill(f32::from_bits(OUTPUT_POISON_BITS));
    production_state
        .logits
        .fill(f32::from_bits(OUTPUT_POISON_BITS));
    let (_, production_allocations) = counted_allocations(|| {
        model.forward(&mut production_state, token, position, &[], &mut |_, _| {});
    });
    assert_eq!(production_allocations, 0, "production dense step allocated");
    let (_, exact_allocations) = counted_allocations(|| {
        let census = model
            .forward_dense_control(
                &mut exact_state,
                &mut exact_workspace,
                token,
                position,
                Gpt2DenseCanaryMode::Exact,
            )
            .expect("allocation-counted exact whole step");
        assert_whole_census(census, &model, 1, 0, Gpt2DenseCanaryMode::Exact);
    });
    assert_eq!(exact_allocations, 0, "exact whole hot step allocated");
    let (_, candidate_allocations) = counted_allocations(|| {
        let census = model
            .forward_dense_control(
                &mut candidate_state,
                &mut candidate_workspace,
                token,
                position,
                Gpt2DenseCanaryMode::CertifiedNative,
            )
            .expect("allocation-counted candidate whole step");
        assert_whole_census(census, &model, 1, 0, Gpt2DenseCanaryMode::CertifiedNative);
    });
    assert_eq!(
        candidate_allocations, 0,
        "candidate whole hot step allocated"
    );
    assert_state_bits(
        &candidate_state,
        &exact_state,
        "allocation-counted whole step",
    );
    assert_state_bits(
        &production_state,
        &exact_state,
        "allocation-counted production/exact step",
    );
    assert_eq!(
        model
            .dense_control_workspace_bytes(&candidate_workspace)
            .expect("post-run workspace byte census"),
        bytes,
        "hot calls changed prepared workspace capacities"
    );
}

#[derive(Debug, Default, PartialEq, Eq)]
struct TraceBits {
    residual: Vec<(usize, Vec<u32>)>,
    qkv: Vec<QkvTraceBits>,
    attention: Vec<(usize, usize, Vec<u32>)>,
}

type QkvTraceBits = (usize, Vec<u32>, Vec<u32>, Vec<u32>);

fn bits(values: &[f32]) -> Vec<u32> {
    values.iter().map(|value| value.to_bits()).collect()
}

fn capture_production_trace(
    model: &Gpt2,
    state: &mut Gpt2State,
    token: usize,
    position: usize,
) -> TraceBits {
    let residual_layers = [0usize, 1];
    let qkv_layers = [0usize, 1];
    let attention_layers = [0usize, 1];
    let request = TraceCaptureRequest {
        residual_layers: &residual_layers,
        qkv_layers: &qkv_layers,
        attention_layers: &attention_layers,
    };
    let mut trace = TraceBits::default();
    {
        let mut residual = |layer: usize, values: &[f32]| {
            trace.residual.push((layer, bits(values)));
        };
        let mut qkv = |layer: usize, q: &[f32], k: &[f32], v: &[f32]| {
            trace.qkv.push((layer, bits(q), bits(k), bits(v)));
        };
        let mut attention = |layer: usize, head: usize, values: &[f32]| {
            trace.attention.push((layer, head, bits(values)));
        };
        let mut sinks = TraceCaptureSinks {
            residual: &mut residual,
            qkv: &mut qkv,
            attention: &mut attention,
        };
        model.forward_capturing_trace(state, token, position, &request, &mut sinks);
    }
    trace
}

fn capture_dense_trace(
    model: &Gpt2,
    state: &mut Gpt2State,
    workspace: &mut uor_r4_model_source::gpt2::Gpt2DenseControlWorkspace,
    token: usize,
    position: usize,
    mode: Gpt2DenseCanaryMode,
) -> TraceBits {
    let residual_layers = [0usize, 1];
    let qkv_layers = [0usize, 1];
    let attention_layers = [0usize, 1];
    let request = TraceCaptureRequest {
        residual_layers: &residual_layers,
        qkv_layers: &qkv_layers,
        attention_layers: &attention_layers,
    };
    let mut trace = TraceBits::default();
    {
        let mut residual = |layer: usize, values: &[f32]| {
            trace.residual.push((layer, bits(values)));
        };
        let mut qkv = |layer: usize, q: &[f32], k: &[f32], v: &[f32]| {
            trace.qkv.push((layer, bits(q), bits(k), bits(v)));
        };
        let mut attention = |layer: usize, head: usize, values: &[f32]| {
            trace.attention.push((layer, head, bits(values)));
        };
        let mut sinks = TraceCaptureSinks {
            residual: &mut residual,
            qkv: &mut qkv,
            attention: &mut attention,
        };
        let census = model
            .forward_dense_control_capturing_trace(
                state, workspace, token, position, mode, &request, &mut sinks,
            )
            .expect("valid traced dense-control step");
        assert_whole_census(census, model, 1, 0, mode);
    }
    trace
}

#[test]
fn dense_trace_uses_the_same_owner_and_preserves_every_tap_bit() {
    let model = Gpt2::load(fixture_dir(), None).expect("load tiny GPT-2 fixture");
    let mut production_state = Gpt2State::new(&model.cfg);
    let mut conventional_state = Gpt2State::new(&model.cfg);
    let mut exact_state = Gpt2State::new(&model.cfg);
    let mut candidate_state = Gpt2State::new(&model.cfg);
    let mut plain_candidate_state = Gpt2State::new(&model.cfg);
    let mut conventional_workspace = model.dense_control_workspace().expect("workspace");
    let mut exact_workspace = model.dense_control_workspace().expect("workspace");
    let mut candidate_workspace = model.dense_control_workspace().expect("workspace");
    let mut plain_workspace = model.dense_control_workspace().expect("workspace");

    let production = capture_production_trace(&model, &mut production_state, 3, 0);
    let conventional = capture_dense_trace(
        &model,
        &mut conventional_state,
        &mut conventional_workspace,
        3,
        0,
        Gpt2DenseCanaryMode::Conventional,
    );
    assert_ne!(
        conventional, production,
        "conventional dense unexpectedly equals v2"
    );

    let exact = capture_dense_trace(
        &model,
        &mut exact_state,
        &mut exact_workspace,
        3,
        0,
        Gpt2DenseCanaryMode::Exact,
    );
    let candidate = capture_dense_trace(
        &model,
        &mut candidate_state,
        &mut candidate_workspace,
        3,
        0,
        Gpt2DenseCanaryMode::CertifiedNative,
    );
    assert_eq!(candidate, exact);
    assert_state_bits(&candidate_state, &exact_state, "traced candidate/exact");
    assert_eq!(candidate, production);
    assert_state_bits(
        &candidate_state,
        &production_state,
        "traced candidate/production",
    );

    let census = model
        .forward_dense_control(
            &mut plain_candidate_state,
            &mut plain_workspace,
            3,
            0,
            Gpt2DenseCanaryMode::CertifiedNative,
        )
        .expect("plain candidate step");
    assert_whole_census(census, &model, 1, 0, Gpt2DenseCanaryMode::CertifiedNative);
    assert_state_bits(
        &plain_candidate_state,
        &candidate_state,
        "trace/plain candidate",
    );

    let mut allocation_state = Gpt2State::new(&model.cfg);
    let residual_layers = [0usize, 1];
    let qkv_layers = [0usize, 1];
    let attention_layers = [0usize, 1];
    let request = TraceCaptureRequest {
        residual_layers: &residual_layers,
        qkv_layers: &qkv_layers,
        attention_layers: &attention_layers,
    };
    let mut residual = |_: usize, _: &[f32]| {};
    let mut qkv = |_: usize, _: &[f32], _: &[f32], _: &[f32]| {};
    let mut attention = |_: usize, _: usize, _: &[f32]| {};
    let mut sinks = TraceCaptureSinks {
        residual: &mut residual,
        qkv: &mut qkv,
        attention: &mut attention,
    };
    let (_, allocations) = counted_allocations(|| {
        model.forward_capturing_trace(&mut allocation_state, 3, 0, &request, &mut sinks);
    });
    assert_eq!(allocations, 0, "production trace hot step allocated");
}

#[test]
fn production_dense_model_is_shareable_and_state_scratch_is_independent() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Gpt2>();
    assert_send_sync::<Gpt2State>();

    let model = Gpt2::load(fixture_dir(), None).expect("load tiny GPT-2 fixture");
    let mut first = Gpt2State::new(&model.cfg);
    let second = Gpt2State::new(&model.cfg);
    let second_before = second.clone();
    model.forward(&mut first, 1, 0, &[], &mut |_, _| {});
    assert_eq!(
        second, second_before,
        "one state mutated another state scratch"
    );
}

#[test]
fn production_dense_validation_is_failure_atomic() {
    let model = Gpt2::load(fixture_dir(), None).expect("load tiny GPT-2 fixture");

    let mut malformed = Gpt2State::new(&model.cfg);
    malformed.logits.pop();
    let malformed_before = malformed.clone();
    model.forward(&mut malformed, 1, 0, &[], &mut |_, _| {});
    assert_eq!(malformed, malformed_before);

    let mut bad_token = Gpt2State::new(&model.cfg);
    let bad_token_before = bad_token.clone();
    let mut sink_hits = 0usize;
    model.forward(&mut bad_token, model.cfg.vocab, 0, &[0], &mut |_, _| {
        sink_hits += 1
    });
    assert_eq!(bad_token, bad_token_before);
    assert_eq!(sink_hits, 0);

    let mut traced = Gpt2State::new(&model.cfg);
    let traced_before = traced.clone();
    let invalid_layer = [model.cfg.n_layer];
    let request = TraceCaptureRequest {
        residual_layers: &invalid_layer,
        qkv_layers: &[],
        attention_layers: &[],
    };
    let mut residual_hits = 0usize;
    {
        let mut residual = |_: usize, _: &[f32]| residual_hits += 1;
        let mut qkv = |_: usize, _: &[f32], _: &[f32], _: &[f32]| {};
        let mut attention = |_: usize, _: usize, _: &[f32]| {};
        let mut sinks = TraceCaptureSinks {
            residual: &mut residual,
            qkv: &mut qkv,
            attention: &mut attention,
        };
        model.forward_capturing_trace(&mut traced, 1, 0, &request, &mut sinks);
    }
    assert_eq!(traced, traced_before);
    assert_eq!(residual_hits, 0);

    let mut states = vec![Gpt2State::new(&model.cfg); 3];
    states[2].logits.pop();
    let states_before = states.clone();
    model.forward_batch(&mut states, &[1, 2, 3], &[0, 0, 0]);
    assert_eq!(states, states_before);

    let mut states = vec![Gpt2State::new(&model.cfg); 3];
    let states_before = states.clone();
    model.forward_batch(&mut states, &[1, 2, model.cfg.vocab], &[0, 0, 0]);
    assert_eq!(states, states_before);
    model.forward_batch(&mut states, &[1, 2], &[0, 0, 0]);
    assert_eq!(states, states_before);
}

fn prefill_dense_states(
    model: &Gpt2,
    histories: &[&[usize]],
    mode: Gpt2DenseCanaryMode,
) -> Vec<Gpt2State> {
    let mut states = vec![Gpt2State::new(&model.cfg); histories.len()];
    let mut workspace = model.dense_control_workspace().expect("serial workspace");
    for (state, history) in states.iter_mut().zip(histories) {
        for (position, &token) in history.iter().enumerate() {
            let census = model
                .forward_dense_control(state, &mut workspace, token, position, mode)
                .expect("valid dense prefill step");
            assert_whole_census(census, model, 1, 0, mode);
        }
    }
    states
}

fn prefill_production_states(model: &Gpt2, histories: &[&[usize]]) -> Vec<Gpt2State> {
    let mut states = vec![Gpt2State::new(&model.cfg); histories.len()];
    for (state, history) in states.iter_mut().zip(histories) {
        for (position, &token) in history.iter().enumerate() {
            model.forward(state, token, position, &[], &mut |_, _| {});
        }
    }
    states
}

#[test]
fn row_reuse_batch_control_matches_serial_for_all_modes_and_divergent_positions() {
    let model = Gpt2::load(fixture_dir(), None).expect("load tiny GPT-2 fixture");
    let histories: [&[usize]; 3] = [&[1], &[2, 3], &[4, 5, 6]];
    let tokens = [7usize, 8, 9];
    let positions = [1usize, 2, 3];

    for mode in [
        Gpt2DenseCanaryMode::Conventional,
        Gpt2DenseCanaryMode::Exact,
        Gpt2DenseCanaryMode::CertifiedNative,
    ] {
        let base = prefill_dense_states(&model, &histories, mode);
        let mut serial = base.clone();
        let mut batched = base;
        let mut serial_workspace = model.dense_control_workspace().expect("serial workspace");
        let mut batch_workspace = model
            .dense_control_workspace_for_batch(3)
            .expect("three-row workspace");
        for state_index in 0..serial.len() {
            let census = model
                .forward_dense_control(
                    &mut serial[state_index],
                    &mut serial_workspace,
                    tokens[state_index],
                    positions[state_index],
                    mode,
                )
                .expect("valid serial reference step");
            assert_whole_census(census, &model, 1, 0, mode);
        }
        let (_, allocations) = counted_allocations(|| {
            let census = model
                .forward_batch_dense_control(
                    &mut batched,
                    &tokens,
                    &positions,
                    &mut batch_workspace,
                    mode,
                )
                .expect("valid row-reuse batch step");
            assert_whole_census(census, &model, 3, 3, mode);
        });
        assert_eq!(allocations, 0, "{mode:?} row-reuse batch allocated");
        for state_index in 0..serial.len() {
            assert_state_bits(
                &batched[state_index],
                &serial[state_index],
                &format!("{mode:?} batch/serial state {state_index}"),
            );
        }
    }

    let lockstep_histories: [&[usize]; 3] = [&[10], &[11], &[12]];
    let lockstep_tokens = [13usize, 14, 15];
    let lockstep_positions = [1usize, 1, 1];
    for mode in [
        Gpt2DenseCanaryMode::Conventional,
        Gpt2DenseCanaryMode::Exact,
        Gpt2DenseCanaryMode::CertifiedNative,
    ] {
        let base = prefill_dense_states(&model, &lockstep_histories, mode);
        let mut serial = base.clone();
        let mut batched = base;
        let mut serial_workspace = model.dense_control_workspace().expect("serial workspace");
        let mut batch_workspace = model
            .dense_control_workspace_for_batch(3)
            .expect("lockstep row-reuse workspace");
        for state_index in 0..serial.len() {
            let census = model
                .forward_dense_control(
                    &mut serial[state_index],
                    &mut serial_workspace,
                    lockstep_tokens[state_index],
                    lockstep_positions[state_index],
                    mode,
                )
                .expect("valid lockstep serial step");
            assert_whole_census(census, &model, 1, 0, mode);
        }
        let census = model
            .forward_batch_dense_control(
                &mut batched,
                &lockstep_tokens,
                &lockstep_positions,
                &mut batch_workspace,
                mode,
            )
            .expect("valid lockstep row-reuse batch step");
        assert_whole_census(census, &model, 3, 3, mode);
        for state_index in 0..serial.len() {
            assert_state_bits(
                &batched[state_index],
                &serial[state_index],
                &format!("{mode:?} lockstep batch/serial state {state_index}"),
            );
        }
        if mode == Gpt2DenseCanaryMode::CertifiedNative {
            let mut production = prefill_production_states(&model, &lockstep_histories);
            let (_, allocations) = counted_allocations(|| {
                model.forward_batch(&mut production, &lockstep_tokens, &lockstep_positions);
            });
            assert_eq!(allocations, 0, "production lockstep batch allocated");
            for state_index in 0..production.len() {
                assert_state_bits(
                    &batched[state_index],
                    &production[state_index],
                    &format!("candidate/production lockstep state {state_index}"),
                );
            }
        }
    }

    let mut production = prefill_production_states(&model, &histories);
    let mut exact = prefill_dense_states(&model, &histories, Gpt2DenseCanaryMode::Exact);
    for state_index in 0..production.len() {
        assert_state_bits(
            &exact[state_index],
            &production[state_index],
            &format!("exact/production prefill {state_index}"),
        );
    }
    let (_, production_allocations) = counted_allocations(|| {
        model.forward_batch(&mut production, &tokens, &positions);
    });
    assert_eq!(
        production_allocations, 0,
        "production divergent batch allocated"
    );
    let mut workspace = model
        .dense_control_workspace_for_batch(3)
        .expect("three-row exact workspace");
    let census = model
        .forward_batch_dense_control(
            &mut exact,
            &tokens,
            &positions,
            &mut workspace,
            Gpt2DenseCanaryMode::Exact,
        )
        .expect("valid exact batch control");
    assert_whole_census(census, &model, 3, 3, Gpt2DenseCanaryMode::Exact);
    for state_index in 0..production.len() {
        assert_state_bits(
            &exact[state_index],
            &production[state_index],
            &format!("exact/production batch {state_index}"),
        );
    }
}

#[test]
fn whole_dense_checked_facades_are_failure_atomic() {
    let model = Gpt2::load(fixture_dir(), None).expect("load tiny GPT-2 fixture");
    let other = Gpt2::load(fixture_dir(), None).expect("load independent fixture model");
    let mut state = Gpt2State::new(&model.cfg);
    let state_before = state.clone();
    let mut foreign_workspace = other.dense_control_workspace().expect("foreign workspace");
    let workspace_before = other
        .dense_control_workspace_fingerprint(&foreign_workspace)
        .expect("foreign workspace fingerprint");
    assert!(model
        .forward_dense_control(
            &mut state,
            &mut foreign_workspace,
            1,
            0,
            Gpt2DenseCanaryMode::CertifiedNative,
        )
        .is_none());
    assert_state_bits(&state, &state_before, "foreign-workspace state");
    assert_eq!(
        other
            .dense_control_workspace_fingerprint(&foreign_workspace)
            .expect("post-rejection foreign workspace fingerprint"),
        workspace_before
    );

    let mut workspace = model
        .dense_control_workspace_for_batch(3)
        .expect("three-row workspace");
    for (token, position) in [(model.cfg.vocab, 0), (1, model.cfg.seq_len)] {
        let mut invalid_state = Gpt2State::new(&model.cfg);
        let state_before = invalid_state.clone();
        let workspace_before = model
            .dense_control_workspace_fingerprint(&workspace)
            .expect("workspace fingerprint");
        assert!(model
            .forward_dense_control(
                &mut invalid_state,
                &mut workspace,
                token,
                position,
                Gpt2DenseCanaryMode::CertifiedNative,
            )
            .is_none());
        assert_state_bits(&invalid_state, &state_before, "invalid serial extent");
        assert_eq!(
            model
                .dense_control_workspace_fingerprint(&workspace)
                .expect("post-rejection workspace fingerprint"),
            workspace_before
        );
    }

    let mut malformed = vec![Gpt2State::new(&model.cfg); 3];
    malformed[2].logits.pop();
    let malformed_before = malformed.clone();
    let workspace_before = model
        .dense_control_workspace_fingerprint(&workspace)
        .expect("workspace fingerprint");
    assert!(model
        .forward_batch_dense_control(
            &mut malformed,
            &[1, 2, 3],
            &[0, 0, 0],
            &mut workspace,
            Gpt2DenseCanaryMode::CertifiedNative,
        )
        .is_none());
    assert_eq!(malformed, malformed_before);
    assert_eq!(
        model
            .dense_control_workspace_fingerprint(&workspace)
            .expect("post-rejection workspace fingerprint"),
        workspace_before
    );

    for (tokens, positions) in [
        (vec![1, 2, model.cfg.vocab], vec![0, 0, 0]),
        (vec![1, 2, 3], vec![0, 0, model.cfg.seq_len]),
        (vec![1, 2], vec![0, 0, 0]),
    ] {
        let mut states = vec![Gpt2State::new(&model.cfg); 3];
        let states_before = states.clone();
        let workspace_before = model
            .dense_control_workspace_fingerprint(&workspace)
            .expect("workspace fingerprint");
        assert!(model
            .forward_batch_dense_control(
                &mut states,
                &tokens,
                &positions,
                &mut workspace,
                Gpt2DenseCanaryMode::CertifiedNative,
            )
            .is_none());
        assert_eq!(states, states_before);
        assert_eq!(
            model
                .dense_control_workspace_fingerprint(&workspace)
                .expect("post-rejection workspace fingerprint"),
            workspace_before
        );
    }

    let mut four_states = vec![Gpt2State::new(&model.cfg); 4];
    let four_before = four_states.clone();
    let workspace_before = model
        .dense_control_workspace_fingerprint(&workspace)
        .expect("workspace fingerprint");
    assert!(model
        .forward_batch_dense_control(
            &mut four_states,
            &[1, 2, 3, 4],
            &[0, 0, 0, 0],
            &mut workspace,
            Gpt2DenseCanaryMode::CertifiedNative,
        )
        .is_none());
    assert_eq!(four_states, four_before);
    assert_eq!(
        model
            .dense_control_workspace_fingerprint(&workspace)
            .expect("post-rejection workspace fingerprint"),
        workspace_before
    );

    let bad_layer = [model.cfg.n_layer];
    let request = TraceCaptureRequest {
        residual_layers: &bad_layer,
        qkv_layers: &[],
        attention_layers: &[],
    };
    let sink_hits = Cell::new(0usize);
    let mut residual = |_: usize, _: &[f32]| sink_hits.set(sink_hits.get() + 1);
    let mut qkv = |_: usize, _: &[f32], _: &[f32], _: &[f32]| {
        sink_hits.set(sink_hits.get() + 1);
    };
    let mut attention = |_: usize, _: usize, _: &[f32]| sink_hits.set(sink_hits.get() + 1);
    let mut sinks = TraceCaptureSinks {
        residual: &mut residual,
        qkv: &mut qkv,
        attention: &mut attention,
    };
    let mut traced = Gpt2State::new(&model.cfg);
    let traced_before = traced.clone();
    let workspace_before = model
        .dense_control_workspace_fingerprint(&workspace)
        .expect("workspace fingerprint");
    assert!(model
        .forward_dense_control_capturing_trace(
            &mut traced,
            &mut workspace,
            1,
            0,
            Gpt2DenseCanaryMode::CertifiedNative,
            &request,
            &mut sinks,
        )
        .is_none());
    assert_eq!(sink_hits.get(), 0);
    assert_state_bits(&traced, &traced_before, "invalid trace request");
    assert_eq!(
        model
            .dense_control_workspace_fingerprint(&workspace)
            .expect("post-rejection workspace fingerprint"),
        workspace_before
    );

    let input = vec![0.25f32; model.cfg.n_embd - 1];
    let mut output = vec![f32::from_bits(OUTPUT_POISON_BITS); 3 * model.cfg.n_embd];
    let output_before = output.clone();
    let workspace_before = model
        .dense_control_workspace_fingerprint(&workspace)
        .expect("workspace fingerprint");
    assert!(model
        .dense_control_matrix_canary(
            &mut workspace,
            Some(0),
            Gpt2DenseControlSite::CAttn,
            &input,
            &mut output,
            Gpt2DenseCanaryMode::CertifiedNative,
        )
        .is_none());
    assert_bits(&output, &output_before, "invalid matrix extent");
    assert_eq!(
        model
            .dense_control_workspace_fingerprint(&workspace)
            .expect("post-rejection workspace fingerprint"),
        workspace_before
    );
}

fn median_nine(mut values: [f64; REAL_PAIRS]) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

fn empirical_bootstrap_median_upper_95_nine(mut values: [f64; REAL_PAIRS]) -> f64 {
    assert!(values
        .iter()
        .all(|value| value.is_finite() && *value >= 0.0));
    values.sort_by(f64::total_cmp);
    const CHOOSE_NINE: [u128; 10] = [1, 9, 36, 84, 126, 126, 84, 36, 9, 1];
    const TOTAL_RESAMPLES: u128 = 9u128.pow(9);

    // For one empirical value `v`, let `j` observations be <= v. Across all
    // 9^9 equiprobable bootstrap resamples, the resampled median is <= v iff
    // at least five draws land among those j observations. Count that event
    // exactly with the binomial polynomial and take the smallest v whose CDF
    // is at least 95%; no RNG or floating probability comparison participates.
    for &candidate in &values {
        let less_or_equal = values.iter().filter(|&&value| value <= candidate).count() as u128;
        let greater = REAL_PAIRS as u128 - less_or_equal;
        let cumulative: u128 = (5..=9)
            .map(|successes| {
                CHOOSE_NINE[successes]
                    * less_or_equal.pow(successes as u32)
                    * greater.pow((9 - successes) as u32)
            })
            .sum();
        if cumulative * 100 >= TOTAL_RESAMPLES * 95 {
            return candidate;
        }
    }
    unreachable!("the maximum empirical value has bootstrap CDF one")
}

#[test]
fn exact_empirical_bootstrap_upper_95_has_the_expected_nine_sample_rank() {
    let ordered = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
    assert_eq!(median_nine(ordered), 5.0);
    assert_eq!(empirical_bootstrap_median_upper_95_nine(ordered), 7.0);

    let permuted = [9.0, 1.0, 6.0, 3.0, 8.0, 2.0, 7.0, 5.0, 4.0];
    assert_eq!(empirical_bootstrap_median_upper_95_nine(permuted), 7.0);

    // Ties are grouped through the empirical CDF, not broken by array index.
    assert_eq!(
        empirical_bootstrap_median_upper_95_nine([1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 4.0]),
        4.0
    );
}

fn assert_exact_control_census(census: Gpt2DenseCanaryCensus, expected_lanes: usize) {
    assert_eq!(census.lanes(), expected_lanes);
    assert_eq!(census.exact_control(), expected_lanes);
    assert_eq!(census.conventional(), 0);
    assert_eq!(census.fast_certified(), 0);
    assert_eq!(census.refined_certified(), 0);
    assert_eq!(census.fallbacks(), Some(0));
}

fn assert_conventional_census(census: Gpt2DenseCanaryCensus, expected_lanes: usize) {
    assert_eq!(census.lanes(), expected_lanes);
    assert_eq!(census.conventional(), expected_lanes);
    assert_eq!(census.exact_control(), 0);
    assert_eq!(census.fast_certified(), 0);
    assert_eq!(census.refined_certified(), 0);
    assert_eq!(census.fallbacks(), Some(0));
}

fn timed_dense_call(
    model: &Gpt2,
    input: &[f32],
    output: &mut [f32],
    workspace: &mut uor_r4_model_source::gpt2::Gpt2DenseCanaryWorkspace,
    mode: Gpt2DenseCanaryMode,
) -> (Duration, Gpt2DenseCanaryCensus) {
    let ((elapsed, census), allocations) = counted_allocations(|| {
        let started = Instant::now();
        let census = model
            .layer0_c_attn_canary(input, output, workspace, mode)
            .expect("hard-bound timed dense call is valid");
        (started.elapsed(), census)
    });
    assert_eq!(allocations, 0, "timed dense call allocated");
    (elapsed, census)
}

/// Cheap consumer hard gate only. Workspace preparation and exact preflight
/// are outside timing; nine alternating matched inputs measure historical
/// Conv1D against the certified-native candidate. The verdict is the strict
/// one-sided 95% exact empirical-bootstrap upper quantile of the paired median.
#[test]
#[ignore = "requires the real pinned GPT-2 124M source and is a local #704 timing gate"]
#[allow(clippy::assertions_on_constants)]
fn real_layer0_c_attn_certified_native_consumer_gate() {
    let root = workspace_root();
    assert_eq!(
        sha256_file(&root.join("crates/uor-r4-model-source/src/gpt2.rs")),
        LEGACY_C_ATTN_GPT2_SHA256,
        "the legacy c_attn gate is superseded: refuse to time changed whole-owner source under the old contract"
    );
    assert_eq!(
        sha256_file(&root.join("crates/uor-r4-model-source/tests/certified_native_dense.rs")),
        LEGACY_C_ATTN_HARNESS_SHA256,
        "the legacy c_attn gate is superseded: refuse to time a changed harness under the old contract"
    );
    assert!(!cfg!(debug_assertions), "run the dense gate with --release");
    let source = real_source();
    for required in ["model.safetensors", "config.json", "source_manifest.json"] {
        assert!(
            source.join(required).is_file(),
            "hard-bound real GPT-2 fixture is missing {} (set UOR_GPT2_SOURCE or install the repo-relative fixture)",
            source.join(required).display()
        );
    }
    let model = Gpt2::load(&source, Some(1)).expect("load real pinned GPT-2");
    bind_dense_identity(
        &source,
        &model,
        DenseGateContract {
            arithmetic_id: DENSE_ARITHMETIC_ID,
            bootstrap_id: BOOTSTRAP_ALGORITHM_ID,
            pairs: REAL_PAIRS,
            warmups_per_arm: REAL_WARMUPS_PER_ARM,
            threshold: MAXIMUM_UPPER_RATIO,
            threshold_rule: "strict-less-than",
        },
    );
    assert_eq!(model.cfg.n_head, 12);
    assert_eq!(model.cfg.n_layer, 12);
    let d = model.cfg.n_embd;
    let out_dim = 3 * d;
    assert_eq!((d, out_dim), (768, REAL_OUTPUT_LANES));

    let mut workspace = model
        .dense_canary_workspace()
        .expect("real model admits prepared dense scratch");
    let mut inputs = vec![vec![0.0f32; d]; REAL_PAIRS];
    for (token, input) in inputs.iter_mut().enumerate() {
        model
            .layer0_c_attn_canary_input(token, 0, input)
            .expect("hard-bound real input is valid");
    }

    let mut exact = vec![0.0f32; out_dim];
    let mut candidate = vec![0.0f32; out_dim];
    let mut conventional = vec![0.0f32; out_dim];
    let mut expected_outputs = Vec::with_capacity(REAL_PAIRS);
    let mut preflight_census = DenseCensusTotal::default();
    for (case, input) in inputs.iter().enumerate() {
        poison_output(&mut exact);
        let exact_census = model
            .layer0_c_attn_canary(
                input,
                &mut exact,
                &mut workspace,
                Gpt2DenseCanaryMode::Exact,
            )
            .expect("exact preflight");
        assert_exact_control_census(exact_census, out_dim);
        assert_output_overwritten(&exact, &format!("real exact preflight {case}"));
        expected_outputs.push(exact.clone());

        poison_output(&mut candidate);
        let census = model
            .layer0_c_attn_canary(
                input,
                &mut candidate,
                &mut workspace,
                Gpt2DenseCanaryMode::CertifiedNative,
            )
            .expect("candidate preflight");
        assert_output_overwritten(&candidate, &format!("real candidate preflight {case}"));
        assert_bits(
            &candidate,
            &expected_outputs[case],
            &format!("real candidate/exact preflight {case}"),
        );
        preflight_census.observe_candidate(census, out_dim);
    }
    assert!(
        preflight_census.fast > 0,
        "real preflight has no fast-certified lane"
    );
    assert!(
        preflight_census.refined > 0,
        "real preflight never exercised TwoSum refinement"
    );
    assert_eq!(
        preflight_census.fallbacks(),
        0,
        "real preflight required exact fallback"
    );

    // Fixed predeclared warmup: exactly 20 calls per arm, candidate then
    // conventional for each deterministic input in round-robin order.
    let mut warmup_census = DenseCensusTotal::default();
    for warmup in 0..REAL_WARMUPS_PER_ARM {
        let input_index = warmup % REAL_PAIRS;
        poison_output(&mut candidate);
        let census = model
            .layer0_c_attn_canary(
                &inputs[input_index],
                &mut candidate,
                &mut workspace,
                Gpt2DenseCanaryMode::CertifiedNative,
            )
            .expect("candidate warmup");
        warmup_census.observe_candidate(census, out_dim);
        assert_output_overwritten(&candidate, &format!("candidate warmup {warmup}"));
        assert_bits(
            &candidate,
            &expected_outputs[input_index],
            &format!("candidate/exact warmup {warmup}"),
        );

        poison_output(&mut conventional);
        let census = model
            .layer0_c_attn_canary(
                &inputs[input_index],
                &mut conventional,
                &mut workspace,
                Gpt2DenseCanaryMode::Conventional,
            )
            .expect("conventional warmup");
        assert_conventional_census(census, out_dim);
        assert_output_overwritten(&conventional, &format!("conventional warmup {warmup}"));
    }
    assert_eq!(warmup_census.calls, REAL_WARMUPS_PER_ARM);
    assert_eq!(warmup_census.fallbacks(), 0);
    eprintln!(
        "CERTIFIED_DENSE_WARMUP calls_per_arm={REAL_WARMUPS_PER_ARM} order=candidate-then-conventional census={warmup_census:?}"
    );

    let mut candidate_seconds = [0.0f64; REAL_PAIRS];
    let mut conventional_seconds = [0.0f64; REAL_PAIRS];
    let mut timed_census = DenseCensusTotal::default();
    for (pair, input) in inputs.iter().enumerate() {
        let candidate_first = pair.is_multiple_of(2);
        let (candidate_elapsed, candidate_census, control_elapsed, control_census);
        if pair.is_multiple_of(2) {
            poison_output(&mut candidate);
            (candidate_elapsed, candidate_census) = timed_dense_call(
                &model,
                input,
                &mut candidate,
                &mut workspace,
                Gpt2DenseCanaryMode::CertifiedNative,
            );
            poison_output(&mut conventional);
            (control_elapsed, control_census) = timed_dense_call(
                &model,
                input,
                &mut conventional,
                &mut workspace,
                Gpt2DenseCanaryMode::Conventional,
            );
        } else {
            poison_output(&mut conventional);
            (control_elapsed, control_census) = timed_dense_call(
                &model,
                input,
                &mut conventional,
                &mut workspace,
                Gpt2DenseCanaryMode::Conventional,
            );
            poison_output(&mut candidate);
            (candidate_elapsed, candidate_census) = timed_dense_call(
                &model,
                input,
                &mut candidate,
                &mut workspace,
                Gpt2DenseCanaryMode::CertifiedNative,
            );
        }

        assert_output_overwritten(&candidate, &format!("timed candidate pair {pair}"));
        assert_bits(
            &candidate,
            &expected_outputs[pair],
            &format!("timed candidate/exact pair {pair}"),
        );
        timed_census.observe_candidate(candidate_census, out_dim);
        assert_conventional_census(control_census, out_dim);
        assert_output_overwritten(&conventional, &format!("timed conventional pair {pair}"));
        std::hint::black_box(&candidate);
        std::hint::black_box(&conventional);
        candidate_seconds[pair] = candidate_elapsed.as_secs_f64();
        conventional_seconds[pair] = control_elapsed.as_secs_f64();
        eprintln!(
            "CERTIFIED_DENSE_SAMPLE pair={pair} first={} candidate_ns={} conventional_ns={} paired_ratio={:.9} candidate_census={candidate_census:?}",
            if candidate_first {
                "candidate"
            } else {
                "conventional"
            },
            candidate_elapsed.as_nanos(),
            control_elapsed.as_nanos(),
            candidate_seconds[pair] / conventional_seconds[pair],
        );
    }
    assert_eq!(timed_census.calls, REAL_PAIRS);
    assert_eq!(timed_census.lanes, REAL_PAIRS * REAL_OUTPUT_LANES);
    assert!(
        timed_census.fast > 0,
        "timed run has no fast-certified lane"
    );
    assert!(
        timed_census.refined > 0,
        "timed run never exercised TwoSum refinement"
    );
    assert_eq!(
        timed_census.fallbacks(),
        0,
        "pinned real timed gate used an exact fallback"
    );

    let paired_ratios =
        std::array::from_fn(|pair| candidate_seconds[pair] / conventional_seconds[pair]);
    let paired_median_ratio = median_nine(paired_ratios);
    let candidate_median = median_nine(candidate_seconds);
    let conventional_median = median_nine(conventional_seconds);
    let ratio_of_medians = candidate_median / conventional_median;
    let bootstrap_median_upper_95 = empirical_bootstrap_median_upper_95_nine(paired_ratios);
    eprintln!(
        "CERTIFIED_DENSE_RESULT arithmetic_id={DENSE_ARITHMETIC_ID} bootstrap_algorithm_id={BOOTSTRAP_ALGORITHM_ID} base_head={BASE_HEAD} uor_matmul_rev={UOR_MATMUL_REV} pairs={REAL_PAIRS} warmups_per_arm={REAL_WARMUPS_PER_ARM} candidate_seconds={candidate_seconds:?} conventional_seconds={conventional_seconds:?} paired_ratios={paired_ratios:?} candidate_median_ns={:.0} conventional_median_ns={:.0} paired_median_ratio={paired_median_ratio:.9} ratio_of_medians={ratio_of_medians:.9} bootstrap_median_upper_95={bootstrap_median_upper_95:.9} timed_census={timed_census:?} preflight_census={preflight_census:?} threshold={MAXIMUM_UPPER_RATIO:.1} threshold_rule=strict-less-than",
        candidate_median * 1e9,
        conventional_median * 1e9,
    );
    assert!(
        bootstrap_median_upper_95 < MAXIMUM_UPPER_RATIO,
        "real layer-0 c_attn one-sided 95% bootstrap median upper {bootstrap_median_upper_95:.9} is not strictly below {MAXIMUM_UPPER_RATIO:.1}"
    );
}

const WHOLE_STORIES: [&[usize]; 3] = [
    &[464, 3290, 373, 257],
    &[15496, 995],
    &[50256, 464, 968, 1971, 318],
];

#[derive(Clone, Copy)]
struct WholeCensusSnapshot {
    layers: [Gpt2DenseLayerCensus; 12],
    lm_head: Gpt2DenseCanaryCensus,
}

fn merge_dense_total(total: &mut DenseCensusTotal, observed: DenseCensusTotal) {
    total.calls += observed.calls;
    total.lanes += observed.lanes;
    total.fast += observed.fast;
    total.refined += observed.refined;
    total.fallback_nonfinite += observed.fallback_nonfinite;
    total.fallback_zero += observed.fallback_zero;
    total.fallback_overflow += observed.fallback_overflow;
    total.fallback_cell += observed.fallback_cell;
}

fn assert_real_candidate_structure(total: DenseCensusTotal, context: &str) {
    assert_eq!(total.calls, WHOLE_OWNER_CALLS, "{context}: owner calls");
    assert_eq!(total.lanes, WHOLE_LANES, "{context}: output lanes");
    assert!(total.fast > 0, "{context}: no fast-certified lane");
    assert!(total.refined > 0, "{context}: no refined-certified lane");
    assert_eq!(
        total.fallbacks(),
        0,
        "{context}: exact fallback must be priced before timing"
    );
}

fn measured_whole_dense_call(
    model: &Gpt2,
    state: &mut Gpt2State,
    workspace: &mut uor_r4_model_source::gpt2::Gpt2DenseControlWorkspace,
    token: usize,
    position: usize,
    mode: Gpt2DenseCanaryMode,
) -> (Duration, WholeCensusSnapshot) {
    ALLOCATIONS.with(|count| count.set(0));
    COUNT_ALLOCATIONS.with(|gate| {
        assert!(!gate.replace(true), "nested whole-dense measurement");
    });
    let measurement = AllocationMeasurement;
    let started = Instant::now();
    let census = model
        .forward_dense_control(state, workspace, token, position, mode)
        .expect("hard-bound whole-dense call is valid");
    let elapsed = started.elapsed();
    drop(measurement);
    let allocations = ALLOCATIONS.with(Cell::get);
    assert_eq!(allocations, 0, "whole-dense hot call allocated");
    assert_eq!(census.layers().len(), 12, "hard-bound GPT-2 layer count");
    let mut layers = [Gpt2DenseLayerCensus::default(); 12];
    layers.copy_from_slice(census.layers());
    (
        elapsed,
        WholeCensusSnapshot {
            layers,
            lm_head: census.lm_head(),
        },
    )
}

fn timed_whole_suite(
    model: &Gpt2,
    state: &mut Gpt2State,
    workspace: &mut uor_r4_model_source::gpt2::Gpt2DenseControlWorkspace,
    mode: Gpt2DenseCanaryMode,
    expected_steps: &[Gpt2State],
) -> (Duration, DenseCensusTotal) {
    let mut elapsed = Duration::ZERO;
    let mut total = DenseCensusTotal::default();
    let mut expected_index = 0usize;
    for story in WHOLE_STORIES {
        state.reset();
        for (position, &token) in story.iter().enumerate() {
            state.logits.fill(f32::from_bits(OUTPUT_POISON_BITS));
            state.hidden.fill(f32::from_bits(OUTPUT_POISON_BITS));
            let (call_elapsed, census) =
                measured_whole_dense_call(model, state, workspace, token, position, mode);
            elapsed += call_elapsed;
            let observed =
                assert_whole_census_parts(&census.layers, census.lm_head, model, 1, 0, mode);
            merge_dense_total(&mut total, observed);
            assert_output_overwritten(&state.logits, "timed whole logits");
            assert_output_overwritten(&state.hidden, "timed whole hidden");
            if mode == Gpt2DenseCanaryMode::CertifiedNative {
                assert_state_bits(
                    state,
                    &expected_steps[expected_index],
                    &format!("timed candidate step {expected_index}"),
                );
            }
            expected_index += 1;
        }
    }
    assert_eq!(expected_index, WHOLE_STEPS);
    (elapsed, total)
}

/// Frozen binding whole-GPT-2 PASS contract. Its reviewed source/harness
/// hashes predate production integration, so the first checks deliberately
/// refuse every rerun rather than timing changed bytes under the old result.
#[test]
#[ignore = "binding whole-dense PASS is frozen; production integration supersedes this timing candidate"]
#[allow(clippy::assertions_on_constants)]
fn real_whole_gpt2_certified_dense_consumer_gate() {
    assert!(
        !cfg!(debug_assertions),
        "run the whole dense gate with --release"
    );
    assert_eq!(
        WHOLE_STORIES.iter().map(|story| story.len()).sum::<usize>(),
        WHOLE_STEPS
    );
    let root = workspace_root();
    assert_eq!(
        sha256_file(&root.join("crates/uor-r4-model-source/src/gpt2.rs")),
        BINDING_WHOLE_GPT2_SHA256,
        "binding whole-dense PASS is frozen: refuse to rerun after production integration"
    );
    assert_eq!(
        sha256_file(&root.join("crates/uor-r4-model-source/tests/certified_native_dense.rs")),
        BINDING_WHOLE_HARNESS_SHA256,
        "binding whole-dense PASS harness is frozen: refuse to rerun changed evidence"
    );
    let expected_gpt2_sha256 = std::env::var(WHOLE_EXPECTED_GPT2_SHA256_ENV)
        .unwrap_or_else(|_| panic!("whole dense gate requires {WHOLE_EXPECTED_GPT2_SHA256_ENV}"));
    let expected_harness_sha256 =
        std::env::var(WHOLE_EXPECTED_HARNESS_SHA256_ENV).unwrap_or_else(|_| {
            panic!("whole dense gate requires {WHOLE_EXPECTED_HARNESS_SHA256_ENV}")
        });
    let gpt2_sha256 = sha256_file(&root.join("crates/uor-r4-model-source/src/gpt2.rs"));
    let harness_sha256 =
        sha256_file(&root.join("crates/uor-r4-model-source/tests/certified_native_dense.rs"));
    assert_eq!(
        gpt2_sha256, expected_gpt2_sha256,
        "gpt2.rs differs from the maintainer-frozen whole-dense candidate"
    );
    assert_eq!(
        harness_sha256, expected_harness_sha256,
        "certified_native_dense.rs differs from the maintainer-frozen whole-dense harness"
    );
    eprintln!(
        "CERTIFIED_WHOLE_DENSE_SOURCE_HASHES gpt2_sha256={gpt2_sha256} harness_sha256={harness_sha256}"
    );
    let source = real_source();
    for required in ["model.safetensors", "config.json", "source_manifest.json"] {
        assert!(
            source.join(required).is_file(),
            "hard-bound real GPT-2 fixture is missing {} (set UOR_GPT2_SOURCE or install the repo-relative fixture)",
            source.join(required).display()
        );
    }
    let model = Gpt2::load(&source, Some(WHOLE_STEPS)).expect("load real pinned GPT-2");
    bind_dense_identity(
        &source,
        &model,
        DenseGateContract {
            arithmetic_id: WHOLE_DENSE_ARITHMETIC_ID,
            bootstrap_id: WHOLE_BOOTSTRAP_ALGORITHM_ID,
            pairs: WHOLE_REAL_PAIRS,
            warmups_per_arm: WHOLE_WARMUP_SUITES_PER_ARM,
            threshold: WHOLE_MAXIMUM_UPPER_RATIO,
            threshold_rule: "less-than-or-equal",
        },
    );
    assert_eq!(model.cfg.n_embd, 768);
    assert_eq!(model.cfg.n_head, 12);
    assert_eq!(model.cfg.n_layer, 12);
    assert_eq!(model.cfg.n_inner, 3072);
    assert_eq!(model.cfg.vocab, 50257);
    assert_eq!(model.cfg.seq_len, WHOLE_STEPS);

    let preparation_started = Instant::now();
    let mut workspace = model
        .dense_control_workspace()
        .expect("prepare model-bound whole-dense workspace");
    let preparation_elapsed = preparation_started.elapsed();
    let workspace_bytes = model
        .dense_control_workspace_bytes(&workspace)
        .expect("whole-dense workspace byte accounting");
    assert_eq!(workspace_bytes.lm_head_transpose(), 154_389_504);
    assert_eq!(workspace_bytes.matrix_bounds(), 1_065_608);
    assert_eq!(workspace_bytes.f64_sum_scratch(), 1_467_672);
    assert_eq!(workspace_bytes.intermediate_scratch(), 268_656);
    assert_eq!(
        workspace_bytes.total(),
        workspace_bytes.lm_head_transpose()
            + workspace_bytes.matrix_bounds()
            + workspace_bytes.f64_sum_scratch()
            + workspace_bytes.intermediate_scratch()
            + workspace_bytes.metadata()
    );
    eprintln!(
        "CERTIFIED_WHOLE_DENSE_WORKSPACE transpose_bytes={} matrix_bound_bytes={} f64_sum_scratch_bytes={} intermediate_scratch_bytes={} metadata_bytes={} total_caller_owned_bytes={} max_batch=1 model_bound=true preparation_ns={} preparation_in_benchmark=false",
        workspace_bytes.lm_head_transpose(),
        workspace_bytes.matrix_bounds(),
        workspace_bytes.f64_sum_scratch(),
        workspace_bytes.intermediate_scratch(),
        workspace_bytes.metadata(),
        workspace_bytes.total(),
        preparation_elapsed.as_nanos(),
    );

    // Bind the timing control to both the current production executor and the
    // independent numpy golden before any candidate gate. Nothing here is in
    // a measured region, and the checked-in golden is never regenerated.
    let golden_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/gpt2-real/golden.json");
    assert!(
        golden_path.is_file(),
        "hard-bound independent GPT-2 golden is missing at {}",
        golden_path.display()
    );
    let golden: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&golden_path).expect("read independent GPT-2 golden"),
    )
    .expect("parse independent GPT-2 golden");
    let golden_cases = golden["cases"].as_array().expect("golden cases array");
    assert_eq!(golden_cases.len(), WHOLE_STORIES.len());
    let mut production_state = Gpt2State::new(&model.cfg);
    let mut conventional_state = Gpt2State::new(&model.cfg);
    for (story_index, story) in WHOLE_STORIES.iter().enumerate() {
        let golden_case = &golden_cases[story_index];
        let golden_tokens: Vec<usize> = golden_case["tokens"]
            .as_array()
            .expect("golden tokens")
            .iter()
            .map(|value| value.as_u64().expect("golden token integer") as usize)
            .collect();
        assert_eq!(&golden_tokens, story, "golden story {story_index}");
        production_state.reset();
        conventional_state.reset();
        for (position, &token) in story.iter().enumerate() {
            model.forward(&mut production_state, token, position, &[], &mut |_, _| {});
            let census = model
                .forward_dense_control(
                    &mut conventional_state,
                    &mut workspace,
                    token,
                    position,
                    Gpt2DenseCanaryMode::Conventional,
                )
                .expect("real production-baseline control step");
            assert_whole_census(census, &model, 1, 0, Gpt2DenseCanaryMode::Conventional);
            assert_state_bits(
                &conventional_state,
                &production_state,
                &format!("real production/conventional story {story_index} position {position}"),
            );
        }

        let golden_hidden: Vec<f32> = golden_case["hidden"]
            .as_array()
            .expect("golden hidden")
            .iter()
            .map(|value| value.as_f64().expect("golden hidden float") as f32)
            .collect();
        assert_eq!(golden_hidden.len(), model.cfg.n_embd);
        let worst_hidden = production_state
            .hidden
            .iter()
            .zip(&golden_hidden)
            .map(|(&actual, &expected)| (actual - expected).abs())
            .fold(0.0f32, f32::max);
        assert!(
            worst_hidden < 5e-2,
            "independent golden hidden delta {worst_hidden:e} for story {story_index}"
        );

        let mut order: Vec<usize> = (0..model.cfg.vocab).collect();
        order.sort_by(|&left, &right| {
            production_state.logits[right]
                .total_cmp(&production_state.logits[left])
                .then_with(|| left.cmp(&right))
        });
        assert_eq!(
            order[0],
            golden_case["argmax"]
                .as_u64()
                .expect("golden argmax integer") as usize,
            "independent golden argmax story {story_index}"
        );
        let golden_top_five: std::collections::BTreeSet<usize> = golden_case["top_k"]
            .as_array()
            .expect("golden top-k")
            .iter()
            .take(5)
            .map(|entry| {
                entry.as_array().expect("golden top-k pair")[0]
                    .as_u64()
                    .expect("golden top-k token") as usize
            })
            .collect();
        let actual_top_five: std::collections::BTreeSet<usize> =
            order.iter().copied().take(5).collect();
        assert_eq!(
            actual_top_five, golden_top_five,
            "independent golden top-5 story {story_index}"
        );
    }
    eprintln!(
        "CERTIFIED_WHOLE_DENSE_BASELINE production_conventional_full_state_bits=PASS independent_golden_hidden_argmax_top5=PASS stories=3"
    );

    // Binding structural gate: candidate only, before the expensive exact
    // owner. Any fallback or incomplete owner census stops the run here.
    let mut structural_state = Gpt2State::new(&model.cfg);
    let mut structural_total = DenseCensusTotal::default();
    for story in WHOLE_STORIES {
        structural_state.reset();
        for (position, &token) in story.iter().enumerate() {
            structural_state
                .logits
                .fill(f32::from_bits(OUTPUT_POISON_BITS));
            structural_state
                .hidden
                .fill(f32::from_bits(OUTPUT_POISON_BITS));
            let census = model
                .forward_dense_control(
                    &mut structural_state,
                    &mut workspace,
                    token,
                    position,
                    Gpt2DenseCanaryMode::CertifiedNative,
                )
                .expect("hard-bound structural candidate step");
            let observed =
                assert_whole_census(census, &model, 1, 0, Gpt2DenseCanaryMode::CertifiedNative);
            merge_dense_total(&mut structural_total, observed);
            assert_output_overwritten(&structural_state.logits, "structural logits");
            assert_output_overwritten(&structural_state.hidden, "structural hidden");
        }
    }
    assert_real_candidate_structure(structural_total, "structural gate");
    eprintln!(
        "CERTIFIED_WHOLE_DENSE_STRUCTURAL verdict=PROCEED census={structural_total:?} expected_calls={WHOLE_OWNER_CALLS} expected_lanes={WHOLE_LANES}"
    );

    // Exact preflight stores the complete expected recurrent state after each
    // deterministic token. Candidate and exact share the prepared workspace
    // sequentially and are compared after every step.
    let mut exact_state = Gpt2State::new(&model.cfg);
    let mut candidate_state = Gpt2State::new(&model.cfg);
    let mut expected_steps = Vec::with_capacity(WHOLE_STEPS);
    let mut preflight_total = DenseCensusTotal::default();
    for story in WHOLE_STORIES {
        exact_state.reset();
        candidate_state.reset();
        for (position, &token) in story.iter().enumerate() {
            exact_state.logits.fill(f32::from_bits(OUTPUT_POISON_BITS));
            exact_state.hidden.fill(f32::from_bits(OUTPUT_POISON_BITS));
            let exact = model
                .forward_dense_control(
                    &mut exact_state,
                    &mut workspace,
                    token,
                    position,
                    Gpt2DenseCanaryMode::Exact,
                )
                .expect("hard-bound exact preflight step");
            assert_whole_census(exact, &model, 1, 0, Gpt2DenseCanaryMode::Exact);
            assert_output_overwritten(&exact_state.logits, "exact preflight logits");
            assert_output_overwritten(&exact_state.hidden, "exact preflight hidden");
            expected_steps.push(exact_state.clone());

            candidate_state
                .logits
                .fill(f32::from_bits(OUTPUT_POISON_BITS));
            candidate_state
                .hidden
                .fill(f32::from_bits(OUTPUT_POISON_BITS));
            let candidate = model
                .forward_dense_control(
                    &mut candidate_state,
                    &mut workspace,
                    token,
                    position,
                    Gpt2DenseCanaryMode::CertifiedNative,
                )
                .expect("hard-bound candidate preflight step");
            let observed = assert_whole_census(
                candidate,
                &model,
                1,
                0,
                Gpt2DenseCanaryMode::CertifiedNative,
            );
            merge_dense_total(&mut preflight_total, observed);
            assert_state_bits(
                &candidate_state,
                &exact_state,
                &format!("whole exact preflight token {}", expected_steps.len() - 1),
            );
        }
    }
    assert_eq!(expected_steps.len(), WHOLE_STEPS);
    assert_real_candidate_structure(preflight_total, "exact preflight");

    // Fixed predeclared warmup: two complete three-story suites per arm. The
    // order is candidate then conventional for each warmup index.
    let mut warmup_state = Gpt2State::new(&model.cfg);
    for warmup in 0..WHOLE_WARMUP_SUITES_PER_ARM {
        let mut expected_index = 0usize;
        let mut candidate_total = DenseCensusTotal::default();
        for story in WHOLE_STORIES {
            warmup_state.reset();
            for (position, &token) in story.iter().enumerate() {
                let census = model
                    .forward_dense_control(
                        &mut warmup_state,
                        &mut workspace,
                        token,
                        position,
                        Gpt2DenseCanaryMode::CertifiedNative,
                    )
                    .expect("candidate warmup step");
                let observed =
                    assert_whole_census(census, &model, 1, 0, Gpt2DenseCanaryMode::CertifiedNative);
                merge_dense_total(&mut candidate_total, observed);
                assert_state_bits(
                    &warmup_state,
                    &expected_steps[expected_index],
                    &format!("candidate warmup {warmup} step {expected_index}"),
                );
                expected_index += 1;
            }
        }
        assert_real_candidate_structure(candidate_total, "candidate warmup");
        for story in WHOLE_STORIES {
            warmup_state.reset();
            for (position, &token) in story.iter().enumerate() {
                let census = model
                    .forward_dense_control(
                        &mut warmup_state,
                        &mut workspace,
                        token,
                        position,
                        Gpt2DenseCanaryMode::Conventional,
                    )
                    .expect("conventional warmup step");
                assert_whole_census(census, &model, 1, 0, Gpt2DenseCanaryMode::Conventional);
            }
        }
    }
    eprintln!(
        "CERTIFIED_WHOLE_DENSE_WARMUP suites_per_arm={WHOLE_WARMUP_SUITES_PER_ARM} stories_per_suite=3 tokens_per_suite={WHOLE_STEPS} order=candidate-then-conventional"
    );

    let mut candidate_seconds = [0.0f64; WHOLE_REAL_PAIRS];
    let mut conventional_seconds = [0.0f64; WHOLE_REAL_PAIRS];
    let mut timed_total = DenseCensusTotal::default();
    let mut timed_state = Gpt2State::new(&model.cfg);
    for pair in 0..WHOLE_REAL_PAIRS {
        let candidate_first = pair.is_multiple_of(2);
        let (candidate_elapsed, candidate_census, conventional_elapsed);
        if candidate_first {
            (candidate_elapsed, candidate_census) = timed_whole_suite(
                &model,
                &mut timed_state,
                &mut workspace,
                Gpt2DenseCanaryMode::CertifiedNative,
                &expected_steps,
            );
            (conventional_elapsed, _) = timed_whole_suite(
                &model,
                &mut timed_state,
                &mut workspace,
                Gpt2DenseCanaryMode::Conventional,
                &expected_steps,
            );
        } else {
            (conventional_elapsed, _) = timed_whole_suite(
                &model,
                &mut timed_state,
                &mut workspace,
                Gpt2DenseCanaryMode::Conventional,
                &expected_steps,
            );
            (candidate_elapsed, candidate_census) = timed_whole_suite(
                &model,
                &mut timed_state,
                &mut workspace,
                Gpt2DenseCanaryMode::CertifiedNative,
                &expected_steps,
            );
        }
        assert_real_candidate_structure(candidate_census, "timed candidate suite");
        merge_dense_total(&mut timed_total, candidate_census);
        candidate_seconds[pair] = candidate_elapsed.as_secs_f64();
        conventional_seconds[pair] = conventional_elapsed.as_secs_f64();
        eprintln!(
            "CERTIFIED_WHOLE_DENSE_SAMPLE pair={pair} first={} candidate_total_ns={} conventional_total_ns={} candidate_ns_per_token={:.0} conventional_ns_per_token={:.0} paired_ratio={:.9} candidate_census={candidate_census:?}",
            if candidate_first {
                "candidate"
            } else {
                "conventional"
            },
            candidate_elapsed.as_nanos(),
            conventional_elapsed.as_nanos(),
            candidate_elapsed.as_nanos() as f64 / WHOLE_STEPS as f64,
            conventional_elapsed.as_nanos() as f64 / WHOLE_STEPS as f64,
            candidate_seconds[pair] / conventional_seconds[pair],
        );
    }
    assert_eq!(timed_total.calls, WHOLE_REAL_PAIRS * WHOLE_OWNER_CALLS);
    assert_eq!(timed_total.lanes, WHOLE_REAL_PAIRS * WHOLE_LANES);
    assert!(timed_total.fast > 0);
    assert!(timed_total.refined > 0);
    assert_eq!(timed_total.fallbacks(), 0);

    let paired_ratios =
        std::array::from_fn(|pair| candidate_seconds[pair] / conventional_seconds[pair]);
    let paired_median_ratio = median_nine(paired_ratios);
    let candidate_median = median_nine(candidate_seconds);
    let conventional_median = median_nine(conventional_seconds);
    let ratio_of_medians = candidate_median / conventional_median;
    let bootstrap_median_upper_95 = empirical_bootstrap_median_upper_95_nine(paired_ratios);
    eprintln!(
        "CERTIFIED_WHOLE_DENSE_RESULT arithmetic_id={WHOLE_DENSE_ARITHMETIC_ID} bootstrap_algorithm_id={WHOLE_BOOTSTRAP_ALGORITHM_ID} pairs={WHOLE_REAL_PAIRS} warmup_suites_per_arm={WHOLE_WARMUP_SUITES_PER_ARM} stories=3 tokens_per_suite={WHOLE_STEPS} candidate_seconds={candidate_seconds:?} conventional_seconds={conventional_seconds:?} paired_ratios={paired_ratios:?} candidate_median_ns_per_token={:.0} conventional_median_ns_per_token={:.0} paired_median_ratio={paired_median_ratio:.9} ratio_of_medians={ratio_of_medians:.9} bootstrap_median_upper_95={bootstrap_median_upper_95:.9} timed_census={timed_total:?} preflight_census={preflight_total:?} structural_census={structural_total:?} workspace_bytes={} threshold={WHOLE_MAXIMUM_UPPER_RATIO:.1} threshold_rule=less-than-or-equal",
        candidate_median * 1e9 / WHOLE_STEPS as f64,
        conventional_median * 1e9 / WHOLE_STEPS as f64,
        workspace_bytes.total(),
    );
    assert!(
        bootstrap_median_upper_95 <= WHOLE_MAXIMUM_UPPER_RATIO,
        "whole GPT-2 one-sided 95% bootstrap median upper {bootstrap_median_upper_95:.9} exceeds {WHOLE_MAXIMUM_UPPER_RATIO:.1}"
    );
}
