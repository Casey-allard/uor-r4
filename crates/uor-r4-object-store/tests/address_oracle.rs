//! Finite host controls for the object-store address oracle.

use serde_json::json;
use std::error::Error as StdError;
use uor_r4_object_store::address_oracle::*;
use uor_r4_object_store::address_read::{
    AddressDirectory, AddressReadOutcome, ObjectTarget, PrimeAddress, VersionSelect,
};
use uor_r4_object_store::bindings::{ingest, PinnedTokenizer};
use uor_r4_object_store::Error as StoreError;
use uor_r4_object_store::{Limits, Store};
use uor_r4_tokenizer::ByteBpeTokenizer;

type TestResult<T = ()> = Result<T, Box<dyn StdError>>;

/// A byte-level fixture carrying every lowercase ASCII letter and the GPT-2
/// space symbol, so prompts and eight-letter values round-trip exactly.
const TOKENIZER_JSON: &str = r#"{
    "pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},
    "model":{"type":"BPE","vocab":{
        "a":0,"b":1,"c":2,"d":3,"e":4,"f":5,"g":6,"h":7,"i":8,"j":9,"k":10,"l":11,"m":12,
        "n":13,"o":14,"p":15,"q":16,"r":17,"s":18,"t":19,"u":20,"v":21,"w":22,"x":23,"y":24,"z":25,
        "Ġ":26
    },"merges":[]}
}"#;

fn tokenizer() -> TestResult<ByteBpeTokenizer> {
    ByteBpeTokenizer::from_tokenizer_json_bytes(TOKENIZER_JSON.as_bytes())
        .ok_or_else(|| "invalid fixture tokenizer".into())
}

fn fixture() -> TestResult<(ByteBpeTokenizer, PanelManifest)> {
    let tokenizer = tokenizer()?;
    let panel = build_panel(&tokenizer)?;
    validate_panel(&panel, Some(&tokenizer))?;
    Ok((tokenizer, panel))
}

fn hit_bytes(outcome: &AddressReadOutcome) -> Option<&[u8]> {
    match outcome {
        AddressReadOutcome::Hit { bytes, .. } | AddressReadOutcome::Previous { bytes, .. } => {
            Some(bytes.as_slice())
        }
        _ => None,
    }
}

fn hit_address(outcome: &AddressReadOutcome) -> Option<&PrimeAddress> {
    match outcome {
        AddressReadOutcome::Hit { address, .. } | AddressReadOutcome::Previous { address, .. } => {
            Some(address)
        }
        _ => None,
    }
}

fn assert_invalid(panel: &PanelManifest) {
    assert!(
        validate_panel(panel, None).is_err(),
        "panel unexpectedly validated"
    );
}

fn expected_text(row: &PanelRow) -> TestResult<&str> {
    Ok(row
        .expected_value()
        .ok_or("value row without an expected value")?
        .text
        .as_str())
}

struct ArmRecords {
    true_records: Vec<ArmRecord>,
    sieve_records: Vec<ArmRecord>,
    random_records: Vec<ArmRecord>,
    no_memory: Vec<ArmRecord>,
    no_address: Vec<ArmRecord>,
}

fn arm_records(panel: &PanelManifest, store: &OracleStore) -> TestResult<ArmRecords> {
    let true_records = panel
        .rows
        .iter()
        .map(|row| store.observe_store_arm(row, Arm::TrueAddress))
        .collect::<Result<Vec<_>, String>>()?;
    let sieve_records = panel
        .rows
        .iter()
        .map(|row| store.observe_store_arm(row, Arm::Sieve))
        .collect::<Result<Vec<_>, String>>()?;
    let random_records = panel
        .rows
        .iter()
        .map(|row| store.observe_store_arm(row, Arm::RandomAddress))
        .collect::<Result<Vec<_>, String>>()?;
    let score_baseline = |arm: Arm| {
        panel
            .rows
            .iter()
            .map(|row| {
                score_model_arm(
                    row,
                    arm,
                    row.expected_value().map(|value| value.text.as_str()),
                )
            })
            .collect::<Result<Vec<_>, String>>()
    };
    Ok(ArmRecords {
        true_records,
        sieve_records,
        random_records,
        no_memory: score_baseline(Arm::NoMemory)?,
        no_address: score_baseline(Arm::NoAddress)?,
    })
}

/// `(rows, b, c)` for one family only. Never pool families into one count.
fn expected_wins(
    panel: &PanelManifest,
    family: RowFamily,
    treatment: &[ArmRecord],
    control: &[ArmRecord],
) -> (u64, u64, u64) {
    let mut rows = 0u64;
    let mut b = 0u64;
    let mut c = 0u64;
    for ((row, treatment_record), control_record) in panel.rows.iter().zip(treatment).zip(control) {
        if row.family != family {
            continue;
        }
        rows += 1;
        if treatment_record.correct() && !control_record.correct() {
            b += 1;
        }
        if control_record.correct() && !treatment_record.correct() {
            c += 1;
        }
    }
    (rows, b, c)
}

#[test]
fn five_arm_enum_and_source_markers_are_stable() -> TestResult {
    assert_eq!(Arm::ALL.len(), 5);
    for (arm, encoded) in [
        (Arm::NoMemory, "\"NoMemory\""),
        (Arm::NoAddress, "\"NoAddress\""),
        (Arm::Sieve, "\"Sieve\""),
        (Arm::TrueAddress, "\"TrueAddress\""),
        (Arm::RandomAddress, "\"RandomAddress\""),
    ] {
        assert_eq!(serde_json::to_string(&arm)?, encoded);
        assert_eq!(serde_json::from_str::<Arm>(encoded)?, arm);
    }
    assert!(Arm::NoMemory.is_model());
    assert!(Arm::NoAddress.is_model());
    assert!(Arm::Sieve.is_store());
    assert!(Arm::TrueAddress.is_store());
    assert!(Arm::RandomAddress.is_store());
    assert_eq!(
        serde_json::to_string(&EmissionSource::ExactStore)?,
        "\"ExactStore\""
    );
    assert_eq!(
        serde_json::to_string(&EmissionSource::SieveStore)?,
        "\"SieveStore\""
    );
    assert_eq!(serde_json::to_string(&EmissionSource::Model)?, "\"Model\"");
    Ok(())
}

#[test]
fn panel_is_deterministic_byte_identical_and_has_exact_families() -> TestResult {
    let tokenizer = tokenizer()?;
    let first = build_panel(&tokenizer)?;
    let second = build_panel(&tokenizer)?;
    let bytes = encode_panel(&first)?;
    assert_eq!(bytes, encode_panel(&second)?);
    assert_eq!(bytes, first.encode()?);

    let parsed = PanelManifest::parse(&bytes)?;
    assert_eq!(parsed, first);
    assert_eq!(parsed.encode()?, bytes);
    assert_eq!(parsed.sha256()?, first.sha256()?);
    assert_eq!(panel_sha256(&first)?, panel_sha256(&second)?);

    assert_eq!(first.schema, PANEL_SCHEMA);
    assert_eq!(first.panel_version, PANEL_VERSION);
    assert_eq!(first.seed, PANEL_SEED);
    assert_eq!(first.rows.len(), PANEL_ROWS);
    let current = first
        .rows
        .iter()
        .filter(|row| row.family == RowFamily::Current)
        .count();
    let previous = first
        .rows
        .iter()
        .filter(|row| row.family == RowFamily::Previous)
        .count();
    let abstention = first
        .rows
        .iter()
        .filter(|row| row.family == RowFamily::Abstention)
        .count();
    assert_eq!(
        (current, previous, abstention),
        (CURRENT_ROWS, PREVIOUS_ROWS, ABSTENTION_ROWS)
    );
    let absent = first
        .rows
        .iter()
        .filter(|row| row.expected_abstention() == Some(AbstentionKind::Absent))
        .count();
    let evicted = first
        .rows
        .iter()
        .filter(|row| row.expected_abstention() == Some(AbstentionKind::Evicted))
        .count();
    assert_eq!((absent, evicted), (ABSENT_ROWS, EVICTED_ROWS));

    assert_eq!(first.limits, PanelLimits::default());
    assert_eq!(first.limits.max_slots, DEFAULT_MAX_SLOTS);
    assert_eq!(first.lexicon.entries.len(), 24);
    assert_eq!(first.lexicon.schema, LEXICON_SCHEMA);
    assert_eq!(first.lexicon.version, LEXICON_VERSION);
    first.validate(Some(&tokenizer))?;

    let mut values = std::collections::BTreeSet::new();
    for row in &first.rows {
        assert!(!row.id.is_empty());
        assert!(!row.prompt_tokens.is_empty());
        assert_ne!(row.random.address, row.address);
        if let Some(expected) = row.expected_value() {
            assert_ne!(row.random.value.text, expected.text);
        }
        for value in row.history.iter().chain(std::iter::once(&row.random.value)) {
            assert_eq!(value.text.len(), 8);
            assert!(value.text.bytes().all(|byte| byte.is_ascii_lowercase()));
            assert!(
                values.insert(value.text.clone()),
                "duplicate value {}",
                value.text
            );
        }
    }
    assert!(prompt_value_leaks(&first).is_empty());
    assert!(corpus_value_leaks(&first, &[9999]).is_empty());
    Ok(())
}

#[test]
fn manifest_rejects_unknown_fields_bad_versions_and_bad_tokenizer_identity() -> TestResult {
    let (tokenizer, panel) = fixture()?;
    let value = serde_json::to_value(&panel)?;

    let mut extra_top = value.clone();
    extra_top["unexpected"] = json!(true);
    assert!(serde_json::from_value::<PanelManifest>(extra_top).is_err());

    let mut extra_row = value.clone();
    extra_row["rows"][0]["unexpected"] = json!(1);
    assert!(serde_json::from_value::<PanelManifest>(extra_row).is_err());

    let mut extra_lexicon = value.clone();
    extra_lexicon["lexicon"]["entries"][0]["unexpected"] = json!(1);
    assert!(serde_json::from_value::<PanelManifest>(extra_lexicon).is_err());

    let mut missing = value;
    missing["limits"]
        .as_object_mut()
        .ok_or("limits is not an object")?
        .remove("max_slots");
    assert!(serde_json::from_value::<PanelManifest>(missing).is_err());

    let mut bad_schema = panel.clone();
    bad_schema.schema = "uor-r4.address-oracle-panel/2".into();
    assert_invalid(&bad_schema);

    let mut bad_version = panel.clone();
    bad_version.panel_version = 2;
    assert_invalid(&bad_version);

    let mut bad_seed = panel.clone();
    bad_seed.seed = "other".into();
    assert_invalid(&bad_seed);

    let mut bad_identity = panel.clone();
    bad_identity.tokenizer_identity = "zz".repeat(32);
    assert_invalid(&bad_identity);

    let mut wrong_identity = panel.clone();
    wrong_identity.tokenizer_identity = "00".repeat(32);
    assert!(validate_panel(&wrong_identity, Some(&tokenizer)).is_err());
    // Structural validation cannot verify a tokenizer identity without its pin.
    assert!(validate_panel(&wrong_identity, None).is_ok());
    Ok(())
}

#[test]
fn panel_rejects_bad_addresses_pairs_tokens_and_bindings() -> TestResult {
    let (tokenizer, panel) = fixture()?;

    let mut composite = panel.clone();
    composite.rows[0].address.primes = vec![2, 9];
    composite.rows[0].address.pairs.clear();
    assert_invalid(&composite);
    assert!(composite.rows[0].address.to_prime_address().is_err());

    let mut outside = panel.clone();
    outside.rows[0].address.pairs = vec![(2, 999)];
    assert_invalid(&outside);

    let mut empty = panel.clone();
    empty.rows[0].address.primes.clear();
    empty.rows[0].address.pairs.clear();
    assert_invalid(&empty);

    let mut unsorted = panel.clone();
    unsorted.rows[0].address.primes.reverse();
    assert_invalid(&unsorted);

    let mut duplicate_pair = panel.clone();
    let pair = duplicate_pair.rows[0].address.pairs[0];
    duplicate_pair.rows[0].address.pairs.push(pair);
    assert_invalid(&duplicate_pair);

    let mut bad_token = panel.clone();
    bad_token.rows[0].random.value.tokens[0] = 99_999;
    assert!(validate_panel(&bad_token, Some(&tokenizer)).is_err());
    // Token IDs are only checkable against the pinned tokenizer.
    assert!(validate_panel(&bad_token, None).is_ok());

    let mut mismatch_token = panel.clone();
    let original = mismatch_token.rows[0].random.value.tokens[0];
    mismatch_token.rows[0].random.value.tokens[0] = if original == 0 { 1 } else { 0 };
    assert!(validate_panel(&mismatch_token, Some(&tokenizer)).is_err());

    let mut bad_prompt_token = panel.clone();
    bad_prompt_token.rows[0].prompt_tokens[0] = 99_999;
    assert!(validate_panel(&bad_prompt_token, Some(&tokenizer)).is_err());
    Ok(())
}

#[test]
fn random_bindings_are_present_value_different_and_control_holds() -> TestResult {
    let (tokenizer, panel) = fixture()?;
    let store = OracleStore::build_with_tokenizer(&panel, &tokenizer)?;

    for row in &panel.rows {
        let random_addr = row.random_address()?;
        assert_ne!(row.random.address, row.address);
        if let Some(expected) = row.expected_value() {
            assert_ne!(row.random.value.text, expected.text);
        }
        let outcome = store.read_exact(&random_addr, RequestedVersion::Current)?;
        assert_eq!(hit_address(&outcome), Some(&random_addr));
        assert_eq!(hit_bytes(&outcome), Some(row.random.value.text.as_bytes()));
    }

    let records = panel
        .rows
        .iter()
        .map(|row| store.observe_store_arm(row, Arm::RandomAddress))
        .collect::<Result<Vec<_>, String>>()?;
    assert!(records.iter().all(|record| record.random_binding_pass));
    assert!(records.iter().all(|record| !record.value_check_pass));
    assert!(records.iter().all(|record| record.wrong_substitution));
    assert!(random_control_holds(&panel, &records)?);

    let mut collision = panel.clone();
    collision.rows[0].random.address = collision.rows[0].address.clone();
    assert_invalid(&collision);

    let mut same_value = panel.clone();
    same_value.rows[0].random.value = panel.rows[0]
        .expected_value()
        .ok_or("no expected value")?
        .clone();
    assert_invalid(&same_value);

    let mut empty_value = panel.clone();
    empty_value.rows[0].random.value.text.clear();
    empty_value.rows[0].random.value.tokens.clear();
    assert_invalid(&empty_value);

    let mut slots = panel.clone();
    slots.limits.max_slots += 1;
    assert_invalid(&slots);
    assert!(OracleStore::build(&slots).is_err());
    Ok(())
}

#[test]
fn sieve_lexicon_and_query_formation_are_versioned_and_deterministic() -> TestResult {
    let (_tokenizer, panel) = fixture()?;
    panel.lexicon.validate()?;

    let mut current_match = 0;
    let mut current_miss = 0;
    for row in panel
        .rows
        .iter()
        .filter(|row| row.family == RowFamily::Current)
    {
        if form_sieve_query(&panel.lexicon, &row.prompt)? == row.true_address()? {
            current_match += 1;
        } else {
            current_miss += 1;
        }
    }
    assert_eq!(current_match, 20);
    assert_eq!(current_miss, 20);
    for row in panel
        .rows
        .iter()
        .filter(|row| row.family == RowFamily::Previous)
    {
        assert_eq!(
            form_sieve_query(&panel.lexicon, &row.prompt)?,
            row.true_address()?
        );
    }
    for row in panel
        .rows
        .iter()
        .filter(|row| row.family == RowFamily::Abstention)
    {
        assert_eq!(
            form_sieve_query(&panel.lexicon, &row.prompt)?,
            row.true_address()?
        );
    }

    let entry = &panel.lexicon.entries[0];
    let single = form_sieve_query(&panel.lexicon, &format!("Please, {}. ", entry.word))?;
    assert_eq!(single.primes(), &[entry.prime]);
    assert!(single.ordered_pairs().is_empty());
    assert!(form_sieve_query(&panel.lexicon, "please and thank you").is_err());

    let long = format!("{} {}", entry.word, "x".repeat(HARD_MAX_PROMPT_BYTES));
    assert!(form_sieve_query(&panel.lexicon, &long).is_err());
    let too_many = vec![entry.word.as_str(); HARD_MAX_KEY_ATOMS + 1].join(" ");
    assert!(form_sieve_query(&panel.lexicon, &too_many).is_err());

    let mut unsorted = panel.lexicon.clone();
    unsorted.entries.swap(0, 1);
    assert!(unsorted.validate().is_err());
    let mut composite = panel.lexicon.clone();
    composite.entries[0].prime = 9;
    assert!(composite.validate().is_err());
    let mut duplicate = panel.lexicon.clone();
    duplicate.entries[1].prime = duplicate.entries[0].prime;
    assert!(duplicate.validate().is_err());
    let mut bad_version = panel.lexicon.clone();
    bad_version.version = 2;
    assert!(bad_version.validate().is_err());
    let mut empty = panel.lexicon.clone();
    empty.entries.clear();
    assert!(empty.validate().is_err());
    Ok(())
}

#[test]
fn exact_reads_follow_directory_current_previous_absent_and_evicted_semantics() -> TestResult {
    let (tokenizer, panel) = fixture()?;
    let store = OracleStore::build_with_tokenizer(&panel, &tokenizer)?;

    for row in &panel.rows {
        let true_addr = row.true_address()?;
        let random_addr = row.random_address()?;
        match row.family {
            RowFamily::Current => {
                let expected = expected_text(row)?;
                let current = store.read_exact(&true_addr, RequestedVersion::Current)?;
                assert!(matches!(current, AddressReadOutcome::Hit { .. }));
                assert_eq!(hit_address(&current), Some(&true_addr));
                assert_eq!(hit_bytes(&current), Some(expected.as_bytes()));
                assert!(matches!(
                    store.read_exact(&true_addr, RequestedVersion::Previous)?,
                    AddressReadOutcome::Absent
                ));
            }
            RowFamily::Previous => {
                let previous = store.read_exact(&true_addr, RequestedVersion::Previous)?;
                match &previous {
                    AddressReadOutcome::Previous {
                        address,
                        version,
                        bytes,
                        ..
                    } => {
                        assert_eq!(address, &true_addr);
                        assert_eq!(*version, 1);
                        assert_eq!(bytes.as_slice(), row.history[0].text.as_bytes());
                    }
                    other => panic!("expected Previous, got {other:?}"),
                }
                let current = store.read_exact(&true_addr, RequestedVersion::Current)?;
                match &current {
                    AddressReadOutcome::Hit {
                        address,
                        version,
                        bytes,
                        ..
                    } => {
                        assert_eq!(address, &true_addr);
                        assert_eq!(*version, 2);
                        assert_eq!(bytes.as_slice(), row.history[1].text.as_bytes());
                    }
                    other => panic!("expected current Hit, got {other:?}"),
                }
            }
            RowFamily::Abstention => match row.expected_abstention() {
                Some(AbstentionKind::Absent) => {
                    assert!(matches!(
                        store.read_exact(&true_addr, RequestedVersion::Current)?,
                        AddressReadOutcome::Absent
                    ));
                    assert!(matches!(
                        store.read_exact(&true_addr, RequestedVersion::Previous)?,
                        AddressReadOutcome::Absent
                    ));
                }
                Some(AbstentionKind::Evicted) => {
                    match store.read_exact(&true_addr, RequestedVersion::Current)? {
                        AddressReadOutcome::Evicted { address, .. } => {
                            assert_eq!(address, true_addr)
                        }
                        other => panic!("expected Evicted, got {other:?}"),
                    }
                    assert!(matches!(
                        store.read_exact(&true_addr, RequestedVersion::Previous)?,
                        AddressReadOutcome::Evicted { .. }
                    ));
                }
                None => panic!("abstention row without a typed abstention"),
            },
        }

        let random = store.read_exact(&random_addr, RequestedVersion::Current)?;
        assert_eq!(hit_address(&random), Some(&random_addr));
        assert_eq!(hit_bytes(&random), Some(row.random.value.text.as_bytes()));
    }

    assert_eq!(store.directory().len(), panel.limits.max_slots);
    assert_eq!(store.panel(), &panel);
    assert!(store.store().record_count() > 0);
    Ok(())
}

#[test]
fn overlapping_unequal_active_and_evicted_keys_never_alias() -> TestResult {
    let mut store = Store::new(Limits::default());
    let alpha = store.fold(b"alpha")?;
    let beta = store.fold(b"beta")?;
    let gamma = store.fold(b"gamma")?;
    let delta = store.fold(b"delta")?;

    let a = PrimeAddress::with_pairs(&[2, 5], &[(2, 5)])?;
    let a_reversed = PrimeAddress::with_pairs(&[2, 5], &[(5, 2)])?;
    let a_plain = PrimeAddress::new(&[2, 5])?;
    let a_composite = PrimeAddress::new(&[2, 5, 7])?;

    let mut dir = AddressDirectory::new(8)?;
    dir.write(
        a.clone(),
        ObjectTarget::WholeObject {
            manifest: alpha.manifest.clone(),
        },
    )?;
    dir.write(
        a_reversed.clone(),
        ObjectTarget::WholeObject {
            manifest: beta.manifest.clone(),
        },
    )?;
    dir.write(
        a_composite.clone(),
        ObjectTarget::WholeObject {
            manifest: gamma.manifest,
        },
    )?;

    assert_eq!(
        hit_bytes(&dir.read_exact(&store, None, &a, VersionSelect::Current)?),
        Some(&b"alpha"[..])
    );
    assert_eq!(
        hit_bytes(&dir.read_exact(&store, None, &a_reversed, VersionSelect::Current)?),
        Some(&b"beta"[..])
    );
    assert_eq!(
        hit_bytes(&dir.read_exact(&store, None, &a_composite, VersionSelect::Current)?),
        Some(&b"gamma"[..])
    );
    assert_eq!(
        dir.read_exact(&store, None, &a_plain, VersionSelect::Current)?,
        AddressReadOutcome::Absent
    );

    // Capacity 1: writing b evicts a. Overlapping-but-unequal keys must be
    // Absent, never a tombstone for the evicted key.
    let b = PrimeAddress::new(&[11, 13])?;
    let mut evicting = AddressDirectory::new(1)?;
    evicting.write(
        a.clone(),
        ObjectTarget::WholeObject {
            manifest: alpha.manifest,
        },
    )?;
    evicting.write(
        b,
        ObjectTarget::WholeObject {
            manifest: delta.manifest,
        },
    )?;
    match evicting.read_exact(&store, None, &a, VersionSelect::Current)? {
        AddressReadOutcome::Evicted { address, .. } => assert_eq!(address, a),
        other => panic!("expected Evicted, got {other:?}"),
    }
    for query in [&a_reversed, &a_composite, &a_plain] {
        assert_eq!(
            evicting.read_exact(&store, None, query, VersionSelect::Current)?,
            AddressReadOutcome::Absent
        );
    }

    assert!(matches!(
        PrimeAddress::new(&[2, 9]),
        Err(StoreError::SchemaMismatch)
    ));
    assert!(matches!(
        PrimeAddress::with_pairs(&[2, 3], &[(3, 5)]),
        Err(StoreError::SchemaMismatch)
    ));
    assert!(AddressSpec {
        primes: vec![2, 9],
        pairs: vec![],
    }
    .to_prime_address()
    .is_err());
    assert!(AddressSpec {
        primes: vec![2, 3],
        pairs: vec![(3, 5)],
    }
    .to_prime_address()
    .is_err());
    assert!(AddressSpec {
        primes: vec![5, 2],
        pairs: vec![],
    }
    .to_prime_address()
    .is_err());
    assert!(AddressSpec {
        primes: vec![2, 3],
        pairs: vec![(2, 3), (2, 3)],
    }
    .to_prime_address()
    .is_err());
    assert!(AddressSpec {
        primes: vec![],
        pairs: vec![],
    }
    .to_prime_address()
    .is_err());
    Ok(())
}

#[test]
fn exact_reads_reject_invalid_token_bindings_and_indices() -> TestResult {
    let tokenizer = tokenizer()?;
    let pinned = PinnedTokenizer::new(&tokenizer)?;
    let mut store = Store::new(Limits::default());
    let ingested = ingest(&mut store, &pinned, b"abc")?;
    let other = store.fold(b"other")?;

    let mut dir = AddressDirectory::new(8)?;
    let bad_binding = PrimeAddress::new(&[2, 3])?;
    dir.write(
        bad_binding.clone(),
        ObjectTarget::TokenOccurrence {
            manifest: other.manifest,
            bindings: ingested.bindings.clone(),
            index: 0,
        },
    )?;
    assert!(matches!(
        dir.read_exact(&store, Some(&pinned), &bad_binding, VersionSelect::Current),
        Err(StoreError::BindingMismatch)
    ));

    let good_binding = PrimeAddress::new(&[2, 3, 5])?;
    dir.write(
        good_binding.clone(),
        ObjectTarget::TokenOccurrence {
            manifest: ingested.manifest.clone(),
            bindings: ingested.bindings.clone(),
            index: 0,
        },
    )?;
    assert_eq!(
        hit_bytes(&dir.read_exact(&store, Some(&pinned), &good_binding, VersionSelect::Current)?),
        Some(&b"a"[..])
    );

    let bad_index = PrimeAddress::new(&[2, 3, 7])?;
    dir.write(
        bad_index.clone(),
        ObjectTarget::TokenOccurrence {
            manifest: ingested.manifest,
            bindings: ingested.bindings,
            index: 99,
        },
    )?;
    assert!(matches!(
        dir.read_exact(&store, Some(&pinned), &bad_index, VersionSelect::Current),
        Err(StoreError::InvalidRange)
    ));
    Ok(())
}

#[test]
fn typed_abstention_source_markers_and_sieve_previous_fallback_are_separate() -> TestResult {
    let (tokenizer, panel) = fixture()?;
    let store = OracleStore::build_with_tokenizer(&panel, &tokenizer)?;
    let current = panel
        .rows
        .iter()
        .find(|row| row.family == RowFamily::Current)
        .ok_or("no current row")?;
    let previous = panel
        .rows
        .iter()
        .find(|row| row.family == RowFamily::Previous)
        .ok_or("no previous row")?;
    let absent = panel
        .rows
        .iter()
        .find(|row| row.expected_abstention() == Some(AbstentionKind::Absent))
        .ok_or("no absent row")?;
    let evicted = panel
        .rows
        .iter()
        .find(|row| row.expected_abstention() == Some(AbstentionKind::Evicted))
        .ok_or("no evicted row")?;

    let missing_current =
        score_store_read(current, Arm::TrueAddress, AddressReadOutcome::Absent, None)?;
    assert!(!missing_current.correct());
    assert!(!missing_current.wrong_substitution);
    assert!(missing_current.emitted_text.is_none());
    assert_eq!(missing_current.source, EmissionSource::ExactStore);
    assert_eq!(missing_current.abstention, Some(AbstentionKind::Absent));
    assert!(!missing_current.selected_address_match);
    assert!(missing_current.formed_query.is_none());
    assert!(!missing_current.query_matches_true);
    assert!(!missing_current.sieve_address_match());
    assert!(!missing_current.sieve_selected_address_match());

    let absent_ok = score_store_read(absent, Arm::TrueAddress, AddressReadOutcome::Absent, None)?;
    assert!(absent_ok.correct());
    assert!(absent_ok.typed_abstention_pass);
    assert!(absent_ok.emitted_text.is_none());
    assert_eq!(absent_ok.source, EmissionSource::ExactStore);

    let evicted_ok = score_store_read(
        evicted,
        Arm::TrueAddress,
        AddressReadOutcome::Evicted {
            address: evicted.true_address()?,
            evicted_at_sequence: 1,
        },
        None,
    )?;
    assert!(evicted_ok.correct());
    assert!(evicted_ok.typed_abstention_pass);

    let wrong_kind = score_store_read(evicted, Arm::TrueAddress, AddressReadOutcome::Absent, None)?;
    assert!(!wrong_kind.correct());
    assert!(!wrong_kind.wrong_substitution);

    let hit_on_abstention = score_store_read(
        absent,
        Arm::TrueAddress,
        AddressReadOutcome::Hit {
            address: absent.true_address()?,
            version: 1,
            sequence: 1,
            object_version: [0; 32],
            bytes: b"wrongvalue".to_vec(),
            whole_object_verified: true,
        },
        None,
    )?;
    assert!(hit_on_abstention.wrong_substitution);
    assert!(!hit_on_abstention.correct());
    assert!(hit_on_abstention.whole_object_verified);
    assert!(hit_on_abstention.selected_address_match);

    assert!(score_store_read(current, Arm::NoMemory, AddressReadOutcome::Absent, None).is_err());
    let model = score_model_arm(current, Arm::NoMemory, Some(expected_text(current)?))?;
    assert!(model.correct());
    assert_eq!(model.source, EmissionSource::Model);
    assert!(score_model_arm(current, Arm::TrueAddress, Some("x")).is_err());

    let fallback = store.observe_store_arm(previous, Arm::Sieve)?;
    assert_eq!(fallback.source, EmissionSource::SieveStore);
    assert_eq!(fallback.formed_query, Some(previous.address.clone()));
    assert!(fallback.query_matches_true);
    assert!(fallback.selected_address_match);
    assert!(!fallback.value_check_pass);
    assert!(fallback.sieve_current_fallback);
    assert!(fallback.sieve_address_match());
    assert!(fallback.sieve_query_address_match());
    assert!(fallback.sieve_selected_address_match());
    assert!(!fallback.sieve_value_check_pass());
    assert!(!fallback.correct());

    let random = store.observe_store_arm(current, Arm::RandomAddress)?;
    assert_eq!(random.source, EmissionSource::ExactStore);
    assert!(random.random_binding_pass);
    assert!(!random.value_check_pass);
    assert!(random.wrong_substitution);
    assert!(random.whole_object_verified);
    assert!(random.selected_address_match);
    assert!(random.formed_query.is_none());
    assert!(!random.query_matches_true);
    assert!(!random.sieve_address_match());
    assert!(!random.sieve_selected_address_match());

    let json = serde_json::to_value(&random)?;
    let back: ArmRecord = serde_json::from_value(json.clone())?;
    assert_eq!(back, random);
    let mut unknown = json;
    unknown["unexpected"] = serde_json::json!(1);
    assert!(serde_json::from_value::<ArmRecord>(unknown).is_err());
    Ok(())
}

#[test]
fn sieve_query_formation_is_separate_from_the_selected_slot() -> TestResult {
    let (tokenizer, panel) = fixture()?;
    let store = OracleStore::build_with_tokenizer(&panel, &tokenizer)?;

    let mut current_match = None;
    let mut current_distractor = None;
    for row in panel
        .rows
        .iter()
        .filter(|row| row.family == RowFamily::Current)
    {
        if form_sieve_query(&panel.lexicon, &row.prompt)? == row.true_address()? {
            current_match = Some(row);
        } else {
            current_distractor = Some(row);
        }
        if current_match.is_some() && current_distractor.is_some() {
            break;
        }
    }
    let current_match = current_match.ok_or("no current match row")?;
    let current_distractor = current_distractor.ok_or("no current distractor row")?;
    let absent = panel
        .rows
        .iter()
        .find(|row| row.expected_abstention() == Some(AbstentionKind::Absent))
        .ok_or("no absent row")?;
    let evicted = panel
        .rows
        .iter()
        .find(|row| row.expected_abstention() == Some(AbstentionKind::Evicted))
        .ok_or("no evicted row")?;

    let match_record = store.observe_store_arm(current_match, Arm::Sieve)?;
    assert_eq!(
        match_record.formed_query,
        Some(current_match.address.clone())
    );
    assert!(match_record.query_matches_true);
    assert!(match_record.selected_address_match);
    assert!(match_record.sieve_address_match());
    assert!(match_record.sieve_query_address_match());
    assert!(match_record.sieve_selected_address_match());

    let distractor_record = store.observe_store_arm(current_distractor, Arm::Sieve)?;
    assert_ne!(
        distractor_record.formed_query,
        Some(current_distractor.address.clone())
    );
    assert!(!distractor_record.query_matches_true);
    assert!(!distractor_record.selected_address_match);
    assert!(!distractor_record.sieve_address_match());
    assert!(!distractor_record.sieve_query_address_match());
    assert!(!distractor_record.sieve_selected_address_match());

    // A formed query can equal the true address even when the read returns a
    // typed abstention: query formation is still correct, no slot was selected.
    assert!(score_store_read(absent, Arm::Sieve, AddressReadOutcome::Absent, None).is_err());
    let absent_missing = score_store_read(
        absent,
        Arm::Sieve,
        AddressReadOutcome::Absent,
        Some(&panel.lexicon),
    )?;
    assert!(absent_missing.query_matches_true);
    assert_eq!(absent_missing.formed_query, Some(absent.address.clone()));
    assert!(!absent_missing.selected_address_match);
    assert!(absent_missing.sieve_address_match());
    assert!(absent_missing.sieve_query_address_match());
    assert!(!absent_missing.sieve_selected_address_match());
    assert!(absent_missing.typed_abstention_pass);

    let evicted_missing = score_store_read(
        evicted,
        Arm::Sieve,
        AddressReadOutcome::Evicted {
            address: evicted.true_address()?,
            evicted_at_sequence: 1,
        },
        Some(&panel.lexicon),
    )?;
    assert!(evicted_missing.query_matches_true);
    assert_eq!(evicted_missing.formed_query, Some(evicted.address.clone()));
    assert!(!evicted_missing.selected_address_match);
    assert!(evicted_missing.sieve_address_match());
    assert!(!evicted_missing.sieve_selected_address_match());
    assert!(evicted_missing.typed_abstention_pass);

    // A true formed query may still select a different present key.
    let absent_random = absent.random_address()?;
    let hit_elsewhere = score_store_read(
        absent,
        Arm::Sieve,
        AddressReadOutcome::Hit {
            address: absent_random,
            version: 1,
            sequence: 1,
            object_version: [0; 32],
            bytes: absent.random.value.text.as_bytes().to_vec(),
            whole_object_verified: true,
        },
        Some(&panel.lexicon),
    )?;
    assert!(hit_elsewhere.query_matches_true);
    assert!(!hit_elsewhere.selected_address_match);
    assert!(hit_elsewhere.sieve_address_match());
    assert!(!hit_elsewhere.sieve_selected_address_match());

    // Conversely, a wrong formed query may select the true address. The two
    // booleans must not collapse into one another.
    let distractor_true = current_distractor.true_address()?;
    let hit_true = score_store_read(
        current_distractor,
        Arm::Sieve,
        AddressReadOutcome::Hit {
            address: distractor_true,
            version: 1,
            sequence: 1,
            object_version: [0; 32],
            bytes: expected_text(current_distractor)?.as_bytes().to_vec(),
            whole_object_verified: true,
        },
        Some(&panel.lexicon),
    )?;
    assert!(!hit_true.query_matches_true);
    assert!(hit_true.selected_address_match);
    assert!(!hit_true.sieve_address_match());
    assert!(hit_true.sieve_selected_address_match());
    assert!(hit_true.value_check_pass);
    Ok(())
}

#[test]
fn hard_gate_requires_exact_40_20_20_and_10_10() -> TestResult {
    let (tokenizer, panel) = fixture()?;
    let store = OracleStore::build_with_tokenizer(&panel, &tokenizer)?;
    let records = arm_records(&panel, &store)?;

    assert!(baseline_records_identical(
        &records.no_memory,
        &records.no_address
    )?);
    let mut tampered_baseline = records.no_address.clone();
    tampered_baseline[0].emitted_text = Some("never".into());
    assert!(!baseline_records_identical(
        &records.no_memory,
        &tampered_baseline
    )?);
    assert!(baseline_records_identical(
        &records.no_memory,
        &records.no_address[..records.no_address.len() - 1]
    )
    .is_err());
    let mut wrong_baseline_arm = records.no_memory.clone();
    wrong_baseline_arm[0].arm = Arm::Sieve;
    assert!(baseline_records_identical(&wrong_baseline_arm, &records.no_address).is_err());

    let gate = hard_gates(&panel, &records.true_records)?;
    assert!(gate.passed());

    let evicted_index = panel
        .rows
        .iter()
        .position(|row| row.expected_abstention() == Some(AbstentionKind::Evicted))
        .ok_or("missing evicted row")?;
    let wrong_address = PrimeAddress::new(&[101, 103])?;
    let wrong_eviction = score_store_read(
        &panel.rows[evicted_index],
        Arm::TrueAddress,
        AddressReadOutcome::Evicted {
            address: wrong_address.clone(),
            evicted_at_sequence: 1,
        },
        None,
    )?;
    assert_eq!(
        wrong_eviction.address,
        Some(AddressSpec::from(&wrong_address))
    );
    let mut wrong_evicted = records.true_records.clone();
    wrong_evicted[evicted_index] = wrong_eviction;
    assert!(!hard_gates(&panel, &wrong_evicted)?.passed());

    assert_eq!(gate.rows, PANEL_ROWS as u64);
    assert_eq!(gate.current_pass, CURRENT_ROWS as u64);
    assert_eq!(gate.current_total, CURRENT_ROWS as u64);
    assert_eq!(gate.previous_pass, PREVIOUS_ROWS as u64);
    assert_eq!(gate.previous_total, PREVIOUS_ROWS as u64);
    assert_eq!(gate.abstention_pass, ABSTENTION_ROWS as u64);
    assert_eq!(gate.abstention_total, ABSTENTION_ROWS as u64);
    assert_eq!(gate.absent_pass, ABSENT_ROWS as u64);
    assert_eq!(gate.absent_total, ABSENT_ROWS as u64);
    assert_eq!(gate.evicted_pass, EVICTED_ROWS as u64);
    assert_eq!(gate.evicted_total, EVICTED_ROWS as u64);
    assert_eq!(gate.wrong_substitutions, 0);

    // Every abstention half must be exactly 10/10, not merely sum to 20.
    let mut bad = gate;
    bad.absent_pass = 9;
    assert!(!bad.passed());
    let mut bad = gate;
    bad.absent_total = 9;
    assert!(!bad.passed());
    let mut bad = gate;
    bad.evicted_pass = 9;
    assert!(!bad.passed());
    let mut bad = gate;
    bad.evicted_total = 9;
    assert!(!bad.passed());
    let mut bad = gate;
    bad.absent_pass = 9;
    bad.evicted_pass = 11;
    assert!(!bad.passed());
    let mut bad = gate;
    bad.absent_total = 11;
    assert!(!bad.passed());

    let mut degraded = records.true_records.clone();
    degraded[0].value_check_pass = false;
    degraded[0].wrong_substitution = true;
    let report = hard_gates(&panel, &degraded)?;
    assert!(!report.passed());
    assert_eq!(report.current_pass, CURRENT_ROWS as u64 - 1);
    assert_eq!(report.wrong_substitutions, 1);

    let mut missing = records.true_records.clone();
    missing.pop();
    assert!(hard_gates(&panel, &missing).is_err());
    let mut duplicate = records.true_records.clone();
    duplicate[1].row_id = duplicate[0].row_id.clone();
    assert!(hard_gates(&panel, &duplicate).is_err());
    let mut wrong_arm = records.true_records.clone();
    wrong_arm[0].arm = Arm::Sieve;
    assert!(hard_gates(&panel, &wrong_arm).is_err());
    Ok(())
}

#[test]
fn contrasts_are_single_family_and_previous_sieve_is_rejected() -> TestResult {
    let (tokenizer, panel) = fixture()?;
    let store = OracleStore::build_with_tokenizer(&panel, &tokenizer)?;
    let records = arm_records(&panel, &store)?;
    let gate = hard_gates(&panel, &records.true_records)?;
    assert!(gate.passed());

    for (population, family, expected_rows) in [
        (
            ContrastPopulation::Current,
            RowFamily::Current,
            CURRENT_ROWS,
        ),
        (
            ContrastPopulation::Previous,
            RowFamily::Previous,
            PREVIOUS_ROWS,
        ),
        (
            ContrastPopulation::Abstention,
            RowFamily::Abstention,
            ABSTENTION_ROWS,
        ),
    ] {
        let contrast = true_vs_no_memory(
            &panel,
            &records.true_records,
            &records.no_memory,
            population,
        )?;
        let (rows, b, c) = expected_wins(&panel, family, &records.true_records, &records.no_memory);
        assert_eq!(contrast.rows, expected_rows as u64);
        assert_eq!(contrast.rows, rows);
        assert_eq!(contrast.b, b);
        assert_eq!(contrast.c, c);
    }

    for (population, family, expected_rows) in [
        (
            ContrastPopulation::Current,
            RowFamily::Current,
            CURRENT_ROWS,
        ),
        (
            ContrastPopulation::Abstention,
            RowFamily::Abstention,
            ABSTENTION_ROWS,
        ),
    ] {
        let contrast = true_vs_sieve(
            &panel,
            &records.true_records,
            &records.sieve_records,
            population,
        )?;
        let (rows, b, c) = expected_wins(
            &panel,
            family,
            &records.true_records,
            &records.sieve_records,
        );
        assert_eq!(contrast.rows, expected_rows as u64);
        assert_eq!(contrast.rows, rows);
        assert_eq!(contrast.b, b);
        assert_eq!(contrast.c, c);
    }
    assert!(true_vs_sieve(
        &panel,
        &records.true_records,
        &records.sieve_records,
        ContrastPopulation::Previous
    )
    .is_err());

    for (population, family, expected_rows) in [
        (
            ContrastPopulation::Current,
            RowFamily::Current,
            CURRENT_ROWS,
        ),
        (
            ContrastPopulation::Abstention,
            RowFamily::Abstention,
            ABSTENTION_ROWS,
        ),
    ] {
        let contrast = sieve_vs_no_memory(
            &panel,
            &records.sieve_records,
            &records.no_memory,
            population,
        )?;
        let (rows, b, c) =
            expected_wins(&panel, family, &records.sieve_records, &records.no_memory);
        assert_eq!(contrast.rows, expected_rows as u64);
        assert_eq!(contrast.rows, rows);
        assert_eq!(contrast.b, b);
        assert_eq!(contrast.c, c);
    }
    assert!(sieve_vs_no_memory(
        &panel,
        &records.sieve_records,
        &records.no_memory,
        ContrastPopulation::Previous
    )
    .is_err());

    // The generic paired function can count the 20 previous true-vs-sieve
    // wins, but no named sieve contrast accepts Previous. A pooled result
    // would include them; the per-family results below cannot.
    let previous_generic = paired_contrast(
        &panel,
        &records.true_records,
        &records.sieve_records,
        ContrastPopulation::Previous,
    )?;
    assert_eq!(previous_generic.rows, PREVIOUS_ROWS as u64);
    assert_eq!(previous_generic.b, PREVIOUS_ROWS as u64);
    assert_eq!(previous_generic.c, 0);
    let current_sieve = true_vs_sieve(
        &panel,
        &records.true_records,
        &records.sieve_records,
        ContrastPopulation::Current,
    )?;
    let abstention_sieve = true_vs_sieve(
        &panel,
        &records.true_records,
        &records.sieve_records,
        ContrastPopulation::Abstention,
    )?;
    let hypothetical_pooled_b = current_sieve.b + previous_generic.b + abstention_sieve.b;
    assert!(current_sieve.b < hypothetical_pooled_b);
    assert!(abstention_sieve.b < hypothetical_pooled_b);

    for (population, encoded) in [
        (ContrastPopulation::Current, "\"Current\""),
        (ContrastPopulation::Previous, "\"Previous\""),
        (ContrastPopulation::Abstention, "\"Abstention\""),
    ] {
        assert_eq!(serde_json::to_string(&population)?, encoded);
        assert_eq!(
            serde_json::from_str::<ContrastPopulation>(encoded)?,
            population
        );
    }
    assert!(serde_json::from_str::<ContrastPopulation>("\"AllRows\"").is_err());
    assert!(serde_json::from_str::<ContrastPopulation>("\"CurrentAndAbstention\"").is_err());

    assert!(true_vs_sieve(
        &panel,
        &records.true_records,
        &records.no_memory,
        ContrastPopulation::Current
    )
    .is_err());
    assert!(true_vs_sieve(
        &panel,
        &records.true_records[..records.true_records.len() - 1],
        &records.sieve_records,
        ContrastPopulation::Current
    )
    .is_err());
    let mut swapped = records.sieve_records.clone();
    swapped.swap(0, 1);
    assert!(true_vs_sieve(
        &panel,
        &records.true_records,
        &swapped,
        ContrastPopulation::Current
    )
    .is_err());
    Ok(())
}

#[test]
fn composed_gate_requires_every_control() -> TestResult {
    let (tokenizer, panel) = fixture()?;
    let store = OracleStore::build_with_tokenizer(&panel, &tokenizer)?;
    let records = arm_records(&panel, &store)?;
    let gate = hard_gates(&panel, &records.true_records)?;
    assert!(gate.passed());

    let baseline = baseline_records_identical(&records.no_memory, &records.no_address)?;
    let random = random_control_holds(&panel, &records.random_records)?;
    let leak_free =
        prompt_value_leaks(&panel).is_empty() && corpus_value_leaks(&panel, &[]).is_empty();
    let passing = composed_gate(gate, baseline, random, true, leak_free, true);
    assert!(passing.passed());

    assert!(!composed_gate(gate, false, true, true, true, true).passed());
    assert!(!composed_gate(gate, true, false, true, true, true).passed());
    assert!(!composed_gate(gate, true, true, false, true, true).passed());
    assert!(!composed_gate(gate, true, true, true, false, true).passed());
    assert!(!composed_gate(gate, true, true, true, true, false).passed());

    // Hard gates alone must not pass: every other control defaults to false.
    let hard_only = ComposedGateReport {
        hard_gates: gate,
        ..Default::default()
    };
    assert!(!hard_only.passed());
    assert!(!ComposedGateReport::default().passed());

    let mut bad_gate = gate;
    bad_gate.absent_pass = 9;
    assert!(!composed_gate(bad_gate, true, true, true, true, true).passed());

    let mut tampered_random = records.random_records.clone();
    tampered_random[0].emitted_text = records.true_records[0].emitted_text.clone();
    tampered_random[0].random_binding_pass = false;
    tampered_random[0].value_check_pass = true;
    assert!(!random_control_holds(&panel, &tampered_random)?);

    let encoded = serde_json::to_value(&passing)?;
    let back: ComposedGateReport = serde_json::from_value(encoded.clone())?;
    assert_eq!(back, passing);
    let mut missing = encoded;
    missing
        .as_object_mut()
        .ok_or("composed gate is not an object")?
        .remove("parity_pass");
    assert!(serde_json::from_value::<ComposedGateReport>(missing).is_err());
    Ok(())
}

#[test]
fn family_outcome_classifier_has_failure_ceiling_gain_and_no_gain_branches() -> TestResult {
    let (tokenizer, panel) = fixture()?;
    let store = OracleStore::build_with_tokenizer(&panel, &tokenizer)?;
    let records = arm_records(&panel, &store)?;
    let gate = hard_gates(&panel, &records.true_records)?;
    assert!(gate.passed());

    let ceiling = classify_family(
        &panel,
        ContrastPopulation::Current,
        &gate,
        &records.true_records,
        &records.no_memory,
    )?;
    assert_eq!(ceiling.family, ContrastPopulation::Current);
    assert_eq!(ceiling.class, OutcomeClass::Ceiling);
    assert!(ceiling.control_exact);
    assert!(ceiling.hard_gates_pass);
    assert_eq!(ceiling.contrast.rows, CURRENT_ROWS as u64);

    let previous_ceiling = classify_family(
        &panel,
        ContrastPopulation::Previous,
        &gate,
        &records.true_records,
        &records.no_memory,
    )?;
    assert_eq!(previous_ceiling.class, OutcomeClass::Ceiling);
    assert!(previous_ceiling.control_exact);
    assert_eq!(previous_ceiling.contrast.rows, PREVIOUS_ROWS as u64);
    assert!(classify_family(
        &panel,
        ContrastPopulation::Previous,
        &gate,
        &records.true_records,
        &records.sieve_records,
    )
    .is_err());

    let gain = classify_family(
        &panel,
        ContrastPopulation::Current,
        &gate,
        &records.true_records,
        &records.sieve_records,
    )?;
    assert_eq!(gain.class, OutcomeClass::Gain);
    assert!(!gain.control_exact);
    assert!(gain.contrast.d() > 0);
    assert_eq!(gain.contrast.rows, CURRENT_ROWS as u64);

    let abstention_gain = classify_family(
        &panel,
        ContrastPopulation::Abstention,
        &gate,
        &records.true_records,
        &records.no_memory,
    )?;
    assert_eq!(abstention_gain.class, OutcomeClass::Gain);
    assert!(!abstention_gain.control_exact);
    assert_eq!(abstention_gain.contrast.d(), ABSTENTION_ROWS as i64);
    assert_eq!(abstention_gain.contrast.rows, ABSTENTION_ROWS as u64);

    // A partially correct control is not "exact"; a non-positive treatment
    // difference is an explicit no-gain, with no training recommendation.
    let mut partial = records.no_memory.clone();
    for (row, record) in panel.rows.iter().zip(&mut partial) {
        if row.family == RowFamily::Current {
            record.value_check_pass = false;
            record.wrong_substitution = true;
            break;
        }
    }
    let no_gain = classify_family(
        &panel,
        ContrastPopulation::Current,
        &gate,
        &records.sieve_records,
        &partial,
    )?;
    assert_eq!(no_gain.class, OutcomeClass::NoGain);
    assert!(!no_gain.control_exact);
    assert!(no_gain.contrast.d() <= 0);
    assert_eq!(no_gain.contrast.rows, CURRENT_ROWS as u64);

    let mut bad_gate = gate;
    bad_gate.absent_pass = 9;
    let failure = classify_family(
        &panel,
        ContrastPopulation::Current,
        &bad_gate,
        &records.true_records,
        &records.sieve_records,
    )?;
    assert_eq!(failure.class, OutcomeClass::InstrumentFailure);
    assert!(!failure.hard_gates_pass);

    for (class, encoded) in [
        (OutcomeClass::InstrumentFailure, "\"InstrumentFailure\""),
        (OutcomeClass::Ceiling, "\"Ceiling\""),
        (OutcomeClass::Gain, "\"Gain\""),
        (OutcomeClass::NoGain, "\"NoGain\""),
    ] {
        assert_eq!(serde_json::to_string(&class)?, encoded);
        assert_eq!(serde_json::from_str::<OutcomeClass>(encoded)?, class);
    }
    let encoded = serde_json::to_value(&no_gain)?;
    let back: FamilyOutcome = serde_json::from_value(encoded)?;
    assert_eq!(back, no_gain);

    assert!(classify_family(
        &panel,
        ContrastPopulation::Current,
        &gate,
        &records.true_records[..records.true_records.len() - 1],
        &records.sieve_records,
    )
    .is_err());
    let mut swapped = records.sieve_records.clone();
    swapped.swap(0, 1);
    assert!(classify_family(
        &panel,
        ContrastPopulation::Current,
        &gate,
        &records.true_records,
        &swapped,
    )
    .is_err());
    Ok(())
}

#[test]
fn panel_limits_are_bounded_and_manifest_parse_is_bounded() -> TestResult {
    let (_tokenizer, panel) = fixture()?;

    let mut p = panel.clone();
    p.limits.max_rows = 0;
    assert_invalid(&p);
    p.limits.max_rows = HARD_MAX_ROWS + 1;
    assert_invalid(&p);
    p.limits.max_rows = PANEL_ROWS - 1;
    assert_invalid(&p);

    let mut p = panel.clone();
    p.limits.max_slots = panel.limits.max_slots + 1;
    assert_invalid(&p);
    p.limits.max_slots = panel.limits.max_slots - 1;
    assert_invalid(&p);

    let mut p = panel.clone();
    p.limits.max_key_atoms = 1;
    assert_invalid(&p);
    p.limits.max_key_atoms = HARD_MAX_KEY_ATOMS + 1;
    assert_invalid(&p);

    let mut p = panel.clone();
    p.limits.max_value_tokens = 1;
    assert_invalid(&p);
    p.limits.max_value_tokens = HARD_MAX_VALUE_TOKENS + 1;
    assert_invalid(&p);

    let mut p = panel.clone();
    p.limits.max_value_bytes = 1;
    assert_invalid(&p);
    p.limits.max_value_bytes = HARD_MAX_VALUE_BYTES + 1;
    assert_invalid(&p);

    let mut p = panel.clone();
    p.limits.max_prompt_tokens = 1;
    assert_invalid(&p);
    p.limits.max_prompt_tokens = HARD_MAX_PROMPT_TOKENS + 1;
    assert_invalid(&p);

    let mut p = panel.clone();
    p.limits.max_prompt_bytes = 1;
    assert_invalid(&p);
    p.limits.max_prompt_bytes = HARD_MAX_PROMPT_BYTES + 1;
    assert_invalid(&p);

    let mut p = panel.clone();
    p.rows[0].address.primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23];
    assert_invalid(&p);

    let mut p = panel.clone();
    p.rows[0].history[0].text = "a".repeat(p.limits.max_value_bytes + 1);
    assert_invalid(&p);

    let mut p = panel.clone();
    p.rows[0].prompt = "a".repeat(p.limits.max_prompt_bytes + 1);
    assert_invalid(&p);

    let mut p = panel.clone();
    p.rows[0].id = "bad id!".into();
    assert_invalid(&p);

    let mut p = panel.clone();
    p.rows[1].address = p.rows[0].address.clone();
    assert_invalid(&p);
    let mut p = panel.clone();
    p.rows[1].random.address = p.rows[0].random.address.clone();
    assert_invalid(&p);
    let mut p = panel.clone();
    p.rows[1].random.address = p.rows[0].address.clone();
    assert_invalid(&p);

    let mut p = panel.clone();
    p.rows[1].random.value = p.rows[0].random.value.clone();
    assert_invalid(&p);
    let mut p = panel.clone();
    p.rows[1].random.value = p.rows[0].history[0].clone();
    assert_invalid(&p);

    let evicted_index = panel
        .rows
        .iter()
        .position(|row| row.expected_abstention() == Some(AbstentionKind::Evicted))
        .ok_or("no evicted row")?;
    let mut p = panel.clone();
    p.rows[evicted_index].expected = ExpectedOutcome::Abstention(AbstentionKind::Absent);
    p.rows[evicted_index].history.clear();
    assert_invalid(&p);

    let previous_index = panel
        .rows
        .iter()
        .position(|row| row.family == RowFamily::Previous)
        .ok_or("no previous row")?;
    let mut p = panel.clone();
    p.rows[previous_index].history[1] = p.rows[previous_index].history[0].clone();
    assert_invalid(&p);

    let mut p = panel.clone();
    p.rows[0].version = RequestedVersion::Previous;
    assert_invalid(&p);
    let mut p = panel.clone();
    p.rows[0].expected = ExpectedOutcome::Abstention(AbstentionKind::Absent);
    assert_invalid(&p);
    let mut p = panel.clone();
    p.rows[0].history.clear();
    assert_invalid(&p);
    let mut p = panel.clone();
    p.rows[0].prompt.clear();
    assert_invalid(&p);
    let mut p = panel.clone();
    p.rows[0].prompt_tokens.clear();
    assert_invalid(&p);
    let mut p = panel.clone();
    p.rows[0].random.value.text.clear();
    p.rows[0].random.value.tokens.clear();
    assert_invalid(&p);

    let huge = vec![b' '; HARD_MAX_PANEL_BYTES + 1];
    assert!(parse_panel(&huge).is_err());
    Ok(())
}

#[test]
fn cross_row_and_corpus_value_leaks_are_detected() -> TestResult {
    let (_tokenizer, panel) = fixture()?;
    assert!(prompt_value_leaks(&panel).is_empty());
    assert!(corpus_value_leaks(&panel, &[9999]).is_empty());

    let leaked_value = panel.rows[1]
        .expected_value()
        .ok_or("no expected value")?
        .text
        .clone();
    let mut leaked = panel.clone();
    leaked.rows[0].prompt.push_str(&leaked_value);
    assert!(prompt_value_leaks(&leaked).contains(&leaked.rows[1].id));
    assert_invalid(&leaked);

    let value = panel.rows[2].expected_value().ok_or("no expected value")?;
    let mut corpus = vec![9999u32];
    corpus.extend(value.tokens.iter().copied());
    let leaks = corpus_value_leaks(&panel, &corpus);
    assert!(leaks.contains(&panel.rows[2].id));
    assert!(corpus_value_leaks(&panel, &[9999u32]).is_empty());
    Ok(())
}

#[test]
fn production_manifest_is_pinned_and_structurally_valid() -> TestResult {
    let bytes = include_bytes!("../../../docs/labs/m1-address-as-read/address-oracle.json");
    let panel = PanelManifest::parse(bytes)?;
    assert_eq!(
        panel.sha256()?,
        "a4aa548f3e38478fdc8cd4ad077d9886cefa3aaa8e2712abe8f38abe0dba4103"
    );
    panel.validate(None)?;
    assert_eq!(panel.rows.len(), PANEL_ROWS);
    Ok(())
}

#[test]
fn emission_token_count_uses_actual_store_and_model_emissions() -> TestResult {
    let (tokenizer, panel) = fixture()?;
    let store = OracleStore::build_with_tokenizer(&panel, &tokenizer)?;
    let row = &panel.rows[0];
    let mut random = store.observe_store_arm(row, Arm::RandomAddress)?;
    random.emitted_text = Some("a".into());
    let expected_len = row
        .expected_value()
        .ok_or("missing expected value")?
        .tokens
        .len();
    let actual = emission_token_count(Some(&tokenizer), &random, 0)?;
    assert_eq!(actual, tokenizer.encode("a").len());
    assert_ne!(actual, expected_len);
    let model = score_model_arm(row, Arm::NoMemory, Some("zzzzzzzz"))?;
    assert_eq!(emission_token_count(Some(&tokenizer), &model, 7)?, 7);
    random.emitted_text = None;
    assert_eq!(emission_token_count(Some(&tokenizer), &random, 0)?, 0);
    Ok(())
}

#[test]
fn invalid_model_emission_utf8_is_marked_not_fatal() {
    let (text, valid) = decode_emission_bytes(&[0xff]);
    assert!(!valid);
    assert!(text.contains(char::REPLACEMENT_CHARACTER));
    let (text, valid) = decode_emission_bytes(b"abcdefgh");
    assert!(valid);
    assert_eq!(text, "abcdefgh");
}
