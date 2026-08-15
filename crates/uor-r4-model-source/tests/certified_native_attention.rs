use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use serde::Deserialize;
use uor_r4_model_source::attention::{AttentionOperatorSpec, CERTIFIED_NATIVE_ARITHMETIC_ID};
use uor_r4_model_source::gpt2::{
    Gpt2, Gpt2AttentionCanaryCensus, Gpt2AttentionCanaryMode, Gpt2AttentionCanaryWorkspace,
    Gpt2State,
};

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
// cells are observational and do not participate in allocation.
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

fn assert_bits(got: &[f32], expected: &[f32], context: &str) {
    assert_eq!(got.len(), expected.len(), "{context}: length");
    for (lane, (&got, &expected)) in got.iter().zip(expected).enumerate() {
        assert_eq!(
            got.to_bits(),
            expected.to_bits(),
            "{context}: lane {lane}: got={got:?} expected={expected:?}"
        );
    }
}

const GPT2_SOURCE_ENV: &str = "UOR_GPT2_SOURCE";
const GPT2_REPOSITORY: &str = "openai-community/gpt2";
const GPT2_REVISION: &str = "607a30d783dfa663caf39e06633721c8d4cfcd7e";
const GPT2_MODEL_KAPPA: &str =
    "blake3:3bca1b7f6c327daecafc16e52d1319375299354e35413fb4e18d24e59b77ce06";
const GPT2_CONFIG_KAPPA: &str =
    "blake3:23e4471d412e06128072b559c031207de920b8a56d7108879d4b487c079a310c";
const GPT2_MODEL_BYTES: usize = 548_105_171;
const GPT2_CONFIG_BYTES: usize = 665;
const GPT2_STEPS: usize = 32;
const GPT2_PAIRS: usize = 5;
const MAXIMUM_RATIO: f64 = 1.10;
const UOR_MATMUL_REV: &str = "b13c98449948174f590e337c4dc25dfc394a07d0";
const STANDARD_V2_DIGEST: &str =
    "blake3:fa4c8f233e217d3903678b7690de5cdfb27d83a4b68c52436cfabbc6ca6cfc59";
const EXPERIMENTAL_V2_DIGEST: &str =
    "blake3:a71d3a9fbfd951528652b837a23d5bfd7742ba79d5082337c19db3a17776e654";
const LEARNED_ABSOLUTE_V2_DIGEST: &str =
    "blake3:ba36fd1fef53a2e3744e1fee60e72677870d1cd2f2b484db755c0a5a74727231";
const PINNED_RUSTC_RELEASE: &str = "1.97.1";
const PINNED_RUSTC_HOST: &str = "aarch64-apple-darwin";

fn real_gpt2_source() -> PathBuf {
    std::env::var_os(GPT2_SOURCE_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(".uor-models/sources/gpt2-124m")
        })
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

fn bind_production_attention_identity() -> String {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let model_source_manifest =
        std::fs::read_to_string(workspace.join("crates/uor-r4-model-source/Cargo.toml"))
            .expect("read model-source Cargo.toml");
    assert!(
        model_source_manifest.contains(&format!("rev = \"{UOR_MATMUL_REV}\"")),
        "model-source Cargo.toml does not pin the declared uor-matmul rev"
    );
    let lockfile = std::fs::read_to_string(workspace.join("Cargo.lock")).expect("read Cargo.lock");
    assert!(
        lockfile.contains(&format!("?rev={UOR_MATMUL_REV}#{UOR_MATMUL_REV}")),
        "Cargo.lock does not resolve the declared uor-matmul rev"
    );

    let records = [
        (AttentionOperatorSpec::standard_v2(), STANDARD_V2_DIGEST),
        (
            AttentionOperatorSpec::experimental_r4_v2(),
            EXPERIMENTAL_V2_DIGEST,
        ),
        (
            AttentionOperatorSpec::learned_absolute_v2(),
            LEARNED_ABSOLUTE_V2_DIGEST,
        ),
    ];
    for (record, expected_digest) in &records {
        assert_eq!(record.version, 2);
        assert_eq!(
            record.params.score_accumulation,
            CERTIFIED_NATIVE_ARITHMETIC_ID
        );
        assert_eq!(record.implementation_digest, *expected_digest);
        assert_eq!(record.declared_digest(), *expected_digest);
    }
    assert_eq!(AttentionOperatorSpec::standard(), records[0].0);
    assert_eq!(AttentionOperatorSpec::experimental_r4(), records[1].0);
    assert_eq!(
        AttentionOperatorSpec::learned_absolute_source_attention(),
        records[2].0
    );
    eprintln!(
        "CERTIFIED_ATTENTION_PRODUCTION arithmetic_id={CERTIFIED_NATIVE_ARITHMETIC_ID} standard_v2_digest={STANDARD_V2_DIGEST} experimental_v2_digest={EXPERIMENTAL_V2_DIGEST} learned_absolute_v2_digest={LEARNED_ABSOLUTE_V2_DIGEST} uor_matmul_rev={UOR_MATMUL_REV}"
    );
    LEARNED_ABSOLUTE_V2_DIGEST.to_owned()
}

#[test]
fn production_attention_identity_is_hard_bound() {
    let _ = bind_production_attention_identity();
}

#[test]
fn checked_canary_facade_matches_production_and_exact_without_allocating() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/gpt2-tiny");
    let model = Gpt2::load(&fixture, None).expect("load tiny GPT-2 fixture");
    let mut workspace = model
        .attention_canary_workspace()
        .expect("valid tiny model admits canary workspace");
    let mut production = Gpt2State::new(&model.cfg);
    let mut certified = Gpt2State::new(&model.cfg);
    let mut exact = Gpt2State::new(&model.cfg);
    let mut conventional = Gpt2State::new(&model.cfg);
    let mut census = Gpt2AttentionCanaryCensus::default();

    for (position, token) in [1usize, 3, 2].into_iter().enumerate() {
        model.forward(&mut production, token, position, &[], &mut |_, _| {});
        census
            .merge(
                model
                    .forward_attention_canary(
                        &mut certified,
                        &mut workspace,
                        token,
                        position,
                        Gpt2AttentionCanaryMode::CertifiedNative,
                    )
                    .expect("checked certified step"),
            )
            .expect("tiny certified census fits usize");
        model
            .forward_attention_canary(
                &mut exact,
                &mut workspace,
                token,
                position,
                Gpt2AttentionCanaryMode::Exact,
            )
            .expect("checked exact step");
        model
            .forward_attention_canary(
                &mut conventional,
                &mut workspace,
                token,
                position,
                Gpt2AttentionCanaryMode::Conventional,
            )
            .expect("checked conventional step");
        assert_bits(
            &certified.logits,
            &production.logits,
            "certified/production",
        );
        assert_bits(&certified.logits, &exact.logits, "certified/exact");
    }
    assert!(census.qk().lanes() > 0);
    assert!(census.value().lanes() > 0);
    assert!(census.qk().certified() > 0);
    assert!(census.value().certified() > 0);
    assert!(production
        .logits
        .iter()
        .zip(&conventional.logits)
        .any(|(current, historical)| current.to_bits() != historical.to_bits()));

    let mut allocation_state = Gpt2State::new(&model.cfg);
    let (_, allocations) = counted_allocations(|| {
        model
            .forward_attention_canary(
                &mut allocation_state,
                &mut workspace,
                1,
                0,
                Gpt2AttentionCanaryMode::CertifiedNative,
            )
            .expect("checked allocation census step")
    });
    assert_eq!(allocations, 0, "checked canary hot step allocated");
}

fn bind_real_snapshot(path: &Path, model: &Gpt2) {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let canonical = path.canonicalize().expect("canonicalize GPT-2 source");
    assert_eq!(model.attention_control_source_kappa(), GPT2_MODEL_KAPPA);
    assert_eq!(model.attention_control_source_bytes(), GPT2_MODEL_BYTES);

    let config = std::fs::read(path.join("config.json")).expect("read GPT-2 config");
    assert_eq!(config.len(), GPT2_CONFIG_BYTES);
    let config_kappa = format!("blake3:{}", blake3::hash(&config).to_hex());
    assert_eq!(config_kappa, GPT2_CONFIG_KAPPA);

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

    let toolchain = std::fs::read_to_string(workspace.join("rust-toolchain.toml"))
        .expect("read pinned rust toolchain");
    assert!(toolchain
        .lines()
        .any(|line| line.trim() == format!("channel = \"{PINNED_RUSTC_RELEASE}\"")));
    let forbidden_build_overrides: Vec<(String, String)> = std::env::vars()
        .filter(|(key, value)| {
            !value.is_empty()
                && (key.contains("RUSTFLAGS") || key.starts_with("CARGO_PROFILE_RELEASE_"))
        })
        .collect();
    assert!(
        forbidden_build_overrides.is_empty(),
        "release canary forbids external build overrides: {forbidden_build_overrides:?}"
    );
    let rustc = Command::new("rustc")
        .current_dir(&workspace)
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
    eprintln!(
        "CERTIFIED_ATTENTION_IDENTITY repository={GPT2_REPOSITORY} revision={GPT2_REVISION} model_kappa={GPT2_MODEL_KAPPA} model_bytes={GPT2_MODEL_BYTES} config_kappa={GPT2_CONFIG_KAPPA} config_bytes={GPT2_CONFIG_BYTES} source_path={} arch={} os={} rustc_release={PINNED_RUSTC_RELEASE} rustc_host={PINNED_RUSTC_HOST} build_overrides=none steps={GPT2_STEPS} pairs={GPT2_PAIRS} threshold={MAXIMUM_RATIO:.2}",
        canonical.display(),
        std::env::consts::ARCH,
        std::env::consts::OS,
    );
    for line in rustc_identity.lines() {
        eprintln!("CERTIFIED_ATTENTION_RUSTC {line}");
    }
}

fn deterministic_tokens(vocab: usize) -> [usize; GPT2_STEPS] {
    assert!(vocab > 1);
    std::array::from_fn(|position| 1 + position.wrapping_mul(7_919).wrapping_add(17) % (vocab - 1))
}

fn run_control_story(
    model: &Gpt2,
    state: &mut Gpt2State,
    workspace: &mut Gpt2AttentionCanaryWorkspace,
    tokens: &[usize; GPT2_STEPS],
    mode: Gpt2AttentionCanaryMode,
) -> Gpt2AttentionCanaryCensus {
    let mut census = Gpt2AttentionCanaryCensus::default();
    for (position, &token) in tokens.iter().enumerate() {
        census
            .merge(
                model
                    .forward_attention_canary(state, workspace, token, position, mode)
                    .expect("hard-bound canary inputs are valid"),
            )
            .expect("32-token canary census fits usize");
    }
    census
}

fn timed_control_story(
    model: &Gpt2,
    state: &mut Gpt2State,
    workspace: &mut Gpt2AttentionCanaryWorkspace,
    tokens: &[usize; GPT2_STEPS],
    mode: Gpt2AttentionCanaryMode,
) -> (Duration, Gpt2AttentionCanaryCensus) {
    state.reset();
    let ((elapsed, census), allocations) = counted_allocations(|| {
        let started = Instant::now();
        let census = run_control_story(model, state, workspace, tokens, mode);
        (started.elapsed(), census)
    });
    assert_eq!(allocations, 0, "the timed checked canary story allocated");
    std::hint::black_box(&state.logits);
    (elapsed, census)
}

fn assert_nonvacuous_candidate_census(context: &str, census: Gpt2AttentionCanaryCensus) {
    for (kind, dots) in [("QK", census.qk()), ("value", census.value())] {
        assert!(dots.lanes() > 0, "{context} {kind}: no lanes executed");
        assert!(
            dots.certified() > 0,
            "{context} {kind}: all-fallback candidate is ineligible"
        );
        let fallbacks = dots.fallbacks().expect("fallback census fits usize");
        assert_eq!(
            dots.certified()
                .checked_add(fallbacks)
                .expect("candidate verdict partition fits usize"),
            dots.lanes(),
            "{context} {kind}: every lane must have one verdict"
        );
    }
}

fn median_five(mut values: [f64; GPT2_PAIRS]) -> f64 {
    values.sort_by(f64::total_cmp);
    values[GPT2_PAIRS / 2]
}

/// Hard gate only: one real GPT-2 32-token story, five alternating pairs.
/// Exact is the checked preflight oracle; only conventional and certified are
/// timed. The `<=1.10x` threshold is the posted #704 Choice-A consumer
/// contract; this local certified-native path does not claim compliance with
/// upstream #38's distinct arithmetic policy.
#[test]
#[ignore = "release-only, hard-bound real GPT-2 certified-attention canary"]
#[allow(clippy::assertions_on_constants)]
fn real_gpt2_certified_attention_32_token_gate() {
    assert!(!cfg!(debug_assertions), "run the canary with --release");
    let learned_absolute_v2_digest = bind_production_attention_identity();
    let source = real_gpt2_source();
    let model = Gpt2::load(&source, Some(GPT2_STEPS)).expect("load hard-bound real GPT-2");
    bind_real_snapshot(&source, &model);
    assert_eq!(model.cfg.n_embd, 768);
    assert_eq!(model.cfg.n_head, 12);
    assert_eq!(model.cfg.n_layer, 12);
    assert_eq!(model.cfg.head_size(), 64);
    let tokens = deterministic_tokens(model.cfg.vocab);
    let mut workspace = model
        .attention_canary_workspace()
        .expect("hard-bound model admits checked canary workspace");

    let mut production_state = Gpt2State::new(&model.cfg);
    let mut production_control_state = Gpt2State::new(&model.cfg);
    let mut production_control_census = Gpt2AttentionCanaryCensus::default();
    for (position, &token) in tokens.iter().enumerate() {
        model.forward(&mut production_state, token, position, &[], &mut |_, _| {});
        production_control_census
            .merge(
                model
                    .forward_attention_canary(
                        &mut production_control_state,
                        &mut workspace,
                        token,
                        position,
                        Gpt2AttentionCanaryMode::CertifiedNative,
                    )
                    .expect("production-control canary inputs"),
            )
            .expect("production-control census fits usize");
        assert_bits(
            &production_control_state.logits,
            &production_state.logits,
            &format!("real production/default v2 position {position}"),
        );
    }
    assert_nonvacuous_candidate_census("production-control", production_control_census);

    let mut conventional_state = Gpt2State::new(&model.cfg);
    let mut exact_state = Gpt2State::new(&model.cfg);
    let mut candidate_state = Gpt2State::new(&model.cfg);
    let mut preflight = Gpt2AttentionCanaryCensus::default();
    for (position, &token) in tokens.iter().enumerate() {
        model
            .forward_attention_canary(
                &mut exact_state,
                &mut workspace,
                token,
                position,
                Gpt2AttentionCanaryMode::Exact,
            )
            .expect("exact preflight inputs");
        preflight
            .merge(
                model
                    .forward_attention_canary(
                        &mut candidate_state,
                        &mut workspace,
                        token,
                        position,
                        Gpt2AttentionCanaryMode::CertifiedNative,
                    )
                    .expect("candidate preflight inputs"),
            )
            .expect("preflight census fits usize");
        assert_bits(
            &candidate_state.logits,
            &exact_state.logits,
            &format!("real candidate/exact position {position}"),
        );
    }
    assert_nonvacuous_candidate_census("preflight", preflight);
    let exact_final = exact_state.logits.clone();

    let _ = timed_control_story(
        &model,
        &mut conventional_state,
        &mut workspace,
        &tokens,
        Gpt2AttentionCanaryMode::Conventional,
    );
    let _ = timed_control_story(
        &model,
        &mut candidate_state,
        &mut workspace,
        &tokens,
        Gpt2AttentionCanaryMode::CertifiedNative,
    );

    let mut conventional_seconds = [0.0f64; GPT2_PAIRS];
    let mut candidate_seconds = [0.0f64; GPT2_PAIRS];
    let mut timed_census = Gpt2AttentionCanaryCensus::default();
    for pair in 0..GPT2_PAIRS {
        let candidate_first = pair % 2 == 0;
        let (candidate_elapsed, conventional_elapsed) = if candidate_first {
            let (candidate, census) = timed_control_story(
                &model,
                &mut candidate_state,
                &mut workspace,
                &tokens,
                Gpt2AttentionCanaryMode::CertifiedNative,
            );
            timed_census
                .merge(census)
                .expect("timed candidate census fits usize");
            assert_bits(
                &candidate_state.logits,
                &exact_final,
                &format!("timed candidate pair {pair}"),
            );
            let (conventional, _) = timed_control_story(
                &model,
                &mut conventional_state,
                &mut workspace,
                &tokens,
                Gpt2AttentionCanaryMode::Conventional,
            );
            (candidate, conventional)
        } else {
            let (conventional, _) = timed_control_story(
                &model,
                &mut conventional_state,
                &mut workspace,
                &tokens,
                Gpt2AttentionCanaryMode::Conventional,
            );
            let (candidate, census) = timed_control_story(
                &model,
                &mut candidate_state,
                &mut workspace,
                &tokens,
                Gpt2AttentionCanaryMode::CertifiedNative,
            );
            timed_census
                .merge(census)
                .expect("timed candidate census fits usize");
            assert_bits(
                &candidate_state.logits,
                &exact_final,
                &format!("timed candidate pair {pair}"),
            );
            (candidate, conventional)
        };
        candidate_seconds[pair] = candidate_elapsed.as_secs_f64();
        conventional_seconds[pair] = conventional_elapsed.as_secs_f64();
        eprintln!(
            "CERTIFIED_ATTENTION_SAMPLE pair={pair} first={} candidate_ns={} conventional_ns={} ratio={:.9}",
            if candidate_first {
                "candidate"
            } else {
                "conventional"
            },
            candidate_elapsed.as_nanos(),
            conventional_elapsed.as_nanos(),
            candidate_seconds[pair] / conventional_seconds[pair],
        );
    }
    assert_nonvacuous_candidate_census("timed", timed_census);

    let ratios = std::array::from_fn(|pair| candidate_seconds[pair] / conventional_seconds[pair]);
    let median_ratio = median_five(ratios);
    let candidate_median = median_five(candidate_seconds);
    let conventional_median = median_five(conventional_seconds);
    let qk_rate = timed_census.qk().certified() as f64 / timed_census.qk().lanes() as f64;
    let value_rate = timed_census.value().certified() as f64 / timed_census.value().lanes() as f64;
    eprintln!(
        "CERTIFIED_ATTENTION_RESULT arithmetic_id={CERTIFIED_NATIVE_ARITHMETIC_ID} learned_absolute_v2_digest={learned_absolute_v2_digest} uor_matmul_rev={UOR_MATMUL_REV} steps={GPT2_STEPS} pairs={GPT2_PAIRS} candidate_median_ns={:.0} conventional_median_ns={:.0} paired_median_ratio={median_ratio:.9} raw_ratios={ratios:?} qk_certification_rate={qk_rate:.9} value_certification_rate={value_rate:.9} timed_census={timed_census:?} preflight_census={preflight:?} production_control_census={production_control_census:?} threshold={MAXIMUM_RATIO:.2} threshold_rule=less-than-or-equal",
        candidate_median * 1e9,
        conventional_median * 1e9,
    );
    assert!(
        median_ratio <= MAXIMUM_RATIO,
        "certified attention misses the local gate: {median_ratio:.6}x > {MAXIMUM_RATIO:.2}x"
    );
}
