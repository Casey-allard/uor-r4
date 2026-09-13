use super::*;

fn session() -> ObjectSession {
    ObjectSession::new([1; 32], [2; 32], 7)
}
fn ingest(s: &mut ObjectSession, text: &[u8]) -> Result<()> {
    for &byte in text {
        s.observe_input(byte, [1, 2], [3, 4, 5, 6])?;
    }
    Ok(())
}
fn step(s: &mut ObjectSession, offer: Offer, actual: Symbol) -> Result<()> {
    s.acknowledge(actual, offer.id)?;
    if matches!(actual, Symbol::Byte(_)) {
        s.commit_key([7, 8], [9, 10, 11, 12])?;
    }
    Ok(())
}
#[test]
fn canonical_scalars_and_ordered_arithmetic() -> Result<()> {
    for n in [i64::MIN, i64::MAX, -100, -1, 0, 1, 9, 10, 999] {
        let (b, len) = encode_i64(n);
        assert_eq!(decode_i64(&b[..usize::from(len)])?, n);
        assert_eq!(&b[..usize::from(len)], n.to_string().as_bytes());
    }
    for b in [
        b"".as_slice(),
        b"+1",
        b"-0",
        b"01",
        b" 1",
        b"1 ",
        b"--1",
        b"1x",
        b"9223372036854775808",
        b"-9223372036854775809",
    ] {
        assert!(decode_i64(b).is_err());
    }
    assert_eq!(execute(Action::SubAB, 2, 9)?, -7);
    assert_eq!(execute(Action::SubAB, 9, 2)?, 7);
    assert_eq!(
        execute(Action::AddAB, i64::MAX, 1),
        Err(ObjectError::Overflow)
    );
    assert_eq!(
        execute(Action::SubAB, i64::MIN, 1),
        Err(ObjectError::Overflow)
    );
    Ok(())
}
#[test]
fn occurrences_extents_duplicates_and_owned_eviction() -> Result<()> {
    let mut s = session();
    ingest(&mut s, b"same same")?;
    let a = s.acquire_occurrence(7, 0, 4)?;
    let b = s.acquire_occurrence(7, 5, 4)?;
    assert_eq!(a.payload(), b.payload());
    assert_ne!(a.reference(), b.reference());
    assert!(s.acquire_occurrence(8, 0, 4).is_err());
    assert!(s.acquire_occurrence(7, 9, 1).is_err());
    assert!(s.acquire_occurrence(7, 0, 0).is_err());
    assert!(s.acquire_occurrence(7, 0, 65).is_err());
    s.begin_turn()?;
    ingest(&mut s, b"x")?;
    assert!(s.acquire_occurrence(7, 8, 2).is_err());
    for _ in 0..256 {
        ingest(&mut s, b"z")?;
    }
    assert!(s.occurrence(7, 0).is_err());
    assert_eq!(a.payload(), b"same");
    let o = s.cache_offer(Symbol::Byte(b's'), Action::AcquireA, Some(a), Some(b))?;
    step(&mut s, o, Symbol::Byte(b's'))?;
    assert_eq!(
        s.active()
            .ok_or(ObjectError::InvalidReference)?
            .lease()
            .payload(),
        b"same"
    );
    let snapshot = s.snapshot()?;
    assert_eq!(ObjectSession::restore(&snapshot, [1; 32], [2; 32])?, s);
    Ok(())
}
#[test]
fn resident_cross_session_namespace_collision_is_rejected() -> Result<()> {
    let mut local = session();
    let mut other = session();
    ingest(&mut local, b"cat")?;
    ingest(&mut other, b"dog")?;
    let foreign = other.acquire_occurrence(7, 0, 3)?;
    let before = local.snapshot()?;
    assert!(local
        .cache_offer(Symbol::Byte(b'd'), Action::AcquireA, Some(foreign), None)
        .is_err());
    assert_eq!(local.snapshot()?, before);
    let mut distinct = ObjectSession::new([1; 32], [2; 32], 9);
    ingest(&mut distinct, b"cat")?;
    let foreign = distinct.acquire_occurrence(9, 0, 3)?;
    assert!(local
        .cache_offer(Symbol::Byte(b'c'), Action::AcquireA, Some(foreign), None)
        .is_err());
    assert_eq!(local.snapshot()?, before);
    Ok(())
}
#[test]
fn cached_offers_unknown_ids_and_external_ingestion() -> Result<()> {
    let mut s = session();
    ingest(&mut s, b"ab")?;
    let a = s.acquire_occurrence(7, 0, 2)?;
    let o = s.cache_offer(Symbol::Byte(b'a'), Action::AcquireA, Some(a), None)?;
    let frozen = s.snapshot()?;
    assert_eq!(
        s.cache_offer(Symbol::Byte(b'a'), Action::AcquireA, Some(a), None)?,
        o
    );
    assert_eq!(s.snapshot()?, frozen);
    assert_eq!(
        s.acknowledge(Symbol::Byte(b'a'), o.id + 1),
        Err(ObjectError::UnknownOffer)
    );
    assert_eq!(s.snapshot()?, frozen);
    s.acknowledge(Symbol::Byte(b'x'), o.id)?;
    assert!(s.active().is_none());
    assert_eq!(s.frontier(), 2);
    let consumed = s.snapshot()?;
    assert_eq!(
        s.acknowledge(Symbol::Byte(b'x'), o.id),
        Err(ObjectError::UnknownOffer)
    );
    assert_eq!(s.snapshot()?, consumed);
    assert_eq!(
        s.cache_offer(Symbol::Byte(b'x'), Action::Hold, None, None),
        Err(ObjectError::KeyRequired)
    );
    let mut resumed = ObjectSession::restore(&consumed, [1; 32], [2; 32])?;
    s.commit_key([0; 2], [0; 4])?;
    resumed.commit_key([0; 2], [0; 4])?;
    assert_eq!(s, resumed);
    assert_eq!(s.frontier(), 3);
    assert_eq!(s.occurrence(7, 2)?.byte, b'x');
    assert!(s.commit_key([0; 2], [0; 4]).is_err());
    let old = s.cache_offer(Symbol::Byte(b'z'), Action::Hold, None, None)?;
    s.observe_input(b'q', [0; 2], [0; 4])?;
    let now = s.cache_offer(Symbol::Byte(b'z'), Action::Hold, None, None)?;
    assert_ne!(old.id, now.id);
    let before = s.snapshot()?;
    assert!(s.acknowledge(Symbol::Byte(b'z'), old.id).is_err());
    assert_eq!(s.snapshot()?, before);
    Ok(())
}
#[test]
fn lease_byte_never_overrides_emission_and_mismatch_cancels() -> Result<()> {
    let mut s = session();
    ingest(&mut s, b"ab")?;
    let a = s.acquire_occurrence(7, 0, 2)?;
    let o = s.cache_offer(Symbol::Byte(b'z'), Action::AcquireA, Some(a), None)?;
    assert_eq!(o.symbol, Symbol::Byte(b'z'));
    step(&mut s, o, Symbol::Byte(b'z'))?;
    assert!(s.active().is_none());
    let o = s.cache_offer(Symbol::Byte(b'a'), Action::AcquireA, Some(a), None)?;
    step(&mut s, o, Symbol::Byte(b'a'))?;
    assert!(s
        .active()
        .ok_or(ObjectError::InvalidReference)?
        .acknowledged());
    let o = s.cache_offer(Symbol::Byte(b'z'), Action::Hold, None, None)?;
    step(&mut s, o, Symbol::Byte(b'z'))?;
    assert!(s.active().is_none());
    assert!(!s.legal_actions(None, None)[5]);
    Ok(())
}
fn arithmetic_setup() -> Result<(ObjectSession, Lease, Lease)> {
    let mut s = session();
    ingest(&mut s, b"19 7")?;
    let a = s.acquire_occurrence(7, 0, 2)?;
    let b = s.acquire_occurrence(7, 3, 1)?;
    Ok((s, a, b))
}
#[test]
fn result_publication_requires_final_ack_and_same_key_once() -> Result<()> {
    let (mut s, a, b) = arithmetic_setup()?;
    let o = s.cache_offer(Symbol::Byte(b'1'), Action::SubAB, Some(a), Some(b))?;
    assert_eq!(
        o.active()
            .ok_or(ObjectError::InvalidReference)?
            .lease()
            .payload(),
        b"12"
    );
    step(&mut s, o, Symbol::Byte(b'1'))?;
    assert!(s.result(7, 1).is_err());
    assert!(!s.legal_actions(None, None)[3]);
    assert!(s.legal_actions(None, None)[5]);
    let o = s.cache_offer(Symbol::Byte(b'2'), Action::Advance, None, None)?;
    let pending = s.snapshot()?;
    let mut resumed = ObjectSession::restore(&pending, [1; 32], [2; 32])?;
    s.acknowledge(Symbol::Byte(b'2'), o.id)?;
    assert!(s.result(7, 1).is_err());
    let ready = s.snapshot()?;
    let mut restored = ObjectSession::restore(&ready, [1; 32], [2; 32])?;
    s.commit_key([11, 12], [13, 14, 15, 16])?;
    restored.commit_key([11, 12], [13, 14, 15, 16])?;
    resumed.acknowledge(Symbol::Byte(b'2'), o.id)?;
    resumed.commit_key([11, 12], [13, 14, 15, 16])?;
    assert_eq!(s, restored);
    assert_eq!(s, resumed);
    let r = *s.result(7, 1)?;
    assert_eq!(r.lease.payload(), b"12");
    assert_eq!(r.lease.keys(), [11, 12]);
    assert_eq!(r.derivation.operands, [a.reference(), b.reference()]);
    assert_eq!(r.derivation.operand_values, [19, 7]);
    assert_eq!(s.publications_this_turn(), 1);
    let before = s.snapshot()?;
    assert!(s.commit_key([11, 12], [13, 14, 15, 16]).is_err());
    assert_eq!(s.snapshot()?, before);
    let derived = s.acquire_result(7, 1)?;
    let o = s.cache_offer(
        Symbol::Byte(b'2'),
        Action::AddAB,
        Some(derived),
        Some(derived),
    )?;
    step(&mut s, o, Symbol::Byte(b'2'))?;
    let o = s.cache_offer(Symbol::Byte(b'4'), Action::Advance, None, None)?;
    step(&mut s, o, Symbol::Byte(b'4'))?;
    assert_eq!(
        s.result(7, 2)?.derivation.operands,
        [derived.reference(); 2]
    );
    assert_eq!(s.result(7, 2)?.derivation.value, 24);
    assert!(!s.legal_actions(Some(a), Some(b))[3]);
    s.begin_turn()?;
    assert!(s.legal_actions(Some(a), Some(b))[3]);
    Ok(())
}
#[test]
fn partial_mismatch_eos_clear_and_repeated_byte_do_not_publish() -> Result<()> {
    for stop in 0..5 {
        let (mut s, a, b) = arithmetic_setup()?;
        let o = s.cache_offer(Symbol::Byte(b'1'), Action::SubAB, Some(a), Some(b))?;
        step(&mut s, o, Symbol::Byte(b'1'))?;
        match stop {
            0 => {
                let o = s.cache_offer(Symbol::Byte(b'2'), Action::Advance, None, None)?;
                step(&mut s, o, Symbol::Byte(b'x'))?;
            }
            1 => {
                let o = s.cache_offer(Symbol::Eos, Action::Hold, None, None)?;
                step(&mut s, o, Symbol::Eos)?;
            }
            2 => {
                let o = s.cache_offer(Symbol::Byte(b'z'), Action::Clear, None, None)?;
                step(&mut s, o, Symbol::Byte(b'z'))?;
            }
            3 => s.begin_turn()?,
            _ => {
                let o = s.cache_offer(Symbol::Byte(b'1'), Action::Hold, None, None)?;
                step(&mut s, o, Symbol::Byte(b'1'))?;
            }
        }
        assert!(s.result(7, 1).is_err());
        assert!(s.active().is_none());
        assert_eq!(s.publications_this_turn(), 0);
        s.snapshot()?;
    }
    Ok(())
}
#[test]
fn turn_reset_counter_bounds_and_snapshot_strictness() -> Result<()> {
    let mut s = session();
    ingest(&mut s, b"x")?;
    let old = s.cache_offer(Symbol::Byte(b'a'), Action::Hold, None, None)?;
    s.reset_session()?;
    assert_eq!(s.epoch(), 8);
    assert_eq!(s.frontier(), 0);
    let now = s.cache_offer(Symbol::Byte(b'a'), Action::Hold, None, None)?;
    assert_ne!(old.id, now.id);
    let before = s.snapshot()?;
    assert!(s.acknowledge(Symbol::Byte(b'a'), old.id).is_err());
    assert_eq!(s.snapshot()?, before);
    assert!(ObjectSession::restore(&before, [3; 32], [2; 32]).is_err());
    let mut trailing = before.clone();
    trailing.push(0);
    assert!(ObjectSession::restore(&trailing, [1; 32], [2; 32]).is_err());
    for at in [0, 12, before.len() - 1] {
        let mut changed = before.clone();
        changed[at] ^= 1;
        assert!(ObjectSession::restore(&changed, [1; 32], [2; 32]).is_err());
    }
    // Rechecksummed malformed option discriminants must also fail structurally.
    let mut invalid = before.clone();
    invalid[113] = 2;
    let end = invalid.len() - 32;
    let hash = *blake3::hash(&invalid[..end]).as_bytes();
    invalid[end..].copy_from_slice(&hash);
    assert!(ObjectSession::restore(&invalid, [1; 32], [2; 32]).is_err());
    let mut full = session();
    for _ in 0..300 {
        ingest(&mut full, b"q")?;
    }
    assert!(full.snapshot()?.len() < SNAPSHOT_MAX);
    let mut exhausted = session();
    exhausted.seen = u64::MAX;
    let saved = exhausted.clone();
    assert_eq!(
        exhausted.observe_input(b'x', [0; 2], [0; 4]),
        Err(ObjectError::Exhausted)
    );
    assert_eq!(exhausted, saved);
    Ok(())
}
#[test]
fn full_snapshot_results_reuse_and_forged_retained_payload_rejected() -> Result<()> {
    let mut s = session();
    for _ in 0..256 {
        ingest(&mut s, b"1")?;
    }
    let a = s.acquire_occurrence(7, 255, 1)?;
    for _ in 0..9 {
        s.begin_turn()?;
        let o = s.cache_offer(Symbol::Byte(b'2'), Action::AddAB, Some(a), Some(a))?;
        step(&mut s, o, Symbol::Byte(b'2'))?;
    }
    assert!(s.result(7, 1).is_err());
    assert_eq!(s.result(7, 9)?.lease.payload(), b"2");
    let r = s.acquire_result(7, 9)?;
    s.cache_offer(Symbol::Byte(b'2'), Action::AcquireA, Some(r), Some(a))?;
    let bytes = s.snapshot()?;
    assert!(bytes.len() < SNAPSHOT_MAX);
    assert_eq!(s.occurrences.iter().flatten().count(), 256);
    assert_eq!(s.results.iter().flatten().count(), 8);
    assert_eq!(ObjectSession::restore(&bytes, [1; 32], [2; 32])?, s);
    let mut bad = session();
    ingest(&mut bad, b"19")?;
    let lease = bad.acquire_occurrence(7, 0, 2)?;
    bad.cache_offer(Symbol::Byte(b'1'), Action::AcquireA, Some(lease), None)?;
    let mut offer = bad.pending.ok_or(ObjectError::UnknownOffer)?;
    if let Some(l) = &mut offer.operands[0] {
        l.bytes[0] = b'8';
    }
    if let Some(a) = &mut offer.active {
        a.lease.bytes[0] = b'8';
    }
    bad.pending = Some(offer);
    assert!(bad.snapshot().is_err());
    let (mut bad, a, b) = arithmetic_setup()?;
    let o = bad.cache_offer(Symbol::Byte(b'1'), Action::SubAB, Some(a), Some(b))?;
    step(&mut bad, o, Symbol::Byte(b'1'))?;
    let o = bad.cache_offer(Symbol::Byte(b'2'), Action::Advance, None, None)?;
    step(&mut bad, o, Symbol::Byte(b'2'))?;
    ingest(&mut bad, b"7")?;
    let seen = bad.frontier();
    if let Some(record) = &mut bad.results[1] {
        record.derivation.operands[1] = Reference::Occurrence {
            epoch: 7,
            turn: 0,
            start: seen - 1,
            end: seen,
        };
    }
    assert!(bad.snapshot().is_err());
    Ok(())
}
