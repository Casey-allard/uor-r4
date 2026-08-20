use uor_r4_graph_format::ScoreQ;

pub const TOKEN_STATE_CAPACITY: usize = 32;
pub const LOCAL_STATE_CAPACITY: usize = 8;
pub const SEGMENT_STATE_CAPACITY: usize = 8;
pub const SESSION_STATE_CAPACITY: usize = 8;

/// One fixed-capacity semantic-state slot reserved for compiler-generated
/// update programs in Phase 8.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SemanticStateSlot {
    pub region_id: u32,
    pub token: u32,
    pub score_q: ScoreQ,
    pub age: u16,
}

/// Skeleton hook payload for compiler-generated local/segment/session updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReservedStateUpdate {
    pub program_id: u16,
    pub slot: SemanticStateSlot,
}

/// Fixed-capacity token history; when saturated it keeps the most recent
/// tokens without allocating.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenState<const CAP: usize> {
    len: usize,
    recent: [u32; CAP],
}

impl<const CAP: usize> Default for TokenState<CAP> {
    fn default() -> Self {
        Self {
            len: 0,
            recent: [0; CAP],
        }
    }
}

impl<const CAP: usize> TokenState<CAP> {
    pub const fn capacity(&self) -> usize {
        CAP
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_slice(&self) -> &[u32] {
        &self.recent[..self.len]
    }

    pub fn clear(&mut self) {
        for token in &mut self.recent[..self.len] {
            *token = 0;
        }
        self.len = 0;
    }

    pub fn push(&mut self, token: u32) {
        if CAP == 0 {
            return;
        }
        if self.len < CAP {
            self.recent[self.len] = token;
            self.len += 1;
            return;
        }
        self.recent.copy_within(1..CAP, 0);
        self.recent[CAP - 1] = token;
    }

    pub fn occurrences(&self, token: u32) -> usize {
        self.as_slice()
            .iter()
            .filter(|&&seen| seen == token)
            .count()
    }
}

/// Fixed-capacity reserved state for the local/segment/session levels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservedState<const CAP: usize> {
    len: usize,
    slots: [SemanticStateSlot; CAP],
    last_program_id: Option<u16>,
    updates_applied: u64,
}

impl<const CAP: usize> Default for ReservedState<CAP> {
    fn default() -> Self {
        Self {
            len: 0,
            slots: [SemanticStateSlot::default(); CAP],
            last_program_id: None,
            updates_applied: 0,
        }
    }
}

impl<const CAP: usize> ReservedState<CAP> {
    pub const fn capacity(&self) -> usize {
        CAP
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_slice(&self) -> &[SemanticStateSlot] {
        &self.slots[..self.len]
    }

    pub fn last_program_id(&self) -> Option<u16> {
        self.last_program_id
    }

    pub fn updates_applied(&self) -> u64 {
        self.updates_applied
    }

    pub fn clear(&mut self) {
        for slot in &mut self.slots[..self.len] {
            *slot = SemanticStateSlot::default();
        }
        self.len = 0;
        self.last_program_id = None;
        self.updates_applied = 0;
    }

    pub fn apply(&mut self, update: ReservedStateUpdate) {
        if CAP == 0 {
            self.last_program_id = Some(update.program_id);
            self.updates_applied += 1;
            return;
        }
        if self.len < CAP {
            self.slots[self.len] = update.slot;
            self.len += 1;
        } else {
            self.slots.copy_within(1..CAP, 0);
            self.slots[CAP - 1] = update.slot;
        }
        self.last_program_id = Some(update.program_id);
        self.updates_applied += 1;
    }

    pub fn update_slot(&mut self, slot: SemanticStateSlot) {
        if CAP == 0 {
            return;
        }
        if self.len < CAP {
            self.slots[self.len] = slot;
            self.len += 1;
        } else {
            self.slots[CAP - 1] = slot;
        }
    }

    pub fn shift_slots(&mut self) {
        if CAP == 0 {
            return;
        }
        if self.len == CAP {
            self.slots.copy_within(1..CAP, 0);
            self.slots[CAP - 1] = SemanticStateSlot::default();
        } else if self.len > 0 {
            // Can just push it up by maintaining logical len, wait:
            // if we want to explicitly shift, we probably shift all elements and decrement len
            if self.len > 0 {
                self.slots.copy_within(1..self.len, 0);
                self.len -= 1;
            }
        }
    }
}

/// One segment-lane ring slot: a quantized content key and its saturating
/// `ScoreQ` weight. Integer fields only (no_std, P-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SegmentSlot {
    pub key: u32,
    pub weight: ScoreQ,
}

/// The #835 segment lane, lowered to the hot path (#836): a fixed-capacity,
/// bounded ring with halving (arithmetic-right-shift) decay and saturating
/// `ScoreQ` accumulation. Caller-owned fixed-capacity state; **no allocation,
/// no multiply/divide/float** (P-4). Bit-equivalent to the #835 reference ring
/// semantics (`docs/prompt_state_spec_835.md` §6 / `docs/scoring_semantics.md`
/// §6): saturating upsert, canonical eviction victim (least weight, ties broken
/// by the larger key so the lower id is retained), division-free decay, and a
/// decode-independent candidate contribution.
///
/// This is a **dormant** runtime primitive: it is not yet wired into the
/// deployed R4Engine scoring path, so serving behavior is unchanged. The
/// descriptor it consumes (`ring_capacity`, `decay_shift`, `boost`) rides in
/// the optional PSTATE section; a later #836 increment activates it on the
/// request path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentRing<const CAP: usize> {
    len: usize,
    slots: [SegmentSlot; CAP],
    decay_shift: u32,
}

impl<const CAP: usize> Default for SegmentRing<CAP> {
    fn default() -> Self {
        Self {
            len: 0,
            slots: [SegmentSlot::default(); CAP],
            decay_shift: 0,
        }
    }
}

impl<const CAP: usize> SegmentRing<CAP> {
    /// A ring with the given halving decay shift; capacity is `CAP`.
    pub fn with_decay(decay_shift: u32) -> Self {
        Self {
            decay_shift,
            ..Self::default()
        }
    }

    pub const fn capacity(&self) -> usize {
        CAP
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_slice(&self) -> &[SegmentSlot] {
        &self.slots[..self.len]
    }

    pub fn clear(&mut self) {
        for slot in &mut self.slots[..self.len] {
            *slot = SegmentSlot::default();
        }
        self.len = 0;
    }

    /// Halving decay: arithmetic right-shift every weight, then drop slots that
    /// reached the floor (`weight <= 0`). Division-free; compaction is in place.
    pub fn decay(&mut self) {
        if self.decay_shift == 0 {
            return;
        }
        let shift = self.decay_shift;
        let mut write = 0usize;
        for read in 0..self.len {
            let decayed = self.slots[read].weight.raw() >> shift;
            if decayed > 0 {
                self.slots[write] = SegmentSlot {
                    key: self.slots[read].key,
                    weight: ScoreQ::from_raw(decayed),
                };
                write += 1;
            }
        }
        for slot in &mut self.slots[write..self.len] {
            *slot = SegmentSlot::default();
        }
        self.len = write;
    }

    /// The eviction victim under the canonical order: least weight, ties broken
    /// by the *larger* key (so the lower id is retained).
    fn victim_index(&self) -> usize {
        let mut best = 0usize;
        let mut i = 1usize;
        while i < self.len {
            let s = self.slots[i];
            let b = self.slots[best];
            if s.weight.raw() < b.weight.raw()
                || (s.weight.raw() == b.weight.raw() && s.key > b.key)
            {
                best = i;
            }
            i += 1;
        }
        best
    }

    /// Insert or refresh a slot with a saturating `ScoreQ` add; returns the
    /// evicted key when the ring was full and the new key displaced a victim.
    pub fn upsert(&mut self, key: u32, add: ScoreQ) -> Option<u32> {
        if CAP == 0 {
            return None;
        }
        for i in 0..self.len {
            if self.slots[i].key == key {
                let weight = self.slots[i].weight.raw().saturating_add(add.raw());
                self.slots[i].weight = ScoreQ::from_raw(weight);
                return None;
            }
        }
        if self.len < CAP {
            self.slots[self.len] = SegmentSlot { key, weight: add };
            self.len += 1;
            return None;
        }
        let victim = self.victim_index();
        let evicted = self.slots[victim].key;
        self.slots[victim] = SegmentSlot { key, weight: add };
        Some(evicted)
    }

    /// The decode-independent `ScoreQ` contribution to a candidate: the
    /// saturating sum of `boost` over live slots keyed by the candidate.
    pub fn contribution(&self, candidate: u32, boost: ScoreQ) -> ScoreQ {
        let mut acc: i32 = 0;
        for i in 0..self.len {
            if self.slots[i].key == candidate {
                acc = acc.saturating_add(boost.raw());
            }
        }
        ScoreQ::from_raw(acc)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeStateLevel {
    Local,
    Segment,
    Session,
}

/// Multi-timescale fixed-capacity runtime state: token state is live today;
/// local/segment/session are reserved with bounded update hooks for Phase 8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeState<
    const TOKEN_CAP: usize = TOKEN_STATE_CAPACITY,
    const LOCAL_CAP: usize = LOCAL_STATE_CAPACITY,
    const SEGMENT_CAP: usize = SEGMENT_STATE_CAPACITY,
    const SESSION_CAP: usize = SESSION_STATE_CAPACITY,
> {
    token: TokenState<TOKEN_CAP>,
    local: ReservedState<LOCAL_CAP>,
    segment: ReservedState<SEGMENT_CAP>,
    session: ReservedState<SESSION_CAP>,
}

impl<
    const TOKEN_CAP: usize,
    const LOCAL_CAP: usize,
    const SEGMENT_CAP: usize,
    const SESSION_CAP: usize,
> Default for RuntimeState<TOKEN_CAP, LOCAL_CAP, SEGMENT_CAP, SESSION_CAP>
{
    fn default() -> Self {
        Self {
            token: TokenState::default(),
            local: ReservedState::default(),
            segment: ReservedState::default(),
            session: ReservedState::default(),
        }
    }
}

impl<
    const TOKEN_CAP: usize,
    const LOCAL_CAP: usize,
    const SEGMENT_CAP: usize,
    const SESSION_CAP: usize,
> RuntimeState<TOKEN_CAP, LOCAL_CAP, SEGMENT_CAP, SESSION_CAP>
{
    pub fn token(&self) -> &TokenState<TOKEN_CAP> {
        &self.token
    }

    pub fn local(&self) -> &ReservedState<LOCAL_CAP> {
        &self.local
    }

    pub fn segment(&self) -> &ReservedState<SEGMENT_CAP> {
        &self.segment
    }

    pub fn session(&self) -> &ReservedState<SESSION_CAP> {
        &self.session
    }

    pub fn local_mut(&mut self) -> &mut ReservedState<LOCAL_CAP> {
        &mut self.local
    }

    pub fn segment_mut(&mut self) -> &mut ReservedState<SEGMENT_CAP> {
        &mut self.segment
    }

    pub fn session_mut(&mut self) -> &mut ReservedState<SESSION_CAP> {
        &mut self.session
    }

    pub fn clear_token_state(&mut self) {
        self.token.clear();
    }

    pub fn record_token(&mut self, token: u32) {
        self.token.push(token);
    }

    pub fn token_occurrences(&self, token: u32) -> usize {
        self.token.occurrences(token)
    }

    pub fn apply_update(&mut self, level: RuntimeStateLevel, update: ReservedStateUpdate) {
        match level {
            RuntimeStateLevel::Local => self.local.apply(update),
            RuntimeStateLevel::Segment => self.segment.apply(update),
            RuntimeStateLevel::Session => self.session.apply(update),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ReservedStateUpdate, RuntimeState, RuntimeStateLevel, SegmentRing, SemanticStateSlot,
        TokenState,
    };
    use alloc::vec::Vec;
    use uor_r4_graph_format::ScoreQ;

    #[test]
    fn token_state_is_fixed_capacity_fifo() {
        let mut state = TokenState::<3>::default();
        state.push(10);
        state.push(20);
        state.push(10);
        state.push(30);
        state.push(40);

        assert_eq!(state.capacity(), 3);
        assert_eq!(state.len(), 3);
        assert_eq!(state.as_slice(), &[10, 30, 40]);
        assert_eq!(state.occurrences(10), 1);
        assert_eq!(state.occurrences(20), 0);
    }

    #[test]
    fn runtime_state_reserves_each_timescale_with_update_hooks() {
        let mut state = RuntimeState::<4, 2, 2, 2>::default();
        let local = ReservedStateUpdate {
            program_id: 7,
            slot: SemanticStateSlot {
                region_id: 1,
                token: 11,
                score_q: ScoreQ::from_raw(5),
                age: 0,
            },
        };
        let segment = ReservedStateUpdate {
            program_id: 9,
            slot: SemanticStateSlot {
                region_id: 2,
                token: 12,
                score_q: ScoreQ::from_raw(6),
                age: 1,
            },
        };
        let session = ReservedStateUpdate {
            program_id: 11,
            slot: SemanticStateSlot {
                region_id: 3,
                token: 13,
                score_q: ScoreQ::from_raw(7),
                age: 2,
            },
        };

        state.apply_update(RuntimeStateLevel::Local, local);
        state.apply_update(RuntimeStateLevel::Segment, segment);
        state.apply_update(RuntimeStateLevel::Session, session);

        assert_eq!(state.local().capacity(), 2);
        assert_eq!(state.segment().capacity(), 2);
        assert_eq!(state.session().capacity(), 2);
        assert_eq!(state.local().last_program_id(), Some(7));
        assert_eq!(state.segment().last_program_id(), Some(9));
        assert_eq!(state.session().last_program_id(), Some(11));
        assert_eq!(state.local().as_slice(), &[local.slot]);
        assert_eq!(state.segment().as_slice(), &[segment.slot]);
        assert_eq!(state.session().as_slice(), &[session.slot]);
    }

    /// Oracle mirroring the #835 reference segment-lane ring semantics
    /// (`docs/prompt_state_spec_835.md` §6): a `Vec`-backed ring with identical
    /// decay / eviction / upsert / contribution rules. The hot-path
    /// `SegmentRing` must match it bit-for-bit (reference-to-packed teeth).
    #[derive(Clone)]
    struct Oracle {
        cap: usize,
        decay_shift: u32,
        slots: Vec<(u32, i32)>,
    }

    impl Oracle {
        fn new(cap: usize, decay_shift: u32) -> Self {
            Self {
                cap,
                decay_shift,
                slots: Vec::new(),
            }
        }
        fn decay(&mut self) {
            if self.decay_shift > 0 {
                for s in &mut self.slots {
                    s.1 >>= self.decay_shift;
                }
                self.slots.retain(|s| s.1 > 0);
            }
        }
        fn victim(&self) -> usize {
            let mut best = 0usize;
            for i in 1..self.slots.len() {
                let s = self.slots[i];
                let b = self.slots[best];
                if s.1 < b.1 || (s.1 == b.1 && s.0 > b.0) {
                    best = i;
                }
            }
            best
        }
        fn upsert(&mut self, key: u32, add: i32) -> Option<u32> {
            if let Some(s) = self.slots.iter_mut().find(|s| s.0 == key) {
                s.1 = s.1.saturating_add(add);
                return None;
            }
            if self.slots.len() < self.cap {
                self.slots.push((key, add));
                return None;
            }
            let v = self.victim();
            let evicted = self.slots[v].0;
            self.slots[v] = (key, add);
            Some(evicted)
        }
        fn contribution(&self, candidate: u32, boost: i32) -> i32 {
            let mut acc = 0i32;
            for s in &self.slots {
                if s.0 == candidate {
                    acc = acc.saturating_add(boost);
                }
            }
            acc
        }
        fn sorted(&self) -> Vec<(u32, i32)> {
            let mut v = self.slots.clone();
            v.sort_unstable();
            v
        }
    }

    fn ring_sorted<const CAP: usize>(ring: &SegmentRing<CAP>) -> Vec<(u32, i32)> {
        let mut v: Vec<(u32, i32)> = ring
            .as_slice()
            .iter()
            .map(|s| (s.key, s.weight.raw()))
            .collect();
        v.sort_unstable();
        v
    }

    /// The hot-path `SegmentRing` reproduces the reference ring bit-for-bit
    /// across a long deterministic fold/decay stream: identical contents,
    /// eviction returns, length, and per-candidate contributions at every step,
    /// for a range of decay shifts. Key space (16) < capacity is exceeded so
    /// eviction and collisions are exercised.
    #[test]
    fn segment_ring_matches_reference_semantics() {
        const CAP: usize = 8;
        const KEYS: u32 = 16; // power of two so we mask instead of dividing
        let base_w: i32 = 1 << 12;
        let boost: i32 = 1 << 20;

        for &decay_shift in &[0u32, 1, 2] {
            let mut ring = SegmentRing::<CAP>::with_decay(decay_shift);
            let mut oracle = Oracle::new(CAP, decay_shift);
            // deterministic xorshift key stream (shifts + xor, no mul/div)
            let mut rng: u32 = 0x9E37_79B9 ^ decay_shift.wrapping_add(1);

            for step in 0..4000u32 {
                if (step & 7) == 7 {
                    ring.decay();
                    oracle.decay();
                } else {
                    rng ^= rng << 13;
                    rng ^= rng >> 17;
                    rng ^= rng << 5;
                    let key = rng & (KEYS - 1);
                    let ev_ring = ring.upsert(key, ScoreQ::from_raw(base_w));
                    let ev_oracle = oracle.upsert(key, base_w);
                    assert_eq!(ev_ring, ev_oracle, "eviction differs at step {step}");
                }
                assert_eq!(ring.len(), oracle.slots.len(), "len differs at step {step}");
                assert_eq!(
                    ring_sorted(&ring),
                    oracle.sorted(),
                    "contents differ at step {step}"
                );
                for candidate in 0..KEYS {
                    assert_eq!(
                        ring.contribution(candidate, ScoreQ::from_raw(boost)).raw(),
                        oracle.contribution(candidate, boost),
                        "contribution differs for candidate {candidate} at step {step}"
                    );
                }
            }
            // capacity is honored throughout
            assert!(ring.len() <= CAP);
        }
    }

    #[test]
    fn segment_ring_zero_capacity_declines_without_panic() {
        let mut ring = SegmentRing::<0>::with_decay(1);
        assert_eq!(ring.upsert(5, ScoreQ::from_raw(1)), None);
        assert_eq!(ring.len(), 0);
        ring.decay();
        assert_eq!(ring.contribution(5, ScoreQ::from_raw(1 << 20)).raw(), 0);
    }
}
