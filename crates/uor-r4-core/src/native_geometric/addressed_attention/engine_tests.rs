use super::*;

struct TestPolicy {
    identity: u16,
    opposite: u16,
    phase: Phase,
    calls: usize,
    choices: usize,
    query_a: [u16; 2],
    query_b: [u16; 2],
    keys: [u16; 2],
    null: [u16; 2],
    extent_a: usize,
    extent_b: usize,
    action: Action,
    symbol: usize,
    frames: Vec<(Phase, [bool; 1024])>,
    fail: bool,
}
impl TestPolicy {
    fn new(g: &BoundGeometry) -> Result<Self> {
        let identity = g.identity();
        let mut opposite = identity;
        let mut minimum = i16::MAX;
        for r in 0..120 {
            let score = geo(g.score([identity; 2], [r; 2]))?;
            if score < minimum {
                minimum = score;
                opposite = r;
            }
        }
        Ok(Self {
            identity,
            opposite,
            phase: Phase::QueryA,
            calls: 0,
            choices: 0,
            query_a: [identity; 2],
            query_b: [identity; 2],
            keys: [identity; 2],
            null: [opposite; 2],
            extent_a: 0,
            extent_b: 0,
            action: Action::Hold,
            symbol: usize::from(b'x'),
            frames: Vec::new(),
            fail: false,
        })
    }
}
impl Policy for TestPolicy {
    fn context(&mut self, phase: Phase, input: &[bool; 1024]) -> Result<u16> {
        self.calls += 1;
        self.phase = phase;
        self.frames.push((phase, *input));
        if self.fail {
            return Err(EngineError::Invalid("deliberate test failure"));
        }
        Ok(phase as u16)
    }
    fn choice(&mut self, head: Head, _context: u16, legal: &[bool]) -> Result<usize> {
        self.choices += 1;
        let value = match head {
            Head::Root(n) if n < 2 => usize::from(if self.phase == Phase::QueryA {
                self.query_a[usize::from(n)]
            } else {
                self.query_b[usize::from(n)]
            }),
            Head::Root(n) if n < 4 => usize::from(self.keys[usize::from(n - 2)]),
            Head::Root(_) => usize::from(self.identity),
            Head::Null(n) => usize::from(self.null[usize::from(n)]),
            Head::Extent => {
                if self.phase == Phase::ExtentA {
                    self.extent_a
                } else {
                    self.extent_b
                }
            }
            Head::Control => usize::from(action_index(self.action)),
            Head::Emission => self.symbol,
        };
        // Deliberate compiled-preference behavior for lengths/control only.
        if matches!(head, Head::Extent | Head::Control)
            && !legal.get(value).copied().unwrap_or(false)
        {
            return legal
                .iter()
                .position(|&b| b)
                .ok_or(EngineError::Invalid("test empty legal"));
        }
        Ok(value)
    }
}
fn setup() -> Result<(BoundGeometry, RuntimeSession, TestPolicy)> {
    let g = geo(BoundGeometry::canonical())?;
    let s = RuntimeSession::new([9; 32], &g, 41)?;
    let p = TestPolicy::new(&g)?;
    Ok((g, s, p))
}
fn packed_byte(bits: &[bool; 1024], offset: usize) -> u8 {
    bits[offset..offset + 8]
        .iter()
        .enumerate()
        .fold(0, |out, (i, &b)| out | (u8::from(b) << i))
}

#[test]
fn input_runs_observe_key_with_native_byte_phases() -> Result<()> {
    let (g, mut s, mut p) = setup()?;
    let trace = s.observe_input(b'A', &g, &mut p)?;
    assert_eq!(trace.circuit_calls(), 2);
    assert!(trace.contexts[6].is_some() && trace.contexts[7].is_some());
    assert_eq!(s.phases(), g.byte_phases(b'A'));
    assert_eq!(s.objects().occurrence(41, 0)?.byte, b'A');
    assert_eq!(s.objects().occurrence(41, 0)?.roots, [g.identity(); 4]);
    assert_eq!(p.frames[0].0, Phase::Observe);
    assert_eq!(packed_byte(&p.frames[0].1, 768), b'A');
    assert!(!p.frames[0].1[801] && !p.frames[0].1[802] && !p.frames[0].1[803]);
    let old = s.phases();
    s.observe_input(255, &g, &mut p)?;
    assert_eq!(
        s.phases(),
        std::array::from_fn(|i| old[i].wrapping_add(g.byte_phases(255)[i]))
    );
    Ok(())
}
#[test]
fn cached_prediction_no_target_and_unknown_id_transactionality() -> Result<()> {
    let (g, mut s, mut p) = setup()?;
    s.observe_input(b'a', &g, &mut p)?;
    p.action = Action::AcquireA;
    p.symbol = usize::from(b'a');
    let offer = s.predict(&g, &mut p)?;
    assert_eq!(offer.trace.circuit_calls(), 6);
    assert_eq!(
        offer.trace.selected[0],
        Some(Reference::Occurrence {
            epoch: 41,
            turn: 0,
            start: 0,
            end: 1
        })
    );
    let calls = p.calls;
    let choices = p.choices;
    let snapshot = s.snapshot()?;
    assert_eq!(s.predict(&g, &mut p)?, offer);
    assert_eq!(p.calls, calls);
    assert_eq!(p.choices, choices);
    assert_eq!(s.snapshot()?, snapshot);
    assert!(s
        .observe(Symbol::Byte(b'a'), offer.offer.id + 1, &g, &mut p)
        .is_err());
    assert_eq!(p.calls, calls);
    assert_eq!(s.snapshot()?, snapshot);
    let emit = p
        .frames
        .iter()
        .rev()
        .find(|(phase, _)| *phase == Phase::Emit)
        .ok_or(EngineError::Invalid("missing emit"))?;
    assert_eq!(packed_byte(&emit.1, 792), b'a');
    assert_eq!(packed_byte(&emit.1, 768), b'a');
    // The different actual byte is supplied only after this cached EMIT frame.
    let trace = s.observe(Symbol::Byte(b'z'), offer.offer.id, &g, &mut p)?;
    assert_eq!(trace.circuit_calls(), 8);
    assert!(s.objects().active().is_none());
    assert_eq!(s.objects().occurrence(41, 1)?.byte, b'z');
    let observe = p
        .frames
        .iter()
        .rev()
        .find(|(phase, _)| *phase == Phase::Observe)
        .ok_or(EngineError::Invalid("missing observe"))?;
    assert_eq!(packed_byte(&observe.1, 768), b'z');
    assert!(!observe.1[803]);
    let done = s.snapshot()?;
    let calls = p.calls;
    assert!(s
        .observe(Symbol::Byte(b'z'), offer.offer.id, &g, &mut p)
        .is_err());
    assert_eq!(p.calls, calls);
    assert_eq!(s.snapshot()?, done);
    Ok(())
}
#[test]
fn null_ties_legal_extent_turn_and_policy_error_rollback() -> Result<()> {
    let (g, mut s, mut p) = setup()?;
    s.observe_input(b'a', &g, &mut p)?;
    p.null = p.keys;
    let offer = s.predict(&g, &mut p)?;
    assert_eq!(offer.trace.selected, [None; 2]);
    assert_eq!(offer.trace.circuit_calls(), 4);
    s.observe(Symbol::Byte(b'x'), offer.offer.id, &g, &mut p)?;
    s.begin_turn()?;
    s.observe_input(b'b', &g, &mut p)?;
    p.null = [p.opposite; 2];
    p.extent_a = 63;
    p.action = Action::AcquireA;
    let offer = s.predict(&g, &mut p)?;
    assert_eq!(
        offer
            .offer
            .active()
            .ok_or(EngineError::Invalid("active"))?
            .lease()
            .payload(),
        b"a"
    );
    let frozen = s.snapshot()?;
    p.fail = true;
    assert!(s
        .observe(offer.offer.symbol, offer.offer.id, &g, &mut p)
        .is_err());
    assert_eq!(s.snapshot()?, frozen);
    p.fail = false;
    s.observe(offer.offer.symbol, offer.offer.id, &g, &mut p)?;
    Ok(())
}
#[test]
fn ordered_two_read_arithmetic_emit_and_same_key_publication() -> Result<()> {
    let (g, mut s, mut p) = setup()?;
    // Position0 keys I; position1 keys -I. Queries therefore bind ordered 9,2.
    p.keys = [p.identity; 2];
    s.observe_input(b'9', &g, &mut p)?;
    p.keys = [p.opposite; 2];
    s.observe_input(b'2', &g, &mut p)?;
    p.query_a = [p.identity; 2];
    p.query_b = [p.opposite; 2];
    // Null must lose against both poles: use a non-pole finite root.
    let neutral = (0..120)
        .find(|&r| r != p.identity && r != p.opposite)
        .ok_or(EngineError::Invalid("neutral"))?;
    p.null = [neutral; 2];
    p.action = Action::SubAB;
    p.symbol = usize::from(b'7');
    let o = s.predict(&g, &mut p)?;
    assert_eq!(o.trace.scalar_decodes, 2);
    assert_eq!(o.trace.selected_operations, 1);
    assert_eq!(
        o.offer
            .active()
            .ok_or(EngineError::Invalid("result"))?
            .lease()
            .payload(),
        b"7"
    );
    assert_ne!(o.trace.selected[0], o.trace.selected[1]);
    assert!(s.objects().result(41, 1).is_err());
    let emit = p
        .frames
        .iter()
        .rev()
        .find(|(phase, _)| *phase == Phase::Emit)
        .ok_or(EngineError::Invalid("emit"))?;
    assert_eq!(packed_byte(&emit.1, 792), b'7');
    assert!(emit.1[804] && emit.1[805] && emit.1[806] && emit.1[844]);
    let calls = p.calls;
    p.keys = [p.identity, p.opposite];
    let trace = s.observe(Symbol::Byte(b'7'), o.offer.id, &g, &mut p)?;
    assert_eq!(p.calls - calls, 2);
    assert_eq!(trace.circuit_calls(), 8);
    assert_eq!(s.work().scalar_decodes, 2);
    assert_eq!(s.work().selected_operations, 1);
    let r = s.objects().result(41, 1)?;
    assert_eq!(r.derivation.operand_values, [9, 2]);
    assert_eq!(r.derivation.value, 7);
    assert_eq!(r.lease.keys(), [p.identity, p.opposite]);
    let snapshot = s.snapshot()?;
    assert!(s
        .observe(Symbol::Byte(b'7'), o.offer.id, &g, &mut p)
        .is_err());
    assert_eq!(s.snapshot()?, snapshot);
    Ok(())
}
#[test]
fn eos_and_prepared_action_stamps_and_snapshot_root_consistency() -> Result<()> {
    let (g, mut s, mut p) = setup()?;
    s.observe_input(b'1', &g, &mut p)?;
    let a = s.objects.acquire_occurrence(41, 0, 1)?;
    let prepared = s.objects.prepare_operands(Some(a), Some(a))?;
    assert_eq!(prepared.scalar_decodes(), 2);
    assert_eq!(prepared.numeric_valid(), [true; 2]);
    let action = s.objects.prepare_action(prepared, Action::AddAB)?;
    s.begin_turn()?;
    assert!(s
        .objects
        .cache_prepared_offer(Symbol::Byte(b'2'), action)
        .is_err());
    p.symbol = 256;
    p.action = Action::Hold;
    let o = s.predict(&g, &mut p)?;
    let calls = p.calls;
    let seen = s.objects.frontier();
    s.observe(Symbol::Eos, o.offer.id, &g, &mut p)?;
    assert_eq!(p.calls, calls);
    assert_eq!(s.objects.frontier(), seen);
    assert!(s.predict(&g, &mut p).is_err());
    let bytes = s.snapshot()?;
    assert_eq!(RuntimeSession::restore(&bytes, [9; 32], &g)?, s);
    let calls = p.calls;
    assert!(s.observe(Symbol::Eos, o.offer.id, &g, &mut p).is_err());
    assert_eq!(p.calls, calls);
    assert_eq!(s.snapshot()?, bytes);
    let mut forged = s.clone();
    forged.roots[0] = p.opposite;
    if let Some(t) = &mut forged.last_trace {
        t.roots_after = forged.roots;
    }
    let forged = forged.snapshot()?;
    assert!(RuntimeSession::restore(&forged, [9; 32], &g).is_err());
    s.begin_turn()?;
    assert!(s.predict(&g, &mut p).is_ok());
    Ok(())
}
#[test]
fn snapshot_resume_full_ring_pending_and_session_reset() -> Result<()> {
    let (g, mut s, mut p) = setup()?;
    for i in 0..256 {
        s.observe_input(i as u8, &g, &mut p)?;
    }
    p.action = Action::AcquireA;
    p.symbol = 0;
    let o = s.predict(&g, &mut p)?;
    let bytes = s.snapshot()?;
    assert!(bytes.len() < 65_536);
    let mut restored = RuntimeSession::restore(&bytes, [9; 32], &g)?;
    let calls = p.calls;
    assert_eq!(restored.predict(&g, &mut p)?, o);
    assert_eq!(p.calls, calls);
    let a = s.observe(Symbol::Byte(0), o.offer.id, &g, &mut p)?;
    let b = restored.observe(Symbol::Byte(0), o.offer.id, &g, &mut p)?;
    assert_eq!(a, b);
    assert_eq!(s, restored);
    assert!(s.objects().occurrence(41, 0).is_err());
    assert_eq!(
        s.objects()
            .active()
            .ok_or(EngineError::Invalid("active"))?
            .lease()
            .payload(),
        &[0]
    );
    let mut corrupt = bytes.clone();
    corrupt[80] ^= 1;
    assert!(RuntimeSession::restore(&corrupt, [9; 32], &g).is_err());
    assert!(RuntimeSession::restore(&bytes, [8; 32], &g).is_err());
    s.begin_turn()?;
    assert!(s.objects().active().is_none());
    let stale = s.predict(&g, &mut p)?;
    s.reset_session(&g)?;
    assert!(s
        .observe(Symbol::Byte(0), stale.offer.id, &g, &mut p)
        .is_err());
    assert_eq!(s.roots(), [g.identity(); 4]);
    assert_eq!(s.phases(), [0; 4]);
    assert_eq!(s.objects().frontier(), 0);
    Ok(())
}
#[test]
fn prepared_overflow_masks_match_checked_scalar_domain() -> Result<()> {
    for a in [i64::MIN, i64::MAX, -1, 0, 1] {
        for b in [i64::MIN, i64::MAX, -1, 0, 1] {
            let mut s = ObjectSession::new([1; 32], [2; 32], 1);
            let (text_a, len_a) = objects::encode_i64(a);
            let (text_b, len_b) = objects::encode_i64(b);
            for &byte in &text_a[..usize::from(len_a)] {
                s.observe_input(byte, [0; 2], [0; 4])?;
            }
            let start_b = s.frontier();
            for &byte in &text_b[..usize::from(len_b)] {
                s.observe_input(byte, [0; 2], [0; 4])?;
            }
            let la = s.acquire_occurrence(1, 0, len_a)?;
            let lb = s.acquire_occurrence(1, start_b, len_b)?;
            let prepared = s.prepare_operands(Some(la), Some(lb))?;
            assert_eq!(prepared.legal()[3], a.checked_add(b).is_some());
            assert_eq!(prepared.legal()[4], a.checked_sub(b).is_some());
            for (action, expected) in [
                (Action::AddAB, a.checked_add(b)),
                (Action::SubAB, a.checked_sub(b)),
            ] {
                if let Some(value) = expected {
                    let active = s.prepare_action(prepared, action)?;
                    assert_eq!(
                        objects::decode_i64(
                            active
                                .active()
                                .ok_or(EngineError::Invalid("prepared result"))?
                                .lease()
                                .payload()
                        )?,
                        value
                    );
                }
            }
        }
    }
    Ok(())
}
