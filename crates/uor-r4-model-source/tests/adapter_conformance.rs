//! #599 adapter-conformance gates: positive-or-rejection coverage for every
//! configuration feature the Llama adapter assumes, plus the deterministic
//! fixture runner's three-state PASS / FAIL / UNAVAILABLE behavior.
//!
//! Everything here is synthetic (in-test snapshot directories) except the
//! two real-fixture arms at the bottom: the non-ignored one asserts the
//! absent pinned SmolLM2 snapshot is reported as UNAVAILABLE evidence
//! (never silently skipped), and the `#[ignore]`d one runs the full parity
//! round trip when the 257 MiB snapshot is present.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use uor_r4_model_source::conformance::{
    canonical_fixture_json, canonical_report_json, parse_fixture_json, record_fixture, run_fixture,
    run_fixture_file, AdapterFeature, AdapterFeatures, AdapterFixture, CaseSpec,
    ChatTemplatePolicy, ConformanceStatus, FixtureIdentity, FixtureSpec, FixtureTolerances,
    PromptToken, ADAPTER_FIXTURE_SCHEMA,
};
use uor_r4_model_source::{HuggingFaceLlamaOracle, SourceIngestKind};

// ---------------------------------------------------------------------------
// Synthetic snapshot helpers (same construction as the in-crate #598 tests).
// ---------------------------------------------------------------------------

fn temp_snapshot_dir(tag: &str) -> PathBuf {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "uor-r4-i599-{tag}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).expect("create synthetic snapshot dir");
    dir
}

fn write(dir: &Path, name: &str, bytes: &[u8]) {
    std::fs::write(dir.join(name), bytes).expect("write synthetic snapshot file");
}

/// Build a well-formed single-file Safetensors shard: contiguous offsets in
/// entry order.
fn shard_bytes(tensors: &[(&str, &str, &[usize], &[u8])]) -> Vec<u8> {
    let mut entries = Vec::new();
    let mut data = Vec::new();
    let mut offset = 0usize;
    for (name, dtype, shape, bytes) in tensors {
        let end = offset + bytes.len();
        let dims = shape
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",");
        entries.push(format!(
            "\"{name}\":{{\"dtype\":\"{dtype}\",\"shape\":[{dims}],\
             \"data_offsets\":[{offset},{end}]}}"
        ));
        data.extend_from_slice(bytes);
        offset = end;
    }
    let header = format!("{{{}}}", entries.join(","));
    let mut out = (header.len() as u64).to_le_bytes().to_vec();
    out.extend_from_slice(header.as_bytes());
    out.extend_from_slice(&data);
    out
}

/// Deterministic finite BF16 payload bytes for `n` elements.
fn bf16_data(n: usize, salt: u16) -> Vec<u8> {
    (0..n)
        .flat_map(|i| {
            (0x3F00u16 | ((i as u16).wrapping_mul(31).wrapping_add(salt) & 0x7F)).to_le_bytes()
        })
        .collect()
}

/// The default tiny geometry: dim 8, hidden 16, 1 layer, 2 heads, 2 kv
/// heads, vocab 10, seq 8.
fn base_config_fields() -> Vec<(&'static str, String)> {
    vec![
        ("hidden_size", "8".to_owned()),
        ("intermediate_size", "16".to_owned()),
        ("num_hidden_layers", "1".to_owned()),
        ("num_attention_heads", "2".to_owned()),
        ("num_key_value_heads", "2".to_owned()),
        ("vocab_size", "10".to_owned()),
        ("max_position_embeddings", "8".to_owned()),
        ("tie_word_embeddings", "true".to_owned()),
    ]
}

/// Render a config.json from the base fields with `overrides` applied
/// (replacing an existing field or appending a new one; values are raw
/// JSON).
fn config_json(overrides: &[(&str, &str)]) -> String {
    let mut fields = base_config_fields();
    for &(name, value) in overrides {
        if let Some(existing) = fields.iter_mut().find(|(field, _)| *field == name) {
            existing.1 = value.to_owned();
        } else {
            fields.push((
                Box::leak(name.to_owned().into_boxed_str()),
                value.to_owned(),
            ));
        }
    }
    let body = fields
        .iter()
        .map(|(name, value)| format!("\"{name}\":{value}"))
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{body}}}")
}

/// The tiny snapshot's tensors for the tied-embedding geometry above.
fn tiny_tensors(untied_lm_head: bool) -> Vec<(String, Vec<usize>, Vec<u8>)> {
    let mut tensors = Vec::new();
    let mut push = |name: &str, shape: &[usize], salt: u16| {
        let elements = shape.iter().product::<usize>();
        tensors.push((name.to_owned(), shape.to_vec(), bf16_data(elements, salt)));
    };
    push("model.embed_tokens.weight", &[10, 8], 20);
    push("model.layers.0.input_layernorm.weight", &[8], 21);
    push("model.layers.0.self_attn.q_proj.weight", &[8, 8], 22);
    push("model.layers.0.self_attn.k_proj.weight", &[8, 8], 23);
    push("model.layers.0.self_attn.v_proj.weight", &[8, 8], 24);
    push("model.layers.0.self_attn.o_proj.weight", &[8, 8], 25);
    push("model.layers.0.post_attention_layernorm.weight", &[8], 26);
    push("model.layers.0.mlp.gate_proj.weight", &[16, 8], 27);
    push("model.layers.0.mlp.down_proj.weight", &[8, 16], 28);
    push("model.layers.0.mlp.up_proj.weight", &[16, 8], 29);
    push("model.norm.weight", &[8], 30);
    if untied_lm_head {
        push("lm_head.weight", &[10, 8], 31);
    }
    tensors
}

/// Write a complete synthetic snapshot: config.json plus a single-file
/// weights artifact matching the tied/untied choice in `config`.
fn write_snapshot(tag: &str, config: &str, untied_lm_head: bool) -> PathBuf {
    let dir = temp_snapshot_dir(tag);
    write(&dir, "config.json", config.as_bytes());
    let tensors = tiny_tensors(untied_lm_head);
    let entries: Vec<(&str, &str, &[usize], &[u8])> = tensors
        .iter()
        .map(|(name, shape, data)| (name.as_str(), "BF16", shape.as_slice(), data.as_slice()))
        .collect();
    write(&dir, "model.safetensors", &shard_bytes(&entries));
    dir
}

/// A rejection-path snapshot: config.json only, deliberately WITHOUT any
/// weights file, so a passing load is impossible and a feature rejection
/// provably happens before any tensor ingestion or observation.
fn write_config_only(tag: &str, config: &str) -> PathBuf {
    let dir = temp_snapshot_dir(tag);
    write(&dir, "config.json", config.as_bytes());
    dir
}

/// Assert a config-only snapshot is rejected with the focused feature kind
/// BEFORE observation (there are no weights to observe with).
fn assert_rejected(tag: &str, overrides: &[(&str, &str)], feature: AdapterFeature) {
    let dir = write_config_only(tag, &config_json(overrides));
    let error = HuggingFaceLlamaOracle::load(&dir)
        .err()
        .unwrap_or_else(|| panic!("{tag}: config outside the declaration must be rejected"));
    match error.kind {
        SourceIngestKind::UnsupportedConfigFeature {
            feature: rejected, ..
        } => assert_eq!(rejected, feature, "{tag}: wrong focused feature"),
        other => panic!("{tag}: expected UnsupportedConfigFeature, got {other:?}"),
    }
}

fn default_spec() -> FixtureSpec {
    FixtureSpec {
        cases: vec![
            CaseSpec {
                name: "bos-then-token".to_owned(),
                prompt: vec![
                    PromptToken {
                        id: 1,
                        bytes: "<s>".to_owned(),
                    },
                    PromptToken {
                        id: 3,
                        bytes: "tok3".to_owned(),
                    },
                ],
            },
            CaseSpec {
                name: "single-token".to_owned(),
                prompt: vec![PromptToken {
                    id: 7,
                    bytes: "tok7".to_owned(),
                }],
            },
        ],
        capture_layers: vec![0],
        top_k: 3,
        tolerances: FixtureTolerances::default(),
    }
}

// ---------------------------------------------------------------------------
// (d) Positive coverage: config features the adapter declares and executes.
// ---------------------------------------------------------------------------

#[test]
fn declared_baseline_config_records_and_passes_fixture() {
    let dir = write_snapshot(
        "baseline",
        &config_json(&[
            ("model_type", "\"llama\""),
            ("hidden_act", "\"silu\""),
            ("rms_norm_eps", "1e-05"),
            ("rope_theta", "10000.0"),
        ]),
        false,
    );
    let fixture = record_fixture(&dir, &default_spec()).expect("record baseline fixture");
    assert_eq!(fixture.schema, ADAPTER_FIXTURE_SCHEMA);
    assert!(fixture.identity.source_kappa.starts_with("blake3:"));
    assert_eq!(fixture.identity.adapter_name, "huggingface-llama");
    assert_eq!(fixture.identity.tokenizer, "huggingface-tokenizer");
    assert_eq!(fixture.identity.source_manifest_kappa, None);
    assert_eq!(fixture.cases.len(), 2);
    assert!(fixture.cases[0].per_layer.contains_key(&0));

    let report = run_fixture(&fixture, &dir);
    assert_eq!(report.status, ConformanceStatus::Pass, "{report:?}");
    assert!(report
        .checks
        .iter()
        .all(|check| check.status == ConformanceStatus::Pass));
}

#[test]
fn untied_embeddings_are_declared_and_load() {
    let dir = write_snapshot(
        "untied",
        &config_json(&[("tie_word_embeddings", "false")]),
        true,
    );
    let fixture = record_fixture(&dir, &default_spec()).expect("record untied fixture");
    let report = run_fixture(&fixture, &dir);
    assert_eq!(report.status, ConformanceStatus::Pass, "{report:?}");
}

#[test]
fn gqa_and_mqa_head_geometry_is_declared_and_loads() {
    // MQA: 2 query heads sharing 1 kv head. The k/v projections shrink to
    // kv_dim = 4.
    let dir = temp_snapshot_dir("mqa");
    write(
        &dir,
        "config.json",
        config_json(&[("num_key_value_heads", "1")]).as_bytes(),
    );
    let mut tensors = tiny_tensors(false);
    for (name, shape, data) in &mut tensors {
        if name.contains("k_proj") || name.contains("v_proj") {
            *shape = vec![4, 8];
            *data = bf16_data(32, 40);
        }
    }
    let entries: Vec<(&str, &str, &[usize], &[u8])> = tensors
        .iter()
        .map(|(name, shape, data)| (name.as_str(), "BF16", shape.as_slice(), data.as_slice()))
        .collect();
    write(&dir, "model.safetensors", &shard_bytes(&entries));
    let fixture = record_fixture(&dir, &default_spec()).expect("record MQA fixture");
    let report = run_fixture(&fixture, &dir);
    assert_eq!(report.status, ConformanceStatus::Pass, "{report:?}");
}

#[test]
fn interleaved_rope_mode_is_declared_and_loads() {
    let dir = write_snapshot(
        "interleaved",
        &config_json(&[("rope_interleaved", "true")]),
        false,
    );
    let fixture = record_fixture(&dir, &default_spec()).expect("record interleaved fixture");
    let report = run_fixture(&fixture, &dir);
    assert_eq!(report.status, ConformanceStatus::Pass, "{report:?}");
}

#[test]
fn declared_rope_theta_range_and_scalar_token_ids_load() {
    let dir = write_snapshot(
        "theta",
        &config_json(&[
            ("rope_theta", "100000.0"),
            ("bos_token_id", "1"),
            ("eos_token_id", "2"),
        ]),
        false,
    );
    HuggingFaceLlamaOracle::load(&dir).expect("declared theta and token ids load");
}

#[test]
fn chat_template_is_tolerated_and_never_interpreted() {
    // The declared policy: the source executor consumes raw token ids; a
    // chat template in the snapshot is carried, not executed.
    assert_eq!(
        AdapterFeatures::huggingface_llama().chat_template,
        ChatTemplatePolicy::NotInterpreted
    );
    let dir = write_snapshot(
        "chat-template",
        &config_json(&[(
            "chat_template",
            "\"{% for m in messages %}{{ m }}{% endfor %}\"",
        )]),
        false,
    );
    HuggingFaceLlamaOracle::load(&dir).expect("chat template presence must not reject");
}

// ---------------------------------------------------------------------------
// (d) Rejection coverage: each undeclared feature fails closed with its
// focused kind, before any tensor is read (the snapshots carry no weights).
// ---------------------------------------------------------------------------

#[test]
fn unexpected_activation_is_rejected() {
    assert_rejected(
        "gelu",
        &[("hidden_act", "\"gelu\"")],
        AdapterFeature::Activation,
    );
}

#[test]
fn undeclared_norm_epsilon_is_rejected() {
    // The executor's rmsnorm epsilon is fixed at the declared 1e-5; a
    // config asking for 1e-6 would previously load and silently run at
    // 1e-5.
    assert_rejected(
        "eps",
        &[("rms_norm_eps", "1e-06")],
        AdapterFeature::NormEpsilon,
    );
}

#[test]
fn undeclared_attention_bias_is_rejected() {
    assert_rejected(
        "attn-bias",
        &[("attention_bias", "true")],
        AdapterFeature::AttentionBias,
    );
}

#[test]
fn undeclared_mlp_bias_is_rejected() {
    assert_rejected("mlp-bias", &[("mlp_bias", "true")], AdapterFeature::MlpBias);
}

#[test]
fn scaled_or_unknown_rope_mode_is_rejected() {
    assert_rejected(
        "rope-scaling",
        &[("rope_scaling", "{\"rope_type\":\"yarn\",\"factor\":2.0}")],
        AdapterFeature::RopeMode,
    );
    assert_rejected(
        "rope-mode",
        &[("rope_interleaved", "\"diagonal\"")],
        AdapterFeature::RopeMode,
    );
}

#[test]
fn rope_theta_outside_declared_range_is_rejected() {
    assert_rejected(
        "theta-negative",
        &[("rope_theta", "-1.0")],
        AdapterFeature::RopeTheta,
    );
}

#[test]
fn gqa_geometry_mismatch_is_rejected() {
    // 3 query heads cannot share 2 kv heads.
    assert_rejected(
        "gqa-mismatch",
        &[
            ("hidden_size", "12"),
            ("num_attention_heads", "3"),
            ("num_key_value_heads", "2"),
        ],
        AdapterFeature::HeadGeometry,
    );
}

#[test]
fn odd_head_size_is_rejected() {
    // hidden 10 over 2 heads gives head size 5; RoPE rotates half-pairs.
    assert_rejected(
        "odd-head",
        &[("hidden_size", "10")],
        AdapterFeature::HeadGeometry,
    );
}

#[test]
fn undeclared_model_type_is_rejected() {
    assert_rejected(
        "model-type",
        &[("model_type", "\"qwen2\"")],
        AdapterFeature::ModelType,
    );
}

#[test]
fn token_id_outside_vocabulary_is_rejected() {
    assert_rejected(
        "bos-range",
        &[("bos_token_id", "99")],
        AdapterFeature::TokenPolicy,
    );
}

#[test]
fn list_valued_token_policy_is_rejected() {
    assert_rejected(
        "eos-list",
        &[("eos_token_id", "[0,2]")],
        AdapterFeature::TokenPolicy,
    );
}

#[test]
fn rejection_happens_before_snapshot_ingestion() {
    // Same undeclared feature, but with valid weights present: the focused
    // feature rejection must still win over every ingestion check, proving
    // the gate runs before the #598 boundary and before any observation.
    let dir = write_snapshot(
        "gate-order",
        &config_json(&[("hidden_act", "\"gelu\"")]),
        false,
    );
    let error = HuggingFaceLlamaOracle::load(&dir)
        .err()
        .expect("undeclared activation must be rejected despite valid weights");
    assert!(matches!(
        error.kind,
        SourceIngestKind::UnsupportedConfigFeature {
            feature: AdapterFeature::Activation,
            ..
        }
    ));
}

// ---------------------------------------------------------------------------
// (c) Runner determinism and the three-state result.
// ---------------------------------------------------------------------------

#[test]
fn runner_fails_with_numeric_deltas_on_perturbed_fixture() {
    let dir = write_snapshot("perturbed", &config_json(&[]), false);
    let mut fixture = record_fixture(&dir, &default_spec()).expect("record fixture");
    fixture.cases[0].logits[0] += 1.0;
    let report = run_fixture(&fixture, &dir);
    assert_eq!(report.status, ConformanceStatus::Fail);
    let logits_check = report
        .checks
        .iter()
        .find(|check| check.name == "case/bos-then-token/logits")
        .expect("logits check present");
    assert_eq!(logits_check.status, ConformanceStatus::Fail);
    let delta = logits_check.delta.expect("numeric delta reported");
    assert!(delta > 0.9 && delta < 1.1, "delta {delta}");
    assert_eq!(logits_check.tolerance, Some(fixture.tolerances.logit_abs));
}

#[test]
fn runner_reports_are_byte_identical_across_runs() {
    let dir = write_snapshot("determinism", &config_json(&[]), false);
    let fixture = record_fixture(&dir, &default_spec()).expect("record fixture");
    let first = canonical_report_json(&run_fixture(&fixture, &dir));
    let second = canonical_report_json(&run_fixture(&fixture, &dir));
    assert_eq!(first, second, "reports must be byte-identical");
    assert!(first.contains("\"schema\":\"uor-r4-adapter-conformance-report/1\""));
}

#[test]
fn fixture_canonical_json_round_trips() {
    let dir = write_snapshot("roundtrip", &config_json(&[]), false);
    let fixture = record_fixture(&dir, &default_spec()).expect("record fixture");
    let first = canonical_fixture_json(&fixture);
    let reparsed = parse_fixture_json(&first).expect("parse canonical fixture");
    assert_eq!(reparsed, fixture);
    assert_eq!(canonical_fixture_json(&reparsed), first);
}

#[test]
fn missing_fixture_file_is_unavailable_evidence() {
    let dir = write_snapshot("no-fixture", &config_json(&[]), false);
    let absent = dir.join("no-such.fixture.json");
    let report = run_fixture_file(&absent, &dir);
    assert_eq!(report.status, ConformanceStatus::Unavailable);
    let named = report.unavailable.expect("prerequisite named");
    assert!(named.contains("no-such.fixture.json"), "{named}");
    assert!(report.checks.is_empty());
}

#[test]
fn fixture_file_round_trip_through_disk_passes() {
    let dir = write_snapshot("fixture-file", &config_json(&[]), false);
    let fixture = record_fixture(&dir, &default_spec()).expect("record fixture");
    let path = dir.join("adapter.fixture.json");
    std::fs::write(&path, canonical_fixture_json(&fixture)).expect("write fixture file");
    let report = run_fixture_file(&path, &dir);
    assert_eq!(report.status, ConformanceStatus::Pass, "{report:?}");
}

#[test]
fn absent_snapshot_is_unavailable_evidence() {
    let dir = write_snapshot("donor", &config_json(&[]), false);
    let fixture = record_fixture(&dir, &default_spec()).expect("record fixture");
    let missing = temp_snapshot_dir("gone").join("never-downloaded");
    let report = run_fixture(&fixture, &missing);
    assert_eq!(report.status, ConformanceStatus::Unavailable);
    let named = report.unavailable.expect("prerequisite named");
    assert!(named.contains("never-downloaded"), "{named}");
}

#[test]
fn unsupported_fixture_schema_is_unavailable_evidence() {
    let dir = write_snapshot("schema", &config_json(&[]), false);
    let mut fixture = record_fixture(&dir, &default_spec()).expect("record fixture");
    fixture.schema = "uor-r4-adapter-fixture/999".to_owned();
    let report = run_fixture(&fixture, &dir);
    assert_eq!(report.status, ConformanceStatus::Unavailable);
    assert!(report
        .unavailable
        .expect("prerequisite named")
        .contains("uor-r4-adapter-fixture/999"));
}

#[test]
fn source_kappa_mismatch_is_a_failing_identity_check() {
    let dir = write_snapshot("identity", &config_json(&[]), false);
    let mut fixture = record_fixture(&dir, &default_spec()).expect("record fixture");
    fixture.identity.source_kappa = format!("blake3:{}", "0".repeat(64));
    let report = run_fixture(&fixture, &dir);
    assert_eq!(report.status, ConformanceStatus::Fail);
    let check = report
        .checks
        .iter()
        .find(|check| check.name == "identity/source-kappa")
        .expect("identity check present");
    assert_eq!(check.status, ConformanceStatus::Fail);
}

// ---------------------------------------------------------------------------
// (e) The real-fixture arm: pinned SmolLM2.
// ---------------------------------------------------------------------------

fn smollm2_source_dir() -> PathBuf {
    PathBuf::from(
        std::env::var("SMOLLM2_SOURCE")
            .unwrap_or_else(|_| ".uor-models/sources/smollm2-135m-instruct".to_owned()),
    )
}

fn smollm2_spec() -> FixtureSpec {
    FixtureSpec {
        cases: vec![CaseSpec {
            name: "bos-prompt".to_owned(),
            prompt: vec![
                PromptToken {
                    id: 1,
                    bytes: "<|im_start|>".to_owned(),
                },
                PromptToken {
                    id: 100,
                    bytes: "token-100".to_owned(),
                },
            ],
        }],
        capture_layers: vec![0],
        top_k: 5,
        tolerances: FixtureTolerances::default(),
    }
}

/// The pinned 257 MiB SmolLM2 snapshot is not in this environment. That is
/// not a silent skip: the runner must return UNAVAILABLE naming the missing
/// snapshot. On a machine where the snapshot IS present, the same call must
/// not report UNAVAILABLE — both branches assert.
#[test]
fn absent_pinned_smollm2_snapshot_reports_unavailable() {
    let dir = smollm2_source_dir();
    let fixture = AdapterFixture {
        schema: ADAPTER_FIXTURE_SCHEMA.to_owned(),
        identity: FixtureIdentity {
            source_manifest_kappa: None,
            source_kappa: format!("blake3:{}", "0".repeat(64)),
            adapter_name: "huggingface-llama".to_owned(),
            adapter_version: env!("CARGO_PKG_VERSION").to_owned(),
            compiler_version: env!("CARGO_PKG_VERSION").to_owned(),
            tokenizer: "huggingface-tokenizer".to_owned(),
        },
        capture_layers: vec![],
        tolerances: FixtureTolerances::default(),
        cases: vec![],
    };
    let report = run_fixture(&fixture, &dir);
    if dir.join("config.json").is_file() {
        assert_ne!(
            report.status,
            ConformanceStatus::Unavailable,
            "snapshot is present; the runner must run it"
        );
    } else {
        assert_eq!(report.status, ConformanceStatus::Unavailable);
        let named = report.unavailable.expect("prerequisite named");
        assert!(
            named.contains(&dir.display().to_string()),
            "UNAVAILABLE must name the missing snapshot: {named}"
        );
    }
}

/// Full real-fixture parity round trip, fixture-gated like the other
/// pinned-snapshot tests in this repository.
#[test]
#[ignore = "requires the downloaded 257 MiB SmolLM2 source"]
fn real_smollm2_fixture_round_trip_passes() {
    let dir = smollm2_source_dir();
    let fixture = record_fixture(&dir, &smollm2_spec()).expect("record SmolLM2 fixture");
    if dir.join("source_manifest.json").is_file() {
        assert!(
            fixture.identity.source_manifest_kappa.is_some(),
            "a snapshot with a #597 manifest must be bound by the fixture identity"
        );
    }
    let report = run_fixture(&fixture, &dir);
    assert_eq!(report.status, ConformanceStatus::Pass, "{report:?}");
    let bytes = canonical_report_json(&report);
    assert_eq!(bytes, canonical_report_json(&run_fixture(&fixture, &dir)));
}
