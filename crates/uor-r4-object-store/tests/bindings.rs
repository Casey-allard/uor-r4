//! Local source-byte tokenization and metadata controls only; no model work.
use std::error::Error;
use uor_r4_object_store::{
    bindings::{ingest, Bindings, PinnedTokenizer},
    Error as StoreError, Limits, Store,
};
use uor_r4_tokenizer::ByteBpeTokenizer;

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
const SOURCE: &str = r#"{
    "pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},
    "added_tokens":[{"id":8,"content":"<end>"},{"id":12,"content":"\u0000"}],
    "model":{"type":"BPE","vocab":{"a":0,"b":1,"c":2,"ab":3,"bc":4,"Ã":5,"©":6,"Ġ":7},"merges":["b c","a b"]}
}"#;
fn tokenizer() -> TestResult<ByteBpeTokenizer> {
    ByteBpeTokenizer::from_tokenizer_json_bytes(SOURCE.as_bytes())
        .ok_or_else(|| "fixture tokenizer invalid".into())
}

#[test]
fn distinct_occurrences_survive_binding_and_record_serialization() -> TestResult {
    let tokenizer = tokenizer()?;
    let pinned = PinnedTokenizer::new(&tokenizer)?;
    let mut store = Store::new(Limits::default());
    let original = b"abc<end>abc\0";
    let ingested = ingest(&mut store, &pinned, original)?;
    assert_eq!(store.rebuild(&ingested.manifest)?.bytes(), original);
    let entries = ingested.bindings.occurrences();
    assert_eq!(entries.len(), 6);
    assert_eq!(entries[0].token_id(), entries[3].token_id());
    assert_ne!(entries[0].index(), entries[3].index());
    assert_ne!(entries[0].byte_range(), entries[3].byte_range());
    assert_eq!(
        ingested.bindings.object_version(),
        ingested.manifest.version()
    );
    assert_eq!(ingested.bindings.tokenizer_identity(), pinned.identity());
    let encoded = ingested.bindings.encode(store.limits())?;
    let restored = Bindings::decode(&encoded, store.limits())?;
    assert_eq!(restored, ingested.bindings);
    assert_eq!(restored.encode(store.limits())?, encoded);
    let mut receiver = Store::new(Limits::default());
    receiver.import_records(&store.export_records()?)?;
    let mut joined = Vec::new();
    for entry in restored.occurrences() {
        let result =
            restored.read_occurrence(&receiver, &ingested.manifest, &pinned, entry.index())?;
        assert!(!result.whole_object_digest_verified());
        joined.extend_from_slice(result.bytes());
    }
    assert_eq!(joined, original);
    Ok(())
}

#[test]
fn byte_spans_preserve_split_utf8_and_empty_input() -> TestResult {
    let tokenizer = tokenizer()?;
    let pinned = PinnedTokenizer::new(&tokenizer)?;
    let mut store = Store::new(Limits::default());
    let ingested = ingest(&mut store, &pinned, "é".as_bytes())?;
    assert_eq!(ingested.bindings.occurrences().len(), 2);
    for index in 0..2 {
        let output =
            ingested
                .bindings
                .read_occurrence(&store, &ingested.manifest, &pinned, index)?;
        assert_eq!(output.bytes().len(), 1);
        assert!(matches!(output.text(), Err(StoreError::Utf8)));
    }
    assert_eq!(store.rebuild(&ingested.manifest)?.bytes(), "é".as_bytes());
    let empty = ingest(&mut store, &pinned, b"")?;
    assert!(empty.bindings.occurrences().is_empty());
    let encoded = empty.bindings.encode(store.limits())?;
    assert_eq!(encoded.len(), 88);
    assert_eq!(Bindings::decode(&encoded, store.limits())?, empty.bindings);
    assert!(matches!(
        empty
            .bindings
            .read_occurrence(&store, &empty.manifest, &pinned, 0),
        Err(StoreError::InvalidRange)
    ));
    Ok(())
}

#[test]
fn normalization_invalid_utf8_and_unsupported_input_are_refused_before_fold() -> TestResult {
    let prefixed = SOURCE.replace("\"add_prefix_space\":false", "\"add_prefix_space\":true");
    let tokenizer_prefix = ByteBpeTokenizer::from_tokenizer_json_bytes(prefixed.as_bytes())
        .ok_or("invalid prefix fixture")?;
    assert!(matches!(
        PinnedTokenizer::new(&tokenizer_prefix),
        Err(StoreError::BindingMismatch)
    ));
    let tokenizer = tokenizer()?;
    let pinned = PinnedTokenizer::new(&tokenizer)?;
    let mut store = Store::new(Limits::default());
    assert!(matches!(
        ingest(&mut store, &pinned, &[0xff]),
        Err(StoreError::Utf8)
    ));
    // The shared permissive encoder drops missing vocabulary bytes. The adapter
    // must detect the loss rather than assign guessed offsets or fold the input.
    assert!(matches!(
        ingest(&mut store, &pinned, b"xyz"),
        Err(StoreError::BindingMismatch)
    ));
    assert!(store.export_records()?.is_empty());
    Ok(())
}

#[test]
fn object_version_and_tokenizer_identity_are_checked_on_every_occurrence_read() -> TestResult {
    let tokenizer = tokenizer()?;
    let pinned = PinnedTokenizer::new(&tokenizer)?;
    let mut store = Store::new(Limits::default());
    let a = ingest(&mut store, &pinned, b"abc")?;
    let b = ingest(&mut store, &pinned, b"abca")?;
    assert!(matches!(
        a.bindings.read_occurrence(&store, &b.manifest, &pinned, 0),
        Err(StoreError::BindingMismatch)
    ));
    // Even semantically equivalent but differently pinned source JSON is another
    // tokenizer identity; source provenance must not silently alias.
    let changed = format!("{SOURCE}\n");
    let other = ByteBpeTokenizer::from_tokenizer_json_bytes(changed.as_bytes())
        .ok_or("invalid alternate fixture")?;
    let other_pin = PinnedTokenizer::new(&other)?;
    assert!(matches!(
        a.bindings
            .read_occurrence(&store, &a.manifest, &other_pin, 0),
        Err(StoreError::BindingMismatch)
    ));
    assert!(matches!(
        a.bindings
            .read_occurrence(&store, &a.manifest, &pinned, u64::MAX),
        Err(StoreError::InvalidRange)
    ));
    Ok(())
}

#[test]
fn malformed_binding_encoding_and_impossible_limits_are_rejected() -> TestResult {
    let tokenizer = tokenizer()?;
    let pinned = PinnedTokenizer::new(&tokenizer)?;
    let mut store = Store::new(Limits::default());
    let ingested = ingest(&mut store, &pinned, b"abc")?;
    let original = ingested.bindings.encode(store.limits())?;
    for end in 0..original.len() {
        assert!(Bindings::decode(&original[..end], store.limits()).is_err());
    }
    let mut trailing = original.clone();
    trailing.push(0);
    assert!(Bindings::decode(&trailing, store.limits()).is_err());
    let mut wrong_schema = original.clone();
    wrong_schema[7] = b'2';
    assert!(matches!(
        Bindings::decode(&wrong_schema, store.limits()),
        Err(StoreError::SchemaMismatch)
    ));
    let mut huge_count = original.clone();
    huge_count[80..88].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(matches!(
        Bindings::decode(&huge_count, store.limits()),
        Err(StoreError::LimitExceeded(_))
    ));
    let mut bad_index = original.clone();
    bad_index[88..96].copy_from_slice(&1u64.to_le_bytes());
    assert!(matches!(
        Bindings::decode(&bad_index, store.limits()),
        Err(StoreError::BindingMismatch)
    ));
    let mut gap = original.clone();
    gap[100..108].copy_from_slice(&1u64.to_le_bytes());
    assert!(matches!(
        Bindings::decode(&gap, store.limits()),
        Err(StoreError::BindingMismatch)
    ));
    let limits = Limits {
        max_occurrences: 1,
        ..Limits::default()
    };
    assert!(matches!(
        Bindings::decode(&original, &limits),
        Err(StoreError::LimitExceeded(_))
    ));
    let mut bounded = Store::new(limits);
    assert!(matches!(
        ingest(&mut bounded, &pinned, b"abc"),
        Err(StoreError::LimitExceeded(_))
    ));
    assert!(bounded.export_records()?.is_empty());
    Ok(())
}

#[test]
fn unassigned_out_of_range_wrong_and_oversized_token_reads_are_refused() -> TestResult {
    let tokenizer = tokenizer()?;
    assert_eq!(tokenizer.token_byte_len(5), Some(1));
    assert_eq!(tokenizer.token_byte_len(8), Some(5));
    assert_eq!(tokenizer.token_byte_len(9), None);
    assert_eq!(tokenizer.token_byte_len(u32::MAX), None);
    assert!(!tokenizer.adds_prefix_space());
    let pinned = PinnedTokenizer::new(&tokenizer)?;
    let mut store = Store::new(Limits::default());
    let ingested = ingest(&mut store, &pinned, b"a")?;
    let original = ingested.bindings.encode(store.limits())?;
    for invalid in [9u32, u32::MAX, 1] {
        let mut wire = original.clone();
        wire[96..100].copy_from_slice(&invalid.to_le_bytes());
        let bindings = Bindings::decode(&wire, store.limits())?;
        assert!(matches!(
            bindings.read_occurrence(&store, &ingested.manifest, &pinned, 0),
            Err(StoreError::BindingMismatch)
        ));
    }
    let long = ingest(&mut store, &pinned, b"<end>")?;
    let mut bounded = Store::new(Limits {
        max_returned_bytes: 4,
        ..Limits::default()
    });
    bounded.import_records(&store.export_records()?)?;
    assert!(matches!(
        long.bindings
            .read_occurrence(&bounded, &long.manifest, &pinned, 0),
        Err(StoreError::LimitExceeded("decoded token bytes"))
    ));
    Ok(())
}

#[test]
fn ingestion_budget_and_quotation_budget_are_intentionally_independent() -> TestResult {
    let tokenizer = tokenizer()?;
    let pinned = PinnedTokenizer::new(&tokenizer)?;
    let mut store = Store::new(Limits {
        max_object_bytes: 5,
        max_returned_bytes: 4,
        ..Limits::default()
    });
    // A five-byte token is valid storage even if this reader returns at most four bytes.
    let stored = ingest(&mut store, &pinned, b"<end>")?;
    assert_eq!(store.range(&stored.manifest, 0..4)?.bytes(), b"<end");
    assert!(matches!(
        stored
            .bindings
            .read_occurrence(&store, &stored.manifest, &pinned, 0),
        Err(StoreError::LimitExceeded("decoded token bytes"))
    ));
    let before = store.export_records()?;
    assert!(matches!(
        ingest(&mut store, &pinned, b"<end>a"),
        Err(StoreError::LimitExceeded("object bytes"))
    ));
    assert_eq!(store.export_records()?, before);
    Ok(())
}
