//! Focused validator for #934's teacher-free canonical-quality diagnostic.
//!
//! The non-ignored tests validate the committed schema, exact integer
//! arithmetic, scope separation, evidence links, and bounded remediation
//! order. The ignored fixture test only re-hashes and reads the already-built
//! #833 `smollm2-360m-broad-clean` bundle. It does not score Gate C, run a
//! teacher forward, rebuild observations, or rerun #908.

use std::io::Read;
use std::path::{Path, PathBuf};

use serde_json::Value;
use uor_r4_graph_format::{GraphView, SectionId};

const COMMITTED_DIAGNOSTIC: &str =
    include_str!("../../../docs/canonical_quality_baseline_934_result.json");
const SCHEMA: &str = "uor-r4-canonical-quality-diagnostic/1";
const POPULATION: u64 = 72_130;

fn repo_root() -> PathBuf {
    if let Some(manifest_dir) = std::env::var_os("CARGO_MANIFEST_DIR") {
        for ancestor in Path::new(&manifest_dir).ancestors() {
            if ancestor.join("model/ledger.toml").is_file() {
                return ancestor.to_path_buf();
            }
        }
    }
    let current_dir = std::env::current_dir().expect("current directory is readable");
    for ancestor in current_dir.ancestors() {
        if ancestor.join("model/ledger.toml").is_file() {
            return ancestor.to_path_buf();
        }
    }
    panic!(
        "repository root not found: no ancestor of CARGO_MANIFEST_DIR or the current directory contains model/ledger.toml"
    );
}

fn value_at<'a>(value: &'a Value, pointer: &str) -> Result<&'a Value, String> {
    value
        .pointer(pointer)
        .ok_or_else(|| format!("missing required field {pointer}"))
}

fn str_at<'a>(value: &'a Value, pointer: &str) -> Result<&'a str, String> {
    value_at(value, pointer)?
        .as_str()
        .ok_or_else(|| format!("{pointer} is not a string"))
}

fn u64_at(value: &Value, pointer: &str) -> Result<u64, String> {
    value_at(value, pointer)?
        .as_u64()
        .ok_or_else(|| format!("{pointer} is not an unsigned integer"))
}

fn i64_at(value: &Value, pointer: &str) -> Result<i64, String> {
    value_at(value, pointer)?
        .as_i64()
        .ok_or_else(|| format!("{pointer} is not an integer"))
}

fn f64_at(value: &Value, pointer: &str) -> Result<f64, String> {
    value_at(value, pointer)?
        .as_f64()
        .ok_or_else(|| format!("{pointer} is not numeric"))
}

fn bool_at(value: &Value, pointer: &str) -> Result<bool, String> {
    value_at(value, pointer)?
        .as_bool()
        .ok_or_else(|| format!("{pointer} is not Boolean"))
}

fn require(condition: bool, message: impl Into<String>) -> Result<(), String> {
    condition.then_some(()).ok_or_else(|| message.into())
}

fn close(actual: f64, expected: f64, tolerance: f64) -> bool {
    (actual - expected).abs() <= tolerance
}

fn is_blake3_cid(value: &str) -> bool {
    value.strip_prefix("blake3:").is_some_and(|hex| {
        hex.len() == 64
            && hex
                .chars()
                .all(|character| character.is_ascii_hexdigit() && !character.is_ascii_uppercase())
    })
}

fn validate_rate(
    diagnostic: &Value,
    base: &str,
    numerator_field: &str,
    denominator: u64,
    rate_field: &str,
) -> Result<(), String> {
    let numerator = u64_at(diagnostic, &format!("{base}/{numerator_field}"))?;
    let reported = f64_at(diagnostic, &format!("{base}/{rate_field}"))?;
    let expected = numerator as f64 * 100.0 / denominator as f64;
    require(
        close(reported, expected, 0.000_000_6),
        format!("{base}/{rate_field} is {reported}, expected {expected} from integer counts"),
    )
}

fn validate_cid_fields(diagnostic: &Value) -> Result<(), String> {
    const CID_POINTERS: &[&str] = &[
        "/input_bindings/source/cid",
        "/input_bindings/corpus/aggregate_cid",
        "/input_bindings/corpus/metadata/cid",
        "/input_bindings/corpus/records/cid",
        "/input_bindings/teacher_artifact/cid",
        "/input_bindings/tokenizer/cid",
        "/input_bindings/tokenizer/raw_definition_cid",
        "/input_bindings/tokenizer/adapter_digest",
        "/input_bindings/cover/cid",
        "/input_bindings/cover/reference_classifier_cid",
        "/input_bindings/graph/cid",
        "/input_bindings/graph/score_report_graph_kappa",
        "/input_bindings/score_report/cid",
        "/input_bindings/release_bundle/cid",
        "/reference_off_serving_908/base_artifact_cid",
        "/reference_off_serving_908/skip_artifact_cid",
        "/reference_off_serving_908/current_run_reproduction/result_cid",
    ];
    for pointer in CID_POINTERS {
        let cid = str_at(diagnostic, pointer)?;
        require(
            is_blake3_cid(cid),
            format!("{pointer} is not a canonical blake3 CID: {cid:?}"),
        )?;
    }
    Ok(())
}

fn validate_canonical_census(diagnostic: &Value) -> Result<(), String> {
    let population = u64_at(diagnostic, "/canonical_full_census/population")?;
    require(population == POPULATION, "canonical population drifted")?;
    require(
        population == u64_at(diagnostic, "/subject/population")?,
        "subject and census populations differ",
    )?;

    let rule12 = u64_at(
        diagnostic,
        "/canonical_full_census/score_rows/rule12_precedence/correct",
    )?;
    let tla = u64_at(
        diagnostic,
        "/canonical_full_census/score_rows/tla3_baseline/correct",
    )?;
    require(rule12 == 17_595, "Rule 1+2 count drifted")?;
    require(tla == 20_284, "TLA count drifted")?;
    require(
        i64_at(
            diagnostic,
            "/canonical_full_census/score_rows/rule12_minus_tla/correct_delta",
        )? == rule12 as i64 - tla as i64,
        "Rule 1+2 minus TLA integer delta is inconsistent",
    )?;
    let expected_gap_permille = (rule12 as f64 - tla as f64) * 1000.0 / population as f64;
    require(
        close(
            f64_at(
                diagnostic,
                "/canonical_full_census/score_rows/rule12_minus_tla/delta_permille",
            )?,
            expected_gap_permille,
            0.000_05,
        ),
        "Rule 1+2 minus TLA permille delta is inconsistent",
    )?;
    validate_rate(
        diagnostic,
        "/canonical_full_census/score_rows/rule12_precedence",
        "correct",
        population,
        "rate_percent",
    )?;
    validate_rate(
        diagnostic,
        "/canonical_full_census/score_rows/tla3_baseline",
        "correct",
        population,
        "rate_percent",
    )?;

    let crosstab = "/canonical_full_census/rule12_vs_tla_crosstab";
    let both = u64_at(diagnostic, &format!("{crosstab}/both_correct"))?;
    let rule12_only = u64_at(diagnostic, &format!("{crosstab}/rule12_only"))?;
    let tla_only = u64_at(diagnostic, &format!("{crosstab}/tla_only"))?;
    let neither = u64_at(diagnostic, &format!("{crosstab}/neither"))?;
    require(
        [both, rule12_only, tla_only, neither].iter().sum::<u64>() == population,
        "paired crosstab does not exhaust the census",
    )?;
    require(
        both + rule12_only == rule12,
        "crosstab Rule 1+2 margin is wrong",
    )?;
    require(both + tla_only == tla, "crosstab TLA margin is wrong")?;

    let mut status_positions = 0u64;
    let mut status_correct = 0u64;
    let mut status_teacher_argmax = 0u64;
    let mut status_any_teacher_top3 = 0u64;
    for status in ["exact_context", "graph", "novel"] {
        let base = format!("/canonical_full_census/status_decomposition/{status}");
        let positions = u64_at(diagnostic, &format!("{base}/positions"))?;
        let correct = u64_at(diagnostic, &format!("{base}/correct"))?;
        let teacher_argmax = u64_at(diagnostic, &format!("{base}/teacher_argmax_present"))?;
        let any_teacher_top3 = u64_at(diagnostic, &format!("{base}/any_teacher_top3_present"))?;
        let ranked_not_selected = u64_at(
            diagnostic,
            &format!("{base}/teacher_argmax_present_not_selected"),
        )?;
        let buckets = value_at(diagnostic, &format!("{base}/teacher_rank_buckets"))?
            .as_array()
            .ok_or_else(|| format!("{base}/teacher_rank_buckets is not an array"))?;
        require(
            buckets.len() == 9,
            format!("{status} rank histogram has wrong shape"),
        )?;
        let bucket_sum = buckets.iter().try_fold(0u64, |sum, bucket| {
            bucket
                .as_u64()
                .map(|count| sum + count)
                .ok_or_else(|| format!("{status} rank bucket is not an unsigned integer"))
        })?;
        require(
            bucket_sum == teacher_argmax,
            format!("{status} rank histogram margin is wrong"),
        )?;
        require(
            teacher_argmax >= correct,
            format!("{status} has more selected-correct than teacher-argmax-present positions"),
        )?;
        require(
            ranked_not_selected == teacher_argmax - correct,
            format!("{status} teacher-argmax-present-not-selected arithmetic is wrong"),
        )?;
        require(
            any_teacher_top3 >= teacher_argmax,
            format!("{status} any-teacher-top3 presence is below teacher-argmax presence"),
        )?;
        validate_rate(diagnostic, &base, "correct", positions, "rate_percent")?;
        status_positions += positions;
        status_correct += correct;
        status_teacher_argmax += teacher_argmax;
        status_any_teacher_top3 += any_teacher_top3;
    }
    require(
        status_positions == population,
        "status positions do not exhaust the census",
    )?;
    require(
        status_correct == rule12,
        "status correct counts do not reproduce Rule 1+2",
    )?;

    let argmax_base = "/canonical_full_census/candidate_presence/teacher_argmax";
    let teacher_top3_base = "/canonical_full_census/candidate_presence/any_teacher_top3";
    require(
        u64_at(diagnostic, &format!("{argmax_base}/present"))? == status_teacher_argmax,
        "status teacher-argmax-presence counts do not reproduce the overall count",
    )?;
    require(
        u64_at(diagnostic, &format!("{teacher_top3_base}/present"))? == status_any_teacher_top3,
        "status any-teacher-top3-presence counts do not reproduce the overall count",
    )?;
    for base in [argmax_base, teacher_top3_base] {
        let present = u64_at(diagnostic, &format!("{base}/present"))?;
        let absent = u64_at(diagnostic, &format!("{base}/absent"))?;
        require(
            present + absent == population,
            format!("{base} does not exhaust the census"),
        )?;
        validate_rate(diagnostic, base, "present", population, "rate_percent")?;
    }
    require(
        u64_at(
            diagnostic,
            "/canonical_full_census/candidate_presence/teacher_argmax_present_but_not_selected",
        )? == status_teacher_argmax - rule12,
        "overall teacher-argmax-present-not-selected arithmetic is wrong",
    )?;

    let raw = value_at(
        diagnostic,
        "/canonical_full_census/probe_depth_histogram/raw",
    )?
    .as_array()
    .ok_or_else(|| "probe-depth histogram is not an array".to_string())?;
    require(raw.len() == 5, "probe-depth histogram has wrong shape")?;
    let raw_sum = raw.iter().try_fold(0u64, |sum, bucket| {
        bucket
            .as_u64()
            .map(|count| sum + count)
            .ok_or_else(|| "probe-depth bucket is not an unsigned integer".to_string())
    })?;
    require(
        raw_sum == population,
        "probe-depth histogram does not exhaust the census",
    )?;
    require(
        u64_at(
            diagnostic,
            "/canonical_full_census/probe_depth_histogram/full_depth_supported",
        )? == u64_at(
            diagnostic,
            "/canonical_full_census/status_decomposition/exact_context/positions",
        )?,
        "full-depth probe support does not match ExactContext count",
    )?;

    let alpha = "/canonical_full_census/residual_alpha_zero_counterfactual";
    let shipped = u64_at(diagnostic, &format!("{alpha}/shipped_alpha_one_correct"))?;
    let zero = u64_at(diagnostic, &format!("{alpha}/alpha_zero_correct"))?;
    let additional = u64_at(diagnostic, &format!("{alpha}/additional_correct"))?;
    require(shipped == 660, "graph shipped-alpha count drifted")?;
    require(
        zero - shipped == additional,
        "graph alpha-zero gain arithmetic is wrong",
    )?;
    require(
        close(
            f64_at(
                diagnostic,
                &format!("{alpha}/full_population_ceiling_permille"),
            )?,
            additional as f64 * 1000.0 / population as f64,
            0.000_05,
        ),
        "graph alpha-zero full-population ceiling is inconsistent",
    )?;
    require(
        !bool_at(diagnostic, &format!("{alpha}/closes_2689_position_deficit"))?
            && additional < tla - rule12,
        "graph alpha-zero counterfactual overclaims deficit closure",
    )?;
    Ok(())
}

fn validate_reference_908(diagnostic: &Value) -> Result<(), String> {
    let base = "/reference_off_serving_908";
    require(
        str_at(diagnostic, &format!("{base}/classification"))? == "reference/off-serving",
        "#908 must not be promoted to normative serving evidence",
    )?;
    require(
        !bool_at(diagnostic, &format!("{base}/canonical_release_evidence"))?,
        "#908 must not be labeled canonical-release evidence",
    )?;
    require(
        str_at(
            diagnostic,
            &format!("{base}/current_run_reproduction/status"),
        )? == "PASS"
            && str_at(
                diagnostic,
                &format!("{base}/current_run_reproduction/execution_scope"),
            )? == "reference/off-serving"
            && !bool_at(
                diagnostic,
                &format!("{base}/current_run_reproduction/canonical_release_evidence"),
            )?,
        "#908 reproduction status or scope is inconsistent",
    )?;
    let population = u64_at(diagnostic, &format!("{base}/population"))?;
    require(population == POPULATION, "#908 population drifted")?;
    for arm in [
        "base_sections_absent",
        "skip_sections_present",
        "label_shuffled_null",
    ] {
        validate_rate(
            diagnostic,
            &format!("{base}/arms/{arm}"),
            "correct",
            population,
            "rate_percent",
        )?;
    }
    let base_correct = u64_at(
        diagnostic,
        &format!("{base}/arms/base_sections_absent/correct"),
    )?;
    let skip_correct = u64_at(
        diagnostic,
        &format!("{base}/arms/skip_sections_present/correct"),
    )?;
    let null_correct = u64_at(
        diagnostic,
        &format!("{base}/arms/label_shuffled_null/correct"),
    )?;
    let paired = format!("{base}/paired_skip_minus_base");
    require(
        i64_at(diagnostic, &format!("{paired}/correct_delta"))?
            == skip_correct as i64 - base_correct as i64,
        "#908 paired integer delta is inconsistent",
    )?;
    require(
        close(
            f64_at(diagnostic, &format!("{paired}/delta_permille"))?,
            (skip_correct as f64 - base_correct as f64) * 1000.0 / population as f64,
            0.000_5,
        ),
        "#908 paired skip delta is inconsistent",
    )?;
    require(
        close(
            f64_at(
                diagnostic,
                &format!("{base}/paired_null_minus_base/delta_permille"),
            )?,
            (null_correct as f64 - base_correct as f64) * 1000.0 / population as f64,
            0.000_5,
        ),
        "#908 paired null delta is inconsistent",
    )?;
    let changed = u64_at(diagnostic, &format!("{base}/reachability/changed"))?;
    let toward = u64_at(diagnostic, &format!("{base}/reachability/toward"))?;
    let away = u64_at(diagnostic, &format!("{base}/reachability/away"))?;
    let neutral = u64_at(diagnostic, &format!("{base}/reachability/neutral_changed"))?;
    require(
        changed == toward + away + neutral,
        "#908 changed decomposition is inconsistent",
    )?;
    require(
        skip_correct as i64 - base_correct as i64 == toward as i64 - away as i64,
        "#908 toward-away decomposition does not reproduce the paired delta",
    )?;
    require(
        close(
            f64_at(diagnostic, &format!("{base}/reachability/ceiling_permille"))?,
            changed as f64 * 1000.0 / population as f64,
            0.000_5,
        ),
        "#908 reachability ceiling is inconsistent",
    )?;
    Ok(())
}

fn validate_remediation_order(diagnostic: &Value) -> Result<(), String> {
    let steps = value_at(diagnostic, "/bounded_remediation_order")?
        .as_array()
        .ok_or_else(|| "bounded_remediation_order is not an array".to_string())?;
    require(!steps.is_empty(), "remediation order is empty")?;
    for (index, step) in steps.iter().enumerate() {
        require(
            step.get("order").and_then(Value::as_u64) == Some(index as u64 + 1),
            format!("remediation step {} is out of order", index + 1),
        )?;
        let max_minutes = step
            .get("max_wall_time_minutes")
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("remediation step {} lacks a time bound", index + 1))?;
        require(
            (1..=60).contains(&max_minutes),
            format!("remediation step {} exceeds the one-hour bound", index + 1),
        )?;
        for field in [
            "action",
            "metric",
            "cheap_instrument",
            "proceed_if",
            "if_negative",
        ] {
            require(
                step.get(field)
                    .and_then(Value::as_str)
                    .is_some_and(|text| !text.trim().is_empty()),
                format!("remediation step {} lacks {field}", index + 1),
            )?;
        }
    }
    Ok(())
}

fn validate_diagnostic(diagnostic: &Value) -> Result<(), String> {
    require(
        str_at(diagnostic, "/schema")? == SCHEMA,
        "unsupported #934 schema",
    )?;
    require(u64_at(diagnostic, "/issue")? == 934, "wrong issue binding")?;
    require(
        str_at(diagnostic, "/status")? == "EMPIRICAL",
        "wrong evidence status",
    )?;
    require(
        str_at(diagnostic, "/execution_scope")? == "teacher-free-certifier-and-reachability-audit",
        "wrong execution scope",
    )?;
    for pointer in [
        "/execution/canonical_gate_c/status",
        "/execution/teacher_forward/status",
        "/execution/observation_or_corpus_rebuild/status",
    ] {
        require(
            str_at(diagnostic, pointer)? == "NOT_RUN",
            format!("{pointer} overstates work performed by #934"),
        )?;
    }
    require(
        !str_at(diagnostic, "/execution/canonical_gate_c/reason")?
            .trim()
            .is_empty(),
        "canonical Gate C NOT_RUN status lacks a reason",
    )?;
    require(
        str_at(diagnostic, "/subject/decode_mode")?
            == "teacher-forced-position greedy argmax agreement"
            && str_at(diagnostic, "/subject/argmax_label_source")? == "corpus-recorded t_argmax"
            && str_at(diagnostic, "/subject/bits_label_source")? == "corpus.next",
        "subject conflates argmax-agreement and bits/token label semantics",
    )?;
    require(
        !bool_at(
            diagnostic,
            "/threshold_semantics/universal_absolute_30_percent_requirement_exists",
        )?,
        "the diagnostic manufactured a universal/current broad absolute 30% requirement",
    )?;
    require(
        bool_at(
            diagnostic,
            "/threshold_semantics/historical_near_30_absolute_gate_exists",
        )?,
        "the diagnostic erased the real historical/current-code 29.7% pinned-profile gate",
    )?;
    require(
        close(
            f64_at(
                diagnostic,
                "/threshold_semantics/legacy_pinned_profile/effective_floor_percent",
            )?,
            29.7,
            f64::EPSILON,
        ),
        "legacy pinned floor drifted",
    )?;
    require(
        !bool_at(
            diagnostic,
            "/threshold_semantics/legacy_pinned_profile/applies_to_subject",
        )?,
        "legacy fixture floor was applied to the broad-clean subject",
    )?;
    require(
        str_at(diagnostic, "/threshold_semantics/canonical_profile/profile")? == "relative_tla",
        "canonical profile is not relative_tla",
    )?;
    require(
        !bool_at(
            diagnostic,
            "/threshold_semantics/canonical_profile/observed_requirement_met",
        )?,
        "the below-TLA canonical row was labeled as meeting relative_tla",
    )?;
    require(
        str_at(diagnostic, "/reachability/artifact/canonical_graph_skmx")? == "ABSENT"
            && str_at(diagnostic, "/reachability/artifact/canonical_graph_psib")? == "ABSENT"
            && !bool_at(diagnostic, "/reachability/artifact/active_skipmix_lane")?,
        "canonical artifact reachability contradicts its section census",
    )?;
    require(
        str_at(diagnostic, "/reachability/runtime/production_verdict")? == "NOT ESTABLISHED",
        "runtime reachability overclaims a production verdict",
    )?;
    validate_cid_fields(diagnostic)?;
    validate_canonical_census(diagnostic)?;
    validate_reference_908(diagnostic)?;
    validate_remediation_order(diagnostic)?;
    Ok(())
}

#[test]
fn committed_diagnostic_has_valid_schema_arithmetic_scope_and_links() {
    let diagnostic: Value =
        serde_json::from_str(COMMITTED_DIAGNOSTIC).expect("#934 diagnostic is valid JSON");
    validate_diagnostic(&diagnostic).expect("#934 diagnostic validates");

    let links = diagnostic["evidence_links"]
        .as_array()
        .expect("evidence_links is an array");
    assert!(!links.is_empty(), "#934 diagnostic has no evidence links");
    for link in links {
        let role = link["role"].as_str().expect("evidence link has a role");
        assert!(!role.trim().is_empty(), "evidence-link role is empty");
        let relative = link["repo_path"]
            .as_str()
            .expect("evidence link has a repository path");
        let path = Path::new(relative);
        assert!(
            path.is_relative(),
            "evidence link is not repository-relative: {relative}"
        );
        assert!(
            !relative.split('/').any(|component| component == ".."),
            "evidence link escapes the repository: {relative}"
        );
        assert!(
            repo_root().join(path).is_file(),
            "evidence link does not resolve to a file: {relative}"
        );
    }
}

#[test]
fn validator_rejects_metric_conflation_and_broken_margins() {
    let mut diagnostic: Value =
        serde_json::from_str(COMMITTED_DIAGNOSTIC).expect("#934 diagnostic is valid JSON");
    diagnostic["threshold_semantics"]["universal_absolute_30_percent_requirement_exists"] =
        Value::Bool(true);
    assert!(
        validate_diagnostic(&diagnostic).is_err(),
        "an invented absolute 30% requirement must fail closed"
    );

    let mut diagnostic: Value =
        serde_json::from_str(COMMITTED_DIAGNOSTIC).expect("#934 diagnostic is valid JSON");
    diagnostic["threshold_semantics"]["historical_near_30_absolute_gate_exists"] =
        Value::Bool(false);
    assert!(
        validate_diagnostic(&diagnostic).is_err(),
        "erasing the real 29.7% pinned-profile gate must fail closed"
    );

    let mut diagnostic: Value =
        serde_json::from_str(COMMITTED_DIAGNOSTIC).expect("#934 diagnostic is valid JSON");
    diagnostic["reference_off_serving_908"]["classification"] =
        Value::String("normative-runtime".to_owned());
    assert!(
        validate_diagnostic(&diagnostic).is_err(),
        "reference/off-serving #908 evidence must not be relabeled normative"
    );

    let mut diagnostic: Value =
        serde_json::from_str(COMMITTED_DIAGNOSTIC).expect("#934 diagnostic is valid JSON");
    diagnostic["execution"]["canonical_gate_c"]["status"] = Value::String("PASS".to_owned());
    assert!(
        validate_diagnostic(&diagnostic).is_err(),
        "an identity-verified report read must not be relabeled a fresh Gate C PASS"
    );

    let mut diagnostic: Value =
        serde_json::from_str(COMMITTED_DIAGNOSTIC).expect("#934 diagnostic is valid JSON");
    diagnostic["canonical_full_census"]["rule12_vs_tla_crosstab"]["tla_only"] =
        Value::from(3_014u64);
    assert!(
        validate_diagnostic(&diagnostic).is_err(),
        "a crosstab that no longer reproduces the row margins must fail"
    );
}

fn hash_file(path: &Path) -> Result<(String, u64), String> {
    let mut file =
        std::fs::File::open(path).map_err(|error| format!("open {}: {error}", path.display()))?;
    let bytes = file
        .metadata()
        .map_err(|error| format!("metadata {}: {error}", path.display()))?
        .len();
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0u8; 1024 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("read {}: {error}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok((format!("blake3:{}", hasher.finalize().to_hex()), bytes))
}

fn verify_bound_file(
    diagnostic: &Value,
    bundle: &Path,
    binding_pointer: &str,
) -> Result<(), String> {
    let relative = str_at(diagnostic, &format!("{binding_pointer}/path"))?;
    let expected_cid = str_at(diagnostic, &format!("{binding_pointer}/cid"))?;
    let expected_bytes = u64_at(diagnostic, &format!("{binding_pointer}/bytes"))?;
    let path = bundle.join(relative);
    let (actual_cid, actual_bytes) = hash_file(&path)?;
    require(
        actual_cid == expected_cid,
        format!(
            "{} CID mismatch: {actual_cid} != {expected_cid}",
            path.display()
        ),
    )?;
    require(
        actual_bytes == expected_bytes,
        format!(
            "{} length mismatch: {actual_bytes} != {expected_bytes}",
            path.display()
        ),
    )
}

fn derived_count(rate: f64, positions: u64) -> u64 {
    (rate * positions as f64).round() as u64
}

/// Presence-gated live binding check. A host without the ignored canonical
/// bundle fails with an explicit UNAVAILABLE verdict; a present but
/// byte-different bundle fails with the exact mismatch.
#[test]
#[ignore = "requires the local #833 smollm2-360m-broad-clean bundle; teacher-free"]
fn local_canonical_bundle_matches_every_binding_and_has_no_skipmix_sections() {
    let diagnostic: Value =
        serde_json::from_str(COMMITTED_DIAGNOSTIC).expect("#934 diagnostic is valid JSON");
    let bundle = std::env::var_os("R4_CANONICAL_QUALITY_BUNDLE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            repo_root()
                .join(".uor-models")
                .join("compiled")
                .join("smollm2-360m-broad-clean")
        });
    let required = [
        "corpus.meta",
        "corpus.records",
        "tless_artifacts.bin",
        "tokenizer.bin",
        "tokenizer_adapter.json",
        "recorded_corpus_binding.json",
        "graph-cover/cover.r4g1",
        "graph/score.r4g1",
        "graph/score_report.json",
        "release-bundle.json",
    ];
    let missing: Vec<&str> = required
        .iter()
        .copied()
        .filter(|relative| !bundle.join(relative).is_file())
        .collect();
    if !missing.is_empty() {
        panic!(
            "UNAVAILABLE: #934 canonical bundle fixture is absent/incomplete at {} (missing: {}); teacher-free live binding check cannot run",
            bundle.display(),
            missing.join(", ")
        );
    }

    for pointer in [
        "/input_bindings/corpus/metadata",
        "/input_bindings/corpus/records",
        "/input_bindings/teacher_artifact",
        "/input_bindings/tokenizer",
        "/input_bindings/cover",
        "/input_bindings/graph",
        "/input_bindings/score_report",
        "/input_bindings/release_bundle",
    ] {
        verify_bound_file(&diagnostic, &bundle, pointer).expect("bound bundle file matches");
    }

    let meta = std::fs::read(bundle.join("corpus.meta")).expect("read canonical corpus.meta");
    let records =
        std::fs::read(bundle.join("corpus.records")).expect("read canonical corpus.records");
    assert_eq!(
        uor_r4_graph_compiler::reproducibility::corpus_stream_kappa(&meta, &records),
        str_at(&diagnostic, "/input_bindings/corpus/aggregate_cid").expect("aggregate corpus CID")
    );

    let score_report_bytes =
        std::fs::read(bundle.join("graph/score_report.json")).expect("read score report");
    let report: Value = serde_json::from_slice(&score_report_bytes).expect("parse score report");
    assert_eq!(
        report["schema"],
        diagnostic["input_bindings"]["score_report"]["schema"]
    );
    assert_eq!(report["config"]["quality_profile"], "relative_tla");
    assert_eq!(
        report["inputs"]["artifact_kappa"],
        diagnostic["input_bindings"]["teacher_artifact"]["cid"]
    );
    assert_eq!(
        report["inputs"]["corpus_kappa"],
        diagnostic["input_bindings"]["corpus"]["aggregate_cid"]
    );
    assert_eq!(
        report["inputs"]["graph_kappa"],
        diagnostic["input_bindings"]["graph"]["score_report_graph_kappa"]
    );
    assert_eq!(report["gate_c"]["held_out_population"], POPULATION);

    let rule12 = &report["gate_c"]["rule12_precedence"];
    let tla = &report["gate_c"]["tla3_baseline"];
    assert_eq!(
        derived_count(
            rule12["top1_agreement"].as_f64().expect("Rule 1+2 rate"),
            POPULATION,
        ),
        17_595
    );
    assert_eq!(
        derived_count(
            tla["top1_agreement"].as_f64().expect("TLA rate"),
            POPULATION,
        ),
        20_284
    );
    assert_eq!(
        report["gate_c"]["win_loss"]["rule12_vs_baseline"],
        serde_json::json!({
            "both_correct": 17269,
            "scorer_only": 326,
            "other_only": 3015,
            "neither": 51520
        })
    );
    assert_eq!(
        report["gate_c"]["rule12_status_counts"],
        serde_json::json!({
            "exact_context": 52398,
            "graph": 19436,
            "novel": 296,
            "exact_context_ngram": 0,
            "exact_context_probe": 52398
        })
    );
    assert_eq!(
        report["gate_c"]["rule12_exct_probe_levels"],
        serde_json::json!([0, 1110, 9237, 9385, 52398])
    );
    assert_eq!(
        derived_count(
            report["gate_c"]["rule12_residual_alpha_sweep"]["graph"]["points"][3][1]
                .as_f64()
                .expect("graph alpha-zero rate"),
            19_436,
        ),
        860
    );

    for (
        status,
        expected_positions,
        expected_correct,
        expected_teacher_argmax_present,
        expected_any_teacher_top3_present,
    ) in [
        ("exact_context", 52_398, 16_933, 35_085, 46_283),
        ("graph", 19_436, 660, 9_962, 16_465),
        ("novel", 296, 2, 100, 215),
    ] {
        let metrics = &report["gate_c"]["rule12_per_status"][status];
        let recall = &report["gate_c"]["rule12_candidate_recall_per_status"][status];
        assert_eq!(metrics["positions"], expected_positions);
        assert_eq!(
            derived_count(
                metrics["top1_agreement"].as_f64().expect("status top-1"),
                expected_positions,
            ),
            expected_correct
        );
        assert_eq!(
            derived_count(
                recall["top1"]
                    .as_f64()
                    .expect("score-report teacher-argmax-presence rate"),
                expected_positions
            ),
            expected_teacher_argmax_present
        );
        assert_eq!(
            derived_count(
                recall["top3"]
                    .as_f64()
                    .expect("score-report any-teacher-top3-presence rate"),
                expected_positions
            ),
            expected_any_teacher_top3_present
        );
    }

    let release: Value = serde_json::from_slice(
        &std::fs::read(bundle.join("release-bundle.json")).expect("read release manifest"),
    )
    .expect("parse release manifest");
    assert_eq!(
        release["schema"],
        diagnostic["input_bindings"]["release_bundle"]["manifest_schema"]
    );
    assert_eq!(
        release["components"]["graph"],
        diagnostic["input_bindings"]["graph"]["cid"]
    );
    assert_eq!(
        release["components"]["signature_artifact"],
        diagnostic["input_bindings"]["teacher_artifact"]["cid"]
    );
    assert_eq!(
        release["components"]["tokenizer"],
        diagnostic["input_bindings"]["tokenizer"]["cid"]
    );
    assert_eq!(
        release["components"]["score_report"],
        diagnostic["input_bindings"]["score_report"]["cid"]
    );

    let tokenizer_adapter: Value = serde_json::from_slice(
        &std::fs::read(bundle.join("tokenizer_adapter.json")).expect("read tokenizer adapter"),
    )
    .expect("parse tokenizer adapter");
    assert_eq!(
        tokenizer_adapter["tokenizer_cid"],
        diagnostic["input_bindings"]["tokenizer"]["raw_definition_cid"]
    );
    assert_eq!(
        tokenizer_adapter["adapter_digest"],
        diagnostic["input_bindings"]["tokenizer"]["adapter_digest"]
    );

    let graph_bytes = std::fs::read(bundle.join("graph/score.r4g1")).expect("read graph");
    let graph = GraphView::parse(&graph_bytes).expect("canonical graph parses");
    graph
        .verify_cids()
        .expect("canonical graph internal CIDs verify");
    assert!(
        graph.section(SectionId::SKMX).is_none(),
        "canonical #833 graph unexpectedly carries SKMX"
    );
    assert!(
        graph.section(SectionId::PSIB).is_none(),
        "canonical #833 graph unexpectedly carries PSIB"
    );

    let source_path = std::env::var_os("R4_CANONICAL_QUALITY_SOURCE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let models_root = bundle
                .parent()
                .and_then(Path::parent)
                .expect("canonical bundle lives below an .uor-models/compiled directory");
            let recorded_path = Path::new(
                str_at(&diagnostic, "/input_bindings/source/path").expect("canonical source path"),
            );
            let relative = recorded_path
                .strip_prefix(".uor-models")
                .unwrap_or(recorded_path);
            models_root.join(relative)
        });
    assert!(
        source_path.is_file(),
        "UNAVAILABLE: #934 canonical source snapshot is absent at {}",
        source_path.display()
    );
    let (source_cid, source_bytes) =
        hash_file(&source_path).expect("hash canonical source snapshot");
    assert_eq!(source_cid, diagnostic["input_bindings"]["source"]["cid"]);
    assert_eq!(
        source_bytes,
        diagnostic["input_bindings"]["source"]["bytes"]
    );

    println!(
        "#934 teacher-free binding VERIFIED: {} (schema-26 census read; Gate C not rerun by this test; #908 reproduction is separately bound in the diagnostic; SKMX/PSIB absent)",
        bundle.display()
    );
}
