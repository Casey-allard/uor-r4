//! Finite CPU controls for the address-as-read consumer over `uor-r4-object-store`.
use std::{
    error::Error,
    sync::atomic::{AtomicUsize, Ordering},
};
use uor_r4_object_store::{
    address_read::{
        AddressDirectory, AddressReadOutcome, ObjectTarget, PrimeAddress, VersionSelect,
    },
    bindings::{ingest, PinnedTokenizer},
    Error as StoreError, Limits, NodeId, Record, RecordBackend, Store,
};
use uor_r4_tokenizer::ByteBpeTokenizer;

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

const TOKENIZER_JSON: &str = r#"{
    "pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},
    "added_tokens":[{"id":8,"content":"<end>"}],
    "model":{"type":"BPE","vocab":{"a":0,"b":1,"c":2,"ab":3,"bc":4,"x":5,"y":6,"z":7},"merges":["b c","a b"]}
}"#;

fn fixture_tokenizer() -> TestResult<ByteBpeTokenizer> {
    ByteBpeTokenizer::from_tokenizer_json_bytes(TOKENIZER_JSON.as_bytes())
        .ok_or_else(|| "invalid fixture tokenizer".into())
}

#[test]
fn exact_hit_resolves_byte_span_whole_object_and_token_occurrence() -> TestResult {
    let tokenizer = fixture_tokenizer()?;
    let pinned = PinnedTokenizer::new(&tokenizer)?;
    let mut store = Store::new(Limits::default());
    let ingested = ingest(&mut store, &pinned, b"abc<end>xyz")?;

    let mut dir = AddressDirectory::new(8)?;
    let addr_span = PrimeAddress::with_pairs(&[3, 5], &[(3, 5)])?;
    let addr_whole = PrimeAddress::new(&[7, 11])?;
    let addr_tok = PrimeAddress::with_pairs(&[13, 17], &[(13, 17)])?;

    assert_eq!(addr_span.semiprime(), Some(15));
    assert_eq!(addr_whole.semiprime(), Some(77));

    dir.write(
        addr_span.clone(),
        ObjectTarget::ByteSpan {
            manifest: ingested.manifest.clone(),
            range: 0..3,
        },
    )?;
    dir.write(
        addr_whole.clone(),
        ObjectTarget::WholeObject {
            manifest: ingested.manifest.clone(),
        },
    )?;
    dir.write(
        addr_tok.clone(),
        ObjectTarget::TokenOccurrence {
            manifest: ingested.manifest.clone(),
            bindings: ingested.bindings.clone(),
            index: 2,
        },
    )?;

    match dir.read_exact(&store, Some(&pinned), &addr_span, VersionSelect::Current)? {
        AddressReadOutcome::Hit {
            address,
            version,
            bytes,
            whole_object_verified,
            ..
        } => {
            assert_eq!(address, addr_span);
            assert_eq!(version, 1);
            assert_eq!(bytes, b"abc");
            assert!(!whole_object_verified);
        }
        other => return Err(format!("expected Hit for span, got {other:?}").into()),
    }

    match dir.read_exact(&store, None, &addr_whole, VersionSelect::Current)? {
        AddressReadOutcome::Hit {
            address,
            version,
            bytes,
            whole_object_verified,
            ..
        } => {
            assert_eq!(address, addr_whole);
            assert_eq!(version, 1);
            assert_eq!(bytes, b"abc<end>xyz");
            assert!(whole_object_verified);
        }
        other => return Err(format!("expected Hit for whole object, got {other:?}").into()),
    }

    match dir.read_exact(&store, Some(&pinned), &addr_tok, VersionSelect::Current)? {
        AddressReadOutcome::Hit {
            address,
            version,
            bytes,
            whole_object_verified,
            ..
        } => {
            assert_eq!(address, addr_tok);
            assert_eq!(version, 1);
            assert_eq!(bytes, b"<end>");
            assert!(!whole_object_verified);
        }
        other => return Err(format!("expected Hit for token occurrence, got {other:?}").into()),
    }
    Ok(())
}

#[test]
fn exact_miss_returns_absent_without_touching_backend() -> TestResult {
    struct CountingBackend<'a> {
        inner: &'a Store,
        fetches: AtomicUsize,
    }
    impl RecordBackend for CountingBackend<'_> {
        fn fetch(&self, id: NodeId) -> uor_r4_object_store::Result<Option<Record>> {
            self.fetches.fetch_add(1, Ordering::SeqCst);
            self.inner.fetch(id)
        }
    }

    let mut store = Store::new(Limits::default());
    let folded = store.fold(b"stored fact payload")?;
    let mut dir = AddressDirectory::new(4)?;
    let written = PrimeAddress::new(&[2, 3])?;
    let unwritten = PrimeAddress::new(&[5, 7])?;
    dir.write(
        written.clone(),
        ObjectTarget::WholeObject {
            manifest: folded.manifest,
        },
    )?;

    let backend = CountingBackend {
        inner: &store,
        fetches: AtomicUsize::new(0),
    };
    let miss = dir.read_exact_from(
        &backend,
        store.limits(),
        None,
        &unwritten,
        VersionSelect::Current,
    )?;
    assert_eq!(miss, AddressReadOutcome::Absent);
    assert_eq!(backend.fetches.load(Ordering::SeqCst), 0);

    let sieve_miss = dir.read_sieve_from(&backend, store.limits(), None, &unwritten)?;
    assert_eq!(sieve_miss, AddressReadOutcome::Absent);
    assert_eq!(backend.fetches.load(Ordering::SeqCst), 0);

    let prev_miss = dir.read_exact_from(
        &backend,
        store.limits(),
        None,
        &written,
        VersionSelect::Previous,
    )?;
    assert_eq!(prev_miss, AddressReadOutcome::Absent);
    assert_eq!(backend.fetches.load(Ordering::SeqCst), 0);
    Ok(())
}

#[test]
fn wrong_address_and_tampered_target_never_alias_stored_value() -> TestResult {
    let tokenizer = fixture_tokenizer()?;
    let pinned = PinnedTokenizer::new(&tokenizer)?;
    let mut store = Store::new(Limits::default());
    let first = store.fold(b"alpha-value")?;
    let second = store.fold(b"beta-value")?;
    let ingested = ingest(&mut store, &pinned, b"abc")?;

    let mut dir = AddressDirectory::new(8)?;
    let addr_a = PrimeAddress::with_pairs(&[2, 5], &[(2, 5)])?;
    let addr_a_reversed = PrimeAddress::with_pairs(&[2, 5], &[(5, 2)])?;
    let addr_b = PrimeAddress::with_pairs(&[3, 7], &[(3, 7)])?;
    let addr_missing = PrimeAddress::new(&[11, 13])?;

    dir.write(
        addr_a.clone(),
        ObjectTarget::WholeObject {
            manifest: first.manifest.clone(),
        },
    )?;
    dir.write(
        addr_b.clone(),
        ObjectTarget::WholeObject {
            manifest: second.manifest.clone(),
        },
    )?;

    // Wrong unwritten address -> Absent.
    assert_eq!(
        dir.read_exact(&store, None, &addr_missing, VersionSelect::Current)?,
        AddressReadOutcome::Absent
    );
    // Reversed ordered pair -> distinct address, never aliases addr_a on exact lookup.
    assert_eq!(
        dir.read_exact(&store, None, &addr_a_reversed, VersionSelect::Current)?,
        AddressReadOutcome::Absent
    );
    // Reading addr_b returns only beta-value, never alpha-value.
    match dir.read_exact(&store, None, &addr_b, VersionSelect::Current)? {
        AddressReadOutcome::Hit { bytes, .. } => assert_eq!(bytes, b"beta-value"),
        other => return Err(format!("expected Hit on addr_b, got {other:?}").into()),
    }

    // Tampered manifest digest is rejected on read.
    let mut forged_bytes = first.manifest.encode();
    forged_bytes[16] ^= 1;
    let forged_manifest = uor_r4_object_store::Manifest::decode(&forged_bytes, store.limits())?;
    let addr_forged = PrimeAddress::new(&[19, 23])?;
    dir.write(
        addr_forged.clone(),
        ObjectTarget::WholeObject {
            manifest: forged_manifest,
        },
    )?;
    assert!(matches!(
        dir.read_exact(&store, None, &addr_forged, VersionSelect::Current),
        Err(StoreError::IntegrityFailure)
    ));

    // Mismatched manifest vs bindings in TokenOccurrence is rejected.
    let addr_bad_bind = PrimeAddress::new(&[29, 31])?;
    dir.write(
        addr_bad_bind.clone(),
        ObjectTarget::TokenOccurrence {
            manifest: second.manifest,
            bindings: ingested.bindings,
            index: 0,
        },
    )?;
    assert!(matches!(
        dir.read_exact(
            &store,
            Some(&pinned),
            &addr_bad_bind,
            VersionSelect::Current
        ),
        Err(StoreError::BindingMismatch)
    ));
    Ok(())
}

#[test]
fn overwrite_preserves_previous_version_and_capacity_ring_reports_evicted() -> TestResult {
    let mut store = Store::new(Limits::default());
    let v1 = store.fold(b"colour:blue")?;
    let v2 = store.fold(b"colour:green")?;
    let other1 = store.fold(b"city:paris")?;
    let other2 = store.fold(b"role:architect")?;

    let mut dir = AddressDirectory::new(2)?;
    let addr_colour = PrimeAddress::new(&[2, 3])?;
    let addr_city = PrimeAddress::new(&[5, 7])?;
    let addr_role = PrimeAddress::new(&[11, 13])?;

    dir.write(
        addr_colour.clone(),
        ObjectTarget::WholeObject {
            manifest: v1.manifest.clone(),
        },
    )?;
    // Same-value reassertion does not overwrite previous with a duplicate of current.
    dir.write(
        addr_colour.clone(),
        ObjectTarget::WholeObject {
            manifest: v1.manifest.clone(),
        },
    )?;
    // Updating with a new distinct target advances version and moves v1 to previous.
    dir.write(
        addr_colour.clone(),
        ObjectTarget::WholeObject {
            manifest: v2.manifest,
        },
    )?;

    match dir.read_exact(&store, None, &addr_colour, VersionSelect::Current)? {
        AddressReadOutcome::Hit { version, bytes, .. } => {
            assert_eq!(version, 2);
            assert_eq!(bytes, b"colour:green");
        }
        other => return Err(format!("expected current Hit, got {other:?}").into()),
    }
    match dir.read_exact(&store, None, &addr_colour, VersionSelect::Previous)? {
        AddressReadOutcome::Previous { version, bytes, .. } => {
            assert_eq!(version, 1);
            assert_eq!(bytes, b"colour:blue");
        }
        other => return Err(format!("expected Previous, got {other:?}").into()),
    }

    // Fill capacity (2) and insert a 3rd distinct address: oldest slot (addr_colour) is evicted.
    dir.write(
        addr_city.clone(),
        ObjectTarget::WholeObject {
            manifest: other1.manifest,
        },
    )?;
    dir.write(
        addr_role.clone(),
        ObjectTarget::WholeObject {
            manifest: other2.manifest,
        },
    )?;

    match dir.read_exact(&store, None, &addr_colour, VersionSelect::Current)? {
        AddressReadOutcome::Evicted {
            address,
            evicted_at_sequence,
        } => {
            assert_eq!(address, addr_colour);
            assert!(evicted_at_sequence > 0);
        }
        other => return Err(format!("expected Evicted, got {other:?}").into()),
    }
    Ok(())
}

#[test]
fn sorted_merge_gcd_sieve_admits_shared_primes_and_ranks_deterministically() -> TestResult {
    // Non-prime or empty factor lists are rejected before any directory work.
    assert!(matches!(
        PrimeAddress::new(&[]),
        Err(StoreError::SchemaMismatch)
    ));
    assert!(matches!(
        PrimeAddress::new(&[1, 2]),
        Err(StoreError::SchemaMismatch)
    ));
    assert!(matches!(
        PrimeAddress::new(&[2, 9]),
        Err(StoreError::SchemaMismatch)
    ));

    let mut store = Store::new(Limits::default());
    let m_one = store.fold(b"one-shared-prime")?;
    let m_two_unordered = store.fold(b"two-primes-reversed-pair")?;
    let m_two_ordered = store.fold(b"two-primes-matching-pair")?;

    let mut dir = AddressDirectory::new(8)?;
    let slot_one = PrimeAddress::with_pairs(&[2, 11], &[(2, 11)])?;
    let slot_two_rev = PrimeAddress::with_pairs(&[2, 3, 7], &[(3, 2)])?;
    let slot_two_ord = PrimeAddress::with_pairs(&[2, 3, 5], &[(2, 3)])?;

    dir.write(
        slot_one,
        ObjectTarget::WholeObject {
            manifest: m_one.manifest,
        },
    )?;
    // Write slot_two_ord BEFORE slot_two_rev so slot_two_rev has a higher sequence
    // number (3 vs 2); slot_two_ord must still win by ordered-pair match (1 vs 0).
    dir.write(
        slot_two_ord.clone(),
        ObjectTarget::WholeObject {
            manifest: m_two_ordered.manifest,
        },
    )?;
    dir.write(
        slot_two_rev.clone(),
        ObjectTarget::WholeObject {
            manifest: m_two_unordered.manifest,
        },
    )?;

    let query = PrimeAddress::with_pairs(&[2, 3, 13], &[(2, 3)])?;
    assert_eq!(query.gcd_shared_primes(&slot_two_ord), 2);
    assert_eq!(query.ordered_pair_overlap(&slot_two_ord), 1);
    assert_eq!(query.ordered_pair_overlap(&slot_two_rev), 0);

    match dir.read_sieve(&store, None, &query)? {
        AddressReadOutcome::Hit { address, bytes, .. } => {
            assert_eq!(address, slot_two_ord);
            assert_eq!(bytes, b"two-primes-matching-pair");
        }
        other => return Err(format!("expected sieve Hit, got {other:?}").into()),
    }

    // When shared_primes and ordered_pair_overlap tie, latest sequence wins.
    let m_two_newer = store.fold(b"two-primes-matching-pair-newer")?;
    let slot_two_newer = PrimeAddress::with_pairs(&[2, 3, 17], &[(2, 3)])?;
    dir.write(
        slot_two_newer.clone(),
        ObjectTarget::WholeObject {
            manifest: m_two_newer.manifest,
        },
    )?;
    match dir.read_sieve(&store, None, &query)? {
        AddressReadOutcome::Hit { address, bytes, .. } => {
            assert_eq!(address, slot_two_newer);
            assert_eq!(bytes, b"two-primes-matching-pair-newer");
        }
        other => return Err(format!("expected newer sieve Hit, got {other:?}").into()),
    }
    Ok(())
}

#[test]
fn byte_identical_round_trip_across_store_export_and_import_at_carry_boundaries() -> TestResult {
    let payload: Vec<u8> = (0..4096).map(|i| ((i * 73 + 19) % 251) as u8).collect();
    let mut writer = Store::new(Limits::default());
    let folded = writer.fold(&payload)?;

    let mut dir = AddressDirectory::new(8)?;
    let addr_whole = PrimeAddress::new(&[2, 3, 5])?;
    let addr_carry = PrimeAddress::new(&[7, 11, 13])?;
    dir.write(
        addr_whole.clone(),
        ObjectTarget::WholeObject {
            manifest: folded.manifest.clone(),
        },
    )?;
    dir.write(
        addr_carry.clone(),
        ObjectTarget::ByteSpan {
            manifest: folded.manifest,
            range: 31..289,
        },
    )?;

    let mut reader = Store::new(Limits::default());
    reader.import_records(&writer.export_records()?)?;

    match dir.read_exact(&reader, None, &addr_whole, VersionSelect::Current)? {
        AddressReadOutcome::Hit {
            bytes,
            whole_object_verified,
            ..
        } => {
            assert!(whole_object_verified);
            assert_eq!(bytes, payload);
        }
        other => return Err(format!("expected whole Hit, got {other:?}").into()),
    }
    match dir.read_exact(&reader, None, &addr_carry, VersionSelect::Current)? {
        AddressReadOutcome::Hit {
            bytes,
            whole_object_verified,
            ..
        } => {
            assert!(!whole_object_verified);
            assert_eq!(bytes, &payload[31..289]);
        }
        other => return Err(format!("expected carry-span Hit, got {other:?}").into()),
    }
    Ok(())
}

#[test]
fn sieve_candidate_counts_cover_active_and_evicted_admission() -> TestResult {
    let mut store = Store::new(Limits::default());
    let folded = store.fold(b"value")?;
    let evicted = PrimeAddress::new(&[2, 3])?;
    let active = PrimeAddress::new(&[3, 5])?;
    let mut dir = AddressDirectory::new(1)?;
    dir.write(
        evicted,
        ObjectTarget::WholeObject {
            manifest: folded.manifest.clone(),
        },
    )?;
    dir.write(
        active,
        ObjectTarget::WholeObject {
            manifest: folded.manifest,
        },
    )?;
    let counts = dir.sieve_candidate_counts(&PrimeAddress::new(&[3, 7])?);
    assert_eq!(counts.active_examined, 1);
    assert_eq!(counts.active_admitted, 1);
    assert_eq!(counts.evicted_examined, 0);
    assert_eq!(counts.evicted_admitted, 0);
    let counts = dir.sieve_candidate_counts(&PrimeAddress::new(&[2, 11])?);
    assert_eq!(counts.active_admitted, 0);
    assert_eq!(counts.evicted_admitted, 1);
    Ok(())
}
