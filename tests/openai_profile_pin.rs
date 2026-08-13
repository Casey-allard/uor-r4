//! #654 phase A: drift gate for the pinned `r4-openai-profile`.
//!
//! Keeps the vendored OpenAI specification and the generated compatibility
//! matrix consistent, so an unreviewed spec bump or a hand-edit of the matrix
//! fails CI rather than silently widening or drifting the profile:
//!
//! - the vendored `profiles/openai/openapi.yaml` is byte-identical to the
//!   pinned upstream blob — its blake3 matches the pin recorded in the matrix
//!   (the git blob sha1 is recorded provenance, reproducible with
//!   `git hash-object`);
//! - the matrix classifies EXACTLY the operations the spec declares — no
//!   missing operation, and none the spec no longer has;
//! - every classification is one of the three profile categories, the counts
//!   are honest, and the `supported` set is exactly the pinned phase-1
//!   text-serving profile, so support cannot silently widen.
//!
//! Regenerate the matrix with `scripts/gen_openai_compat_matrix.py` after any
//! reviewed spec bump.

use std::collections::BTreeSet;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn profile_dir() -> PathBuf {
    root().join("profiles").join("openai")
}

fn matrix() -> serde_json::Value {
    let bytes = std::fs::read(profile_dir().join("compatibility_matrix.json"))
        .expect("read compatibility_matrix.json");
    serde_json::from_slice(&bytes).expect("compatibility matrix parses as JSON")
}

/// Every operationId the spec declares, re-derived by the same 6-space
/// indentation scan the generator uses.
fn spec_operation_ids(spec: &str) -> BTreeSet<String> {
    spec.lines()
        .filter_map(|line| line.strip_prefix("      operationId:"))
        .map(|rest| rest.trim().to_owned())
        .collect()
}

#[test]
fn vendored_spec_matches_the_pinned_blake3() {
    let spec =
        std::fs::read(profile_dir().join("openapi.yaml")).expect("read vendored openapi.yaml");
    let measured = format!("blake3:{}", blake3::hash(&spec).to_hex());
    let pinned = matrix()["spec_pin"]["openapi_yaml_blake3"]
        .as_str()
        .expect("pin records the spec blake3")
        .to_owned();
    assert_eq!(
        measured, pinned,
        "the vendored openapi.yaml must be byte-identical to the pinned upstream blob"
    );
}

#[test]
fn matrix_covers_exactly_the_spec_operations() {
    let spec = std::fs::read_to_string(profile_dir().join("openapi.yaml")).expect("read spec");
    let spec_ids = spec_operation_ids(&spec);
    assert!(!spec_ids.is_empty(), "the spec declares operations");

    let doc = matrix();
    let matrix_ids: BTreeSet<String> = doc["operations"]
        .as_array()
        .expect("operations array")
        .iter()
        .map(|op| {
            op["operation_id"]
                .as_str()
                .expect("operation_id is a string")
                .to_owned()
        })
        .collect();

    // Drift both ways: a spec bump that adds/removes an operation, or a matrix
    // edit that diverges from the spec, fails here.
    let missing: Vec<&String> = spec_ids.difference(&matrix_ids).collect();
    let extra: Vec<&String> = matrix_ids.difference(&spec_ids).collect();
    assert!(
        missing.is_empty(),
        "operations in the spec but absent from the matrix (regenerate the matrix): {missing:?}"
    );
    assert!(
        extra.is_empty(),
        "operations in the matrix but absent from the spec (spec drift): {extra:?}"
    );

    // The recorded counts are honest.
    let total = doc["counts"]["total"].as_u64().expect("total count");
    assert_eq!(total as usize, spec_ids.len(), "counts.total == spec ops");
}

#[test]
fn classifications_are_valid_and_supported_set_is_pinned() {
    let doc = matrix();
    let operations = doc["operations"].as_array().expect("operations array");

    let mut counts = std::collections::BTreeMap::new();
    let mut supported: BTreeSet<String> = BTreeSet::new();
    for op in operations {
        let class = op["classification"]
            .as_str()
            .expect("classification string");
        assert!(
            matches!(class, "supported" | "unsupported" | "not-applicable"),
            "classification must be one of the three profile categories, got {class:?}"
        );
        *counts.entry(class.to_owned()).or_insert(0u64) += 1;
        if class == "supported" {
            supported.insert(
                op["operation_id"]
                    .as_str()
                    .expect("operation_id string")
                    .to_owned(),
            );
        }
    }

    // Recorded per-category counts match the operations.
    for category in ["supported", "unsupported", "not-applicable"] {
        let recorded = doc["counts"][category].as_u64().unwrap_or(0);
        let actual = counts.get(category).copied().unwrap_or(0);
        assert_eq!(recorded, actual, "counts.{category} matches the operations");
    }

    // The phase-1 text-serving profile is pinned: support cannot silently
    // widen without editing this test.
    let expected: BTreeSet<String> = [
        "createChatCompletion",
        "createResponse",
        "listModels",
        "retrieveModel",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    assert_eq!(
        supported, expected,
        "the supported set is exactly the pinned phase-1 text-serving profile"
    );
}
