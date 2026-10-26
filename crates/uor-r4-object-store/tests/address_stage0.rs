#![cfg(feature = "stage0-lab")]
use serde_json::json;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::path::{Path, PathBuf};
use uor_r4_object_store::address_stage0::{
    build_manifest, clears_primary, collect_original_rows, compare_parity_rows, corpus_value_leaks,
    create_out_dir_after, d11_aggregate_rows, git_head, mark_parity_stop, normalize_emission,
    original_aggregate, padded_top8, parity_row_lines, parse_manifest, planted_wrong_holds,
    primary_pass, set_parity_aggregates, u16_le_tokens, validate_config_pins, validate_head,
    validate_head_against_git, validate_manifest, validate_manifest_cli_pins,
    values_missing_from_corpus, D11ParityRow, OriginalRow, PairedWins,
    CONFIG_PATH_NORMALIZATION_NOTE, MANIFEST_SHA256, ORIGINAL_CONFIG_SHA256, PARITY_ROW_SCHEMA,
    PROMPT_TOKEN_LIMIT, RETAINED_RECEIPT_SHA256, STAGE0_SCHEMA, STAGE_T_ARCHIVE_SHA256,
    TOKENIZER_SHA256,
};
use uor_r4_tokenizer::ByteBpeTokenizer;

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

const TOKENIZER_JSON: &str = r#"{
    "pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},
    "added_tokens":[{"id":8,"content":"<end>"}],
    "model":{"type":"BPE","vocab":{"a":0,"b":1,"c":2,"ab":3,"bc":4,"x":5,"y":6,"z":7},"merges":["b c","a b"]}
}"#;

fn tokenizer() -> TestResult<ByteBpeTokenizer> {
    ByteBpeTokenizer::from_tokenizer_json_bytes(TOKENIZER_JSON.as_bytes())
        .ok_or_else(|| "invalid fixture tokenizer".into())
}

#[test]
fn manifest_is_deterministic_matched_and_leak_free() -> TestResult {
    let tokenizer = tokenizer()?;
    let first = build_manifest(&tokenizer)?;
    let second = build_manifest(&tokenizer)?;
    assert_eq!(
        serde_json::to_string_pretty(&first)?,
        serde_json::to_string_pretty(&second)?
    );
    validate_manifest(&first)?;
    assert_eq!(first.rows.len(), 40);
    let mut values = std::collections::BTreeSet::new();
    for row in &first.rows {
        assert!(values.insert(row.value.clone()));
        assert_ne!(row.value, row.random_hint);
        assert_eq!(row.value_tokens.len(), row.random_tokens.len());
        assert_eq!(row.prompt_tokens.len(), row.random_prompt_tokens.len());
        assert!(row.prompt_tokens.len() <= 192);
        assert!(!row.no_hint_prompt.contains(&row.value));
        assert!(!row.random_prompt.contains(&row.value));
        assert!(!row.no_hint_prompt.contains("supplied value is"));
        assert!(row.prompt.contains("supplied value is"));
        assert!(row.random_prompt.contains("supplied value is"));
    }
    Ok(())
}

#[test]
fn decoder_accepts_only_one_exact_eight_letter_value() -> TestResult {
    assert_eq!(normalize_emission("abcdefgh")?, "abcdefgh");
    assert_eq!(normalize_emission("  a b c d e f g h  ")?, "abcdefgh");
    assert!(normalize_emission("abcdefg").is_err());
    assert!(normalize_emission("abcdefghi").is_err());
    assert!(normalize_emission("abcdefgh.").is_err());
    assert!(normalize_emission("abcde-fg").is_err());
    Ok(())
}

#[test]
fn corpus_leak_check_finds_an_exact_value_token_sequence() -> TestResult {
    let tokenizer = tokenizer()?;
    let manifest = build_manifest(&tokenizer)?;
    assert!(values_missing_from_corpus(&manifest, &[]));
    assert!(!values_missing_from_corpus(
        &manifest,
        &manifest.rows[0].value_tokens
    ));
    Ok(())
}

#[test]
fn validator_rejects_duplicate_values() -> TestResult {
    let tokenizer = tokenizer()?;
    let mut manifest = build_manifest(&tokenizer)?;
    manifest.rows[1].value = manifest.rows[0].value.clone();
    assert!(validate_manifest(&manifest).is_err());
    Ok(())
}

fn pinned_manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/labs/m1-address-as-read/stage0-copy.json")
}

#[test]
fn pinned_copy_manifest_hashes_and_parses() -> TestResult {
    let path = pinned_manifest_path();
    let bytes = std::fs::read(&path)?;
    let digest = format!("{:x}", Sha256::digest(&bytes));
    assert_eq!(
        digest,
        MANIFEST_SHA256,
        "the pinned Stage 0 manifest moved: {}",
        path.display()
    );
    assert_eq!(
        MANIFEST_SHA256,
        "ea36011870e3f6d0add87e9fc1d25115ffc77f2a9206d13bfe304c6785351fa5"
    );
    let manifest = parse_manifest(&bytes)?;
    assert_eq!(manifest.schema, STAGE0_SCHEMA);
    assert_eq!(manifest.rows.len(), 40);
    for row in &manifest.rows {
        // The runner's copy probe is non-vacuous only with real tokens.
        assert!(!row.value_tokens.is_empty());
        assert!(!row.prompt_tokens.is_empty());
        assert!(!row.random_prompt_tokens.is_empty());
        assert!(!row.no_hint_prompt_tokens.is_empty());
        assert!(row.prompt_tokens.len() < PROMPT_TOKEN_LIMIT);
        assert!(row.random_prompt_tokens.len() < PROMPT_TOKEN_LIMIT);
        assert!(row.no_hint_prompt_tokens.len() < PROMPT_TOKEN_LIMIT);
        for prompt_tokens in [
            &row.prompt_tokens,
            &row.random_prompt_tokens,
            &row.no_hint_prompt_tokens,
        ] {
            assert!(prompt_tokens.len() + row.value_tokens.len() <= PROMPT_TOKEN_LIMIT);
        }
    }
    validate_manifest(&manifest)?;
    Ok(())
}

#[test]
fn parse_manifest_rejects_an_unknown_field_or_wrong_schema() -> TestResult {
    let tokenizer = tokenizer()?;
    let manifest = build_manifest(&tokenizer)?;
    let mut value = serde_json::to_value(&manifest)?;
    value["rows"][0]["extra"] = json!(1);
    assert!(parse_manifest(&serde_json::to_vec(&value)?).is_err());
    let mut value = serde_json::to_value(&manifest)?;
    value["schema"] = json!("uor-r4.address-stage0-copy/2");
    assert!(parse_manifest(&serde_json::to_vec(&value)?).is_err());
    Ok(())
}

#[test]
fn original_report_rows_are_collected_recursively_in_traversal_order() -> TestResult {
    let report = json!({
        "schema": "uor-r4.discrete-corpus-driver/1",
        "aggregate": {"scored_rows": 2, "top1_correct": 1, "target_admitted_rows": 2},
        "batches": [
            {"rows": [
                {"target_position": 41, "target_token": 1155,
                 "top8": [552, 561, 935, 542, 690, 772, 971, 1258],
                 "top1_correct": 0, "admitted_cells": 8, "target_cell_rank": 9,
                 "unrelated": true}
            ]},
            {"nested": {"deeper": [
                {"target_position": 42, "target_token": 1156,
                 "top8": [1156, 2, 3, 4, 5, 6, 7, 8],
                 "top1_correct": 1, "admitted_cells": 7, "target_cell_rank": 7}
            ]}}
        ],
        "count_reference": {"top8": [1, 2, 3, 4, 5, 6, 7, 8], "top1_correct": 1, "scored_rows": 2}
    });
    let rows = collect_original_rows(&report)?;
    assert_eq!(
        rows,
        vec![
            OriginalRow {
                target_position: 41,
                target_token: 1155,
                top8: [552, 561, 935, 542, 690, 772, 971, 1258],
                top1_correct: 0,
                admitted_cells: 8,
                target_cell_rank: 9,
            },
            OriginalRow {
                target_position: 42,
                target_token: 1156,
                top8: [1156, 2, 3, 4, 5, 6, 7, 8],
                top1_correct: 1,
                admitted_cells: 7,
                target_cell_rank: 7,
            },
        ]
    );
    let malformed = json!({"target_position": 1, "target_token": 2,
                           "top8": [1, 2], "top1_correct": 0, "admitted_cells": 8});
    assert!(collect_original_rows(&malformed).is_err());
    let missing = json!({"target_position": 1, "target_token": 2,
                         "top8": [1, 2, 3, 4, 5, 6, 7, 8], "top1_correct": 0});
    assert!(collect_original_rows(&missing)?.is_empty());
    Ok(())
}

#[test]
fn primary_margin_uses_the_registered_strict_floor() {
    assert!(clears_primary(25, 0));
    assert!(!clears_primary(24, 0));
    assert!(clears_primary(26, 1));
    // d = 28 and d^2 = 4(b + c) exactly; the clause is strict.
    assert!(!clears_primary(112, 84));
    assert!(clears_primary(113, 84));
    assert!(!clears_primary(0, 0));
    assert!(primary_pass(
        PairedWins { b: 25, c: 0 },
        PairedWins { b: 26, c: 1 }
    ));
    assert!(!primary_pass(
        PairedWins { b: 25, c: 0 },
        PairedWins { b: 24, c: 0 }
    ));
}

#[test]
fn mismatch_stop_report_carries_all_three_aggregates_and_the_delta() {
    let mut report = json!({
        "schema": "uor-r4.address-stage0-parity/1",
        "rows": 14_287,
        "comparison": {"mismatched_rows": 3, "first_mismatches": []},
    });
    // The runner calls this immediately after the D11 aggregate and before
    // its mismatch, top-1/top-8 and admission stops.
    let admission_delta = set_parity_aggregates(
        &mut report,
        (460, 1_500, 5_010),
        (455, 1_484, 5_009),
        14_287,
    );
    // The runner's stop_parity uses this same transition.
    mark_parity_stop(&mut report, "3 per-row parity mismatches");

    assert_eq!(report["pass"].as_bool(), Some(false));
    assert_eq!(
        report["instrument_stop"].as_str(),
        Some("3 per-row parity mismatches")
    );
    assert_eq!(report["comparison"]["mismatched_rows"].as_u64(), Some(3));
    assert_eq!(report["d11_aggregate"]["rows"].as_u64(), Some(14_287));
    assert_eq!(report["d11_aggregate"]["top1"].as_u64(), Some(460));
    assert_eq!(report["d11_aggregate"]["top8"].as_u64(), Some(1_500));
    assert_eq!(
        report["d11_aggregate"]["target_admitted"].as_u64(),
        Some(5_010)
    );
    assert_eq!(report["original_aggregate"]["rows"].as_u64(), Some(14_287));
    assert_eq!(report["original_aggregate"]["top1"].as_u64(), Some(455));
    assert_eq!(report["original_aggregate"]["top8"].as_u64(), Some(1_484));
    assert_eq!(
        report["original_aggregate"]["target_admitted"].as_u64(),
        Some(5_009)
    );
    assert_eq!(report["position_strip"]["d11_top1"].as_u64(), Some(460));
    assert_eq!(
        report["position_strip"]["retained_top1"].as_u64(),
        Some(455)
    );
    assert_eq!(report["position_strip"]["top1_delta"].as_i64(), Some(5));
    assert_eq!(report["position_strip"]["d11_top8"].as_u64(), Some(1_500));
    assert_eq!(
        report["position_strip"]["retained_top8"].as_u64(),
        Some(1_484)
    );
    assert_eq!(report["position_strip"]["top8_delta"].as_i64(), Some(16));
    assert_eq!(
        report["position_strip"]["d11_target_admitted"].as_u64(),
        Some(5_010)
    );
    assert_eq!(
        report["position_strip"]["original_target_admitted"].as_u64(),
        Some(5_009)
    );
    assert_eq!(
        report["position_strip"]["admission_delta"].as_i64(),
        Some(1)
    );
    assert_eq!(
        report["position_strip"]["admission_aggregate_matches"].as_bool(),
        Some(false)
    );
    assert_eq!(admission_delta, 1);
}

#[test]
fn planted_wrong_value_must_differ_on_every_applicable_row() -> TestResult {
    let tokenizer = tokenizer()?;
    let manifest = build_manifest(&tokenizer)?;
    let different: Vec<Option<String>> = manifest
        .rows
        .iter()
        .map(|row| Some(row.random_hint.clone()))
        .collect();
    assert!(planted_wrong_holds(&manifest, &different)?);
    let mut one_exact = different.clone();
    one_exact[3] = Some(manifest.rows[3].value.clone());
    assert!(!planted_wrong_holds(&manifest, &one_exact)?);
    assert!(!planted_wrong_holds(&manifest, &vec![None; 40])?);
    assert!(planted_wrong_holds(&manifest, &different[..39]).is_err());
    Ok(())
}

#[test]
fn u16_corpus_reads_skip_the_uort_header_and_are_little_endian() -> TestResult {
    assert_eq!(u16_le_tokens(&[1, 0, 2, 0, 255, 255])?, vec![1, 2, 65_535]);
    let mut bytes = vec![0u8; 64];
    bytes[..4].copy_from_slice(b"UORT");
    bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
    bytes[8..16].copy_from_slice(&2u64.to_le_bytes());
    bytes[16..20].copy_from_slice(&4096u32.to_le_bytes());
    bytes.extend_from_slice(&[7, 0, 9, 0]);
    assert_eq!(u16_le_tokens(&bytes)?, vec![7, 9]);
    assert!(u16_le_tokens(&[1]).is_err());
    assert!(u16_le_tokens(&bytes[..40]).is_err());
    Ok(())
}

#[test]
fn corpus_value_leaks_names_rows_whose_exact_tokens_appear() -> TestResult {
    let tokenizer = tokenizer()?;
    let manifest = build_manifest(&tokenizer)?;
    assert!(corpus_value_leaks(&manifest, &[]).is_empty());
    let appears = |haystack: &[u32], needle: &[u32]| {
        !needle.is_empty()
            && haystack
                .windows(needle.len())
                .any(|window| window == needle)
    };
    for row in &manifest.rows {
        let leaks = corpus_value_leaks(&manifest, &row.value_tokens);
        if row.value_tokens.is_empty() {
            // The tiny fixture tokenizer drops bytes outside its vocabulary;
            // an empty sequence is never reported as a leak.
            assert!(!leaks.contains(&row.id));
            continue;
        }
        assert!(leaks.contains(&row.id));
        // The fixture tokenizer can map one value's token sequence into
        // another's; every leaked row must really appear in this corpus.
        for id in &leaks {
            let leaked = manifest.rows.iter().find(|r| &r.id == id).unwrap();
            assert!(appears(&row.value_tokens, &leaked.value_tokens));
        }
    }
    Ok(())
}

#[test]
fn stage0_head_must_be_a_full_lowercase_commit() {
    let full = "0123456789abcdef0123456789abcdef01234567";
    assert!(validate_head(full).is_ok());
    assert!(validate_head(&full[..39]).is_err());
    assert!(validate_head(&format!("{}g", &full[..39])).is_err());
    assert!(validate_head(&full.to_uppercase()).is_err());
    assert!(validate_head("").is_err());
}

#[test]
fn retained_receipt_and_stage_t_archive_pins_are_registered() -> TestResult {
    assert_eq!(
        RETAINED_RECEIPT_SHA256,
        "a7d23490c511fc7a391c95546646cd6aaef391950a5fb541fd13d2029a6edf8e"
    );
    assert_eq!(
        STAGE_T_ARCHIVE_SHA256,
        "c004db506bf64903231c5c225a07e9969d5e3344c51a67eaec9429265e5f44b6"
    );
    let retained_config = std::fs::read(pinned_config_path())?;
    assert_eq!(
        format!("{:x}", Sha256::digest(&retained_config)),
        ORIGINAL_CONFIG_SHA256
    );
    assert!(CONFIG_PATH_NORMALIZATION_NOTE.contains("sed"));
    Ok(())
}

fn pinned_config_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/labs/m1-bvg-k-stage-t-training/configs/t16-frozen-zero-seed1.json")
}

fn aggregate_stop_report(reason: &str) -> serde_json::Value {
    let mut report = json!({
        "schema": "uor-r4.address-stage0-parity/1",
        "rows": 2,
        "comparison": {"mismatched_rows": 0},
    });
    let delta = set_parity_aggregates(&mut report, (1, 1, 1), (1, 1, 1), 2);
    assert_eq!(delta, 0);
    mark_parity_stop(&mut report, reason);
    report
}

#[test]
fn nonzero_status_stop_report_carries_aggregates() {
    let report = aggregate_stop_report("a D11 parity row has nonzero status");
    assert_eq!(
        report["instrument_stop"].as_str(),
        Some("a D11 parity row has nonzero status")
    );
    assert_eq!(report["d11_aggregate"]["rows"].as_u64(), Some(2));
    assert_eq!(report["d11_aggregate"]["top1"].as_u64(), Some(1));
    assert_eq!(report["original_aggregate"]["top1"].as_u64(), Some(1));
    assert_eq!(report["position_strip"]["d11_top1"].as_u64(), Some(1));
    assert_eq!(
        report["position_strip"]["retained_top1"].as_u64(),
        Some(455)
    );
}

#[test]
fn nondeterminism_stop_report_carries_aggregates() {
    let report = aggregate_stop_report("the two D11 parity runs differ byte-for-byte");
    assert_eq!(
        report["instrument_stop"].as_str(),
        Some("the two D11 parity runs differ byte-for-byte")
    );
    assert!(report["d11_aggregate"].is_object());
    assert!(report["original_aggregate"].is_object());
    assert!(report["position_strip"].is_object());
    assert_eq!(
        report["position_strip"]["retained_top8"].as_u64(),
        Some(1_484)
    );
}

#[test]
fn admission_delta_stop_report_carries_aggregates() {
    let mut report = json!({"schema": "uor-r4.address-stage0-parity/1", "rows": 2});
    let delta = set_parity_aggregates(&mut report, (1, 1, 2), (1, 1, 1), 2);
    mark_parity_stop(&mut report, "admission aggregate differs");
    assert_eq!(delta, 1);
    assert_eq!(
        report["position_strip"]["admission_delta"].as_i64(),
        Some(1)
    );
    assert_eq!(
        report["position_strip"]["admission_aggregate_matches"].as_bool(),
        Some(false)
    );
    assert_eq!(
        report["position_strip"]["d11_target_admitted"].as_u64(),
        Some(2)
    );
    assert_eq!(
        report["position_strip"]["original_target_admitted"].as_u64(),
        Some(1)
    );
    assert!(report["d11_aggregate"].is_object());
    assert!(report["original_aggregate"].is_object());
}

#[test]
fn padded_top8_pads_and_refuses_nine() -> TestResult {
    assert_eq!(padded_top8(&[])?, [u32::MAX; 8]);
    assert_eq!(
        padded_top8(&[1, 2])?,
        [
            1,
            2,
            u32::MAX,
            u32::MAX,
            u32::MAX,
            u32::MAX,
            u32::MAX,
            u32::MAX
        ]
    );
    assert!(padded_top8(&[0; 9]).is_err());
    Ok(())
}

#[test]
fn parity_rows_include_schema_and_admitted_count() -> TestResult {
    let rows = vec![D11ParityRow {
        ordinal: 7,
        target_position: 41,
        target_token: 2,
        status: 0,
        prediction: 2,
        top8: padded_top8(&[2])?,
        admitted_cells: vec![10, 11, 12],
    }];
    let lines = parity_row_lines(&rows)?;
    assert_eq!(lines.iter().filter(|byte| **byte == b'\n').count(), 1);
    let line: serde_json::Value = serde_json::from_slice(&lines[..lines.len() - 1])?;
    assert_eq!(line["schema"], PARITY_ROW_SCHEMA);
    assert_eq!(line["ordinal"].as_u64(), Some(7));
    assert_eq!(line["admitted_count"].as_u64(), Some(3));
    assert_eq!(
        line["admitted_cells"].as_array().map(|cells| cells.len()),
        Some(3)
    );
    assert_eq!(parity_row_lines(&rows)?, lines);
    Ok(())
}

#[test]
fn compare_parity_rows_uses_positional_join_and_reports_admitted_count() -> TestResult {
    let original = vec![OriginalRow {
        target_position: 10,
        target_token: 30,
        top8: [30, 31, 32, 33, 34, 35, 36, 37],
        top1_correct: 1,
        admitted_cells: 2,
        target_cell_rank: 1,
    }];
    let d11 = vec![D11ParityRow {
        ordinal: 0,
        target_position: 10,
        target_token: 30,
        status: 0,
        prediction: 30,
        top8: [30, 31, 32, 33, 34, 35, 36, 37],
        admitted_cells: vec![100, 101],
    }];
    let comparison = compare_parity_rows(&d11, &original)?;
    assert_eq!(comparison["mismatched_rows"].as_u64(), Some(0));
    let checks = comparison["checks"].as_array().expect("checks");
    assert!(!checks.iter().any(|value| value == "ordinal"));
    assert!(checks.iter().any(|value| value == "admitted_count"));
    assert!(comparison["row_order_join"]
        .as_str()
        .unwrap_or_default()
        .contains("no ordinal"));

    let mut bad = d11.clone();
    bad[0].target_position = 11;
    bad[0].prediction = 31;
    bad[0].top8 = [31, 32, 33, 34, 35, 36, 37, 38];
    bad[0].admitted_cells = vec![100];
    let comparison = compare_parity_rows(&bad, &original)?;
    assert_eq!(comparison["mismatched_rows"].as_u64(), Some(1));
    let first = &comparison["first_mismatches"][0];
    let reasons = first["reasons"].as_array().expect("reasons");
    for reason in [
        "target_position",
        "prediction",
        "top8",
        "admitted_count",
        "top1_correct",
    ] {
        assert!(
            reasons.iter().any(|value| value == reason),
            "missing {reason}"
        );
    }
    assert!(first.get("ordinal").is_none());
    assert!(first["d11"].get("ordinal").is_none());
    Ok(())
}

#[test]
fn d11_aggregate_skips_nonzero_status_and_counts_admission() -> TestResult {
    let target_cells = vec![10, 11, 12];
    let rows = vec![
        D11ParityRow {
            ordinal: 0,
            target_position: 1,
            target_token: 0,
            status: 0,
            prediction: 0,
            top8: padded_top8(&[0])?,
            admitted_cells: vec![10, 11],
        },
        D11ParityRow {
            ordinal: 1,
            target_position: 2,
            target_token: 1,
            status: 1,
            prediction: 0,
            top8: [u32::MAX; 8],
            admitted_cells: vec![11],
        },
        D11ParityRow {
            ordinal: 2,
            target_position: 3,
            target_token: 2,
            status: 0,
            prediction: 0,
            top8: padded_top8(&[0, 2])?,
            admitted_cells: vec![11],
        },
    ];
    assert_eq!(d11_aggregate_rows(&rows, &target_cells)?, (1, 2, 1));
    Ok(())
}

#[test]
fn original_aggregate_recomputes_admission_from_target_cell_rank() -> TestResult {
    let rows = vec![
        OriginalRow {
            target_position: 1,
            target_token: 7,
            top8: [7, 2, 3, 4, 5, 6, 7, 8],
            top1_correct: 1,
            admitted_cells: 8,
            target_cell_rank: 8,
        },
        OriginalRow {
            target_position: 2,
            target_token: 9,
            top8: [1, 2, 3, 4, 5, 6, 7, 8],
            top1_correct: 0,
            admitted_cells: 8,
            target_cell_rank: 9,
        },
    ];
    let report = json!({
        "aggregate": {
            "scored_rows": 2,
            "top1_correct": 1,
            "top8_correct": 1,
            "target_admitted_rows": 1,
        }
    });
    assert_eq!(original_aggregate(&report, &rows)?, (1, 1, 1));

    let mut bad_aggregate = report.clone();
    bad_aggregate["aggregate"]["target_admitted_rows"] = json!(0);
    assert!(original_aggregate(&bad_aggregate, &rows).is_err());

    let mut bad_rank = rows.clone();
    bad_rank[1].target_cell_rank = 8;
    assert!(original_aggregate(&report, &bad_rank).is_err());
    Ok(())
}

#[test]
fn manifest_rejects_prompt_at_limit() -> TestResult {
    let tokenizer = tokenizer()?;
    let mut manifest = build_manifest(&tokenizer)?;
    let row = &mut manifest.rows[0];
    row.prompt_tokens = vec![0; PROMPT_TOKEN_LIMIT];
    row.random_prompt_tokens = vec![0; PROMPT_TOKEN_LIMIT];
    row.value_tokens.clear();
    row.random_tokens.clear();
    assert!(validate_manifest(&manifest).is_err());
    Ok(())
}

#[test]
fn manifest_rejects_prompt_plus_value_over_limit() -> TestResult {
    let tokenizer = tokenizer()?;
    let mut manifest = build_manifest(&tokenizer)?;
    let row = &mut manifest.rows[0];
    row.prompt_tokens = vec![0; PROMPT_TOKEN_LIMIT - 2];
    row.random_prompt_tokens = vec![0; PROMPT_TOKEN_LIMIT - 2];
    row.value_tokens = vec![0; 3];
    row.random_tokens = vec![0; 3];
    assert!(validate_manifest(&manifest).is_err());
    Ok(())
}

#[test]
fn config_pins_cover_train_and_dev() -> TestResult {
    let config: serde_json::Value = serde_json::from_slice(&std::fs::read(pinned_config_path())?)?;
    validate_config_pins(&config)?;
    for pointer in [
        "/dev/tokens/sha256",
        "/dev/mask/sha256",
        "/train/tokens/sha256",
        "/train/mask/sha256",
        "/tokenizer/sha256",
    ] {
        let mut bad = config.clone();
        *bad.pointer_mut(pointer).expect("config pointer") = json!("00");
        assert!(validate_config_pins(&bad).is_err(), "{pointer}");
    }
    let mut bad = config.clone();
    *bad.pointer_mut("/dev/expected_eligible_targets")
        .expect("eligible pointer") = json!(0);
    assert!(validate_config_pins(&bad).is_err());
    Ok(())
}

#[test]
fn clears_primary_widens_before_sum() {
    assert!(clears_primary(u64::MAX, 1));
    assert!(!clears_primary(u64::MAX, u64::MAX));
}

#[test]
fn manifest_cli_pins_require_registered_hashes() {
    assert!(validate_manifest_cli_pins(TOKENIZER_SHA256, MANIFEST_SHA256).is_ok());
    assert!(validate_manifest_cli_pins("00", MANIFEST_SHA256).is_err());
    assert!(validate_manifest_cli_pins(TOKENIZER_SHA256, "00").is_err());
}

#[test]
fn head_is_checked_against_git_head() -> TestResult {
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR"));
    let head = git_head(checkout)?;
    validate_head_against_git(&head, checkout)?;
    assert!(validate_head_against_git(&"0".repeat(40), checkout).is_err());
    Ok(())
}

#[test]
fn device_init_failure_does_not_create_out_dir() -> TestResult {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let out_dir = std::env::temp_dir().join(format!(
        "uor-r4-stage0-no-out-{}-{nanos}",
        std::process::id()
    ));
    assert!(!out_dir.exists());
    let result = create_out_dir_after(&out_dir, || Err::<(), _>("device init failed".to_string()));
    assert!(result.is_err());
    assert!(!out_dir.exists());
    Ok(())
}
